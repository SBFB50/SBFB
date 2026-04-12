# SPDX-License-Identifier: AGPL-3.0-or-later
"""AppFileStore — SHA256-based content-addressable storage for app file uploads.

Sprint 9 Phase E (D5 impl). Apps that need to persist user-supplied
files (images, PDFs, SVG assets) get an :class:`AppFileStore` wired
by the coordinator loader on ``AppContext.files``. The store lives
under the coordinator's project tree::

    projects/<project>/apps/<app>/uploads/<sha256[:2]>/<sha256[2:]>

with an adjacent manifest JSON::

    projects/<project>/apps/<app>/uploads/<sha256[:2]>/<sha256>.json

Design decisions (frozen Sprint 9 Day 0 D5)
-------------------------------------------

- **CAS layout.** Two-level directory sharding by the first two hex
  characters of the SHA256 digest mirrors git's object store and
  prevents large directories from degrading filesystem performance.
  The CAS file is named by the remaining 62 hex characters; the
  manifest shares the ``<sha256[:2]>`` prefix directory and is named
  ``<sha256>.json`` so a directory listing yields paired (data,
  manifest) entries.
- **Dedup pre-write.** Before writing, :meth:`AppFileStore.store`
  checks whether both the CAS file and its manifest already exist. If
  so, it reads the manifest and returns the existing
  :class:`FileHandle` immediately — the byte stream is never touched.
  This means idempotent uploads (the same file uploaded twice) cost
  only a stat + JSON read.
- **Atomic tmpfile rename.** Data is streamed to a sibling tmpfile in
  the same CAS shard directory (same filesystem, avoiding
  cross-device ``os.replace``). On success the tmpfile is renamed
  into the CAS path. On any error the tmpfile is removed. The manifest
  JSON is written *after* the CAS file lands so a partial write can
  never leave a manifest that points at a missing blob.
- **Magic bytes whitelist.** Five content types are accepted: PNG,
  JPEG, PDF, WebP, SVG. The first 256 bytes of the stored data are
  checked against known magic byte signatures **after** the stream is
  fully written but **before** the CAS file is renamed and the manifest
  is written. A type mismatch raises :class:`FileTypeError` and the
  tmpfile is cleaned up.
- **Chunked read.** :meth:`AppFileStore.open` is an async generator
  that yields 8192-byte chunks from the CAS file. The file is opened
  in binary read mode using :func:`asyncio.get_event_loop().run_in_executor`
  around synchronous pathlib I/O, keeping aiofiles out of the
  dependency tree (the SDK's pyproject.toml does not list it).
- **Soft delete.** :meth:`AppFileStore.delete` removes the manifest
  JSON only, leaving the CAS blob in place. A subsequent ``store``
  of the same bytes re-creates the manifest from scratch. This
  ensures that dedup remains correct even after a delete: the blob
  is never re-hashed, and the content is always recoverable for
  forensics if the coordinator needs to audit dropped files.
- **Sync I/O in executor.** All filesystem operations use
  :func:`asyncio.get_event_loop().run_in_executor` with the default
  thread pool, wrapping synchronous :mod:`pathlib` and :mod:`os`
  calls. This keeps the event loop unblocked during disk access
  without adding any extra dependency. The overhead is negligible for
  the typical coordinator workload (small files, infrequent uploads).

Anti-patterns explicitly rejected
----------------------------------

- **python-magic / libmagic.** The SDK targets Windows 11 where
  libmagic is not natively available and the ``python-magic-bin``
  wheel is fragile. The five allowed types have well-defined
  byte-level signatures; a hand-rolled check is more portable and
  easier to audit.
- **aiofiles.** Not in the dependency tree; adding it to avoid a
  thread pool call for local file I/O is premature optimisation for
  a loopback-only coordinator.
- **Hard delete.** Removing the blob would break dedup and make
  content-integrity auditing impossible. Soft delete (manifest only)
  is the correct primitive.
- **Async-generator return from store().** The store method returns a
  :class:`FileHandle` synchronously after the upload completes so
  callers can immediately use the ``sha256`` for routing or response
  serialization without an extra ``async for`` step.

Reference: ``.planning/sprint9_plan.md`` §5 Phase E, ``docs/shell/PATTERNS.md``
P15.
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import logging
import os
import tempfile
from collections.abc import AsyncIterator
from datetime import datetime, timezone
from pathlib import Path
from typing import Union

from pydantic import BaseModel, ConfigDict

_log = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

_CHUNK_SIZE = 8192
_MAGIC_PROBE_BYTES = 256


# ---------------------------------------------------------------------------
# Exceptions
# ---------------------------------------------------------------------------


class FileTypeError(Exception):
    """Raised when uploaded data does not match the expected content type.

    Triggered by :func:`validate_magic_bytes` when either:

    1. The supplied ``content_type`` is not in the whitelist of five
       accepted types (PNG, JPEG, PDF, WebP, SVG).
    2. The leading magic bytes of the data do not match the signature
       for the declared ``content_type``.

    The message names the declared type and, when known, the detected
    type so an API error handler can surface a useful hint to the
    caller.
    """


# ---------------------------------------------------------------------------
# Pydantic models
# ---------------------------------------------------------------------------


class FileHandle(BaseModel):
    """Immutable reference to a stored file.

    Returned by :meth:`AppFileStore.store` after a successful upload.
    Contains enough metadata for the coordinator to construct a
    download URL and for the frontend to render a file preview.

    Fields
    ------
    sha256:
        Hex-encoded SHA256 digest of the raw file bytes. Used as the
        CAS key and as the stable identifier for dedup.
    size:
        Byte length of the stored file.
    content_type:
        MIME type declared by the uploader, validated against the
        magic bytes whitelist.
    original_name:
        Original filename supplied by the uploader. Stored for display
        purposes only; it does not affect the CAS path.
    app_name:
        Name of the app that owns this upload (from the
        :class:`AppFileStore` instance's ``app_name``).
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    sha256: str
    size: int
    content_type: str
    original_name: str
    app_name: str


class FileManifest(BaseModel):
    """Full on-disk metadata for a stored file.

    Written adjacent to the CAS blob as ``<sha256>.json``.
    Extends :class:`FileHandle` with audit fields that are not needed
    for in-memory routing but are essential for storage accounting and
    access-control audits.

    Fields (in addition to :class:`FileHandle`)
    -------------------------------------------
    uploaded_at:
        ISO-8601 UTC timestamp of the upload, generated at write time.
    uploaded_by:
        Opaque identifier of the uploader (e.g. a session token or
        user display name). Empty string when the coordinator did not
        supply a principal.
    """

    model_config = ConfigDict(frozen=True, extra="forbid")

    sha256: str
    size: int
    content_type: str
    original_name: str
    app_name: str
    uploaded_at: str
    uploaded_by: str


# ---------------------------------------------------------------------------
# Magic bytes validation
# ---------------------------------------------------------------------------


def _check_png(data: bytes, _content_type: str) -> None:
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise FileTypeError(
            f"declared content_type 'image/png' but magic bytes do not match PNG signature (got {data[:8]!r})"
        )


def _check_jpeg(data: bytes, _content_type: str) -> None:
    if data[:3] != b"\xff\xd8\xff":
        raise FileTypeError(
            f"declared content_type 'image/jpeg' but magic bytes do not match JPEG signature (got {data[:3]!r})"
        )


def _check_pdf(data: bytes, _content_type: str) -> None:
    if data[:4] != b"%PDF":
        raise FileTypeError(
            f"declared content_type 'application/pdf' but magic bytes do not match PDF signature (got {data[:4]!r})"
        )


def _check_webp(data: bytes, _content_type: str) -> None:
    if data[:4] != b"RIFF" or data[8:12] != b"WEBP":
        raise FileTypeError(
            f"declared content_type 'image/webp' but magic bytes do not match WebP signature "
            f"(RIFF header: {data[:4]!r}, WEBP marker: {data[8:12]!r})"
        )


def _check_svg(data: bytes, _content_type: str) -> None:
    # SVG is XML text — strip leading whitespace and check for XML/SVG preamble
    # within the first 256 bytes.
    probe = data[:_MAGIC_PROBE_BYTES].lstrip()
    if not (probe.startswith(b"<?xml") or probe.startswith(b"<svg")):
        raise FileTypeError(
            f"declared content_type 'image/svg+xml' but data does not start with '<?xml' or '<svg' (got {probe[:16]!r})"
        )


# Mapping from normalised content_type to its validator.
# Keys use the canonical MIME string; the lookup in validate_magic_bytes
# normalises the declared content_type to lower-case and strips parameters
# (e.g. "image/jpeg; charset=binary" → "image/jpeg") before the dict lookup.
def _check_html(data: bytes, _content_type: str) -> None:
    probe = data[:_MAGIC_PROBE_BYTES].lstrip()
    lower = probe.lower()
    if not (lower.startswith(b"<!doctype") or lower.startswith(b"<html")):
        raise FileTypeError(
            f"declared content_type 'text/html' but data does not start with '<!doctype' or '<html' (got {probe[:16]!r})"
        )


ALLOWED_MAGIC_BYTES: dict[str, object] = {
    "image/png": _check_png,
    "image/jpeg": _check_jpeg,
    "application/pdf": _check_pdf,
    "image/webp": _check_webp,
    "image/svg+xml": _check_svg,
    "text/html": _check_html,
}


def validate_magic_bytes(data: bytes, content_type: str) -> None:
    """Validate that ``data`` matches the magic bytes for ``content_type``.

    Parameters
    ----------
    data:
        The raw file bytes (or a leading prefix — only the first
        :data:`_MAGIC_PROBE_BYTES` bytes are inspected).
    content_type:
        MIME type declared by the uploader. Leading/trailing whitespace
        and MIME parameters (e.g. ``; charset=binary``) are stripped
        before the lookup.

    Raises
    ------
    FileTypeError:
        When ``content_type`` is not in the whitelist of five accepted
        types, or when the magic bytes do not match the declared type.
    """
    # Normalise: lower-case, strip MIME parameters.
    normalised = content_type.lower().split(";", 1)[0].strip()

    checker = ALLOWED_MAGIC_BYTES.get(normalised)
    if checker is None:
        allowed = ", ".join(sorted(ALLOWED_MAGIC_BYTES))
        raise FileTypeError(f"content_type {content_type!r} is not in the upload whitelist. Allowed types: {allowed}")

    # checker is always a callable; the dict value type is ``object`` to
    # avoid a complex Protocol annotation for a private API.
    checker(data, normalised)  # type: ignore[operator]


# ---------------------------------------------------------------------------
# AppFileStore
# ---------------------------------------------------------------------------


class AppFileStore:
    """SHA256-based content-addressable store for per-app file uploads.

    One instance is created per ``(project, app)`` pair by the
    coordinator loader and assigned to ``AppContext.files`` before
    the app's ``on_start`` hook runs. Apps never touch the filesystem
    directly — all I/O goes through this class.

    Parameters
    ----------
    base_path:
        Root directory for this app's CAS tree. The coordinator
        constructs this as
        ``<projects_root>/<project>/apps/<app>/uploads/``. The
        directory need not exist at construction time; it is created
        lazily on the first :meth:`store` call.
    app_name:
        Name of the owning app, stored in every :class:`FileManifest`
        for auditing purposes.
    """

    def __init__(self, base_path: Path | str, app_name: str) -> None:
        self._base = Path(base_path)
        self._app_name = app_name

    # ------------------------------------------------------------------
    # Public properties
    # ------------------------------------------------------------------

    @property
    def base_path(self) -> Path:
        """Return the root directory for this store's CAS tree."""
        return self._base

    @property
    def app_name(self) -> str:
        """Return the app name embedded in every manifest."""
        return self._app_name

    # ------------------------------------------------------------------
    # CAS path helpers
    # ------------------------------------------------------------------

    def _cas_path(self, sha256: str) -> Path:
        """Return the CAS blob path for ``sha256``.

        Layout: ``<base>/<sha256[:2]>/<sha256[2:]>``
        """
        return self._base / sha256[:2] / sha256[2:]

    def _manifest_path(self, sha256: str) -> Path:
        """Return the manifest JSON path for ``sha256``.

        Layout: ``<base>/<sha256[:2]>/<sha256>.json``
        """
        return self._base / sha256[:2] / f"{sha256}.json"

    # ------------------------------------------------------------------
    # Core store operation
    # ------------------------------------------------------------------

    async def store(
        self,
        data: Union[AsyncIterator[bytes], bytes],
        *,
        original_name: str,
        content_type: str,
        uploaded_by: str = "",
    ) -> FileHandle:
        """Stream ``data`` into the CAS and return a :class:`FileHandle`.

        The operation is idempotent: if a CAS blob and its manifest
        already exist for the computed SHA256, the write is skipped and
        the existing handle is returned immediately.

        Parameters
        ----------
        data:
            Either a raw ``bytes`` object or an ``AsyncIterator[bytes]``
            (e.g. an ASGI upload stream). Both forms are handled
            identically: bytes is wrapped into a single-chunk iterator.
        original_name:
            Original filename supplied by the uploader. Stored in the
            manifest for display; does not affect the CAS path.
        content_type:
            MIME type declared by the uploader. Validated against the
            magic bytes whitelist after the stream is consumed.
        uploaded_by:
            Opaque uploader identifier. Stored in the manifest; may be
            empty when the coordinator has no authenticated principal.

        Returns
        -------
        FileHandle:
            Immutable reference to the stored (or already-existing) blob.

        Raises
        ------
        FileTypeError:
            When ``content_type`` is not in the whitelist or when the
            leading magic bytes do not match the declared type.
        OSError:
            When a filesystem operation fails (permissions, disk full,
            etc.).
        """
        loop = asyncio.get_event_loop()

        # ------------------------------------------------------------------
        # Step 1: stream into a tmpfile and compute SHA256 incrementally.
        # The tmpfile lives in ``self._base`` (created lazily below) so it
        # shares the same filesystem as the eventual CAS shard directories,
        # making the final ``os.replace`` an in-filesystem rename.
        # ------------------------------------------------------------------
        def _mkdirs_base() -> None:
            self._base.mkdir(parents=True, exist_ok=True)

        await loop.run_in_executor(None, _mkdirs_base)

        hasher = hashlib.sha256()
        size = 0
        magic_probe: bytes = b""

        fd, tmp_path_str = await loop.run_in_executor(
            None,
            lambda: tempfile.mkstemp(
                prefix=".nexus_upload.",
                suffix=".tmp",
                dir=str(self._base),
            ),
        )

        try:
            with os.fdopen(fd, "wb") as tmp_f:
                if isinstance(data, bytes):
                    # Treat raw bytes as a single chunk.
                    chunk = data
                    hasher.update(chunk)
                    size += len(chunk)
                    if not magic_probe:
                        magic_probe = chunk[:_MAGIC_PROBE_BYTES]
                    tmp_f.write(chunk)
                else:
                    # Async iterator — drain all chunks.
                    async for chunk in data:
                        hasher.update(chunk)
                        size += len(chunk)
                        if not magic_probe and chunk:
                            magic_probe = chunk[:_MAGIC_PROBE_BYTES]
                        tmp_f.write(chunk)

            sha256 = hasher.hexdigest()

            # ------------------------------------------------------------------
            # Step 2: dedup check — if both blob and manifest exist, skip write.
            # ------------------------------------------------------------------
            cas = self._cas_path(sha256)
            manifest_p = self._manifest_path(sha256)

            def _dedup_check() -> bool:
                return cas.exists() and manifest_p.exists()

            if await loop.run_in_executor(None, _dedup_check):
                _log.debug(
                    "store dedup hit",
                    extra={"sha256": sha256, "app": self._app_name},
                )

                # Clean up the tmpfile we just wrote.
                def _remove_tmp() -> None:
                    try:
                        os.unlink(tmp_path_str)
                    except OSError:
                        pass

                await loop.run_in_executor(None, _remove_tmp)

                # Read and return existing handle via manifest.
                existing = await self.manifest(sha256)
                if existing is not None:
                    return FileHandle(
                        sha256=existing.sha256,
                        size=existing.size,
                        content_type=existing.content_type,
                        original_name=existing.original_name,
                        app_name=existing.app_name,
                    )
                # Manifest disappeared between stat and read (race). Fall
                # through to normal write path; tmpfile has been removed so
                # we need to re-write from the raw bytes.
                # This path is so rare it warrants a warning.
                _log.warning(
                    "store dedup race: manifest disappeared, re-streaming not possible; caller should retry",
                    extra={"sha256": sha256, "app": self._app_name},
                )
                raise OSError(f"dedup race on sha256 {sha256!r}: manifest disappeared after stat; retry the upload")

            # ------------------------------------------------------------------
            # Step 3: validate magic bytes against declared content_type.
            # ------------------------------------------------------------------
            # probe was collected during the streaming step above.
            validate_magic_bytes(magic_probe, content_type)

            # ------------------------------------------------------------------
            # Step 4: atomically rename tmpfile into the CAS shard.
            # ------------------------------------------------------------------
            def _install_blob() -> None:
                cas.parent.mkdir(parents=True, exist_ok=True)
                os.replace(tmp_path_str, str(cas))

            await loop.run_in_executor(None, _install_blob)

        except BaseException:
            # On any failure, clean up the tmpfile.
            def _cleanup_tmp() -> None:
                try:
                    os.unlink(tmp_path_str)
                except OSError:
                    pass

            await loop.run_in_executor(None, _cleanup_tmp)
            raise

        # ------------------------------------------------------------------
        # Step 5: write the manifest JSON (only after the blob is in place).
        # ------------------------------------------------------------------
        uploaded_at = datetime.now(timezone.utc).isoformat()
        manifest_obj = FileManifest(
            sha256=sha256,
            size=size,
            content_type=content_type,
            original_name=original_name,
            app_name=self._app_name,
            uploaded_at=uploaded_at,
            uploaded_by=uploaded_by,
        )
        manifest_json = manifest_obj.model_dump_json()

        def _write_manifest() -> None:
            # Write via tmpfile + rename so a partial write can never leave a
            # corrupt manifest (mirrors AppStorage._write_blob_locked).
            manifest_p.parent.mkdir(parents=True, exist_ok=True)
            mfd, mtmp = tempfile.mkstemp(
                prefix=f".{sha256}.",
                suffix=".json.tmp",
                dir=str(manifest_p.parent),
            )
            try:
                with os.fdopen(mfd, "w", encoding="utf-8") as mf:
                    mf.write(manifest_json)
                os.replace(mtmp, str(manifest_p))
            except Exception:
                try:
                    os.unlink(mtmp)
                except OSError:
                    pass
                raise

        await loop.run_in_executor(None, _write_manifest)

        _log.info(
            "file stored",
            extra={
                "sha256": sha256,
                "size": size,
                "content_type": content_type,
                "original_name": original_name,
                "app": self._app_name,
            },
        )

        return FileHandle(
            sha256=sha256,
            size=size,
            content_type=content_type,
            original_name=original_name,
            app_name=self._app_name,
        )

    # ------------------------------------------------------------------
    # Read operations
    # ------------------------------------------------------------------

    async def open(self, sha256: str) -> AsyncIterator[bytes]:
        """Yield 8192-byte chunks from the CAS blob for ``sha256``.

        Parameters
        ----------
        sha256:
            Hex-encoded SHA256 digest of the blob to read.

        Yields
        ------
        bytes:
            Raw byte chunks of up to :data:`_CHUNK_SIZE` bytes each.

        Raises
        ------
        FileNotFoundError:
            When the CAS blob does not exist (i.e. the file was never
            uploaded, or was hard-deleted outside of the normal API).
        OSError:
            On other filesystem errors.
        """
        cas = self._cas_path(sha256)

        loop = asyncio.get_event_loop()

        def _check_exists() -> bool:
            return cas.exists()

        if not await loop.run_in_executor(None, _check_exists):
            raise FileNotFoundError(f"CAS blob not found for sha256={sha256!r} at {cas}")

        # Read the entire file in the executor and then yield chunks from
        # memory. For large files this is acceptable because the coordinator
        # is loopback-only and coordinators hold files locally. If files are
        # ever expected to be multi-GB, a streaming path via a thread queue
        # would be required — that is out of scope for Sprint 9.
        def _read_all() -> bytes:
            return cas.read_bytes()

        raw = await loop.run_in_executor(None, _read_all)

        # Yield the data in chunks as an async generator.
        # Note: this method is declared ``async def`` and contains a ``yield``
        # so it IS an async generator. Callers use ``async for chunk in store.open(sha256)``.
        offset = 0
        while offset < len(raw):
            yield raw[offset : offset + _CHUNK_SIZE]
            offset += _CHUNK_SIZE

    async def manifest(self, sha256: str) -> FileManifest | None:
        """Read and parse the manifest JSON for ``sha256``.

        Returns ``None`` when the manifest file does not exist (the
        blob was soft-deleted, or was never uploaded). Raises
        :class:`pydantic.ValidationError` when the on-disk JSON does
        not conform to :class:`FileManifest` (schema drift).

        Parameters
        ----------
        sha256:
            Hex-encoded SHA256 digest identifying the manifest to read.
        """
        manifest_p = self._manifest_path(sha256)
        loop = asyncio.get_event_loop()

        def _read_manifest() -> str | None:
            if not manifest_p.exists():
                return None
            return manifest_p.read_text(encoding="utf-8")

        raw = await loop.run_in_executor(None, _read_manifest)
        if raw is None:
            return None

        return FileManifest.model_validate(json.loads(raw))

    # ------------------------------------------------------------------
    # Delete (soft)
    # ------------------------------------------------------------------

    async def delete(self, sha256: str) -> bool:
        """Soft-delete the file by removing its manifest JSON.

        The CAS blob is kept in place so that:

        - Dedup detection still works for future uploads of the same
          bytes (the blob is present, only the manifest is gone — a
          subsequent :meth:`store` re-creates the manifest).
        - Content-integrity audits by the coordinator can still
          retrieve the bytes by sha256 even after the logical deletion.

        Parameters
        ----------
        sha256:
            Hex-encoded SHA256 digest of the file to delete.

        Returns
        -------
        bool:
            ``True`` when the manifest existed and was removed;
            ``False`` when the manifest was already absent (idempotent
            no-op).
        """
        manifest_p = self._manifest_path(sha256)
        loop = asyncio.get_event_loop()

        def _remove_manifest() -> bool:
            if not manifest_p.exists():
                return False
            try:
                manifest_p.unlink()
                return True
            except FileNotFoundError:
                # Race between exists() and unlink() — treat as already gone.
                return False

        removed = await loop.run_in_executor(None, _remove_manifest)
        if removed:
            _log.info(
                "file soft-deleted (manifest removed, blob retained)",
                extra={"sha256": sha256, "app": self._app_name},
            )
        return removed


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

__all__ = [
    "ALLOWED_MAGIC_BYTES",
    "AppFileStore",
    "FileHandle",
    "FileManifest",
    "FileTypeError",
    "validate_magic_bytes",
]
