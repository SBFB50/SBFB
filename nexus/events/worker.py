"""
NEXUS -- ReactiveWorker abstract base class.

Every worker in the reactive pipeline inherits from ReactiveWorker:
- Owns an asyncio.Queue (bounded, maxsize=500)
- run() loop: pull event -> handle() -> publish output events
- handle() is abstract -- subclasses implement domain logic
- Built-in error handling, status tracking, metrics, and circuit breaker
"""

from __future__ import annotations

import asyncio
import logging
import time
from abc import ABC, abstractmethod
from datetime import datetime, timezone
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Worker status constants
# ---------------------------------------------------------------------------

STATUS_IDLE = "idle"
STATUS_PROCESSING = "processing"
STATUS_ERROR = "error"
STATUS_STOPPED = "stopped"
STATUS_CIRCUIT_OPEN = "circuit_open"

_DEFAULT_QUEUE_SIZE = 500

# Circuit breaker thresholds
_CB_CONSECUTIVE_FAILURES = 5
_CB_INITIAL_BACKOFF_S = 30.0
_CB_EXTENDED_BACKOFF_S = 60.0


class ReactiveWorker(ABC):
    """Base class for all event-driven workers.

    Subclasses must implement:
        - ``name``         : class attribute or property identifying the worker
        - ``subscriptions``: class attribute listing EventTypes to listen to
        - ``handle(event)``  : process one event, return list of output events

    Example::

        class EvidenceChunker(ReactiveWorker):
            name = "evidence_chunker"
            subscriptions = [EventType.EVIDENCE_PROCESSED]

            async def handle(self, event: NexusEvent) -> list[NexusEvent]:
                chunks = await self._chunk(event.payload["evidence_id"])
                return [NexusEvent(
                    event_type=EventType.EVIDENCE_CHUNKED,
                    case_id=event.case_id,
                    payload={"chunk_count": len(chunks)},
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )]
    """

    # Subclasses must set these
    name: str = "unnamed_worker"
    subscriptions: list[EventType] = []

    def __init__(self, bus: EventBus, queue_size: int = _DEFAULT_QUEUE_SIZE) -> None:
        self._bus = bus
        self._queue: asyncio.Queue[NexusEvent | None] = asyncio.Queue(
            maxsize=queue_size,
        )
        self._task: asyncio.Task | None = None

        # Status tracking
        self._status: str = STATUS_IDLE
        self._events_processed: int = 0
        self._events_errored: int = 0
        self._last_event_at: str | None = None
        self._last_error: str | None = None
        self._started_at: str | None = None
        self._total_processing_ms: float = 0.0

        # Circuit breaker state
        self._consecutive_errors: int = 0
        self._events_dropped: int = 0

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def register(self) -> None:
        """Subscribe this worker's queue to its declared event types."""
        for etype in self.subscriptions:
            self._bus.subscribe(etype, self._queue)
        logger.info(
            "Worker [%s] registered for %s",
            self.name,
            [e.value for e in self.subscriptions],
        )

    def start(self) -> asyncio.Task:
        """Launch the run loop as an asyncio Task."""
        self._task = asyncio.create_task(self._run(), name=f"worker-{self.name}")
        self._started_at = datetime.now(timezone.utc).isoformat()
        self._status = STATUS_IDLE
        logger.info("Worker [%s] started", self.name)
        return self._task

    async def stop(self) -> None:
        """Signal the worker to shut down gracefully."""
        # Sentinel None breaks the run loop
        try:
            self._queue.put_nowait(None)
        except asyncio.QueueFull:
            pass

        if self._task and not self._task.done():
            try:
                await asyncio.wait_for(self._task, timeout=5.0)
            except asyncio.TimeoutError:
                self._task.cancel()
                logger.warning("Worker [%s] cancelled after timeout", self.name)

        self._status = STATUS_STOPPED
        logger.info(
            "Worker [%s] stopped  processed=%d  errors=%d",
            self.name,
            self._events_processed,
            self._events_errored,
        )

    # ------------------------------------------------------------------
    # Core loop
    # ------------------------------------------------------------------

    async def _run(self) -> None:
        """Main loop: pull events, handle, publish outputs.

        Includes a circuit breaker: after ``_CB_CONSECUTIVE_FAILURES``
        errors in a row the worker enters ``STATUS_CIRCUIT_OPEN``, pauses
        for a backoff period, then retries a single event (half-open).
        """
        while True:
            # ---- Circuit breaker: open state ----
            if self._consecutive_errors >= _CB_CONSECUTIVE_FAILURES:
                backoff = (
                    _CB_INITIAL_BACKOFF_S
                    if self._consecutive_errors == _CB_CONSECUTIVE_FAILURES
                    else _CB_EXTENDED_BACKOFF_S
                )
                self._status = STATUS_CIRCUIT_OPEN
                logger.warning(
                    "Worker [%s] circuit breaker OPEN after %d consecutive errors "
                    "-- backing off %.0fs",
                    self.name,
                    self._consecutive_errors,
                    backoff,
                )
                # Sleep but remain responsive to shutdown signals:
                # peek into the queue for a sentinel None during backoff.
                try:
                    peek = await asyncio.wait_for(
                        self._queue.get(), timeout=backoff,
                    )
                    if peek is None:
                        self._queue.task_done()
                        break
                    # Got a real event during backoff -- mark the get() as
                    # done, then put the event back for normal processing.
                    self._queue.task_done()
                    try:
                        self._queue.put_nowait(peek)
                    except asyncio.QueueFull:
                        self._events_dropped += 1
                        logger.warning(
                            "Worker [%s] dropped event during circuit breaker backoff "
                            "(queue full)",
                            self.name,
                        )
                except asyncio.TimeoutError:
                    pass  # Backoff elapsed normally

            self._status = STATUS_IDLE
            event = await self._queue.get()

            # Sentinel None = shutdown signal
            if event is None:
                self._queue.task_done()
                break

            self._status = STATUS_PROCESSING
            t0 = time.monotonic()
            try:
                output_events = await self.handle(event)

                # Publish any output events
                if output_events:
                    for out_event in output_events:
                        await self._bus.publish(out_event)

                # Mark processed in persistent log
                await self._bus.mark_processed(event.event_id, self.name)

                elapsed_ms = (time.monotonic() - t0) * 1000
                self._total_processing_ms += elapsed_ms
                self._events_processed += 1
                self._last_event_at = datetime.now(timezone.utc).isoformat()

                # Success: reset circuit breaker
                if self._consecutive_errors > 0:
                    logger.info(
                        "Worker [%s] circuit breaker CLOSED (recovered after %d errors)",
                        self.name,
                        self._consecutive_errors,
                    )
                    self._consecutive_errors = 0

                logger.debug(
                    "Worker [%s] handled %s in %.1fms",
                    self.name,
                    event.event_type.value,
                    elapsed_ms,
                )

            except Exception as exc:
                self._events_errored += 1
                self._consecutive_errors += 1
                self._status = STATUS_ERROR
                self._last_error = f"{event.event_type.value} @ {event.event_id}"
                logger.error(
                    "Worker [%s] error handling %s (event=%s, consecutive_errors=%d): %s",
                    self.name,
                    event.event_type.value,
                    event.event_id,
                    self._consecutive_errors,
                    exc,
                )
            finally:
                self._queue.task_done()

    # ------------------------------------------------------------------
    # Abstract interface
    # ------------------------------------------------------------------

    @abstractmethod
    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        """Process a single event. Return zero or more output events.

        Must be overridden by every concrete worker.  Returning an empty
        list is fine -- it just means no downstream events are produced.
        """
        ...

    # ------------------------------------------------------------------
    # Status / Metrics
    # ------------------------------------------------------------------

    def get_status(self) -> dict[str, Any]:
        """Return a status dict suitable for the monitoring UI."""
        avg_ms = (
            self._total_processing_ms / self._events_processed
            if self._events_processed > 0
            else 0.0
        )
        return {
            "name": self.name,
            "status": self._status,
            "queue_size": self._queue.qsize(),
            "queue_maxsize": self._queue.maxsize,
            "events_processed": self._events_processed,
            "events_errored": self._events_errored,
            "consecutive_errors": self._consecutive_errors,
            "avg_processing_ms": round(avg_ms, 1),
            "last_event_at": self._last_event_at,
            "last_error": self._last_error,
            "started_at": self._started_at,
            "subscriptions": [e.value for e in self.subscriptions],
        }
