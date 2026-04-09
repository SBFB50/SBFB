"""
NEXUS GOV -- Cross-Source Contradiction Analyzer.

The brain of NEXUS GOV. Detects contradictions across ALL sources:
- Tweet says X <-> Vote Y on same subject
- Interview 2020 <-> Declaration 2026 (position reversal)
- Promise <-> Actual vote
- TV statement <-> Written declaration
- Press article content <-> Prior stated positions

Subscribes to every new data event. For each new item, searches for
prior positions from the same politician on similar subjects, then
uses LLM to analyze pairs for factual contradictions.

Features:
- Fuzzy subject matching (keyword overlap) for broader candidate selection
- Temporal awareness (dates included in LLM prompt for evolution detection)
- Press article support (extracts per-politician content, limit 3 politicians)
"""

from __future__ import annotations

import asyncio
from typing import Any

from loguru import logger

from nexus.engine import NexusEvent, ReactiveWorker
from nexus.gov.events import GovEventType


def _subject_keywords(subject: str) -> set[str]:
    """Extract meaningful keywords from a subject string (lowercase, 3+ chars)."""
    if not subject:
        return set()
    # Split on whitespace and common separators, keep words >= 3 chars
    words = subject.lower().replace(",", " ").replace("/", " ").replace("-", " ").split()
    # Filter out short stopwords
    stopwords = {"les", "des", "une", "que", "qui", "par", "sur", "pour", "dans", "avec", "est", "pas"}
    return {w for w in words if len(w) >= 3 and w not in stopwords}


def _subjects_overlap(subject_a: str, subject_b: str) -> bool:
    """Return True if two subjects share at least one meaningful keyword."""
    kw_a = _subject_keywords(subject_a)
    kw_b = _subject_keywords(subject_b)
    if not kw_a or not kw_b:
        return False
    return bool(kw_a & kw_b)


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

        # --- Press articles need per-politician handling ---
        if event_type == GovEventType.GOV_PRESS_ADDED:
            return await self._handle_press(event)

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

        if not politician_id or not new_text:
            return []

        # Idempotency
        if item_id in self._processed:
            return []
        self._processed.add(item_id)
        self._trim_cache()

        # Detect contradictions for this single politician
        return await self._compare_against_prior(
            politician_id=politician_id,
            item_id=item_id,
            new_text=new_text,
            new_subject=new_subject,
            new_source=new_source,
            new_date=new_date,
            event_type_label=event_type.value,
            parent_event_id=event.event_id,
        )

    # ------------------------------------------------------------------
    # Press article handler: iterate over mentioned politicians
    # ------------------------------------------------------------------

    async def _handle_press(self, event: NexusEvent) -> list[NexusEvent]:
        """Handle GOV_PRESS_ADDED by checking each mentioned politician."""
        output: list[NexusEvent] = []
        payload = event.payload
        item_id = payload.get("article_id", "")

        if not item_id:
            return []

        # Idempotency
        if item_id in self._processed:
            return []
        self._processed.add(item_id)
        self._trim_cache()

        # Fetch full article
        article = await self._db.get_press_article(item_id)
        if not article:
            return []

        article_text = article.get("summary", "") or article.get("title", "")
        article_source = article.get("url", "")
        article_date = article.get("published_at", "")

        if not article_text:
            return []

        # Extract politician IDs from the payload (list) or DB field (comma-separated)
        politician_ids: list[str] = payload.get("politicians", [])
        if not politician_ids:
            mentioned_str = article.get("politicians_mentioned", "")
            if mentioned_str:
                politician_ids = [pid.strip() for pid in mentioned_str.split(",") if pid.strip()]

        if not politician_ids:
            return []

        # Limit to first 3 politicians to avoid overloading LLM
        for pol_id in politician_ids[:3]:
            try:
                results = await self._compare_against_prior(
                    politician_id=pol_id,
                    item_id=item_id,
                    new_text=article_text[:1000],
                    new_subject=article.get("subjects", "") or article.get("title", ""),
                    new_source=article_source,
                    new_date=article_date,
                    event_type_label="press",
                    parent_event_id=event.event_id,
                )
                output.extend(results)
            except Exception as exc:
                logger.debug("Press contradiction check failed for {}: {}", pol_id, exc)

            await asyncio.sleep(0)  # Yield between politicians

        return output

    # ------------------------------------------------------------------
    # Core comparison logic (shared by all event types)
    # ------------------------------------------------------------------

    async def _compare_against_prior(
        self,
        *,
        politician_id: str,
        item_id: str,
        new_text: str,
        new_subject: str,
        new_source: str,
        new_date: str,
        event_type_label: str,
        parent_event_id: str,
    ) -> list[NexusEvent]:
        """Compare new content against prior positions/social posts for contradictions."""
        output: list[NexusEvent] = []

        # Fetch prior positions (increased limit for fuzzy subject matching)
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

        # Rank candidates: fuzzy subject match first, then recent
        top = self._rank_candidates(candidates, new_subject)

        # Use LLM to detect contradictions
        if not self._router:
            # No LLM available -- skip
            return []

        from nexus.engine import POLITICAL_CONTRADICTION_PROMPT, TaskType

        for cand in top:
            try:
                prompt = POLITICAL_CONTRADICTION_PROMPT.format(
                    date_a=cand["date"],
                    type_a=cand["type"],
                    subject=cand["subject"] or new_subject or "General",
                    text_a=cand["text"],
                    source_a=cand["source"],
                    date_b=new_date,
                    type_b=event_type_label,
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
                                parent_event_id=parent_event_id,
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

    # ------------------------------------------------------------------
    # Candidate ranking with fuzzy subject matching
    # ------------------------------------------------------------------

    @staticmethod
    def _rank_candidates(
        candidates: list[dict[str, str]],
        new_subject: str,
    ) -> list[dict[str, str]]:
        """Rank candidates by subject relevance. Fuzzy keyword overlap first, then rest.

        Returns at most 10 candidates (max LLM calls per item).
        """
        matching: list[dict[str, str]] = []
        non_matching: list[dict[str, str]] = []

        for cand in candidates:
            if _subjects_overlap(cand.get("subject", ""), new_subject):
                matching.append(cand)
            else:
                non_matching.append(cand)

        # Subject matches first, then remaining candidates (already date-sorted from DB)
        ranked = matching + non_matching
        return ranked[:10]  # Max 10 comparisons per new item

    # ------------------------------------------------------------------
    # Cache management
    # ------------------------------------------------------------------

    def _trim_cache(self) -> None:
        """Keep the idempotency cache bounded."""
        if len(self._processed) > 10000:
            self._processed = set(list(self._processed)[-5000:])
