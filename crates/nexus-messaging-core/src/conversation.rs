// SPDX-License-Identifier: AGPL-3.0-or-later
//! One conversation's state machine.
//!
//! Membership: the creator is the admin; it alone signs `Roster` envelopes
//! (strictly increasing `epoch`). An invitee proves it holds an invitation by
//! opening an Olm channel with the invitation's single-use key (`Join`).
//!
//! Keys: each member encrypts with its own Megolm outbound session and hands
//! the session key to every other member over Olm (`Direct`), lazily, at its
//! next send. Olm channels to offline members open on their published
//! fallback key. The outbound session rotates after
//! [`ROTATE_AFTER_MESSAGES`], after [`ROTATE_AFTER_SECS`], and whenever a
//! member is removed — a removed member never receives the new key.
//!
//! The daemon feeds EVERY document entry to [`Conversation::ingest`],
//! including the member's own (its own Megolm session is registered
//! inbound too), so history is rebuilt uniformly from the document.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};
use vodozemac::Curve25519PublicKey;
use vodozemac::megolm::{
    GroupSession, GroupSessionPickle, InboundGroupSession, InboundGroupSessionPickle,
    MegolmMessage, SessionConfig as MegolmConfig, SessionKey,
};
use vodozemac::olm::{OlmMessage, Session, SessionConfig as OlmConfig, SessionPickle};

use crate::envelope::{Envelope, Inner, MemberProfile, Wire};
use crate::identity::Identity;
use crate::invite::{Invitation, InvitationBody};
use crate::{ChatError, MAX_MEMBERS, MAX_TEXT_BYTES, Result, b64, unb64};

pub const ROTATE_AFTER_MESSAGES: u32 = 100;
pub const ROTATE_AFTER_SECS: u64 = 7 * 24 * 3600;
/// Olm sessions kept per peer (concurrent channel set-up can create two).
const OLM_SESSIONS_PER_PEER: usize = 3;
/// Messages waiting for their Megolm key; oldest dropped beyond this.
const MAX_PENDING: usize = 512;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatEvent {
    Text {
        from: String,
        body: String,
        sent_at: u64,
    },
    MembersChanged {
        epoch: u64,
        members: Vec<MemberProfile>,
    },
    /// The local member is no longer in the roster.
    Removed,
    /// Admin moved the conversation to a fresh document (after a removal).
    Moved { ticket: String, epoch: u64 },
}

/// Result of ingesting one envelope: events for the app, envelopes the
/// daemon must write back to the document.
#[derive(Debug, Default)]
pub struct Ingested {
    pub events: Vec<ChatEvent>,
    pub outgoing: Vec<Envelope>,
}

struct Inbound {
    sender: String,
    session: InboundGroupSession,
}

pub struct Conversation {
    me: String,
    admin: String,
    epoch: u64,
    in_roster: bool,
    removed: bool,
    members: BTreeMap<String, MemberProfile>,
    seq: u64,
    outbound: GroupSession,
    out_created_at: u64,
    out_count: u32,
    shared: BTreeSet<String>,
    inbound: HashMap<String, Inbound>,
    olm: HashMap<String, Vec<Session>>,
    pending_invites: BTreeMap<String, u64>,
    seen: BTreeSet<(String, u64)>,
    pending: Vec<Envelope>,
}

fn olm_wire(msg: &OlmMessage) -> (u8, String) {
    let (t, bytes) = msg.to_parts();
    (t as u8, b64(&bytes))
}

fn olm_parse(t: u8, b: &str) -> Result<OlmMessage> {
    OlmMessage::from_parts(t as usize, &unb64(b, "olm message")?)
        .map_err(|_| ChatError::Malformed("olm message"))
}

fn crypto<E: std::fmt::Display>(e: E) -> ChatError {
    ChatError::Crypto(e.to_string())
}

impl Conversation {
    fn blank(me: String, admin: String, now: u64) -> Self {
        let mut c = Self {
            me,
            admin,
            epoch: 0,
            in_roster: false,
            removed: false,
            members: BTreeMap::new(),
            seq: 0,
            outbound: GroupSession::new(MegolmConfig::version_1()),
            out_created_at: now,
            out_count: 0,
            shared: BTreeSet::new(),
            inbound: HashMap::new(),
            olm: HashMap::new(),
            pending_invites: BTreeMap::new(),
            seen: BTreeSet::new(),
            pending: Vec::new(),
        };
        c.register_own_outbound();
        c
    }

