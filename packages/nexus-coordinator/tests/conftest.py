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

import pytest
from nexus_coordinator import paths as _paths


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
