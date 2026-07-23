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
//! part; the per-effect capability binding is Phases K/L).
//!
//! ## Determinism
//!
//! Everything the walk decides is either DERIVED from the plan topology
//! (reducer) or JOURNALED BEFORE it is acted on (`branch_taken`,
//! `map_fanout`, consumptions, cancellation). The ready-set is dispatched
//! in canonical id order under a bounded in-flight cap — admission order
//! is a pure function of the journal, never of a HashMap or a clock. The
//! echo activity of Phase E is the only effect target; real adapters are
//! Phase K.
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
//!
//! ## What Phase J does NOT do (honest bounds)
//!
//! Real effects, retries/backoff, per-node timeouts, receipts,
//! `outcome_unknown` reconciliation and the external-effect window are
//! Phase K. Capability attenuation is L, the secret vault and the signed
//! HITL approval are L1. A failing effect target surfaces as an error and
//! leaves the run resumable — error-policy routing (`on_node_failure`) is
//! K's retry layer.

use std::collections::BTreeMap;

use sbfb_workflow_core::compile::{CompiledNodeInstance, CompiledPlan, compiled_plan_digest};
use sbfb_workflow_core::graph::{JoinPolicy, WorkflowNode};

use crate::clock::Clock;
use crate::echo::{EffectTarget, TargetError};
use crate::engine::EngineError;
use crate::event::{CancelOrigin, TerminalReason};
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
use crate::store::{RunStatus, RunStore, StoreError};

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

/// Host-side configuration of one orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Lease owner identity of this writer.
    pub writer_id: String,
    /// Lease TTL (takeover heuristic horizon).
    pub lease_ttl_ms: i64,
    /// Bounded in-flight dispatch cap, ≥ 1.
    pub max_in_flight: usize,
}

impl OrchestratorConfig {
    /// A config with the default knobs for `writer_id`.
    pub fn new(writer_id: impl Into<String>) -> Self {
        Self {
            writer_id: writer_id.into(),
            lease_ttl_ms: DEFAULT_LEASE_TTL_MS,
            max_in_flight: DEFAULT_MAX_IN_FLIGHT,
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

    /// Run — or resume — `run_id` of `plan` with `input` against `target`,
    /// draining `signals`, until the run completes, parks on an open
    /// interruption, or closes. Calling it again with the same identifiers
    /// is the RESUME path: memoised effects are served from the journal
    /// and the target is not touched again (§12.6 — resuming never re-runs
    /// a confirmed effect).
    #[allow(clippy::too_many_lines)]
    pub fn execute(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        input: &serde_json::Value,
        target: &mut dyn EffectTarget,
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
            // journal before anything else this pass.
            for (timer_id, _deadline) in self.store.due_timers(run_id, now)? {
                if timer_id == WORKFLOW_TIMEOUT_TIMER_ID {
                    self.store.fire_timer_with_cancel(
                        run_id,
                        &timer_id,
                        CancelOrigin::WorkflowTimeout,
                    )?;
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
                    // pending, honestly journaled; a result that ARRIVES
                    // late lands through `record_late_completion` (kept,
                    // frontier untouched) and full reconciliation is the
                    // Phase K effect layer.
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
            }
            candidates.sort();
            candidates.dedup();
            let batch: Vec<String> = candidates
                .into_iter()
                .take(self.config.max_in_flight.max(1))
                .collect();
            for rid in &batch {
                let (did_execute, did_memoise) = self.dispatch(plan, run_id, rid, target)?;
                executed += usize::from(did_execute);
                memoised += usize::from(did_memoise);
                progressed = true;
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

    /// Dispatch one READY node. Returns `(executed, memoised)` for the
    /// effect leaves; routing decisions count as neither.
    fn dispatch(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        target: &mut dyn EffectTarget,
    ) -> Result<(bool, bool), OrchestratorError> {
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
                Ok((false, false))
            }
            WorkflowNode::Classifier(node) => {
                // The model call is an EFFECT (echo in Phase J); the label
                // is read from the effect OUTPUT and validated against the
                // closed set — an out-of-set proposal routes to `default`
                // deterministically (§19.3), it never mints a port.
                let (did_execute, did_memoise) = self.run_effect(plan, run_id, rid, target)?;
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
                Ok((did_execute, did_memoise))
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
                Ok((false, false))
            }
            // Repeat/Subworkflow activate by DERIVATION (their children
            // enter the ready-set); Parallel/Join/Waits never reach the
            // ready-set at all.
            WorkflowNode::Repeat(_)
            | WorkflowNode::Subworkflow(_)
            | WorkflowNode::Parallel(_)
            | WorkflowNode::Join(_)
            | WorkflowNode::WaitSignal(_)
            | WorkflowNode::HumanReview(_) => Ok((false, false)),
            // Every remaining kind is an effect leaf on the echo activity.
            _ => self.run_effect(plan, run_id, rid, target),
        }
    }

    /// The Phase E per-step mechanics — memoised / pending / intent /
    /// apply / completion — against the production store doors. The spike
    /// engine keeps its own linear copy: its crash harness is the frozen
    /// proof of Phase E, and this walk must not disturb it; both paths go
    /// through the SAME store doors, which carry the guarantees.
    fn run_effect(
        &mut self,
        plan: &CompiledPlan,
        run_id: &str,
        rid: &str,
        target: &mut dyn EffectTarget,
    ) -> Result<(bool, bool), OrchestratorError> {
        let input = self.effect_input_map(plan, run_id, rid)?;
        let request = StepRequest::new(rid.to_string(), target.name(), input.clone())?;
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
            return Ok((false, true));
        }

        let effect_id = match self.store.pending_effect(run_id, rid)? {
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
                if pending.target_idempotent != target.is_idempotent() {
                    return Err(EngineError::TargetIdempotencyChanged {
                        node_instance_id: rid.to_string(),
                        journaled: pending.target_idempotent,
                        live: target.is_idempotent(),
                    }
                    .into());
                }
                if pending.dedup_scope != target.dedup_scope() {
                    return Err(EngineError::TargetScopeChanged {
                        node_instance_id: rid.to_string(),
                        journaled: pending.dedup_scope,
                        live: target.dedup_scope(),
                    }
                    .into());
                }
                if !pending.target_idempotent {
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
        Ok((true, false))
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
            _ => self.node_settled_successfully(plan, run_id, from_structural, from_rid),
        }
    }

    fn node_settled_successfully(
        &self,
        plan: &CompiledPlan,
        run_id: &str,
        _structural: &str,
        rid: &str,
    ) -> Result<bool, OrchestratorError> {
        let events = self.store.load_events(run_id)?;
        let view = reduce(plan, &events)?;
        Ok(view.node_states.get(rid).copied() == Some(NodeState::Succeeded))
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
                    // succeeded on another port).
                    let delivered = plan
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
                        })?;
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
            // Effect leaves: invert the echo receipt (JCS of the input
            // map; single-port maps collapse to the port's value).
            _ => {
                let done = self
                    .store
                    .completed_effect(run_id, rid)?
                    .ok_or_else(|| OrchestratorError::Plan(format!("{rid} has no receipt yet")))?;
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
