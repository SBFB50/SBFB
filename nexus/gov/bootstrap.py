"""
NEXUS GOV -- Government Investigation Bootstrap.

Autonomous module that creates and maintains a continuous investigation
of the French government. On startup, it:

1. Creates a dedicated case "Gouvernement Francais" (idempotent)
2. Seeds politician entities from nosdeputes.fr / nossenateurs.fr
3. Generates SearXNG monitoring jobs for each politician
4. Starts the reactive investigation (20 workers + MonitoringLoop)

The system then runs 24/7 with zero human intervention -- SearXNG
monitors the web, articles are ingested as evidence, entities are
extracted and linked in Neo4j, contradictions are detected by LLM,
and the timeline is built automatically.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.config import settings
from nexus.engine import Database, get_db

GOV_CASE_NAME = "Gouvernement Francais"
GOV_CASE_REF = "GOV-FR-AUTO"

# SearXNG query templates per politician
_QUERY_TEMPLATES = [
    '"{name}" declaration',
    '"{name}" vote assemblee',
    '"{name}" senat OR assemblee',
    '"{name}" polemique OR scandale OR affaire',
]


async def bootstrap_government(inv_manager: Any, neo4j: Any = None, chroma: Any = None) -> tuple[str | None, Any]:
    """Bootstrap the autonomous government investigation.

    Idempotent -- safe to call on every startup. Returns a tuple of
    (case_id, gov_manager) or (None, None) if bootstrap is disabled
    or fails.
    """
    if not getattr(settings, "auto_government_monitoring", True):
        logger.info("Government monitoring disabled (auto_government_monitoring=False)")
        return None, None

    try:
        async with get_db() as conn:
            db = Database(conn)

            # 1. Find or create the government case
            case_id = await _ensure_case(db)
            logger.info("Government case ready: {}", case_id)

            # 2. Seed politician entities
            seeded = await _seed_politicians(db, case_id)
            logger.info("Government entities seeded: {} politicians", seeded)

            # 3. Generate monitoring jobs
            jobs = await _generate_monitoring_jobs(db, case_id)
            logger.info("Government monitoring jobs: {} active", jobs)

        # 4. Start the investigation (outside DB context -- manager uses its own)
        if inv_manager is not None:
            running = await inv_manager.start_investigation(case_id)
            if running:
                logger.info("Government investigation STARTED -- running autonomously")
            else:
                logger.info("Government investigation already running")
        else:
            logger.warning("No investigation manager -- government monitoring passive only")

        # 5. Start the GovManager (10 reactive workers + periodic timer)
        gov_manager = None
        try:
            from nexus.gov.events import GovManager

            gov_manager = GovManager(
                router=getattr(inv_manager, "_router", None) if inv_manager else None,
                neo4j=neo4j,
                chroma=chroma,
            )
            await gov_manager.start()
            logger.info("GovManager started with {} workers", len(gov_manager._workers))
        except Exception as exc:
            logger.warning("GovManager failed to start: {}", exc)
            gov_manager = None

        return case_id, gov_manager

    except Exception as exc:
        logger.exception("Government bootstrap failed: {}", exc)
        return None, None


async def _ensure_case(db: Database) -> str:
    """Find or create the government investigation case."""
    cases = await db.list_cases()
    for c in cases:
        if c.get("reference") == GOV_CASE_REF:
            return c["id"]

    case = await db.create_case(
        name=GOV_CASE_NAME,
        reference=GOV_CASE_REF,
        description=(
            "Investigation autonome et continue du gouvernement francais. "
            "Surveillance des declarations, votes, et activites de tous les "
            "deputes et senateurs via SearXNG. Detection automatique des "
            "contradictions et construction du graphe de relations."
        ),
        status="active",
    )
    logger.info("Created government case: {} ({})", case["id"], GOV_CASE_NAME)
    return case["id"]


async def _seed_politicians(db: Database, case_id: str) -> int:
    """Seed politician entities from PoliGraph API."""
    from nexus.gov.scraper import ParliamentScraper

    existing = await db.list_entities_by_case(case_id)
    existing_names = {e["name"].lower() for e in existing}

    scraper = ParliamentScraper()
    seeded = 0

    try:
        deputies = await scraper.fetch_deputies()
        import asyncio
        await asyncio.sleep(1)
        senators = await scraper.fetch_senators()
        politicians = deputies + senators
    except Exception as exc:
        logger.warning("Failed to fetch politicians: {}", exc)
        return 0

    for pol in politicians:
        name = pol.get("name", "").strip()
        if not name or name.lower() in existing_names:
            continue

        try:
            await db.create_entity(
                case_id=case_id,
                name=name,
                entity_type="person",
                description=f"{pol.get('party', 'SE')} — Elu",
                metadata={
                    "party": pol.get("party"),
                    "party_full": pol.get("party_full"),
                    "party_color": pol.get("party_color"),
                    "photo_url": pol.get("photo_url"),
                    "slug": pol.get("slug"),
                    "source": "government_bootstrap",
                    "api": "poligraph",
                },
            )
            existing_names.add(name.lower())
            seeded += 1
        except Exception as exc:
            logger.debug("Failed to seed entity '{}': {}", name, exc)

    return seeded


async def _generate_monitoring_jobs(db: Database, case_id: str) -> int:
    """Generate SearXNG monitoring jobs for each politician entity."""
    entities = await db.list_entities_by_case(case_id)

    # Only process entities seeded by government bootstrap
    gov_entities = [
        e for e in entities
        if isinstance(e.get("metadata"), dict)
        and e["metadata"].get("source") == "government_bootstrap"
    ]

    # Check existing jobs to avoid duplicates
    existing_jobs = await db.list_jobs_by_case(case_id, active_only=True, limit=100_000)
    existing_queries = {j.get("query", "") for j in existing_jobs}

    created = 0
    for entity in gov_entities:
        name = entity["name"]
        entity_id = entity["id"]

        for template in _QUERY_TEMPLATES:
            query = template.format(name=name)
            if query in existing_queries:
                continue

            try:
                await db.create_monitoring_job(
                    case_id=case_id,
                    job_type="searxng",
                    query=query,
                    entity_id=entity_id,
                    interval_hours=6,
                )
                existing_queries.add(query)
                created += 1
            except Exception as exc:
                logger.debug("Failed to create job for '{}': {}", query[:50], exc)

    return len(existing_queries)
