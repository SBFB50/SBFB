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

#![deny(unsafe_code)]
#![cfg_attr(test, allow(unsafe_code))]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod attestations;
pub mod blobs;
pub mod canonical;
pub mod compute_group;
pub mod crypto;
pub mod curator;
pub mod dht_quorum;
pub mod discovery;
pub mod dns_fallback;
pub mod doc_sync;
pub mod docs;
pub mod error;
pub mod gossip;
pub mod hooks;
pub mod key_rotation;
pub mod keystore;
pub mod node;
pub mod node_directory;
pub mod pkarr_resolver;
pub mod pow;
pub mod pow_gossip;
pub mod relay_config;
pub mod relay_pow_policy;
pub mod schemas;
pub mod seed;
pub mod shard;
pub mod shard_plan;
pub mod signed_list;
pub mod task;
pub mod tls_pinning;
pub mod tor_transport;
pub mod verification;

pub use attestations::{
    AgeWitness, AgeWitnessError, CONTRIBUTOR_ATTESTATION_PREDICATE_TYPE,
    CONTRIBUTOR_ATTESTATION_STATEMENT_TYPE, ContributorAttestation, ContributorAttestationError,
    ContributorPredicate, DELEGATION_ALGO_OPENPGP_ED25519, DELEGATION_ALGO_SSH_ED25519,
    DELEGATION_ALGO_SSH_RSA, DelegationCert, DelegationCertError, DelegationScope,
    ForgeContribution, MIN_AGE_DAYS, MIN_WITNESS_AGE_DAYS, SigType,
};
pub use blobs::{BlobsClient, Store};
pub use canonical::{
    DOMAIN_AGE_WITNESS_V1, DOMAIN_CLAIM_V1, DOMAIN_COMPUTE_GROUP_V1,
    DOMAIN_CONTRIBUTOR_ATTESTATION_V1, DOMAIN_CURATOR_LIST_V1, DOMAIN_DELEGATION_CERT_V1,
    DOMAIN_DURESS_ACK_V1, DOMAIN_FEED_V1, DOMAIN_INVITE_V1, DOMAIN_KEY_ROTATION_V1,
    DOMAIN_KUDOS_V1, DOMAIN_NODE_DIRECTORY_V1, DOMAIN_POW_V1, DOMAIN_PROVENANCE_V1,
    DOMAIN_RESULT_V1, DOMAIN_RUN_PROOF_V1, DOMAIN_SEED_REQUEST_V1, DOMAIN_SEED_RESPONSE_V1,
    DOMAIN_SHARD_PLAN_V1, DOMAIN_TASK_V1, DOMAIN_WARRANT_CANARY_V1, canonical_bytes,
};
pub use compute_group::{
    COMPUTE_GROUP_FORMAT_VERSION, COMPUTE_GROUP_ID_MAX, COMPUTE_GROUP_MAX_MEMBERS, ComputeGroup,
    ComputeGroupEntry,
};
pub use crypto::{Blake3Chain, KeyPair, blake3_hash, verify};
pub use curator::{
    CURATOR_CATEGORY_MAX, CURATOR_DESCRIPTION_MAX, CURATOR_LIST_FORMAT_VERSION,
    CURATOR_LIST_MAX_ENTRIES, CURATOR_PROJECT_ID_MAX, CURATOR_PROJECT_NAME_MAX,
    ContributorRegistry, CuratorList, CuratorListEntry, CuratorProjectRef,
};
pub use dht_quorum::{QuorumError, QuorumRecord, QuorumResolver, redundant_resolve};
pub use discovery::{DiscoveryClient, NodeAddrInfo};
pub use dns_fallback::{
    DEFAULT_DNS_TIMEOUT, DEFAULT_DOMAIN_SUFFIX, DNS_FALLBACK_DOMAIN_ENV, DNS_FALLBACK_ENABLED_ENV,
    DOH_CLOUDFLARE_IP, DOH_CLOUDFLARE_TLS_NAME, DOH_GOOGLE_IP, DOH_GOOGLE_TLS_NAME, DOH_PORT,
    DOT_PORT, DnsEndpoint, DnsFallbackConfig, DnsFallbackResolve, DnsFallbackResolver,
    concat_txt_strings, load_dns_fallback_from_env,
};
pub use doc_sync::{KeepaliveConfig, spawn_doc_sync_keepalive};
pub use docs::{DocHandle, DocsClient};
pub use error::{NexusError, Result};
pub use gossip::{
    AgeAdmissionOutcome, AgeAdmissionPolicy, DifficultyTarget, GossipClient, GossipEvent,
    TopicHandle, TopicReceiver, TopicSender, evaluate_age_admission,
};
pub use key_rotation::{
    DEFAULT_TRANSITION_DAYS, KEY_ROTATION_FORMAT_VERSION, KEY_ROTATION_TOPIC,
    KeyRotationAnnouncement, MAX_TRANSITION_DAYS, REASON_MAX_BYTES, RevocationCache,
    RevocationEntry, SignedKeyRotation,
};
pub use keystore::{
    ARGON2_MEM_COST_KIB, ARGON2_PARALLELISM, ARGON2_TIME_COST, BLOB_FILE_NAME,
    BLOB_FILE_NAME_DURESS, BLOB_HEADER_LEN, BLOB_MAGIC, BLOB_VERSION, DOMAIN_KEYSTORE_V1, Identity,
    IdentityMode, KEYRING_ACCOUNT_DURESS, KEYRING_ACCOUNT_NORMAL, KEYRING_SERVICE, KdfParams,
    KeyStore, KeyStoreError, LocalFileKeyStore, NONCE_LEN, SALT_LEN, SBFB_IDENTITY_SECRET_HEX_ENV,
    TAG_LEN, UnlockError,
};
pub use node::{
    BlobStore, ExtraProtocolFactory, Node, NodeConfig, SEED_ALPN, SHARD_ALPN, create_node,
    create_node_with_config, create_node_with_protocols,
};
pub use node_directory::{
    CatalogApp, NODE_DIRECTORY_ARCHIVE_HASH_MAX, NODE_DIRECTORY_CATEGORY_MAX,
    NODE_DIRECTORY_DESCRIPTION_MAX, NODE_DIRECTORY_FORMAT_VERSION, NODE_DIRECTORY_MAX_ENTRIES,
    NODE_DIRECTORY_PROJECT_ID_MAX, NODE_DIRECTORY_PROJECT_NAME_MAX, NodeDirectory,
    NodeDirectoryEntry, is_valid_archive_hash,
};
pub use pkarr_resolver::{
    CUSTOM_PKARR_RELAYS_ENV, DEFAULT_PKARR_RELAY_URL, PkarrQuorumResolver,
    load_quorum_resolvers_from_env,
};
// `crypto::verify` is Ed25519 signature verification. `pow::verify` is
// Hashcash PoW verification. Both are useful at the root — we re-export
// the PoW one under a distinct name to keep the crypto signer unchanged
// for the Python side (`nexus_core_py.verify` = signature verify).
pub use pow::{
    DEFAULT_DIFFICULTY_BITS, EscalatingPolicy, HashcashChallenge, HashcashProof,
    MAX_DIFFICULTY_BITS, MAX_PROOF_AGE_SECS, POW_FORMAT_VERSION, PowError, escalating_difficulty,
    leading_zero_bits, should_reset_daily, should_reset_daily_at, solve as pow_solve,
    verify as pow_verify, verify_at as pow_verify_at, verify_stateless as pow_verify_stateless,
};
pub use pow_gossip::{
    PowEnvelope, PowGossipError, PowSolveCache, PowVerifyCache, SESSION_WINDOW, SOLVE_TIMEOUT,
};
pub use relay_config::{
    CUSTOM_RELAYS_ENV, DEV_MODE_ENV, RELAYS_FILE_NAME, RelayEntry, RelayListFile, SBFB_HOME_ENV,
    load_relay_map, relays_file_path, validate_relay_url,
};
pub use relay_pow_policy::{
    CUSTOM_POW_POLICY_ENV, DEFAULT_POLICY as DEFAULT_POW_POLICY, RELAY_POW_POLICY_FILE_NAME,
    RelayPowPolicy, RelayPowPolicyFile, load_relay_pow_policy, load_relay_pow_policy_from,
    relay_pow_policy_file_path,
};
pub use schemas::{
    TASK_RESPONSE_DOMAIN_TAG, TASK_RESPONSE_VERSION, TaskResponse, ToolCall, task_response_schema,
};
pub use seed::{
    SEED_FORMAT_VERSION, SEED_NONCE_LEN, SEED_TS_WINDOW_SECS, SeedDecision, SeedRequest,
    SeedRequestEnvelope, SeedResponse, SeedResponseEnvelope, random_nonce,
};
pub use shard::{
    MAX_SHARD_FRAME_BYTES, SHARD_REJECT_NOT_MEMBER, ShardProtocol, conn_rtt, open_shard_connection,
    read_frame, shard_protocol_factory, write_frame,
};
pub use shard_plan::{
    KvCachePolicy, RUN_PROOF_FORMAT_VERSION, RUN_PROOF_MAX_PARTICIPANTS, RunMetrics, RunProof,
    RunProofEntry, SESSION_ID_MAX, SHARD_GROUP_ID_MAX, SHARD_HASHES_MAX, SHARD_PLAN_FORMAT_VERSION,
    SHARD_PLAN_MAX_ASSIGNMENTS, ShardAssignment, ShardPlan, ShardRole, ShardedSessionManifest,
    ShardedSessionManifestEntry,
};
pub use signed_list::SignedList;
pub use task::{Claim, ClaimEntry, ResultEntry, ResultPayload, RuntimeTuple, Task, TaskEntry};
pub use tls_pinning::{
    CUSTOM_PINS_FILE_ENV, PIN_FILE_FORMAT_VERSION, PinError, PinSource, PinValidator,
    RELAY_PINS_FILE_NAME, RelayPin, RelayPinsFile, extract_spki_sha256,
    extract_spki_sha256_from_pem, relay_pins_file_path,
};
pub use tor_transport::{TorConfig, TorTransport};
pub use verification::{CheckStatus, LayerResult, VerificationReport, Verifier, spot_check_rate};
