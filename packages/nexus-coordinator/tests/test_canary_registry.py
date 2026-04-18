# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 20 Phase E.3 — federated warrant canary registry tests.

Unit tests for :mod:`nexus_coordinator.canary_registry`. The
registry is a self-contained pydantic + json-on-disk module with
no iroh / FastAPI dependency, so the tests are pure data flow :
build observations, observe, persist, reload, inspect freshness.
"""

from __future__ import annotations

from datetime import date
from pathlib import Path

from nexus_coordinator.canary_registry import (
    CanaryObservation,
    CanaryRegistry,
    DuressAckObservation,
)


def _canary(pubkey_suffix: str = "0", date_str: str = "2026-04-15") -> CanaryObservation:
    pk = ("a" * 63) + pubkey_suffix
    sig = "b" * 128
    return CanaryObservation(
        version=1,
        date=date_str,
        headline=f"headline {date_str}",
        next_update="2026-05-30",
        pubkey_hex=pk,
        signature_hex=sig,
    )


def _ack(pubkey_suffix: str = "0", date_str: str = "2026-04-18") -> DuressAckObservation:
    pk = ("a" * 63) + pubkey_suffix
    sig = "c" * 128
    return DuressAckObservation(
        version=1,
        date=date_str,
        message=f"daily phrase {date_str}",
        pubkey_hex=pk,
        signature_hex=sig,
    )


def test_registry_observe_canary_updates_state(tmp_path: Path) -> None:
    """An observed canary lands in the registry, freshness derives
    from its date relative to ``today``."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")

    obs = _canary(date_str="2026-04-15")
    reg.observe_canary(obs)

    snapshot = reg._canaries_snapshot()
    assert obs.pubkey_hex in snapshot, "canary indexed by pubkey_hex"
    assert snapshot[obs.pubkey_hex] == obs

    # Freshness check : 3 days after the canary date = fresh.
    fresh = reg.freshness(obs.pubkey_hex, today=date(2026, 4, 18))
    assert fresh.canary_date == "2026-04-15"
    assert fresh.canary_age_days == 3
    assert fresh.canary_status == "fresh"

    # Unknown pubkey → status missing across the board.
    unknown = reg.freshness("0" * 64, today=date(2026, 4, 18))
    assert unknown.canary_status == "missing"
    assert unknown.canary_age_days is None


def test_registry_stale_detection_30_45_60_days_thresholds(tmp_path: Path) -> None:
    """Verify the three thresholds in the canary freshness ladder
    : <30 days = fresh, 30..45 = warn, >=45 = stale."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")
    obs = _canary(date_str="2026-04-15")
    reg.observe_canary(obs)

    # Day 0..29 → fresh.
    f29 = reg.freshness(obs.pubkey_hex, today=date(2026, 5, 14))  # +29 days
    assert f29.canary_age_days == 29
    assert f29.canary_status == "fresh"

    # Day 30 → warn (cadence overshoot, dead-man-switch grace).
    f30 = reg.freshness(obs.pubkey_hex, today=date(2026, 5, 15))  # +30 days
    assert f30.canary_age_days == 30
    assert f30.canary_status == "warn"

    # Day 44 → still warn.
    f44 = reg.freshness(obs.pubkey_hex, today=date(2026, 5, 29))  # +44 days
    assert f44.canary_status == "warn"

    # Day 45 → stale (dead-man-switch fired).
    f45 = reg.freshness(obs.pubkey_hex, today=date(2026, 5, 30))  # +45 days
    assert f45.canary_age_days == 45
    assert f45.canary_status == "stale"

    # Day 60 → still stale.
    f60 = reg.freshness(obs.pubkey_hex, today=date(2026, 6, 14))  # +60 days
    assert f60.canary_status == "stale"


def test_registry_persist_reload_roundtrip_preserves_state(tmp_path: Path) -> None:
    """A registry persists observations to JSON and a fresh
    instance over the same path reloads them byte-for-byte."""
    persist_path = tmp_path / "subdir" / "canary-registry.json"

    # First instance : observe + persist.
    reg1 = CanaryRegistry(persist_path)
    canary = _canary(pubkey_suffix="1", date_str="2026-04-10")
    ack = _ack(pubkey_suffix="1", date_str="2026-04-17")
    reg1.observe_canary(canary)
    reg1.observe_duress_ack(ack)

    assert persist_path.exists(), "persist creates the file (and parent)"
    raw = persist_path.read_text(encoding="utf-8")
    assert canary.pubkey_hex in raw
    assert ack.pubkey_hex in raw

    # Fresh instance : reload from disk.
    reg2 = CanaryRegistry(persist_path)
    assert reg2._canaries_snapshot() == reg1._canaries_snapshot()
    assert reg2._duress_acks_snapshot() == reg1._duress_acks_snapshot()

    # Older observation for the same pubkey is silently ignored
    # (per registry contract — keep most-recent).
    older = _canary(pubkey_suffix="1", date_str="2026-03-01")
    reg2.observe_canary(older)
    assert reg2._canaries_snapshot()[canary.pubkey_hex].date == "2026-04-10"


def test_registry_tracks_duress_ack_separately_from_canary(tmp_path: Path) -> None:
    """Duress acks must be tracked on a separate axis from
    monthly canaries — same pubkey, separate freshness ladders.

    This is the Phase E.4 invariant : a maintainer who issues a
    fresh duress ack today but is 40 days late on the monthly
    canary is in a different operational state than one who is
    fresh on both — the registry exposes both axes."""
    reg = CanaryRegistry(tmp_path / "canary-registry.json")

    # Same pubkey for both observations — same maintainer, two
    # streams.
    canary = _canary(pubkey_suffix="2", date_str="2026-03-15")  # 40 days old at observation_today
    ack = _ack(pubkey_suffix="2", date_str="2026-04-23")  # 1 day old
    reg.observe_canary(canary)
    reg.observe_duress_ack(ack)

    today = date(2026, 4, 24)
    fresh = reg.freshness(canary.pubkey_hex, today=today)

    # Canary is in the warn band (40 days old, 30 <= age < 45).
    assert fresh.canary_status == "warn"
    assert fresh.canary_age_days == 40

    # Duress ack is fresh (<2 days).
    assert fresh.duress_ack_status == "fresh"
    assert fresh.duress_ack_age_days == 1

    # And critically : the duress_ack_date must be EXACTLY the
    # ack date, not derived from the canary.
    assert fresh.duress_ack_date == "2026-04-23"
    assert fresh.canary_date == "2026-03-15"

    # Network health surfaces both axes in the summary.
    health = reg.network_health(today=today)
    assert health.summary["maintainers_total"] == 1
    assert health.summary["canary_warn"] == 1
    assert health.summary["duress_ack_fresh"] == 1
    assert len(health.maintainers) == 1
