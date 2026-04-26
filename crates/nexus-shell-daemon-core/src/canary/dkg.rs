// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 30 Phase C — FROST DKG distribution layer.
//!
//! Wraps [`super::frost::frost_keygen_trusted_dealer`] (Sprint 20
//! Phase E.2) with JSON-serializable file representations suitable
//! for air-gapped cross-machine distribution of FROST K-of-N key
//! shares per `WARRANT_CANARY_HARDENING.md §4.2`.

use frost::keys::{KeyPackage, PublicKeyPackage};
use frost_ed25519 as frost;
use serde::{Deserialize, Serialize};

use super::frost::{frost_keygen_trusted_dealer, FrostError, FrostKeyShare, FrostPubkey};

/// JSON-serializable individual FROST key share for distribution
/// to one participant's air-gapped machine.
///
/// Written by `sbfb canary frost trusted-dealer` as
/// `canary-share-{participant}.frost.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgShareFile {
    /// 1-based participant index (matches `IdentifierList::Default`
    /// ordering from `generate_with_dealer`).
    pub participant: u16,
    /// Hex-encoded serialized `frost::keys::KeyPackage` (postcard).
    pub key_package_hex: String,
    /// K threshold (how many shares must cooperate to sign).
    pub min_signers: u16,
    /// N total shares dealt.
    pub max_signers: u16,
}

/// JSON-serializable FROST public key package. Published in
/// `CANARY.txt` and shared with all verifiers.
///
/// Written by `sbfb canary frost trusted-dealer` as
/// `canary-pubkey-package.frost.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DkgPubkeyFile {
    /// 32-byte Ed25519 verifying key in hex (the "public key" that
    /// verifiers use to check aggregated signatures).
    pub verifying_key_hex: String,
    /// Hex-encoded serialized `frost::keys::PublicKeyPackage`
    /// (postcard). Needed by the coordinator at aggregate time.
    pub pubkey_package_hex: String,
    /// K threshold.
    pub min_signers: u16,
    /// N total shares.
    pub max_signers: u16,
}

/// Run the FROST trusted-dealer DKG and return JSON-serializable
/// file representations of each share and the public key package.
///
/// `min_signers` is K (threshold), `max_signers` is N (total).
/// RFC 9591 §6 requires `2 <= K <= N`.
pub fn generate_dkg(
    min_signers: u16,
    max_signers: u16,
) -> Result<(Vec<DkgShareFile>, DkgPubkeyFile), FrostError> {
    let (shares, pubkey) = frost_keygen_trusted_dealer(min_signers, max_signers)?;

    let mut share_files = Vec::with_capacity(shares.len());
    for (i, share) in shares.iter().enumerate() {
        let kp_bytes = share
            .key_package
            .serialize()
            .map_err(|e| FrostError::Keygen(format!("serialize key package: {e}")))?;
        share_files.push(DkgShareFile {
            participant: (i + 1) as u16,
            key_package_hex: hex::encode(kp_bytes),
            min_signers,
            max_signers,
        });
    }

    let vk_bytes = pubkey.to_bytes()?;
    let pp_bytes = pubkey
        .package()
        .serialize()
        .map_err(|e| FrostError::Keygen(format!("serialize pubkey package: {e}")))?;

    let pubkey_file = DkgPubkeyFile {
        verifying_key_hex: hex::encode(vk_bytes),
        pubkey_package_hex: hex::encode(pp_bytes),
        min_signers,
        max_signers,
    };

    Ok((share_files, pubkey_file))
}

/// Deserialize a [`DkgShareFile`] back into a [`FrostKeyShare`].
pub fn load_share(file: &DkgShareFile) -> Result<FrostKeyShare, FrostError> {
    let kp_bytes = hex::decode(&file.key_package_hex)
        .map_err(|e| FrostError::KeyPackageConversion(format!("decode key package hex: {e}")))?;
    let key_package = KeyPackage::deserialize(&kp_bytes)
        .map_err(|e| FrostError::KeyPackageConversion(format!("deserialize key package: {e}")))?;
    let identifier = *key_package.identifier();
    Ok(FrostKeyShare {
        identifier,
        key_package,
    })
}

