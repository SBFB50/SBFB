"""
NEXUS GOV -- Social Publish Worker (Phase 6.6).

Auto-generates and publishes factual posts about political contradictions.
Supports Bluesky (AT Protocol) and Twitter/X (twikit).

Triggered by GOV_CONTRADICTION_FOUND events.
Rate-limited to 3 posts per day.
All platforms are optional — graceful skip if not configured.
"""

from __future__ import annotations

import json
import os
from datetime import datetime, timedelta, timezone
from typing import Any, Optional

import httpx
from loguru import logger

from nexus.engine import (
    _new_id, _now_iso, _row_to_dict, get_db,
    NexusEvent, ReactiveWorker,
)
from nexus.gov.events import GovEventType

# Platform credentials — all optional
BSKY_HANDLE = os.environ.get("NEXUS_BSKY_HANDLE", "")
BSKY_PASSWORD = os.environ.get("NEXUS_BSKY_PASSWORD", "")

TWITTER_USER = os.environ.get("NEXUS_TWITTER_USER", "")
TWITTER_PASS = os.environ.get("NEXUS_TWITTER_PASS", "")
TWITTER_EMAIL = os.environ.get("NEXUS_TWITTER_EMAIL", "")

# Rate limit: max posts per day
MAX_POSTS_PER_DAY = 3


