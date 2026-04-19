// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 18 Phase E2 — warrant canary.
//!
//! A warrant canary is a monthly signed declaration that the
//! project maintainer has **not** been compelled by any
//! government order to modify / backdoor / disclose project code
//! or user data under a gag order. The canary's continued
//! publication is evidence of maintainer independence; a missed
//! publication (> 45 days stale) is a deliberate dead-man-switch
//! signal.
//!
//! ## Wire format
//!
//! A [`Canary`] serializes to JSON and is broadcast on the
//! gossip topic derived from [`WARRANT_CANARY_TOPIC_SEED`]. The
//! same payload is mirrored at `CANARY.txt` in the git repo
//! root via the [`format_canary_txt`] helper so anyone who can
//! clone the repo can verify the signature without talking to
//! the SBFB network.
//!
//! The Ed25519 signature covers the [`CanarySigned`] struct
//! (i.e. the canary minus the signature field) run through
//! [`nexus_core_rs::canonical_bytes`] with
//! [`DOMAIN_WARRANT_CANARY_V1`], so a valid canary signature
//! cannot be replayed as a task / result / claim / curator-list
//! / provenance signature.
//!
//! ## Signing key
//!
//! The maintainer's canary key is a persistent Ed25519 keypair
//! loaded from `<sbfb_home>/canary-key.key` via
//! [`nexus_core_rs::KeyPair::load_or_generate`]. It is
//! intentionally distinct from the daemon's ephemeral
//! `create_node` identity (which rotates per boot) — a warrant
//! canary needs a stable maintainer key that outlives any single
//! daemon process so verifiers can trust a single long-lived
//! pubkey across months of canary publications.
//!
//! ## Frequency
//!
//! The canary is rebuilt monthly; the [`CanarySigned::next_update`]
//! field is always `date + 45 days` so a 30-day cadence has a
//! 15-day grace period before verifiers should alarm.

use async_trait::async_trait;
use nexus_core_rs::canonical::{canonical_bytes, DOMAIN_WARRANT_CANARY_V1};
use nexus_core_rs::crypto::{verify, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Date, Duration, OffsetDateTime};

pub mod attestation;
pub mod duress_ack;
pub mod frost;
pub mod signer;

pub use attestation::{Attestation, AttestationProvider, NoopAttestation};
pub use duress_ack::{
    build_duress_ack, duress_ack_topic_id, verify_duress_ack, DuressAck, DuressAckSigned,
    DURESS_ACK_TOPIC_SEED,
};
pub use frost::{
    frost_keygen_trusted_dealer, FrostCanarySigner, FrostError, FrostKeyShare, FrostPubkey,
};
pub use signer::{CanarySigner, Ed25519CanarySigner};

/// Canary wire format version. Bumped on a signature-invalidating
/// field change; the `v1` suffix on [`DOMAIN_WARRANT_CANARY_V1`]
/// tracks this independently.
pub const CANARY_VERSION: u16 = 1;

/// Grace period between two canary publications. A missed
/// publication past this window is the dead-man-switch alarm.
pub const CANARY_VALIDITY_DAYS: i64 = 45;

/// Hard cap on the `headline` field. A warrant canary headline
/// is intended to be a short news-article title (typically one
/// sentence). Capping at 512 bytes defuses a trivial
/// gossip-broadcast DoS where a malformed caller passes a
/// multi-megabyte string into `build_canary` and every subscriber
/// re-propagates it.
pub const MAX_HEADLINE_LEN: usize = 512;

/// Seed string used to derive the warrant canary gossip topic id.
/// The topic id is `blake3(WARRANT_CANARY_TOPIC_SEED)[..32]`.
pub const WARRANT_CANARY_TOPIC_SEED: &[u8] = b"nexus-grid/warrant-canary/v1";

