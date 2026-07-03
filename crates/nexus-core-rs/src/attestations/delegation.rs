// SPDX-License-Identifier: AGPL-3.0-or-later
//! DelegationCert — bridging SBFB node_id to forge SSH signing keys (Couche 3).
//!
//! Sprint 23 Phase F. A contributor self-signs a certificate binding
//! their SBFB Ed25519 `node_id` to an SSH/PGP signing key fingerprint
//! used on external forges (GitHub, Codeberg, Forgejo, etc.). The
//! verifier uses this bridge to attribute `git log --show-signature`
//! commits back to a SBFB identity without trusting the forge operator.
//!
//! ## Wire format
//!
//! ```text
//! DelegationCert {
//!     node_id:                      [u8; 32],     // SBFB Ed25519 pubkey
//!     delegated_pubkey_algo:        String,       // "ssh-ed25519" | "ssh-rsa" | "openpgp-ed25519"
//!     delegated_pubkey_fingerprint: String,       // SHA-256 hex lowercase (64 chars)
//!     issued_at_ts:                 i64,          // UTC unix seconds
//!     expires_at_ts:                Option<i64>,  // optional TTL (None = no expiry)
//!     node_sig:                     [u8; 64],     // Ed25519 over JCS canonical bytes
//!                                                 // with DOMAIN_DELEGATION_CERT_V1
//! }
//! ```
//!
//! ## Pre-launch policy
//!
//! Stable pre-launch. Any redefinition of the byte layout lands as an
//! in-place edit of v1 until the first `v1.0` tag (cf. `CLAUDE.md`
//! §Pre-launch protocol policy).
//!
//! ## Design-only scope (S23)
//!
//! This struct is the format primitive only. Runtime wiring (keyring
//! lookup, `git log` parser, multi-forge cross-validation) lands
//! S24-S27. Cf. [`docs/security/CONTRIBUTOR_ATTESTATION_RFC.md`](
//! ../../../docs/security/CONTRIBUTOR_ATTESTATION_RFC.md) §3-§7.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_DELEGATION_CERT_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

fn default_trust_level() -> u8 {
    3
}

/// Accepted values for [`DelegationCert::delegated_pubkey_algo`].
pub const DELEGATION_ALGO_SSH_ED25519: &str = "ssh-ed25519";
/// SSH RSA algorithm identifier.
pub const DELEGATION_ALGO_SSH_RSA: &str = "ssh-rsa";
/// OpenPGP Ed25519 algorithm identifier (future).
pub const DELEGATION_ALGO_OPENPGP_ED25519: &str = "openpgp-ed25519";

/// Error type for delegation cert operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationCertError {
    /// Ed25519 signature verification failed.
    BadSignature(String),
    /// A required field failed validation.
    BadField {
        /// Name of the field.
        field: String,
        /// Why it failed.
        reason: String,
    },
    /// The cert has passed its `expires_at_ts`.
    Expired {
        /// When the cert expired.
        expires_at: i64,
        /// Current time at verification.
        now: i64,
    },
    /// JCS canonical serialization failed.
    CanonicalFailed(String),
}

impl std::fmt::Display for DelegationCertError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DelegationCertError::BadSignature(msg) => {
                write!(f, "delegation cert signature invalid: {msg}")
            }
            DelegationCertError::BadField { field, reason } => {
                write!(f, "delegation cert bad field {field:?}: {reason}")
            }
            DelegationCertError::Expired { expires_at, now } => {
                write!(
                    f,
                    "delegation cert expired (expires_at={expires_at}, now={now})"
                )
            }
            DelegationCertError::CanonicalFailed(msg) => {
                write!(f, "delegation cert canonical bytes failed: {msg}")
            }
        }
    }
}

impl std::error::Error for DelegationCertError {}

impl From<DelegationCertError> for NexusError {
    fn from(e: DelegationCertError) -> Self {
        NexusError::Crypto(e.to_string())
    }
}

/// Scope of a delegation: which organisation and forges it covers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationScope {
    /// Human-readable organisation name (e.g. `"FlowUP"`).
    pub org_name: String,
    /// Forge URLs the delegation covers (e.g. `["https://github.com/SBFB50/SBFB"]`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forge_urls: Vec<String>,
}

