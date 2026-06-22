// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase I — N3 activation commit-reveal (opML-style dispute anchor).
//!
//! N3 is the **dispute** level of the graded verification schema (addendum §3):
//! never assigned from criticality, only escalated when a shard run is
//! contested. Its job is to let a worker **non-repudiably commit**, per
//! inter-stage frontier, to the activation fingerprint it stands behind, so a
//! later dispute can replay that exact frontier and pin misexecution to a
//! specific stage.
//!
//! ## Commit-reveal, not byte-equality (cross-GPU honesty)
//!
//! opML (arXiv 2401.17555) builds its fraud-proof on a **bit-exact
//! deterministic** VM (fixed-point + softfloat + fixed seed), reducing a dispute
//! to a single divergent instruction. SBFB has **no** such VM — GPU
//! non-determinism is the very reason N0 uses the locality-sensitive
//! [`crate::toploc`] fingerprint instead of a hash. So an N3 reveal is verified
//! in **two** distinct steps, and the second is NEVER commitment equality:
//!
//! 1. **Binding** — the revealed `(sketch, nonce)` must open the committed
//!    `BLAKE3(sketch.to_bytes() || nonce)`. This proves the worker is revealing
//!    the same fingerprint it committed to (non-repudiation), and the nonce
//!    hides it before reveal (a dispute can be opened without the activations
//!    leaking earlier).
//! 2. **Correctness** — the revealed sketch is compared to the verifier's
//!    independent recompute via the **tolerant**
//!    [`crate::toploc::ToplocFingerprint::compare`] (exponent-exact +
//!    mantissa-mean/median), which a cross-GPU honest re-run passes and a
//!    model/precision swap fails. Comparing the 32-byte commitments by equality
//!    here would false-reject every honest cross-hardware reveal (BLAKE3 destroys
//!    locality — one bit avalanches).
//!
//! This is therefore **not** an opML soundness guarantee: it binds *which*
//! fingerprint a worker stands behind and localises a contested frontier; it
//! does not cryptographically prove correct execution (that is N4 zkML,
//! out of scope). See [`crate::sentinel`] for the O(1) statistical localiser
//! that flags *which* frontier to dispute in the first place.
//!
//! ## Wire shape (mirror of [`crate::shard_plan`])
//!
//! [`ActivationCommitPayload`] is the *unsigned* payload whose every field
//! contributes to the canonical bytes; [`ActivationCommitEntry`] wraps it with a
//! **redundant** signer identity (attribution split-brain mitigation) and an
//! Ed25519 signature over [`canonical_bytes`] tagged with
//! [`DOMAIN_ACTIVATION_COMMIT_V1`]. The signature and the redundant identity are
//! never part of the canonical bytes. The reveal ([`ActivationReveal`]) travels
//! **off** the signed envelope: it carries the full
//! [`crate::toploc::ToplocFingerprint`] sketch (the comparable), which the
//! 32-byte on-wire commitment slot cannot.
//!
//! ## No floats, 0 bump
//!
//! The payload is all-integer / `[u8; 32]` / bounded-`String`; the commitment
//! pre-image is the sketch's all-integer canonical bytes plus the nonce. A new
//! signed type with its own domain tag bumps nothing (pre-launch additive
//! policy, the S74 `DOMAIN_SEED_REQUEST_V1` pattern).

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_ACTIVATION_COMMIT_V1, canonical_bytes};
use crate::crypto::{BLAKE3_BYTES, KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES, blake3_hash};
use crate::error::{NexusError, Result};
use crate::toploc::ToplocFingerprint;

/// Current on-wire version for [`ActivationCommitPayload`]. A brand-new signed
/// type, so introducing it bumps nothing (the S74 `DOMAIN_SEED_REQUEST_V1`
/// pattern). Verifiers refuse a payload whose `version` they do not understand.
pub const ACTIVATION_COMMIT_FORMAT_VERSION: u16 = 1;

/// Per-field byte cap on a commit's `session_id` (mirrors
/// [`crate::shard_plan::SESSION_ID_MAX`]): a short stable handle, not a blob.
pub const ACTIVATION_SESSION_ID_MAX: usize = 128;

/// Length of the hiding nonce. 32 bytes of OS entropy makes a commitment
/// pre-image dictionary attack on the (small-cardinality) sketch space
/// infeasible, so the commit hides the fingerprint until reveal.
pub const ACTIVATION_NONCE_BYTES: usize = 32;

