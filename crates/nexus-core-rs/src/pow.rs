// SPDX-License-Identifier: AGPL-3.0-or-later
//! Hashcash SHA256 proof-of-work primitive for gossip Sybil
//! resistance.
//!
//! Sprint 19 Phase B (HARDENING_ROADMAP §3 S19 item 1): before
//! publishing on a gossip topic, a node MUST solve a Hashcash
//! puzzle whose canonical bytes include both the topic id **and**
//! the publisher's Ed25519 pubkey. The solution is attached to
//! each broadcast and re-verified receiver-side, with a per
//! `(publisher_pubkey, topic)` cache amortising the cost across
//! a 15-minute session window.
//!
//! Why this shape:
//!
//! - **Publisher-bound** : the `publisher_pubkey` field inside
//!   [`HashcashChallenge`] makes a solved proof non-replayable
//!   across identities. A botnet that steals one solution from
//!   the wire cannot reuse it under a second pubkey.
//! - **Topic-bound** : the `topic` field prevents cross-topic
//!   replay. A proof solved for the curator list topic is invalid
//!   for the task dispatch topic, so an adversary must pay the
//!   per-topic cost to flood multiple topics.
//! - **Time-bound** : the `issued_at` field (unix seconds) lets
//!   the receiver reject proofs older than a configurable window.
//!   Sprint 19 policy rejects anything older than 30 minutes ;
//!   the 15-minute session cache guarantees a legit publisher
//!   never hits the boundary.
//! - **Stateless verify** : verification is a single SHA256 +
//!   leading-zero-bits compare. No state, no async, no I/O. The
//!   cache is pure optimisation ; a cold-start receiver that
//!   evicts its cache still delivers identical security.
//!
//! ## Standards compatibility
//!
//! Hashcash targets SHA256 historically — Bitcoin, Tor rend-point
//! PoW (2023), Lightning invoice PoW all use it. Using SHA256
//! rather than BLAKE3 (which is already a workspace dep and ~3x
//! faster) is a deliberate choice driven by S29
//! Cure53/ToB audit clarity : an external auditor can replay the
//! proof computation with any off-the-shelf SHA256 library
//! without SBFB-specific knowledge.
//!
//! ## Forward-compat paths
//!
//! - **S22 kudos-weighted admission** : the receiver verify path
//!   will add a `kudos_score >= policy.threshold` check alongside
//!   the PoW verify. The cache key stays `(pubkey, topic)` ; only
//!   the verify predicate changes.
//! - **S26 post-quantum migration** : `publisher_pubkey` is a
//!   `[u8; 32]` today (Ed25519). When the hybrid ML-DSA-65 +
//!   Ed25519 cutover lands, the field becomes a variable-length
//!   byte vector and the format version bumps `v1 → v2`. Since
//!   the v1.0 tag has not shipped, pre-launch policy
//!   (`CLAUDE.md §Pre-launch protocol policy`) lets us redefine
//!   the v1 bytes in place ; post-v1.0 the tolerant-decoder rule
//!   kicks in.
//!
//! ## Example
//!
//! ```no_run
//! # use nexus_core_rs::pow::{HashcashChallenge, solve, verify};
//! # use std::time::Duration;
//! let publisher_pubkey = [0xAB; 32];
//! let topic = [0xCD; 32];
//! let challenge = HashcashChallenge::new(topic, publisher_pubkey, 12);
//! let proof = solve(&challenge, Duration::from_secs(5)).expect("solve");
//! assert!(verify(&proof).is_ok());
//! ```

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::canonical::{canonical_bytes, DOMAIN_POW_V1};
use crate::crypto::PUBLIC_KEY_LENGTH;
use crate::error::NexusError;

/// Current PoW challenge format version.
///
/// Pre-launch policy (`CLAUDE.md §Pre-launch protocol policy`) :
/// stays at 1 until the v1.0 tag. A post-v1.0 PQC migration bumps
/// to 2 and ships a tolerant decoder.
pub const POW_FORMAT_VERSION: u16 = 1;

