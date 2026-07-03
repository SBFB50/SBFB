// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sharded-inference wire primitives (Sprint 77 Phase C).
//!
//! These are the signed data structures that describe one pipeline-parallel
//! run of a model too large for any single worker's VRAM, split across the
//! members of a private [`crate::compute_group::ComputeGroup`]:
//!
//! - [`ShardAssignment`] — one worker's contiguous block of model layers
//!   `[layer_start, layer_end)`, the BLAKE3 hash-pin of its shard weights,
//!   and its KV-cache / fallback policy. Unsigned: it only has meaning
//!   inside a signed [`ShardedSessionManifest`].
//! - [`ShardPlan`] — the ordered list of [`ShardAssignment`]s. The Vec
//!   order *is* the pipeline order; [`ShardPlan::is_pipeline_contiguous`]
//!   checks it covers the layer range with no gap or overlap.
//! - [`ShardedSessionManifest`] / [`ShardedSessionManifestEntry`] — the
//!   plan the session **initiator** signs ("here is the run I AUTHORISE"),
//!   under [`DOMAIN_SHARD_PLAN_V1`].
//! - [`RunProof`] / [`RunProofEntry`] — what a **worker** signs after
//!   executing its block ("here is what I EXECUTED"), under
//!   [`DOMAIN_RUN_PROOF_V1`].
//!
//! ## Wire shape (mirror of [`crate::compute_group`])
//!
//! Every signed primitive is a pair: an *unsigned* payload struct whose
//! every field contributes to the canonical bytes, plus an `*Entry`
//! envelope wrapping the payload with a **redundant** signer identity
//! (attribution split-brain mitigation, the same check
//! [`crate::compute_group::ComputeGroupEntry`] /
//! [`crate::node_directory::NodeDirectoryEntry`] apply) and an Ed25519
//! signature over [`canonical_bytes`] with the family's domain tag. The
//! `signature` and redundant identity are NEVER part of the canonical
//! bytes.
//!
//! ## Signature attestation scope (auto-attestation, NOT proof of work)
//!
//! A valid [`ShardedSessionManifestEntry`] signature proves only that *the
//! initiator authored this plan*; a valid [`RunProofEntry`] signature
//! proves only *which worker* produced it (non-repudiation). Neither
//! attests that the computation is **correct**. The
//! [`RunProof::activation_fingerprint`] carries the N0 TOPLOC commitment
//! (Sprint 77 Phase G, [`crate::toploc`]); its independent tolerant recompute
//! is N1/N2 (Phase H/I). Until a verifier recomputes it, a consumer must treat
//! a `RunProof` exactly like [`crate::task::ResultPayload::model_digest`]: a
//! self-claim, never a guarantee.
//!
//! ## No floats in signed payloads
//!
//! [`RunMetrics`] is all-integer on purpose. The canonical on-wire format
//! forbids floats — an `f64` does not round-trip bit-identically across
//! platforms, so a signer (Rust) and a verifier (e.g. a Python client)
//! would derive divergent canonical bytes and the Ed25519 signature would
//! not verify (see [`crate::verification`]). All-integer metrics also let
//! every payload here derive `Eq`.
//!
//! ## DoS mitigation
//!
//! [`SHARD_PLAN_MAX_ASSIGNMENTS`], [`RUN_PROOF_MAX_PARTICIPANTS`],
//! [`SESSION_ID_MAX`], [`SHARD_GROUP_ID_MAX`] and [`SHARD_HASHES_MAX`]
//! bound every network-deserialised collection / string — enforced at BOTH
//! sign and verify so a node can never produce a payload its own peers
//! would reject.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_RUN_PROOF_V1, DOMAIN_SHARD_PLAN_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Current on-wire version for [`ShardedSessionManifest`] payloads.
///
/// Independent from every existing `*_FORMAT_VERSION`: a brand-new signed
/// type, so introducing it bumps nothing (pre-launch additive policy, the
/// S74 `DOMAIN_SEED_REQUEST_V1` pattern). Verifiers refuse a payload whose
/// `version` they do not understand.
pub const SHARD_PLAN_FORMAT_VERSION: u16 = 1;

/// Current on-wire version for [`RunProof`] payloads. See
/// [`SHARD_PLAN_FORMAT_VERSION`].
pub const RUN_PROOF_FORMAT_VERSION: u16 = 1;

/// Hard upper bound on the number of [`ShardAssignment`]s a single signed
/// [`ShardPlan`] may carry (mirrors
/// [`crate::compute_group::COMPUTE_GROUP_MAX_MEMBERS`]). A realistic
/// sharded-inference fan-out is 3-5 machines (addendum §1); 256 is
/// generous and well below a RAM / verification pain threshold.
pub const SHARD_PLAN_MAX_ASSIGNMENTS: usize = 256;

/// Hard upper bound on the number of pipeline participants a single
/// [`RunProof`] may list. Mirrors [`SHARD_PLAN_MAX_ASSIGNMENTS`].
pub const RUN_PROOF_MAX_PARTICIPANTS: usize = 256;

/// Per-field byte cap on a `session_id`. A session id is a short stable
/// handle, not a free-text blob (mirrors
/// [`crate::compute_group::COMPUTE_GROUP_ID_MAX`]).
pub const SESSION_ID_MAX: usize = 128;

