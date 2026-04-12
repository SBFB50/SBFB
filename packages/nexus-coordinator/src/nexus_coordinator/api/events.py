# SPDX-License-Identifier: AGPL-3.0-or-later
"""``GET /app/{name}/events`` — Server-Sent Events bridge over :class:`AppEvents`.

Sprint 9 Phase C (D2 SSE bridge). The route subscribes to the
per-app :class:`nexus_sdk.AppEvents` bus, streams every matching
:class:`nexus_sdk.EventEnvelope` as a ``text/event-stream``
``data: <json>`` line, and emits a comment heartbeat every 30 s
so dead client connections surface as a write error and the
``finally:`` cleanup runs.

Wire format
-----------

Each envelope lands as one SSE message::

    data: {"topic":"party.refreshed","payload":{...},"timestamp":"2026-04-12T...","trace_id":"abc..."}\n\n

Heartbeat (every 30 s) lands as an SSE comment line that the
browser's ``EventSource`` ignores::

    : ping\n\n

Cleanup contract (R7 mitigation)
--------------------------------

The subscription registration lives inside the streaming
generator's ``async with bus.subscribe(pattern):`` context
manager. The generator's ``finally:`` block aclose's the
subscriber unconditionally — even when Starlette tears the
connection down on a brutal client disconnect via
:class:`anyio.get_cancelled_exc_class`. The dedicated
``test_events_sse_disconnect_unregisters_subscriber`` pins this
contract.

The route is intentionally NOT under ``/app/{name}`` via the
``apps`` router — it lives in its own router so the bus
plumbing is grep-able and the ``include_router`` call site is
unambiguous about the SSE surface.
"""

from __future__ import annotations

import asyncio
import logging
from typing import TYPE_CHECKING, Any, AsyncIterator

import anyio
from fastapi import APIRouter, HTTPException, Query, Request
from fastapi.responses import StreamingResponse
from nexus_sdk import AppEvents
from pydantic import BaseModel, Field

if TYPE_CHECKING:
    from nexus_sdk import AppContext

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/app", tags=["events"])


_HEARTBEAT_INTERVAL_SECONDS = 30.0


async def render_sse_stream(
    bus: AppEvents,
    pattern: str,
    *,
    heartbeat_interval_seconds: float = _HEARTBEAT_INTERVAL_SECONDS,
) -> AsyncIterator[bytes]:
    """Yield SSE-framed envelopes from ``bus`` until cancellation.

    Extracted from the route handler so the streaming contract
    can be exercised end-to-end without an ASGI transport (which
    has a long-standing issue with :class:`StreamingResponse`
    cancellation under :class:`httpx.ASGITransport` in tests).

    The ``async with bus.subscribe(pattern):`` block lives in the
    body so its ``finally:`` aclose lands on every cancellation
    path — that is the R7 mitigation. Heartbeats are emitted as
    ``: ping`` SSE comment lines after every
    ``heartbeat_interval_seconds`` of receive timeout, so a dead
    client surfaces on the next yield.
    """
    async with bus.subscribe(pattern) as stream:
        while True:
            try:
                envelope = await asyncio.wait_for(
                    stream.receive(),
                    timeout=heartbeat_interval_seconds,
                )
            except asyncio.TimeoutError:
                yield b": ping\n\n"
                continue
            except (anyio.EndOfStream, anyio.ClosedResourceError):
                return
            payload = envelope.model_dump_json()
            yield f"data: {payload}\n\n".encode("utf-8")


def _app_contexts(request: Request) -> dict[str, "AppContext"]:
    """Return the coordinator's per-app context registry.

    Same shape as :func:`nexus_coordinator.api.apps._app_contexts`
    so future refactors can converge on a single helper without a
    public API change.
    """
    coord = request.app.state.coordinator
    return getattr(coord, "app_contexts", {})


@router.get("/{name}/events")
async def stream_app_events(
    request: Request,
    name: str,
    pattern: str = Query("*", min_length=1, max_length=256),
) -> StreamingResponse:
    """Stream every event matching ``pattern`` from app ``name``'s bus.

    The default pattern ``*`` subscribes to every topic the app
    publishes — useful for an integration test or a debug tab.
    Production consumers should pass a precise glob like
    ``party.refreshed`` so they only get the topics they care
    about and avoid the per-event JSON serialisation cost.

    Failure modes:

    - 404 — unknown app or app context not yet wired by the
      coordinator loader.
    - 503 — the app context exists but ``ctx.events`` is
      ``None`` (a coordinator bug, because the loader always
      wires the bus alongside the storage).
    """
    contexts = _app_contexts(request)
    ctx = contexts.get(name)
    if ctx is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    if ctx.events is None:
        raise HTTPException(
            status_code=503,
            detail=f"app {name!r} has no AppContext.events wired",
        )

    return StreamingResponse(
        render_sse_stream(ctx.events, pattern),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


class _PublishRequest(BaseModel):
    """Body for the admin ``POST /app/{name}/events/_publish`` route.

    The route is intentionally non-discoverable from the React
    shell — it exists so the Sprint 9 Phase C Playwright e2e can
    drive a publish onto the in-process bus without spinning up a
    live worker daemon. Production consumers always go through a
    ``@nexus_worker`` handler that owns the publish call site.
    """

    model_config = {"extra": "forbid"}

    topic: str = Field(..., min_length=1, max_length=256)
    payload: dict[str, Any] = Field(default_factory=dict)


@router.post("/{name}/events/_publish")
async def admin_publish_event(
    request: Request,
    name: str,
    body: _PublishRequest,
) -> dict[str, str]:
    """Publish an envelope onto the per-app bus directly.

    Sprint 9 Phase C admin endpoint reserved for the e2e suite.
    The handler validates the body, looks up the app context,
    awaits :meth:`AppEvents.publish` and returns the published
    topic + a synthetic ``status`` field. Failure modes mirror
    :func:`stream_app_events`: 404 for an unknown app, 503 when
    the bus is not wired (a coordinator bug).
    """
    contexts = _app_contexts(request)
    ctx = contexts.get(name)
    if ctx is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")
    if ctx.events is None:
        raise HTTPException(
            status_code=503,
            detail=f"app {name!r} has no AppContext.events wired",
        )
    await ctx.events.publish(body.topic, body.payload)
    return {"status": "published", "topic": body.topic}
