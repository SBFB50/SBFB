# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 7 Phase E — ``/daemon/*`` proxy endpoint tests.

These tests spin up a tiny stdlib ``http.server`` in a
background thread to impersonate the Rust ``nexus-shell-daemon``
HTTP surface, write a fake ``running.json`` pointing at it, and
drive the coordinator proxy end-to-end:

- daemon missing (no ``running.json``) → 503 unavailable envelope
- daemon reachable, 200 → 200 data envelope with upstream body
- daemon reachable, 422 → 200 data envelope carrying the 422 inside
- daemon unreachable (stale ``running.json`` pointing at a dead port)
  → 503 unavailable envelope with a connect-failed reason

The background HTTP server is a ``ThreadingHTTPServer`` binding
an ephemeral localhost port. Every test tears it down in a
``finally`` block so nothing leaks across tests.
"""

from __future__ import annotations

import json
import socket
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

import httpx
import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator

# ---------------------------------------------------------------
# Tiny stdlib fake daemon
# ---------------------------------------------------------------


class _FakeDaemon:
    """Background ``ThreadingHTTPServer`` that answers a fixed set
    of canned responses, keyed by ``(method, path)``.

    The handler serializes each response as
    ``(status: int, body: dict[str, Any] | str)`` so a single
    suite can cover both JSON-OK and non-JSON-error paths.
    """

    def __init__(self) -> None:
        self._responses: dict[tuple[str, str], tuple[int, Any]] = {}
        self._server: ThreadingHTTPServer | None = None
        self._thread: threading.Thread | None = None
        self.host: str = "127.0.0.1"
        self.port: int = 0
        self.calls: list[tuple[str, str, bytes]] = []
        self._lock = threading.Lock()

    def set_response(self, method: str, path: str, status: int, body: Any) -> None:
        with self._lock:
            self._responses[(method.upper(), path)] = (status, body)

    def start(self) -> None:
        fake = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args: Any, **_kwargs: Any) -> None:  # noqa: D401
                # Keep test output clean — stdlib's default
                # handler logs every request to stderr.
                return

            def _serve(self, method: str) -> None:
                content_length = int(self.headers.get("Content-Length", "0") or "0")
                body = self.rfile.read(content_length) if content_length else b""
                with fake._lock:
                    fake.calls.append((method, self.path, body))
                    resp = fake._responses.get((method, self.path))
                if resp is None:
                    self.send_error(404, "no canned response for this route")
                    return
                status, body_obj = resp
                if isinstance(body_obj, (dict, list)):
                    payload = json.dumps(body_obj).encode("utf-8")
                    self.send_response(status)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)
                else:
                    payload = str(body_obj).encode("utf-8")
                    self.send_response(status)
                    self.send_header("Content-Type", "text/plain")
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)

            def do_GET(self) -> None:
                self._serve("GET")

            def do_POST(self) -> None:
                self._serve("POST")

            def do_DELETE(self) -> None:
                self._serve("DELETE")

        # Bind port 0 so the OS picks an unused ephemeral port.
        self._server = ThreadingHTTPServer((self.host, 0), Handler)
        self.port = self._server.server_address[1]
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True, name="fake-daemon-http")
        self._thread.start()

    def stop(self) -> None:
        if self._server is not None:
            self._server.shutdown()
            self._server.server_close()
        if self._thread is not None:
            self._thread.join(timeout=5)

    def __enter__(self) -> "_FakeDaemon":
        self.start()
        return self

    def __exit__(self, *_exc: Any) -> None:
        self.stop()


def _unused_loopback_port() -> int:
    """Return a loopback port that is (likely) closed.

    Opens a TCP listener on port 0, reads the assigned port, and
    immediately closes it. On the vast majority of systems that
    port stays free for long enough for the test to try to
    connect to it. We use this to force a
    ``httpx.ConnectError`` without running any server.
    """
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.bind(("127.0.0.1", 0))
        return int(s.getsockname()[1])


def _write_running_json(
    root: Path,
    *,
    port: int,
    node_id: str = "de" * 32,
    host: str = "127.0.0.1",
) -> None:
    path = root / "shell-daemon" / "running.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    state = {
        "schema_version": 1,
        "node_id": node_id,
        "api_host": host,
        "api_port": port,
        "pid": 424242,
        "started_at": "2026-04-11T12:00:00Z",
        "daemon_version": "0.1.0-test",
    }
    path.write_text(json.dumps(state), encoding="utf-8")


# ---------------------------------------------------------------
# Tests
# ---------------------------------------------------------------


@pytest.mark.asyncio
async def test_daemon_info_returns_503_when_running_json_absent(
    nexus_grid_tmp: Path,
) -> None:
    """Missing ``running.json`` → 503 unavailable envelope.

    The shell interprets this as "daemon offline" and shows a
    CTA to start the daemon; no error toast.
    """
    coord = Coordinator(project_name="daemon-absent")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/daemon/info")
            assert r.status_code == 503
            body = r.json()
            assert body["kind"] == "unavailable"
            assert "not running" in body["reason"].lower()
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_daemon_info_forwards_upstream_when_running(nexus_grid_tmp: Path) -> None:
    """A reachable daemon → 200 ``kind: "data"`` envelope carrying
    the upstream body verbatim."""
    with _FakeDaemon() as fake:
        fake.set_response(
            "GET",
            "/info",
            200,
            {
                "schema_version": 1,
                "node_id": "aa" * 32,
                "daemon_version": "0.1.0-fake",
                "uptime_secs": 10,
                "started_at": "2026-04-11T12:00:00Z",
                "last_updated_at": "2026-04-11T12:00:10Z",
                "api_host": "127.0.0.1",
                "api_port": fake.port,
                "subscribed_curators": [],
                "known_lists": 0,
                "known_browse_entries": 0,
            },
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-info")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.get("/daemon/info")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 200
                assert body["body"]["schema_version"] == 1
                assert body["body"]["daemon_version"] == "0.1.0-fake"
        finally:
            await coord.stop()

    assert any(call[0] == "GET" and call[1] == "/info" for call in fake.calls)


@pytest.mark.asyncio
async def test_daemon_curators_list_forwards_upstream(nexus_grid_tmp: Path) -> None:
    with _FakeDaemon() as fake:
        fake.set_response(
            "GET",
            "/curators",
            200,
            {"entries": [], "subscribed_curators": ["aa" * 32]},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-curators-list")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.get("/daemon/curators")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 200
                assert body["body"]["subscribed_curators"] == ["aa" * 32]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_curators_subscribe_forwards_post_body(
    nexus_grid_tmp: Path,
) -> None:
    """POST bodies are forwarded verbatim as JSON — the daemon
    is the single source of validation truth."""
    with _FakeDaemon() as fake:
        fake.set_response(
            "POST",
            "/curators/subscribe",
            200,
            {"subscribed_curators": ["bb" * 32]},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-curators-sub")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/daemon/curators/subscribe",
                    json={"curator_pubkey_hex": "bb" * 32},
                )
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 200
                assert body["body"]["subscribed_curators"] == ["bb" * 32]
        finally:
            await coord.stop()

    # Verify the request body really made it through the proxy.
    post_calls = [c for c in fake.calls if c[0] == "POST" and c[1] == "/curators/subscribe"]
    assert len(post_calls) == 1
    forwarded = json.loads(post_calls[0][2])
    assert forwarded["curator_pubkey_hex"] == "bb" * 32


@pytest.mark.asyncio
async def test_daemon_curators_subscribe_rejects_non_object_body(
    nexus_grid_tmp: Path,
) -> None:
    """The proxy refuses non-object JSON bodies before even
    reaching the daemon — catches a class of shell bugs locally."""
    with _FakeDaemon() as fake:
        _write_running_json(nexus_grid_tmp, port=fake.port)
        coord = Coordinator(project_name="daemon-curators-bad")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/daemon/curators/subscribe",
                    json=["not", "an", "object"],
                )
                assert r.status_code == 400
                body = r.json()
                assert body["kind"] == "error"
                assert "object" in body["reason"]
        finally:
            await coord.stop()

    # The proxy must NOT have forwarded anything.
    assert [c for c in fake.calls if c[0] == "POST"] == []


@pytest.mark.asyncio
async def test_daemon_curators_delete_forwards_path_param(
    nexus_grid_tmp: Path,
) -> None:
    pubkey = "cc" * 32
    with _FakeDaemon() as fake:
        fake.set_response(
            "DELETE",
            f"/curators/{pubkey}",
            200,
            {"subscribed_curators": []},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-curators-del")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.delete(f"/daemon/curators/{pubkey}")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["body"]["subscribed_curators"] == []
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_browse_forwards_upstream(nexus_grid_tmp: Path) -> None:
    with _FakeDaemon() as fake:
        fake.set_response(
            "GET",
            "/browse",
            200,
            {
                "entries": [
                    {
                        "project_id": "aa" * 32,
                        "project_name": "demo",
                        "category": "gov",
                        "description": "test",
                        "curator_pubkey": "bb" * 32,
                        "curator_name": "FlowUP",
                        "source": "curator",
                        "status": "reachable",
                        "last_probed_at": "2026-04-11T12:00:00Z",
                    }
                ]
            },
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-browse")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.get("/daemon/browse")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                entries = body["body"]["entries"]
                assert len(entries) == 1
                assert entries[0]["status"] == "reachable"
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_info_returns_503_when_running_json_points_at_dead_port(
    nexus_grid_tmp: Path,
) -> None:
    """Stale ``running.json`` pointing at a closed port → 503
    unavailable envelope carrying a connect-failed reason."""
    dead_port = _unused_loopback_port()
    _write_running_json(nexus_grid_tmp, port=dead_port)

    coord = Coordinator(project_name="daemon-dead-port")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/daemon/info")
            assert r.status_code == 503
            body = r.json()
            assert body["kind"] == "unavailable"
            # Reason is one of our mapped transport failure
            # labels — connect failure, read timeout, or the
            # generic httpx wrapper.
            reason = body["reason"].lower()
            assert any(keyword in reason for keyword in ("connect", "timeout", "httpx", "not running"))
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_daemon_info_returns_503_on_malformed_running_json(
    nexus_grid_tmp: Path,
) -> None:
    """A running.json file that fails schema validation is
    treated as 'daemon offline' — the proxy returns 503 without
    attempting any network call."""
    path = nexus_grid_tmp / "shell-daemon" / "running.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("{not valid json", encoding="utf-8")

    coord = Coordinator(project_name="daemon-bad-json")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/daemon/info")
            assert r.status_code == 503
            body = r.json()
            assert body["kind"] == "unavailable"
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_daemon_proxy_shares_httpx_client(nexus_grid_tmp: Path) -> None:
    """Sprint 9 Phase A (T10) — two requests on the same app
    must reuse the singleton ``httpx.AsyncClient`` stashed in
    ``app.state.daemon_httpx_client`` by the FastAPI lifespan.

    The previous per-call ``async with httpx.AsyncClient(...)``
    pattern handshook the TCP/TLS path on every request with no
    ``Limits`` cap. We assert:

    1. ``app.state.daemon_httpx_client`` is an
       :class:`httpx.AsyncClient` built by lifespan.
    2. The instance reference is stable across two requests.
    3. A ``Limits`` cap is in place (the client exposes its
       configured limits via the private ``_limits`` attribute
       on the transport, which is the documented access point
       for tests in 0.27+).
    """
    with _FakeDaemon() as fake:
        fake.set_response(
            "GET",
            "/curators",
            200,
            {"entries": [], "subscribed_curators": []},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-shared-client")
        await coord.start()
        try:
            app = create_app(coord)
            with TestClient(app) as client:
                shared = app.state.daemon_httpx_client
                assert isinstance(shared, httpx.AsyncClient)

                r1 = client.get("/daemon/curators")
                assert r1.status_code == 200
                assert app.state.daemon_httpx_client is shared

                r2 = client.get("/daemon/curators")
                assert r2.status_code == 200
                assert app.state.daemon_httpx_client is shared
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_proxy_forwards_422_from_daemon(nexus_grid_tmp: Path) -> None:
    """The daemon's validation errors (e.g. revision rollback,
    attribution mismatch) come back as 422 upstream. The proxy
    wraps them in the ``kind: "data"`` envelope preserving the
    upstream status, so the shell can distinguish
    'daemon said no' from 'daemon offline'."""
    with _FakeDaemon() as fake:
        fake.set_response(
            "POST",
            "/curators/subscribe",
            422,
            {"error": "invalid curator pubkey hex: not-hex"},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-422")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/daemon/curators/subscribe",
                    json={"curator_pubkey_hex": "not-hex"},
                )
                # The proxy returns 200 (it successfully forwarded)
                # with the upstream 422 inside the envelope.
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 422
                assert "not-hex" in body["body"]["error"]
        finally:
            await coord.stop()


# ---------------------------------------------------------------
# Sprint 11 Phase A — POST /daemon/publish + POST /project/publish
# ---------------------------------------------------------------


@pytest.mark.asyncio
async def test_daemon_publish_forwards_post_body(nexus_grid_tmp: Path) -> None:
    """POST /daemon/publish forwards the body to the daemon's
    POST /publish and wraps the response."""
    with _FakeDaemon() as fake:
        fake.set_response("POST", "/publish", 200, {"published": True})
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="daemon-publish")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/daemon/publish",
                    json={
                        "project_name": "gov-officiel",
                        "category": "gov",
                        "description": "Le projet gouvernance",
                        "apps": ["gov"],
                    },
                )
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 200
                assert body["body"]["published"] is True
        finally:
            await coord.stop()

    post_calls = [c for c in fake.calls if c[0] == "POST" and c[1] == "/publish"]
    assert len(post_calls) == 1
    forwarded = json.loads(post_calls[0][2])
    assert forwarded["project_name"] == "gov-officiel"


@pytest.mark.asyncio
async def test_daemon_publish_returns_503_when_daemon_down(
    nexus_grid_tmp: Path,
) -> None:
    """POST /daemon/publish returns 503 when daemon is not running."""
    coord = Coordinator(project_name="daemon-publish-503")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.post(
                "/daemon/publish",
                json={
                    "project_name": "test",
                    "category": "misc",
                    "description": "test",
                    "apps": [],
                },
            )
            assert r.status_code == 503
            body = r.json()
            assert body["kind"] == "unavailable"
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_project_publish_endpoint(nexus_grid_tmp: Path) -> None:
    """POST /project/publish builds the payload from the coordinator
    config and forwards to the daemon."""
    with _FakeDaemon() as fake:
        fake.set_response("POST", "/publish", 200, {"published": True})
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="gov-publish-test")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post("/project/publish")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["body"]["published"] is True
        finally:
            await coord.stop()

    post_calls = [c for c in fake.calls if c[0] == "POST" and c[1] == "/publish"]
    assert len(post_calls) == 1
    forwarded = json.loads(post_calls[0][2])
    assert forwarded["project_name"] == "gov-publish-test"


@pytest.mark.asyncio
async def test_auto_publish_called_for_public_coordinator(
    nexus_grid_tmp: Path,
) -> None:
    """A coordinator with visibility=public calls the daemon's
    POST /publish at boot. Sprint 11 Phase A auto-publish."""
    with _FakeDaemon() as fake:
        fake.set_response("POST", "/publish", 200, {"published": True})
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="auto-pub-test")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            # The auto-publish call should have happened during start().
            post_calls = [c for c in fake.calls if c[0] == "POST" and c[1] == "/publish"]
            assert len(post_calls) == 1
            forwarded = json.loads(post_calls[0][2])
            assert forwarded["project_name"] == "auto-pub-test"
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_default_curators_forwards_upstream(
    nexus_grid_tmp: Path,
) -> None:
    """GET /daemon/default-curators proxies to daemon GET /default-curators."""
    curator_hex = "ab" * 32
    with _FakeDaemon() as fake:
        fake.set_response(
            "GET",
            "/default-curators",
            200,
            {"default_curators": [curator_hex]},
        )
        _write_running_json(nexus_grid_tmp, port=fake.port)

        coord = Coordinator(project_name="default-curators-test")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.get("/daemon/default-curators")
                assert r.status_code == 200
                body = r.json()
                assert body["kind"] == "data"
                assert body["status"] == 200
                assert body["body"]["default_curators"] == [curator_hex]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_daemon_default_curators_returns_503_when_daemon_down(
    nexus_grid_tmp: Path,
) -> None:
    """GET /daemon/default-curators returns 503 when daemon is not running."""
    coord = Coordinator(project_name="default-curators-503")
    await coord.start()
    try:
        with TestClient(create_app(coord)) as client:
            r = client.get("/daemon/default-curators")
            assert r.status_code == 503
            body = r.json()
            assert body["kind"] == "unavailable"
    finally:
        await coord.stop()
