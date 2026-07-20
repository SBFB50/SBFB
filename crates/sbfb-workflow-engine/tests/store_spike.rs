// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 83 Phase E — the durable-execution spike harness.
//!
//! Every guarantee below has TWO arms: the guaranteed arm, and a RED CONTROL
//! arm where the guarantee is switched off and the test asserts the bad state
//! IS observed. Both arms are assertions inside the suite, so `exit 0` means
//! "the guarantee holds AND removing it breaks something" — a green suite can
//! no longer be vacuous.
//!
//! Naming: every test name starts with `store_` so the `G-STORE` gate command
//! (`cargo nextest run -p sbfb-workflow-engine store`) matches at least one
//! test — a zero-match nextest filter exits 4, i.e. a silently RED gate. No
//! test name contains `effect`, so the `G-EFFECT` gate command stays owned by
//! its own phase instead of being greened prematurely by this one.
//!
//! The tests live in `tests/` rather than in `src/`: the out-of-process arm
//! reads its parameters from the environment, and the crate charter keeps
//! `src/` free of any environment read or path literal.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use sbfb_workflow_engine::echo::{
    EffectTarget, FileEchoTarget, read_side_outputs, side_output_count,
};
use sbfb_workflow_engine::engine::{EngineError, RunSpec, SpikeEngine, StepSpec};
use sbfb_workflow_engine::event::{RunEventKind, TerminalReason};
use sbfb_workflow_engine::fault::{FaultAction, FaultHook, FaultPoint, NoFaults, SpikeKnobs};
use sbfb_workflow_engine::hash::{
    HashError, IdempotencyKey, StepRequest, canonical_digest, derive_effect_id, step_request_hash,
};
use sbfb_workflow_engine::store::{RunStatus, RunStore};

/// This very file, used by the artifact test to prove that every scenario the
/// phase artifact claims corresponds to a test function that really exists.
const SELF_SOURCE: &str = include_str!("store_spike.rs");

/// Exit code the out-of-process writer uses to die mid-run. Chosen outside
/// the libtest range (0 = pass, 101 = panic) so the parent cannot mistake a
/// panicking child for a killed one.
const CHILD_EXIT_CODE: i32 = 87;

const RUN_ID: &str = "run-spike-e";
const NODE: &str = "root/echo#1";

// ── harness plumbing ──────────────────────────────────────────────────────

struct Ctx {
    _dir: tempfile::TempDir,
    db: PathBuf,
    log: PathBuf,
}

fn ctx() -> Ctx {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("run.sqlite3");
    let log = dir.path().join("side-outputs.tsv");
    Ctx { _dir: dir, db, log }
}

