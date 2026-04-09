"""
NEXUS GOV -- Press Sync Worker.

Monitors French political news from RSS feeds and SearXNG.
Runs hourly. Extracts politician mentions using simple name matching.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

# French political news RSS feeds
RSS_FEEDS = [
    ("Le Monde", "https://www.lemonde.fr/politique/rss_full.xml"),
    ("Le Figaro", "https://www.lefigaro.fr/rss/figaro_politique.xml"),
    ("Franceinfo", "https://www.francetvinfo.fr/politique.rss"),
    ("Liberation", "https://www.liberation.fr/arc/outboundfeeds/rss/category/politique/"),
    ("Public Senat", "https://www.publicsenat.fr/rss"),
    ("LCP", "https://lcp.fr/rss"),
    ("Politico", "https://www.politico.eu/feed/"),
]


class GovPressSyncWorker(ReactiveWorker):
    name = "gov_press_sync"
    subscriptions = [GovEventType.TICK_HOURLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        # Fetch RSS feeds
        try:
            import feedparser
        except ImportError:
            logger.warning("feedparser not installed, skipping RSS sync")
            return []

        import httpx

        # Load politician names for mention detection
        politicians = await self._db.list_politicians(limit=100_000)
        name_to_id: dict[str, str] = {p["name"].lower(): p["id"] for p in politicians}

        for source_name, feed_url in RSS_FEEDS:
            try:
                async with httpx.AsyncClient(timeout=15.0) as client:
                    resp = await client.get(feed_url)
                    if resp.status_code != 200:
                        continue
                    feed = feedparser.parse(resp.text)
            except Exception as exc:
                logger.debug("RSS fetch failed {}: {}", source_name, exc)
                continue

            for entry in feed.entries[:20]:  # Last 20 articles per feed
                title = entry.get("title", "")
                url = entry.get("link", "")
                if not url:
                    continue

                summary = entry.get("summary", entry.get("description", ""))[:500]
                published = entry.get("published", "")

                # Detect politician mentions in title + summary
                text_lower = (title + " " + summary).lower()
                mentioned_ids: list[str] = []
                for pname, pid in name_to_id.items():
                    # Match on last name (more reliable than full name in headlines)
                    parts = pname.split()
                    if len(parts) >= 2 and parts[-1] in text_lower:
                        mentioned_ids.append(pid)

                if not mentioned_ids:
                    continue  # Skip articles that don't mention any politician

                # Store as comma-separated string (DB schema expects str)
                mentioned_str = ",".join(mentioned_ids)

                try:
                    article = await self._db.create_press_article(
                        title=title,
                        url=url,
                        source_name=source_name,
                        published_at=published,
                        summary=summary[:500],
                        politicians_mentioned=mentioned_str,
                    )
                    output.append(NexusEvent(
                        event_type=GovEventType.GOV_PRESS_ADDED,
                        case_id="gov",
                        payload={
                            "article_id": article["id"],
                            "title": title,
                            "politicians": mentioned_ids,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    ))
                except Exception as exc:
                    logger.debug("Press article skip (likely duplicate): {}", exc)

            await asyncio.sleep(1.0)  # Rate limit between feeds

        if output:
            logger.info("Press sync: {} new articles ingested", len(output))
        return output
