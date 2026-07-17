// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared `#[cfg(test)]` HTTP test harness + golden characterization net —
//! gathered verbatim from `http.rs::tests` (Sprint 82 Phase N2, PO-10).
//!
//! Hosts the in-process router/state builders (`build_test_router_ext` and
//! its posture wrappers, `mk_state*`) consumed by `http.rs::tests` and the
//! per-domain `*_api.rs` test modules, plus the cross-domain
//! `golden_http_*` family (Sprint 82 Phase M) — the behavior-preserving
//! safety net of the http.rs domain splits. The golden family lives here
//! as ONE atomic block: it must never be fragmented per domain, and a
//! safety net must not live inside the file it protects.

use std::path::Path as FsPath;
use std::sync::Arc;
use std::time::SystemTime;

use axum::Router;
use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use axum::middleware;
use nexus_core_rs::{KeyPair, Node, PowSolveCache, create_node};
use nexus_shell_daemon_core::auth::AuthState;
use nexus_shell_daemon_core::blob_serve::BlobServeCache;
use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseEntry};
use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
use tokio::sync::RwLock;
use tower::ServiceExt;

use crate::http::{DaemonHttpState, build_router};

/// Sprint 16 Phase A: known-valid bearer token used by every
/// test via [`build_test_router`]. 64-char lowercase hex,
/// the shape
/// [`nexus_shell_daemon_core::auth::load_or_generate_token`]
/// would produce but fixed so assertions stay deterministic.
pub(crate) const TEST_TOKEN: &str =
    "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

/// Canonicalized `X-SBFB-Token` header name for the test-only
/// layer below. Kept as a separate const because `HeaderMap`
/// wants a `HeaderName`, not a `&str`.
const AUTH_HEADER_NAME: axum::http::HeaderName =
    axum::http::HeaderName::from_static("x-sbfb-token");

/// Header posture of a test router: [`TestHeaders::Inject`]
/// adds the outer test-only layer built by
/// [`build_test_router_ext`]; [`TestHeaders::Raw`] hands back
/// the bare production router so a test drives `Origin`
/// itself (CORS assertions).
pub(crate) enum TestHeaders {
    Inject,
    Raw,
}

/// Single parameterized constructor behind every test-router
/// variant (Sprint 82 Phase M dedup): cors origins, optional
/// SPA `web_root`, header posture. Only the outermost injected
/// layer is synthetic — every route still runs the real
/// [`auth_required`] middleware, and the 401 / 403 paths are
/// covered by `auth::tests` in the core crate. The injected
/// layer does exactly two things, together: it adds
/// `X-SBFB-Token` and a loopback `Host` on every inbound
/// request, and it strips `Origin` (so the CORS gate sees the
/// no-Origin shape non-CORS tests expect). Nothing else may be
/// added here — e.g. the feed-insert internal-header tests
/// assert the harness does NOT set `x-sbfb-feed-internal`.
pub(crate) fn build_test_router_ext(
    state: Arc<DaemonHttpState>,
    cors: &[String],
    web_root: Option<&FsPath>,
    headers: TestHeaders,
) -> Router {
    use axum::http::HeaderValue;
    use axum::http::header::{HOST, ORIGIN};
    let router = build_router(
        state,
        AuthState::new(TEST_TOKEN.to_string()),
        cors,
        web_root,
    );
    match headers {
        TestHeaders::Raw => router,
        TestHeaders::Inject => router.layer(middleware::from_fn(
            |mut req: axum::extract::Request, next: middleware::Next| async move {
                let h = req.headers_mut();
                if !h.contains_key(AUTH_HEADER_NAME) {
                    h.insert(AUTH_HEADER_NAME, HeaderValue::from_static(TEST_TOKEN));
                }
                if !h.contains_key(HOST) {
                    h.insert(HOST, HeaderValue::from_static("127.0.0.1:0"));
                }
                h.remove(ORIGIN);
                next.run(req).await
            },
        )),
    }
}

