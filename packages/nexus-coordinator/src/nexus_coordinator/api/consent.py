# SPDX-License-Identifier: AGPL-3.0-or-later
"""``/consent/*`` endpoints — GPU sharing preferences.

Sprint 16 Phase C. Persists the level / caps / whitelist the user
picks in :file:`web/src/components/GpuConsentDialog.tsx` to
:file:`{SBFB_HOME}/consent.json`. The Rust worker
(:mod:`nexus_worker_core::consent`) watches that same file via the
``notify`` crate and reloads its in-memory state without a
restart, so a "Save" click in the dialog applies on the next
claim tick.

Endpoints
---------
* ``GET /consent/get`` — current preferences (defaults if absent)
* ``POST /consent/set`` — replace the full payload
* ``POST /consent/whitelist/add`` — append a project_id to L3
* ``POST /consent/whitelist/remove`` — remove a project_id

The atomic-write pattern (sibling ``.tmp`` then ``os.replace``)
mirrors :mod:`nexus_coordinator.registry` so a crash mid-write
never leaves the file half-truncated.
"""

from __future__ import annotations

import os
import re
from pathlib import Path
from typing import Literal

import structlog
from fastapi import APIRouter, HTTPException
from pydantic import BaseModel, Field, field_validator

_log = structlog.get_logger(__name__)

router = APIRouter(prefix="/consent", tags=["consent"])

# -----------------------------------------------------------------
# Path helpers (mirror nexus_worker_core::consent::sbfb_home)
# -----------------------------------------------------------------

SBFB_HOME_ENV = "SBFB_HOME"


def sbfb_home() -> Path:
    """Resolve ``~/.sbfb/`` for the current user.

    Honours the ``SBFB_HOME`` env var so pytest can redirect the
    file at a hermetic ``tmp_path``. Same env var as the Rust
    worker's resolver so a single override flips both sides.
    """
    override = os.environ.get(SBFB_HOME_ENV, "").strip()
    if override:
        return Path(override)
    home = os.environ.get("HOME") or os.environ.get("USERPROFILE")
    if not home:
        # Fall back to a sub-dir of CWD rather than crashing the
        # request — the worker would log "consent disabled" on
        # the same machine, so we degrade gracefully here too.
        return Path.cwd() / ".sbfb"
    return Path(home) / ".sbfb"


def consent_path() -> Path:
    return sbfb_home() / "consent.json"


# -----------------------------------------------------------------
# Pydantic schema (wire-compatible with the Rust ConsentConfig)
# -----------------------------------------------------------------

# 64-char hex ed25519 public key — the same shape the worker
# emits in its ``node_id`` field. Used to validate L3 whitelist
# entries before we persist them.
_NODE_ID_RE = re.compile(r"^[0-9a-fA-F]{64}$")


class Caps(BaseModel):
    """Hard caps enforced worker-side. ``None`` means "no cap"."""

    max_watts: int | None = Field(default=400, ge=1, le=2000)
    max_vram_mb: int | None = Field(default=16 * 1024, ge=1)
    max_hours_day: float | None = Field(default=12.0, ge=0.0, le=24.0)


_LEVEL_THREAT_NOTES: dict[int, str] = {
    1: "Aucune exposition tierce. Seules vos propres apps s'exécutent.",
    2: "Apps open source vérifiées (SLSA L1). Exposition Sybil si contributeur malveillant.",
    3: "Apps sélectionnées manuellement. Vous êtes responsable de la vérification.",
    4: "Toute app publique du réseau. Risque maximum de consommation abusive.",
}

_LEVEL_RESIDUAL_THREATS: dict[int, list[str]] = {
    1: [],
    2: ["R2-supply-chain", "R5-kudos-linkability"],
    3: ["R2-supply-chain", "R3-rate-limit-absent", "R5-kudos-linkability"],
    4: ["R2-supply-chain", "R3-rate-limit-absent", "R4-consent-race", "R5-kudos-linkability"],
}


class ConsentConfig(BaseModel):
    """Wire format: byte-identical to the Rust ConsentConfig.

    ``level_threat_note`` and ``residual_threats_acknowledged`` are
    computed server-side per level (cf. THREAT_MODEL §9.1).  The
    Rust worker ignores these fields (no ``deny_unknown_fields``).
    """

    level: Literal[1, 2, 3, 4] = 1
    caps: Caps = Field(default_factory=Caps)
    allowed_project_ids: list[str] = Field(default_factory=list)
    own_node_id: str = ""
    level_threat_note: str = ""
    residual_threats_acknowledged: list[str] = Field(default_factory=list)

    @field_validator("allowed_project_ids")
    @classmethod
    def _validate_node_ids(cls, v: list[str]) -> list[str]:
        for item in v:
            if not _NODE_ID_RE.match(item):
                raise ValueError(f"invalid node_id format: {item!r}")
        return v


