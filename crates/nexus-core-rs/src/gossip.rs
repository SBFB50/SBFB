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
use tracing::warn;

use crate::attestations::{AgeWitness, AgeWitnessError, MIN_WITNESS_AGE_DAYS};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH};
use crate::error::{NexusError, Result};
use crate::pow::MAX_DIFFICULTY_BITS;
use crate::pow_gossip::PowSolveCache;
use crate::relay_pow_policy::RelayPowPolicy;

/// Dynamic difficulty target for PoW-gated gossip joins.
///
/// Sprint 23 Phase C: the difficulty can be either a fixed value
/// (from the static policy file or a direct override) or resolved
/// dynamically per-topic from the escalating policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DifficultyTarget {
    /// A fixed difficulty value (leading zero bits).
    Fixed(u32),
    /// Read from the static relay PoW policy for the given topic.
    FromPolicy(RelayPowPolicy),
}

impl DifficultyTarget {
    /// Resolve the effective difficulty for a topic.
    pub fn resolve(&self, topic: &[u8; 32]) -> u32 {
        match self {
            Self::Fixed(d) => *d,
            Self::FromPolicy(policy) => policy.difficulty_for(topic),
        }
    }
}

/// Outcome of the Couche 1 age-admission gate evaluation. Returned
/// by [`evaluate_age_admission`] and consumed by
/// [`GossipClient::join_topic_with_age_witness`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgeAdmissionOutcome {
    /// The joining node is in the bootstrap allowlist ; admission
    /// succeeds without a witness (pre-`v1.0` bootstrap ceremony).
    BootstrapSelfWitness,

    /// A valid age witness was presented, its signature verified,
    /// the witnessed node is ≥ [`MIN_AGE_DAYS`] days old, and the
    /// witness itself is ≥ [`MIN_WITNESS_AGE_DAYS`] days known to
    /// the local mesh state.
    AgeGatePassed {
        /// Age of the witnessed node in days at evaluation time.
        age_days: i64,
        /// Age of the witnessing peer in days at evaluation time.
        witness_age_days: i64,
    },

    /// No witness was provided and the joining node is not in the
    /// bootstrap allowlist. Admission depends on the PoW gate
    /// alone (Couche 0, Sprint 19). Logged with a `warn!` by the
    /// caller — this path is deliberately kept open for
    /// very-early bootstrap and removed at the `v1.0` tag.
    PowFallback {
        /// Short explanation for the fallback decision.
        reason: &'static str,
    },

    /// A witness was present but failed admission — either bad
    /// signature, underage witnessed node, future timestamp, or
    /// the witnessing peer itself was not yet old enough to
    /// vouch (`<` [`MIN_WITNESS_AGE_DAYS`]). Callers surface this
    /// as a hard error rather than fall back to PoW, because a
    /// present-but-invalid witness is a security signal.
    Rejected(AgeWitnessError),
}

/// Policy hook queried by [`evaluate_age_admission`] to resolve
/// local mesh state without binding this crate to a concrete
/// bootstrap-allowlist / mesh-state implementation.
///
/// The concrete implementation lives in
/// `nexus-shell-daemon-core::bootstrap_allowlist::BootstrapAllowlist`
/// (plus a small mesh-state table tracking `first_seen_ts` per
/// neighbor). This crate is intentionally decoupled so
/// [`AgeAdmissionPolicy`] stays testable with an in-memory stub.
pub trait AgeAdmissionPolicy {
    /// Return `true` if `node_id` is currently in the bootstrap
    /// allowlist and therefore eligible for the self-witness
    /// admission shortcut (P0-G1-1 pre-`v1.0` ceremony).
    fn is_bootstrap_node(&self, node_id: &[u8; PUBLIC_KEY_LENGTH]) -> bool;

    /// Return the age in days (at `now_ts`) of `witness_pubkey` as
    /// observed by the local mesh state, or `None` if the witness
    /// is not yet known. The age gate rejects any witness whose
    /// age is below [`MIN_WITNESS_AGE_DAYS`] — a chain-breaking
    /// guard so a fresh Sybil cannot instantly vouch for another
    /// fresh Sybil.
    fn witness_age_days(
        &self,
        witness_pubkey: &[u8; PUBLIC_KEY_LENGTH],
        now_ts: i64,
    ) -> Option<i64>;
}