    /// Start a conversation as its admin.
    pub fn create(
        me: &Identity,
        profile: MemberProfile,
        now: u64,
    ) -> Result<(Self, Vec<Envelope>)> {
        let my_ed = me.public().ed;
        if profile.identity != me.public() {
            return Err(ChatError::Malformed("own profile"));
        }
        let mut c = Self::blank(my_ed.clone(), my_ed.clone(), now);
        c.members.insert(my_ed, profile.clone());
        c.epoch = 1;
        c.in_roster = true;
        let out = vec![
            c.seal(me, Wire::Profile { profile })?,
            c.seal(
                me,
                Wire::Roster {
                    epoch: 1,
                    members: c.member_list(),
                },
            )?,
        ];
        Ok((c, out))
    }

    /// Admin: mint an invitation carrying the document WRITE ticket.
    pub fn invite(
        &mut self,
        me: &mut Identity,
        doc_ticket: String,
        title: String,
        ttl_secs: u64,
        now: u64,
    ) -> Result<Invitation> {
        self.require_admin()?;
        if self.members.len() >= MAX_MEMBERS {
            return Err(ChatError::Full);
        }
        let otk = me.one_time_key();
        let expires_at = now + ttl_secs;
        self.pending_invites.insert(otk.clone(), expires_at);
        let body = InvitationBody {
            v: 1,
            inviter: self.members[&self.me].clone(),
            one_time_key: otk,
            doc_ticket,
            title,
            expires_at,
        };
        Invitation::sign(me, body)
    }

    /// Invitee: join from a verified invitation.
    pub fn accept(
        me: &mut Identity,
        profile: MemberProfile,
        invitation: &Invitation,
        now: u64,
    ) -> Result<(Self, Vec<Envelope>)> {
        if profile.identity != me.public() {
            return Err(ChatError::Malformed("own profile"));
        }
        let admin = invitation.body.inviter.clone();
        let mut c = Self::blank(me.public().ed, admin.identity.ed.clone(), now);
        c.members.insert(admin.identity.ed.clone(), admin.clone());
        c.members.insert(c.me.clone(), profile.clone());

        let otk = Curve25519PublicKey::from_base64(&invitation.body.one_time_key)
            .map_err(|_| ChatError::Malformed("invitation key"))?;
        let mut session = me
            .account()
            .create_outbound_session(OlmConfig::version_1(), admin.identity.curve_key()?, otk)
            .map_err(crypto)?;
        let proof = Inner::JoinProof {
            profile: profile.clone(),
            invite_otk: invitation.body.one_time_key.clone(),
        };
        let (olm_type, olm) = olm_wire(&session.encrypt(proof.to_padded()).map_err(crypto)?);
        c.olm.insert(admin.identity.ed.clone(), vec![session]);

        let out = vec![
            c.seal(me, Wire::Profile { profile })?,
            c.seal(me, Wire::Join { olm_type, olm })?,
        ];
        Ok((c, out))
    }

    pub fn send_text(&mut self, me: &mut Identity, body: &str, now: u64) -> Result<Vec<Envelope>> {
        if self.removed {
            return Err(ChatError::NotMember);
        }
        if body.len() > MAX_TEXT_BYTES {
            return Err(ChatError::TooLarge);
        }
        if self.out_count >= ROTATE_AFTER_MESSAGES
            || now.saturating_sub(self.out_created_at) >= ROTATE_AFTER_SECS
        {
            self.rotate(now);
        }
        let mut out = self.share_outbound_key(me)?;
        let inner = Inner::Text {
            body: body.to_owned(),
            sent_at: now,
        };
        let megolm = b64(&self.outbound.encrypt(inner.to_padded()).to_bytes());
        self.out_count += 1;
        out.push(self.seal(
            me,
            Wire::Message {
                session_id: self.outbound.session_id(),
                megolm,
            },
        )?);
        Ok(out)
    }

