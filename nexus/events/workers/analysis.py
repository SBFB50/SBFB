"""
NEXUS -- AnalysisPipelineWorker.

Subscribes to EVIDENCE_CHUNKED.  Implements a 10-second debounce
to collect multiple chunk events before running a single incremental
analysis.  Emits ANALYSIS_COMPLETED when the analysis pipeline finishes.
"""

from __future__ import annotations

import asyncio
import logging
import time
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

_DEBOUNCE_SECONDS = 10.0


class AnalysisPipelineWorker(ReactiveWorker):
    """Runs incremental analysis with debounce on chunked evidence."""

    name = "analysis_pipeline"
    subscriptions = [EventType.EVIDENCE_CHUNKED]

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
        self._pipeline = None

        # Debounce state
        self._pending_evidence_ids: list[str] = []
        self._last_event_time: float = 0.0
        self._debounce_task: asyncio.Task | None = None

    def _get_pipeline(self):
        if self._pipeline is None:
            from nexus.core.analysis_pipeline import AnalysisPipeline
            self._pipeline = AnalysisPipeline(
                self._db, self._router, self._chroma, self._neo4j,
            )
        return self._pipeline

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id", "")
        if evidence_id and evidence_id not in self._pending_evidence_ids:
            self._pending_evidence_ids.append(evidence_id)

        self._last_event_time = time.monotonic()

        # Cancel any existing debounce timer
        if self._debounce_task and not self._debounce_task.done():
            self._debounce_task.cancel()

        # Start a new debounce timer with error tracking callback
        self._debounce_task = asyncio.create_task(
            self._debounced_analysis(event.case_id, event.event_id)
        )
        self._debounce_task.add_done_callback(self._on_debounce_done)

        # Return empty here; the debounce task will publish directly
        return []

    async def _debounced_analysis(
        self, case_id: str, parent_event_id: str
    ) -> None:
        """Wait for the debounce window then run analysis."""
        try:
            await asyncio.sleep(_DEBOUNCE_SECONDS)

            # Check if more events arrived during the sleep
            elapsed = time.monotonic() - self._last_event_time
            if elapsed < _DEBOUNCE_SECONDS:
                # Another event arrived, let the next debounce handle it
                return

            evidence_ids = list(self._pending_evidence_ids)
            self._pending_evidence_ids.clear()

            if not evidence_ids:
                return

            logger.info(
                "AnalysisPipeline: running incremental analysis for %d evidence items",
                len(evidence_ids),
            )

            pipeline = self._get_pipeline()
            # Use the first evidence ID as focus
            run = await pipeline.run_incremental_analysis(
                case_id=case_id,
                trigger="reactive_pipeline",
                new_evidence_id=evidence_ids[0] if len(evidence_ids) == 1 else None,
            )

            output = NexusEvent(
                event_type=EventType.ANALYSIS_COMPLETED,
                case_id=case_id,
                payload={
                    "run_id": run.id if hasattr(run, "id") else str(run),
                    "evidence_ids": evidence_ids,
                    "status": getattr(run, "status", "completed"),
                },
                source_worker=self.name,
                parent_event_id=parent_event_id,
            )
            await self._bus.publish(output)

        except asyncio.CancelledError:
            # Debounce was reset by a newer event -- expected
            pass
        except Exception as exc:
            self._last_error = f"debounced_analysis failed: {exc}"
            self._events_errored += 1
            logger.exception("AnalysisPipeline: debounced analysis failed")

    def _on_debounce_done(self, task: asyncio.Task) -> None:
        """Done-callback for the debounce task.

        Catches exceptions that propagated out of _debounced_analysis
        (should not happen given the try/except there, but acts as a
        safety net for truly unexpected errors like KeyboardInterrupt
        subclasses or framework bugs).
        """
        if task.cancelled():
            return
        exc = task.exception()
        if exc is not None:
            self._last_error = f"debounce task exception: {exc!r}"
            self._events_errored += 1
            logger.error(
                "AnalysisPipeline: debounce task died with unhandled exception: %r",
                exc,
            )
