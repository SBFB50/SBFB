# SPDX-License-Identifier: AGPL-3.0-or-later
"""Unit tests for :mod:`nexus_coordinator.api.consent`.

Sprint 16 Phase C. Pairs with the Rust worker tests in
``crates/nexus-worker-core/src/consent.rs``: the two sides must
read/write byte-identical JSON or the dialog will silently desync
from the worker enforcement.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient
from nexus_coordinator.api.consent import (
    SBFB_HOME_ENV,
    consent_path,
)
from nexus_coordinator.api.consent import (
    router as consent_router,
)

_NODE_ID_A = "a" * 64
_NODE_ID_B = "b" * 64


@pytest.fixture
def consent_client(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> TestClient:
    """Build a TestClient that hits the consent router with
    ``SBFB_HOME`` redirected at a hermetic ``tmp_path``."""
    monkeypatch.setenv(SBFB_HOME_ENV, str(tmp_path / "sbfb"))
    app = FastAPI()
    app.include_router(consent_router)
    return TestClient(app)


def test_get_returns_defaults_when_file_missing(consent_client: TestClient) -> None:
    response = consent_client.get("/consent/get")
    assert response.status_code == 200
    body = response.json()
    assert body["level"] == 1
    assert body["caps"]["max_watts"] == 400
    assert body["allowed_project_ids"] == []
    assert body["own_node_id"] == ""


def test_set_persists_full_payload(consent_client: TestClient) -> None:
    payload = {
        "level": 4,
        "caps": {"max_watts": 250, "max_vram_mb": 8192, "max_hours_day": 6.0},
        "allowed_project_ids": [_NODE_ID_A],
        "own_node_id": "self",
    }
    response = consent_client.post("/consent/set", json=payload)
    assert response.status_code == 200
    saved = response.json()
    assert saved["level"] == 4

    # Re-read via GET to confirm it round-trips through disk.
    again = consent_client.get("/consent/get").json()
    assert again["level"] == 4
    assert again["caps"]["max_watts"] == 250
    assert again["allowed_project_ids"] == [_NODE_ID_A]


def test_set_rejects_invalid_level(consent_client: TestClient) -> None:
    payload = {
        "level": 9,
        "caps": {"max_watts": 100, "max_vram_mb": 1024, "max_hours_day": 1.0},
        "allowed_project_ids": [],
        "own_node_id": "self",
    }
    response = consent_client.post("/consent/set", json=payload)
    assert response.status_code == 422  # Pydantic Literal validation fail


def test_set_rejects_malformed_node_id(consent_client: TestClient) -> None:
    payload = {
        "level": 3,
        "caps": {"max_watts": 100, "max_vram_mb": 1024, "max_hours_day": 1.0},
        "allowed_project_ids": ["not-hex"],
        "own_node_id": "self",
    }
    response = consent_client.post("/consent/set", json=payload)
    assert response.status_code == 422


def test_whitelist_add_appends_unique_node_id(consent_client: TestClient) -> None:
    add = consent_client.post("/consent/whitelist/add", json={"project_id": _NODE_ID_A})
    assert add.status_code == 200
    assert add.json()["allowed_project_ids"] == [_NODE_ID_A]

    # Idempotent: adding the same id is a no-op, no duplicate.
    again = consent_client.post("/consent/whitelist/add", json={"project_id": _NODE_ID_A})
    assert again.status_code == 200
    assert again.json()["allowed_project_ids"] == [_NODE_ID_A]

    # Adding a different id appends.
    second = consent_client.post("/consent/whitelist/add", json={"project_id": _NODE_ID_B})
    assert second.status_code == 200
    assert sorted(second.json()["allowed_project_ids"]) == sorted([_NODE_ID_A, _NODE_ID_B])


def test_whitelist_add_rejects_repo_url_stub(consent_client: TestClient) -> None:
    response = consent_client.post(
        "/consent/whitelist/add",
        json={"repo_url": "https://github.com/example/repo"},
    )
    assert response.status_code == 422
    # Surface the resolution-not-wired hint so callers know what to do.
    assert "node_id" in response.json()["detail"]


def test_whitelist_remove_is_idempotent(consent_client: TestClient, tmp_path: Path) -> None:
    # Pre-populate with two ids.
    consent_client.post("/consent/whitelist/add", json={"project_id": _NODE_ID_A})
    consent_client.post("/consent/whitelist/add", json={"project_id": _NODE_ID_B})

    rm = consent_client.post("/consent/whitelist/remove", json={"project_id": _NODE_ID_A})
    assert rm.status_code == 200
    assert rm.json()["allowed_project_ids"] == [_NODE_ID_B]

    # Removing again is a no-op — the worker treats add/remove as
    # idempotent so the dialog never has to worry about race
    # conditions with concurrent saves.
    again = consent_client.post("/consent/whitelist/remove", json={"project_id": _NODE_ID_A})
    assert again.status_code == 200
    assert again.json()["allowed_project_ids"] == [_NODE_ID_B]


def test_atomic_write_leaves_no_tmp_behind(
    consent_client: TestClient, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # Re-resolve the path under the monkeypatched SBFB_HOME so we
    # can inspect what's left in the directory after a write.
    monkeypatch.setenv(SBFB_HOME_ENV, str(tmp_path / "sbfb"))
    payload = {
        "level": 2,
        "caps": {"max_watts": 100, "max_vram_mb": 1024, "max_hours_day": 1.0},
        "allowed_project_ids": [],
        "own_node_id": "self",
    }
    response = consent_client.post("/consent/set", json=payload)
    assert response.status_code == 200

    target = consent_path()
    assert target.exists()
    siblings = sorted(p.name for p in target.parent.iterdir())
    # The atomic write pattern should rename `consent.json.tmp`
    # away — leaving a tmp behind would mean the rename failed
    # and the file is corrupt.
    assert siblings == ["consent.json"]

    # Sanity: contents parse as the persisted level.
    body = json.loads(target.read_text(encoding="utf-8"))
    assert body["level"] == 2
