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
pub mod keystore;
pub mod node;
pub mod pkarr_resolver;
pub mod pow;
pub mod pow_gossip;
pub mod relay_config;
pub mod relay_pow_policy;
pub mod schemas;
pub mod task;
pub mod tls_pinning;
pub mod verification;

pub use blobs::BlobsClient;
pub use canonical::{
    canonical_bytes, DOMAIN_CLAIM_V1, DOMAIN_CURATOR_LIST_V1, DOMAIN_DURESS_ACK_V1,
    DOMAIN_INVITE_V1, DOMAIN_KUDOS_V1, DOMAIN_POW_V1, DOMAIN_RESULT_V1, DOMAIN_TASK_V1,
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
pub use keystore::{
    Identity, IdentityMode, KdfParams, KeyStore, KeyStoreError, LocalFileKeyStore, UnlockError,
    ARGON2_MEM_COST_KIB, ARGON2_PARALLELISM, ARGON2_TIME_COST, BLOB_FILE_NAME,
    BLOB_FILE_NAME_DURESS, BLOB_HEADER_LEN, BLOB_MAGIC, BLOB_VERSION, DOMAIN_KEYSTORE_V1,
    KEYRING_ACCOUNT_DURESS, KEYRING_ACCOUNT_NORMAL, KEYRING_SERVICE, NONCE_LEN, SALT_LEN,
    SBFB_IDENTITY_SECRET_HEX_ENV, TAG_LEN,
};
pub use node::{create_node, create_node_with_config, Node, NodeConfig};
pub use pkarr_resolver::{
    load_quorum_resolvers_from_env, PkarrQuorumResolver, CUSTOM_PKARR_RELAYS_ENV,
    DEFAULT_PKARR_RELAY_URL,
};
// `crypto::verify` is Ed25519 signature verification. `pow::verify` is
// Hashcash PoW verification. Both are useful at the root — we re-export
// the PoW one under a distinct name to keep the crypto signer unchanged
// for the Python side (`nexus_core_py.verify` = signature verify).
pub use pow::{
    leading_zero_bits, solve as pow_solve, verify as pow_verify, verify_at as pow_verify_at,
    verify_stateless as pow_verify_stateless, HashcashChallenge, HashcashProof, PowError,
    DEFAULT_DIFFICULTY_BITS, MAX_DIFFICULTY_BITS, MAX_PROOF_AGE_SECS, POW_FORMAT_VERSION,
};
pub use pow_gossip::{
    PowEnvelope, PowGossipError, PowSolveCache, PowVerifyCache, SESSION_WINDOW, SOLVE_TIMEOUT,
};
pub use relay_config::{
    load_relay_map, relays_file_path, validate_relay_url, RelayEntry, RelayListFile,
    CUSTOM_RELAYS_ENV, DEV_MODE_ENV, RELAYS_FILE_NAME, SBFB_HOME_ENV,
};
pub use relay_pow_policy::{
    load_relay_pow_policy, load_relay_pow_policy_from, relay_pow_policy_file_path, RelayPowPolicy,
    RelayPowPolicyFile, CUSTOM_POW_POLICY_ENV, DEFAULT_POLICY as DEFAULT_POW_POLICY,
    RELAY_POW_POLICY_FILE_NAME,
};
pub use schemas::{
    task_response_schema, TaskResponse, ToolCall, TASK_RESPONSE_DOMAIN_TAG, TASK_RESPONSE_VERSION,
};
pub use task::{Claim, ClaimEntry, ResultEntry, ResultPayload, Task, TaskEntry};
pub use tls_pinning::{
    extract_spki_sha256, extract_spki_sha256_from_pem, relay_pins_file_path, PinError, PinSource,
    PinValidator, RelayPin, RelayPinsFile, CUSTOM_PINS_FILE_ENV, PIN_FILE_FORMAT_VERSION,
    RELAY_PINS_FILE_NAME,
};
pub use verification::{spot_check_rate, CheckStatus, LayerResult, VerificationReport, Verifier};
