// SPDX-License-Identifier: AGPL-3.0-or-later
//! Peer-attested age witness for gossip admission (Couche 1).
//!
//! Sprint 22 Phase C. A new node joining a gossip topic must prove
//! age ≥ [`MIN_AGE_DAYS`] days on top of the Sprint 19 Hashcash PoW
//! gate. Because `iroh 0.98` does not expose an intrinsic node-id
//! timestamp (confirmed `docs.rs/iroh` + `iroh-gossip` CHANGELOG),
//! age is asserted via an externally-signed witness : an already-
//! established peer (a node active in the mesh for at least
//! [`MIN_WITNESS_AGE_DAYS`] days, verified at runtime by the admission
//! logic in [`crate::gossip`]) signs a statement saying "I saw
//! `node_id` for the first time at `first_seen_ts`".
//!
//! Rejected alternatives (cf. kickoff §4 D1 Couche 1) :
//!
//! - **Pure PoW** — IEEE S&P 2024 formal impossibility (arXiv
//!   2212.05197) on behaviour-score admission evadable by
//!   behaviour-faking.
//! - **Tor-Guard-style dirauth-centralised age** — incompatible
//!   with the SBFB charter "No central server".
//! - **Node_id intrinsic timestamp** — not available in iroh 0.98.
//!
//! ## Wire format
//!
//! ```text
//! AgeWitness {
//!     node_id:          [u8; 32],   // the witnessed peer's Ed25519 pubkey
//!     first_seen_ts:    i64,        // unix seconds, when witness first saw node_id
//!     witness_pubkey:   [u8; 32],   // the witnessing peer's Ed25519 pubkey
//!     witness_sig:      [u8; 64],   // Ed25519 over JCS canonical bytes with
//!                                   // DOMAIN_AGE_WITNESS_V1 prefix
//! }
//! ```
//!
//! ## Pre-launch policy
//!
//! Stable pre-launch. No `version` field : any redefinition of the
//! byte layout lands as an in-place edit of v1 until the first
//! `v1.0` tag (cf. `CLAUDE.md` §Pre-launch protocol policy).

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{canonical_bytes, DOMAIN_AGE_WITNESS_V1};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// Minimum age (days) required for a witnessed node to pass the
/// Couche 1 admission gate. 7 days mirrors the Tor Guard 8-day
/// eligibility rule (empirically tuned), minus one day of headroom
/// to account for clock skew between witness and verifier.
pub const MIN_AGE_DAYS: i64 = 7;

/// Minimum age (days) of the witnessing peer itself. A brand-new
/// peer cannot witness a newer peer — that would let an attacker
/// bootstrap a Sybil chain where node_0 witnesses node_1 witnesses
/// node_2 etc. 30 days establishes a chain-breaking gap : an
/// attacker who wants to admit a fresh Sybil must first sustain a
/// witness identity for a month.
pub const MIN_WITNESS_AGE_DAYS: i64 = 30;

/// Seconds per day, used for age arithmetic. Declared here so the
/// [`AgeWitness::age_days`] computation does not drift if a reader
/// "fixes" the math inline.
pub const SECONDS_PER_DAY: i64 = 86_400;

/// Error type for age witness operations. Kept distinct from
/// [`NexusError`] so call sites can discriminate without matching a
/// string message — the gossip admission logic in
/// [`crate::gossip`] uses the distinction to emit precise warn logs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgeWitnessError {
    /// The signature failed Ed25519 verification.
    BadSignature(String),
    /// The witnessed age is below [`MIN_AGE_DAYS`].
    Underage {
        /// Age at verification time (days).
        age_days: i64,
        /// Threshold that was not met.
        required: i64,
    },
    /// The witness claims a first-seen timestamp in the future
    /// (clock skew or deliberate forgery). Verifiers must reject
    /// any witness with `first_seen_ts > now`.
    FutureTimestamp {
        /// `first_seen_ts` carried by the witness.
        claimed: i64,
        /// `now` at verification time.
        now: i64,
    },
    /// Canonical bytes generation failed (should be impossible for
    /// this struct but kept for defence in depth).
    CanonicalFailed(String),
}

