// SPDX-License-Identifier: AGPL-3.0-or-later
//! E2E test: real daemon serves a real zip with COEP/COOP/CSP headers.
//! Closes P2-REVIEW-C-2 (3/3 MANDATORY).

use std::io::Write;

use anyhow::Result;
use nexus_test_harness::DaemonCluster;
use zip::write::SimpleFileOptions;

fn make_test_zip() -> Vec<u8> {
    let buf = std::io::Cursor::new(Vec::new());
    let mut writer = zip::ZipWriter::new(buf);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    writer
        .start_file("index.html", options)
        .expect("start_file");
    writer
        .write_all(b"<!DOCTYPE html><html><body>ok</body></html>")
        .expect("write");
    writer.finish().expect("finish").into_inner()
}

#[tokio::test]
async fn blob_serve_coep_headers_on_real_zip() -> Result<()> {
    let mut cluster = DaemonCluster::spawn(1).await?;
    let d = &cluster.nodes[0];
    let client = reqwest::Client::new();

    let zip_bytes = make_test_zip();
    let publish_resp = client
        .post(format!("{}/publish-blob", d.http_url()))
        .header("X-SBFB-Token", &d.auth_token)
        .header("Host", format!("127.0.0.1:{}", d.http_port))
        .body(zip_bytes)
        .send()
        .await?;
    assert_eq!(publish_resp.status(), 200, "publish-blob should succeed");

    let publish_body: serde_json::Value = publish_resp.json().await?;
    let hash = publish_body["hash"]
        .as_str()
        .expect("response should have hash field");

    let serve_resp = client
        .get(format!("{}/blob-serve/{}/index.html", d.http_url(), hash))
        .header("X-SBFB-Token", &d.auth_token)
        .header("Host", format!("127.0.0.1:{}", d.http_port))
        .send()
        .await?;
    assert_eq!(serve_resp.status(), 200, "blob-serve should return 200");

    let headers = serve_resp.headers();

    let coop = headers
        .get("cross-origin-opener-policy")
        .expect("COOP header missing")
        .to_str()?;
    assert_eq!(coop, "same-origin", "COOP must be same-origin");

    let coep = headers
        .get("cross-origin-embedder-policy")
        .expect("COEP header missing")
        .to_str()?;
    assert_eq!(coep, "require-corp", "COEP must be require-corp");

    let csp = headers
        .get("content-security-policy")
        .expect("CSP header missing")
        .to_str()?;
    assert!(
        csp.contains("connect-src 'none'"),
        "CSP must contain connect-src 'none', got: {csp}"
    );

    let body = serve_resp.text().await?;
    assert!(body.contains("ok"), "body should contain 'ok'");

    cluster.shutdown().await?;
    Ok(())
}
