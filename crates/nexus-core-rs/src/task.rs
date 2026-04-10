//! Task queue domain types for SBFB.
//!
//! These are the canonical data structures that move through the
//! iroh-docs replicas powering each project's task queue. A
//! coordinator writes [`TaskEntry`] values into the tasks doc,
//! workers write [`ClaimEntry`] values to race for ownership, and
//! once they finish inference they write [`ResultEntry`] values
//! back into the same doc.
//!
//! ## Canonical serialization
//!
//! Signatures are computed over a **canonical byte representation**
//! produced by [`crate::canonical::canonical_bytes`]. That function
//! uses RFC 8785 JSON Canonicalization Scheme (via the `serde_jcs`
//! crate) plus a type-specific domain separation prefix, so every
//! peer in every language reproduces the exact same bytes and
//! verification is deterministic across the Rust worker and the
//! Python coordinator.
//!
//! The module doc on [`crate::canonical`] explains the prefix
//! layout and the rationale; nothing in this module — or anywhere
//! in the crate — should ever hand-roll its own signing bytes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{canonical_bytes, DOMAIN_CLAIM_V1, DOMAIN_RESULT_V1, DOMAIN_TASK_V1};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::Result;

/// Current on-wire version for Task and Result payloads.
///
/// Bump this when the canonical serialization format changes in a
/// way that breaks signature compatibility. Consumers should refuse
/// to verify entries with a version they don't understand.
pub const TASK_FORMAT_VERSION: u16 = 1;

/// An LLM inference task as the coordinator creates it.
///
/// A `Task` is the fully-signed unit that gets written into the
/// tasks doc. Workers discover it, claim it, run inference, and
/// reply with a [`ResultEntry`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Task {
    /// Canonical format version. Must equal [`TASK_FORMAT_VERSION`]
    /// to be accepted by this build.
    pub version: u16,

    /// Stable task identifier (typically a UUIDv4 or a hash).
    /// Callers are free to choose the scheme, but it MUST be unique
    /// within a given project and MUST be stable across the task's
    /// entire lifecycle.
    pub task_id: String,

    /// Short category tag (e.g. `"analysis"`, `"summary"`,
    /// `"contradiction_check"`) that the worker uses to route to
    /// the right local prompt template.
    pub task_type: String,

    /// The actual user prompt the worker will run against the LLM.
    pub prompt: String,

    /// Optional system prompt / role instructions, empty string
    /// if none.
    pub system_prompt: String,

    /// Name of the model the coordinator expects to run against
    /// (e.g. `"llama-3.1-8b"`). The worker verifies this matches
    /// its local Ollama setup before accepting the task.
    pub model: String,

    /// 1 = highest priority, 10 = lowest. Workers pull lowest
    /// numeric value first.
    pub priority: u8,

    /// Unix timestamp (seconds since epoch) when the coordinator
    /// created this task. Used for FIFO ordering within a given
    /// priority bucket.
    pub created_at: u64,

    /// Optional parent task id, for causal ordering across tasks
    /// that must run in sequence. Empty string means no parent.
    ///
    /// iroh-docs itself has no causal ordering — it is LWW by
    /// timestamp — so workers are responsible for honoring this
    /// field at execution time.
    pub parent_task_id: String,

    /// Free-form metadata that the coordinator wants to propagate
    /// through to the worker and into the result. The map MUST
    /// contain only string values (no nested objects, no floats)
    /// to keep the canonical serialization stable.
    pub metadata: BTreeMap<String, String>,
}

impl Task {
    /// Create a new Task with the current format version and no
    /// parent. Convenience constructor for the common case; fields
    /// can be mutated directly after construction if needed.
    pub fn new(
        task_id: impl Into<String>,
        task_type: impl Into<String>,
        prompt: impl Into<String>,
        model: impl Into<String>,
        priority: u8,
        created_at: u64,
    ) -> Self {
        Task {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.into(),
            task_type: task_type.into(),
            prompt: prompt.into(),
            system_prompt: String::new(),
            model: model.into(),
            priority,
            created_at,
            parent_task_id: String::new(),
            metadata: BTreeMap::new(),
        }
    }
}

