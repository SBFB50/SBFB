//! iroh node lifecycle (Sprint 1 scope).
//!
//! A `Node` is a handle to a running `iroh::Endpoint` — the top-level
//! iroh object that owns the QUIC transport, the Ed25519 identity and
//! the discovery services. The SBFB coordinator and workers each own
//! exactly one `Node` for the life of their process.
//!
//! ## Sprint 1 minimal API
//!
//! - [`create_node`] — boot a fresh anonymous node with a random key
//! - [`Node::node_id`] — stable Ed25519 public key as a short string
//! - [`Node::shutdown`] — graceful close of the endpoint
//!
//! Sprint 2 will add persistent key loading, pkarr publishing,
//! discovery config, custom ALPN protocols and a real-time observer
//! of the subset of iroh events the coordinator cares about.

use std::fmt;

use iroh::endpoint::presets;
use iroh::Endpoint;
use tracing::{debug, info};

use crate::error::{NexusError, Result};

/// A handle to a running iroh endpoint.
///
/// The node is alive as long as this struct is alive. Dropping it
/// without calling [`Node::shutdown`] first still closes the endpoint
/// but skips the graceful drain, which may leave in-flight QUIC
/// streams unacknowledged. Callers that care should always
/// `await node.shutdown()` before dropping.
pub struct Node {
    endpoint: Endpoint,
}

impl Node {
    /// Return the short textual form of this node's Ed25519 public
    /// key (z-base32 encoded). This is the `EndpointId` that peers
    /// use to look up the node via pkarr DHT.
    ///
    /// In iroh 0.97 the method on `Endpoint` is `id()`, returning
    /// an `EndpointId`. We render it via its `Display` impl.
    pub fn node_id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Gracefully shut down the underlying iroh endpoint.
    ///
    /// This drains in-flight QUIC connections, tears down the
    /// discovery services, and releases the UDP sockets.
    ///
    /// `Endpoint::close()` in iroh 0.97 is infallible — it takes
    /// `&self` and returns `()` after best-effort draining. Our
    /// wrapper consumes `self` so the `Node` handle cannot be
    /// accidentally reused after shutdown.
    pub async fn shutdown(self) -> Result<()> {
        debug!("shutting down iroh endpoint");
        self.endpoint.close().await;
        info!("iroh endpoint closed cleanly");
        Ok(())
    }

    /// Access the underlying iroh [`Endpoint`] for advanced callers
    /// that need features not yet wrapped by nexus-core-rs.
    ///
    /// Prefer the typed wrappers in this crate when available — this
    /// escape hatch exists so Sprint 1 prototyping and the Rust
    /// learning work can use the raw iroh API without having to
    /// re-plumb everything through nexus-core-rs first.
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("node_id", &self.node_id())
            .finish()
    }
}

/// Boot a fresh anonymous iroh node with a random Ed25519 identity.
///
/// The node binds on an OS-assigned UDP port, enables the default
/// discovery services (local network multicast + pkarr DHT) and is
/// ready to accept connections as soon as this function returns.
///
/// ## Errors
///
/// Returns [`NexusError::Endpoint`] if the iroh endpoint cannot bind
/// (most commonly because the UDP socket is not allowed by the OS or
/// the machine is offline).
///
/// ## Example
///
/// ```no_run
/// # async fn example() -> nexus_core_rs::Result<()> {
/// let node = nexus_core_rs::create_node().await?;
/// println!("node id = {}", node.node_id());
/// node.shutdown().await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_node() -> Result<Node> {
    debug!("building iroh endpoint with the N0 preset (pkarr + relay)");

    // iroh 0.97: the builder takes a `Preset` that bundles the
    // default n0 discovery (pkarr DHT) and relay configuration.
    // `presets::N0` is the canonical "just make it work" choice
    // for volunteer compute networks like SBFB.
    let endpoint = Endpoint::builder(presets::N0)
        .bind()
        .await
        .map_err(|e| NexusError::Endpoint(format!("bind failed: {e}")))?;

    info!(node_id = %endpoint.id(), "iroh endpoint ready");
    Ok(Node { endpoint })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_node_returns_a_usable_handle() {
        let node = create_node()
            .await
            .expect("create_node should boot on a standard dev machine");

        let id = node.node_id();
        assert!(!id.is_empty(), "node id must be non-empty");
        assert!(
            id.len() >= 32,
            "node id should be ~52 chars of z-base32, got {} chars",
            id.len()
        );

        node.shutdown().await.expect("shutdown should succeed");
    }

    #[tokio::test]
    async fn two_nodes_have_distinct_identities() {
        let a = create_node().await.expect("node a boots");
        let b = create_node().await.expect("node b boots");

        assert_ne!(
            a.node_id(),
            b.node_id(),
            "each create_node() call must mint a fresh Ed25519 keypair"
        );

        a.shutdown().await.ok();
        b.shutdown().await.ok();
    }
}