fn input(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

fn one_step_run(pairs: &[(&str, &str)]) -> RunSpec {
    RunSpec {
        run_id: RUN_ID.to_string(),
        steps: vec![StepSpec {
            node_instance_id: NODE.to_string(),
            input: input(pairs),
        }],
    }
}

fn engine(db: &Path, knobs: SpikeKnobs) -> SpikeEngine {
    SpikeEngine::new(RunStore::open(db).expect("open store"), knobs)
}

/// The idempotency key the engine derives for a step — recomputed here so the
/// harness can count side outputs from another process.
fn key_for(run_id: &str, node: &str, pairs: &[(&str, &str)]) -> IdempotencyKey {
    let request = StepRequest::new(node, "echo", input(pairs)).expect("valid request");
    let hash = step_request_hash(&request).expect("hash");
    IdempotencyKey::new(run_id, node, derive_effect_id(run_id, node, &hash)).expect("valid key")
}

/// The side-output line an UNVALIDATED encoder would produce — the exact
/// shape `IdempotencyKey::as_line` had before the components were checked.
/// Test-only, and the only honest way to show what the validation gate buys.
fn raw_key_line(run_id: &str, node_instance_id: &str, effect_id: &str) -> String {
    format!("{run_id}\u{1f}{node_instance_id}\u{1f}{effect_id}")
}

/// Halts the run at one frontier and counts what it saw.
struct HaltAt {
    point: FaultPoint,
    seen: RefCell<Vec<FaultPoint>>,
}

impl HaltAt {
    fn new(point: FaultPoint) -> Self {
        Self {
            point,
            seen: RefCell::new(Vec::new()),
        }
    }

    fn hits(&self, point: FaultPoint) -> usize {
        self.seen.borrow().iter().filter(|p| **p == point).count()
    }
}

impl FaultHook for HaltAt {
    fn hit(&self, point: FaultPoint) -> FaultAction {
        self.seen.borrow_mut().push(point);
        if point == self.point {
            FaultAction::Halt
        } else {
            FaultAction::Continue
        }
    }
}

/// Kills the WHOLE PROCESS at one frontier, without unwinding and without
/// closing the SQLite connection. Used by the out-of-process arm.
struct ExitAt {
    point: FaultPoint,
}

impl FaultHook for ExitAt {
    fn hit(&self, point: FaultPoint) -> FaultAction {
        if point == self.point {
            std::process::exit(CHILD_EXIT_CODE);
        }
        FaultAction::Continue
    }
}

fn expect_halt(result: Result<sbfb_workflow_engine::RunOutcome, EngineError>, at: FaultPoint) {
    match result {
        Err(EngineError::Halted { at: got }) => assert_eq!(got, at, "halted at the wrong frontier"),
        other => panic!("expected a halt at {at:?}, got {other:?}"),
    }
}

// ── (1) journal + outbox atomicity ────────────────────────────────────────

/// GUARANTEED: the intent entry and its outbox row are one transaction, so no
/// crash frontier can observe one without the other.
///
/// RED CONTROL: with `without_atomic_outbox` the two writes are split, and the
/// test asserts an ORPHAN outbox row IS observed — proving the guaranteed arm
/// is not asserting a state that is unreachable anyway.
#[test]
fn store_journal_outbox_atomic() {
    // THE discriminating arm: crash INSIDE the guaranteed transaction,
    // between the journal append and the outbox insert. Both writes must be
    // rolled back together.
    //
    // Without this arm the test could not tell one transaction from two: the
    // guaranteed path simply had no window to crash in, so splitting
    // `record_intent_atomic` into two transactions left the whole suite green
    // (measured — this arm exists because of that measurement).
    let inside = ctx();
    let mut inside_eng = engine(&inside.db, SpikeKnobs::guaranteed());
    let mut inside_target = FileEchoTarget::idempotent(&inside.log);
    expect_halt(
        inside_eng.execute(
            &one_step_run(&[("a", "1")]),
            &mut inside_target,
            &HaltAt::new(FaultPoint::InsideIntentTransaction),
        ),
        FaultPoint::InsideIntentTransaction,
    );
    assert_eq!(
        inside_eng.store().event_count(RUN_ID).expect("events"),
        1,
        "the intent entry must be rolled back with the outbox row — only run_started survives"
    );
    assert!(
        inside_eng
            .store()
            .pending_effect(RUN_ID, NODE)
            .expect("pending")
            .is_none(),
        "no outbox row may survive a rollback of the intent transaction"
    );
    assert_eq!(
        inside_eng
            .store()
            .orphan_outbox_rows(RUN_ID)
            .expect("orphans"),
        0
    );
    assert_eq!(
        read_side_outputs(&inside.log).expect("side outputs").len(),
        0
    );

    // Guaranteed arm: crash right after the intent is durable.
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    let hook = HaltAt::new(FaultPoint::AfterIntentBeforeEffect);

    expect_halt(
        eng.execute(&one_step_run(&[("a", "1")]), &mut target, &hook),
        FaultPoint::AfterIntentBeforeEffect,
    );

    assert_eq!(
        eng.store().orphan_outbox_rows(RUN_ID).expect("orphans"),
        0,
        "atomic path must never leave an outbox row without its journal entry"
    );
    assert_eq!(
        eng.store().event_count(RUN_ID).expect("events"),
        2,
        "run_started + effect_intended"
    );
    assert!(
        eng.store()
            .pending_effect(RUN_ID, NODE)
            .expect("pending")
            .is_some(),
        "the intent must be durable before the effect is attempted"
    );
    assert_eq!(
        hook.hits(FaultPoint::BetweenOutboxAndIntentEvent),
        0,
        "the split window must not even exist on the guaranteed path"
    );
    assert_eq!(
        read_side_outputs(&c.log).expect("side outputs").len(),
        0,
        "nothing may reach the outside before the intent is durable"
    );

    // RED CONTROL: split the two writes and crash in between.
    let red = ctx();
    let mut red_eng = engine(&red.db, SpikeKnobs::without_atomic_outbox());
    let mut red_target = FileEchoTarget::idempotent(&red.log);
    let red_hook = HaltAt::new(FaultPoint::BetweenOutboxAndIntentEvent);

    expect_halt(
        red_eng.execute(&one_step_run(&[("a", "1")]), &mut red_target, &red_hook),
        FaultPoint::BetweenOutboxAndIntentEvent,
    );

    assert_eq!(
        red_eng.store().orphan_outbox_rows(RUN_ID).expect("orphans"),
        1,
        "red control: splitting the writes MUST expose an orphan outbox row"
    );
    assert_eq!(
        red_eng.store().event_count(RUN_ID).expect("events"),
        1,
        "red control: only run_started is journaled, the effect is invisible to the run"
    );
}

// ── (2) restart without a duplicate side output ───────────────────────────

/// GUARANTEED: killing the run at any of the three frontiers and restarting
/// yields EXACTLY ONE side output for the idempotency key.
///
/// RED CONTROL: against a target that CLAIMS idempotency without honouring it,
/// the same restart at B2 produces TWO side outputs — proving the property
/// comes from the target's deduplication, exactly as the engine clause says.
#[test]
fn store_kill_restart_replay_no_duplicate_side_output() {
    let frontiers = [
        FaultPoint::AfterIntentBeforeEffect,
        FaultPoint::AfterEffectBeforeResult,
        FaultPoint::AfterResult,
    ];
    let key = key_for(RUN_ID, NODE, &[("a", "1")]);

    for frontier in frontiers {
        let c = ctx();
        {
            let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
            let mut target = FileEchoTarget::idempotent(&c.log);
            expect_halt(
                eng.execute(
                    &one_step_run(&[("a", "1")]),
                    &mut target,
                    &HaltAt::new(frontier),
                ),
                frontier,
            );
        }

        // Restart: a brand-new engine over the same store file.
        let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&c.log);
        let outcome = resumed
            .execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
            .unwrap_or_else(|e| panic!("resume after {frontier:?} failed: {e}"));

        assert_eq!(
            side_output_count(&c.log, &key).expect("count"),
            1,
            "exactly one side output after a crash at {frontier:?}"
        );
        assert_eq!(
            read_side_outputs(&c.log).expect("side outputs").len(),
            1,
            "and none under any other key: an effect id minted afresh on \
             resume would write a second line that a per-key count misses"
        );
        assert_eq!(
            resumed.store().run_status(RUN_ID).expect("status"),
            Some(RunStatus::Completed),
            "the resumed run must reach completion after {frontier:?}"
        );
        assert_eq!(
            resumed.store().orphan_outbox_rows(RUN_ID).expect("orphans"),
            0
        );
        assert_eq!(
            outcome.steps_executed + outcome.steps_memoised,
            1,
            "the single step is either replayed or memoised, never both"
        );
    }

    // RED CONTROL: a lying target at the irreducible B2 frontier.
    let red = ctx();
    {
        let mut eng = engine(&red.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::lying(&red.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut target,
                &HaltAt::new(FaultPoint::AfterEffectBeforeResult),
            ),
            FaultPoint::AfterEffectBeforeResult,
        );
    }
    let mut resumed = engine(&red.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::lying(&red.log);
    resumed
        .execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
        .expect("resume against the lying target");
    assert_eq!(
        side_output_count(&red.log, &key).expect("count"),
        2,
        "red control: without target-side deduplication the B2 frontier DOES double the side output"
    );
}

// ── (3) fail-closed on a divergent replay ─────────────────────────────────

/// GUARANTEED: a replayed step whose request hash differs from the journaled
/// one stops the run in a TERMINAL state with a typed error — no stale answer,
/// no panic, no automatic re-drive.
///
/// RED CONTROL: with `without_step_hash_verification` the same divergence is
/// swallowed and the run happily serves the OLD answer for the NEW question.
#[test]
fn store_replay_diverges_fails_closed() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    eng.execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
        .expect("first run");

    let err = eng
        .execute(&one_step_run(&[("a", "2")]), &mut target, &NoFaults)
        .expect_err("a divergent replay must fail closed");
    match err {
        EngineError::StepHashMismatch {
            ref node_instance_id,
            ref expected,
            ref got,
        } => {
            assert_eq!(node_instance_id, NODE);
            assert_ne!(expected, got, "the mismatch must carry both hashes");
        }
        other => panic!("expected StepHashMismatch, got {other:?}"),
    }
    assert_eq!(
        eng.store().run_status(RUN_ID).expect("status"),
        Some(RunStatus::Terminal)
    );
    assert_eq!(
        eng.store()
            .terminal_reason(RUN_ID)
            .expect("reason")
            .as_deref(),
        Some("step_hash_mismatch")
    );
    assert_eq!(
        side_output_count(&c.log, &key_for(RUN_ID, NODE, &[("a", "2")])).expect("count"),
        0,
        "the divergent request must never reach the target"
    );
    // A terminal run is never re-driven automatically.
    match eng.execute(&one_step_run(&[("a", "2")]), &mut target, &NoFaults) {
        Err(EngineError::RunAlreadyTerminal { .. }) => {}
        other => panic!("a terminal run must stay terminal, got {other:?}"),
    }

    // RED CONTROL: same divergence, verification off.
    let red = ctx();
    let mut red_eng = engine(&red.db, SpikeKnobs::without_step_hash_verification());
    let mut red_target = FileEchoTarget::idempotent(&red.log);
    let first = red_eng
        .execute(&one_step_run(&[("a", "1")]), &mut red_target, &NoFaults)
        .expect("first run");
    let second = red_eng
        .execute(&one_step_run(&[("a", "2")]), &mut red_target, &NoFaults)
        .expect("red control: without verification the divergent replay continues");
    assert_eq!(
        second.steps_memoised, 1,
        "red control: the divergent step is served from the journal"
    );
    assert_eq!(
        second.receipts.get(NODE),
        first.receipts.get(NODE),
        "red control: the OLD answer is returned for the NEW question"
    );
    assert_eq!(
        red_eng.store().run_status(RUN_ID).expect("status"),
        Some(RunStatus::Completed),
        "red control: the run continues instead of failing closed"
    );
}

