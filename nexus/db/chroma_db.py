"""
NEXUS -- ChromaDB vector store client.

Manages collections for semantic search across investigation data:
- evidence_chunks     : chunked evidence embeddings (primary RAG source)
- entity_contexts     : entity description + context embeddings
- monitoring_results  : monitoring hits (used for deduplication)
- hypothesis_reasoning: hypothesis snapshots for semantic retrieval
- evidence_texts      : DEPRECATED -- superseded by evidence_chunks

Image collections (managed by ImageSearchEngine):
- image_dinov2        : DINOv2 embeddings for image-to-image similarity
- image_clip          : CLIP embeddings for text-to-image search

Embeddings are pre-computed by Ollama (nomic-embed-text) via the LLMRouter.
ChromaDB is used purely as a vector store — no internal embedding function.
Runs against a ChromaDB Docker server over HTTP.
"""

from __future__ import annotations

import warnings
from itertools import combinations
from typing import Any, Dict, List, Optional, Tuple

import chromadb
from chromadb.api.models.Collection import Collection
from chromadb.errors import ChromaError
from loguru import logger

from nexus.config import settings

# ---------------------------------------------------------------------------
# Collection names (constants)
# ---------------------------------------------------------------------------
_EVIDENCE_CHUNKS_COLLECTION = "evidence_chunks"
_ENTITY_COLLECTION = "entity_contexts"
_MONITORING_COLLECTION = "monitoring_results"
_HYPOTHESIS_COLLECTION = "hypothesis_reasoning"
_CASE_MEMORY_COLLECTION = "case_memory"

# DEPRECATED: replaced by evidence_chunks (managed via EmbeddingStore)
_EVIDENCE_COLLECTION_LEGACY = "evidence_texts"

_ALL_COLLECTIONS = (
    _EVIDENCE_CHUNKS_COLLECTION,
    _ENTITY_COLLECTION,
    _MONITORING_COLLECTION,
    _HYPOTHESIS_COLLECTION,
    _CASE_MEMORY_COLLECTION,
)

# All collections including image (for stats/diagnostics)
_ALL_COLLECTIONS_FULL = (
    *_ALL_COLLECTIONS,
    _EVIDENCE_COLLECTION_LEGACY,
    "image_dinov2",
    "image_clip",
)

# Default collections for unified cross-collection search
_DEFAULT_SEARCH_COLLECTIONS = (
    _EVIDENCE_CHUNKS_COLLECTION,
    _ENTITY_COLLECTION,
    _MONITORING_COLLECTION,
    _CASE_MEMORY_COLLECTION,
)


