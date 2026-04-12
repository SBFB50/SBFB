# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/tasks/submit`` and ``/tasks`` endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, HTTPException, Request
from pydantic import BaseModel, Field

from nexus_coordinator.dispatcher import SubmitRequest

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter(prefix="/tasks", tags=["tasks"])


class TaskCreateBody(BaseModel):
    """POST /tasks/submit body."""

    task_type: str = Field(..., min_length=1, max_length=64)
    prompt: str = Field(..., min_length=1)
    model: str = Field(..., min_length=1, max_length=128)
    system_prompt: str = ""
    priority: int = Field(5, ge=1, le=10)
    parent_task_id: str = ""
    metadata: dict[str, str] | None = None
    task_id: str | None = None


def _coord(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.post("/submit")
async def submit_task(request: Request, body: TaskCreateBody) -> dict[str, Any]:
    coord = _coord(request)
    dispatcher = coord.dispatcher
    if dispatcher is None:
        raise HTTPException(status_code=503, detail="dispatcher not yet initialised")
    task_id = await dispatcher.submit(
        SubmitRequest(
            task_type=body.task_type,
            prompt=body.prompt,
            model=body.model,
            system_prompt=body.system_prompt,
            priority=body.priority,
            parent_task_id=body.parent_task_id,
            metadata=body.metadata,
            task_id=body.task_id,
        )
    )
    return {"task_id": task_id}


@router.get("")
async def list_tasks(request: Request, state: str | None = None, limit: int = 100) -> dict[str, Any]:
    coord = _coord(request)
    dispatcher = coord.dispatcher
    if dispatcher is None:
        raise HTTPException(status_code=503, detail="dispatcher not yet initialised")
    rows = await dispatcher.list_tasks(state=state, limit=limit)
    return {"tasks": rows, "count": len(rows)}
