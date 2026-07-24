// SPDX-License-Identifier: AGPL-3.0-or-later
//! The reducer: the §12.3 run/node state machine as a PURE function of
//! `(compiled plan, journal)` — Sprint 83 Phase J.
//!
//! ## What "état rendu = réduction du journal" means here
//!
//! The journal records DECISIONS and EFFECTS: intents, completions, routing
//! decisions (`branch_taken`), map fan-outs, signal/decision consumptions,
//! cancellation. Everything else — pending, ready, waiting, skipped,
//! succeeded-by-structure — is DERIVED from those entries plus the plan's
//! topology, on every reduction, from scratch. The reducer never parses a
//! material payload: control flow is rebuilt from decision entries alone,
//! which is what makes `deterministic_replay` a theorem about this function
//! rather than a property of scheduler memory (G-AUTHORITY, THREAT §19
//! replay-divergence mitigation).
//!
//! ## Dead-path elimination (WCP-30)
//!
//! A `switch`/`classifier`/`human.review` delivers on exactly ONE output
//! port. Every node whose EVERY inbound alternative is dead (its source
//! skipped, or a routing source that took another port) derives `skipped`,
//! and skip PROPAGATES. A `join` fires on its declared policy counting
//! skipped sources as never-arriving — a join fed only by dead branches is
//! itself skipped, never a silent forever-hang.
//!
//! ## Effect-layer states (Sprint 83 Phase K)
//!
//! Phase K derives the states J only typed: `failed_retryable` (journaled
//! attempt failures under the scope's retry budget), `failed_terminal`
//! (a non-retryable failure, or the budget exhausted — under
//! `route-error-port` the node then DELIVERS on its reserved `error`
//! port), and the node-level `outcome_unknown` (a journaled
//! `node_outcome_unknown` parks the node for reconciliation; the run
//! stays alive). A parked node is settled by EVIDENCE only: a completion
//! whose receipt was recovered through a journaled reconciliation — the
//! scan refuses a completion on a parked node with no
//! `reconciliation_recorded` before it, so a reconciliation can never be
//! skipped and an unknown can never be laundered into a success.
//! Still never derived here: `created`/`validated`/`queued` (run — Hub
//! submission pipeline) and `dispatched` (the dispatch/ack split needs a
//! transport that acknowledges; the local activity layer has none).
//! Counting note: the research §12.3 list carries 11 node states; the
//! plan's prose says "10 états de node" — this module types the RESEARCH
//! list, the wider of the two.

use std::collections::BTreeMap;

use sbfb_workflow_core::compile::{CompiledEdge, CompiledNodeInstance, CompiledPlan, ERROR_PORT};
use sbfb_workflow_core::graph::{JoinPolicy, WorkflowNode};

use crate::event::{ReconcileResolution, RunEvent, RunEventKind, TerminalReason};
use crate::runtime_id::{child_structural, extend_scope, parent_scope, structural_id};

/// The engine-internal repeat verdict journaled as a `branch_taken` port:
/// the loop body runs again.
pub const REPEAT_PORT_CONTINUE: &str = "continue";
/// The engine-internal repeat verdict: the loop stops before (or at) its
/// structural bound.
pub const REPEAT_PORT_STOP: &str = "stop";
/// The reserved default port of `switch`/`classifier` routing.
pub const DEFAULT_PORT: &str = "default";

/// §12.3 run states. The reducer derives the ones Phase J can produce;
/// `Created`/`Validated`/`Queued` are typed for vocabulary stability and
/// documented as never-derived here (they belong to the Hub submission
/// pipeline, before an engine run exists).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    /// Typed, never derived by this reducer (pre-engine lifecycle).
    Created,
    /// Typed, never derived by this reducer (pre-engine lifecycle).
    Validated,
    /// Typed, never derived by this reducer (pre-engine lifecycle).
    Queued,
    /// The run advances (or is resumable).
    Running,
    /// At least one `human.review` interruption is open.
    WaitingHuman,
    /// At least one `wait.signal` interruption is open (and no human one —
    /// the human wait wins the label when both are open, because a human
    /// is the slower, scarcer resource to surface).
    WaitingExternal,
    /// A cancellation was journaled and the run has not yet closed.
    CancelRequested,
    /// Closed: every reachable node settled and nothing failed.
    Completed,
    /// Closed by a fail-closed diagnosis (hash divergence — the Phase J
    /// vocabulary for a terminal that is neither cancel nor unknown).
    Failed,
    /// Closed as cancelled (§12.7): emission stopped, late results kept
    /// but not propagated.
    Cancelled,
    /// Closed because an unconfirmed non-idempotent effect makes the
    /// honest outcome unknowable (§12.5).
    OutcomeUnknown,
}