/// Deserialize a [`DkgPubkeyFile`] back into a [`FrostPubkey`].
pub fn load_pubkey(file: &DkgPubkeyFile) -> Result<FrostPubkey, FrostError> {
    let pp_bytes = hex::decode(&file.pubkey_package_hex)
        .map_err(|e| FrostError::Keygen(format!("decode pubkey package hex: {e}")))?;
    let pubkey_package = PublicKeyPackage::deserialize(&pp_bytes)
        .map_err(|e| FrostError::Keygen(format!("deserialize pubkey package: {e}")))?;
    Ok(FrostPubkey::from_package(pubkey_package))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canary::{build_canary, verify_canary};
    use nexus_core_rs::crypto::verify;
    use time::Date;

    fn a_date() -> Date {
        Date::from_calendar_date(2026, time::Month::April, 26).unwrap()
    }

    #[test]
    fn dkg_generate_serialize_roundtrip() {
        let (share_files, pubkey_file) = generate_dkg(2, 3).expect("generate DKG K=2/N=3");
        assert_eq!(share_files.len(), 3);
        assert_eq!(pubkey_file.min_signers, 2);
        assert_eq!(pubkey_file.max_signers, 3);
        assert_eq!(pubkey_file.verifying_key_hex.len(), 64);

        for (i, sf) in share_files.iter().enumerate() {
            assert_eq!(sf.participant, (i + 1) as u16);
            assert_eq!(sf.min_signers, 2);
            assert_eq!(sf.max_signers, 3);
            assert!(!sf.key_package_hex.is_empty());
        }

        let json = serde_json::to_string_pretty(&share_files[0]).expect("share to JSON");
        let deser: DkgShareFile = serde_json::from_str(&json).expect("JSON to share");
        let _share = load_share(&deser).expect("load deserialized share");

        let pubkey = load_pubkey(&pubkey_file).expect("load pubkey");
        let vk = pubkey.to_bytes().expect("verifying key bytes");
        assert_eq!(hex::encode(vk), pubkey_file.verifying_key_hex);
    }

    #[test]
    fn dkg_roundtrip_produces_valid_signature() {
        let (share_files, pubkey_file) = generate_dkg(2, 3).expect("generate DKG");

        let shares: Vec<FrostKeyShare> = share_files
            .iter()
            .map(|sf| load_share(sf).expect("load share"))
            .collect();
        let pubkey = load_pubkey(&pubkey_file).expect("load pubkey");

        let msg = b"DKG roundtrip test 2026-04-26";
        let sig = super::super::frost::frost_sign_with_shares(&shares[..2], &pubkey, msg)
            .expect("sign with 2 of 3 shares");

        let vk = pubkey.to_bytes().expect("verifying key");
        verify(&vk, msg, &sig).expect("Ed25519 verify");
    }

    #[test]
    fn dkg_roundtrip_canary_verifies() {
        let (share_files, pubkey_file) = generate_dkg(2, 3).expect("generate DKG");

        let shares: Vec<FrostKeyShare> = share_files
            .iter()
            .map(|sf| load_share(sf).expect("load share"))
            .collect();
        let pubkey = load_pubkey(&pubkey_file).expect("load pubkey");

        let signer = super::super::frost::FrostCanarySigner::from_parts(shares, pubkey, 2);
        let canary =
            build_canary(a_date(), "DKG roundtrip headline".into(), &signer).expect("build canary");
        verify_canary(&canary).expect("canary verifies");
    }

    #[test]
    fn dkg_rejects_invalid_params() {
        assert!(generate_dkg(1, 1).is_err(), "K=1 must be rejected");
        assert!(generate_dkg(0, 3).is_err(), "K=0 must be rejected");
        assert!(generate_dkg(3, 2).is_err(), "K>N must be rejected");
    }
}
