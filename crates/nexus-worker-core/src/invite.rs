// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB invite-token format and verification.
//!
//! An "invite" is a signed capability token the coordinator of
//! a project hands out to contributors it wants to recruit. The
//! worker receives a single text string like
//!
//! ```text
//! nx1AAAAZMLKI2TFOJQWG5DPN5XHGIDBMZRWKY3XMQXDCOJVHE......
//! ```
//!
//! runs `nexus-worker join <invite>`, and if the signature is
//! valid and the token has not expired, the project is added
//! to the allowlist (W7).
//!
//! ## Contents
//!
//! Every invite carries exactly what the worker needs to
//! enroll in the project and, later, dial the coordinator
//! without going through pkarr:
//!
//! - `version` — wire-format version, starts at 1
//! - `project_id` — iroh document namespace id as a string
//! - `project_name` — human-readable name for the CLI
//! - `coordinator_pubkey` — 32-byte Ed25519 public key, also the
//!   issuer of the signature
//! - `coordinator_addr` — optional compact endpoint addr string
//!   (relay URL or direct addrs) minted by the coordinator side
//! - `scope` — permissions the token grants
//! - `expires_at_unix` — token expiry in unix seconds
//!
//! The whole payload is serialized to canonical JSON, signed
//! with the coordinator's Ed25519 secret key, and wrapped in
//! an [`Invite`] struct alongside the signature.
//!
//! ## Encoding
//!
//! The wire format is `nx1` + Base32 (no padding, alphabet
//! RFC4648) of the JSON-encoded [`Invite`]. Base32 was picked
//! over hex (more compact, ~1.6× denser) and over Base64 (not
//! URL-safe without `_`/`-` which break when pasted in a
//! terminal that helpfully strips trailing `=`). The `nx1`
//! prefix lets the CLI detect and reject random paste
//! garbage before even trying to decode.
//!
//! ## Verification
//!
//! [`Invite::decode`] parses the wire format, validates the
//! signature against `payload.coordinator_pubkey`, and returns
//! the typed struct. [`Invite::ensure_not_expired(now)`] is a
//! separate step so the CLI can show a helpful "this invite
//! expired X days ago" message instead of generic
//! `decode_failed`.

use std::time::{SystemTime, UNIX_EPOCH};

use data_encoding::BASE32_NOPAD;
use nexus_core_rs::canonical::{DOMAIN_INVITE_V1, canonical_bytes};
use nexus_core_rs::{KeyPair, verify};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Wire-format version. Bumped whenever [`InvitePayload`]
/// changes in a way that breaks backwards compatibility. Old
/// workers must refuse to decode a newer version rather than
/// guess.
///
/// Version 2 (Sprint 4 Phase C) adds `tasks_doc_ticket` so the
/// worker can import the coordinator's task doc during `join`
/// without a separate out-of-band ticket. Sprint 3 version 1
/// invites are rejected (the Sprint 4 kickoff §2 decision C is a
/// hard bump — no v1 was ever distributed).
pub const INVITE_FORMAT_VERSION: u16 = 2;

/// Human-readable prefix prepended to every invite string.
/// Chosen so a paste-capture in a chat window is obviously a
/// nexus-grid invite without needing a decoder.
pub const INVITE_PREFIX: &str = "nx1";

// =================================================================
// Scope
// =================================================================

/// Permission level granted by an invite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InviteScope {
    /// Full worker access: the holder can claim and execute
    /// tasks, and gets credited kudos for completed work. This
    /// is the default and what `nexus-worker join` uses.
    Worker,
    /// Read-only: the holder can observe project state but
    /// cannot claim tasks. Planned for v1.1 when the
    /// spectator / dashboard UX lands.
    Observer,
}

impl InviteScope {
    /// True iff this scope allows the holder to claim and
    /// execute compute tasks. Observer → false.
    pub fn can_serve_tasks(&self) -> bool {
        matches!(self, Self::Worker)
    }
}

// =================================================================
// Payload and envelope
// =================================================================

