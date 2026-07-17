// SPDX-License-Identifier: AGPL-3.0-or-later
//! Blob-serve loopback HTTP domain — extracted verbatim from `http.rs`
//! (Sprint 82 Phase S4, PO-10 extended discipline: the domain's tests
//! co-migrated below via the shared `crate::test_support` harness).
//!
//! The render path of the platform: `GET /blob-serve/{hash}/{*path}`
//! serves files from cached zip archives into the sandboxed iframe, with
//! the CSP middleware injecting the security headers on ALL responses
//! (T37; single-source contract `nexus_core_rs::csp::BLOB_SERVE_CSP`,
//! re-exported through `nexus_shell_daemon_core::blob_serve`). Also hosts
//! the directory-only pull resolution cluster (Sprint 75 Phase D, shared
//! with `seed_api.rs`) and the `POST /panic/wipe` thin dispatch over
//! `crate::panic::PanicWipeService` (Sprint 20 Phase B). Routes stay
//! registered in `crate::http::build_router`: the blob-serve nest and its
//! CSP layer remain wired in `public_routes` WITHOUT bearer (public by
//! construction), `panic/wipe` stays in `authed_routes`. Route paths,
//! JSON shapes and status codes are unchanged.

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Json};
use nexus_core_rs::BlobsClient;
use nexus_shell_daemon_core::blob_serve;
use tracing::{debug, warn};

use crate::http::{DaemonHttpState, ErrorResponse};

/// Middleware that injects security headers on every blob-serve
/// response, including error responses.
pub(crate) async fn blob_serve_csp_middleware(request: Request, next: Next) -> impl IntoResponse {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(
        "content-security-policy",
        blob_serve::BLOB_SERVE_CSP.parse().unwrap(),
    );
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert(
        "cross-origin-opener-policy",
        blob_serve::BLOB_SERVE_COOP.parse().unwrap(),
    );
    headers.insert(
        "cross-origin-embedder-policy",
        blob_serve::BLOB_SERVE_COEP.parse().unwrap(),
    );
    // CORP: allow sub-resources (CSS, JS, images) to load even when
    // the document has an opaque origin (from CSP sandbox or iframe
    // sandbox attribute). Without this, COEP require-corp blocks
    // same-path resources that appear cross-origin to the opaque
    // origin.
    headers.insert(
        "cross-origin-resource-policy",
        "cross-origin".parse().unwrap(),
    );
    response
}

// =================================================================
// Directory-only pull resolution (Sprint 75 Phase D, carry PULL-2)
// =================================================================

/// Cap on the ordered provider vector handed to the multi-provider fetch.
/// The anchor plus at most `PULL_PROVIDER_CAP - 1` TTL-fresh seeders: a Sybil
/// swarm padding the SeedRegistry can never make the downloader attempt an
/// unbounded dial chain (THREAT_MODEL §15 row D; SEED-2 bounds the registry
/// itself, this bounds what one fetch will try).
const PULL_PROVIDER_CAP: usize = 8;

/// Wall-clock budget for one directory-only pull (the whole capped provider
/// chain, worst case every provider dead). The existing single-provider
/// ticket tier carries no explicit budget; the multi-provider chain gets one
/// so a fully-dead provider set fails the HTTP request instead of hanging it.
pub(crate) const DIRECTORY_PULL_TIMEOUT_SECS: u64 = 120;

/// Locate, across every SUBSCRIBED node directory, the catalog app whose
/// `archive_hash` equals `hash_hex`. Returns `(project_id, anchor_node_id_hex)`
/// of the first match (snapshot order is deterministic, sorted by node_id).
/// Empty archive hashes (placeholder rows) never match.
fn find_directory_app_by_hash(
    dirs: &[nexus_core_rs::NodeDirectoryEntry],
    hash_hex: &str,
) -> Option<(String, String)> {
    for dir in dirs {
        for app in &dir.directory.catalog {
            if !app.archive_hash.is_empty() && app.archive_hash == hash_hex {
                return Some((app.project_id.clone(), hex::encode(dir.directory.node_id)));
            }
        }
    }
    None
}

