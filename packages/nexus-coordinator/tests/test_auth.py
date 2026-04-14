# SPDX-License-Identifier: AGPL-3.0-or-later
"""Unit tests for :mod:`nexus_coordinator.auth`.

Sprint 16 Phase A (D1). Mirrors the Rust side tests in
``crates/nexus-shell-daemon-core/src/auth.rs`` so the two
implementations cannot drift silently.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from nexus_coordinator.auth import (
    AUTH_HEADER,
    AUTH_TOKEN_ENV,
    TOKEN_HEX_LEN,
    LoopbackAuthMiddleware,
    auth_token_path,
    coordinator_socket_path,
    is_loopback_host,
    is_loopback_origin,
    load_token,
    sbfb_home,
    sbfb_run_dir,
)

_VALID_TOKEN = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef"


def _build_app_with_auth(token: str) -> FastAPI:
    app = FastAPI()
    app.add_middleware(LoopbackAuthMiddleware, token=token)

    @app.get("/health")
    async def health() -> dict[str, str]:
        return {"status": "ok"}

    @app.get("/protected")
    async def protected() -> dict[str, str]:
        return {"secret": "yes"}

    return app


def test_is_loopback_host_matches_expected() -> None:
    assert is_loopback_host("localhost")
    assert is_loopback_host("localhost:7777")
    assert is_loopback_host("127.0.0.1")
    assert is_loopback_host("127.0.0.1:8080")
    assert is_loopback_host("[::1]")
    assert is_loopback_host("[::1]:7777")
    assert not is_loopback_host("attacker.com")
    assert not is_loopback_host("0.0.0.0")
    assert not is_loopback_host("example.com:7777")
    assert not is_loopback_host("localhost:not-a-port")
    assert not is_loopback_host("")


def test_is_loopback_origin_matches_expected() -> None:
    assert is_loopback_origin("http://localhost")
    assert is_loopback_origin("http://localhost:5173")
    assert is_loopback_origin("http://127.0.0.1:8080")
    assert is_loopback_origin("http://[::1]:7777")
    assert not is_loopback_origin("https://localhost")
    assert not is_loopback_origin("http://attacker.com")
    assert not is_loopback_origin("http://localhost/path")


def test_middleware_rejects_wrong_length_token() -> None:
    app = FastAPI()
    with pytest.raises(ValueError):
        app.add_middleware(LoopbackAuthMiddleware, token="short")
        # FastAPI lazily builds middleware — trigger by calling the factory
        app.build_middleware_stack()


def test_health_is_public() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers.pop(AUTH_HEADER, None)
    resp = client.get("/health")
    assert resp.status_code == 200
    assert resp.json() == {"status": "ok"}


def test_protected_rejects_missing_token() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers.pop(AUTH_HEADER, None)
    resp = client.get("/protected")
    assert resp.status_code == 401


def test_protected_rejects_wrong_token() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers[AUTH_HEADER] = "abcdef" * 10 + "abcd"  # wrong value, right len
    resp = client.get("/protected")
    assert resp.status_code == 401


def test_protected_accepts_valid_triple() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers[AUTH_HEADER] = _VALID_TOKEN
    resp = client.get("/protected")
    assert resp.status_code == 200
    assert resp.json() == {"secret": "yes"}


def test_protected_rejects_rebound_host() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app, base_url="http://attacker.com")
    client.headers[AUTH_HEADER] = _VALID_TOKEN
    resp = client.get("/protected")
    assert resp.status_code == 403


def test_protected_rejects_cross_origin() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers[AUTH_HEADER] = _VALID_TOKEN
    resp = client.get("/protected", headers={"origin": "https://attacker.com"})
    assert resp.status_code == 403


def test_protected_accepts_loopback_origin() -> None:
    app = _build_app_with_auth(_VALID_TOKEN)
    client = TestClient(app)
    client.headers[AUTH_HEADER] = _VALID_TOKEN
    resp = client.get(
        "/protected",
        headers={"origin": "http://localhost:5173"},
    )
    assert resp.status_code == 200


def test_load_token_reads_env_var(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv(AUTH_TOKEN_ENV, _VALID_TOKEN)
    assert load_token() == _VALID_TOKEN


def test_load_token_reads_file_when_env_absent(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv(AUTH_TOKEN_ENV, raising=False)
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    path = auth_token_path()
    assert path is not None
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(_VALID_TOKEN, encoding="utf-8")
    assert load_token() == _VALID_TOKEN


def test_load_token_rejects_malformed_file(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv(AUTH_TOKEN_ENV, raising=False)
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    path = auth_token_path()
    assert path is not None
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("not-hex!!", encoding="utf-8")
    assert load_token() is None


def test_load_token_rejects_wrong_length(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv(AUTH_TOKEN_ENV, raising=False)
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    path = auth_token_path()
    assert path is not None
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("deadbeef", encoding="utf-8")
    assert load_token() is None


def test_sbfb_home_honours_override(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    assert sbfb_home() == tmp_path


def test_token_hex_len_constant() -> None:
    assert TOKEN_HEX_LEN == 64


# =================================================================
# Sprint 16 Phase B (D2): UDS path helpers
# =================================================================


def test_sbfb_run_dir_resolves_under_home(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    assert sbfb_run_dir() == tmp_path / "run"


def test_coordinator_socket_path_under_run_dir(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("SBFB_HOME", str(tmp_path))
    assert coordinator_socket_path() == tmp_path / "run" / "coordinator.sock"


def test_sbfb_run_dir_returns_none_without_home(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.delenv("SBFB_HOME", raising=False)
    monkeypatch.delenv("HOME", raising=False)
    monkeypatch.delenv("USERPROFILE", raising=False)
    assert sbfb_run_dir() is None
    assert coordinator_socket_path() is None
