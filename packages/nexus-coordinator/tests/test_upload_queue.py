# SPDX-License-Identifier: AGPL-3.0-or-later
"""UploadQueue primitive tests — Sprint 19 Phase D.

Tests are deterministic by injecting ``rng`` (``random.Random(seed)``)
and ``now_fn`` (mutable closure) so there is no dependency on
``freezegun`` or system wallclock stability. The SQLite file lives
in ``tmp_path`` so each test gets an isolated DB.
"""

from __future__ import annotations

import asyncio
import json
import math
import random
import statistics
from pathlib import Path

import aiosqlite
import pytest
from nexus_coordinator.upload_queue import (
    QueueFullError,
    UploadQueue,
    _bucket,
)


class _Clock:
    """Mutable clock injected as ``now_fn`` into :class:`UploadQueue`."""

    def __init__(self, t: float = 1_000_000.0) -> None:
        self.t = t

    def __call__(self) -> float:
        return self.t

    def advance(self, seconds: float) -> None:
        self.t += seconds


def _payload(tid: str = "t-fixed-id", *, prompt: str = "hello") -> dict[str, object]:
    return {
        "task_type": "analysis",
        "prompt": prompt,
        "model": "stub-model:latest",
        "priority": 5,
        "task_id": tid,
    }


class _RecordingEmit:
    """Emit callback that records every payload + optionally raises."""

    def __init__(self, *, raise_n_first: int = 0) -> None:
        self.calls: list[dict[str, object]] = []
        self._raise_n = raise_n_first

    async def __call__(self, payload: dict[str, object]) -> str:
        if self._raise_n > 0:
            self._raise_n -= 1
            raise RuntimeError("simulated emit failure")
        self.calls.append(payload)
        return str(payload.get("task_id", "t-unknown"))


# ---------------------------------------------------------------------
# 1 — Distribution properties
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_schedule_within_max_range(tmp_path: Path) -> None:
    """100 draws must all respect the ``max_jitter_s`` clamp."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(42),
        now_fn=clock,
    )
    await queue.init()

    async with aiosqlite.connect(queue.db_path) as db:
        pass  # force open

    # Internal max = max_jitter_s - flush_interval_s = 270s.
    # Observable p99 = 270 + 30 flush tick = 300s max.
    for i in range(100):
        await queue.schedule(_payload(tid=f"t-{i:03d}"))

    async with aiosqlite.connect(queue.db_path) as db:
        db.row_factory = aiosqlite.Row
        async with db.execute("SELECT deliver_at, enqueued_at FROM delayed_uploads") as cursor:
            rows = await cursor.fetchall()
    delays = [r["deliver_at"] - r["enqueued_at"] for r in rows]
    assert len(delays) == 100
    assert max(delays) <= 270.0 + 1e-6, f"max delay {max(delays)} breaks internal clamp"
    assert min(delays) >= 0.0


@pytest.mark.asyncio
async def test_schedule_median_around_theoretical(tmp_path: Path) -> None:
    """On 1000 draws with seed=42 the median must land near the
    theoretical ln(2)·mean ≈ 62s. Tolerance ±25s accounts for
    sample variance at n=1000 plus the truncation to
    ``internal_max``."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(42),
        now_fn=clock,
    )
    await queue.init()

    draws = [queue._draw_delay() for _ in range(1000)]
    theoretical_median = math.log(2) * 90.0  # ≈ 62.38
    observed = statistics.median(draws)
    assert abs(observed - theoretical_median) < 25.0, (
        f"median drift too large: observed={observed}, theoretical={theoretical_median}"
    )


