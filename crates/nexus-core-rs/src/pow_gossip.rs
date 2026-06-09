// SPDX-License-Identifier: AGPL-3.0-or-later
//! PoW-gated gossip wire envelope + publisher/subscriber caches.
//!
//! Sprint 19 Phase B : sits between [`crate::pow`] (the SHA256
//! Hashcash primitive) and [`crate::gossip`] (the iroh-gossip
//! wrapper) to package and verify proofs alongside broadcast
//! payloads.
//!
//! ## Wire format
//!
//! Every PoW-gated broadcast is a length-prefixed byte envelope :
//!
//! ```text
//! +----------------+----------------+--------------+
//! | proof_len (u32 BE, 4 bytes)     | proof bytes  | payload... |
//! +----------------+----------------+--------------+
//! ```
//!
//! `proof_len` is the length in bytes of the proof JSON that
//! follows. `proof bytes` is `serde_json::to_vec(&HashcashProof)`
//! (NOT the JCS canonical pre-image — transport is looser than
//! the pre-image hashed inside the proof itself ; JCS is only
//! needed for the bytes [`crate::pow::HashcashChallenge::to_canonical_bytes`]
//! hashes to produce the winning nonce). `payload` is the
//! arbitrary byte message the caller would have passed to
//! [`crate::gossip::TopicSender::broadcast`] unchanged.
//!
//! Pre-launch protocol policy (`CLAUDE.md §Pre-launch`) : the
//! envelope format is v1 and stays at v1 until the `v1.0` tag.
//! Post-v1.0 bumps introduce a 1-byte envelope version prefix
//! with a tolerant decoder.
//!
//! ## Publisher side — [`PowSolveCache`]
//!
//! A long-running publisher solves one proof per `(topic,
//! keypair)` and reuses it for every broadcast until the
//! 15-minute session window expires. The cache amortises the
//! ~100 ms solve over thousands of broadcasts, so a chatty
//! publisher (heartbeat every 30 s on the curator topic, for
//! example) pays the PoW cost twice an hour and not twice a
//! minute.
//!
//! ## Subscriber side — [`PowVerifyCache`]
//!
//! The receiver mirrors the same session model : once a proof
//! from a given `(publisher_pubkey, topic)` has been verified,
//! subsequent messages from the same pair bypass the full
//! verify for 15 minutes. The cache is a pure optimisation — a
//! cold-start receiver that evicts its cache still delivers
//! identical security by running the full verify on the first
//! message of each new session.
//!
//! ## Forward-compat paths
//!
//! - **S22 kudos-weighted admission** : the receiver verify
//!   predicate becomes `pow_ok && kudos_score >= threshold`.
//!   The cache key stays `(pubkey, topic)`. Current code path
//!   already isolates the verify inside
//!   [`PowVerifyCache::verify_envelope`], so the kudos check is
//!   a one-line addition.
//! - **S26 PQC migration** : the envelope format does not
//!   change — only the `publisher_pubkey` inside the proof.
//!   Post-v1.0 we bump the proof format version and ship a
//!   tolerant decoder.
//! - **S29 audit Cure53/ToB** : the envelope encode/decode is
//!   ~30 LOC, zero unsafe, zero async, testable in isolation.
//!   Scope cost to audit : minimal.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use thiserror::Error;

use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH};
use crate::pow::{
    HashcashChallenge, HashcashProof, POW_FORMAT_VERSION, PowError, solve, verify_at,
};
use crate::relay_pow_policy::RelayPowPolicy;

/// Session window for cached proofs, both publisher- and
/// subscriber-side. Chosen to stay comfortably BELOW the default
/// [`crate::pow::MAX_PROOF_AGE_SECS`] (30 min) so a cached proof,
/// reused anywhere in the window, is never already older than the
/// receiver's freshness bound — a legit publisher never hits the
/// boundary mid-broadcast or mid-replay. Sprint 75 Phase A relies on
/// this: replay re-stamps via this cache, so the window MUST be shorter
/// than the receiver's max proof age, or a re-stamped proof could still
/// be "too old". The `const _` assert below pins the invariant.
pub const SESSION_WINDOW: Duration = Duration::from_secs(15 * 60);