// ── (4) the deterministic echo target ─────────────────────────────────────

/// The echo target is deterministic, deduplicates on the idempotency key, and
/// the digest primitive it shares with the engine is insensitive to map
/// insertion order and fails closed on a value JCS cannot represent.
#[test]
fn store_echo_target_deterministic() {
    let c = ctx();
    let mut target = FileEchoTarget::idempotent(&c.log);
    let key = key_for(RUN_ID, NODE, &[("a", "1")]);
    let payload = input(&[("a", "1"), ("b", "2")]);

    let first = target.apply(&key, &payload).expect("apply");
    let second = target.apply(&key, &payload).expect("re-apply");
    assert_eq!(first, second, "same input must echo the same receipt");
    assert_eq!(
        side_output_count(&c.log, &key).expect("count"),
        1,
        "re-applying the same key must not produce a second side output"
    );

    // JCS sorts object members at serialization time: two insertion orders
    // must digest identically. Meaningful under `--workspace`, where
    // `serde_json/preserve_order` is enabled by another crate in the graph.
    let mut ordered = serde_json::Map::new();
    ordered.insert("a".into(), serde_json::json!(1));
    ordered.insert("b".into(), serde_json::json!(2));
    let mut reversed = serde_json::Map::new();
    reversed.insert("b".into(), serde_json::json!(2));
    reversed.insert("a".into(), serde_json::json!(1));
    assert_eq!(
        canonical_digest("t", &serde_json::Value::Object(ordered)).expect("digest"),
        canonical_digest("t", &serde_json::Value::Object(reversed)).expect("digest"),
        "the digest must not depend on map insertion order"
    );

    // Non-finite floats are refused. MEASURED, not assumed: NESTED in a
    // struct — the only shape this crate hashes — serde_jcs 0.2.0 does NOT
    // error on them, it serializes NaN, +Infinity and -Infinity to `null`, so
    // all three collide with each other and with an explicit null. (A BARE
    // top-level float does return Err; the generalisation "the library
    // refuses non-finite floats" is what is false.)
    // The crate rejects the collapsed form itself rather than trusting the
    // library, otherwise two different step requests could share one hash and
    // the fail-closed replay check would wave the second one through.
    #[derive(serde::Serialize)]
    struct WithFloat {
        v: f64,
    }
    for (label, v) in [
        ("NaN", f64::NAN),
        ("+inf", f64::INFINITY),
        ("-inf", f64::NEG_INFINITY),
    ] {
        match canonical_digest("t", &WithFloat { v }) {
            Err(HashError::NonRepresentable { ref path }) => assert_eq!(path, "$.v"),
            other => panic!("{label} must fail closed, got {other:?}"),
        }
    }

    // The collision the rejection prevents is REAL, not hypothetical: the
    // three non-finite floats and an explicit null share one canonical form.
    // This is the red control of the guard above — remove the guard and these
    // four values digest identically.
    let collapsed = serde_json::json!({ "v": null });
    for v in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            serde_json::to_value(WithFloat { v }).expect("to_value"),
            collapsed,
            "non-finite floats collapse to null — that is why they are refused"
        );
    }
}

// ── (5) durability contract ───────────────────────────────────────────────

/// The pragma contract is the only durability evidence a crash harness cannot
/// provide: SQLite commits survive an APPLICATION crash at any `synchronous`
/// level, so `FULL` is asserted here rather than claimed in prose.
#[test]
fn store_run_pragma_durability_contract() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    assert_eq!(
        store.pragma_i64("synchronous").expect("pragma"),
        2,
        "synchronous must be FULL (2)"
    );
    assert_eq!(
        store.pragma_text("journal_mode").expect("pragma"),
        "wal",
        "journal_mode must be WAL"
    );
    assert_eq!(store.pragma_i64("foreign_keys").expect("pragma"), 1);
}

// ── (6) the irreducible frontier, honestly named ──────────────────────────

/// Against a target that does NOT declare idempotency, an unconfirmed effect
/// is reported as `outcome_unknown`: no automatic retry (which could double
/// the side output) and no invented success.
#[test]
fn store_outcome_unknown_on_non_idempotent_target() {
    let c = ctx();
    let key = key_for(RUN_ID, NODE, &[("a", "1")]);
    {
        let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::non_idempotent(&c.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut target,
                &HaltAt::new(FaultPoint::AfterEffectBeforeResult),
            ),
            FaultPoint::AfterEffectBeforeResult,
        );
    }
    assert_eq!(side_output_count(&c.log, &key).expect("count"), 1);

    let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::non_idempotent(&c.log);
    match resumed.execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults) {
        Err(EngineError::OutcomeUnknown {
            ref node_instance_id,
            ..
        }) => assert_eq!(node_instance_id, NODE),
        other => panic!("expected OutcomeUnknown, got {other:?}"),
    }
    assert_eq!(
        side_output_count(&c.log, &key).expect("count"),
        1,
        "an unconfirmed effect must NOT be retried against a non-idempotent target"
    );
    assert_eq!(
        resumed.store().run_status(RUN_ID).expect("status"),
        Some(RunStatus::Terminal)
    );
    assert_eq!(
        resumed
            .store()
            .terminal_reason(RUN_ID)
            .expect("reason")
            .as_deref(),
        Some("outcome_unknown")
    );
    let _ = TerminalReason::OutcomeUnknown;
}

// ── (7) replay does not grow the journal ──────────────────────────────────