/// Build the production router plus the injected test-only
/// header layer so the tests below can keep their one-liner
/// `Request::builder().uri(..)` shape without re-attaching
/// headers by hand 40+ times.
pub(crate) fn build_test_router(state: Arc<DaemonHttpState>) -> Router {
    build_test_router_ext(state, &[], None, TestHeaders::Inject)
}

/// Build a [`DaemonHttpState`] backed by a live iroh node.
/// Every HTTP test spins up a fresh node because the
/// browse route reaches through the Arc<Node> to probe
/// endpoints. The `_node_guard` return keeps the node
/// alive for the scope of the test; letting it drop
/// calls the synchronous Drop path which is fine for
/// unit tests.
pub(crate) async fn mk_state() -> Arc<DaemonHttpState> {
    mk_state_with_mode(nexus_core_rs::IdentityMode::Normal).await
}

pub(crate) async fn mk_state_with_sbfb_home(home: std::path::PathBuf) -> Arc<DaemonHttpState> {
    let mut state = (*mk_state().await).clone();
    state.sbfb_home = Some(home);
    Arc::new(state)
}

pub(crate) async fn mk_state_with_mode(mode: nexus_core_rs::IdentityMode) -> Arc<DaemonHttpState> {
    mk_state_with_mode_tx(mode, tokio::sync::mpsc::channel(8).0).await
}

// Variant that injects a caller-supplied gossip_cmd_tx so a test can hold
// the receiver and assert what the announce path pushed to the outbox
// (remediation #8). The default mk_state drops the rx, which closes the
// channel — fine for tests that don't assert on it.
pub(crate) async fn mk_state_with_mode_tx(
    mode: nexus_core_rs::IdentityMode,
    gossip_cmd_tx: crate::runtime::GossipCmdTx,
) -> Arc<DaemonHttpState> {
    let node = create_node().await.expect("boot test node");
    let tmp = tempfile::tempdir().expect("tempdir");
    let keystore = Arc::new(nexus_core_rs::LocalFileKeyStore::new(tmp.path()));
    let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
        keystore,
        tmp.path().join("state.sqlite"),
        tmp.path().join("blob-cache"),
        Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
    ));
    // The tempdir is intentionally leaked so the panic service's
    // keystore can still reference its directory across the
    // state's lifetime. Unit tests never actually invoke the
    // panic service, so the directory stays untouched.
    std::mem::forget(tmp);
    Arc::new(DaemonHttpState {
        node_id: node.node_id(),
        daemon_version: "0.1.0-test".to_string(),
        boot_time: SystemTime::now(),
        api_host: "127.0.0.1".to_string(),
        api_port: 12345,
        curator_runtime: Arc::new(CuratorRuntime::new(None)),
        browse_aggregator: Arc::new(BrowseAggregator::new()),
        node: Arc::new(node),
        gossip_sender: Arc::new(RwLock::new(None)),
        gossip_cmd_tx,
        default_curators: vec![],
        blob_serve_cache: Arc::new(BlobServeCache::new(8)),
        identity_mode: mode,
        panic_wipe,
        pow_solve_cache: Arc::new(PowSolveCache::new()),
        pow_policy: nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
        pow_keypair: Arc::new(KeyPair::generate()),
        curator_gossip_topic: nexus_shell_daemon_core::iroh_runtime::curator_topic_id(),
        coordinator_db: std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("test coordinator DB"),
        )),
        result_event_tx: tokio::sync::broadcast::channel(8).0,
        canary_registry: {
            let tmp = tempfile::tempdir().expect("canary tmp");
            std::sync::Arc::new(std::sync::Mutex::new(
                nexus_coordinator_rs::canary_registry::CanaryRegistry::new(
                    tmp.keep().join("canary_registry.json"),
                ),
            ))
        },
        canary_input: Some(std::sync::Arc::new(
            nexus_coordinator_rs::canary_input::CanaryInputManager::new(None, None, None),
        )),
        sbfb_home: None,
        project_doc: None,
        task_dispatch_tx: None,
        local_worker: std::sync::Arc::new(crate::local_worker::LocalWorkerSupervisor::new()),
        app_storage: crate::storage_api::new_app_storage(),
        storage_namespaces: crate::storage_api::new_storage_namespaces(),
        storage_write_limiter: Arc::new(
            nexus_shell_daemon_core::storage_limiter::StorageWriteLimiter::new(),
        ),
        feed_sync_state: None,
        feed_rate_limiter: Arc::new(nexus_shell_daemon_core::feed_limiter::FeedRateLimiter::new()),
        feed_join_handles: Arc::new(std::sync::Mutex::new(Vec::new())),
        feed_join_shutdown: Arc::new(tokio::sync::watch::channel(false).0),
        preview_store: nexus_shell_daemon_core::preview::PreviewStore::new(
            nexus_shell_daemon_core::preview::DEFAULT_TTL,
        ),
        seed_registry: Arc::new(crate::seed_registry::SeedRegistry::new()),
        shard_sessions: Arc::new(crate::shard_session::ShardSessionRegistry::default()),
    })
}

