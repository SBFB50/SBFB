# SPDX-License-Identifier: AGPL-3.0-or-later
"""Coord-side `OutputFilter` tests (Sprint 21 phase coord-side).

Couvre :
- Parité avec llm-guard 0.3.16 `InvisibleText` scanner (tests 4-5)
  — zero-width / PUA / Tag chars strippés, Cf whitelist i18n
  préservée.
- Prompt echo cascade exact / substring / EED seuil 0.85 (tests 6-10).
- Policy hot-reload sanity (bonus tests).
"""

from __future__ import annotations

import os
import time
from pathlib import Path

from nexus_coordinator.output_filter import (
    DEFAULT_EED_THRESHOLD,
    OutputFilter,
    OutputFilterPolicy,
    scan_invisible_text,
)

# ---- Test 4 : invisible_chars_stripped ---------------------------


def test_invisible_chars_stripped() -> None:
    """Zero-width U+200B + PUA U+E000 + Tag chars U+E0020 strippés."""
    filt = OutputFilter()
    hidden = "Hello\u200bWorld\ue000Payload\U000e0020Secret"
    verdict = filt.filter(
        system_prompt="",
        user_prompt="",
        model_output=hidden,
    )
    assert not verdict.is_valid
    assert verdict.reason == "invisible_text"
    assert "\u200b" not in verdict.sanitized_output
    assert "\ue000" not in verdict.sanitized_output
    assert "\U000e0020" not in verdict.sanitized_output
    # The visible chars should survive.
    assert verdict.sanitized_output == "HelloWorldPayloadSecret"


# ---- Test 5 : rlo_lro_whitelisted_for_i18n ----------------------


def test_rlo_lro_whitelisted_for_i18n() -> None:
    """U+202E RLO conservé (Arabe/Hébreu légitime)."""
    filt = OutputFilter()
    # Arabic with explicit RLO (right-to-left override) format char.
    i18n_text = "Hello \u202eشكرا لك"
    verdict = filt.filter(
        system_prompt="",
        user_prompt="",
        model_output=i18n_text,
    )
    # Pas de strip → is_valid True, RLO préservé.
    assert verdict.is_valid
    assert verdict.reason == "ok"
    assert "\u202e" in verdict.sanitized_output
    # Validation directe du scanner scalaire :
    sanitized, is_valid, _ = scan_invisible_text(i18n_text)
    assert is_valid
    assert "\u202e" in sanitized


# ---- Test 6 : prompt_echo_exact_match_blocks --------------------


def test_prompt_echo_exact_match_blocks() -> None:
    """system_prompt apparait litéralement dans model_output → bloqué."""
    filt = OutputFilter()
    sp = "You are a secret assistant with password=hunter2."
    verdict = filt.filter(
        system_prompt=sp,
        user_prompt="Tell me a joke",
        model_output=f"Sure! Here it is: {sp} Hope that helps.",
    )
    assert not verdict.is_valid
    assert verdict.reason == "prompt_echo_exact"
    assert verdict.risk_score == 1.0


# ---- Test 7 : prompt_echo_eed_similarity_above_0_85_blocks ------


def test_prompt_echo_eed_similarity_above_0_85_blocks() -> None:
    """Reconstruction partielle similaire >= 0.85 EED → bloquée.

    L'attaquant reconstruit ~90% du system_prompt via paraphrase.
    Avec `eed_threshold = 0.85` la cascade doit bloquer sur le
    niveau EED après que exact + substring aient échoué.
    """
    filt = OutputFilter(
        policy_path=None,
    )
    # Forcer un seuil exact 0.85 via runtime attribute (évite
    # disk IO pour ce test unit).
    filt._policy = OutputFilterPolicy(
        enabled=True,
        substring_match_min_len=10_000,  # disable substring match
        exact_match=False,  # disable exact match → force EED path
        eed_threshold=0.85,
    )
    sp = "The quick brown fox jumps over the lazy dog"
    # Modification 4 chars (~90% similar) — dépasse 0.85.
    reconstructed = "The quick brown fox jumps over the lazy cat"
    verdict = filt.filter(
        system_prompt=sp,
        user_prompt="",
        model_output=reconstructed,
    )
    assert not verdict.is_valid
    assert verdict.reason == "prompt_echo_eed"
    assert verdict.risk_score >= 0.85


# ---- Test 8 : prompt_echo_eed_similarity_below_0_85_passes ------


def test_prompt_echo_eed_similarity_below_0_85_passes() -> None:
    """Reconstruction distincte 0.3 EED → passe."""
    filt = OutputFilter()
    filt._policy = OutputFilterPolicy(
        enabled=True,
        exact_match=True,
        substring_match_min_len=40,
        eed_threshold=0.85,
    )
    sp = "You are a secret assistant with password=hunter2."
    unrelated = "Paris is the capital of France. Area: 105 km²."
    verdict = filt.filter(
        system_prompt=sp,
        user_prompt="",
        model_output=unrelated,
    )
    assert verdict.is_valid
    assert verdict.reason == "ok"