/// Default Hashcash difficulty in leading-zero bits for new gossip
/// subscriptions. 2^18 ≈ 262 144 hash evaluations, ≈ 100 ms on a
/// single modern CPU core (measured via `cargo bench --bench pow`).
///
/// Rationale (`sprint19_kickoff.md §4 D2`) :
/// - 2^20 (~400 ms) was rejected as too expensive for mobile
///   publishers.
/// - 2^16 (~25 ms) was rejected as too cheap for a botnet.
/// - 2^18 is the Tor rend-point PoW baseline and the Lightning
///   invoice PoW order of magnitude.
pub const DEFAULT_DIFFICULTY_BITS: u32 = 18;

/// Maximum difficulty the solver will accept. Higher values would
/// make [`solve`] run for hours on consumer hardware with no
/// operational benefit — the cap exists to fail loud on a
/// mis-configured policy rather than burning CPU silently.
pub const MAX_DIFFICULTY_BITS: u32 = 30;

/// Maximum age in seconds for an accepted proof. Proofs older
/// than this are rejected at verify time so a captured solution
/// cannot be replayed indefinitely. 30 minutes chosen to comfortably
/// exceed the 15-minute session cache window (sprint 19 plan §5).
pub const MAX_PROOF_AGE_SECS: u64 = 1_800;

/// Errors the PoW primitive surfaces to callers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum PowError {
    /// The solver exceeded its wall-clock budget before finding a
    /// valid nonce. Callers should surface this to the operator —
    /// it usually signals a mis-configured difficulty, not an
    /// attack.
    #[error("PoW solver timed out before finding a valid nonce")]
    Timeout,

    /// The proof's recomputed hash does not match the stored
    /// `hash` field. Indicates a tampered proof.
    #[error("PoW proof hash does not match recomputed hash")]
    HashMismatch,

    /// The proof's hash does not meet the declared difficulty.
    #[error("PoW proof does not meet difficulty (need {need} leading zero bits, got {got})")]
    InsufficientDifficulty {
        /// Declared difficulty (leading zero bits required).
        need: u32,
        /// Actual leading zero bits on the recomputed hash.
        got: u32,
    },

    /// The proof's declared difficulty exceeds
    /// [`MAX_DIFFICULTY_BITS`]. A proof with such an absurd
    /// difficulty is almost certainly fabricated.
    #[error("PoW difficulty {got} exceeds maximum {max}")]
    DifficultyOutOfRange {
        /// Declared difficulty.
        got: u32,
        /// Allowed maximum.
        max: u32,
    },

    /// The proof's `issued_at` is older than [`MAX_PROOF_AGE_SECS`]
    /// relative to the provided verification timestamp.
    #[error("PoW proof too old (issued {age_secs}s ago, max {max_secs}s)")]
    Expired {
        /// Observed age in seconds.
        age_secs: u64,
        /// Maximum age allowed.
        max_secs: u64,
    },

    /// The proof's `issued_at` is in the future relative to the
    /// verification timestamp. This is always a mis-configured
    /// clock on the publisher or a fabricated proof.
    #[error("PoW proof issued in the future ({skew_secs}s ahead of now)")]
    IssuedInFuture {
        /// Observed clock skew in seconds.
        skew_secs: u64,
    },

    /// The challenge's `format_version` is not [`POW_FORMAT_VERSION`].
    #[error("unknown PoW format version {got} (expected {expected})")]
    UnknownVersion {
        /// Observed version.
        got: u16,
        /// Expected version.
        expected: u16,
    },

    /// Canonical byte serialization failed. Should be unreachable —
    /// [`HashcashChallenge`] has no fields that can fail JCS.
    #[error("PoW canonical serialization failed: {0}")]
    Canonical(String),
}

impl From<NexusError> for PowError {
    fn from(e: NexusError) -> Self {
        PowError::Canonical(e.to_string())
    }
}

