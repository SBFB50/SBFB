# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 21 Phase D — quarantine queue REST endpoints.

Exposes three endpoints under the loopback bearer auth already
enforced app-wide by :class:`LoopbackAuthMiddleware` (Sprint 16):

- ``GET  /quarantine/list?status=<pending|flushed|dropped|all>``
  — paginated-light listing of quarantine entries (no cursor for
  Phase D ; cardinality stays bounded by 15-min TTL).
- ``POST /quarantine/flush/{row_id}`` — operator-accept marker
  (re-injection into gossip is hors-scope Phase D, design §5.1).
- ``POST /quarantine/drop/{row_id}`` — operator-reject marker.

The router itself contains no auth checks because every request
already passes through the middleware (pattern miroir
``api/canary.py`` Sprint 20 Phase E). Tests that talk to the app
factory must inject ``SBFB_AUTH_TOKEN`` and a loopback ``Host:``
header to satisfy the middleware (cf. Sprint 16 test harness).

Hex-encoded ``sender_pubkey`` and ``payload_bytes`` are returned
in the JSON body — raw bytes do not survive the JSON round-trip
and the operator typically inspects the pubkey lookup-style.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import structlog
from fastapi import APIRouter, HTTPException, Query, Request

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

router = APIRouter()


def _coordinator(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


def _serialize(row: dict[str, Any]) -> dict[str, Any]:
    """Convert raw aiosqlite row dict into a JSON-safe shape.

    The two ``BLOB`` columns (``sender_pubkey`` 32-byte Ed25519
    and ``payload_bytes`` opaque) are hex-encoded so the response
    survives JSON serialisation cleanly.
    """
    return {
        "id": row["id"],
        "topic": row["topic"],
        "sender_pubkey_hex": bytes(row["sender_pubkey"]).hex(),
        "payload_bytes_hex": bytes(row["payload_bytes"]).hex(),
        "received_at_epoch_s": row["received_at_epoch_s"],
        "rate_strikes": row["rate_strikes"],
        "pow_status": row["pow_status"],
        "flush_status": row["flush_status"],
    }


@router.get("/quarantine/list")
async def list_entries(
    request: Request,
    status: str = Query("pending", description="Filter by flush_status; 'all' returns every row."),
) -> dict[str, Any]:
    """Return quarantine entries filtered by ``status``."""
    coord = _coordinator(request)
    if coord.quarantine_queue is None:
        raise HTTPException(status_code=503, detail="quarantine queue not initialised")
    try:
        rows = await coord.quarantine_queue.list(status=status)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    return {"entries": [_serialize(r) for r in rows], "count": len(rows)}


@router.post("/quarantine/flush/{row_id}")
async def flush_entry(request: Request, row_id: int) -> dict[str, Any]:
    """Mark a pending entry as ``flushed`` (operator accept).

    Returns ``{"updated": true}`` on success or HTTP 404 if the row
    id is missing or already non-pending.
    """
    coord = _coordinator(request)
    if coord.quarantine_queue is None:
        raise HTTPException(status_code=503, detail="quarantine queue not initialised")
    updated = await coord.quarantine_queue.flush(row_id)
    if not updated:
        raise HTTPException(status_code=404, detail=f"row {row_id} not found or already non-pending")
    return {"updated": True, "row_id": row_id, "new_status": "flushed"}


@router.post("/quarantine/drop/{row_id}")
async def drop_entry(request: Request, row_id: int) -> dict[str, Any]:
    """Mark a pending entry as ``dropped`` (operator reject)."""
    coord = _coordinator(request)
    if coord.quarantine_queue is None:
        raise HTTPException(status_code=503, detail="quarantine queue not initialised")
    updated = await coord.quarantine_queue.drop(row_id)
    if not updated:
        raise HTTPException(status_code=404, detail=f"row {row_id} not found or already non-pending")
    return {"updated": True, "row_id": row_id, "new_status": "dropped"}
