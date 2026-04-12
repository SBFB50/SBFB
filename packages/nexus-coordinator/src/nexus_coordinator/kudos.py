# SPDX-License-Identifier: AGPL-3.0-or-later
"""Kudos ledger: append-only Ed25519-signed hash chain of credits.

Formula::

    kudos = tokens × quality_factor × trust_multiplier

- ``tokens`` comes from the worker's reported ``tokens_generated``.
- ``quality_factor`` defaults to 1.0 in v1.0 and can be tuned per
  task_type by the caller (Phase D+).
- ``trust_multiplier`` defaults to 1.0 until the validator has a
  per-worker trust history (Phase D adds this).

Hash chain layout::

    prev_hash[0]  = 32 zero bytes
    entry[N]      = { worker_pubkey_hex, task_id, tokens,
                      quality_factor, trust_multiplier, amount,
                      awarded_at }
    entry_bytes[N]  = jcs.canonicalize(entry[N])
    entry_hash[N] = sha256(prev_hash[N-1] || DOMAIN_KUDOS_V1 || 0x00 || entry_bytes[N])
    entry_sig[N]  = Ed25519.sign(coord_secret, entry_hash[N])
    prev_hash[N]  = entry_hash[N]

:meth:`KudosLedger.verify_chain_integrity` replays the chain from
row 1 and returns ``(True, None)`` on success or
``(False, first_bad_id)`` when a row fails its hash or signature
check.

Matches ``DOMAIN_KUDOS_V1`` in ``crates/nexus-core-rs/src/canonical.rs``.
"""

from __future__ import annotations

import hashlib
import time
from dataclasses import dataclass
from pathlib import Path

import aiosqlite
import jcs
import nacl.signing
import structlog

_log = structlog.get_logger(__name__)

# Matches nexus_core_rs::canonical::DOMAIN_KUDOS_V1.
DOMAIN_KUDOS_V1 = b"nexus-kudos-v1"
_ZERO_HASH = bytes(32)


@dataclass
class KudosEntry:
    """A single row in the kudos ledger, as returned by
    :meth:`KudosLedger.list_entries`."""

    id: int
    worker_pubkey: bytes
    task_id: str
    tokens: int
    quality_factor: float
    trust_multiplier: float
    amount: float
    awarded_at: int
    prev_hash: bytes
    entry_hash: bytes
    entry_sig: bytes


def _canonical_entry_bytes(
    worker_pubkey: bytes,
    task_id: str,
    tokens: int,
    quality_factor: float,
    trust_multiplier: float,
    amount: float,
    awarded_at: int,
) -> bytes:
    """Produce the canonical bytes (RFC 8785 JCS) for a kudos entry."""
    # Key names use lowercase_snake_case to match the canonical
    # nexus-grid convention; jcs sorts them lexicographically at
    # serialization time regardless of insertion order here.
    obj = {
        "amount": amount,
        "awarded_at": awarded_at,
        "quality_factor": quality_factor,
        "task_id": task_id,
        "tokens": tokens,
        "trust_multiplier": trust_multiplier,
        "worker_pubkey_hex": worker_pubkey.hex(),
    }
    # jcs.canonicalize returns bytes on recent releases.
    return jcs.canonicalize(obj)


def _entry_hash(prev_hash: bytes, entry_bytes: bytes) -> bytes:
    """Compute the hash-chain hash for a single entry."""
    h = hashlib.sha256()
    h.update(prev_hash)
    h.update(DOMAIN_KUDOS_V1)
    h.update(b"\x00")
    h.update(entry_bytes)
    return h.digest()


