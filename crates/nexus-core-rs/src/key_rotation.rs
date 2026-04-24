// SPDX-License-Identifier: AGPL-3.0-or-later
//! Key rotation ceremony for Ed25519 identities.
//!
//! A node operator (curator, coordinator, worker) rotates their
//! Ed25519 identity by publishing a [`KeyRotationAnnouncement`]
//! **signed by the old key**. The announcement names the new public
//! key and a transition window (default 7 days) during which both
//! keys are accepted. After the window expires, the old key is
//! fully revoked.
//!
//! The [`RevocationCache`] tracks applied announcements in memory.
//! Pre-v1.0 there are zero external nodes, so persistence (SQLite)
//! is deferred to S26.
//!
//! Wire format: `KEY_ROTATION_FORMAT_VERSION = 1`, domain
//! [`DOMAIN_KEY_ROTATION_V1`]. Gossip topic:
//! `nexus-grid/key-rotation/v1`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{canonical_bytes, DOMAIN_KEY_ROTATION_V1};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Current on-wire version for key rotation announcements.
///
/// Pre-launch policy: stays at 1 until tag v1.0. No tolerant
/// decoder. `#[serde(default)]` only for runtime robustness.
pub const KEY_ROTATION_FORMAT_VERSION: u16 = 1;

/// Default transition window in days. Both old and new keys are
/// valid during this period.
pub const DEFAULT_TRANSITION_DAYS: u16 = 7;

/// Maximum length of the `reason` field in bytes. Prevents a DoS
/// via gossip with a pathologically large rotation announcement.
pub const REASON_MAX_BYTES: usize = 280;

/// Maximum transition window. 90 days is generous; longer values
/// are likely a bug or an attempt to keep a compromised key alive.
pub const MAX_TRANSITION_DAYS: u16 = 90;

const SECS_PER_DAY: u64 = 86_400;

/// Gossip topic for key rotation announcements.
pub const KEY_ROTATION_TOPIC: &str = "nexus-grid/key-rotation/v1";

// =================================================================
// Types
// =================================================================

/// A self-signed key rotation announcement.
///
/// The old key holder proves possession by signing the canonical
/// bytes of this struct (with [`DOMAIN_KEY_ROTATION_V1`]) using
/// the old signing key. Consumers verify the signature against
/// `old_public_key` before applying the rotation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KeyRotationAnnouncement {
    /// Format version. Must equal [`KEY_ROTATION_FORMAT_VERSION`].
    pub version: u16,

    /// The public key being rotated away.
    pub old_public_key: [u8; PUBLIC_KEY_LENGTH],

    /// The replacement public key.
    pub new_public_key: [u8; PUBLIC_KEY_LENGTH],

    /// Unix timestamp (seconds) when the rotation was published.
    pub timestamp: u64,

    /// Human-readable reason for the rotation.
    pub reason: String,

    /// Transition window in days. Both keys are valid during this
    /// period. After expiry, `old_public_key` is fully revoked.
    pub transition_days: u16,
}

/// A signed rotation announcement envelope, ready for gossip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedKeyRotation {
    /// The announcement payload.
    pub announcement: KeyRotationAnnouncement,

    /// Ed25519 signature over the canonical bytes of
    /// [`Self::announcement`] (64 bytes).
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl KeyRotationAnnouncement {
    /// Build a new announcement. Validates field constraints.
    pub fn new(
        old_public_key: [u8; PUBLIC_KEY_LENGTH],
        new_public_key: [u8; PUBLIC_KEY_LENGTH],
        timestamp: u64,
        reason: impl Into<String>,
        transition_days: u16,
    ) -> Result<Self> {
        let reason = reason.into();
        validate_fields(&old_public_key, &new_public_key, &reason, transition_days)?;
        Ok(Self {
            version: KEY_ROTATION_FORMAT_VERSION,
            old_public_key,
            new_public_key,
            timestamp,
            reason,
            transition_days,
        })
    }
}