/// A self-signed certificate binding a SBFB `node_id` to an external
/// forge signing key fingerprint.
///
/// The contributor signs this with their SBFB node private key. No
/// coordinator involvement — the cert is published alongside
/// `SBFB.json` in the deploy archive under
/// `.sbfb/delegations/<fingerprint>.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationCert {
    /// Ed25519 public key of the SBFB node delegating to the forge key.
    pub node_id: [u8; PUBLIC_KEY_LENGTH],
    /// Algorithm of the delegated forge key (e.g. `"ssh-ed25519"`).
    pub delegated_pubkey_algo: String,
    /// SHA-256 fingerprint of the delegated key, lowercase hex (64 chars).
    pub delegated_pubkey_fingerprint: String,
    /// UTC unix seconds when this cert was issued.
    pub issued_at_ts: i64,
    /// Optional expiry. `None` means no expiry; best-practice is annual re-issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at_ts: Option<i64>,
    /// Trust level delegated (1 = minimal, 5 = full). Default 3.
    /// Runtime tolerance: certs serialized before S27 omit this field
    /// and deserialize to `3` via `#[serde(default)]`.
    #[serde(default = "default_trust_level")]
    pub trust_level: u8,
    /// Optional delegation scope (org + forge URLs). `None` = unbounded.
    /// Runtime tolerance: certs serialized before S27 omit this field
    /// and deserialize to `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<DelegationScope>,
    /// Ed25519 signature by `node_id` over canonical JCS bytes with
    /// [`DOMAIN_DELEGATION_CERT_V1`](crate::canonical::DOMAIN_DELEGATION_CERT_V1).
    #[serde(with = "BigArray")]
    pub node_sig: [u8; SIGNATURE_BYTES],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct DelegationCertPayload {
    node_id: [u8; PUBLIC_KEY_LENGTH],
    delegated_pubkey_algo: String,
    delegated_pubkey_fingerprint: String,
    issued_at_ts: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expires_at_ts: Option<i64>,
    #[serde(default = "default_trust_level")]
    trust_level: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    scope: Option<DelegationScope>,
}

impl DelegationCert {
    /// Build and sign a [`DelegationCert`].
    ///
    /// `node_keypair` is the contributor's SBFB Ed25519 key. The cert
    /// binds `node_keypair.public_bytes()` to `delegated_pubkey_fingerprint`
    /// under algorithm `delegated_pubkey_algo`.
    pub fn sign(
        delegated_pubkey_algo: &str,
        delegated_pubkey_fingerprint: &str,
        issued_at_ts: i64,
        expires_at_ts: Option<i64>,
        trust_level: u8,
        scope: Option<DelegationScope>,
        node_keypair: &KeyPair,
    ) -> Result<Self> {
        validate_algo(delegated_pubkey_algo)?;
        validate_fingerprint(delegated_pubkey_fingerprint)?;
        validate_trust_level(trust_level)?;

        let node_id = node_keypair.public_bytes();
        let payload = DelegationCertPayload {
            node_id,
            delegated_pubkey_algo: delegated_pubkey_algo.to_string(),
            delegated_pubkey_fingerprint: delegated_pubkey_fingerprint.to_string(),
            issued_at_ts,
            expires_at_ts,
            trust_level,
            scope: scope.clone(),
        };
        let bytes = canonical_bytes(&payload, DOMAIN_DELEGATION_CERT_V1).map_err(|e| {
            NexusError::Crypto(DelegationCertError::CanonicalFailed(e.to_string()).to_string())
        })?;
        let node_sig = node_keypair.sign(&bytes);
        Ok(DelegationCert {
            node_id,
            delegated_pubkey_algo: delegated_pubkey_algo.to_string(),
            delegated_pubkey_fingerprint: delegated_pubkey_fingerprint.to_string(),
            issued_at_ts,
            expires_at_ts,
            trust_level,
            scope,
            node_sig,
        })
    }

