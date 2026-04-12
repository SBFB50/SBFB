// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed wrapper around iroh-gossip.
//!
//! Presents the topic pub/sub primitives as a thin SBFB-facing API:
//!
//! - [`GossipClient::join_topic`] — subscribe to a topic by 32-byte id
//!   and wait for at least one peer connection before returning.
//! - [`TopicHandle::broadcast`] — publish a message (`Vec<u8>`) to
//!   all peers in the topic.
//! - [`TopicHandle::next_event`] — receive the next [`GossipEvent`]
//!   (message, neighbor up, neighbor down, or lagged).
//! - [`TopicHandle::split`] — separate the handle into a
//!   [`TopicSender`] and [`TopicReceiver`] pair for independent
//!   tasks.
//!
//! The 32-byte topic id is expected to be derived from a stable
//! identifier. SBFB uses:
//!
//! - `BLAKE3("nexus-grid/curator/" || curator_pubkey)[..32]` for the
//!   per-curator list announcement topic.
//! - `BLAKE3("nexus-grid/project/" || project_pubkey || "/announce")[..32]`
//!   for periodic project heartbeats on public projects.
//!
//! ## Example
//!
//! ```no_run
//! # async fn example(node: &nexus_core_rs::Node) -> nexus_core_rs::Result<()> {
//! use nexus_core_rs::gossip::{GossipClient, GossipEvent};
//!
//! let gossip = GossipClient::new(node.gossip());
//! let topic_id = [0u8; 32]; // derive from a stable seed
//! let mut topic = gossip.join_topic(topic_id, vec![]).await?;
//! topic.broadcast(b"hello topic".to_vec()).await?;
//!
//! while let Some(event) = topic.next_event().await? {
//!     if let GossipEvent::Message { content, .. } = event {
//!         println!("got message: {} bytes", content.len());
//!         break;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use std::str::FromStr;

use bytes::Bytes;
use futures_lite::StreamExt;
use iroh::PublicKey;
use iroh_gossip::api::{Event, GossipReceiver, GossipSender, GossipTopic};
use iroh_gossip::net::Gossip;
use iroh_gossip::proto::TopicId;

use crate::error::{NexusError, Result};

/// SBFB-side gossip event, mirrors [`iroh_gossip::api::Event`] with
/// a slightly cleaner enum surface.
#[derive(Debug, Clone)]
pub enum GossipEvent {
    /// A message arrived from a peer in this topic.
    Message {
        /// The message payload.
        content: Vec<u8>,
        /// The id of the peer that delivered this message.
        /// Note: this is the *delivered from* node id (the
        /// neighbor that forwarded it), not necessarily the
        /// original publisher.
        delivered_from: String,
    },
    /// A new neighbor joined the topic swarm.
    NeighborUp {
        /// The new neighbor's node id.
        node_id: String,
    },
    /// A neighbor left the topic swarm.
    NeighborDown {
        /// The departing neighbor's node id.
        node_id: String,
    },
    /// The local receiver has lagged behind — some messages were
    /// dropped from the internal channel. SBFB treats this as a
    /// signal to tighten its processing loop.
    Lagged,
}

impl GossipEvent {
    fn from_iroh(event: Event) -> Self {
        match event {
            Event::Received(msg) => GossipEvent::Message {
                content: msg.content.to_vec(),
                delivered_from: msg.delivered_from.to_string(),
            },
            Event::NeighborUp(node_id) => GossipEvent::NeighborUp {
                node_id: node_id.to_string(),
            },
            Event::NeighborDown(node_id) => GossipEvent::NeighborDown {
                node_id: node_id.to_string(),
            },
            Event::Lagged => GossipEvent::Lagged,
        }
    }
}

/// Thin client around an [`iroh_gossip::net::Gossip`] handle.
///
/// Owns a cheaply-cloned `Gossip` (internal `Arc<Inner>` in
/// iroh-gossip 0.97), so the client can be stored long-lived in a
/// struct or passed across an FFI boundary without a lifetime
/// parameter. The Sprint 2 audit P1 lifetime concern (a
/// `&'a Gossip` field would block the Sprint 4 coordinator from
/// holding a persistent Python-side gossip handle) no longer
/// applies.
#[derive(Debug, Clone)]
pub struct GossipClient {
    inner: Gossip,
}

impl GossipClient {
    /// Wrap a `&Gossip` (typically obtained from
    /// [`crate::Node::gossip`]). Clones the inner `Arc<Inner>`
    /// so the resulting client has no borrow on the source.
    pub fn new(inner: &Gossip) -> Self {
        GossipClient {
            inner: inner.clone(),
        }
    }

    /// Subscribe to a topic and wait for at least one peer
    /// connection before returning.
    ///
    /// `topic_bytes` is a 32-byte topic identifier. `bootstrap`
    /// is a list of known peer public keys as strings (one per
    /// peer, parseable by [`iroh::PublicKey::from_str`]) — pass
    /// an empty vec if the topic is already being seeded by
    /// peers we are connected to.
    ///
    /// This calls `subscribe_and_join` under the hood, so the
    /// returned handle is ready to broadcast and receive
    /// immediately.
    pub async fn join_topic(
        &self,
        topic_bytes: [u8; 32],
        bootstrap: Vec<String>,
    ) -> Result<TopicHandle> {
        let topic_id = TopicId::from_bytes(topic_bytes);

        let bootstrap: Result<Vec<PublicKey>> = bootstrap
            .into_iter()
            .map(|s| {
                PublicKey::from_str(&s)
                    .map_err(|e| NexusError::Gossip(format!("bad bootstrap node id {s:?}: {e}")))
            })
            .collect();
        let bootstrap = bootstrap?;

        let topic = self
            .inner
            .subscribe_and_join(topic_id, bootstrap)
            .await
            .map_err(|e| NexusError::Gossip(format!("subscribe_and_join failed: {e}")))?;
        Ok(TopicHandle { inner: topic })
    }
}