/// The binding + hiding commitment of a frontier fingerprint:
/// `BLAKE3(sketch.to_bytes() || nonce)`.
///
/// The pre-image is all-integer (the sketch's canonical bytes,
/// [`ToplocFingerprint::to_bytes`]) followed by the 32-byte nonce, so a Rust
/// committer and a Python verifier derive identical bytes. Equal commitments ⟺
/// the same `(sketch, nonce)`; this is the **binding** check, never the
/// correctness verdict (that is the tolerant [`ToplocFingerprint::compare`]).
#[must_use]
pub fn activation_commitment(
    sketch: &ToplocFingerprint,
    nonce: &[u8; ACTIVATION_NONCE_BYTES],
) -> [u8; BLAKE3_BYTES] {
    let mut pre = sketch.to_bytes();
    pre.extend_from_slice(nonce);
    blake3_hash(&pre)
}

/// The unsigned activation-commit payload: a worker's binding commitment to the
/// activation fingerprint at one shard frontier.
///
/// Every field contributes to the canonical bytes the worker signs; nothing
/// outside this struct (the envelope's redundant `worker_pubkey` / `signature`)
/// is covered by the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationCommitPayload {
    /// Must equal [`ACTIVATION_COMMIT_FORMAT_VERSION`]. `#[serde(default)]` is
    /// intentionally NOT applied: a missing version is malformed, not a
    /// runtime-tolerant omission.
    pub version: u16,

    /// Ed25519 public key of the worker that ran this frontier and signed the
    /// commit. Cross-checked against the envelope's redundant `worker_pubkey`.
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// The session this commit belongs to — binds the commit to one
    /// [`crate::shard_plan::ShardedSessionManifest::session_id`] so a commit from
    /// a past session cannot be replayed into a current dispute. Bounded by
    /// [`ACTIVATION_SESSION_ID_MAX`].
    pub session_id: String,

    /// The inter-stage frontier this commit pins (a shard boundary
    /// `layer_end`, [`crate::shard_plan::ShardAssignment::layer_end`]). Binding
    /// the frontier index into the signed pre-image stops a worker re-mapping a
    /// commit to a different frontier after the fact (anti-grinding, the
    /// discipline of the N1 draw seed, [`crate::verifiable_draw`]).
    pub frontier_index: u32,

    /// The binding + hiding commitment [`activation_commitment`] of the frontier
    /// fingerprint. The full sketch is revealed off-envelope ([`ActivationReveal`]).
    pub commitment: [u8; BLAKE3_BYTES],
}

impl ActivationCommitPayload {
    /// Construct a commit at the current format version. `commitment` is the
    /// output of [`activation_commitment`] over the frontier sketch and a fresh
    /// high-entropy nonce.
    #[must_use]
    pub fn new(
        worker_pubkey: [u8; PUBLIC_KEY_LENGTH],
        session_id: impl Into<String>,
        frontier_index: u32,
        commitment: [u8; BLAKE3_BYTES],
    ) -> Self {
        ActivationCommitPayload {
            version: ACTIVATION_COMMIT_FORMAT_VERSION,
            worker_pubkey,
            session_id: session_id.into(),
            frontier_index,
            commitment,
        }
    }
}

