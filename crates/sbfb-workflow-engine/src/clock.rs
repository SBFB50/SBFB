// SPDX-License-Identifier: AGPL-3.0-or-later
//! The host-neutral clock trait (Sprint 83 Phase J).
//!
//! The crate was ALREADY clockless: every store method that needs a
//! wall-clock value takes `now_ms` from its caller (store.rs module doc).
//! This trait is the promotion of that injection pattern to the scheduler
//! level, so the orchestrator can be driven by a manual clock in tests and
//! by the host's real clock in production — the REDUCER itself never reads
//! time at all.
//!
//! ## What the journal never learns
//!
//! Clock readings reach exactly two places: durable STATE tables (timer
//! deadlines, lease expiry, retention ages) and decisions that are
//! journaled AS DECISIONS (`cancel_requested` on a timeout). Replay reads
//! the journal, never the clock — the same run reduces to the same state on
//! a machine whose clock disagrees (THREAT §19 timer-skew mitigation).
//!
//! ## Regression policy
//!
//! A clock that goes BACKWARDS below a value the scheduler already acted on
//! is refused loudly by the orchestrator (`clock_regression_loud` test),
//! never clamped silently: a regressive clock re-arming an already-fired
//! timer or resurrecting an expired signal is the tampering class this
//! refusal closes.

/// A source of wall-clock milliseconds. Host-neutral: the engine never
/// constructs one itself.
pub trait Clock {
    /// Current wall-clock time in milliseconds since an arbitrary epoch
    /// (the host decides; only monotonicity ACROSS CALLS matters to the
    /// scheduler, which journals every decision it takes on a reading).
    fn now_ms(&self) -> i64;
}

/// A manually-driven clock for deterministic tests — public because the
/// out-of-crate harness (and later host wiring code) needs a controllable
/// clock, exactly like `FileEchoTarget` is public for the same reason.
#[derive(Debug)]
pub struct ManualClock {
    now_ms: std::cell::Cell<i64>,
}

impl ManualClock {
    /// A clock frozen at `now_ms`.
    pub fn at(now_ms: i64) -> Self {
        Self {
            now_ms: std::cell::Cell::new(now_ms),
        }
    }

    /// Move the clock to an absolute value. Moving it BACKWARDS is allowed
    /// here — the orchestrator's regression refusal is exactly what a test
    /// needs to exercise.
    pub fn set(&self, now_ms: i64) {
        self.now_ms.set(now_ms);
    }

    /// Advance the clock by `delta_ms`.
    pub fn advance(&self, delta_ms: i64) {
        self.now_ms.set(self.now_ms.get() + delta_ms);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> i64 {
        self.now_ms.get()
    }
}
