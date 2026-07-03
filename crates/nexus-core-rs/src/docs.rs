// SPDX-License-Identifier: AGPL-3.0-or-later
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
use iroh_docs::api::Doc as IrohDoc;
use iroh_docs::api::protocol::{AddrInfoOptions, ShareMode};
use iroh_docs::engine::LiveEvent;
use iroh_docs::protocol::Docs;
use iroh_docs::store::Query;
use iroh_docs::{AuthorId, DocTicket, NamespaceId};

use crate::error::{NexusError, Result};

/// Thin client around an [`iroh_docs::protocol::Docs`] handle.
///
/// Owns a cheaply-cloned `Docs` handle (iroh-docs 0.101 derives
/// `Clone` with an internal `Arc`), so the client can be stored
/// long-lived in a struct or passed across an FFI boundary without
/// a lifetime parameter. This mirrors the Sprint 4 Day 0 change to
/// [`crate::gossip::GossipClient`] and removes a symmetric
/// footgun: a `&'a Docs` field would have blocked the Sprint 4
/// coordinator from keeping a persistent docs client on the
/// Python side.
#[derive(Debug, Clone)]
pub struct DocsClient {
    inner: Docs,
}

impl DocsClient {
    /// Wrap a `&Docs` (typically obtained from
    /// [`crate::Node::docs`]). Clones the inner `Arc` so the
    /// resulting client has no borrow on the source.
    pub fn new(inner: &Docs) -> Self {
        DocsClient {
            inner: inner.clone(),
        }
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
    ) -> Result<(DocHandle, impl Stream<Item = Result<LiveEvent>> + use<>)> {
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
    /// In iroh-docs 0.101 the RPC layer never yields `Ok(None)`: an
    /// absent replica surfaces as an `Err` whose message contains
    /// "Replica not found" (`OpenError::NotFound`, erased to a string
    /// across RPC). Re-verified unchanged from 0.98 at the Sprint 81
    /// Phase B bump (upstream v0.101.0: `store.rs:24-27` keeps the
    /// byte-identical Display, `api.rs:262-265` still hardcodes
    /// `Ok(Some)`). The `Option` is kept to mirror the upstream
    /// signature; callers that must distinguish legitimate absence
    /// from store corruption match on the error message.
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

    /// Return the latest entry per key for entries matching `prefix`.
    ///
    /// Uses `Query::single_latest_per_key()` to deduplicate: when
    /// multiple authors wrote to the same key, only the entry with
    /// the highest timestamp is returned. Useful for "ideas/{uuid}"
    /// style keys where each idea is unique.
    pub async fn get_many_latest_per_key_prefix(
        &self,
        prefix: impl AsRef<[u8]>,
    ) -> Result<Vec<iroh_docs::Entry>> {
        let query = Query::single_latest_per_key().key_prefix(prefix);
        let stream = self
            .inner
            .get_many(query)
            .await
            .map_err(|e| NexusError::Docs(format!("get_many latest_per_key failed: {e}")))?;
        tokio::pin!(stream);
        let mut out = Vec::new();
        while let Some(res) = stream.next().await {
            out.push(res.map_err(|e| {
                NexusError::Docs(format!("get_many latest_per_key stream error: {e}"))
            })?);
        }
        Ok(out)
    }

    /// Return a single entry for a key, taking the latest across
    /// all authors. Returns `None` if no entry exists for the key.
    pub async fn get_latest_by_key(
        &self,
        key: impl AsRef<[u8]>,
    ) -> Result<Option<iroh_docs::Entry>> {
        let query = Query::single_latest_per_key().key_exact(key);
        let stream = self
            .inner
            .get_many(query)
            .await
            .map_err(|e| NexusError::Docs(format!("get_latest_by_key failed: {e}")))?;
        tokio::pin!(stream);
        match stream.next().await {
            Some(res) => Ok(Some(res.map_err(|e| {
                NexusError::Docs(format!("get_latest_by_key stream error: {e}"))
            })?)),
            None => Ok(None),
        }
    }

    // ------------------------------------------------------------------
    // Live sync
    // ------------------------------------------------------------------

    /// Subscribe to all [`LiveEvent`]s on this document.
    ///
    /// The returned stream yields `NexusError::Docs` on any
    /// underlying iroh error. SBFB coordinators consume this
    /// stream to observe new results arriving from workers.
    pub async fn subscribe(
        &self,
    ) -> Result<impl Stream<Item = Result<LiveEvent>> + Send + Unpin + use<>> {
        let stream = self
            .inner
            .subscribe()
            .await
            .map_err(|e| NexusError::Docs(format!("subscribe failed: {e}")))?;
        Ok(stream
            .map(|ev| ev.map_err(|e| NexusError::Docs(format!("live event stream error: {e}")))))
    }

    // ------------------------------------------------------------------
    // Sync
    // ------------------------------------------------------------------

    /// Enter this document's live sync-set, optionally dialing `peers`.
    ///
    /// Opening a doc (`open_doc`/`create_doc`) does NOT enter the
    /// sync-set — verified against iroh-docs 0.101 (recalibrated at
    /// the Sprint 81 Phase B bump; mechanism unchanged from 0.98):
    /// only `start_sync` inserts the namespace into the engine's
    /// `SyncState` (`engine/live.rs:408-414`). A node outside the
    /// sync-set (a) never gossip-broadcasts its incremental
    /// `LocalInsert` writes (gated by `is_syncing`,
    /// `engine/live.rs:713`) and (b) REJECTS every incoming sync
    /// request with `AbortReason::NotFound` (`engine/state.rs:97`).
    /// `share_write`/`share_read` and `import_ticket` enter the
    /// sync-set as a side-effect; a coordinator that only ever
    /// re-OPENS a persisted doc must call this explicitly (Sprint 81
    /// Phase A4 boot fix).
    ///
    /// With an empty `peers` list nothing is dialed by the caller, but
    /// iroh-docs merges the peers PERSISTED in the store
    /// (`register_useful_peer` / `get_sync_peers`) and re-dials them
    /// (`DirectJoin`) — bounded by the store's known-peer list
    /// (`PEERS_PER_DOC_CACHE_SIZE = 5`, `store.rs:17`). Idempotent on
    /// an already-syncing doc.
    pub async fn start_sync(&self, peers: Vec<iroh::EndpointAddr>) -> Result<()> {
        self.inner
            .start_sync(peers)
            .await
            .map_err(|e| NexusError::Docs(format!("start_sync failed: {e}")))
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
pub use iroh_docs::engine::LiveEvent as DocsLiveEvent;
pub use iroh_docs::{
    AuthorId as DocsAuthorId, DocTicket as DocsTicket, Entry as DocsEntry,
    NamespaceId as DocsNamespaceId,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, create_node};
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

    /// S81 Phase C (carry #2, requalified by the preflight): the
    /// `DocTicket` string a coordinator persists (M8 `doc_ticket`
    /// column) and serves to JOIN endpoints must survive the
    /// Display -> FromStr round-trip under the CURRENT lock
    /// (iroh-docs 0.101). The 0.98 -> 0.101 wire compat itself was
    /// settled by byte-diffing the vendored `ticket.rs` (struct,
    /// `TicketWireFormat::Variant0`, `KIND="doc"`, postcard body all
    /// identical; only `Ticket` trait method NAMES changed, invisible
    /// to SBFB which only uses Display + FromStr) — no genuine 0.98
    /// fixture exists to commit, and pre-launch policy makes one a
    /// non-scenario.
    #[tokio::test]
    async fn doc_ticket_string_round_trips_under_current_lock() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());
        let doc = docs.create_doc().await.unwrap();

        let ticket = doc.share_write().await.expect("mint write ticket");
        let s = ticket.to_string(); // == what the DB column persists
        let parsed: DocsTicket = s.parse().expect("persisted ticket string re-parses");
        assert_eq!(
            parsed.capability.id(),
            doc.id(),
            "round-tripped ticket must preserve the NamespaceId"
        );
        assert_eq!(
            parsed.to_string(),
            s,
            "Display -> FromStr -> Display must be idempotent"
        );

        node.shutdown().await.ok();
    }