/// RAW-posture router for the CORS tests below: no injected
/// layer, so the test-supplied `Origin` reaches the CORS gate.
pub(crate) fn build_cors_test_router(state: Arc<DaemonHttpState>, cors: &[String]) -> Router {
    build_test_router_ext(state, cors, None, TestHeaders::Raw)
}

/// Injected-posture router with an SPA `web_root`. Returns the
/// `TempDir` alongside the router: `ServeDir` reads the
/// directory at request time, so the caller must keep it alive
/// (folding it into a bare `Router` would drop the directory
/// and break the SPA fallback).
pub(crate) fn build_test_router_with_web_root(
    state: Arc<DaemonHttpState>,
) -> (Router, tempfile::TempDir) {
    use std::fs;
    let tmp = tempfile::tempdir().expect("tempdir for web_root");
    fs::write(
        tmp.path().join("index.html"),
        "<!doctype html><div id=root></div>",
    )
    .unwrap();
    let router = build_test_router_ext(state, &[], Some(tmp.path()), TestHeaders::Inject);
    (router, tmp)
}

// =============================================================
// Sprint 82 Phase M — HTTP golden characterization (D3).
// Safety net for splitting `http.rs` into per-domain modules:
// each case pins the exact (status, significant headers, body)
// of one surface on the current tree, so a behavior-preserving
// refactor can prove the moved handlers answer identically by
// re-running these tests. Fixtures were captured by executing
// the surfaces, never hand-derived from handler code.
// =============================================================

/// Response fields whose values depend on the fresh per-test
/// node (identity, feed revision, content-addressed archive of
/// a node-specific catalog). Strictly the OBSERVED volatile
/// set — keeping the allowlist minimal means a real drift on
/// any other field still fails the golden.
const GOLDEN_VOLATILE_FIELDS: &[&str] = &["node_id", "revision", "archive_hash"];

const GOLDEN_REDACTED: &str = "<VOLATILE>";

/// Replace volatile fields with a fixed placeholder anywhere in
/// the JSON tree. Key-order handling is not needed:
/// `serde_json::Map` equality is key-order-insensitive.
fn golden_redact(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map.iter_mut() {
                if GOLDEN_VOLATILE_FIELDS.contains(&key.as_str()) {
                    *val = serde_json::Value::String(GOLDEN_REDACTED.to_string());
                } else {
                    golden_redact(val);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                golden_redact(item);
            }
        }
        _ => {}
    }
}

