// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase E.1 — `CanarySigner` trait abstraction.
//!
//! The trait decouples canary signing from any one signature
//! algorithm so Phase E.2 can drop in a `FrostCanarySigner`
//! (threshold K-of-N over Ed25519 via FROST RFC 9591) without
//! touching `build_canary` callers, and Phase E.5 can decouple
//! signing from attestation entirely.
//!
//! ## Wire format
//!
//! Every implementation MUST produce a 64-byte signature that
//! verifies against a 32-byte Ed25519 public key under
//! [`nexus_core_rs::crypto::verify`]. FROST sigs over Ed25519 are
//! byte-for-byte indistinguishable from a standalone Ed25519 sig
//! per RFC 8032 (the aggregated signature is itself a valid
//! Ed25519 signature), so `CanarySigned v1` wire format stays
//! frozen across the trait migration.

use nexus_core_rs::crypto::{PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use nexus_core_rs::KeyPair;

/// Abstraction over the maintainer key used to sign warrant
/// canaries.
///
/// Implementations MUST guarantee that `(pubkey, sign(msg))`
/// verifies under [`nexus_core_rs::crypto::verify`]. That is
/// the only invariant the rest of the canary pipeline assumes.
pub trait CanarySigner: Send + Sync {
    /// 32-byte Ed25519 public key the produced signatures verify
    /// against.
    fn pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH];

    /// Sign `message` and return a 64-byte Ed25519 signature.
    /// For threshold implementations, the returned signature is
    /// the aggregated FROST signature, byte-identical in shape to
    /// a standalone Ed25519 sig.
    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES];
}

/// Default single-key implementation backed by a
/// [`nexus_core_rs::KeyPair`]. This is what the maintainer's
/// `<sbfb_home>/canary-key.key` deserializes into.
pub struct Ed25519CanarySigner {
    inner: KeyPair,
}

impl Ed25519CanarySigner {
    /// Wrap an existing keypair as a canary signer.
    pub fn new(inner: KeyPair) -> Self {
        Self { inner }
    }

    /// Borrow the underlying keypair. Useful in code paths that
    /// still need the raw `KeyPair` (e.g. duress ack signing
    /// re-uses the same key).
    pub fn keypair(&self) -> &KeyPair {
        &self.inner
    }
}

impl CanarySigner for Ed25519CanarySigner {
    fn pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        self.inner.public_bytes()
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        self.inner.sign(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::crypto::verify;

    #[test]
    fn signer_trait_roundtrip_identical_to_baseline_ed25519() {
        let kp = KeyPair::generate();
        let signer = Ed25519CanarySigner::new(kp.clone());

        let msg = b"sprint 20 phase E.1 trait abstraction";
        let sig_via_trait = signer.sign(msg);
        let sig_baseline = kp.sign(msg);

        // Same key + same msg + Ed25519 deterministic signing
        // (RFC 8032 §5.1.6) => byte-identical signature whether
        // produced through the trait or the raw KeyPair.
        assert_eq!(sig_via_trait, sig_baseline);

        // And both sigs verify against the same pubkey.
        let pk = signer.pubkey();
        verify(&pk, msg, &sig_via_trait).expect("trait sig verifies");
        verify(&pk, msg, &sig_baseline).expect("baseline sig verifies");
    }

    #[test]
    fn signer_trait_pubkey_matches_baseline() {
        let kp = KeyPair::generate();
        let baseline_pk = kp.public_bytes();

        let signer = Ed25519CanarySigner::new(kp);
        assert_eq!(signer.pubkey(), baseline_pk);
    }
}
