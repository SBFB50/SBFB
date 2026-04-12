# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/health`` and ``/project`` endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

from fastapi import APIRouter, Request

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter()


def _coordinator(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.get("/health")
async def health(request: Request) -> dict[str, object]:
    """Liveness probe: confirms the coordinator has a booted Node.

    Returns 200 with the coordinator's current state summary. Used
    by the CLI ``start`` subprocess smoke test and by the e2e
    Sprint 4 acceptance test to wait for the coordinator to be
    ready before submitting tasks.
    """
    return _coordinator(request).health_payload()


@router.get("/project")
async def project(request: Request) -> dict[str, object]:
    """Project metadata: name, visibility, doc_id, author_id.

    Does NOT expose the full tasks_doc_ticket (the prefix only) so
    a read-only curl to ``/project`` cannot hand out write access.
    Full tickets are only emitted through the ``/invite/create``
    endpoint landing in Phase C.
    """
    return _coordinator(request).project_payload()
