// SPDX-License-Identifier: AGPL-3.0-or-later
//! Discovery helpers for SBFB.
//!
//! In iroh 1.0.1 the `presets::N0` preset wires pkarr DHT discovery
//! automatically: every node we boot publishes its current
//! [`NodeAddr`] into the pkarr DHT and subscribes to lookups for
//! node ids it tries to dial. SBFB therefore does **not** need to
//! call any explicit publish or resolve method — the protocol
//! stack handles it under the hood.
//!
//! What this module exposes is the view of our own identity that
//! the rest of SBFB (and the Python bindings) needs:
//!
//! - [`NodeAddrInfo`] — a serializable snapshot of this node's
//!   addressing information (node id, optional relay URL, direct
//!   socket addresses). Used by Sprint 4 to build invite links
//!   and by the coordinator to publish its own manifest.
//! - [`DiscoveryClient::my_addr`] — wait until the endpoint has a
//!   ready [`NodeAddr`] and convert it into a [`NodeAddrInfo`].
//!
//! Possible extensions beyond the Sprint 4 baseline (not built here):
//!
//! - Explicit resolve(node_id) for proactive peer lookup
//! - Curator list record publishing on top of pkarr signed packets

use std::collections::BTreeSet;
use std::str::FromStr;
use std::time::Duration;

use iroh::{Endpoint, EndpointAddr, EndpointId, TransportAddr, Watcher as _};
use iroh_blobs::ALPN as BLOBS_ALPN;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::error::{NexusError, Result};

/// Serializable snapshot of a node's addressing information.
///
/// This is what you embed in an invite link or a public project
/// manifest so a peer can reach this node without going through
/// the pkarr DHT. It is also what callers read out of
/// [`DiscoveryClient::my_addr`] to learn their own address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeAddrInfo {
    /// Hex-encoded 32-byte Ed25519 public key (the node's
    /// identity / endpoint id).
    pub node_id: String,
    /// Optional home relay URL (iroh relay network). Always set
    /// when the node uses the default `presets::N0` preset.
    pub relay_url: Option<String>,
    /// Direct UDP socket addresses the node is listening on
    /// (local + NAT-mapped). Stored as stringified `SocketAddr`
    /// values for language-neutral serialization.
    pub direct_addresses: Vec<String>,
}

/// Thin client around an [`iroh::Endpoint`] for SBFB-facing
/// discovery operations.
#[derive(Debug, Clone, Copy)]
pub struct DiscoveryClient<'a> {
    endpoint: &'a Endpoint,
}

impl<'a> DiscoveryClient<'a> {
    /// Wrap an `&Endpoint` (typically [`crate::Node::endpoint`]).
    pub fn new(endpoint: &'a Endpoint) -> Self {
        DiscoveryClient { endpoint }
    }

    /// Return this node's Ed25519 public key as a 64-char hex
    /// string — the same value as [`crate::Node::node_id`].
    pub fn my_node_id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Wait until the endpoint has a ready [`EndpointAddr`] and
    /// return the raw iroh [`EndpointAddr`] (with relay URL and
    /// direct socket addresses populated).
    ///
    /// This is the primitive used by [`DiscoveryClient::my_addr`]
    /// and by Sprint 3+ code that mints `BlobTicket` / `DocTicket`
    /// values. Most SBFB code should prefer `my_addr()` which
    /// returns a serializable snapshot.
    pub async fn my_endpoint_addr(&self) -> Result<EndpointAddr> {
        let mut watcher = self.endpoint.watch_addr();

        // Grab the current snapshot. If iroh already has at least
        // one relay/direct address we are done in zero awaits.
        let mut ep_addr = watcher.get();

        // Otherwise block on the next update() and repeat. We cap
        // the number of iterations to protect against a watcher
        // that never produces a non-empty value.
        for _ in 0..20 {
            if !ep_addr.addrs.is_empty() {
                break;
            }
            watcher
                .updated()
                .await
                .map_err(|e| NexusError::Discovery(format!("watcher disconnected: {e}")))?;
            ep_addr = watcher.get();
        }

        if ep_addr.addrs.is_empty() {
            return Err(NexusError::Discovery(
                "endpoint address set never populated after 20 update iterations".into(),
            ));
        }

        Ok(ep_addr)
    }

