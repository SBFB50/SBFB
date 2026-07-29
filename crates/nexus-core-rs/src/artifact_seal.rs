// SPDX-License-Identifier: AGPL-3.0-or-later
//! Per-artefact sealing for the workflow artefact ACL (Sprint 83 Phase M).
//!
//! ## Why this lives here and not in the engine
//!
//! `sbfb-workflow-engine` defines the [`ArtifactSealer`] seam but cannot
//! implement it: the crate is keyless by charter and its dependency set holds no
//! source of entropy — no `rand`, no `getrandom`, no AEAD crate. Every seal needs
//! a FRESH nonce, so sealing is host work by construction. This module is the
//! host primitive; the daemon wires it to the engine's trait.
//!
//! [`ArtifactSealer`]: https://docs.rs/sbfb-workflow-engine
//!
//! ## The key model: one key per artefact, and why
//!
//! Each artefact gets its OWN 256-bit key, minted at seal time from the OS CSPRNG
//! and handed back to the caller as the read capability. Nothing here stores it.
//!
//! That is what makes deletion mean something. `blobs.rs` is explicit that
//! dropping the last tag only makes a blob GC-eligible and that no GC is
//! scheduled, so bytes survive a delete — on disk, in the blob-serve LRU cache,
//! and in any remote copy already fetched. Destroying a per-artefact key is
//! therefore the only step that changes what an adversary can read, and it can
//! only work if the key is not shared with another artefact.
//!
//! ## Bounds — no custody, therefore no at-rest bound
//!
//! Nothing here holds the key: it is minted and handed straight back to the
//! caller, and no artefact custody exists anywhere in this crate — neither
//! `vault` nor `keystore` exposes an artefact API. So NO at-rest bound covers an
//! artefact key today, because no artefact key is at rest.
//!
//! The `vault` bound — Argon2id(PIN), with the OS-keyring second layer INERT
//! (`keyring` is declared without a backend feature, so the builder is a mock;
//! carry `KEYSTORE-LAYER2-INERT`) — is what that module guarantees FOR ITS OWN
//! entries. It is cited here as the PATTERN a future custody should follow, not
//! as a protection this module provides: this module claims no layer at all,
//! single or double. A caller that keeps a capability in a process-lifetime
//! cache defeats the shredding this design buys.
//!
//! The AAD binds the ciphertext to the artefact's MODE, so bytes sealed as
//! `local` cannot be re-labelled `published` by an attacker holding them: the
//! tag stops verifying. It does NOT bind the digest — the digest is computed
//! OVER this output, so including it would be circular.

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroizing;

/// Length of the AES-GCM nonce, in bytes.
///
/// Declared locally rather than imported from [`crate::keystore::NONCE_LEN`],
/// and the decoupling is DELIBERATE: that constant belongs to the keystore's
/// at-rest format, this one to the artefact's. They share no header, no domain
/// and no key, so importing it would make a change to the keystore format
/// silently re-cut every stored artefact. Two independent `12`s for two
/// independent formats — stated here because the silence, not the value, was
/// the defect.
const NONCE_LEN: usize = 12;

/// Length of the per-artefact key, in bytes.
pub const ARTIFACT_KEY_LEN: usize = 32;

/// Domain separator for the additional authenticated data. Keeps an artefact
/// ciphertext from ever verifying under another use of AES-GCM in this tree.
///
/// NOT a `DOMAIN_*_V1` signing family, and deliberately not named like one — the
/// same note `hitl_approval.rs` carries for the same reason. Nothing signs with
/// it: it is an AEAD separator, so it is outside the DOMAIN census (frozen at
/// 29) and renaming it to `DOMAIN_ARTIFACT_SEAL_V1` would enrol it by name alone
/// and contradict this phase's "0 new signing family" claim. Its exact bytes are
/// pinned by `aad_matches_its_golden_vector`.
const ARTIFACT_SEAL_AAD_DOMAIN: &[u8] = b"sbfb.artifact.seal.v1";

/// Field separator inside the AAD preimage, named for the reason
/// `vault.rs:186` and `hitl_approval.rs:33` name theirs: a bare `0x1f` in the
/// middle of a builder is a magic number in a byte string the threat model
/// publishes as a contract (README §6.9).
const SEP: u8 = 0x1f;

