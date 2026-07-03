// SPDX-License-Identifier: AGPL-3.0-or-later
//! Singleton registry: the `running.json` marker file.
//!
//! Sprint 7 D2 (frozen in the kickoff) picks a **strict
//! singleton** model: one `nexus-shell-daemon` per user at any
//! time. Multi-instance would mean two daemons subscribing to
//! the same curator gossip topic → amplification, and the shell
//! would not know which daemon's `/curators` endpoint to trust.
//!
//! The singleton is enforced via a platform-independent pid
//! file: on boot the daemon writes a `running.json` with its own
//! pid, and a second `start` invocation reads that file, checks
//! whether the pid still belongs to a `nexus-shell-daemon`
//! process, and bails out with a user-friendly error if it does.
//! Stale files left by crashed daemons are silently overwritten
//! with a warning log.
//!
//! ## Atomic write
//!
//! [`write_running`] writes through a temp file + rename pattern
//! identical to the worker state writer and the Python
//! coordinator registry. On POSIX this is atomic; on Windows
//! NTFS `rename` is atomic for same-directory moves.
//!
//! ## Pid recycling (R3 mitigation)
//!
//! [`check_stale_or_bail`] compares both the pid **and** the
//! process name before declaring the daemon "live". A pid
//! recycled by the OS for an unrelated `notepad.exe` must not
//! be mistaken for a running daemon. The helper substring-matches
//! the process name against `"nexus-shell-daemon"` so Windows
//! `.exe` suffixes and debug-vs-release paths both pass.
//!
//! ## Schema versioning
//!
//! [`SCHEMA_VERSION`] is literal in every `running.json`. Breaking
//! changes must bump it in lock-step with any TypeScript Zod mirror
//! on the coordinator-proxy side. Additive changes (new optional
//! fields) may stay on the same version as long as an older reader
//! can still parse a newer file.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sysinfo::{Pid, System};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::{debug, warn};

/// On-disk schema version. Bumped on breaking changes only.
pub const SCHEMA_VERSION: u32 = 1;

/// Expected process name substring used by
/// [`check_stale_or_bail`] to decide whether a pid still
/// belongs to a live shell daemon. `sysinfo::Process::name`
/// returns the executable file name, which is
/// `"nexus-shell-daemon"` on Unix and
/// `"nexus-shell-daemon.exe"` on Windows.
///
/// Matching is done through [`process_name_matches`] which
/// normalizes hyphens to underscores on **both** sides before
/// a substring check. That normalization is what lets the
/// cargo test harness — which compiles this binary as
/// `nexus_shell_daemon-<hash>[.exe]` (underscore-separated
/// because of the Rust crate name rule) — still match the
/// production hyphenated `EXPECTED_PROCESS_NAME`, so the
/// singleton enforcement is exercisable end-to-end from
/// inside a `#[tokio::test]`. Without normalization the
/// registry would classify a running test-binary daemon as
/// `Stale` and the singleton check would silently become a
/// no-op in unit tests — a blind spot caught by the Sprint 7
/// Phase A review.
pub const EXPECTED_PROCESS_NAME: &str = "nexus-shell-daemon";

// =================================================================
// Types
// =================================================================

