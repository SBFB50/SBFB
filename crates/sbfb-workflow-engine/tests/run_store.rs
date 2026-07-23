// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 83 Phase I — the full run store harness.
//!
//! Same doctrine as `store_spike.rs` (Phase E): every guarantee has TWO
//! arms — the guaranteed one, and a RED CONTROL where the guarantee is
//! removed (or the raw bad state is reached through a `spike-faults`-gated
//! door) and the test asserts the bad state IS observed. A green suite must
//! never be vacuous.
//!
//! Gate naming: every test name starts with `store_` (`G-STORE` filter);
//! the upcasting tests contain `upgrade_replay` (`G-UPGRADE-REPLAY` filter,
//! `cargo nextest run -p sbfb-workflow-engine upgrade_replay`); the lease
//! tests contain `single_writer` (`G-SINGLE-WRITER` filter). No test name
//! contains `effect`, so the `G-EFFECT` gate stays owned by Phase K.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use sbfb_workflow_engine::event::{RunEventKind, RunEventUpcaster, TerminalReason, UpcastError};
use sbfb_workflow_engine::fault::{FaultAction, FaultHook, FaultPoint};
use sbfb_workflow_engine::hash::{StepRequest, derive_effect_id, step_request_hash};
use sbfb_workflow_engine::store::{
    GcOutcome, RetentionPolicy, RunStatus, RunStore, SNAPSHOT_FORMAT_VERSION, SnapshotRead,
    StoreError,
};

/// Exit code of the out-of-process lease writer — outside the libtest range
/// (0 = pass, 101 = panic) so the parent cannot mistake a panicking child
/// for a killed one. Mirrors `store_spike.rs`.
const CHILD_EXIT_CODE: i32 = 87;

const NODE: &str = "root/echo#1";

struct Ctx {
    _dir: tempfile::TempDir,
    db: PathBuf,
}

fn ctx() -> Ctx {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("run.sqlite3");
    Ctx { _dir: dir, db }
}

fn input(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect()
}

/// Drive one step of a run straight through the STORE (intent + completion),
/// without an engine or a target: these tests exercise store-level
/// guarantees, and a synthetic receipt keeps them independent of the echo
/// target's own behaviour.
fn complete_one_step(store: &RunStore, run_id: &str, node: &str) {
    let request = StepRequest::new(node, "echo", input(&[("a", "1")])).expect("request");
    let request_hash = step_request_hash(&request).expect("hash");
    let effect_id = derive_effect_id(run_id, node, &request_hash);
    store
        .record_intent_atomic(run_id, node, &request_hash, &effect_id, true, "test-scope")
        .expect("intent");
    store
        .record_completion(run_id, node, &effect_id, &request_hash, "receipt")
        .expect("completion");
}

/// A CLOSED completed run with `closed_at_ms`, one completed step deep.
fn make_completed_run(store: &RunStore, run_id: &str, closed_at_ms: i64) {
    store.ensure_run(run_id, "digest").expect("ensure");
    complete_one_step(store, run_id, NODE);
    store
        .record_run_completed(run_id, Some(closed_at_ms))
        .expect("close");
}

/// A TERMINAL run with a PENDING outbox row (the outcome_unknown shape) and
/// `closed_at_ms`.
fn make_terminal_run_with_pending(store: &RunStore, run_id: &str, closed_at_ms: i64) {
    store.ensure_run(run_id, "digest").expect("ensure");
    let request = StepRequest::new(NODE, "echo", input(&[("a", "1")])).expect("request");
    let request_hash = step_request_hash(&request).expect("hash");
    let effect_id = derive_effect_id(run_id, NODE, &request_hash);
    store
        .record_intent_atomic(run_id, NODE, &request_hash, &effect_id, false, "test-scope")
        .expect("intent");
    store
        .record_terminal(
            run_id,
            Some(NODE),
            TerminalReason::OutcomeUnknown,
            None,
            None,
            Some(closed_at_ms),
        )
        .expect("terminal");
}

/// Halts inside a transaction at one point — local mirror of the
/// `store_spike.rs` hook.
struct HaltAt(FaultPoint);

impl FaultHook for HaltAt {
    fn hit(&self, point: FaultPoint) -> FaultAction {
        if point == self.0 {
            FaultAction::Halt
        } else {
            FaultAction::Continue
        }
    }
}

// ── upcasting + downgrade guards (G-UPGRADE-REPLAY) ───────────────────────

/// Synthetic hop v1→v2 used by the store-level upcast arm: renames the
/// legacy kind. Refuses anything else, so a skipped hop is observable.
fn hop_rename_legacy(value: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut obj = match value {
        serde_json::Value::Object(o) => o,
        other => return Err(format!("expected object, got {other}")),
    };
    match obj.get("kind").and_then(|k| k.as_str()) {
        Some("run_started_legacy") => {
            obj.insert(
                "kind".to_string(),
                serde_json::Value::String("run_started".to_string()),
            );
            Ok(serde_json::Value::Object(obj))
        }
        other => Err(format!("expected run_started_legacy, got {other:?}")),
    }
}

/// An identity hop — the red control proving a hop that does NOT transform
/// cannot silently pass: the typed parse downstream refuses the old shape.
fn hop_identity(value: serde_json::Value) -> Result<serde_json::Value, String> {
    Ok(value)
}

