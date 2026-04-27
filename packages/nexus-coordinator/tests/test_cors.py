# SPDX-License-Identifier: AGPL-3.0-or-later
"""CORS middleware tests (Sprint 33 Phase A).

Verify that create_app() respects the cors_origins parameter:
loopback always allowed, external origins accepted only when
explicitly configured.
"""

from __future__ import annotations

from pathlib import Path

import pytest
from fastapi.testclient import TestClient
from nexus_coordinator.api.app import create_app
from nexus_coordinator.coordinator import Coordinator


@pytest.mark.asyncio
async def test_cors_default_localhost_only(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="cors-default")
    await coord.start()
    try:
        app = create_app(coord)
        client = TestClient(app)
        resp = client.options(
            "/health",
            headers={
                "origin": "http://192.168.1.10:8080",
                "access-control-request-method": "GET",
            },
        )
        acao = resp.headers.get("access-control-allow-origin")
        assert acao is None or "192.168.1.10" not in acao
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_cors_custom_origin_accepted(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="cors-custom")
    await coord.start()
    try:
        app = create_app(coord, cors_origins=["http://192.168.1.10:8080"])
        client = TestClient(app)
        resp = client.options(
            "/health",
            headers={
                "origin": "http://192.168.1.10:8080",
                "access-control-request-method": "GET",
            },
        )
        acao = resp.headers.get("access-control-allow-origin")
        assert acao == "http://192.168.1.10:8080"
    finally:
        await coord.stop()


@pytest.mark.asyncio
async def test_cors_custom_preserves_localhost(nexus_grid_tmp: Path) -> None:
    coord = Coordinator(project_name="cors-preserve")
    await coord.start()
    try:
        app = create_app(coord, cors_origins=["http://192.168.1.10:8080"])
        client = TestClient(app)
        resp = client.options(
            "/health",
            headers={
                "origin": "http://127.0.0.1:5173",
                "access-control-request-method": "GET",
            },
        )
        acao = resp.headers.get("access-control-allow-origin")
        assert acao == "http://127.0.0.1:5173"
    finally:
        await coord.stop()
