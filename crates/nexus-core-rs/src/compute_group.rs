// SPDX-License-Identifier: AGPL-3.0-or-later
//! Private compute-group admission allowlist (Sprint 77 Phase B).
//!
//! A **compute group** is the explicit, signed admission boundary of a
//! private sharded-inference session: an Ed25519 allowlist of the
//! `worker_pubkey`s authorised to open a `sbfb/shard/1` data-plane
//! connection and carry a block of model layers. A worker absent from the
//! allowlist is rejected at the ALPN handshake — before any activation
//! frame is read (see [`crate::shard::ShardProtocol`]).
//!
//! ## Why a private group, not an open pool (Day-0 D5, scope cut #8)
//!
//! Sharded inference puts a worker *inside* the inference pipeline: it
//! sees the activations of the layers it runs in the clear (there is no
//! consumer-grade GPU TEE in 2026, so confidentiality face to the workers
//! is a physical limit, scope cut #4). The mitigation is **admission
//! control**: only peers an initiator has explicitly enrolled may join.
//! The allowlist bounds *who can participate*; it does **not** encrypt the
//! activations and it does **not** guarantee ≥1 honest member (SI-4
//! collusion stays a residual — see `SPLIT_INFERENCE_DESIGN.md`). It is a
//! routing-of-trust primitive for the closed pilot (R-iroh-audit P0 — no
//! public surface), never an open group-discovery mechanism.
//!
//! ## Where it sits relative to invites (M19)
//!
//! This reuses the M19 crypto *machinery* (Ed25519 + JCS canonical bytes +
//! domain separation) but is a distinct type: an [`crate::seed`] invite
//! binds a `(project_id, archive_hash)` pair for seeding; a
//! [`ComputeGroup`] binds a *set of worker identities* for one private
//! compute session. The initiator signs the allowlist but is **not** a
//! network authority — it is an ad-hoc private group between peers who
//! already know each other (the pilot model), not a central registry.
//!
//! ## Wire shape (mirror of [`crate::node_directory`])
//!
//! A [`ComputeGroup`] is the *unsigned* payload: the initiator's identity,
//! a monotonic `revision`, a stable `group_id`, and a bounded list of
//! member `worker_pubkey`s. A [`ComputeGroupEntry`] wraps it together with
//! a redundant `initiator` field (attribution split-brain mitigation, the
//! same check [`crate::node_directory::NodeDirectoryEntry`] /
//! [`crate::seed::SeedRequestEnvelope`] already apply) and an Ed25519
//! signature over [`canonical_bytes`] of the group with
//! [`DOMAIN_COMPUTE_GROUP_V1`] as the domain tag. The `signature` and
//! redundant `initiator` are NEVER part of the canonical bytes.
//!
//! ## DoS mitigation
//!
//! [`COMPUTE_GROUP_MAX_MEMBERS`] bounds the allowlist size and
//! [`COMPUTE_GROUP_ID_MAX`] bounds the `group_id` string — enforced at
//! BOTH sign and verify so an initiator cannot accidentally produce a
//! group its own members reject.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_COMPUTE_GROUP_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Current on-wire version for [`ComputeGroup`] payloads.
///
/// Independent from every existing `*_FORMAT_VERSION`: this is a brand-new
/// signed type, so introducing it bumps nothing (pre-launch additive
/// policy, the S74 `DOMAIN_SEED_REQUEST_V1` pattern). Verifiers refuse a
/// payload whose `version` they do not understand.
pub const COMPUTE_GROUP_FORMAT_VERSION: u16 = 1;

/// Hard upper bound on the number of members a single signed compute group
/// may carry. Caps the DoS impact of an initiator publishing a
/// pathologically large allowlist (mirrors
/// [`crate::node_directory::NODE_DIRECTORY_MAX_ENTRIES`]). 256 distinct
/// GPUs in one private pilot session is well above any realistic
/// sharded-inference fan-out (3-5 machines, addendum §7) and well below a
/// RAM / verification pain threshold.
pub const COMPUTE_GROUP_MAX_MEMBERS: usize = 256;

/// Per-field byte cap on [`ComputeGroup::group_id`]. A `group_id` is a
/// short stable handle for the private session, not a free-text blob.
pub const COMPUTE_GROUP_ID_MAX: usize = 128;

