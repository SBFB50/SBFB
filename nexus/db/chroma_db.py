"""
NEXUS -- ChromaDB vector store client.

Manages 4 collections for semantic search across investigation data:
- evidence_texts      : full-text embeddings of evidence
- entity_contexts     : entity description + context embeddings
- monitoring_results  : monitoring hits (used for deduplication)
- hypothesis_reasoning: hypothesis snapshots for semantic retrieval

Embeddings are pre-computed by Ollama (nomic-embed-text) via the LLMRouter.
ChromaDB is used purely as a vector store — no internal embedding function.
Runs against a ChromaDB Docker server over HTTP.
"""

from __future__ import annotations

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
_EVIDENCE_COLLECTION = "evidence_texts"
_ENTITY_COLLECTION = "entity_contexts"
_MONITORING_COLLECTION = "monitoring_results"
_HYPOTHESIS_COLLECTION = "hypothesis_reasoning"

_ALL_COLLECTIONS = (
    _EVIDENCE_COLLECTION,
    _ENTITY_COLLECTION,
    _MONITORING_COLLECTION,
    _HYPOTHESIS_COLLECTION,
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
        """Create or retrieve the 4 NEXUS collections.

        Uses ``get_or_create_collection`` so it is safe to call repeatedly.
        ``embedding_function=None`` because we supply pre-computed vectors.
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
    # Evidence collection
    # ==================================================================

    def add_evidence(
        self,
        evidence_id: str,
        case_id: str,
        text: str,
        embedding: list[float],
        metadata: Optional[dict] = None,
    ) -> None:
        """Store an evidence embedding with its full text."""
        meta = dict(metadata or {})
        meta["case_id"] = case_id
        try:
            self._col(_EVIDENCE_COLLECTION).add(
                ids=[evidence_id],
                documents=[text],
                embeddings=[embedding],
                metadatas=[meta],
            )
            logger.debug("Added evidence '{}' to ChromaDB", evidence_id)
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

        Returns [{id, text, distance, metadata}, ...] sorted by relevance.
        """
        try:
            raw = self._col(_EVIDENCE_COLLECTION).query(
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
        """
        col = self._col(_EVIDENCE_COLLECTION)
        try:
            # Retrieve the source evidence's embedding
            source = col.get(
                ids=[evidence_id],
                include=["embeddings"],
            )
            if not source["ids"] or not source["embeddings"]:
                logger.warning(
                    "Evidence '{}' not found in ChromaDB", evidence_id
                )
                return []

            source_embedding = source["embeddings"][0]

            # Query for similar items (request extra to account for self)
            raw = col.query(
                query_embeddings=[source_embedding],
                n_results=n_results + 1,
                where={"case_id": case_id},
                include=["documents", "distances", "metadatas"],
            )
            results = self._format_results(raw)
            # Exclude the source item itself
            return [r for r in results if r["id"] != evidence_id][:n_results]
        except ChromaError as exc:
            logger.error(
                "find_similar_evidence failed (id={}): {}", evidence_id, exc
            )
            raise

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
        """
        col = self._col(_EVIDENCE_COLLECTION)
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
        """Remove a single evidence item from the vector store."""
        try:
            self._col(_EVIDENCE_COLLECTION).delete(ids=[evidence_id])
            logger.debug("Deleted evidence '{}' from ChromaDB", evidence_id)
        except ChromaError as exc:
            logger.error(
                "Failed to delete evidence '{}': {}", evidence_id, exc
            )
            raise

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
    # Utilities
    # ==================================================================

    def get_collection_stats(self) -> Dict[str, int]:
        """Return the number of items in each collection.

        Returns a dict like ``{"evidence_texts": 142, ...}``.
        """
        stats: Dict[str, int] = {}
        for name in _ALL_COLLECTIONS:
            try:
                stats[name] = self._col(name).count()
            except Exception as exc:
                logger.warning(
                    "Could not count collection '{}': {}", name, exc
                )
                stats[name] = -1
        return stats

    def clear_case_data(self, case_id: str) -> None:
        """Remove **all** data for a case across every collection.

        Useful when deleting a case from the system entirely.
        """
        for name in _ALL_COLLECTIONS:
            col = self._col(name)
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
