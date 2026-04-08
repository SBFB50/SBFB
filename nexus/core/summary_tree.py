"""
NEXUS -- Hierarchical summary tree (RAPTOR-style).

Maintains a tree of summaries at multiple granularity levels:
- Level 0: Individual evidence summaries (1:1 with evidence)
- Level 1: Cluster summaries (grouped by theme/entity)
- Level 2: Case-level summary (one per case)

Updated incrementally when new evidence is added.

Clustering uses cosine similarity on nomic-embed-text embeddings
with agglomerative clustering (scipy) for rebuild, and a simple
nearest-centroid approach for incremental updates.
"""

from __future__ import annotations

import json
import uuid
from typing import Any, Optional

import numpy as np
from loguru import logger

try:
    from scipy.cluster.hierarchy import fcluster, linkage
    from scipy.spatial.distance import pdist
    _HAS_SCIPY = True
except ImportError:
    _HAS_SCIPY = False
    logger.warning(
        "scipy not installed -- agglomerative clustering unavailable, "
        "rebuild_tree will fall back to single-cluster mode"
    )

from nexus.db.sqlite_db import Database
from nexus.llm.prompts import CLUSTER_SUMMARY_PROMPT, CASE_SUMMARY_PROMPT
from nexus.llm.router import LLMRouter, TaskType

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

# Cosine similarity threshold: below this, a new cluster is created
# instead of merging into the nearest existing one.
_CLUSTER_SIMILARITY_THRESHOLD = 0.40

# Maximum number of evidence items in one cluster before it splits
# (during rebuild only).
_MAX_CLUSTER_SIZE = 20

# Minimum number of evidence items to attempt clustering during rebuild.
_MIN_EVIDENCE_FOR_CLUSTERING = 2

# Distance threshold for agglomerative clustering (cosine distance).
# With nomic-embed-text 768-dim embeddings, intra-topic distances are
# typically 0.85-0.95 and cross-topic distances are 0.98-1.05.
_AGGLOMERATIVE_DISTANCE_THRESHOLD = 0.98


def _cosine_similarity(a: np.ndarray, b: np.ndarray) -> float:
    """Cosine similarity between two vectors."""
    dot = np.dot(a, b)
    norm_a = np.linalg.norm(a)
    norm_b = np.linalg.norm(b)
    if norm_a == 0 or norm_b == 0:
        return 0.0
    return float(dot / (norm_a * norm_b))


def _compute_centroid(vectors: list[list[float]]) -> list[float]:
    """Compute the centroid (mean) of a list of vectors."""
    if not vectors:
        return []
    arr = np.array(vectors, dtype=np.float32)
    centroid = arr.mean(axis=0)
    # Normalize to unit vector for cosine comparisons
    norm = np.linalg.norm(centroid)
    if norm > 0:
        centroid = centroid / norm
    return centroid.tolist()


