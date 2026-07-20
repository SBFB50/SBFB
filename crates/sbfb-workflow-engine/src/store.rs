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
//! ## Retention
//!
//! The journal and the outbox grow without bound here. Bounds, retention and
//! garbage collection are OUT OF SCOPE for this spike (routed to the run
//! store phase, `G-STORE`) and are called out in the phase artifact rather
//! than silently inherited.

use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};
use rusqlite_migration::{M, Migrations};

use crate::event::{RUN_EVENT_FORMAT_VERSION, RunEvent, RunEventKind, TerminalReason};
use crate::fault::{FaultAction, FaultHook, FaultPoint};

/// Schema of the spike store. One migration: the spike has no released
/// on-disk state to migrate FROM. Table names (`runs`, `run_events`,
/// `outbox`) are chosen to be extensible by the full run store without a
/// DROP.
static MIGRATIONS: &[M<'static>] = &[M::up(
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
)];

/// An effect is queued but not confirmed.
const OUTBOX_PENDING: &str = "pending";
/// An effect returned its receipt.
const OUTBOX_APPLIED: &str = "applied";

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
}

/// Handle on one run store file.
pub struct RunStore {
    conn: Connection,
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
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let mut store = Self { conn };
        store.assert_pragma_text("journal_mode", "wal")?;
        store.assert_pragma_i64("synchronous", 2, "FULL")?;
        store.assert_pragma_i64("foreign_keys", 1, "ON")?;

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
    /// `faults` is consulted BETWEEN the two writes, inside the transaction.
    /// That fault point is the only thing that makes this method's guarantee
    /// falsifiable: halting there must roll back BOTH writes, so a test can
    /// assert that neither survived. Without an injection point inside the
    /// transaction, splitting this method into two transactions would leave
    /// every test green.
    ///
    /// The parameter list is long because it IS the outbox row: every one of
    /// these values is a column written in this single transaction. Bundling
    /// them into a struct would add an indirection without removing anything
    /// a caller has to supply.
    #[allow(clippy::too_many_arguments)]
    pub fn record_intent_atomic(
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
    /// `faults` fires BETWEEN the append and the transition, inside the
    /// transaction — the same falsifiability device as
    /// [`RunStore::record_intent_atomic`].
    #[allow(clippy::too_many_arguments)]
    pub fn record_completion(
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
    pub fn record_terminal(
        &self,
        run_id: &str,
        node_instance_id: Option<&str>,
        reason: TerminalReason,
        expected_hash: Option<&str>,
        got_hash: Option<&str>,
    ) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
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
            conn.execute(
                "UPDATE runs SET status = ?1, terminal_reason = ?2 WHERE run_id = ?3",
                params![RunStatus::Terminal.as_str(), reason.as_str(), run_id],
            )?;
            Ok(())
        })
    }

    /// Close a run that reached its last step.
    pub fn record_run_completed(&self, run_id: &str) -> Result<(), StoreError> {
        self.in_immediate_tx(|conn| {
            append_event(conn, run_id, None, None, &RunEventKind::RunCompleted)?;
            conn.execute(
                "UPDATE runs SET status = ?1 WHERE run_id = ?2",
                params![RunStatus::Completed.as_str(), run_id],
            )?;
            Ok(())
        })
    }

    /// Every journal entry of a run, in sequence order.
    pub fn load_events(&self, run_id: &str) -> Result<Vec<RunEvent>, StoreError> {
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
            let kind: RunEventKind = serde_json::from_str(&payload)
                .map_err(|e| StoreError::Corrupt(format!("event payload at {sequence}: {e}")))?;
            events.push(RunEvent {
                // READ from the row, not re-injected from the current
                // constant: a version the reader supplies is not a version,
                // it is a guess that always agrees with itself.
                format_version: u16::try_from(format_version).map_err(|_| {
                    StoreError::Corrupt(format!(
                        "event format version out of range: {format_version}"
                    ))
                })?,
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

    /// `BEGIN IMMEDIATE` / `COMMIT` / `ROLLBACK` around one closure — the
    /// explicit-transaction shape already used by the coordinator feed
    /// (`nexus-coordinator-rs/src/public_feed.rs`).
    ///
    /// NOT re-entrant: SQLite refuses a nested `BEGIN IMMEDIATE`. This
    /// matters because one caller runs a [`FaultHook`] inside the closure —
    /// a hook that called back into the store would deadlock its own
    /// transaction. Hooks must only observe, halt, or kill the process.
    fn in_immediate_tx<T>(
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
