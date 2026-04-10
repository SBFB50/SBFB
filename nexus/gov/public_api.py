"""
NEXUS GOV -- Public API.

Read-only, rate-limited API for external consumers.
Prefix: /api/v1/gov/
"""
from __future__ import annotations

import time
from collections import defaultdict
from typing import Any, Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from loguru import logger
from pydantic import BaseModel, Field

from nexus.gov.db import GovernmentDatabase
from nexus.api.deps import get_government_database

router = APIRouter(prefix="/api/v1/gov", tags=["GOV Public API"])

# ============================================================================
# Response models
# ============================================================================


class PaginationMeta(BaseModel):
    """Metadata de pagination."""

    page: int
    limit: int
    total: int
    total_pages: int = Field(alias="totalPages")

    model_config = {"populate_by_name": True}


class PoliticianPublic(BaseModel):
    """Representation publique d'un politicien."""

    id: str
    name: str
    party: Optional[str] = None
    chamber: Optional[str] = None
    role: Optional[str] = None
    constituency: Optional[str] = None
    position_count: int = 0
    contradiction_count: int = 0


class PoliticianListResponse(BaseModel):
    """Liste paginee de politiciens."""

    data: list[PoliticianPublic]
    pagination: PaginationMeta


class PoliticianDetailPublic(BaseModel):
    """Detail complet d'un politicien."""

    id: str
    name: str
    party: Optional[str] = None
    chamber: Optional[str] = None
    role: Optional[str] = None
    constituency: Optional[str] = None
    metadata: Optional[dict[str, Any]] = None


class PositionPublic(BaseModel):
    """Position politique publique."""

    id: str
    politician_id: str = ""
    subject: str = ""
    position_type: str = ""
    content: Optional[str] = None
    source_url: Optional[str] = None
    date: Optional[str] = None


class PositionListResponse(BaseModel):
    """Liste paginee de positions."""

    data: list[PositionPublic]
    pagination: PaginationMeta


class ContradictionPublic(BaseModel):
    """Contradiction politique publique."""

    id: str
    politician_id: str = ""
    subject: str = ""
    description: str = ""
    severity: str = ""
    detected_at: Optional[str] = None


class ContradictionListResponse(BaseModel):
    """Liste paginee de contradictions."""

    data: list[ContradictionPublic]
    pagination: PaginationMeta


class VotePublic(BaseModel):
    """Vote parlementaire public."""

    id: str
    politician_id: str = ""
    subject: str = ""
    position_type: str = ""
    content: Optional[str] = None
    date: Optional[str] = None


class VoteListResponse(BaseModel):
    """Liste paginee de votes."""

    data: list[VotePublic]
    pagination: PaginationMeta


class AffairPublic(BaseModel):
    """Affaire judiciaire publique."""

    id: str
    title: str = ""
    description: Optional[str] = None
    status: Optional[str] = None
    created_at: Optional[str] = None


class AffairListResponse(BaseModel):
    """Liste paginee d'affaires."""

    data: list[AffairPublic]
    pagination: PaginationMeta


class LawPublic(BaseModel):
    """Loi publique."""

    id: str
    title: str = ""
    status: Optional[str] = None
    date_initial: Optional[str] = None
    url: Optional[str] = None


class LawListResponse(BaseModel):
    """Liste paginee de lois."""

    data: list[LawPublic]
    pagination: PaginationMeta


class PressArticlePublic(BaseModel):
    """Article de presse public."""

    id: str
    title: str = ""
    url: Optional[str] = None
    source_name: Optional[str] = None
    published_at: Optional[str] = None
    summary: Optional[str] = None
    sentiment: Optional[str] = None


class PressListResponse(BaseModel):
    """Liste paginee d'articles de presse."""

    data: list[PressArticlePublic]
    pagination: PaginationMeta


class StatsPublic(BaseModel):
    """Statistiques globales du module GOV."""

    politicians: int = 0
    positions: int = 0
    contradictions: int = 0
    laws: int = 0
    affairs: int = 0
    press: int = 0
    social_posts: int = 0
    last_scan: Optional[str] = None


