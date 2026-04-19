# SPDX-License-Identifier: AGPL-3.0-or-later
"""QuarantineQueue primitive tests — Sprint 21 Phase D.

Tests are deterministic by injecting ``now_fn`` (mutable closure)
so there is no dependency on ``freezegun`` or system wallclock
stability. The SQLite file lives in ``tmp_path`` so each test
gets an isolated DB. Pattern miroir Sprint 19 D
``test_upload_queue.py`` — same ``_Clock`` shape, same tmp_path
discipline.
"""

from __future__ import annotations

import asyncio
from pathlib import Path

import pytest
from nexus_coordinator.quarantine_queue import QuarantineQueue


class _Clock:
    """Mutable clock injected as ``now_fn`` into :class:`QuarantineQueue`."""

    def __init__(self, t: int = 1_000_000) -> None:
        self.t = t

    def __call__(self) -> int:
        return self.t

    def advance(self, seconds: int) -> None:
        self.t += seconds


def _pubkey(byte: int = 0x01) -> bytes:
    return bytes([byte] * 32)


def _payload(seed: int = 1) -> bytes:
    return bytes([seed]) * 256


@pytest.mark.asyncio
async def test_add_then_list_returns_entry(tmp_path: Path) -> None:
    """add() inserts a row, list(status='pending') returns it with
    every column populated correctly."""
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=60.0,
        now_fn=_Clock(1_700_000_000),
    )
    await queue.init()

    row_id = await queue.add(
        topic="nexus-grid/test/v1",
        sender_pubkey=_pubkey(0x07),
        payload_bytes=_payload(0x42),
        rate_strikes=2,
        pow_status="valid",
    )
    assert row_id == 1

    rows = await queue.list(status="pending")
    assert len(rows) == 1
    row = rows[0]
    assert row["id"] == 1
    assert row["topic"] == "nexus-grid/test/v1"
    assert bytes(row["sender_pubkey"]) == _pubkey(0x07)
    assert bytes(row["payload_bytes"]) == _payload(0x42)
    assert row["received_at_epoch_s"] == 1_700_000_000
    assert row["rate_strikes"] == 2
    assert row["pow_status"] == "valid"
    assert row["flush_status"] == "pending"


@pytest.mark.asyncio
async def test_ttl_15min_auto_drop(tmp_path: Path) -> None:
    """An entry inserted at t=0 is gone after the sweep at t=901
    (TTL = 900s). Mock clock so the test runs in milliseconds."""
    clock = _Clock(1_000_000)
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=60.0,
        now_fn=clock,
    )
    await queue.init()

    await queue.add(
        topic="t",
        sender_pubkey=_pubkey(),
        payload_bytes=b"x",
        rate_strikes=0,
        pow_status="valid",
    )
    assert len(await queue.list(status="pending")) == 1

    # Advance past TTL window and trigger sweep manually (no
    # need to spawn the loop task — the primitive is testable
    # standalone).
    clock.advance(901)
    deleted = await queue._auto_drop_expired()
    assert deleted == 1
    assert await queue.list(status="pending") == []


@pytest.mark.asyncio
async def test_manual_flush_marks_status(tmp_path: Path) -> None:
    """flush(row_id) sets flush_status='flushed' and returns True
    once. A second flush call returns False (no longer pending).
    Re-injection into gossip is hors-scope Phase D — only the
    audit-trail status flip is asserted here."""
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=60.0,
    )
    await queue.init()

    row_id = await queue.add(
        topic="t",
        sender_pubkey=_pubkey(),
        payload_bytes=b"x",
        rate_strikes=1,
        pow_status="valid",
    )
    assert await queue.flush(row_id) is True
    assert await queue.flush(row_id) is False  # already non-pending

    pending = await queue.list(status="pending")
    flushed = await queue.list(status="flushed")
    assert pending == []
    assert len(flushed) == 1
    assert flushed[0]["flush_status"] == "flushed"


@pytest.mark.asyncio
async def test_manual_drop_sets_status(tmp_path: Path) -> None:
    """drop(row_id) sets flush_status='dropped'. The entry stays
    in the table for audit trail (only TTL-sweep removes pending
    rows)."""
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=60.0,
    )
    await queue.init()

    row_id = await queue.add(
        topic="t",
        sender_pubkey=_pubkey(),
        payload_bytes=b"x",
        rate_strikes=0,
        pow_status="invalid",
    )
    assert await queue.drop(row_id) is True
    assert await queue.drop(row_id) is False

    dropped = await queue.list(status="dropped")
    assert len(dropped) == 1
    assert dropped[0]["flush_status"] == "dropped"
    assert dropped[0]["pow_status"] == "invalid"  # audit metadata preserved


@pytest.mark.asyncio
async def test_cardinality_1k_entries_sweeps_clean(tmp_path: Path) -> None:
    """Bulk-insert 1k entries (one aiosqlite connect+commit per
    add, the production hot path), then trigger a TTL sweep — the
    primitive must handle the load without panic and the sweep
    must DELETE every expired row in one call. 1k is well above
    the steady-state estimate (~150 entries / 15-min TTL window
    cf. design §6.1) and stays comfortably inside the pytest
    60s default timeout. A dedicated 10k stress harness is
    deferred to a Phase F bench follow-up."""
    clock = _Clock(1_000_000)
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=60.0,
        now_fn=clock,
    )
    await queue.init()

    for i in range(1_000):
        await queue.add(
            topic=f"topic-{i % 10}",
            sender_pubkey=_pubkey(0x01),
            payload_bytes=b"x" * 256,
            rate_strikes=0,
            pow_status="valid",
        )
    assert len(await queue.list(status="pending")) == 1_000

    clock.advance(901)
    deleted = await queue._auto_drop_expired()
    assert deleted == 1_000
    assert await queue.list(status="pending") == []


@pytest.mark.asyncio
async def test_add_rejects_invalid_pow_status(tmp_path: Path) -> None:
    """Defensive: add() refuses an out-of-domain ``pow_status``
    value so a buggy caller cannot corrupt the audit column."""
    queue = QuarantineQueue(db_path=tmp_path / "q.sqlite")
    await queue.init()
    with pytest.raises(ValueError, match="pow_status"):
        await queue.add(
            topic="t",
            sender_pubkey=_pubkey(),
            payload_bytes=b"x",
            rate_strikes=0,
            pow_status="bogus",
        )


@pytest.mark.asyncio
async def test_list_rejects_invalid_status(tmp_path: Path) -> None:
    """Defensive: list() refuses unknown status filters."""
    queue = QuarantineQueue(db_path=tmp_path / "q.sqlite")
    await queue.init()
    with pytest.raises(ValueError, match="status"):
        await queue.list(status="bogus")


@pytest.mark.asyncio
async def test_start_shutdown_idempotency(tmp_path: Path) -> None:
    """start() spawns the sweep loop ; calling start() twice raises ;
    shutdown() stops the loop cleanly even if no entries were added."""
    queue = QuarantineQueue(
        db_path=tmp_path / "q.sqlite",
        ttl_seconds=900,
        sweep_interval_s=0.05,  # fast sweep so the loop ticks during the test
    )
    await queue.start()
    with pytest.raises(RuntimeError, match="already started"):
        await queue.start()
    # Let the sweep loop tick at least once with no entries
    await asyncio.sleep(0.1)
    await queue.shutdown()
    # shutdown is idempotent (the second call hits a None task
    # and returns without raising)
    await queue.shutdown()
