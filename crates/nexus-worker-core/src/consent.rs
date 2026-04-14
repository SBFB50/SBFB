// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU sharing consent + per-day caps (Sprint 16 Phase C).
//!
//! Sister module to [`crate::allowlist`]. Where `allowlist` records
//! which projects the worker has explicitly enrolled into via the
//! invite-token flow, this module records the *broader* policy the
//! user picked at first boot in the Network UI: how much GPU time
//! they want to share with the public network (4 levels) and what
//! daily caps the worker must enforce (max watts, max VRAM, max
//! hours/day).
//!
//! ## On-disk layout
//!
//! Two JSON files live under the SBFB user dir (`~/.sbfb/` on
//! Unix, `%APPDATA%\sbfb\` on Windows):
//!
//! - `consent.json` — preferences. Written by the coordinator
//!   when the user clicks "Save" in `GpuConsentDialog`. Reloaded
//!   live by the worker via [`ConsentWatcher`] (uses the `notify`
//!   crate so changes apply without restart).
//! - `usage.json` — running counters for the day. Written by the
//!   worker every time it completes a task. Reset to zero the
//!   first time a new local-midnight boundary is crossed.
//!
//! Both writes are atomic (`tmp + rename`) so a crash mid-write
//! never leaves the worker reading garbage.
//!
//! ## Why a separate file from `allowlist`
//!
//! `allowlist` is "did the user enroll in *this specific project*
//! (yes/no)". Consent is "what overall sharing policy did the user
//! pick (L1..L4) plus what hard caps". The plan put both bullet
//! points in the same Phase, but the disk format and the lookup
//! patterns are different enough that mixing them in one file
//! would muddy both. The Phase C kickoff and audit treat them as
//! one delivery — see `Sprint 16 Phase C` commit body for the
//! deviation note.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, warn};

// =================================================================
// Errors
// =================================================================

#[derive(Debug, Error)]
pub enum ConsentError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("serde_json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("notify watcher error: {0}")]
    Notify(#[from] notify::Error),

    #[error("consent state lock poisoned")]
    Poisoned,

    #[error("invalid consent level {0} (expected 1..=4)")]
    InvalidLevel(u8),
}

pub type ConsentResult<T> = Result<T, ConsentError>;

// =================================================================
// Public types — consent.json
// =================================================================

/// User-picked GPU sharing level. Values are stable on the wire
/// (serialized as the integer 1..=4) so the React shell, the
/// FastAPI coordinator, and the Rust worker agree byte-for-byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(into = "u8", try_from = "u8")]
pub enum ConsentLevel {
    /// L1 — accept tasks only from projects whose `project_id`
    /// equals the worker's own `node_id`. Default at first boot
    /// (GDPR-safe: zero opt-in).
    OwnProjects,
    /// L2 — accept tasks from any project flagged
    /// `is_open_source = true` in its [`ProjectAnnouncement`].
    /// The flag is set by the coordinator at deploy-from-repo
    /// time (Phase D); for tasks where the flag is absent the
    /// worker treats it as `false` (reject).
    OpenSource,
    /// L3 — accept tasks only from projects whose `project_id`
    /// is in [`ConsentConfig::allowed_project_ids`]. The user
    /// edits the whitelist via the Network page or via the
    /// "Contribuer mon GPU" button on `BrowsedProject`.
    Whitelist,
    /// L4 — accept tasks from every public project, subject to
    /// the [`Caps`] budget below.
    All,
}

impl From<ConsentLevel> for u8 {
    fn from(value: ConsentLevel) -> Self {
        match value {
            ConsentLevel::OwnProjects => 1,
            ConsentLevel::OpenSource => 2,
            ConsentLevel::Whitelist => 3,
            ConsentLevel::All => 4,
        }
    }
}

impl TryFrom<u8> for ConsentLevel {
    type Error = ConsentError;
    fn try_from(value: u8) -> ConsentResult<Self> {
        match value {
            1 => Ok(Self::OwnProjects),
            2 => Ok(Self::OpenSource),
            3 => Ok(Self::Whitelist),
            4 => Ok(Self::All),
            other => Err(ConsentError::InvalidLevel(other)),
        }
    }
}

/// Hard caps the worker must enforce regardless of the consent
/// level. `None` on a field means "no cap". Default values come
/// from [`Caps::default`] which uses conservative MVP figures
/// (~400W, 16 GB VRAM, 12h/day) — well below the RTX 5080's
/// physical ceilings so the cap is intentional, not accidental.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Caps {
    /// Maximum sustained watts the worker is allowed to draw
    /// while running a task. A task whose `estimated_watts`
    /// exceeds this is rejected at claim time (see
    /// [`should_accept_task`]).
    pub max_watts: Option<u32>,
    /// Maximum VRAM in megabytes a single task may request.
    pub max_vram_mb: Option<u64>,
    /// Maximum cumulative wall-clock task duration per local
    /// day. Resets at local midnight via [`UsageTracker`].
    pub max_hours_day: Option<f64>,
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            max_watts: Some(400),
            max_vram_mb: Some(16 * 1024),
            max_hours_day: Some(12.0),
        }
    }
}

