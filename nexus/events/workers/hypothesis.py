"""
NEXUS -- HypothesisWorker.

Subscribes to ANALYSIS_COMPLETED.  If no hypotheses exist for the case,
generates new ones via HypothesisEngine.  Otherwise re-evaluates all
active hypotheses.  Emits HYPOTHESIS_CREATED or HYPOTHESIS_SCORED.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class HypothesisWorker(ReactiveWorker):
    """Generates or re-evaluates hypotheses after analysis completes."""

    name = "hypothesis_engine"
    subscriptions = [EventType.ANALYSIS_COMPLETED, EventType.EVIDENCE_PROCESSED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
        chroma: Any = None,
        neo4j: Any = None,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._chroma = chroma
        self._neo4j = neo4j
        self._engine = None

    def _get_engine(self):
        if self._engine is None:
            from nexus.core.hypothesis_engine import HypothesisEngine
            self._engine = HypothesisEngine(
                self._db, self._router, self._chroma, self._neo4j,
            )
        return self._engine

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        engine = self._get_engine()
        output: list[NexusEvent] = []

        # For EVIDENCE_PROCESSED: only trigger if enough evidence (≥3)
        if event.event_type == EventType.EVIDENCE_PROCESSED:
            evidence = await self._db.list_evidence_by_case(event.case_id)
            if len(evidence) < 3:
                return []

        # Check if hypotheses already exist
        existing = await self._db.list_hypotheses_by_case(event.case_id)

        if not existing:
            # Generate initial hypotheses
            logger.info(
                "HypothesisWorker: no hypotheses for case %s, generating",
                event.case_id,
            )
            created = await engine.generate_hypotheses(event.case_id)

            for hyp in created:
                output.append(NexusEvent(
                    event_type=EventType.HYPOTHESIS_CREATED,
                    case_id=event.case_id,
                    payload={
                        "hypothesis_id": hyp["id"],
                        "title": hyp.get("title", ""),
                        "score": hyp.get("current_score", 50.0),
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                ))

            logger.info(
                "HypothesisWorker: generated %d hypotheses for case %s",
                len(created), event.case_id,
            )
        else:
            # Re-evaluate all active hypotheses
            logger.info(
                "HypothesisWorker: re-evaluating %d hypotheses for case %s",
                len(existing), event.case_id,
            )
            snapshots = await engine.evaluate_all(event.case_id)

            for snap in snapshots:
                output.append(NexusEvent(
                    event_type=EventType.HYPOTHESIS_SCORED,
                    case_id=event.case_id,
                    payload={
                        "hypothesis_id": snap.get("hypothesis_id", ""),
                        "previous_score": snap.get("previous_score", 0),
                        "new_score": snap.get("score", 0),
                        "delta": snap.get("delta", 0),
                    },
                    source_worker=self.name,
                    parent_event_id=event.event_id,
                ))

            logger.info(
                "HypothesisWorker: re-evaluated %d hypotheses for case %s",
                len(snapshots), event.case_id,
            )

        return output