/// Return the BLAKE3-derived 32-byte gossip topic id for the
/// warrant canary flow. Mirrors the shape of
/// [`crate::iroh_runtime::curator_topic_id`] so the two signing
/// surfaces stay symmetric on the wire.
pub fn warrant_canary_topic_id() -> [u8; 32] {
    *blake3::hash(WARRANT_CANARY_TOPIC_SEED).as_bytes()
}

/// Errors the canary module surfaces to callers.
#[derive(Debug, Error)]
pub enum CanaryError {
    /// Serialization of a [`CanarySigned`] via
    /// [`nexus_core_rs::canonical_bytes`] failed. Practically
    /// never fires — the struct has no `HashMap` or
    /// non-string-keyed fields.
    #[error("canonical serialization failed: {0}")]
    Canonical(String),

    /// The Ed25519 signature did not validate against the
    /// embedded pubkey. Thrown by
    /// [`verify_canary`] when the message bytes or the pubkey
    /// have been tampered with.
    #[error("signature does not validate: {0}")]
    Signature(String),

    /// The declared version field on the canary is not
    /// [`CANARY_VERSION`].
    #[error("unsupported canary version {0}, expected {CANARY_VERSION}")]
    UnsupportedVersion(u16),

    /// The signature or pubkey hex could not be decoded to the
    /// fixed-size byte arrays.
    #[error("malformed hex field: {0}")]
    BadHex(String),

    /// The `headline` field exceeds [`MAX_HEADLINE_LEN`] bytes.
    /// See the const doc for the rationale.
    #[error("headline exceeds {MAX_HEADLINE_LEN}-byte cap ({0} bytes)")]
    HeadlineTooLong(usize),

    /// The gossip broadcast sink returned an error. The caller
    /// chose the concrete error message.
    #[error("gossip broadcast failed: {0}")]
    Broadcast(String),
}

/// The signed portion of a [`Canary`]. All fields of this struct
/// go into [`canonical_bytes`] under
/// [`DOMAIN_WARRANT_CANARY_V1`]; the signature covers exactly
/// these bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanarySigned {
    /// Must equal [`CANARY_VERSION`].
    #[serde(rename = "v")]
    pub version: u16,

    /// UTC date of this canary, `YYYY-MM-DD`.
    pub date: String,

    /// Maintainer-chosen headline proving the canary was minted
    /// on-or-after `date` (typically a major news headline of
    /// the day). Must be supplied manually — no automated scrape
    /// path, to prevent a compelled maintainer from publishing a
    /// stale but syntactically-valid canary via cron.
    pub headline: String,

    /// UTC date past which a missing canary update triggers an
    /// alarm. Set to `date + CANARY_VALIDITY_DAYS` at build time.
    pub next_update: String,

    /// Lowercase hex of the maintainer's Ed25519 public key
    /// (64 chars). Carrying the pubkey in the payload lets any
    /// offline verifier check the signature without an external
    /// key-directory lookup.
    pub pubkey_hex: String,
}

/// A signed warrant canary. See the module-level docs for the
/// full semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Canary {
    /// The signed portion of the canary. Sign / verify run over
    /// [`canonical_bytes`] of this nested struct.
    #[serde(flatten)]
    pub signed: CanarySigned,

    /// Lowercase hex of the 64-byte Ed25519 signature.
    pub signature_hex: String,
}