/// An OLD run (synthetic older payload shape, pinned by a committed
/// fixture) replays through the store once the chain carries its hop — and
/// the journal row keeps its ORIGINAL version as provenance.
#[test]
fn store_upgrade_replay_old_payload_through_synthetic_chain() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/upcast_v1_synthetic_payload.json"))
            .expect("fixture parses");
    let payload = serde_json::to_string(&fixture["payload"]).expect("payload");

    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-up", "digest").expect("ensure");
    store
        .append_raw_event_for_harness("run-up", 1, "run_started", &payload)
        .expect("raw row");
    // In the SYNTHETIC world of this arm, "current" is 2 — the run_started
    // row `ensure_run` just wrote (a real, current-shape payload) plays the
    // role of an already-current row, so it is marked version 2; only the
    // fixture row is the OLD shape at version 1.
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE run_events SET format_version = 2 WHERE run_id = 'run-up' AND sequence = 0",
        [],
    )
    .expect("mark current");
    drop(conn);

    // Guaranteed arm: the hop brings the old shape to the current one.
    let chain: &[RunEventUpcaster] = &[hop_rename_legacy];
    let events = store
        .load_events_with_chain("run-up", chain, 2)
        .expect("upcasted load");
    // Row 0 is run_started from ensure_run; row 1 is the synthetic one.
    assert_eq!(events.len(), 2);
    assert_eq!(events[1].kind, RunEventKind::RunStarted);
    assert_eq!(
        events[1].format_version, 1,
        "the event keeps the version its ROW was written under (provenance)"
    );

    // RED CONTROL (per-hop, S1a Axon precedent): a chain missing its hop is
    // a LOUD ChainGap, never a silent skip…
    let err = store
        .load_events_with_chain("run-up", &[], 2)
        .expect_err("a chain with a gap must refuse");
    assert!(
        matches!(
            err,
            StoreError::Upcast(UpcastError::ChainGap {
                chain_len: 0,
                current: 2
            })
        ),
        "got {err:?}"
    );

    // …and a hop that does NOT transform leaves the old shape to the typed
    // parser, which refuses it: the transformation itself is load-bearing.
    let identity_chain: &[RunEventUpcaster] = &[hop_identity];
    let err = store
        .load_events_with_chain("run-up", identity_chain, 2)
        .expect_err("an identity hop must not smuggle the old shape through");
    assert!(matches!(err, StoreError::Corrupt(_)), "got {err:?}");
}

