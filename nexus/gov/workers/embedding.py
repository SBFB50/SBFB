"""NEXUS GOV -- Embedding Worker. Vectorizes all political text for RAG."""
from __future__ import annotations

from typing import Any

from loguru import logger

from nexus.engine import ReactiveWorker, NexusEvent
from nexus.gov.events import GovEventType

GOV_COLLECTION = "gov_corpus"


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
        doc_id = ""
        metadata = {}

        if etype == GovEventType.GOV_POSITION_ADDED:
            pos_id = payload.get("position_id", "")
            if not pos_id:
                return []
            doc_id = f"pos_{pos_id}"
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
            doc_id = f"social_{post_id}"
            text = payload.get("content", "")
            metadata = {
                "type": "social",
                "platform": payload.get("platform", ""),
                "politician_id": payload.get("politician_id", ""),
            }

        elif etype == GovEventType.GOV_TRANSCRIPTION_READY:
            trans_id = payload.get("transcription_id", "")
            doc_id = f"trans_{trans_id}"
            # Get first 2000 chars of transcription
            transcriptions = await self._db.list_transcriptions_by_politician(
                payload.get("politician_id", ""), limit=10
            )
            for t in transcriptions:
                if t.get("id") == trans_id:
                    text = (t.get("transcription", ""))[:2000]
                    break
            metadata = {
                "type": "transcription",
                "politician_id": payload.get("politician_id", ""),
                "title": payload.get("title", ""),
            }

        elif etype == GovEventType.GOV_PRESS_ADDED:
            article_id = payload.get("article_id", "")
            doc_id = f"press_{article_id}"
            text = payload.get("title", "")
            metadata = {"type": "press"}

        if not text or not doc_id:
            return []

        # Generate embedding via Ollama
        try:
            if self._router:
                from nexus.engine import TaskType
                embedding = await self._router.route_embedding(text[:1000])
            else:
                embedding = None

            if embedding:
                collection.upsert(
                    ids=[doc_id],
                    documents=[text[:1000]],
                    embeddings=[embedding],
                    metadatas=[metadata],
                )
            else:
                # Store without embedding (text-only for later)
                collection.upsert(
                    ids=[doc_id],
                    documents=[text[:1000]],
                    metadatas=[metadata],
                )

            logger.debug("Embedded gov doc: {}", doc_id[:50])
        except Exception as exc:
            logger.debug("Embed failed for {}: {}", doc_id[:30], exc)

        return []
