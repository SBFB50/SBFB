// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node directory domain types for SBFB PULL discovery (Sprint 75).
//!
//! A **node directory** is a signed, self-published catalog of the
//! apps a single node hosts (or seeds): "here is what you can pull
//! from me". It is the read-side substrate of the PULL discovery
//! pivot — Browse becomes *list of nodes → a node's catalogue →
//! download*, instead of the PUSH-ephemeral PoW-gated announcement
//! that expired after 30 minutes and left older apps invisible to
//! fresh peers.
//!
//! ## Why a sibling type, not an overloaded curator list
//!
//! A [`crate::curator::CuratorList`] vouches for OTHER projects: its
//! [`crate::curator::CuratorProjectRef`] has no `archive_hash` and
//! conflates `project_id == node_id`. A node advertising ITS OWN
//! catalog needs the per-app BLAKE3 `archive_hash` (the content
//! address a puller fetches + integrity-checks) and a single durable
//! `node_id` (the dialable handle). Every mature decentralized system
//! gives self-publication its own signed type rather than overloading
//! a curation primitive — NIP-65 (`kind:10002`), Radicle's INVENTORY
//! announcement, F-Droid's per-repo index, BEP-44 mutable items.
//! [`NodeDirectory`] follows that prior art while reusing the
//! CuratorList crypto/cap/revision machinery *verbatim*.
//!
//! ## Wire shape
//!
//! A [`NodeDirectory`] is the *unsigned* payload: the node's identity
//! (`node_id`), a monotonic `revision` counter, and a bounded list of
//! [`CatalogApp`] entries.
//!
//! A [`NodeDirectoryEntry`] wraps the payload together with:
//!
//! 1. A redundant `node_id` field that MUST equal `directory.node_id`.
//!    This catches the attribution split-brain bug where a forwarder
//!    staples a different pubkey to the envelope than the one inside
//!    the payload — the same mitigation
//!    [`crate::curator::CuratorListEntry`] and
//!    [`crate::seed::SeedRequestEnvelope`] already apply.
//! 2. An Ed25519 signature over [`crate::canonical::canonical_bytes`]
//!    of the directory with
//!    [`crate::canonical::DOMAIN_NODE_DIRECTORY_V1`] as the domain tag.
//!
//! ## Provenance invariant (verrou 4 — seeder != author)
//!
//! The `node_id` that signs a directory is the AUTHOR of that catalog
//! listing. Each [`CatalogApp`] carries the *author's* `archive_hash`
//! (content-addressed BLAKE3); a node merely re-advertising or seeding
//! someone else's app never signs that app's provenance. The directory
//! signature attests "I, this node, claim to host these hashes" — not
//! "I authored these apps". BLAKE3 content-addressing remains the truth
//! of joinability: a forged catalog can over-claim but can never serve
//! bytes it does not hold (a puller verifies the hash on fetch).
//!
//! ## DoS mitigation
//!
//! [`NODE_DIRECTORY_MAX_ENTRIES`] bounds the catalog size and the
//! per-field caps bound each string — identical posture to the curator
//! list (Sprint 7 R5/A-4), enforced at BOTH sign and verify so a node
//! cannot accidentally produce a directory its own subscribers reject.
//!
//! ## Revision rollback protection
//!
//! The `revision` field is a monotonic counter. The shell-daemon
//! ingest path refuses to overwrite a stored directory unless the new
//! `revision` is **strictly greater** than the stored one. This is
//! enforced in the runtime via the shared
//! `verify_signed_list_ingest` gate, not here — this module is the
//! crypto layer only.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_NODE_DIRECTORY_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Current on-wire version for [`NodeDirectory`] payloads.
///
/// Independent from every existing `*_FORMAT_VERSION`: this is a
/// brand-new signed type, so introducing it bumps nothing (pre-launch
/// additive policy, the S74 `DOMAIN_SEED_REQUEST_V1` pattern).
/// Consumers refuse entries with a version they do not understand.
pub const NODE_DIRECTORY_FORMAT_VERSION: u16 = 1;

/// Hard upper bound on the number of apps a single signed node
/// directory may carry. Caps the DoS impact of a node publishing a
/// pathologically large catalog (mirrors
/// [`crate::curator::CURATOR_LIST_MAX_ENTRIES`]). 256 is well above a
/// realistic per-node app count and well below any RAM / gossip pain
/// threshold on the receiving daemon.
pub const NODE_DIRECTORY_MAX_ENTRIES: usize = 256;

