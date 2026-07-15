// SPDX-License-Identifier: AGPL-3.0-or-later
//! DNS-based fallback resolver for pkarr node discovery.
//!
//! When the pkarr relay quorum ([`crate::dht_quorum::redundant_resolve`])
//! fails — all relays unreachable or timed out — this module provides
//! an alternative resolution path via encrypted DNS: DoH (RFC 8484)
//! and DoT (RFC 7858).
//!
//! pkarr records are natively DNS-compatible (they encode as standard
//! DNS resource records signed by the node's Ed25519 key). A
//! pkdns-compatible server (`github.com/pubky/pkdns`) bridges the
//! mainline DHT to standard DNS, serving these records at
//! `<node_id>.<domain_suffix>`. This module queries such servers via
//! DoH/DoT rather than plain DNS to prevent metadata leakage.
//!
//! ## Threat model
//!
//! DNS is **not** a trust anchor — pkarr records are Ed25519-signed,
//! so a rogue DNS server cannot forge content. The fallback only
//! provides transport-level resilience, extending the set of network
//! paths through which a client can obtain the signed record.
//!
//! ## Sprint 24 Phase E
//!
//! Integration point: [`crate::browse::BrowseAggregator`] (in
//! `nexus-shell-daemon-core`) wires the fallback into its
//! `probe_and_cache` method. When the pkarr quorum returns
//! `AllFailed`, the aggregator tries DNS before marking the peer
//! `Unreachable`. Eclipse-by-DHT (`NoMajority`) is NOT overridden
//! by DNS — a disagreeing quorum is a security signal, not a
//! connectivity issue.

use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use hickory_resolver::Resolver;
use hickory_resolver::config::{ConnectionConfig, NameServerConfig, ResolverConfig, ResolverOpts};
use hickory_resolver::net::runtime::TokioRuntimeProvider;
use hickory_resolver::proto::rr::RData;
use tracing::debug;

use crate::error::{NexusError, Result};

/// Default DoH endpoint — Cloudflare (1.1.1.1).
pub const DOH_CLOUDFLARE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
/// TLS name for Cloudflare DoH/DoT.
pub const DOH_CLOUDFLARE_TLS_NAME: &str = "cloudflare-dns.com";

/// Default DoH endpoint — Google (8.8.8.8).
pub const DOH_GOOGLE_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));
/// TLS name for Google DoH/DoT.
pub const DOH_GOOGLE_TLS_NAME: &str = "dns.google";

/// Standard DoH port (HTTPS, RFC 8484).
pub const DOH_PORT: u16 = 443;
/// Standard DoT port (RFC 7858).
pub const DOT_PORT: u16 = 853;

/// Default timeout for a single DNS fallback lookup.
pub const DEFAULT_DNS_TIMEOUT: Duration = Duration::from_secs(5);

/// Default domain suffix for pkarr-over-DNS TXT records.
pub const DEFAULT_DOMAIN_SUFFIX: &str = "_pkarr.sbfb.net";

/// Environment variable to enable DNS fallback (`"true"` or `"1"`).
pub const DNS_FALLBACK_ENABLED_ENV: &str = "SBFB_DNS_FALLBACK_ENABLED";

/// Environment variable overriding the TXT record domain suffix.
pub const DNS_FALLBACK_DOMAIN_ENV: &str = "SBFB_DNS_FALLBACK_DOMAIN";

// -----------------------------------------------------------------
// Trait (enables mocking in browse aggregator tests)
// -----------------------------------------------------------------

/// Trait for DNS fallback resolution.
///
/// Implemented by [`DnsFallbackResolver`] in production and by
/// test mocks in `nexus-shell-daemon-core::browse::tests`.
#[async_trait]
pub trait DnsFallbackResolve: Send + Sync {
    /// Short label for structured logging.
    fn label(&self) -> &str;

    /// Resolve a node via DNS TXT records.
    ///
    /// Returns the concatenated TXT record bytes on success. The
    /// caller does **not** interpret these bytes — the return value
    /// is a "the record exists" signal that green-lights the
    /// downstream iroh probe.
    async fn resolve_node(&self, node_id_hex: &str) -> anyhow::Result<Vec<u8>>;
}

// -----------------------------------------------------------------
// Config
// -----------------------------------------------------------------

