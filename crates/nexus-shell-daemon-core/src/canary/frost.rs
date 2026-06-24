// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase E.2 — FROST-ed25519 threshold canary signer.
//!
//! Wires the [`super::CanarySigner`] trait over a FROST K-of-N
//! threshold scheme (RFC 9591 jan 2025, ZcashFoundation
//! reference implementation, Trail of Bits audit 2023). The
//! aggregated signature [`FrostCanarySigner::sign`] returns is a
//! 64-byte Ed25519 RFC 8032 signature byte-identical to one a
//! standalone [`super::Ed25519CanarySigner`] would produce, so
//! the entire verifier path
//! ([`nexus_core_rs::crypto::verify`], `verify_canary`,
//! `parse_canary_txt`) keeps working unchanged.
//!
//! ## Threat coverage
//!
//! - **T-canary-key-exfil** : K=2/N=3 means an attacker who
//!   compromises one maintainer key cannot publish canaries on
//!   their own — they need to coerce K=2 maintainers
//!   simultaneously. With cross-juridiction recruitment (Sprint
//!   25-30 enforcement track) the attacker faces K legal-system
//!   barriers in parallel.
//! - **T-canary-gag-order** : a single subpoenaed maintainer can
//!   refuse to produce their FROST signature share. The K=2/N=3
//!   threshold means the canary can still be aggregated by 2
//!   non-coerced maintainers ; if 2+ are coerced, the canary
//!   stops publishing — dead-man-switch fires, exactly as
//!   designed.
//!
//! ## Scope (Phase E.2 = primitive scaffolding)
//!
//! This module ships the **in-process** primitive : a single
//! [`FrostCanarySigner`] holds all K signing key packages and
//! runs the round-1 / round-2 / aggregate sequence locally. That
//! is enough to validate the wire-format invariant
//! (FROST sig = standalone Ed25519 sig) and provide the
//! abstraction the cross-juridiction federation layer (Sprint
//! 25-30) will plug into. The real DKG-cross-process orchestration
//! layer is out of scope here per the G8 phase pre-flight verdict.
//!
//! ## Minimum threshold = K=2/N=2
//!
//! RFC 9591 §6.1 requires `K >= 2` (a "K=1 threshold" is
//! degenerate ; `frost-ed25519` v2.x rejects it at construct
//! time, surfaced here as `FrostError::Keygen`). The smallest
//! legitimate FROST configuration is therefore K=2/N=2 (both
//! shares cooperate, no redundancy), and the most useful
//! starting point is K=2/N=3 (one share can be lost without
//! losing signing capability). For a K=1-equivalent, use
//! [`super::Ed25519CanarySigner`] directly — the `CanarySigner`
//! trait abstraction makes that swap a single line at the call
//! site.

use std::collections::BTreeMap;

use frost::keys::{IdentifierList, KeyPackage, PublicKeyPackage};
use frost::{Identifier, Signature, SigningPackage};
use frost_ed25519 as frost;
use rand::rngs::OsRng;
use thiserror::Error;

use nexus_core_rs::crypto::{PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};

use super::signer::CanarySigner;

/// Errors the FROST canary path can surface to callers.
#[derive(Debug, Error)]
pub enum FrostError {
    /// FROST keygen via trusted dealer failed (e.g. invalid
    /// `(min_signers, max_signers)` parameters per RFC 9591 §6).
    #[error("frost trusted dealer keygen failed: {0}")]
    Keygen(String),

    /// Converting a `SecretShare` into a `KeyPackage` failed —
    /// indicates a corrupted secret share, never expected on
    /// freshly-dealt keys.
    #[error("frost share -> key package conversion failed: {0}")]
    KeyPackageConversion(String),

    /// Round 2 share generation failed for a participant.
    #[error("frost round 2 sign failed for participant {0:?}: {1}")]
    Round2(String, String),

    /// Aggregation rejected the supplied signature shares — for
    /// example, fewer than `min_signers` shares supplied, or a
    /// tampered share that fails internal verification.
    #[error("frost aggregate failed: {0}")]
    Aggregate(String),

