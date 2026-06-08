// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-node seed protocol payloads (Sprint 74 Phase E).
//!
//! A *seed* is a peer that replicates a public app's archive blob so
//! the app stays reachable even when the author's node is offline. The
//! seeder is **not** a co-author: it re-announces the author-signed
//! `archive_hash` (content-addressed by BLAKE3) and never signs any
//! provenance. This is the Radicle invariant "seeder ≠ delegate".
//!
//! This module carries the **pure signed payloads** exchanged over the
//! `sbfb/seed/0` ALPN — exactly like [`crate::task`], it touches no
//! iroh, tokio, or DB; the daemon-side handler ([`nexus-shell-daemon`]
//! `seed_protocol`) composes these with the transport and storage.
//!
//! ## Direction (topology T-b, preflight NF-4)
//!
//! The author's node (the requester) dials a chosen peer (the seeder)
//! and sends a [`SeedRequestEnvelope`] saying "please keep MY app
//! online". The seeder verifies, fetches+pins the blob, and replies
//! with a signed [`SeedResponseEnvelope`]. The symmetric case — a
//! trusted peer enrolling itself as a seeder of someone else's app via
//! a revocable invite token — reuses the same envelopes with a non-empty
//! `invite_token`.
//!
//! The **voluntary community seed** path (any node helping keep a public
//! app online) does NOT use these envelopes at all: it is a unilateral
//! local act on already-public content (fetch+pin a distant blob), and
//! lands on the Phase D pin + Phase F `SeedAnnounced` primitives.
//!
//! ## Signature contract
//!
//! Identical to [`crate::task::TaskEntry`] / `ClaimEntry`: the payload
//! plus a 32-byte `author_pubkey` and a 64-byte Ed25519 `signature` over
//! [`canonical_bytes`] of the payload with a dedicated domain tag. The
//! `signature` and `author_pubkey` live on the *envelope* and are NEVER
//! part of the canonical bytes.

use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

use crate::canonical::{DOMAIN_SEED_REQUEST_V1, DOMAIN_SEED_RESPONSE_V1, canonical_bytes};
use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};
use crate::error::{NexusError, Result};

/// On-wire version for the seed protocol payloads.
///
/// Pre-launch policy: stays at 1 until the tagged v1.0 freeze (cf.
/// CLAUDE.md "Pre-launch protocol protocol policy"). The ALPN string
/// `sbfb/seed/0` carries the protocol generation; this field is the
/// fine-grained payload version under that ALPN.
pub const SEED_FORMAT_VERSION: u16 = 1;

/// Length in bytes of the anti-replay nonce.
pub const SEED_NONCE_LEN: usize = 32;

/// Freshness window (seconds) a seeder tolerates between `SeedRequest.ts`
/// and its own clock. A live req/resp is not an append-only log, so the
/// window is tight (2 minutes) — enough for reasonable clock skew while
/// bounding the replay window. (Do NOT reuse the feed's 30-day future
/// gate; that is for a historical log.)
pub const SEED_TS_WINDOW_SECS: u64 = 120;

