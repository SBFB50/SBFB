// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public feed — append-only signed event log for the SBFB protocol.
//!
//! Each operation records a verifiable protocol event (release
//! published, source became stale, etc.) in a BLAKE3 hash-chain
//! with Ed25519 per-entry signatures. The feed can be replayed
//! from genesis to reconstruct a `PublicRegistryView`.
//!
//! Wire format version: `FEED_FORMAT_VERSION = 1` under the
//! post-v1.0 versioning regime (each break bumps the version,
//! decoders accept a range, `#[serde(default)]` for compat).

use serde::{Deserialize, Serialize};

/// Wire format version for the public feed.
/// Post-v1.0 regime: each breaking change bumps this value.
pub const FEED_FORMAT_VERSION: u16 = 1;

// ---------------------------------------------------------------------------
// Operation payloads
// ---------------------------------------------------------------------------

/// Payload for a release-published event.
///
/// `is_open_source` is server-derived at publish time (not user-settable).
/// Valid only when the full verification chain is present:
/// `repo_url + commit_sha + artifact_hash + provenance_hash`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleasePublishedPayload {
    pub project_id: String,
    pub repo_url: String,
    pub commit_sha: String,
    pub artifact_hash: String,
    #[serde(default)]
    pub provenance_hash: Option<String>,
    pub is_open_source: bool,
}

/// Payload for a source-became-stale event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceBecameStalePayload {
    pub project_id: String,
    pub reason: String,
}

/// Discriminated union of all public feed operation types.
///
/// Sprint 1 implements `ReleasePublished` and `SourceBecameStale`.
/// Future variants (`CuratorVouched`, `BuildQuorumReached`,
/// `SourceRecovered`, `SearchManifestPublished`) are defined in
/// the protocol spec but implemented in Sprint 2+.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op_type")]
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
}

// ---------------------------------------------------------------------------
// Feed entry (stored + transmitted)
// ---------------------------------------------------------------------------

/// A single entry in the public feed log.
///
/// `entry_hash` and `signature` are computed from the canonical
/// representation of `FeedEntryCanonical`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedEntry {
    pub version: u16,
    pub seq: u64,
    pub op: PublicFeedOperation,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub entry_hash: String,
    pub prev_hash: String,
    pub signature: String,
}

/// Canonical representation of a feed entry for hashing and signing.
///
/// This struct is serialized via JCS (RFC 8785) with domain
/// separation `DOMAIN_FEED_V1` to produce deterministic bytes.
/// The `entry_hash` is `BLAKE3(canonical_bytes)` and the
/// `signature` is `Ed25519::sign(canonical_bytes)`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntryCanonical {
    pub version: u16,
    pub op: PublicFeedOperation,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub prev_hash: String,
}

impl FeedEntry {
    /// Build the canonical struct for this entry (for hashing/signing).
    pub fn to_canonical(&self) -> FeedEntryCanonical {
        FeedEntryCanonical {
            version: self.version,
            op: self.op.clone(),
            author_pubkey: self.author_pubkey.clone(),
            timestamp: self.timestamp,
            prev_hash: self.prev_hash.clone(),
        }
    }
}

/// Genesis prev_hash — the sentinel value for the first entry.
pub const GENESIS_PREV_HASH: &str = "genesis";

/// Compute the entry hash for a feed entry's canonical form.
pub fn compute_feed_entry_hash(canonical: &FeedEntryCanonical) -> Result<String, String> {
    let bytes = nexus_core_rs::canonical_bytes(canonical, nexus_core_rs::DOMAIN_FEED_V1)
        .map_err(|e| format!("canonical serialization failed: {e}"))?;
    let hash = blake3::hash(&bytes);
    Ok(hex::encode(hash.as_bytes()))
}

