"""
NEXUS GOV -- EU Parliament Sync Worker.

Fetches French MEPs from the European Parliament open data API.
https://data.europarl.europa.eu/api/v2/meps/show-current

Subscription: TICK_WEEKLY
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

import httpx

EU_PARL_API = "https://data.europarl.europa.eu/api/v2/meps/show-current"

_MAX_RETRIES = 3
_RETRY_BACKOFF = 2.0  # seconds, doubled each retry


class GovEUParliamentSyncWorker(ReactiveWorker):
    """Sync French MEPs from the European Parliament open data API weekly."""

    name = "gov_eu_parliament_sync"
    subscriptions = [GovEventType.TICK_WEEKLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    # ------------------------------------------------------------------
    # HTTP helper with retry + exponential backoff
    # ------------------------------------------------------------------

    async def _fetch_with_retry(
        self,
        url: str,
        params: dict | None = None,
        headers: dict | None = None,
        *,
        max_retries: int = _MAX_RETRIES,
        timeout: float = 30.0,
    ) -> dict | None:
        """Fetch JSON with retry and exponential backoff."""
        for attempt in range(max_retries):
            try:
                async with httpx.AsyncClient(timeout=timeout) as client:
                    resp = await client.get(url, params=params, headers=headers)
                    resp.raise_for_status()
                    return resp.json()
            except httpx.HTTPStatusError as exc:
                # Don't retry 4xx client errors (except 429 Too Many Requests)
                if exc.response.status_code < 500 and exc.response.status_code != 429:
                    logger.warning(
                        "[{}] HTTP {}: {}", self.name, exc.response.status_code, url[:80],
                    )
                    return None
                if attempt < max_retries - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.warning(
                        "[{}] Fetch attempt {}/{} failed (HTTP {}), retrying in {:.0f}s",
                        self.name, attempt + 1, max_retries, exc.response.status_code, wait,
                    )
                    await asyncio.sleep(wait)
                else:
                    logger.error(
                        "[{}] Fetch failed after {} attempts: {}", self.name, max_retries, exc,
                    )
                    return None
            except Exception as exc:
                if attempt < max_retries - 1:
                    wait = _RETRY_BACKOFF * (2 ** attempt)
                    logger.warning(
                        "[{}] Fetch attempt {}/{} failed: {}, retrying in {:.0f}s",
                        self.name, attempt + 1, max_retries, exc, wait,
                    )
                    await asyncio.sleep(wait)
                else:
                    logger.error(
                        "[{}] Fetch failed after {} attempts: {}", self.name, max_retries, exc,
                    )
                    return None
        return None

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        logger.info("[gov_eu_parliament_sync] Starting French MEP sync from EU Parliament API...")

        data = await self._fetch_with_retry(
            EU_PARL_API, headers={"Accept": "application/json"},
        )
        if data is None:
            logger.warning("[gov_eu_parliament_sync] EU Parliament API unavailable after retries")
            return output

        # Parse MEPs -- API structure varies, handle multiple formats
        meps = data.get("results", data.get("data", []))
        if isinstance(data, dict) and "meps" in data:
            meps = data["meps"]

        await asyncio.sleep(0)  # cancellation point

        # Load existing politicians for dedup
        existing = await self._db.list_politicians(limit=100_000)
        existing_names = {p["name"].lower() for p in existing}

        added = 0

        for mep in meps:
            name = ""
            country = ""

            if isinstance(mep, dict):
                name = mep.get("fullName", mep.get("name", ""))
                country = mep.get("country", mep.get("nationality", ""))
                if not name:
                    first = mep.get("firstName", mep.get("givenName", ""))
                    last = mep.get("lastName", mep.get("familyName", ""))
                    name = f"{first} {last}".strip()

            if not name:
                continue

            # Filter: French MEPs only
            country_lower = (country or "").lower()
            if country_lower and "fran" not in country_lower and "fr" != country_lower:
                continue

            if name.lower() in existing_names:
                continue

            try:
                mep_id = mep.get("id", "")
                photo_url = mep.get("photoUrl", "")
                if not photo_url and mep_id:
                    photo_url = f"https://www.europarl.europa.eu/mepphoto/{mep_id}.jpg"
                official_url = ""
                if mep_id:
                    official_url = f"https://www.europarl.europa.eu/meps/fr/{mep_id}"

                created = await self._db.create_politician(
                    name=name,
                    chamber="europe",
                    party=mep.get("politicalGroup", mep.get("group", "")),
                    role="eurodeput",
                    photo_url=photo_url,
                    official_url=official_url,
                )
                existing_names.add(name.lower())
                added += 1
            except Exception as exc:
                logger.debug("[gov_eu_parliament_sync] Create '{}': {}", name, exc)

            if added % 50 == 0:
                await asyncio.sleep(0)  # cancellation point

        logger.info(
            "[gov_eu_parliament_sync] Sync complete: {} MEPs parsed, {} French MEPs added",
            len(meps),
            added,
        )

        if added > 0:
            output.append(
                NexusEvent(
                    event_type=GovEventType.GOV_POLITICIAN_ADDED,
                    case_id="gov",
                    payload={
                        "source": "eu_parliament_api",
                        "chamber": "europe",
                        "new_count": added,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            )

        return output