/// DOWNGRADE GUARD, production path: a row written by a NEWER engine is
/// refused fail-closed — and version 0 is corrupt, not old.
#[test]
fn store_upgrade_replay_refuses_newer_format_version() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-newer", "digest").expect("ensure");
    store
        .append_raw_event_for_harness("run-newer", 2, "run_started", r#"{"kind":"run_started"}"#)
        .expect("raw row");
    let err = store
        .load_events("run-newer")
        .expect_err("a newer row must be refused");
    assert!(
        matches!(
            err,
            StoreError::Upcast(UpcastError::NewerThanSupported {
                found: 2,
                supported: 1
            })
        ),
        "got {err:?}"
    );

    store.ensure_run("run-zero", "digest").expect("ensure");
    store
        .append_raw_event_for_harness("run-zero", 0, "run_started", r#"{"kind":"run_started"}"#)
        .expect("raw row");
    let err = store
        .load_events("run-zero")
        .expect_err("version 0 must be refused");
    assert!(
        matches!(err, StoreError::Upcast(UpcastError::VersionZero)),
        "got {err:?}"
    );
}

/// DOWNGRADE GUARD, schema level: a store whose `user_version` exceeds the
/// migrations this binary carries refuses to OPEN — older code never runs
/// against newer tables (this is also what protects a restore from a
/// too-new backup).
#[test]
fn store_upgrade_replay_schema_version_guard_at_open() {
    let c = ctx();
    drop(RunStore::open(&c.db).expect("create"));

    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.pragma_update(None, "user_version", 99).expect("bump");
    drop(conn);

    let err = RunStore::open(&c.db).expect_err("a too-new schema must refuse to open");
    assert!(
        matches!(
            err,
            StoreError::UnsupportedSchemaVersion {
                found: 99,
                // 3 since Phase J appended M3 (waits/signals/timers).
                supported: 3
            }
        ),
        "got {err:?}"
    );
}

// ── single-writer lease fencing (G-SINGLE-WRITER) ─────────────────────────

/// The fencing guarantee: after a takeover, the STALE handle emits NOTHING —
/// no journal entry, no outbox row — whatever its own clock believes. Safety
/// is the epoch CAS inside the write transaction; expiry only gates the
/// takeover.
#[test]
fn store_single_writer_lease_fencing() {
    let c = ctx();
    let mut a = RunStore::open(&c.db).expect("open a");
    let mut b = RunStore::open(&c.db).expect("open b");

    assert_eq!(
        a.acquire_lease("writer-a", 0, 1_000).expect("a acquires"),
        1
    );

    // A live lease cannot be stolen…
    let err = b
        .acquire_lease("writer-b", 500, 1_000)
        .expect_err("live lease must refuse");
    assert!(
        matches!(err, StoreError::LeaseHeld { ref owner, expires_at_ms: 1_000 } if owner == "writer-a"),
        "got {err:?}"
    );

    // …until it expires: the takeover bumps the epoch.
    assert_eq!(
        b.acquire_lease("writer-b", 1_500, 1_000).expect("takeover"),
        2
    );

    // THE guarantee: the stale handle's next write aborts inside its own
    // transaction and leaves NOTHING behind — its wall clock plays no part
    // (from A's point of view its lease "should" still be running; the
    // epoch CAS is what says otherwise).
    let err = a
        .ensure_run("run-fenced", "digest")
        .expect_err("fenced-out writer must fail");
    assert!(
        matches!(err, StoreError::LeaseLost { epoch: 1, .. }),
        "got {err:?}"
    );
    assert_eq!(
        a.run_status("run-fenced").expect("status"),
        None,
        "the fenced-out write must have been rolled back entirely"
    );

    // A fenced-out intent emits no outbox row either.
    b.ensure_run("run-b", "digest").expect("b writes fine");
    let request = StepRequest::new(NODE, "echo", input(&[("a", "1")])).expect("request");
    let request_hash = step_request_hash(&request).expect("hash");
    let effect_id = derive_effect_id("run-b", NODE, &request_hash);
    let err = a
        .record_intent_atomic("run-b", NODE, &request_hash, &effect_id, true, "scope")
        .expect_err("fenced-out intent must fail");
    assert!(matches!(err, StoreError::LeaseLost { .. }), "got {err:?}");
    assert!(
        a.pending_effect("run-b", NODE).expect("pending").is_none(),
        "no outbox row may survive a fenced-out transaction"
    );
    assert_eq!(a.orphan_outbox_rows("run-b").expect("orphans"), 0);

    // A stale renewal is refused the same way, and clears the handle.
    let err = a.renew_lease(1_600, 1_000).expect_err("stale renew");
    assert!(matches!(err, StoreError::LeaseLost { .. }), "got {err:?}");
    assert_eq!(a.active_lease_epoch(), None);

    // Re-acquisition by the SAME owner name also bumps the epoch: a
    // restarted process invalidates its own zombie handles.
    let mut b2 = RunStore::open(&c.db).expect("open b2");
    assert_eq!(
        b2.acquire_lease("writer-b", 1_600, 1_000)
            .expect("same-owner reacquire"),
        3
    );
    let err = b
        .ensure_run("run-zombie", "digest")
        .expect_err("the zombie handle of the same owner must be fenced out");
    assert!(
        matches!(err, StoreError::LeaseLost { epoch: 2, .. }),
        "got {err:?}"
    );

    // An expired lease can be re-taken and writing resumes.
    assert_eq!(
        a.acquire_lease("writer-a", 999_999, 1_000)
            .expect("a re-acquires after expiry"),
        4
    );
    a.ensure_run("run-a-again", "digest")
        .expect("a writes again");
}

/// Crash-during-lease under a REAL process death: the child acquires the
/// lease, records an intent and dies without releasing. The parent observes
/// the held lease, takes over after expiry (epoch bump) and finishes the
/// step under its own fence. An in-process halt cannot prove this — the
/// lease of a dead holder only ever goes away through takeover.
#[test]
fn store_single_writer_takeover_after_real_writer_death() {
    let exe = std::env::current_exe().expect("current_exe");
    let c = ctx();

    let out = Command::new(&exe)
        .args([
            "store_single_writer_child_dies_holding_lease",
            "--exact",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("SBFB_RUNSTORE_DB", &c.db)
        .output()
        .expect("spawn the child writer");
    assert_eq!(
        out.status.code(),
        Some(CHILD_EXIT_CODE),
        "the child must die holding the lease (stdout: {}, stderr: {})",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Assert the state the DEAD holder left: a held lease and a pending
    // intent (checking the exit code alone would prove nothing — the E
    // measurement).
    let mut store = RunStore::open(&c.db).expect("open after death");
    let lease = store.lease_state().expect("lease").expect("lease row");
    assert_eq!(lease.owner, "child-writer");
    assert_eq!(lease.epoch, 1);
    assert!(
        store
            .pending_effect("run-lease", NODE)
            .expect("pending")
            .is_some(),
        "the child's intent must be durable"
    );

    // Before expiry the dead holder still blocks an aspiring owner…
    let err = store
        .acquire_lease("parent-writer", lease.expires_at_ms - 1, 1_000)
        .expect_err("not expired yet");
    assert!(matches!(err, StoreError::LeaseHeld { .. }), "got {err:?}");

    // …after expiry the takeover bumps the epoch and the run finishes
    // under the new fence.
    assert_eq!(
        store
            .acquire_lease("parent-writer", lease.expires_at_ms + 1, 60_000)
            .expect("takeover after real death"),
        2
    );
    let pending = store
        .pending_effect("run-lease", NODE)
        .expect("pending")
        .expect("still pending");
    assert!(pending.target_idempotent);
    store
        .record_completion(
            "run-lease",
            NODE,
            &pending.effect_id,
            &pending.request_hash,
            "receipt",
        )
        .expect("completion under the new fence");
    assert!(
        store
            .completed_effect("run-lease", NODE)
            .expect("completed")
            .is_some()
    );
    assert_eq!(store.orphan_outbox_rows("run-lease").expect("orphans"), 0);
    // And the run actually FINISHES under the new fence — completion alone
    // only moves the frontier (Codex r1: proving the takeover is proving
    // the run can be driven to its close, not merely one write).
    store
        .record_run_completed("run-lease", Some(200_000))
        .expect("close under the new fence");
    assert_eq!(
        store.run_status("run-lease").expect("status"),
        Some(RunStatus::Completed)
    );
}

/// The child of the out-of-process lease arm. `#[ignore]` — only ever runs
/// when the parent spawns it (same mechanism as the Phase E child writer).
#[test]
#[ignore = "spawned by store_single_writer_takeover_after_real_writer_death"]
fn store_single_writer_child_dies_holding_lease() {
    let db = PathBuf::from(std::env::var_os("SBFB_RUNSTORE_DB").expect("SBFB_RUNSTORE_DB"));
    let mut store = RunStore::open(&db).expect("child open");
    store
        .acquire_lease("child-writer", 0, 60_000)
        .expect("child acquires");
    store.ensure_run("run-lease", "digest").expect("child run");
    let request = StepRequest::new(NODE, "echo", input(&[("a", "1")])).expect("request");
    let request_hash = step_request_hash(&request).expect("hash");
    let effect_id = derive_effect_id("run-lease", NODE, &request_hash);
    store
        .record_intent_atomic("run-lease", NODE, &request_hash, &effect_id, true, "scope")
        .expect("child intent");
    // Die WITHOUT releasing, without unwinding, without closing SQLite.
    std::process::exit(CHILD_EXIT_CODE);
}

// ── disposable snapshots ──────────────────────────────────────────────────

/// A snapshot is a CACHE: stale ⇒ discarded, poisoned ⇒ DETECTED and
/// discarded, dropped ⇒ nothing changes — and the journal serves the same
/// truth throughout.
#[test]
fn store_snapshot_is_disposable_cache() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-snap", 1_000);
    let journal_before = store.load_events("run-snap").expect("events");

    let state = serde_json::json!({"frontier": NODE, "steps": 1});
    store
        .write_snapshot("run-snap", 3, &state)
        .expect("write snapshot");
    match store.read_snapshot("run-snap").expect("read") {
        SnapshotRead::Valid(s) => {
            assert_eq!(s.upto_sequence, 3);
            assert_eq!(s.state, state);
        }
        other => panic!("expected Valid, got {other:?}"),
    }

    // POISON the stored state directly (tampering path), leaving the
    // recorded digest behind: the read DETECTS it and discards.
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE snapshots SET state = ?1 WHERE run_id = ?2",
        rusqlite::params![r#"{"frontier":"root/echo#1","steps":99}"#, "run-snap"],
    )
    .expect("poison");
    drop(conn);
    assert_eq!(
        store.read_snapshot("run-snap").expect("read"),
        SnapshotRead::Poisoned,
        "a tampered snapshot must be detected, never trusted"
    );

    // The digest covers the WHOLE served tuple: moving the sequence bound
    // alone (state intact) is detected too (review D4-1).
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE snapshots SET state = ?1, upto_sequence = 999 WHERE run_id = ?2",
        rusqlite::params![serde_json::to_string(&state).expect("state"), "run-snap"],
    )
    .expect("tamper the bound, restore the state");
    drop(conn);
    assert_eq!(
        store.read_snapshot("run-snap").expect("read"),
        SnapshotRead::Poisoned,
        "a lying upto_sequence under an intact state must be detected"
    );
    // Restore the poisoned-state shape the next arm expects.
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE snapshots SET state = ?1, upto_sequence = 3 WHERE run_id = ?2",
        rusqlite::params![r#"{"frontier":"root/echo#1","steps":99}"#, "run-snap"],
    )
    .expect("re-poison state only");
    drop(conn);

    // RED CONTROL — with the digest check off, the poisoned state IS served:
    // the bad state is reachable, so the guard above is load-bearing.
    match store
        .read_snapshot_unverified("run-snap")
        .expect("unverified read")
    {
        SnapshotRead::Valid(s) => assert_eq!(s.state["steps"], 99),
        other => panic!("the unverified door must serve the poison, got {other:?}"),
    }

    // A different snapshot FORMAT version is stale — invalidation, never
    // interpretation.
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE snapshots SET snapshot_version = ?1 WHERE run_id = ?2",
        rusqlite::params![SNAPSHOT_FORMAT_VERSION + 1, "run-snap"],
    )
    .expect("stale");
    drop(conn);
    assert!(matches!(
        store.read_snapshot("run-snap").expect("read"),
        SnapshotRead::Stale { .. }
    ));

    // Dropping the cache changes NOTHING observable: the journal is the
    // truth and re-derives the same state.
    store.drop_snapshot("run-snap").expect("drop");
    assert_eq!(
        store.read_snapshot("run-snap").expect("read"),
        SnapshotRead::Missing
    );
    assert_eq!(
        store.load_events("run-snap").expect("events"),
        journal_before,
        "the journal must be untouched by every snapshot operation"
    );

    // The digest door refuses a null anywhere in the state — same rule as
    // every digest of this crate.
    let err = store
        .write_snapshot("run-snap", 3, &serde_json::json!({"x": null}))
        .expect_err("null in snapshot state");
    assert!(
        matches!(err, StoreError::InvalidInput { .. }),
        "got {err:?}"
    );
}

// ── retention / GC ────────────────────────────────────────────────────────

/// The retention policy purges ONLY closed runs past the horizon, spares
/// the most recent `keep_last`, never touches a running run, a pending
/// outbox row or the terminal diagnosis — and a second pass is idempotent.
#[test]
fn store_gc_purges_only_closed_runs_within_policy() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");

    make_completed_run(&store, "run-old-completed", 1_000);
    make_terminal_run_with_pending(&store, "run-old-terminal", 1_000);
    make_completed_run(&store, "run-recent-completed", 5_000);
    store.ensure_run("run-running", "digest").expect("running");
    complete_one_step(&store, "run-running", NODE);

    let outcome = store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: 2_000,
                keep_last: 1,
            },
        )
        .expect("gc");
    assert_eq!(
        outcome,
        GcOutcome {
            completed_purged: 1,
            terminal_purged: 1,
            skipped_pending_outbox: 0,
        }
    );

    // The old completed run is fully gone.
    assert_eq!(store.run_status("run-old-completed").expect("status"), None);
    assert_eq!(store.event_count("run-old-completed").expect("events"), 0);

    // The old terminal run keeps its DIAGNOSIS: the runs row (with its
    // terminal reason), the run_terminal entry, and the pending outbox row.
    assert_eq!(
        store.run_status("run-old-terminal").expect("status"),
        Some(RunStatus::Terminal)
    );
    assert_eq!(
        store
            .terminal_reason("run-old-terminal")
            .expect("reason")
            .as_deref(),
        Some("outcome_unknown")
    );
    let remaining = store.load_events("run-old-terminal").expect("events");
    assert_eq!(remaining.len(), 1, "only the run_terminal entry survives");
    assert!(matches!(
        remaining[0].kind,
        RunEventKind::RunTerminal { .. }
    ));
    assert!(
        store
            .pending_effect("run-old-terminal", NODE)
            .expect("pending")
            .is_some(),
        "a pending outbox row is never garbage"
    );

    // `keep_last` spared the most recent closed run; the running run was
    // never a candidate.
    assert_eq!(
        store.run_status("run-recent-completed").expect("status"),
        Some(RunStatus::Completed)
    );
    assert_eq!(
        store.run_status("run-running").expect("status"),
        Some(RunStatus::Running)
    );

    // Idempotent: a second pass counts nothing new.
    let again = store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: 2_000,
                keep_last: 1,
            },
        )
        .expect("gc again");
    assert_eq!(again, GcOutcome::default());

    // A run closed WITHOUT a clock (closed_at NULL) is never a candidate.
    make_completed_run(&store, "run-clockless", 100);
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE runs SET closed_at_ms = NULL WHERE run_id = 'run-clockless'",
        [],
    )
    .expect("clear closed_at");
    drop(conn);
    let none = store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
        )
        .expect("gc clockless");
    // (run-recent-completed becomes purgeable under this looser policy —
    // only the clockless run must survive it.)
    assert_eq!(none.completed_purged, 1);
    assert_eq!(
        store.run_status("run-clockless").expect("status"),
        Some(RunStatus::Completed),
        "no close date, no purge"
    );

    // A COMPLETED run holding a pending outbox row (anomaly) is SKIPPED and
    // counted, never half-purged.
    make_completed_run(&store, "run-anomaly", 200);
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE outbox SET state = 'pending', receipt = NULL WHERE run_id = 'run-anomaly'",
        [],
    )
    .expect("forge pending");
    drop(conn);
    let skipped = store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
        )
        .expect("gc anomaly");
    assert_eq!(skipped.skipped_pending_outbox, 1);
    assert_eq!(
        store.run_status("run-anomaly").expect("status"),
        Some(RunStatus::Completed),
        "a completed run with a pending row must stay intact"
    );
}