    /// Admin: remove a member and point everyone else to a fresh document
    /// (`new_ticket`). The removed member keeps old history, nothing new.
    pub fn remove_member(
        &mut self,
        me: &mut Identity,
        ed: &str,
        new_ticket: String,
        now: u64,
    ) -> Result<Vec<Envelope>> {
        self.require_admin()?;
        if ed == self.me || self.members.remove(ed).is_none() {
            return Err(ChatError::NotMember);
        }
        self.epoch += 1;
        self.rotate(now);
        let mut out = vec![self.seal(
            me,
            Wire::Roster {
                epoch: self.epoch,
                members: self.member_list(),
            },
        )?];
        let peers: Vec<String> = self
            .members
            .keys()
            .filter(|k| **k != self.me)
            .cloned()
            .collect();
        for peer in peers {
            let inner = Inner::Moved {
                ticket: new_ticket.clone(),
                epoch: self.epoch,
            };
            out.push(self.direct(me, &peer, &inner)?);
        }
        Ok(out)
    }

    pub fn ingest(&mut self, me: &mut Identity, env: &Envelope, now: u64) -> Result<Ingested> {
        let id = (env.from.clone(), env.seq);
        if self.removed || self.seen.contains(&id) {
            return Ok(Ingested::default());
        }
        // Marked seen only once handled: a rejected envelope must not shadow
        // a later valid one reusing (from, seq) — e.g. a retried join.
        let ingested = self.dispatch(me, env, now)?;
        self.seen.insert(id);
        Ok(ingested)
    }

    fn dispatch(&mut self, me: &mut Identity, env: &Envelope, now: u64) -> Result<Ingested> {
        match &env.wire {
            Wire::Profile { profile } => {
                if profile.identity.ed != env.from {
                    return Err(ChatError::Malformed("profile"));
                }
                if let Some(slot) = self.members.get_mut(&env.from) {
                    *slot = profile.clone();
                }
                Ok(Ingested::default())
            }
            Wire::Roster { epoch, members } => self.on_roster(env, *epoch, members, now),
            Wire::Join { olm_type, olm } => self.on_join(me, env, *olm_type, olm, now),
            Wire::Direct { to, olm_type, olm } => {
                if *to != self.me {
                    return Ok(Ingested::default());
                }
                self.on_direct(me, env, *olm_type, olm)
            }
            Wire::Message { session_id, megolm } => {
                let mut ingested = Ingested::default();
                match self.decrypt_message(env, session_id, megolm)? {
                    Some(ev) => ingested.events.push(ev),
                    None => {
                        if self.pending.len() >= MAX_PENDING {
                            self.pending.remove(0);
                        }
                        self.pending.push(env.clone());
                    }
                }
                Ok(ingested)
            }
        }
    }

    fn on_roster(
        &mut self,
        env: &Envelope,
        epoch: u64,
        members: &[MemberProfile],
        now: u64,
    ) -> Result<Ingested> {
        if env.from != self.admin {
            return Err(ChatError::NotAdmin);
        }
        if epoch <= self.epoch {
            return Ok(Ingested::default());
        }
        if members.len() > MAX_MEMBERS {
            return Err(ChatError::Full);
        }
        let next: BTreeMap<String, MemberProfile> = members
            .iter()
            .map(|p| (p.identity.ed.clone(), p.clone()))
            .collect();
        let lost_someone = self.members.keys().any(|k| !next.contains_key(k));
        let me_listed = next.contains_key(&self.me);
        self.epoch = epoch;

        let mut ingested = Ingested::default();
        if !me_listed {
            if self.in_roster {
                self.removed = true;
                ingested.events.push(ChatEvent::Removed);
            }
            // An invitee replaying history sees rosters from before its join.
            return Ok(ingested);
        }
        let mine = self.members.get(&self.me).cloned();
        self.members = next;
        if let Some(mine) = mine {
            self.members.insert(self.me.clone(), mine);
        }
        self.in_roster = true;
        if lost_someone {
            self.rotate(now);
        }
        ingested.events.push(ChatEvent::MembersChanged {
            epoch,
            members: self.member_list(),
        });
        Ok(ingested)
    }

