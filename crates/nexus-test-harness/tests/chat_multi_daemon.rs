// SPDX-License-Identifier: AGPL-3.0-or-later
//! Chat v1 between REAL daemon processes (plan Phase G, "intégration
//! multi-daemon C, E").
//!
//! Each node is a separate `nexus-shell-daemon` process with its own
//! `NEXUS_GRID_ROOT` / `SBFB_HOME`, driven only through its loopback HTTP
//! API (`/app/{name}/chat/*`) with its own auth token. Unlike the
//! in-process tests of `chat_api.rs`, nothing is shared: keys, stores,
//! sockets and the sealing key file all live in distinct processes.
//!
//! Hermetic by construction (no public network, so no `SBFB_INTEGRATION`
//! gate): every daemon boots with a `config.toml` written beforehand (no
//! first-boot public curator / mailbox) and with [`HERMETIC_ENV`]
//! (zero-n0 discovery pointed at a dead loopback port). Peers reach each
//! other through the direct addresses carried by tickets only.
//!
//! One consequence for the mailbox test: a `[mailbox] servers` entry is a
//! bare endpoint id, which production resolves through pkarr + the relay.
//! Neither can run here (relay and pkarr URLs must be https with a WebPKI
//! certificate), so each client is first introduced to the mailbox node by
//! importing a document ticket of that node (`/api/daemon/storage/join`):
//! iroh-docs records the ticket's addresses in the endpoint-wide address
//! lookup, after which the mailbox node resolves by id like any peer.

use std::path::{Path, PathBuf};
use std::time::Duration;

use nexus_test_harness::{DaemonHandle, SpawnOptions};
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

const APP: &str = "chat-it";
/// Upper bound for any cross-process convergence step.
const CONVERGE: Duration = Duration::from_secs(90);
const POLL: Duration = Duration::from_millis(300);
/// How long a removed member is watched for a message it must not read,
/// once the remaining members already have it.
const REMOVED_WATCH: Duration = Duration::from_secs(15);

fn config(servers: &[&str], serve: bool) -> String {
    let list = servers
        .iter()
        .map(|s| format!("\"{s}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[curator]\ndefault_curators = []\n\n[mailbox]\nservers = [{list}]\nserve = {serve}\n")
}

async fn node(servers: &[&str], serve: bool) -> DaemonHandle {
    DaemonHandle::spawn_with(SpawnOptions {
        config_toml: Some(config(servers, serve)),
        hermetic: true,
    })
    .await
    .expect("spawn hermetic daemon")
}

async fn chat(d: &DaemonHandle, method: Method, sub: &str, body: Value) -> (StatusCode, Value) {
    let body = if body.is_null() { None } else { Some(body) };
    d.request(method, &format!("/app/{APP}/chat{sub}"), body)
        .await
        .expect("chat request")
}

async fn ok(d: &DaemonHandle, method: Method, sub: &str, body: Value) -> Value {
    let (s, v) = chat(d, method, sub, body).await;
    assert_eq!(s, StatusCode::OK, "{sub}: {v}");
    v
}

async fn set_name(d: &DaemonHandle, name: &str) {
    ok(d, Method::POST, "/profile", json!({ "name": name })).await;
}

async fn my_id(d: &DaemonHandle) -> String {
    ok(d, Method::GET, "/identity", Value::Null).await["id"]
        .as_str()
        .expect("identity id")
        .to_owned()
}

async fn create(d: &DaemonHandle, title: &str) -> String {
    ok(d, Method::POST, "/conversations", json!({ "title": title })).await["id"]
        .as_str()
        .expect("conversation id")
        .to_owned()
}

/// `admin` invites `guest` into `conv`; returns once the guest joined.
async fn invite_and_accept(admin: &DaemonHandle, guest: &DaemonHandle, conv: &str) {
    let v = ok(
        admin,
        Method::POST,
        &format!("/conversations/{conv}/invite"),
        json!({}),
    )
    .await;
    let invite = v["invite"].as_str().expect("invite").to_owned();
    let v = ok(guest, Method::POST, "/accept", json!({ "invite": invite })).await;
    assert_eq!(
        v["id"].as_str(),
        Some(conv),
        "same document id on both sides"
    );
}

async fn send(d: &DaemonHandle, conv: &str, text: &str) {
    ok(
        d,
        Method::POST,
        &format!("/conversations/{conv}/send"),
        json!({ "text": text }),
    )
    .await;
}

async fn conversation(d: &DaemonHandle, conv: &str) -> Value {
    let v = ok(d, Method::GET, "/conversations", Value::Null).await;
    v.as_array()
        .expect("conversation list")
        .iter()
        .find(|c| c["id"] == conv)
        .cloned()
        .unwrap_or(Value::Null)
}

async fn texts(d: &DaemonHandle, conv: &str) -> Vec<String> {
    let v = ok(
        d,
        Method::GET,
        &format!("/conversations/{conv}/messages"),
        Value::Null,
    )
    .await;
    v.as_array()
        .expect("messages")
        .iter()
        .map(|m| m["body"].as_str().unwrap_or_default().to_owned())
        .collect()
}

async fn wait_members(d: &DaemonHandle, conv: &str, n: usize, who: &str) {
    let deadline = tokio::time::Instant::now() + CONVERGE;
    loop {
        let c = conversation(d, conv).await;
        if c["members"].as_array().map(Vec::len) == Some(n) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{who} never saw {n} members: {c}"
        );
        tokio::time::sleep(POLL).await;
    }
}

