"""
NEXUS Compute -- 3-Layer Proof-of-Computation Verification.

Combines three independent verification layers:

Couche 1 — Ed25519 signature (WHO sent the result)
  → Proves identity via cryptographic non-repudiation
  → Instant verification: ~0.1ms

Couche 2 — Ollama model digest (WHICH model is loaded)
  → SHA256 of model weights file = unique per model
  → Compared against server whitelist
  → Instant verification: string comparison

Couche 3 — Logprob fingerprinting (DID the right model run)
  → Calibration prompts produce unique logprob distributions per model
  → KL-divergence between expected and actual profiles
  → >95% accuracy with 8 queries (LLMmap, USENIX Security 2025)
  → Only applied to 10% of tasks (calibration_prompt set)

BOINC-style spot-checking — Safety net
  → 5% of tasks re-executed on trusted GPU
  → Result similarity check (semantic, not exact)
  → Trust score adjustment on divergence

Trust scoring:
  +1  per accepted task (all checks pass)
  +5  per spot-check pass
  -20 per spot-check fail
  -50 per signature invalid
  <20 → auto-ban
"""

from __future__ import annotations

import math
import random
from typing import Any, Optional

from loguru import logger

from nexus.compute.crypto import verify_signature


# ============================================================================
# Couche 2 — Model digest whitelist
# ============================================================================

# Populated at runtime by scanning trusted GPU or manually configured.
# Maps model_name → expected SHA256 digest.
_DIGEST_WHITELIST: dict[str, str] = {}


def register_digest(model: str, digest: str) -> None:
    """Register a known-good model digest in the whitelist."""
    _DIGEST_WHITELIST[model] = digest
    logger.debug("Registered digest for {}: {}...", model, digest[:16])


def get_digest_whitelist() -> dict[str, str]:
    """Return the current digest whitelist (read-only copy)."""
    return dict(_DIGEST_WHITELIST)


def verify_digest(model: str, reported_digest: str) -> tuple[bool, str]:
    """Verify a model digest against the whitelist.

    Returns (passed, reason).
    """
    if not _DIGEST_WHITELIST:
        # No whitelist configured — skip check (log warning)
        return True, "no_whitelist"

    if not reported_digest:
        return False, "missing_digest"

    expected = _DIGEST_WHITELIST.get(model)
    if expected is None:
        # Model not in whitelist — accept but flag
        return True, "model_not_in_whitelist"

    if reported_digest == expected:
        return True, "digest_match"

    return False, f"digest_mismatch (expected {expected[:16]}..., got {reported_digest[:16]}...)"


# ============================================================================
# Couche 3 — Logprob fingerprinting
# ============================================================================

# Calibration prompts sent with 10% of tasks
CALIBRATION_PROMPTS = [
    "La capitale de la France est",
    "Le president de la Republique en 2026 est",
    "L'article 49.3 permet au gouvernement de",
    "Le nombre de deputes a l'Assemblee nationale est",
    "La devise de la France est",
    "Le Senat est compose de",
    "La Constitution de la Cinquieme Republique date de",
    "Le Premier ministre est nomme par",
]

# Reference logprob profiles per model (calibrated on trusted GPU)
# Maps model → prompt → {token: logprob}
_LOGPROB_PROFILES: dict[str, dict[str, dict[str, float]]] = {}


def register_logprob_profile(model: str, prompt: str, profile: dict[str, float]) -> None:
    """Register a reference logprob profile for a model+prompt pair."""
    if model not in _LOGPROB_PROFILES:
        _LOGPROB_PROFILES[model] = {}
    _LOGPROB_PROFILES[model][prompt] = profile


def get_random_calibration_prompt() -> str:
    """Get a random calibration prompt for fingerprinting."""
    return random.choice(CALIBRATION_PROMPTS)


def should_calibrate() -> bool:
    """Return True for ~10% of calls (calibration rate)."""
    return random.random() < 0.10


