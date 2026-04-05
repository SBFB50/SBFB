"""
NEXUS -- Investigation API router.

Controls autonomous investigation loops for cases:
- Start/stop investigation per case
- View investigation status
- View autonomous action log (analysis_runs with trigger='autonomous_loop')
"""

from __future__ import annotations

from fastapi import APIRouter, Depends, HTTPException, Request
from loguru import logger

from nexus.api.deps import get_database
from nexus.db.sqlite_db import Database

router = APIRouter(tags=["investigation"])


# ------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------

def _get_manager(request: Request):
    """Retrieve the InvestigationManager from app.state."""
    mgr = getattr(request.app.state, "investigation_manager", None)
    if mgr is None:
        raise HTTPException(
            status_code=503,
            detail="Investigation manager is not available",
        )
    return mgr


# ------------------------------------------------------------------
# GET /api/investigations -- Status of all active investigations
# ------------------------------------------------------------------

@router.get("/api/investigations")
async def list_investigations(request: Request) -> dict:
    """Return status of all active autonomous investigations."""
    mgr = _get_manager(request)
    return mgr.get_status()


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/investigation/start
# ------------------------------------------------------------------

@router.post("/api/cases/{case_id}/investigation/start")
async def start_investigation(
    case_id: str,
    request: Request,
    db: Database = Depends(get_database),
) -> dict:
    """Start autonomous investigation for a case."""
    # Verify the case exists
    case = await db.get_case(case_id)
    if case is None:
        raise HTTPException(
            status_code=404, detail=f"Case not found: {case_id}"
        )

    mgr = _get_manager(request)
    started = await mgr.start_investigation(case_id)

    if started:
        logger.info("API: Started investigation for case {}", case_id)
        return {"status": "started", "case_id": case_id}
    else:
        return {"status": "already_running", "case_id": case_id}


# ------------------------------------------------------------------
# POST /api/cases/{case_id}/investigation/stop
# ------------------------------------------------------------------

@router.post("/api/cases/{case_id}/investigation/stop")
async def stop_investigation(
    case_id: str,
    request: Request,
) -> dict:
    """Stop autonomous investigation for a case."""
    mgr = _get_manager(request)
    stopped = await mgr.stop_investigation(case_id)

    if stopped:
        logger.info("API: Stopped investigation for case {}", case_id)
        return {"status": "stopped", "case_id": case_id}
    else:
        return {"status": "not_running", "case_id": case_id}


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/investigation/status
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/investigation/status")
async def get_investigation_status(
    case_id: str,
    request: Request,
) -> dict:
    """Return detailed status for a case investigation."""
    mgr = _get_manager(request)
    status = mgr.get_investigation_status(case_id)

    if status is None:
        return {
            "case_id": case_id,
            "running": False,
            "cycle_count": 0,
            "last_action": None,
            "last_cycle_at": None,
            "started_at": None,
        }

    return status


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/investigation/log
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/investigation/log")
async def get_investigation_log(
    case_id: str,
    limit: int = 50,
    db: Database = Depends(get_database),
) -> list[dict]:
    """Return the journal of autonomous actions for a case.

    Fetches analysis_runs where trigger='autonomous_loop' or
    run_type='self_questioning'.
    """
    # Get all analysis runs for this case and filter for autonomous ones
    all_runs = await db.list_runs_by_case(case_id, limit=limit * 2)

    autonomous_runs = [
        r
        for r in all_runs
        if r.get("trigger") == "autonomous_loop"
        or r.get("run_type") == "self_questioning"
    ]

    return autonomous_runs[:limit]