/// The unsigned compute-group allowlist payload.
///
/// Every field here contributes to the canonical bytes the initiator
/// signs; nothing outside this struct (the envelope's redundant
/// `initiator` / `signature`) is covered by the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComputeGroup {
    /// Must equal [`COMPUTE_GROUP_FORMAT_VERSION`] to be accepted by this
    /// build. `#[serde(default)]` is intentionally NOT applied: a missing
    /// version is a malformed group, not a runtime-tolerant omission.
    pub version: u16,

    /// Stable, initiator-chosen handle for this private session. Lets a
    /// later session manifest (Phase C/J) correlate a running pipeline to
    /// the allowlist that admits its workers. Bounded by
    /// [`COMPUTE_GROUP_ID_MAX`].
    pub group_id: String,

    /// Ed25519 public key of the node that owns + signs this allowlist.
    /// This is the group *owner*, NOT necessarily a worker: if the
    /// initiator also runs a shard it must appear in [`Self::members`] as
    /// well. Cross-checked against the envelope's redundant `initiator`.
    pub initiator: [u8; PUBLIC_KEY_LENGTH],

    /// Monotonic revision counter. The initiator bumps it when rotating
    /// the allowlist (adding / removing members). Rollback protection (a
    /// stateful "strictly greater" check) is an ingest-layer concern, not
    /// enforced in this crypto module — mirrors
    /// [`crate::node_directory::NodeDirectory::revision`].
    pub revision: u64,

    /// The authorised worker public keys (the admission allowlist). A peer
    /// whose `conn.remote_id()` is not in this list is rejected at the
    /// `sbfb/shard/1` handshake. MUST have
    /// `members.len() <= COMPUTE_GROUP_MAX_MEMBERS`. An empty list is a
    /// valid (degenerate) group that admits no one.
    pub members: Vec<[u8; PUBLIC_KEY_LENGTH]>,
}

impl ComputeGroup {
    /// Construct a new, empty compute group at the current format version.
    /// Push members via [`Self::with_member`] before signing.
    pub fn new(
        initiator: [u8; PUBLIC_KEY_LENGTH],
        group_id: impl Into<String>,
        revision: u64,
    ) -> Self {
        ComputeGroup {
            version: COMPUTE_GROUP_FORMAT_VERSION,
            group_id: group_id.into(),
            initiator,
            revision,
            members: Vec::new(),
        }
    }

    /// Builder-style: append an authorised `worker_pubkey`.
    pub fn with_member(mut self, worker_pubkey: [u8; PUBLIC_KEY_LENGTH]) -> Self {
        self.members.push(worker_pubkey);
        self
    }

    /// Whether `worker_pubkey` is on the admission allowlist.
    ///
    /// Membership is checked against [`Self::members`] only — the
    /// [`Self::initiator`] is the group owner and is NOT implicitly
    /// admitted as a worker (it must be added explicitly if it runs a
    /// shard).
    pub fn is_member(&self, worker_pubkey: &[u8; PUBLIC_KEY_LENGTH]) -> bool {
        self.members.iter().any(|m| m == worker_pubkey)
    }
}

