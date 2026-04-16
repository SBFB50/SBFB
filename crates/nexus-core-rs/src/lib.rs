// SPDX-License-Identifier: AGPL-3.0-or-later
//! # nexus-core-rs
//!
//! Core Rust library for the SBFB P2P compute network. Wraps the iroh
//! stack (net, docs, gossip, blobs, pkarr) behind a small, stable API
//! that is also exposed to Python via `crates/nexus-core-py` (PyO3).
//!
//! This crate is intentionally thin: it does not implement any
//! business logic (task dispatching, kudos, apps, ...). Those live in
//! the Python coordinator (`packages/nexus-coordinator`) and in the
//! `nexus-worker` binary. The only responsibility here is to give
//! callers a stable handle to a running iroh node and a minimal
//! surface for the primitives the coordinator needs.
//!
//! ## Sprint 1 scope
//!
//! `create_node()` only. Sprint 2 will add `Doc`, `Gossip`, `Blobs`,
//! `Discovery` and `Verifier` wrappers as separate submodules.
//!
//! ## Example (Rust)
//!
//! ```no_run
//! # async fn example() -> nexus_core_rs::Result<()> {
//! let node = nexus_core_rs::create_node().await?;
//! tracing::info!("node id: {}", node.node_id());
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod blobs;
pub mod canonical;
pub mod crypto;
pub mod curator;
pub mod dht_quorum;
pub mod discovery;
pub mod docs;
pub mod error;
pub mod gossip;
pub mod node;
pub mod pkarr_resolver;
pub mod relay_config;
pub mod task;
pub mod verification;

pub use blobs::BlobsClient;
pub use canonical::{
    canonical_bytes, DOMAIN_CLAIM_V1, DOMAIN_CURATOR_LIST_V1, DOMAIN_INVITE_V1, DOMAIN_KUDOS_V1,
    DOMAIN_RESULT_V1, DOMAIN_TASK_V1,
};
pub use crypto::{blake3_hash, verify, Blake3Chain, KeyPair};
pub use curator::{
    CuratorList, CuratorListEntry, CuratorProjectRef, CURATOR_CATEGORY_MAX,
    CURATOR_DESCRIPTION_MAX, CURATOR_LIST_FORMAT_VERSION, CURATOR_LIST_MAX_ENTRIES,
    CURATOR_PROJECT_ID_MAX, CURATOR_PROJECT_NAME_MAX,
};
pub use dht_quorum::{redundant_resolve, QuorumError, QuorumRecord, QuorumResolver};
pub use discovery::{DiscoveryClient, NodeAddrInfo};
pub use docs::{DocHandle, DocsClient};
pub use error::{NexusError, Result};
pub use gossip::{GossipClient, GossipEvent, TopicHandle, TopicReceiver, TopicSender};
pub use node::{create_node, create_node_with_config, Node, NodeConfig};
pub use pkarr_resolver::{
    load_quorum_resolvers_from_env, PkarrQuorumResolver, CUSTOM_PKARR_RELAYS_ENV,
    DEFAULT_PKARR_RELAY_URL,
};
pub use relay_config::{
    load_relay_map, relays_file_path, validate_relay_url, RelayEntry, RelayListFile,
    CUSTOM_RELAYS_ENV, DEV_MODE_ENV, RELAYS_FILE_NAME, SBFB_HOME_ENV,
};
pub use task::{Claim, ClaimEntry, ResultEntry, ResultPayload, Task, TaskEntry};
pub use verification::{spot_check_rate, CheckStatus, LayerResult, VerificationReport, Verifier};