/// The `running.json` payload.
///
/// Kept intentionally smaller than the Python coordinator's
/// `RunningState` — a daemon is **global per user**, not per
/// project, so there is no `project_name` or `visibility` field.
/// The `node_id` is the iroh endpoint id the daemon booted with,
/// not a persistent identity: Phase A regenerates a fresh key on
/// every boot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunningState {
    /// Always [`SCHEMA_VERSION`]. Deserializers reject mismatches.
    pub schema_version: u32,

    /// Ed25519 public key hex (64 lowercase chars) of the running
    /// daemon's iroh endpoint. May be empty during the brief
    /// window between `running.json` write and iroh node boot,
    /// but the Phase A runtime always writes this after `create_node()`.
    pub node_id: String,

    /// The `api_host` the HTTP server is bound to. Should always
    /// be `"127.0.0.1"` under the D1 loopback-only contract.
    pub api_host: String,

    /// The real port the HTTP listener bound to. Resolved from
    /// the `TcpListener::local_addr()` after `bind`, **not** from
    /// the config, so the ephemeral port 0 case works.
    pub api_port: u16,

    /// OS process id of the running daemon. Used by
    /// [`check_stale_or_bail`] to distinguish a live singleton
    /// from a crash-leftover file.
    pub pid: u32,

    /// RFC 3339 UTC timestamp of the daemon's boot. Phase E
    /// renders this in the shell's "Daemon offline" banner so
    /// users can see how long the daemon has been alive.
    pub started_at: String,

    /// Version of the `nexus-shell-daemon-core` crate the
    /// running daemon was compiled against. The shell compares
    /// this against its own schema version so a mismatched
    /// daemon (user upgraded the shell but forgot to restart
    /// the daemon) can be detected and flagged.
    pub daemon_version: String,
}

/// Outcome of [`check_stale_or_bail`]. The binary's start handler
/// matches on this to decide whether to boot, bail, or clean up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StaleOutcome {
    /// No `running.json` at the expected path — fresh boot.
    NoFile,
    /// A `running.json` exists but its pid is dead (crashed
    /// previous daemon). The binary should log a warning,
    /// overwrite it, and proceed.
    Stale {
        /// The dead pid that was recorded in the file.
        pid: u32,
        /// The running state as parsed from the file, for
        /// audit / logging.
        state: RunningState,
    },
    /// A `running.json` exists **and** its pid belongs to a live
    /// `nexus-shell-daemon` process. The binary must refuse to
    /// boot: the singleton is enforced.
    Live {
        /// The live pid the file points at.
        pid: u32,
        /// The running state as parsed from the file, for
        /// audit / logging.
        state: RunningState,
    },
    /// The file exists but is unreadable / malformed. Treated
    /// the same as `Stale` by the binary: overwrite and proceed.
    Corrupt {
        /// What specifically went wrong, for log output.
        reason: String,
    },
}

// =================================================================
// Errors
// =================================================================

/// Errors from the registry writer. All variants are logged by
/// the binary; write failures are fatal (the daemon refuses to
/// boot without a valid `running.json`) while read failures
/// degrade gracefully to `StaleOutcome::Corrupt`.
#[derive(Debug, Error)]
pub enum RegistryError {
    #[error("failed to serialize running.json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to create parent directory {path}: {source}")]
    MkDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write temporary running.json file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to rename {tmp} to {dest}: {source}")]
    Rename {
        tmp: PathBuf,
        dest: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

// =================================================================
// Writer
// =================================================================

/// Write a [`RunningState`] atomically to `dest`.
///
/// The function:
/// 1. Serializes `state` into pretty JSON so the file is
///    human-readable when debugging.
/// 2. Ensures every parent directory exists.
/// 3. Writes the bytes to `dest.with_extension("json.tmp")`
///    (fsynced).
/// 4. Renames the temp file over `dest`.
pub fn write_running(state: &RunningState, dest: &Path) -> Result<(), RegistryError> {
    let body = serde_json::to_vec_pretty(state)?;

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| RegistryError::MkDir {
            path: parent.to_path_buf(),
            source: e,
        })?;
    }

    let tmp = dest.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| RegistryError::Write {
            path: tmp.clone(),
            source: e,
        })?;
        f.write_all(&body).map_err(|e| RegistryError::Write {
            path: tmp.clone(),
            source: e,
        })?;
        f.sync_all().map_err(|e| RegistryError::Write {
            path: tmp.clone(),
            source: e,
        })?;
    }

    fs::rename(&tmp, dest).map_err(|e| RegistryError::Rename {
        tmp: tmp.clone(),
        dest: dest.to_path_buf(),
        source: e,
    })?;

    debug!(path = %dest.display(), pid = state.pid, "running.json written");
    Ok(())
}

