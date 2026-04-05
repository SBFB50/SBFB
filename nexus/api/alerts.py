"""
NEXUS -- Alerts API router.

Endpoints for querying and managing investigation alerts.
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, Depends, HTTPException

from nexus.db.models import Alert
from nexus.db.sqlite_db import Database

from nexus.api.deps import get_database

router = APIRouter(tags=["alerts"])


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/alerts — list alerts for a case
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/alerts",
    response_model=list[Alert],
)
async def list_alerts(
    case_id: str,
    severity: Optional[str] = None,
    unread_only: bool = False,
    limit: int = 100,
    db: Database = Depends(get_database),
) -> list[Alert]:
    """List alerts for a case, optionally filtered by severity and read status."""
    rows = await db.list_alerts_by_case(
        case_id,
        unread_only=unread_only,
        severity=severity,
        limit=limit,
    )
    return [Alert(**r) for r in rows]


# ------------------------------------------------------------------
# PUT /api/alerts/{alert_id}/read — mark an alert as read
# ------------------------------------------------------------------

@router.put("/api/alerts/{alert_id}/read")
async def mark_alert_read(
    alert_id: str,
    db: Database = Depends(get_database),
) -> dict:
    """Mark a single alert as read."""
    success = await db.mark_alert_read(alert_id)
    if not success:
        raise HTTPException(
            status_code=404,
            detail=f"Alert not found: {alert_id}",
        )
    return {"alert_id": alert_id, "is_read": True}


# ------------------------------------------------------------------
# GET /api/alerts/unread-count — count unread alerts
# ------------------------------------------------------------------

@router.get("/api/alerts/unread-count")
async def unread_alert_count(
    case_id: Optional[str] = None,
    db: Database = Depends(get_database),
) -> dict:
    """Get the number of unread alerts.

    If ``case_id`` is provided, counts only for that case.
    Otherwise counts across all cases.
    """
    if case_id:
        count = await db.count_unread_alerts(case_id)
    else:
        # Count across all cases -- use a direct query
        cursor = await db._conn.execute(
            "SELECT COUNT(*) FROM alerts WHERE is_read = 0"
        )
        row = await cursor.fetchone()
        count = row[0] if row else 0

    return {"unread_count": count, "case_id": case_id}
