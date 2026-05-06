// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hot-reload loader for [`RelayPowPolicy`].
//!
//! Sprint 20 Phase C wires the S19 PoW Hashcash primitive into the
//! daemon runtime. The primitive already knows how to *parse* a
//! policy file (cf. [`nexus_core_rs::relay_pow_policy`]) ; this
//! module adds the piece the runtime needs to benefit from per-topic
//! overrides without restarting : a `notify`-driven watcher that
//! re-reads the TOML on every write and atomically swaps the shared
//! state an operator (or the Phase D dashboard) may edit while the
//! daemon is running.
//!
//! Pattern mirrors the S16 `ConsentWatcher` (nexus-worker-core) and
//! the S18 D-1 [`crate::auth::TokenRotatorWatcher`] : watch the
//! **parent directory** (not the file), filter events by absolute
//! path, debounce 50 ms to catch write+rename atomic rewrites, and
//! keep the previous in-memory policy on any reload failure. A
//! malformed TOML edit never locks the daemon out of its PoW gate —
//! it just prints a `warn!` and keeps enforcing the last known-good
//! policy.
//!
//! ## Why parent-dir watch
//!
//! Atomic rewrites (editors, the operator's `mv tmp
//! relay_pow_policy.toml`, and the tempfile + rename idiom) detach a
//! file-level watch because the inode changes. Watching the parent
//! directory and filtering by path is the portable fix used
//! throughout the workspace.
//!
//! ## Error semantics
//!
//! - **Missing file at spawn** → start on [`DEFAULT_POW_POLICY`] (2^18
//!   default difficulty, no overrides). The daemon always has a live
//!   policy, even if the operator never creates the file.
//! - **Malformed TOML at spawn** → surface the error upstream. A fresh
//!   boot with a broken config is a loud problem the operator should
//!   fix before the daemon opens the gossip gate.
//! - **Malformed TOML at reload** → `warn!` with the parse error and
//!   keep the previous in-memory policy. A mid-lifetime typo must not
//!   lock peers out.
//! - **File deletion at reload** → `warn!` and keep the previous
//!   policy, same rationale as the TokenRotator C-4 fix.

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use nexus_core_rs::{
    DEFAULT_POW_POLICY as DEFAULT_POLICY, RelayPowPolicy, load_relay_pow_policy_from,
};
use tracing::{debug, warn};

/// Live, reload-on-change handle to a [`RelayPowPolicy`]. Held by
/// the daemon runtime for the whole process lifetime — dropping the
/// watcher shuts the `notify` observer down and closes the
/// background reload thread.
///
/// Multiple callers (the gossip receive loop, the `POST /publish`
/// HTTP handler, the test harness) hold cheap
/// [`Arc<RwLock<RelayPowPolicy>>`] clones obtained via
/// [`PowPolicyWatcher::shared`] and read the inner policy on every
/// verify or solve. A launcher-issued rotation (operator edits the
/// file) propagates without a daemon restart.
pub struct PowPolicyWatcher {
    inner: Arc<RwLock<RelayPowPolicy>>,
    /// Held for `Drop` : the watcher background thread stops when
    /// the notify channel closes.
    _watcher: notify::RecommendedWatcher,
    /// Joined on `Drop` for clean teardown ; tests rely on the
    /// watcher thread exiting before the tempdir disappears.
    _join: Option<std::thread::JoinHandle<()>>,
}

impl PowPolicyWatcher {
    /// Build a watcher rooted at `path`. The initial policy is
    /// loaded synchronously via [`load_relay_pow_policy_from`] : a
    /// missing file resolves to [`DEFAULT_POLICY`], a malformed TOML
    /// surfaces as an error so the operator sees the problem at
    /// boot.
    ///
    /// The watcher keeps an absolute reference to `path` ; the
    /// caller is expected to hand over an already-canonicalised
    /// path (the daemon uses
    /// [`nexus_core_rs::relay_pow_policy_file_path`] which returns a
    /// canonical `$SBFB_HOME/relay_pow_policy.toml`).
    pub fn spawn(path: PathBuf) -> anyhow::Result<Self> {
        let initial = load_relay_pow_policy_from(&path)
            .map_err(|e| anyhow::anyhow!("load PoW policy at boot: {e}"))?;
        Self::spawn_with_initial(path, initial)
    }