    /// The FROST verifying key bytes did not have the expected
    /// 32-byte Ed25519 length. Should never happen with the
    /// `frost-ed25519` ciphersuite.
    #[error("verifying key has unexpected length: {got} bytes, expected {PUBLIC_KEY_LENGTH}")]
    BadPubkeyLength { got: usize },

    /// The FROST aggregated signature did not have the expected
    /// 64-byte Ed25519 length.
    #[error("aggregated signature has unexpected length: {got} bytes, expected {SIGNATURE_BYTES}")]
    BadSigLength { got: usize },
}

/// A single FROST K-of-N share held by one participant.
///
/// In the Phase E.2 in-process scaffolding, the
/// [`FrostCanarySigner`] holds all `N` of these locally; moving
/// them onto K independent maintainer machines (the distribution
/// layer) is tracked future work, not yet built.
#[derive(Debug, Clone)]
pub struct FrostKeyShare {
    pub identifier: Identifier,
    pub key_package: KeyPackage,
}

/// The shared FROST verifying key — distributed publicly so
/// verifiers can validate aggregated signatures with the
/// standard Ed25519 verify path.
///
/// Derived from a [`PublicKeyPackage`]. Its byte serialization
/// is the 32-byte Ed25519 public key embedded in
/// [`super::CanarySigned::pubkey_hex`].
#[derive(Debug, Clone)]
pub struct FrostPubkey {
    inner: PublicKeyPackage,
}

impl FrostPubkey {
    /// Construct from a deserialized [`PublicKeyPackage`].
    pub fn from_package(pkg: PublicKeyPackage) -> Self {
        Self { inner: pkg }
    }

    /// Borrow the underlying FROST public key package.
    pub fn package(&self) -> &PublicKeyPackage {
        &self.inner
    }

    /// Serialize the verifying key to its 32-byte Ed25519
    /// representation. Returns the bytes that go into
    /// `pubkey_hex` on the wire.
    pub fn to_bytes(&self) -> Result<[u8; PUBLIC_KEY_LENGTH], FrostError> {
        let bytes = self
            .inner
            .verifying_key()
            .serialize()
            .map_err(|e| FrostError::Keygen(format!("serialize verifying key: {e}")))?;
        bytes
            .try_into()
            .map_err(|v: Vec<u8>| FrostError::BadPubkeyLength { got: v.len() })
    }
}

/// FROST trusted-dealer keygen — produces `max_signers` key
/// shares and the shared public key package.
///
/// Used directly by [`FrostCanarySigner::trusted_dealer`] as
/// well as by tests that exercise low-level aggregate / tamper
/// behaviour.
pub fn frost_keygen_trusted_dealer(
    min_signers: u16,
    max_signers: u16,
) -> Result<(Vec<FrostKeyShare>, FrostPubkey), FrostError> {
    let rng = OsRng;
    let (shares, pubkey_package) =
        frost::keys::generate_with_dealer(max_signers, min_signers, IdentifierList::Default, rng)
            .map_err(|e| FrostError::Keygen(e.to_string()))?;

    let mut key_shares = Vec::with_capacity(shares.len());
    for (id, secret_share) in shares {
        let kp = KeyPackage::try_from(secret_share)
            .map_err(|e| FrostError::KeyPackageConversion(e.to_string()))?;
        key_shares.push(FrostKeyShare {
            identifier: id,
            key_package: kp,
        });
    }

    Ok((
        key_shares,
        FrostPubkey {
            inner: pubkey_package,
        },
    ))
}

