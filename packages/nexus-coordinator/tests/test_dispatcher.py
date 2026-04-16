# SPDX-License-Identifier: AGPL-3.0-or-later
"""Dispatcher tests: submit → doc write → task_state row.

Uses a real in-process coordinator (via the nexus_grid_tmp
fixture) so every submit round-trips through the real iroh-docs
wrapper and the SQLite mirror.
"""

from __future__ import annotations

from pathlib import Path

import nexus_core
import pytest
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.dispatcher import SubmitRequest


@pytest.mark.asyncio
async def test_submit_creates_doc_entry_and_state_row(
    nexus_grid_tmp: Path,
) -> None:
    coord = Coordinator(project_name="demo-dispatch")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        task_id = await coord.dispatcher.submit(
            SubmitRequest(
                task_type="analysis",
                prompt="Echo hello",
                model="stub-model:latest",
            )
        )
        assert task_id.startswith("t-")

        # task_state has exactly one pending row.
        tasks = await coord.dispatcher.list_tasks()
        assert len(tasks) == 1
        assert tasks[0]["state"] == "pending"
        assert tasks[0]["task_id"] == task_id

        # The doc has exactly one task:* entry under the coordinator
        # author, and its JSON value verifies.
        entries = await coord.state.doc.get_many_by_prefix(b"task:")
        assert len(entries) == 1
        e = entries[0]
        assert e["key"] == f"task:{task_id}".encode("utf-8")
        blob = await coord.state.node.blobs().get_bytes(e["hash"])
        nexus_core.verify_task_entry(blob.decode("utf-8"))  # raises on failure
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_submit_many_tasks_preserves_order(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="demo-many")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        ids = []
        for i in range(10):
            tid = await coord.dispatcher.submit(
                SubmitRequest(
                    task_type="analysis",
                    prompt=f"Task number {i}",
                    model="stub-model:latest",
                    priority=5,
                )
            )
            ids.append(tid)

        tasks = await coord.dispatcher.list_tasks(limit=50)
        assert len(tasks) == 10
        assert {t["task_id"] for t in tasks} == set(ids)
        assert all(t["state"] == "pending" for t in tasks)

        # list_tasks with a state filter.
        pending = await coord.dispatcher.list_tasks(state="pending")
        assert len(pending) == 10
        claimed = await coord.dispatcher.list_tasks(state="claimed")
        assert len(claimed) == 0
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_mark_claimed_and_completed_update_state(
    nexus_grid_tmp: Path,
) -> None:
    coord = Coordinator(project_name="demo-states")
    await coord.start()
    try:
        d = coord.dispatcher
        assert d is not None
        tid = await d.submit(SubmitRequest(task_type="a", prompt="p", model="m"))
        worker_pk = b"\x09" * 32
        await d.mark_claimed(tid, worker_pk)
        tasks = await d.list_tasks()
        assert tasks[0]["state"] == "claimed"

        await d.mark_completed(tid, b"\xaa" * 32)
        tasks = await d.list_tasks()
        assert tasks[0]["state"] == "completed"
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# Sprint 18 Phase D — TaskEntry wire-through (is_open_source + estimates)
# ---------------------------------------------------------------------------


async def _read_last_task_dict(coord: Coordinator) -> dict[str, object]:
    """Fetch the only ``task:*`` entry on the project doc and
    return the decoded JSON of the signed TaskEntry's ``task``
    sub-object. Dispatcher tests use this to assert that the 4
    Phase D fields land in the canonical bytes the worker sees.
    """
    import json

    entries = await coord.state.doc.get_many_by_prefix(b"task:")
    assert len(entries) == 1
    blob = await coord.state.node.blobs().get_bytes(entries[0]["hash"])
    entry = json.loads(blob.decode("utf-8"))
    return entry["task"]  # type: ignore[no-any-return]