impl SignedKeyRotation {
    /// Sign an announcement with the **old** keypair.
    pub fn sign(announcement: KeyRotationAnnouncement, old_keypair: &KeyPair) -> Result<Self> {
        if announcement.old_public_key != old_keypair.public_bytes() {
            return Err(NexusError::Crypto(
                "old_public_key does not match signing keypair".into(),
            ));
        }
        validate_fields(
            &announcement.old_public_key,
            &announcement.new_public_key,
            &announcement.reason,
            announcement.transition_days,
        )?;
        let bytes = canonical_bytes(&announcement, DOMAIN_KEY_ROTATION_V1)?;
        let signature = old_keypair.sign(&bytes);
        Ok(Self {
            announcement,
            signature,
        })
    }

    /// Verify the announcement signature against `old_public_key`.
    pub fn verify(&self) -> Result<()> {
        if self.announcement.version != KEY_ROTATION_FORMAT_VERSION {
            return Err(NexusError::Crypto(format!(
                "key rotation version mismatch (got {}, expected {})",
                self.announcement.version, KEY_ROTATION_FORMAT_VERSION
            )));
        }
        validate_fields(
            &self.announcement.old_public_key,
            &self.announcement.new_public_key,
            &self.announcement.reason,
            self.announcement.transition_days,
        )?;
        let bytes = canonical_bytes(&self.announcement, DOMAIN_KEY_ROTATION_V1)?;
        crate::crypto::verify(&self.announcement.old_public_key, &bytes, &self.signature)
    }
}

fn validate_fields(
    old_pk: &[u8; PUBLIC_KEY_LENGTH],
    new_pk: &[u8; PUBLIC_KEY_LENGTH],
    reason: &str,
    transition_days: u16,
) -> Result<()> {
    if old_pk == new_pk {
        return Err(NexusError::Crypto(
            "old and new public keys must differ".into(),
        ));
    }
    if reason.len() > REASON_MAX_BYTES {
        return Err(NexusError::Crypto(format!(
            "reason has {} bytes, exceeds REASON_MAX_BYTES={}",
            reason.len(),
            REASON_MAX_BYTES
        )));
    }
    if transition_days > MAX_TRANSITION_DAYS {
        return Err(NexusError::Crypto(format!(
            "transition_days {} exceeds MAX_TRANSITION_DAYS={}",
            transition_days, MAX_TRANSITION_DAYS
        )));
    }
    Ok(())
}

// =================================================================
// Revocation cache
// =================================================================

/// An entry in the revocation cache, tracking one key rotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevocationEntry {
    /// The replacement public key.
    pub new_public_key: [u8; PUBLIC_KEY_LENGTH],
    /// Unix timestamp when the transition started.
    pub transition_start: u64,
    /// Transition window in days.
    pub transition_days: u16,
    /// Human-readable reason.
    pub reason: String,
}

/// In-memory cache of key rotation announcements. Keyed by the
/// **old** public key. Pre-v1.0 there are zero external nodes, so
/// in-memory is sufficient (persistence deferred S26).
#[derive(Debug, Clone, Default)]
pub struct RevocationCache {
    entries: HashMap<[u8; PUBLIC_KEY_LENGTH], RevocationEntry>,
}

impl RevocationCache {
    /// Create an empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check whether `public_key` has been fully revoked (the
    /// transition window has expired).
    pub fn is_revoked(&self, public_key: &[u8; PUBLIC_KEY_LENGTH], now_ts: u64) -> bool {
        self.entries.get(public_key).is_some_and(|entry| {
            let expiry = entry
                .transition_start
                .saturating_add(u64::from(entry.transition_days) * SECS_PER_DAY);
            now_ts >= expiry
        })
    }

    /// Check whether `public_key` is currently in a transition
    /// window (both old and new keys are valid).
    pub fn is_in_transition(&self, public_key: &[u8; PUBLIC_KEY_LENGTH], now_ts: u64) -> bool {
        self.entries.get(public_key).is_some_and(|entry| {
            let expiry = entry
                .transition_start
                .saturating_add(u64::from(entry.transition_days) * SECS_PER_DAY);
            now_ts < expiry
        })
    }

