# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/tasks/submit`` and ``/tasks`` endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel, Field

from nexus_coordinator.dispatcher import SubmitRequest
from nexus_coordinator.upload_queue import QueueFullError

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter(prefix="/tasks", tags=["tasks"])


class TaskCreateBody(BaseModel):
    """POST /tasks/submit body.

    Sprint 18 Phase D: ``is_open_source`` and ``estimated_*`` are
    deliberately absent from this schema. They are *derived
    server-side* by the handler from project config
    (``identity.repo_url`` → ``is_open_source``) and from the
    submitting app's :meth:`NexusApp.cost_estimate`. A client
    that tacks those fields onto the JSON body sees them dropped
    by ``extra="ignore"``-style Pydantic forbid behavior (the
    default on BaseModel is to silently ignore unknown fields);
    the invariant from Sprint 16 D-1 (*no user override on
    ``is_open_source``*) holds by construction. ``app_name`` is a
    hint to the handler so it can pick the right app's cost
    estimate; if omitted or unknown the handler falls back to
    the conservative SDK default.
    """

    task_type: str = Field(..., min_length=1, max_length=64)
    prompt: str = Field(..., min_length=1)
    model: str = Field(..., min_length=1, max_length=128)
    system_prompt: str = ""
    priority: int = Field(5, ge=1, le=10)
    parent_task_id: str = ""
    metadata: dict[str, str] | None = None
    task_id: str | None = None
    app_name: str | None = Field(
        default=None,
        description=(
            "Optional name of the NexusApp submitting this task. "
            "Used by the handler to pick the app's "
            "cost_estimate() override when crafting the TaskEntry."
        ),
    )


def _coord(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


def _derive_cost_estimate(coord: "Coordinator", app_name: str | None) -> tuple[int, int, float]:
    """Resolve the (watts, vram_mb, hours) estimate for the task.

    Sprint 18 Phase D helper — looks up the app instance by name
    on the coordinator and returns its
    :meth:`NexusApp.cost_estimate` tuple. Falls back to the SDK
    conservative default ``(100, 2000, 0.1)`` when ``app_name``
    is ``None`` or the app is not registered on this coordinator
    (e.g. direct ``/tasks/submit`` from an external CLI).
    """
    fallback: tuple[int, int, float] = (100, 2000, 0.1)
    if not app_name:
        return fallback
    app = coord.apps.get(app_name)
    if app is None:
        return fallback
    try:
        watts, vram_mb, hours = app.cost_estimate()
    except Exception:  # pragma: no cover - defensive against buggy overrides
        return fallback
    return (int(watts), int(vram_mb), float(hours))


@router.post("/submit")
async def submit_task(request: Request, body: TaskCreateBody) -> dict[str, Any]:
    coord = _coord(request)
    dispatcher = coord.dispatcher
    if dispatcher is None:
        raise HTTPException(status_code=503, detail="dispatcher not yet initialised")
    is_open_source = coord.config.identity.repo_url is not None
    watts, vram_mb, hours = _derive_cost_estimate(coord, body.app_name)
    # Sprint 19 Phase D: route through the delayed upload queue.
    # The queue persists the payload, draws a random delay, and
    # returns the resolved ``task_id`` immediately. When the
    # operator has disabled the queue (``upload_queue.enabled =
    # false`` in coordinator.toml, e.g. for dev), the queue falls
    # back to a passthrough mode that calls ``dispatcher.submit``
    # synchronously — the client sees identical latency as pre-S19.
    upload_queue = coord.upload_queue
    if upload_queue is None:
        raise HTTPException(status_code=503, detail="upload queue not yet initialised")
    submit_req = SubmitRequest(
        task_type=body.task_type,
        prompt=body.prompt,
        model=body.model,
        system_prompt=body.system_prompt,
        priority=body.priority,
        parent_task_id=body.parent_task_id,
        metadata=body.metadata,
        task_id=body.task_id,
        is_open_source=is_open_source,
        estimated_watts=watts,
        estimated_vram_mb=vram_mb,
        estimated_hours=hours,
    )
    try:
        task_id = await upload_queue.schedule(submit_req)
    except QueueFullError as exc:
        raise HTTPException(
            status_code=429,
            detail=str(exc),
            headers={"Retry-After": "30"},
        ) from exc
    return {"task_id": task_id}


@router.get("")
async def list_tasks(request: Request, state: str | None = None, limit: int = 100) -> dict[str, Any]:
    coord = _coord(request)
    dispatcher = coord.dispatcher
    if dispatcher is None:
        raise HTTPException(status_code=503, detail="dispatcher not yet initialised")
    rows = await dispatcher.list_tasks(state=state, limit=limit)
    return {"tasks": rows, "count": len(rows)}
