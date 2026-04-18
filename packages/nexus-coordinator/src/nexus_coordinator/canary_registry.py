# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated warrant canary registry.

Aggregates every signed warrant canary the coordinator has
observed (typically from peer-side gossip subscribe + local CLI
publish, but the source is opaque to the registry) and exposes
freshness state per maintainer pubkey.

## Shape

The registry is keyed by maintainer **pubkey** (lowercase hex of
the 32-byte Ed25519 verifying key embedded in
:class:`CanarySigned`) and stores the most recent canary date /
headline / next-update / signature seen for that pubkey, plus the
most recent duress ack date observed (Phase E.4 channel — same
maintainer, separate cadence).

## Wire-format invariants honoured

- The registry never re-signs anything — it just records what it
  observed. Signature verification is the responsibility of the
  caller (`verify_canary` / `verify_duress_ack` Rust path,
  surfaced over HTTP by the daemon side later).
- The pubkey strings are stored verbatim from the signed payload,
  64 lowercase hex chars. The registry does no normalisation.
- The wire format embedded in the persisted JSON is the same
  ``{ signed: CanarySigned, signature_hex }`` shape the daemon
  emits — verifiers can re-verify offline by re-deriving the
  canonical bytes (cf. ``DOMAIN_WARRANT_CANARY_V1``).

## Threat model coverage