@pytest.mark.asyncio
async def test_submit_defaults_is_open_source_false_when_not_set(
    nexus_grid_tmp: Path,
) -> None:
    coord = Coordinator(project_name="d-default-closed")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        await coord.dispatcher.submit(
            SubmitRequest(task_type="a", prompt="p", model="m"),
        )
        task = await _read_last_task_dict(coord)
        assert task["is_open_source"] is False
        assert task["estimated_watts"] == 0
        assert task["estimated_vram_mb"] == 0
        assert task["estimated_hours"] == 0.0
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_submit_writes_phase_d_fields_verbatim(
    nexus_grid_tmp: Path,
) -> None:
    coord = Coordinator(project_name="d-verbatim")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        await coord.dispatcher.submit(
            SubmitRequest(
                task_type="a",
                prompt="p",
                model="m",
                is_open_source=True,
                estimated_watts=250,
                estimated_vram_mb=8000,
                estimated_hours=0.25,
            ),
        )
        task = await _read_last_task_dict(coord)
        assert task["is_open_source"] is True
        assert task["estimated_watts"] == 250
        assert task["estimated_vram_mb"] == 8000
        assert task["estimated_hours"] == 0.25
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_derives_is_open_source_from_identity_repo_url(
    nexus_grid_tmp: Path,
) -> None:
    """/tasks/submit handler derives ``is_open_source=true`` when
    ``config.identity.repo_url`` is set, even though the HTTP body
    never carries the field. Confirms the Sprint 16 D-1 invariant
    still holds end-to-end after Phase D wire-through.
    """
    from fastapi.testclient import TestClient
    from nexus_coordinator.api.app import create_app

    coord = Coordinator(project_name="d-identity-repo")
    coord.config.identity.repo_url = "https://github.com/example/app"
    # Sprint 19 Phase D — disable the delayed upload queue so the
    # task lands on the doc before _read_last_task_dict runs. The
    # queue's semantics are covered end-to-end in
    # tests/test_api_tasks_delayed.py.
    coord.config.upload_queue.enabled = False
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.post(
                "/tasks/submit",
                json={"task_type": "a", "prompt": "p", "model": "m"},
            )
            assert r.status_code == 200, r.json()
        task = await _read_last_task_dict(coord)
        assert task["is_open_source"] is True
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_uses_registered_app_cost_estimate(
    nexus_grid_tmp: Path,
) -> None:
    """/tasks/submit with ``app_name`` pointing at a registered
    NexusApp picks up that app's ``cost_estimate()`` override.
    """
    from fastapi.testclient import TestClient
    from nexus_coordinator.api.app import create_app
    from nexus_sdk import AppContext, AppManifest, NexusApp

    class HeavyApp(NexusApp):
        manifest = AppManifest(name="heavy", version="0.1.0")

        async def on_start(self, ctx: AppContext) -> None:
            pass

        async def on_stop(self) -> None:
            pass

        def cost_estimate(self) -> tuple[int, int, float]:
            return (350, 14000, 0.45)

    coord = Coordinator(project_name="d-app-hint")
    coord.config.upload_queue.enabled = False
    await coord.start()
    try:
        coord.apps["heavy"] = HeavyApp()
        with TestClient(create_app(coord)) as client:
            r = client.post(
                "/tasks/submit",
                json={
                    "task_type": "a",
                    "prompt": "p",
                    "model": "m",
                    "app_name": "heavy",
                },
            )
            assert r.status_code == 200, r.json()
        task = await _read_last_task_dict(coord)
        assert task["estimated_watts"] == 350
        assert task["estimated_vram_mb"] == 14000
        assert task["estimated_hours"] == 0.45
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_falls_back_to_sdk_defaults_when_app_missing(
    nexus_grid_tmp: Path,
) -> None:
    """Submitting with a stranger ``app_name`` (or none) yields
    the conservative SDK fallback estimate ``(100, 2000, 0.1)``.
    """
    from fastapi.testclient import TestClient
    from nexus_coordinator.api.app import create_app

    coord = Coordinator(project_name="d-fallback")
    coord.config.upload_queue.enabled = False
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.post(
                "/tasks/submit",
                json={
                    "task_type": "a",
                    "prompt": "p",
                    "model": "m",
                    "app_name": "not-registered",
                },
            )
            assert r.status_code == 200, r.json()
        task = await _read_last_task_dict(coord)
        assert task["estimated_watts"] == 100
        assert task["estimated_vram_mb"] == 2000
        assert task["estimated_hours"] == 0.1
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_ignores_client_attempt_to_set_is_open_source(
    nexus_grid_tmp: Path,
) -> None:
    """Regression for the Sprint 16 D-1 invariant. A client that
    tacks ``is_open_source=true`` and ``estimated_watts=9999`` onto
    the JSON body must NOT see those values echoed into the signed
    TaskEntry. The handler always re-derives them server-side.
    """
    from fastapi.testclient import TestClient
    from nexus_coordinator.api.app import create_app

    coord = Coordinator(project_name="d-no-override")
    # private project, no repo_url → is_open_source must be False.
    coord.config.upload_queue.enabled = False
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.post(
                "/tasks/submit",
                json={
                    "task_type": "a",
                    "prompt": "p",
                    "model": "m",
                    "is_open_source": True,
                    "estimated_watts": 9999,
                    "estimated_vram_mb": 999_999,
                    "estimated_hours": 99.9,
                },
            )
            assert r.status_code == 200, r.json()
        task = await _read_last_task_dict(coord)
        assert task["is_open_source"] is False
        assert task["estimated_watts"] == 100  # SDK default
        assert task["estimated_vram_mb"] == 2000
        assert task["estimated_hours"] == 0.1
    finally:
        await coord.stop()