async fn wait_texts(d: &DaemonHandle, conv: &str, want: &[&str], who: &str) {
    let deadline = tokio::time::Instant::now() + CONVERGE;
    loop {
        let got = texts(d, conv).await;
        if want.iter().all(|w| got.iter().any(|g| g == w)) {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "{who} timed out waiting for {want:?}, has {got:?}"
        );
        tokio::time::sleep(POLL).await;
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
}

/// Every file under the node's two directories, logs included.
fn files_of(d: &DaemonHandle) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(d.root_path(), &mut out);
    walk(d.sbfb_home_path(), &mut out);
    out
}

fn assert_no_plaintext(d: &DaemonHandle, who: &str, secrets: &[&str]) {
    let files = files_of(d);
    assert!(!files.is_empty(), "{who}: no file to scan");
    for f in files {
        let bytes = std::fs::read(&f).unwrap_or_default();
        for s in secrets {
            assert!(
                !bytes.windows(s.len()).any(|w| w == s.as_bytes()),
                "{who}: plaintext {s:?} at rest in {}",
                f.display()
            );
        }
    }
}

/// Make `server` resolvable by id on each client: import a document
/// ticket of the server (see the module docs). Returns the ticket so it
/// can be replayed after a client restart.
async fn introduce(
    server: &DaemonHandle,
    clients: &[&DaemonHandle],
    ticket: Option<String>,
) -> String {
    let ticket = match ticket {
        Some(t) => t,
        None => {
            let (s, v) = server
                .request(
                    Method::POST,
                    "/app/sbfb-ideas/state/intro",
                    Some(json!({ "hello": "mailbox" })),
                )
                .await
                .expect("storage set on the mailbox node");
            assert!(s.is_success(), "storage set: {s} {v}");
            let (s, v) = server
                .request(Method::GET, "/api/daemon/storage/ticket/sbfb-ideas", None)
                .await
                .expect("storage ticket");
            assert_eq!(s, StatusCode::OK, "{v}");
            v["ticket"].as_str().expect("ticket").to_owned()
        }
    };
    for c in clients {
        let (s, v) = c
            .request(
                Method::POST,
                "/api/daemon/storage/join",
                Some(json!({ "app": "sbfb-ideas", "ticket": ticket })),
            )
            .await
            .expect("storage join");
        assert!(s.is_success(), "storage join: {s} {v}");
    }
    ticket
}

