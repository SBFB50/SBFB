"""
NEXUS GOV -- Affairs (judicial/legal) sync worker.

Fetches judicial affairs from PoliGraph API and stores them in the
gov DB. Uses IdentityResolver to match politician names to existing
records.

Subscription: TICK_DAILY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.identity import IdentityResolver
from nexus.gov.scraper import ParliamentScraper


class GovAffairsSyncWorker(ReactiveWorker):
    """Sync judicial affairs from PoliGraph daily."""

    name = "gov_affairs_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()
        self._resolver = IdentityResolver(db)

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_affairs_sync] Starting affairs sync from PoliGraph...")

            affairs = await self._scraper.fetch_poligraph_affairs(max_pages=5)
            if not affairs:
                logger.warning("[gov_affairs_sync] No affairs fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Build identity cache for fuzzy matching
            await self._resolver.build_cache()

            # Build existing affair titles per politician for dedup
            all_pols = await self._db.list_politicians(limit=100_000)
            existing_affairs: dict[str, set[str]] = {}  # pol_id -> {title_lower}
            for pol in all_pols:
                pol_affairs = await self._db.list_affairs_by_politician(pol["id"])
                existing_affairs[pol["id"]] = {
                    a.get("title", "").lower().strip() for a in pol_affairs
                }
                await asyncio.sleep(0)  # cancellation point

            new_count = 0
            unresolved_count = 0

            for affair in affairs:
                # PoliGraph affair structure
                title = affair.get("title", affair.get("nom", "")).strip()
                if not title:
                    continue

                # Resolve politician
                politician_name = affair.get("politician", affair.get("politicien", ""))
                if isinstance(politician_name, dict):
                    politician_name = politician_name.get("nom", "")
                politician_name = str(politician_name).strip()

                if not politician_name:
                    unresolved_count += 1
                    continue

                # Extract an external ID from the affair data
                affair_id = str(
                    affair.get("id", affair.get("_id", ""))
                )

                match = await self._resolver.resolve(
                    politician_name,
                    source="poligraph",
                    external_id=affair_id or politician_name,
                )

                if not match or match.get("action") == "none":
                    unresolved_count += 1
                    continue

                politician_id = match["politician_id"]

                # Dedup by title
                title_lower = title.lower().strip()
                if politician_id in existing_affairs:
                    if title_lower in existing_affairs[politician_id]:
                        continue

                # Extract affair details
                description = affair.get("description", affair.get("resume", ""))
                status = affair.get("status", affair.get("statut", "enquete"))
                category = affair.get("category", affair.get("categorie", ""))
                source_url = affair.get("url", affair.get("source_url", ""))
                date_start = affair.get("date_start", affair.get("date", ""))

                try:
                    await self._db.create_affair(
                        politician_id=politician_id,
                        title=title[:500],
                        description=str(description)[:2000] if description else None,
                        status=str(status) if status else "enquete",
                        category=str(category) if category else None,
                        source_url=str(source_url) if source_url else None,
                        date_start=str(date_start)[:10] if date_start else None,
                    )
                    existing_affairs.setdefault(politician_id, set()).add(title_lower)
                    new_count += 1
                except Exception as exc:
                    logger.debug(
                        "[gov_affairs_sync] Create affair '{}' for '{}': {}",
                        title[:40],
                        politician_name,
                        exc,
                    )

                if new_count % 50 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_affairs_sync] Sync complete: {} affairs, {} new, {} unresolved",
                len(affairs),
                new_count,
                unresolved_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_AFFAIR_ADDED,
                        case_id="gov",
                        payload={
                            "source": "poligraph",
                            "total": len(affairs),
                            "new_count": new_count,
                            "unresolved_count": unresolved_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_affairs_sync] Error during sync: {}", exc)

        return output