/// Locate, across every SUBSCRIBED node directory, the catalog app with
/// `project_id`. Returns `(archive_hash_hex, anchor_node_id_hex)`; rows
/// without an archive (empty hash) are skipped — there is nothing to pull.
///
/// Sprint 75 Phase F (review-D deferral): `want_hash` narrows the first-match
/// to the EXACT archive version the caller asked about — two subscribed
/// anchors listing the same `project_id` with different hashes (a fork, or an
/// older release) would otherwise resolve to whichever anchor sorts first,
/// and the caller would pin bytes it did not ask for. `None` keeps the
/// version-agnostic first-match (today's behaviour for callers that only know
/// the project id).
pub(crate) fn find_directory_app_by_project(
    dirs: &[nexus_core_rs::NodeDirectoryEntry],
    project_id: &str,
    want_hash: Option<&str>,
) -> Option<(String, String)> {
    for dir in dirs {
        for app in &dir.directory.catalog {
            if app.project_id == project_id
                && !app.archive_hash.is_empty()
                && want_hash.is_none_or(|w| w == app.archive_hash)
            {
                return Some((app.archive_hash.clone(), hex::encode(dir.directory.node_id)));
            }
        }
    }
    None
}

/// Build the ORDERED provider vector for a directory-only pull (Q5): the
/// anchor that published the directory first (it authored the listing and is
/// the most likely holder), then the TTL-fresh seeders of
/// `(project_id, archive_hash)` from the best-effort SeedRegistry. Deduped,
/// self excluded (we never dial ourselves), malformed ids skipped, capped at
/// [`PULL_PROVIDER_CAP`] (the loop stops pushing at the cap; the primitive
/// additionally enforces its own never-exceed ceiling). The iroh-blobs
/// `Downloader` consumes the vec in iteration order and retries the next
/// provider when one fails — so this ordering IS the fallback policy. A
/// lying seeder entry costs one failed dial, never integrity: the requested
/// object is the BLAKE3 hash itself.
///
/// Known availability residual (review Phase D): the seeder tail comes from
/// `seeders_recent`, which sorts lexicographically — a Sybil minting keys
/// with low hex prefixes can deterministically occupy the capped slots and
/// crowd an honest seeder out of the dial set (the anchor slot is never
/// crowdable). Integrity holds regardless (BLAKE3); random sampling of the
/// fresh-seeder set is the tracked mitigation, carried to the S76 audit.
pub(crate) fn directory_pull_providers(
    seed_registry: &crate::seed_registry::SeedRegistry,
    my_node_id: &str,
    anchor_hex: &str,
    project_id: &str,
    archive_hash_hex: &str,
    now: u64,
) -> Vec<iroh::EndpointId> {
    use std::str::FromStr as _;
    fn push_unique(providers: &mut Vec<iroh::EndpointId>, my_node_id: &str, hex_id: &str) {
        if hex_id == my_node_id {
            return;
        }
        if let Ok(id) = iroh::EndpointId::from_str(hex_id)
            && !providers.contains(&id)
        {
            providers.push(id);
        }
    }
    let mut providers: Vec<iroh::EndpointId> = Vec::new();
    push_unique(&mut providers, my_node_id, anchor_hex);
    for seeder in seed_registry.seeders_recent(project_id, archive_hash_hex, now) {
        if providers.len() >= PULL_PROVIDER_CAP {
            break;
        }
        push_unique(&mut providers, my_node_id, &seeder);
    }
    providers
}

/// `POST /panic/wipe` — Sprint 20 Phase B. Irreversibly destroy
/// the daemon's on-disk state (identity blobs + OS keyring
/// entries + subscriptions.json + blob cache) then exit the
/// process. Triggered by the shell's 5-tap `Ctrl+Shift+Alt+W`
/// gesture. The handler replies 200 BEFORE scheduling the exit
/// so the shell receives confirmation; the actual
/// `process::exit` runs from a spawned tokio task that sleeps
/// 100 ms to let axum flush the response.
pub(crate) async fn panic_wipe(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    warn!("POST /panic/wipe — executing irreversible wipe");
    let service = Arc::clone(&state.panic_wipe);
    match service.execute() {
        Ok(_) => {
            // Schedule the process exit on a background task so
            // the HTTP response can actually be written back.
            // `exit_only` skips re-running `execute` — the wipe
            // already happened synchronously above.
            tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                service.exit_only(0);
            });
            (StatusCode::OK, Json(serde_json::json!({ "wiped": true }))).into_response()
        }
        Err(e) => {
            warn!(error = %e, "panic wipe execute failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("panic wipe failed: {e}"),
                }),
            )
                .into_response()
        }
    }
}

