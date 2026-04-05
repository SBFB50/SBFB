"""
NEXUS -- Embedding store for chunked evidence.

Manages a ChromaDB collection of evidence chunks with their
embeddings. Supports adding, querying, and deleting chunks.

This is the RAG layer: evidence is chunked (by TextChunker), embedded
(by LLMRouter.embed_batch via nomic-embed-text), and stored here for
semantic retrieval during analysis.
"""

from __future__ import annotations

from typing import Any

from chromadb.errors import ChromaError
from loguru import logger

from nexus.db.chroma_db import ChromaClient
from nexus.llm.router import LLMRouter

_CHUNKS_COLLECTION = "evidence_chunks"


class EmbeddingStore:
    """Store and retrieve evidence chunk embeddings via ChromaDB."""

    def __init__(self, chroma_client: ChromaClient, llm_router: LLMRouter) -> None:
        self._chroma = chroma_client
        self._router = llm_router
        self._collection = None
        self._init_collection()

    def _init_collection(self) -> None:
        """Create or get the evidence_chunks collection."""
        try:
            self._collection = self._chroma._client.get_or_create_collection(
                name=_CHUNKS_COLLECTION,
                embedding_function=None,
                metadata={"hnsw:space": "cosine"},
            )
            logger.debug(
                "EmbeddingStore collection '{}' ready ({} items)",
                _CHUNKS_COLLECTION,
                self._collection.count(),
            )
        except Exception as exc:
            logger.error(
                "Failed to init EmbeddingStore collection '{}': {}",
                _CHUNKS_COLLECTION,
                exc,
            )
            raise

    # ------------------------------------------------------------------
    # Indexing
    # ------------------------------------------------------------------

    async def index_evidence(self, evidence: dict, chunks: list[dict]) -> int:
        """Embed and store all chunks of an evidence.

        Returns number of chunks indexed.
        """
        if not chunks:
            return 0

        evidence_id = evidence.get("id", "unknown")

        # Embed all chunks in batch via nomic-embed-text
        texts = [c["text"] for c in chunks]
        try:
            embeddings = await self._router.embed_batch(texts)
        except Exception as exc:
            logger.error(
                "Embedding failed for evidence {} ({} chunks): {}",
                evidence_id,
                len(texts),
                exc,
            )
            return 0

        # Build IDs and metadata for ChromaDB
        ids = [f"{evidence_id}_chunk_{i}" for i in range(len(chunks))]
        metadatas = []
        for c in chunks:
            meta = dict(c.get("metadata", {}))
            # Ensure all metadata values are ChromaDB-compatible (str, int, float, bool)
            clean_meta = {}
            for k, v in meta.items():
                if v is None:
                    clean_meta[k] = ""
                elif isinstance(v, (str, int, float, bool)):
                    clean_meta[k] = v
                else:
                    clean_meta[k] = str(v)
            metadatas.append(clean_meta)

        # Store in ChromaDB (upsert so re-indexing is safe)
        try:
            self._collection.upsert(
                ids=ids,
                embeddings=embeddings,
                documents=texts,
                metadatas=metadatas,
            )
            logger.info(
                "Indexed {} chunks for evidence {}", len(chunks), evidence_id
            )
        except ChromaError as exc:
            logger.error(
                "ChromaDB upsert failed for evidence {}: {}", evidence_id, exc
            )
            return 0

        return len(chunks)

    # ------------------------------------------------------------------
    # Search
    # ------------------------------------------------------------------

    async def search(
        self,
        query: str,
        case_id: str,
        n_results: int = 20,
        evidence_type: str | None = None,
        min_reliability: int = 0,
    ) -> list[dict]:
        """Semantic search over evidence chunks.

        Returns [{chunk_text, evidence_id, title, source, distance, metadata}]
        """
        try:
            query_embedding = await self._router.embed(query)
        except Exception as exc:
            logger.error("Failed to embed query: {}", exc)
            return []

        # Build the where filter
        where: dict[str, Any] = {"case_id": case_id}
        if evidence_type:
            where["evidence_type"] = evidence_type

        # If we have multiple conditions, ChromaDB needs $and
        if len(where) > 1:
            where = {"$and": [{k: v} for k, v in where.items()]}

        try:
            results = self._collection.query(
                query_embeddings=[query_embedding],
                n_results=n_results,
                where=where,
                include=["documents", "metadatas", "distances"],
            )
        except ChromaError as exc:
            logger.error("Chunk search failed (case={}): {}", case_id, exc)
            return []

        if not results or not results.get("ids") or not results["ids"][0]:
            return []

        # Format results, applying min_reliability post-filter
        formatted: list[dict] = []
        for i in range(len(results["ids"][0])):
            meta = results["metadatas"][0][i] if results.get("metadatas") else {}
            reliability = meta.get("reliability", 100)
            if isinstance(reliability, str):
                try:
                    reliability = int(reliability)
                except ValueError:
                    reliability = 100

            if reliability >= min_reliability:
                formatted.append({
                    "chunk_text": results["documents"][0][i] if results.get("documents") else "",
                    "evidence_id": meta.get("evidence_id", ""),
                    "title": meta.get("title", ""),
                    "source": meta.get("source", ""),
                    "distance": results["distances"][0][i] if results.get("distances") else 0.0,
                    "metadata": meta,
                })

        return formatted

    async def search_multi(
        self,
        queries: list[str],
        case_id: str,
        n_per_query: int = 10,
    ) -> list[dict]:
        """Search with multiple queries, deduplicate results.

        Useful for multi-faceted retrieval (e.g. different aspects
        of a question each generating a separate query).
        Results are deduplicated by chunk ID and sorted by best distance.
        """
        seen_chunks: dict[str, dict] = {}  # chunk_text -> best result

        for query in queries:
            results = await self.search(
                query=query,
                case_id=case_id,
                n_results=n_per_query,
            )
            for r in results:
                key = r["chunk_text"][:200]  # deduplicate by text prefix
                existing = seen_chunks.get(key)
                if existing is None or r["distance"] < existing["distance"]:
                    seen_chunks[key] = r

        # Sort by distance (closest first for cosine)
        deduplicated = sorted(seen_chunks.values(), key=lambda x: x["distance"])
        return deduplicated

    # ------------------------------------------------------------------
    # Deletion
    # ------------------------------------------------------------------

    def delete_evidence_chunks(self, evidence_id: str) -> None:
        """Delete all chunks for an evidence.

        Finds chunks by evidence_id in metadata and removes them.
        """
        try:
            # Query for all chunk IDs belonging to this evidence
            matches = self._collection.get(
                where={"evidence_id": evidence_id},
                include=[],
            )
            ids_to_delete = matches.get("ids", [])
            if ids_to_delete:
                self._collection.delete(ids=ids_to_delete)
                logger.info(
                    "Deleted {} chunks for evidence '{}'",
                    len(ids_to_delete),
                    evidence_id,
                )
            else:
                logger.debug(
                    "No chunks found to delete for evidence '{}'", evidence_id
                )
        except ChromaError as exc:
            logger.error(
                "Failed to delete chunks for evidence '{}': {}",
                evidence_id,
                exc,
            )

    def delete_case_chunks(self, case_id: str) -> None:
        """Delete all chunks for a case.

        Removes every chunk whose metadata contains this case_id.
        """
        try:
            matches = self._collection.get(
                where={"case_id": case_id},
                include=[],
            )
            ids_to_delete = matches.get("ids", [])
            if ids_to_delete:
                self._collection.delete(ids=ids_to_delete)
                logger.info(
                    "Deleted {} chunks for case '{}'",
                    len(ids_to_delete),
                    case_id,
                )
            else:
                logger.debug(
                    "No chunks found to delete for case '{}'", case_id
                )
        except ChromaError as exc:
            logger.error(
                "Failed to delete chunks for case '{}': {}", case_id, exc
            )

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    def get_stats(self) -> dict:
        """Return collection stats."""
        try:
            count = self._collection.count()
            return {
                "collection": _CHUNKS_COLLECTION,
                "total_chunks": count,
            }
        except Exception as exc:
            logger.error("Failed to get EmbeddingStore stats: {}", exc)
            return {
                "collection": _CHUNKS_COLLECTION,
                "total_chunks": -1,
                "error": str(exc),
            }
