# SPDX-License-Identifier: AGPL-3.0-or-later
"""Loopback bearer auth for the coordinator FastAPI.

Sprint 16 Phase A (D1 — defense en profondeur loopback).

Three checks applied to every request except ``/health`` and the
``/blob-serve/*`` proxy chain:

1. ``X-SBFB-Token: <hex>`` must match the token the launcher has
   persisted at ``~/.sbfb/auth_token``.
2. ``Host:`` must resolve to a loopback name — ``localhost``,
   ``127.0.0.1``, or ``[::1]`` — with an optional port. Blocks
   DNS rebinding (CVE-2025-49596, CVSS 9.4).
3. ``Origin:`` is either absent (CLI / curl / server-to-server)
   or a loopback HTTP URL (the React shell served from any
   ``http://localhost:*``).

Mirrors ``crates/nexus-shell-daemon-core/src/auth.rs`` exactly so
a request that passes the daemon's middleware would also pass the
coordinator's (and vice versa). Any drift between the two
implementations is a bug.
"""

from __future__ import annotations

import hmac
import os
from pathlib import Path

from starlette.middleware.base import BaseHTTPMiddleware
from starlette.requests import Request
from starlette.responses import PlainTextResponse
from starlette.types import ASGIApp

#: HTTP header name carrying the loopback bearer token. Kept as
#: the single source of truth so every caller uses the same
#: spelling (lowercase — Starlette normalizes header lookups
#: case-insensitively but emitting lowercase keeps the grep line
#: obvious in threat model docs).
AUTH_HEADER = "x-sbfb-token"

#: Environment variable the coordinator reads at startup to get
#: the token. Set by the launcher before spawning the
#: coordinator, or left unset to fall back to the on-disk file.
AUTH_TOKEN_ENV = "SBFB_AUTH_TOKEN"

#: Length of the hex-encoded token (256 bits / 4 bits per hex
#: char). Used to reject tokens of a wrong shape early.
TOKEN_HEX_LEN = 64

#: Paths exempted from the triple check. ``/health`` is the
#: launcher probe; the coordinator has no blob-serve route of
#: its own but the list is kept future-proof.
_PUBLIC_PATHS: frozenset[str] = frozenset({"/health"})


def sbfb_home() -> Path | None:
    """Return ``~/.sbfb`` for the current user, honouring the
    ``SBFB_HOME`` env override so tests and the launcher can
    redirect both files at the same tempdir.
    """
    override = os.environ.get("SBFB_HOME")
    if override:
        return Path(override)
    home = os.environ.get("HOME") or os.environ.get("USERPROFILE")
    if not home:
        return None
    return Path(home) / ".sbfb"


def auth_token_path() -> Path | None:
    """Return the path of the on-disk token file."""
    home = sbfb_home()
    return None if home is None else home / "auth_token"


def load_token() -> str | None:
    """Resolve the loopback bearer token.

    Precedence:

    1. ``SBFB_AUTH_TOKEN`` env var (non-empty) — the launcher sets
       this before spawning the coordinator so the child process
       does not have to read the file.
    2. ``~/.sbfb/auth_token`` — the on-disk fallback.

    Returns ``None`` if neither source is available; callers
    decide whether that is fatal (production) or a warn-only
    state (test without the launcher in the loop).
    """
    env = os.environ.get(AUTH_TOKEN_ENV)
    if env:
        return env
    path = auth_token_path()
    if path is None or not path.exists():
        return None
    try:
        raw = path.read_text(encoding="utf-8").strip()
    except OSError:
        return None
    if len(raw) != TOKEN_HEX_LEN or not all(c in "0123456789abcdefABCDEF" for c in raw):
        return None
    return raw


def is_loopback_host(host: str) -> bool:
    """Return ``True`` iff the raw ``Host:`` header value points
    at a loopback name with an optional port.

    Accepts ``localhost``, ``127.0.0.1``, and ``[::1]``, each
    optionally followed by ``:PORT`` where PORT is a valid u16.
    """
    if not host:
        return False
    if host.startswith("["):
        end = host.find("]")
        if end < 0:
            return False
        inside = host[1:end]
        tail = host[end + 1 :]
        host_only = inside
        port_str: str | None = None
        if tail:
            if not tail.startswith(":"):
                return False
            port_str = tail[1:]
    elif ":" in host:
        host_only, port_str = host.rsplit(":", 1)
    else:
        host_only, port_str = host, None

    if host_only not in ("localhost", "127.0.0.1", "::1"):
        return False
    if port_str is not None:
        try:
            port = int(port_str)
        except ValueError:
            return False
        if not 0 <= port <= 65535:
            return False
    return True


def is_loopback_origin(origin: str) -> bool:
    """Return ``True`` iff the ``Origin:`` header value is an
    HTTP loopback URL with no path.
    """
    prefix = "http://"
    if not origin.startswith(prefix):
        return False
    authority = origin[len(prefix) :]
    if "/" in authority:
        return False
    return is_loopback_host(authority)


class LoopbackAuthMiddleware(BaseHTTPMiddleware):
    """Starlette middleware enforcing the triple check.

    The token is captured at middleware construction rather than
    re-read per request so the hot path is a constant-time
    compare only. The caller is responsible for constructing a
    fresh middleware (and therefore a fresh FastAPI app) if it
    wants the token to rotate — matches the Syncthing / Jupyter
    / BOINC pattern of "delete the file, restart".
    """

    def __init__(self, app: ASGIApp, token: str) -> None:
        super().__init__(app)
        if len(token) != TOKEN_HEX_LEN:
            raise ValueError(f"loopback token must be {TOKEN_HEX_LEN} hex chars, got {len(token)}")
        self._token = token

    async def dispatch(self, request: Request, call_next):
        path = request.url.path
        if path in _PUBLIC_PATHS:
            return await call_next(request)

        # 1. Bearer token
        provided = request.headers.get(AUTH_HEADER, "")
        if not hmac.compare_digest(provided, self._token):
            return PlainTextResponse("missing or invalid token", status_code=401)

        # 2. Host allowlist (block DNS rebinding)
        host_hdr = request.headers.get("host", "")
        if not is_loopback_host(host_hdr):
            return PlainTextResponse("host not allowed", status_code=403)

        # 3. Origin check (absent is fine for CLI callers)
        origin_hdr = request.headers.get("origin")
        if origin_hdr is not None and not is_loopback_origin(origin_hdr):
            return PlainTextResponse("origin not allowed", status_code=403)

        return await call_next(request)


__all__ = [
    "AUTH_HEADER",
    "AUTH_TOKEN_ENV",
    "TOKEN_HEX_LEN",
    "LoopbackAuthMiddleware",
    "auth_token_path",
    "is_loopback_host",
    "is_loopback_origin",
    "load_token",
    "sbfb_home",
]
