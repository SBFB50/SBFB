//! Curator list domain types for SBFB.
//!
//! A **curator list** is a signed collection of project endpoints a
//! specific curator vouches for. Workers (via the
//! `nexus-shell-daemon`) subscribe to curator lists they trust,
//! fetch the signed blob, verify its signature, and surface the
//! entries on the React shell's "Browse" page — without any
//! central registry.
//!
//! ## Wire shape
//!
//! A [`CuratorList`] is the *unsigned* payload: the curator's
//! identity, a monotonic revision counter, a creation timestamp,
//! and a bounded list of [`CuratorProjectRef`] entries.
//!
//! A [`CuratorListEntry`] wraps the payload together with:
//!
//! 1. A redundant `curator_pubkey` field that MUST equal
//!    `list.curator_pubkey`. This catches a specific attribution
//!    split-brain bug where a forwarder attaches a different pubkey
//!    to the envelope than the one inside the payload — the
//!    Sprint 2 audit found and fixed the equivalent bug in
//!    [`crate::task::ClaimEntry`] and the same pattern applies here.
//! 2. An Ed25519 signature over
//!    [`crate::canonical::canonical_bytes`] of the list with
//!    [`crate::canonical::DOMAIN_CURATOR_LIST_V1`] as the domain
//!    separation tag.
//!
//! ## DoS mitigation (Sprint 7 plan R5)
//!
//! [`CURATOR_LIST_MAX_ENTRIES`] bounds how many projects a single
//! signed list can announce. A malicious curator publishing a list
//! with 1M entries would balloon the shell-daemon's RAM cache and
//! flood gossip; the verifier refuses any list whose
//! `entries.len()` exceeds the cap. The cap is conservative (256)
//! so early-access curator lists comfortably fit while the attack
//! surface stays bounded.
//!
//! ## Revision rollback protection (Sprint 7 plan R6)
//!
//! The `revision` field is a monotonic counter. The shell-daemon
//! Phase C runtime keeps a `DashMap<curator_pubkey, latest_entry>`
//! and refuses to overwrite an entry unless the new revision is
//! **strictly greater** than the stored one. This is enforced in
//! the runtime, not here — this module is the crypto layer only.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{canonical_bytes, DOMAIN_CURATOR_LIST_V1};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Current on-wire version for CuratorList payloads.
///
/// Bump this when the canonical serialization format changes in a
/// way that breaks signature compatibility. Consumers refuse
/// entries with a version they don't understand.
pub const CURATOR_LIST_FORMAT_VERSION: u16 = 1;

/// Hard upper bound on the number of entries a single signed
/// curator list may carry. Exists to cap the DoS impact of a
/// malicious curator publishing a pathologically large list
/// (Sprint 7 plan R5). 256 is well above the realistic early-
/// access curator count and well below any RAM / gossip pain
/// threshold on the receiving shell-daemon.
pub const CURATOR_LIST_MAX_ENTRIES: usize = 256;

// =================================================================
// Types
// =================================================================

/// The unsigned curator list payload.
///
/// Signed via [`CuratorListEntry::sign`] and verified via
/// [`CuratorListEntry::verify_signature`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuratorList {
    /// Canonical format version. Must equal
    /// [`CURATOR_LIST_FORMAT_VERSION`] to be accepted.
    pub version: u16,

    /// The curator's Ed25519 public key (32 bytes). This is the
    /// only network-visible curator identity; display names go in
    /// [`CuratorList::curator_name`].
    pub curator_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Human-readable curator name (display only). Kept on the
    /// payload rather than in a sidecar so a single verified blob
    /// is enough for the shell to render a card.
    pub curator_name: String,

    /// Unix timestamp (seconds since epoch) when this revision was
    /// published. Used only for display; rollback protection is
    /// driven by [`CuratorList::revision`].
    pub created_at: u64,

    /// Monotonic revision counter. The shell-daemon Phase C
    /// runtime refuses to overwrite a stored list unless the new
    /// `revision` is strictly greater than the stored one (Sprint
    /// 7 plan R6). A curator bumping to a lower value is treated
    /// as a rollback attempt and ignored.
    pub revision: u64,

    /// The vouched-for projects. MUST have
    /// `entries.len() <= CURATOR_LIST_MAX_ENTRIES` — verifiers
    /// reject any list that exceeds the cap.
    pub entries: Vec<CuratorProjectRef>,
}

