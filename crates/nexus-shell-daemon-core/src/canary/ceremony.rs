// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 30 Phase C — FROST signing ceremony step-by-step.
//!
//! Breaks the in-process [`super::frost::frost_sign_with_shares`]
//! into discrete round-1 / round-2 / aggregate steps with
//! JSON-serializable intermediate types, enabling the air-gapped
//! cross-machine workflow described in
//! `WARRANT_CANARY_HARDENING.md §4.3`.

use std::collections::BTreeMap;

use frost::keys::{KeyPackage, PublicKeyPackage};
use frost::{Identifier, SigningPackage};
use frost_ed25519 as frost;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

use nexus_core_rs::crypto::SIGNATURE_BYTES;

use super::frost::FrostError;

/// Round 1 commitment (public — shared with the coordinator).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyCommitment {
    pub participant: u16,
    pub commitment_hex: String,
}

/// Round 1 nonces (SECRET — kept locally, destroyed after round 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonyNonces {
    pub participant: u16,
    pub nonces_hex: String,
}

/// Coordinator's signing package (distributed to all K participants
/// for round 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonySigningPackage {
    pub signing_package_hex: String,
}

/// Round 2 signature share (shared with the coordinator for
/// aggregation).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CeremonySignatureShare {
    pub participant: u16,
    pub signature_share_hex: String,
}

/// Execute round 1 for one participant: generate nonces and
/// commitment from their key package.
///
/// Returns `(commitment, nonces)`. The commitment is sent to the
/// coordinator; the nonces are kept locally for round 2.
pub fn ceremony_round1(
    participant: u16,
    key_package: &KeyPackage,
) -> Result<(CeremonyCommitment, CeremonyNonces), FrostError> {
    let mut rng = OsRng;
    let (nonces, commitment) = frost::round1::commit(key_package.signing_share(), &mut rng);

    let commitment_bytes = commitment.serialize().map_err(|e| {
        FrostError::Round2(
            format!("{participant}"),
            format!("serialize commitment: {e}"),
        )
    })?;
    let nonces_bytes = nonces.serialize().map_err(|e| {
        FrostError::Round2(format!("{participant}"), format!("serialize nonces: {e}"))
    })?;

    Ok((
        CeremonyCommitment {
            participant,
            commitment_hex: hex::encode(commitment_bytes),
        },
        CeremonyNonces {
            participant,
            nonces_hex: hex::encode(nonces_bytes),
        },
    ))
}

/// Coordinator step: collect commitments from K participants and
/// the message to sign, produce a signing package for distribution.
pub fn build_signing_package(
    commitments: &[CeremonyCommitment],
    message: &[u8],
) -> Result<CeremonySigningPackage, FrostError> {
    let mut map = BTreeMap::new();
    for c in commitments {
        let id = Identifier::try_from(c.participant).map_err(|e| {
            FrostError::Aggregate(format!(
                "identifier from participant {}: {e}",
                c.participant
            ))
        })?;
        let bytes = hex::decode(&c.commitment_hex)
            .map_err(|e| FrostError::Aggregate(format!("decode commitment hex: {e}")))?;
        let commitment = frost::round1::SigningCommitments::deserialize(&bytes)
            .map_err(|e| FrostError::Aggregate(format!("deserialize commitment: {e}")))?;
        map.insert(id, commitment);
    }

    let signing_package = SigningPackage::new(map, message);
    let sp_bytes = signing_package
        .serialize()
        .map_err(|e| FrostError::Aggregate(format!("serialize signing package: {e}")))?;

    Ok(CeremonySigningPackage {
        signing_package_hex: hex::encode(sp_bytes),
    })
}