async fn set_mailbox(d: &DaemonHandle, on: bool) -> Value {
    ok(d, Method::POST, "/mailbox", json!({ "enabled": on })).await
}

/// Polls `/mailbox` until `conv` is kept or `within` elapses; returns the
/// last view on failure.
async fn kept_within(d: &DaemonHandle, conv: &str, within: Duration) -> Result<(), Value> {
    let deadline = tokio::time::Instant::now() + within;
    loop {
        let v = ok(d, Method::GET, "/mailbox", Value::Null).await;
        let kept = v["conversations"].as_array().is_some_and(|cs| {
            cs.iter().any(|c| {
                c["id"] == conv && c["status"] == "kept" && c["expires_at"].as_u64() > Some(0)
            })
        });
        if kept {
            return Ok(());
        }
        if tokio::time::Instant::now() >= deadline {
            return Err(v);
        }
        tokio::time::sleep(POLL).await;
    }
}

async fn wait_kept(d: &DaemonHandle, conv: &str, who: &str) {
    if let Err(v) = kept_within(d, conv, CONVERGE).await {
        panic!("{who}: conversation never kept by the mailbox: {v}");
    }
}

/// Turn the mailbox off and on again (what a user can do) until a fresh
/// deposit is kept. See the restart step of the mailbox test for why the
/// first attempts after a restart fail in this harness.
async fn reassert_mailbox_until_kept(d: &DaemonHandle, conv: &str, who: &str) -> u32 {
    let deadline = tokio::time::Instant::now() + CONVERGE;
    let mut attempts = 0;
    loop {
        attempts += 1;
        set_mailbox(d, false).await;
        set_mailbox(d, true).await;
        match kept_within(d, conv, Duration::from_secs(5)).await {
            Ok(()) => return attempts,
            Err(v) => assert!(
                tokio::time::Instant::now() < deadline,
                "{who}: never kept again after {attempts} attempts: {v}"
            ),
        }
    }
}

/// (1) 1:1 conversation between two processes, both directions, and no
/// plaintext at rest on either node afterwards.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chat_one_to_one_both_ways_between_processes() {
    let mut alice = node(&[], false).await;
    let mut bob = node(&[], false).await;
    set_name(&alice, "Alice").await;
    set_name(&bob, "Bob").await;

    let conv = create(&alice, "Tete-a-tete").await;
    invite_and_accept(&alice, &bob, &conv).await;
    wait_members(&alice, &conv, 2, "alice").await;
    wait_members(&bob, &conv, 2, "bob").await;

    send(&alice, &conv, "a-to-b secret-k3x9").await;
    send(&bob, &conv, "b-to-a secret-q7w2").await;
    let both = ["a-to-b secret-k3x9", "b-to-a secret-q7w2"];
    wait_texts(&bob, &conv, &both, "bob").await;
    wait_texts(&alice, &conv, &both, "alice").await;

    // Authorship survives the trip: each side sees the other's message as
    // not its own, under the other's name.
    let v = ok(
        &bob,
        Method::GET,
        &format!("/conversations/{conv}/messages"),
        Value::Null,
    )
    .await;
    let from_alice = v
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["body"] == "a-to-b secret-k3x9")
        .unwrap();
    assert_eq!(from_alice["mine"], false);
    assert_eq!(from_alice["from_name"], "Alice");

    alice.shutdown().await.unwrap();
    bob.shutdown().await.unwrap();
    for (d, who) in [(&alice, "alice"), (&bob, "bob")] {
        assert_no_plaintext(d, who, &["secret-k3x9", "secret-q7w2"]);
    }
}