class GovSocialPublishWorker(ReactiveWorker):
    """Publishes factual posts about detected contradictions."""

    name = "gov_social_publish"
    subscriptions = [GovEventType.GOV_CONTRADICTION_FOUND]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        payload = event.payload

        contradiction_id = payload.get("contradiction_id", "")
        politician_id = payload.get("politician_id", "")
        description = payload.get("description", "")
        severity = payload.get("severity", "medium")

        if not description:
            logger.debug("Social publish: empty contradiction description, skipping")
            return output

        # Check daily rate limit
        if await self._is_rate_limited():
            logger.info(
                "Social publish: rate limit reached ({}/day), skipping",
                MAX_POSTS_PER_DAY,
            )
            return output

        # Get politician name
        politician_name = await self._get_politician_name(politician_id)

        # Get contradiction subject
        subject = await self._get_contradiction_subject(contradiction_id)

        # Build post text
        post_text = await self._build_post_text(
            politician_name=politician_name,
            subject=subject,
            description=description,
            severity=severity,
        )

        if not post_text:
            logger.warning("Social publish: failed to generate post text")
            return output

        # Check if any platform is configured
        platforms_configured = bool(BSKY_HANDLE) or bool(TWITTER_USER)

        if not platforms_configured:
            logger.info(
                "Social publish: no platform configured, post generated but not sent: {}",
                post_text[:80],
            )
            # Still store the generated post for audit purposes
            await self._store_post(
                politician_id=politician_id,
                content=post_text,
                platform="nexus_draft",
                post_id=None,
                metadata={"contradiction_id": contradiction_id, "status": "draft"},
            )
            return output

        # Publish to Bluesky
        bsky_result = await self._publish_bluesky(post_text)
        if bsky_result:
            post_row = await self._store_post(
                politician_id=politician_id,
                content=post_text,
                platform="nexus_bluesky",
                post_id=bsky_result.get("uri", ""),
                metadata={
                    "contradiction_id": contradiction_id,
                    "bsky_uri": bsky_result.get("uri", ""),
                    "bsky_cid": bsky_result.get("cid", ""),
                },
            )
            output.append(
                NexusEvent(
                    event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                    case_id="gov",
                    payload={
                        "post_id": post_row.get("id", "") if post_row else "",
                        "platform": "nexus_bluesky",
                        "politician_id": politician_id,
                        "content": post_text,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            )
            logger.info("Published to Bluesky: {}", post_text[:80])

        # Publish to Twitter/X
        twitter_result = await self._publish_twitter(post_text)
        if twitter_result:
            post_row = await self._store_post(
                politician_id=politician_id,
                content=post_text,
                platform="nexus_twitter",
                post_id=twitter_result.get("tweet_id", ""),
                metadata={
                    "contradiction_id": contradiction_id,
                    "tweet_id": twitter_result.get("tweet_id", ""),
                },
            )
            output.append(
                NexusEvent(
                    event_type=GovEventType.GOV_SOCIAL_POST_ADDED,
                    case_id="gov",
                    payload={
                        "post_id": post_row.get("id", "") if post_row else "",
                        "platform": "nexus_twitter",
                        "politician_id": politician_id,
                        "content": post_text,
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                )
            )
            logger.info("Published to Twitter/X: {}", post_text[:80])

        return output

    # ------------------------------------------------------------------
    # Post text generation
    # ------------------------------------------------------------------

    async def _build_post_text(
        self,
        *,
        politician_name: str,
        subject: str,
        description: str,
        severity: str,
    ) -> str:
        """Build a factual post text (max 280 chars)."""
        # Try LLM generation first
        if self._router:
            try:
                from nexus.engine import TaskType

                prompt = (
                    "Redige un court message factuel (max 250 caracteres) "
                    "pour signaler une contradiction politique detectee.\n\n"
                    "REGLES STRICTES:\n"
                    "- Ton neutre et factuel, presomption d'innocence\n"
                    "- Dire 'contradiction detectee', jamais 'mensonge'\n"
                    "- Pas d'emojis\n"
                    "- Terminer par 'Source: NEXUS GOV'\n"
                    "- Ne pas depasser 250 caracteres\n\n"
                    f"Politicien: {politician_name}\n"
                    f"Sujet: {subject}\n"
                    f"Description: {description[:200]}\n\n"
                    "MESSAGE:"
                )
                llm_text = await self._router.route(TaskType.SUMMARIZE, prompt)
                if llm_text:
                    llm_text = llm_text.strip().strip('"').strip("'")
                    # Ensure it fits in 280 chars
                    if len(llm_text) <= 280:
                        return llm_text
                    # Truncate with ellipsis if too long
                    return llm_text[:277] + "..."
            except Exception as exc:
                logger.debug("Social publish LLM generation failed: {}", exc)

        # Fallback: build post manually
        short_desc = description[:120] if len(description) > 120 else description
        post = f"{politician_name}: {subject}\nContradiction detectee — {short_desc}\nSource: NEXUS GOV"

        if len(post) > 280:
            # Shorten description to fit
            max_desc_len = 280 - len(f"{politician_name}: {subject}\nContradiction detectee — \nSource: NEXUS GOV")
            if max_desc_len > 20:
                post = (
                    f"{politician_name}: {subject}\n"
                    f"Contradiction detectee — {description[:max_desc_len]}...\n"
                    "Source: NEXUS GOV"
                )
            else:
                post = (
                    f"{politician_name}: contradiction detectee ({subject})\n"
                    "Source: NEXUS GOV"
                )

        return post[:280]

    # ------------------------------------------------------------------
    # Rate limiting
    # ------------------------------------------------------------------

    async def _is_rate_limited(self) -> bool:
        """Check if we've hit the daily post limit."""
        try:
            async with get_db() as conn:
                cursor = await conn.execute(
                    "SELECT COUNT(*) FROM gov_social_posts "
                    "WHERE platform IN ('nexus_bluesky', 'nexus_twitter') "
                    "AND created_at >= datetime('now', '-1 day')"
                )
                row = await cursor.fetchone()
                count = row[0] if row else 0
                return count >= MAX_POSTS_PER_DAY
        except Exception as exc:
            logger.debug("Rate limit check failed: {}", exc)
            return False

    # ------------------------------------------------------------------
    # Data fetching helpers
    # ------------------------------------------------------------------

    async def _get_politician_name(self, politician_id: str) -> str:
        """Get politician name from ID."""
        if not politician_id:
            return "Politicien inconnu"
        try:
            async with get_db() as conn:
                cursor = await conn.execute(
                    "SELECT name FROM gov_politicians WHERE id = ?",
                    (politician_id,),
                )
                row = await cursor.fetchone()
                if row:
                    return row[0]
        except Exception as exc:
            logger.debug("Failed to fetch politician name: {}", exc)
        return "Politicien inconnu"

    async def _get_contradiction_subject(self, contradiction_id: str) -> str:
        """Get contradiction subject from ID."""
        if not contradiction_id:
            return "sujet non precise"
        try:
            async with get_db() as conn:
                cursor = await conn.execute(
                    "SELECT subject FROM gov_contradictions WHERE id = ?",
                    (contradiction_id,),
                )
                row = await cursor.fetchone()
                if row:
                    return row[0]
        except Exception as exc:
            logger.debug("Failed to fetch contradiction subject: {}", exc)
        return "sujet non precise"

    # ------------------------------------------------------------------
    # Platform publishing
    # ------------------------------------------------------------------

    async def _publish_bluesky(self, text: str) -> Optional[dict]:
        """Publish to Bluesky via AT Protocol. Returns record dict or None."""
        if not BSKY_HANDLE or not BSKY_PASSWORD:
            return None

        try:
            async with httpx.AsyncClient(timeout=30.0) as client:
                # Create session (authenticate)
                session_resp = await client.post(
                    "https://bsky.social/xrpc/com.atproto.server.createSession",
                    json={
                        "identifier": BSKY_HANDLE,
                        "password": BSKY_PASSWORD,
                    },
                )
                if session_resp.status_code != 200:
                    logger.warning(
                        "Bluesky auth failed ({}): {}",
                        session_resp.status_code,
                        session_resp.text[:200],
                    )
                    return None

                session_data = session_resp.json()
                token = session_data["accessJwt"]
                did = session_data["did"]

                # Create post record
                now_iso = datetime.now(timezone.utc).isoformat()
                create_resp = await client.post(
                    "https://bsky.social/xrpc/com.atproto.repo.createRecord",
                    headers={"Authorization": f"Bearer {token}"},
                    json={
                        "repo": did,
                        "collection": "app.bsky.feed.post",
                        "record": {
                            "$type": "app.bsky.feed.post",
                            "text": text,
                            "createdAt": now_iso,
                        },
                    },
                )

                if create_resp.status_code == 200:
                    return create_resp.json()
                else:
                    logger.warning(
                        "Bluesky post failed ({}): {}",
                        create_resp.status_code,
                        create_resp.text[:200],
                    )
                    return None

        except Exception as exc:
            logger.error("Bluesky publish error: {}", exc)
            return None

    async def _publish_twitter(self, text: str) -> Optional[dict]:
        """Publish to Twitter/X via twikit. Returns dict with tweet_id or None."""
        if not TWITTER_USER or not TWITTER_PASS:
            return None

        try:
            from twikit import Client as TwikitClient
        except ImportError:
            logger.debug("twikit not installed, skipping Twitter publish")
            return None

        try:
            client = TwikitClient("fr")
            await client.login(
                auth_info_1=TWITTER_USER,
                auth_info_2=TWITTER_EMAIL or TWITTER_USER,
                password=TWITTER_PASS,
            )

            tweet = await client.create_tweet(text=text)
            tweet_id = getattr(tweet, "id", "") or ""

            logger.info("Twitter post created: {}", tweet_id)
            return {"tweet_id": str(tweet_id)}

        except Exception as exc:
            logger.error("Twitter publish error: {}", exc)
            return None

    # ------------------------------------------------------------------
    # Storage
    # ------------------------------------------------------------------

    async def _store_post(
        self,
        *,
        politician_id: str,
        content: str,
        platform: str,
        post_id: Optional[str],
        metadata: Optional[dict] = None,
    ) -> Optional[dict]:
        """Store published post in gov_social_posts table."""
        try:
            from nexus.gov.db import GovernmentDatabase

            async with get_db() as conn:
                db = GovernmentDatabase(conn)
                row = await db.create_social_post(
                    politician_id=politician_id,
                    platform=platform,
                    content=content,
                    post_id=post_id or _new_id(),
                    posted_at=_now_iso(),
                    metadata=metadata,
                )
                return row
        except Exception as exc:
            logger.error("Failed to store social post: {}", exc)
            return None
