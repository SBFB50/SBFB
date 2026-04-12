# SPDX-License-Identifier: AGPL-3.0-or-later
"""Thin HTTP client for the coordinator's compute API.

Apps reach the local coordinator through an instance of
:class:`ComputeClient` passed into their ``on_start`` via
:class:`nexus_sdk.AppContext`. The client wraps the coordinator's
``POST /tasks/submit`` endpoint and surfaces a future-style
``submit_task`` coroutine.

Phase D v1: submit + fire-and-forget task id return. Phase E+
will add polling / websocket streaming for results.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import httpx


@dataclass
class SubmittedTask:
    """Returned by :meth:`ComputeClient.submit_task`.

    Holds the coordinator-assigned ``task_id`` and the submission
    timestamp so the caller can poll for the result later.
    """

    task_id: str
    submitted_at: int = 0


class ComputeClient:
    """HTTP client for the local coordinator's ``/tasks`` routes."""

    def __init__(self, coordinator_url: str, *, timeout: float = 10.0) -> None:
        self._base_url = coordinator_url.rstrip("/")
        self._timeout = timeout

    async def submit_task(
        self,
        *,
        task_type: str,
        prompt: str,
        model: str,
        system_prompt: str = "",
        priority: int = 5,
        metadata: dict[str, str] | None = None,
        task_id: str | None = None,
    ) -> SubmittedTask:
        """Submit a new task via ``POST /tasks/submit``.

        Returns immediately once the coordinator has accepted the
        task (not when a worker has finished it).
        """
        body: dict[str, Any] = {
            "task_type": task_type,
            "prompt": prompt,
            "model": model,
            "system_prompt": system_prompt,
            "priority": priority,
        }
        if metadata is not None:
            body["metadata"] = metadata
        if task_id is not None:
            body["task_id"] = task_id

        async with httpx.AsyncClient(base_url=self._base_url, timeout=self._timeout) as client:
            response = await client.post("/tasks/submit", json=body)
            response.raise_for_status()
            return SubmittedTask(task_id=response.json()["task_id"])

    async def list_tasks(self, state: str | None = None, limit: int = 100) -> list[dict[str, Any]]:
        """Return the coordinator's recent tasks, optionally filtered by state."""
        params: dict[str, Any] = {"limit": limit}
        if state:
            params["state"] = state
        async with httpx.AsyncClient(base_url=self._base_url, timeout=self._timeout) as client:
            response = await client.get("/tasks", params=params)
            response.raise_for_status()
            return list(response.json().get("tasks", []))
