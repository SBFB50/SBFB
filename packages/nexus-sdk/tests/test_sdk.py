"""Unit tests for the nexus-sdk base classes and decorators."""

from __future__ import annotations

import pytest
from nexus_sdk import (
    AppContext,
    AppManifest,
    ComputeClient,
    NexusApp,
    nexus_route,
    nexus_tab,
    nexus_worker,
)


class SampleApp(NexusApp):
    manifest = AppManifest(
        name="sample",
        version="0.1.0",
        description="Test fixture app",
    )

    @nexus_route("/ping")
    async def ping(self):
        return {"msg": "pong"}

    @nexus_route("/echo", methods=["POST", "PUT"])
    async def echo(self, body):
        return body

    @nexus_worker(name="sample_worker", model="stub-model:latest")
    async def worker(self, ctx):
        return {"ran": True}

    @nexus_tab(name="Sample", icon="book")
    def tab(self):
        return {"description": "sample tab"}

    async def on_start(self, ctx: AppContext) -> None:
        self.ctx = ctx

    async def on_stop(self) -> None:
        pass


def test_manifest_is_required() -> None:
    class Bad(NexusApp):
        async def on_start(self, ctx):  # type: ignore[override]
            pass

        async def on_stop(self):  # type: ignore[override]
            pass

    with pytest.raises(TypeError, match="manifest"):
        Bad()  # type: ignore[abstract]


def test_app_collects_routes_workers_and_tabs() -> None:
    app = SampleApp()
    routes = app.routes()
    workers = app.workers()
    tabs = app.tabs()

    assert {r.path for r in routes} == {"/ping", "/echo"}
    echo = next(r for r in routes if r.path == "/echo")
    assert "POST" in echo.methods and "PUT" in echo.methods

    assert len(workers) == 1
    assert workers[0].name == "sample_worker"
    assert workers[0].model == "stub-model:latest"

    assert len(tabs) == 1
    assert tabs[0].name == "Sample"
    assert tabs[0].icon == "book"


def test_manifest_pydantic_validation() -> None:
    with pytest.raises(ValueError):
        AppManifest(name="", version="0.1")  # type: ignore[call-arg]
    m = AppManifest(name="x", version="0.1")
    assert m.license == "AGPL-3.0"


@pytest.mark.asyncio
async def test_on_start_receives_app_context() -> None:
    app = SampleApp()
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:12345"),
        project_name="unit-test",
    )
    await app.on_start(ctx)
    assert app.ctx is ctx
    await app.on_stop()


def test_compute_client_constructs_cleanly() -> None:
    c = ComputeClient("http://127.0.0.1:8765/")
    assert c is not None  # just a sanity check; real HTTP tested separately


def test_discover_apps_returns_list() -> None:
    # No entry points registered for nexus-sdk itself, so this
    # should be an empty list — the function must still run and
    # not raise.
    from nexus_sdk import discover_apps

    apps = discover_apps()
    assert isinstance(apps, list)
