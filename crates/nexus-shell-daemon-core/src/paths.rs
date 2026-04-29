// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared filesystem layout for the shell daemon.
//!
//! The daemon writes two things on disk during a normal boot:
//!
//! 1. `running.json` (see [`crate::registry`]) — the singleton
//!    marker the shell (via the coordinator `/daemon` proxy)
//!    uses to discover a live daemon.
//! 2. `config.toml` (see [`crate::config`]) — the persisted
//!    [`crate::config::ShellDaemonConfig`] when the user runs
//!    `nexus-shell-daemon config set ...`.
//!
//! Both files live under a single directory, platform-resolved
//! via the shared `~/.nexus-grid/` root. The root itself is
//! exactly the directory the coordinator uses
//! (`nexus_coordinator.paths`) and the worker uses
//! (`nexus_worker_core::paths`) so a given machine has a single
//! on-disk footprint for the whole SBFB stack.
//!
//! ## Why duplicate `nexus_grid_root`?
//!
//! [`nexus_worker_core::paths::nexus_grid_root`] already
//! implements this helper. We re-implement it here on purpose
//! rather than creating a cross-crate dependency between two
//! sibling crates that have **no other reason** to couple:
//! the shell daemon does not use the worker's allowlist, gpu
//! monitor, ollama client, or engine loop, and vice-versa.
//! A cross-crate dep for a 20-line helper would lock them
//! together for any future release. The duplication is
//! documented and the `NEXUS_GRID_ROOT` env contract is the
//! single source of truth both implementations must honour.
//!
//! If the two implementations ever drift, the failing test is
//! [`tests::daemon_root_uses_nexus_grid_convention`] — it asserts
//! the last path segment is `"nexus-grid"`, which is the same
//! invariant the worker side tests.

use std::path::PathBuf;

use directories::BaseDirs;

/// Environment variable honoured by [`nexus_grid_root`] so
/// integration tests (the Phase A e2e suite that spawns a real
/// `nexus-shell-daemon start --headless`, Phase E Playwright,
/// future end-to-end harnesses) can redirect the whole
/// nexus-grid tree at a hermetic throwaway dir without
/// clobbering the developer's real user data. Matches the
/// `NEXUS_GRID_ROOT` override in `nexus_coordinator.paths` and
/// `nexus_worker_core::paths::NEXUS_GRID_ROOT_ENV`.
pub const NEXUS_GRID_ROOT_ENV: &str = "NEXUS_GRID_ROOT";

/// Return `~/.nexus-grid/` — the shared nexus-grid root, matching
/// the path the Python coordinator and the Rust worker write
/// under.
///
/// If [`NEXUS_GRID_ROOT_ENV`] is set in the environment, its
/// value is used verbatim — this is the single override point
/// for tests that need a hermetic tree. Otherwise falls back to
/// the platform's `BaseDirs` user data directory.
///
/// Returns `None` on the rare platform where neither `HOME` nor
/// `%APPDATA%` is set AND the override is not set. Callers
/// should degrade gracefully: the runtime logs the failure and
/// refuses to boot, which is safer than writing state to `.`.
pub fn nexus_grid_root() -> Option<PathBuf> {
    if let Ok(override_dir) = std::env::var(NEXUS_GRID_ROOT_ENV) {
        if !override_dir.is_empty() {
            return Some(PathBuf::from(override_dir));
        }
    }
    BaseDirs::new().map(|b| b.data_dir().join("nexus-grid"))
}

/// Return `<root>/shell-daemon/` — the directory holding every
/// shell-daemon file. Sits next to `<root>/worker/` and
/// `<root>/projects/`, each of which is owned by a different
/// SBFB process. Names chosen to keep `ls ~/.nexus-grid/`
/// self-descriptive.
pub fn shell_daemon_dir() -> Option<PathBuf> {
    nexus_grid_root().map(|r| r.join("shell-daemon"))
}

/// Return `<root>/shell-daemon/running.json` — the singleton
/// marker file the [`crate::registry`] module writes on boot
/// and removes on shutdown.
pub fn running_json_path() -> Option<PathBuf> {
    shell_daemon_dir().map(|d| d.join("running.json"))
}

