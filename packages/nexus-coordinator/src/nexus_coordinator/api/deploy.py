# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 12 Phase B — ``POST /project/deploy`` endpoint.

Upload a zip archive containing a web app and publish it to the P2P
network. The coordinator:

1. Reads the zip bytes from the upload.
2. Validates that the zip contains an ``index.html``.
3. Stores the zip as an iroh blob via the daemon's
   ``POST /publish-blob``.
4. Publishes a v2 project announcement with ``archive_hash`` via
   the daemon's ``POST /publish``.

The endpoint returns the BLAKE3 hash of the stored blob so the
caller can construct a ``GET /blob-serve/{hash}/index.html`` URL.
"""

from __future__ import annotations

import io
import zipfile

import httpx
import structlog
from fastapi import APIRouter, HTTPException, Request, UploadFile

from nexus_coordinator.api.daemon import _daemon_base_url, _read_running_state

_log = structlog.get_logger(__name__)

router = APIRouter(tags=["deploy"])

# 100 MB — consistent with blob-serve daemon DEFAULT_MAX_DECOMPRESSED_BYTES.
MAX_DEPLOY_BYTES: int = 100 * 1024 * 1024


def _validate_zip(data: bytes) -> None:
    """Raise HTTPException(400) if ``data`` is not a valid zip with index.html."""
    try:
        with zipfile.ZipFile(io.BytesIO(data), "r") as zf:
            names = zf.namelist()
    except (zipfile.BadZipFile, Exception) as e:
        raise HTTPException(status_code=400, detail=f"invalid zip archive: {e}") from e

    if "index.html" not in names:
        raise HTTPException(
            status_code=400,
            detail="zip archive must contain an index.html at the root",
        )


async def _store_blob(request: Request, zip_bytes: bytes) -> str:
    """Store bytes as a blob via the daemon's POST /publish-blob.

    Returns the hex hash of the stored blob.
    """
    state = _read_running_state()
    if state is None:
        raise HTTPException(status_code=503, detail="shell-daemon not running")

    url = f"{_daemon_base_url(state)}/publish-blob"
    client: httpx.AsyncClient = request.app.state.daemon_httpx_client
    try:
        resp = await client.post(
            url,
            content=zip_bytes,
            headers={"Content-Type": "application/octet-stream"},
        )
    except httpx.HTTPError as e:
        raise HTTPException(status_code=503, detail=f"daemon unreachable: {e}") from e

    if resp.status_code != 200:
        raise HTTPException(
            status_code=502,
            detail=f"daemon /publish-blob returned {resp.status_code}: {resp.text}",
        )

    body = resp.json()
    return body["hash"]


async def _publish_with_archive(request: Request, hash_hex: str) -> None:
    """Publish a v2 announcement with the archive hash."""
    coord = request.app.state.coordinator
    state = _read_running_state()
    if state is None:
        _log.warning("publish skipped: daemon not running")
        return

    url = f"{_daemon_base_url(state)}/publish"
    payload = {
        "project_name": coord.project_name,
        "category": coord.config.identity.description or "general",
        "description": coord.config.identity.description or coord.project_name,
        "apps": list(coord.apps.keys()),
        "archive_hash": hash_hex,
    }
    client: httpx.AsyncClient = request.app.state.daemon_httpx_client
    try:
        resp = await client.post(url, json=payload)
        if resp.status_code != 200:
            _log.warning("publish returned non-200", status=resp.status_code)
    except httpx.HTTPError as e:
        _log.warning("publish failed", error=str(e))


@router.post("/project/deploy")
async def deploy_project(
    archive: UploadFile,
    request: Request,
) -> dict:
    """Upload a zip archive and publish to the P2P network."""
    zip_bytes = await archive.read()
    if len(zip_bytes) > MAX_DEPLOY_BYTES:
        raise HTTPException(
            status_code=413,
            detail=(
                f"Upload exceeds the maximum allowed size of "
                f"{MAX_DEPLOY_BYTES} bytes ({MAX_DEPLOY_BYTES // (1024 * 1024)} MB)"
            ),
        )
    _log.info("deploy: received zip", size=len(zip_bytes))

    _validate_zip(zip_bytes)

    hash_hex = await _store_blob(request, zip_bytes)
    _log.info("deploy: blob stored", hash=hash_hex)

    await _publish_with_archive(request, hash_hex)

    return {"deployed": True, "hash": hash_hex}