/// A Hashcash puzzle binding a topic id to a publisher pubkey at
/// a given moment.
///
/// Canonical-bytes-serialisable via [`canonical_bytes`] with the
/// [`DOMAIN_POW_V1`] domain tag. Two challenges with different
/// `topic` / `publisher_pubkey` / `issued_at` / `difficulty` /
/// `format_version` produce disjoint pre-image spaces — a solved
/// nonce for one is useless for another.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HashcashChallenge {
    /// Format version. Always [`POW_FORMAT_VERSION`] for v1. Named
    /// `v` on the wire to keep canonical bytes short.
    #[serde(rename = "v")]
    pub format_version: u16,

    /// 32-byte gossip topic id (typically derived from a
    /// BLAKE3-hashed seed via [`crate::curator::curator_topic_id`]
    /// or similar).
    pub topic: [u8; 32],

    /// Ed25519 public key of the publisher who will sign or
    /// broadcast messages under this proof. Binding the proof to
    /// a pubkey stops a botnet from harvesting one solution from
    /// the wire and reusing it under a fresh identity.
    pub publisher_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Unix seconds at which the challenge was issued. Verified
    /// against [`MAX_PROOF_AGE_SECS`] at receive time.
    pub issued_at: u64,

    /// Required leading zero bits on the SHA256 hash. Must be
    /// `<=` [`MAX_DIFFICULTY_BITS`].
    pub difficulty: u32,
}

impl HashcashChallenge {
    /// Create a fresh challenge at the current wall-clock time.
    ///
    /// Clamps the difficulty at [`MAX_DIFFICULTY_BITS`] defensively —
    /// a caller that requests more is probably reading a malformed
    /// policy file. The clamp is silent ; callers who want to fail
    /// loud on over-difficulty should check before constructing.
    pub fn new(
        topic: [u8; 32],
        publisher_pubkey: [u8; PUBLIC_KEY_LENGTH],
        difficulty: u32,
    ) -> Self {
        Self::new_at(topic, publisher_pubkey, difficulty, unix_now())
    }

    /// Create a challenge at an explicit unix-seconds timestamp.
    /// Used by tests to pin the clock.
    pub fn new_at(
        topic: [u8; 32],
        publisher_pubkey: [u8; PUBLIC_KEY_LENGTH],
        difficulty: u32,
        issued_at: u64,
    ) -> Self {
        HashcashChallenge {
            format_version: POW_FORMAT_VERSION,
            topic,
            publisher_pubkey,
            issued_at,
            difficulty: difficulty.min(MAX_DIFFICULTY_BITS),
        }
    }

    /// Serialize the challenge to its canonical byte
    /// representation, with the [`DOMAIN_POW_V1`] domain tag
    /// prefix. This is the pre-image the solver hashes.
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, PowError> {
        canonical_bytes(self, DOMAIN_POW_V1).map_err(PowError::from)
    }
}

/// A solved Hashcash proof. Holds the challenge, the winning
/// nonce, and the resulting 32-byte hash for quick rejection of
/// hand-crafted proofs at verify time (a caller that tampers with
/// `nonce` would have to recompute `hash` too, which defeats the
/// point of tampering).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HashcashProof {
    /// The challenge this proof solves.
    pub challenge: HashcashChallenge,

    /// The nonce that, hashed together with the challenge bytes,
    /// produces a digest with `challenge.difficulty` leading zero
    /// bits.
    pub nonce: u64,

    /// The resulting SHA256 digest. Stored so [`verify`] can
    /// reject a tampered `nonce` without re-running the full
    /// search. A caller that tampers with both `nonce` and `hash`
    /// is caught by the recompute-and-compare in [`verify`].
    pub hash: [u8; 32],
}

