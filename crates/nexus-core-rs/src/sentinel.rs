// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase I — SENTINEL forward-EMA inter-stage corruption localiser.
//!
//! SENTINEL (arXiv 2603.03592) localises *which* pipeline stage corrupted a
//! computation by tracking a per-stage exponential moving average (EMA) of an
//! activation statistic and flagging the stage whose current value deviates from
//! its running baseline. The original is a **training** detector (it tracks both
//! the forward activations *and* the backward gradients, with separate momenta
//! β_h / β_g). SBFB sharding is **inference, forward-only**: there is no backward
//! pass, so only the forward-activation half of SENTINEL is portable. This module
//! implements exactly that half over the inter-stage frontier signals of one
//! forward run.
//!
//! ## O(1) direct localisation — NOT a bisection
//!
//! Each frontier's signal is checked against the running EMA in **O(1)**, so a
//! corrupt stage is flagged the moment it is observed — there is no interactive
//! binary search. A true opML bisection (interactive fraud-proof reducing a
//! dispute to one instruction, [`crate::activation_commit`]) is **O(log L)** and
//! a separate mechanism; conflating "O(1)" with "bisection" is a category error.
//! SENTINEL answers *which frontier to dispute*; the commit-reveal of
//! [`crate::activation_commit`] then non-repudiably anchors that frontier.
//!
//! ## All-integer EMA (no float)
//!
//! The EMA runs in integer **basis points** (`alpha_bp / 10000`), so it
//! round-trips bit-identically across platforms — the same no-float discipline
//! as [`crate::shard_plan::RunMetrics`] and the
//! [`crate::verifiable_draw`] basis-point selection. The frontier signal is an
//! integer activation statistic (e.g. an L1 norm `Σ|activation_i|` quantised
//! like [`crate::toploc::bf16_bits`]); how it is extracted from the GPU is the
//! worker backend's job (Phase F), not this pure detector's.
//!
//! ## Robustness + honest limits (THREAT_MODEL §16 N3)
//!
//! A flagged (anomalous) frontier does **not** update the EMA baseline, so a
//! single transient spike cannot pull the baseline toward itself and desync the
//! rest of the pipeline (an outlier-rejecting trend, the spirit of SENTINEL's
//! Tukey fence). What it does NOT defend against is a **slow drift** that stays
//! just under threshold every step (SI-11 baseline poisoning) and an adversary
//! that knows the fixed threshold: the threshold is a static security parameter
//! here, not the adaptive IQR fence of the paper, and re-calibration on the real
//! rig is a tracked S78 carry. Like the N1 anti-lazy-verifier gap (Phase H), this is a
//! disclosed mitigation, not a guarantee; a cryptographic guarantee is N4 zkML
//! (out of scope).

/// Basis-point denominator for the EMA and the deviation threshold (10 000 = 100%).
pub const SENTINEL_BP_DENOMINATOR: u128 = 10_000;

/// EMA smoothing factor in basis points. 9000 (= 0.9) weights the running
/// baseline heavily so a single frontier cannot swing it — the forward momentum
/// β_h the SENTINEL paper uses.
pub const SENTINEL_ALPHA_BP: u128 = 9_000;

/// Relative deviation, in basis points of the EMA, at or above which a frontier
/// is flagged as corrupt: `|signal - ema| * 10000 >= THRESH * ema`. 5000 bp =
/// 50% — an abrupt half-or-more jump in the inter-stage activation statistic. A
/// fixed integer threshold (calibration on the real rig is a tracked S78 carry).
pub const SENTINEL_DEVIATION_THRESH_BP: u128 = 5_000;

/// One integer EMA step in basis points:
/// `ema_next = (alpha_bp * sample + (10000 - alpha_bp) * ema_prev) / 10000`.
///
/// `saturating_*` keeps it total even for pathological `u128` inputs (a real
/// activation statistic is far from the saturation boundary); the division by
/// the basis-point denominator is the only rounding, toward zero.
#[must_use]
pub fn ema_step(ema_prev: u128, sample: u128, alpha_bp: u128) -> u128 {
    let weighted_sample = alpha_bp.saturating_mul(sample);
    let weighted_prev = SENTINEL_BP_DENOMINATOR
        .saturating_sub(alpha_bp)
        .saturating_mul(ema_prev);
    weighted_sample.saturating_add(weighted_prev) / SENTINEL_BP_DENOMINATOR
}

/// Streaming forward-EMA monitor for one pipeline's inter-stage frontier signals.
///
/// Feed it one integer activation statistic per frontier with [`observe`]; it
/// reports in **O(1)** whether that frontier deviates from the running baseline.
/// The first frontier seeds the baseline (warmup); a flagged frontier does not
/// update it (outlier rejection).
///
/// [`observe`]: SentinelMonitor::observe
#[derive(Debug, Clone)]
pub struct SentinelMonitor {
    ema: Option<u128>,
    alpha_bp: u128,
    thresh_bp: u128,
}

impl Default for SentinelMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl SentinelMonitor {
    /// A monitor with the default [`SENTINEL_ALPHA_BP`] / [`SENTINEL_DEVIATION_THRESH_BP`].
    #[must_use]
    pub fn new() -> Self {
        Self::with_params(SENTINEL_ALPHA_BP, SENTINEL_DEVIATION_THRESH_BP)
    }

