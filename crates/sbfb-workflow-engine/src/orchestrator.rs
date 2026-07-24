// SPDX-License-Identifier: AGPL-3.0-or-later
//! The local deterministic orchestrator (Sprint 83 Phase J): the scheduler
//! that walks a `CompiledPlan.v1` graph over the durable run store.
//!
//! ## Authority and shape
//!
//! The run's authority is the INITIATING NODE (WA-3): one local
//! orchestrator owns the plan, the journal, the frontier, the timers and
//! the interruptions. It is synchronous and SINGLE-WRITER — it acquires
//! the store lease before writing, and every write transaction re-asserts
//! the fencing epoch, so a zombie orchestrator from a previous incarnation
//! emits NOTHING after a takeover.
//!
//! The scheduler only ever DISPATCHES nodes of the compiled plan — a node
//! that is not a plan occurrence cannot be executed, which is the runtime
//! half of "the signed graph IS the plan" (T-PLAN-LLM-MUTATION's scheduler
//! part). The effective-capability COMPUTATION is the [`crate::capability`]
//! module (the monotone intersection of the seven authority layers); this
//! scheduler does not yet consult it per effect.
//!
//! ## Determinism
//!
//! Everything the walk decides is either DERIVED from the plan topology
//! (reducer) or JOURNALED BEFORE it is acted on (`branch_taken`,
//! `map_fanout`, `join_fired`, attempt failures, parks, consumptions,
//! cancellation). The ready-set is dispatched in canonical id order under
//! a bounded in-flight cap — admission order is a pure function of the
//! journal, never of a HashMap or a clock. Retry BACKOFF is durable
//! pacing (`er:` timers): it changes WHEN a retry dispatches, never what
//! any outcome is.
//!
//! ## Value plumbing
//!
//! Material values flow OUTSIDE the journal: effect receipts live in the
//! outbox, signal payloads in the signal table, the run input in the runs
//! row. A node's effect input is the map `{inbound port → canonical JCS of
//! the port's value}`; the echo receipt is the JCS of that map, and the
//! output resolution inverts it (single-port maps collapse to the port's
//! value, so a value passes through a leaf chain UNCHANGED — what makes
//! the golden fixtures' pre-seeded routing fields reach their switch).
//! A structured activity receipt (Phase K) carries its `output` member
//! instead; a terminally-failed leaf resolves to a deterministic error
//! object on its `error` edge.
//!
//! ## The effect layer (Sprint 83 Phase K)
//!
//! [`Orchestrator::execute`] drives the FROZEN blocking Phase E/J walk
//! (echo target, journal behaviour pinned by the golden fixtures);
//! [`Orchestrator::execute_activity`] drives the non-blocking
//! [`Activity`] adapter and owns what J deferred: per-attempt deadlines
//! from the plan's `default_node_timeout_seconds`, journaled failures
//! under the scope's `max_retries_per_node`, `on_node_failure` routing
//! (`fail-workflow` closes as `node_failed`; `route-error-port` delivers
//! the reserved `error` edge), the §12.5 unknowable outcome (terminal, or
//! PARKED for an operator [`Orchestrator::reconcile_unknown`] when the
//! host policy says so), the §12.7 late-result routing on cancel, and the
//! fencing epoch presented target-side.
//!
//! ## What Phase K does NOT do (honest bounds)
//!
//! Capability attenuation is COMPUTED by the [`crate::capability`] module;
//! this effect layer does not itself consult that computation (binding it
//! into dispatch is the run-surface phase). The secret vault and the signed
//! HITL approval are L1, the rich provider attestation (Ed25519 receipt with
//! model/CLI facts) is the Phase O adapter layer. Budget METERING
//! (`cancel_on_budget_exhausted`) still has no meter. Failure REASONS
//! surface in errors, never in the journal (payload-free by type).

use std::collections::BTreeMap;

use sbfb_workflow_core::compile::{
    CompiledNodeInstance, CompiledPlan, ERROR_PORT, compiled_plan_digest,
};
use sbfb_workflow_core::graph::{JoinPolicy, NodeFailureAction, WorkflowNode};

use crate::activity::{Activity, ActivityError, ActivityReceipt, EffectContext, PollOutcome};
use crate::clock::Clock;
use crate::echo::{EffectTarget, TargetError};
use crate::engine::EngineError;
use crate::event::{
    CancelOrigin, ReconcileOrigin, ReconcileResolution, RunEventKind, TerminalReason,
};
use crate::hash::{
    HashError, IdempotencyKey, StepRequest, canonical_string, derive_effect_id, derive_wait_nonce,
    step_request_hash, validate_identity_component,
};
use crate::reducer::{
    DEFAULT_PORT, NodeState, REPEAT_PORT_CONTINUE, REPEAT_PORT_STOP, ReduceError, RunState,
    RunView, reduce,
};
use crate::runtime_id::{child_structural, extend_scope, parent_scope, structural_id};
use crate::signal::SignalSource;
use crate::store::{AttemptOutcome, OpenAttempt, PendingEffect, RunStatus, RunStore, StoreError};

/// Context string of the orchestrated run's spec digest: binds the run to
/// `(plan digest, input payload)`, so a resume with another plan or input
/// is refused where per-step hashes cannot see it. Local, never signed.
pub const ORCHESTRATED_SPEC_DIGEST_CONTEXT: &str = "sbfb.workflow.engine.orchestrated-spec.v0";

/// Timer id of the run-wide `workflow_timeout_seconds` deadline.
pub const WORKFLOW_TIMEOUT_TIMER_ID: &str = "workflow-timeout";

/// Input field a classifier's echo activity reads its proposed label from.
/// With the real model adapter (Phase K) the label comes from the model
/// OUTPUT; the echo target returns its input, so the two coincide — and
/// the closed-set validation below is the same either way (research §19.3:
/// a proposal outside `branch_labels` routes to `default`,
/// deterministically).
pub const CLASSIFIER_LABEL_FIELD: &str = "classifier_label";

/// The reserved signal channel a `human.review` wait listens on. Not a
/// pack-author name: node signal names come from the plan, this one from
/// the engine, and the `/` keeps it outside any single-segment name.
pub const HUMAN_DECISION_CHANNEL: &str = "human.review/decision";

/// Default bounded in-flight dispatch cap (backpressure). A HOST policy
/// knob — deliberately not a field of the signed plan, which has no
/// max-concurrency vocabulary; admission order stays canonical whatever
/// the cap, so the cap changes pacing, never outcomes.
pub const DEFAULT_MAX_IN_FLIGHT: usize = 8;

/// Default lease TTL an orchestrator asks for.
pub const DEFAULT_LEASE_TTL_MS: i64 = 30_000;

/// Timer-id prefix of the retry-backoff pacing timers (Phase K):
/// `er:<effect_id>#<failed attempt>`. Effect ids are 64 hex chars, so the
/// id stays far under the identity bound whatever the node id length.
pub const RETRY_TIMER_PREFIX: &str = "er:";

/// First retry backoff (doubles per failed attempt).
pub const RETRY_BACKOFF_BASE_MS: i64 = 1_000;

/// Backoff ceiling — pacing stays bounded (THREAT §19.2 retry-storm), and
/// pacing NEVER decides outcomes: only the journaled failure count and
/// the plan's retry budget do.
pub const RETRY_BACKOFF_MAX_MS: i64 = 60_000;

/// The bounded exponential backoff after `failed_attempts` failures.
fn backoff_ms(failed_attempts: u32) -> i64 {
    let shift = failed_attempts.saturating_sub(1).min(16);
    RETRY_BACKOFF_BASE_MS
        .saturating_mul(1_i64 << shift)
        .min(RETRY_BACKOFF_MAX_MS)
}

/// Pacing timer id of the retry after `failed_attempts` failures.
fn retry_timer_id(effect_id: &str, failed_attempts: u32) -> String {
    format!("{RETRY_TIMER_PREFIX}{effect_id}#{failed_attempts}")
}

/// Host-side configuration of one orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Lease owner identity of this writer.
    pub writer_id: String,
    /// Lease TTL (takeover heuristic horizon).
    pub lease_ttl_ms: i64,
    /// Bounded in-flight dispatch cap, ≥ 1.
    pub max_in_flight: usize,
    /// The reconciliation policy (§12.5/§12.7 "pas propagé sans politique
    /// explicite", Phase K): when `true`, an unknowable non-idempotent
    /// effect PARKS its node (`node_outcome_unknown`, run stays alive)
    /// for an operator reconciliation; when `false` (DEFAULT — the
    /// Phase J behaviour, unchanged) the run closes terminally as
    /// `outcome_unknown`. A HOST policy, deliberately not a field of the
    /// signed plan.
    pub park_unknown_for_reconciliation: bool,
}

impl OrchestratorConfig {
    /// A config with the default knobs for `writer_id`.
    pub fn new(writer_id: impl Into<String>) -> Self {
        Self {
            writer_id: writer_id.into(),
            lease_ttl_ms: DEFAULT_LEASE_TTL_MS,
            max_in_flight: DEFAULT_MAX_IN_FLIGHT,
            park_unknown_for_reconciliation: false,
        }
    }
}

/// What one `execute` call did and where the run stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunProgress {
    /// Run identity.
    pub run_id: String,
    /// Effects applied during THIS call.
    pub steps_executed: usize,
    /// Effects served from the journal without touching the target.
    pub steps_memoised: usize,
    /// The final reduction of this call.
    pub view: RunView,
}

