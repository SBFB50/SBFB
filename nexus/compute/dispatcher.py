"""
NEXUS Compute -- Task Dispatcher.

Manages the lifecycle of GPU tasks:
- Creates tasks from worker requests
- Assigns tasks to available nodes (priority + model affinity)
- Validates results (basic + spot-check)
- Reaps stale/expired tasks
- Monitors node heartbeats and marks offline nodes
"""

from __future__ import annotations

import asyncio
import random
from datetime import datetime, timezone, timedelta
from typing import Any, Optional

from loguru import logger

from nexus.compute.db import ComputeDatabase
from nexus.compute.events import ComputeEventType
from nexus.compute.model_selector import ModelSelector, MODEL_TIERS
from nexus.compute.verification import ResultVerifier
from nexus.config import settings
from nexus.engine import get_db, NexusEvent


class TaskDispatcher:
    """Central dispatcher for distributed GPU compute tasks.

    Runs as a singleton on the server. Manages task creation, assignment,
    validation, and lifecycle.
    """

    def __init__(
        self,
        bus: Any = None,
        model_selector: Optional[ModelSelector] = None,
    ) -> None:
        self._bus = bus
        self._model_selector = model_selector
        self._running = False
        self._reaper_task: Optional[asyncio.Task] = None
        self._heartbeat_task: Optional[asyncio.Task] = None

    @property
    def current_model(self) -> str:
        if self._model_selector:
            return self._model_selector.target_model
        return ""

    @property
    def current_tier(self) -> str:
        if self._model_selector:
            return self._model_selector.target_tier
        return ""

    @property
    def model_selector(self) -> Optional[ModelSelector]:
        return self._model_selector

    @property
    def running(self) -> bool:
        return self._running

    async def start(self) -> None:
        """Start background tasks (reaper, heartbeat monitor)."""
        if self._running:
            return
        self._running = True

        # Start background reaper (cleans stale tasks)
        self._reaper_task = asyncio.create_task(self._reaper_loop())
        # Start heartbeat monitor (marks offline nodes)
        self._heartbeat_task = asyncio.create_task(self._heartbeat_monitor())

        logger.info("TaskDispatcher started (model: {}, tier: {})", self.current_model, self.current_tier)

    async def stop(self) -> None:
        """Stop background tasks."""
        self._running = False
        for task in (self._reaper_task, self._heartbeat_task):
            if task and not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
        logger.info("TaskDispatcher stopped")

    async def recalculate_model(self) -> str:
        """Delegate model recalculation to ModelSelector."""
        if self._model_selector:
            status = await self._model_selector.recalculate()
            return status.get("target_model", "")
        return ""

    # ------------------------------------------------------------------
    # Task creation (called by GOV workers or API)
    # ------------------------------------------------------------------

    async def submit_task(
        self,
        task_type: str,
        prompt: str,
        system_prompt: str = "",
        priority: int = 5,
        timeout_seconds: int = 300,
        source_worker: str = "",
        require_logprobs: bool = False,
        max_retries: int = 3,
        metadata: Optional[dict] = None,
    ) -> dict:
        """Submit a new LLM task to the distributed queue.

        The model is auto-selected based on the current network capacity
        and transition state (urgent tasks may use the previous model).
        """
        if self._model_selector:
            model = self._model_selector.get_task_model(task_type, priority)
            if not model:
                model = self._model_selector.target_model or MODEL_TIERS[1]["model"]
            exec_mode = self._model_selector.get_task_execution_mode(task_type, priority)
        else:
            model = MODEL_TIERS[1]["model"]
            exec_mode = "local"

        async with get_db() as conn:
            db = ComputeDatabase(conn)
            task = await db.create_task(
                task_type=task_type,
                prompt=prompt,
                system_prompt=system_prompt,
                model=model,
                priority=priority,
                timeout_seconds=timeout_seconds,
                source_worker=source_worker,
                require_logprobs=require_logprobs,
                max_retries=max_retries,
                execution_mode=exec_mode.value if hasattr(exec_mode, "value") else str(exec_mode),
                metadata=metadata,
            )

        if self._bus:
            await self._bus.publish(NexusEvent(
                event_type=ComputeEventType.COMPUTE_TASK_CREATED,
                case_id="compute",
                payload={"task_id": task["id"], "task_type": task_type, "priority": priority},
                source_worker="task_dispatcher",
            ))

        return task

    # ------------------------------------------------------------------
    # Result validation
    # ------------------------------------------------------------------

    async def validate_result(
        self,
        task_id: str,
        node_id: str,
        result_text: str,
        tokens_generated: int = 0,
        generation_time_ms: int = 0,
        model_digest: str = "",
        logprobs: str = "",
        signature: str = "",
    ) -> dict:
        """Validate and store a task result using 3-layer verification.

        Layer 1: Ed25519 signature (identity)
        Layer 2: Model digest whitelist (model loaded)
        Layer 3: Logprob fingerprinting (model actually used)
        + BOINC-style spot-checking (safety net)

        Returns {"accepted": bool, "message": str, "trust_delta": int}.
        """
        async with get_db() as conn:
            db = ComputeDatabase(conn)

            task = await db.get_task(task_id)
            if not task:
                return {"accepted": False, "message": "Task not found", "trust_delta": 0}

            if task.get("assigned_to") != node_id:
                return {"accepted": False, "message": "Task not assigned to this node", "trust_delta": 0}

            if task.get("status") != "assigned":
                return {"accepted": False, "message": f"Task status is '{task.get('status')}', expected 'assigned'", "trust_delta": 0}

            node = await db.get_node(node_id)
            if not node:
                return {"accepted": False, "message": "Node not found", "trust_delta": 0}

            # Basic validation: non-empty result
            if not result_text or len(result_text.strip()) < 5:
                await db.fail_task(task_id, "Empty or too short result")
                await db.increment_node_stats(node_id, errored=1)
                await db.update_node_trust(node_id, -5)
                return {"accepted": False, "message": "Result too short", "trust_delta": -5}

            # --- 3-Layer Proof-of-Computation Verification ---
            verifier = ResultVerifier()

            # Parse logprobs JSON if provided
            parsed_logprobs = None
            if logprobs:
                try:
                    import json
                    parsed_logprobs = json.loads(logprobs)
                except (json.JSONDecodeError, TypeError):
                    parsed_logprobs = None

            verification = verifier.verify(
                task_id=task_id,
                node_id=node_id,
                result_text=result_text,
                model=task.get("model", ""),
                model_digest=model_digest,
                signature_b64=signature,
                public_key_pem=node.get("public_key", "").encode() if node.get("public_key") else b"",
                calibration_prompt=task.get("calibration_prompt", ""),
                logprobs=parsed_logprobs,
            )

            trust_delta = verification["trust_delta"]

            # Auto-ban on critical verification failure
            if verification["ban"]:
                await db.ban_node(node_id)
                await db.fail_task(task_id, "Verification failed: " + str(verification["checks"]))
                logger.warning("Node {} BANNED — verification failed: {}", node_id[:8], verification["checks"])
                return {"accepted": False, "message": "Verification failed", "trust_delta": trust_delta}

            if not verification["passed"]:
                await db.fail_task(task_id, "Verification checks failed")
                await db.increment_node_stats(node_id, errored=1)
                await db.update_node_trust(node_id, trust_delta)
                return {"accepted": False, "message": "Verification failed", "trust_delta": trust_delta}

            # --- All checks passed — store and accept ---

            # Store the result
            await db.store_result(
                task_id=task_id,
                node_id=node_id,
                result_text=result_text,
                tokens_generated=tokens_generated,
                generation_time_ms=generation_time_ms,
                model_digest=model_digest,
                logprobs=logprobs,
                signature=signature,
            )

            # Calculate tokens/sec
            tokens_per_sec = None
            if generation_time_ms > 0 and tokens_generated > 0:
                tokens_per_sec = tokens_generated / (generation_time_ms / 1000)

            # Mark task as completed
            await db.complete_task(task_id, result_text, validated=True, validation_score=1.0)

            # Update node stats and trust
            await db.increment_node_stats(
                node_id, completed=1, tokens_per_sec=tokens_per_sec,
            )
            await db.update_node_trust(node_id, trust_delta)

            # Update node status back to idle
            await db.update_node_status(node_id, "idle")

            # BOINC-style spot-checking
            if verifier.spot_check_needed(node.get("trust_score", 50)) and self._bus:
                await self._bus.publish(NexusEvent(
                    event_type=ComputeEventType.COMPUTE_SPOT_CHECK_NEEDED,
                    case_id="compute",
                    payload={
                        "task_id": task_id,
                        "node_id": node_id,
                        "result_text": result_text[:500],
                        "prompt": task.get("prompt", "")[:500],
                    },
                    source_worker="task_dispatcher",
                ))

        if self._bus:
            await self._bus.publish(NexusEvent(
                event_type=ComputeEventType.COMPUTE_TASK_COMPLETED,
                case_id="compute",
                payload={
                    "task_id": task_id,
                    "node_id": node_id,
                    "task_type": task.get("task_type", ""),
                    "tokens_generated": tokens_generated,
                    "generation_time_ms": generation_time_ms,
                    "verification": verification["checks"],
                },
                source_worker="task_dispatcher",
            ))

        return {
            "accepted": True,
            "message": "Result accepted (verified)",
            "trust_delta": trust_delta,
        }

    @staticmethod
    def _get_spot_check_rate(trust_score: int) -> float:
        """Return spot-check probability based on trust score.

        Shared with ResultVerifier.spot_check_needed() — keep in sync.
        """
        if trust_score >= 80:
            return 0.01  # 1% for trusted nodes
        if trust_score >= 50:
            return 0.05  # 5% standard (includes default score 50)
        return 0.20  # 20% for suspect nodes

    # ------------------------------------------------------------------
    # Background: stale task reaper
    # ------------------------------------------------------------------

    async def _reaper_loop(self) -> None:
        """Periodically reset stale assigned tasks."""
        interval = settings.compute_reaper_interval
        while self._running:
            try:
                await asyncio.sleep(interval)
                if not self._running:
                    break

                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    count = await db.expire_stale_tasks()

                if count > 0:
                    logger.info("Task reaper: reset {} stale tasks", count)
                    if self._bus:
                        await self._bus.publish(NexusEvent(
                            event_type=ComputeEventType.COMPUTE_TASK_EXPIRED,
                            case_id="compute",
                            payload={"count": count},
                            source_worker="task_reaper",
                        ))

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("Task reaper error: {}", exc)
                await asyncio.sleep(10)

    # ------------------------------------------------------------------
    # Background: heartbeat monitor
    # ------------------------------------------------------------------

    async def _heartbeat_monitor(self) -> None:
        """Mark nodes as offline if no heartbeat beyond configured timeout."""
        timeout = settings.compute_heartbeat_timeout
        while self._running:
            try:
                await asyncio.sleep(30)
                if not self._running:
                    break

                async with get_db() as conn:
                    db = ComputeDatabase(conn)
                    nodes = await db.get_online_nodes()

                    now = datetime.now(timezone.utc)
                    disconnected = 0
                    for node in nodes:
                        last_hb = node.get("last_heartbeat")
                        if not last_hb:
                            continue

                        # Parse ISO timestamp
                        try:
                            hb_time = datetime.fromisoformat(last_hb.replace("Z", "+00:00"))
                            if hb_time.tzinfo is None:
                                hb_time = hb_time.replace(tzinfo=timezone.utc)
                        except (ValueError, AttributeError):
                            continue

                        if (now - hb_time).total_seconds() > timeout:
                            await db.update_node_status(node["id"], "offline")
                            disconnected += 1

                            # Unassign tasks from offline node
                            tasks = await db.list_tasks(
                                status="assigned", assigned_to=node["id"],
                            )
                            for task in tasks:
                                await db.fail_task(task["id"], "Node went offline")

                    if disconnected > 0:
                        logger.info("Heartbeat monitor: {} nodes marked offline", disconnected)
                        # Recalculate model after node changes
                        await self.recalculate_model()

                        if self._bus:
                            await self._bus.publish(NexusEvent(
                                event_type=ComputeEventType.COMPUTE_NODE_DISCONNECTED,
                                case_id="compute",
                                payload={"count": disconnected},
                                source_worker="heartbeat_monitor",
                            ))

            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("Heartbeat monitor error: {}", exc)
                await asyncio.sleep(10)

    # ------------------------------------------------------------------
    # Public info
    # ------------------------------------------------------------------

    def get_status(self) -> dict:
        """Return dispatcher status for health checks."""
        return {
            "running": self._running,
            "current_model": self.current_model,
            "current_tier": self.current_tier,
            "reaper_active": self._reaper_task is not None and not self._reaper_task.done() if self._reaper_task else False,
            "heartbeat_active": self._heartbeat_task is not None and not self._heartbeat_task.done() if self._heartbeat_task else False,
        }
