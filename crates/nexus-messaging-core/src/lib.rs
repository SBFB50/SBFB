// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB chat v1 core (ADR `docs/architecture/ADR_CHAT_V1.md`): end-to-end
//! encryption for 1:1 and small groups, run by the local daemon, carried as
//! opaque signed entries in one iroh-docs document per conversation.
//!
//! Crypto is vodozemac: Olm for pairwise channels, Megolm for each member's
//! outbound group session. This crate does no I/O and owns no clock — the
//! caller passes `now` (unix seconds) wherever time matters.
//!
//! Layout of a conversation document (keys are UTF-8):
//!
//! - `m/<sender_ed>/<seq:020>` — one [`envelope::Envelope`] per entry
//!   ([`envelope::Envelope::doc_key`]).
//!
//! Everything a member needs to reach another member asynchronously (its
//! identity and Olm fallback key) travels in signed `Profile` envelopes, so
//! two members never have to be online at the same time.

pub mod conversation;
pub mod envelope;
pub mod identity;
pub mod invite;
pub mod mailbox;

pub use conversation::{ChatEvent, Conversation, ROTATE_AFTER_MESSAGES, ROTATE_AFTER_SECS};
pub use envelope::{Envelope, MemberProfile, Wire};
pub use identity::{Identity, PublicIdentity};
pub use invite::{INVITE_PREFIX, Invitation, InvitationBody};

/// Hard cap on members of one conversation (ADR: keeps a group on the
/// "conversation" side of the doctrine line, not a platform).
pub const MAX_MEMBERS: usize = 50;

/// Hard cap on one text message, in bytes of UTF-8.
pub const MAX_TEXT_BYTES: usize = 16 * 1024;

/// Errors surfaced by the messaging core. Messages never echo plaintext.
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error("malformed {0}")]
    Malformed(&'static str),
    #[error("bad signature on {0}")]
    BadSignature(&'static str),
    #[error("invitation expired")]
    Expired,
    #[error("sender is not a member of this conversation")]
    NotMember,
    #[error("only the conversation admin may do this")]
    NotAdmin,
    #[error("conversation is full ({MAX_MEMBERS} members)")]
    Full,
    #[error("message too large")]
    TooLarge,
    #[error("no key for this message yet")]
    MissingKey,
    #[error("crypto: {0}")]
    Crypto(String),
}

pub type Result<T> = std::result::Result<T, ChatError>;

pub(crate) fn b64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

pub(crate) fn unb64(s: &str, what: &'static str) -> Result<Vec<u8>> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .map_err(|_| ChatError::Malformed(what))
}
