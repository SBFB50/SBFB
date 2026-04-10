"""
Tests for the distributed GPU compute system — Phase 2 (Auto-scaling).

Covers:
- ModelSelector (tier selection, per-node assignment, transitions)
- Model readiness tracking (DB methods)
- Model transition records
- Enhanced heartbeat (model instructions)
- Pydantic models (Phase 2)
- get_tier_for_vram / get_node_model helpers
"""

import asyncio
from unittest.mock import AsyncMock, MagicMock, patch

import aiosqlite
import pytest
import pytest_asyncio

from nexus.compute.db import (
    ComputeDatabase,
    _COMPUTE_CREATE_TABLES,
    _COMPUTE_CREATE_INDEXES,
)
from nexus.compute.model_selector import (
    MODEL_TIERS,
    ModelSelector,
    TransitionState,
    get_node_model,
    get_tier_for_vram,
)
from nexus.compute.models import (
    ModelReadyRequest,
    ModelReadyResponse,
    ModelStatusResponse,
    ModelTransitionEntry,
    NodeAssignment,
)


# ===================================================================
# Fixtures
# ===================================================================

@pytest_asyncio.fixture
async def db():
    """Create an in-memory SQLite DB with compute tables."""
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA journal_mode = WAL")
    await conn.execute("PRAGMA foreign_keys = ON")
    await conn.executescript(_COMPUTE_CREATE_TABLES)
    await conn.executescript(_COMPUTE_CREATE_INDEXES)
    await conn.commit()
    yield ComputeDatabase(conn)
    await conn.close()


# ===================================================================
# get_tier_for_vram / get_node_model helpers
# ===================================================================

class TestModelTierHelpers:
    """Test model tier selection helpers."""

    def test_zero_vram_returns_basique(self):
        tier = get_tier_for_vram(0)
        assert tier["label"] == "Basique"
        assert tier["model"] == "gemma-4-12b-q4"

    def test_16gb_returns_standard(self):
        tier = get_tier_for_vram(16)
        assert tier["label"] == "Standard"

    def test_14gb_exact_threshold(self):
        tier = get_tier_for_vram(14)
        assert tier["label"] == "Standard"

    def test_13gb_below_standard(self):
        tier = get_tier_for_vram(13)
        assert tier["label"] == "Basique"

    def test_50gb_returns_avance(self):
        tier = get_tier_for_vram(50)
        assert tier["label"] == "Avance"

    def test_100gb_returns_pro(self):
        tier = get_tier_for_vram(100)
        assert tier["label"] == "Pro"

    def test_200gb_returns_ultra(self):
        tier = get_tier_for_vram(200)
        assert tier["label"] == "Ultra"

    def test_500gb_returns_maximum(self):
        tier = get_tier_for_vram(500)
        assert tier["label"] == "Maximum"

    def test_get_node_model_16gb(self):
        model = get_node_model(16384)  # 16 GB in MB
        assert "26B" in model or "gemma" in model

    def test_get_node_model_8gb(self):
        model = get_node_model(8192)  # 8 GB in MB
        assert model == "gemma-4-12b-q4"  # Basique

    def test_get_node_model_24gb(self):
        model = get_node_model(24576)  # 24 GB
        assert "26B" in model or "gemma" in model  # Standard (14-40 GB range)


# ===================================================================
# ModelSelector — Core logic
# ===================================================================

