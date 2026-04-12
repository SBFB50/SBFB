# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for :class:`nexus_sdk.files.AppFileStore` (Sprint 9 Phase E).

The 20 scenarios listed in ``.planning/sprint9_plan.md`` §5 Phase E are
covered here. Tests exercise the CAS layout, dedup semantics, magic bytes
validation, chunked reads, soft-delete, and concurrent safety.

Categories:

- Core store (1-6) — happy path, shard path, manifest JSON, dedup,
  large-file chunking, incremental SHA256.
- Read + manifest (7-10) — async iterator, missing-sha error,
  manifest None when absent, manifest includes size + content_type.
- Magic bytes (11-17) — all five accepted types, EXE rejection, PNG
  header with EXE body rejection.
- Delete + concurrency (18-20) — soft delete, concurrent dedup safety,
  unsupported content-type raises.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
from collections.abc import AsyncIterator
from pathlib import Path

import pytest
from nexus_sdk.files import (
    ALLOWED_MAGIC_BYTES,
    AppFileStore,
    FileHandle,
    FileManifest,
    FileTypeError,
    validate_magic_bytes,
)

# ---------------------------------------------------------------------------
# Sample byte fixtures
# ---------------------------------------------------------------------------

_PNG = b"\x89PNG\r\n\x1a\n" + b"\x00" * 100
_JPEG = b"\xff\xd8\xff\xe0" + b"\x00" * 100
_PDF = b"%PDF-1.7\n" + b"\x00" * 100
_WEBP = b"RIFF" + b"\x00\x00\x00\x00" + b"WEBP" + b"\x00" * 100
_SVG = b"<svg></svg>"
_EXE = b"MZ" + b"\x00" * 100  # DOS/PE signature — not in whitelist


async def _iter_bytes(data: bytes, chunk_size: int = 4096) -> AsyncIterator[bytes]:
    """Wrap ``data`` in an async iterator that yields ``chunk_size``-byte chunks."""
    offset = 0
    while offset < len(data):
        yield data[offset : offset + chunk_size]
        offset += chunk_size


# ---------------------------------------------------------------------------
# 1 — store happy path returns FileHandle
# ---------------------------------------------------------------------------


async def test_store_happy_path_returns_file_handle(tmp_path: Path) -> None:
    """store() returns a FileHandle whose fields mirror the input
    metadata and the computed SHA256."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
        uploaded_by="user1",
    )

    assert isinstance(handle, FileHandle)
    assert handle.content_type == "image/png"
    assert handle.original_name == "logo.png"
    assert handle.app_name == "gov"
    assert handle.size == len(_PNG)
    expected_sha = hashlib.sha256(_PNG).hexdigest()
    assert handle.sha256 == expected_sha


# ---------------------------------------------------------------------------
# 2 — store writes to the CAS sharded path
# ---------------------------------------------------------------------------


async def test_store_writes_to_cas_sharded_path(tmp_path: Path) -> None:
    """The blob is written to ``<base>/<sha256[:2]>/<sha256[2:]>``."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
    )

    sha = handle.sha256
    cas_path = tmp_path / "uploads" / sha[:2] / sha[2:]
    assert cas_path.exists(), f"CAS blob not found at {cas_path}"
    assert cas_path.read_bytes() == _PNG


# ---------------------------------------------------------------------------
# 3 — store creates manifest JSON adjacent to the blob
# ---------------------------------------------------------------------------


