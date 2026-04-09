"""
NEXUS GOV -- Law (dossier legislatif) sync worker.

Downloads the AN Dossiers_Legislatifs.json.zip (8.7MB, 8600+ dossiers)
and creates law records in the gov DB.

Subscription: TICK_DAILY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.scraper import ParliamentScraper


class GovLawSyncWorker(ReactiveWorker):
    """Sync legislative dossiers from AN daily."""

    name = "gov_law_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_law_sync] Starting legislative dossier sync...")

            dossiers = await self._scraper.fetch_dossiers_legislatifs()
            if not dossiers:
                logger.warning("[gov_law_sync] No dossiers fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Build existing UIDs for dedup
            existing_laws = await self._db.list_laws(limit=100_000)
            existing_uids: set[str] = set()
            for law in existing_laws:
                uid = law.get("uid")
                if uid:
                    existing_uids.add(uid)

            new_count = 0

            for dl in dossiers:
                uid = dl.get("uid", "")
                titre = dl.get("titre", "")
                if not uid or not titre:
                    continue

                # Skip existing
                if uid in existing_uids:
                    continue

                source_url = f"https://www.assemblee-nationale.fr/dyn/17/dossiers/{uid}"

                try:
                    await self._db.create_law(
                        title=titre[:500],
                        uid=uid,
                        procedure=dl.get("procedure"),
                        initiator_ref=dl.get("initiateur_ref"),
                        date_initial=dl.get("date"),
                        legislature=dl.get("legislature", "17"),
                        source_url=source_url,
                    )
                    existing_uids.add(uid)
                    new_count += 1
                except Exception as exc:
                    logger.debug("[gov_law_sync] Create law '{}': {}", uid, exc)

                if new_count % 500 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_law_sync] Sync complete: {} dossiers parsed, {} new laws",
                len(dossiers),
                new_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_LAW_ADDED,
                        case_id="gov",
                        payload={
                            "source": "assemblee_nationale",
                            "total": len(dossiers),
                            "new_count": new_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_law_sync] Error during sync: {}", exc)

        return output