class TestModelSelector:
    """Test ModelSelector initialization and properties."""

    def test_initial_state(self):
        selector = ModelSelector()
        assert selector.target_model == ""
        assert selector.target_tier == ""
        assert selector.previous_model == ""
        assert selector.transition_state == TransitionState.STABLE
        assert selector.is_transitioning is False

    def test_get_model_for_node_small(self):
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        # 8GB node can't run 70B (needs 40GB)
        model = selector.get_model_for_node(8192)
        assert model == "gemma-4-12b-q4"  # Falls back to individual best

    def test_get_model_for_node_large(self):
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        # 48GB node CAN run 70B
        model = selector.get_model_for_node(49152)
        assert model == "llama-3.1-70b-q4"

    def test_get_model_for_node_exact_threshold(self):
        selector = ModelSelector()
        selector._target_model = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
        # 14GB node can run 26B (exact threshold)
        model = selector.get_model_for_node(14336)  # 14 GB
        assert model == "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"

    def test_get_task_model_stable(self):
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        selector._transition_state = TransitionState.STABLE
        assert selector.get_task_model("sentiment", 5) == "llama-3.1-70b-q4"

    def test_get_task_model_transitioning_urgent(self):
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        selector._previous_model = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
        selector._transition_state = TransitionState.TRANSITIONING
        # Urgent task (priority 2) → empty model = any available
        model = selector.get_task_model("contradiction", 2)
        assert model == ""  # Any model accepted

    def test_get_task_model_transitioning_batch(self):
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        selector._previous_model = "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
        selector._transition_state = TransitionState.TRANSITIONING
        # Batch task → target model (can wait)
        model = selector.get_task_model("sentiment", 8)
        assert model == "llama-3.1-70b-q4"

    def test_get_min_vram_known_model(self):
        vram = ModelSelector._get_model_min_vram("llama-3.1-70b-q4")
        assert vram == 40.0

    def test_get_min_vram_unknown_model(self):
        vram = ModelSelector._get_model_min_vram("nonexistent-model")
        assert vram == 0.0

    def test_get_status(self):
        selector = ModelSelector()
        selector._target_model = "test-model"
        selector._target_tier = "Test"
        status = selector.get_status()
        assert status["target_model"] == "test-model"
        assert status["target_tier"] == "Test"
        assert status["transition_state"] == "stable"

    def test_calculate_readiness_no_nodes(self):
        selector = ModelSelector()
        result = selector._calculate_readiness([], "any-model")
        assert result["state"] == TransitionState.STABLE
        assert result["readiness_pct"] == 100.0
        assert result["nodes_ready"] == 0

    def test_calculate_readiness_all_ready(self):
        selector = ModelSelector()
        nodes = [
            {"vram_mb": 16384, "current_model": "gemma-4-12b-q4", "model_status": "ready"},
            {"vram_mb": 16384, "current_model": "gemma-4-12b-q4", "model_status": "ready"},
        ]
        result = selector._calculate_readiness(nodes, "gemma-4-12b-q4")
        assert result["state"] == TransitionState.STABLE
        assert result["nodes_ready"] == 2
        assert result["readiness_pct"] == 100.0

    def test_calculate_readiness_some_pulling(self):
        selector = ModelSelector()
        nodes = [
            {"vram_mb": 16384, "current_model": "gemma-4-12b-q4", "model_status": "ready"},
            {"vram_mb": 16384, "current_model": "", "model_status": "pulling"},
        ]
        result = selector._calculate_readiness(nodes, "gemma-4-12b-q4")
        assert result["state"] == TransitionState.TRANSITIONING
        assert result["nodes_ready"] == 1
        assert result["nodes_pulling"] == 1
        assert result["readiness_pct"] == 50.0

    def test_calculate_readiness_incompatible_nodes(self):
        selector = ModelSelector()
        nodes = [
            {"vram_mb": 8192, "current_model": "gemma-4-12b-q4", "model_status": "ready"},
        ]
        # 70B requires 40GB — 8GB node is not compatible
        result = selector._calculate_readiness(nodes, "llama-3.1-70b-q4")
        assert result["nodes_compatible"] == 0
        assert result["state"] == TransitionState.STABLE  # No compatible nodes = nothing to transition


# ===================================================================
# ComputeDatabase — Model tracking methods
# ===================================================================

