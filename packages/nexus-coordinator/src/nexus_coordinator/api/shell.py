# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 5 Phase A — ``/shell/*`` endpoints.

The shell calls ``GET /shell/discover`` on any coordinator it
already knows about to learn about every *other* coordinator
the user has running on the same machine. See
``.planning/sprint5_plan.md`` §2.1 and §4.2.

The endpoint has no auth because the whole shell layer runs on
``127.0.0.1`` — a process on another machine cannot reach it.
Binding to a public interface is a user decision that Sprint 5
does not encourage (the coordinator defaults to loopback).
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from fastapi import APIRouter, Request

from nexus_coordinator.registry import SCHEMA_VERSION, discover_running

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

router = APIRouter(prefix="/shell", tags=["shell"])


def _coord(request: Request) -> "Coordinator":
    return request.app.state.coordinator  # type: ignore[no-any-return]


@router.get("/discover")
async def discover(request: Request) -> dict[str, Any]:
    """Return every coordinator with a live ``running.json`` entry.

    The ``self`` field identifies which coordinator served the
    request so the shell can deduplicate the entry it already
    knows about. Stale files (a coordinator that crashed without
    removing its ``running.json``) are still returned here;
    marking them offline is the shell's job via a ``/health``
    roundtrip on each entry.
    """
    coord = _coord(request)
    entries = discover_running()
    return {
        "schema_version": SCHEMA_VERSION,
        "coordinators": [e.model_dump() for e in entries],
        "count": len(entries),
        "self": {
            "project_name": coord.project_name,
            "node_id": coord.state.node_id,
            "api_host": coord.config.network.api_host,
            "api_port": coord.config.network.api_port,
        },
    }
