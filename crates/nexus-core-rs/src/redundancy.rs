// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase I — N2 tolerant redundancy (M-of-N fingerprint quorum).
//!
//! N2 is the high-criticality verification level of the sharding addendum (§3):
//! a task runs on `redundancy_factor` independent workers and is accepted iff a
//! quorum of them **corroborate the same computation**. Unlike the exact
//! `result_text` quorum on the whole-model path
//! ([`crate::task`] / coordinator `validate_quorum_pre_guardrail`), shard workers
//! run on heterogeneous GPUs whose floating-point non-determinism makes a
//! byte-exact agreement impossible — two honest re-runs differ in the low
//! mantissa bits (the very reason N0 uses the locality-sensitive
//! [`crate::toploc`] fingerprint, never a hash). So N2 agreement is the
//! **tolerant** [`crate::toploc::ToplocFingerprint::compare`] (exponent-exact +
//! mantissa-mean/median under threshold), generalised to M-of-N.
//!
//! ## Mutual agreement, NOT a pivot star (transitivity)
//!
//! Tolerant agreement is **not transitive**: `A ≈ B` and `B ≈ C` does not imply
//! `A ≈ C` (a "straddling" fingerprint can sit in the tolerance band of two
//! mutually-divergent ones). Counting how many fingerprints agree with a single
//! pivot would therefore over-count — a lone straddler `B` between divergent `A`
//! and `C` would inflate a 3-way quorum that does not exist. N2 instead requires
//! a **clique**: the largest set whose members are *pairwise* tolerant
//! ([`largest_agreeing_cluster`]). With a redundancy factor in the single digits
//! ([`N2_MAX_FINGERPRINTS`]) the exact maximum clique is cheap.
//!
//! ## Non-falsifiability — the verdict rests on SIGNED inputs
//!
//! *Which* tasks use N2 is selected from the task's criticality
//! ([`crate::verification::criticality_maps_to_verification_level`]), which is
//! **advisory** because `redundancy_factor` is excluded from the canonical bytes
//! (Sprint 23 `34c77ce`). The ACCEPT/REJECT verdict here must therefore be fed
//! fingerprints derived from **signed** [`crate::shard_plan::RunProof`]s — never
//! decisions taken on the unsigned field directly. This module is the pure
//! verdict logic; the caller (coordinator) supplies the signed-derived
//! fingerprints.
//!
//! ## Threat honesty (THREAT_MODEL §16 N2)
//!
//! A coalition of M workers that agree on a *close-but-wrong* fingerprint
//! (within the tolerance band) defeats N2 — an instance of the SI-4 integrity
//! surface, assumed bounded by the closed pilot + anti-Sybil admission, never by
//! an economic stake (PO-12). The tolerance threshold is a security parameter:
//! too wide silently accepts a swap, too tight false-rejects honest cross-GPU
//! variance. N2 reuses the calibrated [`crate::toploc`] thresholds rather than
//! inventing its own.

use crate::toploc::ToplocFingerprint;

/// Default minimum number of mutually-agreeing fingerprints for an M-of-N
/// tolerant quorum, when a caller has no task-specific `redundancy_factor` to
/// derive a majority from. Two corroborating workers are the smallest set that
/// can detect a single divergent one. A caller with a known `redundancy_factor`
/// should pass a strict majority (`redundancy_factor / 2 + 1`) to
/// [`tolerant_quorum_accepts`], mirroring the exact quorum's
/// `count > redundancy_factor / 2`.
pub const TOLERANT_QUORUM_MIN_AGREE: usize = 2;

/// Hard upper bound on the number of fingerprints a single N2 quorum considers.
/// A redundancy quorum is a single-digit fan-out (addendum §1: 3-5 machines);
/// this bound keeps the exact maximum-clique search trivially fast and caps a
/// pathological caller. Fingerprints beyond it are not considered (a redundancy
/// quorum never legitimately exceeds it).
pub const N2_MAX_FINGERPRINTS: usize = 32;

