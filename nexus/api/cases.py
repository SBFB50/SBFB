"""
NEXUS -- Cases API router.

CRUD endpoints for investigation cases + aggregate statistics.
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException

from nexus.core.case_manager import CaseManager
from nexus.db.models import Case, CaseCreate, CaseUpdate

from nexus.api.deps import get_case_manager

router = APIRouter(prefix="/api/cases", tags=["cases"])


# ------------------------------------------------------------------
# POST /api/cases
# ------------------------------------------------------------------

@router.post("", response_model=Case, status_code=201)
async def create_case(
    data: CaseCreate,
    mgr: CaseManager = Depends(get_case_manager),
) -> Case:
    """Create a new investigation case."""
    return await mgr.create_case(data)


# ------------------------------------------------------------------
# GET /api/cases
# ------------------------------------------------------------------

@router.get("", response_model=list[Case])
async def list_cases(
    status: str | None = None,
    mgr: CaseManager = Depends(get_case_manager),
) -> list[Case]:
    """List all cases, optionally filtered by status."""
    return await mgr.list_cases(status=status)


# ------------------------------------------------------------------
# GET /api/cases/{case_id}
# ------------------------------------------------------------------

@router.get("/{case_id}", response_model=Case)
async def get_case(
    case_id: str,
    mgr: CaseManager = Depends(get_case_manager),
) -> Case:
    """Retrieve a single case by ID."""
    try:
        return await mgr.get_case(case_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))


# ------------------------------------------------------------------
# PUT /api/cases/{case_id}
# ------------------------------------------------------------------

@router.put("/{case_id}", response_model=Case)
async def update_case(
    case_id: str,
    data: CaseUpdate,
    mgr: CaseManager = Depends(get_case_manager),
) -> Case:
    """Update an existing case (partial update)."""
    try:
        return await mgr.update_case(case_id, data)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))


# ------------------------------------------------------------------
# DELETE /api/cases/{case_id}
# ------------------------------------------------------------------

@router.delete("/{case_id}", status_code=204)
async def delete_case(
    case_id: str,
    mgr: CaseManager = Depends(get_case_manager),
) -> None:
    """Delete a case and all dependent data (cascade)."""
    try:
        await mgr.delete_case(case_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/stats
# ------------------------------------------------------------------

@router.get("/{case_id}/stats")
async def get_case_stats(
    case_id: str,
    mgr: CaseManager = Depends(get_case_manager),
) -> dict:
    """Return aggregate statistics for a case."""
    try:
        return await mgr.get_case_stats(case_id)
    except ValueError as exc:
        raise HTTPException(status_code=404, detail=str(exc))
