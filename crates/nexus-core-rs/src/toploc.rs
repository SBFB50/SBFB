// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase G — N0 TOPLOC fingerprint (verifiable-inference primitive).
//!
//! TOPLOC (arXiv 2501.16007, PrimeIntellect-ai/toploc) is a locality-sensitive
//! fingerprint over the **top-k largest-magnitude entries of the last hidden
//! state**. It detects a model / precision swap (~100%) while tolerating the
//! floating-point non-determinism of an honest re-run on a different GPU — the
//! property a byte-exact hash cannot offer (`verification.rs` Layer 3 used hash
//! equality precisely because the canonical wire format forbids floats).
//!
//! ## What Phase G delivers (and what it does NOT)
//!
//! - A canonical, **all-integer** fingerprint of a top-k extraction
//!   ([`ToplocFingerprint`]): the top-k indices plus each value's **bf16 bit
//!   pattern** (`u16`), index-sorted so the bytes are deterministic regardless
//!   of the caller's magnitude ordering.
//! - A 32-byte **BLAKE3 commitment** of that canonical encoding
//!   ([`ToplocFingerprint::commitment`]). This is what goes on the wire: the
//!   `[u8; 32]` slots [`crate::task::ResultPayload::logprobs_hash`] (whole-model
//!   path) and [`crate::shard_plan::RunProof::activation_fingerprint`] (shard
//!   path) — **0 bump wire**, the slots already exist. The full encoding is
//!   258 B/32 tok in the original GF(65497)-compressed form; SBFB uses the
//!   direct integer sketch (~768 B/32 tok) for auditability — the GF
//!   compression is a deferrable on-wire optimisation, not needed while only the
//!   32-byte commitment is transported (Phase G).
//! - A **tolerant comparison** primitive ([`ToplocFingerprint::compare`]) over
//!   the full sketch: exponent-mismatch count + mantissa absolute-error
//!   mean/median against per-precision thresholds (bf16: 38 / 10 / 8). It is
//!   exposed for the verifiers but **not wired in-vivo here**.
//!
//! ## Commitment is a binding, NOT a tolerant comparator
//!
//! A BLAKE3 hash destroys locality (one bit flip avalanches), so the 32-byte
//! commitment can only ever be compared by **equality** — it binds a worker to
//! one fingerprint and detects a swap by inequality. The real *tolerant* check
//! needs the full sketch on both sides, which has **no on-wire carrier in Phase
//! G**: transporting the full payload and running the tolerant comparison
//! cross-worker in-vivo is **Phase H (N1 VRF spot-check) / Phase I (N2 tolerant
//! redundancy)**. This is exactly the "separate off-canonical payload" the old
//! Layer-3 doc-note anticipated.
//!
//! ## Auto-attestation caveat
//!
//! A commitment a worker writes for its own run is a **self-claim**: it proves
//! nothing about correctness until an independent verifier (N1/N2, Phase H/I)
//! re-runs the same model + prompt and recomputes the fingerprint. Treat it
//! exactly like [`crate::task::ResultPayload::model_digest`] — never a guarantee.
//!
//! ## All-integer (no-float) pre-image — load-bearing
//!
//! The hashed pre-image must be all-integer or a Rust signer and a Python
//! verifier would derive divergent bytes and the commitment would never match.
//! The float top-k values are quantised to their **bf16 bits** — the top 16
//! bits of the IEEE-754 word, `bf16_bits(value)`, a deterministic truncation —
//! before they enter the encoding. fp32 is intentionally not the on-encoding
//! dtype (bf16 is the top half of an fp32 word; the thresholds below are the
//! paper's bf16 set).

use std::collections::HashMap;

use crate::crypto::{BLAKE3_BYTES, blake3_hash};
use crate::error::{NexusError, Result};

/// Number of largest-magnitude activations the fingerprint keeps (TOPLOC k).
/// Also the hard cap on entries decoded from bytes (DoS bound).
pub const TOPLOC_TOP_K: usize = 128;

