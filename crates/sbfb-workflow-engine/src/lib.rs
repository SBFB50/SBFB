// SPDX-License-Identifier: AGPL-3.0-or-later
//! # sbfb-workflow-engine — host-neutral durable-execution engine
//!
//! This crate is the IMPURE half of the two-crate durable-execution core
//! (kickoff D3): the reducer, the append-only run journal, the transactional
//! outbox and the effect targets live here, while
//! [`sbfb-workflow-core`](../sbfb_workflow_core/index.html) stays pure
//! (schema identities, canonicalisation, static validation — no database).
//!
//! Three layers, one store: the Phase E SPIKE ([`engine::SpikeEngine`],
//! linear runs — kept intact as the frozen proof of the durable core), the
//! Phase I FULL RUN STORE ([`store::RunStore`] — lease fencing, inbox,
//! snapshots, GC, backup/restore), and the Phase J ORCHESTRATOR
//! ([`orchestrator::Orchestrator`] + [`reducer::reduce`] — the
//! deterministic graph scheduler over `CompiledPlan.v1`, §12.3 states as a
//! pure reduction of the journal, durable timers, mono-usage signals,
//! honest cancellation).
//!
//! ## What Sprint 83 Phase E proves, and what it does not
//!
//! Phase E is a BLOCKING spike: its job is to make the durable-execution
//! guarantee falsifiable BEFORE any execution format is frozen. Stating that
//! guarantee precisely matters more than stating it strongly — an earlier
//! wording said "at most ONE unconfirmed ATTEMPT per key", which is FALSE:
//! after a crash at B2 the effect has already been attempted, and every
//! resume calls the target again. What is bounded is the INTENT, not the
//! number of attempts. Three clauses, in decreasing order of what this crate
//! can promise on its own:
//!
//! 1. **Engine clause (proven here)** — at most ONE durable INTENT exists per
//!    idempotency key `(run_id, node_instance_id, effect_id)`, and every
//!    re-emission reuses THAT SAME KEY: the engine never mints a fresh one on
//!    resume. The journal entry recording the intent and the outbox row
//!    carrying the effect are written in the SAME SQLite transaction, so no
//!    state exists where one is durable and the other is not — and the same
//!    holds for the completion write.
//!    What is promised is RE-EMISSION under a stable key, not delivery: the
//!    engine re-attempts, it cannot make the target succeed (`apply` may
//!    return an error every time). Against a non-idempotent target it does
//!    not even re-attempt — AT MOST ONE attempt, possibly ZERO, since a crash
//!    at B1 (intent durable, effect not yet attempted) resumes straight into
//!    `outcome_unknown` without calling the target at all. No liveness claim
//!    is made here.
//! 2. **Target clause** — "one observable side output" is only true against a
//!    target that DEDUPLICATES on that key, in the same durable backend. What
//!    the engine actually enforces is equality of the SCOPE TOKEN the target
//!    declares ([`echo::EffectTarget::dedup_scope`]), journaled at intent
//!    time — two honest targets can share a name and deduplicate in two
//!    different stores, and that is what this catches. Making the token
//!    globally discriminating is the TARGET's responsibility: the spike's
//!    file target returns its path as injected, so the same relative path
//!    used from two working directories would compare equal while pointing
//!    at two files. [`echo::FileEchoTarget`]
//!    implements that deduplication, and the lying-target arm is the red
//!    control proving the property comes from the target, not from wishful
//!    thinking.
//! 3. **Concurrency clause** — clause 2 holds in a SINGLE-WRITER SERIALISED
//!    model only. `FileEchoTarget` deduplicates by read-then-append without a
//!    lock, so two writers resuming the same run concurrently can both read
//!    "absent" and both append. Single-writer fencing was out of scope for
//!    the SPIKE, which is exactly why the model was named here instead of
//!    being left implicit in a "not proven" list — and Phase I delivered it:
//!    a store handle that acquired the lease re-asserts its monotone epoch
//!    inside EVERY write transaction ([`store::RunStore::acquire_lease`]),
//!    so a fenced-out writer emits nothing. An UNFENCED handle (no lease
//!    acquired) still writes in the spike's single-process model; fencing is
//!    an acquired posture, not an ambient default. And the fence covers the
//!    STORE's writes, not the external-effect window: a writer suspended
//!    between its committed intent and `target.apply`, deposed meanwhile,
//!    can still reach the target — that window stays bounded by the
//!    TARGET's deduplication (clause 2) until the effect layer verifies the
//!    fencing token target-side (Phase K).
//!
//! The irreducible frontier is B2 (the effect was applied, its result was
//! not yet journaled). A crash there leaves the engine unable to know
//! whether the side output happened. Against an idempotent target the
//! engine re-applies with the same key and the target collapses it — note
//! that this is a SECOND attempt, and a run crashing at B2 repeatedly makes
//! N of them; what stays bounded is the intent and the key. Against a target
//! that does NOT declare idempotency the engine refuses to retry and ends
//! the run in the terminal `outcome_unknown` state
//! ([`event::TerminalReason::OutcomeUnknown`]) — never an invented success.
//!
//! **Durability is claimed as logic-only.** The store opens with
//! `journal_mode=WAL` + `synchronous=FULL` (contract asserted by a test),
//! but no crash harness can prove `fsync`: SQLite commits are durable across
//! APPLICATION crashes whatever the `synchronous` setting, and only a power
//! loss discriminates. The harness therefore proves recovery logic and
//! transaction atomicity, and the pragma contract is the (weaker) durability
//! evidence. This is stated in the phase artifact as
//! `"durability_claim": "logic-only"` rather than glossed over.
//!
//! ## Charter — host-neutral
//!
//! The crate resolves NO home directory, reads NO environment variable and
//! embeds NO path literal: every path (the SQLite file, the echo target's
//! side-output log) is INJECTED by the caller. That keeps the engine usable
//! from a daemon, a test, or a CLI without a hidden host contract, and it is
//! the same discipline the S83 A1 TEST-ISOLATION carry closed elsewhere.
//!
//! ## Falsifiability surface
//!
//! [`fault::FaultHook`] and [`fault::SpikeKnobs`] exist so each of the three
//! ENGINE guarantees has a RED CONTROL: an arm where the guarantee is
//! switched off and the test asserts the bad state IS observed. (Target-side
//! deduplication has its own arm, the lying target; invariants enforced by
//! SQLite itself have none — removing them would test the database.) Without that, a green suite proves
//! nothing (a vacuously-passing test looks identical to a real one). The
//! defaults are the guaranteed behaviour — [`fault::NoFaults`] never fires
//! and [`fault::SpikeKnobs::guaranteed`] enables every check.
//!
//! ## Sprint 83 Phase I — the full run store
//!
//! On top of the spike, Phase I delivered the COMPLETE run store in
//! [`store`]: interpretation-time upcasting with a fail-closed downgrade
//! guard at BOTH levels (schema `user_version` at open, event
//! `format_version` at read), single-writer lease fencing (monotone epoch
//! CAS inside every write transaction — wall clock is only the takeover
//! heuristic), a deduplicating inbox, disposable digest-checked snapshots,
//! policy-bounded retention/GC that never touches a running run, a pending
//! outbox row or a terminal diagnosis, and `VACUUM INTO` backup with a
//! validated restore. The `RunEvent.v1` envelope moved to
//! `sbfb_workflow_core::run_event` (the spike's "keep it local while
//! GO/NO-GO is open" rationale expired with the GO), and [`event`]
//! re-exports it. Phase I also closed the E harness debt: every PRODUCTION
//! entry point ([`SpikeEngine::new`], [`SpikeEngine::execute`], the store's
//! `record_*`) takes no fault hook and no knobs — the falsifiability
//! surface lives behind `spike-faults` alone.
//!
//! ## Pre-launch protocol
//!
//! [`event::RUN_EVENT_FORMAT_VERSION`] is a NEW constant, not a bump of any
//! existing wire format.
//!
//! Whether this crate carries a documented frontier is settled by the
//! TEST-ACTOR question — who READS this primitive? — and by nothing else.
//! The answer here: the Rust of this crate and its tests, plus a LATER
//! PROCESS RUNNING THE SAME CODE (that is what the store file and the
//! side-output log are). No other node, no external client, no sandboxed
//! app, no other model. Hence no frontier tag and no contract-doc entry.
//! Accessorily — never as the criterion itself — nothing here leaves the
//! node, so the crate declares no `DOMAIN_*` signing family and leaves the
//! frozen `DOMAIN_*_V1` census where it stands (29 since S83 Phase F; cf.
//! `CLAUDE.md §Pre-launch protocol policy`). Taking that second fact FOR the
//! criterion is the conflation that once let real frontiers go undocumented
//! for a whole sprint; the order of the two sentences here is deliberate.
//!
//! The journal is digest-only by TYPE: [`event::RunEvent`] can carry an
//! effect id and a receipt DIGEST, never receipt content. Material payloads
//! live in the outbox row keyed by the idempotency key, so a journal that
//! outlives the process cannot leak prompt or output content.

