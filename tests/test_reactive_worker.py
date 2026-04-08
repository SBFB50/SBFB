"""
Tests for the ReactiveWorker ABC (nexus/events/worker.py).

Uses a concrete _CounterWorker to exercise lifecycle, event handling,
circuit breaker, and metrics.
"""

import asyncio

import pytest
import pytest_asyncio

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import (
    ReactiveWorker,
    STATUS_CIRCUIT_OPEN,
    STATUS_IDLE,
    STATUS_STOPPED,
    _CB_CONSECUTIVE_FAILURES,
)


# ---------------------------------------------------------------------------
# Concrete test worker
# ---------------------------------------------------------------------------

class _CounterWorker(ReactiveWorker):
    """Minimal worker for testing.  Records received events."""

    name = "test_counter"
    subscriptions = [EventType.EVIDENCE_ADDED]

    def __init__(
        self,
        bus: EventBus,
        *,
        output_events: list[NexusEvent] | None = None,
        raise_error: bool = False,
    ) -> None:
        super().__init__(bus, queue_size=50)
        self.received: list[NexusEvent] = []
        self._output_events = output_events or []
        self._raise_error = raise_error

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        self.received.append(event)
        if self._raise_error:
            raise RuntimeError("intentional test error")
        return list(self._output_events)


def _make_event(
    event_type: EventType = EventType.EVIDENCE_ADDED,
    case_id: str = "case-1",
    **kwargs,
) -> NexusEvent:
    return NexusEvent(event_type=event_type, case_id=case_id, **kwargs)


# ===================================================================
# TestWorkerLifecycle
# ===================================================================