/// A single DNS endpoint (DoH or DoT).
#[derive(Debug, Clone)]
pub struct DnsEndpoint {
    /// IP address of the DNS server.
    pub ip: IpAddr,
    /// Port (443 for DoH, 853 for DoT).
    pub port: u16,
    /// TLS server name for certificate validation.
    pub tls_name: String,
}

/// Configuration for the DNS fallback resolver.
#[derive(Debug, Clone)]
pub struct DnsFallbackConfig {
    /// Whether fallback is active.
    pub enabled: bool,
    /// Domain suffix appended to the node_id hex for TXT lookups.
    pub domain_suffix: String,
    /// Per-lookup timeout.
    pub timeout: Duration,
    /// DoH (DNS-over-HTTPS) endpoints.
    pub doh_endpoints: Vec<DnsEndpoint>,
    /// DoT (DNS-over-TLS) endpoints.
    pub dot_endpoints: Vec<DnsEndpoint>,
}

impl Default for DnsFallbackConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain_suffix: DEFAULT_DOMAIN_SUFFIX.to_string(),
            timeout: DEFAULT_DNS_TIMEOUT,
            doh_endpoints: vec![
                DnsEndpoint {
                    ip: DOH_CLOUDFLARE_IP,
                    port: DOH_PORT,
                    tls_name: DOH_CLOUDFLARE_TLS_NAME.to_string(),
                },
                DnsEndpoint {
                    ip: DOH_GOOGLE_IP,
                    port: DOH_PORT,
                    tls_name: DOH_GOOGLE_TLS_NAME.to_string(),
                },
            ],
            dot_endpoints: vec![
                DnsEndpoint {
                    ip: DOH_CLOUDFLARE_IP,
                    port: DOT_PORT,
                    tls_name: DOH_CLOUDFLARE_TLS_NAME.to_string(),
                },
                DnsEndpoint {
                    ip: DOH_GOOGLE_IP,
                    port: DOT_PORT,
                    tls_name: DOH_GOOGLE_TLS_NAME.to_string(),
                },
            ],
        }
    }
}

// -----------------------------------------------------------------
// Resolver
// -----------------------------------------------------------------

/// hickory 0.26: `TokioAsyncResolver` is gone upstream; the
/// tokio-backed resolver is `Resolver<TokioRuntimeProvider>`.
type TokioResolver = Resolver<TokioRuntimeProvider>;

/// Transport of one of the two raced resolvers. Replaces the
/// hickory 0.24 `Protocol` config enum (removed in 0.26): the
/// fallback only speaks DoH and DoT, so a two-variant enum makes
/// "unsupported protocol" unrepresentable instead of a runtime
/// rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DnsTransport {
    Doh,
    Dot,
}

/// DNS fallback resolver using DoH (RFC 8484) + DoT (RFC 7858).
///
/// Constructed from a [`DnsFallbackConfig`]. The resolver tries
/// DoH first (faster on modern networks where HTTPS is unblocked),
/// then falls back to DoT (port 853, may bypass HTTP-layer DPI).
pub struct DnsFallbackResolver {
    doh_resolver: TokioResolver,
    dot_resolver: TokioResolver,
    domain_suffix: String,
}

impl DnsFallbackResolver {
    /// Build a resolver from config.
    pub fn new(config: &DnsFallbackConfig) -> Result<Self> {
        let doh_resolver = Self::build_resolver(&config.doh_endpoints, DnsTransport::Doh, config)?;
        let dot_resolver = Self::build_resolver(&config.dot_endpoints, DnsTransport::Dot, config)?;
        Ok(Self {
            doh_resolver,
            dot_resolver,
            domain_suffix: config.domain_suffix.clone(),
        })
    }

