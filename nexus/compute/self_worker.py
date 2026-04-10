"""
NEXUS Compute -- Embedded Self-Worker.

The server automatically contributes its own GPU to the compute network.
No external CLI needed — the person who installs NEXUS is automatically
the first contributor.

Lifecycle:
1. On startup, auto-registers as a compute node (if not already registered)
2. Detects local GPU via nvidia-smi/pynvml
3. Runs the task loop: pull → Ollama generate → submit result
4. Can be paused/resumed via API button in the frontend

This replaces the need for `pip install nexus-worker` + `nexus-worker register`
for the server operator. External contributors still use the CLI.
"""

from __future__ import annotations

import asyncio
import hashlib
import platform
import shutil
import subprocess
import time
from typing import Any, Optional

import httpx
from loguru import logger

from nexus.compute.db import ComputeDatabase
from nexus.config import settings
from nexus.engine import get_db


class SelfWorker:
    """Embedded GPU worker that runs inside the server process.

    Auto-registers on first start, then loops: pull task → Ollama → submit.
    """

    def __init__(self) -> None:
        self._node_id: str = ""
        self._api_key: str = ""
        self._gpu_model: str = ""
        self._vram_mb: int = 0
        self._current_model: str = ""
        self._running = False
        self._paused = False
        self._task: Optional[asyncio.Task] = None
        self._heartbeat_task: Optional[asyncio.Task] = None

        # Stats
        self.tasks_completed: int = 0
        self.tasks_errored: int = 0
        self.last_tokens_per_sec: float = 0.0

    @property
    def running(self) -> bool:
        return self._running

    @property
    def paused(self) -> bool:
        return self._paused

    @property
    def node_id(self) -> str:
        return self._node_id

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Start the self-worker: detect GPU, register, begin task loop."""
        if self._running:
            return

        # Detect GPU
        gpu = self._detect_gpu()
        self._gpu_model = gpu["gpu_model"]
        self._vram_mb = gpu["vram_mb"]

        if self._vram_mb == 0:
            logger.warning("SelfWorker: no GPU detected — disabled")
            return

        # Auto-register (or find existing self-worker node)
        await self._ensure_registered()

        if not self._node_id:
            logger.warning("SelfWorker: registration failed — disabled")
            return

        self._running = True
        self._task = asyncio.create_task(self._task_loop())
        self._heartbeat_task = asyncio.create_task(self._heartbeat_loop())

        logger.info(
            "SelfWorker started — {} ({} MB VRAM), node: {}",
            self._gpu_model, self._vram_mb, self._node_id[:8],
        )

    async def stop(self) -> None:
        """Stop the self-worker."""
        self._running = False
        for t in (self._task, self._heartbeat_task):
            if t and not t.done():
                t.cancel()
                try:
                    await t
                except asyncio.CancelledError:
                    pass

        # Mark offline
        if self._node_id:
            try:
                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    await db.update_node_status(self._node_id, "offline")
                    await db.log_disconnect(self._node_id)
            except Exception:
                pass

        logger.info("SelfWorker stopped (tasks: {}, errors: {})", self.tasks_completed, self.tasks_errored)

    def pause(self) -> None:
        self._paused = True
        logger.info("SelfWorker paused")

    def resume(self) -> None:
        self._paused = False
        logger.info("SelfWorker resumed")

    # ------------------------------------------------------------------
    # GPU detection (inline, no external dependency)
    # ------------------------------------------------------------------

    @staticmethod
    def _detect_gpu() -> dict:
        """Detect GPU model and VRAM."""
        # Try pynvml
        try:
            import pynvml
            pynvml.nvmlInit()
            handle = pynvml.nvmlDeviceGetHandleByIndex(0)
            name = pynvml.nvmlDeviceGetName(handle)
            if isinstance(name, bytes):
                name = name.decode("utf-8")
            mem = pynvml.nvmlDeviceGetMemoryInfo(handle)
            vram_mb = mem.total // (1024 * 1024)
            pynvml.nvmlShutdown()
            return {"gpu_model": name, "vram_mb": vram_mb}
        except Exception:
            pass

        # Try nvidia-smi
        if shutil.which("nvidia-smi"):
            try:
                r = subprocess.run(
                    ["nvidia-smi", "--query-gpu=name,memory.total", "--format=csv,noheader,nounits"],
                    capture_output=True, text=True, timeout=10,
                )
                if r.returncode == 0:
                    parts = [p.strip() for p in r.stdout.strip().split(",")]
                    if len(parts) >= 2:
                        return {"gpu_model": parts[0], "vram_mb": int(float(parts[1]))}
            except Exception:
                pass

        return {"gpu_model": "Unknown", "vram_mb": 0}

    # ------------------------------------------------------------------
    # Auto-registration
    # ------------------------------------------------------------------

    async def _ensure_registered(self) -> None:
        """Register as a compute node if not already done."""
        async with get_db() as conn:
            db = ComputeDatabase(conn)

            # Check if self-worker already exists (by name convention)
            nodes = await db.list_nodes()
            for n in nodes:
                if n.get("name") == "_self_worker_":
                    self._node_id = n["id"]
                    # Update GPU info in case it changed
                    await db.heartbeat(self._node_id, self._current_model)
                    await db.update_node_status(self._node_id, "idle")
                    await db.log_connect(self._node_id)
                    logger.info("SelfWorker: reconnected as existing node {}", self._node_id[:8])
                    return

            # Register new self-worker node
            node, api_key = await db.register_node(
                name="_self_worker_",
                gpu_model=self._gpu_model,
                vram_mb=self._vram_mb,
                ip="127.0.0.1",
                platform=platform.system().lower(),
            )
            self._node_id = node["id"]
            self._api_key = api_key
            await db.log_connect(self._node_id)
            logger.info("SelfWorker: registered as new node {}", self._node_id[:8])

    # ------------------------------------------------------------------
    # Heartbeat loop
    # ------------------------------------------------------------------

    async def _heartbeat_loop(self) -> None:
        """Send periodic heartbeats to keep the node alive."""
        while self._running:
            try:
                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    await db.heartbeat(self._node_id, self._current_model)
                    status = "idle" if not self._paused else "idle"
                    await db.update_node_status(self._node_id, status)
            except Exception:
                pass
            await asyncio.sleep(15)

    # ------------------------------------------------------------------
    # Task loop (direct DB access, no HTTP round-trip)
    # ------------------------------------------------------------------

    async def _task_loop(self) -> None:
        """Main loop: pull task from DB → execute via Ollama → store result."""
        await asyncio.sleep(3)  # Let system settle

        while self._running:
            try:
                if self._paused:
                    await asyncio.sleep(2)
                    continue

                # Pull next task directly from DB
                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    task = await db.pull_next_task(
                        self._node_id, model=self._current_model,
                    )

                if not task:
                    await asyncio.sleep(2)
                    continue

                # Mark node busy
                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    await db.update_node_status(self._node_id, "busy")

                # Execute via Ollama
                result = await self._execute_task(task)

                if result:
                    # Store result directly in DB
                    async with get_db() as conn:
                        db = ComputeDatabase(conn)
                        await db.store_result(
                            task_id=task["id"],
                            node_id=self._node_id,
                            result_text=result["text"],
                            tokens_generated=result.get("tokens", 0),
                            generation_time_ms=result.get("duration_ms", 0),
                            model_digest=result.get("digest", ""),
                        )
                        await db.complete_task(task["id"], result["text"], validated=True)
                        await db.increment_node_stats(
                            self._node_id, completed=1,
                            tokens_per_sec=result.get("tokens_per_sec"),
                        )
                        await db.update_node_trust(self._node_id, 1)
                        await db.update_node_status(self._node_id, "idle")

                    self.tasks_completed += 1
                else:
                    async with get_db() as conn:
                        db = ComputeDatabase(conn)
                        await db.fail_task(task["id"], "Ollama execution failed")
                        await db.increment_node_stats(self._node_id, errored=1)
                        await db.update_node_status(self._node_id, "idle")
                    self.tasks_errored += 1

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("SelfWorker task loop error: {}", exc)
                self.tasks_errored += 1
                await asyncio.sleep(5)

    async def _execute_task(self, task: dict) -> Optional[dict]:
        """Execute an LLM task via local Ollama."""
        prompt = task.get("prompt", "")
        system_prompt = task.get("system_prompt", "")
        model = task.get("model", "") or settings.model_fast
        timeout = task.get("timeout_seconds", 300)

        self._current_model = model

        payload: dict[str, Any] = {"model": model, "prompt": prompt, "stream": False}
        if system_prompt:
            payload["system"] = system_prompt

        start_ms = int(time.time() * 1000)

        try:
            async with httpx.AsyncClient(
                base_url=settings.ollama_base_url, timeout=float(timeout),
            ) as client:
                resp = await client.post("/api/generate", json=payload)
                resp.raise_for_status()
                data = resp.json()

            end_ms = int(time.time() * 1000)
            duration_ms = end_ms - start_ms
            text = data.get("response", "")
            eval_count = data.get("eval_count", 0)
            eval_duration = data.get("eval_duration", 0)

            tok_s = 0.0
            if eval_duration > 0:
                tok_s = eval_count / (eval_duration / 1e9)
            elif duration_ms > 0 and eval_count > 0:
                tok_s = eval_count / (duration_ms / 1000)

            self.last_tokens_per_sec = tok_s

            return {
                "text": text,
                "tokens": eval_count,
                "duration_ms": duration_ms,
                "tokens_per_sec": tok_s if tok_s > 0 else None,
                "digest": "",
            }

        except Exception as exc:
            logger.debug("SelfWorker Ollama error: {}", exc)
            return None

    # ------------------------------------------------------------------
    # Status
    # ------------------------------------------------------------------

    def get_status(self) -> dict:
        return {
            "running": self._running,
            "paused": self._paused,
            "node_id": self._node_id,
            "gpu_model": self._gpu_model,
            "vram_mb": self._vram_mb,
            "current_model": self._current_model,
            "tasks_completed": self.tasks_completed,
            "tasks_errored": self.tasks_errored,
            "last_tokens_per_sec": self.last_tokens_per_sec,
        }
