# SPDX-License-Identifier: AGPL-3.0-or-later
"""Shared pytest fixtures for nexus-coordinator.

The integration tests spin up a real in-process iroh node via
:class:`nexus_coordinator.coordinator.Coordinator`. No mocks — if
the underlying iroh wrapper regresses, these tests catch it.

Isolation strategy: each test gets its own tmp path, and every
path helper in :mod:`nexus_coordinator.paths` is monkey-patched
so the coordinator writes to ``tmp_path/nexus-grid/...`` instead
of the real user data dir. That keeps tests hermetic and
parallelizable.
"""

from __future__ import annotations

from pathlib import Path
from typing import Iterator

import httpx
import pytest
from fastapi.testclient import TestClient
from nexus_coordinator import auth as _auth
from nexus_coordinator import paths as _paths

# Sprint 16 Phase A (D1): a known-valid 64-char hex token so the
# tests that boot the FastAPI factory can pass the loopback auth
# middleware without touching disk. Shape chosen to be visually
# distinct from a real cryptographic token.
_TEST_AUTH_TOKEN = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef"


@pytest.fixture(autouse=True)
def _inject_loopback_auth(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Autouse fixture: publish a known token + pre-attach the
    ``X-SBFB-Token`` and ``Host`` headers to every TestClient.

    We monkeypatch the ``__init__`` of :class:`TestClient` so any
    test that builds a client via the stock
    ``TestClient(create_app(...))`` call gets the bearer for free.
    Tests that deliberately probe the 401/403 paths override the
    headers on their client via ``client.headers.pop(...)``.
    """
    monkeypatch.setenv(_auth.AUTH_TOKEN_ENV, _TEST_AUTH_TOKEN)

    original_tc_init = TestClient.__init__

    def patched_tc_init(self, *args, **kwargs):  # type: ignore[no-untyped-def]
        # Default the TestClient base URL to a loopback host so
        # httpx emits `Host: 127.0.0.1` automatically — passes
        # LoopbackAuthMiddleware.is_loopback_host() without any
        # per-test header plumbing.
        kwargs.setdefault("base_url", "http://127.0.0.1")
        original_tc_init(self, *args, **kwargs)
        self.headers.setdefault(_auth.AUTH_HEADER, _TEST_AUTH_TOKEN)

    monkeypatch.setattr(TestClient, "__init__", patched_tc_init)

    # Same story for tests that build an ``httpx.AsyncClient``
    # directly with an ASGI transport (e.g. ``test_files.py``).
    # The monkeypatch rewrites the default ``base_url`` to a
    # loopback host and injects the bearer header, so existing
    # tests keep working without modification.
    original_ac_init = httpx.AsyncClient.__init__

    def patched_ac_init(self, *args, **kwargs):  # type: ignore[no-untyped-def]
        if kwargs.get("base_url") in (None, "http://testserver"):
            kwargs["base_url"] = "http://127.0.0.1"
        existing_headers = kwargs.get("headers") or {}
        merged = dict(existing_headers)
        merged.setdefault(_auth.AUTH_HEADER, _TEST_AUTH_TOKEN)
        kwargs["headers"] = merged
        original_ac_init(self, *args, **kwargs)

    monkeypatch.setattr(httpx.AsyncClient, "__init__", patched_ac_init)


@pytest.fixture
def nexus_grid_tmp(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Iterator[Path]:
    """Redirect every ``nexus_coordinator.paths.*`` function at
    ``tmp_path/nexus-grid/``.

    Yields the redirected root so tests can assert on files inside
    it (``~/.nexus-grid/projects/<name>/coord.key`` lives at
    ``tmp_path/nexus-grid/projects/<name>/coord.key``).
    """
    root = tmp_path / "nexus-grid"
    root.mkdir(parents=True, exist_ok=True)

    def _nexus_grid_root() -> Path:
        return root

    def _projects_root() -> Path:
        return root / "projects"

    def _project_dir(project_name: str) -> Path:
        return root / "projects" / project_name

    def _coord_key_path(project_name: str) -> Path:
        return _project_dir(project_name) / "coord.key"

    def _coord_config_path(project_name: str) -> Path:
        return _project_dir(project_name) / "coordinator.toml"

    def _iroh_data_path(project_name: str) -> Path:
        return _project_dir(project_name) / "iroh-data"

    def _running_state_path(project_name: str) -> Path:
        return _project_dir(project_name) / "running.json"

    def _worker_state_path() -> Path:
        return root / "worker" / "state.json"

    def _shell_daemon_registry_path() -> Path:
        return root / "shell-daemon" / "running.json"

    monkeypatch.setattr(_paths, "nexus_grid_root", _nexus_grid_root)
    monkeypatch.setattr(_paths, "projects_root", _projects_root)
    monkeypatch.setattr(_paths, "project_dir", _project_dir)
    monkeypatch.setattr(_paths, "coord_key_path", _coord_key_path)
    monkeypatch.setattr(_paths, "coord_config_path", _coord_config_path)
    monkeypatch.setattr(_paths, "iroh_data_path", _iroh_data_path)
    monkeypatch.setattr(_paths, "running_state_path", _running_state_path)
    monkeypatch.setattr(_paths, "worker_state_path", _worker_state_path)
    monkeypatch.setattr(_paths, "shell_daemon_registry_path", _shell_daemon_registry_path)

    yield root
