"""
Tests for the distributed GPU compute system — Phase 4 (Hybrid Mode).

Philosophy: ALWAYS pool GPU power to run the BIGGEST model possible.
More contributors = bigger model = better political analysis.

Covers:
- HybridRouter routing decisions (maximize model, not classify tasks)
- ExecutionMode enum
- ExoBackend initialization
- Integration with ModelSelector
- Pydantic models (Phase 4)
- Worker exo peer
- Config defaults
- Module imports
"""

from unittest.mock import AsyncMock, patch

import pytest

from nexus.compute.hybrid import (
    ExecutionMode,
    ExoBackend,
    HybridRouter,
)
from nexus.compute.models import HybridStatusResponse, ModelStatusResponse
from nexus.compute.model_selector import ModelSelector
from nexus.config import Settings


# ===================================================================
# ExecutionMode enum
# ===================================================================

class TestExecutionMode:
    """Test execution mode values."""

    def test_local(self):
        assert ExecutionMode.LOCAL == "local"

    def test_distributed(self):
        assert ExecutionMode.DISTRIBUTED == "distributed"

    def test_overflow(self):
        assert ExecutionMode.OVERFLOW == "overflow"

    def test_count(self):
        assert len(ExecutionMode) == 4  # LOCAL, DISTRIBUTED, PETALS, OVERFLOW


# ===================================================================
# HybridRouter — Always maximize model size
# ===================================================================

