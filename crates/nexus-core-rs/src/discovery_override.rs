// SPDX-License-Identifier: AGPL-3.0-or-later
//! Zero-n0 discovery override (Sprint 81 Phase E2 — PLAN B C8).
//!
//! iroh `presets::N0` wires the endpoint to the n0-run
//! infrastructure unconditionally : it **publishes** the node's
//! address to the n0 pkarr relay (`PkarrPublisher::n0_dns()`) AND
//! **resolves** peers through the same n0 service
//! (`DnsAddressLookup::n0_dns()` outside browsers), on top of the
//! n0 relay fleet for connectivity. With that fleet EOL on
//! 2026-09-30, SBFB carries a hedge mode where the network keeps
//! converging with ZERO n0 services alive : a self-hosted
//! `iroh-relay` for connectivity plus a self-hosted pkarr relay
//! (`iroh-dns-server`) for discovery — publish and resolve both
//! move off n0 (resolve rides `PkarrResolver` HTTP against the
//! same `/pkarr` endpoint the publisher PUTs to, so one server
//! covers both directions). Ops runbook :
//! `docs/release/IROH_SELFHOST_OPS.md`.
//!
//! `presets::N0` **stays the default** — this module only ADDS an
//! opt-in mode, resolved from the environment into a validated
//! [`DiscoveryPlan`] that `node.rs` applies on a `presets::Minimal`
//! base (a base that wires nothing n0-related, so there is nothing
//! to forget to clear).
//!
//! ## Fail-loud by design
//!
//! Every half-configured state refuses to boot instead of
//! degrading :
//!
//! - gate value not recognised → error (a typo like
//!   `SBFB_ZERO_N0=yes` must never silently fall back to the n0
//!   path — "partially on n0" is indistinguishable from "working"
//!   until the n0 EOL day it is not) ;
//! - gate ON but no self-hosted pkarr relay URL → error (the
//!   endpoint could neither publish nor resolve any address) ;
//! - gate ON but no custom relay configured
//!   ([`crate::relay_config::load_relay_map`] returns `None`) →
//!   error (the endpoint would keep homing on the n0 relay fleet,
//!   a partial zero-n0 that defeats the objective).
//!
//! ## Not the canary, not duress-gated
//!
//! - [`crate::pkarr_resolver::CUSTOM_PKARR_RELAYS_ENV`]
//!   (`SBFB_PKARR_RELAYS`) feeds ONLY the browse anti-eclipse
//!   quorum canary — it never touches the endpoint's discovery.
//!   The zero-n0 knobs are deliberately distinct names.
//! - The zero-n0 mode is a connectivity substrate of the same
//!   class as `SBFB_CUSTOM_RELAYS` : read at boot, applied
//!   identically in and out of duress. Gating it on duress would
//!   create an observable transport fingerprint, violating the
//!   duress indistinguishability anti-goal ; the dial of real
//!   peers under duress is already blocked upstream by the
//!   Sprint 81 Phase C `sync_set_entry_in_duress` gate whatever
//!   discovery backend is active.

use std::env;

use iroh::RelayMap;
use url::Url;

use crate::error::{NexusError, Result};
use crate::relay_config::{CUSTOM_RELAYS_ENV, RELAYS_FILE_NAME, enforce_url_policy};

/// Environment variable gating the zero-n0 discovery override.
///
/// Recognised values : `1` / `true` enable, `0` / `false` / empty /
/// unset disable. Any OTHER value fails loud at boot — unlike the
/// permissive [`crate::relay_config::DEV_MODE_ENV`] parsing, a typo
/// here must never silently keep the node on the n0 path (see the
/// module docs).
pub const ZERO_N0_ENV: &str = "SBFB_ZERO_N0";

