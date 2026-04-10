"""
NEXUS GOV -- Twitter/X Sync Worker.

Collects public tweets from French politicians via twikit GuestClient.
No Twitter API key required -- uses guest token for read-only access.
Falls back to SearXNG search if twikit is unavailable or rate-limited.
"""

from __future__ import annotations

import asyncio
from datetime import datetime
from typing import Any, Optional

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType

# SearXNG fallback
import httpx

SEARXNG_URL = "http://localhost:8888/search"

# Lazy-loaded twikit availability flag
_TWIKIT_AVAILABLE: Optional[bool] = None


def _check_twikit() -> bool:
    """Check whether twikit is installed (cached after first call)."""
    global _TWIKIT_AVAILABLE
    if _TWIKIT_AVAILABLE is None:
        try:
            from twikit.guest import GuestClient  # noqa: F401

            _TWIKIT_AVAILABLE = True
        except ImportError:
            _TWIKIT_AVAILABLE = False
            logger.warning(
                "twikit not installed -- Twitter sync will use SearXNG fallback"
            )
    return _TWIKIT_AVAILABLE


class GovTwitterSyncWorker(ReactiveWorker):
    name = "gov_twitter_sync"
    subscriptions = [GovEventType.TICK_HOURLY]

    def __init__(self, bus: Any, db: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._batch_size = 20  # politicians per tick
        self._offset = 0
        self._client: Any = None  # twikit GuestClient (lazy)
        self._client_failed = False  # sticky flag after repeated failures
        self._consecutive_client_errors = 0

    # ------------------------------------------------------------------
    # twikit guest client lifecycle
    # ------------------------------------------------------------------

    async def _get_client(self) -> Any:
        """Return an activated GuestClient, or None if unavailable."""
        if not _check_twikit():
            return None

        # If we had too many consecutive failures, stop trying until next tick
        if self._client_failed:
            return None

        if self._client is not None:
            return self._client

        try:
            from twikit.guest import GuestClient

            client = GuestClient()
            await client.activate()
            self._client = client
            self._consecutive_client_errors = 0
            logger.info("Twitter GuestClient activated")
            return self._client
        except Exception as exc:
            logger.warning("Failed to activate Twitter GuestClient: {}", exc)
            self._client = None
            return None

    def _record_client_error(self) -> None:
        """Track consecutive twikit errors; disable after 3 in a row."""
        self._consecutive_client_errors += 1
        if self._consecutive_client_errors >= 3:
            logger.warning(
                "Twitter GuestClient failed {} times in a row -- "
                "disabling for this tick, will retry next tick",
                self._consecutive_client_errors,
            )
            self._client_failed = True
            self._client = None  # Force re-activation next tick

    # ------------------------------------------------------------------
    # Main handler
    # ------------------------------------------------------------------

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []

        # Reset sticky failure flag each tick so we retry twikit
        self._client_failed = False
        self._consecutive_client_errors = 0

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
                new_posts = await self._fetch_tweets_twikit(pol, event)
                if new_posts is not None:
                    output.extend(new_posts)
                else:
                    # twikit unavailable or failed -- fall back to SearXNG
                    fallback_posts = await self._fetch_tweets_searxng(pol, event)
                    output.extend(fallback_posts)
            except Exception as exc:
                logger.debug("Twitter sync skip '{}': {}", name, exc)

            await asyncio.sleep(2.0)

        if output:
            logger.info("Twitter sync: {} new posts", len(output))
        return output

    # ------------------------------------------------------------------
    # twikit-based fetching
    # ------------------------------------------------------------------

    async def _fetch_tweets_twikit(
        self, pol: dict, event: NexusEvent
    ) -> Optional[list[NexusEvent]]:
        """Fetch tweets via twikit GuestClient.

        Returns list of events on success, or None if twikit is
        unavailable / failed (caller should fall back to SearXNG).
        """
        client = await self._get_client()
        if client is None:
            return None

        # Build search query -- prefer twitter handle from metadata
        metadata = pol.get("metadata") or {}
        if isinstance(metadata, str):
            try:
                import json

                metadata = json.loads(metadata)
            except Exception:
                metadata = {}

        handle = metadata.get("twitter_handle") or metadata.get("twitter")
        if handle:
            # Strip leading @ if present
            handle = handle.lstrip("@")
            query = f"from:{handle}"
        else:
            # Search by name (less precise, but still useful)
            query = f'"{pol["name"]}" lang:fr'

        try:
            tweets = await client.search_tweet(query, product="Latest")
        except Exception as exc:
            logger.warning(
                "twikit search failed for '{}': {}", pol["name"], exc
            )
            self._record_client_error()
            # Force new guest token on next attempt
            self._client = None
            return None

        output: list[NexusEvent] = []
        tweet_list = list(tweets)[:10] if tweets else []

        for tweet in tweet_list:
            try:
                tweet_id = str(tweet.id)
                screen_name = getattr(tweet.user, "screen_name", "") if tweet.user else ""
                url = f"https://x.com/{screen_name}/status/{tweet_id}" if screen_name else ""

                # Parse posted_at -- twikit returns a datetime string
                posted_at = ""
                raw_date = getattr(tweet, "created_at", None)
                if raw_date:
                    if isinstance(raw_date, datetime):
                        posted_at = raw_date.isoformat()
                    elif isinstance(raw_date, str):
                        # twikit format: "Wed Oct 10 20:19:24 +0000 2018"
                        try:
                            dt = datetime.strptime(
                                raw_date, "%a %b %d %H:%M:%S %z %Y"
                            )
                            posted_at = dt.isoformat()
                        except ValueError:
                            posted_at = raw_date

                post = await self._db.create_social_post(
                    politician_id=pol["id"],
                    platform="twitter",
                    post_id=tweet_id,
                    content=(tweet.text or "")[:2000],
                    url=url,
                    media_type="text",
                    posted_at=posted_at,
                    likes=getattr(tweet, "favorite_count", 0) or 0,
                    shares=getattr(tweet, "retweet_count", 0) or 0,
                    comments=getattr(tweet, "reply_count", 0) or 0,
                )
                output.append(
                    NexusEvent(
                        event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                        case_id="gov",
                        payload={
                            "post_id": post["id"],
                            "platform": "twitter",
                            "politician": pol["name"],
                            "tweet_id": tweet_id,
                        },
                        source_worker=self.name,
                        parent_event_id=event.event_id,
                    )
                )
            except Exception:
                pass  # Duplicate post_id (UNIQUE constraint) or parse error

        # Reset error counter on success
        self._consecutive_client_errors = 0
        return output

    # ------------------------------------------------------------------
    # SearXNG fallback (original approach)
    # ------------------------------------------------------------------

    async def _fetch_tweets_searxng(
        self, pol: dict, event: NexusEvent
    ) -> list[NexusEvent]:
        """Fallback: search SearXNG for tweets when twikit is unavailable."""
        output: list[NexusEvent] = []
        name = pol["name"]

        try:
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
                    return []
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

        except Exception as exc:
            logger.debug("Twitter SearXNG fallback skip '{}': {}", name, exc)

        return output