/// A signed [`ActivationCommitPayload`].
///
/// The signature is over [`canonical_bytes`] of the inner payload with
/// [`DOMAIN_ACTIVATION_COMMIT_V1`] as the domain tag (mirror of
/// [`crate::shard_plan::RunProofEntry`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationCommitEntry {
    /// The commit payload.
    pub payload: ActivationCommitPayload,

    /// Redundant Ed25519 pubkey of the signing worker. MUST equal
    /// [`ActivationCommitPayload::worker_pubkey`]; the verifier rejects a
    /// mismatch (attribution split-brain mitigation).
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of [`Self::payload`].
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl ActivationCommitEntry {
    /// Sign an [`ActivationCommitPayload`] with the worker keypair.
    ///
    /// Validates, before signing (mirror of
    /// [`crate::shard_plan::RunProofEntry::sign`]):
    /// 1. `payload.worker_pubkey == keypair.public_bytes()`.
    /// 2. DoS caps ([`check_activation_commit_caps`]).
    pub fn sign(payload: ActivationCommitPayload, keypair: &KeyPair) -> Result<Self> {
        if payload.worker_pubkey != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "activation commit: worker_pubkey does not match signing keypair".into(),
            ));
        }
        check_activation_commit_caps(&payload)?;
        let bytes = canonical_bytes(&payload, DOMAIN_ACTIVATION_COMMIT_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(ActivationCommitEntry {
            worker_pubkey: keypair.public_bytes(),
            payload,
            signature,
        })
    }

    /// Verify an [`ActivationCommitEntry`].
    ///
    /// Checks, in order (cap-before-crypto, mirror of
    /// [`crate::shard_plan::RunProofEntry::verify_signature`]):
    /// 1. `payload.version == ACTIVATION_COMMIT_FORMAT_VERSION`.
    /// 2. DoS caps (before hashing).
    /// 3. `payload.worker_pubkey == self.worker_pubkey` (attribution).
    /// 4. Ed25519 signature valid over the canonical bytes with
    ///    [`DOMAIN_ACTIVATION_COMMIT_V1`].
    pub fn verify_signature(&self) -> Result<()> {
        if self.payload.version != ACTIVATION_COMMIT_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "activation commit version mismatch (got {}, expected {})",
                self.payload.version, ACTIVATION_COMMIT_FORMAT_VERSION
            )));
        }
        check_activation_commit_caps(&self.payload)?;
        if self.payload.worker_pubkey != self.worker_pubkey {
            return Err(NexusError::Crypto(
                "activation commit: payload worker_pubkey does not match envelope worker_pubkey"
                    .into(),
            ));
        }
        let bytes = canonical_bytes(&self.payload, DOMAIN_ACTIVATION_COMMIT_V1)?;
        crate::crypto::verify(&self.worker_pubkey, &bytes, &self.signature)
    }
}

/// Reject a commit whose `session_id` exceeds its DoS cap. Enforced at sign AND
/// verify (mirror of [`crate::shard_plan`]'s `check_run_proof_caps`).
fn check_activation_commit_caps(payload: &ActivationCommitPayload) -> Result<()> {
    if payload.session_id.len() > ACTIVATION_SESSION_ID_MAX {
        return Err(NexusError::Crypto(format!(
            "activation commit session_id has {} bytes, exceeds ACTIVATION_SESSION_ID_MAX={}",
            payload.session_id.len(),
            ACTIVATION_SESSION_ID_MAX
        )));
    }
    Ok(())
}

/// The revealed opening of an [`ActivationCommitPayload`], carried **off** the
/// signed envelope. Deliberately not `Serialize`/`Deserialize`: it transports a
/// full [`ToplocFingerprint`] (whose only on-wire form is the DoS-bounded
/// [`ToplocFingerprint::to_bytes`]/[`ToplocFingerprint::from_bytes`]), not the
/// 32-byte commitment.
#[derive(Debug, Clone)]
pub struct ActivationReveal {
    /// The full frontier fingerprint — the comparable the 32-byte commitment
    /// slot cannot carry.
    pub sketch: ToplocFingerprint,
    /// The hiding nonce used to form the commitment.
    pub nonce: [u8; ACTIVATION_NONCE_BYTES],
}

impl ActivationReveal {
    /// Bundle a frontier sketch with its hiding nonce.
    #[must_use]
    pub fn new(sketch: ToplocFingerprint, nonce: [u8; ACTIVATION_NONCE_BYTES]) -> Self {
        ActivationReveal { sketch, nonce }
    }

    /// The commitment this reveal opens to ([`activation_commitment`]).
    #[must_use]
    pub fn commitment(&self) -> [u8; BLAKE3_BYTES] {
        activation_commitment(&self.sketch, &self.nonce)
    }

    /// Whether this reveal opens `committed`'s commitment (the binding check).
    #[must_use]
    pub fn opens(&self, committed: &ActivationCommitPayload) -> bool {
        self.commitment() == committed.commitment
    }
}

/// The verdict of opening a contested commit against a verifier's independent
/// recompute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevealVerdict {
    /// The reveal does not open the commitment (wrong sketch or nonce). The
    /// worker is repudiating its own commit — a provable protocol fault,
    /// independent of correctness.
    CommitmentMismatch,
    /// The reveal opens the commitment AND the revealed sketch tolerantly matches
    /// the verifier's recompute: the contested frontier ran honestly.
    Accepted,
    /// The reveal opens the commitment but the revealed sketch diverges from the
    /// verifier's recompute beyond [`crate::toploc`] tolerance: misexecution
    /// localised to this frontier.
    Divergent,
}