def verify_logprobs(
    model: str,
    calibration_prompt: str,
    actual_logprobs: dict[str, float],
    threshold: float = 0.5,
) -> tuple[bool, str]:
    """Verify logprob fingerprint against reference profile.

    Uses max absolute difference (simpler than KL-divergence,
    more robust to missing tokens).

    Returns (passed, reason).
    """
    if not _LOGPROB_PROFILES:
        return True, "no_profiles_configured"

    model_profiles = _LOGPROB_PROFILES.get(model)
    if not model_profiles:
        return True, "model_not_profiled"

    expected = model_profiles.get(calibration_prompt)
    if not expected:
        return True, "prompt_not_profiled"

    if not actual_logprobs:
        return False, "missing_logprobs"

    # Compare top tokens
    max_diff = 0.0
    matched = 0
    for token, expected_logp in expected.items():
        actual_logp = actual_logprobs.get(token)
        if actual_logp is not None:
            diff = abs(expected_logp - actual_logp)
            max_diff = max(max_diff, diff)
            matched += 1

    if matched == 0:
        return False, "no_matching_tokens"

    if max_diff > threshold:
        return False, f"logprob_divergence ({max_diff:.3f} > {threshold})"

    return True, f"logprob_match (max_diff={max_diff:.3f}, matched={matched})"


# ============================================================================
# Combined 3-layer verification
# ============================================================================

class ResultVerifier:
    """Runs all 3 verification layers on a compute result.

    Usage::

        verifier = ResultVerifier()
        result = verifier.verify(
            task_id="abc", node_id="xyz", result_text="...",
            model="llama-70b", model_digest="sha256:...",
            signature_b64="...", public_key_pem=b"...",
            calibration_prompt="La capitale...",
            logprobs={"Paris": -0.03, ...},
        )
        # result = {"passed": True, "checks": {...}, "trust_delta": 1}
    """

    def verify(
        self,
        task_id: str,
        node_id: str,
        result_text: str,
        model: str,
        model_digest: str = "",
        signature_b64: str = "",
        public_key_pem: bytes = b"",
        calibration_prompt: str = "",
        logprobs: Optional[dict[str, float]] = None,
    ) -> dict[str, Any]:
        """Run all verification layers.

        Returns:
            {
                "passed": bool,
                "checks": {
                    "signature": {"passed": bool, "reason": str},
                    "digest": {"passed": bool, "reason": str},
                    "logprobs": {"passed": bool, "reason": str},
                },
                "trust_delta": int,
                "ban": bool,
            }
        """
        checks: dict[str, dict[str, Any]] = {}
        trust_delta = 1  # Default: +1 for accepted task
        ban = False

        # Couche 1: Ed25519 signature
        if signature_b64 and public_key_pem:
            sig_ok = verify_signature(
                public_key_pem, signature_b64,
                task_id, result_text, model_digest, node_id,
            )
            checks["signature"] = {"passed": sig_ok, "reason": "valid" if sig_ok else "invalid_signature"}
            if not sig_ok:
                trust_delta = -50
                ban = True
                return {"passed": False, "checks": checks, "trust_delta": trust_delta, "ban": ban}
        else:
            checks["signature"] = {"passed": True, "reason": "not_provided"}

        # Couche 2: Model digest
        digest_ok, digest_reason = verify_digest(model, model_digest)
        checks["digest"] = {"passed": digest_ok, "reason": digest_reason}
        if not digest_ok:
            trust_delta = -50
            ban = True
            return {"passed": False, "checks": checks, "trust_delta": trust_delta, "ban": ban}

        # Couche 3: Logprob fingerprinting (only if calibration was requested)
        if calibration_prompt and logprobs:
            lp_ok, lp_reason = verify_logprobs(model, calibration_prompt, logprobs)
            checks["logprobs"] = {"passed": lp_ok, "reason": lp_reason}
            if not lp_ok:
                # Don't ban immediately — flag as suspect, increase spot-check rate
                trust_delta = -5
                return {"passed": True, "checks": checks, "trust_delta": trust_delta, "ban": False}
        else:
            checks["logprobs"] = {"passed": True, "reason": "not_calibrated"}

        return {"passed": True, "checks": checks, "trust_delta": trust_delta, "ban": ban}

    def spot_check_needed(self, trust_score: int) -> bool:
        """Determine if this result should be spot-checked on trusted GPU."""
        if trust_score >= 80:
            rate = 0.01  # 1% for trusted
        elif trust_score >= 50:
            rate = 0.05  # 5% standard
        else:
            rate = 0.20  # 20% for suspect
        return random.random() < rate
