// SPDX-License-Identifier: AGPL-3.0-or-later
//! pkarr relay client adapter implementing [`QuorumResolver`].
//!
//! Wraps [`iroh::address_lookup::pkarr::PkarrRelayClient`] (iroh
//! 0.97) so that `N` instances — one per pkarr relay in our
//! federation — can feed the [`crate::dht_quorum::redundant_resolve`]
//! 2/3 quorum primitive. Byte-for-byte comparison is performed on
//! the canonical pkarr relay payload returned by the iroh client
//! (`SignedPacket::to_relay_payload`), which is stable across
//! relays serving the same record.
//!
//! ## Sprint 19 Phase A
//!
//! Closes Sprint 18 audit finding C-1 (P2 carry-over) : the
//! `dht_quorum` primitive shipped ready-to-wire in Sprint 18 but
//! had no production call site. This module is the missing
//! adapter between the generic [`QuorumResolver`] trait and the
//! concrete pkarr relay lookups that the shell daemon performs
//! before dialing a peer — flipping the Eclipse-by-DHT defence
//! from "armed" to "engaged".
//!
//! The adapter only performs the lookup — it deliberately does
//! **not** parse the returned `SignedPacket` into an
//! [`iroh::EndpointAddr`] or seed any local lookup cache. Because
//! pkarr packets are signed by the node's own secret key, a
//! rogue relay cannot forge a fake address — it can only refuse
//! to answer, time out, or serve a stale payload. The canary
//! semantics we want are exactly "do the three relays agree that
//! the same bytes represent this node right now?" : on agreement
//! the caller is free to dial via iroh's normal discovery path
//! (preset `N0`), on disagreement the caller skips the dial and
//! reports the peer as unreachable.

use std::str::FromStr;
use std::sync::Arc;

use async_trait::async_trait;
use iroh::EndpointId;
use iroh::address_lookup::pkarr::PkarrRelayClient;
use iroh::tls::{CaRootsConfig, default_provider};
use url::Url;

use crate::dht_quorum::QuorumResolver;
use crate::error::{NexusError, Result};

/// Default pkarr relay URL — shipped here as a reference value
/// operators can copy into `SBFB_PKARR_RELAYS` when seeding a
/// quorum. Sprint 19 deliberately does NOT wire this URL into
/// the canary automatically : a 1-slot quorum or three copies of
/// the same URL do not provide inter-relay cross-checking, so
/// the eclipse defence only engages once the operator supplies
/// ≥ 2 distinct relays (ONG-run targets coming in Sprint 20+
/// alongside the encryption-at-rest big-rock).
pub const DEFAULT_PKARR_RELAY_URL: &str = "https://dns.iroh.link/pkarr";

/// Environment variable carrying a comma-separated list of pkarr
/// relay URLs to wire as the Sprint 19 Phase A canary quorum.
/// Mirrors the S18 [`crate::relay_config::CUSTOM_RELAYS_ENV`]
/// pattern for iroh relays — kept as an env-only knob in Phase A
/// (no JSON file loader yet) to minimise config surface ; a
/// `~/.sbfb/pkarr_relays.json` loader arrives in Sprint 20+ when
/// the ONG-run relay federation has concrete targets.
///
/// Empty or absent value = no canary (aggregator falls through
/// to direct probe). At least 2 distinct URLs are recommended
/// for cross-relay eclipse detection ; a single URL works but
/// provides only "lookup must succeed" semantics, not quorum.
pub const CUSTOM_PKARR_RELAYS_ENV: &str = "SBFB_PKARR_RELAYS";

/// Adapter implementing [`QuorumResolver`] on top of a single
/// pkarr relay client.
///
/// Instantiate `N` of these (one per relay URL) and hand them to
/// [`crate::dht_quorum::redundant_resolve`] to get a 2-of-N
/// canary on the resolved record.
pub struct PkarrQuorumResolver {
    /// Short label used in `redundant_resolve` warn logs when
    /// this resolver disagrees with the majority or errors.
    /// Derived from the relay URL's host component so operators
    /// can attribute a dissenting lookup to a concrete relay.
    label: String,
    /// The underlying iroh pkarr relay client. Cheap to construct
    /// and cheap to `.resolve()` — holds an internal `reqwest`
    /// HTTP client built against our WebPKI root store.
    client: PkarrRelayClient,
}

