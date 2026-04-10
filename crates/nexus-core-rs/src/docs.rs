//! Typed wrapper around iroh-docs.
//!
//! Presents the `iroh_docs::protocol::Docs` and `iroh_docs::api::Doc`
//! client types through a slim SBFB-facing API that converts all
//! errors to [`NexusError::Docs`] and keeps the rest of the crate
//! (plus the Python bindings) independent from the specific
//! iroh-docs version we currently depend on.
//!
//! ## Scope
//!
//! Sprint 2 wraps the surface SBFB actually consumes:
//!
//! - author management (create, default, list)
//! - document lifecycle (create, import from ticket,
//!   import_and_subscribe, open, list, drop)
//! - entry IO on a single doc (set_bytes, get_exact, subscribe,
//!   share)
//!
//! Everything else in iroh-docs stays reachable via [`Node::docs`]
//! for advanced callers that want to work with the raw handle.
//!
//! ## Example
//!
//! ```no_run
//! # async fn example(node: &nexus_core_rs::Node) -> nexus_core_rs::Result<()> {
//! use nexus_core_rs::docs::DocsClient;
//!
//! let docs = DocsClient::new(node.docs());
//! let author = docs.author_create().await?;
//! let doc = docs.create_doc().await?;
//!
//! let _hash = doc.set(author, b"greeting", b"hello").await?;
//! if let Some(entry) = doc.get_exact(author, b"greeting").await? {
//!     println!("key: {:?}", entry.key());
//! }
//! # Ok(())
//! # }
//! ```

use futures_lite::{Stream, StreamExt};
use iroh_docs::api::protocol::{AddrInfoOptions, ShareMode};
use iroh_docs::api::Doc as IrohDoc;
use iroh_docs::engine::LiveEvent;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{AuthorId, DocTicket, NamespaceId};

use crate::error::{NexusError, Result};

/// Thin client around an [`iroh_docs::protocol::Docs`] handle.
///
/// Construct via [`DocsClient::new`] from a borrowed `&Docs`. The
/// client is `Copy`-ish (it only holds a reference), so pass it
/// around freely.
#[derive(Debug, Clone, Copy)]
pub struct DocsClient<'a> {
    inner: &'a Docs,
}

impl<'a> DocsClient<'a> {
    /// Wrap a `&Docs` (typically obtained from
    /// [`crate::Node::docs`]).
    pub fn new(inner: &'a Docs) -> Self {
        DocsClient { inner }
    }

    // ------------------------------------------------------------------
    // Authors
    // ------------------------------------------------------------------

    /// Create a fresh document author and return its id.
    ///
    /// Save the returned `AuthorId` somewhere stable — SBFB
    /// coordinators persist it alongside their node secret key
    /// so writes keep their identity across restarts.
    pub async fn author_create(&self) -> Result<AuthorId> {
        self.inner
            .author_create()
            .await
            .map_err(|e| NexusError::Docs(format!("author_create failed: {e}")))
    }

    /// Return the node's default author, creating it on first call
    /// for persistent nodes.
    pub async fn author_default(&self) -> Result<AuthorId> {
        self.inner
            .author_default()
            .await
            .map_err(|e| NexusError::Docs(format!("author_default failed: {e}")))
    }

    // ------------------------------------------------------------------
    // Documents
    // ------------------------------------------------------------------

    /// Create a new empty document and return a handle to it.
    pub async fn create_doc(&self) -> Result<DocHandle> {
        let doc = self
            .inner
            .create()
            .await
            .map_err(|e| NexusError::Docs(format!("create failed: {e}")))?;
        Ok(DocHandle { inner: doc })
    }

    /// Import a document from a [`DocTicket`] and join the peers
    /// it contains.
    ///
    /// Use this on the receiving side of a share: one node calls
    /// [`DocHandle::share`] to produce a ticket, sends it to the
    /// other node (out of band or via a gossip topic), and that
    /// node calls `import_ticket(ticket)` to start syncing.
    pub async fn import_ticket(&self, ticket: DocTicket) -> Result<DocHandle> {
        let doc = self
            .inner
            .import(ticket)
            .await
            .map_err(|e| NexusError::Docs(format!("import failed: {e}")))?;
        Ok(DocHandle { inner: doc })
    }