/// A signed [`ComputeGroup`], ready to be stored / shared with the
/// session's members.
///
/// The signature is computed over [`canonical_bytes`] of the inner
/// [`ComputeGroup`] with [`DOMAIN_COMPUTE_GROUP_V1`] as the domain tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComputeGroupEntry {
    /// The allowlist itself.
    pub group: ComputeGroup,

    /// Redundant Ed25519 pubkey of the signing initiator. MUST equal
    /// [`ComputeGroup::initiator`]; the verifier rejects any entry where
    /// the two disagree (attribution split-brain mitigation, same pattern
    /// as [`crate::node_directory::NodeDirectoryEntry`]).
    pub initiator: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of [`Self::group`]
    /// (64 bytes; `serde_big_array` because serde does not derive for
    /// arrays > 32).
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl ComputeGroupEntry {
    /// Sign a [`ComputeGroup`] with the initiator keypair.
    ///
    /// Validates three invariants before signing (mirror of
    /// [`crate::node_directory::NodeDirectoryEntry::sign`]):
    ///
    /// 1. `group.initiator == keypair.public_bytes()` — the caller signs
    ///    their own group; a mismatch surfaces the typo immediately.
    /// 2. `group.members.len() <= COMPUTE_GROUP_MAX_MEMBERS` — the DoS cap
    ///    applies to signing too, so a node cannot produce a group its own
    ///    members reject.
    /// 3. `group.group_id` is within [`COMPUTE_GROUP_ID_MAX`].
    pub fn sign(group: ComputeGroup, keypair: &KeyPair) -> Result<Self> {
        if group.initiator != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "compute group: initiator does not match signing keypair".into(),
            ));
        }
        check_group_caps(&group)?;
        let bytes = canonical_bytes(&group, DOMAIN_COMPUTE_GROUP_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(ComputeGroupEntry {
            group,
            initiator: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify a [`ComputeGroupEntry`].
    ///
    /// Checks, in order (mirror of
    /// [`crate::node_directory::NodeDirectoryEntry::verify_signature`]):
    ///
    /// 1. `group.version == COMPUTE_GROUP_FORMAT_VERSION` — reject a
    ///    payload the current build does not understand.
    /// 2. `group.members.len() <= COMPUTE_GROUP_MAX_MEMBERS` and
    ///    `group_id` within cap — enforce the DoS bounds before hashing.
    /// 3. `group.initiator == self.initiator` — attribution consistency.
    /// 4. Ed25519 signature valid over the canonical bytes of `group`
    ///    with [`DOMAIN_COMPUTE_GROUP_V1`].
    ///
    /// Revision rollback protection is NOT checked here — it is a stateful
    /// property for the ingest/storage layer.
    pub fn verify_signature(&self) -> Result<()> {
        if self.group.version != COMPUTE_GROUP_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "compute group version mismatch (got {}, expected {})",
                self.group.version, COMPUTE_GROUP_FORMAT_VERSION
            )));
        }
        check_group_caps(&self.group)?;
        if self.group.initiator != self.initiator {
            return Err(NexusError::Crypto(
                "compute group: payload initiator does not match envelope initiator".into(),
            ));
        }
        let bytes = canonical_bytes(&self.group, DOMAIN_COMPUTE_GROUP_V1)?;
        crate::crypto::verify(&self.initiator, &bytes, &self.signature)
    }

    /// Convenience: membership check on the wrapped group.
    pub fn is_member(&self, worker_pubkey: &[u8; PUBLIC_KEY_LENGTH]) -> bool {
        self.group.is_member(worker_pubkey)
    }
}

