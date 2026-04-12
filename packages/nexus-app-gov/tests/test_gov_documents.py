# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for the gov Documents tab — Sprint 9 Phase E (20th tab).

Six scenarios verify the file upload + CAS listing surface introduced
by :meth:`nexus_app_gov.app.GovApp.documents_tab`:

1. ``test_documents_tab_empty_state_when_no_uploads`` — empty block
   when ``gov_documents`` has no rows.
2. ``test_documents_tab_lists_uploaded_via_db_app`` — table block
   with one row per inserted document.
3. ``test_documents_tab_descriptor_uses_v2_schema`` — schema_version
   equals 2 (TabViewV2).
4. ``test_documents_tab_renders_file_upload_block`` — the descriptor
   always contains a ``file_upload`` kind block.
5. ``test_gov_app_accepts_pdf_and_images`` — the ``@nexus_app_files``
   decorator stores the correct MIME allowlist on the class.
6. ``test_gov_documents_migration_creates_table`` — the SQL migration
   ``001_documents.sql`` creates the ``gov_documents`` table plus the
   ``idx_gov_documents_sha256`` index.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path
from typing import Any

import aiosqlite
import pytest
from nexus_app_gov import GovApp
from nexus_sdk import AppContext, AppDatabaseClient, AppStorage, ComputeClient
from nexus_sdk.registry import FILES_ATTR

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

_DOCUMENTS_SCHEMA = """
CREATE TABLE IF NOT EXISTS gov_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sha256 TEXT NOT NULL,
    original_name TEXT NOT NULL DEFAULT '',
    content_type TEXT NOT NULL DEFAULT '',
    size INTEGER NOT NULL DEFAULT 0,
    uploaded_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);
"""


def _make_ctx(tmp_path: Path) -> tuple[GovApp, AppContext]:
    """Build a started GovApp + AppContext wired with a writable app.sqlite.

    Mirrors the coordinator's two-client model: the default
    ``AppDatabaseClient`` at ``app.sqlite`` is exposed as both
    ``ctx.dbs["default"]`` (auto-sync via ``__post_init__``) and
    ``ctx.dbs["app"]`` (wired by ``GovApp.on_start``).
    """
    db_path = tmp_path / "apps" / "gov" / "app.sqlite"
    db_path.parent.mkdir(parents=True, exist_ok=True)
    storage_path = tmp_path / "apps" / "gov" / "storage.json"
    app_db = AppDatabaseClient(db_path)
    ctx = AppContext(
        compute=ComputeClient("http://127.0.0.1:65500"),
        project_name="gov-docs-test",
        app_name="gov",
        db=app_db,
        storage=AppStorage(storage_path),
    )
    return GovApp(), ctx


def _seed_documents(db_path: Path, count: int) -> None:
    """Synchronously insert ``count`` rows into ``gov_documents``."""
    conn = sqlite3.connect(db_path)
    try:
        conn.executescript(_DOCUMENTS_SCHEMA)
        rows = [
            (
                f"sha256-{i:04x}",
                f"document_{i}.pdf",
                "application/pdf",
                1024 * (i + 1),
                f"2026-04-{i + 1:02d}T12:00:00Z",
            )
            for i in range(count)
        ]
        conn.executemany(
            "INSERT INTO gov_documents (sha256, original_name, content_type, size, uploaded_at) VALUES (?, ?, ?, ?, ?)",
            rows,
        )
        conn.commit()
    finally:
        conn.close()


def _block_kinds(descriptor: dict[str, Any]) -> list[str]:
    return [block["kind"] for block in descriptor["blocks"]]


# ---------------------------------------------------------------------------
# 1 — empty state
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_documents_tab_empty_state_when_no_uploads(tmp_path: Path) -> None:
    """When no rows are present in ``gov_documents`` the tab renders
    an ``empty`` block (not a ``table`` block) so the React shell
    can display the placeholder text without crashing on an empty
    columns array."""
    app, ctx = _make_ctx(tmp_path)
    await app.on_start(ctx)

    descriptor = await app.documents_tab()

    kinds = _block_kinds(descriptor)
    assert "empty" in kinds, f"expected an empty block, got kinds={kinds}"
    assert "table" not in kinds


