"""
NEXUS -- MemoryWorker.

Subscribes to high-value investigation events (hypothesis shifts,
contradictions, suspect scores, analysis results) and extracts
structured insights via a fast LLM call (gemma4:e4b).

Insights are persisted in SQLite (investigation_memory) and embedded
in ChromaDB (case_memory) for future retrieval by the RAG pipeline.
Does NOT emit downstream events -- memories are consumed passively.
"""

from __future__ import annotations

import json
import logging
import time
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)

# Debounce window: skip duplicate (case_id, event_type) within this period
_DEBOUNCE_SECONDS = 10.0

# Only store insights with importance >= this threshold
_MIN_IMPORTANCE = 0.4

_INSIGHT_PROMPT = """\
Tu es un analyste d'investigation criminelle. A partir de l'evenement suivant, \
extrais un insight structurel pour la memoire de l'enquete.

Type d'evenement: {event_type}
Contexte: {context}

Reponds UNIQUEMENT en JSON valide (pas de markdown, pas de commentaire):
{{
  "insight_type": "hypothesis_shift|contradiction_pattern|entity_connection|timeline_gap|profile_inference",
  "summary": "1-2 phrases d'insight en francais",
  "importance": 0.0-1.0,
  "confidence": 0.0-1.0,
  "related_entity_names": ["nom1", "nom2"]
}}
"""


class MemoryWorker(ReactiveWorker):
    """Extracts and stores investigation insights from significant events."""

    name = "memory_worker"
    subscriptions = [
        EventType.HYPOTHESIS_CREATED,
        EventType.HYPOTHESIS_SCORED,
        EventType.CONTRADICTION_FOUND,
        EventType.SUSPECT_SCORED,
        EventType.ANALYSIS_COMPLETED,
    ]

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
        # Debounce tracker: (case_id, event_type) -> last_processed_time
        self._last_seen: dict[tuple[str, str], float] = {}

    def _should_debounce(self, case_id: str, event_type: str) -> bool:
        """Return True if we should skip this event (too recent)."""
        key = (case_id, event_type)
        now = time.monotonic()
        last = self._last_seen.get(key, 0.0)
        if now - last < _DEBOUNCE_SECONDS:
            return True
        self._last_seen[key] = now
        return False

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        if self._should_debounce(event.case_id, event.event_type.value):
            logger.debug(
                "MemoryWorker: debounced %s for case %s",
                event.event_type.value,
                event.case_id,
            )
            return []

        # Build context string from event payload
        context = json.dumps(event.payload, ensure_ascii=False, default=str)

        # Call fast LLM for insight extraction
        from nexus.llm.router import TaskType

        prompt = _INSIGHT_PROMPT.format(
            event_type=event.event_type.value,
            context=context,
        )

        try:
            insight = await self._router.route_json(
                TaskType.EVIDENCE_SUMMARY,
                prompt,
            )
        except Exception:
            logger.exception(
                "MemoryWorker: LLM call failed for %s", event.event_type.value
            )
            return []

        # Validate required fields
        if not isinstance(insight, dict) or "summary" not in insight:
            logger.warning("MemoryWorker: invalid LLM response: %s", insight)
            return []

        importance = float(insight.get("importance", 0.5))
        if importance < _MIN_IMPORTANCE:
            logger.debug(
                "MemoryWorker: insight below threshold (%.2f < %.2f)",
                importance,
                _MIN_IMPORTANCE,
            )
            return []

        # Store in SQLite
        from nexus.db.sqlite_db import Database, get_db

        related_entities = insight.get("related_entity_names", [])

        try:
            async with get_db() as conn:
                db = Database(conn)
                memory = await db.create_investigation_memory(
                    case_id=event.case_id,
                    insight_type=insight.get("insight_type", "unknown"),
                    source_event_type=event.event_type.value,
                    importance=importance,
                    confidence=float(insight.get("confidence", 0.7)),
                    summary=insight["summary"],
                    full_context=context,
                    related_entities=related_entities,
                )
        except Exception:
            logger.exception("MemoryWorker: failed to store memory in SQLite")
            return []

        # Embed in ChromaDB
        if self._chroma is not None:
            try:
                embedding = await self._router.embed(insight["summary"])
                self._chroma.add_memory(
                    memory_id=memory["id"],
                    case_id=event.case_id,
                    text=insight["summary"],
                    embedding=embedding,
                    metadata={
                        "insight_type": insight.get("insight_type", "unknown"),
                        "importance": importance,
                        "source_event_type": event.event_type.value,
                    },
                )
            except Exception:
                logger.exception(
                    "MemoryWorker: failed to embed memory in ChromaDB"
                )
                # Non-fatal: SQLite record already saved

        logger.info(
            "MemoryWorker: stored insight (type=%s, importance=%.2f) for case %s",
            insight.get("insight_type"),
            importance,
            event.case_id,
        )

        # No downstream events -- memories are consumed passively
        return []