    fn on_join(
        &mut self,
        me: &mut Identity,
        env: &Envelope,
        olm_type: u8,
        olm: &str,
        now: u64,
    ) -> Result<Ingested> {
        if self.me != self.admin {
            return Ok(Ingested::default());
        }
        let OlmMessage::PreKey(pre) = olm_parse(olm_type, olm)? else {
            return Err(ChatError::Malformed("join"));
        };
        let claimed_otk = pre.one_time_key().to_base64();
        match self.pending_invites.get(&claimed_otk) {
            Some(exp) if *exp > now => {}
            Some(_) => return Err(ChatError::Expired),
            None => return Err(ChatError::Malformed("join: unknown invitation")),
        }
        if self.members.len() >= MAX_MEMBERS {
            return Err(ChatError::Full);
        }
        let created = me
            .account_mut()
            .create_inbound_session(OlmConfig::version_1(), pre.identity_key(), &pre)
            .map_err(crypto)?;
        let Inner::JoinProof {
            profile,
            invite_otk,
        } = Inner::from_padded(&created.plaintext)?
        else {
            return Err(ChatError::Malformed("join proof"));
        };
        if invite_otk != claimed_otk
            || profile.identity.ed != env.from
            || profile.identity.curve != pre.identity_key().to_base64()
        {
            return Err(ChatError::Malformed("join proof"));
        }
        self.pending_invites.remove(&claimed_otk);
        self.olm.insert(env.from.clone(), vec![created.session]);
        self.members.insert(env.from.clone(), profile);
        self.epoch += 1;
        let roster = self.seal(
            me,
            Wire::Roster {
                epoch: self.epoch,
                members: self.member_list(),
            },
        )?;
        Ok(Ingested {
            events: vec![ChatEvent::MembersChanged {
                epoch: self.epoch,
                members: self.member_list(),
            }],
            outgoing: vec![roster],
        })
    }

    fn on_direct(
        &mut self,
        me: &mut Identity,
        env: &Envelope,
        olm_type: u8,
        olm: &str,
    ) -> Result<Ingested> {
        if !self.members.contains_key(&env.from) {
            return Err(ChatError::NotMember);
        }
        let plain = self.olm_decrypt(me, &env.from, olm_parse(olm_type, olm)?)?;
        let mut ingested = Ingested::default();
        match Inner::from_padded(&plain)? {
            Inner::SessionKey { session_id, key } => {
                let key = SessionKey::from_base64(&key)
                    .map_err(|_| ChatError::Malformed("session key"))?;
                let session = InboundGroupSession::new(&key, MegolmConfig::version_1());
                if session.session_id() != session_id {
                    return Err(ChatError::Malformed("session key id"));
                }
                // First binding wins: a member cannot re-bind someone else's session.
                self.inbound.entry(session_id).or_insert(Inbound {
                    sender: env.from.clone(),
                    session,
                });
                ingested.events = self.retry_pending()?;
            }
            Inner::Moved { ticket, epoch } if env.from == self.admin => {
                ingested.events.push(ChatEvent::Moved { ticket, epoch });
            }
            _ => return Err(ChatError::Malformed("direct payload")),
        }
        Ok(ingested)
    }

    /// Re-publish the local member's profile (e.g. after a name change).
    pub fn announce_profile(&mut self, me: &Identity, profile: MemberProfile) -> Result<Envelope> {
        if profile.identity != me.public() {
            return Err(ChatError::Malformed("own profile"));
        }
        self.members.insert(self.me.clone(), profile.clone());
        self.seal(me, Wire::Profile { profile })
    }

    /// Re-read a stored `Message` envelope without touching ingest state.
    /// Megolm inbound sessions decrypt any index from their first known one,
    /// so the daemon rebuilds history from the document on demand and never
    /// keeps plaintext at rest. `None` = not a message, or no key for it.
    pub fn read_message(&mut self, env: &Envelope) -> Option<ChatEvent> {
        match &env.wire {
            Wire::Message { session_id, megolm } => {
                self.decrypt_message(env, session_id, megolm).ok().flatten()
            }
            _ => None,
        }
    }

