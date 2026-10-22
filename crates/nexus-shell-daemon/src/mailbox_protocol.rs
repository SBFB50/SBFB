// SPDX-License-Identifier: AGPL-3.0-or-later
//! Chat v1 offline mailbox, daemon side (ADR_CHAT_V1 decision 5, plan
//! Phase E). ALPN `sbfb/mailbox/0`.
//!
//! **Server** ([`MailboxServer`], only on a node with `[mailbox] serve =
//! true`): a depositor hands over a READ ticket of a conversation document;
//! the server imports a read-only replica and keeps it in its sync-set, so
//! members that are never online together converge through it. The replica
//! can serve every entry and author none.
//!
//! Security (THREAT_MODEL §20):
//! - **Attribution** — every registration is keyed by the QUIC-authenticated
//!   dialer (`conn.remote_id()`), never by a request field. The request
//!   carries no signature: the handshake already proves the dialer holds
//!   its node key, and nothing is attributed to anyone else.
//! - **Write tickets refused** — a ticket granting write is rejected before
//!   anything is stored: the namespace secret never reaches this node.
//! - **No third-party dialing** — the imported ticket is rebuilt with the
//!   dialer as its only peer; addresses of other nodes a ticket may list are
//!   dropped, so a deposit cannot point the server at arbitrary endpoints.
//! - **Bounded** — [`MailboxRegistry`] enforces the per-node and global
//!   document caps and the TTL; the sweep evicts any document above the
//!   size cap.
//!
//! Known limit: dropping a replica removes the namespace and its entries
//! but not the content blobs — iroh-blobs garbage collection is not
//! enabled on SBFB nodes (enabling it requires protecting every untagged
//! blob the rest of the daemon relies on). Dead blobs of dropped mailbox
//! documents stay on disk until that lands.
//!
//! **Client** ([`mailbox_call`]): one request, one response per connection;
//! the chat service drives it ([`crate::chat_api`]).

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use iroh::{Endpoint, EndpointAddr, EndpointId};
use nexus_core_rs::docs::{DocsCapability, DocsClient, DocsNamespaceId, DocsTicket};
use nexus_messaging_core::mailbox::{
    Admission, MAILBOX_FORMAT_VERSION, MAX_MAILBOX_MSG_BYTES, MailboxLimits, MailboxOp,
    MailboxRegistry, MailboxRequest, MailboxResponse, MailboxVerdict, Release,
};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Registry file under the daemon root.
pub const MAILBOX_REGISTRY_FILE: &str = "mailbox.json";

/// How often expired and oversize documents are swept.
const SWEEP_EVERY: Duration = Duration::from_secs(10 * 60);

/// Bytes charged per entry on top of its content when sizing a document
/// (key, namespace, author, signature, hash, length, timestamp).
const ENTRY_OVERHEAD_BYTES: u64 = 256;

/// Bound on one client exchange, dial included.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Server side: registry + read-only replicas.
pub struct MailboxServer {
    docs: OnceLock<DocsClient>,
    path: PathBuf,
    limits: MailboxLimits,
    registry: Mutex<MailboxRegistry>,
}

impl std::fmt::Debug for MailboxServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MailboxServer")
            .field("path", &self.path)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

impl MailboxServer {
    /// Load the registry at `path` (absent = empty). An unreadable file is
    /// logged and replaced by an empty registry: refusing to boot an anchor
    /// over its mailbox bookkeeping would take every other role down too.
    pub fn open(path: PathBuf, limits: MailboxLimits) -> Self {
        let registry = match std::fs::read(&path) {
            Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or_else(|e| {
                warn!(path = %path.display(), error = %e, "unreadable mailbox registry, starting empty");
                MailboxRegistry::default()
            }),
            Err(_) => MailboxRegistry::default(),
        };
        Self {
            docs: OnceLock::new(),
            path,
            limits,
            registry: Mutex::new(registry),
        }
    }

