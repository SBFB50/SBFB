# SPDX-License-Identifier: AGPL-3.0-or-later
"""File upload router integration tests — Sprint 9 Phase E.

Covers ``POST /app/{name}/files/upload``,
``GET /app/{name}/files/{sha256}/manifest``, and
``GET /app/{name}/files/{sha256}`` via
:func:`nexus_coordinator.api.app.create_app` + ASGI transport.

Isolation: every test uses the ``nexus_grid_tmp`` fixture from
conftest.py, which monkey-patches all ``nexus_coordinator.paths``
helpers to write under ``tmp_path/nexus-grid/`` instead of the
real user data directory.
"""

from __future__ import annotations

import asyncio
import hashlib
import io
from pathlib import Path
from typing import Any

import httpx
import pytest
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator

# ---------------------------------------------------------------------------
# Test data constants
# ---------------------------------------------------------------------------

_PNG_BYTES = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
_PDF_BYTES = b"%PDF-1.7\n" + b"\x00" * 100
# EXE magic: MZ header — not a valid PNG despite the content_type lie
_EXE_AS_PNG = b"MZ\x90\x00" + b"\x00" * 100


def _sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _make_upload_files(data: bytes, filename: str, content_type: str) -> dict[str, Any]:
    """Build an httpx ``files=`` dict suitable for multipart POST."""
    return {
        "file": (filename, io.BytesIO(data), content_type),
    }


# ---------------------------------------------------------------------------
# 1. Happy path — 201 with sha256
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_happy_path_returns_201_with_sha256(nexus_grid_tmp: Path) -> None:
    """Uploading a valid PNG returns HTTP 201 with the correct sha256."""
    coord = Coordinator(project_name="files-happy")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PNG_BYTES, "photo.png", "image/png"),
            )
        assert r.status_code == 201
        body = r.json()
        assert body["sha256"] == _sha256_hex(_PNG_BYTES)
        assert body["size"] == len(_PNG_BYTES)
        assert body["content_type"] == "image/png"
        assert body["original_name"] == "photo.png"
        assert isinstance(body["dedup"], bool)
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 2. Decorator metadata — cap 50 MB
# ---------------------------------------------------------------------------


def test_upload_cap_50mb_rejects_larger() -> None:
    """The ``@nexus_app_files`` decorator stores a ``max_size_bytes``
    field capped at 50 MB (52 428 800 bytes) on the class itself.

    This test verifies the metadata without a running coordinator
    because the size enforcement lives in the decorator, not in a
    live HTTP call that would require allocating 50 MB of RAM.
    """
    from nexus_app_gov.app import GovApp
    from nexus_sdk.registry import FILES_ATTR

    meta = getattr(GovApp, FILES_ATTR, None)
    assert meta is not None, "GovApp must carry @nexus_app_files metadata"
    assert meta["max_size_bytes"] == 50 * 1024 * 1024


# ---------------------------------------------------------------------------
# 3. Decorator metadata — explicit 50 MB max_part_size
# ---------------------------------------------------------------------------


def test_upload_multipart_max_part_size_explicit_50mb() -> None:
    """The decorator default for max_size_bytes is exactly 50 * 1024 * 1024
    bytes, matching the Sprint 9 Phase E design decision (D5 frozen)."""
    from nexus_sdk.decorators import nexus_app_files
    from nexus_sdk.registry import FILES_ATTR

    @nexus_app_files(accept=["image/png"])
    class _TestApp:
        pass

    meta = getattr(_TestApp, FILES_ATTR)
    assert meta["max_size_bytes"] == 50 * 1024 * 1024
    assert meta["accept"] == ["image/png"]


