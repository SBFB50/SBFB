// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed wrapper around iroh-blobs content-addressed storage.
//!
//! Exposes just the operations SBFB needs for curator lists and
//! blob pinning:
//!
//! - [`BlobsClient::add_bytes`] — store a byte slice and return its
//!   content hash (raw 32 bytes)
//! - [`BlobsClient::get_bytes`] — fetch a blob by hash
//! - [`BlobsClient::has`] — check whether a blob is present locally
//! - [`BlobsClient::fetch_ticket`] — download a blob from a remote
//!   peer via a [`iroh_blobs::ticket::BlobTicket`] string
//!
//! Curator lists flow through this layer: the curator publishes a
//! signed JSON blob via `add_bytes`, announces the resulting ticket
//! on a gossip topic, and subscribers use `fetch_ticket` on receiving
//! the announcement to pull the full list content.
//!
//! ## Example
//!
//! ```no_run
//! # async fn example(node: &nexus_core_rs::Node) -> nexus_core_rs::Result<()> {
//! use nexus_core_rs::blobs::BlobsClient;
//!
//! let blobs = BlobsClient::new(node.blobs_store());
//! let hash = blobs.add_bytes(b"hello blobs").await?;
//! assert_eq!(hash.len(), 32);
//!
//! let data = blobs.get_bytes(hash).await?;
//! assert_eq!(data, b"hello blobs");
//! # Ok(())
//! # }
//! ```

use std::str::FromStr;

use iroh::Endpoint;
use iroh::address_lookup::memory::MemoryLookup;
use iroh_blobs::Hash;
// Re-exported (see `lib.rs`): `Store` is already reachable through the
// public `BlobsClient::new(&Store)` signature and `Node::blobs_store`, so
// surfacing the name lets crates that only depend on nexus-core-rs (e.g.
// nexus-worker-core, which has no direct iroh-blobs dependency) hold an
// owned `Store` clone without importing iroh-blobs.
pub use iroh_blobs::api::Store;
use iroh_blobs::api::downloader::Downloader;
use iroh_blobs::ticket::BlobTicket;

use crate::error::{NexusError, Result};

// Re-export BlobTicket so downstream callers (including the Python
// bindings) don't need iroh-blobs as a direct dependency.
pub use iroh_blobs::ticket::BlobTicket as BlobsTicket;

/// Thin client around an iroh-blobs [`Store`].
///
/// Constructed via [`BlobsClient::new`] from a borrowed `&Store`
/// (typically [`crate::Node::blobs_store`]).
#[derive(Debug, Clone, Copy)]
pub struct BlobsClient<'a> {
    inner: &'a Store,
}

impl<'a> BlobsClient<'a> {
    /// Wrap a `&Store`.
    pub fn new(inner: &'a Store) -> Self {
        BlobsClient { inner }
    }

    /// Store a byte slice and return its BLAKE3 content hash.
    ///
    /// The returned `[u8; 32]` is the raw hash; pass it back to
    /// [`BlobsClient::get_bytes`] to retrieve the data. The blob
    /// is pinned with a named tag equal to the hex of its hash
    /// so the in-memory store does not garbage-collect it under
    /// the caller's feet.
    pub async fn add_bytes(&self, data: impl AsRef<[u8]>) -> Result<[u8; 32]> {
        let bytes = data.as_ref().to_vec();
        // iroh-blobs 0.99: add_bytes() returns a TagInfo where
        // `.hash` is a field (not a method).
        let tag_info = self
            .inner
            .blobs()
            .add_bytes(bytes)
            .await
            .map_err(|e| NexusError::Blobs(format!("add_bytes failed: {e}")))?;
        Ok(*tag_info.hash.as_bytes())
    }

    /// Retrieve a blob by its hash.
    ///
    /// Returns the full blob content as `Vec<u8>`. For very large
    /// blobs you should use `Store::blobs().reader(hash)` for
    /// streaming access; this wrapper is intentionally simple.
    pub async fn get_bytes(&self, hash: [u8; 32]) -> Result<Vec<u8>> {
        let hash = Hash::from_bytes(hash);
        let bytes = self
            .inner
            .blobs()
            .get_bytes(hash)
            .await
            .map_err(|e| NexusError::Blobs(format!("get_bytes failed: {e}")))?;
        Ok(bytes.to_vec())
    }

    /// Return true if the given hash is present in the local
    /// store (i.e. the data has been added or downloaded).
    pub async fn has(&self, hash: [u8; 32]) -> Result<bool> {
        let hash = Hash::from_bytes(hash);
        self.inner
            .blobs()
            .has(hash)
            .await
            .map_err(|e| NexusError::Blobs(format!("has failed: {e}")))
    }

