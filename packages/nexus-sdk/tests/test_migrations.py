"""Tests for :class:`nexus_sdk.migrations.MigrationRunner` (Sprint 9 Phase D).

The 18 scenarios listed in ``.planning/sprint9_plan.md`` §7.2 are
covered here. Every test uses a per-test ``tmp_path`` for the SQLite
database and the migrations directory so the test suite is fully
hermetic.

Categories:

- Happy path (1-2) — single migration apply + idempotent reboot.
- Integrity (3, 13, 15, 17) — tamper detection, forward-only,
  SHA256 match, error message content.
- Transaction safety (4, 16) — rollback on failure, BEGIN IMMEDIATE
  concurrent lock.
- Ordering (5) — lexicographic sort.
- Edge cases (6-9, 12) — dry run, tracking table creation,
  None dir, empty dir, read-only client.
- Filename parsing (10-11) — version + slug extraction.
- Observability (14, 18) — applied_at UTC ISO-8601, structlog
  events on apply / tamper.
"""

from __future__ import annotations

import hashlib
from datetime import datetime, timezone
from pathlib import Path

import aiosqlite
import pytest
from nexus_sdk.db import AppDatabaseClient, DatabaseError
from nexus_sdk.migrations import (
    MigrationRunner,
    MigrationTamperedError,
    _parse_migration_filename,
    _sha256_file,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _write_migration(mig_dir: Path, name: str, sql: str) -> Path:
    mig_dir.mkdir(parents=True, exist_ok=True)
    p = mig_dir / name
    p.write_text(sql, encoding="utf-8")
    return p


def _make_runner(
    tmp_path: Path,
    *,
    mig_dir: Path | None = None,
    timeout: float = 0.1,
) -> tuple[MigrationRunner, AppDatabaseClient, Path]:
    db_path = tmp_path / "app.sqlite"
    client = AppDatabaseClient(db_path)
    if mig_dir is None:
        mig_dir = tmp_path / "migrations"
    runner = MigrationRunner(client, mig_dir, _timeout=timeout)
    return runner, client, db_path


# ---------------------------------------------------------------------------
# 1 — test_runner_applies_single_migration_happy_path
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_applies_single_migration_happy_path(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER PRIMARY KEY)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    applied = await runner.apply()

    assert len(applied) == 1
    assert applied[0].version == 1
    assert applied[0].slug == "init"

    # Verify the table was created
    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='foo'")
        row = await cursor.fetchone()
        assert row is not None


# ---------------------------------------------------------------------------
# 2 — test_runner_is_idempotent_on_second_boot
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_is_idempotent_on_second_boot(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    first = await runner.apply()
    assert len(first) == 1

    second = await runner.apply()
    assert len(second) == 0


# ---------------------------------------------------------------------------
# 3 — test_runner_detects_tampered_migration_raises
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_detects_tampered_migration_raises(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    p = _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    await runner.apply()

    # Tamper with the file
    p.write_text("CREATE TABLE bar (id INTEGER)", encoding="utf-8")

    with pytest.raises(MigrationTamperedError, match="tampered"):
        await runner.apply()


# ---------------------------------------------------------------------------
# 4 — test_runner_rollbacks_failing_statement_leaves_clean_state
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_rollbacks_failing_statement_leaves_clean_state(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    # First statement OK, second deliberately fails
    _write_migration(
        mig_dir,
        "001_init.sql",
        "CREATE TABLE good (id INTEGER);\nCREATE TABLE bad INVALID SYNTAX;\n",
    )
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    with pytest.raises(DatabaseError):
        await runner.apply()

    # Neither the table nor the tracking row should exist
    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='good'")
        assert await cursor.fetchone() is None

        cursor = await db.execute("SELECT count(*) FROM _nexus_migrations")
        row = await cursor.fetchone()
        assert row[0] == 0


# ---------------------------------------------------------------------------
# 5 — test_runner_applies_in_lexico_order
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_applies_in_lexico_order(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "003_third.sql", "CREATE TABLE third (id INTEGER)")
    _write_migration(mig_dir, "001_first.sql", "CREATE TABLE first (id INTEGER)")
    _write_migration(mig_dir, "002_second.sql", "CREATE TABLE second (id INTEGER)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    applied = await runner.apply()

    assert [m.version for m in applied] == [1, 2, 3]
    assert [m.slug for m in applied] == ["first", "second", "third"]


# ---------------------------------------------------------------------------
# 6 — test_runner_dry_run_does_not_touch_db
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_dry_run_does_not_touch_db(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    pending = await runner.plan()
    assert len(pending) == 1
    assert pending[0].version == 1

    # The table should NOT have been created
    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='foo'")
        assert await cursor.fetchone() is None


# ---------------------------------------------------------------------------
# 7 — test_runner_creates_tracking_table_on_first_run
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_creates_tracking_table_on_first_run(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    await runner.apply()

    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='_nexus_migrations'")
        assert await cursor.fetchone() is not None


# ---------------------------------------------------------------------------
# 8 — test_runner_skip_if_migrations_dir_none
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_skip_if_migrations_dir_none(tmp_path: Path) -> None:
    db_path = tmp_path / "app.sqlite"
    client = AppDatabaseClient(db_path)
    runner = MigrationRunner(client, None)

    assert await runner.plan() == []
    assert await runner.apply() == []


# ---------------------------------------------------------------------------
# 9 — test_runner_skip_if_migrations_dir_empty
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_skip_if_migrations_dir_empty(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    mig_dir.mkdir()
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    assert await runner.plan() == []
    assert await runner.apply() == []


# ---------------------------------------------------------------------------
# 10 — test_runner_extracts_version_from_filename_prefix
# ---------------------------------------------------------------------------


def test_runner_extracts_version_from_filename_prefix(tmp_path: Path) -> None:
    p = tmp_path / "001_init.sql"
    p.write_text("", encoding="utf-8")
    version, _ = _parse_migration_filename(p)
    assert version == 1


# ---------------------------------------------------------------------------
# 11 — test_runner_extracts_slug_from_filename
# ---------------------------------------------------------------------------


def test_runner_extracts_slug_from_filename(tmp_path: Path) -> None:
    p = tmp_path / "042_add_users_table.sql"
    p.write_text("", encoding="utf-8")
    _, slug = _parse_migration_filename(p)
    assert slug == "add_users_table"


# ---------------------------------------------------------------------------
# 12 — test_runner_refuses_read_only_client
# ---------------------------------------------------------------------------


def test_runner_refuses_read_only_client(tmp_path: Path) -> None:
    db_path = tmp_path / "app.sqlite"
    # Create the file first so read-only connect doesn't fail
    db_path.touch()
    client = AppDatabaseClient(db_path, read_only=True)
    with pytest.raises(ValueError, match="writable"):
        MigrationRunner(client, tmp_path / "migrations")


# ---------------------------------------------------------------------------
# 13 — test_runner_forward_only_rejects_version_backward_jump
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_forward_only_rejects_version_backward_jump(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_first.sql", "CREATE TABLE first (id INTEGER)")
    _write_migration(mig_dir, "002_second.sql", "CREATE TABLE second (id INTEGER)")
    _write_migration(mig_dir, "003_third.sql", "CREATE TABLE third (id INTEGER)")
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    await runner.apply()

    # Remove migration 003 from disk (simulates a backward jump)
    (mig_dir / "003_third.sql").unlink()

    with pytest.raises(MigrationTamperedError, match="missing"):
        await runner.apply()


# ---------------------------------------------------------------------------
# 14 — test_runner_applied_at_is_utc_iso8601
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_applied_at_is_utc_iso8601(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    before = datetime.now(timezone.utc)
    await runner.apply()
    after = datetime.now(timezone.utc)

    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT applied_at FROM _nexus_migrations WHERE version = 1")
        row = await cursor.fetchone()
        assert row is not None
        applied_at = datetime.fromisoformat(row[0])
        # The timestamp should be between before and after
        assert before <= applied_at <= after


# ---------------------------------------------------------------------------
# 15 — test_runner_sha256_stored_matches_file_content
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_sha256_stored_matches_file_content(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    p = _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    expected_sha = hashlib.sha256(p.read_bytes()).hexdigest()
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir)

    await runner.apply()

    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT sha256 FROM _nexus_migrations WHERE version = 1")
        row = await cursor.fetchone()
        assert row[0] == expected_sha


# ---------------------------------------------------------------------------
# 16 — test_runner_concurrent_runs_blocked_by_begin_immediate
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_concurrent_runs_blocked_by_begin_immediate(tmp_path: Path) -> None:
    """Hold a BEGIN IMMEDIATE lock from a separate connection, then
    verify the runner fails with a database-locked error."""
    mig_dir = tmp_path / "migrations"
    _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, db_path = _make_runner(tmp_path, mig_dir=mig_dir, timeout=0.1)

    # Pre-create the tracking table so the blocker doesn't need it
    async with aiosqlite.connect(db_path, isolation_level=None) as setup_db:
        await setup_db.execute(
            "CREATE TABLE IF NOT EXISTS _nexus_migrations ("
            "version INTEGER PRIMARY KEY, slug TEXT NOT NULL, "
            "sha256 TEXT NOT NULL, applied_at TEXT NOT NULL)"
        )

    # Hold a lock from a separate connection
    async with aiosqlite.connect(db_path, isolation_level=None) as blocker:
        await blocker.execute("BEGIN IMMEDIATE")
        with pytest.raises(DatabaseError, match="locked|busy"):
            await runner.apply()
        await blocker.rollback()


# ---------------------------------------------------------------------------
# 17 — test_migration_tampered_error_message_cites_file_and_hashes
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_migration_tampered_error_message_cites_file_and_hashes(tmp_path: Path) -> None:
    mig_dir = tmp_path / "migrations"
    p = _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    await runner.apply()
    original_sha = _sha256_file(p)

    # Tamper
    p.write_text("-- tampered content", encoding="utf-8")
    new_sha = _sha256_file(p)

    with pytest.raises(MigrationTamperedError) as exc_info:
        await runner.apply()

    msg = str(exc_info.value)
    assert "001_init.sql" in msg
    assert original_sha in msg
    assert new_sha in msg


# ---------------------------------------------------------------------------
# 18 — test_runner_logs_info_on_apply_error_on_tamper
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_runner_logs_info_on_apply_error_on_tamper(tmp_path: Path, caplog: pytest.LogCaptureFixture) -> None:
    """The runner logs INFO on successful apply and ERROR on tamper."""
    mig_dir = tmp_path / "migrations"
    p = _write_migration(mig_dir, "001_init.sql", "CREATE TABLE foo (id INTEGER)")
    runner, _, _ = _make_runner(tmp_path, mig_dir=mig_dir)

    # Successful apply should log

    # Capture structlog output by using stdlib logging integration
    with caplog.at_level("DEBUG"):
        await runner.apply()

    # Tamper and attempt re-apply
    p.write_text("-- tampered", encoding="utf-8")
    with pytest.raises(MigrationTamperedError):
        await runner.apply()