/// RED CONTROL — the unguarded purge door reaches the exact bad states the
/// production guards prevent (a running run amputated, a pending row gone),
/// proving those guards are load-bearing and the green arms above are not
/// vacuous.
#[test]
fn store_gc_unguarded_purge_reaches_the_bad_state() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-running", "digest").expect("running");
    complete_one_step(&store, "run-running", NODE);

    store
        .gc_purge_run_unguarded("run-running")
        .expect("unguarded purge");
    assert_eq!(
        store.run_status("run-running").expect("status"),
        None,
        "without the guards a RUNNING run is destroyed — the bad state is real"
    );
}

/// Per-run purge is ONE transaction: a crash inside it rolls the whole run
/// back — never a journal amputated while the runs row survives.
#[test]
fn store_gc_crash_mid_purge_rolls_back_the_whole_run() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-crash", 1_000);
    let events_before = store.event_count("run-crash").expect("events");

    let err = store
        .gc_with_faults(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
            &HaltAt(FaultPoint::InsideGcTransaction),
        )
        .expect_err("halt inside the purge");
    assert!(
        matches!(
            err,
            StoreError::HaltedInTransaction {
                at: FaultPoint::InsideGcTransaction
            }
        ),
        "got {err:?}"
    );
    assert_eq!(
        store.run_status("run-crash").expect("status"),
        Some(RunStatus::Completed),
        "the runs row must have survived the rollback"
    );
    assert_eq!(
        store.event_count("run-crash").expect("events"),
        events_before,
        "the journal must be fully intact after the rollback"
    );
}