    /// Import a ticket AND create a LiveEvent subscription in the
    /// same call, so there is no window between import and subscribe
    /// where the first few entries could be missed.
    pub async fn import_and_subscribe(
        &self,
        ticket: DocTicket,
    ) -> Result<(DocHandle, impl Stream<Item = Result<LiveEvent>>)> {
        let (doc, stream) = self
            .inner
            .import_and_subscribe(ticket)
            .await
            .map_err(|e| NexusError::Docs(format!("import_and_subscribe failed: {e}")))?;

        // Lift the inner stream error type into our NexusError.
        let stream = stream
            .map(|ev| ev.map_err(|e| NexusError::Docs(format!("live event stream error: {e}"))));

        Ok((DocHandle { inner: doc }, stream))
    }

    /// Open an existing document by its namespace id.
    ///
    /// Returns `Ok(None)` if the document is not present on this
    /// node.
    pub async fn open_doc(&self, id: NamespaceId) -> Result<Option<DocHandle>> {
        let maybe = self
            .inner
            .open(id)
            .await
            .map_err(|e| NexusError::Docs(format!("open failed: {e}")))?;
        Ok(maybe.map(|inner| DocHandle { inner }))
    }

    /// Drop a document from the local node.
    ///
    /// Destructive: permanently removes the namespace secret key
    /// and all local entries. Content blobs are subject to GC
    /// unless referenced elsewhere.
    pub async fn drop_doc(&self, id: NamespaceId) -> Result<()> {
        self.inner
            .drop_doc(id)
            .await
            .map_err(|e| NexusError::Docs(format!("drop_doc failed: {e}")))
    }

    /// List all document authors for which this node holds the
    /// secret key (i.e. authors that can still produce writes).
    ///
    /// Collects the streaming response into a `Vec<AuthorId>` for
    /// ergonomic consumption from Python.
    pub async fn author_list(&self) -> Result<Vec<AuthorId>> {
        let stream = self
            .inner
            .author_list()
            .await
            .map_err(|e| NexusError::Docs(format!("author_list failed: {e}")))?;
        tokio::pin!(stream);
        let mut out = Vec::new();
        while let Some(res) = stream.next().await {
            out.push(res.map_err(|e| NexusError::Docs(format!("author_list stream error: {e}")))?);
        }
        Ok(out)
    }

    /// List every open document on this node (namespace ids only).
    ///
    /// The underlying iroh-docs stream also yields a
    /// `CapabilityKind`, but the wrapper drops it because SBFB
    /// callers only need to iterate namespaces. Use
    /// [`DocsClient::open_doc`] on each id to get a handle.
    pub async fn list_docs(&self) -> Result<Vec<NamespaceId>> {
        let stream = self
            .inner
            .list()
            .await
            .map_err(|e| NexusError::Docs(format!("list failed: {e}")))?;
        tokio::pin!(stream);
        let mut out = Vec::new();
        while let Some(res) = stream.next().await {
            let (id, _cap) =
                res.map_err(|e| NexusError::Docs(format!("list stream error: {e}")))?;
            out.push(id);
        }
        Ok(out)
    }
}

/// Handle to a single open document.
///
/// Wraps [`iroh_docs::api::Doc`] with an API that always returns
/// [`NexusError::Docs`] on failure.
#[derive(Debug, Clone)]
pub struct DocHandle {
    inner: IrohDoc,
}

impl DocHandle {
    /// Return the namespace id of this document.
    pub fn id(&self) -> NamespaceId {
        self.inner.id()
    }

    /// Borrow the underlying `iroh_docs::api::Doc` for advanced
    /// operations this wrapper doesn't cover yet.
    pub fn inner(&self) -> &IrohDoc {
        &self.inner
    }

    // ------------------------------------------------------------------
    // Writes
    // ------------------------------------------------------------------