    /// Hand the server the node's docs once the node exists (the ALPN
    /// handler is built before it), re-enter every held document into the
    /// sync-set — a re-opened replica is outside it and would refuse every
    /// sync — and start the sweep loop.
    pub fn attach(self: &Arc<Self>, docs: DocsClient) {
        if self.docs.set(docs.clone()).is_err() {
            return;
        }
        let server = Arc::clone(self);
        tokio::spawn(async move {
            server.reopen_all(&docs).await;
            loop {
                server.sweep(&docs, now_secs()).await;
                tokio::time::sleep(SWEEP_EVERY).await;
            }
        });
    }

    async fn reopen_all(&self, docs: &DocsClient) {
        let mut reg = self.registry.lock().await;
        let mut missing = Vec::new();
        for ns in reg.namespaces() {
            match open(docs, &ns).await {
                Some(doc) => {
                    if let Err(e) = doc.start_sync(Vec::new()).await {
                        debug!(error = %e, "mailbox start_sync on reopen");
                    }
                }
                None => missing.push(ns),
            }
        }
        for ns in &missing {
            reg.evict(ns);
        }
        if !missing.is_empty() {
            self.persist(&reg);
        }
        info!(held = reg.namespaces().len(), "mailbox server ready");
    }

    /// Drop expired documents, then evict any document above the size cap.
    pub async fn sweep(&self, docs: &DocsClient, now: u64) {
        let mut reg = self.registry.lock().await;
        let mut changed = false;
        for ns in reg.expire(now) {
            drop_replica(docs, &ns).await;
            changed = true;
        }
        for ns in reg.namespaces() {
            let Some(doc) = open(docs, &ns).await else {
                continue;
            };
            let entries = doc.get_many_by_prefix(b"").await;
            close(&doc).await;
            let Ok(entries) = entries else {
                continue;
            };
            let size: u64 = entries
                .iter()
                .map(|e| e.content_len().saturating_add(ENTRY_OVERHEAD_BYTES))
                .sum();
            if size > self.limits.max_doc_bytes {
                warn!(namespace = %ns, size, "mailbox document above the size cap, evicted");
                reg.evict(&ns);
                drop_replica(docs, &ns).await;
                changed = true;
            }
        }
        if changed {
            self.persist(&reg);
        }
    }

    /// Decide one request from `dialer`. Pure of stream I/O.
    pub async fn handle(&self, bytes: &[u8], dialer: EndpointId, now: u64) -> MailboxResponse {
        let Some(docs) = self.docs.get() else {
            return MailboxResponse::new(MailboxVerdict::Unavailable, 0);
        };
        let req: MailboxRequest = match serde_json::from_slice(bytes) {
            Ok(r) => r,
            Err(e) => {
                debug!(error = %e, "mailbox: undecodable request");
                return MailboxResponse::new(MailboxVerdict::BadRequest, 0);
            }
        };
        if req.version != MAILBOX_FORMAT_VERSION {
            return MailboxResponse::new(MailboxVerdict::BadRequest, 0);
        }
        match req.op {
            MailboxOp::Deposit { ticket } => self.deposit(docs, &ticket, dialer, now).await,
            MailboxOp::Withdraw { namespace } => self.withdraw(docs, &namespace, dialer).await,
        }
    }

    async fn deposit(
        &self,
        docs: &DocsClient,
        ticket: &str,
        dialer: EndpointId,
        now: u64,
    ) -> MailboxResponse {
        let Ok(ticket) = DocsTicket::from_str(ticket.trim()) else {
            return MailboxResponse::new(MailboxVerdict::BadRequest, 0);
        };
        let ns = match ticket.capability {
            DocsCapability::Read(ns) => ns,
            DocsCapability::Write(_) => {
                return MailboxResponse::new(MailboxVerdict::WriteTicket, 0);
            }
        };
        let peers = peers_for(ticket.nodes, dialer);
        let ns_key = ns.to_string();
        let node_key = dialer.to_string();
        let mut reg = self.registry.lock().await;
        let Some((admission, expires_at)) = reg.deposit(&ns_key, &node_key, now, &self.limits)
        else {
            return MailboxResponse::new(MailboxVerdict::Quota, 0);
        };
        // A refresh only checks that the replica still exists: syncing is
        // the depositor's job (it dials this always-on node after every
        // deposit and periodically, `chat_api::sync_with_servers`).
        let held = match admission {
            Admission::Held => open(docs, &ns_key).await,
            Admission::New => None,
        };
        if let Some(doc) = held {
            close(&doc).await;
        } else if let Err(e) = import(docs, ns, peers).await {
            // No replica exists for anyone: the registration cannot stand.
            warn!(error = %e, "mailbox: replica import failed");
            reg.evict(&ns_key);
            self.persist(&reg);
            return MailboxResponse::new(MailboxVerdict::ImportFailed, 0);
        }
        self.persist(&reg);
        debug!(namespace = %ns_key, depositor = %node_key, "mailbox: kept");
        MailboxResponse::new(MailboxVerdict::Kept, expires_at)
    }