/// Per-field byte cap on [`ShardedSessionManifest::group_id`]. Mirrors
/// [`crate::compute_group::COMPUTE_GROUP_ID_MAX`] — the manifest's
/// `group_id` correlates the running pipeline to the
/// [`crate::compute_group::ComputeGroup`] that admits its workers.
pub const SHARD_GROUP_ID_MAX: usize = 128;

/// Hard upper bound on the number of BLAKE3 shard-weight hashes a single
/// [`ShardAssignment`] may pin. A layer block is one weight artifact in
/// practice; the cap bounds a pathological assignment.
pub const SHARD_HASHES_MAX: usize = 64;

/// The role a worker plays in the pipeline.
///
/// An enumerated domain (mirrors the named-constant discipline): a worker
/// runs a block of layers. Kept as an enum rather than a free string so an
/// unknown role is a deserialization error at the signed boundary, not a
/// silently-tolerated value.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ShardRole {
    /// Runs a contiguous block of transformer layers.
    LayerWorker,
}

/// How a worker manages the KV cache for its layer block.
///
/// Frozen to `LocalEphemeral` for S77 (addendum §1): each worker keeps its
/// KV cache locally and ephemerally; distributed KV cache is post-S77
/// (scope cut #5). An enum so the domain stays closed at the signed
/// boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum KvCachePolicy {
    /// KV cache kept local to the worker, discarded at session end.
    LocalEphemeral,
}

/// One worker's contiguous block of model layers in a pipeline.
///
/// Unsigned on its own — it carries meaning only inside a signed
/// [`ShardedSessionManifest`]. The layer range is **half-open**:
/// `[layer_start, layer_end)`, so a block runs layers `layer_start`
/// through `layer_end - 1`, and two consecutive blocks are contiguous when
/// `next.layer_start == prev.layer_end`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ShardAssignment {
    /// Ed25519 public key of the worker that runs this block. MUST be a
    /// member of the session's [`crate::compute_group::ComputeGroup`]
    /// allowlist (checked at the `sbfb/shard/1` handshake, Phase B).
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Inclusive lower bound of the contiguous layer block.
    pub layer_start: u32,

    /// Exclusive upper bound of the contiguous layer block. A well-formed
    /// assignment has `layer_start < layer_end` (non-empty block); the
    /// pipeline-level contiguity invariant is checked by
    /// [`ShardPlan::is_pipeline_contiguous`].
    pub layer_end: u32,

    /// What this worker does in the pipeline.
    pub role: ShardRole,

    /// BLAKE3 hash-pin of the shard weight artifact(s) this worker must
    /// load. Pinning the weights by content hash makes a worker that swaps
    /// in different weights detectable. Bounded by [`SHARD_HASHES_MAX`].
    pub shard_hashes: Vec<[u8; 32]>,

    /// How the worker manages its KV cache.
    pub kv_cache_policy: KvCachePolicy,

    /// Optional fallback worker to re-route this block to on churn
    /// (Petals-style active re-balancing, addendum §2). `#[serde(default)]`
    /// runtime tolerance: a client that omits it deserializes to `None`
    /// rather than failing — it is genuinely optional, not identity.
    #[serde(default)]
    pub fallback_node: Option<[u8; PUBLIC_KEY_LENGTH]>,

    /// BLAKE3 hash of the launch profile (runtime config) the worker must
    /// boot with, so every shard of a session agrees on its parameters.
    pub launch_profile_hash: [u8; 32],
}

/// An ordered list of [`ShardAssignment`]s describing a full pipeline.
///
/// The Vec order **is** the pipeline order. JCS preserves array order in
/// the canonical bytes (it only sorts object keys), so a signed plan keeps
/// its order — but consumers should still validate contiguity rather than
/// trust position alone (see [`Self::is_pipeline_contiguous`]).
// FRONTIER: ShardPlan domain=DOMAIN_SHARD_PLAN_V1 version=SHARD_PLAN_FORMAT_VERSION
// Sprint 79 Phase B · doctrine §7 — its generated JSON schema lives in
// `crates/nexus-core-rs/src/schemas/shard.rs`; gated by scripts/check-frontier-contracts.sh.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ShardPlan {
    /// The per-worker layer-block assignments, in pipeline order. Bounded
    /// by [`SHARD_PLAN_MAX_ASSIGNMENTS`].
    pub assignments: Vec<ShardAssignment>,
}

impl ShardPlan {
    /// Construct a plan from an ordered list of assignments.
    pub fn new(assignments: Vec<ShardAssignment>) -> Self {
        ShardPlan { assignments }
    }

    /// Whether the assignments, **in their current Vec order**, form a
    /// gap-free, non-overlapping pipeline: each block is non-empty
    /// (`layer_start < layer_end`) and every block (after the first) starts
    /// exactly where the previous one ended.
    ///
    /// This does NOT require the first block to start at layer 0 — a plan
    /// may legitimately describe a sub-range. A caller validating a full
    /// model additionally checks `assignments.first().layer_start == 0` and
    /// `assignments.last().layer_end == total_layers`. The stateful
    /// "covers exactly `[0..L)`" check lives in the scheduler (Phase D);
    /// this is the structural invariant the wire primitive can offer.
    pub fn is_pipeline_contiguous(&self) -> bool {
        let mut prev_end: Option<u32> = None;
        for a in &self.assignments {
            if a.layer_start >= a.layer_end {
                return false; // empty / inverted block
            }
            if let Some(end) = prev_end
                && a.layer_start != end
            {
                return false; // gap or overlap
            }
            prev_end = Some(a.layer_end);
        }
        true
    }
}

