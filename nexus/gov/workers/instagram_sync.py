"""
NEXUS GOV -- Instagram Sync Worker.

Collects public Instagram posts from French politicians via SearXNG search.
No Instagram API key required — uses web search to find recent posts.
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


class GovInstagramSyncWorker(ReactiveWorker):
    name = "gov_instagram_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._batch_size = 30
        self._offset = 0

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        politicians = await self._db.list_politicians(
            limit=self._batch_size, offset=self._offset
        )
        if not politicians:
            self._offset = 0
            return []
        self._offset += self._batch_size

        for pol in politicians:
            try:
                async with httpx.AsyncClient(timeout=15.0) as client:
                    resp = await client.get(
                        SEARXNG_URL,
                        params={
                            "q": f'"{pol["name"]}" site:instagram.com',
                            "format": "json",
                            "language": "fr",
                        },
                    )
                    if resp.status_code != 200:
                        continue
                    data = resp.json()

                for r in data.get("results", [])[:3]:
                    url = r.get("url", "")
                    if "instagram.com" not in url:
                        continue
                    content = r.get("content", r.get("title", ""))
                    if not content:
                        continue
                    media_url = r.get("img_src", "")
                    try:
                        post = await self._db.create_social_post(
                            politician_id=pol["id"],
                            platform="instagram",
                            content=content[:2000],
                            url=url,
                            media_type="image" if media_url else "text",
                            media_url=media_url,
                            posted_at=r.get("publishedDate", ""),
                        )
                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                                case_id="gov",
                                payload={
                                    "post_id": post["id"],
                                    "platform": "instagram",
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )
                    except Exception:
                        pass  # Duplicate URL

                await asyncio.sleep(2.0)
            except Exception as exc:
                logger.debug("IG sync skip '{}': {}", pol["name"], exc)

        if output:
            logger.info("Instagram sync: {} new posts", len(output))
        return output
