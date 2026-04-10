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

/// Return `~/.nexus-grid/` — the shared nexus-grid root, matching
/// the path the Python coordinator writes under.
///
/// Returns `None` on the rare platform where neither `HOME` nor
/// `%APPDATA%` is set (CI sandboxes, embedded systems). Callers
/// should degrade gracefully: the state writer simply skips the
/// flush tick and logs a warning.
pub fn nexus_grid_root() -> Option<PathBuf> {
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