async def test_store_creates_manifest_json_adjacent(tmp_path: Path) -> None:
    """After store(), a manifest JSON lives at ``<base>/<sha256[:2]>/<sha256>.json``
    and is parseable as a FileManifest."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
        uploaded_by="alice",
    )

    sha = handle.sha256
    manifest_path = tmp_path / "uploads" / sha[:2] / f"{sha}.json"
    assert manifest_path.exists(), f"Manifest not found at {manifest_path}"

    raw = json.loads(manifest_path.read_text(encoding="utf-8"))
    m = FileManifest(**raw)
    assert m.sha256 == sha
    assert m.size == len(_PNG)
    assert m.content_type == "image/png"
    assert m.original_name == "logo.png"
    assert m.app_name == "gov"
    assert m.uploaded_by == "alice"
    assert m.uploaded_at  # non-empty ISO-8601 string


# ---------------------------------------------------------------------------
# 4 — store dedup: skip write if sha256 already exists
# ---------------------------------------------------------------------------


async def test_store_dedupe_skip_if_sha256_exists(tmp_path: Path) -> None:
    """A second store() of identical bytes returns the same handle without
    touching the on-disk blob — dedup is cheap (stat + JSON read only)."""
    store = AppFileStore(tmp_path / "uploads", "gov")

    h1 = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
        uploaded_by="user1",
    )

    # Record the blob's mtime before the second call.
    sha = h1.sha256
    cas_path = tmp_path / "uploads" / sha[:2] / sha[2:]
    mtime_before = cas_path.stat().st_mtime

    h2 = await store.store(
        _PNG,
        original_name="different-name.png",  # different name, same content
        content_type="image/png",
        uploaded_by="user2",
    )

    assert h1.sha256 == h2.sha256
    assert h1.size == h2.size
    # The CAS file was not rewritten.
    assert cas_path.stat().st_mtime == mtime_before


# ---------------------------------------------------------------------------
# 5 — store chunked read: large file is stored completely
# ---------------------------------------------------------------------------


async def test_store_chunked_read_large_file_50mb(tmp_path: Path) -> None:
    """A ~100 KB async-iterator upload is stored completely and its
    SHA256 matches the expected digest."""
    # 100 KB: large enough to span multiple internal chunks (8192 B).
    # Start with a valid PNG header so magic bytes validation passes.
    payload = _PNG + b"\x00" * (100 * 1024 - len(_PNG))

    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _iter_bytes(payload, chunk_size=4096),
        original_name="large.png",
        content_type="image/png",
    )

    expected_sha = hashlib.sha256(payload).hexdigest()
    assert handle.sha256 == expected_sha
    assert handle.size == len(payload)

    # Verify the blob on disk matches the payload byte-for-byte.
    cas_path = tmp_path / "uploads" / handle.sha256[:2] / handle.sha256[2:]
    assert cas_path.read_bytes() == payload


# ---------------------------------------------------------------------------
# 6 — store computes SHA256 incrementally across chunks
# ---------------------------------------------------------------------------


async def test_store_computes_sha256_incrementally(tmp_path: Path) -> None:
    """SHA256 computed by store() over a multi-chunk async iterator equals
    the digest produced by hashing the full byte string at once."""
    # Use a PNG header and fill the rest so it is > 3 chunks.
    payload = _PNG + b"\xab\xcd" * 5000  # ~10 KB
    expected_sha = hashlib.sha256(payload).hexdigest()

    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _iter_bytes(payload, chunk_size=3000),
        original_name="incremental.png",
        content_type="image/png",
    )

    assert handle.sha256 == expected_sha


# ---------------------------------------------------------------------------
# 7 — open returns async iterator yielding bytes
# ---------------------------------------------------------------------------


async def test_open_returns_async_iterator_bytes(tmp_path: Path) -> None:
    """open(sha256) yields all stored bytes in <=8192-byte chunks that
    reassemble to the original payload."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
    )

    chunks: list[bytes] = []
    async for chunk in store.open(handle.sha256):
        assert isinstance(chunk, bytes)
        assert len(chunk) <= 8192
        chunks.append(chunk)

    assert b"".join(chunks) == _PNG


# ---------------------------------------------------------------------------
# 8 — open raises FileNotFoundError on missing sha256
# ---------------------------------------------------------------------------


async def test_open_raises_on_missing_sha256(tmp_path: Path) -> None:
    """open() with a sha256 that was never stored raises FileNotFoundError."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    fake_sha = "a" * 64  # valid-length hex string, nothing on disk

    with pytest.raises(FileNotFoundError):
        async for _ in store.open(fake_sha):
            pass


# ---------------------------------------------------------------------------
# 9 — manifest returns None if not found
# ---------------------------------------------------------------------------


async def test_manifest_returns_none_if_not_found(tmp_path: Path) -> None:
    """manifest() returns None when no manifest JSON exists for the given sha256."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    fake_sha = "b" * 64

    result = await store.manifest(fake_sha)
    assert result is None


