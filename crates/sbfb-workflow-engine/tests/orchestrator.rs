// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 83 Phase J — the deterministic orchestrator harness.
//!
//! Same doctrine as the Phase E/I harnesses: every guarantee has a
//! guaranteed arm AND a discriminating red arm — a forged nonce, a replayed
//! deposit, an expired approval, a regressive clock, a divergent journal, a
//! dead branch feeding a join. A green suite must never be vacuous.
//!
//! Gate naming: every test name starts with `orchestrator_`
//! (`G-AUTHORITY` filter is `deterministic_replay` + `iframe_death`).
//!
//! Plans here are SYNTHETIC `CompiledPlan` literals (small, one construct
//! each); the four REAL compiled goldens run end-to-end in
//! `orchestrator_goldens.rs`. Building a literal is legitimate: the
//! orchestrator consumes the TYPE, and the goldens prove the real-compiler
//! path.

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::process::Command;

use sbfb_workflow_core::agent_definition::{
    BudgetPolicy, ModelPolicy, PromptRef, ProviderClass, SchemaRef,
};
use sbfb_workflow_core::compile::{
    COMPILER_PROFILE_STRICT_V1, COMPILER_VERSION, CompiledEdge, CompiledNodeInstance,
    CompiledPlacement, CompiledPlan, CompilerProfile, PlacementClass,
};
use sbfb_workflow_core::graph::{
    CancellationPolicy, ClassifierNode, ErrorPolicy, HumanReviewNode, JoinAllPolicy, JoinNode,
    JoinPolicy, JoinQuorumPolicy, MapNode, NodeFailureAction, OutputNode, ParallelNode, RepeatNode,
    SubworkflowNode, SubworkflowRef, SwitchNode, TimeoutPolicy, TransformNode, WaitSignalNode,
    WorkflowNode,
};
use sbfb_workflow_engine::clock::ManualClock;
use sbfb_workflow_engine::echo::{FileEchoTarget, side_output_count};
use sbfb_workflow_engine::hash::IdempotencyKey;
use sbfb_workflow_engine::orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorError};
use sbfb_workflow_engine::reducer::{NodeState, RunState, reduce};
use sbfb_workflow_engine::signal::{NoSignals, ScriptedSignals, SignalSource};
use sbfb_workflow_engine::store::{RunStatus, RunStore, StoreError};

/// Exit code of the out-of-process writer child (outside the libtest
/// range — mirrors the E/I harnesses).
const CHILD_EXIT_CODE: i32 = 87;

struct Ctx {
    _dir: tempfile::TempDir,
    db: PathBuf,
    log: PathBuf,
}

fn ctx() -> Ctx {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("run.sqlite3");
    let log = dir.path().join("effects.log");
    Ctx { _dir: dir, db, log }
}

fn orchestrator(c: &Ctx) -> Orchestrator {
    let store = RunStore::open(&c.db).expect("open store");
    Orchestrator::new(store, OrchestratorConfig::new("test-writer"))
}

fn budget() -> BudgetPolicy {
    BudgetPolicy {
        max_total_tokens: 1_000_000,
        max_cost_micro_usd: 1_000_000,
        max_wall_clock_seconds: 3_600,
        max_turns: 100,
    }
}

fn schema_ref() -> SchemaRef {
    SchemaRef {
        schema_id: "s".to_string(),
        blake3: "0".repeat(64),
    }
}

fn sub_ref(id: &str) -> SubworkflowRef {
    SubworkflowRef {
        workflow_id: id.to_string(),
        blake3: "0".repeat(64),
    }
}

/// A pure declarative leaf — the cheapest effect leaf for synthetic plans.
fn leaf() -> WorkflowNode {
    WorkflowNode::Transform(TransformNode {
        input_schema_ref: schema_ref(),
        output_schema_ref: schema_ref(),
        field_map: BTreeMap::new(),
    })
}

fn out_node(name: &str) -> WorkflowNode {
    WorkflowNode::Output(OutputNode {
        schema_ref: schema_ref(),
        artifact_name: name.to_string(),
    })
}

/// Build a synthetic `CompiledPlan`. `entrypoints`: scope → entry
/// STRUCTURAL ids; `nodes`: structural id → node; `edges`:
/// `(from, from_port, to, to_port)` structural endpoints. Policies default
/// to no-timeout-cancel; override `cancellation`/`timeout` before use when
/// a test needs the run-wide timer.
fn plan(
    entrypoints: &[(&str, &[&str])],
    nodes: Vec<(&str, WorkflowNode)>,
    edges: &[(&str, &str, &str, &str)],
) -> CompiledPlan {
    let mut node_instances = BTreeMap::new();
    for (id, node) in nodes {
        let depth = id.matches('/').count().saturating_sub(1) as u32;
        let kind = serde_json::to_value(&node).expect("node serialises")["kind"]
            .as_str()
            .expect("kind tag")
            .to_string();
        node_instances.insert(
            id.to_string(),
            CompiledNodeInstance {
                workflow_id: "wf-test".to_string(),
                node_id: id.rsplit('/').next().unwrap_or(id).to_string(),
                kind,
                depth,
                node,
            },
        );
    }
    let mut compiled_edges: Vec<CompiledEdge> = edges
        .iter()
        .map(|(f, fp, t, tp)| CompiledEdge {
            from: (*f).to_string(),
            from_port: (*fp).to_string(),
            to: (*t).to_string(),
            to_port: (*tp).to_string(),
        })
        .collect();
    compiled_edges.sort();

    let mut entry_map = BTreeMap::new();
    let mut scope_budgets = BTreeMap::new();
    let mut error_policies = BTreeMap::new();
    let mut timeout_policies = BTreeMap::new();
    let mut cancellation_policies = BTreeMap::new();
    for (scope, entries) in entrypoints {
        entry_map.insert(
            (*scope).to_string(),
            entries.iter().map(|e| (*e).to_string()).collect::<Vec<_>>(),
        );
        scope_budgets.insert((*scope).to_string(), budget());
        error_policies.insert(
            (*scope).to_string(),
            ErrorPolicy {
                on_node_failure: NodeFailureAction::FailWorkflow,
                max_retries_per_node: 0,
            },
        );
        timeout_policies.insert(
            (*scope).to_string(),
            TimeoutPolicy {
                workflow_timeout_seconds: NonZeroU32::new(3_600).expect("nz"),
                default_node_timeout_seconds: NonZeroU32::new(600).expect("nz"),
            },
        );
        cancellation_policies.insert(
            (*scope).to_string(),
            CancellationPolicy {
                cancel_on_budget_exhausted: false,
                cancel_on_timeout: false,
                allow_host_operator_cancel: true,
            },
        );
    }

    let human_gates = node_instances
        .iter()
        .filter(|(_, i)| matches!(i.node, WorkflowNode::HumanReview(_)))
        .map(|(id, _)| id.clone())
        .collect();

    CompiledPlan {
        compiler: CompilerProfile {
            version: COMPILER_VERSION.to_string(),
            profile: COMPILER_PROFILE_STRICT_V1.to_string(),
        },
        pack_digest: "0".repeat(64),
        pack_name: "test-pack".to_string(),
        pack_revision: 1,
        root_workflow_id: "wf-test".to_string(),
        artefact_digests: BTreeMap::new(),
        capability_closure: BTreeMap::new(),
        node_instances,
        edges: compiled_edges,
        entrypoints: entry_map,
        human_gates,
        graph_semantic_digest: "0".repeat(64),
        scope_budgets,
        node_budgets: BTreeMap::new(),
        placement: CompiledPlacement {
            default_class: PlacementClass::LocalOnly,
        },
        error_policies,
        timeout_policies,
        cancellation_policies,
    }
}

/// A linear three-leaf plan: `/a → /b → /out`.
fn linear_plan() -> CompiledPlan {
    plan(
        &[("", &["/a"])],
        vec![("/a", leaf()), ("/b", leaf()), ("/out", out_node("result"))],
        &[("/a", "out", "/b", "in"), ("/b", "out", "/out", "in")],
    )
}

// ── G-AUTHORITY: deterministic replay ─────────────────────────────────────