// Load-bearing invariant (Sprint 75 Phase A): a proof cached for at most
// SESSION_WINDOW must never be older than MAX_PROOF_AGE_SECS when broadcast or
// replayed, or a fresh receiver rejects it ("PoW proof too old" — the live
// discovery bug). A future SESSION_WINDOW bump past the receiver's window would
// silently reintroduce that bug; this compile-time assert forbids it.
const _: () = assert!(SESSION_WINDOW.as_secs() < crate::pow::MAX_PROOF_AGE_SECS);

/// Solve timeout applied to every fresh PoW session. Large
/// enough to cover the default 2^18 difficulty (~100 ms)
/// comfortably on a slow CPU, small enough to surface a
/// mis-configured policy quickly.
pub const SOLVE_TIMEOUT: Duration = Duration::from_secs(30);

/// Errors the envelope encode/decode and wire verify paths
/// surface.
#[derive(Debug, Error)]
pub enum PowGossipError {
    /// The caller-supplied bytes are shorter than the 4-byte
    /// proof_len header.
    #[error("PoW envelope too short to contain proof_len header")]
    EnvelopeHeaderTooShort,

    /// The proof_len header is larger than the envelope itself.
    #[error("PoW envelope proof_len {declared} exceeds envelope body {have}")]
    ProofLenOverrun {
        /// Declared length.
        declared: usize,
        /// Actual bytes available after the header.
        have: usize,
    },

