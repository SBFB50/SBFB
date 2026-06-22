// SPDX-License-Identifier: AGPL-3.0-or-later
//! 3-layer proof-of-computation verification.
//!
//! Direct port of `nexus/compute/verification.py` to Rust, with
//! the signature layer delegating to [`ResultEntry::verify_signature`]
//! (which uses the canonical bytes format from [`crate::task`]).
//!
//! ## Layer 1 — Ed25519 signature (WHO produced it)
//!
//! Proves identity via cryptographic non-repudiation. Failure here
//! is treated as critical: trust -50 AND auto-ban, because a bad
//! signature means either the key was stolen or someone is actively
//! trying to forge results. ~0.1ms.
//!
//! ## Layer 2 — Model digest whitelist (WHICH model ran)
//!
//! A per-model BLAKE3 digest compared against a whitelist of
//! approved digests per model name. An empty whitelist is treated as
//! "no check configured" (the layer passes with a `NoWhitelist`
//! reason). A digest mismatch is treated as critical: -50 trust +
//! auto-ban.
//!
//! Sprint 76 Phase C doc-note (D3 etage 1): the worker currently
//! reports BLAKE3 of the model NAME string, NOT the weights file —
//! the Ollama backend exposes no clean file digest. A real
//! weights-file digest is gated on a file-exposing backend
//! (`llm_llama_cpp`, Sprint 77). This `Verifier` is also not wired
//! into the live result path today (it has no production caller); the
//! live path is the hash-exact quorum over `result_text`
//! (`validate_quorum_pre_guardrail`). Callers must not treat the
//! digest as a weights attestation until S77.
//!
//! ## Layer 3 — N0 TOPLOC fingerprint (DID the model actually run)
//!
//! Sprint 77 Phase G upgrades this layer from an inert logprob hash
//! to the real **N0 TOPLOC commitment** ([`crate::toploc`]). The
//! worker fingerprints the top-k of its last hidden state and
//! reports the 32-byte BLAKE3 commitment of the canonical
//! all-integer encoding in `logprobs_hash`; we compare it for
//! equality against a registered reference. A mismatch is NOT a
//! critical failure — it lowers trust by 5 (suspect) but does not
//! ban, so the next dispatch can run a spot-check.
//!
//! The layer still compares by hash **equality**: a BLAKE3
//! commitment binds a worker to one fingerprint, but a hash
//! destroys locality, so it can only detect a swap by inequality —
//! it is NOT the tolerant comparator. The tolerant exponent/mantissa
//! comparison ([`crate::toploc::ToplocFingerprint::compare`]) needs
//! the full sketch on both sides; running it in-vivo cross-worker is
//! the N1 spot-check (Phase H) and N2 tolerant redundancy (Phase I),
//! i.e. the "separate off-canonical payload" this note previously
//! deferred to a later release. The wire field keeps the name
//! `logprobs_hash` (0 bump wire).
//!
//! **Auto-attestation caveat:** a commitment a worker reports for
//! its own run is a self-claim, not proof of correct computation,
//! until an independent verifier (N1/N2, Phase H/I) recomputes the
//! fingerprint. Treat it like [`crate::task::ResultPayload::model_digest`].

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::crypto::BLAKE3_BYTES;
use crate::task::{ResultEntry, TaskEntry};

/// Status of a single verification check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    /// The check passed.
    Passed,
    /// The check was skipped (no data to check against).
    Skipped,
    /// The check failed.
    Failed,
}

/// Result of a single verification layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerResult {
    /// Pass / skip / fail outcome.
    pub status: CheckStatus,
    /// Human-readable reason string, for logging and debugging.
    pub reason: String,
}

impl LayerResult {
    fn passed(reason: impl Into<String>) -> Self {
        LayerResult {
            status: CheckStatus::Passed,
            reason: reason.into(),
        }
    }
    fn skipped(reason: impl Into<String>) -> Self {
        LayerResult {
            status: CheckStatus::Skipped,
            reason: reason.into(),
        }
    }
    fn failed(reason: impl Into<String>) -> Self {
        LayerResult {
            status: CheckStatus::Failed,
            reason: reason.into(),
        }
    }
}