class KudosLedger:
    """Append-only kudos store backed by the ``kudos_ledger`` SQLite table."""

    def __init__(self, *, db_path: Path, coord_secret: bytes) -> None:
        self._db_path = db_path
        self._signing_key = nacl.signing.SigningKey(coord_secret)

    async def credit(
        self,
        *,
        worker_pubkey: bytes,
        task_id: str,
        tokens: int,
        quality_factor: float = 1.0,
        trust_multiplier: float = 1.0,
        awarded_at: int | None = None,
    ) -> int:
        """Append a new credit to the ledger. Returns the row id.

        The caller is responsible for making the (worker, task_id)
        tuple unique if they want no double-credit protection —
        the ledger itself treats every call as legitimate.
        """
        if len(worker_pubkey) != 32:
            raise ValueError(f"worker_pubkey must be 32 bytes, got {len(worker_pubkey)}")
        if tokens < 0:
            raise ValueError(f"tokens must be non-negative, got {tokens}")

        amount = float(tokens) * quality_factor * trust_multiplier
        ts = awarded_at if awarded_at is not None else int(time.time())

        async with aiosqlite.connect(self._db_path) as db:
            # Take the latest entry_hash so the chain is continuous
            # across concurrent calls. SQLite's default behaviour is
            # serializable so this read + insert pair is effectively
            # atomic as long as no other process writes to the same
            # DB.
            async with db.execute("SELECT entry_hash FROM kudos_ledger ORDER BY id DESC LIMIT 1") as cursor:
                row = await cursor.fetchone()
            prev_hash = row[0] if row else _ZERO_HASH

            entry_bytes = _canonical_entry_bytes(
                worker_pubkey,
                task_id,
                tokens,
                quality_factor,
                trust_multiplier,
                amount,
                ts,
            )
            entry_hash = _entry_hash(prev_hash, entry_bytes)
            entry_sig = self._signing_key.sign(entry_hash).signature

            cursor = await db.execute(
                """
                INSERT INTO kudos_ledger (
                    worker_pubkey, task_id, tokens, quality_factor,
                    trust_multiplier, amount, awarded_at, prev_hash,
                    entry_hash, entry_sig
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    worker_pubkey,
                    task_id,
                    tokens,
                    quality_factor,
                    trust_multiplier,
                    amount,
                    ts,
                    prev_hash,
                    entry_hash,
                    entry_sig,
                ),
            )
            row_id = cursor.lastrowid
            await cursor.close()
            await db.commit()

        _log.info(
            "kudos credited",
            worker_pubkey_hex=worker_pubkey.hex(),
            task_id=task_id,
            tokens=tokens,
            amount=amount,
            row_id=row_id,
        )
        return int(row_id or 0)

    async def list_entries(self, worker_pubkey: bytes | None = None, limit: int = 100) -> list[KudosEntry]:
        """Return recent ledger entries, optionally filtered by worker."""
        query = (
            "SELECT id, worker_pubkey, task_id, tokens, quality_factor, "
            "trust_multiplier, amount, awarded_at, prev_hash, entry_hash, entry_sig "
            "FROM kudos_ledger"
        )
        params: tuple = ()
        if worker_pubkey is not None:
            query += " WHERE worker_pubkey = ?"
            params = (worker_pubkey,)
        query += " ORDER BY id DESC LIMIT ?"
        params = (*params, limit)

        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(query, params) as cursor:
                rows = await cursor.fetchall()
        return [
            KudosEntry(
                id=row[0],
                worker_pubkey=row[1],
                task_id=row[2],
                tokens=row[3],
                quality_factor=row[4],
                trust_multiplier=row[5],
                amount=row[6],
                awarded_at=row[7],
                prev_hash=row[8],
                entry_hash=row[9],
                entry_sig=row[10],
            )
            for row in rows
        ]

    async def total_for_worker(self, worker_pubkey: bytes) -> float:
        """Sum of ``amount`` across every credit for a single worker."""
        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(
                "SELECT COALESCE(SUM(amount), 0.0) FROM kudos_ledger WHERE worker_pubkey = ?",
                (worker_pubkey,),
            ) as cursor:
                row = await cursor.fetchone()
        return float(row[0]) if row else 0.0

    async def verify_chain_integrity(self) -> tuple[bool, int | None]:
        """Replay the whole chain and return (ok, first_bad_row_id).

        - ``(True, None)`` — chain is intact from row 1 to the head.
        - ``(False, N)``  — row ``N`` fails either its hash or its
          signature check. ``N`` is 1-indexed.
        """
        verify_key = self._signing_key.verify_key

        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(
                """
                SELECT id, worker_pubkey, task_id, tokens, quality_factor,
                       trust_multiplier, amount, awarded_at, prev_hash,
                       entry_hash, entry_sig
                FROM kudos_ledger
                ORDER BY id ASC
                """
            ) as cursor:
                rows = await cursor.fetchall()

        expected_prev = _ZERO_HASH
        for row in rows:
            (
                row_id,
                worker_pubkey,
                task_id,
                tokens,
                quality_factor,
                trust_multiplier,
                amount,
                awarded_at,
                prev_hash,
                entry_hash,
                entry_sig,
            ) = row

            if prev_hash != expected_prev:
                _log.warning(
                    "kudos chain break: prev_hash mismatch",
                    row_id=row_id,
                    expected=expected_prev.hex(),
                    actual=prev_hash.hex(),
                )
                return (False, int(row_id))

            entry_bytes = _canonical_entry_bytes(
                worker_pubkey,
                task_id,
                tokens,
                quality_factor,
                trust_multiplier,
                amount,
                awarded_at,
            )
            recomputed = _entry_hash(prev_hash, entry_bytes)
            if recomputed != entry_hash:
                _log.warning(
                    "kudos chain break: entry_hash tampered",
                    row_id=row_id,
                )
                return (False, int(row_id))

            try:
                verify_key.verify(entry_hash, entry_sig)
            except Exception:
                _log.warning("kudos chain break: bad signature", row_id=row_id)
                return (False, int(row_id))

            expected_prev = entry_hash

        return (True, None)


def kudos_dependency_missing_hint() -> str:
    """Return a helpful install hint if PyNaCl is missing at import.

    Included for discoverability — the actual import at the top of
    the module will raise ``ModuleNotFoundError`` before this
    helper is reachable, but the message is surfaced by any caller
    that does an isinstance check.
    """
    return "pip install pynacl>=1.5"
