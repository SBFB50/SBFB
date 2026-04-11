"""Unit tests for the nexus-sdk base classes and decorators."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

import pytest
from nexus_sdk import (
    AppContext,
    AppManifest,
    ComputeClient,
    NexusApp,
    WorkerNotFound,
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


# ---------------------------------------------------------------------------
# Sprint 8 Phase A — resolve_worker + submit_task
# ---------------------------------------------------------------------------


def test_resolve_worker_short_name_matches() -> None:
    app = SampleApp()
    desc = app.resolve_worker("sample_worker")
    assert desc.name == "sample_worker"
    assert desc.model == "stub-model:latest"


def test_resolve_worker_prefixed_name_matches_own_app() -> None:
    # `<app>.<worker>` routing: the prefix must match the
    # current manifest name; the worker is then looked up
    # exactly like the bare-name case.
    app = SampleApp()
    desc = app.resolve_worker("sample.sample_worker")
    assert desc.name == "sample_worker"


def test_resolve_worker_prefixed_name_foreign_app_rejected() -> None:
    app = SampleApp()
    # Cross-app dispatch is out of Sprint 8 scope — refuse
    # loudly so the dispatcher never silently lands a task
    # on the wrong worker.
    with pytest.raises(WorkerNotFound, match="not_sample"):
        app.resolve_worker("not_sample.sample_worker")


def test_resolve_worker_unknown_name_raises() -> None:
    app = SampleApp()
    with pytest.raises(WorkerNotFound, match="ghost_worker"):
        app.resolve_worker("ghost_worker")


# ---------------------------------------------------------------------------
# submit_task wiring: the AppContext helper delegates to the
# underlying ComputeClient via resolve_worker(...). A minimal stub
# client lets us assert the call shape without spinning up a real
# coordinator.
# ---------------------------------------------------------------------------


@dataclass
class _StubSubmittedTask:
    task_id: str
    submitted_at: int = 0


class _StubComputeClient:
    """Records the last call to submit_task so assertions can
    introspect the forwarded args."""

    def __init__(self) -> None:
        self.last_kwargs: dict[str, Any] | None = None

    async def submit_task(self, **kwargs: Any) -> _StubSubmittedTask:
        self.last_kwargs = kwargs
        return _StubSubmittedTask(task_id="task-42")


@pytest.mark.asyncio
async def test_submit_task_without_app_backref_refuses() -> None:
    # A manually constructed AppContext (no backref) is what a
    # unit test might build; submit_task must refuse with a
    # clear error rather than blowing up on `None.resolve_worker`.
    ctx = AppContext(
        compute=_StubComputeClient(),  # type: ignore[arg-type]
        project_name="unit-test",
    )
    with pytest.raises(RuntimeError, match="NexusApp backref"):
        await ctx.submit_task("sample_worker", {"x": 1})


@pytest.mark.asyncio
async def test_submit_task_delegates_to_compute_client() -> None:
    app = SampleApp()
    stub = _StubComputeClient()
    ctx = AppContext(
        compute=stub,  # type: ignore[arg-type]
        project_name="unit-test",
        app_name=app.manifest.name,
        _app=app,
    )
    task_id = await ctx.submit_task(
        "sample_worker",
        {"query": "hello", "n": 3},
        priority=7,
        parent_task_id="parent-1",
    )
    assert task_id == "task-42"
    assert stub.last_kwargs is not None
    assert stub.last_kwargs["task_type"] == "sample_worker"
    # JSON must be sorted so the prompt is deterministic (helps
    # downstream dedup and canonical hashing).
    assert stub.last_kwargs["prompt"] == '{"n": 3, "query": "hello"}'
    assert stub.last_kwargs["model"] == "stub-model:latest"
    assert stub.last_kwargs["priority"] == 7
    assert stub.last_kwargs["metadata"] == {"parent_task_id": "parent-1"}


@pytest.mark.asyncio
async def test_submit_task_without_parent_id_omits_metadata() -> None:
    # When no parent_task_id is passed the helper must not send
    # an empty metadata dict (the coordinator's /tasks/submit
    # body is stricter about unknown / empty metadata fields
    # after the Sprint 4 canonical_bytes lockdown).
    app = SampleApp()
    stub = _StubComputeClient()
    ctx = AppContext(
        compute=stub,  # type: ignore[arg-type]
        project_name="unit-test",
        app_name=app.manifest.name,
        _app=app,
    )
    await ctx.submit_task("sample_worker", {})
    assert stub.last_kwargs is not None
    assert stub.last_kwargs["metadata"] is None
