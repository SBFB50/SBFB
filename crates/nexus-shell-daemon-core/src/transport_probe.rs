// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase E.6 (post-G8 ajusté) — boot-time UDP QUIC
//! transport probe with degraded-mode observability.
//!
//! ## What this module does
//!
//! At daemon boot, run a short probe (default : up to 3 dial
//! attempts in a 10-second window) of the configured iroh
//! endpoint's ability to establish a **direct UDP QUIC** path to
//! a known peer. If every attempt fails, emit a structured
//! `tracing::warn!` event with a `transport.degraded_mode = true`
//! field so ops dashboards can flag the daemon as "running in
//! relay-WSS-only mode" without changing anything in the data
//! plane.
//!
//! ## What this module DOES NOT do (G8 finding 2026-04-18)
//!
//! The original Sprint 20 plan §8.1 E.6 called for a manual
//! switch to `RelayMode::Custom` with a `relay_wss_only = true`
//! flag. The G8 phase pre-flight scan
//! ([`sprint20_phase_E_preflight.md`]) showed that this flag
//! does not exist : iroh 0.91 already removed the raw-TCP option
//! from the relay client (cf. blog post
//! `iroh-0-91-0-the-last-relay-break`). WebSockets / TLS over
//! TCP 443 is the *only* mode the relay client speaks since 0.91,
//! and this is still true under iroh-relay 1.0.1 (re-verified at
//! the S81 Phase E re-cert: the client data path is
//! `tokio_websockets` in `client/conn.rs`; the separate
//! `DEFAULT_RELAY_QUIC_PORT` 7842 serves QUIC *address
//! discovery*, not a relay data path). The fall-back from a
//! failed UDP QUIC hole-punch to a relay-WSS path is
//! **automatic** inside iroh and requires no client-side
//! configuration.
//!
//! Consequently this module is intentionally observability-only :
//! it never reaches into [`iroh::Endpoint`] to mutate the relay
//! mode, and it never tries to "force WSS". It just measures and
//! reports.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::warn;

/// Outcome of a [`probe_with_retries`] run.
///
/// A `Direct` outcome means at least one dial attempt completed
/// inside the budget — the daemon has a working UDP QUIC path
/// (potentially via hole-punching). A `Degraded` outcome means
/// every attempt failed and the daemon is running on the
/// automatic relay-WSS fallback path baked into iroh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProbeOutcome {
    /// At least one direct UDP QUIC dial succeeded.
    Direct,
    /// Every dial attempt failed — falling back to relay-WSS.
    Degraded,
}

impl TransportProbeOutcome {
    /// Whether this outcome indicates the daemon is running in
    /// the relay-WSS degraded mode.
    pub fn is_degraded(self) -> bool {
        matches!(self, Self::Degraded)
    }
}

/// Configuration knobs for [`probe_with_retries`]. The defaults
/// match the Sprint 20 §Phase E.6 plan : 3 attempts, 10-second
/// total budget, ~3.3-second per-attempt timeout.
#[derive(Debug, Clone, Copy)]
pub struct ProbeConfig {
    /// How many dial attempts to make before giving up.
    pub max_attempts: u32,
    /// Per-attempt deadline — after this, the attempt is
    /// considered a failure (the prober itself decides whether
    /// to enforce it via tokio timeout or trust an underlying
    /// timeout).
    pub attempt_timeout: Duration,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            attempt_timeout: Duration::from_millis(3_333),
        }
    }
}

/// Trait abstraction over "make one probe dial, return whether
/// it succeeded". The live impl is an `iroh::Endpoint` connect
/// call (wired by the daemon binary at boot) ; tests use a mock
/// that records call count + returns a scripted boolean.
///
/// Returning `bool` rather than a richer `Result` is deliberate :
/// the probe is best-effort observability, not a control-flow
/// gate. A failure could be DNS, firewall, hole-punch failure,
/// or just an idle relay — all collapse to "no direct path".
#[async_trait::async_trait]
pub trait TransportProber: Send + Sync {
    /// Attempt a single direct UDP QUIC dial to a known peer.
    /// Implementations SHOULD enforce `attempt_timeout`
    /// internally (e.g. via `tokio::time::timeout`) — the
    /// driver loop trusts the result without imposing a second
    /// timeout on top.
    async fn probe_once(&self, attempt_timeout: Duration) -> bool;
}