/// A signed Task, ready to be written to the tasks doc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskEntry {
    /// The Task itself.
    pub task: Task,

    /// Ed25519 public key of the coordinator that signed this task
    /// (32 bytes).
    pub author_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of the task
    /// (64 bytes). See [`crate::canonical`] for the exact layout.
    /// serde-big-array is used because serde does not derive
    /// Serialize/Deserialize for arrays > 32.
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl TaskEntry {
    /// Sign a Task with the given keypair and produce a signed
    /// entry ready to be written to the tasks doc.
    pub fn sign(task: Task, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&task, DOMAIN_TASK_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(TaskEntry {
            task,
            author_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the signature on this entry.
    ///
    /// Returns `Ok(())` on valid signature, or
    /// [`crate::error::NexusError::Crypto`] on any failure (bad
    /// bytes, wrong key, tampered task).
    pub fn verify_signature(&self) -> Result<()> {
        let bytes = canonical_bytes(&self.task, DOMAIN_TASK_V1)?;
        crate::crypto::verify(&self.author_pubkey, &bytes, &self.signature)
    }
}

/// A worker's reply to a [`TaskEntry`].
///
/// Written into the results doc once inference completes. The
/// coordinator reads these, runs the 3-layer verification
/// (signature + model digest + logprob), and credits the worker
/// with kudos if everything checks out.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResultPayload {
    /// Canonical format version, equal to [`TASK_FORMAT_VERSION`].
    pub version: u16,

    /// Which Task this is a reply for. Must match
    /// [`Task::task_id`] of an existing TaskEntry in the tasks doc.
    pub task_id: String,

    /// The LLM's text output.
    pub result_text: String,

    /// Number of tokens generated. Used for kudos accounting and
    /// tokens/sec throughput stats.
    pub tokens_generated: u64,

    /// Inference wall-clock time in milliseconds. Used for
    /// throughput stats and for detecting implausibly-fast replies
    /// that may indicate cheating.
    pub generation_time_ms: u64,

    /// BLAKE3 hash of the exact model file the worker loaded.
    /// Serves as layer 2 of the verification stack: the coordinator
    /// compares this against a whitelist of known-good model
    /// digests and rejects any worker that used the wrong model.
    pub model_digest: [u8; 32],

    /// BLAKE3 hash of the canonical logprob fingerprint produced by
    /// running a calibration prompt on the same model. Serves as
    /// layer 3 of the verification stack.
    ///
    /// 32 zeros means "logprobs not provided" (the coordinator
    /// can then decide whether to accept or reject based on its
    /// policy for that task_type).
    pub logprobs_hash: [u8; 32],

    /// Unix timestamp when the worker started inference.
    pub started_at: u64,

    /// Unix timestamp when the worker finished inference.
    pub finished_at: u64,
}

/// A signed ResultPayload, ready to be written to the results doc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResultEntry {
    /// The result payload itself.
    pub payload: ResultPayload,

    /// Ed25519 public key of the worker that produced the result
    /// (32 bytes).
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of the payload
    /// (64 bytes). See [`TaskEntry`] for the serde-big-array note.
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl ResultEntry {
    /// Sign a ResultPayload with the worker's keypair.
    pub fn sign(payload: ResultPayload, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&payload, DOMAIN_RESULT_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(ResultEntry {
            payload,
            worker_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the signature on this result.
    pub fn verify_signature(&self) -> Result<()> {
        let bytes = canonical_bytes(&self.payload, DOMAIN_RESULT_V1)?;
        crate::crypto::verify(&self.worker_pubkey, &bytes, &self.signature)
    }
}

/// A worker's claim on a task. Written into the tasks doc as a
/// sidecar entry (`claim:<task_id>`) to signal "I am working on
/// this, please do not assign it again".
///
/// Because iroh-docs is LWW (last-write-wins by timestamp), two
/// workers can legitimately write a [`ClaimEntry`] for the same
/// task at the same time. The coordinator breaks ties by keeping
/// the earliest-timestamped valid claim and the loser discards its
/// in-flight work.
///
/// The claim is always wrapped in a [`ClaimEntry`] before hitting
/// the doc — a raw `Claim` has no authenticity guarantee and the
/// coordinator refuses to act on it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claim {
    /// Canonical format version.
    pub version: u16,

    /// Which task is being claimed.
    pub task_id: String,

    /// The worker's public key.
    pub claimed_by: [u8; PUBLIC_KEY_LENGTH],

    /// Unix timestamp when the claim was written. The coordinator
    /// uses this to break ties between concurrent claims.
    pub claimed_at: u64,
}

impl Claim {
    /// Construct a new Claim with the current format version.
    pub fn new(
        task_id: impl Into<String>,
        claimed_by: [u8; PUBLIC_KEY_LENGTH],
        claimed_at: u64,
    ) -> Self {
        Claim {
            version: TASK_FORMAT_VERSION,
            task_id: task_id.into(),
            claimed_by,
            claimed_at,
        }
    }
}

/// A signed [`Claim`], ready to be written to the tasks doc under
/// `claim:<task_id>`.
///
/// Audit P2 item 2 (2026-04-10): prior to Sprint 4, `Claim` only
/// had a `new()` constructor and no way to authenticate the
/// claimant. The coordinator cannot safely break LWW ties without
/// a signature, so [`ClaimEntry`] wraps the claim alongside the
/// worker's public key and an Ed25519 signature over the canonical
/// bytes (with domain prefix [`DOMAIN_CLAIM_V1`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimEntry {
    /// The claim itself.
    pub claim: Claim,

    /// Ed25519 public key of the worker that minted this claim.
    /// Must equal [`Claim::claimed_by`] for the entry to verify;
    /// the redundancy catches split-brain bugs where a forwarder
    /// attributes a claim to a different key than the one that
    /// signed it.
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of the claim.
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl ClaimEntry {
    /// Sign a Claim with the worker's keypair.
    ///
    /// The resulting entry carries `worker_pubkey =
    /// keypair.public_bytes()`. [`ClaimEntry::verify_signature`]
    /// additionally checks that `claim.claimed_by == worker_pubkey`
    /// to prevent attribution mismatch.
    pub fn sign(claim: Claim, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&claim, DOMAIN_CLAIM_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(ClaimEntry {
            claim,
            worker_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the signature on this claim entry.
    ///
    /// Two checks:
    ///
    /// 1. `claim.claimed_by == worker_pubkey` (attribution
    ///    consistency).
    /// 2. Ed25519 signature is valid over
    ///    `canonical_bytes(&claim, DOMAIN_CLAIM_V1)`.
    pub fn verify_signature(&self) -> Result<()> {
        if self.claim.claimed_by != self.worker_pubkey {
            return Err(crate::error::NexusError::Crypto(
                "claim.claimed_by does not match worker_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.claim, DOMAIN_CLAIM_V1)?;
        crate::crypto::verify(&self.worker_pubkey, &bytes, &self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task() -> Task {
        let mut t = Task::new(
            "task-001",
            "analysis",
            "What is the capital of France?",
            "llama-3.1-8b",
            5,
            1_712_345_678,
        );
        t.system_prompt = "You are a helpful assistant.".into();
        t.metadata.insert("project".into(), "demo".into());
        t.metadata.insert("user".into(), "alice".into());
        t
    }

    fn sample_result_payload() -> ResultPayload {
        ResultPayload {
            version: TASK_FORMAT_VERSION,
            task_id: "task-001".into(),
            result_text: "Paris.".into(),
            tokens_generated: 2,
            generation_time_ms: 150,
            model_digest: [0xAA; 32],
            logprobs_hash: [0xBB; 32],
            started_at: 1_712_345_680,
            finished_at: 1_712_345_680,
        }
    }

    #[test]
    fn task_canonical_bytes_is_deterministic() {
        let t = sample_task();
        let a = canonical_bytes(&t, DOMAIN_TASK_V1).unwrap();
        let b = canonical_bytes(&t, DOMAIN_TASK_V1).unwrap();
        assert_eq!(a, b, "same input must produce identical bytes");
    }

    #[test]
    fn metadata_order_does_not_matter() {
        let mut t1 = Task::new("id", "t", "p", "m", 5, 0);
        t1.metadata.insert("a".into(), "1".into());
        t1.metadata.insert("b".into(), "2".into());

        let mut t2 = Task::new("id", "t", "p", "m", 5, 0);
        // insert in reversed order
        t2.metadata.insert("b".into(), "2".into());
        t2.metadata.insert("a".into(), "1".into());

        // BTreeMap guarantees alphabetical iteration and JCS
        // sorts map keys at every level, so the canonical bytes
        // must match regardless of insertion order.
        assert_eq!(
            canonical_bytes(&t1, DOMAIN_TASK_V1).unwrap(),
            canonical_bytes(&t2, DOMAIN_TASK_V1).unwrap()
        );
    }

    #[test]
    fn task_entry_sign_then_verify() {
        let kp = KeyPair::generate();
        let entry = TaskEntry::sign(sample_task(), &kp).unwrap();
        entry.verify_signature().expect("signature must verify");
        assert_eq!(entry.author_pubkey, kp.public_bytes());
    }

    #[test]
    fn task_entry_rejects_tampered_task() {
        let kp = KeyPair::generate();
        let mut entry = TaskEntry::sign(sample_task(), &kp).unwrap();
        entry.task.prompt = "TAMPERED".into();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn task_entry_rejects_wrong_signer() {
        let kp = KeyPair::generate();
        let other = KeyPair::generate();
        let mut entry = TaskEntry::sign(sample_task(), &kp).unwrap();
        entry.author_pubkey = other.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn result_entry_sign_then_verify() {
        let kp = KeyPair::generate();
        let entry = ResultEntry::sign(sample_result_payload(), &kp).unwrap();
        entry.verify_signature().expect("signature must verify");
    }

    #[test]
    fn result_entry_rejects_tampered_text() {
        let kp = KeyPair::generate();
        let mut entry = ResultEntry::sign(sample_result_payload(), &kp).unwrap();
        entry.payload.result_text = "Berlin.".into();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn task_roundtrip_through_canonical_bytes() {
        // Strip the domain prefix + null separator, then the
        // remaining bytes are plain JCS JSON and can round-trip
        // through serde_json::from_slice.
        let t = sample_task();
        let bytes = canonical_bytes(&t, DOMAIN_TASK_V1).unwrap();
        let body = &bytes[DOMAIN_TASK_V1.len() + 1..];
        let restored: Task = serde_json::from_slice(body).unwrap();
        assert_eq!(t, restored);
    }

    #[test]
    fn claim_entry_sign_and_verify() {
        let kp = KeyPair::generate();
        let claim = Claim::new("task-001", kp.public_bytes(), 1_712_345_700);
        let entry = ClaimEntry::sign(claim, &kp).unwrap();
        entry
            .verify_signature()
            .expect("claim signature must verify");
    }

    #[test]
    fn claim_entry_rejects_tampered_claim() {
        let kp = KeyPair::generate();
        let claim = Claim::new("task-001", kp.public_bytes(), 1_712_345_700);
        let mut entry = ClaimEntry::sign(claim, &kp).unwrap();
        entry.claim.task_id = "task-999".into();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn claim_entry_rejects_attribution_mismatch() {
        // worker_pubkey on the envelope must match claim.claimed_by.
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        let claim = Claim::new("task-001", kp_a.public_bytes(), 1_712_345_700);
        let mut entry = ClaimEntry::sign(claim, &kp_a).unwrap();
        entry.worker_pubkey = kp_b.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn claim_entry_rejects_wrong_signer() {
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let claim = Claim::new("task-001", real.public_bytes(), 1_712_345_700);
        // Sign with real, then flip both the signer pubkey AND
        // claim.claimed_by so the attribution check passes — the
        // raw signature must still fail because it was produced
        // by `real`, not `impostor`.
        let mut entry = ClaimEntry::sign(claim, &real).unwrap();
        entry.worker_pubkey = impostor.public_bytes();
        entry.claim.claimed_by = impostor.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn version_field_is_present_in_canonical_output() {
        // JCS preserves all JSON content; after stripping the
        // domain prefix + null separator the body must contain
        // the exact key/value `"version":1`.
        let t = sample_task();
        let bytes = canonical_bytes(&t, DOMAIN_TASK_V1).unwrap();
        let body = &bytes[DOMAIN_TASK_V1.len() + 1..];
        let text = std::str::from_utf8(body).unwrap();
        assert!(
            text.contains(&format!("\"version\":{TASK_FORMAT_VERSION}")),
            "version field missing from canonical output: {text}"
        );
    }

    #[test]
    fn task_and_claim_canonical_bytes_do_not_collide() {
        // Regression: a ResultPayload shaped enough like a Task
        // must not share canonical bytes with a Task, thanks to
        // the domain prefix. This protects against cross-type
        // signature replay.
        let kp = KeyPair::generate();
        let claim = Claim::new("task-001", kp.public_bytes(), 1);
        let claim_bytes = canonical_bytes(&claim, DOMAIN_CLAIM_V1).unwrap();
        let task_bytes = canonical_bytes(&claim, DOMAIN_TASK_V1).unwrap();
        assert_ne!(claim_bytes, task_bytes);
    }
}
