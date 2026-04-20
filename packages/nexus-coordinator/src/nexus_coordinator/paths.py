# SPDX-License-Identifier: AGPL-3.0-or-later
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


def app_db_path(project_name: str, app_name: str) -> Path:
    """Path to the per-app SQLite file for ``app_name`` in ``project_name``.

    Sprint 8 Phase B (D3 impl): the coordinator loader wires an
    :class:`nexus_sdk.AppDatabaseClient` at this location to each
    app's :attr:`nexus_sdk.AppContext.db` before calling ``on_start``.
    An app that wants to read from an existing database — e.g.
    ``nexus-app-gov`` pointing at the legacy ``nexus/gov/govdata.db``
    — swaps ``ctx.db`` in its ``on_start`` hook.

    Does not create the parent directory; the loader creates it
    lazily so a read-only inspection of the path (tests, docs
    tooling) cannot mutate the filesystem.
    """
    return project_dir(project_name) / "apps" / app_name / "app.sqlite"


def app_storage_path(project_name: str, app_name: str) -> Path:
    """Path to the per-app JSON storage file for ``app_name`` in ``project_name``.

    Sprint 9 Phase B (D1 impl): the coordinator loader instantiates
    a :class:`nexus_sdk.AppStorage` at this location and assigns it
    to :attr:`nexus_sdk.AppContext.storage` BEFORE calling
    :meth:`nexus_sdk.NexusApp.on_start`. The file is created lazily
    by ``AppStorage`` on the first flush — this helper does not
    touch the filesystem so a read-only inspection (tests, docs
    tooling) cannot leave artefacts behind.

    Lives next to ``app.sqlite`` under ``apps/<app>/`` so the per-app
    state directory holds every persistence surface in one place.
    Honours the ``NEXUS_GRID_ROOT`` env override transparently
    via :func:`project_dir` (pattern P6).
    """
    return project_dir(project_name) / "apps" / app_name / "storage.json"


def app_uploads_path(project_name: str, app_name: str) -> Path:
    """Path to the per-app CAS uploads directory.

    Sprint 9 Phase E (D3 impl): the coordinator loader instantiates
    an :class:`nexus_sdk.AppFileStore` at this location and assigns
    it to :attr:`nexus_sdk.AppContext.files` BEFORE calling
    :meth:`nexus_sdk.NexusApp.on_start`. The directory is created
    lazily by ``AppFileStore`` on the first store — this helper does
    not touch the filesystem.
    """
    return project_dir(project_name) / "apps" / app_name / "uploads"


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


def canary_registry_path() -> Path:
    """Path to the federated warrant canary registry persistence file.

    Sprint 20 Phase E.3: the Python coordinator maintains an
    aggregator of every signed warrant canary it has observed
    (locally signed via the CLI, or imported from a federated
    peer). The registry persists to ``<root>/canary-registry.json``
    so freshness state survives coordinator restarts. The file is
    process-global (single user can run multiple coordinator
    project instances; the registry is shared because all of
    them observe the same gossip topic).
    """
    return nexus_grid_root() / "canary-registry.json"


def contributor_registry_path() -> Path:
    """Path to the contributor attestation registry SQLite file.

    Sprint 22 Phase C (Couche 2): the coordinator records a signed
    ContributorAttestation per ``(project_id, contributor_node_id)``
    pair at verified-deploy time. Queries by the daemon's curator-
    list governance-strong gate (``/api/contributor/verify/...``)
    hit this file. Kept process-global like the canary registry
    because the invariant is per-user, not per-project : a single
    user can publish multiple projects and each has one
    contributor graph.
    """
    return nexus_grid_root() / "contributor_registry.sqlite"
