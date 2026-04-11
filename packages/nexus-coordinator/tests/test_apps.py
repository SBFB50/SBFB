"""Coordinator app-loader integration tests.

Phase D closure requires that ``nexus-coordinator start`` picks
up the gov and hello-world apps via their entry_points and that
their manifests are reachable through ``/app/{name}/manifest``.

Sprint 8 Phase A updates this suite to:

- remove the legacy_descriptor fallback assertions (D4)
- exercise the new Sprint 8 D1 ``/tasks/submit`` route
- exercise the new Sprint 8 D2 ``/commands`` + ``/commands/{cmd}/invoke`` routes
- assert that a bad tab descriptor now raises HTTP 422 (no
  silent legacy fallback anymore)
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

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
        # Sprint 8 Phase A: coord.app_contexts must carry a
        # populated AppContext for every mounted app — the
        # /tasks/submit route depends on this invariant.
        assert set(coord.app_contexts.keys()) == set(coord.apps.keys())
    finally:
        await coord.stop()


def test_app_db_path_resolves_under_project_tree(nexus_grid_tmp: Path) -> None:
    """Sprint 8 Phase B (D3 path helper): ``app_db_path`` lives at
    ``<nexus-grid-root>/projects/<project>/apps/<app>/app.sqlite``
    so the per-app SQLite file stays inside the per-project
    directory tree the coordinator already owns."""
    from nexus_coordinator.paths import app_db_path as _app_db_path

    expected = nexus_grid_tmp / "projects" / "demo-project" / "apps" / "demo-app" / "app.sqlite"
    assert _app_db_path("demo-project", "demo-app") == expected
    # app_db_path must be a pure path computation — it must not
    # create the parent directory at call time (that's the
    # loader's job, not the helper's).
    assert not expected.parent.exists()


@pytest.mark.asyncio
async def test_app_db_wired_in_loader(nexus_grid_tmp: Path) -> None:
    """Sprint 8 Phase B (D3 wiring): every app_context gets an
    :class:`AppDatabaseClient` instance wired by the coordinator
    loader. The default path is the per-app SQLite under the
    project tree; an app that overrides ``ctx.db`` in its
    ``on_start`` hook can still do so (tested separately via
    the gov app suite)."""
    from nexus_sdk import AppDatabaseClient

    coord = Coordinator(project_name="demo-db-wire")
    await coord.start()
    try:
        for name, ctx in coord.app_contexts.items():
            assert ctx.db is not None, f"app {name!r} got no AppContext.db"
            assert isinstance(ctx.db, AppDatabaseClient)
            # The default path resolves under the per-project
            # tree; the parent directory exists (the loader
            # mkdirs it before AppDatabaseClient construction).
            assert ctx.db.db_path.parent.exists()
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
            # Sprint 8 A: the per-app summary gained a `commands`
            # count. Zero for the pre-Sprint-8 apps; we just
            # assert it exists as an int field.
            for entry in listed["apps"]:
                assert isinstance(entry.get("commands"), int)

            r = client.get("/app/gov/manifest")
            assert r.status_code == 200
            body = r.json()
            assert body["manifest"]["name"] == "gov"
            assert len(body["routes"]) == 1
            assert body["routes"][0]["path"] == "/statements"
            # Sprint 8 Phase D: the gov manifest now advertises
            # three workers — the Sprint 4 contradiction_detector
            # stub plus the two RAG workers rag_search (on
            # nomic-embed-text) and rag_ask (on the heretic gemma
            # model) introduced by Phase D.
            worker_models = {w["name"]: w["model"] for w in body["workers"]}
            assert set(worker_models.keys()) == {
                "contradiction_detector",
                "rag_search",
                "rag_ask",
            }
            assert worker_models["contradiction_detector"] == "stub-model:latest"
            assert worker_models["rag_search"] == "nomic-embed-text"
            assert worker_models["rag_ask"] == "juilpark/gemma-4-26B-A4B-it-heretic:q4_k_m"
            # Sprint 8 Phase D: the gov manifest now ships
            # nineteen tabs — thirteen Batch 1+2 tabs carried over
            # from Phase C plus six Batch 3 tabs (Alertes, Affaires,
            # Lois, Factchecks, Recherche, Question).
            tab_names = {t["name"] for t in body["tabs"]}
            assert tab_names == {
                "Contradictions",
                "Dashboard",
                "Politiciens",
                "Politicien",
                "Biographie",
                "Positions",
                "Sujets",
                "Scan",
                "Workers",
                "Pipeline",
                "Social",
                "Presse",
                "Transcriptions",
                "Alertes",
                "Affaires",
                "Lois",
                "Factchecks",
                "Recherche",
                "Question",
            }
            # Sprint 8 A: manifest endpoint now ships a `commands`
            # list (empty for the pre-Sprint-8 gov stub).
            assert body["commands"] == []

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
    """Sprint 5 Phase B + Sprint 6 Phase A + Sprint 8 Phase A (D4):
    ``GET /app/{name}/tabs/{tab_name}/descriptor`` invokes the tab
    fn (sync or async) and returns ``{descriptor: TabView}`` on
    success. The Sprint 6 ``legacy_descriptor`` fallback is
    retired: an invalid descriptor now fails the request with
    HTTP 422 instead of shipping under a legacy flag.

    Exercises the hello-world app's "Hello" tab which is a sync
    function returning a valid :class:`TabView` (post Sprint 6
    Phase B port).
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
            # Sprint 8 D4: the legacy_descriptor flag is gone.
            assert "legacy_descriptor" not in body
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
    through the coordinator as ``{descriptor: <dumped>}`` (no
    ``legacy_descriptor`` flag — Sprint 8 D4 removal)."""
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
            # Sprint 8 D4: no more legacy_descriptor field.
            assert "legacy_descriptor" not in body
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
async def test_tab_descriptor_raises_422_on_invalid_schema(nexus_grid_tmp: Path) -> None:
    """Sprint 8 Phase A (D4): a tab that returns a non-TabView
    payload used to trip the legacy_descriptor fallback. The
    fallback is gone — the coordinator now fails the request
    with HTTP 422 and a detail message carrying the TabView
    error count, so the shell renders a visible error banner
    instead of silently displaying degraded data."""
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

    coord = Coordinator(project_name="demo-legacy-422")
    await coord.start()
    try:
        coord.apps["legacy-app"] = LegacyApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app/legacy-app/tabs/Old/descriptor")
            assert r.status_code == 422
            detail = r.json()["detail"]
            assert "Old" in detail
            assert "legacy-app" in detail
            assert "TabView" in detail
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# Sprint 8 Phase A — D1 submit_task route
# ---------------------------------------------------------------------------


class _FakeComputeClient:
    """Stub ComputeClient that records the last submit_task call
    and returns a deterministic task id. Bypasses the real
    dispatcher so the route tests don't need a live project
    doc / allowlist setup."""

    def __init__(self) -> None:
        self.last_kwargs: dict[str, Any] | None = None

    async def submit_task(self, **kwargs: Any):  # type: ignore[no-untyped-def]
        self.last_kwargs = kwargs

        class _Submitted:
            task_id = "task-sprint8-a"

        return _Submitted()


@pytest.mark.asyncio
async def test_submit_app_task_happy_path(nexus_grid_tmp: Path) -> None:
    """Sprint 8 D1: POST /app/{name}/tasks/submit delegates to
    the app's bound AppContext and returns the task id the
    dispatcher assigned."""
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_worker

    class SubmitFixtureApp(NexusApp):
        manifest = AppManifest(name="submitfx", version="0.1.0")

        @nexus_worker(name="echo", model="stub-model:latest")
        async def worker_echo(self, ctx):  # type: ignore[no-untyped-def]
            return {}

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    coord = Coordinator(project_name="demo-submit-a")
    await coord.start()
    try:
        fake_app = SubmitFixtureApp()
        fake_compute = _FakeComputeClient()
        coord.apps["submitfx"] = fake_app
        coord.app_contexts["submitfx"] = AppContext(
            compute=fake_compute,  # type: ignore[arg-type]
            project_name=coord.project_name,
            app_name="submitfx",
            _app=fake_app,
        )

        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post(
                "/app/submitfx/tasks/submit",
                json={
                    "worker": "echo",
                    "payload": {"q": "hello", "n": 3},
                    "priority": 7,
                    "parent_task_id": "parent-9",
                },
            )
            assert r.status_code == 200
            assert r.json() == {"task_id": "task-sprint8-a"}
            assert fake_compute.last_kwargs is not None
            assert fake_compute.last_kwargs["task_type"] == "echo"
            assert fake_compute.last_kwargs["model"] == "stub-model:latest"
            assert fake_compute.last_kwargs["priority"] == 7
            # Sorted JSON is a deterministic prompt contract.
            assert fake_compute.last_kwargs["prompt"] == '{"n": 3, "q": "hello"}'
            assert fake_compute.last_kwargs["metadata"] == {"parent_task_id": "parent-9"}
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_submit_app_task_unknown_app_404(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-submit-404")
    await coord.start()
    try:
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post(
                "/app/ghost/tasks/submit",
                json={"worker": "x", "payload": {}},
            )
            assert r.status_code == 404
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_submit_app_task_unknown_worker_422(nexus_grid_tmp: Path) -> None:
    """Sprint 8 D1: a routing key that resolve_worker cannot
    match surfaces as HTTP 422 with the WorkerNotFound message
    in the detail — the shell can render it verbatim."""
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_worker

    class SubmitFixtureApp(NexusApp):
        manifest = AppManifest(name="submitfx2", version="0.1.0")

        @nexus_worker(name="echo", model="stub-model")
        async def worker_echo(self, ctx):  # type: ignore[no-untyped-def]
            return {}

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    coord = Coordinator(project_name="demo-submit-badworker")
    await coord.start()
    try:
        fake_app = SubmitFixtureApp()
        coord.apps["submitfx2"] = fake_app
        coord.app_contexts["submitfx2"] = AppContext(
            compute=_FakeComputeClient(),  # type: ignore[arg-type]
            project_name=coord.project_name,
            app_name="submitfx2",
            _app=fake_app,
        )
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post(
                "/app/submitfx2/tasks/submit",
                json={"worker": "ghost", "payload": {}},
            )
            assert r.status_code == 422
            assert "ghost" in r.json()["detail"]
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# Sprint 8 Phase A — D2 commands routes
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_list_app_commands_returns_descriptors(nexus_grid_tmp: Path) -> None:
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_command

    class CmdApp(NexusApp):
        manifest = AppManifest(name="cmdroute", version="0.1.0")

        @nexus_command("detect", description="Détecter")
        async def cmd_detect(self) -> None:
            return None

        @nexus_command("refresh", description="Rafraîchir", icon="refresh", group="Gov")
        async def cmd_refresh(self) -> None:
            return None

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    coord = Coordinator(project_name="demo-cmds-list")
    await coord.start()
    try:
        coord.apps["cmdroute"] = CmdApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.get("/app/cmdroute/commands")
            assert r.status_code == 200
            cmds = r.json()
            assert len(cmds) == 2
            names = {c["name"] for c in cmds}
            assert names == {"detect", "refresh"}
            # Schema version must be present in the serialized
            # descriptor — the shell Zod mirror asserts on it.
            assert all(c["schema_version"] == 1 for c in cmds)
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_invoke_app_command_runs_method(nexus_grid_tmp: Path) -> None:
    from nexus_sdk import AppContext, AppManifest, NexusApp, nexus_command

    class CmdApp(NexusApp):
        manifest = AppManifest(name="cmdinvoke", version="0.1.0")
        called = False

        @nexus_command("go", description="Go")
        async def cmd_go(self) -> dict[str, Any]:
            type(self).called = True
            return {"navigation": {"path": "/app/cmdinvoke/tabs/home"}}

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    coord = Coordinator(project_name="demo-cmd-invoke")
    await coord.start()
    try:
        coord.apps["cmdinvoke"] = CmdApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post("/app/cmdinvoke/commands/go/invoke")
            assert r.status_code == 200
            body = r.json()
            assert body["result"] == {"navigation": {"path": "/app/cmdinvoke/tabs/home"}}
            assert CmdApp.called is True
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_invoke_app_command_unknown_raises_404(nexus_grid_tmp: Path) -> None:
    from nexus_sdk import AppContext, AppManifest, NexusApp

    class EmptyCmdApp(NexusApp):
        manifest = AppManifest(name="emptycmd", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

    coord = Coordinator(project_name="demo-cmd-unknown")
    await coord.start()
    try:
        coord.apps["emptycmd"] = EmptyCmdApp()
        app = create_app(coord)
        with TestClient(app) as client:
            r = client.post("/app/emptycmd/commands/ghost/invoke")
            assert r.status_code == 404
    finally:
        await coord.stop()