/// Build a signed canary using `signer` as the maintainer key.
///
/// `signer` is any [`CanarySigner`] implementation:
/// - [`Ed25519CanarySigner`] — single-key (default, monthly cron-
///   free human-driven publish flow).
/// - [`FrostCanarySigner`] — threshold K-of-N over Ed25519 via
///   FROST RFC 9591 (opt-in, K=2/N=3 cross-juridiction maintainer
///   federation).
///
/// Both produce a 64-byte Ed25519 signature byte-identical on the
/// wire — the verifier path is the same `verify(pubkey, msg, sig)`
/// call against [`DOMAIN_WARRANT_CANARY_V1`] canonical bytes.
///
/// `date` is the canary date (typically today's UTC date). The
/// `next_update` field is computed as `date + CANARY_VALIDITY_DAYS`.
pub fn build_canary(
    date: Date,
    headline: String,
    signer: &dyn CanarySigner,
) -> Result<Canary, CanaryError> {
    if headline.len() > MAX_HEADLINE_LEN {
        return Err(CanaryError::HeadlineTooLong(headline.len()));
    }
    let next_update = date.saturating_add(Duration::days(CANARY_VALIDITY_DAYS));
    let signed = CanarySigned {
        version: CANARY_VERSION,
        date: format_date(date),
        headline,
        next_update: format_date(next_update),
        pubkey_hex: hex::encode(signer.pubkey()),
    };
    let bytes = canonical_bytes(&signed, DOMAIN_WARRANT_CANARY_V1)
        .map_err(|e| CanaryError::Canonical(e.to_string()))?;
    let signature = signer.sign(&bytes);
    Ok(Canary {
        signed,
        signature_hex: hex::encode(signature),
    })
}

/// Return today's UTC date. Exposed so the binary can call
/// `build_canary(today_utc(), ...)` without pulling in `time`.
pub fn today_utc() -> Date {
    OffsetDateTime::now_utc().date()
}

/// Verify a [`Canary`] signature + version. Returns `Ok(())` if
/// the canary is well-formed and the signature matches.
pub fn verify_canary(canary: &Canary) -> Result<(), CanaryError> {
    if canary.signed.version != CANARY_VERSION {
        return Err(CanaryError::UnsupportedVersion(canary.signed.version));
    }

    let pubkey = decode_fixed_hex::<PUBLIC_KEY_LENGTH>(&canary.signed.pubkey_hex, "pubkey")?;
    let sig = decode_fixed_hex::<SIGNATURE_BYTES>(&canary.signature_hex, "signature")?;

    let bytes = canonical_bytes(&canary.signed, DOMAIN_WARRANT_CANARY_V1)
        .map_err(|e| CanaryError::Canonical(e.to_string()))?;
    verify(&pubkey, &bytes, &sig).map_err(|e| CanaryError::Signature(e.to_string()))
}

/// Serialize a canary to the JCS canonical (RFC 8785) JSON bytes
/// broadcast on the warrant canary gossip topic.
///
/// Sprint 21 Phase E (T-NN tech debt resolved) : migrated from
/// `serde_json::to_vec` to `serde_jcs::to_vec` so the wire bytes
/// are byte-identical across Rust and Python observers. Aligns
/// the canary transport on the project-wide JCS pattern adopted
/// Sprint 4 Day 0 (commit `1c1fcfb`) for Task / Result / Claim /
/// Curator. The signing path already uses JCS via
/// [`nexus_core_rs::canonical_bytes`] so this migration does NOT
/// break any existing signature — `verify_canary` continues to
/// hash `canonical_bytes(&canary.signed, DOMAIN_WARRANT_CANARY_V1)`,
/// not the wire bytes.
///
/// Pre-launch protocol policy : no canary has been published in
/// production yet, so even an observer cache built from the old
/// `serde_json::to_vec` bytes would be wiped at the first prod
/// publish. Zero migration risk.
pub fn canary_wire_bytes(canary: &Canary) -> Result<Vec<u8>, CanaryError> {
    serde_jcs::to_vec(canary).map_err(|e| CanaryError::Canonical(e.to_string()))
}

