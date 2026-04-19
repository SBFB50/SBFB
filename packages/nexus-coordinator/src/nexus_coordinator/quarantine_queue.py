# SPDX-License-Identifier: AGPL-3.0-or-later
"""Quarantine queue — Sprint 21 Phase D defense-in-depth DoS triangle.

Closes the third leg of the C-DosFlood S21 defence (after rate-
limit Phase A and PoW gossip subscribe S20 Phase C wire). When a
gossip message has passed PoW + rate-limit but matches a soft
borderline heuristic (cumulated rate-strikes from a single sender,
suspicious payload pattern, etc.), the subscriber routes it
through :class:`QuarantineQueue` instead of handing it directly to
the runtime. The operator inspects the queue via
``nexus-coordinator quarantine list`` and either ``flush`` (mark
accepted for audit trail) or ``drop`` (mark rejected) each entry.
A 15-minute TTL auto-drops pending entries silently to bound disk
growth.

Design doc: ``.planning/research/S21_phase_D_quarantine_design.md``.
G8 preflight: ``.planning/active/sprint21_phase_D_preflight.md``
verdict SCOPE-CUT-CONSISTENT. Pattern source: Sprint 19 Phase D
``upload_queue.py`` (aiosqlite WAL + asyncio sweep loop).

Hors-scope Phase D (carry S22+):

- Wire-up automatique depuis le subscriber gossip ; Phase D livre
  uniquement la primitive + REST + CLI. Le wire-up a besoin du
  Sybil/kudos contexte S22+.
- Re-injection automatique dans le gossip layer sur ``flush``.
- Migration runner schema evolution post-v1.0.
"""

from __future__ import annotations

import asyncio
import time
from pathlib import Path
from typing import Any, Callable

import aiosqlite
import structlog

_log = structlog.get_logger(__name__)


#: Allowed values for the ``pow_status`` column. Persisted as audit
#: metadata only — the queue does not re-verify PoW at flush
#: (design §5.2).
_VALID_POW_STATUS: frozenset[str] = frozenset({"valid", "missing", "invalid"})

#: Allowed values for the ``flush_status`` column.
_VALID_FLUSH_STATUS: frozenset[str] = frozenset({"pending", "flushed", "dropped"})


