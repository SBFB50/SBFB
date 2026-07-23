// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 83 Phase J — deterministic end-to-end execution of the four REAL
//! compiled goldens (J.2 "fixtures", preflight DÉC-1).
//!
//! Each golden runs to `RunCompleted` under a DECISION SCRIPT derived by
//! INTROSPECTING the plan itself — no hand-maintained per-golden constants
//! that would silently rot when a golden evolves:
//!
//! - `switch` discriminants and `classifier` labels are PRE-SEEDED in the
//!   run input (the echo activity passes values through unchanged, so a
//!   seeded field reaches its router);
//! - `repeat` until-flags are seeded true (early stop; the exact
//!   iteration-count arithmetic is `orchestrator_bounded_repeat`'s job);
//! - `map` collections are seeded within the plan's structural bound;
//! - `wait.signal` deposits and `human.review` decisions come from a
//!   [`ScriptedSignals`] script built from the plan's wait/review nodes.
//!
//! The goldens are read from the WORKSPACE-RELATIVE core crate path
//! (committed fixtures, `CARGO_MANIFEST_DIR` anchored — the same
//! intra-workspace posture as the shared lock file). Reading them through
//! a core loader was considered and rejected: `golden_store` is core's
//! test support (`pub(crate)`), and widening core's API for a fixture
//! path is more surface than the path costs.

use std::path::PathBuf;

use sbfb_workflow_core::compile::CompiledPlan;
use sbfb_workflow_core::graph::WorkflowNode;
use sbfb_workflow_engine::clock::ManualClock;
use sbfb_workflow_engine::echo::FileEchoTarget;
use sbfb_workflow_engine::orchestrator::{
    CLASSIFIER_LABEL_FIELD, Orchestrator, OrchestratorConfig,
};
use sbfb_workflow_engine::reducer::{RunState, reduce};
use sbfb_workflow_engine::signal::ScriptedSignals;
use sbfb_workflow_engine::store::RunStore;

fn golden_plan(name: &str) -> CompiledPlan {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../sbfb-workflow-core/src")
        .join(format!("{name}.compiled.golden.json"));
    let raw =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}

/// Build the seeded input + decision script by introspecting the plan.
fn seeded(plan: &CompiledPlan) -> (serde_json::Value, ScriptedSignals) {
    let mut seed = serde_json::Map::new();
    for instance in plan.node_instances.values() {
        match &instance.node {
            WorkflowNode::Switch(n) => {
                seed.insert(
                    n.discriminant_field.clone(),
                    serde_json::json!(n.branch_labels[0]),
                );
            }
            WorkflowNode::Classifier(n) => {
                seed.insert(
                    CLASSIFIER_LABEL_FIELD.to_string(),
                    serde_json::json!(n.branch_labels[0]),
                );
            }
            WorkflowNode::Repeat(n) => {
                seed.insert(n.until_output_flag.clone(), serde_json::json!(true));
            }
            _ => {}
        }
    }
    let base = serde_json::Value::Object(seed.clone());
    for instance in plan.node_instances.values() {
        if let WorkflowNode::Map(n) = &instance.node {
            let count = 2.min(n.max_cardinality.get()) as usize;
            seed.insert(
                n.collection_field.clone(),
                serde_json::Value::Array(vec![base.clone(); count]),
            );
        }
    }

    let mut script = ScriptedSignals::new();
    for (id, instance) in &plan.node_instances {
        match &instance.node {
            WorkflowNode::WaitSignal(_) => {
                script = script.signal(id, base.clone());
            }
            WorkflowNode::HumanReview(n) => {
                script = script.decision(id, &n.decision_labels[0], base.clone());
            }
            _ => {}
        }
    }
    (serde_json::Value::Object(seed), script)
}

/// Drive one golden to completion, then assert the replay invariant on the
/// REAL plan: a second execute serves everything from the journal without
/// growing it, and the reduction is a pure function of the journal.
fn drive(name: &str) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("run.sqlite3");
    let log = dir.path().join("effects.log");
    let plan = golden_plan(name);
    let (input, mut script) = seeded(&plan);
    let run_id = format!("golden-{name}");

    let store = RunStore::open(&db).expect("open");
    let mut orch = Orchestrator::new(store, OrchestratorConfig::new("golden-writer"));
    let clock = ManualClock::at(1_000);
    let mut target = FileEchoTarget::idempotent(&log);

    // The script may need several passes (a wait can only be answered once
    // it is OPEN); the orchestrator returns at each park, the script is
    // drained across calls. Bound the passes by the wait/review count + 2.
    let waits = plan
        .node_instances
        .values()
        .filter(|i| {
            matches!(
                i.node,
                WorkflowNode::WaitSignal(_) | WorkflowNode::HumanReview(_)
            )
        })
        .count();
    let mut last = None;
    for _pass in 0..waits + 2 {
        let progress = orch
            .execute(&plan, &run_id, &input, &mut target, &clock, &mut script)
            .unwrap_or_else(|e| panic!("golden {name}: {e}"));
        let state = progress.view.run_state;
        last = Some(progress);
        if state == RunState::Completed {
            break;
        }
    }
    let last = last.expect("at least one pass");
    assert_eq!(
        last.view.run_state,
        RunState::Completed,
        "golden {name} must complete; final view: {:?}",
        last.view
    );

    // Replay: nothing re-executes, the journal does not grow, and the
    // reduction is pure.
    let count_before = orch.store().event_count(&run_id).expect("count");
    let replay = orch
        .execute(&plan, &run_id, &input, &mut target, &clock, &mut script)
        .expect("replay");
    assert_eq!(
        replay.steps_executed, 0,
        "golden {name}: replay re-executed"
    );
    assert_eq!(
        orch.store().event_count(&run_id).expect("count"),
        count_before,
        "golden {name}: replay grew the journal"
    );
    let events = orch.store().load_events(&run_id).expect("events");
    assert_eq!(
        reduce(&plan, &events).expect("reduce a"),
        reduce(&plan, &events).expect("reduce b"),
        "golden {name}: reduction must be pure"
    );
}

#[test]
fn orchestrator_golden_factory_runs_to_completion() {
    drive("factory");
}

#[test]
fn orchestrator_golden_ideas_runs_to_completion() {
    drive("ideas");
}

#[test]
fn orchestrator_golden_conception_runs_to_completion() {
    drive("conception");
}

#[test]
fn orchestrator_golden_babel_runs_to_completion() {
    drive("babel");
}
