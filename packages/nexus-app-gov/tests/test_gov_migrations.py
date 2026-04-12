# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for gov app migration consumer (Sprint 9 Phase D).

The 4 scenarios listed in ``.planning/sprint9_plan.md`` §7.4 verify
that the GovApp correctly wires two database clients and that the
``001_documents.sql`` migration applies cleanly through the
:class:`nexus_sdk.MigrationRunner`.
"""

from __future__ import annotations

from pathlib import Path

import aiosqlite
import pytest
from nexus_app_gov.app import GovApp
from nexus_sdk import (
    AppContext,
    AppDatabaseClient,
    AppEvents,
    AppStorage,
    ComputeClient,
    MigrationRunner,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_ctx(tmp_path: Path) -> tuple[AppContext, Path]:
    """Build an AppContext that mirrors what the coordinator wires
    for the gov app: a writable default client at app.sqlite,
    storage, and events. Returns ``(ctx, db_path)``."""
    db_path = tmp_path / "apps" / "gov" / "app.sqlite"
    db_path.parent.mkdir(parents=True, exist_ok=True)
    default_db = AppDatabaseClient(db_path)
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-mig-test",
        app_name="gov",
        db=default_db,
        storage=AppStorage(tmp_path / "apps" / "gov" / "storage.json"),
        events=AppEvents(),
    )
    return ctx, db_path


# ---------------------------------------------------------------------------
# 1 — test_gov_migration_001_creates_documents_table
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_gov_migration_001_creates_documents_table(tmp_path: Path) -> None:
    """The runner applies ``001_documents.sql`` and creates the
    ``gov_documents`` table in the writable app.sqlite."""
    ctx, db_path = _make_ctx(tmp_path)
    app = GovApp()
    await app.on_start(ctx)

    runner = MigrationRunner(ctx.dbs["default"], app.manifest.migrations_dir)
    applied = await runner.apply()

    assert len(applied) == 1
    assert applied[0].slug == "documents"

    async with aiosqlite.connect(db_path) as db:
        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='gov_documents'")
        assert await cursor.fetchone() is not None


# ---------------------------------------------------------------------------
# 2 — test_gov_migration_is_idempotent_on_coordinator_restart
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_gov_migration_is_idempotent_on_coordinator_restart(tmp_path: Path) -> None:
    """Running the runner twice (simulating a coordinator restart)
    applies the migration only once."""
    ctx, _ = _make_ctx(tmp_path)
    app = GovApp()
    await app.on_start(ctx)

    runner = MigrationRunner(ctx.dbs["default"], app.manifest.migrations_dir)
    first = await runner.apply()
    assert len(first) == 1

    second = await runner.apply()
    assert len(second) == 0


# ---------------------------------------------------------------------------
# 3 — test_gov_dbs_contains_db_gov_and_db_app
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_gov_dbs_contains_db_gov_and_db_app(tmp_path: Path) -> None:
    """After ``on_start``, ``ctx.dbs`` contains both ``gov`` (legacy
    read-only or default fallback) and ``app`` (writable) keys,
    plus the coordinator-wired ``default``."""
    ctx, _ = _make_ctx(tmp_path)
    app = GovApp()
    await app.on_start(ctx)

    assert "default" in ctx.dbs
    assert "gov" in ctx.dbs
    assert "app" in ctx.dbs
    # "app" is the same client as "default" (writable per-app sqlite)
    assert ctx.dbs["app"] is ctx.dbs["default"]


# ---------------------------------------------------------------------------
# 4 — test_gov_db_gov_is_read_only_db_app_is_writable
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_gov_db_gov_is_read_only_db_app_is_writable(tmp_path: Path) -> None:
    """The legacy client (gov) is read-only when the legacy file
    exists, and the app client is always writable."""
    # Create a fake legacy govdata.db so on_start picks it up.
    # We patch _legacy_govdata_db_path to point at a tmp file.
    legacy_path = tmp_path / "nexus" / "gov" / "govdata.db"
    legacy_path.parent.mkdir(parents=True, exist_ok=True)
    legacy_path.touch()

    import nexus_app_gov.app as gov_mod

    original = gov_mod._legacy_govdata_db_path

    def _patched() -> Path:
        return legacy_path

    gov_mod._legacy_govdata_db_path = _patched  # type: ignore[assignment]
    try:
        ctx, _ = _make_ctx(tmp_path)
        app = GovApp()
        await app.on_start(ctx)

        # ctx.db was swapped to legacy read-only
        assert ctx.db is not None
        assert ctx.db.read_only is True
        # dbs["gov"] is the same read-only client
        assert ctx.dbs["gov"].read_only is True
        # dbs["app"] / dbs["default"] are writable
        assert ctx.dbs["app"].read_only is False
        assert ctx.dbs["default"].read_only is False
    finally:
        gov_mod._legacy_govdata_db_path = original  # type: ignore[assignment]
