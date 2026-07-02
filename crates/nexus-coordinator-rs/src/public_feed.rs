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
use serde_json::Value;

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
    /// Human-readable app name (Sprint 75 Phase C, WIRE-1). Additive and
    /// 0-bump: a `ReleasePublished` op historically carried no name, so the FTS5
    /// index left search-by-name empty for the feed path (only the gossip
    /// `ProjectAnnouncement` path indexed a name). A producer now sets this so a
    /// release becomes full-text searchable by name; an op without it
    /// deserializes to `None` and serializes to byte-identical output
    /// (`skip_serializing_if`), preserving the pre-launch additive policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
    /// Free-form category tag (Sprint 75 Phase C, WIRE-1). Same additive 0-bump
    /// shape as [`Self::project_name`]; lets the FTS5 index match a release by
    /// category. `None` for any op that omits it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// Payload for a source-became-stale event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceBecameStalePayload {
    pub project_id: String,
    pub reason: String,
}

/// Payload for a curator-vouched event.
///
/// Records that a curator publicly endorses a project. The
/// `curator_pubkey` is the Ed25519 public key of the curator
/// who vouches, validated as hex-64.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuratorVouchedPayload {
    pub project_id: String,
    pub curator_pubkey: String,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Payload for a curator-disendorsed event.
///
/// Records that a curator withdraws endorsement of a project.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CuratorDisendorsedPayload {
    pub project_id: String,
    pub curator_pubkey: String,
    #[serde(default)]
    pub reason: Option<String>,
}

/// Payload for a seed-announced event (Sprint 74 Phase F).
///
/// Records that a node holds and serves an app's archive blob, so a
/// best-effort "Toi + N pairs" availability count can aggregate the
/// seeders of a project. `seeder_node_id` is the announcing node's
/// public key and MUST equal the `FeedEntry.author_pubkey` that signs
/// the entry (the seeder signs only its OWN seed claim — it is
/// DISTINCT from the app author and never re-attributes authorship,
/// R5 / Radicle delegate!=seeder). `archive_hash` is the BLAKE3 of the
/// blob the seeder holds; content-addressing remains the truth of
/// reachability (a forged announcement cannot let a node serve bytes
/// it does not have), so the count may over-state but never lies about
/// a fetch succeeding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedAnnouncedPayload {
    pub project_id: String,
    pub seeder_node_id: String,
    pub archive_hash: String,
}

/// Discriminated union of all public feed operation types.
///
/// `ReleasePublished` and `SourceBecameStale` since Sprint 1.
/// `CuratorVouched` and `CuratorDisendorsed` since Sprint 67.
/// `SeedAnnounced` since Sprint 74 Phase F.
/// Future variants (`BuildQuorumReached`, `SourceRecovered`,
/// `SearchManifestPublished`) use the raw-op forward compat
/// path (pattern P51) until implemented.
///
/// Adding a typed variant does NOT bump `FEED_FORMAT_VERSION` — the
/// wire-format version lives on the `FeedEntry` envelope (`version`),
/// not on this op union (S67 precedent: `CuratorVouched`/
/// `CuratorDisendorsed` were added as variants with 0 bump). A typed
/// variant is preferred over a pure raw-op `Value` for an op whose
/// fields must be validated at insert time (it gives
/// `validate_known_operation` a place to reject malformed payloads
/// instead of storing opaque junk).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op_type")]
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
    CuratorVouched(CuratorVouchedPayload),
    CuratorDisendorsed(CuratorDisendorsedPayload),
    SeedAnnounced(SeedAnnouncedPayload),
}

// ---------------------------------------------------------------------------
// Feed entry (stored + transmitted)
// ---------------------------------------------------------------------------

/// A single entry in the public feed log.
///
/// `entry_hash` and `signature` are computed from the canonical
/// representation of `FeedEntryCanonical`.
///
/// `op` is a raw `serde_json::Value` for forward compatibility:
/// nodes store and propagate unknown operation types without
/// interpretation (CloudEvents-style extensibility). Use
/// [`try_parse_op`] to attempt typed deserialization of known ops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedEntry {
    pub version: u16,
    pub seq: u64,
    pub op: Value,
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
    pub op: Value,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub prev_hash: String,
}

