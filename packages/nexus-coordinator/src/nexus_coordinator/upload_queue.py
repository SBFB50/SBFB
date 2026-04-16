# SPDX-License-Identifier: AGPL-3.0-or-later
"""Delayed upload queue — Sprint 19 Phase D anti-correlation.

Every ``/tasks/submit`` posts through :class:`UploadQueue` instead
of calling the dispatcher directly. :meth:`UploadQueue.schedule`
draws a cryptographically-random delay (exponential mean=90s,
clamped to ``max_jitter_s - flush_interval_s`` internally so the
observable p99 stays under ``max_jitter_s``), persists the payload
in a SQLite WAL table (``delayed_uploads``), and returns the
resolved ``task_id`` immediately. A background scheduler loop
wakes every ``flush_interval_s`` seconds, selects rows whose
``deliver_at`` is in the past, calls the injected emit function
(in production: :meth:`Dispatcher.submit`), and deletes the row on
success.

Design doc: ``.planning/research/S19_phase_D_delayed_upload_queue_design.md``.
Threat model: ``docs/security/P2P_THREATS.md §6.3`` dragnet
metadata correlation. The queue breaks the short-window timing
correlation between a loopback POST and an upstream gossip emit;
it does not pretend to defend against a global passive adversary
(mix-net Loopix is tracked for Sprint 25+).

Idempotency note: :meth:`Dispatcher.submit` is idempotent by
design (duplicate ``task_id`` returns early without re-signing or
re-writing). This lets the queue retry an emit that partially
completed before a coordinator crash without producing duplicate
task entries on the doc.
"""

from __future__ import annotations

import asyncio
import json
import math
import secrets
import time
from dataclasses import asdict, is_dataclass
from pathlib import Path
from typing import Any, Awaitable, Callable, Protocol

import aiosqlite
import structlog

_log = structlog.get_logger(__name__)


class QueueFullError(RuntimeError):
    """Raised by :meth:`UploadQueue.schedule` when the queue has
    more than ``hard_cap`` pending rows. The API layer translates
    this into HTTP 429 Too Many Requests."""


class _RandomSource(Protocol):
    def random(self) -> float: ...


EmitFn = Callable[[dict[str, Any]], Awaitable[str]]
"""Emit callback signature. Takes a JSON-serializable payload
(typically :func:`dataclasses.asdict` of a
:class:`nexus_coordinator.dispatcher.SubmitRequest`) and returns
the resolved ``task_id`` as a string. Must be idempotent on
``task_id`` (a duplicate call with the same ``task_id`` must not
double-emit — Sprint 19 D design §6.3)."""