/// The unsigned sharded-session manifest payload.
///
/// Every field here contributes to the canonical bytes the initiator
/// signs; nothing outside this struct (the envelope's redundant
/// `initiator` / `signature`) is covered by the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct ShardedSessionManifest {
    /// Must equal [`SHARD_PLAN_FORMAT_VERSION`]. `#[serde(default)]` is
    /// intentionally NOT applied: a missing version is a malformed
    /// manifest, not a runtime-tolerant omission.
    pub version: u16,

    /// Ed25519 public key of the session initiator (the
    /// [`crate::compute_group::ComputeGroup`] owner). Cross-checked against
    /// the envelope's redundant `initiator`.
    pub initiator: [u8; PUBLIC_KEY_LENGTH],

    /// Stable handle for this session. Bounded by [`SESSION_ID_MAX`].
    pub session_id: String,

    /// The `group_id` of the [`crate::compute_group::ComputeGroup`] whose
    /// allowlist admits this session's workers. A consumer correlates the
    /// two; the stateful match is enforced where both are in hand
    /// (scheduler / ingest, Phase D/J), not in this crypto module. Bounded
    /// by [`SHARD_GROUP_ID_MAX`].
    pub group_id: String,

    /// Monotonic revision counter (the initiator bumps it when re-planning
    /// the same session). Rollback protection is a stateful ingest-layer
    /// concern, not enforced here — mirrors
    /// [`crate::compute_group::ComputeGroup::revision`].
    pub revision: u64,

    /// The ordered plan of per-worker layer blocks.
    pub plan: ShardPlan,

    /// BLAKE3 digest of the model every shard must run.
    pub model_digest: [u8; 32],

    /// BLAKE3 digest of the tokenizer, so every shard agrees on it.
    pub tokenizer_hash: [u8; 32],

    /// BLAKE3 digest of the chat template, so every shard agrees on it.
    pub chat_template_hash: [u8; 32],
}

impl ShardedSessionManifest {
    /// Construct a manifest at the current format version.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        initiator: [u8; PUBLIC_KEY_LENGTH],
        session_id: impl Into<String>,
        group_id: impl Into<String>,
        revision: u64,
        plan: ShardPlan,
        model_digest: [u8; 32],
        tokenizer_hash: [u8; 32],
        chat_template_hash: [u8; 32],
    ) -> Self {
        ShardedSessionManifest {
            version: SHARD_PLAN_FORMAT_VERSION,
            initiator,
            session_id: session_id.into(),
            group_id: group_id.into(),
            revision,
            plan,
            model_digest,
            tokenizer_hash,
            chat_template_hash,
        }
    }
}

/// A signed [`ShardedSessionManifest`].
///
/// The signature is computed over [`canonical_bytes`] of the inner
/// manifest with [`DOMAIN_SHARD_PLAN_V1`] as the domain tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ShardedSessionManifestEntry {
    /// The manifest itself.
    pub manifest: ShardedSessionManifest,

    /// Redundant Ed25519 pubkey of the signing initiator. MUST equal
    /// [`ShardedSessionManifest::initiator`]; the verifier rejects any
    /// entry where the two disagree (attribution split-brain mitigation).
    pub initiator: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of [`Self::manifest`]
    /// (64 bytes; `serde_big_array` because serde does not derive for
    /// arrays > 32).
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl ShardedSessionManifestEntry {
    /// Sign a [`ShardedSessionManifest`] with the initiator keypair.
    ///
    /// Validates, before signing (mirror of
    /// [`crate::compute_group::ComputeGroupEntry::sign`]):
    /// 1. `manifest.initiator == keypair.public_bytes()`.
    /// 2. DoS caps ([`check_manifest_caps`]) — so a node cannot produce a
    ///    manifest its own peers reject.
    pub fn sign(manifest: ShardedSessionManifest, keypair: &KeyPair) -> Result<Self> {
        if manifest.initiator != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "shard manifest: initiator does not match signing keypair".into(),
            ));
        }
        check_manifest_caps(&manifest)?;
        let bytes = canonical_bytes(&manifest, DOMAIN_SHARD_PLAN_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(ShardedSessionManifestEntry {
            initiator: keypair.public_bytes(),
            manifest,
            signature,
        })
    }

    /// Verify a [`ShardedSessionManifestEntry`].
    ///
    /// Checks, in order (mirror of
    /// [`crate::compute_group::ComputeGroupEntry::verify_signature`]):
    /// 1. `manifest.version == SHARD_PLAN_FORMAT_VERSION`.
    /// 2. DoS caps (before hashing).
    /// 3. `manifest.initiator == self.initiator` (attribution).
    /// 4. Ed25519 signature valid over the canonical bytes with
    ///    [`DOMAIN_SHARD_PLAN_V1`].
    pub fn verify_signature(&self) -> Result<()> {
        if self.manifest.version != SHARD_PLAN_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "shard manifest version mismatch (got {}, expected {})",
                self.manifest.version, SHARD_PLAN_FORMAT_VERSION
            )));
        }
        check_manifest_caps(&self.manifest)?;
        if self.manifest.initiator != self.initiator {
            return Err(NexusError::Crypto(
                "shard manifest: payload initiator does not match envelope initiator".into(),
            ));
        }
        let bytes = canonical_bytes(&self.manifest, DOMAIN_SHARD_PLAN_V1)?;
        crate::crypto::verify(&self.initiator, &bytes, &self.signature)
    }
}

