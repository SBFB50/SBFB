"""
NEXUS -- QueryGeneratorWorker.

Subscribes to HYPOTHESIS_CREATED and ENTITY_ENRICHED.
Generates new monitoring search queries via LLM and creates
monitoring jobs in the database for the MonitoringScheduler
to pick up.  Does not emit events (creates DB jobs instead).
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class QueryGeneratorWorker(ReactiveWorker):
    """Generates monitoring queries from new hypotheses and enriched entities."""

    name = "query_generator"
    subscriptions = [
        EventType.HYPOTHESIS_CREATED,
        EventType.ENTITY_ENRICHED,
        EventType.ENTITY_DISCOVERED,
    ]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if event.event_type == EventType.HYPOTHESIS_CREATED:
            await self._generate_from_hypothesis(event)
        elif event.event_type == EventType.ENTITY_ENRICHED:
            await self._generate_from_entity(event)
        elif event.event_type == EventType.ENTITY_DISCOVERED:
            await self._generate_from_discovered_entity(event)

        # This worker creates DB jobs, does not emit events
        return []

    async def _generate_from_hypothesis(self, event: NexusEvent) -> None:
        """Generate monitoring queries from a new hypothesis."""
        hypothesis_title = event.payload.get("title", "")
        if not hypothesis_title:
            return

        logger.info(
            "QueryGenerator: generating queries from hypothesis '%s'",
            hypothesis_title[:50],
        )

        try:
            from nexus.llm.prompts import QUERY_REFORMULATION_PROMPT
            from nexus.llm.router import TaskType

            # Build context for query generation
            context = f"Nouvelle hypothese: {hypothesis_title}"

            prompt = QUERY_REFORMULATION_PROMPT.format(
                query=hypothesis_title,
                context=context,
            )

            raw = await self._router.route(TaskType.EVIDENCE_SUMMARY, prompt)
            queries = self._parse_queries(raw)

            for query_text in queries:
                await self._create_monitoring_job(
                    event.case_id, query_text, "hypothesis"
                )

            logger.info(
                "QueryGenerator: created %d monitoring jobs from hypothesis",
                len(queries),
            )

        except Exception as exc:
            logger.warning(
                "QueryGenerator: failed to generate queries from hypothesis: %s",
                exc,
            )

    async def _generate_from_entity(self, event: NexusEvent) -> None:
        """Generate monitoring queries from an enriched entity."""
        name = event.payload.get("name", "")
        entity_type = event.payload.get("entity_type", "")

        if not name or entity_type not in ("person", "organization"):
            return

        logger.info(
            "QueryGenerator: generating queries for entity '%s' (%s)",
            name, entity_type,
        )

        queries = [
            f'"{name}" investigation',
            f'"{name}" news',
        ]

        if entity_type == "person":
            queries.append(f'"{name}" casier judiciaire')
            queries.append(f'"{name}" condamnation')

        for query_text in queries:
            try:
                await self._create_monitoring_job(
                    event.case_id, query_text, "entity_enrichment"
                )
            except Exception as exc:
                logger.warning(
                    "QueryGenerator: failed to create job for '%s': %s",
                    query_text, exc,
                )

        logger.info(
            "QueryGenerator: created %d monitoring jobs from entity '%s'",
            len(queries), name,
        )

    async def _generate_from_discovered_entity(self, event: NexusEvent) -> None:
        """Generate cross-entity monitoring queries when a new entity is discovered.

        Combines new entity with existing key entities (victim, locations)
        to generate targeted search queries.
        """
        name = event.payload.get("name", "")
        entity_type = event.payload.get("entity_type", "")

        if not name or entity_type not in ("person", "location"):
            return

        # Get case context for cross-referencing
        try:
            case = await self._db.get_case(event.case_id)
            case_name = case.get("name", "") if case else ""

            # Get existing person entities (for cross-queries with locations)
            persons = await self._db.list_entities_by_case(event.case_id, entity_type="person")
            locations = await self._db.list_entities_by_case(event.case_id, entity_type="location")

            # Avoid creating too many jobs
            existing_jobs = await self._db.list_jobs_by_case(event.case_id)
            existing_queries = {j.get("query", "").lower() for j in existing_jobs}

            queries = []
            if entity_type == "location":
                # Combine location with known persons
                for p in persons[:3]:  # limit to top 3 persons
                    q = f'"{p["name"]}" "{name}"'
                    if q.lower() not in existing_queries:
                        queries.append(q)
            elif entity_type == "person":
                # Combine person with known locations
                for loc in locations[:3]:
                    q = f'"{name}" "{loc["name"]}"'
                    if q.lower() not in existing_queries:
                        queries.append(q)

            for q in queries[:3]:  # max 3 new queries per entity
                await self._create_monitoring_job(event.case_id, q, "entity_cross_query")

            if queries:
                logger.info(
                    "QueryGenerator: created %d cross-entity queries from %s '%s'",
                    min(len(queries), 3), entity_type, name,
                )

        except Exception as exc:
            logger.warning("QueryGenerator: cross-entity generation failed: %s", exc)

    async def _create_monitoring_job(
        self, case_id: str, query: str, trigger: str
    ) -> None:
        """Create a monitoring job in the database."""
        try:
            await self._db.create_monitoring_job(
                case_id=case_id,
                job_type="searxng",
                query=query,
                interval_hours=0,  # immediate re-run
            )
            logger.debug("QueryGenerator: created job '%s' (trigger=%s)", query[:50], trigger)
        except Exception as exc:
            logger.debug("QueryGenerator: job creation failed: %s", exc)

    @staticmethod
    def _parse_queries(raw: str) -> list[str]:
        """Parse LLM output into a list of query strings."""
        queries: list[str] = []
        for line in raw.strip().splitlines():
            line = line.strip()
            # Remove numbering like "1.", "2.", etc.
            if line and len(line) > 3:
                # Strip leading number/bullet
                cleaned = line.lstrip("0123456789.-) ").strip()
                if cleaned and len(cleaned) > 5:
                    queries.append(cleaned)
        return queries[:5]  # Max 5 queries
