"""
Tests for Phase 6 — Security: Proof-of-Computation (3 layers).

Covers:
- Ed25519 key generation, signing, verification (Couche 1)
- Model digest whitelist verification (Couche 2)
- Logprob fingerprinting verification (Couche 3)
- Combined 3-layer ResultVerifier
- Trust score deltas per verification outcome
- Calibration prompt selection
"""

import pytest

from nexus.compute.crypto import (
    HAS_CRYPTO,
    generate_keypair,
    sign_result,
    verify_signature,
    _build_payload,
)
from nexus.compute.verification import (
    CALIBRATION_PROMPTS,
    ResultVerifier,
    get_random_calibration_prompt,
    register_digest,
    register_logprob_profile,
    should_calibrate,
    verify_digest,
    verify_logprobs,
    _DIGEST_WHITELIST,
    _LOGPROB_PROFILES,
)


# ===================================================================
# Couche 1: Ed25519
# ===================================================================

class TestEd25519Crypto:
    """Test Ed25519 key management and signing."""

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_generate_keypair(self):
        priv, pub = generate_keypair()
        assert len(priv) > 0
        assert len(pub) > 0
        assert b"PRIVATE KEY" in priv
        assert b"PUBLIC KEY" in pub

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_sign_and_verify(self):
        priv, pub = generate_keypair()
        sig = sign_result(priv, "task-1", "Result text here", "sha256:abc", "node-1")
        assert len(sig) > 0

        ok = verify_signature(pub, sig, "task-1", "Result text here", "sha256:abc", "node-1")
        assert ok is True

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_verify_wrong_data_fails(self):
        priv, pub = generate_keypair()
        sig = sign_result(priv, "task-1", "Result A", "sha256:abc", "node-1")

        # Verify with different result_text
        ok = verify_signature(pub, sig, "task-1", "Result B", "sha256:abc", "node-1")
        assert ok is False

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_verify_wrong_key_fails(self):
        priv1, pub1 = generate_keypair()
        _, pub2 = generate_keypair()
        sig = sign_result(priv1, "task-1", "Result", "sha256:abc", "node-1")

        # Verify with wrong public key
        ok = verify_signature(pub2, sig, "task-1", "Result", "sha256:abc", "node-1")
        assert ok is False

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_verify_empty_signature_fails(self):
        _, pub = generate_keypair()
        ok = verify_signature(pub, "", "task-1", "Result", "sha256:abc", "node-1")
        assert ok is False

    def test_sign_without_crypto_returns_empty(self):
        """If no private key provided, signing returns empty string."""
        sig = sign_result(b"", "task-1", "Result", "sha256:abc", "node-1")
        assert sig == ""

    def test_build_payload_deterministic(self):
        p1 = _build_payload("t1", "result", "digest", "node")
        p2 = _build_payload("t1", "result", "digest", "node")
        assert p1 == p2

    def test_build_payload_truncates(self):
        long_text = "x" * 5000
        payload = _build_payload("t1", long_text, "d", "n")
        assert len(payload) < 5000  # Truncated


# ===================================================================
# Couche 2: Digest whitelist
# ===================================================================

class TestDigestWhitelist:
    """Test model digest verification."""

    def setup_method(self):
        _DIGEST_WHITELIST.clear()

    def test_no_whitelist_passes(self):
        ok, reason = verify_digest("llama-70b", "sha256:anything")
        assert ok is True
        assert reason == "no_whitelist"

    def test_missing_digest_fails(self):
        register_digest("llama-70b", "sha256:expected")
        ok, reason = verify_digest("llama-70b", "")
        assert ok is False
        assert reason == "missing_digest"

    def test_matching_digest_passes(self):
        register_digest("llama-70b", "sha256:abc123")
        ok, reason = verify_digest("llama-70b", "sha256:abc123")
        assert ok is True
        assert reason == "digest_match"

    def test_wrong_digest_fails(self):
        register_digest("llama-70b", "sha256:expected")
        ok, reason = verify_digest("llama-70b", "sha256:wrong")
        assert ok is False
        assert "digest_mismatch" in reason

    def test_unknown_model_passes(self):
        register_digest("llama-70b", "sha256:abc")
        ok, reason = verify_digest("unknown-model", "sha256:xyz")
        assert ok is True
        assert reason == "model_not_in_whitelist"


# ===================================================================
# Couche 3: Logprob fingerprinting
# ===================================================================

