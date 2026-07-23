// SPDX-License-Identifier: AGPL-3.0-or-later
//! The run store: append-only journal, run frontier and transactional
//! outbox, in one embedded SQLite database.
//!
//! ## The rule the whole spike rests on
//!
//! An entry is appended to the journal, the run sequence (and, on
//! completion, the frontier) is advanced, and the outbox row is written — in
//! ONE transaction. Never two. A split would
//! create a state where an effect is queued but no journal entry knows about
//! it (or the reverse), and recovery would either drop the effect or emit it
//! twice. A degraded constructor splitting the two writes exists solely to
//! make that failure observable in a test, and lives behind the
//! `spike-faults` feature — a normal build cannot reach it.
//!
//! ## Durability posture
//!
//! `journal_mode = WAL` + `synchronous = FULL` + `foreign_keys = ON` +
//! a 5s busy timeout, mirroring the coordinator database
//! (`nexus-coordinator-rs/src/db.rs`). `synchronous = FULL` is asserted by a
//! pragma-contract test because it cannot be proven by a crash harness:
//! SQLite commits are durable across application crashes at any
//! `synchronous` level, and only a power loss discriminates.
//!
//! ## Host neutrality
//!
//! [`RunStore::open`] takes the path from its caller. The crate resolves no
//! home directory and reads no environment variable, and the store is never
//! opened in memory: an in-memory database has neither WAL nor `-wal`/`-shm`
//! recovery, so a harness built on it would prove nothing about restart.
//!
//! ## The full run store (Sprint 83 Phase I)
//!
//! On top of the spike's journal + outbox, Phase I added — forward-only, as
//! appended migrations, never editing an existing `M::up`:
//!
//! - **Interpretation-time upcasting + downgrade guard** — `load_events`
//!   reads the payload as a `Value`, runs the pure upcaster chain
//!   (`sbfb_workflow_core::run_event::upcast_payload`), then parses the
//!   CURRENT `RunEventKind`. A row written by a NEWER engine is refused
//!   fail-closed, never mis-read; a row whose version has no upcasting path
//!   is a loud error, never a silent skip.
//! - **Single-writer lease fencing** — a one-row `lease` table with a
//!   MONOTONE `epoch`. Safety comes from a compare-and-set inside every
//!   write transaction (`UPDATE … WHERE owner = ? AND epoch = ?`,
//!   `rows_affected == 1` or the transaction aborts), NEVER from the
//!   wall-clock expiry: `expires_at_ms` is only the heuristic an ASPIRING
//!   owner uses to decide the previous holder is dead and bump the epoch
//!   (Kleppmann fencing tokens; Temporal `rangeID`). `BEGIN IMMEDIATE`
//!   already serialises simultaneous write transactions on one file — the
//!   fence exists for the STALE owner, the process that still believes it
//!   holds a lease that was taken over.
//! - **Inbox** — at-least-once + dedup on `(run_id, message_id)` via
//!   `INSERT OR IGNORE` under a unique key, the S76 Phase D
//!   `task_results UNIQUE (task_id, worker_id)` precedent, plus the
//!   standard `processed_at_ms` observability column.
//! - **Disposable snapshots** — a per-run cache of a reduction of the log,
//!   invalidated by `snapshot_version` and a content digest; NEVER a source
//!   of truth. A poisoned or stale snapshot reads as discarded, and the
//!   journal always re-derives the same state.
//! - **Retention / GC** — only CLOSED runs (`completed`/`terminal`) older
//!   than an explicit horizon are purged, sparing the most recent
//!   `keep_last`. Never purged: a running run, a pending outbox row, the
//!   terminal diagnosis (the `run_terminal` entry and the `runs` row of a
//!   terminal run survive every pass). GC is an accepted, POLICY-BOUNDED
//!   loss of audit evidence — stated, not hidden.
//! - **Backup / restore** — `VACUUM INTO` (SQL, transactionally-consistent
//!   snapshot; the `backup` cargo feature stays off) written to a temporary
//!   sibling then atomically renamed; restore validates the backup file and
//!   replaces the CLOSED store, deleting stale `-wal`/`-shm` siblings. A
//!   point-in-time snapshot, not PITR — and restore is by nature an
//!   anti-rollback surface (THREAT_MODEL A11).
//!
//! Wall-clock values (`now_ms`) are INJECTED by the caller everywhere: the
//! crate stays host-neutral (no clock read), tests stay deterministic, and
//! the journal itself keeps carrying no time at all.

use std::path::{Path, PathBuf};

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use rusqlite_migration::{M, Migrations};

use crate::event::{
    RUN_EVENT_FORMAT_VERSION, RUN_EVENT_UPCASTERS, RunEvent, RunEventKind, RunEventUpcaster,
    TerminalReason, UpcastError, upcast_payload,
};
use crate::fault::{FaultAction, FaultHook, FaultPoint, NoFaults};

/// Schema of the run store. FORWARD-ONLY: every phase APPENDS an `M::up`,
/// never edits an existing one (the S82 Phase T per-class discipline). The
/// first migration is the Phase E spike; the second is the Phase I full run
/// store. Table names (`runs`, `run_events`, `outbox`) were chosen in E to
/// be extensible without a DROP — Phase I banks exactly that.
static MIGRATIONS: &[M<'static>] = &[
    M::up(
        "CREATE TABLE runs (
         run_id          TEXT PRIMARY KEY,
         status          TEXT NOT NULL,
         next_sequence   INTEGER NOT NULL,
         spec_digest     TEXT NOT NULL,
         frontier_node   TEXT,
         terminal_reason TEXT
     );
     CREATE TABLE run_events (
         run_id           TEXT NOT NULL,
         sequence         INTEGER NOT NULL,
         format_version   INTEGER NOT NULL,
         kind             TEXT NOT NULL,
         node_instance_id TEXT,
         request_hash     TEXT,
         payload          TEXT NOT NULL,
         PRIMARY KEY (run_id, sequence),
         FOREIGN KEY (run_id) REFERENCES runs(run_id)
     );
     CREATE TABLE outbox (
         run_id           TEXT NOT NULL,
         node_instance_id TEXT NOT NULL,
         effect_id        TEXT NOT NULL,
         state            TEXT NOT NULL,
         intent_sequence  INTEGER,
         request_hash     TEXT NOT NULL,
         target_idempotent INTEGER NOT NULL,
         dedup_scope      TEXT NOT NULL,
         receipt          TEXT,
         PRIMARY KEY (run_id, node_instance_id, effect_id),
         FOREIGN KEY (run_id) REFERENCES runs(run_id)
     );
     -- The reducer assumes at most ONE effect per node instance: it reads the
     -- memoised and the pending row with a single-row query. The primary key
     -- above does NOT enforce that (it includes effect_id), so the assumption
     -- is made fail-closed here instead of being trusted. Without it, two rows
     -- for one node would make the memoised answer non-deterministic and leave
     -- the other effect unresolved forever, silently.
     CREATE UNIQUE INDEX outbox_one_effect_per_node
         ON outbox (run_id, node_instance_id);",
    ),
    // Phase I — the full run store. Integer columns are i64 (native SQLite;
    // never u64/usize, which rusqlite 0.38 stops converting by default —
    // forward-compat for the QPO-3 maintenance bump).
    M::up(
        "-- Single-writer fencing. One row; `epoch` is the MONOTONE fencing
         -- token bumped at every acquisition. Safety = the epoch CAS inside
         -- every write transaction; `expires_at_ms` is ONLY the takeover
         -- heuristic an aspiring owner consults, never a commit condition.
         CREATE TABLE lease (
             id            INTEGER PRIMARY KEY CHECK (id = 1),
             owner         TEXT    NOT NULL,
             epoch         INTEGER NOT NULL,
             expires_at_ms INTEGER NOT NULL
         );
         -- At-least-once inbox: redelivery collapses on the unique key
         -- (S76 D precedent, `task_results UNIQUE (task_id, worker_id)` +
         -- INSERT OR IGNORE), `processed_at_ms` for observability.
         CREATE TABLE inbox (
             run_id          TEXT NOT NULL,
             message_id      TEXT NOT NULL,
             processed_at_ms INTEGER NOT NULL,
             PRIMARY KEY (run_id, message_id),
             FOREIGN KEY (run_id) REFERENCES runs(run_id)
         );
         -- Disposable per-run cache of a reduction of the log. NEVER a
         -- source of truth: `snapshot_version` invalidates the format,
         -- `state_digest` detects poison, and dropping the row must never
         -- change any observable behaviour.
         CREATE TABLE snapshots (
             run_id           TEXT NOT NULL,
             snapshot_version INTEGER NOT NULL,
             upto_sequence    INTEGER NOT NULL,
             state            TEXT NOT NULL,
             state_digest     TEXT NOT NULL,
             PRIMARY KEY (run_id),
             FOREIGN KEY (run_id) REFERENCES runs(run_id)
         );
         -- When a run CLOSED (completed or terminal), for retention only.
         -- NULL = never closed OR closed by a caller without a clock; a run
         -- without a close time is NEVER purged (fail-safe: no date, no GC).
         -- State column, not journal: run_events stays clock-free.
         ALTER TABLE runs ADD COLUMN closed_at_ms INTEGER;",
    ),
    // Phase J — the orchestrator's durable state: open waits, deposited
    // signals, absolute-deadline timers, and the run's material input.
    // Same discipline as M2: forward-only append, i64 columns, wall-clock
    // values in STATE tables only (the journal stays clock-free).
    M::up(
        "-- One OPEN interruption per node instance (wait.signal or
         -- human.review): the engine-derived mono-usage nonce a deposit
         -- must name. `status` flips to consumed in the SAME transaction
         -- that journals the consumption — the §12.6 mono-usage rule is
         -- this row, not scheduler memory.
         CREATE TABLE waits (
             run_id           TEXT NOT NULL,
             node_instance_id TEXT NOT NULL,
             signal_name      TEXT NOT NULL,
             nonce            TEXT NOT NULL,
             status           TEXT NOT NULL,
             PRIMARY KEY (run_id, node_instance_id),
             FOREIGN KEY (run_id) REFERENCES runs(run_id)
         );
         -- Deposited signals/decisions. MATERIAL payload lives here (state
         -- table), never in the journal — the journal carries its digest.
         -- Redelivery of the same nonce collapses on the primary key
         -- (INSERT OR IGNORE, the inbox precedent). `expires_at_ms` is the
         -- DEPOSITOR's validity bound, checked at consumption time.
         CREATE TABLE signals (
             run_id          TEXT NOT NULL,
             nonce           TEXT NOT NULL,
             payload         TEXT NOT NULL,
             decision_label  TEXT,
             expires_at_ms   INTEGER,
             deposited_at_ms INTEGER NOT NULL,
             consumed        INTEGER NOT NULL,
             PRIMARY KEY (run_id, nonce),
             FOREIGN KEY (run_id) REFERENCES runs(run_id)
         );
         -- Durable timers: ABSOLUTE deadlines, never durations. A restart
         -- re-reads the original deadline instead of re-arming (the
         -- delay-as-remainder drift the prior-art scan flagged): INSERT OR
         -- IGNORE on schedule means the FIRST deadline written wins for a
         -- timer id, forever.
         CREATE TABLE timers (
             run_id      TEXT NOT NULL,
             timer_id    TEXT NOT NULL,
             deadline_ms INTEGER NOT NULL,
             fired       INTEGER NOT NULL,
             PRIMARY KEY (run_id, timer_id),
             FOREIGN KEY (run_id) REFERENCES runs(run_id)
         );
         -- The run's material input payload (state column: material data
         -- never reaches the journal; the spec digest binds it to the run).
         ALTER TABLE runs ADD COLUMN input_payload TEXT;
         -- Store-wide clock watermark: the highest now_ms any scheduling
         -- pass ever presented. A later pass with a SMALLER reading is
         -- refused LOUDLY before it can un-expire a deposit or re-judge a
         -- deadline (THREAT §19.1 timer skew — review r1 SD-2 closed the
         -- between-passes regression window the per-deposit guard left).
         CREATE TABLE clock_watermark (
             id         INTEGER PRIMARY KEY CHECK (id = 1),
             max_now_ms INTEGER NOT NULL
         );",
    ),
];

/// An effect is queued but not confirmed.
const OUTBOX_PENDING: &str = "pending";
/// An effect returned its receipt.
const OUTBOX_APPLIED: &str = "applied";

/// A wait is open: its nonce is spendable exactly once.
const WAIT_OPEN: &str = "open";

/// One deposit row as read by the consumption door:
/// `(payload, decision_label, expires_at_ms, deposited_at_ms, consumed)`.
type DepositRow = (String, Option<String>, Option<i64>, i64, i64);

/// Ceiling on one signal/decision payload's serialised size (256 KiB) —
/// the deposit-door counterpart of `IDENTITY_MAX_LEN` for material data.
pub const MAX_SIGNAL_PAYLOAD_BYTES: usize = 256 * 1024;
/// A wait was consumed — its nonce is spent forever.
const WAIT_CONSUMED: &str = "consumed";

/// Version of the snapshot FORMAT (the shape of `state`). A stored snapshot
/// with a different version is stale and reads as discarded — invalidation,
/// never migration: snapshots are a cache rebuilt from the journal.
pub const SNAPSHOT_FORMAT_VERSION: i64 = 1;

/// Context string of the snapshot digest. Since review D4-1 it covers the
/// WHOLE served tuple (version, sequence bound, state) — the historical
/// "state" in the context string stays as-is (changing it would invalidate
/// nothing but honest caches, yet any rename ritual here buys zero
/// security: the string is a domain separator, not a description).
const SNAPSHOT_STATE_DIGEST_CONTEXT: &str = "sbfb.workflow.engine.snapshot-state.v0";

/// Context string of the signal-payload digest journaled by a consumption
/// (`signal_consumed` / `human_decision_recorded`). Only the digest reaches
/// the journal; the payload stays in the `signals` state table.
const SIGNAL_PAYLOAD_DIGEST_CONTEXT: &str = "sbfb.workflow.engine.signal-payload.v0";

/// Where a run stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// Advancing, or resumable after a crash.
    Running,
    /// Every step is memoised and the run is closed.
    Completed,
    /// Stopped and not resumable (see [`TerminalReason`]).
    Terminal,
}

