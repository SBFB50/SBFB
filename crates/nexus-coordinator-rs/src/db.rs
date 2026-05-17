// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator SQLite persistence layer.
//!
//! Owns `~/.sbfb/coordinator.db` with schema versioning to prevent
//! silent drift during the Python→Rust gradual migration (G1 D3 ⚠️).

use std::path::Path;

use rusqlite::Connection;
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
];

pub struct StorageNamespaceRow {
    pub namespace_id: Vec<u8>,
    pub doc_ticket: Option<String>,
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
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let migrations = Migrations::new(MIGRATIONS.to_vec());
        migrations.to_latest(&mut conn)?;

        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, CoordinatorError> {
        let mut conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

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

    pub fn set_task_result(
        &self,
        task_id: &str,
        worker_node_id: &str,
        result_hash: &str,
        updated_at: u64,
    ) -> Result<bool, CoordinatorError> {
        let changed = self.conn.execute(
            "UPDATE tasks SET status = 'completed', worker_node_id = ?1, result_hash = ?2, updated_at = ?3
             WHERE task_id = ?4 AND status IN ('pending', 'dispatched', 'awaiting_quorum')",
            rusqlite::params![worker_node_id, result_hash, updated_at, task_id],
        )?;
        Ok(changed > 0)
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
            .set_task_result("task-003", "worker-a", "result-hash-x", 1714300200)
            .expect("set result");
        assert!(ok);

        let fetched = db.get_task("task-003").expect("get").expect("found");
        assert_eq!(fetched.status, TaskStatus::Completed);
        assert_eq!(fetched.worker_node_id.as_deref(), Some("worker-a"));
        assert_eq!(fetched.result_hash.as_deref(), Some("result-hash-x"));
    }

    #[test]
    fn set_task_result_rejects_already_completed() {
        let db = CoordinatorDb::open_in_memory().expect("open");
        db.insert_task(&make_task_record("task-004"))
            .expect("insert");
        db.set_task_result("task-004", "w1", "r1", 100)
            .expect("first");

        let second = db
            .set_task_result("task-004", "w2", "r2", 200)
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
}