class TestLogprobFingerprinting:
    """Test logprob verification."""

    def setup_method(self):
        _LOGPROB_PROFILES.clear()

    def test_no_profiles_passes(self):
        ok, reason = verify_logprobs("model", "prompt", {"Paris": -0.03})
        assert ok is True
        assert reason == "no_profiles_configured"

    def test_matching_logprobs_passes(self):
        register_logprob_profile("llama-70b", "La capitale de la France est", {
            "Paris": -0.03, "Lyon": -4.6, "Marseille": -5.3,
        })
        ok, reason = verify_logprobs("llama-70b", "La capitale de la France est", {
            "Paris": -0.05, "Lyon": -4.5, "Marseille": -5.1,
        })
        assert ok is True
        assert "logprob_match" in reason

    def test_divergent_logprobs_fails(self):
        register_logprob_profile("llama-70b", "La capitale de la France est", {
            "Paris": -0.03, "Lyon": -4.6,
        })
        ok, reason = verify_logprobs("llama-70b", "La capitale de la France est", {
            "Paris": -2.0, "Lyon": -1.0,  # Way off
        })
        assert ok is False
        assert "logprob_divergence" in reason

    def test_missing_logprobs_fails(self):
        register_logprob_profile("llama-70b", "prompt", {"Paris": -0.03})
        ok, reason = verify_logprobs("llama-70b", "prompt", {})
        assert ok is False
        assert reason == "missing_logprobs"

    def test_calibration_prompts_not_empty(self):
        assert len(CALIBRATION_PROMPTS) >= 4

    def test_random_calibration_prompt(self):
        prompt = get_random_calibration_prompt()
        assert prompt in CALIBRATION_PROMPTS

    def test_should_calibrate_probabilistic(self):
        # Run 1000 times, expect ~100 True (10%)
        results = [should_calibrate() for _ in range(1000)]
        rate = sum(results) / len(results)
        assert 0.03 < rate < 0.20  # Wide tolerance for randomness


# ===================================================================
# Combined ResultVerifier
# ===================================================================

class TestResultVerifier:
    """Test the 3-layer combined verifier."""

    def setup_method(self):
        _DIGEST_WHITELIST.clear()
        _LOGPROB_PROFILES.clear()

    def test_all_pass_no_checks(self):
        """No signatures, no digest whitelist, no calibration → passes with +1 trust."""
        v = ResultVerifier()
        result = v.verify(
            task_id="t1", node_id="n1", result_text="Some result",
            model="llama-70b",
        )
        assert result["passed"] is True
        assert result["trust_delta"] == 1
        assert result["ban"] is False

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_valid_signature_passes(self):
        priv, pub = generate_keypair()
        sig = sign_result(priv, "t1", "Result", "sha256:d", "n1")

        v = ResultVerifier()
        result = v.verify(
            task_id="t1", node_id="n1", result_text="Result",
            model="m", model_digest="sha256:d",
            signature_b64=sig, public_key_pem=pub,
        )
        assert result["passed"] is True
        assert result["checks"]["signature"]["passed"] is True

    @pytest.mark.skipif(not HAS_CRYPTO, reason="cryptography not installed")
    def test_invalid_signature_bans(self):
        _, pub = generate_keypair()

        v = ResultVerifier()
        result = v.verify(
            task_id="t1", node_id="n1", result_text="Result",
            model="m", model_digest="d",
            signature_b64="invalid_base64_sig", public_key_pem=pub,
        )
        assert result["passed"] is False
        assert result["trust_delta"] == -50
        assert result["ban"] is True

    def test_digest_mismatch_bans(self):
        register_digest("llama-70b", "sha256:expected")

        v = ResultVerifier()
        result = v.verify(
            task_id="t1", node_id="n1", result_text="Result",
            model="llama-70b", model_digest="sha256:wrong",
        )
        assert result["passed"] is False
        assert result["trust_delta"] == -50
        assert result["ban"] is True

    def test_logprob_divergence_flags_but_no_ban(self):
        register_logprob_profile("llama-70b", "prompt", {"Paris": -0.03})

        v = ResultVerifier()
        result = v.verify(
            task_id="t1", node_id="n1", result_text="Result",
            model="llama-70b",
            calibration_prompt="prompt",
            logprobs={"Paris": -5.0},  # Way off
        )
        assert result["passed"] is True  # Still accepted (suspect, not banned)
        assert result["trust_delta"] == -5
        assert result["ban"] is False

    def test_spot_check_needed_trusted(self):
        v = ResultVerifier()
        # Trusted nodes (>80) should rarely need spot-check
        checks = [v.spot_check_needed(90) for _ in range(1000)]
        rate = sum(checks) / len(checks)
        assert rate < 0.05  # Should be ~1%

    def test_spot_check_needed_suspect(self):
        v = ResultVerifier()
        # Suspect nodes (<50) should often need spot-check
        checks = [v.spot_check_needed(30) for _ in range(1000)]
        rate = sum(checks) / len(checks)
        assert rate > 0.10  # Should be ~20%


# ===================================================================
# Module imports
# ===================================================================

class TestPhase6Imports:
    """Test Phase 6 module imports."""

    def test_import_crypto(self):
        from nexus.compute.crypto import generate_keypair, sign_result, verify_signature
        assert callable(generate_keypair)

    def test_import_verification(self):
        from nexus.compute.verification import ResultVerifier, verify_digest, verify_logprobs
        assert ResultVerifier is not None

    def test_import_calibration_prompts(self):
        from nexus.compute.verification import CALIBRATION_PROMPTS
        assert len(CALIBRATION_PROMPTS) >= 4
