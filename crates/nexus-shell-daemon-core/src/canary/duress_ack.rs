// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase E.4 — duress ack channel.
//!
//! The monthly warrant canary ([`super::Canary`]) signals "no
//! gag order received this past 30-45 day window". That cadence
//! is too coarse for a maintainer who wants a finer anti-coercion
//! signal — by the time a missed canary triggers a verifier
//! alarm, weeks have passed.
//!
//! A **duress ack** is a daily-or-better short signed heartbeat
//! published on a separate gossip topic
//! ([`DURESS_ACK_TOPIC_SEED`]) that says "I, the maintainer,
//! voluntarily produced this ack on this date". A federated
//! verifier (cf. Phase E.3 `CanaryRegistry`) tracks the most
//! recent ack age **independently** from the monthly canary
//! age, so a sudden gap in acks is observable in days, not
//! weeks.
//!
//! ## Threat model
//!
//! - **T-canary-coercion** (operator forced to keep publishing) :
//!   a duress ack is signed by the maintainer's CLI under
//!   `sbfb canary ack` — there is no scheduler, no GHA cron, no
//!   way for an attacker who steals the running daemon to keep
//!   producing acks. A coerced maintainer who stops typing
//!   `sbfb canary ack` triggers the alarm in 1-2 days.
//! - **NOT** a coverage of T-canary-key-exfil — that requires
//!   FROST K-of-N (Phase E.2) and is orthogonal.
//!
//! ## Wire format
//!
//! [`DuressAckSigned`] is signed under
//! [`nexus_core_rs::canonical::DOMAIN_DURESS_ACK_V1`] (separate
//! domain tag from `DOMAIN_WARRANT_CANARY_V1` — a duress ack
//! signature cannot be replayed as a canary signature and vice
//! versa). The wire envelope is the same shape as
//! [`super::Canary`] : flatten signed body + `signature_hex`.

use nexus_core_rs::canonical::{canonical_bytes, DOMAIN_DURESS_ACK_V1};
use nexus_core_rs::crypto::{verify, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Date, OffsetDateTime};

use super::signer::CanarySigner;

/// Duress ack wire format version. Pre-launch policy : stays at
/// 1 until tag v1.0 (cf. CLAUDE.md "Pre-launch protocol policy").
pub const DURESS_ACK_VERSION: u16 = 1;

/// Hard cap on the optional `message` field. A duress ack
/// message is intended to be a single short reassurance token
/// (a code phrase, a daily news snippet) — capping at 256 bytes
/// keeps the gossip overhead trivial.
pub const MAX_DURESS_ACK_MESSAGE_LEN: usize = 256;

/// Seed for the duress ack gossip topic. Deliberately distinct
/// from `WARRANT_CANARY_TOPIC_SEED` so a verifier can subscribe
/// to one without the other — relays / mirrors can chose to
/// retain only the canary stream and skip the chattier ack
/// stream, or vice versa.
pub const DURESS_ACK_TOPIC_SEED: &[u8] = b"nexus-grid/canary-duress-ack/v1";

/// Errors the duress ack module surfaces to callers.
#[derive(Debug, Error)]
pub enum DuressAckError {
    #[error("canonical serialization failed: {0}")]
    Canonical(String),

    #[error("signature does not validate: {0}")]
    Signature(String),

    #[error("unsupported duress ack version {0}, expected {DURESS_ACK_VERSION}")]
    UnsupportedVersion(u16),

    #[error("malformed hex field: {0}")]
    BadHex(String),

    #[error("message exceeds {MAX_DURESS_ACK_MESSAGE_LEN}-byte cap ({0} bytes)")]
    MessageTooLong(usize),
}

/// Return the BLAKE3-derived 32-byte gossip topic id for the
/// duress ack flow.
pub fn duress_ack_topic_id() -> [u8; 32] {
    *blake3::hash(DURESS_ACK_TOPIC_SEED).as_bytes()
}

/// Signed portion of a duress ack — runs through `canonical_bytes`
/// under [`DOMAIN_DURESS_ACK_V1`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuressAckSigned {
    /// Must equal [`DURESS_ACK_VERSION`].
    #[serde(rename = "v")]
    pub version: u16,

    /// UTC date of the ack, `YYYY-MM-DD`.
    pub date: String,

    /// Optional maintainer message — a short string a verifier
    /// can use as a freshness witness (a daily news headline,
    /// a code phrase the maintainer agreed on with verifiers).
    /// May be empty.
    pub message: String,

    /// Lowercase hex of the maintainer's Ed25519 public key
    /// (64 chars). Same key as the corresponding warrant canary
    /// — verifiers can correlate ack streams with canary streams
    /// by pubkey.
    pub pubkey_hex: String,
}

/// A signed duress ack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DuressAck {
    #[serde(flatten)]
    pub signed: DuressAckSigned,

    /// Lowercase hex of the 64-byte Ed25519 signature.
    pub signature_hex: String,
}

