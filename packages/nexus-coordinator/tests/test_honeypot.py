# SPDX-License-Identifier: AGPL-3.0-or-later
"""Tests for honeypot.py — eclipse detection canary peers."""

import time

from nexus_coordinator.honeypot import (
    CanaryPeerFactory,
    CanaryRotationScheduler,
    EclipseDetector,
)


class TestCanaryPeerFactory:
    def test_generates_valid_ed25519(self) -> None:
        peers = CanaryPeerFactory.generate(3)
        assert len(peers) == 3
        for p in peers:
            # Ed25519 pubkey = 32 bytes = 64 hex chars
            assert len(p.public_key_hex) == 64
            # Must be valid hex
            bytes.fromhex(p.public_key_hex)

    def test_rotation_produces_new_peers(self) -> None:
        batch_a = CanaryPeerFactory.generate(3)
        batch_b = CanaryPeerFactory.generate(3)
        keys_a = {p.public_key_hex for p in batch_a}
        keys_b = {p.public_key_hex for p in batch_b}
        # Probability of collision is negligible
        assert keys_a != keys_b


class TestEclipseDetector:
    def test_below_threshold_no_alert(self) -> None:
        detector = EclipseDetector(threshold=0.80, required_rotations=3)
        canaries = CanaryPeerFactory.generate(10)
        canary_keys = {p.public_key_hex for p in canaries}
        # Worker sees 6 out of 10 canaries = 60% < 80%
        seen = set(list(canary_keys)[:6])
        detector.report_neighborhood("worker-1", seen)
        alerts = detector.evaluate(canaries)
        assert len(alerts) == 0

    def test_above_threshold_3_rotations_alert(self) -> None:
        detector = EclipseDetector(threshold=0.80, required_rotations=3)

        for rotation in range(3):
            canaries = CanaryPeerFactory.generate(10)
            canary_keys = {p.public_key_hex for p in canaries}
            # Worker sees 9 out of 10 = 90% >= 80%
            seen = set(list(canary_keys)[:9])
            detector.report_neighborhood("worker-bad", seen)
            alerts = detector.evaluate(canaries)
            detector.advance_rotation()

            if rotation < 2:
                assert len(alerts) == 0
            else:
                assert len(alerts) == 1
                assert alerts[0].worker_id == "worker-bad"
                assert alerts[0].consecutive_rotations == 3

    def test_resets_on_rotation_miss(self) -> None:
        detector = EclipseDetector(threshold=0.80, required_rotations=3)

        # 2 rotations above threshold
        for _ in range(2):
            canaries = CanaryPeerFactory.generate(10)
            canary_keys = {p.public_key_hex for p in canaries}
            seen = set(list(canary_keys)[:9])
            detector.report_neighborhood("worker-1", seen)
            detector.evaluate(canaries)
            detector.advance_rotation()

        assert detector.streak.get("worker-1") == 2

        # 3rd rotation: worker drops below threshold (sees 5/10 = 50%)
        canaries = CanaryPeerFactory.generate(10)
        canary_keys = {p.public_key_hex for p in canaries}
        seen = set(list(canary_keys)[:5])
        detector.report_neighborhood("worker-1", seen)
        alerts = detector.evaluate(canaries)
        assert len(alerts) == 0
        # Streak reset because worker was below threshold
        assert "worker-1" not in detector.streak


class TestCanaryRotationScheduler:
    def test_should_rotate_on_fresh(self) -> None:
        sched = CanaryRotationScheduler(interval_s=3600)
        assert sched.should_rotate()

    def test_should_not_rotate_before_interval(self) -> None:
        sched = CanaryRotationScheduler(interval_s=3600)
        now = time.time()
        sched.rotate(now=now)
        assert not sched.should_rotate(now=now + 1800)

    def test_rotate_produces_canaries(self) -> None:
        sched = CanaryRotationScheduler(canary_count=5)
        alerts = sched.rotate()
        assert len(sched.current_canaries) == 5
        # First rotation has no alerts (no previous data)
        assert len(alerts) == 0
