// SPDX-License-Identifier: AGPL-3.0-or-later
//! iroh node lifecycle with full SBFB protocol stack.
//!
//! A `Node` is a handle to a running iroh [`Endpoint`] that also
//! carries the three meta-protocols SBFB depends on: iroh-docs
//! (replicated key/value logs for tasks and results), iroh-gossip
//! (topic-based broadcast for curator lists and coordination), and
//! iroh-blobs (content-addressed storage for curator list blobs).
//!
//! All three protocols are wired through an iroh [`Router`] which
//! multiplexes incoming connections by ALPN and dispatches them to
//! the right handler. `Node::shutdown` delegates to
//! [`Router::shutdown`] which runs the graceful teardown sequence
//! (cancel-token → protocol handlers drain → `Endpoint::close`).
//!
//! ## Creation
//!
//! - [`create_node`] — fresh random keypair, default config
//! - [`create_node_with_config`] — supply a [`NodeConfig`] with
//!   an optional persistent secret key and custom ALPNs
//!
//! Persistence (Sprint 66 Phase A):
//! - [`BlobStore`] selects [`FsStore`] (redb) when `data_dir` is set,
//!   [`MemStore`] otherwise. Both deref to [`Store`].
//!
//! ## Example
//!
//! ```no_run
//! # async fn example() -> nexus_core_rs::Result<()> {
//! let node = nexus_core_rs::create_node().await?;
//! tracing::info!("node id: {}", node.node_id());
//! // access the protocol stack
//! let _docs = node.docs();
//! let _gossip = node.gossip();
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

use std::fmt;
use std::path::PathBuf;

use iroh::address_lookup::memory::MemoryLookup;
use iroh::address_lookup::{PkarrPublisher, PkarrResolver};
use iroh::endpoint::presets;
use iroh::protocol::{DynProtocolHandler, Router};
use iroh::{Endpoint, RelayMode, SecretKey};
use iroh_blobs::api::Store;
use iroh_blobs::store::fs::FsStore;
use iroh_blobs::store::mem::MemStore;
use iroh_blobs::{ALPN as BLOBS_ALPN, BlobsProtocol};
use iroh_docs::ALPN as DOCS_ALPN;
use iroh_docs::protocol::Docs;
use iroh_gossip::ALPN as GOSSIP_ALPN;
use iroh_gossip::net::Gossip;
use tracing::{debug, info, warn};

use crate::crypto::SECRET_KEY_BYTES;
use crate::discovery_override::DiscoveryPlan;
use crate::error::{NexusError, Result};

/// ALPN for the Sprint 74 cross-node seed protocol. A 4th protocol
/// alongside blobs / gossip / docs, registered via
/// [`create_node_with_protocols`] (the handler lives in the daemon, not
/// here, because it carries the coordinator DB + node keypair).
///
/// The trailing `/0` is the protocol generation; the payloads carry a
/// fine-grained `version` field under it (see
/// [`crate::seed::SEED_FORMAT_VERSION`]).
pub const SEED_ALPN: &[u8] = b"sbfb/seed/0";

/// ALPN for the Sprint 77 sharded-inference data plane. Registered via
/// the same [`create_node_with_protocols`] `extra_protocols` mechanism as
/// [`SEED_ALPN`]; the handler ([`crate::shard::ShardProtocol`]) carries a
/// private [`crate::compute_group::ComputeGroupEntry`] allowlist and
/// rejects a non-member at the handshake before any activation frame.
///
/// The trailing `/1` is the protocol generation; the activation frames
/// carry a fine-grained `version` under it (see
/// [`crate::compute_group::COMPUTE_GROUP_FORMAT_VERSION`] for the admission
/// payload).
pub const SHARD_ALPN: &[u8] = b"sbfb/shard/1";

/// A factory that builds an extra ALPN protocol handler once the node's
/// blob store, endpoint and address lookup exist.
///
/// The iroh [`Router`] accepts NO post-spawn protocol registration —
/// every ALPN must be handed to the builder before `.spawn()`. But a
/// handler like the Sprint 74 seed protocol needs BOTH core node state
/// (the blob store / endpoint, created inside
/// [`create_node_with_config`]) AND caller state (the coordinator DB +
/// keypair, created later in the daemon). This factory resolves the
/// chicken-and-egg: the daemon builds the closure capturing its own
/// state, and [`create_node_with_protocols`] invokes it with the freshly
/// created store/endpoint/lookup right before wiring the Router.
pub type ExtraProtocolFactory =
    Box<dyn FnOnce(&Store, &Endpoint, &MemoryLookup) -> Box<dyn DynProtocolHandler> + Send>;