/// Build a signed duress ack. `signer` is any [`CanarySigner`]
/// implementation — the same key used for the monthly canary
/// is the canonical choice, but the trait is generic for the
/// FROST K-of-N case.
pub fn build_duress_ack(
    date: Date,
    message: String,
    signer: &dyn CanarySigner,
) -> Result<DuressAck, DuressAckError> {
    if message.len() > MAX_DURESS_ACK_MESSAGE_LEN {
        return Err(DuressAckError::MessageTooLong(message.len()));
    }
    let signed = DuressAckSigned {
        version: DURESS_ACK_VERSION,
        date: format_date(date),
        message,
        pubkey_hex: hex::encode(signer.pubkey()),
    };
    let bytes = canonical_bytes(&signed, DOMAIN_DURESS_ACK_V1)
        .map_err(|e| DuressAckError::Canonical(e.to_string()))?;
    let signature = signer.sign(&bytes);
    Ok(DuressAck {
        signed,
        signature_hex: hex::encode(signature),
    })
}

/// Verify a [`DuressAck`] signature + version.
pub fn verify_duress_ack(ack: &DuressAck) -> Result<(), DuressAckError> {
    if ack.signed.version != DURESS_ACK_VERSION {
        return Err(DuressAckError::UnsupportedVersion(ack.signed.version));
    }
    let pubkey = decode_fixed_hex::<PUBLIC_KEY_LENGTH>(&ack.signed.pubkey_hex, "pubkey")?;
    let sig = decode_fixed_hex::<SIGNATURE_BYTES>(&ack.signature_hex, "signature")?;
    let bytes = canonical_bytes(&ack.signed, DOMAIN_DURESS_ACK_V1)
        .map_err(|e| DuressAckError::Canonical(e.to_string()))?;
    verify(&pubkey, &bytes, &sig).map_err(|e| DuressAckError::Signature(e.to_string()))
}

/// Today's UTC date — exposed so the binary can call
/// `build_duress_ack(today_utc(), ...)` without pulling `time`
/// directly.
pub fn today_utc() -> Date {
    OffsetDateTime::now_utc().date()
}

fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day(),)
}

fn decode_fixed_hex<const N: usize>(
    s: &str,
    field: &'static str,
) -> Result<[u8; N], DuressAckError> {
    let mut out = [0u8; N];
    hex::decode_to_slice(s, &mut out)
        .map_err(|e| DuressAckError::BadHex(format!("bad {field} hex: {e}")))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canary::signer::Ed25519CanarySigner;
    use crate::canary::warrant_canary_topic_id;
    use nexus_core_rs::KeyPair;

    fn a_date() -> Date {
        Date::from_calendar_date(2026, time::Month::April, 18).unwrap()
    }

    #[test]
    fn duress_ack_signed_roundtrip() {
        let signer = Ed25519CanarySigner::new(KeyPair::generate());
        let ack = build_duress_ack(a_date(), "daily code phrase".into(), &signer)
            .expect("build duress ack works");

        assert_eq!(ack.signed.version, DURESS_ACK_VERSION);
        assert_eq!(ack.signed.date, "2026-04-18");
        assert_eq!(ack.signed.message, "daily code phrase");
        assert_eq!(ack.signed.pubkey_hex.len(), 64);
        assert_eq!(ack.signature_hex.len(), 128);

        verify_duress_ack(&ack).expect("freshly built ack verifies");

        // Tamper resistance.
        let mut tampered = ack.clone();
        tampered.signed.message = "forged".into();
        let err = verify_duress_ack(&tampered).expect_err("tampered message must fail verify");
        assert!(matches!(err, DuressAckError::Signature(_)));
    }

    #[test]
    fn duress_ack_topic_id_deterministic_and_distinct_from_canary() {
        let a = duress_ack_topic_id();
        let b = duress_ack_topic_id();
        assert_eq!(a, b, "topic id is a pure function of seed");
        assert_eq!(a.len(), 32);

        // Critical : the two topics MUST be distinct so a
        // duress ack signature cannot be replayed as a canary
        // signature and vice versa (independent gossip
        // partitioning + independent domain tags).
        let canary_topic = warrant_canary_topic_id();
        assert_ne!(
            a, canary_topic,
            "duress ack topic must be distinct from warrant canary topic"
        );

        assert_eq!(DURESS_ACK_TOPIC_SEED, b"nexus-grid/canary-duress-ack/v1");
    }

    #[test]
    fn duress_ack_rejects_oversize_message() {
        let signer = Ed25519CanarySigner::new(KeyPair::generate());
        let oversize = "x".repeat(MAX_DURESS_ACK_MESSAGE_LEN + 1);
        let err =
            build_duress_ack(a_date(), oversize, &signer).expect_err("message over cap must fail");
        assert!(matches!(err, DuressAckError::MessageTooLong(_)));
    }
}
