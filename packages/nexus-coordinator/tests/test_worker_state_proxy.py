# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 5 Phase A — ``GET /worker-state`` proxy endpoint tests."""

from __future__ import annotations

import json
from datetime import UTC, datetime, timedelta
from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator import paths as _paths
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


def _valid_state(last_updated_at: str | None = None) -> dict:
    """Build a minimal v1 WorkerState payload matching the Rust
    state_writer output. Keeps the fixture tiny and focused."""
    return {
        "schema_version": 1,
        "node_id": "de" * 32,
        "worker_version": "0.1.0",
        "uptime_secs": 42,
        "started_at": "2026-04-10T14:00:00Z",
        "last_updated_at": last_updated_at or datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "gpu": {
            "name": "NVIDIA GeForce RTX 5080",
            "memory_total_mb": 16384,
            "memory_used_mb": 5123,
            "utilization_pct": 42,
            "temperature_c": 61,
            "power_draw_w": 180.0,
        },
        "projects_served": [
            {
                "project_name": "demo-proj",
                "doc_id": "bb" * 32,
                "kudos_total": 0,
                "tasks_completed": 3,
            }
        ],
        "last_task": {
            "task_id": "task-1",
            "project_name": "demo-proj",
            "prompt_preview": "hello",
            "status": "completed",
            "completed_at": "2026-04-10T14:20:00Z",
        },
    }


def _write_state(state: dict, path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state), encoding="utf-8")


@pytest.mark.asyncio
async def test_worker_state_returns_not_running_when_file_absent(
    nexus_grid_tmp: Path,
) -> None:
    """No ``state.json`` on disk ⇒ ``{running: false}``."""
    coord = Coordinator(project_name="ws-absent")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/worker-state")
            assert r.status_code == 200
            assert r.json() == {"running": False}
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_worker_state_parses_fresh_snapshot(nexus_grid_tmp: Path) -> None:
    """A valid snapshot with a recent ``last_updated_at`` is
    reported as ``running: true, stale: false``."""
    _write_state(_valid_state(), _paths.worker_state_path())

    coord = Coordinator(project_name="ws-fresh")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/worker-state")
            assert r.status_code == 200
            body = r.json()
            assert body["running"] is True
            assert body["stale"] is False
            state = body["state"]
            assert state["schema_version"] == 1
            assert state["gpu"]["name"] == "NVIDIA GeForce RTX 5080"
            assert state["projects_served"][0]["tasks_completed"] == 3
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_worker_state_marks_stale_when_old(nexus_grid_tmp: Path) -> None:
    """Snapshots older than 15 s are marked ``stale: true`` but
    still returned — the shell renders a warning banner."""
    old_ts = (datetime.now(UTC) - timedelta(seconds=120)).strftime("%Y-%m-%dT%H:%M:%SZ")
    _write_state(_valid_state(last_updated_at=old_ts), _paths.worker_state_path())

    coord = Coordinator(project_name="ws-stale")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/worker-state")
            body = r.json()
            assert body["running"] is True
            assert body["stale"] is True
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_worker_state_handles_invalid_json(nexus_grid_tmp: Path) -> None:
    """Corrupted ``state.json`` returns ``running: false`` with
    an ``error`` field for diagnostics."""
    path = _paths.worker_state_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("not valid json {{", encoding="utf-8")

    coord = Coordinator(project_name="ws-corrupt")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/worker-state")
            body = r.json()
            assert body["running"] is False
            assert "invalid JSON" in body.get("error", "")
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_worker_state_handles_schema_mismatch(nexus_grid_tmp: Path) -> None:
    """A schema_version != 1 payload is rejected cleanly."""
    _write_state(
        {"schema_version": 999, "node_id": "aa" * 32},
        _paths.worker_state_path(),
    )

    coord = Coordinator(project_name="ws-schema")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/worker-state")
            body = r.json()
            assert body["running"] is False
            assert "schema" in body.get("error", "").lower()
    finally:
        await coord.stop()
