# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/health``, ``/project``, and ``/project/publish`` endpoints."""

from __future__ import annotations

from typing import TYPE_CHECKING

import structlog
from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

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


@router.post("/project/publish")
async def publish_project(request: Request) -> JSONResponse:
    """Publish this project to the P2P network via the daemon.

    Sprint 11 Phase A. Reads the coordinator's own config to build
    the publish payload, then forwards it to the daemon's
    ``POST /publish`` via the ``/daemon/publish`` proxy. Returns
    the daemon's response as-is.
    """
    from nexus_coordinator.api import daemon as _daemon_mod

    coord = _coordinator(request)
    payload = {
        "project_name": coord.project_name,
        "category": coord.config.identity.description or "general",
        "description": coord.config.identity.description or coord.project_name,
        "apps": list(coord.apps.keys()) if coord.apps else [],
    }
    return await _daemon_mod._forward(request, "POST", "/publish", json_body=payload)