    /// Write a key/value entry to the document under `author`.
    ///
    /// Returns the BLAKE3 hash (as raw 32 bytes) of the stored
    /// value. The hash is useful for correlating later LiveEvents
    /// or for dedupe when pushing the same value twice.
    pub async fn set(
        &self,
        author: AuthorId,
        key: impl Into<Vec<u8>>,
        value: impl Into<Vec<u8>>,
    ) -> Result<[u8; 32]> {
        let hash = self
            .inner
            .set_bytes(author, key.into(), value.into())
            .await
            .map_err(|e| NexusError::Docs(format!("set_bytes failed: {e}")))?;
        Ok(*hash.as_bytes())
    }

    // ------------------------------------------------------------------
    // Reads
    // ------------------------------------------------------------------

    /// Read a single entry by `(author, key)`.
    ///
    /// Returns `Ok(None)` if the entry doesn't exist, `Ok(Some(entry))`
    /// if it does. `include_empty` is wired to `false` so deletion
    /// tombstones do not show up here.
    pub async fn get_exact(
        &self,
        author: AuthorId,
        key: impl AsRef<[u8]>,
    ) -> Result<Option<iroh_docs::Entry>> {
        self.inner
            .get_exact(author, key.as_ref(), false)
            .await
            .map_err(|e| NexusError::Docs(format!("get_exact failed: {e}")))
    }

    /// Return every entry whose key starts with `prefix`, from
    /// any author.
    ///
    /// Used by SBFB workers to scan task-doc entries by the
    /// `"task:"` / `"claim:"` / `"result:"` prefixes. Collects
    /// the streaming response into a `Vec<Entry>` so Python
    /// callers can iterate ergonomically; for documents with
    /// thousands of entries consider using `inner().get_many(...)`
    /// directly for streaming.
    pub async fn get_many_by_prefix(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<Vec<iroh_docs::Entry>> {
        let query = Query::key_prefix(prefix);
        let stream = self
            .inner
            .get_many(query)
            .await
            .map_err(|e| NexusError::Docs(format!("get_many failed: {e}")))?;
        tokio::pin!(stream);
        let mut out = Vec::new();
        while let Some(res) = stream.next().await {
            out.push(res.map_err(|e| NexusError::Docs(format!("get_many stream error: {e}")))?);
        }
        Ok(out)
    }

    // ------------------------------------------------------------------
    // Live sync
    // ------------------------------------------------------------------

    /// Subscribe to all [`LiveEvent`]s on this document.
    ///
    /// The returned stream yields `NexusError::Docs` on any
    /// underlying iroh error. SBFB coordinators consume this
    /// stream to observe new results arriving from workers.
    pub async fn subscribe(&self) -> Result<impl Stream<Item = Result<LiveEvent>> + Send + Unpin> {
        let stream = self
            .inner
            .subscribe()
            .await
            .map_err(|e| NexusError::Docs(format!("subscribe failed: {e}")))?;
        Ok(stream
            .map(|ev| ev.map_err(|e| NexusError::Docs(format!("live event stream error: {e}")))))
    }

    // ------------------------------------------------------------------
    // Sharing
    // ------------------------------------------------------------------

    /// Produce a write-capable [`DocTicket`] that another node can
    /// pass to [`DocsClient::import_ticket`] to join this document
    /// with full write access.
    ///
    /// Uses `AddrInfoOptions::RelayAndAddresses` so the ticket
    /// carries both the relay URL and direct addresses — maximum
    /// chance of a successful connection from behind NAT.
    pub async fn share_write(&self) -> Result<DocTicket> {
        self.inner
            .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
            .await
            .map_err(|e| NexusError::Docs(format!("share write failed: {e}")))
    }

    /// Produce a read-only [`DocTicket`].
    pub async fn share_read(&self) -> Result<DocTicket> {
        self.inner
            .share(ShareMode::Read, AddrInfoOptions::RelayAndAddresses)
            .await
            .map_err(|e| NexusError::Docs(format!("share read failed: {e}")))
    }
}

// Re-export the iroh-docs types that appear in our public API so
// downstream callers don't need to add iroh-docs as a direct dep.
pub use iroh_docs::{
    AuthorId as DocsAuthorId, DocTicket as DocsTicket, NamespaceId as DocsNamespaceId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_node, Node};
    use std::time::Duration;
    use tokio::time::timeout;