    fn decrypt_message(
        &mut self,
        env: &Envelope,
        session_id: &str,
        megolm: &str,
    ) -> Result<Option<ChatEvent>> {
        let Some(inbound) = self.inbound.get_mut(session_id) else {
            return Ok(None);
        };
        if inbound.sender != env.from {
            return Err(ChatError::BadSignature("message sender"));
        }
        let msg = MegolmMessage::from_bytes(&unb64(megolm, "megolm message")?)
            .map_err(|_| ChatError::Malformed("megolm message"))?;
        let plain = inbound.session.decrypt(&msg).map_err(crypto)?;
        match Inner::from_padded(&plain.plaintext)? {
            Inner::Text { body, sent_at } => Ok(Some(ChatEvent::Text {
                from: env.from.clone(),
                body,
                sent_at,
            })),
            _ => Err(ChatError::Malformed("message payload")),
        }
    }

    fn retry_pending(&mut self) -> Result<Vec<ChatEvent>> {
        let waiting = std::mem::take(&mut self.pending);
        let mut events = Vec::new();
        for env in waiting {
            if let Wire::Message { session_id, megolm } = &env.wire {
                match self.decrypt_message(&env, session_id, megolm) {
                    Ok(Some(ev)) => events.push(ev),
                    Ok(None) => self.pending.push(env),
                    Err(_) => {}
                }
            }
        }
        Ok(events)
    }

    fn olm_decrypt(&mut self, me: &mut Identity, peer: &str, msg: OlmMessage) -> Result<Vec<u8>> {
        if let Some(sessions) = self.olm.get_mut(peer) {
            for s in sessions.iter_mut() {
                if let Ok(p) = s.decrypt(&msg) {
                    return Ok(p);
                }
            }
        }
        let OlmMessage::PreKey(pre) = msg else {
            return Err(ChatError::MissingKey);
        };
        let curve = self.members[peer].identity.curve_key()?;
        let created = me
            .account_mut()
            .create_inbound_session(OlmConfig::version_1(), curve, &pre)
            .map_err(crypto)?;
        let sessions = self.olm.entry(peer.to_owned()).or_default();
        sessions.insert(0, created.session);
        sessions.truncate(OLM_SESSIONS_PER_PEER);
        Ok(created.plaintext)
    }

    fn direct(&mut self, me: &Identity, peer: &str, inner: &Inner) -> Result<Envelope> {
        if self.olm.get(peer).is_none_or(Vec::is_empty) {
            let profile = &self.members[peer];
            let fallback = Curve25519PublicKey::from_base64(&profile.fallback_key)
                .map_err(|_| ChatError::Malformed("fallback key"))?;
            let session = me
                .account()
                .create_outbound_session(
                    OlmConfig::version_1(),
                    profile.identity.curve_key()?,
                    fallback,
                )
                .map_err(crypto)?;
            self.olm.insert(peer.to_owned(), vec![session]);
        }
        let session = &mut self.olm.get_mut(peer).expect("just ensured")[0];
        let (olm_type, olm) = olm_wire(&session.encrypt(inner.to_padded()).map_err(crypto)?);
        self.seal(
            me,
            Wire::Direct {
                to: peer.to_owned(),
                olm_type,
                olm,
            },
        )
    }

    fn share_outbound_key(&mut self, me: &Identity) -> Result<Vec<Envelope>> {
        let inner = Inner::SessionKey {
            session_id: self.outbound.session_id(),
            key: self.outbound.session_key().to_base64(),
        };
        let peers: Vec<String> = self
            .members
            .keys()
            .filter(|k| **k != self.me && !self.shared.contains(*k))
            .cloned()
            .collect();
        let mut out = Vec::with_capacity(peers.len());
        for peer in peers {
            out.push(self.direct(me, &peer, &inner)?);
            self.shared.insert(peer);
        }
        Ok(out)
    }

    fn rotate(&mut self, now: u64) {
        self.outbound = GroupSession::new(MegolmConfig::version_1());
        self.out_created_at = now;
        self.out_count = 0;
        self.shared.clear();
        self.register_own_outbound();
    }

    fn register_own_outbound(&mut self) {
        let inbound =
            InboundGroupSession::new(&self.outbound.session_key(), MegolmConfig::version_1());
        self.inbound.insert(
            self.outbound.session_id(),
            Inbound {
                sender: self.me.clone(),
                session: inbound,
            },
        );
    }