/// Compute the SHA256 digest of `(canonical_bytes(challenge) ||
/// nonce_le_bytes)`. Used by both the solver and the verifier,
/// so the two sides stay byte-identical.
fn sha256_of(challenge_bytes: &[u8], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(challenge_bytes);
    hasher.update(nonce.to_le_bytes());
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

/// Count the number of leading zero bits in a 32-byte digest.
/// Works by iterating bytes left-to-right and counting leading
/// zeros of the first non-zero byte.
pub fn leading_zero_bits(hash: &[u8; 32]) -> u32 {
    let mut count = 0u32;
    for &byte in hash.iter() {
        if byte == 0 {
            count += 8;
        } else {
            count += byte.leading_zeros();
            return count;
        }
    }
    count
}

/// Return the current time in unix seconds.
///
/// Falls back to 0 if the system clock is somehow before the
/// epoch. The solver's nonce search does not depend on this value
/// being monotonic ; only [`verify`] does, and it tolerates the
/// rare clock-before-epoch case by treating such proofs as
/// `IssuedInFuture` at the receiver.
fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Search for a nonce that makes `sha256(canonical(challenge) ||
/// nonce_le_bytes)` start with `challenge.difficulty` zero bits.
///
/// Bounded by `timeout` : if no solution is found in time, returns
/// [`PowError::Timeout`]. The solver checks the deadline every
/// 1 024 iterations to keep tight loops tight but still bail
/// quickly on over-difficulty policies.
pub fn solve(challenge: &HashcashChallenge, timeout: Duration) -> Result<HashcashProof, PowError> {
    if challenge.format_version != POW_FORMAT_VERSION {
        return Err(PowError::UnknownVersion {
            got: challenge.format_version,
            expected: POW_FORMAT_VERSION,
        });
    }
    if challenge.difficulty > MAX_DIFFICULTY_BITS {
        return Err(PowError::DifficultyOutOfRange {
            got: challenge.difficulty,
            max: MAX_DIFFICULTY_BITS,
        });
    }

    let challenge_bytes = challenge.to_canonical_bytes()?;
    let target_bits = challenge.difficulty;
    let deadline = Instant::now() + timeout;

    // Nonce iteration is deterministic : start at 0 and walk up.
    // Deterministic ordering makes the benchmark reproducible
    // across runs on the same difficulty, which matters for
    // regression testing. A random start point would hide
    // trivial-looking bugs (off-by-one on the target bits count,
    // for instance) behind wall-clock noise.
    for nonce in 0u64.. {
        if nonce % 1_024 == 0 && Instant::now() > deadline {
            return Err(PowError::Timeout);
        }
        let hash = sha256_of(&challenge_bytes, nonce);
        if leading_zero_bits(&hash) >= target_bits {
            return Ok(HashcashProof {
                challenge: challenge.clone(),
                nonce,
                hash,
            });
        }
    }
    // u64::MAX nonces exhausted without a hit is statistically
    // impossible for any realistic difficulty ; treat as timeout
    // for API simplicity.
    Err(PowError::Timeout)
}

/// Verify a proof without a clock check. Tests the hash matches
/// the recomputed digest and that the leading-zero-bits count
/// meets the declared difficulty. Pure function, no I/O.
pub fn verify_stateless(proof: &HashcashProof) -> Result<(), PowError> {
    if proof.challenge.format_version != POW_FORMAT_VERSION {
        return Err(PowError::UnknownVersion {
            got: proof.challenge.format_version,
            expected: POW_FORMAT_VERSION,
        });
    }
    if proof.challenge.difficulty > MAX_DIFFICULTY_BITS {
        return Err(PowError::DifficultyOutOfRange {
            got: proof.challenge.difficulty,
            max: MAX_DIFFICULTY_BITS,
        });
    }

    let challenge_bytes = proof.challenge.to_canonical_bytes()?;
    let recomputed = sha256_of(&challenge_bytes, proof.nonce);
    if recomputed != proof.hash {
        return Err(PowError::HashMismatch);
    }
    let bits = leading_zero_bits(&recomputed);
    if bits < proof.challenge.difficulty {
        return Err(PowError::InsufficientDifficulty {
            need: proof.challenge.difficulty,
            got: bits,
        });
    }
    Ok(())
}

/// Full verify : stateless checks + freshness bounds against the
/// caller-provided `now_unix_secs`. The receiver side of the
/// gossip PoW pipeline calls this.
pub fn verify_at(proof: &HashcashProof, now_unix_secs: u64) -> Result<(), PowError> {
    verify_stateless(proof)?;

    if proof.challenge.issued_at > now_unix_secs {
        return Err(PowError::IssuedInFuture {
            skew_secs: proof.challenge.issued_at - now_unix_secs,
        });
    }
    let age = now_unix_secs - proof.challenge.issued_at;
    if age > MAX_PROOF_AGE_SECS {
        return Err(PowError::Expired {
            age_secs: age,
            max_secs: MAX_PROOF_AGE_SECS,
        });
    }
    Ok(())
}

/// Full verify using the current system clock.
pub fn verify(proof: &HashcashProof) -> Result<(), PowError> {
    verify_at(proof, unix_now())
}

// =================================================================
// Escalating difficulty policy
// =================================================================

/// Geometric difficulty ramp per (consumer, model) tuple.
///
/// Each `tranche_size` tasks the consumer submits for a given model,
/// the required difficulty doubles (×`multiplier`). The ramp resets
/// daily at midnight UTC.
///
/// The policy is coordinator-local (not serialised on the wire) — it
/// drives the `difficulty` field value injected into
/// [`HashcashChallenge`] at solve time.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalatingPolicy {
    /// Starting difficulty (leading zero bits) at task_count=0.
    pub base_difficulty: u32,
    /// Geometric factor applied per tranche (typically 2.0).
    pub multiplier: f64,
    /// Number of tasks per escalation step.
    pub tranche_size: u32,
    /// Ceiling difficulty (will not exceed this OR [`MAX_DIFFICULTY_BITS`]).
    pub max_difficulty: u32,
}

