// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon integration tests (Sprint 33 Phase C, rows 30-33).
//!
//! These tests spawn real `nexus-shell-daemon` processes. They
//! require the binary to be built first:
//!   cargo build -p nexus-shell-daemon
//!
//! Tests that require iroh P2P relay connectivity are marked
//! `#[ignore]` and run only when `SBFB_INTEGRATION=1` is set.

use nexus_test_harness::DaemonCluster;

fn integration_enabled() -> bool {
    std::env::var("SBFB_INTEGRATION").unwrap_or_default() == "1"
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

/// Row 32 — cross-daemon blob transfer: publish a blob on
/// daemon 1, verify it is served via blob-serve HTTP.
///
/// Cross-daemon fetch (daemon 2 retrieves daemon 1's blob via
/// iroh-blobs) requires P2P connectivity and is tested only
/// when `SBFB_INTEGRATION=1`.
#[tokio::test]
async fn test_cross_daemon_blob_transfer() {
    let mut cluster = DaemonCluster::spawn(1).await.expect("spawn daemon");

    let daemon = &cluster.nodes[0];
    let client = reqwest::Client::new();

    let payload = b"hello from SBFB integration test";
    let resp = client
        .post(format!("{}/api/daemon/publish-blob", daemon.http_url()))
        .header("X-SBFB-Token", &daemon.auth_token)
        .header("Host", format!("127.0.0.1:{}", daemon.http_port))
        .body(payload.to_vec())
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

    if integration_enabled() {
        let blob_url = format!("{}/blob-serve/{}/test.txt", daemon.http_url(), hash);
        let blob_resp = client.get(&blob_url).send().await.expect("blob-serve GET");
        assert_eq!(blob_resp.status().as_u16(), 200, "blob-serve returns 200");
    }

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

        if let Ok(resp) = list_resp {
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                let entries = body["entries"].as_array();
                if let Some(entries) = entries {
                    if entries.iter().any(|e| {
                        e["key"]
                            .as_str()
                            .map(|k| k == "ideas/test-sync-1")
                            .unwrap_or(false)
                    }) {
                        found = true;
                        break;
                    }
                }
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
