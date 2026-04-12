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
