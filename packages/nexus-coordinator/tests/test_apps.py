"""Coordinator app-loader integration tests.

Phase D closure requires that ``nexus-coordinator start`` picks
up the gov and hello-world apps via their entry_points and that
their manifests are reachable through ``/app/{name}/manifest``.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


@pytest.mark.asyncio
async def test_coordinator_discovers_gov_and_hello_apps(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-apps")
    await coord.start()
    try:
        names = {app.manifest.name for app in coord.apps.values()}
        assert "gov" in names
        assert "hello" in names
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_app_manifest_endpoint_returns_gov(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-apps-http")
    await coord.start()
    try:
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app")
            assert r.status_code == 200
            listed = r.json()
            assert listed["count"] >= 2
            names = {a["name"] for a in listed["apps"]}
            assert "gov" in names
            assert "hello" in names

            r = client.get("/app/gov/manifest")
            assert r.status_code == 200
            body = r.json()
            assert body["manifest"]["name"] == "gov"
            assert len(body["routes"]) == 1
            assert body["routes"][0]["path"] == "/statements"
            assert len(body["workers"]) == 1
            assert body["workers"][0]["name"] == "contradiction_detector"
            assert len(body["tabs"]) == 1
            assert body["tabs"][0]["name"] == "Contradictions"

            r = client.get("/app/hello/manifest")
            assert r.status_code == 200
            hello_body = r.json()
            assert hello_body["manifest"]["name"] == "hello"

            r = client.get("/app/does-not-exist/manifest")
            assert r.status_code == 404
    finally:
        await coord.stop()
