// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared filesystem layout between the Rust worker and the
//! Python coordinator.
//!
//! The Sprint 5 shell reads the worker's live state through the
//! coordinator proxy, which in turn reads a `state.json` file on
//! disk (see [`worker_state_file`]). Both sides must agree on the
//! exact location of that file.
//!
//! The Python coordinator uses
//! `platformdirs.user_data_dir("nexus-grid", appauthor=False)`
//! to resolve its root, which yields:
//!
//! - Linux:   `~/.local/share/nexus-grid`
//! - macOS:   `~/Library/Application Support/nexus-grid`
//! - Windows: `%APPDATA%\nexus-grid`
//!
//! The Rust side used to mirror a *different* layout through the
//! [`directories`] crate
//! (`<data>/FlowUP/nexus-grid/data/`). That older layout is still
//! valid for the worker's own config, allowlist and iroh state —
//! this module deliberately does **not** replace
//! [`crate::config::WorkerPaths`]. It only resolves the shared
//! `~/.nexus-grid/` root so that the Sprint 5 shell integration
//! (a brand-new file written by the worker and read by the
//! coordinator) lands in the same place on both sides.
//!
//! If you find yourself reaching for [`nexus_grid_root`] from
//! inside the W3 config loader you are almost certainly doing
//! something wrong — use [`crate::config::WorkerPaths`] instead.
//! This module exists for the shell integration only.

use std::path::PathBuf;

use directories::BaseDirs;

/// Environment variable honoured by [`nexus_grid_root`] so
/// integration tests (the Sprint 5 Python test that spawns a
/// real `nexus-worker start --stub-ollama`, Playwright
/// globalSetup, future e2e harnesses) can redirect the whole
/// nexus-grid tree at a hermetic throwaway dir without
/// clobbering the developer's real user data. Matches the
/// coordinator-side override in `nexus_coordinator.paths`.
pub const NEXUS_GRID_ROOT_ENV: &str = "NEXUS_GRID_ROOT";

/// Return `~/.nexus-grid/` — the shared nexus-grid root, matching
/// the path the Python coordinator writes under.
///
/// If [`NEXUS_GRID_ROOT_ENV`] is set in the environment, its
/// value is used verbatim — this is the single override point
/// for tests that need a hermetic tree. Otherwise falls back to
/// the platform's `BaseDirs` user data directory.
///
/// Returns `None` on the rare platform where neither `HOME` nor
/// `%APPDATA%` is set (CI sandboxes, embedded systems) AND the
/// override is not set. Callers should degrade gracefully: the
/// state writer simply skips the flush tick and logs a warning.
pub fn nexus_grid_root() -> Option<PathBuf> {
    if let Ok(override_dir) = std::env::var(NEXUS_GRID_ROOT_ENV) {
        if !override_dir.is_empty() {
            return Some(PathBuf::from(override_dir));
        }
    }
    BaseDirs::new().map(|b| b.data_dir().join("nexus-grid"))
}

/// Return `<root>/worker/` — the directory holding the worker's
/// shell-facing state file.
pub fn worker_state_dir() -> Option<PathBuf> {
    nexus_grid_root().map(|r| r.join("worker"))
}

/// Return `<root>/worker/state.json` — the canonical worker state
/// snapshot path.
///
/// The file is rewritten atomically every
/// [`crate::config::Engine::state_flush_secs`] seconds by the
/// engine's main loop and is read lazily by the coordinator
/// `/worker-state` endpoint.
pub fn worker_state_file() -> Option<PathBuf> {
    worker_state_dir().map(|d| d.join("state.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nexus_grid_root_resolves_on_this_platform() {
        // CI runners on Linux / macOS / Windows all have HOME or
        // APPDATA set, so this should never be None in practice.
        // If a future sandbox breaks the assumption we want the
        // state_writer to short-circuit, not the whole test suite,
        // so the helper returns `Option`.
        let root = nexus_grid_root().expect("BaseDirs must resolve on CI");
        assert!(root.ends_with("nexus-grid"));
    }

    #[test]
    fn worker_state_file_is_under_worker_dir() {
        let file = worker_state_file().expect("BaseDirs must resolve on CI");
        assert_eq!(file.file_name().unwrap(), "state.json");
        assert_eq!(file.parent().unwrap().file_name().unwrap(), "worker");
        assert_eq!(
            file.parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_name()
                .unwrap(),
            "nexus-grid"
        );
    }
}