    /// Wait until the endpoint has a ready [`EndpointAddr`] and
    /// return a [`NodeAddrInfo`] snapshot.
    ///
    /// In iroh 1.0.1 addressing is exposed as [`Endpoint::addr`]
    /// (synchronous) and [`Endpoint::watch_addr`] (watcher). This
    /// helper delegates to [`DiscoveryClient::my_endpoint_addr`]
    /// and then converts the raw `EndpointAddr` into our
    /// serializable snapshot.
    pub async fn my_addr(&self) -> Result<NodeAddrInfo> {
        let ep_addr = self.my_endpoint_addr().await?;

        let node_id = ep_addr.id.to_string();

        let mut relay_url: Option<String> = None;
        let mut direct_addresses: BTreeSet<String> = BTreeSet::new();

        for t in &ep_addr.addrs {
            match t {
                // Prefer the first relay URL we see. In
                // presets::N0 there is normally exactly one.
                TransportAddr::Relay(url) if relay_url.is_none() => {
                    relay_url = Some(url.to_string());
                }
                TransportAddr::Ip(sa) => {
                    direct_addresses.insert(sa.to_string());
                }
                _ => {}
            }
        }

        Ok(NodeAddrInfo {
            node_id,
            relay_url,
            direct_addresses: direct_addresses.into_iter().collect(),
        })
    }

    /// Probe whether a remote endpoint id is reachable **right
    /// now** by attempting to dial it over the iroh-blobs ALPN
    /// under a wall-clock timeout.
    ///
    /// Sprint 7 Phase D plan R1: `Endpoint::lookup(id)` is not
    /// yet wrapped by this crate, so the browse reachability
    /// check falls back on `Endpoint::connect(id, ALPN)` with a
    /// short timeout. Because every SBFB node accepts the blobs
    /// protocol at boot (see [`crate::node::create_node`]), a
    /// connect probe to [`iroh_blobs::ALPN`] is the least
    /// invasive liveness check — it triggers the full pkarr DHT
    /// lookup + relay dial + direct-path race and reports back
    /// whatever iroh comes up with.
    ///
    /// Arguments:
    ///
    /// - `endpoint_id_hex` — 64-char lowercase hex of the peer's
    ///   Ed25519 public key. Accepted format is what
    ///   [`crate::Node::node_id`] returns.
    /// - `timeout_duration` — hard deadline. A 2 s probe is the
    ///   right default: fast enough to keep the browse UI
    ///   responsive, slow enough to absorb a pkarr round-trip
    ///   under typical residential NAT conditions.
    ///
    /// Returns:
    ///
    /// - `Ok(true)` — `Endpoint::connect` returned a live
    ///   connection before the deadline. The connection is
    ///   immediately dropped; the probe does not keep any state
    ///   and does not open any bi-directional stream.
    /// - `Ok(false)` — `Endpoint::connect` returned an error
    ///   (peer not found, relay refused, ALPN mismatch, …) OR
    ///   the deadline elapsed. Both cases collapse to the same
    ///   "unreachable" bucket because the shell's Browse page
    ///   UX does not distinguish.
    /// - `Err(NexusError::Discovery(..))` — the input hex could
    ///   not be parsed as an `EndpointId`. A malformed input is
    ///   a caller bug, not a network condition, so it surfaces
    ///   as a hard error instead of collapsing to `false`.
    pub async fn probe_reachable(
        &self,
        endpoint_id_hex: &str,
        timeout_duration: Duration,
    ) -> Result<bool> {
        let endpoint_id = EndpointId::from_str(endpoint_id_hex).map_err(|e| {
            NexusError::Discovery(format!("bad endpoint id hex {endpoint_id_hex}: {e}"))
        })?;

        let connect_fut = self.endpoint.connect(endpoint_id, BLOBS_ALPN);
        match timeout(timeout_duration, connect_fut).await {
            Ok(Ok(_conn)) => Ok(true),
            Ok(Err(_dial_err)) => Ok(false),
            Err(_elapsed) => Ok(false),
        }
    }
}

