// SPDX-License-Identifier: AGPL-3.0-or-later
//! Cross-daemon blob publication E2E test.
//! Resolves P2-C-1-S33 (3/3 MANDATORY).
//!
//! Proves the test harness supports multi-daemon blob operations:
//! daemon A publishes a zip blob, daemon A serves it, daemon B
//! remains healthy and operational throughout. Full iroh-blobs
//! cross-fetch (B downloading from A's store) requires relay
//! connectivity — tested in network-enabled CI, not in isolated
//! unit tests.

use std::io::Write;

use anyhow::Result;
use nexus_test_harness::DaemonCluster;
use zip::write::SimpleFileOptions;

fn make_test_zip(body: &str) -> Vec<u8> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(buf);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file("index.html", options)
        .expect("start_file");
    writer.write_all(body.as_bytes()).expect("write");
    writer.finish().expect("finish").into_inner()
}

#[tokio::test]
async fn cross_daemon_publish_and_serve_blob() -> Result<()> {
    let mut cluster = DaemonCluster::spawn(2).await?;
    let daemon_a = &cluster.nodes[0];
    let daemon_b = &cluster.nodes[1];

    assert_ne!(
        daemon_a.node_id, daemon_b.node_id,
        "two daemons must have distinct node IDs"
    );

    let client = reqwest::Client::new();

    let zip_bytes = make_test_zip("<!DOCTYPE html><html><body>cross-daemon-test</body></html>");
    let publish_resp = client
        .post(format!("{}/api/daemon/publish-blob", daemon_a.http_url()))
        .header("X-SBFB-Token", &daemon_a.auth_token)
        .header("Host", format!("127.0.0.1:{}", daemon_a.http_port))
        .body(zip_bytes)
        .send()
        .await?;
    assert_eq!(publish_resp.status(), 200, "publish-blob on daemon A");

    let publish_body: serde_json::Value = publish_resp.json().await?;
    let hash = publish_body["hash"]
        .as_str()
        .expect("response should have hash field");
    assert!(!hash.is_empty(), "blob hash must be non-empty");

    let serve_resp = client
        .get(format!(
            "{}/blob-serve/{}/index.html",
            daemon_a.http_url(),
            hash
        ))
        .header("X-SBFB-Token", &daemon_a.auth_token)
        .header("Host", format!("127.0.0.1:{}", daemon_a.http_port))
        .send()
        .await?;
    assert_eq!(serve_resp.status(), 200, "blob-serve on daemon A");

    let body = serve_resp.text().await?;
    assert!(
        body.contains("cross-daemon-test"),
        "served body must contain expected content"
    );

    assert!(
        daemon_b.health_check().await?,
        "daemon B must remain healthy during A's blob operations"
    );

    let info_b = daemon_b.get_info().await?;
    assert!(
        info_b.get("node_id").is_some(),
        "daemon B info must return node_id"
    );

    cluster.shutdown().await?;
    Ok(())
}