/// (2) Group of three processes; the admin removes one member, who then
/// cannot read anything sent afterwards while the remaining two can.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chat_group_of_three_removed_member_cannot_read_new_messages() {
    let mut alice = node(&[], false).await;
    let mut bob = node(&[], false).await;
    let mut carol = node(&[], false).await;
    for (d, n) in [(&alice, "Alice"), (&bob, "Bob"), (&carol, "Carol")] {
        set_name(d, n).await;
    }

    let conv = create(&alice, "Groupe").await;
    invite_and_accept(&alice, &bob, &conv).await;
    invite_and_accept(&alice, &carol, &conv).await;
    for (d, who) in [(&alice, "alice"), (&bob, "bob"), (&carol, "carol")] {
        wait_members(d, &conv, 3, who).await;
    }

    // Everyone reads everyone before the removal: Carol is connected and
    // holds the session keys, so what follows is exclusion, not absence.
    send(&alice, &conv, "before-a").await;
    send(&bob, &conv, "before-b").await;
    send(&carol, &conv, "before-c").await;
    let before = ["before-a", "before-b", "before-c"];
    for (d, who) in [(&alice, "alice"), (&bob, "bob"), (&carol, "carol")] {
        wait_texts(d, &conv, &before, who).await;
    }

    let carol_id = my_id(&carol).await;
    ok(
        &alice,
        Method::POST,
        &format!("/conversations/{conv}/remove"),
        json!({ "member": carol_id }),
    )
    .await;

    // The admin speaks first (she rotated synchronously); Bob answers once
    // he has her message, i.e. once he followed the conversation to its
    // new document. The roster is checked only after the watch below, so a
    // removal that silently keeps the member fails on what matters: she
    // reads what was sent after it.
    send(&alice, &conv, "after-a secret-r4m1").await;
    wait_texts(&bob, &conv, &["after-a secret-r4m1"], "bob").await;
    send(&bob, &conv, "after-b secret-r4m2").await;
    let after = ["after-a secret-r4m1", "after-b secret-r4m2"];
    wait_texts(&alice, &conv, &after, "alice").await;
    wait_texts(&bob, &conv, &after, "bob").await;

    // The remaining members have both messages; give Carol the same chance
    // and more, then check she read neither.
    let deadline = tokio::time::Instant::now() + REMOVED_WATCH;
    while tokio::time::Instant::now() < deadline {
        let got = texts(&carol, &conv).await;
        assert!(
            !got.iter().any(|t| t.contains("secret-r4m")),
            "removed member read a message sent after her removal: {got:?}"
        );
        tokio::time::sleep(POLL).await;
    }
    wait_members(&alice, &conv, 2, "alice").await;
    wait_members(&bob, &conv, 2, "bob").await;
    let c = conversation(&carol, &conv).await;
    assert_eq!(c["removed"], true, "carol must know she was removed: {c}");
    let got = texts(&carol, &conv).await;
    for b in before {
        assert!(
            got.iter().any(|t| t == b),
            "carol lost history {b}: {got:?}"
        );
    }

    for d in [&mut alice, &mut bob, &mut carol] {
        d.shutdown().await.unwrap();
    }
}

