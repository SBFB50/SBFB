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


@pytest.mark.asyncio
async def test_app_tab_descriptor_endpoint_invokes_tab(nexus_grid_tmp: Path) -> None:
    """Sprint 5 Phase B + Sprint 6 Phase A: ``GET /app/{name}/
    tabs/{tab_name}/descriptor`` invokes the tab fn (sync or
    async) and returns a ``{descriptor, legacy_descriptor}`` body.

    Exercises the hello-world app's "Hello" tab which is a sync
    function returning a valid :class:`TabView` (post Phase B
    port), so ``legacy_descriptor`` is ``False``.
    """
    coord = Coordinator(project_name="demo-tab-desc")
    await coord.start()
    try:
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app/hello/tabs/Hello/descriptor")
            assert r.status_code == 200
            body = r.json()
            assert "descriptor" in body
            assert body.get("legacy_descriptor") is False
            desc = body["descriptor"]
            assert desc["schema_version"] == 1
            assert desc["tab_name"] == "hello"
            assert isinstance(desc["blocks"], list)

            # Unknown app → 404
            r = client.get("/app/does-not-exist/tabs/X/descriptor")
            assert r.status_code == 404

            # Unknown tab on known app → 404
            r = client.get("/app/hello/tabs/Nope/descriptor")
            assert r.status_code == 404
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_schema_driven_descriptor_validates(nexus_grid_tmp: Path) -> None:
    """Sprint 6 D3: a valid TabView returned by a tab round-trips
    through the coordinator with ``legacy_descriptor: false`` and
    a normalised payload that the React renderer can consume."""
    from nexus_sdk import (  # local import — avoids cross-test fixture loading
        AppContext,
        AppManifest,
        NexusApp,
        nexus_tab,
    )
    from nexus_sdk.view import TabView, heading, metric, section

    class SchemaDrivenApp(NexusApp):
        manifest = AppManifest(name="schema-driven", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

        @nexus_tab(name="Dash", icon="activity")
        def dash(self) -> dict:
            return TabView(
                tab_name="dash",
                title="Dashboard",
                blocks=[
                    heading(level=1, text="Metrics"),
                    metric(label="Total", value=42, tone="ok"),
                    section(title="Empty", blocks=[]),
                ],
            ).model_dump()

    coord = Coordinator(project_name="demo-schema-driven")
    await coord.start()
    try:
        coord.apps["schema-driven"] = SchemaDrivenApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app/schema-driven/tabs/Dash/descriptor")
            assert r.status_code == 200
            body = r.json()
            assert body["legacy_descriptor"] is False
            desc = body["descriptor"]
            assert desc["schema_version"] == 1
            assert desc["tab_name"] == "dash"
            assert desc["title"] == "Dashboard"
            assert len(desc["blocks"]) == 3
            assert desc["blocks"][0]["kind"] == "heading"
            assert desc["blocks"][1]["kind"] == "metric"
            assert desc["blocks"][1]["tone"] == "ok"
            assert desc["blocks"][2]["kind"] == "section"
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_legacy_descriptor_falls_back(nexus_grid_tmp: Path) -> None:
    """Sprint 6 D3 fallback: a non-TabView descriptor is preserved
    verbatim with ``legacy_descriptor: true`` so unported apps
    keep working through one release."""
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_tab

    class LegacyApp(NexusApp):
        manifest = AppManifest(name="legacy-app", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

        @nexus_tab(name="Old", icon="archive")
        def old(self) -> dict:
            return {"description": "legacy free-form dict", "rows": [1, 2, 3]}

    coord = Coordinator(project_name="demo-legacy")
    await coord.start()
    try:
        coord.apps["legacy-app"] = LegacyApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app/legacy-app/tabs/Old/descriptor")
            assert r.status_code == 200
            body = r.json()
            assert body["legacy_descriptor"] is True
            assert body["descriptor"] == {
                "description": "legacy free-form dict",
                "rows": [1, 2, 3],
            }
    finally:
        await coord.stop()


def test_legacy_descriptor_sweep_empty_apps() -> None:
    """Sprint 6 audit D-3: sweep over an empty dict returns empty."""
    from nexus_coordinator.api.apps import legacy_descriptor_sweep

    assert legacy_descriptor_sweep({}) == {}


def test_legacy_descriptor_sweep_mixed_apps() -> None:
    """Sprint 6 audit D-3: sweep flags apps whose sync tabs return
    non-TabView payloads and leaves ported apps alone."""
    from nexus_coordinator.api.apps import legacy_descriptor_sweep
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_tab
    from nexus_sdk.view import TabView, heading, metric

    class PortedApp(NexusApp):
        manifest = AppManifest(name="ported", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:  # noqa: ARG002
            pass

        async def on_stop(self) -> None:
            pass

        @nexus_tab(name="Dashboard", icon="gauge")
        def dash(self) -> dict:
            return TabView(
                tab_name="dash",
                blocks=[
                    heading(level=1, text="Ported"),
                    metric(label="ok", value=1),
                ],
            ).model_dump()

    class LegacyApp(NexusApp):
        manifest = AppManifest(name="legacy", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:  # noqa: ARG002
            pass

        async def on_stop(self) -> None:
            pass

        @nexus_tab(name="Old", icon="archive")
        def old(self) -> dict:
            return {"description": "raw dict, not TabView"}

        @nexus_tab(name="Broken", icon="alert")
        def broken(self) -> dict:
            raise RuntimeError("descriptor raised at boot sweep")

    class AsyncOnlyApp(NexusApp):
        manifest = AppManifest(name="async-only", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:  # noqa: ARG002
            pass

        async def on_stop(self) -> None:
            pass

        @nexus_tab(name="Slow", icon="clock")
        async def slow(self) -> dict:
            # Async tabs must be skipped by the sweep — they would
            # otherwise hang a slow HTTP fetch during boot.
            return {"not": "reached"}

    apps: dict[str, NexusApp] = {
        "ported": PortedApp(),
        "legacy": LegacyApp(),
        "async-only": AsyncOnlyApp(),
    }
    result = legacy_descriptor_sweep(apps)

    # Ported app is not flagged
    assert "ported" not in result
    # Async-only app is not flagged (its sync tab count is zero)
    assert "async-only" not in result
    # Legacy app has both the raw-dict tab AND the raising tab
    assert "legacy" in result
    assert set(result["legacy"]) == {"Old", "Broken"}
