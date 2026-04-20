# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 23 Phase E — honeypot canary peers for eclipse detection.

Design-distinct from the Sprint 18 warrant canary (legal
transparency report signed manually by maintainer, cf. ``04c9621``
threat-model rejection). These are **dummy P2P peer identities**
generated programmatically by the coordinator to detect eclipse
attacks on the DHT neighborhood.

Three collaborating objects:

:class:`CanaryPeerFactory`
    Generates fresh Ed25519 keypairs representing dummy nodes.
    Each keypair is ephemeral — used only for a single rotation
    window (default 6 h) then discarded.

:class:`EclipseDetector`
    Tracks which real workers report seeing which canary peers in
    their neighborhood. If a single worker appears co-located with
    >80% of canary peers across 3 consecutive rotations, an
    :class:`EclipseAlert` is raised.

:class:`CanaryRotationScheduler`
    Manages the rotation cadence. Provides :meth:`should_rotate`
    (True when 6 h have elapsed since last rotation) and
    :meth:`rotate` (generates new canary set, resets detector
    counters for the new window).

Scope cuts (Sprint 23):
- Auto-quarantine on eclipse alert → post-Gate 3
- Canary peers publishing to pkarr → S24 (requires daemon integration)
- Persistent canary history → P2-E carry
"""

from __future__ import annotations

import time
from dataclasses import dataclass

import nacl.signing
import structlog

_log = structlog.get_logger(__name__)

DEFAULT_CANARY_COUNT = 5
DEFAULT_ROTATION_INTERVAL_S = 6 * 3600  # 6 hours
ECLIPSE_CO_LOCATION_THRESHOLD = 0.80
ECLIPSE_CONSECUTIVE_ROTATIONS = 3


@dataclass(frozen=True)
class CanaryPeer:
    """A single ephemeral dummy peer identity."""

    public_key_hex: str
    created_at: float


@dataclass
class EclipseAlert:
    """Raised when a worker shows eclipse co-location pattern."""

    worker_id: str
    co_location_pct: float
    consecutive_rotations: int
    detected_at: float


class CanaryPeerFactory:
    """Generates ephemeral Ed25519 keypairs for canary peers."""

    @staticmethod
    def generate(count: int = DEFAULT_CANARY_COUNT) -> list[CanaryPeer]:
        """Generate ``count`` fresh canary peers with new Ed25519 keys."""
        now = time.time()
        peers: list[CanaryPeer] = []
        for _ in range(count):
            sk = nacl.signing.SigningKey.generate()
            pub_hex = sk.verify_key.encode().hex()
            peers.append(CanaryPeer(public_key_hex=pub_hex, created_at=now))
        return peers


class EclipseDetector:
    """Track canary peer co-location reports per worker.

    Call :meth:`report_neighborhood` for each worker's neighborhood
    snapshot. Call :meth:`evaluate` after a rotation to check for
    eclipse patterns. Call :meth:`advance_rotation` when the canary
    set is rotated to shift the sliding window.
    """

    def __init__(
        self,
        threshold: float = ECLIPSE_CO_LOCATION_THRESHOLD,
        required_rotations: int = ECLIPSE_CONSECUTIVE_ROTATIONS,
    ) -> None:
        self._threshold = threshold
        self._required_rotations = required_rotations
        # worker_id -> number of consecutive rotations above threshold
        self._streak: dict[str, int] = {}
        # Current rotation: worker_id -> set of canary pubkeys seen
        self._current_sightings: dict[str, set[str]] = {}

    @property
    def streak(self) -> dict[str, int]:
        """Read-only view of the current streak counters."""
        return dict(self._streak)

    def report_neighborhood(
        self,
        worker_id: str,
        canary_pubkeys_in_neighborhood: set[str],
    ) -> None:
        """Record which canary peers a worker reports seeing."""
        existing = self._current_sightings.setdefault(worker_id, set())
        existing.update(canary_pubkeys_in_neighborhood)

    def evaluate(
        self,
        canary_set: list[CanaryPeer],
    ) -> list[EclipseAlert]:
        """Check all workers for eclipse pattern at end of rotation.

        Returns a list of alerts for workers exceeding the threshold
        for the required number of consecutive rotations.
        """
        if not canary_set:
            return []

        canary_keys = {p.public_key_hex for p in canary_set}
        total = len(canary_keys)
        alerts: list[EclipseAlert] = []
        now = time.time()

        workers_above: set[str] = set()
        for worker_id, seen in self._current_sightings.items():
            overlap = len(seen & canary_keys)
            pct = overlap / total
            if pct >= self._threshold:
                workers_above.add(worker_id)
                streak = self._streak.get(worker_id, 0) + 1
                self._streak[worker_id] = streak
                if streak >= self._required_rotations:
                    alerts.append(
                        EclipseAlert(
                            worker_id=worker_id,
                            co_location_pct=pct,
                            consecutive_rotations=streak,
                            detected_at=now,
                        )
                    )
                    _log.warning(
                        "eclipse_alert",
                        worker_id=worker_id,
                        co_location_pct=round(pct, 3),
                        streak=streak,
                    )

        # Reset streak for workers that were NOT above threshold
        for worker_id in list(self._streak):
            if worker_id not in workers_above:
                del self._streak[worker_id]

        return alerts

    def advance_rotation(self) -> None:
        """Clear current-rotation sightings for a fresh window."""
        self._current_sightings.clear()


class CanaryRotationScheduler:
    """Manages the canary rotation cadence."""

    def __init__(
        self,
        interval_s: float = DEFAULT_ROTATION_INTERVAL_S,
        canary_count: int = DEFAULT_CANARY_COUNT,
    ) -> None:
        self._interval_s = interval_s
        self._canary_count = canary_count
        self._last_rotation: float = 0.0
        self._current_canaries: list[CanaryPeer] = []
        self._detector = EclipseDetector()

    @property
    def current_canaries(self) -> list[CanaryPeer]:
        return list(self._current_canaries)

    @property
    def detector(self) -> EclipseDetector:
        return self._detector

    def should_rotate(self, now: float | None = None) -> bool:
        """True if enough time has elapsed since the last rotation."""
        if now is None:
            now = time.time()
        return (now - self._last_rotation) >= self._interval_s

    def rotate(self, now: float | None = None) -> list[EclipseAlert]:
        """Evaluate the current rotation, generate new canaries, advance.

        Returns any eclipse alerts from the completed rotation.
        """
        if now is None:
            now = time.time()

        # Evaluate current window before rotating
        alerts = self._detector.evaluate(self._current_canaries)

        # Advance detector and generate fresh canaries
        self._detector.advance_rotation()
        self._current_canaries = CanaryPeerFactory.generate(self._canary_count)
        self._last_rotation = now

        _log.info(
            "canary_rotation",
            canary_count=len(self._current_canaries),
            alerts_raised=len(alerts),
        )
        return alerts