Phase E.3 is the **observability** layer. It does NOT make any
trust decision on its own — a stale registry is just data, the
operator interprets it. By exposing the freshness through
``GET /api/canary/network-health``, the shell can render a fleet
view ("3 maintainers all fresh / 1 maintainer 38 days stale, last
ack 4 days ago") that gives a human a single pane to spot a
coercion pattern earlier than re-deriving it from raw gossip.

## Persistence

The registry persists to a single JSON file at
:func:`canary_registry_path` on every observation. The format is
forward-compatible-by-construction (new fields are added at the
top level with sensible defaults, never inside the per-pubkey
objects which mirror the wire format byte-for-byte).
"""

from __future__ import annotations

import json
from collections.abc import Iterable
from datetime import date, datetime, timezone
from pathlib import Path
from typing import Any

import structlog
from pydantic import BaseModel, ConfigDict, Field

_log = structlog.get_logger(__name__)

# Mirror of the Rust :rust:`CANARY_VALIDITY_DAYS` constant.
# Keep these two in sync — the freshness alarm thresholds derive
# from the canary's own validity window.
CANARY_VALIDITY_DAYS: int = 45

# Freshness alarm thresholds. A canary is "fresh" if its date is
# within the validity window, "warn" if it has overshot the window
# but stays within the soft tail, and "stale" past the soft tail.
# These match the Sprint 18 Phase E2 design : 30-day cadence,
# 15-day grace, 30-day soft tail before fully alarming.
WARN_THRESHOLD_DAYS: int = 30  # cadence ; canary is expected every 30 days
ALARM_THRESHOLD_DAYS: int = CANARY_VALIDITY_DAYS  # 45 ; past this = dead-man-switch fired

# Duress acks have a much tighter cadence (daily-or-better).
# Past 2 days without an ack is the operator's first signal.
DURESS_ACK_WARN_DAYS: int = 2
DURESS_ACK_ALARM_DAYS: int = 7


class CanaryObservation(BaseModel):
    """The signed body + signature the registry persists per pubkey.

    Mirrors the Rust :rust:`Canary` wire shape exactly. Fields are
    typed permissively (``str`` for the date / pubkey / sig hex)
    because the Python side does not re-validate the cryptography
    — it stores what it observed and re-emits it verbatim.
    """

    model_config = ConfigDict(extra="forbid")

    version: int = Field(..., description="CANARY_VERSION constant ; pre-launch always 1.")
    date: str = Field(..., description="UTC date YYYY-MM-DD of the canary.")
    headline: str
    next_update: str
    pubkey_hex: str = Field(..., min_length=64, max_length=64)
    signature_hex: str = Field(..., min_length=128, max_length=128)


class DuressAckObservation(BaseModel):
    """The signed body + signature of an observed duress ack.

    Mirrors the Rust :rust:`DuressAck` wire shape.
    """

    model_config = ConfigDict(extra="forbid")

    version: int = Field(..., description="DURESS_ACK_VERSION constant ; pre-launch always 1.")
    date: str = Field(..., description="UTC date YYYY-MM-DD of the ack.")
    message: str
    pubkey_hex: str = Field(..., min_length=64, max_length=64)
    signature_hex: str = Field(..., min_length=128, max_length=128)


class CanaryFreshness(BaseModel):
    """Freshness diagnostic for a single maintainer pubkey.

    The registry computes this on demand from the stored
    observation timestamps, not from the gossip event timestamps,
    so a verifier seeing the API response can reason directly about
    "the maintainer last claimed liberty on this date".
    """

    model_config = ConfigDict(extra="forbid")

    pubkey_hex: str
    canary_date: str | None = Field(default=None, description="Most recent canary date observed.")
    canary_age_days: int | None = Field(default=None, ge=0)
    canary_status: str = Field(
        default="missing",
        description="One of fresh / warn / stale / missing.",
    )
    duress_ack_date: str | None = Field(default=None, description="Most recent duress ack observed.")
    duress_ack_age_days: int | None = Field(default=None, ge=0)
    duress_ack_status: str = Field(
        default="missing",
        description="One of fresh / warn / stale / missing.",
    )


class NetworkHealth(BaseModel):
    """Top-level shape returned by ``GET /api/canary/network-health``.

    Designed for direct consumption by the React shell — the
    counts in ``summary`` give a one-glance fleet picture, the
    per-maintainer ``maintainers`` array drives a detail view.
    """

    model_config = ConfigDict(extra="forbid")

    summary: dict[str, int]
    maintainers: list[CanaryFreshness]
    observed_at: str = Field(..., description="UTC RFC 3339 timestamp of this snapshot.")


def _today_utc() -> date:
    return datetime.now(timezone.utc).date()


def _classify_canary_age(age_days: int) -> str:
    if age_days < WARN_THRESHOLD_DAYS:
        return "fresh"
    if age_days < ALARM_THRESHOLD_DAYS:
        return "warn"
    return "stale"


def _classify_duress_age(age_days: int) -> str:
    if age_days < DURESS_ACK_WARN_DAYS:
        return "fresh"
    if age_days < DURESS_ACK_ALARM_DAYS:
        return "warn"
    return "stale"


class CanaryRegistry:
    """In-memory + on-disk aggregator of observed warrant canaries.

    Thread safety : single-threaded by construction. The
    coordinator process is asyncio-driven and the registry is
    only mutated from FastAPI request handlers and the
    coordinator's own boot path, never from a thread pool.
    """

    def __init__(self, persist_path: Path) -> None:
        self._persist_path = persist_path
        self._canaries: dict[str, CanaryObservation] = {}
        self._duress_acks: dict[str, DuressAckObservation] = {}
        self._load_if_exists()

    # ------------------------------------------------------------------
    # Persistence
    # ------------------------------------------------------------------

    def _load_if_exists(self) -> None:
        if not self._persist_path.exists():
            return
        try:
            raw = json.loads(self._persist_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            _log.warning(
                "canary_registry.load_failed",
                path=str(self._persist_path),
                error=str(exc),
            )
            return
        for item in raw.get("canaries", []):
            try:
                obs = CanaryObservation.model_validate(item)
            except Exception as exc:  # pydantic ValidationError or any unexpected shape
                _log.warning("canary_registry.bad_canary_entry", error=str(exc))
                continue
            self._canaries[obs.pubkey_hex] = obs
        for item in raw.get("duress_acks", []):
            try:
                obs = DuressAckObservation.model_validate(item)
            except Exception as exc:
                _log.warning("canary_registry.bad_duress_ack_entry", error=str(exc))
                continue
            self._duress_acks[obs.pubkey_hex] = obs

    def _persist(self) -> None:
        self._persist_path.parent.mkdir(parents=True, exist_ok=True)
        payload = {
            "canaries": [obs.model_dump() for obs in self._canaries.values()],
            "duress_acks": [obs.model_dump() for obs in self._duress_acks.values()],
        }
        # Atomic write : write to a temp file then rename so a
        # crashed coordinator never leaves a half-written JSON
        # blob behind.
        tmp = self._persist_path.with_suffix(self._persist_path.suffix + ".tmp")
        tmp.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
        tmp.replace(self._persist_path)

    # ------------------------------------------------------------------
    # Observation
    # ------------------------------------------------------------------

    def observe_canary(self, observation: CanaryObservation) -> None:
        """Record a freshly observed warrant canary.

        Only the most recent canary per pubkey is kept (by ``date``
        string lexicographic order, which is a valid total order
        for ISO-8601 ``YYYY-MM-DD``). An older observation for the
        same pubkey is silently ignored.
        """
        prev = self._canaries.get(observation.pubkey_hex)
        if prev is not None and prev.date >= observation.date:
            return
        self._canaries[observation.pubkey_hex] = observation
        self._persist()

    def observe_duress_ack(self, observation: DuressAckObservation) -> None:
        """Record a freshly observed duress ack."""
        prev = self._duress_acks.get(observation.pubkey_hex)
        if prev is not None and prev.date >= observation.date:
            return
        self._duress_acks[observation.pubkey_hex] = observation
        self._persist()

    # ------------------------------------------------------------------
    # Inspection
    # ------------------------------------------------------------------

    def known_pubkeys(self) -> Iterable[str]:
        """Pubkeys with at least one observation (canary or ack)."""
        return set(self._canaries) | set(self._duress_acks)

    def freshness(self, pubkey_hex: str, today: date | None = None) -> CanaryFreshness:
        """Compute the freshness diagnostic for a single pubkey."""
        today = today or _today_utc()
        result = CanaryFreshness(pubkey_hex=pubkey_hex)

        canary = self._canaries.get(pubkey_hex)
        if canary is not None:
            result.canary_date = canary.date
            try:
                age = (today - date.fromisoformat(canary.date)).days
            except ValueError:
                age = None
            if age is not None and age >= 0:
                result.canary_age_days = age
                result.canary_status = _classify_canary_age(age)

        ack = self._duress_acks.get(pubkey_hex)
        if ack is not None:
            result.duress_ack_date = ack.date
            try:
                age = (today - date.fromisoformat(ack.date)).days
            except ValueError:
                age = None
            if age is not None and age >= 0:
                result.duress_ack_age_days = age
                result.duress_ack_status = _classify_duress_age(age)

        return result

    def network_health(self, today: date | None = None) -> NetworkHealth:
        """Snapshot of every known maintainer's freshness."""
        today = today or _today_utc()
        per_pubkey = [self.freshness(pk, today=today) for pk in sorted(self.known_pubkeys())]

        summary: dict[str, int] = {
            "maintainers_total": len(per_pubkey),
            "canary_fresh": 0,
            "canary_warn": 0,
            "canary_stale": 0,
            "canary_missing": 0,
            "duress_ack_fresh": 0,
            "duress_ack_warn": 0,
            "duress_ack_stale": 0,
            "duress_ack_missing": 0,
        }
        for entry in per_pubkey:
            summary[f"canary_{entry.canary_status}"] += 1
            summary[f"duress_ack_{entry.duress_ack_status}"] += 1

        return NetworkHealth(
            summary=summary,
            maintainers=per_pubkey,
            observed_at=datetime.now(timezone.utc).isoformat(timespec="seconds"),
        )

    # ------------------------------------------------------------------
    # Test affordance
    # ------------------------------------------------------------------

    def _canaries_snapshot(self) -> dict[str, CanaryObservation]:
        """Read-only snapshot for tests."""
        return dict(self._canaries)

    def _duress_acks_snapshot(self) -> dict[str, DuressAckObservation]:
        """Read-only snapshot for tests."""
        return dict(self._duress_acks)


def coerce_canary_payload(payload: dict[str, Any]) -> CanaryObservation:
    """Build a :class:`CanaryObservation` from the daemon's wire payload.

    The Rust :rust:`Canary` struct serializes to a flat object
    via ``#[serde(flatten)]`` on the inner ``signed`` field, so
    the on-the-wire JSON is::

        {
            "v": 1, "date": "...", "headline": "...",
            "next_update": "...", "pubkey_hex": "...",
            "signature_hex": "..."
        }

    This helper handles the ``v`` -> ``version`` rename and tolerates
    payloads that already use the explicit ``version`` key (Python
    test fixtures, manual ``POST`` bodies).
    """
    body = dict(payload)
    if "v" in body and "version" not in body:
        body["version"] = body.pop("v")
    return CanaryObservation.model_validate(body)


def coerce_duress_ack_payload(payload: dict[str, Any]) -> DuressAckObservation:
    """Same as :func:`coerce_canary_payload` for duress acks."""
    body = dict(payload)
    if "v" in body and "version" not in body:
        body["version"] = body.pop("v")
    return DuressAckObservation.model_validate(body)