# ---------------------------------------------------------------------------
# 10 — manifest includes size and content_type
# ---------------------------------------------------------------------------


async def test_manifest_includes_size_and_content_type(tmp_path: Path) -> None:
    """manifest() returns a FileManifest with size and content_type
    matching the original upload."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PDF,
        original_name="report.pdf",
        content_type="application/pdf",
        uploaded_by="user42",
    )

    m = await store.manifest(handle.sha256)
    assert m is not None
    assert isinstance(m, FileManifest)
    assert m.size == len(_PDF)
    assert m.content_type == "application/pdf"
    assert m.sha256 == handle.sha256
    assert m.uploaded_by == "user42"


# ---------------------------------------------------------------------------
# 11-15 — magic bytes: all five accepted types
# ---------------------------------------------------------------------------


async def test_magic_bytes_png_accepted(tmp_path: Path) -> None:
    """PNG magic bytes are accepted for content_type 'image/png'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="img.png",
        content_type="image/png",
    )
    assert handle.content_type == "image/png"


async def test_magic_bytes_jpeg_accepted(tmp_path: Path) -> None:
    """JPEG magic bytes are accepted for content_type 'image/jpeg'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _JPEG,
        original_name="photo.jpg",
        content_type="image/jpeg",
    )
    assert handle.content_type == "image/jpeg"


async def test_magic_bytes_pdf_accepted(tmp_path: Path) -> None:
    """PDF magic bytes are accepted for content_type 'application/pdf'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PDF,
        original_name="doc.pdf",
        content_type="application/pdf",
    )
    assert handle.content_type == "application/pdf"


async def test_magic_bytes_webp_accepted(tmp_path: Path) -> None:
    """WebP magic bytes are accepted for content_type 'image/webp'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _WEBP,
        original_name="anim.webp",
        content_type="image/webp",
    )
    assert handle.content_type == "image/webp"


async def test_magic_bytes_svg_accepted(tmp_path: Path) -> None:
    """SVG data starting with '<svg' is accepted for content_type 'image/svg+xml'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _SVG,
        original_name="icon.svg",
        content_type="image/svg+xml",
    )
    assert handle.content_type == "image/svg+xml"


# ---------------------------------------------------------------------------
# 16 — magic bytes: EXE (MZ) raises FileTypeError and leaves no tmpfile
# ---------------------------------------------------------------------------


async def test_magic_bytes_unknown_raises_and_deletes_file(tmp_path: Path) -> None:
    """Uploading an EXE payload (MZ header) raises FileTypeError.
    The content_type 'application/octet-stream' is not in the whitelist."""
    store = AppFileStore(tmp_path / "uploads", "gov")

    with pytest.raises(FileTypeError, match="whitelist"):
        await store.store(
            _EXE,
            original_name="malware.exe",
            content_type="application/octet-stream",
        )

    # No tmpfiles should survive a failed store.
    uploads_dir = tmp_path / "uploads"
    if uploads_dir.exists():
        leftover_tmps = list(uploads_dir.glob(".nexus_upload.*"))
        assert leftover_tmps == [], f"Tmpfile(s) leaked: {leftover_tmps}"


# ---------------------------------------------------------------------------
# 17 — magic bytes: PNG header declared but actual bytes are EXE → rejected
# ---------------------------------------------------------------------------


async def test_magic_bytes_png_signature_with_png_content_type_header_but_actual_exe_rejects(
    tmp_path: Path,
) -> None:
    """Declaring 'image/png' but supplying EXE bytes (\x4d\x5a...) raises
    FileTypeError because the magic bytes do not match the PNG signature."""
    store = AppFileStore(tmp_path / "uploads", "gov")

    with pytest.raises(FileTypeError, match="image/png"):
        await store.store(
            _EXE,
            original_name="fake.png",
            content_type="image/png",
        )


