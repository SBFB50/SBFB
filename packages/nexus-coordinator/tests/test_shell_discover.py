"""Sprint 5 Phase A — ``GET /shell/discover`` endpoint tests."""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.registry import SCHEMA_VERSION, write_running_state


@pytest.mark.asyncio
async def test_shell_discover_returns_own_entry(nexus_grid_tmp: Path) -> None:
    """When the serving coordinator has its own running.json
    written, ``/shell/discover`` returns it in the coordinators
    list and echoes the same values via the ``self`` field."""
    coord = Coordinator(project_name="shell-self")
    await coord.start()
    write_running_state(coord)
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/shell/discover")
            assert r.status_code == 200
            body = r.json()
            assert body["schema_version"] == SCHEMA_VERSION
            assert body["count"] == 1
            assert len(body["coordinators"]) == 1
            entry = body["coordinators"][0]
            assert entry["project_name"] == "shell-self"
            assert entry["node_id"] == coord.state.node_id
            assert entry["doc_id"] == coord.state.doc_id

            self_info = body["self"]
            assert self_info["project_name"] == "shell-self"
            assert self_info["node_id"] == coord.state.node_id
            assert self_info["api_host"] == coord.config.network.api_host
            assert self_info["api_port"] == coord.config.network.api_port
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_shell_discover_returns_multiple_coordinators(nexus_grid_tmp: Path) -> None:
    """If two projects have running.json files side by side,
    both are listed. Each ``Coordinator`` here is booted in-process
    so we can serve the endpoint from either and see both entries."""
    coord_a = Coordinator(project_name="alpha")
    coord_b = Coordinator(project_name="beta")
    # Avoid the default ports colliding on the CI runner; we are
    # not actually binding these coordinators to uvicorn here, but
    # the port value is what the discover endpoint emits.
    coord_a.config.network.api_port = 18765
    coord_b.config.network.api_port = 18766
    await coord_a.start()
    await coord_b.start()
    write_running_state(coord_a)
    write_running_state(coord_b)

    try:
        with TestClient(create_app(coord_a)) as client:
            r = client.get("/shell/discover")
            assert r.status_code == 200
            body = r.json()
            assert body["count"] == 2
            names = {c["project_name"] for c in body["coordinators"]}
            assert names == {"alpha", "beta"}
            # Serving coordinator identifies itself.
            assert body["self"]["project_name"] == "alpha"
    finally:
        await coord_a.stop()
        await coord_b.stop()


@pytest.mark.asyncio
async def test_shell_discover_empty_when_no_running_files(nexus_grid_tmp: Path) -> None:
    """A coordinator that has just started but has not yet had
    ``write_running_state`` called on it yields an empty discover
    response. (This is the test-only path — the CLI ``start``
    command calls ``write_running_state`` before serving.)"""
    coord = Coordinator(project_name="shell-empty")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/shell/discover")
            assert r.status_code == 200
            body = r.json()
            assert body["count"] == 0
            assert body["coordinators"] == []
            assert body["self"]["project_name"] == "shell-empty"
    finally:
        await coord.stop()
