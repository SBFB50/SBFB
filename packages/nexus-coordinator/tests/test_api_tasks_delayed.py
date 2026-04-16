# SPDX-License-Identifier: AGPL-3.0-or-later
"""Integration tests for the Sprint 19 Phase D delayed upload queue.

Exercise the full ``/tasks/submit`` → upload_queue → dispatcher
path end-to-end. Two scenarios:

1. Queue enabled, short mean + short flush interval: the submit
   returns ``200`` with a ``task_id`` immediately, the task sits
   in ``delayed_uploads`` briefly, then lands on the doc after
   the scheduler fires.
2. Queue hard cap reached: the submit returns ``429 Too Many
   Requests`` with a ``Retry-After`` header.
"""

from __future__ import annotations

import asyncio
import time
from pathlib import Path

import aiosqlite
import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


async def _doc_has_task(coord: Coordinator) -> bool:
    entries = await coord.state.doc.get_many_by_prefix(b"task:")
    return len(entries) > 0


@pytest.mark.asyncio
async def test_api_submit_pipes_through_queue_and_eventually_lands(
    nexus_grid_tmp: Path,
) -> None:
    """``POST /tasks/submit`` returns 200 immediately (task in
    queue, not yet on the doc), then the scheduler fires and the
    task ends up on the doc. We set ``mean_jitter_s=0.05`` and
    ``flush_interval_s=0.05`` so the whole round-trip completes
    well under the test timeout without relying on freezegun.
    """
    coord = Coordinator(project_name="qd-submit-pipe")
    coord.config.upload_queue.enabled = True
    coord.config.upload_queue.mean_jitter_s = 0.05
    # max_jitter must be > flush_interval per UploadQueue invariant.
    coord.config.upload_queue.max_jitter_s = 1.0
    coord.config.upload_queue.flush_interval_s = 0.05
    await coord.start()
    try:
        assert coord.upload_queue is not None
        upload_db = coord.project_dir / "upload_queue.sqlite"

        with TestClient(create_app(coord)) as client:
            response = client.post(
                "/tasks/submit",
                json={"task_type": "analysis", "prompt": "p", "model": "m"},
            )
        assert response.status_code == 200, response.json()
        task_id = response.json()["task_id"]
        assert task_id.startswith("t-")

        # Poll until the task reaches the doc, with a generous cap
        # (~5s) that is still order-of-magnitude safe versus the
        # 60s pytest timeout inherited from pyproject.toml.
        deadline = time.monotonic() + 5.0
        while time.monotonic() < deadline:
            if await _doc_has_task(coord):
                break
            await asyncio.sleep(0.05)
        assert await _doc_has_task(coord), "task never landed on doc"

        # After landing, the upload_queue row was deleted.
        async with aiosqlite.connect(upload_db) as db:
            async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
                remaining = await cursor.fetchone()
        assert remaining[0] == 0, "delayed_uploads still has rows after flush"

        # The dispatcher's task_state mirror also picked up the row.
        assert coord.dispatcher is not None
        tasks = await coord.dispatcher.list_tasks()
        assert len(tasks) == 1
        assert tasks[0]["task_id"] == task_id
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_submit_returns_429_past_hard_cap(
    nexus_grid_tmp: Path,
) -> None:
    """Over the configured hard cap, the API must return 429 Too
    Many Requests with a ``Retry-After`` header. Uses a tiny cap
    (2) so the test stays fast without actually queuing 100k tasks.
    """
    coord = Coordinator(project_name="qd-submit-cap")
    coord.config.upload_queue.enabled = True
    # Huge mean so nothing flushes during the test window — we
    # want the rows to stay in the queue long enough for the
    # third submit to trip the cap.
    coord.config.upload_queue.mean_jitter_s = 3600.0
    coord.config.upload_queue.max_jitter_s = 7200.0
    coord.config.upload_queue.flush_interval_s = 60.0
    coord.config.upload_queue.soft_cap = 2
    coord.config.upload_queue.hard_cap = 2
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            for i in range(2):
                r = client.post(
                    "/tasks/submit",
                    json={
                        "task_type": "analysis",
                        "prompt": f"p{i}",
                        "model": "m",
                    },
                )
                assert r.status_code == 200, r.json()

            r_over = client.post(
                "/tasks/submit",
                json={"task_type": "analysis", "prompt": "over", "model": "m"},
            )
            assert r_over.status_code == 429, r_over.json()
            assert r_over.headers.get("Retry-After") is not None
    finally:
        await coord.stop()
