// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-worker project allowlist — the local SQLite database
//! that records which projects this machine has opted into and
//! how much compute it has burned on each one.
//!
//! The allowlist lives at
//! `<data_dir>/allowlist.sqlite3` (resolved by [`crate::config::WorkerPaths::default_allowlist_db`])
//! and is only read/written from the worker process. The schema
//! is versioned via [`rusqlite_migration`] so future waves can
//! add columns without losing enrolled projects.
//!
//! ## Consent model
//!
//! The SBFB plan (Sprint 3 "Modèle de projet" section) requires
//! that **every** contributor explicitly enrols in a project
//! before its tasks are claimed. This crate is the only place
//! where enrolment is recorded; the W9 engine loop refuses to
//! claim a task whose project is not `enabled = 1` in this
//! database. No code path anywhere else in the worker mutates
//! enrolment state.
//!
//! ## Threading
//!
//! The connection is wrapped in a `std::sync::Mutex` because
//! `rusqlite::Connection` is `!Sync`. Queries are cheap and
//! rarely contended (the engine polls at `task_poll_interval_ms`
//! which defaults to 2s), so the mutex never shows up in
//! profiles. Async callers should dispatch via
//! `tokio::task::spawn_blocking` when they need to keep the
//! runtime responsive during large batch updates.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use rusqlite::{params, Connection, OptionalExtension, Row};
use rusqlite_migration::{Migrations, M};
use serde::{Deserialize, Serialize};
use thiserror::Error;

// =================================================================
// Errors
// =================================================================

/// Failures the allowlist layer can produce.
#[derive(Debug, Error)]
pub enum AllowlistError {
    /// Underlying SQLite / rusqlite error (busy, bad schema,
    /// constraint violation, ...).
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// rusqlite_migration could not apply the schema migrations.
    #[error("schema migration failed: {0}")]
    Migration(#[from] rusqlite_migration::Error),

    /// Failed to create the parent directory of the database
    /// file, or some other filesystem error encountered while
    /// opening the database.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Attempted to enroll a project that is already in the
    /// allowlist. Use `enable` / `set_budget` instead.
    #[error("project already enrolled: {0}")]
    AlreadyEnrolled(String),

    /// Requested operation on an unknown project id.
    #[error("project not found: {0}")]
    NotFound(String),

    /// The connection mutex was poisoned by a panic on another
    /// thread — this is a hard error because the state is now
    /// untrustworthy and the engine must stop.
    #[error("allowlist connection mutex poisoned")]
    Poisoned,
}

/// Convenient `Result` alias.
pub type AllowlistResult<T> = std::result::Result<T, AllowlistError>;

// =================================================================
// Migrations — versioned so future waves can add columns safely
// =================================================================

/// Embedded schema migrations, lazy-initialized on first use.
///
/// Every migration is an `M::up` statement. Adding a new
/// migration at the end of the vec is backwards compatible:
/// on existing worker installations the first open after
/// upgrade detects the `user_version` gap and runs only the
/// new statements atomically. Never edit or delete an existing
/// migration entry once it has shipped — that would make
/// already-upgraded workers irrecoverable.
fn migrations() -> &'static Migrations<'static> {
    static MIGRATIONS: OnceLock<Migrations<'static>> = OnceLock::new();
    MIGRATIONS.get_or_init(|| {
        Migrations::new(vec![
            // ---- v1 (Sprint 3 W7) ----
            M::up(
                r#"
                CREATE TABLE projects (
                    id              TEXT PRIMARY KEY NOT NULL,
                    name            TEXT NOT NULL,
                    enabled         INTEGER NOT NULL DEFAULT 1,
                    budget_joules   INTEGER NOT NULL DEFAULT 0,
                    joined_at       TEXT NOT NULL,
                    tasks_completed INTEGER NOT NULL DEFAULT 0,
                    joules_used     INTEGER NOT NULL DEFAULT 0
                );
                CREATE INDEX projects_enabled_idx ON projects (enabled);
                "#,
            ),
            // ---- v2 (Sprint 4 Phase C) ----
            //
            // Add tasks_doc_ticket TEXT column. It carries the
            // serialized iroh-docs write ticket that invite v2
            // embedded at join time, so the W9.1 runtime drop-in
            // (Phase D) can import the project doc without a
            // separate out-of-band exchange. Existing v1 rows get
            // NULL, which the engine interprets as "legacy v1
            // enrollment, cannot claim tasks until a v2 invite
            // rejoins the project" (Phase D drop-in).
            M::up(
                r#"
                ALTER TABLE projects ADD COLUMN tasks_doc_ticket TEXT;
                "#,
            ),
        ])
    })
}

// =================================================================
// Public types
// =================================================================

/// Parameters required to enroll a new project.
///
/// Kept deliberately small: W8 (invite tokens) is responsible
/// for validating the invite and producing one of these; every
/// other caller either goes through W8 or is a test fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewProject {
    /// Iroh document namespace id that uniquely identifies the
    /// project across the SBFB network. Stored as an opaque
    /// string; the allowlist does not parse it.
    pub id: String,
    /// Human-readable project name extracted from the invite or
    /// the manifest. Shown in the `projects list` CLI output
    /// and in the W10 TUI dashboard.
    pub name: String,
    /// Whether the project should start in the enabled state.
    /// The `join` subcommand passes `true`; administrative
    /// scripts that pre-seed a fixture may pass `false`.
    pub enabled: bool,
    /// Daily energy budget in joules. `0` = unlimited (the
    /// engine will serve the project until some other limit is
    /// hit).
    pub budget_joules: u64,
    /// Serialized iroh-docs write ticket extracted from the
    /// invite v2 payload. `None` for observer-scope enrolments
    /// or for legacy records — the Phase D runtime drop-in
    /// skips any project whose ticket is `None` when claiming
    /// tasks.
    pub tasks_doc_ticket: Option<String>,
}

