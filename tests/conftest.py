"""
Shared pytest fixtures for NEXUS test suite.

Provides an in-memory SQLite database fixture so tests run
without any external service (Ollama, Neo4j, ChromaDB, etc.).
"""

import asyncio
import os
import sys
from pathlib import Path

import pytest
import pytest_asyncio
import aiosqlite

# Make sure the project root is on sys.path
_project_root = str(Path(__file__).resolve().parent.parent)
if _project_root not in sys.path:
    sys.path.insert(0, _project_root)


# Force asyncio event loop mode for all async tests
@pytest.fixture(scope="session")
def event_loop_policy():
    return asyncio.DefaultEventLoopPolicy()


@pytest_asyncio.fixture
async def memory_conn():
    """Yield an aiosqlite connection backed by :memory:.

    The connection has row_factory set and foreign keys enabled,
    matching the real ``get_db()`` behavior.
    """
    conn = await aiosqlite.connect(":memory:")
    conn.row_factory = aiosqlite.Row
    await conn.execute("PRAGMA foreign_keys=ON")

    # Create the full schema (imported from the real DDL)
    from nexus.db.sqlite_db import _CREATE_TABLES, _CREATE_INDEXES

    await conn.executescript(_CREATE_TABLES)
    await conn.executescript(_CREATE_INDEXES)
    await conn.commit()

    yield conn
    await conn.close()


@pytest_asyncio.fixture
async def db(memory_conn):
    """Yield a Database instance bound to an in-memory connection."""
    from nexus.db.sqlite_db import Database

    return Database(memory_conn)


@pytest_asyncio.fixture
async def bus(tmp_path):
    """Yield an EventBus backed by a temporary SQLite file.

    Uses a file (not :memory:) because EventBus opens multiple
    connections internally via aiosqlite.connect(db_path).
    """
    from nexus.events.bus import EventBus

    db_file = str(tmp_path / "bus_test.db")
    b = EventBus(db_path=db_file)
    await b.start()
    yield b
    await b.stop()