impl PkarrQuorumResolver {
    /// Build a resolver pointing at a single pkarr relay URL.
    ///
    /// Uses iroh's default CA root config
    /// ([`iroh::tls::CaRootsConfig::default`], which is
    /// `EmbeddedWebPki` — the Mozilla-trusted roots compiled into
    /// the `webpki-roots` crate), matching the root set iroh
    /// itself ships with for relay connections. Fails loud when
    /// the TLS `ClientConfig` cannot be built (e.g. the
    /// cryptographic provider refuses to initialise) — returning
    /// an error is the right call because a silent fallback to
    /// plaintext would quietly undermine the eclipse defence this
    /// module exists for.
    ///
    /// The label defaults to the URL's host (e.g.
    /// `"dns.iroh.link"`) and falls back to `"unknown"` only if
    /// the URL has no host component — which cannot happen for a
    /// URL that already parsed into a [`Url`], but the fallback
    /// avoids an `unwrap` on the happy path.
    pub fn new(pkarr_relay_url: Url) -> Result<Self> {
        let label = pkarr_relay_url.host_str().unwrap_or("unknown").to_string();
        let tls_config = CaRootsConfig::default()
            .client_config(default_provider())
            .map_err(|e| {
                NexusError::Endpoint(format!("pkarr quorum resolver TLS config failed: {e}"))
            })?;
        let client = PkarrRelayClient::new(pkarr_relay_url, tls_config);
        Ok(Self { label, client })
    }

    /// Build a quorum-sized resolver set from a list of pkarr
    /// relay URLs. Collects per-URL construction errors into the
    /// first failing one so a single malformed entry aborts the
    /// whole set loudly rather than silently running with fewer
    /// resolvers than expected.
    ///
    /// The returned vector is sized 1:1 with `urls`. Callers who
    /// want a 3-slot quorum should hand exactly three URLs ;
    /// smaller sets degrade the quorum to unanimity (see
    /// [`DEFAULT_PKARR_RELAY_URL`]).
    pub fn build_set<I>(urls: I) -> Result<Vec<Arc<dyn QuorumResolver>>>
    where
        I: IntoIterator<Item = Url>,
    {
        urls.into_iter()
            .map(|u| Self::new(u).map(|r| Arc::new(r) as Arc<dyn QuorumResolver>))
            .collect()
    }
}

/// Sprint 19 Phase A — resolve the canary resolver set from
/// [`CUSTOM_PKARR_RELAYS_ENV`].
///
/// Returns :
///
/// - `Ok(Some(vec))` when the env var is set and every URL
///   parses. Vec length matches the number of non-empty
///   comma-separated entries.
/// - `Ok(None)` when the env var is absent or empty after
///   trimming — the daemon then boots an aggregator without the
///   canary gate (pre-Sprint-19 behaviour byte-for-byte).
/// - `Err(_)` when the env var is set but at least one URL fails
///   to parse. Boot should abort : a broken pkarr relay config
///   is operator-visible state, not a soft "use defaults"
///   condition.
pub fn load_quorum_resolvers_from_env() -> Result<Option<Vec<Arc<dyn QuorumResolver>>>> {
    let raw = match std::env::var(CUSTOM_PKARR_RELAYS_ENV) {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };
    let parsed: Vec<Url> = raw
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            Url::parse(s)
                .map_err(|e| NexusError::Endpoint(format!("invalid pkarr relay URL {s:?}: {e}")))
        })
        .collect::<Result<Vec<_>>>()?;
    if parsed.is_empty() {
        return Ok(None);
    }
    PkarrQuorumResolver::build_set(parsed).map(Some)
}

#[async_trait]
impl QuorumResolver for PkarrQuorumResolver {
    fn label(&self) -> &str {
        &self.label
    }

