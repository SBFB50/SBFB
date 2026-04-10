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

from pathlib import Path

from platformdirs import user_data_dir

_APP_NAME = "nexus-grid"
_APP_AUTHOR = False  # disable the Windows "company\\product" nesting


def nexus_grid_root() -> Path:
    """Return the nexus-grid root directory for the current user."""
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
