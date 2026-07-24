// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 83 Phase K — the effect layer harness: retries, timeouts,
//! `outcome_unknown`, reconciliation, fencing target-side, FS/process
//! effects with descendant cleanup.
//!
//! Same doctrine as the E/I/J harnesses: every guarantee has a guaranteed
//! arm AND a discriminating red arm, and each named test says WHICH
//! production mutation it kills — a green suite must never be vacuous.
//!
//! Gate naming: `double_command_single_transition` (G-EFFECT),
//! `non_idempotent_without_receipt_becomes_unknown` (G-UNKNOWN),
//! `fs_effect_bound_to_head_index_worktree` +
//! `descendant_process_cleanup_{windows,unix}` (G-FS-PROCESS-EFFECT).

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::PathBuf;

use sbfb_workflow_core::agent_definition::{BudgetPolicy, SchemaRef};
use sbfb_workflow_core::compile::{
    COMPILER_PROFILE_STRICT_V1, COMPILER_VERSION, CompiledEdge, CompiledNodeInstance,
    CompiledPlacement, CompiledPlan, CompilerProfile, MAX_DECISION_LABEL_BYTES, PlacementClass,
};
use sbfb_workflow_core::graph::{
    CancellationPolicy, ErrorPolicy, JoinAnyPolicy, JoinNode, JoinPolicy, NodeFailureAction,
    OutputNode, ParallelNode, TimeoutPolicy, TransformNode, WorkflowNode,
};
use sbfb_workflow_engine::activity::{
    ACTIVITY_RECEIPT_KIND, Activity, ActivityError, ActivityReceipt, EffectContext,
    FencedFileActivity, PollOutcome, ScriptedActivity, fenced_side_output_count,
    read_fenced_side_outputs,
};
use sbfb_workflow_engine::clock::ManualClock;
use sbfb_workflow_engine::engine::EngineError;
use sbfb_workflow_engine::event::{ReconcileResolution, RunEventKind};
use sbfb_workflow_engine::hash::{
    IdempotencyKey, StepRequest, canonical_string, step_request_hash,
};
use sbfb_workflow_engine::orchestrator::{Orchestrator, OrchestratorConfig, OrchestratorError};
use sbfb_workflow_engine::reducer::{NodeState, ReduceError, RunState, reduce};
use sbfb_workflow_engine::signal::NoSignals;
use sbfb_workflow_engine::store::{
    MAX_SIGNAL_PAYLOAD_BYTES, RetentionPolicy, RunStatus, RunStore, StoreError,
};

struct Ctx {
    dir: tempfile::TempDir,
    db: PathBuf,
    log: PathBuf,
}

fn ctx() -> Ctx {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("run.sqlite3");
    let log = dir.path().join("effects.log");
    Ctx { dir, db, log }
}

fn orchestrator(c: &Ctx) -> Orchestrator {
    let store = RunStore::open(&c.db).expect("open store");
    Orchestrator::new(store, OrchestratorConfig::new("test-writer"))
}

fn parking_orchestrator(c: &Ctx) -> Orchestrator {
    let store = RunStore::open(&c.db).expect("open store");
    let mut config = OrchestratorConfig::new("test-writer");
    config.park_unknown_for_reconciliation = true;
    Orchestrator::new(store, config)
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

/// Synthetic `CompiledPlan` (the J-harness builder, unchanged policies:
/// 600s node timeout, 0 retries, fail-workflow — tests override).
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

fn single_leaf_plan() -> CompiledPlan {
    plan(&[("", &["/a"])], vec![("/a", leaf())], &[])
}

fn two_leaf_plan() -> CompiledPlan {
    plan(
        &[("", &["/a"])],
        vec![("/a", leaf()), ("/out", out_node("result"))],
        &[("/a", "out", "/out", "in")],
    )
}

fn ready(value: serde_json::Value) -> PollOutcome {
    PollOutcome::Ready {
        output: value,
        facts: BTreeMap::new(),
    }
}

fn failed(retryable: bool) -> PollOutcome {
    PollOutcome::Failed {
        retryable,
        reason: "scripted failure".to_string(),
    }
}

fn count_kind(store: &RunStore, run_id: &str, rid: &str, want: &str) -> usize {
    store
        .load_events(run_id)
        .expect("events")
        .iter()
        .filter(|e| e.node_instance_id.as_deref() == Some(rid))
        .filter(|e| e.kind.discriminant() == want)
        .count()
}

// ── G-EFFECT — double command, single transition ─────────────────────────

/// Duplicate commands and duplicate drives collapse: exactly ONE journal
/// completion (`effect_completed == 1`), ZERO re-execution on the second
/// drive (`steps_executed == 0`, journal not grown), and ONE side output
/// for the key. What each assertion actually catches: a second
/// `effect_completed` would appear if the memoised short-circuit
/// (`completed_effect` → `DispatchOutcome::memoised`) were neutralised;
/// the side-output count catches an activity that re-applies instead of
/// collapsing on the key. (The side-output collapse here rests on
/// `FencedFileActivity`'s own key-dedup, exercised as the honest idempotent
/// target; the engine-side guarantee is the single-completion journal.)
#[test]
fn double_command_single_transition() {
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"seed": 1});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = FencedFileActivity::new(&c.log);
    let first = orch
        .execute_activity(&p, "run-dup", &input, &mut activity, &clock, &mut NoSignals)
        .expect("first drive");
    assert_eq!(first.view.run_state, RunState::Completed);
    assert_eq!(first.steps_executed, 1);

    let count_before = orch.store().event_count("run-dup").expect("count");
    let again = orch
        .execute_activity(&p, "run-dup", &input, &mut activity, &clock, &mut NoSignals)
        .expect("second drive");
    assert_eq!(again.steps_executed, 0, "no re-application on a re-drive");
    assert_eq!(
        orch.store().event_count("run-dup").expect("count"),
        count_before,
        "a duplicate drive must not grow the journal"
    );

    assert_eq!(
        count_kind(orch.store(), "run-dup", "/a", "effect_completed"),
        1
    );
    let done = orch
        .store()
        .completed_effect("run-dup", "/a")
        .expect("read")
        .expect("completed");
    let key = IdempotencyKey::new("run-dup", "/a", done.effect_id.as_str()).expect("key");
    assert_eq!(
        fenced_side_output_count(&c.log, &key).expect("count"),
        1,
        "exactly one side output per key"
    );
}

/// RED ARM at the door: a SECOND completion for an already-applied effect
/// is refused by the store (`state = pending` guard) — the duplicate
/// RESULT can never produce a second transition.
#[test]
fn duplicate_completion_refused_by_door() {
    let c = ctx();
    let p = single_leaf_plan();
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity = FencedFileActivity::new(&c.log);
    orch.execute_activity(
        &p,
        "run-dup-door",
        &serde_json::json!({"seed": 2}),
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("drive");
    let done = orch
        .store()
        .completed_effect("run-dup-door", "/a")
        .expect("read")
        .expect("completed");
    let err = orch
        .store()
        .record_completion(
            "run-dup-door",
            "/a",
            &done.effect_id,
            &done.request_hash,
            &done.receipt,
        )
        .expect_err("a second completion must be refused");
    assert!(matches!(err, StoreError::Corrupt(_)), "got {err:?}");
}

// ── G-UNKNOWN — the NEW §12.5 trigger: no receipt at the deadline ────────

/// A non-idempotent effect whose attempt crosses its deadline WITHOUT a
/// receipt is UNKNOWABLE: the run closes `outcome_unknown` — ZERO
/// automatic retry, never an invented success. This exercises the
/// timeout path added in Phase K, not the E/J pending-resume already
/// proven. What the assertions catch: the terminal reason is
/// `outcome_unknown` (not `node_failed` — a non-idempotent timeout is
/// UNKNOWABLE, not a retryable failure the budget could exhaust), exactly
/// ONE `start:` (no automatic retry), and ZERO `effect_attempt_failed`
/// (an unknowable outcome is journaled as a park/terminal, never as a
/// failed attempt).
#[test]
fn non_idempotent_without_receipt_becomes_unknown() {
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"seed": 3});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", false);
    activity.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);

    let progress = orch
        .execute_activity(&p, "run-unk", &input, &mut activity, &clock, &mut NoSignals)
        .expect("in-flight drive");
    assert_eq!(
        progress.view.node_states.get("/a").copied(),
        Some(NodeState::Running),
        "the attempt is in flight"
    );

    clock.set(601_000); // deadline = 1_000 + 600s (the plan's node timeout)
    let err = orch
        .execute_activity(&p, "run-unk", &input, &mut activity, &clock, &mut NoSignals)
        .expect_err("no receipt at the deadline must be unknowable");
    assert!(
        matches!(
            &err,
            OrchestratorError::Step(e) if matches!(**e, EngineError::OutcomeUnknown { .. })
        ),
        "got {err:?}"
    );

    assert_eq!(
        orch.store().terminal_reason("run-unk").expect("reason"),
        Some("outcome_unknown".to_string())
    );
    let starts = activity
        .calls
        .iter()
        .filter(|c| c.starts_with("start:"))
        .count();
    assert_eq!(starts, 1, "zero automatic retry: {:?}", activity.calls);
    assert_eq!(
        count_kind(orch.store(), "run-unk", "/a", "effect_attempt_failed"),
        0,
        "an unknowable outcome is not a failure"
    );
    assert!(
        orch.store()
            .open_attempts("run-unk")
            .expect("open")
            .is_empty(),
        "the terminal path closes its attempt bookkeeping too"
    );
}

// ── Retries, backoff and the plan's policies ─────────────────────────────

