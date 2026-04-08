"""
NEXUS -- Central event bus with SQLite persistence.

The EventBus is the backbone of the reactive architecture:
- Publish/subscribe via asyncio.Queue per subscriber
- Every event persisted to the ``event_log`` table before fan-out
- Circuit breaker: max events per type per minute to prevent storms
  Events are always persisted BEFORE the circuit breaker check so that
  rate-limited events are preserved (status='rate_limited') and can be
  replayed on restart instead of being permanently lost.
- Replay: unprocessed and rate-limited events re-delivered on startup
"""

from __future__ import annotations

import asyncio
import json
import logging
import time
from collections import defaultdict
from datetime import datetime, timedelta, timezone
from typing import Any

import aiosqlite

from nexus.config import settings
from nexus.events.types import EventType, NexusEvent

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_CIRCUIT_BREAKER_MAX = 100        # max events per type per 60-second window
_CIRCUIT_BREAKER_WINDOW = 60.0    # seconds
_REPLAY_BATCH_SIZE = 200          # events per replay batch


# ---------------------------------------------------------------------------
# SQL DDL for the event_log table
# ---------------------------------------------------------------------------

_EVENT_LOG_DDL = """
CREATE TABLE IF NOT EXISTS event_log (
    id TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    case_id TEXT NOT NULL,
    payload TEXT,
    source_worker TEXT,
    parent_event_id TEXT,
    status TEXT DEFAULT 'pending',
    processed_by TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    processed_at DATETIME
);
CREATE INDEX IF NOT EXISTS idx_event_log_status ON event_log(status);
CREATE INDEX IF NOT EXISTS idx_event_log_type ON event_log(event_type);
CREATE INDEX IF NOT EXISTS idx_event_log_case ON event_log(case_id);
"""


# ---------------------------------------------------------------------------
# EventBus
# ---------------------------------------------------------------------------