    fn build_resolver(
        endpoints: &[DnsEndpoint],
        transport: DnsTransport,
        config: &DnsFallbackConfig,
    ) -> Result<TokioResolver> {
        if endpoints.is_empty() {
            return Err(NexusError::Endpoint(format!(
                "no DNS endpoints configured for transport {transport:?}"
            )));
        }

        let mut name_servers = Vec::with_capacity(endpoints.len());
        for ep in endpoints {
            // Per-endpoint TLS name (P2-E-1): each endpoint validates
            // against its own certificate name, never a global one.
            let server_name: Arc<str> = Arc::from(ep.tls_name.as_str());
            let mut conn = match transport {
                // `None` path selects the standard "/dns-query" (RFC 8484).
                DnsTransport::Doh => ConnectionConfig::https(server_name, None),
                DnsTransport::Dot => ConnectionConfig::tls(server_name),
            };
            conn.port = ep.port;
            // trust_negative_responses=false must stay EXPLICIT: the
            // 0.26 default flipped to true, and negative caching would
            // defeat the DoH/DoT race in resolve_node.
            name_servers.push(NameServerConfig::new(ep.ip, false, vec![conn]));
        }

        let resolver_config = ResolverConfig::from_parts(None, vec![], name_servers);
        let mut builder =
            Resolver::builder_with_config(resolver_config, TokioRuntimeProvider::default());
        let opts: &mut ResolverOpts = builder.options_mut();
        opts.timeout = config.timeout;
        opts.attempts = 2;

        builder
            .build()
            .map_err(|e| NexusError::Endpoint(format!("failed to build DNS resolver: {e}")))
    }

    /// Build the DNS query name for a node_id.
    pub fn build_query_name(&self, node_id_hex: &str) -> Result<String> {
        if node_id_hex.len() != 64 || !node_id_hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(NexusError::Endpoint(format!(
                "invalid node_id hex for DNS lookup: {node_id_hex:?}"
            )));
        }
        Ok(format!("{node_id_hex}.{}.", self.domain_suffix))
    }

    async fn resolve_txt_via(
        resolver: &TokioResolver,
        query: &str,
        protocol_label: &str,
    ) -> anyhow::Result<Vec<u8>> {
        let lookup = resolver
            .txt_lookup(query)
            .await
            .map_err(|e| anyhow::anyhow!("{protocol_label} TXT lookup failed for {query}: {e}"))?;

        let mut data = Vec::new();
        // hickory 0.26: txt_lookup returns the generic `Lookup`; TXT
        // payloads are extracted from the answer records' RData.
        for record in lookup.answers() {
            if let RData::TXT(txt) = &record.data {
                for segment in txt.txt_data.iter() {
                    data.extend_from_slice(segment);
                }
            }
        }
        if data.is_empty() {
            anyhow::bail!("{protocol_label} TXT lookup returned empty data for {query}");
        }
        Ok(data)
    }
}

#[async_trait]
impl DnsFallbackResolve for DnsFallbackResolver {
    fn label(&self) -> &str {
        "dns-fallback-doh-dot"
    }

    async fn resolve_node(&self, node_id_hex: &str) -> anyhow::Result<Vec<u8>> {
        let query = self
            .build_query_name(node_id_hex)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        let doh_fut = Self::resolve_txt_via(&self.doh_resolver, &query, "DoH");
        let dot_fut = Self::resolve_txt_via(&self.dot_resolver, &query, "DoT");

        tokio::pin!(doh_fut, dot_fut);

        // Race both protocols concurrently. First success wins; if the
        // first responder fails, await the remaining one before giving up.
        tokio::select! {
            doh = &mut doh_fut => match doh {
                Ok(data) => {
                    debug!(node_id = %node_id_hex, bytes = data.len(), "DNS fallback resolved via DoH");
                    Ok(data)
                }
                Err(doh_err) => {
                    debug!(node_id = %node_id_hex, error = %doh_err, "DoH failed, waiting for DoT");
                    match dot_fut.await {
                        Ok(data) => {
                            debug!(node_id = %node_id_hex, bytes = data.len(), "DNS fallback resolved via DoT");
                            Ok(data)
                        }
                        Err(dot_err) => Err(anyhow::anyhow!(
                            "DNS fallback failed for {node_id_hex}: DoH={doh_err}, DoT={dot_err}"
                        )),
                    }
                }
            },
            dot = &mut dot_fut => match dot {
                Ok(data) => {
                    debug!(node_id = %node_id_hex, bytes = data.len(), "DNS fallback resolved via DoT");
                    Ok(data)
                }
                Err(dot_err) => {
                    debug!(node_id = %node_id_hex, error = %dot_err, "DoT failed, waiting for DoH");
                    match doh_fut.await {
                        Ok(data) => {
                            debug!(node_id = %node_id_hex, bytes = data.len(), "DNS fallback resolved via DoH");
                            Ok(data)
                        }
                        Err(doh_err) => Err(anyhow::anyhow!(
                            "DNS fallback failed for {node_id_hex}: DoH={doh_err}, DoT={dot_err}"
                        )),
                    }
                }
            },
        }
    }
}

