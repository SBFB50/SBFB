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
        self._processed_evidence: set[str] = set()

    def _chunks_exist_in_chroma(self, evidence_id: str) -> bool:
        """Check if chunks are already indexed in ChromaDB for this evidence.

        Lightweight check: queries by evidence_id metadata, returns only IDs,
        limited to 1 result.  No embedding computation needed.
        """
        try:
            col = self._chroma._collections.get("evidence_chunks")
            if col is None:
                return False
            result = col.get(
                where={"evidence_id": evidence_id},
                include=[],
                limit=1,
            )
            return bool(result and result.get("ids"))
        except Exception:
            # On error, assume not indexed — let the normal path handle it
            return False

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        evidence_id = event.payload.get("evidence_id")
        if not evidence_id:
            return []

        # Idempotency guard: skip if already processed in this worker session
        if evidence_id in self._processed_evidence:
            logger.debug(
                "Chunks already indexed for evidence %s, skipping",
                evidence_id,
            )
            return [NexusEvent(
                event_type=EventType.EVIDENCE_CHUNKED,
                case_id=event.case_id,
                payload={
                    "evidence_id": evidence_id,
                    "chunk_count": 0,
                    "skipped": True,
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            )]

        evidence = await self._db.get_evidence(evidence_id)
        if not evidence:
            logger.warning("ChunkerEmbed: evidence %s not found", evidence_id)
            return []

        if not self._chroma:
            logger.debug("ChunkerEmbed: no ChromaDB client, skipping")
            return []

        # Idempotency guard: check ChromaDB for existing chunks
        if self._chunks_exist_in_chroma(evidence_id):
            self._processed_evidence.add(evidence_id)
            logger.debug(
                "Chunks already indexed for evidence %s, skipping",
                evidence_id,
            )
            return [NexusEvent(
                event_type=EventType.EVIDENCE_CHUNKED,
                case_id=event.case_id,
                payload={
                    "evidence_id": evidence_id,
                    "chunk_count": 0,
                    "skipped": True,
                },
                source_worker=self.name,
                parent_event_id=event.event_id,
            )]

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

        self._processed_evidence.add(evidence_id)

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