/// The per-node deadline comes from the plan's
/// `default_node_timeout_seconds` (first consumer) and a timed-out
/// IDEMPOTENT attempt retries under the SAME effect id, paced by the
/// durable backoff timer. Mutations killed: ignoring the plan timeout
/// (the 600_999 drive would already journal a failure), and minting a
/// fresh effect id on retry (the receipt would not carry attempt 2 under
/// the same key).
#[test]
fn node_timeout_and_retry_consume_plan_policies() {
    let c = ctx();
    let mut p = single_leaf_plan();
    p.error_policies.insert(
        "".to_string(),
        ErrorPolicy {
            on_node_failure: NodeFailureAction::FailWorkflow,
            max_retries_per_node: 2,
        },
    );
    let input = serde_json::json!({"seed": 4});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script(
        "/a",
        vec![
            PollOutcome::Pending,
            PollOutcome::Pending,
            PollOutcome::Pending,
            PollOutcome::Pending,
            ready(serde_json::json!({"ok": true})),
        ],
    );

    orch.execute_activity(
        &p,
        "run-retry",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");

    // One ms BEFORE the plan deadline: nothing may fail yet.
    clock.set(600_999);
    orch.execute_activity(
        &p,
        "run-retry",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("still in flight");
    assert_eq!(
        count_kind(orch.store(), "run-retry", "/a", "effect_attempt_failed"),
        0,
        "the plan's 600s node timeout must be honoured"
    );

    // AT the deadline: attempt 1 fails retryably, backoff timer armed.
    clock.set(601_000);
    orch.execute_activity(
        &p,
        "run-retry",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("timeout pass");
    assert_eq!(
        count_kind(orch.store(), "run-retry", "/a", "effect_attempt_failed"),
        1
    );

    // After the 1s backoff: attempt 2 under the SAME key succeeds.
    clock.set(602_000);
    let done_progress = orch
        .execute_activity(
            &p,
            "run-retry",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("retry pass");
    assert_eq!(done_progress.view.run_state, RunState::Completed);

    let done = orch
        .store()
        .completed_effect("run-retry", "/a")
        .expect("read")
        .expect("completed");
    let receipt = ActivityReceipt::parse(&done.receipt).expect("structured receipt");
    assert_eq!(receipt.attempt, 2, "the retry, not a fresh effect");
    assert_eq!(receipt.effect_id, done.effect_id, "same stable effect id");
    assert_eq!(receipt.kind, ACTIVITY_RECEIPT_KIND);
}

/// CRASH LEG of the retry pacing (ex-Codex K-RETRY-01): the journaled
/// failure and its backoff timer are TWO transactions — this test
/// reproduces the state a crash between them leaves (failure journaled,
/// attempt closed, NO `er:` timer, exactly what the direct store door
/// produces) and proves the next drive RE-ARMS the timer idempotently
/// instead of stranding the run in `FailedRetryable` forever. Mutation
/// killed: removing the re-arm block in `run_activity_effect` (no clock
/// would ever fire a nonexistent timer — the run would stay Running with
/// a single `start:`).
#[test]
fn retry_timer_lost_in_crash_rearmed_on_resume() {
    let c = ctx();
    let mut p = single_leaf_plan();
    p.error_policies.insert(
        "".to_string(),
        ErrorPolicy {
            on_node_failure: NodeFailureAction::FailWorkflow,
            max_retries_per_node: 2,
        },
    );
    let input = serde_json::json!({"seed": 16});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    // Pending repeats: the first drive leaves the attempt genuinely in
    // flight (the internal passes would consume a finite script).
    activity.script("/a", vec![PollOutcome::Pending]);
    orch.execute_activity(
        &p,
        "run-crash",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");

    // The crash state: journal the failure at the STORE door directly —
    // failure journaled + attempt closed, and NO backoff timer scheduled
    // (the orchestrator's second transaction never ran).
    let pending = orch
        .store()
        .pending_effect("run-crash", "/a")
        .expect("read")
        .expect("pending");
    orch.store()
        .record_attempt_failed("run-crash", "/a", &pending.effect_id, 1, true)
        .expect("failure without its timer");
    // Attempt 2 will confirm.
    activity.script("/a", vec![ready(serde_json::json!({"ok": true}))]);

    // Resume: this pass RE-ARMS the lost timer (deadline now + backoff).
    clock.set(10_000);
    orch.execute_activity(
        &p,
        "run-crash",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("re-arm pass");

    // Past the re-armed backoff (1s): the timer fires, attempt 2 runs and
    // the run completes — the run was never stranded.
    clock.set(11_100);
    let done = orch
        .execute_activity(
            &p,
            "run-crash",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("retry pass");
    assert_eq!(done.view.run_state, RunState::Completed);
    let starts = activity
        .calls
        .iter()
        .filter(|c| c.starts_with("start:"))
        .count();
    assert_eq!(
        starts, 2,
        "the crash-lost timer was re-armed and the retry ran: {:?}",
        activity.calls
    );
}

/// `max_retries_per_node` (first consumer) is a hard budget: once
/// exhausted the node settles FailedTerminal and `fail-workflow` closes
/// the run as `node_failed`. Mutation killed: an unbounded retry loop (a
/// third `start:` would appear instead of the terminal).
#[test]
fn retry_budget_exhausted_closes_run_node_failed() {
    let c = ctx();
    let mut p = single_leaf_plan();
    p.error_policies.insert(
        "".to_string(),
        ErrorPolicy {
            on_node_failure: NodeFailureAction::FailWorkflow,
            max_retries_per_node: 1,
        },
    );
    let input = serde_json::json!({"seed": 5});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script("/a", vec![failed(true), failed(true)]);

    orch.execute_activity(
        &p,
        "run-budget",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("first failure pass");
    clock.set(2_000); // past the 1s backoff
    let progress = orch
        .execute_activity(
            &p,
            "run-budget",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("budget-exhausting pass");
    assert_eq!(progress.view.run_state, RunState::Failed);
    assert_eq!(
        orch.store().terminal_reason("run-budget").expect("reason"),
        Some("node_failed".to_string())
    );
    assert_eq!(
        count_kind(orch.store(), "run-budget", "/a", "effect_attempt_failed"),
        2
    );
    let starts = activity
        .calls
        .iter()
        .filter(|c| c.starts_with("start:"))
        .count();
    assert_eq!(starts, 2, "budget is 1 + max_retries attempts, never more");
}

/// Under `route-error-port` a terminal failure DELIVERS on the reserved
/// `error` edge instead of closing the run: the error handler consumes a
/// deterministic error value and the workflow completes. Mutation
/// killed: treating a FailedTerminal source as dead on every port (the
/// handler would be skipped and the run would hang or fail).
#[test]
fn route_error_port_delivers_error_edge() {
    let c = ctx();
    let mut p = plan(
        &[("", &["/a"])],
        vec![("/a", leaf()), ("/b", leaf())],
        &[("/a", "error", "/b", "in")],
    );
    p.error_policies.insert(
        "".to_string(),
        ErrorPolicy {
            on_node_failure: NodeFailureAction::RouteErrorPort,
            max_retries_per_node: 0,
        },
    );
    let input = serde_json::json!({"seed": 6});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script("/a", vec![failed(false)]);
    activity.script("/b", vec![ready(serde_json::json!({"handled": true}))]);

    let progress = orch
        .execute_activity(&p, "run-err", &input, &mut activity, &clock, &mut NoSignals)
        .expect("error-routed run");
    assert_eq!(progress.view.run_state, RunState::Completed);
    assert_eq!(
        progress.view.node_states.get("/a").copied(),
        Some(NodeState::FailedTerminal)
    );
    assert_eq!(
        progress.view.node_states.get("/b").copied(),
        Some(NodeState::Succeeded)
    );

    // The handler consumed the DETERMINISTIC error value.
    let error_value = serde_json::json!({"error": {"node": "/a", "attempts": 1}});
    let mut expected_input = BTreeMap::new();
    expected_input.insert(
        "in".to_string(),
        canonical_string(&error_value).expect("canonical"),
    );
    let expected_hash = step_request_hash(
        &StepRequest::new("/b".to_string(), "scripted", expected_input).expect("request"),
    )
    .expect("hash");
    let done = orch
        .store()
        .completed_effect("run-err", "/b")
        .expect("read")
        .expect("completed");
    assert_eq!(done.request_hash, expected_hash);
}

/// FAIL-CLOSED: a terminal failure under `route-error-port` whose node has
/// NO wired `error` edge (here a leaf the plan left unwired) cannot be
/// routed — the run closes `node_failed` rather than silently completing
/// by swallowing the failure. Mutation killed: closing only under
/// `fail-workflow` (the failure would be swallowed and the run would
/// complete).
#[test]
fn route_error_port_without_edge_fails_closed() {
    let c = ctx();
    let mut p = single_leaf_plan();
    p.error_policies.insert(
        "".to_string(),
        ErrorPolicy {
            on_node_failure: NodeFailureAction::RouteErrorPort,
            max_retries_per_node: 0,
        },
    );
    let input = serde_json::json!({"seed": 14});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script("/a", vec![failed(false)]);

    let progress = orch
        .execute_activity(
            &p,
            "run-noedge",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("fails closed, not swallowed");
    assert_eq!(progress.view.run_state, RunState::Failed);
    assert_eq!(
        orch.store().terminal_reason("run-noedge").expect("reason"),
        Some("node_failed".to_string()),
        "a failure with nowhere to route must not be swallowed"
    );
}

// ── G-FS-PROCESS-EFFECT — worktree-bound FS effects ──────────────────────

/// A worktree-state precondition mismatch REFUSES the effect before any
/// byte is written. Mutation killed: disabling the precondition compare
/// (the file would appear and the run would complete).
#[test]
fn fs_effect_bound_to_head_index_worktree() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let c = ctx();
    let root = c.dir.path().join("wt");
    std::fs::create_dir(&root).expect("mkdir");
    let actual = WorktreeState {
        head: "a".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let expected_divergent = WorktreeState {
        head: "b".repeat(40),
        ..actual.clone()
    };
    let p = single_leaf_plan();
    let input = serde_json::json!({"path": "file.txt", "content": "hello"});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = WorktreeFsWriteActivity::new(
        &root,
        Box::new(FixedWorktreeState(actual.clone())),
        expected_divergent,
    );
    let progress = orch
        .execute_activity(
            &p,
            "run-fs-miss",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    assert_eq!(
        orch.store().terminal_reason("run-fs-miss").expect("reason"),
        Some("node_failed".to_string())
    );
    assert!(
        !root.join("file.txt").exists(),
        "a precondition mismatch must not mutate the tree"
    );

    // Matching state: the SAME request applies, idempotently.
    let mut orch2 = Orchestrator::new(
        RunStore::open(&c.dir.path().join("run2.sqlite3")).expect("open"),
        OrchestratorConfig::new("test-writer"),
    );
    let mut activity2 = WorktreeFsWriteActivity::new(
        &root,
        Box::new(FixedWorktreeState(actual.clone())),
        actual.clone(),
    );
    let done = orch2
        .execute_activity(
            &p,
            "run-fs-hit",
            &input,
            &mut activity2,
            &clock,
            &mut NoSignals,
        )
        .expect("matching state applies");
    assert_eq!(done.view.run_state, RunState::Completed);
    assert_eq!(
        std::fs::read_to_string(root.join("file.txt")).expect("written"),
        "hello"
    );
}

/// The FS precondition is bound to the DURABLE effect identity, replay-safe
/// (ex-Codex K-FS-STATE-BIND-01): the EXPECTED worktree state is folded
/// into `dedup_scope`, which the store journals and re-compares on resume
/// (`TargetScopeChanged` fail-closed). So an intent journaled against
/// expected state S0 cannot be resumed by an activity reconstructed against
/// a DIFFERENT state S1 — the plan was made for S0, not S1. Mutation
/// killed: dropping the expected state from `dedup_scope` (the two scopes
/// would collide and a moved-worktree resume would pass every durable
/// guard and mutate the tree). This proves the BINDING; the general
/// `TargetScopeChanged` machinery is proven at the store/spike level.
#[test]
fn fs_effect_precondition_bound_to_durable_identity() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let root = std::path::Path::new("wt");
    let s0 = WorktreeState {
        head: "0".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let s1 = WorktreeState {
        head: "9".repeat(40),
        ..s0.clone()
    };
    let a0 =
        WorktreeFsWriteActivity::new(root, Box::new(FixedWorktreeState(s0.clone())), s0.clone());
    let a1 = WorktreeFsWriteActivity::new(root, Box::new(FixedWorktreeState(s0)), s1);
    assert_ne!(
        a0.dedup_scope(),
        a1.dedup_scope(),
        "a different expected worktree state must change the dedup scope"
    );
}

/// The worktree root inside `dedup_scope` is CANONICALISED to an absolute
/// path (ex-Codex r3 K-FS-ROOT-REL): a RELATIVE root kept raw would keep
/// the SAME scope string across a resume in a different working directory
/// while designating a DIFFERENT tree — the durable guard would pass and
/// the effect would mutate a tree the plan was never made for. Proven by
/// aliasing: a relative spelling and the absolute spelling of the SAME
/// directory must produce ONE scope. Mutation killed: dropping the
/// constructor `canonicalize` freeze (the raw relative spelling then
/// diverges from the absolute one and the two scopes differ).
///
/// Invariant relied upon: this test binary NEVER mutates the process cwd
/// (the constructor freeze resolves a relative spelling against the LIVE
/// cwd at construction time, not the captured variable) — keep
/// `set_current_dir` out of `effects.rs` or this assertion turns flaky
/// under shared-process `cargo test`.
#[test]
fn fs_effect_root_canonicalised_in_dedup_scope() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let cwd = std::env::current_dir().expect("cwd");
    let name = cwd.file_name().expect("cwd has a name").to_owned();
    let relative = std::path::Path::new("..").join(name);
    let s = WorktreeState {
        head: "0".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let rel = WorktreeFsWriteActivity::new(
        &relative,
        Box::new(FixedWorktreeState(s.clone())),
        s.clone(),
    );
    let abs = WorktreeFsWriteActivity::new(&cwd, Box::new(FixedWorktreeState(s.clone())), s);
    assert_eq!(
        rel.dedup_scope(),
        abs.dedup_scope(),
        "a relative and an absolute spelling of the same root must share one durable scope"
    );
}

/// Confinement: a path escaping the worktree (`..`) is refused before
/// any write. Mutation killed: dropping the component check.
#[test]
fn fs_effect_confinement_refuses_escape() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let c = ctx();
    let root = c.dir.path().join("wt");
    std::fs::create_dir(&root).expect("mkdir");
    let state = WorktreeState {
        head: "a".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let p = single_leaf_plan();
    let input = serde_json::json!({"path": "../evil.txt", "content": "escape"});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity =
        WorktreeFsWriteActivity::new(&root, Box::new(FixedWorktreeState(state.clone())), state);
    let progress = orch
        .execute_activity(
            &p,
            "run-fs-esc",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    assert!(
        !c.dir.path().join("evil.txt").exists(),
        "an escaping path must never be written"
    );
}

// ── Target-side fencing (the Phase E clause-3 window) ────────────────────

/// A DEPOSED writer is rejected AT THE TARGET: an application presenting
/// a lower epoch than the highest the backend has durably seen refuses —
/// the window between a committed intent and `apply` no longer belongs
/// to deduplication alone.
#[test]
fn stale_writer_rejected_at_target() {
    let c = ctx();
    let mut current = FencedFileActivity::new(&c.log);
    let key_b = IdempotencyKey::new("run-f", "/b", "e-current").expect("key");
    let ctx_b = EffectContext {
        key: key_b,
        attempt: 1,
        fencing_epoch: 2,
        deadline_ms: i64::MAX,
        input: BTreeMap::from([("in".to_string(), "{\"v\":2}".to_string())]),
    };
    current.start(&ctx_b).expect("current writer applies");

    let mut stale = FencedFileActivity::new(&c.log);
    let key_a = IdempotencyKey::new("run-f", "/a", "e-stale").expect("key");
    let ctx_a = EffectContext {
        key: key_a,
        attempt: 1,
        fencing_epoch: 1,
        deadline_ms: i64::MAX,
        input: BTreeMap::from([("in".to_string(), "{\"v\":1}".to_string())]),
    };
    let err = stale
        .start(&ctx_a)
        .expect_err("a deposed writer must be refused");
    assert!(
        matches!(
            err,
            ActivityError::Fenced {
                seen: 2,
                presented: 1
            }
        ),
        "got {err:?}"
    );
    assert_eq!(
        read_fenced_side_outputs(&c.log).expect("read").len(),
        1,
        "the stale application left no side output"
    );
}

/// RED CONTROL — the same stale application with the fence check REMOVED
/// goes through and appends: the refusal above comes from the check, not
/// from wishful thinking.
#[test]
fn fencing_red_control_double_output() {
    let c = ctx();
    let mut current = FencedFileActivity::new(&c.log);
    let ctx_b = EffectContext {
        key: IdempotencyKey::new("run-f", "/b", "e-current").expect("key"),
        attempt: 1,
        fencing_epoch: 2,
        deadline_ms: i64::MAX,
        input: BTreeMap::from([("in".to_string(), "{\"v\":2}".to_string())]),
    };
    current.start(&ctx_b).expect("current writer applies");

    let mut unfenced = FencedFileActivity::ignoring_fence(&c.log);
    let ctx_a = EffectContext {
        key: IdempotencyKey::new("run-f", "/a", "e-stale").expect("key"),
        attempt: 1,
        fencing_epoch: 1,
        deadline_ms: i64::MAX,
        input: BTreeMap::from([("in".to_string(), "{\"v\":1}".to_string())]),
    };
    unfenced
        .start(&ctx_a)
        .expect("without the check the stale writer applies");
    assert_eq!(
        read_fenced_side_outputs(&c.log).expect("read").len(),
        2,
        "the bad state IS reachable when the guard is off"
    );
}

// ── §12.7 — late result after cancel ─────────────────────────────────────

/// A result that finished JUST BEFORE the cancel stop is KEPT through
/// `record_late_completion` — journaled, frontier untouched, downstream
/// NEVER dispatched. This is the production caller of the late door.
/// Mutation killed: routing the late result through the normal
/// completion (the frontier would advance) or dropping it (no
/// `effect_completed` entry).
#[test]
fn late_result_after_cancel_kept_without_propagation() {
    let c = ctx();
    let p = two_leaf_plan();
    let input = serde_json::json!({"seed": 7});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script(
        "/a",
        vec![
            PollOutcome::Pending,
            PollOutcome::Pending,
            ready(serde_json::json!({"late": true})),
        ],
    );

    orch.execute_activity(
        &p,
        "run-late",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    assert!(orch.request_cancel("run-late", &clock).expect("cancel"));

    let progress = orch
        .execute_activity(
            &p,
            "run-late",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("cancel drain");
    assert_eq!(progress.view.run_state, RunState::Cancelled);

    assert_eq!(
        count_kind(orch.store(), "run-late", "/a", "effect_completed"),
        1,
        "the late result is KEPT"
    );
    assert_eq!(
        orch.store().frontier_node("run-late").expect("frontier"),
        None,
        "the late door never advances the frontier"
    );
    assert_eq!(
        count_kind(orch.store(), "run-late", "/out", "effect_intended"),
        0,
        "nothing downstream is ever dispatched"
    );
    assert!(
        orch.store()
            .open_attempts("run-late")
            .expect("open")
            .is_empty()
    );
}

// ── §12.5 — park and reconcile ───────────────────────────────────────────

/// With the host's reconciliation policy ON, an unknowable outcome PARKS
/// the node (run alive) and an operator `result_recovered` — carrying
/// the ACTUAL receipt — settles it through the completion door; the run
/// then continues downstream. Mutation killed: a reconciliation that
/// completes without evidence (the reducer's causal guard goes red — see
/// `completion_on_parked_without_reconciliation_refused`).
#[test]
fn park_and_reconcile_result_recovered_continues_run() {
    let c = ctx();
    let p = two_leaf_plan();
    let input = serde_json::json!({"seed": 8});
    let clock = ManualClock::at(1_000);

    let mut orch = parking_orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", false);
    activity.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
    activity.script("/out", vec![ready(serde_json::json!({"published": true}))]);

    orch.execute_activity(
        &p,
        "run-park",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    clock.set(601_000);
    let parked = orch
        .execute_activity(
            &p,
            "run-park",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("parking drive");
    assert_eq!(parked.view.parked_unknown, vec!["/a".to_string()]);
    assert_eq!(parked.view.run_state, RunState::Running);
    assert_eq!(
        orch.store().run_status("run-park").expect("status"),
        Some(RunStatus::Running),
        "a parked run stays alive"
    );

    // The operator recovered the ACTUAL result from the target's backend.
    let pending = orch
        .store()
        .pending_effect("run-park", "/a")
        .expect("read")
        .expect("pending");
    let recovered_output = serde_json::json!({"recovered": true});
    let recovered = ActivityReceipt {
        kind: ACTIVITY_RECEIPT_KIND.to_string(),
        run_id: "run-park".to_string(),
        node_instance_id: "/a".to_string(),
        effect_id: pending.effect_id.clone(),
        attempt: 1,
        request_hash: pending.request_hash.clone(),
        activity: "scripted".to_string(),
        dedup_scope: pending.dedup_scope.clone(),
        fencing_epoch: 0,
        facts: BTreeMap::new(),
        output: recovered_output.clone(),
    }
    .render()
    .expect("render");
    orch.reconcile_unknown(
        "run-park",
        "/a",
        ReconcileResolution::ResultRecovered,
        Some(&recovered),
        &clock,
    )
    .expect("reconcile");

    let done = orch
        .execute_activity(
            &p,
            "run-park",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("continue downstream");
    assert_eq!(done.view.run_state, RunState::Completed);

    // Causal order in the journal: park → reconciliation → completion.
    let events = orch.store().load_events("run-park").expect("events");
    let pos = |want: &str| {
        events
            .iter()
            .position(|e| {
                e.node_instance_id.as_deref() == Some("/a") && e.kind.discriminant() == want
            })
            .unwrap_or_else(|| panic!("missing {want}"))
    };
    assert!(pos("node_outcome_unknown") < pos("reconciliation_recorded"));
    assert!(pos("reconciliation_recorded") < pos("effect_completed"));

    // Downstream consumed the RECOVERED value.
    let mut expected_input = BTreeMap::new();
    expected_input.insert(
        "in".to_string(),
        canonical_string(&recovered_output).expect("canonical"),
    );
    let expected_hash = step_request_hash(
        &StepRequest::new("/out".to_string(), "scripted", expected_input).expect("request"),
    )
    .expect("hash");
    let out_done = orch
        .store()
        .completed_effect("run-park", "/out")
        .expect("read")
        .expect("completed");
    assert_eq!(out_done.request_hash, expected_hash);
}

/// `abandon` closes the run as `outcome_unknown` — and the closed state
/// is ABSORBING: a later `result_recovered` is refused, so a terminal
/// unknown can NEVER become a success (K.4's binary condition).
#[test]
fn reconcile_abandon_is_terminal_and_absorbing() {
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"seed": 9});
    let clock = ManualClock::at(1_000);

    let mut orch = parking_orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", false);
    activity.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);

    orch.execute_activity(
        &p,
        "run-aband",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    clock.set(601_000);
    orch.execute_activity(
        &p,
        "run-aband",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("parking drive");

    orch.reconcile_unknown(
        "run-aband",
        "/a",
        ReconcileResolution::Abandon,
        None,
        &clock,
    )
    .expect("abandon");
    assert_eq!(
        orch.store().run_status("run-aband").expect("status"),
        Some(RunStatus::Terminal)
    );
    assert_eq!(
        orch.store().terminal_reason("run-aband").expect("reason"),
        Some("outcome_unknown".to_string())
    );

    // A VALID, correctly-bound receipt (so it passes the evidence gate)
    // still cannot resurrect the terminal run: the store's absorbing guard
    // refuses it. (A forged/opaque blob is rejected even earlier, by the
    // evidence gate — proven in `reconcile_recovered_rejects_foreign_receipt`.)
    let pending = orch
        .store()
        .pending_effect("run-aband", "/a")
        .expect("read")
        .expect("pending");
    let bound = ActivityReceipt {
        kind: ACTIVITY_RECEIPT_KIND.to_string(),
        run_id: "run-aband".to_string(),
        node_instance_id: "/a".to_string(),
        effect_id: pending.effect_id.clone(),
        attempt: 1,
        request_hash: pending.request_hash.clone(),
        activity: "scripted".to_string(),
        dedup_scope: pending.dedup_scope.clone(),
        fencing_epoch: 0,
        facts: BTreeMap::new(),
        output: serde_json::json!({"recovered": true}),
    }
    .render()
    .expect("render");
    let err = orch
        .reconcile_unknown(
            "run-aband",
            "/a",
            ReconcileResolution::ResultRecovered,
            Some(&bound),
            &clock,
        )
        .expect_err("a terminal unknown must never become a success");
    assert!(
        matches!(
            err,
            OrchestratorError::Store(StoreError::InvalidTransition { .. })
        ),
        "got {err:?}"
    );
    assert_eq!(
        orch.store().terminal_reason("run-aband").expect("reason"),
        Some("outcome_unknown".to_string()),
        "the diagnosis survives"
    );
}

/// A `result_recovered` without the receipt is refused: recovery without
/// evidence is the laundering K.4 forbids.
#[test]
fn reconcile_recovered_requires_receipt() {
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"seed": 10});
    let clock = ManualClock::at(1_000);
    let mut orch = parking_orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", false);
    activity.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
    orch.execute_activity(
        &p,
        "run-noev",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    clock.set(601_000);
    orch.execute_activity(
        &p,
        "run-noev",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("parking drive");
    let err = orch
        .reconcile_unknown(
            "run-noev",
            "/a",
            ReconcileResolution::ResultRecovered,
            None,
            &clock,
        )
        .expect_err("no receipt, no recovery");
    assert!(matches!(err, OrchestratorError::ReconcileNeedsReceipt));
}

/// A `result_recovered` carrying a structured receipt bound to a DIFFERENT
/// effect is refused: a receipt for another effect is not evidence for
/// this one ("requires the receipt" must mean the RIGHT receipt).
/// Mutation killed (loop r2, "parse-laxiste"): re-accepting an opaque
/// unparsable blob as a receipt (the empty/garbage/foreign-kind arms of
/// this test turn red — a lax parse would mint a success out of noise).
#[test]
fn reconcile_recovered_rejects_foreign_receipt() {
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"seed": 13});
    let clock = ManualClock::at(1_000);
    let mut orch = parking_orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", false);
    activity.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
    orch.execute_activity(
        &p,
        "run-foreign",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    clock.set(601_000);
    orch.execute_activity(
        &p,
        "run-foreign",
        &input,
        &mut activity,
        &clock,
        &mut NoSignals,
    )
    .expect("parking drive");

    let pending = orch
        .store()
        .pending_effect("run-foreign", "/a")
        .expect("read")
        .expect("pending");
    // A receipt whose effect_id belongs to another effect.
    let foreign = ActivityReceipt {
        kind: ACTIVITY_RECEIPT_KIND.to_string(),
        run_id: "run-foreign".to_string(),
        node_instance_id: "/a".to_string(),
        effect_id: "some-other-effect".to_string(),
        attempt: 1,
        request_hash: pending.request_hash.clone(),
        activity: "scripted".to_string(),
        dedup_scope: pending.dedup_scope.clone(),
        fencing_epoch: 0,
        facts: BTreeMap::new(),
        output: serde_json::json!({"forged": true}),
    }
    .render()
    .expect("render");
    let err = orch
        .reconcile_unknown(
            "run-foreign",
            "/a",
            ReconcileResolution::ResultRecovered,
            Some(&foreign),
            &clock,
        )
        .expect_err("a foreign receipt is not evidence");
    assert!(
        matches!(err, OrchestratorError::ReconcileReceiptMismatch { .. }),
        "got {err:?}"
    );

    // An OPAQUE (non-`ActivityReceipt`) blob is ALSO refused — recovery is
    // evidence-only, and an empty/foreign string is not evidence. Mutation
    // killed (ex-Codex K-RECONCILE-RAW-01): accepting it in the `if let
    // Some(parsed)` shape would launder the unknown into a success and
    // advance the run.
    for blob in ["", "not json", "{\"kind\":\"other\"}"] {
        let err = orch
            .reconcile_unknown(
                "run-foreign",
                "/a",
                ReconcileResolution::ResultRecovered,
                Some(blob),
                &clock,
            )
            .expect_err("an opaque blob is not evidence");
        assert!(
            matches!(err, OrchestratorError::ReconcileReceiptMismatch { .. }),
            "blob {blob:?} → {err:?}"
        );
    }
    // The node is still parked, never completed.
    assert_eq!(
        orch.store().run_status("run-foreign").expect("status"),
        Some(RunStatus::Running)
    );
    assert_eq!(
        count_kind(orch.store(), "run-foreign", "/a", "effect_completed"),
        0,
        "no opaque blob ever produced a completion"
    );
}

// ── Cancel drain reaps an orphan (K-CANCEL-01) ───────────────────────────

/// On cancel, an attempt whose activity incarnation no longer knows the key
/// (a previous incarnation's orphan) is REAPED by its journaled handle —
/// not left to a no-op `cancel`. Mutation killed: routing `UnknownAttempt`
/// through the plain `cancel` branch (the `reap:` call vanishes and a
/// surviving process group would leak past the cancel).
#[test]
fn cancel_drain_reaps_orphan_by_journaled_handle() {
    let c = ctx();
    let p = two_leaf_plan();
    let input = serde_json::json!({"seed": 15});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    // The first incarnation reports a durable handle; the orchestrator
    // persists it (`set_attempt_handle`).
    let mut first = ScriptedActivity::new("scripted", true).with_handle("pgid:4242");
    first.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
    orch.execute_activity(
        &p,
        "run-orphan-cancel",
        &input,
        &mut first,
        &clock,
        &mut NoSignals,
    )
    .expect("in-flight");
    assert!(
        orch.request_cancel("run-orphan-cancel", &clock)
            .expect("cancel")
    );

    // A fresh incarnation that does NOT know the in-flight key drives the
    // cancel drain.
    let mut reborn = ScriptedActivity::new("scripted", true);
    let progress = orch
        .execute_activity(
            &p,
            "run-orphan-cancel",
            &input,
            &mut reborn,
            &clock,
            &mut NoSignals,
        )
        .expect("cancel drain");
    assert_eq!(progress.view.run_state, RunState::Cancelled);
    // The reap received the JOURNALED handle — replacing
    // `open.runtime_handle` with `None` in the drain would fail this.
    assert!(
        reborn.calls.iter().any(|c| c == "reap:/a:pgid:4242"),
        "the drain reaped the orphan by its journaled handle: {:?}",
        reborn.calls
    );
    assert!(
        orch.store()
            .open_attempts("run-orphan-cancel")
            .expect("open")
            .is_empty()
    );
}

// ── Reconciliation causal order (K-CAUSAL-01) ────────────────────────────

/// A `reconciliation_recorded(result_recovered)` that PRECEDES its park is
/// refused at reduction — the shape that would pre-authorise a laundering
/// completion (`intent → reconciliation → park → completion`). Only
/// `park → reconciliation → completion` is a valid order.
#[test]
fn reconciliation_before_park_refused_by_reducer() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-preauth", "digest").expect("run");
    store
        .record_intent_atomic("run-preauth", "/a", "hash", "e", false, "scope")
        .expect("intent");
    // Reconciliation BEFORE any park.
    store
        .append_raw_node_event_for_harness(
            "run-preauth",
            "/a",
            "reconciliation_recorded",
            "{\"kind\":\"reconciliation_recorded\",\"effect_id\":\"e\",\"origin\":\"host_operator\",\"resolution\":\"result_recovered\"}",
        )
        .expect("raw append");
    let events = store.load_events("run-preauth").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("pre-authorisation refused");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("reconciliation without a matching prior park")
        ),
        "got {err:?}"
    );
}

/// The reducer binds the EFFECT ID across the effect-layer events
/// (ex-Codex K-CAUSAL-02): a journal that parks effect `e1` but reconciles
/// a DIFFERENT effect `e2` on the same node is causally invalid — the
/// reconciliation does not match its park. Under production doors the
/// writer always supplies the node's single effect id; this guards a
/// foreign/corrupt journal. Mutation killed (loop r2, "contains_key"):
/// weakening the reducer's id-equality check to mere key presence (the
/// e1/e2 mismatch then reduces cleanly and this test turns red).
#[test]
fn reconciliation_effect_id_mismatch_refused_by_reducer() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-idmix", "digest").expect("run");
    store
        .record_intent_atomic("run-idmix", "/a", "hash", "e1", false, "scope")
        .expect("intent");
    store
        .record_node_outcome_unknown("run-idmix", "/a", "e1")
        .expect("park e1");
    // Reconciliation names a DIFFERENT effect id.
    store
        .append_raw_node_event_for_harness(
            "run-idmix",
            "/a",
            "reconciliation_recorded",
            "{\"kind\":\"reconciliation_recorded\",\"effect_id\":\"e2\",\"origin\":\"host_operator\",\"resolution\":\"result_recovered\"}",
        )
        .expect("raw append");
    let events = store.load_events("run-idmix").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("effect-id mismatch refused");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("reconciliation without a matching prior park")
        ),
        "got {err:?}"
    );
}

// ── Process env is part of the dedup scope (K-PROC-SCOPE-01) ──────────────

/// Two process activities with the SAME program and args but DIFFERENT
/// granted environments have DISTINCT dedup scopes — a resume with a
/// different env cannot pass the scope guard as the same observable work.
#[test]
fn process_env_is_part_of_dedup_scope() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let mut env_a = BTreeMap::new();
    env_a.insert("PATH".to_string(), "/a".to_string());
    let mut env_b = BTreeMap::new();
    env_b.insert("PATH".to_string(), "/b".to_string());
    let cwd = std::env::temp_dir();
    let a = ProcessActivity::new("p", "/bin/true", &cwd, vec![], true, false, env_a);
    let b = ProcessActivity::new("p", "/bin/true", &cwd, vec![], true, false, env_b);
    assert_ne!(
        a.dedup_scope(),
        b.dedup_scope(),
        "a different env must change the dedup scope"
    );
}

/// Two process activities differing ONLY in their fixed working directory
/// have DISTINCT dedup scopes (ex-Codex r4 K-PROC-CWD-SCOPE-03): without
/// the cwd in the durable identity, a resume in another directory kept the
/// same token while a relative program — or its relative I/O — designated
/// another tree. Mutation killed: dropping `"cwd"` from the scope JCS (the
/// two scopes collide and this assertion turns red).
#[test]
fn process_cwd_is_part_of_dedup_scope() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let dir_a = tempfile::tempdir().expect("tempdir a");
    let dir_b = tempfile::tempdir().expect("tempdir b");
    let a = ProcessActivity::new(
        "p",
        "/bin/true",
        dir_a.path(),
        vec![],
        true,
        false,
        BTreeMap::new(),
    );
    let b = ProcessActivity::new(
        "p",
        "/bin/true",
        dir_b.path(),
        vec![],
        true,
        false,
        BTreeMap::new(),
    );
    assert_ne!(
        a.dedup_scope(),
        b.dedup_scope(),
        "a different working directory must change the dedup scope"
    );
}

/// The FROZEN constructor cwd is CANONICALISED: a relative and an
/// absolute spelling of the SAME directory produce ONE durable scope —
/// mirror of `fs_effect_root_canonicalised_in_dedup_scope` (ex-review
/// K-PROC-CWD-FREEZE-NOTEST: without this test, dropping the constructor
/// freeze left the whole suite green while a relative cwd kept its raw
/// spelling in the scope). Mutation killed: dropping the `canonicalize`
/// in `ProcessActivity::new` (the two scopes diverge). Same invariant as
/// the FS aliasing test: this binary never mutates the process cwd.
#[test]
fn process_cwd_canonicalised_in_dedup_scope() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let cwd = std::env::current_dir().expect("cwd");
    let name = cwd.file_name().expect("cwd has a name").to_owned();
    let relative = std::path::Path::new("..").join(name);
    let rel = ProcessActivity::new(
        "p",
        "/bin/true",
        &relative,
        vec![],
        true,
        false,
        BTreeMap::new(),
    );
    let abs = ProcessActivity::new("p", "/bin/true", &cwd, vec![], true, false, BTreeMap::new());
    assert_eq!(
        rel.dedup_scope(),
        abs.dedup_scope(),
        "a relative and an absolute spelling of the same cwd must share one durable scope"
    );
}

/// An ABSOLUTE program is frozen canonical at construction (ex-Codex r5
/// K-PROC-PROGRAM-RESOLVE-05): two absolute spellings of the SAME
/// executable (one routed through `sub/..`) produce ONE durable scope.
/// Mutation killed: dropping the program freeze in `ProcessActivity::new`
/// (the two spellings diverge and the scopes differ).
#[test]
fn process_program_canonicalised_in_dedup_scope() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join("sub")).expect("mkdir sub");
    let prog = dir.path().join("prog.exe");
    std::fs::write(&prog, b"stub").expect("write prog");
    let aliased = dir.path().join("sub").join("..").join("prog.exe");
    let a = ProcessActivity::new(
        "p",
        &aliased,
        dir.path(),
        vec![],
        true,
        false,
        BTreeMap::new(),
    );
    let b = ProcessActivity::new("p", &prog, dir.path(), vec![], true, false, BTreeMap::new());
    assert_eq!(
        a.dedup_scope(),
        b.dedup_scope(),
        "two absolute spellings of the same program must share one durable scope"
    );
}

/// A RELATIVE program spelling WITH separators is REFUSED at `start`,
/// BEFORE any spawn (ex-Codex r5 K-PROC-PROGRAM-RESOLVE-05): std documents
/// its resolution as platform-ambiguous (parent cwd vs child
/// `current_dir`), so one durable scope could run two different
/// executables across a resume. Discriminating assertion: the failure is
/// the GATE's `Refused` (non-retryable) — mutation killed: dropping the
/// gate in `start` (the spawn then fails as a retryable `Io`, or worse
/// succeeds, and the `vec![false]` assertion turns red).
#[test]
fn process_relative_program_refused() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"noop": true});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc-rel",
        std::path::Path::new("bin").join("tool"),
        c.dir.path(),
        vec![],
        true,
        false,
        host_env(),
    );
    let progress = orch
        .execute_activity(&p, "run-rel", &input, &mut activity, &clock, &mut NoSignals)
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    let failures: Vec<bool> = orch
        .store()
        .load_events("run-rel")
        .expect("events")
        .iter()
        .filter_map(|e| match &e.kind {
            RunEventKind::EffectAttemptFailed { retryable, .. } => Some(*retryable),
            _ => None,
        })
        .collect();
    assert_eq!(
        failures,
        vec![false],
        "the refusal must come from the program gate (non-retryable)"
    );
}