/// Full consent payload — what the coordinator writes to
/// `consent.json` after the user clicks "Save" in the dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsentConfig {
    pub level: ConsentLevel,
    pub caps: Caps,
    /// Hex-encoded `node_id`s the user explicitly added in L3.
    /// Stored as a `HashSet` so the L3 lookup in
    /// [`should_accept_task`] is O(1).
    #[serde(default)]
    pub allowed_project_ids: HashSet<String>,
    /// Worker's own `node_id`, captured here so [`should_accept_task`]
    /// can match L1 without an extra lookup. The coordinator
    /// fills this on every write so the file is self-contained.
    #[serde(default)]
    pub own_node_id: String,
}

impl ConsentConfig {
    /// Default for a fresh install — L1 (own projects only),
    /// conservative caps, empty whitelist. The `own_node_id`
    /// must be filled by the caller because this module has no
    /// access to the worker's keypair.
    pub fn default_for(own_node_id: impl Into<String>) -> Self {
        Self {
            level: ConsentLevel::OwnProjects,
            caps: Caps::default(),
            allowed_project_ids: HashSet::new(),
            own_node_id: own_node_id.into(),
        }
    }

    /// Read from disk. Returns the defaults if the file does not
    /// exist (first boot, before the dialog has been saved).
    pub fn load_or_default(path: &Path, own_node_id: impl Into<String>) -> ConsentResult<Self> {
        match fs::read(path) {
            Ok(bytes) => {
                let mut cfg: ConsentConfig = serde_json::from_slice(&bytes)?;
                if cfg.own_node_id.is_empty() {
                    cfg.own_node_id = own_node_id.into();
                }
                Ok(cfg)
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Self::default_for(own_node_id)),
            Err(e) => Err(e.into()),
        }
    }

    /// Write atomically (`tmp + rename`) so a crash mid-write
    /// cannot leave the worker reading a half-truncated file.
    pub fn save_atomic(&self, path: &Path) -> ConsentResult<()> {
        atomic_write_json(path, self)
    }
}

// =================================================================
// Public types — usage.json
// =================================================================

/// Per-day usage counter, persisted to `usage.json`. The
/// `today_local` field stores the local date for which
/// `hours_today` is meaningful; on the first call after a local
/// midnight rollover the tracker resets `hours_today` to 0 and
/// updates `today_local`.
///
/// Serializes the date as `YYYY-MM-DD` so a human reading the
/// file can sanity-check it without a chrono dependency on the
/// reading side.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UsageState {
    /// Local-time date for which [`Self::hours_today`] is
    /// counted. Stored as `YYYY-MM-DD` (ISO-8601 calendar).
    pub today_local: String,
    /// Cumulative wall-clock hours spent on tasks during
    /// `today_local`.
    pub hours_today: f64,
}

impl UsageState {
    fn for_today_local() -> Self {
        Self {
            today_local: today_local_iso(),
            hours_today: 0.0,
        }
    }
}

/// Stateful tracker around [`UsageState`] that handles the
/// midnight-local rollover and the atomic persistence.
///
/// Created once at worker boot, then mutated in-place by the
/// engine loop on every completed task. Keep one per process —
/// concurrent writers would race on the local-midnight reset.
#[derive(Debug)]
pub struct UsageTracker {
    path: PathBuf,
    state: UsageState,
}