    async fn withdraw(
        &self,
        docs: &DocsClient,
        namespace: &str,
        dialer: EndpointId,
    ) -> MailboxResponse {
        let Ok(ns) = DocsNamespaceId::from_str(namespace.trim()) else {
            return MailboxResponse::new(MailboxVerdict::BadRequest, 0);
        };
        let ns_key = ns.to_string();
        let mut reg = self.registry.lock().await;
        match reg.withdraw(&ns_key, &dialer.to_string()) {
            Release::NotHeld => {}
            Release::StillHeld => self.persist(&reg),
            Release::Last => {
                drop_replica(docs, &ns_key).await;
                self.persist(&reg);
            }
        }
        MailboxResponse::new(MailboxVerdict::Released, 0)
    }

    fn persist(&self, reg: &MailboxRegistry) {
        let write = || -> std::io::Result<()> {
            let tmp = self.path.with_extension("tmp");
            std::fs::write(&tmp, serde_json::to_vec(reg)?)?;
            std::fs::rename(&tmp, &self.path)
        };
        if let Err(e) = write() {
            warn!(path = %self.path.display(), error = %e, "persisting mailbox registry failed");
        }
    }

    #[cfg(test)]
    pub(crate) async fn held(&self) -> Vec<String> {
        self.registry.lock().await.namespaces()
    }
}

/// The peers a deposited replica is told about: the dialer only. A ticket
/// may list any address; keeping the dialer's own entries (its current
/// direct addresses) and nothing else means a deposit can never make this
/// node dial a third party.
fn peers_for(ticket_nodes: Vec<EndpointAddr>, dialer: EndpointId) -> Vec<EndpointAddr> {
    let mut peers: Vec<EndpointAddr> = ticket_nodes
        .into_iter()
        .filter(|a| a.id == dialer)
        .collect();
    if peers.is_empty() {
        peers.push(EndpointAddr::new(dialer));
    }
    peers
}

/// Open a held replica. iroh-docs counts open handles per replica and
/// `drop_doc` releases exactly one (plus the sync engine's): every held
/// replica keeps ONE long-lived handle — from its import, or from the boot
/// re-open — and every other `open` must be paired with [`close`], or the
/// replica can never be dropped.
async fn open(docs: &DocsClient, ns: &str) -> Option<nexus_core_rs::DocHandle> {
    let id = DocsNamespaceId::from_str(ns).ok()?;
    docs.open_doc(id).await.ok().flatten()
}

async fn close(doc: &nexus_core_rs::DocHandle) {
    if let Err(e) = doc.inner().close().await {
        debug!(error = %e, "mailbox: closing a transient handle");
    }
}

async fn import(
    docs: &DocsClient,
    ns: DocsNamespaceId,
    peers: Vec<EndpointAddr>,
) -> nexus_core_rs::Result<()> {
    docs.import_ticket(DocsTicket::new(DocsCapability::Read(ns), peers))
        .await
        .map(|_| ())
}

async fn drop_replica(docs: &DocsClient, ns: &str) {
    let Ok(id) = DocsNamespaceId::from_str(ns) else {
        return;
    };
    if let Err(e) = docs.drop_doc(id).await {
        debug!(namespace = %ns, error = %e, "mailbox: drop_doc");
    }
}

