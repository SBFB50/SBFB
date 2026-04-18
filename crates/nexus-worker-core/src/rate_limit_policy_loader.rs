// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hot-reload loader for [`RateLimitPolicy`].
//!
//! Sprint 21 Phase A wires the rate-limit primitive into the worker
//! engine (R1 scope-cut worker-engine gate). The primitive itself
//! parses a [`RateLimitPolicy`] from `~/.sbfb/rate_limit_policy.toml`
//! via `serde` + `toml` ; this module adds the piece the engine
//! needs to benefit from per-consumer overrides without restarting :
//! a `notify`-driven watcher that re-reads the TOML on every write
//! and atomically swaps the shared state an operator may edit while
//! the worker is running.
//!
//! Pattern mirrors the S16 `ConsentWatcher` (nexus-worker-core) and
//! the S20 Phase C `PowPolicyWatcher` (nexus-shell-daemon-core) :
//! watch the **parent directory** (not the file), filter events by
//! absolute path, debounce 50 ms to catch write+rename atomic
//! rewrites, and keep the previous in-memory policy on any reload
//! failure. A malformed TOML edit never locks the worker out of its
//! rate-limit gate — it just prints a `warn!` and keeps enforcing
//! the last known-good policy.
//!
//! ## Why parent-dir watch
//!
//! Atomic rewrites (editors, the operator's `mv tmp
//! rate_limit_policy.toml`, and the tempfile + rename idiom) detach
//! a file-level watch because the inode changes. Watching the parent
//! directory and filtering by path is the portable fix used
//! throughout the workspace.
//!
//! ## Error semantics
//!
//! - **Missing file at spawn** → start on [`RateLimitPolicy::default`]
//!   (60 req/min + burst x2, no overrides). The worker always has a
//!   live policy, even if the operator never creates the file.
//! - **Malformed TOML at spawn** → surface the error upstream. A
//!   fresh boot with a broken config is a loud problem the operator
//!   should fix before the worker opens the engine gate.
//! - **Malformed TOML at reload** → `warn!` with the parse error and
//!   keep the previous in-memory policy. A mid-lifetime typo must
//!   not lock tasks out.
//! - **File deletion at reload** → `warn!` and keep the previous
//!   policy, same rationale as the `TokenRotator` C-4 fix (S18) and
//!   `PowPolicyWatcher::removal_keeps_previous_policy` (S20).

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tracing::{debug, warn};

use crate::rate_limit::RateLimitPolicy;