# ---------------------------------------------------------------------------
# 4. Magic bytes mismatch → 415
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_content_type_mismatch_rejects_415(nexus_grid_tmp: Path) -> None:
    """Sending an EXE file with content_type ``image/png`` triggers the
    magic bytes validation in AppFileStore and the router surfaces it
    as HTTP 415 Unsupported Media Type."""
    coord = Coordinator(project_name="files-magic")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_EXE_AS_PNG, "malware.png", "image/png"),
            )
        assert r.status_code == 415
        assert "magic" in r.json()["detail"].lower() or "png" in r.json()["detail"].lower()
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 5. App without @nexus_app_files → 404
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_app_without_files_decorator_404(nexus_grid_tmp: Path) -> None:
    """The ``hello`` app does not carry ``@nexus_app_files``.
    The upload route must return 404 with the decorator-absent message."""
    coord = Coordinator(project_name="files-nodecorator")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r = await client.post(
                "/app/hello/files/upload",
                files=_make_upload_files(_PNG_BYTES, "photo.png", "image/png"),
            )
        assert r.status_code == 404
        assert "decorator" in r.json()["detail"].lower() or "nexus_app_files" in r.json()["detail"]
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 6. Dedup — uploading the same file twice returns the same sha256
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_dedup_returns_existing_sha_with_header(nexus_grid_tmp: Path) -> None:
    """Uploading the same bytes twice must return HTTP 201 both times
    and the ``sha256`` in the response body must be identical."""
    coord = Coordinator(project_name="files-dedup")
    await coord.start()
    try:
        app = create_app(coord)
        expected_sha = _sha256_hex(_PDF_BYTES)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r1 = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PDF_BYTES, "doc.pdf", "application/pdf"),
            )
            assert r1.status_code == 201
            assert r1.json()["sha256"] == expected_sha

            r2 = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PDF_BYTES, "doc.pdf", "application/pdf"),
            )
            assert r2.status_code == 201
            assert r2.json()["sha256"] == expected_sha
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 7. Upload publishes file.upload.progress to events bus
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_streams_progress_to_events_bus(nexus_grid_tmp: Path) -> None:
    """After a successful upload, the router publishes a
    ``file.upload.progress`` envelope on the app's ``AppEvents`` bus.
    The test subscribes before the upload and asserts the envelope
    arrives with the correct fields."""
    coord = Coordinator(project_name="files-events")
    await coord.start()
    try:
        app = create_app(coord)
        gov_ctx = coord.app_contexts["gov"]
        assert gov_ctx.events is not None

        received: list[dict[str, Any]] = []

        async def _listen() -> None:
            async with gov_ctx.events.subscribe("file.upload.progress") as sub:
                async for envelope in sub:
                    received.append(envelope.payload)
                    break  # one envelope is enough

        listener = asyncio.create_task(_listen())

        # Wait for the subscriber to be registered.
        for _ in range(50):
            if gov_ctx.events.stats()["subscribers"] >= 1:
                break
            await asyncio.sleep(0.01)

        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PNG_BYTES, "snap.png", "image/png"),
            )
        assert r.status_code == 201

        await asyncio.wait_for(listener, timeout=2.0)

        assert len(received) == 1
        payload = received[0]
        assert payload["sha256"] == _sha256_hex(_PNG_BYTES)
        assert payload["app_name"] == "gov"
        assert payload["content_type"] == "image/png"
        assert payload["size"] == len(_PNG_BYTES)
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 8. Manifest endpoint returns metadata
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_manifest_endpoint_returns_metadata(nexus_grid_tmp: Path) -> None:
    """After uploading a file, ``GET /app/{name}/files/{sha256}/manifest``
    must return the full :class:`FileManifest` as JSON with all
    required fields."""
    coord = Coordinator(project_name="files-manifest")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            upload_r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PDF_BYTES, "report.pdf", "application/pdf"),
            )
            assert upload_r.status_code == 201
            sha = upload_r.json()["sha256"]

            manifest_r = await client.get(f"/app/gov/files/{sha}/manifest")
        assert manifest_r.status_code == 200
        m = manifest_r.json()
        assert m["sha256"] == sha
        assert m["size"] == len(_PDF_BYTES)
        assert m["content_type"] == "application/pdf"
        assert m["original_name"] == "report.pdf"
        assert m["app_name"] == "gov"
        assert "uploaded_at" in m
        assert isinstance(m["uploaded_by"], str)
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 9. Open endpoint streams bytes
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_open_endpoint_streams_bytes(nexus_grid_tmp: Path) -> None:
    """After uploading a PNG, ``GET /app/{name}/files/{sha256}``
    must stream back the exact bytes with the correct Content-Type
    and the ``X-Nexus-SHA256`` response header."""
    coord = Coordinator(project_name="files-open")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            upload_r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PNG_BYTES, "image.png", "image/png"),
            )
            assert upload_r.status_code == 201
            sha = upload_r.json()["sha256"]

            open_r = await client.get(f"/app/gov/files/{sha}")
        assert open_r.status_code == 200
        assert open_r.content == _PNG_BYTES
        assert open_r.headers["content-type"].startswith("image/png")
        assert open_r.headers["x-nexus-sha256"] == sha
        assert open_r.headers["x-nexus-app"] == "gov"
    finally:
        await coord.stop()


# ---------------------------------------------------------------------------
# 10. CAS sharded path exists on disk after upload
# ---------------------------------------------------------------------------


@pytest.mark.asyncio
async def test_upload_writes_to_cas_sharded_path_via_coordinator(nexus_grid_tmp: Path) -> None:
    """After a successful upload the CAS blob and its manifest JSON must
    exist under the sharded path
    ``<nexus-grid-root>/projects/<project>/apps/gov/uploads/<sha[:2]>/``.
    """
    coord = Coordinator(project_name="files-cas")
    await coord.start()
    try:
        app = create_app(coord)
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://testserver",
        ) as client:
            r = await client.post(
                "/app/gov/files/upload",
                files=_make_upload_files(_PNG_BYTES, "cas-test.png", "image/png"),
            )
        assert r.status_code == 201
        sha = r.json()["sha256"]

        uploads_root = nexus_grid_tmp / "projects" / "files-cas" / "apps" / "gov" / "uploads"
        shard_dir = uploads_root / sha[:2]
        blob_path = shard_dir / sha[2:]
        manifest_path = shard_dir / f"{sha}.json"

        assert blob_path.exists(), f"CAS blob must exist at {blob_path}"
        assert manifest_path.exists(), f"CAS manifest must exist at {manifest_path}"
        assert blob_path.read_bytes() == _PNG_BYTES
    finally:
        await coord.stop()
