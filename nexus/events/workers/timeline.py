"""
NEXUS -- TimelineWorker.

Subscribes to EVIDENCE_PROCESSED.  Rebuilds the case timeline
whenever new evidence is processed, aggregating dates from all
data sources.  Emits TIMELINE_REBUILT.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class TimelineWorker(ReactiveWorker):
    """Rebuilds the case timeline when evidence changes."""

    name = "timeline_builder"
    subscriptions = [EventType.EVIDENCE_PROCESSED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        neo4j: Any = None,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._neo4j = neo4j
        self._builder = None

    def _get_builder(self):
        if self._builder is None:
            from nexus.core.timeline_builder import TimelineBuilder
            self._builder = TimelineBuilder(self._db, self._neo4j)
        return self._builder

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        logger.info(
            "TimelineWorker: rebuilding timeline for case %s",
            event.case_id,
        )

        builder = self._get_builder()

        try:
            entries = await builder.build_timeline(event.case_id)

            logger.info(
                "TimelineWorker: rebuilt timeline with %d entries for case %s",
                len(entries), event.case_id,
            )

            return [NexusEvent(
                event_type=EventType.TIMELINE_REBUILT,
                case_id=event.case_id,
                payload={
                    "entry_count": len(entries),
                    "evidence_id": event.payload.get("evidence_id", ""),
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            )]

        except Exception as exc:
            logger.warning(
                "TimelineWorker: rebuild failed for case %s: %s",
                event.case_id, exc,
            )
            return []
