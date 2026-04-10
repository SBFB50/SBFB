"""Sprint 5 Phase A — local coordinator registry.

Decision D1 of ``.planning/sprint5_plan.md`` §2.1: every
``nexus-coordinator start <name>`` process writes a
``running.json`` file inside its project directory at boot and
removes it at shutdown. The shell does not read those files
directly — it interrogates ``GET /shell/discover`` on any
coordinator it already knows about, and that endpoint globs
``<projects_root>/*/running.json`` to return the full list.

Schema is frozen at ``schema_version = 1``. Any breaking change
requires a bump of both this module and the
``ShellDiscoverResponseSchema`` on the TypeScript side.

Atomic write: the writer emits ``running.json.tmp`` first then
renames it over the destination. Stale files left by a crashed
coordinator are tolerated — the shell's health-check roundtrip
sees the coordinator as unreachable and marks it offline.
"""

from __future__ import annotations

import json
import os
from datetime import UTC, datetime
from pathlib import Path
from typing import TYPE_CHECKING, Literal

import structlog
from pydantic import BaseModel, Field, ValidationError

from nexus_coordinator import paths as _paths

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

SCHEMA_VERSION: Literal[1] = 1


class RunningState(BaseModel):
    """The ``running.json`` payload. See plan §2.1."""

    schema_version: Literal[1] = SCHEMA_VERSION
    project_name: str = Field(..., min_length=1, max_length=64)
    node_id: str = Field(..., description="Ed25519 pubkey hex, 64 chars lowercase.")
    doc_id: str = Field(
        ...,
        description="iroh-docs namespace id hex (may be empty during boot).",
    )
    api_host: str
    api_port: int = Field(..., ge=1, le=65535)
    pid: int = Field(..., ge=1)
    started_at: str = Field(
        ...,
        description="ISO-8601 UTC with microseconds, e.g. 2026-04-10T14:23:00.123456+00:00.",
    )
    visibility: Literal["public", "private"]


def write_running_state(coord: "Coordinator") -> Path:
    """Atomically write ``running.json`` for a booted coordinator.

    Called by the CLI ``start`` command after ``coord.start()``
    has returned and uvicorn is about to take over the event
    loop. Every field is derived from the live coordinator state
    — no caller-supplied overrides so there is no way for the
    file to misreport the running process.
    """
    if coord.state.node_id is None:
        raise RuntimeError("cannot write running.json before Coordinator.start() populates node_id")

    entry = RunningState(
        project_name=coord.project_name,
        node_id=coord.state.node_id,
        doc_id=coord.state.doc_id or "",
        api_host=coord.config.network.api_host,
        api_port=coord.config.network.api_port,
        pid=os.getpid(),
        started_at=datetime.now(UTC).isoformat(),
        visibility=coord.config.network.visibility,  # type: ignore[arg-type]
    )

    dest = _paths.running_state_path(coord.project_name)
    dest.parent.mkdir(parents=True, exist_ok=True)

    # Atomic write: sibling .tmp file then rename.
    tmp = dest.with_suffix(".json.tmp")
    body = entry.model_dump_json(indent=2)
    tmp.write_text(body, encoding="utf-8")
    os.replace(tmp, dest)

    _log.info(
        "running.json written",
        project=coord.project_name,
        path=str(dest),
        pid=entry.pid,
    )
    return dest


def remove_running_state(project_name: str) -> None:
    """Best-effort removal of ``running.json`` during shutdown.

    The CLI ``start`` command calls this in the ``finally:``
    block so an OS-killed coordinator leaves the file behind but
    a clean exit removes it. Stale files are tolerated by the
    shell via the health-check roundtrip.
    """
    path = _paths.running_state_path(project_name)
    try:
        path.unlink(missing_ok=True)
        _log.info("running.json removed", project=project_name, path=str(path))
    except OSError as e:
        # A permissions error or a concurrent removal is
        # non-fatal — log and move on; the shell detects stale
        # entries via the health-check.
        _log.warning(
            "failed to remove running.json",
            project=project_name,
            path=str(path),
            error=str(e),
        )


def discover_running() -> list[RunningState]:
    """Scan the projects root for every ``running.json`` file.

    Malformed or unreadable entries are logged and skipped so a
    single broken project cannot poison the whole discover
    response. Callers get a best-effort snapshot, not a
    transaction.
    """
    projects = _paths.projects_root()
    if not projects.exists():
        return []

    entries: list[RunningState] = []
    for candidate in sorted(projects.glob("*/running.json")):
        try:
            body = candidate.read_text(encoding="utf-8")
        except OSError as e:
            _log.warning("failed to read running.json", path=str(candidate), error=str(e))
            continue

        try:
            raw = json.loads(body)
        except json.JSONDecodeError as e:
            _log.warning("running.json is not valid JSON", path=str(candidate), error=str(e))
            continue

        try:
            entry = RunningState.model_validate(raw)
        except ValidationError as e:
            _log.warning(
                "running.json schema mismatch; ignoring",
                path=str(candidate),
                error=str(e),
            )
            continue

        entries.append(entry)

    return entries