/// Best-effort removal of `running.json` on graceful shutdown.
///
/// A non-existent file is not an error — the daemon may never
/// have finished writing it (very early boot failure), or the
/// user may have wiped the directory. Any other OS error is
/// logged and swallowed so the shutdown sequence is not
/// derailed by a stale permission bit.
pub fn remove_running(path: &Path) {
    match fs::remove_file(path) {
        Ok(()) => debug!(path = %path.display(), "running.json removed"),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!(path = %path.display(), "running.json already absent");
        }
        Err(e) => warn!(path = %path.display(), error = %e, "failed to remove running.json"),
    }
}

/// Best-effort read of an existing `running.json`. Returns
/// `None` if the file is missing, unreadable, or malformed.
///
/// Used from inside [`check_stale_or_bail`] (and tests); it is NOT
/// wired into any HTTP handler — `/api/daemon/info` serves a
/// distinct `DaemonStateSnapshot`, not this `RunningState`.
pub fn read_running(path: &Path) -> Option<RunningState> {
    let body = fs::read_to_string(path).ok()?;
    serde_json::from_str::<RunningState>(&body).ok()
}

// =================================================================
// Stale / live detection
// =================================================================

/// Decide whether a [`RunningState`] at `path` represents a live
/// daemon, a dead daemon (stale), a corrupt file, or nothing at
/// all.
///
/// The logic is:
/// 1. If the file does not exist → `NoFile`.
/// 2. If the file is unreadable or schema-mismatched → `Corrupt`.
/// 3. If the recorded pid does NOT match a live process whose
///    name contains [`EXPECTED_PROCESS_NAME`] → `Stale`.
/// 4. Otherwise → `Live`.
///
/// The process name check is the R3 mitigation: a pid recycled
/// by the OS for a different binary must not be mistaken for a
/// live shell daemon.
pub fn check_stale_or_bail(path: &Path) -> StaleOutcome {
    if !path.exists() {
        return StaleOutcome::NoFile;
    }

    let body = match fs::read_to_string(path) {
        Ok(b) => b,
        Err(e) => {
            return StaleOutcome::Corrupt {
                reason: format!("read failed: {e}"),
            };
        }
    };

    let state: RunningState = match serde_json::from_str(&body) {
        Ok(s) => s,
        Err(e) => {
            return StaleOutcome::Corrupt {
                reason: format!("json parse: {e}"),
            };
        }
    };

    if state.schema_version != SCHEMA_VERSION {
        return StaleOutcome::Corrupt {
            reason: format!(
                "schema_version mismatch (file={}, current={})",
                state.schema_version, SCHEMA_VERSION
            ),
        };
    }

    let pid = state.pid;
    if is_process_alive(pid, EXPECTED_PROCESS_NAME) {
        StaleOutcome::Live { pid, state }
    } else {
        StaleOutcome::Stale { pid, state }
    }
}

