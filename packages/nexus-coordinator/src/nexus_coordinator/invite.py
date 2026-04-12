# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coordinator-side invite minting, tracking, and revocation.

The wire format itself lives in the Rust crate
``nexus-worker-core::invite`` and is reached through
:func:`nexus_core.mint_invite` / :func:`nexus_core.decode_invite`.
This module adds the **persistence** layer the Rust crate does
not care about:

- ``invites`` SQLite table with (id, wire, scope, expires_at,
  max_uses, uses_count, revoked_at, note) so the coordinator can
  list / revoke / cap invites at the Python API boundary.
- :class:`InviteLedger` wraps CRUD + validity checks.

The Rust decoder does the cryptographic work (signature +
expiry + Worker-requires-ticket); this ledger layers the
"coordinator-issued and not revoked" gate on top.
"""

from __future__ import annotations

import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import aiosqlite
import nexus_core
import structlog

_log = structlog.get_logger(__name__)

_INVITES_SCHEMA = """
CREATE TABLE IF NOT EXISTS invites (
    id            TEXT PRIMARY KEY,
    wire          TEXT NOT NULL UNIQUE,
    scope         TEXT NOT NULL,
    project_id    TEXT NOT NULL,
    project_name  TEXT NOT NULL,
    expires_at    INTEGER NOT NULL,
    max_uses      INTEGER,
    uses_count    INTEGER NOT NULL DEFAULT 0,
    revoked_at    INTEGER,
    note          TEXT,
    created_at    INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS invites_by_revoked ON invites(revoked_at);
CREATE INDEX IF NOT EXISTS invites_by_expires_at ON invites(expires_at);
"""


@dataclass
class InviteRecord:
    """A single row from the ``invites`` table."""

    id: str
    wire: str
    scope: str
    project_id: str
    project_name: str
    expires_at: int
    max_uses: int | None
    uses_count: int
    revoked_at: int | None
    note: str | None
    created_at: int


class InviteLedger:
    """CRUD + validity tracking for coordinator-issued invites."""

    def __init__(self, *, db_path: Path, coord_secret: bytes) -> None:
        self._db_path = db_path
        self._coord_secret = coord_secret

    async def init(self) -> None:
        async with aiosqlite.connect(self._db_path) as db:
            await db.executescript(_INVITES_SCHEMA)
            await db.commit()

    async def mint(
        self,
        *,
        project_id: str,
        project_name: str,
        scope: str,
        tasks_doc_ticket: str | None,
        expiry_secs: int,
        coordinator_addr: str | None = None,
        max_uses: int | None = None,
        note: str | None = None,
    ) -> InviteRecord:
        """Mint a new invite and persist it. Returns the full record.

        Validates scope + ticket combination before calling the
        Rust mint function, so a caller error (worker scope
        without ticket) is surfaced as a ``ValueError`` with the
        same message shape as the Rust side.
        """
        if scope not in ("worker", "observer"):
            raise ValueError(f"scope must be 'worker' or 'observer', got {scope!r}")
        if scope == "worker" and not tasks_doc_ticket:
            raise ValueError("tasks_doc_ticket is required when scope == 'worker'")
        if expiry_secs <= 0:
            raise ValueError(f"expiry_secs must be positive, got {expiry_secs}")

        now = int(time.time())
        expires_at = now + expiry_secs
        wire = nexus_core.mint_invite(
            self._coord_secret,
            project_id,
            project_name,
            coordinator_addr,
            tasks_doc_ticket,
            scope,
            expires_at,
        )

        record = InviteRecord(
            id=f"inv-{uuid.uuid4().hex[:16]}",
            wire=wire,
            scope=scope,
            project_id=project_id,
            project_name=project_name,
            expires_at=expires_at,
            max_uses=max_uses,
            uses_count=0,
            revoked_at=None,
            note=note,
            created_at=now,
        )

        async with aiosqlite.connect(self._db_path) as db:
            await db.execute(
                """
                INSERT INTO invites (
                    id, wire, scope, project_id, project_name, expires_at,
                    max_uses, uses_count, revoked_at, note, created_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, NULL, ?, ?)
                """,
                (
                    record.id,
                    record.wire,
                    record.scope,
                    record.project_id,
                    record.project_name,
                    record.expires_at,
                    record.max_uses,
                    record.note,
                    record.created_at,
                ),
            )
            await db.commit()

        _log.info(
            "invite minted",
            invite_id=record.id,
            scope=scope,
            project_id=project_id,
            expires_at=expires_at,
        )
        return record

    async def revoke(self, invite_id: str) -> bool:
        """Mark an invite as revoked. Returns True if the row
        existed and was not already revoked."""
        now = int(time.time())
        async with aiosqlite.connect(self._db_path) as db:
            cursor = await db.execute(
                "UPDATE invites SET revoked_at = ? WHERE id = ? AND revoked_at IS NULL",
                (now, invite_id),
            )
            affected = cursor.rowcount
            await cursor.close()
            await db.commit()
        return bool(affected)

    async def list_invites(self, limit: int = 100) -> list[InviteRecord]:
        async with aiosqlite.connect(self._db_path) as db:
            async with db.execute(
                """
                SELECT id, wire, scope, project_id, project_name, expires_at,
                       max_uses, uses_count, revoked_at, note, created_at
                FROM invites ORDER BY created_at DESC LIMIT ?
                """,
                (limit,),
            ) as cursor:
                rows = await cursor.fetchall()
        return [
            InviteRecord(
                id=row[0],
                wire=row[1],
                scope=row[2],
                project_id=row[3],
                project_name=row[4],
                expires_at=row[5],
                max_uses=row[6],
                uses_count=row[7],
                revoked_at=row[8],
                note=row[9],
                created_at=row[10],
            )
            for row in rows
        ]

    async def get(self, invite_id: str) -> InviteRecord | None:
        rows = await self.list_invites(limit=10_000)
        return next((r for r in rows if r.id == invite_id), None)

    @staticmethod
    def decode(wire: str, *, now_unix: int | None = None) -> dict[str, Any]:
        """Parse an ``nx1...`` string, verify signature + expiry,
        and return the payload fields as a plain dict."""
        return nexus_core.decode_invite(wire, now_unix)
