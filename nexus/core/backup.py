"""
NEXUS -- Backup manager.

Handles creation, listing, and restoration of backups.
Each backup is a ZIP archive containing:
  - SQLite database copy (nexus.db)
  - metadata.json (timestamp, version, file sizes)
"""

from __future__ import annotations

import json
import shutil
import uuid
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List

from loguru import logger

from nexus.config import settings


class BackupManager:
    """Manage NEXUS database backups.

    Usage::

        bm = BackupManager()
        backup_id = await bm.create_backup()
        backups = bm.list_backups()
        await bm.restore_backup(backup_id)
    """

    def __init__(self, backup_dir: Path | None = None) -> None:
        self._backup_dir = backup_dir or (settings.data_dir / "backups")
        self._backup_dir.mkdir(parents=True, exist_ok=True)
        logger.debug("BackupManager initialised (dir={})", self._backup_dir)

    # ------------------------------------------------------------------
    # Create backup
    # ------------------------------------------------------------------

    async def create_backup(self) -> str:
        """Create a ZIP backup of the SQLite database.

        Returns the backup ID (UUID) which can be used to restore later.
        The backup is created atomically: the SQLite file is copied first,
        then zipped.
        """
        backup_id = str(uuid.uuid4())
        timestamp = datetime.utcnow().strftime("%Y%m%d_%H%M%S")
        zip_name = f"nexus_backup_{timestamp}_{backup_id[:8]}.zip"
        zip_path = self._backup_dir / zip_name

        db_path = settings.sqlite_path

        if not db_path.exists():
            raise FileNotFoundError(
                f"SQLite database not found at {db_path}. Cannot create backup."
            )

        logger.info("Creating backup {} -> {}", backup_id, zip_path)

        # Copy the database to a temp file first (safe copy while WAL may be active)
        # Using shutil.copy2 preserves metadata
        temp_db = self._backup_dir / f"_temp_{backup_id}.db"
        try:
            shutil.copy2(str(db_path), str(temp_db))

            # Also copy WAL and SHM files if they exist (for consistency)
            wal_path = db_path.with_suffix(".db-wal")
            shm_path = db_path.with_suffix(".db-shm")

            if wal_path.exists():
                shutil.copy2(str(wal_path), str(temp_db.with_suffix(".db-wal")))
            if shm_path.exists():
                shutil.copy2(str(shm_path), str(temp_db.with_suffix(".db-shm")))

            # Build metadata
            db_size = temp_db.stat().st_size
            metadata = {
                "backup_id": backup_id,
                "created_at": datetime.utcnow().isoformat(),
                "nexus_version": "0.1.0",
                "sqlite_path": str(db_path),
                "db_size_bytes": db_size,
                "filename": zip_name,
            }

            # Create the ZIP archive
            with zipfile.ZipFile(str(zip_path), "w", zipfile.ZIP_DEFLATED) as zf:
                zf.write(str(temp_db), "nexus.db")
                zf.writestr(
                    "metadata.json",
                    json.dumps(metadata, indent=2, ensure_ascii=False),
                )

            zip_size = zip_path.stat().st_size
            logger.info(
                "Backup created: {} (db={}B, zip={}B)",
                zip_name,
                db_size,
                zip_size,
            )

        finally:
            # Clean up temp files
            for suffix in [".db", ".db-wal", ".db-shm"]:
                tmp = self._backup_dir / f"_temp_{backup_id}{suffix}"
                if tmp.exists():
                    tmp.unlink()

        return backup_id

    # ------------------------------------------------------------------
    # List backups
    # ------------------------------------------------------------------

    def list_backups(self) -> List[Dict[str, Any]]:
        """List all available backups, sorted by date (newest first).

        Returns a list of dicts with:
          - backup_id
          - filename
          - created_at
          - size_bytes
          - metadata (if readable)
        """
        backups: List[Dict[str, Any]] = []

        for zip_path in sorted(
            self._backup_dir.glob("nexus_backup_*.zip"),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        ):
            entry: Dict[str, Any] = {
                "filename": zip_path.name,
                "size_bytes": zip_path.stat().st_size,
                "path": str(zip_path),
            }

            # Try to read metadata from the ZIP
            try:
                with zipfile.ZipFile(str(zip_path), "r") as zf:
                    if "metadata.json" in zf.namelist():
                        meta = json.loads(zf.read("metadata.json"))
                        entry["backup_id"] = meta.get("backup_id")
                        entry["created_at"] = meta.get("created_at")
                        entry["db_size_bytes"] = meta.get("db_size_bytes")
                        entry["nexus_version"] = meta.get("nexus_version")
            except (zipfile.BadZipFile, json.JSONDecodeError, KeyError) as exc:
                logger.warning("Failed to read metadata from {}: {}", zip_path.name, exc)
                entry["backup_id"] = None
                entry["created_at"] = datetime.fromtimestamp(
                    zip_path.stat().st_mtime
                ).isoformat()

            backups.append(entry)

        return backups

    # ------------------------------------------------------------------
    # Restore backup
    # ------------------------------------------------------------------

    async def restore_backup(self, backup_id: str) -> None:
        """Restore the database from a backup.

        Finds the backup ZIP by its ID, extracts the SQLite file,
        and replaces the current database.

        WARNING: This will overwrite the current database. The current
        database is backed up automatically before restoration.

        Parameters
        ----------
        backup_id : str
            The UUID returned by ``create_backup()``.
        """
        # Find the backup file
        backup_path = self._find_backup(backup_id)
        if backup_path is None:
            raise FileNotFoundError(
                f"Backup not found: {backup_id}"
            )

        logger.info("Restoring backup {} from {}", backup_id, backup_path)

        db_path = settings.sqlite_path

        # Create an automatic backup of the current DB before restoring
        if db_path.exists():
            safety_backup = db_path.with_suffix(
                f".pre_restore_{datetime.utcnow().strftime('%Y%m%d_%H%M%S')}.db"
            )
            shutil.copy2(str(db_path), str(safety_backup))
            logger.info("Safety backup created: {}", safety_backup)

        # Extract the database from the ZIP
        with zipfile.ZipFile(str(backup_path), "r") as zf:
            if "nexus.db" not in zf.namelist():
                raise ValueError(
                    f"Invalid backup archive: 'nexus.db' not found in {backup_path.name}"
                )

            # Extract to a temp location, then move atomically
            temp_restore = db_path.with_suffix(".restoring.db")
            try:
                with zf.open("nexus.db") as src, open(str(temp_restore), "wb") as dst:
                    shutil.copyfileobj(src, dst)

                # Remove WAL/SHM files (they belong to the old DB)
                for suffix in [".db-wal", ".db-shm"]:
                    wal = db_path.with_suffix(suffix)
                    if wal.exists():
                        wal.unlink()

                # Replace the database
                shutil.move(str(temp_restore), str(db_path))

                logger.info("Database restored from backup {}", backup_id)

            except Exception:
                # Clean up on failure
                if temp_restore.exists():
                    temp_restore.unlink()
                raise

    # ------------------------------------------------------------------
    # Private helpers
    # ------------------------------------------------------------------

    def _find_backup(self, backup_id: str) -> Path | None:
        """Find a backup ZIP file by its backup_id."""
        for zip_path in self._backup_dir.glob("nexus_backup_*.zip"):
            try:
                with zipfile.ZipFile(str(zip_path), "r") as zf:
                    if "metadata.json" in zf.namelist():
                        meta = json.loads(zf.read("metadata.json"))
                        if meta.get("backup_id") == backup_id:
                            return zip_path
            except (zipfile.BadZipFile, json.JSONDecodeError):
                continue

        # Fallback: check if backup_id is in the filename
        for zip_path in self._backup_dir.glob(f"*{backup_id[:8]}*.zip"):
            return zip_path

        return None
