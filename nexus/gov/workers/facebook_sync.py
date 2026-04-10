"""
NEXUS GOV -- Facebook Sync Worker.

Collects public Facebook posts from French politicians via SearXNG search.
No Facebook API key required — uses web search to find recent posts.
"""

from __future__ import annotations

import asyncio
import re
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

import httpx

SEARXNG_URL = "http://localhost:8888/search"

# Patterns that indicate a Facebook content URL (posts, stories, photos, videos, watch)
_FB_URL_RE = re.compile(r"https?://(www\.|m\.|web\.)?facebook\.com/")


def _extract_engagement(result: dict) -> dict[str, Any]:
    """Try to extract engagement metrics from SearXNG result metadata.

    SearXNG may include engagement hints in the content snippet or
    metadata fields depending on the engine (Google, Bing, etc.).
    """
    engagement: dict[str, Any] = {}

    content = result.get("content", "")
    title = result.get("title", "")
    combined = f"{title} {content}"

    # Try to parse likes/reactions count from snippet text
    likes_match = re.search(r"(\d[\d\s,.]*)\s*(?:like|j'aime|réaction|reaction)", combined, re.IGNORECASE)
    if likes_match:
        raw = likes_match.group(1).replace(" ", "").replace(",", "").replace(".", "")
        try:
            engagement["likes"] = int(raw)
        except ValueError:
            pass

    comments_match = re.search(r"(\d[\d\s,.]*)\s*(?:comment|commentaire)", combined, re.IGNORECASE)
    if comments_match:
        raw = comments_match.group(1).replace(" ", "").replace(",", "").replace(".", "")
        try:
            engagement["comments"] = int(raw)
        except ValueError:
            pass

    shares_match = re.search(r"(\d[\d\s,.]*)\s*(?:share|partage)", combined, re.IGNORECASE)
    if shares_match:
        raw = shares_match.group(1).replace(" ", "").replace(",", "").replace(".", "")
        try:
            engagement["shares"] = int(raw)
        except ValueError:
            pass

    # SearXNG sometimes provides these as top-level keys
    for key in ("score", "views", "likes"):
        if key in result and result[key]:
            engagement[key] = result[key]

    return engagement


def _build_content(result: dict) -> str:
    """Build rich content string from all available SearXNG fields."""
    parts: list[str] = []

    title = (result.get("title") or "").strip()
    if title:
        parts.append(title)

    content = (result.get("content") or "").strip()
    if content and content != title:
        parts.append(content)

    # Some engines return a longer 'description' or 'long_description'
    for field in ("description", "long_description"):
        extra = (result.get(field) or "").strip()
        if extra and extra not in parts:
            parts.append(extra)

    return "\n".join(parts)[:2000] if parts else ""


def _detect_media_type(result: dict) -> str:
    """Infer media type from URL and result metadata."""
    url = result.get("url", "")
    if "/videos/" in url or "/watch" in url or "/reel" in url:
        return "video"
    if "/photos/" in url or "/photo" in url:
        return "image"
    if result.get("img_src"):
        return "image"
    return "text"


class GovFacebookSyncWorker(ReactiveWorker):
    name = "gov_facebook_sync"
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
                            "q": f'"{pol["name"]}" site:facebook.com',
                            "format": "json",
                            "language": "fr",
                        },
                    )
                    if resp.status_code != 200:
                        continue
                    data = resp.json()

                for r in data.get("results", [])[:10]:
                    url = r.get("url", "")
                    # Accept any facebook.com/* URL (posts, pages, videos, photos)
                    if not _FB_URL_RE.match(url):
                        continue

                    content = _build_content(r)
                    if not content:
                        continue

                    media_type = _detect_media_type(r)
                    engagement = _extract_engagement(r)

                    try:
                        post = await self._db.create_social_post(
                            politician_id=pol["id"],
                            platform="facebook",
                            content=content,
                            url=url,
                            media_type=media_type,
                            posted_at=r.get("publishedDate", ""),
                        )
                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                                case_id="gov",
                                payload={
                                    "post_id": post["id"],
                                    "platform": "facebook",
                                    "politician_id": pol["id"],
                                    "politician_name": pol["name"],
                                    "media_type": media_type,
                                    "engagement": engagement,
                                    "url": url,
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )
                    except Exception:
                        pass  # Duplicate URL

                await asyncio.sleep(2.0)
            except Exception as exc:
                logger.debug("FB sync skip '{}': {}", pol["name"], exc)

        if output:
            logger.info("Facebook sync: {} new posts", len(output))
        return output
