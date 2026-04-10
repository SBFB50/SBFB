"""
Tests for Phase 5 — Dashboard public + gamification (badges).

Covers:
- Badge calculation logic (task milestones, VRAM threshold, early adopter)
- Badge DB CRUD (award, get, summary)
- Badge table creation
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
    """Create in-memory DB with all compute tables."""
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA foreign_keys = ON")
    await conn.executescript(_COMPUTE_CREATE_TABLES)
    await conn.executescript(_COMPUTE_CREATE_INDEXES)
    await conn.commit()
    yield ComputeDatabase(conn)
    await conn.close()


class TestBadgeCalculation:
    """Test badge award logic."""

    @pytest.mark.asyncio
    async def test_first_task_badge(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=1)
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "first_task" in badge_ids

    @pytest.mark.asyncio
    async def test_centurion_badge(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=100)
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "first_task" in badge_ids
        assert "centurion" in badge_ids

    @pytest.mark.asyncio
    async def test_millionnaire_badge(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=1000)
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "millionnaire" in badge_ids

    @pytest.mark.asyncio
    async def test_pilier_badge(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=10000)
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "pilier" in badge_ids

    @pytest.mark.asyncio
    async def test_power_node_badge(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="RTX 4090", vram_mb=24577, ip="1.1.1.1")
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "power_node" in badge_ids

    @pytest.mark.asyncio
    async def test_power_node_not_awarded_below_threshold(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="A", gpu_model="RTX 5080", vram_mb=16384, ip="1.1.1.1")
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "power_node" not in badge_ids

    @pytest.mark.asyncio
    async def test_early_adopter_badge(self, db: ComputeDatabase):
        # First node registered
        node, _ = await db.register_node(name="First", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "early_adopter" in badge_ids

    @pytest.mark.asyncio
    async def test_early_adopter_not_after_10(self, db: ComputeDatabase):
        # Register 10 nodes first
        for i in range(10):
            await db.register_node(name=f"Node{i}", gpu_model="G", vram_mb=16384, ip=f"1.1.1.{i}")
        # 11th node
        node, _ = await db.register_node(name="Late", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        badges = await db.calculate_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        assert "early_adopter" not in badge_ids

    @pytest.mark.asyncio
    async def test_no_badges_for_zero_tasks(self, db: ComputeDatabase):
        node, _ = await db.register_node(name="New", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        badges = await db.calculate_badges(node["id"])
        # Only early_adopter (first node) — no task badges
        badge_ids = [b["badge_id"] for b in badges]
        assert "first_task" not in badge_ids
        assert "centurion" not in badge_ids

    @pytest.mark.asyncio
    async def test_badges_idempotent(self, db: ComputeDatabase):
        """Calling calculate_badges twice should not duplicate badges."""
        node, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        await db.increment_node_stats(node["id"], completed=100)
        await db.calculate_badges(node["id"])
        await db.calculate_badges(node["id"])  # Second call
        badges = await db.get_node_badges(node["id"])
        badge_ids = [b["badge_id"] for b in badges]
        # No duplicates
        assert len(badge_ids) == len(set(badge_ids))

    @pytest.mark.asyncio
    async def test_nonexistent_node_returns_empty(self, db: ComputeDatabase):
        badges = await db.calculate_badges("nonexistent")
        assert badges == []


class TestBadgeSummary:
    """Test badge summary across all nodes."""

    @pytest.mark.asyncio
    async def test_summary_empty(self, db: ComputeDatabase):
        summary = await db.get_all_badges_summary()
        assert summary == []

    @pytest.mark.asyncio
    async def test_summary_with_badges(self, db: ComputeDatabase):
        n1, _ = await db.register_node(name="A", gpu_model="G", vram_mb=16384, ip="1.1.1.1")
        n2, _ = await db.register_node(name="B", gpu_model="G", vram_mb=16384, ip="2.2.2.2")
        await db.increment_node_stats(n1["id"], completed=1)
        await db.increment_node_stats(n2["id"], completed=1)
        await db.calculate_badges(n1["id"])
        await db.calculate_badges(n2["id"])

        summary = await db.get_all_badges_summary()
        # Both nodes should have first_task + early_adopter
        summary_dict = {s["badge_id"]: s["count"] for s in summary}
        assert summary_dict.get("first_task", 0) == 2
        assert summary_dict.get("early_adopter", 0) == 2
