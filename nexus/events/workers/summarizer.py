"""
NEXUS -- SummarizerWorker.

Subscribes to EVIDENCE_ADDED.  The summary is already generated
inside EvidenceProcessor, so this worker simply verifies the summary
exists and emits EVIDENCE_PROCESSED to signal downstream workers
(chunker, contradiction detector, etc.) that the evidence is ready.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class SummarizerWorker(ReactiveWorker):
    """Confirms evidence is summarised and emits EVIDENCE_PROCESSED."""

    name = "summarizer"
    subscriptions = [EventType.EVIDENCE_ADDED]

    def __init__(self, bus: EventBus, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        evidence = await self._db.get_evidence(evidence_id)
        if not evidence:
            logger.warning("Summarizer: evidence %s not found", evidence_id)
            return []

        status = evidence.get("status", "")
        summary = evidence.get("summary", "")

        if status != "processed":
            logger.debug(
                "Summarizer: evidence %s status=%s (not yet processed), skipping",
                evidence_id, status,
            )
            return []

        logger.info(
            "Summarizer: evidence %s processed (summary=%d chars)",
            evidence_id, len(summary),
        )

        return [NexusEvent(
            event_type=EventType.EVIDENCE_PROCESSED,
            case_id=event.case_id,
            payload={
                "evidence_id": evidence_id,
                "title": evidence.get("title", ""),
                "has_summary": bool(summary),
                "evidence_type": evidence.get("evidence_type", ""),
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
