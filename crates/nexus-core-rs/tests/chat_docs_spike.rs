// SPDX-License-Identifier: AGPL-3.0-or-later
//! Chat v1 spike A (GO/NO-GO), transport half: one iroh-docs document per
//! conversation carries opaque ciphertext between members, and a read-only
//! replica (the anchor mailbox) can store it and serve it to a member who
//! was never online at the same time as the sender.

use std::time::Duration;

use nexus_core_rs::{BlobsClient, DocHandle, DocsClient, Node, create_node};

const WAIT: Duration = Duration::from_secs(30);

async fn has_all(node: &Node, doc: &DocHandle, want: &[(&[u8], &[u8])]) -> bool {
    let blobs = BlobsClient::new(node.blobs_store());
    let Ok(entries) = doc.get_many_by_prefix(b"m/").await else {
        return false;
    };
    for (key, value) in want {
        let Some(entry) = entries.iter().find(|e| e.key() == *key) else {
            return false;
        };
        match blobs.get_bytes(*entry.content_hash().as_bytes()).await {
            Ok(bytes) if bytes == *value => {}
            _ => return false,
        }
    }
    true
}

async fn wait_for(node: &Node, doc: &DocHandle, want: &[(&[u8], &[u8])], who: &str) {
    let deadline = tokio::time::Instant::now() + WAIT;
    while tokio::time::Instant::now() < deadline {
        if has_all(node, doc, want).await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    panic!("{who} did not converge within {WAIT:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn conversation_doc_syncs_members_and_mailbox_relays_offline() {
    let alice = create_node().await.unwrap();
    let bob = create_node().await.unwrap();
    let mailbox = create_node().await.unwrap();

    let a_docs = DocsClient::new(alice.docs());
    let a_author = a_docs.author_create().await.unwrap();
    let a_doc = a_docs.create_doc().await.unwrap();
    let write_ticket = a_doc.share_write().await.unwrap();
    let read_ticket = a_doc.share_read().await.unwrap();

    let b_docs = DocsClient::new(bob.docs());
    let b_author = b_docs.author_create().await.unwrap();
    let b_doc = b_docs.import_ticket(write_ticket.clone()).await.unwrap();
    let m_docs = DocsClient::new(mailbox.docs());
    let m_author = m_docs.author_create().await.unwrap();
    let m_doc = m_docs.import_ticket(read_ticket).await.unwrap();

    // Members exchange opaque ciphertext both ways.
    a_doc
        .set(a_author, b"m/a/1".to_vec(), b"\x01cipher-a1".to_vec())
        .await
        .unwrap();
    b_doc
        .set(b_author, b"m/b/1".to_vec(), b"\x02cipher-b1".to_vec())
        .await
        .unwrap();
    let round1: &[(&[u8], &[u8])] = &[(b"m/a/1", b"\x01cipher-a1"), (b"m/b/1", b"\x02cipher-b1")];
    wait_for(&alice, &a_doc, round1, "alice").await;
    wait_for(&bob, &b_doc, round1, "bob").await;
    wait_for(&mailbox, &m_doc, round1, "mailbox").await;

    // The mailbox holds a READ capability: it cannot author entries.
    assert!(
        m_doc
            .set(m_author, b"m/x/1".to_vec(), b"forged".to_vec())
            .await
            .is_err(),
        "a read-only replica must not be able to write"
    );

    // Alice writes while Carol has never been online, then goes away.
    a_doc
        .set(a_author, b"m/a/2".to_vec(), b"\x01cipher-a2".to_vec())
        .await
        .unwrap();
    let round2: &[(&[u8], &[u8])] = &[
        (b"m/a/1", b"\x01cipher-a1"),
        (b"m/b/1", b"\x02cipher-b1"),
        (b"m/a/2", b"\x01cipher-a2"),
    ];
    wait_for(&mailbox, &m_doc, round2, "mailbox").await;
    alice.shutdown().await.unwrap();
    bob.shutdown().await.unwrap();

    // Carol joins later with the member ticket and only the mailbox online:
    // she must get the whole history from the read-only replica.
    let carol = create_node().await.unwrap();
    let c_docs = DocsClient::new(carol.docs());
    let c_doc = c_docs.import_ticket(write_ticket).await.unwrap();
    c_doc
        .start_sync(vec![mailbox.endpoint().addr()])
        .await
        .unwrap();
    wait_for(&carol, &c_doc, round2, "carol via mailbox").await;

    carol.shutdown().await.unwrap();
    mailbox.shutdown().await.unwrap();
}
