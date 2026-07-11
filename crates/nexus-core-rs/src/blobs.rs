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

use iroh::address_lookup::memory::MemoryLookup;
use iroh::{Endpoint, EndpointId};
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

/// Hard safety bound on the provider set [`BlobsClient::fetch_hash_multi`]
/// will dial (Sprint 75 Phase D). Enforced INSIDE the primitive — not a
/// caller convention — so no future call site can hand the downloader an
/// unbounded dial chain. Callers typically apply a tighter policy cap
/// (the daemon's directory pull uses 8); this is the never-exceed ceiling.
pub const MAX_FETCH_PROVIDERS: usize = 16;

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
    /// [`BlobsClient::get_bytes`] to retrieve the data. iroh-blobs assigns an
    /// auto-named tag on add (so a fresh blob is not GC'd immediately), but that
    /// auto-tag's name is not surfaced here. To pin a blob under a stable,
    /// removable name (e.g. the Sprint 74 keep-online pin) use
    /// [`BlobsClient::set_tag`] / [`BlobsClient::delete_tag`].
    pub async fn add_bytes(&self, data: impl AsRef<[u8]>) -> Result<[u8; 32]> {
        let bytes = data.as_ref().to_vec();
        // iroh-blobs 0.103: add_bytes() returns a TagInfo where
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

    /// Pin a blob under an explicit, deterministically-named tag so the store
    /// never garbage-collects it (Sprint 74 Phase D keep-online). The caller
    /// chooses `name` (e.g. `keep-online/<project_id>`) so it can later be
    /// removed via [`BlobsClient::delete_tag`]. Keyed PER INTENT (not per hash)
    /// so two apps sharing one blob can be pinned/unpinned independently —
    /// removing one app's tag never orphans a blob another app still pins.
    pub async fn set_tag(&self, name: &str, hash: [u8; 32]) -> Result<()> {
        let h = Hash::from_bytes(hash);
        self.inner
            .tags()
            .set(name.as_bytes(), iroh_blobs::HashAndFormat::raw(h))
            .await
            .map_err(|e| NexusError::Blobs(format!("set tag failed: {e}")))?;
        Ok(())
    }

    /// Remove a named keep-online tag. Once a blob carries no tags the store MAY
    /// garbage-collect it — though no GC is scheduled today, so this makes the
    /// blob GC-eligible, it does not free disk now (Sprint 74 Phase D; the
    /// reaper is post-launch). `delete` of a missing tag is not an error.
    pub async fn delete_tag(&self, name: &str) -> Result<()> {
        self.inner
            .tags()
            .delete(name.as_bytes())
            .await
            .map_err(|e| NexusError::Blobs(format!("delete tag failed: {e}")))?;
        Ok(())
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

    /// Download a blob by its **bare content hash** from an ordered set
    /// of candidate providers (Sprint 75 Phase D, carry PULL-2).
    ///
    /// Unlike [`BlobsClient::fetch_ticket`] this needs NO pre-existing
    /// [`BlobTicket`]: the requested object IS the BLAKE3 hash, and each
    /// provider is a bare [`EndpointId`] that iroh resolves to a dialable
    /// address through the `presets::N0` pkarr discovery wired at node
    /// boot (a caller that already knows a provider's `EndpointAddr` can
    /// seed the node's `MemoryLookup` beforehand, exactly like the
    /// ticket path does). This is the consumer leg of the PULL discovery
    /// model: a directory listing advertises `(node_id, archive_hash)`
    /// only — the producer-side `mint_ticket_for_hash` helper cannot
    /// serve here because the puller does not hold the blob yet.
    ///
    /// `providers` is **ordered** (Q5): the iroh-blobs `Downloader`
    /// consumes an `IntoIterator<Item = EndpointId>` as its provider
    /// stream in iteration order, retrying the next provider when one
    /// fails. Callers put the publishing anchor first, then the
    /// best-effort seeders. Content-addressing is the integrity gate
    /// (verrou 4 / THREAT_MODEL §15): a malicious provider in the set
    /// can only fail its own attempt — it can never serve bytes other
    /// than the exact requested hash — so the fallback chain degrades
    /// availability at worst, never authenticity. No internal timeout:
    /// the caller bounds the whole call (`tokio::time::timeout`) with
    /// its own budget. The provider set IS bounded here
    /// ([`MAX_FETCH_PROVIDERS`]) — a guardrail belongs in the
    /// primitive, not in caller conventions (S73 audit lesson), so a
    /// future caller can never hand the downloader an unbounded dial
    /// chain even if it skips the daemon-side policy cap.
    pub async fn fetch_hash_multi(
        &self,
        endpoint: &Endpoint,
        hash: [u8; 32],
        mut providers: Vec<EndpointId>,
    ) -> Result<[u8; 32]> {
        if providers.is_empty() {
            return Err(NexusError::Blobs(
                "fetch_hash_multi requires at least one provider".into(),
            ));
        }
        // Safety bound in the primitive: callers order best-first (Q5), so
        // truncating keeps the highest-priority providers.
        providers.truncate(MAX_FETCH_PROVIDERS);
        let hash = Hash::from_bytes(hash);
        let downloader = Downloader::new(self.inner, endpoint);
        downloader
            .download(hash, providers)
            .await
            .map_err(|e| NexusError::Blobs(format!("multi-provider download failed: {e}")))?;
        Ok(*hash.as_bytes())
    }

    /// Multi-provider variant of [`BlobsClient::fetch_and_pin`]
    /// (Sprint 75 Phase D): download a bare hash from an ordered
    /// provider set, then pin it under `tag_name` so the store keeps it.
    /// Used by the voluntary-seed path for a directory-only app, where
    /// no `BlobTicket` exists — only `(anchor node_id, archive_hash)`
    /// plus the best-effort seeder set.
    pub async fn fetch_and_pin_multi(
        &self,
        endpoint: &Endpoint,
        hash: [u8; 32],
        providers: Vec<EndpointId>,
        tag_name: &str,
    ) -> Result<[u8; 32]> {
        let hash = self.fetch_hash_multi(endpoint, hash, providers).await?;
        self.set_tag(tag_name, hash).await?;
        Ok(hash)
    }

    /// Fetch a blob by ticket AND immediately pin it under a removable
    /// tag so the store does not garbage-collect it (Sprint 74 Phase E).
    ///
    /// Composes [`BlobsClient::fetch_ticket`] + [`BlobsClient::set_tag`].
    /// A seeder calls this when it agrees to keep a remote app online:
    /// `fetch_ticket` alone leaves the blob untagged (GC-eligible — that
    /// is fine for the re-fetchable curator-list flow, but a seed pin
    /// must survive), so this method tags it under `tag_name`
    /// (typically `keep-online/<project_id>`, the same tag the local
    /// keep-online toggle uses, unifying the ON/OFF + boot re-announce
    /// machinery). Content-addressing (BLAKE3) guarantees the fetched
    /// bytes can only be the exact `ticket.hash()` — a malicious source
    /// cannot serve altered content.
    pub async fn fetch_and_pin(
        &self,
        endpoint: &Endpoint,
        memory_lookup: &MemoryLookup,
        ticket_str: &str,
        tag_name: &str,
    ) -> Result<[u8; 32]> {
        let hash = self
            .fetch_ticket(endpoint, memory_lookup, ticket_str)
            .await?;
        self.set_tag(tag_name, hash).await?;
        Ok(hash)
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

    #[test]
    fn blob_ticket_string_round_trips_under_current_lock() {
        // Sprint 81 Phase D — locks the persisted-ticket string contract.
        // The daemon persists `AnchorLocator.ticket` in `anchors.json` as a
        // `BlobTicket` string (written at directory ingest, re-parsed by the
        // boot re-pull via `BlobTicket::from_str`), so the encode/parse pair
        // must stay stable under the pinned iroh-blobs. A POPULATED
        // EndpointAddr (relay + direct addr) mirrors what a live node mints —
        // an id-only addr would under-test the persisted shape. The addr must
        // stay within what the ticket wire preserves — one relay + IP addrs
        // (the encode keeps `relay_urls().next()` + `ip_addrs()` and drops
        // extra relays / Custom transports by design), or the verbatim addr
        // assert below would fail by wire design, not by regression. Pure
        // encode/parse: no node, no store, no dial.
        use std::net::SocketAddr;

        use iroh_blobs::BlobFormat;

        let id = EndpointId::from_str(&hex::encode(crate::KeyPair::generate().public_bytes()))
            .expect("a fresh Ed25519 pubkey is a valid EndpointId");
        let relay = iroh::RelayUrl::from_str("https://relay.sbfb.invalid./")
            .expect("static relay URL parses");
        let direct: SocketAddr = "192.0.2.7:4433".parse().expect("static socket addr parses");
        let addr = iroh::EndpointAddr::new(id)
            .with_relay_url(relay)
            .with_ip_addr(direct);

        let hash = Hash::new(b"anchors-json-ticket-contract");
        let ticket = BlobTicket::new(addr.clone(), hash, BlobFormat::Raw);
        let ticket_str = ticket.to_string();

        let parsed =
            BlobTicket::from_str(&ticket_str).expect("a minted ticket string must re-parse");
        assert_eq!(
            parsed.to_string(),
            ticket_str,
            "string encoding is idempotent"
        );
        let (got_addr, got_hash, got_format) = parsed.into_parts();
        assert_eq!(got_hash, hash, "hash survives the string round-trip");
        assert_eq!(
            got_format,
            BlobFormat::Raw,
            "format survives the string round-trip"
        );
        assert_eq!(
            got_addr, addr,
            "populated EndpointAddr (id + relay + direct) survives verbatim"
        );
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

        // Mint a BlobTicket that embeds node A's DIRECT socket addrs
        // ONLY (S81 Phase K): stripping the relay URL pins the transfer
        // to the loopback path, so this test is hermetic — it neither
        // races through nor depends on the public n0 relay (EOL
        // 2026-09-30). Poll until a direct addr exists (the local
        // socket binds well before any relay handshake).
        let my_addr = {
            let disco = crate::discovery::DiscoveryClient::new(node_a.endpoint());
            let mut direct_only = None;
            for _ in 0..100 {
                let full = disco
                    .my_endpoint_addr()
                    .await
                    .expect("node A should publish its address");
                let directs: Vec<std::net::SocketAddr> = full.ip_addrs().copied().collect();
                if !directs.is_empty() {
                    let mut addr = iroh::EndpointAddr::new(full.id);
                    for d in directs {
                        addr = addr.with_ip_addr(d);
                    }
                    direct_only = Some(addr);
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            direct_only.expect("node A must expose a direct socket addr")
        };

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

    #[tokio::test]
    async fn fetch_hash_multi_rejects_empty_providers() {
        // An empty provider set is a caller bug (the directory resolution
        // produced nothing dialable) — surfaced as a hard error, never a
        // silent hang on a download that can have no source.
        let node = spawn_node().await;
        let blobs = BlobsClient::new(node.blobs_store());
        let err = blobs
            .fetch_hash_multi(node.endpoint(), [7u8; 32], Vec::new())
            .await;
        assert!(err.is_err(), "empty provider vec must error immediately");
        node.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fetch_falls_back_to_seeder_when_anchor_offline() {
        // Sprint 75 Phase D — the load-bearing multi-provider FALLBACK test
        // (plan D.3 #1). A directory-only app advertises (anchor node_id,
        // archive_hash). When the anchor is dead, the download must still
        // succeed via another provider in the vec (a seeder that actually
        // holds the BLAKE3) instead of failing outright, and the bytes must
        // be the exact requested hash. The "anchor" here is an EndpointId
        // derived from a keypair that never booted a node — pkarr has no
        // record for it, so its dial attempt can only fail.
        //
        // Honest scope: this proves dead-provider RESILIENCE + integrity. It
        // does NOT pin that the anchor is dialed strictly FIRST — the
        // anchor-first ordering of the vec is asserted at construction
        // (`fetch_provider_ordering`, daemon side), and the in-order
        // consumption is iroh-blobs 0.103 documented behavior (blanket
        // `ContentDiscovery for IntoIterator` yields iteration order);
        // instrumenting actual dial order would require a protocol shim.
        use std::time::Duration;

        let seeder = spawn_node().await;
        let puller = spawn_node().await;

        let blobs_seeder = BlobsClient::new(seeder.blobs_store());
        let payload = b"directory-only-app-archive-bytes".to_vec();
        let hash = blobs_seeder.add_bytes(&payload).await.unwrap();

        // Dead anchor: a valid Ed25519 endpoint id that was never published.
        let dead_anchor =
            EndpointId::from_str(&hex::encode(crate::KeyPair::generate().public_bytes()))
                .expect("a fresh Ed25519 pubkey is a valid EndpointId");
        let seeder_id = EndpointId::from_str(&seeder.node_id()).unwrap();

        // Seed the puller's lookup with the seeder's address so the fallback
        // leg can dial without depending on live pkarr propagation timing.
        let seeder_addr = crate::discovery::DiscoveryClient::new(seeder.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must publish its address");
        puller.memory_lookup().add_endpoint_info(seeder_addr);

        let blobs_puller = BlobsClient::new(puller.blobs_store());
        let fetched = tokio::time::timeout(
            Duration::from_secs(120),
            blobs_puller.fetch_hash_multi(puller.endpoint(), hash, vec![dead_anchor, seeder_id]),
        )
        .await
        .expect("fallback download must complete within the test budget")
        .expect("download must succeed via the seeder fallback");
        assert_eq!(fetched, hash, "returned hash must be the requested hash");
        assert!(
            blobs_puller.has(hash).await.unwrap(),
            "the puller now holds the blob fetched from the seeder"
        );
        let got = blobs_puller.get_bytes(hash).await.unwrap();
        assert_eq!(got, payload, "content matches the author bytes (BLAKE3)");

        seeder.shutdown().await.ok();
        puller.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn fetch_hash_multi_fails_when_provider_lacks_hash() {
        // Integrity invariant (verrou 4 / THREAT_MODEL §15): a provider that
        // does NOT hold the requested hash can only fail the download — it
        // can never substitute different bytes, because the requested object
        // IS the BLAKE3 hash. Ask a live node for a hash it does not have.
        use std::time::Duration;

        let provider = spawn_node().await;
        let puller = spawn_node().await;

        // The provider holds SOME blob, but we request a DIFFERENT hash.
        let blobs_provider = BlobsClient::new(provider.blobs_store());
        blobs_provider
            .add_bytes(b"some-other-content")
            .await
            .unwrap();
        let absent_hash = *Hash::new(b"content-the-provider-never-stored").as_bytes();

        let provider_addr = crate::discovery::DiscoveryClient::new(provider.endpoint())
            .my_endpoint_addr()
            .await
            .expect("provider must publish its address");
        puller.memory_lookup().add_endpoint_info(provider_addr);
        let provider_id = EndpointId::from_str(&provider.node_id()).unwrap();

        let blobs_puller = BlobsClient::new(puller.blobs_store());
        let res = tokio::time::timeout(
            Duration::from_secs(120),
            blobs_puller.fetch_hash_multi(puller.endpoint(), absent_hash, vec![provider_id]),
        )
        .await
        .expect("the failed download must resolve within the test budget");
        assert!(
            res.is_err(),
            "a provider lacking the hash must fail the fetch, never serve other bytes"
        );
        assert!(
            !blobs_puller.has(absent_hash).await.unwrap(),
            "nothing was stored under the requested hash"
        );

        provider.shutdown().await.ok();
        puller.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seeder_fetches_tags_pins_blob() {
        // Sprint 74 Phase E (R3): fetch_and_pin downloads a remote blob AND
        // pins it under a removable tag so the seeder's store keeps it. The
        // returned hash matches the ticket; the blob is present locally and the
        // pin tag is set.
        use iroh_blobs::BlobFormat;

        let node_a = spawn_node().await;
        let node_b = spawn_node().await;

        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let payload = b"seed-me-please-author-signed".to_vec();
        let hash_bytes = blobs_a.add_bytes(&payload).await.unwrap();

        let my_addr = crate::discovery::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node A should publish its address");
        let ticket = BlobTicket::new(my_addr, Hash::from_bytes(hash_bytes), BlobFormat::Raw);

        let blobs_b = BlobsClient::new(node_b.blobs_store());
        let tag = "keep-online/seed-test-app";
        let fetched = blobs_b
            .fetch_and_pin(
                node_b.endpoint(),
                node_b.memory_lookup(),
                &ticket.to_string(),
                tag,
            )
            .await
            .expect("fetch_and_pin should succeed on loopback");
        assert_eq!(fetched, hash_bytes, "returned hash matches the ticket");
        assert!(
            blobs_b.has(hash_bytes).await.unwrap(),
            "the seeder now holds the blob"
        );
        // The pin tag is present (proves set_tag ran inside fetch_and_pin).
        let tags = node_b.blobs_store().tags();
        let found = tags
            .get(tag.as_bytes())
            .await
            .expect("tags().get must not error");
        assert!(found.is_some(), "fetch_and_pin must leave a pin tag behind");

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }
}