/// Re-executing a completed run appends ZERO entries: memoisation is silent,
/// so a journal cannot grow without bound just by being replayed.
#[test]
fn store_replay_of_completed_run_appends_no_events() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    let spec = one_step_run(&[("a", "1")]);

    eng.execute(&spec, &mut target, &NoFaults).expect("first");
    let after_first = eng.store().event_count(RUN_ID).expect("count");

    let replay = eng.execute(&spec, &mut target, &NoFaults).expect("replay");
    assert_eq!(replay.steps_memoised, 1);
    assert_eq!(replay.steps_executed, 0);
    assert_eq!(
        eng.store().event_count(RUN_ID).expect("count"),
        after_first,
        "replaying a completed run must append nothing"
    );
    assert_eq!(
        read_side_outputs(&c.log).expect("side outputs").len(),
        1,
        "replay must not touch the outside"
    );
}

// ── (8) golden event stream ───────────────────────────────────────────────

/// The journal of a fixed two-step run is pinned by a committed golden.
/// STRICT (mirrors `factory_profile.rs`): a missing or empty golden FAILS
/// asking for regeneration, so deleting it cannot self-heal the drift gate.
///
/// Regenerate an intentional change with
/// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-engine store_run_event_stream_matches_golden`.
#[test]
fn store_run_event_stream_matches_golden() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    let spec = RunSpec {
        run_id: "run-golden".to_string(),
        steps: vec![
            StepSpec {
                node_instance_id: "root/echo#1".to_string(),
                input: input(&[("a", "1")]),
            },
            StepSpec {
                node_instance_id: "root/echo#2".to_string(),
                input: input(&[("b", "2")]),
            },
        ],
    };
    eng.execute(&spec, &mut target, &NoFaults).expect("run");

    let events = eng.store().load_events("run-golden").expect("events");
    assert_golden(
        "run_event.golden.json",
        &serde_json::to_value(&events).expect("serialize events"),
    );
}

/// Compare against a committed golden, STRICTLY: a missing or empty golden
/// FAILS asking for regeneration, so deleting it cannot self-heal the drift
/// gate. Only `UPDATE_SNAPSHOTS=1` ever writes.
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
        "run journal drift — the event stream of the fixed scenario changed"
    );

    // Round-trip through the TYPE, not only through `Value`: otherwise
    // `RunEvent`'s `Deserialize` and its `deny_unknown_fields` would never be
    // exercised by anything at all (nothing else in the crate deserializes a
    // `RunEvent` — `load_events` rebuilds it field by field).
    let typed: Vec<sbfb_workflow_engine::RunEvent> =
        serde_json::from_value(golden).expect("the golden must deserialize into RunEvent");
    assert_eq!(
        &serde_json::to_value(&typed).expect("re-serialize"),
        live,
        "the golden must round-trip through the typed envelope"
    );
}

/// The TERMINAL path has its own golden. The happy path alone would leave
/// the entry that carries the fail-closed diagnosis — reason plus both
/// diverging hashes — unpinned, which is the entry a post-mortem depends on.
///
/// Regenerate with
/// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-engine store_terminal_event_stream_matches_golden`.
#[test]
fn store_terminal_event_stream_matches_golden() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    let first = RunSpec {
        run_id: "run-terminal-golden".to_string(),
        steps: vec![StepSpec {
            node_instance_id: "root/echo#1".to_string(),
            input: input(&[("a", "1")]),
        }],
    };
    eng.execute(&first, &mut target, &NoFaults).expect("run");

    let diverged = RunSpec {
        steps: vec![StepSpec {
            node_instance_id: "root/echo#1".to_string(),
            input: input(&[("a", "2")]),
        }],
        ..first.clone()
    };
    eng.execute(&diverged, &mut target, &NoFaults)
        .expect_err("divergence");

    let events = eng
        .store()
        .load_events("run-terminal-golden")
        .expect("events");
    assert_golden(
        "run_event_terminal.golden.json",
        &serde_json::to_value(&events).expect("serialize"),
    );
}

// ── (9) the phase artifact cannot lie ─────────────────────────────────────

/// The GO/NO-GO artifact is machine-checked against this suite: `status=GO`
/// requires every declared scenario to name a test function that EXISTS in
/// this file and to carry both arms. Without this, `status` would be prose.
///
/// RED CONTROL: a mutated artifact (scenario renamed, red arm dropped, GO
/// without scenarios) must be REJECTED by the same validator.
#[test]
fn store_spike_artifact_matches_suite() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(".planning")
        .join("active")
        .join("sprint83_e_durable_spike.json");
    let raw =
        std::fs::read_to_string(&path).expect("sprint83_e_durable_spike.json must be committed");
    let artifact: serde_json::Value = serde_json::from_str(&raw).expect("artifact must be JSON");

    validate_artifact(&artifact, SELF_SOURCE).expect("the committed artifact must match the suite");

    // RED CONTROL 1 — a scenario naming a test that does not exist.
    let mut renamed = artifact.clone();
    renamed["scenarios"][0]["name"] = serde_json::json!("store_a_test_that_does_not_exist");
    validate_artifact(&renamed, SELF_SOURCE)
        .expect_err("red control: a scenario with no matching test must be rejected");

    // RED CONTROL 2 — GO while a red control arm did not pass.
    let mut half_proven = artifact.clone();
    half_proven["scenarios"][0]["red_control"] = serde_json::json!("SKIPPED");
    validate_artifact(&half_proven, SELF_SOURCE)
        .expect_err("red control: GO without both arms must be rejected");

    // RED CONTROL 3 — GO with no scenarios at all.
    let mut empty = artifact.clone();
    empty["scenarios"] = serde_json::json!([]);
    validate_artifact(&empty, SELF_SOURCE)
        .expect_err("red control: GO with an empty scenario list must be rejected");

    // RED CONTROL 4 — a supporting test that does not exist either.
    let mut ghost = artifact.clone();
    ghost["supporting_tests"] = serde_json::json!(["store_a_supporting_test_that_never_existed"]);
    validate_artifact(&ghost, SELF_SOURCE)
        .expect_err("red control: a supporting test with no matching function must be rejected");

    // RED CONTROL 5 — a test count that does not match the suite.
    let mut miscounted = artifact.clone();
    miscounted["nextest"]["executed"] = serde_json::json!(99);
    validate_artifact(&miscounted, SELF_SOURCE)
        .expect_err("red control: a wrong test count must be rejected");

    // RED CONTROL 6 — a red arm whose nature is undeclared or invented.
    let mut mislabelled = artifact.clone();
    mislabelled["scenarios"][0]["red_control_kind"] = serde_json::json!("looks-fine-to-me");
    validate_artifact(&mislabelled, SELF_SOURCE)
        .expect_err("red control: an unknown red_control_kind must be rejected");
}