/// The signed part of an invite. Every field here contributes
/// to the canonical bytes that the coordinator signs; nothing
/// outside this struct is covered by the signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitePayload {
    /// Wire-format version. Always equal to [`INVITE_FORMAT_VERSION`]
    /// at mint time.
    pub version: u16,
    /// Iroh document namespace id the worker will enroll into.
    /// Stored as the opaque string form coordinators use.
    pub project_id: String,
    /// Human-readable project name, for the `join` CLI
    /// confirmation prompt and the `projects list` output.
    pub project_name: String,
    /// Coordinator's Ed25519 public key — the issuer of the
    /// signature. The worker validates `signature` against
    /// this field so a tampered token cannot change the
    /// issuer.
    #[serde(with = "hex_bytes32")]
    pub coordinator_pubkey: [u8; 32],
    /// Optional compact form of the coordinator's EndpointAddr
    /// (relay URL or direct address). W9 uses it to seed the
    /// node's MemoryLookup so the worker can dial without
    /// pkarr discovery on first contact. `None` means the
    /// worker must resolve the coordinator through pkarr.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinator_addr: Option<String>,
    /// **v2 field**: serialized iroh-docs write ticket for the
    /// project's tasks doc. The worker imports the doc on `join`
    /// via `DocsClient::import_and_subscribe(ticket)` so it can
    /// scan `task:*` / `claim:*` / `result:*` entries without a
    /// separate out-of-band exchange.
    ///
    /// Mandatory when `scope == Worker` (the `Invite::mint`
    /// constructor enforces this); optional for `scope ==
    /// Observer`, where a read-only worker can still function
    /// without the ticket.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks_doc_ticket: Option<String>,
    /// Permission level granted by this invite.
    pub scope: InviteScope,
    /// Token expiry in unix seconds. Workers reject tokens
    /// where `now >= expires_at_unix`.
    pub expires_at_unix: u64,
}

impl InvitePayload {
    /// Serialize the payload to the canonical bytes used as the
    /// signature input.
    ///
    /// Delegates to [`nexus_core_rs::canonical::canonical_bytes`]
    /// with the [`DOMAIN_INVITE_V1`] domain tag, which gives us
    /// RFC 8785 JCS output plus type-level separation from tasks,
    /// results and claims. The call is infallible for a
    /// well-formed `InvitePayload` (no NaN/Inf floats, no
    /// non-string map keys), so we unwrap with an `expect` that
    /// can only fire on a memory exhaustion or similar hard
    /// failure in `serde_jcs`.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        canonical_bytes(self, DOMAIN_INVITE_V1)
            .expect("InvitePayload must be infallibly JCS-serializable")
    }
}

/// Full invite: payload + signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invite {
    pub payload: InvitePayload,
    /// Ed25519 signature over [`InvitePayload::canonical_bytes`].
    #[serde(with = "hex_bytes64")]
    pub signature: [u8; 64],
}

// =================================================================
// Errors
// =================================================================

/// Failures the invite module can produce.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InviteError {
    /// The wire-format prefix was missing or wrong. Callers can
    /// use this to distinguish "pasted garbage" from
    /// "malformed invite".
    #[error("invite is missing the {INVITE_PREFIX} prefix")]
    BadPrefix,

    /// Base32 decoding failed.
    #[error("invite base32 decoding failed: {0}")]
    BadBase32(String),

    /// JSON parsing of the decoded bytes failed.
    #[error("invite json decoding failed: {0}")]
    BadJson(String),

    /// The `version` field in the decoded invite is newer than
    /// the worker knows how to handle.
    #[error(
        "invite has unsupported version {0} (this worker knows version {INVITE_FORMAT_VERSION})"
    )]
    UnsupportedVersion(u16),

    /// The Ed25519 signature did not verify against the
    /// coordinator public key inside the invite.
    #[error("invite signature verification failed: {0}")]
    BadSignature(String),

    /// The token's `expires_at_unix` is in the past.
    #[error("invite expired at unix {expired_at} (now {now})")]
    Expired { expired_at: u64, now: u64 },

    /// Invite v2 requires `tasks_doc_ticket` when
    /// `scope == Worker`, but the caller left it `None`. Produced
    /// at mint time; the decoder also refuses such a payload
    /// because it would be impossible for a worker to act on.
    #[error("Worker-scoped invite must carry a tasks_doc_ticket")]
    MissingTasksDocTicket,
}

// =================================================================
// Mint / encode / decode / verify
// =================================================================

