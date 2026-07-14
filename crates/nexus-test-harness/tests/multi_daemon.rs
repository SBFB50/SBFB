// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon integration tests (Sprint 33 Phase C, rows 30-33).
//!
//! These tests spawn real `nexus-shell-daemon` processes. They
//! require the binary to be built first:
//!   cargo build -p nexus-shell-daemon
//!
//! Relay-gated class (S81 Phase K libellé): the tests that require
//! iroh P2P relay connectivity are NOT `#[ignore]`-marked — they
//! SELF-SKIP at runtime unless `SBFB_INTEGRATION=1`, so a default CI
//! run reports them green WITHOUT exercising any network path. Real
//! coverage of this class comes from the dedicated nightly/manual
//! workflow (`.github/workflows/integration-nightly.yml`) and the
//! live T2 acceptance harness; never count a default green run as
//! relay coverage. Sprint 82 Phase D repaired the 5 deterministic
//! test-rots the S81 A3 first real run exposed (artefact
//! `sprint81_a3_integration_run_098.txt`): the blob test (now
//! `test_blob_serve_local_zip_roundtrip`, de-gated — purely local, so
//! it is NOT part of the self-skip class) published a raw blob against
//! the zip-only blob-serve (S12), and the four feed tests (three in
//! this file, one in `nexus-coordinator-rs`) omitted the
//! `x-sbfb-feed-internal` header required since S65 (ace05b0). The
//! former product-signal `test_cross_daemon_gossip_exchange` now
//! converges in the current tree (fresh run HEAD 2931b82, 4/4 in
//! 2.3-3.4s vs 33s timeout under 0.98) — attributed to the S81
//! transport delta (iroh 1.0.1); the loopback measurement does not
//! isolate which S81 feature closed it. The negative auth-tier guard
//! is covered hermetically by
//! `feed_insert_rejects_without_internal_header` in `nexus-shell-daemon`.

use nexus_test_harness::DaemonCluster;

fn integration_enabled() -> bool {
    std::env::var("SBFB_INTEGRATION").unwrap_or_default() == "1"
}

/// Build a minimal in-memory zip archive (STORED, no compression) so a
/// test can publish a real archive to blob-serve, which is zip-only
/// since S12. Mirrors the daemon's own `make_zip` test helper.
fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in files {
        writer.start_file(*name, options).expect("zip start_file");
        writer.write_all(data).expect("zip write_all");
    }
    writer.finish().expect("zip finish").into_inner()
}