    /// Look up the revocation entry for a key, if any.
    pub fn get(&self, public_key: &[u8; PUBLIC_KEY_LENGTH]) -> Option<&RevocationEntry> {
        self.entries.get(public_key)
    }

    /// Apply a verified rotation announcement to the cache.
    ///
    /// The caller MUST have called [`SignedKeyRotation::verify`]
    /// before calling this. This method trusts that the signature
    /// has already been validated.
    ///
    /// Returns `Err` if the cache already contains an entry for the
    /// same `old_public_key` with a `transition_start` >= the new
    /// announcement's timestamp (stale rotation rejected).
    pub fn apply_verified(&mut self, announcement: &KeyRotationAnnouncement) -> Result<()> {
        if let Some(existing) = self.entries.get(&announcement.old_public_key) {
            if announcement.timestamp <= existing.transition_start {
                tracing::warn!(
                    old_key = hex::encode(announcement.old_public_key),
                    existing_ts = existing.transition_start,
                    incoming_ts = announcement.timestamp,
                    "stale_rotation_rejected"
                );
                return Err(NexusError::Crypto("stale rotation rejected".into()));
            }
            tracing::info!(
                old_key = hex::encode(announcement.old_public_key),
                "rotation_updated"
            );
        }
        self.entries.insert(
            announcement.old_public_key,
            RevocationEntry {
                new_public_key: announcement.new_public_key,
                transition_start: announcement.timestamp,
                transition_days: announcement.transition_days,
                reason: announcement.reason.clone(),
            },
        );
        Ok(())
    }

    /// Verify a [`SignedKeyRotation`] and apply it to the cache in
    /// one step. Returns `Err` if the signature is invalid.
    pub fn apply_announcement(&mut self, signed: &SignedKeyRotation) -> Result<()> {
        signed.verify()?;
        self.apply_verified(&signed.announcement)
    }

    /// Number of entries in the cache.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::DOMAIN_TASK_V1;

    fn ts(days_from_epoch: u64) -> u64 {
        days_from_epoch * SECS_PER_DAY
    }