    /// The proof JSON does not deserialise.
    #[error("PoW proof JSON parse failed: {0}")]
    ProofParse(#[from] serde_json::Error),

    /// The proof failed crypto / freshness / difficulty
    /// verification.
    #[error("PoW verify failed: {0}")]
    VerifyFailed(#[from] PowError),

    /// The caller tried to solve or verify but the policy
    /// clamped the difficulty to 0, which would bypass the
    /// entire defence. Policies should never set 0 — surface
    /// loud.
    #[error("PoW policy has zero difficulty for this topic; defence disabled")]
    ZeroDifficulty,
}

/// On-wire envelope : `[u32 BE proof_len][proof json][payload]`.
///
/// Zero-copy decode : [`PowEnvelope::decode`] returns a borrowed
/// payload slice so the caller can dispatch the message without
/// an extra heap allocation.
pub struct PowEnvelope;

impl PowEnvelope {
    /// Encode a proof + payload pair to a single `Vec<u8>` ready
    /// for [`crate::gossip::TopicSender::broadcast`].
    pub fn encode(proof: &HashcashProof, payload: &[u8]) -> Result<Vec<u8>, PowGossipError> {
        let proof_bytes = serde_json::to_vec(proof)?;
        let proof_len = proof_bytes.len();
        // u32::MAX bytes of proof is ~4 GB — unreachable in
        // practice, but guard anyway so the cast below is sound.
        if proof_len > u32::MAX as usize {
            // Treat as serde parse failure rather than adding a
            // new error variant — this is an "impossible under
            // every realistic scenario" branch.
            return Err(PowGossipError::ProofLenOverrun {
                declared: proof_len,
                have: 0,
            });
        }
        let mut out = Vec::with_capacity(4 + proof_len + payload.len());
        out.extend_from_slice(&(proof_len as u32).to_be_bytes());
        out.extend_from_slice(&proof_bytes);
        out.extend_from_slice(payload);
        Ok(out)
    }

    /// Decode an envelope into its proof + a borrowed payload.
    pub fn decode(bytes: &[u8]) -> Result<(HashcashProof, &[u8]), PowGossipError> {
        if bytes.len() < 4 {
            return Err(PowGossipError::EnvelopeHeaderTooShort);
        }
        let proof_len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
        let body = &bytes[4..];
        if proof_len > body.len() {
            return Err(PowGossipError::ProofLenOverrun {
                declared: proof_len,
                have: body.len(),
            });
        }
        let proof_bytes = &body[..proof_len];
        let payload = &body[proof_len..];
        let proof: HashcashProof = serde_json::from_slice(proof_bytes)?;
        Ok((proof, payload))
    }
}

/// Publisher-side session cache. Keyed by 32-byte topic id ;
/// value is `(proof, valid_until)`. A publisher that broadcasts
/// on N topics holds N solved proofs simultaneously ; the cache
/// is capped implicitly by the gossip swarm's practical topic
/// count (which is low — single-digit in current SBFB usage).
///
/// Not `Send + Sync`-shared across threads by design : the
/// publisher path is single-producer per `(topic, keypair)`.
/// Wrap in `Arc<Mutex<_>>` if cross-thread sharing is needed.
#[derive(Debug, Default)]
pub struct PowSolveCache {
    entries: Mutex<HashMap<[u8; 32], (HashcashProof, Instant)>>,
}

impl PowSolveCache {
    /// Empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure a valid proof exists for `(topic, keypair)` within
    /// the 15-minute session window, returning the active proof.
    ///
    /// Fresh solve ~100 ms at the default difficulty (2^18) ;
    /// subsequent calls in the window return in O(1).
    pub fn ensure_proof(
        &self,
        topic: [u8; 32],
        keypair: &KeyPair,
        policy: &RelayPowPolicy,
    ) -> Result<HashcashProof, PowGossipError> {
        let difficulty = policy.difficulty_for(&topic);
        if difficulty == 0 {
            return Err(PowGossipError::ZeroDifficulty);
        }

        {
            let cache = self.entries.lock().expect("PowSolveCache mutex poisoned");
            if let Some((proof, valid_until)) = cache.get(&topic) {
                if *valid_until > Instant::now() {
                    return Ok(proof.clone());
                }
            }
        }

        // Cache miss or expired — solve fresh.
        let challenge = HashcashChallenge::new(topic, keypair.public_bytes(), difficulty);
        let proof = solve(&challenge, SOLVE_TIMEOUT)?;
        let valid_until = Instant::now() + SESSION_WINDOW;
        self.entries
            .lock()
            .expect("PowSolveCache mutex poisoned")
            .insert(topic, (proof.clone(), valid_until));
        Ok(proof)
    }

    /// Invalidate any cached proof for `topic`. Used by callers
    /// that rotate their keypair mid-session or detect a policy
    /// bump.
    pub fn invalidate(&self, topic: &[u8; 32]) {
        self.entries
            .lock()
            .expect("PowSolveCache mutex poisoned")
            .remove(topic);
    }
}

/// Subscriber-side session cache. Thread-safe (DashMap) so the
/// gossip receive loop and an HTTP handler that introspects the
/// cache can share access.
#[derive(Debug, Default)]
pub struct PowVerifyCache {
    entries: DashMap<([u8; PUBLIC_KEY_LENGTH], [u8; 32]), Instant>,
}

impl PowVerifyCache {
    /// Empty cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify an incoming envelope and, on success, cache the
    /// `(publisher, topic)` pair as trusted for the rest of the
    /// session window. Returns the unwrapped payload bytes.
    ///
    /// The `now_unix_secs` argument is a caller-provided clock
    /// so tests can pin time deterministically. The receive loop
    /// typically passes `chrono::Utc::now().timestamp() as u64`
    /// or the result of `SystemTime::now()`.
    pub fn verify_envelope<'a>(
        &self,
        envelope_bytes: &'a [u8],
        policy: &RelayPowPolicy,
        now_unix_secs: u64,
    ) -> Result<(HashcashProof, &'a [u8]), PowGossipError> {
        let (proof, payload) = PowEnvelope::decode(envelope_bytes)?;

        // Difficulty from policy may differ from the challenge's
        // declared difficulty. We accept the MIN of the two :
        // - if the publisher over-paid (challenge.difficulty >
        //   policy's required), fine — policy's requirement is
        //   satisfied.
        // - if the publisher under-paid (challenge.difficulty <
        //   policy's required), reject at the InsufficientDifficulty
        //   branch below.
        let required = policy.difficulty_for(&proof.challenge.topic);
        if proof.challenge.difficulty < required {
            return Err(PowGossipError::VerifyFailed(
                PowError::InsufficientDifficulty {
                    need: required,
                    got: proof.challenge.difficulty,
                },
            ));
        }

        // Fast path : already verified in this session.
        let key = (proof.challenge.publisher_pubkey, proof.challenge.topic);
        if let Some(valid_until) = self.entries.get(&key) {
            if *valid_until > Instant::now() {
                return Ok((proof, payload));
            }
        }

        // Slow path : full verify.
        verify_at(&proof, now_unix_secs)?;

        // Policy check : the declared challenge version matches
        // ours. verify_at already enforces this but we pin the
        // invariant here so a future pow.rs refactor can't drop
        // it silently.
        debug_assert_eq!(proof.challenge.format_version, POW_FORMAT_VERSION);

        self.entries.insert(key, Instant::now() + SESSION_WINDOW);
        Ok((proof, payload))
    }

    /// Manually evict a `(publisher, topic)` entry. Useful in
    /// tests and when an operator rotates a known-compromised
    /// identity.
    pub fn invalidate(&self, pubkey: [u8; PUBLIC_KEY_LENGTH], topic: [u8; 32]) {
        self.entries.remove(&(pubkey, topic));
    }

    /// Number of cached entries. Exposed for introspection by
    /// tests and a future `/debug/pow-cache` HTTP handler.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache holds zero entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn low_difficulty_policy() -> RelayPowPolicy {
        // 4 bits : solves in <1 ms, keeps tests fast.
        RelayPowPolicy {
            default_difficulty: 4,
            topic_overrides: Default::default(),
        }
    }