/// A BARE program name is REFUSED at `start` like any non-absolute
/// spelling (ex-Codex r6 K-PROC-BARE-PATH-AMBIENT-06): the Windows std
/// resolver (`resolve_exe`) falls back to the parent executable's
/// directory, the system directories and the PARENT's ambient `PATH` when
/// the scoped child map misses the name — one durable scope could run two
/// different executables across a resume. The engine resolves nothing:
/// the host injects an ABSOLUTE path. Mutation killed: narrowing the gate
/// back to `components().count() > 1` (the bare name then reaches the
/// spawn and this `vec![false]` assertion turns red).
#[test]
fn process_bare_program_refused() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"noop": true});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc-bare",
        "tool",
        c.dir.path(),
        vec![],
        true,
        false,
        host_env(),
    );
    let progress = orch
        .execute_activity(
            &p,
            "run-bare",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    let failures: Vec<bool> = orch
        .store()
        .load_events("run-bare")
        .expect("events")
        .iter()
        .filter_map(|e| match &e.kind {
            RunEventKind::EffectAttemptFailed { retryable, .. } => Some(*retryable),
            _ => None,
        })
        .collect();
    assert_eq!(
        failures,
        vec![false],
        "the refusal must come from the absolute-only gate (non-retryable)"
    );
}

