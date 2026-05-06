// SPDX-License-Identifier: AGPL-3.0-or-later
//! Pre-`v1.0` bootstrap allowlist for Couche 1 age admission.
//!
//! Sprint 22 Phase C — P0-G1-1 ceremony. A peer-attested age
//! gate (cf. [`nexus_core_rs::attestations::AgeWitness`]) requires
//! at least one already-aged peer in the mesh to vouch for new
//! entrants. The first SBFB node on the network has no such peer
//! by construction — a chicken-and-egg that pure peer-attestation
//! cannot resolve.
//!
//! Resolution : a tiny operator-maintained TOML file names ≤ 20
//! pre-v1.0 seed nodes that are admitted via the "self-witness"
//! shortcut in [`nexus_core_rs::gossip::evaluate_age_admission`].
//! The allowlist is **time-bounded** — every entry carries an
//! `expires_at_tag = "v1.0"` field that the daemon enforces once
//! the monorepo tip reaches the first `v1.0` tag. After `v1.0`
//! the allowlist becomes a no-op and normal peer attestation is
//! the only path.
//!
//! ## Hot-reload semantics
//!
//! Pattern mirrors [`crate::pow_policy_loader::PowPolicyWatcher`]
//! (S20 Phase C) : a `notify` watcher re-reads the TOML on every
//! write, debounces 50 ms to catch atomic rewrites, and keeps the
//! last-known-good list on parse errors or deletions. The
//! operator never locks the daemon out by fat-fingering a
//! comma.
//!
//! ## Config shape
//!
//! ```toml
//! [[bootstrap]]
//! node_id_hex = "a3b1c2d3..." # 64-char lowercase hex
//! added_at = "2026-04-19"
//! reason = "initial publisher seed"
//! expires_at_tag = "v1.0"
//! ```
//!
//! Cap : [`MAX_BOOTSTRAP_ENTRIES`] = 20. Beyond that the loader
//! rejects the file loud — an accidentally-large allowlist would
//! dilute the Sybil gate.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use nexus_core_rs::AgeAdmissionPolicy;
use nexus_core_rs::crypto::PUBLIC_KEY_LENGTH;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Name of the allowlist file inside `~/.sbfb/`.
pub const BOOTSTRAP_ALLOWLIST_FILE_NAME: &str = "bootstrap_allowlist.toml";

/// Environment variable pointing at an absolute path to a custom
/// bootstrap allowlist. Takes precedence over the
/// `$SBFB_HOME/bootstrap_allowlist.toml` default. Mirrors the
/// `SBFB_POW_POLICY_PATH` pattern.
pub const CUSTOM_BOOTSTRAP_ALLOWLIST_ENV: &str = "SBFB_BOOTSTRAP_ALLOWLIST_PATH";

/// Hard cap on the number of bootstrap entries. An allowlist
/// larger than this dilutes the Sybil-resistance gate and is
/// almost certainly a mis-edit.
pub const MAX_BOOTSTRAP_ENTRIES: usize = 20;

/// Tag that marks the go-live boundary. Entries carrying this
/// string as their `expires_at_tag` become inert once the running
/// daemon reports a build tag greater than or equal to `v1.0`.
/// Pre-launch we never check this — entries are always active.
pub const DEFAULT_EXPIRES_AT_TAG: &str = "v1.0";

/// On-disk TOML shape. Separate from [`BootstrapAllowlist`]
/// because the TOML layer sees hex strings while the runtime
/// works with raw 32-byte keys.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BootstrapAllowlistFile {
    /// One entry per pre-v1.0 seed node.
    #[serde(default)]
    pub bootstrap: Vec<BootstrapEntry>,
}