impl UsageTracker {
    /// Read the existing file, or build a fresh one for today's
    /// local date if it does not exist.
    pub fn load_or_default(path: impl Into<PathBuf>) -> ConsentResult<Self> {
        let path = path.into();
        let state = match fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound => UsageState::for_today_local(),
            Err(e) => return Err(e.into()),
        };
        Ok(Self { path, state })
    }

    /// Hours used so far today. Calling this implicitly performs
    /// the local-midnight rollover check, so the returned value
    /// is always for "now in local time".
    pub fn hours_used_today(&mut self) -> f64 {
        self.maybe_reset();
        self.state.hours_today
    }

    /// Record `duration_hours` worth of work just completed.
    /// Persists immediately so a crash never loses budget
    /// accounting (worst case is a duplicate count if the crash
    /// happens between the in-memory bump and the rename).
    pub fn record_task(&mut self, duration_hours: f64) -> ConsentResult<()> {
        self.maybe_reset();
        self.state.hours_today += duration_hours;
        atomic_write_json(&self.path, &self.state)
    }

    /// Test-only override: simulate "the day rolled over to
    /// `new_today_local`" without waiting for chrono::Local.
    /// Public so the file watcher integration test can drive
    /// rollover deterministically.
    #[doc(hidden)]
    pub fn force_today_for_test(&mut self, new_today_local: impl Into<String>) {
        self.state.today_local = new_today_local.into();
        self.state.hours_today = 0.0;
    }

    fn maybe_reset(&mut self) {
        let today = today_local_iso();
        if self.state.today_local != today {
            debug!(
                old = %self.state.today_local,
                new = %today,
                "consent: usage rolled over to new local day"
            );
            self.state.today_local = today;
            self.state.hours_today = 0.0;
        }
    }

    /// Read access to the on-disk path (handy for diagnostics
    /// and tests).
    pub fn path(&self) -> &Path {
        &self.path
    }
}

// =================================================================
// Decision: should this task be accepted?
// =================================================================

/// Cheap, copy-friendly snapshot of the per-task fields the
/// consent layer cares about. The engine loop builds one of
/// these from the iroh `TaskEntry` it just polled, then calls
/// [`should_accept_task`] before doing anything else with it.
#[derive(Debug, Clone)]
pub struct TaskContext<'a> {
    /// `node_id` of the project that produced the task.
    pub project_id: &'a str,
    /// True iff the project's `ProjectAnnouncement` carries the
    /// `is_open_source = true` flag set by the coordinator at
    /// deploy-from-repo time. Phase D wires this through end-to-
    /// end; until then the engine passes `false` for every task,
    /// which means L2 rejects everything (acceptable: L2 is
    /// inert until Phase D ships).
    pub is_open_source: bool,
    /// Estimated sustained watts the task will draw.
    pub estimated_watts: u32,
    /// Estimated VRAM footprint in megabytes.
    pub estimated_vram_mb: u64,
    /// Estimated wall-clock duration in hours.
    pub estimated_hours: f64,
}