// -----------------------------------------------------------------
// Environment loader
// -----------------------------------------------------------------

/// Load DNS fallback config from environment variables.
///
/// - `SBFB_DNS_FALLBACK_ENABLED=true|1` → enabled with defaults
/// - `SBFB_DNS_FALLBACK_DOMAIN=custom.example.com` → override suffix
/// - Absent / empty → `Ok(None)` (fallback disabled, pre-S24 behaviour)
pub fn load_dns_fallback_from_env() -> Result<Option<DnsFallbackConfig>> {
    let enabled = match std::env::var(DNS_FALLBACK_ENABLED_ENV) {
        Ok(v) => v.eq_ignore_ascii_case("true") || v == "1",
        Err(_) => return Ok(None),
    };
    if !enabled {
        return Ok(None);
    }

    let mut config = DnsFallbackConfig {
        enabled: true,
        ..DnsFallbackConfig::default()
    };

    if let Ok(domain) = std::env::var(DNS_FALLBACK_DOMAIN_ENV) {
        let trimmed = domain.trim();
        if !trimmed.is_empty() {
            config.domain_suffix = trimmed.to_string();
        }
    }

    Ok(Some(config))
}

// -----------------------------------------------------------------
// Concatenate TXT record bytes (public for reuse in tests)
// -----------------------------------------------------------------