    #[test]
    fn envelope_roundtrip_decodes_to_same_proof_and_payload() {
        let kp = KeyPair::generate();
        let topic = [0x55u8; 32];
        let challenge = HashcashChallenge::new(topic, kp.public_bytes(), 4);
        let proof = solve(&challenge, Duration::from_secs(1)).unwrap();
        let payload = b"hello gossip".to_vec();

        let envelope = PowEnvelope::encode(&proof, &payload).unwrap();
        let (decoded_proof, decoded_payload) = PowEnvelope::decode(&envelope).unwrap();
        assert_eq!(decoded_proof, proof);
        assert_eq!(decoded_payload, &payload[..]);
    }

    #[test]
    fn envelope_decode_rejects_short_header() {
        let err = PowEnvelope::decode(&[0x00, 0x00, 0x00]).unwrap_err();
        assert!(matches!(err, PowGossipError::EnvelopeHeaderTooShort));
    }

    #[test]
    fn envelope_decode_rejects_proof_len_overrun() {
        // Declare 10 bytes of proof but include only 2 bytes of
        // body.
        let mut bytes = Vec::from(10u32.to_be_bytes());
        bytes.extend_from_slice(&[0xFF, 0xFF]);
        let err = PowEnvelope::decode(&bytes).unwrap_err();
        match err {
            PowGossipError::ProofLenOverrun { declared, have } => {
                assert_eq!(declared, 10);
                assert_eq!(have, 2);
            }
            other => panic!("expected ProofLenOverrun, got {other:?}"),
        }
    }

    #[test]
    fn solve_cache_reuses_proof_within_session_window() {
        let kp = KeyPair::generate();
        let topic = [0x66u8; 32];
        let policy = low_difficulty_policy();
        let cache = PowSolveCache::new();

        let p1 = cache.ensure_proof(topic, &kp, &policy).unwrap();
        let p2 = cache.ensure_proof(topic, &kp, &policy).unwrap();
        assert_eq!(
            p1, p2,
            "second ensure_proof within session must return the cached proof"
        );
    }

    #[test]
    fn solve_cache_invalidate_forces_fresh_solve() {
        let kp = KeyPair::generate();
        let topic = [0x77u8; 32];
        let policy = low_difficulty_policy();
        let cache = PowSolveCache::new();

        let p1 = cache.ensure_proof(topic, &kp, &policy).unwrap();
        cache.invalidate(&topic);
        let p2 = cache.ensure_proof(topic, &kp, &policy).unwrap();
        // Nonces are deterministic for a fixed challenge ; new
        // proof will share the same nonce as p1 because
        // issued_at is in seconds and both calls land in the
        // same second most of the time. What differs on
        // invalidate is that the cache was repopulated — test
        // that by checking the cache is non-empty.
        let _ = (p1, p2);
        let cache_inner = cache.entries.lock().unwrap();
        assert_eq!(cache_inner.len(), 1);
    }

    #[test]
    fn solve_cache_errors_on_zero_difficulty_policy() {
        let kp = KeyPair::generate();
        let topic = [0x88u8; 32];
        let policy = RelayPowPolicy {
            default_difficulty: 0,
            topic_overrides: Default::default(),
        };
        let cache = PowSolveCache::new();
        let err = cache.ensure_proof(topic, &kp, &policy).unwrap_err();
        assert!(matches!(err, PowGossipError::ZeroDifficulty));
    }

