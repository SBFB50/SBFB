# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 12 Phase B — tests for ``POST /project/deploy`` + auto-publish archive.

4 scenarios:
1. deploy_valid_zip — upload a valid zip → 200 + hash
2. deploy_invalid_zip — upload garbage → 400
3. deploy_missing_index — zip without index.html → 400
4. deploy_oversized_zip — upload > MAX_DEPLOY_BYTES → 413
"""

from __future__ import annotations

import io
import json
import threading
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator

# ---------------------------------------------------------------
# Fake daemon that answers POST /publish-blob and POST /publish
# ---------------------------------------------------------------


class _FakeDaemon:
    def __init__(self) -> None:
        self._server: ThreadingHTTPServer | None = None
        self._thread: threading.Thread | None = None
        self.host: str = "127.0.0.1"
        self.port: int = 0
        self.calls: list[tuple[str, str, bytes]] = []
        self._lock = threading.Lock()

    def start(self) -> None:
        fake = self

        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_args: Any, **_kwargs: Any) -> None:
                return

            def do_POST(self) -> None:
                content_length = int(self.headers.get("Content-Length", "0") or "0")
                body = self.rfile.read(content_length) if content_length else b""
                with fake._lock:
                    fake.calls.append(("POST", self.path, body))

                if self.path == "/publish-blob":
                    resp = json.dumps({"hash": "ab" * 32}).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(resp)))
                    self.end_headers()
                    self.wfile.write(resp)
                elif self.path == "/publish":
                    resp = json.dumps({"published": True}).encode("utf-8")
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(resp)))
                    self.end_headers()
                    self.wfile.write(resp)
                else:
                    self.send_error(404)

        self._server = ThreadingHTTPServer((self.host, 0), Handler)
        self.port = self._server.server_address[1]
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)
        self._thread.start()

    def stop(self) -> None:
        if self._server:
            self._server.shutdown()
            self._server.server_close()
        if self._thread:
            self._thread.join(timeout=5)

    def __enter__(self) -> "_FakeDaemon":
        self.start()
        return self

    def __exit__(self, *_exc: Any) -> None:
        self.stop()


def _write_running_json(root: Path, *, port: int) -> None:
    path = root / "shell-daemon" / "running.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    state = {
        "schema_version": 1,
        "node_id": "de" * 32,
        "api_host": "127.0.0.1",
        "api_port": port,
        "pid": 424242,
        "started_at": "2026-04-13T12:00:00Z",
        "daemon_version": "0.1.0-test",
    }
    path.write_text(json.dumps(state), encoding="utf-8")


def _make_zip(files: dict[str, str]) -> bytes:
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as zf:
        for name, content in files.items():
            zf.writestr(name, content)
    return buf.getvalue()


# ---------------------------------------------------------------
# Tests
# ---------------------------------------------------------------


@pytest.mark.asyncio
async def test_deploy_valid_zip(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy with a valid zip → 200 + hash."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-test")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"index.html": "<h1>Hello</h1>"})
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("app.zip", zip_bytes, "application/zip")},
                )
                assert r.status_code == 200
                body = r.json()
                assert body["deployed"] is True
                assert body["hash"] == "ab" * 32

                # Verify daemon was called
                assert any(p == "/publish-blob" for _, p, _ in daemon.calls)
                assert any(p == "/publish" for _, p, _ in daemon.calls)
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_invalid_zip(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy with garbage bytes → 400."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-bad")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("bad.zip", b"not a zip", "application/zip")},
                )
                assert r.status_code == 400
                assert "invalid zip" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_missing_index_html(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy with a zip without index.html → 400."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-no-index")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"app.js": "console.log('hi')"})
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("app.zip", zip_bytes, "application/zip")},
                )
                assert r.status_code == 400
                assert "index.html" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_public_without_repo_url_rejected(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy for a public project without repo_url → 400."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-public-no-repo")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"index.html": "<h1>Hello</h1>"})
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("app.zip", zip_bytes, "application/zip")},
                )
                assert r.status_code == 400
                assert "repo_url" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_public_with_repo_url_accepted(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy for a public project with repo_url → 200."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-public-with-repo")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"index.html": "<h1>Hello</h1>"})
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("app.zip", zip_bytes, "application/zip")},
                    data={"repo_url": "https://github.com/example/app"},
                )
                assert r.status_code == 200
                body = r.json()
                assert body["deployed"] is True
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_private_without_repo_url_accepted(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy for a private project without repo_url → 200."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-private-no-repo")
        # visibility defaults to "private" — no repo_url needed
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"index.html": "<h1>Hello</h1>"})
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("app.zip", zip_bytes, "application/zip")},
                )
                assert r.status_code == 200
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_oversized_zip(nexus_grid_tmp: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    """POST /project/deploy with a zip > MAX_DEPLOY_BYTES → 413."""
    import nexus_coordinator.api.deploy as deploy_mod

    # Patch to a tiny limit so the test stays fast.
    monkeypatch.setattr(deploy_mod, "MAX_DEPLOY_BYTES", 64)

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-too-big")
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                zip_bytes = _make_zip({"index.html": "x" * 200})
                assert len(zip_bytes) > 64  # sanity: zip exceeds patched limit
                r = client.post(
                    "/project/deploy",
                    files={"archive": ("big.zip", zip_bytes, "application/zip")},
                )
                assert r.status_code == 413
                assert "maximum allowed size" in r.json()["detail"]
        finally:
            await coord.stop()
