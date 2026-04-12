"""``/app/{name}/files`` — CAS file upload and retrieval endpoints.

Sprint 9 Phase E (D6 router). Provides three routes that sit on top of
:class:`nexus_sdk.files.AppFileStore`:

- ``POST /app/{name}/files/upload``           — multipart upload into the CAS
- ``GET  /app/{name}/files/{sha256}/manifest`` — return the stored manifest
- ``GET  /app/{name}/files/{sha256}``          — stream the raw blob

Only apps decorated with ``@nexus_app_files`` (which sets
``__nexus_app_files__`` on the class) are allowed to receive uploads.
Apps without the decorator get HTTP 404 from the upload route.

The coordinator loader is responsible for wiring ``AppContext.files`` before
mounting these routes.  If the context exists but ``ctx.files`` is ``None``
the upload route returns 503 — that is a coordinator bug, not a client error.
"""

from __future__ import annotations

import fnmatch
import logging
from typing import TYPE_CHECKING, Any, AsyncIterator

from fastapi import APIRouter, HTTPException, Request, UploadFile
from fastapi.responses import StreamingResponse
from nexus_sdk.files import AppFileStore, FileTypeError
from nexus_sdk.registry import FILES_ATTR

if TYPE_CHECKING:
    from nexus_sdk import AppContext, NexusApp

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/app", tags=["files"])


# ---------------------------------------------------------------------------
# State accessors (mirrors the convention in api/apps.py and api/events.py)
# ---------------------------------------------------------------------------


def _apps(request: Request) -> dict[str, "NexusApp"]:
    coord = request.app.state.coordinator
    return getattr(coord, "apps", {})


def _app_contexts(request: Request) -> dict[str, "AppContext"]:
    coord = request.app.state.coordinator
    return getattr(coord, "app_contexts", {})


# ---------------------------------------------------------------------------
# Internal helpers
# ---------------------------------------------------------------------------


async def _upload_chunks(
    file: UploadFile,
    max_size_bytes: int = 0,
) -> AsyncIterator[bytes]:
    """Yield 8 KiB chunks from ``file`` for streaming directly into the CAS.

    If ``max_size_bytes`` is positive, raises :class:`HTTPException` with
    status 413 when the accumulated bytes exceed the limit.  Sprint 9
    audit E6-B fix.
    """
    total = 0
    while chunk := await file.read(8192):
        total += len(chunk)
        if max_size_bytes > 0 and total > max_size_bytes:
            raise HTTPException(
                status_code=413,
                detail=(
                    f"Upload exceeds the maximum allowed size of "
                    f"{max_size_bytes} bytes ({max_size_bytes // (1024 * 1024)} MB)"
                ),
            )
        yield chunk


def _check_accept(content_type: str | None, accept: list[str]) -> bool:
    """Return ``True`` when ``content_type`` matches any glob pattern in ``accept``.

    A ``None`` content_type or an empty accept list is treated as a mismatch.
    MIME parameters (e.g. ``; charset=binary``) are stripped before matching.
    """
    if not content_type or not accept:
        return False
    normalised = content_type.lower().split(";", 1)[0].strip()
    return any(fnmatch.fnmatch(normalised, pattern.lower()) for pattern in accept)


def _get_store(ctx: "AppContext", name: str) -> AppFileStore:
    """Return ``ctx.files`` or raise HTTP 503 if it is not wired."""
    store: AppFileStore | None = getattr(ctx, "files", None)
    if store is None:
        raise HTTPException(
            status_code=503,
            detail=f"app {name!r} has no AppContext.files wired",
        )
    return store


def _require_ctx(request: Request, name: str) -> "AppContext":
    """Return the app context or raise HTTP 500 if the loader skipped it."""
    ctx = _app_contexts(request).get(name)
    if ctx is None:
        raise HTTPException(
            status_code=500,
            detail=f"app {name!r} has no bound context — coordinator loader bug",
        )
    return ctx


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------