/// Collect the names of the real `#[test]` functions of `source`, and those
/// carrying `#[ignore]`.
///
/// Scanning for the ATTRIBUTE (line-leading) rather than for `fn <name>(`
/// matters twice over: a substring search would accept a plain helper such as
/// `validate_artifact` as if it were a test, and this very function mentions
/// both attribute spellings, so a textual count would include itself.
fn test_functions(source: &str) -> (Vec<String>, Vec<String>) {
    let mut tests = Vec::new();
    let mut ignored = Vec::new();
    let mut seen_test = false;
    let mut seen_ignore = false;
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#[test]") {
            seen_test = true;
            seen_ignore = false;
            continue;
        }
        if seen_test && trimmed.starts_with("#[ignore") {
            seen_ignore = true;
            continue;
        }
        if seen_test && let Some(rest) = trimmed.strip_prefix("fn ") {
            let name = rest.split('(').next().unwrap_or("").to_string();
            if seen_ignore {
                ignored.push(name.clone());
            }
            tests.push(name);
            seen_test = false;
            seen_ignore = false;
        }
    }
    (tests, ignored)
}

/// `status=GO` is only accepted when every scenario names a real `#[test]`
/// of this suite, the scenario list has no duplicate, both arms are `PASS`
/// with a declared red-control kind, and the declared test counts match the
/// suite.
///
/// What this CANNOT check, stated so the artifact's wording stays no stronger
/// than its mechanism: it cannot tell that an assertion was deleted from
/// inside a test, and it cannot observe the exit code of the run that
/// produced it. It binds the artifact to the SHAPE of the suite, not to a
/// past execution.
fn validate_artifact(artifact: &serde_json::Value, source: &str) -> Result<(), String> {
    let status = artifact
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or("artifact has no status")?;
    if status != "GO" && status != "NO-GO" {
        return Err(format!("status must be GO or NO-GO, found {status}"));
    }

    let scenarios = artifact
        .get("scenarios")
        .and_then(|v| v.as_array())
        .ok_or("artifact has no scenarios array")?;

    if status == "GO" && scenarios.is_empty() {
        return Err("status=GO with zero scenarios".to_string());
    }

    let (tests, ignored) = test_functions(source);
    let mut declared: Vec<&str> = Vec::new();

    for scenario in scenarios {
        let name = scenario
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or("scenario without a name")?;
        if !tests.iter().any(|t| t == name) {
            return Err(format!("scenario {name} is not a #[test] of this suite"));
        }
        if ignored.iter().any(|t| t == name) {
            return Err(format!("scenario {name} is #[ignore]d — it never runs"));
        }
        if declared.contains(&name) {
            return Err(format!("scenario {name} is declared twice"));
        }
        declared.push(name);
        if status == "GO" {
            for arm in ["green_arm", "red_control"] {
                let value = scenario.get(arm).and_then(|v| v.as_str()).unwrap_or("");
                if value != "PASS" {
                    return Err(format!("status=GO but {name}.{arm} is {value:?}"));
                }
            }
            // Not every red arm is the same thing, and lumping them under one
            // label overstates the evidence: "remove the guarantee and the bad
            // state appears" is strictly stronger than "check the precondition
            // actually happened". The distinction is declared and checked.
            match scenario.get("red_control_kind").and_then(|v| v.as_str()) {
                Some("guarantee-removed") | Some("anti-vacuity") => {}
                other => {
                    return Err(format!(
                        "{name}.red_control_kind must be guarantee-removed or anti-vacuity, \
                         found {other:?}"
                    ));
                }
            }
        }
    }

    // The test count is derived from the SOURCE, never trusted as prose: an
    // artifact whose whole argument is "the claims are machine-checked" must
    // not carry an unchecked number (the first cut said 12 where the quoted
    // command reports 15 executed + 1 ignored).
    let declared_executed = artifact
        .get("nextest")
        .and_then(|n| n.get("executed"))
        .and_then(|v| v.as_u64())
        .ok_or("artifact has no nextest.executed count")?;
    let declared_ignored = artifact
        .get("nextest")
        .and_then(|n| n.get("ignored"))
        .and_then(|v| v.as_u64())
        .ok_or("artifact has no nextest.ignored count")?;
    let ignored_count = ignored.len() as u64;
    let executed_count = tests.len() as u64 - ignored_count;
    if declared_ignored != ignored_count || declared_executed != executed_count {
        return Err(format!(
            "artifact declares {declared_executed} executed / {declared_ignored} ignored, \
             the suite has {executed_count} executed / {ignored_count} ignored"
        ));
    }

    // Supporting tests are named the same way and checked the same way: a
    // list of names nobody verifies would be prose, not evidence.
    for entry in artifact
        .get("supporting_tests")
        .and_then(|v| v.as_array())
        .unwrap_or(&Vec::new())
    {
        let name = entry
            .as_str()
            .ok_or("supporting_tests entries must be strings")?;
        if !tests.iter().any(|t| t == name) {
            return Err(format!(
                "supporting test {name} is not a #[test] of this suite"
            ));
        }
    }
    Ok(())
}

// ── (10) out-of-process: a writer that really dies ────────────────────────

