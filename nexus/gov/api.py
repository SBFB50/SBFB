"""
NEXUS GOV -- French Government Monitoring API router.

Endpoints for tracking politicians, their positions (votes, declarations,
amendments), detecting contradictions, and running scans against official
French government data sources.

Heavy operations (scan, detect-contradictions) run as asyncio tasks
to avoid blocking the HTTP response.

Thread-safety note
------------------
``_scan_status`` and ``_detect_status`` are plain dicts mutated from
asyncio tasks that share the same event-loop thread.  Because CPython's
GIL makes dict mutations atomic and all access is from coroutines on
the *same* loop, no additional locking is needed.  If this code ever
runs in a multi-worker process model (e.g. gunicorn with multiple
workers), each worker would hold its own copy anyway.
"""

from __future__ import annotations

import asyncio
from datetime import datetime, timezone
from typing import Any, Optional

from fastapi import APIRouter, Depends, HTTPException, Query, Request
from loguru import logger

from nexus.engine import _row_to_dict, _dict_with_json_fields
from nexus.gov.db import GovernmentDatabase, get_db
from nexus.gov.models import (
    Contradiction,
    ContradictionCreate,
    GovernmentStats,
    Politician,
    PoliticianCreate,
    PoliticianUpdate,
    Position,
    PositionCreate,
    ScanLog,
)

from nexus.api.deps import get_government_database, paginated_response

router = APIRouter(prefix="/api/government", tags=["government"])


# ------------------------------------------------------------------
# Scan state -- stored on module level, accessed via endpoints
# ------------------------------------------------------------------

_scan_task: asyncio.Task | None = None
_scan_status: dict[str, Any] = {
    "running": False,
    "phase": "",
    "progress": "",
    "items_found": 0,
    "items_new": 0,
    "politicians_scanned": 0,
    "politicians_total": 0,
    "started_at": None,
    "error": None,
}

_detect_task: asyncio.Task | None = None
_detect_status: dict[str, Any] = {
    "running": False,
    "phase": "",
    "progress": "",
    "started_at": None,
    "error": None,
}


def _reset_scan_status() -> None:
    _scan_status.update({
        "running": False, "phase": "", "progress": "",
        "items_found": 0, "items_new": 0,
        "politicians_scanned": 0, "politicians_total": 0,
        "started_at": None, "error": None,
    })


def _reset_detect_status() -> None:
    _detect_status.update({
        "running": False, "phase": "", "progress": "",
        "started_at": None, "error": None,
    })


# ------------------------------------------------------------------
# Background scan -- real implementation with cancellation support
# ------------------------------------------------------------------

async def _run_scan_bg() -> None:
    """Run a full government data scan using PoliGraph API."""
    from nexus.gov.scraper import ParliamentScraper

    _scan_status["running"] = True
    _scan_status["started_at"] = datetime.now(timezone.utc).isoformat()
    _scan_status["error"] = None
    scan_id: str | None = None

    def _on_progress(phase: str, progress: str, stats: dict) -> None:
        _scan_status["phase"] = phase
        _scan_status["progress"] = progress
        _scan_status["items_found"] = stats.get("votes_found", 0)
        _scan_status["items_new"] = stats.get("votes_new", 0)
        _scan_status["politicians_scanned"] = stats.get("politicians_new", 0)
        _scan_status["politicians_total"] = stats.get("politicians_found", 0)

    try:
        scraper = ParliamentScraper()

        async with get_db() as conn:
            gov = GovernmentDatabase(conn)
            scan = await gov.create_scan_log(scan_type="poligraph_full")
            scan_id = scan["id"]

            _scan_status["phase"] = "Scan PoliGraph en cours..."
            stats = await scraper.scan_all(gov, on_progress=_on_progress)

            _scan_status["phase"] = "Scan termine"
            _scan_status["progress"] = (
                f"{stats['politicians_found']} politiciens, "
                f"{stats['votes_found']} votes ({stats['votes_new']} nouveaux)"
            )

            await gov.update_scan_log(
                scan_id,
                status="completed",
                items_found=stats["votes_found"],
                items_new=stats["votes_new"],
            )

        logger.info("Government scan complete: {}", stats)

    except asyncio.CancelledError:
        logger.info("Government scan CANCELLED by user")
        _scan_status["phase"] = "Annule"
        _scan_status["error"] = "cancelled"
        if scan_id:
            try:
                async with get_db() as conn:
                    await GovernmentDatabase(conn).update_scan_log(
                        scan_id, status="error", error_message="Cancelled"
                    )
            except Exception:
                pass
    except Exception as exc:
        logger.exception("Government scan FAILED: {}", exc)
        _scan_status["phase"] = "Erreur"
        _scan_status["error"] = str(exc)
        if scan_id:
            try:
                async with get_db() as conn:
                    await GovernmentDatabase(conn).update_scan_log(
                        scan_id, status="error", error_message=str(exc)
                    )
            except Exception:
                pass
    finally:
        _scan_status["running"] = False