/// Windows arm: an ABSOLUTE program WITHOUT the `.exe` suffix is REFUSED
/// at `start` (ex-Codex r7 K-PROC-ABS-EXE-SUFFIX-07): std re-resolves a
/// suffix-less absolute by trying the `path.exe` SIBLING before `path`
/// itself (`resolve_exe`), so one durable scope could cover two files and
/// the pick could flip between attempts when the sibling appears or
/// vanishes. Mutation killed: dropping the suffix gate (the spawn then
/// fails as a retryable `Io` — or worse, runs the sibling — and the
/// `vec![false]` assertion turns red). The `current_exe()`-based tests
/// stay green: a real test binary DOES end in `.exe`.
#[cfg(windows)]
#[test]
fn process_absolute_without_exe_refused_on_windows() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"noop": true});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc-noexe",
        c.dir.path().join("tool"),
        c.dir.path(),
        vec![],
        true,
        false,
        host_env(),
    );
    let progress = orch
        .execute_activity(
            &p,
            "run-noexe",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    let failures: Vec<bool> = orch
        .store()
        .load_events("run-noexe")
        .expect("events")
        .iter()
        .filter_map(|e| match &e.kind {
            RunEventKind::EffectAttemptFailed { retryable, .. } => Some(*retryable),
            _ => None,
        })
        .collect();
    assert_eq!(
        failures,
        vec![false],
        "the refusal must come from the `.exe` suffix gate (non-retryable)"
    );
}