/// Accept iff the exponent-mismatch count is **strictly below** this. The bf16
/// exponent is robust to GPU non-determinism (a small relative perturbation
/// almost never changes the 8 exponent bits), so a same-model re-run stays well
/// under 38 out of `k=128`, while a model/precision swap blows past it. From
/// arXiv 2501.16007v2 (bf16 set); re-calibration on the real rig is Phase K.
pub const TOPLOC_THRESH_EXP_MISMATCH: u32 = 38;

/// Accept iff the mantissa absolute-error **mean** is strictly below this
/// (compared as `sum < THRESH * count` to stay integer-only even locally).
pub const TOPLOC_THRESH_MANT_MEAN: u32 = 10;

/// Accept iff the mantissa absolute-error **median** is strictly below this
/// (compared as `median*2 < THRESH*2` to keep the even-length midpoint integer).
pub const TOPLOC_THRESH_MANT_MEDIAN: u32 = 8;

/// bf16 exponent field mask (8 bits) over a `u16` bit pattern.
const BF16_EXP_MASK: u16 = 0x7F80;
/// Right shift to bring the bf16 exponent field to the low bits.
const BF16_EXP_SHIFT: u32 = 7;
/// bf16 mantissa field mask (7 bits).
const BF16_MANT_MASK: u16 = 0x007F;

/// Quantise an `f32` activation to its bf16 bit pattern: the top 16 bits of the
/// IEEE-754 word (a deterministic round-to-zero truncation, not hardware
/// round-to-nearest — the at-most-1-ULP mantissa difference versus a true bf16
/// tensor is absorbed by the tolerant comparison thresholds). This is the only
/// float→integer boundary; everything downstream is integer.
#[must_use]
pub fn bf16_bits(value: f32) -> u16 {
    (value.to_bits() >> 16) as u16
}

/// The 8-bit bf16 exponent field of a bit pattern.
#[must_use]
fn bf16_exp(bits: u16) -> u16 {
    (bits & BF16_EXP_MASK) >> BF16_EXP_SHIFT
}

/// The 7-bit bf16 mantissa field of a bit pattern.
#[must_use]
fn bf16_mant(bits: u16) -> u16 {
    bits & BF16_MANT_MASK
}

/// The canonical, all-integer N0 fingerprint of one top-k extraction.
///
/// `indices` and `value_bits` are parallel and **index-sorted ascending**, so
/// two extractions of the same top-k set produce identical bytes (hence
/// identical [`commitment`](Self::commitment)) regardless of the order the
/// caller passed them in. `value_bits[i]` is the bf16 bit pattern of the
/// activation at `indices[i]`.
///
/// Deliberately not `Serialize`/`Deserialize`: only the 32-byte
/// [`commitment`](Self::commitment) crosses the wire, and the bounded
/// [`from_bytes`](Self::from_bytes) is the only decoder (a serde decoder would
/// bypass the [`TOPLOC_TOP_K`] DoS cap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToplocFingerprint {
    /// Top-k activation indices, sorted ascending. Bounded by [`TOPLOC_TOP_K`].
    indices: Vec<u32>,
    /// bf16 bit pattern of each kept activation, parallel to `indices`.
    value_bits: Vec<u16>,
}

/// The outcome of a tolerant [`ToplocFingerprint::compare`].
///
/// All-integer: `mant_err_mean < THRESH` is evaluated as `mant_err_sum <
/// THRESH * mant_err_count`, and the median as `mant_err_median_x2 < 2*THRESH`,
/// so no float is needed even for the local decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToplocComparison {
    /// Number of replay indices whose bf16 exponent differs from the prover's
    /// (or which the prover did not list in its top-k at all).
    pub exp_mismatches: u32,
    /// Sum of mantissa absolute errors over indices that matched on exponent.
    pub mant_err_sum: u64,
    /// Number of indices that contributed to `mant_err_sum`.
    pub mant_err_count: u32,
    /// Median mantissa absolute error **times two** (integer midpoint).
    /// `u64::MAX` when no index matched on exponent (sentinel reject).
    pub mant_err_median_x2: u64,
    /// Whether all three thresholds passed.
    pub accepted: bool,
}

