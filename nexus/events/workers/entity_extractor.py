"""
NEXUS -- EntityExtractorWorker.

Subscribes to EVIDENCE_ADDED events.  Entity extraction already
happens inside EvidenceProcessor, so this worker reads the entities
that were created for the evidence and emits ENTITY_DISCOVERED for
each one, enabling downstream workers to react per-entity.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class EntityExtractorWorker(ReactiveWorker):
    """Emits ENTITY_DISCOVERED for each entity linked to new evidence."""

    name = "entity_extractor"
    subscriptions = [EventType.EVIDENCE_ADDED]

    def __init__(self, bus: EventBus, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._processed_evidence: set[str] = set()

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        # Idempotency guard: skip if we already emitted events for this evidence
        if evidence_id in self._processed_evidence:
            logger.debug(
                "Entities already extracted for evidence %s, skipping",
                evidence_id,
            )
            return []

        # Read mentions created by EvidenceProcessor
        mentions = await self._db.list_mentions_by_evidence(evidence_id)
        if not mentions:
            logger.debug(
                "EntityExtractor: no entities found for evidence %s",
                evidence_id,
            )
            return []

        # Collect unique entity IDs from mentions
        seen_entity_ids: set[str] = set()
        output: list[NexusEvent] = []

        for mention in mentions:
            entity_id = mention["entity_id"]
            if entity_id in seen_entity_ids:
                continue
            seen_entity_ids.add(entity_id)

            entity = await self._db.get_entity(entity_id)
            if not entity:
                continue

            output.append(NexusEvent(
                event_type=EventType.ENTITY_DISCOVERED,
                case_id=event.case_id,
                payload={
                    "entity_id": entity["id"],
                    "name": entity.get("name", ""),
                    "entity_type": entity.get("entity_type", "other"),
                    "description": entity.get("description", ""),
                    "evidence_id": evidence_id,
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            ))

        self._processed_evidence.add(evidence_id)

        logger.info(
            "EntityExtractor: emitted %d ENTITY_DISCOVERED for evidence %s",
            len(output), evidence_id,
        )
        return output