/// Try to parse a raw `op` Value into a known `PublicFeedOperation`.
/// Returns `None` for unknown operation types (forward compat).
pub fn try_parse_op(op: &Value) -> Option<PublicFeedOperation> {
    serde_json::from_value(op.clone()).ok()
}

/// Extract the `op_type` discriminant from a raw op Value.
pub fn op_type(op: &Value) -> Option<&str> {
    op.get("op_type").and_then(|v| v.as_str())
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
/// For known ops (parseable via [`try_parse_op`]): validates
/// project_id hex-64, repo_url HTTPS, commit_sha hex-40,
/// artifact_hash hex-64, reason in the protocol allowlist,
/// `is_open_source: true` requires provenance_hash (spec §2.1).
///
/// For unknown ops: accepts with size check only (store + forward).
/// Size: serialized JSON must not exceed `MAX_OPERATION_JSON_SIZE`.
///
/// A KNOWN `op_type` that fails to parse into its typed payload (a missing or
/// wrong-typed required field) is REJECTED as malformed rather than stored as an
/// opaque "unknown" op — otherwise a peer could smuggle junk under a recognised
/// discriminant that downstream consumers would silently drop. Genuinely unknown
/// `op_type`s still pass with a size check only (raw-op forward compat, P51).
pub fn validate_feed_operation(op: &Value) -> Result<(), String> {
    let json = serde_json::to_string(op).map_err(|e| format!("payload serialization: {e}"))?;
    if json.len() > MAX_OPERATION_JSON_SIZE {
        return Err(format!(
            "operation payload exceeds {} bytes limit",
            MAX_OPERATION_JSON_SIZE
        ));
    }
    match try_parse_op(op) {
        Some(typed) => validate_known_operation(&typed)?,
        None => {
            if let Some(ot) = op_type(op) {
                if KNOWN_OP_TYPES.contains(&ot) {
                    return Err(format!(
                        "malformed {ot} operation: known op_type failed to parse \
                         (missing or wrong-typed required field)"
                    ));
                }
            }
        }
    }
    // F-3 (Sprint 74 Phase F): a SeedAnnounced op carries NO payload-level
    // signature — the signature is the FeedEntry-level Ed25519 over
    // DOMAIN_FEED_V1. The internally-tagged enum ignores unknown keys on parse
    // (serde does not support deny_unknown_fields with `#[serde(tag)]`), so a
    // remote op could smuggle a spurious `sig` (or any extra) key that survives
    // into the stored raw `op`. Enforce the exact key set so the invariant holds
    // on the wire, not just at the producer.
    if op_type(op) == Some("SeedAnnounced") {
        if let Some(obj) = op.as_object() {
            const ALLOWED: &[&str] = &["op_type", "project_id", "seeder_node_id", "archive_hash"];
            if let Some(extra) = obj.keys().find(|k| !ALLOWED.contains(&k.as_str())) {
                return Err(format!(
                    "SeedAnnounced op carries an unexpected field '{extra}' \
                     (allowed: op_type, project_id, seeder_node_id, archive_hash; \
                     no payload-level sig — F-3)"
                ));
            }
        }
    }
    Ok(())
}

/// The set of `op_type` discriminants this build knows how to parse + validate.
/// A wire op whose `op_type` is in this set MUST parse into its typed payload
/// (else it is malformed and rejected); an `op_type` NOT in this set is treated
/// as a forward-compat unknown op (stored + forwarded, never interpreted).
const KNOWN_OP_TYPES: &[&str] = &[
    "ReleasePublished",
    "SourceBecameStale",
    "CuratorVouched",
    "CuratorDisendorsed",
    "SeedAnnounced",
];

fn validate_known_operation(op: &PublicFeedOperation) -> Result<(), String> {
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
        PublicFeedOperation::CuratorVouched(p) => {
            if !is_hex_exact(&p.project_id, 64) {
                return Err("project_id must be 64 hex characters".to_string());
            }
            if !is_hex_exact(&p.curator_pubkey, 64) {
                return Err("curator_pubkey must be 64 hex characters".to_string());
            }
        }
        PublicFeedOperation::CuratorDisendorsed(p) => {
            if !is_hex_exact(&p.project_id, 64) {
                return Err("project_id must be 64 hex characters".to_string());
            }
            if !is_hex_exact(&p.curator_pubkey, 64) {
                return Err("curator_pubkey must be 64 hex characters".to_string());
            }
        }
        PublicFeedOperation::SeedAnnounced(p) => {
            if !is_hex_exact(&p.project_id, 64) {
                return Err("project_id must be 64 hex characters".to_string());
            }
            if !is_hex_exact(&p.seeder_node_id, 64) {
                return Err("seeder_node_id must be 64 hex characters".to_string());
            }
            if !is_hex_exact(&p.archive_hash, 64) {
                return Err("archive_hash must be 64 hex characters".to_string());
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
    op: Value,
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
    op: Value,
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
    op: Value,
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

    let op_type_str = op_type(&op).unwrap_or("Unknown").to_string();
    let payload = serde_json::to_string(&op).map_err(|e| format!("payload serialization: {e}"))?;

    let row = crate::db::FeedEntryRow {
        seq: 0,
        op_type: op_type_str,
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
        let op: Value =
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
/// Rejects entries with `version != FEED_FORMAT_VERSION`, and
/// enforces the `prev_hash` FORMAT (genesis sentinel or lowercase
/// hex-64). Does NOT check prev_hash linkage or existence — this
/// function runs on freshly received entries BEFORE insert
/// (`feed_sync` ingest), where the predecessor may legitimately not
/// have arrived yet (out-of-order iroh-docs sync). Linkage is the job
/// of [`verify_chain`] / the materializer fold over the full set.
/// Note the format guard is a minor hardening only: `prev_hash` is
/// part of the signed canonical form, so any post-signature tampering
/// is already caught by the entry_hash recomputation below.
pub fn verify_entry(entry: &FeedEntry) -> Result<(), String> {
    if entry.version != FEED_FORMAT_VERSION {
        return Err(format!(
            "unsupported feed version {}, expected {}",
            entry.version, FEED_FORMAT_VERSION
        ));
    }

    if entry.prev_hash != GENESIS_PREV_HASH
        && (entry.prev_hash.len() != 64
            || !entry
                .prev_hash
                .bytes()
                .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')))
    {
        return Err(format!(
            "entry seq {}: malformed prev_hash (expected \"genesis\" or lowercase hex-64)",
            entry.seq
        ));
    }

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

    fn sample_release_published_typed() -> PublicFeedOperation {
        PublicFeedOperation::ReleasePublished(ReleasePublishedPayload {
            project_id: "a1".repeat(32),
            repo_url: "https://github.com/org/app".to_string(),
            commit_sha: "a".repeat(40),
            artifact_hash: "b".repeat(64),
            provenance_hash: Some("c".repeat(64)),
            is_open_source: true,
            project_name: None,
            category: None,
        })
    }

    fn sample_source_stale_typed() -> PublicFeedOperation {
        PublicFeedOperation::SourceBecameStale(SourceBecameStalePayload {
            project_id: "a1".repeat(32),
            reason: "repo_unreachable".to_string(),
        })
    }

    fn sample_release_published() -> Value {
        serde_json::to_value(sample_release_published_typed()).unwrap()
    }

    fn sample_source_stale() -> Value {
        serde_json::to_value(sample_source_stale_typed()).unwrap()
    }

    #[test]
    fn test_feed_operation_serde_roundtrip() {
        let ops = vec![
            sample_release_published_typed(),
            sample_source_stale_typed(),
        ];
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

    fn sample_seed_announced_typed() -> PublicFeedOperation {
        PublicFeedOperation::SeedAnnounced(SeedAnnouncedPayload {
            project_id: "a1".repeat(32),
            seeder_node_id: "b2".repeat(32),
            archive_hash: "c3".repeat(32),
        })
    }

    #[test]
    fn seed_announced_raw_op_no_version_bump() {
        // Adding the SeedAnnounced typed variant rides the UNCHANGED FeedEntry
        // envelope under DOMAIN_FEED_V1 — it is a new op, not a wire-format
        // break. Per the pre-launch protocol policy, a new operation does NOT
        // bump FEED_FORMAT_VERSION (S67 CuratorVouched/CuratorDisendorsed
        // precedent: still 1 after those additions).
        let op = serde_json::to_value(sample_seed_announced_typed()).unwrap();

        // 1. The op carries the expected discriminant + fields on the wire.
        assert_eq!(op_type(&op), Some("SeedAnnounced"));
        assert_eq!(op.get("project_id").unwrap().as_str().unwrap().len(), 64);
        assert_eq!(
            op.get("seeder_node_id").unwrap().as_str().unwrap().len(),
            64
        );
        assert_eq!(op.get("archive_hash").unwrap().as_str().unwrap().len(), 64);
        // The signature lives at the FeedEntry level (F-3): the op payload
        // carries NO `sig` field of its own.
        assert!(op.get("sig").is_none());

        // 2. It validates (insert-time field validation on the typed variant).
        assert!(validate_feed_operation(&op).is_ok());

        // 3. It round-trips through the typed enum (op_type routing works).
        assert_eq!(try_parse_op(&op), Some(sample_seed_announced_typed()));

        // 4. The wire-format version is unchanged after building/validating it.
        assert_eq!(FEED_FORMAT_VERSION, 1);

        // 5. A malformed SeedAnnounced is REJECTED at validate (not stored as
        //    junk that would pollute the count) — the typed variant's gain.
        let bad_hex = serde_json::json!({
            "op_type": "SeedAnnounced",
            "project_id": "too-short",
            "seeder_node_id": "b2".repeat(32),
            "archive_hash": "c3".repeat(32),
        });
        assert!(validate_feed_operation(&bad_hex).is_err());

        // 6. A SeedAnnounced MISSING a required field fails the typed parse; it
        //    must be rejected as a malformed KNOWN op, NOT stored as an opaque
        //    unknown op (C1 — known op_type that fails to parse).
        let missing_field = serde_json::json!({
            "op_type": "SeedAnnounced",
            "project_id": "a1".repeat(32),
            "seeder_node_id": "b2".repeat(32),
        });
        assert!(try_parse_op(&missing_field).is_none());
        let err = validate_feed_operation(&missing_field).unwrap_err();
        assert!(err.contains("malformed SeedAnnounced"), "{err}");

        // 7. A SeedAnnounced smuggling a payload-level `sig` (or any extra key)
        //    is rejected — F-3: the signature lives at the FeedEntry level only.
        let with_sig = serde_json::json!({
            "op_type": "SeedAnnounced",
            "project_id": "a1".repeat(32),
            "seeder_node_id": "b2".repeat(32),
            "archive_hash": "c3".repeat(32),
            "sig": "de".repeat(64),
        });
        let err = validate_feed_operation(&with_sig).unwrap_err();
        assert!(err.contains("unexpected field 'sig'"), "{err}");

        // 8. A genuinely UNKNOWN op_type still passes (forward compat, P51).
        let future = serde_json::json!({ "op_type": "FutureSeedThing", "x": 1 });
        assert!(validate_feed_operation(&future).is_ok());
    }

    #[test]
    fn test_compute_feed_entry_hash_deterministic() {
        // Spec §7 test vector — inline data to keep vector stable.
        // The test vector was computed with the typed enum; the raw-op
        // migration (S65) preserves the same JCS output so the hash
        // MUST remain identical.
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: serde_json::to_value(PublicFeedOperation::ReleasePublished(
                ReleasePublishedPayload {
                    project_id: "abc123def456".to_string(),
                    repo_url: "https://github.com/org/app".to_string(),
                    commit_sha: "a".repeat(40),
                    artifact_hash: "b".repeat(64),
                    provenance_hash: Some("c".repeat(64)),
                    is_open_source: true,
                    project_name: None,
                    category: None,
                },
            ))
            .unwrap(),
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
        let op = serde_json::to_value(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: None,
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ))
        .unwrap();
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
        let to_val = |op: PublicFeedOperation| -> Value { serde_json::to_value(op).unwrap() };

        // project_id not hex-64
        let bad_pid = to_val(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "short".to_string(),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ));
        assert!(
            validate_feed_operation(&bad_pid)
                .unwrap_err()
                .contains("project_id")
        );

        // repo_url not HTTPS
        let bad_url = to_val(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: "http://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ));
        assert!(
            validate_feed_operation(&bad_url)
                .unwrap_err()
                .contains("repo_url")
        );

        // commit_sha not hex-40
        let bad_sha = to_val(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "z".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ));
        assert!(
            validate_feed_operation(&bad_sha)
                .unwrap_err()
                .contains("commit_sha")
        );

        // artifact_hash not hex-64
        let bad_art = to_val(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: "https://github.com/org/app".to_string(),
                commit_sha: "a".repeat(40),
                artifact_hash: "short".to_string(),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ));
        assert!(
            validate_feed_operation(&bad_art)
                .unwrap_err()
                .contains("artifact_hash")
        );

        // SourceBecameStale empty reason
        let bad_reason = to_val(PublicFeedOperation::SourceBecameStale(
            SourceBecameStalePayload {
                project_id: "a1".repeat(32),
                reason: "".to_string(),
            },
        ));
        assert!(
            validate_feed_operation(&bad_reason)
                .unwrap_err()
                .contains("reason")
        );

        // SourceBecameStale unknown reason
        let bad_unknown_reason = to_val(PublicFeedOperation::SourceBecameStale(
            SourceBecameStalePayload {
                project_id: "a1".repeat(32),
                reason: "unknown".to_string(),
            },
        ));
        assert!(
            validate_feed_operation(&bad_unknown_reason)
                .unwrap_err()
                .contains("reason")
        );

        // SourceBecameStale bad project_id
        let bad_stale_pid = to_val(PublicFeedOperation::SourceBecameStale(
            SourceBecameStalePayload {
                project_id: "not-hex".to_string(),
                reason: "repo_unreachable".to_string(),
            },
        ));
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
                validate_feed_operation(&to_val(PublicFeedOperation::SourceBecameStale(
                    SourceBecameStalePayload {
                        project_id: "a1".repeat(32),
                        reason: (*reason).to_string(),
                    },
                )))
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
    fn test_verify_entry_prev_hash_format() {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[9u8; 32]);
        let pk = hex::encode(kp.public_bytes());

        let signed_with_prev = |prev_hash: &str| -> FeedEntry {
            let canonical = FeedEntryCanonical {
                version: FEED_FORMAT_VERSION,
                op: sample_release_published(),
                author_pubkey: pk.clone(),
                timestamp: 1000,
                prev_hash: prev_hash.to_string(),
            };
            let bytes = compute_feed_canonical_bytes(&canonical).unwrap();
            FeedEntry {
                version: FEED_FORMAT_VERSION,
                seq: 1,
                op: sample_release_published(),
                author_pubkey: pk.clone(),
                timestamp: 1000,
                entry_hash: compute_feed_entry_hash(&canonical).unwrap(),
                prev_hash: prev_hash.to_string(),
                signature: hex::encode(kp.sign(&bytes)),
                pow_nonce: None,
            }
        };

        // Malformed prev_hash is rejected even when signed coherently
        // (wf4 format guard: genesis sentinel or lowercase hex-64).
        let upper = "A".repeat(64);
        let short = "a".repeat(63);
        for bad in ["", "xyz", upper.as_str(), short.as_str()] {
            let err = verify_entry(&signed_with_prev(bad)).unwrap_err();
            assert!(
                err.contains("malformed prev_hash"),
                "prev_hash {bad:?} must be rejected as malformed, got: {err}"
            );
        }

        // An out-of-order entry (well-formed prev_hash whose
        // predecessor has NOT arrived yet) must be ACCEPTED —
        // verify_entry never checks existence/linkage, otherwise the
        // out-of-order iroh-docs ingest path would break.
        let unknown_predecessor = "c".repeat(64);
        assert!(verify_entry(&signed_with_prev(&unknown_predecessor)).is_ok());
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
        let op = serde_json::to_value(PublicFeedOperation::ReleasePublished(
            ReleasePublishedPayload {
                project_id: "a1".repeat(32),
                repo_url: oversized_url,
                commit_sha: "a".repeat(40),
                artifact_hash: "b".repeat(64),
                provenance_hash: Some("c".repeat(64)),
                is_open_source: true,
                project_name: None,
                category: None,
            },
        ))
        .unwrap();
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
            let op = serde_json::to_value(PublicFeedOperation::ReleasePublished(
                ReleasePublishedPayload {
                    project_id: "a1".repeat(32),
                    repo_url: url.to_string(),
                    commit_sha: "a".repeat(40),
                    artifact_hash: "b".repeat(64),
                    provenance_hash: Some("c".repeat(64)),
                    is_open_source: true,
                    project_name: None,
                    category: None,
                },
            ))
            .unwrap();
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
            let op = serde_json::to_value(PublicFeedOperation::ReleasePublished(
                ReleasePublishedPayload {
                    project_id: "a1".repeat(32),
                    repo_url: "https://github.com/org/app".to_string(),
                    commit_sha: "a".repeat(40),
                    artifact_hash: hash.to_string(),
                    provenance_hash: Some("c".repeat(64)),
                    is_open_source: true,
                    project_name: None,
                    category: None,
                },
            ))
            .unwrap();
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
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[13u8; 32]);
        let pk = hex::encode(kp.public_bytes());

        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let canonical_bytes = compute_feed_canonical_bytes(&canonical).unwrap();
        let entry_hash = compute_feed_entry_hash(&canonical).unwrap();
        let signature = hex::encode(kp.sign(&canonical_bytes));

        // Valid entry passes verify_entry
        let valid_entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1_700_000_000,
            entry_hash: entry_hash.clone(),
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: signature.clone(),
            pow_nonce: None,
        };
        assert!(verify_entry(&valid_entry).is_ok(), "valid entry must pass");

        // Tampered entry: change timestamp (1 bit flip in canonical input)
        // This makes the stored entry_hash not match recomputed hash
        let tampered_entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk,
            timestamp: 1_700_000_001,
            entry_hash,
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature,
            pow_nonce: None,
        };
        let result = verify_entry(&tampered_entry);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("entry_hash mismatch"),
            "tampered canonical field must cause hash mismatch in verify_entry"
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
    fn test_adversarial_future_timestamp_rejected() {
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

    // -- Sprint 65 Phase A: raw-op migration + version guard --

    #[test]
    fn test_verify_entry_rejects_wrong_version() {
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);
        let canonical = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: sample_release_published(),
            author_pubkey: pk.clone(),
            timestamp: 1000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let bytes = compute_feed_canonical_bytes(&canonical).unwrap();
        let hash = compute_feed_entry_hash(&canonical).unwrap();
        let mut entry = FeedEntry {
            version: FEED_FORMAT_VERSION,
            seq: 1,
            op: sample_release_published(),
            author_pubkey: pk,
            timestamp: 1000,
            entry_hash: hash,
            prev_hash: GENESIS_PREV_HASH.to_string(),
            signature: hex::encode(kp.sign(&bytes)),
            pow_nonce: None,
        };
        assert!(verify_entry(&entry).is_ok());
        entry.version = 99;
        let result = verify_entry(&entry);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unsupported feed version"));
    }

    #[test]
    fn test_unknown_op_roundtrip() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let unknown_op = serde_json::json!({
            "op_type": "BuildQuorumReached",
            "project_id": "a1".repeat(32),
            "quorum": 3
        });
        let entry =
            insert_feed_operation(&db, unknown_op.clone(), &pk, |d| kp.sign(d).to_vec()).unwrap();
        assert_eq!(entry.seq, 1);
        assert_eq!(op_type(&entry.op), Some("BuildQuorumReached"));
        assert!(try_parse_op(&entry.op).is_none());

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].op, unknown_op);
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_canonical_bytes_value_vs_typed() {
        let typed_op = sample_release_published_typed();
        let value_op = serde_json::to_value(&typed_op).unwrap();

        let canonical_typed = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: serde_json::to_value(&typed_op).unwrap(),
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };
        let canonical_value = FeedEntryCanonical {
            version: FEED_FORMAT_VERSION,
            op: value_op,
            author_pubkey: "d".repeat(64),
            timestamp: 1_700_000_000,
            prev_hash: GENESIS_PREV_HASH.to_string(),
        };

        let bytes_typed = compute_feed_canonical_bytes(&canonical_typed).unwrap();
        let bytes_value = compute_feed_canonical_bytes(&canonical_value).unwrap();
        assert_eq!(
            bytes_typed, bytes_value,
            "canonical bytes must be identical for typed and Value ops"
        );

        let hash_typed = compute_feed_entry_hash(&canonical_typed).unwrap();
        let hash_value = compute_feed_entry_hash(&canonical_value).unwrap();
        assert_eq!(hash_typed, hash_value);
    }

    // -- Sprint 67 Phase A: CuratorVouched / CuratorDisendorsed --

    fn sample_curator_vouched() -> PublicFeedOperation {
        PublicFeedOperation::CuratorVouched(CuratorVouchedPayload {
            project_id: "a1".repeat(32),
            curator_pubkey: "d".repeat(64),
            reason: Some("quality project".into()),
        })
    }

    fn sample_curator_disendorsed() -> PublicFeedOperation {
        PublicFeedOperation::CuratorDisendorsed(CuratorDisendorsedPayload {
            project_id: "a1".repeat(32),
            curator_pubkey: "d".repeat(64),
            reason: Some("inactive".into()),
        })
    }

    #[test]
    fn test_curator_vouched_roundtrip() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let op_val = serde_json::to_value(sample_curator_vouched()).unwrap();
        let entry =
            insert_feed_operation(&db, op_val.clone(), &pk, |d| kp.sign(d).to_vec()).unwrap();
        assert_eq!(entry.seq, 1);
        assert_eq!(op_type(&entry.op), Some("CuratorVouched"));
        let parsed = try_parse_op(&entry.op).unwrap();
        assert_eq!(parsed, sample_curator_vouched());

        let entries = replay_all(&db).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(verify_chain(&entries).is_ok());
    }

    #[test]
    fn test_curator_disendorsed_roundtrip() {
        let db = CoordinatorDb::open_in_memory().unwrap();
        let kp = test_keypair();
        let pk = pubkey_hex(&kp);

        let op_val = serde_json::to_value(sample_curator_disendorsed()).unwrap();
        let entry = insert_feed_operation(&db, op_val, &pk, |d| kp.sign(d).to_vec()).unwrap();
        assert_eq!(entry.seq, 1);
        assert_eq!(op_type(&entry.op), Some("CuratorDisendorsed"));
        let parsed = try_parse_op(&entry.op).unwrap();
        assert_eq!(parsed, sample_curator_disendorsed());
    }

    #[test]
    fn test_curator_vouched_validation_rejects_bad_pubkey() {
        let op = serde_json::to_value(PublicFeedOperation::CuratorVouched(CuratorVouchedPayload {
            project_id: "a1".repeat(32),
            curator_pubkey: "short".into(),
            reason: None,
        }))
        .unwrap();
        let result = validate_feed_operation(&op);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("curator_pubkey"));
    }

    #[test]
    fn test_curator_vouched_unknown_op_forward_compat() {
        let unknown = serde_json::json!({
            "op_type": "FutureOp",
            "data": "whatever"
        });
        assert!(try_parse_op(&unknown).is_none());
        assert!(validate_feed_operation(&unknown).is_ok());
    }
}
