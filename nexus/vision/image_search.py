"""
NEXUS -- Image similarity search engine backed by ChromaDB.

Uses two separate ChromaDB collections:
- image_dinov2: DINOv2 embeddings (768-dim) for image-to-image similarity
- image_clip:   CLIP embeddings (512-dim) for text-to-image search

Relies on VisualEmbedder for embedding generation and ChromaClient for storage.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any, Dict, List, Optional

from loguru import logger

from nexus.db.chroma_db import ChromaClient
from nexus.vision.embeddings import VisualEmbedder

# Collection names
_DINOV2_COLLECTION = "image_dinov2"
_CLIP_COLLECTION = "image_clip"


class ImageSearchEngine:
    """Search engine for images stored in ChromaDB.

    Uses two ChromaDB collections:
    - image_dinov2: DINOv2 embeddings (768-dim) for image-to-image similarity
    - image_clip: CLIP embeddings (512-dim) for text-to-image search
    """

    def __init__(
        self,
        chroma_client: ChromaClient,
        visual_embedder: VisualEmbedder,
    ) -> None:
        self._chroma = chroma_client
        self._embedder = visual_embedder
        self._init_collections()

    def _init_collections(self) -> None:
        """Create or get the image embedding collections."""
        # DINOv2 collection (768-dim, cosine)
        self._dinov2_col = self._chroma._client.get_or_create_collection(
            name=_DINOV2_COLLECTION,
            metadata={"hnsw:space": "cosine"},
        )
        # CLIP collection (512-dim, cosine)
        self._clip_col = self._chroma._client.get_or_create_collection(
            name=_CLIP_COLLECTION,
            metadata={"hnsw:space": "cosine"},
        )
        logger.info(
            "Image search collections initialised (dinov2={}, clip={})",
            self._dinov2_col.count(),
            self._clip_col.count(),
        )

    # ------------------------------------------------------------------
    # Indexing
    # ------------------------------------------------------------------

    def index_image(
        self,
        evidence_id: str,
        case_id: str,
        image_path: str | Path,
        description: str = "",
    ) -> None:
        """Index an image in both DINOv2 and CLIP collections.

        The image is embedded by both models and upserted into each
        collection so it can be retrieved via image similarity (DINOv2)
        or text query (CLIP).
        """
        path_str = str(image_path)
        meta = {
            "case_id": case_id,
            "path": path_str,
            "description": description,
        }

        # DINOv2 embedding
        dinov2_emb = self._embedder.embed_image_dinov2(image_path)
        self._dinov2_col.upsert(
            ids=[evidence_id],
            embeddings=[dinov2_emb],
            metadatas=[meta],
        )

        # CLIP embedding (model swap happens inside embedder)
        clip_emb = self._embedder.embed_image_clip(image_path)
        self._clip_col.upsert(
            ids=[evidence_id],
            embeddings=[clip_emb],
            metadatas=[meta],
        )

        logger.debug(
            "Indexed image '{}' (evidence={}, case={})",
            path_str,
            evidence_id,
            case_id,
        )

    def index_image_batch(
        self,
        items: list[dict],
    ) -> int:
        """Index multiple images.

        Each item must have keys: evidence_id, case_id, image_path.
        Optional key: description.

        Returns the number of successfully indexed images.
        """
        indexed = 0
        for item in items:
            try:
                self.index_image(
                    evidence_id=item["evidence_id"],
                    case_id=item["case_id"],
                    image_path=item["image_path"],
                    description=item.get("description", ""),
                )
                indexed += 1
            except Exception as exc:
                logger.warning(
                    "Failed to index image '{}': {}",
                    item.get("image_path", "?"),
                    exc,
                )
        return indexed

    # ------------------------------------------------------------------
    # Search
    # ------------------------------------------------------------------

    @staticmethod
    def _format_results(raw: dict) -> List[Dict[str, Any]]:
        """Convert ChromaDB query results into a flat list of dicts."""
        if not raw or not raw.get("ids") or not raw["ids"][0]:
            return []

        ids = raw["ids"][0]
        distances = (raw.get("distances") or [[]])[0]
        metadatas = (raw.get("metadatas") or [[]])[0]

        results: List[Dict[str, Any]] = []
        for i, doc_id in enumerate(ids):
            meta = metadatas[i] if i < len(metadatas) else {}
            distance = distances[i] if i < len(distances) else None
            results.append(
                {
                    "evidence_id": doc_id,
                    "path": meta.get("path", ""),
                    "case_id": meta.get("case_id", ""),
                    "description": meta.get("description", ""),
                    "distance": distance,
                    "similarity": round(1.0 - distance, 4) if distance is not None else None,
                }
            )
        return results

    def search_by_image(
        self,
        image_path: str | Path,
        case_id: Optional[str] = None,
        n_results: int = 5,
    ) -> List[Dict[str, Any]]:
        """Find similar images using DINOv2 embeddings.

        Args:
            image_path: Path to the query image.
            case_id: Optional filter to restrict results to a single case.
            n_results: Maximum number of results to return.

        Returns:
            List of dicts with keys: evidence_id, path, case_id,
            description, distance, similarity.
        """
        emb = self._embedder.embed_image_dinov2(image_path)
        where = {"case_id": case_id} if case_id else None
        raw = self._dinov2_col.query(
            query_embeddings=[emb],
            n_results=n_results,
            where=where,
            include=["distances", "metadatas"],
        )
        return self._format_results(raw)

    def search_by_text(
        self,
        query: str,
        case_id: Optional[str] = None,
        n_results: int = 5,
    ) -> List[Dict[str, Any]]:
        """Find images matching a text query using CLIP.

        Args:
            query: Natural language description of the desired image.
            case_id: Optional filter to restrict results to a single case.
            n_results: Maximum number of results to return.

        Returns:
            List of dicts with keys: evidence_id, path, case_id,
            description, distance, similarity.
        """
        emb = self._embedder.embed_text_clip(query)
        where = {"case_id": case_id} if case_id else None
        raw = self._clip_col.query(
            query_embeddings=[emb],
            n_results=n_results,
            where=where,
            include=["distances", "metadatas"],
        )
        return self._format_results(raw)

    def find_similar_evidence(
        self,
        evidence_id: str,
        case_id: Optional[str] = None,
        n_results: int = 5,
    ) -> List[Dict[str, Any]]:
        """Find images similar to an already-indexed evidence image.

        Retrieves the stored DINOv2 embedding for *evidence_id* and
        queries for neighbours.  Returns an empty list if the evidence
        has not been indexed.
        """
        existing = self._dinov2_col.get(
            ids=[evidence_id], include=["embeddings"]
        )
        if not existing["ids"] or not existing["embeddings"]:
            logger.warning(
                "Evidence '{}' not found in image_dinov2 collection",
                evidence_id,
            )
            return []

        emb = existing["embeddings"][0]
        where = {"case_id": case_id} if case_id else None
        raw = self._dinov2_col.query(
            query_embeddings=[emb],
            n_results=n_results + 1,  # +1 to account for self
            where=where,
            include=["distances", "metadatas"],
        )
        results = self._format_results(raw)
        # Exclude the query image itself
        return [r for r in results if r["evidence_id"] != evidence_id][
            :n_results
        ]

    # ------------------------------------------------------------------
    # Deletion
    # ------------------------------------------------------------------

    def delete_image(self, evidence_id: str) -> None:
        """Remove an image from both collections."""
        try:
            self._dinov2_col.delete(ids=[evidence_id])
        except Exception as exc:
            logger.debug("delete_image: DINOv2 deletion failed for '{}': {}", evidence_id, exc)
        try:
            self._clip_col.delete(ids=[evidence_id])
        except Exception as exc:
            logger.debug("delete_image: CLIP deletion failed for '{}': {}", evidence_id, exc)
        logger.debug("Deleted image '{}' from search index", evidence_id)

    def delete_case_images(self, case_id: str) -> None:
        """Remove all images for a case from both collections."""
        for col in (self._dinov2_col, self._clip_col):
            try:
                matches = col.get(
                    where={"case_id": case_id}, include=[]
                )
                ids_to_delete = matches["ids"] or []
                if ids_to_delete:
                    col.delete(ids=ids_to_delete)
            except Exception as exc:
                logger.warning(
                    "Failed to delete case '{}' images from {}: {}",
                    case_id,
                    col.name,
                    exc,
                )
        logger.info("Deleted all image embeddings for case '{}'", case_id)

    # ------------------------------------------------------------------
    # Stats
    # ------------------------------------------------------------------

    def get_stats(self) -> Dict[str, int]:
        """Return item counts for both image collections."""
        return {
            _DINOV2_COLLECTION: self._dinov2_col.count(),
            _CLIP_COLLECTION: self._clip_col.count(),
        }