/// The orchestrator refused to advance.
#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    /// Store failure (lease loss included).
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Canonicalisation failure.
    #[error(transparent)]
    Hash(#[from] HashError),
    /// Effect target failure — the run stays resumable.
    #[error(transparent)]
    Target(#[from] TargetError),
    /// Journal/plan disagreement.
    #[error(transparent)]
    Reduce(#[from] ReduceError),
    /// A step-level fail-closed refusal (hash divergence, idempotency or
    /// scope change, outcome unknown) — the Phase E vocabulary, reused.
    #[error(transparent)]
    Step(Box<EngineError>),
    /// The plan cannot be digested or is internally inconsistent with the
    /// walk (a structural id the plan does not contain).
    #[error("plan: {0}")]
    Plan(String),
    /// A resume presented a different plan or input than the recorded run.
    #[error("spec digest mismatch for run {run_id}: recorded {expected}, replayed {got}")]
    SpecDigestMismatch {
        /// Run identity.
        run_id: String,
        /// Digest recorded at creation.
        expected: String,
        /// Digest of this call's (plan, input).
        got: String,
    },
    /// The run already closed with a terminal diagnosis.
    #[error("run {run_id} is terminal: {reason}")]
    RunAlreadyTerminal {
        /// Run identity.
        run_id: String,
        /// Recorded reason.
        reason: String,
    },
    /// A map's runtime collection exceeds the STRUCTURAL bound of the
    /// signed plan — refused before anything is journaled (the bound is
    /// data of the plan, never stretched at runtime).
    #[error("map {node_instance_id}: collection of {len} exceeds max_cardinality {bound}")]
    MapCardinalityExceeded {
        /// The map node.
        node_instance_id: String,
        /// Runtime collection size.
        len: usize,
        /// Structural bound.
        bound: u32,
    },
    /// A node's routing/collection field is absent or ill-typed in its
    /// input payload.
    #[error("node {node_instance_id}: {reason}")]
    BadPayload {
        /// The node whose dispatch failed.
        node_instance_id: String,
        /// What was wrong.
        reason: String,
    },
    /// The activity adapter failed at its boundary (Phase K). A
    /// [`ActivityError::Fenced`] surfaces here: a deposed writer must
    /// stop, not journal.
    #[error(transparent)]
    Activity(#[from] ActivityError),
    /// A reconciliation was asked on a node that is not parked
    /// `outcome_unknown` (Phase K): nothing to reconcile — refusing loudly
    /// is what keeps the door from becoming a generic completion writer.
    #[error("node {node_instance_id} is not parked outcome_unknown")]
    NotParked {
        /// The node the operator named.
        node_instance_id: String,
    },
    /// A `result_recovered` reconciliation carried no receipt: recovery
    /// without evidence is the laundering K.4 forbids.
    #[error("result_recovered reconciliation requires the recovered receipt")]
    ReconcileNeedsReceipt,
    /// A `result_recovered` reconciliation carried a structured receipt
    /// bound to a DIFFERENT node/effect: a receipt for another effect is
    /// not evidence for this one.
    #[error("recovered receipt does not bind to {node_instance_id}'s effect")]
    ReconcileReceiptMismatch {
        /// The parked node the operator named.
        node_instance_id: String,
    },
    // NOTE: the clock-regression refusal (THREAT §19.1 timer skew) lives at
    // the STORE door — `StoreError::SignalRefused` from
    // `consume_signal_atomic` — because the deposit's durable
    // `deposited_at_ms` is what a regressive reading contradicts. Review r1
    // removed a dead orchestrator-level variant that claimed the guard
    // without carrying it.
}

impl From<EngineError> for OrchestratorError {
    fn from(e: EngineError) -> Self {
        Self::Step(Box::new(e))
    }
}

/// The effect adapter one `execute` call drives: the FROZEN blocking
/// target of the Phase E/J walk, or the Phase K non-blocking activity.
/// Private on purpose — the public surface is the two `execute` methods,
/// and the Target arm's journal behaviour is byte-frozen by the J golden
/// fixtures.
enum EffectDriver<'a> {
    /// The blocking spike-era target (echo). Journal behaviour identical
    /// to Phase J: intent → apply → completion, no attempts machinery.
    Target(&'a mut dyn EffectTarget),
    /// The Phase K non-blocking activity: attempts, retries/backoff,
    /// per-attempt deadlines, park/reconciliation, target-side fencing.
    Activity(&'a mut dyn Activity),
}

impl EffectDriver<'_> {
    fn name(&self) -> String {
        match self {
            Self::Target(t) => t.name().to_string(),
            Self::Activity(a) => a.name().to_string(),
        }
    }

    fn is_idempotent(&self) -> bool {
        match self {
            Self::Target(t) => t.is_idempotent(),
            Self::Activity(a) => a.is_idempotent(),
        }
    }

    fn dedup_scope(&self) -> String {
        match self {
            Self::Target(t) => t.dedup_scope(),
            Self::Activity(a) => a.dedup_scope(),
        }
    }
}

/// The local deterministic orchestrator over one run store.
#[derive(Debug)]
pub struct Orchestrator {
    store: RunStore,
    config: OrchestratorConfig,
}

impl Orchestrator {
    /// Build an orchestrator over an already-open store.
    pub fn new(store: RunStore, config: OrchestratorConfig) -> Self {
        Self { store, config }
    }

    /// Borrow the store (assertions, inspection).
    pub fn store(&self) -> &RunStore {
        &self.store
    }

    /// Mutably borrow the store.
    pub fn store_mut(&mut self) -> &mut RunStore {
        &mut self.store
    }

    /// Request a HOST-OPERATOR cancellation (§12.7). Always honoured —
    /// `allow_host_operator_cancel: false` in a pack is a claim the engine
    /// refuses to let opt out of operator authority, so no policy check
    /// guards this door. Returns `true` when THIS call journaled the
    /// request. The next `execute` call drains and closes the run.
    pub fn request_cancel(
        &mut self,
        run_id: &str,
        clock: &dyn Clock,
    ) -> Result<bool, OrchestratorError> {
        self.acquire(clock)?;
        Ok(self
            .store
            .record_cancel_requested(run_id, CancelOrigin::HostOperator)?)
    }

    fn acquire(&mut self, clock: &dyn Clock) -> Result<(), OrchestratorError> {
        let now = clock.now_ms();
        self.store
            .acquire_lease(&self.config.writer_id, now, self.config.lease_ttl_ms)?;
        Ok(())
    }

    /// Reconcile a PARKED `outcome_unknown` node (§12.5, Phase K) — an
    /// OPERATOR-LOCAL gesture, like restore/GC: no network route exposes
    /// it. Two resolutions, both journaled with a closed origin:
    ///
    /// - [`ReconcileResolution::Abandon`] — the outcome stays unknowable;
    ///   the run closes terminally as `outcome_unknown` (honest, never a
    ///   success).
    /// - [`ReconcileResolution::ResultRecovered`] — the operator
    ///   recovered the effect's ACTUAL receipt from the target's own
    ///   backend; it goes through the COMPLETION door (digest journaled,
    ///   frontier advanced), so the node's success rests on evidence.
    ///   The next `execute*` call continues downstream.
    ///
    /// Refused on a run that is not RUNNING (closed states are
    /// absorbing — reconciling a TERMINAL `outcome_unknown` is exactly
    /// the laundering K.4 forbids) and on a node that is not parked.
    pub fn reconcile_unknown(
        &mut self,
        run_id: &str,
        rid: &str,
        resolution: ReconcileResolution,
        recovered_receipt: Option<&str>,
        clock: &dyn Clock,
    ) -> Result<(), OrchestratorError> {
        self.acquire(clock)?;
        let now = clock.now_ms();
        self.store.advance_clock_watermark(now)?;

        let events = self.store.load_events(run_id)?;
        let parked = events.iter().any(|e| {
            e.node_instance_id.as_deref() == Some(rid)
                && matches!(e.kind, RunEventKind::NodeOutcomeUnknown { .. })
        });
        let completed = events.iter().any(|e| {
            e.node_instance_id.as_deref() == Some(rid)
                && matches!(e.kind, RunEventKind::EffectCompleted { .. })
        });
        if !parked || completed {
            return Err(OrchestratorError::NotParked {
                node_instance_id: rid.to_string(),
            });
        }
        let pending = self
            .store
            .pending_effect(run_id, rid)?
            .ok_or_else(|| OrchestratorError::Plan(format!("no pending intent for {rid}")))?;

        match resolution {
            ReconcileResolution::Abandon => {
                self.store.record_reconciliation(
                    run_id,
                    rid,
                    &pending.effect_id,
                    ReconcileOrigin::HostOperator,
                    ReconcileResolution::Abandon,
                )?;
                self.store.record_terminal(
                    run_id,
                    Some(rid),
                    TerminalReason::OutcomeUnknown,
                    None,
                    None,
                    Some(now),
                )?;
            }
            ReconcileResolution::ResultRecovered => {
                let receipt = recovered_receipt.ok_or(OrchestratorError::ReconcileNeedsReceipt)?;
                // Recovery is EVIDENCE-only: the operator MUST supply a
                // structured `ActivityReceipt` that PARSES and BINDS to this
                // exact node/effect. An opaque/empty/foreign string is not
                // evidence — it would launder an unknown into a success (the
                // completion door alone binds only the request hash, which a
                // hand-written blob can carry). A raw spike echo receipt is
                // NOT a valid recovery here: the reconcile door is stricter
                // than the normal in-run completion path on purpose.
                let parsed = ActivityReceipt::parse(receipt).ok_or_else(|| {
                    OrchestratorError::ReconcileReceiptMismatch {
                        node_instance_id: rid.to_string(),
                    }
                })?;
                if parsed.run_id != run_id
                    || parsed.node_instance_id != rid
                    || parsed.effect_id != pending.effect_id
                    || parsed.request_hash != pending.request_hash
                {
                    return Err(OrchestratorError::ReconcileReceiptMismatch {
                        node_instance_id: rid.to_string(),
                    });
                }
                self.store.record_reconciliation(
                    run_id,
                    rid,
                    &pending.effect_id,
                    ReconcileOrigin::HostOperator,
                    ReconcileResolution::ResultRecovered,
                )?;
                self.store.record_completion(
                    run_id,
                    rid,
                    &pending.effect_id,
                    &pending.request_hash,
                    receipt,
                )?;
            }
        }
        Ok(())
    }

    /// Run — or resume — `run_id` of `plan` with `input` against `target`,
    /// draining `signals`, until the run completes, parks on an open
    /// interruption, or closes. Calling it again with the same identifiers
    /// is the RESUME path: memoised effects are served from the journal
    /// and the target is not touched again (§12.6 — resuming never re-runs
    /// a confirmed effect).
    ///
    /// This is the Phase E/J BLOCKING walk, journal-frozen by the golden
    /// fixtures. The Phase K effect layer (retries, timeouts, park,
    /// target-side fencing) drives [`Orchestrator::execute_activity`].
    pub fn execute(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        input: &serde_json::Value,
        target: &mut dyn EffectTarget,
        clock: &dyn Clock,
        signals: &mut dyn SignalSource,
    ) -> Result<RunProgress, OrchestratorError> {
        self.execute_inner(
            plan,
            run_id,
            input,
            &mut EffectDriver::Target(target),
            clock,
            signals,
        )
    }

    /// Run — or resume — `run_id` against a NON-BLOCKING [`Activity`]
    /// (Sprint 83 Phase K). Same walk and same guarantees as
    /// [`Orchestrator::execute`], plus the effect layer: per-attempt
    /// deadlines from the plan's `default_node_timeout_seconds`,
    /// journaled attempt failures, bounded retry backoff under the
    /// scope's `max_retries_per_node`, the honest §12.5 unknowable
    /// outcome (terminal, or PARKED when
    /// [`OrchestratorConfig::park_unknown_for_reconciliation`] is set),
    /// the §12.7 late-result routing on cancel, and the fencing epoch
    /// propagated target-side. An in-flight effect returns the call with
    /// its node `Running` — drive again to poll.
    pub fn execute_activity(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        input: &serde_json::Value,
        activity: &mut dyn Activity,
        clock: &dyn Clock,
        signals: &mut dyn SignalSource,
    ) -> Result<RunProgress, OrchestratorError> {
        self.execute_inner(
            plan,
            run_id,
            input,
            &mut EffectDriver::Activity(activity),
            clock,
            signals,
        )
    }

    #[allow(clippy::too_many_lines)]
    fn execute_inner(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        input: &serde_json::Value,
        driver: &mut EffectDriver<'_>,
        clock: &dyn Clock,
        signals: &mut dyn SignalSource,
    ) -> Result<RunProgress, OrchestratorError> {
        validate_identity_component("run_id", run_id)?;
        let plan_digest =
            compiled_plan_digest(plan).map_err(|e| OrchestratorError::Plan(e.to_string()))?;
        let spec_digest = crate::hash::canonical_digest(
            ORCHESTRATED_SPEC_DIGEST_CONTEXT,
            &(plan_digest.as_str(), input),
        )?;

        self.acquire(clock)?;

        match self.store.run_status(run_id)? {
            Some(RunStatus::Terminal) => {
                return Err(OrchestratorError::RunAlreadyTerminal {
                    run_id: run_id.to_string(),
                    reason: self
                        .store
                        .terminal_reason(run_id)?
                        .unwrap_or_else(|| "unknown".to_string()),
                });
            }
            Some(RunStatus::Completed) | Some(RunStatus::Running) => {
                let recorded = self.store.spec_digest(run_id)?;
                if recorded.as_deref() != Some(spec_digest.as_str()) {
                    return Err(OrchestratorError::SpecDigestMismatch {
                        run_id: run_id.to_string(),
                        expected: recorded.unwrap_or_default(),
                        got: spec_digest,
                    });
                }
            }
            None => {}
        }

        let created = self
            .store
            .ensure_run_with_input(run_id, &spec_digest, input)?;
        if !created {
            // The row existed before this call — bind, even for the race
            // where the status read above saw nothing.
            let recorded = self.store.spec_digest(run_id)?;
            if recorded.as_deref() != Some(spec_digest.as_str()) {
                return Err(OrchestratorError::SpecDigestMismatch {
                    run_id: run_id.to_string(),
                    expected: recorded.unwrap_or_default(),
                    got: spec_digest,
                });
            }
        }

        // The run-wide timeout timer: an ABSOLUTE deadline computed ONCE at
        // creation from the plan's root policies — a restart re-reads it
        // (INSERT OR IGNORE), never re-arms it.
        // The watermark judges THIS reading before any deadline derives
        // from it (Codex r2 P1: a deadline computed from a reading the
        // watermark would refuse must never become the irreversible first
        // write).
        // ONE reading, judged then reused (Codex r3: a second
        // `clock.now_ms()` call after the watermark would let a
        // schizophrenic clock hand the deadline a regressive value the
        // watermark never saw — TOCTOU between judgement and use).
        let judged_now = clock.now_ms();
        self.store.advance_clock_watermark(judged_now)?;

        // Scheduled on EVERY call, not only at creation (Codex r1 P1): a
        // crash between run creation and this write would otherwise lose
        // the deadline forever — created=false on resume. INSERT OR IGNORE
        // keeps the FIRST successfully-written deadline authoritative; the
        // honest residue is that a pre-first-write crash re-derives the
        // deadline from the RESUME's clock (later than the original).
        let _ = created;
        if let (Some(cancel_policy), Some(timeout_policy)) = (
            plan.cancellation_policies.get(""),
            plan.timeout_policies.get(""),
        ) && cancel_policy.cancel_on_timeout
        {
            let deadline = judged_now.saturating_add(
                i64::from(timeout_policy.workflow_timeout_seconds.get()).saturating_mul(1000),
            );
            self.store
                .schedule_timer(run_id, WORKFLOW_TIMEOUT_TIMER_ID, deadline)?;
        }

        let mut executed = 0usize;
        let mut memoised = 0usize;

        loop {
            let now = clock.now_ms();
            // 0. The durable watermark refuses a REGRESSIVE reading for
            // the whole pass — before any expiry or deadline is judged
            // (THREAT §19.1; the operator-cancel door stays outside it:
            // operator authority must not be blockable by a broken clock).
            self.store.advance_clock_watermark(now)?;

            // 1. Durable timers first: a due deadline is a decision to
            // journal before anything else this pass. Retry-backoff
            // timers (`er:`) are PACING — they fire without journaling
            // (outcomes never depend on them, only dispatch timing does).
            for (timer_id, _deadline) in self.store.due_timers(run_id, now)? {
                if timer_id == WORKFLOW_TIMEOUT_TIMER_ID {
                    self.store.fire_timer_with_cancel(
                        run_id,
                        &timer_id,
                        CancelOrigin::WorkflowTimeout,
                    )?;
                } else if timer_id.starts_with(RETRY_TIMER_PREFIX) {
                    self.store.fire_timer_plain(run_id, &timer_id)?;
                }
            }

            // 2. Reduce.
            let events = self.store.load_events(run_id)?;
            let view = reduce(plan, &events)?;

            match view.run_state {
                RunState::Completed
                | RunState::Failed
                | RunState::Cancelled
                | RunState::OutcomeUnknown => {
                    return Ok(RunProgress {
                        run_id: run_id.to_string(),
                        steps_executed: executed,
                        steps_memoised: memoised,
                        view,
                    });
                }
                RunState::CancelRequested => {
                    // §12.7: "stop d'émission" is ABSOLUTE — the close
                    // NEVER calls `target.apply`, because a pending intent
                    // whose effect was never attempted (a B1 crash) would
                    // otherwise get its FIRST emission after the
                    // cancellation (Codex r1 P1). Pending rows stay
                    // pending, honestly journaled.
                    //
                    // Activity mode (Phase K): in-flight attempts are
                    // STOPPED before the close — the descendant process
                    // tree dies — and a result that finished just before
                    // the stop is ROUTED through `record_late_completion`
                    // (kept, frontier untouched, never propagated): the
                    // §12.7 late-result door gets its production caller
                    // here.
                    if let EffectDriver::Activity(activity) = driver {
                        self.drain_open_attempts_on_cancel(plan, run_id, *activity)?;
                    }
                    self.store.record_terminal(
                        run_id,
                        None,
                        TerminalReason::Cancelled,
                        None,
                        None,
                        Some(now),
                    )?;
                    continue;
                }
                _ => {}
            }

            // 3. Open the waits of newly-parked nodes and drain the host's
            // signal source into durable deposits.
            let mut progressed = false;

            // 3a. FREEZE fired joins (Phase K, the J carry): a join the
            // reducer sees Succeeded without a journaled cohort gets its
            // delivered sources recorded NOW — a late-arriving source can
            // no longer widen a quorum or replace an `any` winner, in the
            // view or in the value resolution.
            for (rid, state) in &view.node_states {
                if *state != NodeState::Succeeded {
                    continue;
                }
                let instance = self.instance(plan, rid)?;
                let WorkflowNode::Join(join) = &instance.node else {
                    continue;
                };
                let already = events.iter().any(|e| {
                    e.node_instance_id.as_deref() == Some(rid.as_str())
                        && matches!(e.kind, RunEventKind::JoinFired { .. })
                });
                if already {
                    continue;
                }
                let sources = self.delivered_join_sources(plan, run_id, rid, join)?;
                self.store
                    .record_decision(run_id, rid, &RunEventKind::JoinFired { sources })?;
                progressed = true;
            }

            // 3b. Terminally-failed nodes close the run with the honest
            // diagnosis (Phase K) UNLESS the failure has somewhere to go:
            // under `route-error-port` the reducer delivers the node's
            // `error` edge and nothing closes here. But a failure with NO
            // wired `error` edge (a classifier — the compiler forbids the
            // edge on it — or a leaf a plan simply left unwired) cannot be
            // routed: it FAILS CLOSED here, never swallowed into a silent
            // completion.
            let mut closed_node_failed = false;
            for (rid, state) in &view.node_states {
                if *state != NodeState::FailedTerminal {
                    continue;
                }
                if !is_effect_leaf(&self.instance(plan, rid)?.node) {
                    continue;
                }
                let routes_error = self.failure_action(plan, rid)
                    == NodeFailureAction::RouteErrorPort
                    && self.has_error_edge(plan, rid);
                if !routes_error {
                    self.store.record_terminal(
                        run_id,
                        Some(rid),
                        TerminalReason::NodeFailed,
                        None,
                        None,
                        Some(now),
                    )?;
                    closed_node_failed = true;
                    break;
                }
            }
            if closed_node_failed {
                continue;
            }
            for rid in &view.waiting {
                let instance = self.instance(plan, rid)?;
                let channel = match &instance.node {
                    WorkflowNode::WaitSignal(n) => n.signal_name.clone(),
                    WorkflowNode::HumanReview(_) => HUMAN_DECISION_CHANNEL.to_string(),
                    _ => continue,
                };
                let nonce = derive_wait_nonce(run_id, rid, &channel);
                self.store.open_wait(run_id, rid, &channel, &nonce)?;
            }
            let open = self.store.open_waits(run_id)?;
            if !open.is_empty() {
                for deposit in signals.poll(&open) {
                    self.store.deposit_signal(
                        run_id,
                        &deposit.nonce,
                        &deposit.payload,
                        deposit.decision_label.as_deref(),
                        deposit.expires_at_ms,
                        now,
                    )?;
                }
                // 4. Consume every wait whose deposit arrived — mono-usage,
                // atomic with its journal entry. "No deposit yet" keeps
                // waiting; every other refusal is LOUD.
                for wait in &open {
                    if !self.store.has_deposit(run_id, &wait.nonce)? {
                        continue;
                    }
                    let instance = self.instance(plan, &wait.node_instance_id)?;
                    let expected_labels: Option<Vec<String>> = match &instance.node {
                        WorkflowNode::HumanReview(n) => Some(n.decision_labels.clone()),
                        _ => None,
                    };
                    self.store.consume_signal_atomic(
                        run_id,
                        &wait.node_instance_id,
                        expected_labels.as_deref(),
                        now,
                    )?;
                    progressed = true;
                }
            }

            // 5. Advance repeats whose current iteration settled: journal
            // the continue/stop verdict (a decision, not an effect).
            progressed |= self.advance_repeats(plan, run_id, &view)?;

            // 6. Dispatch in canonical order under the bounded in-flight
            // cap. Candidates are the READY set PLUS the effect leaves the
            // reducer sees as RUNNING — an intent without a completion is
            // exactly what a writer crash mid-effect leaves behind, and
            // the resume path (re-apply idempotent / outcome_unknown /
            // hash divergence, all fail-closed) lives in `run_effect`'s
            // pending branch, which only a re-dispatch can reach.
            let mut candidates: Vec<String> = view.ready.clone();
            for (rid, state) in &view.node_states {
                if *state == NodeState::Running && is_effect_leaf(&self.instance(plan, rid)?.node) {
                    candidates.push(rid.clone());
                }
                // Retryable failures re-enter the ready set in ACTIVITY
                // mode — paced by the durable retry timer, gated inside
                // `run_activity_effect` (Phase K).
                if *state == NodeState::FailedRetryable
                    && matches!(driver, EffectDriver::Activity(_))
                    && is_effect_leaf(&self.instance(plan, rid)?.node)
                {
                    candidates.push(rid.clone());
                }
            }
            candidates.sort();
            candidates.dedup();
            let batch: Vec<String> = candidates
                .into_iter()
                .take(self.config.max_in_flight.max(1))
                .collect();
            for rid in &batch {
                let outcome = self.dispatch(plan, run_id, rid, driver, now)?;
                executed += usize::from(outcome.executed);
                memoised += usize::from(outcome.memoised);
                progressed |= outcome.progressed;
            }

            if progressed {
                continue;
            }

            // 7. Nothing moved: close a fully-settled run, otherwise hand
            // the state back (parked or externally blocked).
            if view.fully_settled
                && self.store.run_status(run_id)? == Some(RunStatus::Running)
                && !view.cancel_requested
            {
                self.store.record_run_completed(run_id, Some(now))?;
                continue;
            }
            return Ok(RunProgress {
                run_id: run_id.to_string(),
                steps_executed: executed,
                steps_memoised: memoised,
                view,
            });
        }
    }

    fn instance<'p>(
        &self,
        plan: &'p CompiledPlan,
        rid: &str,
    ) -> Result<&'p CompiledNodeInstance, OrchestratorError> {
        plan.node_instances
            .get(&structural_id(rid))
            .ok_or_else(|| OrchestratorError::Plan(format!("no plan occurrence for {rid}")))
    }

    /// Journal the continue/stop verdicts of repeats whose current
    /// iteration settled.
    fn advance_repeats(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        view: &RunView,
    ) -> Result<bool, OrchestratorError> {
        let mut progressed = false;
        for (rid, state) in &view.node_states {
            if *state != NodeState::Running {
                continue;
            }
            let instance = self.instance(plan, rid)?;
            let WorkflowNode::Repeat(repeat) = &instance.node else {
                continue;
            };
            let events = self.store.load_events(run_id)?;
            let verdicts = events
                .iter()
                .filter(|e| e.node_instance_id.as_deref() == Some(rid.as_str()))
                .filter_map(|e| match &e.kind {
                    crate::event::RunEventKind::BranchTaken { port } => Some(port.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if verdicts.iter().any(|v| v == REPEAT_PORT_STOP) {
                continue; // settled verdict; reducer will catch up
            }
            let bound = repeat.max_iterations.get() as usize;
            let current = verdicts.len().min(bound - 1);
            let child_scope = extend_scope(rid, current as u32);
            if !self.scope_settled(plan, run_id, &structural_id(rid), &child_scope, view)? {
                continue;
            }
            // The iteration's output decides: stop on the flag, or on the
            // STRUCTURAL bound of the signed plan — never a runtime
            // counter of the engine's own.
            let output = self.scope_output(plan, run_id, &structural_id(rid), &child_scope)?;
            let flag = output
                .get(&repeat.until_output_flag)
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let verdict = if flag || current + 1 >= bound {
                REPEAT_PORT_STOP
            } else {
                REPEAT_PORT_CONTINUE
            };
            self.store.record_decision(
                run_id,
                rid,
                &crate::event::RunEventKind::BranchTaken {
                    port: verdict.to_string(),
                },
            )?;
            progressed = true;
        }
        Ok(progressed)
    }

    fn scope_settled(
        &self,
        plan: &CompiledPlan,
        _run_id: &str,
        expanding_structural: &str,
        child_scope_runtime: &str,
        view: &RunView,
    ) -> Result<bool, OrchestratorError> {
        let Some(entries) = plan.entrypoints.get(expanding_structural) else {
            return Ok(true);
        };
        let _ = entries;
        // DIRECT children only (Codex r1 P1): a nested map/repeat child
        // carries ITS OWN iteration suffix at runtime, so pasting the
        // structural rest under this scope's runtime id names an instance
        // that never exists and the outer repeat would never advance. The
        // reducer already folds a nested expanding node's whole subtree
        // into that node's own settled state — recursion does the work.
        Ok(plan
            .node_instances
            .keys()
            .filter(|id| parent_scope(id) == expanding_structural)
            .all(|structural| {
                let rest = &structural[expanding_structural.len()..];
                let rid = format!("{child_scope_runtime}{rest}");
                view.node_states
                    .get(&rid)
                    .copied()
                    .is_some_and(NodeState::is_settled)
            }))
    }

    /// Dispatch one READY node. Routing decisions count as neither
    /// executed nor memoised, but always as progress; an in-flight
    /// activity that is not due yet is the one NON-progressing outcome
    /// (the caller returns the state instead of spinning).
    fn dispatch(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        driver: &mut EffectDriver<'_>,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        let instance = self.instance(plan, rid)?;
        match instance.node.clone() {
            WorkflowNode::Switch(node) => {
                let payload = self.input_value(plan, run_id, rid)?;
                let discriminant = payload
                    .get(&node.discriminant_field)
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| OrchestratorError::BadPayload {
                        node_instance_id: rid.to_string(),
                        reason: format!(
                            "discriminant field `{}` absent or not a string",
                            node.discriminant_field
                        ),
                    })?;
                let port = if node.branch_labels.iter().any(|l| l == discriminant) {
                    discriminant.to_string()
                } else {
                    DEFAULT_PORT.to_string()
                };
                self.store.record_decision(
                    run_id,
                    rid,
                    &crate::event::RunEventKind::BranchTaken { port },
                )?;
                Ok(DispatchOutcome::decision())
            }
            WorkflowNode::Classifier(node) => {
                // The model call is an EFFECT; the label is read from the
                // effect OUTPUT and validated against the closed set — an
                // out-of-set proposal routes to `default` deterministically
                // (§19.3), it never mints a port. In activity mode the
                // effect may still be in flight (or failed/parked): the
                // decision is journaled only once the effect CONFIRMED.
                let outcome = self.run_effect(plan, run_id, rid, driver, now)?;
                if self.store.completed_effect(run_id, rid)?.is_none() {
                    return Ok(outcome);
                }
                let output = self.node_output(plan, run_id, rid)?;
                let proposed = output
                    .get(CLASSIFIER_LABEL_FIELD)
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or(DEFAULT_PORT);
                let port = if node.branch_labels.iter().any(|l| l == proposed) {
                    proposed.to_string()
                } else {
                    DEFAULT_PORT.to_string()
                };
                self.store.record_decision(
                    run_id,
                    rid,
                    &crate::event::RunEventKind::BranchTaken { port },
                )?;
                Ok(outcome)
            }
            WorkflowNode::Map(node) => {
                let payload = self.input_value(plan, run_id, rid)?;
                let collection = payload
                    .get(&node.collection_field)
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| OrchestratorError::BadPayload {
                        node_instance_id: rid.to_string(),
                        reason: format!(
                            "collection field `{}` absent or not an array",
                            node.collection_field
                        ),
                    })?;
                let len = collection.len();
                let bound = node.max_cardinality.get();
                if len > bound as usize {
                    // The bound is DATA OF THE SIGNED PLAN; an oversized
                    // collection is refused, the bound is never stretched.
                    return Err(OrchestratorError::MapCardinalityExceeded {
                        node_instance_id: rid.to_string(),
                        len,
                        bound,
                    });
                }
                self.store.record_decision(
                    run_id,
                    rid,
                    &crate::event::RunEventKind::MapFanout {
                        cardinality: len as u32,
                    },
                )?;
                Ok(DispatchOutcome::decision())
            }
            // Repeat/Subworkflow activate by DERIVATION (their children
            // enter the ready-set); Parallel/Join/Waits never reach the
            // ready-set at all.
            WorkflowNode::Repeat(_)
            | WorkflowNode::Subworkflow(_)
            | WorkflowNode::Parallel(_)
            | WorkflowNode::Join(_)
            | WorkflowNode::WaitSignal(_)
            | WorkflowNode::HumanReview(_) => Ok(DispatchOutcome::decision()),
            // Every remaining kind is an effect leaf on the driver.
            _ => self.run_effect(plan, run_id, rid, driver, now),
        }
    }

    /// The shared fail-closed replay checks (memoised divergence, pending
    /// divergence, idempotency-claim and dedup-scope drift) plus the
    /// per-driver step mechanics. Both paths go through the SAME store
    /// doors, which carry the guarantees; the spike engine keeps its own
    /// linear copy as the frozen Phase E proof.
    fn run_effect(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        driver: &mut EffectDriver<'_>,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        let input = self.effect_input_map(plan, run_id, rid)?;
        let request = StepRequest::new(rid.to_string(), driver.name(), input.clone())?;
        let request_hash = step_request_hash(&request)?;

        if let Some(done) = self.store.completed_effect(run_id, rid)? {
            if done.request_hash != request_hash {
                if self.store.run_status(run_id)? == Some(RunStatus::Running) {
                    // `closed_at = None` is DELIBERATE on every divergence
                    // terminal (here and below, mirroring the spike): a run
                    // that failed closed on a hash mismatch is forensic
                    // evidence, and no close date means it is NEVER
                    // GC-eligible. Cancelled runs pass `Some(now)` — their
                    // retention is a policy matter, not a diagnosis.
                    self.store.record_terminal(
                        run_id,
                        Some(rid),
                        TerminalReason::StepHashMismatch,
                        Some(&done.request_hash),
                        Some(&request_hash),
                        None,
                    )?;
                }
                return Err(EngineError::StepHashMismatch {
                    node_instance_id: rid.to_string(),
                    expected: done.request_hash,
                    got: request_hash,
                }
                .into());
            }
            return Ok(DispatchOutcome::memoised());
        }

        let pending = match self.store.pending_effect(run_id, rid)? {
            Some(pending) => {
                if pending.request_hash != request_hash {
                    self.store.record_terminal(
                        run_id,
                        Some(rid),
                        TerminalReason::StepHashMismatch,
                        Some(&pending.request_hash),
                        Some(&request_hash),
                        None,
                    )?;
                    return Err(EngineError::StepHashMismatch {
                        node_instance_id: rid.to_string(),
                        expected: pending.request_hash,
                        got: request_hash,
                    }
                    .into());
                }
                if pending.target_idempotent != driver.is_idempotent() {
                    return Err(EngineError::TargetIdempotencyChanged {
                        node_instance_id: rid.to_string(),
                        journaled: pending.target_idempotent,
                        live: driver.is_idempotent(),
                    }
                    .into());
                }
                if pending.dedup_scope != driver.dedup_scope() {
                    return Err(EngineError::TargetScopeChanged {
                        node_instance_id: rid.to_string(),
                        journaled: pending.dedup_scope,
                        live: driver.dedup_scope(),
                    }
                    .into());
                }
                Some(pending)
            }
            None => None,
        };

        match driver {
            EffectDriver::Target(target) => {
                let effect_id = match pending {
                    Some(pending) => {
                        if !pending.target_idempotent {
                            // The Phase J resume semantics, unchanged: an
                            // unconfirmed non-idempotent effect is
                            // unknowable — the blocking path has no
                            // attempt bookkeeping to narrow it.
                            self.store.record_terminal(
                                run_id,
                                Some(rid),
                                TerminalReason::OutcomeUnknown,
                                None,
                                None,
                                None,
                            )?;
                            return Err(EngineError::OutcomeUnknown {
                                node_instance_id: rid.to_string(),
                                effect_id: pending.effect_id,
                            }
                            .into());
                        }
                        pending.effect_id
                    }
                    None => {
                        let effect_id = derive_effect_id(run_id, rid, &request_hash);
                        self.store.record_intent_atomic(
                            run_id,
                            rid,
                            &request_hash,
                            &effect_id,
                            target.is_idempotent(),
                            &target.dedup_scope(),
                        )?;
                        effect_id
                    }
                };
                let key = IdempotencyKey::new(run_id, rid, effect_id.as_str())?;
                let receipt = target.apply(&key, &input)?;
                self.store
                    .record_completion(run_id, rid, &effect_id, &request_hash, &receipt)?;
                Ok(DispatchOutcome::executed())
            }
            EffectDriver::Activity(activity) => self.run_activity_effect(
                plan,
                run_id,
                rid,
                *activity,
                pending,
                &request_hash,
                input,
                now,
            ),
        }
    }

    /// The Phase K non-blocking step mechanics: attempts under the single
    /// durable intent, per-attempt deadlines, journaled failures with
    /// bounded backoff, the honest §12.5 unknowable outcome, and the
    /// fencing epoch presented target-side.
    #[allow(clippy::too_many_arguments)]
    fn run_activity_effect(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        activity: &mut dyn Activity,
        pending: Option<PendingEffect>,
        request_hash: &str,
        input: BTreeMap<String, String>,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        let pending = match pending {
            Some(pending) => pending,
            None => {
                let effect_id = derive_effect_id(run_id, rid, request_hash);
                self.store.record_intent_atomic(
                    run_id,
                    rid,
                    request_hash,
                    &effect_id,
                    activity.is_idempotent(),
                    &activity.dedup_scope(),
                )?;
                self.store
                    .pending_effect(run_id, rid)?
                    .ok_or_else(|| OrchestratorError::Plan(format!("intent vanished for {rid}")))?
            }
        };
        let effect_id = pending.effect_id.clone();
        let key = IdempotencyKey::new(run_id, rid, effect_id.as_str())?;
        let fencing_epoch = self
            .store
            .active_lease_epoch()
            .unwrap_or(crate::activity::UNFENCED_EPOCH);

        match self.store.open_attempt(run_id, rid)? {
            None => {
                let failures = self.journaled_failures(run_id, rid)?;
                if failures > 0 {
                    // Pacing gate: the retry timer scheduled at the last
                    // failure must have fired (the due-timer step fires it
                    // when its ABSOLUTE deadline passed the watermarked
                    // clock). Pacing changes timing, never outcomes.
                    let timer = retry_timer_id(&effect_id, failures);
                    // Self-heal (mirror of the workflow-timeout re-propose):
                    // the failure and its backoff timer are journaled in TWO
                    // transactions; a crash between them would lose the timer
                    // and strand the run in FailedRetryable forever. Re-arm
                    // idempotently — INSERT OR IGNORE, so the normal path
                    // (timer already present) is a no-op, and a crash-lost
                    // timer is re-created with a fresh backoff deadline.
                    let deadline = now.saturating_add(backoff_ms(failures));
                    self.store.schedule_timer(run_id, &timer, deadline)?;
                    if !self.store.timer_fired(run_id, &timer)? {
                        return Ok(DispatchOutcome::idle());
                    }
                }
                let attempt = failures + 1;
                let deadline = now.saturating_add(self.node_timeout_ms(plan, rid));
                self.store.record_attempt_started(
                    run_id, rid, &effect_id, attempt, now, deadline, None,
                )?;
                let ctx = EffectContext {
                    key: key.clone(),
                    attempt,
                    fencing_epoch,
                    deadline_ms: deadline,
                    input: input.clone(),
                };
                match activity.start(&ctx) {
                    Ok(()) => {
                        if let Some(handle) = activity.runtime_handle(&ctx) {
                            self.store
                                .set_attempt_handle(run_id, rid, &effect_id, attempt, &handle)?;
                        }
                        // A synchronous activity may already be Ready:
                        // poll once in the same pass. Whatever the poll
                        // says, THIS branch journaled an intent and/or
                        // opened an attempt — durable progress: the loop
                        // must re-reduce so the caller never gets a
                        // pre-intent view back.
                        let outcome = self
                            .poll_open_attempt(plan, run_id, rid, activity, &pending, &ctx, now)?;
                        Ok(DispatchOutcome {
                            progressed: true,
                            ..outcome
                        })
                    }
                    Err(ActivityError::Fenced { seen, presented }) => {
                        // A deposed writer must STOP — surfacing loudly,
                        // journaling nothing (its journal write would be
                        // fenced by the CAS anyway).
                        Err(ActivityError::Fenced { seen, presented }.into())
                    }
                    Err(err) => {
                        let retryable = start_failure_retryable(&err);
                        self.record_failure_with_backoff(
                            plan, run_id, rid, &effect_id, attempt, retryable, now,
                        )?;
                        Ok(DispatchOutcome::progressed_only())
                    }
                }
            }
            Some(open) => {
                let ctx = EffectContext {
                    key,
                    attempt: open.attempt,
                    fencing_epoch,
                    deadline_ms: open.deadline_ms,
                    input,
                };
                self.poll_open_attempt_resumable(
                    plan, run_id, rid, activity, &pending, &ctx, &open, now,
                )
            }
        }
    }

    /// Poll the attempt this pass started (no orphan handling needed —
    /// the activity knows the key).
    #[allow(clippy::too_many_arguments)]
    fn poll_open_attempt(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        activity: &mut dyn Activity,
        pending: &PendingEffect,
        ctx: &EffectContext,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        match activity.poll(ctx)? {
            PollOutcome::Ready { output, facts } => {
                self.complete_activity(run_id, rid, pending, ctx, activity, output, facts)?;
                Ok(DispatchOutcome::executed())
            }
            PollOutcome::Pending => {
                self.judge_deadline(plan, run_id, rid, activity, pending, ctx, now)
            }
            PollOutcome::Failed { retryable, reason } => {
                let _ = reason; // journal stays payload-free by type
                self.record_failure_with_backoff(
                    plan,
                    run_id,
                    rid,
                    &pending.effect_id,
                    ctx.attempt,
                    retryable,
                    now,
                )?;
                Ok(DispatchOutcome::progressed_only())
            }
        }
    }

    /// Poll a PRE-EXISTING open attempt — the resume path: an activity
    /// that does not know the key anymore is a previous incarnation's
    /// orphan, reaped by the journaled handle then decided by the
    /// JOURNALED idempotency claim (§12.5).
    #[allow(clippy::too_many_arguments)]
    fn poll_open_attempt_resumable(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        activity: &mut dyn Activity,
        pending: &PendingEffect,
        ctx: &EffectContext,
        open: &OpenAttempt,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        match activity.poll(ctx) {
            Ok(outcome) => match outcome {
                PollOutcome::Ready { output, facts } => {
                    self.complete_activity(run_id, rid, pending, ctx, activity, output, facts)?;
                    Ok(DispatchOutcome::executed())
                }
                PollOutcome::Pending => {
                    self.judge_deadline(plan, run_id, rid, activity, pending, ctx, now)
                }
                PollOutcome::Failed { retryable, reason } => {
                    let _ = reason;
                    self.record_failure_with_backoff(
                        plan,
                        run_id,
                        rid,
                        &pending.effect_id,
                        ctx.attempt,
                        retryable,
                        now,
                    )?;
                    Ok(DispatchOutcome::progressed_only())
                }
            },
            Err(ActivityError::UnknownAttempt) => {
                // Orphan of a previous incarnation. Best-effort reap of
                // the journaled handle, then the §12.5 decision on the
                // JOURNALED claim: an interrupted idempotent attempt is a
                // retryable failure; a non-idempotent one is unknowable —
                // the attempt row narrows E's window (intent→completion)
                // to attempt-start→completion, and inside it the honest
                // answer is still "unknown".
                activity.reap_orphan(ctx, open.runtime_handle.as_deref())?;
                if pending.target_idempotent {
                    self.record_failure_with_backoff(
                        plan,
                        run_id,
                        rid,
                        &pending.effect_id,
                        ctx.attempt,
                        true,
                        now,
                    )?;
                    Ok(DispatchOutcome::progressed_only())
                } else {
                    self.unknowable_outcome(run_id, rid, pending, now)
                }
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Judge an in-flight attempt against its ABSOLUTE deadline: not due
    /// keeps it in flight (no progress); due stops it and applies §12.5 —
    /// idempotent targets fail retryably, non-idempotent ones are
    /// unknowable.
    #[allow(clippy::too_many_arguments)]
    fn judge_deadline(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        activity: &mut dyn Activity,
        pending: &PendingEffect,
        ctx: &EffectContext,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        if now < ctx.deadline_ms {
            return Ok(DispatchOutcome::idle());
        }
        // Deadline crossed: stop the work (kills the descendant tree for
        // a process activity) BEFORE deciding — a still-running effect
        // must not race its own verdict.
        activity.cancel(ctx)?;
        if pending.target_idempotent {
            self.record_failure_with_backoff(
                plan,
                run_id,
                rid,
                &pending.effect_id,
                ctx.attempt,
                true,
                now,
            )?;
            Ok(DispatchOutcome::progressed_only())
        } else {
            self.unknowable_outcome(run_id, rid, pending, now)
        }
    }

    /// §12.5's unknowable outcome, under the host's reconciliation
    /// policy: PARK the node (run stays alive, operator reconciles) or
    /// close the run terminally — NEVER an automatic retry, NEVER an
    /// invented success.
    fn unknowable_outcome(
        &mut self,
        run_id: &str,
        rid: &str,
        pending: &PendingEffect,
        now: i64,
    ) -> Result<DispatchOutcome, OrchestratorError> {
        if self.config.park_unknown_for_reconciliation {
            self.store
                .record_node_outcome_unknown(run_id, rid, &pending.effect_id)?;
            Ok(DispatchOutcome::progressed_only())
        } else {
            // TERMINAL FIRST, then close the attempt row: the two doors are
            // separate transactions, and this order fails safe — a crash
            // between them leaves a closed run with an open attempt row
            // (benign bookkeeping, purged by GC, never resumed). The
            // inverse order would leave a RUNNING run whose closed attempt
            // row collides with the next `record_attempt_started` PK on
            // resume — a permanently stuck run (re-review K).
            self.store.record_terminal(
                run_id,
                Some(rid),
                TerminalReason::OutcomeUnknown,
                None,
                None,
                Some(now),
            )?;
            self.store.close_open_attempt(
                run_id,
                rid,
                &pending.effect_id,
                AttemptOutcome::Unknown,
            )?;
            Err(EngineError::OutcomeUnknown {
                node_instance_id: rid.to_string(),
                effect_id: pending.effect_id.clone(),
            }
            .into())
        }
    }

    /// Journal one attempt failure, then — when the scope's budget still
    /// allows a retry — schedule the bounded-backoff pacing timer the
    /// next dispatch is gated on.
    #[allow(clippy::too_many_arguments)]
    fn record_failure_with_backoff(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        effect_id: &str,
        attempt: u32,
        retryable: bool,
        now: i64,
    ) -> Result<(), OrchestratorError> {
        self.store
            .record_attempt_failed(run_id, rid, effect_id, attempt, retryable)?;
        let budget = self.max_retries(plan, rid);
        if retryable && attempt <= budget {
            let deadline = now.saturating_add(backoff_ms(attempt));
            self.store
                .schedule_timer(run_id, &retry_timer_id(effect_id, attempt), deadline)?;
        }
        Ok(())
    }

    /// Confirm the effect: render the structured §17.1(6) receipt and
    /// write the completion through the store door (digest to the
    /// journal, material to the outbox).
    #[allow(clippy::too_many_arguments)]
    fn complete_activity(
        &mut self,
        run_id: &str,
        rid: &str,
        pending: &PendingEffect,
        ctx: &EffectContext,
        activity: &dyn Activity,
        output: serde_json::Value,
        facts: BTreeMap<String, String>,
    ) -> Result<(), OrchestratorError> {
        let receipt = ActivityReceipt {
            kind: crate::activity::ACTIVITY_RECEIPT_KIND.to_string(),
            run_id: run_id.to_string(),
            node_instance_id: rid.to_string(),
            effect_id: pending.effect_id.clone(),
            attempt: ctx.attempt,
            request_hash: pending.request_hash.clone(),
            activity: activity.name().to_string(),
            dedup_scope: pending.dedup_scope.clone(),
            fencing_epoch: ctx.fencing_epoch,
            facts,
            output,
        }
        .render()?;
        self.store.record_completion(
            run_id,
            rid,
            &pending.effect_id,
            &pending.request_hash,
            &receipt,
        )?;
        Ok(())
    }

    /// Count the journaled attempt failures of one node.
    fn journaled_failures(&self, run_id: &str, rid: &str) -> Result<u32, OrchestratorError> {
        let events = self.store.load_events(run_id)?;
        let count = events
            .iter()
            .filter(|e| e.node_instance_id.as_deref() == Some(rid))
            .filter(|e| matches!(e.kind, RunEventKind::EffectAttemptFailed { .. }))
            .count();
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    /// The scope's per-node timeout, in milliseconds (exact scope, else
    /// root, else the conservative 1s floor — a missing policy must never
    /// mean "no deadline").
    fn node_timeout_ms(&self, plan: &CompiledPlan, rid: &str) -> i64 {
        let scope = parent_scope(&structural_id(rid)).to_string();
        plan.timeout_policies
            .get(&scope)
            .or_else(|| plan.timeout_policies.get(""))
            .map_or(1_000, |policy| {
                i64::from(policy.default_node_timeout_seconds.get()).saturating_mul(1000)
            })
    }

    /// The scope's `max_retries_per_node` (exact scope, else root, else
    /// 0 — the reducer's mirror; the two MUST agree or the pacing gate
    /// and the derived state drift apart).
    fn max_retries(&self, plan: &CompiledPlan, rid: &str) -> u32 {
        let scope = parent_scope(&structural_id(rid)).to_string();
        plan.error_policies
            .get(&scope)
            .or_else(|| plan.error_policies.get(""))
            .map_or(0, |policy| policy.max_retries_per_node)
    }

    /// Whether the node has an `error` edge wired FROM it — the port a
    /// `route-error-port` failure delivers on. Absent (a classifier, or a
    /// leaf a plan left unwired) means the failure has nowhere to route and
    /// must fail closed.
    fn has_error_edge(&self, plan: &CompiledPlan, rid: &str) -> bool {
        let structural = structural_id(rid);
        plan.edges
            .iter()
            .any(|e| e.from == structural && e.from_port == ERROR_PORT)
    }

    /// The scope's failure action (exact scope, else root, else
    /// fail-workflow — fail-closed).
    fn failure_action(&self, plan: &CompiledPlan, rid: &str) -> NodeFailureAction {
        let scope = parent_scope(&structural_id(rid)).to_string();
        plan.error_policies
            .get(&scope)
            .or_else(|| plan.error_policies.get(""))
            .map_or(NodeFailureAction::FailWorkflow, |policy| {
                policy.on_node_failure
            })
    }

    /// The delivered sources of a join at fire time — the cohort the
    /// `join_fired` entry freezes.
    fn delivered_join_sources(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        join: &sbfb_workflow_core::graph::JoinNode,
    ) -> Result<Vec<String>, OrchestratorError> {
        let structural = structural_id(rid);
        let scope_structural = parent_scope(&structural).to_string();
        let scope_runtime = parent_scope(rid).to_string();
        let mut sources = Vec::new();
        for source in &join.source_node_ids {
            let source_structural = child_structural(&scope_structural, source);
            let source_rid = format!(
                "{scope_runtime}{}",
                &source_structural[scope_structural.len()..]
            );
            let delivered = plan
                .edges
                .iter()
                .filter(|e| e.from == source_structural && e.to == structural)
                .try_fold(false, |acc, edge| {
                    Ok::<bool, OrchestratorError>(
                        acc || self.edge_delivered(
                            plan,
                            run_id,
                            &edge.from,
                            &source_rid,
                            &edge.from_port,
                        )?,
                    )
                })?;
            if delivered {
                sources.push(source.clone());
            }
        }
        Ok(sources)
    }

    /// Cancel-path drain (§12.7, Phase K): stop every open attempt; a
    /// result that FINISHED just before the stop is kept through
    /// `record_late_completion` (frontier untouched, never propagated),
    /// everything else closes as cancelled bookkeeping.
    fn drain_open_attempts_on_cancel(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        activity: &mut dyn Activity,
    ) -> Result<(), OrchestratorError> {
        let fencing_epoch = self
            .store
            .active_lease_epoch()
            .unwrap_or(crate::activity::UNFENCED_EPOCH);
        for open in self.store.open_attempts(run_id)? {
            let rid = open.node_instance_id.clone();
            let Some(pending) = self.store.pending_effect(run_id, &rid)? else {
                self.store.close_attempt(
                    run_id,
                    &rid,
                    &open.effect_id,
                    open.attempt,
                    AttemptOutcome::Cancelled,
                )?;
                continue;
            };
            let input = self.effect_input_map(plan, run_id, &rid)?;
            let ctx = EffectContext {
                key: IdempotencyKey::new(run_id, rid.as_str(), pending.effect_id.as_str())?,
                attempt: open.attempt,
                fencing_epoch,
                deadline_ms: open.deadline_ms,
                input,
            };
            // One LAST look before the stop. A result OBSERVED Ready at
            // this poll is evidence and is kept (§12.7) — but NEVER
            // propagated: the late door leaves the frontier alone. (Honest
            // bound: a result that finishes AFTER this poll but before the
            // stop has no second poll and is dropped — an inherent race,
            // not a lost-forever guarantee.)
            match activity.poll(&ctx) {
                Ok(PollOutcome::Ready { output, facts }) => {
                    let receipt = ActivityReceipt {
                        kind: crate::activity::ACTIVITY_RECEIPT_KIND.to_string(),
                        run_id: run_id.to_string(),
                        node_instance_id: rid.clone(),
                        effect_id: pending.effect_id.clone(),
                        attempt: ctx.attempt,
                        request_hash: pending.request_hash.clone(),
                        activity: activity.name().to_string(),
                        dedup_scope: pending.dedup_scope.clone(),
                        fencing_epoch: ctx.fencing_epoch,
                        facts,
                        output,
                    }
                    .render()?;
                    self.store.record_late_completion(
                        run_id,
                        &rid,
                        &pending.effect_id,
                        &pending.request_hash,
                        &receipt,
                    )?;
                }
                Err(ActivityError::UnknownAttempt) => {
                    // A PREVIOUS incarnation's orphan: this activity does
                    // not know the key, so `cancel` would be a no-op and
                    // the surviving process group would leak past the
                    // cancel. Reap it by the JOURNALED handle (the same
                    // door the normal resume uses), then close the row.
                    activity.reap_orphan(&ctx, open.runtime_handle.as_deref())?;
                    self.store.close_attempt(
                        run_id,
                        &rid,
                        &open.effect_id,
                        open.attempt,
                        AttemptOutcome::Cancelled,
                    )?;
                }
                // Still in flight (or a scripted failure): stop the work and
                // close the bookkeeping.
                Ok(PollOutcome::Pending) | Ok(PollOutcome::Failed { .. }) => {
                    activity.cancel(&ctx)?;
                    self.store.close_attempt(
                        run_id,
                        &rid,
                        &open.effect_id,
                        open.attempt,
                        AttemptOutcome::Cancelled,
                    )?;
                }
                // A REAL adapter error (fenced-out, precondition, I/O…)
                // propagates — the same discipline as the normal resume
                // path, never swallowed into a silent close (ex-Codex
                // K-CANCEL-ERR-02).
                Err(err) => return Err(err.into()),
            }
        }
        Ok(())
    }

    // ── value resolution ─────────────────────────────────────────────────
    //
    // Material values live OUTSIDE the journal (receipts, signal payloads,
    // run input). Resolution is recursive over the DAG and deliberately
    // uncached: no half-trusted in-memory frontier (the Phase E posture).
    // The cost is named, not hidden: `edge_delivered` re-loads the journal
    // (and re-reduces) PER INBOUND EDGE, so a resolution pass is
    // super-linear in journal length × fan-in. Acceptable for a solo-node
    // engine on plans this size; the disposable snapshot table (M2) is the
    // sanctioned cache if a real fan-out ever makes it measurable.

    /// The effect input map of a leaf: `{inbound port → canonical JCS}`.
    fn effect_input_map(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
    ) -> Result<BTreeMap<String, String>, OrchestratorError> {
        let value = self.input_ports(plan, run_id, rid)?;
        let mut map = BTreeMap::new();
        for (port, v) in value {
            map.insert(port, canonical_string(&v)?);
        }
        Ok(map)
    }

    /// The single input VALUE of a node: the sole port's value, or the
    /// port map as an object when several ports feed it.
    fn input_value(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
    ) -> Result<serde_json::Value, OrchestratorError> {
        let mut ports = self.input_ports(plan, run_id, rid)?;
        if ports.len() == 1 {
            Ok(ports.pop_first().expect("len checked").1)
        } else {
            Ok(serde_json::Value::Object(ports.into_iter().collect()))
        }
    }

    /// Inbound port values of a node, resolved through delivered edges.
    fn input_ports(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
    ) -> Result<BTreeMap<String, serde_json::Value>, OrchestratorError> {
        let structural = structural_id(rid);
        let scope_structural = parent_scope(&structural).to_string();
        let scope_runtime = parent_scope(rid).to_string();

        let inbound: Vec<_> = plan.edges.iter().filter(|e| e.to == structural).collect();
        if inbound.is_empty() {
            // Entrypoint: its value is the SCOPE input.
            let value = self.scope_input(plan, run_id, &scope_structural, &scope_runtime)?;
            let mut map = BTreeMap::new();
            map.insert("in".to_string(), value);
            return Ok(map);
        }

        let mut ports: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        for edge in inbound {
            let from_rid = format!("{scope_runtime}{}", &edge.from[scope_structural.len()..]);
            if !self.edge_delivered(plan, run_id, &edge.from, &from_rid, &edge.from_port)? {
                continue;
            }
            // First delivered edge wins a port (exclusive-branch OR): in a
            // deterministic walk "first" is canonical-edge order, which is
            // the plan's canonical sort.
            if ports.contains_key(&edge.to_port) {
                continue;
            }
            let value =
                self.output_for_edge(plan, run_id, &edge.from, &from_rid, &edge.from_port)?;
            ports.insert(edge.to_port.clone(), value);
        }
        Ok(ports)
    }

    /// Whether an edge delivered: its source succeeded and (for routing
    /// sources) took exactly this port.
    fn edge_delivered(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        from_structural: &str,
        from_rid: &str,
        from_port: &str,
    ) -> Result<bool, OrchestratorError> {
        let instance = plan
            .node_instances
            .get(from_structural)
            .ok_or_else(|| OrchestratorError::Plan(format!("no occurrence {from_structural}")))?;
        let events = self.store.load_events(run_id)?;
        match &instance.node {
            WorkflowNode::Switch(_) | WorkflowNode::Classifier(_) => {
                let taken = events
                    .iter()
                    .filter(|e| e.node_instance_id.as_deref() == Some(from_rid))
                    .filter_map(|e| match &e.kind {
                        crate::event::RunEventKind::BranchTaken { port } => Some(port.clone()),
                        _ => None,
                    })
                    .next_back();
                Ok(taken.as_deref() == Some(from_port))
            }
            WorkflowNode::HumanReview(_) => {
                let decided = events
                    .iter()
                    .filter(|e| e.node_instance_id.as_deref() == Some(from_rid))
                    .filter_map(|e| match &e.kind {
                        crate::event::RunEventKind::HumanDecisionRecorded {
                            decision_label,
                            ..
                        } => Some(decision_label.clone()),
                        _ => None,
                    })
                    .next_back();
                Ok(decided.as_deref() == Some(from_port))
            }
            _ => self.node_delivery(plan, run_id, from_rid, from_port),
        }
    }

    /// Delivery of a non-routing source's edge, mirroring the reducer
    /// (Phase K): a SUCCEEDED node delivers on every port except the
    /// reserved `error` one; a FAILED-TERMINAL node delivers on `error`
    /// alone (`route-error-port`).
    fn node_delivery(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        from_port: &str,
    ) -> Result<bool, OrchestratorError> {
        let events = self.store.load_events(run_id)?;
        let view = reduce(plan, &events)?;
        Ok(match view.node_states.get(rid).copied() {
            Some(NodeState::Succeeded) => from_port != ERROR_PORT,
            Some(NodeState::FailedTerminal) => from_port == ERROR_PORT,
            _ => false,
        })
    }

    /// The output value an edge carries from its (succeeded) source.
    fn output_for_edge(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        from_structural: &str,
        from_rid: &str,
        _from_port: &str,
    ) -> Result<serde_json::Value, OrchestratorError> {
        let instance = plan
            .node_instances
            .get(from_structural)
            .ok_or_else(|| OrchestratorError::Plan(format!("no occurrence {from_structural}")))?;
        match &instance.node {
            // Routing/broadcast nodes PASS THROUGH their own input.
            WorkflowNode::Switch(_) | WorkflowNode::Parallel(_) => {
                self.input_value(plan, run_id, from_rid)
            }
            WorkflowNode::Join(join) => {
                let scope_structural = parent_scope(from_structural).to_string();
                let scope_runtime = parent_scope(from_rid).to_string();
                // The FROZEN cohort wins (Phase K): once `join_fired` is
                // journaled, the value is collected from that cohort
                // alone — a source arriving later can no longer replace
                // an `any` winner or widen the collection.
                let frozen: Option<Vec<String>> = self
                    .store
                    .load_events(run_id)?
                    .iter()
                    .filter(|e| e.node_instance_id.as_deref() == Some(from_rid))
                    .find_map(|e| match &e.kind {
                        RunEventKind::JoinFired { sources } => Some(sources.clone()),
                        _ => None,
                    });
                let mut collected: BTreeMap<String, serde_json::Value> = BTreeMap::new();
                for source in &join.source_node_ids {
                    let source_structural = child_structural(&scope_structural, source);
                    let source_rid = format!(
                        "{scope_runtime}{}",
                        &source_structural[scope_structural.len()..]
                    );
                    // A source contributes its VALUE only when its edge
                    // toward THIS join DELIVERED — the same criterion the
                    // reducer's join_state counts (Codex r2 P1: state-only
                    // collection would include a routing source that
                    // succeeded on another port) — restricted to the
                    // frozen cohort when one is journaled.
                    let delivered = match &frozen {
                        Some(cohort) => cohort.contains(source),
                        None => plan
                            .edges
                            .iter()
                            .filter(|e| e.from == source_structural && e.to == *from_structural)
                            .try_fold(false, |acc, edge| {
                                Ok::<bool, OrchestratorError>(
                                    acc || self.edge_delivered(
                                        plan,
                                        run_id,
                                        &edge.from,
                                        &source_rid,
                                        &edge.from_port,
                                    )?,
                                )
                            })?,
                    };
                    if delivered {
                        collected
                            .insert(source.clone(), self.node_output(plan, run_id, &source_rid)?);
                    }
                }
                match &join.policy {
                    // ANY: the first arrived source in canonical order —
                    // deterministic under out-of-order completion because
                    // "first" is id order, never arrival order.
                    JoinPolicy::Any(_) => Ok(collected
                        .pop_first()
                        .map(|(_, v)| v)
                        .unwrap_or(serde_json::Value::Object(Default::default()))),
                    _ => Ok(serde_json::Value::Object(collected.into_iter().collect())),
                }
            }
            _ => self.node_output(plan, run_id, from_rid),
        }
    }

    /// The output VALUE of a settled node.
    fn node_output(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
    ) -> Result<serde_json::Value, OrchestratorError> {
        let structural = structural_id(rid);
        let instance = plan
            .node_instances
            .get(&structural)
            .ok_or_else(|| OrchestratorError::Plan(format!("no occurrence {structural}")))?;
        match &instance.node {
            WorkflowNode::WaitSignal(_) | WorkflowNode::HumanReview(_) => Ok(self
                .store
                .consumed_signal(run_id, rid)?
                .map(|c| c.payload)
                .unwrap_or(serde_json::Value::Object(Default::default()))),
            WorkflowNode::Switch(_) | WorkflowNode::Parallel(_) | WorkflowNode::Join(_) => {
                self.output_for_edge(plan, run_id, &structural, rid, "")
            }
            WorkflowNode::Map(map) => {
                let events = self.store.load_events(run_id)?;
                let n = events
                    .iter()
                    .filter(|e| e.node_instance_id.as_deref() == Some(rid))
                    .find_map(|e| match &e.kind {
                        crate::event::RunEventKind::MapFanout { cardinality } => Some(*cardinality),
                        _ => None,
                    })
                    .unwrap_or(0);
                let _ = map;
                // Children reduce BY ENCODED INDEX — arrival order plays
                // no part (§12.2; LittleHorse/DBOS child-id precedent).
                let mut items = Vec::with_capacity(n as usize);
                for i in 0..n {
                    let child_scope = extend_scope(rid, i);
                    items.push(self.scope_output(plan, run_id, &structural, &child_scope)?);
                }
                Ok(serde_json::Value::Array(items))
            }
            WorkflowNode::Repeat(_) => {
                let events = self.store.load_events(run_id)?;
                let verdicts = events
                    .iter()
                    .filter(|e| e.node_instance_id.as_deref() == Some(rid))
                    .filter(|e| matches!(&e.kind, crate::event::RunEventKind::BranchTaken { .. }))
                    .count();
                let last = verdicts.saturating_sub(1);
                let child_scope = extend_scope(rid, last as u32);
                self.scope_output(plan, run_id, &structural, &child_scope)
            }
            WorkflowNode::Subworkflow(_) => self.scope_output(plan, run_id, &structural, rid),
            // Effect leaves: a structured activity receipt carries its
            // output member (Phase K); the spike's raw echo receipt
            // inverts the input map (single-port maps collapse to the
            // port's value). A FAILED-TERMINAL leaf resolves to a
            // deterministic error object — what the `error` edge carries.
            _ => {
                let Some(done) = self.store.completed_effect(run_id, rid)? else {
                    let failures = self.journaled_failures(run_id, rid)?;
                    if failures > 0 {
                        return Ok(serde_json::json!({
                            "error": { "node": rid, "attempts": failures }
                        }));
                    }
                    return Err(OrchestratorError::Plan(format!("{rid} has no receipt yet")));
                };
                if let Some(receipt) = ActivityReceipt::parse(&done.receipt) {
                    return Ok(receipt.output);
                }
                let map: BTreeMap<String, String> = serde_json::from_str(&done.receipt)
                    .map_err(|e| OrchestratorError::Plan(format!("receipt of {rid}: {e}")))?;
                if map.len() == 1 {
                    let (_, jcs) = map.into_iter().next().expect("len checked");
                    serde_json::from_str(&jcs)
                        .map_err(|e| OrchestratorError::Plan(format!("value of {rid}: {e}")))
                } else {
                    let mut obj = serde_json::Map::new();
                    for (port, jcs) in map {
                        let v: serde_json::Value = serde_json::from_str(&jcs).map_err(|e| {
                            OrchestratorError::Plan(format!("value of {rid}:{port}: {e}"))
                        })?;
                        obj.insert(port, v);
                    }
                    Ok(serde_json::Value::Object(obj))
                }
            }
        }
    }

    /// The input VALUE of a scope: run input at the root, the map item at
    /// a map iteration, the previous iteration's output (or the repeat's
    /// own input at iteration 0) inside a repeat, the expanding node's
    /// input for a subworkflow.
    fn scope_input(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> Result<serde_json::Value, OrchestratorError> {
        if scope_structural.is_empty() {
            return Ok(self
                .store
                .run_input(run_id)?
                .unwrap_or(serde_json::Value::Object(Default::default())));
        }
        let instance = plan
            .node_instances
            .get(scope_structural)
            .ok_or_else(|| OrchestratorError::Plan(format!("no occurrence {scope_structural}")))?;
        // The runtime id of the EXPANDING node = the scope runtime minus
        // its own iteration suffix.
        let (expanding_rid, iteration) = split_iteration(scope_runtime);
        match &instance.node {
            WorkflowNode::Map(map) => {
                let payload = self.input_value(plan, run_id, &expanding_rid)?;
                let items = payload
                    .get(&map.collection_field)
                    .and_then(serde_json::Value::as_array)
                    .ok_or_else(|| OrchestratorError::BadPayload {
                        node_instance_id: expanding_rid.clone(),
                        reason: format!("collection field `{}` vanished", map.collection_field),
                    })?;
                let i = iteration.unwrap_or(0) as usize;
                items
                    .get(i)
                    .cloned()
                    .ok_or_else(|| OrchestratorError::BadPayload {
                        node_instance_id: expanding_rid,
                        reason: format!("no item {i} in the journaled fan-out"),
                    })
            }
            WorkflowNode::Repeat(_) => match iteration {
                None | Some(0) => self.input_value(plan, run_id, &expanding_rid),
                Some(i) => {
                    let previous = extend_scope(&expanding_rid, i - 1);
                    self.scope_output(plan, run_id, scope_structural, &previous)
                }
            },
            WorkflowNode::Subworkflow(_) => self.input_value(plan, run_id, &expanding_rid),
            _ => Err(OrchestratorError::Plan(format!(
                "{scope_structural} is not an expanding node"
            ))),
        }
    }

    /// The output VALUE of a scope: its single `output` node's value, or
    /// `{artifact_name → value}` when it publishes several.
    fn scope_output(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> Result<serde_json::Value, OrchestratorError> {
        let mut outputs: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        for (structural, instance) in &plan.node_instances {
            if parent_scope(structural) != scope_structural {
                continue;
            }
            if let WorkflowNode::Output(out) = &instance.node {
                let rest = &structural[scope_structural.len()..];
                let rid = format!("{scope_runtime}{rest}");
                if self.store.completed_effect(run_id, &rid)?.is_some() {
                    outputs.insert(
                        out.artifact_name.clone(),
                        self.node_output(plan, run_id, &rid)?,
                    );
                }
            }
        }
        if outputs.len() == 1 {
            Ok(outputs.pop_first().expect("len checked").1)
        } else {
            Ok(serde_json::Value::Object(outputs.into_iter().collect()))
        }
    }
}

/// What one dispatch did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DispatchOutcome {
    executed: bool,
    memoised: bool,
    progressed: bool,
}

impl DispatchOutcome {
    /// A routing/structural decision was journaled (or nothing had to
    /// be): the pass moved.
    fn decision() -> Self {
        Self {
            executed: false,
            memoised: false,
            progressed: true,
        }
    }

    /// The effect confirmed this pass.
    fn executed() -> Self {
        Self {
            executed: true,
            memoised: false,
            progressed: true,
        }
    }

    /// The journal already carried the answer.
    fn memoised() -> Self {
        Self {
            executed: false,
            memoised: true,
            progressed: true,
        }
    }

    /// Something was journaled (a failure, a park) without confirming.
    fn progressed_only() -> Self {
        Self {
            executed: false,
            memoised: false,
            progressed: true,
        }
    }

    /// In flight and not due: nothing moved — the caller must NOT spin.
    fn idle() -> Self {
        Self {
            executed: false,
            memoised: false,
            progressed: false,
        }
    }
}

/// Whether a START-side adapter refusal is worth retrying: transient I/O
/// is; a precondition mismatch or a contract refusal is not (the world
/// the plan was made against is gone, or the request itself is out of
/// contract).
fn start_failure_retryable(err: &ActivityError) -> bool {
    matches!(err, ActivityError::Io(_))
}

/// A node whose dispatch goes through the effect-step mechanics — the
/// candidates a resume must RE-dispatch when the reducer sees them
/// `Running` (pending intent left by a crash). Structural/expanding nodes
/// and interruptions resolve by derivation or consumption, never here.
fn is_effect_leaf(node: &WorkflowNode) -> bool {
    !matches!(
        node,
        WorkflowNode::Switch(_)
            | WorkflowNode::Parallel(_)
            | WorkflowNode::Join(_)
            | WorkflowNode::Map(_)
            | WorkflowNode::Repeat(_)
            | WorkflowNode::Subworkflow(_)
            | WorkflowNode::WaitSignal(_)
            | WorkflowNode::HumanReview(_)
    )
}

/// Split a runtime SCOPE id into the expanding node's runtime id and its
/// iteration index, when the last segment carries one.
fn split_iteration(scope_runtime: &str) -> (String, Option<u32>) {
    let last_segment_start = scope_runtime.rfind('/').map_or(0, |p| p + 1);
    let segment = &scope_runtime[last_segment_start..];
    if let Some(marker) = segment.rfind("~2") {
        let digits = &segment[marker + 2..];
        if !digits.is_empty()
            && digits.bytes().all(|b| b.is_ascii_digit())
            && let Ok(i) = digits.parse::<u32>()
        {
            let base = format!(
                "{}{}",
                &scope_runtime[..last_segment_start],
                &segment[..marker]
            );
            return (base, Some(i));
        }
    }
    (scope_runtime.to_string(), None)
}