    async fn spawn_node() -> Node {
        create_node()
            .await
            .expect("create_node should boot on a standard dev machine")
    }

    #[tokio::test]
    async fn author_create_returns_distinct_ids() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());

        let a = docs.author_create().await.unwrap();
        let b = docs.author_create().await.unwrap();

        assert_ne!(a, b, "two author_create calls must yield distinct ids");
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn create_doc_and_set_get_roundtrip() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        let hash = doc.set(author, b"k".to_vec(), b"v".to_vec()).await.unwrap();
        assert_eq!(hash.len(), 32);

        let entry = doc.get_exact(author, b"k").await.unwrap();
        assert!(entry.is_some(), "entry must be retrievable");

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn get_many_by_prefix_returns_matching_entries_only() {
        // Regression test for Sprint 2 audit S5 finding: the
        // wrapper lacked a prefix-scan, which is a prerequisite
        // for the Sprint 3 worker that iterates "task:*" /
        // "claim:*" / "result:*" entries.
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        doc.set(author, b"task:001".to_vec(), b"payload-1".to_vec())
            .await
            .unwrap();
        doc.set(author, b"task:002".to_vec(), b"payload-2".to_vec())
            .await
            .unwrap();
        doc.set(author, b"result:001".to_vec(), b"done".to_vec())
            .await
            .unwrap();
        doc.set(author, b"claim:001".to_vec(), b"claimed".to_vec())
            .await
            .unwrap();

        let tasks = doc.get_many_by_prefix(b"task:").await.unwrap();
        assert_eq!(
            tasks.len(),
            2,
            "prefix scan should return exactly the two task: entries"
        );
        for entry in &tasks {
            assert!(
                entry.key().starts_with(b"task:"),
                "entry key {:?} should start with task:",
                entry.key()
            );
        }

        let all = doc.get_many_by_prefix(b"").await.unwrap();
        assert_eq!(all.len(), 4, "empty prefix matches all entries");

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn author_list_and_list_docs_report_local_state() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());

        let a1 = docs.author_create().await.unwrap();
        let a2 = docs.author_create().await.unwrap();
        let authors = docs.author_list().await.unwrap();
        assert!(
            authors.contains(&a1) && authors.contains(&a2),
            "author_list must include every author_create result"
        );

        let doc_a = docs.create_doc().await.unwrap();
        let doc_b = docs.create_doc().await.unwrap();
        let listed = docs.list_docs().await.unwrap();
        assert!(
            listed.contains(&doc_a.id()) && listed.contains(&doc_b.id()),
            "list_docs must include every locally-created namespace id"
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn two_nodes_sync_via_share_import() {
        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let docs_a = DocsClient::new(node_a.docs());
        let docs_b = DocsClient::new(node_b.docs());

        let author = docs_a.author_create().await.unwrap();
        let doc_a = docs_a.create_doc().await.unwrap();
        let ticket = doc_a.share_write().await.unwrap();

        let (doc_b, mut events_b) = docs_b.import_and_subscribe(ticket).await.unwrap();

        // Give the nodes a moment to handshake before writing.
        tokio::time::sleep(Duration::from_millis(500)).await;

        doc_a
            .set(author, b"greeting".to_vec(), b"hello".to_vec())
            .await
            .unwrap();

        // Wait for an InsertRemote with our key on node B.
        let observed = timeout(Duration::from_secs(10), async {
            while let Some(ev) = events_b.next().await {
                match ev {
                    Ok(LiveEvent::InsertRemote { entry, .. }) => {
                        if entry.key() == b"greeting" {
                            return Ok::<_, NexusError>(());
                        }
                    }
                    Ok(_) => {}
                    Err(e) => return Err(e),
                }
            }
            Err(NexusError::Docs("event stream ended".into()))
        })
        .await
        .expect("timeout waiting for InsertRemote");

        observed.expect("stream error");

        assert_eq!(doc_a.id(), doc_b.id(), "same namespace on both sides");

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