impl Invite {
    /// Build and sign a new invite with the supplied coordinator
    /// keypair. Automatically fills `version`,
    /// `coordinator_pubkey`, and signs the canonical bytes.
    ///
    /// Returns [`InviteError::MissingTasksDocTicket`] if
    /// `scope == Worker && tasks_doc_ticket.is_none()` — a
    /// Worker-scoped invite is useless without the ticket and we
    /// refuse to mint one in that state.
    pub fn mint(
        coordinator: &KeyPair,
        project_id: impl Into<String>,
        project_name: impl Into<String>,
        coordinator_addr: Option<String>,
        tasks_doc_ticket: Option<String>,
        scope: InviteScope,
        expires_at_unix: u64,
    ) -> Result<Self, InviteError> {
        if matches!(scope, InviteScope::Worker) && tasks_doc_ticket.is_none() {
            return Err(InviteError::MissingTasksDocTicket);
        }
        let payload = InvitePayload {
            version: INVITE_FORMAT_VERSION,
            project_id: project_id.into(),
            project_name: project_name.into(),
            coordinator_pubkey: coordinator.public_bytes(),
            coordinator_addr,
            tasks_doc_ticket,
            scope,
            expires_at_unix,
        };
        let signature = coordinator.sign(&payload.canonical_bytes());
        Ok(Self { payload, signature })
    }

    /// Serialize to the wire format: `nx1` + Base32 of the
    /// JSON-encoded invite.
    pub fn encode(&self) -> String {
        let json =
            serde_json::to_vec(self).expect("Invite must be infallibly serializable for encoding");
        let body = BASE32_NOPAD.encode(&json);
        format!("{INVITE_PREFIX}{body}")
    }

    /// Parse a wire-format invite and verify its signature.
    ///
    /// On success the returned [`Invite`] is guaranteed to
    /// carry a valid signature from
    /// `payload.coordinator_pubkey`. Callers should still call
    /// [`Invite::ensure_not_expired`] before acting on it.
    pub fn decode(wire: &str) -> Result<Self, InviteError> {
        let body = wire
            .strip_prefix(INVITE_PREFIX)
            .ok_or(InviteError::BadPrefix)?;

        // Uppercase to be forgiving: base32 alphabets are
        // case-insensitive but `data_encoding`'s NOPAD codec
        // only accepts uppercase.
        let upper = body.to_ascii_uppercase();
        let bytes = BASE32_NOPAD
            .decode(upper.as_bytes())
            .map_err(|e| InviteError::BadBase32(e.to_string()))?;

        let invite: Invite =
            serde_json::from_slice(&bytes).map_err(|e| InviteError::BadJson(e.to_string()))?;

        if invite.payload.version != INVITE_FORMAT_VERSION {
            return Err(InviteError::UnsupportedVersion(invite.payload.version));
        }

        // Enforce the Worker-requires-ticket rule on decode too,
        // so a hand-crafted payload that bypassed `mint` still
        // fails closed.
        if matches!(invite.payload.scope, InviteScope::Worker)
            && invite.payload.tasks_doc_ticket.is_none()
        {
            return Err(InviteError::MissingTasksDocTicket);
        }

        invite.verify_signature()?;
        Ok(invite)
    }

    /// Verify the signature against the embedded coordinator
    /// public key. Called automatically by [`Invite::decode`];
    /// exposed separately for callers that build an [`Invite`]
    /// by hand in tests.
    pub fn verify_signature(&self) -> Result<(), InviteError> {
        verify(
            &self.payload.coordinator_pubkey,
            &self.payload.canonical_bytes(),
            &self.signature,
        )
        .map_err(|e| InviteError::BadSignature(e.to_string()))
    }

    /// Return `Err(Expired)` when the provided `now_unix` is at
    /// or after the token expiry. Otherwise returns `Ok(())`.
    pub fn ensure_not_expired(&self, now_unix: u64) -> Result<(), InviteError> {
        if now_unix >= self.payload.expires_at_unix {
            Err(InviteError::Expired {
                expired_at: self.payload.expires_at_unix,
                now: now_unix,
            })
        } else {
            Ok(())
        }
    }
}

/// Return the current unix timestamp as `u64`. Clock skew
/// tolerance is left to the caller — the CLI uses the system
/// clock, tests use a fixed value.
pub fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// =================================================================
// Hex (de)serialization helpers for the signature + pubkey
// =================================================================

/// Serde helper: [u8; 32] as hex string.
mod hex_bytes32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = <String as Deserialize>::deserialize(d)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        let arr: [u8; 32] = bytes.try_into().map_err(|v: Vec<u8>| {
            serde::de::Error::custom(format!("expected 32 bytes, got {}", v.len()))
        })?;
        Ok(arr)
    }
}

