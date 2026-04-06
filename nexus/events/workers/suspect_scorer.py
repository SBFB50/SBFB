"""
NEXUS -- SuspectScorerWorker.

Subscribes to HYPOTHESIS_SCORED and CONTRADICTION_FOUND.
Runs SuspectScorer.score_all_suspects to recalculate composite
suspicion scores for all person entities.  Emits SUSPECT_SCORED
for each suspect.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class SuspectScorerWorker(ReactiveWorker):
    """Recalculates suspect scores when hypotheses or contradictions change."""

    name = "suspect_scorer"
    subscriptions = [
        EventType.HYPOTHESIS_SCORED,
        EventType.CONTRADICTION_FOUND,
    ]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
        neo4j: Any = None,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._neo4j = neo4j
        self._scorer = None

    def _get_scorer(self):
        if self._scorer is None:
            from nexus.core.suspect_scorer import SuspectScorer
            self._scorer = SuspectScorer(
                self._db, self._router, self._neo4j,
            )
        return self._scorer

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        trigger = (
            "hypothesis_scored"
            if event.event_type == EventType.HYPOTHESIS_SCORED
            else "contradiction_found"
        )

        logger.info(
            "SuspectScorer: rescoring all suspects for case %s (trigger=%s)",
            event.case_id, trigger,
        )

        scorer = self._get_scorer()
        results = await scorer.score_all_suspects(
            case_id=event.case_id,
            trigger=trigger,
        )

        output: list[NexusEvent] = []
        for result in results:
            output.append(NexusEvent(
                event_type=EventType.SUSPECT_SCORED,
                case_id=event.case_id,
                payload={
                    "suspect_id": result.get("suspect_id", ""),
                    "entity_id": result.get("entity_id", ""),
                    "name": result.get("name", ""),
                    "score": result.get("score", 0),
                    "factors": result.get("factors", {}),
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            ))

        logger.info(
            "SuspectScorer: scored %d suspects for case %s",
            len(results), event.case_id,
        )
        return output
