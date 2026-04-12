# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 9 Phase C — gov refresh_party_cache + party.refreshed.

Three tests covering the publisher half of the AppContext.events
story:

1. ``test_refresh_party_cache_publishes_envelope`` — calling the
   worker fans an envelope onto the bus that a same-app
   subscriber observes.
2. ``test_refresh_party_cache_payload_shape`` — the payload
   carries an ``int count`` and an ISO-8601 ``refreshed_at``
   string the SSE bridge can serialize.
3. ``test_refresh_party_cache_publishes_zero_when_no_table`` —
   a missing ``gov_parties`` table degrades to ``count=0``
   instead of raising, so a fresh install still publishes a
   well-formed envelope the consumer can render as an empty
   grid.
"""

from __future__ import annotations

import asyncio
import sqlite3
from pathlib import Path

import pytest
from nexus_app_gov import GovApp
from nexus_sdk import (
    AppContext,
    AppDatabaseClient,
    AppEvents,
    ComputeClient,
    EventEnvelope,
)


def _seed_parties_db(db_path: Path, count: int) -> None:
    """Create a tiny ``gov_parties`` table with ``count`` rows."""
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript(
            """
            CREATE TABLE gov_parties (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                short_name TEXT
            );
            """
        )
        rows = [(f"p-{i}", f"Party {i}", f"P{i}") for i in range(count)]
        conn.executemany("INSERT INTO gov_parties VALUES (?, ?, ?)", rows)
        conn.commit()
    finally:
        conn.close()


def _make_ctx(*, db: AppDatabaseClient | None, events: AppEvents) -> AppContext:
    """Build an AppContext with the bus + db a worker test needs."""
    return AppContext(
        compute=ComputeClient("http://127.0.0.1:65501"),
        project_name="gov-events-test",
        app_name="gov",
        db=db,
        events=events,
    )


@pytest.mark.asyncio
async def test_refresh_party_cache_publishes_envelope(tmp_path: Path) -> None:
    """Calling the worker publishes one ``party.refreshed``
    envelope on ``ctx.events`` that a parallel subscriber
    receives via the standard ``async with subscribe`` shape."""
    db_path = tmp_path / "parties.sqlite"
    _seed_parties_db(db_path, count=3)

    bus = AppEvents()
    db = AppDatabaseClient(db_path, read_only=True)
    ctx = _make_ctx(db=db, events=bus)
    app = GovApp()
    try:
        async with bus.subscribe("party.refreshed") as stream:
            await app.refresh_party_cache(ctx)
            envelope = await asyncio.wait_for(stream.receive(), timeout=1.0)
            assert isinstance(envelope, EventEnvelope)
            assert envelope.topic == "party.refreshed"
            assert envelope.payload["count"] == 3
    finally:
        await bus.aclose()


@pytest.mark.asyncio
async def test_refresh_party_cache_payload_shape(tmp_path: Path) -> None:
    """The published payload is exactly ``{count: int,
    refreshed_at: str}`` — the count is an int (not a sqlite
    Row), and ``refreshed_at`` is an ISO 8601 string the SSE
    bridge can json.dumps without a custom encoder."""
    db_path = tmp_path / "parties.sqlite"
    _seed_parties_db(db_path, count=2)

    bus = AppEvents()
    db = AppDatabaseClient(db_path, read_only=True)
    ctx = _make_ctx(db=db, events=bus)
    app = GovApp()
    try:
        async with bus.subscribe("party.refreshed") as stream:
            result = await app.refresh_party_cache(ctx)
            envelope = await asyncio.wait_for(stream.receive(), timeout=1.0)
        assert result == envelope.payload
        assert envelope.payload.keys() == {"count", "refreshed_at"}
        assert isinstance(envelope.payload["count"], int)
        assert envelope.payload["count"] == 2
        assert isinstance(envelope.payload["refreshed_at"], str)
        # ISO 8601 with timezone — fromisoformat must round-trip.
        from datetime import datetime

        parsed = datetime.fromisoformat(envelope.payload["refreshed_at"])
        assert parsed.tzinfo is not None
    finally:
        await bus.aclose()


@pytest.mark.asyncio
async def test_refresh_party_cache_publishes_zero_when_no_table(tmp_path: Path) -> None:
    """A fresh install missing ``gov_parties`` must still emit a
    well-formed ``party.refreshed`` envelope with ``count=0``
    so the consumer can render the empty state without an HTTP
    error."""
    db_path = tmp_path / "empty.sqlite"
    # Create an empty database without the gov_parties table.
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript("CREATE TABLE other_table (id INTEGER PRIMARY KEY);")
        conn.commit()
    finally:
        conn.close()

    bus = AppEvents()
    db = AppDatabaseClient(db_path, read_only=True)
    ctx = _make_ctx(db=db, events=bus)
    app = GovApp()
    try:
        async with bus.subscribe("party.refreshed") as stream:
            result = await app.refresh_party_cache(ctx)
            envelope = await asyncio.wait_for(stream.receive(), timeout=1.0)
        assert result["count"] == 0
        assert envelope.payload["count"] == 0
        assert "refreshed_at" in envelope.payload
    finally:
        await bus.aclose()
