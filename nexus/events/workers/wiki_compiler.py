"""
NEXUS -- WikiCompilerWorker.

Subscribes to EVIDENCE_PROCESSED, ENTITY_ENRICHED, HYPOTHESIS_SCORED.
Compiles investigation data into a live Markdown wiki.
Emits WIKI_UPDATED after each compilation.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class WikiCompilerWorker(ReactiveWorker):
    """Compiles investigation data into wiki pages."""

    name = "wiki_compiler"
    subscriptions = [
        EventType.EVIDENCE_PROCESSED,
        EventType.ENTITY_ENRICHED,
        EventType.HYPOTHESIS_CREATED,   # compile new hypotheses immediately
        EventType.HYPOTHESIS_SCORED,
        EventType.CONTRADICTION_FOUND,  # update wiki with contradictions
        EventType.SUSPECT_SCORED,       # compile suspect pages
    ]

    def __init__(self, bus: EventBus, db: Any, router: Any) -> None:
        super().__init__(bus)
        self._db = db
        self._router = router
        self._compiler = None

    def _get_compiler(self):
        if self._compiler is None:
            from nexus.core.wiki_compiler import WikiCompiler
            self._compiler = WikiCompiler(self._db, self._router)
        return self._compiler

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        compiler = self._get_compiler()
        updated_pages: list[str] = []

        try:
            if event.event_type == EventType.EVIDENCE_PROCESSED:
                evidence_id = event.payload.get("evidence_id", "")
                if evidence_id:
                    updated_pages = await compiler.compile_evidence(event.case_id, evidence_id)

            elif event.event_type == EventType.HYPOTHESIS_SCORED:
                page = await compiler.compile_hypothesis_update(event.case_id)
                if page:
                    updated_pages = [page]

            elif event.event_type == EventType.HYPOTHESIS_CREATED:
                page = await compiler.compile_hypothesis_update(event.case_id)
                if page:
                    updated_pages = [page]

            elif event.event_type in (EventType.CONTRADICTION_FOUND, EventType.SUSPECT_SCORED):
                # Recompile hypotheses page (contradictions/suspects affect analysis)
                page = await compiler.compile_hypothesis_update(event.case_id)
                if page:
                    updated_pages = [page]

            elif event.event_type == EventType.ENTITY_ENRICHED:
                entity_id = event.payload.get("entity_id", "")
                if entity_id:
                    mentions = await self._db.list_mentions_by_entity(entity_id)
                    for m in mentions[:3]:
                        pages = await compiler.compile_evidence(event.case_id, m["evidence_id"])
                        updated_pages.extend(pages)

        except Exception as exc:
            logger.error("WikiCompiler failed for case %s: %s", event.case_id, exc)
            return []

        # Cross-link all pages after compilation
        if updated_pages:
            try:
                await compiler.cross_link_pages(event.case_id)
            except Exception as exc:
                logger.debug("WikiCompiler: cross-linking failed: %s", exc)

        if not updated_pages:
            return []

        logger.info("WikiCompiler: %d pages updated for case %s", len(updated_pages), event.case_id)

        return [NexusEvent(
            event_type=EventType.WIKI_UPDATED,
            case_id=event.case_id,
            payload={"pages_updated": updated_pages, "count": len(updated_pages)},
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