// ── inbox ─────────────────────────────────────────────────────────────────

/// At-least-once + dedup: a redelivery collapses on the unique key. The
/// `false` return IS the discriminating assertion — dropping the primary
/// key (the dedup) would make the second insert succeed and turn it `true`.
#[test]
fn store_inbox_dedup_collapses_redelivery() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    store.ensure_run("run-inbox", "digest").expect("ensure");

    assert!(
        store
            .record_inbox_message("run-inbox", "msg-1", 10)
            .expect("first delivery")
    );
    assert!(
        !store
            .record_inbox_message("run-inbox", "msg-1", 20)
            .expect("redelivery"),
        "a redelivery must be collapsed by the unique key"
    );
    assert_eq!(store.inbox_count("run-inbox").expect("count"), 1);

    // A different message id is new.
    assert!(
        store
            .record_inbox_message("run-inbox", "msg-2", 30)
            .expect("second message")
    );

    // A message for a run that does not exist is a FOREIGN KEY error —
    // `OR IGNORE` does not swallow FK violations.
    let err = store
        .record_inbox_message("run-ghost", "msg-1", 40)
        .expect_err("unknown run");
    assert!(matches!(err, StoreError::Sqlite(_)), "got {err:?}");

    // The identity gate holds for message ids too.
    let err = store
        .record_inbox_message("run-inbox", "bad\tid", 50)
        .expect_err("control character");
    assert!(
        matches!(err, StoreError::InvalidInput { .. }),
        "got {err:?}"
    );
}

// ── backup / restore ──────────────────────────────────────────────────────

/// Backup → restore round-trips the exact journal of the backup moment; a
/// restore REWINDS (anti-rollback surface, stated); invalid and too-new
/// backups are refused before anything is touched.
#[test]
fn store_backup_restore_round_trip() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-1", 1_000);
    let events_at_backup = store.load_events("run-1").expect("events");

    let backup = c._dir.path().join("backup.sqlite3");

    // A stale .tmp from an interrupted earlier copy is replaced, and the
    // final destination is never a partial file.
    std::fs::write(c._dir.path().join("backup.sqlite3.tmp"), b"garbage").expect("stale tmp");
    store.backup_into(&backup).expect("backup");

    // An existing destination is refused, never overwritten.
    let err = store
        .backup_into(&backup)
        .expect_err("existing destination");
    assert!(
        matches!(err, StoreError::BackupDestinationExists { .. }),
        "got {err:?}"
    );

    // The store moves on AFTER the backup…
    make_completed_run(&store, "run-2", 2_000);
    drop(store);

    // …and the restored copy is the BACKUP moment: run-1 intact,
    // run-2 absent (the rewind is the documented anti-rollback surface).
    let restored_path = c._dir.path().join("restored.sqlite3");
    RunStore::restore_from_backup(&backup, &restored_path).expect("restore");
    let restored = RunStore::open(&restored_path).expect("open restored");
    assert_eq!(
        restored.load_events("run-1").expect("events"),
        events_at_backup
    );
    assert_eq!(restored.run_status("run-2").expect("status"), None);
    drop(restored);

    // An arbitrary file is not a backup.
    let junk = c._dir.path().join("junk.bin");
    std::fs::write(&junk, b"not a database").expect("junk");
    let err =
        RunStore::restore_from_backup(&junk, &restored_path).expect_err("junk must be refused");
    assert!(
        matches!(err, StoreError::InvalidBackup { .. }),
        "got {err:?}"
    );

    // A backup from a NEWER schema is refused BEFORE anything is touched —
    // the same downgrade guard as open().
    let too_new = c._dir.path().join("too_new.sqlite3");
    std::fs::copy(&backup, &too_new).expect("copy");
    let conn = rusqlite::Connection::open(&too_new).expect("raw open");
    conn.pragma_update(None, "user_version", 99).expect("bump");
    drop(conn);
    let err = RunStore::restore_from_backup(&too_new, &restored_path)
        .expect_err("too-new backup must be refused");
    assert!(
        matches!(err, StoreError::UnsupportedSchemaVersion { .. }),
        "got {err:?}"
    );
}

