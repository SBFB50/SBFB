"""
NEXUS GOV -- Senate sync worker.

Fetches senators from the official Senat API and creates/updates
politicians in the gov DB.

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.scraper import ParliamentScraper


class GovSenatSyncWorker(ReactiveWorker):
    """Sync senators from senat.fr API weekly."""

    name = "gov_senat_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_senat_sync] Starting senator sync from senat.fr...")

            senators = await self._scraper.fetch_senators()
            if not senators:
                logger.warning("[gov_senat_sync] No senators fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Load existing politicians for dedup
            existing = await self._db.list_politicians(limit=100_000)
            existing_names = {p["name"].lower(): p for p in existing}

            new_count = 0
            updated_count = 0

            for sen in senators:
                name = sen.get("name", "").strip()
                if not name:
                    continue

                key = name.lower()
                if key not in existing_names:
                    try:
                        created = await self._db.create_politician(
                            name=name,
                            chamber=sen.get("chamber", "senat"),
                            party=sen.get("party"),
                            role=sen.get("role", "senateur"),
                            constituency=sen.get("constituency"),
                            photo_url=sen.get("photo_url"),
                            official_url=sen.get("official_url"),
                        )
                        existing_names[key] = created
                        new_count += 1
                    except Exception as exc:
                        logger.debug("[gov_senat_sync] Create '{}': {}", name, exc)
                else:
                    # Update existing if party or constituency changed
                    existing_pol = existing_names[key]
                    updates = {}
                    if sen.get("party") and sen["party"] != existing_pol.get("party"):
                        updates["party"] = sen["party"]
                    if sen.get("constituency") and sen["constituency"] != existing_pol.get("constituency"):
                        updates["constituency"] = sen["constituency"]
                    if sen.get("photo_url") and not existing_pol.get("photo_url"):
                        updates["photo_url"] = sen["photo_url"]
                    if updates:
                        try:
                            await self._db.update_politician(existing_pol["id"], **updates)
                            updated_count += 1
                        except Exception as exc:
                            logger.debug("[gov_senat_sync] Update '{}': {}", name, exc)

                if (new_count + updated_count) % 100 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_senat_sync] Sync complete: {} senators, {} new, {} updated",
                len(senators),
                new_count,
                updated_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_POLITICIAN_ADDED,
                        case_id="gov",
                        payload={
                            "source": "senat_api",
                            "chamber": "senat",
                            "total": len(senators),
                            "new_count": new_count,
                            "updated_count": updated_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_senat_sync] Error during sync: {}", exc)

        return output