impl Default for EscalatingPolicy {
    fn default() -> Self {
        Self {
            base_difficulty: DEFAULT_DIFFICULTY_BITS,
            multiplier: 2.0,
            tranche_size: 10,
            max_difficulty: MAX_DIFFICULTY_BITS,
        }
    }
}

/// Compute the effective difficulty for a consumer that has already
/// submitted `task_count` tasks in the current daily window.
///
/// Formula: `base_difficulty × multiplier^(task_count / tranche_size)`
/// clamped to `[base_difficulty, max_difficulty]` and to
/// [`MAX_DIFFICULTY_BITS`] as an absolute ceiling.
pub fn escalating_difficulty(policy: &EscalatingPolicy, task_count: u64) -> u32 {
    if task_count == 0 || policy.tranche_size == 0 {
        return policy.base_difficulty.min(MAX_DIFFICULTY_BITS);
    }
    let exponent = task_count / u64::from(policy.tranche_size);
    if exponent == 0 {
        return policy.base_difficulty.min(MAX_DIFFICULTY_BITS);
    }
    let factor = policy.multiplier.powi(exponent as i32);
    let raw = (f64::from(policy.base_difficulty) * factor).round() as u64;
    let clamped = raw
        .min(u64::from(policy.max_difficulty))
        .min(u64::from(MAX_DIFFICULTY_BITS));
    (clamped as u32).max(policy.base_difficulty)
}

/// Check whether a daily reset is due given the last reset timestamp.
///
/// Returns `true` if midnight UTC has passed since `last_reset`.
/// Uses day-of-epoch comparison: `unix_secs / 86400`.
pub fn should_reset_daily(last_reset: SystemTime) -> bool {
    let last_secs = last_reset
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    now_secs / 86_400 > last_secs / 86_400
}