/// Configuration for booting a [`Node`].
///
/// Use `NodeConfig::default()` for the simplest case (fresh
/// random identity, in-memory stores, default ALPNs). Call
/// [`NodeConfig::with_secret_key`] to supply a persistent
/// identity loaded from disk, and [`NodeConfig::with_data_dir`]
/// to persist the docs replica + default author across reboots.
///
/// Persistence scope:
///
/// - `data_dir` applies to **iroh-docs** (`docs.redb` +
///   `default-author`) and **iroh-blobs** (`blobs/` subdirectory
///   via [`FsStore`]). Both stores survive daemon restarts.
#[derive(Debug, Clone, Default)]
pub struct NodeConfig {
    /// If Some, the node boots with this Ed25519 secret key
    /// (32 bytes). If None, a fresh random key is generated.
    pub secret_key_bytes: Option<[u8; SECRET_KEY_BYTES]>,

    /// If Some, the docs replica and default author are stored on
    /// disk under this directory. The directory is created on
    /// demand; iroh-docs writes `docs.redb` and `default-author`
    /// inside. Persistent mode is necessary for the coordinator
    /// reboot flow (same author id + same namespace id across
    /// process restarts).
    pub data_dir: Option<PathBuf>,
}

impl NodeConfig {
    /// Use this specific secret key instead of generating a random
    /// one. Typical flow: call [`crate::KeyPair::load_or_generate`]
    /// to get a stable key tied to a file on disk, then pass its
    /// secret bytes here.
    pub fn with_secret_key(mut self, secret: [u8; SECRET_KEY_BYTES]) -> Self {
        self.secret_key_bytes = Some(secret);
        self
    }

    /// Persist the iroh-docs replica and default author storage
    /// under `path`. Without this, the node runs fully in-memory
    /// and every reboot produces fresh author / namespace ids.
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = Some(path.into());
        self
    }
}

/// Backend-agnostic blob store: either in-memory ([`MemStore`]) or
/// filesystem-backed ([`FsStore`]). Both variants deref to [`Store`]
/// so callers that only need the API surface get `&Store` from
/// [`Node::blobs_store`] regardless of the backing implementation.
pub enum BlobStore {
    /// In-memory store — data lost on process exit.
    Mem(MemStore),
    /// Filesystem-backed store (redb) — data persists across restarts.
    Fs(FsStore),
}

impl std::ops::Deref for BlobStore {
    type Target = Store;
    fn deref(&self) -> &Store {
        match self {
            BlobStore::Mem(s) => s,
            BlobStore::Fs(s) => s,
        }
    }
}

/// A running iroh node with the full SBFB protocol stack.
///
/// Holds the endpoint, the Docs/Gossip/Blobs protocol handlers
/// and the Router that dispatches incoming connections to them.
/// Dropping the Node without calling [`Node::shutdown`] closes
/// the endpoint non-gracefully; callers that care should always
/// `await node.shutdown()` first.
pub struct Node {
    endpoint: Endpoint,
    docs: Docs,
    gossip: Gossip,
    blobs_store: BlobStore,
    router: Router,
    memory_lookup: MemoryLookup,
}

impl Node {
    /// Return the short textual form of this node's Ed25519
    /// public key as 64 hex chars.
    pub fn node_id(&self) -> String {
        self.endpoint.id().to_string()
    }

    /// Access the underlying iroh [`Endpoint`] for advanced
    /// callers that need features not yet wrapped by
    /// nexus-core-rs (e.g. direct connection dialing).
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    /// Access the Docs protocol handle.
    ///
    /// Use this to create/import documents, create authors and
    /// manage replicated key/value logs. The typed
    /// [`crate::docs`] wrapper sits on top of this raw handle.
    pub fn docs(&self) -> &Docs {
        &self.docs
    }

    /// Access the Gossip protocol handle.
    ///
    /// Use this to subscribe to topics and broadcast messages.
    /// The typed [`crate::gossip`] wrapper sits on top of this
    /// raw handle.
    pub fn gossip(&self) -> &Gossip {
        &self.gossip
    }