/// §12.3 node-instance states. Phase K derives `FailedRetryable`,
/// `FailedTerminal` and the node-level `OutcomeUnknown`; only
/// `Dispatched` remains typed-never-derived (a dispatch/ack split needs
/// an acknowledging transport the local activity layer does not have).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Not yet reachable: some input is still pending.
    Pending,
    /// Every input satisfied — the scheduler may dispatch it.
    Ready,
    /// Typed, never derived (a dispatch/ack split needs an acknowledging
    /// transport; the local activity layer has none).
    Dispatched,
    /// An effect was intended and not yet completed.
    Running,
    /// Parked on an open interruption (`wait.signal`, `human.review`).
    Waiting,
    /// Done: effect completed, decision journaled, or structure satisfied.
    Succeeded,
    /// The last journaled attempt failed retryably and the scope's retry
    /// budget is not exhausted: the scheduler may re-dispatch it (paced by
    /// the durable retry timer, which changes timing, never outcomes).
    FailedRetryable,
    /// A non-retryable failure, or the retry budget exhausted. Settled:
    /// under `route-error-port` the node delivers on its reserved `error`
    /// port; under `fail-workflow` the orchestrator closes the run
    /// (`node_failed`).
    FailedTerminal,
    /// Dead path: every inbound alternative is dead (WCP-30).
    Skipped,
    /// The run closed as cancelled while this node was not yet settled.
    Cancelled,
    /// PARKED (§12.5): a journaled `node_outcome_unknown` — the effect's
    /// outcome is unknowable, the node awaits an operator reconciliation,
    /// and the scheduler never re-dispatches it. Not settled (the run
    /// cannot auto-complete around it) and not ready.
    OutcomeUnknown,
}

impl NodeState {
    /// A settled node needs nothing more from the scheduler.
    pub fn is_settled(self) -> bool {
        matches!(
            self,
            Self::Succeeded | Self::Skipped | Self::Cancelled | Self::FailedTerminal
        )
    }
}

/// The reduction of one run: pure data, rebuilt from scratch on every call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunView {
    /// Derived §12.3 run state.
    pub run_state: RunState,
    /// State of every MATERIALISED node instance (runtime ids: iteration
    /// suffixes included; an inactive iteration's nodes simply do not
    /// appear yet).
    pub node_states: BTreeMap<String, NodeState>,
    /// Runtime ids the scheduler may dispatch NOW, in canonical order —
    /// empty once a cancellation is journaled (emission stops, §12.7).
    pub ready: Vec<String>,
    /// Runtime ids parked on an open interruption, in canonical order.
    pub waiting: Vec<String>,
    /// Runtime ids parked as `outcome_unknown` awaiting reconciliation
    /// (Phase K), in canonical order. On a run CLOSED as cancelled the
    /// cancel overlay wins (the node reads `cancelled`, reconciliation is
    /// refused on a closed run anyway); the journal keeps the park entry
    /// either way.
    pub parked_unknown: Vec<String>,
    /// A cancellation entry exists (whether or not the run closed yet).
    pub cancel_requested: bool,
    /// Every materialised node is settled (and the root scope is fully
    /// derived): the scheduler may close the run.
    pub fully_settled: bool,
}

/// The reduction failed — the journal and the plan disagree structurally.
/// Loud, never repaired: a journal that names unknown nodes was written
/// against ANOTHER plan, and deriving anything from it would be the
/// stale-answer class the spec digest exists to refuse.
#[derive(Debug, thiserror::Error)]
pub enum ReduceError {
    /// An event names a node instance the plan cannot contain.
    #[error("journal names unknown node instance: {runtime_id}")]
    UnknownNode {
        /// The offending runtime id.
        runtime_id: String,
    },
    /// A `map_fanout` entry exceeds the structural bound of the signed
    /// plan. The scheduler refuses BEFORE journaling, so this can only be
    /// a corrupt or foreign journal.
    #[error("journaled fan-out {cardinality} exceeds max_cardinality {bound} at {runtime_id}")]
    FanoutBeyondBound {
        /// The map node.
        runtime_id: String,
        /// Journaled cardinality.
        cardinality: u32,
        /// Structural bound of the plan.
        bound: u32,
    },
    /// The journal is CAUSALLY invalid (Phase K hardening, THREAT §19.1
    /// replay-divergence carry): a completion with no intent before it, an
    /// attempt failure with no intent, or a completion on a PARKED node
    /// with no reconciliation before it — the laundering shape. The
    /// production writers cannot produce these orders; a journal that
    /// carries one was written by something else, and deriving from it
    /// would trust what the doors never allowed.
    #[error("journal causally invalid at {runtime_id}: {reason}")]
    CausalOrder {
        /// The offending node instance.
        runtime_id: String,
        /// Which order was violated.
        reason: &'static str,
    },
}

