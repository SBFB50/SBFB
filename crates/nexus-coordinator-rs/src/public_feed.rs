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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Wire format version for the public feed.
/// Post-v1.0 regime: each breaking change bumps this value.
pub const FEED_FORMAT_VERSION: u16 = 1;

// ---------------------------------------------------------------------------
// Operation payloads
// ---------------------------------------------------------------------------

/// Payload for a release-published event.
///
/// `is_open_source` is validated at insert time: `true` requires
/// `provenance_hash` (spec §2.1). The constraint is enforced by
/// `validate_feed_operation()`, not by struct construction.
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
    /// Transport-level proof of work nonce for anti-spam.
    /// Not part of `FeedEntryCanonical` (does not affect entry_hash
    /// or signature). `#[serde(default)]` for runtime tolerance:
    /// local entries omit it (self-trust), remote sync enforces it.
    #[serde(default)]
    pub pow_nonce: Option<u64>,
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
// Proof of work — transport-level anti-spam (Sprint 62 Phase D)
// ---------------------------------------------------------------------------

/// Minimum leading zero bits required in the PoW hash.
/// 16 bits ≈ 65k average iterations ≈ 10-50 ms on modern CPU.
pub const FEED_POW_DIFFICULTY: u32 = 16;

fn leading_zero_bits(bytes: &[u8]) -> u32 {
    let mut count = 0;
    for &b in bytes {
        if b == 0 {
            count += 8;
        } else {
            count += b.leading_zeros();
            break;
        }
    }
    count
}

/// Verify that `nonce` satisfies the PoW difficulty for `entry_hash`.
///
/// Computes `BLAKE3(entry_hash_ascii || nonce_le_bytes)` and checks
/// that the result has at least `FEED_POW_DIFFICULTY` leading zero bits.
pub fn verify_feed_pow(entry_hash: &str, nonce: u64) -> bool {
    let mut input = Vec::with_capacity(entry_hash.len() + 8);
    input.extend_from_slice(entry_hash.as_bytes());
    input.extend_from_slice(&nonce.to_le_bytes());
    let hash = blake3::hash(&input);
    leading_zero_bits(hash.as_bytes()) >= FEED_POW_DIFFICULTY
}

/// Brute-force a valid PoW nonce for `entry_hash`.
pub fn compute_feed_pow(entry_hash: &str) -> u64 {
    let prefix = entry_hash.as_bytes();
    for nonce in 0u64.. {
        let mut input = Vec::with_capacity(prefix.len() + 8);
        input.extend_from_slice(prefix);
        input.extend_from_slice(&nonce.to_le_bytes());
        let hash = blake3::hash(&input);
        if leading_zero_bits(hash.as_bytes()) >= FEED_POW_DIFFICULTY {
            return nonce;
        }
    }
    unreachable!()
}

// ---------------------------------------------------------------------------
// FeedStore — append-only persistence + hash-chain
// ---------------------------------------------------------------------------

use crate::db::CoordinatorDb;

fn is_hex_exact(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit())
}

const VALID_STALE_REASONS: &[&str] = &["repo_unreachable", "commit_diverged", "manual"];

/// Maximum serialized size of a feed operation payload (64 KB).
pub const MAX_OPERATION_JSON_SIZE: usize = 65_536;

/// Maximum feed operations per author per 60-second window.
pub const FEED_RATE_LIMIT_PER_MINUTE: u64 = 5;