/// Compute the canonical bytes for signing.
pub fn compute_feed_canonical_bytes(canonical: &FeedEntryCanonical) -> Result<Vec<u8>, String> {
    nexus_core_rs::canonical_bytes(canonical, nexus_core_rs::DOMAIN_FEED_V1)
        .map_err(|e| format!("canonical serialization failed: {e}"))
}

// ---------------------------------------------------------------------------
// FeedStore — append-only persistence + hash-chain
// ---------------------------------------------------------------------------

use crate::db::CoordinatorDb;

/// Insert a new operation into the feed with hash-chain and signature.
///
/// The caller provides the operation, author identity, and a signing
/// closure. The function atomically reads the previous hash, computes
/// the canonical bytes, signs, hashes, and persists.
pub fn insert_feed_operation(
    db: &CoordinatorDb,
    op: PublicFeedOperation,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    let prev_hash = db
        .get_last_feed_entry_hash()
        .map_err(|e| format!("db error: {e}"))?
        .unwrap_or_else(|| GENESIS_PREV_HASH.to_string());

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let canonical = FeedEntryCanonical {
        version: FEED_FORMAT_VERSION,
        op: op.clone(),
        author_pubkey: author_pubkey.to_string(),
        timestamp,
        prev_hash: prev_hash.clone(),
    };

    let canonical_bytes = compute_feed_canonical_bytes(&canonical)?;
    let signature = sign_fn(&canonical_bytes);
    let entry_hash = compute_feed_entry_hash(&canonical)?;

    let op_type = match &op {
        PublicFeedOperation::ReleasePublished(_) => "ReleasePublished",
        PublicFeedOperation::SourceBecameStale(_) => "SourceBecameStale",
    };
    let payload = serde_json::to_string(&op).map_err(|e| format!("payload serialization: {e}"))?;

    let row = crate::db::FeedEntryRow {
        seq: 0,
        op_type: op_type.to_string(),
        payload,
        author: author_pubkey.to_string(),
        signature: hex::encode(&signature),
        entry_hash: entry_hash.clone(),
        prev_hash: prev_hash.clone(),
        created_at: timestamp,
    };

    let seq = db
        .insert_feed_entry(&row)
        .map_err(|e| format!("db insert: {e}"))?;

    Ok(FeedEntry {
        version: FEED_FORMAT_VERSION,
        seq,
        op,
        author_pubkey: author_pubkey.to_string(),
        timestamp,
        entry_hash,
        prev_hash,
        signature: hex::encode(signature),
    })
}

/// Replay all feed entries from genesis in sequence order.
pub fn replay_all(db: &CoordinatorDb) -> Result<Vec<FeedEntry>, String> {
    let rows = db
        .get_feed_entries()
        .map_err(|e| format!("db error: {e}"))?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let op: PublicFeedOperation =
            serde_json::from_str(&row.payload).map_err(|e| format!("payload parse: {e}"))?;
        entries.push(FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: row.seq,
            op,
            author_pubkey: row.author,
            timestamp: row.created_at,
            entry_hash: row.entry_hash,
            prev_hash: row.prev_hash,
            signature: row.signature,
        });
    }
    Ok(entries)
}

