"""
NEXUS -- ChunkerEmbedWorker.

Subscribes to EVIDENCE_PROCESSED.  Chunks the evidence text via
TextChunker and indexes chunks into ChromaDB via EmbeddingStore.
Emits EVIDENCE_CHUNKED when indexing is complete.
"""

from __future__ import annotations

import logging
from typing import Any

from nexus.events.bus import EventBus
from nexus.events.types import EventType, NexusEvent
from nexus.events.worker import ReactiveWorker

logger = logging.getLogger(__name__)


class ChunkerEmbedWorker(ReactiveWorker):
    """Chunks evidence text and indexes embeddings for RAG retrieval."""

    name = "chunker_embed"
    subscriptions = [EventType.EVIDENCE_PROCESSED]

    def __init__(
        self,
        bus: EventBus,
        db: Any,
        chroma: Any,
        router: Any,
    ) -> None:
        super().__init__(bus)
        self._db = db
        self._chroma = chroma
        self._router = router

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        evidence = await self._db.get_evidence(evidence_id)
        if not evidence:
            logger.warning("ChunkerEmbed: evidence %s not found", evidence_id)
            return []

        if not self._chroma:
            logger.debug("ChunkerEmbed: no ChromaDB client, skipping")
            return []

        from nexus.config import settings
        from nexus.core.chunker import TextChunker
        from nexus.core.embedding_store import EmbeddingStore

        chunker = TextChunker(
            chunk_size=settings.rag_chunk_size,
            overlap=settings.rag_chunk_overlap,
        )
        chunks = chunker.chunk_evidence(evidence)

        if not chunks:
            logger.debug("ChunkerEmbed: no text to chunk for evidence %s", evidence_id)
            return []

        store = EmbeddingStore(self._chroma, self._router)
        n_indexed = await store.index_evidence(evidence, chunks)

        logger.info(
            "ChunkerEmbed: indexed %d chunks for evidence %s",
            n_indexed, evidence_id,
        )

        return [NexusEvent(
            event_type=EventType.EVIDENCE_CHUNKED,
            case_id=event.case_id,
            payload={
                "evidence_id": evidence_id,
                "chunk_count": n_indexed,
            },
            source_worker=self.name,
            parent_event_id=event.event_id,
        )]