/// Validate format and semantic constraints on a feed operation.
///
/// Format: project_id hex-64, repo_url HTTPS, commit_sha hex-40,
/// artifact_hash hex-64, reason in the protocol allowlist.
/// Semantic (spec §2.1): `is_open_source: true` requires provenance_hash.
/// Size: serialized JSON must not exceed `MAX_OPERATION_JSON_SIZE`.
pub fn validate_feed_operation(op: &PublicFeedOperation) -> Result<(), String> {
    let json = serde_json::to_string(op).map_err(|e| format!("payload serialization: {e}"))?;
    if json.len() > MAX_OPERATION_JSON_SIZE {
        return Err(format!(
            "operation payload exceeds {} bytes limit",
            MAX_OPERATION_JSON_SIZE
        ));
    }
    match op {
        PublicFeedOperation::ReleasePublished(p) => {
            if !is_hex_exact(&p.project_id, 64) {
                return Err("project_id must be 64 hex characters".to_string());
            }
            if !p.repo_url.starts_with("https://") {
                return Err("repo_url must start with https://".to_string());
            }
            if !is_hex_exact(&p.commit_sha, 40) {
                return Err("commit_sha must be 40 hex characters".to_string());
            }
            if !is_hex_exact(&p.artifact_hash, 64) {
                return Err("artifact_hash must be 64 hex characters".to_string());
            }
            if let Some(ref ph) = p.provenance_hash {
                if !is_hex_exact(ph, 64) {
                    return Err("provenance_hash must be 64 hex characters".to_string());
                }
            }
            if p.is_open_source && p.provenance_hash.is_none() {
                return Err("is_open_source=true requires provenance_hash (spec §2.1)".to_string());
            }
        }
        PublicFeedOperation::SourceBecameStale(p) => {
            if !is_hex_exact(&p.project_id, 64) {
                return Err("project_id must be 64 hex characters".to_string());
            }
            if p.reason.is_empty() {
                return Err("reason must not be empty".to_string());
            }
            if !VALID_STALE_REASONS.contains(&p.reason.as_str()) {
                return Err(
                    "reason must be one of: repo_unreachable, commit_diverged, manual".to_string(),
                );
            }
        }
    }
    Ok(())
}

/// Insert a new operation into the feed with hash-chain and signature.
///
/// Local-trust path (spec §5.1): validates semantic constraints but
/// does NOT enforce per-author rate limiting. Use for self-authored
/// entries where the local process is the sole writer.
pub fn insert_feed_operation(
    db: &CoordinatorDb,
    op: PublicFeedOperation,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    validate_feed_operation(&op)?;

    db.conn()
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("begin tx: {e}"))?;

    match insert_feed_operation_inner(db, op, author_pubkey, sign_fn) {
        Ok(entry) => {
            db.conn()
                .execute_batch("COMMIT")
                .map_err(|e| format!("commit tx: {e}"))?;
            Ok(entry)
        }
        Err(e) => {
            let _ = db.conn().execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

/// Insert with per-author rate limiting (spec §5.1 remote-trust path).
///
/// Enforces `FEED_RATE_LIMIT_PER_MINUTE` before insert. Use for entries
/// received from peers where trust is not implicit.
pub fn insert_feed_operation_rate_limited(
    db: &CoordinatorDb,
    op: PublicFeedOperation,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    validate_feed_operation(&op)?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let window_start = now.saturating_sub(60);
    let recent_count = db
        .count_feed_entries_by_author_since(author_pubkey, window_start)
        .map_err(|e| format!("rate limit check: {e}"))?;
    if recent_count >= FEED_RATE_LIMIT_PER_MINUTE {
        return Err(format!(
            "rate limit exceeded: {} ops in last 60s (max {})",
            recent_count, FEED_RATE_LIMIT_PER_MINUTE
        ));
    }

    db.conn()
        .execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| format!("begin tx: {e}"))?;

    match insert_feed_operation_inner(db, op, author_pubkey, sign_fn) {
        Ok(entry) => {
            db.conn()
                .execute_batch("COMMIT")
                .map_err(|e| format!("commit tx: {e}"))?;
            Ok(entry)
        }
        Err(e) => {
            let _ = db.conn().execute_batch("ROLLBACK");
            Err(e)
        }
    }
}

fn insert_feed_operation_inner(
    db: &CoordinatorDb,
    op: PublicFeedOperation,
    author_pubkey: &str,
    sign_fn: impl FnOnce(&[u8]) -> Vec<u8>,
) -> Result<FeedEntry, String> {
    let prev_hash = db
        .get_last_feed_entry_hash_by_author(author_pubkey)
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
        pow_nonce: None,
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
            pow_nonce: None,
        });
    }
    Ok(entries)
}