/// Whether two frontier fingerprints tolerantly agree, **symmetrically**.
///
/// [`crate::toploc::ToplocFingerprint::compare`] is directional (it aligns the
/// replay's indices against the prover's map), so when two fingerprints list
/// different index sets `a.compare(b)` and `b.compare(a)` can disagree. N2 has
/// no designated prover among peer workers, so agreement requires **both**
/// directions to accept — the stricter, order-independent relation.
#[must_use]
pub fn fingerprints_agree(a: &ToplocFingerprint, b: &ToplocFingerprint) -> bool {
    a.compare(b).accepted && b.compare(a).accepted
}

/// The size of the largest **clique** of pairwise-agreeing fingerprints.
///
/// Builds the symmetric agreement graph ([`fingerprints_agree`]) and returns the
/// size of its maximum clique by exact branch-and-bound. Only the first
/// [`N2_MAX_FINGERPRINTS`] are considered (a redundancy quorum never legitimately
/// exceeds that — the bound keeps the NP-hard search bounded). Returns 0 for an
/// empty input, 1 for a singleton.
#[must_use]
pub fn largest_agreeing_cluster(fingerprints: &[ToplocFingerprint]) -> usize {
    let n = fingerprints.len().min(N2_MAX_FINGERPRINTS);
    if n == 0 {
        return 0;
    }

    // Symmetric agreement adjacency (diagonal implicitly true: a vertex is in
    // its own clique). Computed once; `compare` is the only cost.
    let adj: Vec<Vec<bool>> = (0..n)
        .map(|i| {
            (0..n)
                .map(|j| i == j || fingerprints_agree(&fingerprints[i], &fingerprints[j]))
                .collect()
        })
        .collect();

    let mut best = 1;
    let candidates: Vec<usize> = (0..n).collect();
    extend_clique(&adj, 0, &candidates, &mut best);
    best
}

/// Branch-and-bound maximum-clique expansion. `current` is the clique size built
/// so far; `candidates` are vertices each adjacent to every member of the
/// current clique. Prunes a branch that cannot beat `best`.
fn extend_clique(adj: &[Vec<bool>], current: usize, candidates: &[usize], best: &mut usize) {
    if current > *best {
        *best = current;
    }
    for (offset, &v) in candidates.iter().enumerate() {
        // Bound: even taking every remaining candidate cannot beat `best`.
        if current + (candidates.len() - offset) <= *best {
            break;
        }
        // Extend the clique with `v`; the new candidate set keeps only the
        // later candidates that are also adjacent to `v` (so the clique stays
        // fully connected).
        let next: Vec<usize> = candidates[offset + 1..]
            .iter()
            .copied()
            .filter(|&u| adj[v][u])
            .collect();
        extend_clique(adj, current + 1, &next, best);
    }
}