/// Sealing or opening an artefact failed.
///
/// Deliberately coarse: an error that distinguished "bad key" from "bad tag"
/// would be an oracle for an attacker probing capabilities. No variant carries
/// plaintext or key material.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ArtifactSealError {
    /// The key is not [`ARTIFACT_KEY_LEN`] bytes.
    #[error("artefact key must be {ARTIFACT_KEY_LEN} bytes")]
    KeyLength,
    /// The sealed blob is shorter than a nonce, or its tag does not verify.
    /// Fail-closed and indistinguishable by design.
    #[error("sealed artefact did not open")]
    DidNotOpen,
    /// Encryption itself failed — the plaintext exceeds AES-GCM's `P_MAX`
    /// (2^36 - 31 bytes), essentially the only way this arm is reachable.
    ///
    /// Distinct from [`ArtifactSealError::DidNotOpen`] because the two are
    /// opposite events: this one means nothing was ever stored, the other means
    /// stored bytes would not come back. Both reach the operator verbatim
    /// through `ArtifactError::SealerRefused`, and labelling a SEALING failure
    /// "sealed artefact did not open" would send them looking for a corrupt
    /// blob that does not exist. No oracle concern here, unlike the opening
    /// path: the caller supplied the plaintext.
    #[error("artefact could not be sealed")]
    SealFailed,
}

/// What `seal` produced: the bytes to store, and the key that reads them back.
///
/// The key is `Zeroizing`, so dropping this value without extracting the key
/// wipes it rather than leaving it in freed heap.
pub struct SealedArtifact {
    /// `nonce || ciphertext+tag` — the bytes the blob store receives.
    ///
    /// That framing is an AT-REST CONTRACT, not an implementation detail: bytes
    /// already in a blob store were framed by whatever version wrote them, so
    /// swapping the two halves would make every existing artefact unopenable.
    /// It is pinned by `each_seal_mints_a_distinct_key_and_nonce`, which
    /// re-derives the plaintext from the primitives instead of calling
    /// [`open_artifact`] — a round trip through this module's own helpers is
    /// blind to any change made symmetrically, the same reason the AAD needs its
    /// golden vector.
    pub stored: Vec<u8>,
    /// The per-artefact read key. Zeroed on drop.
    pub key: Zeroizing<Vec<u8>>,
}

impl SealedArtifact {
    /// Pin the zeroising WRAPPER, not merely the behaviour.
    ///
    /// Swapping `Zeroizing<Vec<u8>>` for a bare `Vec<u8>` WOULD compile and leave
    /// every other test in this module green — `&sealed.key`, `.len()` and the
    /// redacting `Debug` behave identically through both, so nothing observable
    /// at runtime changes while the doc-comment above goes on promising the key
    /// is wiped.
    ///
    /// What stops it is the type ANNOTATION at the call site in
    /// `key_is_held_in_a_zeroising_wrapper`, not this accessor: `.len()` exists
    /// on both types, so calling the accessor and reading a length proves
    /// nothing. Measured on the engine's twin, where the assertion was written
    /// without the annotation and the coherent mutation stayed green.
    #[cfg(test)]
    pub(crate) fn key_is_zeroizing(&self) -> &Zeroizing<Vec<u8>> {
        &self.key
    }
}

impl std::fmt::Debug for SealedArtifact {
    /// Redacting: prints sizes, never bytes. The stored half is ciphertext and
    /// would be harmless, but a `{:?}` that dumped one half trains readers to
    /// expect the other.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SealedArtifact {{ stored: <{} bytes>, key: <redacted {} bytes> }}",
            self.stored.len(),
            self.key.len()
        )
    }
}

/// Seal `plaintext` under a freshly minted per-artefact key.
///
/// `mode_label` is the artefact mode as a stable string; it
/// is bound into the AAD, so opening with a different label fails.
pub fn seal_artifact(
    mode_label: &str,
    plaintext: &[u8],
) -> Result<SealedArtifact, ArtifactSealError> {
    let mut key = Zeroizing::new(vec![0u8; ARTIFACT_KEY_LEN]);
    OsRng.fill_bytes(&mut key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| ArtifactSealError::KeyLength)?;
    let aad = seal_aad(mode_label);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad: &aad,
            },
        )
        .map_err(|_| ArtifactSealError::SealFailed)?;

    let mut stored = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    stored.extend_from_slice(&nonce_bytes);
    stored.extend_from_slice(&ciphertext);

    Ok(SealedArtifact { stored, key })
}

/// Open bytes produced by [`seal_artifact`].
pub fn open_artifact(
    mode_label: &str,
    key: &[u8],
    stored: &[u8],
) -> Result<Zeroizing<Vec<u8>>, ArtifactSealError> {
    if key.len() != ARTIFACT_KEY_LEN {
        return Err(ArtifactSealError::KeyLength);
    }
    if stored.len() < NONCE_LEN {
        return Err(ArtifactSealError::DidNotOpen);
    }
    let (nonce_bytes, ciphertext) = stored.split_at(NONCE_LEN);

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| ArtifactSealError::KeyLength)?;
    let aad = seal_aad(mode_label);
    cipher
        .decrypt(
            Nonce::from_slice(nonce_bytes),
            Payload {
                msg: ciphertext,
                aad: &aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| ArtifactSealError::DidNotOpen)
}

