"""NEXUS GOV -- Embedding Worker. Vectorizes all political text for RAG."""
from __future__ import annotations

import hashlib
from typing import Any

from loguru import logger

from nexus.engine import ReactiveWorker, NexusEvent
from nexus.gov.events import GovEventType

GOV_COLLECTION = "gov_corpus"

# Chunking constants for long texts (transcriptions)
CHUNK_SIZE = 800
CHUNK_OVERLAP = 200


class GovEmbedWorker(ReactiveWorker):
    name = "gov_embed"
    subscriptions = [
        GovEventType.GOV_POSITION_ADDED,
        GovEventType.GOV_SOCIAL_POST_ADDED,
        GovEventType.GOV_TRANSCRIPTION_READY,
        GovEventType.GOV_PRESS_ADDED,
    ]

    def __init__(self, bus: Any, db: Any, chroma: Any = None, router: Any = None) -> None:
        super().__init__(bus)
        self._db = db
        self._chroma = chroma
        self._router = router
        self._collection = None

    @staticmethod
    def _make_embed_id(source_type: str, source_id: str, chunk: int = 0) -> str:
        """Deterministic embedding ID to prevent duplicates.

        Uses MD5 hash of ``gov:{source_type}:{source_id}:{chunk}`` so that
        re-processing the same item always produces the same ID, and ChromaDB
        ``upsert`` overwrites rather than duplicating.
        """
        raw = f"gov:{source_type}:{source_id}:{chunk}"
        return hashlib.md5(raw.encode()).hexdigest()

    @staticmethod
    def _chunk_text(text: str) -> list[str]:
        """Split long text into overlapping chunks for embedding.

        Returns a list of chunks. Short texts (<= CHUNK_SIZE) are returned
        as a single-element list.
        """
        if len(text) <= CHUNK_SIZE:
            return [text]

        chunks: list[str] = []
        start = 0
        while start < len(text):
            end = start + CHUNK_SIZE
            chunks.append(text[start:end])
            start += CHUNK_SIZE - CHUNK_OVERLAP
        return chunks

    def _get_collection(self):
        if self._collection is None and self._chroma is not None:
            try:
                self._collection = self._chroma._client.get_or_create_collection(
                    name=GOV_COLLECTION,
                    metadata={"hnsw:space": "cosine"},
                )
            except Exception as exc:
                logger.warning("ChromaDB gov collection failed: {}", exc)
        return self._collection

    async def handle(self, event: NexusEvent) -> list[NexusEvent]:
        collection = self._get_collection()
        if collection is None:
            return []

        etype = event.event_type
        payload = event.payload

        text = ""
        source_type = ""
        source_id = ""
        metadata: dict[str, Any] = {}

        if etype == GovEventType.GOV_POSITION_ADDED:
            pos_id = payload.get("position_id", "")
            if not pos_id:
                return []
            source_type = "position"
            source_id = pos_id
            # Fetch position
            pos = await self._db.get_position(pos_id)
            if pos:
                text = f"{pos.get('subject', '')} — {pos.get('position_text', '')}"
                metadata = {
                    "type": "position",
                    "politician_id": pos.get("politician_id", ""),
                    "date": pos.get("date", ""),
                    "source_url": pos.get("source_url", ""),
                }

        elif etype == GovEventType.GOV_SOCIAL_POST_ADDED:
            post_id = payload.get("post_id", "")
            if not post_id:
                return []
            source_type = "social"
            source_id = post_id
            text = payload.get("content", "")
            metadata = {
                "type": "social",
                "platform": payload.get("platform", ""),
                "politician_id": payload.get("politician_id", ""),
            }

        elif etype == GovEventType.GOV_TRANSCRIPTION_READY:
            trans_id = payload.get("transcription_id", "")
            if not trans_id:
                return []
            source_type = "transcription"
            source_id = trans_id
            # Get full transcription text
            transcriptions = await self._db.list_transcriptions_by_politician(
                payload.get("politician_id", ""), limit=10
            )
            for t in transcriptions:
                if t.get("id") == trans_id:
                    text = t.get("transcription", "")
                    break
            metadata = {
                "type": "transcription",
                "politician_id": payload.get("politician_id", ""),
                "title": payload.get("title", ""),
            }

        elif etype == GovEventType.GOV_PRESS_ADDED:
            article_id = payload.get("article_id", "")
            if not article_id:
                return []
            source_type = "press"
            source_id = article_id
            title = payload.get("title", "")
            summary = payload.get("summary", "")
            text = f"{title}\n{summary}".strip() if summary else title
            metadata = {
                "type": "press",
                "politician_id": payload.get("politician_id", ""),
                "source_url": payload.get("source_url", ""),
            }

        if not text or not source_type or not source_id:
            return []

        # For long texts (transcriptions), chunk into overlapping segments
        chunks = self._chunk_text(text)

        for chunk_idx, chunk_text in enumerate(chunks):
            embed_id = self._make_embed_id(source_type, source_id, chunk_idx)
            chunk_meta = {**metadata}
            if len(chunks) > 1:
                chunk_meta["chunk"] = chunk_idx
                chunk_meta["total_chunks"] = len(chunks)

            try:
                embedding = None
                if self._router:
                    from nexus.engine import TaskType
                    embedding = await self._router.route_embedding(chunk_text[:1000])

                if embedding:
                    collection.upsert(
                        ids=[embed_id],
                        documents=[chunk_text[:1000]],
                        embeddings=[embedding],
                        metadatas=[chunk_meta],
                    )
                else:
                    # Store without embedding (text-only for later)
                    collection.upsert(
                        ids=[embed_id],
                        documents=[chunk_text[:1000]],
                        metadatas=[chunk_meta],
                    )

                logger.debug("Embedded gov doc: {} (chunk {}/{})", embed_id[:12], chunk_idx + 1, len(chunks))
            except Exception as exc:
                logger.debug("Embed failed for {} chunk {}: {}", embed_id[:12], chunk_idx, exc)

        return []
