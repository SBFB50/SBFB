# SPDX-License-Identifier: AGPL-3.0-or-later
"""MCP server — local-only Streamable HTTP, 3 tools whitelist.

Sprint 26 Phase B (D1). Uses the official ``mcp`` SDK (v1.27+,
modelcontextprotocol/python-sdk) instead of hand-rolled JSON-RPC.
The server is mounted at ``/mcp`` on the coordinator FastAPI app.

Security layers (defence in depth):

1. ``LoopbackAuthMiddleware`` — bearer + Host + Origin triple check
   (applied to all routes by the FastAPI factory, Sprint 16).
2. ``CapabilityGateMiddleware`` — checks ``mcp_server_expose``
   capability before any MCP request reaches the SDK handler.
3. Stateless mode (``json_response=True``) — no server-side session
   state, no SSE push.

The three tools mirror the bridge postMessage whitelist (Sprint 13):
``task_submit``, ``storage_get``, ``storage_set``.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

import structlog
from mcp.server.fastmcp import FastMCP
from mcp.server.transport_security import TransportSecuritySettings
from starlette.responses import JSONResponse
from starlette.types import ASGIApp, Receive, Scope, Send

from nexus_coordinator.capability_store import get_capabilities_store
from nexus_coordinator.dispatcher import SubmitRequest
from nexus_coordinator.upload_queue import QueueFullError

if TYPE_CHECKING:
    from nexus_coordinator.coordinator import Coordinator

_log = structlog.get_logger(__name__)

_coordinator: Coordinator | None = None

_NO_DNS_REBINDING = TransportSecuritySettings(
    enable_dns_rebinding_protection=False,
)


def set_coordinator(coord: Coordinator) -> None:
    global _coordinator  # noqa: PLW0603
    _coordinator = coord


def _get_coordinator() -> "Coordinator":
    if _coordinator is None:
        raise RuntimeError("MCP server used before coordinator wired")
    return _coordinator


def _register_tools(server: FastMCP) -> None:
    """Register the 3 MCP tools on *server*."""

    @server.tool()
    async def task_submit(
        project_id: str,
        prompt: str,
        model: str = "",
    ) -> dict[str, Any]:
        """Submit a compute task to the SBFB network."""
        coord = _get_coordinator()
        if coord.upload_queue is None:
            return {"error": "upload queue not initialised"}
        submit_req = SubmitRequest(
            task_type="llm",
            prompt=prompt,
            model=model or "default",
            system_prompt="",
            priority=5,
            parent_task_id="",
            metadata={"source": "mcp", "project_id": project_id},
            task_id=None,
            is_open_source=coord.config.identity.repo_url is not None,
            estimated_watts=100,
            estimated_vram_mb=2000,
            estimated_hours=0.1,
            redundancy_factor=1,
        )
        try:
            task_id = await coord.upload_queue.schedule(submit_req)
        except QueueFullError:
            return {"error": "queue full, retry later"}
        return {"task_id": task_id}

    @server.tool()
    async def storage_get(project_id: str, key: str) -> dict[str, Any]:
        """Read a value from an app's storage namespace."""
        coord = _get_coordinator()
        ctx = coord.app_contexts.get(project_id)
        if ctx is None:
            return {"error": f"app {project_id!r} not found"}
        if ctx.storage is None:
            return {"error": f"app {project_id!r} has no storage"}
        value = await ctx.storage.get(key)
        return {"key": key, "value": value}

    @server.tool()
    async def storage_set(
        project_id: str,
        key: str,
        value: str,
    ) -> dict[str, Any]:
        """Write a value into an app's storage namespace."""
        coord = _get_coordinator()
        ctx = coord.app_contexts.get(project_id)
        if ctx is None:
            return {"error": f"app {project_id!r} not found"}
        if ctx.storage is None:
            return {"error": f"app {project_id!r} has no storage"}
        await ctx.storage.set(key, value)
        return {"ok": True}


class CapabilityGateMiddleware:
    """ASGI middleware — 403 when ``mcp_server_expose`` is disabled."""

    def __init__(self, app: ASGIApp) -> None:
        self.app = app

    async def __call__(self, scope: Scope, receive: Receive, send: Send) -> None:
        if scope["type"] == "http":
            store = get_capabilities_store()
            if store is None or not store.is_enabled("mcp_server_expose"):
                response = JSONResponse(
                    {
                        "detail": (
                            "capability 'mcp_server_expose' is disabled. "
                            "Run `nexus-admin capability enable mcp_server_expose` "
                            "to activate."
                        ),
                    },
                    status_code=403,
                )
                await response(scope, receive, send)
                return
        await self.app(scope, receive, send)


def build_mcp_server() -> FastMCP:
    """Create a fresh ``FastMCP`` instance with the 3 SBFB tools.

    Each call returns a new instance — the SDK's session manager is
    single-use, so production gets one instance (via
    :func:`create_mcp_app`) and tests can create as many as needed.
    """
    server = FastMCP(
        "sbfb",
        stateless_http=True,
        json_response=True,
        # DNS rebinding protection is handled by LoopbackAuthMiddleware
        # (Sprint 16) which validates Host + Origin + bearer on every
        # request. The SDK's built-in check is redundant and rejects
        # valid loopback hosts without a port suffix.
        transport_security=_NO_DNS_REBINDING,
    )
    _register_tools(server)
    return server


def create_mcp_app(coord: Coordinator) -> tuple[ASGIApp, FastMCP]:
    """Build the MCP ASGI app with capability gate, wired to *coord*.

    Returns ``(asgi_app, server)`` — the caller needs the server
    reference to manage the session manager lifecycle.
    """
    set_coordinator(coord)
    server = build_mcp_server()
    asgi_app = CapabilityGateMiddleware(server.streamable_http_app())
    return asgi_app, server