/// Format a canary as the multi-line ASCII declaration stored at
/// `CANARY.txt` in the repo root. The format is the spec from
/// Sprint 18 kickoff §D5.
pub fn format_canary_txt(canary: &Canary) -> String {
    let s = &canary.signed;
    format!(
        "SBFB Warrant Canary\n\
         Date: {date} (UTC)\n\
         Headline: {headline}\n\
         \n\
         Declaration:\n  \
         As of the date above, the SBFB project maintainer(s) have NOT:\n  \
         - received any National Security Letter, secret subpoena,\n    \
         or gag order from any government agency\n  \
         - been compelled to modify or backdoor any code or cryptographic\n    \
         key material used by the project\n  \
         - been compelled to provide user data to any third party\n\
         \n  \
         This canary is signed and published monthly. If the canary is\n  \
         not updated for >{validity} days, assume it has been compromised OR\n  \
         the project has been compelled and cannot disclose.\n\
         \n\
         Next scheduled update: {next_update}\n\
         \n\
         Signed:\n  \
         Ed25519 signature over the canonical bytes (domain {domain})\n  \
         sig: {signature}\n  \
         pub: {pubkey}\n",
        date = s.date,
        headline = s.headline,
        validity = CANARY_VALIDITY_DAYS,
        next_update = s.next_update,
        domain = std::str::from_utf8(DOMAIN_WARRANT_CANARY_V1).unwrap_or("nexus-warrant-canary-v1"),
        signature = canary.signature_hex,
        pubkey = s.pubkey_hex,
    )
}

/// Inverse of [`format_canary_txt`] — extract the signed fields
/// from a human-readable `CANARY.txt` so the CLI's `verify`
/// subcommand (and `scripts/verify-canary.sh`) can re-validate a
/// canary on disk without a side-car JSON file.
///
/// The parser is line-oriented and tolerates extra whitespace /
/// trailing blank lines so a human edit that reflows the
/// declaration paragraph does not accidentally break verification
/// — only the six field lines (`Date:`, `Headline:`,
/// `Next scheduled update:`, `sig:`, `pub:`) are semantically
/// meaningful.
pub fn parse_canary_txt(text: &str) -> Result<Canary, CanaryError> {
    let date = extract_field(text, "Date:", Some(" (UTC)"))?;
    let headline = extract_field(text, "Headline:", None)?;
    let next_update = extract_field(text, "Next scheduled update:", None)?;
    let signature_hex = extract_field(text, "sig:", None)?;
    let pubkey_hex = extract_field(text, "pub:", None)?;

    Ok(Canary {
        signed: CanarySigned {
            version: CANARY_VERSION,
            date,
            headline,
            next_update,
            pubkey_hex,
        },
        signature_hex,
    })
}

fn extract_field(text: &str, key: &str, trailing: Option<&str>) -> Result<String, CanaryError> {
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix(key) {
            let mut value = rest.trim().to_string();
            if let Some(suffix) = trailing {
                if let Some(short) = value.strip_suffix(suffix) {
                    value = short.trim_end().to_string();
                }
            }
            if value.is_empty() {
                return Err(CanaryError::BadHex(format!(
                    "empty value for field {key:?}"
                )));
            }
            return Ok(value);
        }
    }
    Err(CanaryError::BadHex(format!(
        "missing field {key:?} in canary text"
    )))
}

/// Minimal abstraction over a gossip broadcast sink. The binary
/// implements this on a `TopicHandle` / `TopicSender`; tests
/// implement it on a `Vec<Vec<u8>>`-backed mock so
/// [`publish_canary`] stays unit-testable without a live iroh
/// node.
#[async_trait]
pub trait CanaryBroadcaster: Send {
    /// Broadcast `bytes` to every peer on the warrant canary
    /// topic. Implementations return a descriptive error string
    /// on failure; the wrapper surfaces it as
    /// [`CanaryError::Broadcast`].
    async fn broadcast(&mut self, bytes: Vec<u8>) -> Result<(), String>;
}

/// Broadcast a canary as a gossip message. Pure glue: the bytes
/// layout is [`canary_wire_bytes`], the transport is the caller's
/// `broadcaster`.
pub async fn publish_canary(
    canary: &Canary,
    broadcaster: &mut (dyn CanaryBroadcaster + Send),
) -> Result<(), CanaryError> {
    let wire = canary_wire_bytes(canary)?;
    broadcaster
        .broadcast(wire)
        .await
        .map_err(CanaryError::Broadcast)
}