enum GoldenBody {
    /// Exact JSON body after volatile-field redaction.
    Json(serde_json::Value),
    /// Exact plain-text body (axum rejections, error strings).
    Text(&'static str),
}

/// One pinned surface: request shape + the exact response
/// contract a behavior-preserving refactor must not change.
struct GoldenCase {
    name: &'static str,
    method: Method,
    uri: &'static str,
    json_body: Option<serde_json::Value>,
    want_status: u16,
    /// Headers asserted literally as (name, value). Volatile
    /// headers (date, last-modified, content-length) are never
    /// listed here.
    want_headers: &'static [(&'static str, &'static str)],
    want_body: GoldenBody,
}

async fn golden_check(app: Router, case: &GoldenCase) {
    let mut builder = Request::builder().method(case.method.clone()).uri(case.uri);
    let body = match &case.json_body {
        Some(json) => {
            builder = builder.header("content-type", "application/json");
            axum::body::Body::from(serde_json::to_vec(json).unwrap())
        }
        None => axum::body::Body::empty(),
    };
    let resp = app.oneshot(builder.body(body).unwrap()).await.unwrap();
    let status = resp.status().as_u16();
    let headers = resp.headers().clone();
    let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    let text = String::from_utf8_lossy(&bytes).into_owned();
    assert_eq!(
        status, case.want_status,
        "[golden:{}] status drifted (body: {text})",
        case.name
    );
    for (header_name, want) in case.want_headers {
        let got = headers.get(*header_name).and_then(|v| v.to_str().ok());
        assert_eq!(
            got,
            Some(*want),
            "[golden:{}] header `{header_name}` drifted",
            case.name
        );
    }
    match &case.want_body {
        GoldenBody::Json(want) => {
            let mut got: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(|e| {
                panic!("[golden:{}] body is not JSON ({e}): {text}", case.name)
            });
            golden_redact(&mut got);
            let mut want = want.clone();
            golden_redact(&mut want);
            assert_eq!(got, want, "[golden:{}] body drifted", case.name);
        }
        GoldenBody::Text(want) => {
            assert_eq!(text, *want, "[golden:{}] body drifted", case.name);
        }
    }
}

/// Run a batch of golden cases against one fresh state through
/// the standard injected-headers test router.
async fn golden_run(cases: &[GoldenCase]) {
    let app = build_test_router(mk_state().await);
    for case in cases {
        golden_check(app.clone(), case).await;
    }
}

/// Public tier: `/health` (no auth) + the blob-serve CSP
/// middleware headers on the error path (T37 pins headers on
/// ALL responses, not just the success path).
#[tokio::test]
async fn golden_http_public_tier() {
    golden_run(&[
        GoldenCase {
            name: "health",
            method: Method::GET,
            uri: "/health",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "status": "ok",
                "schema_version": 1,
                "daemon_version": "0.1.0-test"
            })),
        },
        GoldenCase {
            name: "blob_serve_bad_hash",
            method: Method::GET,
            uri: "/blob-serve/not-a-hash/index.html",
            json_body: None,
            want_status: 400,
            want_headers: &[
                ("content-type", "text/plain; charset=utf-8"),
                (
                    "content-security-policy",
                    "default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:; \
                     connect-src 'none'; worker-src 'none'; frame-src 'none'; \
                     object-src 'none'; base-uri 'none'; form-action 'none'; \
                     frame-ancestors *; sandbox allow-scripts",
                ),
                ("x-content-type-options", "nosniff"),
                ("cross-origin-opener-policy", "same-origin"),
                ("cross-origin-embedder-policy", "require-corp"),
                ("cross-origin-resource-policy", "cross-origin"),
            ],
            want_body: GoldenBody::Text("invalid hash hex"),
        },
    ])
    .await;
}

/// Shard-session domain (split target: 6 loopback handlers).
#[tokio::test]
async fn golden_http_shard_session_domain() {
    golden_run(&[
        GoldenCase {
            name: "shard_session_get_missing",
            method: Method::GET,
            uri: "/api/daemon/shard-session/nonexistent-session",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "found": false,
                "session": null
            })),
        },
        GoldenCase {
            name: "shard_session_group_empty",
            method: Method::POST,
            uri: "/api/daemon/shard-session/group",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `group_id` at line 1 column 2",
            ),
        },
    ])
    .await;
}

