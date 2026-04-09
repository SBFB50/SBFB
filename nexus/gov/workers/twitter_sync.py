"""
NEXUS GOV -- Twitter/X Sync Worker.

Collects public tweets from French politicians via SearXNG search.
No Twitter API key required — uses web search to find recent tweets.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType

import httpx

SEARXNG_URL = "http://localhost:8888/search"


class GovTwitterSyncWorker(ReactiveWorker):
    name = "gov_twitter_sync"
    subscriptions = [GovEventType.TICK_HOURLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._batch_size = 20  # politicians per tick
        self._offset = 0

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        # Get batch of politicians
        politicians = await self._db.list_politicians(
            limit=self._batch_size, offset=self._offset
        )
        if not politicians:
            self._offset = 0
            return []
        self._offset += self._batch_size

        for pol in politicians:
            name = pol["name"]
            try:
                # Search SearXNG for their tweets
                async with httpx.AsyncClient(timeout=15.0) as client:
                    resp = await client.get(
                        SEARXNG_URL,
                        params={
                            "q": f'"{name}" site:twitter.com OR site:x.com',
                            "format": "json",
                            "categories": "general",
                            "language": "fr",
                        },
                    )
                    if resp.status_code != 200:
                        continue
                    data = resp.json()

                results = data.get("results", [])[:5]
                for r in results:
                    url = r.get("url", "")
                    if "twitter.com" not in url and "x.com" not in url:
                        continue

                    content = r.get("content", r.get("title", ""))
                    if not content:
                        continue

                    try:
                        post = await self._db.create_social_post(
                            politician_id=pol["id"],
                            platform="twitter",
                            content=content[:2000],
                            url=url,
                            media_type="text",
                            posted_at=r.get("publishedDate", ""),
                        )
                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                                case_id="gov",
                                payload={
                                    "post_id": post["id"],
                                    "platform": "twitter",
                                    "politician": name,
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )
                    except Exception:
                        pass  # Duplicate URL

                await asyncio.sleep(2.0)
            except Exception as exc:
                logger.debug("Twitter sync skip '{}': {}", name, exc)

        if output:
            logger.info("Twitter sync: {} new posts", len(output))
        return output