// =================================================================
// Helpers
// =================================================================

fn format_date(d: Date) -> String {
    // ISO-8601 extended, zero-padded, no timezone suffix — the
    // struct-level `(UTC)` suffix on the txt mirror already
    // carries that.
    format!("{:04}-{:02}-{:02}", d.year(), u8::from(d.month()), d.day(),)
}

fn decode_fixed_hex<const N: usize>(s: &str, field: &'static str) -> Result<[u8; N], CanaryError> {
    let mut out = [0u8; N];
    hex::decode_to_slice(s, &mut out)
        .map_err(|e| CanaryError::BadHex(format!("bad {field} hex: {e}")))?;
    Ok(out)
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::KeyPair;

    fn a_date() -> Date {
        Date::from_calendar_date(2026, time::Month::April, 15).unwrap()
    }

    fn ed25519_signer() -> Ed25519CanarySigner {
        Ed25519CanarySigner::new(KeyPair::generate())
    }

    #[test]
    fn build_canary_includes_date_headline_sig() {
        let signer = ed25519_signer();
        let c = build_canary(a_date(), "NYT 2026-04-15: test".into(), &signer)
            .expect("build_canary works on well-formed input");

        assert_eq!(c.signed.version, CANARY_VERSION);
        assert_eq!(c.signed.date, "2026-04-15");
        assert_eq!(c.signed.headline, "NYT 2026-04-15: test");
        assert_eq!(c.signed.next_update, "2026-05-30");
        assert_eq!(c.signed.pubkey_hex.len(), 64);
        assert_eq!(c.signature_hex.len(), 128);
    }

    #[test]
    fn verify_canary_accepts_valid_signature() {
        let signer = ed25519_signer();
        let c = build_canary(a_date(), "headline".into(), &signer).unwrap();
        verify_canary(&c).expect("a freshly-built canary must verify");
    }

    #[test]
    fn verify_canary_rejects_tampered_message() {
        let signer = ed25519_signer();
        let mut c = build_canary(a_date(), "real".into(), &signer).unwrap();
        c.signed.headline = "forged".into();
        let err = verify_canary(&c).expect_err("tampered headline must fail verify");
        assert!(matches!(err, CanaryError::Signature(_)));
    }

    #[test]
    fn verify_canary_rejects_wrong_pubkey() {
        let signer = ed25519_signer();
        let attacker = ed25519_signer();
        let mut c = build_canary(a_date(), "real".into(), &signer).unwrap();
        // Swap the embedded pubkey to the attacker's, leaving
        // the legitimate signature in place.
        c.signed.pubkey_hex = hex::encode(attacker.pubkey());
        let err = verify_canary(&c).expect_err("pubkey swap must fail verify");
        assert!(matches!(err, CanaryError::Signature(_)));
    }

    #[tokio::test]
    async fn publish_canary_emits_gossip_event() {
        struct MockBroadcaster {
            sent: Vec<Vec<u8>>,
        }

        #[async_trait]
        impl CanaryBroadcaster for MockBroadcaster {
            async fn broadcast(&mut self, bytes: Vec<u8>) -> Result<(), String> {
                self.sent.push(bytes);
                Ok(())
            }
        }

        let signer = ed25519_signer();
        let c = build_canary(a_date(), "headline".into(), &signer).unwrap();

        let mut mock = MockBroadcaster { sent: vec![] };
        publish_canary(&c, &mut mock).await.expect("mock broadcast");

        assert_eq!(mock.sent.len(), 1, "publish must emit exactly one event");
        let decoded: Canary =
            serde_json::from_slice(&mock.sent[0]).expect("broadcast bytes must be valid JSON");
        assert_eq!(decoded, c);
        verify_canary(&decoded).expect("round-tripped canary must still verify");
    }

    /// Sprint 21 Phase E (T-NN tech debt resolved) — confirm
    /// `canary_wire_bytes` returns JCS canonical (RFC 8785) bytes.
    /// The cross-language guarantee : a Python observer that
    /// `jcs.canonicalize`s the same logical canary produces the
    /// exact same bytes, so a hash-of-wire-bytes (used downstream
    /// for dedup or observation tracking) matches across both
    /// implementations.
    #[test]
    fn wire_bytes_is_jcs_canonical_cross_language() {
        let signer = ed25519_signer();
        let canary = build_canary(a_date(), "headline".into(), &signer).unwrap();

        // 1. The wire bytes must equal `serde_jcs::to_vec` directly.
        let wire = canary_wire_bytes(&canary).expect("canary_wire_bytes works");
        let direct_jcs = serde_jcs::to_vec(&canary).expect("serde_jcs works");
        assert_eq!(
            wire, direct_jcs,
            "canary_wire_bytes must use serde_jcs (T-NN tech debt resolved)"
        );

        // 2. JCS guarantees lexicographic key ordering. Round-tripping
        //    via serde_json::Value (which loses ordering) and re-
        //    serialising via serde_jcs must produce the same bytes —
        //    this is the property a Python observer relies on
        //    (Python's `jcs.canonicalize(json.loads(bytes))` =
        //    `bytes`).
        let value: serde_json::Value =
            serde_json::from_slice(&wire).expect("wire bytes are valid JSON");
        let re_canonical = serde_jcs::to_vec(&value).expect("re-canonicalise");
        assert_eq!(
            wire, re_canonical,
            "JCS round-trip via Value must be byte-identical (cross-language guarantee)"
        );
    }

    #[test]
    fn topic_id_is_deterministic_and_32_bytes() {
        let a = warrant_canary_topic_id();
        let b = warrant_canary_topic_id();
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        assert_eq!(WARRANT_CANARY_TOPIC_SEED, b"nexus-grid/warrant-canary/v1");
    }

    #[test]
    fn build_canary_rejects_oversize_headline() {
        let signer = ed25519_signer();
        let oversize = "x".repeat(MAX_HEADLINE_LEN + 1);
        let err = build_canary(a_date(), oversize, &signer)
            .expect_err("headline over MAX_HEADLINE_LEN must fail");
        assert!(matches!(err, CanaryError::HeadlineTooLong(_)));
    }

    #[test]
    fn build_canary_accepts_headline_at_exact_cap() {
        let signer = ed25519_signer();
        let at_cap = "y".repeat(MAX_HEADLINE_LEN);
        build_canary(a_date(), at_cap, &signer)
            .expect("headline exactly at MAX_HEADLINE_LEN must succeed");
    }

    #[test]
    fn parse_canary_txt_round_trips_through_format() {
        let signer = ed25519_signer();
        let original = build_canary(a_date(), "headline text".into(), &signer).unwrap();
        let txt = format_canary_txt(&original);
        let parsed = parse_canary_txt(&txt).expect("round-trip parse");
        assert_eq!(parsed, original);
        verify_canary(&parsed).expect("round-tripped canary must verify");
    }

    #[test]
    fn format_canary_txt_contains_key_fields() {
        let signer = ed25519_signer();
        let c = build_canary(a_date(), "big news".into(), &signer).unwrap();
        let txt = format_canary_txt(&c);
        assert!(txt.contains("SBFB Warrant Canary"));
        assert!(txt.contains("Date: 2026-04-15 (UTC)"));
        assert!(txt.contains("Headline: big news"));
        assert!(txt.contains("Next scheduled update: 2026-05-30"));
        assert!(txt.contains(&c.signature_hex));
        assert!(txt.contains(&c.signed.pubkey_hex));
    }
}
