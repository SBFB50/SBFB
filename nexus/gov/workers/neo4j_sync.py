"""
NEXUS GOV -- Neo4j Graph Sync Worker.

Builds and maintains the political influence graph in Neo4j:
  (Politician)-[:VOTED_FOR]->(Law)
  (Politician)-[:MEMBER_OF]->(Party)
  (Politician)-[:MENTIONED_WITH]->(Politician) [in press/social]
  (Politician)-[:INVOLVED_IN]->(Affair)
  (Politician)-[:SAID {date, source}]->(Position)
  (Position)-[:CONTRADICTS]->(Position)
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType


class GovNeo4jSyncWorker(ReactiveWorker):
    name = "gov_neo4j_sync"
    subscriptions = [
        GovEventType.GOV_POSITION_ADDED,
        GovEventType.GOV_AFFAIR_ADDED,
        GovEventType.GOV_PRESS_ADDED,
        GovEventType.GOV_CONTRADICTION_FOUND,
        GovEventType.GOV_POLITICIAN_ADDED,
    ]

    def __init__(self, bus: Any, db: Any, neo4j: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._neo4j = neo4j

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if self._neo4j is None:
            return []  # Neo4j not available, skip silently

        etype = event.event_type
        payload = event.payload

        try:
            if etype == GovEventType.GOV_POLITICIAN_ADDED:
                await self._sync_politician(payload)

            elif etype == GovEventType.GOV_POSITION_ADDED:
                await self._sync_position(payload)

            elif etype == GovEventType.GOV_AFFAIR_ADDED:
                await self._sync_affair(payload)

            elif etype == GovEventType.GOV_PRESS_ADDED:
                await self._sync_press_mentions(payload)

            elif etype == GovEventType.GOV_CONTRADICTION_FOUND:
                await self._sync_contradiction(payload)

        except Exception as exc:
            logger.debug("Neo4j gov sync error: {}", exc)

        return []  # Neo4j sync is a leaf node, no downstream events

    async def _sync_politician(self, payload: dict) -> None:
        """Create/update politician node."""
        pol_id = payload.get("politician_id", "")
        name = payload.get("name", "")
        party = payload.get("party", "")

        query = """
        MERGE (p:GovPolitician {gov_id: $id})
        SET p.name = $name, p.party = $party, p.updated = datetime()
        WITH p
        MERGE (party:GovParty {name: $party})
        MERGE (p)-[:MEMBER_OF]->(party)
        """
        await self._neo4j.run_query(query, {"id": pol_id, "name": name, "party": party})

    async def _sync_position(self, payload: dict) -> None:
        """Create position node and link to politician."""
        pol_id = payload.get("politician_id", "")
        position_id = payload.get("position_id", "")
        subject = payload.get("subject", "")
        stance = payload.get("stance", "")

        query = """
        MATCH (p:GovPolitician {gov_id: $pol_id})
        MERGE (pos:GovPosition {gov_id: $pos_id})
        SET pos.subject = $subject, pos.stance = $stance
        MERGE (p)-[:SAID]->(pos)
        WITH pos
        MERGE (subj:GovSubject {name: $subject})
        MERGE (pos)-[:ABOUT]->(subj)
        """
        await self._neo4j.run_query(query, {
            "pol_id": pol_id, "pos_id": position_id,
            "subject": subject, "stance": stance,
        })

    async def _sync_affair(self, payload: dict) -> None:
        """Create affair node and link to politician."""
        pol_id = payload.get("politician_id", "")
        affair_id = payload.get("affair_id", "")
        title = payload.get("title", "")

        query = """
        MATCH (p:GovPolitician {gov_id: $pol_id})
        MERGE (a:GovAffair {gov_id: $affair_id})
        SET a.title = $title
        MERGE (p)-[:INVOLVED_IN]->(a)
        """
        await self._neo4j.run_query(query, {
            "pol_id": pol_id, "affair_id": affair_id, "title": title,
        })

    async def _sync_press_mentions(self, payload: dict) -> None:
        """Create co-mention edges between politicians mentioned in same article."""
        politicians = payload.get("politicians", [])
        if len(politicians) < 2:
            return

        # Create MENTIONED_WITH edges between all pairs
        for i in range(len(politicians)):
            for j in range(i + 1, len(politicians)):
                query = """
                MATCH (a:GovPolitician {gov_id: $id_a})
                MATCH (b:GovPolitician {gov_id: $id_b})
                MERGE (a)-[r:MENTIONED_WITH]-(b)
                SET r.count = COALESCE(r.count, 0) + 1, r.updated = datetime()
                """
                await self._neo4j.run_query(query, {
                    "id_a": politicians[i], "id_b": politicians[j],
                })

    async def _sync_contradiction(self, payload: dict) -> None:
        """Create CONTRADICTS edge between positions."""
        contradiction_id = payload.get("contradiction_id", "")
        pol_id = payload.get("politician_id", "")
        description = payload.get("description", "")

        query = """
        MATCH (p:GovPolitician {gov_id: $pol_id})
        MERGE (c:GovContradiction {gov_id: $contra_id})
        SET c.description = $desc
        MERGE (p)-[:HAS_CONTRADICTION]->(c)
        """
        await self._neo4j.run_query(query, {
            "pol_id": pol_id, "contra_id": contradiction_id, "desc": description[:500],
        })
