"""MigrationRunner — forward-only SQL migration runner with SHA256 tamper detection.

Sprint 9 Phase D (D4 impl). Every app that needs a mutable schema
ships SQL files under ``<app_package>/migrations/`` and declares
``migrations_dir`` on its :class:`nexus_sdk.AppManifest`. The
coordinator runs the runner at boot after each app's ``on_start``
hook, before the dispatcher starts accepting tasks.

Design decisions (frozen in sprint9_kickoff.md §4 D4):

- Files scanned in lexicographic order (``001_init.sql`` <
  ``002_foo.sql``). No timestamp prefix, no DAG.
- Each migration applied in a ``BEGIN IMMEDIATE`` transaction on
  the target SQLite.
- SHA256 of the file content stored at apply time in
  ``_nexus_migrations.sha256``. On subsequent boots the runner
  re-hashes every applied migration and raises
  :class:`MigrationTamperedError` on mismatch.
- Forward-only: no down-migration. Rollback pattern is
  ``git revert`` + a new migration that undoes the effect.
- Tracking table: ``_nexus_migrations(version INT PRIMARY KEY,
  slug TEXT, sha256 TEXT, applied_at TEXT)``.
"""

from __future__ import annotations

import hashlib
import sqlite3
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

import aiosqlite
import structlog

from nexus_sdk.db import AppDatabaseClient, DatabaseError

_log = structlog.get_logger(__name__)

_TRACKING_TABLE = "_nexus_migrations"


class MigrationTamperedError(Exception):
    """Raised when a previously-applied migration file's SHA256
    no longer matches the hash recorded at apply time.

    The message cites both the file path and the expected vs actual
    hashes so an operator can diff and understand the divergence.
    """


@dataclass(frozen=True)
class PendingMigration:
    """One migration file discovered on disk."""

    version: int
    slug: str
    path: Path
    sha256: str


def _parse_migration_filename(path: Path) -> tuple[int, str]:
    """Extract ``(version, slug)`` from a migration filename.

    Expected pattern: ``NNN_slug.sql`` where NNN is a zero-padded
    integer prefix and slug is the descriptive remainder.
    """
    stem = path.stem  # e.g. "001_documents"
    parts = stem.split("_", 1)
    version = int(parts[0])
    slug = parts[1] if len(parts) > 1 else ""
    return version, slug