fn seal_aad(mode_label: &str) -> Vec<u8> {
    let mut aad = Vec::with_capacity(ARTIFACT_SEAL_AAD_DOMAIN.len() + 1 + mode_label.len());
    aad.extend_from_slice(ARTIFACT_SEAL_AAD_DOMAIN);
    aad.push(SEP);
    aad.extend_from_slice(mode_label.as_bytes());
    aad
}

#[cfg(test)]
mod tests {
    use super::*;

    const CANARY: &str = "SBFB-SEAL-CANARY-9d41b7-do-not-leak";

    #[test]
    fn round_trip_recovers_plaintext() {
        let sealed = seal_artifact("local", CANARY.as_bytes()).expect("seal");
        let opened = open_artifact("local", &sealed.key, &sealed.stored).expect("open");
        assert_eq!(&opened[..], CANARY.as_bytes());
    }

    #[test]
    fn stored_bytes_never_contain_plaintext() {
        let sealed = seal_artifact("local", CANARY.as_bytes()).expect("seal");
        let found = sealed
            .stored
            .windows(CANARY.len())
            .any(|w| w == CANARY.as_bytes());
        assert!(!found, "sealed output leaked its plaintext");

        // Positive control: the same search DOES find the canary in the
        // plaintext itself, so a green assertion above is not a broken search.
        let plaintext = CANARY.as_bytes();
        assert!(
            plaintext
                .windows(CANARY.len())
                .any(|w| w == CANARY.as_bytes())
        );
    }

    /// Both halves of the freshness claim, checked SEPARATELY.
    ///
    /// The earlier version of this test compared only the whole stored blobs and
    /// the keys. That is not discriminating: with a CONSTANT nonce the blobs
    /// still differ, because the keys differ — so the assertion passed while the
    /// nonce was fixed at zero (measured). A nonce claim has to look at the
    /// nonce, which is the first `NONCE_LEN` bytes of the stored form.
    #[test]
    fn each_seal_mints_a_distinct_key_and_nonce() {
        let first = seal_artifact("local", b"same").expect("seal");
        let second = seal_artifact("local", b"same").expect("seal");

        assert_ne!(first.key, second.key, "keys must not be shared");

        let first_nonce = &first.stored[..NONCE_LEN];
        let second_nonce = &second.stored[..NONCE_LEN];
        assert_ne!(
            first_nonce, second_nonce,
            "the nonce must be fresh per seal, not a constant"
        );
        assert_ne!(
            first_nonce,
            [0u8; NONCE_LEN].as_slice(),
            "an all-zero nonce is the constant-nonce failure, caught explicitly"
        );

        assert_ne!(
            first.stored, second.stored,
            "identical plaintext must not produce identical ciphertext"
        );

        // The FRAMING, re-derived independently rather than through
        // `open_artifact`. `seal_artifact` and `open_artifact` build the stored
        // form with the same two lines, so a symmetric change — say
        // `ciphertext || nonce` with a matching `split_at(len - NONCE_LEN)` —
        // round-trips perfectly and was MEASURED to leave every other test green
        // across all three crates, this test included: with the halves swapped, the nonce
        // assertions above are asserting the freshness of ciphertext bytes. Only
        // decrypting with the framing spelled out here can go red.
        let cipher = Aes256Gcm::new_from_slice(&first.key).expect("key length");
        let recovered = cipher
            .decrypt(
                Nonce::from_slice(&first.stored[..NONCE_LEN]),
                Payload {
                    msg: &first.stored[NONCE_LEN..],
                    aad: &seal_aad("local"),
                },
            )
            .expect("stored is nonce || ciphertext+tag, in that order");
        assert_eq!(&recovered[..], b"same");
    }

    /// The key is held in a zeroising wrapper. Pinned as a COMPILED fact, since
    /// no runtime assertion can observe freed heap.
    #[test]
    fn key_is_held_in_a_zeroising_wrapper() {
        let sealed = seal_artifact("local", b"x").expect("seal");
        let wrapper: &Zeroizing<Vec<u8>> = sealed.key_is_zeroizing();
        assert_eq!(wrapper.len(), ARTIFACT_KEY_LEN);
    }

    #[test]
    fn wrong_key_does_not_open() {
        let sealed = seal_artifact("local", CANARY.as_bytes()).expect("seal");
        let wrong = vec![0u8; ARTIFACT_KEY_LEN];
        assert_eq!(
            open_artifact("local", &wrong, &sealed.stored),
            Err(ArtifactSealError::DidNotOpen)
        );
    }