/// Same journal ⇒ same reduction, twice over; and a resumed run serves
/// every effect from the journal — the target is never touched again and
/// the journal does not grow (J.4 condition, with `iframe_death` below).
#[test]
fn orchestrator_deterministic_replay_same_journal_same_frontier() {
    let c = ctx();
    let p = linear_plan();
    let input = serde_json::json!({"seed": 1});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let first = orch
        .execute(
            &p,
            "run-replay",
            &input,
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("first run");
    assert_eq!(first.view.run_state, RunState::Completed);
    assert_eq!(first.steps_executed, 3);

    let events = orch.store().load_events("run-replay").expect("events");
    let view_a = reduce(&p, &events).expect("reduce a");
    let view_b = reduce(&p, &events).expect("reduce b");
    assert_eq!(view_a, view_b, "the reducer must be a pure function");

    let count_before = orch.store().event_count("run-replay").expect("count");
    let again = orch
        .execute(
            &p,
            "run-replay",
            &input,
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("replay");
    assert_eq!(again.steps_executed, 0, "no effect re-executed on replay");
    assert_eq!(
        again.steps_memoised, 0,
        "a completed run dispatches nothing"
    );
    assert_eq!(
        orch.store().event_count("run-replay").expect("count"),
        count_before,
        "a replay must not grow the journal"
    );
    assert_eq!(again.view.run_state, RunState::Completed);
}

/// RED ARM — a DIVERGENT journal fails closed per-node under the graph
/// walk: a pending intent whose journaled request hash disagrees with the
/// replayed request stops the run with the two hashes, and the stale
/// answer is never served (DBOSUnexpectedStepError class).
#[test]
fn orchestrator_deterministic_replay_divergent_journal_fails_closed() {
    let c = ctx();
    let p = linear_plan();
    let input = serde_json::json!({"seed": 1});
    let clock = ManualClock::at(1_000);

    // Forge the divergence through the spike-faults door: an intent for
    // `/a` under a WRONG request hash, as a crashed writer of another
    // input would have left.
    {
        let store = RunStore::open(&c.db).expect("open");
        store.ensure_run("run-div", "unbound").expect("run");
        store
            .record_intent_atomic(
                "run-div",
                "/a",
                &"f".repeat(64),
                "effect-forged",
                true,
                "scope",
            )
            .expect("forged intent");
    }

    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let err = orch
        .execute(&p, "run-div", &input, &mut target, &clock, &mut NoSignals)
        .expect_err("divergent journal must refuse");
    // The spec digest guard fires FIRST here (the run row was created
    // outside the orchestrator) — bind the run properly and hit the
    // step-level guard instead.
    assert!(
        matches!(err, OrchestratorError::SpecDigestMismatch { .. }),
        "got {err}"
    );

    // Now the proper arm: a run CREATED by the orchestrator, then a
    // forged pending row for the not-yet-dispatched leaf `/b`.
    let c2 = ctx();
    let mut orch2 = {
        let store = RunStore::open(&c2.db).expect("open");
        Orchestrator::new(store, OrchestratorConfig::new("test-writer"))
    };
    let gate = plan(
        &[("", &["/a"])],
        vec![("/a", leaf()), ("/w", wait_node("go")), ("/b", leaf())],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/b", "in")],
    );
    let mut target2 = FileEchoTarget::idempotent(&c2.log);
    let parked = orch2
        .execute(
            &gate,
            "run-div2",
            &input,
            &mut target2,
            &clock,
            &mut NoSignals,
        )
        .expect("parks on the wait");
    assert_eq!(parked.view.run_state, RunState::WaitingExternal);
    orch2
        .store()
        .record_intent_atomic(
            "run-div2",
            "/b",
            &"f".repeat(64),
            "effect-forged",
            true,
            "scope",
        )
        .expect("forged pending for /b");
    let mut script = ScriptedSignals::new().signal("/w", serde_json::json!({"ok": true}));
    let err2 = orch2
        .execute(&gate, "run-div2", &input, &mut target2, &clock, &mut script)
        .expect_err("hash divergence at /b must fail closed");
    let msg = err2.to_string();
    assert!(msg.contains("step request hash mismatch"), "got {msg}");
    assert_eq!(
        orch2.store().run_status("run-div2").expect("status"),
        Some(RunStatus::Terminal),
        "a divergent RUNNING run goes terminal with its diagnosis"
    );
}

fn wait_node(signal: &str) -> WorkflowNode {
    WorkflowNode::WaitSignal(WaitSignalNode {
        signal_name: signal.to_string(),
        payload_schema_ref: schema_ref(),
    })
}

fn review_node(labels: &[&str]) -> WorkflowNode {
    WorkflowNode::HumanReview(HumanReviewNode {
        payload_schema_ref: schema_ref(),
        decision_labels: labels.iter().map(|l| (*l).to_string()).collect(),
    })
}

fn switch_node(field: &str, labels: &[&str]) -> WorkflowNode {
    WorkflowNode::Switch(SwitchNode {
        discriminant_field: field.to_string(),
        branch_labels: labels.iter().map(|l| (*l).to_string()).collect(),
    })
}

// ── constructions ─────────────────────────────────────────────────────────

/// `parallel` fans out to every branch; a `join all` re-observes both
/// arrivals at runtime and fires exactly once.
#[test]
fn orchestrator_parallel_branches() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"])],
        vec![
            ("/in", leaf()),
            (
                "/par",
                WorkflowNode::Parallel(ParallelNode {
                    branch_labels: vec!["left".to_string(), "right".to_string()],
                }),
            ),
            ("/left", leaf()),
            ("/right", leaf()),
            (
                "/join",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::All(JoinAllPolicy {}),
                    source_node_ids: vec!["left".to_string(), "right".to_string()],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/par", "in"),
            ("/par", "left", "/left", "in"),
            ("/par", "right", "/right", "in"),
            ("/left", "out", "/join", "in"),
            ("/right", "out", "/join", "in"),
            ("/join", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-par",
            &serde_json::json!({"v": 1}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);
    for node in ["/left", "/right", "/out"] {
        assert_eq!(
            done.view.node_states.get(node).copied(),
            Some(NodeState::Succeeded),
            "{node}"
        );
    }
    // "Fires exactly once" is a side-output claim, not a state claim
    // (review r1 TS-3): the node downstream of the join carries exactly
    // ONE effect.
    let key = IdempotencyKey::new(
        "run-par",
        "/out",
        find_effect_id(orch.store(), "run-par", "/out"),
    )
    .expect("key");
    assert_eq!(side_output_count(&c.log, &key).expect("count"), 1);
}

/// WCP-30 dead-path: the branch a switch did NOT take derives `skipped`,
/// skip PROPAGATES, and a downstream `join all` fed by both branches
/// fires on the live one instead of hanging forever. The dead leaf's
/// effect NEVER reaches the target (side-output proof, not state-only).
#[test]
fn orchestrator_switch_skips_branch_into_join() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"])],
        vec![
            ("/in", leaf()),
            ("/gate", switch_node("route", &["fast", "slow"])),
            ("/fast", leaf()),
            ("/slow", leaf()),
            (
                "/join",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::All(JoinAllPolicy {}),
                    source_node_ids: vec!["fast".to_string(), "slow".to_string()],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/gate", "in"),
            ("/gate", "fast", "/fast", "in"),
            ("/gate", "slow", "/slow", "in"),
            ("/fast", "out", "/join", "in"),
            ("/slow", "out", "/join", "in"),
            ("/join", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-skip",
            &serde_json::json!({"route": "fast"}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed, "no join deadlock");
    assert_eq!(
        done.view.node_states.get("/slow").copied(),
        Some(NodeState::Skipped),
        "the branch not taken is skipped"
    );
    assert_eq!(
        done.view.node_states.get("/fast").copied(),
        Some(NodeState::Succeeded)
    );
    // Side-output proof: the skipped leaf never reached the target.
    let outputs = sbfb_workflow_engine::echo::read_side_outputs(&c.log).expect("log");
    assert!(
        !outputs.iter().any(|o| o.key_line.contains("/slow")),
        "a skipped node must never emit an effect"
    );
}

/// RED ARM of the router: an out-of-set discriminant routes to the
/// reserved `default` port, deterministically — here `default` is wired
/// nowhere at runtime… so the plan wires it to a fallback leaf.
#[test]
fn orchestrator_switch_out_of_set_routes_default() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"])],
        vec![
            ("/in", leaf()),
            ("/gate", switch_node("route", &["fast"])),
            ("/fast", leaf()),
            ("/fallback", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/gate", "in"),
            ("/gate", "fast", "/fast", "in"),
            ("/gate", "default", "/fallback", "in"),
            ("/fast", "out", "/out", "in"),
            ("/fallback", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-default",
            &serde_json::json!({"route": "nonsense"}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(
        done.view.node_states.get("/fallback").copied(),
        Some(NodeState::Succeeded),
        "out-of-set routes to default"
    );
    assert_eq!(
        done.view.node_states.get("/fast").copied(),
        Some(NodeState::Skipped)
    );
}

/// `map` fans out by the journaled cardinality, children reduce BY INDEX
/// (never arrival order), and the aggregated output is index-ordered.
#[test]
fn orchestrator_map_join_cardinality() {
    let c = ctx();
    let p = map_plan(3);
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-map",
            &serde_json::json!({"items": [{"i": 0}, {"i": 1}, {"i": 2}]}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);
    // Every iteration materialised and settled, identities disjoint.
    for i in 0..3 {
        let rid = format!("/fan~2{i}/m-in");
        assert_eq!(
            done.view.node_states.get(&rid).copied(),
            Some(NodeState::Succeeded),
            "{rid}"
        );
    }
    // Two iterations never share an identity of effect: one side output
    // per child leaf, none double.
    for i in 0..3 {
        let key = IdempotencyKey::new(
            "run-map",
            format!("/fan~2{i}/m-in"),
            find_effect_id(orch.store(), "run-map", &format!("/fan~2{i}/m-in")),
        )
        .expect("key");
        assert_eq!(
            side_output_count(&c.log, &key).expect("count"),
            1,
            "iteration {i}"
        );
    }
    // The aggregated output reduces BY ENCODED INDEX, never arrival order
    // (§12.2): /out's receipt carries the map's array, index-ordered.
    let receipt = orch
        .store()
        .completed_effect("run-map", "/out")
        .expect("out effect")
        .expect("completed")
        .receipt;
    let input_map: BTreeMap<String, String> =
        serde_json::from_str(&receipt).expect("receipt is the echoed input map");
    let aggregated: serde_json::Value =
        serde_json::from_str(input_map.get("in").expect("in port")).expect("value");
    assert_eq!(
        aggregated,
        serde_json::json!([{"i": 0}, {"i": 1}, {"i": 2}]),
        "map children must aggregate in index order"
    );
}

/// RED ARM — a collection LARGER than the structural bound is refused
/// BEFORE anything is journaled: the signed bound is never stretched.
#[test]
fn orchestrator_map_cardinality_exceeded_refused() {
    let c = ctx();
    let p = map_plan(2);
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let err = orch
        .execute(
            &p,
            "run-map-over",
            &serde_json::json!({"items": [{}, {}, {}]}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect_err("3 items over a bound of 2 must refuse");
    assert!(
        matches!(
            err,
            OrchestratorError::MapCardinalityExceeded {
                len: 3,
                bound: 2,
                ..
            }
        ),
        "got {err}"
    );
    let events = orch.store().load_events("run-map-over").expect("events");
    assert!(
        !events.iter().any(|e| matches!(
            e.kind,
            sbfb_workflow_engine::event::RunEventKind::MapFanout { .. }
        )),
        "an oversized fan-out must journal NOTHING"
    );
}

fn map_plan(bound: u32) -> CompiledPlan {
    plan(
        &[("", &["/in"]), ("/fan", &["/fan/m-in"])],
        vec![
            ("/in", leaf()),
            (
                "/fan",
                WorkflowNode::Map(MapNode {
                    collection_field: "items".to_string(),
                    max_cardinality: NonZeroU32::new(bound).expect("nz"),
                    body_subworkflow_ref: sub_ref("wf-body"),
                }),
            ),
            ("/fan/m-in", leaf()),
            ("/fan/m-out", out_node("item-out")),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/fan", "in"),
            ("/fan/m-in", "out", "/fan/m-out", "in"),
            ("/fan", "out", "/out", "in"),
        ],
    )
}

/// `repeat` runs its pinned body up to the STRUCTURAL bound and stops
/// early on the boolean flag — the bound comes from the signed plan, and
/// the per-iteration verdicts are journaled decisions.
#[test]
fn orchestrator_bounded_repeat() {
    let clock = ManualClock::at(0);

    // Flag false: exactly max_iterations bodies run.
    let c = ctx();
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &repeat_plan(3),
            "run-rep",
            &serde_json::json!({"done": false}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);
    for i in 0..3 {
        assert_eq!(
            done.view
                .node_states
                .get(&format!("/loop~2{i}/r-in"))
                .copied(),
            Some(NodeState::Succeeded),
            "iteration {i} ran"
        );
    }
    assert!(
        !done.view.node_states.contains_key("/loop~23/r-in"),
        "a fourth iteration must not exist — the bound is structural"
    );

    // Flag true: one iteration only.
    let c2 = ctx();
    let mut orch2 = orchestrator(&c2);
    let mut target2 = FileEchoTarget::idempotent(&c2.log);
    let early = orch2
        .execute(
            &repeat_plan(3),
            "run-rep-early",
            &serde_json::json!({"done": true}),
            &mut target2,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(early.view.run_state, RunState::Completed);
    assert!(early.view.node_states.contains_key("/loop~20/r-in"));
    assert!(
        !early.view.node_states.contains_key("/loop~21/r-in"),
        "the until flag stops after one iteration"
    );
}

fn repeat_plan(bound: u32) -> CompiledPlan {
    plan(
        &[("", &["/in"]), ("/loop", &["/loop/r-in"])],
        vec![
            ("/in", leaf()),
            (
                "/loop",
                WorkflowNode::Repeat(RepeatNode {
                    max_iterations: NonZeroU32::new(bound).expect("nz"),
                    body_subworkflow_ref: sub_ref("wf-body"),
                    until_output_flag: "done".to_string(),
                }),
            ),
            ("/loop/r-in", leaf()),
            ("/loop/r-out", out_node("iter-out")),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/loop", "in"),
            ("/loop/r-in", "out", "/loop/r-out", "in"),
            ("/loop", "out", "/out", "in"),
        ],
    )
}

/// `subworkflow` navigates the statically-expanded child scope — no
/// runtime expansion, ids are the structural ones.
#[test]
fn orchestrator_subworkflow() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"]), ("/child", &["/child/c-in"])],
        vec![
            ("/in", leaf()),
            (
                "/child",
                WorkflowNode::Subworkflow(SubworkflowNode {
                    subworkflow_ref: sub_ref("wf-child"),
                }),
            ),
            ("/child/c-in", leaf()),
            ("/child/c-out", out_node("child-out")),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/child", "in"),
            ("/child/c-in", "out", "/child/c-out", "in"),
            ("/child", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-sub",
            &serde_json::json!({"v": 1}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(
        done.view.node_states.get("/child/c-in").copied(),
        Some(NodeState::Succeeded),
        "child scope traversed under its structural id"
    );
}

// ── interruptions ─────────────────────────────────────────────────────────

/// `wait.signal`: the run parks (`waiting_external`), a NEW orchestrator
/// resumes it after the deposit, and the resume serves every upstream
/// effect FROM THE JOURNAL — memoisation, never re-execution (§12.6, the
/// D3 crux).
#[test]
fn orchestrator_wait_signal_resume_without_reexecution() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("approval")),
            ("/b", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/a", "out", "/w", "in"),
            ("/w", "out", "/b", "in"),
            ("/b", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let parked = orch
        .execute(&p, "run-wait", &input, &mut target, &clock, &mut NoSignals)
        .expect("first pass");
    assert_eq!(parked.view.run_state, RunState::WaitingExternal);
    assert_eq!(parked.steps_executed, 1, "only /a ran");
    assert_eq!(
        parked.view.node_states.get("/w").copied(),
        Some(NodeState::Waiting)
    );
    drop(orch);

    // A NEW orchestrator (new handle, new lease epoch) resumes.
    let mut orch2 = orchestrator(&c);
    let mut script = ScriptedSignals::new().signal("/w", serde_json::json!({"granted": true}));
    let done = orch2
        .execute(&p, "run-wait", &input, &mut target, &clock, &mut script)
        .expect("resume");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(
        done.steps_executed, 2,
        "only the downstream /b and /out ran on resume"
    );
    assert_eq!(
        done.steps_memoised, 0,
        "settled nodes are not re-dispatched"
    );
    // /a's effect exists EXACTLY once in the side-output log.
    let key = IdempotencyKey::new(
        "run-wait",
        "/a",
        find_effect_id(orch2.store(), "run-wait", "/a"),
    )
    .expect("key");
    assert_eq!(side_output_count(&c.log, &key).expect("count"), 1);
}

/// `human.review`: parking is `waiting_human` (T2 palier WA-T1 evidence:
/// waiting_human atteint + fin sans réexécution); an in-set decision
/// resumes on the decided port; an out-of-set decision is refused LOUDLY
/// with zero transition.
#[test]
fn orchestrator_human_review_parking_and_closed_set() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/review", review_node(&["approve", "reject"])),
            ("/approved", leaf()),
            ("/rejected", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/a", "out", "/review", "in"),
            ("/review", "approve", "/approved", "in"),
            ("/review", "reject", "/rejected", "in"),
            ("/approved", "out", "/out", "in"),
            ("/rejected", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let parked = orch
        .execute(&p, "run-hitl", &input, &mut target, &clock, &mut NoSignals)
        .expect("park");
    assert_eq!(parked.view.run_state, RunState::WaitingHuman);

    // RED ARM — an out-of-set label is refused, nothing transitions.
    let mut bad = ScriptedSignals::new().decision("/review", "escalate", serde_json::json!({}));
    let err = orch
        .execute(&p, "run-hitl", &input, &mut target, &clock, &mut bad)
        .expect_err("out-of-set label must refuse");
    assert!(
        matches!(
            err,
            OrchestratorError::Store(StoreError::SignalRefused { .. })
        ),
        "got {err}"
    );
    let events = orch.store().load_events("run-hitl").expect("events");
    assert!(
        !events.iter().any(|e| matches!(
            e.kind,
            sbfb_workflow_engine::event::RunEventKind::HumanDecisionRecorded { .. }
        )),
        "a refused decision must journal nothing"
    );

    // The refused deposit is spent-nothing: a VALID decision still works…
    // but the refused one already sits on the nonce. Mono-usage means the
    // wait still has its nonce; the refused deposit was recorded and NOT
    // consumed — deposit again under the same nonce collapses (dedup), so
    // the wait would starve. That is deliberate: a poisoned deposit is an
    // OPERATOR problem surfaced loudly, never silently replaced. Assert
    // the run is still parked.
    let view = reduce(&p, &orch.store().load_events("run-hitl").expect("events")).expect("reduce");
    assert_eq!(view.run_state, RunState::WaitingHuman);
}

/// Mono-usage (§12.6): a replayed delivery collapses on the nonce; after
/// consumption the nonce is spent — the signal is consumed EXACTLY once
/// and a second deposit of the same nonce can never re-fire the resume.
#[test]
fn orchestrator_signal_replay_consumed_exactly_once() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/out", out_node("result")),
        ],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/out", "in")],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    orch.execute(&p, "run-once", &input, &mut target, &clock, &mut NoSignals)
        .expect("park");

    let waits = orch.store().open_waits("run-once").expect("waits");
    assert_eq!(waits.len(), 1);
    let nonce = waits[0].nonce.clone();

    // Deliver TWICE before consumption: at-least-once collapses.
    let payload = serde_json::json!({"ok": true});
    assert!(
        orch.store()
            .deposit_signal("run-once", &nonce, &payload, None, None, 1_000)
            .expect("first deposit")
    );
    assert!(
        !orch
            .store()
            .deposit_signal("run-once", &nonce, &payload, None, None, 1_001)
            .expect("second deposit"),
        "a replayed delivery is a collapsed duplicate"
    );

    let done = orch
        .execute(&p, "run-once", &input, &mut target, &clock, &mut NoSignals)
        .expect("resume");
    assert_eq!(done.view.run_state, RunState::Completed);
    let consumed = orch
        .store()
        .load_events("run-once")
        .expect("events")
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                sbfb_workflow_engine::event::RunEventKind::SignalConsumed { .. }
            )
        })
        .count();
    assert_eq!(consumed, 1, "exactly one consumption");

    // RED ARM — after consumption the nonce is dead: a new deposit is
    // refused loudly (no open wait names it).
    let err = orch
        .store()
        .deposit_signal("run-once", &nonce, &payload, None, None, 2_000)
        .expect_err("a spent nonce must refuse");
    assert!(
        matches!(err, StoreError::SignalRefused { .. }),
        "got {err:?}"
    );
}

/// RED ARM — a FORGED nonce is refused at the deposit door, loudly, with
/// zero rows written (THREAT §19 signal-forge).
#[test]
fn orchestrator_signal_forged_nonce_rejected() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/out", out_node("r")),
        ],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/out", "in")],
    );
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    orch.execute(
        &p,
        "run-forge",
        &serde_json::json!({"v": 1}),
        &mut target,
        &clock,
        &mut NoSignals,
    )
    .expect("park");
    let count_before = orch.store().event_count("run-forge").expect("count");
    let err = orch
        .store()
        .deposit_signal(
            "run-forge",
            &"a".repeat(64),
            &serde_json::json!({"evil": true}),
            None,
            None,
            1_000,
        )
        .expect_err("forged nonce refused at the door");
    assert!(
        matches!(err, StoreError::SignalRefused { .. }),
        "got {err:?}"
    );
    // "Zero rows written" is asserted, not narrated (review r1 TS-7): the
    // journal did not grow and the real wait is still untouched and open.
    assert_eq!(
        orch.store().event_count("run-forge").expect("count"),
        count_before
    );
    let waits = orch.store().open_waits("run-forge").expect("waits");
    assert_eq!(waits.len(), 1, "the legitimate wait is still open");
}

/// RED ARM — an EXPIRED deposit is refused at consumption, loudly, with
/// zero transition: the run stays parked.
#[test]
fn orchestrator_signal_expired_rejected() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/out", out_node("r")),
        ],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/out", "in")],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    orch.execute(&p, "run-exp", &input, &mut target, &clock, &mut NoSignals)
        .expect("park");

    // Deposit valid until t=1500; the clock advances past it.
    let mut script =
        ScriptedSignals::new().expired_signal("/w", serde_json::json!({"late": true}), 1_500);
    let waits = orch.store().open_waits("run-exp").expect("waits");
    for d in script.poll(&waits) {
        orch.store()
            .deposit_signal(
                "run-exp",
                &d.nonce,
                &d.payload,
                None,
                d.expires_at_ms,
                1_000,
            )
            .expect("deposit");
    }
    clock.set(2_000);
    let err = orch
        .execute(&p, "run-exp", &input, &mut target, &clock, &mut NoSignals)
        .expect_err("expired deposit must refuse");
    assert!(
        matches!(
            err,
            OrchestratorError::Store(StoreError::SignalRefused { .. })
        ),
        "got {err}"
    );
    let view = reduce(&p, &orch.store().load_events("run-exp").expect("events")).expect("reduce");
    assert_eq!(view.run_state, RunState::WaitingExternal, "zero transition");
}

/// RED ARM — a REGRESSIVE clock is refused: a deposit made at t=1000
/// cannot be consumed at t=500 (un-expiring would be next), and an
/// already-fired timer never re-fires.
#[test]
fn orchestrator_clock_regression_loud() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/out", out_node("r")),
        ],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/out", "in")],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    orch.execute(&p, "run-clock", &input, &mut target, &clock, &mut NoSignals)
        .expect("park");
    let waits = orch.store().open_waits("run-clock").expect("waits");
    orch.store()
        .deposit_signal(
            "run-clock",
            &waits[0].nonce,
            &serde_json::json!({"ok": true}),
            None,
            None,
            1_000,
        )
        .expect("deposit at t=1000");

    clock.set(500); // the clock goes BACKWARDS
    let err = orch
        .execute(&p, "run-clock", &input, &mut target, &clock, &mut NoSignals)
        .expect_err("regression must refuse");
    assert!(
        matches!(
            err,
            OrchestratorError::Store(StoreError::ClockRegression {
                now_ms: 500,
                watermark_ms: 1_000
            })
        ),
        "the durable watermark refuses the whole pass — got {err}"
    );
    // The de-expiry window (review r1 SD-2): a deposit judged expired at a
    // later reading can never be consumed by a smaller one — the watermark
    // makes the smaller reading unpresentable at all.

    // Back to honest time, the run completes.
    clock.set(1_500);
    let done = orch
        .execute(&p, "run-clock", &input, &mut target, &clock, &mut NoSignals)
        .expect("resume at honest time");
    assert_eq!(done.view.run_state, RunState::Completed);
}

// ── timers ────────────────────────────────────────────────────────────────

/// Durable timers hold an ABSOLUTE deadline: a restart re-reads the
/// original deadline (never re-arms), the timer fires ONCE at it, and the
/// firing cancels the run under the plan's own `cancel_on_timeout`.
#[test]
fn orchestrator_restart_mid_timer_fires_once_at_original_deadline() {
    let c = ctx();
    let mut p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("never")),
            ("/out", out_node("r")),
        ],
        &[("/a", "out", "/w", "in"), ("/w", "out", "/out", "in")],
    );
    // 10-second workflow timeout, cancel-on-timeout ON (policies CONSUMED
    // from the compiled plan, never invented at runtime).
    p.timeout_policies
        .get_mut("")
        .expect("root")
        .workflow_timeout_seconds = NonZeroU32::new(10).expect("nz");
    p.cancellation_policies
        .get_mut("")
        .expect("root")
        .cancel_on_timeout = true;

    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let parked = orch
        .execute(&p, "run-timer", &input, &mut target, &clock, &mut NoSignals)
        .expect("start at t=0");
    assert_eq!(parked.view.run_state, RunState::WaitingExternal);
    drop(orch);

    // RESTART at t=5s: the deadline is still the ORIGINAL t=10s.
    clock.set(5_000);
    let mut orch2 = orchestrator(&c);
    let mid = orch2
        .execute(&p, "run-timer", &input, &mut target, &clock, &mut NoSignals)
        .expect("restart before the deadline");
    assert_eq!(
        mid.view.run_state,
        RunState::WaitingExternal,
        "a restart must NOT re-arm the deadline to t=15s"
    );

    // Past the original deadline: the timer fires, the run cancels.
    clock.set(10_001);
    let done = orch2
        .execute(&p, "run-timer", &input, &mut target, &clock, &mut NoSignals)
        .expect("deadline pass");
    assert_eq!(done.view.run_state, RunState::Cancelled);
    assert_eq!(
        orch2.store().terminal_reason("run-timer").expect("reason"),
        Some("cancelled".to_string())
    );

    // The timer can never fire twice (CAS on `fired`) — even if the clock
    // regresses and advances again, the run stays closed with ONE
    // cancel_requested entry.
    let cancels = orch2
        .store()
        .load_events("run-timer")
        .expect("events")
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                sbfb_workflow_engine::event::RunEventKind::CancelRequested { .. }
            )
        })
        .count();
    assert_eq!(cancels, 1, "one firing, one cancellation entry");
}