/// Generate a fresh 32-byte random anti-replay nonce via the OS RNG.
///
/// Mirrors [`crate::crypto::KeyPair::generate`]'s use of `OsRng`.
pub fn random_nonce() -> Vec<u8> {
    use rand::RngCore;
    use rand::rngs::OsRng;
    let mut nonce = vec![0u8; SEED_NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// A request to a specific peer asking it to seed (fetch + pin +
/// re-announce) a specific public app archive. Sent over ALPN
/// `sbfb/seed/0`.
///
/// Every field here contributes to the canonical bytes the requester
/// signs; nothing outside this struct (i.e. the envelope's
/// `author_pubkey` / `signature`) is covered by the signature.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRequest {
    /// Must equal [`SEED_FORMAT_VERSION`] to be accepted by this build.
    pub version: u16,

    /// `blake3(name)`-derived per-app id of the app to seed.
    pub project_id: String,

    /// Hex-encoded (64 chars) BLAKE3 archive hash — the exact content
    /// the seeder will fetch + pin. Content-addressing guarantees the
    /// seeder can only ever end up with these exact bytes (R5 / §13).
    pub archive_hash: String,

    /// `BlobTicket` string (provider `EndpointAddr` + hash + format) so
    /// the seeder can dial the source without pkarr. Non-empty.
    pub archive_ticket: String,

    /// Ed25519 public key of the node that built + signed this request.
    /// Cross-checked against the QUIC-authenticated dialer id
    /// (`conn.remote_id()`) AND the envelope's `author_pubkey`.
    pub requester_node_id: [u8; PUBLIC_KEY_LENGTH],

    /// 32-byte random anti-replay nonce (`OsRng`). Serialized as a JSON
    /// array of numbers under JCS (deterministic).
    pub nonce: Vec<u8>,

    /// Unix seconds when the request was minted. Freshness-gated by the
    /// seeder ([`SEED_TS_WINDOW_SECS`]).
    pub ts: u64,

    /// Revocable invite token (an opaque id indexing the receiver's
    /// local `seed_invite` ledger, Tailscale model). Empty when the
    /// requester IS the app's own node (self-designation of its own
    /// second machine, same node key) — and empty for the voluntary
    /// path, which does not use `SeedRequest` at all.
    pub invite_token: String,
}

/// A signed [`SeedRequest`], ready to send on the wire.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedRequestEnvelope {
    /// The request payload.
    pub request: SeedRequest,
    /// Ed25519 public key of the requester. Must equal
    /// `request.requester_node_id`.
    pub author_pubkey: [u8; PUBLIC_KEY_LENGTH],
    /// Ed25519 signature over `canonical_bytes(request,
    /// DOMAIN_SEED_REQUEST_V1)`.
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl SeedRequestEnvelope {
    /// Sign a [`SeedRequest`] with the requester keypair.
    pub fn sign(request: SeedRequest, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&request, DOMAIN_SEED_REQUEST_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(SeedRequestEnvelope {
            request,
            author_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the signature AND the attribution consistency
    /// (`request.requester_node_id == author_pubkey`). Mirrors
    /// [`crate::task::ClaimEntry::verify_signature`].
    ///
    /// Returns [`NexusError::Crypto`] on any failure (attribution
    /// mismatch, tampered payload, wrong key).
    pub fn verify_signature(&self) -> Result<()> {
        if self.request.requester_node_id != self.author_pubkey {
            return Err(NexusError::Crypto(
                "seed request: requester_node_id does not match author_pubkey".into(),
            ));
        }
        let bytes = canonical_bytes(&self.request, DOMAIN_SEED_REQUEST_V1)?;
        crate::crypto::verify(&self.author_pubkey, &bytes, &self.signature)
    }
}

/// The seeder's decision on a [`SeedRequest`].
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SeedDecision {
    /// The seeder accepted and has fetched + pinned the blob.
    Accepted,
    /// The seeder rejected; see [`SeedResponse::reason`].
    Rejected,
}

/// A seeder's reply to a [`SeedRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedResponse {
    /// Equal to [`SEED_FORMAT_VERSION`].
    pub version: u16,
    /// Echo of the request's `project_id` for correlation.
    pub project_id: String,
    /// Echo of the request's `nonce` for correlation / anti-confusion.
    pub nonce: Vec<u8>,
    /// Accept or reject.
    pub decision: SeedDecision,
    /// Short machine reason code on `Rejected` (e.g. `"bad-sig"`,
    /// `"stale-ts"`, `"replay"`, `"no-invite"`, `"invite-revoked"`,
    /// `"invite-expired"`, `"invite-exhausted"`, `"not-approved"`,
    /// `"fetch-failed"`, `"bad-request"`). Empty on `Accepted`.
    pub reason: String,
    /// Unix seconds when the seeder produced this response.
    pub ts: u64,
}

impl SeedResponse {
    /// Build an `Accepted` response echoing the request's correlation
    /// fields.
    pub fn accepted(project_id: impl Into<String>, nonce: Vec<u8>, ts: u64) -> Self {
        SeedResponse {
            version: SEED_FORMAT_VERSION,
            project_id: project_id.into(),
            nonce,
            decision: SeedDecision::Accepted,
            reason: String::new(),
            ts,
        }
    }

    /// Build a `Rejected` response with a short reason code.
    pub fn rejected(
        project_id: impl Into<String>,
        nonce: Vec<u8>,
        reason: impl Into<String>,
        ts: u64,
    ) -> Self {
        SeedResponse {
            version: SEED_FORMAT_VERSION,
            project_id: project_id.into(),
            nonce,
            decision: SeedDecision::Rejected,
            reason: reason.into(),
            ts,
        }
    }
}

/// A signed [`SeedResponse`], signed by the SEEDER's node key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SeedResponseEnvelope {
    /// The response payload.
    pub response: SeedResponse,
    /// Ed25519 public key of the seeder.
    pub author_pubkey: [u8; PUBLIC_KEY_LENGTH],
    /// Ed25519 signature over `canonical_bytes(response,
    /// DOMAIN_SEED_RESPONSE_V1)`.
    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}

impl SeedResponseEnvelope {
    /// Sign a [`SeedResponse`] with the seeder keypair.
    pub fn sign(response: SeedResponse, keypair: &KeyPair) -> Result<Self> {
        let bytes = canonical_bytes(&response, DOMAIN_SEED_RESPONSE_V1)?;
        let signature = keypair.sign(&bytes);
        Ok(SeedResponseEnvelope {
            response,
            author_pubkey: keypair.public_bytes(),
            signature,
        })
    }

    /// Verify the seeder's signature over the response. The caller
    /// should additionally cross-check `author_pubkey` against the
    /// QUIC-authenticated peer it dialed.
    pub fn verify_signature(&self) -> Result<()> {
        let bytes = canonical_bytes(&self.response, DOMAIN_SEED_RESPONSE_V1)?;
        crate::crypto::verify(&self.author_pubkey, &bytes, &self.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request(requester: &KeyPair) -> SeedRequest {
        SeedRequest {
            version: SEED_FORMAT_VERSION,
            project_id: "a".repeat(64),
            archive_hash: "b".repeat(64),
            archive_ticket: "blobacafake-ticket".into(),
            requester_node_id: requester.public_bytes(),
            nonce: random_nonce(),
            ts: 1_700_000_000,
            invite_token: String::new(),
        }
    }

    #[test]
    fn request_sign_then_verify_roundtrip() {
        let kp = KeyPair::generate();
        let env = SeedRequestEnvelope::sign(sample_request(&kp), &kp).unwrap();
        env.verify_signature().expect("freshly signed must verify");
        assert_eq!(env.author_pubkey, kp.public_bytes());
        assert_eq!(env.request.version, SEED_FORMAT_VERSION);
    }

    #[test]
    fn request_verify_rejects_tampered_payload() {
        let kp = KeyPair::generate();
        let mut env = SeedRequestEnvelope::sign(sample_request(&kp), &kp).unwrap();
        env.request.archive_hash = "c".repeat(64);
        assert!(
            env.verify_signature().is_err(),
            "tampering with the signed payload must fail verification"
        );
    }

    #[test]
    fn request_verify_rejects_tampered_signature() {
        let kp = KeyPair::generate();
        let mut env = SeedRequestEnvelope::sign(sample_request(&kp), &kp).unwrap();
        env.signature[0] ^= 0xFF;
        assert!(env.verify_signature().is_err());
    }

    #[test]
    fn request_verify_rejects_attribution_mismatch() {
        // A valid signature over the payload, but the embedded
        // requester_node_id points at a different key than author_pubkey.
        let kp = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut req = sample_request(&kp);
        req.requester_node_id = impostor.public_bytes();
        let mut env = SeedRequestEnvelope::sign(req, &kp).unwrap();
        // sign() set author_pubkey == kp; requester_node_id == impostor.
        // The attribution check must fire BEFORE the crypto check.
        assert!(
            env.verify_signature().is_err(),
            "requester_node_id != author_pubkey must be rejected"
        );
        // Make author_pubkey match requester_node_id (impostor) but keep
        // kp's signature: now the crypto check fires.
        env.author_pubkey = impostor.public_bytes();
        assert!(env.verify_signature().is_err());
    }

    #[test]
    fn request_verify_rejects_wrong_issuer() {
        let real = KeyPair::generate();
        let impostor = KeyPair::generate();
        let mut env = SeedRequestEnvelope::sign(sample_request(&real), &real).unwrap();
        // Swap both the embedded id AND the envelope key to the impostor;
        // the signature is still real's, so crypto verify fails.
        env.request.requester_node_id = impostor.public_bytes();
        env.author_pubkey = impostor.public_bytes();
        assert!(env.verify_signature().is_err());
    }

    #[test]
    fn response_sign_then_verify_roundtrip() {
        let seeder = KeyPair::generate();
        let resp = SeedResponse::accepted("a".repeat(64), random_nonce(), 1_700_000_001);
        let env = SeedResponseEnvelope::sign(resp, &seeder).unwrap();
        env.verify_signature()
            .expect("freshly signed response verifies");
        assert_eq!(env.author_pubkey, seeder.public_bytes());
        assert_eq!(env.response.decision, SeedDecision::Accepted);
        assert!(env.response.reason.is_empty());
    }

    #[test]
    fn response_rejected_carries_reason() {
        let seeder = KeyPair::generate();
        let resp = SeedResponse::rejected("a".repeat(64), random_nonce(), "replay", 42);
        let env = SeedResponseEnvelope::sign(resp, &seeder).unwrap();
        env.verify_signature().unwrap();
        assert_eq!(env.response.decision, SeedDecision::Rejected);
        assert_eq!(env.response.reason, "replay");
    }

    #[test]
    fn request_and_response_domains_are_disjoint() {
        // A request signature must NOT verify as a response and vice
        // versa — the domain separation prevents cross-replay. We prove
        // it by signing the SAME logical bytes under each envelope: the
        // canonical byte strings differ by domain prefix.
        let kp = KeyPair::generate();
        let req = sample_request(&kp);
        let req_bytes = canonical_bytes(&req, DOMAIN_SEED_REQUEST_V1).unwrap();
        let as_response = canonical_bytes(&req, DOMAIN_SEED_RESPONSE_V1).unwrap();
        assert_ne!(
            req_bytes, as_response,
            "request and response domains must produce distinct byte strings"
        );
    }

    #[test]
    fn random_nonce_is_32_bytes_and_distinct() {
        let a = random_nonce();
        let b = random_nonce();
        assert_eq!(a.len(), SEED_NONCE_LEN);
        assert_eq!(b.len(), SEED_NONCE_LEN);
        assert_ne!(a, b, "two fresh nonces must (overwhelmingly) differ");
    }

    #[test]
    fn envelope_json_roundtrips() {
        let kp = KeyPair::generate();
        let env = SeedRequestEnvelope::sign(sample_request(&kp), &kp).unwrap();
        let json = serde_json::to_vec(&env).unwrap();
        let back: SeedRequestEnvelope = serde_json::from_slice(&json).unwrap();
        assert_eq!(back, env);
        back.verify_signature().unwrap();
    }
}
