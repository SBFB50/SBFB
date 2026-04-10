"""
Tests for the distributed GPU compute system (Phase 1).

Covers:
- ComputeDatabase CRUD (nodes, tasks, results, stats, leaderboard)
- Task lifecycle (create -> assign -> complete/fail -> retry -> expire)
- Auth helpers (API key generation, hashing, IP hashing)
- TaskDispatcher (model selection, result validation, spot-check rates)
- ComputeEventType completeness
- Pydantic models validation
- API endpoint structure
- Config defaults
"""

import asyncio
import hashlib
import secrets
from unittest.mock import AsyncMock, MagicMock, patch

import aiosqlite
import pytest
import pytest_asyncio

from nexus.compute.db import (
    ComputeDatabase,
    _generate_api_key,
    _hash_api_key,
    _hash_ip,
    init_compute_db,
    _COMPUTE_CREATE_TABLES,
    _COMPUTE_CREATE_INDEXES,
)
from nexus.compute.events import ComputeEventType, ComputeDatabaseProxy
from nexus.compute.dispatcher import TaskDispatcher
from nexus.compute.model_selector import MODEL_TIERS
from nexus.compute.models import (
    NodeRegisterRequest,
    NodeRegisterResponse,
    NodeHeartbeatRequest,
    TaskCreateRequest,
    TaskResultRequest,
    TaskPullResponse,
    NetworkStatsResponse,
    LeaderboardEntry,
    LeaderboardResponse,
)
from nexus.config import Settings


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
# Auth helpers
# ===================================================================

class TestAuthHelpers:
    """Test API key and IP hashing utilities."""

    def test_generate_api_key_length(self):
        key = _generate_api_key()
        assert len(key) >= 32  # token_urlsafe(32) produces ~43 chars

    def test_generate_api_key_unique(self):
        keys = {_generate_api_key() for _ in range(100)}
        assert len(keys) == 100  # all unique

    def test_hash_api_key_deterministic(self):
        key = "test-api-key-123"
        h1 = _hash_api_key(key)
        h2 = _hash_api_key(key)
        assert h1 == h2
        assert len(h1) == 64  # SHA-256 hex

    def test_hash_api_key_differs(self):
        assert _hash_api_key("key1") != _hash_api_key("key2")

    def test_hash_ip_deterministic(self):
        h1 = _hash_ip("192.168.1.1")
        h2 = _hash_ip("192.168.1.1")
        assert h1 == h2
        assert len(h1) == 64

    def test_hash_ip_privacy(self):
        """IP hash should not be reversible to original IP."""
        h = _hash_ip("10.0.0.1")
        assert "10.0.0.1" not in h


# ===================================================================
# ComputeDatabase — Node CRUD
# ===================================================================