/// Lease lifecycle: a successful renewal EXTENDS the expiry (the takeover
/// heuristic moves), and releasing marks the row FREE while PRESERVING the
/// monotone epoch — a re-acquisition after a release always bumps, so a
/// zombie handle from before the release can never pass the CAS again.
#[test]
fn store_single_writer_renew_release_keep_epoch_monotone() {
    let c = ctx();
    let mut a = RunStore::open(&c.db).expect("open a");

    // Input guards (the symmetric ones message_id already has).
    let err = a
        .acquire_lease("writer-a", 0, 0)
        .expect_err("ttl must be positive");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "lease ttl",
                ..
            }
        ),
        "got {err:?}"
    );
    let err = a
        .acquire_lease("bad\towner", 0, 1_000)
        .expect_err("control character in owner");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "lease owner",
                ..
            }
        ),
        "got {err:?}"
    );
    let err = a
        .acquire_lease("writer-a", i64::MAX, 1_000)
        .expect_err("now + ttl overflow");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "lease ttl",
                ..
            }
        ),
        "got {err:?}"
    );
    let err = a
        .acquire_lease("writer-a", -1, 1_000)
        .expect_err("negative clock");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "lease clock",
                ..
            }
        ),
        "got {err:?}"
    );

    // Successful renewal extends the expiry (and the clock guard holds on
    // the renew path too, ADV-1).
    assert_eq!(a.acquire_lease("writer-a", 0, 1_000).expect("acquire"), 1);
    let err = a
        .renew_lease(-1, 1_000)
        .expect_err("negative clock on renew");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "lease clock",
                ..
            }
        ),
        "got {err:?}"
    );
    a.renew_lease(500, 1_000).expect("renew");
    let lease = a.lease_state().expect("state").expect("held");
    assert_eq!(lease.expires_at_ms, 1_500, "renewal must extend the expiry");
    // And an aspiring owner is still refused before the NEW expiry.
    let mut b = RunStore::open(&c.db).expect("open b");
    let err = b
        .acquire_lease("writer-b", 1_200, 1_000)
        .expect_err("renewed lease still held");
    assert!(
        matches!(
            err,
            StoreError::LeaseHeld {
                expires_at_ms: 1_500,
                ..
            }
        ),
        "got {err:?}"
    );

    // Release frees the lease (no holder reported) but the handle keeps
    // writing UNFENCED afterwards — the documented spike posture.
    a.release_lease().expect("release");
    assert_eq!(a.active_lease_epoch(), None);
    assert!(
        a.lease_state().expect("state").is_none(),
        "released = no holder"
    );
    a.ensure_run("run-unfenced", "digest")
        .expect("an unfenced handle still writes");

    // THE MONOTONICITY GUARANTEE: re-acquiring after a release NEVER
    // restarts the epoch at 1 — the freed row kept it, so the zombie
    // scenario (old handle at (owner, 1) matching a fresh epoch-1 row) is
    // structurally impossible.
    let mut a2 = RunStore::open(&c.db).expect("open a2");
    assert_eq!(
        a2.acquire_lease("writer-a", 2_000, 1_000)
            .expect("re-acquire after release"),
        2,
        "the epoch must keep climbing across release/re-acquire"
    );
    // The pre-release zombie (writer-a, epoch 1) is fenced out.
    let (zombie_owner, zombie_epoch) = ("writer-a", 1);
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    let matched = conn
        .execute(
            "UPDATE lease SET epoch = epoch WHERE id = 1 AND owner = ?1 AND epoch = ?2",
            rusqlite::params![zombie_owner, zombie_epoch],
        )
        .expect("cas probe");
    assert_eq!(
        matched, 0,
        "a zombie epoch from before the release must never match again"
    );
}

/// Restore OVER an existing store deletes its stale `-wal`/`-shm` siblings
/// (they belong to the REPLACED database; SQLite replaying them over the
/// restored file would corrupt it) — the real-world restore scenario the
/// round-trip test does not exercise.
#[test]
fn store_backup_restore_overwrites_live_siblings() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-1", 1_000);
    let events_at_backup = store.load_events("run-1").expect("events");
    let backup = c._dir.path().join("backup.sqlite3");
    store.backup_into(&backup).expect("backup");

    // The store keeps living AFTER the backup, then closes.
    make_completed_run(&store, "run-2", 2_000);
    drop(store);

    // Plant sibling files where the replaced store's WAL would live —
    // whether left by a crash or by an unclean copy, they must NOT survive
    // the restore.
    let name = c.db.file_name().unwrap().to_string_lossy().into_owned();
    let wal = c.db.with_file_name(format!("{name}-wal"));
    let shm = c.db.with_file_name(format!("{name}-shm"));
    std::fs::write(&wal, b"stale wal of the replaced database").expect("plant wal");
    std::fs::write(&shm, b"stale shm").expect("plant shm");

    RunStore::restore_from_backup(&backup, &c.db).expect("restore over existing store");
    assert!(
        !wal.exists(),
        "the stale -wal sibling must be deleted by the restore"
    );
    assert!(
        !shm.exists(),
        "the stale -shm sibling must be deleted by the restore"
    );

    let restored = RunStore::open(&c.db).expect("reopen restored");
    assert_eq!(
        restored.load_events("run-1").expect("events"),
        events_at_backup
    );
    assert_eq!(
        restored.run_status("run-2").expect("status"),
        None,
        "the restore rewound to the backup moment"
    );
}