    /// S81 Phase C: a hostile / malformed ticket string (e.g. read
    /// from a tampered DB column or a bad JOIN request body) must
    /// surface as `Err`, never panic — the consumers parse with
    /// `match .parse()` and turn this into an HTTP error.
    #[test]
    fn doc_ticket_hostile_string_fails_to_parse() {
        assert!("not-a-ticket".parse::<DocsTicket>().is_err());
        assert!("doc".parse::<DocsTicket>().is_err());
        assert!(
            "docaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .parse::<DocsTicket>()
                .is_err(),
            "a base32-shaped but truncated body must fail cleanly"
        );
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

    #[tokio::test]
    async fn get_latest_by_key_returns_most_recent_across_authors() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());
        let blobs = crate::BlobsClient::new(node.blobs_store());

        let author_a = docs.author_create().await.unwrap();
        let author_b = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        doc.set(
            author_a,
            b"ideas/001".to_vec(),
            b"{\"title\":\"old\"}".to_vec(),
        )
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        doc.set(
            author_b,
            b"ideas/001".to_vec(),
            b"{\"title\":\"new\"}".to_vec(),
        )
        .await
        .unwrap();

        let entry = doc
            .get_latest_by_key(b"ideas/001")
            .await
            .unwrap()
            .expect("entry must exist");

