// SPDX-License-Identifier: AGPL-3.0-or-later
//! Watermark z-test detector — SynthID-inspired PRF verification
//! (Sprint 40 Phase C, port of watermark_detector.py S27).
//!
//! Coordinator-side complement to the worker-side injector at
//! `crates/nexus-worker-core/src/llm/watermark.rs`. Both share the
//! same HMAC-SHA256 PRF so the detector can verify whether output
//! tokens carry watermark bias.

use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct WatermarkResult {
    pub is_watermarked: bool,
    pub z_score: f64,
    pub green_ratio: f64,
    pub token_count: usize,
}

pub struct WatermarkDetector {
    secret: Vec<u8>,
    window: usize,
    threshold: f64,
}

impl WatermarkDetector {
    pub fn new(secret: &[u8], window_size: usize, threshold_z: f64) -> Self {
        Self {
            secret: secret.to_vec(),
            window: window_size,
            threshold: threshold_z,
        }
    }

    pub fn detect(&self, token_ids: &[u32]) -> WatermarkResult {
        let n = token_ids.len();
        if n <= self.window {
            return WatermarkResult {
                is_watermarked: false,
                z_score: 0.0,
                green_ratio: 0.0,
                token_count: n,
            };
        }

        let mut green = 0usize;
        let mut scored = 0usize;
        for i in self.window..n {
            let start = i.saturating_sub(self.window);
            let context = &token_ids[start..i];
            let score = prf_score(&self.secret, context, token_ids[i]);
            if score >= 0.5 {
                green += 1;
            }
            scored += 1;
        }

        if scored == 0 {
            return WatermarkResult {
                is_watermarked: false,
                z_score: 0.0,
                green_ratio: 0.0,
                token_count: n,
            };
        }

        let green_ratio = green as f64 / scored as f64;
        let std_dev = (0.25 / scored as f64).sqrt();
        let z_score = (green_ratio - 0.5) / std_dev;

        WatermarkResult {
            is_watermarked: z_score >= self.threshold,
            z_score,
            green_ratio,
            token_count: n,
        }
    }
}

fn prf_score(secret: &[u8], context: &[u32], token_id: u32) -> f64 {
    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts any key length");
    for &ctx_tok in context {
        mac.update(&ctx_tok.to_le_bytes());
    }
    mac.update(&token_id.to_le_bytes());
    let result = mac.finalize().into_bytes();
    let top8 = u64::from_be_bytes(result[..8].try_into().unwrap());
    top8 as f64 / u64::MAX as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_not_watermarked_random() {
        let det = WatermarkDetector::new(b"secret-key", 4, 2.0);
        let tokens: Vec<u32> = (0..100).collect();
        let result = det.detect(&tokens);
        assert!(!result.is_watermarked || result.z_score < 4.0);
        assert_eq!(result.token_count, 100);
    }

    #[test]
    fn detect_too_few_tokens() {
        let det = WatermarkDetector::new(b"key", 4, 2.0);
        let result = det.detect(&[1, 2, 3]);
        assert!(!result.is_watermarked);
        assert_eq!(result.token_count, 3);
    }

    #[test]
    fn prf_score_deterministic() {
        let s1 = prf_score(b"test-secret", &[1, 2, 3, 4], 42);
        let s2 = prf_score(b"test-secret", &[1, 2, 3, 4], 42);
        assert!((s1 - s2).abs() < f64::EPSILON);
        assert!((0.0..1.0).contains(&s1));
    }

    #[test]
    fn prf_score_different_tokens_differ() {
        let s1 = prf_score(b"test-secret", &[1, 2, 3, 4], 10);
        let s2 = prf_score(b"test-secret", &[1, 2, 3, 4], 11);
        assert_ne!(s1.to_bits(), s2.to_bits());
    }
}