/// Run a full round-1 / round-2 / aggregate sequence over
/// `shares` for `message`. Returns the aggregated 64-byte
/// Ed25519 signature.
///
/// The `shares` slice MUST hold exactly `min_signers` (or more)
/// entries, all with distinct identifiers, otherwise FROST's
/// internal `aggregate` step will reject the result. This is
/// what the `frost_aggregate_refuses_partial_below_k_threshold`
/// test exercises.
pub fn frost_sign_with_shares(
    shares: &[FrostKeyShare],
    pubkey: &FrostPubkey,
    message: &[u8],
) -> Result<[u8; SIGNATURE_BYTES], FrostError> {
    let mut rng = OsRng;

    // Round 1 — each participant produces (nonces, commitments).
    let mut commitments = BTreeMap::new();
    let mut nonces_by_id = BTreeMap::new();
    for share in shares {
        let (nonces, commitment) =
            frost::round1::commit(share.key_package.signing_share(), &mut rng);
        commitments.insert(share.identifier, commitment);
        nonces_by_id.insert(share.identifier, nonces);
    }

    // SigningPackage — the coordinator-side bundle handed back
    // to every signer for round 2.
    let signing_package = SigningPackage::new(commitments, message);

    // Round 2 — each participant produces a signature share.
    let mut sig_shares = BTreeMap::new();
    for share in shares {
        let nonces = nonces_by_id
            .get(&share.identifier)
            .expect("nonces inserted alongside commitments above");
        let sig_share = frost::round2::sign(&signing_package, nonces, &share.key_package)
            .map_err(|e| FrostError::Round2(format!("{:?}", share.identifier), e.to_string()))?;
        sig_shares.insert(share.identifier, sig_share);
    }

    // Aggregate — the coordinator combines the K shares into a
    // single 64-byte Ed25519 signature.
    let aggregated: Signature = frost::aggregate(&signing_package, &sig_shares, pubkey.package())
        .map_err(|e| FrostError::Aggregate(e.to_string()))?;

    let bytes = aggregated
        .serialize()
        .map_err(|e| FrostError::Aggregate(format!("serialize signature: {e}")))?;

    bytes
        .try_into()
        .map_err(|v: Vec<u8>| FrostError::BadSigLength { got: v.len() })
}

/// Threshold-signing implementation of [`CanarySigner`] backed
/// by FROST K-of-N over Ed25519.
///
/// In Phase E.2 baseline scaffolding, this struct holds all `N`
/// key shares in-process and runs the K-of-N protocol locally
/// when [`CanarySigner::sign`] is called. The `min_signers`
/// field records the K threshold the keygen targeted.
///
/// The [`Cargo.toml`] dep comment + module-level doc explain
/// why the in-process layout is acceptable for the primitive
/// scaffolding tier (Niveau 0); cross-juridiction distribution
/// to K independent maintainer machines is the Niveau 1
/// enforcement track scheduled Sprint 25-30.
#[derive(Debug)]
pub struct FrostCanarySigner {
    shares: Vec<FrostKeyShare>,
    pubkey: FrostPubkey,
    min_signers: u16,
}

impl FrostCanarySigner {
    /// Build a fresh threshold signer via the FROST trusted-dealer
    /// keygen. `min_signers` is the K threshold (how many shares
    /// must cooperate to sign), `max_signers` is N (how many
    /// shares are dealt). RFC 9591 §6 requires
    /// `1 <= K <= N <= 65535`.
    pub fn trusted_dealer(min_signers: u16, max_signers: u16) -> Result<Self, FrostError> {
        let (shares, pubkey) = frost_keygen_trusted_dealer(min_signers, max_signers)?;
        Ok(Self {
            shares,
            pubkey,
            min_signers,
        })
    }

    /// Construct from pre-existing shares and pubkey (e.g. loaded
    /// from DKG share files via [`super::dkg::load_share`]).
    pub fn from_parts(shares: Vec<FrostKeyShare>, pubkey: FrostPubkey, min_signers: u16) -> Self {
        Self {
            shares,
            pubkey,
            min_signers,
        }
    }

    /// Borrow the K shares the signer holds. Useful for tests
    /// that need to exercise tamper / partial-aggregate paths.
    pub fn shares(&self) -> &[FrostKeyShare] {
        &self.shares
    }

    /// Borrow the shared FROST verifying key.
    pub fn pubkey_package(&self) -> &FrostPubkey {
        &self.pubkey
    }

    /// Return the K threshold this signer was set up with.
    pub fn min_signers(&self) -> u16 {
        self.min_signers
    }
}

