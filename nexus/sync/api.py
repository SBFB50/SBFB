"""
NEXUS Sync -- API endpoints for database synchronization.

Endpoints:
  GET  /api/sync/version    — Current database version
  GET  /api/sync/tables     — List of synced tables
  GET  /api/sync/status     — Sync system status
  GET  /api/sync/snapshot   — Download full database file (initial sync)
  WS   /ws/sync             — WebSocket for real-time changesets
"""

from __future__ import annotations

from pathlib import Path

from fastapi import APIRouter, Request, WebSocket
from fastapi.responses import FileResponse, StreamingResponse
from loguru import logger

from nexus.config import settings


router = APIRouter(tags=["sync"])


# ============================================================================
# REST endpoints
# ============================================================================

@router.get("/api/sync/version")
async def get_sync_version(request: Request) -> dict:
    """Current database version for sync."""
    broadcaster = getattr(request.app.state, "sync_broadcaster", None)
    if not broadcaster:
        return {"version": 0, "sync_enabled": False}
    return {
        "version": broadcaster.db_version,
        "sync_enabled": True,
        "crsqlite": broadcaster.crsqlite_available,
    }


@router.get("/api/sync/tables")
async def get_sync_tables(request: Request) -> dict:
    """List of tables available for sync."""
    broadcaster = getattr(request.app.state, "sync_broadcaster", None)
    if not broadcaster:
        return {"tables": [], "sync_enabled": False}
    from nexus.sync.broadcaster import SYNC_TABLES
    return {"tables": SYNC_TABLES, "sync_enabled": True}


@router.get("/api/sync/status")
async def get_sync_status(request: Request) -> dict:
    """Sync system status (broadcaster state, clients, version)."""
    broadcaster = getattr(request.app.state, "sync_broadcaster", None)
    if not broadcaster:
        return {"running": False, "sync_enabled": False}
    return broadcaster.get_status()


@router.get("/api/sync/snapshot")
async def get_sync_snapshot(request: Request):
    """Download the full database file for initial sync.

    Supports Range header for resume on interrupted downloads.
    The database is served as a binary file (~500MB for production).
    """
    db_path = Path(settings.sqlite_path)
    if not db_path.exists():
        from fastapi import HTTPException
        raise HTTPException(status_code=404, detail="Database file not found")

    return FileResponse(
        path=str(db_path),
        media_type="application/x-sqlite3",
        filename="nexus.db",
    )


# ============================================================================
# WebSocket endpoint
# ============================================================================

@router.websocket("/ws/sync")
async def websocket_sync(websocket: WebSocket):
    """WebSocket endpoint for real-time changeset sync.

    Protocol:
    1. Server sends: {"type": "version", "version": N, "tables": [...]}
    2. Server sends: {"type": "changes", "version": N, "changes": [[...]]}
    3. Server sends: {"type": "ping", "version": N} (every 30s keepalive)
    4. Client: read-only, no messages expected
    """
    broadcaster = getattr(websocket.app.state, "sync_broadcaster", None)
    if not broadcaster or not broadcaster._running:
        await websocket.close(code=1013, reason="Sync not available")
        return

    await broadcaster.handle_client(websocket)