    /// Download a blob from a remote peer using a [`BlobTicket`]
    /// string.
    ///
    /// The ticket carries the provider's [`iroh_base::EndpointAddr`],
    /// the content hash, and the blob format. This method:
    ///
    /// 1. Parses the ticket string.
    /// 2. Seeds the provided [`MemoryLookup`] with the ticket's
    ///    `EndpointAddr` so the endpoint can dial the peer without
    ///    needing pkarr/DNS discovery.
    /// 3. Spawns a [`Downloader`] against the given `endpoint` +
    ///    this client's store, and runs the download to completion.
    ///
    /// On success the blob bytes are now present in the local store
    /// and the returned `[u8; 32]` is the raw content hash (which
    /// equals `ticket.hash()` for single-blob tickets). Feed the
    /// hash back to [`BlobsClient::get_bytes`] to retrieve the
    /// content.
    ///
    /// SBFB curator list flow: workers subscribe to a gossip topic,
    /// receive a `BlobTicket` string in the next `GossipEvent`, call
    /// `fetch_ticket(endpoint, memory_lookup, ticket_str)`, then
    /// `get_bytes(hash)` to parse the curator list JSON.
    pub async fn fetch_ticket(
        &self,
        endpoint: &Endpoint,
        memory_lookup: &MemoryLookup,
        ticket_str: &str,
    ) -> Result<[u8; 32]> {
        let ticket = BlobTicket::from_str(ticket_str)
            .map_err(|e| NexusError::Blobs(format!("invalid blob ticket: {e}")))?;

        let (addr, hash, _format) = ticket.into_parts();
        let endpoint_id = addr.id;

        // Seed the address lookup with the provider's addr so the
        // downloader's dial attempt can resolve endpoint_id.
        memory_lookup.add_endpoint_info(addr);

        let downloader = Downloader::new(self.inner, endpoint);
        downloader
            .download(hash, vec![endpoint_id])
            .await
            .map_err(|e| NexusError::Blobs(format!("download failed: {e}")))?;

        Ok(*hash.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, create_node};

    async fn spawn_node() -> Node {
        create_node().await.expect("boot")
    }

    #[tokio::test]
    async fn add_then_get_roundtrip() {
        let node = spawn_node().await;
        let blobs = BlobsClient::new(node.blobs_store());

        let payload = b"nexus grid test blob content";
        let hash = blobs.add_bytes(payload).await.unwrap();
        assert_eq!(hash.len(), 32);

        let fetched = blobs.get_bytes(hash).await.unwrap();
        assert_eq!(fetched, payload);

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn has_returns_true_after_add() {
        let node = spawn_node().await;
        let blobs = BlobsClient::new(node.blobs_store());

        let hash = blobs.add_bytes(b"has-check").await.unwrap();
        assert!(blobs.has(hash).await.unwrap());

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn has_returns_false_for_unknown_hash() {
        let node = spawn_node().await;
        let blobs = BlobsClient::new(node.blobs_store());

        // All-zeros hash is never produced for real data.
        let unknown = [0u8; 32];
        assert!(!blobs.has(unknown).await.unwrap());

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn same_content_yields_same_hash() {
        let node = spawn_node().await;
        let blobs = BlobsClient::new(node.blobs_store());

        let h1 = blobs.add_bytes(b"dedup me").await.unwrap();
        let h2 = blobs.add_bytes(b"dedup me").await.unwrap();
        assert_eq!(h1, h2, "content-addressed: same bytes -> same hash");

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn two_nodes_fetch_blob_via_ticket() {
        // Regression test for Sprint 2 audit S7 finding: the
        // wrapper had no way to fetch a blob announced on gossip
        // by its ticket. This test exercises the full curator
        // list flow: node A adds a blob and mints a ticket, node
        // B parses the ticket and downloads the blob into its
        // own store.
        use iroh_blobs::BlobFormat;

        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        // Node A stores the curator list payload locally.
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let payload = b"curator-list-content-v1".to_vec();
        let hash_bytes = blobs_a.add_bytes(&payload).await.unwrap();

        // Mint a BlobTicket that embeds node A's current
        // EndpointAddr (relay + direct addrs) so node B can dial
        // A without pkarr/DNS discovery. Wait until node A has at
        // least one address registered before minting.
        let my_addr = crate::discovery::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node A should publish its address");

        let ticket = BlobTicket::new(my_addr, Hash::from_bytes(hash_bytes), BlobFormat::Raw);
        let ticket_str = ticket.to_string();

        // Node B fetches via the ticket.
        let blobs_b = BlobsClient::new(node_b.blobs_store());
        let fetched_hash = blobs_b
            .fetch_ticket(node_b.endpoint(), node_b.memory_lookup(), &ticket_str)
            .await
            .expect("fetch_ticket should succeed on loopback");
        assert_eq!(
            fetched_hash, hash_bytes,
            "returned hash must match ticket hash"
        );

        // The blob is now in node B's local store.
        let got = blobs_b.get_bytes(fetched_hash).await.unwrap();
        assert_eq!(got, payload, "downloaded content matches source");

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