class SearchResultItem(BaseModel):
    """Resultat de recherche semantique."""

    id: str
    text: str = ""
    metadata: Optional[dict[str, Any]] = None
    score: float = 0.0


class SearchResponse(BaseModel):
    """Reponse de recherche."""

    query: str
    results: list[SearchResultItem]
    total: int


class SourceHealthItem(BaseModel):
    """Sante d'une source de donnees."""

    name: str
    url: str = ""
    status: str = "unknown"
    response_time_ms: float = 0.0
    consecutive_failures: int = 0
    error: str = ""


class HealthResponse(BaseModel):
    """Sante de l'API et des sources."""

    status: str
    sources: list[SourceHealthItem]


# ============================================================================
# Rate limiter
# ============================================================================

_rate_limits: dict[str, list[float]] = defaultdict(list)
_RATE_LIMIT = 60  # requests per minute per IP
_WINDOW = 60  # seconds
_CLEANUP_INTERVAL = 300  # purge stale IPs every 5 minutes
_last_cleanup: float = 0.0


def _check_rate_limit(request: Request) -> None:
    global _last_cleanup
    ip = request.client.host if request.client else "unknown"
    now = time.time()
    _rate_limits[ip] = [t for t in _rate_limits[ip] if t > now - _WINDOW]
    if len(_rate_limits[ip]) >= _RATE_LIMIT:
        raise HTTPException(429, "Rate limit exceeded. Max 60 requests/minute.")
    _rate_limits[ip].append(now)
    # Periodic cleanup: remove IPs with no recent requests to prevent unbounded growth
    if now - _last_cleanup > _CLEANUP_INTERVAL:
        _last_cleanup = now
        stale = [k for k, v in _rate_limits.items() if not v or v[-1] < now - _WINDOW]
        for k in stale:
            del _rate_limits[k]


def _paginate(page: int, limit: int, total: int) -> dict:
    """Build pagination metadata dict."""
    return {
        "page": page,
        "limit": limit,
        "total": total,
        "totalPages": (total + limit - 1) // limit,
    }


# ============================================================================
# Endpoints -- Politicians
# ============================================================================