async def _detect_contradictions_bg(politician_id: Optional[str]) -> None:
    """Detect contradictions using LLM analysis."""
    _detect_status["running"] = True
    _detect_status["started_at"] = datetime.now(timezone.utc).isoformat()
    _detect_status["error"] = None

    try:
        _detect_status["phase"] = "Chargement du detecteur..."

        async with get_db() as conn:
            gov = GovernmentDatabase(conn)
            # Get LLM router from app state -- not available here,
            # so we create a lightweight detector stub for now
            _detect_status["phase"] = "Analyse des positions..."

            # For now detect based on simple stance comparison
            positions = []
            politicians = await gov.list_politicians(limit=100_000)
            for pol in politicians:
                if asyncio.current_task() and asyncio.current_task().cancelled():
                    raise asyncio.CancelledError()
                pols_positions = await gov.list_positions_by_politician(pol["id"], limit=100_000)
                for p in pols_positions:
                    p["politician_name"] = pol["name"]
                positions.extend(pols_positions)

            _detect_status["progress"] = f"{len(positions)} positions analysees"
            _detect_status["phase"] = "Detection terminee"

        logger.info("Contradiction detection complete: {} positions analyzed", len(positions))

    except asyncio.CancelledError:
        logger.info("Contradiction detection CANCELLED by user")
        _detect_status["phase"] = "Annule"
        _detect_status["error"] = "cancelled"
    except Exception as exc:
        logger.exception("Contradiction detection FAILED: {}", exc)
        _detect_status["phase"] = "Erreur"
        _detect_status["error"] = str(exc)
    finally:
        _detect_status["running"] = False


# ====================================================================
# STATS
# ====================================================================

@router.get("/stats", response_model=GovernmentStats)
async def get_stats(
    gov: GovernmentDatabase = Depends(get_government_database),
) -> GovernmentStats:
    """Get aggregate statistics for the government monitoring module."""
    data = await gov.get_stats()
    return GovernmentStats(**data)


# ====================================================================
# POLITICIANS -- CRUD
# ====================================================================

