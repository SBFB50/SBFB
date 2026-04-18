// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase E.5 — `AttestationProvider` trait.
//!
//! Decouple the **signing** of a warrant canary
//! ([`super::CanarySigner`]) from any **attestation** the
//! maintainer might want to embed alongside it (TEE quote,
//! transparency log inclusion proof, third-party notary
//! receipt…). Phase E only ships the
//! `NoopAttestation` impl; real backends (Intel TDX quote, AMD
//! SEV-SNP report, AWS Nitro Enclaves attestation document) are
//! the Sprint 25-30 hardware-attestation track scoped in
//! `docs/security/HARDENING_ROADMAP.md §3` line "Warrant canary
//! Niveau 1 enforcement".
//!
//! ## Why decouple now
//!
//! The current single-key, human-driven canary flow has two
//! independent strengthening axes:
//!
//! 1. **Signing trust** : single Ed25519 key → threshold FROST
//!    K-of-N (Phase E.2). Lives in [`super::CanarySigner`].
//! 2. **Process trust** : "trust me, I ran this on my laptop"
//!    → "this signature was produced inside an attested TEE
//!    that loaded a measured supply-chain-signed binary". Lives
//!    here.
//!
//! Putting the two on the same trait would force every TEE-less
//! maintainer to drag in a hardware dependency they cannot use,
//! and conversely force every TEE-attested maintainer to
//! re-implement signing from scratch. A separate
//! [`AttestationProvider`] keeps both axes orthogonal, which is
//! also how the Confidential Computing Consortium pattern recom-
//! mends combining the two (cf. CC SIG TEE Attestation
//! Architecture v1.2, §4.3 "separation of signing and
//! attestation surfaces").

use serde::{Deserialize, Serialize};

/// An opaque attestation receipt. The `kind` discriminator is
/// the public, freezeable identifier verifiers use to dispatch
/// to the right verification routine (e.g. "tdx-quote-v4",
/// "snp-report", "nitro-attestation-doc"). The `payload` is the
/// raw bytes of the receipt as defined by that backend's spec.
///
/// `NoopAttestation` always returns `kind = "noop"` with an
/// empty payload — verifiers MUST treat that as "no attestation
/// was claimed" and fall back to plain signature verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    /// Backend identifier. Lowercase, ASCII, kebab-case. Frozen
    /// per backend; never reused across incompatible payload
    /// formats (analogous to a `_VERSION` field in a wire struct).
    pub kind: String,

    /// Backend-specific receipt bytes.
    #[serde(with = "hex_serde")]
    pub payload: Vec<u8>,
}

/// Trait implemented by every attestation backend.
///
/// Implementations are typically platform-specific and gated on
/// hardware availability — `NoopAttestation` is the universal
/// fallback that requires no special environment.
pub trait AttestationProvider: Send + Sync {
    /// Return an [`Attestation`] over `binding`.
    ///
    /// `binding` is the canonical bytes the attestation should
    /// commit to (typically `blake3(canary_canonical_bytes)` —
    /// the caller decides). Implementations SHOULD include
    /// `binding` in whatever data the underlying TEE/notary
    /// signs so the receipt cryptographically binds to the
    /// canary content rather than just attesting "the binary
    /// ran".
    fn attest(&self, binding: &[u8]) -> Attestation;
}

/// Universal fallback that asserts no attestation is claimed.
/// Verifiers seeing `kind = "noop"` MUST NOT treat the canary
/// as TEE-backed; the canary is still cryptographically valid
/// via its [`super::CanarySigner`] signature, just without the
/// hardware-rooted process trust layer.
pub struct NoopAttestation;

impl AttestationProvider for NoopAttestation {
    fn attest(&self, _binding: &[u8]) -> Attestation {
        Attestation {
            kind: "noop".to_string(),
            payload: Vec::new(),
        }
    }
}

mod hex_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(de: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(de)?;
        hex::decode(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canary::{build_canary, signer::Ed25519CanarySigner, verify_canary};
    use nexus_core_rs::KeyPair;
    use time::Date;

    #[test]
    fn noop_attestation_returns_noop_kind_empty_payload() {
        let provider = NoopAttestation;
        let receipt = provider.attest(b"any binding bytes");
        assert_eq!(receipt.kind, "noop");
        assert!(receipt.payload.is_empty());

        // Stable across calls — pure no-op.
        let again = provider.attest(b"different binding");
        assert_eq!(again, receipt);
    }

    #[test]
    fn signer_decoupled_from_attestation_provider() {
        // Phase E.5 invariant : a CanarySigner on its own is
        // sufficient to produce a verifiable canary. The
        // AttestationProvider is an orthogonal axis that can be
        // added (Sprint 25-30) without changing the wire format
        // or the verify path.
        let signer = Ed25519CanarySigner::new(KeyPair::generate());
        let date = Date::from_calendar_date(2026, time::Month::April, 18).unwrap();
        let canary = build_canary(date, "decoupled signer test".into(), &signer)
            .expect("build canary without any attestation");

        // Standalone signature verifies — no attestation required.
        verify_canary(&canary).expect("canary verifies even with no attestation");

        // Attestation, when produced, is independent — its
        // `noop` payload doesn't appear anywhere in the canary.
        let provider = NoopAttestation;
        let _receipt = provider.attest(b"orthogonal binding");
    }
}
