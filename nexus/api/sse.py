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
from nexus.gov.events import GovEventType

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


@router.get("/gov/events")
async def gov_event_stream(request: Request):
    """SSE stream for government module events.

    Streams real-time updates from the government monitoring pipeline:
    positions, contradictions, press articles, social posts, affairs,
    patterns, and alerts.
    """
    gov_manager = getattr(request.app.state, "gov_manager", None)
    if gov_manager is None or not gov_manager.running:
        raise HTTPException(503, "Government module unavailable")

    bus = gov_manager.bus
    if bus is None:
        raise HTTPException(503, "Government EventBus not running")

    bridge = SSEBridge(bus)

    # Subscribe to the event types the frontend cares about
    gov_types = [
        GovEventType.GOV_POSITION_ADDED,
        GovEventType.GOV_CONTRADICTION_FOUND,
        GovEventType.GOV_PRESS_ADDED,
        GovEventType.GOV_SOCIAL_POST_ADDED,
        GovEventType.GOV_AFFAIR_ADDED,
        GovEventType.GOV_PATTERN_DETECTED,
        GovEventType.GOV_ALERT_CREATED,
        GovEventType.GOV_POLITICIAN_ADDED,
        GovEventType.GOV_DECLARATION_ADDED,
        GovEventType.GOV_FACTCHECK_ADDED,
        GovEventType.GOV_TRANSCRIPTION_READY,
    ]

    return EventSourceResponse(
        bridge.stream(gov_types),  # type: ignore[arg-type]  # GovEventType compatible
        ping=15,
    )