/// A BARE leaf log spelling ("effects.log", `parent() == Some("")`) is
/// frozen against the CONSTRUCTION cwd (ex-Codex r6 K-FENCE-BARE-LEAF-06):
/// the bare spelling and the absolute spelling of the SAME future log in
/// the cwd produce ONE durable scope — the exact r5 scenario (one scope
/// string, two logs across two ambient cwd) cannot ride the empty-parent
/// branch. Mutation killed: dropping the `.`-substitution in
/// `freeze_side_output_path` (the bare spelling stays raw and diverges).
/// Same invariant as the other aliasing tests: this binary never mutates
/// the process cwd.
#[test]
fn fenced_bare_leaf_frozen_in_dedup_scope() {
    let cwd = std::env::current_dir().expect("cwd");
    let bare = FencedFileActivity::new("k_fence_bare_leaf_probe.log");
    let abs = FencedFileActivity::new(cwd.join("k_fence_bare_leaf_probe.log"));
    assert_eq!(
        bare.dedup_scope(),
        abs.dedup_scope(),
        "a bare and an absolute spelling of the same future log must share one durable scope"
    );
}

/// The fenced backend's log path is FROZEN at construction (ex-Codex r5
/// K-FENCE-SCOPE-REL-05): for the nominal not-yet-created log, the
/// canonical PARENT joined with the leaf name — two spellings of the SAME
/// future log (one routed through `sub/..`) produce ONE durable scope, so
/// a resume cannot silently bind the dedup/fencing to a DIFFERENT journal.
/// Mutation killed: dropping `freeze_side_output_path` in
/// `FencedFileActivity::new` (the two spellings diverge).
#[test]
fn fenced_log_path_frozen_in_dedup_scope() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir(dir.path().join("sub")).expect("mkdir sub");
    let aliased = dir.path().join("sub").join("..").join("effects.log");
    let plain = dir.path().join("effects.log");
    let a = FencedFileActivity::new(&aliased);
    let b = FencedFileActivity::new(&plain);
    assert_eq!(
        a.dedup_scope(),
        b.dedup_scope(),
        "two spellings of the same future log must share one durable scope"
    );
}

