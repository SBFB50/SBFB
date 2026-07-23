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
//! ## States deliberately never derived in Phase J
//!
//! The full §12.3 vocabulary is typed here, honestly wider than what J
//! emits: `created`/`validated`/`queued` (run) belong to the Hub submission
//! pipeline; `dispatched`/`failed_retryable`/`failed_terminal` AND the
//! node-level `outcome_unknown` (node) to the Phase K effect layer
//! (retries, timeouts, error routing, per-node reconciliation — at J the
//! RUN goes `outcome_unknown` through the resume path while the node shows
//! its last derived state). They exist as variants so the vocabulary is
//! stable, and this module documents that no reduction path produces them
//! yet. Counting note: the research §12.3 list carries 11 node states; the
//! plan's prose says "10 états de node" — this module types the RESEARCH
//! list, the wider of the two.

use std::collections::BTreeMap;

use sbfb_workflow_core::compile::{CompiledEdge, CompiledNodeInstance, CompiledPlan};
use sbfb_workflow_core::graph::{JoinPolicy, WorkflowNode};

use crate::event::{RunEvent, RunEventKind, TerminalReason};
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

/// §12.3 node-instance states. `Dispatched`, `FailedRetryable` and
/// `FailedTerminal` are typed but never derived in Phase J (the effect
/// layer of Phase K owns them).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Not yet reachable: some input is still pending.
    Pending,
    /// Every input satisfied — the scheduler may dispatch it.
    Ready,
    /// Typed, never derived in Phase J (K's dispatch/ack split).
    Dispatched,
    /// An effect was intended and not yet completed.
    Running,
    /// Parked on an open interruption (`wait.signal`, `human.review`).
    Waiting,
    /// Done: effect completed, decision journaled, or structure satisfied.
    Succeeded,
    /// Typed, never derived in Phase J (K's retry layer).
    FailedRetryable,
    /// Typed, never derived in Phase J (K's retry layer).
    FailedTerminal,
    /// Dead path: every inbound alternative is dead (WCP-30).
    Skipped,
    /// The run closed as cancelled while this node was not yet settled.
    Cancelled,
    /// Typed, never derived in Phase J at node level (the RUN goes
    /// `outcome_unknown` through the resume path; per-node refinement is
    /// the K reconciliation layer).
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
}

/// What the event scan extracted — decisions and effects, keyed by RUNTIME
/// id, before any topology work.
#[derive(Debug, Default)]
struct Facts {
    intended: BTreeMap<String, ()>,
    completed: BTreeMap<String, ()>,
    /// One entry per `branch_taken` on the node, in journal order (a
    /// repeat accumulates one verdict per iteration).
    branches: BTreeMap<String, Vec<String>>,
    fanout: BTreeMap<String, u32>,
    signal_consumed: BTreeMap<String, ()>,
    human_decision: BTreeMap<String, String>,
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
    for (id, state) in &node_states {
        match state {
            // Emission stops at the first cancellation entry (§12.7).
            // Load-bearing for every EXTERNAL consumer of the view; the
            // orchestrator's own loop additionally short-circuits on
            // `CancelRequested` before its dispatch step (the M3/M3'
            // mutation pair proved which guard carries which weight).
            NodeState::Ready if !cancel_requested => ready.push(id.clone()),
            NodeState::Waiting if !cancel_requested => waiting.push(id.clone()),
            _ => {}
        }
    }

    let fully_settled = !node_states.is_empty() && node_states.values().all(|s| s.is_settled());

    let run_state = match resolver.facts.terminal_reason {
        Some(TerminalReason::Cancelled) => RunState::Cancelled,
        Some(TerminalReason::OutcomeUnknown) => RunState::OutcomeUnknown,
        Some(TerminalReason::StepHashMismatch) => RunState::Failed,
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
            RunEventKind::EffectIntended { .. } => {
                if let Some(rid) = &event.node_instance_id {
                    facts.intended.insert(rid.clone(), ());
                }
            }
            RunEventKind::EffectCompleted { .. } => {
                if let Some(rid) = &event.node_instance_id {
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
                // journaled decision; both must exist to be done.
                if self.facts.branches.contains_key(runtime) {
                    return NodeState::Succeeded;
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

            // ── effect leaves (echo activity in Phase J) ──────────────────
            _ => {
                if self.facts.completed.contains_key(runtime) {
                    return NodeState::Succeeded;
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

    /// Join derivation: policy over DECLARED sources, re-observed here at
    /// runtime (H normalised joins as liveness, never authority — the
    /// reducer counts real arrivals).
    fn join_state(
        &mut self,
        join: &sbfb_workflow_core::graph::JoinNode,
        _structural: &str,
        _runtime: &str,
        scope_structural: &str,
        scope_runtime: &str,
    ) -> NodeState {
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
            NodeState::Skipped | NodeState::Cancelled | NodeState::FailedTerminal => {
                EdgeDelivery::Dead
            }
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
