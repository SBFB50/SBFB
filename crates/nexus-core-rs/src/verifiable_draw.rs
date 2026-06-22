// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase H — N1 verifiable draw (spot-check verifier selection).
//!
//! The N1 spot-check (VeriLLM arXiv:2509.24257, DiFR arXiv:2511.20621) needs to
//! pick *which* worker re-executes a ~1% prefill pass to audit a peer's N0
//! [`crate::toploc`] fingerprint, with two properties:
//!
//! - **Unpredictable to the verified worker** — a prover must not know
//!   ex-ante whether it will be spot-checked (otherwise it cheats only when
//!   unwatched). The Sprint 40 `simple_hash(task_id)` selector was the exact
//!   anti-pattern: `BLAKE3(task_id)` is publicly computable, so a worker knows
//!   in advance. This module replaces it.
//! - **Deterministic and publicly verifiable** — any peer can re-check that a
//!   self-declared verifier was *legitimately* drawn (one-honest-verifier:
//!   the audit holds as long as a single honest verifier is among those drawn).
//!
//! ## Construction (and its honesty caveat)
//!
//! A potential verifier signs the draw `seed` with its node Ed25519 key; the
//! signature is the draw **proof** and `BLAKE3(domain || proof)` is the draw
//! **output**. Selection compares the output against an integer threshold
//! proportional to the sampling rate. The `seed` MUST be data the *verified*
//! worker cannot choose — `session_id || epoch || result_commitment`, all
//! already signed — or it could grind the seed to pick a colluding verifier
//! (THREAT_MODEL §16, surface "predictibilite-tirage / grinding").
//!
//! **This is NOT an ECVRF (RFC 9381).** Ed25519 is malleable, so a third party
//! can derive a *different* valid signature for the same `(pubkey, message)`
//! without the key, which means draw *uniqueness* is not cryptographically
//! proven; and Ed25519 is not a PRF, so *unpredictability* is not proven
//! either. We deliberately reuse the existing [`crate::crypto`] Ed25519 path
//! (0 new dependency, mirroring the Phase D `blake3(session_id || pubkey)`
//! deterministic-sampling precedent) rather than pull in a heavyweight ECVRF
//! crate on a second curve. For a 1–5% spot-check under one-honest-verifier
//! this is a **mitigation, not a guarantee** — it raises the cost of evading
//! audit, it does not make evasion impossible. A formal ECVRF (and the N4 zkML
//! guarantee) stay out of S77 scope.
//!
//! ## All-integer (no-float)
//!
//! Selection and the temperature/seed derivation are all-integer: the rate is
//! basis points, the threshold comparison is `u128`, and the temperature is a
//! milli-unit `u32` the worker converts to an `f32` only at the GPU boundary
//! (exactly like [`crate::toploc::bf16_bits`] is the only float boundary in N0).

use crate::canonical::DOMAIN_VRF_DRAW_V1;
use crate::crypto::{
    BLAKE3_BYTES, KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES, blake3_hash, verify,
};
use crate::error::Result;

/// Basis-point denominator for a sampling rate. `rate_bp = 100` is 1%,
/// `rate_bp = VRF_RATE_DENOMINATOR` (10 000) is 100% ("always selected").
pub const VRF_RATE_DENOMINATOR: u32 = 10_000;

/// Exclusive upper bound on the derived spot-check temperature, in milli-units
/// (`2000` ⇒ temperature `2.0`). The verifier re-runs at this derived
/// temperature so the prover cannot precompute the sampling distribution; the
/// comparison stays tolerant ([`crate::toploc::ToplocFingerprint::compare`]),
/// never strict equality at `temperature > 0`.
pub const VRF_MAX_TEMP_MILLI: u32 = 2_000;

/// Personalisation suffix for the deterministic temperature derivation.
const VRF_TEMP_PERSO: &[u8] = b"temp";
/// Personalisation suffix for the deterministic sampling-seed derivation.
const VRF_SEED_PERSO: &[u8] = b"seed";