/// All-integer execution metrics carried by a [`RunProof`].
///
/// No floats: a signed JCS payload must round-trip bit-identically across
/// platforms (see the module-level note). Rates are expressed in
/// integer-friendly units (milli-tokens/sec, bytes, milliseconds).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default, JsonSchema)]
pub struct RunMetrics {
    /// Time to first token, milliseconds.
    pub ttft_ms: u64,

    /// Decode throughput in **milli-tokens per second** (tokens/sec ×
    /// 1000), so a fractional rate stays an integer (e.g. 2_300 = 2.3
    /// tok/s). Avoids a float ratio in the signed bytes.
    pub decode_milli_tokens_per_sec: u64,

    /// 95th-percentile per-token latency, milliseconds.
    pub p95_token_latency_ms: u64,

    /// Bytes received over the shard data plane during the run.
    pub network_rx_bytes: u64,

    /// Bytes sent over the shard data plane during the run.
    pub network_tx_bytes: u64,

    /// Number of mid-run worker drops the pipeline recovered from (churn).
    pub worker_drop_count: u32,
}

/// The unsigned run-proof payload — a worker's self-attestation of one
/// executed run.
///
/// **Attestation scope:** a valid signature proves only *which worker*
/// signed this (non-repudiation). It does NOT attest that the computation
/// was correct. See the module-level note.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct RunProof {
    /// Must equal [`RUN_PROOF_FORMAT_VERSION`]; `#[serde(default)]` NOT
    /// applied (a missing version is malformed).
    pub version: u16,

    /// Ed25519 public key of the worker that executed and signed this.
    /// Cross-checked against the envelope's redundant `worker_pubkey`.
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// The session this proof belongs to — binds the proof to one
    /// [`ShardedSessionManifest::session_id`] so an honest proof from a
    /// past session cannot be replayed into a current one. Bounded by
    /// [`SESSION_ID_MAX`].
    pub session_id: String,

    /// BLAKE3 digest of the model the worker ran.
    pub model_digest: [u8; 32],

    /// BLAKE3 digest binding the prompt/precision profile. TOPLOC requires
    /// the verifier to supply the same model + prompt + precision to
    /// recompute the fingerprint; this is that external binding.
    pub prompt_profile_hash: [u8; 32],

    /// **N0 TOPLOC commitment** (Sprint 77 Phase G). The 32-byte BLAKE3
    /// commitment of the worker's canonical
    /// [`crate::toploc::ToplocFingerprint`] over the top-k of its block's last
    /// hidden state (post-norm, last shard). 32 zeros means "not provided"
    /// (e.g. a non-`llm_llama_cpp` backend, where hidden states are unavailable
    /// and N0 is infeasible).
    ///
    /// **Binding only, not a tolerant proof:** a BLAKE3 commitment is compared
    /// by equality; the tolerant exponent/mantissa recompute lives in
    /// [`crate::toploc::ToplocFingerprint::compare`] and runs cross-worker in
    /// the N1 spot-check (Phase H) / N2 redundancy (Phase I), once the full
    /// sketch is transported off this 32-byte slot. Until an independent
    /// verifier recomputes it this is a self-claim — see the module note on
    /// auto-attestation. `#[serde(default)]` runtime tolerance: an omitted slot
    /// deserializes to zeros.
    #[serde(default)]
    pub activation_fingerprint: [u8; 32],

    /// All-integer execution metrics.
    pub metrics: RunMetrics,

    /// The other pipeline participants this run involved (their Ed25519
    /// pubkeys). Bounded by [`RUN_PROOF_MAX_PARTICIPANTS`].
    pub participants: Vec<[u8; PUBLIC_KEY_LENGTH]>,
}

impl RunProof {
    /// Construct a run-proof at the current format version. The N0 TOPLOC
    /// fingerprint slot defaults to zero ("not provided"); a worker overwrites
    /// it with its commitment ([`crate::toploc`]) before signing.
    pub fn new(
        worker_pubkey: [u8; PUBLIC_KEY_LENGTH],
        session_id: impl Into<String>,
        model_digest: [u8; 32],
        prompt_profile_hash: [u8; 32],
        metrics: RunMetrics,
        participants: Vec<[u8; PUBLIC_KEY_LENGTH]>,
    ) -> Self {
        RunProof {
            version: RUN_PROOF_FORMAT_VERSION,
            worker_pubkey,
            session_id: session_id.into(),
            model_digest,
            prompt_profile_hash,
            activation_fingerprint: [0u8; 32],
            metrics,
            participants,
        }
    }
}

