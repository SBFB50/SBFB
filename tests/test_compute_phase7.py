"""
Tests for Phase 7 — Petals: split 405B across 50 GPUs.

Covers:
- PetalsBackend initialization and fallback
- SwarmManager health states
- HybridRouter PETALS execution mode
- ModelSelector Petals integration
- Swarm health enum
- Config defaults
- Module imports
"""

import pytest

from nexus.compute.petals_backend import PetalsBackend, HAS_PETALS
from nexus.compute.swarm import SwarmManager, SwarmHealth, MODEL_BLOCKS
from nexus.compute.hybrid import HybridRouter, ExecutionMode
from nexus.compute.model_selector import ModelSelector
from nexus.config import Settings


# ===================================================================
# PetalsBackend
# ===================================================================

class TestPetalsBackend:
    """Test Petals backend initialization."""

    def test_init_default(self):
        backend = PetalsBackend()
        assert backend.model_name == "meta-llama/Meta-Llama-3.1-405B"
        assert backend.loaded is False

    def test_init_custom_model(self):
        backend = PetalsBackend(model_name="meta-llama/Meta-Llama-3.1-70B")
        assert backend.model_name == "meta-llama/Meta-Llama-3.1-70B"

    def test_available_reflects_import(self):
        backend = PetalsBackend()
        assert backend.available == HAS_PETALS

    def test_get_status_not_loaded(self):
        backend = PetalsBackend()
        status = backend.get_status()
        assert status["loaded"] is False
        assert "model" in status

    @pytest.mark.asyncio
    async def test_unload(self):
        backend = PetalsBackend()
        await backend.unload()
        assert backend.loaded is False


# ===================================================================
# SwarmManager
# ===================================================================

class TestSwarmManager:
    """Test Petals swarm monitoring."""

    def test_initial_state(self):
        mgr = SwarmManager()
        assert mgr.health == SwarmHealth.UNKNOWN
        assert mgr.nodes_online == 0
        assert mgr.blocks_covered == 0
        assert mgr.is_ready is False

    def test_coverage_pct_zero(self):
        mgr = SwarmManager()
        mgr._blocks_total = 0
        assert mgr.coverage_pct == 0.0

    def test_coverage_pct_full(self):
        mgr = SwarmManager()
        mgr._blocks_total = 80
        mgr._blocks_covered = 80
        assert mgr.coverage_pct == 100.0

    def test_coverage_pct_partial(self):
        mgr = SwarmManager()
        mgr._blocks_total = 80
        mgr._blocks_covered = 40
        assert mgr.coverage_pct == 50.0

    def test_is_ready_when_healthy(self):
        mgr = SwarmManager()
        mgr._health = SwarmHealth.HEALTHY
        assert mgr.is_ready is True

    def test_is_ready_when_degraded(self):
        mgr = SwarmManager()
        mgr._health = SwarmHealth.DEGRADED
        assert mgr.is_ready is False

    def test_get_status(self):
        mgr = SwarmManager()
        status = mgr.get_status()
        assert "health" in status
        assert "blocks_total" in status
        assert "blocks_covered" in status
        assert "coverage_pct" in status
        assert "is_ready" in status


# ===================================================================
# SwarmHealth enum
# ===================================================================

class TestSwarmHealth:
    """Test swarm health states."""

    def test_healthy(self):
        assert SwarmHealth.HEALTHY == "healthy"

    def test_degraded(self):
        assert SwarmHealth.DEGRADED == "degraded"

    def test_offline(self):
        assert SwarmHealth.OFFLINE == "offline"

    def test_unknown(self):
        assert SwarmHealth.UNKNOWN == "unknown"

    def test_count(self):
        assert len(SwarmHealth) == 4


# ===================================================================
# MODEL_BLOCKS
# ===================================================================