/// The in-process arms halt the run with a typed error; this one kills the
/// WHOLE process at each frontier — no unwinding, no destructor, no
/// `sqlite3_close` — and asserts a fresh process recovers the frontier with a
/// single side output.
///
/// What this does NOT prove is stated in the artifact: it is not a power-loss
/// test, so it says nothing about `fsync` (SQLite is durable across
/// application crashes at any `synchronous` level).
#[test]
fn store_process_exit_recovers_frontier_without_duplicate() {
    let exe = std::env::current_exe().expect("current_exe");
    let key = key_for(RUN_ID, NODE, &[("a", "1")]);

    for frontier in ["B1", "B2", "B3"] {
        let c = ctx();
        let out = Command::new(&exe)
            .args([
                "store_child_writer_exits_mid_run",
                "--exact",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("SBFB_SPIKE_DB", &c.db)
            .env("SBFB_SPIKE_LOG", &c.log)
            .env("SBFB_SPIKE_FAULT", frontier)
            .output()
            .expect("spawn the child writer");

        assert_eq!(
            out.status.code(),
            Some(CHILD_EXIT_CODE),
            "the child must die at {frontier} (stdout: {}, stderr: {})",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );

        // Assert the STATE THE DEAD WRITER LEFT BEHIND, per frontier.
        // Checking only the exit code proves nothing: a child that exits 87
        // without doing anything leaves a pristine store, the parent then
        // runs from scratch and trivially finds "one side output, run
        // completed" — measured, the whole arm stayed green under exactly
        // that mutation. These assertions are what make the arm discriminating.
        {
            let dead = RunStore::open(&c.db).expect("open the store the child left");
            let outputs = read_side_outputs(&c.log).expect("side outputs");
            match frontier {
                "B1" => {
                    assert!(
                        dead.pending_effect(RUN_ID, NODE)
                            .expect("pending")
                            .is_some(),
                        "B1: the intent must be durable"
                    );
                    assert_eq!(outputs.len(), 0, "B1: nothing may have reached the outside");
                }
                "B2" => {
                    assert!(
                        dead.pending_effect(RUN_ID, NODE)
                            .expect("pending")
                            .is_some(),
                        "B2: the effect must still be unconfirmed"
                    );
                    assert_eq!(outputs.len(), 1, "B2: the side output DID happen");
                    assert_eq!(
                        dead.frontier_node(RUN_ID).expect("frontier"),
                        None,
                        "B2: an intended step is NOT a reached one — the \
                         frontier must not have moved"
                    );
                }
                "B3" => {
                    assert!(
                        dead.completed_effect(RUN_ID, NODE)
                            .expect("completed")
                            .is_some(),
                        "B3: the result must be journaled"
                    );
                    assert_eq!(outputs.len(), 1);
                    assert_ne!(
                        dead.run_status(RUN_ID).expect("status"),
                        Some(RunStatus::Completed),
                        "B3: the run must NOT be closed yet"
                    );
                    assert_eq!(
                        dead.frontier_node(RUN_ID).expect("frontier").as_deref(),
                        Some(NODE),
                        "B3: the frontier must have moved to the completed step"
                    );
                }
                other => panic!("unhandled frontier {other}"),
            }
            assert_eq!(dead.orphan_outbox_rows(RUN_ID).expect("orphans"), 0);
        }

        let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&c.log);
        resumed
            .execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
            .unwrap_or_else(|e| panic!("recovery after a real process death at {frontier}: {e}"));

        assert_eq!(
            side_output_count(&c.log, &key).expect("count"),
            1,
            "exactly one side output after a real process death at {frontier}"
        );
        assert_eq!(
            read_side_outputs(&c.log).expect("side outputs").len(),
            1,
            "and no side output under ANY OTHER key either — counting by key \
             alone would miss a duplicate minted under a fresh effect id"
        );
        assert_eq!(
            resumed.store().run_status(RUN_ID).expect("status"),
            Some(RunStatus::Completed)
        );
        assert_eq!(
            resumed.store().orphan_outbox_rows(RUN_ID).expect("orphans"),
            0
        );
    }
}

/// The child writer of the out-of-process arm. `#[ignore]` so it only ever
/// runs when the parent spawns it explicitly (neither `cargo test` nor
/// `cargo nextest run` executes ignored tests by default). Its parameters
/// come from the environment, which is why this harness lives outside `src/`.
#[test]
#[ignore = "spawned by store_process_exit_recovers_frontier_without_duplicate"]
fn store_child_writer_exits_mid_run() {
    let db = PathBuf::from(std::env::var_os("SBFB_SPIKE_DB").expect("SBFB_SPIKE_DB"));
    let log = PathBuf::from(std::env::var_os("SBFB_SPIKE_LOG").expect("SBFB_SPIKE_LOG"));
    let frontier = std::env::var("SBFB_SPIKE_FAULT").expect("SBFB_SPIKE_FAULT");
    let point = match frontier.as_str() {
        "B1" => FaultPoint::AfterIntentBeforeEffect,
        "B2" => FaultPoint::AfterEffectBeforeResult,
        "B3" => FaultPoint::AfterResult,
        other => panic!("unknown frontier {other}"),
    };

    let mut eng = engine(&db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&log);
    let _ = eng.execute(&one_step_run(&[("a", "1")]), &mut target, &ExitAt { point });
    panic!("the child writer was expected to die at {frontier}");
}

// ── (15) the completion transaction is atomic too ─────────────────────────

/// The completion write is composite as well (journal entry + outbox
/// transition + frontier). Halting INSIDE it must roll all three back.
///
/// Without this arm, "every guarantee carries a red control" would be false
/// for the completion path: only the intent transaction had a fault point.
#[test]
fn store_completion_transaction_is_atomic() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    expect_halt(
        eng.execute(
            &one_step_run(&[("a", "1")]),
            &mut target,
            &HaltAt::new(FaultPoint::InsideCompletionTransaction),
        ),
        FaultPoint::InsideCompletionTransaction,
    );

    // The side output DID happen (the halt is after `apply`), but nothing of
    // the completion is durable: no completed effect, the row is still
    // pending, the frontier has not moved.
    assert_eq!(read_side_outputs(&c.log).expect("outputs").len(), 1);
    assert!(
        eng.store()
            .completed_effect(RUN_ID, NODE)
            .expect("completed")
            .is_none(),
        "the completion entry must be rolled back with the outbox transition"
    );
    assert!(
        eng.store()
            .pending_effect(RUN_ID, NODE)
            .expect("pending")
            .is_some(),
        "the effect stays unconfirmed"
    );
    assert_eq!(eng.store().frontier_node(RUN_ID).expect("frontier"), None);
    assert_eq!(
        eng.store().event_count(RUN_ID).expect("events"),
        2,
        "run_started + effect_intended only"
    );

    // And the resume still converges to exactly one side output.
    let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    resumed
        .execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
        .expect("resume");
    assert_eq!(read_side_outputs(&c.log).expect("outputs").len(), 1);
    assert_eq!(
        resumed.store().run_status(RUN_ID).expect("status"),
        Some(RunStatus::Completed)
    );

    // RED CONTROL — split the completion into two transactions. The journal
    // then says the effect COMPLETED while the outbox row is still pending:
    // an inconsistent state the atomic path cannot reach. Without this arm,
    // the completion path had a crash point but no arm REMOVING its
    // guarantee, so the module-level claim was false for it.
    let red = ctx();
    let mut red_eng = engine(&red.db, SpikeKnobs::without_atomic_completion());
    let mut red_target = FileEchoTarget::idempotent(&red.log);
    red_eng
        .execute(&one_step_run(&[("a", "1")]), &mut red_target, &NoFaults)
        .expect("the split path still returns Ok — that is the danger");
    assert!(
        red_eng
            .store()
            .pending_effect(RUN_ID, NODE)
            .expect("pending")
            .is_some(),
        "red control: the outbox row stays PENDING although the journal \
         already records the completion"
    );
    assert!(
        red_eng
            .store()
            .completed_effect(RUN_ID, NODE)
            .expect("completed")
            .is_none(),
        "red control: journal and outbox disagree — exactly the state the \
         atomic path makes unreachable"
    );
    // Assert the JOURNAL side directly. `completed_effect` only reads the
    // outbox, so the two assertions above would stay green even if the split
    // path wrote NOTHING at all — the arm would then prove the absence of a
    // write instead of the inconsistency between two writes.
    let red_events = red_eng.store().load_events(RUN_ID).expect("events");
    assert_eq!(
        red_events
            .iter()
            .filter(|e| matches!(e.kind, RunEventKind::EffectCompleted { .. }))
            .count(),
        1,
        "red control: the completion entry IS committed — alone"
    );
    assert_eq!(
        red_events.len(),
        4,
        "run_started + effect_intended + effect_completed + run_completed"
    );
    assert_eq!(
        red_eng.store().frontier_node(RUN_ID).expect("frontier"),
        None,
        "red control: the frontier never moved, though the journal says completed"
    );
}

