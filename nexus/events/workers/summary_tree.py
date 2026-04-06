"""
NEXUS -- SummaryTreeWorker.

Subscribes to EVIDENCE_PROCESSED.  Updates the RAPTOR hierarchical
summary tree (evidence -> cluster -> case) when new evidence is
processed.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class SummaryTreeWorker(ReactiveWorker):
    """Updates the RAPTOR summary tree on new evidence."""

    name = "summary_tree"
    subscriptions = [EventType.EVIDENCE_PROCESSED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        router: Any,
        chroma: Any = None,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._chroma = chroma
        self._tree = None

    def _get_tree(self):
        if self._tree is None:
            from nexus.core.summary_tree import SummaryTree
            self._tree = SummaryTree(self._db, self._router, self._chroma)
        return self._tree

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        logger.info(
            "SummaryTree: updating tree for evidence %s in case %s",
            evidence_id[:8], event.case_id,
        )

        tree = self._get_tree()

        try:
            await tree.update_for_new_evidence(event.case_id, evidence_id)
            logger.info(
                "SummaryTree: tree updated for evidence %s", evidence_id[:8]
            )
        except Exception as exc:
            logger.warning(
                "SummaryTree: update failed for evidence %s: %s",
                evidence_id[:8], exc,
            )

        return []