/// A signed [`RunProof`].
///
/// The signature is computed over [`canonical_bytes`] of the inner proof
/// with [`DOMAIN_RUN_PROOF_V1`] as the domain tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunProofEntry {
    /// The proof itself.
    pub proof: RunProof,

    /// Redundant Ed25519 pubkey of the signing worker. MUST equal
    /// [`RunProof::worker_pubkey`]; the verifier rejects a mismatch
    /// (attribution split-brain mitigation).
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of [`Self::proof`].
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl RunProofEntry {
    /// Sign a [`RunProof`] with the worker keypair (mirror of
    /// [`ShardedSessionManifestEntry::sign`]).
    pub fn sign(proof: RunProof, keypair: &KeyPair) -> Result<Self> {
        if proof.worker_pubkey != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "run proof: worker_pubkey does not match signing keypair".into(),
            ));
        }
        check_run_proof_caps(&proof)?;
        let bytes = canonical_bytes(&proof, DOMAIN_RUN_PROOF_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(RunProofEntry {
            worker_pubkey: keypair.public_bytes(),
            proof,
            signature,
        })
    }

    /// Verify a [`RunProofEntry`] (mirror of
    /// [`ShardedSessionManifestEntry::verify_signature`]).
    pub fn verify_signature(&self) -> Result<()> {
        if self.proof.version != RUN_PROOF_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "run proof version mismatch (got {}, expected {})",
                self.proof.version, RUN_PROOF_FORMAT_VERSION
            )));
        }
        check_run_proof_caps(&self.proof)?;
        if self.proof.worker_pubkey != self.worker_pubkey {
            return Err(NexusError::Crypto(
                "run proof: payload worker_pubkey does not match envelope worker_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.proof, DOMAIN_RUN_PROOF_V1)?;
        crate::crypto::verify(&self.worker_pubkey, &bytes, &self.signature)
    }
}

/// Reject a manifest whose collections / strings exceed their DoS caps.
/// Enforced at sign AND verify (mirror of
/// [`crate::compute_group`]'s `check_group_caps`).
fn check_manifest_caps(manifest: &ShardedSessionManifest) -> Result<()> {
    if manifest.plan.assignments.len() > SHARD_PLAN_MAX_ASSIGNMENTS {
        return Err(NexusError::Crypto(format!(
            "shard plan has {} assignments, exceeds SHARD_PLAN_MAX_ASSIGNMENTS={}",
            manifest.plan.assignments.len(),
            SHARD_PLAN_MAX_ASSIGNMENTS
        )));
    }
    if manifest.session_id.len() > SESSION_ID_MAX {
        return Err(NexusError::Crypto(format!(
            "shard manifest session_id has {} bytes, exceeds SESSION_ID_MAX={}",
            manifest.session_id.len(),
            SESSION_ID_MAX
        )));
    }
    if manifest.group_id.len() > SHARD_GROUP_ID_MAX {
        return Err(NexusError::Crypto(format!(
            "shard manifest group_id has {} bytes, exceeds SHARD_GROUP_ID_MAX={}",
            manifest.group_id.len(),
            SHARD_GROUP_ID_MAX
        )));
    }
    for a in &manifest.plan.assignments {
        if a.shard_hashes.len() > SHARD_HASHES_MAX {
            return Err(NexusError::Crypto(format!(
                "shard assignment has {} shard_hashes, exceeds SHARD_HASHES_MAX={}",
                a.shard_hashes.len(),
                SHARD_HASHES_MAX
            )));
        }
    }
    Ok(())
}

/// Reject a run-proof whose collections / strings exceed their DoS caps.
/// Enforced at sign AND verify.
fn check_run_proof_caps(proof: &RunProof) -> Result<()> {
    if proof.participants.len() > RUN_PROOF_MAX_PARTICIPANTS {
        return Err(NexusError::Crypto(format!(
            "run proof has {} participants, exceeds RUN_PROOF_MAX_PARTICIPANTS={}",
            proof.participants.len(),
            RUN_PROOF_MAX_PARTICIPANTS
        )));
    }
    if proof.session_id.len() > SESSION_ID_MAX {
        return Err(NexusError::Crypto(format!(
            "run proof session_id has {} bytes, exceeds SESSION_ID_MAX={}",
            proof.session_id.len(),
            SESSION_ID_MAX
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_assignment(worker: &KeyPair, start: u32, end: u32) -> ShardAssignment {
        ShardAssignment {
            worker_pubkey: worker.public_bytes(),
            layer_start: start,
            layer_end: end,
            role: ShardRole::LayerWorker,
            shard_hashes: vec![[7u8; 32]],
            kv_cache_policy: KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [9u8; 32],
        }
    }

    fn sample_manifest(initiator: &KeyPair, workers: &[&KeyPair]) -> ShardedSessionManifest {
        let mut assignments = Vec::new();
        for (i, w) in workers.iter().enumerate() {
            let start = (i as u32) * 16;
            assignments.push(sample_assignment(w, start, start + 16));
        }
        ShardedSessionManifest::new(
            initiator.public_bytes(),
            "session-70b-1",
            "pilot-70b",
            1,
            ShardPlan::new(assignments),
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        )
    }

    fn sample_proof(worker: &KeyPair, others: &[&KeyPair]) -> RunProof {
        RunProof::new(
            worker.public_bytes(),
            "session-70b-1",
            [1u8; 32],
            [4u8; 32],
            RunMetrics {
                ttft_ms: 1200,
                decode_milli_tokens_per_sec: 2300,
                p95_token_latency_ms: 450,
                network_rx_bytes: 1_048_576,
                network_tx_bytes: 524_288,
                worker_drop_count: 0,
            },
            others.iter().map(|k| k.public_bytes()).collect(),
        )
    }

    #[test]
    fn shard_plan_signature_roundtrip() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let manifest = sample_manifest(&initiator, &[&w1, &w2]);
        let entry = ShardedSessionManifestEntry::sign(manifest, &initiator).unwrap();
        entry
            .verify_signature()
            .expect("freshly signed manifest must verify");
        assert_eq!(entry.initiator, initiator.public_bytes());
        assert_eq!(entry.manifest.version, SHARD_PLAN_FORMAT_VERSION);
        assert_eq!(entry.manifest.plan.assignments.len(), 2);
    }

    #[test]
    fn shard_assignment_serde_roundtrip() {
        // The unsigned sub-struct round-trips through JSON, and a missing
        // optional `fallback_node` deserializes to None (runtime tolerance).
        let w = KeyPair::generate();
        let a = sample_assignment(&w, 0, 16);
        let json = serde_json::to_vec(&a).unwrap();
        let back: ShardAssignment = serde_json::from_slice(&json).unwrap();
        assert_eq!(back, a);

        let minimal = r#"{
            "worker_pubkey": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
            "layer_start": 0, "layer_end": 16, "role": "layer_worker",
            "shard_hashes": [], "kv_cache_policy": "local_ephemeral",
            "launch_profile_hash": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        }"#;
        let parsed: ShardAssignment = serde_json::from_str(minimal).unwrap();
        assert_eq!(parsed.fallback_node, None, "omitted fallback_node => None");
    }

    #[test]
    fn run_proof_signature_roundtrip() {
        let worker = KeyPair::generate();
        let peer = KeyPair::generate();
        let proof = sample_proof(&worker, &[&peer]);
        let entry = RunProofEntry::sign(proof, &worker).unwrap();
        entry
            .verify_signature()
            .expect("freshly signed run proof must verify");
        assert_eq!(entry.worker_pubkey, worker.public_bytes());
        assert_eq!(
            entry.proof.activation_fingerprint, [0u8; 32],
            "RunProof::new defaults the N0 fingerprint slot to zero (not provided)"
        );
    }

    #[test]
    fn manifest_verify_rejects_tampered_payload() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let mut entry =
            ShardedSessionManifestEntry::sign(sample_manifest(&initiator, &[&w1]), &initiator)
                .unwrap();
        // Bump the revision after signing.
        entry.manifest.revision += 1;
        assert!(
            entry.verify_signature().is_err(),
            "mutating the manifest after signing must fail verification"
        );
    }

    #[test]
    fn manifest_verify_rejects_tampered_signature() {
        let initiator = KeyPair::generate();
        let mut entry =
            ShardedSessionManifestEntry::sign(sample_manifest(&initiator, &[]), &initiator)
                .unwrap();
        entry.signature[0] ^= 0xFF;
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn manifest_verify_rejects_attribution_mismatch() {
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut entry =
            ShardedSessionManifestEntry::sign(sample_manifest(&real, &[]), &real).unwrap();
        entry.initiator = impostor.public_bytes();
        assert!(
            entry.verify_signature().is_err(),
            "payload initiator != envelope initiator must be rejected"
        );
    }

    #[test]
    fn manifest_sign_rejects_wrong_signer() {
        let owner = KeyPair::generate();
        let other = KeyPair::generate();
        assert!(
            ShardedSessionManifestEntry::sign(sample_manifest(&owner, &[]), &other).is_err(),
            "signing someone else's manifest must fail at sign time"
        );
    }

    #[test]
    fn manifest_rejects_oversized_assignments() {
        let initiator = KeyPair::generate();
        let filler = KeyPair::generate();
        let mut manifest = sample_manifest(&initiator, &[]);
        manifest.plan.assignments = (0..=SHARD_PLAN_MAX_ASSIGNMENTS)
            .map(|i| sample_assignment(&filler, i as u32, i as u32 + 1))
            .collect();
        // Sign-side cap.
        assert!(
            ShardedSessionManifestEntry::sign(manifest.clone(), &initiator).is_err(),
            "sign must reject an over-capacity plan"
        );
        // Verify-side cap: forge an envelope and confirm verify rejects it
        // before hashing.
        let entry = ShardedSessionManifestEntry {
            initiator: initiator.public_bytes(),
            manifest,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // The forged signature is zero, but the cap is checked BEFORE the
        // crypto verify — so asserting the error is the *cap* error (not a
        // signature error) proves the cap fires independently of signature
        // validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn manifest_rejects_oversized_group_id() {
        let initiator = KeyPair::generate();
        let mut manifest = sample_manifest(&initiator, &[]);
        manifest.group_id = "x".repeat(SHARD_GROUP_ID_MAX + 1);
        // Sign-side cap.
        assert!(ShardedSessionManifestEntry::sign(manifest.clone(), &initiator).is_err());
        // Verify-side cap: forge an envelope and confirm verify rejects it
        // before hashing.
        let entry = ShardedSessionManifestEntry {
            initiator: initiator.public_bytes(),
            manifest,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // The forged signature is zero, but the cap is checked BEFORE the
        // crypto verify — so asserting the error is the *cap* error (not a
        // signature error) proves the cap fires independently of signature
        // validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn run_proof_rejects_oversized_participants() {
        let worker = KeyPair::generate();
        let mut proof = sample_proof(&worker, &[]);
        proof.participants = vec![[0u8; PUBLIC_KEY_LENGTH]; RUN_PROOF_MAX_PARTICIPANTS + 1];
        // Sign-side cap.
        assert!(RunProofEntry::sign(proof.clone(), &worker).is_err());
        // Verify-side cap.
        let entry = RunProofEntry {
            worker_pubkey: worker.public_bytes(),
            proof,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // Zero signature, but the cap is checked BEFORE the crypto verify —
        // asserting the *cap* error proves the cap fires independently of
        // signature validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn run_proof_verify_rejects_attribution_mismatch() {
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut entry = RunProofEntry::sign(sample_proof(&real, &[]), &real).unwrap();
        entry.worker_pubkey = impostor.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn manifest_and_run_proof_domain_separated() {
        // The two families use distinct domain tags, so even byte-identical
        // JSON bodies produce different canonical pre-images: a manifest
        // signature can never be replayed as a run-proof signature.
        let kp = KeyPair::generate();
        let manifest = sample_manifest(&kp, &[]);
        let as_shard_plan = canonical_bytes(&manifest, DOMAIN_SHARD_PLAN_V1).unwrap();
        let as_run_proof = canonical_bytes(&manifest, DOMAIN_RUN_PROOF_V1).unwrap();
        assert_ne!(
            as_shard_plan, as_run_proof,
            "shard-plan and run-proof domains must produce distinct byte strings"
        );
    }

    #[test]
    fn cross_domain_signature_rejected() {
        // Mint a signature over a value under DOMAIN_RUN_PROOF_V1, then try
        // to verify it as if it were under DOMAIN_SHARD_PLAN_V1. The domain
        // prefix differs, so verification must fail — the core anti-replay
        // property across the two new families.
        let kp = KeyPair::generate();
        let manifest = sample_manifest(&kp, &[]);
        let wrong_domain_bytes = canonical_bytes(&manifest, DOMAIN_RUN_PROOF_V1).unwrap();
        let wrong_sig = kp.sign(&wrong_domain_bytes);
        let entry = ShardedSessionManifestEntry {
            initiator: kp.public_bytes(),
            manifest,
            signature: wrong_sig,
        };
        assert!(
            entry.verify_signature().is_err(),
            "a signature minted under the run-proof domain must not verify as a manifest"
        );
    }

    #[test]
    fn shard_plan_contiguity_detects_gap_and_overlap() {
        let a = KeyPair::generate();
        let b = KeyPair::generate();
        // Contiguous: [0..16) then [16..32).
        let good = ShardPlan::new(vec![
            sample_assignment(&a, 0, 16),
            sample_assignment(&b, 16, 32),
        ]);
        assert!(good.is_pipeline_contiguous());
        // Gap: [0..16) then [20..32).
        let gap = ShardPlan::new(vec![
            sample_assignment(&a, 0, 16),
            sample_assignment(&b, 20, 32),
        ]);
        assert!(!gap.is_pipeline_contiguous());
        // Overlap: [0..20) then [16..32).
        let overlap = ShardPlan::new(vec![
            sample_assignment(&a, 0, 20),
            sample_assignment(&b, 16, 32),
        ]);
        assert!(!overlap.is_pipeline_contiguous());
        // Empty/inverted block: [16..16).
        let empty = ShardPlan::new(vec![sample_assignment(&a, 16, 16)]);
        assert!(!empty.is_pipeline_contiguous());
    }

    #[test]
    fn entries_json_roundtrip_and_reverify() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let manifest_entry =
            ShardedSessionManifestEntry::sign(sample_manifest(&initiator, &[&w1]), &initiator)
                .unwrap();
        let mj = serde_json::to_vec(&manifest_entry).unwrap();
        let mback: ShardedSessionManifestEntry = serde_json::from_slice(&mj).unwrap();
        assert_eq!(mback, manifest_entry);
        mback.verify_signature().unwrap();

        let worker = KeyPair::generate();
        let proof_entry = RunProofEntry::sign(sample_proof(&worker, &[&w1]), &worker).unwrap();
        let pj = serde_json::to_vec(&proof_entry).unwrap();
        let pback: RunProofEntry = serde_json::from_slice(&pj).unwrap();
        assert_eq!(pback, proof_entry);
        pback.verify_signature().unwrap();
    }

    // --- Defensive-branch coverage (Phase C review Dimension 3) ---
    // The error branches below are correct mirrors of the patron but were
    // not exercised by the core round-trip set; each test isolates one
    // branch so a refactor that drops it goes red.

    #[test]
    fn manifest_verify_rejects_wrong_version() {
        // The payload is signed with a VALID signature over an unknown
        // version, so the only thing that can reject it is the version gate
        // (checked first in verify_signature, before the crypto check).
        let initiator = KeyPair::generate();
        let mut manifest = sample_manifest(&initiator, &[]);
        manifest.version = SHARD_PLAN_FORMAT_VERSION + 1;
        let entry = ShardedSessionManifestEntry::sign(manifest, &initiator).unwrap();
        assert!(
            entry.verify_signature().is_err(),
            "an unknown manifest version must be rejected at the version gate"
        );
    }

    #[test]
    fn run_proof_verify_rejects_wrong_version() {
        let worker = KeyPair::generate();
        let mut proof = sample_proof(&worker, &[]);
        proof.version = RUN_PROOF_FORMAT_VERSION + 1;
        let entry = RunProofEntry::sign(proof, &worker).unwrap();
        assert!(
            entry.verify_signature().is_err(),
            "an unknown run-proof version must be rejected at the version gate"
        );
    }

    #[test]
    fn manifest_rejects_oversized_session_id() {
        let initiator = KeyPair::generate();
        let mut manifest = sample_manifest(&initiator, &[]);
        manifest.session_id = "s".repeat(SESSION_ID_MAX + 1);
        // Sign-side cap.
        assert!(ShardedSessionManifestEntry::sign(manifest.clone(), &initiator).is_err());
        // Verify-side cap.
        let entry = ShardedSessionManifestEntry {
            initiator: initiator.public_bytes(),
            manifest,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // The forged signature is zero, but the cap is checked BEFORE the
        // crypto verify — so asserting the error is the *cap* error (not a
        // signature error) proves the cap fires independently of signature
        // validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn run_proof_rejects_oversized_session_id() {
        let worker = KeyPair::generate();
        let mut proof = sample_proof(&worker, &[]);
        proof.session_id = "s".repeat(SESSION_ID_MAX + 1);
        // Sign-side cap.
        assert!(RunProofEntry::sign(proof.clone(), &worker).is_err());
        // Verify-side cap: forge an envelope and confirm verify rejects it
        // before hashing.
        let entry = RunProofEntry {
            worker_pubkey: worker.public_bytes(),
            proof,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // Zero signature, but the cap is checked BEFORE the crypto verify —
        // asserting the *cap* error proves the cap fires independently of
        // signature validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn manifest_rejects_oversized_shard_hashes() {
        // The per-assignment SHARD_HASHES_MAX branch (iterated inside
        // check_manifest_caps) — enforced at sign AND verify.
        let initiator = KeyPair::generate();
        let worker = KeyPair::generate();
        let mut manifest = sample_manifest(&initiator, &[]);
        let mut a = sample_assignment(&worker, 0, 16);
        a.shard_hashes = vec![[0u8; 32]; SHARD_HASHES_MAX + 1];
        manifest.plan = ShardPlan::new(vec![a]);
        // Sign-side cap.
        assert!(ShardedSessionManifestEntry::sign(manifest.clone(), &initiator).is_err());
        // Verify-side cap.
        let entry = ShardedSessionManifestEntry {
            initiator: initiator.public_bytes(),
            manifest,
            signature: [0u8; SIGNATURE_BYTES],
        };
        // The forged signature is zero, but the cap is checked BEFORE the
        // crypto verify — so asserting the error is the *cap* error (not a
        // signature error) proves the cap fires independently of signature
        // validity (Codex Phase C round 2).
        assert!(
            entry
                .verify_signature()
                .unwrap_err()
                .to_string()
                .contains("exceeds"),
            "verify must reject on the DoS cap, before the signature check"
        );
    }

    #[test]
    fn run_proof_sign_rejects_wrong_signer() {
        let owner = KeyPair::generate();
        let other = KeyPair::generate();
        assert!(
            RunProofEntry::sign(sample_proof(&owner, &[]), &other).is_err(),
            "signing someone else's run proof must fail at sign time"
        );
    }

    #[test]
    fn run_proof_verify_rejects_tampered_payload() {
        let worker = KeyPair::generate();
        let mut entry = RunProofEntry::sign(sample_proof(&worker, &[]), &worker).unwrap();
        // Inflate a metric after signing.
        entry.proof.metrics.ttft_ms += 1;
        assert!(
            entry.verify_signature().is_err(),
            "mutating the proof after signing must fail verification"
        );
    }

    #[test]
    fn run_proof_verify_rejects_tampered_signature() {
        let worker = KeyPair::generate();
        let mut entry = RunProofEntry::sign(sample_proof(&worker, &[]), &worker).unwrap();
        entry.signature[0] ^= 0xFF;
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn run_proof_cross_domain_signature_rejected() {
        // Symmetric to cross_domain_signature_rejected, the other direction:
        // a signature minted under DOMAIN_SHARD_PLAN_V1 must not verify as a
        // run proof (which checks under DOMAIN_RUN_PROOF_V1).
        let worker = KeyPair::generate();
        let proof = sample_proof(&worker, &[]);
        let wrong_domain_bytes = canonical_bytes(&proof, DOMAIN_SHARD_PLAN_V1).unwrap();
        let wrong_sig = worker.sign(&wrong_domain_bytes);
        let entry = RunProofEntry {
            worker_pubkey: worker.public_bytes(),
            proof,
            signature: wrong_sig,
        };
        assert!(
            entry.verify_signature().is_err(),
            "a signature minted under the shard-plan domain must not verify as a run proof"
        );
    }
}
