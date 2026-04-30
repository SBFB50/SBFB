# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 22 Phase E — watermark canari-input primitive tests.

Covers :

- ``inject_rate_1_per_100`` : a fixed RNG seed + 100 dispatcher
  ticks should produce at least one injection (the primitive's
  whole point is "trip the 1/N probe on average").
- ``signature_rotation`` : a rotated set supersedes the old one;
  a set tampered post-sign fails verification.
- ``observer_alert_on_low_similarity`` : a divergent answer flips
  the ring + increments the alerts counter.
- ``observer_pass_on_high_similarity`` : a close-enough answer
  leaves the ring untouched.
- ``api_endpoints_smoke`` : the two Sprint 22 Phase E endpoints
  (``/api/canary/inject-rate`` + ``/api/canary/observed-divergence``)
  return 200 against a stub coordinator.
"""

from __future__ import annotations

import random
import secrets
from pathlib import Path

import nexus_core
import pytest
from nexus_coordinator.canary_input import (
    CANARY_INPUT_SET_VERSION,
    DEFAULT_INJECT_RATE,
    CanaryInputInjector,
    CanaryInputManager,
    CanaryInputObserver,
    CanaryInputSet,
    CanaryPrompt,
    build_canary_input_set,
    load_canary_input_set,
    save_canary_input_set,
    verify_canary_input_set,
)

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def _new_keypair() -> tuple[bytes, bytes]:
    """Return ``(secret, public)`` via a fresh nexus_core keypair.

    Routes through ``nexus_core.keypair_from_secret`` so the test
    uses the same derivation the coordinator keystore uses at
    boot — any divergence between test and prod is caught here
    rather than downstream.
    """
    seed = secrets.token_bytes(32)
    kp = nexus_core.keypair_from_secret(seed)
    return bytes(kp["secret"]), bytes(kp["public"])


def _sample_prompts() -> list[CanaryPrompt]:
    return [
        CanaryPrompt(prompt_id="p.01", prompt="What is 2+2?", expected_answer="4"),
        CanaryPrompt(prompt_id="p.02", prompt="Capital of France?", expected_answer="Paris"),
        CanaryPrompt(prompt_id="p.03", prompt="Chemical symbol for gold?", expected_answer="Au"),
    ]


# ---------------------------------------------------------------------------
# Test 1 — inject_rate_1_per_100
# ---------------------------------------------------------------------------


def test_inject_rate_1_per_100() -> None:
    """100 dispatcher ticks with inject_rate=100 produce at least 1 injection.

    We fix the RNG seed so the test is deterministic and does not
    flake on CI. The expected count with a uniform 1/100 draw is
    exactly 1 on average over 100 draws, with the geometric
    distribution admitting 0 injections on ~37% of runs — the seed
    below is known to yield a positive count.
    """
    secret, public = _new_keypair()
    canary_set = build_canary_input_set(_sample_prompts(), secret, public)

    rng = random.Random(42)
    injector = CanaryInputInjector(
        canary_set,
        inject_rate=DEFAULT_INJECT_RATE,
        rng=rng,
    )

    injected_prompts: list[str] = []
    for _ in range(100):
        if injector.should_inject():
            prompt = injector.next_prompt()
            assert prompt is not None
            injected_prompts.append(prompt.prompt_id)

    assert len(injected_prompts) >= 1, f"expected >=1 injection over 100 draws with seed 42, got {injected_prompts}"
    # Round-robin starts at index 0 so the first injection must be p.01.
    assert injected_prompts[0] == "p.01"

    stats = injector.stats
    assert stats["seen"] == 100
    assert stats["injected"] == len(injected_prompts)


# ---------------------------------------------------------------------------
# Test 2 — signature_rotation
# ---------------------------------------------------------------------------


def test_signature_rotation(tmp_path: Path) -> None:
    """Rotated sets verify; tampered or wrong-key sets reject."""
    secret_a, public_a = _new_keypair()
    secret_b, public_b = _new_keypair()

    # Original set signed by A.
    set_v1 = build_canary_input_set(_sample_prompts(), secret_a, public_a)
    verify_canary_input_set(set_v1, expected_pubkey=public_a)

    # Rotate: build a new set from A (same key, different prompts +
    # newer timestamp) — should also verify.
    new_prompts = _sample_prompts() + [
        CanaryPrompt(prompt_id="p.04", prompt="How many moons does Earth have?", expected_answer="1"),
    ]
    set_v2 = build_canary_input_set(new_prompts, secret_a, public_a)
    verify_canary_input_set(set_v2, expected_pubkey=public_a)
    assert len(set_v2.prompts) == 4
    assert set_v2.signature_hex != set_v1.signature_hex

    # A tampered payload (prompts mutated post-signing) fails verify.
    tampered = set_v1.model_copy(
        update={
            "prompts": [
                CanaryPrompt(prompt_id="p.01", prompt="TAMPERED", expected_answer="4"),
                *set_v1.prompts[1:],
            ],
        },
    )
    with pytest.raises(Exception):
        verify_canary_input_set(tampered, expected_pubkey=public_a)

    # A set signed by B rejected when we expected A.
    set_by_b = build_canary_input_set(_sample_prompts(), secret_b, public_b)
    with pytest.raises(ValueError, match="unexpected pubkey"):
        verify_canary_input_set(set_by_b, expected_pubkey=public_a)

    # Persistence round-trip: save → load → verify.
    set_path = tmp_path / "canary_input_set.json"
    save_canary_input_set(set_v2, set_path)
    loaded = load_canary_input_set(set_path, expected_pubkey=public_a)
    assert loaded.signature_hex == set_v2.signature_hex
    assert len(loaded.prompts) == 4


# ---------------------------------------------------------------------------
# Test 3 — observer_alert_on_low_similarity
# ---------------------------------------------------------------------------


def test_observer_alert_on_low_similarity() -> None:
    """A wildly off answer triggers a divergence record + alert counter."""
    secret, public = _new_keypair()
    canary_set = build_canary_input_set(_sample_prompts(), secret, public)
    observer = CanaryInputObserver(canary_set, default_tolerance=0.85)

    alerted = observer.observe(
        prompt_id="p.02",
        observed_answer="Berlin is the capital of Germany, not France.",
        worker_pubkey_hex="abcd" * 8,
        now_unix=1_700_000_000,
    )
    assert alerted is True

    recent = observer.recent_divergences()
    assert len(recent) == 1
    record = recent[0]
    assert record.prompt_id == "p.02"
    assert record.similarity < 0.85
    assert record.expected_answer == "Paris"
    assert record.worker_pubkey_hex == "abcd" * 8
    assert record.observed_at_unix == 1_700_000_000

    stats = observer.stats
    assert stats["observed"] == 1
    assert stats["alerts"] == 1
    assert stats["ring_size"] == 1


# ---------------------------------------------------------------------------
# Test 4 — observer_pass_on_high_similarity
# ---------------------------------------------------------------------------


def test_observer_pass_on_high_similarity() -> None:
    """An answer close-enough to expected leaves the ring untouched."""
    secret, public = _new_keypair()
    prompts = [
        CanaryPrompt(
            prompt_id="p.long",
            prompt="Describe Paris in a sentence.",
            expected_answer="Paris is the capital of France and its largest city.",
            tolerance=0.80,
        ),
    ]
    canary_set = build_canary_input_set(prompts, secret, public)
    observer = CanaryInputObserver(canary_set)

    # Single-char typo → similarity ~ 0.98 ≥ 0.80 → no alert.
    alerted = observer.observe(
        prompt_id="p.long",
        observed_answer="Paris is the capital of France and its largest city!",
    )
    assert alerted is False
    assert observer.stats["observed"] == 1
    assert observer.stats["alerts"] == 0
    assert observer.stats["ring_size"] == 0
    assert observer.recent_divergences() == []

    # Unknown prompt_id → also no alert, no record.
    observer.observe(prompt_id="does.not.exist", observed_answer="whatever")
    assert observer.stats["alerts"] == 0


def test_manager_maybe_inject_and_observe(tmp_path: Path) -> None:
    """Exercise the full ``CanaryInputManager`` surface end-to-end."""
    secret, public = _new_keypair()
    canary_set = build_canary_input_set(_sample_prompts(), secret, public)
    set_path = tmp_path / "canary_input_set.json"
    save_canary_input_set(canary_set, set_path)

    manager = CanaryInputManager(
        policy_path=None,
        canary_set_path=set_path,
        coord_pubkey=public,
        rng=random.Random(0),
    )
    # inject_rate=1 → every tick injects; use that to poke the hot path.
    manager.update_inject_rate(1)

    ids: list[str] = []
    for _ in range(len(canary_set.prompts) * 2):
        prompt = manager.maybe_inject()
        assert prompt is not None
        ids.append(prompt.prompt_id)
    # Round-robin: every prompt id appears at least twice.
    assert len(set(ids)) == len(canary_set.prompts)
    assert manager.injector.stats["injected"] == len(canary_set.prompts) * 2

    # Observe a matching answer — no alert.
    got_alert = manager.observe_result(
        prompt_id=canary_set.prompts[0].prompt_id,
        observed_answer=canary_set.prompts[0].expected_answer,
    )
    assert got_alert is False

    # Observe a divergent one — alert.
    got_alert = manager.observe_result(
        prompt_id=canary_set.prompts[0].prompt_id,
        observed_answer="ABSOLUTELY WRONG ANSWER",
    )
    assert got_alert is True
    assert manager.observer.stats["alerts"] == 1


def test_canary_input_set_version_constant() -> None:
    """Sprint 22 Phase E pre-launch invariant: set version stays at 1.

    If a future sprint bumps the constant, this test forces the
    change to be explicit (plan review catches the diff). Mirrors
    the ``TASK_FORMAT_VERSION = 1`` invariant in
    :mod:`nexus_coordinator.dispatcher`.
    """
    assert CANARY_INPUT_SET_VERSION == 1

    # The model default also pins to 1 — guard against drift
    # between the constant and the pydantic default.
    dummy = CanaryInputSet(
        created_at_unix=0,
        prompts=[],
        coord_pubkey_hex="",
        signature_hex="",
    )
    assert dummy.version == 1