/// What the event scan extracted — decisions and effects, keyed by RUNTIME
/// id, before any topology work.
#[derive(Debug, Default)]
struct Facts {
    /// Node → the effect id its `EffectIntended` minted. Every later
    /// effect-layer event on the node (completion, attempt failure, park,
    /// reconciliation) must carry THIS id — a journal that mixes effect ids
    /// on one node is causally invalid (Phase K).
    intended: BTreeMap<String, String>,
    completed: BTreeMap<String, ()>,
    /// One entry per `branch_taken` on the node, in journal order (a
    /// repeat accumulates one verdict per iteration).
    branches: BTreeMap<String, Vec<String>>,
    fanout: BTreeMap<String, u32>,
    signal_consumed: BTreeMap<String, ()>,
    human_decision: BTreeMap<String, String>,
    /// Retryable flags of journaled attempt failures, in journal order
    /// (Phase K).
    attempt_failures: BTreeMap<String, Vec<bool>>,
    /// Node → effect id it parked `outcome_unknown` awaiting reconciliation
    /// (Phase K).
    parked_unknown: BTreeMap<String, String>,
    /// Node → effect id whose parked unknown was reconciled
    /// `result_recovered` — the causal licence for a later completion of
    /// THAT effect (Phase K).
    reconciled_recovered: BTreeMap<String, String>,
    /// Joins whose delivered-source cohort was FROZEN by a journaled
    /// `join_fired` (Phase K): the reducer only needs to know a join has
    /// fired (it then reads Succeeded); the frozen cohort itself is read
    /// by the orchestrator's value resolution, straight from the journal.
    join_fired: BTreeMap<String, ()>,
    cancel_requested: bool,
    run_completed: bool,
    terminal_reason: Option<TerminalReason>,
}

/// Topology of one scope, precomputed once per reduction.
struct ScopeIndex<'p> {
    /// Structural ids of the scope's nodes, sorted (BTreeMap order).
    node_ids: Vec<&'p str>,
    /// Inbound edges per destination structural id.
    in_edges: BTreeMap<&'p str, Vec<&'p CompiledEdge>>,
    /// Entry structural ids of the scope.
    entrypoints: Vec<&'p str>,
}

/// Everything the derivation needs, plus the memo cache.
struct Resolver<'p> {
    plan: &'p CompiledPlan,
    scopes: BTreeMap<&'p str, ScopeIndex<'p>>,
    facts: Facts,
    /// Memoised node states by RUNTIME id.
    cache: BTreeMap<String, NodeState>,
}

/// Reduce a journal against its compiled plan.
pub fn reduce(plan: &CompiledPlan, events: &[RunEvent]) -> Result<RunView, ReduceError> {
    let facts = scan(plan, events)?;
    let mut resolver = Resolver {
        plan,
        scopes: index_scopes(plan),
        facts,
        cache: BTreeMap::new(),
    };

    // Derive the root scope; child scopes materialise recursively as their
    // expanding nodes activate.
    resolver.derive_scope("", "");

    let node_states = resolver.cache.clone();
    let cancel_requested = resolver.facts.cancel_requested;
    let closed_cancelled = resolver.facts.terminal_reason == Some(TerminalReason::Cancelled);

    // Cancel overlay: once the run CLOSED as cancelled, every unsettled
    // node is `cancelled` — the journal keeps what really happened, the
    // view says honestly that the rest never will.
    let node_states: BTreeMap<String, NodeState> = if closed_cancelled {
        node_states
            .into_iter()
            .map(|(id, s)| {
                if s.is_settled() || s == NodeState::Succeeded {
                    (id, s)
                } else {
                    (id, NodeState::Cancelled)
                }
            })
            .collect()
    } else {
        node_states
    };

    let mut ready: Vec<String> = Vec::new();
    let mut waiting: Vec<String> = Vec::new();
    let mut parked_unknown: Vec<String> = Vec::new();
    for (id, state) in &node_states {
        match state {
            // Emission stops at the first cancellation entry (§12.7).
            // Load-bearing for every EXTERNAL consumer of the view; the
            // orchestrator's own loop additionally short-circuits on
            // `CancelRequested` before its dispatch step (the M3/M3'
            // mutation pair proved which guard carries which weight).
            NodeState::Ready if !cancel_requested => ready.push(id.clone()),
            NodeState::Waiting if !cancel_requested => waiting.push(id.clone()),
            // Parked nodes stay listed under a mere cancel REQUEST; once
            // the run CLOSED as cancelled the overlay above already
            // relabelled them.
            NodeState::OutcomeUnknown => parked_unknown.push(id.clone()),
            _ => {}
        }
    }

    let fully_settled = !node_states.is_empty() && node_states.values().all(|s| s.is_settled());

    let run_state = match resolver.facts.terminal_reason {
        Some(TerminalReason::Cancelled) => RunState::Cancelled,
        Some(TerminalReason::OutcomeUnknown) => RunState::OutcomeUnknown,
        Some(TerminalReason::StepHashMismatch) | Some(TerminalReason::NodeFailed) => {
            RunState::Failed
        }
        None if resolver.facts.run_completed => RunState::Completed,
        None if cancel_requested => RunState::CancelRequested,
        None => {
            let waits_human = waiting.iter().any(|id| {
                matches!(
                    resolver.instance(id).map(|i| &i.node),
                    Some(WorkflowNode::HumanReview(_))
                )
            });
            if waits_human {
                RunState::WaitingHuman
            } else if !waiting.is_empty() {
                RunState::WaitingExternal
            } else {
                RunState::Running
            }
        }
    };

    Ok(RunView {
        run_state,
        node_states,
        ready,
        waiting,
        parked_unknown,
        cancel_requested,
        fully_settled,
    })
}