/// Return `true` iff two process-name strings match under the
/// registry's normalization rules.
///
/// Rules:
/// - Lowercase both sides (Windows filenames are case-insensitive;
///   `Process.name()` may report `Nexus-Shell-Daemon.exe`).
/// - Replace every `-` with `_`. This bridges the hyphen-vs-
///   underscore gap between the production binary
///   (`nexus-shell-daemon[.exe]`, hyphens: clap `[[bin]] name`)
///   and the cargo test binary (`nexus_shell_daemon-<hash>[.exe]`,
///   underscores: Rust crate name rule).
/// - Strip a trailing `.exe` on the observed side (Windows).
/// - Accept only one of the following exact shapes:
///   1. `<expected>` (the production binary, sans extension).
///   2. `<expected>_core` (sibling core crate in production —
///      reserved for a future split, accepted pre-emptively so
///      we don't revisit the rule).
///   3. `<expected>_<hex hash>` — cargo test binary for the
///      `nexus-shell-daemon` crate. The tail after the `_`
///      must be non-empty and contain nothing but ASCII hex
///      digits (`cargo test` names its test binaries
///      `<crate>-<16-char hex hash>`, which normalizes to a
///      `_hex` suffix after the hyphen→underscore pass).
///   4. `<expected>_core_<hex hash>` — cargo test binary for
///      `nexus-shell-daemon-core`, same pattern.
///
/// Sprint 7 audit finding D-1: the previous implementation did a
/// naive `contains` substring check after normalization. That
/// accepted legitimate cases (`nexus-shell-daemon`,
/// `nexus_shell_daemon_core-abc123`) but also passed unrelated
/// binaries that happened to embed the prefix
/// (`nexus-shell-daemon-launcher.exe`,
/// `my-nexus-shell-daemon-wrapper`). The boundary-aware version
/// here refuses those while keeping every accepted case
/// unchanged, which keeps the unit-test-inside-binary singleton
/// enforcement working end-to-end.
pub fn process_name_matches(observed: &str, expected: &str) -> bool {
    fn norm(s: &str) -> String {
        s.to_lowercase().replace('-', "_")
    }
    let observed_norm = norm(observed);
    let expected_norm = norm(expected);

    let observed_trimmed: &str = observed_norm
        .strip_suffix(".exe")
        .unwrap_or(observed_norm.as_str());

    let roots = [expected_norm.clone(), format!("{expected_norm}_core")];
    for root in &roots {
        if observed_trimmed == root.as_str() {
            return true;
        }
        if let Some(tail) = observed_trimmed.strip_prefix(&format!("{root}_"))
            && !tail.is_empty()
            && tail.chars().all(|c| c.is_ascii_hexdigit())
        {
            return true;
        }
        // Linux /proc/[pid]/comm truncates to 15 chars. Accept if the
        // observed name is a prefix of a root and at least 15 chars long.
        if observed_trimmed.len() >= 15 && root.starts_with(observed_trimmed) {
            return true;
        }
    }
    false
}

/// Return `true` iff `pid` currently maps to a process whose
/// executable name matches `expected_name` under
/// [`process_name_matches`].
///
/// Uses `sysinfo::System::new_all()` which performs a full
/// process table refresh on construction. This is a one-shot
/// boot-time check, not a hot path, so the cost is irrelevant —
/// and the simpler API avoids the `refresh_processes` signature
/// drift between sysinfo 0.30 / 0.31 / 0.32.
pub fn is_process_alive(pid: u32, expected_name: &str) -> bool {
    let sys = System::new_all();
    match sys.process(Pid::from_u32(pid)) {
        Some(proc_handle) => {
            // `process.name()` returns `&OsStr` on sysinfo 0.32;
            // lossy conversion is fine because the normalization
            // step will lowercase and substring-match anyway.
            let name = proc_handle.name().to_string_lossy();
            process_name_matches(&name, expected_name)
        }
        None => false,
    }
}

// =================================================================
// Builder helper
// =================================================================

/// Build a [`RunningState`] with `started_at = now` and the
/// current process's pid. The `node_id`, `api_host`, `api_port`
/// and `daemon_version` fields are caller-supplied because they
/// depend on the runtime (live `Node`, bound `TcpListener`,
/// compile-time version constant) rather than the clock.
pub fn new_running_state(
    node_id: String,
    api_host: String,
    api_port: u16,
    daemon_version: String,
) -> RunningState {
    RunningState {
        schema_version: SCHEMA_VERSION,
        node_id,
        api_host,
        api_port,
        pid: std::process::id(),
        started_at: iso_utc_now(),
        daemon_version,
    }
}