impl RunStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Terminal => "terminal",
        }
    }

    fn parse(raw: &str) -> Result<Self, StoreError> {
        match raw {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "terminal" => Ok(Self::Terminal),
            other => Err(StoreError::Corrupt(format!("unknown run status: {other}"))),
        }
    }
}

/// A step whose effect completed: replay returns this instead of acting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoisedEffect {
    /// Effect id recorded at intent time.
    pub effect_id: String,
    /// Step request hash recorded at intent time — compared against the
    /// freshly computed one on replay.
    pub request_hash: String,
    /// Receipt returned by the target. Material payloads live here, never in
    /// the journal.
    pub receipt: String,
}

/// A step whose effect was intended but never confirmed. The unresolved
/// frontier after a crash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEffect {
    /// Effect id recorded at intent time.
    pub effect_id: String,
    /// Step request hash recorded at intent time.
    pub request_hash: String,
    /// Whether the target declared itself idempotent WHEN THE INTENT WAS
    /// RECORDED. This journaled value is the one that AUTHORISES a
    /// re-application; the live target is still asked at resume time, but
    /// only to check its answer has not changed (a mismatch is fail-closed).
    /// Trusting the live answer alone would let a different target with the
    /// same name authorise a re-application it never promised to collapse.
    pub target_idempotent: bool,
    /// Identity of the durable store the target deduplicates in, recorded at
    /// intent time. A resume against a DIFFERENT backend must not be allowed
    /// to reuse the key: the receipt lives in the first one.
    pub dedup_scope: String,
}

/// The store failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// SQLite refused an operation.
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    /// The schema could not be brought up to date.
    #[error("migration: {0}")]
    Migration(#[from] rusqlite_migration::Error),
    /// A stored row cannot be interpreted. Loud rather than silently
    /// repaired: a journal that cannot be read must never be replaced by an
    /// empty one.
    #[error("run store corrupt: {0}")]
    Corrupt(String),
    /// SQLite did not honour a durability pragma. `pragma_update` CANNOT
    /// report this: when SQLite refuses a journal mode it returns the mode
    /// currently in force as a result row, which `execute_batch` discards. On
    /// a filesystem without shared-memory support the store would open in
    /// `delete` mode, with the documented durability posture quietly not in
    /// effect. The pragmas are therefore read back.
    #[error("pragma {pragma}: requested {requested}, in force {actual}")]
    PragmaRefused {
        /// Pragma name.
        pragma: String,
        /// What was asked for.
        requested: String,
        /// What SQLite actually applied.
        actual: String,
    },
    /// A fault hook halted the run INSIDE a transaction; everything the
    /// transaction had written is rolled back. Harness only.
    #[error("halted inside a transaction at {at:?}")]
    HaltedInTransaction {
        /// Where the transaction was interrupted.
        at: crate::fault::FaultPoint,
    },
    /// The store file's schema version is NEWER than this binary knows.
    /// Opening it anyway would run older code against newer tables — the
    /// schema-level half of the "downgrade dangereux" the plan refuses
    /// (the event-level half is [`StoreError::Upcast`]).
    #[error("store schema version {found} is newer than the {supported} known migrations")]
    UnsupportedSchemaVersion {
        /// `PRAGMA user_version` found in the file.
        found: i64,
        /// Number of migrations this binary carries.
        supported: usize,
    },
    /// A journal payload could not be brought to the current envelope
    /// version. [`UpcastError::NewerThanSupported`] is the event-level
    /// downgrade guard firing.
    #[error("journal upcast: {0}")]
    Upcast(#[from] UpcastError),
    /// The lease is held by another live owner. Wanting it is not a fault —
    /// the caller retries after `expires_at_ms` or walks away.
    #[error("lease held by {owner} until {expires_at_ms}")]
    LeaseHeld {
        /// Current holder.
        owner: String,
        /// When an aspiring owner may attempt a takeover (heuristic only).
        expires_at_ms: i64,
    },
    /// The epoch CAS failed inside a write transaction: this handle's lease
    /// was taken over. The whole transaction rolled back — a fenced-out
    /// writer emits NOTHING (no journal entry, no outbox row).
    #[error("lease lost: {owner} epoch {epoch} was fenced out")]
    LeaseLost {
        /// Owner this handle believed it was.
        owner: String,
        /// Epoch this handle held.
        epoch: i64,
    },
    /// Backup refuses to overwrite an existing destination.
    #[error("backup destination already exists: {path}")]
    BackupDestinationExists {
        /// Refused destination path.
        path: String,
    },
    /// The backup file failed validation before a restore.
    #[error("invalid backup: {reason}")]
    InvalidBackup {
        /// Why it was refused.
        reason: String,
    },
    /// Filesystem step of backup/restore failed.
    #[error("backup/restore io: {0}")]
    Io(#[from] std::io::Error),
    /// A closed run (completed or terminal) was asked to change state.
    /// Closed states are ABSORBING (Codex r2): without this, a terminal
    /// diagnosis could be re-labelled completed and then GC'd away.
    #[error("run {run_id} is {from} — closed states are absorbing, refusing {to}")]
    InvalidTransition {
        /// Run whose transition was refused.
        run_id: String,
        /// Status found in the row.
        from: String,
        /// Status the caller asked for.
        to: &'static str,
    },
    /// A caller-supplied argument is outside the accepted domain (lease
    /// owner, inbox message id, TTL, snapshot state, non-UTF-8 path…).
    #[error("invalid {what}: {reason}")]
    InvalidInput {
        /// Which argument.
        what: &'static str,
        /// Why it was refused.
        reason: String,
    },
    /// A signal consumption was refused — LOUD, with zero transition
    /// (THREAT §19 signal-forge/replay mitigation): unknown or spent
    /// nonce, expired deposit, or a decision label outside the node's
    /// closed set. Refusal is an ERROR to the depositor's driver, never a
    /// silent skip, so a forged or replayed delivery is observable.
    #[error("signal refused for {node_instance_id}: {reason}")]
    SignalRefused {
        /// Parked node whose wait refused the deposit.
        node_instance_id: String,
        /// Why it was refused.
        reason: String,
    },
    /// The injected clock went BACKWARDS below the store's durable
    /// watermark. Refused loudly for the whole scheduling pass: a
    /// regressive reading could otherwise un-expire a deposit judged
    /// expired by an earlier pass (THREAT §19.1 timer skew, review r1
    /// SD-2 — the per-deposit guard alone left this between-passes
    /// window).
    #[error("clock regression: now {now_ms} is below the durable watermark {watermark_ms}")]
    ClockRegression {
        /// The regressive reading.
        now_ms: i64,
        /// The highest reading any pass ever presented.
        watermark_ms: i64,
    },
}

/// One OPEN interruption of a run (Phase J): a `wait.signal` or
/// `human.review` node parked until a deposit names its nonce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWait {
    /// RUNTIME node instance id of the parked node (iteration suffixes
    /// included when the wait sits inside a map/repeat scope).
    pub node_instance_id: String,
    /// Static signal name of the awaited signal (`wait.signal`), or the
    /// engine's reserved decision channel (`human.review`).
    pub signal_name: String,
    /// The engine-derived mono-usage nonce a deposit must name.
    pub nonce: String,
}

/// A consumed deposit, returned by the atomic consumption door.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsumedSignal {
    /// Material payload of the deposit.
    pub payload: serde_json::Value,
    /// Decision label carried by the deposit, when any.
    pub decision_label: Option<String>,
}

/// The lease row, for inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseState {
    /// Current holder.
    pub owner: String,
    /// Monotone fencing token.
    pub epoch: i64,
    /// Takeover heuristic — NEVER a commit condition.
    pub expires_at_ms: i64,
}

/// A stored snapshot that passed every validity check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    /// Journal sequence (exclusive upper bound) the state reduces.
    pub upto_sequence: u64,
    /// The cached reduction, pure JSON data.
    pub state: serde_json::Value,
}

/// What reading a snapshot found. Only [`SnapshotRead::Valid`] carries
/// state; every other arm means "rebuild from the journal" — a snapshot is
/// a cache, so none of them is an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotRead {
    /// No snapshot stored for this run.
    Missing,
    /// Stored under a different snapshot format version — invalidated.
    Stale {
        /// Version found in the row.
        found_version: i64,
    },
    /// The state does not match its recorded digest (or does not parse):
    /// tampered with or corrupt. DETECTED and discarded, never trusted.
    Poisoned,
    /// Intact and current.
    Valid(Snapshot),
}

/// What GC may purge. Both bounds are explicit — there is no implicit
/// default policy, because GC is an accepted loss of audit evidence and the
/// caller must SAY what it accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionPolicy {
    /// Only runs closed at least this long ago are candidates.
    pub min_age_ms: i64,
    /// The most recently closed candidates spared unconditionally.
    pub keep_last: u32,
}

/// What one per-run purge transaction decided (internal).
enum GcAction {
    PurgedCompleted,
    PurgedTerminal,
    SkippedPendingOutbox,
    /// The in-transaction re-checks found nothing to purge: already reduced
    /// on a previous pass, vanished, re-dated out of eligibility, back to
    /// running, or protected by the re-ranked `keep_last` window. Counted
    /// nowhere — the purge stays idempotent.
    NotEligible,
}

/// What one GC pass did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GcOutcome {
    /// Completed runs fully purged.
    pub completed_purged: usize,
    /// Terminal runs purged down to their diagnosis (the `runs` row and the
    /// `run_terminal` entry survive).
    pub terminal_purged: usize,
    /// Candidates SKIPPED because a pending outbox row exists — a pending
    /// effect is never garbage.
    pub skipped_pending_outbox: usize,
}

/// Handle on one run store file.
///
/// A handle MAY hold an acquired lease ([`RunStore::acquire_lease`]); while
/// it does, EVERY write transaction re-asserts ownership with an epoch CAS
/// and aborts on [`StoreError::LeaseLost`]. A handle WITHOUT an active lease
/// writes unfenced — the Phase E single-process spike mode, still what the
/// spike-level engine tests use. The RUNTIME WRITER path acquires (the
/// Phase J orchestrator takes the lease before writing; the daemon's I1
/// routes stay lease-free READS); unfenced mode is a documented posture,
/// not a hidden default the caller can mistake for fencing.
pub struct RunStore {
    conn: Connection,
    active_lease: Option<(String, i64)>,
}

impl std::fmt::Debug for RunStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunStore").finish_non_exhaustive()
    }
}

