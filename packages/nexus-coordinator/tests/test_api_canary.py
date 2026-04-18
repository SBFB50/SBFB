# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated canary registry API tests."""

from __future__ import annotations

from pathlib import Path

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


def test_api_canary_network_health_returns_expected_shape(tmp_path: Path) -> None:
    """``GET /api/canary/network-health`` returns the
    NetworkHealth schema FastAPI dumps from pydantic, even when
    the registry is empty (zero maintainers, all summary
    counters at 0)."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    app = _build_app(reg)
    client = TestClient(app)

    # Empty registry baseline — schema must still hold up.
    resp = client.get("/api/canary/network-health")
    assert resp.status_code == 200, resp.text
    data = resp.json()
    assert set(data.keys()) == {"summary", "maintainers", "observed_at"}
    assert data["maintainers"] == []
    assert data["summary"]["maintainers_total"] == 0
    assert data["summary"]["canary_fresh"] == 0
    # Summary keys exist for every status bucket — front-end can
    # render them directly without nullability handling.
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

    # Now POST a canary observation through the API and
    # re-query — the registry must surface it.
    pk = ("a" * 63) + "f"
    canary_payload = {
        "v": 1,  # confirms the v -> version coercion path
        "date": "2026-04-15",
        "headline": "API test headline",
        "next_update": "2026-05-30",
        "pubkey_hex": pk,
        "signature_hex": "b" * 128,
    }
    post = client.post(
        "/api/canary/observed",
        json={"kind": "canary", "payload": canary_payload},
    )
    assert post.status_code == 200, post.text
    assert post.json() == {"status": "observed", "kind": "canary"}

    re_resp = client.get("/api/canary/network-health")
    assert re_resp.status_code == 200
    re_data = re_resp.json()
    assert re_data["summary"]["maintainers_total"] == 1
    assert len(re_data["maintainers"]) == 1
    entry = re_data["maintainers"][0]
    assert entry["pubkey_hex"] == pk
    assert entry["canary_date"] == "2026-04-15"
    # status depends on today's wall clock — just assert one of
    # the legal values to keep the test deterministic.
    assert entry["canary_status"] in {"fresh", "warn", "stale"}

    # Bad payload shape -> 422.
    bad = client.post(
        "/api/canary/observed",
        json={"kind": "canary", "payload": {"oops": "missing fields"}},
    )
    assert bad.status_code == 422

    # Unknown kind -> 400.
    unknown = client.post(
        "/api/canary/observed",
        json={"kind": "garbage", "payload": canary_payload},
    )
    assert unknown.status_code == 400
