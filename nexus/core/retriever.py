"""
NEXUS -- Hybrid retriever for investigation RAG.

Retrieves the most relevant evidence chunks using three strategies:
1. Semantic: cosine similarity via ChromaDB embeddings (evidence_chunks)
2. Graph: Neo4j traversal from entities mentioned in the query
3. Recency: boost recent evidence and monitoring results

Results are merged, deduplicated, and reranked.

Usage::

    retriever = InvestigationRetriever(chroma, neo4j, router, db)
    chunks = await retriever.retrieve("timeline of victim's last night", case_id)
    context = await retriever.build_analysis_context(case_id, max_tokens=4000)
"""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from typing import Any, Optional

from loguru import logger

from nexus.core.embedding_store import EmbeddingStore
from nexus.db.chroma_db import ChromaClient
from nexus.db.neo4j_db import Neo4jClient
from nexus.db.sqlite_db import Database
from nexus.llm.router import LLMRouter, TaskType


# Weight constants for hybrid reranking
_SEMANTIC_WEIGHT = 0.50
_GRAPH_WEIGHT = 0.25
_FTS_WEIGHT = 0.15
_RECENCY_WEIGHT = 0.10

# How many days count as "recent" for recency boosting
_RECENCY_HORIZON_DAYS = 30


class InvestigationRetriever:
    """Retrieve relevant context for LLM analysis.

    Combines semantic search (ChromaDB evidence_chunks), graph traversal
    (Neo4j entity connections), and recency boosting into a single
    ranked result set.
    """

    def __init__(
        self,
        chroma: ChromaClient | None,
        neo4j: Neo4jClient | None,
        router: LLMRouter,
        db: Database,
    ) -> None:
        self._chroma = chroma
        self._neo4j = neo4j
        self._router = router
        self._db = db
        # Build the EmbeddingStore wrapper if ChromaDB is available
        self._embedding_store: EmbeddingStore | None = None
        if chroma is not None:
            try:
                self._embedding_store = EmbeddingStore(chroma, router)
            except Exception as exc:
                logger.warning(
                    "Could not init EmbeddingStore for retriever: {}", exc
                )

    # ==================================================================
    # Main retrieve
    # ==================================================================

    async def retrieve(
        self,
        query: str,
        case_id: str,
        n_results: int = 20,
        strategy: str = "hybrid",  # "semantic", "graph", "hybrid"
        include_hypotheses: bool = False,
    ) -> list[dict]:
        """Retrieve relevant evidence chunks.

        Hybrid strategy:
        1. Semantic search via ChromaDB unified cross-collection (top 2*n)
        2. Graph search: find entities in query, traverse Neo4j,
           get connected evidence
        3. Merge + deduplicate by evidence_id + chunk_text
        4. Rerank by (semantic * 0.50 + graph * 0.25 + fts * 0.15 + recency * 0.10)

        Args:
            include_hypotheses: When True, also search hypothesis_reasoning
                collection (useful for hypothesis-focused retrieval).
        """
        results: list[dict] = []

        if strategy in ("semantic", "hybrid"):
            semantic = await self._semantic_search(
                query, case_id, n_results * 2,
                include_hypotheses=include_hypotheses,
            )
            results.extend(semantic)

        if strategy in ("graph", "hybrid") and self._neo4j:
            graph = await self._graph_search(query, case_id, n_results)
            results.extend(graph)

        # FTS5 lexical search (catches exact keywords semantic may miss)
        if strategy in ("hybrid",):
            fts = await self._fts_search(query, case_id, n_results)
            results.extend(fts)

        # Deduplicate by evidence_id + chunk_text prefix, keep best score
        deduped = self._deduplicate(results)

        # Rerank
        ranked = self._rerank(deduped, n_results)

        return ranked

    # ==================================================================
    # Hypothesis-focused retrieval
    # ==================================================================

    async def retrieve_for_hypothesis(
        self,
        hypothesis: dict,
        case_id: str,
        n_results: int = 15,
    ) -> dict[str, list[dict]]:
        """Retrieve evidence relevant to a specific hypothesis.

        Uses hypothesis title + description as query.
        Also retrieves evidence that CONTRADICTS the hypothesis.
        Includes hypothesis_reasoning collection for cross-referencing
        with other hypothesis snapshots.
        """
        # Supporting evidence (include hypothesis_reasoning for cross-reference)
        query = f"{hypothesis.get('title', '')}. {hypothesis.get('description', '')}"
        supporting = await self.retrieve(
            query, case_id, n_results, include_hypotheses=True,
        )

        # Contradicting evidence (negate the hypothesis)
        contra_query = f"Ce qui contredit: {hypothesis.get('title', '')}"
        contradicting = await self.retrieve(
            contra_query, case_id, n_results // 2, include_hypotheses=True,
        )

        return {
            "supporting": supporting,
            "contradicting": contradicting,
        }

    # ==================================================================
    # Context builder for LLM
    # ==================================================================

    async def build_analysis_context(
        self,
        case_id: str,
        focus: str | None = None,
        max_tokens: int = 4000,
    ) -> str:
        """Build a focused context for LLM analysis.

        Instead of loading ALL evidence, retrieves only what's relevant.

        Args:
            case_id: The case to analyze.
            focus: Optional focus query (e.g. "timeline of victim's last
                   night").
            max_tokens: Max approximate tokens for the context.

        Returns a structured text block ready for LLM injection.
        """
        # Get case summary
        case = await self._db.get_case(case_id)

        # Get hypotheses (just titles + scores, not full text)
        hypotheses = await self._db.list_hypotheses_by_case(
            case_id, status="active"
        )
        hyp_summary = "\n".join(
            f"- {h['title']} ({h.get('current_score', 50):.0f}%)"
            for h in hypotheses[:10]
        )

        # Get entity summary (top entities by name)
        entities = await self._db.list_entities_by_case(case_id)
        ent_summary = "\n".join(
            f"- {e['name']} ({e.get('entity_type', '?')})"
            for e in entities[:20]
        )

        # Retrieve relevant chunks
        if focus:
            chunks = await self.retrieve(focus, case_id, n_results=15)
        else:
            # Default: retrieve based on case description + top hypothesis
            query = case.get("description", "") if case else ""
            if hypotheses:
                query += f". Hypothese principale: {hypotheses[0]['title']}"
            chunks = await self.retrieve(query, case_id, n_results=20)

        # Build context text within token budget
        case_name = case.get("name", "") if case else ""
        case_ref = case.get("reference", "") if case else ""
        case_desc = case.get("description", "") if case else ""

        context_parts: list[str] = [
            f"DOSSIER: {case_name} ({case_ref})",
            f"Description: {case_desc}",
        ]
        if hyp_summary:
            context_parts.append(f"\nHYPOTHESES ACTIVES:\n{hyp_summary}")
        if ent_summary:
            context_parts.append(f"\nENTITES PRINCIPALES:\n{ent_summary}")

        # Add RAPTOR case summary if available
        try:
            case_summary_row = await self._db.get_case_summary(case_id)
            if case_summary_row and case_summary_row.get("summary"):
                context_parts.append(
                    f"\nRESUME GLOBAL DU DOSSIER:\n{case_summary_row['summary']}"
                )
        except Exception:
            pass

        # Add latest timeline from analysis runs if available
        try:
            runs = await self._db.list_runs_by_case(
                case_id, status="completed", limit=5
            )
            for run in runs:
                if run.get("run_type") == "timeline_rebuild":
                    output = run.get("output_summary", "")
                    if output and "CHRONOLOGIE" in output:
                        context_parts.append(f"\n{output[:2000]}")
                    break  # only include the most recent timeline
        except Exception:
            pass

        context_parts.append("\nPREUVES PERTINENTES:")

        # Add chunks until token budget is reached
        char_budget = max_tokens * 4  # rough token-to-char ratio
        current_chars = sum(len(p) for p in context_parts)

        for chunk in chunks:
            title = chunk.get("title", "?")
            source = chunk.get("source", "?")
            text = chunk.get("chunk_text", "")
            chunk_text = f"\n[{title} -- {source}]: {text}"
            if current_chars + len(chunk_text) > char_budget:
                break
            context_parts.append(chunk_text)
            current_chars += len(chunk_text)

        return "\n".join(context_parts)

    # ==================================================================
    # Semantic search (private)
    # ==================================================================

    async def _semantic_search(
        self,
        query: str,
        case_id: str,
        n: int,
        include_hypotheses: bool = False,
    ) -> list[dict]:
        """Search ChromaDB via unified cross-collection search.

        Uses ``ChromaClient.unified_search()`` to query evidence_chunks,
        entity_contexts, and monitoring_results in a single pass. When
        *include_hypotheses* is True, also searches hypothesis_reasoning.

        Falls back to EmbeddingStore single-collection search if ChromaDB
        unified search is unavailable.

        Each result is enriched with a ``_semantic_score`` in [0, 1]
        derived from the cosine distance (score = 1 - distance).
        """
        if self._chroma is None and self._embedding_store is None:
            logger.debug("No ChromaDB available; semantic search skipped")
            return []

        # --- Strategy 1: Unified cross-collection search ---
        if self._chroma is not None:
            try:
                query_embedding = await self._router.embed(query)

                collections = [
                    "evidence_chunks",
                    "entity_contexts",
                    "monitoring_results",
                ]
                if include_hypotheses:
                    collections.append("hypothesis_reasoning")

                # Distribute n across collections (evidence gets the lion's share)
                n_per_col = max(3, n // len(collections))

                raw_results = await self._chroma.unified_search(
                    query_embedding=query_embedding,
                    case_id=case_id,
                    collections=collections,
                    n_per_collection=n_per_col,
                )

                results: list[dict] = []
                for r in raw_results:
                    distance = r.get("distance") or 1.0
                    semantic_score = max(0.0, 1.0 - distance)
                    meta = r.get("metadata") or {}
                    entry = {
                        "chunk_text": r.get("text", ""),
                        "evidence_id": meta.get("evidence_id", r.get("id", "")),
                        "title": meta.get("title", ""),
                        "source": meta.get("source", ""),
                        "metadata": meta,
                        "_semantic_score": semantic_score,
                        "_graph_score": 0.0,
                        "_recency_score": self._compute_recency_score(meta),
                        "_source": f"semantic:{r.get('collection', 'unknown')}",
                        "_collection": r.get("collection", ""),
                    }
                    results.append(entry)

                logger.debug(
                    "Unified semantic search for case {}: {} results from {} collections",
                    case_id,
                    len(results),
                    len(collections),
                )
                return results[:n]

            except Exception as exc:
                logger.warning(
                    "Unified search failed, falling back to EmbeddingStore: {}",
                    exc,
                )
                # Fall through to Strategy 2

        # --- Strategy 2: Fallback to EmbeddingStore (evidence_chunks only) ---
        if self._embedding_store is None:
            logger.debug("No EmbeddingStore available; semantic search skipped")
            return []

        try:
            raw_results = await self._embedding_store.search(
                query=query,
                case_id=case_id,
                n_results=n,
            )
        except Exception as exc:
            logger.error("Semantic search failed: {}", exc)
            return []

        results = []
        for r in raw_results:
            distance = r.get("distance", 1.0)
            semantic_score = max(0.0, 1.0 - distance)
            entry = {
                "chunk_text": r.get("chunk_text", ""),
                "evidence_id": r.get("evidence_id", ""),
                "title": r.get("title", ""),
                "source": r.get("source", ""),
                "metadata": r.get("metadata", {}),
                "_semantic_score": semantic_score,
                "_graph_score": 0.0,
                "_recency_score": self._compute_recency_score(r.get("metadata", {})),
                "_source": "semantic",
            }
            results.append(entry)

        logger.debug(
            "Semantic search for case {}: {} results", case_id, len(results)
        )
        return results

    # ==================================================================
    # Graph search (private)
    # ==================================================================

    async def _graph_search(
        self,
        query: str,
        case_id: str,
        n: int,
    ) -> list[dict]:
        """Extract entities from query, traverse Neo4j, get connected evidence.

        Steps:
        1. Use the fast LLM (gemma4:e4b) to extract entity names from query
        2. Match those names to known Neo4j nodes in this case
        3. Traverse 1-2 hops to find connected Evidence nodes
        4. Load the evidence text from SQLite for each result
        5. Assign a graph_score boosted by centrality and bridge importance

        Returns a list of dicts compatible with semantic results.
        """
        if self._neo4j is None:
            return []

        # Step 1: Extract entity names from query using fast LLM
        entity_names = await self._extract_entity_names(query, case_id)
        if not entity_names:
            logger.debug("Graph search: no entities extracted from query")
            return []

        # Step 2: Find matching nodes in Neo4j for this case
        try:
            all_nodes = await self._neo4j.find_nodes_by_case(case_id)
        except Exception as exc:
            logger.warning("Neo4j find_nodes_by_case failed: {}", exc)
            return []

        # Build a name -> node_id lookup (lowercase for fuzzy matching)
        name_to_nodes: dict[str, list[dict]] = {}
        for node in all_nodes:
            node_name = (node.get("name") or "").lower().strip()
            if node_name:
                name_to_nodes.setdefault(node_name, []).append(node)
            # Also index aliases
            aliases_str = node.get("aliases", "")
            if aliases_str:
                for alias in str(aliases_str).split(","):
                    alias_clean = alias.strip().lower()
                    if alias_clean:
                        name_to_nodes.setdefault(alias_clean, []).append(node)

        # Match extracted entity names to nodes
        matched_node_ids: set[str] = set()
        for name in entity_names:
            name_lower = name.lower().strip()
            # Exact match first
            if name_lower in name_to_nodes:
                for node in name_to_nodes[name_lower]:
                    matched_node_ids.add(node["id"])
            else:
                # Partial match (entity name appears as substring of node name)
                for node_name, nodes in name_to_nodes.items():
                    if name_lower in node_name or node_name in name_lower:
                        for node in nodes:
                            matched_node_ids.add(node["id"])

        if not matched_node_ids:
            logger.debug("Graph search: no nodes matched extracted entities")
            return []

        logger.debug(
            "Graph search: matched {} nodes from {} entities",
            len(matched_node_ids),
            len(entity_names),
        )

        # Step 2b: Load centrality data to boost important entities
        centrality_map: dict[str, float] = {}  # node_id -> centrality bonus
        try:
            central = await self._neo4j.get_central_entities(case_id, limit=30)
            if central:
                max_degree = max(e["degree"] for e in central) or 1
                for e in central:
                    # Normalize degree to [0, 0.3] bonus
                    centrality_map[e["id"]] = 0.3 * (e["degree"] / max_degree)
        except Exception as exc:
            logger.debug("Centrality lookup failed (non-critical): {}", exc)

        bridge_ids: set[str] = set()
        try:
            important = await self._neo4j.get_entity_importance(case_id, limit=15)
            bridge_ids = {e["id"] for e in important if e.get("betweenness_approx", 0) > 0}
        except Exception as exc:
            logger.debug("Importance lookup failed (non-critical): {}", exc)

        # Step 3: Traverse 1-2 hops from each matched node to find Evidence
        evidence_ids_with_hops: dict[str, int] = {}  # evidence_id -> min_hops
        evidence_source_nodes: dict[str, set[str]] = {}  # evidence_id -> source node ids
        for node_id in matched_node_ids:
            try:
                subgraph = await self._neo4j.get_neighbors(node_id, depth=2)
                for gnode in subgraph.get("nodes", []):
                    labels = gnode.get("_labels", [])
                    if "Evidence" in labels:
                        eid = gnode.get("id", "")
                        if eid:
                            # Compute approximate hop distance
                            hop = 1 if eid in {
                                e.get("from") or e.get("to")
                                for e in subgraph.get("edges", [])
                                if (e.get("from") == node_id or e.get("to") == node_id)
                            } else 2
                            existing = evidence_ids_with_hops.get(eid, 99)
                            evidence_ids_with_hops[eid] = min(existing, hop)
                            evidence_source_nodes.setdefault(eid, set()).add(node_id)
            except Exception as exc:
                logger.warning("Graph traversal failed for node {}: {}", node_id, exc)

        if not evidence_ids_with_hops:
            logger.debug("Graph search: no evidence found via graph traversal")
            return []

        # Step 4: Load evidence from SQLite, apply centrality + bridge boosts
        results: list[dict] = []
        for eid, hops in evidence_ids_with_hops.items():
            try:
                evidence = await self._db.get_evidence(eid)
                if evidence is None:
                    continue
                # Base graph score: 1.0 for 1-hop, 0.5 for 2-hop
                graph_score = 1.0 if hops <= 1 else 0.5

                # Boost: centrality of source nodes
                source_nodes = evidence_source_nodes.get(eid, set())
                centrality_bonus = max(
                    (centrality_map.get(nid, 0.0) for nid in source_nodes),
                    default=0.0,
                )
                graph_score = min(1.0, graph_score + centrality_bonus)

                # Boost: if evidence is connected via a bridge entity
                if source_nodes & bridge_ids:
                    graph_score = min(1.0, graph_score + 0.15)

                # Boost: evidence connected through multiple matched entities
                if len(source_nodes) > 1:
                    multi_entity_bonus = 0.1 * min(len(source_nodes) - 1, 3)
                    graph_score = min(1.0, graph_score + multi_entity_bonus)

                text = evidence.get("summary") or (evidence.get("raw_text", "")[:2000])
                entry = {
                    "chunk_text": text,
                    "evidence_id": eid,
                    "title": evidence.get("title", ""),
                    "source": evidence.get("source", ""),
                    "metadata": {
                        "evidence_type": evidence.get("evidence_type", ""),
                        "reliability": evidence.get("reliability", 50),
                        "created_at": evidence.get("created_at", ""),
                    },
                    "_semantic_score": 0.0,
                    "_graph_score": graph_score,
                    "_recency_score": self._compute_recency_score({
                        "created_at": evidence.get("created_at", ""),
                    }),
                    "_source": "graph",
                }
                results.append(entry)
            except Exception as exc:
                logger.warning("Failed to load evidence {} for graph results: {}", eid, exc)

        logger.debug(
            "Graph search for case {}: {} results from {} evidence nodes",
            case_id,
            len(results),
            len(evidence_ids_with_hops),
        )
        # Limit to n results, sorted by graph_score descending
        results.sort(key=lambda x: x["_graph_score"], reverse=True)
        return results[:n]

    # ==================================================================
    # FTS5 lexical search (private)
    # ==================================================================

    @staticmethod
    def _sanitize_fts_query(query: str) -> str:
        """Sanitize a user query for FTS5 MATCH.

        Wraps each word in double quotes to force literal matching,
        stripping any embedded quotes and FTS5 operators.
        """
        # Remove FTS5 special characters
        cleaned = (
            query.replace('"', " ").replace("*", " ").replace("^", " ")
            .replace("(", " ").replace(")", " ").replace(":", " ")
        )
        words = [w.strip() for w in cleaned.split() if len(w.strip()) >= 2]
        # Skip FTS5 boolean operators
        fts_operators = {"AND", "OR", "NOT", "NEAR"}
        words = [w for w in words if w.upper() not in fts_operators]
        if not words:
            return ""
        # Wrap each word in double quotes for literal matching
        return " ".join(f'"{w}"' for w in words)

    async def _fts_search(
        self,
        query: str,
        case_id: str,
        n: int,
    ) -> list[dict]:
        """Full-text search via SQLite FTS5.

        Catches exact keyword matches that semantic search may miss
        (proper names, case numbers, dates, technical terms).
        Uses a dedicated _fts_score field for independent weight tuning.
        """
        try:
            fts_query = self._sanitize_fts_query(query)
            if not fts_query:
                return []

            raw = await self._db.search_evidence_fts(
                case_id=case_id,
                query=fts_query,
                limit=n,
            )
        except Exception as exc:
            logger.debug("FTS5 search failed (non-blocking): {}", exc)
            return []

        results: list[dict] = []
        n_results = max(len(raw), 1)
        for i, ev in enumerate(raw):
            # BM25-ranked results: assign decaying score based on position
            fts_score = max(0.1, 1.0 - (i / n_results))
            text = ev.get("summary") or (ev.get("raw_text", "")[:2000])
            entry = {
                "chunk_text": text,
                "evidence_id": ev.get("id", ""),
                "title": ev.get("title", ""),
                "source": ev.get("source", ""),
                "metadata": {
                    "evidence_type": ev.get("evidence_type", ""),
                    "reliability": ev.get("reliability", 50),
                    "created_at": ev.get("created_at", ""),
                },
                "_semantic_score": 0.0,
                "_graph_score": 0.0,
                "_fts_score": fts_score,
                "_recency_score": self._compute_recency_score({
                    "created_at": ev.get("created_at", ""),
                }),
                "_source": "fts5",
            }
            results.append(entry)

        if results:
            logger.debug(
                "FTS5 search for case {}: {} results", case_id, len(results)
            )
        return results

    # ==================================================================
    # Entity extraction from query (private)
    # ==================================================================

    async def _extract_entity_names(
        self,
        query: str,
        case_id: str,
    ) -> list[str]:
        """Extract entity names from a query string.

        Uses a fast approach:
        1. Try matching against known entities in the DB (cheap)
        2. If no matches found, use the fast LLM to extract names
        """
        # First pass: match against known entities (no LLM call needed)
        entities = await self._db.list_entities_by_case(case_id)
        query_lower = query.lower()

        matched_names: list[str] = []
        for ent in entities:
            name = ent.get("name", "")
            if name.lower() in query_lower:
                matched_names.append(name)
            # Check aliases too
            aliases = ent.get("aliases") or []
            if isinstance(aliases, list):
                for alias in aliases:
                    if isinstance(alias, str) and alias.lower() in query_lower:
                        matched_names.append(name)
                        break

        if matched_names:
            return list(set(matched_names))

        # Second pass: use gemma4:e4b to extract entity names
        extract_prompt = (
            "Extrais UNIQUEMENT les noms propres (personnes, lieux, organisations, "
            "vehicules, telephones) mentionnes dans ce texte. "
            "Reponds avec une liste simple, un nom par ligne, RIEN d'autre.\n\n"
            f"Texte: {query[:2000]}"
        )
        try:
            raw = await self._router.route(
                TaskType.ENTITY_EXTRACTION, extract_prompt
            )
            # Parse simple line-based response
            names = [
                line.strip().lstrip("- ").strip()
                for line in raw.strip().splitlines()
                if line.strip() and not line.strip().startswith("#")
            ]
            # Filter out empty or too-short names
            return [n for n in names if len(n) >= 2][:10]
        except Exception as exc:
            logger.warning("Entity extraction from query failed: {}", exc)
            return []

    # ==================================================================
    # Deduplication (private)
    # ==================================================================

    def _deduplicate(self, results: list[dict]) -> list[dict]:
        """Merge results by evidence_id + chunk_text prefix, keep best scores.

        When a chunk appears in both semantic and graph results, merge
        the scores so the hybrid ranking benefits from both signals.
        """
        seen: dict[str, dict] = {}

        for r in results:
            # Build a dedup key from evidence_id + first 200 chars of text
            eid = r.get("evidence_id", "")
            text_prefix = r.get("chunk_text", "")[:200]
            key = f"{eid}::{text_prefix}"

            existing = seen.get(key)
            if existing is None:
                seen[key] = dict(r)
            else:
                # Merge: keep the maximum of each score component
                for score_key in ("_semantic_score", "_graph_score", "_fts_score", "_recency_score"):
                    existing[score_key] = max(
                        existing.get(score_key, 0.0),
                        r.get(score_key, 0.0),
                    )
                # Track combined source
                existing["_source"] = "hybrid"

        return list(seen.values())

    # ==================================================================
    # Reranking (private)
    # ==================================================================

    def _rerank(self, results: list[dict], n: int) -> list[dict]:
        """Rerank results by composite score and return top n.

        composite = semantic * 0.50 + graph * 0.25 + fts * 0.15 + recency * 0.10
        """
        for r in results:
            composite = (
                r.get("_semantic_score", 0.0) * _SEMANTIC_WEIGHT
                + r.get("_graph_score", 0.0) * _GRAPH_WEIGHT
                + r.get("_fts_score", 0.0) * _FTS_WEIGHT
                + r.get("_recency_score", 0.0) * _RECENCY_WEIGHT
            )
            r["_composite_score"] = composite

        # Sort descending by composite score
        results.sort(key=lambda x: x["_composite_score"], reverse=True)

        return results[:n]

    # ==================================================================
    # Recency scoring (private)
    # ==================================================================

    @staticmethod
    def _compute_recency_score(metadata: dict) -> float:
        """Compute a recency score in [0, 1] based on created_at.

        Items created within the last _RECENCY_HORIZON_DAYS days get
        a score linearly decaying from 1.0 (today) to 0.0 (horizon).
        Older items or items without timestamps get 0.0.
        """
        created_str = metadata.get("created_at") or metadata.get("ingestion_date", "")
        if not created_str:
            return 0.0

        try:
            # Handle various ISO-8601 formats
            if isinstance(created_str, datetime):
                created = created_str
            else:
                created_str = str(created_str).replace("Z", "+00:00")
                # Strip timezone info for naive comparison
                created = datetime.fromisoformat(created_str.split("+")[0])
        except (ValueError, TypeError):
            return 0.0

        now = datetime.now(timezone.utc)
        age = now - created
        horizon = timedelta(days=_RECENCY_HORIZON_DAYS)

        if age <= timedelta(0):
            return 1.0
        if age >= horizon:
            return 0.0

        return 1.0 - (age.total_seconds() / horizon.total_seconds())
