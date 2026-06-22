// SPDX-License-Identifier: AGPL-3.0-or-later
//! N1 spot-check selection + incentive gate (Sprint 77 Phase H).
//!
//! This module upgrades the Sprint 40 `RerunSampler` — whose
//! `simple_hash(task_id)` selector was *publicly predictable* (a worker could
//! compute `BLAKE3(task_id)` and know ex-ante whether it would be re-run, so it
//! cheated only when unwatched) — to the N1 **verifiable draw** lottery
//! ([`nexus_core_rs::verifiable_draw`]): unpredictable to the verified worker,
//! deterministic, and publicly re-checkable (one-honest-verifier).
//!
//! It also replaces the old `DivergenceScorer` byte-equality with the TOLERANT
//! TOPLOC comparator ([`nexus_core_rs::toploc::ToplocFingerprint::compare`]),
//! because honest GPU non-determinism makes byte-equality a false-reject
//! cross-hardware (see PATTERNS §P60.2 / §P64).
//!
//! ## Incentive — reputational, never monetary (PO-12, Day 0 #7)
//!
//! A drawn verifier that actually performs the spot-check earns **kudos
//! reputation** (non-monetary, non-transferable) via the existing
//! [`crate::kudos_ledger::credit`] — there is no `curator` module; the kudos
//! ledger IS the reputation mechanism. Crediting is gated by
//! [`spotcheck_creditable`]: the verifier must have been genuinely VRF-drawn
//! AND have produced a valid SIGNED run-proof of the re-execution. A
//! self-declared "I checked it" never credits.
//!
//! **Honesty (THREAT_MODEL §16):** this is a *mitigation*, not a guarantee. A
//! rational lazy verifier can simply not get drawn often / not verify; the
//! sanction for a false or lazy verifier is strictly non-economic (no credit /
//! negative trust delta on the prover path) — **never** a stake/bond/slash
//! (VeriLLM derives its game-theoretic defense from slashing, which PO-12
//! forbids here). There is therefore no anti-lazy-verifier defense in S77; the
//! cryptographic guarantee (N4 zkML) stays out of scope.

use nexus_core_rs::toploc::ToplocFingerprint;
use nexus_core_rs::{KeyPair, RunProofEntry, VrfDraw, vrf_draw, vrf_is_selected, vrf_verify};

/// Minimum percentage of output tokens an honest spot-check re-run must
/// reproduce under the shared VRF-derived seed. DiFR (arXiv:2511.20621)
/// reports >98% exact token match under a fixed sampling seed; we require 95%
/// to absorb the ~1% non-deterministic tail. Comparing tokens (Token-DiFR) in
/// ADDITION to the activation fingerprint closes the "forge tokens then
/// back-compute a matching fingerprint" evasion (Activation-DiFR alone does not
/// verify the sampling).
pub const TOKEN_AGREEMENT_PCT: u32 = 95;

/// A verifier's self-test: am I drawn as an N1 spot-check verifier for this
/// `seed` at `rate_bp` (basis points, [`VRF_RATE_DENOMINATOR`] = 100%)? Returns
/// the draw (with its publishable proof) iff selected, so a peer can re-verify
/// with [`verify_spotcheck_selection`].
///
/// `seed` MUST be data the *verified* worker cannot choose — the convention is
/// `session_id || epoch || result_commitment`, all already signed — otherwise a
/// worker grinds the seed to steer who audits it (THREAT_MODEL §16).
#[must_use]
pub fn draw_spotcheck(verifier: &KeyPair, seed: &[u8], rate_bp: u32) -> Option<VrfDraw> {
    let draw = vrf_draw(verifier, seed);
    if vrf_is_selected(&draw.output, rate_bp) {
        Some(draw)
    } else {
        None
    }
}

/// Re-verify, from a published proof, that `verifier_pubkey` was legitimately
/// drawn at `rate_bp` for `seed`. Returns `false` on an invalid proof OR a
/// proof that verifies but does not clear the selection threshold.
#[must_use]
pub fn verify_spotcheck_selection(
    verifier_pubkey: &[u8; 32],
    seed: &[u8],
    proof: &[u8; 64],
    rate_bp: u32,
) -> bool {
    match vrf_verify(verifier_pubkey, seed, proof) {
        Ok(output) => vrf_is_selected(&output, rate_bp),
        Err(_) => false,
    }
}