/// Seed domain (invites ledger, availability count, voluntary
/// seed validation).
#[tokio::test]
async fn golden_http_seed_domain() {
    golden_run(&[
        GoldenCase {
            name: "seed_invites_list",
            method: Method::GET,
            uri: "/api/daemon/seed/invites/proj-golden",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({ "invites": [] })),
        },
        GoldenCase {
            name: "seed_count",
            method: Method::GET,
            uri: "/api/daemon/seed-count/proj-golden",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "peer_count": 0,
                "self_seeding": false,
                "self_pin_enabled": null
            })),
        },
        GoldenCase {
            name: "seed_voluntary_empty",
            method: Method::POST,
            uri: "/api/daemon/seed",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `project_id` at line 1 column 2",
            ),
        },
    ])
    .await;
}

/// FROST domain (4 ceremony admin handlers).
#[tokio::test]
async fn golden_http_frost_domain() {
    golden_run(&[
        GoldenCase {
            name: "frost_round1_empty",
            method: Method::POST,
            uri: "/api/canary/frost/round1",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `participant` at line 1 column 2",
            ),
        },
        GoldenCase {
            name: "frost_trusted_dealer_empty",
            method: Method::POST,
            uri: "/api/canary/frost/trusted-dealer",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `k` at line 1 column 2",
            ),
        },
    ])
    .await;
}

/// Coordinator domain (task submit validation, kudos read,
/// kudos chain verify).
#[tokio::test]
async fn golden_http_coordinator_domain() {
    golden_run(&[
        GoldenCase {
            name: "kudos_get",
            method: Method::GET,
            uri: "/api/v1/kudos/proj-golden",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "project_id": "proj-golden",
                "total": 0,
                "contributors": []
            })),
        },
        GoldenCase {
            name: "kudos_verify_chain",
            method: Method::GET,
            uri: "/api/v1/kudos/proj-golden/verify",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({ "valid": true })),
        },
        GoldenCase {
            name: "tasks_submit_empty",
            method: Method::POST,
            uri: "/api/v1/tasks/submit",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `project_id` at line 1 column 2",
            ),
        },
    ])
    .await;
}

/// Curators domain (list + pubkey validation on unsubscribe).
#[tokio::test]
async fn golden_http_curators_domain() {
    golden_run(&[
        GoldenCase {
            name: "curators_list",
            method: Method::GET,
            uri: "/api/daemon/curators",
            json_body: None,
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "entries": [],
                "subscribed_curators": []
            })),
        },
        GoldenCase {
            name: "curators_unsubscribe_unknown",
            method: Method::DELETE,
            uri: "/api/daemon/curators/deadbeef",
            json_body: None,
            want_status: 400,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "error": "invalid curator pubkey hex (expected 64 lowercase chars): deadbeef"
            })),
        },
    ])
    .await;
}

/// Publish domain — the three entry points (`publish_project`,
/// `publish_blob`, `publish_directory`) are co-located in
/// `publish_api.rs` (S82 Phase S; they were scattered across
/// `http.rs` when this net was laid), and the net still covers
/// all three. The `publish-blob` hash is the BLAKE3 of the
/// fixed two-byte body `{}` — content-addressed, deterministic.
/// `directory/publish` echoes node identity + a node-specific
/// catalog archive: those fields are redacted as volatile.
#[tokio::test]
async fn golden_http_publish_domain() {
    golden_run(&[
        GoldenCase {
            name: "publish_empty",
            method: Method::POST,
            uri: "/api/daemon/publish",
            json_body: Some(serde_json::json!({})),
            want_status: 422,
            want_headers: &[("content-type", "text/plain; charset=utf-8")],
            want_body: GoldenBody::Text(
                "Failed to deserialize the JSON body into the target type: \
                 missing field `project_name` at line 1 column 2",
            ),
        },
        GoldenCase {
            name: "publish_blob_empty",
            method: Method::POST,
            uri: "/api/daemon/publish-blob",
            json_body: Some(serde_json::json!({})),
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "hash": "6e46dd10defc9b56c29a6ec56b508c21f54c08192194e4df25bf36f0c9c3c279"
            })),
        },
        GoldenCase {
            name: "directory_publish_empty",
            method: Method::POST,
            uri: "/api/daemon/directory/publish",
            json_body: Some(serde_json::json!({})),
            want_status: 200,
            want_headers: &[("content-type", "application/json")],
            want_body: GoldenBody::Json(serde_json::json!({
                "node_id": GOLDEN_REDACTED,
                "revision": GOLDEN_REDACTED,
                "catalog_len": 0,
                "archive_hash": GOLDEN_REDACTED
            })),
        },
    ])
    .await;
}