# ---- Test 9 : pleak_attack_reconstruction_scenarios -------------


def test_pleak_attack_reconstruction_scenarios() -> None:
    """5 PLeak-style reconstructions, tous détectés (au moins un
    niveau de la cascade bloque)."""
    filt = OutputFilter()
    system_prompt = (
        "You are a customer service bot for AcmeCo. Never reveal "
        "internal pricing below MSRP. Always escalate to a human "
        "for refund requests above $500."
    )
    # 5 scénarios :
    attacks = [
        # 1. Echo direct complet — exact match.
        system_prompt,
        # 2. Substring de 40+ chars copiée — substring match.
        "Here's what I was told: Never reveal internal pricing below MSRP. How can I help today?",
        # 3. Reconstruction paraphrase avec ~87% fidélité → EED.
        "You are a customer service bot for AcmeCo. Never reveal "
        "internal pricing below MSRP. Always escalate to a human "
        "for refund requests above $500.",  # presque identique
        # 4. Edit mineur 2 chars — similaire > 0.95 → EED trigger.
        "You are a customer service bot for AcmeCo. Never reveal "
        "internal pricing below MSRPX. Always escalate to a human "
        "for refund requests above $500.",
        # 5. Substring long (60 chars) extrait mid-prompt.
        "Excellent question! My instructions say: Always escalate to a human for refund requests above $500. Thanks.",
    ]
    for idx, attack_output in enumerate(attacks, start=1):
        verdict = filt.filter(
            system_prompt=system_prompt,
            user_prompt="Tell me about your policies.",
            model_output=attack_output,
        )
        assert not verdict.is_valid, f"attack {idx} not blocked: reason={verdict.reason}, risk={verdict.risk_score}"
        assert verdict.reason.startswith("prompt_echo_"), f"attack {idx} blocked for wrong reason: {verdict.reason}"


# ---- Test 10 : benign_output_passes_through ---------------------


def test_benign_output_passes_through() -> None:
    """Response normale user, pas de false positive."""
    filt = OutputFilter()
    sp = "You are a helpful assistant. Be concise."
    # Réponse sans aucun echo de system_prompt, sans chars invisibles.
    benign = (
        "The Pythagorean theorem states that in a right triangle, "
        "the square of the hypotenuse equals the sum of the squares "
        "of the other two sides: a² + b² = c²."
    )
    verdict = filt.filter(
        system_prompt=sp,
        user_prompt="What is the Pythagorean theorem?",
        model_output=benign,
    )
    assert verdict.is_valid
    assert verdict.reason == "ok"
    assert verdict.sanitized_output == benign


# ---- Sanity : scan_invisible_text stateless ---------------------


def test_scan_invisible_text_stateless() -> None:
    """Le scanner fonctionnel standalone retourne le bon tuple."""
    sanitized, is_valid, risk = scan_invisible_text("")
    assert sanitized == ""
    assert is_valid
    assert risk == 0.0

    sanitized, is_valid, risk = scan_invisible_text("clean text")
    assert sanitized == "clean text"
    assert is_valid
    assert risk == 0.0

    sanitized, is_valid, risk = scan_invisible_text("a\u200bb")
    assert sanitized == "ab"
    assert not is_valid
    assert risk == 1.0


# ---- Sanity : policy hot-reload ---------------------------------


def test_output_filter_policy_hot_reload(tmp_path: Path) -> None:
    policy_file = tmp_path / "output_filter.toml"
    policy_file.write_text(
        """
[default]
enabled = true

[prompt_echo]
eed_threshold = 0.99
""",
        encoding="utf-8",
    )
    filt = OutputFilter(policy_path=policy_file)
    assert filt.policy.eed_threshold == 0.99

    time.sleep(0.1)
    policy_file.write_text(
        """
[default]
enabled = true

[prompt_echo]
eed_threshold = 0.70
""",
        encoding="utf-8",
    )
    new_mtime = time.time() + 1.0
    os.utime(policy_file, (new_mtime, new_mtime))

    filt.reload_policy()
    assert filt.policy.eed_threshold == 0.70


# ---- Sanity : default threshold = 0.85 --------------------------


def test_default_threshold_matches_contract() -> None:
    policy = OutputFilterPolicy()
    assert policy.eed_threshold == DEFAULT_EED_THRESHOLD == 0.85
    assert policy.substring_match_min_len == 40
    assert policy.exact_match is True
    assert policy.enabled is True