/// Resolve a dispute at one frontier: open the worker's signed commit against the
/// verifier's independent recompute.
///
/// Binding first (the reveal must open the commitment), then the **tolerant**
/// correctness compare — never commitment equality (which a cross-GPU honest
/// re-run fails by construction). The caller has already checked the
/// [`ActivationCommitEntry`] signature and that the recompute used the same
/// model + prompt + frontier.
#[must_use]
pub fn verify_reveal(
    committed: &ActivationCommitPayload,
    reveal: &ActivationReveal,
    verifier_recompute: &ToplocFingerprint,
) -> RevealVerdict {
    if !reveal.opens(committed) {
        return RevealVerdict::CommitmentMismatch;
    }
    if verifier_recompute.compare(&reveal.sketch).accepted {
        RevealVerdict::Accepted
    } else {
        RevealVerdict::Divergent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_sketch() -> ToplocFingerprint {
        ToplocFingerprint::from_topk(&[(1, 100.0), (3, 200.0), (5, 50.0), (7, -30.0)])
    }

    fn sample_payload(
        worker: &KeyPair,
        sketch: &ToplocFingerprint,
        nonce: &[u8; 32],
    ) -> ActivationCommitPayload {
        ActivationCommitPayload::new(
            worker.public_bytes(),
            "session-70b-1",
            16,
            activation_commitment(sketch, nonce),
        )
    }

    #[test]
    fn n3_activation_commit_reveal_roundtrip() {
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x5Au8; 32];
        let payload = sample_payload(&worker, &sketch, &nonce);

        // Sign + verify the commit envelope.
        let entry = ActivationCommitEntry::sign(payload.clone(), &worker).unwrap();
        entry
            .verify_signature()
            .expect("freshly signed activation commit must verify");
        assert_eq!(entry.worker_pubkey, worker.public_bytes());
        assert_eq!(entry.payload.version, ACTIVATION_COMMIT_FORMAT_VERSION);

        // Reveal opens the commitment (binding).
        let reveal = ActivationReveal::new(sketch.clone(), nonce);
        assert!(
            reveal.opens(&entry.payload),
            "the reveal must open its commit"
        );

        // Verdict against an honest cross-GPU recompute (close, not byte-equal)
        // is Accepted via the TOLERANT compare — NOT commitment equality.
        let recompute =
            ToplocFingerprint::from_topk(&[(1, 101.0), (3, 202.0), (5, 50.5), (7, -30.25)]);
        assert_ne!(
            reveal.commitment(),
            activation_commitment(&recompute, &nonce),
            "honest recompute differs byte-wise (the verdict is tolerant, not hash-eq)"
        );
        assert_eq!(
            verify_reveal(&entry.payload, &reveal, &recompute),
            RevealVerdict::Accepted
        );

        // JSON round-trip of the signed envelope re-verifies.
        let j = serde_json::to_vec(&entry).unwrap();
        let back: ActivationCommitEntry = serde_json::from_slice(&j).unwrap();
        assert_eq!(back, entry);
        back.verify_signature().unwrap();
    }

    #[test]
    fn n3_reveal_rejects_wrong_nonce_and_wrong_sketch() {
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x11u8; 32];
        let payload = sample_payload(&worker, &sketch, &nonce);

        // Right sketch, wrong nonce → does not open (binding broken).
        let bad_nonce = ActivationReveal::new(sketch.clone(), [0x22u8; 32]);
        assert!(!bad_nonce.opens(&payload));
        assert_eq!(
            verify_reveal(&payload, &bad_nonce, &sketch),
            RevealVerdict::CommitmentMismatch
        );

        // Right nonce, different sketch → does not open.
        let other_sketch = ToplocFingerprint::from_topk(&[(2, 9.0), (4, 8.0)]);
        let bad_sketch = ActivationReveal::new(other_sketch, nonce);
        assert!(!bad_sketch.opens(&payload));
        assert_eq!(
            verify_reveal(&payload, &bad_sketch, &sketch),
            RevealVerdict::CommitmentMismatch
        );
    }

    #[test]
    fn n3_reveal_divergent_localizes_misexecution_cross_gpu() {
        // The reveal opens the commitment (honest binding) but the verifier's
        // recompute is a model/precision swap (disjoint top-k) → Divergent: the
        // tolerant compare rejects, localising misexecution at this frontier.
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x33u8; 32];
        let payload = sample_payload(&worker, &sketch, &nonce);
        let reveal = ActivationReveal::new(sketch, nonce);
        assert!(reveal.opens(&payload), "binding holds");

        let swapped = ToplocFingerprint::from_topk(&[(80, 1.0), (81, 2.0), (82, 3.0)]);
        assert_eq!(
            verify_reveal(&payload, &reveal, &swapped),
            RevealVerdict::Divergent
        );
    }

    #[test]
    fn n3_commit_binds_session_and_frontier_anti_replay() {
        // A commit signed for (session A, frontier 16) must not verify if its
        // session_id or frontier_index is mutated after signing — both are in the
        // canonical pre-image, so the signature breaks (anti-replay across
        // sessions / frontiers).
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x44u8; 32];
        let mut entry =
            ActivationCommitEntry::sign(sample_payload(&worker, &sketch, &nonce), &worker).unwrap();

        let mut cross_session = entry.clone();
        cross_session.payload.session_id = "session-OTHER".into();
        assert!(
            cross_session.verify_signature().is_err(),
            "replaying a commit into another session must fail"
        );

        entry.payload.frontier_index += 1;
        assert!(
            entry.verify_signature().is_err(),
            "re-mapping a commit to another frontier must fail"
        );
    }

    #[test]
    fn n3_commit_sign_and_verify_reject_wrong_signer_and_attribution() {
        let owner = KeyPair::generate();
        let other = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x55u8; 32];

        // Signing someone else's commit fails at sign time.
        assert!(
            ActivationCommitEntry::sign(sample_payload(&owner, &sketch, &nonce), &other).is_err()
        );

        // Tampering the envelope's redundant identity is rejected at verify.
        let mut entry =
            ActivationCommitEntry::sign(sample_payload(&owner, &sketch, &nonce), &owner).unwrap();
        entry.worker_pubkey = other.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn n3_commit_verify_rejects_wrong_version_and_tamper() {
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x66u8; 32];

        // Unknown version is rejected at the version gate (signed validly first).
        let mut bad_ver = sample_payload(&worker, &sketch, &nonce);
        bad_ver.version = ACTIVATION_COMMIT_FORMAT_VERSION + 1;
        let entry = ActivationCommitEntry::sign(bad_ver, &worker).unwrap();
        assert!(entry.verify_signature().is_err());

        // Mutating the commitment after signing breaks the signature.
        let mut tampered =
            ActivationCommitEntry::sign(sample_payload(&worker, &sketch, &nonce), &worker).unwrap();
        tampered.payload.commitment[0] ^= 0xFF;
        assert!(tampered.verify_signature().is_err());

        // Flipped signature byte → rejected.
        let mut bad_sig =
            ActivationCommitEntry::sign(sample_payload(&worker, &sketch, &nonce), &worker).unwrap();
        bad_sig.signature[0] ^= 0xFF;
        assert!(bad_sig.verify_signature().is_err());
    }

    #[test]
    fn n3_commit_caps_reject_oversized_session_id_before_crypto() {
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x77u8; 32];
        let mut payload = sample_payload(&worker, &sketch, &nonce);
        payload.session_id = "s".repeat(ACTIVATION_SESSION_ID_MAX + 1);

        // Sign-side cap.
        assert!(ActivationCommitEntry::sign(payload.clone(), &worker).is_err());

        // Verify-side cap fires BEFORE the crypto check: forge a zero signature
        // and assert the error is the *cap* error, not a signature error.
        let entry = ActivationCommitEntry {
            worker_pubkey: worker.public_bytes(),
            payload,
            signature: [0u8; SIGNATURE_BYTES],
        };
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
    fn n3_domain_separated_from_run_proof() {
        // An activation-commit signature can never be replayed as a run-proof
        // signature even minted with the same key: distinct domain tags →
        // disjoint canonical pre-images.
        let worker = KeyPair::generate();
        let sketch = sample_sketch();
        let nonce = [0x88u8; 32];
        let payload = sample_payload(&worker, &sketch, &nonce);
        let as_commit = canonical_bytes(&payload, DOMAIN_ACTIVATION_COMMIT_V1).unwrap();
        let as_run_proof =
            canonical_bytes(&payload, crate::canonical::DOMAIN_RUN_PROOF_V1).unwrap();
        assert_ne!(as_commit, as_run_proof);
    }
}