/// A single row from the `projects` table, in the shape the
/// W7/W9/W10 callers consume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub budget_joules: u64,
    pub joined_at: String,
    pub tasks_completed: u64,
    pub joules_used: u64,
    pub tasks_doc_ticket: Option<String>,
}

impl Project {
    /// Remaining energy budget for today, in joules.
    ///
    /// `None` = unlimited (`budget_joules == 0`). `Some(0)` =
    /// budget exhausted (engine must stop claiming tasks for
    /// this project until the daily reset in W9.1).
    pub fn budget_remaining_joules(&self) -> Option<u64> {
        if self.budget_joules == 0 {
            None
        } else {
            Some(self.budget_joules.saturating_sub(self.joules_used))
        }
    }

    /// True iff the project should be considered claim-able by
    /// the engine: enabled AND either unlimited budget or budget
    /// not yet exhausted.
    pub fn is_serveable(&self) -> bool {
        if !self.enabled {
            return false;
        }
        match self.budget_remaining_joules() {
            None => true,
            Some(j) => j > 0,
        }
    }
}

fn project_from_row(row: &Row<'_>) -> rusqlite::Result<Project> {
    Ok(Project {
        id: row.get(0)?,
        name: row.get(1)?,
        enabled: row.get::<_, i64>(2)? != 0,
        budget_joules: row.get::<_, i64>(3)? as u64,
        joined_at: row.get(4)?,
        tasks_completed: row.get::<_, i64>(5)? as u64,
        joules_used: row.get::<_, i64>(6)? as u64,
        tasks_doc_ticket: row.get(7)?,
    })
}

// =================================================================
// Allowlist handle
// =================================================================

