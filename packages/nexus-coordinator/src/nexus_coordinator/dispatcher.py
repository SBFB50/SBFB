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
from nexus_coordinator.guardrails import (
    GuardrailChain,
    GuardrailContext,
    StageGuardrailMap,
)
from nexus_coordinator.hooks import HookRunner
from nexus_coordinator.pii_redactor import PiiRedactor
from nexus_coordinator.rerun import RerunSampler

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
    # Sprint 23 Phase D — redundancy voting.
    redundancy_factor: int = 1


class Dispatcher:
    """Owns ``task:*`` writes on the project doc and the task_state
    mirror SQLite table.

    Sprint 21 phase coord-side : ``pii_redactor`` hook layer 2
    defense-in-depth. Si fourni, ``Dispatcher.submit`` appelle
    ``pii_redactor.redact()`` sur ``req.prompt`` et
    ``req.system_prompt`` AVANT ``nexus_core.sign_task`` —
    garantit que le TaskEntry signé qui part sur iroh-docs ne
    contient jamais de PII brutes, même si la couche iframe
    client (phase B `d5b0035`) n'a pas tourné.
    """

    def __init__(
        self,
        *,
        db_path: Path,
        doc: Any,  # nexus_core.Doc
        author_id: str,
        coord_secret: bytes,
        pii_redactor: PiiRedactor | None = None,
        redundancy_dispatcher: Any | None = None,  # RedundancyDispatcher
        input_chain: GuardrailChain | None = None,
        stage_guards: StageGuardrailMap | None = None,
        hook_runner: HookRunner | None = None,
        rerun_sampler: RerunSampler | None = None,
    ) -> None:
        self._db_path = db_path
        self._doc = doc
        self._author_id = author_id
        self._coord_secret = coord_secret
        self._pii_redactor = pii_redactor
        self._redundancy_dispatcher = redundancy_dispatcher
        if stage_guards is not None:
            self._stage_guards: StageGuardrailMap = stage_guards
        elif input_chain is not None:
            self._stage_guards = {"on_task_dispatched": input_chain}
        else:
            self._stage_guards = {}
        self._hook_runner = hook_runner
        self._rerun_sampler = rerun_sampler

    async def init(self) -> None:
        """Ensure the DB schema exists. Call once at coordinator start."""
        await init_db(self._db_path)

    async def submit(self, req: SubmitRequest) -> str:
        """Sign and write a new task to the doc. Returns the assigned
        ``task_id``.

        Idempotent on ``task_id``: if a row already exists in
        ``task_state`` for the supplied id the method returns it
        without re-signing, re-writing the doc, or re-inserting the
        row. This is required by the Sprint 19 Phase D delayed
        upload queue which retries an emit that partially completed
        before a coordinator crash (design §6.3 + upload_queue.py
        docstring).
        """
        task_id = req.task_id or f"t-{uuid.uuid4().hex}"

        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute("SELECT 1 FROM task_state WHERE task_id = ?", (task_id,)) as cursor:
                existing = await cursor.fetchone()
        if existing is not None:
            _log.info(
                "task already submitted, skipping duplicate",
                task_id=task_id,
            )
            return task_id

        now = int(time.time())
        dispatch_chain = self._stage_guards.get("on_task_dispatched")
        if dispatch_chain is not None:
            ctx = GuardrailContext(
                task_id=task_id,
                system_prompt=req.system_prompt,
                user_prompt=req.prompt,
            )
            prompt_for_wire = await dispatch_chain.run(ctx, req.prompt)
            system_prompt_for_wire = await dispatch_chain.run(ctx, req.system_prompt)
        elif self._pii_redactor is not None:
            prompt_for_wire = self._pii_redactor.redact(req.prompt)
            system_prompt_for_wire = self._pii_redactor.redact(req.system_prompt)
        else:
            prompt_for_wire = req.prompt
            system_prompt_for_wire = req.system_prompt
        task_dict = {
            "version": TASK_FORMAT_VERSION,
            "task_id": task_id,
            "task_type": req.task_type,
            "prompt": prompt_for_wire,
            "system_prompt": system_prompt_for_wire,
            "model": req.model,
            "priority": req.priority,
            "created_at": now,
            "parent_task_id": req.parent_task_id,
            "metadata": req.metadata or {},
            "is_open_source": bool(req.is_open_source),
            "estimated_watts": int(req.estimated_watts),
            "estimated_vram_mb": int(req.estimated_vram_mb),
            "estimated_hours": float(req.estimated_hours),
            "redundancy_factor": int(req.redundancy_factor),
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
                INSERT OR IGNORE INTO task_state (
                    task_id, state, task_json, task_type, model,
                    priority, submitted_at
                ) VALUES (?, 'pending', ?, ?, ?, ?, ?)
                """,
                (task_id, signed, req.task_type, req.model, req.priority, now),
            )
            await db.commit()

        if req.redundancy_factor > 1 and self._redundancy_dispatcher is not None:
            self._redundancy_dispatcher.register_task(task_id, req.redundancy_factor)

        _log.info("task submitted", task_id=task_id, task_type=req.task_type, model=req.model)
        if self._hook_runner is not None:
            await self._hook_runner.fire(
                "on_task_dispatched",
                task_id=task_id,
                metadata={"task_type": req.task_type, "model": req.model},
            )
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
        """Called by the validator on a signature-valid result.

        When a ``RerunSampler`` is wired, completed non-rerun tasks
        are evaluated for spot-check re-dispatch. The sampler decides
        based on ``sample_rate``; if selected, a re-run task is
        submitted with the same parameters but a distinct task_id.
        """
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

        if self._rerun_sampler is not None and self._rerun_sampler.should_rerun(task_id):
            await self._schedule_rerun(task_id)

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

    async def _schedule_rerun(self, original_task_id: str) -> None:
        """Read the original task parameters and submit a re-run."""
        assert self._rerun_sampler is not None
        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(
                "SELECT task_json FROM task_state WHERE task_id = ?",
                (original_task_id,),
            ) as cursor:
                row = await cursor.fetchone()
        if row is None:
            return
        try:
            entry = json.loads(row[0])
        except (json.JSONDecodeError, TypeError):
            return
        task = entry.get("task", entry) if isinstance(entry, dict) else {}
        if not isinstance(task, dict):
            return

        rerun_id = self._rerun_sampler.make_rerun_id(original_task_id)
        req = SubmitRequest(
            task_type=task.get("task_type", "unknown"),
            prompt=task.get("prompt", ""),
            model=task.get("model", "unknown"),
            system_prompt=task.get("system_prompt", ""),
            priority=task.get("priority", 5),
            task_id=rerun_id,
            is_open_source=task.get("is_open_source", False),
            estimated_watts=task.get("estimated_watts", 0),
            estimated_vram_mb=task.get("estimated_vram_mb", 0),
            estimated_hours=task.get("estimated_hours", 0.0),
            redundancy_factor=1,
        )
        await self.submit(req)
        _log.info(
            "rerun_task_scheduled",
            original_task_id=original_task_id,
            rerun_task_id=rerun_id,
        )