/// Combined report from running all 3 verification layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    /// Did the result pass overall? True iff signature and digest
    /// both passed and logprobs did not definitively fail.
    pub passed: bool,

    /// Layer 1 outcome.
    pub signature: LayerResult,
    /// Layer 2 outcome.
    pub digest: LayerResult,
    /// Layer 3 outcome.
    pub logprobs: LayerResult,

    /// Trust score delta to apply to the worker on this result.
    /// +1 for accepted, -5 for suspect (logprob mismatch),
    /// -50 for critical failure (signature or digest).
    pub trust_delta: i32,

    /// If true, the worker should be auto-banned immediately.
    /// Set on critical failures (signature or digest mismatch).
    pub ban: bool,
}

/// Runs the 3-layer verification stack.
///
/// Construct one and hold it for the life of the coordinator
/// process. `register_digest()` and `register_logprob_profile()`
/// populate the layer-2 and layer-3 references respectively.
#[derive(Debug, Default)]
pub struct Verifier {
    /// Maps model name -> whitelisted BLAKE3 model digest. Today the
    /// worker reports a hash of the model NAME, not the weights file
    /// (S76 Phase C doc-note; a real weights digest is gated on
    /// `llm_llama_cpp`, S77).
    digest_whitelist: HashMap<String, [u8; BLAKE3_BYTES]>,
    /// Maps (model, calibration_prompt_id) -> reference logprobs hash.
    logprob_profiles: HashMap<String, HashMap<String, [u8; BLAKE3_BYTES]>>,
}

impl Verifier {
    /// Create a fresh verifier with empty whitelists.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a known-good model digest.
    ///
    /// Subsequent verifications will require results for this model
    /// name to carry a matching digest.
    pub fn register_digest(&mut self, model: impl Into<String>, digest: [u8; BLAKE3_BYTES]) {
        self.digest_whitelist.insert(model.into(), digest);
    }

    /// Register a reference logprob fingerprint hash for a
    /// (model, calibration_prompt_id) pair.
    pub fn register_logprob_profile(
        &mut self,
        model: impl Into<String>,
        calibration_prompt_id: impl Into<String>,
        expected_hash: [u8; BLAKE3_BYTES],
    ) {
        self.logprob_profiles
            .entry(model.into())
            .or_default()
            .insert(calibration_prompt_id.into(), expected_hash);
    }

    /// Return whether the digest whitelist has any entries.
    pub fn has_digest_whitelist(&self) -> bool {
        !self.digest_whitelist.is_empty()
    }