/// Thread-safe handle to the per-worker allowlist database.
///
/// Construct via [`Allowlist::open`] (on-disk) or
/// [`Allowlist::open_in_memory`] (tests / ephemeral fixtures).
/// Both apply all pending migrations before returning, so the
/// caller can immediately start using the full schema.
pub struct Allowlist {
    conn: Mutex<Connection>,
    path: Option<PathBuf>,
}

impl std::fmt::Debug for Allowlist {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Allowlist")
            .field("path", &self.path)
            .finish()
    }
}

impl Allowlist {
    /// Open or create the SQLite database at `path`, applying
    /// every pending schema migration.
    ///
    /// Missing parent directories are created via
    /// `std::fs::create_dir_all` before the file is opened, so
    /// callers never need to `ensure_dirs` themselves.
    pub fn open(path: impl AsRef<Path>) -> AllowlistResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let mut conn = Connection::open(path)?;
        Self::prepare(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: Some(path.to_path_buf()),
        })
    }

    /// Open an ephemeral in-memory database for tests. The
    /// connection is tied to the returned handle and destroyed
    /// on drop.
    pub fn open_in_memory() -> AllowlistResult<Self> {
        let mut conn = Connection::open_in_memory()?;
        Self::prepare(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
            path: None,
        })
    }

    fn prepare(conn: &mut Connection) -> AllowlistResult<()> {
        // Sensible pragmas: WAL journaling survives crashes and
        // concurrent readers, foreign_keys just in case a
        // future migration adds them, busy_timeout absorbs
        // short-lived contention without surfacing errors.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(std::time::Duration::from_millis(500))?;

        migrations().to_latest(conn)?;
        Ok(())
    }

    /// Path to the underlying SQLite file, or `None` when the
    /// allowlist is in-memory.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    fn with_conn<T, F>(&self, f: F) -> AllowlistResult<T>
    where
        F: FnOnce(&Connection) -> AllowlistResult<T>,
    {
        let guard = self.conn.lock().map_err(|_| AllowlistError::Poisoned)?;
        f(&guard)
    }

    // -----------------------------------------------------------
    // Mutations
    // -----------------------------------------------------------

    /// Enroll a fresh project. Fails with
    /// [`AllowlistError::AlreadyEnrolled`] if the id already
    /// exists; the caller is expected to handle that case
    /// explicitly (e.g. "project already joined, use
    /// `projects enable` instead").
    pub fn enroll(&self, project: NewProject) -> AllowlistResult<()> {
        let joined_at = current_iso8601();
        self.with_conn(|conn| {
            let affected = conn
                .execute(
                    "INSERT OR IGNORE INTO projects \
                       (id, name, enabled, budget_joules, joined_at, tasks_doc_ticket) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        project.id,
                        project.name,
                        project.enabled as i64,
                        project.budget_joules as i64,
                        joined_at,
                        project.tasks_doc_ticket,
                    ],
                )
                .map_err(AllowlistError::from)?;
            if affected == 0 {
                return Err(AllowlistError::AlreadyEnrolled(project.id.clone()));
            }
            Ok(())
        })
    }

    /// Mark a project as enabled (the engine will claim its
    /// tasks). No-op if the project is already enabled. Errors
    /// with `NotFound` if the id is unknown.
    pub fn enable(&self, id: &str) -> AllowlistResult<()> {
        self.set_enabled(id, true)
    }

    /// Mark a project as disabled (no new claims; in-flight
    /// tasks drain). Errors with `NotFound` if the id is
    /// unknown.
    pub fn disable(&self, id: &str) -> AllowlistResult<()> {
        self.set_enabled(id, false)
    }

    fn set_enabled(&self, id: &str, enabled: bool) -> AllowlistResult<()> {
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE projects SET enabled = ?1 WHERE id = ?2",
                params![enabled as i64, id],
            )?;
            if n == 0 {
                return Err(AllowlistError::NotFound(id.to_string()));
            }
            Ok(())
        })
    }

    /// Set the daily energy budget in joules. `0` means
    /// unlimited.
    pub fn set_budget(&self, id: &str, joules: u64) -> AllowlistResult<()> {
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE projects SET budget_joules = ?1 WHERE id = ?2",
                params![joules as i64, id],
            )?;
            if n == 0 {
                return Err(AllowlistError::NotFound(id.to_string()));
            }
            Ok(())
        })
    }

    /// Record a completed task: increments `tasks_completed`
    /// and adds `joules_used` to the running counter.
    ///
    /// This is the *only* place the engine should update usage
    /// counters — see the Sprint 3 W9 plan.
    pub fn record_task(&self, id: &str, joules_used: u64) -> AllowlistResult<()> {
        self.with_conn(|conn| {
            let n = conn.execute(
                "UPDATE projects \
                 SET tasks_completed = tasks_completed + 1, \
                     joules_used     = joules_used + ?1 \
                 WHERE id = ?2",
                params![joules_used as i64, id],
            )?;
            if n == 0 {
                return Err(AllowlistError::NotFound(id.to_string()));
            }
            Ok(())
        })
    }

    /// Remove a project entirely. Used by the CLI for "I don't
    /// want this project anymore" cleanups, distinct from
    /// `disable` which keeps the stats.
    pub fn remove(&self, id: &str) -> AllowlistResult<()> {
        self.with_conn(|conn| {
            let n = conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
            if n == 0 {
                return Err(AllowlistError::NotFound(id.to_string()));
            }
            Ok(())
        })
    }

    // -----------------------------------------------------------
    // Queries
    // -----------------------------------------------------------

    /// Look up a single project by id. Returns `None` if the id
    /// is unknown — NOT an error.
    pub fn get(&self, id: &str) -> AllowlistResult<Option<Project>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, enabled, budget_joules, joined_at, \
                        tasks_completed, joules_used, tasks_doc_ticket \
                 FROM projects WHERE id = ?1",
            )?;
            let result = stmt.query_row(params![id], project_from_row).optional()?;
            Ok(result)
        })
    }

    /// Return every enrolled project, enabled or not, ordered
    /// by name for stable CLI output.
    pub fn list(&self) -> AllowlistResult<Vec<Project>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, enabled, budget_joules, joined_at, \
                        tasks_completed, joules_used, tasks_doc_ticket \
                 FROM projects ORDER BY name COLLATE NOCASE",
            )?;
            let rows = stmt
                .query_map([], project_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
    }

    /// Return only enabled projects — the fast path the engine
    /// uses on its poll tick. Uses the `projects_enabled_idx`
    /// index so the query is constant-time in the number of
    /// disabled projects.
    pub fn list_enabled(&self) -> AllowlistResult<Vec<Project>> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, name, enabled, budget_joules, joined_at, \
                        tasks_completed, joules_used, tasks_doc_ticket \
                 FROM projects WHERE enabled = 1 ORDER BY name COLLATE NOCASE",
            )?;
            let rows = stmt
                .query_map([], project_from_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok(rows)
        })
    }

    /// Return the current schema `user_version`. Primarily
    /// useful in tests and for diagnostics in the `stats`
    /// subcommand.
    pub fn schema_version(&self) -> AllowlistResult<usize> {
        self.with_conn(|conn| {
            let v: u32 = conn.query_row("PRAGMA user_version;", [], |r| r.get(0))?;
            Ok(v as usize)
        })
    }
}