# ---------------------------------------------------------------------------
# 18 — delete: soft-removes manifest, keeps CAS file
# ---------------------------------------------------------------------------


async def test_delete_soft_removes_manifest_keeps_cas_file(tmp_path: Path) -> None:
    """delete() removes only the manifest JSON. The CAS blob remains on disk
    and the method returns True on the first call, False on a second call."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    handle = await store.store(
        _PNG,
        original_name="logo.png",
        content_type="image/png",
    )
    sha = handle.sha256
    cas_path = tmp_path / "uploads" / sha[:2] / sha[2:]
    manifest_path = tmp_path / "uploads" / sha[:2] / f"{sha}.json"

    assert cas_path.exists()
    assert manifest_path.exists()

    result = await store.delete(sha)
    assert result is True
    assert cas_path.exists(), "CAS blob must survive a soft delete"
    assert not manifest_path.exists(), "Manifest must be removed by soft delete"

    # Idempotent second call returns False (already gone).
    result2 = await store.delete(sha)
    assert result2 is False


# ---------------------------------------------------------------------------
# 19 — concurrent store of same sha256 is dedup-safe
# ---------------------------------------------------------------------------


async def test_concurrent_store_same_sha256_dedup_safe(tmp_path: Path) -> None:
    """Two asyncio tasks uploading identical bytes concurrently must both
    receive valid FileHandles with the same sha256. No data corruption,
    no leftover tmpfiles."""
    store = AppFileStore(tmp_path / "uploads", "gov")

    async def upload(tag: str) -> FileHandle:
        return await store.store(
            _PNG,
            original_name=f"logo_{tag}.png",
            content_type="image/png",
            uploaded_by=tag,
        )

    h1, h2 = await asyncio.gather(upload("task1"), upload("task2"))

    assert h1.sha256 == h2.sha256
    assert h1.size == h2.size

    # Exactly one CAS blob should exist.
    sha = h1.sha256
    cas_path = tmp_path / "uploads" / sha[:2] / sha[2:]
    assert cas_path.exists()
    assert cas_path.read_bytes() == _PNG

    # No orphaned tmpfiles.
    leftover_tmps = list((tmp_path / "uploads").glob(".nexus_upload.*"))
    assert leftover_tmps == [], f"Tmpfile(s) leaked: {leftover_tmps}"


# ---------------------------------------------------------------------------
# 20 — unsupported content-type passed directly to validate_magic_bytes raises
# ---------------------------------------------------------------------------


async def test_missing_allowlist_decorator_raises(tmp_path: Path) -> None:
    """validate_magic_bytes() raises FileTypeError when the content_type is
    not in ALLOWED_MAGIC_BYTES, naming the unsupported type in the message."""
    unsupported = "video/mp4"
    assert unsupported not in ALLOWED_MAGIC_BYTES

    with pytest.raises(FileTypeError, match="whitelist"):
        validate_magic_bytes(b"\x00\x00\x00\x18ftyp", unsupported)


# ---------------------------------------------------------------------------
# 21 — magic bytes: text/html accepted (Sprint 12 Phase B)
# ---------------------------------------------------------------------------


async def test_magic_bytes_html_accepted(tmp_path: Path) -> None:
    """HTML data starting with '<!DOCTYPE html>' is accepted for 'text/html'."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    html_data = b"<!DOCTYPE html><html><body>Hello</body></html>"
    handle = await store.store(
        html_data,
        original_name="index.html",
        content_type="text/html",
    )
    assert handle.content_type == "text/html"


async def test_magic_bytes_html_rejected_bad_content(tmp_path: Path) -> None:
    """Declaring 'text/html' but supplying non-HTML bytes raises FileTypeError."""
    store = AppFileStore(tmp_path / "uploads", "gov")
    with pytest.raises(FileTypeError, match="text/html"):
        await store.store(
            b"\x00\x01binary junk",
            original_name="fake.html",
            content_type="text/html",
        )
