// SPDX-License-Identifier: AGPL-3.0-or-later
//! Invitations: a signed, expiring, single-use link that lets someone join a
//! conversation. It carries the WRITE ticket of the conversation document,
//! so it must be shared over a channel the inviter trusts (shown as a QR or
//! pasted), never published.

use serde::{Deserialize, Serialize};

use crate::envelope::MemberProfile;
use crate::identity::Identity;
use crate::{ChatError, Result, b64, unb64};

/// Text form: `sbfb-chat:<base64url(json)>`.
pub const INVITE_PREFIX: &str = "sbfb-chat:";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitationBody {
    pub v: u32,
    /// The inviter (always the conversation admin in v1).
    pub inviter: MemberProfile,
    /// Single-use Olm key the invitee must consume to prove possession.
    pub one_time_key: String,
    /// iroh-docs WRITE ticket of the conversation document.
    pub doc_ticket: String,
    pub title: String,
    pub expires_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invitation {
    pub body: InvitationBody,
    pub sig: String,
}

fn signing_bytes(body: &InvitationBody) -> Result<Vec<u8>> {
    nexus_core_rs::canonical_bytes(body, nexus_core_rs::DOMAIN_CHAT_INVITE_V1)
        .map_err(|_| ChatError::Malformed("invitation"))
}

impl Invitation {
    pub fn sign(inviter: &Identity, body: InvitationBody) -> Result<Self> {
        if body.inviter.identity != inviter.public() {
            return Err(ChatError::Malformed("invitation inviter"));
        }
        let sig = inviter.sign(&signing_bytes(&body)?);
        Ok(Self { body, sig })
    }

    pub fn encode(&self) -> String {
        format!(
            "{INVITE_PREFIX}{}",
            b64(&serde_json::to_vec(self).expect("invitation serializes"))
        )
    }

    /// Parse, verify the signature and the expiry.
    pub fn decode(text: &str, now: u64) -> Result<Self> {
        let raw = text
            .trim()
            .strip_prefix(INVITE_PREFIX)
            .ok_or(ChatError::Malformed("invitation"))?;
        let inv: Invitation = serde_json::from_slice(&unb64(raw, "invitation")?)
            .map_err(|_| ChatError::Malformed("invitation"))?;
        if inv.body.v != 1 {
            return Err(ChatError::Malformed("invitation version"));
        }
        inv.body
            .inviter
            .identity
            .verify(&signing_bytes(&inv.body)?, &inv.sig, "invitation")?;
        if now >= inv.body.expires_at {
            return Err(ChatError::Expired);
        }
        Ok(inv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &Identity) -> MemberProfile {
        MemberProfile {
            identity: id.public(),
            fallback_key: id.fallback_key(),
            node_id: "node".into(),
            name: "Alice".into(),
        }
    }

    fn invitation(id: &mut Identity, expires_at: u64) -> Invitation {
        let body = InvitationBody {
            v: 1,
            inviter: profile(id),
            one_time_key: id.one_time_key(),
            doc_ticket: "docticket".into(),
            title: "Projet".into(),
            expires_at,
        };
        Invitation::sign(id, body).unwrap()
    }

    #[test]
    fn encode_decode_verifies_and_expires() {
        let mut alice = Identity::new();
        let inv = invitation(&mut alice, 100);
        let text = inv.encode();
        assert!(text.starts_with(INVITE_PREFIX));
        assert_eq!(Invitation::decode(&text, 99).unwrap(), inv);
        assert!(matches!(
            Invitation::decode(&text, 100),
            Err(ChatError::Expired)
        ));
    }

    #[test]
    fn tampered_invitation_rejected() {
        let mut alice = Identity::new();
        let mut inv = invitation(&mut alice, 100);
        inv.body.doc_ticket = "attacker-doc".into();
        assert!(matches!(
            Invitation::decode(&inv.encode(), 1),
            Err(ChatError::BadSignature(_))
        ));
        assert!(Invitation::decode("sbfb-chat:!!", 1).is_err());
        assert!(Invitation::decode("other:abc", 1).is_err());
    }

    #[test]
    fn cannot_sign_for_someone_else() {
        let mut alice = Identity::new();
        let mallory = Identity::new();
        let body = invitation(&mut alice, 100).body;
        assert!(Invitation::sign(&mallory, body).is_err());
    }
}