class ChromaClient:
    """High-level ChromaDB client for NEXUS.

    Usage::

        chroma = ChromaClient()
        chroma.init_collections()
        chroma.add_evidence(evidence_id="...", case_id="...", ...)
        results = chroma.search_evidence(case_id="...", query_embedding=[...])
        chroma.close()
    """

    def __init__(
        self,
        host: str | None = None,
        port: int | None = None,
    ) -> None:
        self._host = host or settings.chroma_host
        self._port = port or settings.chroma_port
        try:
            self._client = chromadb.HttpClient(
                host=self._host,
                port=self._port,
            )
            logger.info(
                "ChromaClient initialised (host={}:{})", self._host, self._port
            )
        except Exception as exc:
            logger.error(
                "Failed to connect to ChromaDB at {}:{} — {}",
                self._host,
                self._port,
                exc,
            )
            raise

        # Lazily populated by init_collections()
        self._collections: Dict[str, Collection] = {}

    # ------------------------------------------------------------------
    # Lifecycle
    # ------------------------------------------------------------------

    def init_collections(self) -> None:
        """Create or retrieve the core NEXUS collections.

        Uses ``get_or_create_collection`` so it is safe to call repeatedly.
        ``embedding_function=None`` because we supply pre-computed vectors.

        Note: evidence_texts is deprecated. evidence_chunks (managed by
        EmbeddingStore) is the primary evidence collection for RAG.
        """
        for name in _ALL_COLLECTIONS:
            try:
                col = self._client.get_or_create_collection(
                    name=name,
                    embedding_function=None,
                    metadata={"hnsw:space": "cosine"},
                )
                self._collections[name] = col
                logger.debug(
                    "Collection '{}' ready ({} items)", name, col.count()
                )
            except Exception as exc:
                logger.error("Failed to init collection '{}': {}", name, exc)
                raise

        # Also register the legacy collection if it already exists (read-only)
        try:
            legacy = self._client.get_or_create_collection(
                name=_EVIDENCE_COLLECTION_LEGACY,
                embedding_function=None,
                metadata={"hnsw:space": "cosine"},
            )
            self._collections[_EVIDENCE_COLLECTION_LEGACY] = legacy
            if legacy.count() > 0:
                logger.warning(
                    "Legacy collection '{}' still has {} items — "
                    "consider migrating to '{}'",
                    _EVIDENCE_COLLECTION_LEGACY,
                    legacy.count(),
                    _EVIDENCE_CHUNKS_COLLECTION,
                )
        except Exception:
            pass  # not critical

        logger.info(
            "All {} ChromaDB collections initialised", len(self._collections)
        )

    def close(self) -> None:
        """Release resources.  HttpClient has no persistent connection to
        close, but we clear internal references for cleanliness."""
        self._collections.clear()
        logger.info("ChromaClient closed")

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _col(self, name: str) -> Collection:
        """Get a collection handle, raising if not yet initialised."""
        if name not in self._collections:
            raise RuntimeError(
                f"Collection '{name}' not initialised. "
                f"Call init_collections() first."
            )
        return self._collections[name]

    @staticmethod
    def _format_results(raw: dict, single_query: bool = True) -> List[Dict[str, Any]]:
        """Convert ChromaDB query results into a flat list of dicts.

        ChromaDB returns nested lists (one per query vector).  For our
        single-query use case we unpack the first (and only) sub-list.

        Returns [{id, text, distance, metadata}, ...].
        """
        if not raw or not raw.get("ids"):
            return []

        ids = raw["ids"][0] if single_query else raw["ids"]
        documents = (raw.get("documents") or [[]])[0] if single_query else (raw.get("documents") or [])
        distances = (raw.get("distances") or [[]])[0] if single_query else (raw.get("distances") or [])
        metadatas = (raw.get("metadatas") or [[]])[0] if single_query else (raw.get("metadatas") or [])

        results: List[Dict[str, Any]] = []
        for i, doc_id in enumerate(ids):
            results.append(
                {
                    "id": doc_id,
                    "text": documents[i] if i < len(documents) else None,
                    "distance": distances[i] if i < len(distances) else None,
                    "metadata": metadatas[i] if i < len(metadatas) else None,
                }
            )
        return results

    # ==================================================================
    # Evidence collection (DEPRECATED — use EmbeddingStore for RAG)
    # ==================================================================

    def add_evidence(
        self,
        evidence_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store an evidence embedding with its full text.

        .. deprecated::
            Use ``EmbeddingStore.index_evidence()`` instead, which writes
            to the ``evidence_chunks`` collection with proper chunking.
        """
        warnings.warn(
            "ChromaClient.add_evidence() is deprecated. "
            "Use EmbeddingStore.index_evidence() for the evidence_chunks collection.",
            DeprecationWarning,
            stacklevel=2,
        )
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_EVIDENCE_COLLECTION_LEGACY).add(
                ids=[evidence_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug("Added evidence '{}' to ChromaDB (legacy)", evidence_id)
        except ChromaError as exc:
            logger.error("Failed to add evidence '{}': {}", evidence_id, exc)
            raise

    def search_evidence(
        self,
        case_id: str,
        query_embedding: list[float],
        n_results: int = 10,
    ) -> List[Dict[str, Any]]:
        """Semantic search over evidence for a given case.

        Searches the ``evidence_chunks`` collection first. Falls back to
        the legacy ``evidence_texts`` collection if evidence_chunks has no
        data for the given case.

        Returns [{id, text, distance, metadata}, ...] sorted by relevance.
        """
        # Prefer evidence_chunks (the modern RAG collection)
        chunks_col = self._collections.get(_EVIDENCE_CHUNKS_COLLECTION)
        if chunks_col is not None:
            try:
                raw = chunks_col.query(
                    query_embeddings=[query_embedding],
                    n_results=n_results,
                    where={"case_id": case_id},
                    include=["documents", "distances", "metadatas"],
                )
                results = self._format_results(raw)
                if results:
                    return results
            except ChromaError:
                pass  # fall through to legacy

        # Fallback: legacy evidence_texts collection
        try:
            raw = self._col(_EVIDENCE_COLLECTION_LEGACY).query(
                query_embeddings=[query_embedding],
                n_results=n_results,
                where={"case_id": case_id},
                include=["documents", "distances", "metadatas"],
            )
            return self._format_results(raw)
        except ChromaError as exc:
            logger.error("Evidence search failed (case={}): {}", case_id, exc)
            raise

    def find_similar_evidence(
        self,
        evidence_id: str,
        case_id: str,
        n_results: int = 5,
    ) -> List[Dict[str, Any]]:
        """Find evidence items similar to an existing one.

        Retrieves the target evidence's embedding, then queries for
        neighbours in the same case (excluding itself).

        Searches evidence_chunks first, falls back to evidence_texts.
        """
        # Try evidence_chunks first
        for col_name in (_EVIDENCE_CHUNKS_COLLECTION, _EVIDENCE_COLLECTION_LEGACY):
            col = self._collections.get(col_name)
            if col is None:
                continue
            try:
                source = col.get(
                    ids=[evidence_id],
                    include=["embeddings"],
                )
                if not source["ids"] or not source["embeddings"]:
                    # Try chunk IDs (evidence_id_chunk_0, etc.)
                    if col_name == _EVIDENCE_CHUNKS_COLLECTION:
                        chunk0_id = f"{evidence_id}_chunk_0"
                        source = col.get(
                            ids=[chunk0_id],
                            include=["embeddings"],
                        )
                        if not source["ids"] or not source["embeddings"]:
                            continue
                    else:
                        continue

                source_embedding = source["embeddings"][0]
                raw = col.query(
                    query_embeddings=[source_embedding],
                    n_results=n_results + 1,
                    where={"case_id": case_id},
                    include=["documents", "distances", "metadatas"],
                )
                results = self._format_results(raw)
                return [r for r in results if r["id"] != evidence_id][:n_results]
            except ChromaError:
                continue

        logger.warning("Evidence '{}' not found in any collection", evidence_id)
        return []

    def find_duplicates(
        self,
        case_id: str,
        threshold: float = 0.92,
    ) -> List[Tuple[str, str, float]]:
        """Detect near-duplicate evidence pairs within a case.

        Returns a list of (id_a, id_b, similarity) tuples where
        similarity >= *threshold*.  Similarity is ``1 - cosine_distance``.

        Warning: O(n^2) for n evidence items — intended for moderate
        collection sizes per case.

        Searches evidence_chunks collection.
        """
        col = self._collections.get(_EVIDENCE_CHUNKS_COLLECTION)
        if col is None:
            col = self._col(_EVIDENCE_COLLECTION_LEGACY)
        try:
            data = col.get(
                where={"case_id": case_id},
                include=["embeddings"],
            )
        except ChromaError as exc:
            logger.error("find_duplicates get failed (case={}): {}", case_id, exc)
            raise

        ids: list[str] = data["ids"] or []
        embeddings: list[list[float]] = data["embeddings"] or []
        if len(ids) < 2:
            return []

        duplicates: List[Tuple[str, str, float]] = []
        for (i, id_a), (j, id_b) in combinations(enumerate(ids), 2):
            vec_a = embeddings[i]
            vec_b = embeddings[j]
            similarity = self._cosine_similarity(vec_a, vec_b)
            if similarity >= threshold:
                duplicates.append((id_a, id_b, round(similarity, 4)))

        duplicates.sort(key=lambda t: t[2], reverse=True)
        logger.debug(
            "find_duplicates: {} pairs above threshold {} in case {}",
            len(duplicates),
            threshold,
            case_id,
        )
        return duplicates

    def delete_evidence(self, evidence_id: str) -> None:
        """Remove a single evidence item from the vector store.

        Deletes from both evidence_chunks and legacy evidence_texts.
        """
        for col_name in (_EVIDENCE_CHUNKS_COLLECTION, _EVIDENCE_COLLECTION_LEGACY):
            col = self._collections.get(col_name)
            if col is None:
                continue
            try:
                col.delete(ids=[evidence_id])
            except ChromaError:
                pass
        logger.debug("Deleted evidence '{}' from ChromaDB", evidence_id)

    # ==================================================================
    # Entity collection
    # ==================================================================

    def add_entity(
        self,
        entity_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store an entity embedding (description + context)."""
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_ENTITY_COLLECTION).add(
                ids=[entity_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug("Added entity '{}' to ChromaDB", entity_id)
        except ChromaError as exc:
            logger.error("Failed to add entity '{}': {}", entity_id, exc)
            raise

    def search_entities(
        self,
        case_id: str,
        query_embedding: list[float],
        n_results: int = 10,
    ) -> List[Dict[str, Any]]:
        """Semantic search over entities for a given case."""
        try:
            raw = self._col(_ENTITY_COLLECTION).query(
                query_embeddings=[query_embedding],
                n_results=n_results,
                where={"case_id": case_id},
                include=["documents", "distances", "metadatas"],
            )
            return self._format_results(raw)
        except ChromaError as exc:
            logger.error("Entity search failed (case={}): {}", case_id, exc)
            raise

    # ==================================================================
    # Case memory collection
    # ==================================================================

    def add_memory(
        self,
        memory_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store an investigation memory embedding."""
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_CASE_MEMORY_COLLECTION).add(
                ids=[memory_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug("Added memory '{}' to ChromaDB", memory_id)
        except ChromaError as exc:
            logger.error("Failed to add memory '{}': {}", memory_id, exc)
            raise

    def delete_entity(self, entity_id: str) -> None:
        """Remove a single entity from the vector store."""
        try:
            self._col(_ENTITY_COLLECTION).delete(ids=[entity_id])
            logger.debug("Deleted entity '{}' from ChromaDB", entity_id)
        except ChromaError as exc:
            logger.error("Failed to delete entity '{}': {}", entity_id, exc)
            raise

    # ==================================================================
    # Monitoring collection
    # ==================================================================

    def add_monitoring_result(
        self,
        result_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store a monitoring result embedding for deduplication."""
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_MONITORING_COLLECTION).add(
                ids=[result_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug("Added monitoring result '{}' to ChromaDB", result_id)
        except ChromaError as exc:
            logger.error(
                "Failed to add monitoring result '{}': {}", result_id, exc
            )
            raise

    def is_duplicate_result(
        self,
        case_id: str,
        embedding: list[float],
        threshold: float = 0.92,
    ) -> bool:
        """Check if a semantically similar monitoring result already exists.

        Uses cosine distance: similarity = 1 - distance.
        Returns True if any existing result has similarity >= threshold.
        """
        col = self._col(_MONITORING_COLLECTION)
        try:
            # Check whether the collection has any data for this case first
            existing = col.get(
                where={"case_id": case_id},
                include=[],
                limit=1,
            )
            if not existing["ids"]:
                return False

            raw = col.query(
                query_embeddings=[embedding],
                n_results=1,
                where={"case_id": case_id},
                include=["distances"],
            )
            if not raw["ids"] or not raw["ids"][0]:
                return False

            # ChromaDB cosine distance: 0 = identical, 2 = opposite
            closest_distance = raw["distances"][0][0]
            similarity = 1.0 - closest_distance
            return similarity >= threshold
        except ChromaError as exc:
            logger.error(
                "Duplicate check failed (case={}): {}", case_id, exc
            )
            raise

    def delete_monitoring_results(self, job_id: str) -> None:
        """Remove all monitoring results associated with a given job.

        Filters by ``job_id`` stored in metadata.
        """
        col = self._col(_MONITORING_COLLECTION)
        try:
            # Retrieve ids matching the job_id, then delete by ids.
            matches = col.get(
                where={"job_id": job_id},
                include=[],
            )
            ids_to_delete = matches["ids"] or []
            if ids_to_delete:
                col.delete(ids=ids_to_delete)
                logger.debug(
                    "Deleted {} monitoring results for job '{}'",
                    len(ids_to_delete),
                    job_id,
                )
            else:
                logger.debug(
                    "No monitoring results found for job '{}'", job_id
                )
        except ChromaError as exc:
            logger.error(
                "Failed to delete monitoring results (job={}): {}",
                job_id,
                exc,
            )
            raise

    # ==================================================================
    # Hypothesis collection
    # ==================================================================

    def add_hypothesis_snapshot(
        self,
        snapshot_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store a hypothesis snapshot embedding for semantic search."""
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_HYPOTHESIS_COLLECTION).add(
                ids=[snapshot_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug(
                "Added hypothesis snapshot '{}' to ChromaDB", snapshot_id
            )
        except ChromaError as exc:
            logger.error(
                "Failed to add hypothesis snapshot '{}': {}", snapshot_id, exc
            )
            raise

    def search_hypotheses(
        self,
        case_id: str,
        query_embedding: list[float],
        n_results: int = 10,
    ) -> List[Dict[str, Any]]:
        """Semantic search over hypothesis snapshots for a given case."""
        try:
            raw = self._col(_HYPOTHESIS_COLLECTION).query(
                query_embeddings=[query_embedding],
                n_results=n_results,
                where={"case_id": case_id},
                include=["documents", "distances", "metadatas"],
            )
            return self._format_results(raw)
        except ChromaError as exc:
            logger.error(
                "Hypothesis search failed (case={}): {}", case_id, exc
            )
            raise

    # ==================================================================
    # Unified cross-collection search
    # ==================================================================

    async def unified_search(
        self,
        query_embedding: list[float],
        case_id: str,
        collections: list[str] | None = None,
        n_per_collection: int = 5,
    ) -> List[Dict[str, Any]]:
        """Search across multiple collections and merge results.

        Default: searches evidence_chunks + entity_contexts + monitoring_results.
        Results are sorted by cosine distance (ascending = most similar).

        Args:
            query_embedding: Pre-computed embedding vector for the query.
            case_id: Filter results to this case.
            collections: List of collection names to search. Defaults to
                evidence_chunks, entity_contexts, monitoring_results.
            n_per_collection: Max results per collection.

        Returns:
            Merged list of dicts sorted by distance, each containing:
            collection, id, text, distance, metadata.
        """
        if collections is None:
            collections = list(_DEFAULT_SEARCH_COLLECTIONS)

        all_results: List[Dict[str, Any]] = []

        for col_name in collections:
            try:
                col = self._client.get_or_create_collection(
                    name=col_name,
                    embedding_function=None,
                    metadata={"hnsw:space": "cosine"},
                )
            except Exception as exc:
                logger.warning(
                    "unified_search: could not access collection '{}': {}",
                    col_name,
                    exc,
                )
                continue

            # Skip empty collections
            if col.count() == 0:
                continue

            try:
                results = col.query(
                    query_embeddings=[query_embedding],
                    n_results=n_per_collection,
                    where={"case_id": case_id},
                    include=["documents", "metadatas", "distances"],
                )
            except ChromaError as exc:
                logger.warning(
                    "unified_search: query failed on '{}' (case={}): {}",
                    col_name,
                    case_id,
                    exc,
                )
                continue

            if not results or not results.get("ids") or not results["ids"][0]:
                continue

            for i in range(len(results["ids"][0])):
                all_results.append({
                    "collection": col_name,
                    "id": results["ids"][0][i],
                    "text": (results.get("documents") or [[]])[0][i]
                    if results.get("documents") and results["documents"][0]
                    else None,
                    "distance": (results.get("distances") or [[]])[0][i]
                    if results.get("distances") and results["distances"][0]
                    else None,
                    "metadata": (results.get("metadatas") or [[]])[0][i]
                    if results.get("metadatas") and results["metadatas"][0]
                    else {},
                })

        # Sort by distance (ascending = most similar for cosine)
        all_results.sort(key=lambda x: x.get("distance") or float("inf"))

        logger.debug(
            "unified_search: {} total results across {} collections for case {}",
            len(all_results),
            len(collections),
            case_id,
        )
        return all_results

    # ==================================================================
    # Batch re-embedding
    # ==================================================================

    async def reindex_case(self, case_id: str, router: Any) -> int:
        """Re-embed all evidence for a case.

        Fetches all evidence from SQLite, re-chunks and re-embeds each one,
        and upserts into the evidence_chunks collection. Useful after an
        embedding model change or parameter tuning.

        Args:
            case_id: The case whose evidence should be re-embedded.
            router: An LLMRouter instance for generating embeddings.

        Returns:
            Total number of chunks re-indexed.
        """
        # Import here to avoid circular imports
        from nexus.core.chunker import TextChunker
        from nexus.core.embedding_store import EmbeddingStore
        from nexus.db.sqlite_db import Database, get_db

        total_chunks = 0

        async with get_db() as conn:
            db = Database(conn)
            evidence_list = await db.list_evidence_by_case(case_id)

        if not evidence_list:
            logger.info("reindex_case: no evidence found for case {}", case_id)
            return 0

        logger.info(
            "reindex_case: re-indexing {} evidence items for case {}",
            len(evidence_list),
            case_id,
        )

        store = EmbeddingStore(self, router)
        chunker = TextChunker()

        for evidence in evidence_list:
            try:
                chunks = chunker.chunk_evidence(evidence)
                if chunks:
                    n = await store.index_evidence(evidence, chunks)
                    total_chunks += n
            except Exception as exc:
                logger.warning(
                    "reindex_case: failed for evidence {}: {}",
                    evidence.get("id", "?"),
                    exc,
                )

        logger.info(
            "reindex_case: completed for case {} — {} chunks indexed from {} evidence items",
            case_id,
            total_chunks,
            len(evidence_list),
        )
        return total_chunks

    # ==================================================================
    # Utilities
    # ==================================================================

    def get_collection_stats(self) -> Dict[str, int]:
        """Return the number of items in each core collection.

        Returns a dict like ``{"evidence_chunks": 142, ...}``.
        """
        stats: Dict[str, int] = {}
        for name in _ALL_COLLECTIONS:
            try:
                col = self._collections.get(name)
                stats[name] = col.count() if col else 0
            except Exception as exc:
                logger.warning(
                    "Could not count collection '{}': {}", name, exc
                )
                stats[name] = -1
        return stats

    def get_detailed_stats(self) -> Dict[str, Dict[str, Any]]:
        """Return detailed stats for all collections (including image and legacy).

        Returns a dict keyed by collection name, each containing:
        - count: number of items
        - error: error message if collection is unavailable
        - deprecated: True for legacy collections
        """
        stats: Dict[str, Dict[str, Any]] = {}
        for name in _ALL_COLLECTIONS_FULL:
            entry: Dict[str, Any] = {}
            try:
                col = self._client.get_or_create_collection(
                    name=name,
                    embedding_function=None,
                    metadata={"hnsw:space": "cosine"},
                )
                entry["count"] = col.count()
            except Exception as exc:
                entry["count"] = 0
                entry["error"] = str(exc)

            if name == _EVIDENCE_COLLECTION_LEGACY:
                entry["deprecated"] = True
                entry["replacement"] = _EVIDENCE_CHUNKS_COLLECTION

            stats[name] = entry

        # Add a summary
        total = sum(
            s.get("count", 0)
            for s in stats.values()
            if isinstance(s.get("count"), int) and s["count"] >= 0
        )
        stats["_summary"] = {
            "total_items": total,
            "collections_active": len(_ALL_COLLECTIONS),
            "collections_total": len(_ALL_COLLECTIONS_FULL),
        }
        return stats

    def clear_case_data(self, case_id: str) -> None:
        """Remove **all** data for a case across every collection.

        Includes evidence_chunks, entity_contexts, monitoring_results,
        hypothesis_reasoning, and the legacy evidence_texts.

        Does NOT delete image collections (use ImageSearchEngine.delete_case_images).
        """
        collections_to_clear = list(_ALL_COLLECTIONS) + [_EVIDENCE_COLLECTION_LEGACY]
        for name in collections_to_clear:
            col = self._collections.get(name)
            if col is None:
                continue
            try:
                matches = col.get(
                    where={"case_id": case_id},
                    include=[],
                )
                ids_to_delete = matches["ids"] or []
                if ids_to_delete:
                    col.delete(ids=ids_to_delete)
                    logger.debug(
                        "Cleared {} items from '{}' for case '{}'",
                        len(ids_to_delete),
                        name,
                        case_id,
                    )
            except ChromaError as exc:
                logger.error(
                    "Failed to clear case '{}' from '{}': {}",
                    case_id,
                    name,
                    exc,
                )
                raise
        logger.info("All ChromaDB data cleared for case '{}'", case_id)

    # ------------------------------------------------------------------
    # Math helpers
    # ------------------------------------------------------------------

    @staticmethod
    def _cosine_similarity(a: list[float], b: list[float]) -> float:
        """Compute cosine similarity between two vectors.

        Returns a value in [-1, 1] where 1 means identical direction.
        """
        dot = sum(x * y for x, y in zip(a, b))
        norm_a = sum(x * x for x in a) ** 0.5
        norm_b = sum(x * x for x in b) ** 0.5
        if norm_a == 0 or norm_b == 0:
            return 0.0
        return dot / (norm_a * norm_b)