/// The result of a verifiable draw: the Ed25519 `proof` (publishable so any
/// peer can re-verify the selection) and the derived 32-byte `output`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VrfDraw {
    /// Ed25519 signature over `DOMAIN_VRF_DRAW_V1 || 0x00 || seed`.
    pub proof: [u8; SIGNATURE_BYTES],
    /// `BLAKE3(DOMAIN_VRF_DRAW_V1 || 0x00 || proof)` — the draw output the
    /// selection threshold is applied to.
    pub output: [u8; BLAKE3_BYTES],
}

/// The exact message that is signed for a draw: `domain || 0x00 || seed`. The
/// `0x00` separator is the same hard boundary [`crate::canonical_bytes`] uses,
/// so a crafted seed cannot smuggle the domain into its own bytes.
fn draw_message(seed: &[u8]) -> Vec<u8> {
    let mut m = Vec::with_capacity(DOMAIN_VRF_DRAW_V1.len() + 1 + seed.len());
    m.extend_from_slice(DOMAIN_VRF_DRAW_V1);
    m.push(0u8);
    m.extend_from_slice(seed);
    m
}

/// The draw output from a proof: `BLAKE3(domain || 0x00 || proof)`. Domain-
/// separated so the output cannot collide with any other BLAKE3 pre-image.
fn draw_output(proof: &[u8; SIGNATURE_BYTES]) -> [u8; BLAKE3_BYTES] {
    let mut pre = Vec::with_capacity(DOMAIN_VRF_DRAW_V1.len() + 1 + SIGNATURE_BYTES);
    pre.extend_from_slice(DOMAIN_VRF_DRAW_V1);
    pre.push(0u8);
    pre.extend_from_slice(proof);
    blake3_hash(&pre)
}

/// Compute a verifiable draw for `seed` with `signer`'s key. Deterministic:
/// the same `(key, seed)` always yields the same proof (RFC 8032) and output.
#[must_use]
pub fn vrf_draw(signer: &KeyPair, seed: &[u8]) -> VrfDraw {
    let proof = signer.sign(&draw_message(seed));
    let output = draw_output(&proof);
    VrfDraw { proof, output }
}

/// Verify a published draw proof and recompute its output.
///
/// # Errors
///
/// Returns [`crate::NexusError::Crypto`] if the Ed25519 signature is invalid
/// for `(pubkey, domain || 0x00 || seed)`.
pub fn vrf_verify(
    pubkey: &[u8; PUBLIC_KEY_LENGTH],
    seed: &[u8],
    proof: &[u8; SIGNATURE_BYTES],
) -> Result<[u8; BLAKE3_BYTES]> {
    verify(pubkey, &draw_message(seed), proof)?;
    Ok(draw_output(proof))
}

/// Whether a draw `output` selects its holder at `rate_bp` basis points.
///
/// All-integer: the first 8 bytes of the output are read as a big-endian `u64`
/// `value`, and the holder is selected iff `value / 2^64 < rate_bp / 10000`,
/// evaluated as `value * 10000 < rate_bp * 2^64` in `u128`. `rate_bp = 0` never
/// selects; `rate_bp >= VRF_RATE_DENOMINATOR` always selects.
#[must_use]
pub fn vrf_is_selected(output: &[u8; BLAKE3_BYTES], rate_bp: u32) -> bool {
    let value = u64::from_be_bytes([
        output[0], output[1], output[2], output[3], output[4], output[5], output[6], output[7],
    ]);
    (u128::from(value)) * u128::from(VRF_RATE_DENOMINATOR) < u128::from(rate_bp) * (1u128 << 64)
}