/// Run up to `cfg.max_attempts` probe dials. Returns `Direct` as
/// soon as one succeeds; otherwise `Degraded` after all attempts
/// have failed. On `Degraded`, emit a single structured
/// `tracing::warn!` so ops dashboards can pick it up without
/// having to count attempts themselves.
pub async fn probe_with_retries(
    prober: &dyn TransportProber,
    cfg: ProbeConfig,
) -> TransportProbeOutcome {
    for attempt in 1..=cfg.max_attempts {
        if prober.probe_once(cfg.attempt_timeout).await {
            return TransportProbeOutcome::Direct;
        }
        tracing::debug!(
            attempt,
            max_attempts = cfg.max_attempts,
            "transport probe: UDP QUIC dial attempt failed, retrying"
        );
    }
    warn!(
        target: "nexus_shell_daemon_core::transport_probe",
        transport_degraded_mode = true,
        max_attempts = cfg.max_attempts,
        attempt_timeout_ms = cfg.attempt_timeout.as_millis() as u64,
        "transport probe: every UDP QUIC dial attempt failed; daemon is running on the iroh relay-WSS fallback path (TCP 443). This is automatic and the data plane keeps working — this log is for ops visibility only."
    );
    TransportProbeOutcome::Degraded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Mock prober that returns a scripted sequence of outcomes.
    struct ScriptedProber {
        /// Bitmask : index `i` -> whether attempt `i` succeeds.
        results: Vec<bool>,
        calls: AtomicU32,
    }

    impl ScriptedProber {
        fn new(results: Vec<bool>) -> Self {
            Self {
                results,
                calls: AtomicU32::new(0),
            }
        }
        fn call_count(&self) -> u32 {
            self.calls.load(Ordering::SeqCst)
        }
    }

    #[async_trait::async_trait]
    impl TransportProber for ScriptedProber {
        async fn probe_once(&self, _attempt_timeout: Duration) -> bool {
            let idx = self.calls.fetch_add(1, Ordering::SeqCst) as usize;
            *self.results.get(idx).unwrap_or(&false)
        }
    }

    #[tokio::test]
    async fn probe_succeeds_first_attempt_returns_direct() {
        let prober = ScriptedProber::new(vec![true, false, false]);
        let outcome = probe_with_retries(&prober, ProbeConfig::default()).await;
        assert_eq!(outcome, TransportProbeOutcome::Direct);
        assert!(!outcome.is_degraded());
        // Short-circuits as soon as one attempt succeeds.
        assert_eq!(prober.call_count(), 1);
    }

    #[tokio::test]
    async fn probe_succeeds_after_two_failures_returns_direct() {
        let prober = ScriptedProber::new(vec![false, false, true]);
        let outcome = probe_with_retries(&prober, ProbeConfig::default()).await;
        assert_eq!(outcome, TransportProbeOutcome::Direct);
        assert_eq!(prober.call_count(), 3);
    }

    #[tokio::test]
    async fn probe_fails_3x_returns_degraded() {
        let prober = ScriptedProber::new(vec![false, false, false]);
        let outcome = probe_with_retries(&prober, ProbeConfig::default()).await;
        assert_eq!(outcome, TransportProbeOutcome::Degraded);
        assert!(outcome.is_degraded());
        assert_eq!(prober.call_count(), 3);
    }

    #[tokio::test]
    async fn probe_with_custom_max_attempts_respects_budget() {
        // Verify the loop honours `max_attempts` rather than the
        // hard-coded default — this is the test that catches a
        // future refactor accidentally pinning 3.
        let prober = ScriptedProber::new(vec![false; 10]);
        let cfg = ProbeConfig {
            max_attempts: 5,
            attempt_timeout: Duration::from_millis(50),
        };
        let outcome = probe_with_retries(&prober, cfg).await;
        assert_eq!(outcome, TransportProbeOutcome::Degraded);
        assert_eq!(prober.call_count(), 5);
    }
}
