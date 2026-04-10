"""Sprint 5 Phase A — ``/worker-state`` proxy endpoint.

Decision D3 option (c): the Rust worker flushes a JSON snapshot
to ``~/.nexus-grid/worker/state.json`` every few seconds. The
coordinator reads that file here and hands it to the shell, so
the shell never touches the filesystem directly.

The schema mirrors the Rust ``WorkerStateSnapshot`` produced by
``nexus_worker_core::engine::state_writer``. See
``.planning/sprint5_plan.md`` §2.3 for the frozen v1 shape.

A missing, malformed, or stale file is a non-error: the shell
renders an explanatory card rather than a crash. Staleness is
defined as ``now - last_updated_at > 15s``.
"""

from __future__ import annotations

import json
from datetime import UTC, datetime, timedelta
from typing import Any, Literal

import structlog
from fastapi import APIRouter
from pydantic import BaseModel, Field, ValidationError

from nexus_coordinator import paths as _paths

_log = structlog.get_logger(__name__)

router = APIRouter(tags=["worker"])

# Anything older than this is reported as ``stale: true``.
_STALE_THRESHOLD = timedelta(seconds=15)


class GpuSnapshotV1(BaseModel):
    name: str
    memory_total_mb: int = Field(..., ge=0)
    memory_used_mb: int = Field(..., ge=0)
    utilization_pct: int = Field(..., ge=0, le=100)
    temperature_c: int = Field(..., ge=0)
    power_draw_w: float = Field(..., ge=0)


class ProjectServedV1(BaseModel):
    project_name: str
    doc_id: str
    kudos_total: int = Field(..., ge=0)
    tasks_completed: int = Field(..., ge=0)


class LastTaskV1(BaseModel):
    task_id: str
    project_name: str
    prompt_preview: str
    status: str
    completed_at: str


class WorkerStateV1(BaseModel):
    schema_version: Literal[1]
    node_id: str
    worker_version: str
    uptime_secs: int = Field(..., ge=0)
    started_at: str
    last_updated_at: str
    gpu: GpuSnapshotV1 | None
    projects_served: list[ProjectServedV1]
    last_task: LastTaskV1 | None


@router.get("/worker-state")
async def worker_state() -> dict[str, Any]:
    """Proxy the local worker's ``state.json`` file to the shell.

    Returns one of:

    - ``{"running": false}`` — no file present (no worker is
      running, or the ``nexus-worker start`` process has not yet
      emitted its first snapshot).
    - ``{"running": true, "stale": false, "state": {...}}`` —
      fresh snapshot, shell renders live cards.
    - ``{"running": true, "stale": true, "state": {...}}`` —
      last update was > 15 s ago, shell renders a warning banner.
    - ``{"running": false, "error": "invalid JSON"}`` — the file
      exists but could not be parsed. The shell treats this like
      an offline worker and logs the reason for diagnostics.
    """
    path = _paths.worker_state_path()
    if not path.exists():
        return {"running": False}

    try:
        body = path.read_text(encoding="utf-8")
    except OSError as e:
        _log.warning("cannot read worker state.json", path=str(path), error=str(e))
        return {"running": False, "error": f"read error: {e}"}

    try:
        raw = json.loads(body)
    except json.JSONDecodeError as e:
        _log.warning("worker state.json is not valid JSON", path=str(path), error=str(e))
        return {"running": False, "error": "invalid JSON"}

    try:
        state = WorkerStateV1.model_validate(raw)
    except ValidationError as e:
        _log.warning("worker state.json schema mismatch", path=str(path), error=str(e))
        return {"running": False, "error": "schema mismatch"}

    stale = _is_stale(state.last_updated_at)
    return {
        "running": True,
        "stale": stale,
        "state": state.model_dump(),
    }


def _is_stale(iso_timestamp: str) -> bool:
    """Return ``True`` if the timestamp is older than the staleness threshold.

    Unparseable timestamps are treated as stale so a broken
    worker that stopped flushing on day 1 doesn't show green in
    the shell forever.
    """
    try:
        # Rust's `time` crate emits RFC 3339 which Python's
        # ``datetime.fromisoformat`` parses natively since 3.11.
        ts = datetime.fromisoformat(iso_timestamp)
    except ValueError:
        return True
    if ts.tzinfo is None:
        ts = ts.replace(tzinfo=UTC)
    age = datetime.now(UTC) - ts
    return age > _STALE_THRESHOLD
