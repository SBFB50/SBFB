"""
NEXUS GOV -- Cross-Source Contradiction Analyzer.

The brain of NEXUS GOV. Detects contradictions across ALL sources:
- Tweet says X <-> Vote Y on same subject
- Interview 2020 <-> Declaration 2026 (position reversal)
- Promise <-> Actual vote
- TV statement <-> Written declaration

Subscribes to every new data event. For each new item, searches for
prior positions from the same politician on similar subjects, then
uses LLM to analyze pairs for factual contradictions.
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.events.types import NexusEvent
from nexus.events.worker import ReactiveWorker
from nexus.gov.events import GovEventType


class GovContradictionAnalyzer(ReactiveWorker):
    name = "gov_contradiction_analyzer"
    subscriptions = [
        GovEventType.GOV_POSITION_ADDED,
        GovEventType.GOV_SOCIAL_POST_ADDED,
        GovEventType.GOV_TRANSCRIPTION_READY,
        GovEventType.GOV_PRESS_ADDED,
    ]

    def __init__(self, bus: Any, db: Any, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._processed: set[str] = set()  # Idempotency guard

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        output: list[NexusEvent] = []
        event_type = event.event_type
        payload = event.payload

        # Determine politician_id and the new text to compare
        politician_id = payload.get("politician_id")
        new_text = ""
        new_subject = ""
        new_source = ""
        new_date = ""
        item_id = ""

        if event_type == GovEventType.GOV_POSITION_ADDED:
            item_id = payload.get("position_id", "")
            # Fetch the position
            pos = await self._db.get_position(item_id) if item_id else None
            if pos:
                new_text = pos.get("position_text", "")
                new_subject = pos.get("subject", "")
                new_source = pos.get("source_url", "")
                new_date = pos.get("date", "")
                politician_id = pos.get("politician_id", politician_id)

        elif event_type == GovEventType.GOV_SOCIAL_POST_ADDED:
            item_id = payload.get("post_id", "")
            # Fetch the social post
            posts = await self._db.list_social_by_politician(
                politician_id or "", limit=1
            )
            # Find by ID in recent posts
            for p in posts:
                if p.get("id") == item_id:
                    new_text = p.get("content", "")
                    new_source = p.get("url", "")
                    new_date = p.get("posted_at", "")
                    break

        elif event_type == GovEventType.GOV_TRANSCRIPTION_READY:
            item_id = payload.get("transcription_id", "")
            politician_id = payload.get("politician_id")
            title = payload.get("title", "")
            # Get transcription text (summarized -- full text too long for LLM)
            transcriptions = await self._db.list_transcriptions_by_politician(
                politician_id or "", limit=10
            )
            for t in transcriptions:
                if t.get("id") == item_id:
                    new_text = (t.get("transcription", ""))[:3000]  # Truncate for LLM
                    new_source = t.get("source_url", "")
                    new_subject = title
                    break

        elif event_type == GovEventType.GOV_PRESS_ADDED:
            item_id = payload.get("article_id", "")
            # Press articles mention multiple politicians
            # For now, skip (would need per-politician extraction)
            return []

        if not politician_id or not new_text:
            return []

        # Idempotency
        if item_id in self._processed:
            return []
        self._processed.add(item_id)
        # Keep cache bounded
        if len(self._processed) > 10000:
            self._processed = set(list(self._processed)[-5000:])

        # Fetch all prior positions from this politician
        prior_positions = await self._db.list_positions_by_politician(
            politician_id, limit=200
        )
        prior_social = await self._db.list_social_by_politician(
            politician_id, limit=100
        )

        # Build comparison candidates: positions with similar subjects
        candidates: list[dict[str, str]] = []
        for pos in prior_positions:
            if pos.get("id") == item_id:
                continue
            pos_text = pos.get("position_text", "")
            pos_subject = pos.get("subject", "")
            if not pos_text:
                continue
            candidates.append(
                {
                    "text": pos_text[:1000],
                    "subject": pos_subject,
                    "date": pos.get("date", ""),
                    "source": pos.get("source_url", ""),
                    "type": pos.get("position_type", "position"),
                }
            )

        # Also include social posts as candidates
        for sp in prior_social:
            if sp.get("id") == item_id:
                continue
            sp_text = sp.get("content", "")
            if not sp_text:
                continue
            candidates.append(
                {
                    "text": sp_text[:1000],
                    "subject": "",
                    "date": sp.get("posted_at", ""),
                    "source": sp.get("url", ""),
                    "type": sp.get("platform", "social"),
                }
            )

        if not candidates:
            return []

        # Select top candidates for comparison (limit LLM calls)
        # Priority: same subject > different type > recent
        top = candidates[:10]  # Max 10 comparisons per new item

        # Use LLM to detect contradictions
        if not self._router:
            # No LLM available -- skip
            return []

        from nexus.llm.prompts import POLITICAL_CONTRADICTION_PROMPT
        from nexus.llm.router import TaskType

        for cand in top:
            try:
                prompt = POLITICAL_CONTRADICTION_PROMPT.format(
                    date_a=cand["date"],
                    type_a=cand["type"],
                    subject=cand["subject"] or new_subject or "General",
                    text_a=cand["text"],
                    source_a=cand["source"],
                    date_b=new_date,
                    type_b=event_type.value,
                    text_b=new_text[:1000],
                    source_b=new_source,
                )

                result = await self._router.route_json(
                    TaskType.CONTRADICTION_DETECTION, prompt
                )

                contradictions: list[dict[str, Any]] = []
                if isinstance(result, dict):
                    contradictions = result.get("contradictions", [])

                for c in contradictions:
                    desc = c.get("description", "")
                    severity = c.get("severity", "medium")
                    if not desc:
                        continue

                    try:
                        record = await self._db.create_contradiction(
                            politician_id=politician_id,
                            position_a_id=cand.get("source", ""),  # Best available ref
                            position_b_id=item_id,
                            subject=cand["subject"] or new_subject or "Cross-source",
                            description=desc,
                            severity=severity,
                        )

                        output.append(
                            NexusEvent(
                                event_type=GovEventType.GOV_CONTRADICTION_FOUND,
                                case_id="gov",
                                payload={
                                    "contradiction_id": record["id"],
                                    "politician_id": politician_id,
                                    "description": desc[:200],
                                    "severity": severity,
                                },
                                source_worker=self.name,
                                parent_event_id=event.event_id,
                            )
                        )
                        logger.info(
                            "Contradiction found for {}: {}",
                            politician_id,
                            desc[:80],
                        )
                    except Exception as exc:
                        logger.debug("Store contradiction failed: {}", exc)

            except Exception as exc:
                logger.debug("LLM contradiction check failed: {}", exc)

            await asyncio.sleep(0)  # Yield for cancellation

        return output