/// The GC re-reads status/eligibility INSIDE each purge transaction: a run
/// whose diagnosis lands BETWEEN the candidate scan and its purge is
/// re-dispatched on the CURRENT row — the terminal diagnosis survives
/// (Codex r1 P1: acting on the scanned status deleted it through the
/// completed branch).
#[test]
fn store_gc_rereads_status_inside_purge_transaction() {
    use std::cell::Cell;

    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-flip", 1_000);

    /// At the interleaving point (outside any transaction), flip the run to
    /// TERMINAL through a SEPARATE raw connection — the concurrent writer
    /// the re-read guard exists for.
    struct FlipToTerminal<'a> {
        db: &'a std::path::Path,
        fired: Cell<bool>,
    }
    impl FaultHook for FlipToTerminal<'_> {
        fn hit(&self, point: FaultPoint) -> FaultAction {
            if point == FaultPoint::BeforeGcPurgeOfRun && !self.fired.get() {
                self.fired.set(true);
                let conn = rusqlite::Connection::open(self.db).expect("raw open");
                conn.execute(
                    "UPDATE runs SET status = 'terminal', terminal_reason = 'outcome_unknown'
                     WHERE run_id = 'run-flip'",
                    [],
                )
                .expect("flip status");
                conn.execute(
                    "INSERT INTO run_events
                         (run_id, sequence, format_version, kind, node_instance_id, request_hash, payload)
                     VALUES ('run-flip', 99, 1, 'run_terminal', NULL, NULL,
                             '{\"kind\":\"run_terminal\",\"reason\":\"outcome_unknown\",\"expected_hash\":null,\"got_hash\":null}')",
                    [],
                )
                .expect("terminal event");
            }
            FaultAction::Continue
        }
    }

    let hook = FlipToTerminal {
        db: &c.db,
        fired: Cell::new(false),
    };
    let outcome = store
        .gc_with_faults(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
            &hook,
        )
        .expect("gc");
    assert!(hook.fired.get(), "the interleaving must have happened");
    // Re-dispatched through the TERMINAL branch: the diagnosis survives.
    assert_eq!(outcome.terminal_purged, 1);
    assert_eq!(outcome.completed_purged, 0);
    assert_eq!(
        store.run_status("run-flip").expect("status"),
        Some(RunStatus::Terminal),
        "the runs row (diagnosis) must survive the interleaved purge"
    );
    let remaining = store.load_events("run-flip").expect("events");
    assert_eq!(remaining.len(), 1);
    assert!(matches!(
        remaining[0].kind,
        RunEventKind::RunTerminal { .. }
    ));
}

/// GC input guards: a negative age (or clock) would move the cutoff into
/// the future and purge protected runs — refused, never clamped.
#[test]
fn store_gc_refuses_negative_age_and_clock() {
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    let err = store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: -1,
                keep_last: 0,
            },
        )
        .expect_err("negative age");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "gc retention age",
                ..
            }
        ),
        "got {err:?}"
    );
    let err = store
        .gc(
            -1,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
        )
        .expect_err("negative clock");
    assert!(
        matches!(
            err,
            StoreError::InvalidInput {
                what: "gc clock",
                ..
            }
        ),
        "got {err:?}"
    );
}

/// Backup validation is an INVENTORY check, not a name check: a
/// syntactically-valid SQLite file with the right user_version and a lone
/// `runs` table must NOT be allowed to destructively replace a healthy
/// store (Codex r1 P1). And the snapshot digest is bound to its OWNING run:
/// copying another run's intact tuple+digest verifies nowhere.
#[test]
fn store_backup_forged_minimal_refused_and_snapshot_bound_to_run() {
    let c = ctx();

    // Forged "backup": right user_version, one table, nothing else.
    let forged = c._dir.path().join("forged.sqlite3");
    let conn = rusqlite::Connection::open(&forged).expect("forge");
    conn.execute_batch("CREATE TABLE runs (run_id TEXT PRIMARY KEY); PRAGMA user_version = 2;")
        .expect("forge schema");
    drop(conn);
    let dest = c._dir.path().join("restored.sqlite3");
    let err = RunStore::restore_from_backup(&forged, &dest).expect_err("forged must be refused");
    assert!(
        matches!(err, StoreError::InvalidBackup { .. }),
        "got {err:?}"
    );

    // Backup alias guard: the live store file (and WAL siblings) are
    // refused as destinations.
    let store = RunStore::open(&c.db).expect("open");
    make_completed_run(&store, "run-a", 1_000);
    let err = store
        .backup_into(&c.db)
        .expect_err("alias of the live store");
    assert!(
        matches!(
            err,
            StoreError::BackupDestinationExists { .. } | StoreError::InvalidInput { .. }
        ),
        "got {err:?}"
    );
    let name = c.db.file_name().unwrap().to_string_lossy().into_owned();
    let wal_alias = c.db.with_file_name(format!("{name}-wal"));
    let err = store
        .backup_into(&wal_alias)
        .expect_err("alias of the wal sibling");
    assert!(
        matches!(
            err,
            StoreError::BackupDestinationExists { .. } | StoreError::InvalidInput { .. }
        ),
        "got {err:?}"
    );

    // Snapshot cross-run substitution: copy run-a's intact tuple+digest
    // into run-b's row — the run-bound digest detects it.
    make_completed_run(&store, "run-b", 2_000);
    let state = serde_json::json!({"steps": 1});
    store.write_snapshot("run-a", 3, &state).expect("snap a");
    store.write_snapshot("run-b", 3, &state).expect("snap b");
    let conn = rusqlite::Connection::open(&c.db).expect("raw open");
    conn.execute(
        "UPDATE snapshots SET state = (SELECT state FROM snapshots WHERE run_id = 'run-a'),
                              state_digest = (SELECT state_digest FROM snapshots WHERE run_id = 'run-a'),
                              upto_sequence = (SELECT upto_sequence FROM snapshots WHERE run_id = 'run-a'),
                              snapshot_version = (SELECT snapshot_version FROM snapshots WHERE run_id = 'run-a')
         WHERE run_id = 'run-b'",
        [],
    )
    .expect("cross-run copy");
    drop(conn);
    assert_eq!(
        store.read_snapshot("run-b").expect("read"),
        SnapshotRead::Poisoned,
        "another run's tuple+digest must not verify here"
    );
    // run-a's own snapshot still verifies (the copy went one way).
    assert!(matches!(
        store.read_snapshot("run-a").expect("read"),
        SnapshotRead::Valid(_)
    ));
}

