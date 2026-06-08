// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator SQLite persistence layer.
//!
//! Owns `~/.sbfb/coordinator.db` with schema versioning to prevent
//! silent drift during the Python→Rust gradual migration (G1 D3 ⚠️).

use std::path::Path;

use rusqlite::Connection;
use rusqlite::OptionalExtension;
use rusqlite_migration::{M, Migrations};

use crate::error::CoordinatorError;
use crate::provenance::ProvenanceRecord;
use crate::types::{KudosEntry, TaskRecord, TaskResultRow, TaskStatus};

static MIGRATIONS: &[M<'static>] = &[
    M::up(
        "CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER NOT NULL
    );
    INSERT INTO schema_version (version) VALUES (1);

    CREATE TABLE IF NOT EXISTS tasks (
        task_id       TEXT PRIMARY KEY,
        status        TEXT NOT NULL DEFAULT 'pending',
        project_id    TEXT NOT NULL,
        model         TEXT NOT NULL,
        created_at    INTEGER NOT NULL,
        updated_at    INTEGER NOT NULL,
        task_hash     TEXT NOT NULL,
        worker_node_id TEXT,
        result_hash   TEXT
    );

    CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks (status);
    CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks (project_id);

    CREATE TABLE IF NOT EXISTS kudos (
        entry_id      TEXT PRIMARY KEY,
        worker_node_id TEXT NOT NULL,
        task_id       TEXT NOT NULL,
        project_id    TEXT NOT NULL,
        amount        INTEGER NOT NULL,
        created_at    INTEGER NOT NULL,
        prev_hash     TEXT NOT NULL,
        entry_hash    TEXT NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_kudos_worker ON kudos (worker_node_id);
    CREATE INDEX IF NOT EXISTS idx_kudos_project ON kudos (project_id);",
    ),
    M::up(
        "CREATE TABLE IF NOT EXISTS pow_task_counts (
        consumer_id    TEXT NOT NULL,
        model_id       TEXT NOT NULL,
        count          INTEGER NOT NULL DEFAULT 0,
        last_reset_utc TEXT NOT NULL,
        PRIMARY KEY (consumer_id, model_id)
    );",
    ),
    M::up(
        "CREATE TABLE IF NOT EXISTS contributor_attestations (
        id                   INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id           TEXT NOT NULL,
        contributor_node_id  TEXT NOT NULL,
        first_deploy_ts      INTEGER NOT NULL,
        commit_sha           TEXT NOT NULL,
        repo_url             TEXT NOT NULL,
        coord_sig_hex        TEXT NOT NULL,
        attestation_json     TEXT NOT NULL,
        recorded_at          INTEGER NOT NULL,
        UNIQUE (project_id, contributor_node_id)
    );
    CREATE INDEX IF NOT EXISTS idx_contrib_project ON contributor_attestations(project_id);
    CREATE INDEX IF NOT EXISTS idx_contrib_node ON contributor_attestations(contributor_node_id);

    CREATE TABLE IF NOT EXISTS invites (
        id            TEXT PRIMARY KEY,
        wire          TEXT NOT NULL UNIQUE,
        scope         TEXT NOT NULL,
        project_id    TEXT NOT NULL,
        project_name  TEXT NOT NULL,
        expires_at    INTEGER NOT NULL,
        max_uses      INTEGER,
        uses_count    INTEGER NOT NULL DEFAULT 0,
        revoked_at    INTEGER,
        note          TEXT,
        created_at    INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_invites_expires ON invites(expires_at);",
    ),
    M::up(
        "CREATE TABLE IF NOT EXISTS quarantine_messages (
        id                  INTEGER PRIMARY KEY AUTOINCREMENT,
        topic               TEXT NOT NULL,
        sender_pubkey_hex   TEXT NOT NULL,
        payload_json        TEXT NOT NULL,
        received_at         INTEGER NOT NULL,
        rate_strikes        INTEGER NOT NULL,
        pow_status          TEXT NOT NULL,
        flush_status        TEXT NOT NULL DEFAULT 'pending'
    );
    CREATE INDEX IF NOT EXISTS idx_quarantine_received ON quarantine_messages(received_at);
    CREATE INDEX IF NOT EXISTS idx_quarantine_sender ON quarantine_messages(sender_pubkey_hex);

    CREATE TABLE IF NOT EXISTS delayed_uploads (
        upload_id           TEXT PRIMARY KEY,
        deliver_at          REAL NOT NULL,
        task_payload_json   TEXT NOT NULL,
        enqueued_at         REAL NOT NULL,
        status              TEXT NOT NULL DEFAULT 'pending'
    );
    CREATE INDEX IF NOT EXISTS idx_delayed_uploads_deliver ON delayed_uploads(deliver_at);",
    ),
    M::up(
        "ALTER TABLE tasks ADD COLUMN task_type TEXT NOT NULL DEFAULT 'inference';
    ALTER TABLE tasks ADD COLUMN redundancy_factor INTEGER NOT NULL DEFAULT 1;

    CREATE TABLE IF NOT EXISTS task_results (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        task_id     TEXT NOT NULL,
        worker_id   TEXT NOT NULL,
        sha256      TEXT NOT NULL,
        created_at  INTEGER NOT NULL,
        UNIQUE (task_id, worker_id)
    );
    CREATE INDEX IF NOT EXISTS idx_task_results_task ON task_results(task_id);",
    ),
    // M6: gossip outbox persistence (Sprint 56 Phase A)
    M::up(
        "CREATE TABLE IF NOT EXISTS gossip_outbox (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        envelope   BLOB NOT NULL,
        added_at   INTEGER NOT NULL
    );",
    ),
    // M7: per-app key-value storage persistence (Sprint 57 Phase B)
    M::up(
        "CREATE TABLE IF NOT EXISTS app_storage (
        app_name   TEXT NOT NULL,
        key        TEXT NOT NULL,
        value      TEXT NOT NULL,
        PRIMARY KEY (app_name, key)
    );",
    ),
    // M8: per-app iroh-docs storage namespace persistence (Sprint 58 Phase C)
    M::up(
        "CREATE TABLE IF NOT EXISTS storage_namespaces (
        app_name       TEXT PRIMARY KEY,
        namespace_id   BLOB NOT NULL,
        doc_ticket     TEXT
    );",
    ),
    // M9: public feed append-only log (Sprint 61 Phase B)
    M::up(
        "CREATE TABLE IF NOT EXISTS public_feed (
        seq         INTEGER PRIMARY KEY AUTOINCREMENT,
        op_type     TEXT NOT NULL,
        payload     TEXT NOT NULL,
        author      TEXT NOT NULL,
        signature   TEXT NOT NULL,
        entry_hash  TEXT NOT NULL,
        prev_hash   TEXT NOT NULL,
        created_at  INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_feed_created ON public_feed(created_at);",
    ),
    // M10: feed cursor for materializer checkpoint (Sprint 61 Phase C)
    M::up(
        "CREATE TABLE IF NOT EXISTS feed_cursor (
        id               INTEGER PRIMARY KEY CHECK (id = 1),
        last_seq         INTEGER NOT NULL,
        last_entry_hash  TEXT NOT NULL
    );",
    ),
    // M11: unique index on entry_hash for feed sync dedup (Sprint 62 Phase B)
    M::up("CREATE UNIQUE INDEX IF NOT EXISTS idx_feed_entry_hash ON public_feed(entry_hash);"),
    // M12: provenance records for verified deploy (Sprint 63 Phase B)
    M::up(
        "CREATE TABLE IF NOT EXISTS provenance_records (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        project_id      TEXT NOT NULL,
        repo_url        TEXT NOT NULL,
        commit_sha      TEXT NOT NULL,
        artifact_hash   TEXT NOT NULL,
        node_id         TEXT NOT NULL,
        signature       TEXT NOT NULL,
        timestamp       TEXT NOT NULL,
        schema_version  INTEGER NOT NULL DEFAULT 1,
        created_at      INTEGER NOT NULL,
        UNIQUE (project_id, artifact_hash)
    );
    CREATE INDEX IF NOT EXISTS idx_prov_project ON provenance_records(project_id);",
    ),
    // M13: add app_version column to provenance_records (Sprint 64 Phase A)
    M::up("ALTER TABLE provenance_records ADD COLUMN app_version TEXT;"),
    // M14: key rotation persistence (Sprint 66 Phase D)
    M::up(
        "CREATE TABLE IF NOT EXISTS key_rotations (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        old_pubkey      TEXT NOT NULL,
        new_pubkey      TEXT NOT NULL,
        timestamp       INTEGER NOT NULL,
        transition_days INTEGER NOT NULL,
        signature       TEXT NOT NULL,
        reason          TEXT NOT NULL DEFAULT '',
        created_at      INTEGER NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_keyrot_old ON key_rotations(old_pubkey);",
    ),
    // M15: FTS5 search index (Sprint 67 Phase B)
    M::up(
        "CREATE VIRTUAL TABLE IF NOT EXISTS search_index USING fts5(
        project_id UNINDEXED,
        project_name,
        category,
        description,
        op_type UNINDEXED,
        source_type UNINDEXED,
        tokenize='unicode61'
    );",
    ),
    // M16: persist the accepted result text so the Operator's network
    // execution arm can retrieve a completed task's output over HTTP
    // (Sprint 72 Phase D). Local DB schema only — NOT a wire format, so
    // the pre-launch policy is unaffected. Append-only ALTER (mirrors
    // M5/M13); `rusqlite_migration` tracks `user_version`.
    M::up("ALTER TABLE tasks ADD COLUMN result_text TEXT;"),
    // M17: enrich the FTS5 search index with the provenance triplet so a
    // search hit can drive a fork (Sprint 73 Phase D; the S74 atelier
    // consumes `repo_url@commit_sha` or `archive_hash` as the blob
    // fallback). FTS5 virtual tables cannot `ALTER TABLE ... ADD COLUMN`,
    // so the canonical evolution path is DROP + recreate with the new
    // columns. The four provenance columns + `is_open_source` are
    // UNINDEXED: a 40/64-hex hash is not a natural-language token, so a
    // MATCH against it is meaningless and would only inflate the index —
    // they are returned, never full-text matchable (mirrors the existing
    // `project_id`/`op_type`/`source_type` UNINDEXED columns from M15).
    // Local schema only — NOT a wire format (search_index is never synced
    // over iroh-docs; each node rebuilds it from the feed it received), so
    // the pre-launch policy is unaffected. The drop loses no durable data:
    // the index is integrally reconstructible from `public_feed` (the boot
    // `rebuild_from_feed` repopulates every row, now carrying the triplet).
    M::up(
        "DROP TABLE IF EXISTS search_index;
    CREATE VIRTUAL TABLE search_index USING fts5(
        project_id UNINDEXED,
        project_name,
        category,
        description,
        op_type UNINDEXED,
        source_type UNINDEXED,
        repo_url UNINDEXED,
        commit_sha UNINDEXED,
        archive_hash UNINDEXED,
        provenance_hash UNINDEXED,
        is_open_source UNINDEXED,
        tokenize='unicode61'
    );",
    ),
    // M18 (Sprint 74 Phase D): per-app "keep online" local pin policy. A LOCAL
    // schema overlay, never on the wire: which self-deployed apps THIS node
    // keeps online (blob skip-GC tag + boot re-broadcast gate). An ABSENT row
    // means enabled-by-default, so pre-M18 apps keep their current always-on
    // behaviour (R6 — additive, never aborts boot).
    M::up(
        "CREATE TABLE IF NOT EXISTS keep_online (
        project_id TEXT PRIMARY KEY,
        enabled INTEGER NOT NULL DEFAULT 1,
        archive_hash TEXT,
        pinned_at INTEGER NOT NULL
    );",
    ),
    // M19 (Sprint 74 Phase E): revocable cross-node seed invite ledger.
    // A LOCAL schema overlay, never on the wire — only the opaque `token`
    // id circulates (inside a SeedRequest), never a row. Modelled on the
    // Tailscale share link (revocable in real time by looking it up here)
    // and on the existing `invites` ledger: a node mints a token to let a
    // trusted peer ask it to seed a SPECIFIC app release, and can revoke it
    // at any time. The token is verified by the node that RECEIVES the
    // SeedRequest, against this table. The invite is a capability over a
    // specific `(project_id, archive_hash)` PAIR — NOT a project namespace:
    // the handler rejects a SeedRequest whose archive_hash differs from the
    // one bound here, so an invited peer cannot make the seeder pin foreign
    // content under the app's keep-online tag (review P2). Single-use or
    // reusable+expiry via `max_uses`/`uses_count`.
    M::up(
        "CREATE TABLE IF NOT EXISTS seed_invite (
        token TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        archive_hash TEXT NOT NULL,
        expires_at INTEGER NOT NULL,
        max_uses INTEGER,
        uses_count INTEGER NOT NULL DEFAULT 0,
        revoked_at INTEGER,
        created_at INTEGER NOT NULL
    );",
    ),
];