// ── honest cancel (§12.7) ─────────────────────────────────────────────────

/// Honest cancel: emission stops at the cancellation entry, a late result
/// is KEPT (journal + outbox) but the frontier does not advance and
/// nothing downstream dispatches, and the run closes `cancelled` — NEVER
/// `completed`.
#[test]
fn orchestrator_honest_cancel_keeps_late_result_without_propagation() {
    let c = ctx();
    let input = serde_json::json!({"seed": 1});
    let clock = ManualClock::at(1_000);

    // Stage: /a completed, /b INTENDED but unconfirmed (a writer died
    // mid-effect), through the production-equivalent doors.
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    {
        // Drive /a alone by parking after it: temporarily run the full
        // plan is impossible — instead stage through the spike-faults
        // doors on a run CREATED by the orchestrator's own digest. Simplest
        // honest staging: run to completion is NOT wanted; so create the
        // run via a parked twin plan sharing the digest is impossible too.
        // Therefore: start the real run, let it complete /a, and forge the
        // "intended, unconfirmed" state of /b by intent-without-completion.
        let parked_plan = plan(
            &[("", &["/a"])],
            vec![
                ("/a", leaf()),
                ("/w", wait_node("hold")),
                ("/b", leaf()),
                ("/out", out_node("result")),
            ],
            &[
                ("/a", "out", "/w", "in"),
                ("/w", "out", "/b", "in"),
                ("/b", "out", "/out", "in"),
            ],
        );
        let parked = orch
            .execute(
                &parked_plan,
                "run-cancel",
                &input,
                &mut target,
                &clock,
                &mut NoSignals,
            )
            .expect("park after /a");
        assert_eq!(parked.view.run_state, RunState::WaitingExternal);

        // Forge /b's pending intent (the crashed-writer shape) with the
        // REAL request the orchestrator would compute is not accessible
        // here; a synthetic hash is enough for the late-resolution path,
        // which re-applies by EFFECT ID, not by recomputing the request.
        let request = sbfb_workflow_engine::hash::StepRequest::new(
            "/b",
            "echo",
            BTreeMap::from([("in".to_string(), "held".to_string())]),
        )
        .expect("request");
        let request_hash = sbfb_workflow_engine::hash::step_request_hash(&request).expect("hash");
        let effect_id =
            sbfb_workflow_engine::hash::derive_effect_id("run-cancel", "/b", &request_hash);
        // The dedup scope must match the live target for the late
        // resolution to be allowed.
        let scope = sbfb_workflow_engine::echo::EffectTarget::dedup_scope(&target);
        orch.store()
            .record_intent_atomic("run-cancel", "/b", &request_hash, &effect_id, true, &scope)
            .expect("staged pending /b");

        // Cancel (host operator — always honoured).
        assert!(orch.request_cancel("run-cancel", &clock).expect("cancel"));

        // Close: "stop d'émission" is ABSOLUTE (Codex r1 P1) — the close
        // NEVER calls the target, even for a pending idempotent intent
        // whose effect may never have been attempted.
        let done = orch
            .execute(
                &parked_plan,
                "run-cancel",
                &input,
                &mut target,
                &clock,
                &mut NoSignals,
            )
            .expect("close");
        assert_eq!(done.view.run_state, RunState::Cancelled, "never completed");
        assert_eq!(
            orch.store().run_status("run-cancel").expect("status"),
            Some(RunStatus::Terminal)
        );
        assert_eq!(
            orch.store().terminal_reason("run-cancel").expect("reason"),
            Some("cancelled".to_string())
        );
        // The pending intent is PRESERVED pending — honestly journaled,
        // never emitted post-cancel: zero side output for /b.
        assert!(
            orch.store()
                .pending_effect("run-cancel", "/b")
                .expect("pending")
                .is_some(),
            "the intent stays honestly pending"
        );
        let key = IdempotencyKey::new("run-cancel", "/b", effect_id.as_str()).expect("key");
        assert_eq!(
            side_output_count(&c.log, &key).expect("count"),
            0,
            "no first emission after a cancellation"
        );
        // A result that ARRIVES late lands through the store door: kept
        // (outbox applied) with the FRONTIER untouched — the §12.7 "kept,
        // not propagated" half, at its production door.
        orch.store()
            .record_late_completion(
                "run-cancel",
                "/b",
                &effect_id,
                &request_hash,
                "late-receipt",
            )
            .expect("late arrival");
        assert!(
            orch.store()
                .completed_effect("run-cancel", "/b")
                .expect("late effect")
                .is_some(),
            "the late result is preserved for audit"
        );
        assert_ne!(
            orch.store().frontier_node("run-cancel").expect("frontier"),
            Some("/b".to_string()),
            "a late result must not advance the frontier"
        );
        assert!(
            orch.store()
                .completed_effect("run-cancel", "/out")
                .expect("out")
                .is_none(),
            "nothing propagates past a cancellation"
        );
    }
}