/// Load a [`RateLimitPolicy`] from a TOML file on disk.
///
/// `Ok(default)` when the file does not exist — the worker always
/// has a live policy even without an operator override. `Err` on
/// any other I/O error or on TOML parse failure.
pub fn load_rate_limit_policy_from(path: &Path) -> anyhow::Result<RateLimitPolicy> {
    match std::fs::read_to_string(path) {
        Ok(src) => {
            let parsed: RateLimitPolicy = toml::from_str(&src).map_err(|e| {
                anyhow::anyhow!("parse rate_limit_policy.toml at {}: {e}", path.display())
            })?;
            Ok(parsed)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(RateLimitPolicy::default()),
        Err(e) => Err(anyhow::anyhow!(
            "read rate_limit_policy.toml at {}: {e}",
            path.display()
        )),
    }
}

/// Live, reload-on-change handle to a [`RateLimitPolicy`]. Held by
/// the worker engine for the whole process lifetime — dropping the
/// watcher shuts the `notify` observer down and closes the
/// background reload thread.
///
/// Multiple callers (the engine task loop, diagnostic commands, the
/// test harness) hold cheap [`Arc<RwLock<RateLimitPolicy>>`] clones
/// obtained via [`RateLimitPolicyWatcher::shared`] and read the
/// inner policy on every task admission check. An operator-issued
/// rotation (e.g. `vi ~/.sbfb/rate_limit_policy.toml`) propagates
/// without a worker restart.
pub struct RateLimitPolicyWatcher {
    inner: Arc<RwLock<RateLimitPolicy>>,
    /// Held for `Drop` : the watcher background thread stops when
    /// the notify channel closes.
    _watcher: notify::RecommendedWatcher,
    /// Joined on `Drop` for clean teardown ; tests rely on the
    /// watcher thread exiting before the tempdir disappears.
    _join: Option<std::thread::JoinHandle<()>>,
}

impl RateLimitPolicyWatcher {
    /// Build a watcher rooted at `path`. The initial policy is
    /// loaded synchronously via [`load_rate_limit_policy_from`] : a
    /// missing file resolves to [`RateLimitPolicy::default`], a
    /// malformed TOML surfaces as an error so the operator sees the
    /// problem at boot.
    pub fn spawn(path: PathBuf) -> anyhow::Result<Self> {
        let initial = load_rate_limit_policy_from(&path)
            .map_err(|e| anyhow::anyhow!("load rate-limit policy at boot: {e}"))?;
        Self::spawn_with_initial(path, initial)
    }

    /// Alternative constructor : start from an explicit policy
    /// (bypassing the disk read). Used by the tests to pin a
    /// starting state without race conditions vs the watcher
    /// thread.
    pub fn spawn_with_initial(path: PathBuf, initial: RateLimitPolicy) -> anyhow::Result<Self> {
        use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

        let inner = Arc::new(RwLock::new(initial));

        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&parent)
            .map_err(|e| anyhow::anyhow!("create rate-limit policy parent dir: {e}"))?;

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)
            .map_err(|e| anyhow::anyhow!("spawn notify watcher: {e}"))?;
        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .map_err(|e| anyhow::anyhow!("watch rate-limit policy parent dir: {e}"))?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let join = std::thread::Builder::new()
            .name("sbfb-rate-limit-policy-watch".into())
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
                            // A manual `rm` keeps the last known-
                            // good policy rather than regressing
                            // to default. Mirror of the C-4 fix in
                            // TokenRotatorWatcher (S18) + the
                            // `removal_keeps_previous_policy` test
                            // in PowPolicyWatcher (S20 Phase C).
                            if matches!(event.kind, EventKind::Remove(_)) {
                                warn!(
                                    path = %path_thread.display(),
                                    "rate_limit_policy.toml removed — keeping in-memory policy until recreated"
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
                            match load_rate_limit_policy_from(&path_thread) {
                                Ok(fresh) => {
                                    if let Ok(mut guard) = inner_thread.write() {
                                        *guard = fresh;
                                        debug!(
                                            path = %path_thread.display(),
                                            "rate_limit_policy.toml reloaded"
                                        );
                                    } else {
                                        warn!(
                                            path = %path_thread.display(),
                                            "policy reload skipped — RwLock poisoned"
                                        );
                                    }
                                }
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        path = %path_thread.display(),
                                        "rate_limit_policy.toml reload failed — keeping in-memory policy"
                                    );
                                }
                            }
                        }
                        Err(e) => warn!(error = %e, "rate-limit policy watcher event error"),
                    }
                }
            })
            .map_err(|e| anyhow::anyhow!("spawn rate-limit policy watcher thread: {e}"))?;

        Ok(Self {
            inner,
            _watcher: watcher,
            _join: Some(join),
        })
    }

    /// Cheap clone of the shared
    /// `Arc<RwLock<RateLimitPolicy>>`. The watcher keeps updating
    /// the inner policy until the `RateLimitPolicyWatcher` is
    /// dropped ; every clone sees every reload.
    pub fn shared(&self) -> Arc<RwLock<RateLimitPolicy>> {
        Arc::clone(&self.inner)
    }

    /// Snapshot of the current policy. Convenient for tests and
    /// diagnostic endpoints ; production code that wants per-request
    /// reload visibility should hold the shared [`Arc<RwLock<_>>`]
    /// instead.
    ///
    /// Graceful degradation on a poisoned lock : we fall back to
    /// the poisoned inner value rather than panic — the rate-limit
    /// gate must stay live even after a writer panic.
    pub fn current(&self) -> RateLimitPolicy {
        match self.inner.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

/// Detached default watcher state — a fresh
/// `Arc<RwLock<RateLimitPolicy>>` seeded with
/// [`RateLimitPolicy::default`]. Useful for tests and for code
/// paths that need an always-on policy handle before the operator
/// creates a file.
pub fn shared_default_policy() -> Arc<RwLock<RateLimitPolicy>> {
    Arc::new(RwLock::new(RateLimitPolicy::default()))
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
        let path = dir.path().join("rate_limit_policy.toml");
        let watcher = RateLimitPolicyWatcher::spawn(path).expect("spawn");
        let policy = watcher.current();
        assert_eq!(policy, RateLimitPolicy::default());
    }

    #[test]
    fn spawn_existing_file_loads_override() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rate_limit_policy.toml");
        fs::write(
            &path,
            r#"
[default]
per_min = 42
burst_multiplier = 1.5

[[overrides.consumer]]
pubkey_hex = "abc123"
per_min = 500
burst_multiplier = 3.0
"#,
        )
        .unwrap();

        let watcher = RateLimitPolicyWatcher::spawn(path).expect("spawn");
        let policy = watcher.current();
        assert_eq!(policy.default.per_min, 42);
        assert!((policy.default.burst_multiplier - 1.5).abs() < 1e-9);
        assert_eq!(policy.overrides.consumer.len(), 1);
        assert_eq!(policy.overrides.consumer[0].pubkey_hex, "abc123");
        assert_eq!(policy.overrides.consumer[0].per_min, 500);
    }

    #[test]
    fn spawn_malformed_toml_fails_loud_at_boot() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rate_limit_policy.toml");
        fs::write(&path, "this = is = not = valid [[").unwrap();
        let err = match RateLimitPolicyWatcher::spawn(path) {
            Ok(_) => panic!("malformed boot must fail"),
            Err(e) => e,
        };
        assert!(
            err.to_string().contains("load rate-limit policy"),
            "got: {}",
            err
        );
    }

    #[test]
    fn policy_hot_reload_live() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rate_limit_policy.toml");
        fs::write(
            &path,
            r#"
[default]
per_min = 10
burst_multiplier = 2.0
"#,
        )
        .unwrap();
        let watcher = RateLimitPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default.per_min, 10);

        // Rewrite the file with a higher default.
        fs::write(
            &path,
            r#"
[default]
per_min = 77
burst_multiplier = 1.0

[[overrides.consumer]]
pubkey_hex = "whitelisted"
per_min = 200
"#,
        )
        .unwrap();

        let shared = watcher.shared();
        let reloaded = wait_for(
            || {
                let p = shared.read().unwrap();
                p.default.per_min == 77 && p.overrides.consumer.len() == 1
            },
            Duration::from_secs(3),
        );
        assert!(reloaded, "watcher never picked up the rewritten file");
    }

    #[test]
    fn malformed_reload_keeps_previous_policy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rate_limit_policy.toml");
        fs::write(
            &path,
            r#"
[default]
per_min = 14
burst_multiplier = 2.0
"#,
        )
        .unwrap();
        let watcher = RateLimitPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default.per_min, 14);

        // Scribble malformed TOML on top.
        fs::write(&path, "this = is = not = valid [[").unwrap();

        // Wait a bit to let the watcher observe the modify event.
        std::thread::sleep(Duration::from_millis(500));

        // Previous policy must still be enforced.
        assert_eq!(
            watcher.current().default.per_min,
            14,
            "malformed reload must preserve the last known-good policy"
        );
    }

    #[test]
    fn removal_keeps_previous_policy() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("rate_limit_policy.toml");
        fs::write(
            &path,
            r#"
[default]
per_min = 9
burst_multiplier = 2.0
"#,
        )
        .unwrap();
        let watcher = RateLimitPolicyWatcher::spawn(path.clone()).expect("spawn");
        assert_eq!(watcher.current().default.per_min, 9);

        fs::remove_file(&path).unwrap();
        std::thread::sleep(Duration::from_millis(300));

        assert_eq!(
            watcher.current().default.per_min,
            9,
            "deleting the file must keep the last known-good policy in memory"
        );
    }

    #[test]
    fn shared_default_policy_returns_baseline() {
        let shared = shared_default_policy();
        let policy = shared.read().unwrap();
        assert_eq!(*policy, RateLimitPolicy::default());
    }
}