impl RunStore {
    /// Open (creating if needed) the store at an INJECTED path.
    ///
    /// Every durability pragma is READ BACK after being set: see
    /// [`StoreError::PragmaRefused`] for why setting them cannot fail loudly
    /// on its own.
    ///
    /// The schema version is guarded BEFORE migrations run: a file whose
    /// `user_version` exceeds the migrations this binary carries was written
    /// by a NEWER engine, and opening it would be the schema-level downgrade
    /// the plan refuses ([`StoreError::UnsupportedSchemaVersion`]) — this
    /// also guards a restore from a too-new backup.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let mut store = Self {
            conn,
            active_lease: None,
        };
        store.assert_pragma_text("journal_mode", "wal")?;
        store.assert_pragma_i64("synchronous", 2, "FULL")?;
        store.assert_pragma_i64("foreign_keys", 1, "ON")?;

        let user_version = store.pragma_i64("user_version")?;
        if user_version > MIGRATIONS.len() as i64 {
            return Err(StoreError::UnsupportedSchemaVersion {
                found: user_version,
                supported: MIGRATIONS.len(),
            });
        }

        Migrations::new(MIGRATIONS.to_vec()).to_latest(&mut store.conn)?;

        Ok(store)
    }

    fn assert_pragma_text(&self, pragma: &str, expected: &str) -> Result<(), StoreError> {
        let actual = self.pragma_text(pragma)?;
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(StoreError::PragmaRefused {
                pragma: pragma.to_string(),
                requested: expected.to_string(),
                actual,
            });
        }
        Ok(())
    }

    fn assert_pragma_i64(
        &self,
        pragma: &str,
        expected: i64,
        requested_label: &str,
    ) -> Result<(), StoreError> {
        let actual = self.pragma_i64(pragma)?;
        if actual != expected {
            return Err(StoreError::PragmaRefused {
                pragma: pragma.to_string(),
                requested: requested_label.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(())
    }

    /// Read an integer pragma. Used by the durability contract test.
    pub fn pragma_i64(&self, name: &str) -> Result<i64, StoreError> {
        Ok(self.conn.pragma_query_value(None, name, |row| row.get(0))?)
    }

    /// Read a text pragma. Used by the durability contract test.
    pub fn pragma_text(&self, name: &str) -> Result<String, StoreError> {
        Ok(self.conn.pragma_query_value(None, name, |row| row.get(0))?)
    }

    /// Status of a run, or `None` when it does not exist yet.
    pub fn run_status(&self, run_id: &str) -> Result<Option<RunStatus>, StoreError> {
        let raw: Option<String> = self
            .conn
            .query_row(
                "SELECT status FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .optional()?;
        raw.map(|r| RunStatus::parse(&r)).transpose()
    }

    /// Why a run is terminal, when it is.
    pub fn terminal_reason(&self, run_id: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT terminal_reason FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    /// Create the run row and its `run_started` entry if the run is new.
    /// Returns `true` when it was created by this call.
    ///
    /// The existence check lives INSIDE the transaction (`ON CONFLICT DO
    /// NOTHING` + the affected-row count). Checking before `BEGIN IMMEDIATE`
    /// would make two simultaneous resumes of the same new run race, and the
    /// loser would surface a `UNIQUE constraint failed` — indistinguishable
    /// from real corruption — instead of the idempotent `Ok(false)` this
    /// signature promises.
    pub fn ensure_run(&self, run_id: &str, spec_digest: &str) -> Result<bool, StoreError> {
        self.in_immediate_tx(|conn| {
            let inserted = conn.execute(
                "INSERT INTO runs (run_id, status, next_sequence, spec_digest)
                 VALUES (?1, ?2, 0, ?3)
                 ON CONFLICT(run_id) DO NOTHING",
                params![run_id, RunStatus::Running.as_str(), spec_digest],
            )?;
            if inserted == 0 {
                return Ok(false);
            }
            append_event(conn, run_id, None, None, &RunEventKind::RunStarted)?;
            Ok(true)
        })
    }

    /// The spec digest recorded when the run was created, if the run exists.
    pub fn spec_digest(&self, run_id: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT spec_digest FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .optional()?)
    }

    /// The completed effect memoised for a node instance, if any.
    pub fn completed_effect(
        &self,
        run_id: &str,
        node_instance_id: &str,
    ) -> Result<Option<MemoisedEffect>, StoreError> {
        let row = self
            .conn
            .query_row(
                "SELECT effect_id, request_hash, receipt FROM outbox
                 WHERE run_id = ?1 AND node_instance_id = ?2 AND state = ?3",
                params![run_id, node_instance_id, OUTBOX_APPLIED],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                    ))
                },
            )
            .optional()?;

        match row {
            None => Ok(None),
            Some((effect_id, request_hash, Some(receipt))) => Ok(Some(MemoisedEffect {
                effect_id,
                request_hash,
                receipt,
            })),
            Some((effect_id, _, None)) => Err(StoreError::Corrupt(format!(
                "applied outbox row without receipt: {effect_id}"
            ))),
        }
    }

    /// The intended-but-unconfirmed effect for a node instance, if any.
    pub fn pending_effect(
        &self,
        run_id: &str,
        node_instance_id: &str,
    ) -> Result<Option<PendingEffect>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT effect_id, request_hash, target_idempotent, dedup_scope FROM outbox
                 WHERE run_id = ?1 AND node_instance_id = ?2 AND state = ?3",
                params![run_id, node_instance_id, OUTBOX_PENDING],
                |row| {
                    Ok(PendingEffect {
                        effect_id: row.get(0)?,
                        request_hash: row.get(1)?,
                        target_idempotent: row.get::<_, i64>(2)? != 0,
                        dedup_scope: row.get(3)?,
                    })
                },
            )
            .optional()?)
    }

    /// GUARANTEED PATH — journal entry, sequence advance and outbox row in
    /// ONE transaction. (The run FRONTIER moves when the effect completes,
    /// not when it is intended: an intended step is not a reached one.)
    ///
    /// Production entry point: takes NO fault hook (Phase I closed the E
    /// debt — the harness surface is `record_intent_atomic_with_faults`,
    /// behind `spike-faults`).
    ///
    /// The parameter list is long because it IS the outbox row: every one of
    /// these values is a column written in this single transaction. Bundling
    /// them into a struct would add an indirection without removing anything
    /// a caller has to supply.
    ///
    /// Crate-only in a normal build: `effect_id` is ENGINE-DERIVED
    /// (`derive_effect_id`), and a public store door taking one from the
    /// caller would undo that (preflight B7). The harness build (feature
    /// `spike-faults`, the crate's own tests) keeps it public.
    #[allow(clippy::too_many_arguments)]
    #[cfg(feature = "spike-faults")]
    pub fn record_intent_atomic(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
        target_idempotent: bool,
        dedup_scope: &str,
    ) -> Result<(), StoreError> {
        self.record_intent_atomic_impl(
            run_id,
            node_instance_id,
            request_hash,
            effect_id,
            target_idempotent,
            dedup_scope,
            &NoFaults,
        )
    }

    /// See the `spike-faults` twin above — same body, crate-only.
    #[allow(clippy::too_many_arguments)]
    #[cfg(not(feature = "spike-faults"))]
    pub(crate) fn record_intent_atomic(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
        target_idempotent: bool,
        dedup_scope: &str,
    ) -> Result<(), StoreError> {
        self.record_intent_atomic_impl(
            run_id,
            node_instance_id,
            request_hash,
            effect_id,
            target_idempotent,
            dedup_scope,
            &NoFaults,
        )
    }

    /// HARNESS entry point — same guaranteed transaction, with the fault
    /// hook consulted between the two writes. Behind `spike-faults`.
    #[cfg(feature = "spike-faults")]
    #[allow(clippy::too_many_arguments)]
    pub fn record_intent_atomic_with_faults(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
        target_idempotent: bool,
        dedup_scope: &str,
        faults: &dyn FaultHook,
    ) -> Result<(), StoreError> {
        self.record_intent_atomic_impl(
            run_id,
            node_instance_id,
            request_hash,
            effect_id,
            target_idempotent,
            dedup_scope,
            faults,
        )
    }

    /// Shared body. `faults` is consulted BETWEEN the two writes, inside the
    /// transaction. That fault point is the only thing that makes this
    /// method's guarantee falsifiable: halting there must roll back BOTH
    /// writes, so a test can assert that neither survived. Without an
    /// injection point inside the transaction, splitting this method into
    /// two transactions would leave every test green.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_intent_atomic_impl(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
        target_idempotent: bool,
        dedup_scope: &str,
        faults: &dyn FaultHook,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            let sequence = append_event(
                conn,
                run_id,
                Some(node_instance_id),
                Some(request_hash),
                &RunEventKind::EffectIntended {
                    effect_id: effect_id.to_string(),
                },
            )?;
            if faults.hit(FaultPoint::InsideIntentTransaction) == FaultAction::Halt {
                return Err(StoreError::HaltedInTransaction {
                    at: FaultPoint::InsideIntentTransaction,
                });
            }
            insert_pending_outbox(
                conn,
                run_id,
                node_instance_id,
                effect_id,
                request_hash,
                target_idempotent,
                dedup_scope,
                Some(sequence),
            )?;
            Ok(())
        })
    }

    /// RED CONTROL half — the outbox row alone, in its own transaction.
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn record_intent_outbox_only(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
        target_idempotent: bool,
        dedup_scope: &str,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            insert_pending_outbox(
                conn,
                run_id,
                node_instance_id,
                effect_id,
                request_hash,
                target_idempotent,
                dedup_scope,
                None,
            )
        })
    }

    /// RED CONTROL half — the journal entry alone, in its own transaction.
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn record_intent_event_only(
        &self,
        run_id: &str,
        node_instance_id: &str,
        request_hash: &str,
        effect_id: &str,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            let sequence = append_event(
                conn,
                run_id,
                Some(node_instance_id),
                Some(request_hash),
                &RunEventKind::EffectIntended {
                    effect_id: effect_id.to_string(),
                },
            )?;
            conn.execute(
                "UPDATE outbox SET intent_sequence = ?1
                 WHERE run_id = ?2 AND node_instance_id = ?3 AND effect_id = ?4",
                params![sequence as i64, run_id, node_instance_id, effect_id],
            )?;
            Ok(())
        })
    }

    /// Journal entry, frontier advance and outbox transition to applied, in
    /// ONE transaction. Only the receipt DIGEST reaches the journal.
    ///
    /// Production entry point: takes NO fault hook (the harness surface is
    /// `record_completion_with_faults`, behind `spike-faults`).
    ///
    /// Crate-only in a normal build, like `record_intent_atomic`: the
    /// effect id it confirms is engine-derived.
    #[cfg(feature = "spike-faults")]
    pub fn record_completion(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        self.record_completion_impl(
            run_id,
            node_instance_id,
            effect_id,
            request_hash,
            receipt,
            &NoFaults,
        )
    }

    /// See the `spike-faults` twin above — same body, crate-only.
    #[cfg(not(feature = "spike-faults"))]
    pub(crate) fn record_completion(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        self.record_completion_impl(
            run_id,
            node_instance_id,
            effect_id,
            request_hash,
            receipt,
            &NoFaults,
        )
    }

    /// HARNESS entry point — same transaction, hook consulted between the
    /// append and the outbox transition. Behind `spike-faults`.
    #[cfg(feature = "spike-faults")]
    #[allow(clippy::too_many_arguments)]
    pub fn record_completion_with_faults(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
        faults: &dyn FaultHook,
    ) -> Result<(), StoreError> {
        self.record_completion_impl(
            run_id,
            node_instance_id,
            effect_id,
            request_hash,
            receipt,
            faults,
        )
    }

    /// Shared body. `faults` fires BETWEEN the append and the transition,
    /// inside the transaction — the same falsifiability device as the
    /// intent path.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_completion_impl(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
        faults: &dyn FaultHook,
    ) -> Result<(), StoreError> {
        let digest = crate::hash::receipt_digest(receipt);
        self.in_immediate_tx(|conn| {
            append_event(
                conn,
                run_id,
                Some(node_instance_id),
                Some(request_hash),
                &RunEventKind::EffectCompleted {
                    effect_id: effect_id.to_string(),
                    receipt_digest: digest.clone(),
                },
            )?;
            if faults.hit(FaultPoint::InsideCompletionTransaction) == FaultAction::Halt {
                return Err(StoreError::HaltedInTransaction {
                    at: FaultPoint::InsideCompletionTransaction,
                });
            }
            let updated = conn.execute(
                "UPDATE outbox SET state = ?1, receipt = ?2
                 WHERE run_id = ?3 AND node_instance_id = ?4 AND effect_id = ?5
                   AND state = ?6",
                params![
                    OUTBOX_APPLIED,
                    receipt,
                    run_id,
                    node_instance_id,
                    effect_id,
                    OUTBOX_PENDING
                ],
            )?;
            if updated != 1 {
                return Err(StoreError::Corrupt(format!(
                    "completion for {effect_id} matched {updated} pending outbox rows"
                )));
            }
            conn.execute(
                "UPDATE runs SET frontier_node = ?1 WHERE run_id = ?2",
                params![node_instance_id, run_id],
            )?;
            Ok(())
        })
    }

    /// Journal a LATE completion — the result of an effect that returned
    /// AFTER `cancel_requested` (research §12.7): the receipt is KEPT (its
    /// entry is appended, the outbox row resolves to applied) but the run
    /// FRONTIER does not advance and nothing downstream is dispatched. The
    /// production door of the honest-cancel promise "résultat tardif
    /// conservé sans propagation": before Phase J the only
    /// event-without-frontier writer was a `spike-faults` red control,
    /// unreachable in a normal build.
    #[cfg(feature = "spike-faults")]
    pub fn record_late_completion(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        self.record_late_completion_impl(run_id, node_instance_id, effect_id, request_hash, receipt)
    }

    /// See the `spike-faults` twin above — same body, crate-only.
    #[cfg(not(feature = "spike-faults"))]
    pub(crate) fn record_late_completion(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        self.record_late_completion_impl(run_id, node_instance_id, effect_id, request_hash, receipt)
    }

    fn record_late_completion_impl(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        let digest = crate::hash::receipt_digest(receipt);
        self.in_immediate_tx(|conn| {
            append_event(
                conn,
                run_id,
                Some(node_instance_id),
                Some(request_hash),
                &RunEventKind::EffectCompleted {
                    effect_id: effect_id.to_string(),
                    receipt_digest: digest.clone(),
                },
            )?;
            let updated = conn.execute(
                "UPDATE outbox SET state = ?1, receipt = ?2
                 WHERE run_id = ?3 AND node_instance_id = ?4 AND effect_id = ?5
                   AND state = ?6",
                params![
                    OUTBOX_APPLIED,
                    receipt,
                    run_id,
                    node_instance_id,
                    effect_id,
                    OUTBOX_PENDING
                ],
            )?;
            if updated != 1 {
                return Err(StoreError::Corrupt(format!(
                    "late completion for {effect_id} matched {updated} pending outbox rows"
                )));
            }
            // Deliberately NO frontier update — that is the whole point.
            Ok(())
        })
    }

    /// RED CONTROL half — the completion ENTRY alone, in its own
    /// transaction, leaving the outbox row `pending`.
    /// Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn record_completion_event_only(
        &self,
        run_id: &str,
        node_instance_id: &str,
        effect_id: &str,
        request_hash: &str,
        receipt: &str,
    ) -> Result<(), StoreError> {
        let digest = crate::hash::receipt_digest(receipt);
        self.in_immediate_tx(|conn| {
            append_event(
                conn,
                run_id,
                Some(node_instance_id),
                Some(request_hash),
                &RunEventKind::EffectCompleted {
                    effect_id: effect_id.to_string(),
                    receipt_digest: digest.clone(),
                },
            )?;
            Ok(())
        })
    }

    /// Stop the run for good, recording why — and, on a divergence, WHICH
    /// two hashes disagreed, so the journal alone supports a post-mortem.
    ///
    /// `closed_at_ms` is the caller's clock, for RETENTION only (it lands in
    /// the `runs` state row, never in the journal). `None` is legal — the
    /// clockless spike engine passes it — and means the run is NEVER GC'd
    /// (fail-safe: no date, no purge).
    pub fn record_terminal(
        &self,
        run_id: &str,
        node_instance_id: Option<&str>,
        reason: TerminalReason,
        expected_hash: Option<&str>,
        got_hash: Option<&str>,
        closed_at_ms: Option<i64>,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            // Closed states are ABSORBING: only a RUNNING run may go
            // terminal. Checked FIRST, in the same transaction — the
            // guarded UPDATE's affected-row count IS the check.
            let moved = conn.execute(
                "UPDATE runs SET status = ?1, terminal_reason = ?2, closed_at_ms = ?3
                 WHERE run_id = ?4 AND status = ?5",
                params![
                    RunStatus::Terminal.as_str(),
                    reason.as_str(),
                    closed_at_ms,
                    run_id,
                    RunStatus::Running.as_str()
                ],
            )?;
            if moved != 1 {
                let from: Option<String> = conn
                    .query_row(
                        "SELECT status FROM runs WHERE run_id = ?1",
                        params![run_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                return Err(StoreError::InvalidTransition {
                    run_id: run_id.to_string(),
                    from: from.unwrap_or_else(|| "absent".to_string()),
                    to: RunStatus::Terminal.as_str(),
                });
            }
            append_event(
                conn,
                run_id,
                node_instance_id,
                None,
                &RunEventKind::RunTerminal {
                    reason,
                    expected_hash: expected_hash.map(str::to_string),
                    got_hash: got_hash.map(str::to_string),
                },
            )?;
            Ok(())
        })
    }

    /// Close a run that reached its last step. `closed_at_ms`: see
    /// [`RunStore::record_terminal`].
    pub fn record_run_completed(
        &self,
        run_id: &str,
        closed_at_ms: Option<i64>,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            // Closed states are ABSORBING: only a RUNNING run may close.
            // Without this, re-labelling a TERMINAL run completed would
            // route its next GC through the completed branch and delete
            // the diagnosis (Codex r2 P1).
            let moved = conn.execute(
                "UPDATE runs SET status = ?1, closed_at_ms = ?2
                 WHERE run_id = ?3 AND status = ?4",
                params![
                    RunStatus::Completed.as_str(),
                    closed_at_ms,
                    run_id,
                    RunStatus::Running.as_str()
                ],
            )?;
            if moved != 1 {
                let from: Option<String> = conn
                    .query_row(
                        "SELECT status FROM runs WHERE run_id = ?1",
                        params![run_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                return Err(StoreError::InvalidTransition {
                    run_id: run_id.to_string(),
                    from: from.unwrap_or_else(|| "absent".to_string()),
                    to: RunStatus::Completed.as_str(),
                });
            }
            append_event(conn, run_id, None, None, &RunEventKind::RunCompleted)?;
            Ok(())
        })
    }

    /// Every journal entry of a run, in sequence order — interpreted through
    /// the PRODUCTION upcaster chain. A row written by a newer engine fails
    /// closed ([`StoreError::Upcast`] carrying
    /// [`UpcastError::NewerThanSupported`]) instead of being mis-read: that
    /// is the event-level downgrade guard, and it lives in
    /// [`upcast_payload`], so bypassing the upcasting door would also lose
    /// the guard — the two cannot drift apart.
    pub fn load_events(&self, run_id: &str) -> Result<Vec<RunEvent>, StoreError> {
        self.load_events_impl(run_id, RUN_EVENT_UPCASTERS, RUN_EVENT_FORMAT_VERSION)
    }

    /// HARNESS entry point — interpret through a SYNTHETIC chain and current
    /// version, so the store-level upcasting path is exercisable while the
    /// production chain is still empty (version 1 is the first ever
    /// written). Behind `spike-faults`.
    #[cfg(feature = "spike-faults")]
    pub fn load_events_with_chain(
        &self,
        run_id: &str,
        chain: &[RunEventUpcaster],
        current: u16,
    ) -> Result<Vec<RunEvent>, StoreError> {
        self.load_events_impl(run_id, chain, current)
    }

    fn load_events_impl(
        &self,
        run_id: &str,
        chain: &[RunEventUpcaster],
        current: u16,
    ) -> Result<Vec<RunEvent>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT sequence, format_version, node_instance_id, request_hash, payload
             FROM run_events WHERE run_id = ?1 ORDER BY sequence ASC",
        )?;
        let rows = stmt.query_map(params![run_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;

        let mut events = Vec::new();
        for row in rows {
            let (sequence, format_version, node_instance_id, request_hash, payload) = row?;
            let row_version = u16::try_from(format_version).map_err(|_| {
                StoreError::Corrupt(format!(
                    "event format version out of range: {format_version}"
                ))
            })?;
            // Interpretation-time upcasting: parse the payload as pure data,
            // bring it to the current shape, THEN parse the typed kind. The
            // journal row itself is never rewritten.
            let raw: serde_json::Value = serde_json::from_str(&payload)
                .map_err(|e| StoreError::Corrupt(format!("event payload at {sequence}: {e}")))?;
            let upcasted = upcast_payload(chain, current, row_version, raw)?;
            let kind: RunEventKind = serde_json::from_value(upcasted).map_err(|e| {
                StoreError::Corrupt(format!("event payload at {sequence} (upcasted): {e}"))
            })?;
            events.push(RunEvent {
                // READ from the row, not re-injected from the current
                // constant: a version the reader supplies is not a version,
                // it is a guess that always agrees with itself. After
                // upcasting this still carries the version the row was
                // WRITTEN under — provenance; only `kind` is current-shaped.
                format_version: row_version,
                run_id: run_id.to_string(),
                sequence: u64::try_from(sequence).map_err(|_| {
                    StoreError::Corrupt(format!("negative sequence in journal: {sequence}"))
                })?,
                node_instance_id,
                request_hash,
                kind,
            });
        }
        Ok(events)
    }

    /// Outbox rows with no journal entry claiming them. Must be 0 on the
    /// guaranteed path — an orphan is an effect the run does not know it
    /// owns.
    ///
    /// The join is on `intent_sequence`, the exact entry that created the
    /// row. Matching on `node_instance_id` alone would call a row
    /// "non-orphan" as soon as ANY intent exists for that node, whatever its
    /// effect id — which would quietly weaken the assertion this method
    /// exists to support.
    pub fn orphan_outbox_rows(&self, run_id: &str) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM outbox o
             WHERE o.run_id = ?1
               AND (
                   o.intent_sequence IS NULL
                   OR NOT EXISTS (
                       SELECT 1 FROM run_events e
                       WHERE e.run_id = o.run_id
                         AND e.sequence = o.intent_sequence
                         AND e.kind = ?2
                         AND e.node_instance_id = o.node_instance_id
                   )
               )",
            params![
                run_id,
                RunEventKind::EffectIntended {
                    effect_id: String::new()
                }
                .discriminant()
            ],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    /// The last node instance whose effect COMPLETED — where the run got to.
    /// Distinct from the last intended step: an intended step is not a
    /// reached one, which is exactly what a crash at B2 leaves behind.
    pub fn frontier_node(&self, run_id: &str) -> Result<Option<String>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT frontier_node FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    /// Journal entries recorded for a run.
    pub fn event_count(&self, run_id: &str) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM run_events WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    // ── single-writer lease (Phase I) ─────────────────────────────────────

    /// Acquire (or take over) the store's single-writer lease for `owner`.
    /// Returns the fencing epoch this handle now holds; every subsequent
    /// write transaction of this handle re-asserts `(owner, epoch)` and
    /// aborts with [`StoreError::LeaseLost`] after a takeover.
    ///
    /// EVERY acquisition bumps the epoch — including a re-acquisition by the
    /// same owner name: a restarted process must invalidate any zombie
    /// handle its previous incarnation left behind (Kleppmann — the token is
    /// bumped per ACQUISITION, never reused). A takeover of a live lease is
    /// only allowed once `expires_at_ms <= now_ms`; that wall-clock reading
    /// is the HEURISTIC deciding the previous holder is dead, never a
    /// commit-time condition.
    pub fn acquire_lease(
        &mut self,
        owner: &str,
        now_ms: i64,
        ttl_ms: i64,
    ) -> Result<i64, StoreError> {
        crate::hash::validate_identity_component("lease_owner", owner).map_err(|e| {
            StoreError::InvalidInput {
                what: "lease owner",
                reason: e.to_string(),
            }
        })?;
        // A negative clock would make the FREED row (expiry 0) read as held
        // (`0 > now_ms`) and never acquirable — refused instead of being
        // left as a documented quirk (review D1-1). Fail-closed either way.
        if now_ms < 0 {
            return Err(StoreError::InvalidInput {
                what: "lease clock",
                reason: format!("now_ms must be >= 0, got {now_ms}"),
            });
        }
        if ttl_ms <= 0 {
            return Err(StoreError::InvalidInput {
                what: "lease ttl",
                reason: format!("must be positive, got {ttl_ms}"),
            });
        }
        let expires_at_ms = now_ms
            .checked_add(ttl_ms)
            .ok_or_else(|| StoreError::InvalidInput {
                what: "lease ttl",
                reason: "now_ms + ttl_ms overflows".to_string(),
            })?;

        let epoch = self.in_immediate_tx_unfenced(|conn| {
            let current: Option<(String, i64, i64)> = conn
                .query_row(
                    "SELECT owner, epoch, expires_at_ms FROM lease WHERE id = 1",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?;
            match current {
                None => {
                    conn.execute(
                        "INSERT INTO lease (id, owner, epoch, expires_at_ms)
                         VALUES (1, ?1, 1, ?2)",
                        params![owner, expires_at_ms],
                    )?;
                    Ok(1)
                }
                Some((holder, epoch, holder_expires)) => {
                    if holder != owner && holder_expires > now_ms {
                        return Err(StoreError::LeaseHeld {
                            owner: holder,
                            expires_at_ms: holder_expires,
                        });
                    }
                    let next = epoch.checked_add(1).ok_or_else(|| {
                        StoreError::Corrupt("lease epoch overflows i64".to_string())
                    })?;
                    conn.execute(
                        "UPDATE lease SET owner = ?1, epoch = ?2, expires_at_ms = ?3
                         WHERE id = 1",
                        params![owner, next, expires_at_ms],
                    )?;
                    Ok(next)
                }
            }
        })?;

        self.active_lease = Some((owner.to_string(), epoch));
        Ok(epoch)
    }

    /// Extend the active lease's expiry. A renewal that finds its epoch
    /// taken over clears the handle's lease and fails with
    /// [`StoreError::LeaseLost`].
    pub fn renew_lease(&mut self, now_ms: i64, ttl_ms: i64) -> Result<(), StoreError> {
        let (owner, epoch) = self
            .active_lease
            .clone()
            .ok_or_else(|| StoreError::InvalidInput {
                what: "lease",
                reason: "no active lease on this handle".to_string(),
            })?;
        if now_ms < 0 {
            return Err(StoreError::InvalidInput {
                what: "lease clock",
                reason: format!("now_ms must be >= 0, got {now_ms}"),
            });
        }
        if ttl_ms <= 0 {
            return Err(StoreError::InvalidInput {
                what: "lease ttl",
                reason: format!("must be positive, got {ttl_ms}"),
            });
        }
        let expires_at_ms = now_ms
            .checked_add(ttl_ms)
            .ok_or_else(|| StoreError::InvalidInput {
                what: "lease ttl",
                reason: "now_ms + ttl_ms overflows".to_string(),
            })?;
        let result = self.in_immediate_tx_unfenced(|conn| {
            let matched = conn.execute(
                "UPDATE lease SET expires_at_ms = ?1 WHERE id = 1 AND owner = ?2 AND epoch = ?3",
                params![expires_at_ms, owner, epoch],
            )?;
            if matched != 1 {
                return Err(StoreError::LeaseLost {
                    owner: owner.clone(),
                    epoch,
                });
            }
            Ok(())
        });
        if matches!(result, Err(StoreError::LeaseLost { .. })) {
            self.active_lease = None;
        }
        result
    }

    /// Release the active lease (best effort: a lease already taken over is
    /// simply no longer ours to free). The handle goes back to unfenced.
    ///
    /// The row is MARKED FREE (empty owner, expiry 0), never deleted: the
    /// epoch must stay MONOTONE across the whole life of the store file.
    /// Deleting the row would let the next acquisition re-INSERT at epoch 1
    /// — and a zombie handle from an earlier incarnation still holding
    /// `(same owner, 1)` would pass the CAS again. An empty owner can never
    /// collide with a real one (`validate_identity_component` refuses empty),
    /// and expiry 0 makes the free row immediately acquirable through the
    /// normal takeover branch, which bumps the epoch.
    pub fn release_lease(&mut self) -> Result<(), StoreError> {
        if let Some((owner, epoch)) = self.active_lease.take() {
            self.in_immediate_tx_unfenced(|conn| {
                conn.execute(
                    "UPDATE lease SET owner = '', expires_at_ms = 0
                     WHERE id = 1 AND owner = ?1 AND epoch = ?2",
                    params![owner, epoch],
                )?;
                Ok(())
            })?;
        }
        Ok(())
    }

    /// The lease HOLDER, if any — inspection/debugging. `None` both when no
    /// lease row was ever created and when the lease was released (the row
    /// then persists with an empty owner, carrying the monotone epoch).
    pub fn lease_state(&self) -> Result<Option<LeaseState>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT owner, epoch, expires_at_ms FROM lease WHERE id = 1",
                [],
                |row| {
                    Ok(LeaseState {
                        owner: row.get(0)?,
                        epoch: row.get(1)?,
                        expires_at_ms: row.get(2)?,
                    })
                },
            )
            .optional()?
            .filter(|l| !l.owner.is_empty()))
    }

    /// The epoch THIS HANDLE believes it holds, if any.
    pub fn active_lease_epoch(&self) -> Option<i64> {
        self.active_lease.as_ref().map(|(_, e)| *e)
    }

    // ── inbox (Phase I) ───────────────────────────────────────────────────

    /// Record an inbound message AT-LEAST-ONCE + dedup: returns `true` when
    /// the message is NEW, `false` when this delivery is a duplicate
    /// (collapsed by `INSERT OR IGNORE` under the `(run_id, message_id)`
    /// primary key — the S76 D `task_results` precedent). A message for a
    /// run that does not exist is a foreign-key error, fail-closed (`OR
    /// IGNORE` does NOT swallow FK violations).
    pub fn record_inbox_message(
        &self,
        run_id: &str,
        message_id: &str,
        processed_at_ms: i64,
    ) -> Result<bool, StoreError> {
        crate::hash::validate_identity_component("message_id", message_id).map_err(|e| {
            StoreError::InvalidInput {
                what: "inbox message id",
                reason: e.to_string(),
            }
        })?;
        self.in_immediate_tx(|conn| {
            let inserted = conn.execute(
                "INSERT OR IGNORE INTO inbox (run_id, message_id, processed_at_ms)
                 VALUES (?1, ?2, ?3)",
                params![run_id, message_id, processed_at_ms],
            )?;
            Ok(inserted == 1)
        })
    }

    /// Messages recorded for a run — inspection.
    pub fn inbox_count(&self, run_id: &str) -> Result<usize, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM inbox WHERE run_id = ?1",
            params![run_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    // ── waits, signals, timers, run input (Phase J) ───────────────────────

    /// Create the run row, its `run_started` entry AND its material input
    /// payload, in one transaction. Same idempotency contract as
    /// [`RunStore::ensure_run`]: `false` means the run already existed and
    /// NOTHING was written — a resume never overwrites the recorded input
    /// (the spec digest binds it; a divergent resume is refused upstream).
    pub(crate) fn ensure_run_with_input(
        &self,
        run_id: &str,
        spec_digest: &str,
        input_payload: &serde_json::Value,
    ) -> Result<bool, StoreError> {
        let payload_text =
            serde_json::to_string(input_payload).map_err(|e| StoreError::InvalidInput {
                what: "run input",
                reason: e.to_string(),
            })?;
        self.in_immediate_tx(|conn| {
            let inserted = conn.execute(
                "INSERT INTO runs (run_id, status, next_sequence, spec_digest, input_payload)
                 VALUES (?1, ?2, 0, ?3, ?4)
                 ON CONFLICT(run_id) DO NOTHING",
                params![
                    run_id,
                    RunStatus::Running.as_str(),
                    spec_digest,
                    payload_text
                ],
            )?;
            if inserted == 0 {
                return Ok(false);
            }
            append_event(conn, run_id, None, None, &RunEventKind::RunStarted)?;
            Ok(true)
        })
    }

    /// The material input payload recorded when the run was created.
    pub fn run_input(&self, run_id: &str) -> Result<Option<serde_json::Value>, StoreError> {
        let raw: Option<Option<String>> = self
            .conn
            .query_row(
                "SELECT input_payload FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .optional()?;
        match raw.flatten() {
            None => Ok(None),
            Some(text) => {
                Ok(Some(serde_json::from_str(&text).map_err(|e| {
                    StoreError::Corrupt(format!("run input payload: {e}"))
                })?))
            }
        }
    }

    /// Open (idempotently) the interruption of a parked node: records the
    /// awaited signal name and the engine-derived mono-usage nonce. A
    /// replayed scheduling pass re-opening the same wait is a no-op — the
    /// FIRST nonce written wins, so a resume cannot mint a second spendable
    /// nonce for the same interruption.
    pub(crate) fn open_wait(
        &self,
        run_id: &str,
        node_instance_id: &str,
        signal_name: &str,
        nonce: &str,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO waits
                     (run_id, node_instance_id, signal_name, nonce, status)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![run_id, node_instance_id, signal_name, nonce, WAIT_OPEN],
            )?;
            Ok(())
        })
    }

    /// Every OPEN wait of a run, ordered by node instance id — what a host
    /// reads to know which deposits the run can consume.
    pub fn open_waits(&self, run_id: &str) -> Result<Vec<PendingWait>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT node_instance_id, signal_name, nonce FROM waits
             WHERE run_id = ?1 AND status = ?2 ORDER BY node_instance_id ASC",
        )?;
        let rows = stmt.query_map(params![run_id, WAIT_OPEN], |row| {
            Ok(PendingWait {
                node_instance_id: row.get(0)?,
                signal_name: row.get(1)?,
                nonce: row.get(2)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Deposit a signal/decision for an OPEN wait, at-least-once + dedup:
    /// `true` when the deposit is new, `false` when this delivery is a
    /// duplicate (collapsed on the `(run_id, nonce)` primary key — the
    /// inbox precedent, so a redelivered approval can never become two
    /// spendable deposits).
    ///
    /// A deposit naming NO open wait's nonce is refused LOUDLY
    /// ([`StoreError::SignalRefused`]) with zero rows written: a forged
    /// nonce is observable at the door, never parked silently (THREAT §19
    /// signal-forge mitigation).
    ///
    /// PUBLIC: depositing is the HOST's side of the seam (daemon route,
    /// harness script). Consumption stays engine-internal.
    pub fn deposit_signal(
        &self,
        run_id: &str,
        nonce: &str,
        payload: &serde_json::Value,
        decision_label: Option<&str>,
        expires_at_ms: Option<i64>,
        now_ms: i64,
    ) -> Result<bool, StoreError> {
        crate::hash::validate_identity_component("nonce", nonce).map_err(|e| {
            StoreError::InvalidInput {
                what: "signal nonce",
                reason: e.to_string(),
            }
        })?;
        // Material payloads are bounded at the door (review r1 SD-3 — the
        // identity guards cap every id at 512 bytes while the payload was
        // unbounded; at the future loopback route this is the
        // write-amplification surface).
        let approx = serde_json::to_string(payload)
            .map(|s| s.len())
            .unwrap_or(usize::MAX);
        if approx > MAX_SIGNAL_PAYLOAD_BYTES {
            return Err(StoreError::InvalidInput {
                what: "signal payload",
                reason: format!(
                    "{approx} bytes exceeds MAX_SIGNAL_PAYLOAD_BYTES ({MAX_SIGNAL_PAYLOAD_BYTES})"
                ),
            });
        }
        // Same canonicalisation door as every digest of the crate: a
        // payload the journal-side digest would refuse (a `null` anywhere)
        // is refused at the DEPOSIT, not discovered at consumption.
        crate::hash::canonical_digest(SIGNAL_PAYLOAD_DIGEST_CONTEXT, payload).map_err(|e| {
            StoreError::InvalidInput {
                what: "signal payload",
                reason: e.to_string(),
            }
        })?;
        let payload_text =
            serde_json::to_string(payload).map_err(|e| StoreError::InvalidInput {
                what: "signal payload",
                reason: e.to_string(),
            })?;
        // STORAGE bound only, never a charset domain (Codex r2 P1): the
        // signed plan's `decision_labels` are arbitrary strings the
        // compiler accepts, and refusing one HERE would make a valid plan
        // unresumable. The closed-set check at consumption is the real
        // gate; this is the anti-bloat cap.
        if let Some(label) = decision_label
            && label.len() > MAX_SIGNAL_PAYLOAD_BYTES
        {
            return Err(StoreError::InvalidInput {
                what: "decision label",
                reason: format!("{} bytes exceeds the storage bound", label.len()),
            });
        }
        self.in_immediate_tx(|conn| {
            let open: i64 = conn.query_row(
                "SELECT COUNT(*) FROM waits w
                 WHERE w.run_id = ?1 AND w.nonce = ?2 AND w.status = ?3
                   AND EXISTS (SELECT 1 FROM runs r
                               WHERE r.run_id = ?1 AND r.status = ?4)",
                params![run_id, nonce, WAIT_OPEN, RunStatus::Running.as_str()],
                |row| row.get(0),
            )?;
            if open != 1 {
                // Covers both a forged/spent nonce AND a CLOSED run whose
                // waits were left open by its close (Codex r1 P2): a
                // terminal run takes no deposit.
                return Err(StoreError::SignalRefused {
                    node_instance_id: "<no open wait>".to_string(),
                    reason: format!("no open wait on a running run names nonce {nonce}"),
                });
            }
            let inserted = conn.execute(
                "INSERT OR IGNORE INTO signals
                     (run_id, nonce, payload, decision_label, expires_at_ms,
                      deposited_at_ms, consumed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
                params![
                    run_id,
                    nonce,
                    payload_text,
                    decision_label,
                    expires_at_ms,
                    now_ms
                ],
            )?;
            Ok(inserted == 1)
        })
    }

    /// Consume the deposit of ONE parked node — the §12.6 mono-usage door.
    /// In ONE transaction: the wait's nonce is looked up, the deposit is
    /// validated (present, unconsumed, unexpired, label inside the closed
    /// set when one is expected), the wait AND the deposit flip to
    /// consumed, and the `signal_consumed`/`human_decision_recorded` entry
    /// is appended. A replayed delivery therefore can never re-fire the
    /// resume: the nonce is spent in the same transaction that journals it.
    ///
    /// `expected_labels`: `None` for a `wait.signal` (a deposit carrying a
    /// decision label is refused), `Some(labels)` for a `human.review` (a
    /// label is REQUIRED and must belong to the closed set — validated
    /// HERE, before anything is written).
    ///
    /// Clock-regression guard: a deposit whose `deposited_at_ms` is in the
    /// FUTURE of the injected `now_ms` is refused loudly — a backwards
    /// clock must not un-expire deposits (THREAT §19 timer-skew).
    pub(crate) fn consume_signal_atomic(
        &self,
        run_id: &str,
        node_instance_id: &str,
        expected_labels: Option<&[String]>,
        now_ms: i64,
    ) -> Result<ConsumedSignal, StoreError> {
        self.in_immediate_tx(|conn| {
            let (status, signal_name, nonce): (String, String, String) = conn
                .query_row(
                    "SELECT status, signal_name, nonce FROM waits
                     WHERE run_id = ?1 AND node_instance_id = ?2",
                    params![run_id, node_instance_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .optional()?
                .ok_or_else(|| StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: "no wait open for this node".to_string(),
                })?;
            if status != WAIT_OPEN {
                return Err(StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: format!("nonce {nonce} already spent"),
                });
            }
            let row: Option<DepositRow> = conn
                .query_row(
                    "SELECT payload, decision_label, expires_at_ms, deposited_at_ms, consumed
                     FROM signals WHERE run_id = ?1 AND nonce = ?2",
                    params![run_id, nonce],
                    |row| {
                        Ok((
                            row.get(0)?,
                            row.get(1)?,
                            row.get(2)?,
                            row.get(3)?,
                            row.get(4)?,
                        ))
                    },
                )
                .optional()?;
            let Some((payload_text, label, expires_at_ms, deposited_at_ms, consumed)) = row
            else {
                return Err(StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: format!("no deposit for nonce {nonce}"),
                });
            };
            if consumed != 0 {
                return Err(StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: format!("deposit for nonce {nonce} already consumed"),
                });
            }
            if deposited_at_ms > now_ms {
                return Err(StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: format!(
                        "clock regression: deposit at {deposited_at_ms} is in the future of {now_ms}"
                    ),
                });
            }
            if let Some(expiry) = expires_at_ms
                && expiry < now_ms
            {
                return Err(StoreError::SignalRefused {
                    node_instance_id: node_instance_id.to_string(),
                    reason: format!("deposit expired at {expiry} (now {now_ms})"),
                });
            }
            let payload: serde_json::Value = serde_json::from_str(&payload_text)
                .map_err(|e| StoreError::Corrupt(format!("signal payload: {e}")))?;
            let payload_digest =
                crate::hash::canonical_digest(SIGNAL_PAYLOAD_DIGEST_CONTEXT, &payload).map_err(
                    |e| StoreError::Corrupt(format!("signal payload digest: {e}")),
                )?;
            let event = match expected_labels {
                None => {
                    if label.is_some() {
                        return Err(StoreError::SignalRefused {
                            node_instance_id: node_instance_id.to_string(),
                            reason: "unexpected decision label on a plain signal".to_string(),
                        });
                    }
                    RunEventKind::SignalConsumed {
                        signal_name: signal_name.clone(),
                        nonce: nonce.clone(),
                        payload_digest,
                    }
                }
                Some(allowed) => {
                    let Some(label) = label.clone() else {
                        return Err(StoreError::SignalRefused {
                            node_instance_id: node_instance_id.to_string(),
                            reason: "missing decision label".to_string(),
                        });
                    };
                    if !allowed.iter().any(|a| a == &label) {
                        return Err(StoreError::SignalRefused {
                            node_instance_id: node_instance_id.to_string(),
                            reason: format!("decision label `{label}` outside the closed set"),
                        });
                    }
                    RunEventKind::HumanDecisionRecorded {
                        nonce: nonce.clone(),
                        decision_label: label,
                        payload_digest,
                    }
                }
            };
            let spent = conn.execute(
                "UPDATE waits SET status = ?1
                 WHERE run_id = ?2 AND node_instance_id = ?3 AND status = ?4",
                params![WAIT_CONSUMED, run_id, node_instance_id, WAIT_OPEN],
            )?;
            // Defence in depth, not the load-bearing guard: the wait's CAS
            // above is what mono-usage rests on (mutation pair M2/M2' —
            // leaving THIS flip out keeps every current test green because
            // a consumed wait already blocks re-consumption and re-deposit;
            // the flag exists so a corrupt waits table cannot silently
            // reopen a spent deposit).
            let marked = conn.execute(
                "UPDATE signals SET consumed = 1
                 WHERE run_id = ?1 AND nonce = ?2 AND consumed = 0",
                params![run_id, nonce],
            )?;
            if spent != 1 || marked != 1 {
                return Err(StoreError::Corrupt(format!(
                    "signal consumption raced: wait rows {spent}, signal rows {marked}"
                )));
            }
            append_event(conn, run_id, Some(node_instance_id), None, &event)?;
            Ok(ConsumedSignal {
                payload,
                decision_label: label,
            })
        })
    }

    /// Advance the store-wide clock watermark to `now_ms`, refusing a
    /// REGRESSIVE reading loudly ([`StoreError::ClockRegression`]). Called
    /// once per scheduling pass, BEFORE any decision consults the clock —
    /// what closes the between-passes de-expiry window (review r1 SD-2).
    pub(crate) fn advance_clock_watermark(&self, now_ms: i64) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO clock_watermark (id, max_now_ms) VALUES (1, ?1)",
                params![now_ms],
            )?;
            let watermark: i64 = conn.query_row(
                "SELECT max_now_ms FROM clock_watermark WHERE id = 1",
                [],
                |row| row.get(0),
            )?;
            if now_ms < watermark {
                return Err(StoreError::ClockRegression {
                    now_ms,
                    watermark_ms: watermark,
                });
            }
            conn.execute(
                "UPDATE clock_watermark SET max_now_ms = ?1 WHERE id = 1",
                params![now_ms],
            )?;
            Ok(())
        })
    }

    /// Whether an UNCONSUMED deposit exists for a nonce — how the
    /// scheduler distinguishes "not delivered yet" (keep waiting) from a
    /// consumption refusal (loud error).
    pub(crate) fn has_deposit(&self, run_id: &str, nonce: &str) -> Result<bool, StoreError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM signals
             WHERE run_id = ?1 AND nonce = ?2 AND consumed = 0",
            params![run_id, nonce],
            |row| row.get(0),
        )?;
        Ok(count == 1)
    }

    /// The payload a parked node's consumption recorded, for downstream
    /// value resolution (`wait.signal` output = the signal payload;
    /// `human.review` output = the decision payload). `None` while nothing
    /// was consumed.
    pub(crate) fn consumed_signal(
        &self,
        run_id: &str,
        node_instance_id: &str,
    ) -> Result<Option<ConsumedSignal>, StoreError> {
        let row: Option<(String, Option<String>)> = self
            .conn
            .query_row(
                "SELECT s.payload, s.decision_label
                 FROM waits w JOIN signals s
                   ON s.run_id = w.run_id AND s.nonce = w.nonce
                 WHERE w.run_id = ?1 AND w.node_instance_id = ?2
                   AND w.status = ?3 AND s.consumed = 1",
                params![run_id, node_instance_id, WAIT_CONSUMED],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((payload_text, decision_label)) => {
                let payload: serde_json::Value = serde_json::from_str(&payload_text)
                    .map_err(|e| StoreError::Corrupt(format!("signal payload: {e}")))?;
                Ok(Some(ConsumedSignal {
                    payload,
                    decision_label,
                }))
            }
        }
    }

    /// Schedule (idempotently) a durable timer at an ABSOLUTE deadline.
    /// `INSERT OR IGNORE`: the FIRST deadline written for a timer id wins
    /// forever, so a restart re-reads the original deadline instead of
    /// re-arming — the delay-as-remainder drift cannot happen.
    pub(crate) fn schedule_timer(
        &self,
        run_id: &str,
        timer_id: &str,
        deadline_ms: i64,
    ) -> Result<(), StoreError> {
        crate::hash::validate_identity_component("timer_id", timer_id).map_err(|e| {
            StoreError::InvalidInput {
                what: "timer id",
                reason: e.to_string(),
            }
        })?;
        self.in_immediate_tx(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO timers (run_id, timer_id, deadline_ms, fired)
                 VALUES (?1, ?2, ?3, 0)",
                params![run_id, timer_id, deadline_ms],
            )?;
            Ok(())
        })
    }

    /// Unfired timers whose deadline has passed, ordered by timer id.
    pub(crate) fn due_timers(
        &self,
        run_id: &str,
        now_ms: i64,
    ) -> Result<Vec<(String, i64)>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT timer_id, deadline_ms FROM timers
             WHERE run_id = ?1 AND fired = 0 AND deadline_ms <= ?2
             ORDER BY timer_id ASC",
        )?;
        let rows = stmt.query_map(params![run_id, now_ms], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Fire one due timer AND journal the cancellation it triggers, in ONE
    /// transaction. The `fired` flip is a CAS (`fired = 0` in the WHERE):
    /// a timer fires AT MOST once across every replay and every clock the
    /// host injects — a regressive clock cannot re-arm it. Returns `false`
    /// when another pass already fired it (nothing written).
    pub(crate) fn fire_timer_with_cancel(
        &self,
        run_id: &str,
        timer_id: &str,
        origin: crate::event::CancelOrigin,
    ) -> Result<bool, StoreError> {
        self.in_immediate_tx(|conn| {
            // Journal hygiene (review r1 SD-5): a deadline crossing a run
            // that already CLOSED must not append an orphan cancellation
            // entry to a closed journal — the fire is gated on Running.
            let fired = conn.execute(
                "UPDATE timers SET fired = 1
                 WHERE run_id = ?1 AND timer_id = ?2 AND fired = 0
                   AND EXISTS (SELECT 1 FROM runs r
                               WHERE r.run_id = ?1 AND r.status = ?3)
                   AND NOT EXISTS (SELECT 1 FROM run_events e
                                   WHERE e.run_id = ?1 AND e.kind = ?4)",
                params![
                    run_id,
                    timer_id,
                    RunStatus::Running.as_str(),
                    RunEventKind::CancelRequested {
                        origin: crate::event::CancelOrigin::HostOperator
                    }
                    .discriminant()
                ],
            )?;
            if fired != 1 {
                // Not fired: already fired, run closed, or a cancellation
                // already journaled (operator + timer must not stack two
                // entries — Codex r1 P2).
                return Ok(false);
            }
            append_event(
                conn,
                run_id,
                None,
                None,
                &RunEventKind::CancelRequested { origin },
            )?;
            Ok(true)
        })
    }

    /// Journal a cancellation request, idempotently: `true` when THIS call
    /// recorded it, `false` when the run already has one (or is no longer
    /// running — a closed run cannot be cancelled, it is already closed).
    /// Host-operator authority is unconditional (a pack cannot opt out),
    /// which is why the door itself carries no policy check.
    pub(crate) fn record_cancel_requested(
        &self,
        run_id: &str,
        origin: crate::event::CancelOrigin,
    ) -> Result<bool, StoreError> {
        self.in_immediate_tx(|conn| {
            let running: i64 = conn.query_row(
                "SELECT COUNT(*) FROM runs WHERE run_id = ?1 AND status = ?2",
                params![run_id, RunStatus::Running.as_str()],
                |row| row.get(0),
            )?;
            if running != 1 {
                return Ok(false);
            }
            let already: i64 = conn.query_row(
                "SELECT COUNT(*) FROM run_events WHERE run_id = ?1 AND kind = ?2",
                params![
                    run_id,
                    RunEventKind::CancelRequested {
                        origin: crate::event::CancelOrigin::HostOperator
                    }
                    .discriminant()
                ],
                |row| row.get(0),
            )?;
            if already > 0 {
                return Ok(false);
            }
            append_event(
                conn,
                run_id,
                None,
                None,
                &RunEventKind::CancelRequested { origin },
            )?;
            Ok(true)
        })
    }

    /// Journal one SCHEDULER DECISION (`branch_taken`, `map_fanout`) for a
    /// node, in its own transaction — written BEFORE the scheduler acts on
    /// the decision, which is the replay-divergence mitigation: the reducer
    /// rebuilds control flow from these entries without parsing any
    /// payload.
    pub(crate) fn record_decision(
        &self,
        run_id: &str,
        node_instance_id: &str,
        kind: &RunEventKind,
    ) -> Result<(), StoreError> {
        debug_assert!(
            matches!(
                kind,
                RunEventKind::BranchTaken { .. } | RunEventKind::MapFanout { .. }
            ),
            "record_decision is the door for scheduler decisions only"
        );
        self.in_immediate_tx(|conn| {
            append_event(conn, run_id, Some(node_instance_id), None, kind)?;
            Ok(())
        })
    }

    // ── disposable snapshots (Phase I) ────────────────────────────────────

    /// Store (or replace) the run's snapshot: a CACHE of a reduction of the
    /// journal up to `upto_sequence`. Pure JSON data only — the canonical
    /// digest door refuses a `null` anywhere in `state`, the same rule every
    /// digest of this crate lives under.
    ///
    /// The digest covers the WHOLE served tuple — format version,
    /// `upto_sequence` and `state` — not the state alone: a tamperer who
    /// kept a valid state but moved the sequence bound would otherwise pass
    /// verification and serve a lying bound to a future reducer (review
    /// D4-1; defence-in-depth, the journal stays the truth either way).
    pub fn write_snapshot(
        &self,
        run_id: &str,
        upto_sequence: u64,
        state: &serde_json::Value,
    ) -> Result<(), StoreError> {
        let state_digest = crate::hash::canonical_digest(
            SNAPSHOT_STATE_DIGEST_CONTEXT,
            &snapshot_digest_subject(run_id, SNAPSHOT_FORMAT_VERSION, upto_sequence, state),
        )
        .map_err(|e| StoreError::InvalidInput {
            what: "snapshot state",
            reason: e.to_string(),
        })?;
        let state_text = serde_json::to_string(state).map_err(|e| StoreError::InvalidInput {
            what: "snapshot state",
            reason: e.to_string(),
        })?;
        let seq = i64::try_from(upto_sequence).map_err(|_| StoreError::InvalidInput {
            what: "snapshot upto_sequence",
            reason: format!("out of i64 range: {upto_sequence}"),
        })?;
        self.in_immediate_tx(|conn| {
            conn.execute(
                "INSERT INTO snapshots (run_id, snapshot_version, upto_sequence, state, state_digest)
                 VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT(run_id) DO UPDATE SET
                     snapshot_version = excluded.snapshot_version,
                     upto_sequence    = excluded.upto_sequence,
                     state            = excluded.state,
                     state_digest     = excluded.state_digest",
                params![run_id, SNAPSHOT_FORMAT_VERSION, seq, state_text, state_digest],
            )?;
            Ok(())
        })
    }

    /// Read the run's snapshot WITH every validity check: format version
    /// (stale ⇒ discarded) and content digest (poisoned ⇒ discarded). Only
    /// [`SnapshotRead::Valid`] may be used, and even then only as a cache —
    /// the journal always re-derives the same state, which is what
    /// disposability means.
    pub fn read_snapshot(&self, run_id: &str) -> Result<SnapshotRead, StoreError> {
        self.read_snapshot_impl(run_id, true)
    }

    /// RED CONTROL — read WITHOUT the digest check, so a test can show the
    /// poisoned state IS observable when the guard is off (proving the guard
    /// in [`RunStore::read_snapshot`] is load-bearing). Behind
    /// `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn read_snapshot_unverified(&self, run_id: &str) -> Result<SnapshotRead, StoreError> {
        self.read_snapshot_impl(run_id, false)
    }

    fn read_snapshot_impl(&self, run_id: &str, verify: bool) -> Result<SnapshotRead, StoreError> {
        let row: Option<(i64, i64, String, String)> = self
            .conn
            .query_row(
                "SELECT snapshot_version, upto_sequence, state, state_digest
                 FROM snapshots WHERE run_id = ?1",
                params![run_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()?;
        let Some((version, seq, state_text, recorded_digest)) = row else {
            return Ok(SnapshotRead::Missing);
        };
        if version != SNAPSHOT_FORMAT_VERSION {
            return Ok(SnapshotRead::Stale {
                found_version: version,
            });
        }
        let Ok(state) = serde_json::from_str::<serde_json::Value>(&state_text) else {
            return Ok(SnapshotRead::Poisoned);
        };
        // A negative sequence bound is a tampered row, and a snapshot is a
        // CACHE: discarded (Poisoned), never a hard failure the caller
        // cannot ignore (Codex r2 L5).
        let Ok(upto_sequence) = u64::try_from(seq) else {
            return Ok(SnapshotRead::Poisoned);
        };
        if verify {
            // Recomputed over the SAME tuple write_snapshot digested —
            // version, sequence bound and state together, so no served
            // field escapes authentication (`version == SNAPSHOT_FORMAT_VERSION`
            // at this point, the Stale arm returned above).
            let recomputed = crate::hash::canonical_digest(
                SNAPSHOT_STATE_DIGEST_CONTEXT,
                &snapshot_digest_subject(run_id, version, upto_sequence, &state),
            );
            match recomputed {
                Ok(d) if d == recorded_digest => {}
                // A tuple that no longer digests (or digests differently)
                // was tampered with after it was written: discarded.
                _ => return Ok(SnapshotRead::Poisoned),
            }
        }
        Ok(SnapshotRead::Valid(Snapshot {
            upto_sequence,
            state,
        }))
    }

    /// Drop the run's snapshot. Dropping a cache changes NOTHING observable
    /// — that property is exactly what the disposability test asserts.
    pub fn drop_snapshot(&self, run_id: &str) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            conn.execute("DELETE FROM snapshots WHERE run_id = ?1", params![run_id])?;
            Ok(())
        })
    }

    // ── retention / GC (Phase I) ──────────────────────────────────────────

    /// Purge CLOSED runs past the policy's horizon, sparing the most recent
    /// `keep_last` candidates. Per-run, in ONE transaction each (a crash
    /// mid-GC leaves every run either fully purged or fully intact).
    ///
    /// Never purged, by construction: a RUNNING run (not a candidate), a
    /// PENDING outbox row (a completed run holding one is skipped and
    /// counted; a terminal run keeps its pending rows), and the terminal
    /// DIAGNOSIS (the `runs` row and the `run_terminal` journal entry of a
    /// terminal run survive every pass). A run whose `closed_at_ms` is NULL
    /// is never a candidate — no date, no purge.
    ///
    /// GC is an accepted, policy-bounded loss of audit evidence: what the
    /// journal loses here is stated by this signature, not hidden behind a
    /// default.
    pub fn gc(&self, now_ms: i64, policy: &RetentionPolicy) -> Result<GcOutcome, StoreError> {
        self.gc_impl(now_ms, policy, &NoFaults)
    }

    /// HARNESS entry point — the fault hook fires INSIDE each per-run purge
    /// transaction, between the child-table deletes and the final step, so
    /// the per-run atomicity is falsifiable. Behind `spike-faults`.
    #[cfg(feature = "spike-faults")]
    pub fn gc_with_faults(
        &self,
        now_ms: i64,
        policy: &RetentionPolicy,
        faults: &dyn FaultHook,
    ) -> Result<GcOutcome, StoreError> {
        self.gc_impl(now_ms, policy, faults)
    }

    fn gc_impl(
        &self,
        now_ms: i64,
        policy: &RetentionPolicy,
        faults: &dyn FaultHook,
    ) -> Result<GcOutcome, StoreError> {
        // A negative clock or age would move the cutoff into the FUTURE and
        // purge runs the policy meant to protect — refused, not clamped.
        if now_ms < 0 {
            return Err(StoreError::InvalidInput {
                what: "gc clock",
                reason: format!("now_ms must be >= 0, got {now_ms}"),
            });
        }
        if policy.min_age_ms < 0 {
            return Err(StoreError::InvalidInput {
                what: "gc retention age",
                reason: format!("min_age_ms must be >= 0, got {}", policy.min_age_ms),
            });
        }
        let cutoff = now_ms.saturating_sub(policy.min_age_ms);
        let mut stmt = self.conn.prepare(
            "SELECT run_id FROM runs
             WHERE status IN (?1, ?2)
               AND closed_at_ms IS NOT NULL
               AND closed_at_ms <= ?3
             ORDER BY closed_at_ms DESC, run_id DESC
             LIMIT -1 OFFSET ?4",
        )?;
        let candidates: Vec<String> = stmt
            .query_map(
                params![
                    RunStatus::Completed.as_str(),
                    RunStatus::Terminal.as_str(),
                    cutoff,
                    i64::from(policy.keep_last)
                ],
                |row| row.get(0),
            )?
            .collect::<Result<_, _>>()?;
        drop(stmt);

        let mut outcome = GcOutcome::default();
        for run_id in candidates {
            // Harness-only interleaving point, OUTSIDE any transaction: a
            // hook may mutate the run through a SEPARATE connection here to
            // prove the re-read below is load-bearing (Codex r1 L6).
            let _ = faults.hit(FaultPoint::BeforeGcPurgeOfRun);
            let action = self.in_immediate_tx(|conn| {
                // EVERYTHING that decides the purge is re-read INSIDE the
                // purge transaction: status, close time, pending rows. The
                // candidate scan above is only a pre-filter — a run whose
                // status flipped (e.g. an unfenced writer recorded a
                // terminal diagnosis) or whose eligibility lapsed between
                // scan and purge is re-dispatched on the CURRENT row, never
                // on the stale scan (Codex r1 P1: acting on the scanned
                // status could delete a terminal diagnosis through the
                // completed branch).
                let row: Option<(String, Option<i64>)> = conn
                    .query_row(
                        "SELECT status, closed_at_ms FROM runs WHERE run_id = ?1",
                        params![run_id],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;
                let Some((status_raw, closed_at_ms)) = row else {
                    // Purged by a concurrent pass — nothing to do.
                    return Ok(GcAction::NotEligible);
                };
                let status = RunStatus::parse(&status_raw)?;
                match closed_at_ms {
                    Some(closed) if closed <= cutoff => {}
                    // Re-opened, re-dated or date cleared since the scan:
                    // no longer eligible.
                    _ => return Ok(GcAction::NotEligible),
                }
                // keep_last is RE-RANKED on the current rows too: a run
                // re-dated since the scan may now sit inside the protected
                // window (Codex r2 P2). Same ordering as the scan.
                if policy.keep_last > 0 {
                    let closed = closed_at_ms.expect("eligibility checked above");
                    let newer: i64 = conn.query_row(
                        "SELECT COUNT(*) FROM runs
                         WHERE status IN (?1, ?2)
                           AND closed_at_ms IS NOT NULL
                           AND closed_at_ms <= ?3
                           AND (closed_at_ms > ?4
                                OR (closed_at_ms = ?4 AND run_id > ?5))",
                        params![
                            RunStatus::Completed.as_str(),
                            RunStatus::Terminal.as_str(),
                            cutoff,
                            closed,
                            run_id
                        ],
                        |row| row.get(0),
                    )?;
                    if newer < i64::from(policy.keep_last) {
                        return Ok(GcAction::NotEligible);
                    }
                }
                let pending: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM outbox WHERE run_id = ?1 AND state = ?2",
                    params![run_id, OUTBOX_PENDING],
                    |row| row.get(0),
                )?;
                match status {
                    // A run that went BACK to running since the scan is
                    // alive: never touched (the scan could only have seen a
                    // closed status, so this is the re-read guard working).
                    RunStatus::Running => Ok(GcAction::NotEligible),
                    RunStatus::Completed => {
                        if pending > 0 {
                            return Ok(GcAction::SkippedPendingOutbox);
                        }
                        conn.execute("DELETE FROM snapshots WHERE run_id = ?1", params![run_id])?;
                        conn.execute("DELETE FROM inbox WHERE run_id = ?1", params![run_id])?;
                        conn.execute("DELETE FROM outbox WHERE run_id = ?1", params![run_id])?;
                        conn.execute("DELETE FROM run_events WHERE run_id = ?1", params![run_id])?;
                        if faults.hit(FaultPoint::InsideGcTransaction) == FaultAction::Halt {
                            return Err(StoreError::HaltedInTransaction {
                                at: FaultPoint::InsideGcTransaction,
                            });
                        }
                        conn.execute("DELETE FROM runs WHERE run_id = ?1", params![run_id])?;
                        Ok(GcAction::PurgedCompleted)
                    }
                    RunStatus::Terminal => {
                        conn.execute("DELETE FROM snapshots WHERE run_id = ?1", params![run_id])?;
                        conn.execute("DELETE FROM inbox WHERE run_id = ?1", params![run_id])?;
                        conn.execute(
                            "DELETE FROM outbox WHERE run_id = ?1 AND state != ?2",
                            params![run_id, OUTBOX_PENDING],
                        )?;
                        let deleted = conn.execute(
                            "DELETE FROM run_events WHERE run_id = ?1 AND kind != ?2",
                            params![
                                run_id,
                                RunEventKind::RunTerminal {
                                    reason: TerminalReason::OutcomeUnknown,
                                    expected_hash: None,
                                    got_hash: None,
                                }
                                .discriminant()
                            ],
                        )?;
                        if faults.hit(FaultPoint::InsideGcTransaction) == FaultAction::Halt {
                            return Err(StoreError::HaltedInTransaction {
                                at: FaultPoint::InsideGcTransaction,
                            });
                        }
                        // The `runs` row and the run_terminal entry SURVIVE:
                        // they are the diagnosis a post-mortem depends on.
                        Ok(if deleted > 0 {
                            GcAction::PurgedTerminal
                        } else {
                            GcAction::NotEligible
                        })
                    }
                }
            })?;
            match action {
                GcAction::PurgedCompleted => outcome.completed_purged += 1,
                GcAction::PurgedTerminal => outcome.terminal_purged += 1,
                GcAction::SkippedPendingOutbox => outcome.skipped_pending_outbox += 1,
                GcAction::NotEligible => {}
            }
        }
        Ok(outcome)
    }

    /// RED CONTROL — purge a run with EVERY guard removed (status, pending
    /// outbox, terminal diagnosis: nothing is spared). Shows the bad state
    /// the production guards prevent is really reachable, proving those
    /// guards are load-bearing. Behind `spike-faults`: a normal build cannot
    /// reach it.
    #[cfg(feature = "spike-faults")]
    pub fn gc_purge_run_unguarded(&self, run_id: &str) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            conn.execute("DELETE FROM snapshots WHERE run_id = ?1", params![run_id])?;
            conn.execute("DELETE FROM inbox WHERE run_id = ?1", params![run_id])?;
            conn.execute("DELETE FROM outbox WHERE run_id = ?1", params![run_id])?;
            conn.execute("DELETE FROM run_events WHERE run_id = ?1", params![run_id])?;
            conn.execute("DELETE FROM runs WHERE run_id = ?1", params![run_id])?;
            Ok(())
        })
    }

    // ── backup / restore (Phase I) ────────────────────────────────────────

    /// Write a transactionally-consistent snapshot of the store to `dest`
    /// via `VACUUM INTO` (SQL — the rusqlite `backup` cargo feature stays
    /// off, workspace-wide). The copy is written to a temporary sibling and
    /// atomically renamed, so `dest` is never a partial file; an existing
    /// destination is refused, never overwritten.
    ///
    /// SQLite only fsyncs the VACUUM INTO output when the connection's
    /// `synchronous` level is NORMAL or FULL — re-asserted here even though
    /// `open` already did, because this method's durability claim depends on
    /// it. This is a POINT-IN-TIME snapshot, not PITR.
    pub fn backup_into(&self, dest: &Path) -> Result<(), StoreError> {
        if dest.exists() {
            return Err(StoreError::BackupDestinationExists {
                path: dest.display().to_string(),
            });
        }
        // Alias guard (Codex r1): the destination (and therefore its .tmp
        // sibling) must not resolve into the live database file or its WAL
        // siblings — `VACUUM INTO` onto the source, or the unconditional
        // .tmp removal below hitting the store, would be self-destruction.
        // `dest` does not exist yet, so the comparison canonicalises the
        // PARENT and compares file names.
        if let Some(db_path) = self.conn.path() {
            let db = Path::new(db_path);
            let same_dir = match (
                dest.parent().and_then(|p| p.canonicalize().ok()),
                db.parent().and_then(|p| p.canonicalize().ok()),
            ) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };
            if same_dir {
                let db_name = db.file_name().map(|n| n.to_os_string());
                for candidate in [
                    dest.file_name().map(|n| n.to_os_string()),
                    dest.file_name()
                        .map(|n| format!("{}.tmp", n.to_string_lossy()).into()),
                ] {
                    let clashes = [
                        db_name.clone(),
                        db_name
                            .as_ref()
                            .map(|n| format!("{}-wal", n.to_string_lossy()).into()),
                        db_name
                            .as_ref()
                            .map(|n| format!("{}-shm", n.to_string_lossy()).into()),
                    ];
                    if candidate.is_some() && clashes.contains(&candidate) {
                        return Err(StoreError::InvalidInput {
                            what: "backup destination",
                            reason: "aliases the live store or its WAL siblings".to_string(),
                        });
                    }
                }
            }
        }
        self.assert_pragma_i64("synchronous", 2, "FULL")?;

        let file_name = dest
            .file_name()
            .ok_or_else(|| StoreError::InvalidInput {
                what: "backup destination",
                reason: "path has no file name".to_string(),
            })?
            .to_string_lossy()
            .into_owned();
        let tmp: PathBuf = dest.with_file_name(format!("{file_name}.tmp"));
        // A stale .tmp is by definition an interrupted, possibly-corrupt
        // copy: replaced, never trusted.
        if tmp.exists() {
            std::fs::remove_file(&tmp)?;
        }
        let tmp_str = tmp.to_str().ok_or_else(|| StoreError::InvalidInput {
            what: "backup destination",
            reason: "non-UTF-8 path".to_string(),
        })?;
        self.conn.execute("VACUUM INTO ?1", params![tmp_str])?;
        std::fs::rename(&tmp, dest)?;
        #[cfg(unix)]
        if let Some(parent) = dest.parent() {
            std::fs::File::open(parent)?.sync_all()?;
        }
        Ok(())
    }

    /// Replace the CLOSED store at `store_path` with a validated backup.
    ///
    /// Validation happens BEFORE anything is touched: the backup must open
    /// read-only, contain the `runs` table, and carry a schema version this
    /// binary knows ([`StoreError::UnsupportedSchemaVersion`] otherwise —
    /// restoring a too-new backup is the schema-level downgrade guard
    /// firing). Stale `-wal`/`-shm` siblings of the replaced file are
    /// deleted: they belong to the OLD database, and SQLite replaying them
    /// over the restored one would corrupt it.
    ///
    /// The store must be CLOSED (no live [`RunStore`] on `store_path`) — on
    /// Windows an open store makes the copy fail with a sharing violation,
    /// which is the guard working, not a bug. Restore is by nature an
    /// anti-rollback surface: it rewinds the journal to the backup's point
    /// in time (THREAT_MODEL A11 / T-RUN-RESTORE-ROLLBACK).
    ///
    /// Accepted OS-local residual: the window between validating the backup
    /// and copying it is not closed, symlinks on either path are followed,
    /// and the copy itself is plain (`fs::copy`, no tmp+rename+fsync like
    /// `backup_into` — a crash mid-restore can leave a partial main file;
    /// re-running the restore is the remedy, the validated backup is still
    /// there). Inside the single-user OS boundary of A11 an actor who can
    /// exploit either can already rewrite the store directly — and a
    /// too-new file swapped into the window is re-caught by the
    /// `user_version` guard at the next [`RunStore::open`].
    pub fn restore_from_backup(backup: &Path, store_path: &Path) -> Result<(), StoreError> {
        let conn =
            Connection::open_with_flags(backup, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(|e| {
                StoreError::InvalidBackup {
                    reason: format!("cannot open read-only: {e}"),
                }
            })?;
        // Opening is LAZY: a non-database file only fails at the first read,
        // so every validation query maps to InvalidBackup, not to a bare
        // SQLite error.
        let user_version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(|e| StoreError::InvalidBackup {
                reason: format!("cannot read user_version: {e}"),
            })?;
        if user_version > MIGRATIONS.len() as i64 {
            return Err(StoreError::UnsupportedSchemaVersion {
                found: user_version,
                supported: MIGRATIONS.len(),
            });
        }
        // A run-store backup ALWAYS carries at least the Phase E schema
        // (user_version >= 1) — version 0 with a `runs` table is a forgery,
        // not an old store.
        if user_version < 1 {
            return Err(StoreError::InvalidBackup {
                reason: format!("user_version {user_version} is not a run store"),
            });
        }
        // The table inventory the claimed version REQUIRES must be present
        // in full: a syntactically-valid SQLite file with the right
        // user_version and a lone `runs` table is not a backup, and letting
        // it through would destructively replace a healthy store (Codex r1
        // P1). Version 1 = the Phase E tables; version 2 adds Phase I's.
        const RUNS_V1: &[&str] = &[
            "run_id",
            "status",
            "next_sequence",
            "spec_digest",
            "frontier_node",
            "terminal_reason",
        ];
        const RUNS_V2: &[&str] = &[
            "run_id",
            "status",
            "next_sequence",
            "spec_digest",
            "frontier_node",
            "terminal_reason",
            "closed_at_ms",
        ];
        const RUN_EVENTS: &[&str] = &[
            "run_id",
            "sequence",
            "format_version",
            "kind",
            "node_instance_id",
            "request_hash",
            "payload",
        ];
        const OUTBOX: &[&str] = &[
            "run_id",
            "node_instance_id",
            "effect_id",
            "state",
            "intent_sequence",
            "request_hash",
            "target_idempotent",
            "dedup_scope",
            "receipt",
        ];
        const RUNS_V3: &[&str] = &[
            "run_id",
            "status",
            "next_sequence",
            "spec_digest",
            "frontier_node",
            "terminal_reason",
            "closed_at_ms",
            "input_payload",
        ];
        const INBOX: &[&str] = &["run_id", "message_id", "processed_at_ms"];
        const SNAPSHOTS: &[&str] = &[
            "run_id",
            "snapshot_version",
            "upto_sequence",
            "state",
            "state_digest",
        ];
        const LEASE: &[&str] = &["id", "owner", "epoch", "expires_at_ms"];
        let required: &[(&str, &[&str])] = if user_version >= 3 {
            &[
                ("runs", RUNS_V3),
                ("run_events", RUN_EVENTS),
                ("outbox", OUTBOX),
                ("inbox", INBOX),
                ("snapshots", SNAPSHOTS),
                ("lease", LEASE),
                (
                    "waits",
                    &[
                        "run_id",
                        "node_instance_id",
                        "signal_name",
                        "nonce",
                        "status",
                    ],
                ),
                (
                    "signals",
                    &[
                        "run_id",
                        "nonce",
                        "payload",
                        "decision_label",
                        "expires_at_ms",
                        "deposited_at_ms",
                        "consumed",
                    ],
                ),
                ("timers", &["run_id", "timer_id", "deadline_ms", "fired"]),
                ("clock_watermark", &["id", "max_now_ms"]),
            ]
        } else if user_version >= 2 {
            &[
                ("runs", RUNS_V2),
                ("run_events", RUN_EVENTS),
                ("outbox", OUTBOX),
                ("inbox", INBOX),
                ("snapshots", SNAPSHOTS),
                ("lease", LEASE),
            ]
        } else {
            &[
                ("runs", RUNS_V1),
                ("run_events", RUN_EVENTS),
                ("outbox", OUTBOX),
            ]
        };
        for (table, columns) in required {
            let present: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    params![table],
                    |row| row.get(0),
                )
                .map_err(|e| StoreError::InvalidBackup {
                    reason: format!("cannot read schema: {e}"),
                })?;
            if present != 1 {
                return Err(StoreError::InvalidBackup {
                    reason: format!("missing table {table} for schema version {user_version}"),
                });
            }
            // Table NAMES are not a schema: six right-named empty shells
            // passed the previous check (Codex r2 P1). The claimed
            // version's COLUMN inventory must be present too (rusqlite's
            // typed pragma call — no SQL built from strings). Column TYPES
            // are not compared: a right-shaped store with lying types
            // fails loudly downstream at the first typed read, which is
            // fail-closed — the guard here is against silently replacing a
            // healthy store with a structurally alien file.
            let mut found: Vec<(String, i64)> = Vec::new();
            conn.pragma(None, "table_info", table, |row| {
                found.push((row.get::<_, String>(1)?, row.get::<_, i64>(5)?));
                Ok(())
            })
            .map_err(|e| StoreError::InvalidBackup {
                reason: format!("cannot read columns of {table}: {e}"),
            })?;
            for column in *columns {
                if !found.iter().any(|(c, _)| c == column) {
                    return Err(StoreError::InvalidBackup {
                        reason: format!(
                            "table {table} lacks column {column} for schema version {user_version}"
                        ),
                    });
                }
            }
            // The PRIMARY KEYS the store's semantics rest on (inbox dedup,
            // journal ordering, one-outbox-row identity) must be DECLARED —
            // right-named columns without them degrade silently (Codex r3).
            let expected_pk: &[&str] = match *table {
                "runs" => &["run_id"],
                "run_events" => &["run_id", "sequence"],
                "outbox" => &["run_id", "node_instance_id", "effect_id"],
                "inbox" => &["run_id", "message_id"],
                "snapshots" => &["run_id"],
                "lease" => &["id"],
                "waits" => &["run_id", "node_instance_id"],
                "signals" => &["run_id", "nonce"],
                "timers" => &["run_id", "timer_id"],
                "clock_watermark" => &["id"],
                _ => &[],
            };
            let mut actual_pk: Vec<&str> = found
                .iter()
                .filter(|(_, pk)| *pk > 0)
                .map(|(c, _)| c.as_str())
                .collect();
            actual_pk.sort_unstable();
            let mut wanted_pk: Vec<&str> = expected_pk.to_vec();
            wanted_pk.sort_unstable();
            if actual_pk != wanted_pk {
                // EXACT set, not membership: a WIDENED key (extra pk column)
                // would legalise duplicates the semantics forbid (Codex r4).
                return Err(StoreError::InvalidBackup {
                    reason: format!(
                        "table {table}: primary key is {actual_pk:?}, expected {wanted_pk:?}"
                    ),
                });
            }
            // Child tables must DECLARE their foreign key to runs — the
            // inbox's unknown-run refusal is this constraint, nothing else.
            let needs_fk = matches!(
                *table,
                "run_events" | "outbox" | "inbox" | "snapshots" | "waits" | "signals" | "timers"
            );
            if needs_fk {
                // The guarantee is precisely "run_id REFERENCES runs" — a
                // mistargeted homonym FK would let record_inbox_message
                // accept unknown runs silently (Codex r4).
                // Grouped by constraint id and required to be MONO-column
                // (a composite FK with a NULLable extra child column stops
                // constraining run_id entirely — Codex r5): some constraint
                // must consist of EXACTLY one row, seq 0, run_id → runs(run_id).
                let mut fk_rows: Vec<(i64, i64, String, String, Option<String>)> = Vec::new();
                conn.pragma(None, "foreign_key_list", table, |row| {
                    fk_rows.push((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ));
                    Ok(())
                })
                .map_err(|e| StoreError::InvalidBackup {
                    reason: format!("cannot read foreign keys of {table}: {e}"),
                })?;
                let fk_ok = fk_rows.iter().any(|(id, seq, target, from, to)| {
                    *seq == 0
                        && target == "runs"
                        && from == "run_id"
                        && to.as_deref().unwrap_or("run_id") == "run_id"
                        && fk_rows.iter().filter(|(other, ..)| other == id).count() == 1
                });
                if !fk_ok {
                    return Err(StoreError::InvalidBackup {
                        reason: format!(
                            "table {table} lacks a single-column run_id REFERENCES runs(run_id) foreign key"
                        ),
                    });
                }
            }
        }
        // The reducer's one-effect-per-node assumption is enforced by this
        // unique index — and a HOMONYM is not the index: uniqueness and the
        // exact indexed columns are checked, not the name alone (Codex r3).
        let mut index_flags: Option<(i64, i64)> = None;
        conn.pragma(None, "index_list", "outbox", |row| {
            if row.get::<_, String>(1)? == "outbox_one_effect_per_node" {
                index_flags = Some((row.get::<_, i64>(2)?, row.get::<_, i64>(4)?));
            }
            Ok(())
        })
        .map_err(|e| StoreError::InvalidBackup {
            reason: format!("cannot read outbox indexes: {e}"),
        })?;
        match index_flags {
            None => {
                return Err(StoreError::InvalidBackup {
                    reason: "outbox lacks index outbox_one_effect_per_node".to_string(),
                });
            }
            Some((unique, _)) if unique != 1 => {
                return Err(StoreError::InvalidBackup {
                    reason: "index outbox_one_effect_per_node is not UNIQUE".to_string(),
                });
            }
            // A PARTIAL unique index (WHERE clause) only deduplicates the
            // rows its predicate covers — not the invariant (Codex r4).
            Some((_, partial)) if partial != 0 => {
                return Err(StoreError::InvalidBackup {
                    reason: "index outbox_one_effect_per_node is PARTIAL".to_string(),
                });
            }
            Some(_) => {}
        }
        let mut indexed: Vec<String> = Vec::new();
        conn.pragma(None, "index_info", "outbox_one_effect_per_node", |row| {
            indexed.push(row.get::<_, String>(2)?);
            Ok(())
        })
        .map_err(|e| StoreError::InvalidBackup {
            reason: format!("cannot read index columns: {e}"),
        })?;
        indexed.sort();
        if indexed != ["node_instance_id", "run_id"] {
            return Err(StoreError::InvalidBackup {
                reason: format!(
                    "index outbox_one_effect_per_node covers {indexed:?}, expected (run_id, node_instance_id)"
                ),
            });
        }
        // Structural integrity of the file itself, before it replaces
        // anything. `quick_check` returns a single "ok" row on a sound
        // database and rows of findings otherwise.
        let quick: String = conn
            .pragma_query_value(None, "quick_check", |row| row.get(0))
            .map_err(|e| StoreError::InvalidBackup {
                reason: format!("cannot run quick_check: {e}"),
            })?;
        if quick != "ok" {
            return Err(StoreError::InvalidBackup {
                reason: format!("quick_check: {quick}"),
            });
        }
        drop(conn);

        // Siblings BEFORE the copy (review D1-3): a crash between the two
        // steps must never leave the restored main file next to the OLD
        // database's `-wal` — deleting first makes the bad interleaving
        // "old main + no wal" (harmless) instead of "new main + old wal"
        // (corruption on replay).
        let name = store_path
            .file_name()
            .ok_or_else(|| StoreError::InvalidInput {
                what: "store path",
                reason: "path has no file name".to_string(),
            })?
            .to_string_lossy()
            .into_owned();
        for suffix in ["-wal", "-shm"] {
            let sibling = store_path.with_file_name(format!("{name}{suffix}"));
            if sibling.exists() {
                std::fs::remove_file(&sibling)?;
            }
        }
        std::fs::copy(backup, store_path)?;
        Ok(())
    }

    /// HARNESS entry point — append a RAW journal row with an arbitrary
    /// format version and payload text, bypassing the typed door, so the
    /// upcasting and downgrade guards can be exercised against rows no
    /// current API can produce (an "older" synthetic shape, a "newer"
    /// version). Behind `spike-faults`: a normal build cannot reach it.
    #[cfg(feature = "spike-faults")]
    pub fn append_raw_event_for_harness(
        &self,
        run_id: &str,
        format_version: i64,
        kind: &str,
        payload: &str,
    ) -> Result<u64, StoreError> {
        self.in_immediate_tx(|conn| {
            let sequence: i64 = conn.query_row(
                "SELECT next_sequence FROM runs WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )?;
            conn.execute(
                "INSERT INTO run_events
                     (run_id, sequence, format_version, kind, node_instance_id, request_hash, payload)
                 VALUES (?1, ?2, ?3, ?4, NULL, NULL, ?5)",
                params![run_id, sequence, format_version, kind, payload],
            )?;
            conn.execute(
                "UPDATE runs SET next_sequence = ?1 WHERE run_id = ?2",
                params![sequence + 1, run_id],
            )?;
            Ok(sequence as u64)
        })
    }

    /// `BEGIN IMMEDIATE` / `COMMIT` / `ROLLBACK` around one closure — the
    /// explicit-transaction shape already used by the coordinator feed
    /// (`nexus-coordinator-rs/src/public_feed.rs`).
    ///
    /// When this handle holds a lease, ownership is re-asserted INSIDE the
    /// transaction, FIRST, with an epoch compare-and-set: a no-op
    /// `UPDATE … WHERE owner = ? AND epoch = ?` whose affected-row count is
    /// the whole check. `rows_affected != 1` means a takeover happened; the
    /// transaction aborts and NOTHING this closure wrote survives — that is
    /// the G-SINGLE-WRITER guarantee. The wall clock plays no part here:
    /// safety is the monotone epoch, expiry is only the takeover heuristic.
    /// The check never lives in a separate read transaction (a read-to-write
    /// upgrade can return `SQLITE_BUSY` without invoking the busy handler —
    /// `BEGIN IMMEDIATE` takes the write lock up front).
    ///
    /// NOT re-entrant: SQLite refuses a nested `BEGIN IMMEDIATE`. This
    /// matters because one caller runs a [`FaultHook`] inside the closure —
    /// a hook that called back into the store would deadlock its own
    /// transaction. Hooks must only observe, halt, or kill the process.
    fn in_immediate_tx<T>(
        &self,
        body: impl FnOnce(&Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.in_immediate_tx_unfenced(|conn| {
            if let Some((owner, epoch)) = &self.active_lease {
                let matched = conn.execute(
                    "UPDATE lease SET epoch = epoch WHERE id = 1 AND owner = ?1 AND epoch = ?2",
                    params![owner, epoch],
                )?;
                if matched != 1 {
                    return Err(StoreError::LeaseLost {
                        owner: owner.clone(),
                        epoch: *epoch,
                    });
                }
            }
            body(conn)
        })
    }

    /// The raw transaction wrapper, WITHOUT the lease fence. Reserved for
    /// lease management itself (acquiring a lease is how a handle EARNS the
    /// fence — it cannot already pass it).
    fn in_immediate_tx_unfenced<T>(
        &self,
        body: impl FnOnce(&Connection) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.conn.execute_batch("BEGIN IMMEDIATE")?;
        match body(&self.conn) {
            Ok(value) => {
                self.conn.execute_batch("COMMIT")?;
                Ok(value)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }
}

/// The value the snapshot digest is computed over: the OWNING RUN plus the
/// whole served tuple (format version, sequence bound, state), so that
/// neither a field of [`Snapshot`] nor the cache's contextual identity
/// escapes authentication — without `run_id`, copying another run's intact
/// tuple+digest into this run's row would verify (Codex r1 L5). One
/// function used by BOTH the write and the verify path — two hand-built
/// subjects could silently diverge.
fn snapshot_digest_subject(
    run_id: &str,
    version: i64,
    upto_sequence: u64,
    state: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "run": run_id,
        "v": version,
        "upto": upto_sequence,
        "state": state,
    })
}

/// Append one entry and advance the run's sequence. Callers MUST already be
/// inside a transaction — the atomicity of "entry + frontier + outbox" is
/// the guarantee under test.
fn append_event(
    conn: &Connection,
    run_id: &str,
    node_instance_id: Option<&str>,
    request_hash: Option<&str>,
    kind: &RunEventKind,
) -> Result<u64, StoreError> {
    let sequence: i64 = conn.query_row(
        "SELECT next_sequence FROM runs WHERE run_id = ?1",
        params![run_id],
        |row| row.get(0),
    )?;
    let payload = serde_json::to_string(kind)
        .map_err(|e| StoreError::Corrupt(format!("serialize event kind: {e}")))?;
    conn.execute(
        "INSERT INTO run_events
             (run_id, sequence, format_version, kind, node_instance_id, request_hash, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            run_id,
            sequence,
            i64::from(RUN_EVENT_FORMAT_VERSION),
            kind.discriminant(),
            node_instance_id,
            request_hash,
            payload
        ],
    )?;
    conn.execute(
        "UPDATE runs SET next_sequence = ?1 WHERE run_id = ?2",
        params![sequence + 1, run_id],
    )?;
    Ok(sequence as u64)
}

#[allow(clippy::too_many_arguments)]
fn insert_pending_outbox(
    conn: &Connection,
    run_id: &str,
    node_instance_id: &str,
    effect_id: &str,
    request_hash: &str,
    target_idempotent: bool,
    dedup_scope: &str,
    intent_sequence: Option<u64>,
) -> Result<(), StoreError> {
    let sequence = intent_sequence
        .map(|s| {
            i64::try_from(s)
                .map_err(|_| StoreError::Corrupt(format!("intent sequence out of range: {s}")))
        })
        .transpose()?;
    conn.execute(
        "INSERT INTO outbox
             (run_id, node_instance_id, effect_id, state, intent_sequence, request_hash,
              target_idempotent, dedup_scope)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            run_id,
            node_instance_id,
            effect_id,
            OUTBOX_PENDING,
            sequence,
            request_hash,
            i64::from(target_idempotent),
            dedup_scope
        ],
    )?;
    Ok(())
}
