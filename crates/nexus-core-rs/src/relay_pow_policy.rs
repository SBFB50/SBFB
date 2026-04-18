// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-topic Hashcash difficulty policy loader.
//!
//! Sprint 19 Phase B : every gossip subscribe solves a Hashcash
//! puzzle whose difficulty is driven by a policy file. The policy
//! ships with a conservative default ([`DEFAULT_POLICY`]) and can
//! be overridden per-topic via `~/.sbfb/relay_pow_policy.toml`.
//!
//! The layered source rules mirror [`crate::relay_config`] :
//!
//! 1. `SBFB_POW_POLICY_PATH` env var : absolute path to a TOML
//!    file, takes precedence.
//! 2. `$SBFB_HOME/relay_pow_policy.toml` (default
//!    `~/.sbfb/relay_pow_policy.toml`).
//! 3. Fallback to [`DEFAULT_POLICY`] : default difficulty
//!    [`crate::pow::DEFAULT_DIFFICULTY_BITS`], no per-topic
//!    overrides.
//!
//! ## TOML schema
//!
//! ```toml
//! # Apply to any topic not explicitly listed below.
//! default_difficulty = 18
//!
//! # Per-topic override. The key is the 64-char lowercase hex
//! # representation of the 32-byte topic id.
//! [topic_overrides]
//! "a1b2c3...deadbeef" = 20  # higher difficulty for hot topics
//! "cafebabe...feedface" = 16  # lower for dev/test channels
//! ```
//!
//! ## Forward-compat paths
//!
//! - **S22 kudos-weighted admission** : the receiver verify path
//!   will add a `kudos_threshold` field alongside
//!   `default_difficulty`, plus `kudos_overrides` next to
//!   `topic_overrides`. Pre-launch policy lets us redefine the
//!   schema in place for v1 ; post-v1.0 we bump a version tag
//!   and ship a tolerant decoder.
//! - **S26 PQC migration** : the policy file is untouched — the
//!   difficulty is orthogonal to the pubkey cipher. Only the
//!   pubkey field in [`crate::pow::HashcashChallenge`] changes.

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{NexusError, Result};
use crate::pow::{DEFAULT_DIFFICULTY_BITS, MAX_DIFFICULTY_BITS};

/// Environment variable pointing at an absolute path to a custom
/// relay PoW policy file. Takes precedence over the
/// `$SBFB_HOME/relay_pow_policy.toml` default.
pub const CUSTOM_POW_POLICY_ENV: &str = "SBFB_POW_POLICY_PATH";

/// Name of the policy file inside `~/.sbfb/`.
pub const RELAY_POW_POLICY_FILE_NAME: &str = "relay_pow_policy.toml";

/// Baseline policy used when no file is present : default
/// difficulty 2^18, no per-topic overrides.
pub const DEFAULT_POLICY: RelayPowPolicy = RelayPowPolicy {
    default_difficulty: DEFAULT_DIFFICULTY_BITS,
    topic_overrides: BTreeMap::new(),
};

/// On-disk TOML shape. Separate from [`RelayPowPolicy`] because
/// the TOML crate cannot deserialize directly into a struct with a
/// non-string key map in `topic_overrides` without a helper.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RelayPowPolicyFile {
    /// Difficulty (leading zero bits) applied to topics not
    /// listed in `topic_overrides`. Absent field → use
    /// [`DEFAULT_DIFFICULTY_BITS`].
    #[serde(default = "default_difficulty_default")]
    pub default_difficulty: u32,

    /// Per-topic difficulty overrides, keyed by the 64-char
    /// lowercase hex of the 32-byte topic id.
    ///
    /// Why TOML : human-edit-friendly, comment-friendly, and the
    /// `toml` crate is already a workspace dep. JSON was
    /// considered but operators tend to hand-edit the policy and
    /// comments make the intent of a per-topic override obvious.
    #[serde(default)]
    pub topic_overrides: BTreeMap<String, u32>,
}

/// Hydrated in-memory policy. Converted from
/// [`RelayPowPolicyFile`] at load time so lookup is a
/// `BTreeMap<[u8; 32], u32>` without per-query hex decoding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayPowPolicy {
    /// Applied to any topic id not present in [`Self::topic_overrides`].
    pub default_difficulty: u32,

    /// Per-topic difficulty overrides. Key is the raw 32-byte
    /// topic id.
    pub topic_overrides: BTreeMap<[u8; 32], u32>,
}

impl Default for RelayPowPolicy {
    fn default() -> Self {
        DEFAULT_POLICY.clone()
    }
}

