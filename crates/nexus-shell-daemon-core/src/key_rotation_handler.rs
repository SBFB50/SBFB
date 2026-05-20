// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gossip message handler for key rotation announcements.
//!
//! Sprint 25 Phase B: the shell-daemon subscribes to the
//! `nexus-grid/key-rotation/v1` gossip topic and feeds each
//! incoming message through [`handle_rotation_message`], which
//! deserializes, verifies the Ed25519 signature against the
//! `old_public_key`, and applies the rotation to the shared
//! [`RevocationCache`].
//!
//! Pattern mirrors [`super::pow_policy_loader`]: the daemon
//! runtime holds an `Arc<RwLock<RevocationCache>>` and passes it
//! to the handler on every incoming gossip event. The handler is
//! pure (no async, no I/O beyond the cache write) so the binary
//! crate's async gossip loop stays simple.

use std::sync::{Arc, RwLock};

use nexus_core_rs::key_rotation::{RevocationCache, SignedKeyRotation};
use tracing::{debug, warn};

/// Process a raw gossip message from the key-rotation topic.
///
/// Returns `Ok(())` if the announcement was valid and applied, or
/// logs a warning and returns `Err` if the message is malformed
/// or the signature fails. Callers should NOT propagate the error
/// to the gossip layer — a bad message from a peer is expected
/// and must not crash the subscribe loop.
pub fn handle_rotation_message(
    cache: &Arc<RwLock<RevocationCache>>,
    raw: &[u8],
) -> Result<(), String> {
    let signed: SignedKeyRotation = serde_json::from_slice(raw).map_err(|e| {
        warn!(error = %e, "key-rotation: malformed gossip message");
        format!("deserialize: {e}")
    })?;

    signed.verify().map_err(|e| {
        warn!(
            error = %e,
            old_key = hex::encode(signed.announcement.old_public_key),
            "key-rotation: signature verification failed"
        );
        format!("verify: {e}")
    })?;

    let old_key_hex = hex::encode(signed.announcement.old_public_key);
    let new_key_hex = hex::encode(signed.announcement.new_public_key);

    match cache.write() {
        Ok(mut guard) => match guard.apply_verified(&signed.announcement) {
            Ok(()) => {
                debug!(
                    old_key = %old_key_hex,
                    new_key = %new_key_hex,
                    transition_days = signed.announcement.transition_days,
                    reason = %signed.announcement.reason,
                    "key-rotation: applied announcement"
                );
                nexus_events_core::emit_event(&nexus_events_core::SecurityEvent::TokenRotation {
                    rotated_at: signed.announcement.timestamp.to_string(),
                });
                Ok(())
            }
            Err(e) => {
                warn!(
                    error = %e,
                    old_key = %old_key_hex,
                    "key-rotation: stale rotation rejected"
                );
                Err(format!("stale: {e}"))
            }
        },
        Err(_) => {
            warn!("key-rotation: RevocationCache RwLock poisoned, skipping");
            Err("cache lock poisoned".into())
        }
    }
}

/// Create a default shared revocation cache for the daemon runtime.
pub fn shared_revocation_cache() -> Arc<RwLock<RevocationCache>> {
    Arc::new(RwLock::new(RevocationCache::new()))
}

/// Populate a RevocationCache from persisted key rotation data.
///
/// Each tuple: `(old_pubkey_hex, new_pubkey_hex, timestamp, transition_days, reason)`.
/// Invalid hex keys are skipped with a warning.
pub fn populate_cache(
    cache: &Arc<RwLock<RevocationCache>>,
    rotations: &[(String, String, u64, u16, String)],
) -> usize {
    use nexus_core_rs::key_rotation::KeyRotationAnnouncement;

    let mut applied = 0usize;
    let mut guard = match cache.write() {
        Ok(g) => g,
        Err(_) => {
            warn!("key-rotation: cache lock poisoned during DB restore");
            return 0;
        }
    };
    for (old_hex, new_hex, timestamp, transition_days, reason) in rotations {
        let old_bytes: [u8; 32] = match hex::decode(old_hex) {
            Ok(v) if v.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                arr
            }
            _ => {
                warn!(old_pubkey = %old_hex, "key-rotation: invalid old_pubkey hex, skipping");
                continue;
            }
        };
        let new_bytes: [u8; 32] = match hex::decode(new_hex) {
            Ok(v) if v.len() == 32 => {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&v);
                arr
            }
            _ => {
                warn!(new_pubkey = %new_hex, "key-rotation: invalid new_pubkey hex, skipping");
                continue;
            }
        };
        let ann = match KeyRotationAnnouncement::new(
            old_bytes,
            new_bytes,
            *timestamp,
            reason,
            *transition_days,
        ) {
            Ok(a) => a,
            Err(e) => {
                warn!(error = %e, "key-rotation: invalid persisted row, skipping");
                continue;
            }
        };
        if guard.apply_verified(&ann).is_ok() {
            applied += 1;
        }
    }
    applied
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::crypto::KeyPair;
    use nexus_core_rs::key_rotation::{KeyRotationAnnouncement, SignedKeyRotation};

    #[test]
    fn handle_valid_rotation_message() {
        let old_kp = KeyPair::generate();
        let new_kp = KeyPair::generate();
        let ann = KeyRotationAnnouncement::new(
            old_kp.public_bytes(),
            new_kp.public_bytes(),
            1_700_000_000,
            "test rotation",
            7,
        )
        .unwrap();
        let signed = SignedKeyRotation::sign(ann, &old_kp).unwrap();
        let raw = serde_json::to_vec(&signed).unwrap();

        let cache = shared_revocation_cache();
        handle_rotation_message(&cache, &raw).expect("valid message");

        let guard = cache.read().unwrap();
        assert_eq!(guard.len(), 1);
        assert!(guard.is_in_transition(&old_kp.public_bytes(), 1_700_000_000));
    }

    #[test]
    fn handle_malformed_json_rejects() {
        let cache = shared_revocation_cache();
        let result = handle_rotation_message(&cache, b"not json");
        assert!(result.is_err());
        assert!(cache.read().unwrap().is_empty());
    }

    #[test]
    fn handle_invalid_signature_rejects() {
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
        let bytes =
            nexus_core_rs::canonical::canonical_bytes(&ann, nexus_core_rs::DOMAIN_KEY_ROTATION_V1)
                .unwrap();
        let bad_sig = impostor.sign(&bytes);
        let forged = SignedKeyRotation {
            announcement: ann,
            signature: bad_sig,
        };
        let raw = serde_json::to_vec(&forged).unwrap();

        let cache = shared_revocation_cache();
        let result = handle_rotation_message(&cache, &raw);
        assert!(result.is_err());
        assert!(cache.read().unwrap().is_empty());
    }
}