/// The `sbfb/mailbox/0` ALPN handler.
#[derive(Debug, Clone)]
pub struct MailboxProtocol {
    server: Arc<MailboxServer>,
}

impl ProtocolHandler for MailboxProtocol {
    async fn accept(&self, conn: Connection) -> Result<(), AcceptError> {
        let dialer = conn.remote_id();
        let (mut send, mut recv) = conn.accept_bi().await?;
        let bytes = recv
            .read_to_end(MAX_MAILBOX_MSG_BYTES)
            .await
            .map_err(AcceptError::from_err)?;
        let resp = self.server.handle(&bytes, dialer, now_secs()).await;
        let out = serde_json::to_vec(&resp).map_err(AcceptError::from_err)?;
        send.write_all(&out).await.map_err(AcceptError::from_err)?;
        send.finish().map_err(AcceptError::from_err)?;
        conn.closed().await;
        Ok(())
    }
}

/// Factory for `create_node_with_protocols`; the server is attached to the
/// node's docs once the node exists ([`MailboxServer::attach`]).
pub fn mailbox_protocol_factory(
    server: Arc<MailboxServer>,
) -> nexus_core_rs::node::ExtraProtocolFactory {
    Box::new(move |_store, _ep, _ml| {
        Box::new(MailboxProtocol { server }) as Box<dyn iroh::protocol::DynProtocolHandler>
    })
}

/// Client side: one request to `server`, its response, within
/// [`CALL_TIMEOUT`]. The server is dialed by id (address lookup resolves
/// it); the response is trusted as the dialed node's because the QUIC
/// handshake authenticated it.
pub async fn mailbox_call(
    endpoint: &Endpoint,
    server: EndpointId,
    req: &MailboxRequest,
) -> Result<MailboxResponse, String> {
    let exchange = async {
        let conn = endpoint
            .connect(EndpointAddr::new(server), nexus_core_rs::MAILBOX_ALPN)
            .await
            .map_err(|e| format!("mailbox dial failed: {e}"))?;
        let (mut send, mut recv) = conn
            .open_bi()
            .await
            .map_err(|e| format!("mailbox open_bi failed: {e}"))?;
        let body = serde_json::to_vec(req).map_err(|e| format!("mailbox encode: {e}"))?;
        send.write_all(&body)
            .await
            .map_err(|e| format!("mailbox write failed: {e}"))?;
        send.finish()
            .map_err(|e| format!("mailbox finish failed: {e}"))?;
        let bytes = recv
            .read_to_end(MAX_MAILBOX_MSG_BYTES)
            .await
            .map_err(|e| format!("mailbox read failed: {e}"))?;
        conn.close(0u32.into(), b"done");
        let resp: MailboxResponse =
            serde_json::from_slice(&bytes).map_err(|e| format!("mailbox decode: {e}"))?;
        if resp.version != MAILBOX_FORMAT_VERSION {
            return Err("mailbox response version mismatch".into());
        }
        Ok(resp)
    };
    tokio::time::timeout(CALL_TIMEOUT, exchange)
        .await
        .map_err(|_| "mailbox call timed out".to_string())?
}