    async fn resolve(&self, node_id_hex: &str) -> anyhow::Result<Vec<u8>> {
        let endpoint_id = EndpointId::from_str(node_id_hex)
            .map_err(|e| anyhow::anyhow!("bad endpoint id hex {node_id_hex:?}: {e}"))?;
        let packet = self
            .client
            .resolve(endpoint_id)
            .await
            .map_err(|e| anyhow::anyhow!("pkarr relay {} resolve failed: {e}", self.label))?;
        // `to_relay_payload()` returns the canonical pkarr wire
        // encoding — stable across relays serving the same signed
        // record, so byte-for-byte equality is the right
        // comparison key for `redundant_resolve`.
        Ok(packet.to_relay_payload().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_derives_label_from_url_host() {
        let url: Url = "https://dns.iroh.link/pkarr".parse().unwrap();
        let r = PkarrQuorumResolver::new(url).expect("TLS config must build");
        assert_eq!(r.label(), "dns.iroh.link");
    }

    #[test]
    fn new_label_strips_port() {
        let url: Url = "https://pkarr.example.org:4443/pkarr".parse().unwrap();
        let r = PkarrQuorumResolver::new(url).expect("TLS config must build");
        assert_eq!(
            r.label(),
            "pkarr.example.org",
            "label must not carry the port component"
        );
    }

    #[tokio::test]
    async fn resolve_rejects_malformed_endpoint_id_hex() {
        // Malformed node id must fail loud at parse time, before
        // any HTTP traffic leaves the process. This protects us
        // from passing caller bugs through to the pkarr relay
        // (which would return a 400 and be slower to surface).
        let url: Url = "https://dns.iroh.link/pkarr".parse().unwrap();
        let r = PkarrQuorumResolver::new(url).expect("TLS config must build");
        let err = r
            .resolve("not-hex-at-all")
            .await
            .expect_err("malformed endpoint id hex must not reach the HTTP client");
        let msg = format!("{err}");
        assert!(
            msg.contains("bad endpoint id hex"),
            "error should mention the parse failure, got {msg:?}"
        );
    }

    #[test]
    fn build_set_returns_one_resolver_per_url() {
        let urls = vec![
            "https://a.example".parse().unwrap(),
            "https://b.example".parse().unwrap(),
            "https://c.example".parse().unwrap(),
        ];
        let set = PkarrQuorumResolver::build_set(urls).expect("all URLs valid");
        assert_eq!(set.len(), 3, "one resolver per URL expected");
        assert_eq!(set[0].label(), "a.example");
        assert_eq!(set[1].label(), "b.example");
        assert_eq!(set[2].label(), "c.example");
    }

    // `load_quorum_resolvers_from_env` touches the process env so
    // we serialise the tests that manipulate it — same pattern
    // relay_config::tests uses — to keep parallel `cargo test`
    // runs from tripping over each other.
    use std::sync::Mutex;
    static ENV_GUARD: Mutex<()> = Mutex::new(());

    #[test]
    fn load_quorum_resolvers_from_env_returns_none_when_unset() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(CUSTOM_PKARR_RELAYS_ENV) };
        let got = load_quorum_resolvers_from_env().expect("env unset must never error");
        assert!(
            got.is_none(),
            "unset env must produce None (canary disabled)"
        );
    }

    #[test]
    fn load_quorum_resolvers_from_env_parses_comma_separated_urls() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe {
            std::env::set_var(
                CUSTOM_PKARR_RELAYS_ENV,
                "https://one.example/pkarr, https://two.example/pkarr,https://three.example/pkarr",
            )
        };
        let set = load_quorum_resolvers_from_env()
            .expect("three valid URLs must succeed")
            .expect("env set must produce Some");
        assert_eq!(set.len(), 3);
        assert_eq!(set[0].label(), "one.example");
        assert_eq!(set[1].label(), "two.example");
        assert_eq!(set[2].label(), "three.example");
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(CUSTOM_PKARR_RELAYS_ENV) };
    }

    #[test]
    fn load_quorum_resolvers_from_env_fails_loud_on_bad_url() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(CUSTOM_PKARR_RELAYS_ENV, "not a url at all") };
        // Can't use `expect_err` here : the Ok type is
        // `Option<Vec<Arc<dyn QuorumResolver>>>` and `dyn
        // QuorumResolver` is deliberately not `Debug`, so the
        // auto-generated message would fail to compile. Pattern
        // match on the variant directly.
        let got = load_quorum_resolvers_from_env();
        match got {
            Ok(_) => panic!("malformed URL must surface as Err, not silent None"),
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("invalid pkarr relay URL"),
                    "error should mention the parse failure, got {msg:?}"
                );
            }
        }
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(CUSTOM_PKARR_RELAYS_ENV) };
    }
}
