"""
NEXUS GOV -- EUR-Lex Legislation Sync Worker.

Fetches EU legislation relevant to France from EUR-Lex via the
SPARQL endpoint at publications.europa.eu.

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType

import httpx

EURLEX_SPARQL = "https://publications.europa.eu/webapi/rdf/sparql"


class GovEURlexSyncWorker(ReactiveWorker):
    """Sync recent EU regulations (French texts) from EUR-Lex weekly."""

    name = "gov_eurlex_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        # SPARQL: recent EU regulations with French title, from current year
        query = """
        PREFIX cdm: <http://publications.europa.eu/ontology/cdm#>
        PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
        SELECT DISTINCT ?work ?title ?date ?celex
        WHERE {
          ?work cdm:work_date_document ?date .
          ?work cdm:work_has_resource-type <http://publications.europa.eu/resource/authority/resource-type/REG> .
          ?exp cdm:expression_belongs_to_work ?work .
          ?exp cdm:expression_uses_language <http://publications.europa.eu/resource/authority/language/FRA> .
          ?exp cdm:expression_title ?title .
          OPTIONAL { ?work cdm:resource_legal_id_celex ?celex }
          FILTER(?date >= xsd:date(CONCAT(STR(YEAR(NOW())), "-01-01")))
        }
        ORDER BY DESC(?date)
        LIMIT 50
        """

        try:
            logger.info("[gov_eurlex_sync] Starting EUR-Lex legislation sync...")

            async with httpx.AsyncClient(timeout=60.0) as client:
                resp = await client.get(
                    EURLEX_SPARQL,
                    params={"query": query, "format": "application/json"},
                )
                if resp.status_code != 200:
                    logger.warning("[gov_eurlex_sync] EUR-Lex SPARQL returned {}", resp.status_code)
                    return output
                data = resp.json()

        except Exception as exc:
            logger.warning("[gov_eurlex_sync] EUR-Lex SPARQL failed: {}", exc)
            return output

        bindings = data.get("results", {}).get("bindings", [])

        await asyncio.sleep(0)  # cancellation point

        added = 0

        for b in bindings:
            title = b.get("title", {}).get("value", "")
            date = b.get("date", {}).get("value", "")[:10]
            celex = b.get("celex", {}).get("value", "")

            if not title:
                continue

            uid = f"EURLEX_{celex}" if celex else f"EURLEX_{hash(title) & 0xFFFFFFFF}"

            # Check if already stored
            existing = await self._db.get_law_by_uid(uid)
            if existing:
                continue

            source_url = ""
            if celex:
                source_url = f"https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=CELEX:{celex}"

            try:
                law = await self._db.create_law(
                    title=title[:500],
                    uid=uid,
                    procedure="Reglement EU",
                    status="adopte",
                    date_promulgation=date,
                    legislature="EU",
                    source_url=source_url,
                )
                added += 1
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_LAW_ADDED,
                        case_id="gov",
                        payload={
                            "law_id": law["id"],
                            "title": title[:100],
                            "source": "eurlex",
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )
            except Exception as exc:
                logger.debug("[gov_eurlex_sync] Law create failed: {}", exc)

            if added % 20 == 0:
                await asyncio.sleep(0)  # cancellation point

        logger.info(
            "[gov_eurlex_sync] Sync complete: {} bindings parsed, {} new EU laws added",
            len(bindings),
            added,
        )

        return output