// =================================================================
// Helpers
// =================================================================

/// Return the current UTC time formatted as an ISO-8601 string
/// (to-the-second precision).
fn current_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_iso8601(secs)
}

/// Pure-function ISO-8601 formatter so tests can feed a fixed
/// timestamp and produce a deterministic string.
fn format_unix_iso8601(unix_secs: u64) -> String {
    // Minimal YYYY-MM-DDTHH:MM:SSZ formatter — avoids pulling
    // `chrono` or `time` just for one line.
    let days = unix_secs / 86_400;
    let secs_of_day = unix_secs % 86_400;
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;

    // Civil-date calculation using Howard Hinnant's algorithm
    // (public domain). Converts days since 1970-01-01 to
    // (year, month, day).
    let days_i = days as i64 + 719_468;
    let era = days_i.div_euclid(146_097);
    let doe = days_i.rem_euclid(146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!("{year:04}-{m:02}-{d:02}T{hour:02}:{minute:02}:{second:02}Z")
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn sample_project(id: &str) -> NewProject {
        NewProject {
            id: id.to_string(),
            name: format!("Project {id}"),
            enabled: true,
            budget_joules: 0,
            tasks_doc_ticket: None,
        }
    }

    #[test]
    fn open_in_memory_applies_migrations() {
        let a = Allowlist::open_in_memory().unwrap();
        // Sprint 4 Phase C bumps the schema to v2 (adds
        // projects.tasks_doc_ticket).
        assert_eq!(a.schema_version().unwrap(), 2);
        assert!(a.list().unwrap().is_empty());
    }

    #[test]
    fn open_on_disk_creates_parent_and_file() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("nested").join("allowlist.sqlite3");
        let a = Allowlist::open(&db_path).unwrap();
        assert_eq!(a.path(), Some(db_path.as_path()));
        assert!(db_path.exists());
        assert_eq!(a.schema_version().unwrap(), 2);
    }

    #[test]
    fn enroll_persists_tasks_doc_ticket() {
        let a = Allowlist::open_in_memory().unwrap();
        let mut p = sample_project("proj-t");
        p.tasks_doc_ticket = Some("nx-doc-ticket-fake".to_string());
        a.enroll(p).unwrap();
        let fetched = a.get("proj-t").unwrap().unwrap();
        assert_eq!(
            fetched.tasks_doc_ticket.as_deref(),
            Some("nx-doc-ticket-fake")
        );
    }

    #[test]
    fn enroll_and_get_and_list_roundtrip() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        a.enroll(sample_project("proj-b")).unwrap();

        let fetched = a.get("proj-a").unwrap().unwrap();
        assert_eq!(fetched.id, "proj-a");
        assert_eq!(fetched.name, "Project proj-a");
        assert!(fetched.enabled);
        assert_eq!(fetched.budget_joules, 0);
        assert_eq!(fetched.tasks_completed, 0);
        assert_eq!(fetched.joules_used, 0);

        let all = a.list().unwrap();
        assert_eq!(all.len(), 2);
        // NOCASE alphabetical
        assert_eq!(all[0].id, "proj-a");
        assert_eq!(all[1].id, "proj-b");
    }

    #[test]
    fn enroll_same_id_twice_errors() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        let err = a.enroll(sample_project("proj-a")).unwrap_err();
        match err {
            AllowlistError::AlreadyEnrolled(id) => assert_eq!(id, "proj-a"),
            other => panic!("expected AlreadyEnrolled, got {other:?}"),
        }
    }

    #[test]
    fn enable_disable_toggles_flag() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        a.disable("proj-a").unwrap();
        assert!(!a.get("proj-a").unwrap().unwrap().enabled);
        a.enable("proj-a").unwrap();
        assert!(a.get("proj-a").unwrap().unwrap().enabled);
    }

    #[test]
    fn enable_unknown_id_errors() {
        let a = Allowlist::open_in_memory().unwrap();
        match a.enable("missing").unwrap_err() {
            AllowlistError::NotFound(id) => assert_eq!(id, "missing"),
            other => panic!("expected NotFound, got {other:?}"),
        }
    }

    #[test]
    fn set_budget_updates_value() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        a.set_budget("proj-a", 1_800_000).unwrap();
        let fetched = a.get("proj-a").unwrap().unwrap();
        assert_eq!(fetched.budget_joules, 1_800_000);
    }

    #[test]
    fn record_task_increments_counters() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        a.record_task("proj-a", 42).unwrap();
        a.record_task("proj-a", 58).unwrap();
        let fetched = a.get("proj-a").unwrap().unwrap();
        assert_eq!(fetched.tasks_completed, 2);
        assert_eq!(fetched.joules_used, 100);
    }

    #[test]
    fn remove_deletes_row() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        a.remove("proj-a").unwrap();
        assert!(a.get("proj-a").unwrap().is_none());
        assert!(a.list().unwrap().is_empty());
    }

    #[test]
    fn list_enabled_filters_disabled() {
        let a = Allowlist::open_in_memory().unwrap();
        a.enroll(sample_project("proj-a")).unwrap();
        let mut p_b = sample_project("proj-b");
        p_b.enabled = false;
        a.enroll(p_b).unwrap();
        a.enroll(sample_project("proj-c")).unwrap();

        let enabled = a.list_enabled().unwrap();
        let ids: Vec<_> = enabled.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["proj-a", "proj-c"]);
    }

    #[test]
    fn budget_remaining_is_none_for_unlimited() {
        let p = Project {
            id: "x".into(),
            name: "X".into(),
            enabled: true,
            budget_joules: 0,
            joined_at: "2026-04-10T00:00:00Z".into(),
            tasks_completed: 3,
            joules_used: 500,
            tasks_doc_ticket: None,
        };
        assert_eq!(p.budget_remaining_joules(), None);
        assert!(p.is_serveable());
    }

    #[test]
    fn budget_remaining_is_some_for_limited() {
        let mut p = Project {
            id: "x".into(),
            name: "X".into(),
            enabled: true,
            budget_joules: 1_000,
            joined_at: "2026-04-10T00:00:00Z".into(),
            tasks_completed: 1,
            joules_used: 400,
            tasks_doc_ticket: None,
        };
        assert_eq!(p.budget_remaining_joules(), Some(600));
        assert!(p.is_serveable());

        p.joules_used = 1_500;
        assert_eq!(p.budget_remaining_joules(), Some(0));
        assert!(!p.is_serveable());
    }

    #[test]
    fn disabled_project_is_never_serveable() {
        let p = Project {
            id: "x".into(),
            name: "X".into(),
            enabled: false,
            budget_joules: 0,
            joined_at: "2026-04-10T00:00:00Z".into(),
            tasks_completed: 0,
            joules_used: 0,
            tasks_doc_ticket: None,
        };
        assert!(!p.is_serveable());
    }

    #[test]
    fn iso8601_formatter_is_reasonable_for_known_epoch() {
        // unix epoch = 1970-01-01T00:00:00Z
        assert_eq!(format_unix_iso8601(0), "1970-01-01T00:00:00Z");
        // 2001-09-09T01:46:40Z = 1_000_000_000
        assert_eq!(format_unix_iso8601(1_000_000_000), "2001-09-09T01:46:40Z");
        // 2026-04-10T00:00:00Z — sanity check for the SBFB
        // pivot date. 1970-01-01 → 2026-04-10 is
        // 56 years (14 of which are leap: 1972..=2024 step 4
        // = 14) = 56*365 + 14 = 20454 days = 1_767_225_600s.
        let apr_10_2026: u64 = 1_775_779_200;
        let s = format_unix_iso8601(apr_10_2026);
        assert_eq!(s, "2026-04-10T00:00:00Z");
    }

    #[test]
    fn reopen_existing_database_keeps_data() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("allowlist.sqlite3");
        {
            let a = Allowlist::open(&db_path).unwrap();
            a.enroll(sample_project("proj-a")).unwrap();
            a.record_task("proj-a", 25).unwrap();
        }
        let a = Allowlist::open(&db_path).unwrap();
        let p = a.get("proj-a").unwrap().unwrap();
        assert_eq!(p.joules_used, 25);
        assert_eq!(p.tasks_completed, 1);
    }
}
