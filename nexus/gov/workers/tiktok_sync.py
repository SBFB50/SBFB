"""
NEXUS GOV -- TikTok Sync Worker.

Collects TikTok videos from French politicians using yt-dlp (flat playlist mode).
No TikTokApi/Playwright dependency -- yt-dlp handles TikTok natively.

For each politician:
  1. Resolve TikTok handle (metadata.tiktok_handle or SearXNG lookup)
  2. yt-dlp flat_playlist to list recent videos (no download)
  3. Store in gov_social_posts (platform="tiktok")
  4. High-view videos (>10k) emit GOV_VIDEO_DOWNLOADED for transcription
"""

from __future__ import annotations

import asyncio
import json
from datetime import datetime, timezone
from typing import Any, Optional

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

import httpx

SEARXNG_URL = "http://localhost:8888/search"

# yt-dlp options for flat playlist extraction (no download)
YTDLP_OPTS = {
    "flat_playlist": True,
    "quiet": True,
    "no_warnings": True,
    "extract_flat": "in_playlist",
    "playlistend": 10,
    "ignoreerrors": True,
}

# Views threshold for triggering video download + transcription
HIGH_VIEW_THRESHOLD = 10_000


def _extract_tiktok_videos(username: str, limit: int = 10) -> list[dict]:
    """Synchronous yt-dlp call to list TikTok videos for a user.

    Returns list of dicts with: id, title, view_count, like_count, url, timestamp.
    Runs in a thread via asyncio.to_thread().
    """
    import yt_dlp

    url = f"https://www.tiktok.com/@{username}"
    opts = {
        **YTDLP_OPTS,
        "playlistend": limit,
    }
    entries = []
    try:
        with yt_dlp.YoutubeDL(opts) as ydl:
            info = ydl.extract_info(url, download=False)
            if not info:
                return []
            for entry in (info.get("entries") or [])[:limit]:
                if not entry:
                    continue
                entries.append({
                    "id": entry.get("id", ""),
                    "title": entry.get("title", ""),
                    "description": entry.get("description", entry.get("title", "")),
                    "view_count": entry.get("view_count") or 0,
                    "like_count": entry.get("like_count") or 0,
                    "comment_count": entry.get("comment_count") or 0,
                    "url": entry.get("url") or entry.get("webpage_url")
                    or f"https://www.tiktok.com/@{username}/video/{entry.get('id', '')}",
                    "timestamp": entry.get("timestamp"),
                    "upload_date": entry.get("upload_date"),
                })
    except Exception as exc:
        logger.debug("yt-dlp TikTok extraction failed for @{}: {}", username, exc)

    return entries


