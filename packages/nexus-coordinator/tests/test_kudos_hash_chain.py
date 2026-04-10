"""Kudos hash-chain integrity tests.

These tests use the real SQLite + jcs + pynacl stack — no mocks.
A flipped byte anywhere in the ledger must break the chain from
that row onward.
"""

from __future__ import annotations

from pathlib import Path

import aiosqlite
import nacl.signing
import pytest
from nexus_coordinator.db.migrations import init_db
from nexus_coordinator.kudos import KudosLedger


def _fresh_secret() -> bytes:
    return bytes(nacl.signing.SigningKey.generate())  # 32 raw bytes


@pytest.mark.asyncio
async def test_credit_single_entry_verifies(tmp_path: Path) -> None:
    db = tmp_path / "state.sqlite"
    await init_db(db)
    secret = _fresh_secret()
    ledger = KudosLedger(db_path=db, coord_secret=secret)

    row_id = await ledger.credit(
        worker_pubkey=b"\x11" * 32,
        task_id="t-1",
        tokens=100,
    )
    assert row_id == 1

    ok, bad = await ledger.verify_chain_integrity()
    assert ok, f"fresh single-row chain should verify, got bad={bad}"
    assert bad is None


@pytest.mark.asyncio
async def test_credit_ten_entries_chain_verifies(tmp_path: Path) -> None:
    db = tmp_path / "state.sqlite"
    await init_db(db)
    secret = _fresh_secret()
    ledger = KudosLedger(db_path=db, coord_secret=secret)

    for i in range(10):
        await ledger.credit(
            worker_pubkey=bytes([i]) * 32,
            task_id=f"t-{i}",
            tokens=10 * (i + 1),
        )

    entries = await ledger.list_entries(limit=100)
    assert len(entries) == 10
    ok, bad = await ledger.verify_chain_integrity()
    assert ok, f"10-row chain should verify, got bad={bad}"


@pytest.mark.asyncio
async def test_tampered_amount_breaks_chain(tmp_path: Path) -> None:
    db = tmp_path / "state.sqlite"
    await init_db(db)
    secret = _fresh_secret()
    ledger = KudosLedger(db_path=db, coord_secret=secret)

    for i in range(5):
        await ledger.credit(
            worker_pubkey=bytes([i + 1]) * 32,
            task_id=f"t-{i}",
            tokens=100,
        )

    # Tamper row 3: flip the amount from 100 to 999. The stored
    # entry_hash and signature were computed over the old amount,
    # so verify_chain_integrity must detect the break at row 3.
    async with aiosqlite.connect(db) as conn:
        await conn.execute("UPDATE kudos_ledger SET amount = 999.0 WHERE id = 3")
        await conn.commit()

    ok, bad = await ledger.verify_chain_integrity()
    assert not ok
    assert bad == 3


@pytest.mark.asyncio
async def test_tampered_entry_hash_breaks_chain(tmp_path: Path) -> None:
    db = tmp_path / "state.sqlite"
    await init_db(db)
    secret = _fresh_secret()
    ledger = KudosLedger(db_path=db, coord_secret=secret)

    for i in range(3):
        await ledger.credit(
            worker_pubkey=bytes([i + 1]) * 32,
            task_id=f"t-{i}",
            tokens=50,
        )

    # Flip a single byte in entry_hash on row 2. The recomputed
    # hash will not match, and the signature verify will also
    # fail (the sig was over the original hash).
    async with aiosqlite.connect(db) as conn:
        async with conn.execute("SELECT entry_hash FROM kudos_ledger WHERE id = 2") as cursor:
            row = await cursor.fetchone()
        assert row is not None
        tampered = bytearray(row[0])
        tampered[0] ^= 0xFF
        await conn.execute(
            "UPDATE kudos_ledger SET entry_hash = ? WHERE id = 2",
            (bytes(tampered),),
        )
        await conn.commit()

    ok, bad = await ledger.verify_chain_integrity()
    assert not ok
    assert bad == 2


@pytest.mark.asyncio
async def test_total_for_worker(tmp_path: Path) -> None:
    db = tmp_path / "state.sqlite"
    await init_db(db)
    secret = _fresh_secret()
    ledger = KudosLedger(db_path=db, coord_secret=secret)

    worker = b"\x42" * 32
    await ledger.credit(worker_pubkey=worker, task_id="t-1", tokens=100)
    await ledger.credit(worker_pubkey=worker, task_id="t-2", tokens=250)
    await ledger.credit(worker_pubkey=b"\x43" * 32, task_id="t-3", tokens=1000)

    total = await ledger.total_for_worker(worker)
    assert total == 350.0