pub struct StorageNamespaceRow {
    pub namespace_id: Vec<u8>,
    pub doc_ticket: Option<String>,
}

/// Retrievable view of a task's completion result (Sprint 72 Phase D).
///
/// Backs `GET /api/v1/tasks/{id}/result`: `result_text` is the accepted
/// human-readable output (populated on completion for both the single
/// and quorum paths, see [`CoordinatorDb::set_task_result`]); it is
/// `None` while the task is still pending/dispatched or was rejected.
pub struct TaskResultDetail {
    pub status: String,
    pub result_text: Option<String>,
    pub result_hash: Option<String>,
}

/// Outcome of verifying + consuming a seed invite token (Sprint 74
/// Phase E). Only `Ok` authorizes a cross-node seed; every other variant
/// maps to a `SeedResponse` rejection reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeedInviteOutcome {
    /// Token valid and not exhausted; a use was just recorded.
    Ok,
    /// No row for this token.
    NotFound,
    /// The author revoked the token.
    Revoked,
    /// `now >= expires_at`.
    Expired,
    /// `max_uses` reached (single-use already redeemed, or reusable cap hit).
    NoUsesLeft,
}

/// A seed invite row, for the local management UI (mint/list/revoke).
#[derive(Debug, Clone)]
pub struct SeedInviteRow {
    pub token: String,
    pub project_id: String,
    /// The specific archive hash this invite authorizes seeding of.
    pub archive_hash: String,
    pub expires_at: i64,
    pub max_uses: Option<i64>,
    pub uses_count: i64,
    pub revoked_at: Option<i64>,
    pub created_at: i64,
}

pub struct CoordinatorDb {
    conn: Connection,
}

