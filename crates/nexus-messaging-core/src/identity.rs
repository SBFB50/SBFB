// SPDX-License-Identifier: AGPL-3.0-or-later
//! A messaging identity: one vodozemac Olm account per (node, app). It is
//! distinct from the iroh node key on purpose — nothing signed by it can be
//! confused with a node-signed SBFB payload, and it never leaves the daemon.

use serde::{Deserialize, Serialize};
use vodozemac::olm::{Account, AccountPickle};
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey, Ed25519Signature};

use crate::{ChatError, Result};

/// Public half of an identity, as it travels in profiles and invitations.
/// Both keys are vodozemac base64.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub curve: String,
    pub ed: String,
}

impl PublicIdentity {
    pub fn curve_key(&self) -> Result<Curve25519PublicKey> {
        Curve25519PublicKey::from_base64(&self.curve).map_err(|_| ChatError::Malformed("curve key"))
    }

    pub fn ed_key(&self) -> Result<Ed25519PublicKey> {
        Ed25519PublicKey::from_base64(&self.ed).map_err(|_| ChatError::Malformed("ed25519 key"))
    }

    /// Verify a base64 signature over `bytes` with this identity's Ed25519 key.
    pub fn verify(&self, bytes: &[u8], sig_b64: &str, what: &'static str) -> Result<()> {
        let sig = Ed25519Signature::from_base64(sig_b64).map_err(|_| ChatError::Malformed(what))?;
        self.ed_key()?
            .verify(bytes, &sig)
            .map_err(|_| ChatError::BadSignature(what))
    }

    /// Short safety number shown to users for out-of-band comparison.
    pub fn fingerprint(&self) -> String {
        let digest = nexus_core_rs::blake3_hash(format!("{}:{}", self.curve, self.ed).as_bytes());
        digest
            .chunks(2)
            .take(6)
            .map(|c| format!("{:05}", u16::from_be_bytes([c[0], c[1]])))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub struct Identity {
    account: Account,
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

impl Identity {
    pub fn new() -> Self {
        let mut account = Account::new();
        account.generate_fallback_key();
        // Keys are never "marked published": vodozemac only lists
        // UNPUBLISHED keys, and this identity hands its keys out itself
        // (profiles, invitations) instead of uploading them to a server.
        Self { account }
    }

    pub fn public(&self) -> PublicIdentity {
        PublicIdentity {
            curve: self.account.curve25519_key().to_base64(),
            ed: self.account.ed25519_key().to_base64(),
        }
    }

    pub fn sign(&self, bytes: &[u8]) -> String {
        self.account.sign(bytes).to_base64()
    }

    /// The current fallback key: reusable, so any member can open an Olm
    /// channel to us while we are offline.
    pub fn fallback_key(&self) -> String {
        self.account
            .fallback_key()
            .values()
            .next()
            .map(|k| k.to_base64())
            .expect("a fallback key is generated at creation")
    }

    /// A fresh single-use key, for an invitation. vodozemac keeps at most
    /// 100 outstanding one-time keys and evicts the oldest beyond that, so
    /// an invitation older than 100 newer ones can no longer be redeemed.
    pub fn one_time_key(&mut self) -> String {
        self.account
            .generate_one_time_keys(1)
            .created
            .first()
            .expect("one key was just generated")
            .to_base64()
    }

    pub(crate) fn account(&self) -> &Account {
        &self.account
    }

    pub(crate) fn account_mut(&mut self) -> &mut Account {
        &mut self.account
    }

    /// Seal the whole account under a 32-byte key (the daemon keeps that key
    /// in its host vault).
    pub fn seal(&self, key: &[u8; 32]) -> String {
        self.account.pickle().encrypt(key)
    }

    pub fn unseal(sealed: &str, key: &[u8; 32]) -> Result<Self> {
        let pickle = AccountPickle::from_encrypted(sealed, key)
            .map_err(|_| ChatError::Crypto("identity unseal failed".into()))?;
        Ok(Self {
            account: Account::from_pickle(pickle),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_verify_and_seal_round_trip() {
        let id = Identity::new();
        let public = id.public();
        let sig = id.sign(b"hello");
        public.verify(b"hello", &sig, "test").unwrap();
        assert!(public.verify(b"hellO", &sig, "test").is_err());

        let restored = Identity::unseal(&id.seal(&[1; 32]), &[1; 32]).unwrap();
        assert_eq!(restored.public(), public);
        assert_eq!(restored.fallback_key(), id.fallback_key());
        assert!(Identity::unseal(&id.seal(&[1; 32]), &[2; 32]).is_err());
    }

    #[test]
    fn fingerprint_is_stable_and_distinct() {
        let a = Identity::new().public();
        let b = Identity::new().public();
        assert_eq!(a.fingerprint(), a.clone().fingerprint());
        assert_ne!(a.fingerprint(), b.fingerprint());
        assert_eq!(a.fingerprint().split(' ').count(), 6);
    }
}