impl ToplocFingerprint {
    /// Build a fingerprint from an already-selected top-k list of
    /// `(index, value)` pairs — the output of the worker-side
    /// `top_k_by_magnitude` (Phase F1). At most [`TOPLOC_TOP_K`] entries are
    /// kept (in input order, so a magnitude-sorted caller keeps the largest),
    /// then sorted by index for a deterministic canonical encoding.
    ///
    /// The float values are quantised to bf16 bits here ([`bf16_bits`]); no
    /// float survives into the struct. Duplicate indices (not expected within a
    /// single hidden state's top-k) are kept as-is and collapse in
    /// [`compare`](Self::compare).
    #[must_use]
    pub fn from_topk(topk: &[(u32, f32)]) -> Self {
        let mut kept: Vec<(u32, u16)> = topk
            .iter()
            .take(TOPLOC_TOP_K)
            .map(|&(idx, v)| (idx, bf16_bits(v)))
            .collect();
        kept.sort_unstable_by_key(|&(idx, _)| idx);
        ToplocFingerprint {
            indices: kept.iter().map(|&(i, _)| i).collect(),
            value_bits: kept.iter().map(|&(_, b)| b).collect(),
        }
    }

    /// The top-k indices (index-sorted ascending).
    #[must_use]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    /// The parallel bf16 bit patterns.
    #[must_use]
    pub fn value_bits(&self) -> &[u16] {
        &self.value_bits
    }

