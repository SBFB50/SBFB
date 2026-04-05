"""
NEXUS -- Audit Trail API router.

Exposes the investigation audit log for querying from the dashboard.

GET  /api/cases/{case_id}/audit           -- full log (filterable)
GET  /api/cases/{case_id}/audit/summary   -- action type counts
GET  /api/cases/{case_id}/audit/timeline  -- chronological timeline
GET  /api/audit/{audit_id}                -- single entry detail
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Depends, Query

from nexus.api.deps import get_database
from nexus.db.sqlite_db import Database

router = APIRouter(tags=["audit"])


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/audit
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/audit")
async def list_audit_log(
    case_id: str,
    action: Optional[str] = Query(None, description="Filter by action type"),
    actor: Optional[str] = Query(None, description="Filter by actor"),
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
) -> List[Dict[str, Any]]:
    """Return the audit log for a case with optional filters."""
    return await db.list_audit_log(
        case_id, action=action, actor=actor, limit=limit, offset=offset,
    )


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/audit/summary
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/audit/summary")
async def audit_summary(
    case_id: str,
    db: Database = Depends(get_database),
) -> Dict[str, Any]:
    """Return counts grouped by action type for the case audit log."""
    total = await db.count_audit_entries(case_id)

    # Count per action type
    action_types = [
        "evidence_added",
        "hypothesis_scored",
        "hypothesis_created",
        "entity_discovered",
        "contradiction_found",
        "monitoring_result",
        "query_generated",
        "evidence_ingested_auto",
        "self_questioning",
        "analysis_started",
        "analysis_completed",
        "investigation_started",
        "investigation_stopped",
    ]
    counts: Dict[str, int] = {}
    for action in action_types:
        c = await db.count_audit_entries(case_id, action=action)
        if c > 0:
            counts[action] = c

    return {
        "case_id": case_id,
        "total": total,
        "by_action": counts,
    }


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/audit/timeline
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/audit/timeline")
async def audit_timeline(
    case_id: str,
    db: Database = Depends(get_database),
) -> List[Dict[str, Any]]:
    """Return the full audit trail sorted chronologically (oldest first)."""
    return await db.get_investigation_timeline(case_id)


# ------------------------------------------------------------------
# GET /api/audit/{audit_id}
# ------------------------------------------------------------------

@router.get("/api/audit/{audit_id}")
async def get_audit_entry(
    audit_id: str,
    db: Database = Depends(get_database),
) -> Optional[Dict[str, Any]]:
    """Return a single audit entry by ID."""
    entry = await db.get_audit_entry(audit_id)
    if entry is None:
        from fastapi import HTTPException
        raise HTTPException(status_code=404, detail="Audit entry not found")
    return entry


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/audit/verify
# ------------------------------------------------------------------

@router.get("/api/cases/{case_id}/audit/verify")
async def verify_audit_chain(
    case_id: str,
    db: Database = Depends(get_database),
) -> Dict[str, Any]:
    """Verify the hash chain integrity of the audit log.

    Returns whether the chain is intact or if tampering was detected.
    """
    from nexus.core.audit import AuditService
    audit = AuditService(db)
    return await audit.verify_chain(case_id)