/// A single entry in a [`CuratorList`]: one project the curator
/// vouches for.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuratorProjectRef {
    /// The project's pkarr node id, lowercase hex (64 chars).
    /// This is the handle Phase D uses to probe reachability.
    pub project_id: String,

    /// Human-readable project name (display only).
    pub project_name: String,

    /// Free-form category tag (`"gov"`, `"coldcase"`,
    /// `"forensics"`, `"misc"`, …). The shell groups browse
    /// results by category.
    pub category: String,

    /// Short description shown on the shell's browse cards.
    /// Bounded in length to keep a single list well under 200 KB
    /// total — 280 chars matches the plan D3 freeze.
    pub description: String,
}

impl CuratorList {
    /// Construct a new [`CuratorList`] with the current format
    /// version and an empty entry set. Fields can be mutated
    /// directly after construction.
    pub fn new(
        curator_pubkey: [u8; PUBLIC_KEY_LENGTH],
        curator_name: impl Into<String>,
        created_at: u64,
        revision: u64,
    ) -> Self {
        CuratorList {
            version: CURATOR_LIST_FORMAT_VERSION,
            curator_pubkey,
            curator_name: curator_name.into(),
            created_at,
            revision,
            entries: Vec::new(),
        }
    }
}

/// A signed [`CuratorList`], ready to be broadcast over gossip.
///
/// The signature is computed over
/// [`canonical_bytes`] of the inner [`CuratorList`] with
/// [`DOMAIN_CURATOR_LIST_V1`] as the domain tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CuratorListEntry {
    /// The list itself.
    pub list: CuratorList,

    /// Redundant Ed25519 pubkey of the signing curator. MUST
    /// equal [`CuratorList::curator_pubkey`]; the verifier rejects
    /// any entry where the two disagree (attribution split-brain
    /// mitigation, same pattern as
    /// [`crate::task::ClaimEntry::verify_signature`]).
    pub curator_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of
    /// [`Self::list`] (64 bytes; `serde_big_array` because serde
    /// does not derive for arrays > 32).
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl CuratorListEntry {
    /// Sign a [`CuratorList`] with the given keypair.
    ///
    /// Validates two invariants before signing:
    ///
    /// 1. `list.curator_pubkey == keypair.public_bytes()` — the
    ///    caller is signing their own list. Mismatches return
    ///    [`NexusError::Crypto`] so the typo surfaces immediately.
    /// 2. `list.entries.len() <= CURATOR_LIST_MAX_ENTRIES` — the
    ///    Sprint 7 R5 DoS cap applies to signing as well as
    ///    verification so a curator cannot accidentally produce a
    ///    list their own subscribers will reject.
    pub fn sign(list: CuratorList, keypair: &KeyPair) -> Result<Self> {
        if list.curator_pubkey != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "list.curator_pubkey does not match signing keypair".into(),
            ));
        }
        if list.entries.len() > CURATOR_LIST_MAX_ENTRIES {
            return Err(NexusError::Crypto(format!(
                "curator list has {} entries, exceeds CURATOR_LIST_MAX_ENTRIES={}",
                list.entries.len(),
                CURATOR_LIST_MAX_ENTRIES
            )));
        }
        let bytes = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(CuratorListEntry {
            list,
            curator_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify a [`CuratorListEntry`].
    ///
    /// Five checks, in order:
    ///
    /// 1. `list.version == CURATOR_LIST_FORMAT_VERSION` — reject
    ///    payloads the current build does not understand.
    /// 2. `list.entries.len() <= CURATOR_LIST_MAX_ENTRIES` —
    ///    enforce the Sprint 7 R5 DoS cap before allocating or
    ///    hashing any further.
    /// 3. `list.curator_pubkey == self.curator_pubkey` —
    ///    attribution consistency.
    /// 4. Ed25519 signature is valid over the canonical bytes of
    ///    `list` with [`DOMAIN_CURATOR_LIST_V1`].
    /// 5. No extra check — the signature alone does not guarantee
    ///    the list is "latest"; revision rollback protection is
    ///    enforced at the `DashMap<pubkey, latest>` layer by the
    ///    shell-daemon runtime, not here.
    pub fn verify_signature(&self) -> Result<()> {
        if self.list.version != CURATOR_LIST_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "curator list version mismatch (got {}, expected {})",
                self.list.version, CURATOR_LIST_FORMAT_VERSION
            )));
        }
        if self.list.entries.len() > CURATOR_LIST_MAX_ENTRIES {
            return Err(NexusError::Crypto(format!(
                "curator list has {} entries, exceeds CURATOR_LIST_MAX_ENTRIES={}",
                self.list.entries.len(),
                CURATOR_LIST_MAX_ENTRIES
            )));
        }
        if self.list.curator_pubkey != self.curator_pubkey {
            return Err(NexusError::Crypto(
                "list.curator_pubkey does not match envelope curator_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.list, DOMAIN_CURATOR_LIST_V1)?;
        crate::crypto::verify(&self.curator_pubkey, &bytes, &self.signature)
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::DOMAIN_TASK_V1;

    fn sample_list(curator_pubkey: [u8; PUBLIC_KEY_LENGTH]) -> CuratorList {
        let mut list = CuratorList::new(curator_pubkey, "FlowUP Curation", 1_712_345_678, 1);
        list.entries.push(CuratorProjectRef {
            project_id: "a".repeat(64),
            project_name: "gov".into(),
            category: "gov".into(),
            description: "Signal processing and intelligence tooling".into(),
        });
        list.entries.push(CuratorProjectRef {
            project_id: "b".repeat(64),
            project_name: "coldcase".into(),
            category: "investigation".into(),
            description: "Cold case investigation toolkit".into(),
        });
        list
    }

    #[test]
    fn new_list_has_format_version_and_empty_entries() {
        let kp = KeyPair::generate();
        let list = CuratorList::new(kp.public_bytes(), "FlowUP", 0, 0);
        assert_eq!(list.version, CURATOR_LIST_FORMAT_VERSION);
        assert!(list.entries.is_empty());
    }

    #[test]
    fn entry_sign_and_verify() {
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &kp).expect("sign");
        entry
            .verify_signature()
            .expect("signature must verify for a well-formed entry");
        assert_eq!(entry.curator_pubkey, kp.public_bytes());
    }

    #[test]
    fn sign_rejects_mismatched_pubkey_in_payload() {
        // The caller passed a list whose curator_pubkey belongs
        // to a different key than the one they are signing with.
        // This is almost always a bug in the caller; surface it
        // immediately.
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        let list = sample_list(kp_b.public_bytes());
        assert!(CuratorListEntry::sign(list, &kp_a).is_err());
    }

    #[test]
    fn verify_rejects_tampered_payload() {
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let mut entry = CuratorListEntry::sign(list, &kp).unwrap();
        entry.list.entries[0].project_name = "TAMPERED".into();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn verify_rejects_attribution_mismatch() {
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        let list = sample_list(kp_a.public_bytes());
        let mut entry = CuratorListEntry::sign(list, &kp_a).unwrap();
        // Envelope pubkey is flipped but the payload still
        // carries kp_a's pubkey — the attribution check must
        // fail before the signature check gets a chance.
        entry.curator_pubkey = kp_b.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn verify_rejects_wrong_signer() {
        // Even a coordinated attribution swap (envelope + payload
        // both point at the impostor) must still fail the raw
        // signature check, because the signature was produced by
        // the real key.
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let list = sample_list(real.public_bytes());
        let mut entry = CuratorListEntry::sign(list, &real).unwrap();
        entry.curator_pubkey = impostor.public_bytes();
        entry.list.curator_pubkey = impostor.public_bytes();
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn verify_rejects_unknown_version() {
        let kp = KeyPair::generate();
        let mut list = sample_list(kp.public_bytes());
        list.version = 99; // future / unknown
                           // Re-sign with the bumped version so the signature is
                           // internally consistent — verify must still refuse based
                           // on the version field alone.
        let bytes = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1).unwrap();
        let signature = kp.sign(&bytes);
        let entry = CuratorListEntry {
            list,
            curator_pubkey: kp.public_bytes(),
            signature,
        };
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn sign_rejects_oversized_entries() {
        let kp = KeyPair::generate();
        let mut list = CuratorList::new(kp.public_bytes(), "flood", 0, 0);
        for i in 0..(CURATOR_LIST_MAX_ENTRIES + 1) {
            list.entries.push(CuratorProjectRef {
                project_id: format!("{:064}", i),
                project_name: format!("p{i}"),
                category: "misc".into(),
                description: String::new(),
            });
        }
        // sign must refuse before producing any bytes.
        assert!(CuratorListEntry::sign(list, &kp).is_err());
    }

    #[test]
    fn verify_rejects_oversized_entries() {
        // An attacker could hand-craft an envelope whose inner
        // list carries > CURATOR_LIST_MAX_ENTRIES entries — verify
        // must still refuse, even if the signature covers all of
        // them.
        let kp = KeyPair::generate();
        let mut list = CuratorList::new(kp.public_bytes(), "flood", 0, 0);
        for i in 0..(CURATOR_LIST_MAX_ENTRIES + 1) {
            list.entries.push(CuratorProjectRef {
                project_id: format!("{:064}", i),
                project_name: format!("p{i}"),
                category: "misc".into(),
                description: String::new(),
            });
        }
        // Bypass the sign-side cap by hand-signing the canonical
        // bytes directly.
        let bytes = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1).unwrap();
        let signature = kp.sign(&bytes);
        let entry = CuratorListEntry {
            list,
            curator_pubkey: kp.public_bytes(),
            signature,
        };
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn cap_boundary_is_accepted() {
        // Exactly CURATOR_LIST_MAX_ENTRIES entries must pass.
        // Regression against an off-by-one in the `>` vs `>=`
        // guard.
        let kp = KeyPair::generate();
        let mut list = CuratorList::new(kp.public_bytes(), "flood", 0, 0);
        for i in 0..CURATOR_LIST_MAX_ENTRIES {
            list.entries.push(CuratorProjectRef {
                project_id: format!("{:064}", i),
                project_name: format!("p{i}"),
                category: "misc".into(),
                description: String::new(),
            });
        }
        let entry = CuratorListEntry::sign(list, &kp).expect("cap boundary should pass");
        entry.verify_signature().expect("verify at cap boundary");
    }

    #[test]
    fn domain_separation_between_curator_and_task() {
        // A signature valid over the curator canonical bytes must
        // not be valid if re-interpreted under another domain tag.
        // Regression against cross-type replay.
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let curator_bytes = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1).unwrap();
        let task_bytes = canonical_bytes(&list, DOMAIN_TASK_V1).unwrap();
        assert_ne!(
            curator_bytes, task_bytes,
            "domain separation must yield distinct byte strings"
        );
    }

    #[test]
    fn roundtrip_through_json() {
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &kp).unwrap();
        let json = serde_json::to_string(&entry).unwrap();
        let back: CuratorListEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back, entry);
        back.verify_signature()
            .expect("round-tripped entry must still verify");
    }

    #[test]
    fn canonical_bytes_are_deterministic_for_same_list() {
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let a = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1).unwrap();
        let b = canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1).unwrap();
        assert_eq!(a, b);
    }
}
