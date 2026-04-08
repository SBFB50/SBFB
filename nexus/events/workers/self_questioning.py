"""
NEXUS -- SelfQuestioningWorker.

Subscribes to HYPOTHESIS_SCORED.  When a top hypothesis score shifts
by more than settings.score_shift_threshold points, challenges it using
adversarial self-questioning via the nexus 26B model.  Creates alerts
with the critique results.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.config import settings
from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class SelfQuestioningWorker(ReactiveWorker):
    """Challenges the top hypothesis when its score shifts significantly."""

    name = "self_questioning"
    subscriptions = [EventType.HYPOTHESIS_SCORED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        delta = abs(event.payload.get("delta", 0))
        if delta < settings.score_shift_threshold:
            return []

        hypothesis_id = event.payload.get("hypothesis_id", "")
        if not hypothesis_id:
            return []

        # Check if this is the top hypothesis
        all_hyps = await self._db.list_hypotheses_by_case(
            event.case_id, status="active"
        )
        if not all_hyps:
            return []

        # Sort by score descending
        all_hyps.sort(
            key=lambda h: h.get("current_score", 0), reverse=True
        )
        top_hyp = all_hyps[0]

        # Only self-question the top hypothesis
        if top_hyp["id"] != hypothesis_id:
            logger.debug(
                "SelfQuestioning: scored hypothesis %s is not the top one, skipping",
                hypothesis_id[:8],
            )
            return []

        logger.info(
            "SelfQuestioning: challenging top hypothesis '%s' (delta=%.1f)",
            top_hyp.get("title", "?")[:50], delta,
        )

        try:
            from nexus.llm.prompts import SELF_QUESTIONING_PROMPT
            from nexus.llm.router import TaskType

            # Build evidence summaries
            evidence_list = await self._db.list_evidence_by_case(event.case_id)
            evidence_summaries = "\n".join(
                f"- [{e.get('title', 'N/A')}]: {(e.get('summary') or '')[:200]}"
                for e in evidence_list
                if e.get("summary")
            ) or "(aucune preuve avec resume)"

            # Build all hypotheses text
            all_hyps_text = "\n".join(
                f"- {h.get('title', '?')} (score: {h.get('current_score', 0):.0f}%): "
                f"{(h.get('description') or '')[:150]}"
                for h in all_hyps
            )

            prompt = SELF_QUESTIONING_PROMPT.format(
                top_hypothesis=top_hyp.get("title", "?"),
                top_score=f"{top_hyp.get('current_score', 0):.0f}",
                top_description=top_hyp.get("description", "")[:500],
                all_hypotheses=all_hyps_text,
                evidence_summaries=evidence_summaries,
            )

            critique = await self._router.route(TaskType.DEEP_ANALYSIS, prompt)

            # Store the critique as an alert
            await self._db.create_alert(
                case_id=event.case_id,
                alert_type="self_questioning",
                severity="info",
                title=f"Auto-critique: {top_hyp.get('title', '?')[:60]}",
                message=critique.strip()[:settings.text_truncation_short],
                related_id=hypothesis_id,
            )

            logger.info(
                "SelfQuestioning: critique generated (%d chars) for hypothesis %s",
                len(critique), hypothesis_id[:8],
            )

        except Exception as exc:
            logger.warning(
                "SelfQuestioning: failed to challenge hypothesis %s: %s",
                hypothesis_id[:8], exc,
            )

        return []
