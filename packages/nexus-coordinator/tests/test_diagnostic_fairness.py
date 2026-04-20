# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for GET /diagnostic/fairness endpoint."""

from __future__ import annotations

from pathlib import Path
from unittest.mock import MagicMock

from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app


def _make_coordinator_stub(tmp_path: Path) -> MagicMock:
    """Build a minimal Coordinator stub with no kudos ledger."""
    coord = MagicMock()
    coord.project_name = "test-proj"
    coord.kudos_ledger = None
    return coord


class TestFairnessEndpoint:
    def test_returns_zeroed_when_no_ledger(self, tmp_path: Path) -> None:
        coord = _make_coordinator_stub(tmp_path)
        app = create_app(coord)
        client = TestClient(app)
        resp = client.get("/diagnostic/fairness")
        assert resp.status_code == 200
        data = resp.json()
        assert data["gini"] == 0.0
        assert data["top_5_pct_share"] == 0.0
        assert data["churn_rate"] == 0.0
        assert data["worker_count"] == 0
