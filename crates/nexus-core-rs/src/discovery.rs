//! Discovery helpers for SBFB.
//!
//! In iroh 0.97 the `presets::N0` preset wires pkarr DHT discovery
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
//! Sprint 4 will add:
//!
//! - Explicit resolve(node_id) for proactive peer lookup
//! - Curator list record publishing on top of pkarr signed packets

use std::collections::BTreeSet;

use iroh::{Endpoint, TransportAddr, Watcher as _};
use serde::{Deserialize, Serialize};

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
    /// return a [`NodeAddrInfo`] snapshot.
    ///
    /// In iroh 0.97 addressing is exposed as [`Endpoint::addr`]
    /// (synchronous) and [`Endpoint::watch_addr`] (watcher). The
    /// watcher's `initialized()` helper only works for Nullable
    /// watcher values; `EndpointAddr` is not Nullable, so we poll
    /// the watcher manually until its inner `addrs: BTreeSet<...>`
    /// is non-empty, then convert to our serializable snapshot.
    pub async fn my_addr(&self) -> Result<NodeAddrInfo> {
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

        let node_id = ep_addr.id.to_string();

        let mut relay_url: Option<String> = None;
        let mut direct_addresses: BTreeSet<String> = BTreeSet::new();

        for t in &ep_addr.addrs {
            match t {
                TransportAddr::Relay(url) => {
                    // Prefer the first relay URL we see. In
                    // presets::N0 there is normally exactly one.
                    if relay_url.is_none() {
                        relay_url = Some(url.to_string());
                    }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create_node;
    use std::time::Duration;
    use tokio::time::timeout;

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
}