    /// Access the blobs content-addressed store.
    pub fn blobs_store(&self) -> &Store {
        &self.blobs_store
    }

    /// Access the in-memory address lookup registered on this
    /// node's endpoint.
    ///
    /// Callers can seed it with `EndpointInfo` / `EndpointAddr`
    /// entries learned out-of-band (typically from parsing a
    /// [`iroh_blobs::ticket::BlobTicket`] or a [`iroh_docs::DocTicket`])
    /// so the endpoint can dial peers whose relay / direct
    /// addresses are not discoverable through pkarr.
    pub fn memory_lookup(&self) -> &MemoryLookup {
        &self.memory_lookup
    }

    /// Gracefully shut down the node.
    ///
    /// Delegates to [`Router::shutdown`] which activates the
    /// router cancel token, drains every registered protocol
    /// handler to completion, and then calls `Endpoint::close()`
    /// internally. Consumes `self` so the handle cannot be reused.
    ///
    /// ## Why not `drop(router)` + `endpoint.close().await`?
    ///
    /// `Router` holds an `AbortOnDropHandle` on its run-loop task.
    /// Dropping the router aborts that task immediately, skipping
    /// the graceful sequence. Calling `endpoint.close().await`
    /// afterwards then races with whatever cleanup the aborted
    /// task had already started. Letting `Router::shutdown` drive
    /// the whole sequence is the only ordering that is race-free.
    pub async fn shutdown(self) -> Result<()> {
        debug!("shutting down SBFB node");
        self.router
            .shutdown()
            .await
            .map_err(|e| NexusError::Endpoint(format!("router shutdown failed: {e}")))?;
        if let Err(e) = self.blobs_store.shutdown().await {
            warn!(error = %e, "blobs store shutdown returned an error");
        }
        drop(self.docs);
        drop(self.gossip);
        drop(self.blobs_store);
        drop(self.memory_lookup);
        drop(self.endpoint);
        info!("SBFB node closed cleanly");
        Ok(())
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("node_id", &self.node_id())
            .finish()
    }
}

/// Boot a fresh node with a random keypair and default config.
///
/// Shortcut for `create_node_with_config(NodeConfig::default())`.
///
/// ## Errors
///
/// Returns [`NexusError::Endpoint`] if the iroh endpoint cannot
/// bind. Returns [`NexusError::Docs`] if the Docs protocol
/// handler fails to spawn.
pub async fn create_node() -> Result<Node> {
    create_node_with_config(NodeConfig::default()).await
}

/// Boot a node with an explicit configuration.
///
/// This is the general form. Use it when you need a persistent
/// identity (loaded via [`crate::KeyPair::load_or_generate`]) or
/// other non-default options. Equivalent to
/// [`create_node_with_protocols`] with no extra protocols.
pub async fn create_node_with_config(cfg: NodeConfig) -> Result<Node> {
    create_node_with_protocols(cfg, Vec::new()).await
}