impl std::fmt::Display for AgeWitnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgeWitnessError::BadSignature(msg) => {
                write!(f, "age witness signature invalid: {msg}")
            }
            AgeWitnessError::Underage { age_days, required } => write!(
                f,
                "age witness claim below minimum (got {age_days}d, required >= {required}d)"
            ),
            AgeWitnessError::FutureTimestamp { claimed, now } => write!(
                f,
                "age witness first_seen_ts={claimed} is in the future (now={now})"
            ),
            AgeWitnessError::CanonicalFailed(msg) => {
                write!(f, "age witness canonical bytes failed: {msg}")
            }
        }
    }
}

impl std::error::Error for AgeWitnessError {}

impl From<AgeWitnessError> for NexusError {
    fn from(e: AgeWitnessError) -> Self {
        NexusError::Crypto(e.to_string())
    }
}

/// A peer-signed age attestation for a given `node_id`.
///
/// The signing peer (`witness_pubkey`) asserts under the
/// [`DOMAIN_AGE_WITNESS_V1`] tag that they first observed
/// `node_id` at unix time `first_seen_ts`.
///
/// Verifiers in the gossip admission path (cf.
/// [`crate::gossip::join_topic_with_age_witness`]) check :
/// 1. `witness_sig` is a valid Ed25519 signature by
///    `witness_pubkey` over the canonical bytes.
/// 2. `first_seen_ts <= now` (no future-dated witnesses).
/// 3. `now - first_seen_ts >= MIN_AGE_DAYS * 86_400` seconds.
/// 4. `witness_pubkey` itself is known to the mesh for
///    `>= MIN_WITNESS_AGE_DAYS` days (enforced at call site, not in
///    [`Self::verify`] — this module is the crypto layer only).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgeWitness {
    /// Ed25519 public key of the witnessed peer (the node joining
    /// the gossip topic).
    pub node_id: [u8; PUBLIC_KEY_LENGTH],

    /// Unix timestamp (seconds) when the witness first saw
    /// `node_id`. Verifiers reject any value greater than the
    /// current time.
    pub first_seen_ts: i64,

    /// Ed25519 public key of the witnessing peer.
    pub witness_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Ed25519 signature by `witness_pubkey` over
    /// [`canonical_bytes`] of the payload (everything above this
    /// field) with [`DOMAIN_AGE_WITNESS_V1`] as the domain tag.
    #[serde(with = "BigArray")]
    pub witness_sig: [u8; SIGNATURE_BYTES],
}

/// Internal signable payload : an [`AgeWitness`] without its
/// `witness_sig`. Used both at sign time and verify time to produce
/// the exact same byte string from the inside ([`AgeWitness::sign`])
/// and from the outside ([`AgeWitness::verify`]).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct AgeWitnessPayload {
    node_id: [u8; PUBLIC_KEY_LENGTH],
    first_seen_ts: i64,
    witness_pubkey: [u8; PUBLIC_KEY_LENGTH],
}

impl AgeWitness {
    /// Produce a signed [`AgeWitness`] from a raw witness keypair.
    ///
    /// The caller is the witnessing peer ; `witness_keypair` must
    /// control the Ed25519 key published as their `witness_pubkey`
    /// in the mesh. `first_seen_ts` is whatever the witness
    /// recorded when they first admitted `node_id` through their
    /// local neighbor-set heuristics.
    pub fn sign(
        node_id: [u8; PUBLIC_KEY_LENGTH],
        first_seen_ts: i64,
        witness_keypair: &KeyPair,
    ) -> Result<Self> {
        let witness_pubkey = witness_keypair.public_bytes();
        let payload = AgeWitnessPayload {
            node_id,
            first_seen_ts,
            witness_pubkey,
        };
        let bytes = canonical_bytes(&payload, DOMAIN_AGE_WITNESS_V1).map_err(|e| {
            NexusError::Crypto(AgeWitnessError::CanonicalFailed(e.to_string()).to_string())
        })?;
        let witness_sig = witness_keypair.sign(&bytes);
        Ok(AgeWitness {
            node_id,
            first_seen_ts,
            witness_pubkey,
            witness_sig,
        })
    }