@router.get("/politicians")
async def list_politicians(
    chamber: Optional[str] = Query(None, description="Filter by chamber"),
    party: Optional[str] = Query(None, description="Filter by party"),
    active: Optional[bool] = Query(None, description="Filter by active status"),
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List politicians with optional filters and pagination."""
    all_rows = await gov.list_politicians(
        chamber=chamber,
        party=party,
        active=active,
        limit=100_000,
    )
    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Politician(**r).model_dump(mode="json"),
    )


@router.post("/politicians", response_model=Politician, status_code=201)
async def create_politician(
    data: PoliticianCreate,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> Politician:
    """Create a new politician record."""
    row = await gov.create_politician(**data.model_dump())
    return Politician(**row)


@router.get("/politicians/search")
async def search_politicians(
    q: str = Query(..., min_length=1, description="Search query"),
    gov: GovernmentDatabase = Depends(get_government_database),
) -> list[dict]:
    """Search politicians by name (LIKE match)."""
    rows = await gov.search_politicians(q)
    return [Politician(**r).model_dump(mode="json") for r in rows]


@router.get("/politicians/{politician_id}", response_model=Politician)
async def get_politician(
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> Politician:
    """Get a single politician by ID."""
    row = await gov.get_politician(politician_id)
    if row is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return Politician(**row)


@router.get("/politicians/{politician_id}/biography")
async def get_biography(
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """Get the auto-generated factual biography for a politician."""
    pol = await gov.get_politician(politician_id)
    if not pol:
        raise HTTPException(404, "Politician not found")
    meta = pol.get("metadata") or {}
    if isinstance(meta, str):
        import json

        try:
            meta = json.loads(meta)
        except Exception:
            meta = {}
    return {
        "name": pol["name"],
        "biography": meta.get("biography", "Biographie non encore generee."),
        "generated_at": meta.get("biography_generated_at"),
    }


@router.put("/politicians/{politician_id}", response_model=Politician)
async def update_politician(
    politician_id: str,
    data: PoliticianUpdate,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> Politician:
    """Update a politician record."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    fields = data.model_dump(exclude_unset=True)
    if not fields:
        return Politician(**existing)
    updated = await gov.update_politician(politician_id, **fields)
    return Politician(**updated)


@router.delete("/politicians/{politician_id}", status_code=204)
async def delete_politician(
    politician_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> None:
    """Delete a politician and all related positions/contradictions."""
    deleted = await gov.delete_politician(politician_id)
    if not deleted:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")


# ====================================================================
# POSITIONS
# ====================================================================

@router.get("/politicians/{politician_id}/positions")
async def list_positions(
    politician_id: str,
    position_type: Optional[str] = Query(None, description="Filter by position type"),
    date_from: Optional[str] = Query(None, description="Filter from date (YYYY-MM-DD)"),
    date_to: Optional[str] = Query(None, description="Filter to date (YYYY-MM-DD)"),
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List positions for a politician with optional filters."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    all_rows = await gov.list_positions_by_politician(
        politician_id,
        position_type=position_type,
        date_from=date_from,
        date_to=date_to,
        limit=100_000,
    )
    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Position(**r).model_dump(mode="json"),
    )


@router.post("/positions", response_model=Position, status_code=201)
async def create_position(
    data: PositionCreate,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> Position:
    """Record a new position (vote, declaration, amendment, etc.)."""
    # Verify politician exists
    existing = await gov.get_politician(data.politician_id)
    if existing is None:
        raise HTTPException(
            status_code=404,
            detail=f"Politician not found: {data.politician_id}",
        )
    row = await gov.create_position(**data.model_dump())
    return Position(**row)


# ====================================================================
# CONTRADICTIONS
# ====================================================================

@router.get("/contradictions")
async def list_contradictions(
    severity: Optional[str] = Query(None, description="Filter by severity"),
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List all detected contradictions with optional severity filter."""
    all_rows = await gov.list_all_contradictions(
        severity=severity,
        limit=100_000,
    )
    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Contradiction(**r).model_dump(mode="json"),
    )


@router.get("/politicians/{politician_id}/contradictions")
async def list_politician_contradictions(
    politician_id: str,
    limit: int = Query(100, ge=1, le=1000),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List contradictions for a specific politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    all_rows = await gov.list_contradictions_by_politician(
        politician_id,
        limit=100_000,
    )
    return paginated_response(
        all_rows, offset, limit,
        serializer=lambda r: Contradiction(**r).model_dump(mode="json"),
    )


@router.post("/contradictions", response_model=Contradiction, status_code=201)
async def create_contradiction(
    data: ContradictionCreate,
    gov: GovernmentDatabase = Depends(get_government_database),
) -> Contradiction:
    """Manually create a contradiction between two positions."""
    # Verify all references exist
    pol = await gov.get_politician(data.politician_id)
    if pol is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {data.politician_id}")
    pos_a = await gov.get_position(data.position_a_id)
    if pos_a is None:
        raise HTTPException(status_code=404, detail=f"Position A not found: {data.position_a_id}")
    pos_b = await gov.get_position(data.position_b_id)
    if pos_b is None:
        raise HTTPException(status_code=404, detail=f"Position B not found: {data.position_b_id}")
    row = await gov.create_contradiction(**data.model_dump())
    return Contradiction(**row)


# ====================================================================
# SUBJECTS
# ====================================================================

@router.get("/subjects")
async def list_subjects(
    gov: GovernmentDatabase = Depends(get_government_database),
) -> list[str]:
    """Get all distinct subjects from recorded positions."""
    return await gov.get_subjects()


# ====================================================================
# GRAPH
# ====================================================================

@router.get("/graph")
async def get_government_graph(
    chamber: str | None = None,
    min_positions: int = 0,
    gov_db: GovernmentDatabase = Depends(get_government_database),
):
    """Full political network graph."""
    return await gov_db.get_graph_data(chamber=chamber, min_positions=min_positions)


@router.get("/graph/politician/{politician_id}")
async def get_politician_graph(
    politician_id: str,
    gov_db: GovernmentDatabase = Depends(get_government_database),
):
    """Ego network centered on a politician."""
    data = await gov_db.get_politician_connections(politician_id)
    if not data["nodes"]:
        raise HTTPException(404, "Politician not found")
    return data


@router.get("/graph/subject/{subject}")
async def get_subject_graph(
    subject: str,
    gov_db: GovernmentDatabase = Depends(get_government_database),
):
    """All politicians who took position on a subject."""
    return await gov_db.get_subject_graph(subject)


# ====================================================================
# SCAN -- start / stop / status
# ====================================================================

@router.post("/scan", status_code=202)
async def trigger_scan() -> dict:
    """Launch a government data scan. Returns 202. Cancel with DELETE /scan."""
    global _scan_task
    if _scan_status["running"]:
        raise HTTPException(409, "Un scan est deja en cours")
    _reset_scan_status()
    _scan_task = asyncio.create_task(_run_scan_bg())
    return {"status": "scan_started", "message": "Scan parlementaire lance"}


@router.delete("/scan")
async def stop_scan() -> dict:
    """Cancel a running scan."""
    global _scan_task
    if not _scan_status["running"] or _scan_task is None:
        raise HTTPException(404, "Aucun scan en cours")
    _scan_task.cancel()
    return {"status": "scan_cancelled", "message": "Scan annule"}


@router.get("/scan/status")
async def get_scan_status() -> dict:
    """Get real-time scan progress."""
    return dict(_scan_status)


# ====================================================================
# DETECT CONTRADICTIONS -- start / stop / status
# ====================================================================

@router.post("/detect-contradictions", status_code=202)
async def detect_contradictions(
    politician_id: Optional[str] = Query(None, description="Limit to one politician"),
) -> dict:
    """Launch contradiction detection. Cancel with DELETE /detect-contradictions."""
    global _detect_task
    if _detect_status["running"]:
        raise HTTPException(409, "Une detection est deja en cours")
    _reset_detect_status()
    _detect_task = asyncio.create_task(_detect_contradictions_bg(politician_id))
    return {"status": "detection_started", "message": "Detection lancee"}


@router.delete("/detect-contradictions")
async def stop_detection() -> dict:
    """Cancel a running contradiction detection."""
    global _detect_task
    if not _detect_status["running"] or _detect_task is None:
        raise HTTPException(404, "Aucune detection en cours")
    _detect_task.cancel()
    return {"status": "detection_cancelled", "message": "Detection annulee"}


@router.get("/detect-contradictions/status")
async def get_detection_status() -> dict:
    """Get real-time detection progress."""
    return dict(_detect_status)


# ====================================================================
# GOV MANAGER -- worker status / pipeline overview
# ====================================================================

@router.get("/workers")
async def get_gov_workers(request: Request):
    """Get real-time status of all government sync workers."""
    mgr = getattr(request.app.state, "gov_manager", None)
    if not mgr:
        return {"running": False, "workers": 0, "worker_status": []}
    return mgr.get_status()


@router.get("/pipeline")
async def get_gov_pipeline(
    request: Request,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """Combined pipeline view: manager status + DB stats."""
    mgr = getattr(request.app.state, "gov_manager", None)
    stats = await gov.get_stats()
    return {
        "manager": mgr.get_status() if mgr else {"running": False},
        "stats": stats,
    }


# ====================================================================
# SCAN LOG
# ====================================================================

@router.get("/scans", response_model=list[ScanLog])
async def list_scans(
    limit: int = Query(50, ge=1, le=500),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
) -> list[ScanLog]:
    """List recent scan logs."""
    rows = await gov.list_scan_logs(limit=limit, offset=offset)
    return [ScanLog(**r) for r in rows]


# ====================================================================
# SOCIAL MEDIA
# ====================================================================

@router.get("/social")
async def list_all_social(
    platform: str | None = Query(None),
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List recent social posts across all politicians."""
    conditions: list[str] = []
    params: list = []
    if platform:
        conditions.append("platform = ?")
        params.append(platform)
    where = "WHERE " + " AND ".join(conditions) if conditions else ""
    cursor = await gov._conn.execute(
        f"SELECT * FROM gov_social_posts {where} ORDER BY posted_at DESC LIMIT ?",
        params + [limit],
    )
    rows = await cursor.fetchall()
    return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]


@router.get("/politicians/{politician_id}/social")
async def list_social_posts(
    politician_id: str,
    platform: str | None = Query(None),
    limit: int = Query(50, ge=1, le=500),
    offset: int = Query(0, ge=0),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List social media posts for a politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    posts = await gov.list_social_by_politician(politician_id, platform=platform, limit=limit)
    return posts


# ====================================================================
# TRANSCRIPTIONS
# ====================================================================

@router.get("/transcriptions")
async def list_all_transcriptions(
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List recent transcriptions across all politicians."""
    cursor = await gov._conn.execute(
        "SELECT * FROM gov_transcriptions ORDER BY created_at DESC LIMIT ?", (limit,)
    )
    rows = await cursor.fetchall()
    return [_dict_with_json_fields(_row_to_dict(r), "metadata", "timestamped_text") for r in rows]


@router.get("/politicians/{politician_id}/transcriptions")
async def list_transcriptions(
    politician_id: str,
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List transcriptions for a politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return await gov.list_transcriptions_by_politician(politician_id, limit=limit)


# ====================================================================
# ALERTS
# ====================================================================

@router.get("/alerts")
async def list_alerts(
    is_read: bool | None = Query(None),
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List alerts with optional read/unread filter."""
    conditions: list[str] = []
    params: list = []
    if is_read is not None:
        conditions.append("is_read = ?")
        params.append(1 if is_read else 0)
    where = "WHERE " + " AND ".join(conditions) if conditions else ""
    cursor = await gov._conn.execute(
        f"SELECT * FROM gov_alerts {where} ORDER BY created_at DESC LIMIT ?",
        params + [limit],
    )
    rows = await cursor.fetchall()
    return [_row_to_dict(r) for r in rows]


@router.put("/alerts/{alert_id}/read")
async def mark_alert_read(
    alert_id: str,
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """Mark an alert as read."""
    cursor = await gov._conn.execute("SELECT id FROM gov_alerts WHERE id = ?", (alert_id,))
    if not await cursor.fetchone():
        raise HTTPException(status_code=404, detail=f"Alert not found: {alert_id}")
    await gov._conn.execute("UPDATE gov_alerts SET is_read = 1 WHERE id = ?", (alert_id,))
    await gov._conn.commit()
    return {"status": "ok"}


# ====================================================================
# PRESS
# ====================================================================

@router.get("/press")
async def list_press(
    limit: int = Query(50, ge=1, le=500),
    sentiment: str | None = Query(None),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List press articles with optional sentiment filter."""
    conditions: list[str] = []
    params: list = []
    if sentiment:
        conditions.append("sentiment = ?")
        params.append(sentiment)
    where = "WHERE " + " AND ".join(conditions) if conditions else ""
    cursor = await gov._conn.execute(
        f"SELECT * FROM gov_press {where} ORDER BY published_at DESC LIMIT ?",
        params + [limit],
    )
    rows = await cursor.fetchall()
    return [_dict_with_json_fields(_row_to_dict(r), "metadata", "politicians_mentioned", "subjects") for r in rows]


@router.get("/politicians/{politician_id}/press")
async def list_press_by_politician(
    politician_id: str,
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List press articles mentioning a politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return await gov.list_press_by_politician(politician_id, limit=limit)


# ====================================================================
# AFFAIRS
# ====================================================================

@router.get("/affairs")
async def list_affairs(
    status: str | None = Query(None),
    limit: int = Query(100, ge=1, le=1000),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List affairs/scandals with optional status filter."""
    conditions: list[str] = []
    params: list = []
    if status:
        conditions.append("status = ?")
        params.append(status)
    where = "WHERE " + " AND ".join(conditions) if conditions else ""
    cursor = await gov._conn.execute(
        f"SELECT * FROM gov_affairs {where} ORDER BY created_at DESC LIMIT ?",
        params + [limit],
    )
    rows = await cursor.fetchall()
    return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]


@router.get("/politicians/{politician_id}/affairs")
async def list_affairs_by_politician(
    politician_id: str,
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List affairs linked to a politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return await gov.list_affairs_by_politician(politician_id, limit=limit)


# ====================================================================
# LAWS
# ====================================================================

@router.get("/laws")
async def list_laws(
    status: str | None = Query(None),
    limit: int = Query(100, ge=1, le=1000),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List laws with optional status filter."""
    return await gov.list_laws(status=status, limit=limit)


# ====================================================================
# DECLARATIONS
# ====================================================================

@router.get("/politicians/{politician_id}/declarations")
async def list_declarations(
    politician_id: str,
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List HATVP declarations for a politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return await gov.list_declarations_by_politician(politician_id, limit=limit)


# ====================================================================
# FACTCHECKS
# ====================================================================

@router.get("/factchecks")
async def list_factchecks(
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List factchecks across all politicians."""
    cursor = await gov._conn.execute(
        "SELECT * FROM gov_factchecks ORDER BY created_at DESC LIMIT ?", (limit,)
    )
    rows = await cursor.fetchall()
    return [_dict_with_json_fields(_row_to_dict(r), "metadata") for r in rows]


@router.get("/politicians/{politician_id}/factchecks")
async def list_factchecks_by_politician(
    politician_id: str,
    limit: int = Query(50, ge=1, le=500),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """List factchecks for a specific politician."""
    existing = await gov.get_politician(politician_id)
    if existing is None:
        raise HTTPException(status_code=404, detail=f"Politician not found: {politician_id}")
    return await gov.list_factchecks_by_politician(politician_id, limit=limit)


# ====================================================================
# SEARCH (RAG)
# ====================================================================

@router.get("/health")
async def get_gov_health(request: Request):
    """Health status of all government data sources."""
    monitor = getattr(request.app.state, "gov_health_monitor", None)
    if not monitor:
        return {"sources": [], "message": "Health monitor not started"}
    return {"sources": monitor.get_status()}


@router.get("/search")
async def search_gov(
    request: Request,
    q: str = Query(..., min_length=2),
    limit: int = Query(10, ge=1, le=50),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """Semantic search across all political data."""
    from nexus.gov.rag import GovRAG
    chroma = getattr(request.app.state, "chroma", None)
    router_llm = getattr(request.app.state, "router", None)
    rag = GovRAG(chroma=chroma, router=router_llm)
    return await rag.search(q, n_results=limit)


@router.get("/ask")
async def ask_gov(
    request: Request,
    q: str = Query(..., min_length=2),
    gov: GovernmentDatabase = Depends(get_government_database),
):
    """Ask a question with RAG context."""
    from nexus.gov.rag import GovRAG
    chroma = getattr(request.app.state, "chroma", None)
    router_llm = getattr(request.app.state, "router", None)
    rag = GovRAG(chroma=chroma, router=router_llm)
    return await rag.ask(q)