// ── (16) divergence on the PENDING branch, and intent counting ────────────

/// The fail-closed hash check exists on TWO branches — the memoised one and
/// the PENDING one — and only the first was ever exercised. A resume with a
/// divergent input after a crash must refuse just as loudly, without applying
/// anything.
///
/// Also asserts, directly, that a resume leaves exactly ONE `effect_intended`
/// entry: an engine minting a fresh effect id on resume would add a second
/// one, and no other assertion in the suite would notice.
#[test]
fn store_pending_branch_diverges_fails_closed() {
    let c = ctx();
    {
        let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&c.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut target,
                &HaltAt::new(FaultPoint::AfterIntentBeforeEffect),
            ),
            FaultPoint::AfterIntentBeforeEffect,
        );
    }

    // Resume with a DIFFERENT input: the pending branch must fail closed.
    let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    match resumed.execute(&one_step_run(&[("a", "2")]), &mut target, &NoFaults) {
        Err(EngineError::StepHashMismatch { .. }) => {}
        other => panic!("the pending branch must fail closed, got {other:?}"),
    }
    assert_eq!(
        read_side_outputs(&c.log).expect("outputs").len(),
        0,
        "a divergent resume must not reach the target at all"
    );
    assert_eq!(
        resumed.store().run_status(RUN_ID).expect("status"),
        Some(RunStatus::Terminal)
    );

    // Intent counting on a healthy resume, in a fresh run.
    let d = ctx();
    {
        let mut eng = engine(&d.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&d.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut target,
                &HaltAt::new(FaultPoint::AfterEffectBeforeResult),
            ),
            FaultPoint::AfterEffectBeforeResult,
        );
    }
    let mut resumed = engine(&d.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&d.log);
    resumed
        .execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
        .expect("resume");
    let intents = resumed
        .store()
        .load_events(RUN_ID)
        .expect("events")
        .into_iter()
        .filter(|e| matches!(e.kind, RunEventKind::EffectIntended { .. }))
        .count();
    assert_eq!(
        intents, 1,
        "a resume must reuse the journaled intent, never mint a second one"
    );
    assert_eq!(read_side_outputs(&d.log).expect("outputs").len(), 1);
}

// ── (17) the dedup BACKEND is bound, not just the target name ─────────────

/// Two honest targets can share a name and an idempotency claim while
/// deduplicating in DIFFERENT durable stores. Resuming against the second one
/// would pass every other guard and apply the key a second time, because the
/// receipt of the first attempt is in the first store.
#[test]
fn store_target_dedup_scope_is_journaled() {
    let c = ctx();
    let other_log = c.db.with_file_name("other-side-outputs.tsv");
    {
        let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&c.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut target,
                &HaltAt::new(FaultPoint::AfterEffectBeforeResult),
            ),
            FaultPoint::AfterEffectBeforeResult,
        );
    }
    assert_eq!(read_side_outputs(&c.log).expect("outputs").len(), 1);

    // Same name, same promise, DIFFERENT backend.
    let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
    let mut elsewhere = FileEchoTarget::idempotent(&other_log);
    match resumed.execute(&one_step_run(&[("a", "1")]), &mut elsewhere, &NoFaults) {
        Err(EngineError::TargetScopeChanged { .. }) => {}
        other => panic!("a changed dedup backend must be refused, got {other:?}"),
    }
    assert_eq!(
        read_side_outputs(&other_log).expect("outputs").len(),
        0,
        "the second backend must not have received a duplicate side output"
    );

    // The honest resume — same backend — still converges.
    let mut same = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    same.execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults)
        .expect("resume against the journaled backend");
    assert_eq!(read_side_outputs(&c.log).expect("outputs").len(), 1);
}

// ── (12) identity components are checked, not assumed ─────────────────────

/// The side-output line and the effect-id preimage are flat encodings. Their
/// unambiguity was originally documented as holding "by construction" while
/// nothing enforced it — on `String` fields anyone fills.
///
/// RED CONTROL: the collision and the forgery the validation prevents are
/// demonstrated on the RAW encodings, so the guard is proven load-bearing
/// rather than merely present.
#[test]
fn store_identity_components_are_validated() {
    // A control character anywhere in an identity component is refused
    // BEFORE anything reaches the outside.
    for hostile in ["run\tid", "run\nid", "run\u{1f}id", ""] {
        let c = ctx();
        let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
        let mut target = FileEchoTarget::idempotent(&c.log);
        let spec = RunSpec {
            run_id: hostile.to_string(),
            steps: vec![StepSpec {
                node_instance_id: NODE.to_string(),
                input: input(&[("a", "1")]),
            }],
        };
        match eng.execute(&spec, &mut target, &NoFaults) {
            Err(EngineError::Hash(HashError::InvalidIdentity { .. })) => {}
            other => panic!("hostile run_id {hostile:?} must be refused, got {other:?}"),
        }
        assert_eq!(
            read_side_outputs(&c.log).expect("side outputs").len(),
            0,
            "a refused identity must never reach the target"
        );
    }
    // Same gate on the node path.
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);
    let spec = RunSpec {
        run_id: RUN_ID.to_string(),
        steps: vec![StepSpec {
            node_instance_id: "root/echo\t#1".to_string(),
            input: input(&[("a", "1")]),
        }],
    };
    assert!(matches!(
        eng.execute(&spec, &mut target, &NoFaults),
        Err(EngineError::Hash(HashError::InvalidIdentity { .. }))
    ));

    // RED CONTROL (a) — the ambiguity is demonstrated on the UNVALIDATED
    // encoder, which is the only way to show what validation buys.
    //
    // (An earlier version of this arm compared two identical string literals
    // with `assert_eq!` and called that a demonstration. It could not fail
    // whatever the production code did — a tautology dressed as a red
    // control, in the one phase whose whole subject is that a green test can
    // be worthless. Kept in mind, not repeated.)
    let left = ("a\u{1f}b", "c", "e");
    let right = ("a", "b\u{1f}c", "e");
    assert_eq!(
        raw_key_line(left.0, left.1, left.2),
        raw_key_line(right.0, right.1, right.2),
        "two DISTINCT identities render one line without validation — a \
         deduplicating target would collapse two different effects into one"
    );
    assert!(
        IdempotencyKey::new(left.0, left.1, left.2).is_err()
            && IdempotencyKey::new(right.0, right.1, right.2).is_err(),
        "and that is precisely why both are refused at the gate"
    );
    // The validated encoder is injective where the raw one is not.
    let a = key_for("run-a", "node-b", &[("k", "v")]);
    let b = key_for("run-a", "node-c", &[("k", "v")]);
    assert_ne!(a.as_line(), b.as_line());

    // RED CONTROL (b) — length prefixing makes the effect-id preimage
    // unambiguous on its own, independently of the validation gate.
    assert_ne!(
        derive_effect_id("r\u{1f}n", "x", "H"),
        derive_effect_id("r", "n\u{1f}x", "H"),
        "length-prefixed components must not collide"
    );
}