# ---------------------------------------------------------------------
# 2 — Persistence
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_schedule_persists_to_sqlite(tmp_path: Path) -> None:
    """Every schedule() writes one row whose JSON payload round-
    trips ``task_id`` and the resource-hint fields."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(42),
        now_fn=clock,
    )
    await queue.init()

    tid = await queue.schedule(_payload("t-known-id"))
    assert tid == "t-known-id"

    async with aiosqlite.connect(queue.db_path) as db:
        db.row_factory = aiosqlite.Row
        async with db.execute("SELECT upload_id, deliver_at, task_payload, enqueued_at FROM delayed_uploads") as cursor:
            rows = await cursor.fetchall()
    assert len(rows) == 1
    stored = json.loads(rows[0]["task_payload"])
    assert stored["task_id"] == "t-known-id"
    assert stored["prompt"] == "hello"
    # deliver_at > enqueued_at since some delay was drawn.
    assert rows[0]["deliver_at"] > rows[0]["enqueued_at"]


# ---------------------------------------------------------------------
# 3 — Flush semantics
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_flush_due_emits_only_due_rows(tmp_path: Path) -> None:
    """_flush_due emits rows whose ``deliver_at`` is in the past
    and leaves future rows untouched."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=0.5,
        max_jitter_s=60.0,
        flush_interval_s=5.0,
        rng=random.Random(7),
        now_fn=clock,
    )
    await queue.init()

    # Force deterministic deliver_at by bypassing _draw_delay:
    # insert 3 rows at now, 3 at now+120.
    now = clock()
    async with aiosqlite.connect(queue.db_path) as db:
        for i in range(3):
            await db.execute(
                "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
                (f"due-{i}", now - 1.0, json.dumps(_payload(f"t-due-{i}")), now),
            )
        for i in range(3):
            await db.execute(
                "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
                (f"future-{i}", now + 120.0, json.dumps(_payload(f"t-fut-{i}")), now),
            )
        await db.commit()

    flushed = await queue._flush_due()
    assert flushed == 3
    emitted_ids = {c["task_id"] for c in emit.calls}
    assert emitted_ids == {"t-due-0", "t-due-1", "t-due-2"}

    async with aiosqlite.connect(queue.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            remaining = await cursor.fetchone()
    assert remaining[0] == 3  # future rows still there


@pytest.mark.asyncio
async def test_flush_due_keeps_row_on_emit_failure(tmp_path: Path) -> None:
    """When emit_fn raises, the row must stay in the table so the
    next flush retries it."""
    emit = _RecordingEmit(raise_n_first=1)
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(1),
        now_fn=clock,
    )
    await queue.init()

    now = clock()
    async with aiosqlite.connect(queue.db_path) as db:
        await db.execute(
            "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
            ("row-1", now - 1.0, json.dumps(_payload("t-retry")), now),
        )
        await db.commit()

    # First tick: emit raises → row stays.
    flushed = await queue._flush_due()
    assert flushed == 0
    async with aiosqlite.connect(queue.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            cnt = await cursor.fetchone()
    assert cnt[0] == 1

    # Second tick: emit succeeds → row deleted.
    flushed = await queue._flush_due()
    assert flushed == 1
    async with aiosqlite.connect(queue.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            cnt = await cursor.fetchone()
    assert cnt[0] == 0


# ---------------------------------------------------------------------
# 4 — Lifecycle (start / scheduler / shutdown)
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_scheduler_loop_wakes_on_interval(tmp_path: Path) -> None:
    """Run the loop with a very short interval + stale rows and
    verify at least one flush happens within the timeout."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=0.05,
        max_jitter_s=0.2,
        flush_interval_s=0.05,
        rng=random.Random(99),
        now_fn=clock,
    )
    await queue.start()

    # Pre-seed one overdue row.
    now = clock()
    async with aiosqlite.connect(queue.db_path) as db:
        await db.execute(
            "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
            ("seed", now - 1.0, json.dumps(_payload("t-loop")), now),
        )
        await db.commit()

    # Give the loop ~6 ticks worth of wall time.
    await asyncio.sleep(0.3)
    await queue.shutdown(drain=False)

    assert any(c["task_id"] == "t-loop" for c in emit.calls)


@pytest.mark.asyncio
async def test_shutdown_drains_pending(tmp_path: Path) -> None:
    """``shutdown(drain=True)`` must flush every row, including
    those whose ``deliver_at`` is still in the future."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=10.0,
        max_jitter_s=100.0,
        flush_interval_s=5.0,
        rng=random.Random(3),
        now_fn=clock,
    )
    await queue.start()

    for i in range(5):
        await queue.schedule(_payload(f"t-drain-{i}"))

    # None of the 5 rows is due (they live ~10s in the future
    # typical) — only drain should flush them.
    await queue.shutdown(drain=True)

    emitted = {c["task_id"] for c in emit.calls}
    assert emitted == {f"t-drain-{i}" for i in range(5)}

    async with aiosqlite.connect(queue.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            cnt = await cursor.fetchone()
    assert cnt[0] == 0


# ---------------------------------------------------------------------
# 5 — Concurrency + backpressure
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_concurrent_schedule_all_land(tmp_path: Path) -> None:
    """50 gather()-ed schedule() calls must each land one row."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(11),
        now_fn=clock,
    )
    await queue.init()

    await asyncio.gather(*[queue.schedule(_payload(f"t-c-{i:03d}")) for i in range(50)])

    async with aiosqlite.connect(queue.db_path) as db:
        db.row_factory = aiosqlite.Row
        async with db.execute("SELECT task_payload FROM delayed_uploads") as cursor:
            rows = await cursor.fetchall()
    assert len(rows) == 50
    ids = {json.loads(r["task_payload"])["task_id"] for r in rows}
    assert ids == {f"t-c-{i:03d}" for i in range(50)}


@pytest.mark.asyncio
async def test_hard_cap_enforced_under_concurrency(tmp_path: Path) -> None:
    """Fire 20 gather()ed schedules against ``hard_cap=5``: exactly
    5 must land, and the 15 surplus callers must each raise
    :class:`QueueFullError`. Regression for Sprint 19 Phase D audit
    P2-1 TOCTOU fix — without the lock around the cap check +
    INSERT, concurrent callers raced past the cap by up to N-1
    rows."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        soft_cap=5,
        hard_cap=5,
        rng=random.Random(17),
        now_fn=clock,
    )
    await queue.init()

    results = await asyncio.gather(
        *[queue.schedule(_payload(f"t-race-{i:03d}")) for i in range(20)],
        return_exceptions=True,
    )
    succeeded = [r for r in results if not isinstance(r, BaseException)]
    failed = [r for r in results if isinstance(r, QueueFullError)]
    others = [
        r
        for r in results
        if isinstance(r, BaseException) and not isinstance(r, QueueFullError)
    ]
    assert not others, f"unexpected exceptions: {others}"
    assert len(succeeded) == 5
    assert len(failed) == 15

    async with aiosqlite.connect(queue.db_path) as db:
        async with db.execute("SELECT COUNT(*) FROM delayed_uploads") as cursor:
            cnt = await cursor.fetchone()
    assert cnt[0] == 5, "hard cap was breached by concurrent submits"


@pytest.mark.asyncio
async def test_hard_cap_raises_queue_full(tmp_path: Path) -> None:
    """Past the hard cap :meth:`schedule` must raise
    :class:`QueueFullError` (the API layer translates to 429)."""
    emit = _RecordingEmit()
    clock = _Clock()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        soft_cap=2,
        hard_cap=3,
        rng=random.Random(5),
        now_fn=clock,
    )
    await queue.init()

    for i in range(3):
        await queue.schedule(_payload(f"t-cap-{i}"))

    with pytest.raises(QueueFullError):
        await queue.schedule(_payload("t-over-cap"))


