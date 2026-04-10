"""Sprint 5 Phase A — running.json registry tests.

Verifies the D1 contract:

- ``write_running_state`` produces a well-formed file that
  parses back into :class:`RunningState`.
- ``remove_running_state`` is idempotent and tolerates a
  missing file.
- ``discover_running`` skips malformed entries rather than
  raising, so a single broken project cannot poison the shell's
  ``/shell/discover`` response.

The heavier "start writes, stop removes" assertion is covered by
``test_shell_discover.py`` via a live :class:`Coordinator` with
a FastAPI ``TestClient`` wrapped around it.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from nexus_coordinator.coordinator import Coordinator
from nexus_coordinator.paths import running_state_path
from nexus_coordinator.registry import (
    SCHEMA_VERSION,
    RunningState,
    discover_running,
    remove_running_state,
    write_running_state,
)


@pytest.mark.asyncio
async def test_running_json_written_on_start(nexus_grid_tmp: Path) -> None:
    """``write_running_state`` produces a schema-v1 file under the
    project directory with the coordinator's live state."""
    coord = Coordinator(project_name="demo-reg")
    await coord.start()
    try:
        path = write_running_state(coord)
        assert path == running_state_path("demo-reg")
        assert path.exists(), "running.json must exist after write"

        body = json.loads(path.read_text(encoding="utf-8"))
        assert body["schema_version"] == SCHEMA_VERSION
        assert body["project_name"] == "demo-reg"
        assert body["node_id"] == coord.state.node_id
        assert body["doc_id"] == coord.state.doc_id
        assert body["api_host"] == coord.config.network.api_host
        assert body["api_port"] == coord.config.network.api_port
        assert body["pid"] > 0
        assert body["visibility"] in {"public", "private"}
        assert "started_at" in body

        # Pydantic must accept the written body verbatim.
        parsed = RunningState.model_validate(body)
        assert parsed.project_name == "demo-reg"
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_running_json_removed_on_clean_stop(nexus_grid_tmp: Path) -> None:
    """``remove_running_state`` deletes the file and is idempotent."""
    coord = Coordinator(project_name="demo-stop")
    await coord.start()
    try:
        write_running_state(coord)
        path = running_state_path("demo-stop")
        assert path.exists()

        remove_running_state("demo-stop")
        assert not path.exists(), "running.json must be gone after remove"

        # Idempotent: a second remove must not raise.
        remove_running_state("demo-stop")
        assert not path.exists()
    finally:
        await coord.stop()


def test_write_running_state_rejects_unbooted_coordinator(nexus_grid_tmp: Path) -> None:
    """Calling ``write_running_state`` before ``start`` is a
    programming error — raise so the CLI surfaces it instead of
    silently writing a bogus file with a None node_id."""
    coord = Coordinator(project_name="demo-unbooted")
    with pytest.raises(RuntimeError, match="node_id"):
        write_running_state(coord)


@pytest.mark.asyncio
async def test_write_running_state_is_atomic_under_repeated_calls(
    nexus_grid_tmp: Path,
) -> None:
    """Successive writes must leave the file fully valid at every
    observation point — there is no window where a reader sees a
    half-written body."""
    coord = Coordinator(project_name="demo-atomic")
    await coord.start()
    try:
        for _ in range(5):
            write_running_state(coord)
            body = json.loads(running_state_path("demo-atomic").read_text(encoding="utf-8"))
            RunningState.model_validate(body)  # raises on partial write
    finally:
        await coord.stop()


def test_discover_running_on_empty_root_returns_nothing(nexus_grid_tmp: Path) -> None:
    """An empty projects root must yield an empty list, not an error."""
    assert discover_running() == []


def test_discover_running_skips_malformed_entries(nexus_grid_tmp: Path) -> None:
    """A garbage ``running.json`` must be logged and skipped; the
    surrounding valid entries are still returned."""
    projects = nexus_grid_tmp / "projects"
    (projects / "broken").mkdir(parents=True)
    (projects / "broken" / "running.json").write_text("not valid json {{", encoding="utf-8")

    (projects / "bad-schema").mkdir(parents=True)
    (projects / "bad-schema" / "running.json").write_text(
        json.dumps({"schema_version": 999, "project_name": "x"}),
        encoding="utf-8",
    )

    (projects / "ok").mkdir(parents=True)
    (projects / "ok" / "running.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "project_name": "ok",
                "node_id": "aa" * 32,
                "doc_id": "bb" * 32,
                "api_host": "127.0.0.1",
                "api_port": 18765,
                "pid": 42,
                "started_at": "2026-04-10T14:00:00+00:00",
                "visibility": "private",
            }
        ),
        encoding="utf-8",
    )

    found = discover_running()
    assert len(found) == 1
    assert found[0].project_name == "ok"