/// Return `<root>/shell-daemon/config.toml` — the on-disk
/// [`crate::config::ShellDaemonConfig`] file. Phase A does not
/// yet auto-create this file; `config set` is wired as a
/// Phase A stub and will be filled in Phase E alongside the
/// coordinator proxy wiring.
pub fn config_toml_path() -> Option<PathBuf> {
    shell_daemon_dir().map(|d| d.join("config.toml"))
}

/// Return `<root>/logs/` — the shared rotating log directory for
/// all SBFB binaries (daemon, launcher). Each binary writes its
/// own log file (`daemon.log`, `launcher.log`) with daily rotation.
pub fn log_dir() -> Option<PathBuf> {
    nexus_grid_root().map(|r| r.join("logs"))
}

/// Return `<root>/shell-daemon/subscriptions.json` — the
/// persistent attention set file the Phase C curator runtime
/// rewrites on every subscribe / unsubscribe call (R7
/// mitigation per `.planning/sprint7_plan.md` §13).
pub fn subscriptions_json_path() -> Option<PathBuf> {
    shell_daemon_dir().map(|d| d.join("subscriptions.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each test in this module mutates the `NEXUS_GRID_ROOT`
    /// env var. Serialize them so cargo's parallel test runner
    /// does not observe a racing value.
    fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    #[test]
    fn daemon_root_uses_nexus_grid_convention() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: guarded by env_lock above; no parallel test can
        // observe the transient removal.
        std::env::remove_var(NEXUS_GRID_ROOT_ENV);

        let root = nexus_grid_root().expect("BaseDirs must resolve on CI");
        assert!(
            root.ends_with("nexus-grid"),
            "root must end with the nexus-grid segment, got {}",
            root.display()
        );
    }

    #[test]
    fn env_override_is_honoured_verbatim() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        // SAFETY: guarded by env_lock so no parallel test will
        // observe the transient set.
        std::env::set_var(NEXUS_GRID_ROOT_ENV, tmp.path());

        let root = nexus_grid_root().expect("override returns Some");
        assert_eq!(root, tmp.path(), "override path must be used verbatim");

        std::env::remove_var(NEXUS_GRID_ROOT_ENV);
    }

    #[test]
    fn empty_override_falls_back_to_base_dirs() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var(NEXUS_GRID_ROOT_ENV, "");

        let root = nexus_grid_root().expect("fallback path");
        assert!(
            root.ends_with("nexus-grid"),
            "empty override should fall back to BaseDirs, got {}",
            root.display()
        );

        std::env::remove_var(NEXUS_GRID_ROOT_ENV);
    }

    #[test]
    fn shell_daemon_paths_are_nested_under_root() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        std::env::set_var(NEXUS_GRID_ROOT_ENV, tmp.path());

        let dir = shell_daemon_dir().expect("shell_daemon_dir");
        assert_eq!(dir, tmp.path().join("shell-daemon"));

        let running = running_json_path().expect("running_json_path");
        assert_eq!(running, dir.join("running.json"));

        let config = config_toml_path().expect("config_toml_path");
        assert_eq!(config, dir.join("config.toml"));

        let logs = log_dir().expect("log_dir");
        assert_eq!(logs, tmp.path().join("logs"));

        let subscriptions = subscriptions_json_path().expect("subscriptions_json_path");
        assert_eq!(subscriptions, dir.join("subscriptions.json"));

        std::env::remove_var(NEXUS_GRID_ROOT_ENV);
    }

    #[test]
    fn log_dir_is_under_grid_root_not_daemon_dir() {
        let _guard = env_lock().lock().unwrap_or_else(|e| e.into_inner());
        let tmp = tempfile::tempdir().expect("tempdir");
        std::env::set_var(NEXUS_GRID_ROOT_ENV, tmp.path());

        let logs = log_dir().expect("log_dir");
        let daemon = shell_daemon_dir().expect("shell_daemon_dir");
        assert_eq!(logs, tmp.path().join("logs"));
        assert!(
            !logs.starts_with(&daemon),
            "log_dir must NOT be under shell-daemon/"
        );

        std::env::remove_var(NEXUS_GRID_ROOT_ENV);
    }
}
