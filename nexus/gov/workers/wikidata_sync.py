"""
NEXUS GOV -- Wikidata enrichment sync worker.

Fetches deputies and senators from Wikidata SPARQL endpoint and enriches
existing politician records with photos, birth dates, and other
biographical data. Uses IdentityResolver for name matching.

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.identity import IdentityResolver
from nexus.gov.scraper import ParliamentScraper


class GovWikidataSyncWorker(ReactiveWorker):
    """Enrich politicians with Wikidata biographical data weekly."""

    name = "gov_wikidata_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()
        self._resolver = IdentityResolver(db)

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_wikidata_sync] Starting Wikidata enrichment...")

            # Build identity cache for fuzzy matching
            await self._resolver.build_cache()

            # Fetch deputies from Wikidata
            wiki_deputies = await self._scraper.fetch_wikidata_deputies()
            await asyncio.sleep(0)  # cancellation point / rate limit

            # Fetch senators from Wikidata
            wiki_senators = await self._scraper.fetch_wikidata_senators()
            await asyncio.sleep(0)  # cancellation point

            all_wiki = wiki_deputies + wiki_senators
            if not all_wiki:
                logger.warning("[gov_wikidata_sync] No Wikidata results")
                return output

            enriched_count = 0
            unresolved_count = 0

            for wp in all_wiki:
                name = wp.get("personLabel", "").strip()
                if not name:
                    continue

                # Extract Wikidata entity ID from URI
                person_uri = wp.get("person", "")
                wikidata_id = person_uri.rsplit("/", 1)[-1] if person_uri else ""

                # Resolve to existing politician
                match = await self._resolver.resolve(
                    name,
                    source="wikidata",
                    external_id=wikidata_id or name,
                )

                if not match or match.get("action") == "none":
                    unresolved_count += 1
                    continue

                politician_id = match["politician_id"]

                # Build update fields from Wikidata
                updates: dict = {}
                image = wp.get("image", "")
                if image:
                    updates["photo_url"] = image

                birth_date = wp.get("birthDate", "")
                birth_place = wp.get("birthPlaceLabel", "")
                party_label = wp.get("partyLabel", "")

                # Store biographical info as metadata enrichment
                meta_enrichment = {}
                if birth_date:
                    meta_enrichment["birth_date"] = birth_date[:10]
                if birth_place:
                    meta_enrichment["birth_place"] = birth_place
                if party_label:
                    meta_enrichment["wikidata_party"] = party_label
                if wikidata_id:
                    meta_enrichment["wikidata_id"] = wikidata_id

                if meta_enrichment:
                    updates["metadata"] = meta_enrichment

                if updates:
                    try:
                        await self._db.update_politician(politician_id, **updates)
                        enriched_count += 1
                    except Exception as exc:
                        logger.debug(
                            "[gov_wikidata_sync] Update '{}': {}", name, exc
                        )

                if enriched_count % 100 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_wikidata_sync] Enrichment complete: {} Wikidata entries, "
                "{} enriched, {} unresolved",
                len(all_wiki),
                enriched_count,
                unresolved_count,
            )

            if enriched_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_POLITICIAN_ADDED,
                        case_id="gov",
                        payload={
                            "source": "wikidata",
                            "wikidata_deputies": len(wiki_deputies),
                            "wikidata_senators": len(wiki_senators),
                            "enriched_count": enriched_count,
                            "unresolved_count": unresolved_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_wikidata_sync] Error during enrichment: {}", exc)

        return output
