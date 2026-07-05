// SPDX-License-Identifier: AGPL-3.0-or-later
//! Custom relay configuration for multi-relai federation phase 1
//! (Sprint 18 Phase C).
//!
//! iroh `presets::N0` wires the four n0-run relays (NA east, NA
//! west, EU, AP) by default. SBFB wants to let operators override
//! that list with their own relays — ONG-run, self-hosted, or a
//! mix — without hand-editing the iroh source. This module
//! resolves the active relay set from three layered sources, in
//! precedence order:
//!
//! 1. `SBFB_CUSTOM_RELAYS` env var (comma-separated URLs). Takes
//!    absolute precedence : intended for container / CI overrides
//!    that must not depend on a user-editable file.
//! 2. `~/.sbfb/relays.json` (or `$SBFB_HOME/relays.json`) on disk.
//!    Human-editable list of relay URLs, same schema as what the
//!    env var encodes. Non-existent file is a no-op (falls through).
//! 3. Fallback to `iroh::defaults::prod::default_relay_map()` —
//!    the four n0 production relays
//!    (`use1-1`/`usw1-1`/`euc1-1`/`aps1-1` `.relay.n0.iroh.link`,
//!    per the vendored iroh 1.0.1 `defaults.rs`). The concrete
//!    hostnames follow whatever the pinned iroh version ships —
//!    they DID change at the 1.0 bump (the `iroh-canary` label was
//!    dropped from every host), so the fallback tracks the live n0
//!    fleet automatically instead of matching any historical set
//!    byte-for-byte. SBFB hardcodes no relay hostname of its own.
//!
//! Public API :
//!
//! - [`load_relay_map`] — try (1) then (2), return `Some(RelayMap)`
//!   with validated URLs ; return `None` when both sources are
//!   absent/empty so callers can fall back to the preset's default.
//! - [`validate_relay_url`] — single-URL policy check. Exported so
//!   tests and future config-editor tooling can reuse the same
//!   rules.
//!
//! # Threat model
//!
//! Custom relays are a trust transfer : a node configured with a
//! rogue relay sees its traffic mediated by that relay until a
//! direct hole-punched path is found. The policy here rejects
//! plaintext HTTP (defence in depth, even though iroh's QUIC
//! tunnel is end-to-end encrypted) and rejects loopback URLs
//! outside dev mode so a stale config file cannot silently route
//! production traffic through a local MITM proxy.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use iroh::{RelayConfig as IrohRelayConfig, RelayMap, RelayUrl};
use serde::{Deserialize, Serialize};

use crate::error::{NexusError, Result};

/// Environment variable that carries a comma-separated list of
/// relay URLs. Empty or absent value falls through to the file
/// source. Matches the 12-factor-app env override pattern used by
/// the worker allowlist (`NEXUS_ALLOWLIST_DIR`) and the state
/// dir (`NEXUS_GRID_ROOT`).
pub const CUSTOM_RELAYS_ENV: &str = "SBFB_CUSTOM_RELAYS";

/// Environment variable that relaxes [`validate_relay_url`] to
/// accept `localhost`/`127.0.0.1`/`[::1]` targets. Off by default
/// so a production deployment whose config file accidentally
/// carries a dev URL fails loud at boot instead of silently
/// routing traffic through a loopback.
pub const DEV_MODE_ENV: &str = "SBFB_DEV_MODE";

/// Override for the `~/.sbfb/` home directory, mirroring the
/// other crates that honour this variable (consent.rs,
/// shell-daemon-core auth.rs). Used by tests to swap in a
/// temp dir.
pub const SBFB_HOME_ENV: &str = "SBFB_HOME";

/// Name of the human-editable relay list file inside `~/.sbfb/`.
pub const RELAYS_FILE_NAME: &str = "relays.json";

/// On-disk representation of the custom relay list. Deliberately
/// minimal : one `relays` array with `url` strings. Additional
/// per-relay knobs (QUIC port override, region, etc.) are left
/// for later sprints to avoid locking in a schema before the
/// ONG-run relay experiment actually ships. Operators who need a
/// non-default QUIC port today can host their relay on the
/// default QUIC port (see `iroh::defaults::DEFAULT_RELAY_QUIC_PORT`,
/// currently 7842) which is what the iroh `RelayConfig::from(url)`
/// conversion selects under the hood.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayListFile {
    /// List of relay entries. Each entry is at minimum an HTTPS
    /// URL. An empty list means "no custom relays — fall back to
    /// the iroh defaults".
    #[serde(default)]
    pub relays: Vec<RelayEntry>,
}

