// SPDX-License-Identifier: AGPL-3.0-or-later
//! Multi-daemon E2E integration tests (relay-gated class — self-skip at
//! runtime unless `SBFB_INTEGRATION=1`; exercised for real by
//! `.github/workflows/integration-nightly.yml` and the live T2 harness,
//! never counted as coverage on a default CI run).
//!
//! S82 Phase D: the feed insert below carries the `x-sbfb-feed-internal`
//! header required since S65 (ace05b0) — without it the endpoint returns
//! 403 and this test never reaches its sync-poll subject.

use nexus_test_harness::DaemonCluster;

fn integration_enabled() -> bool {
    // `== "1"`, mirror of the harness gate (S81 Phase K harmonisation):
    // `.is_ok()` treated `SBFB_INTEGRATION=0` as ENABLED.
    std::env::var("SBFB_INTEGRATION").unwrap_or_default() == "1"
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

    // Insert 3 feed operations on daemon 1 (all hex-valid fixtures)
    for i in 0u64..3 {
        let project_id = format!("{:0>64x}", i + 1);
        let commit_sha = format!("{:0>40x}", i + 100);
        let artifact_hash = format!("{:0>64x}", i + 200);
        let provenance_hash = format!("{:0>64x}", i + 300);
        let resp = client
            .post(format!("{}/api/daemon/feed/insert", d1.http_url()))
            .header("X-SBFB-Token", &d1.auth_token)
            .header("Host", format!("127.0.0.1:{}", d1.http_port))
            // Internal-only feed insert since S65 (ace05b0,
            // P2-FEED-INSERT-NO-AUTH-TIER): the loopback harness drives the
            // sanctioned internal path (403 without this header).
            .header("x-sbfb-feed-internal", "1")
            .json(&serde_json::json!({
                "op": {
                    "op_type": "ReleasePublished",
                    "project_id": project_id,
                    "repo_url": format!("https://github.com/org/app{i}"),
                    "commit_sha": commit_sha,
                    "artifact_hash": artifact_hash,
                    "provenance_hash": provenance_hash,
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

    // Poll daemon 2 feed status until 3 entries synced (timeout 60s).
    // Uses /api/daemon/feed/status which reads count_feed_entries()
    // and get_feed_last_seq() directly from the DB — does NOT depend
    // on the materializer's save_feed_cursor().
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut final_count = 0u64;
    let mut final_last_seq = 0u64;
    while tokio::time::Instant::now() < deadline {
        let status_resp = client
            .get(format!("{}/api/daemon/feed/status", d2.http_url()))
            .header("X-SBFB-Token", &d2.auth_token)
            .header("Host", format!("127.0.0.1:{}", d2.http_port))
            .send()
            .await;
        if let Ok(resp) = status_resp
            && let Ok(body) = resp.json::<serde_json::Value>().await
        {
            final_count = body["count"].as_u64().unwrap_or(0);
            final_last_seq = body["last_seq"].as_u64().unwrap_or(0);
            if final_count >= 3 {
                break;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    assert!(
        final_count >= 3,
        "daemon 2 must sync 3 feed entries within 60s, got {final_count}"
    );
    assert!(
        final_last_seq >= 3,
        "last_seq must be >= 3 after sync, got {final_last_seq}"
    );

    cluster.shutdown().await.expect("cluster shutdown");
}