/// Row 30 — smoke test: spawn 2 daemons, health check both,
/// verify they have distinct node_ids.
#[tokio::test]
async fn test_two_daemons_boot_and_respond() {
    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");

    assert_eq!(cluster.nodes.len(), 2);

    let a = &cluster.nodes[0];
    let b = &cluster.nodes[1];

    assert!(
        a.health_check().await.expect("health a"),
        "daemon a healthy"
    );
    assert!(
        b.health_check().await.expect("health b"),
        "daemon b healthy"
    );

    assert_ne!(
        a.node_id, b.node_id,
        "two daemons must have distinct node_ids"
    );
    assert_ne!(
        a.http_port, b.http_port,
        "two daemons must bind to different ports"
    );

    assert!(!a.node_id.is_empty(), "node_id a must be non-empty");
    assert!(!b.node_id.is_empty(), "node_id b must be non-empty");

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Row 31 — cross-daemon discovery: verify both daemons are
/// reachable via their info endpoint and report distinct iroh
/// endpoint identities.
///
/// Full P2P discovery (pkarr/relay) requires network access and
/// is tested only when `SBFB_INTEGRATION=1`.
#[tokio::test]
async fn test_cross_daemon_discovery() {
    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");

    let info_a = cluster.nodes[0].get_info().await.expect("info a");
    let info_b = cluster.nodes[1].get_info().await.expect("info b");

    let id_a = info_a["node_id"].as_str().unwrap_or_default();
    let id_b = info_b["node_id"].as_str().unwrap_or_default();

    assert_ne!(id_a, id_b, "info endpoints report distinct node_ids");
    assert!(!id_a.is_empty(), "node_id a non-empty");
    assert!(!id_b.is_empty(), "node_id b non-empty");

    assert!(
        info_a["daemon_version"].is_string(),
        "info a has daemon_version"
    );
    assert!(
        info_b["daemon_version"].is_string(),
        "info b has daemon_version"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Row 32 — blob round-trip through blob-serve: publish a real zip
/// archive on a daemon and fetch an inner file back via blob-serve.
///
/// Purely LOCAL single-daemon path (no P2P/relay), so the whole
/// round-trip runs unconditionally. blob-serve decompresses a zip and
/// serves inner paths; it rejects a non-zip body with 400 (S12
/// zip-archive-only contract) — publishing a real zip is the repair
/// for the S81 A3 test-rot (a raw blob was published, then the GET
/// asserted 200). Renamed from `test_cross_daemon_blob_transfer` in
/// S82 Phase D: the old name claimed a cross-daemon iroh fetch this
/// test never performed (it always spawned a single daemon).
#[tokio::test]
async fn test_blob_serve_local_zip_roundtrip() {
    let mut cluster = DaemonCluster::spawn(1).await.expect("spawn daemon");

    let daemon = &cluster.nodes[0];
    let client = reqwest::Client::new();

    let file_bytes: &[u8] = b"hello from SBFB integration test";
    let zip_bytes = make_zip(&[("test.txt", file_bytes)]);
    let resp = client
        .post(format!("{}/api/daemon/publish-blob", daemon.http_url()))
        .header("X-SBFB-Token", &daemon.auth_token)
        .header("Host", format!("127.0.0.1:{}", daemon.http_port))
        .body(zip_bytes)
        .send()
        .await
        .expect("publish-blob request");

    assert!(
        resp.status().is_success(),
        "publish-blob returned {}",
        resp.status()
    );

    let body: serde_json::Value = resp.json().await.expect("parse publish-blob response");
    let hash = body["hash"].as_str().expect("response has hash field");
    assert!(!hash.is_empty(), "hash must be non-empty");

    // Local blob-serve round-trip: fetch the inner file and check the
    // exact bytes come back (200 + body). No relay, so no gate.
    let blob_url = format!("{}/blob-serve/{}/test.txt", daemon.http_url(), hash);
    let blob_resp = client.get(&blob_url).send().await.expect("blob-serve GET");
    assert_eq!(blob_resp.status().as_u16(), 200, "blob-serve returns 200");
    let served = blob_resp.bytes().await.expect("blob-serve body");
    assert_eq!(
        served.as_ref(),
        file_bytes,
        "blob-serve must return the exact file bytes from the zip"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Sprint 57 Phase A — E2E gossip exchange: daemon B subscribes
/// to daemon A as curator, A publishes a project, B discovers it
/// via browse after gossip relay.
///
/// Requires iroh relay connectivity (`SBFB_INTEGRATION=1`).
#[tokio::test]
async fn test_cross_daemon_gossip_exchange() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable gossip E2E");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");

    let node_a_id = cluster.nodes[0].node_id.clone();

    // B subscribes to A as curator
    let sub_resp = cluster.nodes[1]
        .subscribe_curator(&node_a_id)
        .await
        .expect("subscribe curator");
    assert!(
        sub_resp["subscribed_curators"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "B must have A in subscribed curators"
    );

    // A publishes a project
    let pub_resp = cluster.nodes[0]
        .publish_project("e2e-gossip-test")
        .await
        .expect("publish project");
    assert!(
        pub_resp.get("published").is_some() || pub_resp.get("project_name").is_some(),
        "publish must return confirmation"
    );

    // Wait for gossip relay (poll B's browse for up to 30s)
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut found = false;
    while tokio::time::Instant::now() < deadline {
        let entries = cluster.nodes[1].browse_projects().await.unwrap_or_default();
        if entries.iter().any(|e| {
            e["project_name"]
                .as_str()
                .map(|n| n == "e2e-gossip-test")
                .unwrap_or(false)
        }) {
            found = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    assert!(
        found,
        "B must discover A's published project via gossip within 30s"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Sprint 58 Phase D — cross-daemon storage sync: daemon A writes
/// an idea, daemon B imports the ticket and receives the entry via
/// iroh-docs replication.
///
/// Requires iroh relay connectivity (`SBFB_INTEGRATION=1`).
#[tokio::test]
async fn test_cross_daemon_storage_sync() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable storage sync E2E");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");
    let client = reqwest::Client::new();

    // Daemon A: write an idea via the storage API
    // Key contains a slash — must be percent-encoded in path (same as
    // the bridge frontend's encodeURIComponent).
    let set_resp = client
        .post(format!(
            "{}/app/sbfb-ideas/state/ideas%2Ftest-sync-1",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "title": "E2E sync test idea",
            "description": "Written by daemon A",
            "author": "test-pubkey-a",
            "created_at": "2026-05-10T12:00:00Z"
        }))
        .send()
        .await
        .expect("storage_set on daemon A");
    assert!(
        set_resp.status().is_success(),
        "storage_set returned {}",
        set_resp.status()
    );

    // Daemon A: get the storage ticket
    let ticket_resp = client
        .get(format!(
            "{}/api/daemon/storage/ticket/sbfb-ideas",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .send()
        .await
        .expect("storage_ticket on daemon A");
    assert!(ticket_resp.status().is_success());
    let ticket_body: serde_json::Value = ticket_resp.json().await.expect("parse ticket response");
    let ticket = ticket_body["ticket"]
        .as_str()
        .expect("ticket field present");
    assert!(!ticket.is_empty(), "ticket must be non-empty");

    // Daemon B: join the storage namespace
    let join_resp = client
        .post(format!(
            "{}/api/daemon/storage/join",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "app": "sbfb-ideas",
            "ticket": ticket
        }))
        .send()
        .await
        .expect("storage_join on daemon B");
    assert!(
        join_resp.status().is_success(),
        "storage_join returned {}",
        join_resp.status()
    );

    // Poll daemon B for the synced entry (up to 30s)
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut found = false;
    while tokio::time::Instant::now() < deadline {
        let list_resp = client
            .get(format!(
                "{}/app/sbfb-ideas/state?prefix=ideas/",
                cluster.nodes[1].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
            .send()
            .await;

        if let Ok(resp) = list_resp
            && resp.status().is_success()
        {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            let entries = body["entries"].as_array();
            if let Some(entries) = entries
                && entries.iter().any(|e| {
                    e["key"]
                        .as_str()
                        .map(|k| k == "ideas/test-sync-1")
                        .unwrap_or(false)
                })
            {
                found = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    assert!(
        found,
        "daemon B must see ideas/test-sync-1 via iroh-docs sync within 30s"
    );

    // Verify version counter incremented on daemon B
    let version_resp = client
        .get(format!(
            "{}/api/daemon/storage/sbfb-ideas/version",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .send()
        .await
        .expect("storage_version on daemon B");
    assert!(version_resp.status().is_success());
    let version_body: serde_json::Value =
        version_resp.json().await.expect("parse version response");
    let version = version_body["version"].as_u64().unwrap_or(0);
    assert!(
        version >= 1,
        "version counter must have incremented (got {})",
        version
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Sprint 62 Phase C — cross-daemon feed sync: daemon A inserts
/// 3 feed operations, daemon B joins via DocTicket, and observes
/// all 3 entries via iroh-docs replication.
///
/// Requires iroh relay connectivity (`SBFB_INTEGRATION=1`).
#[tokio::test]
async fn test_cross_daemon_feed_sync() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable feed sync E2E");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");
    let client = reqwest::Client::new();

    // Daemon A: insert 3 feed operations
    for i in 0..3u8 {
        let project_id = format!("{:02x}", 0xa0 + i).repeat(32);
        let resp = client
            .post(format!(
                "{}/api/daemon/feed/insert",
                cluster.nodes[0].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
            .header("Content-Type", "application/json")
            // `/api/daemon/feed/insert` is internal-only since S65 (ace05b0,
            // P2-FEED-INSERT-NO-AUTH-TIER): 403 without this header. The
            // loopback harness holds the daemon's own bearer and drives the
            // sanctioned internal insert path to seed the sync fixtures.
            .header("x-sbfb-feed-internal", "1")
            .json(&serde_json::json!({
                "op": {
                    "op_type": "ReleasePublished",
                    "project_id": project_id,
                    "repo_url": "https://github.com/org/app",
                    "commit_sha": "a".repeat(40),
                    "artifact_hash": "b".repeat(64),
                    "provenance_hash": "c".repeat(64),
                    "is_open_source": true
                }
            }))
            .send()
            .await
            .expect("feed insert request");
        assert!(
            resp.status().is_success(),
            "feed insert {} returned {}",
            i,
            resp.status()
        );
    }

    // Verify daemon A has 3 entries
    let status_a: serde_json::Value = client
        .get(format!(
            "{}/api/daemon/feed/status",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .send()
        .await
        .expect("feed status A")
        .json()
        .await
        .expect("parse feed status A");
    assert_eq!(
        status_a["count"].as_u64(),
        Some(3),
        "daemon A must have 3 feed entries"
    );

    // Daemon A: get feed ticket
    let ticket_resp: serde_json::Value = client
        .get(format!(
            "{}/api/daemon/feed/ticket",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .send()
        .await
        .expect("feed ticket")
        .json()
        .await
        .expect("parse feed ticket");
    let ticket = ticket_resp["ticket"]
        .as_str()
        .expect("ticket field present");
    assert!(!ticket.is_empty(), "ticket must be non-empty");

    // Daemon B: join feed namespace
    let join_resp = client
        .post(format!(
            "{}/api/daemon/feed/join",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "ticket": ticket }))
        .send()
        .await
        .expect("feed join");
    assert!(
        join_resp.status().is_success(),
        "feed join returned {}",
        join_resp.status()
    );

    // Poll daemon B for synced entries (up to 30s)
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut synced = false;
    while tokio::time::Instant::now() < deadline {
        let resp = client
            .get(format!(
                "{}/api/daemon/feed/status",
                cluster.nodes[1].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
            .send()
            .await;

        if let Ok(resp) = resp
            && resp.status().is_success()
        {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if body["count"].as_u64() == Some(3) {
                synced = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    assert!(
        synced,
        "daemon B must see 3 feed entries via iroh-docs sync within 30s"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Sprint 62 Phase C — feed offline catch-up: daemon A publishes
/// N entries, daemon B joins AFTER publication and catches up the
/// full history via iroh-docs range reconciliation.
///
/// Requires iroh relay connectivity (`SBFB_INTEGRATION=1`).
#[tokio::test]
async fn test_feed_offline_catchup() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable feed offline catchup E2E");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");
    let client = reqwest::Client::new();

    // Daemon A: insert 5 operations BEFORE B joins
    for i in 0..5u8 {
        let project_id = format!("{:02x}", 0xc0 + i).repeat(32);
        let resp = client
            .post(format!(
                "{}/api/daemon/feed/insert",
                cluster.nodes[0].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
            .header("Content-Type", "application/json")
            // `/api/daemon/feed/insert` is internal-only since S65 (ace05b0,
            // P2-FEED-INSERT-NO-AUTH-TIER): 403 without this header. The
            // loopback harness holds the daemon's own bearer and drives the
            // sanctioned internal insert path to seed the sync fixtures.
            .header("x-sbfb-feed-internal", "1")
            .json(&serde_json::json!({
                "op": {
                    "op_type": "ReleasePublished",
                    "project_id": project_id,
                    "repo_url": "https://github.com/org/offline-test",
                    "commit_sha": "d".repeat(40),
                    "artifact_hash": "e".repeat(64),
                    "provenance_hash": "f".repeat(64),
                    "is_open_source": true
                }
            }))
            .send()
            .await
            .expect("feed insert");
        assert!(resp.status().is_success());
    }

    // Daemon A: get feed ticket
    let ticket_resp: serde_json::Value = client
        .get(format!(
            "{}/api/daemon/feed/ticket",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .send()
        .await
        .expect("feed ticket")
        .json()
        .await
        .expect("parse ticket");
    let ticket = ticket_resp["ticket"].as_str().expect("ticket");

    // Daemon B: join AFTER all entries published (offline catch-up)
    let join_resp = client
        .post(format!(
            "{}/api/daemon/feed/join",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "ticket": ticket }))
        .send()
        .await
        .expect("feed join");
    assert!(join_resp.status().is_success());

    // Poll until B catches up
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut caught_up = false;
    while tokio::time::Instant::now() < deadline {
        let resp = client
            .get(format!(
                "{}/api/daemon/feed/status",
                cluster.nodes[1].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
            .send()
            .await;

        if let Ok(resp) = resp
            && resp.status().is_success()
        {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if body["count"].as_u64() == Some(5) {
                // Verify author stats match
                let authors = body["authors"].as_array();
                if let Some(authors) = authors
                    && authors.len() == 1
                    && authors[0]["count"].as_u64() == Some(5)
                {
                    caught_up = true;
                    break;
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }

    assert!(
        caught_up,
        "daemon B must catch up all 5 entries from daemon A within 30s"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Sprint 62 Phase C — feed replay idempotent: daemon B joins,
/// syncs, then re-joins the same namespace. The second join must
/// NOT create duplicate entries in the local feed.
///
/// Requires iroh relay connectivity (`SBFB_INTEGRATION=1`).
#[tokio::test]
async fn test_feed_replay_idempotent() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to enable feed replay idempotent E2E");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");
    let client = reqwest::Client::new();

    // Daemon A: insert 2 feed operations
    for i in 0..2u8 {
        let project_id = format!("{:02x}", 0xe0 + i).repeat(32);
        let resp = client
            .post(format!(
                "{}/api/daemon/feed/insert",
                cluster.nodes[0].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
            .header("Content-Type", "application/json")
            // `/api/daemon/feed/insert` is internal-only since S65 (ace05b0,
            // P2-FEED-INSERT-NO-AUTH-TIER): 403 without this header. The
            // loopback harness holds the daemon's own bearer and drives the
            // sanctioned internal insert path to seed the sync fixtures.
            .header("x-sbfb-feed-internal", "1")
            .json(&serde_json::json!({
                "op": {
                    "op_type": "ReleasePublished",
                    "project_id": project_id,
                    "repo_url": "https://github.com/org/replay-test",
                    "commit_sha": "1".repeat(40),
                    "artifact_hash": "2".repeat(64),
                    "provenance_hash": "3".repeat(64),
                    "is_open_source": true
                }
            }))
            .send()
            .await
            .expect("feed insert");
        assert!(resp.status().is_success());
    }

    let ticket_resp: serde_json::Value = client
        .get(format!(
            "{}/api/daemon/feed/ticket",
            cluster.nodes[0].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[0].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[0].http_port))
        .send()
        .await
        .expect("feed ticket")
        .json()
        .await
        .expect("parse ticket");
    let ticket = ticket_resp["ticket"].as_str().expect("ticket");

    // First join — daemon B syncs
    let join1 = client
        .post(format!(
            "{}/api/daemon/feed/join",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "ticket": ticket }))
        .send()
        .await
        .expect("feed join 1");
    assert!(join1.status().is_success());

    // Wait for first sync
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut first_sync = false;
    while tokio::time::Instant::now() < deadline {
        let resp = client
            .get(format!(
                "{}/api/daemon/feed/status",
                cluster.nodes[1].http_url()
            ))
            .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
            .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
            .send()
            .await;
        if let Ok(resp) = resp
            && resp.status().is_success()
        {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if body["count"].as_u64() == Some(2) {
                first_sync = true;
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(first_sync, "first sync must complete within 30s");

    // Second join — re-import same ticket (replay)
    let join2 = client
        .post(format!(
            "{}/api/daemon/feed/join",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({ "ticket": ticket }))
        .send()
        .await
        .expect("feed join 2");
    assert!(join2.status().is_success());

    // Wait a bit for any duplicate processing
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Verify count is still 2 (no duplicates)
    let final_status: serde_json::Value = client
        .get(format!(
            "{}/api/daemon/feed/status",
            cluster.nodes[1].http_url()
        ))
        .header("X-SBFB-Token", &cluster.nodes[1].auth_token)
        .header("Host", format!("127.0.0.1:{}", cluster.nodes[1].http_port))
        .send()
        .await
        .expect("final feed status")
        .json()
        .await
        .expect("parse final status");

    assert_eq!(
        final_status["count"].as_u64(),
        Some(2),
        "replay must not create duplicates: expected 2, got {:?}",
        final_status["count"]
    );

    cluster.shutdown().await.expect("graceful shutdown");
}

/// Row 33 — cross-daemon task stub: verify the daemon exposes
/// its API surface and can accept authenticated requests.
///
/// Real task dispatch (coordinator → worker → result) is out of
/// scope for daemon-only tests — it requires a running
/// coordinator and worker. This test verifies the daemon's
/// /info endpoint responds with the expected structure.
#[tokio::test]
async fn test_cross_daemon_task_stub() {
    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2 daemons");

    for (i, daemon) in cluster.nodes.iter().enumerate() {
        let info = daemon.get_info().await.unwrap_or_else(|e| {
            panic!("daemon {} info failed: {}", i, e);
        });

        assert!(info["node_id"].is_string(), "daemon {} has node_id", i);
        assert!(
            info["daemon_version"].is_string(),
            "daemon {} has daemon_version",
            i
        );

        let health = daemon.health_check().await.expect("health check");
        assert!(health, "daemon {} responds healthy", i);
    }

    assert_ne!(
        cluster.nodes[0].node_id, cluster.nodes[1].node_id,
        "two daemons have distinct identities for task routing"
    );

    cluster.shutdown().await.expect("graceful shutdown");
}
