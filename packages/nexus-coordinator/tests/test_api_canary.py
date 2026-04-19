# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated canary registry API tests.

Sprint 21 Phase E (T-NN+1 tech debt resolved) update : the
``POST /api/canary/observed`` endpoint now verifies the Ed25519
signature at ingest. Tests use ``nexus_core.build_canary`` to
produce real signed canaries instead of the previous forged
payloads (which the new verify path correctly rejects with HTTP
401).
"""

from __future__ import annotations

import json
import secrets
from pathlib import Path

import nexus_core
from fastapi import FastAPI
from fastapi.testclient import TestClient
from nexus_coordinator.api.canary import router as canary_router
from nexus_coordinator.canary_registry import CanaryRegistry


class _StubCoordinator:
    """Minimal coordinator stand-in for the API integration test.

    The :func:`network_health` endpoint only needs
    ``coord.canary_registry`` — mocking the rest of the
    coordinator out lets the test focus on the wire shape
    without booting an iroh node.
    """

    def __init__(self, registry: CanaryRegistry) -> None:
        self.canary_registry = registry


def _build_app(registry: CanaryRegistry) -> FastAPI:
    app = FastAPI()
    app.state.coordinator = _StubCoordinator(registry)  # type: ignore[attr-defined]
    app.include_router(canary_router)
    return app


def _signed_canary_payload(headline: str = "API test headline") -> dict[str, object]:
    """Return a freshly-signed canary wire payload via the Rust
    ``build_canary`` PyO3 binding. The returned dict is the flat
    JSON shape the daemon emits over gossip — the same shape the
    ``observed`` endpoint accepts."""
    secret = secrets.token_bytes(32)
    canary_json = nexus_core.build_canary("2026-04-15", headline, secret)
    return json.loads(canary_json)


def test_api_canary_network_health_empty_registry_returns_expected_shape(tmp_path: Path) -> None:
    """``GET /api/canary/network-health`` returns the
    NetworkHealth schema FastAPI dumps from pydantic, even when
    the registry is empty (zero maintainers, all summary
    counters at 0)."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    resp = client.get("/api/canary/network-health")
    assert resp.status_code == 200, resp.text
    data = resp.json()
    assert set(data.keys()) == {"summary", "maintainers", "observed_at"}
    assert data["maintainers"] == []
    assert data["summary"]["maintainers_total"] == 0
    assert data["summary"]["canary_fresh"] == 0
    for k in (
        "canary_fresh",
        "canary_warn",
        "canary_stale",
        "canary_missing",
        "duress_ack_fresh",
        "duress_ack_warn",
        "duress_ack_stale",
        "duress_ack_missing",
    ):
        assert k in data["summary"]
    assert isinstance(data["observed_at"], str)
    assert data["observed_at"].startswith("20")  # plausible RFC 3339


def test_observed_endpoint_accepts_valid_canary(tmp_path: Path) -> None:
    """A canary signed via the Rust ``build_canary`` path passes
    the Ed25519 verify-at-ingest check (Sprint 21 Phase E T-NN+1)
    and is recorded in the registry."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    payload = _signed_canary_payload()
    post = client.post(
        "/api/canary/observed",
        json={"kind": "canary", "payload": payload},
    )
    assert post.status_code == 200, post.text
    assert post.json() == {"status": "observed", "kind": "canary"}

    re_resp = client.get("/api/canary/network-health")
    assert re_resp.status_code == 200
    re_data = re_resp.json()
    assert re_data["summary"]["maintainers_total"] == 1
    assert len(re_data["maintainers"]) == 1
    entry = re_data["maintainers"][0]
    assert entry["pubkey_hex"] == payload["pubkey_hex"]
    assert entry["canary_date"] == "2026-04-15"
    assert entry["canary_status"] in {"fresh", "warn", "stale"}


def test_observed_endpoint_rejects_malformed_signature(tmp_path: Path) -> None:
    """A canary with a forged ``signature_hex`` is rejected with
    HTTP 401 by the verify-at-ingest gate (Sprint 21 Phase E
    T-NN+1) — the registry must NOT record it."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    payload = _signed_canary_payload()
    payload["signature_hex"] = "b" * 128  # tamper

    post = client.post(
        "/api/canary/observed",
        json={"kind": "canary", "payload": payload},
    )
    assert post.status_code == 401, post.text
    assert "signature verification failed" in post.json()["detail"]

    re_resp = client.get("/api/canary/network-health")
    assert re_resp.status_code == 200
    assert re_resp.json()["summary"]["maintainers_total"] == 0


def test_observed_endpoint_rejects_missing_fields(tmp_path: Path) -> None:
    """A payload missing required canary fields trips the Rust
    JSON parse before the cryptographic verify, surfaced as
    HTTP 401 (the verify-at-ingest gate is the first guard)."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    bad = client.post(
        "/api/canary/observed",
        json={"kind": "canary", "payload": {"oops": "missing fields"}},
    )
    assert bad.status_code == 401


def test_observed_endpoint_rejects_unknown_kind(tmp_path: Path) -> None:
    """``kind`` outside {canary, duress_ack} returns 400 before
    any verify is attempted (the dispatch happens first)."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    payload = _signed_canary_payload()
    unknown = client.post(
        "/api/canary/observed",
        json={"kind": "garbage", "payload": payload},
    )
    assert unknown.status_code == 400