/// Parse `[mailbox] servers` (already shape-checked at config load).
pub fn parse_servers(hexes: &[String]) -> Vec<EndpointId> {
    hexes
        .iter()
        .filter_map(|h| {
            let bytes: [u8; 32] = hex::decode(h).ok()?.try_into().ok()?;
            EndpointId::from_bytes(&bytes).ok()
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use nexus_core_rs::discovery::DiscoveryClient;
    use nexus_core_rs::{
        BlobsClient, DocHandle, KeyPair, Node, NodeConfig, create_node, create_node_with_protocols,
    };

    pub(crate) const LIMITS: MailboxLimits = MailboxLimits {
        ttl_secs: 3600,
        max_docs_total: 8,
        max_docs_per_node: 4,
        max_doc_bytes: 1 << 20,
    };

    pub(crate) struct Anchor {
        pub node: Node,
        pub server: Arc<MailboxServer>,
        pub addr: EndpointAddr,
        pub path: PathBuf,
        _dir: tempfile::TempDir,
    }

    impl Anchor {
        pub fn id(&self) -> EndpointId {
            self.addr.id
        }
        pub fn docs(&self) -> DocsClient {
            DocsClient::new(self.node.docs())
        }
    }

    pub(crate) async fn anchor(limits: MailboxLimits) -> Anchor {
        anchor_with(tempfile::tempdir().unwrap(), NodeConfig::default(), limits).await
    }

    async fn anchor_with(dir: tempfile::TempDir, cfg: NodeConfig, limits: MailboxLimits) -> Anchor {
        let path = dir.path().join(MAILBOX_REGISTRY_FILE);
        let server = Arc::new(MailboxServer::open(path.clone(), limits));
        let node = create_node_with_protocols(
            cfg,
            vec![(
                nexus_core_rs::MAILBOX_ALPN.to_vec(),
                mailbox_protocol_factory(Arc::clone(&server)),
            )],
        )
        .await
        .unwrap();
        server.attach(DocsClient::new(node.docs()));
        let addr = DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        Anchor {
            node,
            server,
            addr,
            path,
            _dir: dir,
        }
    }

    async fn client(anchor: &Anchor) -> Node {
        let n = create_node().await.unwrap();
        n.memory_lookup().add_endpoint_info(anchor.addr.clone());
        n
    }

    async fn doc_with(node: &Node, entries: &[(&str, &[u8])]) -> DocHandle {
        let docs = DocsClient::new(node.docs());
        let author = docs.author_create().await.unwrap();
        let doc = docs.create_doc().await.unwrap();
        for (k, v) in entries {
            doc.set(author, k.as_bytes().to_vec(), v.to_vec())
                .await
                .unwrap();
        }
        doc
    }

    /// Wait until `node` holds `n` entries under `m/` with their content.
    pub(crate) async fn wait_entries(node: &Node, ns: DocsNamespaceId, n: usize) -> Vec<Vec<u8>> {
        let docs = DocsClient::new(node.docs());
        let blobs = BlobsClient::new(node.blobs_store());
        let deadline = tokio::time::Instant::now() + Duration::from_secs(45);
        loop {
            let opened = docs.open_doc(ns).await.ok().flatten();
            let listed = match &opened {
                Some(doc) => {
                    let l = doc.get_many_by_prefix(b"m/").await.ok();
                    // Never leave a test handle open: it would pin the
                    // replica against the very drops these tests assert.
                    close(doc).await;
                    l
                }
                None => None,
            };
            if let Some(entries) = listed {
                let mut out = Vec::new();
                for e in &entries {
                    if let Ok(b) = blobs.get_bytes(*e.content_hash().as_bytes()).await {
                        out.push(b);
                    }
                }
                if out.len() >= n {
                    return out;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "timed out waiting for {n} entries"
            );
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
    }

    async fn call(node: &Node, anchor: &Anchor, req: MailboxRequest) -> MailboxResponse {
        mailbox_call(node.endpoint(), anchor.id(), &req)
            .await
            .expect("mailbox call")
    }

    fn some_id() -> EndpointId {
        EndpointId::from_bytes(&KeyPair::generate().public_bytes()).unwrap()
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn deposited_history_outlives_the_depositor_and_the_replica_cannot_write() {
        let anchor = anchor(LIMITS).await;
        let alice = client(&anchor).await;
        let doc = doc_with(&alice, &[("m/a/1", b"cipher-1"), ("m/a/2", b"cipher-2")]).await;
        let ns = doc.id();
        let ticket = doc.share_read().await.unwrap().to_string();

        let resp = call(&alice, &anchor, MailboxRequest::deposit(ticket)).await;
        assert_eq!(resp.verdict, MailboxVerdict::Kept);
        assert!(resp.expires_at > now_secs() + 3000);
        assert_eq!(anchor.server.held().await, vec![ns.to_string()]);
        wait_entries(&anchor.node, ns, 2).await;

        // The replica is read-only: the server cannot author an entry.
        let docs = anchor.docs();
        let replica = docs.open_doc(ns).await.unwrap().unwrap();
        let author = docs.author_create().await.unwrap();
        assert!(
            replica
                .set(author, b"m/x/1".to_vec(), b"forged".to_vec())
                .await
                .is_err()
        );
        close(&replica).await;

        // The depositor leaves for good; a member that never met it gets the
        // whole history from the mailbox alone.
        alice.shutdown().await.ok();
        let carol = client(&anchor).await;
        DocsClient::new(carol.docs())
            .import_ticket(DocsTicket::new(
                DocsCapability::Read(ns),
                vec![anchor.addr.clone()],
            ))
            .await
            .unwrap();
        let mut got = wait_entries(&carol, ns, 2).await;
        got.sort();
        assert_eq!(got, vec![b"cipher-1".to_vec(), b"cipher-2".to_vec()]);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn write_tickets_and_malformed_requests_store_nothing() {
        let anchor = anchor(LIMITS).await;
        let alice = client(&anchor).await;
        let doc = doc_with(&alice, &[("m/a/1", b"c")]).await;
        let write = doc.share_write().await.unwrap().to_string();
        let resp = call(&alice, &anchor, MailboxRequest::deposit(write)).await;
        assert_eq!(resp.verdict, MailboxVerdict::WriteTicket);
        let resp = call(
            &alice,
            &anchor,
            MailboxRequest::deposit("docgarbage".into()),
        )
        .await;
        assert_eq!(resp.verdict, MailboxVerdict::BadRequest);
        let resp = call(&alice, &anchor, MailboxRequest::withdraw("nope".into())).await;
        assert_eq!(resp.verdict, MailboxVerdict::BadRequest);
        // A future version is refused even when its payload would be valid.
        let read = doc.share_read().await.unwrap().to_string();
        let future = serde_json::json!({"version": 2, "op": {"deposit": {"ticket": read}}});
        assert_eq!(
            anchor
                .server
                .handle(future.to_string().as_bytes(), some_id(), now_secs())
                .await
                .verdict,
            MailboxVerdict::BadRequest
        );
        assert!(anchor.server.held().await.is_empty());
        assert!(
            !anchor.docs().list_docs().await.unwrap().contains(&doc.id()),
            "no replica was created"
        );
    }

    #[tokio::test]
    async fn unattached_server_is_unavailable() {
        let dir = tempfile::tempdir().unwrap();
        let server = MailboxServer::open(dir.path().join("m.json"), LIMITS);
        let req = serde_json::to_vec(&MailboxRequest::withdraw("x".into())).unwrap();
        assert_eq!(
            server.handle(&req, some_id(), 0).await.verdict,
            MailboxVerdict::Unavailable
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn per_node_quota_then_withdraw_frees_and_drops_the_replica() {
        let anchor = anchor(MailboxLimits {
            max_docs_per_node: 1,
            ..LIMITS
        })
        .await;
        let alice = client(&anchor).await;
        let d1 = doc_with(&alice, &[("m/a/1", b"1")]).await;
        let d2 = doc_with(&alice, &[("m/a/1", b"2")]).await;
        let t1 = d1.share_read().await.unwrap().to_string();
        let t2 = d2.share_read().await.unwrap().to_string();
        assert_eq!(
            call(&alice, &anchor, MailboxRequest::deposit(t1.clone()))
                .await
                .verdict,
            MailboxVerdict::Kept
        );
        // A refresh is never refused at the cap — and must not leave a
        // handle behind that would block the drop below.
        assert_eq!(
            call(&alice, &anchor, MailboxRequest::deposit(t1))
                .await
                .verdict,
            MailboxVerdict::Kept
        );
        assert_eq!(
            call(&alice, &anchor, MailboxRequest::deposit(t2.clone()))
                .await
                .verdict,
            MailboxVerdict::Quota
        );
        assert_eq!(
            call(
                &alice,
                &anchor,
                MailboxRequest::withdraw(d1.id().to_string())
            )
            .await
            .verdict,
            MailboxVerdict::Released
        );
        assert!(anchor.server.held().await.is_empty());
        assert!(!anchor.docs().list_docs().await.unwrap().contains(&d1.id()));
        assert_eq!(
            call(&alice, &anchor, MailboxRequest::deposit(t2))
                .await
                .verdict,
            MailboxVerdict::Kept
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn sweep_drops_expired_and_oversize_documents_and_registry_persists() {
        let anchor = anchor(MailboxLimits {
            ttl_secs: 100,
            // One 100-byte entry plus its overhead (356) is above the cap.
            max_doc_bytes: 300,
            ..LIMITS
        })
        .await;
        let alice = client(&anchor).await;
        let small = doc_with(&alice, &[]).await;
        let big = doc_with(&alice, &[("m/a/1", &[7u8; 100])]).await;
        for d in [&small, &big] {
            let t = d.share_read().await.unwrap().to_string();
            assert_eq!(
                call(&alice, &anchor, MailboxRequest::deposit(t))
                    .await
                    .verdict,
                MailboxVerdict::Kept
            );
        }
        // The registry is on disk after each deposit.
        let reloaded = MailboxServer::open(anchor.path.clone(), LIMITS);
        assert_eq!(reloaded.held().await.len(), 2);

        wait_entries(&anchor.node, big.id(), 1).await;
        anchor.server.sweep(&anchor.docs(), now_secs()).await;
        assert_eq!(anchor.server.held().await, vec![small.id().to_string()]);
        assert!(!anchor.docs().list_docs().await.unwrap().contains(&big.id()));

        anchor.server.sweep(&anchor.docs(), now_secs() + 101).await;
        assert!(anchor.server.held().await.is_empty());
        assert!(
            !anchor
                .docs()
                .list_docs()
                .await
                .unwrap()
                .contains(&small.id())
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn a_restarted_server_still_serves_what_it_held() {
        let secret = KeyPair::generate().secret_bytes();
        let dir = tempfile::tempdir().unwrap();
        let cfg = || {
            NodeConfig::default()
                .with_secret_key(secret)
                .with_data_dir(dir.path().join("iroh"))
        };
        let first = anchor_with(tempfile::tempdir().unwrap(), cfg(), LIMITS).await;
        let alice = client(&first).await;
        let doc = doc_with(&alice, &[("m/a/1", b"kept-1")]).await;
        let ns = doc.id();
        let t = doc.share_read().await.unwrap().to_string();
        assert_eq!(
            call(&alice, &first, MailboxRequest::deposit(t))
                .await
                .verdict,
            MailboxVerdict::Kept
        );
        wait_entries(&first.node, ns, 1).await;
        alice.shutdown().await.ok();
        // Restart: same identity, same stores, same registry file.
        let Anchor { node, _dir, .. } = first;
        node.shutdown().await.ok();
        let second = anchor_with(_dir, cfg(), LIMITS).await;
        assert_eq!(second.server.held().await, vec![ns.to_string()]);
        // A re-opened replica is outside the sync-set unless the server puts
        // it back: a newcomer must still get the history from it alone.
        let carol = client(&second).await;
        DocsClient::new(carol.docs())
            .import_ticket(DocsTicket::new(
                DocsCapability::Read(ns),
                vec![second.addr.clone()],
            ))
            .await
            .unwrap();
        assert_eq!(wait_entries(&carol, ns, 1).await, vec![b"kept-1".to_vec()]);
    }

    #[test]
    fn ticket_peers_are_reduced_to_the_dialer() {
        let dialer = some_id();
        let third = some_id();
        assert_eq!(
            peers_for(vec![EndpointAddr::new(third)], dialer),
            vec![EndpointAddr::new(dialer)],
            "a third party listed in the ticket is never dialed"
        );
        let own = EndpointAddr::new(dialer).with_ip_addr("127.0.0.1:4242".parse().unwrap());
        assert_eq!(
            peers_for(vec![EndpointAddr::new(third), own.clone()], dialer),
            vec![own],
            "the dialer's own addresses are kept"
        );
    }
}