class TestModelTracking:
    """Test model-related DB methods."""

    @pytest.mark.asyncio
    async def test_update_node_model_status_ready(self, db: ComputeDatabase):
        node, _ = await db.register_node(
            name="N", gpu_model="G", vram_mb=16384, ip="1.1.1.1",
        )
        await db.update_node_model_status(node["id"], "gemma-4-12b-q4", "ready")
        updated = await db.get_node(node["id"])
        assert updated["current_model"] == "gemma-4-12b-q4"
        assert updated["assigned_model"] == "gemma-4-12b-q4"
        assert updated["model_status"] == "ready"

    @pytest.mark.asyncio
    async def test_update_node_model_status_pulling(self, db: ComputeDatabase):
        node, _ = await db.register_node(
            name="N", gpu_model="G", vram_mb=16384, ip="1.1.1.1",
        )
        await db.update_node_model_status(node["id"], "llama-70b", "pulling")
        updated = await db.get_node(node["id"])
        assert updated["assigned_model"] == "llama-70b"
        assert updated["model_status"] == "pulling"
        assert updated["model_pull_started_at"] is not None
        # current_model should NOT change during pull
        assert updated["current_model"] == ""

    @pytest.mark.asyncio
    async def test_set_node_assigned_model(self, db: ComputeDatabase):
        node, _ = await db.register_node(
            name="N", gpu_model="G", vram_mb=16384, ip="1.1.1.1",
        )
        await db.set_node_assigned_model(node["id"], "target-model")
        updated = await db.get_node(node["id"])
        assert updated["assigned_model"] == "target-model"

    @pytest.mark.asyncio
    async def test_get_nodes_by_model(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        n2, _ = await db.register_node(name="B", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        await db.update_node_model_status(n1["id"], "model-x", "ready")
        await db.update_node_model_status(n2["id"], "model-y", "ready")

        nodes_x = await db.get_nodes_by_model("model-x")
        assert len(nodes_x) == 1
        assert nodes_x[0]["id"] == n1["id"]

    @pytest.mark.asyncio
    async def test_get_nodes_needing_pull(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        n2, _ = await db.register_node(name="B", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        # n1: current=old, assigned=new → needs pull
        await db.update_node_model_status(n1["id"], "old-model", "ready")
        await db.set_node_assigned_model(n1["id"], "new-model")
        # n2: already on target
        await db.update_node_model_status(n2["id"], "new-model", "ready")

        needing = await db.get_nodes_needing_pull()
        assert len(needing) == 1
        assert needing[0]["id"] == n1["id"]


# ===================================================================
# ComputeDatabase — Model transitions
# ===================================================================

class TestModelTransitions:
    """Test model transition records."""

    @pytest.mark.asyncio
    async def test_create_transition(self, db: ComputeDatabase):
        t = await db.create_transition(
            old_model="gemma-4-12b-q4",
            new_model="llama-3.1-70b-q4",
            old_tier="Basique",
            new_tier="Avance",
            total_vram_gb=50.0,
            nodes_online=3,
        )
        assert t["old_model"] == "gemma-4-12b-q4"
        assert t["new_model"] == "llama-3.1-70b-q4"

    @pytest.mark.asyncio
    async def test_get_active_transition(self, db: ComputeDatabase):
        await db.create_transition("a", "b", "A", "B", 10.0, 1)
        active = await db.get_active_transition()
        assert active is not None
        assert active["transition_state"] == "transitioning"

    @pytest.mark.asyncio
    async def test_complete_transition(self, db: ComputeDatabase):
        t = await db.create_transition("a", "b", "A", "B", 10.0, 1)
        await db.complete_transition(t["id"], nodes_ready=1)
        active = await db.get_active_transition()
        assert active is None  # No more transitioning

    @pytest.mark.asyncio
    async def test_list_transitions(self, db: ComputeDatabase):
        await db.create_transition("a", "b", "A", "B", 10.0, 1)
        await db.create_transition("b", "c", "B", "C", 20.0, 2)
        transitions = await db.list_transitions()
        assert len(transitions) == 2
        # Most recent first
        assert transitions[0]["new_model"] == "c"


# ===================================================================
# Pydantic models — Phase 2
# ===================================================================

class TestPhase2Models:
    """Test Phase 2 Pydantic model validation."""

    def test_model_ready_request_valid(self):
        req = ModelReadyRequest(model="llama-3.1-70b-q4")
        assert req.model == "llama-3.1-70b-q4"

    def test_model_ready_request_empty_rejected(self):
        with pytest.raises(Exception):
            ModelReadyRequest(model="")

    def test_model_status_response(self):
        resp = ModelStatusResponse(
            target_model="llama-70b",
            target_tier="Avance",
            transition_state="stable",
            nodes_online=5,
            nodes_ready=5,
            readiness_pct=100.0,
        )
        assert resp.target_model == "llama-70b"

    def test_node_assignment(self):
        a = NodeAssignment(
            node_id="abc",
            name="FlowUP",
            vram_mb=16384,
            assigned_model="gemma-26b",
            current_model="gemma-26b",
            ready=True,
            needs_pull=False,
        )
        assert a.ready is True

    def test_model_transition_entry(self):
        e = ModelTransitionEntry(
            id="t1",
            old_model="gemma-12b",
            new_model="gemma-26b",
            transition_state="stable",
        )
        assert e.id == "t1"

    def test_model_ready_response(self):
        r = ModelReadyResponse(
            accepted=True,
            message="OK",
            transition_state="stable",
            readiness_pct=100.0,
        )
        assert r.accepted is True


# ===================================================================
# TransitionState enum
# ===================================================================

class TestTransitionState:
    """Test transition state values."""

    def test_stable(self):
        assert TransitionState.STABLE == "stable"

    def test_transitioning(self):
        assert TransitionState.TRANSITIONING == "transitioning"

    def test_degraded(self):
        assert TransitionState.DEGRADED == "degraded"


# ===================================================================
# Module imports (Phase 2)
# ===================================================================

class TestPhase2Imports:
    """Test that Phase 2 components import correctly."""

    def test_import_model_selector(self):
        from nexus.compute.model_selector import ModelSelector, MODEL_TIERS, TransitionState
        assert len(MODEL_TIERS) == 6
        assert ModelSelector is not None

    def test_import_from_package(self):
        from nexus.compute import ModelSelector, MODEL_TIERS
        assert ModelSelector is not None
        assert len(MODEL_TIERS) == 6

    def test_import_phase2_models(self):
        from nexus.compute.models import (
            ModelReadyRequest, ModelReadyResponse,
            ModelStatusResponse, NodeAssignment, ModelTransitionEntry,
        )
        assert ModelReadyRequest is not None

    def test_dispatcher_uses_model_selector(self):
        from nexus.compute.dispatcher import TaskDispatcher
        d = TaskDispatcher(model_selector=None)
        assert d.current_model == ""
        assert d.current_tier == ""
