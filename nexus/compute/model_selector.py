"""
NEXUS Compute -- Model Selector (Auto-scaling).

Automatically selects the best LLM model based on available GPU resources:
- Global model: best model the network can collectively support
- Per-node model: best model each individual node can run locally
- Transition management: graceful model switches with mixed-mode support
- Pull tracking: knows which nodes have which models ready

Model tiers (ordered by VRAM requirement):
  0 GB  → gemma-4-12b-q4          (Basique)
  14 GB → gemma-4-26b-q4          (Standard)
  40 GB → llama-3.1-70b-q4        (Avance)
  80 GB → qwen-2.5-110b-q4        (Pro)
  150 GB → llama-3.1-405b-q2      (Ultra)
  300 GB → llama-3.1-405b         (Maximum)
"""

from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from enum import Enum
from typing import Any, Optional

from loguru import logger

from nexus.compute.db import ComputeDatabase
from nexus.compute.hybrid import ExecutionMode, HybridRouter
from nexus.config import settings
from nexus.engine import get_db

try:
    from nexus.compute.swarm import SwarmManager
    HAS_SWARM = True
except ImportError:
    HAS_SWARM = False


# ============================================================================
# Model tiers
# ============================================================================

MODEL_TIERS: list[dict[str, Any]] = [
    {"min_vram_gb": 0, "model": "gemma-4-12b-q4", "label": "Basique"},
    {"min_vram_gb": 14, "model": "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m", "label": "Standard"},
    {"min_vram_gb": 40, "model": "llama-3.1-70b-q4", "label": "Avance"},
    {"min_vram_gb": 80, "model": "qwen-2.5-110b-q4", "label": "Pro"},
    {"min_vram_gb": 150, "model": "llama-3.1-405b-q2", "label": "Ultra"},
    {"min_vram_gb": 300, "model": "llama-3.1-405b", "label": "Maximum"},
]


class TransitionState(str, Enum):
    """Model transition states."""
    STABLE = "stable"           # All nodes on target model
    TRANSITIONING = "transitioning"  # Some nodes still pulling
    DEGRADED = "degraded"       # Transition stalled (some nodes can't pull)


def get_tier_for_vram(vram_gb: float) -> dict[str, Any]:
    """Return the best model tier for a given VRAM amount."""
    best = MODEL_TIERS[0]
    for tier in MODEL_TIERS:
        if vram_gb >= tier["min_vram_gb"]:
            best = tier
    return best


def get_node_model(vram_mb: int) -> str:
    """Return the best model a single node can run based on its VRAM."""
    vram_gb = vram_mb / 1024
    return get_tier_for_vram(vram_gb)["model"]


# ============================================================================
# ModelSelector
# ============================================================================