        let content = blobs
            .get_bytes(*entry.content_hash().as_bytes())
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&content).unwrap();
        assert_eq!(json["title"], "new", "latest entry wins across authors");

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn get_many_latest_per_key_prefix_deduplicates() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());

        let author_a = docs.author_create().await.unwrap();
        let author_b = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        doc.set(author_a, b"ideas/001".to_vec(), b"v1".to_vec())
            .await
            .unwrap();
        doc.set(author_b, b"ideas/001".to_vec(), b"v2".to_vec())
            .await
            .unwrap();
        doc.set(author_a, b"ideas/002".to_vec(), b"v3".to_vec())
            .await
            .unwrap();

        let entries = doc.get_many_latest_per_key_prefix(b"ideas/").await.unwrap();
        assert_eq!(
            entries.len(),
            2,
            "single_latest_per_key should return 1 entry per key"
        );

        let all_entries = doc.get_many_by_prefix(b"ideas/").await.unwrap();
        assert_eq!(
            all_entries.len(),
            3,
            "get_many_by_prefix returns all entries including multi-author"
        );

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn storage_crud_via_iroh_docs() {
        let node = spawn_node().await;
        let docs = DocsClient::new(node.docs());
        let blobs = crate::BlobsClient::new(node.blobs_store());

        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();

        doc.set(
            author,
            b"ideas/abc".to_vec(),
            b"{\"title\":\"test idea\"}".to_vec(),
        )
        .await
        .unwrap();

        let entry = doc
            .get_latest_by_key(b"ideas/abc")
            .await
            .unwrap()
            .expect("entry must exist after set");
        let content = blobs
            .get_bytes(*entry.content_hash().as_bytes())
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&content).unwrap();
        assert_eq!(json["title"], "test idea");

        doc.set(
            author,
            b"ideas/abc".to_vec(),
            b"{\"deleted\":true}".to_vec(),
        )
        .await
        .unwrap();

        let tombstone_entry = doc
            .get_latest_by_key(b"ideas/abc")
            .await
            .unwrap()
            .expect("tombstone entry must exist");
        let tombstone_content = blobs
            .get_bytes(*tombstone_entry.content_hash().as_bytes())
            .await
            .unwrap();
        let tombstone_json: serde_json::Value = serde_json::from_slice(&tombstone_content).unwrap();
        assert_eq!(tombstone_json["deleted"], true, "tombstone marks deletion");

        node.shutdown().await.ok();
    }
}