/// Whether an N1 spot-check is creditable to its verifier (reputational, never
/// monetary). Creditable iff ALL hold:
///
/// 1. the verifier was genuinely VRF-drawn for `seed` at `rate_bp` (not
///    self-appointed) — re-checked from the published `proof`;
/// 2. the `verifier_run_proof` is a VALID signed run-proof (proof the
///    re-execution was actually done — never a bare self-declaration);
/// 3. that run-proof is FROM this verifier (`worker_pubkey == verifier_pubkey`).
///
/// The tolerant COMPARE outcome ([`spotcheck_prover_honest`]) decides the
/// *prover's* trust; it does NOT gate the *verifier's* credit (an honest
/// verifier that correctly reports a mismatch still did the work).
#[must_use]
pub fn spotcheck_creditable(
    verifier_pubkey: &[u8; 32],
    seed: &[u8],
    proof: &[u8; 64],
    rate_bp: u32,
    verifier_run_proof: &RunProofEntry,
) -> bool {
    verify_spotcheck_selection(verifier_pubkey, seed, proof, rate_bp)
        && verifier_run_proof.proof.worker_pubkey == *verifier_pubkey
        && verifier_run_proof.verify_signature().is_ok()
}

/// Tolerant N1 verdict on the PROVER's activation fingerprint: the verifier's
/// re-extracted fingerprint (`replay`) against the prover's claimed one, via the
/// TOLERANT TOPLOC comparator — NOT byte-equality (GPU non-determinism). This
/// replaces the Sprint 40 `DivergenceScorer` byte-equality.
#[must_use]
pub fn spotcheck_activation_ok(prover: &ToplocFingerprint, replay: &ToplocFingerprint) -> bool {
    prover.compare(replay).accepted
}

/// Token-DiFR: `(matching, total)` output-token agreement between prover and
/// verifier under the shared VRF-derived seed. `total` is the longer of the two
/// sequences, so a length mismatch counts the surplus as disagreement. Counts
/// are `u64` (lossless from `usize`) so no token-count truncation is possible.
#[must_use]
pub fn token_agreement(prover_tokens: &[u32], replay_tokens: &[u32]) -> (u64, u64) {
    let total = prover_tokens.len().max(replay_tokens.len()) as u64;
    let matching = prover_tokens
        .iter()
        .zip(replay_tokens.iter())
        .filter(|(a, b)| a == b)
        .count() as u64;
    (matching, total)
}

/// Whether prover/verifier output tokens agree to at least `pct` percent. Two
/// empty sequences vacuously agree. All-integer: `matching*100 >= pct*total`.
#[must_use]
pub fn tokens_agree(prover_tokens: &[u32], replay_tokens: &[u32], pct: u32) -> bool {
    let (matching, total) = token_agreement(prover_tokens, replay_tokens);
    total == 0 || matching * 100 >= u64::from(pct) * total
}

