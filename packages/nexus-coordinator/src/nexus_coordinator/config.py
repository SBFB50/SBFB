# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coordinator configuration.

:class:`CoordinatorConfig` is a Pydantic v2 settings model that
layers three sources in this precedence order (higher wins):

1. Environment variables with prefix ``NEXUS_COORD__`` (double
   underscore is the nested delimiter — e.g.
   ``NEXUS_COORD__NETWORK__API_PORT=9001``).
2. The project's ``coordinator.toml`` file, read via the stdlib
   ``tomllib``.
3. Hard-coded defaults declared on each model class.

Persistence: :meth:`CoordinatorConfig.save` writes the current
config back to ``coordinator.toml`` via ``tomli_w``. The file is
created on ``nexus-coordinator init`` and re-read on every
``start``.

Section split mirrors the Sprint 3 ``worker.toml`` structure so
the two TOML files look familiar side by side.
"""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

import tomli_w
from pydantic import BaseModel, Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Identity(BaseModel):
    """Identity section — project name, human description, author id.

    ``author_id`` is the iroh-docs AuthorId the coordinator uses to
    write task entries. Created on first boot via
    :meth:`nexus_core.Node.docs_author_create` and persisted here.
    ``doc_id`` is the namespace id of the project doc, also
    persisted so subsequent boots reopen via
    :meth:`nexus_core.Node.docs_open`.
    """

    name: str = Field(default="", description="Short project name (used as the directory name).")
    description: str = Field(default="", description="Human-readable description.")
    author_id: str | None = Field(
        default=None,
        description="iroh-docs AuthorId hex, persisted after first boot.",
    )
    doc_id: str | None = Field(
        default=None,
        description="iroh-docs NamespaceId hex, persisted after first boot.",
    )
    repo_url: str | None = Field(
        default=None,
        description=(
            "Public Git repository URL of the last successful "
            "deploy-from-repo. Set by the coordinator on "
            "verified-deploy success (Sprint 14) and persisted so "
            "the dispatcher (Sprint 18 Phase D) can derive "
            "``Task.is_open_source`` across restarts. Absence "
            "means the project was never deployed from a public "
            "repo — tasks crafted for it carry ``is_open_source=false``."
        ),
    )


class Network(BaseModel):
    """Network section — how the coordinator exposes itself."""

    api_host: str = Field(
        default="127.0.0.1",
        description="Bind address for the FastAPI control API. Default is loopback-only.",
    )
    api_port: int = Field(
        default=8765,
        ge=1,
        le=65535,
        description="Bind port for the FastAPI control API.",
    )
    visibility: str = Field(
        default="private",
        description="Either 'public' (publish Node id via iroh-pkarr) or 'private' (invite-only).",
    )


class Policy(BaseModel):
    """Policy section — dispatcher + validator behavior toggles."""

    claim_timeout_secs: int = Field(
        default=300,
        ge=10,
        description="Seconds after which a claimed-but-unfinished task is marked timed-out and re-queued.",
    )
    max_pending_tasks: int = Field(
        default=10_000,
        ge=1,
        description="Cap on the number of pending tasks in the tasks doc before submit is throttled.",
    )
    result_poll_interval_ms: int = Field(
        default=500,
        ge=50,
        description="Validator loop backoff when the LiveEvent stream is idle.",
    )


class UploadQueue(BaseModel):
    """Delayed-upload queue section — Sprint 19 Phase D anti-correlation.

    Randomizes the time between a coordinator-local submit and the
    actual gossip emit so an external observer cannot trivially
    correlate a loopback POST with an upstream broadcast. The
    default parameters come from the Phase D design doc
    (exponential mean=90s clamped to 270s internally → 300s
    observable when accounting for flush granularity). Operators
    who need a different privacy/UX trade-off can tune each field
    via coordinator.toml; ``enabled = false`` is a dev escape hatch
    that routes submits directly to the dispatcher.
    """

    enabled: bool = Field(
        default=True,
        description=(
            "Route /tasks/submit through the delayed queue. Set to false in dev only — disables anti-correlation."
        ),
    )
    mean_jitter_s: float = Field(
        default=90.0,
        gt=0.0,
        description="Mean of the exponential delay distribution (seconds).",
    )
    max_jitter_s: float = Field(
        default=300.0,
        gt=0.0,
        description=(
            "Hard ceiling on any single draw (seconds). The "
            "effective internal clamp is ``max_jitter_s - "
            "flush_interval_s`` so the observable p99 stays ≤ "
            "max_jitter_s once scheduler granularity is factored in."
        ),
    )
    flush_interval_s: float = Field(
        default=30.0,
        gt=0.0,
        description="Scheduler wake-up interval (seconds).",
    )
    soft_cap: int = Field(
        default=10_000,
        ge=1,
        description="Queue size threshold above which schedule() logs a WARN.",
    )
    hard_cap: int = Field(
        default=100_000,
        ge=1,
        description="Queue size ceiling — schedule() raises QueueFullError above this.",
    )


class QuarantineQueue(BaseModel):
    """Quarantine queue section — Sprint 21 Phase D defense-in-depth.

    Holds borderline gossip messages that passed PoW + rate-limit
    but matched a soft heuristic, waiting for operator review via
    ``nexus-coordinator quarantine list/flush/drop``. The TTL
    auto-drops pending entries silently after ``ttl_seconds`` to
    bound disk growth.
    """

    ttl_seconds: int = Field(
        default=900,
        ge=1,
        description=(
            "Pending-entry TTL window (seconds). Default 15 min "
            "matches kickoff §D4 ligne 590. Audit entries (flushed "
            "or dropped) survive past TTL."
        ),
    )
    sweep_interval_s: float = Field(
        default=30.0,
        gt=0.0,
        description="Sweep loop interval between TTL DELETE batches (seconds).",
    )


class CoordinatorConfig(BaseSettings):
    """Top-level settings model.

    Environment overrides use the prefix ``NEXUS_COORD__`` with
    ``__`` as nested delimiter, matching the Sprint 3 worker's
    ``NEXUS_WORKER__`` convention.
    """

    model_config = SettingsConfigDict(
        env_prefix="NEXUS_COORD__",
        env_nested_delimiter="__",
        extra="ignore",
    )

    identity: Identity = Field(default_factory=Identity)
    network: Network = Field(default_factory=Network)
    policy: Policy = Field(default_factory=Policy)
    upload_queue: UploadQueue = Field(default_factory=UploadQueue)
    quarantine_queue: QuarantineQueue = Field(default_factory=QuarantineQueue)

    @classmethod
    def load(cls, path: Path | None) -> "CoordinatorConfig":
        """Load a config from a TOML file, applying env overrides on top.

        If ``path`` is ``None`` or the file does not exist, return
        a config built only from defaults + environment. Callers
        that expect a persisted file should check
        ``path.exists()`` themselves before calling.
        """
        toml_values: dict[str, Any] = {}
        if path is not None and path.exists():
            toml_values = tomllib.loads(path.read_text(encoding="utf-8"))
        # BaseSettings' priority: init kwargs > env > defaults.
        # We pass the TOML values as init kwargs, then env and
        # defaults fill in anything missing.
        return cls(**toml_values)

    def save(self, path: Path) -> None:
        """Write the config to a TOML file atomically-ish.

        The current implementation writes to a sibling ``.tmp``
        file then ``replace``s it, so a crash mid-write leaves the
        previous version intact. Sufficient for coordinator state
        (never rewritten at high frequency).

        ``exclude_none=True`` because ``tomli_w`` refuses to
        serialize ``None`` — at ``init`` time ``identity.author_id``
        and ``identity.doc_id`` are still None, and the model's
        default-factory populates them again on the next
        ``CoordinatorConfig.load`` via the field defaults. Sprint
        5 Phase B discovered this when the Playwright globalSetup
        invoked ``nexus-coordinator init`` for the first time in
        a hermetic env.
        """
        path.parent.mkdir(parents=True, exist_ok=True)
        dumped = self.model_dump(mode="python", exclude_none=True)
        tmp = path.with_suffix(path.suffix + ".tmp")
        tmp.write_bytes(tomli_w.dumps(dumped).encode("utf-8"))
        tmp.replace(path)