// ── Reducer causal guards (Phase K hardening) ────────────────────────────

/// A completion with no intent before it is refused at reduction — a
/// journal no production door can write must not derive anything.
#[test]
fn completion_without_intent_refused_by_reducer() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-causal", "digest").expect("run");
    store
        .append_raw_node_event_for_harness(
            "run-causal",
            "/a",
            "effect_completed",
            "{\"kind\":\"effect_completed\",\"effect_id\":\"e\",\"receipt_digest\":\"d\"}",
        )
        .expect("raw append");
    let events = store.load_events("run-causal").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("causally invalid");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("without a matching prior intent")
        ),
        "got {err:?}"
    );
}

/// A completion on a PARKED node with no reconciliation before it is the
/// laundering shape — refused at reduction.
#[test]
fn completion_on_parked_without_reconciliation_refused() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-launder", "digest").expect("run");
    store
        .record_intent_atomic("run-launder", "/a", "hash", "e", false, "scope")
        .expect("intent");
    store
        .record_node_outcome_unknown("run-launder", "/a", "e")
        .expect("park");
    store
        .append_raw_node_event_for_harness(
            "run-launder",
            "/a",
            "effect_completed",
            "{\"kind\":\"effect_completed\",\"effect_id\":\"e\",\"receipt_digest\":\"d\"}",
        )
        .expect("raw append");
    let events = store.load_events("run-launder").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("laundering refused");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("without a matching reconciliation")
        ),
        "got {err:?}"
    );
}

// ── JoinFired — the frozen cohort (J carry) ──────────────────────────────

