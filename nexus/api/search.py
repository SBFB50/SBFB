"""
NEXUS -- Search API router (ChromaDB vector search).

Exposes semantic search capabilities over the investigation data:
- Full-text semantic search (evidence or entities)
- Unified cross-collection search
- Similar evidence discovery
- Near-duplicate detection
- Collection stats and diagnostics
- Batch re-indexing
"""

from __future__ import annotations

from typing import Any, Dict, List, Literal, Optional, Tuple

from fastapi import APIRouter, Depends, HTTPException, Query
from pydantic import BaseModel, Field

from nexus.api.deps import get_chroma, get_llm_router
from nexus.db.chroma_db import ChromaClient
from nexus.llm.router import LLMRouter

router = APIRouter(prefix="/api", tags=["search"])


# ------------------------------------------------------------------
# Request / response schemas
# ------------------------------------------------------------------

class SearchRequest(BaseModel):
    """Body for POST /api/cases/{case_id}/search."""
    query: str
    n_results: int = Field(default=10, ge=1, le=100)
    collection: Literal["evidence", "entities"] = "evidence"


class UnifiedSearchRequest(BaseModel):
    """Body for POST /api/cases/{case_id}/search/unified."""
    query: str
    n_per_collection: int = Field(default=5, ge=1, le=50)
    collections: Optional[List[str]] = Field(
        default=None,
        description=(
            "Collections to search. Defaults to evidence_chunks, "
            "entity_contexts, monitoring_results."
        ),
    )


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/search
# ------------------------------------------------------------------

@router.post("/cases/{case_id}/search")
async def semantic_search(
    case_id: str,
    body: SearchRequest,
    chroma: ChromaClient = Depends(get_chroma),
    llm: LLMRouter = Depends(get_llm_router),
) -> Dict[str, Any]:
    """Semantic search over evidence or entities for a given case.

    The query text is embedded via nomic-embed-text, then matched
    against the chosen ChromaDB collection.
    """
    # Embed the query text
    query_embedding = await llm.embed(body.query)

    if body.collection == "evidence":
        results = chroma.search_evidence(
            case_id=case_id,
            query_embedding=query_embedding,
            n_results=body.n_results,
        )
    else:
        results = chroma.search_entities(
            case_id=case_id,
            query_embedding=query_embedding,
            n_results=body.n_results,
        )

    return {
        "query": body.query,
        "collection": body.collection,
        "count": len(results),
        "results": results,
    }


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/search/unified
# ------------------------------------------------------------------

@router.post("/cases/{case_id}/search/unified")
async def unified_search(
    case_id: str,
    body: UnifiedSearchRequest,
    chroma: ChromaClient = Depends(get_chroma),
    llm: LLMRouter = Depends(get_llm_router),
) -> Dict[str, Any]:
    """Cross-collection semantic search for a given case.

    Searches across multiple ChromaDB collections (evidence_chunks,
    entity_contexts, monitoring_results by default) and returns
    merged results sorted by similarity.
    """
    query_embedding = await llm.embed(body.query)

    results = await chroma.unified_search(
        query_embedding=query_embedding,
        case_id=case_id,
        collections=body.collections,
        n_per_collection=body.n_per_collection,
    )

    return {
        "query": body.query,
        "collections": body.collections or [
            "evidence_chunks", "entity_contexts", "monitoring_results",
        ],
        "count": len(results),
        "results": results,
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/similar/{evidence_id}
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/similar/{evidence_id}")
async def find_similar(
    case_id: str,
    evidence_id: str,
    n_results: int = Query(default=5, ge=1, le=50),
    chroma: ChromaClient = Depends(get_chroma),
) -> Dict[str, Any]:
    """Find evidence items semantically similar to an existing one.

    Uses the stored embedding of the target evidence to query for
    neighbours in the same case.
    """
    results = chroma.find_similar_evidence(
        evidence_id=evidence_id,
        case_id=case_id,
        n_results=n_results,
    )
    return {
        "source_id": evidence_id,
        "count": len(results),
        "results": results,
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/duplicates
# ------------------------------------------------------------------

@router.get("/cases/{case_id}/duplicates")
async def find_duplicates(
    case_id: str,
    threshold: float = Query(default=0.92, ge=0.5, le=1.0),
    chroma: ChromaClient = Depends(get_chroma),
) -> Dict[str, Any]:
    """Detect near-duplicate evidence pairs within a case.

    Returns pairs where cosine similarity >= threshold.
    Warning: O(n^2) -- intended for moderate collection sizes.
    """
    duplicates: List[Tuple[str, str, float]] = chroma.find_duplicates(
        case_id=case_id,
        threshold=threshold,
    )
    return {
        "threshold": threshold,
        "count": len(duplicates),
        "pairs": [
            {"id_a": a, "id_b": b, "similarity": sim}
            for a, b, sim in duplicates
        ],
    }


# ------------------------------------------------------------------
# GET /api/search/stats
# ------------------------------------------------------------------

@router.get("/search/stats")
async def search_stats(
    chroma: ChromaClient = Depends(get_chroma),
) -> Dict[str, Any]:
    """Return detailed stats for all ChromaDB collections.

    Includes item counts for every collection (evidence_chunks,
    entity_contexts, monitoring_results, hypothesis_reasoning,
    image_dinov2, image_clip) plus a summary with totals.
    """
    stats = chroma.get_detailed_stats()
    return stats


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/search/reindex
# ------------------------------------------------------------------

@router.post("/cases/{case_id}/search/reindex")
async def reindex_case(
    case_id: str,
    chroma: ChromaClient = Depends(get_chroma),
    llm: LLMRouter = Depends(get_llm_router),
) -> Dict[str, Any]:
    """Re-embed all evidence for a case.

    Useful after embedding model change or parameter tuning.
    Re-chunks and re-embeds all evidence into evidence_chunks.
    """
    total_chunks = await chroma.reindex_case(case_id=case_id, router=llm)
    return {
        "case_id": case_id,
        "chunks_indexed": total_chunks,
    }
