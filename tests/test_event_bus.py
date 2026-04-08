"""
Tests for the EventBus (nexus/events/bus.py).

Covers lifecycle, pub/sub, persistence, circuit breaker, and NexusEvent.
"""

import asyncio
import json

import aiosqlite
import pytest
import pytest_asyncio

from nexus.events.bus import EventBus, _CIRCUIT_BREAKER_MAX, _CIRCUIT_BREAKER_WINDOW
from nexus.events.types import EventType, NexusEvent


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _make_event(
    event_type: EventType = EventType.EVIDENCE_ADDED,
    case_id: str = "case-1",
    **kwargs,
) -> NexusEvent:
    return NexusEvent(event_type=event_type, case_id=case_id, **kwargs)


# ===================================================================
# TestEventBusLifecycle
# ===================================================================

class TestEventBusLifecycle:

    @pytest.mark.asyncio
    async def test_start_creates_event_log_table(self, tmp_path):
        db_file = str(tmp_path / "lifecycle.db")
        bus = EventBus(db_path=db_file)
        await bus.start()

        async with aiosqlite.connect(db_file) as db:
            cursor = await db.execute(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='event_log'"
            )
            row = await cursor.fetchone()
        assert row is not None
        await bus.stop()

    @pytest.mark.asyncio
    async def test_stop_sends_sentinel_to_subscribers(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        await bus.stop()
        # After stop, sentinel None should be in the queue
        item = q.get_nowait()
        assert item is None

    @pytest.mark.asyncio
    async def test_publish_rejected_when_not_running(self, tmp_path):
        db_file = str(tmp_path / "notrunning.db")
        bus = EventBus(db_path=db_file)
        # Don't call start()
        result = await bus.publish(_make_event())
        assert result is False

    @pytest.mark.asyncio
    async def test_stats_after_operations(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        await bus.publish(_make_event())

        stats = bus.get_stats()
        assert stats["running"] is True
        assert stats["events_published"] >= 1
        assert "subscriber_count" in stats
        assert stats["subscriber_count"]["evidence_added"] == 1


# ===================================================================
# TestEventBusPubSub
# ===================================================================

class TestEventBusPubSub:

    @pytest.mark.asyncio
    async def test_publish_delivers_to_subscriber(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        event = _make_event(payload={"key": "val"})
        accepted = await bus.publish(event)

        assert accepted is True
        received = q.get_nowait()
        assert received.event_id == event.event_id
        assert received.payload == {"key": "val"}

    @pytest.mark.asyncio
    async def test_publish_to_multiple_subscribers(self, bus):
        q1: asyncio.Queue = asyncio.Queue(maxsize=10)
        q2: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q1)
        bus.subscribe(EventType.EVIDENCE_ADDED, q2)

        event = _make_event()
        await bus.publish(event)

        assert q1.get_nowait().event_id == event.event_id
        assert q2.get_nowait().event_id == event.event_id

    @pytest.mark.asyncio
    async def test_unsubscribe_stops_delivery(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)
        bus.unsubscribe(EventType.EVIDENCE_ADDED, q)

        await bus.publish(_make_event())
        assert q.empty()

    @pytest.mark.asyncio
    async def test_wrong_event_type_not_delivered(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        await bus.publish(_make_event(event_type=EventType.ANALYSIS_COMPLETED))
        assert q.empty()

    @pytest.mark.asyncio
    async def test_full_queue_drops_event(self, bus):
        q: asyncio.Queue = asyncio.Queue(maxsize=1)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        # Fill the queue
        await bus.publish(_make_event())
        # This one should be dropped for this subscriber
        dropped_before = bus.dropped_count
        await bus.publish(_make_event())
        assert bus.dropped_count > dropped_before

    @pytest.mark.asyncio
    async def test_event_persisted_before_fanout(self, bus):
        event = _make_event()
        await bus.publish(event)

        async with aiosqlite.connect(bus._db_path) as db:
            cursor = await db.execute(
                "SELECT id, event_type, case_id FROM event_log WHERE id = ?",
                (event.event_id,),
            )
            row = await cursor.fetchone()
        assert row is not None
        assert row[0] == event.event_id
        assert row[1] == "evidence_added"
        assert row[2] == "case-1"


# ===================================================================
# TestEventBusPersistence
# ===================================================================

class TestEventBusPersistence:

    @pytest.mark.asyncio
    async def test_persist_event_all_fields(self, bus):
        event = _make_event(
            payload={"evidence_id": "ev-42"},
            source_worker="test_worker",
        )
        await bus.publish(event)

        async with aiosqlite.connect(bus._db_path) as db:
            db.row_factory = aiosqlite.Row
            cursor = await db.execute(
                "SELECT * FROM event_log WHERE id = ?", (event.event_id,)
            )
            row = await cursor.fetchone()

        assert row["event_type"] == "evidence_added"
        assert row["case_id"] == "case-1"
        assert json.loads(row["payload"])["evidence_id"] == "ev-42"
        assert row["source_worker"] == "test_worker"

    @pytest.mark.asyncio
    async def test_mark_processed_updates_status(self, bus):
        event = _make_event()
        await bus.publish(event)

        await bus.mark_processed(event.event_id, "test_worker")

        async with aiosqlite.connect(bus._db_path) as db:
            db.row_factory = aiosqlite.Row
            cursor = await db.execute(
                "SELECT status, processed_by FROM event_log WHERE id = ?",
                (event.event_id,),
            )
            row = await cursor.fetchone()

        assert row["status"] == "processed"
        assert row["processed_by"] == "test_worker"

    @pytest.mark.asyncio
    async def test_replay_pending_on_start(self, tmp_path):
        """Events left as 'pending' are replayed when the bus restarts."""
        db_file = str(tmp_path / "replay.db")

        # Phase 1: create bus, publish event, stop without marking processed
        bus1 = EventBus(db_path=db_file)
        await bus1.start()
        event = _make_event()
        await bus1.publish(event)
        await bus1.stop()

        # Phase 2: new bus instance, subscribe, start => should replay
        bus2 = EventBus(db_path=db_file)
        q: asyncio.Queue = asyncio.Queue(maxsize=100)
        bus2.subscribe(EventType.EVIDENCE_ADDED, q)
        await bus2.start()

        assert not q.empty()
        replayed = q.get_nowait()
        assert replayed.event_id == event.event_id
        assert bus2._events_replayed >= 1
        await bus2.stop()

    @pytest.mark.asyncio
    async def test_replay_rate_limited_events(self, tmp_path):
        """Rate-limited events are also replayed on restart."""
        db_file = str(tmp_path / "replay_rl.db")
        bus1 = EventBus(db_path=db_file)
        await bus1.start()

        # Manually insert a rate_limited event
        async with aiosqlite.connect(db_file) as db:
            await db.execute(
                """
                INSERT INTO event_log (id, event_type, case_id, payload, status, created_at)
                VALUES (?, ?, ?, ?, 'rate_limited', datetime('now'))
                """,
                ("rl-evt-1", "evidence_added", "case-1", "{}"),
            )
            await db.commit()
        await bus1.stop()

        # Restart and check replay
        bus2 = EventBus(db_path=db_file)
        q: asyncio.Queue = asyncio.Queue(maxsize=100)
        bus2.subscribe(EventType.EVIDENCE_ADDED, q)
        await bus2.start()

        assert not q.empty()
        assert bus2._events_replayed >= 1
        await bus2.stop()

    @pytest.mark.asyncio
    async def test_cleanup_old_events(self, tmp_path):
        """Old processed events are cleaned up."""
        db_file = str(tmp_path / "cleanup.db")
        bus = EventBus(db_path=db_file)
        await bus.start()

        # Insert an old processed event (>7 days ago)
        async with aiosqlite.connect(db_file) as db:
            await db.execute(
                """
                INSERT INTO event_log (id, event_type, case_id, payload, status, created_at)
                VALUES (?, ?, ?, ?, 'processed', datetime('now', '-10 days'))
                """,
                ("old-evt-1", "evidence_added", "case-1", "{}"),
            )
            await db.commit()

        await bus._cleanup_event_log()

        async with aiosqlite.connect(db_file) as db:
            cursor = await db.execute(
                "SELECT id FROM event_log WHERE id = 'old-evt-1'"
            )
            row = await cursor.fetchone()
        assert row is None
        await bus.stop()


# ===================================================================
# TestEventBusCircuitBreaker
# ===================================================================

class TestEventBusCircuitBreaker:

    @pytest.mark.asyncio
    async def test_circuit_breaker_trips_after_max(self, bus):
        """Publishing >_CIRCUIT_BREAKER_MAX events of the same type trips the breaker."""
        for i in range(_CIRCUIT_BREAKER_MAX):
            result = await bus.publish(_make_event())
            assert result is True, f"Event {i} should be accepted"

        # The next event should be rate-limited
        result = await bus.publish(_make_event())
        assert result is False

    @pytest.mark.asyncio
    async def test_rate_limited_event_still_persisted(self, bus):
        """A rate-limited event is persisted with status='rate_limited'."""
        for _ in range(_CIRCUIT_BREAKER_MAX):
            await bus.publish(_make_event())

        rl_event = _make_event()
        result = await bus.publish(rl_event)
        assert result is False

        async with aiosqlite.connect(bus._db_path) as db:
            cursor = await db.execute(
                "SELECT status FROM event_log WHERE id = ?",
                (rl_event.event_id,),
            )
            row = await cursor.fetchone()
        assert row[0] == "rate_limited"

    @pytest.mark.asyncio
    async def test_different_event_types_independent(self, bus):
        """Publishing many of type A doesn't trip the breaker for type B."""
        for _ in range(_CIRCUIT_BREAKER_MAX):
            await bus.publish(_make_event(event_type=EventType.EVIDENCE_ADDED))

        # Type A is now limited
        assert await bus.publish(_make_event(event_type=EventType.EVIDENCE_ADDED)) is False

        # Type B should still be accepted
        assert await bus.publish(_make_event(event_type=EventType.ANALYSIS_COMPLETED)) is True

    @pytest.mark.asyncio
    async def test_circuit_breaker_resets_after_window(self, bus):
        """After the rate window elapses, events are accepted again."""
        import time as _time
        from unittest.mock import patch

        # Trip the breaker
        for _ in range(_CIRCUIT_BREAKER_MAX):
            await bus.publish(_make_event())

        assert await bus.publish(_make_event()) is False

        # Fast-forward time past the window
        future_time = _time.monotonic() + _CIRCUIT_BREAKER_WINDOW + 1.0
        with patch("time.monotonic", return_value=future_time):
            result = await bus.publish(_make_event())
        assert result is True


# ===================================================================
# TestNexusEvent
# ===================================================================

class TestNexusEvent:

    def test_event_is_frozen(self):
        event = _make_event()
        with pytest.raises(AttributeError):
            event.case_id = "other"  # type: ignore[misc]

    def test_event_defaults(self):
        event = _make_event()
        assert event.event_id  # UUID generated
        assert event.timestamp  # ISO timestamp generated
        assert event.source_worker == ""
        assert event.parent_event_id is None
        assert event.payload == {}

    def test_event_parent_chain(self):
        parent = _make_event()
        child = _make_event(parent_event_id=parent.event_id)
        assert child.parent_event_id == parent.event_id
        assert child.event_id != parent.event_id