    /// A monitor with explicit smoothing / threshold basis points (for tests and
    /// rig calibration). `alpha_bp` is clamped to `[0, 10000]`.
    #[must_use]
    pub fn with_params(alpha_bp: u128, thresh_bp: u128) -> Self {
        SentinelMonitor {
            ema: None,
            alpha_bp: alpha_bp.min(SENTINEL_BP_DENOMINATOR),
            thresh_bp,
        }
    }

    /// The current baseline EMA, or `None` before the first frontier.
    #[must_use]
    pub fn baseline(&self) -> Option<u128> {
        self.ema
    }

    /// Observe one frontier's forward activation statistic. Returns `true` iff it
    /// deviates from the running baseline beyond the threshold. O(1).
    ///
    /// The first call seeds the baseline and never flags (warmup). A flagged
    /// frontier does NOT update the baseline (so a transient spike cannot poison
    /// the rest); a within-tolerance frontier folds into the EMA.
    pub fn observe(&mut self, signal: u128) -> bool {
        match self.ema {
            None => {
                self.ema = Some(signal);
                false
            }
            Some(ema) => {
                // |signal - ema| * DENOM >= thresh_bp * ema, all-integer (a
                // relative test so it is scale-invariant across layer depths).
                let deviates = signal.abs_diff(ema).saturating_mul(SENTINEL_BP_DENOMINATOR)
                    >= self.thresh_bp.saturating_mul(ema);
                if !deviates {
                    self.ema = Some(ema_step(ema, signal, self.alpha_bp));
                }
                deviates
            }
        }
    }
}

/// Locate the first corrupt frontier in a forward run from its ordered per-frontier
/// activation statistics, using the default [`SentinelMonitor`].
///
/// Returns the index of the first frontier whose signal deviates from the running
/// EMA beyond [`SENTINEL_DEVIATION_THRESH_BP`], or `None` if the whole pipeline
/// stays within tolerance. Direct O(1)-per-frontier localisation — never an
/// O(log L) bisection.
#[must_use]
pub fn localize_corrupted_frontier(signals: &[u128]) -> Option<usize> {
    let mut monitor = SentinelMonitor::new();
    for (index, &signal) in signals.iter().enumerate() {
        if monitor.observe(signal) {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentinel_ema_step_is_integer_no_float() {
        // ema_next = (9000*102 + 1000*100) / 10000 = (918000 + 100000)/10000 = 101.
        assert_eq!(ema_step(100, 102, SENTINEL_ALPHA_BP), 101);
        // A steady signal is a fixed point of the EMA.
        assert_eq!(ema_step(101, 101, SENTINEL_ALPHA_BP), 101);
        // alpha 0 → baseline never moves; alpha 10000 → snaps to the sample.
        assert_eq!(ema_step(100, 999, 0), 100);
        assert_eq!(ema_step(100, 999, SENTINEL_BP_DENOMINATOR), 999);
        // Saturating: a pathological u128 sample does not panic.
        let _ = ema_step(u128::MAX, u128::MAX, SENTINEL_ALPHA_BP);
    }

    #[test]
    fn n3_sentinel_localizes_corrupted_stage() {
        // A smooth pipeline with one corrupt frontier (index 4: a 5× spike).
        let signals = [100u128, 102, 101, 99, 500, 103];
        assert_eq!(
            localize_corrupted_frontier(&signals),
            Some(4),
            "SENTINEL must flag exactly the corrupt frontier (direct O(1), not bisection)"
        );

        // A fully healthy pipeline (smooth drift within tolerance) flags nothing.
        let healthy = [100u128, 102, 101, 99, 103, 104, 100];
        assert_eq!(localize_corrupted_frontier(&healthy), None);

        // The corrupt frontier is pinned exactly — neither neighbour is flagged.
        let early = [100u128, 100, 700, 100, 100];
        assert_eq!(localize_corrupted_frontier(&early), Some(2));
    }

    #[test]
    fn sentinel_warmup_and_outlier_does_not_poison_baseline() {
        let mut m = SentinelMonitor::new();
        // Warmup: first frontier seeds the baseline, never flags.
        assert!(!m.observe(100));
        assert_eq!(m.baseline(), Some(100));
        // A within-tolerance frontier folds into the EMA.
        assert!(!m.observe(102));
        assert_eq!(m.baseline(), Some(101));
        // A spike flags AND is rejected from the baseline (no poisoning): the
        // baseline is unchanged by the flagged sample.
        let before = m.baseline();
        assert!(m.observe(1000));
        assert_eq!(
            m.baseline(),
            before,
            "a flagged outlier must not update the EMA baseline"
        );
        // The next in-trend frontier is still measured against the clean baseline.
        assert!(!m.observe(101));
    }

    #[test]
    fn sentinel_short_inputs_flag_nothing() {
        // Fewer than two frontiers cannot deviate (no baseline to test against).
        assert_eq!(localize_corrupted_frontier(&[]), None);
        assert_eq!(localize_corrupted_frontier(&[42]), None);
    }

    #[test]
    fn sentinel_threshold_boundary_is_inclusive_reject() {
        // Deviation exactly at the threshold (50% of a 100 baseline = 50) flags
        // (the comparison is `>=`), one below does not.
        let mut at = SentinelMonitor::new();
        assert!(!at.observe(100));
        assert!(at.observe(150), "exactly +50% flags (>=)");

        let mut under = SentinelMonitor::new();
        assert!(!under.observe(100));
        assert!(!under.observe(149), "just under +50% does not flag");
    }
}
