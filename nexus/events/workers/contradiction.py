"""
NEXUS -- ContradictionWorker.

Subscribes to EVIDENCE_PROCESSED.  Runs the ContradictionDetector
to find contradictions between evidence in the case.
Emits CONTRADICTION_FOUND for each contradiction detected.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class ContradictionWorker(ReactiveWorker):
    """Detects contradictions between evidence items."""

    name = "contradiction_detector"
    subscriptions = [EventType.EVIDENCE_PROCESSED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._detector = None

    def _get_detector(self):
        if self._detector is None:
            from nexus.core.contradiction_detector import ContradictionDetector
            self._detector = ContradictionDetector(self._db, self._router)
        return self._detector

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        # Only run when we have enough evidence to compare
        evidence_list = await self._db.list_evidence_by_case(event.case_id)
        if len(evidence_list) < 2:
            logger.debug(
                "ContradictionWorker: only %d evidence items, skipping",
                len(evidence_list),
            )
            return []

        logger.info(
            "ContradictionWorker: detecting contradictions for case %s (%d evidence)",
            event.case_id, len(evidence_list),
        )

        detector = self._get_detector()
        contradictions = await detector.detect_contradictions(event.case_id)

        output: list[NexusEvent] = []
        for c in contradictions:
            output.append(NexusEvent(
                event_type=EventType.CONTRADICTION_FOUND,
                case_id=event.case_id,
                payload={
                    "type": c.get("type", "evidence_contradiction"),
                    "description": c.get("description", ""),
                    "severity": c.get("severity", "medium"),
                    "evidence_1_id": c.get("evidence_1_id", ""),
                    "evidence_2_id": c.get("evidence_2_id", ""),
                    "evidence_1_title": c.get("evidence_1_title", ""),
                    "evidence_2_title": c.get("evidence_2_title", ""),
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            ))

        logger.info(
            "ContradictionWorker: found %d contradictions for case %s",
            len(contradictions), event.case_id,
        )
        return output