/// Environment variable carrying the comma-separated list of
/// self-hosted pkarr relay URLs used for BOTH publishing this
/// node's address and resolving peers when zero-n0 is enabled.
///
/// At least one URL is required when [`ZERO_N0_ENV`] is on ; two or
/// more DISTINCT relays are recommended so the discovery plane does
/// not share a single point of failure with the relay plane (see
/// THREAT_MODEL — the n0 fleet this replaces spans four regions).
/// Every URL passes the same policy as
/// [`crate::relay_config::validate_relay_url`] : https-only,
/// loopback rejected outside `SBFB_DEV_MODE=1`.
///
/// Deliberately NOT [`crate::pkarr_resolver::CUSTOM_PKARR_RELAYS_ENV`]
/// (`SBFB_PKARR_RELAYS`) : that knob feeds only the browse quorum
/// canary and re-using it here would couple two unrelated surfaces.
pub const ZERO_N0_PKARR_RELAYS_ENV: &str = "SBFB_ZERO_N0_PKARR_RELAYS";

/// A validated zero-n0 discovery plan, ready for `node.rs` to apply
/// on a `presets::Minimal` builder base.
///
/// Both fields are guaranteed non-empty by
/// [`load_discovery_override`] — the coupling is the point : a plan
/// with relays but no pkarr relay (or vice versa) is exactly the
/// silent partial-n0 state this module exists to refuse.
#[derive(Debug, Clone)]
pub struct DiscoveryPlan {
    /// Self-hosted pkarr relay URLs. Each one gets a
    /// `PkarrPublisher` (HTTP PUT) AND a `PkarrResolver` (HTTP GET)
    /// on the endpoint.
    pub pkarr_relays: Vec<Url>,
    /// The operator's custom relay map (from `SBFB_CUSTOM_RELAYS` /
    /// `relays.json`), applied as `RelayMode::Custom`.
    pub relay_map: RelayMap,
}

/// Resolve the zero-n0 discovery override from the environment.
///
/// Returns :
///
/// - `Ok(None)` when the mode is disabled ([`ZERO_N0_ENV`] unset,
///   empty, `0` or `false`) — the caller keeps the default
///   `presets::N0` path byte-identically.
/// - `Ok(Some(plan))` when the mode is enabled AND fully
///   configured : at least one valid self-hosted pkarr relay URL
///   ([`ZERO_N0_PKARR_RELAYS_ENV`]) and at least one custom relay
///   ([`crate::relay_config::load_relay_map`] → `Some`).
/// - `Err(_)` on every half-configured or malformed state — boot
///   must abort, never degrade (module docs, "Fail-loud by
///   design").
///
/// This function is the pure decision core (env → plan | error) :
/// it touches no network and no store, so the parse / validation /
/// coupling logic is unit-testable hermetically. The iroh `Builder`
/// exposes no pre-bind getter on its lookup list, which makes this
/// factoring the only dep-free way to prove the decision logic ;
/// the end-to-end "the endpoint really omits n0" proof lives in the
/// `node.rs` two-node test against in-process pkarr + relay
/// servers.
pub fn load_discovery_override() -> Result<Option<DiscoveryPlan>> {
    let raw = match env::var(ZERO_N0_ENV) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let trimmed = raw.trim();
    let enabled = if trimmed.is_empty() || trimmed == "0" || trimmed.eq_ignore_ascii_case("false") {
        false
    } else if trimmed == "1" || trimmed.eq_ignore_ascii_case("true") {
        true
    } else {
        return Err(NexusError::Endpoint(format!(
            "{ZERO_N0_ENV}={trimmed:?} is not a recognised value \
             (1/true to enable, 0/false/empty/unset to disable); \
             refusing to guess — a typo must never silently keep \
             this node on the n0 discovery path"
        )));
    };
    if !enabled {
        return Ok(None);
    }

    let pkarr_raw = env::var(ZERO_N0_PKARR_RELAYS_ENV).unwrap_or_default();
    let pkarr_relays = pkarr_raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(validate_zero_n0_pkarr_url)
        .collect::<Result<Vec<_>>>()?;
    if pkarr_relays.is_empty() {
        return Err(NexusError::Endpoint(format!(
            "zero-n0 mode is enabled ({ZERO_N0_ENV}=1) but \
             {ZERO_N0_PKARR_RELAYS_ENV} carries no pkarr relay URL; \
             without a self-hosted pkarr relay this endpoint could \
             neither publish nor resolve any address — refusing to \
             boot (see docs/release/IROH_SELFHOST_OPS.md)"
        )));
    }

    // Fail-loud coupling : zero-n0 without a custom relay map would
    // keep the endpoint homing on the n0 relay fleet — a partial
    // zero-n0 that silently defeats the EOL objective.
    let relay_map = crate::relay_config::load_relay_map()?.ok_or_else(|| {
        NexusError::Endpoint(format!(
            "zero-n0 mode is enabled ({ZERO_N0_ENV}=1) but no custom \
             relay is configured ({CUSTOM_RELAYS_ENV} or \
             ~/.sbfb/{RELAYS_FILE_NAME}); the endpoint would keep \
             homing on the n0 relay fleet — refusing to boot (see \
             docs/release/IROH_SELFHOST_OPS.md)"
        ))
    })?;

    Ok(Some(DiscoveryPlan {
        pkarr_relays,
        relay_map,
    }))
}