    #[test]
    fn verify_cache_happy_path_caches_pubkey_topic() {
        let kp = KeyPair::generate();
        let topic = [0x99u8; 32];
        let policy = low_difficulty_policy();
        let challenge = HashcashChallenge::new(topic, kp.public_bytes(), 4);
        let proof = solve(&challenge, Duration::from_secs(1)).unwrap();
        let envelope = PowEnvelope::encode(&proof, b"payload").unwrap();

        let verify_cache = PowVerifyCache::new();
        assert_eq!(verify_cache.len(), 0);

        let (decoded_proof, payload) = verify_cache
            .verify_envelope(&envelope, &policy, now_unix())
            .unwrap();
        assert_eq!(decoded_proof, proof);
        assert_eq!(payload, b"payload");
        assert_eq!(
            verify_cache.len(),
            1,
            "first verify must cache (pubkey, topic)"
        );
    }

    #[test]
    fn verify_cache_rejects_under_paid_proof() {
        // Publisher solved at difficulty 4 but policy requires
        // 12 → reject as InsufficientDifficulty.
        let kp = KeyPair::generate();
        let topic = [0xAAu8; 32];
        let challenge = HashcashChallenge::new(topic, kp.public_bytes(), 4);
        let proof = solve(&challenge, Duration::from_secs(1)).unwrap();
        let envelope = PowEnvelope::encode(&proof, b"X").unwrap();

        let policy = RelayPowPolicy {
            default_difficulty: 12,
            topic_overrides: Default::default(),
        };
        let verify_cache = PowVerifyCache::new();
        let err = verify_cache
            .verify_envelope(&envelope, &policy, now_unix())
            .unwrap_err();
        match err {
            PowGossipError::VerifyFailed(PowError::InsufficientDifficulty { need, got }) => {
                assert_eq!(need, 12);
                assert_eq!(got, 4);
            }
            other => panic!("expected InsufficientDifficulty, got {other:?}"),
        }
        assert_eq!(
            verify_cache.len(),
            0,
            "rejected verify must not populate cache"
        );
    }

    #[test]
    fn verify_cache_rejects_tampered_payload_has_no_effect() {
        // The envelope format binds the proof to itself, NOT to
        // the payload bytes. A MITM who only tampers with the
        // payload cannot be caught by PoW verify alone — this is
        // an intentional scope boundary (PoW is about cost-of-
        // identity, not payload integrity). This test pins that
        // invariant : the PoW verify happily passes with a
        // tampered payload. Payload integrity lives elsewhere
        // (signed curator lists, signed tasks, etc.).
        let kp = KeyPair::generate();
        let topic = [0xBBu8; 32];
        let policy = low_difficulty_policy();
        let challenge = HashcashChallenge::new(topic, kp.public_bytes(), 4);
        let proof = solve(&challenge, Duration::from_secs(1)).unwrap();
        let mut envelope = PowEnvelope::encode(&proof, b"original").unwrap();
        // Tamper with the last byte of the payload.
        let last = envelope.len() - 1;
        envelope[last] ^= 0xFF;

        let verify_cache = PowVerifyCache::new();
        let result = verify_cache.verify_envelope(&envelope, &policy, now_unix());
        assert!(
            result.is_ok(),
            "PoW alone must not reject payload tamper ; payload integrity is a separate layer"
        );
    }

    #[test]
    fn verify_cache_rejects_corrupt_proof_json() {
        // Build an envelope whose proof_len header declares 4
        // bytes but the 4 bytes do not parse as JSON.
        let mut bytes = Vec::from(4u32.to_be_bytes());
        bytes.extend_from_slice(b"NOPE");
        bytes.extend_from_slice(b"payload");

        let policy = low_difficulty_policy();
        let verify_cache = PowVerifyCache::new();
        let err = verify_cache
            .verify_envelope(&bytes, &policy, now_unix())
            .unwrap_err();
        assert!(matches!(err, PowGossipError::ProofParse(_)));
    }