impl std::fmt::Debug for CoordinatorDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinatorDb").finish_non_exhaustive()
    }
}

impl CoordinatorDb {
    pub fn open(path: &Path) -> Result<Self, CoordinatorError> {
        let mut conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Explicit 5s busy timeout: a hot feed reindex (Sprint 73 Phase C) may
        // briefly contend with another writer on the single connection; wait
        // and retry rather than failing fast with SQLITE_BUSY. Made explicit
        // rather than relying on the driver/SQLite implicit default.
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, CoordinatorError> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        // Keep the busy timeout in parity with the on-disk `open` path so test
        // and production share the same locking behaviour (Sprint 73 Phase C).
        conn.busy_timeout(std::time::Duration::from_secs(5))?;

        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn schema_version(&self) -> Result<i64, CoordinatorError> {
        let version: i64 =
            self.conn
                .query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
                    row.get(0)
                })?;
        Ok(version)
    }

    pub fn insert_task(&self, record: &TaskRecord) -> Result<(), CoordinatorError> {
        self.conn.execute(
            "INSERT INTO tasks (task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash, task_type, redundancy_factor)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                record.task_id,
                record.status.as_str(),
                record.project_id,
                record.model,
                record.created_at,
                record.updated_at,
                record.task_hash,
                record.worker_node_id,
                record.result_hash,
                record.task_type,
                record.redundancy_factor,
            ],
        )?;
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash, task_type, redundancy_factor
             FROM tasks WHERE task_id = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![task_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(TaskRecord {
                task_id: row.get(0)?,
                status: TaskStatus::from_str_lossy(&row.get::<_, String>(1)?),
                project_id: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                task_hash: row.get(6)?,
                worker_node_id: row.get(7)?,
                result_hash: row.get(8)?,
                task_type: row.get(9)?,
                redundancy_factor: row.get::<_, u8>(10)?,
            })),
            None => Ok(None),
        }
    }

    pub fn list_tasks(
        &self,
        status: Option<&str>,
        limit: usize,
    ) -> Result<Vec<TaskRecord>, CoordinatorError> {
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match status {
            Some(s) => (
                "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash, task_type, redundancy_factor
                 FROM tasks WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![Box::new(s.to_string()), Box::new(limit as i64)],
            ),
            None => (
                "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash, task_type, redundancy_factor
                 FROM tasks ORDER BY created_at DESC LIMIT ?1",
                vec![Box::new(limit as i64)],
            ),
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(TaskRecord {
                task_id: row.get(0)?,
                status: TaskStatus::from_str_lossy(&row.get::<_, String>(1)?),
                project_id: row.get(2)?,
                model: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                task_hash: row.get(6)?,
                worker_node_id: row.get(7)?,
                result_hash: row.get(8)?,
                task_type: row.get(9)?,
                redundancy_factor: row.get::<_, u8>(10)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn update_task_status(
        &self,
        task_id: &str,
        status: TaskStatus,
        updated_at: u64,
    ) -> Result<bool, CoordinatorError> {
        let changed = self.conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE task_id = ?3",
            rusqlite::params![status.as_str(), updated_at, task_id],
        )?;
        Ok(changed > 0)
    }

    /// Mark a task completed, recording the worker, the provenance
    /// `result_hash`, AND the human-readable `result_text` (Sprint 72
    /// Phase D). The text is what `GET /api/v1/tasks/{id}/result`
    /// returns: the single-result path passes the worker's
    /// `payload.result_text`, the quorum path passes the agreed text
    /// (`best_hash`, which on that path IS the `result_text`).
    pub fn set_task_result(
        &self,
        task_id: &str,
        worker_node_id: &str,
        result_hash: &str,
        result_text: &str,
        updated_at: u64,
    ) -> Result<bool, CoordinatorError> {
        let changed = self.conn.execute(
            "UPDATE tasks SET status = 'completed', worker_node_id = ?1, result_hash = ?2, result_text = ?3, updated_at = ?4
             WHERE task_id = ?5 AND status IN ('pending', 'dispatched', 'awaiting_quorum')",
            rusqlite::params![worker_node_id, result_hash, result_text, updated_at, task_id],
        )?;
        Ok(changed > 0)
    }

    /// Read a completed task's retrievable result (Sprint 72 Phase D).
    /// Returns `None` if the task does not exist. `result_text` inside
    /// the detail is `None` while the task is not yet completed.
    pub fn get_task_result(
        &self,
        task_id: &str,
    ) -> Result<Option<TaskResultDetail>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, result_text, result_hash FROM tasks WHERE task_id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![task_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(TaskResultDetail {
                status: row.get(0)?,
                result_text: row.get(1)?,
                result_hash: row.get(2)?,
            })),
            None => Ok(None),
        }
    }

    pub fn insert_task_result(
        &self,
        task_id: &str,
        worker_id: &str,
        sha256: &str,
        created_at: u64,
    ) -> Result<bool, CoordinatorError> {
        let changed = self.conn.execute(
            "INSERT OR IGNORE INTO task_results (task_id, worker_id, sha256, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![task_id, worker_id, sha256, created_at],
        )?;
        Ok(changed > 0)
    }

    pub fn get_task_results(&self, task_id: &str) -> Result<Vec<TaskResultRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, worker_id, sha256, created_at
             FROM task_results WHERE task_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![task_id], |row| {
            Ok(TaskResultRow {
                task_id: row.get(0)?,
                worker_id: row.get(1)?,
                sha256: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn insert_kudos(&self, entry: &KudosEntry) -> Result<(), CoordinatorError> {
        self.conn.execute(
            "INSERT INTO kudos (entry_id, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                entry.entry_id,
                entry.worker_node_id,
                entry.task_id,
                entry.project_id,
                entry.amount,
                entry.created_at,
                entry.prev_hash,
                entry.entry_hash,
            ],
        )?;
        Ok(())
    }

    pub fn get_worker_kudos_total(&self, worker_node_id: &str) -> Result<u64, CoordinatorError> {
        let total: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM kudos WHERE worker_node_id = ?1",
            rusqlite::params![worker_node_id],
            |row| row.get(0),
        )?;
        Ok(total as u64)
    }

    pub fn get_project_kudos_total(&self, project_id: &str) -> Result<u64, CoordinatorError> {
        let total: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM kudos WHERE project_id = ?1",
            rusqlite::params![project_id],
            |row| row.get(0),
        )?;
        Ok(total as u64)
    }

    pub fn get_last_entry_hash(
        &self,
        project_id: &str,
    ) -> Result<Option<String>, CoordinatorError> {
        // rowid tiebreaker ensures deterministic ordering when
        // multiple entries share the same created_at second.
        let mut stmt = self.conn.prepare(
            "SELECT entry_hash FROM kudos WHERE project_id = ?1 ORDER BY created_at DESC, rowid DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![project_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn get_project_entries(
        &self,
        project_id: &str,
    ) -> Result<Vec<KudosEntry>, CoordinatorError> {
        // rowid tiebreaker: same rationale as get_last_entry_hash.
        let mut stmt = self.conn.prepare(
            "SELECT entry_id, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash
             FROM kudos WHERE project_id = ?1 ORDER BY created_at ASC, rowid ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok(KudosEntry {
                entry_id: row.get(0)?,
                worker_node_id: row.get(1)?,
                task_id: row.get(2)?,
                project_id: row.get(3)?,
                amount: row.get::<_, i64>(4)? as u64,
                created_at: row.get::<_, i64>(5)? as u64,
                prev_hash: row.get(6)?,
                entry_hash: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn load_outbox(&self) -> Result<Vec<Vec<u8>>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT envelope FROM gossip_outbox ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn insert_outbox(&self, envelope: &[u8]) -> Result<(), CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO gossip_outbox (envelope, added_at) VALUES (?1, ?2)",
            rusqlite::params![envelope, now as i64],
        )?;
        Ok(())
    }

    pub fn clear_outbox(&self) -> Result<(), CoordinatorError> {
        self.conn.execute("DELETE FROM gossip_outbox", [])?;
        Ok(())
    }

    // --- M18 keep_online (Sprint 74 Phase D): per-app local pin policy ---

    /// Set (or clear) a per-app keep-online pin. `archive_hash` records the blob
    /// the pin tags (for the skip-GC tag); pass the deployed app's archive hash.
    /// `INSERT OR REPLACE` keeps a single row per `project_id`.
    pub fn set_keep_online(
        &self,
        project_id: &str,
        enabled: bool,
        archive_hash: Option<&str>,
    ) -> Result<(), CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.conn.execute(
            "INSERT OR REPLACE INTO keep_online (project_id, enabled, archive_hash, pinned_at) \
             VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![project_id, enabled as i64, archive_hash, now as i64],
        )?;
        Ok(())
    }

    /// Return the keep-online state for a project: `Some((enabled, archive_hash))`
    /// if a row exists, `None` otherwise (absent = enabled-by-default, R6).
    pub fn get_keep_online(
        &self,
        project_id: &str,
    ) -> Result<Option<(bool, Option<String>)>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT enabled, archive_hash FROM keep_online WHERE project_id = ?1")?;
        let mut rows = stmt.query(rusqlite::params![project_id])?;
        if let Some(row) = rows.next()? {
            let enabled: i64 = row.get(0)?;
            let archive_hash: Option<String> = row.get(1)?;
            Ok(Some((enabled != 0, archive_hash)))
        } else {
            Ok(None)
        }
    }

    /// List the `project_id`s a node has EXPLICITLY turned off (`enabled = 0`).
    /// The boot re-broadcast gate skips these; an absent row is never returned
    /// (those stay enabled-by-default).
    pub fn list_keep_online_disabled(&self) -> Result<Vec<String>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT project_id FROM keep_online WHERE enabled = 0")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // --- M19 seed_invite (Sprint 74 Phase E): revocable seed invites ---

    /// Mint a revocable seed invite token authorizing a trusted peer to
    /// enrol as a seeder of `project_id`. `expires_at` is unix seconds;
    /// `max_uses == None` means unlimited (reusable until expiry/revoke).
    /// `INSERT OR REPLACE` so re-minting the same `token` resets it.
    pub fn mint_seed_invite(
        &self,
        token: &str,
        project_id: &str,
        archive_hash: &str,
        expires_at: i64,
        max_uses: Option<i64>,
    ) -> Result<(), CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        self.conn.execute(
            "INSERT OR REPLACE INTO seed_invite \
             (token, project_id, archive_hash, expires_at, max_uses, uses_count, revoked_at, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL, ?6)",
            rusqlite::params![token, project_id, archive_hash, expires_at, max_uses, now],
        )?;
        Ok(())
    }

    /// Verify a seed invite token for the `(project_id, archive_hash)` PAIR
    /// at `now` (unix secs) and, if valid, record one use atomically.
    /// Returns the [`SeedInviteOutcome`]. A token bound to a different
    /// project OR a different archive_hash than the one being requested is
    /// treated as [`SeedInviteOutcome::NotFound`] (it does not authorize
    /// THIS content) — this is what prevents an invited peer from making
    /// the seeder pin foreign content under the app's tag (review P2). The
    /// row type is inferred from the closure (no written tuple type — keeps
    /// clippy::type_complexity off this hot path).
    pub fn consume_seed_invite(
        &self,
        token: &str,
        project_id: &str,
        archive_hash: &str,
        now: i64,
    ) -> Result<SeedInviteOutcome, CoordinatorError> {
        let row = self
            .conn
            .query_row(
                "SELECT project_id, archive_hash, expires_at, max_uses, uses_count, revoked_at \
                 FROM seed_invite WHERE token = ?1",
                rusqlite::params![token],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                        r.get::<_, Option<i64>>(3)?,
                        r.get::<_, i64>(4)?,
                        r.get::<_, Option<i64>>(5)?,
                    ))
                },
            )
            .optional()?;
        let Some((row_project_id, row_archive_hash, expires_at, max_uses, uses_count, revoked_at)) =
            row
        else {
            return Ok(SeedInviteOutcome::NotFound);
        };
        // The token must authorize THIS exact (project, content) pair.
        if row_project_id != project_id || row_archive_hash != archive_hash {
            return Ok(SeedInviteOutcome::NotFound);
        }
        if revoked_at.is_some() {
            return Ok(SeedInviteOutcome::Revoked);
        }
        if now >= expires_at {
            return Ok(SeedInviteOutcome::Expired);
        }
        if let Some(max) = max_uses {
            if uses_count >= max {
                return Ok(SeedInviteOutcome::NoUsesLeft);
            }
        }
        self.conn.execute(
            "UPDATE seed_invite SET uses_count = uses_count + 1 WHERE token = ?1",
            rusqlite::params![token],
        )?;
        Ok(SeedInviteOutcome::Ok)
    }

    /// Revoke a seed invite token. Returns true if a still-active token
    /// was revoked (false if unknown or already revoked).
    pub fn revoke_seed_invite(&self, token: &str) -> Result<bool, CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0) as i64;
        let changed = self.conn.execute(
            "UPDATE seed_invite SET revoked_at = ?1 WHERE token = ?2 AND revoked_at IS NULL",
            rusqlite::params![now, token],
        )?;
        Ok(changed > 0)
    }

    /// List seed invites for a project (most recent first), for the
    /// local management UI.
    pub fn list_seed_invites(
        &self,
        project_id: &str,
    ) -> Result<Vec<SeedInviteRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT token, project_id, archive_hash, expires_at, max_uses, uses_count, revoked_at, created_at \
             FROM seed_invite WHERE project_id = ?1 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |r| {
            Ok(SeedInviteRow {
                token: r.get(0)?,
                project_id: r.get(1)?,
                archive_hash: r.get(2)?,
                expires_at: r.get(3)?,
                max_uses: r.get(4)?,
                uses_count: r.get(5)?,
                revoked_at: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn load_all_storage(
        &self,
    ) -> Result<
        std::collections::HashMap<String, std::collections::HashMap<String, serde_json::Value>>,
        CoordinatorError,
    > {
        let mut stmt = self
            .conn
            .prepare("SELECT app_name, key, value FROM app_storage")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut map: std::collections::HashMap<
            String,
            std::collections::HashMap<String, serde_json::Value>,
        > = std::collections::HashMap::new();
        for row in rows {
            let (app_name, key, value_str) = row?;
            let value: serde_json::Value =
                serde_json::from_str(&value_str).unwrap_or(serde_json::Value::String(value_str));
            map.entry(app_name).or_default().insert(key, value);
        }
        Ok(map)
    }

    pub fn upsert_storage(
        &self,
        app_name: &str,
        key: &str,
        value: &serde_json::Value,
    ) -> Result<(), CoordinatorError> {
        let value_str = serde_json::to_string(value).unwrap_or_default();
        self.conn.execute(
            "INSERT INTO app_storage (app_name, key, value) VALUES (?1, ?2, ?3)
             ON CONFLICT(app_name, key) DO UPDATE SET value = excluded.value",
            rusqlite::params![app_name, key, value_str],
        )?;
        Ok(())
    }

    pub fn delete_storage(&self, app_name: &str, key: &str) -> Result<(), CoordinatorError> {
        self.conn.execute(
            "DELETE FROM app_storage WHERE app_name = ?1 AND key = ?2",
            rusqlite::params![app_name, key],
        )?;
        Ok(())
    }

    pub fn get_storage_namespace(
        &self,
        app_name: &str,
    ) -> Result<Option<StorageNamespaceRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT namespace_id, doc_ticket FROM storage_namespaces WHERE app_name = ?1",
        )?;
        let mut rows = stmt.query(rusqlite::params![app_name])?;
        match rows.next()? {
            Some(row) => Ok(Some(StorageNamespaceRow {
                namespace_id: row.get(0)?,
                doc_ticket: row.get(1)?,
            })),
            None => Ok(None),
        }
    }

    pub fn set_storage_namespace(
        &self,
        app_name: &str,
        namespace_id: &[u8],
        ticket: Option<&str>,
    ) -> Result<(), CoordinatorError> {
        self.conn.execute(
            "INSERT INTO storage_namespaces (app_name, namespace_id, doc_ticket)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(app_name) DO UPDATE SET namespace_id = excluded.namespace_id, doc_ticket = excluded.doc_ticket",
            rusqlite::params![app_name, namespace_id, ticket],
        )?;
        Ok(())
    }

    pub(crate) fn conn(&self) -> &Connection {
        &self.conn
    }

    #[doc(hidden)]
    #[cfg(any(test, feature = "test-support"))]
    pub fn execute_batch_raw(&self, sql: &str) -> Result<(), CoordinatorError> {
        self.conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn get_project_contributors(
        &self,
        project_id: &str,
    ) -> Result<Vec<(String, u64)>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT worker_node_id, SUM(amount) FROM kudos WHERE project_id = ?1 GROUP BY worker_node_id ORDER BY SUM(amount) DESC",
        )?;
        let rows = stmt.query_map(rusqlite::params![project_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
    pub fn list_kudos_entries(
        &self,
        worker_node_id: Option<&str>,
    ) -> Result<Vec<KudosEntry>, CoordinatorError> {
        let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match worker_node_id {
            Some(w) => (
                "SELECT entry_id, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash
                 FROM kudos WHERE worker_node_id = ?1 ORDER BY created_at DESC, rowid DESC",
                vec![Box::new(w.to_string())],
            ),
            None => (
                "SELECT entry_id, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash
                 FROM kudos ORDER BY created_at DESC, rowid DESC",
                vec![],
            ),
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(KudosEntry {
                entry_id: row.get(0)?,
                worker_node_id: row.get(1)?,
                task_id: row.get(2)?,
                project_id: row.get(3)?,
                amount: row.get::<_, i64>(4)? as u64,
                created_at: row.get::<_, i64>(5)? as u64,
                prev_hash: row.get(6)?,
                entry_hash: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn worker_contributions(&self) -> Result<Vec<f64>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT COALESCE(SUM(amount), 0) FROM kudos GROUP BY worker_node_id")?;
        let rows = stmt.query_map([], |row| row.get::<_, f64>(0))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn active_workers_since(
        &self,
        since_epoch: u64,
    ) -> Result<std::collections::HashSet<String>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT worker_node_id FROM kudos WHERE created_at >= ?1")?;
        let rows = stmt.query_map(rusqlite::params![since_epoch as i64], |row| {
            row.get::<_, String>(0)
        })?;
        let mut result = std::collections::HashSet::new();
        for row in rows {
            result.insert(row?);
        }
        Ok(result)
    }

    // -- Provenance methods (Sprint 63 Phase B) --

    pub fn insert_provenance_record(
        &self,
        project_id: &str,
        record: &ProvenanceRecord,
    ) -> Result<(), CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.conn.execute(
            "INSERT OR REPLACE INTO provenance_records
             (project_id, repo_url, commit_sha, artifact_hash, node_id, signature, timestamp, schema_version, created_at, app_version)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            rusqlite::params![
                project_id,
                record.repo_url,
                record.commit_sha,
                record.artifact_hash,
                record.node_id,
                record.signature,
                record.timestamp,
                record.schema_version,
                now as i64,
                record.app_version,
            ],
        )?;
        Ok(())
    }

    pub fn get_provenance_by_project(
        &self,
        project_id: &str,
    ) -> Result<Option<ProvenanceRecord>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT repo_url, commit_sha, artifact_hash, node_id, signature, timestamp, schema_version, app_version
             FROM provenance_records WHERE project_id = ?1
             ORDER BY created_at DESC, rowid DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![project_id])?;
        match rows.next()? {
            Some(row) => Ok(Some(ProvenanceRecord {
                repo_url: row.get(0)?,
                commit_sha: row.get(1)?,
                artifact_hash: row.get(2)?,
                node_id: row.get(3)?,
                signature: row.get(4)?,
                timestamp: row.get(5)?,
                schema_version: row.get::<_, u32>(6)?,
                app_version: row.get(7)?,
            })),
            None => Ok(None),
        }
    }

    // -- Feed cursor methods (Sprint 61 Phase C) --

    pub fn load_feed_cursor(&self) -> Result<Option<(u64, String)>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT last_seq, last_entry_hash FROM feed_cursor WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => {
                let seq: u64 = row.get::<_, i64>(0)? as u64;
                let hash: String = row.get(1)?;
                Ok(Some((seq, hash)))
            }
            None => Ok(None),
        }
    }

    pub fn save_feed_cursor(
        &self,
        last_seq: u64,
        last_entry_hash: &str,
    ) -> Result<(), CoordinatorError> {
        self.conn.execute(
            "INSERT INTO feed_cursor (id, last_seq, last_entry_hash)
             VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET last_seq = excluded.last_seq, last_entry_hash = excluded.last_entry_hash",
            rusqlite::params![last_seq as i64, last_entry_hash],
        )?;
        Ok(())
    }

    pub fn get_feed_entries_after_seq(
        &self,
        after_seq: u64,
    ) -> Result<Vec<FeedEntryRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT seq, op_type, payload, author, signature, entry_hash, prev_hash, created_at
             FROM public_feed WHERE seq > ?1 ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![after_seq as i64], |row| {
            Ok(FeedEntryRow {
                seq: row.get::<_, i64>(0)? as u64,
                op_type: row.get(1)?,
                payload: row.get(2)?,
                author: row.get(3)?,
                signature: row.get(4)?,
                entry_hash: row.get(5)?,
                prev_hash: row.get(6)?,
                created_at: row.get::<_, i64>(7)? as u64,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    // -- Public feed methods (Sprint 61 Phase B) --

    pub fn insert_feed_entry(&self, row: &FeedEntryRow) -> Result<u64, CoordinatorError> {
        self.conn.execute(
            "INSERT INTO public_feed (op_type, payload, author, signature, entry_hash, prev_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                row.op_type,
                row.payload,
                row.author,
                row.signature,
                row.entry_hash,
                row.prev_hash,
                row.created_at as i64
            ],
        )?;
        Ok(self.conn.last_insert_rowid() as u64)
    }

    pub fn get_feed_entries(&self) -> Result<Vec<FeedEntryRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT seq, op_type, payload, author, signature, entry_hash, prev_hash, created_at
             FROM public_feed ORDER BY seq ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(FeedEntryRow {
                seq: row.get::<_, i64>(0)? as u64,
                op_type: row.get(1)?,
                payload: row.get(2)?,
                author: row.get(3)?,
                signature: row.get(4)?,
                entry_hash: row.get(5)?,
                prev_hash: row.get(6)?,
                created_at: row.get::<_, i64>(7)? as u64,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_last_feed_entry_hash(&self) -> Result<Option<String>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT entry_hash FROM public_feed ORDER BY seq DESC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn feed_entry_exists_by_hash(&self, entry_hash: &str) -> Result<bool, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM public_feed WHERE entry_hash = ?1 LIMIT 1")?;
        let mut rows = stmt.query(rusqlite::params![entry_hash])?;
        Ok(rows.next()?.is_some())
    }

    pub fn delete_feed_entry_if_tail(&self, entry_hash: &str) -> Result<bool, CoordinatorError> {
        let changes = self.conn.execute(
            "DELETE FROM public_feed WHERE entry_hash = ?1
             AND NOT EXISTS (SELECT 1 FROM public_feed WHERE prev_hash = ?1)",
            rusqlite::params![entry_hash],
        )?;
        Ok(changes > 0)
    }

    pub fn get_last_feed_entry_hash_by_author(
        &self,
        author: &str,
    ) -> Result<Option<String>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT entry_hash FROM public_feed WHERE author = ?1 ORDER BY seq DESC LIMIT 1",
        )?;
        let mut rows = stmt.query(rusqlite::params![author])?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get(0)?)),
            None => Ok(None),
        }
    }

    pub fn count_feed_entries(&self) -> Result<u64, CoordinatorError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM public_feed", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    pub fn get_feed_last_seq(&self) -> Result<Option<u64>, CoordinatorError> {
        let mut stmt = self.conn.prepare("SELECT MAX(seq) FROM public_feed")?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => {
                let val: Option<i64> = row.get(0)?;
                Ok(val.map(|v| v as u64))
            }
            None => Ok(None),
        }
    }

    pub fn count_feed_entries_by_author_since(
        &self,
        author: &str,
        since_epoch: u64,
    ) -> Result<u64, CoordinatorError> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM public_feed WHERE author = ?1 AND created_at >= ?2",
            rusqlite::params![author, since_epoch as i64],
            |row| row.get(0),
        )?;
        Ok(count as u64)
    }

    pub fn insert_key_rotation(
        &self,
        old_pubkey: &str,
        new_pubkey: &str,
        timestamp: u64,
        transition_days: u16,
        signature: &str,
        reason: &str,
    ) -> Result<(), CoordinatorError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO key_rotations (old_pubkey, new_pubkey, timestamp, transition_days, signature, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                old_pubkey,
                new_pubkey,
                timestamp as i64,
                transition_days as i64,
                signature,
                reason,
                now as i64,
            ],
        )?;
        Ok(())
    }

    pub fn load_key_rotations(&self) -> Result<Vec<KeyRotationRow>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT old_pubkey, new_pubkey, timestamp, transition_days, signature, reason
             FROM key_rotations ORDER BY id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(KeyRotationRow {
                old_pubkey: row.get(0)?,
                new_pubkey: row.get(1)?,
                timestamp: row.get::<_, i64>(2)? as u64,
                transition_days: row.get::<_, i64>(3)? as u16,
                signature: row.get(4)?,
                reason: row.get(5)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_feed_author_stats(&self) -> Result<Vec<(String, u64)>, CoordinatorError> {
        let mut stmt = self
            .conn
            .prepare("SELECT author, COUNT(*) FROM public_feed GROUP BY author ORDER BY author")?;
        let rows = stmt.query_map([], |row| {
            let pubkey: String = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((pubkey, count as u64))
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
}

pub struct KeyRotationRow {
    pub old_pubkey: String,
    pub new_pubkey: String,
    pub timestamp: u64,
    pub transition_days: u16,
    pub signature: String,
    pub reason: String,
}

pub struct FeedEntryRow {
    pub seq: u64,
    pub op_type: String,
    pub payload: String,
    pub author: String,
    pub signature: String,
    pub entry_hash: String,
    pub prev_hash: String,
    pub created_at: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_task_record(task_id: &str) -> TaskRecord {
        TaskRecord {
            task_id: task_id.to_string(),
            status: TaskStatus::Pending,
            project_id: "proj-1".to_string(),
            model: "llama3".to_string(),
            created_at: 1714300000,
            updated_at: 1714300000,
            task_hash: "abc123".to_string(),
            worker_node_id: None,
            result_hash: None,
            task_type: "inference".to_string(),
            redundancy_factor: 1,
        }
    }

    #[test]
    fn open_in_memory_and_check_schema_version() {
        let db = CoordinatorDb::open_in_memory().expect("open in-memory");
        assert_eq!(db.schema_version().expect("version"), 1);
    }

    #[test]
    fn insert_and_get_task() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let record = make_task_record("task-001");
        db.insert_task(&record).expect("insert");

        let fetched = db.get_task("task-001").expect("get").expect("found");
        assert_eq!(fetched.task_id, "task-001");
        assert_eq!(fetched.status, TaskStatus::Pending);
        assert_eq!(fetched.model, "llama3");
    }

    #[test]
    fn update_task_status() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_task(&make_task_record("task-002"))
            .expect("insert");

        let updated = db
            .update_task_status("task-002", TaskStatus::Dispatched, 1714300100)
            .expect("update");
        assert!(updated);

        let fetched = db.get_task("task-002").expect("get").expect("found");
        assert_eq!(fetched.status, TaskStatus::Dispatched);
        assert_eq!(fetched.updated_at, 1714300100);
    }

    #[test]
    fn set_task_result_transitions_to_completed() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_task(&make_task_record("task-003"))
            .expect("insert");

        let ok = db
            .set_task_result(
                "task-003",
                "worker-a",
                "result-hash-x",
                "the model output",
                1714300200,
            )
            .expect("set result");
        assert!(ok);

        let fetched = db.get_task("task-003").expect("get").expect("found");
        assert_eq!(fetched.status, TaskStatus::Completed);
        assert_eq!(fetched.worker_node_id.as_deref(), Some("worker-a"));
        assert_eq!(fetched.result_hash.as_deref(), Some("result-hash-x"));
    }

    // Sprint 72 Phase D: the accepted text is persisted and retrievable
    // via `get_task_result` — the primitive the Operator network arm
    // reads to render a completed task's reply in the chat.
    #[test]
    fn set_task_result_persists_retrievable_text() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_task(&make_task_record("task-text-1"))
            .expect("insert");

        // Pending task: no result text yet (route would 404).
        let pending = db
            .get_task_result("task-text-1")
            .expect("get")
            .expect("row");
        assert_eq!(pending.status, "pending");
        assert!(pending.result_text.is_none());

        db.set_task_result(
            "task-text-1",
            "worker-a",
            "sig-hex",
            "hello from the network",
            1714300200,
        )
        .expect("set result");

        let done = db
            .get_task_result("task-text-1")
            .expect("get")
            .expect("row");
        assert_eq!(done.status, "completed");
        assert_eq!(done.result_text.as_deref(), Some("hello from the network"));
        assert_eq!(done.result_hash.as_deref(), Some("sig-hex"));
    }

    #[test]
    fn get_task_result_none_for_missing_task() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        assert!(db.get_task_result("nope").expect("get").is_none());
    }

    #[test]
    fn set_task_result_rejects_already_completed() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_task(&make_task_record("task-004"))
            .expect("insert");
        db.set_task_result("task-004", "w1", "r1", "first text", 100)
            .expect("first");

        let second = db
            .set_task_result("task-004", "w2", "r2", "second text", 200)
            .expect("second");
        assert!(!second, "already-completed task must reject second result");
    }

    #[test]
    fn get_nonexistent_task_returns_none() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        assert!(db.get_task("nope").expect("get").is_none());
    }

    #[test]
    fn insert_and_sum_kudos() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let entry = KudosEntry {
            entry_id: "k1".into(),
            worker_node_id: "worker-a".into(),
            task_id: "task-001".into(),
            project_id: "proj-1".into(),
            amount: 100,
            created_at: 1714300000,
            prev_hash: "genesis".into(),
            entry_hash: "hash-k1".into(),
        };
        db.insert_kudos(&entry).expect("insert k1");

        let entry2 = KudosEntry {
            entry_id: "k2".into(),
            worker_node_id: "worker-a".into(),
            task_id: "task-002".into(),
            project_id: "proj-1".into(),
            amount: 50,
            created_at: 1714300100,
            prev_hash: "hash-k1".into(),
            entry_hash: "hash-k2".into(),
        };
        db.insert_kudos(&entry2).expect("insert k2");

        assert_eq!(db.get_worker_kudos_total("worker-a").expect("total"), 150);
        assert_eq!(db.get_worker_kudos_total("nobody").expect("total"), 0);
    }

    #[test]
    fn open_file_creates_db_and_returns_schema_v1() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");
        let db = CoordinatorDb::open(&path).expect("open file");
        assert!(path.exists());
        assert_eq!(db.schema_version().expect("version"), 1);
    }

    #[test]
    fn open_file_activates_wal_mode() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");
        let db = CoordinatorDb::open(&path).expect("open file");
        let mode: String = db
            .conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("pragma");
        assert_eq!(mode.to_lowercase(), "wal");
    }

    #[test]
    fn insert_and_load_outbox() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_outbox(b"envelope-1").expect("insert 1");
        db.insert_outbox(b"envelope-2").expect("insert 2");
        db.insert_outbox(b"envelope-3").expect("insert 3");

        let loaded = db.load_outbox().expect("load");
        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0], b"envelope-1");
        assert_eq!(loaded[1], b"envelope-2");
        assert_eq!(loaded[2], b"envelope-3");
    }

    #[test]
    fn clear_outbox() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_outbox(b"envelope-a").expect("insert");
        assert_eq!(db.load_outbox().expect("load").len(), 1);

        db.clear_outbox().expect("clear");
        assert!(db.load_outbox().expect("load").is_empty());
    }

    #[test]
    fn migration_m18_creates_keep_online_table() {
        // M18 must create the keep_online table (a successful insert proves it).
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.set_keep_online(
            "a".repeat(64).as_str(),
            true,
            Some("bb".repeat(32).as_str()),
        )
        .expect("M18 keep_online table must exist");
    }

    #[test]
    fn keep_online_toggle_persists_m18() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let pid = "a".repeat(64);
        let hash = "bb".repeat(32);

        // Absent row = enabled-by-default (None), never in the disabled list.
        assert_eq!(db.get_keep_online(&pid).unwrap(), None);
        assert!(db.list_keep_online_disabled().unwrap().is_empty());

        // ON.
        db.set_keep_online(&pid, true, Some(&hash)).unwrap();
        assert_eq!(
            db.get_keep_online(&pid).unwrap(),
            Some((true, Some(hash.clone())))
        );
        assert!(db.list_keep_online_disabled().unwrap().is_empty());

        // OFF (single row per project — INSERT OR REPLACE).
        db.set_keep_online(&pid, false, Some(&hash)).unwrap();
        assert_eq!(
            db.get_keep_online(&pid).unwrap(),
            Some((false, Some(hash.clone())))
        );
        assert_eq!(db.list_keep_online_disabled().unwrap(), vec![pid.clone()]);

        // Back ON.
        db.set_keep_online(&pid, true, Some(&hash)).unwrap();
        assert!(db.list_keep_online_disabled().unwrap().is_empty());
    }

    #[test]
    fn migration_m19_creates_seed_invite_table() {
        // M19 must create the seed_invite table (a successful mint proves it).
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.mint_seed_invite(
            "tok-1",
            &"a".repeat(64),
            &"b".repeat(64),
            9_999_999_999,
            Some(1),
        )
        .expect("M19 seed_invite table must exist");
    }

    #[test]
    fn seed_invite_lifecycle_mint_consume_revoke() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let pid = "a".repeat(64);
        let ah = "b".repeat(64);
        let now = 1_700_000_000_i64;

        // Unknown token.
        assert_eq!(
            db.consume_seed_invite("nope", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::NotFound
        );

        // Reusable token (max_uses=None) consumes repeatedly while valid.
        db.mint_seed_invite("reuse", &pid, &ah, now + 1000, None)
            .unwrap();
        assert_eq!(
            db.consume_seed_invite("reuse", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::Ok
        );
        assert_eq!(
            db.consume_seed_invite("reuse", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::Ok
        );

        // Wrong project_id => NotFound (the token does not authorize THIS app).
        assert_eq!(
            db.consume_seed_invite("reuse", &"c".repeat(64), &ah, now)
                .unwrap(),
            SeedInviteOutcome::NotFound
        );

        // Wrong archive_hash => NotFound (capability is over the (project,
        // content) PAIR — an invited peer cannot swap in foreign content,
        // review P2).
        assert_eq!(
            db.consume_seed_invite("reuse", &pid, &"d".repeat(64), now)
                .unwrap(),
            SeedInviteOutcome::NotFound
        );

        // Expired.
        assert_eq!(
            db.consume_seed_invite("reuse", &pid, &ah, now + 2000)
                .unwrap(),
            SeedInviteOutcome::Expired
        );

        // Revocation.
        assert!(db.revoke_seed_invite("reuse").unwrap());
        assert_eq!(
            db.consume_seed_invite("reuse", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::Revoked
        );
        // Double-revoke is a no-op (already revoked).
        assert!(!db.revoke_seed_invite("reuse").unwrap());
    }

    #[test]
    fn seed_invite_single_use_exhausts() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let pid = "a".repeat(64);
        let ah = "b".repeat(64);
        let now = 1_700_000_000_i64;
        db.mint_seed_invite("single", &pid, &ah, now + 1000, Some(1))
            .unwrap();
        assert_eq!(
            db.consume_seed_invite("single", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::Ok
        );
        assert_eq!(
            db.consume_seed_invite("single", &pid, &ah, now).unwrap(),
            SeedInviteOutcome::NoUsesLeft
        );

        // list surfaces it for the UI.
        let rows = db.list_seed_invites(&pid).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].token, "single");
        assert_eq!(rows[0].archive_hash, ah);
        assert_eq!(rows[0].uses_count, 1);
    }

    #[test]
    fn outbox_survives_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");

        {
            let db = CoordinatorDb::open(&path).expect("open");
            db.insert_outbox(b"persistent-envelope").expect("insert");
        }

        let db2 = CoordinatorDb::open(&path).expect("reopen");
        let loaded = db2.load_outbox().expect("load");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0], b"persistent-envelope");
    }

    #[test]
    fn shared_db_dispatcher_persists_across_calls() {
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            CoordinatorDb::open_in_memory().expect("open"),
        ));
        let kp = nexus_core_rs::crypto::KeyPair::generate();

        let sub = crate::types::TaskSubmission {
            project_id: "proj".into(),
            task_type: "inference".into(),
            prompt: "test".into(),
            system_prompt: String::new(),
            model: "llama3".into(),
            priority: 5,
            parent_task_id: String::new(),
            metadata: std::collections::BTreeMap::new(),
            is_open_source: false,
            estimated_watts: 0,
            estimated_vram_mb: 0,
            estimated_hours: 0.0,
            redundancy_factor: 1,
            verifiable: false,
        };

        let task_id = {
            let guard = db.lock().unwrap();
            let entry = crate::dispatcher::submit_task(&guard, &kp, sub).expect("submit");
            entry.task.task_id
        };

        let guard = db.lock().unwrap();
        let record = guard.get_task(&task_id).expect("get").expect("found");
        assert_eq!(record.status, crate::types::TaskStatus::Pending);
    }

    #[test]
    fn storage_persistence_survives_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");

        {
            let db = CoordinatorDb::open(&path).expect("open");
            db.upsert_storage("myapp", "theme", &serde_json::json!("dark"))
                .expect("upsert");
        }

        let db2 = CoordinatorDb::open(&path).expect("reopen");
        let loaded = db2.load_all_storage().expect("load");
        let val = loaded
            .get("myapp")
            .and_then(|m| m.get("theme"))
            .expect("key present");
        assert_eq!(val, &serde_json::json!("dark"));
    }

    #[test]
    fn upsert_storage_overwrite() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.upsert_storage("app1", "counter", &serde_json::json!(1))
            .expect("first");
        db.upsert_storage("app1", "counter", &serde_json::json!(42))
            .expect("second");

        let loaded = db.load_all_storage().expect("load");
        let val = loaded
            .get("app1")
            .and_then(|m| m.get("counter"))
            .expect("key present");
        assert_eq!(val, &serde_json::json!(42));
    }

    #[test]
    fn delete_storage_nonexistent() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.delete_storage("noapp", "nokey").expect("no error");
    }

    #[test]
    fn load_all_storage_multiple_apps() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.upsert_storage("app-a", "k1", &serde_json::json!("v1"))
            .expect("upsert");
        db.upsert_storage("app-b", "k2", &serde_json::json!(99))
            .expect("upsert");
        db.upsert_storage("app-a", "k3", &serde_json::json!(true))
            .expect("upsert");

        let loaded = db.load_all_storage().expect("load");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded["app-a"].len(), 2);
        assert_eq!(loaded["app-b"].len(), 1);
        assert_eq!(loaded["app-a"]["k1"], serde_json::json!("v1"));
        assert_eq!(loaded["app-b"]["k2"], serde_json::json!(99));
    }

    #[test]
    fn storage_namespace_crud() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let ns_id = [42u8; 32];

        assert!(db.get_storage_namespace("sbfb-ideas").unwrap().is_none());

        db.set_storage_namespace("sbfb-ideas", &ns_id, Some("ticket-abc"))
            .expect("set");
        let row = db
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("must exist");
        assert_eq!(row.namespace_id, ns_id.to_vec());
        assert_eq!(row.doc_ticket.as_deref(), Some("ticket-abc"));

        db.set_storage_namespace("sbfb-ideas", &ns_id, Some("ticket-xyz"))
            .expect("upsert");
        let row2 = db
            .get_storage_namespace("sbfb-ideas")
            .unwrap()
            .expect("must exist");
        assert_eq!(row2.doc_ticket.as_deref(), Some("ticket-xyz"));
    }

    #[test]
    fn provenance_insert_and_retrieve() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let record = crate::provenance::generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        db.insert_provenance_record("proj-test", &record)
            .expect("insert");
        let fetched = db
            .get_provenance_by_project("proj-test")
            .expect("get")
            .expect("found");
        assert_eq!(fetched.repo_url, record.repo_url);
        assert_eq!(fetched.commit_sha, record.commit_sha);
        assert_eq!(fetched.artifact_hash, record.artifact_hash);
        assert_eq!(fetched.node_id, record.node_id);
        assert_eq!(fetched.signature, record.signature);
        assert_eq!(fetched.timestamp, record.timestamp);
        assert_eq!(fetched.schema_version, record.schema_version);

        assert!(
            db.get_provenance_by_project("nonexistent")
                .expect("get")
                .is_none()
        );
    }

    #[test]
    fn provenance_insert_with_version() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let mut record = crate::provenance::generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        record.app_version = Some("2.1.0".to_string());
        db.insert_provenance_record("proj-versioned", &record)
            .expect("insert");
        let fetched = db
            .get_provenance_by_project("proj-versioned")
            .expect("get")
            .expect("found");
        assert_eq!(fetched.app_version, Some("2.1.0".to_string()));
    }

    #[test]
    fn provenance_insert_without_version_backward_safe() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let record = crate::provenance::generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        db.insert_provenance_record("proj-no-version", &record)
            .expect("insert");
        let fetched = db
            .get_provenance_by_project("proj-no-version")
            .expect("get")
            .expect("found");
        assert_eq!(fetched.app_version, None);
    }

    #[test]
    fn migration_m14_creates_key_rotations_table() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");
        let db = CoordinatorDb::open(&path).expect("open");
        let count: i64 = db
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='key_rotations'",
                [],
                |row| row.get(0),
            )
            .expect("query");
        assert_eq!(count, 1, "key_rotations table must exist after M14");
    }

    #[test]
    fn key_rotation_insert_and_load() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_key_rotation(
            "aabbcc",
            "ddeeff",
            1_700_000_000,
            7,
            "sig_hex",
            "test rotation",
        )
        .expect("insert");
        let rows = db.load_key_rotations().expect("load");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].old_pubkey, "aabbcc");
        assert_eq!(rows[0].new_pubkey, "ddeeff");
        assert_eq!(rows[0].timestamp, 1_700_000_000);
        assert_eq!(rows[0].transition_days, 7);
        assert_eq!(rows[0].signature, "sig_hex");
        assert_eq!(rows[0].reason, "test rotation");
    }

    #[test]
    fn key_rotation_survives_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");
        {
            let db = CoordinatorDb::open(&path).expect("open");
            db.insert_key_rotation("aa", "bb", 100, 14, "sig", "reason")
                .expect("insert");
        }
        let db2 = CoordinatorDb::open(&path).expect("reopen");
        let rows = db2.load_key_rotations().expect("load");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].old_pubkey, "aa");
    }

    #[test]
    fn coordinator_db_synchronous_full() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("coordinator.db");
        let db = CoordinatorDb::open(&path).expect("open");
        let sync_val: i64 = db
            .conn
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .expect("pragma query");
        assert_eq!(sync_val, 2, "synchronous must be FULL (2) in WAL mode");
    }
}