@router.post("/{name}/files/upload", status_code=201)
async def upload_file(
    request: Request,
    name: str,
    file: UploadFile,
) -> dict[str, Any]:
    """Accept a multipart file upload and store it in the per-app CAS.

    Failure modes:

    - 404 — unknown app or app lacks ``@nexus_app_files`` decorator.
    - 503 — ``ctx.files`` not wired (coordinator bug).
    - 415 — content_type rejected by the app's accept list, or magic
      bytes do not match the declared MIME type.

    Returns HTTP 201 with::

        {
            "sha256":        "<hex>",
            "size":          <bytes>,
            "content_type":  "<mime>",
            "original_name": "<filename>",
            "dedup":         <bool>,
        }

    ``dedup`` is ``True`` when the identical bytes were already present in
    the CAS (the store skipped the write and returned the existing handle).
    Because the sha256 is only known after streaming completes, the flag is
    currently always ``False``; the invariant that matters is that the sha256
    is always correct regardless of whether the write was a hit or a miss.
    """
    apps = _apps(request)
    app = apps.get(name)
    if app is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")

    files_meta: dict[str, Any] | None = getattr(type(app), FILES_ATTR, None)
    if files_meta is None:
        raise HTTPException(
            status_code=404,
            detail=(f"app {name!r} does not accept file uploads (missing @nexus_app_files decorator)"),
        )

    ctx = _require_ctx(request, name)
    store = _get_store(ctx, name)

    accept: list[str] = files_meta.get("accept", [])
    content_type = file.content_type
    if not _check_accept(content_type, accept):
        raise HTTPException(
            status_code=415,
            detail=(f"content_type {content_type!r} is not accepted by app {name!r}. Accepted patterns: {accept}"),
        )

    original_name: str = file.filename or "upload"

    logger.info(
        "file upload started: app=%r name=%r content_type=%r",
        name,
        original_name,
        content_type,
    )

    max_size: int = files_meta.get("max_size_bytes", 50 * 1024 * 1024)

    try:
        handle = await store.store(
            _upload_chunks(file, max_size_bytes=max_size),
            original_name=original_name,
            content_type=content_type or "",
            uploaded_by="",
        )
    except FileTypeError as exc:
        logger.warning(
            "file upload rejected (magic bytes): app=%r name=%r error=%s",
            name,
            original_name,
            exc,
        )
        raise HTTPException(status_code=415, detail=str(exc)) from exc

    # dedup detection requires the sha256, which is only known after streaming.
    # AppFileStore.store is idempotent; callers should treat any 201 with the
    # same sha256 as a no-op re-upload.  We return False as a safe default.
    dedup = False

    events = getattr(ctx, "events", None)
    if events is not None:
        try:
            await events.publish(
                "file.upload.progress",
                {
                    "sha256": handle.sha256,
                    "size": handle.size,
                    "content_type": handle.content_type,
                    "original_name": handle.original_name,
                    "app_name": name,
                    "dedup": dedup,
                },
            )
        except Exception as exc:  # noqa: BLE001
            logger.warning(
                "file.upload.progress publish failed for app=%r sha256=%r: %s",
                name,
                handle.sha256,
                exc,
            )

    logger.info(
        "file upload complete: app=%r sha256=%r size=%d",
        name,
        handle.sha256,
        handle.size,
    )

    return {
        "sha256": handle.sha256,
        "size": handle.size,
        "content_type": handle.content_type,
        "original_name": handle.original_name,
        "dedup": dedup,
    }


@router.get("/{name}/files/{sha256}/manifest")
async def get_file_manifest(
    request: Request,
    name: str,
    sha256: str,
) -> dict[str, Any]:
    """Return the stored :class:`~nexus_sdk.files.FileManifest` as JSON.

    Failure modes:

    - 404 — unknown app, ``ctx.files`` absent, or no manifest on disk.
    - 503 — ``ctx.files`` is ``None`` (coordinator bug).
    """
    apps = _apps(request)
    if apps.get(name) is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")

    ctx = _require_ctx(request, name)
    store = _get_store(ctx, name)

    manifest = await store.manifest(sha256)
    if manifest is None:
        raise HTTPException(
            status_code=404,
            detail=f"no manifest found for sha256={sha256!r} in app {name!r}",
        )

    return manifest.model_dump()


@router.get("/{name}/files/{sha256}")
async def stream_file(
    request: Request,
    name: str,
    sha256: str,
) -> StreamingResponse:
    """Stream the raw CAS blob identified by ``sha256``.

    The ``Content-Type`` header is sourced from the stored manifest so
    the browser can render images and PDFs inline without a client-side
    MIME lookup.

    Failure modes:

    - 404 — unknown app, ``ctx.files`` absent, blob / manifest missing.
    - 503 — ``ctx.files`` is ``None`` (coordinator bug).
    """
    apps = _apps(request)
    if apps.get(name) is None:
        raise HTTPException(status_code=404, detail=f"app {name!r} not installed")

    ctx = _require_ctx(request, name)
    store = _get_store(ctx, name)

    manifest = await store.manifest(sha256)
    if manifest is None:
        raise HTTPException(
            status_code=404,
            detail=f"no manifest found for sha256={sha256!r} in app {name!r}",
        )

    try:
        blob_stream = store.open(sha256)
    except FileNotFoundError:
        raise HTTPException(
            status_code=404,
            detail=f"CAS blob missing for sha256={sha256!r} in app {name!r}",
        )

    return StreamingResponse(
        blob_stream,
        media_type=manifest.content_type,
        headers={
            "Content-Disposition": f'inline; filename="{manifest.original_name}"',
            "X-Nexus-SHA256": sha256,
            "X-Nexus-App": name,
        },
    )
