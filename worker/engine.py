"""
nexus-worker — Task execution engine.

Main loop:
1. Heartbeat → get model instructions
2. Pull model if needed (ollama pull)
3. Pull task from server
4. Execute via Ollama local
5. Submit result to server
6. Repeat

Handles graceful pause/resume, error backoff, and model transitions.
"""

from __future__ import annotations

import asyncio
import subprocess
import time
from enum import Enum
from typing import Any, Callable, Optional

import httpx
from loguru import logger

from worker.client import NexusClient
from worker.config import load_config

try:
    from nexus.compute.crypto import sign_result as _sign_result
    HAS_SIGNING = True
except ImportError:
    HAS_SIGNING = False
    def _sign_result(*args, **kwargs) -> str:
        return ""


class WorkerState(str, Enum):
    IDLE = "idle"
    PULLING_MODEL = "pulling_model"
    PROCESSING = "processing"
    PAUSED = "paused"
    ERROR = "error"
    STOPPED = "stopped"


class WorkerEngine:
    """Core task execution engine for a GPU contributor node.

    Manages the heartbeat → pull → execute → submit loop,
    model management, and graceful lifecycle.
    """

    def __init__(
        self,
        client: NexusClient,
        ollama_url: str = "http://localhost:11434",
        poll_interval: float = 2.0,
        heartbeat_interval: float = 15.0,
        node_id: str = "",
        private_key_pem: bytes = b"",
        on_state_change: Optional[Callable] = None,
        on_task_complete: Optional[Callable] = None,
        on_stats_update: Optional[Callable] = None,
    ) -> None:
        self._client = client
        self._ollama_url = ollama_url
        self._node_id = node_id
        self._private_key_pem = private_key_pem
        self._poll_interval = poll_interval
        self._heartbeat_interval = heartbeat_interval

        # Callbacks for dashboard updates
        self._on_state_change = on_state_change
        self._on_task_complete = on_task_complete
        self._on_stats_update = on_stats_update

        # State
        self._state = WorkerState.IDLE
        self._current_model: str = ""
        self._current_task: Optional[dict] = None
        self._running = False
        self._paused = False
        self._pulling_model: str = ""  # Model currently being pulled (prevents duplicates)
        self._state_before_pause: WorkerState = WorkerState.IDLE

        # Stats
        self.session_tasks: int = 0
        self.session_errors: int = 0
        self.session_start: float = 0.0
        self.total_tokens: int = 0
        self.last_tokens_per_sec: float = 0.0
        self.network_stats: dict = {}
        self.leaderboard: list = []

        # Background tasks
        self._main_task: Optional[asyncio.Task] = None
        self._heartbeat_task: Optional[asyncio.Task] = None

        # Backoff
        self._consecutive_errors = 0
        self._max_backoff = 60.0

    @property
    def state(self) -> WorkerState:
        return self._state

    @property
    def current_model(self) -> str:
        return self._current_model

    @property
    def current_task(self) -> Optional[dict]:
        return self._current_task

    @property
    def uptime_seconds(self) -> float:
        if self.session_start > 0:
            return time.time() - self.session_start
        return 0.0

    def _set_state(self, state: WorkerState) -> None:
        self._state = state
        if self._on_state_change:
            self._on_state_change(state)

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start the worker engine."""
        if self._running:
            return
        self._running = True
        self.session_start = time.time()
        self._set_state(WorkerState.IDLE)

        self._heartbeat_task = asyncio.create_task(self._heartbeat_loop())
        self._main_task = asyncio.create_task(self._main_loop())

        logger.info("Worker engine started")

    async def stop(self) -> None:
        """Stop the worker engine gracefully."""
        self._running = False
        self._set_state(WorkerState.STOPPED)

        for task in (self._main_task, self._heartbeat_task):
            if task and not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass

        await self._client.close()
        logger.info("Worker engine stopped (tasks: {}, errors: {})", self.session_tasks, self.session_errors)

    def pause(self) -> None:
        """Pause task processing after the current task finishes."""
        self._paused = True
        self._state_before_pause = self._state
        self._set_state(WorkerState.PAUSED)
        logger.info("Worker paused (current task will finish)")

    def resume(self) -> None:
        """Resume task processing, restoring previous state."""
        self._paused = False
        restore = self._state_before_pause
        if restore in (WorkerState.PAUSED, WorkerState.STOPPED, WorkerState.ERROR):
            restore = WorkerState.IDLE
        self._set_state(restore)
        logger.info("Worker resumed")

    # ------------------------------------------------------------------
    # Heartbeat loop
    # ------------------------------------------------------------------

    async def _heartbeat_loop(self) -> None:
        """Send periodic heartbeats to the server."""
        while self._running:
            try:
                status = "idle" if self._state in (WorkerState.IDLE, WorkerState.PAUSED) else "busy"
                resp = await self._client.heartbeat(
                    current_model=self._current_model,
                    status=status,
                )

                # Check if server wants us to pull a new model
                model_required = resp.get("model_required", "")
                message = resp.get("message", "")

                if (
                    message.startswith("pull_model:")
                    and model_required != self._current_model
                    and model_required != self._pulling_model
                ):
                    asyncio.create_task(self._pull_model(model_required))

                # Periodically fetch network stats
                try:
                    self.network_stats = await self._client.get_stats()
                    lb = await self._client.get_leaderboard(limit=10)
                    self.leaderboard = lb.get("entries", [])
                    if self._on_stats_update:
                        self._on_stats_update(self.network_stats, self.leaderboard)
                except Exception:
                    pass

                self._consecutive_errors = 0

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("Heartbeat error: {}", exc)
                self._consecutive_errors += 1

            await asyncio.sleep(self._heartbeat_interval)

    # ------------------------------------------------------------------
    # Model management
    # ------------------------------------------------------------------

    async def _pull_model(self, model: str) -> None:
        """Pull a model via Ollama."""
        if self._pulling_model:
            return  # Already pulling

        self._pulling_model = model
        self._set_state(WorkerState.PULLING_MODEL)
        logger.info("Pulling model: {}", model)

        try:
            async with httpx.AsyncClient(base_url=self._ollama_url, timeout=600.0) as client:
                # Check if model already exists
                try:
                    resp = await client.post("/api/show", json={"name": model})
                    if resp.status_code == 200:
                        logger.info("Model {} already available", model)
                        self._current_model = model
                        self._pulling_model = ""
                        await self._client.report_model_ready(model)
                        self._set_state(WorkerState.IDLE)
                        return
                except Exception:
                    pass

                # Pull the model (streaming response)
                async with client.stream("POST", "/api/pull", json={"name": model}) as resp:
                    async for line in resp.aiter_lines():
                        if not self._running:
                            break

            self._current_model = model
            await self._client.report_model_ready(model)
            logger.info("Model {} pulled successfully", model)

        except Exception as exc:
            logger.error("Failed to pull model {}: {}", model, exc)

        self._pulling_model = ""
        self._set_state(WorkerState.IDLE)

    async def _get_model_digest(self, model: str) -> str:
        """Get SHA256 digest of an Ollama model."""
        try:
            async with httpx.AsyncClient(base_url=self._ollama_url, timeout=10.0) as client:
                resp = await client.post("/api/show", json={"name": model})
                if resp.status_code == 200:
                    return resp.json().get("digest", "")
        except Exception:
            pass
        return ""

    # ------------------------------------------------------------------
    # Main task loop
    # ------------------------------------------------------------------

    async def _main_loop(self) -> None:
        """Main loop: pull task → execute → submit result."""
        # Wait for first heartbeat
        await asyncio.sleep(2)

        while self._running:
            try:
                if self._paused:
                    await asyncio.sleep(1)
                    continue

                if self._state == WorkerState.PULLING_MODEL:
                    await asyncio.sleep(2)
                    continue

                if not self._current_model:
                    await asyncio.sleep(self._poll_interval)
                    continue

                # Pull next task
                task = await self._client.pull_task()

                if task is None:
                    # No task available — wait and retry
                    self._set_state(WorkerState.IDLE)
                    await asyncio.sleep(self._poll_interval)
                    continue

                # Execute the task
                self._current_task = task
                self._set_state(WorkerState.PROCESSING)

                result = await self._execute_task(task)

                if result:
                    # Submit result
                    resp = await self._client.submit_result(**result)

                    if resp.get("accepted"):
                        self.session_tasks += 1
                        self._consecutive_errors = 0
                        if self._on_task_complete:
                            self._on_task_complete(task, result, resp)
                    else:
                        logger.warning("Result rejected: {}", resp.get("message", ""))
                        self.session_errors += 1
                else:
                    # Execution failed (timeout, Ollama error)
                    self.session_errors += 1
                    self._consecutive_errors += 1

                self._current_task = None
                self._set_state(WorkerState.IDLE)

                # Backoff if execution failed
                if not result and self._consecutive_errors > 0:
                    await self._backoff()

            except asyncio.CancelledError:
                break
            except httpx.HTTPStatusError as exc:
                logger.warning("Server error: {} {}", exc.response.status_code, exc.response.text[:200])
                self.session_errors += 1
                self._consecutive_errors += 1
                await self._backoff()
            except Exception as exc:
                logger.error("Task loop error: {}", exc)
                self.session_errors += 1
                self._consecutive_errors += 1
                await self._backoff()

    async def _execute_task(self, task: dict) -> Optional[dict]:
        """Execute an LLM task via local Ollama.

        Returns a dict ready for submit_result(), or None on failure.
        """
        task_id = task["task_id"]
        prompt = task["prompt"]
        system_prompt = task.get("system_prompt", "")
        model = task.get("model", "") or self._current_model
        timeout = task.get("timeout_seconds", 300)

        logger.info("Processing task {} (type: {}, model: {})", task_id[:8], task.get("task_type", "?"), model)

        start_ms = int(time.time() * 1000)

        try:
            payload: dict[str, Any] = {
                "model": model,
                "prompt": prompt,
                "stream": False,
            }
            if system_prompt:
                payload["system"] = system_prompt

            async with httpx.AsyncClient(base_url=self._ollama_url, timeout=float(timeout)) as client:
                resp = await client.post("/api/generate", json=payload)
                resp.raise_for_status()
                data = resp.json()

            end_ms = int(time.time() * 1000)
            generation_time_ms = end_ms - start_ms

            result_text = data.get("response", "")
            eval_count = data.get("eval_count", 0)
            eval_duration = data.get("eval_duration", 0)

            # Tokens/sec from Ollama's own measurement
            if eval_duration > 0:
                self.last_tokens_per_sec = eval_count / (eval_duration / 1e9)
            elif generation_time_ms > 0 and eval_count > 0:
                self.last_tokens_per_sec = eval_count / (generation_time_ms / 1000)

            self.total_tokens += eval_count

            # Get model digest
            digest = await self._get_model_digest(model)

            # Ed25519 sign the result (Couche 1)
            sig = ""
            if self._private_key_pem:
                sig = _sign_result(self._private_key_pem, task_id, result_text, digest, self._node_id)

            logger.info(
                "Task {} done ({} tokens, {:.1f}s, {:.1f} tok/s{})",
                task_id[:8], eval_count, generation_time_ms / 1000, self.last_tokens_per_sec,
                ", signed" if sig else "",
            )

            return {
                "task_id": task_id,
                "result_text": result_text,
                "tokens_generated": eval_count,
                "generation_time_ms": generation_time_ms,
                "model_digest": digest,
                "signature": sig,
            }

        except httpx.TimeoutException:
            logger.warning("Task {} timed out ({}s)", task_id[:8], timeout)
            return None
        except Exception as exc:
            logger.error("Task {} execution failed: {}", task_id[:8], exc)
            return None

    async def _backoff(self) -> None:
        """Exponential backoff on consecutive errors."""
        delay = min(2 ** self._consecutive_errors, self._max_backoff)
        logger.debug("Backing off for {:.1f}s (errors: {})", delay, self._consecutive_errors)
        await asyncio.sleep(delay)