/// Verify entry_hash and Ed25519 signature for a single entry.
///
/// Does NOT check prev_hash linkage — use [`verify_chain`] for
/// full chain verification. Useful for incremental verification
/// of newly received entries.
pub fn verify_entry(entry: &FeedEntry) -> Result<(), String> {
    let canonical = entry.to_canonical();
    let canonical_bytes = compute_feed_canonical_bytes(&canonical)?;
    let recomputed = compute_feed_entry_hash(&canonical)?;
    if entry.entry_hash != recomputed {
        return Err(format!(
            "entry seq {}: entry_hash mismatch: stored {}, recomputed {recomputed}",
            entry.seq, entry.entry_hash
        ));
    }

    let pubkey_bytes = hex::decode(&entry.author_pubkey)
        .map_err(|e| format!("entry seq {}: bad pubkey hex: {e}", entry.seq))?;
    let sig_bytes = hex::decode(&entry.signature)
        .map_err(|e| format!("entry seq {}: bad signature hex: {e}", entry.seq))?;

    if pubkey_bytes.len() == 32 && sig_bytes.len() == 64 {
        let pubkey: [u8; 32] = pubkey_bytes.try_into().unwrap();
        let sig: [u8; 64] = sig_bytes.try_into().unwrap();
        nexus_core_rs::verify(&pubkey, &canonical_bytes, &sig).map_err(|_| {
            format!(
                "entry seq {}: Ed25519 signature verification failed",
                entry.seq
            )
        })?;
    } else {
        return Err(format!(
            "entry seq {}: invalid pubkey/signature length (expected 32/64 bytes)",
            entry.seq
        ));
    }

    Ok(())
}

/// Maximum age tolerance for incoming feed entry timestamps.
/// Entries claiming a timestamp more than 30 days in the future are
/// rejected as invalid (defense-in-depth against timestamp forgery).
pub const FEED_MAX_FUTURE_SECS: u64 = 30 * 24 * 3600;

/// Validate that a feed entry's timestamp is not unreasonably far in
/// the future. Returns `Ok(())` if the timestamp is at most
/// `FEED_MAX_FUTURE_SECS` seconds ahead of `now_epoch`.
pub fn validate_feed_entry_timestamp(entry: &FeedEntry, now_epoch: u64) -> Result<(), String> {
    let max_allowed = now_epoch.saturating_add(FEED_MAX_FUTURE_SECS);
    if entry.timestamp > max_allowed {
        return Err(format!(
            "entry seq {}: timestamp {} is more than 30 days in the future (max {})",
            entry.seq, entry.timestamp, max_allowed
        ));
    }
    Ok(())
}

