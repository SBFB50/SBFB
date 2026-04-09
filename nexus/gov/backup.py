"""Automated PostgreSQL backup for NEXUS GOV."""
import asyncio
import os
import subprocess
import logging
from datetime import datetime, timezone

logger = logging.getLogger(__name__)


async def backup_database(
    db_url: str = None,
    backup_dir: str = None,
    retention_days: int = 30,
) -> str | None:
    """Run pg_dump and manage retention. Returns backup file path."""
    db_url = db_url or os.environ.get("GOV_DATABASE_URL", "")
    if not db_url or not db_url.startswith("postgres"):
        logger.info("No PostgreSQL URL configured, skipping backup")
        return None

    backup_dir = backup_dir or os.path.join(
        os.environ.get("NEXUS_DATA_DIR", "data"), "backups"
    )
    os.makedirs(backup_dir, exist_ok=True)

    timestamp = datetime.now(timezone.utc).strftime("%Y%m%d_%H%M%S")
    filename = f"nexus_gov_{timestamp}.sql.gz"
    filepath = os.path.join(backup_dir, filename)

    # pg_dump with gzip
    cmd = f'pg_dump "{db_url}" | gzip > "{filepath}"'
    proc = await asyncio.create_subprocess_shell(
        cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE
    )
    _, stderr = await proc.communicate()

    if proc.returncode != 0:
        logger.error("Backup failed: %s", stderr.decode())
        return None

    logger.info(
        "Backup created: %s (%d bytes)", filepath, os.path.getsize(filepath)
    )

    # Retention: delete old backups
    await _cleanup_old_backups(backup_dir, retention_days)

    return filepath


async def _cleanup_old_backups(backup_dir: str, retention_days: int):
    """Delete backups older than retention_days."""
    import time

    cutoff = time.time() - (retention_days * 86400)
    for f in os.listdir(backup_dir):
        if f.startswith("nexus_gov_") and f.endswith(".sql.gz"):
            path = os.path.join(backup_dir, f)
            if os.path.getmtime(path) < cutoff:
                os.unlink(path)
                logger.info("Deleted old backup: %s", f)