/// Full N1 verdict that the PROVER ran honestly: the activation fingerprint
/// passes the tolerant compare AND the output tokens agree under the shared
/// seed. A worker that forges tokens then back-computes a matching fingerprint
/// fails the token half (the Activation-DiFR-only evasion).
#[must_use]
pub fn spotcheck_prover_honest(
    prover_fp: &ToplocFingerprint,
    replay_fp: &ToplocFingerprint,
    prover_tokens: &[u32],
    replay_tokens: &[u32],
) -> bool {
    spotcheck_activation_ok(prover_fp, replay_fp)
        && tokens_agree(prover_tokens, replay_tokens, TOKEN_AGREEMENT_PCT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::CoordinatorDb;
    use crate::kudos_ledger;
    use nexus_core_rs::{RunMetrics, RunProof, VRF_RATE_DENOMINATOR};

    fn signed_run_proof(verifier: &KeyPair, session: &str) -> RunProofEntry {
        let proof = RunProof::new(
            verifier.public_bytes(),
            session,
            [0u8; 32],
            [0u8; 32],
            RunMetrics {
                ttft_ms: 10,
                decode_milli_tokens_per_sec: 2_300,
                p95_token_latency_ms: 20,
                network_rx_bytes: 0,
                network_tx_bytes: 0,
                worker_drop_count: 0,
            },
            vec![],
        );
        RunProofEntry::sign(proof, verifier).expect("sign run proof")
    }

    #[test]
    fn draw_spotcheck_selects_at_full_rate_never_at_zero() {
        let kp = KeyPair::generate();
        let seed = b"session-1|epoch-1|commit-1";
        assert!(
            draw_spotcheck(&kp, seed, VRF_RATE_DENOMINATOR).is_some(),
            "rate 100% always draws"
        );
        assert!(draw_spotcheck(&kp, seed, 0).is_none(), "rate 0 never draws");
    }

    #[test]
    fn verify_spotcheck_selection_round_trips_and_rejects_tampering() {
        let kp = KeyPair::generate();
        let seed = b"session-2|epoch-5|commit-xyz";
        let draw = draw_spotcheck(&kp, seed, VRF_RATE_DENOMINATOR).expect("drawn at 100%");
        assert!(verify_spotcheck_selection(
            &kp.public_bytes(),
            seed,
            &draw.proof,
            VRF_RATE_DENOMINATOR
        ));
        // Wrong key, tampered seed, and rate 0 all fail.
        let other = KeyPair::generate();
        assert!(!verify_spotcheck_selection(
            &other.public_bytes(),
            seed,
            &draw.proof,
            VRF_RATE_DENOMINATOR
        ));
        assert!(!verify_spotcheck_selection(
            &kp.public_bytes(),
            b"other-seed",
            &draw.proof,
            VRF_RATE_DENOMINATOR
        ));
        assert!(!verify_spotcheck_selection(
            &kp.public_bytes(),
            seed,
            &draw.proof,
            0
        ));
    }

    #[test]
    fn incentive_credits_reputation_on_honest_spotcheck() {
        let db = CoordinatorDb::open_in_memory().expect("db");
        let verifier = KeyPair::generate();
        let seed = b"session-9|epoch-2|commit-abc";
        // rate 100% → deterministically selected for the test.
        let draw = draw_spotcheck(&verifier, seed, VRF_RATE_DENOMINATOR).expect("selected");
        let run_proof = signed_run_proof(&verifier, "session-9");

        // A genuine, proven spot-check is creditable.
        assert!(spotcheck_creditable(
            &verifier.public_bytes(),
            seed,
            &draw.proof,
            VRF_RATE_DENOMINATOR,
            &run_proof,
        ));

        // Crediting reuses the EXISTING non-monetary kudos ledger.
        let worker_id = hex::encode(verifier.public_bytes());
        kudos_ledger::credit(&db, "proj-spotcheck", &worker_id, "spotcheck-1", 64, 1_000)
            .expect("credit");

        // Non-monetary (PO-12): reputation ACCRUES from a zero prior balance —
        // no stake/deposit/burn is required or deducted. The verifier started at
        // 0 kudos and now holds exactly one positive, non-transferable line; no
        // balance is spent and `credit` takes no monetary argument.
        assert!(
            db.get_project_kudos_total("proj-spotcheck").expect("total") > 0,
            "an honest spot-check credits reputation"
        );
        let entries = db
            .get_project_entries("proj-spotcheck")
            .expect("ledger entries");
        assert_eq!(
            entries.len(),
            1,
            "exactly one reputation line — nothing is spent or deducted"
        );
        assert!(
            entries[0].amount > 0,
            "reputation is credited (positive), never debited"
        );
    }

    #[test]
    fn lazy_or_unselected_verifier_is_not_creditable() {
        let verifier = KeyPair::generate();
        let seed = b"session-3|epoch-1|commit-1";
        let draw = draw_spotcheck(&verifier, seed, VRF_RATE_DENOMINATOR).expect("selected");
        let run_proof = signed_run_proof(&verifier, "session-3");

        // Not drawn (rate 0) → not creditable even with a valid run-proof.
        assert!(!spotcheck_creditable(
            &verifier.public_bytes(),
            seed,
            &draw.proof,
            0,
            &run_proof,
        ));

        // Drawn, but the run-proof is from a DIFFERENT worker → not creditable
        // (cannot claim someone else's work).
        let impostor = KeyPair::generate();
        let impostor_proof = signed_run_proof(&impostor, "session-3");
        assert!(!spotcheck_creditable(
            &verifier.public_bytes(),
            seed,
            &draw.proof,
            VRF_RATE_DENOMINATOR,
            &impostor_proof,
        ));
    }

    #[test]
    fn spotcheck_activation_uses_tolerant_compare_not_byte_equality() {
        // Same indices/exponents, mantissa off by ~2 bf16 ULPs (the values are
        // exactly bf16-representable so the truncation is deterministic): the
        // tolerant compare ACCEPTS, and the commitments DIFFER, proving this is
        // not byte-equality.
        let prover = ToplocFingerprint::from_topk(&[(1, 100.0), (3, 200.0), (5, 50.0)]);
        let replay_close = ToplocFingerprint::from_topk(&[(1, 101.0), (3, 202.0), (5, 50.5)]);
        assert_ne!(
            prover.commitment(),
            replay_close.commitment(),
            "the fixtures must differ byte-wise (else the test is vacuous)"
        );
        assert!(
            spotcheck_activation_ok(&prover, &replay_close),
            "honest FP noise must pass the tolerant compare"
        );
        // A different model → disjoint top-k → rejected.
        let replay_swap = ToplocFingerprint::from_topk(&[(40, 9.0), (41, 8.0), (42, 7.0)]);
        assert!(
            !spotcheck_activation_ok(&prover, &replay_swap),
            "a model swap must be rejected"
        );
    }

    #[test]
    fn token_difr_catches_forged_tokens() {
        // Honest re-run: tokens match → agree.
        let prover = [10u32, 20, 30, 40, 50];
        let same = [10u32, 20, 30, 40, 50];
        assert!(tokens_agree(&prover, &same, TOKEN_AGREEMENT_PCT));

        // Forged sampling: most tokens differ → fails the 95% bar even though
        // an attacker could have back-computed a matching fingerprint.
        let forged = [10u32, 99, 98, 97, 96];
        assert!(!tokens_agree(&prover, &forged, TOKEN_AGREEMENT_PCT));

        // Length mismatch counts the surplus as disagreement — in BOTH
        // directions (the verdict is symmetric: total = the longer side).
        let truncated = [10u32, 20];
        assert!(!tokens_agree(&prover, &truncated, TOKEN_AGREEMENT_PCT));
        assert!(!tokens_agree(&truncated, &prover, TOKEN_AGREEMENT_PCT)); // replay longer

        // One side empty, the other non-empty → 0/N agreement → fails (both
        // orderings), distinct from the both-empty vacuous case below.
        assert!(!tokens_agree(&[], &[1, 2], TOKEN_AGREEMENT_PCT));
        assert!(!tokens_agree(&[1, 2], &[], TOKEN_AGREEMENT_PCT));

        // Two empty sequences vacuously agree (no tokens to disprove).
        assert!(tokens_agree(&[], &[], TOKEN_AGREEMENT_PCT));
    }

    #[test]
    fn token_agreement_counts_matching_and_total() {
        let (m, t) = token_agreement(&[1, 2, 3, 4], &[1, 2, 9, 4]);
        assert_eq!((m, t), (3, 4), "3 of 4 positions match");
        let (m, t) = token_agreement(&[1, 2, 3], &[1, 2, 3, 4, 5]);
        assert_eq!((m, t), (3, 5), "total is the longer sequence");
    }

    #[test]
    fn spotcheck_prover_honest_requires_both_activation_and_tokens() {
        let prover_fp = ToplocFingerprint::from_topk(&[(1, 100.0), (3, 200.0), (5, 50.0)]);
        let close_fp = ToplocFingerprint::from_topk(&[(1, 101.0), (3, 202.0), (5, 50.5)]);
        let prover_tok = [1u32, 2, 3, 4, 5];

        // Activation OK + tokens OK → honest.
        assert!(spotcheck_prover_honest(
            &prover_fp,
            &close_fp,
            &prover_tok,
            &[1, 2, 3, 4, 5]
        ));
        // Activation OK but tokens forged → NOT honest (Token-DiFR catches it).
        assert!(!spotcheck_prover_honest(
            &prover_fp,
            &close_fp,
            &prover_tok,
            &[9, 9, 9, 9, 9]
        ));
        // Tokens OK but activation swapped → NOT honest.
        let swap_fp = ToplocFingerprint::from_topk(&[(40, 9.0), (41, 8.0)]);
        assert!(!spotcheck_prover_honest(
            &prover_fp,
            &swap_fp,
            &prover_tok,
            &[1, 2, 3, 4, 5]
        ));
    }
}
