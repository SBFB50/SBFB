// SPDX-License-Identifier: AGPL-3.0-or-later
//! Curators loopback HTTP domain — extracted verbatim from `http.rs`
//! (Sprint 82 Phase R, PO-10 extended discipline: the domain's 10
//! router-driven tests co-migrated below via `crate::test_support`).
//!
//! Lists the cached signed curator lists plus the attention set
//! (GET /curators, Sprint 7 Phase C), subscribe with the Sprint 20
//! Phase B duress gate and the Sprint 81 Phase E3 hot-join gossip
//! push, unsubscribe, and default-curators (config `[curator]`
//! section, Sprint 11 Phase B). T0 tier: the routes stay registered
//! in `crate::http::build_router` inside `authed_routes` (loopback
//! bearer + Host + Origin) and re-point here by full path; route
//! paths, JSON shapes and status codes are unchanged. The
//! `runtime_error_to_response` helper and the SHARED `ErrorResponse`
//! DTO stay in `http.rs` (consumed by 5 non-curators handlers).

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::{DaemonHttpState, runtime_error_to_response};

/// Body of `POST /curators/subscribe`.
///
/// `deny_unknown_fields` (Sprint 8 audit G-3): the previous
/// definition silently ignored extra JSON fields because that is
/// serde's default. Defense-in-depth demands that a future
/// extension requiring a new field must be wired through on
/// both ends at the same commit — an extra field in the body
/// now fails loud with a 422 instead of being dropped on the
/// floor and forgotten.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscribeCuratorRequest {
    /// Lowercase hex of the curator's Ed25519 public key (64 chars).
    pub curator_pubkey_hex: String,
}

/// Body of both `POST /curators/subscribe` (success) and
/// `DELETE /curators/{pubkey}` (success). Returns the current
/// sorted list of subscribed curator pubkeys so the shell can
/// refresh its UI in a single roundtrip.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubscriptionsResponse {
    pub subscribed_curators: Vec<String>,
}

/// Body of `GET /curators`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CuratorsListResponse {
    /// Every signed curator list the daemon has cached, keyed
    /// by curator pubkey (sorted ascending). Each entry is a
    /// verbatim `CuratorListEntry` — the shell re-renders it
    /// with the trust-by-construction invariant that any entry
    /// in this array has already been verified.
    pub entries: Vec<nexus_core_rs::CuratorListEntry>,
    /// The current attention set (sorted hex pubkeys). The
    /// shell compares this against `entries` to render "waiting
    /// on first announcement" placeholders for subscribed
    /// curators that have not yet broadcast.
    pub subscribed_curators: Vec<String>,
}

/// Body of `GET /default-curators`. Sprint 11 Phase B.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultCuratorsResponse {
    /// Configured default curator Ed25519 public keys (hex).
    pub default_curators: Vec<String>,
}

/// `GET /curators` — list every cached curator list + the
/// current attention set.
pub(crate) async fn list_curators(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /curators");
    let body = CuratorsListResponse {
        entries: state.curator_runtime.list_snapshot(),
        subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
    };
    (StatusCode::OK, Json(body))
}

/// `POST /curators/subscribe` — add a curator pubkey to the
/// attention set. Idempotent: subscribing twice is a no-op that
/// still returns 200.
pub(crate) async fn subscribe_curator(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SubscribeCuratorRequest>,
) -> impl IntoResponse {
    debug!(curator = %req.curator_pubkey_hex, "POST /curators/subscribe");
    // Sprint 20 Phase B : in duress mode, accept the request but
    // do NOT mutate the attention set. The shell still sees a
    // 200 so the UI is quiet; the fake identity never subscribes
    // a real curator.
    if crate::noop_identity::curator_subscribe_in_duress(state.identity_mode)
        == crate::noop_identity::SubscribeOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(SubscriptionsResponse {
                subscribed_curators: Vec::new(),
            }),
        )
            .into_response();
    }
    match state.curator_runtime.subscribe(&req.curator_pubkey_hex) {
        Ok(_) => {
            // Sprint 81 Phase E3: hot-join the freshly subscribed peer on the
            // live gossip topic. Before E3 the bootstrap set was read ONCE at
            // boot (`spawn_gossip_subscribe_task`), so a runtime subscribe
            // never dialed — the browse stayed empty until a daemon restart.
            // Best-effort send, mirror of `browse_pull`: the attention-set
            // mutation above is the durable state, the join is connectivity.
            // The key is dialable iff it IS the peer's endpoint id (true for
            // the observed subscribe-by-node-id flow); a pure signing-curator
            // key makes the join a silent best-effort no-op, exactly like the
            // boot-time bootstrap dial of that same key.
            // Duress safety is by construction: the duress early-return above
            // makes this push unreachable under the decoy key, and this
            // handler is the ONLY producer of `GossipCmd::JoinPeers` (locked
            // by the duress-empty channel test).
            let _ = state
                .gossip_cmd_tx
                .send(crate::runtime::GossipCmd::JoinPeers(vec![
                    req.curator_pubkey_hex.clone(),
                ]))
                .await;
            (
                StatusCode::OK,
                Json(SubscriptionsResponse {
                    subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
                }),
            )
                .into_response()
        }
        Err(e) => runtime_error_to_response(e).into_response(),
    }
}