/// A single pre-v1.0 seed-node entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BootstrapEntry {
    /// 64-char lowercase hex of the seed node's Ed25519 public
    /// key.
    pub node_id_hex: String,

    /// Operator-provided ISO-8601 date recording when the entry
    /// was added. Display only — the admission logic does not
    /// consult this field.
    pub added_at: String,

    /// Operator-provided rationale. Display only.
    pub reason: String,

    /// Tag string at which the entry expires. MUST equal
    /// [`DEFAULT_EXPIRES_AT_TAG`] for now — any other value is a
    /// future-proofing error that the loader rejects to surface
    /// the mistake early.
    pub expires_at_tag: String,
}

/// Hydrated in-memory allowlist. The `BTreeSet` holds raw 32-byte
/// `node_id` bytes for O(log n) membership lookup without
/// per-query hex decoding.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BootstrapAllowlist {
    /// Raw Ed25519 public keys of the bootstrap seed nodes.
    nodes: BTreeSet<[u8; PUBLIC_KEY_LENGTH]>,
}

impl BootstrapAllowlist {
    /// Empty allowlist. Used as the fallback when no file is
    /// present.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Convert a parsed [`BootstrapAllowlistFile`] to a hydrated
    /// [`BootstrapAllowlist`]. Validates :
    ///
    /// - `bootstrap.len() <= MAX_BOOTSTRAP_ENTRIES`,
    /// - each `node_id_hex` is 64 chars + valid lowercase hex,
    /// - each `expires_at_tag == DEFAULT_EXPIRES_AT_TAG`,
    /// - `added_at` and `reason` are non-empty (display sanity).
    ///
    /// Loud failure is kinder than silent truncation : a
    /// malformed allowlist that "looks fine" could silently let
    /// an attacker's node through the Sybil gate.
    pub fn from_file(file: BootstrapAllowlistFile) -> Result<Self, BootstrapAllowlistError> {
        if file.bootstrap.len() > MAX_BOOTSTRAP_ENTRIES {
            return Err(BootstrapAllowlistError::TooManyEntries(
                file.bootstrap.len(),
            ));
        }
        let mut nodes = BTreeSet::new();
        for (idx, entry) in file.bootstrap.into_iter().enumerate() {
            if entry.expires_at_tag != DEFAULT_EXPIRES_AT_TAG {
                return Err(BootstrapAllowlistError::BadExpiresAt {
                    index: idx,
                    got: entry.expires_at_tag,
                });
            }
            if entry.node_id_hex.len() != 64 {
                return Err(BootstrapAllowlistError::BadNodeIdLength {
                    index: idx,
                    got: entry.node_id_hex.len(),
                });
            }
            if !entry
                .node_id_hex
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
            {
                return Err(BootstrapAllowlistError::BadNodeIdEncoding { index: idx });
            }
            if entry.added_at.trim().is_empty() {
                return Err(BootstrapAllowlistError::EmptyField {
                    index: idx,
                    field: "added_at",
                });
            }
            if entry.reason.trim().is_empty() {
                return Err(BootstrapAllowlistError::EmptyField {
                    index: idx,
                    field: "reason",
                });
            }
            let mut node_id = [0u8; PUBLIC_KEY_LENGTH];
            hex::decode_to_slice(&entry.node_id_hex, &mut node_id).map_err(|e| {
                BootstrapAllowlistError::BadNodeIdHex {
                    index: idx,
                    detail: e.to_string(),
                }
            })?;
            nodes.insert(node_id);
        }
        Ok(BootstrapAllowlist { nodes })
    }

    /// Number of entries in the allowlist.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// True iff the allowlist has no entries.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Return `true` if `node_id` is a bootstrap seed.
    pub fn contains(&self, node_id: &[u8; PUBLIC_KEY_LENGTH]) -> bool {
        self.nodes.contains(node_id)
    }
}

