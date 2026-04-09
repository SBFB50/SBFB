"""
NEXUS GOV -- Deputy sync worker.

Fetches active deputies from data.gouv.fr CSV (Datan dataset) and
creates/updates politicians in the gov DB.

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.scraper import ParliamentScraper


class GovDeputeSyncWorker(ReactiveWorker):
    """Sync deputies from data.gouv.fr weekly."""

    name = "gov_depute_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_depute_sync] Starting deputy sync from data.gouv.fr...")

            deputies = await self._scraper.fetch_deputies()
            if not deputies:
                logger.warning("[gov_depute_sync] No deputies fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Load existing politicians for dedup
            existing = await self._db.list_politicians(limit=100_000)
            existing_names = {p["name"].lower(): p for p in existing}

            new_count = 0
            updated_count = 0

            for dep in deputies:
                name = dep.get("name", "").strip()
                if not name:
                    continue

                key = name.lower()
                if key not in existing_names:
                    # Create new politician
                    try:
                        created = await self._db.create_politician(
                            name=name,
                            chamber=dep.get("chamber", "assemblee"),
                            party=dep.get("party"),
                            role=dep.get("role", "depute"),
                            constituency=dep.get("constituency"),
                            photo_url=dep.get("photo_url"),
                            official_url=dep.get("official_url"),
                        )
                        existing_names[key] = created
                        new_count += 1
                    except Exception as exc:
                        logger.debug("[gov_depute_sync] Create '{}': {}", name, exc)
                else:
                    # Update existing if party or constituency changed
                    existing_pol = existing_names[key]
                    updates = {}
                    if dep.get("party") and dep["party"] != existing_pol.get("party"):
                        updates["party"] = dep["party"]
                    if dep.get("constituency") and dep["constituency"] != existing_pol.get("constituency"):
                        updates["constituency"] = dep["constituency"]
                    if dep.get("photo_url") and not existing_pol.get("photo_url"):
                        updates["photo_url"] = dep["photo_url"]
                    if updates:
                        try:
                            await self._db.update_politician(existing_pol["id"], **updates)
                            updated_count += 1
                        except Exception as exc:
                            logger.debug("[gov_depute_sync] Update '{}': {}", name, exc)

                if (new_count + updated_count) % 100 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_depute_sync] Sync complete: {} deputies, {} new, {} updated",
                len(deputies),
                new_count,
                updated_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_POLITICIAN_ADDED,
                        case_id="gov",
                        payload={
                            "source": "data_gouv_fr",
                            "chamber": "assemblee",
                            "total": len(deputies),
                            "new_count": new_count,
                            "updated_count": updated_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_depute_sync] Error during sync: {}", exc)

        return output