/// Codex r2 closures, all four in one arm: (1) closed states are ABSORBING
/// (a terminal run cannot be re-labelled completed, so GC can never route
/// its diagnosis through the completed branch); (2) a forged backup with
/// the six right-named tables but alien columns is refused; (3) the
/// `keep_last` window is re-ranked INSIDE the purge transaction; (4) a
/// negative snapshot sequence reads as Poisoned (ignorable cache), never a
/// hard error.
#[test]
fn store_gc_absorbing_states_and_backup_column_inventory() {
    use std::cell::Cell;

    // (1) Absorbing transitions, both directions.
    let c = ctx();
    let store = RunStore::open(&c.db).expect("open");
    make_terminal_run_with_pending(&store, "run-term", 1_000);
    let err = store
        .record_run_completed("run-term", Some(2_000))
        .expect_err("a terminal run must not close as completed");
    assert!(
        matches!(err, StoreError::InvalidTransition { .. }),
        "got {err:?}"
    );
    make_completed_run(&store, "run-done", 1_000);
    let err = store
        .record_terminal(
            "run-done",
            None,
            TerminalReason::OutcomeUnknown,
            None,
            None,
            Some(2_000),
        )
        .expect_err("a completed run must not go terminal");
    assert!(
        matches!(err, StoreError::InvalidTransition { .. }),
        "got {err:?}"
    );
    // The diagnosis is therefore un-purgeable via re-labelling: GC still
    // sees run-term as terminal and preserves its row + entry.
    store
        .gc(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 0,
            },
        )
        .expect("gc");
    assert_eq!(
        store.run_status("run-term").expect("status"),
        Some(RunStatus::Terminal)
    );

    // (4) Negative snapshot sequence: Poisoned, not an error.
    store
        .write_snapshot("run-term", 1, &serde_json::json!({"s": 1}))
        .expect("snap");
    let conn = rusqlite::Connection::open(&c.db).expect("raw");
    conn.execute(
        "UPDATE snapshots SET upto_sequence = -1 WHERE run_id = 'run-term'",
        [],
    )
    .expect("tamper");
    drop(conn);
    assert_eq!(
        store.read_snapshot("run-term").expect("read"),
        SnapshotRead::Poisoned
    );

    // (2) Forged backup: six right names, alien columns.
    let forged = c._dir.path().join("forged6.sqlite3");
    let conn = rusqlite::Connection::open(&forged).expect("forge");
    conn.execute_batch(
        "CREATE TABLE runs (x TEXT); CREATE TABLE run_events (x TEXT);
         CREATE TABLE outbox (x TEXT); CREATE TABLE inbox (x TEXT);
         CREATE TABLE snapshots (x TEXT); CREATE TABLE lease (x TEXT);
         CREATE UNIQUE INDEX outbox_one_effect_per_node ON outbox (x);
         PRAGMA user_version = 2;",
    )
    .expect("forge schema");
    drop(conn);
    let dest = c._dir.path().join("restored6.sqlite3");
    let err = RunStore::restore_from_backup(&forged, &dest)
        .expect_err("six right-named alien tables must be refused");
    assert!(
        matches!(err, StoreError::InvalidBackup { .. }),
        "got {err:?}"
    );

    // (2b) SEMANTIC index validation: a REAL backup whose unique index is
    // swapped for a right-named non-unique homonym is refused — names are
    // not the index (Codex r3 P1).
    let degraded = c._dir.path().join("degraded.sqlite3");
    store.backup_into(&degraded).expect("real backup");
    let conn = rusqlite::Connection::open(&degraded).expect("open degraded");
    conn.execute_batch(
        "DROP INDEX outbox_one_effect_per_node;
         CREATE INDEX outbox_one_effect_per_node ON outbox (run_id, node_instance_id);",
    )
    .expect("degrade index");
    drop(conn);
    let err = RunStore::restore_from_backup(&degraded, &dest)
        .expect_err("a non-unique homonym index must be refused");
    assert!(
        matches!(err, StoreError::InvalidBackup { ref reason } if reason.contains("not UNIQUE")),
        "got {err:?}"
    );
    // A PARTIAL unique homonym is refused too (Codex r4): its predicate
    // only deduplicates the rows it covers, not the invariant.
    let conn = rusqlite::Connection::open(&degraded).expect("open degraded");
    conn.execute_batch(
        "DROP INDEX outbox_one_effect_per_node;
         CREATE UNIQUE INDEX outbox_one_effect_per_node
             ON outbox (run_id, node_instance_id) WHERE state = 'applied';",
    )
    .expect("partial index");
    drop(conn);
    let err = RunStore::restore_from_backup(&degraded, &dest)
        .expect_err("a partial homonym index must be refused");
    assert!(
        matches!(err, StoreError::InvalidBackup { ref reason } if reason.contains("PARTIAL")),
        "got {err:?}"
    );

    // (3) keep_last re-ranked inside the transaction: the scanned victim
    // becomes the most recent run at the interleaving point and must be
    // spared by the re-ranked protected window.
    let c2 = ctx();
    let store2 = RunStore::open(&c2.db).expect("open");
    make_completed_run(&store2, "run-old", 1_000);
    make_completed_run(&store2, "run-new", 5_000);

    struct Redate<'a> {
        db: &'a std::path::Path,
        fired: Cell<bool>,
    }
    impl FaultHook for Redate<'_> {
        fn hit(&self, point: FaultPoint) -> FaultAction {
            if point == FaultPoint::BeforeGcPurgeOfRun && !self.fired.get() {
                self.fired.set(true);
                let conn = rusqlite::Connection::open(self.db).expect("raw");
                conn.execute(
                    "UPDATE runs SET closed_at_ms = 9_000 WHERE run_id = 'run-old'",
                    [],
                )
                .expect("re-date");
            }
            FaultAction::Continue
        }
    }
    let hook = Redate {
        db: &c2.db,
        fired: Cell::new(false),
    };
    // Scan (keep_last = 1) selects run-old as the purge victim; the hook
    // then makes run-old the most recent — the re-ranked window spares it.
    let outcome = store2
        .gc_with_faults(
            10_000,
            &RetentionPolicy {
                min_age_ms: 0,
                keep_last: 1,
            },
            &hook,
        )
        .expect("gc");
    assert!(hook.fired.get());
    assert_eq!(outcome.completed_purged, 0, "the re-dated run is protected");
    assert_eq!(
        store2.run_status("run-old").expect("status"),
        Some(RunStatus::Completed)
    );
}