/// Execute round 2 for one participant: produce a signature share
/// from their nonces, the signing package, and their key package.
pub fn ceremony_round2(
    nonces: &CeremonyNonces,
    signing_package: &CeremonySigningPackage,
    key_package: &KeyPackage,
) -> Result<CeremonySignatureShare, FrostError> {
    let p = format!("{}", nonces.participant);

    let nonces_bytes = hex::decode(&nonces.nonces_hex)
        .map_err(|e| FrostError::Round2(p.clone(), format!("decode nonces hex: {e}")))?;
    let signing_nonces = frost::round1::SigningNonces::deserialize(&nonces_bytes)
        .map_err(|e| FrostError::Round2(p.clone(), format!("deserialize nonces: {e}")))?;

    let sp_bytes = hex::decode(&signing_package.signing_package_hex)
        .map_err(|e| FrostError::Round2(p.clone(), format!("decode signing package hex: {e}")))?;
    let sp = SigningPackage::deserialize(&sp_bytes)
        .map_err(|e| FrostError::Round2(p.clone(), format!("deserialize signing package: {e}")))?;

    let sig_share = frost::round2::sign(&sp, &signing_nonces, key_package)
        .map_err(|e| FrostError::Round2(p, e.to_string()))?;

    let share_bytes = sig_share.serialize();

    Ok(CeremonySignatureShare {
        participant: nonces.participant,
        signature_share_hex: hex::encode(share_bytes),
    })
}

