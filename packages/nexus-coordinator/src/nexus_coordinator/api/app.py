# SPDX-License-Identifier: AGPL-3.0-or-later
"""FastAPI application factory.

The factory takes an already-started :class:`Coordinator` and
produces an app with its routes wired in. Phase A only mounts the
health router; later phases add ``/tasks``, ``/results``,
``/kudos``, ``/invite``, ``/app/...`` on the same app.
"""

from __future__ import annotations

from contextlib import asynccontextmanager
from typing import TYPE_CHECKING

import httpx
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from nexus_coordinator.auth import LoopbackAuthMiddleware, load_token

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator


def create_app(coordinator: "Coordinator") -> FastAPI:
    """Build a FastAPI app bound to an already-started coordinator.

    The ``coordinator`` argument must have had :meth:`start`
    called. The app's lifespan is a no-op because the coordinator
    manages its own iroh Node lifecycle — we don't want uvicorn to
    spin a second one up.
    """
    from nexus_coordinator.api.apps import router as apps_router
    from nexus_coordinator.api.canary import router as canary_router
    from nexus_coordinator.api.consent import router as consent_router
    from nexus_coordinator.api.contributor import router as contributor_router
    from nexus_coordinator.api.daemon import router as daemon_router
    from nexus_coordinator.api.deploy import router as deploy_router
    from nexus_coordinator.api.diagnostic import router as diagnostic_router
    from nexus_coordinator.api.events import router as events_router
    from nexus_coordinator.api.files import router as files_router
    from nexus_coordinator.api.health import router as health_router
    from nexus_coordinator.api.invites import router as invites_router
    from nexus_coordinator.api.kudos import router as kudos_router
    from nexus_coordinator.api.quarantine import router as quarantine_router
    from nexus_coordinator.api.shell import router as shell_router
    from nexus_coordinator.api.tasks import router as tasks_router
    from nexus_coordinator.api.worker_state import router as worker_state_router

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        # The coordinator is started by the caller; we don't boot
        # or shut down iroh from the FastAPI lifespan because
        # `uv run nexus-coordinator start` owns the process and we
        # want tests to share a single instance across requests.
        #
        # Sprint 9 Phase A (T10) — the shell-daemon proxy opens a
        # module-level singleton `httpx.AsyncClient` here, keyed
        # to the app instance via `app.state.daemon_httpx_client`.
        # A `Limits(max_connections=10)` cap bounds bursts from
        # rapid shell refreshes, and the `aclose()` on shutdown
        # releases sockets cleanly. The per-call `async with
        # httpx.AsyncClient(...)` pattern it replaces would
        # re-handshake on every request.
        timeout = httpx.Timeout(connect=2.0, read=10.0, write=2.0, pool=2.0)
        limits = httpx.Limits(max_connections=10, max_keepalive_connections=5)
        app.state.daemon_httpx_client = httpx.AsyncClient(timeout=timeout, limits=limits)
        # Sprint 26 Phase B: MCP session manager lifecycle.
        # Explicit __aenter__/__aexit__ because the context manager
        # must span the FastAPI lifespan yield boundary.
        mcp_srv = getattr(app.state, "mcp_server", None)
        mcp_ctx = mcp_srv.session_manager.run() if mcp_srv else None
        if mcp_ctx:
            await mcp_ctx.__aenter__()
        try:
            yield
        finally:
            if mcp_ctx:
                await mcp_ctx.__aexit__(None, None, None)
            await app.state.daemon_httpx_client.aclose()

    app = FastAPI(
        title=f"nexus-coordinator[{coordinator.project_name}]",
        version="0.1.0",
        lifespan=lifespan,
    )

    # Sprint 16 Phase A (D1): loopback bearer + Host + Origin
    # triple check. Added BEFORE the CORS middleware so that in
    # Starlette's reverse middleware wrapping order, CORS ends up
    # as the outer layer and can answer OPTIONS preflights (which
    # carry no auth header by spec) without the auth layer
    # intercepting them.
    #
    # Tests that construct the app directly (pytest, Starlette
    # TestClient) must inject ``SBFB_AUTH_TOKEN`` via
    # monkeypatch before calling :func:`create_app`.
    token = load_token()
    if token is None:
        raise RuntimeError(
            "loopback auth token missing — the launcher must set SBFB_AUTH_TOKEN "
            "or persist ~/.sbfb/auth_token before starting the coordinator"
        )
    app.add_middleware(LoopbackAuthMiddleware, token=token)

    # Sprint 5 Phase B: allow the local shell (Vite dev server at
    # 127.0.0.1:5173, or any other loopback port while hacking
    # on the web/ app) to hit the coordinator from a different
    # origin than the API itself. This is strictly loopback —
    # the coordinator defaults to `api_host=127.0.0.1` so an
    # off-box request cannot reach it anyway, and tightening
    # the allow list to regex `http://(127\.0\.0\.1|localhost):\d+`
    # keeps the browser from exposing the endpoints to any
    # malicious site the user might visit.
    #
    # Added LAST so the CORS layer wraps the auth layer: OPTIONS
    # preflight requests are answered directly by CORS, while
    # real GET/POST/DELETE requests still traverse auth first.
    app.add_middleware(
        CORSMiddleware,
        allow_origin_regex=r"^https?://(127\.0\.0\.1|localhost)(:\d+)?$",
        allow_credentials=False,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    # Stash the coordinator on the app state so routers can reach
    # it via `request.app.state.coordinator` instead of a global.
    app.state.coordinator = coordinator  # type: ignore[attr-defined]
    app.include_router(health_router)
    app.include_router(tasks_router)
    app.include_router(kudos_router)
    app.include_router(invites_router)
    app.include_router(apps_router)
    app.include_router(events_router)
    app.include_router(files_router)
    app.include_router(shell_router)
    app.include_router(worker_state_router)
    app.include_router(daemon_router)
    app.include_router(deploy_router)
    app.include_router(consent_router)
    app.include_router(canary_router)
    app.include_router(contributor_router)
    app.include_router(quarantine_router)
    app.include_router(diagnostic_router)

    # Sprint 26 Phase B (D1): MCP server local-only Streamable HTTP.
    # Mounted AFTER all routers so it sits at /mcp. The
    # LoopbackAuthMiddleware already covers bearer + Host + Origin;
    # CapabilityGateMiddleware adds the mcp_server_expose check.
    from nexus_coordinator.mcp_server import create_mcp_app

    mcp_asgi, mcp_server = create_mcp_app(coordinator)
    app.mount("/mcp", mcp_asgi)
    app.state.mcp_server = mcp_server  # type: ignore[attr-defined]

    return app