/// Errors returned by [`BootstrapAllowlist::from_file`] and the
/// loader helpers. Kept distinct so callers can surface a precise
/// diagnostic to the operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapAllowlistError {
    /// The file declared more than [`MAX_BOOTSTRAP_ENTRIES`]
    /// entries.
    TooManyEntries(usize),
    /// An entry's `expires_at_tag` is not
    /// [`DEFAULT_EXPIRES_AT_TAG`].
    BadExpiresAt {
        /// Zero-based index of the offending entry.
        index: usize,
        /// Verbatim value read from the file.
        got: String,
    },
    /// `node_id_hex` is not 64 characters long.
    BadNodeIdLength {
        /// Zero-based index of the offending entry.
        index: usize,
        /// Length observed.
        got: usize,
    },
    /// `node_id_hex` contains non-lowercase-hex characters.
    BadNodeIdEncoding {
        /// Zero-based index of the offending entry.
        index: usize,
    },
    /// `node_id_hex` failed hex decoding despite passing the
    /// character check.
    BadNodeIdHex {
        /// Zero-based index of the offending entry.
        index: usize,
        /// Low-level decode error.
        detail: String,
    },
    /// `added_at` or `reason` is empty.
    EmptyField {
        /// Zero-based index of the offending entry.
        index: usize,
        /// Name of the empty field.
        field: &'static str,
    },
    /// TOML parse error.
    ParseError(String),
    /// Filesystem read error.
    IoError(String),
}

impl std::fmt::Display for BootstrapAllowlistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootstrapAllowlistError::TooManyEntries(n) => {
                write!(
                    f,
                    "bootstrap allowlist has {n} entries, exceeds MAX_BOOTSTRAP_ENTRIES={MAX_BOOTSTRAP_ENTRIES}"
                )
            }
            BootstrapAllowlistError::BadExpiresAt { index, got } => write!(
                f,
                "entry #{index}: expires_at_tag={got:?}, expected {DEFAULT_EXPIRES_AT_TAG:?}"
            ),
            BootstrapAllowlistError::BadNodeIdLength { index, got } => {
                write!(
                    f,
                    "entry #{index}: node_id_hex has {got} chars, expected 64"
                )
            }
            BootstrapAllowlistError::BadNodeIdEncoding { index } => {
                write!(f, "entry #{index}: node_id_hex must be lowercase hex")
            }
            BootstrapAllowlistError::BadNodeIdHex { index, detail } => {
                write!(f, "entry #{index}: node_id_hex decode failed: {detail}")
            }
            BootstrapAllowlistError::EmptyField { index, field } => {
                write!(f, "entry #{index}: {field} must be non-empty")
            }
            BootstrapAllowlistError::ParseError(msg) => write!(f, "TOML parse error: {msg}"),
            BootstrapAllowlistError::IoError(msg) => write!(f, "read error: {msg}"),
        }
    }
}

impl std::error::Error for BootstrapAllowlistError {}

