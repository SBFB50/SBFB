# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 27 Phase B — watermark detector tests.

Four tests per plan §4 Phase B.3:
- watermarked output (biased green tokens → z > threshold)
- non-watermarked output (random tokens → z < threshold)
- edge cases (empty, short, boundary)
- PRF determinism (same input → same score)
"""

from __future__ import annotations

import hmac as hmac_mod
import struct

from nexus_coordinator.watermark_detector import WatermarkDetector, WatermarkResult

SECRET = b"test-watermark-secret-for-phase-b"
WINDOW = 4


def _prf_score(secret: bytes, context: list[int], token_id: int) -> float:
    """Standalone PRF to generate biased token sequences."""
    msg = b""
    for ctx_tok in context:
        msg += struct.pack("<I", ctx_tok & 0xFFFFFFFF)
    msg += struct.pack("<I", token_id & 0xFFFFFFFF)
    digest = hmac_mod.new(secret, msg, "sha256").digest()
    top8 = int.from_bytes(digest[:8], "big")
    return top8 / ((1 << 64) - 1)


def _generate_watermarked_tokens(secret: bytes, window: int, count: int, vocab: int = 1000) -> list[int]:
    """Generate a token sequence biased towards green tokens.

    For each position, pick the token with the highest PRF score
    among a sample — guaranteeing most tokens are green.
    """
    tokens: list[int] = [42, 100, 200, 300]  # seed context
    for _ in range(count):
        context = tokens[-window:]
        best_token = 0
        best_score = -1.0
        for candidate in range(0, vocab, 10):  # sample every 10th
            score = _prf_score(secret, context, candidate)
            if score > best_score:
                best_score = score
                best_token = candidate
        tokens.append(best_token)
    return tokens


def _generate_random_tokens(count: int, vocab: int = 1000) -> list[int]:
    """Generate a pseudo-random token sequence (no watermark bias).

    Uses a simple LCG to be deterministic without importing random.
    """
    tokens: list[int] = []
    state = 12345
    for _ in range(count):
        state = (state * 1103515245 + 12345) & 0x7FFFFFFF
        tokens.append(state % vocab)
    return tokens


def test_watermark_detector_watermarked_output() -> None:
    """Biased green tokens must produce z_score > threshold."""
    detector = WatermarkDetector(SECRET, window_size=WINDOW, threshold_z=2.0)
    tokens = _generate_watermarked_tokens(SECRET, WINDOW, count=200)

    result = detector.is_watermarked(tokens)
    assert isinstance(result, WatermarkResult)
    assert result.is_watermarked, (
        f"watermarked output should be detected: z={result.z_score:.2f}, green_ratio={result.green_ratio:.2f}"
    )
    assert result.z_score >= 2.0
    assert result.green_ratio > 0.6


def test_watermark_detector_non_watermarked_output() -> None:
    """Random tokens should not be flagged as watermarked."""
    detector = WatermarkDetector(SECRET, window_size=WINDOW, threshold_z=2.0)
    tokens = _generate_random_tokens(count=200)

    result = detector.is_watermarked(tokens)
    assert isinstance(result, WatermarkResult)
    assert not result.is_watermarked, f"random output should not be detected: z={result.z_score:.2f}"
    assert result.z_score < 2.0
    assert 0.3 < result.green_ratio < 0.7


def test_watermark_detector_edge_cases() -> None:
    """Empty, short, and boundary outputs."""
    detector = WatermarkDetector(SECRET, window_size=WINDOW, threshold_z=2.0)

    # Empty output
    result = detector.is_watermarked([])
    assert not result.is_watermarked
    assert result.z_score == 0.0
    assert result.token_count == 0

    # Shorter than window — cannot score
    result = detector.is_watermarked([1, 2, 3])
    assert not result.is_watermarked
    assert result.z_score == 0.0
    assert result.token_count == 3

    # Exactly window_size — cannot score (need > window tokens)
    result = detector.is_watermarked([1, 2, 3, 4])
    assert not result.is_watermarked
    assert result.token_count == 4

    # window_size + 1 — scores exactly 1 token
    result = detector.is_watermarked([1, 2, 3, 4, 5])
    assert result.token_count == 5
    # z_score is either very positive or very negative for 1 sample
    assert isinstance(result.z_score, float)


def test_watermark_prf_determinism() -> None:
    """Same secret + context + token must always produce the same score."""
    detector = WatermarkDetector(SECRET, window_size=WINDOW)
    context = (100, 200, 300, 400)
    token = 42

    score1 = detector._prf_score(token, context)
    score2 = detector._prf_score(token, context)
    assert score1 == score2, "PRF must be deterministic"
    assert 0.0 <= score1 < 1.0, "score must be in [0, 1)"

    # Different token → different score
    score3 = detector._prf_score(43, context)
    assert score1 != score3, "different tokens should produce different scores"

    # Different secret → different score
    other = WatermarkDetector(b"different-secret-for-prf-test!!", window_size=WINDOW)
    score4 = other._prf_score(token, context)
    assert score1 != score4, "different secrets should produce different scores"