def _sha256_file(path: Path) -> str:
    """Compute SHA256 hex digest of a file's content."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


class MigrationRunner:
    """Scan, verify, and apply SQL migrations for an app's database.

    Parameters
    ----------
    client:
        A writable :class:`AppDatabaseClient`. The runner uses its
        ``db_path`` to open raw aiosqlite connections with explicit
        transaction control (``BEGIN IMMEDIATE``). Raises
        ``ValueError`` if the client is read-only.
    migrations_dir:
        Directory containing ``NNN_slug.sql`` files. If ``None``,
        the directory does not exist, or is empty, :meth:`plan`
        and :meth:`apply` return empty lists.
    """

    def __init__(
        self,
        client: AppDatabaseClient,
        migrations_dir: Path | None,
        *,
        _timeout: float = 5.0,
    ) -> None:
        if client.read_only:
            raise ValueError("migrations runner requires a writable client")
        self._db_path = client.db_path
        self._migrations_dir = migrations_dir
        self._timeout = _timeout

    # ------------------------------------------------------------------
    # Scanning
    # ------------------------------------------------------------------

    def _scan_files(self) -> list[PendingMigration]:
        """Return all migration files sorted by lexicographic order."""
        if self._migrations_dir is None:
            return []
        if not self._migrations_dir.exists() or not self._migrations_dir.is_dir():
            return []
        files = sorted(self._migrations_dir.glob("*.sql"))
        result: list[PendingMigration] = []
        for f in files:
            version, slug = _parse_migration_filename(f)
            sha = _sha256_file(f)
            result.append(PendingMigration(version=version, slug=slug, path=f, sha256=sha))
        return result

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _connect(self) -> aiosqlite.Connection:
        """Open a raw aiosqlite connection with manual transaction
        control (``isolation_level=None``) so we can issue
        ``BEGIN IMMEDIATE`` explicitly."""
        return aiosqlite.connect(
            self._db_path,
            timeout=self._timeout,
            isolation_level=None,
        )

    @staticmethod
    async def _ensure_tracking_table(db: aiosqlite.Connection) -> None:
        await db.execute(
            f"CREATE TABLE IF NOT EXISTS {_TRACKING_TABLE} ("
            "version INTEGER PRIMARY KEY, "
            "slug TEXT NOT NULL, "
            "sha256 TEXT NOT NULL, "
            "applied_at TEXT NOT NULL"
            ")"
        )

    @staticmethod
    async def _get_applied(
        db: aiosqlite.Connection,
    ) -> dict[int, dict[str, str]]:
        cursor = await db.execute(f"SELECT version, slug, sha256, applied_at FROM {_TRACKING_TABLE} ORDER BY version")
        rows = await cursor.fetchall()
        return {row[0]: {"slug": row[1], "sha256": row[2], "applied_at": row[3]} for row in rows}

    async def _verify_integrity(
        self,
        all_files: list[PendingMigration],
        applied: dict[int, dict[str, str]],
    ) -> None:
        """Check SHA256 of every applied migration against current
        files on disk. Raises :class:`MigrationTamperedError` on
        mismatch or if a previously-applied file is missing."""
        file_by_version = {m.version: m for m in all_files}
        for version, info in applied.items():
            if version not in file_by_version:
                raise MigrationTamperedError(
                    f"Previously applied migration version {version} "
                    f"(slug: {info['slug']}) is missing from "
                    f"{self._migrations_dir}"
                )
            current_sha = file_by_version[version].sha256
            if current_sha != info["sha256"]:
                _log.error(
                    "migration tampered",
                    version=version,
                    file=file_by_version[version].path.name,
                    expected_sha256=info["sha256"],
                    actual_sha256=current_sha,
                )
                raise MigrationTamperedError(
                    f"Migration {file_by_version[version].path.name} has been "
                    f"tampered with: expected SHA256 {info['sha256']}, "
                    f"got {current_sha}"
                )

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    async def plan(self) -> list[PendingMigration]:
        """Return pending migrations without applying.

        Also verifies the integrity of already-applied migrations.
        Raises :class:`MigrationTamperedError` if any applied
        migration's SHA256 no longer matches.
        """
        all_files = self._scan_files()
        if not all_files:
            return []
        async with self._connect() as db:
            await self._ensure_tracking_table(db)
            applied = await self._get_applied(db)
            await self._verify_integrity(all_files, applied)
            return [m for m in all_files if m.version not in applied]

    async def apply(self) -> list[PendingMigration]:
        """Apply all pending migrations in lexicographic order.

        Each migration runs in a ``BEGIN IMMEDIATE`` transaction.
        If a migration fails, the transaction is rolled back and
        the error is propagated — no partial application.

        Returns the list of successfully applied migrations.
        """
        all_files = self._scan_files()
        if not all_files:
            return []

        applied_list: list[PendingMigration] = []
        async with self._connect() as db:
            await self._ensure_tracking_table(db)
            applied = await self._get_applied(db)
            await self._verify_integrity(all_files, applied)

            pending = [m for m in all_files if m.version not in applied]
            for m in pending:
                sql_text = m.path.read_text(encoding="utf-8")
                statements = [s.strip() for s in sql_text.split(";") if s.strip()]
                now = datetime.now(timezone.utc).isoformat()

                try:
                    await db.execute("BEGIN IMMEDIATE")
                    for stmt in statements:
                        await db.execute(stmt)
                    await db.execute(
                        f"INSERT INTO {_TRACKING_TABLE} (version, slug, sha256, applied_at) VALUES (?, ?, ?, ?)",
                        (m.version, m.slug, m.sha256, now),
                    )
                    await db.commit()
                    applied_list.append(m)
                    _log.info(
                        "migration applied",
                        version=m.version,
                        slug=m.slug,
                        file=m.path.name,
                    )
                except sqlite3.Error as exc:
                    try:
                        await db.rollback()
                    except sqlite3.Error:
                        pass
                    _log.error(
                        "migration failed",
                        version=m.version,
                        slug=m.slug,
                        error=str(exc),
                    )
                    raise DatabaseError(f"migration {m.path.name} failed: {exc}") from exc

        return applied_list


__all__ = ["MigrationRunner", "MigrationTamperedError", "PendingMigration"]