/// `DELETE /curators/{pubkey}` — remove a curator from the
/// attention set and evict any cached list they had published.
pub(crate) async fn unsubscribe_curator(
    State(state): State<Arc<DaemonHttpState>>,
    Path(pubkey): Path<String>,
) -> impl IntoResponse {
    debug!(curator = %pubkey, "DELETE /curators/{{pubkey}}");
    match state.curator_runtime.unsubscribe(&pubkey) {
        Ok(_) => (
            StatusCode::OK,
            Json(SubscriptionsResponse {
                subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex(),
            }),
        )
            .into_response(),
        Err(e) => runtime_error_to_response(e).into_response(),
    }
}

/// `GET /default-curators` — return the daemon's configured
/// default curator pubkeys from `[curator]` config section.
/// Sprint 11 Phase B.
pub(crate) async fn default_curators(
    State(state): State<Arc<DaemonHttpState>>,
) -> impl IntoResponse {
    debug!("GET /default-curators");
    (
        StatusCode::OK,
        Json(DefaultCuratorsResponse {
            default_curators: state.default_curators.clone(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{KeyPair, PowSolveCache, create_node};
    use nexus_shell_daemon_core::blob_serve::BlobServeCache;
    use nexus_shell_daemon_core::browse::BrowseAggregator;
    use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
    use std::time::SystemTime;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    use crate::test_support::*;

    #[tokio::test]
    async fn list_curators_returns_empty_when_nothing_cached() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: CuratorsListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
        assert!(list.subscribed_curators.is_empty());
    }

    #[tokio::test]
    async fn subscribe_then_list_then_delete_happy_path() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        // POST /curators/subscribe
        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: hex_key.clone(),
        })
        .unwrap();
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(sub.subscribed_curators, vec![hex_key.clone()]);

        // GET /curators must now show the pubkey in the set.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let list: CuratorsListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(list.subscribed_curators, vec![hex_key.clone()]);
        // No entries yet — no real gossip announcement received.
        assert!(list.entries.is_empty());

        // DELETE /curators/{pubkey}
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri(format!("/api/daemon/curators/{hex_key}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(sub.subscribed_curators.is_empty());
    }

    #[tokio::test]
    async fn subscribe_rejects_extra_fields() {
        // Sprint 8 audit G-3: a body that carries an extra
        // field on top of `curator_pubkey_hex` must be rejected
        // by axum's JSON extractor because the DTO now carries
        // `#[serde(deny_unknown_fields)]`. Axum surfaces the
        // serde rejection as HTTP 422 Unprocessable Entity.
        //
        // This is defense-in-depth: it catches the case where a
        // future shell extension starts sending a new field and
        // forgets to update the daemon side. Silent drop used
        // to mask those bugs for a full release cycle — the
        // tightened contract turns them into a first-commit
        // failure instead.
        let app = build_test_router(mk_state().await);
        let body: Vec<u8> = br#"{"curator_pubkey_hex":"aa","evil_field":"surprise"}"#.to_vec();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Axum maps `deny_unknown_fields` rejection to 422.
        // Accept either 400 or 422 in case a future axum version
        // changes the mapping — the important property is that
        // the request is refused outright, not silently ignored.
        assert!(
            resp.status() == StatusCode::UNPROCESSABLE_ENTITY
                || resp.status() == StatusCode::BAD_REQUEST,
            "expected 4xx from deny_unknown_fields, got {}",
            resp.status()
        );
    }

    #[tokio::test]
    async fn subscribe_rejects_bad_pubkey_hex_as_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: "not-hex".to_string(),
        })
        .unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    /// Sprint 81 Phase E3 #1 (CONTROL→GREEN): a Normal-mode
    /// subscribe must push exactly one `GossipCmd::JoinPeers` with
    /// the subscribed pubkey so the live gossip task dials the
    /// peer immediately. Pre-fix this channel stayed EMPTY (the
    /// bootstrap set was read once at boot) — that emptiness was
    /// the reproduced live defect (browse blank until restart).
    #[tokio::test]
    async fn subscribe_curator_pushes_hot_join_for_subscribed_peer() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let app = build_test_router(Arc::clone(&state));
        let kp = KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: hex_key.clone(),
        })
        .unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // The handler awaits the send before responding, so the
        // command is already in the channel once oneshot returns.
        let cmd = rx.try_recv().expect("subscribe must push a gossip command");
        let crate::runtime::GossipCmd::JoinPeers(peers) = cmd else {
            panic!("expected GossipCmd::JoinPeers, got a different command");
        };
        assert_eq!(peers, vec![hex_key]);
        // Exactly one push — no duplicate or stray command.
        assert!(
            matches!(
                rx.try_recv(),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty)
            ),
            "subscribe must push exactly one JoinPeers"
        );
    }

    /// Sprint 81 Phase E3 #2 (negative, duress): under Duress the
    /// handler early-returns BEFORE the subscribe and its hot-join
    /// push, so the gossip command channel must stay EMPTY — zero
    /// new dials under the decoy key. This locks the
    /// by-construction placement (the `Ok` arm is the ONLY
    /// producer of `GossipCmd::JoinPeers`) against regression.
    #[tokio::test]
    async fn subscribe_curator_in_duress_pushes_no_hot_join() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Duress, tx).await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: "cd".repeat(32),
        })
        .unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        // Duress still ACKs 200 (quiet UI) but never joins.
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(
            matches!(
                rx.try_recv(),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty)
            ),
            "duress subscribe must push nothing to the gossip task"
        );
    }

    /// Sprint 81 Phase E3 #3 (negative, invalid key): a pubkey that
    /// fails `parse_pubkey_hex` takes the `Err` arm (400) and must
    /// not push any gossip command — the hot-join only ever fires
    /// for a key the curator runtime actually accepted.
    #[tokio::test]
    async fn subscribe_curator_invalid_hex_pushes_no_hot_join() {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: "not-hex".to_string(),
        })
        .unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert!(
            matches!(
                rx.try_recv(),
                Err(tokio::sync::mpsc::error::TryRecvError::Empty)
            ),
            "a rejected subscribe must push nothing to the gossip task"
        );
    }

    // ---------------------------------------------------------
    // Sprint 11 Phase B: default-curators endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn default_curators_returns_empty_when_unconfigured() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/default-curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: DefaultCuratorsResponse = serde_json::from_slice(&body).unwrap();
        assert!(res.default_curators.is_empty());
    }

    #[tokio::test]
    async fn default_curators_returns_configured_list() {
        let node = create_node().await.expect("boot test node");
        let curator_hex = "ab".repeat(32);
        let tmp = tempfile::tempdir().expect("tempdir");
        let keystore = Arc::new(nexus_core_rs::LocalFileKeyStore::new(tmp.path()));
        let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
            keystore,
            tmp.path().join("state.sqlite"),
            tmp.path().join("blob-cache"),
            Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
        ));
        std::mem::forget(tmp);
        let state = Arc::new(DaemonHttpState {
            node_id: node.node_id(),
            daemon_version: "0.1.0-test".to_string(),
            boot_time: SystemTime::now(),
            api_host: "127.0.0.1".to_string(),
            api_port: 12345,
            curator_runtime: Arc::new(CuratorRuntime::new(None)),
            browse_aggregator: Arc::new(BrowseAggregator::new()),
            node: Arc::new(node),
            gossip_sender: Arc::new(RwLock::new(None)),
            gossip_cmd_tx: tokio::sync::mpsc::channel(8).0,
            default_curators: vec![curator_hex.clone()],
            blob_serve_cache: Arc::new(BlobServeCache::new(8)),
            identity_mode: nexus_core_rs::IdentityMode::Normal,
            panic_wipe,
            pow_solve_cache: Arc::new(PowSolveCache::new()),
            pow_policy: nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
            pow_keypair: Arc::new(KeyPair::generate()),
            curator_gossip_topic: nexus_shell_daemon_core::iroh_runtime::curator_topic_id(),
            coordinator_db: std::sync::Arc::new(std::sync::Mutex::new(
                nexus_coordinator_rs::db::CoordinatorDb::open_in_memory()
                    .expect("test coordinator DB"),
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
            feed_rate_limiter: Arc::new(
                nexus_shell_daemon_core::feed_limiter::FeedRateLimiter::new(),
            ),
            feed_join_handles: Arc::new(std::sync::Mutex::new(Vec::new())),
            feed_join_shutdown: Arc::new(tokio::sync::watch::channel(false).0),
            preview_store: nexus_shell_daemon_core::preview::PreviewStore::new(
                nexus_shell_daemon_core::preview::DEFAULT_TTL,
            ),
            seed_registry: Arc::new(crate::seed_registry::SeedRegistry::new()),
            shard_sessions: Arc::new(crate::shard_session::ShardSessionRegistry::default()),
        });
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/default-curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: DefaultCuratorsResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.default_curators, vec![curator_hex]);
    }

    /// #B-rt-2 The `/curators/subscribe` handler in Duress mode
    /// returns 200 but does NOT add the pubkey to the attention
    /// set. The peer observer sees an ACK; the local state is
    /// silently untouched.
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_rejects_curator_subscribe_real() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));
        let curator_hex = "cd".repeat(32);

        let body = serde_json::to_vec(&SubscribeCuratorRequest {
            curator_pubkey_hex: curator_hex.clone(),
        })
        .unwrap();

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/curators/subscribe")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let sub_resp: SubscriptionsResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            sub_resp.subscribed_curators.is_empty(),
            "Duress mode must not grow the attention set"
        );

        // Verify the curator runtime itself stays empty.
        assert!(
            state.curator_runtime.subscribed_pubkeys_hex().is_empty(),
            "curator_runtime attention set must not mutate under Duress"
        );
    }
}