    /// KNOWN-ANSWER VECTOR for the AAD: domain, separator and label, byte for
    /// byte.
    ///
    /// Without it the AAD is pinned by NOTHING: `seal` and `open` derive it from
    /// the same helper, so the round trip is blind by construction — measured,
    /// reducing `seal_aad` to the bare label leaves every other test green across
    /// both crates. The threat model publishes this exact byte string as the at-rest
    /// contract (§19.5), so a silent change there is a format break nobody would
    /// notice. Same reasoning, same crate, as `vault.rs`'s
    /// `vault_kek_is_domain_separated_from_keystore`: any self-consistent
    /// derivation passes, therefore the answer must be written down.
    #[test]
    fn aad_matches_its_golden_vector() {
        assert_eq!(
            seal_aad("local"),
            b"sbfb.artifact.seal.v1\x1flocal".to_vec()
        );
        assert_eq!(
            seal_aad("published"),
            b"sbfb.artifact.seal.v1\x1fpublished".to_vec()
        );
        // The separator is a real byte in the middle, not decoration.
        assert!(seal_aad("published").contains(&0x1f));
        assert_eq!(ARTIFACT_SEAL_AAD_DOMAIN, b"sbfb.artifact.seal.v1");
    }

    #[test]
    fn mode_relabelling_does_not_open() {
        let sealed = seal_artifact("local", CANARY.as_bytes()).expect("seal");
        assert_eq!(
            open_artifact("published", &sealed.key, &sealed.stored),
            Err(ArtifactSealError::DidNotOpen),
            "the AAD must bind the artefact mode"
        );
    }

    /// A flipped bit anywhere in the stored form is refused — in the NONCE, in
    /// the ciphertext BODY, and in the trailing tag.
    ///
    /// Three named positions rather than one, because the earlier version
    /// flipped only the last byte: that is the tag, the one region whose
    /// rejection is the AEAD's most obvious property. A nonce or body flip
    /// exercises a different path through `decrypt` and neither was covered.
    #[test]
    fn tampered_ciphertext_does_not_open() {
        let sealed = seal_artifact("local", CANARY.as_bytes()).expect("seal");
        let body = NONCE_LEN + CANARY.len() / 2;
        let tag = sealed.stored.len() - 1;
        for (region, index) in [("nonce", 0), ("body", body), ("tag", tag)] {
            let mut tampered = sealed.stored.clone();
            tampered[index] ^= 0x01;
            assert_eq!(
                open_artifact("local", &sealed.key, &tampered),
                Err(ArtifactSealError::DidNotOpen),
                "a flipped bit in the {region} must fail closed"
            );
        }
    }

    /// Every length up to and past a complete frame fails closed rather than
    /// panicking.
    ///
    /// The bound runs PAST `NONCE_LEN`, on purpose: stopping there exercised
    /// only this module's own `stored.len() < NONCE_LEN` guard and never the
    /// range where a nonce is present but the tag is short or absent. Those
    /// lengths exit through the DEPENDENCY's check (`aead`), which a version
    /// bump could move — so they must be pinned here, not assumed.
    /// `NONCE_LEN + 17` covers the empty ciphertext with a full 16-byte tag,
    /// plus one.
    #[test]
    fn truncated_input_fails_closed_without_panicking() {
        for len in 0..NONCE_LEN + 17 {
            assert_eq!(
                open_artifact("local", &[0u8; ARTIFACT_KEY_LEN], &vec![0u8; len]),
                Err(ArtifactSealError::DidNotOpen),
                "a {len}-byte input must fail closed"
            );
        }
    }

    #[test]
    fn wrong_key_length_is_refused() {
        let sealed = seal_artifact("local", b"x").expect("seal");
        assert_eq!(
            open_artifact("local", &[0u8; 16], &sealed.stored),
            Err(ArtifactSealError::KeyLength)
        );
    }

    /// The `Debug` rendering is pinned by EQUALITY, not by substring.
    ///
    /// A `contains("<redacted 32 bytes>")` plus a hex-only negative filter stays
    /// green while `Debug` also dumps the 32 key bytes as a DECIMAL array after
    /// the marker — measured, the whole module stayed green against exactly
    /// that. (Test counts are deliberately not quoted anywhere in this module:
    /// the next test added re-stales them, which is the class rather than the
    /// instance.) It is the same
    /// defect this repository already closed at P1 severity on the secret side,
    /// and the threat model says so in the sentence this very commit edits:
    /// "epingle en EGALITE EXACTE … aveugle a un `{:?}` du slice, qui deverse
    /// tout en decimal". The twin in `sbfb_workflow_engine::artifact_acl` used
    /// equality; this side did not.
    #[test]
    fn debug_redacts_key_material() {
        let sealed = seal_artifact("local", b"x").expect("seal");
        let rendered = format!("{sealed:?}");
        assert_eq!(
            rendered,
            format!(
                "SealedArtifact {{ stored: <{} bytes>, key: <redacted 32 bytes> }}",
                sealed.stored.len()
            )
        );
    }
}
