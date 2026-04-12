# SPDX-License-Identifier: AGPL-3.0-or-later
"""End-to-end boot test for the coordinator.

Spins up a real iroh Node in-process via
:class:`nexus_coordinator.coordinator.Coordinator`, verifies the
project doc is created, the author id is minted, the write ticket
is minted, and the FastAPI app responds on ``/health`` and
``/project``.

No network access required — iroh runs loopback-only when no
peers are introduced, and the coordinator fixture uses a tmp
directory as its data dir.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


@pytest.mark.asyncio
async def test_coordinator_boots_creates_doc_and_author(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-boot")
    await coord.start()
    try:
        assert coord.state.node is not None
        assert coord.state.doc is not None
        assert coord.state.node_id, "node_id must be populated after start"
        assert coord.state.author_id, "author_id must be minted on first boot"
        assert coord.state.doc_id, "doc_id must be populated"
        assert coord.state.tasks_doc_ticket, "write ticket must be minted"
        # Config should have been persisted with the new author + doc ids.
        assert coord.config.identity.author_id == coord.state.author_id
        assert coord.config.identity.doc_id == coord.state.doc_id
        assert coord.config_path.exists()
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_health_and_project_endpoints_respond(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-http")
    coord.config.identity.description = "integration test"
    await coord.start()
    try:
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/health")
            assert r.status_code == 200
            body = r.json()
            assert body["status"] == "ok"
            assert body["project"] == "demo-http"
            assert body["node_id"] == coord.state.node_id
            assert body["doc_id"] == coord.state.doc_id

            r = client.get("/project")
            assert r.status_code == 200
            body = r.json()
            assert body["name"] == "demo-http"
            assert body["description"] == "integration test"
            assert body["visibility"] == "private"
            assert body["doc_id"] == coord.state.doc_id
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_coordinator_reboot_reuses_doc_and_author(
    nexus_grid_tmp: Path,
) -> None:
    """A second start on the same project directory must return
    the same doc_id / author_id persisted from the first run."""
    coord1 = Coordinator(project_name="demo-reboot")
    await coord1.start()
    first_doc_id = coord1.state.doc_id
    first_author_id = coord1.state.author_id
    first_node_id = coord1.state.node_id
    await coord1.stop()

    coord2 = Coordinator(project_name="demo-reboot")
    await coord2.start()
    try:
        assert coord2.state.doc_id == first_doc_id, "doc_id must persist across reboots"
        # Node id is derived from the persistent secret key, so it
        # must be identical.
        assert coord2.state.node_id == first_node_id, "node_id must persist across reboots"
        # author_id survives via coordinator.toml persistence.
        assert coord2.state.author_id == first_author_id
    finally:
        await coord2.stop()
