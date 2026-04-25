# SPDX-License-Identifier: AGPL-3.0-or-later
"""Sprint 27 Phase B — output watermark z-test detector.

Coordinator-side complement to the worker-side watermark injector
(``crates/nexus-worker-core/src/llm/watermark.rs``). Both sides
share the same HMAC-SHA256 PRF so the detector can verify whether
an output was generated with watermark bias without access to the
model or its logits.

**Non-blocking**: a non-watermarked result is logged as a warning
but never rejected — the worker may have watermark injection
disabled or may be running the Ollama backend (which has no logit
hook).

Technique: SynthID-inspired green/red token partition (Nature 2024
Google DeepMind). BIRA-resistant vs Kirchenbauer KGW (rejected,
arXiv:2509.23019 Sept 2025) via multi-token context window +
per-task rotating secret.
"""

from __future__ import annotations

import dataclasses
import hmac
import math
import struct


@dataclasses.dataclass(frozen=True, slots=True)
class WatermarkResult:
    """Outcome of a watermark z-test on a result's token IDs."""

    is_watermarked: bool
    z_score: float
    green_ratio: float
    token_count: int


class WatermarkDetector:
    """Z-test binomial detector for SynthID-inspired watermark.

    Parameters
    ----------
    secret : bytes
        Shared secret (same as the ``watermark_seed`` in the Task
        dispatch payload).
    window_size : int
        Number of preceding tokens used as PRF context. Must match
        the worker's ``[watermark] window_size``.
    threshold_z : float
        Minimum z-score to declare "watermarked". 2.0 corresponds
        to a ~2.3 % false-positive rate under H0 (no watermark).
    """

    def __init__(
        self,
        secret: bytes,
        window_size: int = 4,
        threshold_z: float = 2.0,
    ) -> None:
        self._secret = secret
        self._window = window_size
        self._threshold = threshold_z

    def is_watermarked(self, token_ids: list[int]) -> WatermarkResult:
        """Run the z-test on a sequence of output token IDs."""
        n = len(token_ids)
        if n <= self._window:
            return WatermarkResult(
                is_watermarked=False,
                z_score=0.0,
                green_ratio=0.0,
                token_count=n,
            )

        green = 0
        scored = 0
        for i in range(self._window, n):
            context = tuple(token_ids[max(0, i - self._window) : i])
            score = self._prf_score(token_ids[i], context)
            if score >= 0.5:
                green += 1
            scored += 1

        if scored == 0:
            return WatermarkResult(
                is_watermarked=False,
                z_score=0.0,
                green_ratio=0.0,
                token_count=n,
            )

        green_ratio = green / scored
        # Z-test: H0 = tokens are uniform (green_ratio ~ 0.5)
        # z = (observed - expected) / std_dev
        # std_dev = sqrt(p * (1-p) / n) with p = 0.5
        std_dev = math.sqrt(0.25 / scored)
        z_score = (green_ratio - 0.5) / std_dev

        return WatermarkResult(
            is_watermarked=z_score >= self._threshold,
            z_score=z_score,
            green_ratio=green_ratio,
            token_count=n,
        )

    def _prf_score(self, token_id: int, context: tuple[int, ...]) -> float:
        """HMAC-SHA256(secret, context || token_id) → [0, 1).

        Byte layout matches the Rust implementation exactly:
        each token ID is 4 bytes little-endian.
        """
        msg = b""
        for ctx_tok in context:
            msg += struct.pack("<I", ctx_tok & 0xFFFFFFFF)
        msg += struct.pack("<I", token_id & 0xFFFFFFFF)
        digest = hmac.new(self._secret, msg, "sha256").digest()
        top8 = int.from_bytes(digest[:8], "big")
        return top8 / ((1 << 64) - 1)