/// Deterministic variant for tests: compare two explicit timestamps.
pub fn should_reset_daily_at(last_reset_unix: u64, now_unix: u64) -> bool {
    now_unix / 86_400 > last_reset_unix / 86_400
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_challenge(difficulty: u32) -> HashcashChallenge {
        HashcashChallenge::new([0x11; 32], [0x22; PUBLIC_KEY_LENGTH], difficulty)
    }

    #[test]
    fn solve_then_verify_happy() {
        // Low difficulty so the test runs in <100ms even on slow CI
        // runners.
        let c = mk_challenge(12);
        let proof = solve(&c, Duration::from_secs(5)).expect("solve 12 bits");
        verify(&proof).expect("verify");
        assert_eq!(proof.challenge, c);
        assert!(leading_zero_bits(&proof.hash) >= 12);
    }

    #[test]
    fn verify_rejects_tampered_nonce() {
        let c = mk_challenge(10);
        let mut proof = solve(&c, Duration::from_secs(2)).unwrap();
        proof.nonce = proof.nonce.wrapping_add(1);
        // The stored hash is now inconsistent with the tampered
        // nonce → HashMismatch.
        assert_eq!(verify(&proof), Err(PowError::HashMismatch));
    }

    #[test]
    fn verify_rejects_tampered_hash() {
        let c = mk_challenge(10);
        let mut proof = solve(&c, Duration::from_secs(2)).unwrap();
        proof.hash[0] ^= 0x01;
        // Recomputed hash no longer matches the tampered field.
        assert_eq!(verify(&proof), Err(PowError::HashMismatch));
    }

    #[test]
    fn verify_rejects_downgraded_difficulty() {
        // An attacker who solves at difficulty 4 and presents the
        // proof with difficulty = 20 is caught by the leading-zero
        // re-check. We simulate the inverse : find a low-difficulty
        // solution then claim high difficulty.
        let c_low = mk_challenge(4);
        let mut proof = solve(&c_low, Duration::from_secs(1)).unwrap();
        proof.challenge.difficulty = 20;
        // But the stored hash is for a 4-bit solution — so the
        // recomputed hash no longer matches (the canonical bytes
        // of the challenge now encode difficulty=20).
        // Fallback case : HashMismatch surfaces first.
        assert_eq!(verify(&proof), Err(PowError::HashMismatch));
    }

    #[test]
    fn solve_difficulty_zero_is_trivial() {
        // Difficulty 0 means "any hash works" → nonce 0 wins.
        let c = mk_challenge(0);
        let proof = solve(&c, Duration::from_millis(10)).unwrap();
        assert_eq!(proof.nonce, 0);
        verify(&proof).unwrap();
    }

    #[test]
    fn solve_timeout_fires_on_overdifficulty() {
        // 28 bits ≈ 268M hashes, won't complete in 50ms on any
        // realistic CPU. The solver must bail with Timeout rather
        // than running forever.
        let c = mk_challenge(28);
        let started = Instant::now();
        let result = solve(&c, Duration::from_millis(50));
        assert_eq!(result, Err(PowError::Timeout));
        // Sanity : the solver obeyed the budget within reason
        // (allow 2x slack for slow CI runners).
        assert!(started.elapsed() < Duration::from_millis(500));
    }

    #[test]
    fn different_topics_yield_different_solutions() {
        // Same pubkey + same difficulty, different topic → the
        // canonical bytes differ, so the hash-matching nonce
        // differs (with overwhelming probability at non-trivial
        // difficulties). We solve two and check nonces are
        // different.
        let c_a = HashcashChallenge::new_at([0xAA; 32], [0x22; 32], 8, 1_000_000);
        let c_b = HashcashChallenge::new_at([0xBB; 32], [0x22; 32], 8, 1_000_000);
        let p_a = solve(&c_a, Duration::from_secs(2)).unwrap();
        let p_b = solve(&c_b, Duration::from_secs(2)).unwrap();
        assert_ne!(p_a.nonce, p_b.nonce);
        // And a proof for A does not verify under B's challenge.
        let mut p_a_as_b = p_a.clone();
        p_a_as_b.challenge.topic = [0xBB; 32];
        assert!(verify(&p_a_as_b).is_err());
    }

    #[test]
    fn different_publishers_yield_different_solutions() {
        // Sprint 19 design invariant : proofs are publisher-bound.
        // Two publishers on the same topic must solve separately.
        let c_a = HashcashChallenge::new_at([0x11; 32], [0xAA; 32], 8, 1_000_000);
        let c_b = HashcashChallenge::new_at([0x11; 32], [0xBB; 32], 8, 1_000_000);
        let p_a = solve(&c_a, Duration::from_secs(2)).unwrap();
        // A's proof cannot be rebranded as B's.
        let mut p_a_as_b = p_a.clone();
        p_a_as_b.challenge.publisher_pubkey = [0xBB; 32];
        assert!(verify(&p_a_as_b).is_err());
        // B solves independently.
        let p_b = solve(&c_b, Duration::from_secs(2)).unwrap();
        assert_ne!(p_a.nonce, p_b.nonce);
    }

    #[test]
    fn verify_rejects_future_issued_at() {
        // Issue the proof 10 s in the future relative to the
        // caller-provided "now", then verify. Must surface
        // IssuedInFuture rather than HashMismatch (the hash is
        // valid — the challenge is just from the future).
        let now = 1_000_000u64;
        let future = HashcashChallenge {
            format_version: POW_FORMAT_VERSION,
            topic: [0x11; 32],
            publisher_pubkey: [0x22; 32],
            issued_at: now + 10,
            difficulty: 4,
        };
        let p = solve(&future, Duration::from_secs(1)).unwrap();
        match verify_at(&p, now) {
            Err(PowError::IssuedInFuture { skew_secs }) => assert_eq!(skew_secs, 10),
            other => panic!("expected IssuedInFuture, got {other:?}"),
        }
    }

    #[test]
    fn verify_rejects_expired_proof() {
        let c = HashcashChallenge {
            format_version: POW_FORMAT_VERSION,
            topic: [0x11; 32],
            publisher_pubkey: [0x22; 32],
            issued_at: 1_000_000,
            difficulty: 4,
        };
        let proof = solve(&c, Duration::from_secs(1)).unwrap();
        // Now = issued_at + 2 * MAX_PROOF_AGE_SECS → expired.
        let now = 1_000_000 + 2 * MAX_PROOF_AGE_SECS;
        match verify_at(&proof, now) {
            Err(PowError::Expired { age_secs, max_secs }) => {
                assert_eq!(age_secs, 2 * MAX_PROOF_AGE_SECS);
                assert_eq!(max_secs, MAX_PROOF_AGE_SECS);
            }
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    #[test]
    fn canonical_bytes_are_deterministic_and_include_domain_tag() {
        // The canonical serialization must be byte-stable across
        // calls (JCS + the v1 domain tag). This is what makes
        // cross-language verification viable — a Python-side PoW
        // verifier would use `jcs` + `DOMAIN_POW_V1` and get the
        // same bytes.
        let c = HashcashChallenge::new_at([0x33; 32], [0x44; 32], 6, 1_700_000_000);
        let a = c.to_canonical_bytes().unwrap();
        let b = c.to_canonical_bytes().unwrap();
        assert_eq!(a, b);
        assert!(a.starts_with(DOMAIN_POW_V1));
        assert_eq!(a[DOMAIN_POW_V1.len()], 0);
    }

    #[test]
    fn leading_zero_bits_boundaries() {
        // All-zero hash : 256 bits.
        let all_zero = [0u8; 32];
        assert_eq!(leading_zero_bits(&all_zero), 256);
        // 0x80 in the first byte : 0 leading zero bits.
        let first_bit_set = {
            let mut h = [0u8; 32];
            h[0] = 0x80;
            h
        };
        assert_eq!(leading_zero_bits(&first_bit_set), 0);
        // 0x01 in the first byte : 7 leading zero bits.
        let lowest_bit = {
            let mut h = [0u8; 32];
            h[0] = 0x01;
            h
        };
        assert_eq!(leading_zero_bits(&lowest_bit), 7);
        // First byte zero, second byte 0x08 : 8 + 4 = 12.
        let twelve = {
            let mut h = [0u8; 32];
            h[1] = 0x08;
            h
        };
        assert_eq!(leading_zero_bits(&twelve), 12);
    }

    #[test]
    fn solve_rejects_unknown_version() {
        let c = HashcashChallenge {
            format_version: 99,
            topic: [0x11; 32],
            publisher_pubkey: [0x22; 32],
            issued_at: 0,
            difficulty: 4,
        };
        assert_eq!(
            solve(&c, Duration::from_millis(10)),
            Err(PowError::UnknownVersion {
                got: 99,
                expected: POW_FORMAT_VERSION,
            })
        );
    }

    #[test]
    fn new_clamps_difficulty_at_maximum() {
        // A caller who requests 100 bits of difficulty gets MAX
        // back — the constructor fails closed rather than letting
        // a runaway policy burn CPU in solve().
        let c = HashcashChallenge::new([0; 32], [0; 32], 100);
        assert_eq!(c.difficulty, MAX_DIFFICULTY_BITS);
    }

    // =============================================================
    // Escalating difficulty policy tests
    // =============================================================

    fn default_escalating() -> EscalatingPolicy {
        EscalatingPolicy {
            base_difficulty: 18,
            multiplier: 2.0,
            tranche_size: 10,
            max_difficulty: 26,
        }
    }

    #[test]
    fn escalating_difficulty_base_at_zero_count() {
        let p = default_escalating();
        assert_eq!(escalating_difficulty(&p, 0), 18);
    }

    #[test]
    fn escalating_difficulty_ramp_first_tranche() {
        let _p = default_escalating();
        // count=10 → exponent=1 → 18×2=36 → clamped to max 26
        // Use lower base to see the ramp clearly:
        let p2 = EscalatingPolicy {
            base_difficulty: 10,
            multiplier: 2.0,
            tranche_size: 10,
            max_difficulty: 26,
        };
        assert_eq!(escalating_difficulty(&p2, 10), 20); // 10×2^1=20
    }

    #[test]
    fn escalating_difficulty_ramp_third_tranche() {
        let p = EscalatingPolicy {
            base_difficulty: 10,
            multiplier: 2.0,
            tranche_size: 10,
            max_difficulty: 30,
        };
        // count=30 → exponent=3 → 10×2^3=80 → clamped to 30
        assert_eq!(escalating_difficulty(&p, 30), 30);
        // count=20 → exponent=2 → 10×4=40 → clamped to 30
        assert_eq!(escalating_difficulty(&p, 20), 30);
        // With higher max to see unclamped:
        let p2 = EscalatingPolicy {
            base_difficulty: 4,
            multiplier: 2.0,
            tranche_size: 5,
            max_difficulty: 30,
        };
        // count=15 → exponent=3 → 4×8=32 → clamped MAX_DIFFICULTY_BITS=30
        assert_eq!(escalating_difficulty(&p2, 15), 30);
    }

    #[test]
    fn escalating_difficulty_cap_max() {
        let policy = default_escalating();
        // count=100_000 → exponent=10_000 → astronomic → clamped
        assert_eq!(
            escalating_difficulty(&policy, 100_000),
            policy.max_difficulty
        );
    }

    #[test]
    fn escalating_difficulty_within_first_tranche_stays_base() {
        let p = default_escalating();
        // count < tranche_size → exponent=0 → base
        for count in 1..10 {
            assert_eq!(escalating_difficulty(&p, count), 18);
        }
    }

    #[test]
    fn escalating_difficulty_overflow_saturates_max() {
        let p = EscalatingPolicy {
            base_difficulty: 20,
            multiplier: 3.0,
            tranche_size: 1,
            max_difficulty: 28,
        };
        // count=100 → exponent=100 → 20×3^100 → overflow → clamped
        assert_eq!(escalating_difficulty(&p, 100), 28);
    }

    #[test]
    fn escalating_difficulty_zero_tranche_returns_base() {
        let p = EscalatingPolicy {
            tranche_size: 0,
            ..default_escalating()
        };
        assert_eq!(escalating_difficulty(&p, 50), 18);
    }

    #[test]
    fn escalating_difficulty_fractional_multiplier() {
        let p = EscalatingPolicy {
            base_difficulty: 10,
            multiplier: 1.5,
            tranche_size: 5,
            max_difficulty: 30,
        };
        // count=5 → exponent=1 → 10×1.5=15
        assert_eq!(escalating_difficulty(&p, 5), 15);
        // count=10 → exponent=2 → 10×2.25=23 (rounded)
        assert_eq!(escalating_difficulty(&p, 10), 23);
    }

    #[test]
    fn should_reset_daily_same_day_is_false() {
        // Two timestamps in the same UTC day → no reset
        let noon = 86_400 * 100 + 43_200; // day 100, 12:00
        let evening = 86_400 * 100 + 79_200; // day 100, 22:00
        assert!(!should_reset_daily_at(noon, evening));
    }

    #[test]
    fn should_reset_daily_next_day_is_true() {
        let late_night = 86_400 * 100 + 86_399; // day 100, 23:59:59
        let next_morning = 86_400 * 101 + 1; // day 101, 00:00:01
        assert!(should_reset_daily_at(late_night, next_morning));
    }

    #[test]
    fn should_reset_daily_epoch_to_day_one() {
        assert!(should_reset_daily_at(0, 86_400));
        assert!(!should_reset_daily_at(0, 86_399));
    }
}