/// Boot a node with an explicit configuration plus extra ALPN protocol
/// handlers built lazily once the node's stores exist.
///
/// `extra_protocols` is a list of `(alpn, factory)` pairs. Each factory
/// is invoked exactly once with the freshly-created blob store, endpoint
/// and address lookup, and its returned handler is registered on the
/// Router before it spawns. See [`ExtraProtocolFactory`] for why this
/// indirection is needed (the Router has no post-spawn registration).
///
/// The three core protocols (blobs / gossip / docs) are always
/// registered first; the extras follow.
pub async fn create_node_with_protocols(
    cfg: NodeConfig,
    extra_protocols: Vec<(Vec<u8>, ExtraProtocolFactory)>,
) -> Result<Node> {
    // Attach a MemoryLookup to every node. Callers seed it with
    // out-of-band peer addresses (e.g. parsed from blob tickets
    // or doc tickets) so that Endpoint::connect / Downloader can
    // resolve endpoint ids to dialable addrs without pkarr.
    let memory_lookup = MemoryLookup::new();

    // Sprint 81 Phase E2 (PLAN B C8) : resolve the zero-n0
    // discovery override BEFORE any builder exists. Fail-loud by
    // design — a malformed or half-configured override aborts the
    // boot here; it must never degrade into a silent partial n0
    // dependency (the n0 fleet is EOL 2026-09-30 and "partially on
    // n0" is indistinguishable from "working" until the day it is
    // not). This is the single Endpoint::builder chokepoint of the
    // workspace, so the override covers daemon, worker and
    // coordinator alike with zero per-caller wiring.
    let zero_n0_plan = crate::discovery_override::load_discovery_override()?;

    let (mut builder, using_custom_relays, home_relay) = match &zero_n0_plan {
        Some(plan) => {
            let home_relay: String = plan
                .relay_map
                .urls::<Vec<_>>()
                .into_iter()
                .next()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "custom-empty".to_string());
            // Loud LOCAL log of the discovery posture (stdout /
            // journal only, never a wire emission — local logs are
            // duress-safe by construction; the §15.1 gating concerns
            // what leaves the machine).
            info!(
                pkarr_relay_count = plan.pkarr_relays.len(),
                relay_count = plan.relay_map.len(),
                home_relay = %home_relay,
                "zero-n0 discovery override active: relays + pkarr self-hosted, no n0 service wired"
            );
            let builder =
                apply_zero_n0_discovery(Endpoint::builder(presets::Minimal), plan, &memory_lookup);
            (builder, true, home_relay)
        }
        None => {
            debug!("building iroh endpoint with the N0 preset");
            let builder = Endpoint::builder(presets::N0).address_lookup(memory_lookup.clone());

            // Sprint 18 Phase C : respect the operator's custom relay
            // list when set (via SBFB_CUSTOM_RELAYS env or
            // ~/.sbfb/relays.json). A missing / empty config falls
            // through to the N0 preset's default relay set — which tracks
            // whatever fleet the pinned iroh version ships (the hostnames
            // DID change at the 1.0 bump: the iroh-canary label was
            // dropped), not any historical set byte-for-byte.
            let custom_relays = crate::relay_config::load_relay_map()
                .map_err(|e| NexusError::Endpoint(format!("invalid relay config: {e}")))?;
            let using_custom_relays = custom_relays.is_some();
            let home_relay: String = match &custom_relays {
                Some(map) => map
                    .urls::<Vec<_>>()
                    .into_iter()
                    .next()
                    .map(|u| u.to_string())
                    .unwrap_or_else(|| "custom-empty".to_string()),
                None => "preset::N0".to_string(),
            };
            let builder = if let Some(map) = custom_relays {
                let relay_count = map.len();
                info!(
                    relay_count,
                    home_relay = %home_relay,
                    "using custom relay map from SBFB config"
                );
                builder.relay_mode(RelayMode::Custom(map))
            } else {
                debug!(home_relay = %home_relay, "no custom relay config — keeping N0 preset defaults");
                builder
            };
            (builder, using_custom_relays, home_relay)
        }
    };

    if let Some(sk_bytes) = cfg.secret_key_bytes {
        let sk = SecretKey::from_bytes(&sk_bytes);
        builder = builder.secret_key(sk);
    }

    let endpoint = builder
        .bind()
        .await
        .map_err(|e| NexusError::Endpoint(format!("bind failed: {e}")))?;

    info!(
        node_id = %endpoint.id(),
        custom_relays = using_custom_relays,
        zero_n0 = zero_n0_plan.is_some(),
        home_relay = %home_relay,
        "iroh endpoint ready"
    );

    // Spawn Blobs + Gossip + Docs protocol handlers and wire them
    // into a Router that dispatches by ALPN. This is the full
    // SBFB protocol stack every node carries.

    let blobs_store = match &cfg.data_dir {
        Some(path) => {
            let blobs_dir = path.join("blobs");
            std::fs::create_dir_all(&blobs_dir).map_err(|e| {
                NexusError::Blobs(format!("failed to create blobs dir {blobs_dir:?}: {e}"))
            })?;
            let fs_store = FsStore::load(&blobs_dir).await.map_err(|e| {
                NexusError::Blobs(format!("FsStore::load({blobs_dir:?}) failed: {e}"))
            })?;
            BlobStore::Fs(fs_store)
        }
        None => BlobStore::Mem(MemStore::default()),
    };
    let gossip = Gossip::builder().spawn(endpoint.clone());
    let docs_builder = match &cfg.data_dir {
        Some(path) => {
            std::fs::create_dir_all(path).map_err(|e| {
                NexusError::Docs(format!("failed to create docs data_dir {path:?}: {e}"))
            })?;
            Docs::persistent(path.clone())
        }
        None => Docs::memory(),
    };
    let docs = docs_builder
        .spawn(endpoint.clone(), (*blobs_store).clone(), gossip.clone())
        .await
        .map_err(|e| NexusError::Docs(format!("Docs spawn failed: {e}")))?;

    let mut router_builder = Router::builder(endpoint.clone())
        .accept(BLOBS_ALPN, BlobsProtocol::new(&blobs_store, None))
        .accept(GOSSIP_ALPN, gossip.clone())
        .accept(DOCS_ALPN, docs.clone());

    // Sprint 74 Phase E: register any extra ALPN handlers (e.g. the
    // cross-node seed protocol). Each factory is invoked now, with the
    // freshly-created store/endpoint/lookup, so a daemon-owned handler
    // can be wired before the Router spawns (no post-spawn registration
    // exists on the Router).
    for (alpn, factory) in extra_protocols {
        let handler = factory(&blobs_store, &endpoint, &memory_lookup);
        router_builder = router_builder.accept(alpn, handler);
    }

    let router = router_builder.spawn();

    Ok(Node {
        endpoint,
        docs,
        gossip,
        blobs_store,
        router,
        memory_lookup,
    })
}