class ModelSelector:
    """Manages model selection and transitions for the compute network.

    Responsibilities:
    - Select the global target model based on total online VRAM
    - Assign per-node models based on individual VRAM
    - Track model readiness per node (who has pulled what)
    - Manage graceful transitions (mixed mode during pull)
    - Provide task assignment guidance (which nodes can serve which model)
    """

    def __init__(self) -> None:
        self._target_model: str = ""
        self._target_tier: str = ""
        self._previous_model: str = ""
        self._transition_state: TransitionState = TransitionState.STABLE
        self._transition_started_at: Optional[str] = None
        self._check_interval: int = 60  # seconds between recalculations
        self._monitor_task: Optional[asyncio.Task] = None
        self._exo_health_task: Optional[asyncio.Task] = None
        self._running = False

        # Hybrid router (Phase 4)
        self._hybrid_router = HybridRouter()
        self._execution_mode: ExecutionMode = ExecutionMode.LOCAL
        self._max_single_node_vram_gb: float = 0.0

        # Petals swarm (Phase 7)
        self._swarm_manager: Optional[object] = None

    # -- Properties --------------------------------------------------------

    @property
    def target_model(self) -> str:
        """The model the network is targeting (may not be ready on all nodes)."""
        return self._target_model

    @property
    def target_tier(self) -> str:
        return self._target_tier

    @property
    def previous_model(self) -> str:
        """Model that was active before current transition (still usable)."""
        return self._previous_model

    @property
    def transition_state(self) -> TransitionState:
        return self._transition_state

    @property
    def is_transitioning(self) -> bool:
        return self._transition_state == TransitionState.TRANSITIONING

    @property
    def hybrid_router(self) -> HybridRouter:
        return self._hybrid_router

    @property
    def execution_mode(self) -> ExecutionMode:
        return self._execution_mode

    # -- Lifecycle ---------------------------------------------------------

    async def start(self) -> None:
        """Start the model selector with periodic recalculation."""
        if self._running:
            return
        self._running = True
        await self.recalculate()
        self._monitor_task = asyncio.create_task(self._monitor_loop())

        # Start exo health check if enabled
        if self._hybrid_router.exo_enabled:
            await self._hybrid_router.check_exo_health()
            self._exo_health_task = asyncio.create_task(self._exo_health_loop())

        # Start Petals swarm manager if enabled
        if settings.petals_enabled and HAS_SWARM:
            self._swarm_manager = SwarmManager()
            await self._swarm_manager.start()

        logger.info(
            "ModelSelector started (target: {}, tier: {}, mode: {})",
            self._target_model, self._target_tier, self._execution_mode.value,
        )

    async def stop(self) -> None:
        """Stop the monitor loop and swarm manager."""
        self._running = False
        for task in (self._monitor_task, self._exo_health_task):
            if task and not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
        if self._swarm_manager and hasattr(self._swarm_manager, "stop"):
            await self._swarm_manager.stop()
        logger.info("ModelSelector stopped")

    # -- Core: model recalculation -----------------------------------------

    async def recalculate(self) -> dict[str, Any]:
        """Recalculate the target model based on current network state.

        Returns a status dict with the new model, tier, transition state,
        and per-node readiness info.
        """
        async with get_db() as conn:
            db = ComputeDatabase(conn)
            nodes = await db.get_online_nodes()

        total_vram_gb = sum(n.get("vram_mb", 0) for n in nodes) / 1024
        best_tier = get_tier_for_vram(total_vram_gb)
        new_model = best_tier["model"]
        new_tier_label = best_tier["label"]

        # Check if model changed
        model_changed = new_model != self._target_model and self._target_model != ""

        if model_changed:
            self._previous_model = self._target_model
            self._transition_started_at = datetime.now(timezone.utc).isoformat()
            logger.info(
                "Model switch: {} -> {} ({:.1f} GB, {} nodes, tier: {})",
                self._target_model, new_model, total_vram_gb, len(nodes), new_tier_label,
            )

        self._target_model = new_model
        self._target_tier = new_tier_label

        # Calculate max single-node VRAM for hybrid routing
        self._max_single_node_vram_gb = max(
            (n.get("vram_mb", 0) / 1024 for n in nodes), default=0.0,
        )
        model_min_vram = self._get_model_min_vram(new_model)
        petals_ready = (
            self._swarm_manager is not None
            and hasattr(self._swarm_manager, "is_ready")
            and self._swarm_manager.is_ready
        )
        self._hybrid_router.update_network_state(
            total_vram_gb=total_vram_gb,
            max_single_node_vram_gb=self._max_single_node_vram_gb,
            target_model_min_vram_gb=model_min_vram,
            petals_ready=petals_ready,
        )

        # Determine execution mode: always maximize model quality
        # Only use Petals when model actually needs distribution (doesn't fit single node)
        if petals_ready and total_vram_gb >= settings.petals_min_vram_gb and model_min_vram > self._max_single_node_vram_gb:
            self._execution_mode = ExecutionMode.PETALS
        elif self._hybrid_router.needs_distributed() and self._hybrid_router.exo_available:
            self._execution_mode = ExecutionMode.DISTRIBUTED
        else:
            self._execution_mode = ExecutionMode.LOCAL

        # Calculate readiness
        readiness = self._calculate_readiness(nodes, new_model)

        if model_changed or self._transition_state == TransitionState.TRANSITIONING:
            self._transition_state = readiness["state"]
        elif not self._target_model:
            # First calculation
            self._transition_state = TransitionState.STABLE

        return {
            "target_model": new_model,
            "target_tier": new_tier_label,
            "previous_model": self._previous_model,
            "transition_state": self._transition_state.value,
            "execution_mode": self._execution_mode.value,
            "total_vram_gb": round(total_vram_gb, 1),
            "max_single_node_vram_gb": round(self._max_single_node_vram_gb, 1),
            "nodes_total": len(nodes),
            **readiness,
        }

    def _calculate_readiness(self, nodes: list[dict], target_model: str) -> dict:
        """Check how many nodes have the target model ready.

        A node is "ready" if its current_model matches the target.
        A node is "compatible" if its individual VRAM can run the target.
        """
        if not nodes:
            return {
                "state": TransitionState.STABLE,
                "nodes_ready": 0,
                "nodes_compatible": 0,
                "nodes_pulling": 0,
                "readiness_pct": 100.0,
            }

        ready = 0
        compatible = 0
        pulling = 0

        for node in nodes:
            node_vram_gb = node.get("vram_mb", 0) / 1024
            node_current = node.get("current_model", "")
            node_model_status = node.get("model_status", "")

            # Can this node even run the target model?
            node_best = get_tier_for_vram(node_vram_gb)["model"]
            can_run_target = node_vram_gb >= self._get_model_min_vram(target_model)

            if can_run_target:
                compatible += 1
                if node_current == target_model:
                    ready += 1
                elif node_model_status == "pulling":
                    pulling += 1

        # Determine transition state
        if compatible == 0:
            state = TransitionState.STABLE
        elif ready >= compatible:
            state = TransitionState.STABLE
        elif pulling > 0 or ready > 0:
            state = TransitionState.TRANSITIONING
        else:
            state = TransitionState.TRANSITIONING  # No one ready yet, but transition started

        readiness_pct = (ready / compatible * 100) if compatible > 0 else 100.0

        return {
            "state": state,
            "nodes_ready": ready,
            "nodes_compatible": compatible,
            "nodes_pulling": pulling,
            "readiness_pct": round(readiness_pct, 1),
        }

    @staticmethod
    def _get_model_min_vram(model: str) -> float:
        """Get the minimum VRAM (GB) required to run a model."""
        for tier in MODEL_TIERS:
            if tier["model"] == model:
                return tier["min_vram_gb"]
        return 0.0

    # -- Per-node model assignment -----------------------------------------

    def get_model_for_node(self, vram_mb: int) -> str:
        """Return the best model this specific node should load.

        During transitions, nodes that can't run the target model
        are assigned the best model they can run individually.
        """
        node_vram_gb = vram_mb / 1024
        target_min_vram = self._get_model_min_vram(self._target_model)

        if node_vram_gb >= target_min_vram:
            # Node can run the network target
            return self._target_model
        else:
            # Node can't run target — assign best individual model
            return get_tier_for_vram(node_vram_gb)["model"]

    async def get_all_node_assignments(self) -> list[dict]:
        """Return model assignments for all online nodes."""
        async with get_db() as conn:
            db = ComputeDatabase(conn)
            nodes = await db.get_online_nodes()

        assignments = []
        for node in nodes:
            assigned_model = self.get_model_for_node(node.get("vram_mb", 0))
            current_model = node.get("current_model", "")
            assignments.append({
                "node_id": node["id"],
                "name": node.get("name", ""),
                "vram_mb": node.get("vram_mb", 0),
                "assigned_model": assigned_model,
                "current_model": current_model,
                "ready": current_model == assigned_model,
                "needs_pull": current_model != assigned_model,
            })
        return assignments

    # -- Model readiness reporting -----------------------------------------

    async def report_model_ready(self, node_id: str, model: str) -> dict:
        """Called when a node finishes pulling its assigned model.

        Validates that the reported model matches what was assigned,
        then updates the node's current_model and model_status
        and recalculates transition state.
        """
        async with get_db() as conn:
            db = ComputeDatabase(conn)
            node = await db.get_node(node_id)
            if not node:
                return {"accepted": False, "message": "Node not found"}

            # Validate: node should only report ready for its assigned model
            assigned = node.get("assigned_model", "")
            expected = assigned or self.get_model_for_node(node.get("vram_mb", 0))
            if model != expected:
                logger.warning(
                    "Node {} reported ready for '{}' but assigned '{}'",
                    node_id[:8], model, expected,
                )
                return {
                    "accepted": False,
                    "message": f"Model mismatch: expected '{expected}', got '{model}'",
                }

            await db.update_node_model_status(node_id, model, "ready")

        # Check if all nodes are now ready
        status = await self.recalculate()

        logger.info(
            "Node {} reports model '{}' ready (network: {:.0f}% ready, state: {})",
            node_id[:8], model, status["readiness_pct"], status["transition_state"],
        )

        return {
            "accepted": True,
            "message": "Model ready acknowledged",
            "transition_state": status["transition_state"],
            "readiness_pct": status["readiness_pct"],
        }

    # -- Task model selection (mixed mode) ---------------------------------

    def get_task_model(self, task_type: str, priority: int) -> str:
        """Select the model for a new task, considering transition state.

        During transitions:
        - Urgent tasks (priority <= 3) → use whatever model is available
        - Batch tasks (priority >= 7) → can wait for target model
        - Normal tasks → prefer target, fallback to previous
        """
        if self._transition_state == TransitionState.STABLE:
            return self._target_model

        # During transition, urgent tasks can use previous model
        if priority <= 3 and self._previous_model:
            return ""  # Empty = any model (affinity will match)

        return self._target_model

    def get_task_execution_mode(self, task_type: str, priority: int) -> ExecutionMode:
        """Determine execution mode for a task (local vs distributed)."""
        model_min_vram = self._get_model_min_vram(self._target_model)
        return self._hybrid_router.route(
            task_type=task_type,
            model=self._target_model,
            model_min_vram_gb=model_min_vram,
        )

    # -- Background monitor ------------------------------------------------

    async def _monitor_loop(self) -> None:
        """Periodically recalculate model selection."""
        while self._running:
            try:
                await asyncio.sleep(self._check_interval)
                if not self._running:
                    break
                await self.recalculate()
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.error("ModelSelector monitor error: {}", exc)
                await asyncio.sleep(10)

    async def _exo_health_loop(self) -> None:
        """Periodically check exo cluster health."""
        interval = settings.exo_health_interval
        while self._running:
            try:
                await asyncio.sleep(interval)
                if not self._running:
                    break
                was_available = self._hybrid_router.exo_available
                is_available = await self._hybrid_router.check_exo_health()
                if was_available != is_available:
                    logger.info(
                        "exo cluster {} (model: {})",
                        "available" if is_available else "unavailable",
                        self._hybrid_router.exo_model,
                    )
                    await self.recalculate()
            except asyncio.CancelledError:
                break
            except Exception as exc:
                logger.debug("exo health check error: {}", exc)
                await asyncio.sleep(10)

    # -- Status ------------------------------------------------------------

    def get_status(self) -> dict:
        """Return model selector status for API/health checks."""
        status = {
            "target_model": self._target_model,
            "target_tier": self._target_tier,
            "previous_model": self._previous_model,
            "transition_state": self._transition_state.value,
            "transition_started_at": self._transition_started_at,
            "execution_mode": self._execution_mode.value,
            "hybrid": self._hybrid_router.get_status(),
            "running": self._running,
        }
        if self._swarm_manager and hasattr(self._swarm_manager, "get_status"):
            status["swarm"] = self._swarm_manager.get_status()
        return status
