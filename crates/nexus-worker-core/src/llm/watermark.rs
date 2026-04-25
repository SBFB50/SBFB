// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 27 Phase B — output watermark PRF and logit bias.
//!
//! SynthID-inspired green/red token partition via HMAC-SHA256 PRF.
//! The same PRF runs coordinator-side in Python for z-test detection
//! (`packages/nexus-coordinator/src/nexus_coordinator/watermark_detector.py`).
//!
//! Not feature-gated: the PRF is pure crypto with no llama.cpp dep.
//! Only the integration into the sampling loop (`llama_cpp.rs`)
//! requires the `llm_llama_cpp` feature.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute the PRF score for a single token given a context window.
///
/// Returns a value in `[0.0, 1.0)`. Tokens with score >= 0.5 are
/// classified as "green" (favored by the watermark bias).
///
/// The input to HMAC is `context_bytes || token_bytes` where each
/// token ID is serialized as 4 bytes little-endian. This matches
/// the Python detector's `_prf_score` exactly.
pub fn prf_score(secret: &[u8], context: &[u32], token_id: u32) -> f64 {
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts any key length");
    for &ctx_tok in context {
        mac.update(&ctx_tok.to_le_bytes());
    }
    mac.update(&token_id.to_le_bytes());
    let result = mac.finalize().into_bytes();
    let top8 = u64::from_be_bytes(result[..8].try_into().unwrap());
    top8 as f64 / u64::MAX as f64
}

/// Compute watermark logit bias for a vocabulary slice.
///
/// For each token in `0..vocab_size`, adds `+delta` to the logit
/// if the token is green (PRF score >= 0.5). Returns a Vec of
/// bias values (0.0 for red tokens, `delta` for green tokens).
///
/// `context` is the last `window_size` output token IDs. If fewer
/// tokens have been generated, the context is shorter (the PRF
/// still produces a deterministic score).
pub fn compute_bias(secret: &[u8], context: &[u32], vocab_size: u32, delta: f32) -> Vec<f32> {
    let mut bias = vec![0.0f32; vocab_size as usize];
    for token_id in 0..vocab_size {
        if prf_score(secret, context, token_id) >= 0.5 {
            bias[token_id as usize] = delta;
        }
    }
    bias
}

/// Check whether watermark injection should activate for a given
/// task. Returns `true` only when the worker config has watermark
/// enabled AND the task carries a non-empty seed.
pub fn should_inject(config_enabled: bool, seed: &[u8]) -> bool {
    config_enabled && !seed.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prf_determinism() {
        let secret = b"test-secret-32-bytes-exactly!1234";
        let context = [100u32, 200, 300, 400];
        let token = 42u32;
        let s1 = prf_score(secret, &context, token);
        let s2 = prf_score(secret, &context, token);
        assert!(
            (s1 - s2).abs() < f64::EPSILON,
            "same input must produce identical score"
        );
        assert!((0.0..1.0).contains(&s1), "score must be in [0, 1)");
    }

    #[test]
    fn prf_different_tokens_differ() {
        let secret = b"test-secret-32-bytes-exactly!1234";
        let context = [1u32, 2, 3, 4];
        let s1 = prf_score(secret, &context, 10);
        let s2 = prf_score(secret, &context, 11);
        assert_ne!(
            s1.to_bits(),
            s2.to_bits(),
            "different tokens should (almost certainly) produce different scores"
        );
    }

    #[test]
    fn compute_bias_applies_delta_to_green_only() {
        let secret = b"watermark-test-key-for-bias-test!";
        let context = [5u32, 10, 15, 20];
        let delta = 2.0f32;
        let vocab = 100u32;
        let bias = compute_bias(secret, &context, vocab, delta);

        assert_eq!(bias.len(), vocab as usize);
        let green_count = bias.iter().filter(|&&b| b > 0.0).count();
        let red_count = bias.iter().filter(|&&b| b == 0.0).count();
        assert!(green_count > 0, "at least some tokens should be green");
        assert!(red_count > 0, "at least some tokens should be red");
        assert_eq!(green_count + red_count, vocab as usize);
        for &b in &bias {
            assert!(b == 0.0 || b == delta);
        }
    }

    #[test]
    fn should_inject_requires_both() {
        assert!(!should_inject(false, &[]));
        assert!(!should_inject(true, &[]));
        assert!(!should_inject(false, b"seed"));
        assert!(should_inject(true, b"seed"));
    }
}