/// Whether an M-of-N tolerant quorum accepts: at least `min_agree` of the
/// supplied frontier fingerprints mutually agree (form a clique of that size).
///
/// `min_agree == 0` accepts vacuously (no corroboration demanded) and
/// `min_agree == 1` accepts any non-empty input — callers pass a meaningful
/// majority (see [`TOLERANT_QUORUM_MIN_AGREE`]).
#[must_use]
pub fn tolerant_quorum_accepts(fingerprints: &[ToplocFingerprint], min_agree: usize) -> bool {
    largest_agreeing_cluster(fingerprints) >= min_agree
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A small honest cross-GPU re-run: same top-k indices, values perturbed by
    /// sub-threshold floating-point noise (the bf16 truncation of `from_topk`
    /// keeps the exponent and most mantissa bits).
    fn close_set() -> Vec<ToplocFingerprint> {
        vec![
            ToplocFingerprint::from_topk(&[(1, 100.0), (3, 200.0), (5, 50.0)]),
            ToplocFingerprint::from_topk(&[(1, 101.0), (3, 202.0), (5, 50.5)]),
            ToplocFingerprint::from_topk(&[(1, 100.5), (3, 201.0), (5, 50.25)]),
        ]
    }

    #[test]
    fn n2_tolerant_quorum_accepts_close_fingerprints() {
        let fps = close_set();
        // All three mutually agree → clique of 3.
        assert_eq!(largest_agreeing_cluster(&fps), 3);
        assert!(tolerant_quorum_accepts(&fps, 2));
        assert!(tolerant_quorum_accepts(&fps, 3));
        // Sanity: the fingerprints are NOT byte-identical (else the test is a
        // tautology and could pass under hash-equality) — N2 is tolerant.
        assert_ne!(
            fps[0].commitment(),
            fps[1].commitment(),
            "close fingerprints must differ byte-wise (tolerant, not hash-eq)"
        );
    }

    #[test]
    fn n2_tolerant_quorum_rejects_divergent() {
        // Two honest-close fingerprints plus one model-swap (disjoint top-k):
        // the swap agrees with neither, so the largest clique is 2, not 3.
        let mut fps = vec![
            ToplocFingerprint::from_topk(&[(1, 100.0), (3, 200.0), (5, 50.0)]),
            ToplocFingerprint::from_topk(&[(1, 101.0), (3, 202.0), (5, 50.5)]),
        ];
        let swap = ToplocFingerprint::from_topk(&[(40, 9.0), (41, 8.0), (42, 7.0)]);
        fps.push(swap);
        assert_eq!(
            largest_agreeing_cluster(&fps),
            2,
            "the swap joins no clique"
        );
        // A 3-of-3 demand is rejected; a 2-of-3 majority still holds.
        assert!(!tolerant_quorum_accepts(&fps, 3));
        assert!(tolerant_quorum_accepts(&fps, 2));

        // All-divergent (three mutually-disjoint swaps) → no pair agrees → 1.
        let all_div = vec![
            ToplocFingerprint::from_topk(&[(0, 1.0), (1, 2.0)]),
            ToplocFingerprint::from_topk(&[(10, 1.0), (11, 2.0)]),
            ToplocFingerprint::from_topk(&[(20, 1.0), (21, 2.0)]),
        ];
        assert_eq!(largest_agreeing_cluster(&all_div), 1);
        assert!(!tolerant_quorum_accepts(&all_div, 2));
    }

    #[test]
    fn n2_clique_defeats_transitivity_straddle() {
        // A ≈ B and B ≈ C but A ≉ C: B's index set covers both A and C, but A and
        // C are mutually disjoint. A pivot-count anchored at B would (wrongly)
        // report a 3-way quorum; the clique is only 2.
        let a = ToplocFingerprint::from_topk(&[(0, 1.0), (1, 1.0), (2, 1.0)]);
        let b = ToplocFingerprint::from_topk(&[
            (0, 1.0),
            (1, 1.0),
            (2, 1.0),
            (3, 1.0),
            (4, 1.0),
            (5, 1.0),
        ]);
        let c = ToplocFingerprint::from_topk(&[(3, 1.0), (4, 1.0), (5, 1.0)]);
        assert!(fingerprints_agree(&a, &b), "A and B agree");
        assert!(fingerprints_agree(&b, &c), "B and C agree");
        assert!(
            !fingerprints_agree(&a, &c),
            "A and C are disjoint → disagree"
        );
        assert_eq!(
            largest_agreeing_cluster(&[a, b, c]),
            2,
            "the straddler must not inflate a 3-way clique"
        );
    }

    #[test]
    fn n2_quorum_edge_cases() {
        // Empty → no agreement.
        assert_eq!(largest_agreeing_cluster(&[]), 0);
        assert!(!tolerant_quorum_accepts(&[], 1));
        assert!(tolerant_quorum_accepts(&[], 0), "min_agree 0 is vacuous");
        // Singleton → a self-clique of 1; a 2-of-N demand fails.
        let one = vec![ToplocFingerprint::from_topk(&[(1, 1.0)])];
        assert_eq!(largest_agreeing_cluster(&one), 1);
        assert!(tolerant_quorum_accepts(&one, 1));
        assert!(!tolerant_quorum_accepts(&one, 2));
    }
}