/// CORS preflight through the RAW router (the only construction
/// where a test-supplied `Origin` reaches the CORS layer).
#[tokio::test]
async fn golden_http_cors_preflight_raw() {
    let app = build_cors_test_router(mk_state().await, &[]);
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/health")
                .header("origin", "http://localhost:3000")
                .header("access-control-request-method", "GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "[golden:cors_preflight]");
    let headers = resp.headers().clone();
    for (header_name, want) in [
        ("access-control-allow-origin", "http://localhost:3000"),
        ("allow", "GET,HEAD"),
        (
            "vary",
            "origin, access-control-request-method, access-control-request-headers",
        ),
    ] {
        let got = headers.get(header_name).and_then(|v| v.to_str().ok());
        assert_eq!(
            got,
            Some(want),
            "[golden:cors_preflight] header `{header_name}` drifted"
        );
    }
    let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    assert!(
        bytes.is_empty(),
        "[golden:cors_preflight] preflight body must be empty"
    );
}

/// SPA fallback with `web_root` Some: `/` serves the exact
/// index.html document written by the harness.
#[tokio::test]
async fn golden_http_spa_fallback() {
    let (app, _tmp) = build_test_router_with_web_root(mk_state().await);
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK, "[golden:spa_root]");
    let headers = resp.headers().clone();
    for (header_name, want) in [("content-type", "text/html"), ("accept-ranges", "bytes")] {
        let got = headers.get(header_name).and_then(|v| v.to_str().ok());
        assert_eq!(
            got,
            Some(want),
            "[golden:spa_root] header `{header_name}` drifted"
        );
    }
    let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
    assert_eq!(
        String::from_utf8_lossy(&bytes),
        "<!doctype html><div id=root></div>",
        "[golden:spa_root] body drifted"
    );
}

// ---------------------------------------------------------------
// Shared cross-domain test fixtures (promoted in Sprint 82 Phase O:
// consumed by the migrated seed_api/publish_api/browse_api tests and
// by the staying http.rs fork/pull-resolution tests).
// ---------------------------------------------------------------

pub(crate) fn own_browse_entry(project_id: &str, name: &str, owner: Option<String>) -> BrowseEntry {
    BrowseEntry {
        project_id: project_id.into(),
        node_id: owner,
        project_name: name.into(),
        category: "tools".into(),
        description: "fixture".into(),
        curator_pubkey: String::new(),
        curator_name: "Self-published".into(),
        source: nexus_shell_daemon_core::browse::BrowseSource::Direct,
        status: nexus_shell_daemon_core::browse::BrowseStatus::Reachable,
        last_probed_at: None,
        archive_ticket: None,
        archive_hash: Some("ab".repeat(32)),
        repo_url: None,
        provenance_hash: None,
        is_open_source: false,
    }
}

pub(crate) fn catalog_app(
    project_id: &str,
    archive_hash: &str,
    name: &str,
) -> nexus_core_rs::CatalogApp {
    nexus_core_rs::CatalogApp {
        project_id: project_id.into(),
        archive_hash: archive_hash.into(),
        project_name: name.into(),
        category: "tools".into(),
        description: "fixture".into(),
    }
}