/// One relay entry in the config file. Only the URL is required.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayEntry {
    /// HTTPS URL of the relay, e.g. `https://relay.example.org`.
    pub url: String,
}

/// Resolve the active relay map from env / file.
///
/// Returns `Some(RelayMap)` when the user has configured at
/// least one custom relay (via env or file) ; returns `None` when
/// both sources are absent/empty so the caller can keep the iroh
/// preset's default relays.
///
/// Never fails for missing config : a missing env var + missing
/// file is the expected "use defaults" state. The only `Err`
/// paths are malformed config (bad JSON, bad URL, rejected
/// URL policy) — those are loud to flag operator mistakes.
pub fn load_relay_map() -> Result<Option<RelayMap>> {
    // 1. env
    if let Ok(raw) = env::var(CUSTOM_RELAYS_ENV) {
        let urls: Vec<&str> = raw
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if !urls.is_empty() {
            let nodes = urls
                .iter()
                .map(|u| build_relay_entry(u))
                .collect::<Result<Vec<_>>>()?;
            return Ok(Some(RelayMap::from_iter(nodes)));
        }
    }

    // 2. file
    if let Some(path) = relays_file_path()
        && path.is_file()
    {
        let raw = fs::read_to_string(&path)
            .map_err(|e| NexusError::Endpoint(format!("failed to read {}: {e}", path.display())))?;
        let cfg: RelayListFile = serde_json::from_str(&raw).map_err(|e| {
            NexusError::Endpoint(format!("failed to parse {}: {e}", path.display()))
        })?;
        if !cfg.relays.is_empty() {
            let nodes = cfg
                .relays
                .iter()
                .map(|entry| build_relay_entry(&entry.url))
                .collect::<Result<Vec<_>>>()?;
            return Ok(Some(RelayMap::from_iter(nodes)));
        }
    }

    // 3. defaults : signal caller to keep preset's relays.
    Ok(None)
}

/// Return the absolute path of the custom relays config file for
/// the current user, or `None` on a platform where neither
/// `SBFB_HOME` nor `HOME` / `USERPROFILE` is set.
pub fn relays_file_path() -> Option<PathBuf> {
    sbfb_home().map(|h| h.join(RELAYS_FILE_NAME))
}

/// Resolve `~/.sbfb/` for the current user. Mirrors the helper in
/// `crates/nexus-worker-core::consent` and
/// `crates/nexus-shell-daemon-core::auth` — kept local to avoid a
/// new cross-crate dep just for one path.
fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var(SBFB_HOME_ENV)
        && !dir.is_empty()
    {
        return Some(PathBuf::from(dir));
    }
    let home = env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())?;
    if home.is_empty() {
        return None;
    }
    Some(PathBuf::from(home).join(".sbfb"))
}

/// Validate a relay URL against the SBFB policy.
///
/// Policy :
///
/// - Scheme **must** be `https`. Plain `http` is rejected even
///   though iroh's QUIC tunnel is separately encrypted : the
///   control channel carrying the relay handshake rides on HTTPS
///   and an `http://` typo here is almost always an operator
///   mistake we want to flag loud.
/// - Host **must not** be a loopback (`localhost`, `127.0.0.1`,
///   `[::1]`) unless `SBFB_DEV_MODE=1` is set. A stale dev
///   config leaking into prod would silently route production
///   traffic through the operator's laptop ; rejecting it by
///   default is cheaper than debugging the incident.
///
/// Returns the parsed [`RelayUrl`] on success.
pub fn validate_relay_url(raw: &str) -> Result<RelayUrl> {
    let url = RelayUrl::from_str(raw)
        .map_err(|e| NexusError::Endpoint(format!("relay url {raw:?} is not a valid URL: {e}")))?;

    // RelayUrl derefs to url::Url ; the shared policy check reads
    // scheme() + host_str() off the deref target.
    enforce_url_policy(&url, raw, "relay url")?;

    Ok(url)
}

/// Shared policy check for every operator-supplied discovery URL
/// (iroh relays here, zero-n0 pkarr relays in
/// [`crate::discovery_override`]) : https-only + loopback rejection
/// outside dev mode. Factored out at Sprint 81 Phase E2 so the two
/// call sites cannot drift apart — the policy rationale lives on
/// [`validate_relay_url`].
///
/// `what` names the URL kind in error messages (e.g. `"relay url"`,
/// `"zero-n0 pkarr relay url"`) so operators can attribute a rejected
/// URL to the right config knob.
pub(crate) fn enforce_url_policy(url: &url::Url, raw: &str, what: &str) -> Result<()> {
    if url.scheme() != "https" {
        return Err(NexusError::Endpoint(format!(
            "{what} {raw:?} must use https scheme (got {:?})",
            url.scheme()
        )));
    }

    let dev_mode = env::var(DEV_MODE_ENV)
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if !dev_mode
        && let Some(host) = url.host_str()
        && (host == "localhost" || host == "127.0.0.1" || host == "[::1]" || host == "::1")
    {
        return Err(NexusError::Endpoint(format!(
            "{what} {raw:?} points to loopback ({host}); \
                     set {DEV_MODE_ENV}=1 if this is intentional"
        )));
    }

    Ok(())
}