/// Validate one self-hosted pkarr relay URL against the same policy
/// as [`crate::relay_config::validate_relay_url`] (https-only,
/// loopback rejected outside `SBFB_DEV_MODE=1`), returning a plain
/// [`Url`] — a pkarr relay URL carries a path (`/pkarr`) and is not
/// a `RelayUrl` semantically.
///
/// Defence in depth against operator misconfig, same class as the
/// relay-side guard : an `http://` typo or a stale dev loopback in
/// the env must fail the boot loudly, not silently weaken the
/// discovery plane.
pub fn validate_zero_n0_pkarr_url(raw: &str) -> Result<Url> {
    let url = Url::parse(raw).map_err(|e| {
        NexusError::Endpoint(format!(
            "zero-n0 pkarr relay url {raw:?} is not a valid URL: {e}"
        ))
    })?;
    enforce_url_policy(&url, raw, "zero-n0 pkarr relay url")?;
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Env var mutation is process-global ; serialise the tests that
    // touch it. Same local-guard pattern as relay_config::tests and
    // pkarr_resolver::tests (nextest isolates per-process anyway ;
    // the guard covers plain `cargo test` threads).
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

    /// Every env key the override reads, directly or through the
    /// relay coupling — captured together so a test leaves no state
    /// behind for its neighbours.
    const ALL_KEYS: &[&str] = &[
        ZERO_N0_ENV,
        ZERO_N0_PKARR_RELAYS_ENV,
        CUSTOM_RELAYS_ENV,
        crate::relay_config::SBFB_HOME_ENV,
        crate::relay_config::DEV_MODE_ENV,
    ];

    #[test]
    fn returns_none_when_unset_or_disabled() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);

        // Unset → None (the default N0 path).
        assert!(load_discovery_override().unwrap().is_none());

        // Every recognised "off" spelling → None, never an error.
        for off in ["0", "false", "FALSE", "", "  "] {
            // SAFETY: test-only; nextest runs each test in its own process.
            unsafe { env::set_var(ZERO_N0_ENV, off) };
            assert!(
                load_discovery_override().unwrap().is_none(),
                "{ZERO_N0_ENV}={off:?} must disable the override"
            );
        }
    }

    #[test]
    fn fails_loud_on_unrecognised_gate_value() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_ENV, "yes") };

        let err = load_discovery_override().unwrap_err();
        assert!(
            err.to_string().contains("not a recognised value"),
            "a typo'd gate value must abort the boot, got {err:?}"
        );
    }

    #[test]
    fn fails_loud_when_pkarr_relays_missing() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_ENV, "1") };

        // Unset list.
        let err = load_discovery_override().unwrap_err();
        assert!(
            err.to_string().contains(ZERO_N0_PKARR_RELAYS_ENV),
            "error must point at the missing pkarr knob, got {err:?}"
        );

        // Set but effectively empty ( commas / spaces ).
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_PKARR_RELAYS_ENV, " , ,") };
        let err = load_discovery_override().unwrap_err();
        assert!(
            err.to_string().contains(ZERO_N0_PKARR_RELAYS_ENV),
            "an all-blank list must fail the same way, got {err:?}"
        );
    }

    #[test]
    fn fails_loud_when_custom_relays_missing() {
        let _g = ENV_GUARD.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);
        // Point SBFB_HOME at an empty dir so no developer
        // ~/.sbfb/relays.json can leak into the assertion.
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(crate::relay_config::SBFB_HOME_ENV, tmp.path()) };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_ENV, "1") };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_PKARR_RELAYS_ENV, "https://pkarr.example.org/pkarr") };

        let err = load_discovery_override().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("no custom relay is configured"),
            "zero-n0 without custom relays must refuse to boot \
             (silent homing on n0 defeats the EOL objective), got {msg:?}"
        );
        assert!(
            msg.contains(CUSTOM_RELAYS_ENV),
            "error must tell the operator which knob to set, got {msg:?}"
        );
    }

    #[test]
    fn builds_plan_when_fully_configured() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(ZERO_N0_ENV, "true") };
        // Two distinct pkarr relays (the recommended shape).
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe {
            env::set_var(
                ZERO_N0_PKARR_RELAYS_ENV,
                "https://pkarr-a.example.org/pkarr, https://pkarr-b.example.org/pkarr",
            )
        };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(CUSTOM_RELAYS_ENV, "https://relay.example.org") };

        let plan = load_discovery_override()
            .expect("fully configured must succeed")
            .expect("gate on must produce Some");
        assert_eq!(plan.pkarr_relays.len(), 2, "both pkarr URLs kept, in order");
        assert_eq!(plan.pkarr_relays[0].host_str(), Some("pkarr-a.example.org"));
        assert_eq!(plan.pkarr_relays[1].host_str(), Some("pkarr-b.example.org"));
        assert_eq!(plan.relay_map.len(), 1, "one custom relay in the map");

        // ON is case-insensitive, mirror of the FALSE spelling on
        // the off side (review D2-3).
        for on in ["TRUE", "True", "1"] {
            // SAFETY: test-only; nextest runs each test in its own process.
            unsafe { env::set_var(ZERO_N0_ENV, on) };
            assert!(
                load_discovery_override()
                    .expect("recognised on-spelling must succeed")
                    .is_some(),
                "{ZERO_N0_ENV}={on:?} must enable the override"
            );
        }
    }

    #[test]
    fn pkarr_url_policy_matches_relay_policy() {
        let _g = ENV_GUARD.lock().unwrap();
        let _snap = EnvSnapshot::capture(ALL_KEYS);

        // Unparseable string → the Url::parse error branch, distinct
        // from the policy rejections below (review D2-1).
        let err = validate_zero_n0_pkarr_url("not a url").unwrap_err();
        assert!(
            err.to_string().contains("not a valid URL"),
            "an unparseable pkarr relay URL must fail at parse time, got {err:?}"
        );

        // http → rejected (same wording class as validate_relay_url).
        let err = validate_zero_n0_pkarr_url("http://pkarr.example.org/pkarr").unwrap_err();
        assert!(
            err.to_string().contains("https scheme"),
            "plain http must be rejected, got {err:?}"
        );

        // Loopback outside dev mode → rejected.
        let err = validate_zero_n0_pkarr_url("https://localhost:8080/pkarr").unwrap_err();
        assert!(
            err.to_string().contains("loopback"),
            "loopback outside dev mode must be rejected, got {err:?}"
        );

        // Loopback in dev mode → accepted (smoke-test parity with
        // the relay-side policy).
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { env::set_var(crate::relay_config::DEV_MODE_ENV, "1") };
        validate_zero_n0_pkarr_url("https://localhost:8080/pkarr")
            .expect("dev mode must allow loopback pkarr relays");
    }
}