# ---------------------------------------------------------------------------
# 2 — rows present → table block
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_documents_tab_lists_uploaded_via_db_app(tmp_path: Path) -> None:
    """When rows exist in ``gov_documents`` the tab renders a ``table``
    block.  The row data must include the ``original_name`` and
    ``content_type`` columns so the UI can display them without
    additional API calls."""
    app, ctx = _make_ctx(tmp_path)
    await app.on_start(ctx)

    # Seed documents into the writable app.sqlite AFTER on_start so
    # ctx.dbs["app"] is already wired; we grab the path from the client.
    db_path = tmp_path / "apps" / "gov" / "app.sqlite"
    _seed_documents(db_path, count=3)

    descriptor = await app.documents_tab()

    kinds = _block_kinds(descriptor)
    assert "table" in kinds, f"expected a table block, got kinds={kinds}"
    assert "empty" not in kinds

    table_block = next(b for b in descriptor["blocks"] if b["kind"] == "table")
    assert len(table_block["rows"]) == 3
    first = table_block["rows"][0]
    assert "original_name" in first
    assert "content_type" in first


# ---------------------------------------------------------------------------
# 3 — schema_version == 2
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_documents_tab_descriptor_uses_v2_schema(tmp_path: Path) -> None:
    """The Documents tab must return a v2 descriptor so the React
    ``TabViewV2`` parser accepts the ``file_upload`` block kind."""
    app, ctx = _make_ctx(tmp_path)
    await app.on_start(ctx)

    descriptor = await app.documents_tab()

    assert descriptor["schema_version"] == 2, f"expected schema_version=2, got {descriptor.get('schema_version')}"


# ---------------------------------------------------------------------------
# 4 — file_upload block always present
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_documents_tab_renders_file_upload_block(tmp_path: Path) -> None:
    """The ``file_upload`` drop-zone must appear regardless of whether
    any documents have been uploaded, so first-time users see the
    upload affordance immediately."""
    app, ctx = _make_ctx(tmp_path)
    await app.on_start(ctx)

    descriptor = await app.documents_tab()

    kinds = _block_kinds(descriptor)
    assert "file_upload" in kinds, f"expected a file_upload block, got kinds={kinds}"

    upload_block = next(b for b in descriptor["blocks"] if b["kind"] == "file_upload")
    # Must carry an accept list so the browser-side <input> restricts
    # the file chooser to PDFs and images.
    assert "accept" in upload_block
    assert len(upload_block["accept"]) > 0


# ---------------------------------------------------------------------------
# 5 — @nexus_app_files decorator metadata
# ---------------------------------------------------------------------------


def test_gov_app_accepts_pdf_and_images() -> None:
    """The ``@nexus_app_files`` decorator must store the correct MIME
    allowlist on the class under ``FILES_ATTR``.  The coordinator
    reads this attribute at upload-request time to enforce the
    allowlist before any bytes reach the CAS."""
    meta = getattr(GovApp, FILES_ATTR, None)
    assert meta is not None, f"GovApp is missing the {FILES_ATTR!r} attribute — @nexus_app_files decorator not applied"
    accept: list[str] = meta["accept"]
    assert "application/pdf" in accept, f"PDF not in accept list: {accept}"
    # At least one image wildcard or specific image MIME must be present.
    assert any(m.startswith("image/") for m in accept), f"no image/* pattern in accept list: {accept}"
    # max_size_bytes must be a positive int.
    assert meta["max_size_bytes"] > 0


# ---------------------------------------------------------------------------
# 6 — migration creates gov_documents table
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_gov_documents_migration_creates_table(tmp_path: Path) -> None:
    """Executing ``001_documents.sql`` via raw aiosqlite must create
    the ``gov_documents`` table and the ``idx_gov_documents_sha256``
    index.  This mirrors what :class:`nexus_sdk.MigrationRunner`
    does at coordinator boot — here we test the SQL itself in
    isolation so a regression in the migration file is caught
    independently of the runner machinery."""
    sql_path = Path(__file__).resolve().parents[1] / "src" / "nexus_app_gov" / "migrations" / "001_documents.sql"
    assert sql_path.exists(), f"migration file not found: {sql_path}"

    db_path = tmp_path / "migration_test.sqlite"
    async with aiosqlite.connect(db_path) as db:
        await db.executescript(sql_path.read_text(encoding="utf-8"))
        await db.commit()

        cursor = await db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='gov_documents'")
        assert await cursor.fetchone() is not None, "gov_documents table not created by 001_documents.sql"

        cursor = await db.execute(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_gov_documents_sha256'"
        )
        assert await cursor.fetchone() is not None, "idx_gov_documents_sha256 index not created by 001_documents.sql"