fn scan(plan: &CompiledPlan, events: &[RunEvent]) -> Result<Facts, ReduceError> {
    let mut facts = Facts::default();
    for event in events {
        if let Some(rid) = &event.node_instance_id {
            // Every journaled node must project onto the plan.
            let structural = structural_id(rid);
            if !plan.node_instances.contains_key(&structural) {
                return Err(ReduceError::UnknownNode {
                    runtime_id: rid.clone(),
                });
            }
        }
        match &event.kind {
            RunEventKind::RunStarted => {}
            RunEventKind::EffectIntended { effect_id } => {
                if let Some(rid) = &event.node_instance_id {
                    facts.intended.insert(rid.clone(), effect_id.clone());
                }
            }
            RunEventKind::EffectCompleted { effect_id, .. } => {
                if let Some(rid) = &event.node_instance_id {
                    // Causal order (Phase K): a completion needs an intent
                    // before it, CARRYING THE SAME effect id, and a
                    // completion on a PARKED node needs a reconciliation of
                    // THAT effect before it — the laundering shape.
                    if facts.intended.get(rid) != Some(effect_id) {
                        return Err(ReduceError::CausalOrder {
                            runtime_id: rid.clone(),
                            reason: "completion without a matching prior intent",
                        });
                    }
                    if let Some(parked_effect) = facts.parked_unknown.get(rid)
                        && facts.reconciled_recovered.get(rid) != Some(parked_effect)
                    {
                        return Err(ReduceError::CausalOrder {
                            runtime_id: rid.clone(),
                            reason: "completion on a parked node without a matching reconciliation",
                        });
                    }
                    facts.completed.insert(rid.clone(), ());
                }
            }
            RunEventKind::BranchTaken { port } => {
                if let Some(rid) = &event.node_instance_id {
                    facts
                        .branches
                        .entry(rid.clone())
                        .or_default()
                        .push(port.clone());
                }
            }
            RunEventKind::MapFanout { cardinality } => {
                if let Some(rid) = &event.node_instance_id {
                    let structural = structural_id(rid);
                    if let Some(instance) = plan.node_instances.get(&structural)
                        && let WorkflowNode::Map(map) = &instance.node
                        && *cardinality > map.max_cardinality.get()
                    {
                        return Err(ReduceError::FanoutBeyondBound {
                            runtime_id: rid.clone(),
                            cardinality: *cardinality,
                            bound: map.max_cardinality.get(),
                        });
                    }
                    facts.fanout.insert(rid.clone(), *cardinality);
                }
            }
            RunEventKind::SignalConsumed { .. } => {
                if let Some(rid) = &event.node_instance_id {
                    facts.signal_consumed.insert(rid.clone(), ());
                }
            }
            RunEventKind::HumanDecisionRecorded { decision_label, .. } => {
                if let Some(rid) = &event.node_instance_id {
                    facts
                        .human_decision
                        .insert(rid.clone(), decision_label.clone());
                }
            }
            RunEventKind::CancelRequested { .. } => {
                facts.cancel_requested = true;
            }
            RunEventKind::EffectAttemptFailed {
                effect_id,
                attempt: _,
                retryable,
            } => {
                if let Some(rid) = &event.node_instance_id {
                    if facts.intended.get(rid) != Some(effect_id) {
                        return Err(ReduceError::CausalOrder {
                            runtime_id: rid.clone(),
                            reason: "attempt failure without a matching prior intent",
                        });
                    }
                    facts
                        .attempt_failures
                        .entry(rid.clone())
                        .or_default()
                        .push(*retryable);
                }
            }
            RunEventKind::NodeOutcomeUnknown { effect_id } => {
                if let Some(rid) = &event.node_instance_id {
                    if facts.intended.get(rid) != Some(effect_id) {
                        return Err(ReduceError::CausalOrder {
                            runtime_id: rid.clone(),
                            reason: "park without a matching prior intent",
                        });
                    }
                    facts.parked_unknown.insert(rid.clone(), effect_id.clone());
                }
            }
            RunEventKind::ReconciliationRecorded {
                effect_id,
                resolution,
                ..
            } => {
                if let Some(rid) = &event.node_instance_id
                    && *resolution == ReconcileResolution::ResultRecovered
                {
                    // A reconciliation only LICENSES a later completion if
                    // the node was ALREADY parked FOR THIS EFFECT — the
                    // journal order must be park → reconciliation →
                    // completion, all on the same effect id. Refuse a
                    // reconciliation that precedes its park, or names a
                    // different effect (the shapes that would pre-authorise a
                    // laundering completion).
                    if facts.parked_unknown.get(rid) != Some(effect_id) {
                        return Err(ReduceError::CausalOrder {
                            runtime_id: rid.clone(),
                            reason: "reconciliation without a matching prior park",
                        });
                    }
                    facts
                        .reconciled_recovered
                        .insert(rid.clone(), effect_id.clone());
                }
            }
            RunEventKind::JoinFired { sources: _ } => {
                if let Some(rid) = &event.node_instance_id {
                    // Presence is all the reducer needs (see `Facts`); the
                    // cohort is read by the orchestrator's value resolution.
                    facts.join_fired.insert(rid.clone(), ());
                }
            }
            RunEventKind::RunCompleted => {
                facts.run_completed = true;
            }
            RunEventKind::RunTerminal { reason, .. } => {
                facts.terminal_reason = Some(*reason);
            }
        }
    }
    Ok(facts)
}

