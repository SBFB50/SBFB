# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for deploy endpoints.

Sprint 12 Phase B — ``POST /project/deploy`` (upload zip).
Sprint 14 Phase A — ``POST /project/deploy-from-repo`` (clone + verify).
"""

from __future__ import annotations

import io
import json
import threading
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from unittest.mock import AsyncMock, patch

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
    """POST /project/deploy for a public project → 400 redirect to deploy-from-repo."""
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
                assert "deploy-from-repo" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_public_with_repo_url_also_rejected(nexus_grid_tmp: Path) -> None:
    """POST /project/deploy for a public project even with repo_url → 400."""
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
                assert r.status_code == 400
                assert "deploy-from-repo" in r.json()["detail"]
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


# ---------------------------------------------------------------
# Sprint 14 — deploy-from-repo tests
# ---------------------------------------------------------------


def _create_fake_repo(tmpdir: Path, *, node_id: str = "de" * 32) -> Path:
    """Create a minimal fake repo directory with SBFB.json + index.html."""
    repo = tmpdir / "fake-repo"
    repo.mkdir(parents=True, exist_ok=True)
    (repo / "SBFB.json").write_text(
        json.dumps({"node_id": node_id, "project_name": "test-app"}),
        encoding="utf-8",
    )
    (repo / "index.html").write_text("<h1>Hello SBFB</h1>", encoding="utf-8")
    # Add a .git directory (should be excluded from zip).
    (repo / ".git").mkdir()
    (repo / ".git" / "config").write_text("[core]", encoding="utf-8")
    return repo


def _make_mock_clone(source_dir: Path):
    """Create a mock clone function that copies from a local directory."""
    import shutil

    async def _mock_clone(repo_url: str, dest: str, *, ref: str | None = None) -> None:
        shutil.copytree(str(source_dir), dest, dirs_exist_ok=True)

    return _mock_clone


@pytest.mark.asyncio
async def test_deploy_from_repo_happy_path(
    nexus_grid_tmp: Path,
    tmp_path: Path,
) -> None:
    """deploy-from-repo with valid repo → 200 + hash + provenance_hash."""
    fake_repo = _create_fake_repo(tmp_path, node_id="de" * 32)

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-repo-test")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with (
                    patch(
                        "nexus_coordinator.api.deploy._clone_repo",
                        side_effect=_make_mock_clone(fake_repo),
                    ),
                    patch(
                        "nexus_coordinator.api.deploy.is_repo_public",
                        new_callable=AsyncMock,
                        return_value=True,
                    ),
                    patch(
                        "nexus_coordinator.api.deploy._git_rev_parse",
                        new_callable=AsyncMock,
                        return_value="abc123def456",
                    ),
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={
                            "repo_url": "https://github.com/test/app",
                        },
                    )
                    assert r.status_code == 200, r.json()
                    body = r.json()
                    assert body["deployed"] is True
                    assert body["hash"] == "ab" * 32
                    assert "provenance_hash" in body
                    assert body["commit_sha"] == "abc123def456"

                    # Verify daemon received /publish-blob and /publish.
                    paths_called = [p for _, p, _ in daemon.calls]
                    assert "/publish-blob" in paths_called
                    assert "/publish" in paths_called

                    # Verify the LAST publish payload has provenance_hash
                    # (the first /publish is from auto-publish on boot).
                    publish_calls = [b for _, p, b in daemon.calls if p == "/publish"]
                    assert len(publish_calls) >= 2
                    publish_body = json.loads(publish_calls[-1])
                    assert "provenance_hash" in publish_body
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_missing_sbfb_json(
    nexus_grid_tmp: Path,
    tmp_path: Path,
) -> None:
    """deploy-from-repo with no SBFB.json → 400."""
    fake_repo = tmp_path / "no-sbfb"
    fake_repo.mkdir()
    (fake_repo / "index.html").write_text("<h1>Hi</h1>", encoding="utf-8")

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-no-sbfb")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with (
                    patch(
                        "nexus_coordinator.api.deploy._clone_repo",
                        side_effect=_make_mock_clone(fake_repo),
                    ),
                    patch(
                        "nexus_coordinator.api.deploy.is_repo_public",
                        new_callable=AsyncMock,
                        return_value=True,
                    ),
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={"repo_url": "https://github.com/test/no-sbfb"},
                    )
                    assert r.status_code == 400
                    assert "SBFB.json" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_wrong_node_id(
    nexus_grid_tmp: Path,
    tmp_path: Path,
) -> None:
    """deploy-from-repo with wrong node_id in SBFB.json → 400."""
    fake_repo = _create_fake_repo(tmp_path, node_id="aa" * 32)

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)  # node_id = "de" * 32
        coord = Coordinator(project_name="deploy-wrong-id")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with (
                    patch(
                        "nexus_coordinator.api.deploy._clone_repo",
                        side_effect=_make_mock_clone(fake_repo),
                    ),
                    patch(
                        "nexus_coordinator.api.deploy.is_repo_public",
                        new_callable=AsyncMock,
                        return_value=True,
                    ),
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={"repo_url": "https://github.com/test/wrong-id"},
                    )
                    assert r.status_code == 400
                    assert "node_id" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_missing_index_html(
    nexus_grid_tmp: Path,
    tmp_path: Path,
) -> None:
    """deploy-from-repo without index.html → 400."""
    fake_repo = tmp_path / "no-index"
    fake_repo.mkdir()
    (fake_repo / "SBFB.json").write_text(
        json.dumps({"node_id": "de" * 32, "project_name": "x"}),
        encoding="utf-8",
    )

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-no-index")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with (
                    patch(
                        "nexus_coordinator.api.deploy._clone_repo",
                        side_effect=_make_mock_clone(fake_repo),
                    ),
                    patch(
                        "nexus_coordinator.api.deploy.is_repo_public",
                        new_callable=AsyncMock,
                        return_value=True,
                    ),
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={"repo_url": "https://github.com/test/no-index"},
                    )
                    assert r.status_code == 400
                    assert "index.html" in r.json()["detail"]
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_private_rejected(
    nexus_grid_tmp: Path,
) -> None:
    """deploy-from-repo for a private project → 400."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-private")
        # visibility defaults to "private"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                r = client.post(
                    "/project/deploy-from-repo",
                    json={"repo_url": "https://github.com/a/b"},
                )
                assert r.status_code == 400
                assert "public" in r.json()["detail"].lower()
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_provenance_in_zip(
    nexus_grid_tmp: Path,
    tmp_path: Path,
) -> None:
    """deploy-from-repo includes provenance.json in the stored zip."""
    fake_repo = _create_fake_repo(tmp_path, node_id="de" * 32)

    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-prov-zip")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with (
                    patch(
                        "nexus_coordinator.api.deploy._clone_repo",
                        side_effect=_make_mock_clone(fake_repo),
                    ),
                    patch(
                        "nexus_coordinator.api.deploy.is_repo_public",
                        new_callable=AsyncMock,
                        return_value=True,
                    ),
                    patch(
                        "nexus_coordinator.api.deploy._git_rev_parse",
                        new_callable=AsyncMock,
                        return_value="abc123",
                    ),
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={"repo_url": "https://github.com/test/prov-app"},
                    )
                    assert r.status_code == 200

                    # Find the LAST blob (first is auto-publish on boot).
                    blob_calls = [b for _, p, b in daemon.calls if p == "/publish-blob"]
                    assert len(blob_calls) >= 2
                    blob_body = blob_calls[-1]
                    # Verify the blob is a valid zip containing provenance.json.
                    with zipfile.ZipFile(io.BytesIO(blob_body)) as zf:
                        names = zf.namelist()
                        assert "provenance.json" in names
                        assert "index.html" in names
                        # .git/ should be excluded.
                        assert not any(n.startswith(".git/") for n in names)
                        # Verify provenance.json is valid JSON.
                        prov = json.loads(zf.read("provenance.json"))
                        assert prov["schema_version"] == 1
                        assert prov["repo_url"] == "https://github.com/test/prov-app"
                        assert "signature" in prov
        finally:
            await coord.stop()


@pytest.mark.asyncio
async def test_deploy_from_repo_repo_not_public(
    nexus_grid_tmp: Path,
) -> None:
    """deploy-from-repo with a private repo → 400."""
    with _FakeDaemon() as daemon:
        _write_running_json(nexus_grid_tmp, port=daemon.port)
        coord = Coordinator(project_name="deploy-not-public")
        coord.config.network.visibility = "public"
        await coord.start()
        try:
            with TestClient(create_app(coord)) as client:
                with patch(
                    "nexus_coordinator.api.deploy.is_repo_public",
                    new_callable=AsyncMock,
                    return_value=False,
                ):
                    r = client.post(
                        "/project/deploy-from-repo",
                        json={"repo_url": "https://github.com/private/repo"},
                    )
                    assert r.status_code == 400
                    assert "public" in r.json()["detail"].lower()
        finally:
            await coord.stop()
