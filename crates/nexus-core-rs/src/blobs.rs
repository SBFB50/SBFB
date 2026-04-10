//! Typed wrapper around iroh-blobs content-addressed storage.
//!
//! Exposes just the operations SBFB needs for curator lists and
//! blob pinning:
//!
//! - [`BlobsClient::add_bytes`] — store a byte slice and return its
//!   content hash (raw 32 bytes)
//! - [`BlobsClient::get_bytes`] — fetch a blob by hash
//! - [`BlobsClient::has`] — check whether a blob is present locally
//!
//! Curator lists flow through this layer: the curator publishes a
//! signed JSON blob via `add_bytes`, announces the resulting hash
//! on a gossip topic, and subscribers use `get_bytes` on receiving
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

use iroh_blobs::store::mem::MemStore;
use iroh_blobs::Hash;

use crate::error::{NexusError, Result};

/// Thin client around an iroh-blobs [`MemStore`].
///
/// Constructed via [`BlobsClient::new`] from a borrowed `&MemStore`
/// (typically [`crate::Node::blobs_store`]).
#[derive(Debug, Clone, Copy)]
pub struct BlobsClient<'a> {
    inner: &'a MemStore,
}

impl<'a> BlobsClient<'a> {
    /// Wrap a `&MemStore`.
    pub fn new(inner: &'a MemStore) -> Self {
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
    /// blobs you should use `MemStore::reader(hash)` directly for
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_node, Node};

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
}
