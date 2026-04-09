"""
NEXUS GOV -- Public API.

Read-only, rate-limited API for external consumers.
Prefix: /api/v1/gov/
"""
from __future__ import annotations

import time
from collections import defaultdict
from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from loguru import logger

from nexus.gov.db import GovernmentDatabase
from nexus.api.deps import get_government_database

router = APIRouter(prefix="/api/v1/gov", tags=["gov-public"])

# Simple in-memory rate limiter
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


@router.get(
    "/politicians",
    summary="List politicians",
    description="Paginated list of all tracked politicians with optional filters.",
)
async def public_list_politicians(
    request: Request,
    chamber: Optional[str] = Query(
        None, description="Filter: assemblee, senat, europe, gouvernement"
    ),
    party: Optional[str] = Query(
        None, description="Filter by party short name (RN, LFI, RE, etc.)"
    ),
    page: int = Query(1, ge=1, description="Page number"),
    limit: int = Query(50, ge=1, le=100, description="Items per page"),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    offset = (page - 1) * limit
    all_rows = await gov.list_politicians(chamber=chamber, party=party, limit=100_000)
    total = len(all_rows)
    page_data = all_rows[offset : offset + limit]
    return {
        "data": page_data,
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": (total + limit - 1) // limit,
        },
    }


@router.get("/politicians/{politician_id}", summary="Get politician detail")
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


@router.get("/politicians/{politician_id}/positions", summary="Politician positions")
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
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": (total + limit - 1) // limit,
        },
    }


@router.get(
    "/politicians/{politician_id}/contradictions",
    summary="Politician contradictions",
)
async def public_list_contradictions(
    request: Request,
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    return await gov.list_contradictions_by_politician(politician_id)


@router.get("/contradictions", summary="All contradictions")
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
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": (total + limit - 1) // limit,
        },
    }


@router.get("/votes", summary="Recent votes")
async def public_list_votes(
    request: Request,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    from nexus.db.sqlite_db import _row_to_dict, get_db

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
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": (total + limit - 1) // limit,
        },
    }


@router.get("/affairs", summary="Judicial affairs")
async def public_list_affairs(
    request: Request,
    page: int = Query(1, ge=1),
    limit: int = Query(50, ge=1, le=100),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    from nexus.db.sqlite_db import _row_to_dict, get_db

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
        "pagination": {
            "page": page,
            "limit": limit,
            "total": total,
            "totalPages": (total + limit - 1) // limit,
        },
    }


@router.get("/stats", summary="Global statistics")
async def public_stats(
    request: Request,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    return await gov.get_stats()


@router.get("/subjects", summary="All tracked subjects")
async def public_subjects(
    request: Request,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    _check_rate_limit(request)
    return await gov.get_subjects()