    /// Verify the Ed25519 signature. Does not check expiry.
    pub fn verify_signature(&self) -> std::result::Result<(), DelegationCertError> {
        let payload = DelegationCertPayload {
            node_id: self.node_id,
            delegated_pubkey_algo: self.delegated_pubkey_algo.clone(),
            delegated_pubkey_fingerprint: self.delegated_pubkey_fingerprint.clone(),
            issued_at_ts: self.issued_at_ts,
            expires_at_ts: self.expires_at_ts,
            trust_level: self.trust_level,
            scope: self.scope.clone(),
        };
        let bytes = canonical_bytes(&payload, DOMAIN_DELEGATION_CERT_V1)
            .map_err(|e| DelegationCertError::CanonicalFailed(e.to_string()))?;
        crate::crypto::verify(&self.node_id, &bytes, &self.node_sig)
            .map_err(|e| DelegationCertError::BadSignature(e.to_string()))
    }

    /// Full verification: signature valid + not expired at `now_ts`.
    pub fn verify(&self, now_ts: i64) -> std::result::Result<(), DelegationCertError> {
        self.verify_signature()?;
        if let Some(exp) = self.expires_at_ts
            && now_ts > exp
        {
            return Err(DelegationCertError::Expired {
                expires_at: exp,
                now: now_ts,
            });
        }
        Ok(())
    }
}

fn validate_algo(algo: &str) -> Result<()> {
    match algo {
        DELEGATION_ALGO_SSH_ED25519 | DELEGATION_ALGO_SSH_RSA | DELEGATION_ALGO_OPENPGP_ED25519 => {
            Ok(())
        }
        _ => Err(NexusError::Crypto(format!(
            "delegated_pubkey_algo: unsupported algorithm {algo:?}"
        ))),
    }
}

fn validate_trust_level(level: u8) -> Result<()> {
    if !(1..=5).contains(&level) {
        return Err(NexusError::Crypto(format!(
            "trust_level: must be 1-5, got {level}"
        )));
    }
    Ok(())
}