    fn make_rotation(
        old_kp: &KeyPair,
        new_kp: &KeyPair,
        timestamp: u64,
        reason: &str,
        transition_days: u16,
    ) -> SignedKeyRotation {
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            timestamp,
            reason,
            transition_days,
        )
        .unwrap();
        SignedKeyRotation::sign(ann, old_kp).unwrap()
    }

    // ---- B.7 core tests ----

    #[test]
    fn sign_verify_rotation_announcement() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, 1_700_000_000, "compromised", 7);
        signed.verify().expect("valid rotation must verify");
        assert_eq!(signed.announcement.old_public_key, old_kp.public_bytes());
        assert_eq!(signed.announcement.new_public_key, new_kp.public_bytes());
    }

    #[test]
    fn wrong_key_rejects() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let impostor = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "test",
            7,
        )
        .unwrap();
        // Sign with impostor instead of old key
        let bytes = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        let signature = impostor.sign(&bytes);
        let signed = SignedKeyRotation {
            announcement: ann,
            signature,
        };
        assert!(signed.verify().is_err());
    }

    #[test]
    fn revocation_cache_apply_and_check() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, ts(100), "routine", 7);

        let mut cache = RevocationCache::new();
        assert!(cache.is_empty());

        cache.apply_announcement(&signed).unwrap();
        assert_eq!(cache.len(), 1);

        // During transition: not revoked, in transition
        let mid = ts(103); // day 103, 3 days after start at day 100
        assert!(!cache.is_revoked(&old_kp.public_bytes(), mid));
        assert!(cache.is_in_transition(&old_kp.public_bytes(), mid));

        // Unknown key: neither revoked nor in transition
        let unknown = KeyPair::generate();
        assert!(!cache.is_revoked(&unknown.public_bytes(), mid));
        assert!(!cache.is_in_transition(&unknown.public_bytes(), mid));
    }

    #[test]
    fn transition_expired() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, ts(100), "compromise", 7);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed).unwrap();

        // After 7 days: revoked
        let after = ts(107);
        assert!(cache.is_revoked(&old_kp.public_bytes(), after));
        assert!(!cache.is_in_transition(&old_kp.public_bytes(), after));
    }

    #[test]
    fn transition_active() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, ts(100), "planned", 7);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed).unwrap();

        // Day 106 = 6 days in, still in transition
        let during = ts(106);
        assert!(!cache.is_revoked(&old_kp.public_bytes(), during));
        assert!(cache.is_in_transition(&old_kp.public_bytes(), during));
    }

    #[test]
    fn domain_separation_distinct() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "test",
            7,
        )
        .unwrap();
        let rotation_bytes = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        let task_bytes = canonical_bytes(&ann, DOMAIN_TASK_V1).unwrap();
        assert_ne!(
            rotation_bytes, task_bytes,
            "domain separation must yield distinct byte strings"
        );
    }

    #[test]
    fn announcement_canonical_deterministic() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "test",
            7,
        )
        .unwrap();
        let a = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        let b = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn roundtrip_through_json() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, 1_700_000_000, "json test", 7);
        let json = serde_json::to_string(&signed).unwrap();
        let back: SignedKeyRotation = serde_json::from_str(&json).unwrap();
        assert_eq!(back, signed);
        back.verify().expect("round-tripped must still verify");
    }

    #[test]
    fn pyo3_verify_key_rotation_roundtrip() {
        // Simulates the PyO3 path: serialize to JSON, deserialize,
        // verify. Same flow as verify_key_rotation in nexus-core-py.
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, 1_700_000_000, "pyo3 test", 7);
        let json_bytes = serde_json::to_vec(&signed).unwrap();
        let back: SignedKeyRotation = serde_json::from_slice(&json_bytes).unwrap();
        back.verify().expect("pyo3 round-trip must verify");
    }

    // ---- edge cases ----

    #[test]
    fn empty_reason_accepted() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, 1_700_000_000, "", 7);
        signed.verify().expect("empty reason is valid");
    }

    #[test]
    fn zero_transition_days() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, ts(100), "immediate", 0);
        signed.verify().expect("zero transition is valid");

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed).unwrap();

        // Immediately revoked (transition window = 0 days)
        assert!(cache.is_revoked(&old_kp.public_bytes(), ts(100)));
        assert!(!cache.is_in_transition(&old_kp.public_bytes(), ts(100)));
    }

    #[test]
    fn max_transition_days_accepted() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, ts(0), "max window", MAX_TRANSITION_DAYS);
        signed.verify().expect("max transition days is valid");
    }

    #[test]
    fn over_max_transition_days_rejected() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let result = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            ts(0),
            "too long",
            MAX_TRANSITION_DAYS + 1,
        );
        assert!(result.is_err());
    }

    #[test]
    fn reason_over_max_rejected() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let result = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "x".repeat(REASON_MAX_BYTES + 1),
            7,
        );
        assert!(result.is_err());
    }

    #[test]
    fn same_key_rejected() {
        let kp = KeyPair::generate();
        let result = KeyRotationAnnouncement::new(
            kp.public_bytes(),
            kp.public_bytes(),
            1_700_000_000,
            "same key",
            7,
        );
        assert!(result.is_err());
    }

    #[test]
    fn tampered_payload_rejected() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let mut signed = make_rotation(&old_kp, &new_kp, 1_700_000_000, "original", 7);
        signed.announcement.reason = "tampered".into();
        assert!(signed.verify().is_err());
    }

    #[test]
    fn version_mismatch_rejected() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let ann = KeyRotationAnnouncement {
            version: 99,
            old_public_key: old_kp.public_bytes(),
            new_public_key: new_kp.public_bytes(),
            timestamp: 1_700_000_000,
            reason: "future".into(),
            transition_days: 7,
        };
        // Bypass the constructor to set version=99
        let bytes = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        let signature = old_kp.sign(&bytes);
        let signed = SignedKeyRotation {
            announcement: ann,
            signature,
        };
        assert!(signed.verify().is_err());
    }

    #[test]
    fn sign_rejects_mismatched_keypair() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let other_kp = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "test",
            7,
        )
        .unwrap();
        // Sign with other_kp but announcement says old_kp
        assert!(SignedKeyRotation::sign(ann, &other_kp).is_err());
    }

    #[test]
    fn cache_accepts_newer_rotation() {
        let old_kp = KeyPair::generate();
        let new_kp1 = KeyPair::generate();
        let new_kp2 = KeyPair::generate();

        let signed1 = make_rotation(&old_kp, &new_kp1, ts(100), "first", 7);
        let signed2 = make_rotation(&old_kp, &new_kp2, ts(105), "second", 14);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed1).unwrap();
        cache.apply_announcement(&signed2).unwrap();

        assert_eq!(cache.len(), 1);
        let entry = cache.get(&old_kp.public_bytes()).unwrap();
        assert_eq!(entry.new_public_key, new_kp2.public_bytes());
        assert_eq!(entry.transition_days, 14);
    }

    #[test]
    fn cache_rejects_stale_rotation() {
        let old_kp = KeyPair::generate();
        let new_kp1 = KeyPair::generate();
        let new_kp2 = KeyPair::generate();

        let signed1 = make_rotation(&old_kp, &new_kp1, ts(200), "first", 7);
        let signed_stale = make_rotation(&old_kp, &new_kp2, ts(100), "stale", 14);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed1).unwrap();
        let result = cache.apply_announcement(&signed_stale);
        assert!(result.is_err());

        let entry = cache.get(&old_kp.public_bytes()).unwrap();
        assert_eq!(entry.new_public_key, new_kp1.public_bytes());
        assert_eq!(entry.transition_start, ts(200));
    }

    #[test]
    fn cache_rejects_same_timestamp_rotation() {
        let old_kp = KeyPair::generate();
        let new_kp1 = KeyPair::generate();
        let new_kp2 = KeyPair::generate();

        let signed1 = make_rotation(&old_kp, &new_kp1, ts(100), "first", 7);
        let signed_same = make_rotation(&old_kp, &new_kp2, ts(100), "same-ts", 14);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed1).unwrap();
        let result = cache.apply_announcement(&signed_same);
        assert!(result.is_err());

        let entry = cache.get(&old_kp.public_bytes()).unwrap();
        assert_eq!(entry.new_public_key, new_kp1.public_bytes());
    }

    #[test]
    fn transition_boundary_exact() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let start = ts(100);
        let signed = make_rotation(&old_kp, &new_kp, start, "boundary", 7);

        let mut cache = RevocationCache::new();
        cache.apply_announcement(&signed).unwrap();

        let boundary = start + 7 * SECS_PER_DAY;
        // Exactly at boundary: revoked (>= expiry)
        assert!(cache.is_revoked(&old_kp.public_bytes(), boundary));
        // One second before boundary: still in transition
        assert!(cache.is_in_transition(&old_kp.public_bytes(), boundary - 1));
    }

    #[test]
    fn apply_invalid_signature_rejected() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let impostor = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "forged",
            7,
        )
        .unwrap();
        let bytes = canonical_bytes(&ann, DOMAIN_KEY_ROTATION_V1).unwrap();
        let bad_sig = impostor.sign(&bytes);
        let forged = SignedKeyRotation {
            announcement: ann,
            signature: bad_sig,
        };

        let mut cache = RevocationCache::new();
        assert!(cache.apply_announcement(&forged).is_err());
        assert!(cache.is_empty());
    }

    #[test]
    fn future_timestamp_accepted() {
        // Pre-v1.0: no clock enforcement on announcements.
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(&old_kp, &new_kp, u64::MAX - 1_000_000, "future", 7);
        signed.verify().expect("future timestamp accepted pre-v1.0");
    }

    #[test]
    fn reason_at_max_boundary_accepted() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let signed = make_rotation(
            &old_kp,
            &new_kp,
            1_700_000_000,
            &"r".repeat(REASON_MAX_BYTES),
            7,
        );
        signed.verify().expect("reason at max boundary is valid");
    }
}