class EventBus:
    """Central publish/subscribe event bus with persistence and rate limiting.

    Usage::

        bus = EventBus()
        await bus.start()

        q = asyncio.Queue(maxsize=100)
        bus.subscribe(EventType.EVIDENCE_ADDED, q)

        await bus.publish(NexusEvent(
            event_type=EventType.EVIDENCE_ADDED,
            case_id="case-1",
            payload={"evidence_id": "ev-1"},
        ))

        event = await q.get()
        await bus.stop()
    """

    def __init__(self, db_path: str | None = None) -> None:
        self._db_path = db_path or str(settings.sqlite_path)
        self._subscriptions: dict[EventType, list[asyncio.Queue]] = defaultdict(list)
        self._running = False

        # Circuit breaker state: {EventType: deque of timestamps}
        self._rate_windows: dict[EventType, list[float]] = defaultdict(list)

        # Counters for stats
        self._events_published = 0
        self._events_dropped = 0
        self._events_replayed = 0

    @property
    def published_count(self) -> int:
        return self._events_published

    @property
    def dropped_count(self) -> int:
        return self._events_dropped

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Initialise the event_log table and replay unprocessed events."""
        settings.data_dir.mkdir(parents=True, exist_ok=True)
        async with aiosqlite.connect(self._db_path) as db:
            await db.executescript(_EVENT_LOG_DDL)
            await db.commit()

        await self._replay_pending()
        self._running = True
        logger.info("EventBus started (db=%s)", self._db_path)

    async def stop(self) -> None:
        """Drain all subscriber queues and shut down."""
        self._running = False
        for queues in self._subscriptions.values():
            for q in queues:
                # Put a sentinel None so blocked consumers can wake up
                try:
                    q.put_nowait(None)
                except asyncio.QueueFull:
                    pass
        logger.info(
            "EventBus stopped  published=%d  dropped=%d  replayed=%d",
            self._events_published,
            self._events_dropped,
            self._events_replayed,
        )

    # ------------------------------------------------------------------
    # Pub / Sub
    # ------------------------------------------------------------------

    def subscribe(self, event_type: EventType, queue: asyncio.Queue) -> None:
        """Register *queue* to receive events of *event_type*."""
        self._subscriptions[event_type].append(queue)
        logger.debug("Subscribed queue to %s (total=%d)",
                      event_type.value, len(self._subscriptions[event_type]))

    def unsubscribe(self, event_type: EventType, queue: asyncio.Queue) -> None:
        """Remove *queue* from *event_type* subscribers."""
        try:
            self._subscriptions[event_type].remove(queue)
        except ValueError:
            pass

    async def publish(self, event: NexusEvent) -> bool:
        """Persist *event* then fan-out to subscriber queues.

        Returns ``True`` if the event was accepted, ``False`` if the
        circuit breaker tripped (rate limit exceeded).

        Events are **always** persisted to SQLite before the circuit
        breaker is checked.  Rate-limited events are marked
        ``'rate_limited'`` so they can be replayed on restart instead
        of being lost permanently.
        """
        if not self._running:
            logger.warning("EventBus not running -- dropping %s", event.event_type.value)
            self._events_dropped += 1
            return False

        # 1. Persist FIRST -- the event is never lost from this point
        await self._persist(event)

        # 2. Circuit breaker check (after persistence)
        if self._is_rate_limited(event.event_type):
            logger.warning(
                "Circuit breaker tripped for %s (%d events in last %ds) "
                "-- event %s persisted as 'rate_limited'",
                event.event_type.value,
                _CIRCUIT_BREAKER_MAX,
                int(_CIRCUIT_BREAKER_WINDOW),
                event.event_id,
            )
            await self._update_event_status(event.event_id, "rate_limited")
            self._events_dropped += 1
            return False

        # 3. Fan-out to subscribers
        await self._fan_out(event)

        self._events_published += 1
        return True

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    def get_stats(self) -> dict[str, Any]:
        """Return runtime statistics for monitoring / the UI."""
        queue_sizes: dict[str, list[int]] = {}
        for etype, queues in self._subscriptions.items():
            queue_sizes[etype.value] = [q.qsize() for q in queues]

        return {
            "running": self._running,
            "events_published": self._events_published,
            "events_dropped": self._events_dropped,
            "events_replayed": self._events_replayed,
            "subscriber_count": {
                k.value: len(v) for k, v in self._subscriptions.items()
            },
            "queue_sizes": queue_sizes,
        }

    # ------------------------------------------------------------------
    # Persistence
    # ------------------------------------------------------------------

    async def _persist(self, event: NexusEvent) -> None:
        """Write event to SQLite event_log."""
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                INSERT INTO event_log
                    (id, event_type, case_id, payload, source_worker,
                     parent_event_id, status, created_at)
                VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)
                """,
                (
                    event.event_id,
                    event.event_type.value,
                    event.case_id,
                    json.dumps(event.payload, ensure_ascii=False, default=str),
                    event.source_worker,
                    event.parent_event_id,
                    event.timestamp,
                ),
            )
            await db.commit()

    async def mark_processed(
        self,
        event_id: str,
        processed_by: str,
    ) -> None:
        """Mark an event as processed in the persistent log."""
        now = datetime.now(timezone.utc).isoformat()
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                UPDATE event_log
                   SET status = 'processed',
                       processed_by = ?,
                       processed_at = ?
                 WHERE id = ?
                """,
                (processed_by, now, event_id),
            )
            await db.commit()

    async def _update_event_status(self, event_id: str, status: str) -> None:
        """Update the status of a persisted event (e.g. 'rate_limited')."""
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                "UPDATE event_log SET status = ? WHERE id = ?",
                (status, event_id),
            )
            await db.commit()

    # ------------------------------------------------------------------
    # Cleanup
    # ------------------------------------------------------------------

    async def _cleanup_event_log(self) -> None:
        """Delete old processed/rate_limited events to prevent unbounded growth.

        Keeps pending events (needed for replay) and recent
        processed/rate_limited events (last 7 days, useful for debugging).
        Old rate_limited events beyond 7 days are cleaned up because they
        would have been replayed on startup already.
        """
        cutoff = (datetime.now(timezone.utc) - timedelta(days=7)).isoformat()
        async with aiosqlite.connect(self._db_path) as db:
            cursor = await db.execute(
                """
                DELETE FROM event_log
                 WHERE status IN ('processed', 'rate_limited')
                   AND created_at < ?
                """,
                (cutoff,),
            )
            deleted = cursor.rowcount
            await db.commit()
        if deleted:
            logger.info("Event log cleanup: deleted %d old processed/rate_limited events", deleted)

    # ------------------------------------------------------------------
    # Replay
    # ------------------------------------------------------------------

    async def _replay_pending(self) -> None:
        """Re-deliver unprocessed and rate-limited events from previous runs."""
        # Prune old processed/rate_limited events before replaying
        await self._cleanup_event_log()

        total_replayed = 0
        async with aiosqlite.connect(self._db_path) as db:
            db.row_factory = aiosqlite.Row
            cursor = await db.execute(
                """
                SELECT id, event_type, case_id, payload,
                       source_worker, parent_event_id, created_at
                  FROM event_log
                 WHERE status IN ('pending', 'rate_limited')
                 ORDER BY created_at ASC
                 LIMIT ?
                """,
                (_REPLAY_BATCH_SIZE,),
            )
            rows = await cursor.fetchall()
            for row in rows:
                try:
                    etype = EventType(row["event_type"])
                except ValueError:
                    logger.warning("Unknown event type in replay: %s", row["event_type"])
                    continue

                payload = {}
                if row["payload"]:
                    try:
                        payload = json.loads(row["payload"])
                    except (json.JSONDecodeError, TypeError):
                        payload = {}

                event = NexusEvent(
                    event_type=etype,
                    case_id=row["case_id"],
                    payload=payload,
                    event_id=row["id"],
                    timestamp=row["created_at"] or "",
                    source_worker=row["source_worker"] or "",
                    parent_event_id=row["parent_event_id"],
                )
                await self._fan_out(event)
                total_replayed += 1

        self._events_replayed = total_replayed
        if total_replayed:
            logger.info("Replayed %d pending events", total_replayed)

    # ------------------------------------------------------------------
    # Fan-out
    # ------------------------------------------------------------------

    async def _fan_out(self, event: NexusEvent) -> None:
        """Deliver event to every queue subscribed to its type.

        Uses put_nowait for non-blocking fan-out.  If a subscriber queue
        is full the event is dropped for that subscriber and logged.
        The event is already persisted in SQLite so it can be replayed
        on restart.
        """
        queues = self._subscriptions.get(event.event_type, [])
        for q in queues:
            try:
                q.put_nowait(event)
            except asyncio.QueueFull:
                self._events_dropped += 1
                logger.error(
                    "Event DROPPED (queue full, put_nowait) "
                    "event_type=%s  case_id=%s  event_id=%s  "
                    "queue_size=%d/%d",
                    event.event_type.value,
                    event.case_id,
                    event.event_id,
                    q.qsize(),
                    q.maxsize,
                )

    # ------------------------------------------------------------------
    # Circuit breaker
    # ------------------------------------------------------------------

    def _is_rate_limited(self, event_type: EventType) -> bool:
        """Return True if *event_type* exceeded its rate window."""
        now = time.monotonic()
        window = self._rate_windows[event_type]

        # Prune timestamps older than the window
        cutoff = now - _CIRCUIT_BREAKER_WINDOW
        self._rate_windows[event_type] = [t for t in window if t > cutoff]
        window = self._rate_windows[event_type]

        if len(window) >= _CIRCUIT_BREAKER_MAX:
            return True

        window.append(now)
        return False
