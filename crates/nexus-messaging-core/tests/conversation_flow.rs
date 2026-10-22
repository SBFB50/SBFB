// SPDX-License-Identifier: AGPL-3.0-or-later
//! End-to-end protocol tests over a simulated conversation document: every
//! party ingests every envelope (its own included), in document order, and
//! writes back whatever `ingest` asks it to — exactly what the daemon does.

use nexus_messaging_core::{
    ChatError, ChatEvent, Conversation, Envelope, Identity, Invitation, MemberProfile,
    ROTATE_AFTER_MESSAGES, Wire,
};

const NOW: u64 = 1_800_000_000;
const DAY: u64 = 86_400;

struct Party {
    name: &'static str,
    id: Identity,
    conv: Option<Conversation>,
    cursor: usize,
    events: Vec<ChatEvent>,
}

impl Party {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            id: Identity::new(),
            conv: None,
            cursor: 0,
            events: Vec::new(),
        }
    }

    fn profile(&self) -> MemberProfile {
        MemberProfile {
            identity: self.id.public(),
            fallback_key: self.id.fallback_key(),
            node_id: format!("node-{}", self.name),
            name: self.name.into(),
        }
    }

    fn texts(&self) -> Vec<String> {
        self.events
            .iter()
            .filter_map(|e| match e {
                ChatEvent::Text { body, .. } => Some(body.clone()),
                _ => None,
            })
            .collect()
    }
}

/// Shared document: envelopes travel as bytes and are re-verified on read.
#[derive(Default)]
struct Doc(Vec<Vec<u8>>);

impl Doc {
    fn write(&mut self, envs: Vec<Envelope>) {
        self.0.extend(envs.iter().map(Envelope::to_bytes));
    }

    /// Let every party with a conversation catch up until nothing new appears.
    fn sync(&mut self, parties: &mut [&mut Party], now: u64) {
        loop {
            let mut wrote = false;
            for p in parties.iter_mut() {
                let Some(conv) = p.conv.as_mut() else {
                    continue;
                };
                while p.cursor < self.0.len() {
                    let env = Envelope::open(&self.0[p.cursor]).expect("doc holds valid envelopes");
                    p.cursor += 1;
                    if let Ok(ing) = conv.ingest(&mut p.id, &env, now) {
                        p.events.extend(ing.events);
                        if !ing.outgoing.is_empty() {
                            self.write(ing.outgoing);
                            wrote = true;
                        }
                    }
                }
            }
            if !wrote {
                return;
            }
        }
    }
}

fn start(alice: &mut Party, doc: &mut Doc) {
    let (conv, out) = Conversation::create(&alice.id, alice.profile(), NOW).unwrap();
    alice.conv = Some(conv);
    doc.write(out);
}

fn invite_and_join(admin: &mut Party, joiner: &mut Party, doc: &mut Doc) {
    let inv = admin
        .conv
        .as_mut()
        .unwrap()
        .invite(&mut admin.id, "ticket".into(), "Projet".into(), DAY, NOW)
        .unwrap();
    let inv = Invitation::decode(&inv.encode(), NOW).unwrap();
    let profile = joiner.profile();
    let (conv, out) = Conversation::accept(&mut joiner.id, profile, &inv, NOW).unwrap();
    joiner.conv = Some(conv);
    doc.write(out);
}

fn say(p: &mut Party, doc: &mut Doc, text: &str, now: u64) {
    let out = p
        .conv
        .as_mut()
        .unwrap()
        .send_text(&mut p.id, text, now)
        .unwrap();
    doc.write(out);
}