/// Build a single iroh [`IrohRelayConfig`] entry from a raw URL.
/// Validates the URL against [`validate_relay_url`] and uses the
/// iroh `From<RelayUrl>` impl to pick the default QUIC config
/// (default port 7842).
fn build_relay_entry(raw: &str) -> Result<IrohRelayConfig> {
    let url = validate_relay_url(raw)?;
    Ok(IrohRelayConfig::from(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env var mutation is process-global ; serialise tests that
    // touch it so parallel execution does not produce flaky
    // cross-leakage (one test unsets while another reads).
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
            // Start from a clean slate for the test body.
            for (k, _) in &pairs {
                // SAFETY: test-only; nextest runs each test in its own process.
                unsafe { env::remove_var(k) };
            }
            EnvSnapshot { pairs }
        }
    }

    impl Drop for EnvSnapshot {
        fn drop(&mut self) {
            for (k, v) in &self.pairs {
                match v {
                    // SAFETY: test-only; nextest runs each test in its own process.
                    Some(val) => unsafe { env::set_var(k, val) },
                    // SAFETY: test-only; nextest runs each test in its own process.
                    None => unsafe { env::remove_var(k) },
                }
            }
        }
    }

    #[test]
    fn load_relay_map_returns_none_when_env_and_file_absent() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(SBFB_HOME_ENV, tmp.path()) };

        let got = load_relay_map().expect("no env + no file must succeed");
        assert!(
            got.is_none(),
            "no env + empty sbfb_home should fall through to defaults"
        );
    }

    #[test]
    fn load_relay_map_parses_env_comma_separated() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe {
            env::set_var(
                CUSTOM_RELAYS_ENV,
                "https://relay1.example.org,https://relay2.example.org",
            )
        };

        let map = load_relay_map()
            .expect("env parse must succeed")
            .expect("env set → Some");
        assert_eq!(map.len(), 2, "two URLs should produce two relay entries");
    }

    #[test]
    fn load_relay_map_parses_json_file() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join(RELAYS_FILE_NAME);
        fs::write(
            &file,
            r#"{"relays":[{"url":"https://file-relay.example.org"}]}"#,
        )
        .unwrap();

        let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(SBFB_HOME_ENV, tmp.path()) };

        let map = load_relay_map()
            .expect("file parse must succeed")
            .expect("file present → Some");
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn load_relay_map_env_overrides_file() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join(RELAYS_FILE_NAME),
            r#"{"relays":[{"url":"https://file-relay.example.org"}]}"#,
        )
        .unwrap();

        let _snap = EnvSnapshot::capture(&[CUSTOM_RELAYS_ENV, SBFB_HOME_ENV, DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(SBFB_HOME_ENV, tmp.path()) };
        // Env carries TWO URLs, file carries ONE — env wins.
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe {
            env::set_var(
                CUSTOM_RELAYS_ENV,
                "https://env1.example.org,https://env2.example.org",
            )
        };

        let map = load_relay_map().unwrap().unwrap();
        assert_eq!(
            map.len(),
            2,
            "env override must win over file (expected 2 URLs from env, not 1 from file)"
        );
    }

    #[test]
    fn validate_relay_url_rejects_http_scheme() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(&[DEV_MODE_ENV]);
        let err = validate_relay_url("http://relay.example.org").unwrap_err();
        assert!(
            err.to_string().contains("https scheme"),
            "error must mention https scheme (got {err:?})"
        );
    }

    #[test]
    fn validate_relay_url_rejects_localhost_non_dev() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(&[DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::remove_var(DEV_MODE_ENV) };
        let err = validate_relay_url("https://localhost:4433").unwrap_err();
        assert!(
            err.to_string().contains("loopback"),
            "error must mention loopback (got {err:?})"
        );
    }

    #[test]
    fn validate_relay_url_accepts_localhost_when_dev() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(&[DEV_MODE_ENV]);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(DEV_MODE_ENV, "1") };
        validate_relay_url("https://localhost:4433")
            .expect("dev mode should allow loopback relays");
    }
}
