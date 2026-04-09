"""
NEXUS GOV -- HATVP declaration sync worker.

Fetches patrimony/interest declarations from HATVP open data CSV and
stores them in the gov DB. Uses IdentityResolver to match declaration
names to existing politicians.

Subscription: TICK_MONTHLY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.identity import IdentityResolver
from nexus.gov.scraper import ParliamentScraper


class GovHATVPSyncWorker(ReactiveWorker):
    """Sync HATVP declarations monthly."""

    name = "gov_hatvp_sync"
    subscriptions = [GovEventType.TICK_MONTHLY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()
        self._resolver = IdentityResolver(db)

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_hatvp_sync] Starting HATVP declarations sync...")

            declarations = await self._scraper.fetch_hatvp()
            if not declarations:
                logger.warning("[gov_hatvp_sync] No HATVP declarations fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Build identity cache for fuzzy matching
            await self._resolver.build_cache()

            # Load existing declarations for dedup (by URL)
            all_pols = await self._db.list_politicians(limit=100_000)
            existing_urls: set[str] = set()
            for pol in all_pols:
                decls = await self._db.list_declarations_by_politician(pol["id"])
                for d in decls:
                    if d.get("url"):
                        existing_urls.add(d["url"])
                await asyncio.sleep(0)  # cancellation point

            new_count = 0
            unresolved_count = 0

            for decl in declarations:
                name = decl.get("name", "").strip()
                if not name:
                    continue

                # Build URL for dedup
                url = ""
                if decl.get("url_dossier"):
                    url = f"https://www.hatvp.fr{decl['url_dossier']}"
                if url and url in existing_urls:
                    continue

                # Resolve politician identity
                match = await self._resolver.resolve(
                    name,
                    source="hatvp",
                    external_id=url or name,
                )

                if not match or match.get("action") == "none":
                    unresolved_count += 1
                    continue

                politician_id = match["politician_id"]

                try:
                    await self._db.create_declaration(
                        politician_id=politician_id,
                        type=decl.get("type_document", "patrimoine"),
                        qualite=decl.get("qualite"),
                        departement=decl.get("departement"),
                        date_publication=decl.get("date_publication"),
                        date_depot=decl.get("date_depot"),
                        url=url,
                        status=decl.get("statut"),
                    )
                    if url:
                        existing_urls.add(url)
                    new_count += 1
                except Exception as exc:
                    logger.debug("[gov_hatvp_sync] Create declaration for '{}': {}", name, exc)

                if new_count % 100 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_hatvp_sync] Sync complete: {} declarations, {} new, {} unresolved",
                len(declarations),
                new_count,
                unresolved_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_DECLARATION_ADDED,
                        case_id="gov",
                        payload={
                            "source": "hatvp",
                            "total": len(declarations),
                            "new_count": new_count,
                            "unresolved_count": unresolved_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_hatvp_sync] Error during sync: {}", exc)

        return output