class GovTikTokSyncWorker(ReactiveWorker):
    """Syncs TikTok videos from politicians using yt-dlp flat playlist mode."""

    name = "gov_tiktok_sync"
    subscriptions = [GovEventType.TICK_DAILY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._batch_size = 20
        self._offset = 0
        # Cache resolved handles: politician_id -> tiktok_username | None
        self._handle_cache: dict[str, Optional[str]] = {}

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
            pol_id = pol["id"]
            pol_name = pol["name"]

            try:
                # 1. Resolve TikTok handle
                handle = await self._resolve_handle(pol)
                if not handle:
                    continue

                # 2. Fetch recent videos via yt-dlp (sync, run in thread)
                entries = await asyncio.to_thread(
                    _extract_tiktok_videos, handle, 10
                )
                if not entries:
                    await asyncio.sleep(3.0)
                    continue

                # 3. Store each video as a social post
                for entry in entries:
                    video_id = entry["id"]
                    if not video_id:
                        continue

                    video_url = entry["url"]
                    description = (entry.get("description") or entry.get("title") or "")[:2000]
                    view_count = entry.get("view_count", 0)
                    like_count = entry.get("like_count", 0)
                    comment_count = entry.get("comment_count", 0)

                    # Parse posted_at from timestamp or upload_date
                    posted_at = ""
                    if entry.get("timestamp"):
                        try:
                            posted_at = datetime.fromtimestamp(
                                entry["timestamp"], tz=timezone.utc
                            ).isoformat()
                        except (OSError, ValueError):
                            pass
                    elif entry.get("upload_date"):
                        try:
                            posted_at = datetime.strptime(
                                entry["upload_date"], "%Y%m%d"
                            ).replace(tzinfo=timezone.utc).isoformat()
                        except ValueError:
                            pass

                    try:
                        post = await self._db.create_social_post(
                            politician_id=pol_id,
                            platform="tiktok",
                            post_id=video_id,
                            content=description,
                            url=video_url,
                            media_type="video",
                            posted_at=posted_at,
                            likes=like_count,
                            comments=comment_count,
                            metadata=json.dumps({
                                "views": view_count,
                                "tiktok_handle": handle,
                            }),
                        )
                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                                case_id="gov",
                                payload={
                                    "post_id": post["id"],
                                    "platform": "tiktok",
                                    "politician": pol_name,
                                    "views": view_count,
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )

                        # 4. High-view videos -> emit for transcription
                        if view_count >= HIGH_VIEW_THRESHOLD:
                            output.append(
                                NexusEvent(
                                    event_type=GovEventType.GOV_VIDEO_DOWNLOADED,
                                    case_id="gov",
                                    payload={
                                        "politician_id": pol_id,
                                        "politician_name": pol_name,
                                        "video_url": video_url,
                                        "title": (entry.get("title") or "")[:200],
                                        "platform": "tiktok",
                                        "views": view_count,
                                    },
                                    source_worker=self.name,
                                    parent_event_id=event.event_id,
                                )
                            )
                            logger.info(
                                "TikTok high-view video: '{}' ({} views) for {}",
                                (entry.get("title") or "")[:50],
                                view_count,
                                pol_name,
                            )

                    except Exception:
                        pass  # Duplicate URL / post_id

                # Rate limiting between politicians
                await asyncio.sleep(3.0)

            except Exception as exc:
                logger.debug("TikTok sync skip '{}': {}", pol_name, exc)

        if output:
            new_posts = sum(
                1 for e in output
                if e.event_type == GovEventType.GOV_SOCIAL_POST_ADDED
            )
            logger.info("TikTok sync: {} new posts", new_posts)
        return output

    async def _resolve_handle(self, pol: dict) -> Optional[str]:
        """Resolve TikTok handle for a politician.

        Priority:
          1. Cached value
          2. metadata.tiktok_handle from DB
          3. SearXNG search fallback
        """
        pol_id = pol["id"]

        # Check cache (None means "already tried, no handle found")
        if pol_id in self._handle_cache:
            return self._handle_cache[pol_id]

        # Check metadata field
        raw_meta = pol.get("metadata")
        if raw_meta:
            try:
                meta = json.loads(raw_meta) if isinstance(raw_meta, str) else raw_meta
                handle = meta.get("tiktok_handle") or meta.get("tiktok")
                if handle:
                    # Normalize: strip @ prefix
                    handle = handle.lstrip("@").strip()
                    if handle:
                        self._handle_cache[pol_id] = handle
                        return handle
            except (json.JSONDecodeError, TypeError):
                pass

        # SearXNG fallback: search for their TikTok profile
        try:
            async with httpx.AsyncClient(timeout=15.0) as client:
                resp = await client.get(
                    SEARXNG_URL,
                    params={
                        "q": f'"{pol["name"]}" site:tiktok.com',
                        "format": "json",
                        "language": "fr",
                    },
                )
                if resp.status_code == 200:
                    data = resp.json()
                    for r in data.get("results", [])[:5]:
                        url = r.get("url", "")
                        # Match pattern: https://www.tiktok.com/@username
                        if "tiktok.com/@" in url:
                            # Extract username from URL
                            at_idx = url.index("tiktok.com/@") + len("tiktok.com/@")
                            rest = url[at_idx:]
                            # Username ends at ? or / or end of string
                            handle = ""
                            for ch in rest:
                                if ch in ("?", "/", "#"):
                                    break
                                handle += ch
                            handle = handle.strip()
                            if handle and len(handle) > 1:
                                self._handle_cache[pol_id] = handle
                                return handle
        except Exception as exc:
            logger.debug(
                "TikTok handle SearXNG lookup failed for '{}': {}",
                pol["name"], exc,
            )

        # No handle found -- cache negative result
        self._handle_cache[pol_id] = None
        return None