    /// Run all 3 verification layers on a `(task, result)` pair.
    ///
    /// `calibration_prompt_id` is used only for layer 3 lookup.
    /// Pass an empty string if the task was not sent with a
    /// calibration prompt (layer 3 will be marked skipped).
    pub fn verify(
        &self,
        task: &TaskEntry,
        result: &ResultEntry,
        calibration_prompt_id: &str,
    ) -> VerificationReport {
        // --- Layer 1: signature ---

        let signature = match result.verify_signature() {
            Ok(()) => LayerResult::passed("valid"),
            Err(e) => {
                return VerificationReport {
                    passed: false,
                    signature: LayerResult::failed(format!("invalid: {e}")),
                    digest: LayerResult::skipped("signature layer failed first"),
                    logprobs: LayerResult::skipped("signature layer failed first"),
                    trust_delta: -50,
                    ban: true,
                };
            }
        };

        // --- Layer 2: model digest ---

        let model = task.task.model.as_str();
        let reported = result.payload.model_digest;

        let digest = if !self.has_digest_whitelist() {
            LayerResult::skipped("no whitelist configured")
        } else if let Some(expected) = self.digest_whitelist.get(model) {
            if *expected == reported {
                LayerResult::passed("digest match")
            } else {
                return VerificationReport {
                    passed: false,
                    signature,
                    digest: LayerResult::failed("digest mismatch"),
                    logprobs: LayerResult::skipped("digest layer failed"),
                    trust_delta: -50,
                    ban: true,
                };
            }
        } else {
            LayerResult::passed("model not in whitelist, allowed")
        };

        // --- Layer 3: logprob fingerprint ---

        let reported_lp = result.payload.logprobs_hash;
        let zero_hash = [0u8; BLAKE3_BYTES];

        let (logprobs, trust_delta) = if calibration_prompt_id.is_empty() {
            (LayerResult::skipped("no calibration prompt"), 1)
        } else if reported_lp == zero_hash {
            (LayerResult::skipped("worker did not report logprobs"), 1)
        } else {
            match self.logprob_profiles.get(model) {
                None => (LayerResult::skipped("model not profiled"), 1),
                Some(by_prompt) => match by_prompt.get(calibration_prompt_id) {
                    None => (LayerResult::skipped("prompt not profiled"), 1),
                    Some(expected) if *expected == reported_lp => {
                        (LayerResult::passed("logprob hash match"), 1)
                    }
                    Some(_) => (LayerResult::failed("logprob hash mismatch"), -5),
                },
            }
        };

        let passed_overall = matches!(logprobs.status, CheckStatus::Passed | CheckStatus::Skipped);

        VerificationReport {
            passed: passed_overall,
            signature,
            digest,
            logprobs,
            trust_delta,
            ban: false,
        }
    }
}