impl RelayPowPolicy {
    /// Return the difficulty required for `topic_id`. Uses a
    /// per-topic override if set, else `default_difficulty`.
    /// Clamps the result at [`MAX_DIFFICULTY_BITS`] as a last-
    /// line defence against a malformed policy that slipped past
    /// the loader.
    pub fn difficulty_for(&self, topic_id: &[u8; 32]) -> u32 {
        self.topic_overrides
            .get(topic_id)
            .copied()
            .unwrap_or(self.default_difficulty)
            .min(MAX_DIFFICULTY_BITS)
    }

    /// Convert a file representation to a hydrated policy.
    /// Rejects any override whose difficulty exceeds
    /// [`MAX_DIFFICULTY_BITS`] : a policy that asks for 100 bits
    /// is almost certainly a typo, and the solver would burn CPU
    /// for hours. Loud failure is kinder than silent clamping
    /// here.
    pub fn from_file(file: RelayPowPolicyFile) -> Result<Self> {
        if file.default_difficulty > MAX_DIFFICULTY_BITS {
            return Err(NexusError::Other(format!(
                "relay PoW policy default_difficulty={} exceeds MAX_DIFFICULTY_BITS={}",
                file.default_difficulty, MAX_DIFFICULTY_BITS,
            )));
        }
        let mut overrides = BTreeMap::new();
        for (hex_key, difficulty) in file.topic_overrides {
            if difficulty > MAX_DIFFICULTY_BITS {
                return Err(NexusError::Other(format!(
                    "relay PoW policy topic_overrides[{hex_key:?}]={difficulty} \
                     exceeds MAX_DIFFICULTY_BITS={MAX_DIFFICULTY_BITS}",
                )));
            }
            if hex_key.len() != 64 {
                return Err(NexusError::Other(format!(
                    "relay PoW policy topic_overrides key {hex_key:?} is not 64 chars"
                )));
            }
            let mut topic = [0u8; 32];
            hex::decode_to_slice(&hex_key, &mut topic).map_err(|e| {
                NexusError::Other(format!(
                    "relay PoW policy topic_overrides key {hex_key:?} is not valid hex: {e}"
                ))
            })?;
            overrides.insert(topic, difficulty);
        }
        Ok(RelayPowPolicy {
            default_difficulty: file.default_difficulty,
            topic_overrides: overrides,
        })
    }
}

fn default_difficulty_default() -> u32 {
    DEFAULT_DIFFICULTY_BITS
}

/// Resolve the active policy file path. `SBFB_POW_POLICY_PATH`
/// env var wins ; else `$SBFB_HOME/relay_pow_policy.toml`
/// (default `~/.sbfb/relay_pow_policy.toml`).
///
/// Returns `None` on a platform where neither `SBFB_HOME` nor
/// `HOME`/`USERPROFILE` is set.
pub fn relay_pow_policy_file_path() -> Option<PathBuf> {
    if let Ok(path) = env::var(CUSTOM_POW_POLICY_ENV) {
        if !path.is_empty() {
            return Some(PathBuf::from(path));
        }
    }
    sbfb_home().map(|h| h.join(RELAY_POW_POLICY_FILE_NAME))
}