class TestHybridRouter:
    """Test hybrid routing: always pool power for biggest model."""

    def test_exo_disabled_always_local(self):
        """When exo is disabled, all tasks run locally."""
        router = HybridRouter(exo_enabled=False)
        result = router.route("sentiment", "llama-70b", 40.0)
        assert result == ExecutionMode.LOCAL

    def test_model_fits_single_node_all_local(self):
        """When the biggest model fits on one GPU, run ALL tasks locally."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 48.0
        # 70B needs 40GB, node has 48GB → fits locally
        result = router.route("sentiment", "llama-70b", 40.0)
        assert result == ExecutionMode.LOCAL

    def test_model_too_big_all_distributed(self):
        """When model exceeds any single node, ALL tasks go distributed."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        # 70B needs 40GB, max node is 16GB → must distribute
        # Even "sentiment" goes distributed — we want the 70B quality
        result = router.route("sentiment", "llama-70b", 40.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_heavy_task_also_distributed(self):
        """Heavy tasks also go distributed — no distinction."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction_detection", "llama-70b", 40.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_exo_unavailable_fallback_local(self):
        """When exo cluster is down, fallback to local with smaller model."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = False
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction_detection", "llama-70b", 40.0)
        assert result == ExecutionMode.LOCAL

    def test_tiny_node_overflow(self):
        """Nodes too small (<8GB) serve as overflow in distributed mode."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        # This node only has 6GB — too small for exo cluster
        result = router.route("sentiment", "llama-70b", 40.0, node_vram_gb=6.0)
        assert result == ExecutionMode.OVERFLOW

    def test_normal_node_distributed_not_overflow(self):
        """Nodes with decent VRAM participate in distribution."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("sentiment", "llama-70b", 40.0, node_vram_gb=16.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_needs_distributed_true(self):
        """needs_distributed() when target exceeds max single node."""
        router = HybridRouter()
        router._target_model_min_vram_gb = 40.0
        router._max_single_node_vram_gb = 16.0
        assert router.needs_distributed() is True

    def test_needs_distributed_false(self):
        """needs_distributed() when target fits on single node."""
        router = HybridRouter()
        router._target_model_min_vram_gb = 14.0
        router._max_single_node_vram_gb = 16.0
        assert router.needs_distributed() is False

    def test_update_network_state(self):
        router = HybridRouter()
        router.update_network_state(
            total_vram_gb=80.0,
            max_single_node_vram_gb=24.0,
            target_model_min_vram_gb=40.0,
            exo_model="llama-70b",
        )
        assert router._total_vram_gb == 80.0
        assert router._max_single_node_vram_gb == 24.0
        assert router._target_model_min_vram_gb == 40.0
        assert router._exo_model == "llama-70b"

    def test_get_status(self):
        router = HybridRouter(exo_url="http://localhost:52415", exo_enabled=True)
        status = router.get_status()
        assert status["exo_enabled"] is True
        assert status["exo_url"] == "http://localhost:52415"
        assert "needs_distributed" in status

    def test_default_exo_disabled(self):
        router = HybridRouter()
        assert router.exo_enabled is False
        assert router.exo_available is False

    def test_26b_on_16gb_nodes_local(self):
        """26B model (14GB) on 16GB nodes → local (fits)."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("any_task", "gemma-26b", 14.0)
        assert result == ExecutionMode.LOCAL

    def test_405b_on_16gb_nodes_distributed(self):
        """405B model (150GB) on 16GB nodes → distributed."""
        router = HybridRouter(exo_enabled=True)
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("any_task", "llama-405b", 150.0)
        assert result == ExecutionMode.DISTRIBUTED


# ===================================================================
# ExoBackend
# ===================================================================

class TestExoBackend:
    """Test exo backend initialization."""

    def test_init(self):
        backend = ExoBackend(exo_url="http://localhost:52415")
        assert backend._exo_url == "http://localhost:52415"

    def test_default_url_from_settings(self):
        backend = ExoBackend()
        assert "52415" in backend._exo_url


# ===================================================================
# ModelSelector + Hybrid integration
# ===================================================================

class TestModelSelectorHybrid:
    """Test ModelSelector hybrid mode integration."""

    def test_selector_has_hybrid_router(self):
        selector = ModelSelector()
        assert selector.hybrid_router is not None
        assert isinstance(selector.hybrid_router, HybridRouter)

    def test_selector_default_execution_mode(self):
        selector = ModelSelector()
        assert selector.execution_mode == ExecutionMode.LOCAL

    def test_get_task_execution_mode_exo_disabled(self):
        """All tasks local when exo disabled (default)."""
        selector = ModelSelector()
        selector._target_model = "llama-3.1-70b-q4"
        mode = selector.get_task_execution_mode("contradiction_detection", 5)
        assert mode == ExecutionMode.LOCAL

    def test_selector_status_includes_hybrid(self):
        selector = ModelSelector()
        status = selector.get_status()
        assert "execution_mode" in status
        assert "hybrid" in status

    def test_selector_status_hybrid_has_needs_distributed(self):
        selector = ModelSelector()
        status = selector.get_status()
        hybrid = status["hybrid"]
        assert "needs_distributed" in hybrid
        assert "total_vram_gb" in hybrid


# ===================================================================
# Pydantic models — Phase 4
# ===================================================================

class TestPhase4Models:
    """Test Phase 4 Pydantic model additions."""

    def test_hybrid_status_response(self):
        resp = HybridStatusResponse(
            execution_mode="distributed",
            exo_enabled=True,
            exo_available=True,
            exo_url="http://localhost:52415",
            exo_model="llama-70b",
            max_single_node_vram_gb=16.0,
            target_model="llama-70b",
            target_tier="Avance",
        )
        assert resp.execution_mode == "distributed"
        assert resp.exo_available is True

    def test_model_status_has_execution_mode(self):
        resp = ModelStatusResponse(
            target_model="test",
            execution_mode="distributed",
            max_single_node_vram_gb=16.0,
        )
        assert resp.execution_mode == "distributed"


# ===================================================================
# Config defaults
# ===================================================================

class TestPhase4Config:
    """Test Phase 4 config defaults."""

    def test_exo_disabled_by_default(self):
        s = Settings()
        assert s.exo_enabled is False

    def test_exo_url_default(self):
        s = Settings()
        assert s.exo_url == "http://localhost:52415"

    def test_exo_health_interval(self):
        s = Settings()
        assert s.exo_health_interval == 30


# ===================================================================
# Worker exo peer
# ===================================================================

class TestExoPeer:
    """Test worker exo peer mode."""

    def test_import(self):
        from worker.exo_peer import ExoPeer
        assert ExoPeer is not None

    def test_init(self):
        from worker.exo_peer import ExoPeer
        peer = ExoPeer(initial_peers="http://server:31330", port=31330)
        assert peer.running is False
        assert peer.healthy is False

    def test_get_status(self):
        from worker.exo_peer import ExoPeer
        peer = ExoPeer()
        status = peer.get_status()
        assert status["running"] is False
        assert status["pid"] is None


# ===================================================================
# Module imports
# ===================================================================

class TestPhase4Imports:
    """Test Phase 4 imports."""

    def test_import_hybrid(self):
        from nexus.compute.hybrid import HybridRouter, ExoBackend, ExecutionMode
        assert HybridRouter is not None

    def test_import_from_package(self):
        from nexus.compute import HybridRouter, ExoBackend, ExecutionMode
        assert ExecutionMode.LOCAL == "local"
        assert ExecutionMode.OVERFLOW == "overflow"

    def test_import_exo_peer(self):
        from worker.exo_peer import ExoPeer
        assert callable(ExoPeer.is_exo_installed)

    def test_import_phase4_models(self):
        from nexus.compute.models import HybridStatusResponse
        assert HybridStatusResponse is not None