impl CanarySigner for FrostCanarySigner {
    fn pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        // The trusted-dealer keygen succeeded, so the verifying
        // key serializes to exactly 32 bytes (Ed25519
        // ciphersuite invariant). expect() is acceptable here :
        // a failure would mean the FROST library returned an
        // out-of-spec key, which is a programmer / library bug,
        // not a user input error.
        self.pubkey
            .to_bytes()
            .expect("frost-ed25519 verifying key always serializes to 32 bytes")
    }

    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES] {
        // We hold all `min_signers` shares in-process and we
        // produced them ourselves via trusted_dealer, so the
        // round1 / round2 / aggregate sequence cannot fail with
        // valid inputs. expect() is the right tool : a panic
        // here would indicate a bug in `frost-ed25519` or in
        // our share construction, not a runtime condition.
        frost_sign_with_shares(
            &self.shares[..self.min_signers as usize],
            &self.pubkey,
            message,
        )
        .expect("in-process FROST sign with self-dealt shares cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canary::{build_canary, signer::Ed25519CanarySigner, verify_canary};
    use nexus_core_rs::KeyPair;
    use nexus_core_rs::crypto::verify;
    use time::Date;

    fn a_date() -> Date {
        Date::from_calendar_date(2026, time::Month::April, 18).unwrap()
    }

    #[test]
    fn frost_dkg_k2_n3_produces_valid_ed25519_sig() {
        // K=2, N=3 — 3 shares dealt, 2 required to sign.
        let signer = FrostCanarySigner::trusted_dealer(2, 3)
            .expect("FROST trusted dealer keygen with (K=2, N=3) succeeds");
        assert_eq!(signer.min_signers(), 2);
        assert_eq!(signer.shares().len(), 3);

        let pubkey = signer.pubkey();
        assert_eq!(pubkey.len(), PUBLIC_KEY_LENGTH);

        let msg = b"FROST K=2/N=3 canary test 2026-04-18";
        let sig = signer.sign(msg);
        assert_eq!(sig.len(), SIGNATURE_BYTES);

        // The aggregated signature MUST be a valid Ed25519
        // RFC 8032 signature against the same 32-byte verifying
        // key — this is the wire-format invariant
        // CanarySigned v1 preservation depends on.
        verify(&pubkey, msg, &sig).expect("FROST K=2/N=3 sig verifies as a standalone Ed25519 sig");

        // And the canary built with this signer round-trips
        // through the standard verify_canary path.
        let canary = build_canary(a_date(), "FROST K=2/N=3 headline".into(), &signer)
            .expect("build_canary with FrostCanarySigner");
        verify_canary(&canary).expect("canary built by FROST signer verifies via standard path");
    }

    #[test]
    fn frost_minimum_threshold_k2_n2_round_trips_as_ed25519() {
        // RFC 9591 §6.1 requires K >= 2 (a "K=1 threshold" is
        // degenerate and the frost-ed25519 v2.x trusted dealer
        // explicitly rejects it). The minimum legitimate FROST
        // configuration is therefore K=2/N=2 — both shares are
        // needed to sign, no redundancy. Anyone wanting a
        // K=1-equivalent should use Ed25519CanarySigner directly
        // (Phase E.1 baseline) — that is exactly what the
        // CanarySigner trait abstraction enables.
        let signer = FrostCanarySigner::trusted_dealer(2, 2)
            .expect("FROST trusted dealer keygen with (K=2, N=2) succeeds");
        assert_eq!(signer.min_signers(), 2);
        assert_eq!(signer.shares().len(), 2);

        let pubkey = signer.pubkey();
        let msg = b"FROST K=2/N=2 minimum threshold";
        let sig = signer.sign(msg);

        // Same byte-for-byte verifier path as a plain
        // Ed25519CanarySigner produces.
        verify(&pubkey, msg, &sig).expect("K=2/N=2 sig verifies as Ed25519");

        // Property : the FROST K=2/N=2 sig is structurally a
        // standalone Ed25519 sig (64 bytes, decodable by
        // ed25519-dalek). We don't assert byte-equality with a
        // hypothetical Ed25519 sig over the same message because
        // FROST nonces are randomized — only the verify path
        // commitment is deterministic.
        assert_eq!(sig.len(), SIGNATURE_BYTES);
    }

    #[test]
    fn frost_trusted_dealer_rejects_k1_per_rfc_9591() {
        // Defensive contract test — ensures we surface a clean
        // FrostError::Keygen rather than panic if a caller
        // tries the spec-invalid K=1 configuration.
        let err = FrostCanarySigner::trusted_dealer(1, 1).expect_err("K=1 must be rejected");
        assert!(
            matches!(err, FrostError::Keygen(_)),
            "expected Keygen error for K=1, got: {err:?}"
        );
    }

    #[test]
    fn frost_aggregate_refuses_partial_below_k_threshold() {
        // K=2 — we deal 3 shares, then try to sign with only 1.
        // FROST cannot produce a valid signature with fewer than
        // K participants. The rejection can surface at either
        // stage : Round2 (the SigningPackage commitments count
        // is below K, so each participant's round2::sign sees
        // "Incorrect number of commitments") or Aggregate (if
        // the implementation forwards through round2 first then
        // catches the threshold mismatch at the aggregator).
        // Both prove the threshold is enforced ; the exact
        // failure layer is an internal frost-ed25519 detail
        // we don't pin.
        let (shares, pubkey) =
            frost_keygen_trusted_dealer(2, 3).expect("trusted dealer (K=2, N=3) succeeds");

        // Take only 1 share (below K=2 threshold).
        let result = frost_sign_with_shares(&shares[..1], &pubkey, b"too few shares");
        let err = result.expect_err("aggregate with K-1 shares MUST fail");
        assert!(
            matches!(err, FrostError::Aggregate(_) | FrostError::Round2(_, _)),
            "expected Aggregate or Round2 failure on partial shares, got: {err:?}"
        );
    }

    #[test]
    fn frost_tampered_share_rejected() {
        // K=2 — deal shares, sign with 2, then swap one share's
        // key_package for one from a different keygen
        // (different secret). FROST aggregate's internal share
        // verification MUST catch this and reject.
        let (good_shares, good_pubkey) =
            frost_keygen_trusted_dealer(2, 3).expect("first trusted dealer (K=2, N=3)");
        let (bad_shares, _bad_pubkey) =
            frost_keygen_trusted_dealer(2, 3).expect("second trusted dealer (K=2, N=3)");

        // Build a hybrid : 1 legitimate share + 1 share from a
        // different keygen (its key_package signs against a
        // different verifying key).
        let tampered = vec![
            good_shares[0].clone(),
            FrostKeyShare {
                // Reuse the legitimate identifier so the BTreeMap
                // keys collide cleanly — the tampering is purely
                // on the key material, not the identifier slot.
                identifier: good_shares[1].identifier,
                key_package: bad_shares[0].key_package.clone(),
            },
        ];

        let result = frost_sign_with_shares(&tampered, &good_pubkey, b"tamper test");
        let err = result.expect_err("aggregate with one share from a different keygen MUST fail");
        // The exact failure point can be Round2 (the bad share
        // signs the wrong commitment) or Aggregate (the share
        // verification rejects the tampered participant
        // contribution). Both are acceptable evidence that
        // FROST caught the tamper.
        assert!(
            matches!(err, FrostError::Round2(_, _) | FrostError::Aggregate(_)),
            "expected Round2 or Aggregate failure on tampered share, got: {err:?}"
        );
    }

    #[test]
    fn frost_sig_verifiable_by_standard_ed25519_verifier() {
        // The crucial wire-format invariant : a FROST sig
        // MUST be accepted by `nexus_core_rs::crypto::verify`
        // (a plain ed25519-dalek::VerifyingKey::verify under
        // the hood) and by `verify_canary` end-to-end.
        let frost_signer = FrostCanarySigner::trusted_dealer(2, 3).expect("FROST keygen succeeds");
        let baseline_signer = Ed25519CanarySigner::new(KeyPair::generate());

        let msg = b"interop test 2026-04-18";

        // FROST sig — must verify exactly like a plain Ed25519 sig.
        let frost_sig = frost_signer.sign(msg);
        verify(&frost_signer.pubkey(), msg, &frost_sig)
            .expect("FROST sig accepted by plain Ed25519 verify");

        // Baseline sig — sanity check the verifier path isn't
        // accidentally permissive.
        let baseline_sig = baseline_signer.sign(msg);
        verify(&baseline_signer.pubkey(), msg, &baseline_sig)
            .expect("baseline Ed25519 sig accepted");

        // Cross-check : a FROST sig MUST NOT verify under a
        // different pubkey.
        let wrong_result = verify(&baseline_signer.pubkey(), msg, &frost_sig);
        assert!(
            wrong_result.is_err(),
            "FROST sig must not verify under unrelated Ed25519 pubkey"
        );
    }
}
