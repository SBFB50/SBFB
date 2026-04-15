# SPDX-License-Identifier: AGPL-3.0-or-later
"""Task dispatcher.

The dispatcher owns the write side of the project doc: it signs
TaskEntry payloads with the coordinator keypair, writes them under
``task:<task_id>`` keys, and mirrors the state into a local SQLite
table (``task_state``) so the FastAPI control plane can serve
``GET /tasks`` without re-scanning the doc on every request.

Scope (Phase B):

- :meth:`Dispatcher.submit` — validates a task request, assigns an
  id if the caller did not, signs via :func:`nexus_core.sign_task`,
  writes to the doc, inserts a ``task_state`` row.
- :meth:`Dispatcher.retry_timed_out` — scans ``task_state`` for
  tasks stuck in ``claimed`` past ``policy.claim_timeout_secs`` and
  flips them back to ``pending`` so the next worker pickup
  re-dispatches them (the doc key is unchanged; the validator
  correlates on ``task_id``).
- :meth:`Dispatcher.list_tasks` — plain SELECT for the API.

This module never fetches results; that's the validator's job.
"""

from __future__ import annotations

import json
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import aiosqlite
import nexus_core
import structlog

from nexus_coordinator.db.migrations import init_db

_log = structlog.get_logger(__name__)


TASK_FORMAT_VERSION = 1


@dataclass
class SubmitRequest:
    """Shape of the body accepted by :meth:`Dispatcher.submit`.

    Kept as a plain dataclass (not a pydantic model) so the
    dispatcher stays decoupled from the FastAPI request layer;
    the API routers convert their pydantic input into one of
    these.

    Sprint 18 Phase D adds four resource-hint fields that the
    :class:`Task` canonical format carries end-to-end to the
    worker's consent layer (``is_open_source``,
    ``estimated_watts``, ``estimated_vram_mb``,
    ``estimated_hours``). These are *never* accepted from the
    HTTP client — the ``/tasks/submit`` handler derives them
    server-side from project config (``repo_url`` presence) and
    from the submitting app's :meth:`NexusApp.cost_estimate`.
    The dispatcher trusts its caller to have performed that
    derivation; its only job is to write the values into the
    signed TaskEntry.
    """

    task_type: str
    prompt: str
    model: str
    system_prompt: str = ""
    priority: int = 5
    parent_task_id: str = ""
    metadata: dict[str, str] | None = None
    task_id: str | None = None
    # Sprint 18 Phase D — task-entry wire-through.
    is_open_source: bool = False
    estimated_watts: int = 0
    estimated_vram_mb: int = 0
    estimated_hours: float = 0.0


class Dispatcher:
    """Owns ``task:*`` writes on the project doc and the task_state
    mirror SQLite table."""

    def __init__(
        self,
        *,
        db_path: Path,
        doc: Any,  # nexus_core.Doc
        author_id: str,
        coord_secret: bytes,
    ) -> None:
        self._db_path = db_path
        self._doc = doc
        self._author_id = author_id
        self._coord_secret = coord_secret

    async def init(self) -> None:
        """Ensure the DB schema exists. Call once at coordinator start."""
        await init_db(self._db_path)

    async def submit(self, req: SubmitRequest) -> str:
        """Sign and write a new task to the doc. Returns the assigned
        ``task_id``."""
        task_id = req.task_id or f"t-{uuid.uuid4().hex}"
        now = int(time.time())
        task_dict = {
            "version": TASK_FORMAT_VERSION,
            "task_id": task_id,
            "task_type": req.task_type,
            "prompt": req.prompt,
            "system_prompt": req.system_prompt,
            "model": req.model,
            "priority": req.priority,
            "created_at": now,
            "parent_task_id": req.parent_task_id,
            "metadata": req.metadata or {},
            "is_open_source": bool(req.is_open_source),
            "estimated_watts": int(req.estimated_watts),
            "estimated_vram_mb": int(req.estimated_vram_mb),
            "estimated_hours": float(req.estimated_hours),
        }
        task_json = json.dumps(task_dict, sort_keys=True)
        signed = nexus_core.sign_task(task_json, self._coord_secret)
        # signed is a JSON-string TaskEntry ({ task, author_pubkey,
        # signature }). Write it to the doc under the task: prefix.
        key = f"task:{task_id}".encode("utf-8")
        value = signed.encode("utf-8")
        await self._doc.set(self._author_id, key, value)

        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                INSERT INTO task_state (
                    task_id, state, task_json, task_type, model,
                    priority, submitted_at
                ) VALUES (?, 'pending', ?, ?, ?, ?, ?)
                """,
                (task_id, signed, req.task_type, req.model, req.priority, now),
            )
            await db.commit()

        _log.info("task submitted", task_id=task_id, task_type=req.task_type, model=req.model)
        return task_id

    async def mark_claimed(self, task_id: str, worker_pubkey: bytes) -> None:
        """Called by the validator when a ``claim:<task_id>`` entry
        appears on the doc. Idempotent."""
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                UPDATE task_state
                SET state = 'claimed',
                    claimed_by_pubkey = ?,
                    claimed_at = ?
                WHERE task_id = ? AND state IN ('pending', 'timed_out')
                """,
                (worker_pubkey, int(time.time()), task_id),
            )
            await db.commit()

    async def mark_completed(self, task_id: str, result_hash: bytes) -> None:
        """Called by the validator on a signature-valid result."""
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                UPDATE task_state
                SET state = 'completed',
                    completed_at = ?,
                    result_hash = ?
                WHERE task_id = ?
                """,
                (int(time.time()), result_hash, task_id),
            )
            await db.commit()

    async def mark_failed(self, task_id: str, reason: str) -> None:
        """Called when a result fails verification."""
        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                UPDATE task_state
                SET state = 'failed',
                    last_error = ?,
                    completed_at = ?
                WHERE task_id = ?
                """,
                (reason, int(time.time()), task_id),
            )
            await db.commit()

    async def retry_timed_out(self, claim_timeout_secs: int) -> int:
        """Flip any ``claimed`` task older than ``claim_timeout_secs``
        back to ``pending``. Returns the number of rows touched."""
        cutoff = int(time.time()) - claim_timeout_secs
        async with aiosqlite.connect(self._db_path) as db:
            cursor = await db.execute(
                """
                UPDATE task_state
                SET state = 'timed_out'
                WHERE state = 'claimed' AND claimed_at IS NOT NULL AND claimed_at < ?
                """,
                (cutoff,),
            )
            rowcount = cursor.rowcount
            await cursor.close()
            await db.commit()
        if rowcount:
            _log.info("tasks timed out, requeued", count=rowcount)
        return rowcount

    async def list_tasks(self, state: str | None = None, limit: int = 100) -> list[dict[str, Any]]:
        """Return recent tasks, optionally filtered by state."""
        query = "SELECT task_id, state, task_type, model, priority, submitted_at, claimed_at, completed_at, last_error FROM task_state"
        params: tuple[Any, ...] = ()
        if state is not None:
            query += " WHERE state = ?"
            params = (state,)
        query += " ORDER BY submitted_at DESC LIMIT ?"
        params = (*params, limit)

        async with aiosqlite.connect(self._db_path) as db:
            db.row_factory = aiosqlite.Row
            async with db.execute(query, params) as cursor:
                rows = await cursor.fetchall()
                return [dict(row) for row in rows]