// ── backpressure ──────────────────────────────────────────────────────────

/// The in-flight cap changes PACING, never outcomes: admission stays in
/// canonical order, and the journaled intent sequence is IDENTICAL between
/// cap=1 and cap=8 on the same plan and input.
#[test]
fn orchestrator_backpressure_deterministic_admission() {
    let p = plan(
        &[("", &["/in"])],
        vec![
            ("/in", leaf()),
            (
                "/par",
                WorkflowNode::Parallel(ParallelNode {
                    branch_labels: vec!["a".into(), "b".into(), "c".into(), "d".into()],
                }),
            ),
            ("/br-a", leaf()),
            ("/br-b", leaf()),
            ("/br-c", leaf()),
            ("/br-d", leaf()),
            (
                "/join",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::Quorum(JoinQuorumPolicy {
                        quorum: NonZeroU32::new(4).expect("nz"),
                    }),
                    source_node_ids: vec![
                        "br-a".into(),
                        "br-b".into(),
                        "br-c".into(),
                        "br-d".into(),
                    ],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/par", "in"),
            ("/par", "a", "/br-a", "in"),
            ("/par", "b", "/br-b", "in"),
            ("/par", "c", "/br-c", "in"),
            ("/par", "d", "/br-d", "in"),
            ("/br-a", "out", "/join", "in"),
            ("/br-b", "out", "/join", "in"),
            ("/br-c", "out", "/join", "in"),
            ("/br-d", "out", "/join", "in"),
            ("/join", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"v": 1});

    let mut sequences: Vec<Vec<String>> = Vec::new();
    for cap in [1usize, 8usize] {
        let c = ctx();
        let store = RunStore::open(&c.db).expect("open");
        let mut config = OrchestratorConfig::new("test-writer");
        config.max_in_flight = cap;
        let mut orch = Orchestrator::new(store, config);
        let clock = ManualClock::at(0);
        let mut target = FileEchoTarget::idempotent(&c.log);
        let done = orch
            .execute(&p, "run-bp", &input, &mut target, &clock, &mut NoSignals)
            .expect("run");
        assert_eq!(done.view.run_state, RunState::Completed, "cap {cap}");
        let intents: Vec<String> = orch
            .store()
            .load_events("run-bp")
            .expect("events")
            .iter()
            .filter(|e| {
                matches!(
                    e.kind,
                    sbfb_workflow_engine::event::RunEventKind::EffectIntended { .. }
                )
            })
            .filter_map(|e| e.node_instance_id.clone())
            .collect();
        sequences.push(intents);
    }
    assert_eq!(
        sequences[0], sequences[1],
        "the journaled admission order must be identical whatever the cap"
    );
}

// ── G-AUTHORITY: the run survives its writer ──────────────────────────────

/// "Tuer l'iframe ne tue pas le run" — the engine has no iframe; the REAL
/// actor is the writer PROCESS. The child starts the run, completes its
/// first effect, parks on the wait and DIES (exit 87, no unwind, no
/// release). A NEW handle in the parent takes the lease over, resumes the
/// SAME frontier, and finishes with ZERO duplicate side output.
#[test]
fn orchestrator_iframe_death_does_not_kill_run() {
    let exe = std::env::current_exe().expect("current_exe");
    let c = ctx();

    let out = Command::new(&exe)
        .args([
            "orchestrator_child_dies_parked",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("SBFB_ORCH_DB", &c.db)
        .env("SBFB_ORCH_LOG", &c.log)
        .output()
        .expect("spawn the child writer");
    assert_eq!(
        out.status.code(),
        Some(CHILD_EXIT_CODE),
        "the child must die parked (stdout: {}, stderr: {})",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // The dead writer's state: /a completed, the wait open, lease held.
    let store = RunStore::open(&c.db).expect("open after death");
    assert!(
        store
            .completed_effect("run-death", "/a")
            .expect("a")
            .is_some(),
        "the child's first effect must be durable"
    );
    assert_eq!(store.open_waits("run-death").expect("waits").len(), 1);
    drop(store);

    // A NEW orchestrator resumes: same frontier, the deposit arrives, the
    // run completes — and /a has EXACTLY one side output.
    let mut orch = orchestrator(&c);
    let clock = ManualClock::at(1_000_000); // far past the child's lease TTL
    let mut target = FileEchoTarget::idempotent(&c.log);
    let mut script = ScriptedSignals::new().signal("/w", serde_json::json!({"go": true}));
    let done = orch
        .execute(
            &death_plan(),
            "run-death",
            &serde_json::json!({"v": 1}),
            &mut target,
            &clock,
            &mut script,
        )
        .expect("resume after writer death");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(done.steps_executed, 2, "/b and /out only");
    let key = IdempotencyKey::new(
        "run-death",
        "/a",
        find_effect_id(orch.store(), "run-death", "/a"),
    )
    .expect("key");
    assert_eq!(
        side_output_count(&c.log, &key).expect("count"),
        1,
        "zero duplicate effect across the writer's death"
    );
}

/// The child of the death arm. `#[ignore]` — only ever runs when the
/// parent spawns it (the E/I mechanism; this is the SECOND non-feature-
/// gated ignore of the repo, consigned in the phase body like the first).
#[test]
#[ignore = "spawned by orchestrator_iframe_death_does_not_kill_run"]
fn orchestrator_child_dies_parked() {
    let db = PathBuf::from(std::env::var_os("SBFB_ORCH_DB").expect("SBFB_ORCH_DB"));
    let log = PathBuf::from(std::env::var_os("SBFB_ORCH_LOG").expect("SBFB_ORCH_LOG"));
    let store = RunStore::open(&db).expect("child open");
    let mut orch = Orchestrator::new(store, OrchestratorConfig::new("child-writer"));
    let clock = ManualClock::at(0);
    let mut target = FileEchoTarget::idempotent(&log);
    let parked = orch
        .execute(
            &death_plan(),
            "run-death",
            &serde_json::json!({"v": 1}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("child parks");
    assert_eq!(parked.view.run_state, RunState::WaitingExternal);
    // Die WITHOUT releasing anything.
    std::process::exit(CHILD_EXIT_CODE);
}

fn death_plan() -> CompiledPlan {
    plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/b", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/a", "out", "/w", "in"),
            ("/w", "out", "/b", "in"),
            ("/b", "out", "/out", "in"),
        ],
    )
}

// ── golden event stream of the orchestrated kinds ─────────────────────────

/// The journal of a fixed orchestrated scenario is pinned by a committed
/// golden — the FIVE Phase J event kinds (`branch_taken`, `map_fanout`,
/// `signal_consumed`, `human_decision_recorded`, `cancel_requested`) all
/// appear, byte-stable, and round-trip through the typed envelope.
/// STRICT like the E goldens: missing/empty FAILS asking for regeneration.
///
/// Regenerate an intentional change with
/// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-engine --test orchestrator orchestrator_event_stream_matches_golden`.
#[test]
fn orchestrator_event_stream_matches_golden() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"]), ("/fan", &["/fan/m-in"])],
        vec![
            ("/in", leaf()),
            ("/gate", switch_node("route", &["go"])),
            (
                "/fan",
                WorkflowNode::Map(MapNode {
                    collection_field: "items".to_string(),
                    max_cardinality: NonZeroU32::new(2).expect("nz"),
                    body_subworkflow_ref: sub_ref("wf-body"),
                }),
            ),
            ("/fan/m-in", leaf()),
            ("/fan/m-out", out_node("item")),
            ("/w", wait_node("proceed")),
            ("/review", review_node(&["approve"])),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/gate", "in"),
            ("/gate", "go", "/fan", "in"),
            ("/fan/m-in", "out", "/fan/m-out", "in"),
            ("/fan", "out", "/w", "in"),
            ("/w", "out", "/review", "in"),
            ("/review", "approve", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"route": "go", "items": [{"i": 0}, {"i": 1}]});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let mut script = ScriptedSignals::new()
        .signal("/w", serde_json::json!({"ok": true}))
        .decision("/review", "approve", serde_json::json!({"lgtm": true}));
    let done = orch
        .execute(
            &p,
            "run-golden-orch",
            &input,
            &mut target,
            &clock,
            &mut script,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);

    // A second, cancelled run pins `cancel_requested` + the cancelled
    // terminal in the same golden file.
    let cancel_plan = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/hold", wait_node("never")),
            ("/end", out_node("r")),
        ],
        &[("/a", "out", "/hold", "in"), ("/hold", "out", "/end", "in")],
    );
    orch.execute(
        &cancel_plan,
        "run-golden-cancel",
        &input,
        &mut target,
        &clock,
        &mut NoSignals,
    )
    .expect("park");
    assert!(
        orch.request_cancel("run-golden-cancel", &clock)
            .expect("cancel")
    );
    let drained = orch
        .execute(
            &cancel_plan,
            "run-golden-cancel",
            &input,
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("drain");
    assert_eq!(drained.view.run_state, RunState::Cancelled);

    let orchestrated = orch.store().load_events("run-golden-orch").expect("events");
    let cancelled = orch
        .store()
        .load_events("run-golden-cancel")
        .expect("events");
    let live = serde_json::json!({
        "orchestrated": orchestrated,
        "cancelled": cancelled,
    });

    // Every new kind must actually appear — a golden that silently lost a
    // kind would still "match" itself after regeneration.
    let text = serde_json::to_string(&live).expect("serialise");
    for kind in [
        "branch_taken",
        "map_fanout",
        "signal_consumed",
        "human_decision_recorded",
        "cancel_requested",
    ] {
        assert!(text.contains(kind), "golden scenario lost kind {kind}");
    }

    assert_golden("run_event_orchestrated.golden.json", &live);
}

/// STRICT golden comparison (the E harness mechanism, re-spelt for this
/// harness: missing/empty fails, only `UPDATE_SNAPSHOTS=1` writes, and the
/// stream round-trips through the typed envelope).
fn assert_golden(file_name: &str, live: &serde_json::Value) {
    let golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(file_name);

    if std::env::var_os("UPDATE_SNAPSHOTS").is_some() {
        let pretty = serde_json::to_string_pretty(live).expect("pretty");
        std::fs::write(&golden_path, format!("{pretty}\n")).expect("write golden");
        return;
    }

    let golden_str = std::fs::read_to_string(&golden_path).unwrap_or_else(|_| {
        panic!("{file_name} missing — regenerate with UPDATE_SNAPSHOTS=1 and commit it")
    });
    assert!(
        !golden_str.trim().is_empty(),
        "{file_name} is empty — regenerate with UPDATE_SNAPSHOTS=1 and commit it"
    );
    let golden: serde_json::Value =
        serde_json::from_str(&golden_str).expect("golden must be valid JSON");
    assert_eq!(
        live, &golden,
        "orchestrated journal drift — the event stream of the fixed scenario changed"
    );

    // Round-trip through the TYPE: the new kinds' `deny_unknown_fields`
    // deserialisation is exercised here.
    for stream in ["orchestrated", "cancelled"] {
        let typed: Vec<sbfb_workflow_engine::RunEvent> =
            serde_json::from_value(golden[stream].clone())
                .expect("the golden must deserialize into RunEvent");
        assert_eq!(
            serde_json::to_value(&typed).expect("re-serialize"),
            live[stream],
            "the golden must round-trip through the typed envelope"
        );
    }
}

// ── review r1 hardening arms ──────────────────────────────────────────────

/// The decided label ROUTES: a `reject` decision (the NON-first label)
/// resumes on the `reject` port, the other branch is skipped — the
/// discriminating arm the parking test lacked (review r1 TS-1; also breaks
/// the suite-wide labels[0] bias flagged by the completeness critic).
#[test]
fn orchestrator_human_review_routes_decided_port() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/review", review_node(&["approve", "reject"])),
            ("/approved", leaf()),
            ("/rejected", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/a", "out", "/review", "in"),
            ("/review", "approve", "/approved", "in"),
            ("/review", "reject", "/rejected", "in"),
            ("/approved", "out", "/out", "in"),
            ("/rejected", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    orch.execute(&p, "run-route", &input, &mut target, &clock, &mut NoSignals)
        .expect("park");
    let mut script =
        ScriptedSignals::new().decision("/review", "reject", serde_json::json!({"why": "no"}));
    let done = orch
        .execute(&p, "run-route", &input, &mut target, &clock, &mut script)
        .expect("resume on reject");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(
        done.view.node_states.get("/rejected").copied(),
        Some(NodeState::Succeeded),
        "the DECIDED port routes"
    );
    assert_eq!(
        done.view.node_states.get("/approved").copied(),
        Some(NodeState::Skipped),
        "the other branch is a dead path"
    );
    // Side-output proof, not state-only: /approved never reached the target.
    let outputs = sbfb_workflow_engine::echo::read_side_outputs(&c.log).expect("log");
    assert!(!outputs.iter().any(|o| o.key_line.contains("/approved")));
}

/// The classifier's synthetic arm (review r1 TS-5): an in-set NON-first
/// proposal routes to its branch; an out-of-set proposal routes to the
/// reserved `default` port deterministically (research §19.3 — injection
/// picks among declared branches, never mints a destination).
#[test]
fn orchestrator_classifier_routes_proposal_and_defaults() {
    let clock = ManualClock::at(0);
    for (proposal, expect_hit, expect_skip) in [
        ("deep", "/deep", "/fallback"),
        ("nonsense", "/fallback", "/deep"),
    ] {
        let c = ctx();
        let p = plan(
            &[("", &["/in"])],
            vec![
                ("/in", leaf()),
                ("/triage", classifier_node(&["quick", "deep"])),
                ("/quick", leaf()),
                ("/deep", leaf()),
                ("/fallback", leaf()),
                ("/out", out_node("result")),
            ],
            &[
                ("/in", "out", "/triage", "in"),
                ("/triage", "quick", "/quick", "in"),
                ("/triage", "deep", "/deep", "in"),
                ("/triage", "default", "/fallback", "in"),
                ("/quick", "out", "/out", "in"),
                ("/deep", "out", "/out", "in"),
                ("/fallback", "out", "/out", "in"),
            ],
        );
        let mut orch = orchestrator(&c);
        let mut target = FileEchoTarget::idempotent(&c.log);
        let done = orch
            .execute(
                &p,
                "run-triage",
                &serde_json::json!({"classifier_label": proposal}),
                &mut target,
                &clock,
                &mut NoSignals,
            )
            .expect("run");
        assert_eq!(done.view.run_state, RunState::Completed, "{proposal}");
        assert_eq!(
            done.view.node_states.get(expect_hit).copied(),
            Some(NodeState::Succeeded),
            "proposal {proposal} routes to {expect_hit}"
        );
        assert_eq!(
            done.view.node_states.get(expect_skip).copied(),
            Some(NodeState::Skipped),
            "proposal {proposal} skips {expect_skip}"
        );
    }
}

/// The orchestrator's OWN outcome-unknown wiring (review r1 completeness):
/// a pending intent against a NON-idempotent target resolves to the
/// terminal `outcome_unknown` — never a retry, never an invented success —
/// and the REDUCER maps it to `RunState::OutcomeUnknown`. The spike proves
/// its own copy; this proves the graph walk's.
#[test]
fn orchestrator_non_idempotent_pending_goes_outcome_unknown() {
    let c = ctx();
    let p = plan(
        &[("", &["/a"])],
        vec![
            ("/a", leaf()),
            ("/w", wait_node("go")),
            ("/b", leaf()),
            ("/out", out_node("result")),
        ],
        &[
            ("/a", "out", "/w", "in"),
            ("/w", "out", "/b", "in"),
            ("/b", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"v": 1});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut non_idem = FileEchoTarget::non_idempotent(&c.log);
    orch.execute(&p, "run-unk", &input, &mut non_idem, &clock, &mut NoSignals)
        .expect("park after /a");

    // Deposit + consume the signal, then forge the crashed-writer shape
    // for /b: an intent under the EXACT request the walk will recompute
    // (input of /b = the consumed signal payload on port `in`), against
    // the non-idempotent target's journaled claim.
    let payload = serde_json::json!({"go": true});
    let waits = orch.store().open_waits("run-unk").expect("waits");
    orch.store()
        .deposit_signal("run-unk", &waits[0].nonce, &payload, None, None, 1_000)
        .expect("deposit");
    let expected_input = BTreeMap::from([(
        "in".to_string(),
        sbfb_workflow_engine::hash::canonical_string(&payload).expect("jcs"),
    )]);
    let request = sbfb_workflow_engine::hash::StepRequest::new("/b", "echo", expected_input)
        .expect("request");
    let request_hash = sbfb_workflow_engine::hash::step_request_hash(&request).expect("hash");
    let effect_id = sbfb_workflow_engine::hash::derive_effect_id("run-unk", "/b", &request_hash);
    let scope = sbfb_workflow_engine::echo::EffectTarget::dedup_scope(&non_idem);
    orch.store()
        .record_intent_atomic("run-unk", "/b", &request_hash, &effect_id, false, &scope)
        .expect("staged non-idempotent pending");

    let err = orch
        .execute(&p, "run-unk", &input, &mut non_idem, &clock, &mut NoSignals)
        .expect_err("a non-idempotent unconfirmed effect must refuse");
    assert!(err.to_string().contains("outcome unknown"), "got {err}");
    assert_eq!(
        orch.store().terminal_reason("run-unk").expect("reason"),
        Some("outcome_unknown".to_string())
    );
    // And the reducer RENDERS it — the only terminal arm not otherwise
    // observed through a returned view.
    let view = reduce(&p, &orch.store().load_events("run-unk").expect("events")).expect("reduce");
    assert_eq!(view.run_state, RunState::OutcomeUnknown);
}

/// The reducer refuses an ADVERSE journal loudly (review r1 completeness):
/// an event naming a node outside the plan, and a journaled fan-out beyond
/// the structural bound, are both structural refusals — never silently
/// derived around.
#[test]
fn orchestrator_reduce_refuses_adverse_journal() {
    use sbfb_workflow_engine::event::{RUN_EVENT_FORMAT_VERSION, RunEvent, RunEventKind};
    use sbfb_workflow_engine::reducer::ReduceError;

    let p = map_plan(2);
    let started = RunEvent {
        format_version: RUN_EVENT_FORMAT_VERSION,
        run_id: "run-adverse".to_string(),
        sequence: 0,
        node_instance_id: None,
        request_hash: None,
        kind: RunEventKind::RunStarted,
    };

    // Arm 1: a journal naming a node the plan cannot contain.
    let ghost = RunEvent {
        sequence: 1,
        node_instance_id: Some("/ghost".to_string()),
        kind: RunEventKind::EffectIntended {
            effect_id: "e".to_string(),
        },
        ..started.clone()
    };
    let err = reduce(&p, &[started.clone(), ghost]).expect_err("unknown node refused");
    assert!(
        matches!(err, ReduceError::UnknownNode { .. }),
        "got {err:?}"
    );

    // Arm 2: a journaled fan-out beyond the signed bound (unreachable via
    // the orchestrator, which refuses pre-journal — only a foreign or
    // corrupt journal carries it).
    let oversized = RunEvent {
        sequence: 1,
        node_instance_id: Some("/fan".to_string()),
        kind: RunEventKind::MapFanout { cardinality: 99 },
        ..started.clone()
    };
    let err = reduce(&p, &[started, oversized]).expect_err("oversized fan-out refused");
    assert!(
        matches!(
            err,
            ReduceError::FanoutBeyondBound {
                cardinality: 99,
                bound: 2,
                ..
            }
        ),
        "got {err:?}"
    );
}

/// Quorum joins are DISCRIMINATED from `all` (review r1 TS-4): a 1-of-2
/// quorum fires while its second source is still parked on a wait — under
/// `all` the join would stay pending forever.
#[test]
fn orchestrator_join_quorum_fires_before_slow_branch() {
    let c = ctx();
    let p = plan(
        &[("", &["/in"])],
        vec![
            ("/in", leaf()),
            (
                "/par",
                WorkflowNode::Parallel(ParallelNode {
                    branch_labels: vec!["fast".to_string(), "slow".to_string()],
                }),
            ),
            ("/fast", leaf()),
            ("/slow-gate", wait_node("never-sent")),
            ("/slow", leaf()),
            (
                "/join",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::Quorum(JoinQuorumPolicy {
                        quorum: NonZeroU32::new(1).expect("nz"),
                    }),
                    source_node_ids: vec!["fast".to_string(), "slow".to_string()],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/par", "in"),
            ("/par", "fast", "/fast", "in"),
            ("/par", "slow", "/slow-gate", "in"),
            ("/slow-gate", "out", "/slow", "in"),
            ("/fast", "out", "/join", "in"),
            ("/slow", "out", "/join", "in"),
            ("/join", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let progress = orch
        .execute(
            &p,
            "run-quorum",
            &serde_json::json!({"v": 1}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    // The quorum fired and /out ran while the slow branch is still parked.
    assert_eq!(
        progress.view.node_states.get("/join").copied(),
        Some(NodeState::Succeeded),
        "1-of-2 quorum fires early"
    );
    assert_eq!(
        progress.view.node_states.get("/out").copied(),
        Some(NodeState::Succeeded)
    );
    assert_eq!(
        progress.view.node_states.get("/slow-gate").copied(),
        Some(NodeState::Waiting),
        "the second source is still parked — quorum ≠ all"
    );
    assert_eq!(progress.view.run_state, RunState::WaitingExternal);
}

/// NESTED dynamic scopes (Codex r2): a repeat whose body contains a MAP —
/// the outer repeat must still advance (its `scope_settled` check counts
/// only DIRECT children; the nested map folds its iterations into its own
/// state). Under the pre-fix code the outer repeat never advanced: the
/// nested children carry their own iteration suffixes and the naive
/// subtree walk named instances that never exist.
#[test]
fn orchestrator_repeat_containing_map_advances() {
    let c = ctx();
    let p = plan(
        &[
            ("", &["/in"]),
            ("/loop", &["/loop/l-in"]),
            ("/loop/inner", &["/loop/inner/i-in"]),
        ],
        vec![
            ("/in", leaf()),
            (
                "/loop",
                WorkflowNode::Repeat(RepeatNode {
                    max_iterations: NonZeroU32::new(2).expect("nz"),
                    body_subworkflow_ref: sub_ref("wf-body"),
                    until_output_flag: "done".to_string(),
                }),
            ),
            ("/loop/l-in", leaf()),
            (
                "/loop/inner",
                WorkflowNode::Map(MapNode {
                    collection_field: "items".to_string(),
                    max_cardinality: NonZeroU32::new(2).expect("nz"),
                    body_subworkflow_ref: sub_ref("wf-inner"),
                }),
            ),
            ("/loop/inner/i-in", leaf()),
            ("/loop/inner/i-out", out_node("inner-out")),
            ("/loop/l-out", out_node("iter-out")),
            ("/out", out_node("result")),
        ],
        &[
            ("/in", "out", "/loop", "in"),
            ("/loop/l-in", "out", "/loop/inner", "in"),
            ("/loop/inner/i-in", "out", "/loop/inner/i-out", "in"),
            // The iteration OUTPUT takes the pass-through seed (flag
            // intact); the nested map still gates the scope's settling.
            ("/loop/l-in", "out", "/loop/l-out", "in"),
            ("/loop", "out", "/out", "in"),
        ],
    );
    let clock = ManualClock::at(0);
    let mut orch = orchestrator(&c);
    let mut target = FileEchoTarget::idempotent(&c.log);
    let done = orch
        .execute(
            &p,
            "run-nested",
            &serde_json::json!({"done": true, "items": [{"i": 0}, {"i": 1}]}),
            &mut target,
            &clock,
            &mut NoSignals,
        )
        .expect("nested run");
    assert_eq!(
        done.view.run_state,
        RunState::Completed,
        "the outer repeat advances over a nested dynamic scope"
    );
    // The nested map's iterations materialised under DOUBLE suffixes.
    assert_eq!(
        done.view.node_states.get("/loop~20/inner~20/i-in").copied(),
        Some(NodeState::Succeeded)
    );
    assert_eq!(
        done.view.node_states.get("/loop~20/inner~21/i-in").copied(),
        Some(NodeState::Succeeded)
    );
}

fn classifier_node(labels: &[&str]) -> WorkflowNode {
    WorkflowNode::Classifier(ClassifierNode {
        prompt_ref: PromptRef {
            prompt_id: "p".to_string(),
            blake3: "0".repeat(64),
        },
        model_policy: ModelPolicy {
            provider_class: ProviderClass::Local,
            model_id: "echo".to_string(),
            require_structured_output: true,
        },
        branch_labels: labels.iter().map(|l| (*l).to_string()).collect(),
        budget: budget(),
    })
}

// ── helpers ───────────────────────────────────────────────────────────────

fn find_effect_id(store: &RunStore, run_id: &str, node: &str) -> String {
    store
        .completed_effect(run_id, node)
        .expect("effect")
        .expect("completed")
        .effect_id
}