/// Reject a group whose membership list or `group_id` exceeds its DoS cap.
/// Enforced at sign AND verify so a group can never be produced that its
/// own members would reject.
fn check_group_caps(group: &ComputeGroup) -> Result<()> {
    if group.members.len() > COMPUTE_GROUP_MAX_MEMBERS {
        return Err(NexusError::Crypto(format!(
            "compute group has {} members, exceeds COMPUTE_GROUP_MAX_MEMBERS={}",
            group.members.len(),
            COMPUTE_GROUP_MAX_MEMBERS
        )));
    }
    if group.group_id.len() > COMPUTE_GROUP_ID_MAX {
        return Err(NexusError::Crypto(format!(
            "compute group group_id has {} bytes, exceeds COMPUTE_GROUP_ID_MAX={}",
            group.group_id.len(),
            COMPUTE_GROUP_ID_MAX
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_group(initiator: &KeyPair, members: &[&KeyPair]) -> ComputeGroup {
        let mut g = ComputeGroup::new(initiator.public_bytes(), "pilot-70b", 1);
        for m in members {
            g = g.with_member(m.public_bytes());
        }
        g
    }

    #[test]
    fn compute_group_signature_roundtrip() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let group = sample_group(&initiator, &[&w1, &w2]);
        let entry = ComputeGroupEntry::sign(group, &initiator).unwrap();
        entry
            .verify_signature()
            .expect("freshly signed group must verify");
        assert_eq!(entry.initiator, initiator.public_bytes());
        assert_eq!(entry.group.version, COMPUTE_GROUP_FORMAT_VERSION);
        assert_eq!(entry.group.members.len(), 2);
    }

    #[test]
    fn compute_group_is_member_excludes_initiator_and_outsiders() {
        let initiator = KeyPair::generate();
        let member = KeyPair::generate();
        let outsider = KeyPair::generate();
        let group = sample_group(&initiator, &[&member]);
        assert!(
            group.is_member(&member.public_bytes()),
            "enrolled worker is a member"
        );
        assert!(
            !group.is_member(&initiator.public_bytes()),
            "the initiator is the owner, NOT implicitly an admitted worker"
        );
        assert!(
            !group.is_member(&outsider.public_bytes()),
            "a non-enrolled peer is not a member"
        );
    }

    #[test]
    fn compute_group_verify_rejects_tampered_payload() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let mut entry =
            ComputeGroupEntry::sign(sample_group(&initiator, &[&w1]), &initiator).unwrap();
        // Smuggle an extra member in after signing.
        entry.group.members.push(KeyPair::generate().public_bytes());
        assert!(
            entry.verify_signature().is_err(),
            "adding a member after signing must fail verification"
        );
    }

    #[test]
    fn compute_group_verify_rejects_tampered_signature() {
        let initiator = KeyPair::generate();
        let mut entry = ComputeGroupEntry::sign(sample_group(&initiator, &[]), &initiator).unwrap();
        entry.signature[0] ^= 0xFF;
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn compute_group_verify_rejects_attribution_mismatch() {
        // A valid signature over the payload, but the envelope staples a
        // different initiator than the one inside the signed group.
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut entry = ComputeGroupEntry::sign(sample_group(&real, &[]), &real).unwrap();
        entry.initiator = impostor.public_bytes();
        assert!(
            entry.verify_signature().is_err(),
            "payload.initiator != envelope.initiator must be rejected"
        );
    }

    #[test]
    fn compute_group_sign_rejects_wrong_signer() {
        // The initiator field inside the group must match the signing key.
        let owner = KeyPair::generate();
        let other = KeyPair::generate();
        let group = sample_group(&owner, &[]);
        assert!(
            ComputeGroupEntry::sign(group, &other).is_err(),
            "signing someone else's group must fail at sign time"
        );
    }

    #[test]
    fn compute_group_rejects_oversized_membership() {
        let initiator = KeyPair::generate();
        let mut group = ComputeGroup::new(initiator.public_bytes(), "big", 1);
        group.members = vec![[0u8; PUBLIC_KEY_LENGTH]; COMPUTE_GROUP_MAX_MEMBERS + 1];
        // Sign-side cap.
        assert!(
            ComputeGroupEntry::sign(group.clone(), &initiator).is_err(),
            "sign must reject an over-capacity allowlist"
        );
        // Verify-side cap: forge an envelope around the oversized group and
        // confirm verify rejects it before hashing.
        let entry = ComputeGroupEntry {
            group,
            initiator: initiator.public_bytes(),
            signature: [0u8; SIGNATURE_BYTES],
        };
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn compute_group_rejects_oversized_group_id() {
        let initiator = KeyPair::generate();
        let group = ComputeGroup::new(
            initiator.public_bytes(),
            "x".repeat(COMPUTE_GROUP_ID_MAX + 1),
            1,
        );
        assert!(ComputeGroupEntry::sign(group, &initiator).is_err());
    }

    #[test]
    fn compute_group_domain_separated_from_node_directory() {
        // A compute-group signature must not collide with a node-directory
        // pre-image: the canonical byte strings differ by domain prefix.
        let initiator = KeyPair::generate();
        let group = sample_group(&initiator, &[]);
        let as_group = canonical_bytes(&group, DOMAIN_COMPUTE_GROUP_V1).unwrap();
        let as_other = canonical_bytes(&group, crate::canonical::DOMAIN_NODE_DIRECTORY_V1).unwrap();
        assert_ne!(
            as_group, as_other,
            "compute-group and node-directory domains must produce distinct byte strings"
        );
    }

    #[test]
    fn compute_group_json_roundtrips() {
        let initiator = KeyPair::generate();
        let w1 = KeyPair::generate();
        let entry = ComputeGroupEntry::sign(sample_group(&initiator, &[&w1]), &initiator).unwrap();
        let json = serde_json::to_vec(&entry).unwrap();
        let back: ComputeGroupEntry = serde_json::from_slice(&json).unwrap();
        assert_eq!(back, entry);
        back.verify_signature().unwrap();
    }
}