class TestWorkerLifecycle:

    @pytest.mark.asyncio
    async def test_register_subscribes_to_bus(self, bus):
        worker = _CounterWorker(bus)
        worker.register()

        stats = bus.get_stats()
        assert stats["subscriber_count"]["evidence_added"] >= 1

    @pytest.mark.asyncio
    async def test_start_creates_task(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        task = worker.start()

        assert isinstance(task, asyncio.Task)
        assert not task.done()
        assert worker._started_at is not None

        await worker.stop()
        assert task.done()

    @pytest.mark.asyncio
    async def test_stop_graceful(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        worker.start()

        await worker.stop()
        status = worker.get_status()
        assert status["status"] == STATUS_STOPPED

    @pytest.mark.asyncio
    async def test_status_reporting(self, bus):
        worker = _CounterWorker(bus)
        status = worker.get_status()

        assert status["name"] == "test_counter"
        assert status["status"] == STATUS_IDLE
        assert status["events_processed"] == 0
        assert status["events_errored"] == 0
        assert status["queue_maxsize"] == 50
        assert "subscriptions" in status
        assert "evidence_added" in status["subscriptions"]


# ===================================================================
# TestWorkerEventHandling
# ===================================================================

class TestWorkerEventHandling:

    @pytest.mark.asyncio
    async def test_receives_and_handles_event(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        worker.start()

        event = _make_event(payload={"key": "val"})
        await bus.publish(event)

        # Give the worker loop time to process
        await asyncio.sleep(0.1)

        assert len(worker.received) == 1
        assert worker.received[0].event_id == event.event_id
        assert worker.get_status()["events_processed"] == 1

        await worker.stop()

    @pytest.mark.asyncio
    async def test_publishes_output_events(self, bus):
        output = _make_event(
            event_type=EventType.ENTITY_DISCOVERED,
            payload={"entity_id": "e-1"},
            source_worker="test_counter",
        )
        worker = _CounterWorker(bus, output_events=[output])
        worker.register()
        worker.start()

        # Subscribe to the output event type
        out_q: asyncio.Queue = asyncio.Queue(maxsize=10)
        bus.subscribe(EventType.ENTITY_DISCOVERED, out_q)

        await bus.publish(_make_event())
        await asyncio.sleep(0.1)

        assert not out_q.empty()
        published = out_q.get_nowait()
        assert published.event_type == EventType.ENTITY_DISCOVERED

        await worker.stop()

    @pytest.mark.asyncio
    async def test_error_increments_counter(self, bus):
        worker = _CounterWorker(bus, raise_error=True)
        worker.register()
        worker.start()

        await bus.publish(_make_event())
        await asyncio.sleep(0.1)

        status = worker.get_status()
        assert status["events_errored"] == 1
        assert status["consecutive_errors"] == 1
        assert status["last_error"] is not None

        await worker.stop()

    @pytest.mark.asyncio
    async def test_multiple_events_processed_sequentially(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        worker.start()

        for i in range(5):
            await bus.publish(_make_event(payload={"i": i}))

        await asyncio.sleep(0.3)

        assert len(worker.received) == 5
        assert worker.get_status()["events_processed"] == 5
        # Verify order preserved
        for i, evt in enumerate(worker.received):
            assert evt.payload["i"] == i

        await worker.stop()

    @pytest.mark.asyncio
    async def test_marks_event_processed_in_bus(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        worker.start()

        event = _make_event()
        await bus.publish(event)
        await asyncio.sleep(0.1)

        # Check event_log status
        import aiosqlite
        async with aiosqlite.connect(bus._db_path) as db:
            cursor = await db.execute(
                "SELECT status, processed_by FROM event_log WHERE id = ?",
                (event.event_id,),
            )
            row = await cursor.fetchone()

        assert row[0] == "processed"
        assert row[1] == "test_counter"

        await worker.stop()


# ===================================================================
# TestWorkerCircuitBreaker
# ===================================================================

class TestWorkerCircuitBreaker:

    @pytest.mark.asyncio
    async def test_circuit_breaker_opens_after_consecutive_failures(self, bus):
        worker = _CounterWorker(bus, raise_error=True)
        worker.register()
        worker.start()

        # Send enough events to trip the breaker
        for _ in range(_CB_CONSECUTIVE_FAILURES):
            await bus.publish(_make_event())

        await asyncio.sleep(0.3)

        status = worker.get_status()
        assert status["consecutive_errors"] >= _CB_CONSECUTIVE_FAILURES
        assert status["status"] in (STATUS_CIRCUIT_OPEN, "error")

        await worker.stop()

    @pytest.mark.asyncio
    async def test_circuit_breaker_closes_on_success(self, bus):
        """After errors, a successful handle() resets the breaker."""
        worker = _CounterWorker(bus, raise_error=True)
        worker.register()
        worker.start()

        # Cause errors
        for _ in range(3):
            await bus.publish(_make_event())
        await asyncio.sleep(0.2)

        assert worker._consecutive_errors == 3

        # Now fix the worker
        worker._raise_error = False
        await bus.publish(_make_event())
        await asyncio.sleep(0.2)

        assert worker._consecutive_errors == 0
        assert worker.get_status()["events_processed"] == 1

        await worker.stop()

    @pytest.mark.asyncio
    async def test_sentinel_breaks_during_backoff(self, bus):
        """Worker should exit cleanly even during circuit breaker backoff."""
        worker = _CounterWorker(bus, raise_error=True)
        # Use tiny queue to ensure we can stop quickly
        worker._queue = asyncio.Queue(maxsize=50)
        worker.register()
        worker.start()

        # Trip the circuit breaker
        for _ in range(_CB_CONSECUTIVE_FAILURES):
            await bus.publish(_make_event())
        await asyncio.sleep(0.3)

        # Stop should succeed within timeout even during backoff
        await asyncio.wait_for(worker.stop(), timeout=5.0)
        assert worker.get_status()["status"] == STATUS_STOPPED


# ===================================================================
# TestWorkerMetrics
# ===================================================================

class TestWorkerMetrics:

    @pytest.mark.asyncio
    async def test_avg_processing_time_tracked(self, bus):
        worker = _CounterWorker(bus)
        worker.register()
        worker.start()

        await bus.publish(_make_event())
        await asyncio.sleep(0.1)

        status = worker.get_status()
        assert status["avg_processing_ms"] >= 0.0
        assert status["events_processed"] == 1

        await worker.stop()

    @pytest.mark.asyncio
    async def test_get_status_shape(self, bus):
        worker = _CounterWorker(bus)
        status = worker.get_status()

        expected_keys = {
            "name", "status", "queue_size", "queue_maxsize",
            "events_processed", "events_errored", "consecutive_errors",
            "avg_processing_ms", "last_event_at", "last_error",
            "started_at", "subscriptions",
        }
        assert set(status.keys()) == expected_keys