/// Per-field byte cap on [`CatalogApp::project_id`]. Matches
/// [`crate::curator::CURATOR_PROJECT_ID_MAX`].
pub const NODE_DIRECTORY_PROJECT_ID_MAX: usize = 128;
/// Per-field byte cap on [`CatalogApp::archive_hash`]. A BLAKE3 hash is
/// 32 bytes → 64 lowercase hex chars; the cap is exactly that so a
/// well-formed hash fits and a flooded over-long string is rejected.
pub const NODE_DIRECTORY_ARCHIVE_HASH_MAX: usize = 64;
/// Per-field byte cap on [`CatalogApp::project_name`]. Matches
/// [`crate::curator::CURATOR_PROJECT_NAME_MAX`].
pub const NODE_DIRECTORY_PROJECT_NAME_MAX: usize = 128;
/// Per-field byte cap on [`CatalogApp::category`]. Matches
/// [`crate::curator::CURATOR_CATEGORY_MAX`].
pub const NODE_DIRECTORY_CATEGORY_MAX: usize = 64;
/// Per-field byte cap on [`CatalogApp::description`]. Matches
/// [`crate::curator::CURATOR_DESCRIPTION_MAX`].
pub const NODE_DIRECTORY_DESCRIPTION_MAX: usize = 280;

// =================================================================
// Types
// =================================================================

/// One app a node advertises in its [`NodeDirectory`].
///
/// The display fields (`project_name`, `category`, `description`) are
/// named to mirror [`crate::curator::CuratorProjectRef`] and the
/// daemon-side `BrowseEntry`, so the aggregator maps a catalog entry
/// to a browse card with no field renaming seam.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogApp {
    /// Per-app identity hex — `blake3(project_name)`, the same id the
    /// feed, deploy, and `BrowseEntry` already use. One node hosts many
    /// apps, so this is NOT the node id.
    pub project_id: String,

    /// Lowercase hex (64 chars) BLAKE3 hash of the app's zip archive —
    /// the exact content a puller fetches + integrity-checks. The
    /// dialable transport address is NOT stored here (a stored ticket
    /// would freeze a stale `EndpointAddr`, the bug Phase A fixed); the
    /// puller downloads the bare hash directly from the publishing
    /// node_id (plus any best-effort seeders) at pull time — no ticket
    /// involved, pkarr resolves the bare endpoint id
    /// (`BlobsClient::fetch_hash_multi`, Sprint 75 Phase D). Empty only
    /// for a directory entry with no archive (e.g. a private
    /// placeholder), which a puller skips.
    pub archive_hash: String,

    /// Human-readable app name (display only).
    pub project_name: String,

    /// Free-form category tag (`"gov"`, `"investigation"`, `"misc"`,
    /// …). The shell groups catalog cards by category.
    pub category: String,

    /// Short description shown on the shell's catalog cards. Bounded by
    /// [`NODE_DIRECTORY_DESCRIPTION_MAX`].
    pub description: String,
}

/// The unsigned node directory payload.
///
/// Signed via [`NodeDirectoryEntry::sign`] and verified via
/// [`NodeDirectoryEntry::verify_signature`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeDirectory {
    /// Canonical format version. Must equal
    /// [`NODE_DIRECTORY_FORMAT_VERSION`] to be accepted.
    pub version: u16,

    /// The publishing node's Ed25519 public key (32 bytes). This is
    /// both the dialable node identity a puller uses to fetch the
    /// catalog's apps AND the signing identity — they are the same key
    /// on a real install (the daemon's long-lived keypair equals its
    /// iroh secret).
    pub node_id: [u8; PUBLIC_KEY_LENGTH],

    /// Monotonic revision counter. The shell-daemon ingest path refuses
    /// to overwrite a stored directory unless the new `revision` is
    /// strictly greater than the stored one. A node bumping to a lower
    /// value is treated as a rollback attempt and ignored.
    pub revision: u64,

    /// The advertised apps. MUST have
    /// `catalog.len() <= NODE_DIRECTORY_MAX_ENTRIES` — verifiers reject
    /// any directory that exceeds the cap.
    pub catalog: Vec<CatalogApp>,
}

