"""
Tests for MonitoringLoop (nexus/events/monitoring_loop.py).

Tests sweep lifecycle, job limits, stats tracking with mocked
monitors and DB.
"""

import asyncio
from datetime import datetime, timezone
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from nexus.events.bus import EventBus
from nexus.events.monitoring_loop import MonitoringLoop


def _make_loop(bus, db_rows=None, sweep_interval=1.0, rate_limit=0.01):
    """Create a MonitoringLoop with mocked router/chroma."""
    router = AsyncMock()
    router.embed = AsyncMock(return_value=[0.1] * 384)
    chroma = MagicMock()

    loop = MonitoringLoop(
        bus=bus,
        router=router,
        chroma=chroma,
        case_id="case-1",
        sweep_interval=sweep_interval,
        rate_limit=rate_limit,
    )
    return loop


# ===================================================================
# TestMonitoringLoopLifecycle
# ===================================================================

class TestMonitoringLoopLifecycle:

    @pytest.mark.asyncio
    async def test_start_creates_background_task(self, bus):
        loop = _make_loop(bus)
        await loop.start()
        assert loop._running is True
        assert loop._task is not None
        assert not loop._task.done()
        await loop.stop()

    @pytest.mark.asyncio
    async def test_stop_cancels_task(self, bus):
        loop = _make_loop(bus)
        await loop.start()
        await loop.stop()
        assert loop._running is False

    @pytest.mark.asyncio
    async def test_stats_tracking(self, bus):
        loop = _make_loop(bus)
        stats = loop.get_stats()
        assert stats["running"] is False
        assert stats["case_id"] == "case-1"
        assert stats["sweeps"] == 0
        assert stats["jobs_executed"] == 0
        assert stats["jobs_timed_out"] == 0
        assert stats["results_stored"] == 0

    @pytest.mark.asyncio
    async def test_double_start_noop(self, bus):
        loop = _make_loop(bus)
        await loop.start()
        task1 = loop._task
        await loop.start()  # Should not create a new task
        assert loop._task is task1
        await loop.stop()


# ===================================================================
# TestSweepExecution
# ===================================================================

class TestSweepExecution:

    @pytest.mark.asyncio
    async def test_sweep_with_no_due_jobs(self, bus):
        loop = _make_loop(bus)

        with patch("nexus.events.monitoring_loop.get_db") as mock_get_db:
            mock_conn = AsyncMock()
            mock_cursor = AsyncMock()
            mock_cursor.fetchall = AsyncMock(return_value=[])
            mock_conn.execute = AsyncMock(return_value=mock_cursor)
            mock_get_db.return_value.__aenter__ = AsyncMock(return_value=mock_conn)
            mock_get_db.return_value.__aexit__ = AsyncMock(return_value=False)

            await loop._sweep_once()

        assert loop._jobs_executed == 0

    @pytest.mark.asyncio
    async def test_sweep_respects_max_jobs_limit(self, bus):
        loop = _make_loop(bus)

        with patch("nexus.events.monitoring_loop.get_db") as mock_get_db:
            mock_conn = AsyncMock()
            mock_cursor = AsyncMock()
            mock_cursor.fetchall = AsyncMock(return_value=[])
            mock_conn.execute = AsyncMock(return_value=mock_cursor)
            mock_get_db.return_value.__aenter__ = AsyncMock(return_value=mock_conn)
            mock_get_db.return_value.__aexit__ = AsyncMock(return_value=False)

            await loop._sweep_once()

            # Verify LIMIT parameter was passed in SQL query
            call_args = mock_conn.execute.call_args
            sql = call_args[0][0]
            assert "LIMIT" in sql


# ===================================================================
# TestGetBeforeDate (adaptive time window)
# ===================================================================

class TestAdaptiveTimeWindow:

    @pytest.mark.asyncio
    async def test_notify_results_advances_window(self, bus):
        loop = _make_loop(bus)
        loop._current_window_year = 2005
        loop._crime_year = 2002

        # No results: dry sweep count increases
        loop.notify_results_found(0)
        assert loop._dry_sweeps >= 1

    @pytest.mark.asyncio
    async def test_notify_results_resets_dry_count(self, bus):
        loop = _make_loop(bus)
        loop._dry_sweeps = 5
        loop.notify_results_found(3)
        assert loop._dry_sweeps == 0
