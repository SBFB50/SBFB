"""
NEXUS -- Periodic timer for reports, backups, and summary tree rebuilds.

Replaces the cycle-count-based triggers from the old OODA loop.
Emits TICK_REPORT, TICK_BACKUP, TICK_SUMMARY_TREE events at
configured intervals.
"""

from __future__ import annotations

import asyncio
from datetime import datetime, timezone

from loguru import logger

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent


class PeriodicTimer:
    """Async loop that publishes tick events at fixed intervals.

    Parameters:
        bus:              EventBus to publish on.
        case_id:          Investigation case this timer belongs to.
        report_interval:  Seconds between TICK_REPORT events  (default 6h).
        backup_interval:  Seconds between TICK_BACKUP events  (default 12h).
        summary_interval: Seconds between TICK_SUMMARY_TREE   (default 90min).
    """

    def __init__(
        self,
        bus: EventBus,
        case_id: str,
        *,
        report_interval: float = 6 * 3600,
        backup_interval: float = 12 * 3600,
        summary_interval: float = 90 * 60,
    ) -> None:
        self._bus = bus
        self._case_id = case_id
        self._intervals: dict[EventType, float] = {
            EventType.TICK_REPORT: report_interval,
            EventType.TICK_BACKUP: backup_interval,
            EventType.TICK_SUMMARY_TREE: summary_interval,
            EventType.TICK_WIKI_LINT: 7200.0,  # every 2 hours
        }
        self._running = False
        self._tasks: list[asyncio.Task] = []

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    async def start(self) -> None:
        """Spawn one async loop per tick type."""
        if self._running:
            return
        self._running = True
        for etype, interval in self._intervals.items():
            task = asyncio.create_task(
                self._tick_loop(etype, interval),
                name=f"timer-{etype.value}-{self._case_id[:8]}",
            )
            self._tasks.append(task)
        logger.info(
            "PeriodicTimer started for case {} ({} tick types)",
            self._case_id[:8],
            len(self._intervals),
        )

    async def stop(self) -> None:
        """Cancel all tick loops."""
        self._running = False
        for task in self._tasks:
            if not task.done():
                task.cancel()
        for task in self._tasks:
            try:
                await task
            except (asyncio.CancelledError, Exception):
                pass
        self._tasks.clear()
        logger.info("PeriodicTimer stopped for case {}", self._case_id[:8])

    # ------------------------------------------------------------------
    # Internal loop
    # ------------------------------------------------------------------

    async def _tick_loop(self, event_type: EventType, interval: float) -> None:
        """Sleep *interval* seconds, then publish a tick event. Repeat."""
        try:
            while self._running:
                await asyncio.sleep(interval)
                if not self._running:
                    break
                event = NexusEvent(
                    event_type=event_type,
                    case_id=self._case_id,
                    payload={
                        "tick_at": datetime.now(timezone.utc).isoformat(),
                    },
                    source_worker="PeriodicTimer",
                )
                await self._bus.publish(event)
                logger.debug(
                    "PeriodicTimer: emitted {} for case {}",
                    event_type.value,
                    self._case_id[:8],
                )
        except asyncio.CancelledError:
            pass
        except Exception as exc:
            logger.error(
                "PeriodicTimer loop error for {}: {}",
                event_type.value,
                exc,
            )