fn iso_utc_now() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn mk_state(pid: u32) -> RunningState {
        RunningState {
            schema_version: SCHEMA_VERSION,
            node_id: "deadbeef".repeat(8),
            api_host: "127.0.0.1".to_string(),
            api_port: 45678,
            pid,
            started_at: "2026-04-11T12:00:00Z".to_string(),
            daemon_version: crate::VERSION.to_string(),
        }
    }

    #[test]
    fn schema_version_is_one() {
        assert_eq!(SCHEMA_VERSION, 1, "bumping this is a breaking change");
    }

    #[test]
    fn write_read_roundtrip_preserves_state() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("shell-daemon").join("running.json");
        let state = mk_state(42);

        write_running(&state, &path).expect("write succeeds");
        assert!(path.exists());

        let read = read_running(&path).expect("read succeeds");
        assert_eq!(read, state);
    }

    #[test]
    fn write_creates_parent_directory() {
        let dir = tempdir().unwrap();
        let path = dir
            .path()
            .join("deeply")
            .join("nested")
            .join("running.json");
        assert!(!path.parent().unwrap().exists());

        write_running(&mk_state(1), &path).expect("write with nested parents");
        assert!(path.exists());
    }

    #[test]
    fn write_is_atomic_on_overwrite() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");

        write_running(&mk_state(1), &path).unwrap();
        write_running(&mk_state(2), &path).unwrap();

        let read = read_running(&path).expect("second read");
        assert_eq!(read.pid, 2);
        assert!(
            !path.with_extension("json.tmp").exists(),
            "temp sibling must not be left behind after a successful rename"
        );
    }

    #[test]
    fn remove_running_silently_ignores_missing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("never-written.json");
        assert!(!path.exists());
        remove_running(&path);
    }

    #[test]
    fn remove_running_removes_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");
        write_running(&mk_state(99), &path).unwrap();
        assert!(path.exists());
        remove_running(&path);
        assert!(!path.exists());
    }

    #[test]
    fn check_stale_or_bail_returns_no_file_when_absent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");
        assert_eq!(check_stale_or_bail(&path), StaleOutcome::NoFile);
    }

    #[test]
    fn check_stale_or_bail_returns_corrupt_on_bad_json() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");
        fs::write(&path, b"not json").unwrap();

        match check_stale_or_bail(&path) {
            StaleOutcome::Corrupt { reason } => {
                assert!(!reason.is_empty());
                assert!(reason.contains("json parse") || reason.contains("parse"));
            }
            other => panic!("expected Corrupt, got {other:?}"),
        }
    }

    #[test]
    fn check_stale_or_bail_returns_corrupt_on_schema_mismatch() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");

        // Synthesize a JSON object with a future schema version
        // directly — the regular RunningState serializer always
        // uses SCHEMA_VERSION so we bypass it here.
        fs::write(
            &path,
            br#"{
  "schema_version": 999,
  "node_id": "deadbeef",
  "api_host": "127.0.0.1",
  "api_port": 1234,
  "pid": 1,
  "started_at": "2026-04-11T12:00:00Z",
  "daemon_version": "0.0.0"
}"#,
        )
        .unwrap();

        match check_stale_or_bail(&path) {
            StaleOutcome::Corrupt { reason } => {
                assert!(reason.contains("schema_version"), "got reason: {reason}");
            }
            other => panic!("expected Corrupt(schema), got {other:?}"),
        }
    }

    #[test]
    fn check_stale_or_bail_returns_stale_for_dead_pid() {
        // A pid of 0 is never a live process on any supported
        // platform, so the name check falls through to `None`
        // and the outcome is Stale.
        let dir = tempdir().unwrap();
        let path = dir.path().join("running.json");
        write_running(&mk_state(0), &path).unwrap();

        match check_stale_or_bail(&path) {
            StaleOutcome::Stale { pid, state } => {
                assert_eq!(pid, 0);
                assert_eq!(state.pid, 0);
            }
            other => panic!("expected Stale, got {other:?}"),
        }
    }

    #[test]
    fn is_process_alive_false_for_zero_pid() {
        // pid 0 is either the scheduler / idle task (Unix) or
        // invalid (Windows). `sysinfo` never exposes it as a
        // named `nexus-shell-daemon`, so the helper must return
        // false regardless of the host OS.
        assert!(!is_process_alive(0, EXPECTED_PROCESS_NAME));
    }

    #[test]
    fn process_name_matches_handles_hyphen_underscore_drift() {
        // Production binary vs. cargo test binary:
        assert!(process_name_matches(
            "nexus-shell-daemon.exe",
            EXPECTED_PROCESS_NAME
        ));
        assert!(process_name_matches(
            "nexus_shell_daemon-abc123.exe",
            EXPECTED_PROCESS_NAME
        ));
        // The core crate's cargo test binary (double
        // underscore because the crate is `nexus-shell-daemon-core`):
        assert!(process_name_matches(
            "nexus_shell_daemon_core-abc123",
            EXPECTED_PROCESS_NAME
        ));
        // Case-insensitive match on Windows:
        assert!(process_name_matches(
            "Nexus-Shell-Daemon.EXE",
            EXPECTED_PROCESS_NAME
        ));
        // Linux /proc/[pid]/comm truncates to 15 chars:
        assert!(process_name_matches(
            "nexus_shell_dae",
            EXPECTED_PROCESS_NAME
        ));
    }

    #[test]
    fn process_name_matches_rejects_unrelated_binaries() {
        // R3 mitigation: pid-recycled notepad.exe must NOT be
        // mistaken for our daemon.
        assert!(!process_name_matches("notepad.exe", EXPECTED_PROCESS_NAME));
        assert!(!process_name_matches("python.exe", EXPECTED_PROCESS_NAME));
        assert!(!process_name_matches(
            "nexus-worker.exe",
            EXPECTED_PROCESS_NAME
        ));
        // A malicious process named to look like loopback
        // truncation must not squeak through.
        assert!(!process_name_matches(
            "nexus-shell.exe",
            EXPECTED_PROCESS_NAME
        ));
    }

    #[test]
    fn process_name_rejects_prefix_extension() {
        // Sprint 7 audit finding D-1 regression guard.
        //
        // The previous `contains` substring check accepted any
        // name that embedded `nexus_shell_daemon` anywhere —
        // including a hypothetical launcher binary that just
        // happens to share the prefix. The boundary-aware check
        // refuses these because the tail after `_` contains
        // non-hex characters, which cannot come from a cargo
        // test hash suffix.
        assert!(!process_name_matches(
            "nexus-shell-daemon-launcher.exe",
            EXPECTED_PROCESS_NAME
        ));
        assert!(!process_name_matches(
            "nexus_shell_daemon_launcher",
            EXPECTED_PROCESS_NAME
        ));
        assert!(!process_name_matches(
            "my-nexus-shell-daemon-wrapper",
            EXPECTED_PROCESS_NAME
        ));
        // Prefix match with a trailing non-hex word masquerading
        // as a hash: the underscore boundary is not enough on
        // its own — the whole tail must be hex.
        assert!(!process_name_matches(
            "nexus_shell_daemon_notahex",
            EXPECTED_PROCESS_NAME
        ));
    }

    #[test]
    fn is_process_alive_matches_live_test_binary_under_expected_name() {
        // Under the hyphen/underscore normalization rule the
        // current cargo-test binary — whose name is
        // `nexus_shell_daemon_core-<hash>[.exe]` — must match
        // the production `EXPECTED_PROCESS_NAME`. This is the
        // property that lets the singleton enforcement be
        // exercised from inside unit tests, which would
        // otherwise silently classify a live test daemon as
        // stale.
        let own_pid = std::process::id();
        assert!(is_process_alive(own_pid, EXPECTED_PROCESS_NAME));
    }

    #[test]
    fn is_process_alive_false_for_wrong_name() {
        // Same pid, but a name substring that our test binary
        // cannot possibly contain. This isolates the R3
        // mitigation: even a live pid must not be treated as
        // our daemon if the name is wrong.
        let own_pid = std::process::id();
        assert!(!is_process_alive(
            own_pid,
            "definitely-not-our-binary-xyzzy"
        ));
    }

    #[test]
    fn new_running_state_uses_current_pid_and_schema_version() {
        let state = new_running_state(
            "n0".to_string(),
            "127.0.0.1".to_string(),
            8080,
            "0.1.0".to_string(),
        );
        assert_eq!(state.schema_version, SCHEMA_VERSION);
        assert_eq!(state.pid, std::process::id());
        assert_eq!(state.api_host, "127.0.0.1");
        assert_eq!(state.api_port, 8080);
        assert!(
            state.started_at.ends_with('Z'),
            "started_at must be RFC 3339 UTC, got {}",
            state.started_at
        );
    }
}
