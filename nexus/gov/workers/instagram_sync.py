"""
NEXUS GOV -- Instagram Sync Worker.

Collects public Instagram posts from French politicians via instagrapi.
Falls back to SearXNG search if instagrapi is not installed.

Primary mode: instagrapi Client (public API, no login needed for public profiles).
Fallback mode: SearXNG web search (legacy behavior, less reliable).
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime
from typing import Any, Optional

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

# Lazy-check for instagrapi availability
_INSTAGRAPI_AVAILABLE = False
try:
    from instagrapi import Client as InstaClient
    from instagrapi.exceptions import (
        ClientError,
        UserNotFound,
        PrivateAccount,
        LoginRequired,
    )

    _INSTAGRAPI_AVAILABLE = True
except ImportError:
    InstaClient = None  # type: ignore[assignment,misc]

import httpx

SEARXNG_URL = "http://localhost:8888/search"

# Media type mapping from instagrapi int codes
_MEDIA_TYPE_MAP = {
    1: "photo",
    2: "video",
    8: "album",
}


class GovInstagramSyncWorker(ReactiveWorker):
    name = "gov_instagram_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._batch_size = 30
        self._offset = 0
        self._client: Optional[Any] = None  # Lazy-init instagrapi Client
        # Cache username -> user_id to avoid repeated lookups
        self._uid_cache: dict[str, int] = {}

    # ------------------------------------------------------------------
    # Instagrapi client (lazy, sync -- all calls wrapped in to_thread)
    # ------------------------------------------------------------------

    def _get_client(self) -> Any:
        """Return or create the instagrapi Client (synchronous, call via to_thread)."""
        if self._client is None and _INSTAGRAPI_AVAILABLE:
            self._client = InstaClient()
            # Public mode -- no login. Adjust settings for resilience.
            self._client.delay_range = [1, 3]
        return self._client

    # ------------------------------------------------------------------
    # Handle
    # ------------------------------------------------------------------

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
                if _INSTAGRAPI_AVAILABLE:
                    posts = await self._fetch_via_instagrapi(pol)
                else:
                    posts = await self._fetch_via_searxng(pol)

                for post_data in posts:
                    try:
                        post = await self._db.create_social_post(
                            politician_id=pol["id"],
                            **post_data,
                        )
                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                                case_id="gov",
                                payload={
                                    "post_id": post["id"],
                                    "platform": "instagram",
                                    "politician": pol["name"],
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )
                    except Exception:
                        pass  # Duplicate post_id / URL

                # Rate limiting between politicians
                await asyncio.sleep(3.0)
            except Exception as exc:
                logger.debug("IG sync skip '{}': {}", pol["name"], exc)

        if output:
            logger.info("Instagram sync: {} new posts", len(output))
        return output

    # ------------------------------------------------------------------
    # Primary: instagrapi
    # ------------------------------------------------------------------

    async def _fetch_via_instagrapi(self, pol: dict) -> list[dict]:
        """Fetch latest Instagram posts for a politician using instagrapi."""
        handle = self._resolve_handle(pol)
        if not handle:
            return []

        try:
            # All instagrapi calls are synchronous -- wrap in to_thread
            user_id = await self._get_user_id(handle)
            if user_id is None:
                return []

            medias = await asyncio.to_thread(
                self._get_client().user_medias, user_id, 10
            )
        except Exception as exc:
            # Private account, rate limit, network error, etc.
            logger.debug("IG instagrapi error for @{}: {}", handle, exc)
            return []

        posts: list[dict] = []
        for m in medias:
            media_type = _MEDIA_TYPE_MAP.get(m.media_type, "photo")

            # Pick the best media URL
            media_url = ""
            if m.media_type == 2 and m.video_url:
                media_url = str(m.video_url)
            elif m.thumbnail_url:
                media_url = str(m.thumbnail_url)

            posted_at = ""
            if m.taken_at:
                posted_at = (
                    m.taken_at.isoformat()
                    if isinstance(m.taken_at, datetime)
                    else str(m.taken_at)
                )

            posts.append(
                {
                    "platform": "instagram",
                    "post_id": str(m.pk),
                    "content": (m.caption_text or "")[:2000],
                    "url": f"https://www.instagram.com/p/{m.code}/",
                    "media_type": media_type,
                    "media_url": media_url,
                    "posted_at": posted_at,
                    "likes": m.like_count or 0,
                    "comments": m.comment_count or 0,
                    "metadata": {
                        "shortcode": m.code,
                        "handle": handle,
                    },
                }
            )

        return posts

    def _resolve_handle(self, pol: dict) -> Optional[str]:
        """Extract Instagram handle from politician metadata, or return None."""
        meta = pol.get("metadata") or {}
        if isinstance(meta, str):
            try:
                meta = json.loads(meta)
            except (json.JSONDecodeError, TypeError):
                meta = {}

        handle = meta.get("instagram_handle") or meta.get("instagram")
        if handle:
            # Strip leading @ if present
            return handle.lstrip("@").strip()

        return None

    async def _get_user_id(self, handle: str) -> Optional[int]:
        """Resolve an Instagram handle to a numeric user_id (cached)."""
        if handle in self._uid_cache:
            return self._uid_cache[handle]

        try:
            uid = await asyncio.to_thread(
                self._get_client().user_id_from_username, handle
            )
            self._uid_cache[handle] = uid
            return uid
        except Exception as exc:
            logger.debug("IG user lookup failed for @{}: {}", handle, exc)
            return None

    # ------------------------------------------------------------------
    # Fallback: SearXNG
    # ------------------------------------------------------------------

    async def _fetch_via_searxng(self, pol: dict) -> list[dict]:
        """Fallback: search SearXNG for Instagram posts (legacy behavior)."""
        posts: list[dict] = []
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
                    return []
                data = resp.json()

            for r in data.get("results", [])[:3]:
                url = r.get("url", "")
                if "instagram.com" not in url:
                    continue
                content = r.get("content", r.get("title", ""))
                if not content:
                    continue
                media_url = r.get("img_src", "")
                posts.append(
                    {
                        "platform": "instagram",
                        "content": content[:2000],
                        "url": url,
                        "media_type": "image" if media_url else "text",
                        "media_url": media_url,
                        "posted_at": r.get("publishedDate", ""),
                    }
                )
        except Exception as exc:
            logger.debug("IG SearXNG fallback error for '{}': {}", pol["name"], exc)

        return posts
