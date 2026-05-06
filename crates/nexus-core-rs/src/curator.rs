// SPDX-License-Identifier: AGPL-3.0-or-later
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

use crate::canonical::{DOMAIN_CURATOR_LIST_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};
use crate::key_rotation::RevocationCache;

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

/// Per-field length caps on [`CuratorProjectRef`] strings.
///
/// Sprint 7 audit finding A-4: without these caps a single entry
/// could carry a 10 MB `description`, bypassing the
/// [`CURATOR_LIST_MAX_ENTRIES`] cap (which counts entries, not
/// bytes). With 256 entries at the caps below, the worst-case
/// serialized size stays well under 200 KB total, which keeps
/// the shell-daemon's `DashMap<curator, CuratorListEntry>` cache
/// and the `/curators` HTTP response bounded.
///
/// The values match the Sprint 8 plan:
/// `project_id <= 128`, `project_name <= 128`,
/// `category <= 64`, `description <= 280`.
pub const CURATOR_PROJECT_ID_MAX: usize = 128;
/// See [`CURATOR_PROJECT_ID_MAX`].
pub const CURATOR_PROJECT_NAME_MAX: usize = 128;
/// See [`CURATOR_PROJECT_ID_MAX`].
pub const CURATOR_CATEGORY_MAX: usize = 64;
/// See [`CURATOR_PROJECT_ID_MAX`].
pub const CURATOR_DESCRIPTION_MAX: usize = 280;

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
        check_entry_field_caps(&list.entries)?;
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
        check_entry_field_caps(&self.list.entries)?;
        if self.list.curator_pubkey != self.curator_pubkey {
            return Err(NexusError::Crypto(
                "list.curator_pubkey does not match envelope curator_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.list, DOMAIN_CURATOR_LIST_V1)?;
        crate::crypto::verify(&self.curator_pubkey, &bytes, &self.signature)
    }

    /// Verify signature and check the curator's key against the
    /// [`RevocationCache`]. If the key is fully revoked (transition
    /// window expired), the entry is rejected. If the key is in
    /// transition, verification succeeds but callers should log a
    /// warning.
    ///
    /// Returns `Ok(true)` if verification passed and the key is in
    /// transition (callers should warn), `Ok(false)` if passed
    /// cleanly (key not in cache or unknown), `Err` if revoked or
    /// signature invalid.
    pub fn verify_with_revocation(&self, cache: &RevocationCache, now_ts: u64) -> Result<bool> {
        if cache.is_revoked(&self.curator_pubkey, now_ts) {
            return Err(NexusError::Crypto(
                "curator key is fully revoked (transition window expired)".into(),
            ));
        }
        self.verify_signature()?;
        Ok(cache.is_in_transition(&self.curator_pubkey, now_ts))
    }

    /// Verify a [`CuratorListEntry`] under the Sprint 22 Couche 2
    /// governance-strong flag : every project entry whose
    /// `project_id` is enrolled in the contributor registry must
    /// have this curator's pubkey listed as a verified contributor.
    ///
    /// `registry` is queried per-entry. Projects for which
    /// [`ContributorRegistry::is_enrolled`] returns `false` are
    /// skipped — Couche 2 gating is opt-in per project, not a
    /// global requirement. Projects that are enrolled but whose
    /// curator pubkey is not a verified contributor cause
    /// verification to reject.
    ///
    /// Runs [`Self::verify_signature`] first ; the contributor
    /// check is strictly additive on top of the base signature
    /// validation.
    ///
    /// NOTE: Interim Sybil-resistance S22. Contributor selection
    /// is still biased toward high-kudos workers (Matthew effect
    /// one layer deeper). Post-v1.0 LT-1 Kudos-v2 reform will
    /// introduce log-utility + DRF + EMA trust to break this
    /// cycle. See:
    /// - `docs/FAIRNESS_VISION.md §7` "Design-conflict S22"
    /// - `docs/release/ROADMAP_COMMITMENTS.md §LT-1`
    pub fn verify_with_contributor_registry<R: ContributorRegistry>(
        &self,
        registry: &R,
    ) -> Result<()> {
        self.verify_signature()?;
        for (idx, entry) in self.list.entries.iter().enumerate() {
            if !registry.is_enrolled(&entry.project_id) {
                // Project not enrolled in governance-strong gate :
                // curator vouching is sufficient (the base-layer
                // signature check above).
                continue;
            }
            if !registry.is_verified_contributor(&entry.project_id, &self.curator_pubkey) {
                return Err(NexusError::Crypto(format!(
                    "curator list entry #{idx}: curator_pubkey is not a verified contributor \
                     for project_id={} under Couche 2 governance-strong flag",
                    entry.project_id
                )));
            }
        }
        Ok(())
    }
}

/// Couche 2 governance-strong verification hook. The shell-daemon
/// injects an implementation that proxies the coordinator's
/// `contributor_registry` SQLite table over HTTP.
///
/// Implementors are expected to cache query results where useful —
/// `verify_with_contributor_registry` calls `is_enrolled` +
/// `is_verified_contributor` per entry, so a large list can result
/// in several hundred lookups. The coordinator-side proxy is
/// loopback-only and synchronous ; the cost is in practice
/// bounded by [`CURATOR_LIST_MAX_ENTRIES`].
pub trait ContributorRegistry {
    /// Return `true` iff `project_id` opts into the Couche 2
    /// governance-strong gate (i.e. a curator list entry for this
    /// project triggers the contributor-attestation check).
    ///
    /// Projects that have not been enrolled by the publisher (via
    /// `SBFB.json` flag or coordinator ceremony — TBD S23+) are
    /// gated by the base-layer signature check only.
    fn is_enrolled(&self, project_id: &str) -> bool;

    /// Return `true` iff `curator_pubkey` is a verified contributor
    /// for `project_id` in the registry — i.e. the coordinator has
    /// previously signed at least one
    /// [`crate::attestations::ContributorAttestation`] for this
    /// pair.
    fn is_verified_contributor(
        &self,
        project_id: &str,
        curator_pubkey: &[u8; PUBLIC_KEY_LENGTH],
    ) -> bool;
}

/// Reject any [`CuratorProjectRef`] whose string fields exceed
/// the Sprint 8 A-4 byte caps. Byte length is used (not char
/// count) because the caps exist to bound the serialized blob
/// size in memory and on the wire.
fn check_entry_field_caps(entries: &[CuratorProjectRef]) -> Result<()> {
    for (idx, entry) in entries.iter().enumerate() {
        if entry.project_id.len() > CURATOR_PROJECT_ID_MAX {
            return Err(NexusError::Crypto(format!(
                "curator entry #{idx} project_id has {} bytes, exceeds CURATOR_PROJECT_ID_MAX={}",
                entry.project_id.len(),
                CURATOR_PROJECT_ID_MAX
            )));
        }
        if entry.project_name.len() > CURATOR_PROJECT_NAME_MAX {
            return Err(NexusError::Crypto(format!(
                "curator entry #{idx} project_name has {} bytes, exceeds CURATOR_PROJECT_NAME_MAX={}",
                entry.project_name.len(),
                CURATOR_PROJECT_NAME_MAX
            )));
        }
        if entry.category.len() > CURATOR_CATEGORY_MAX {
            return Err(NexusError::Crypto(format!(
                "curator entry #{idx} category has {} bytes, exceeds CURATOR_CATEGORY_MAX={}",
                entry.category.len(),
                CURATOR_CATEGORY_MAX
            )));
        }
        if entry.description.len() > CURATOR_DESCRIPTION_MAX {
            return Err(NexusError::Crypto(format!(
                "curator entry #{idx} description has {} bytes, exceeds CURATOR_DESCRIPTION_MAX={}",
                entry.description.len(),
                CURATOR_DESCRIPTION_MAX
            )));
        }
    }
    Ok(())
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

    #[test]
    fn verify_rejects_oversized_fields() {
        // Sprint 8 A-4 tightening: per-field string caps must
        // catch a single pathological entry that flooded one
        // string, even when the total entry count stays within
        // CURATOR_LIST_MAX_ENTRIES. Exercise every cap
        // independently so a future regression that drops one
        // check is caught by at least one of the four asserts.

        let kp = KeyPair::generate();

        // description over 280 bytes — the loudest failure mode
        // called out by the audit finding.
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].description = "x".repeat(CURATOR_DESCRIPTION_MAX + 1);
        assert!(CuratorListEntry::sign(list, &kp).is_err());

        // project_id over 128 bytes — keeping the pkarr node id
        // bound prevents a UTF-8 balloon through the key field.
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].project_id = "a".repeat(CURATOR_PROJECT_ID_MAX + 1);
        assert!(CuratorListEntry::sign(list, &kp).is_err());

        // project_name over 128 bytes.
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].project_name = "n".repeat(CURATOR_PROJECT_NAME_MAX + 1);
        assert!(CuratorListEntry::sign(list, &kp).is_err());

        // category over 64 bytes.
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].category = "c".repeat(CURATOR_CATEGORY_MAX + 1);
        assert!(CuratorListEntry::sign(list, &kp).is_err());

        // Defense in depth: a hand-signed envelope that bypasses
        // the sign-side cap must still be refused at verify time.
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].description = "y".repeat(CURATOR_DESCRIPTION_MAX + 1);
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
    fn cap_exactly_at_boundary_is_accepted() {
        // Off-by-one regression guard: exactly at each cap must
        // still sign + verify. The non-ascii boundary (`len()`
        // counts bytes, not code points) is exercised for
        // `description` which is the most likely field to carry
        // multibyte characters.
        let kp = KeyPair::generate();
        let mut list = sample_list(kp.public_bytes());
        list.entries[0].project_id = "a".repeat(CURATOR_PROJECT_ID_MAX);
        list.entries[0].project_name = "n".repeat(CURATOR_PROJECT_NAME_MAX);
        list.entries[0].category = "c".repeat(CURATOR_CATEGORY_MAX);
        list.entries[0].description = "d".repeat(CURATOR_DESCRIPTION_MAX);
        let entry = CuratorListEntry::sign(list, &kp).expect("cap boundary signs");
        entry.verify_signature().expect("cap boundary verifies");
    }

    /// Minimal in-memory [`ContributorRegistry`] stub used by the
    /// Couche 2 governance-strong gate tests. Real registry hook
    /// lives in `nexus-shell-daemon/src/http.rs` (proxy) and the
    /// Python `contributor_registry.py` (SQLite source of truth).
    struct StubRegistry {
        enrolled: std::collections::BTreeSet<String>,
        verified: std::collections::BTreeSet<(String, [u8; PUBLIC_KEY_LENGTH])>,
    }

    impl StubRegistry {
        fn new() -> Self {
            Self {
                enrolled: Default::default(),
                verified: Default::default(),
            }
        }
        fn enroll(mut self, project_id: &str) -> Self {
            self.enrolled.insert(project_id.to_string());
            self
        }
        fn verify(mut self, project_id: &str, curator: [u8; PUBLIC_KEY_LENGTH]) -> Self {
            self.verified.insert((project_id.to_string(), curator));
            self
        }
    }

    impl ContributorRegistry for StubRegistry {
        fn is_enrolled(&self, project_id: &str) -> bool {
            self.enrolled.contains(project_id)
        }
        fn is_verified_contributor(
            &self,
            project_id: &str,
            curator_pubkey: &[u8; PUBLIC_KEY_LENGTH],
        ) -> bool {
            self.verified
                .contains(&(project_id.to_string(), *curator_pubkey))
        }
    }

    #[test]
    fn verify_rejects_non_contributor_if_enforce() {
        // Enroll the project but do NOT register the curator as a
        // verified contributor — governance-strong gate must reject.
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let project_id = list.entries[0].project_id.clone();
        let entry = CuratorListEntry::sign(list, &kp).unwrap();

        let registry = StubRegistry::new().enroll(&project_id);
        let err = entry
            .verify_with_contributor_registry(&registry)
            .expect_err("non-contributor must reject under enforcement");
        match err {
            NexusError::Crypto(msg) => {
                assert!(msg.contains("not a verified contributor"), "msg: {msg}")
            }
            other => panic!("expected Crypto error, got {other:?}"),
        }
    }

    #[test]
    fn verify_admits_registered_contributor() {
        // Enroll + register : gate must admit.
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let project_a = list.entries[0].project_id.clone();
        let project_b = list.entries[1].project_id.clone();
        let entry = CuratorListEntry::sign(list, &kp).unwrap();

        let registry = StubRegistry::new()
            .enroll(&project_a)
            .verify(&project_a, kp.public_bytes())
            .enroll(&project_b)
            .verify(&project_b, kp.public_bytes());
        entry
            .verify_with_contributor_registry(&registry)
            .expect("fully registered curator admits");
    }

    #[test]
    fn verify_allows_non_enrolled_project_without_contributor_gate() {
        // Project not enrolled → gate inert ; base signature check
        // alone is sufficient. Matches the opt-in Couche 2 design.
        let kp = KeyPair::generate();
        let list = sample_list(kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &kp).unwrap();
        let registry = StubRegistry::new(); // nothing enrolled
        entry
            .verify_with_contributor_registry(&registry)
            .expect("unenrolled projects bypass Couche 2 gate");
    }

    // ---- Key rotation / revocation integration (Sprint 25 Phase B) ----

    #[test]
    fn curator_verify_with_revoked_key_rejects() {
        let curator_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let list = sample_list(curator_kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &curator_kp).unwrap();

        let mut cache = RevocationCache::new();
        let ann = crate::key_rotation::KeyRotationAnnouncement::new(
            curator_kp.public_bytes(),
            new_kp.public_bytes(),
            1_000_000,
            "compromised",
            7,
        )
        .unwrap();
        let signed = crate::key_rotation::SignedKeyRotation::sign(ann, &curator_kp).unwrap();
        cache.apply_announcement(&signed).unwrap();

        // After transition window (day 8): fully revoked
        let after = 1_000_000 + 8 * 86_400;
        let result = entry.verify_with_revocation(&cache, after);
        assert!(result.is_err());
    }

    #[test]
    fn curator_verify_with_transitioning_key_warns() {
        let curator_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let list = sample_list(curator_kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &curator_kp).unwrap();

        let mut cache = RevocationCache::new();
        let ann = crate::key_rotation::KeyRotationAnnouncement::new(
            curator_kp.public_bytes(),
            new_kp.public_bytes(),
            1_000_000,
            "planned rotation",
            7,
        )
        .unwrap();
        let signed = crate::key_rotation::SignedKeyRotation::sign(ann, &curator_kp).unwrap();
        cache.apply_announcement(&signed).unwrap();

        // During transition (day 3): accepted but returns true (warn)
        let during = 1_000_000 + 3 * 86_400;
        let in_transition = entry
            .verify_with_revocation(&cache, during)
            .expect("transitioning key must be accepted");
        assert!(in_transition, "must signal in-transition state");
    }

    #[test]
    fn curator_verify_with_clean_key_passes() {
        let curator_kp = KeyPair::generate();
        let list = sample_list(curator_kp.public_bytes());
        let entry = CuratorListEntry::sign(list, &curator_kp).unwrap();

        let cache = RevocationCache::new(); // empty
        let in_transition = entry
            .verify_with_revocation(&cache, 1_700_000_000)
            .expect("clean key must pass");
        assert!(!in_transition, "clean key must not signal transition");
    }
}
