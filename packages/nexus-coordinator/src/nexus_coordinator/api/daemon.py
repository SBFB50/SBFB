# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 7 Phase E — ``/daemon/*`` proxy endpoints.

The React shell reaches into the local ``nexus-shell-daemon``
binary exclusively through this coordinator proxy. Sprint 7 D1
(HTTP loopback via coordinator proxy) freezes that contract:
the browser never learns the daemon's direct URL, every call
goes ``shell → coordinator → daemon``, and the coordinator's
existing CORS layer is the single trust boundary.

The daemon writes a ``running.json`` atomically on boot at
``~/.nexus-grid/shell-daemon/running.json``. Each request here:

1. Reads the file via :func:`_read_running_state`.
2. Constructs ``http://<api_host>:<api_port>`` as the daemon
   base URL.
3. Forwards the request through an :class:`httpx.AsyncClient`
   with a short connect timeout and a reasonable read timeout.
4. Maps transport failures to a ``503`` response with a typed
   ``DaemonUnavailable`` JSON body. The React shell treats that
   as "daemon offline" and renders a CTA to start it, rather
   than showing a spinner forever.

Discriminated response pattern: every success path returns
``{"kind": "data", ...}`` and every failure returns
``{"kind": "unavailable", "reason": "..."}``. The TypeScript
Zod schema on the shell side dispatches on ``kind`` so missing
daemon is part of the normal UX, not an error state.
"""

from __future__ import annotations

import json
from typing import Any, Literal

import httpx
import structlog
from fastapi import APIRouter, Request
from fastapi.responses import JSONResponse
from pydantic import BaseModel, Field, ValidationError

from nexus_coordinator import paths as _paths

_log = structlog.get_logger(__name__)

router = APIRouter(prefix="/daemon", tags=["daemon"])

# Sprint 9 Phase A (T10) — the connect + read deadlines are now
# owned by the ``app.state.daemon_httpx_client`` singleton the
# FastAPI ``lifespan`` constructs in
# :mod:`nexus_coordinator.api.app`. Per-call ``async with
# httpx.AsyncClient(...)`` is gone: every proxy handler reaches
# the shared client via ``request.app.state.daemon_httpx_client``
# so that a burst of shell refreshes shares the same connection
# pool (``Limits(max_connections=10)``). The previous per-call
# construction re-did the TCP + TLS handshake on every request
# which, while cheap on loopback, was unbounded under burst.


# =================================================================
# RunningState shape as written by the Rust daemon
# =================================================================


class DaemonRunningState(BaseModel):
    """Mirror of ``nexus_shell_daemon_core::registry::RunningState``.

    **Not** the same shape as the coordinator's own
    :class:`RunningState` — the shell daemon is global per user
    so there is no ``project_name`` / ``visibility`` field, and
    the daemon carries its compile-time version so the shell
    can detect a mismatched binary.
    """

    schema_version: Literal[1]
    node_id: str = Field(..., description="Ed25519 pubkey hex, 64 chars lowercase.")
    api_host: str
    api_port: int = Field(..., ge=1, le=65535)
    pid: int = Field(..., ge=1)
    started_at: str
    daemon_version: str


def _read_running_state() -> DaemonRunningState | None:
    """Best-effort read of the daemon's ``running.json``.

    Returns ``None`` if the file does not exist, is unreadable,
    is malformed JSON, or fails schema validation. Each failure
    mode is logged at warn level so operators can diagnose a
    broken install without attaching a debugger.
    """
    path = _paths.shell_daemon_registry_path()
    if not path.exists():
        return None
    try:
        body = path.read_text(encoding="utf-8")
    except OSError as e:
        _log.warning("cannot read shell-daemon running.json", path=str(path), error=str(e))
        return None
    try:
        raw = json.loads(body)
    except json.JSONDecodeError as e:
        _log.warning("shell-daemon running.json invalid JSON", path=str(path), error=str(e))
        return None
    try:
        return DaemonRunningState.model_validate(raw)
    except ValidationError as e:
        _log.warning(
            "shell-daemon running.json schema mismatch",
            path=str(path),
            error=str(e),
        )
        return None


def _daemon_base_url(state: DaemonRunningState) -> str:
    return f"http://{state.api_host}:{state.api_port}"


# =================================================================
# Response envelopes
# =================================================================


def _unavailable(reason: str) -> JSONResponse:
    """Build a 503 response carrying the discriminated
    ``kind: "unavailable"`` envelope the shell listens for.
    """
    return JSONResponse(
        status_code=503,
        content={"kind": "unavailable", "reason": reason},
    )


async def _forward(
    request: Request,
    method: str,
    suffix: str,
    json_body: dict[str, Any] | None = None,
) -> JSONResponse:
    """Forward a request to the shell daemon and wrap the
    response in the discriminated envelope.

    On daemon-offline (no running.json, or the daemon refuses
    the connection) returns a 503 with
    ``{"kind": "unavailable", "reason": "..."}``.

    On daemon-reachable returns whatever the daemon returned,
    wrapped as ``{"kind": "data", "status": <int>, "body": <json>}``.
    The wrapping preserves the upstream status code so the shell
    can still distinguish 400 (bad curator hex) from 422
    (attribution mismatch) from 500 (persistence failure).
    """
    state = _read_running_state()
    if state is None:
        return _unavailable("shell-daemon not running")

    url = f"{_daemon_base_url(state)}{suffix}"
    client: httpx.AsyncClient = request.app.state.daemon_httpx_client
    try:
        response = await client.request(method, url, json=json_body)
    except httpx.ConnectError as e:
        _log.warning("shell-daemon connect failed", url=url, error=str(e))
        return _unavailable(f"connect failed: {e}")
    except httpx.ReadTimeout as e:
        _log.warning("shell-daemon read timed out", url=url, error=str(e))
        return _unavailable(f"read timeout: {e}")
    except httpx.HTTPError as e:
        _log.warning("shell-daemon httpx error", url=url, error=str(e))
        return _unavailable(f"httpx error: {e}")

    try:
        body = response.json()
    except json.JSONDecodeError as e:
        _log.warning(
            "shell-daemon returned non-JSON body",
            url=url,
            status=response.status_code,
            error=str(e),
        )
        return _unavailable(f"non-json body: status={response.status_code}")

    return JSONResponse(
        status_code=200,
        content={
            "kind": "data",
            "status": response.status_code,
            "body": body,
        },
    )


# =================================================================
# Handlers
# =================================================================


@router.get("/info")
async def daemon_info(request: Request) -> JSONResponse:
    """Proxy ``GET /info`` on the shell daemon.

    Returns a ``DaemonStateSnapshot`` wrapped in the discriminated
    envelope so the shell can render a "daemon offline" state
    from the same React component tree.
    """
    return await _forward(request, "GET", "/info")


@router.get("/curators")
async def daemon_list_curators(request: Request) -> JSONResponse:
    """Proxy ``GET /curators`` on the shell daemon."""
    return await _forward(request, "GET", "/curators")


@router.post("/curators/subscribe")
async def daemon_subscribe_curator(request: Request) -> JSONResponse:
    """Proxy ``POST /curators/subscribe`` on the shell daemon.

    Body schema is whatever the shell sends; we forward
    verbatim so the daemon's validation remains the single
    source of truth. On bad body we return a 400 envelope
    without even reaching the daemon.
    """
    try:
        body = await request.json()
    except json.JSONDecodeError as e:
        return JSONResponse(
            status_code=400,
            content={"kind": "error", "reason": f"bad request json: {e}"},
        )
    if not isinstance(body, dict):
        return JSONResponse(
            status_code=400,
            content={"kind": "error", "reason": "request body must be a JSON object"},
        )
    return await _forward(request, "POST", "/curators/subscribe", json_body=body)


@router.delete("/curators/{pubkey_hex}")
async def daemon_unsubscribe_curator(request: Request, pubkey_hex: str) -> JSONResponse:
    """Proxy ``DELETE /curators/{pubkey}`` on the shell daemon."""
    return await _forward(request, "DELETE", f"/curators/{pubkey_hex}")


@router.get("/browse")
async def daemon_list_browse(request: Request) -> JSONResponse:
    """Proxy ``GET /browse`` on the shell daemon.

    This is the heaviest proxy call — the daemon probes every
    referenced project endpoint under a 2 s timeout each. The
    10 s read timeout here gives the aggregator enough headroom
    to probe ~5 uncached projects before the shell sees a
    timeout.
    """
    return await _forward(request, "GET", "/browse")


@router.get("/default-curators")
async def daemon_default_curators(request: Request) -> JSONResponse:
    """Proxy ``GET /default-curators`` on the shell daemon.

    Sprint 11 Phase B. Returns the daemon's configured default
    curator pubkeys so the shell can display them on the Curators
    page.
    """
    return await _forward(request, "GET", "/default-curators")


@router.post("/publish")
async def daemon_publish_project(request: Request) -> JSONResponse:
    """Proxy ``POST /publish`` on the shell daemon.

    Sprint 11 Phase A. Called by the coordinator's
    ``POST /project/publish`` endpoint (or directly by the CLI)
    to broadcast a project announcement on the gossip topic.
    Body schema is forwarded verbatim — the daemon validates.
    """
    try:
        body = await request.json()
    except json.JSONDecodeError as e:
        return JSONResponse(
            status_code=400,
            content={"kind": "error", "reason": f"bad request json: {e}"},
        )
    if not isinstance(body, dict):
        return JSONResponse(
            status_code=400,
            content={"kind": "error", "reason": "request body must be a JSON object"},
        )
    return await _forward(request, "POST", "/publish", json_body=body)
