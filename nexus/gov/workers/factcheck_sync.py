"""
NEXUS GOV -- Factcheck Sync Worker.

Fetches fact-checks from Google Fact Check Tools API.
Runs daily. Matches claims to politicians.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType

GOOGLE_FACTCHECK_API = "https://factchecktools.googleapis.com/v1alpha1/claims:search"

# French political search terms for fact-check discovery
FACTCHECK_QUERIES = [
    "assemblee nationale",
    "depute francais",
    "senateur francais",
    "gouvernement france",
    "president republique",
    "premier ministre",
]


class GovFactcheckSyncWorker(ReactiveWorker):
    name = "gov_factcheck_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        import httpx
        from nexus.config import settings

        api_key = getattr(settings, "google_factcheck_api_key", "")
        if not api_key:
            logger.debug("Google Fact Check API key not configured, skipping")
            return []

        output: list[NexusEvent] = []
        politicians = await self._db.list_politicians(limit=100_000)
        name_to_id: dict[str, str] = {p["name"].lower(): p["id"] for p in politicians}

        for query in FACTCHECK_QUERIES:
            try:
                async with httpx.AsyncClient(timeout=15.0) as client:
                    resp = await client.get(
                        GOOGLE_FACTCHECK_API,
                        params={"query": query, "languageCode": "fr", "key": api_key},
                    )
                    if resp.status_code != 200:
                        continue
                    data = resp.json()
            except Exception as exc:
                logger.debug("Factcheck API failed for '{}': {}", query, exc)
                continue

            for claim in data.get("claims", []):
                claim_text = claim.get("text", "")
                claimant = claim.get("claimant", "")
                claim_date = claim.get("claimDate", "")[:10] if claim.get("claimDate") else ""

                reviews = claim.get("claimReview", [])
                if not reviews:
                    continue

                review = reviews[0]
                rating = review.get("textualRating", "")
                review_url = review.get("url", "")
                reviewer = review.get("publisher", {}).get("name", "")

                # Match claimant to politician
                pol_id = name_to_id.get(claimant.lower())
                if not pol_id:
                    # Try partial match on last name
                    for pname, pid in name_to_id.items():
                        if claimant.lower() in pname or pname.split()[-1] in claimant.lower():
                            pol_id = pid
                            break

                try:
                    fc = await self._db.create_factcheck(
                        claim=claim_text,
                        claim_date=claim_date,
                        claimant=claimant,
                        politician_id=pol_id,
                        rating=rating,
                        review_url=review_url,
                        reviewer=reviewer,
                    )
                    output.append(NexusEvent(
                        event_type=GovEventType.GOV_FACTCHECK_ADDED,
                        case_id="gov",
                        payload={
                            "factcheck_id": fc["id"],
                            "claim": claim_text[:100],
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    ))
                except Exception as exc:
                    logger.debug("Factcheck skip: {}", exc)

            await asyncio.sleep(0.5)

        if output:
            logger.info("Factcheck sync: {} new fact-checks", len(output))
        return output