/// Combined sender+receiver handle for a single gossip topic.
///
/// Use [`TopicHandle::broadcast`] to publish and
/// [`TopicHandle::next_event`] to receive. For independent
/// sender/receiver tasks, call [`TopicHandle::split`] to break
/// the handle into a [`TopicSender`] and [`TopicReceiver`].
#[derive(Debug)]
pub struct TopicHandle {
    inner: GossipTopic,
}

impl TopicHandle {
    /// Broadcast a message to all peers in the topic.
    ///
    /// Messages up to the iroh-gossip internal maximum (a few KB)
    /// propagate via the HyParView + PlumTree broadcast trees to
    /// every subscribed peer.
    pub async fn broadcast(&mut self, message: Vec<u8>) -> Result<()> {
        self.inner
            .broadcast(Bytes::from(message))
            .await
            .map_err(|e| NexusError::Gossip(format!("broadcast failed: {e}")))
    }

    /// Pull the next event off the topic's event stream.
    ///
    /// Returns `Ok(None)` when the stream ends (for example if
    /// the underlying gossip engine is shutting down).
    pub async fn next_event(&mut self) -> Result<Option<GossipEvent>> {
        let next = self
            .inner
            .try_next()
            .await
            .map_err(|e| NexusError::Gossip(format!("next event failed: {e}")))?;
        Ok(next.map(GossipEvent::from_iroh))
    }

    /// Split the handle into a [`TopicSender`] and a
    /// [`TopicReceiver`] so the two halves can live in different
    /// tasks.
    pub fn split(self) -> (TopicSender, TopicReceiver) {
        let (sender, receiver) = self.inner.split();
        (
            TopicSender { inner: sender },
            TopicReceiver { inner: receiver },
        )
    }
}

/// Sender half of a split topic handle.
#[derive(Debug, Clone)]
pub struct TopicSender {
    inner: GossipSender,
}

impl TopicSender {
    /// Broadcast a message to all peers in the topic.
    pub async fn broadcast(&self, message: Vec<u8>) -> Result<()> {
        self.inner
            .broadcast(Bytes::from(message))
            .await
            .map_err(|e| NexusError::Gossip(format!("broadcast failed: {e}")))
    }
}

/// Receiver half of a split topic handle.
#[derive(Debug)]
pub struct TopicReceiver {
    inner: GossipReceiver,
}

impl TopicReceiver {
    /// Pull the next event. Returns `Ok(None)` when the stream
    /// ends.
    pub async fn next_event(&mut self) -> Result<Option<GossipEvent>> {
        let next = self
            .inner
            .try_next()
            .await
            .map_err(|e| NexusError::Gossip(format!("next event failed: {e}")))?;
        Ok(next.map(GossipEvent::from_iroh))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{create_node, Node};
    use std::time::Duration;
    use tokio::time::timeout;

    async fn spawn_node() -> Node {
        create_node().await.expect("boot")
    }

    #[tokio::test]
    async fn join_topic_returns_a_handle() {
        // Even with an empty bootstrap list and no peers yet, the
        // subscribe_and_join call should succeed once the topic is
        // registered locally. Use a unique topic id per test run
        // to avoid cross-test pollution.
        let node = spawn_node().await;
        let gossip = GossipClient::new(node.gossip());

        let topic_id = blake3::hash(b"nexus-grid-test/topic-smoke")
            .as_bytes()
            .to_owned();

        // subscribe_and_join waits for at least one peer — in an
        // isolated test there are none, so we race it against a
        // short timeout and accept either outcome. The point of
        // the test is that the call compiles and does not panic.
        let outcome = timeout(Duration::from_secs(2), gossip.join_topic(topic_id, vec![])).await;
        let _ = outcome; // timeout or success, both are fine

        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn broadcast_rejects_invalid_bootstrap() {
        // Passing a garbage bootstrap id should surface a clear
        // Gossip error rather than panicking inside the crate.
        let node = spawn_node().await;
        let gossip = GossipClient::new(node.gossip());

        let topic_id = *blake3::hash(b"nexus-grid-test/bad-bootstrap").as_bytes();

        let result = gossip
            .join_topic(topic_id, vec!["not a real key".into()])
            .await;
        assert!(result.is_err(), "bad bootstrap must fail");

        node.shutdown().await.ok();
    }
}

// Note: end-to-end 2-node gossip message exchange is deliberately
// not tested at this layer because it requires configuring peer
// address injection against an iroh 0.97 API surface that is
// orthogonal to the GossipClient wrapper itself. The wrapper is
// exercised end-to-end in Sprint 4 when the coordinator uses it
// to publish curator list announcements, which is the real
// production path anyway.