    /// Verify the signature on this witness. Does **not** check
    /// age ; for the full admission gate use
    /// [`Self::verify_with_age`].
    pub fn verify_signature(&self) -> std::result::Result<(), AgeWitnessError> {
        let payload = AgeWitnessPayload {
            node_id: self.node_id,
            first_seen_ts: self.first_seen_ts,
            witness_pubkey: self.witness_pubkey,
        };
        let bytes = canonical_bytes(&payload, DOMAIN_AGE_WITNESS_V1)
            .map_err(|e| AgeWitnessError::CanonicalFailed(e.to_string()))?;
        crate::crypto::verify(&self.witness_pubkey, &bytes, &self.witness_sig)
            .map_err(|e| AgeWitnessError::BadSignature(e.to_string()))
    }

    /// Full admission check : signature valid + timestamp not in
    /// the future + age ≥ [`MIN_AGE_DAYS`].
    ///
    /// Verifiers should also enforce that `self.witness_pubkey`
    /// itself is known to the mesh for at least
    /// [`MIN_WITNESS_AGE_DAYS`] days. That check lives in the
    /// gossip runtime which has the mesh state ; this function is
    /// intentionally stateless.
    pub fn verify_with_age(&self, now_ts: i64) -> std::result::Result<(), AgeWitnessError> {
        self.verify_signature()?;
        if self.first_seen_ts > now_ts {
            return Err(AgeWitnessError::FutureTimestamp {
                claimed: self.first_seen_ts,
                now: now_ts,
            });
        }
        let age = self.age_days(now_ts);
        if age < MIN_AGE_DAYS {
            return Err(AgeWitnessError::Underage {
                age_days: age,
                required: MIN_AGE_DAYS,
            });
        }
        Ok(())
    }