class QuarantineQueue:
    """Async SQLite-backed quarantine queue for borderline gossip
    messages.

    The queue is single-writer (one coordinator owns one DB file)
    and uses a single asyncio event loop. The internal ``_lock``
    serialises ``add`` against the TTL sweep loop's ``DELETE`` so a
    concurrent add does not race with a sweep that just decided
    "no rows expired" before the new row landed.

    Test injection points:

    - ``now_fn``: callable returning a unix-epoch int. Production
      passes ``lambda: int(time.time())``; tests pass a mutable
      lambda to mock wall-clock advancement without depending on
      freezegun (pattern miroir S19 D upload_queue.py).
    - ``ttl_seconds``: TTL window. Production = 900 (15 min,
      kickoff §D4 ligne 590). Tests can shrink to verify sweep
      semantics without long sleeps.
    - ``sweep_interval_s``: interval between TTL sweeps. Production
      = 30s. Tests can shrink to keep the suite fast.
    """

    def __init__(
        self,
        *,
        db_path: Path,
        ttl_seconds: int = 900,
        sweep_interval_s: float = 30.0,
        now_fn: Callable[[], int] | None = None,
    ) -> None:
        if ttl_seconds <= 0:
            raise ValueError("ttl_seconds must be > 0")
        if sweep_interval_s <= 0.0:
            raise ValueError("sweep_interval_s must be > 0")

        self.db_path = db_path
        self.ttl_seconds = int(ttl_seconds)
        self.sweep_interval = float(sweep_interval_s)
        self._now: Callable[[], int] = now_fn if now_fn is not None else (lambda: int(time.time()))

        self._lock = asyncio.Lock()
        self._sweep_task: asyncio.Task[None] | None = None
        self._stopping = asyncio.Event()

    # ------------------------------------------------------------------
    # Schema lifecycle
    # ------------------------------------------------------------------

    async def init(self) -> None:
        """Create the ``quarantine_messages`` table if missing.

        WAL mode is enabled at connection time so the queue stays
        durable across coordinator crashes (design §2.1) and a
        concurrent reader (e.g. a debugging ``sqlite3`` shell) can
        read while the sweep loop writes.
        """
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        async with aiosqlite.connect(self.db_path) as db:
            await db.execute("PRAGMA journal_mode = WAL")
            await db.execute("PRAGMA synchronous = NORMAL")
            await db.execute(
                """
                CREATE TABLE IF NOT EXISTS quarantine_messages (
                    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
                    topic               TEXT NOT NULL,
                    sender_pubkey       BLOB NOT NULL,
                    payload_bytes       BLOB NOT NULL,
                    received_at_epoch_s INTEGER NOT NULL,
                    rate_strikes        INTEGER NOT NULL,
                    pow_status          TEXT NOT NULL,
                    flush_status        TEXT NOT NULL DEFAULT 'pending'
                )
                """
            )
            await db.execute(
                "CREATE INDEX IF NOT EXISTS idx_quarantine_received ON quarantine_messages(received_at_epoch_s)"
            )
            await db.execute("CREATE INDEX IF NOT EXISTS idx_quarantine_sender ON quarantine_messages(sender_pubkey)")
            await db.commit()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def add(
        self,
        *,
        topic: str,
        sender_pubkey: bytes,
        payload_bytes: bytes,
        rate_strikes: int,
        pow_status: str,
    ) -> int:
        """Insert a borderline message and return its row id.

        ``pow_status`` is persisted as audit metadata; the queue
        never re-verifies the proof. Caller is responsible for
        having validated PoW upstream (design §5.2).
        """
        if pow_status not in _VALID_POW_STATUS:
            raise ValueError(f"pow_status must be one of {sorted(_VALID_POW_STATUS)}, got {pow_status!r}")
        if rate_strikes < 0:
            raise ValueError("rate_strikes must be >= 0")

        received_at = self._now()
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                cursor = await db.execute(
                    "INSERT INTO quarantine_messages "
                    "(topic, sender_pubkey, payload_bytes, received_at_epoch_s, "
                    " rate_strikes, pow_status, flush_status) "
                    "VALUES (?, ?, ?, ?, ?, ?, 'pending')",
                    (topic, sender_pubkey, payload_bytes, received_at, rate_strikes, pow_status),
                )
                row_id = cursor.lastrowid or 0
                await db.commit()
        _log.info(
            "quarantine entry added",
            row_id=row_id,
            topic=topic,
            rate_strikes=rate_strikes,
            pow_status=pow_status,
        )
        return row_id

    async def list(self, *, status: str = "pending") -> list[dict[str, Any]]:
        """Return entries filtered by ``flush_status``.

        ``status='all'`` returns every row regardless of status.
        Other values must be one of ``pending`` / ``flushed`` /
        ``dropped``. Results are ordered by ``received_at_epoch_s``
        ascending (oldest first) so the operator sees urgent
        decisions first.
        """
        if status != "all" and status not in _VALID_FLUSH_STATUS:
            raise ValueError(f"status must be 'all' or one of {sorted(_VALID_FLUSH_STATUS)}, got {status!r}")
        async with aiosqlite.connect(self.db_path) as db:
            db.row_factory = aiosqlite.Row
            if status == "all":
                cursor = await db.execute(
                    "SELECT id, topic, sender_pubkey, payload_bytes, "
                    "received_at_epoch_s, rate_strikes, pow_status, flush_status "
                    "FROM quarantine_messages ORDER BY received_at_epoch_s ASC"
                )
            else:
                cursor = await db.execute(
                    "SELECT id, topic, sender_pubkey, payload_bytes, "
                    "received_at_epoch_s, rate_strikes, pow_status, flush_status "
                    "FROM quarantine_messages WHERE flush_status = ? "
                    "ORDER BY received_at_epoch_s ASC",
                    (status,),
                )
            rows = await cursor.fetchall()
        return [dict(row) for row in rows]

    async def flush(self, row_id: int) -> bool:
        """Mark a pending entry as ``flushed`` (operator accept).

        Returns ``True`` if a row was updated, ``False`` if the row
        id was missing or already non-pending. Re-injection into
        the gossip layer is hors-scope Phase D — see design §7.3.
        """
        return await self._set_status(row_id, "flushed")

    async def drop(self, row_id: int) -> bool:
        """Mark a pending entry as ``dropped`` (operator reject).

        Returns ``True`` if a row was updated. The row stays in the
        table for audit trail; only the auto-TTL sweep removes
        ``pending`` rows (kickoff §D4 ligne 591-592 silent drop).
        """
        return await self._set_status(row_id, "dropped")

    async def start(self) -> None:
        """Initialise the schema and spawn the TTL sweep loop.

        Idempotent — calling twice raises ``RuntimeError`` to avoid
        leaking a background task (pattern miroir
        ``UploadQueue.start`` ligne 226-228).
        """
        if self._sweep_task is not None:
            raise RuntimeError("quarantine queue already started")
        await self.init()
        self._stopping.clear()
        self._sweep_task = asyncio.create_task(self._sweep_loop())

    async def shutdown(self) -> None:
        """Stop the sweep loop. The DB file is left intact so audit
        entries (``flushed`` / ``dropped``) survive the restart.
        """
        self._stopping.set()
        if self._sweep_task is not None:
            try:
                await self._sweep_task
            except asyncio.CancelledError:
                pass
            self._sweep_task = None

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    async def _set_status(self, row_id: int, new_status: str) -> bool:
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                cursor = await db.execute(
                    "UPDATE quarantine_messages SET flush_status = ? WHERE id = ? AND flush_status = 'pending'",
                    (new_status, row_id),
                )
                updated = (cursor.rowcount or 0) > 0
                await db.commit()
        if updated:
            _log.info("quarantine entry status updated", row_id=row_id, new_status=new_status)
        return updated

    async def _sweep_loop(self) -> None:
        """Wake every ``sweep_interval`` seconds and auto-drop
        pending entries whose TTL has expired. Catches all
        exceptions per tick so the loop never dies on a transient
        SQLite error (pattern miroir ``UploadQueue._flush_loop``).
        """
        while not self._stopping.is_set():
            try:
                await asyncio.wait_for(self._stopping.wait(), timeout=self.sweep_interval)
                break  # stop signalled
            except asyncio.TimeoutError:
                try:
                    await self._auto_drop_expired()
                except Exception as exc:  # noqa: BLE001 — never let the loop die
                    _log.error("quarantine sweep tick failed", error=str(exc))

    async def _auto_drop_expired(self) -> int:
        """DELETE pending entries whose ``received_at_epoch_s <
        now - ttl_seconds``. Returns count deleted. Drop silently
        at log info level (kickoff §D4 ligne 591-592)."""
        cutoff = self._now() - self.ttl_seconds
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                cursor = await db.execute(
                    "DELETE FROM quarantine_messages WHERE received_at_epoch_s < ? AND flush_status = 'pending'",
                    (cutoff,),
                )
                deleted = cursor.rowcount or 0
                await db.commit()
        if deleted > 0:
            _log.info("quarantine TTL sweep dropped expired entries", count=deleted)
        return deleted


__all__ = ["QuarantineQueue"]
