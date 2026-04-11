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
        Path to the SQLite file. May or may not exist — aiosqlite
        creates an empty file on first connect if the parent
        directory is writable. Tab handlers are expected to guard
        their SQL with try/except around :class:`DatabaseError` so
        queries against an empty or missing schema fall back to
        empty-state rendering instead of surfacing a 500 to the
        shell.

    Notes
    -----
    The coordinator's loader constructs one instance per app at
    boot and assigns it to :attr:`AppContext.db`. Apps may swap
    the attribute in their ``on_start`` hook to point at a
    different path — that is how ``nexus-app-gov`` redirects to
    the legacy ``nexus/gov/govdata.db``.
    """

    def __init__(self, db_path: Path | str) -> None:
        self._db_path = Path(db_path)

    @property
    def db_path(self) -> Path:
        """Return the SQLite file this client reads from."""
        return self._db_path

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
            async with aiosqlite.connect(self._db_path) as db:
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
            async with aiosqlite.connect(self._db_path) as db:
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
        """
        try:
            async with aiosqlite.connect(self._db_path) as db:
                await db.execute(query, tuple(params or ()))
                await db.commit()
        except sqlite3.Error as e:
            raise DatabaseError(f"execute failed: {e}") from e


__all__ = ["AppDatabaseClient", "DatabaseError"]
