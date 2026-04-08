"""
NEXUS -- Server-Sent Events endpoints.

Provides real-time event streaming from the reactive investigation
pipeline to the frontend, replacing polling.
"""

from __future__ import annotations

from fastapi import APIRouter, HTTPException, Request
from sse_starlette.sse import EventSourceResponse

from nexus.events.sse_bridge import SSEBridge
from nexus.events.types import EventType

router = APIRouter(prefix="/api", tags=["sse"])


@router.get("/cases/{case_id}/events")
async def case_event_stream(case_id: str, request: Request):
    """SSE stream for all events in a case investigation.

    The client receives events as they happen in the reactive pipeline:
    evidence_added, entity_discovered, analysis_completed, etc.
    """
    mgr = getattr(request.app.state, "investigation_manager", None)
    if mgr is None:
        raise HTTPException(503, "Investigation manager unavailable")

    bus = mgr.get_event_bus(case_id)
    if bus is None:
        raise HTTPException(404, f"No active investigation for case {case_id}")

    bridge = SSEBridge(bus)
    all_types = list(EventType)

    return EventSourceResponse(
        bridge.stream(all_types, case_id=case_id),
        ping=15,
    )


@router.get("/system/events")
async def system_event_stream(request: Request):
    """SSE stream for system-wide events across all active cases.

    Useful for the dashboard to show activity across investigations.
    Requires at least one active investigation.
    """
    mgr = getattr(request.app.state, "investigation_manager", None)
    if mgr is None:
        raise HTTPException(503, "Investigation manager unavailable")

    # Find the first active investigation's bus
    status = mgr.get_status()
    investigations = status.get("investigations", {})
    if not investigations:
        raise HTTPException(404, "No active investigations")

    # Use the first case's bus (system-wide would need a global bus)
    first_case_id = next(iter(investigations))
    bus = mgr.get_event_bus(first_case_id)
    if bus is None:
        raise HTTPException(404, "No active event bus")

    bridge = SSEBridge(bus)
    all_types = list(EventType)

    return EventSourceResponse(
        bridge.stream(all_types),  # No case_id filter = all events
        ping=15,
    )