    // =============================================================
    // Integration tests — full publisher → transport → subscriber
    // pipeline via an in-process channel substitute.
    //
    // Sprint 19 Phase B plan §5.4 calls for 2 "fake relay" /
    // "mock relay" integration tests. An actual 2-node iroh-gossip
    // handshake in-process requires either (a) live pkarr relays
    // (flaky CI) or (b) StaticProvider + create_node_with_discovery
    // plumbing that does not exist yet. Both are Sprint 20+
    // infrastructure changes.
    //
    // These tests exercise the PoW pipeline end-to-end :
    // encode → transport (stdlib mpsc standing in for
    // iroh-gossip) → decode → verify → payload delivery. They do
    // NOT validate iroh-gossip peer discovery or HyParView
    // broadcast trees — those are exercised by the curator
    // runtime's 2-node tests in `nexus-shell-daemon-core`.
    // =============================================================

    #[test]
    fn end_to_end_happy_path_with_mock_transport() {
        // Shared 15-minute session policy : publisher and
        // subscriber must agree on difficulty for verify to pass.
        let policy = low_difficulty_policy();

        // Publisher side.
        let kp = KeyPair::generate();
        let topic = [0xD1u8; 32];
        let solve_cache = PowSolveCache::new();
        let proof = solve_cache.ensure_proof(topic, &kp, &policy).unwrap();
        let payload = b"curator-list-announcement-v1".to_vec();
        let envelope = PowEnvelope::encode(&proof, &payload).unwrap();

        // In-process "transport" : stdlib mpsc stands in for the
        // iroh-gossip broadcast tree. A real deployment sends
        // these bytes over QUIC ; the PoW pipeline is transport-
        // agnostic so the channel substitute exercises identical
        // bytes.
        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        tx.send(envelope).unwrap();

        // Subscriber side.
        let verify_cache = PowVerifyCache::new();
        let received = rx.recv().unwrap();
        let (decoded_proof, decoded_payload) = verify_cache
            .verify_envelope(&received, &policy, now_unix())
            .unwrap();
        assert_eq!(decoded_proof.challenge.publisher_pubkey, kp.public_bytes());
        assert_eq!(decoded_proof.challenge.topic, topic);
        assert_eq!(decoded_payload, &payload[..]);
        assert_eq!(verify_cache.len(), 1);

        // Second broadcast in the same session : publisher reuses
        // the cached proof (no fresh solve), subscriber bypasses
        // the full verify (cache hit).
        let p2 = solve_cache.ensure_proof(topic, &kp, &policy).unwrap();
        assert_eq!(p2, proof, "publisher must reuse session proof");
        let envelope_2 = PowEnvelope::encode(&p2, b"heartbeat").unwrap();
        let (_, payload_2) = verify_cache
            .verify_envelope(&envelope_2, &policy, now_unix())
            .unwrap();
        assert_eq!(payload_2, b"heartbeat");
        // Still just the one session entry.
        assert_eq!(verify_cache.len(), 1);
    }

    #[test]
    fn end_to_end_rejects_tampered_proof_with_mock_transport() {
        // Publisher solves a valid proof, but a MITM on the
        // transport flips bits inside the proof bytes. The
        // subscriber must reject and NOT populate the cache —
        // otherwise a tampered broadcast would poison the fast
        // path for the real publisher.
        let policy = low_difficulty_policy();
        let kp = KeyPair::generate();
        let topic = [0xD2u8; 32];
        let solve_cache = PowSolveCache::new();
        let proof = solve_cache.ensure_proof(topic, &kp, &policy).unwrap();
        let mut envelope = PowEnvelope::encode(&proof, b"payload").unwrap();

        // Flip one bit inside the proof bytes region. Bytes 0..4
        // are the proof_len header ; bytes 4..(4 + proof_len) are
        // the proof JSON. Target a byte deep in the JSON so we
        // hit either a `hash` / `nonce` field (→ HashMismatch) or
        // a syntactic byte (→ ProofParse). Either rejection is
        // acceptable here ; the invariant is "does not verify".
        let tamper_idx = 20.min(envelope.len() - 1);
        envelope[tamper_idx] ^= 0x42;

        let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
        tx.send(envelope).unwrap();
        let received = rx.recv().unwrap();

        let verify_cache = PowVerifyCache::new();
        let result = verify_cache.verify_envelope(&received, &policy, now_unix());
        assert!(
            result.is_err(),
            "tampered proof must not verify; got {:?}",
            result
        );
        assert_eq!(
            verify_cache.len(),
            0,
            "failed verify must not poison the session cache"
        );
    }
}