/// `GET /blob-serve/{hash}/{*path}` — serve a file from a cached
/// zip archive with CSP headers. Sprint 12 Phase A.
///
/// If the archive is not in cache, attempts to load it from the
/// local blob store. If not in the local store either, returns 404.
pub(crate) async fn blob_serve(
    State(state): State<Arc<DaemonHttpState>>,
    Path((hash, path)): Path<(String, String)>,
) -> impl IntoResponse {
    // Strip leading slash from wildcard capture.
    let path = path.strip_prefix('/').unwrap_or(&path);
    // Default to index.html if path is empty.
    let path = if path.is_empty() { "index.html" } else { path };

    debug!(hash = %hash, path = %path, "GET /blob-serve");

    if !blob_serve::validate_zip_path(path) {
        return (StatusCode::BAD_REQUEST, "invalid path").into_response();
    }

    // Load into cache if not already present.
    if !state.blob_serve_cache.has(&hash) {
        let hash_bytes: [u8; 32] = match hex::decode(&hash).ok().and_then(|b| b.try_into().ok()) {
            Some(h) => h,
            None => return (StatusCode::BAD_REQUEST, "invalid hash hex").into_response(),
        };
        // Acquire the zip bytes from, in order: the ephemeral preview store
        // (Sprint 68), the local blob store, then — for an app DISCOVERED ON THE
        // NETWORK whose zip lives on the announcing node — a P2P download via the
        // archive ticket resolved from the browse aggregator, and finally — for a
        // DIRECTORY-ONLY app (Sprint 75 Phase D, closed GAP R5a) — a
        // multi-provider download by bare hash from the publishing anchor + the
        // best-effort seeders. Without those network tiers, any app the user did
        // not publish himself never renders (the whole point of "the network
        // distributes the app").
        let blobs = BlobsClient::new(state.node.blobs_store());
        let zip_bytes: Vec<u8> = if let Some(z) = state.preview_store.get(&hash) {
            z
        } else if let Ok(z) = blobs.get_bytes(hash_bytes).await {
            z
        } else if let Some(ticket) = state.browse_aggregator.find_archive_ticket_by_hash(&hash) {
            // The ticket carries the providing node's EndpointAddr; download the
            // blob into our local store, then read it back.
            if let Err(e) = blobs
                .fetch_ticket(state.node.endpoint(), state.node.memory_lookup(), &ticket)
                .await
            {
                warn!(error = %e, hash = %hash, "P2P archive fetch failed");
                return (
                    StatusCode::BAD_GATEWAY,
                    "failed to fetch app archive from network",
                )
                    .into_response();
            }
            match blobs.get_bytes(hash_bytes).await {
                Ok(z) => z,
                Err(_) => {
                    return (StatusCode::BAD_GATEWAY, "fetched archive unavailable")
                        .into_response();
                }
            }
        } else if let Some((project_id, anchor_hex)) =
            find_directory_app_by_hash(&state.curator_runtime.directory_snapshot(), &hash)
        {
            // Directory-only app: the listing advertises (anchor node_id,
            // archive_hash) and deliberately NO ticket (a stored ticket would
            // freeze a stale address — the Phase A bug). Fetch the bare hash
            // from the anchor first, then the TTL-fresh seeders (Q5 ordering);
            // pkarr resolves the bare EndpointIds. Content-addressing is the
            // integrity gate: whatever provider answers, the bytes ARE the
            // requested BLAKE3 or the download fails.
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let providers = directory_pull_providers(
                &state.seed_registry,
                &state.node_id,
                &anchor_hex,
                &project_id,
                &hash,
                now,
            );
            if providers.is_empty() {
                return (StatusCode::BAD_GATEWAY, "no dialable provider for this app")
                    .into_response();
            }
            match tokio::time::timeout(
                std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                blobs.fetch_hash_multi(state.node.endpoint(), hash_bytes, providers),
            )
            .await
            {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    warn!(error = %e, hash = %hash, "directory-only archive fetch failed");
                    return (
                        StatusCode::BAD_GATEWAY,
                        "failed to fetch app archive from network",
                    )
                        .into_response();
                }
                Err(_) => {
                    warn!(hash = %hash, "directory-only archive fetch timed out");
                    return (StatusCode::BAD_GATEWAY, "app archive fetch timed out")
                        .into_response();
                }
            }
            // Read back BY THE REQUESTED HASH — the same post-fetch integrity
            // re-check as the ticket tier (verrou 4: only the author's exact
            // bytes can land under this hash).
            match blobs.get_bytes(hash_bytes).await {
                Ok(z) => z,
                Err(_) => {
                    return (StatusCode::BAD_GATEWAY, "fetched archive unavailable")
                        .into_response();
                }
            }
        } else {
            return (StatusCode::NOT_FOUND, "blob not found").into_response();
        };
        if let Err(e) = state.blob_serve_cache.load(
            &hash,
            &zip_bytes,
            blob_serve::DEFAULT_MAX_DECOMPRESSED_BYTES,
        ) {
            warn!(error = %e, "failed to decompress zip");
            return (StatusCode::BAD_REQUEST, format!("invalid archive: {e}")).into_response();
        }
    }

    // Serve the file from cache.
    let file_bytes = match state.blob_serve_cache.get_file(&hash, path) {
        Some(b) => b,
        None => return (StatusCode::NOT_FOUND, "file not found in archive").into_response(),
    };

    let content_type = blob_serve::detect_content_type(path, &file_bytes);

    // CSP + X-Content-Type-Options are injected by
    // blob_serve_csp_middleware on ALL responses (T37).
    (
        StatusCode::OK,
        [
            ("Content-Type", content_type),
            ("Cache-Control", "public, max-age=3600, immutable"),
        ],
        file_bytes,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{KeyPair, create_node};
    use tower::ServiceExt;

    use crate::test_support::*;

    // ---- Sprint 75 Phase D: directory-only pull + node identity ----

    #[test]
    fn directory_resolvers_match_hash_and_project() {
        // The two R5 resolution helpers (review Phase D: previously untested
        // glue). by_hash: exact match wins, EMPTY archive hashes never match
        // (a placeholder row must not shadow a real one when the query is
        // empty/bogus), multi-directory scan, miss -> None. by_project:
        // archive-less rows are skipped (nothing to pull).
        let kp1 = KeyPair::generate();
        let kp2 = KeyPair::generate();
        let pid_a = "1".repeat(64); // placeholder row, NO archive
        let pid_b = "2".repeat(64);
        let pid_c = "3".repeat(64);
        let h1 = "a1".repeat(32);
        let h2 = "b2".repeat(32);

        let mut dir1 = nexus_core_rs::NodeDirectory::new(kp1.public_bytes(), 1);
        dir1.catalog = vec![
            nexus_core_rs::CatalogApp {
                project_id: pid_a.clone(),
                archive_hash: String::new(),
                project_name: "Placeholder".into(),
                category: "tools".into(),
                description: "no archive".into(),
            },
            catalog_app(&pid_b, &h1, "Babel"),
        ];
        let entry1 = nexus_core_rs::NodeDirectoryEntry::sign(dir1, &kp1).unwrap();
        let mut dir2 = nexus_core_rs::NodeDirectory::new(kp2.public_bytes(), 1);
        dir2.catalog = vec![catalog_app(&pid_c, &h2, "Atlas")];
        let entry2 = nexus_core_rs::NodeDirectoryEntry::sign(dir2, &kp2).unwrap();
        let dirs = vec![entry1, entry2];

        // by_hash: each hash resolves to ITS app + ITS anchor.
        assert_eq!(
            find_directory_app_by_hash(&dirs, &h1),
            Some((pid_b.clone(), hex::encode(kp1.public_bytes())))
        );
        assert_eq!(
            find_directory_app_by_hash(&dirs, &h2),
            Some((pid_c.clone(), hex::encode(kp2.public_bytes())))
        );
        // An empty query NEVER matches the placeholder's empty hash.
        assert_eq!(find_directory_app_by_hash(&dirs, ""), None);
        // Unknown hash -> None.
        assert_eq!(find_directory_app_by_hash(&dirs, &"ff".repeat(32)), None);

        // by_project: a real row resolves; an archive-less row is skipped.
        assert_eq!(
            find_directory_app_by_project(&dirs, &pid_b, None),
            Some((h1.clone(), hex::encode(kp1.public_bytes())))
        );
        assert_eq!(find_directory_app_by_project(&dirs, &pid_a, None), None);
        assert_eq!(
            find_directory_app_by_project(&dirs, &"9".repeat(64), None),
            None
        );

        // Sprint 75 Phase F (review-D deferral): `want_hash` discriminates
        // between two anchors listing the SAME project id with different
        // archive versions — the first-match must not pin bytes the caller
        // did not ask for.
        let kp3 = KeyPair::generate();
        let h3 = "d4".repeat(32);
        let mut dir3 = nexus_core_rs::NodeDirectory::new(kp3.public_bytes(), 1);
        dir3.catalog = vec![catalog_app(&pid_b, &h3, "Babel (derived)")];
        let entry3 = nexus_core_rs::NodeDirectoryEntry::sign(dir3, &kp3).unwrap();
        let mut dirs_collided = dirs.clone();
        dirs_collided.push(entry3);

        // Version-agnostic: still the first anchor's version (pre-F behaviour).
        assert_eq!(
            find_directory_app_by_project(&dirs_collided, &pid_b, None),
            Some((h1.clone(), hex::encode(kp1.public_bytes())))
        );
        // Discriminated: the requested version resolves to ITS anchor, even
        // when another anchor's listing of the same project sorts first.
        assert_eq!(
            find_directory_app_by_project(&dirs_collided, &pid_b, Some(&h3)),
            Some((h3.clone(), hex::encode(kp3.public_bytes())))
        );
        assert_eq!(
            find_directory_app_by_project(&dirs_collided, &pid_b, Some(&h1)),
            Some((h1.clone(), hex::encode(kp1.public_bytes())))
        );
        // A version nobody lists resolves to None (the handler 404s instead
        // of silently pinning a different version).
        assert_eq!(
            find_directory_app_by_project(&dirs_collided, &pid_b, Some(&"ee".repeat(32))),
            None
        );
    }

    #[test]
    fn fetch_provider_ordering() {
        // Q5 (plan D.3 #2): the provider vector is ORDERED — the publishing
        // anchor first, then the TTL-fresh seeders — deduped, self excluded,
        // capped. The iroh-blobs Downloader consumes it in iteration order,
        // so this vector IS the fallback policy.
        let reg = crate::seed_registry::SeedRegistry::new();
        let now = 1_700_000_000u64;
        let pid = "a".repeat(64);
        let hash = "cc".repeat(32);
        let me = hex::encode(KeyPair::generate().public_bytes());
        let anchor = hex::encode(KeyPair::generate().public_bytes());
        let s1 = hex::encode(KeyPair::generate().public_bytes());
        let s2 = hex::encode(KeyPair::generate().public_bytes());

        reg.record(&pid, &hash, &s1, now, now);
        reg.record(&pid, &hash, &s2, now, now);
        // The anchor also announced itself as a seeder → must dedup, not dial twice.
        reg.record(&pid, &hash, &anchor, now, now);
        // Our own node announced → must be excluded (we never dial ourselves).
        reg.record(&pid, &hash, &me, now, now);
        // A malformed id in the registry is skipped, never a panic.
        reg.record(&pid, &hash, "not-hex-at-all", now, now);

        let providers = directory_pull_providers(&reg, &me, &anchor, &pid, &hash, now);
        use std::str::FromStr as _;
        let anchor_id = iroh::EndpointId::from_str(&anchor).unwrap();
        assert_eq!(
            providers[0], anchor_id,
            "the anchor must be the FIRST provider (Q5 ordering)"
        );
        assert_eq!(
            providers.len(),
            3,
            "anchor + 2 seeders; anchor deduped, self + malformed excluded"
        );
        assert!(providers.contains(&iroh::EndpointId::from_str(&s1).unwrap()));
        assert!(providers.contains(&iroh::EndpointId::from_str(&s2).unwrap()));
        assert!(!providers.contains(&iroh::EndpointId::from_str(&me).unwrap()));

        // The cap bounds a Sybil-padded registry: many distinct fresh seeders
        // can never grow the dial chain past PULL_PROVIDER_CAP.
        for _ in 0..(PULL_PROVIDER_CAP + 5) {
            let sybil = hex::encode(KeyPair::generate().public_bytes());
            reg.record(&pid, &hash, &sybil, now, now);
        }
        let capped = directory_pull_providers(&reg, &me, &anchor, &pid, &hash, now);
        assert_eq!(capped.len(), PULL_PROVIDER_CAP, "provider vector is capped");
        assert_eq!(capped[0], anchor_id, "the anchor survives the cap in front");
    }

    // ---------------------------------------------------------
    // Sprint 12 Phase A: blob-serve handler (moved here from http.rs
    // with its domain in S82 Phase S4; the publish-blob tests live in
    // publish_api.rs since S82 Phase S)
    // ---------------------------------------------------------

    #[tokio::test]
    async fn blob_serve_returns_file_from_cached_zip() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // First, store a zip blob.
        let zip_bytes = make_zip(&[
            ("index.html", b"<h1>Hello SBFB</h1>"),
            ("assets/main.js", b"console.log('ok')"),
        ]);
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash = blobs.add_bytes(zip_bytes).await.unwrap();
        let hash_hex = hex::encode(hash);

        // GET /blob-serve/{hash}/index.html
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/index.html"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Check CSP header.
        let csp = resp
            .headers()
            .get("Content-Security-Policy")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(csp.contains("connect-src 'none'"));

        // Check X-Content-Type-Options.
        assert_eq!(
            resp.headers()
                .get("X-Content-Type-Options")
                .unwrap()
                .to_str()
                .unwrap(),
            "nosniff"
        );

        // Check COOP/COEP isolation headers.
        assert_eq!(
            resp.headers()
                .get("Cross-Origin-Opener-Policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "same-origin"
        );
        assert_eq!(
            resp.headers()
                .get("Cross-Origin-Embedder-Policy")
                .unwrap()
                .to_str()
                .unwrap(),
            "require-corp"
        );

        // Check content.
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert_eq!(&body[..], b"<h1>Hello SBFB</h1>");

        // GET sub-resource.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/assets/main.js"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("Content-Type")
            .unwrap()
            .to_str()
            .unwrap();
        assert!(ct.contains("javascript"));
    }

    /// THE product test (real cross-node boundary, no mock): an app whose zip
    /// lives on ANOTHER node must render. Node A hosts the zip; node B knows it
    /// only through a browse entry carrying the archive ticket; GET /blob-serve
    /// on B P2P-downloads the zip from A and serves it. Before the fix, blob-serve
    /// read only B's local store and returned 404 -> any app not self-published
    /// never loaded.
    #[tokio::test]
    async fn remote_app_renders_via_p2p_fetch() {
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};

        // Node A hosts the app zip.
        let node_a = create_node().await.expect("node A");
        let blobs_a = BlobsClient::new(node_a.blobs_store());
        let zip = make_zip(&[("index.html", b"<html><body>remote</body></html>")]);
        let hash = blobs_a.add_bytes(zip).await.unwrap();
        let hash_hex = hex::encode(hash);
        let addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .expect("node A address");
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        // Node B (the visitor) only knows the app via a browse entry + ticket.
        let state = mk_state().await; // state.node is node B
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: "remote-app".into(),
            node_id: None,
            project_name: "Remote App".into(),
            category: "tools".into(),
            description: "lives on node A".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: Some(ticket),
            archive_hash: Some(hash_hex.clone()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/index.html"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "remote app must render via P2P fetch from node A"
        );
        let body = to_bytes(resp.into_body(), 65536).await.unwrap();
        assert_eq!(&body[..], b"<html><body>remote</body></html>");

        node_a.shutdown().await.ok();
    }

    #[tokio::test]
    async fn blob_serve_returns_404_for_unknown_hash() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{}/index.html", "ab".repeat(32)))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn blob_serve_rejects_path_traversal() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/blob-serve/{}/..%2F..%2Fetc%2Fpasswd",
                        "ab".repeat(32)
                    ))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        // Either 400 (path validation) or 404 (hash not found first).
        assert!(
            resp.status() == StatusCode::BAD_REQUEST || resp.status() == StatusCode::NOT_FOUND,
            "expected 400 or 404, got {}",
            resp.status()
        );
    }

    /// Sprint 13 Phase A (T37): error responses from blob-serve
    /// must also carry CSP + X-Content-Type-Options headers, not
    /// just the 200 success path.
    #[tokio::test]
    async fn blob_serve_error_responses_have_csp() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{}/index.html", "ab".repeat(32)))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        // T37: CSP must be present on error responses too.
        assert!(
            resp.headers().get("content-security-policy").is_some(),
            "CSP header missing on 404 blob-serve response",
        );
        assert!(
            resp.headers().get("x-content-type-options").is_some(),
            "X-Content-Type-Options header missing on 404 blob-serve response",
        );
        assert!(
            resp.headers().get("cross-origin-opener-policy").is_some(),
            "COOP header missing on 404 blob-serve response",
        );
        assert!(
            resp.headers().get("cross-origin-embedder-policy").is_some(),
            "COEP header missing on 404 blob-serve response",
        );
    }

    /// Sprint 79 Phase H: the CSP header SERVED by the daemon must be
    /// byte-for-byte equal to the single-source contract
    /// `nexus_core_rs::csp::BLOB_SERVE_CSP` — on BOTH the 200 success path
    /// and the 404 error path. The pre-existing assertions only check a
    /// substring (`contains("connect-src 'none'")`, success path above) or
    /// mere presence (`.is_some()`, T37 — the 404 test above); neither catches
    /// a drift in any OTHER directive of the served string. This is the
    /// runtime backing of the T2 acceptance field `blob_serve_csp_equals_contract`:
    /// it proves the Phase E gate protects the CSP that is ACTUALLY served, not
    /// a fictional one. The production middleware injects `blob_serve::BLOB_SERVE_CSP`
    /// (re-exported from this same const), so equality here witnesses the whole
    /// served path, not just the const definition.
    #[tokio::test]
    async fn blob_serve_csp_header_byte_exact_matches_contract() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // 200 path: store a zip and GET its index.html.
        let zip_bytes = make_zip(&[("index.html", b"<h1>Hello SBFB</h1>")]);
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash = blobs.add_bytes(zip_bytes).await.unwrap();
        let hash_hex = hex::encode(hash);

        let resp_200 = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{hash_hex}/index.html"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_200.status(), StatusCode::OK);
        assert_eq!(
            resp_200
                .headers()
                .get("content-security-policy")
                .expect("CSP header on 200 blob-serve response")
                .to_str()
                .unwrap(),
            nexus_core_rs::csp::BLOB_SERVE_CSP,
            "served CSP on 200 drifted from the single-source BLOB_SERVE_CSP contract",
        );

        // 404 path: GET a hash that does not exist (middleware posts CSP on errors too).
        let resp_404 = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/blob-serve/{}/index.html", "ab".repeat(32)))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp_404.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            resp_404
                .headers()
                .get("content-security-policy")
                .expect("CSP header on 404 blob-serve response")
                .to_str()
                .unwrap(),
            nexus_core_rs::csp::BLOB_SERVE_CSP,
            "served CSP on 404 drifted from the single-source BLOB_SERVE_CSP contract",
        );
    }
}