    /// Compute age in days at `now_ts`. Clamped to zero when
    /// `now_ts < first_seen_ts` (future witnesses would otherwise
    /// report negative ages).
    pub fn age_days(&self, now_ts: i64) -> i64 {
        let delta = now_ts.saturating_sub(self.first_seen_ts);
        if delta < 0 {
            0
        } else {
            delta / SECONDS_PER_DAY
        }
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node_id() -> [u8; PUBLIC_KEY_LENGTH] {
        [0xABu8; PUBLIC_KEY_LENGTH]
    }

    #[test]
    fn sign_verify_roundtrip() {
        let witness = KeyPair::generate();
        let w = AgeWitness::sign(sample_node_id(), 1_700_000_000, &witness).expect("sign succeeds");
        w.verify_signature().expect("signature verifies");
        assert_eq!(w.node_id, sample_node_id());
        assert_eq!(w.witness_pubkey, witness.public_bytes());
        assert_eq!(w.first_seen_ts, 1_700_000_000);
    }

    #[test]
    fn verify_rejects_tampered_node_id() {
        let witness = KeyPair::generate();
        let mut w = AgeWitness::sign(sample_node_id(), 1_700_000_000, &witness).expect("sign");
        // Mutate the witnessed node_id post-signing ; signature no
        // longer matches.
        w.node_id = [0xCDu8; PUBLIC_KEY_LENGTH];
        let err = w.verify_signature().expect_err("tampered must reject");
        assert!(matches!(err, AgeWitnessError::BadSignature(_)));
    }

    #[test]
    fn verify_rejects_tampered_first_seen_ts() {
        let witness = KeyPair::generate();
        let mut w = AgeWitness::sign(sample_node_id(), 1_700_000_000, &witness).expect("sign");
        // Move the timestamp 30 days earlier to claim extra age.
        w.first_seen_ts -= 30 * SECONDS_PER_DAY;
        let err = w.verify_signature().expect_err("tampered ts must reject");
        assert!(matches!(err, AgeWitnessError::BadSignature(_)));
    }

    #[test]
    fn age_days_precision_and_edge_cases() {
        let witness = KeyPair::generate();
        let base_ts = 1_700_000_000_i64;
        let w = AgeWitness::sign(sample_node_id(), base_ts, &witness).expect("sign");

        // Same-day → 0.
        assert_eq!(w.age_days(base_ts), 0);
        // Exactly 1 day later → 1.
        assert_eq!(w.age_days(base_ts + SECONDS_PER_DAY), 1);
        // 7 days + 1 second → still 7.
        assert_eq!(w.age_days(base_ts + 7 * SECONDS_PER_DAY + 1), 7);
        // 7 days - 1 second → 6.
        assert_eq!(w.age_days(base_ts + 7 * SECONDS_PER_DAY - 1), 6);
        // now_ts < first_seen_ts → clamp to 0 (future witness).
        assert_eq!(w.age_days(base_ts - SECONDS_PER_DAY), 0);
    }

    #[test]
    fn min_age_enforced_rejects_6_days_admits_7_days() {
        let witness = KeyPair::generate();
        let first_seen = 1_700_000_000_i64;
        let w = AgeWitness::sign(sample_node_id(), first_seen, &witness).expect("sign");

        // 6d 23h → reject.
        let now_under = first_seen + 7 * SECONDS_PER_DAY - 3600;
        let err = w
            .verify_with_age(now_under)
            .expect_err("6d 23h must reject");
        assert!(matches!(err, AgeWitnessError::Underage { .. }));

        // Exactly 7d → admit.
        let now_ok = first_seen + 7 * SECONDS_PER_DAY;
        w.verify_with_age(now_ok).expect("7d admitted");

        // 30d → admit.
        let now_ok_big = first_seen + 30 * SECONDS_PER_DAY;
        w.verify_with_age(now_ok_big).expect("30d admitted");
    }

    #[test]
    fn verify_rejects_future_timestamp() {
        let witness = KeyPair::generate();
        let first_seen = 2_000_000_000_i64; // far-future
        let w = AgeWitness::sign(sample_node_id(), first_seen, &witness).expect("sign");
        // now is before first_seen → FutureTimestamp.
        let err = w
            .verify_with_age(1_000_000_000)
            .expect_err("future ts must reject");
        assert!(matches!(err, AgeWitnessError::FutureTimestamp { .. }));
    }

    #[test]
    fn different_witnesses_produce_distinct_signatures() {
        let w1 = KeyPair::generate();
        let w2 = KeyPair::generate();
        let a1 = AgeWitness::sign(sample_node_id(), 1_700_000_000, &w1).expect("sign 1");
        let a2 = AgeWitness::sign(sample_node_id(), 1_700_000_000, &w2).expect("sign 2");
        assert_ne!(a1.witness_pubkey, a2.witness_pubkey);
        assert_ne!(a1.witness_sig, a2.witness_sig);
    }

    #[test]
    fn canonical_bytes_are_domain_separated_from_other_payloads() {
        let witness = KeyPair::generate();
        let payload = AgeWitnessPayload {
            node_id: sample_node_id(),
            first_seen_ts: 1_700_000_000,
            witness_pubkey: witness.public_bytes(),
        };
        let as_witness = canonical_bytes(&payload, DOMAIN_AGE_WITNESS_V1).unwrap();
        let as_task = canonical_bytes(&payload, crate::canonical::DOMAIN_TASK_V1).unwrap();
        // Same struct bytes but different domain tag → different
        // canonical bytes by construction. Cross-replay impossible.
        assert_ne!(as_witness, as_task);
    }
}