class UploadQueue:
    """Async SQLite-backed delay queue.

    The queue is single-writer (one coordinator owns one DB file)
    and uses a single asyncio event loop. Concurrent
    :meth:`schedule` calls across ``asyncio.gather`` are serialised
    by the internal ``_lock`` so the cap check + INSERT stays
    atomic against itself (two gather()ed schedules cannot both
    pass a near-the-cap check with the same size snapshot) and
    against the flush loop's DELETE.

    Test injection points (cf. design §6.9):

    - ``rng``: an object with a ``.random()`` → ``float ∈ [0, 1)``.
      Production passes :class:`secrets.SystemRandom` (CSPRNG);
      tests pass :class:`random.Random(seed=42)` for determinism.
    - ``now_fn``: callable returning a unix-epoch float. Production
      passes :func:`time.time`; tests pass a mutable lambda to mock
      wallclock advancement without depending on freezegun.
    """

    def __init__(
        self,
        *,
        db_path: Path,
        emit_fn: EmitFn,
        mean_jitter_s: float = 90.0,
        max_jitter_s: float = 300.0,
        flush_interval_s: float = 30.0,
        soft_cap: int = 10_000,
        hard_cap: int = 100_000,
        enabled: bool = True,
        rng: _RandomSource | None = None,
        now_fn: Callable[[], float] | None = None,
    ) -> None:
        if mean_jitter_s <= 0.0:
            raise ValueError("mean_jitter_s must be > 0")
        if max_jitter_s <= 0.0:
            raise ValueError("max_jitter_s must be > 0")
        if flush_interval_s <= 0.0:
            raise ValueError("flush_interval_s must be > 0")
        if flush_interval_s >= max_jitter_s:
            raise ValueError(
                "flush_interval_s must be < max_jitter_s so the internal "
                "clamp max_jitter_s - flush_interval_s stays positive"
            )
        if hard_cap < soft_cap:
            raise ValueError("hard_cap must be >= soft_cap")

        self.db_path = db_path
        self.emit_fn = emit_fn
        self.mean = float(mean_jitter_s)
        self.max = float(max_jitter_s)
        self.flush_interval = float(flush_interval_s)
        self.soft_cap = int(soft_cap)
        self.hard_cap = int(hard_cap)
        self.enabled = bool(enabled)

        # Internal clamp so that deliver_at + flush granularity
        # still lands under max_jitter_s observable (design §5.3).
        self._internal_max = max(1e-3, self.max - self.flush_interval)

        self._rng: _RandomSource = rng if rng is not None else secrets.SystemRandom()
        self._now: Callable[[], float] = now_fn if now_fn is not None else time.time

        self._lock = asyncio.Lock()
        self._loop_task: asyncio.Task[None] | None = None
        self._stopping = asyncio.Event()

    # ------------------------------------------------------------------
    # Schema lifecycle
    # ------------------------------------------------------------------

    async def init(self) -> None:
        """Create the ``delayed_uploads`` table if missing. WAL mode
        is enabled at connection time so the queue stays durable
        across coordinator crashes (design §5.2)."""
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        async with aiosqlite.connect(self.db_path) as db:
            await db.execute("PRAGMA journal_mode = WAL")
            await db.execute("PRAGMA synchronous = NORMAL")
            await db.execute(
                """
                CREATE TABLE IF NOT EXISTS delayed_uploads (
                    upload_id    TEXT PRIMARY KEY,
                    deliver_at   REAL NOT NULL,
                    task_payload TEXT NOT NULL,
                    enqueued_at  REAL NOT NULL
                )
                """
            )
            await db.execute("CREATE INDEX IF NOT EXISTS idx_delayed_uploads_deliver_at ON delayed_uploads(deliver_at)")
            await db.commit()

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def schedule(self, task_payload: dict[str, Any] | Any) -> str:
        """Enqueue a task with a random delay, return its ``task_id``.

        When ``enabled=False`` (dev escape hatch), the payload is
        emitted synchronously and the dispatcher's ``task_id`` is
        returned directly — no row is ever written to SQLite.
        """
        payload_dict = self._coerce_payload(task_payload)

        if not self.enabled:
            return await self.emit_fn(payload_dict)

        task_id = payload_dict.get("task_id")
        if not task_id:
            task_id = f"t-{secrets.token_hex(16)}"
            payload_dict["task_id"] = task_id

        delay = self._draw_delay()
        now = self._now()
        deliver_at = now + delay
        upload_id = secrets.token_hex(16)

        # The cap check + INSERT sit inside the same ``_lock`` so
        # two gather()ed schedules cannot both pass a near-the-cap
        # check with the same size snapshot. Without this, asyncio
        # cession points between ``_size`` and the INSERT let
        # concurrent callers race past ``hard_cap`` by up to N-1
        # rows (fix from Sprint 19 Phase D audit P2-1).
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                async with db.execute(
                    "SELECT COUNT(*) FROM delayed_uploads"
                ) as cursor:
                    row = await cursor.fetchone()
                size = int(row[0]) if row else 0
                if size >= self.hard_cap:
                    raise QueueFullError(
                        f"upload queue full ({size} >= hard cap {self.hard_cap})"
                    )
                if size >= self.soft_cap:
                    _log.warning(
                        "upload queue near soft cap",
                        size=size,
                        soft_cap=self.soft_cap,
                        hard_cap=self.hard_cap,
                    )
                await db.execute(
                    "INSERT INTO delayed_uploads "
                    "(upload_id, deliver_at, task_payload, enqueued_at) "
                    "VALUES (?, ?, ?, ?)",
                    (upload_id, deliver_at, json.dumps(payload_dict), now),
                )
                await db.commit()

        _log.info(
            "upload scheduled",
            upload_id=upload_id,
            task_id=task_id,
            delay_s=round(delay, 1),
            bucket=_bucket(delay),
        )
        return task_id

    async def start(self) -> None:
        """Initialise the schema and spawn the flush loop. Idempotent
        — calling twice raises ``RuntimeError`` to avoid leaking a
        background task."""
        if self._loop_task is not None:
            raise RuntimeError("upload queue already started")
        await self.init()
        await self._rerandomize_stale_on_boot()
        self._stopping.clear()
        self._loop_task = asyncio.create_task(self._flush_loop())

    async def shutdown(self, *, drain: bool = True) -> None:
        """Stop the flush loop. If ``drain`` is true, force-flush
        every pending row regardless of ``deliver_at`` — a crash-
        safe coordinator shutdown that prefers over-emit to data
        loss (design §5.1 shutdown comment)."""
        self._stopping.set()
        if self._loop_task is not None:
            try:
                await self._loop_task
            except asyncio.CancelledError:
                pass
            self._loop_task = None

        if drain:
            await self._flush_all_remaining()

    # ------------------------------------------------------------------
    # Internals
    # ------------------------------------------------------------------

    def _draw_delay(self) -> float:
        u = self._rng.random()
        if u <= 0.0:
            u = 1e-18
        raw = -self.mean * math.log(u)
        return min(raw, self._internal_max)

    async def _size(self) -> int:
        async with aiosqlite.connect(self.db_path) as db:
            async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
                row = await cursor.fetchone()
        return int(row[0]) if row else 0

    async def _flush_due(self) -> int:
        """Emit every row whose ``deliver_at`` is in the past. On
        emit success the row is deleted inside the same
        transaction so a mid-flush crash leaves the uncommitted
        rows available for retry. On emit failure the row is left
        in place and retried at the next tick."""
        now = self._now()
        flushed = 0
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                db.row_factory = aiosqlite.Row
                async with db.execute(
                    "SELECT upload_id, task_payload FROM delayed_uploads "
                    "WHERE deliver_at <= ? ORDER BY deliver_at ASC LIMIT 1000",
                    (now,),
                ) as cursor:
                    due_rows = await cursor.fetchall()

                for row in due_rows:
                    payload = json.loads(row["task_payload"])
                    try:
                        await self.emit_fn(payload)
                    except Exception as exc:  # noqa: BLE001
                        _log.error(
                            "upload emit failed, leaving in queue for retry",
                            upload_id=row["upload_id"],
                            error=str(exc),
                        )
                        continue
                    await db.execute(
                        "DELETE FROM delayed_uploads WHERE upload_id = ?",
                        (row["upload_id"],),
                    )
                    flushed += 1
                await db.commit()

        if flushed:
            _log.info("upload queue flushed", count=flushed)
        return flushed

    async def _flush_all_remaining(self) -> int:
        """Force-emit every remaining row ignoring ``deliver_at``.
        Used by :meth:`shutdown` when ``drain=True``."""
        flushed = 0
        async with self._lock:
            async with aiosqlite.connect(self.db_path) as db:
                db.row_factory = aiosqlite.Row
                async with db.execute("SELECT upload_id, task_payload FROM delayed_uploads") as cursor:
                    rows = await cursor.fetchall()
                for row in rows:
                    payload = json.loads(row["task_payload"])
                    try:
                        await self.emit_fn(payload)
                    except Exception as exc:  # noqa: BLE001
                        _log.warning(
                            "shutdown drain emit failed",
                            upload_id=row["upload_id"],
                            error=str(exc),
                        )
                        continue
                    await db.execute(
                        "DELETE FROM delayed_uploads WHERE upload_id = ?",
                        (row["upload_id"],),
                    )
                    flushed += 1
                await db.commit()
        if flushed:
            _log.info("upload queue drained on shutdown", count=flushed)
        return flushed

    async def _flush_loop(self) -> None:
        while not self._stopping.is_set():
            try:
                await asyncio.wait_for(self._stopping.wait(), timeout=self.flush_interval)
                break  # stop signalled
            except asyncio.TimeoutError:
                try:
                    await self._flush_due()
                except Exception as exc:  # noqa: BLE001 — never let the loop die
                    _log.error("upload queue flush tick failed", error=str(exc))

    async def _rerandomize_stale_on_boot(self) -> None:
        """Mitigate thundering herd after a long downtime (design
        §6.7). Any row whose ``deliver_at`` is already past at
        boot gets a fresh delay draw so the post-restart burst
        does not annihilate the anti-correlation property."""
        now = self._now()
        async with aiosqlite.connect(self.db_path) as db:
            db.row_factory = aiosqlite.Row
            async with db.execute(
                "SELECT upload_id FROM delayed_uploads WHERE deliver_at <= ?",
                (now,),
            ) as cursor:
                stale = await cursor.fetchall()
            if not stale:
                return
            for row in stale:
                new_delay = self._draw_delay()
                await db.execute(
                    "UPDATE delayed_uploads SET deliver_at = ? WHERE upload_id = ?",
                    (now + new_delay, row["upload_id"]),
                )
            await db.commit()
        _log.info(
            "upload queue re-randomized stale rows on boot",
            count=len(stale),
        )

    @staticmethod
    def _coerce_payload(task_payload: dict[str, Any] | Any) -> dict[str, Any]:
        if isinstance(task_payload, dict):
            return dict(task_payload)
        if is_dataclass(task_payload) and not isinstance(task_payload, type):
            return asdict(task_payload)
        raise TypeError(f"task_payload must be a dict or dataclass instance, got {type(task_payload).__name__}")


def _bucket(delay_s: float) -> str:
    """Log INFO histogram bucket name (design §5.5)."""
    if delay_s < 30:
        return "0-30"
    if delay_s < 60:
        return "30-60"
    if delay_s < 120:
        return "60-120"
    if delay_s < 180:
        return "120-180"
    if delay_s < 240:
        return "180-240"
    return "240-300"