    /// Number of kept entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Whether the fingerprint is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    /// Serialise to the canonical, all-integer byte string:
    /// `[count: u32 BE]` then `count ×` (`[index: u32 BE][value_bits: u16 BE]`).
    /// This is the BLAKE3 pre-image; it round-trips through [`from_bytes`].
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let n = self.indices.len();
        let mut out = Vec::with_capacity(4 + n * 6);
        out.extend_from_slice(&(n as u32).to_be_bytes());
        for (&idx, &vb) in self.indices.iter().zip(self.value_bits.iter()) {
            out.extend_from_slice(&idx.to_be_bytes());
            out.extend_from_slice(&vb.to_be_bytes());
        }
        out
    }

    /// Parse a fingerprint from [`to_bytes`](Self::to_bytes) output.
    ///
    /// # Errors
    ///
    /// Rejects a header count above [`TOPLOC_TOP_K`] (DoS bound, before any
    /// allocation proportional to it) and any length that does not match the
    /// declared count exactly.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(NexusError::Other(
                "toploc fingerprint: truncated header (< 4 bytes)".into(),
            ));
        }
        let n = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        if n > TOPLOC_TOP_K {
            return Err(NexusError::Other(format!(
                "toploc fingerprint declares {n} entries, exceeds TOPLOC_TOP_K={TOPLOC_TOP_K}"
            )));
        }
        let expected = 4 + n * 6;
        if bytes.len() != expected {
            return Err(NexusError::Other(format!(
                "toploc fingerprint length {} != expected {expected} for {n} entries",
                bytes.len()
            )));
        }
        let mut indices = Vec::with_capacity(n);
        let mut value_bits = Vec::with_capacity(n);
        for i in 0..n {
            let off = 4 + i * 6;
            indices.push(u32::from_be_bytes([
                bytes[off],
                bytes[off + 1],
                bytes[off + 2],
                bytes[off + 3],
            ]));
            value_bits.push(u16::from_be_bytes([bytes[off + 4], bytes[off + 5]]));
        }
        Ok(ToplocFingerprint {
            indices,
            value_bits,
        })
    }

    /// The 32-byte BLAKE3 commitment of the canonical encoding — the value
    /// written into the `[u8; 32]` wire slot. Equal commitments ⟺ byte-identical
    /// fingerprints; a mismatch is a swap **or** honest non-determinism, so the
    /// authoritative verdict is the tolerant [`compare`](Self::compare) (Phase
    /// H/I), not the commitment alone.
    #[must_use]
    pub fn commitment(&self) -> [u8; BLAKE3_BYTES] {
        blake3_hash(&self.to_bytes())
    }

    /// Tolerantly compare a `replay` fingerprint (an independent verifier's
    /// re-extraction) against `self` (the prover's claim), aligning by index.
    ///
    /// For each replay index: if the prover listed it, a differing bf16
    /// exponent counts as a mismatch, else its mantissa absolute error is
    /// recorded; a replay index the prover never listed counts as a mismatch.
    /// Accept iff exponent mismatches are below [`TOPLOC_THRESH_EXP_MISMATCH`]
    /// **and** the mantissa-error mean and median are below their thresholds
    /// (with no exponent-matching index at all treated as a reject sentinel).
    #[must_use]
    pub fn compare(&self, replay: &ToplocFingerprint) -> ToplocComparison {
        let prover: HashMap<u32, u16> = self
            .indices
            .iter()
            .copied()
            .zip(self.value_bits.iter().copied())
            .collect();

        let mut exp_mismatches: u32 = 0;
        let mut mant_errs: Vec<u16> = Vec::with_capacity(replay.indices.len());
        for (&idx, &vb_replay) in replay.indices.iter().zip(replay.value_bits.iter()) {
            match prover.get(&idx) {
                Some(&vb_prover) => {
                    if bf16_exp(vb_prover) != bf16_exp(vb_replay) {
                        exp_mismatches += 1;
                    } else {
                        mant_errs.push(bf16_mant(vb_prover).abs_diff(bf16_mant(vb_replay)));
                    }
                }
                None => exp_mismatches += 1,
            }
        }

        let count = mant_errs.len() as u32;
        let sum: u64 = mant_errs.iter().map(|&e| u64::from(e)).sum();
        mant_errs.sort_unstable();
        let median_x2: u64 = if mant_errs.is_empty() {
            u64::MAX
        } else if mant_errs.len() % 2 == 1 {
            2 * u64::from(mant_errs[mant_errs.len() / 2])
        } else {
            u64::from(mant_errs[mant_errs.len() / 2 - 1])
                + u64::from(mant_errs[mant_errs.len() / 2])
        };

        let exp_ok = exp_mismatches < TOPLOC_THRESH_EXP_MISMATCH;
        // mean < THRESH  <=>  sum < THRESH * count  (count > 0)
        let mean_ok = count > 0 && sum < u64::from(TOPLOC_THRESH_MANT_MEAN) * u64::from(count);
        // median < THRESH  <=>  median_x2 < 2 * THRESH
        let median_ok =
            !mant_errs.is_empty() && median_x2 < 2 * u64::from(TOPLOC_THRESH_MANT_MEDIAN);
        let accepted = exp_ok && mean_ok && median_ok;

        ToplocComparison {
            exp_mismatches,
            mant_err_sum: sum,
            mant_err_count: count,
            mant_err_median_x2: median_x2,
            accepted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a bf16 bit pattern from explicit exponent (8 bits) and mantissa
    /// (7 bits), sign 0. Lets tests craft same-exp / cross-exp fixtures exactly.
    fn bits(exp: u16, mant: u16) -> u16 {
        ((exp & 0xFF) << BF16_EXP_SHIFT) | (mant & BF16_MANT_MASK)
    }

    /// Build a fingerprint directly from `(index, value_bits)` (index-sorted),
    /// bypassing the f32 quantisation so a test controls the exact bits.
    fn fp(entries: &[(u32, u16)]) -> ToplocFingerprint {
        let mut e = entries.to_vec();
        e.sort_unstable_by_key(|&(i, _)| i);
        ToplocFingerprint {
            indices: e.iter().map(|&(i, _)| i).collect(),
            value_bits: e.iter().map(|&(_, b)| b).collect(),
        }
    }

    #[test]
    fn toploc_fingerprint_encode_decode_roundtrip() {
        // From f32 top-k (the real worker path), encode → decode → identical.
        let topk: Vec<(u32, f32)> = vec![(5, -12.5), (1, 100.0), (9, 3.25), (3, -7.0), (7, 42.0)];
        let f = ToplocFingerprint::from_topk(&topk);
        // Index-sorted canonical order.
        assert_eq!(f.indices(), &[1, 3, 5, 7, 9]);
        let bytes = f.to_bytes();
        assert_eq!(bytes.len(), 4 + 5 * 6, "header u32 + 5 × (u32+u16)");
        let back = ToplocFingerprint::from_bytes(&bytes).expect("decode");
        assert_eq!(back, f, "round-trip is exact");
    }

    #[test]
    fn bf16_bits_is_top_half_of_f32() {
        // 1.0_f32 = 0x3F80_0000 → bf16 0x3F80 (exp 0x7F, mant 0).
        assert_eq!(bf16_bits(1.0), 0x3F80);
        assert_eq!(bf16_exp(bf16_bits(1.0)), 0x7F);
        assert_eq!(bf16_mant(bf16_bits(1.0)), 0);
        // 2.0_f32 = 0x4000_0000 → bf16 0x4000.
        assert_eq!(bf16_bits(2.0), 0x4000);
    }

    #[test]
    fn toploc_detects_model_swap() {
        // Prover top-k.
        let prover = fp(&[
            (0, bits(128, 0)),
            (1, bits(130, 4)),
            (2, bits(125, 2)),
            (3, bits(140, 1)),
            (4, bits(135, 7)),
        ]);
        // A different model → disjoint top-k indices (and exponents).
        let swapped = fp(&[
            (10, bits(90, 5)),
            (11, bits(95, 1)),
            (12, bits(100, 3)),
            (13, bits(88, 6)),
            (14, bits(92, 2)),
        ]);
        // 1) The wire commitment differs (binding by inequality).
        assert_ne!(
            prover.commitment(),
            swapped.commitment(),
            "a model swap must produce a different commitment"
        );
        // 2) The tolerant compare rejects: every replay index is absent from the
        //    prover → all exponent mismatches, no mantissa data → sentinel.
        let cmp = prover.compare(&swapped);
        assert_eq!(cmp.exp_mismatches, 5);
        assert_eq!(cmp.mant_err_count, 0);
        assert!(!cmp.accepted, "disjoint top-k must be rejected");
    }

    #[test]
    fn toploc_accepts_same_model_within_threshold() {
        // Powers-of-two-ish bases: mantissa 0, so a +delta stays inside the
        // 7-bit field (no exponent overflow).
        let prover = fp(&[
            (0, bits(128, 0)),
            (1, bits(130, 0)),
            (2, bits(125, 0)),
            (3, bits(140, 0)),
            (4, bits(135, 0)),
        ]);
        // Same model, FP noise: same indices, same exponents, mantissa +1.
        let replay_same = fp(&[
            (0, bits(128, 1)),
            (1, bits(130, 1)),
            (2, bits(125, 1)),
            (3, bits(140, 1)),
            (4, bits(135, 1)),
        ]);
        let cmp = prover.compare(&replay_same);
        assert_eq!(cmp.exp_mismatches, 0);
        assert_eq!(cmp.mant_err_count, 5);
        assert_eq!(cmp.mant_err_sum, 5, "mantissa error 1 each");
        assert!(
            cmp.accepted,
            "same model within FP tolerance must be accepted ({cmp:?})"
        );
    }

    #[test]
    fn toploc_rejects_mantissa_drift_over_threshold() {
        let prover = fp(&[(0, bits(128, 0)), (1, bits(130, 0)), (2, bits(125, 0))]);
        // Same exponents but a large mantissa drift (+20 each): mean 20 ≥ 10.
        let replay = fp(&[(0, bits(128, 20)), (1, bits(130, 20)), (2, bits(125, 20))]);
        let cmp = prover.compare(&replay);
        assert_eq!(cmp.exp_mismatches, 0);
        assert_eq!(cmp.mant_err_count, 3);
        assert!(
            !cmp.accepted,
            "mantissa drift past the mean threshold must be rejected ({cmp:?})"
        );
    }

    #[test]
    fn compare_counts_missing_replay_index_as_mismatch() {
        let prover = fp(&[(0, bits(128, 0)), (1, bits(130, 0))]);
        // One shared index (exp+mant identical), one the prover never listed.
        let replay = fp(&[(0, bits(128, 0)), (99, bits(130, 0))]);
        let cmp = prover.compare(&replay);
        assert_eq!(cmp.exp_mismatches, 1, "index 99 absent from prover");
        assert_eq!(cmp.mant_err_count, 1, "only the shared index contributes");
    }

    #[test]
    fn commitment_is_blake3_of_canonical_bytes() {
        let f = fp(&[(2, bits(100, 3)), (0, bits(120, 1))]);
        assert_eq!(f.commitment(), blake3_hash(&f.to_bytes()));
    }

    #[test]
    fn canonical_order_is_input_order_independent() {
        // The same set passed in two different orders → identical bytes and
        // identical commitment (index-sorted canonicalisation).
        let a = ToplocFingerprint::from_topk(&[(9, 1.0), (1, 2.0), (5, 3.0)]);
        let b = ToplocFingerprint::from_topk(&[(1, 2.0), (5, 3.0), (9, 1.0)]);
        assert_eq!(a.to_bytes(), b.to_bytes());
        assert_eq!(a.commitment(), b.commitment());
    }

    #[test]
    fn from_topk_caps_at_top_k() {
        let many: Vec<(u32, f32)> = (0..(TOPLOC_TOP_K as u32 + 50))
            .map(|i| (i, i as f32))
            .collect();
        let f = ToplocFingerprint::from_topk(&many);
        assert_eq!(f.len(), TOPLOC_TOP_K, "kept entries capped at TOPLOC_TOP_K");
    }

    #[test]
    fn from_bytes_rejects_oversized_count() {
        // Header claims TOPLOC_TOP_K + 1 entries → rejected before allocation.
        let mut bytes = ((TOPLOC_TOP_K as u32) + 1).to_be_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; 6]);
        let err = ToplocFingerprint::from_bytes(&bytes).unwrap_err();
        assert!(err.to_string().contains("exceeds TOPLOC_TOP_K"));
    }

    #[test]
    fn from_bytes_rejects_length_mismatch() {
        // Count says 2 entries (needs 4 + 12 = 16 bytes) but only 10 provided.
        let mut bytes = 2u32.to_be_bytes().to_vec();
        bytes.extend_from_slice(&[0u8; 6]);
        assert!(ToplocFingerprint::from_bytes(&bytes).is_err());
    }

    #[test]
    fn identical_fingerprint_compares_accepted() {
        let f = fp(&[(0, bits(128, 3)), (1, bits(130, 5)), (2, bits(125, 1))]);
        let cmp = f.compare(&f);
        assert_eq!(cmp.exp_mismatches, 0);
        assert_eq!(cmp.mant_err_sum, 0);
        assert!(cmp.accepted, "a fingerprint always accepts itself");
    }

    #[test]
    fn compare_even_length_median() {
        // An EVEN mantissa-error count exercises the midpoint-average branch of
        // the median (errs[n/2-1] + errs[n/2]).
        let prover = fp(&[
            (0, bits(128, 0)),
            (1, bits(130, 0)),
            (2, bits(125, 0)),
            (3, bits(140, 0)),
        ]);
        // Errors [1,2,3,8] (count 4): median = (2+3)/2 = 2.5 → median_x2 = 5.
        let accept = fp(&[
            (0, bits(128, 1)),
            (1, bits(130, 2)),
            (2, bits(125, 3)),
            (3, bits(140, 8)),
        ]);
        let cmp = prover.compare(&accept);
        assert_eq!(cmp.mant_err_count, 4);
        assert_eq!(cmp.mant_err_median_x2, 5, "even midpoint = errs[1]+errs[2]");
        assert!(cmp.accepted, "median 2.5 < 8 accepts ({cmp:?})");
        // Errors [8,8,9,9]: median_x2 = 8+9 = 17 ≥ 16 → rejected on the median.
        let reject = fp(&[
            (0, bits(128, 8)),
            (1, bits(130, 8)),
            (2, bits(125, 9)),
            (3, bits(140, 9)),
        ]);
        let cmp = prover.compare(&reject);
        assert_eq!(cmp.mant_err_median_x2, 17);
        assert!(!cmp.accepted, "even-length median at 8.5 rejects ({cmp:?})");
    }

    #[test]
    fn compare_same_index_exponent_mismatch() {
        // Shared indices where ONE has a differing exponent — isolates the
        // Some(_)+exp-differs branch (distinct from the missing-index branch).
        let prover = fp(&[(0, bits(128, 5)), (1, bits(130, 5)), (2, bits(125, 5))]);
        let replay = fp(&[
            (0, bits(99, 5)),  // exponent differs → exp_mismatch
            (1, bits(130, 6)), // exponent matches, mantissa off by 1
            (2, bits(125, 5)), // identical
        ]);
        let cmp = prover.compare(&replay);
        assert_eq!(
            cmp.exp_mismatches, 1,
            "one shared index has a wrong exponent"
        );
        assert_eq!(cmp.mant_err_count, 2, "the two exp-matching indices");
        assert_eq!(cmp.mant_err_sum, 1);
        assert!(
            cmp.accepted,
            "a single exponent mismatch is within tolerance"
        );
    }

    #[test]
    fn compare_threshold_boundaries() {
        // Lock the strict `<` boundaries so an off-by-one (< vs <=) goes red.

        // Mantissa errors with the median pinned at 0, varying the mean.
        let mant_cmp = |errs: &[u16]| {
            let prover: Vec<(u32, u16)> = errs
                .iter()
                .enumerate()
                .map(|(i, _)| (i as u32, bits(130, 0)))
                .collect();
            let replay: Vec<(u32, u16)> = errs
                .iter()
                .enumerate()
                .map(|(i, &e)| (i as u32, bits(130, e)))
                .collect();
            fp(&prover).compare(&fp(&replay))
        };
        // mean == THRESH (sum 50 == 10×5) rejects; mean just under accepts.
        assert!(
            !mant_cmp(&[0, 0, 0, 0, 50]).accepted,
            "mean == THRESH_MANT_MEAN rejects (strict <)"
        );
        assert!(
            mant_cmp(&[0, 0, 0, 0, 49]).accepted,
            "mean just under accepts"
        );
        // median == THRESH (8) rejects; median just under (7) accepts.
        assert!(
            !mant_cmp(&[8, 8, 8]).accepted,
            "median == THRESH_MANT_MEDIAN rejects (strict <)"
        );
        assert!(mant_cmp(&[7, 7, 7]).accepted, "median just under accepts");

        // Exponent-mismatch boundary: T mismatches + 2 clean indices (so the
        // mantissa stats are non-empty and pass), varying the mismatch count.
        let exp_cmp = |n_mismatch: u32| {
            let mut prover = Vec::new();
            let mut replay = Vec::new();
            for i in 0..n_mismatch {
                prover.push((i, bits(128, 0)));
                replay.push((i, bits(100, 0))); // different exponent
            }
            prover.push((1000, bits(130, 0)));
            replay.push((1000, bits(130, 0)));
            prover.push((1001, bits(130, 0)));
            replay.push((1001, bits(130, 0)));
            fp(&prover).compare(&fp(&replay))
        };
        let at = exp_cmp(TOPLOC_THRESH_EXP_MISMATCH);
        assert_eq!(at.exp_mismatches, TOPLOC_THRESH_EXP_MISMATCH);
        assert!(
            !at.accepted,
            "exp_mismatches == THRESH_EXP_MISMATCH rejects (strict <)"
        );
        assert!(
            exp_cmp(TOPLOC_THRESH_EXP_MISMATCH - 1).accepted,
            "exp_mismatches just under THRESH accepts"
        );
    }
}
