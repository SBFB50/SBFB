"""Filesystem layout helpers.

The coordinator keeps every project isolated under its own
directory beneath the user data dir. Mirrors the Sprint 3 worker
layout (``~/.nexus-grid/worker.toml``) so both binaries share the
same root.

Layout::

    ~/.nexus-grid/
    ├── worker.toml              # owned by nexus-worker (Sprint 3)
    └── projects/
        └── <project-name>/
            ├── coord.key        # Ed25519 secret, perm 600
            ├── coordinator.toml # persistent config
            └── iroh-data/       # iroh node storage

On Windows, ``platformdirs.user_data_dir("nexus-grid")`` resolves
to ``%APPDATA%\\nexus-grid`` which is equivalent for our
non-roaming read/write needs.
"""

from __future__ import annotations

import os
from pathlib import Path

from platformdirs import user_data_dir

_APP_NAME = "nexus-grid"
_APP_AUTHOR = False  # disable the Windows "company\\product" nesting

#: Environment variable honoured by :func:`nexus_grid_root` so
#: integration tests (Python pytest fixtures, Playwright
#: globalSetup, e2e scripts) can point the whole nexus-grid tree
#: at a throw-away directory without touching the user's real
#: data dir. Mirrors the Sprint 3 worker ``NEXUS_WORKER__*`` env
#: override philosophy.
_ROOT_OVERRIDE_ENV = "NEXUS_GRID_ROOT"


def nexus_grid_root() -> Path:
    """Return the nexus-grid root directory for the current user.

    If ``NEXUS_GRID_ROOT`` is set in the environment, its value is
    used verbatim — this is the single override point for tests
    that need a hermetic tree. Otherwise falls back to the
    platform's user data directory via ``platformdirs``.
    """
    override = os.environ.get(_ROOT_OVERRIDE_ENV)
    if override:
        return Path(override)
    return Path(user_data_dir(_APP_NAME, _APP_AUTHOR))


def projects_root() -> Path:
    """Return the parent directory for all coordinator projects."""
    return nexus_grid_root() / "projects"


def project_dir(project_name: str) -> Path:
    """Return the directory for a specific project.

    Does not create the directory — callers that intend to write to
    it should call :meth:`pathlib.Path.mkdir` with
    ``parents=True, exist_ok=True``.
    """
    return projects_root() / project_name


def coord_key_path(project_name: str) -> Path:
    """Path to the Ed25519 secret for a given project."""
    return project_dir(project_name) / "coord.key"


def coord_config_path(project_name: str) -> Path:
    """Path to the persistent ``coordinator.toml`` for a project."""
    return project_dir(project_name) / "coordinator.toml"


def iroh_data_path(project_name: str) -> Path:
    """Path to the iroh node storage directory for a project."""
    return project_dir(project_name) / "iroh-data"


def running_state_path(project_name: str) -> Path:
    """Path to the shell-facing ``running.json`` registry entry.

    Sprint 5 Phase A decision D1: every coordinator that is
    currently ``start``'d writes a tiny JSON file at this location
    with its live node_id, port, pid, etc. The shell reads the
    collection via the ``GET /shell/discover`` endpoint on any
    connected coordinator, which globs
    ``<projects_root>/*/running.json``. See
    :mod:`nexus_coordinator.registry` for the schema.
    """
    return project_dir(project_name) / "running.json"


def worker_state_path() -> Path:
    """Path to the worker's shell-facing ``state.json`` snapshot.

    Mirror of the Rust helper ``nexus_worker_core::paths::worker_state_file``
    — both sides must resolve to the exact same file. The Rust
    worker flushes this every ``state_flush_secs`` seconds, the
    coordinator proxies it to the shell via
    ``GET /worker-state``.
    """
    return nexus_grid_root() / "worker" / "state.json"


def shell_daemon_registry_path() -> Path:
    """Path to the ``nexus-shell-daemon`` singleton ``running.json``.

    Sprint 7 Phase E: the Rust shell daemon writes its
    ``running.json`` atomically on boot at
    ``<root>/shell-daemon/running.json`` (see
    ``crates/nexus-shell-daemon-core/src/paths.rs::running_json_path``
    and the Phase A + Phase C commits). The coordinator's new
    ``/daemon/*`` proxy router reads that file on every request
    to discover the live daemon's loopback port, then forwards
    the call via ``httpx.AsyncClient``.

    The schema carried by the file differs from the coordinator's
    own ``running.json`` — the daemon is global per user (no
    ``project_name``, no ``visibility``), and the file includes a
    ``daemon_version`` field. See
    ``crates/nexus-shell-daemon-core/src/registry.rs::RunningState``
    for the exact shape.
    """
    return nexus_grid_root() / "shell-daemon" / "running.json"