/// Serde helper: [u8; 64] as hex string (Ed25519 signature).
mod hex_bytes64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        let s = <String as Deserialize>::deserialize(d)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        let arr: [u8; 64] = bytes.try_into().map_err(|v: Vec<u8>| {
            serde::de::Error::custom(format!("expected 64 bytes, got {}", v.len()))
        })?;
        Ok(arr)
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// A handful of invite tests don't care what ticket string is
    /// embedded, just that one is present. Use a recognisable
    /// placeholder so test failures are easy to grep.
    const FAKE_DOC_TICKET: &str = "fake-doc-ticket-bytes-AAAAAA";

    fn sample_invite(coord: &KeyPair, expires_at_unix: u64) -> Invite {
        Invite::mint(
            coord,
            "proj-abc".to_string(),
            "Test Project".to_string(),
            Some("https://relay.example.com/".to_string()),
            Some(FAKE_DOC_TICKET.to_string()),
            InviteScope::Worker,
            expires_at_unix,
        )
        .expect("sample mint is well-formed")
    }

    #[test]
    fn mint_and_verify_round_trip() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 2_000_000_000);
        invite.verify_signature().unwrap();
        assert_eq!(invite.payload.version, INVITE_FORMAT_VERSION);
        assert_eq!(invite.payload.coordinator_pubkey, coord.public_bytes());
        assert_eq!(invite.payload.scope, InviteScope::Worker);
    }

    #[test]
    fn encode_and_decode_preserves_payload() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 2_000_000_000);
        let wire = invite.encode();
        assert!(wire.starts_with(INVITE_PREFIX));

        let back = Invite::decode(&wire).unwrap();
        assert_eq!(back, invite);
    }

    #[test]
    fn encode_is_lowercase_tolerant_on_decode() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 2_000_000_000);
        let wire = invite.encode().to_ascii_lowercase();
        // Lowercase the *body* too — prefix stays lowercase
        // already. The decode helper upper-cases before
        // base32-parsing so this must still succeed.
        let back = Invite::decode(&wire).unwrap();
        assert_eq!(back.payload.project_id, "proj-abc");
    }

    #[test]
    fn decode_rejects_missing_prefix() {
        let bad = "notaninvite";
        match Invite::decode(bad).unwrap_err() {
            InviteError::BadPrefix => {}
            other => panic!("expected BadPrefix, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_bad_base32() {
        let bad = format!("{INVITE_PREFIX}!!!not-base32!!!");
        match Invite::decode(&bad).unwrap_err() {
            InviteError::BadBase32(_) => {}
            other => panic!("expected BadBase32, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_tampered_signature() {
        let coord = KeyPair::generate();
        let mut invite = sample_invite(&coord, 2_000_000_000);
        invite.signature[0] ^= 0xFF;
        let wire = invite.encode();
        match Invite::decode(&wire).unwrap_err() {
            InviteError::BadSignature(_) => {}
            other => panic!("expected BadSignature, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_tampered_payload() {
        let coord = KeyPair::generate();
        let mut invite = sample_invite(&coord, 2_000_000_000);
        // Flip a byte in the project name; the signature was
        // computed over the old canonical bytes and must fail.
        invite.payload.project_name = "Evil Project".to_string();
        let wire = invite.encode();
        match Invite::decode(&wire).unwrap_err() {
            InviteError::BadSignature(_) => {}
            other => panic!("expected BadSignature, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_wrong_issuer() {
        let real_coord = KeyPair::generate();
        let impostor = KeyPair::generate();

        // Mint with real_coord, then swap the embedded pubkey
        // to the impostor's — the signature now doesn't match
        // the embedded pubkey.
        let mut invite = sample_invite(&real_coord, 2_000_000_000);
        invite.payload.coordinator_pubkey = impostor.public_bytes();
        let wire = invite.encode();

        match Invite::decode(&wire).unwrap_err() {
            InviteError::BadSignature(_) => {}
            other => panic!("expected BadSignature, got {other:?}"),
        }
    }

    #[test]
    fn decode_rejects_unsupported_version() {
        let coord = KeyPair::generate();
        let mut invite = sample_invite(&coord, 2_000_000_000);
        invite.payload.version = 99;
        // Re-sign so the signature is valid for the bumped
        // version; the decoder should still reject on
        // UnsupportedVersion before even looking at the sig.
        invite.signature = coord.sign(&invite.payload.canonical_bytes());
        let wire = invite.encode();

        match Invite::decode(&wire).unwrap_err() {
            InviteError::UnsupportedVersion(v) => assert_eq!(v, 99),
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn ensure_not_expired_accepts_future_timestamp() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 2_000_000_000);
        invite.ensure_not_expired(1_000_000_000).unwrap();
    }

    #[test]
    fn ensure_not_expired_rejects_past_timestamp() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 1_000_000_000);
        match invite.ensure_not_expired(2_000_000_000).unwrap_err() {
            InviteError::Expired { expired_at, now } => {
                assert_eq!(expired_at, 1_000_000_000);
                assert_eq!(now, 2_000_000_000);
            }
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    #[test]
    fn scope_can_serve_tasks_helper() {
        assert!(InviteScope::Worker.can_serve_tasks());
        assert!(!InviteScope::Observer.can_serve_tasks());
    }

    #[test]
    fn worker_scope_requires_tasks_doc_ticket() {
        let coord = KeyPair::generate();
        let err = Invite::mint(
            &coord,
            "proj-abc".to_string(),
            "Test Project".to_string(),
            None,
            None, // tasks_doc_ticket deliberately missing
            InviteScope::Worker,
            2_000_000_000,
        )
        .unwrap_err();
        assert!(matches!(err, InviteError::MissingTasksDocTicket));
    }

    #[test]
    fn observer_scope_may_omit_tasks_doc_ticket() {
        let coord = KeyPair::generate();
        let invite = Invite::mint(
            &coord,
            "proj-abc".to_string(),
            "Test Project".to_string(),
            None,
            None, // observers do not need a task doc
            InviteScope::Observer,
            2_000_000_000,
        )
        .expect("observer invite without ticket must mint successfully");
        assert!(invite.payload.tasks_doc_ticket.is_none());
        let wire = invite.encode();
        let back = Invite::decode(&wire).unwrap();
        assert_eq!(back.payload.scope, InviteScope::Observer);
        assert!(back.payload.tasks_doc_ticket.is_none());
    }

    #[test]
    fn tasks_doc_ticket_round_trips_through_wire_format() {
        let coord = KeyPair::generate();
        let invite = sample_invite(&coord, 2_000_000_000);
        let wire = invite.encode();
        let back = Invite::decode(&wire).unwrap();
        assert_eq!(
            back.payload.tasks_doc_ticket,
            Some(FAKE_DOC_TICKET.to_string())
        );
    }

    #[test]
    fn decode_refuses_v1_after_hard_bump() {
        // Hand-craft a v1 invite and confirm the Sprint 4 decoder
        // refuses it as UnsupportedVersion.
        let coord = KeyPair::generate();
        let mut invite = sample_invite(&coord, 2_000_000_000);
        invite.payload.version = 1;
        // Re-sign so the bytes have a valid signature over the
        // v1-numbered payload; the decoder must still reject on
        // version mismatch before checking the sig.
        invite.signature = coord.sign(&invite.payload.canonical_bytes());
        let wire = invite.encode();
        match Invite::decode(&wire).unwrap_err() {
            InviteError::UnsupportedVersion(v) => assert_eq!(v, 1),
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn decode_refuses_worker_without_ticket_even_when_hand_crafted() {
        // Re-sign a payload where scope=Worker and ticket=None
        // to prove the decode-side check is also active, not just
        // the mint-side one.
        let coord = KeyPair::generate();
        let mut invite = sample_invite(&coord, 2_000_000_000);
        invite.payload.tasks_doc_ticket = None;
        invite.signature = coord.sign(&invite.payload.canonical_bytes());
        let wire = invite.encode();
        match Invite::decode(&wire).unwrap_err() {
            InviteError::MissingTasksDocTicket => {}
            other => panic!("expected MissingTasksDocTicket, got {other:?}"),
        }
    }

    #[test]
    fn invite_without_coordinator_addr_omits_field() {
        let coord = KeyPair::generate();
        let invite = Invite::mint(
            &coord,
            "proj-x".to_string(),
            "X".to_string(),
            None,
            Some(FAKE_DOC_TICKET.to_string()),
            InviteScope::Worker,
            2_000_000_000,
        )
        .unwrap();
        let json = serde_json::to_string(&invite).unwrap();
        assert!(
            !json.contains("coordinator_addr"),
            "None coordinator_addr must be omitted; got: {json}"
        );

        // Still round-trips.
        let wire = invite.encode();
        let back = Invite::decode(&wire).unwrap();
        assert_eq!(back, invite);
    }
}