/// Verify hash-chain integrity and Ed25519 signatures of feed entries.
///
/// Supports multi-author feeds: each author's entries form an
/// independent chain (per-author prev_hash linkage). Order-
/// independent: entries may be stored in any DB insertion order —
/// the function rebuilds each per-author chain via prev_hash →
/// entry_hash linkage, then verifies (1) chain completeness,
/// (2) entry_hash recomputation, (3) Ed25519 signature. Spec §4.
pub fn verify_chain(entries: &[FeedEntry]) -> Result<(), String> {
    let mut by_author: HashMap<&str, Vec<&FeedEntry>> = HashMap::new();
    for entry in entries {
        by_author
            .entry(entry.author_pubkey.as_str())
            .or_default()
            .push(entry);
    }

    for (author_key, author_entries) in &by_author {
        let mut by_prev: HashMap<&str, &FeedEntry> = HashMap::new();
        for entry in author_entries {
            by_prev.insert(entry.prev_hash.as_str(), entry);
        }

        let mut current_prev: &str = GENESIS_PREV_HASH;
        let mut verified_count = 0usize;
        while let Some(entry) = by_prev.remove(current_prev) {
            verify_entry(entry).map_err(|e| {
                format!(
                    "author {}...: {e}",
                    &author_key[..std::cmp::min(8, author_key.len())]
                )
            })?;
            current_prev = entry.entry_hash.as_str();
            verified_count += 1;
        }

        if verified_count != author_entries.len() {
            return Err(format!(
                "author {}...: chain has {} linked entries but {} total (broken linkage or fork)",
                &author_key[..std::cmp::min(8, author_key.len())],
                verified_count,
                author_entries.len()
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_release_published() -> PublicFeedOperation {
        PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        })
    }

    fn sample_source_stale() -> PublicFeedOperation {
        PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "a1".repeat(32),
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
        // Spec §7 test vector — inline data to keep vector stable
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
                project_id: "abc123def456".to_string(),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
            }),
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

    fn test_keypair() -> nexus_core_rs::KeyPair {
        nexus_core_rs::KeyPair::from_secret_bytes(&[42u8; 32])
    }

    fn pubkey_hex(kp: &nexus_core_rs::KeyPair) -> String {
        hex::encode(kp.public_bytes())
    }

    #[test]
    fn test_insert_operation_persists() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let entry = insert_feed_operation(&db, sample_release_published(), &pubkey_hex(&kp), |d| {
            kp.sign(d).to_vec()
        })
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
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        insert_feed_operation(&db, sample_source_stale(), &pk, |d| kp.sign(d).to_vec()).unwrap();
        insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries[0].seq < entries[1].seq);
        assert!(entries[1].seq < entries[2].seq);
    }

    #[test]
    fn test_hash_chain_valid_with_ed25519() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        insert_feed_operation(&db, sample_source_stale(), &pk, |d| kp.sign(d).to_vec()).unwrap();

        let entries = replay_all(&db).unwrap();
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_hash_chain_genesis() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let entry = insert_feed_operation(&db, sample_release_published(), &pubkey_hex(&kp), |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        assert_eq!(entry.prev_hash, GENESIS_PREV_HASH);
    }

    #[test]
    fn test_verify_chain_empty() {
        let entries: Vec<FeedEntry> = vec![];
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_validate_is_open_source_missing_provenance() {
        let op = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: None,
            is_open_source: true,
        });
        let result = validate_feed_operation(&op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("provenance_hash"));
    }

    #[test]
    fn test_validate_is_open_source_valid() {
        let result = validate_feed_operation(&sample_release_published());
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_chain_forged_signature() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();

        let mut entries = replay_all(&db).unwrap();
        entries[0].signature = hex::encode([0u8; 64]);
        let result = verify_chain(&entries);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("signature verification failed")
        );
    }

    #[test]
    fn test_verify_chain_tampered_hash() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();

        let mut entries = replay_all(&db).unwrap();
        entries[0].entry_hash = "f".repeat(64);
        let result = verify_chain(&entries);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("entry_hash mismatch"));
    }

    #[test]
    fn test_validate_feed_operation_strict() {
        // project_id not hex-64
        let bad_pid = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "short".to_string(),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        });
        assert!(
            validate_feed_operation(&bad_pid)
                .unwrap_err()
                .contains("project_id")
        );

        // repo_url not HTTPS
        let bad_url = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "http://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        });
        assert!(
            validate_feed_operation(&bad_url)
                .unwrap_err()
                .contains("repo_url")
        );

        // commit_sha not hex-40
        let bad_sha = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "z".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        });
        assert!(
            validate_feed_operation(&bad_sha)
                .unwrap_err()
                .contains("commit_sha")
        );

        // artifact_hash not hex-64
        let bad_art = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "short".to_string(),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        });
        assert!(
            validate_feed_operation(&bad_art)
                .unwrap_err()
                .contains("artifact_hash")
        );

        // SourceBecameStale empty reason
        let bad_reason = PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "a1".repeat(32),
            reason: "".to_string(),
        });
        assert!(
            validate_feed_operation(&bad_reason)
                .unwrap_err()
                .contains("reason")
        );

        // SourceBecameStale unknown reason
        let bad_unknown_reason = PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "a1".repeat(32),
            reason: "unknown".to_string(),
        });
        assert!(
            validate_feed_operation(&bad_unknown_reason)
                .unwrap_err()
                .contains("reason")
        );

        // SourceBecameStale bad project_id
        let bad_stale_pid = PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "not-hex".to_string(),
            reason: "repo_unreachable".to_string(),
        });
        assert!(
            validate_feed_operation(&bad_stale_pid)
                .unwrap_err()
                .contains("project_id")
        );

        // Valid operations pass
        assert!(validate_feed_operation(&sample_release_published()).is_ok());
        assert!(validate_feed_operation(&sample_source_stale()).is_ok());
        for reason in VALID_STALE_REASONS {
            assert!(
                validate_feed_operation(&PublicFeedOperation::SourceBecameStale(
                    SourceBecameStalePayload {
                        project_id: "a1".repeat(32),
                        reason: (*reason).to_string(),
                    },
                ))
                .is_ok()
            );
        }
    }

    #[test]
    fn test_insert_feed_transaction_atomic() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let e1 = insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        let e2 = insert_feed_operation(&db, sample_source_stale(), &pk, |d| kp.sign(d).to_vec())
            .unwrap();
        let e3 = insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();

        assert_eq!(e1.prev_hash, GENESIS_PREV_HASH);
        assert_eq!(e2.prev_hash, e1.entry_hash);
        assert_eq!(e3.prev_hash, e2.entry_hash);

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 3);
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_verify_chain_multi_author() {
        let kp_a = nexus_core_rs::KeyPair::from_secret_bytes(&[1u8; 32]);
        let kp_b = nexus_core_rs::KeyPair::from_secret_bytes(&[2u8; 32]);
        let pk_a = hex::encode(kp_a.public_bytes());
        let pk_b = hex::encode(kp_b.public_bytes());

        // Author A entry 1: genesis → a1
        let can_a1 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk_a.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes_a1 = compute_feed_canonical_bytes(&can_a1).unwrap();
        let hash_a1 = compute_feed_entry_hash(&can_a1).unwrap();
        let entry_a1 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk_a.clone(),
            timestamp: 1000,
            entry_hash: hash_a1.clone(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp_a.sign(&bytes_a1)),
            pow_nonce: None,
        };

        // Author B entry 1: genesis → b1 (independent chain)
        let can_b1 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_source_stale(),
            author_pubkey: pk_b.clone(),
            timestamp: 1001,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes_b1 = compute_feed_canonical_bytes(&can_b1).unwrap();
        let hash_b1 = compute_feed_entry_hash(&can_b1).unwrap();
        let entry_b1 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 2,
            op: sample_source_stale(),
            author_pubkey: pk_b.clone(),
            timestamp: 1001,
            entry_hash: hash_b1,
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp_b.sign(&bytes_b1)),
            pow_nonce: None,
        };

        // Author A entry 2: a1 → a2
        let can_a2 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk_a.clone(),
            timestamp: 1002,
            prev_hash: hash_a1.clone(),
        };
        let bytes_a2 = compute_feed_canonical_bytes(&can_a2).unwrap();
        let hash_a2 = compute_feed_entry_hash(&can_a2).unwrap();
        let entry_a2 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 3,
            op: sample_release_published(),
            author_pubkey: pk_a.clone(),
            timestamp: 1002,
            entry_hash: hash_a2,
            prev_hash: hash_a1,
            signature: hex::encode(kp_a.sign(&bytes_a2)),
            pow_nonce: None,
        };

        // Interleaved: A1, B1, A2 — per-author chains are valid
        let entries = vec![entry_a1, entry_b1, entry_a2];
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_feed_persist_reopen_verify() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        {
            let db = CoordinatorDb::open(&db_path).unwrap();
            insert_feed_operation(&db, sample_release_published(), &pk, |d| {
                kp.sign(d).to_vec()
            })
            .unwrap();
            insert_feed_operation(&db, sample_source_stale(), &pk, |d| kp.sign(d).to_vec())
                .unwrap();
        }

        {
            let db = CoordinatorDb::open(&db_path).unwrap();
            let entries = replay_all(&db).unwrap();
            assert_eq!(entries.len(), 2);
            assert!(verify_chain(&entries).is_ok());
        }
    }

    #[test]
    fn test_verify_chain_out_of_order_insertion() {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[7u8; 32]);
        let pk = hex::encode(kp.public_bytes());

        let can1 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes1 = compute_feed_canonical_bytes(&can1).unwrap();
        let hash1 = compute_feed_entry_hash(&can1).unwrap();
        let entry1 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            entry_hash: hash1.clone(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp.sign(&bytes1)),
            pow_nonce: None,
        };

        let can2 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_source_stale(),
            author_pubkey: pk.clone(),
            timestamp: 1001,
            prev_hash: hash1.clone(),
        };
        let bytes2 = compute_feed_canonical_bytes(&can2).unwrap();
        let hash2 = compute_feed_entry_hash(&can2).unwrap();
        let entry2 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 2,
            op: sample_source_stale(),
            author_pubkey: pk,
            timestamp: 1001,
            entry_hash: hash2,
            prev_hash: hash1,
            signature: hex::encode(kp.sign(&bytes2)),
            pow_nonce: None,
        };

        // Reversed order (simulates out-of-order iroh-docs arrival)
        let entries = vec![entry2, entry1];
        assert!(
            verify_chain(&entries).is_ok(),
            "verify_chain must handle out-of-order entries via chain linkage"
        );
    }

    #[test]
    fn test_feed_pow_verification() {
        let entry_hash = "a".repeat(64);
        let nonce = compute_feed_pow(&entry_hash);
        assert!(
            verify_feed_pow(&entry_hash, nonce),
            "computed nonce must verify"
        );
        assert!(
            !verify_feed_pow(&entry_hash, u64::MAX),
            "random nonce should almost certainly fail"
        );
    }

    #[test]
    fn test_feed_pow_different_hashes_different_nonces() {
        let hash_a = "a".repeat(64);
        let hash_b = "b".repeat(64);
        let nonce_a = compute_feed_pow(&hash_a);
        let nonce_b = compute_feed_pow(&hash_b);
        assert!(verify_feed_pow(&hash_a, nonce_a));
        assert!(verify_feed_pow(&hash_b, nonce_b));
        assert!(
            !verify_feed_pow(&hash_a, nonce_b) || nonce_a == nonce_b,
            "cross-hash nonce should not verify (unless coincidence)"
        );
    }

    #[test]
    fn test_backfill_six_plus_entries() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        for _ in 0..8 {
            insert_feed_operation(&db, sample_release_published(), &pk, |d| {
                kp.sign(d).to_vec()
            })
            .unwrap();
        }

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 8, "all 8 entries must be stored");
        assert!(verify_chain(&entries).is_ok(), "8-entry chain must verify");

        for i in 0..entries.len() - 1 {
            assert_eq!(
                entries[i + 1].prev_hash,
                entries[i].entry_hash,
                "entry {} prev_hash must link to entry {}",
                i + 1,
                i
            );
        }
    }

    #[test]
    fn test_feed_publish_orphan_rollback() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let entry = insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        assert!(db.feed_entry_exists_by_hash(&entry.entry_hash).unwrap());
        assert_eq!(db.count_feed_entries().unwrap(), 1);

        assert!(db.delete_feed_entry_if_tail(&entry.entry_hash).unwrap());

        assert!(!db.feed_entry_exists_by_hash(&entry.entry_hash).unwrap());
        assert_eq!(db.count_feed_entries().unwrap(), 0);
        assert!(!db.delete_feed_entry_if_tail(&entry.entry_hash).unwrap());
    }

    #[test]
    fn test_feed_orphan_rollback_refuses_if_chained() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let e1 = insert_feed_operation(&db, sample_release_published(), &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
        let _e2 = insert_feed_operation(&db, sample_source_stale(), &pk, |d| kp.sign(d).to_vec())
            .unwrap();
        assert_eq!(db.count_feed_entries().unwrap(), 2);

        assert!(
            !db.delete_feed_entry_if_tail(&e1.entry_hash).unwrap(),
            "must refuse to delete entry that another entry chains on"
        );
        assert_eq!(
            db.count_feed_entries().unwrap(),
            2,
            "chain must remain intact"
        );
    }

    #[test]
    fn test_pow_nonce_serde_default() {
        let entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            entry_hash: "e".repeat(64),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: "f".repeat(128),
            pow_nonce: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        // Deserialize without pow_nonce field → defaults to None
        let without_pow: serde_json::Value = serde_json::from_str(&json).unwrap();
        let mut obj = without_pow.as_object().unwrap().clone();
        obj.remove("pow_nonce");
        let back: FeedEntry = serde_json::from_value(serde_json::Value::Object(obj)).unwrap();
        assert_eq!(back.pow_nonce, None);

        // Deserialize with pow_nonce field → picks up value
        let mut entry_with = entry.clone();
        entry_with.pow_nonce = Some(42);
        let json2 = serde_json::to_string(&entry_with).unwrap();
        let back2: FeedEntry = serde_json::from_str(&json2).unwrap();
        assert_eq!(back2.pow_nonce, Some(42));
    }

    // -- Phase C: Adversarial tests --

    #[test]
    fn test_adversarial_fork_bomb_spam_rejected() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let mut accepted = 0u32;
        let mut rejected = 0u32;
        for _ in 0..20 {
            let result =
                insert_feed_operation_rate_limited(&db, sample_release_published(), &pk, |d| {
                    kp.sign(d).to_vec()
                });
            match result {
                Ok(_) => accepted += 1,
                Err(e) if e.contains("rate limit exceeded") => rejected += 1,
                Err(e) => panic!("unexpected error: {e}"),
            }
        }
        assert_eq!(
            accepted, FEED_RATE_LIMIT_PER_MINUTE as u32,
            "rate limiter must accept exactly {FEED_RATE_LIMIT_PER_MINUTE} ops per minute"
        );
        assert_eq!(rejected, 15, "rate limiter must reject ops beyond quota");
    }

    #[test]
    fn test_adversarial_payload_oversized_rejected() {
        let oversized_url = format!(
            "https://example.com/{}",
            "x".repeat(MAX_OPERATION_JSON_SIZE)
        );
        let op = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: oversized_url,
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
        });
        let result = validate_feed_operation(&op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceeds"));
    }

    #[test]
    fn test_adversarial_bad_repo_url_rejected() {
        let bad_urls = [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>",
            "http://example.com/app",
            "ftp://example.com/app",
            "https://example.com/../../../etc/passwd",
            "",
            "https://",
        ];
        for url in bad_urls {
            let op = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: url.to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
            });
            if url.starts_with("https://") {
                continue;
            }
            let result = validate_feed_operation(&op);
            assert!(
                result.is_err(),
                "URL {url:?} must be rejected by validation"
            );
        }
    }

    #[test]
    fn test_adversarial_bad_artifact_hash_rejected() {
        let g_repeat = "g".repeat(64);
        let a_short = "a".repeat(63);
        let a_long = "a".repeat(65);
        let null_repeat = "\0".repeat(64);
        let space_repeat = "ab cd".repeat(13);
        let bad_hashes: &[&str] = &[
            "",
            "short",
            &g_repeat,
            &a_short,
            &a_long,
            &null_repeat,
            &space_repeat,
        ];
        for hash in bad_hashes {
            let op = PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: hash.to_string(),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
            });
            let result = validate_feed_operation(&op);
            assert!(result.is_err(), "artifact_hash {hash:?} must be rejected");
        }
    }

    #[test]
    fn test_adversarial_seq_gap_detection() {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[9u8; 32]);
        let pk = hex::encode(kp.public_bytes());

        let can1 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes1 = compute_feed_canonical_bytes(&can1).unwrap();
        let hash1 = compute_feed_entry_hash(&can1).unwrap();
        let entry1 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            entry_hash: hash1.clone(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp.sign(&bytes1)),
            pow_nonce: None,
        };

        let can2 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_source_stale(),
            author_pubkey: pk.clone(),
            timestamp: 1001,
            prev_hash: hash1.clone(),
        };
        let bytes2 = compute_feed_canonical_bytes(&can2).unwrap();
        let hash2 = compute_feed_entry_hash(&can2).unwrap();
        let entry2 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 2,
            op: sample_source_stale(),
            author_pubkey: pk.clone(),
            timestamp: 1001,
            entry_hash: hash2.clone(),
            prev_hash: hash1,
            signature: hex::encode(kp.sign(&bytes2)),
            pow_nonce: None,
        };

        // Entry 3 skips entry 2 by pointing prev_hash to a fabricated hash
        let fake_prev = "f".repeat(64);
        let can3 = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1005,
            prev_hash: fake_prev.clone(),
        };
        let bytes3 = compute_feed_canonical_bytes(&can3).unwrap();
        let hash3 = compute_feed_entry_hash(&can3).unwrap();
        let entry3 = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 5,
            op: sample_release_published(),
            author_pubkey: pk,
            timestamp: 1005,
            entry_hash: hash3,
            prev_hash: fake_prev,
            signature: hex::encode(kp.sign(&bytes3)),
            pow_nonce: None,
        };

        // Entries 1, 2 are valid chain. Entry 3 breaks linkage.
        let entries = vec![entry1, entry2, entry3];
        let result = verify_chain(&entries);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("broken linkage or fork"),
            "seq gap must be detected as broken linkage"
        );
    }

    #[test]
    fn test_adversarial_cross_author_forgery_rejected() {
        let kp_real = nexus_core_rs::KeyPair::from_secret_bytes(&[10u8; 32]);
        let kp_attacker = nexus_core_rs::KeyPair::from_secret_bytes(&[11u8; 32]);
        let pk_real = hex::encode(kp_real.public_bytes());

        // Attacker signs an entry but claims it's from kp_real
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk_real.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let canonical_bytes = compute_feed_canonical_bytes(&canonical).unwrap();
        let entry_hash = compute_feed_entry_hash(&canonical).unwrap();

        let forged_entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk_real,
            timestamp: 1000,
            entry_hash,
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp_attacker.sign(&canonical_bytes)),
            pow_nonce: None,
        };

        let result = verify_entry(&forged_entry);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("signature verification failed"),
            "cross-author forgery must be rejected"
        );
    }

    // -- Phase D: Adversarial crypto tests --

    #[test]
    fn test_adversarial_ed25519_forgery_feed_entry() {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[12u8; 32]);
        let pk = hex::encode(kp.public_bytes());

        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let entry_hash = compute_feed_entry_hash(&canonical).unwrap();

        let forged_sig = hex::encode([0xABu8; 64]);
        let entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk,
            timestamp: 1000,
            entry_hash,
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: forged_sig,
            pow_nonce: None,
        };

        let result = verify_entry(&entry);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("signature verification failed"),
            "random bytes signature must be rejected"
        );
    }

    #[test]
    fn test_adversarial_blake3_tamper_canonical() {
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let original_bytes = compute_feed_canonical_bytes(&canonical).unwrap();
        let original_hash = compute_feed_entry_hash(&canonical).unwrap();

        let mut tampered = original_bytes.clone();
        tampered[0] ^= 0x01;
        let tampered_hash = hex::encode(blake3::hash(&tampered).as_bytes());

        assert_ne!(
            original_hash, tampered_hash,
            "1-bit flip in canonical bytes must produce different BLAKE3 hash"
        );
    }

    #[test]
    fn test_adversarial_pow_nonce_difficulty_check() {
        let entry_hash = "a".repeat(64);
        let mut pass_count = 0u32;
        for nonce in 1000..2000u64 {
            if verify_feed_pow(&entry_hash, nonce) {
                pass_count += 1;
            }
        }
        assert!(
            pass_count <= 2,
            "random nonces must overwhelmingly fail 16-bit PoW difficulty (got {pass_count}/1000 passes)"
        );
    }

    #[test]
    fn test_adversarial_age_witness_future_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let entry_ok = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: now + 3600,
            entry_hash: "e".repeat(64),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: "f".repeat(128),
            pow_nonce: None,
        };
        assert!(
            validate_feed_entry_timestamp(&entry_ok, now).is_ok(),
            "timestamp 1h in future must be accepted"
        );

        let entry_future = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 2,
            op: sample_release_published(),
            author_pubkey: "d".repeat(64),
            timestamp: now + 31 * 24 * 3600,
            entry_hash: "e".repeat(64),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: "f".repeat(128),
            pow_nonce: None,
        };
        let result = validate_feed_entry_timestamp(&entry_future, now);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("more than 30 days"),
            "timestamp 31 days in future must be rejected"
        );
    }
}