    fn seal(&mut self, me: &Identity, wire: Wire) -> Result<Envelope> {
        self.seq += 1;
        Envelope::seal(me, self.seq, wire)
    }

    fn require_admin(&self) -> Result<()> {
        if self.me == self.admin {
            Ok(())
        } else {
            Err(ChatError::NotAdmin)
        }
    }

    fn member_list(&self) -> Vec<MemberProfile> {
        self.members.values().cloned().collect()
    }

    pub fn members(&self) -> Vec<MemberProfile> {
        self.member_list()
    }

    pub fn admin(&self) -> &str {
        &self.admin
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn is_admin(&self) -> bool {
        self.me == self.admin
    }

    pub fn is_removed(&self) -> bool {
        self.removed
    }

    /// Persist: every vodozemac secret is pickled under `key`; the daemon
    /// stores the returned string in its sealed host vault.
    pub fn seal_state(&self, key: &[u8; 32]) -> String {
        let state = State {
            me: self.me.clone(),
            admin: self.admin.clone(),
            epoch: self.epoch,
            in_roster: self.in_roster,
            removed: self.removed,
            members: self.members.clone(),
            seq: self.seq,
            outbound: self.outbound.pickle().encrypt(key),
            out_created_at: self.out_created_at,
            out_count: self.out_count,
            shared: self.shared.clone(),
            inbound: self
                .inbound
                .iter()
                .map(|(id, i)| {
                    (
                        id.clone(),
                        i.sender.clone(),
                        i.session.pickle().encrypt(key),
                    )
                })
                .collect(),
            olm: self
                .olm
                .iter()
                .map(|(p, ss)| {
                    (
                        p.clone(),
                        ss.iter().map(|s| s.pickle().encrypt(key)).collect(),
                    )
                })
                .collect(),
            pending_invites: self.pending_invites.clone(),
            seen: self.seen.iter().cloned().collect(),
            pending: self.pending.clone(),
        };
        serde_json::to_string(&state).expect("state serializes")
    }

    pub fn unseal_state(sealed: &str, key: &[u8; 32]) -> Result<Self> {
        let s: State =
            serde_json::from_str(sealed).map_err(|_| ChatError::Malformed("conversation state"))?;
        let bad = |_| ChatError::Crypto("conversation unseal failed".into());
        let mut inbound = HashMap::new();
        for (id, sender, p) in s.inbound {
            let session = InboundGroupSession::from_pickle(
                InboundGroupSessionPickle::from_encrypted(&p, key).map_err(bad)?,
            );
            inbound.insert(id, Inbound { sender, session });
        }
        let mut olm = HashMap::new();
        for (peer, list) in s.olm {
            let mut sessions = Vec::new();
            for p in list {
                sessions.push(Session::from_pickle(
                    SessionPickle::from_encrypted(&p, key).map_err(bad)?,
                ));
            }
            olm.insert(peer, sessions);
        }
        Ok(Self {
            me: s.me,
            admin: s.admin,
            epoch: s.epoch,
            in_roster: s.in_roster,
            removed: s.removed,
            members: s.members,
            seq: s.seq,
            outbound: GroupSession::from_pickle(
                GroupSessionPickle::from_encrypted(&s.outbound, key).map_err(bad)?,
            ),
            out_created_at: s.out_created_at,
            out_count: s.out_count,
            shared: s.shared,
            inbound,
            olm,
            pending_invites: s.pending_invites,
            seen: s.seen.into_iter().collect(),
            pending: s.pending,
        })
    }
}

#[derive(Serialize, Deserialize)]
struct State {
    me: String,
    admin: String,
    epoch: u64,
    in_roster: bool,
    removed: bool,
    members: BTreeMap<String, MemberProfile>,
    seq: u64,
    outbound: String,
    out_created_at: u64,
    out_count: u32,
    shared: BTreeSet<String>,
    inbound: Vec<(String, String, String)>,
    olm: Vec<(String, Vec<String>)>,
    pending_invites: BTreeMap<String, u64>,
    seen: Vec<(String, u64)>,
    pending: Vec<Envelope>,
}