@router.get(
    "/politicians",
    summary="Liste des politiciens",
    description="Liste paginee de tous les politiciens suivis avec filtres optionnels.",
    response_model=PoliticianListResponse,
    tags=["GOV Public API"],
)
async def public_list_politicians(
    request: Request,
    chamber: Optional[str] = Query(
        None, description="Filtre: assemblee, senat, europe, gouvernement"
    ),
    party: Optional[str] = Query(
        None, description="Filtre par parti (RN, LFI, RE, etc.)"
    ),
    page: int = Query(1, ge=1, description="Numero de page"),
    limit: int = Query(50, ge=1, le=100, description="Elements par page"),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    offset = (page - 1) * limit
    all_rows = await gov.list_politicians(chamber=chamber, party=party, limit=100_000)
    total = len(all_rows)
    page_data = all_rows[offset : offset + limit]
    return {
        "data": page_data,
        "pagination": _paginate(page, limit, total),
    }


@router.get(
    "/politicians/{politician_id}",
    summary="Detail d'un politicien",
    response_model=PoliticianDetailPublic,
    tags=["GOV Public API"],
)
async def public_get_politician(
    request: Request,
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    pol = await gov.get_politician(politician_id)
    if not pol:
        raise HTTPException(404, "Politician not found")
    return pol


@router.get(
    "/politicians/{politician_id}/positions",
    summary="Positions d'un politicien",
    response_model=PositionListResponse,
    tags=["GOV Public API"],
)
async def public_list_positions(
    request: Request,
    politician_id: str,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=200),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    offset = (page - 1) * limit
    rows = await gov.list_positions_by_politician(politician_id, limit=100_000)
    total = len(rows)
    return {
        "data": rows[offset : offset + limit],
        "pagination": _paginate(page, limit, total),
    }


@router.get(
    "/politicians/{politician_id}/contradictions",
    summary="Contradictions d'un politicien",
    tags=["GOV Public API"],
)
async def public_list_contradictions(
    request: Request,
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    return await gov.list_contradictions_by_politician(politician_id)


# ============================================================================
# Endpoints -- Contradictions
# ============================================================================


@router.get(
    "/contradictions",
    summary="Toutes les contradictions",
    response_model=ContradictionListResponse,
    tags=["GOV Public API"],
)
async def public_all_contradictions(
    request: Request,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    offset = (page - 1) * limit
    rows = await gov.list_all_contradictions(limit=100_000)
    total = len(rows)
    return {
        "data": rows[offset : offset + limit],
        "pagination": _paginate(page, limit, total),
    }


# ============================================================================
# Endpoints -- Votes
# ============================================================================


@router.get(
    "/votes",
    summary="Votes recents",
    response_model=VoteListResponse,
    tags=["GOV Public API"],
)
async def public_list_votes(
    request: Request,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    from nexus.engine import _row_to_dict, get_db

    async with get_db() as conn:
        cursor = await conn.execute(
            "SELECT * FROM gov_positions WHERE position_type = 'vote' "
            "ORDER BY date DESC LIMIT ? OFFSET ?",
            (limit, (page - 1) * limit),
        )
        rows = [_row_to_dict(r) for r in await cursor.fetchall()]
        cursor2 = await conn.execute(
            "SELECT COUNT(*) as cnt FROM gov_positions WHERE position_type = 'vote'"
        )
        total_row = await cursor2.fetchone()
        total = total_row[0] if total_row else 0
    return {
        "data": rows,
        "pagination": _paginate(page, limit, total),
    }


# ============================================================================
# Endpoints -- Affairs
# ============================================================================


@router.get(
    "/affairs",
    summary="Affaires judiciaires",
    response_model=AffairListResponse,
    tags=["GOV Public API"],
)
async def public_list_affairs(
    request: Request,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    from nexus.engine import _row_to_dict, get_db

    async with get_db() as conn:
        cursor = await conn.execute(
            "SELECT * FROM gov_affairs ORDER BY created_at DESC LIMIT ? OFFSET ?",
            (limit, (page - 1) * limit),
        )
        rows = [_row_to_dict(r) for r in await cursor.fetchall()]
        cursor2 = await conn.execute("SELECT COUNT(*) as cnt FROM gov_affairs")
        total = (await cursor2.fetchone())[0]
    return {
        "data": rows,
        "pagination": _paginate(page, limit, total),
    }


# ============================================================================
# Endpoints -- Laws
# ============================================================================


@router.get(
    "/laws",
    summary="Lois et projets de loi",
    description="Liste paginee des lois suivies, filtrable par statut.",
    response_model=LawListResponse,
    tags=["GOV Public API"],
)
async def public_laws(
    request: Request,
    page: int = Query(1, ge=1, description="Numero de page"),
    limit: int = Query(50, ge=1, le=100, description="Elements par page"),
    status: Optional[str] = Query(
        None, description="Filtre par statut (adopte, en_discussion, rejete, etc.)"
    ),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    # Fetch all matching laws then paginate in-memory (consistent with other endpoints)
    all_rows = await gov.list_laws(status=status, limit=100_000)
    total = len(all_rows)
    offset = (page - 1) * limit
    return {
        "data": all_rows[offset : offset + limit],
        "pagination": _paginate(page, limit, total),
    }


# ============================================================================
# Endpoints -- Press
# ============================================================================


@router.get(
    "/press",
    summary="Articles de presse",
    description="Liste paginee des articles de presse, filtrable par sentiment.",
    response_model=PressListResponse,
    tags=["GOV Public API"],
)
async def public_press(
    request: Request,
    page: int = Query(1, ge=1, description="Numero de page"),
    limit: int = Query(50, ge=1, le=100, description="Elements par page"),
    sentiment: Optional[str] = Query(
        None, description="Filtre par sentiment (positive, negative, neutral)"
    ),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    from nexus.engine import _row_to_dict, _dict_with_json_fields, get_db

    # Use direct query to support sentiment filtering with proper pagination
    async with get_db() as conn:
        conditions = []
        params: list[Any] = []
        if sentiment is not None:
            conditions.append("sentiment = ?")
            params.append(sentiment)
        where = ("WHERE " + " AND ".join(conditions)) if conditions else ""

        # Count
        count_q = f"SELECT COUNT(*) FROM gov_press {where}"
        cursor_c = await conn.execute(count_q, params)
        total = (await cursor_c.fetchone())[0]

        # Data
        offset = (page - 1) * limit
        data_q = f"SELECT * FROM gov_press {where} ORDER BY published_at DESC, created_at DESC LIMIT ? OFFSET ?"
        cursor_d = await conn.execute(data_q, params + [limit, offset])
        rows = [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in await cursor_d.fetchall()]

    return {
        "data": rows,
        "pagination": _paginate(page, limit, total),
    }


# ============================================================================
# Endpoints -- Stats
# ============================================================================


@router.get(
    "/stats",
    summary="Statistiques globales",
    description="Compteurs agreges pour toutes les tables du module GOV.",
    response_model=StatsPublic,
    tags=["GOV Public API"],
)
async def public_stats(
    request: Request,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    return await gov.get_stats()


# ============================================================================
# Endpoints -- Subjects
# ============================================================================


@router.get(
    "/subjects",
    summary="Sujets suivis",
    description="Liste de tous les sujets politiques distincts.",
    tags=["GOV Public API"],
)
async def public_subjects(
    request: Request,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> list[str]:
    _check_rate_limit(request)
    return await gov.get_subjects()


# ============================================================================
# Endpoints -- Search
# ============================================================================


@router.get(
    "/search",
    summary="Recherche semantique",
    description="Recherche semantique (RAG) ou lexicale (FTS5 fallback) sur les donnees politiques.",
    response_model=SearchResponse,
    tags=["GOV Public API"],
)
async def public_search(
    request: Request,
    q: str = Query(..., min_length=2, description="Requete de recherche"),
    limit: int = Query(10, ge=1, le=50, description="Nombre de resultats"),
):
    _check_rate_limit(request)

    # Try semantic search via GovRAG first
    from nexus.gov.rag import GovRAG

    chroma = getattr(request.app.state, "chroma", None)
    router_llm = getattr(request.app.state, "router", None)
    rag = GovRAG(chroma=chroma, router=router_llm)
    results = await rag.search(q, n_results=limit)

    # Fallback to FTS5 politician search if RAG returns nothing
    if not results:
        from nexus.api.deps import get_government_database as _get_gov
        from nexus.engine import get_db

        async with get_db() as conn:
            gov = GovernmentDatabase(conn)
            politicians = await gov.search_politicians(q)
            results = [
                {
                    "id": p.get("id", ""),
                    "text": p.get("name", ""),
                    "metadata": {
                        "type": "politician",
                        "party": p.get("party"),
                        "chamber": p.get("chamber"),
                    },
                    "score": 1.0,
                }
                for p in politicians[:limit]
            ]

    return {
        "query": q,
        "results": results,
        "total": len(results),
    }


# ============================================================================
# Endpoints -- Health
# ============================================================================


@router.get(
    "/health",
    summary="Sante de l'API et des sources",
    description="Etat de sante de l'API publique et des sources de donnees surveillees.",
    response_model=HealthResponse,
    tags=["GOV Public API"],
)
async def public_health(request: Request):
    _check_rate_limit(request)

    monitor = getattr(request.app.state, "gov_health_monitor", None)
    if monitor is not None:
        sources = monitor.get_status()
    else:
        sources = []

    # Determine aggregate status
    if not sources:
        overall = "unknown"
    elif all(s.get("status") == "healthy" for s in sources):
        overall = "healthy"
    elif any(s.get("status") == "down" for s in sources):
        overall = "degraded"
    else:
        overall = "operational"

    return {
        "status": overall,
        "sources": sources,
    }
