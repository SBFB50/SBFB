"""
NEXUS -- EvidenceIngestWorker.

Subscribes to MONITORING_RESULT events, filters by relevance >= 50,
and calls EvidenceProcessor.process_text_input to ingest the result
as new evidence.  Emits EVIDENCE_ADDED on success.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_RELEVANCE_THRESHOLD = 50


class EvidenceIngestWorker(ReactiveWorker):
    """Ingests high-relevance monitoring results as evidence."""

    name = "evidence_ingest"
    subscriptions = [EventType.MONITORING_RESULT]

    def __init__(
        self,
        bus: EventBus,
        evidence_processor: Any,
    ) -> None:
        super().__init__(bus)
        self._processor = evidence_processor

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        payload = event.payload
        relevance = payload.get("relevance_score", 0)

        if relevance < _RELEVANCE_THRESHOLD:
            logger.debug(
                "EvidenceIngest: skipping result with relevance %s (< %s)",
                relevance, _RELEVANCE_THRESHOLD,
            )
            return []

        title = payload.get("title", "Monitoring result")
        text = payload.get("snippet", "") or payload.get("raw_text", "")
        source = payload.get("url", payload.get("source", "monitoring"))

        if not text.strip():
            logger.debug("EvidenceIngest: empty text, skipping")
            return []

        logger.info(
            "EvidenceIngest: ingesting '%s' (relevance=%s) for case %s",
            title[:60], relevance, event.case_id,
        )

        evidence = await self._processor.process_text_input(
            case_id=event.case_id,
            title=title,
            text=text,
            source=source,
        )

        return [NexusEvent(
            event_type=EventType.EVIDENCE_ADDED,
            case_id=event.case_id,
            payload={
                "evidence_id": evidence.id,
                "title": evidence.title,
                "evidence_type": evidence.evidence_type,
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
