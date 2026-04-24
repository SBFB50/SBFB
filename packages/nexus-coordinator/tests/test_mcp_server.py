# SPDX-License-Identifier: AGPL-3.0-or-later
"""MCP server tests — Sprint 26 Phase B.

Tests the MCP Streamable HTTP endpoint mounted at ``/mcp``,
including capability gate, tool dispatch, and JSON-RPC protocol
compliance (via the official ``mcp`` SDK).

All tests are hermetic: each test gets a fresh ``tmp_path``,
monkeypatched paths, and a fresh ``FastMCP`` instance (the SDK's
session manager is single-use per instance).
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import httpx
import pytest
from nexus_coordinator import auth as _auth
from nexus_coordinator.capability_store import init_capabilities
from nexus_coordinator.mcp_server import (
    CapabilityGateMiddleware,
    build_mcp_server,
    set_coordinator,
)

_TEST_AUTH_TOKEN = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef"

_INIT_PARAMS = {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {"name": "test", "version": "1.0"},
}


# -- Lightweight coordinator stub -------------------------------------------


@dataclass
class _FakeUploadQueue:
    last_req: Any = None

    async def schedule(self, req: Any) -> str:
        self.last_req = req
        return "task-id-abc"


@dataclass
class _FakeStorage:
    _data: dict[str, Any] = field(default_factory=dict)

    async def get(self, key: str) -> Any:
        return self._data.get(key)

    async def set(self, key: str, value: Any) -> None:
        self._data[key] = value


@dataclass
class _FakeAppContext:
    storage: _FakeStorage | None = field(default_factory=_FakeStorage)


@dataclass
class _FakeIdentity:
    repo_url: str | None = None


@dataclass
class _FakeConfig:
    identity: _FakeIdentity = field(default_factory=_FakeIdentity)


@dataclass
class _FakeCoordinator:
    upload_queue: _FakeUploadQueue | None = field(default_factory=_FakeUploadQueue)
    app_contexts: dict[str, _FakeAppContext] = field(default_factory=dict)
    config: _FakeConfig = field(default_factory=_FakeConfig)
    project_name: str = "test-project"


# -- Fixtures ---------------------------------------------------------------


@pytest.fixture
def fake_coord() -> _FakeCoordinator:
    coord = _FakeCoordinator()
    coord.app_contexts["myapp"] = _FakeAppContext()
    return coord


@pytest.fixture
def _enable_mcp(tmp_path: Path) -> None:
    """Enable the mcp_server_expose capability."""
    store = init_capabilities(tmp_path / "capabilities.toml")
    store.enable("mcp_server_expose", "test")


@pytest.fixture
def _disable_mcp(tmp_path: Path) -> None:
    """Keep mcp_server_expose disabled (default)."""
    init_capabilities(tmp_path / "capabilities.toml")


def _jsonrpc(method: str, params: dict | None = None, req_id: int = 1) -> dict:
    msg: dict[str, Any] = {"jsonrpc": "2.0", "method": method, "id": req_id}
    if params is not None:
        msg["params"] = params
    return msg


async def _mcp_session(coord, *, enabled: bool = True):
    """Build a fresh MCP ASGI app + enter the session manager.

    Returns ``(app, server)`` inside a running session context.
    Caller must ``await server.session_manager.run().__aexit__(...)``
    when done, or use :func:`_mcp_post` which manages this per call.
    """
    set_coordinator(coord)
    server = build_mcp_server()
    inner = server.streamable_http_app()
    app = CapabilityGateMiddleware(inner) if enabled else inner
    return app, server


async def _mcp_post(
    coord,
    payloads: list[dict],
    *,
    enabled: bool = True,
    token: str = _TEST_AUTH_TOKEN,
) -> list[httpx.Response]:
    """Send one or more JSON-RPC requests to a fresh MCP server.

    Creates a fresh ``FastMCP`` per call so the session manager
    lifecycle is self-contained.
    """
    set_coordinator(coord)
    server = build_mcp_server()
    app = CapabilityGateMiddleware(server.streamable_http_app())

    headers: dict[str, str] = {
        "content-type": "application/json",
        "accept": "application/json, text/event-stream",
        _auth.AUTH_HEADER: token,
        "host": "127.0.0.1",
    }
    responses: list[httpx.Response] = []
    async with server.session_manager.run():
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://127.0.0.1",
            headers=headers,
        ) as client:
            for payload in payloads:
                resp = await client.post("/mcp", content=json.dumps(payload))
                responses.append(resp)
    return responses


# -- Protocol tests ----------------------------------------------------------


class TestMcpInitialize:
    @pytest.mark.asyncio
    async def test_initialize_returns_capabilities(self, fake_coord, _enable_mcp) -> None:
        [resp] = await _mcp_post(fake_coord, [_jsonrpc("initialize", _INIT_PARAMS)])
        assert resp.status_code == 200
        body = resp.json()
        assert body.get("jsonrpc") == "2.0"
        result = body["result"]
        assert "capabilities" in result
        assert "serverInfo" in result


class TestMcpListTools:
    @pytest.mark.asyncio
    async def test_list_tools_returns_three(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [_jsonrpc("initialize", _INIT_PARAMS), _jsonrpc("tools/list", req_id=2)],
        )
        assert resp.status_code == 200
        tools = resp.json()["result"]["tools"]
        names = {t["name"] for t in tools}
        assert names == {"task_submit", "storage_get", "storage_set"}

    @pytest.mark.asyncio
    async def test_tools_have_input_schema(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [_jsonrpc("initialize", _INIT_PARAMS), _jsonrpc("tools/list", req_id=2)],
        )
        tools = resp.json()["result"]["tools"]
        for tool in tools:
            assert "inputSchema" in tool
            schema = tool["inputSchema"]
            assert schema["type"] == "object"
            assert "properties" in schema


# -- Tool call tests ---------------------------------------------------------


class TestMcpCallTaskSubmit:
    @pytest.mark.asyncio
    async def test_task_submit_dispatches(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [
                _jsonrpc("initialize", _INIT_PARAMS),
                _jsonrpc(
                    "tools/call",
                    {"name": "task_submit", "arguments": {"project_id": "proj1", "prompt": "hello"}},
                    req_id=3,
                ),
            ],
        )
        assert resp.status_code == 200
        text = resp.json()["result"]["content"][0]["text"]
        parsed = json.loads(text)
        assert parsed["task_id"] == "task-id-abc"
        assert fake_coord.upload_queue.last_req is not None
        assert fake_coord.upload_queue.last_req.prompt == "hello"


class TestMcpCallStorageGet:
    @pytest.mark.asyncio
    async def test_storage_get_returns_value(self, fake_coord, _enable_mcp) -> None:
        fake_coord.app_contexts["myapp"].storage._data["color"] = "blue"
        [_, resp] = await _mcp_post(
            fake_coord,
            [
                _jsonrpc("initialize", _INIT_PARAMS),
                _jsonrpc(
                    "tools/call",
                    {"name": "storage_get", "arguments": {"project_id": "myapp", "key": "color"}},
                    req_id=3,
                ),
            ],
        )
        text = resp.json()["result"]["content"][0]["text"]
        assert json.loads(text)["value"] == "blue"

    @pytest.mark.asyncio
    async def test_storage_get_unknown_app(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [
                _jsonrpc("initialize", _INIT_PARAMS),
                _jsonrpc(
                    "tools/call",
                    {"name": "storage_get", "arguments": {"project_id": "nonexistent", "key": "k"}},
                    req_id=3,
                ),
            ],
        )
        text = resp.json()["result"]["content"][0]["text"]
        assert "not found" in text


class TestMcpCallStorageSet:
    @pytest.mark.asyncio
    async def test_storage_set_writes(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [
                _jsonrpc("initialize", _INIT_PARAMS),
                _jsonrpc(
                    "tools/call",
                    {"name": "storage_set", "arguments": {"project_id": "myapp", "key": "color", "value": "red"}},
                    req_id=3,
                ),
            ],
        )
        text = resp.json()["result"]["content"][0]["text"]
        assert json.loads(text)["ok"] is True
        assert fake_coord.app_contexts["myapp"].storage._data["color"] == "red"


# -- Error handling tests ----------------------------------------------------


class TestMcpErrors:
    @pytest.mark.asyncio
    async def test_unknown_method_returns_error(self, fake_coord, _enable_mcp) -> None:
        [_, resp] = await _mcp_post(
            fake_coord,
            [_jsonrpc("initialize", _INIT_PARAMS), _jsonrpc("nonexistent/method", req_id=2)],
        )
        assert resp.status_code == 200
        assert "error" in resp.json()

    @pytest.mark.asyncio
    async def test_invalid_json_returns_parse_error(self, fake_coord, _enable_mcp) -> None:
        set_coordinator(fake_coord)
        server = build_mcp_server()
        app = CapabilityGateMiddleware(server.streamable_http_app())
        headers = {
            "content-type": "application/json",
            "accept": "application/json, text/event-stream",
            _auth.AUTH_HEADER: _TEST_AUTH_TOKEN,
            "host": "127.0.0.1",
        }
        async with server.session_manager.run():
            async with httpx.AsyncClient(
                transport=httpx.ASGITransport(app=app),
                base_url="http://127.0.0.1",
                headers=headers,
            ) as client:
                resp = await client.post("/mcp", content="not json{{{")
        assert resp.status_code in (200, 400)
        body = resp.json()
        assert "error" in body


# -- Capability gate tests ---------------------------------------------------


class TestCapabilityGate:
    @pytest.mark.asyncio
    async def test_disabled_returns_403(self, fake_coord, _disable_mcp) -> None:
        set_coordinator(fake_coord)
        server = build_mcp_server()
        app = CapabilityGateMiddleware(server.streamable_http_app())
        headers = {
            "content-type": "application/json",
            "accept": "application/json, text/event-stream",
            _auth.AUTH_HEADER: _TEST_AUTH_TOKEN,
            "host": "127.0.0.1",
        }
        async with httpx.AsyncClient(
            transport=httpx.ASGITransport(app=app),
            base_url="http://127.0.0.1",
            headers=headers,
        ) as client:
            resp = await client.post(
                "/mcp",
                content=json.dumps(_jsonrpc("initialize", _INIT_PARAMS)),
            )
        assert resp.status_code == 403
        assert "mcp_server_expose" in resp.json()["detail"]

    @pytest.mark.asyncio
    async def test_enabled_passes_through(self, fake_coord, _enable_mcp) -> None:
        [resp] = await _mcp_post(fake_coord, [_jsonrpc("initialize", _INIT_PARAMS)])
        assert resp.status_code == 200


# -- Integration round-trip --------------------------------------------------


class TestMcpRoundTrip:
    @pytest.mark.asyncio
    async def test_full_round_trip(self, fake_coord, _enable_mcp) -> None:
        """initialize -> list -> call each tool."""
        fake_coord.app_contexts["demo"] = _FakeAppContext()

        responses = await _mcp_post(
            fake_coord,
            [
                # 1. initialize
                _jsonrpc("initialize", _INIT_PARAMS),
                # 2. tools/list
                _jsonrpc("tools/list", req_id=2),
                # 3. task_submit
                _jsonrpc(
                    "tools/call",
                    {"name": "task_submit", "arguments": {"project_id": "demo", "prompt": "test task"}},
                    req_id=3,
                ),
                # 4. storage_set
                _jsonrpc(
                    "tools/call",
                    {"name": "storage_set", "arguments": {"project_id": "demo", "key": "x", "value": "42"}},
                    req_id=4,
                ),
                # 5. storage_get
                _jsonrpc(
                    "tools/call",
                    {"name": "storage_get", "arguments": {"project_id": "demo", "key": "x"}},
                    req_id=5,
                ),
            ],
        )

        assert responses[0].status_code == 200  # initialize
        assert len(responses[1].json()["result"]["tools"]) == 3  # list
        text3 = json.loads(responses[2].json()["result"]["content"][0]["text"])
        assert "task_id" in text3  # task_submit
        text4 = json.loads(responses[3].json()["result"]["content"][0]["text"])
        assert text4["ok"] is True  # storage_set
        text5 = json.loads(responses[4].json()["result"]["content"][0]["text"])
        assert text5["value"] == "42"  # storage_get