/// Apply a validated zero-n0 [`DiscoveryPlan`] to an endpoint
/// builder (Sprint 81 Phase E2 — PLAN B C8).
///
/// Base contract : the caller hands a builder created from
/// `presets::Minimal` — a base that wires NOTHING n0-related
/// (`Builder::empty()` + crypto provider only), so there is nothing
/// to forget to clear. The alternative (`N0` +
/// `clear_address_lookup`) was rejected : `clear_address_lookup`
/// wipes the WHOLE lookup vector, including lookups pushed before
/// it, which makes the assembly order-sensitive and footgun-prone.
///
/// Wired in order :
///
/// 1. the caller's [`MemoryLookup`] — load-bearing : it resolves
///    peers from out-of-band ticket addresses (blob / doc tickets,
///    seed / shard paths). Any assembly that drops it silently
///    breaks ticket-based dialing.
/// 2. one [`PkarrPublisher`] AND one [`PkarrResolver`] per
///    self-hosted pkarr relay URL — publish and resolve both move
///    off n0. The publisher keeps its default relay-only address
///    filter, so no direct IP address leaks to the pkarr server
///    (same posture as the `N0` preset).
/// 3. `RelayMode::Custom` with the operator's relay map — the
///    [`crate::discovery_override::load_discovery_override`]
///    coupling guarantees it is non-empty (zero-n0 with no custom
///    relay refuses to boot instead of silently homing on n0).
///
/// Kept as a dedicated function so the hermetic two-node test below
/// exercises the EXACT assembly the production path uses (the iroh
/// `Builder` exposes no pre-bind getter on its lookup list — this
/// function boundary is the testable seam).
fn apply_zero_n0_discovery(
    builder: iroh::endpoint::Builder,
    plan: &DiscoveryPlan,
    memory_lookup: &MemoryLookup,
) -> iroh::endpoint::Builder {
    let mut builder = builder.address_lookup(memory_lookup.clone());
    for url in &plan.pkarr_relays {
        builder = builder
            .address_lookup(PkarrPublisher::builder(url.clone()))
            .address_lookup(PkarrResolver::builder(url.clone()));
    }
    builder.relay_mode(RelayMode::Custom(plan.relay_map.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;

    #[tokio::test]
    async fn create_node_returns_a_usable_handle() {
        let node = create_node()
            .await
            .expect("create_node should boot on a standard dev machine");

        let id = node.node_id();
        assert!(!id.is_empty(), "node id must be non-empty");
        assert!(
            id.len() >= 32,
            "node id should be ~64 chars hex, got {} chars",
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

    #[tokio::test]
    async fn persistent_secret_key_reboots_with_same_id() {
        let kp = KeyPair::generate();
        let secret = kp.secret_bytes();

        let cfg = NodeConfig::default().with_secret_key(secret);
        let node_a = create_node_with_config(cfg.clone()).await.unwrap();
        let id_a = node_a.node_id();
        node_a.shutdown().await.ok();

        let node_b = create_node_with_config(cfg).await.unwrap();
        let id_b = node_b.node_id();
        node_b.shutdown().await.ok();

        assert_eq!(
            id_a, id_b,
            "booting twice with the same secret key must give the same node id"
        );
    }

    #[tokio::test]
    async fn persistent_data_dir_reboots_with_same_doc_and_author() {
        use crate::docs::DocsClient;

        // Sprint 4 Phase A regression test: booting a Node with
        // the same data_dir must reopen the previously-created
        // iroh-docs namespaces and authors. Without the data_dir
        // wiring, the second boot produces fresh in-memory ids
        // and the coordinator's reboot flow breaks.
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_path_buf();
        let kp = KeyPair::generate();
        let secret = kp.secret_bytes();

        let cfg = NodeConfig::default()
            .with_secret_key(secret)
            .with_data_dir(data_dir.clone());

        // Boot #1: create an author and a doc, remember their ids.
        let node_a = create_node_with_config(cfg.clone()).await.unwrap();
        let docs_a = DocsClient::new(node_a.docs());
        let author_a = docs_a.author_create().await.unwrap();
        let doc_a = docs_a.create_doc().await.unwrap();
        let doc_a_id = doc_a.id();
        node_a.shutdown().await.ok();

        // Boot #2: reopen. The doc must be listed, the author
        // must still be in author_list.
        let node_b = create_node_with_config(cfg).await.unwrap();
        let docs_b = DocsClient::new(node_b.docs());
        let listed = docs_b.list_docs().await.unwrap();
        assert!(
            listed.contains(&doc_a_id),
            "persistent doc_id {doc_a_id:?} must survive reboot, got listed={listed:?}"
        );
        let authors = docs_b.author_list().await.unwrap();
        assert!(
            authors.contains(&author_a),
            "persistent author_id {author_a:?} must survive reboot, got authors={authors:?}"
        );
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn node_exposes_protocol_stack_handles() {
        let node = create_node().await.expect("boot");
        // These are compile-time checks: the methods exist and
        // return the expected types.
        let _docs: &Docs = node.docs();
        let _gossip: &Gossip = node.gossip();
        let _endpoint: &Endpoint = node.endpoint();
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn shutdown_closes_endpoint_without_race() {
        // Regression test for the Sprint 2 audit finding S4: the
        // old shutdown did `drop(router) + endpoint.close().await`
        // which races because Router carries an AbortOnDropHandle.
        // The new path calls `router.shutdown().await` exclusively,
        // which drives the graceful sequence. After shutdown the
        // endpoint must report itself as closed.
        let node = create_node().await.expect("boot");
        let ep = node.endpoint().clone();
        node.shutdown().await.expect("graceful shutdown");
        assert!(
            ep.is_closed(),
            "endpoint should be closed after Node::shutdown"
        );
    }

    #[tokio::test]
    async fn persistent_fsstore_survives_reboot() {
        use crate::blobs::BlobsClient;

        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().to_path_buf();
        let kp = KeyPair::generate();
        let secret = kp.secret_bytes();

        let cfg = NodeConfig::default()
            .with_secret_key(secret)
            .with_data_dir(data_dir.clone());

        let node_a = create_node_with_config(cfg.clone()).await.unwrap();
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(b"persistent-blob").await.unwrap();
        assert!(blobs_a.has(hash).await.unwrap());
        node_a.shutdown().await.unwrap();

        let node_b = create_node_with_config(cfg).await.unwrap();
        let blobs_b = BlobsClient::new(node_b.blobs_store());
        let data = blobs_b.get_bytes(hash).await.unwrap();
        assert_eq!(data, b"persistent-blob");
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn data_dir_creates_blobs_subdir() {
        let tmp = tempfile::tempdir().unwrap();
        let data_dir = tmp.path().join("iroh");
        let cfg = NodeConfig::default().with_data_dir(data_dir.clone());
        let node = create_node_with_config(cfg).await.unwrap();
        assert!(
            data_dir.join("blobs").exists(),
            "blobs/ subdir must be created inside data_dir"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn memstore_still_works_without_data_dir() {
        use crate::blobs::BlobsClient;

        let node = create_node().await.unwrap();
        let blobs = BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"mem-mode").await.unwrap();
        let data = blobs.get_bytes(hash).await.unwrap();
        assert_eq!(data, b"mem-mode");
        node.shutdown().await.ok();
    }

    /// Minimal in-process pkarr relay speaking the two HTTP verbs
    /// the production client uses: `PUT /pkarr/{z32}` (what
    /// [`iroh::address_lookup::PkarrPublisher`] sends) and
    /// `GET /pkarr/{z32}` (what [`iroh::address_lookup::PkarrResolver`]
    /// fetches). Needed because iroh's own `test_utils` pkarr server
    /// only routes PUT — its tests resolve through the test DNS
    /// server (`DnsAddressLookup`), which is NOT our production
    /// resolve path (Option B rides `PkarrResolver` HTTP). Stored
    /// payloads are opaque bytes: the signed packet round-trips
    /// verbatim, so no pkarr decoding is required here. Raw-tokio
    /// HTTP/1.1 on purpose — zero new (dev-)dependency.
    async fn run_fake_pkarr_relay() -> (
        url::Url,
        std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>>,
        tokio::task::JoinHandle<()>,
    ) {
        use std::collections::HashMap;
        use std::sync::Arc;

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::TcpListener;

        fn header_end(buf: &[u8]) -> Option<usize> {
            buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4)
        }

        let store: Arc<tokio::sync::Mutex<HashMap<String, Vec<u8>>>> = Arc::default();
        let listener = TcpListener::bind(("127.0.0.1", 0))
            .await
            .expect("fake pkarr relay binds on loopback");
        let addr = listener.local_addr().expect("bound addr");
        let url: url::Url = format!("http://{addr}/pkarr").parse().expect("valid url");

        let served = store.clone();
        let handle = tokio::spawn(async move {
            while let Ok((mut sock, _peer)) = listener.accept().await {
                let store = served.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    let mut tmp = [0u8; 4096];
                    let head_len = loop {
                        let Ok(n) = sock.read(&mut tmp).await else {
                            return;
                        };
                        if n == 0 {
                            return;
                        }
                        buf.extend_from_slice(&tmp[..n]);
                        if let Some(pos) = header_end(&buf) {
                            break pos;
                        }
                        if buf.len() > 64 * 1024 {
                            return; // header flood — drop, test-only
                        }
                    };
                    let head = String::from_utf8_lossy(&buf[..head_len]).to_string();
                    let mut lines = head.lines();
                    let request_line = lines.next().unwrap_or_default().to_string();
                    let mut content_length = 0usize;
                    for line in lines {
                        if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                            content_length = v.trim().parse().unwrap_or(0);
                        }
                    }
                    let mut body = buf[head_len..].to_vec();
                    while body.len() < content_length {
                        let Ok(n) = sock.read(&mut tmp).await else {
                            return;
                        };
                        if n == 0 {
                            break;
                        }
                        body.extend_from_slice(&tmp[..n]);
                    }
                    let mut parts = request_line.split_whitespace();
                    let method = parts.next().unwrap_or_default().to_string();
                    let path = parts.next().unwrap_or_default().to_string();
                    let response: Vec<u8> = match method.as_str() {
                        "PUT" => {
                            store.lock().await.insert(path, body);
                            b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                                .to_vec()
                        }
                        "GET" => match store.lock().await.get(&path) {
                            Some(bytes) => {
                                let mut r = format!(
                                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
                                    bytes.len()
                                )
                                .into_bytes();
                                r.extend_from_slice(bytes);
                                r
                            }
                            None => {
                                b"HTTP/1.1 404 Not Found\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                                    .to_vec()
                            }
                        },
                        _ => {
                            b"HTTP/1.1 405 Method Not Allowed\r\ncontent-length: 0\r\nconnection: close\r\n\r\n"
                                .to_vec()
                        }
                    };
                    let _ = sock.write_all(&response).await;
                    let _ = sock.shutdown().await;
                });
            }
        });

        (url, store, handle)
    }

    #[tokio::test]
    async fn zero_n0_two_nodes_converge_via_self_hosted_stack() {
        // Sprint 81 Phase E2 — the ONLY hermetic end-to-end proof
        // that the zero-n0 assembly really omits n0: two endpoints
        // built by the SAME `apply_zero_n0_discovery` function the
        // production path uses, pointed at an in-process pkarr
        // relay (PUT+GET fake above) and an in-process iroh relay
        // (`run_relay_server`, dev-dep `test-utils`), then
        // connected BY ENDPOINT ID ALONE. Resolution can only come
        // from the self-hosted pkarr relay (each endpoint gets a
        // FRESH, EMPTY MemoryLookup, so no out-of-band address can
        // leak in) and the dial can only ride the self-hosted relay
        // (the publisher's default filter publishes the relay URL
        // only — no direct IP ever reaches the pkarr server).
        //
        // Split of proof duties: the env → plan decision logic
        // (parse / policy / fail-loud coupling) is covered by the
        // discovery_override Tier-A tests; the LIVE half (real VPS
        // relay + real iroh-dns-server, zero n0 on the wire) is the
        // T2 acceptance artefact, never a unit test (hermeticity —
        // no DNS/network egress from the suite).
        //
        // Deltas vs the production assembly, all test-only: the
        // in-process relay serves a self-signed certificate, hence
        // `ca_tls_config(insecure_skip_verify)` (production trust
        // stays WebPKI-only), and a test ALPN replaces the Router
        // stack.
        use std::time::Duration;

        use iroh::test_utils::run_relay_server;
        use iroh::tls::CaTlsConfig;

        use crate::discovery_override::DiscoveryPlan;

        const TEST_ALPN: &[u8] = b"sbfb/test/zero-n0";
        const PUBLISH_TIMEOUT: Duration = Duration::from_secs(30);
        const CONNECT_DEADLINE: Duration = Duration::from_secs(30);

        let (pkarr_url, pkarr_store, _pkarr_task) = run_fake_pkarr_relay().await;
        let (relay_map, _relay_url, _relay_guard) = run_relay_server()
            .await
            .expect("in-process iroh relay boots");

        let plan = DiscoveryPlan {
            pkarr_relays: vec![pkarr_url],
            relay_map,
        };

        let build = |plan: &DiscoveryPlan| {
            apply_zero_n0_discovery(
                Endpoint::builder(presets::Minimal),
                plan,
                &MemoryLookup::new(),
            )
            .ca_tls_config(CaTlsConfig::insecure_skip_verify())
            .alpns(vec![TEST_ALPN.to_vec()])
        };

        let ep1 = build(&plan)
            .bind()
            .await
            .expect("ep1 binds under the zero-n0 assembly");
        let ep2 = build(&plan)
            .bind()
            .await
            .expect("ep2 binds under the zero-n0 assembly");

        // Accept incoming connections on ep1 (loop instead of
        // taking exactly one: retransmits can surface as extra
        // accepts — upstream test_utils does the same).
        let accept_task = tokio::spawn({
            let ep1 = ep1.clone();
            async move {
                while let Some(incoming) = ep1.accept().await {
                    if let Ok(accepting) = incoming.accept() {
                        let _conn = accepting.await;
                    }
                }
            }
        });

        // The publisher runs in a background task and republishes
        // when the home relay is established — first make sure a
        // PUT reached the fake pkarr relay at all (attributable
        // failure: publish side vs resolve/dial side).
        let publish_deadline = tokio::time::Instant::now() + PUBLISH_TIMEOUT;
        loop {
            if !pkarr_store.lock().await.is_empty() {
                break;
            }
            assert!(
                tokio::time::Instant::now() < publish_deadline,
                "ep1 never published its address to the self-hosted pkarr relay"
            );
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Connect BY ID ONLY: no ticket, no memory-lookup seed, no
        // n0 service anywhere in the graph. Retry inside a bounded
        // deadline — the very first packet published may predate
        // the home-relay establishment (no dialable addr yet); the
        // publisher republishes as soon as the relay lands.
        let connect_deadline = tokio::time::Instant::now() + CONNECT_DEADLINE;
        let conn = loop {
            match ep2.connect(ep1.id(), TEST_ALPN).await {
                Ok(conn) => break conn,
                Err(e) => {
                    assert!(
                        tokio::time::Instant::now() < connect_deadline,
                        "ep2 must resolve ep1 via the self-hosted pkarr relay and dial \
                         via the self-hosted iroh relay; last error: {e}"
                    );
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
            }
        };
        conn.close(0u32.into(), b"done");

        accept_task.abort();
        ep1.close().await;
        ep2.close().await;
    }
}