pub mod clock;
pub mod echo;
pub mod engine;
pub mod event;
pub mod fault;
pub mod hash;
pub mod orchestrator;
pub mod reducer;
pub mod runtime_id;
pub mod signal;
pub mod store;

pub use clock::{Clock, ManualClock};
pub use engine::{EngineError, RunOutcome, RunSpec, SpikeEngine, StepSpec};
pub use event::{
    CancelOrigin, RUN_EVENT_FORMAT_VERSION, RunEvent, RunEventKind, TerminalReason, UpcastError,
};
pub use orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorError, RunProgress};
pub use reducer::{NodeState, ReduceError, RunState, RunView, reduce};
pub use signal::{NoSignals, ScriptedSignals, SignalDeposit, SignalSource};
// The falsifiability surface is harness-only since Phase I: no production
// entry point takes a hook or knobs, so the types are only re-exported at
// the top level for the `spike-faults` test build. (`fault` stays a public
// module for the error variants that name a `FaultPoint`.)
#[cfg(feature = "spike-faults")]
pub use fault::{FaultHook, FaultPoint, NoFaults, SpikeKnobs};
pub use hash::{IdempotencyKey, StepRequest, canonical_digest, step_request_hash};
pub use store::{
    GcOutcome, LeaseState, RetentionPolicy, RunStatus, RunStore, Snapshot, SnapshotRead, StoreError,
};
