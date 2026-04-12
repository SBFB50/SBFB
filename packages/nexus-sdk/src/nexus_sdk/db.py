# SPDX-License-Identifier: AGPL-3.0-or-later
"""AppDatabaseClient — aiosqlite wrapper for per-app SQLite access.

Sprint 8 Phase B (D3 impl): apps reach their private SQLite file
through an :class:`AppDatabaseClient` instance wired on
``AppContext.db`` by the coordinator loader at boot. An app can
override the default path in its ``on_start`` hook — that is how
``nexus-app-gov`` points at the legacy ``nexus/gov/govdata.db``
instead of the fresh per-app file under the coordinator tree.

Connection model: every call opens a fresh ``aiosqlite.connect``,
runs the statement, and closes on exit. This mirrors the existing
pattern in :mod:`nexus_coordinator.kudos`,
:mod:`nexus_coordinator.invite`, :mod:`nexus_coordinator.dispatcher`
— none of them cache connections either. The read-heavy Batch 1
workload (six gov tab handlers querying on descriptor requests)
sees tens of milliseconds of SQLite open/close overhead per call,
which is negligible against the FastAPI round-trip and sidesteps
the lifecycle / locking complexity of a cached connection.

Error surface: every query wraps ``sqlite3.Error`` in a
dedicated :class:`DatabaseError` so call sites can match a single
exception type without importing aiosqlite themselves. Tab
handlers that run against an empty or missing schema typically
catch :class:`DatabaseError` and render an empty TabView state
rather than propagating a 500.

Read-only safety (Sprint 9 Phase 0 audit gate D-FX-1)
-----------------------------------------------------

The default ctor is writable (``read_only=False``) so an app can
own its per-app SQLite under the coordinator tree without extra
ceremony. When the app swaps the client to point at a precious
external database — the canonical case is ``nexus-app-gov``
redirecting to the legacy ``nexus/gov/govdata.db`` (4 years of
scraping data) — it MUST instantiate the override with
``read_only=True``. The flag activates two layers of protection:

1. The connection is opened via the SQLite URI form
   ``file:<path>?mode=ro`` which makes the underlying connection
   refuse any write at the kernel level — even a hand-crafted
   ``UPDATE`` issued via ``fetchall`` (which never makes sense
   semantically but is technically possible) would fail with
   ``sqlite3.OperationalError: attempt to write a readonly database``.
2. :meth:`execute` short-circuits at the Python layer with a
   :class:`DatabaseError` BEFORE opening any connection. This is
   defense-in-depth: even if a future SQLite version loosened
   the URI semantics, the wrapper still refuses.

Sprint 8 originally shipped without this distinction; the audit
gate at the start of Sprint 9 caught the gap and required the fix
before Phase A could open.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path
from typing import Any, Sequence

import aiosqlite


class DatabaseError(Exception):
    """Raised by :class:`AppDatabaseClient` on SQL errors.

    Wraps the underlying ``sqlite3.Error`` (re-exported by
    aiosqlite as ``aiosqlite.Error``) so tab handlers can catch
    a single exception type without pulling in the aiosqlite
    import themselves.
    """


class AppDatabaseClient:
    """Thin aiosqlite wrapper with dict-based row access.

    Parameters
    ----------
    db_path:
        Path to the SQLite file. May or may not exist — when
        ``read_only=False`` aiosqlite creates an empty file on
        first connect if the parent directory is writable. When
        ``read_only=True`` the file MUST exist already; opening a
        missing path with ``mode=ro`` raises
        :class:`DatabaseError` because SQLite refuses to
        materialise a file under read-only mode. Tab handlers are
        expected to guard their SQL with try/except around
        :class:`DatabaseError` so queries against an empty or
        missing schema fall back to empty-state rendering instead
        of surfacing a 500 to the shell.
    read_only:
        When ``True`` the connection is opened in SQLite ``mode=ro``
        and :meth:`execute` short-circuits to a
        :class:`DatabaseError`. The default is ``False`` so an app
        owning a per-app SQLite under the coordinator tree can
        write freely; an app pointing at a precious external
        database (canonical case: ``nexus-app-gov`` redirecting to
        ``nexus/gov/govdata.db``) MUST opt in to the read-only
        guard.

    Notes
    -----
    The coordinator's loader constructs one instance per app at
    boot and assigns it to :attr:`AppContext.db`. Apps may swap
    the attribute in their ``on_start`` hook to point at a
    different path — that is how ``nexus-app-gov`` redirects to
    the legacy ``nexus/gov/govdata.db``, with ``read_only=True``
    to protect the legacy data.
    """

    def __init__(self, db_path: Path | str, *, read_only: bool = False) -> None:
        self._db_path = Path(db_path)
        self._read_only = read_only

    @property
    def db_path(self) -> Path:
        """Return the SQLite file this client reads from."""
        return self._db_path

    @property
    def read_only(self) -> bool:
        """Return whether this client refuses writes (Sprint 9
        Phase 0 audit gate D-FX-1)."""
        return self._read_only

    def _connect(self) -> aiosqlite.Connection:
        """Return an aiosqlite connection honouring the read-only
        flag.

        When ``read_only`` is true the connection is opened via
        the SQLite URI form ``file:<path>?mode=ro`` so even a
        hand-crafted ``UPDATE`` issued through ``fetchall`` is
        rejected at the SQLite layer.
        """
        if self._read_only:
            uri = f"file:{self._db_path.as_posix()}?mode=ro"
            return aiosqlite.connect(uri, uri=True)
        return aiosqlite.connect(self._db_path)

    async def fetchall(
        self,
        query: str,
        params: Sequence[Any] | None = None,
    ) -> list[dict[str, Any]]:
        """Run ``query`` and return every matched row as a dict.

        Returns an empty list when the query matches no rows.
        Raises :class:`DatabaseError` on any SQL error, including
        missing tables (``no such table`` wraps cleanly).
        """
        try:
            async with self._connect() as db:
                db.row_factory = aiosqlite.Row
                async with db.execute(query, tuple(params or ())) as cursor:
                    rows = await cursor.fetchall()
                    return [dict(row) for row in rows]
        except sqlite3.Error as e:
            raise DatabaseError(f"fetchall failed: {e}") from e

    async def fetchone(
        self,
        query: str,
        params: Sequence[Any] | None = None,
    ) -> dict[str, Any] | None:
        """Run ``query`` and return the first matched row as a
        dict, or ``None`` if no row matched.

        Raises :class:`DatabaseError` on any SQL error.
        """
        try:
            async with self._connect() as db:
                db.row_factory = aiosqlite.Row
                async with db.execute(query, tuple(params or ())) as cursor:
                    row = await cursor.fetchone()
                    return dict(row) if row is not None else None
        except sqlite3.Error as e:
            raise DatabaseError(f"fetchone failed: {e}") from e

    async def execute(
        self,
        query: str,
        params: Sequence[Any] | None = None,
    ) -> None:
        """Run a mutating statement (INSERT / UPDATE / DELETE / DDL).

        Commits on success. Raises :class:`DatabaseError` on any
        SQL error. Sprint 8 Phase B does not ship a migration
        runner — apps that need a schema bootstrap issue their
        ``CREATE TABLE IF NOT EXISTS`` statements directly here
        from their ``on_start`` hook.

        Read-only short-circuit (Sprint 9 Phase 0 audit gate
        D-FX-1): when the client was instantiated with
        ``read_only=True`` this method raises
        :class:`DatabaseError` BEFORE opening any connection. This
        is defense-in-depth on top of the SQLite ``mode=ro`` URI
        used by ``_connect`` — even if a future SQLite version
        loosened the URI semantics, the wrapper still refuses.
        """
        if self._read_only:
            raise DatabaseError(
                "AppDatabaseClient is read-only: refusing execute() "
                f"on {self._db_path.as_posix()!r}. Pass read_only=False "
                "at construction time if writes are intended."
            )
        try:
            async with self._connect() as db:
                await db.execute(query, tuple(params or ()))
                await db.commit()
        except sqlite3.Error as e:
            raise DatabaseError(f"execute failed: {e}") from e


__all__ = ["AppDatabaseClient", "DatabaseError"]
