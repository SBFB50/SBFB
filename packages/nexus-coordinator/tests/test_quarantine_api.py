# SPDX-License-Identifier: AGPL-3.0-or-later
"""Quarantine REST API tests — Sprint 21 Phase D.

Two test families:

1. **Endpoint contract tests** mount only the quarantine router on
   a stub coordinator (no auth middleware) — confirms list/flush/
   drop wiring + JSON shape + 404 on missing row + 503 when the
   queue is uninitialised. Pattern miroir ``test_api_canary.py``.
2. **Auth integration tests** mount the quarantine router behind
   :class:`LoopbackAuthMiddleware` and verify every endpoint
   inherits the bearer/Host/Origin triple check (cf.
   ``test_auth.py`` patterns).
"""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from nexus_coordinator.api.quarantine import router as quarantine_router
from nexus_coordinator.auth import AUTH_HEADER, LoopbackAuthMiddleware
from nexus_coordinator.quarantine_queue import QuarantineQueue

_VALID_TOKEN = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef"


class _StubCoordinator:
    def __init__(self, queue: QuarantineQueue | None) -> None:
        self.quarantine_queue = queue


def _build_app(queue: QuarantineQueue | None, *, with_auth: bool = False) -> FastAPI:
    app = FastAPI()
    app.state.coordinator = _StubCoordinator(queue)  # type: ignore[attr-defined]
    if with_auth:
        app.add_middleware(LoopbackAuthMiddleware, token=_VALID_TOKEN)
    app.include_router(quarantine_router)
    return app


@pytest.fixture
async def started_queue(tmp_path: Path) -> QuarantineQueue:
    queue = QuarantineQueue(db_path=tmp_path / "q.sqlite", sweep_interval_s=60.0)
    await queue.init()
    return queue


# ----------------------------------------------------------------------
# Endpoint contract tests
# ----------------------------------------------------------------------


@pytest.mark.asyncio
async def test_list_endpoint_returns_added_entry(started_queue: QuarantineQueue) -> None:
    await started_queue.add(
        topic="api/test/v1",
        sender_pubkey=bytes(range(32)),
        payload_bytes=b"\xab\xcd",
        rate_strikes=3,
        pow_status="missing",
    )
    app = _build_app(started_queue)
    with TestClient(app) as client:
        resp = client.get("/quarantine/list")
    assert resp.status_code == 200
    data = resp.json()
    assert data["count"] == 1
    entry = data["entries"][0]
    assert entry["topic"] == "api/test/v1"
    assert entry["sender_pubkey_hex"] == bytes(range(32)).hex()
    assert entry["payload_bytes_hex"] == "abcd"
    assert entry["rate_strikes"] == 3
    assert entry["pow_status"] == "missing"
    assert entry["flush_status"] == "pending"


@pytest.mark.asyncio
async def test_flush_endpoint_then_drop_returns_404(started_queue: QuarantineQueue) -> None:
    row_id = await started_queue.add(
        topic="t",
        sender_pubkey=b"\x01" * 32,
        payload_bytes=b"x",
        rate_strikes=0,
        pow_status="valid",
    )
    app = _build_app(started_queue)
    with TestClient(app) as client:
        ok = client.post(f"/quarantine/flush/{row_id}")
        already = client.post(f"/quarantine/drop/{row_id}")
    assert ok.status_code == 200
    assert ok.json()["new_status"] == "flushed"
    assert already.status_code == 404


def test_list_endpoint_503_when_queue_missing() -> None:
    """If ``coord.quarantine_queue`` is None the endpoint must
    return 503 Service Unavailable rather than crash."""
    app = _build_app(None)
    with TestClient(app) as client:
        resp = client.get("/quarantine/list")
    assert resp.status_code == 503


# ----------------------------------------------------------------------
# Auth integration tests
# ----------------------------------------------------------------------


def _loopback_headers(token: str = _VALID_TOKEN) -> dict[str, str]:
    return {AUTH_HEADER: token, "Host": "127.0.0.1:7777"}


@pytest.mark.asyncio
async def test_bearer_auth_required(started_queue: QuarantineQueue) -> None:
    """Without ``X-SBFB-Token`` the endpoint returns 401 (Sprint 16
    triple check inherited from the global middleware)."""
    app = _build_app(started_queue, with_auth=True)
    with TestClient(app) as client:
        # TestClient injects its default Host header automatically — strip
        # the auth one only.
        client.headers.pop(AUTH_HEADER, None)
        resp = client.get("/quarantine/list", headers={"Host": "127.0.0.1:7777"})
    assert resp.status_code == 401


@pytest.mark.asyncio
async def test_host_origin_check(started_queue: QuarantineQueue) -> None:
    """A non-loopback Host header trips the DNS-rebind defence
    (CVE-2025-49596) — the middleware returns 403."""
    app = _build_app(started_queue, with_auth=True)
    with TestClient(app) as client:
        resp = client.get(
            "/quarantine/list",
            headers={AUTH_HEADER: _VALID_TOKEN, "Host": "attacker.com"},
        )
    assert resp.status_code == 403


@pytest.mark.asyncio
async def test_flush_endpoint_protected_by_auth(started_queue: QuarantineQueue) -> None:
    """flush/{id} also rejects unauthenticated callers — confirms
    the middleware applies to every quarantine route, not just
    list."""
    row_id = await started_queue.add(
        topic="t",
        sender_pubkey=b"\x01" * 32,
        payload_bytes=b"x",
        rate_strikes=0,
        pow_status="valid",
    )
    app = _build_app(started_queue, with_auth=True)
    with TestClient(app) as client:
        client.headers.pop(AUTH_HEADER, None)
        unauth = client.post(
            f"/quarantine/flush/{row_id}",
            headers={"Host": "127.0.0.1:7777"},
        )
        ok = client.post(
            f"/quarantine/flush/{row_id}",
            headers=_loopback_headers(),
        )
    assert unauth.status_code == 401
    assert ok.status_code == 200
