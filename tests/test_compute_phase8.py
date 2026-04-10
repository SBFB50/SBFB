"""
Tests for Phase 8 — Swarm public permanent.

Covers:
- Uptime logging (connect, disconnect, duration)
- Node uptime stats (total, sessions, current)
- Network uptime (30d hours, 7d streak, uptime %)
- Contributor impact (tasks by type, tokens, percentile)
"""

import aiosqlite
import pytest
import pytest_asyncio

from nexus.compute.db import (
    ComputeDatabase,
    _COMPUTE_CREATE_TABLES,
    _COMPUTE_CREATE_INDEXES,
)


@pytest_asyncio.fixture
async def db():
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA foreign_keys = ON")
    await conn.executescript(_COMPUTE_CREATE_TABLES)
    await conn.executescript(_COMPUTE_CREATE_INDEXES)
    await conn.commit()
    yield ComputeDatabase(conn)
    await conn.close()


# ===================================================================
# Uptime tracking
# ===================================================================

class TestUptimeLogging:
    """Test uptime connect/disconnect logging."""

    @pytest.mark.asyncio
    async def test_log_connect(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        log_id = await db.log_connect(node["id"])
        assert log_id is not None
        assert len(log_id) > 0

    @pytest.mark.asyncio
    async def test_log_disconnect(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.log_connect(node["id"])
        await db.log_disconnect(node["id"])
        # Should have duration set
        cursor = await db._conn.execute(
            "SELECT duration_seconds FROM compute_uptime_log WHERE node_id = ?",
            (node["id"],),
        )
        row = await cursor.fetchone()
        assert row is not None
        assert row[0] >= 0  # Duration at least 0 (instant connect/disconnect)

    @pytest.mark.asyncio
    async def test_multiple_sessions(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.log_connect(node["id"])
        await db.log_disconnect(node["id"])
        await db.log_connect(node["id"])
        await db.log_disconnect(node["id"])
        uptime = await db.get_node_uptime(node["id"])
        assert uptime["sessions"] == 2


class TestNodeUptime:
    """Test per-node uptime stats."""

    @pytest.mark.asyncio
    async def test_no_sessions(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        uptime = await db.get_node_uptime(node["id"])
        assert uptime["sessions"] == 0
        assert uptime["total_seconds"] == 0
        assert uptime["current_session_seconds"] == 0

    @pytest.mark.asyncio
    async def test_open_session(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.log_connect(node["id"])
        uptime = await db.get_node_uptime(node["id"])
        assert uptime["sessions"] == 1
        assert uptime["current_session_seconds"] >= 0

    @pytest.mark.asyncio
    async def test_closed_session(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.log_connect(node["id"])
        await db.log_disconnect(node["id"])
        uptime = await db.get_node_uptime(node["id"])
        assert uptime["sessions"] == 1
        assert uptime["current_session_seconds"] == 0


class TestNetworkUptime:
    """Test network-wide uptime stats."""

    @pytest.mark.asyncio
    async def test_empty_network(self, db: ComputeDatabase):
        uptime = await db.get_network_uptime()
        assert uptime["total_node_hours_30d"] == 0.0
        assert uptime["nodes_with_7d_streak"] == 0
        assert uptime["nodes_online"] == 0
        assert uptime["uptime_pct"] == 0.0

    @pytest.mark.asyncio
    async def test_with_nodes(self, db: ComputeDatabase):
        await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.register_node(name="B", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        uptime = await db.get_network_uptime()
        assert uptime["nodes_total"] == 2
        assert uptime["nodes_online"] == 2  # Both idle = online
        assert uptime["uptime_pct"] == 100.0


# ===================================================================
# Contributor impact
# ===================================================================

class TestNodeImpact:
    """Test contributor impact stats."""

    @pytest.mark.asyncio
    async def test_nonexistent_node(self, db: ComputeDatabase):
        impact = await db.get_node_impact("nonexistent")
        assert impact == {}

    @pytest.mark.asyncio
    async def test_new_node_impact(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="RTX 5080", vram_mb=16384, ip="1.1.1.1")
        impact = await db.get_node_impact(node["id"])
        assert impact["node_id"] == node["id"]
        assert impact["name"] == "A"
        assert impact["tasks_completed"] == 0
        assert impact["tasks_by_type"] == []
        assert impact["tokens_this_week"] == 0
        assert impact["percentile"] == 0

    @pytest.mark.asyncio
    async def test_node_with_tasks(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="Pro", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        n2, _ = await db.register_node(name="Newbie", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        await db.increment_node_stats(n1["id"], completed=100)
        await db.increment_node_stats(n2["id"], completed=10)

        impact = await db.get_node_impact(n1["id"])
        assert impact["tasks_completed"] == 100
        assert impact["percentile"] == 50  # 1 of 2 nodes below

    @pytest.mark.asyncio
    async def test_impact_includes_uptime(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.log_connect(node["id"])
        impact = await db.get_node_impact(node["id"])
        assert "uptime" in impact
        assert impact["uptime"]["sessions"] == 1