class TestComputeNodes:
    """Test GPU node registration and management."""

    @pytest.mark.asyncio
    async def test_register_node(self, db: ComputeDatabase):
        node, api_key = await db.register_node(
            name="TestNode",
            gpu_model="NVIDIA RTX 5080",
            vram_mb=16384,
            ip="192.168.1.100",
            platform="linux",
            ollama_version="0.5.7",
        )
        assert node["name"] == "TestNode"
        assert node["gpu_model"] == "NVIDIA RTX 5080"
        assert node["vram_mb"] == 16384
        assert node["status"] == "idle"
        assert node["trust_score"] == 50
        assert node["platform"] == "linux"
        assert len(api_key) >= 32
        # API key hash stored, not raw key
        assert "api_key_hash" in node
        assert node["api_key_hash"] == _hash_api_key(api_key)

    @pytest.mark.asyncio
    async def test_get_node(self, db: ComputeDatabase):
        node, _ = await db.register_node(
            name="Node1", gpu_model="RTX 4090", vram_mb=24576, ip="10.0.0.1",
        )
        fetched = await db.get_node(node["id"])
        assert fetched is not None
        assert fetched["name"] == "Node1"

    @pytest.mark.asyncio
    async def test_get_node_not_found(self, db: ComputeDatabase):
        result = await db.get_node("nonexistent-id")
        assert result is None

    @pytest.mark.asyncio
    async def test_get_node_by_api_key(self, db: ComputeDatabase):
        node, api_key = await db.register_node(
            name="AuthNode", gpu_model="RTX 3080", vram_mb=10240, ip="10.0.0.2",
        )
        found = await db.get_node_by_api_key(api_key)
        assert found is not None
        assert found["id"] == node["id"]

    @pytest.mark.asyncio
    async def test_get_node_by_bad_api_key(self, db: ComputeDatabase):
        await db.register_node(
            name="Node", gpu_model="RTX 3060", vram_mb=12288, ip="10.0.0.3",
        )
        result = await db.get_node_by_api_key("wrong-key")
        assert result is None

    @pytest.mark.asyncio
    async def test_list_nodes(self, db: ComputeDatabase):
        await db.register_node(name="A", gpu_model="RTX 5080", vram_mb=16384, ip="1.1.1.1")
        await db.register_node(name="B", gpu_model="RTX 4090", vram_mb=24576, ip="2.2.2.2")
        nodes = await db.list_nodes()
        assert len(nodes) == 2

    @pytest.mark.asyncio
    async def test_list_nodes_filter_status(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="A", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        n2, _ = await db.register_node(name="B", gpu_model="G", vram_mb=1000, ip="2.2.2.2")
        await db.update_node_status(n2["id"], "offline")
        idle = await db.list_nodes(status="idle")
        assert len(idle) == 1
        assert idle[0]["id"] == n1["id"]

    @pytest.mark.asyncio
    async def test_get_online_nodes(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16000, ip="1.1.1.1")
        n2, _ = await db.register_node(name="B", gpu_model="G", vram_mb=24000, ip="2.2.2.2")
        await db.update_node_status(n2["id"], "busy")
        n3, _ = await db.register_node(name="C", gpu_model="G", vram_mb=8000, ip="3.3.3.3")
        await db.update_node_status(n3["id"], "offline")
        online = await db.get_online_nodes()
        assert len(online) == 2  # idle + busy, not offline

    @pytest.mark.asyncio
    async def test_heartbeat(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="HB", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        result = await db.heartbeat(node["id"], "llama-3.1-70b")
        assert result is True
        updated = await db.get_node(node["id"])
        assert updated["current_model"] == "llama-3.1-70b"

    @pytest.mark.asyncio
    async def test_heartbeat_nonexistent(self, db: ComputeDatabase):
        result = await db.heartbeat("fake-id")
        assert result is False

    @pytest.mark.asyncio
    async def test_update_node_trust(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="T", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        assert node["trust_score"] == 50

        new_score = await db.update_node_trust(node["id"], 10)
        assert new_score == 60

        new_score = await db.update_node_trust(node["id"], -70)
        assert new_score == 0  # clamped to 0

        new_score = await db.update_node_trust(node["id"], 200)
        assert new_score == 100  # clamped to 100

    @pytest.mark.asyncio
    async def test_ban_node(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="Bad", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        await db.ban_node(node["id"])
        banned = await db.get_node(node["id"])
        assert banned["status"] == "banned"
        assert banned["trust_score"] == 0

    @pytest.mark.asyncio
    async def test_increment_node_stats(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="Stats", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=5, errored=1, tokens_per_sec=42.0)
        updated = await db.get_node(node["id"])
        assert updated["tasks_completed"] == 5
        assert updated["tasks_errored"] == 1
        assert updated["avg_tokens_per_sec"] == 42.0

    @pytest.mark.asyncio
    async def test_delete_node(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="Del", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        result = await db.delete_node(node["id"])
        assert result is True
        assert await db.get_node(node["id"]) is None


# ===================================================================
# ComputeDatabase — Task CRUD
# ===================================================================

class TestComputeTasks:
    """Test task queue operations."""

    @pytest.mark.asyncio
    async def test_create_task(self, db: ComputeDatabase):
        task = await db.create_task(
            task_type="sentiment",
            prompt="Analysez le sentiment de ce texte...",
            model="gemma-4-26b-q4",
            priority=3,
        )
        assert task["task_type"] == "sentiment"
        assert task["status"] == "pending"
        assert task["priority"] == 3
        assert task["model"] == "gemma-4-26b-q4"

    @pytest.mark.asyncio
    async def test_pull_next_task_priority(self, db: ComputeDatabase):
        """Higher priority (lower number) tasks should be pulled first."""
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")

        await db.create_task(task_type="low", prompt="low prio", priority=8)
        await db.create_task(task_type="high", prompt="high prio", priority=1)
        await db.create_task(task_type="medium", prompt="med prio", priority=5)

        task = await db.pull_next_task(node["id"])
        assert task["task_type"] == "high"
        assert task["status"] == "assigned"
        assert task["assigned_to"] == node["id"]

    @pytest.mark.asyncio
    async def test_pull_task_empty_queue(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        task = await db.pull_next_task(node["id"])
        assert task is None

    @pytest.mark.asyncio
    async def test_pull_task_model_affinity(self, db: ComputeDatabase):
        """Tasks matching the node's model should be pulled first."""
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")

        t1 = await db.create_task(task_type="a", prompt="any", model="other-model", priority=1)
        t2 = await db.create_task(task_type="b", prompt="match", model="llama-70b", priority=5)

        task = await db.pull_next_task(node["id"], model="llama-70b")
        assert task["id"] == t2["id"]  # Model affinity wins over priority

    @pytest.mark.asyncio
    async def test_complete_task(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        await db.create_task(task_type="test", prompt="test", priority=5)
        task = await db.pull_next_task(node["id"])

        completed = await db.complete_task(task["id"], "Result text", validated=True, validation_score=0.95)
        assert completed["status"] == "completed"
        assert completed["result"] == "Result text"
        assert completed["result_validated"] == 1
        assert completed["validation_score"] == 0.95

    @pytest.mark.asyncio
    async def test_fail_task_with_retry(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        task = await db.create_task(task_type="test", prompt="test", max_retries=3)
        pulled = await db.pull_next_task(node["id"])

        # First failure: should reset to pending
        failed = await db.fail_task(pulled["id"], "Timeout")
        assert failed["status"] == "pending"
        assert failed["retry_count"] == 1
        assert failed["assigned_to"] is None

    @pytest.mark.asyncio
    async def test_fail_task_max_retries(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        task = await db.create_task(task_type="test", prompt="test", max_retries=1)
        pulled = await db.pull_next_task(node["id"])

        # Already retry_count=0, max_retries=1, so first fail hits max
        failed = await db.fail_task(pulled["id"], "Permanent error")
        assert failed["status"] == "failed"
        assert failed["error_message"] == "Permanent error"

    @pytest.mark.asyncio
    async def test_list_tasks(self, db: ComputeDatabase):
        await db.create_task(task_type="a", prompt="test1")
        await db.create_task(task_type="b", prompt="test2")
        tasks = await db.list_tasks()
        assert len(tasks) == 2

    @pytest.mark.asyncio
    async def test_list_tasks_filter_status(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        await db.create_task(task_type="a", prompt="p1")
        await db.create_task(task_type="b", prompt="p2")
        await db.pull_next_task(node["id"])  # assigns first task

        pending = await db.list_tasks(status="pending")
        assert len(pending) == 1
        assigned = await db.list_tasks(status="assigned")
        assert len(assigned) == 1

    @pytest.mark.asyncio
    async def test_count_tasks(self, db: ComputeDatabase):
        await db.create_task(task_type="a", prompt="test1")
        await db.create_task(task_type="b", prompt="test2")
        assert await db.count_tasks() == 2
        assert await db.count_tasks("pending") == 2
        assert await db.count_tasks("completed") == 0


# ===================================================================
# ComputeDatabase — Results
# ===================================================================

class TestComputeResults:
    """Test result storage and validation."""

    @pytest.mark.asyncio
    async def test_store_result(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        task = await db.create_task(task_type="test", prompt="test")

        result = await db.store_result(
            task_id=task["id"],
            node_id=node["id"],
            result_text="Generated analysis text",
            tokens_generated=150,
            generation_time_ms=3200,
            model_digest="sha256:abc123",
        )
        assert result["task_id"] == task["id"]
        assert result["node_id"] == node["id"]
        assert result["tokens_generated"] == 150
        assert result["validated"] == 0

    @pytest.mark.asyncio
    async def test_validate_result(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="N", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        task = await db.create_task(task_type="test", prompt="test")
        result = await db.store_result(task["id"], node["id"], "text")

        await db.validate_result(result["id"], method="spot_check")
        # Verify via raw query
        cursor = await db._conn.execute(
            "SELECT validated, validation_method FROM compute_results WHERE id = ?",
            (result["id"],),
        )
        row = await cursor.fetchone()
        assert row[0] == 1
        assert row[1] == "spot_check"


# ===================================================================
# ComputeDatabase — Stats & Leaderboard
# ===================================================================

class TestComputeStats:
    """Test network stats and leaderboard."""

    @pytest.mark.asyncio
    async def test_network_stats_empty(self, db: ComputeDatabase):
        stats = await db.get_network_stats()
        assert stats["nodes_online"] == 0
        assert stats["vram_total_gb"] == 0.0
        assert stats["tasks_pending"] == 0

    @pytest.mark.asyncio
    async def test_network_stats_with_data(self, db: ComputeDatabase):
        await db.register_node(name="A", gpu_model="RTX 5080", vram_mb=16384, ip="1.1.1.1")
        await db.register_node(name="B", gpu_model="RTX 4090", vram_mb=24576, ip="2.2.2.2")
        await db.create_task(task_type="test", prompt="test")

        stats = await db.get_network_stats()
        assert stats["nodes_online"] == 2
        assert stats["vram_total_gb"] == pytest.approx(40.0, abs=0.1)
        assert stats["tasks_pending"] == 1

    @pytest.mark.asyncio
    async def test_leaderboard(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="Pro", gpu_model="RTX 5080", vram_mb=16384, ip="1.1.1.1")
        n2, _ = await db.register_node(name="Newbie", gpu_model="RTX 3060", vram_mb=12288, ip="2.2.2.2")
        await db.increment_node_stats(n1["id"], completed=100)
        await db.increment_node_stats(n2["id"], completed=10)

        leaders = await db.get_leaderboard()
        assert len(leaders) == 2
        assert leaders[0]["name"] == "Pro"
        assert leaders[0]["rank"] == 1
        assert leaders[1]["name"] == "Newbie"
        assert leaders[1]["rank"] == 2

    @pytest.mark.asyncio
    async def test_leaderboard_excludes_banned(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="Good", gpu_model="G", vram_mb=1000, ip="1.1.1.1")
        n2, _ = await db.register_node(name="Bad", gpu_model="G", vram_mb=1000, ip="2.2.2.2")
        await db.ban_node(n2["id"])

        leaders = await db.get_leaderboard()
        assert len(leaders) == 1
        assert leaders[0]["name"] == "Good"


# ===================================================================
# TaskDispatcher — Model selection
# ===================================================================

class TestTaskDispatcher:
    """Test dispatcher logic."""

    def test_model_tiers_ordered(self):
        """Model tiers must be sorted by min_vram_gb ascending."""
        for i in range(len(MODEL_TIERS) - 1):
            assert MODEL_TIERS[i]["min_vram_gb"] <= MODEL_TIERS[i + 1]["min_vram_gb"]

    def test_model_tiers_has_zero(self):
        """First tier must work with 0 VRAM (fallback)."""
        assert MODEL_TIERS[0]["min_vram_gb"] == 0

    def test_spot_check_rate_trusted(self):
        assert TaskDispatcher._get_spot_check_rate(90) == 0.01  # 1%
        assert TaskDispatcher._get_spot_check_rate(80) == 0.01  # 1% (boundary)

    def test_spot_check_rate_standard(self):
        assert TaskDispatcher._get_spot_check_rate(60) == 0.05  # 5%
        assert TaskDispatcher._get_spot_check_rate(50) == 0.05  # 5% (default trust = standard)

    def test_spot_check_rate_suspect(self):
        assert TaskDispatcher._get_spot_check_rate(30) == 0.20  # 20%
        assert TaskDispatcher._get_spot_check_rate(49) == 0.20  # 20% (just below standard)


# ===================================================================
# ComputeEventType -- Completeness
# ===================================================================

class TestComputeEventTypes:
    """Verify all compute event types exist."""

    def test_node_lifecycle_events(self):
        assert ComputeEventType.COMPUTE_NODE_REGISTERED == "compute_node_registered"
        assert ComputeEventType.COMPUTE_NODE_CONNECTED == "compute_node_connected"
        assert ComputeEventType.COMPUTE_NODE_DISCONNECTED == "compute_node_disconnected"
        assert ComputeEventType.COMPUTE_NODE_BANNED == "compute_node_banned"

    def test_task_lifecycle_events(self):
        assert ComputeEventType.COMPUTE_TASK_CREATED == "compute_task_created"
        assert ComputeEventType.COMPUTE_TASK_ASSIGNED == "compute_task_assigned"
        assert ComputeEventType.COMPUTE_TASK_COMPLETED == "compute_task_completed"
        assert ComputeEventType.COMPUTE_TASK_FAILED == "compute_task_failed"
        assert ComputeEventType.COMPUTE_TASK_EXPIRED == "compute_task_expired"

    def test_validation_events(self):
        assert ComputeEventType.COMPUTE_RESULT_VALIDATED == "compute_result_validated"
        assert ComputeEventType.COMPUTE_RESULT_REJECTED == "compute_result_rejected"
        assert ComputeEventType.COMPUTE_SPOT_CHECK_NEEDED == "compute_spot_check_needed"

    def test_model_event(self):
        assert ComputeEventType.COMPUTE_MODEL_CHANGED == "compute_model_changed"

    def test_tick_events(self):
        assert ComputeEventType.COMPUTE_TICK_HEARTBEAT == "compute_tick_heartbeat"
        assert ComputeEventType.COMPUTE_TICK_REAPER == "compute_tick_reaper"

    def test_total_event_count(self):
        """Ensure all expected events are present (no accidental removal)."""
        assert len(ComputeEventType) == 16


# ===================================================================
# Pydantic models — Validation
# ===================================================================

class TestPydanticModels:
    """Test request/response model validation."""

    def test_node_register_valid(self):
        req = NodeRegisterRequest(
            name="TestGPU",
            gpu_model="NVIDIA RTX 5080",
            vram_mb=16384,
        )
        assert req.name == "TestGPU"
        assert req.vram_mb == 16384

    def test_node_register_empty_name_rejected(self):
        with pytest.raises(Exception):  # ValidationError
            NodeRegisterRequest(name="", gpu_model="G", vram_mb=1000)

    def test_node_register_negative_vram_rejected(self):
        with pytest.raises(Exception):
            NodeRegisterRequest(name="N", gpu_model="G", vram_mb=-1)

    def test_task_create_priority_bounds(self):
        req = TaskCreateRequest(task_type="test", prompt="test", priority=1)
        assert req.priority == 1

        with pytest.raises(Exception):
            TaskCreateRequest(task_type="test", prompt="test", priority=0)

        with pytest.raises(Exception):
            TaskCreateRequest(task_type="test", prompt="test", priority=11)

    def test_task_result_valid(self):
        req = TaskResultRequest(
            task_id="abc123",
            result_text="Analysis result",
            tokens_generated=50,
            generation_time_ms=1200,
        )
        assert req.task_id == "abc123"

    def test_task_result_empty_text_rejected(self):
        with pytest.raises(Exception):
            TaskResultRequest(task_id="abc", result_text="")

    def test_network_stats_response(self):
        stats = NetworkStatsResponse(
            nodes_online=5,
            nodes_total=8,
            vram_total_gb=76.0,
            tasks_today=342,
        )
        assert stats.nodes_online == 5
        assert stats.vram_total_gb == 76.0

    def test_leaderboard_response(self):
        entry = LeaderboardEntry(
            rank=1,
            name="FlowUP",
            gpu_model="RTX 5080",
            vram_mb=16384,
            tasks_completed=1203,
        )
        assert entry.rank == 1
        resp = LeaderboardResponse(entries=[entry], total_contributors=10)
        assert len(resp.entries) == 1


# ===================================================================
# Config defaults
# ===================================================================

class TestComputeConfig:
    """Test compute config defaults."""

    def test_compute_enabled_default(self):
        s = Settings()
        assert s.compute_enabled is True

    def test_compute_heartbeat_timeout(self):
        s = Settings()
        assert s.compute_heartbeat_timeout == 90

    def test_compute_task_timeout(self):
        s = Settings()
        assert s.compute_task_default_timeout == 300

    def test_compute_spot_check_rate(self):
        s = Settings()
        assert s.compute_spot_check_rate == 0.05

    def test_compute_max_retries(self):
        s = Settings()
        assert s.compute_max_retries == 3

    def test_compute_rate_limit(self):
        s = Settings()
        assert s.compute_rate_limit_per_minute == 100


# ===================================================================
# ComputeDatabaseProxy
# ===================================================================

class TestComputeDatabaseProxy:
    """Test the proxy used by long-lived workers."""

    def test_proxy_creates_method(self):
        proxy = ComputeDatabaseProxy()
        method = proxy.get_network_stats
        assert callable(method)

    def test_proxy_different_methods(self):
        proxy = ComputeDatabaseProxy()
        m1 = proxy.list_nodes
        m2 = proxy.get_leaderboard
        # Each call creates a new proxy function
        assert m1 is not m2


# ===================================================================
# Module imports
# ===================================================================

class TestModuleImports:
    """Test that all compute module components import correctly."""

    def test_import_compute_package(self):
        import nexus.compute
        assert hasattr(nexus.compute, "ComputeDatabase")
        assert hasattr(nexus.compute, "TaskDispatcher")
        assert hasattr(nexus.compute, "ComputeManager")
        assert hasattr(nexus.compute, "ComputeEventType")

    def test_import_db(self):
        from nexus.compute.db import ComputeDatabase, init_compute_db
        assert ComputeDatabase is not None

    def test_import_dispatcher(self):
        from nexus.compute.dispatcher import TaskDispatcher
        from nexus.compute.model_selector import MODEL_TIERS
        assert len(MODEL_TIERS) == 6

    def test_import_events(self):
        from nexus.compute.events import ComputeEventType
        assert len(ComputeEventType) >= 10

    def test_import_models(self):
        from nexus.compute.models import (
            NodeRegisterRequest, NodeRegisterResponse,
            TaskPullResponse, TaskResultRequest, TaskResultResponse,
            NetworkStatsResponse, LeaderboardResponse,
        )
        assert NodeRegisterRequest is not None

    def test_import_manager(self):
        from nexus.compute.manager import ComputeManager
        assert ComputeManager is not None
