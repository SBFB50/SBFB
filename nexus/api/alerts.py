"""
NEXUS -- Alerts API router.

Endpoints for querying and managing investigation alerts.
"""

from __future__ import annotations

from typing import Optional

from fastapi import APIRouter, Depends, HTTPException, Query

from nexus.db.models import Alert
from nexus.db.sqlite_db import Database

from nexus.api.deps import get_database, paginated_response

router = APIRouter(tags=["alerts"])


# ------------------------------------------------------------------
# GET /api/cases/{case_id}/alerts — list alerts for a case
# ------------------------------------------------------------------

@router.get(
    "/api/cases/{case_id}/alerts",
)
async def list_alerts(
    case_id: str,
    severity: Optional[str] = None,
    unread_only: bool = False,
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    db: Database = Depends(get_database),
):
    """List alerts for a case, optionally filtered by severity and read status, with pagination."""
    all_rows = await db.list_alerts_by_case(
        case_id,
        unread_only=unread_only,
        severity=severity,
        limit=100_000,
    )

    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Alert(**r).model_dump(mode="json"),
    )


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
        # TODO: add a db.count_all_unread_alerts() method to avoid raw SQL
        # For now, access the connection directly as no public method exists
        cursor = await db._conn.execute(
            "SELECT COUNT(*) FROM alerts WHERE is_read = 0"
        )
        row = await cursor.fetchone()
        count = row[0] if row else 0

    return {"unread_count": count, "case_id": case_id}