impl NodeDirectory {
    /// Construct a new [`NodeDirectory`] at the current format version
    /// with an empty catalog. Push [`CatalogApp`] entries before
    /// signing.
    pub fn new(node_id: [u8; PUBLIC_KEY_LENGTH], revision: u64) -> Self {
        NodeDirectory {
            version: NODE_DIRECTORY_FORMAT_VERSION,
            node_id,
            revision,
            catalog: Vec::new(),
        }
    }
}

/// A signed [`NodeDirectory`], ready to be stored as a blob and
/// announced over gossip.
///
/// The signature is computed over [`canonical_bytes`] of the inner
/// [`NodeDirectory`] with [`DOMAIN_NODE_DIRECTORY_V1`] as the domain
/// tag.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeDirectoryEntry {
    /// The directory itself.
    pub directory: NodeDirectory,

    /// Redundant Ed25519 pubkey of the signing node. MUST equal
    /// [`NodeDirectory::node_id`]; the verifier rejects any entry where
    /// the two disagree (attribution split-brain mitigation, same
    /// pattern as [`crate::curator::CuratorListEntry`] /
    /// [`crate::seed::SeedRequestEnvelope`]).
    pub node_id: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature over the canonical bytes of [`Self::directory`]
    /// (64 bytes; `serde_big_array` because serde does not derive for
    /// arrays > 32).
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl NodeDirectoryEntry {
    /// Sign a [`NodeDirectory`] with the given node keypair.
    ///
    /// Validates three invariants before signing:
    ///
    /// 1. `directory.node_id == keypair.public_bytes()` — the caller is
    ///    signing their own directory. Mismatches return
    ///    [`NexusError::Crypto`] so the typo surfaces immediately.
    /// 2. `directory.catalog.len() <= NODE_DIRECTORY_MAX_ENTRIES` — the
    ///    DoS cap applies to signing as well as verification so a node
    ///    cannot accidentally produce a directory its own subscribers
    ///    reject.
    /// 3. Each [`CatalogApp`] string field is within its per-field cap.
    pub fn sign(directory: NodeDirectory, keypair: &KeyPair) -> Result<Self> {
        if directory.node_id != keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "directory.node_id does not match signing keypair".into(),
            ));
        }
        if directory.catalog.len() > NODE_DIRECTORY_MAX_ENTRIES {
            return Err(NexusError::Crypto(format!(
                "node directory has {} entries, exceeds NODE_DIRECTORY_MAX_ENTRIES={}",
                directory.catalog.len(),
                NODE_DIRECTORY_MAX_ENTRIES
            )));
        }
        check_catalog_field_caps(&directory.catalog)?;
        let bytes = canonical_bytes(&directory, DOMAIN_NODE_DIRECTORY_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(NodeDirectoryEntry {
            directory,
            node_id: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify a [`NodeDirectoryEntry`].
    ///
    /// Checks, in order:
    ///
    /// 1. `directory.version == NODE_DIRECTORY_FORMAT_VERSION` — reject
    ///    payloads the current build does not understand.
    /// 2. `directory.catalog.len() <= NODE_DIRECTORY_MAX_ENTRIES` —
    ///    enforce the DoS cap before hashing.
    /// 3. Each [`CatalogApp`] string field is within its per-field cap.
    /// 4. `directory.node_id == self.node_id` — attribution
    ///    consistency.
    /// 5. Ed25519 signature is valid over the canonical bytes of
    ///    `directory` with [`DOMAIN_NODE_DIRECTORY_V1`].
    ///
    /// Revision rollback protection is NOT checked here — it is a
    /// stateful property enforced at the ingest/storage layer.
    pub fn verify_signature(&self) -> Result<()> {
        if self.directory.version != NODE_DIRECTORY_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "node directory version mismatch (got {}, expected {})",
                self.directory.version, NODE_DIRECTORY_FORMAT_VERSION
            )));
        }
        if self.directory.catalog.len() > NODE_DIRECTORY_MAX_ENTRIES {
            return Err(NexusError::Crypto(format!(
                "node directory has {} entries, exceeds NODE_DIRECTORY_MAX_ENTRIES={}",
                self.directory.catalog.len(),
                NODE_DIRECTORY_MAX_ENTRIES
            )));
        }
        check_catalog_field_caps(&self.directory.catalog)?;
        if self.directory.node_id != self.node_id {
            return Err(NexusError::Crypto(
                "directory.node_id does not match envelope node_id".into(),
            ));
        }
        let bytes = canonical_bytes(&self.directory, DOMAIN_NODE_DIRECTORY_V1)?;
        crate::crypto::verify(&self.node_id, &bytes, &self.signature)
    }
}