/// A fired join journals its delivered cohort, and a source arriving
/// LATER no longer replaces the `any` winner: downstream consumes the
/// FROZEN value. Mutation killed: re-observing the cohort at value
/// resolution (canonical order would hand the later `a` to `/out`).
#[test]
fn join_fired_freezes_cohort() {
    let c = ctx();
    let p = plan(
        &[("", &["/p"])],
        vec![
            (
                "/p",
                WorkflowNode::Parallel(ParallelNode {
                    branch_labels: vec!["a".to_string(), "b".to_string()],
                }),
            ),
            ("/a", leaf()),
            ("/b", leaf()),
            (
                "/j",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::Any(JoinAnyPolicy {}),
                    source_node_ids: vec!["a".to_string(), "b".to_string()],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/p", "a", "/a", "in"),
            ("/p", "b", "/b", "in"),
            ("/a", "out", "/j", "in"),
            ("/b", "out", "/j", "in"),
            ("/j", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"seed": 11});
    let clock = ManualClock::at(1_000);

    let a_value = serde_json::json!({"from": "a"});
    let b_value = serde_json::json!({"from": "b"});
    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    activity.script("/a", vec![PollOutcome::Pending, ready(a_value)]);
    activity.script("/b", vec![ready(b_value.clone())]);
    activity.script("/out", vec![ready(serde_json::json!({"done": true}))]);

    let progress = orch
        .execute_activity(
            &p,
            "run-join",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(progress.view.run_state, RunState::Completed);

    // The journaled cohort is the FIRST delivered source.
    let events = orch.store().load_events("run-join").expect("events");
    let fired = events
        .iter()
        .find_map(|e| match &e.kind {
            RunEventKind::JoinFired { sources } => Some(sources.clone()),
            _ => None,
        })
        .expect("join_fired journaled");
    assert_eq!(fired, vec!["b".to_string()]);

    // `/out` consumed b's value even though `a` (canonically first)
    // completed BEFORE `/out` dispatched.
    let mut expected_input = BTreeMap::new();
    expected_input.insert(
        "in".to_string(),
        canonical_string(&b_value).expect("canonical"),
    );
    let expected_hash = step_request_hash(
        &StepRequest::new("/out".to_string(), "scripted", expected_input).expect("request"),
    )
    .expect("hash");
    let out_done = orch
        .store()
        .completed_effect("run-join", "/out")
        .expect("read")
        .expect("completed");
    assert_eq!(out_done.request_hash, expected_hash);
    assert_eq!(
        count_kind(orch.store(), "run-join", "/a", "effect_completed"),
        1,
        "the late source still completed (its result is kept, not consumed)"
    );
}

// ── GC over Phase J/K child state (latent FK gap closed) ─────────────────

/// GC purges a run that holds timer AND attempt rows — the child-state
/// deletes the purge was missing since J (a completed run with a timer
/// row tripped the runs FOREIGN KEY; surfaced by the K attempts table).
#[test]
fn gc_purges_run_with_timers_and_attempts() {
    let c = ctx();
    let mut p = single_leaf_plan();
    // A run-wide timeout policy schedules a durable timer row at start.
    p.cancellation_policies.insert(
        "".to_string(),
        CancellationPolicy {
            cancel_on_budget_exhausted: false,
            cancel_on_timeout: true,
            allow_host_operator_cancel: true,
        },
    );
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity = FencedFileActivity::new(&c.log);
    let done = orch
        .execute_activity(
            &p,
            "run-gc",
            &serde_json::json!({"seed": 12}),
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(done.view.run_state, RunState::Completed);

    let outcome = orch
        .store()
        .gc(
            10_000_000,
            &RetentionPolicy {
                min_age_ms: 1,
                keep_last: 0,
            },
        )
        .expect("gc must not trip the child-state foreign keys");
    assert_eq!(outcome.completed_purged, 1);
    assert_eq!(orch.store().run_status("run-gc").expect("status"), None);
}

// ── Contract mirrors ─────────────────────────────────────────────────────

/// The compiler's label bound and the engine's deposit-door storage bound
/// are the SAME constant value — the alignment the J carry asked for; a
/// drift here re-opens the valid-but-unresumable plan class.
#[test]
fn decision_label_bound_mirrors_store_bound() {
    assert_eq!(MAX_DECISION_LABEL_BYTES, MAX_SIGNAL_PAYLOAD_BYTES);
}

/// Confinement, LEAF symlink: an existing symlink AT THE FINAL component
/// is refused (its parent canonicalises inside the tree, but following the
/// leaf link would write outside). Unix-only because creating a symlink on
/// Windows needs a privilege CI rarely grants; the guard itself
/// (`symlink_metadata` on the leaf) is cross-platform. Mutation killed:
/// dropping the leaf `symlink_metadata` check (the write would follow the
/// link outside the worktree).
#[cfg(unix)]
#[test]
fn fs_effect_leaf_symlink_refused() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let c = ctx();
    let root = c.dir.path().join("wt");
    std::fs::create_dir(&root).expect("mkdir");
    let outside = c.dir.path().join("outside.txt");
    // A leaf symlink INSIDE the tree pointing OUTSIDE it.
    std::os::unix::fs::symlink(&outside, root.join("link.txt")).expect("symlink");

    let state = WorktreeState {
        head: "a".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let p = single_leaf_plan();
    let input = serde_json::json!({"path": "link.txt", "content": "escape"});
    let clock = ManualClock::at(1_000);

    let mut orch = orchestrator(&c);
    let mut activity =
        WorktreeFsWriteActivity::new(&root, Box::new(FixedWorktreeState(state.clone())), state);
    let progress = orch
        .execute_activity(
            &p,
            "run-fs-link",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    assert!(
        !outside.exists(),
        "a leaf symlink must never be written through"
    );
}

/// Confinement, NON-REGULAR leaf: a special file (here a Unix socket) at
/// the final component is refused AT THE GATE — else the
/// idempotent-collapse `fs::read` would block on it or turn the effect
/// into IPC. Unix-only (creating a socket file). Mutation killed: the
/// `Ok(meta) if meta.is_file()` narrowing — under a bare `Ok(_) => {}` the
/// socket still fails, but LATER and DIFFERENTLY (an Io error at
/// `File::create`, journaled `retryable: true`); the gate's refusal is
/// `Refused`, journaled `retryable: false` — the assertion reads the
/// flag, which discriminates the two paths.
#[cfg(unix)]
#[test]
fn fs_effect_non_regular_leaf_refused() {
    use sbfb_workflow_engine::fs_effect::{
        FixedWorktreeState, WorktreeFsWriteActivity, WorktreeState,
    };
    let c = ctx();
    let root = c.dir.path().join("wt");
    std::fs::create_dir(&root).expect("mkdir");
    // A Unix socket file INSIDE the tree at the target leaf.
    let _listener = std::os::unix::net::UnixListener::bind(root.join("sock")).expect("bind socket");

    let state = WorktreeState {
        head: "a".repeat(40),
        index_digest: "1".repeat(64),
        worktree_digest: "2".repeat(64),
    };
    let p = single_leaf_plan();
    let input = serde_json::json!({"path": "sock", "content": "x"});
    let clock = ManualClock::at(1_000);
    let mut orch = orchestrator(&c);
    let mut activity =
        WorktreeFsWriteActivity::new(&root, Box::new(FixedWorktreeState(state.clone())), state);
    let progress = orch
        .execute_activity(
            &p,
            "run-fs-sock",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("refusal settles the run");
    assert_eq!(progress.view.run_state, RunState::Failed);
    // The discriminating assertion: the GATE refused (non-retryable) —
    // the failure did not come from a later Io stumble (retryable).
    let failures: Vec<bool> = orch
        .store()
        .load_events("run-fs-sock")
        .expect("events")
        .iter()
        .filter_map(|e| match &e.kind {
            RunEventKind::EffectAttemptFailed { retryable, .. } => Some(*retryable),
            _ => None,
        })
        .collect();
    assert_eq!(
        failures,
        vec![false],
        "the refusal must come from the confinement gate (non-retryable)"
    );
}

// ── Crash recovery: orphan attempt of a previous incarnation ─────────────

/// The RESUME path (§12.5): a NEW activity incarnation that does not know
/// the in-flight key is a previous incarnation's orphan — reaped by the
/// journaled handle, then decided by the JOURNALED idempotency claim. An
/// idempotent orphan is a retryable failure (attempt narrowed, re-tried
/// under the same key); a non-idempotent one is UNKNOWABLE. This is the
/// crash-recovery leg of K.4 that the single-instance tests never exercise.
/// Mutation killed: skipping `reap_orphan` (the `reap:` call vanishes) or
/// deciding a non-idempotent orphan as retryable (it would loop, not
/// close).
#[test]
fn orphan_attempt_reaped_and_decided_by_journaled_claim() {
    let clock = ManualClock::at(1_000);

    // Idempotent orphan → retryable failure, reap observed.
    {
        let c = ctx();
        let p = single_leaf_plan();
        let input = serde_json::json!({"seed": 20});
        let mut orch = orchestrator(&c);
        let mut first = ScriptedActivity::new("scripted", true);
        first.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
        orch.execute_activity(&p, "run-orph-i", &input, &mut first, &clock, &mut NoSignals)
            .expect("in-flight");

        // A fresh incarnation that does NOT know the key.
        let mut reborn = ScriptedActivity::new("scripted", true);
        orch.execute_activity(
            &p,
            "run-orph-i",
            &input,
            &mut reborn,
            &clock,
            &mut NoSignals,
        )
        .expect("orphan resume");
        assert!(
            reborn.calls.iter().any(|c| c.starts_with("reap:/a")),
            "the orphan was reaped: {:?}",
            reborn.calls
        );
        assert_eq!(
            count_kind(orch.store(), "run-orph-i", "/a", "effect_attempt_failed"),
            1,
            "an idempotent orphan is a retryable failure"
        );
    }

    // Non-idempotent orphan → unknowable, reap observed, run terminal.
    {
        let c = ctx();
        let p = single_leaf_plan();
        let input = serde_json::json!({"seed": 21});
        let mut orch = orchestrator(&c);
        let mut first = ScriptedActivity::new("scripted", false);
        first.script("/a", vec![PollOutcome::Pending, PollOutcome::Pending]);
        orch.execute_activity(&p, "run-orph-n", &input, &mut first, &clock, &mut NoSignals)
            .expect("in-flight");

        let mut reborn = ScriptedActivity::new("scripted", false);
        let err = orch
            .execute_activity(
                &p,
                "run-orph-n",
                &input,
                &mut reborn,
                &clock,
                &mut NoSignals,
            )
            .expect_err("a non-idempotent orphan is unknowable");
        assert!(
            matches!(
                &err,
                OrchestratorError::Step(e) if matches!(**e, EngineError::OutcomeUnknown { .. })
            ),
            "got {err:?}"
        );
        assert!(reborn.calls.iter().any(|c| c.starts_with("reap:/a")));
        assert_eq!(
            orch.store().terminal_reason("run-orph-n").expect("reason"),
            Some("outcome_unknown".to_string())
        );
        assert_eq!(
            count_kind(orch.store(), "run-orph-n", "/a", "effect_attempt_failed"),
            0,
            "an unknowable orphan is not a failure"
        );
    }
}

// ── Reducer causal guards — the two remaining shapes ─────────────────────

/// An attempt failure with no intent before it is refused at reduction —
/// symmetric with `completion_without_intent_refused_by_reducer`.
#[test]
fn attempt_failure_without_intent_refused_by_reducer() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-af", "digest").expect("run");
    store
        .append_raw_node_event_for_harness(
            "run-af",
            "/a",
            "effect_attempt_failed",
            "{\"kind\":\"effect_attempt_failed\",\"effect_id\":\"e\",\"attempt\":1,\"retryable\":true}",
        )
        .expect("raw append");
    let events = store.load_events("run-af").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("causally invalid");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("attempt failure without a matching prior intent")
        ),
        "got {err:?}"
    );
}

/// A park (`node_outcome_unknown`) with no intent before it is refused at
/// reduction — the last of the four causal shapes.
#[test]
fn park_without_intent_refused_by_reducer() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-pk", "digest").expect("run");
    store
        .append_raw_node_event_for_harness(
            "run-pk",
            "/a",
            "node_outcome_unknown",
            "{\"kind\":\"node_outcome_unknown\",\"effect_id\":\"e\"}",
        )
        .expect("raw append");
    let events = store.load_events("run-pk").expect("events");
    let err = reduce(&single_leaf_plan(), &events).expect_err("causally invalid");
    assert!(
        matches!(
            &err,
            ReduceError::CausalOrder { reason, .. } if reason.contains("park without a matching prior intent")
        ),
        "got {err:?}"
    );
}

// ── JoinFired — a MULTI-SOURCE frozen cohort (quorum) ────────────────────

/// A `quorum(2)` join freezes a TWO-source cohort at fire time, and value
/// resolution collects exactly those two — the multi-source counterpart of
/// `join_fired_freezes_cohort` (which froze a single `any` winner).
#[test]
fn join_quorum_freezes_multi_source_cohort() {
    use sbfb_workflow_core::graph::JoinQuorumPolicy;
    let c = ctx();
    let p = plan(
        &[("", &["/p"])],
        vec![
            (
                "/p",
                WorkflowNode::Parallel(ParallelNode {
                    branch_labels: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                }),
            ),
            ("/a", leaf()),
            ("/b", leaf()),
            ("/c", leaf()),
            (
                "/j",
                WorkflowNode::Join(JoinNode {
                    policy: JoinPolicy::Quorum(JoinQuorumPolicy {
                        quorum: NonZeroU32::new(2).expect("nz"),
                    }),
                    source_node_ids: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                }),
            ),
            ("/out", out_node("result")),
        ],
        &[
            ("/p", "a", "/a", "in"),
            ("/p", "b", "/b", "in"),
            ("/p", "c", "/c", "in"),
            ("/a", "out", "/j", "in"),
            ("/b", "out", "/j", "in"),
            ("/c", "out", "/j", "in"),
            ("/j", "out", "/out", "in"),
        ],
    );
    let input = serde_json::json!({"seed": 22});
    let clock = ManualClock::at(1_000);

    let a_value = serde_json::json!({"from": "a"});
    let b_value = serde_json::json!({"from": "b"});
    let c_value = serde_json::json!({"from": "c"});
    let mut orch = orchestrator(&c);
    let mut activity = ScriptedActivity::new("scripted", true);
    // `/b` and `/c` deliver first → quorum of 2 met by {b, c}, cohort
    // frozen; `/a` delivers LATER but can no longer widen the cohort (it
    // still completes, so the run settles).
    activity.script("/a", vec![PollOutcome::Pending, ready(a_value)]);
    activity.script("/b", vec![ready(b_value.clone())]);
    activity.script("/c", vec![ready(c_value.clone())]);
    activity.script("/out", vec![ready(serde_json::json!({"done": true}))]);

    let progress = orch
        .execute_activity(
            &p,
            "run-quorum",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("run");
    assert_eq!(progress.view.run_state, RunState::Completed);

    let events = orch.store().load_events("run-quorum").expect("events");
    let fired = events
        .iter()
        .find_map(|e| match &e.kind {
            RunEventKind::JoinFired { sources } => Some(sources.clone()),
            _ => None,
        })
        .expect("join_fired journaled");
    assert_eq!(fired, vec!["b".to_string(), "c".to_string()], "cohort of 2");

    // `/out` consumed exactly {b, c} — never `/a`.
    let mut collected = BTreeMap::new();
    collected.insert("b".to_string(), b_value);
    collected.insert("c".to_string(), c_value);
    let mut expected_input = BTreeMap::new();
    expected_input.insert(
        "in".to_string(),
        canonical_string(&serde_json::to_value(&collected).expect("value")).expect("canonical"),
    );
    let expected_hash = step_request_hash(
        &StepRequest::new("/out".to_string(), "scripted", expected_input).expect("request"),
    )
    .expect("hash");
    let out_done = orch
        .store()
        .completed_effect("run-quorum", "/out")
        .expect("read")
        .expect("completed");
    assert_eq!(out_done.request_hash, expected_hash);
}

// ── G-FS-PROCESS-EFFECT — descendant process cleanup ─────────────────────

/// GRANDCHILD (ignored; run only via the process activity): heartbeats an
/// fsync'd file forever — the witness whose CESSATION proves the tree
/// died and whose GROWTH proves it was alive (anti-vacuity).
#[test]
#[ignore]
fn effects_grandchild_heartbeats() {
    use std::io::Write;
    let heartbeat = heartbeat_path_from_env();
    let mut n: u64 = 0;
    loop {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&heartbeat)
            .expect("open heartbeat");
        writeln!(f, "beat {n}").expect("write");
        f.sync_all().expect("sync");
        n += 1;
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

/// CHILD (ignored; spawned by `ProcessActivity`): spawns the grandchild
/// then blocks on it — a two-level tree whose leaves outlive any naive
/// single-pid kill.
#[test]
#[ignore]
fn effects_child_spawns_grandchild_and_sleeps() {
    let exe = std::env::current_exe().expect("current exe");
    let mut grandchild = std::process::Command::new(exe)
        .args([
            "effects_grandchild_heartbeats",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .spawn()
        .expect("spawn grandchild");
    let _ = grandchild.wait();
}

/// The minimal environment a TEST grants a spawned child. The test (not
/// the crate) reads the ambient env — the crate's charter forbids IT from
/// doing so, so the host injects the map. `SYSTEMROOT` lets Windows load
/// the base DLLs; `PATH`/`HOME`/`TMPDIR` cover the loader on Unix.
fn host_env() -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "SYSTEMROOT",
        "SystemRoot",
        "TEMP",
        "TMP",
        "HOME",
        "TMPDIR",
    ] {
        if let Ok(value) = std::env::var(key) {
            env.insert(key.to_string(), value);
        }
    }
    env
}

fn heartbeat_path_from_env() -> PathBuf {
    let raw = std::env::var(sbfb_workflow_engine::process::EFFECT_INPUT_ENV).expect("input env");
    let ports: BTreeMap<String, String> = serde_json::from_str(&raw).expect("ports");
    let inner: serde_json::Value =
        serde_json::from_str(ports.get("in").expect("in port")).expect("inner");
    PathBuf::from(
        inner
            .get("heartbeat")
            .and_then(serde_json::Value::as_str)
            .expect("heartbeat path"),
    )
}

/// Bounded witness poll: the heartbeat file must GROW (proof of life).
fn wait_heartbeat_growth(path: &std::path::Path) {
    let size = |p: &std::path::Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    let first = size(path);
    for _ in 0..600 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        if size(path) > first && size(path) > 0 {
            return;
        }
    }
    panic!("grandchild never proved life (heartbeat did not grow)");
}

/// Bounded witness poll: the heartbeat must STOP (two stable reads
/// 500ms apart) — never a raced PID check.
fn assert_heartbeat_stops(path: &std::path::Path) {
    let size = |p: &std::path::Path| std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    for _ in 0..30 {
        let before = size(path);
        std::thread::sleep(std::time::Duration::from_millis(500));
        if size(path) == before {
            return;
        }
    }
    panic!("descendant survived the tree kill (heartbeat still growing)");
}

fn descendant_cleanup_body() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let heartbeat = c.dir.path().join("heartbeat.log");
    let p = single_leaf_plan();
    let input = serde_json::json!({"heartbeat": heartbeat.to_string_lossy()});
    let clock = ManualClock::at(1_000);

    let exe = std::env::current_exe().expect("current exe");
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc",
        exe,
        c.dir.path(),
        vec![
            "effects_child_spawns_grandchild_and_sleeps".to_string(),
            "--exact".to_string(),
            "--ignored".to_string(),
            "--nocapture".to_string(),
            "--test-threads=1".to_string(),
        ],
        false,
        false,
        host_env(),
    );

    let progress = orch
        .execute_activity(
            &p,
            "run-desc",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("spawn drive");
    assert_eq!(
        progress.view.node_states.get("/a").copied(),
        Some(NodeState::Running)
    );

    // Anti-vacuity: the grandchild must be ALIVE before the kill.
    wait_heartbeat_growth(&heartbeat);

    assert!(orch.request_cancel("run-desc", &clock).expect("cancel"));
    let cancelled = orch
        .execute_activity(
            &p,
            "run-desc",
            &input,
            &mut activity,
            &clock,
            &mut NoSignals,
        )
        .expect("cancel drain kills the tree");
    assert_eq!(cancelled.view.run_state, RunState::Cancelled);

    // The whole tree is dead: the heartbeat stops.
    assert_heartbeat_stops(&heartbeat);
}

/// The Windows arm of K.3: a two-level process tree dies with the
/// kill-on-close Job Object — grandchild included. Mutation killed:
/// killing only the direct child (the grandchild's heartbeat would keep
/// growing).
#[cfg(windows)]
#[test]
fn descendant_process_cleanup_windows() {
    descendant_cleanup_body();
}

/// The Unix arm of K.3: the process group dies under `kill(-pgid)` —
/// grandchild included.
#[cfg(unix)]
#[test]
fn descendant_process_cleanup_unix() {
    descendant_cleanup_body();
}

/// A no-op child (ignored; run only via the process activity): returns
/// immediately, exit code 0 — the happy path of the real process adapter.
#[test]
#[ignore]
fn effects_exit_zero() {}

/// The process adapter's SUCCESS branch: a child that exits 0 confirms the
/// effect (`PollOutcome::Ready`, receipt facts carry `exit_code = 0`) and
/// the run completes. The descendant-cleanup tests only exercise the
/// cancel→kill path; this proves the everyday success path of a real
/// process effect.
#[test]
fn process_effect_exit_zero_completes() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"noop": true});
    let clock = ManualClock::at(1_000);
    let exe = std::env::current_exe().expect("current exe");
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc-ok",
        exe,
        c.dir.path(),
        vec![
            "effects_exit_zero".to_string(),
            "--exact".to_string(),
            "--ignored".to_string(),
            "--test-threads=1".to_string(),
        ],
        true,
        false,
        host_env(),
    );

    // The manual clock stays at 1_000 (deadline 601_000), so the attempt
    // never times out; drive until the child exits (fast, but not instant).
    let mut completed = false;
    for _ in 0..300 {
        let progress = orch
            .execute_activity(&p, "run-ok", &input, &mut activity, &clock, &mut NoSignals)
            .expect("drive");
        if progress.view.run_state == RunState::Completed {
            completed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(completed, "the process effect never completed");

    let row = orch
        .store()
        .completed_effect("run-ok", "/a")
        .expect("read")
        .expect("completed");
    let receipt = ActivityReceipt::parse(&row.receipt).expect("structured receipt");
    assert_eq!(
        receipt.facts.get("exit_code").map(String::as_str),
        Some("0")
    );
}

/// A child (ignored; run only via the process activity) that writes a
/// marker file at a RELATIVE path — it lands wherever the child's working
/// directory actually is.
#[test]
#[ignore]
fn effects_child_writes_cwd_marker() {
    std::fs::write("cwd_marker.txt", b"here").expect("write marker");
}

/// The spawn RUNS in the constructor-fixed working directory (ex-Codex r4
/// K-PROC-CWD-SCOPE-03): a child writing a RELATIVE path lands in the
/// activity's `cwd`, not in the runtime's ambient one. Mutation killed:
/// dropping the `current_dir` call in `ProcessActivity::start` (the marker
/// lands in the test process's cwd and this assertion turns red).
#[test]
fn process_effect_runs_in_fixed_cwd() {
    use sbfb_workflow_engine::process::ProcessActivity;
    let c = ctx();
    let p = single_leaf_plan();
    let input = serde_json::json!({"marker": true});
    let clock = ManualClock::at(1_000);
    let exe = std::env::current_exe().expect("current exe");
    let mut orch = orchestrator(&c);
    let mut activity = ProcessActivity::new(
        "proc-cwd",
        exe,
        c.dir.path(),
        vec![
            "effects_child_writes_cwd_marker".to_string(),
            "--exact".to_string(),
            "--ignored".to_string(),
            "--test-threads=1".to_string(),
        ],
        true,
        false,
        host_env(),
    );

    let mut completed = false;
    for _ in 0..300 {
        let progress = orch
            .execute_activity(&p, "run-cwd", &input, &mut activity, &clock, &mut NoSignals)
            .expect("drive");
        if progress.view.run_state == RunState::Completed {
            completed = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert!(completed, "the cwd-marker process effect never completed");
    let marker = c.dir.path().join("cwd_marker.txt");
    assert!(
        marker.is_file(),
        "the relative marker must land in the FIXED cwd, found nothing at {}",
        marker.display()
    );
}
