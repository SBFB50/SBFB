// SPDX-License-Identifier: AGPL-3.0-or-later
//! # Two-node iroh-docs live sync (Sprint 1 J9-J10 prototype)
//!
//! Runs two iroh endpoints in the same process, wires them both for
//! the full iroh-docs meta protocol stack (Docs + Gossip + Blobs),
//! has node A create a document and share a ticket, has node B
//! import the ticket and subscribe to live changes, then has node A
//! write a key/value entry and asserts that node B observes it
//! within a few seconds via `LiveEvent::InsertRemote`.
//!
//! This is the Sprint 1 J9-J10 deliverable described in the SBFB
//! plan: the end-to-end smoke test that the iroh-docs primitive
//! actually syncs state across nodes on this machine before Sprint 2
//! starts wrapping it into `nexus-core-rs::docs`.
//!
//! Run with:
//!
//! ```text
//! cargo run --example two_nodes_docs_sync -p nexus-core-rs
//! ```
//!
//! Expected output on success:
//!
//! ```text
//! node A id: <hex>
//! node B id: <hex>
//! doc created, namespace: <hex>
//! ticket shared: <base32 ticket>
//! node B imported doc
//! node A wrote entry: greeting = "hello from node A"
//! node B observed InsertRemote: greeting = "hello from node A"
//! === SYNC OK ===
//! ```

use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures_lite::StreamExt;
use iroh::Endpoint;
use iroh::endpoint::presets;
use iroh::protocol::Router;
use iroh_blobs::ALPN as BLOBS_ALPN;
use iroh_blobs::BlobsProtocol;
use iroh_blobs::store::mem::MemStore;
use iroh_docs::ALPN as DOCS_ALPN;
use iroh_docs::api::protocol::{AddrInfoOptions, ShareMode};
use iroh_docs::engine::LiveEvent;
use iroh_docs::protocol::Docs;
use iroh_gossip::ALPN as GOSSIP_ALPN;
use iroh_gossip::net::Gossip;
use tokio::time::timeout;
use tracing::info;

/// A fully-wired iroh node with Docs + Gossip + Blobs protocols.
///
/// Kept as a struct so we can return all the pieces from a single
/// setup helper and clean them up at the end.
struct FullNode {
    endpoint: Endpoint,
    docs: Docs,
    _router: Router,
}

impl FullNode {
    /// Boot an endpoint and register Docs + Gossip + Blobs on it.
    async fn spawn(label: &str) -> Result<Self> {
        let endpoint = Endpoint::bind(presets::N0)
            .await
            .with_context(|| format!("{label}: failed to bind endpoint"))?;

        info!(label, id = %endpoint.id(), "endpoint ready");

        let blobs = MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let docs = Docs::memory()
            .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
            .await
            .with_context(|| format!("{label}: failed to spawn Docs"))?;

        let router = Router::builder(endpoint.clone())
            .accept(BLOBS_ALPN, BlobsProtocol::new(&blobs, None))
            .accept(GOSSIP_ALPN, gossip)
            .accept(DOCS_ALPN, docs.clone())
            .spawn();

        Ok(Self {
            endpoint,
            docs,
            _router: router,
        })
    }

    async fn shutdown(self) {
        // Router owns clones of the protocol handlers; dropping it
        // closes them. Then we explicitly close the endpoint.
        drop(self._router);
        self.endpoint.close().await;
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // --- Setup: two independent full nodes on the same machine ---

    let node_a = FullNode::spawn("A").await?;
    let node_b = FullNode::spawn("B").await?;

    println!("node A id: {}", node_a.endpoint.id());
    println!("node B id: {}", node_b.endpoint.id());

    // --- Node A creates an author and a new doc ---

    let author_a = node_a
        .docs
        .author_create()
        .await
        .context("author_create on node A")?;

    let doc_a = node_a.docs.create().await.context("create doc on node A")?;
    println!("doc created, namespace: {}", doc_a.id());

    // --- Node A shares a write-capable ticket ---

    let ticket = doc_a
        .share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses)
        .await
        .context("share ticket from node A")?;
    println!("ticket shared: {ticket}");

    // --- Node B imports the doc via the ticket ---

    let doc_b = node_b
        .docs
        .import(ticket)
        .await
        .context("import ticket on node B")?;
    println!("node B imported doc");

    // --- Node B subscribes BEFORE node A writes ---

    let mut events = doc_b.subscribe().await.context("subscribe on node B")?;

    // Give the nodes a moment to establish their connection.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // --- Node A writes an entry ---

    let key = b"greeting".to_vec();
    let value = b"hello from node A".to_vec();
    doc_a
        .set_bytes(author_a, key.clone(), value.clone())
        .await
        .context("set_bytes on node A")?;
    println!("node A wrote entry: greeting = \"hello from node A\"");

    // --- Node B waits for the InsertRemote LiveEvent ---

    let observed = timeout(Duration::from_secs(10), async {
        while let Some(event) = events.next().await {
            let ev = event.context("stream error")?;
            match ev {
                LiveEvent::InsertRemote { entry, .. } => {
                    let entry_key = entry.key().to_vec();
                    if entry_key == key {
                        return Ok::<_, anyhow::Error>(entry);
                    }
                }
                LiveEvent::SyncFinished(ev) => {
                    tracing::debug!("sync finished: {ev:?}");
                }
                LiveEvent::NeighborUp(pk) => {
                    tracing::debug!("neighbor up: {pk}");
                }
                other => {
                    tracing::debug!("other event: {other:?}");
                }
            }
        }
        Err(anyhow!("event stream ended without seeing our entry"))
    })
    .await
    .context("timed out waiting for InsertRemote after 10s")??;

    println!(
        "node B observed InsertRemote: {} = \"{}\"",
        String::from_utf8_lossy(observed.key()),
        String::from_utf8_lossy(&value),
    );
    println!("=== SYNC OK ===");

    // --- Clean shutdown ---

    node_a.shutdown().await;
    node_b.shutdown().await;

    Ok(())
}