fn index_scopes(plan: &CompiledPlan) -> BTreeMap<&str, ScopeIndex<'_>> {
    let mut scopes: BTreeMap<&str, ScopeIndex<'_>> = BTreeMap::new();
    for (scope, entries) in &plan.entrypoints {
        scopes.insert(
            scope.as_str(),
            ScopeIndex {
                node_ids: Vec::new(),
                in_edges: BTreeMap::new(),
                entrypoints: entries.iter().map(String::as_str).collect(),
            },
        );
    }
    for id in plan.node_instances.keys() {
        let scope = parent_scope(id);
        if let Some(index) = scopes.get_mut(scope) {
            index.node_ids.push(id.as_str());
        }
    }
    for edge in &plan.edges {
        let scope = parent_scope(&edge.to);
        if let Some(index) = scopes.get_mut(scope) {
            index
                .in_edges
                .entry(edge.to.as_str())
                .or_default()
                .push(edge);
        }
    }
    scopes
}

/// Tri-state of a node's inputs during derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Inputs {
    /// Every required port has a delivered value.
    Satisfied,
    /// Some port can never be delivered — the node is on a dead path.
    Dead,
    /// Still waiting on an upstream node.
    Pending,
}

impl<'p> Resolver<'p> {
    fn instance(&self, runtime_id: &str) -> Option<&'p CompiledNodeInstance> {
        self.plan.node_instances.get(&structural_id(runtime_id))
    }

    /// Derive every node of `scope_structural` under the RUNTIME scope
    /// `scope_runtime`, recursing into activated child scopes.
    fn derive_scope(&mut self, scope_structural: &str, scope_runtime: &str) {
        let (node_ids, entry_check): (Vec<String>, bool) = match self.scopes.get(scope_structural) {
            Some(index) => (
                index.node_ids.iter().map(|s| (*s).to_string()).collect(),
                true,
            ),
            None => (Vec::new(), false),
        };
        if !entry_check {
            return;
        }
        for structural in node_ids {
            let runtime = runtime_of(scope_structural, scope_runtime, &structural);
            let state = self.state(&structural, &runtime, scope_structural, scope_runtime);
            self.cache.insert(runtime.clone(), state);

            // Child scopes materialise for activated expanding nodes.
            let Some(instance) = self.plan.node_instances.get(&structural) else {
                continue;
            };
            match &instance.node {
                WorkflowNode::Subworkflow(_) if self.scope_entered(&runtime, state) => {
                    self.derive_scope(&structural, &runtime);
                }
                WorkflowNode::Map(_) => {
                    if let Some(n) = self.facts.fanout.get(&runtime).copied() {
                        for i in 0..n {
                            let child_runtime = extend_scope(&runtime, i);
                            self.derive_scope(&structural, &child_runtime);
                        }
                    }
                }
                WorkflowNode::Repeat(node) if self.scope_entered(&runtime, state) => {
                    let verdicts = self
                        .facts
                        .branches
                        .get(&runtime)
                        .cloned()
                        .unwrap_or_default();
                    // Iteration 0 exists as soon as the repeat is
                    // entered; iteration i+1 exists after a journaled
                    // `continue` verdict for iteration i, up to the
                    // STRUCTURAL bound of the signed plan.
                    let mut active = 1usize;
                    for verdict in &verdicts {
                        if verdict == REPEAT_PORT_CONTINUE {
                            active += 1;
                        } else {
                            break;
                        }
                    }
                    let bound = node.max_iterations.get() as usize;
                    for i in 0..active.min(bound) {
                        let child_runtime = extend_scope(&runtime, i as u32);
                        self.derive_scope(&structural, &child_runtime);
                    }
                }
                _ => {}
            }
        }
    }

    /// An expanding node's child scope is entered once the node stopped
    /// being pending/skipped: its inputs are satisfied (Ready) or its
    /// children are in flight (Running/Succeeded).
    fn scope_entered(&self, _runtime: &str, state: NodeState) -> bool {
        matches!(
            state,
            NodeState::Ready | NodeState::Running | NodeState::Succeeded
        )
    }

    /// The state of one node — memoised; recursion is bounded by the DAG
    /// the compiler proved.
    fn state(
        &mut self,
        structural: &str,
        runtime: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> NodeState {
        if let Some(cached) = self.cache.get(runtime) {
            return *cached;
        }
        let state = self.derive_state(structural, runtime, scope_structural, scope_runtime);
        self.cache.insert(runtime.to_string(), state);
        state
    }

    fn derive_state(
        &mut self,
        structural: &str,
        runtime: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> NodeState {
        let Some(instance) = self.plan.node_instances.get(structural) else {
            return NodeState::Pending;
        };
        let node = instance.node.clone();

        match &node {
            // ── interruptions ─────────────────────────────────────────────
            WorkflowNode::WaitSignal(_) => {
                if self.facts.signal_consumed.contains_key(runtime) {
                    return NodeState::Succeeded;
                }
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Satisfied => NodeState::Waiting,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }
            WorkflowNode::HumanReview(_) => {
                if self.facts.human_decision.contains_key(runtime) {
                    return NodeState::Succeeded;
                }
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Satisfied => NodeState::Waiting,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }

            // ── pure structure ────────────────────────────────────────────
            WorkflowNode::Parallel(_) => {
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    // A parallel node only BROADCASTS: structure satisfied
                    // ⇒ done, no decision, no effect, nothing journaled.
                    Inputs::Satisfied => NodeState::Succeeded,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }
            WorkflowNode::Join(join) => {
                self.join_state(join, structural, runtime, scope_structural, scope_runtime)
            }

            // ── routing with a journaled decision ─────────────────────────
            WorkflowNode::Switch(_) => {
                if self.facts.branches.contains_key(runtime) {
                    return NodeState::Succeeded;
                }
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Satisfied => NodeState::Ready,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }
            WorkflowNode::Classifier(_) => {
                // A classifier is an EFFECT (the model call) plus a
                // journaled decision; both must exist to be done. Its
                // effect can fail or park like any leaf (Phase K); a
                // completed effect whose decision is not yet journaled
                // stays Running.
                if self.facts.branches.contains_key(runtime) {
                    return NodeState::Succeeded;
                }
                if self.facts.completed.contains_key(runtime) {
                    return NodeState::Running;
                }
                if let Some(state) = self.failure_state(structural, runtime) {
                    return state;
                }
                if self.facts.intended.contains_key(runtime) {
                    return NodeState::Running;
                }
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Satisfied => NodeState::Ready,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }

            // ── expanding nodes ───────────────────────────────────────────
            WorkflowNode::Subworkflow(_) => {
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                    Inputs::Satisfied => {
                        if self.child_scope_settled(structural, runtime) {
                            NodeState::Succeeded
                        } else {
                            NodeState::Running
                        }
                    }
                }
            }
            WorkflowNode::Map(_) => {
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                    Inputs::Satisfied => match self.facts.fanout.get(runtime).copied() {
                        None => NodeState::Ready,
                        Some(n) => {
                            let all = (0..n).all(|i| {
                                let child = extend_scope(runtime, i);
                                self.child_scope_settled_at(structural, &child)
                            });
                            if all {
                                NodeState::Succeeded
                            } else {
                                NodeState::Running
                            }
                        }
                    },
                }
            }
            WorkflowNode::Repeat(repeat) => {
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                    Inputs::Satisfied => {
                        let verdicts = self
                            .facts
                            .branches
                            .get(runtime)
                            .cloned()
                            .unwrap_or_default();
                        let bound = repeat.max_iterations.get() as usize;
                        let stopped = verdicts.iter().any(|v| v == REPEAT_PORT_STOP)
                            || verdicts.len() >= bound;
                        if stopped {
                            // Done once the LAST active iteration settled.
                            let last = verdicts.len().saturating_sub(1).min(bound - 1);
                            let child = extend_scope(runtime, last as u32);
                            if self.child_scope_settled_at(structural, &child) {
                                NodeState::Succeeded
                            } else {
                                NodeState::Running
                            }
                        } else {
                            NodeState::Running
                        }
                    }
                }
            }

            // ── effect leaves ─────────────────────────────────────────────
            _ => {
                if self.facts.completed.contains_key(runtime) {
                    // A completion always wins — on a previously parked
                    // node it can only exist behind a journaled
                    // reconciliation (the scan refuses it otherwise), so
                    // this success rests on recovered EVIDENCE, never on a
                    // re-label.
                    return NodeState::Succeeded;
                }
                if let Some(state) = self.failure_state(structural, runtime) {
                    return state;
                }
                if self.facts.intended.contains_key(runtime) {
                    return NodeState::Running;
                }
                match self.inputs(structural, runtime, scope_structural, scope_runtime) {
                    Inputs::Satisfied => NodeState::Ready,
                    Inputs::Dead => NodeState::Skipped,
                    Inputs::Pending => NodeState::Pending,
                }
            }
        }
    }

    /// Effect-layer progression shared by the effect leaves and the
    /// classifier (Phase K): a journaled park wins over failures, and a
    /// failure's terminality is a PURE function of the plan's scope error
    /// policy plus the count of journaled attempt failures.
    fn failure_state(&self, structural: &str, runtime: &str) -> Option<NodeState> {
        if self.facts.parked_unknown.contains_key(runtime) {
            return Some(NodeState::OutcomeUnknown);
        }
        let failures = self.facts.attempt_failures.get(runtime)?;
        let non_retryable = failures.iter().any(|retryable| !*retryable);
        // The budget is `1 + max_retries` attempts in total.
        let budget_exhausted = failures.len() > self.max_retries_for(structural) as usize;
        if non_retryable || budget_exhausted {
            Some(NodeState::FailedTerminal)
        } else {
            Some(NodeState::FailedRetryable)
        }
    }

    /// The scope's `max_retries_per_node`: exact scope policy, else the
    /// root's, else 0 (first failure is terminal — fail-closed, never an
    /// invented budget).
    fn max_retries_for(&self, structural: &str) -> u32 {
        let scope = parent_scope(structural);
        self.plan
            .error_policies
            .get(scope)
            .or_else(|| self.plan.error_policies.get(""))
            .map_or(0, |policy| policy.max_retries_per_node)
    }

    /// Join derivation: policy over DECLARED sources, re-observed here at
    /// runtime (H normalised joins as liveness, never authority — the
    /// reducer counts real arrivals).
    fn join_state(
        &mut self,
        join: &sbfb_workflow_core::graph::JoinNode,
        _structural: &str,
        runtime: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> NodeState {
        // A journaled `join_fired` FREEZES the decision (Phase K): the
        // cohort was recorded when the policy was satisfied, and a source
        // arriving later can no longer widen a quorum or replace an `any`
        // winner — re-observation stops here.
        if self.facts.join_fired.contains_key(runtime) {
            return NodeState::Succeeded;
        }
        let mut succeeded = 0usize;
        let mut dead = 0usize;
        let total = join.source_node_ids.len();
        for source in &join.source_node_ids {
            // Sources are node ids of the SAME scope (H validation).
            let source_structural = child_structural(scope_structural, source);
            let source_runtime = runtime_of(scope_structural, scope_runtime, &source_structural);
            let state = self.state(
                &source_structural,
                &source_runtime,
                scope_structural,
                scope_runtime,
            );
            // A source "arrived" when its EDGE toward this join DELIVERED,
            // not when the node merely succeeded (Codex r1 P1): a routing
            // source that succeeded by taking ANOTHER port never arrives
            // here — counting its state would fire the join on a value
            // that went elsewhere. Delivery reuses the same edge rules as
            // input derivation; a source with no edge to this join (a
            // plan H would refuse anyway) counts by its state, fail-open
            // to the stricter edge form being absent.
            let source_edges: Vec<&CompiledEdge> = self
                .plan
                .edges
                .iter()
                .filter(|e| e.from == source_structural && e.to == *_structural)
                .collect();
            let (delivered, all_dead) = if source_edges.is_empty() {
                (
                    state == NodeState::Succeeded,
                    matches!(
                        state,
                        NodeState::Skipped | NodeState::Cancelled | NodeState::FailedTerminal
                    ),
                )
            } else {
                let mut any_delivered = false;
                let mut every_dead = true;
                for edge in source_edges {
                    match self.edge_delivery(edge, &source_runtime, state) {
                        EdgeDelivery::Delivered => {
                            any_delivered = true;
                            every_dead = false;
                        }
                        EdgeDelivery::Dead => {}
                        EdgeDelivery::Pending => every_dead = false,
                    }
                }
                (any_delivered, every_dead)
            };
            if delivered {
                succeeded += 1;
            } else if all_dead {
                dead += 1;
            }
        }
        // `alive` = sources that arrived or still can (everything not on a
        // dead path). ALL fires when every LIVE source arrived — skipped
        // sources count as never-arriving, the WCP-30 structured partial
        // join, so a join fed through an exclusive switch never hangs on
        // the branch not taken. A join whose policy can no longer be
        // satisfied is itself SKIPPED (dead-path propagation), never a
        // silent forever-hang.
        let alive = total - dead;
        match &join.policy {
            JoinPolicy::All(_) => {
                if alive == 0 {
                    NodeState::Skipped
                } else if succeeded == alive {
                    NodeState::Succeeded
                } else {
                    NodeState::Pending
                }
            }
            JoinPolicy::Any(_) => {
                if succeeded >= 1 {
                    NodeState::Succeeded
                } else if alive == 0 {
                    NodeState::Skipped
                } else {
                    NodeState::Pending
                }
            }
            JoinPolicy::Quorum(q) => {
                let needed = q.quorum.get() as usize;
                if succeeded >= needed {
                    NodeState::Succeeded
                } else if alive < needed {
                    NodeState::Skipped
                } else {
                    NodeState::Pending
                }
            }
        }
    }

    /// Every node of the child scope of `expanding_structural`, derived
    /// under `child_runtime`, is settled.
    fn child_scope_settled_at(&mut self, expanding_structural: &str, child_runtime: &str) -> bool {
        let node_ids: Vec<String> = match self.scopes.get(expanding_structural) {
            Some(index) => index.node_ids.iter().map(|s| (*s).to_string()).collect(),
            None => return true,
        };
        node_ids.iter().all(|structural| {
            let runtime = runtime_of(expanding_structural, child_runtime, structural);
            self.state(structural, &runtime, expanding_structural, child_runtime)
                .is_settled()
        })
    }

    fn child_scope_settled(&mut self, expanding_structural: &str, runtime: &str) -> bool {
        self.child_scope_settled_at(expanding_structural, runtime)
    }

    /// Tri-state of a node's inputs (see [`Inputs`]).
    fn inputs(
        &mut self,
        structural: &str,
        _runtime: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> Inputs {
        let (edges, is_entry): (Vec<CompiledEdge>, bool) = match self.scopes.get(scope_structural) {
            Some(index) => (
                index
                    .in_edges
                    .get(structural)
                    .map(|edges| edges.iter().map(|e| (*e).clone()).collect())
                    .unwrap_or_default(),
                index.entrypoints.contains(&structural),
            ),
            None => (Vec::new(), false),
        };
        if edges.is_empty() {
            // Entrypoints have no inbound edges: they are satisfied as soon
            // as their scope is derived at all (the caller only derives an
            // ACTIVE scope). A non-entry node without edges cannot exist
            // (H reachability), but Pending is the safe answer.
            return if is_entry {
                Inputs::Satisfied
            } else {
                Inputs::Pending
            };
        }

        // Group inbound edges by destination port: a port is an OR over its
        // edges; the node is an AND over its ports.
        let mut ports: BTreeMap<&str, (bool, bool)> = BTreeMap::new(); // (delivered, all_dead)
        let mut by_port: BTreeMap<&str, Vec<&CompiledEdge>> = BTreeMap::new();
        for edge in &edges {
            by_port.entry(edge.to_port.as_str()).or_default().push(edge);
        }
        for (port, port_edges) in by_port {
            let mut delivered = false;
            let mut all_dead = true;
            for edge in port_edges {
                let from_structural = structural_id(&edge.from);
                let from_runtime = runtime_of(scope_structural, scope_runtime, &edge.from);
                let from_state = self.state(
                    &from_structural,
                    &from_runtime,
                    scope_structural,
                    scope_runtime,
                );
                match self.edge_delivery(edge, &from_runtime, from_state) {
                    EdgeDelivery::Delivered => {
                        delivered = true;
                        all_dead = false;
                    }
                    EdgeDelivery::Dead => {}
                    EdgeDelivery::Pending => {
                        all_dead = false;
                    }
                }
            }
            ports.insert(port, (delivered, all_dead));
        }

        if ports.values().any(|(_, all_dead)| *all_dead) {
            Inputs::Dead
        } else if ports.values().all(|(delivered, _)| *delivered) {
            Inputs::Satisfied
        } else {
            Inputs::Pending
        }
    }

    fn edge_delivery(
        &self,
        edge: &CompiledEdge,
        from_runtime: &str,
        from_state: NodeState,
    ) -> EdgeDelivery {
        let routing_port: Option<&str> = match self.instance(from_runtime).map(|i| &i.node) {
            Some(WorkflowNode::Switch(_)) | Some(WorkflowNode::Classifier(_)) => self
                .facts
                .branches
                .get(from_runtime)
                .and_then(|v| v.last())
                .map(String::as_str),
            Some(WorkflowNode::HumanReview(_)) => self
                .facts
                .human_decision
                .get(from_runtime)
                .map(String::as_str),
            _ => None,
        };
        match from_state {
            NodeState::Succeeded => match routing_port {
                // A routing source delivers on exactly ONE port; every
                // other outbound edge is dead (WCP-30 dead-path start).
                Some(taken) if taken == edge.from_port => EdgeDelivery::Delivered,
                Some(_) => EdgeDelivery::Dead,
                None => EdgeDelivery::Delivered,
            },
            // A terminally-failed node DELIVERS on its reserved `error`
            // port and is dead everywhere else (Phase K `route-error-port`;
            // the compiler only wires that port under the routing policy,
            // so a fail-workflow scope never has the edge to deliver on).
            NodeState::FailedTerminal => {
                if edge.from_port == ERROR_PORT {
                    EdgeDelivery::Delivered
                } else {
                    EdgeDelivery::Dead
                }
            }
            NodeState::Skipped | NodeState::Cancelled => EdgeDelivery::Dead,
            _ => EdgeDelivery::Pending,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdgeDelivery {
    Delivered,
    Dead,
    Pending,
}

/// The RUNTIME id of a node: replace its structural scope prefix by the
/// runtime scope (which may carry iteration suffixes).
fn runtime_of(scope_structural: &str, scope_runtime: &str, structural: &str) -> String {
    match structural.strip_prefix(scope_structural) {
        Some(rest) => format!("{scope_runtime}{rest}"),
        None => structural.to_string(),
    }
}