/// What an address this node hands out — in a doc ticket, a blob ticket,
/// an announcement — may say about where it lives (chat plan, Phase I).
///
/// The address reaches whoever the ticket reaches: an invitation link or
/// QR code, every node of a gossip topic. A connected peer still learns
/// the direct paths in-band once the connection is up; this only governs
/// what is published ahead of any connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AddrPolicy {
    /// Relay URLs and non-global IP addresses only (loopback, private
    /// IPv4, IPv4 link-local, IPv6 unique-local). The default: a public IP
    /// is never published, and a LAN peer still connects directly. A host
    /// whose interfaces carry global addresses only hands out relay-only
    /// tickets.
    #[default]
    Discreet,
    /// Every address, public IPs included. For an always-on node whose
    /// public address is the point (an anchor on a VPS).
    Full,
}

/// The addresses of `addr` that `policy` lets out. Allow-list, not
/// block-list: an address of a kind not named here (a global or CGNAT
/// IP, a documentation range, a custom transport) is dropped under
/// [`AddrPolicy::Discreet`].
pub fn discreet_addr(addr: &EndpointAddr, policy: AddrPolicy) -> EndpointAddr {
    EndpointAddr {
        id: addr.id,
        addrs: addr
            .addrs
            .iter()
            .filter(|t| policy.keeps(t))
            .cloned()
            .collect(),
    }
}

impl AddrPolicy {
    /// Whether `addr` may leave the node under this policy — the one rule
    /// behind [`discreet_addr`] and [`AddrPolicy::gossip_filter`].
    pub fn keeps(self, addr: &TransportAddr) -> bool {
        match (self, addr) {
            (AddrPolicy::Full, _) => true,
            (AddrPolicy::Discreet, TransportAddr::Relay(_)) => true,
            (AddrPolicy::Discreet, TransportAddr::Ip(sa)) => is_non_global_ip(sa.ip()),
            (AddrPolicy::Discreet, _) => false,
        }
    }

    /// The same rule for the peer data iroh-gossip sends about this node:
    /// it travels beyond direct neighbors (join forwarding, shuffles) to
    /// nodes this one never connected to.
    pub fn gossip_filter(self) -> iroh::address_lookup::AddrFilter {
        iroh::address_lookup::AddrFilter::new(move |addrs| {
            std::borrow::Cow::Owned(addrs.iter().filter(|a| self.keeps(a)).cloned().collect())
        })
    }
}

