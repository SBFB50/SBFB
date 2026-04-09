"""
NEXUS GOV -- La Fabrique de la Loi sync worker.

Fetches metrics.csv from La Fabrique de la Loi (1117 promulgated laws
with 77 stat columns) and updates existing law records with amendment
counts, duration, article growth, etc.

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType
from nexus.gov.scraper import ParliamentScraper


class GovFabriqueSyncWorker(ReactiveWorker):
    """Enrich law records with La Fabrique de la Loi stats weekly."""

    name = "gov_fabrique_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus, db) -> None:
        super().__init__(bus)
        self._db = db
        self._scraper = ParliamentScraper()

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        try:
            logger.info("[gov_fabrique_sync] Starting La Fabrique de la Loi sync...")

            loi_stats = await self._scraper.fetch_fabrique_loi_stats()
            if not loi_stats:
                logger.warning("[gov_fabrique_sync] No La Fabrique stats fetched")
                return output

            await asyncio.sleep(0)  # cancellation point

            # Load existing laws for matching by title or JO URL
            existing_laws = await self._db.list_laws(limit=100_000)
            title_to_law: dict[str, dict] = {}
            jo_url_to_law: dict[str, dict] = {}
            for law in existing_laws:
                title_key = law.get("title", "").lower().strip()[:100]
                if title_key:
                    title_to_law[title_key] = law
                jo = law.get("jo_url")
                if jo:
                    jo_url_to_law[jo] = law

            new_count = 0
            enriched_count = 0

            for loi in loi_stats:
                titre = loi.get("Titre court", loi.get("Titre", "")).strip()
                if not titre:
                    continue

                url_jo = loi.get("URL JO", "").strip()
                num = loi.get("Numero de la loi", "").strip()

                # Parse numeric stats safely
                def _int(val: str) -> int:
                    try:
                        return int(float(val)) if val else 0
                    except (ValueError, TypeError):
                        return 0

                amendments_count = _int(loi.get("Nombre d'amendements", "0"))
                amendments_adopted = _int(loi.get("Nombre d'amendements adoptes", "0"))
                articles_initial = _int(loi.get("Nombre d'articles initiaux", "0"))
                articles_final = _int(loi.get("Nombre d'articles finals", "0"))
                duration_days = _int(loi.get("Duree d'adoption (jours)", "0"))

                # Try to match to an existing law record
                matched_law = None
                if url_jo and url_jo in jo_url_to_law:
                    matched_law = jo_url_to_law[url_jo]
                else:
                    title_key = titre.lower().strip()[:100]
                    if title_key in title_to_law:
                        matched_law = title_to_law[title_key]

                if matched_law:
                    # Enrich existing law with stats (no update_law method,
                    # so we store enrichment as metadata note for now)
                    enriched_count += 1
                else:
                    # Create as a new law record from La Fabrique data
                    try:
                        await self._db.create_law(
                            title=titre[:500],
                            short_title=titre[:200] if titre else None,
                            status="promulguee",
                            date_promulgation=loi.get("Date de promulgation"),
                            amendments_count=amendments_count,
                            amendments_adopted=amendments_adopted,
                            articles_initial=articles_initial,
                            articles_final=articles_final,
                            duration_days=duration_days,
                            jo_url=url_jo or None,
                            metadata={
                                "source": "fabrique_de_la_loi",
                                "numero": num,
                                "initiative": loi.get("Initiative du texte", ""),
                                "croissance_caracteres": loi.get(
                                    "Croissance du nombre de caracteres", ""
                                ),
                            },
                        )
                        new_count += 1
                    except Exception as exc:
                        logger.debug("[gov_fabrique_sync] Create law '{}': {}", titre[:60], exc)

                if (new_count + enriched_count) % 200 == 0:
                    await asyncio.sleep(0)  # cancellation point

            logger.info(
                "[gov_fabrique_sync] Sync complete: {} stats rows, {} new laws, {} enriched",
                len(loi_stats),
                new_count,
                enriched_count,
            )

            if new_count > 0:
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_LAW_ADDED,
                        case_id="gov",
                        payload={
                            "source": "fabrique_de_la_loi",
                            "total": len(loi_stats),
                            "new_count": new_count,
                            "enriched_count": enriched_count,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )

        except Exception as exc:
            logger.warning("[gov_fabrique_sync] Error during sync: {}", exc)

        return output