/// Deterministically derive the spot-check **temperature** (milli-units, in
/// `[0, VRF_MAX_TEMP_MILLI)`) from a draw output. Reproducible by any auditor,
/// unpredictable to the prover before the draw — the DiFR anti-evasion measure
/// (randomise temperature so a worker cannot precompute the sampling output).
#[must_use]
pub fn derive_spotcheck_temp_milli(output: &[u8; BLAKE3_BYTES]) -> u32 {
    let h = personalised(output, VRF_TEMP_PERSO);
    let raw = u32::from_be_bytes([h[0], h[1], h[2], h[3]]);
    raw % VRF_MAX_TEMP_MILLI
}

/// Deterministically derive the spot-check **sampling seed** from a draw
/// output. Shared by prover and verifier so a fixed-seed re-run reproduces the
/// token sequence (DiFR: >98% token match under a fixed seed) — the verifier
/// then compares tokens (Token-DiFR), not just the activation fingerprint.
#[must_use]
pub fn derive_spotcheck_seed(output: &[u8; BLAKE3_BYTES]) -> u64 {
    let h = personalised(output, VRF_SEED_PERSO);
    u64::from_be_bytes([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]])
}

/// `BLAKE3(output || perso)` — a domain-separated sub-derivation so the
/// temperature and seed never collide with each other or with the draw output.
fn personalised(output: &[u8; BLAKE3_BYTES], perso: &[u8]) -> [u8; BLAKE3_BYTES] {
    let mut pre = Vec::with_capacity(BLAKE3_BYTES + perso.len());
    pre.extend_from_slice(output);
    pre.extend_from_slice(perso);
    blake3_hash(&pre)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n1_vrf_selects_deterministic_verifier() {
        let kp = KeyPair::generate();
        let seed = b"session-7|epoch-3|commit-abc";

        // Deterministic: the same (key, seed) yields the same draw.
        let a = vrf_draw(&kp, seed);
        let b = vrf_draw(&kp, seed);
        assert_eq!(a, b, "draw is deterministic for a fixed (key, seed)");

        // Publicly verifiable: the proof re-derives the same output.
        let recomputed = vrf_verify(&kp.public_bytes(), seed, &a.proof).expect("valid proof");
        assert_eq!(
            recomputed, a.output,
            "vrf_verify recomputes the draw output"
        );

        // A different seed yields a different draw (no fixed pre-selection).
        let other = vrf_draw(&kp, b"session-7|epoch-4|commit-abc");
        assert_ne!(other.output, a.output, "a different seed redraws");
    }

    #[test]
    fn vrf_verify_rejects_tampered_proof_and_wrong_key() {
        let kp = KeyPair::generate();
        let seed = b"seed-bytes";
        let draw = vrf_draw(&kp, seed);

        // Wrong public key → rejected.
        let other = KeyPair::generate();
        assert!(vrf_verify(&other.public_bytes(), seed, &draw.proof).is_err());

        // Tampered seed → the signature no longer matches → rejected.
        assert!(vrf_verify(&kp.public_bytes(), b"seed-bytez", &draw.proof).is_err());

        // Flipped proof byte → rejected.
        let mut bad = draw.proof;
        bad[0] ^= 0x01;
        assert!(vrf_verify(&kp.public_bytes(), seed, &bad).is_err());
    }

    #[test]
    fn vrf_is_selected_rate_boundaries() {
        // rate 0 never selects; rate == denominator (100%) always selects.
        let max_out = [0xFFu8; BLAKE3_BYTES]; // value = u64::MAX
        let min_out = [0x00u8; BLAKE3_BYTES]; // value = 0
        assert!(!vrf_is_selected(&max_out, 0), "rate 0 never selects");
        assert!(
            !vrf_is_selected(&min_out, 0),
            "rate 0 never selects (even value 0)"
        );
        assert!(
            vrf_is_selected(&max_out, VRF_RATE_DENOMINATOR),
            "rate 100% selects the maximum output"
        );
        assert!(
            vrf_is_selected(&min_out, VRF_RATE_DENOMINATOR),
            "rate 100% selects the minimum output"
        );
        // A rate above the denominator still always selects (saturating intent).
        assert!(vrf_is_selected(&max_out, VRF_RATE_DENOMINATOR + 1));
    }

    #[test]
    fn vrf_is_selected_threshold_is_proportional() {
        // value just below the 50% threshold (2^63 - 1) is selected at 50%;
        // value at the threshold (2^63) is not (strict `<`).
        let mut below = [0u8; BLAKE3_BYTES];
        below[..8].copy_from_slice(&(0x7FFF_FFFF_FFFF_FFFFu64).to_be_bytes());
        let mut at = [0u8; BLAKE3_BYTES];
        at[..8].copy_from_slice(&(0x8000_0000_0000_0000u64).to_be_bytes());
        let half = VRF_RATE_DENOMINATOR / 2; // 5000 bp = 50%
        assert!(vrf_is_selected(&below, half), "just below 50% selects");
        assert!(
            !vrf_is_selected(&at, half),
            "exactly 50% does not (strict <)"
        );
    }

    #[test]
    fn n1_spot_check_randomizes_temp_and_seed() {
        let kp = KeyPair::generate();
        // Two different draws (different seeds) produce different temp+seed,
        // each deterministic from its own output.
        let d1 = vrf_draw(&kp, b"session-1|epoch-1|commit-1");
        let d2 = vrf_draw(&kp, b"session-1|epoch-2|commit-2");

        let t1 = derive_spotcheck_temp_milli(&d1.output);
        let t1_again = derive_spotcheck_temp_milli(&d1.output);
        assert_eq!(t1, t1_again, "temperature is deterministic from the output");
        assert!(t1 < VRF_MAX_TEMP_MILLI, "temperature within bound");

        // Randomisation across draws. A single `t1 != t2` would be flaky at
        // ~1/VRF_MAX_TEMP_MILLI (the temp lives in [0, 2000)); sampling 8 distinct
        // draws makes an all-equal false negative ~(1/2000)^7 — effectively
        // impossible — so we assert the derived temperatures are not constant.
        let temps: std::collections::HashSet<u32> = (0u32..8)
            .map(|i| {
                derive_spotcheck_temp_milli(&vrf_draw(&kp, format!("s|{i}").as_bytes()).output)
            })
            .collect();
        assert!(
            temps.iter().all(|&t| t < VRF_MAX_TEMP_MILLI),
            "all derived temperatures within bound"
        );
        assert!(
            temps.len() > 1,
            "temperature varies across draws (not a constant)"
        );

        let s1 = derive_spotcheck_seed(&d1.output);
        let s1_again = derive_spotcheck_seed(&d1.output);
        let s2 = derive_spotcheck_seed(&d2.output);
        assert_eq!(s1, s1_again, "seed is deterministic from the output");
        // The sampling seed is a u64, so a cross-draw collision is ~2^-64.
        assert_ne!(s1, s2, "a different draw randomises the sampling seed");

        // temp and seed are independent sub-derivations (no accidental reuse).
        assert_ne!(
            u64::from(t1),
            s1,
            "temp and seed derive from distinct personalisations"
        );
    }

    #[test]
    fn temp_and_seed_are_domain_separated_from_output() {
        // The temp/seed sub-derivation must not equal the raw output prefix,
        // i.e. the personalisation actually changes the hash.
        let kp = KeyPair::generate();
        let d = vrf_draw(&kp, b"x");
        let raw_prefix = u32::from_be_bytes([d.output[0], d.output[1], d.output[2], d.output[3]]);
        // Astronomically unlikely to collide; guards against forgetting perso.
        let temp_raw = {
            let h = personalised(&d.output, VRF_TEMP_PERSO);
            u32::from_be_bytes([h[0], h[1], h[2], h[3]])
        };
        assert_ne!(raw_prefix, temp_raw, "temp derivation is personalised");
    }
}