    /// Alternative constructor : start from an explicit policy
    /// (bypassing the disk read). Used by the tests to pin a
    /// starting state without race conditions vs the watcher
    /// thread.
    pub fn spawn_with_initial(path: PathBuf, initial: RelayPowPolicy) -> anyhow::Result<Self> {
        use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

        let inner = Arc::new(RwLock::new(initial));

        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&parent)
            .map_err(|e| anyhow::anyhow!("create PoW policy parent dir: {e}"))?;

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)
            .map_err(|e| anyhow::anyhow!("spawn notify watcher: {e}"))?;
        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .map_err(|e| anyhow::anyhow!("watch PoW policy parent dir: {e}"))?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let join = std::thread::Builder::new()
            .name("sbfb-pow-policy-watch".into())
            .spawn(move || {
                while let Ok(evt) = rx.recv() {
                    match evt {
                        Ok(event) => {
                            // Filter : notify reports every sibling
                            // under the watched dir ; we only care
                            // about our own file.
                            if !event.paths.iter().any(|p| p == &path_thread) {
                                continue;
                            }
                            // A manual rm keeps the last known-good
                            // policy rather than regressing to
                            // default. Mirror of the C-4 fix in
                            // TokenRotatorWatcher.
                            if matches!(event.kind, EventKind::Remove(_)) {
                                warn!(
                                    path = %path_thread.display(),
                                    "relay_pow_policy.toml removed — keeping in-memory policy until recreated"
                                );
                                continue;
                            }
                            if !matches!(
                                event.kind,
                                EventKind::Modify(_) | EventKind::Create(_)
                            ) {
                                continue;
                            }
                            // Debounce : editors emit Create+Modify
                            // in close succession and `write_atomic`
                            // (tempfile + rename) reports a
                            // Modify(Name(To)) that beats the
                            // rename's filesystem flush by a hair.
                            std::thread::sleep(Duration::from_millis(50));
                            match load_relay_pow_policy_from(&path_thread) {
                                Ok(fresh) => {
                                    match inner_thread.write() { Ok(mut guard) => {
                                        *guard = fresh;
                                        debug!(
                                            path = %path_thread.display(),
                                            "relay_pow_policy.toml reloaded"
                                        );
                                    } _ => {
                                        warn!(
                                            path = %path_thread.display(),
                                            "policy reload skipped — RwLock poisoned"
                                        );
                                    }}
                                }
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        path = %path_thread.display(),
                                        "relay_pow_policy.toml reload failed — keeping in-memory policy"
                                    );
                                }
                            }
                        }
                        Err(e) => warn!(error = %e, "PoW policy watcher event error"),
                    }
                }
            })
            .map_err(|e| anyhow::anyhow!("spawn PoW policy watcher thread: {e}"))?;

        Ok(Self {
            inner,
            _watcher: watcher,
            _join: Some(join),
        })
    }

    /// Cheap clone of the shared
    /// `Arc<RwLock<RelayPowPolicy>>`. The watcher keeps updating
    /// the inner policy until the `PowPolicyWatcher` is dropped ;
    /// every clone sees every reload.
    pub fn shared(&self) -> Arc<RwLock<RelayPowPolicy>> {
        Arc::clone(&self.inner)
    }

    /// Snapshot of the current policy. Convenient for tests ;
    /// production code should hold the shared [`Arc<RwLock<_>>`]
    /// so a long-lived `send_with_pow` loop always observes the
    /// latest override.
    ///
    /// Graceful degradation on a poisoned lock : the call-sites in
    /// the daemon (`http.rs::wrap_payload_with_pow` and
    /// `runtime.rs::spawn_gossip_subscribe_task` receive loop) all
    /// fall back to the poisoned inner value rather than propagate
    /// a panic. This helper matches that contract so production and
    /// test readers share one behaviour.
    pub fn current(&self) -> RelayPowPolicy {
        match self.inner.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

/// Default fallback watcher state — a detached
/// `Arc<RwLock<RelayPowPolicy>>` seeded with
/// [`DEFAULT_POLICY`], useful for tests and for code paths that
/// need an always-on policy handle (the `POST /publish` handler
/// pre-boot, for example).
pub fn shared_default_policy() -> Arc<RwLock<RelayPowPolicy>> {
    Arc::new(RwLock::new(DEFAULT_POLICY.clone()))
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, Instant};
    use tempfile::tempdir;

    fn wait_for<F: Fn() -> bool>(check: F, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if check() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(25));
        }
        false
    }

    #[test]
    fn spawn_missing_file_uses_default_policy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        let watcher = PowPolicyWatcher::spawn(path).expect("spawn");
        let policy = watcher.current();
        assert_eq!(policy, DEFAULT_POLICY);
    }

    #[test]
    fn spawn_existing_file_loads_override() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        let hex_topic = "a".repeat(64);
        fs::write(
            &path,
            format!(
                r#"
default_difficulty = 12

[topic_overrides]
"{hex_topic}" = 20
"#
            ),
        )
        .unwrap();

        let watcher = PowPolicyWatcher::spawn(path).expect("spawn");
        let policy = watcher.current();
        assert_eq!(policy.default_difficulty, 12);
        assert_eq!(policy.difficulty_for(&[0xAAu8; 32]), 20);
        assert_eq!(policy.difficulty_for(&[0xBBu8; 32]), 12);
    }

    #[test]
    fn spawn_malformed_toml_fails_loud_at_boot() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        fs::write(&path, "this = is = not = valid [[").unwrap();
        let err = match PowPolicyWatcher::spawn(path) {
            Ok(_) => panic!("malformed boot must fail"),
            Err(e) => e,
        };
        assert!(err.to_string().contains("load PoW policy"), "got: {}", err);
    }

    #[test]
    fn reload_on_file_change_picks_up_override() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        // Start with an empty file → default policy.
        fs::write(&path, "default_difficulty = 10").unwrap();
        let watcher = PowPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default_difficulty, 10);

        // Rewrite the file with a higher default.
        let hex_topic = "b".repeat(64);
        fs::write(
            &path,
            format!(
                r#"
default_difficulty = 22

[topic_overrides]
"{hex_topic}" = 25
"#
            ),
        )
        .unwrap();

        let shared = watcher.shared();
        let reloaded = wait_for(
            || {
                let p = shared.read().unwrap();
                p.default_difficulty == 22 && p.difficulty_for(&[0xBBu8; 32]) == 25
            },
            Duration::from_secs(3),
        );
        assert!(reloaded, "watcher never picked up the rewritten file");
    }

    #[test]
    fn malformed_reload_keeps_previous_policy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        fs::write(&path, "default_difficulty = 14").unwrap();
        let watcher = PowPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default_difficulty, 14);

        // Scribble malformed TOML on top.
        fs::write(&path, "this = is = not = valid [[").unwrap();

        // Wait a bit to let the watcher observe the modify event.
        std::thread::sleep(Duration::from_millis(500));

        // Previous policy must still be enforced (14, not default 18).
        assert_eq!(
            watcher.current().default_difficulty,
            14,
            "malformed reload must preserve the last known-good policy"
        );
    }

    #[test]
    fn removal_keeps_previous_policy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        fs::write(&path, "default_difficulty = 9").unwrap();
        let watcher = PowPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default_difficulty, 9);

        fs::remove_file(&path).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        assert_eq!(
            watcher.current().default_difficulty,
            9,
            "deleting the file must keep the last known-good policy in memory"
        );
    }

    #[test]
    fn shared_default_policy_returns_baseline() {
        let shared = shared_default_policy();
        let policy = shared.read().unwrap();
        assert_eq!(*policy, DEFAULT_POLICY);
    }

    #[test]
    fn shared_handle_tracks_reloads() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("relay_pow_policy.toml");
        fs::write(&path, "default_difficulty = 16").unwrap();
        let watcher = PowPolicyWatcher::spawn(path.clone()).expect("spawn");

        let shared = watcher.shared();
        // Cloning gives the same Arc, so a reload visible through
        // `current()` is also visible through `shared.read()`.
        assert_eq!(shared.read().unwrap().default_difficulty, 16);

        fs::write(&path, "default_difficulty = 19").unwrap();
        let updated = wait_for(
            || shared.read().unwrap().default_difficulty == 19,
            Duration::from_secs(3),
        );
        assert!(updated, "shared handle did not see the reload");
    }
}
