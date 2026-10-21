// SPDX-License-Identifier: AGPL-3.0-or-later
//! Chat v1 spike A (GO/NO-GO): vodozemac covers the v1 crypto shape —
//! Olm pairwise channels carrying Megolm session keys, a 3-member group,
//! wire round-trip of every artefact, and member removal by rotation.

use vodozemac::megolm::{
    GroupSession, InboundGroupSession, MegolmMessage, SessionConfig as MegolmConfig, SessionKey,
};
use vodozemac::olm::{Account, OlmMessage, SessionConfig as OlmConfig};

struct Member {
    account: Account,
}

impl Member {
    fn new() -> Self {
        Self {
            account: Account::new(),
        }
    }

    /// Publish one one-time key, as an invitation would carry it.
    fn one_time_key(&mut self) -> vodozemac::Curve25519PublicKey {
        self.account.generate_one_time_keys(1);
        let key = *self.account.one_time_keys().values().next().unwrap();
        self.account.mark_keys_as_published();
        key
    }
}

/// Send `session_key` from `from` to `to` over a fresh Olm channel and
/// return the inbound Megolm session `to` derives from it.
fn share_group_key(
    from: &Member,
    to: &mut Member,
    session_key: &SessionKey,
) -> InboundGroupSession {
    let otk = to.one_time_key();
    let mut outbound = from
        .account
        .create_outbound_session(OlmConfig::version_1(), to.account.curve25519_key(), otk)
        .expect("outbound olm session");
    let wire = match outbound.encrypt(session_key.to_bytes()).unwrap() {
        OlmMessage::PreKey(m) => m.to_bytes(),
        OlmMessage::Normal(_) => panic!("first olm message must be a pre-key message"),
    };

    let pre_key = vodozemac::olm::PreKeyMessage::from_bytes(&wire).expect("pre-key decode");
    let inbound = to
        .account
        .create_inbound_session(
            OlmConfig::version_1(),
            from.account.curve25519_key(),
            &pre_key,
        )
        .expect("inbound olm session");
    let key = SessionKey::from_bytes(&inbound.plaintext).expect("session key decode");
    InboundGroupSession::new(&key, MegolmConfig::version_1())
}

fn send(group: &mut GroupSession, text: &str) -> Vec<u8> {
    group.encrypt(text.as_bytes()).to_bytes()
}

fn recv(inbound: &mut InboundGroupSession, wire: &[u8]) -> Result<String, String> {
    let msg = MegolmMessage::from_bytes(wire).map_err(|e| e.to_string())?;
    inbound
        .decrypt(&msg)
        .map(|d| String::from_utf8(d.plaintext).unwrap())
        .map_err(|e| e.to_string())
}

#[test]
fn group_of_three_then_removal_by_rotation() {
    let alice = Member::new();
    let mut bob = Member::new();
    let mut carol = Member::new();

    let mut alice_out = GroupSession::new(MegolmConfig::version_1());
    let key = alice_out.session_key();
    let mut bob_in = share_group_key(&alice, &mut bob, &key);
    let mut carol_in = share_group_key(&alice, &mut carol, &key);

    let m1 = send(&mut alice_out, "salut le groupe");
    assert_eq!(recv(&mut bob_in, &m1).unwrap(), "salut le groupe");
    assert_eq!(recv(&mut carol_in, &m1).unwrap(), "salut le groupe");

    // Tampering is rejected (Megolm messages are signed + MACed).
    let mut forged = m1.clone();
    let mid = forged.len() / 2;
    forged[mid] ^= 0x01;
    assert!(recv(&mut bob_in, &forged).is_err());

    // Carol is removed: Alice rotates and re-shares with Bob only.
    let mut alice_out = GroupSession::new(MegolmConfig::version_1());
    let key2 = alice_out.session_key();
    let mut bob_in2 = share_group_key(&alice, &mut bob, &key2);

    let m2 = send(&mut alice_out, "carol ne lit plus ceci");
    assert_eq!(recv(&mut bob_in2, &m2).unwrap(), "carol ne lit plus ceci");
    assert!(
        recv(&mut carol_in, &m2).is_err(),
        "a removed member must not decrypt the rotated session"
    );
}

#[test]
fn one_to_one_is_a_group_of_two() {
    let alice = Member::new();
    let mut bob = Member::new();
    let mut alice_out = GroupSession::new(MegolmConfig::version_1());
    let mut bob_in = share_group_key(&alice, &mut bob, &alice_out.session_key());
    for i in 0..5 {
        let m = send(&mut alice_out, &format!("msg {i}"));
        assert_eq!(recv(&mut bob_in, &m).unwrap(), format!("msg {i}"));
    }
}

#[test]
fn account_pickle_round_trips_under_a_key() {
    let account = Account::new();
    let identity = account.curve25519_key();
    let pickle_key = [7u8; 32];
    let sealed = account.pickle().encrypt(&pickle_key);
    let restored = Account::from_pickle(
        vodozemac::olm::AccountPickle::from_encrypted(&sealed, &pickle_key).unwrap(),
    );
    assert_eq!(restored.curve25519_key(), identity);
    assert!(vodozemac::olm::AccountPickle::from_encrypted(&sealed, &[8u8; 32]).is_err());
}
