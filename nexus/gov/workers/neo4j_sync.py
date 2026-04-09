"""
NEXUS GOV -- Neo4j Graph Sync Worker.

Builds and maintains the political influence graph in Neo4j:
  (Politician)-[:VOTED_FOR]->(Law)
  (Politician)-[:VOTED_AGAINST]->(Law)
  (Politician)-[:ABSTAINED]->(Law)
  (Politician)-[:MEMBER_OF]->(Party)
  (Politician)-[:MENTIONED_WITH]->(Politician) [in press/social]
  (Politician)-[:INVOLVED_IN]->(Affair)
  (Politician)-[:SAID {date, source}]->(Position)
  (Politician)-[:DECLARED]->(Declaration) [HATVP]
  (Position)-[:CONTRADICTS]->(Position)
  (Politician)-[:HAS_CONTRADICTION]->(Contradiction)
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


class GovNeo4jSyncWorker(ReactiveWorker):
    name = "gov_neo4j_sync"
    subscriptions = [
        GovEventType.GOV_POSITION_ADDED,
        GovEventType.GOV_AFFAIR_ADDED,
        GovEventType.GOV_PRESS_ADDED,
        GovEventType.GOV_CONTRADICTION_FOUND,
        GovEventType.GOV_POLITICIAN_ADDED,
        GovEventType.GOV_DECLARATION_ADDED,
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
                # Also sync vote relationship if this is a vote-type position
                await self._sync_vote(payload)

            elif etype == GovEventType.GOV_AFFAIR_ADDED:
                await self._sync_affair(payload)

            elif etype == GovEventType.GOV_PRESS_ADDED:
                await self._sync_press_mentions(payload)

            elif etype == GovEventType.GOV_CONTRADICTION_FOUND:
                await self._sync_contradiction(payload)
                # Also create direct position-to-position CONTRADICTS edge
                await self._sync_contradiction_positions(payload)

            elif etype == GovEventType.GOV_DECLARATION_ADDED:
                await self._sync_declarations(payload)

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

    async def _sync_vote(self, payload: dict) -> None:
        """Create voting relationship between politician and law.

        Only applies when position_type is 'vote'. Creates a Law node
        and a VOTED_FOR / VOTED_AGAINST / ABSTAINED edge.
        """
        if payload.get("position_type") != "vote":
            return

        pol_id = payload.get("politician_id", "")
        subject = payload.get("subject", "")
        stance = payload.get("stance", "")
        date = payload.get("date", "")

        if not pol_id or not subject:
            return

        # Create or merge Law node
        await self._neo4j.run_query(
            "MERGE (l:GovLaw {title: $subject}) SET l.updated_at = datetime()",
            {"subject": subject},
        )

        # Determine relationship type from stance
        if stance == "pour":
            rel_type = "VOTED_FOR"
        elif stance == "contre":
            rel_type = "VOTED_AGAINST"
        else:
            rel_type = "ABSTAINED"

        await self._neo4j.run_query(
            f"""MATCH (p:GovPolitician {{gov_id: $pol_id}}), (l:GovLaw {{title: $subject}})
                MERGE (p)-[r:{rel_type}]->(l)
                SET r.date = $date, r.stance = $stance""",
            {"pol_id": pol_id, "subject": subject, "date": date, "stance": stance},
        )

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
        """Create HAS_CONTRADICTION edge between politician and contradiction node."""
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

    async def _sync_contradiction_positions(self, payload: dict) -> None:
        """Create CONTRADICTS edge between two position nodes.

        This provides a direct position-to-position link in addition to the
        politician-level HAS_CONTRADICTION relationship.
        """
        pos_a = payload.get("position_a_id", "")
        pos_b = payload.get("position_b_id", "")
        if not pos_a or not pos_b:
            return

        await self._neo4j.run_query(
            """MATCH (a:GovPosition {gov_id: $pos_a}), (b:GovPosition {gov_id: $pos_b})
               MERGE (a)-[r:CONTRADICTS]->(b)
               SET r.severity = $severity, r.description = $description""",
            {
                "pos_a": pos_a,
                "pos_b": pos_b,
                "severity": payload.get("severity", ""),
                "description": (payload.get("description", "") or "")[:200],
            },
        )

    async def _sync_declaration(self, pol_id: str, declaration: dict) -> None:
        """Create DECLARED relationship between politician and declaration node."""
        dec_id = declaration.get("id", "")
        if not dec_id or not pol_id:
            return

        await self._neo4j.run_query(
            """MERGE (d:GovDeclaration {gov_id: $dec_id})
               SET d.type = $type, d.date = $date, d.url = $url
               WITH d
               MATCH (p:GovPolitician {gov_id: $pol_id})
               MERGE (p)-[:DECLARED]->(d)""",
            {
                "dec_id": dec_id,
                "pol_id": pol_id,
                "type": declaration.get("type", ""),
                "date": declaration.get("date_publication", ""),
                "url": declaration.get("url", ""),
            },
        )

    async def _sync_declarations(self, payload: dict) -> None:
        """Sync declarations when GOV_DECLARATION_ADDED fires.

        The HATVP sync worker emits a summary event with new_count.
        We fetch recent declarations from the DB and create Neo4j edges.
        """
        new_count = payload.get("new_count", 0)
        if new_count <= 0:
            return

        try:
            # Fetch all politicians and their declarations to sync new ones
            politicians = await self._db.list_politicians(limit=10000)
            for pol in politicians:
                declarations = await self._db.list_declarations_by_politician(pol["id"])
                # Only sync the most recent ones (matching new_count as upper bound)
                for decl in declarations[:new_count]:
                    await self._sync_declaration(pol["id"], decl)
        except Exception as exc:
            logger.debug("Neo4j declaration sync error: {}", exc)