/// Loopback, private IPv4 (RFC 1918), IPv4 link-local, or IPv6
/// unique-local. The shared CGNAT range `100.64.0.0/10` is NOT private
/// here: it is the carrier's address space, shared by its subscribers.
/// IPv6 link-local is not kept either: iroh never advertises one, its
/// zone id is meaningless on another host, and an EUI-64 interface id
/// embeds the MAC address.
fn is_non_global_ip(ip: std::net::IpAddr) -> bool {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => is_non_global_ip(IpAddr::V4(v4)),
            None => v6.is_loopback() || v6.is_unique_local(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_node;

    #[test]
    fn discreet_addr_drops_global_ips() {
        let id = EndpointId::from_bytes(&crate::KeyPair::generate().public_bytes()).unwrap();
        let relay: iroh::RelayUrl = "https://relay.sbfb.invalid./".parse().unwrap();
        let ip = |s: &str| TransportAddr::Ip(s.parse().unwrap());
        let kept = [
            ip("127.0.0.1:1"),
            ip("[::1]:1"),
            ip("192.168.1.2:1"),
            ip("10.0.0.1:1"),
            ip("172.16.5.4:1"),
            ip("169.254.3.3:1"),
            ip("[fd00::1]:1"),
            ip("[::ffff:192.168.1.9]:1"),
            TransportAddr::Relay(relay),
        ];
        let dropped = [
            ip("8.8.8.8:1"),
            ip("[2001:4860::1]:1"),
            ip("100.64.1.1:1"),
            ip("100.127.255.254:1"),
            ip("198.51.100.7:1"),
            ip("[::ffff:8.8.8.8]:1"),
            ip("0.0.0.0:1"),
            ip("[fe80::1%3]:1"),
        ];
        let all = EndpointAddr {
            id,
            addrs: kept.iter().chain(dropped.iter()).cloned().collect(),
        };

        let out = discreet_addr(&all, AddrPolicy::Discreet);
        assert_eq!(out.id, id);
        assert_eq!(out.addrs, kept.iter().cloned().collect());
        assert_eq!(discreet_addr(&all, AddrPolicy::Full), all);
        assert_eq!(AddrPolicy::default(), AddrPolicy::Discreet);
    }

    #[tokio::test]
    async fn my_node_id_is_stable() {
        let node = create_node().await.unwrap();
        let disco = DiscoveryClient::new(node.endpoint());

        let id1 = disco.my_node_id();
        let id2 = disco.my_node_id();
        assert_eq!(id1, id2);
        assert_eq!(id1, node.node_id());

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn my_addr_returns_address_info_within_reasonable_time() {
        let node = create_node().await.unwrap();
        let disco = DiscoveryClient::new(node.endpoint());

        // On a dev machine the n0 preset gets a relay URL within
        // a few hundred ms and direct addresses shortly after.
        // Cap the wait at 15s to keep the test bounded; in CI
        // the relay can take a bit longer.
        let info = timeout(Duration::from_secs(15), disco.my_addr())
            .await
            .expect("my_addr should resolve within 15 seconds")
            .unwrap();

        assert_eq!(info.node_id, node.node_id());
        assert!(
            info.relay_url.is_some() || !info.direct_addresses.is_empty(),
            "a ready NodeAddr must carry at least a relay url or a direct address"
        );

        node.shutdown().await.ok();
    }

    #[test]
    fn node_addr_info_serde_roundtrip() {
        let info = NodeAddrInfo {
            node_id: "abc123".into(),
            relay_url: Some("https://euc1-1.relay.n0.iroh-canary.iroh.link".into()),
            direct_addresses: vec!["192.168.1.10:12345".into(), "10.0.0.1:12345".into()],
        };
        let json = serde_json::to_string(&info).unwrap();
        let back: NodeAddrInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info, back);
    }

    #[tokio::test]
    async fn probe_reachable_rejects_malformed_hex() {
        // A bad hex string is a caller bug, not a network
        // condition — surfaces as `Err`, not `Ok(false)`, so
        // the browse aggregator can distinguish the two.
        let node = create_node().await.unwrap();
        let disco = DiscoveryClient::new(node.endpoint());
        let err = disco
            .probe_reachable("not-hex-at-all", Duration::from_millis(100))
            .await;
        assert!(err.is_err());
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn probe_reachable_returns_false_for_random_unknown_id() {
        // A fresh node id we never publish anywhere will fail
        // pkarr resolution. Depending on the iroh dialer the
        // connect call may either error fast or hit the timeout
        // — both collapse to `Ok(false)` here.
        let node = create_node().await.unwrap();
        let disco = DiscoveryClient::new(node.endpoint());
        let unknown = "0".repeat(64); // never minted, never advertised
        let reachable = disco
            .probe_reachable(&unknown, Duration::from_millis(500))
            .await
            .expect("hex is well-formed so the call must return Ok");
        assert!(
            !reachable,
            "an unadvertised node id must never be reachable"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn probe_reachable_finds_a_seeded_local_peer() {
        // Two nodes in the same process: node A exposes its
        // EndpointAddr, we seed it into node B's address
        // lookup, and then node B's probe_reachable must
        // succeed because the dial can resolve A without
        // needing pkarr.
        let node_a = create_node().await.unwrap();
        let node_b = create_node().await.unwrap();

        // Publish A's address into B's memory_lookup — this is
        // exactly the same trick `blobs.rs::fetch_ticket` uses
        // to dial a peer off a BlobTicket without DHT traffic.
        let a_addr = DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node A must publish its address");
        node_b.memory_lookup().add_endpoint_info(a_addr);

        let disco_b = DiscoveryClient::new(node_b.endpoint());
        let reachable = disco_b
            .probe_reachable(&node_a.node_id(), Duration::from_secs(5))
            .await
            .expect("probe must not error when target is seeded");
        assert!(
            reachable,
            "node A should be reachable from node B after seeding memory_lookup"
        );

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