/// `$SBFB_HOME` / `~/.sbfb`. Mirror of the helper in
/// [`crate::relay_config`].
fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var(crate::relay_config::SBFB_HOME_ENV) {
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

/// Load the active policy.
///
/// - Missing file → [`DEFAULT_POLICY`].
/// - Parse error → loud `Err`.
/// - Policy-invariant violation (over-max difficulty, bad hex
///   key) → loud `Err`.
pub fn load_relay_pow_policy() -> Result<RelayPowPolicy> {
    let Some(path) = relay_pow_policy_file_path() else {
        return Ok(DEFAULT_POLICY.clone());
    };
    load_relay_pow_policy_from(&path)
}

/// Load the policy from an explicit path. Thin twin of
/// [`load_relay_pow_policy`] that bypasses the
/// `SBFB_POW_POLICY_PATH` / `sbfb_home` resolution.
///
/// Sprint 20 Phase C : the daemon-side `PowPolicyWatcher` resolves
/// the path once at boot and re-invokes this helper on every
/// `notify` reload, so the resolution logic does not race with the
/// watcher. A missing file still collapses to
/// [`DEFAULT_POLICY`] — identical contract to the env-var variant.
pub fn load_relay_pow_policy_from(path: &Path) -> Result<RelayPowPolicy> {
    if !path.is_file() {
        return Ok(DEFAULT_POLICY.clone());
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| NexusError::Other(format!("failed to read {}: {e}", path.display())))?;
    let file: RelayPowPolicyFile = toml::from_str(&raw)
        .map_err(|e| NexusError::Other(format!("failed to parse {}: {e}", path.display())))?;
    RelayPowPolicy::from_file(file)
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Same ENV_GUARD pattern as relay_config.rs + pkarr_resolver.rs :
    // env var mutation is process-global, serialise across tests.
    static ENV_GUARD: Mutex<()> = Mutex::new(());

    struct EnvSnapshot {
        pairs: Vec<(&'static str, Option<String>)>,
    }

    impl EnvSnapshot {
        fn capture(keys: &[&'static str]) -> Self {
            let pairs = keys
                .iter()
                .map(|k| (*k, env::var(k).ok()))
                .collect::<Vec<_>>();
            for (k, _) in &pairs {
                env::remove_var(k);
            }
            EnvSnapshot { pairs }
        }
    }

    impl Drop for EnvSnapshot {
        fn drop(&mut self) {
            for (k, v) in &self.pairs {
                match v {
                    Some(val) => env::set_var(k, val),
                    None => env::remove_var(k),
                }
            }
        }
    }

    #[test]
    fn default_policy_is_2_18() {
        assert_eq!(DEFAULT_POLICY.default_difficulty, DEFAULT_DIFFICULTY_BITS);
        assert_eq!(DEFAULT_DIFFICULTY_BITS, 18);
        assert!(DEFAULT_POLICY.topic_overrides.is_empty());
    }

    #[test]
    fn load_policy_returns_default_when_file_absent() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let _snap =
            EnvSnapshot::capture(&[CUSTOM_POW_POLICY_ENV, crate::relay_config::SBFB_HOME_ENV]);
        env::set_var(crate::relay_config::SBFB_HOME_ENV, tmp.path());

        let policy = load_relay_pow_policy().expect("no file → default");
        assert_eq!(policy, DEFAULT_POLICY);
    }

    #[test]
    fn load_policy_parses_per_topic_override() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("pol.toml");
        let hex_topic = "a".repeat(64);
        fs::write(
            &path,
            format!(
                r#"
default_difficulty = 14

[topic_overrides]
"{hex_topic}" = 22
"#
            ),
        )
        .unwrap();

        let _snap =
            EnvSnapshot::capture(&[CUSTOM_POW_POLICY_ENV, crate::relay_config::SBFB_HOME_ENV]);
        env::set_var(CUSTOM_POW_POLICY_ENV, &path);

        let policy = load_relay_pow_policy().expect("parse ok");
        assert_eq!(policy.default_difficulty, 14);
        let topic_bytes = [0xAAu8; 32];
        assert_eq!(policy.difficulty_for(&topic_bytes), 22);
        // A topic not in the override map falls back to default.
        let other_topic = [0xCCu8; 32];
        assert_eq!(policy.difficulty_for(&other_topic), 14);
    }

    #[test]
    fn load_policy_rejects_invalid_toml() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("broken.toml");
        fs::write(&path, "this is = not = valid = toml [[").unwrap();

        let _snap =
            EnvSnapshot::capture(&[CUSTOM_POW_POLICY_ENV, crate::relay_config::SBFB_HOME_ENV]);
        env::set_var(CUSTOM_POW_POLICY_ENV, &path);

        let err = load_relay_pow_policy().expect_err("invalid TOML must surface");
        let msg = err.to_string();
        assert!(msg.contains("failed to parse"), "got: {msg}");
    }

    #[test]
    fn load_policy_rejects_difficulty_over_max() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("big.toml");
        fs::write(&path, "default_difficulty = 100").unwrap();

        let _snap =
            EnvSnapshot::capture(&[CUSTOM_POW_POLICY_ENV, crate::relay_config::SBFB_HOME_ENV]);
        env::set_var(CUSTOM_POW_POLICY_ENV, &path);

        let err = load_relay_pow_policy().expect_err("over-max must surface");
        assert!(err.to_string().contains("exceeds MAX_DIFFICULTY_BITS"));
    }

    #[test]
    fn difficulty_for_clamps_stored_override_over_max() {
        // A malformed policy that slipped past the loader still
        // clamps at serve time. Tested by hand-constructing the
        // hydrated struct directly (bypassing from_file guards).
        let mut overrides = BTreeMap::new();
        let topic = [0x33u8; 32];
        overrides.insert(topic, 99);
        let policy = RelayPowPolicy {
            default_difficulty: 18,
            topic_overrides: overrides,
        };
        assert_eq!(policy.difficulty_for(&topic), MAX_DIFFICULTY_BITS);
    }
}
