"""FastAPI application factory.

The factory takes an already-started :class:`Coordinator` and
produces an app with its routes wired in. Phase A only mounts the
health router; later phases add ``/tasks``, ``/results``,
``/kudos``, ``/invite``, ``/app/...`` on the same app.
"""

from __future__ import annotations

from contextlib import asynccontextmanager
from typing import TYPE_CHECKING

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

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
    from nexus_coordinator.api.daemon import router as daemon_router
    from nexus_coordinator.api.health import router as health_router
    from nexus_coordinator.api.invites import router as invites_router
    from nexus_coordinator.api.kudos import router as kudos_router
    from nexus_coordinator.api.shell import router as shell_router
    from nexus_coordinator.api.tasks import router as tasks_router
    from nexus_coordinator.api.worker_state import router as worker_state_router

    @asynccontextmanager
    async def lifespan(_app: FastAPI):
        # The coordinator is started by the caller; we don't boot
        # or shut down iroh from the FastAPI lifespan because
        # `uv run nexus-coordinator start` owns the process and we
        # want tests to share a single instance across requests.
        yield

    app = FastAPI(
        title=f"nexus-coordinator[{coordinator.project_name}]",
        version="0.1.0",
        lifespan=lifespan,
    )

    # Sprint 5 Phase B: allow the local shell (Vite dev server at
    # 127.0.0.1:5173, or any other loopback port while hacking
    # on the web/ app) to hit the coordinator from a different
    # origin than the API itself. This is strictly loopback —
    # the coordinator defaults to `api_host=127.0.0.1` so an
    # off-box request cannot reach it anyway, and tightening
    # the allow list to regex `http://(127\.0\.0\.1|localhost):\d+`
    # keeps the browser from exposing the endpoints to any
    # malicious site the user might visit.
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
    app.include_router(shell_router)
    app.include_router(worker_state_router)
    app.include_router(daemon_router)
    return app