/// Helper: create a minimal zip archive in memory.
pub(crate) fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
    use std::io::Write;
    let buf = Vec::new();
    let cursor = std::io::Cursor::new(buf);
    let mut writer = zip::ZipWriter::new(cursor);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, data) in files {
        writer.start_file(*name, options).unwrap();
        writer.write_all(data).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

/// POST a forked workspace's zip to `/api/v1/deploy-workspace` and return the
/// HTTP status. Mirrors `publish_app` but for the local-redeploy path.
pub(crate) async fn deploy_workspace_app(
    state: &Arc<DaemonHttpState>,
    name: &str,
    zip: Vec<u8>,
) -> StatusCode {
    let uri = format!(
        "/api/v1/deploy-workspace?project_name={}&category=tools&description=forked",
        name.replace(' ', "%20")
    );
    build_test_router(Arc::clone(state))
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(uri)
                .body(axum::body::Body::from(zip))
                .unwrap(),
        )
        .await
        .unwrap()
        .status()
}

/// Sign a directory under `kp` (the anchor identity, possibly never
/// dialable), host its blob on `host`, and ingest it into `state`'s
/// curator runtime through the REAL subscription-gated path (subscribe +
/// announcement + blob fetch + signature/revision verify).
pub(crate) async fn ingest_remote_directory(
    state: &Arc<DaemonHttpState>,
    host: &Node,
    kp: &KeyPair,
    catalog: Vec<nexus_core_rs::CatalogApp>,
    revision: u64,
) -> nexus_core_rs::NodeDirectoryEntry {
    let mut dir = nexus_core_rs::NodeDirectory::new(kp.public_bytes(), revision);
    dir.catalog = catalog;
    let entry = nexus_core_rs::NodeDirectoryEntry::sign(dir, kp).expect("sign directory");
    let body = serde_json::to_vec(&entry).unwrap();
    let blobs_host = nexus_core_rs::BlobsClient::new(host.blobs_store());
    let blob_hash = blobs_host.add_bytes(&body).await.unwrap();
    let host_addr = nexus_core_rs::DiscoveryClient::new(host.endpoint())
        .my_endpoint_addr()
        .await
        .expect("host must expose an address");
    let ticket = iroh_blobs::ticket::BlobTicket::new(
        host_addr,
        iroh_blobs::Hash::from_bytes(blob_hash),
        iroh_blobs::BlobFormat::Raw,
    )
    .to_string();
    let ann = nexus_shell_daemon_core::iroh_runtime::NodeDirectoryAnnouncement::new(
        kp.public_bytes(),
        ticket,
    );
    state
        .curator_runtime
        .subscribe(&hex::encode(kp.public_bytes()))
        .expect("subscribe to the anchor");
    state
        .curator_runtime
        .process_directory_announcement_bytes(&ann.to_bytes().unwrap(), &state.node)
        .await
        .expect("directory must ingest through the real gate")
}

/// Minimal valid task submission (promoted in Sprint 82 Phase Q:
/// consumed by both the migrated coordinator_api tests and the staying
/// http.rs tasks_api tests).
pub(crate) fn make_test_submission() -> nexus_coordinator_rs::types::TaskSubmission {
    nexus_coordinator_rs::types::TaskSubmission {
        project_id: "test-project".into(),
        task_type: "analysis".into(),
        prompt: "Analyze this".into(),
        system_prompt: String::new(),
        model: "llama3".into(),
        priority: 5,
        parent_task_id: String::new(),
        metadata: std::collections::BTreeMap::new(),
        is_open_source: false,
        estimated_watts: 0,
        estimated_vram_mb: 0,
        estimated_hours: 0.0,
        redundancy_factor: 1,
        verifiable: false,
        required_runtime: None,
    }
}
