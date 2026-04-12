# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 5 Phase C — end-to-end worker state roundtrip.

Spawns a real ``nexus-worker start --stub-ollama`` subprocess in
a hermetic ``NEXUS_GRID_ROOT`` tree, waits for the Rust
state_writer to flush its first ``state.json``, then pokes the
coordinator's ``/worker-state`` proxy and verifies the payload
matches the Rust-side schema.

This is the proof that the Rust worker and the Python
coordinator proxy speak the exact same file format — decision
D3 option (c) only works if both sides agree on the JSON shape.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator import paths as _paths
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator

REPO_ROOT = Path(__file__).resolve().parents[3]


def _find_worker_binary() -> Path | None:
    """Locate the compiled ``nexus-worker`` binary.

    Prefers the debug build at ``target/debug/nexus-worker(.exe)``
    because that's what ``cargo build`` produces when the dev
    loop runs tests. A release build is picked up as a fallback.
    Returns ``None`` if neither is present so the test can skip
    rather than fail on a fresh clone.
    """
    suffix = ".exe" if sys.platform == "win32" else ""
    for profile in ("debug", "release"):
        candidate = REPO_ROOT / "target" / profile / f"nexus-worker{suffix}"
        if candidate.exists():
            return candidate
    return None


def _spawn_worker(binary: Path, grid_root: Path, worker_config_dir: Path) -> subprocess.Popen[bytes]:
    env = {
        **os.environ,
        "NEXUS_GRID_ROOT": str(grid_root),
        # Point the worker's own config at a scratch location so
        # we don't race against the developer's real worker.toml.
        # `--config <path>` drives `WorkerPaths::resolve` to
        # derive every other dir from this parent.
        # Rich prints a checkmark on register success which needs
        # utf-8 on a Windows cp1252 console.
        "PYTHONIOENCODING": "utf-8",
    }
    cfg = worker_config_dir / "worker.toml"
    cfg.parent.mkdir(parents=True, exist_ok=True)

    # Register first so the worker has a config + secret key
    # before we try `start`.
    subprocess.run(
        [
            str(binary),
            "--config",
            str(cfg),
            "register",
            "--name",
            "pytest-worker",
        ],
        env=env,
        check=True,
        capture_output=True,
    )

    return subprocess.Popen(
        [
            str(binary),
            "--config",
            str(cfg),
            "start",
            "--stub-ollama",
        ],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.PIPE,
    )


def _wait_for_state_file(path: Path, timeout_s: float = 20.0) -> dict:
    deadline = time.monotonic() + timeout_s
    while time.monotonic() < deadline:
        if path.exists():
            try:
                body = json.loads(path.read_text(encoding="utf-8"))
                # The first flush may coincide with a partial
                # write if we peek at the wrong moment; require
                # the version field to be populated to be sure
                # we have a full JSON object.
                if body.get("schema_version") == 1:
                    return body
            except (json.JSONDecodeError, OSError):
                pass
        time.sleep(0.3)
    raise TimeoutError(f"state.json never materialised at {path} in {timeout_s}s")


@pytest.mark.asyncio
async def test_worker_state_roundtrip_with_stub_ollama(
    nexus_grid_tmp: Path,
) -> None:
    """Spawn worker → wait for state.json → proxy via coordinator → assert.

    The assertions check the fields the shell actually reads,
    not just that the file exists. A regression in the Rust
    `WorkerStateSnapshot` serde shape or the Python `WorkerStateV1`
    validator would fail here immediately.
    """
    binary = _find_worker_binary()
    if binary is None:
        pytest.skip("nexus-worker binary not built; run `cargo build -p nexus-worker` first")

    # Path on disk where the Rust state_writer will drop its
    # snapshot — mirrors `nexus_coordinator.paths.worker_state_path`
    # which the conftest fixture already monkey-patched to
    # `<tmp>/nexus-grid/worker/state.json`.
    state_file = _paths.worker_state_path()

    # The Python conftest monkey-patches `paths.*` functions but
    # not the Rust-side env var. Set NEXUS_GRID_ROOT explicitly
    # so the Rust worker writes to the same tmp tree the Python
    # coordinator reads from.
    grid_root = nexus_grid_tmp

    worker_cfg_dir = nexus_grid_tmp / "worker-cfg"
    proc = _spawn_worker(binary, grid_root, worker_cfg_dir)

    try:
        body = _wait_for_state_file(state_file, timeout_s=25.0)
        assert body["schema_version"] == 1
        assert isinstance(body["node_id"], str)
        assert len(body["node_id"]) == 64
        assert isinstance(body["worker_version"], str)
        assert isinstance(body["uptime_secs"], int)
        assert "gpu" in body  # key present, value may be null
        assert isinstance(body["projects_served"], list)
        assert "last_task" in body

        # Now proxy the file through a live coordinator's
        # /worker-state endpoint — this is the full shell path.
        coord = Coordinator(project_name="ws-roundtrip")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.get("/worker-state")
                assert r.status_code == 200
                payload = r.json()
                assert payload["running"] is True
                assert payload["stale"] is False, "fresh snapshot must not be stale"
                state = payload["state"]
                assert state["schema_version"] == 1
                assert state["node_id"] == body["node_id"]
                assert state["worker_version"] == body["worker_version"]
        finally:
            await coord.stop()

    finally:
        # Kill the worker process tree; /F on Windows, SIGTERM
        # + SIGKILL on POSIX.
        if proc.poll() is None:
            if sys.platform == "win32":
                subprocess.run(
                    ["taskkill", "/PID", str(proc.pid), "/T", "/F"],
                    capture_output=True,
                )
            else:
                proc.terminate()
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    proc.kill()
        # Drain stderr for diagnostic purposes on CI, ignore here
        try:
            proc.stderr.read() if proc.stderr else None
        except Exception:  # noqa: BLE001
            pass
        # Clean up the worker config scratch dir
        shutil.rmtree(worker_cfg_dir, ignore_errors=True)
