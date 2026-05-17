// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon E2E integration tests (gated behind SBFB_INTEGRATION).

use nexus_test_harness::DaemonCluster;

fn integration_enabled() -> bool {
    std::env::var("SBFB_INTEGRATION").is_ok()
}

#[tokio::test]
async fn test_new_node_full_sync_and_verify() {
    if !integration_enabled() {
        eprintln!("skipping: set SBFB_INTEGRATION=1 to run");
        return;
    }

    let mut cluster = DaemonCluster::spawn(2).await.expect("spawn 2-node cluster");
    let d1 = &cluster.nodes[0];
    let d2 = &cluster.nodes[1];

    let client = reqwest::Client::new();

    // Insert 3 feed operations on daemon 1
    for i in 0..3 {
        let resp = client
            .post(format!("{}/api/daemon/feed/insert", d1.http_url()))
            .header("X-SBFB-Token", &d1.auth_token)
            .header("Host", format!("127.0.0.1:{}", d1.http_port))
            .json(&serde_json::json!({
                "op": {
                    "op_type": "ReleasePublished",
                    "project_id": format!("{:0>64}", format!("project{i}")),
                    "repo_url": format!("https://github.com/org/app{i}"),
                    "commit_sha": format!("{:0>40}", format!("commit{i}")),
                    "artifact_hash": format!("{:0>64}", format!("artifact{i}")),
                    "provenance_hash": format!("{:0>64}", format!("prov{i}")),
                    "is_open_source": true,
                }
            }))
            .send()
            .await
            .unwrap_or_else(|e| panic!("feed insert {i} failed: {e}"));
        assert!(
            resp.status().is_success(),
            "feed insert {i} returned {}",
            resp.status()
        );
    }

    // Get feed ticket from daemon 1
    let ticket_resp = client
        .get(format!("{}/api/daemon/feed/ticket", d1.http_url()))
        .header("X-SBFB-Token", &d1.auth_token)
        .header("Host", format!("127.0.0.1:{}", d1.http_port))
        .send()
        .await
        .expect("feed ticket request");
    let ticket_body: serde_json::Value = ticket_resp.json().await.expect("ticket json");
    let ticket = ticket_body["ticket"]
        .as_str()
        .expect("ticket field missing");

    // Daemon 2 joins feed via ticket
    let join_resp = client
        .post(format!("{}/api/daemon/feed/join", d2.http_url()))
        .header("X-SBFB-Token", &d2.auth_token)
        .header("Host", format!("127.0.0.1:{}", d2.http_port))
        .json(&serde_json::json!({ "ticket": ticket }))
        .send()
        .await
        .expect("feed join request");
    assert!(
        join_resp.status().is_success(),
        "feed join returned {}",
        join_resp.status()
    );

    // Poll daemon 2 feed status until 3 entries synced (timeout 60s)
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut synced = false;
    while tokio::time::Instant::now() < deadline {
        let status_resp = client
            .get(format!("{}/api/daemon/feed/status", d2.http_url()))
            .header("X-SBFB-Token", &d2.auth_token)
            .header("Host", format!("127.0.0.1:{}", d2.http_port))
            .send()
            .await;
        if let Ok(resp) = status_resp {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                if let Some(count) = body["entry_count"].as_u64() {
                    if count >= 3 {
                        synced = true;
                        break;
                    }
                }
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(synced, "daemon 2 must sync 3 feed entries within 60s");

    // Verify cursor is coherent on daemon 2
    let cursor_resp = client
        .get(format!("{}/api/daemon/feed/cursor", d2.http_url()))
        .header("X-SBFB-Token", &d2.auth_token)
        .header("Host", format!("127.0.0.1:{}", d2.http_port))
        .send()
        .await
        .expect("cursor request");
    let cursor_body: serde_json::Value = cursor_resp.json().await.expect("cursor json");
    let cursor_seq = cursor_body["seq"].as_u64().unwrap_or(0);
    assert!(
        cursor_seq >= 3,
        "cursor seq must be >= 3 after sync, got {cursor_seq}"
    );

    cluster.shutdown().await.expect("cluster shutdown");
}