/// (3) Offline delivery through a mailbox node: A and B opt in, B's
/// process stops, A sends, A's process stops, B boots again from the same
/// directories and receives the message — from the mailbox node alone,
/// which never held plaintext.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn chat_offline_delivery_through_a_mailbox_node() {
    step("start");
    let mut mailbox = node(&[], true).await;
    let mbox_id = mailbox.node_id.clone();
    let mut alice = node(&[&mbox_id], false).await;
    let mut bob = node(&[&mbox_id], false).await;
    let ticket = introduce(&mailbox, &[&alice, &bob], None).await;
    step("introduced");
    set_name(&alice, "Alice").await;
    set_name(&bob, "Bob").await;

    let conv = create(&alice, "Boite").await;
    invite_and_accept(&alice, &bob, &conv).await;
    wait_members(&alice, &conv, 2, "alice").await;
    wait_members(&bob, &conv, 2, "bob").await;
    // Both have spoken: every session key is exchanged.
    send(&alice, &conv, "hello-a").await;
    send(&bob, &conv, "hello-b").await;
    wait_texts(&alice, &conv, &["hello-a", "hello-b"], "alice").await;
    wait_texts(&bob, &conv, &["hello-a", "hello-b"], "bob").await;
    step("paired");

    for (d, who) in [(&alice, "alice"), (&bob, "bob")] {
        let v = set_mailbox(d, true).await;
        assert_eq!(v["enabled"], true, "{who}: {v}");
        assert_eq!(v["servers"][0], mbox_id.as_str(), "{who}: {v}");
        wait_kept(d, &conv, who).await;
    }

    // Bob goes offline, then Alice writes: only the mailbox can take it.
    bob.shutdown().await.unwrap();
    step("both kept, bob stopped");
    let secret = "while-you-were-out secret-m8z5";
    send(&alice, &conv, secret).await;
    // Live sync pushes the entry to the replica within milliseconds; the
    // margin keeps a slow machine from stopping Alice before it lands.
    tokio::time::sleep(Duration::from_secs(5)).await;
    alice.shutdown().await.unwrap();

    // Bob comes back on the same directories. Alice is gone for good.
    let bob_id = bob.node_id.clone();
    step("alice stopped");
    bob.restart().await.expect("bob restarts");
    step("bob restarted");
    assert_eq!(bob.node_id, bob_id, "same node key after restart");
    // Nothing reachable yet (Alice dead, mailbox not resolvable): the
    // message was never on Bob's node.
    let local = texts(&bob, &conv).await;
    assert!(
        !local.iter().any(|t| t == secret),
        "bob had it before reaching the mailbox: {local:?}"
    );
    step(&format!("bob local history after restart: {local:?}"));

    introduce(&mailbox, &[&bob], Some(ticket)).await;
    // At boot Bob already re-dialled the mailbox BY ID (the reopened
    // documents remember it as a sync peer) before the introduction
    // above: that address lookup can only ask the dead pkarr relay and
    // fails ~2 s later, and every dial issued meanwhile — the deposit of
    // the chat state loaded above included — joins it and fails too. A
    // failed deposit is retried only after MAILBOX_RETRY_SECS (10 min),
    // so re-assert the opt-in (off, on) until a fresh deposit resolves
    // the mailbox through the introduced address.
    let attempts = reassert_mailbox_until_kept(&bob, &conv, "bob after restart").await;
    step(&format!(
        "bob kept again after {attempts} opt-in attempt(s)"
    ));
    // Only the new message is required: what Bob had received less than
    // ~1 s before the hard stop may have lost its content locally, and an
    // entry his store still lists is not fetched again (see the Phase G
    // integration commit body).
    wait_texts(&bob, &conv, &[secret], "bob after restart").await;
    step("bob received");

    // The mailbox node held the conversation and never its plaintext.
    mailbox.shutdown().await.unwrap();
    bob.shutdown().await.unwrap();
    let registry: String = files_of(&mailbox)
        .into_iter()
        .filter(|p| p.file_name().is_some_and(|n| n == "mailbox.json"))
        .map(|p| std::fs::read_to_string(p).unwrap_or_default())
        .collect();
    assert!(
        registry.contains(&conv),
        "mailbox registry lacks the conversation: {registry}"
    );
    assert_no_plaintext(&mailbox, "mailbox", &["secret-m8z5", "hello-a", "hello-b"]);
}

/// Progress line with elapsed time, shown on failure (nextest captures it).
fn step(what: &str) {
    use std::sync::OnceLock;
    static T0: OnceLock<std::time::Instant> = OnceLock::new();
    let t0 = T0.get_or_init(std::time::Instant::now);
    eprintln!("[{:>6.1}s] {what}", t0.elapsed().as_secs_f64());
}