fn validate_fingerprint(fp: &str) -> Result<()> {
    if fp.len() != 64 {
        return Err(NexusError::Crypto(format!(
            "delegated_pubkey_fingerprint: expected 64 hex chars, got {}",
            fp.len()
        )));
    }
    if !fp
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(NexusError::Crypto(
            "delegated_pubkey_fingerprint: must be lowercase hex".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_FINGERPRINT: &str =
        "a1b2c3d4e5f6071829304a5b6c7d8e9f0a1b2c3d4e5f6071829304a5b6c7d8e9";

    fn sample_scope() -> DelegationScope {
        DelegationScope {
            org_name: "FlowUP".to_string(),
            forge_urls: vec!["https://github.com/SBFB50/SBFB".to_string()],
        }
    }

    #[test]
    fn sign_verify_roundtrip() {
        let kp = KeyPair::generate();
        let cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect("sign succeeds");
        cert.verify_signature().expect("signature valid");
        assert_eq!(cert.node_id, kp.public_bytes());
        assert_eq!(cert.delegated_pubkey_algo, DELEGATION_ALGO_SSH_ED25519);
        assert_eq!(cert.delegated_pubkey_fingerprint, SAMPLE_FINGERPRINT);
        assert_eq!(cert.issued_at_ts, 1_713_600_000);
        assert_eq!(cert.expires_at_ts, None);
        assert_eq!(cert.trust_level, 3);
        assert_eq!(cert.scope, None);
    }

    #[test]
    fn sign_verify_with_expiry() {
        let kp = KeyPair::generate();
        let cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_RSA,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            Some(1_745_136_000),
            3,
            None,
            &kp,
        )
        .expect("sign succeeds");
        cert.verify(1_713_700_000).expect("not expired");
        let err = cert.verify(1_745_200_000).expect_err("expired");
        assert!(matches!(err, DelegationCertError::Expired { .. }));
    }

    #[test]
    fn verify_rejects_tampered_fingerprint() {
        let kp = KeyPair::generate();
        let mut cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect("sign");
        cert.delegated_pubkey_fingerprint =
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();
        let err = cert.verify_signature().expect_err("tampered");
        assert!(matches!(err, DelegationCertError::BadSignature(_)));
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let kp = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect("sign");
        cert.node_id = impostor.public_bytes();
        let err = cert.verify_signature().expect_err("wrong key");
        assert!(matches!(err, DelegationCertError::BadSignature(_)));
    }

    #[test]
    fn domain_separation_distinct_from_other_payloads() {
        let kp = KeyPair::generate();
        let payload = DelegationCertPayload {
            node_id: kp.public_bytes(),
            delegated_pubkey_algo: DELEGATION_ALGO_SSH_ED25519.to_string(),
            delegated_pubkey_fingerprint: SAMPLE_FINGERPRINT.to_string(),
            issued_at_ts: 1_713_600_000,
            expires_at_ts: None,
            trust_level: 3,
            scope: None,
        };
        let as_delegation = canonical_bytes(&payload, DOMAIN_DELEGATION_CERT_V1).unwrap();
        let as_task = canonical_bytes(&payload, crate::canonical::DOMAIN_TASK_V1).unwrap();
        let as_age = canonical_bytes(&payload, crate::canonical::DOMAIN_AGE_WITNESS_V1).unwrap();
        assert_ne!(as_delegation, as_task);
        assert_ne!(as_delegation, as_age);
    }

    #[test]
    fn serde_roundtrip_json() {
        let kp = KeyPair::generate();
        let cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            Some(1_745_136_000),
            4,
            Some(sample_scope()),
            &kp,
        )
        .expect("sign");
        let json = serde_json::to_string(&cert).expect("serialize");
        let restored: DelegationCert = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(cert, restored);
        restored
            .verify_signature()
            .expect("sig still valid after roundtrip");
    }

    #[test]
    fn rejects_invalid_algo() {
        let kp = KeyPair::generate();
        let err = DelegationCert::sign(
            "pgp-rsa-4096",
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect_err("bad algo");
        assert!(err.to_string().contains("unsupported algorithm"));
    }

    #[test]
    fn rejects_invalid_fingerprint_length() {
        let kp = KeyPair::generate();
        let err = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            "abcdef",
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect_err("bad fp");
        assert!(err.to_string().contains("expected 64 hex chars"));
    }

    #[test]
    fn rejects_uppercase_fingerprint() {
        let kp = KeyPair::generate();
        let upper = "A1B2C3D4E5F6071829304A5B6C7D8E9F0A1B2C3D4E5F6071829304A5B6C7D8E9";
        let err = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            upper,
            1_713_600_000,
            None,
            3,
            None,
            &kp,
        )
        .expect_err("bad fp");
        assert!(err.to_string().contains("lowercase hex"));
    }

    #[test]
    fn delegation_cert_v1_with_trust_level() {
        let kp = KeyPair::generate();
        let cert = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            5,
            Some(sample_scope()),
            &kp,
        )
        .expect("sign");
        assert_eq!(cert.trust_level, 5);
        assert_eq!(cert.scope.as_ref().unwrap().org_name, "FlowUP");
        cert.verify_signature()
            .expect("valid with trust_level + scope");

        let err = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            0,
            None,
            &kp,
        )
        .expect_err("trust_level 0 invalid");
        assert!(err.to_string().contains("trust_level"));

        let err = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            None,
            6,
            None,
            &kp,
        )
        .expect_err("trust_level 6 invalid");
        assert!(err.to_string().contains("trust_level"));
    }

    #[test]
    fn delegation_cert_canonical_jcs_deterministic() {
        let kp = KeyPair::generate();
        let cert1 = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            Some(1_745_136_000),
            4,
            Some(sample_scope()),
            &kp,
        )
        .expect("sign1");
        let cert2 = DelegationCert::sign(
            DELEGATION_ALGO_SSH_ED25519,
            SAMPLE_FINGERPRINT,
            1_713_600_000,
            Some(1_745_136_000),
            4,
            Some(sample_scope()),
            &kp,
        )
        .expect("sign2");
        assert_eq!(
            cert1.node_sig, cert2.node_sig,
            "JCS canonical must be deterministic"
        );
    }
}