class TestModelBlocks:
    """Test model block definitions."""

    def test_405b_blocks(self):
        assert MODEL_BLOCKS["meta-llama/Meta-Llama-3.1-405B"] == 126

    def test_70b_blocks(self):
        assert MODEL_BLOCKS["meta-llama/Meta-Llama-3.1-70B"] == 80

    def test_8b_blocks(self):
        assert MODEL_BLOCKS["meta-llama/Meta-Llama-3.1-8B"] == 32


# ===================================================================
# HybridRouter — PETALS mode
# ===================================================================

class TestHybridRouterPetals:
    """Test Petals routing in HybridRouter."""

    def test_petals_mode_when_ready(self):
        router = HybridRouter(exo_enabled=True)
        router._petals_enabled = True
        router._petals_ready = True
        router._petals_min_vram_gb = 150.0
        router._total_vram_gb = 200.0
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction", "llama-405b", 150.0)
        assert result == ExecutionMode.PETALS

    def test_petals_not_ready_falls_to_exo(self):
        router = HybridRouter(exo_enabled=True)
        router._petals_enabled = True
        router._petals_ready = False
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction", "llama-70b", 40.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_petals_disabled_uses_exo(self):
        router = HybridRouter(exo_enabled=True)
        router._petals_enabled = False
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction", "llama-70b", 40.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_petals_ready_but_low_vram_uses_exo(self):
        router = HybridRouter(exo_enabled=True)
        router._petals_enabled = True
        router._petals_ready = True
        router._petals_min_vram_gb = 150.0
        router._total_vram_gb = 80.0  # Not enough for Petals
        router._exo_available = True
        router._max_single_node_vram_gb = 16.0
        result = router.route("contradiction", "llama-70b", 40.0)
        assert result == ExecutionMode.DISTRIBUTED

    def test_execution_mode_enum_has_petals(self):
        assert ExecutionMode.PETALS == "petals"
        assert len(ExecutionMode) == 4

    def test_get_status_includes_petals(self):
        router = HybridRouter()
        status = router.get_status()
        assert "petals_enabled" in status
        assert "petals_ready" in status


# ===================================================================
# ModelSelector — Petals integration
# ===================================================================

class TestModelSelectorPetals:
    """Test ModelSelector Petals integration."""

    def test_default_no_swarm(self):
        selector = ModelSelector()
        assert selector._swarm_manager is None

    def test_status_includes_swarm_when_available(self):
        selector = ModelSelector()
        # Simulate swarm manager
        class FakeSwarm:
            def get_status(self):
                return {"health": "healthy", "blocks_covered": 80}
        selector._swarm_manager = FakeSwarm()
        status = selector.get_status()
        assert "swarm" in status
        assert status["swarm"]["health"] == "healthy"


# ===================================================================
# Config defaults
# ===================================================================

class TestPhase7Config:
    """Test Phase 7 config defaults."""

    def test_petals_disabled_by_default(self):
        s = Settings()
        assert s.petals_enabled is False

    def test_petals_model_default(self):
        s = Settings()
        assert "405B" in s.petals_model

    def test_petals_initial_peers_empty(self):
        s = Settings()
        assert s.petals_initial_peers == []

    def test_petals_health_interval(self):
        s = Settings()
        assert s.petals_health_interval == 60

    def test_petals_min_vram(self):
        s = Settings()
        assert s.petals_min_vram_gb == 150


# ===================================================================
# Module imports
# ===================================================================

class TestPhase7Imports:
    """Test Phase 7 imports."""

    def test_import_petals_backend(self):
        from nexus.compute.petals_backend import PetalsBackend
        assert PetalsBackend is not None

    def test_import_swarm(self):
        from nexus.compute.swarm import SwarmManager, SwarmHealth, MODEL_BLOCKS
        assert len(SwarmHealth) == 4

    def test_import_hybrid_petals_mode(self):
        from nexus.compute.hybrid import ExecutionMode
        assert ExecutionMode.PETALS == "petals"
