// SPDX-License-Identifier: AGPL-3.0-or-later
//! Signed envelopes: the only thing ever written to a conversation document.
//!
//! The outer envelope is public to anyone holding the document (members and
//! a read-only mailbox): it reveals the sender's messaging key, a sequence
//! number and the envelope kind — never plaintext. Every envelope is signed
//! by its sender under [`nexus_core_rs::DOMAIN_CHAT_ENVELOPE_V1`].

use serde::{Deserialize, Serialize};

use crate::identity::{Identity, PublicIdentity};
use crate::{ChatError, Result};

/// Upper bound on one encoded envelope, checked before parsing.
pub const MAX_ENVELOPE_BYTES: usize = 128 * 1024;

/// Plaintext padding buckets: an observer of ciphertext sizes only learns
/// the bucket, not the exact length.
pub const PAD_BUCKETS: [usize; 5] = [256, 1024, 4096, 16 * 1024, 64 * 1024];

/// What a member publishes so others can reach it while it is offline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberProfile {
    pub identity: PublicIdentity,
    /// Olm fallback key (vodozemac base64).
    pub fallback_key: String,
    /// iroh node id hosting this member (lets the daemon dial / sync).
    pub node_id: String,
    /// Display name chosen by the member (not authenticated beyond the
    /// envelope signature; shown next to the fingerprint).
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum Wire {
    /// The sender's reachability card.
    Profile { profile: MemberProfile },
    /// Admin-signed member list; `epoch` strictly increases.
    Roster {
        epoch: u64,
        members: Vec<MemberProfile>,
    },
    /// Invitee -> admin: Olm pre-key message opened with the invitation's
    /// one-time key, proving possession of the invitation.
    Join { olm_type: u8, olm: String },
    /// Olm-encrypted control payload for ONE member (`to` = its ed key).
    Direct {
        to: String,
        olm_type: u8,
        olm: String,
    },
    /// Megolm group message.
    Message { session_id: String, megolm: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Sender's messaging Ed25519 key (vodozemac base64).
    pub from: String,
    pub seq: u64,
    pub wire: Wire,
    pub sig: String,
}

#[derive(Serialize)]
struct Signed<'a> {
    from: &'a str,
    seq: u64,
    wire: &'a Wire,
}

fn signing_bytes(from: &str, seq: u64, wire: &Wire) -> Result<Vec<u8>> {
    nexus_core_rs::canonical_bytes(
        &Signed { from, seq, wire },
        nexus_core_rs::DOMAIN_CHAT_ENVELOPE_V1,
    )
    .map_err(|_| ChatError::Malformed("envelope"))
}

impl Envelope {
    pub fn seal(me: &Identity, seq: u64, wire: Wire) -> Result<Self> {
        let from = me.public().ed;
        let sig = me.sign(&signing_bytes(&from, seq, &wire)?);
        Ok(Self {
            from,
            seq,
            wire,
            sig,
        })
    }

    /// Decode AND verify. Anything that fails here is dropped by callers.
    pub fn open(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(ChatError::TooLarge);
        }
        let env: Envelope =
            serde_json::from_slice(bytes).map_err(|_| ChatError::Malformed("envelope"))?;
        env.verify()?;
        Ok(env)
    }

    pub fn verify(&self) -> Result<()> {
        let sender = PublicIdentity {
            curve: String::new(),
            ed: self.from.clone(),
        };
        sender.verify(
            &signing_bytes(&self.from, self.seq, &self.wire)?,
            &self.sig,
            "envelope",
        )
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("envelope serializes")
    }

    /// Document key: `m/<hex(sender ed)>/<seq:020>` — hex because vodozemac
    /// base64 may contain `/`; zero-padded so keys sort by sequence.
    pub fn doc_key(&self) -> Result<String> {
        Ok(format!("m/{}/{:020}", ed_hex(&self.from)?, self.seq))
    }
}

/// Hex of a vodozemac base64 Ed25519 key, used in document keys.
pub fn ed_hex(ed_b64: &str) -> Result<String> {
    let key = vodozemac::Ed25519PublicKey::from_base64(ed_b64)
        .map_err(|_| ChatError::Malformed("ed25519 key"))?;
    Ok(key.as_bytes().iter().map(|b| format!("{b:02x}")).collect())
}

/// Encrypted payloads (inside Olm or Megolm). Never visible on the wire.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "k", rename_all = "snake_case")]
pub(crate) enum Inner {
    Text {
        body: String,
        sent_at: u64,
    },
    SessionKey {
        session_id: String,
        key: String,
    },
    Moved {
        ticket: String,
        epoch: u64,
    },
    JoinProof {
        profile: MemberProfile,
        invite_otk: String,
    },
}

impl Inner {
    pub(crate) fn to_padded(&self) -> Vec<u8> {
        pad(&serde_json::to_vec(self).expect("inner serializes"))
    }

    pub(crate) fn from_padded(bytes: &[u8]) -> Result<Self> {
        serde_json::from_slice(&unpad(bytes)?).map_err(|_| ChatError::Malformed("payload"))
    }
}

pub(crate) fn pad(body: &[u8]) -> Vec<u8> {
    let need = body.len() + 4;
    let size = PAD_BUCKETS
        .iter()
        .copied()
        .find(|b| *b >= need)
        .unwrap_or(need);
    let mut out = Vec::with_capacity(size);
    out.extend_from_slice(&(body.len() as u32).to_be_bytes());
    out.extend_from_slice(body);
    out.resize(size, 0);
    out
}

pub(crate) fn unpad(bytes: &[u8]) -> Result<Vec<u8>> {
    let len_bytes: [u8; 4] = bytes
        .get(..4)
        .and_then(|s| s.try_into().ok())
        .ok_or(ChatError::Malformed("padding"))?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    bytes
        .get(4..4 + len)
        .map(<[u8]>::to_vec)
        .ok_or(ChatError::Malformed("padding"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(me: &Identity) -> Envelope {
        Envelope::seal(
            me,
            7,
            Wire::Message {
                session_id: "s".into(),
                megolm: "m".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn seal_open_round_trip_and_tamper_rejected() {
        let me = Identity::new();
        let env = sample(&me);
        assert_eq!(Envelope::open(&env.to_bytes()).unwrap(), env);

        let mut forged = env.clone();
        forged.seq = 8;
        assert!(matches!(
            Envelope::open(&forged.to_bytes()),
            Err(ChatError::BadSignature(_))
        ));

        let mut stolen = env;
        stolen.from = Identity::new().public().ed;
        assert!(Envelope::open(&stolen.to_bytes()).is_err());
    }

    #[test]
    fn doc_key_is_path_safe_and_sorted() {
        let me = Identity::new();
        let k = sample(&me).doc_key().unwrap();
        let parts: Vec<_> = k.split('/').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[1].len(), 64);
        assert_eq!(parts[2], "00000000000000000007");
    }

    #[test]
    fn padding_hides_length_within_bucket() {
        assert_eq!(pad(b"a").len(), 256);
        assert_eq!(pad(&[1; 300]).len(), 1024);
        assert_eq!(unpad(&pad(b"hello")).unwrap(), b"hello");
        assert!(unpad(&[0, 0, 1]).is_err());
        assert!(unpad(&[0, 0, 0, 9, 1]).is_err());
    }

    #[test]
    fn oversized_input_rejected_before_parse() {
        assert!(matches!(
            Envelope::open(&vec![b' '; MAX_ENVELOPE_BYTES + 1]),
            Err(ChatError::TooLarge)
        ));
    }
}