/// Coordinator step: aggregate K signature shares into a single
/// 64-byte Ed25519 signature.
pub fn ceremony_aggregate(
    signing_package: &CeremonySigningPackage,
    shares: &[CeremonySignatureShare],
    pubkey_package: &PublicKeyPackage,
) -> Result<[u8; SIGNATURE_BYTES], FrostError> {
    let sp_bytes = hex::decode(&signing_package.signing_package_hex)
        .map_err(|e| FrostError::Aggregate(format!("decode signing package hex: {e}")))?;
    let sp = SigningPackage::deserialize(&sp_bytes)
        .map_err(|e| FrostError::Aggregate(format!("deserialize signing package: {e}")))?;

    let mut sig_shares = BTreeMap::new();
    for s in shares {
        let id = Identifier::try_from(s.participant).map_err(|e| {
            FrostError::Aggregate(format!(
                "identifier from participant {}: {e}",
                s.participant
            ))
        })?;
        let bytes = hex::decode(&s.signature_share_hex)
            .map_err(|e| FrostError::Aggregate(format!("decode sig share hex: {e}")))?;
        let sig_share = frost::round2::SignatureShare::deserialize(&bytes)
            .map_err(|e| FrostError::Aggregate(format!("deserialize sig share: {e}")))?;
        sig_shares.insert(id, sig_share);
    }

    let aggregated = frost::aggregate(&sp, &sig_shares, pubkey_package)
        .map_err(|e| FrostError::Aggregate(e.to_string()))?;

    let bytes = aggregated
        .serialize()
        .map_err(|e| FrostError::Aggregate(format!("serialize signature: {e}")))?;

    bytes
        .try_into()
        .map_err(|v: Vec<u8>| FrostError::BadSigLength { got: v.len() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canary::dkg::{generate_dkg, load_pubkey, load_share};
    use crate::canary::frost::FrostKeyShare;
    use crate::canary::verify_canary;
    use nexus_core_rs::crypto::verify;
    use time::Date;

    fn a_date() -> Date {
        Date::from_calendar_date(2026, time::Month::April, 26).unwrap()
    }

    fn setup_dkg() -> (
        Vec<FrostKeyShare>,
        PublicKeyPackage,
        Vec<crate::canary::dkg::DkgShareFile>,
        crate::canary::dkg::DkgPubkeyFile,
    ) {
        let (share_files, pubkey_file) = generate_dkg(2, 3).expect("DKG K=2/N=3");
        let shares: Vec<FrostKeyShare> = share_files
            .iter()
            .map(|sf| load_share(sf).expect("load share"))
            .collect();
        let pubkey = load_pubkey(&pubkey_file).expect("load pubkey");
        let pp = pubkey.package().clone();
        (shares, pp, share_files, pubkey_file)
    }

    #[test]
    fn ceremony_full_roundtrip_3_participants() {
        let (shares, pubkey_package, share_files, pubkey_file) = setup_dkg();
        let message = b"FROST ceremony roundtrip test 2026-04-26";

        let (c1, n1) = ceremony_round1(share_files[0].participant, &shares[0].key_package)
            .expect("round1 participant 1");
        let (c2, n2) = ceremony_round1(share_files[1].participant, &shares[1].key_package)
            .expect("round1 participant 2");

        let sp = build_signing_package(&[c1, c2], message).expect("build signing package");

        let ss1 = ceremony_round2(&n1, &sp, &shares[0].key_package).expect("round2 participant 1");
        let ss2 = ceremony_round2(&n2, &sp, &shares[1].key_package).expect("round2 participant 2");

        let sig = ceremony_aggregate(&sp, &[ss1, ss2], &pubkey_package).expect("aggregate");

        let vk_bytes = hex::decode(&pubkey_file.verifying_key_hex).expect("decode vk");
        let vk: [u8; 32] = vk_bytes.try_into().expect("vk 32 bytes");
        verify(&vk, message, &sig).expect("aggregated sig verifies as Ed25519");
    }

    #[test]
    fn ceremony_insufficient_signers_rejected() {
        let (shares, _pubkey_package, share_files, _) = setup_dkg();
        let message = b"insufficient signers test";

        let (c1, n1) =
            ceremony_round1(share_files[0].participant, &shares[0].key_package).expect("round1");

        let sp = build_signing_package(&[c1], message).expect("build sp with 1 commitment");

        // FROST catches insufficient commitments at round2 (not
        // aggregate): the signing package has fewer than K=2
        // commitments, so round2::sign rejects immediately.
        let result = ceremony_round2(&n1, &sp, &shares[0].key_package);
        assert!(result.is_err(), "round2 with 1 commitment on K=2 must fail");
    }

    #[test]
    fn ceremony_tampered_message_detected() {
        let (shares, pubkey_package, share_files, pubkey_file) = setup_dkg();

        let (c1, n1) =
            ceremony_round1(share_files[0].participant, &shares[0].key_package).expect("round1 p1");
        let (c2, n2) =
            ceremony_round1(share_files[1].participant, &shares[1].key_package).expect("round1 p2");

        let sp = build_signing_package(&[c1, c2], b"original message").expect("build sp");

        let ss1 = ceremony_round2(&n1, &sp, &shares[0].key_package).expect("round2 p1");
        let ss2 = ceremony_round2(&n2, &sp, &shares[1].key_package).expect("round2 p2");

        let sig = ceremony_aggregate(&sp, &[ss1, ss2], &pubkey_package).expect("aggregate");

        let vk_bytes = hex::decode(&pubkey_file.verifying_key_hex).expect("vk");
        let vk: [u8; 32] = vk_bytes.try_into().expect("32 bytes");
        let tampered_result = verify(&vk, b"tampered message", &sig);
        assert!(
            tampered_result.is_err(),
            "tampered message must fail verify"
        );
    }

    #[test]
    fn ceremony_produces_canary_compatible_signature() {
        let (shares, pubkey_package, share_files, pubkey_file) = setup_dkg();
        let headline = "ceremony canary headline";
        let date = a_date();
        let next_update =
            date.saturating_add(time::Duration::days(super::super::CANARY_VALIDITY_DAYS));
        let signed = super::super::CanarySigned {
            version: super::super::CANARY_VERSION,
            date: format!(
                "{:04}-{:02}-{:02}",
                date.year(),
                u8::from(date.month()),
                date.day()
            ),
            headline: headline.to_string(),
            next_update: format!(
                "{:04}-{:02}-{:02}",
                next_update.year(),
                u8::from(next_update.month()),
                next_update.day()
            ),
            pubkey_hex: pubkey_file.verifying_key_hex.clone(),
        };
        let canonical = nexus_core_rs::canonical::canonical_bytes(
            &signed,
            nexus_core_rs::canonical::DOMAIN_WARRANT_CANARY_V1,
        )
        .expect("canonical bytes");

        let (c1, n1) =
            ceremony_round1(share_files[0].participant, &shares[0].key_package).expect("round1 p1");
        let (c2, n2) =
            ceremony_round1(share_files[1].participant, &shares[1].key_package).expect("round1 p2");

        let sp = build_signing_package(&[c1, c2], &canonical).expect("build sp");

        let ss1 = ceremony_round2(&n1, &sp, &shares[0].key_package).expect("round2 p1");
        let ss2 = ceremony_round2(&n2, &sp, &shares[1].key_package).expect("round2 p2");

        let sig = ceremony_aggregate(&sp, &[ss1, ss2], &pubkey_package).expect("aggregate");

        let canary = super::super::Canary {
            signed,
            signature_hex: hex::encode(sig),
        };
        verify_canary(&canary).expect("canary built via ceremony verifies");
    }
}