/// Compute the spot-check probability for a worker given their
/// current trust score. Mirrors the Python
/// `ResultVerifier.spot_check_needed` rate table exactly:
///
/// - trust >= 80  → 1% (trusted)
/// - trust >= 50  → 5% (standard)
/// - otherwise    → 20% (suspect)
pub fn spot_check_rate(trust_score: i32) -> f64 {
    if trust_score >= 80 {
        0.01
    } else if trust_score >= 50 {
        0.05
    } else {
        0.20
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::task::{ResultPayload, TASK_FORMAT_VERSION, Task, TaskEntry};

    fn sample_task(model: &str) -> TaskEntry {
        let kp = KeyPair::generate();
        TaskEntry::sign(
            Task::new("task-1", "analysis", "Q", model, 5, 1_712_345_678),
            &kp,
        )
        .unwrap()
    }

    fn sample_result(model_digest: [u8; 32], logprobs_hash: [u8; 32]) -> (KeyPair, ResultEntry) {
        let kp = KeyPair::generate();
        let payload = ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: "task-1".into(),
            result_text: "Paris.".into(),
            tokens_generated: 2,
            generation_time_ms: 100,
            model_digest,
            logprobs_hash,
            started_at: 1_712_345_680,
            finished_at: 1_712_345_680,
            output_token_ids: vec![],
        };
        let entry = ResultEntry::sign(payload, &kp).unwrap();
        (kp, entry)
    }

    #[test]
    fn empty_verifier_accepts_valid_signature() {
        let task = sample_task("llama-3.1-8b");
        let (_, result) = sample_result([0u8; 32], [0u8; 32]);

        let v = Verifier::new();
        let report = v.verify(&task, &result, "");

        assert!(report.passed);
        assert_eq!(report.signature.status, CheckStatus::Passed);
        assert_eq!(report.digest.status, CheckStatus::Skipped);
        assert_eq!(report.logprobs.status, CheckStatus::Skipped);
        assert_eq!(report.trust_delta, 1);
        assert!(!report.ban);
    }

    #[test]
    fn tampered_signature_causes_ban() {
        let task = sample_task("llama-3.1-8b");
        let (_, mut result) = sample_result([0u8; 32], [0u8; 32]);
        result.payload.result_text = "Berlin.".into();

        let v = Verifier::new();
        let report = v.verify(&task, &result, "");

        assert!(!report.passed);
        assert_eq!(report.signature.status, CheckStatus::Failed);
        assert_eq!(report.trust_delta, -50);
        assert!(report.ban);
    }

    #[test]
    fn digest_whitelist_match_passes() {
        let task = sample_task("llama-3.1-8b");
        let good_digest = [0xAA; 32];
        let (_, result) = sample_result(good_digest, [0u8; 32]);

        let mut v = Verifier::new();
        v.register_digest("llama-3.1-8b", good_digest);

        let report = v.verify(&task, &result, "");
        assert!(report.passed);
        assert_eq!(report.digest.status, CheckStatus::Passed);
    }

    #[test]
    fn digest_whitelist_mismatch_causes_ban() {
        let task = sample_task("llama-3.1-8b");
        let wrong_digest = [0xFF; 32];
        let (_, result) = sample_result(wrong_digest, [0u8; 32]);

        let mut v = Verifier::new();
        v.register_digest("llama-3.1-8b", [0xAA; 32]);

        let report = v.verify(&task, &result, "");
        assert!(!report.passed);
        assert_eq!(report.digest.status, CheckStatus::Failed);
        assert_eq!(report.trust_delta, -50);
        assert!(report.ban);
    }

    #[test]
    fn unprofiled_model_passes_digest() {
        // If the whitelist has entries but not for this model,
        // the layer passes ("not in whitelist, allowed") rather
        // than ban — matches the Python behavior.
        let task = sample_task("unknown-model");
        let (_, result) = sample_result([0x55; 32], [0u8; 32]);

        let mut v = Verifier::new();
        v.register_digest("some-other-model", [0xAA; 32]);

        let report = v.verify(&task, &result, "");
        assert!(report.passed);
        assert_eq!(report.digest.status, CheckStatus::Passed);
    }

    #[test]
    fn toploc_commitment_match_passes() {
        // Phase G: the layer-3 reference is now a real N0 TOPLOC commitment
        // (BLAKE3 of the canonical fingerprint), compared by equality.
        let task = sample_task("llama-3.1-8b");
        let commitment =
            crate::toploc::ToplocFingerprint::from_topk(&[(3, 12.0), (7, -8.0), (1, 5.0)])
                .commitment();
        let (_, result) = sample_result([0u8; 32], commitment);

        let mut v = Verifier::new();
        v.register_logprob_profile("llama-3.1-8b", "prompt-a", commitment);

        let report = v.verify(&task, &result, "prompt-a");
        assert!(report.passed);
        assert_eq!(report.logprobs.status, CheckStatus::Passed);
        assert_eq!(report.trust_delta, 1);
    }

    #[test]
    fn toploc_commitment_mismatch_lowers_trust_without_ban() {
        // A different model → different top-k → different commitment → the
        // equality check fails (suspect, not ban: the tolerant recompute is
        // the N1/N2 verifier's job, Phase H/I).
        let task = sample_task("llama-3.1-8b");
        let reported =
            crate::toploc::ToplocFingerprint::from_topk(&[(3, 12.0), (7, -8.0)]).commitment();
        let reference =
            crate::toploc::ToplocFingerprint::from_topk(&[(9, 99.0), (2, -50.0)]).commitment();
        assert_ne!(reported, reference, "fixtures must differ");
        let (_, result) = sample_result([0u8; 32], reported);

        let mut v = Verifier::new();
        v.register_logprob_profile("llama-3.1-8b", "prompt-a", reference);

        let report = v.verify(&task, &result, "prompt-a");
        // overall passed (not banworthy) but trust_delta = -5
        assert!(!report.passed);
        assert_eq!(report.logprobs.status, CheckStatus::Failed);
        assert_eq!(report.trust_delta, -5);
        assert!(!report.ban);
    }

    #[test]
    fn spot_check_rate_matches_python_tiers() {
        assert_eq!(spot_check_rate(100), 0.01);
        assert_eq!(spot_check_rate(80), 0.01); // boundary
        assert_eq!(spot_check_rate(79), 0.05);
        assert_eq!(spot_check_rate(50), 0.05); // boundary
        assert_eq!(spot_check_rate(49), 0.20);
        assert_eq!(spot_check_rate(0), 0.20);
    }
}
