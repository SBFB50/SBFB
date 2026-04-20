# SPDX-License-Identifier: AGPL-3.0-or-later
"""Per-(consumer, model) PoW task counter with daily UTC reset.

Sprint 23 Phase C. Tracks how many tasks a consumer has submitted
for a given model within the current UTC day. The count drives the
escalating PoW difficulty ramp (Rust-side ``EscalatingPolicy``):
difficulty doubles every ``tranche_size`` tasks submitted.

The counter resets at midnight UTC (day-of-epoch comparison). On
startup, ``reset_expired()`` purges stale rows so disk doesn't
grow unbounded.

Schema: SQLite WAL (same pattern as quarantine_queue S21 Phase D).
"""

from __future__ import annotations

from datetime import datetime, timezone
from pathlib import Path

import aiosqlite
import structlog

_log = structlog.get_logger(__name__)

_SCHEMA = """\
CREATE TABLE IF NOT EXISTS pow_task_counts (
    consumer_id TEXT NOT NULL,
    model_id    TEXT NOT NULL,
    count       INTEGER NOT NULL DEFAULT 0,
    last_reset_utc TEXT NOT NULL,
    PRIMARY KEY (consumer_id, model_id)
);
"""


def _today_utc() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%d")


def _day_of_epoch(iso_date: str) -> int:
    dt = datetime.strptime(iso_date, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    return int(dt.timestamp()) // 86_400


class PowCounter:
    """Async SQLite counter per (consumer_id, model_id) with daily reset."""

    def __init__(self, db_path: Path) -> None:
        self._db_path = db_path
        self._db: aiosqlite.Connection | None = None

    async def open(self) -> None:
        self._db = await aiosqlite.connect(self._db_path)
        await self._db.execute("PRAGMA journal_mode=WAL")
        await self._db.execute("PRAGMA busy_timeout=5000")
        await self._db.executescript(_SCHEMA)
        await self._db.commit()

    async def close(self) -> None:
        if self._db:
            await self._db.close()
            self._db = None

    async def increment(self, consumer_id: str, model_id: str) -> int:
        """Increment and return the new count for (consumer, model).

        If the stored row belongs to a previous UTC day, it is reset
        to 1 (today's first task).
        """
        assert self._db is not None
        today = _today_utc()

        row = await self._db.execute_fetchall(
            "SELECT count, last_reset_utc FROM pow_task_counts WHERE consumer_id = ? AND model_id = ?",
            (consumer_id, model_id),
        )

        if row and row[0][1] == today:
            new_count = row[0][0] + 1
            await self._db.execute(
                "UPDATE pow_task_counts SET count = ? WHERE consumer_id = ? AND model_id = ?",
                (new_count, consumer_id, model_id),
            )
        else:
            new_count = 1
            await self._db.execute(
                "INSERT OR REPLACE INTO pow_task_counts "
                "(consumer_id, model_id, count, last_reset_utc) "
                "VALUES (?, ?, 1, ?)",
                (consumer_id, model_id, today),
            )

        await self._db.commit()
        _log.debug(
            "pow_counter.increment",
            consumer_id=consumer_id,
            model_id=model_id,
            count=new_count,
        )
        return new_count

    async def get_count(self, consumer_id: str, model_id: str) -> int:
        """Return current count for (consumer, model), 0 if absent or expired."""
        assert self._db is not None
        today = _today_utc()

        row = await self._db.execute_fetchall(
            "SELECT count, last_reset_utc FROM pow_task_counts WHERE consumer_id = ? AND model_id = ?",
            (consumer_id, model_id),
        )

        if not row:
            return 0
        if row[0][1] != today:
            return 0
        return row[0][0]

    async def reset_expired(self) -> int:
        """Delete all rows whose last_reset_utc is before today. Returns deleted count."""
        assert self._db is not None
        today = _today_utc()

        cursor = await self._db.execute(
            "DELETE FROM pow_task_counts WHERE last_reset_utc < ?",
            (today,),
        )
        await self._db.commit()
        deleted = cursor.rowcount
        if deleted > 0:
            _log.info("pow_counter.reset_expired", deleted=deleted)
        return deleted
