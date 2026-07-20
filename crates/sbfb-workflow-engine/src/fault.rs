// SPDX-License-Identifier: AGPL-3.0-or-later
//! Falsifiability surface: crash points and red-control knobs.
//!
//! A durable-execution suite that only ever runs the happy path is
//! indistinguishable from one that asserts nothing. Each of the three ENGINE
//! guarantees — step-hash verification, intent atomicity, completion
//! atomicity — therefore has a switch that REMOVES it, so a test can assert
//! the bad state is observed when the guarantee is off; that assertion is
//! what proves the green arm is not vacuous. (Target-side deduplication has
//! its own removal arm, the lying target.) Guarantees that are NOT switchable
//! are the ones enforced by SQLite itself — a unique index, a foreign key —
//! where the removal arm would test the database, not this crate.
//!
//! The defaults are the guaranteed behaviour: [`NoFaults`] never fires and
//! [`SpikeKnobs::guaranteed`] enables every check. There is no way to build
//! a degraded [`SpikeKnobs`] implicitly — each degraded constructor is named
//! after the guarantee it removes.

/// A point in the engine where the process can be interrupted.
///
/// The names are the crash frontiers of the spike, in execution order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FaultPoint {
    /// INSIDE the guaranteed intent transaction, between the journal append
    /// and the outbox insert. Halting here must roll BOTH back.
    ///
    /// This point is what makes atomicity falsifiable. Injecting only in the
    /// degraded path would leave the guaranteed path with no window to crash
    /// in, and a test that never crashes between the two writes cannot tell
    /// one transaction from two: splitting `record_intent_atomic` in half
    /// would keep the whole suite green.
    InsideIntentTransaction,
    /// INSIDE the completion transaction, between the journal append and the
    /// outbox transition to `applied`. Same role as
    /// [`FaultPoint::InsideIntentTransaction`], for the other composite
    /// write: without it, "every guarantee carries a red control" would be
    /// false for the completion path.
    InsideCompletionTransaction,
    /// Between the outbox insert and the journal append — a window that only
    /// EXISTS when the atomic-outbox guarantee is switched off. Unreachable
    /// on the guaranteed path, where both writes are one transaction.
    BetweenOutboxAndIntentEvent,
    /// B1 — intent is durable, the effect has not been attempted.
    AfterIntentBeforeEffect,
    /// B2 — the effect was applied, its result is not journaled yet. The
    /// irreducible frontier.
    AfterEffectBeforeResult,
    /// B3 — the result is journaled, the run has not advanced past the step.
    AfterResult,
}

/// What the engine should do when it reaches a fault point.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultAction {
    /// Keep going (the production answer).
    Continue,
    /// Stop the run here with a typed error, simulating an in-process death
    /// without unwinding through the store.
    Halt,
}

/// Decides what happens at each [`FaultPoint`].
///
/// A hook that never returns (a real process exit) is a valid
/// implementation: that is how the out-of-process arm of the harness kills a
/// writer mid-run.
pub trait FaultHook {
    /// Called by the engine at every fault point it reaches.
    ///
    /// A hook may observe, halt, or kill the process — it must NOT call back
    /// into the store: [`FaultPoint::InsideIntentTransaction`] fires with a
    /// transaction open, and SQLite refuses a nested `BEGIN IMMEDIATE`.
    fn hit(&self, point: FaultPoint) -> FaultAction;
}

/// The production hook: nothing ever fires.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoFaults;

impl FaultHook for NoFaults {
    fn hit(&self, _point: FaultPoint) -> FaultAction {
        FaultAction::Continue
    }
}

/// Engine-side guarantees that a red control can remove.
///
/// Both guarantees default to on, and the fields are PRIVATE: a degraded
/// value is only reachable through an explicitly named constructor, so
/// neither `SpikeKnobs::default()` nor a struct literal can silently disable
/// a guarantee. (The fields were public in the first cut, which made the
/// "no way to build a degraded SpikeKnobs implicitly" claim false — a struct
/// literal walked straight past the named constructors.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpikeKnobs {
    verify_step_hash: bool,
    atomic_outbox: bool,
    atomic_completion: bool,
}

impl SpikeKnobs {
    /// Whether the memoised step request hash is re-checked on replay.
    pub const fn verify_step_hash(&self) -> bool {
        self.verify_step_hash
    }

    /// Whether the intent event and its outbox row are one transaction.
    pub const fn atomic_outbox(&self) -> bool {
        self.atomic_outbox
    }

    /// Whether the completion entry and its outbox transition are one
    /// transaction.
    pub const fn atomic_completion(&self) -> bool {
        self.atomic_completion
    }
    /// Every guarantee on. The only configuration a caller should run.
    pub const fn guaranteed() -> Self {
        Self {
            verify_step_hash: true,
            atomic_outbox: true,
            atomic_completion: true,
        }
    }

    /// RED CONTROL — stop verifying the replayed step request hash. Proves
    /// the fail-closed arm is load-bearing (with the check off, a divergent
    /// replay silently continues).
    ///
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub const fn without_step_hash_verification() -> Self {
        Self {
            verify_step_hash: false,
            ..Self::guaranteed()
        }
    }

    /// RED CONTROL — split the intent event and its outbox row into two
    /// transactions. Proves the atomic path is load-bearing (with the split,
    /// a crash in between leaves an orphan outbox row).
    ///
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub const fn without_atomic_outbox() -> Self {
        Self {
            atomic_outbox: false,
            ..Self::guaranteed()
        }
    }

    /// RED CONTROL — split the completion entry and its outbox transition
    /// into two transactions. Without it the completion path had a crash
    /// point but no arm REMOVING the guarantee, so the module claim below
    /// was false for that path.
    ///
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub const fn without_atomic_completion() -> Self {
        Self {
            atomic_completion: false,
            ..Self::guaranteed()
        }
    }
}

impl Default for SpikeKnobs {
    fn default() -> Self {
        Self::guaranteed()
    }
}