/// Concatenate raw TXT record character strings into a single byte
/// vector. DNS TXT records split payloads >255 bytes across multiple
/// character strings (RFC 1035 §3.3.14); this function reassembles
/// them in wire order.
pub fn concat_txt_strings(strings: &[Box<[u8]>]) -> Vec<u8> {
    let total: usize = strings.iter().map(|s| s.len()).sum();
    let mut out = Vec::with_capacity(total);
    for s in strings {
        out.extend_from_slice(s);
    }
    out
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_GUARD: Mutex<()> = Mutex::new(());

    // ---------------------------------------------------------
    // Config defaults
    // ---------------------------------------------------------

    #[test]
    fn config_default_has_expected_values() {
        let cfg = DnsFallbackConfig::default();
        assert!(!cfg.enabled);
        assert_eq!(cfg.domain_suffix, DEFAULT_DOMAIN_SUFFIX);
        assert_eq!(cfg.timeout, DEFAULT_DNS_TIMEOUT);
        assert_eq!(cfg.doh_endpoints.len(), 2, "Cloudflare + Google DoH");
        assert_eq!(cfg.dot_endpoints.len(), 2, "Cloudflare + Google DoT");
        assert_eq!(cfg.doh_endpoints[0].ip, DOH_CLOUDFLARE_IP);
        assert_eq!(cfg.doh_endpoints[0].port, DOH_PORT);
        assert_eq!(cfg.doh_endpoints[1].ip, DOH_GOOGLE_IP);
        assert_eq!(cfg.dot_endpoints[0].port, DOT_PORT);
    }

    // ---------------------------------------------------------
    // Query name building
    // ---------------------------------------------------------

    #[test]
    fn build_query_name_valid_hex() {
        let cfg = DnsFallbackConfig::default();
        let resolver = DnsFallbackResolver::new(&cfg).expect("build resolver");
        let id = "a".repeat(64);
        let name = resolver.build_query_name(&id).unwrap();
        assert_eq!(name, format!("{id}.{DEFAULT_DOMAIN_SUFFIX}."));
    }

    #[test]
    fn build_query_name_rejects_short_hex() {
        let cfg = DnsFallbackConfig::default();
        let resolver = DnsFallbackResolver::new(&cfg).expect("build resolver");
        let err = resolver.build_query_name("abcd").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("invalid node_id hex"), "got: {msg}");
    }

    #[test]
    fn build_query_name_rejects_non_hex() {
        let cfg = DnsFallbackConfig::default();
        let resolver = DnsFallbackResolver::new(&cfg).expect("build resolver");
        let bad = "g".repeat(64);
        let err = resolver.build_query_name(&bad).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("invalid node_id hex"), "got: {msg}");
    }

    // ---------------------------------------------------------
    // TXT concatenation
    // ---------------------------------------------------------

    #[test]
    fn concat_txt_strings_reassembles_split_payload() {
        let strings: Vec<Box<[u8]>> =
            vec![b"hello"[..].into(), b" "[..].into(), b"world"[..].into()];
        let out = concat_txt_strings(&strings);
        assert_eq!(out, b"hello world");
    }

    #[test]
    fn concat_txt_strings_empty_returns_empty() {
        let out = concat_txt_strings(&[]);
        assert!(out.is_empty());
    }

    // ---------------------------------------------------------
    // Env loading
    // ---------------------------------------------------------

    #[test]
    fn load_env_returns_none_when_unset() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_ENABLED_ENV) };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_DOMAIN_ENV) };
        let got = load_dns_fallback_from_env().expect("unset must not error");
        assert!(got.is_none());
    }

    #[test]
    fn load_env_returns_none_when_disabled() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(DNS_FALLBACK_ENABLED_ENV, "false") };
        let got = load_dns_fallback_from_env().expect("disabled must not error");
        assert!(got.is_none());
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_ENABLED_ENV) };
    }

    #[test]
    fn load_env_returns_config_when_enabled() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(DNS_FALLBACK_ENABLED_ENV, "true") };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_DOMAIN_ENV) };
        let cfg = load_dns_fallback_from_env()
            .expect("enabled must not error")
            .expect("enabled must produce Some");
        assert!(cfg.enabled);
        assert_eq!(cfg.domain_suffix, DEFAULT_DOMAIN_SUFFIX);
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_ENABLED_ENV) };
    }

    #[test]
    fn load_env_respects_custom_domain() {
        let _g = ENV_GUARD.lock().unwrap();
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(DNS_FALLBACK_ENABLED_ENV, "1") };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::set_var(DNS_FALLBACK_DOMAIN_ENV, "custom.example.com") };
        let cfg = load_dns_fallback_from_env()
            .expect("must not error")
            .expect("must produce Some");
        assert_eq!(cfg.domain_suffix, "custom.example.com");
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_ENABLED_ENV) };
        // SAFETY: test-only; nextest runs each test in its own process.
        unsafe { std::env::remove_var(DNS_FALLBACK_DOMAIN_ENV) };
    }

    // ---------------------------------------------------------
    // P2-E-1 : per-endpoint TLS name (S25 Phase A)
    // ---------------------------------------------------------

    #[test]
    fn per_endpoint_tls_name_used_doh() {
        let cfg = DnsFallbackConfig {
            doh_endpoints: vec![
                DnsEndpoint {
                    ip: DOH_CLOUDFLARE_IP,
                    port: DOH_PORT,
                    tls_name: "custom-cf.example.com".to_string(),
                },
                DnsEndpoint {
                    ip: DOH_GOOGLE_IP,
                    port: DOH_PORT,
                    tls_name: "custom-google.example.com".to_string(),
                },
            ],
            ..DnsFallbackConfig::default()
        };
        let resolver =
            DnsFallbackResolver::new(&cfg).expect("build resolver with per-endpoint TLS");
        assert_eq!(resolver.label(), "dns-fallback-doh-dot");
    }

    #[test]
    fn per_endpoint_tls_name_used_dot() {
        let cfg = DnsFallbackConfig {
            dot_endpoints: vec![
                DnsEndpoint {
                    ip: DOH_CLOUDFLARE_IP,
                    port: DOT_PORT,
                    tls_name: "dot-cf.example.com".to_string(),
                },
                DnsEndpoint {
                    ip: DOH_GOOGLE_IP,
                    port: DOT_PORT,
                    tls_name: "dot-google.example.com".to_string(),
                },
            ],
            ..DnsFallbackConfig::default()
        };
        let resolver =
            DnsFallbackResolver::new(&cfg).expect("build resolver with per-endpoint DoT TLS");
        assert_eq!(resolver.label(), "dns-fallback-doh-dot");
    }

    // `build_resolver_rejects_unsupported_protocol` was removed with
    // the hickory 0.26 bump (S82 Phase K): the local `DnsTransport`
    // enum only has Doh/Dot variants, so an unsupported protocol is
    // unrepresentable and the runtime guard it exercised is gone.

    #[test]
    fn build_resolver_rejects_empty_endpoints() {
        let cfg = DnsFallbackConfig::default();
        let err = DnsFallbackResolver::build_resolver(&[], DnsTransport::Doh, &cfg).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("no DNS endpoints"), "got: {msg}");
    }
}