/// Verdict from [`should_accept_task`]. `Accept` means the
/// engine may proceed with the claim; every `Reject` carries a
/// machine-readable reason so the engine can log a structured
/// event for observability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AllowOutcome {
    Accept,
    Reject(RejectReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectReason {
    /// L1 picked, but the task is from a different project.
    NotOwnProject,
    /// L2 picked, but the task's `is_open_source` flag is
    /// false (or absent, which the worker treats as false).
    NotOpenSource,
    /// L3 picked, but `project_id` is not in
    /// [`ConsentConfig::allowed_project_ids`].
    NotInWhitelist,
    /// `estimated_watts` exceeds [`Caps::max_watts`].
    CapWatts,
    /// `estimated_vram_mb` exceeds [`Caps::max_vram_mb`].
    CapVram,
    /// `estimated_hours` would push today's cumulative usage
    /// beyond [`Caps::max_hours_day`].
    CapHoursToday,
}

/// The single source of truth for "may the worker claim this
/// task?". Pure with respect to its inputs (no I/O, no clock):
/// reads `usage.hours_used_today` which has its own clock side-
/// effect, but everything else is a deterministic function of
/// the arguments.
pub fn should_accept_task(
    task: &TaskContext<'_>,
    consent: &ConsentConfig,
    usage: &mut UsageTracker,
) -> AllowOutcome {
    // 1) Level filter.
    match consent.level {
        ConsentLevel::OwnProjects => {
            if task.project_id != consent.own_node_id {
                return AllowOutcome::Reject(RejectReason::NotOwnProject);
            }
        }
        ConsentLevel::OpenSource => {
            if !task.is_open_source {
                return AllowOutcome::Reject(RejectReason::NotOpenSource);
            }
        }
        ConsentLevel::Whitelist => {
            if !consent.allowed_project_ids.contains(task.project_id) {
                return AllowOutcome::Reject(RejectReason::NotInWhitelist);
            }
        }
        ConsentLevel::All => {}
    }

    // 2) Caps.
    if let Some(max_w) = consent.caps.max_watts {
        if task.estimated_watts > max_w {
            return AllowOutcome::Reject(RejectReason::CapWatts);
        }
    }
    if let Some(max_v) = consent.caps.max_vram_mb {
        if task.estimated_vram_mb > max_v {
            return AllowOutcome::Reject(RejectReason::CapVram);
        }
    }
    if let Some(max_h) = consent.caps.max_hours_day {
        let used = usage.hours_used_today();
        if used + task.estimated_hours > max_h {
            return AllowOutcome::Reject(RejectReason::CapHoursToday);
        }
    }

    AllowOutcome::Accept
}

// =================================================================
// File watcher — pick up `consent.json` edits without a restart
// =================================================================

/// Live, reload-on-change handle to a [`ConsentConfig`]. The
/// engine loop holds one of these and calls
/// [`ConsentWatcher::current`] at every claim tick — no need to
/// re-`load` the file by hand. When the coordinator rewrites
/// `consent.json` (via `POST /consent/set` or
/// `/consent/whitelist/*`), the `notify` background thread
/// re-reads the file and swaps the new state in atomically.
pub struct ConsentWatcher {
    inner: Arc<RwLock<ConsentConfig>>,
    _watcher: RecommendedWatcher,
    _join: Option<thread::JoinHandle<()>>,
}

impl ConsentWatcher {
    /// Spin up a watcher on `path`. Reads the file (or builds
    /// the default for `own_node_id` if absent) and starts the
    /// `notify` background thread. The thread debounces with a
    /// 50 ms sleep so an editor that does write+rename triggers
    /// at most one reload.
    pub fn spawn(path: impl Into<PathBuf>, own_node_id: impl Into<String>) -> ConsentResult<Self> {
        let path = path.into();
        let own_node_id = own_node_id.into();
        let initial = ConsentConfig::load_or_default(&path, &own_node_id)?;
        let inner = Arc::new(RwLock::new(initial));

        // Watch the parent directory rather than the file itself
        // so we still see write+rename atomic replacements (the
        // file inode changes when the coordinator rewrites it).
        let parent = path.parent().map(Path::to_path_buf).unwrap_or_else(|| {
            // Fallback for paths without a parent (e.g. relative
            // "consent.json" called from an integration test).
            PathBuf::from(".")
        });
        // Make sure the parent exists so notify::watch doesn't
        // fail on first boot.
        if !parent.exists() {
            fs::create_dir_all(&parent)?;
        }

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;
        watcher.watch(&parent, RecursiveMode::NonRecursive)?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let own_node_thread = own_node_id.clone();
        let join = thread::Builder::new()
            .name("sbfb-consent-watch".into())
            .spawn(move || {
                while let Ok(evt) = rx.recv() {
                    match evt {
                        Ok(event) => {
                            // Filter to events that touch the
                            // file we care about. notify reports
                            // every entry under the watched
                            // dir, so without this filter we
                            // would reload on unrelated writes
                            // (usage.json, allowlist.sqlite3...).
                            if !event.paths.iter().any(|p| p == &path_thread) {
                                continue;
                            }
                            if !matches!(
                                event.kind,
                                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                            ) {
                                continue;
                            }
                            // Debounce write+rename: editors
                            // typically emit Create+Modify in
                            // quick succession.
                            thread::sleep(Duration::from_millis(50));
                            match ConsentConfig::load_or_default(&path_thread, &own_node_thread) {
                                Ok(new_cfg) => {
                                    if let Ok(mut guard) = inner_thread.write() {
                                        *guard = new_cfg;
                                        debug!(
                                            path = %path_thread.display(),
                                            "consent.json reloaded"
                                        );
                                    }
                                }
                                Err(e) => warn!(
                                    error = %e,
                                    path = %path_thread.display(),
                                    "consent.json reload failed"
                                ),
                            }
                        }
                        Err(e) => warn!(error = %e, "consent watcher event error"),
                    }
                }
            })?;

        Ok(Self {
            inner,
            _watcher: watcher,
            _join: Some(join),
        })
    }

    /// Cheap clone of the current consent state. Returns a
    /// `ConsentConfig` rather than a guard so the caller doesn't
    /// hold the read lock across long operations like a task
    /// claim that does network I/O.
    pub fn current(&self) -> ConsentResult<ConsentConfig> {
        let guard = self.inner.read().map_err(|_| ConsentError::Poisoned)?;
        Ok(guard.clone())
    }

    /// Test-only escape hatch: replace the in-memory state
    /// without going through the file. Lets unit tests exercise
    /// `should_accept_task` without spinning a watcher thread.
    #[doc(hidden)]
    pub fn force_set_for_test(&self, new_cfg: ConsentConfig) -> ConsentResult<()> {
        let mut guard = self.inner.write().map_err(|_| ConsentError::Poisoned)?;
        *guard = new_cfg;
        Ok(())
    }
}

// =================================================================
// Shared on-disk paths (aligned with the Python coordinator)
// =================================================================
//
// The coordinator's `nexus_coordinator.api.consent` module writes
// these same paths via its own helper, keyed off the same
// `SBFB_HOME` env var. Keep the two in sync — the worker and the
// coordinator both need to read/write the exact same files or the
// dialog will silently desync from the worker enforcement.
//
// Duplicating `nexus_shell_daemon_core::auth::sbfb_home` here
// rather than adding a cross-crate dep is a deliberate Phase C
// choice: worker-core has no other reason to pull shell-daemon
// code in, and the helper is 10 lines. A future Sprint can hoist
// the shared logic into `nexus-core-rs::paths` if it grows.

/// Env var override — set to a temp dir in integration tests so
/// the worker does not touch the developer's real `~/.sbfb/`.
/// Same variable name as the shell daemon's `auth::sbfb_home`.
pub const SBFB_HOME_ENV: &str = "SBFB_HOME";

/// Resolve `~/.sbfb/` for the current user, or `None` on the
/// rare platform where neither `SBFB_HOME` nor `HOME` /
/// `USERPROFILE` is set.
pub fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var(SBFB_HOME_ENV) {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    let home = env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())?;
    if home.is_empty() {
        return None;
    }
    Some(PathBuf::from(home).join(".sbfb"))
}

