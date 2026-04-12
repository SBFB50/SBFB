# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for :class:`nexus_sdk.db.AppDatabaseClient`.

The client is a thin aiosqlite wrapper and these tests exercise
its contract on a per-test temp SQLite file. All eight tests
cover the surface listed in ``.planning/sprint8_plan.md`` §5.1:
init roundtrip, dict row shape, fetchone None, execute insert,
parameterized queries, missing file behaviour, concurrent
fetchall, and DatabaseError wrapping of bad SQL.
"""

from __future__ import annotations

import asyncio
import sqlite3
from pathlib import Path

import pytest
from nexus_sdk.db import AppDatabaseClient, DatabaseError


def _seed_schema(db_path: Path) -> None:
    """Populate a temp SQLite with a two-row ``t`` table.

    Uses the synchronous :mod:`sqlite3` stdlib so the fixture
    is deterministic and independent of the aiosqlite event
    loop — we only verify the async wrapper by reading what
    we just wrote.
    """
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript(
            """
            CREATE TABLE t (id INTEGER PRIMARY KEY, name TEXT NOT NULL, score INTEGER);
            INSERT INTO t (id, name, score) VALUES (1, 'alice', 42);
            INSERT INTO t (id, name, score) VALUES (2, 'bob', 7);
            """
        )
        conn.commit()
    finally:
        conn.close()


@pytest.mark.asyncio
async def test_db_path_roundtrip(tmp_path: Path) -> None:
    """The ``db_path`` property returns the Path passed in
    construction — trivial but documents the public surface."""
    db_file = tmp_path / "roundtrip.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    assert client.db_path == db_file


@pytest.mark.asyncio
async def test_fetchall_returns_dict_rows(tmp_path: Path) -> None:
    """``fetchall`` must produce a list of ``dict[str, Any]``
    keyed by column name so tab handlers can pass the rows
    straight into TabView helpers."""
    db_file = tmp_path / "dict.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    rows = await client.fetchall("SELECT id, name, score FROM t ORDER BY id")
    assert rows == [
        {"id": 1, "name": "alice", "score": 42},
        {"id": 2, "name": "bob", "score": 7},
    ]


@pytest.mark.asyncio
async def test_fetchone_match_and_none(tmp_path: Path) -> None:
    """``fetchone`` returns a dict on match and ``None`` on miss.

    Covers both branches in one test so callers can trust the
    single-row-or-nothing contract documented on the method.
    """
    db_file = tmp_path / "fetchone.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)

    hit = await client.fetchone("SELECT * FROM t WHERE name = ?", ("alice",))
    assert hit == {"id": 1, "name": "alice", "score": 42}

    miss = await client.fetchone("SELECT * FROM t WHERE name = ?", ("carol",))
    assert miss is None


@pytest.mark.asyncio
async def test_execute_commits_and_persists(tmp_path: Path) -> None:
    """``execute`` must commit — a subsequent ``fetchall`` from a
    fresh connection sees the insert. Regression against a prior
    bug where we forgot ``await db.commit()``."""
    db_file = tmp_path / "execute.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    await client.execute(
        "INSERT INTO t (id, name, score) VALUES (?, ?, ?)",
        (3, "carol", 99),
    )
    rows = await client.fetchall("SELECT name FROM t ORDER BY id")
    assert [r["name"] for r in rows] == ["alice", "bob", "carol"]


@pytest.mark.asyncio
async def test_parameterized_query_binds_safely(tmp_path: Path) -> None:
    """A value containing SQL punctuation must be bound as a
    parameter, not spliced into the statement. If the binding is
    broken, the query either raises or matches the wrong row —
    neither is acceptable."""
    db_file = tmp_path / "inject.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    # A literal value that would be dangerous if interpolated into
    # the SQL string as plain text — it must round-trip as a bound
    # parameter and match zero rows.
    evil = "alice'; DROP TABLE t; --"
    rows = await client.fetchall("SELECT * FROM t WHERE name = ?", (evil,))
    assert rows == []
    # Verify the table still exists and the original rows are
    # intact — the "DROP TABLE" fragment was bound as data, not
    # interpreted as SQL.
    check = await client.fetchall("SELECT COUNT(*) AS n FROM t")
    assert check == [{"n": 2}]


@pytest.mark.asyncio
async def test_missing_file_raises_on_unknown_table(tmp_path: Path) -> None:
    """Querying against a non-existent SQLite file creates an
    empty database at that path; the query then fails with
    ``no such table`` wrapped as :class:`DatabaseError`.

    Tab handlers guard this case and render an empty state — the
    test pins the observable contract.
    """
    db_file = tmp_path / "missing.sqlite"
    assert not db_file.exists()
    client = AppDatabaseClient(db_file)
    with pytest.raises(DatabaseError) as excinfo:
        await client.fetchall("SELECT * FROM never_existed")
    # aiosqlite materialises an empty file on first connect
    assert db_file.exists()
    # The underlying cause must still be a sqlite3 operational
    # error — preserved via __cause__ so debugging stays trivial.
    assert isinstance(excinfo.value.__cause__, sqlite3.Error)


@pytest.mark.asyncio
async def test_concurrent_fetchall_is_safe(tmp_path: Path) -> None:
    """Multiple concurrent ``fetchall`` calls share no state — the
    per-request connect pattern means each call runs in its own
    aiosqlite session, and ``asyncio.gather`` over ten of them
    must all succeed with identical results."""
    db_file = tmp_path / "concurrent.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    results = await asyncio.gather(*(client.fetchall("SELECT COUNT(*) AS n FROM t") for _ in range(10)))
    assert len(results) == 10
    assert all(row == [{"n": 2}] for row in results)


@pytest.mark.asyncio
async def test_bad_sql_raises_database_error(tmp_path: Path) -> None:
    """Malformed SQL is wrapped as :class:`DatabaseError` with the
    original ``sqlite3.Error`` preserved on ``__cause__``."""
    db_file = tmp_path / "bad_sql.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file)
    with pytest.raises(DatabaseError):
        await client.fetchall("SELECT ** FROM t WHERE not valid sql")


# ---------------------------------------------------------------------------
# Sprint 9 Phase 0 audit gate (D-FX-1) — read-only enforcement
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_readonly_blocks_execute(tmp_path: Path) -> None:
    """A client constructed with ``read_only=True`` rejects every
    :meth:`AppDatabaseClient.execute` call at the Python layer
    BEFORE opening any connection.

    Defense-in-depth: even if a future SQLite version loosened the
    ``mode=ro`` URI semantics, this short-circuit still keeps
    writes from reaching the legacy ``nexus/gov/govdata.db`` file.
    """
    db_file = tmp_path / "readonly_execute.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file, read_only=True)
    assert client.read_only is True

    with pytest.raises(DatabaseError) as excinfo:
        await client.execute(
            "INSERT INTO t (id, name, score) VALUES (?, ?, ?)",
            (3, "carol", 99),
        )
    assert "read-only" in str(excinfo.value)

    # The original two rows must still be intact — the execute()
    # call short-circuited before SQLite ever saw the statement.
    rows = await client.fetchall("SELECT id FROM t ORDER BY id")
    assert [r["id"] for r in rows] == [1, 2]


@pytest.mark.asyncio
async def test_readonly_uri_blocks_kernel_level_writes(tmp_path: Path) -> None:
    """A client constructed with ``read_only=True`` opens its
    connection in SQLite ``mode=ro``, so even a write smuggled
    through ``fetchall`` is rejected at the SQLite kernel level
    with ``OperationalError`` wrapped as :class:`DatabaseError`.

    This is the second layer of the defense-in-depth: the
    Python-side guard in :meth:`execute` could be bypassed by a
    caller that hand-crafts an ``INSERT`` and routes it through
    ``fetchall``, but the URI mode means SQLite itself refuses.
    """
    db_file = tmp_path / "readonly_uri.sqlite"
    _seed_schema(db_file)
    client = AppDatabaseClient(db_file, read_only=True)

    with pytest.raises(DatabaseError):
        # fetchall is the read path — feeding it an INSERT is
        # nonsensical but technically possible. The mode=ro URI
        # makes SQLite reject it before the cursor materialises.
        await client.fetchall(
            "INSERT INTO t (id, name, score) VALUES (?, ?, ?)",
            (4, "dave", 17),
        )

    # The original two rows must still be intact.
    rows = await client.fetchall("SELECT id FROM t ORDER BY id")
    assert [r["id"] for r in rows] == [1, 2]


@pytest.mark.asyncio
async def test_readonly_refuses_missing_file(tmp_path: Path) -> None:
    """``read_only=True`` on a non-existent file raises
    :class:`DatabaseError` because SQLite ``mode=ro`` refuses to
    materialise an empty database.

    Pinned because the writable default DOES create the file
    (`test_missing_file_raises_on_unknown_table` above) — the two
    behaviours are intentionally different and the read-only
    contract is the safer one for the legacy gov DB use case.
    """
    db_file = tmp_path / "readonly_missing.sqlite"
    assert not db_file.exists()
    client = AppDatabaseClient(db_file, read_only=True)
    with pytest.raises(DatabaseError):
        await client.fetchall("SELECT 1")
    # The file must still NOT exist — mode=ro must not create it.
    assert not db_file.exists()