class SummaryTree:
    """Hierarchical summary tree for a case.

    Usage::

        tree = SummaryTree(db, router, chroma)
        await tree.update_for_new_evidence(case_id, evidence_id)
        context = await tree.get_relevant_summaries(case_id, query)
    """

    def __init__(
        self,
        db: Database,
        router: LLMRouter,
        chroma: Any = None,
    ) -> None:
        self._db = db
        self._router = router
        self._chroma = chroma

    # ==================================================================
    # Public API
    # ==================================================================

    async def update_for_new_evidence(
        self, case_id: str, evidence_id: str
    ) -> None:
        """Update the summary tree after a new evidence is added.

        1. Get evidence summary (level 0 -- already exists)
        2. Find or create the best cluster for this evidence
        3. Regenerate the cluster summary (level 1)
        4. Regenerate the case summary (level 2)
        """
        # 1. Get the evidence summary
        evidence = await self._db.get_evidence(evidence_id)
        if not evidence:
            logger.warning(
                "SummaryTree: evidence {} not found, skipping", evidence_id
            )
            return

        summary = evidence.get("summary") or ""
        if not summary.strip():
            logger.debug(
                "SummaryTree: evidence {} has no summary, skipping tree update",
                evidence_id,
            )
            return

        logger.info(
            "SummaryTree: updating tree for new evidence {} in case {}",
            evidence_id[:8],
            case_id[:8],
        )

        # 2. Find or create the best cluster
        try:
            cluster_id = await self._find_best_cluster(case_id, summary)
        except Exception as exc:
            logger.error(
                "SummaryTree: cluster assignment failed for {}: {}",
                evidence_id[:8],
                exc,
            )
            return

        # Add evidence to the cluster's evidence_ids list
        cluster = await self._db.get_cluster(cluster_id)
        if cluster:
            existing_ids = json.loads(cluster.get("evidence_ids") or "[]")
            if evidence_id not in existing_ids:
                existing_ids.append(evidence_id)
                await self._db.update_cluster(
                    cluster_id,
                    evidence_ids=json.dumps(existing_ids),
                )

            # Update centroid with the new evidence embedding
            await self._update_cluster_centroid(cluster_id)

        # 3. Regenerate cluster summary
        try:
            await self._generate_cluster_summary(cluster_id)
        except Exception as exc:
            logger.error(
                "SummaryTree: cluster summary generation failed for {}: {}",
                cluster_id[:8],
                exc,
            )

        # 4. Case summary deferred — too expensive to regenerate on every
        #    new evidence (loads nexus 26B each time). Will be generated
        #    by the SummaryTreeWorker on TICK_SUMMARY_TREE or via rebuild_tree().
        logger.debug(
            "SummaryTree: case summary deferred for {} (will rebuild on next tick)",
            case_id[:8],
        )

        logger.info(
            "SummaryTree: tree updated for evidence {} -> cluster {}",
            evidence_id[:8],
            cluster_id[:8],
        )

    async def get_case_summary(self, case_id: str) -> str:
        """Get the current level-2 case summary."""
        row = await self._db.get_case_summary(case_id)
        if row:
            return row.get("summary") or ""
        return ""

    async def get_cluster_summaries(self, case_id: str) -> list[dict]:
        """Get all level-1 cluster summaries.

        Returns a list of dicts:
        [{"id", "title", "summary", "evidence_ids"}, ...]
        """
        clusters = await self._db.list_clusters_by_case(case_id)
        result = []
        for c in clusters:
            result.append({
                "id": c["id"],
                "title": c.get("title") or "",
                "summary": c.get("summary") or "",
                "evidence_ids": json.loads(c.get("evidence_ids") or "[]"),
            })
        return result

    async def get_relevant_summaries(
        self, case_id: str, query: str, n_clusters: int = 3
    ) -> dict:
        """Get the case summary + most relevant cluster summaries for a query.

        Returns::

            {
                "case_summary": str,
                "cluster_summaries": [
                    {"title": str, "summary": str, "evidence_ids": [str]}
                ]
            }

        Uses embedding similarity to rank clusters against the query.
        """
        case_summary = await self.get_case_summary(case_id)
        clusters = await self._db.list_clusters_by_case(case_id)

        if not clusters:
            return {
                "case_summary": case_summary,
                "cluster_summaries": [],
            }

        # Embed the query
        try:
            query_embedding = await self._router.embed(query)
            query_vec = np.array(query_embedding, dtype=np.float32)
        except Exception as exc:
            logger.warning(
                "SummaryTree: query embedding failed, returning all clusters: {}",
                exc,
            )
            # Fallback: return all clusters without ranking
            return {
                "case_summary": case_summary,
                "cluster_summaries": [
                    {
                        "title": c.get("title") or "",
                        "summary": c.get("summary") or "",
                        "evidence_ids": json.loads(
                            c.get("evidence_ids") or "[]"
                        ),
                    }
                    for c in clusters[:n_clusters]
                ],
            }

        # Score each cluster by cosine similarity between query and centroid
        scored: list[tuple[float, dict]] = []
        for c in clusters:
            centroid_raw = c.get("embedding_centroid")
            if not centroid_raw:
                # No centroid -- assign a low similarity
                scored.append((0.0, c))
                continue
            try:
                centroid = np.array(
                    json.loads(centroid_raw), dtype=np.float32
                )
                sim = _cosine_similarity(query_vec, centroid)
                scored.append((sim, c))
            except (json.JSONDecodeError, ValueError):
                scored.append((0.0, c))

        # Sort by similarity descending, take top n
        scored.sort(key=lambda x: x[0], reverse=True)
        top = scored[:n_clusters]

        return {
            "case_summary": case_summary,
            "cluster_summaries": [
                {
                    "title": c.get("title") or "",
                    "summary": c.get("summary") or "",
                    "evidence_ids": json.loads(
                        c.get("evidence_ids") or "[]"
                    ),
                }
                for _, c in top
            ],
        }

    async def rebuild_tree(self, case_id: str) -> None:
        """Rebuild the entire summary tree from scratch.

        Useful after bulk import or when tree is corrupted.
        1. Get all evidence summaries
        2. Cluster them by embedding similarity (agglomerative)
        3. Generate cluster summaries
        4. Generate case summary
        """
        logger.info("SummaryTree: rebuilding tree for case {}", case_id[:8])

        # 1. Gather all processed evidence with summaries
        all_evidence = await self._db.list_evidence_by_case(
            case_id, status="processed"
        )
        evidence_with_summaries = [
            ev for ev in all_evidence if (ev.get("summary") or "").strip()
        ]

        if not evidence_with_summaries:
            logger.info(
                "SummaryTree: no evidence with summaries for case {}, "
                "nothing to rebuild",
                case_id[:8],
            )
            return

        # Delete existing clusters and case summary for this case
        existing_clusters = await self._db.list_clusters_by_case(case_id)
        for c in existing_clusters:
            await self._db.delete_cluster(c["id"])

        # 2. Embed all summaries
        summaries = [ev["summary"] for ev in evidence_with_summaries]
        evidence_ids = [ev["id"] for ev in evidence_with_summaries]

        try:
            embeddings = await self._router.embed_batch(summaries)
        except Exception as exc:
            logger.error(
                "SummaryTree: batch embedding failed during rebuild: {}", exc
            )
            return

        if len(evidence_with_summaries) < _MIN_EVIDENCE_FOR_CLUSTERING or not _HAS_SCIPY:
            # Too few items or scipy unavailable: put everything in one cluster
            cluster_labels = [0] * len(evidence_with_summaries)
        else:
            # 3. Agglomerative clustering
            cluster_labels = self._agglomerative_cluster(embeddings)

        # Group evidence by cluster label
        clusters_map: dict[int, list[int]] = {}
        for idx, label in enumerate(cluster_labels):
            clusters_map.setdefault(label, []).append(idx)

        logger.info(
            "SummaryTree: rebuild produced {} clusters from {} evidence items",
            len(clusters_map),
            len(evidence_with_summaries),
        )

        # 4. Create cluster records and generate summaries
        for label, indices in clusters_map.items():
            cluster_evidence_ids = [evidence_ids[i] for i in indices]
            cluster_embeddings = [embeddings[i] for i in indices]
            centroid = _compute_centroid(cluster_embeddings)

            cluster_row = await self._db.create_cluster(
                case_id=case_id,
                title=None,  # Will be set by summary generation
                summary=None,
                evidence_ids=json.dumps(cluster_evidence_ids),
                embedding_centroid=json.dumps(centroid),
            )
            cluster_id = cluster_row["id"]

            # Generate summary for this cluster
            try:
                await self._generate_cluster_summary(cluster_id)
            except Exception as exc:
                logger.warning(
                    "SummaryTree: cluster summary failed during rebuild "
                    "for cluster {}: {}",
                    cluster_id[:8],
                    exc,
                )

        # 5. Generate case summary
        try:
            await self._generate_case_summary(case_id)
        except Exception as exc:
            logger.error(
                "SummaryTree: case summary generation failed during rebuild: {}",
                exc,
            )

        logger.info(
            "SummaryTree: rebuild complete for case {} "
            "({} clusters, {} evidence items)",
            case_id[:8],
            len(clusters_map),
            len(evidence_with_summaries),
        )

    # ==================================================================
    # Internal methods
    # ==================================================================

    async def _find_best_cluster(
        self, case_id: str, evidence_summary: str
    ) -> str:
        """Find the best existing cluster for a new evidence, or create one.

        Uses embedding similarity against cluster centroids.
        If best match < threshold, creates a new cluster.

        Returns the cluster_id.
        """
        clusters = await self._db.list_clusters_by_case(case_id)

        # If no clusters exist, create the first one
        if not clusters:
            logger.debug(
                "SummaryTree: no clusters for case {}, creating first one",
                case_id[:8],
            )
            return await self._create_new_cluster(case_id)

        # Embed the evidence summary
        try:
            evidence_embedding = await self._router.embed(evidence_summary)
        except Exception as exc:
            logger.warning(
                "SummaryTree: embedding failed, creating new cluster: {}", exc
            )
            return await self._create_new_cluster(case_id)

        ev_vec = np.array(evidence_embedding, dtype=np.float32)

        # Compare against each cluster centroid
        best_sim = -1.0
        best_cluster_id: Optional[str] = None

        for cluster in clusters:
            centroid_raw = cluster.get("embedding_centroid")
            if not centroid_raw:
                continue

            try:
                centroid = np.array(
                    json.loads(centroid_raw), dtype=np.float32
                )
                sim = _cosine_similarity(ev_vec, centroid)
                if sim > best_sim:
                    best_sim = sim
                    best_cluster_id = cluster["id"]
            except (json.JSONDecodeError, ValueError):
                continue

        if best_cluster_id and best_sim >= _CLUSTER_SIMILARITY_THRESHOLD:
            logger.debug(
                "SummaryTree: evidence matched cluster {} (sim={:.3f})",
                best_cluster_id[:8],
                best_sim,
            )
            return best_cluster_id

        # No good match -- create a new cluster
        logger.debug(
            "SummaryTree: best sim={:.3f} < threshold={}, creating new cluster",
            best_sim,
            _CLUSTER_SIMILARITY_THRESHOLD,
        )
        return await self._create_new_cluster(case_id)

    async def _create_new_cluster(self, case_id: str) -> str:
        """Create a new empty cluster for a case."""
        cluster_row = await self._db.create_cluster(
            case_id=case_id,
            title=None,
            summary=None,
            evidence_ids=json.dumps([]),
            embedding_centroid=None,
        )
        return cluster_row["id"]

    async def _update_cluster_centroid(self, cluster_id: str) -> None:
        """Recompute the centroid for a cluster from its evidence embeddings."""
        cluster = await self._db.get_cluster(cluster_id)
        if not cluster:
            return

        evidence_ids = json.loads(cluster.get("evidence_ids") or "[]")
        if not evidence_ids:
            return

        # Gather summaries for embedding
        summaries = []
        for eid in evidence_ids:
            ev = await self._db.get_evidence(eid)
            if ev and (ev.get("summary") or "").strip():
                summaries.append(ev["summary"])

        if not summaries:
            return

        try:
            embeddings = await self._router.embed_batch(summaries)
            centroid = _compute_centroid(embeddings)
            await self._db.update_cluster(
                cluster_id,
                embedding_centroid=json.dumps(centroid),
            )
        except Exception as exc:
            logger.warning(
                "SummaryTree: centroid update failed for cluster {}: {}",
                cluster_id[:8],
                exc,
            )

    async def _generate_cluster_summary(self, cluster_id: str) -> str:
        """Regenerate a cluster summary from its evidence summaries.

        Uses the fast model (gemma4:e4b via EVIDENCE_SUMMARY task type)
        to generate a JSON with title + summary.
        """
        cluster = await self._db.get_cluster(cluster_id)
        if not cluster:
            return ""

        evidence_ids = json.loads(cluster.get("evidence_ids") or "[]")
        if not evidence_ids:
            return ""

        # Gather evidence summaries
        evidence_summaries_parts: list[str] = []
        for eid in evidence_ids:
            ev = await self._db.get_evidence(eid)
            if ev:
                title = ev.get("title", "Sans titre")
                summary = ev.get("summary", "")
                if summary:
                    evidence_summaries_parts.append(
                        f"[{title}]: {summary}"
                    )

        if not evidence_summaries_parts:
            return ""

        evidence_text = "\n\n".join(evidence_summaries_parts)

        prompt = CLUSTER_SUMMARY_PROMPT.format(
            n=len(evidence_summaries_parts),
            evidence_summaries=evidence_text,
        )

        try:
            result = await self._router.route_json(
                TaskType.EVIDENCE_SUMMARY, prompt
            )
            title = result.get("title", "Groupe thematique")
            summary = result.get("summary", "")
        except Exception as exc:
            logger.debug("SummaryTree: JSON cluster summary failed, falling back to plain text: {}", exc)
            # Fallback: use plain text generation
            try:
                raw = await self._router.route(
                    TaskType.EVIDENCE_SUMMARY, prompt
                )
                title = "Groupe thematique"
                summary = raw.strip()
            except Exception as exc:
                logger.error(
                    "SummaryTree: cluster summary LLM call failed: {}", exc
                )
                return ""

        # Update the cluster record
        await self._db.update_cluster(
            cluster_id,
            title=title,
            summary=summary,
        )

        logger.debug(
            "SummaryTree: cluster {} summary generated: '{}'",
            cluster_id[:8],
            title,
        )
        return summary

    async def _generate_case_summary(self, case_id: str) -> str:
        """Regenerate the case-level summary from cluster summaries.

        Uses the deep model (nexus 26B) for comprehensive synthesis.
        """
        clusters = await self._db.list_clusters_by_case(case_id)
        if not clusters:
            return ""

        # Build cluster summaries text
        cluster_parts: list[str] = []
        cluster_ids: list[str] = []
        for c in clusters:
            title = c.get("title") or "Sans titre"
            summary = c.get("summary") or ""
            n_evidence = len(json.loads(c.get("evidence_ids") or "[]"))
            if summary:
                cluster_parts.append(
                    f"### {title} ({n_evidence} preuves)\n{summary}"
                )
                cluster_ids.append(c["id"])

        if not cluster_parts:
            return ""

        # Get case info
        case = await self._db.get_case(case_id)
        case_name = case.get("name", "Dossier inconnu") if case else "Dossier inconnu"
        case_reference = case.get("reference", "N/A") if case else "N/A"

        # Get active hypotheses
        hypotheses = await self._db.list_hypotheses_by_case(
            case_id, status="active"
        )
        hypotheses_text = "\n".join(
            f"- {h['title']} (score: {h['current_score']}%): "
            f"{h.get('description', '')[:200]}"
            for h in hypotheses
        ) if hypotheses else "(aucune hypothese active)"

        prompt = CASE_SUMMARY_PROMPT.format(
            n=len(cluster_parts),
            case_name=case_name,
            case_reference=case_reference,
            cluster_summaries="\n\n".join(cluster_parts),
            hypotheses=hypotheses_text,
        )

        try:
            summary = await self._router.route(
                TaskType.DEEP_ANALYSIS, prompt
            )
            summary = summary.strip()
        except Exception as exc:
            logger.error(
                "SummaryTree: case summary LLM call failed: {}", exc
            )
            return ""

        # Upsert the case summary record
        await self._db.create_or_update_case_summary(
            case_id=case_id,
            summary=summary,
            cluster_ids=json.dumps(cluster_ids),
        )

        logger.info(
            "SummaryTree: case summary updated for {} ({} clusters)",
            case_id[:8],
            len(cluster_parts),
        )
        return summary

    # ==================================================================
    # Clustering algorithm
    # ==================================================================

    @staticmethod
    def _agglomerative_cluster(
        embeddings: list[list[float]],
    ) -> list[int]:
        """Agglomerative clustering on embedding vectors.

        Uses cosine distance with a distance threshold to determine
        the number of clusters automatically.

        Returns a list of integer labels (one per input embedding).
        """
        if not _HAS_SCIPY:
            # Fallback: treat all embeddings as a single cluster
            return [0] * len(embeddings)

        n = len(embeddings)
        if n <= 1:
            return list(range(n))

        arr = np.array(embeddings, dtype=np.float32)

        # Normalize vectors for cosine distance computation
        norms = np.linalg.norm(arr, axis=1, keepdims=True)
        norms[norms == 0] = 1.0
        arr_normalized = arr / norms

        # Compute pairwise cosine distances
        distances = pdist(arr_normalized, metric="cosine")

        # Hierarchical clustering with average linkage
        Z = linkage(distances, method="average")

        # Cut the dendrogram at the distance threshold
        labels = fcluster(
            Z, t=_AGGLOMERATIVE_DISTANCE_THRESHOLD, criterion="distance"
        )

        # fcluster labels start at 1; shift to 0-based
        return [int(l) - 1 for l in labels]
