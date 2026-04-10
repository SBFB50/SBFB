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
