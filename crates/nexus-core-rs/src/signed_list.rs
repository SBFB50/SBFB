// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared abstraction over self-published, signed, revision-replaceable
//! list payloads (Sprint 75 Phase B).
//!
//! Both [`crate::curator::CuratorListEntry`] and
//! [`crate::node_directory::NodeDirectoryEntry`] are fetched, verified,
//! and replicated by the shell-daemon through the *same* gossip ingest
//! gate: verify the signature, cross-check the announcement pubkey
//! against the payload's signer, then reject a non-monotonic revision.
//!
//! Copying that 3-step security gate per type is the drift risk R1 (a
//! future fix to one arm silently skips the other). The [`SignedList`]
//! trait lets one generic ingest helper own those three checks for
//! every signed-list type. The trait deliberately exposes ONLY the
//! fields the type-agnostic gate needs — the signer pubkey, the
//! revision, and the full signature verification — so a new signed-list
//! type opts into the shared gate by implementing three trivial
//! accessors.

use crate::crypto::PUBLIC_KEY_LENGTH;
use crate::curator::CuratorListEntry;
use crate::error::Result;
use crate::node_directory::NodeDirectoryEntry;

/// A signed, self-published list payload that flows through the shared
/// gossip ingest gate.
///
/// Implementors must guarantee that [`SignedList::verify`] is the
/// authoritative full verification (version, caps, attribution split-
/// brain, Ed25519 signature) and that [`SignedList::signer_pubkey`]
/// returns the key that produced the signature — the gate cross-checks
/// it against the gossip announcement's declared pubkey.
pub trait SignedList {
    /// The Ed25519 public key that signed this list — the author /
    /// node identity. The ingest gate cross-checks this against the
    /// announcement's declared pubkey (attribution split-brain
    /// mitigation).
    fn signer_pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH];

    /// The monotonic revision counter used for rollback protection. The
    /// ingest gate rejects a new entry whose revision is not strictly
    /// greater than the stored one.
    fn list_revision(&self) -> u64;

    /// Full signature verification: version, caps, attribution
    /// consistency, and the Ed25519 signature over the canonical bytes.
    /// Returns [`crate::error::NexusError`] on any failure.
    fn verify(&self) -> Result<()>;
}

impl SignedList for CuratorListEntry {
    fn signer_pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.curator_pubkey
    }

    fn list_revision(&self) -> u64 {
        self.list.revision
    }

    fn verify(&self) -> Result<()> {
        self.verify_signature()
    }
}

impl SignedList for NodeDirectoryEntry {
    fn signer_pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.node_id
    }

    fn list_revision(&self) -> u64 {
        self.directory.revision
    }

    fn verify(&self) -> Result<()> {
        self.verify_signature()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::KeyPair;
    use crate::curator::CuratorList;
    use crate::node_directory::{CatalogApp, NodeDirectory};

    #[test]
    fn curator_entry_exposes_signer_and_revision() {
        let kp = KeyPair::generate();
        let mut list = CuratorList::new(kp.public_bytes(), "Curation", 0, 7);
        list.entries.push(crate::curator::CuratorProjectRef {
            project_id: "a".repeat(64),
            project_name: "p".into(),
            category: "misc".into(),
            description: String::new(),
        });
        let entry = CuratorListEntry::sign(list, &kp).unwrap();
        assert_eq!(entry.signer_pubkey(), kp.public_bytes());
        assert_eq!(entry.list_revision(), 7);
        entry
            .verify()
            .expect("trait verify delegates to verify_signature");
    }

    #[test]
    fn node_directory_entry_exposes_signer_and_revision() {
        let kp = KeyPair::generate();
        let mut dir = NodeDirectory::new(kp.public_bytes(), 3);
        dir.catalog.push(CatalogApp {
            project_id: "a".repeat(64),
            archive_hash: "b".repeat(64),
            project_name: "Babel".into(),
            category: "translation".into(),
            description: String::new(),
        });
        let entry = NodeDirectoryEntry::sign(dir, &kp).unwrap();
        assert_eq!(entry.signer_pubkey(), kp.public_bytes());
        assert_eq!(entry.list_revision(), 3);
        entry
            .verify()
            .expect("trait verify delegates to verify_signature");
    }
}