# ---------------------------------------------------------------------
# 6 — Boot recovery
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_start_rerandomizes_stale_rows(tmp_path: Path) -> None:
    """At boot, every row whose ``deliver_at`` is already past gets
    a fresh random delay so the post-restart burst does not
    collapse the anti-correlation property."""
    emit = _RecordingEmit()
    clock = _Clock()
    db_path = tmp_path / "uq.sqlite"

    # Pre-seed two stale rows + one still-future row.
    queue_bootstrap = UploadQueue(
        db_path=db_path,
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(1),
        now_fn=clock,
    )
    await queue_bootstrap.init()
    now = clock()
    async with aiosqlite.connect(db_path) as db:
        for i, deliver in enumerate([now - 3600, now - 1800, now + 200]):
            await db.execute(
                "INSERT INTO delayed_uploads (upload_id, deliver_at, task_payload, enqueued_at) VALUES (?, ?, ?, ?)",
                (f"boot-{i}", deliver, json.dumps(_payload(f"t-boot-{i}")), now),
            )
        await db.commit()

    # Fresh queue + start — stale rows should be re-randomized.
    queue = UploadQueue(
        db_path=db_path,
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        rng=random.Random(2),
        now_fn=clock,
    )
    await queue.start()
    try:
        async with aiosqlite.connect(db_path) as db:
            db.row_factory = aiosqlite.Row
            async with db.execute("SELECT upload_id, deliver_at FROM delayed_uploads ORDER BY upload_id") as cursor:
                rows = await cursor.fetchall()
        by_id = {r["upload_id"]: r["deliver_at"] for r in rows}
        # The two stale rows now land strictly in the future.
        assert by_id["boot-0"] > now
        assert by_id["boot-1"] > now
        # And the internal clamp still holds (max 270s = 300 - 30).
        assert by_id["boot-0"] <= now + 270.0 + 1e-6
        assert by_id["boot-1"] <= now + 270.0 + 1e-6
        # The non-stale row was left alone.
        assert by_id["boot-2"] == pytest.approx(now + 200.0)
    finally:
        await queue.shutdown(drain=False)


# ---------------------------------------------------------------------
# 7 — Disabled passthrough
# ---------------------------------------------------------------------


@pytest.mark.asyncio
async def test_disabled_passthrough_skips_db(tmp_path: Path) -> None:
    """When ``enabled=False`` :meth:`schedule` must call emit_fn
    directly and never touch SQLite."""
    emit = _RecordingEmit()
    queue = UploadQueue(
        db_path=tmp_path / "uq.sqlite",
        emit_fn=emit,
        mean_jitter_s=90.0,
        max_jitter_s=300.0,
        flush_interval_s=30.0,
        enabled=False,
        rng=random.Random(1),
        now_fn=_Clock(),
    )
    # Note: no init(), no start().
    tid = await queue.schedule(_payload("t-pass"))
    assert tid == "t-pass"
    assert [c["task_id"] for c in emit.calls] == ["t-pass"]
    # SQLite file untouched — no table ever created.
    assert not queue.db_path.exists()


# ---------------------------------------------------------------------
# 8 — Bucket helper (log INFO histogram)
# ---------------------------------------------------------------------


def test_bucket_partitions_delay_range() -> None:
    """Every delay in [0, 300] must fall into exactly one bucket."""
    assert _bucket(0.0) == "0-30"
    assert _bucket(29.99) == "0-30"
    assert _bucket(30.0) == "30-60"
    assert _bucket(59.99) == "30-60"
    assert _bucket(60.0) == "60-120"
    assert _bucket(119.99) == "60-120"
    assert _bucket(120.0) == "120-180"
    assert _bucket(180.0) == "180-240"
    assert _bucket(240.0) == "240-300"
    assert _bucket(299.99) == "240-300"
    assert _bucket(400.0) == "240-300"  # over max still lands sensibly