/// Verify the hash-chain integrity of a sequence of feed entries.
///
/// Checks that each entry's `entry_hash` matches the recomputed
/// BLAKE3 hash, and that `prev_hash` links are consistent.
pub fn verify_chain(entries: &[FeedEntry]) -> Result<(), String> {
    let mut expected_prev = GENESIS_PREV_HASH.to_string();

    for (i, entry) in entries.iter().enumerate() {
        if entry.prev_hash != expected_prev {
            return Err(format!(
                "entry {i} (seq {}): prev_hash mismatch: expected {expected_prev}, got {}",
                entry.seq, entry.prev_hash
            ));
        }

        let canonical = entry.to_canonical();
        let recomputed = compute_feed_entry_hash(&canonical)?;
        if entry.entry_hash != recomputed {
            return Err(format!(
                "entry {i} (seq {}): entry_hash mismatch: stored {}, recomputed {recomputed}",
                entry.seq, entry.entry_hash
            ));
        }

        expected_prev = entry.entry_hash.clone();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_release_published() -> PublicFeedOperation {
        PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "abc123def456".to_string(),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        })
    }

    fn sample_source_stale() -> PublicFeedOperation {
        PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "abc123def456".to_string(),
            reason: "repo_unreachable".to_string(),
        })
    }

    #[test]
    fn test_feed_operation_serde_roundtrip() {
        let ops = vec![sample_release_published(), sample_source_stale()];
        for op in ops {
            let json = serde_json::to_string(&op).unwrap();
            let back: PublicFeedOperation = serde_json::from_str(&json).unwrap();
            assert_eq!(op, back);
        }
    }

    #[test]
    fn test_canonical_bytes_feed_deterministic() {
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes1 = compute_feed_canonical_bytes(&canonical).unwrap();
        let bytes2 = compute_feed_canonical_bytes(&canonical).unwrap();
        assert_eq!(bytes1, bytes2);
        assert!(!bytes1.is_empty());
    }

    #[test]
    fn test_feed_format_version() {
        assert_eq!(FEED_FORMAT_VERSION, 1);
    }

    #[test]
    fn test_compute_feed_entry_hash_deterministic() {
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let hash1 = compute_feed_entry_hash(&canonical).unwrap();
        let hash2 = compute_feed_entry_hash(&canonical).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
        assert_eq!(
            hash1,
            "f81ced7da512d9615a63e67e99b70fa89a1116b7101c0d3f313d83caf569ae2a"
        );
    }

    #[test]
    fn test_entry_hash_changes_with_prev_hash() {
        let mut canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let hash_genesis = compute_feed_entry_hash(&canonical).unwrap();
        canonical.prev_hash = "f".repeat(64);
        let hash_chained = compute_feed_entry_hash(&canonical).unwrap();
        assert_ne!(hash_genesis, hash_chained);
    }

    // -- Phase B tests: FeedStore persistence + hash-chain --

    fn dummy_sign(data: &[u8]) -> Vec<u8> {
        blake3::hash(data).as_bytes().to_vec()
    }

    #[test]
    fn test_insert_operation_persists() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let entry =
            insert_feed_operation(&db, sample_release_published(), &"d".repeat(64), dummy_sign)
                .unwrap();
        assert_eq!(entry.seq, 1);
        assert_eq!(entry.prev_hash, GENESIS_PREV_HASH);
        assert!(!entry.entry_hash.is_empty());

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].op, sample_release_published());
    }

    #[test]
    fn test_replay_all_ordered() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        insert_feed_operation(&db, sample_release_published(), &"d".repeat(64), dummy_sign)
            .unwrap();
        insert_feed_operation(&db, sample_source_stale(), &"d".repeat(64), dummy_sign).unwrap();
        insert_feed_operation(&db, sample_release_published(), &"e".repeat(64), dummy_sign)
            .unwrap();

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries[0].seq < entries[1].seq);
        assert!(entries[1].seq < entries[2].seq);
    }

    #[test]
    fn test_hash_chain_valid() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        insert_feed_operation(&db, sample_release_published(), &"d".repeat(64), dummy_sign)
            .unwrap();
        insert_feed_operation(&db, sample_source_stale(), &"d".repeat(64), dummy_sign).unwrap();
        insert_feed_operation(&db, sample_release_published(), &"e".repeat(64), dummy_sign)
            .unwrap();

        let entries = replay_all(&db).unwrap();
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_hash_chain_genesis() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let entry =
            insert_feed_operation(&db, sample_release_published(), &"d".repeat(64), dummy_sign)
                .unwrap();
        assert_eq!(entry.prev_hash, GENESIS_PREV_HASH);
    }

    #[test]
    fn test_verify_chain_empty() {
        let entries: Vec<FeedEntry> = vec![];
        assert!(verify_chain(&entries).is_ok());
    }
}