/// Reject any [`CatalogApp`] whose string fields exceed the per-field
/// byte caps. Byte length is used (not char count) because the caps
/// exist to bound the serialized blob size in memory and on the wire.
///
/// Whether `s` is a valid [`CatalogApp::archive_hash`]: either empty (a
/// non-pullable placeholder a puller skips) or exactly
/// [`NODE_DIRECTORY_ARCHIVE_HASH_MAX`] (64) lowercase hex chars — a real BLAKE3
/// archive hash. Enforced at sign AND verify so a directory can never advertise
/// a malformed, unfetchable content address. The authoring route skips entries
/// that fail this rather than truncating the hash (truncating a content address
/// yields a *different*, unfetchable hash, not a clamped display string).
pub fn is_valid_archive_hash(s: &str) -> bool {
    s.is_empty()
        || (s.len() == NODE_DIRECTORY_ARCHIVE_HASH_MAX
            && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
}

fn check_catalog_field_caps(catalog: &[CatalogApp]) -> Result<()> {
    for (idx, app) in catalog.iter().enumerate() {
        if app.project_id.len() > NODE_DIRECTORY_PROJECT_ID_MAX {
            return Err(NexusError::Crypto(format!(
                "catalog app #{idx} project_id has {} bytes, exceeds NODE_DIRECTORY_PROJECT_ID_MAX={}",
                app.project_id.len(),
                NODE_DIRECTORY_PROJECT_ID_MAX
            )));
        }
        if !is_valid_archive_hash(&app.archive_hash) {
            return Err(NexusError::Crypto(format!(
                "catalog app #{idx} archive_hash must be empty or exactly \
                 {NODE_DIRECTORY_ARCHIVE_HASH_MAX} lowercase hex chars (a BLAKE3 hash); got {} bytes",
                app.archive_hash.len()
            )));
        }
        if app.project_name.len() > NODE_DIRECTORY_PROJECT_NAME_MAX {
            return Err(NexusError::Crypto(format!(
                "catalog app #{idx} project_name has {} bytes, exceeds NODE_DIRECTORY_PROJECT_NAME_MAX={}",
                app.project_name.len(),
                NODE_DIRECTORY_PROJECT_NAME_MAX
            )));
        }
        if app.category.len() > NODE_DIRECTORY_CATEGORY_MAX {
            return Err(NexusError::Crypto(format!(
                "catalog app #{idx} category has {} bytes, exceeds NODE_DIRECTORY_CATEGORY_MAX={}",
                app.category.len(),
                NODE_DIRECTORY_CATEGORY_MAX
            )));
        }
        if app.description.len() > NODE_DIRECTORY_DESCRIPTION_MAX {
            return Err(NexusError::Crypto(format!(
                "catalog app #{idx} description has {} bytes, exceeds NODE_DIRECTORY_DESCRIPTION_MAX={}",
                app.description.len(),
                NODE_DIRECTORY_DESCRIPTION_MAX
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
    use crate::canonical::DOMAIN_CURATOR_LIST_V1;

    fn sample_app(seed: char) -> CatalogApp {
        CatalogApp {
            project_id: seed.to_string().repeat(64),
            archive_hash: "a".repeat(64),
            project_name: "Babel".into(),
            category: "translation".into(),
            description: "Community translation protocol".into(),
        }
    }

    fn sample_directory(node_id: [u8; PUBLIC_KEY_LENGTH]) -> NodeDirectory {
        let mut dir = NodeDirectory::new(node_id, 1);
        dir.catalog.push(sample_app('a'));
        dir.catalog.push(sample_app('b'));
        dir
    }

    #[test]
    fn new_directory_has_format_version_and_empty_catalog() {
        let kp = KeyPair::generate();
        let dir = NodeDirectory::new(kp.public_bytes(), 0);
        assert_eq!(dir.version, NODE_DIRECTORY_FORMAT_VERSION);
        assert!(dir.catalog.is_empty());
    }

    #[test]
    fn node_directory_sign_verify_roundtrip() {
        let kp = KeyPair::generate();
        let dir = sample_directory(kp.public_bytes());
        let entry = NodeDirectoryEntry::sign(dir, &kp).expect("sign");
        entry
            .verify_signature()
            .expect("a well-formed entry must verify");
        assert_eq!(entry.node_id, kp.public_bytes());
        assert_eq!(entry.directory.version, NODE_DIRECTORY_FORMAT_VERSION);
    }

    #[test]
    fn sign_rejects_mismatched_node_id() {
        // The caller passed a directory whose node_id belongs to a
        // different key than the one they are signing with.
        let kp = KeyPair::generate();
        let other = KeyPair::generate();
        let dir = sample_directory(other.public_bytes());
        assert!(NodeDirectoryEntry::sign(dir, &kp).is_err());
    }

    #[test]
    fn verify_rejects_tampered_payload() {
        let kp = KeyPair::generate();
        let mut entry = NodeDirectoryEntry::sign(sample_directory(kp.public_bytes()), &kp).unwrap();
        entry.directory.catalog[0].archive_hash = "f".repeat(64);
        assert!(
            entry.verify_signature().is_err(),
            "tampering with the signed payload must fail verification"
        );
    }

    #[test]
    fn verify_rejects_tampered_signature() {
        let kp = KeyPair::generate();
        let mut entry = NodeDirectoryEntry::sign(sample_directory(kp.public_bytes()), &kp).unwrap();
        entry.signature[0] ^= 0xFF;
        assert!(entry.verify_signature().is_err());
    }

    #[test]
    fn verify_rejects_attribution_mismatch() {
        // A valid signature over the payload, but the envelope node_id
        // disagrees with the inner directory.node_id.
        let kp = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut entry = NodeDirectoryEntry::sign(sample_directory(kp.public_bytes()), &kp).unwrap();
        entry.node_id = impostor.public_bytes();
        assert!(
            entry.verify_signature().is_err(),
            "directory.node_id != envelope node_id must be rejected"
        );
    }

    #[test]
    fn node_directory_caps_enforced() {
        let kp = KeyPair::generate();
        // Exactly the cap must pass.
        let mut dir = NodeDirectory::new(kp.public_bytes(), 1);
        for _ in 0..NODE_DIRECTORY_MAX_ENTRIES {
            dir.catalog.push(sample_app('a'));
        }
        let entry = NodeDirectoryEntry::sign(dir, &kp).expect("cap boundary should pass");
        entry.verify_signature().expect("verify at cap boundary");

        // One over the cap must fail at sign time.
        let mut over = NodeDirectory::new(kp.public_bytes(), 1);
        for _ in 0..(NODE_DIRECTORY_MAX_ENTRIES + 1) {
            over.catalog.push(sample_app('a'));
        }
        assert!(NodeDirectoryEntry::sign(over, &kp).is_err());
    }

    #[test]
    fn archive_hash_cap_enforced() {
        let kp = KeyPair::generate();
        let mut dir = NodeDirectory::new(kp.public_bytes(), 1);
        let mut app = sample_app('a');
        app.archive_hash = "a".repeat(NODE_DIRECTORY_ARCHIVE_HASH_MAX + 1);
        dir.catalog.push(app);
        assert!(
            NodeDirectoryEntry::sign(dir, &kp).is_err(),
            "an over-long archive_hash must be rejected by the per-field cap"
        );
    }

    #[test]
    fn archive_hash_format_enforced() {
        // archive_hash is a content address, not a display string: it must be
        // empty or exactly 64 LOWERCASE hex chars. A malformed hash is rejected
        // at sign AND verify (a truncated/junk hash is unfetchable).
        let kp = KeyPair::generate();
        let signed_with = |hash: &str| {
            let mut app = sample_app('a');
            app.archive_hash = hash.to_string();
            let mut dir = NodeDirectory::new(kp.public_bytes(), 1);
            dir.catalog.push(app);
            NodeDirectoryEntry::sign(dir, &kp)
        };
        assert!(
            signed_with(&"a".repeat(64)).is_ok(),
            "valid 64 lowercase hex"
        );
        assert!(signed_with("").is_ok(), "empty placeholder allowed");
        assert!(signed_with(&"A".repeat(64)).is_err(), "uppercase rejected");
        assert!(signed_with(&"g".repeat(64)).is_err(), "non-hex rejected");
        assert!(
            signed_with(&"a".repeat(63)).is_err(),
            "wrong length rejected"
        );

        // A forged entry whose signature is valid over a junk-hash payload must
        // still fail verify — verifiers enforce the wire invariant, not just the
        // authoring route.
        let mut app = sample_app('a');
        app.archive_hash = "z".repeat(64);
        let mut dir = NodeDirectory::new(kp.public_bytes(), 1);
        dir.catalog.push(app);
        let bytes = canonical_bytes(&dir, DOMAIN_NODE_DIRECTORY_V1).unwrap();
        let forged = NodeDirectoryEntry {
            directory: dir,
            node_id: kp.public_bytes(),
            signature: kp.sign(&bytes),
        };
        assert!(
            forged.verify_signature().is_err(),
            "verify must reject a malformed archive_hash even with a valid signature"
        );
    }

    #[test]
    fn each_per_field_cap_independently_enforced() {
        // Mirror curator.rs `verify_rejects_oversized_fields`: exercise EVERY
        // per-field cap independently so a regression that drops any single
        // check is caught by at least one assert (archive_hash has its own
        // test above).
        let kp = KeyPair::generate();
        let over = |mutate: fn(&mut CatalogApp)| {
            let mut app = sample_app('a');
            mutate(&mut app);
            let mut dir = NodeDirectory::new(kp.public_bytes(), 1);
            dir.catalog.push(app);
            NodeDirectoryEntry::sign(dir, &kp).is_err()
        };
        assert!(over(
            |a| a.project_id = "a".repeat(NODE_DIRECTORY_PROJECT_ID_MAX + 1)
        ));
        assert!(over(
            |a| a.project_name = "a".repeat(NODE_DIRECTORY_PROJECT_NAME_MAX + 1)
        ));
        assert!(over(
            |a| a.category = "a".repeat(NODE_DIRECTORY_CATEGORY_MAX + 1)
        ));
        assert!(over(
            |a| a.description = "a".repeat(NODE_DIRECTORY_DESCRIPTION_MAX + 1)
        ));
    }

    #[test]
    fn node_directory_cross_domain_bytes_differ() {
        // The canonical bytes under the node-directory domain differ from the
        // same payload under the curator-list domain (the domain prefix makes
        // the pre-images disjoint). This is a necessary precondition for
        // non-replayability; the *sufficient* guarantee is exercised by
        // `node_directory_cross_domain_signature_rejected` below.
        let kp = KeyPair::generate();
        let dir = sample_directory(kp.public_bytes());
        let directory_bytes = canonical_bytes(&dir, DOMAIN_NODE_DIRECTORY_V1).unwrap();
        let as_curator = canonical_bytes(&dir, DOMAIN_CURATOR_LIST_V1).unwrap();
        assert_ne!(
            directory_bytes, as_curator,
            "domain separation must yield distinct byte strings"
        );
    }

    #[test]
    fn node_directory_cross_domain_signature_rejected() {
        // The real anti-replay guarantee: a signature produced over the
        // directory payload under the WRONG domain (curator-list) must NOT
        // verify as a NodeDirectoryEntry. This is the only assertion that
        // exercises the domain tag *inside* verify_signature — a hypothetical
        // bug using the wrong domain there would slip past the byte-inequality
        // test but is caught here.
        let kp = KeyPair::generate();
        let dir = sample_directory(kp.public_bytes());
        let wrong_domain_sig = kp.sign(&canonical_bytes(&dir, DOMAIN_CURATOR_LIST_V1).unwrap());
        let forged = NodeDirectoryEntry {
            directory: dir,
            node_id: kp.public_bytes(),
            signature: wrong_domain_sig,
        };
        assert!(
            forged.verify_signature().is_err(),
            "a signature minted under a different domain must not verify as a directory"
        );
    }

    #[test]
    fn json_roundtrips() {
        let kp = KeyPair::generate();
        let entry = NodeDirectoryEntry::sign(sample_directory(kp.public_bytes()), &kp).unwrap();
        let json = serde_json::to_vec(&entry).unwrap();
        let back: NodeDirectoryEntry = serde_json::from_slice(&json).unwrap();
        assert_eq!(back, entry);
        back.verify_signature()
            .expect("round-tripped entry must still verify");
    }
}
