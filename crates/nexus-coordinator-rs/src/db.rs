// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator SQLite persistence layer.
//!
//! Owns `~/.sbfb/coordinator.db` with schema versioning to prevent
//! silent drift during the Python→Rust gradual migration (G1 D3 ⚠️).

use std::path::Path;

use rusqlite::Connection;
use rusqlite_migration::{Migrations, M};

use crate::error::CoordinatorError;
use crate::types::{KudosEntry, TaskRecord, TaskStatus};

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
];

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
            "INSERT INTO tasks (task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
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
            ],
        )?;
        Ok(())
    }

    pub fn get_task(&self, task_id: &str) -> Result<Option<TaskRecord>, CoordinatorError> {
        let mut stmt = self.conn.prepare(
            "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash
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
                "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash
                 FROM tasks WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2",
                vec![Box::new(s.to_string()), Box::new(limit as i64)],
            ),
            None => (
                "SELECT task_id, status, project_id, model, created_at, updated_at, task_hash, worker_node_id, result_hash
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
             WHERE task_id = ?4 AND status IN ('pending', 'dispatched')",
            rusqlite::params![worker_node_id, result_hash, updated_at, task_id],
        )?;
        Ok(changed > 0)
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
    fn shared_db_dispatcher_persists_across_calls() {
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            CoordinatorDb::open_in_memory().expect("open"),
        ));
        let kp = nexus_core_rs::crypto::KeyPair::generate();

        let sub = crate::types::TaskSubmission {
            project_id: "proj".into(),
            task_type: "analysis".into(),
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
}