/// Resolve the active allowlist path.
/// `SBFB_BOOTSTRAP_ALLOWLIST_PATH` env var wins ; else
/// `$SBFB_HOME/bootstrap_allowlist.toml`.
pub fn bootstrap_allowlist_file_path() -> Option<PathBuf> {
    if let Ok(path) = env::var(CUSTOM_BOOTSTRAP_ALLOWLIST_ENV) {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    sbfb_home().map(|h| h.join(BOOTSTRAP_ALLOWLIST_FILE_NAME))
}

fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var("SBFB_HOME") {
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

/// Load the active allowlist.
///
/// - Missing file → empty allowlist.
/// - Parse error → loud `Err`.
/// - Validation error → loud `Err`.
pub fn load_bootstrap_allowlist() -> Result<BootstrapAllowlist, BootstrapAllowlistError> {
    let Some(path) = bootstrap_allowlist_file_path() else {
        return Ok(BootstrapAllowlist::empty());
    };
    load_bootstrap_allowlist_from(&path)
}

/// Load from an explicit path. Missing file → empty allowlist.
pub fn load_bootstrap_allowlist_from(
    path: &Path,
) -> Result<BootstrapAllowlist, BootstrapAllowlistError> {
    if !path.is_file() {
        return Ok(BootstrapAllowlist::empty());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| BootstrapAllowlistError::IoError(format!("{}: {e}", path.display())))?;
    let file: BootstrapAllowlistFile = toml::from_str(&raw)
        .map_err(|e| BootstrapAllowlistError::ParseError(format!("{}: {e}", path.display())))?;
    BootstrapAllowlist::from_file(file)
}

/// Hot-reload watcher. Pattern mirrors
/// [`crate::pow_policy_loader::PowPolicyWatcher`].
///
/// The watcher keeps an [`Arc<RwLock<BootstrapAllowlist>>`] shared
/// with the daemon runtime ; every reload swaps the inner value.
/// A malformed TOML at reload time keeps the previous list (same
/// fail-closed semantics as PoW policy).
pub struct BootstrapAllowlistWatcher {
    inner: Arc<RwLock<BootstrapAllowlist>>,
    _watcher: notify::RecommendedWatcher,
    _join: Option<std::thread::JoinHandle<()>>,
}

impl BootstrapAllowlistWatcher {
    /// Build a watcher rooted at `path`. The initial allowlist is
    /// loaded synchronously — a missing file yields an empty
    /// allowlist, a malformed TOML surfaces a loud error at boot.
    pub fn spawn(path: PathBuf) -> anyhow::Result<Self> {
        let initial = load_bootstrap_allowlist_from(&path)
            .map_err(|e| anyhow::anyhow!("load bootstrap allowlist at boot: {e}"))?;
        Self::spawn_with_initial(path, initial)
    }

    /// Alternative constructor : seed from an explicit allowlist
    /// (bypasses disk read). Used by tests to pin a starting
    /// state without race conditions vs the watcher thread.
    pub fn spawn_with_initial(path: PathBuf, initial: BootstrapAllowlist) -> anyhow::Result<Self> {
        use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

        let inner = Arc::new(RwLock::new(initial));
        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        std::fs::create_dir_all(&parent)
            .map_err(|e| anyhow::anyhow!("create bootstrap allowlist parent dir: {e}"))?;

        let (tx, rx) = std::sync::mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)
            .map_err(|e| anyhow::anyhow!("spawn notify watcher: {e}"))?;
        watcher
            .watch(&parent, RecursiveMode::NonRecursive)
            .map_err(|e| anyhow::anyhow!("watch bootstrap allowlist parent dir: {e}"))?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let join = std::thread::Builder::new()
            .name("sbfb-bootstrap-allowlist-watch".into())
            .spawn(move || {
                while let Ok(evt) = rx.recv() {
                    match evt {
                        Ok(event) => {
                            if !event.paths.iter().any(|p| p == &path_thread) {
                                continue;
                            }
                            if matches!(event.kind, EventKind::Remove(_)) {
                                warn!(
                                    path = %path_thread.display(),
                                    "bootstrap_allowlist.toml removed — keeping in-memory list until recreated"
                                );
                                continue;
                            }
                            if !matches!(
                                event.kind,
                                EventKind::Modify(_) | EventKind::Create(_)
                            ) {
                                continue;
                            }
                            std::thread::sleep(Duration::from_millis(50));
                            match load_bootstrap_allowlist_from(&path_thread) {
                                Ok(fresh) => {
                                    match inner_thread.write() { Ok(mut guard) => {
                                        *guard = fresh;
                                        debug!(
                                            path = %path_thread.display(),
                                            "bootstrap_allowlist.toml reloaded"
                                        );
                                    } _ => {
                                        warn!(
                                            path = %path_thread.display(),
                                            "allowlist reload skipped — RwLock poisoned"
                                        );
                                    }}
                                }
                                Err(e) => warn!(
                                    error = %e,
                                    path = %path_thread.display(),
                                    "bootstrap_allowlist.toml reload failed — keeping in-memory list"
                                ),
                            }
                        }
                        Err(e) => warn!(error = %e, "bootstrap allowlist watcher event error"),
                    }
                }
            })
            .map_err(|e| anyhow::anyhow!("spawn bootstrap allowlist watcher thread: {e}"))?;

        Ok(Self {
            inner,
            _watcher: watcher,
            _join: Some(join),
        })
    }

    /// Cheap clone of the shared `Arc<RwLock<_>>`. Updates
    /// propagate to every holder.
    pub fn shared(&self) -> Arc<RwLock<BootstrapAllowlist>> {
        Arc::clone(&self.inner)
    }

    /// Snapshot of the current allowlist. Graceful degradation on
    /// a poisoned lock — returns the inner value rather than
    /// panicking, matching the `PowPolicyWatcher::current`
    /// contract.
    pub fn current(&self) -> BootstrapAllowlist {
        match self.inner.read() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

/// An [`AgeAdmissionPolicy`] implementation composed of a
/// bootstrap allowlist + a mesh-state oracle closure.
///
/// The daemon runtime wires this by wrapping its own
/// `first_seen_ts` table (or any equivalent mesh observation
/// store) behind the closure. Tests stub the closure directly.
pub struct AllowlistPolicy<F>
where
    F: Fn(&[u8; PUBLIC_KEY_LENGTH], i64) -> Option<i64> + Send + Sync,
{
    /// Shared allowlist handle — cloned from a
    /// [`BootstrapAllowlistWatcher`] at construction time.
    allowlist: Arc<RwLock<BootstrapAllowlist>>,
    /// Mesh-state oracle returning the age in days of a witness
    /// at a given timestamp, or `None` if the witness is
    /// unknown.
    witness_age: F,
}

impl<F> AllowlistPolicy<F>
where
    F: Fn(&[u8; PUBLIC_KEY_LENGTH], i64) -> Option<i64> + Send + Sync,
{
    /// Build a policy from an allowlist handle + oracle closure.
    pub fn new(allowlist: Arc<RwLock<BootstrapAllowlist>>, witness_age: F) -> Self {
        Self {
            allowlist,
            witness_age,
        }
    }
}

impl<F> AgeAdmissionPolicy for AllowlistPolicy<F>
where
    F: Fn(&[u8; PUBLIC_KEY_LENGTH], i64) -> Option<i64> + Send + Sync,
{
    fn is_bootstrap_node(&self, node_id: &[u8; PUBLIC_KEY_LENGTH]) -> bool {
        match self.allowlist.read() {
            Ok(guard) => guard.contains(node_id),
            Err(poisoned) => poisoned.into_inner().contains(node_id),
        }
    }

    fn witness_age_days(
        &self,
        witness_pubkey: &[u8; PUBLIC_KEY_LENGTH],
        now_ts: i64,
    ) -> Option<i64> {
        (self.witness_age)(witness_pubkey, now_ts)
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use std::time::Instant;
    use tempfile::tempdir;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

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

    fn sample_toml(node_hex: &str) -> String {
        format!(
            r#"
[[bootstrap]]
node_id_hex = "{node_hex}"
added_at = "2026-04-19"
reason = "initial publisher seed"
expires_at_tag = "v1.0"
"#
        )
    }

    #[test]
    fn load_toml_schema_parses_valid_entry() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        let hex_node = "a".repeat(64);
        fs::write(&path, sample_toml(&hex_node)).unwrap();
        let list = load_bootstrap_allowlist_from(&path).expect("parse ok");
        assert_eq!(list.len(), 1);
        let node_id = [0xAAu8; PUBLIC_KEY_LENGTH];
        assert!(list.contains(&node_id));
    }

    #[test]
    fn is_bootstrap_node_true_for_listed_false_for_other() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        let hex_node = "b".repeat(64);
        fs::write(&path, sample_toml(&hex_node)).unwrap();
        let list = load_bootstrap_allowlist_from(&path).unwrap();
        let listed = [0xBBu8; PUBLIC_KEY_LENGTH];
        let unknown = [0x11u8; PUBLIC_KEY_LENGTH];
        assert!(list.contains(&listed));
        assert!(!list.contains(&unknown));
    }

    #[test]
    fn rejects_bad_expires_at_tag() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        let hex_node = "c".repeat(64);
        let content = format!(
            r#"
[[bootstrap]]
node_id_hex = "{hex_node}"
added_at = "2026-04-19"
reason = "initial publisher seed"
expires_at_tag = "never"
"#
        );
        fs::write(&path, content).unwrap();
        let err = load_bootstrap_allowlist_from(&path).expect_err("bad expires must reject");
        assert!(matches!(err, BootstrapAllowlistError::BadExpiresAt { .. }));
    }

    #[test]
    fn rejects_uppercase_hex() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        // 64 valid hex chars but uppercase.
        let hex_node = "A".repeat(64);
        fs::write(&path, sample_toml(&hex_node)).unwrap();
        let err = load_bootstrap_allowlist_from(&path).expect_err("uppercase must reject");
        assert!(matches!(
            err,
            BootstrapAllowlistError::BadNodeIdEncoding { .. }
        ));
    }

    #[test]
    fn rejects_oversized_allowlist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        let mut content = String::new();
        for i in 0..=MAX_BOOTSTRAP_ENTRIES {
            let byte = (i % 16) as u8;
            let hex_char = format!("{byte:x}");
            let hex_node = hex_char.repeat(64);
            content.push_str(&format!(
                r#"
[[bootstrap]]
node_id_hex = "{hex_node}"
added_at = "2026-04-19"
reason = "seed {i}"
expires_at_tag = "v1.0"
"#
            ));
        }
        fs::write(&path, content).unwrap();
        let err = load_bootstrap_allowlist_from(&path).expect_err("oversize must reject");
        assert!(matches!(err, BootstrapAllowlistError::TooManyEntries(_)));
    }

    #[test]
    fn missing_file_returns_empty_allowlist() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("does_not_exist.toml");
        let list = load_bootstrap_allowlist_from(&path).expect("missing → empty");
        assert!(list.is_empty());
    }

    #[test]
    fn watcher_picks_up_file_rewrite() {
        let _g = ENV_GUARD.lock().unwrap();
        let dir = tempdir().unwrap();
        let path = dir.path().join("bootstrap_allowlist.toml");
        // Start empty (no file).
        let watcher = BootstrapAllowlistWatcher::spawn(path.clone()).unwrap();
        assert!(watcher.current().is_empty());

        // Write a fresh allowlist and wait for reload.
        let hex_node = "d".repeat(64);
        fs::write(&path, sample_toml(&hex_node)).unwrap();

        let shared = watcher.shared();
        let got = wait_for(
            || !shared.read().unwrap().is_empty(),
            Duration::from_secs(3),
        );
        assert!(got, "watcher never picked up the new allowlist");
        let node_id = [0xDDu8; PUBLIC_KEY_LENGTH];
        assert!(shared.read().unwrap().contains(&node_id));
    }

    #[test]
    fn allowlist_policy_bootstraps_listed_nodes() {
        let hex_node = "e".repeat(64);
        let file_shape = BootstrapAllowlistFile {
            bootstrap: vec![BootstrapEntry {
                node_id_hex: hex_node.clone(),
                added_at: "2026-04-19".into(),
                reason: "test seed".into(),
                expires_at_tag: "v1.0".into(),
            }],
        };
        let list = BootstrapAllowlist::from_file(file_shape).unwrap();
        let shared = Arc::new(RwLock::new(list));
        let policy = AllowlistPolicy::new(shared, |_, _| Some(45));
        let listed = [0xEEu8; PUBLIC_KEY_LENGTH];
        let unknown = [0x11u8; PUBLIC_KEY_LENGTH];
        assert!(policy.is_bootstrap_node(&listed));
        assert!(!policy.is_bootstrap_node(&unknown));
        // Oracle is returned as-is for any witness pubkey.
        assert_eq!(policy.witness_age_days(&unknown, 1_700_000_000), Some(45));
    }
}