def _threat_fields_for_level(level: int) -> dict[str, str | list[str]]:
    """Return derived threat fields for a consent level (pure function)."""
    return {
        "level_threat_note": _LEVEL_THREAT_NOTES.get(level, ""),
        "residual_threats_acknowledged": list(_LEVEL_RESIDUAL_THREATS.get(level, [])),
    }


class WhitelistEntry(BaseModel):
    """Body of ``POST /consent/whitelist/add`` and ``/remove``.

    Either ``project_id`` (a node_id hex) or ``repo_url`` (a Git
    URL the coordinator resolves to a node_id via the local
    browse aggregator). At least one must be set.
    """

    project_id: str | None = None
    repo_url: str | None = None

    @field_validator("project_id")
    @classmethod
    def _validate_id(cls, v: str | None) -> str | None:
        if v is not None and not _NODE_ID_RE.match(v):
            raise ValueError(f"invalid node_id format: {v!r}")
        return v


# -----------------------------------------------------------------
# Storage
# -----------------------------------------------------------------


def _load_atomic() -> ConsentConfig:
    path = consent_path()
    try:
        body = path.read_text(encoding="utf-8")
    except FileNotFoundError:
        return ConsentConfig()
    try:
        return ConsentConfig.model_validate_json(body)
    except ValueError as exc:
        # A corrupt file means the user's prefs are unreadable —
        # the worker would also fall back to defaults, so do the
        # same here rather than crashing the API.
        _log.warning("consent.json invalid; returning defaults", error=str(exc))
        return ConsentConfig()


def _save_atomic(cfg: ConsentConfig) -> None:
    path = consent_path()
    path.parent.mkdir(parents=True, exist_ok=True)
    body = cfg.model_dump_json(indent=2)
    tmp = path.with_suffix(".json.tmp")
    tmp.write_text(body, encoding="utf-8")
    os.replace(tmp, path)


# -----------------------------------------------------------------
# Endpoints
# -----------------------------------------------------------------


@router.get("/get")
async def get_consent() -> ConsentConfig:
    """Return the current preferences, or built-in defaults if
    the user has never opened the dialog."""
    cfg = _load_atomic()
    return cfg.model_copy(update=_threat_fields_for_level(cfg.level))


@router.post("/set")
async def set_consent(cfg: ConsentConfig) -> ConsentConfig:
    """Replace the full consent payload. Pydantic validates the
    level / caps / whitelist shape before we touch disk."""
    _save_atomic(cfg)
    _log.info(
        "consent.json updated",
        level=cfg.level,
        whitelist_size=len(cfg.allowed_project_ids),
    )
    return cfg.model_copy(update=_threat_fields_for_level(cfg.level))


@router.post("/whitelist/add")
async def whitelist_add(entry: WhitelistEntry) -> ConsentConfig:
    """Append a project to the L3 whitelist.

    Idempotent — adding an already-present project is a no-op
    rather than an error so the "Contribuer mon GPU" button on
    Browse can be clicked twice without surprising the user.

    Resolution: when ``project_id`` is set we use it directly;
    when ``repo_url`` is set we look it up in the local browse
    aggregator (Sprint 13). Phase C ships the shape; the
    repo_url -> node_id resolver is a stub that returns 422
    until the aggregator wiring lands in a follow-up.
    """
    if entry.project_id is None and entry.repo_url is None:
        raise HTTPException(status_code=422, detail="project_id or repo_url required")

    pid = entry.project_id
    if pid is None:
        # repo_url path: not wired yet. Surface a clear 422 so
        # the dialog can prompt the user to paste the node_id
        # directly until the aggregator lookup ships.
        raise HTTPException(
            status_code=422,
            detail="repo_url -> node_id resolution not yet wired; paste the node_id hex instead",
        )

    cfg = _load_atomic()
    if pid not in cfg.allowed_project_ids:
        cfg.allowed_project_ids.append(pid)
        _save_atomic(cfg)
        _log.info("consent whitelist add", project_id=pid)
    return cfg


@router.post("/whitelist/remove")
async def whitelist_remove(entry: WhitelistEntry) -> ConsentConfig:
    """Remove a project from the L3 whitelist. Idempotent."""
    if entry.project_id is None:
        raise HTTPException(status_code=422, detail="project_id required")

    cfg = _load_atomic()
    if entry.project_id in cfg.allowed_project_ids:
        cfg.allowed_project_ids.remove(entry.project_id)
        _save_atomic(cfg)
        _log.info("consent whitelist remove", project_id=entry.project_id)
    return cfg
