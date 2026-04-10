"""Minimal in-process schema migrator.

Sprint 4 Phase B only has a single schema version (1) so the
migrator is trivially a script runner. Later phases will bump
:data:`LATEST_SCHEMA_VERSION` and add ``upgrade_N_to_M`` helpers.
"""

from __future__ import annotations

from importlib.resources import files
from pathlib import Path

import aiosqlite

LATEST_SCHEMA_VERSION = 1


async def init_db(db_path: Path) -> None:
    """Create or upgrade the coordinator state DB.

    Reads ``db/schema.sql`` from the installed package (via
    :func:`importlib.resources.files`) and runs it as a single
    script. The statements are all idempotent so running on an
    existing DB is a no-op.
    """
    db_path.parent.mkdir(parents=True, exist_ok=True)
    schema = files("nexus_coordinator.db").joinpath("schema.sql").read_text(encoding="utf-8")
    async with aiosqlite.connect(db_path) as db:
        # WAL + NORMAL = fast-and-safe for the mostly-append
        # workload of task_state and kudos_ledger.
        await db.execute("PRAGMA journal_mode = WAL")
        await db.execute("PRAGMA synchronous = NORMAL")
        await db.executescript(schema)
        await db.commit()

        # Assert we're at the expected version so future migrations
        # can safely assume the on-disk layout.
        async with db.execute("SELECT MAX(version) FROM schema_version") as cursor:
            row = await cursor.fetchone()
            current = row[0] if row and row[0] is not None else 0
        if current < LATEST_SCHEMA_VERSION:
            raise RuntimeError(
                f"schema_version {current} predates the bundled schema "
                f"(expected {LATEST_SCHEMA_VERSION}); upgrade path not yet implemented"
            )
