"""
NEXUS -- Neo4jSyncWorker.

Subscribes to ENTITY_DISCOVERED, EVIDENCE_PROCESSED, HYPOTHESIS_SCORED.
Syncs entities, evidence nodes, and hypotheses to the Neo4j graph
database.  Emits ENTITY_ENRICHED after successful graph sync.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class Neo4jSyncWorker(ReactiveWorker):
    """Keeps Neo4j graph in sync with SQLite data."""

    name = "neo4j_sync"
    subscriptions = [
        EventType.ENTITY_DISCOVERED,
        EventType.EVIDENCE_PROCESSED,
        EventType.HYPOTHESIS_SCORED,
    ]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        neo4j: Any,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._neo4j = neo4j

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if not self._neo4j:
            logger.debug("Neo4jSync: no Neo4j client, skipping")
            return []

        # Neo4j sync is idempotent by design: all Cypher queries use MERGE
        # (not CREATE), so re-syncing the same data is a safe no-op.
        # No additional idempotency guard needed.
        if event.event_type == EventType.ENTITY_DISCOVERED:
            return await self._sync_entity(event)
        elif event.event_type == EventType.EVIDENCE_PROCESSED:
            return await self._sync_evidence(event)
        elif event.event_type == EventType.HYPOTHESIS_SCORED:
            return await self._sync_hypothesis(event)

        return []

    async def _sync_entity(self, event: NexusEvent) -> list[NexusEvent]:
        """Sync a discovered entity to Neo4j and link to evidence."""
        entity_id = event.payload.get("entity_id")
        evidence_id = event.payload.get("evidence_id")

        if not entity_id:
            return []

        entity = await self._db.get_entity(entity_id)
        if not entity:
            return []

        await self._neo4j.sync_entity(entity, event.case_id)

        if evidence_id:
            await self._neo4j.link_evidence_to_entity(evidence_id, entity_id)

        logger.info(
            "Neo4jSync: synced entity '%s' (%s) to graph",
            entity.get("name", "?"), entity_id[:8],
        )

        return [NexusEvent(
            event_type=EventType.ENTITY_ENRICHED,
            case_id=event.case_id,
            payload={
                "entity_id": entity_id,
                "name": entity.get("name", ""),
                "entity_type": entity.get("entity_type", ""),
                "enrichment": "neo4j_sync",
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]

    async def _sync_evidence(self, event: NexusEvent) -> list[NexusEvent]:
        """Sync an evidence node to Neo4j."""
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        ev = await self._db.get_evidence(evidence_id)
        if not ev:
            return []

        await self._neo4j.sync_evidence(
            evidence_id=evidence_id,
            case_id=event.case_id,
            title=ev.get("title", ""),
            evidence_type=ev.get("evidence_type", ""),
            reliability=ev.get("reliability", 50),
        )

        logger.info("Neo4jSync: synced evidence %s to graph", evidence_id[:8])
        return []

    async def _sync_hypothesis(self, event: NexusEvent) -> list[NexusEvent]:
        """Sync a scored hypothesis to Neo4j."""
        hypothesis_id = event.payload.get("hypothesis_id")
        if not hypothesis_id:
            return []

        hyp = await self._db.get_hypothesis(hypothesis_id)
        if not hyp:
            return []

        await self._neo4j.sync_hypothesis(
            hypothesis_id,
            event.case_id,
            hyp.get("title", ""),
            hyp.get("current_score", 50.0),
            hyp.get("status", "active"),
        )

        logger.info(
            "Neo4jSync: synced hypothesis %s (score=%.1f) to graph",
            hypothesis_id[:8], hyp.get("current_score", 0),
        )
        return []
