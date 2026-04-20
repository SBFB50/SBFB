# SPDX-License-Identifier: AGPL-3.0-or-later
"""Redundancy voting tests — Sprint 23 Phase D."""

from __future__ import annotations

from pathlib import Path

import pytest
from nexus_coordinator.redundancy import (
    RedundancyDispatcher,
    VoteVerdict,
    hash_result_bytes,
)

# -------------------------------------------------------------------
# Pure unit tests — redundancy module
# -------------------------------------------------------------------


def test_dispatch_factor_1_passthrough() -> None:
    """factor=1 task is not registered as redundant."""
    rd = RedundancyDispatcher()
    rd.register_task("t-1", factor=1)
    assert not rd.is_redundant("t-1")


def test_dispatch_factor_3_three_workers() -> None:
    """factor=3 task is tracked and needs 3 results before vote."""
    rd = RedundancyDispatcher()
    rd.register_task("t-3", factor=3)
    assert rd.is_redundant("t-3")

    result = rd.collect_result("t-3", "w-a", b"same-output")
    assert result is None
    result = rd.collect_result("t-3", "w-b", b"same-output")
    assert result is None
    result = rd.collect_result("t-3", "w-c", b"same-output")
    assert result is not None
    assert result.verdict == VoteVerdict.MAJORITY


def test_collect_all_match() -> None:
    """3 identical results → Majority with no outliers."""
    rd = RedundancyDispatcher()
    rd.register_task("t-all", factor=3)
    payload = b"identical result bytes"

    rd.collect_result("t-all", "w-1", payload)
    rd.collect_result("t-all", "w-2", payload)
    outcome = rd.collect_result("t-all", "w-3", payload)

    assert outcome is not None
    assert outcome.verdict == VoteVerdict.MAJORITY
    assert outcome.canonical_hash == hash_result_bytes(payload)
    assert outcome.outlier_worker_ids == []


def test_collect_2_of_3_match() -> None:
    """2 match, 1 differs → Majority + quarantine the outlier."""
    rd = RedundancyDispatcher()
    rd.register_task("t-2of3", factor=3)

    rd.collect_result("t-2of3", "w-good-1", b"correct")
    rd.collect_result("t-2of3", "w-bad", b"wrong")
    outcome = rd.collect_result("t-2of3", "w-good-2", b"correct")

    assert outcome is not None
    assert outcome.verdict == VoteVerdict.MAJORITY
    assert outcome.canonical_hash == hash_result_bytes(b"correct")
    assert outcome.outlier_worker_ids == ["w-bad"]


def test_collect_all_differ() -> None:
    """3 different results → Mismatch, all quarantined."""
    rd = RedundancyDispatcher()
    rd.register_task("t-diff", factor=3)

    rd.collect_result("t-diff", "w-a", b"aaa")
    rd.collect_result("t-diff", "w-b", b"bbb")
    outcome = rd.collect_result("t-diff", "w-c", b"ccc")

    assert outcome is not None
    assert outcome.verdict == VoteVerdict.MISMATCH
    assert set(outcome.outlier_worker_ids) == {"w-a", "w-b", "w-c"}


def test_quarantine_notifies_curator() -> None:
    """quarantine_outliers records worker IDs for curator pickup."""
    rd = RedundancyDispatcher()
    rd.register_task("t-q", factor=3)

    rd.collect_result("t-q", "w-1", b"ok")
    rd.collect_result("t-q", "w-2", b"bad")
    outcome = rd.collect_result("t-q", "w-3", b"ok")

    assert outcome is not None
    rd.quarantine_outliers("t-q", outcome.outlier_worker_ids)
    q = rd.get_quarantined("t-q")
    assert "w-2" in q


def test_factor_5_majority() -> None:
    """3/5 match → Majority (threshold = 3)."""
    rd = RedundancyDispatcher()
    rd.register_task("t-5", factor=5)

    rd.collect_result("t-5", "w-1", b"correct")
    rd.collect_result("t-5", "w-2", b"wrong-a")
    rd.collect_result("t-5", "w-3", b"correct")
    rd.collect_result("t-5", "w-4", b"wrong-b")
    outcome = rd.collect_result("t-5", "w-5", b"correct")

    assert outcome is not None
    assert outcome.verdict == VoteVerdict.MAJORITY
    assert set(outcome.outlier_worker_ids) == {"w-2", "w-4"}


def test_hash_canonical_deterministic() -> None:
    """Same bytes → same hash, different bytes → different hash."""
    a = hash_result_bytes(b"hello world")
    b = hash_result_bytes(b"hello world")
    c = hash_result_bytes(b"hello world!")
    assert a == b
    assert a != c
    assert len(a) == 64  # SHA-256 hex digest


# -------------------------------------------------------------------
# Integration tests — dispatcher routing + API
# -------------------------------------------------------------------


@pytest.mark.asyncio
async def test_dispatcher_routes_redundant(nexus_grid_tmp: Path) -> None:
    """factor>1 task registers with the RedundancyDispatcher."""
    from nexus_coordinator.coordinator import Coordinator
    from nexus_coordinator.dispatcher import SubmitRequest

    rd = RedundancyDispatcher()
    coord = Coordinator(project_name="demo-redundant")
    await coord.start()
    try:
        assert coord.dispatcher is not None
        coord.dispatcher._redundancy_dispatcher = rd  # noqa: SLF001

        await coord.dispatcher.submit(
            SubmitRequest(
                task_type="analysis",
                prompt="Echo hello",
                model="stub-model:latest",
                redundancy_factor=3,
            )
        )
        tasks = await coord.dispatcher.list_tasks()
        assert len(tasks) == 1
        tid = tasks[0]["task_id"]
        assert rd.is_redundant(tid)
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_api_accepts_redundancy_factor(nexus_grid_tmp: Path) -> None:
    """POST /tasks/submit with redundancy_factor=3 persists it."""
    import json

    from fastapi.testclient import TestClient
    from nexus_coordinator.api.app import create_app
    from nexus_coordinator.coordinator import Coordinator

    coord = Coordinator(project_name="demo-api-rf")
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
                    "redundancy_factor": 3,
                },
            )
            assert r.status_code == 200, r.json()

        entries = await coord.state.doc.get_many_by_prefix(b"task:")
        assert len(entries) == 1
        blob = await coord.state.node.blobs().get_bytes(entries[0]["hash"])
        entry = json.loads(blob.decode("utf-8"))
        assert entry["task"]["redundancy_factor"] == 3
    finally:
        await coord.stop()