/// Path to `consent.json` — the preferences file the coordinator
/// rewrites when the user clicks "Save" in the dialog.
pub fn consent_config_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("consent.json"))
}

/// Path to `usage.json` — the daily counter the worker rewrites
/// after every completed task.
pub fn usage_state_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("usage.json"))
}

// =================================================================
// Helpers
// =================================================================

fn atomic_write_json<T: Serialize>(path: &Path, value: &T) -> ConsentResult<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let body = serde_json::to_vec_pretty(value)?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn today_local_iso() -> String {
    chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string()
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use tempfile::tempdir;

    fn task<'a>(
        project_id: &'a str,
        is_open_source: bool,
        watts: u32,
        vram: u64,
        hours: f64,
    ) -> TaskContext<'a> {
        TaskContext {
            project_id,
            is_open_source,
            estimated_watts: watts,
            estimated_vram_mb: vram,
            estimated_hours: hours,
        }
    }

    fn fresh_usage(dir: &tempfile::TempDir) -> UsageTracker {
        UsageTracker::load_or_default(dir.path().join("usage.json")).unwrap()
    }

    // ----- Level filter -----

    #[test]
    fn l1_accepts_own_project_rejects_other() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let consent = ConsentConfig::default_for("node-self");

        assert_eq!(
            should_accept_task(
                &task("node-self", false, 100, 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Accept
        );
        assert_eq!(
            should_accept_task(
                &task("node-other", false, 100, 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Reject(RejectReason::NotOwnProject)
        );
    }

    #[test]
    fn l2_accepts_open_source_rejects_closed() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::OpenSource;

        assert_eq!(
            should_accept_task(
                &task("node-other", true, 100, 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Accept
        );
        assert_eq!(
            should_accept_task(
                &task("node-other", false, 100, 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Reject(RejectReason::NotOpenSource)
        );
    }

    #[test]
    fn l3_accepts_whitelist_rejects_other() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::Whitelist;
        consent.allowed_project_ids.insert("proj-a".into());

        assert_eq!(
            should_accept_task(&task("proj-a", false, 100, 1024, 0.5), &consent, &mut usage),
            AllowOutcome::Accept
        );
        assert_eq!(
            should_accept_task(&task("proj-b", false, 100, 1024, 0.5), &consent, &mut usage),
            AllowOutcome::Reject(RejectReason::NotInWhitelist)
        );
    }

    #[test]
    fn l3_empty_whitelist_rejects_everyone() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::Whitelist;

        assert_eq!(
            should_accept_task(
                &task("node-self", true, 100, 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Reject(RejectReason::NotInWhitelist)
        );
    }

    #[test]
    fn l4_accepts_all_subject_to_caps() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::All;

        assert_eq!(
            should_accept_task(&task("any", false, 100, 1024, 0.5), &consent, &mut usage),
            AllowOutcome::Accept
        );
    }

    // ----- Caps -----

    #[test]
    fn cap_watts_rejects_overage() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::All;
        consent.caps.max_watts = Some(200);

        assert_eq!(
            should_accept_task(&task("any", false, 250, 1024, 0.5), &consent, &mut usage),
            AllowOutcome::Reject(RejectReason::CapWatts)
        );
        assert_eq!(
            should_accept_task(&task("any", false, 199, 1024, 0.5), &consent, &mut usage),
            AllowOutcome::Accept
        );
    }

    #[test]
    fn cap_vram_rejects_overage() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::All;
        consent.caps.max_vram_mb = Some(8 * 1024);

        assert_eq!(
            should_accept_task(
                &task("any", false, 100, 16 * 1024, 0.5),
                &consent,
                &mut usage
            ),
            AllowOutcome::Reject(RejectReason::CapVram)
        );
    }

    #[test]
    fn cap_hours_rejects_when_cumulated_overage() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::All;
        consent.caps.max_hours_day = Some(2.0);

        usage.record_task(1.5).unwrap();
        // 1.5 + 0.6 = 2.1 > 2.0 -> reject
        assert_eq!(
            should_accept_task(&task("any", false, 100, 1024, 0.6), &consent, &mut usage),
            AllowOutcome::Reject(RejectReason::CapHoursToday)
        );
        // 1.5 + 0.4 = 1.9 <= 2.0 -> accept
        assert_eq!(
            should_accept_task(&task("any", false, 100, 1024, 0.4), &consent, &mut usage),
            AllowOutcome::Accept
        );
    }

    #[test]
    fn caps_none_means_no_cap() {
        let dir = tempdir().unwrap();
        let mut usage = fresh_usage(&dir);
        let mut consent = ConsentConfig::default_for("node-self");
        consent.level = ConsentLevel::All;
        consent.caps = Caps {
            max_watts: None,
            max_vram_mb: None,
            max_hours_day: None,
        };

        assert_eq!(
            should_accept_task(
                &task("any", false, 9999, 1_000_000, 24.0),
                &consent,
                &mut usage
            ),
            AllowOutcome::Accept
        );
    }

    // ----- UsageTracker -----

    #[test]
    fn usage_record_persists_across_reload() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("usage.json");
        {
            let mut u = UsageTracker::load_or_default(&path).unwrap();
            u.record_task(0.25).unwrap();
            u.record_task(0.50).unwrap();
        }
        let mut u = UsageTracker::load_or_default(&path).unwrap();
        assert!((u.hours_used_today() - 0.75).abs() < 1e-9);
    }

    #[test]
    fn usage_resets_on_local_midnight_rollover() {
        let dir = tempdir().unwrap();
        let mut u = fresh_usage(&dir);
        u.record_task(3.0).unwrap();
        assert!((u.hours_used_today() - 3.0).abs() < 1e-9);

        u.force_today_for_test("1970-01-01");
        // After force, the in-memory date is 1970, today_local_iso
        // returns 2026+, so the next call rebuilds to 0.
        assert_eq!(u.hours_used_today(), 0.0);
    }

    // ----- ConsentConfig persistence -----

    #[test]
    fn consent_load_or_default_returns_defaults_when_missing() {
        let dir = tempdir().unwrap();
        let cfg = ConsentConfig::load_or_default(&dir.path().join("absent.json"), "self").unwrap();
        assert_eq!(cfg.level, ConsentLevel::OwnProjects);
        assert_eq!(cfg.own_node_id, "self");
        assert!(cfg.allowed_project_ids.is_empty());
        // Defaults match the README expectation.
        assert_eq!(cfg.caps.max_watts, Some(400));
    }

    #[test]
    fn consent_save_then_load_roundtrips() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("consent.json");
        let mut cfg = ConsentConfig::default_for("self");
        cfg.level = ConsentLevel::Whitelist;
        cfg.allowed_project_ids.insert("proj-a".into());
        cfg.caps.max_watts = Some(150);
        cfg.save_atomic(&path).unwrap();

        let bytes = fs::read(&path).unwrap();
        // serde_json round-trips into the same struct.
        let loaded: ConsentConfig = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(loaded, cfg);
    }

    #[test]
    fn consent_level_serializes_as_integer() {
        let cfg = ConsentConfig {
            level: ConsentLevel::Whitelist,
            caps: Caps::default(),
            allowed_project_ids: HashSet::new(),
            own_node_id: "self".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&cfg).unwrap();
        // Wire format must stay {"level": 3, ...} so the React
        // shell and the FastAPI coordinator can speak it without
        // a custom deserializer.
        assert_eq!(v["level"], serde_json::json!(3));
    }

    #[test]
    fn consent_level_invalid_value_errors() {
        let bad = serde_json::json!({
            "level": 9,
            "caps": {"max_watts": null, "max_vram_mb": null, "max_hours_day": null},
            "allowed_project_ids": [],
            "own_node_id": "self"
        });
        let r: Result<ConsentConfig, _> = serde_json::from_value(bad);
        assert!(r.is_err(), "level=9 must fail to deserialize");
    }

    // ----- File watcher -----

    #[test]
    fn watcher_picks_up_external_rewrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("consent.json");

        // Seed with L1 default.
        ConsentConfig::default_for("self")
            .save_atomic(&path)
            .unwrap();

        let watcher = ConsentWatcher::spawn(&path, "self").unwrap();
        assert_eq!(watcher.current().unwrap().level, ConsentLevel::OwnProjects);

        // Rewrite externally to L4.
        let mut new_cfg = ConsentConfig::default_for("self");
        new_cfg.level = ConsentLevel::All;
        new_cfg.save_atomic(&path).unwrap();

        // Poll for up to 3s for the watcher thread to apply.
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut last = ConsentLevel::OwnProjects;
        while Instant::now() < deadline {
            last = watcher.current().unwrap().level;
            if last == ConsentLevel::All {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("watcher never reloaded — last level seen = {last:?}");
    }

    // Regression guard for Sprint 16 audit finding C-3: when the
    // inner RwLock is poisoned (a write-holding thread panicked),
    // `current()` must surface `ConsentError::Poisoned` so the
    // engine runtime can fail-closed instead of silently falling
    // back to an "accept all" branch. See `engine/runtime.rs`
    // consent filter block — the `Err(_)` arm must `continue;`.
    #[test]
    fn watcher_current_errors_on_poisoned_lock() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("consent.json");
        ConsentConfig::default_for("self")
            .save_atomic(&path)
            .unwrap();
        let watcher = ConsentWatcher::spawn(&path, "self").unwrap();

        // Poison the inner RwLock by panicking while holding the
        // write lock. `force_set_for_test` takes a write lock;
        // we drop a panicking closure inside the same thread.
        let inner = Arc::clone(&watcher.inner);
        let handle = thread::spawn(move || {
            let _guard = inner.write().unwrap();
            panic!("intentional poison for test");
        });
        let _ = handle.join(); // JoinError (the panic) is expected.

        let err = watcher.current().expect_err("poisoned lock surfaces error");
        assert!(
            matches!(err, ConsentError::Poisoned),
            "expected Poisoned, got {err:?}"
        );
    }
}