// ── (13) the run is bound to its spec ─────────────────────────────────────

/// A step ADDED or REMOVED between two executions of the same `run_id` is
/// invisible to per-step hashes (they only cover steps present on both
/// sides). The spec digest closes that hole, and a completed run cannot be
/// extended.
#[test]
fn store_run_is_bound_to_its_spec() {
    let c = ctx();
    let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
    let mut target = FileEchoTarget::idempotent(&c.log);

    let two_steps = RunSpec {
        run_id: RUN_ID.to_string(),
        steps: vec![
            StepSpec {
                node_instance_id: "root/echo#1".to_string(),
                input: input(&[("a", "1")]),
            },
            StepSpec {
                node_instance_id: "root/echo#2".to_string(),
                input: input(&[("b", "2")]),
            },
        ],
    };
    eng.execute(&two_steps, &mut target, &NoFaults)
        .expect("first run");
    let closed_at = eng.store().event_count(RUN_ID).expect("count");

    // Truncated spec: must NOT return a success describing a one-step run
    // while the journal holds the second effect.
    match eng.execute(&one_step_run(&[("a", "1")]), &mut target, &NoFaults) {
        Err(EngineError::RunAlreadyCompleted { .. })
        | Err(EngineError::SpecDigestMismatch { .. }) => {}
        other => panic!("a truncated spec must be refused, got {other:?}"),
    }

    // Extended spec: must not append entries after the closing entry.
    let mut extended = two_steps.clone();
    extended.steps.push(StepSpec {
        node_instance_id: "root/echo#3".to_string(),
        input: input(&[("c", "3")]),
    });
    match eng.execute(&extended, &mut target, &NoFaults) {
        Err(EngineError::RunAlreadyCompleted { .. })
        | Err(EngineError::SpecDigestMismatch { .. }) => {}
        other => panic!("a completed run must not be extendable, got {other:?}"),
    }
    assert_eq!(
        eng.store().event_count(RUN_ID).expect("count"),
        closed_at,
        "a refused spec must not have appended anything"
    );
    assert_eq!(
        read_side_outputs(&c.log).expect("side outputs").len(),
        2,
        "and must not have applied a third effect"
    );

    // The identical spec is still replayable: refusing extension must not
    // break resume.
    let replay = eng
        .execute(&two_steps, &mut target, &NoFaults)
        .expect("identical spec must still replay");
    assert_eq!(replay.steps_memoised, 2);
    assert_eq!(replay.steps_executed, 0);

    // A duplicated node path is refused before anything runs.
    let dup = RunSpec {
        run_id: "run-dup".to_string(),
        steps: vec![
            StepSpec {
                node_instance_id: "root/send#1".to_string(),
                input: input(&[("to", "alice")]),
            },
            StepSpec {
                node_instance_id: "root/send#1".to_string(),
                input: input(&[("to", "alice")]),
            },
        ],
    };
    assert!(matches!(
        eng.execute(&dup, &mut target, &NoFaults),
        Err(EngineError::DuplicateNodeInstance { .. })
    ));
}

// ── (14) the target's promise is journaled, not re-asked ──────────────────

/// The idempotency claim that authorises re-applying an unconfirmed effect is
/// read from the JOURNAL. Re-asking the live object would let a target with
/// the same name but a different promise authorise a re-application after the
/// crash — with no hash divergence to catch it.
#[test]
fn store_target_idempotency_claim_is_journaled() {
    let c = ctx();
    let key = key_for(RUN_ID, NODE, &[("a", "1")]);
    {
        let mut eng = engine(&c.db, SpikeKnobs::guaranteed());
        let mut honest = FileEchoTarget::non_idempotent(&c.log);
        expect_halt(
            eng.execute(
                &one_step_run(&[("a", "1")]),
                &mut honest,
                &HaltAt::new(FaultPoint::AfterEffectBeforeResult),
            ),
            FaultPoint::AfterEffectBeforeResult,
        );
    }
    assert_eq!(side_output_count(&c.log, &key).expect("count"), 1);

    // Resume with a target that NOW claims idempotency: refused.
    let mut resumed = engine(&c.db, SpikeKnobs::guaranteed());
    let mut swapped = FileEchoTarget::idempotent(&c.log);
    match resumed.execute(&one_step_run(&[("a", "1")]), &mut swapped, &NoFaults) {
        Err(EngineError::TargetIdempotencyChanged {
            journaled, live, ..
        }) => {
            assert!(!journaled, "the journal recorded a non-idempotent target");
            assert!(live, "the live target claims idempotency");
        }
        other => panic!("a changed idempotency claim must be refused, got {other:?}"),
    }
    assert_eq!(
        side_output_count(&c.log, &key).expect("count"),
        1,
        "the swap must not have authorised a re-application"
    );
}

// ── (11) the defaults are the guaranteed configuration ────────────────────

/// A caller who writes `SpikeKnobs::default()` gets every guarantee. The
/// degraded configurations are only reachable through constructors named
/// after the guarantee they remove.
#[test]
fn store_knobs_default_is_guaranteed() {
    assert_eq!(SpikeKnobs::default(), SpikeKnobs::guaranteed());
    assert!(SpikeKnobs::guaranteed().verify_step_hash());
    assert!(SpikeKnobs::guaranteed().atomic_outbox());
    assert!(!SpikeKnobs::without_step_hash_verification().verify_step_hash());
    assert!(!SpikeKnobs::without_atomic_outbox().atomic_outbox());
    assert!(SpikeKnobs::guaranteed().atomic_completion());
    assert!(!SpikeKnobs::without_atomic_completion().atomic_completion());
    // Each degraded constructor removes ONE guarantee and leaves the rest on.
    assert!(SpikeKnobs::without_atomic_completion().verify_step_hash());
    assert!(SpikeKnobs::without_atomic_completion().atomic_outbox());
    assert!(FileEchoTarget::idempotent("x").is_idempotent());
    assert!(!FileEchoTarget::non_idempotent("x").is_idempotent());
    assert!(
        FileEchoTarget::lying("x").is_idempotent(),
        "the lying target must CLAIM idempotency — that is what makes it a red control"
    );
}