#[test]
fn one_to_one_then_group_of_three() {
    let (mut alice, mut bob, mut carol) =
        (Party::new("alice"), Party::new("bob"), Party::new("carol"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    assert_eq!(bob.conv.as_ref().unwrap().members().len(), 2);

    say(&mut alice, &mut doc, "salut bob", NOW);
    say(&mut bob, &mut doc, "salut alice", NOW);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    assert_eq!(bob.texts(), ["salut bob", "salut alice"]);
    assert_eq!(alice.texts(), ["salut bob", "salut alice"]);

    invite_and_join(&mut alice, &mut carol, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);
    say(&mut bob, &mut doc, "bienvenue carol", NOW);
    say(&mut carol, &mut doc, "merci", NOW);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);

    for p in [&alice, &bob, &carol] {
        assert!(
            p.texts()
                .ends_with(&["bienvenue carol".into(), "merci".into()]),
            "{}",
            p.name
        );
        assert_eq!(p.conv.as_ref().unwrap().members().len(), 3, "{}", p.name);
    }
    // Carol joined after the 1:1: Bob's first key predates her, she cannot
    // read the old messages.
    assert!(!carol.texts().contains(&"salut alice".to_string()));
}

#[test]
fn message_before_its_key_is_held_then_delivered() {
    let (mut alice, mut bob) = (Party::new("alice"), Party::new("bob"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);

    let mut out = alice
        .conv
        .as_mut()
        .unwrap()
        .send_text(&mut alice.id, "dans le desordre", NOW)
        .unwrap();
    out.reverse(); // the Message lands before the Direct carrying its key
    doc.write(out);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    assert_eq!(bob.texts(), ["dans le desordre"]);
}

#[test]
fn removed_member_reads_nothing_new_and_others_move() {
    let (mut alice, mut bob, mut carol) =
        (Party::new("alice"), Party::new("bob"), Party::new("carol"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    invite_and_join(&mut alice, &mut carol, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);
    say(&mut alice, &mut doc, "avant", NOW);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);

    // A malicious Carol keeps a copy of her keys and ignores her removal.
    let key = [3u8; 32];
    let spy_state = carol.conv.as_ref().unwrap().seal_state(&key);
    let spy_id = carol.id.seal(&key);
    let removal_at = doc.0.len();

    let carol_ed = carol.id.public().ed;
    let out = alice
        .conv
        .as_mut()
        .unwrap()
        .remove_member(&mut alice.id, &carol_ed, "new-ticket".into(), NOW)
        .unwrap();
    doc.write(out);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);
    assert!(carol.events.contains(&ChatEvent::Removed));
    assert!(
        bob.events
            .iter()
            .any(|e| matches!(e, ChatEvent::Moved { ticket, .. } if ticket == "new-ticket"))
    );

    say(&mut alice, &mut doc, "apres", NOW);
    say(&mut bob, &mut doc, "apres aussi", NOW);
    doc.sync(&mut [&mut alice, &mut bob, &mut carol], NOW);
    assert!(
        bob.texts()
            .ends_with(&["apres".into(), "apres aussi".into()])
    );
    assert_eq!(
        carol.texts(),
        ["avant"],
        "carol must not read anything after removal"
    );

    // Cryptographic check: the spy copy sees every post-removal MESSAGE
    // (never the roster that excludes it) and still decrypts nothing.
    let mut spy_id = Identity::unseal(&spy_id, &key).unwrap();
    let mut spy = Conversation::unseal_state(&spy_state, &key).unwrap();
    let mut leaked = Vec::new();
    for bytes in &doc.0[removal_at..] {
        let env = Envelope::open(bytes).unwrap();
        if matches!(env.wire, Wire::Message { .. } | Wire::Direct { .. })
            && let Ok(ing) = spy.ingest(&mut spy_id, &env, NOW)
        {
            leaked.extend(
                ing.events
                    .into_iter()
                    .filter(|e| matches!(e, ChatEvent::Text { .. })),
            );
        }
    }
    assert!(leaked.is_empty(), "removed member decrypted: {leaked:?}");

    // Even with direct access to the ciphertext, Carol has no key: no
    // Direct addressed to her exists after the removal.
    let tail: Vec<Envelope> = doc.0.iter().map(|b| Envelope::open(b).unwrap()).collect();
    let removal_idx = tail
        .iter()
        .rposition(|e| matches!(&e.wire, Wire::Roster { members, .. } if members.len() == 2 && e.from == alice.id.public().ed))
        .unwrap();
    assert!(
        !tail[removal_idx..]
            .iter()
            .any(|e| matches!(&e.wire, Wire::Direct { to, .. } if *to == carol_ed))
    );
}

#[test]
fn invitation_is_single_use_and_expires() {
    let (mut alice, mut bob, mut mallory) = (
        Party::new("alice"),
        Party::new("bob"),
        Party::new("mallory"),
    );
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    let inv = alice
        .conv
        .as_mut()
        .unwrap()
        .invite(&mut alice.id, "ticket".into(), "Projet".into(), DAY, NOW)
        .unwrap();

    let bob_profile = bob.profile();
    let (bob_conv, out) = Conversation::accept(&mut bob.id, bob_profile, &inv, NOW).unwrap();
    bob.conv = Some(bob_conv);
    doc.write(out);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    assert_eq!(alice.conv.as_ref().unwrap().members().len(), 2);

    // Same invitation replayed by someone else: rejected, roster unchanged.
    let mallory_profile = mallory.profile();
    let (_, replay) =
        Conversation::accept(&mut mallory.id, mallory_profile.clone(), &inv, NOW).unwrap();
    let join = replay
        .into_iter()
        .find(|e| matches!(e.wire, Wire::Join { .. }))
        .unwrap();
    let err = alice
        .conv
        .as_mut()
        .unwrap()
        .ingest(&mut alice.id, &join, NOW)
        .unwrap_err();
    assert!(matches!(err, ChatError::Malformed(_)), "{err:?}");
    assert_eq!(alice.conv.as_ref().unwrap().members().len(), 2);

    // An unused invitation past its expiry is refused at join time too.
    let late = alice
        .conv
        .as_mut()
        .unwrap()
        .invite(&mut alice.id, "ticket".into(), "Projet".into(), 10, NOW)
        .unwrap();
    let (_, out) = Conversation::accept(&mut mallory.id, mallory_profile, &late, NOW).unwrap();
    let join = out
        .into_iter()
        .find(|e| matches!(e.wire, Wire::Join { .. }))
        .unwrap();
    let err = alice
        .conv
        .as_mut()
        .unwrap()
        .ingest(&mut alice.id, &join, NOW + 11)
        .unwrap_err();
    assert!(matches!(err, ChatError::Expired), "{err:?}");
}

#[test]
fn only_the_admin_can_change_the_roster() {
    let (mut alice, mut bob) = (Party::new("alice"), Party::new("bob"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);

    let coup = Envelope::seal(
        &bob.id,
        99,
        Wire::Roster {
            epoch: 50,
            members: vec![bob.profile()],
        },
    )
    .unwrap();
    let err = alice
        .conv
        .as_mut()
        .unwrap()
        .ingest(&mut alice.id, &coup, NOW)
        .unwrap_err();
    assert!(matches!(err, ChatError::NotAdmin));
    assert_eq!(alice.conv.as_ref().unwrap().members().len(), 2);
    assert!(
        bob.conv
            .as_mut()
            .unwrap()
            .remove_member(&mut bob.id, &alice.id.public().ed, "t".into(), NOW)
            .is_err()
    );
}

#[test]
fn state_survives_seal_and_rotation_reshares_keys() {
    let (mut alice, mut bob) = (Party::new("alice"), Party::new("bob"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    say(&mut alice, &mut doc, "un", NOW);
    doc.sync(&mut [&mut alice, &mut bob], NOW);

    // Both sides restart from sealed state.
    let key = [9u8; 32];
    for p in [&mut alice, &mut bob] {
        let sealed_conv = p.conv.as_ref().unwrap().seal_state(&key);
        let sealed_id = p.id.seal(&key);
        p.id = Identity::unseal(&sealed_id, &key).unwrap();
        p.conv = Some(Conversation::unseal_state(&sealed_conv, &key).unwrap());
    }
    assert!(
        Conversation::unseal_state(&alice.conv.as_ref().unwrap().seal_state(&key), &[1; 32])
            .is_err()
    );

    say(&mut bob, &mut doc, "deux", NOW);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    assert_eq!(alice.texts(), ["un", "deux"]);

    // Rotation: after the threshold a new session id appears and Bob still reads.
    let sessions_before = session_ids(&doc, &alice);
    for i in 0..=ROTATE_AFTER_MESSAGES {
        say(&mut alice, &mut doc, &format!("m{i}"), NOW + 1);
    }
    doc.sync(&mut [&mut alice, &mut bob], NOW + 1);
    assert!(session_ids(&doc, &alice).len() > sessions_before.len());
    assert_eq!(
        bob.texts().last().unwrap(),
        &format!("m{ROTATE_AFTER_MESSAGES}")
    );

    // Time-based rotation too.
    let before = session_ids(&doc, &alice).len();
    say(&mut alice, &mut doc, "une semaine plus tard", NOW + 8 * DAY);
    doc.sync(&mut [&mut alice, &mut bob], NOW + 8 * DAY);
    assert_eq!(session_ids(&doc, &alice).len(), before + 1);
    assert_eq!(bob.texts().last().unwrap(), "une semaine plus tard");
}

fn session_ids(doc: &Doc, who: &Party) -> std::collections::BTreeSet<String> {
    let ed = who.id.public().ed;
    doc.0
        .iter()
        .map(|b| Envelope::open(b).unwrap())
        .filter(|e| e.from == ed)
        .filter_map(|e| match e.wire {
            Wire::Message { session_id, .. } => Some(session_id),
            _ => None,
        })
        .collect()
}

#[test]
fn a_member_cannot_replay_anothers_ciphertext_as_its_own() {
    let (mut alice, mut bob) = (Party::new("alice"), Party::new("bob"));
    let mut doc = Doc::default();
    start(&mut alice, &mut doc);
    invite_and_join(&mut alice, &mut bob, &mut doc);
    doc.sync(&mut [&mut alice, &mut bob], NOW);
    say(&mut alice, &mut doc, "de la part d'alice", NOW);
    doc.sync(&mut [&mut alice, &mut bob], NOW);

    let alice_msg = doc
        .0
        .iter()
        .map(|b| Envelope::open(b).unwrap())
        .rfind(|e| matches!(e.wire, Wire::Message { .. }))
        .unwrap();
    let forged = Envelope::seal(&bob.id, 500, alice_msg.wire.clone()).unwrap();
    let err = alice
        .conv
        .as_mut()
        .unwrap()
        .ingest(&mut alice.id, &forged, NOW)
        .unwrap_err();
    assert!(matches!(err, ChatError::BadSignature(_)), "{err:?}");
}