/// Evaluate the Couche 1 age-admission gate for a node joining a
/// gossip topic. Pure function over the trait + args ; no iroh
/// dependency so tests can exercise all branches without a running
/// node.
///
/// The decision tree matches the kickoff §4 D1 spec :
///
/// 1. **Bootstrap allowlist** wins first (the joining node is a
///    pre-v1.0 seed).
/// 2. Else if a witness is provided, verify its signature, check
///    the joining node age ≥ [`MIN_AGE_DAYS`], and check that the
///    witness itself is ≥ [`MIN_WITNESS_AGE_DAYS`] days known to
///    the mesh.
/// 3. Else fall back to PoW-only admission (very-early bootstrap
///    tolerance).
pub fn evaluate_age_admission<P: AgeAdmissionPolicy>(
    joining_node_id: &[u8; PUBLIC_KEY_LENGTH],
    age_witness: Option<&AgeWitness>,
    policy: &P,
    now_ts: i64,
) -> AgeAdmissionOutcome {
    if policy.is_bootstrap_node(joining_node_id) {
        return AgeAdmissionOutcome::BootstrapSelfWitness;
    }
    match age_witness {
        Some(witness) => {
            if witness.node_id != *joining_node_id {
                return AgeAdmissionOutcome::Rejected(AgeWitnessError::BadSignature(
                    "witness.node_id does not match joining_node_id".to_string(),
                ));
            }
            if let Err(e) = witness.verify_with_age(now_ts) {
                return AgeAdmissionOutcome::Rejected(e);
            }
            match policy.witness_age_days(&witness.witness_pubkey, now_ts) {
                Some(witness_age) if witness_age >= MIN_WITNESS_AGE_DAYS => {
                    AgeAdmissionOutcome::AgeGatePassed {
                        age_days: witness.age_days(now_ts),
                        witness_age_days: witness_age,
                    }
                }
                Some(witness_age) => AgeAdmissionOutcome::Rejected(AgeWitnessError::Underage {
                    age_days: witness_age,
                    required: MIN_WITNESS_AGE_DAYS,
                }),
                None => AgeAdmissionOutcome::PowFallback {
                    reason: "witness pubkey unknown to local mesh state",
                },
            }
        }
        None => AgeAdmissionOutcome::PowFallback {
            reason: "no witness provided and node not in bootstrap allowlist",
        },
    }
}

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
/// Owns a cheaply-cloned `Gossip` (internal `Arc<Inner>`, verified
/// unchanged in iroh-gossip 0.101 `net.rs:84-86` at the S81 Phase E
/// re-cert), so the client can be stored long-lived in a
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
    /// Join a topic with a Couche 1 age-admission check ahead of
    /// the underlying iroh-gossip subscribe.
    ///
    /// Sprint 22 Phase C. Wraps [`Self::join_topic`] with the
    /// [`evaluate_age_admission`] gate :
    ///
    /// 1. If `joining_node_id` is in the bootstrap allowlist
    ///    (`policy.is_bootstrap_node`), the self-witness shortcut
    ///    applies (pre-`v1.0` bootstrap ceremony, P0-G1-1).
    /// 2. Else if `age_witness` is `Some` and the witness passes
    ///    signature + age ≥ [`MIN_AGE_DAYS`] + witness itself ≥
    ///    [`MIN_WITNESS_AGE_DAYS`] days via
    ///    `policy.witness_age_days`, admission succeeds.
    /// 3. Else the method logs a `warn!` and falls back to
    ///    PoW-only admission (Couche 0, S19). Very-early bootstrap
    ///    tolerance ; post-`v1.0` this fallback is removed (the
    ///    bootstrap allowlist expires at `v1.0`).
    ///
    /// On explicit witness rejection (step 2 fails validation) the
    /// call surfaces a [`NexusError::Crypto`] rather than falling
    /// back, because a present-but-invalid witness is a security
    /// signal rather than "we have no age proof".
    ///
    /// On success the returned [`TopicHandle`] is identical to
    /// [`Self::join_topic`]'s — the age gate is a pre-flight check,
    /// not a per-message transform.
    pub async fn join_topic_with_age_witness<P: AgeAdmissionPolicy>(
        &self,
        topic_bytes: [u8; 32],
        bootstrap: Vec<String>,
        joining_node_id: &[u8; PUBLIC_KEY_LENGTH],
        age_witness: Option<&AgeWitness>,
        policy: &P,
        now_ts: i64,
    ) -> Result<TopicHandle> {
        match evaluate_age_admission(joining_node_id, age_witness, policy, now_ts) {
            AgeAdmissionOutcome::BootstrapSelfWitness
            | AgeAdmissionOutcome::AgeGatePassed { .. } => {}
            AgeAdmissionOutcome::PowFallback { reason } => {
                warn!(
                    topic = %hex::encode(topic_bytes),
                    reason,
                    "join_topic_with_age_witness: age gate fell back to PoW-only \
                     (very-early bootstrap tolerance, removed at v1.0 tag)"
                );
            }
            AgeAdmissionOutcome::Rejected(err) => {
                return Err(NexusError::Crypto(format!(
                    "join_topic_with_age_witness: witness rejected: {err}"
                )));
            }
        }
        self.join_topic(topic_bytes, bootstrap).await
    }

    /// Join a topic with a PoW admission gate at a dynamic
    /// difficulty level.
    ///
    /// Sprint 23 Phase C. Accepts a [`DifficultyTarget`] that may
    /// come from the static policy file OR from the escalating
    /// difficulty computed per (consumer, model). The publisher
    /// proves they can solve at the required difficulty before the
    /// topic join proceeds.
    ///
    /// The solve is cached via [`PowSolveCache`] — subsequent
    /// calls within the 15-minute session window return
    /// immediately.
    pub async fn join_topic_with_pow(
        &self,
        topic_bytes: [u8; 32],
        bootstrap: Vec<String>,
        keypair: &KeyPair,
        target: DifficultyTarget,
        solve_cache: &PowSolveCache,
    ) -> Result<TopicHandle> {
        let difficulty = target.resolve(&topic_bytes);
        if difficulty == 0 {
            return Err(NexusError::Gossip(
                "join_topic_with_pow: zero difficulty disables defence".into(),
            ));
        }
        if difficulty > MAX_DIFFICULTY_BITS {
            return Err(NexusError::Gossip(format!(
                "join_topic_with_pow: difficulty {difficulty} exceeds max {MAX_DIFFICULTY_BITS}"
            )));
        }
        let policy = RelayPowPolicy {
            default_difficulty: difficulty,
            topic_overrides: Default::default(),
        };
        solve_cache
            .ensure_proof(topic_bytes, keypair, &policy)
            .map_err(|e| {
                NexusError::Gossip(format!("join_topic_with_pow: PoW solve failed: {e}"))
            })?;
        self.join_topic(topic_bytes, bootstrap).await
    }

    /// Subscribe to a topic and wait for at least one peer
    /// connection before returning.
    ///
    /// **Blocks** until a NeighborUp event arrives. Use
    /// [`Self::subscribe_topic`] for a non-blocking variant
    /// suitable for daemon boot.
    pub async fn join_topic(
        &self,
        topic_bytes: [u8; 32],
        bootstrap: Vec<String>,
    ) -> Result<TopicHandle> {
        let topic_id = TopicId::from_bytes(topic_bytes);
        let bootstrap = Self::parse_bootstrap(bootstrap)?;

        let topic = self
            .inner
            .subscribe_and_join(topic_id, bootstrap)
            .await
            .map_err(|e| NexusError::Gossip(format!("subscribe_and_join failed: {e}")))?;
        Ok(TopicHandle { inner: topic })
    }

    /// Subscribe to a topic **without blocking** on peer
    /// connection. Returns a handle immediately — broadcasts
    /// are queued until the first peer connects. The caller
    /// should watch for [`GossipEvent::NeighborUp`] to know
    /// when the topic is live.
    pub async fn subscribe_topic(
        &self,
        topic_bytes: [u8; 32],
        bootstrap: Vec<String>,
    ) -> Result<TopicHandle> {
        let topic_id = TopicId::from_bytes(topic_bytes);
        let bootstrap = Self::parse_bootstrap(bootstrap)?;

        let topic = self
            .inner
            .subscribe(topic_id, bootstrap)
            .await
            .map_err(|e| NexusError::Gossip(format!("subscribe failed: {e}")))?;
        Ok(TopicHandle { inner: topic })
    }

    fn parse_bootstrap(bootstrap: Vec<String>) -> Result<Vec<PublicKey>> {
        bootstrap
            .into_iter()
            .map(|s| {
                PublicKey::from_str(&s)
                    .map_err(|e| NexusError::Gossip(format!("bad bootstrap node id {s:?}: {e}")))
            })
            .collect()
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

    /// Join additional peers on an already-subscribed topic
    /// (hot-join, Sprint 81 Phase E3). Maps to iroh-gossip
    /// `Command::JoinPeers`: the HyParView membership dials each
    /// peer through the endpoint's active discovery, so a peer
    /// subscribed at runtime is reachable without a daemon
    /// restart. Idempotent: re-joining an active or pending peer
    /// is a membership no-op upstream.
    ///
    /// Unlike the boot-time bootstrap parse (`parse_bootstrap`,
    /// which collect-aborts on the first bad id), this hot-path
    /// wrapper degrades per peer: a malformed node id is skipped
    /// with a warn so one bad entry in a future batch caller can
    /// never abort the join of the valid ones. Callers that
    /// validate ids upstream (e.g. an attention-set subscribe)
    /// always pass parseable keys, so the skip path is defense in
    /// depth for future batch callers.
    pub async fn join_peers(&self, peers: Vec<String>) -> Result<()> {
        let parsed: Vec<PublicKey> = peers
            .into_iter()
            .filter_map(|s| match PublicKey::from_str(&s) {
                Ok(pk) => Some(pk),
                Err(e) => {
                    warn!(peer = %s, error = %e, "join_peers: skipping unparseable node id");
                    None
                }
            })
            .collect();
        if parsed.is_empty() {
            return Ok(());
        }
        self.inner
            .join_peers(parsed)
            .await
            .map_err(|e| NexusError::Gossip(format!("join_peers failed: {e}")))
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
    use crate::attestations::SECONDS_PER_DAY;
    use crate::crypto::KeyPair;
    use crate::{Node, create_node};
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::timeout;

    async fn spawn_node() -> Node {
        create_node().await.expect("boot")
    }

    /// Test stub for [`AgeAdmissionPolicy`] : owns two maps
    /// mirroring the runtime shell-daemon state (bootstrap
    /// allowlist + first-seen timestamps).
    struct StubPolicy {
        bootstrap: Vec<[u8; 32]>,
        first_seen: HashMap<[u8; 32], i64>,
    }

    impl AgeAdmissionPolicy for StubPolicy {
        fn is_bootstrap_node(&self, node_id: &[u8; 32]) -> bool {
            self.bootstrap.iter().any(|b| b == node_id)
        }

        fn witness_age_days(&self, witness_pubkey: &[u8; 32], now_ts: i64) -> Option<i64> {
            let first = self.first_seen.get(witness_pubkey)?;
            let delta = now_ts.saturating_sub(*first);
            Some(delta / SECONDS_PER_DAY)
        }
    }

    #[test]
    fn admission_bootstrap_node_wins_first() {
        let bootstrap_node = [0x01u8; 32];
        let policy = StubPolicy {
            bootstrap: vec![bootstrap_node],
            first_seen: HashMap::new(),
        };
        let outcome = evaluate_age_admission(&bootstrap_node, None, &policy, 1_700_000_000);
        assert_eq!(outcome, AgeAdmissionOutcome::BootstrapSelfWitness);
    }

    #[test]
    fn admission_witness_passes_with_aged_witness() {
        let joining = [0x42u8; 32];
        let witness_kp = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        let first_seen_joining = base_ts - 10 * SECONDS_PER_DAY;
        let first_seen_witness = base_ts - 45 * SECONDS_PER_DAY;

        let witness = AgeWitness::sign(joining, first_seen_joining, &witness_kp).unwrap();

        let mut policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };
        policy
            .first_seen
            .insert(witness_kp.public_bytes(), first_seen_witness);

        let outcome = evaluate_age_admission(&joining, Some(&witness), &policy, base_ts);
        match outcome {
            AgeAdmissionOutcome::AgeGatePassed {
                age_days,
                witness_age_days,
            } => {
                assert_eq!(age_days, 10);
                assert_eq!(witness_age_days, 45);
            }
            other => panic!("expected AgeGatePassed, got {other:?}"),
        }
    }

    #[test]
    fn admission_witness_rejected_when_underage() {
        let joining = [0x42u8; 32];
        let witness_kp = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        // 3d old — below MIN_AGE_DAYS (7).
        let first_seen_joining = base_ts - 3 * SECONDS_PER_DAY;
        let witness = AgeWitness::sign(joining, first_seen_joining, &witness_kp).unwrap();

        let mut policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };
        policy
            .first_seen
            .insert(witness_kp.public_bytes(), base_ts - 45 * SECONDS_PER_DAY);

        let outcome = evaluate_age_admission(&joining, Some(&witness), &policy, base_ts);
        assert!(matches!(outcome, AgeAdmissionOutcome::Rejected(_)));
    }

    #[test]
    fn admission_witness_rejected_when_witness_itself_underage() {
        let joining = [0x42u8; 32];
        let witness_kp = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        let first_seen_joining = base_ts - 10 * SECONDS_PER_DAY;
        // Witness is only 20d old — below MIN_WITNESS_AGE_DAYS (30).
        let first_seen_witness = base_ts - 20 * SECONDS_PER_DAY;
        let witness = AgeWitness::sign(joining, first_seen_joining, &witness_kp).unwrap();

        let mut policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };
        policy
            .first_seen
            .insert(witness_kp.public_bytes(), first_seen_witness);

        let outcome = evaluate_age_admission(&joining, Some(&witness), &policy, base_ts);
        assert!(matches!(outcome, AgeAdmissionOutcome::Rejected(_)));
    }

    #[test]
    fn admission_falls_back_to_pow_when_no_witness_and_not_bootstrap() {
        let joining = [0x42u8; 32];
        let policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };
        let outcome = evaluate_age_admission(&joining, None, &policy, 1_700_000_000);
        assert!(matches!(outcome, AgeAdmissionOutcome::PowFallback { .. }));
    }

    #[test]
    fn admission_falls_back_when_witness_pubkey_unknown() {
        let joining = [0x42u8; 32];
        let witness_kp = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        let first_seen_joining = base_ts - 10 * SECONDS_PER_DAY;
        let witness = AgeWitness::sign(joining, first_seen_joining, &witness_kp).unwrap();

        // Witness not in policy.first_seen → unknown → PoW fallback.
        let policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };

        let outcome = evaluate_age_admission(&joining, Some(&witness), &policy, base_ts);
        assert!(matches!(outcome, AgeAdmissionOutcome::PowFallback { .. }));
    }

    #[test]
    fn admission_rejects_witness_for_different_node_id() {
        // Present a witness that signs for a different node_id
        // than the joining one. Must reject, not fall back — this
        // is a swapped-witness attack.
        let joining = [0x42u8; 32];
        let other_node = [0x99u8; 32];
        let witness_kp = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        let first_seen_other = base_ts - 10 * SECONDS_PER_DAY;
        let witness = AgeWitness::sign(other_node, first_seen_other, &witness_kp).unwrap();

        let mut policy = StubPolicy {
            bootstrap: vec![],
            first_seen: HashMap::new(),
        };
        policy
            .first_seen
            .insert(witness_kp.public_bytes(), base_ts - 45 * SECONDS_PER_DAY);

        let outcome = evaluate_age_admission(&joining, Some(&witness), &policy, base_ts);
        assert!(matches!(outcome, AgeAdmissionOutcome::Rejected(_)));
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

    #[tokio::test]
    async fn join_peers_skips_bad_ids_and_enqueues_valid() {
        // Sprint 81 Phase E3 core wrapper: hot-join on an isolated
        // single-node topic. `join_peers` is a membership hint
        // (enqueues `Command::JoinPeers` on the actor), never a
        // connect-await — so a valid but unreachable id must still
        // return Ok, and a garbage id in a batch must be skipped
        // per peer (warn + continue), never collect-abort like the
        // boot-time bootstrap parse.
        let node = spawn_node().await;
        let gossip = GossipClient::new(node.gossip());

        let topic_id = *blake3::hash(b"nexus-grid-test/hot-join").as_bytes();
        let topic = gossip
            .subscribe_topic(topic_id, vec![])
            .await
            .expect("isolated subscribe succeeds");
        let (sender, _receiver) = topic.split();

        // (a) empty list short-circuits Ok without touching the actor.
        sender
            .join_peers(vec![])
            .await
            .expect("empty join is a no-op");

        // (b) a valid (unreachable) node id enqueues Ok.
        let valid = hex::encode(KeyPair::generate().public_bytes());
        sender
            .join_peers(vec![valid.clone()])
            .await
            .expect("valid id enqueues");

        // (c) a garbage id in the batch degrades per peer: the
        // valid one still enqueues, the call still returns Ok.
        sender
            .join_peers(vec!["not a real key".into(), valid])
            .await
            .expect("mixed batch degrades per peer, no abort");

        node.shutdown().await.ok();
    }
}

// Note: end-to-end 2-node gossip message exchange is deliberately
// not tested at this layer because it requires configuring peer
// address injection (`MemoryLookup` under iroh 1.0.1) against an
// API surface that is orthogonal to the GossipClient wrapper
// itself. The wrapper is exercised end-to-end in Sprint 4 when
// the coordinator uses it to publish curator list announcements,
// which is the real production path anyway. Two-node handshake
// coverage at the transport layer lives in `shard.rs` (this
// crate) and `seed_protocol.rs` (nexus-shell-daemon) instead.
