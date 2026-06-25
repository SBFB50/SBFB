// SPDX-License-Identifier: AGPL-3.0-or-later
//! HTTP surface for `nexus-shell-daemon`.
//!
//! The daemon's HTTP listener is loopback-only. Daemon-specific
//! JSON routes live under `/api/daemon/*` so they never collide
//! with SPA document routes (`/browse`, `/curators`) when the
//! daemon serves the React shell via `--web-root`.
//!
//! - `GET    /health`                         — liveness probe (public)
//! - `GET    /api/daemon/info`                — daemon state snapshot
//! - `GET    /api/daemon/curators`            — list every cached curator list
//! - `POST   /api/daemon/curators/subscribe`  — add a curator to the attention set
//! - `DELETE /api/daemon/curators/{pubkey}`   — remove a curator
//! - `GET    /api/daemon/browse`              — aggregated browse entries
//! - `GET    /api/daemon/nodes`               — subscribed node directories (catalog publishers)
//! - `POST   /api/daemon/publish`             — publish a project announcement
//! - `POST   /api/daemon/publish-blob`        — upload a zip archive blob
//! - `POST   /api/daemon/directory/publish`   — publish this node's signed catalog
//! - `GET    /api/daemon/default-curators`    — config-provided curator list
//! - `POST   /api/daemon/panic/wipe`          — irreversible identity wipe
//! - `GET    /api/daemon/diagnostic/neighborhood` — peer snapshot
//!
//! ## CORS
//!
//! By default the daemon trusts only loopback origins:
//!
//! - `http://127.0.0.1[:port]`
//! - `http://localhost[:port]`
//!
//! The `--cors-origin` CLI flag (repeatable) extends the
//! allowlist with extra origins for multi-node access.
//! The env fallback `NEXUS_DAEMON_CORS_ORIGINS` (comma-
//! separated) is merged when the flag is absent.

use std::path::Path as FsPath;
use std::sync::Arc;
use std::time::SystemTime;

use axum::{
    Router,
    body::Bytes,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post},
};
use nexus_core_rs::{
    BlobsClient, KeyPair, Node, PowEnvelope, PowSolveCache, RelayPowPolicy, TopicSender,
};
use nexus_shell_daemon_core::auth::{AuthState, auth_required};
use nexus_shell_daemon_core::blob_serve::{self, BlobServeCache};
use nexus_shell_daemon_core::browse::{BrowseAggregatorHandle, BrowseEntry};
use nexus_shell_daemon_core::iroh_runtime::{CuratorRuntimeError, CuratorRuntimeHandle};
use nexus_shell_daemon_core::state::{DaemonStateSnapshot, StateInputs};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info, warn};

/// Shared handle to the gossip topic sender. `None` until the
/// gossip task has joined the curator topic. Sprint 11 Phase A.
pub type GossipSenderHandle = Arc<RwLock<Option<TopicSender>>>;

/// Shared state handed to every axum route.
///
/// Holds the static fields needed by `/info` plus the
/// [`CuratorRuntimeHandle`] that curator routes and `/info`'s
/// subscribed-curators / known-lists / known-browse-entries
/// counters read from, the Phase D [`BrowseAggregatorHandle`],
/// and an `Arc<Node>` so the browse handler can drive the
/// [`nexus_core_rs::DiscoveryClient::probe_reachable`] path.
#[derive(Debug, Clone)]
pub struct DaemonHttpState {
    pub node_id: String,
    pub daemon_version: String,
    pub boot_time: SystemTime,
    pub api_host: String,
    pub api_port: u16,
    pub curator_runtime: CuratorRuntimeHandle,
    pub browse_aggregator: BrowseAggregatorHandle,
    /// Shared iroh node handle. The browse route reaches
    /// through the Arc to call `DiscoveryClient::probe_reachable`
    /// on the endpoint.
    pub node: Arc<Node>,
    /// Gossip topic sender handle. `None` until the gossip task
    /// has joined the curator topic. Used by `POST /publish` to
    /// broadcast project announcements. Sprint 11 Phase A.
    pub gossip_sender: GossipSenderHandle,
    /// Channel to push commands to the gossip task (outbox,
    /// republish). Sprint 53 Phase D.
    pub gossip_cmd_tx: crate::runtime::GossipCmdTx,
    /// Default curator pubkeys from `[curator]` config section.
    /// Sprint 11 Phase B. Exposed via `GET /default-curators`.
    pub default_curators: Vec<String>,
    /// Sprint 12 Phase A: LRU cache of decompressed zip archives
    /// for the blob-serve endpoint.
    pub blob_serve_cache: Arc<BlobServeCache>,
    /// Sprint 20 Phase B : duress-mode flag. Set to
    /// [`IdentityMode::Duress`] when the daemon was booted via an
    /// identity unlocked under the duress PIN. Every outbound
    /// handler routes through `crate::noop_identity` to noop
    /// publishes / subscribes under the fake keypair. Normal
    /// boots leave this at `IdentityMode::Normal` (the default).
    pub identity_mode: nexus_core_rs::IdentityMode,
    /// Sprint 20 Phase B : panic wipe service, provisioned at
    /// boot with the real keystore + state-db + blob-cache paths
    /// and the production [`crate::panic::RealExit`] strategy.
    /// Consumed by the `POST /panic/wipe` handler.
    pub panic_wipe: Arc<crate::panic::PanicWipeService>,
    /// Sprint 20 Phase C : PoW solve cache (publisher side). Each
    /// outbound broadcast wraps its payload with a Hashcash proof
    /// minted via [`PowSolveCache::ensure_proof`]. The cache keeps
    /// one live proof per topic for 15 min, so a chatty publisher
    /// pays the ~100 ms solve twice an hour, not per-message.
    pub pow_solve_cache: Arc<PowSolveCache>,
    /// Sprint 20 Phase C : shared PoW policy handle. Read on every
    /// solve so a hot-reloaded `relay_pow_policy.toml` (cf.
    /// [`nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher`])
    /// takes effect for the next outbound broadcast without a
    /// restart.
    pub pow_policy: Arc<std::sync::RwLock<RelayPowPolicy>>,
    /// Sprint 20 Phase C : the daemon's long-lived Ed25519 keypair,
    /// used as the `publisher_pubkey` anchor in Hashcash challenges.
    /// When the launcher's `sbfb unlock` hands over a persistent
    /// identity, the keypair matches the node's iroh secret ; on
    /// legacy `cargo run` paths a fresh ephemeral keypair is minted
    /// alongside the ephemeral iroh identity.
    pub pow_keypair: Arc<KeyPair>,
    /// Sprint 20 Phase C : pre-computed 32-byte curator gossip
    /// topic id. The publish handler uses this to key the
    /// [`PowSolveCache`] instead of recomputing the BLAKE3 hash on
    /// every broadcast.
    pub curator_gossip_topic: [u8; 32],
    /// Sprint 36 Phase A : shared coordinator DB for the Rust-native
    /// task dispatcher and result validator. Opened once at boot from
    /// `~/.sbfb/coordinator.db` (WAL mode). Handlers lock briefly
    /// for each SQL operation (~1 ms).
    pub coordinator_db: std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    pub result_event_tx: crate::validator_loop::ResultEventSender,
    /// Sprint 39 Phase B : warrant canary observation registry (Rust
    /// port of canary_registry.py). Tracks observed canary signings
    /// and duress acks, computes per-pubkey freshness.
    pub canary_registry:
        std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::canary_registry::CanaryRegistry>>,
    pub canary_input:
        Option<std::sync::Arc<nexus_coordinator_rs::canary_input::CanaryInputManager>>,
    pub sbfb_home: Option<std::path::PathBuf>,
    /// Sprint 49 Phase A: handle to the project iroh-docs document.
    /// `Some` in coordinator mode (daemon start), `None` in tests
    /// that don't need doc wiring. Read by the task submit handler
    /// (dispatch + on-demand local worker spawn) and, since Sprint 76
    /// Phase H, by `GET /api/daemon/project-info` (exposes the doc id
    /// so the bridge routes a compute task to the node's own worker).
    pub project_doc: Option<std::sync::Arc<nexus_core_rs::docs::DocHandle>>,
    /// Sprint 49 Phase A: MPSC sender for the dispatch loop. HTTP
    /// task submit handler sends signed TaskEntry values here; the
    /// dispatch loop writes them to the project doc sequentially.
    pub task_dispatch_tx: Option<crate::dispatch_loop::TaskEntrySender>,
    /// 2026-06-05 hotfix #5 (maillon A): supervises the on-demand
    /// co-located compute worker. The task submit handler nudges it
    /// (`ensure_spawned`) so a node executes its own Network tasks
    /// without the user running `nexus-worker` by hand.
    pub local_worker: std::sync::Arc<crate::local_worker::LocalWorkerSupervisor>,
    /// Sprint 56 Phase C: per-app in-memory key-value storage for the
    /// bridge `storage_*` methods.
    pub app_storage: crate::storage_api::AppStorage,
    /// Sprint 58 Phase C: per-app iroh-docs storage namespaces for
    /// P2P replicated apps. Keyed by app name (e.g. "sbfb-ideas").
    pub storage_namespaces: crate::storage_api::StorageNamespaces,
    /// Sprint 59 Phase C: per-author per-app GCRA rate limiter for
    /// storage write endpoints. 10 writes/min/author/app.
    pub storage_write_limiter: Arc<nexus_shell_daemon_core::storage_limiter::StorageWriteLimiter>,
    /// Sprint 62 Phase B: feed sync state (iroh-docs namespace for
    /// the public feed). `None` if boot_feed_namespace failed.
    pub feed_sync_state: Option<Arc<crate::feed_sync::FeedSyncState>>,
    /// Sprint 62 Phase D: per-author GCRA rate limiter for remote
    /// feed entry ingestion. 5 ops/min/author.
    pub feed_rate_limiter: Arc<nexus_shell_daemon_core::feed_limiter::FeedRateLimiter>,
    /// Sprint 66 Phase C: tracked JoinHandles for feed_join spawned
    /// tasks, drained at shutdown for clean join.
    pub feed_join_handles: Arc<std::sync::Mutex<Vec<tokio::task::JoinHandle<()>>>>,
    /// Sprint 66 Phase C: shutdown signal for feed_join tasks.
    /// Each task subscribes to get a Receiver.
    pub feed_join_shutdown: Arc<tokio::sync::watch::Sender<bool>>,
    /// Sprint 68 Phase B: ephemeral preview store for
    /// `sbfb-factory preview` uploads.
    pub preview_store: nexus_shell_daemon_core::preview::PreviewStore,
    /// Sprint 74 Phase F: best-effort in-memory multi-seed registry. Fed by the
    /// feed ingest path with REMOTE `SeedAnnounced` ops; read by
    /// `GET /api/daemon/seed-count/{project_id}` to render "Toi + N pairs (vus
    /// recemment)". Ephemeral by design (a freshness count has no value outside
    /// its TTL window).
    pub seed_registry: Arc<crate::seed_registry::SeedRegistry>,
}

impl DaemonHttpState {
    fn snapshot(&self) -> DaemonStateSnapshot {
        // Read the live curator runtime counts rather than
        // leaving the Phase A zero-fallback in place. This is
        // additive — the snapshot schema stays at v1, only the
        // populated values change.
        let subscribed_curators = self.curator_runtime.subscribed_pubkeys_hex();
        let known_lists = self.curator_runtime.known_list_count() as u32;
        let known_browse_entries = self.curator_runtime.known_entry_count() as u32;

        DaemonStateSnapshot::from_inputs(StateInputs {
            node_id: self.node_id.clone(),
            daemon_version: self.daemon_version.clone(),
            boot_time: self.boot_time,
            api_host: self.api_host.clone(),
            api_port: self.api_port,
            subscribed_curators,
            known_lists,
            known_browse_entries,
        })
    }
}

/// Build the axum [`Router`] carrying every Phase A + Phase C
/// route. The caller hands us an [`Arc<DaemonHttpState>`] plus
/// the loopback bearer token; the router clones the state into
/// each handler via the axum `State` extractor and applies the
/// [`auth_required`] middleware on every non-public route.
///
/// ## Auth exemption (Sprint 16 Phase A)
///
/// Two route surfaces bypass the bearer/Host/Origin middleware:
///
/// - `GET /health` — liveness probe, reached by the launcher
///   before it has the token.
/// - `/blob-serve/{hash}/{*path}` — served into a sandboxed iframe
///   with CSP `connect-src 'none'` (Sprint 12 Phase A). The
///   iframe cannot inject a custom header, and its Origin is
///   `null` because the sandbox strips same-origin. The blob
///   content is already public by construction (anyone on the
///   P2P network can fetch the zip by hash), so exempting the
///   route does not leak anything new.
pub fn build_router(
    state: Arc<DaemonHttpState>,
    auth: AuthState,
    cors_origins: &[String],
    web_root: Option<&FsPath>,
) -> Router {
    // Sprint 13 Phase A (T37): blob-serve routes get a CSP
    // middleware that injects security headers on ALL responses
    // (200, 400, 404, 500) — not just the success path.
    let blob_serve_routes = Router::new()
        .route("/{hash}/{*path}", get(blob_serve))
        .layer(middleware::from_fn(blob_serve_csp_middleware));

    // Public routes: no bearer, no Host check, no Origin check.
    let public_routes = Router::new()
        .route("/health", get(health))
        .nest("/blob-serve", blob_serve_routes);

    // Public token bootstrap: the React shell served by this
    // daemon needs the bearer token to call authenticated routes.
    // Host + Origin loopback checks (inside handler) prevent
    // DNS rebinding and cross-origin leaks — same checks as
    // auth_required minus the bearer token itself.
    let auth_for_token = auth.clone();
    let token_route = Router::new()
        .route("/auth/token", get(auth_token_public))
        .with_state(auth_for_token);

    // Sprint 18 audit fix D-1 : the caller picks the variant
    // (`AuthState::Static` for the legacy single-token boot path,
    // `AuthState::Rotated` once the launcher writes a `tokens.json`).
    // The middleware reads the inner state on every request so a
    // rotation reaches `auth_required` without rebuilding the router.

    // Authenticated surface: every other route requires
    // X-SBFB-Token + loopback Host + (absent or loopback) Origin.
    let authed_routes = Router::new()
        .route("/api/daemon/info", get(info))
        .route("/api/daemon/project-info", get(project_info))
        .route("/api/daemon/curators", get(list_curators))
        .route("/api/daemon/curators/subscribe", post(subscribe_curator))
        .route("/api/daemon/curators/{pubkey}", delete(unsubscribe_curator))
        .route("/api/daemon/browse", get(list_browse))
        .route("/api/daemon/browse/pull", post(browse_pull))
        // Sprint 74 Phase D: toggle a self-deployed app's local keep-online pin.
        .route("/api/daemon/keep-online", post(set_keep_online))
        // Sprint 74 Phase E: cross-node seed. `/seed` = voluntary community
        // seed of a distant public app; `/seed/invite*` = revocable invite
        // ledger for the authenticated `sbfb/seed/0` protocol.
        .route("/api/daemon/seed", post(seed_voluntary))
        // Sprint 75 Phase E: REQUESTER leg of the authenticated
        // `sbfb/seed/0` protocol — ask a designated peer (my anchor) to
        // seed an app this node holds. Scriptable, headless-compatible.
        .route("/api/daemon/seed/request", post(seed_request_peer))
        .route("/api/daemon/seed/invite", post(seed_invite_mint))
        .route("/api/daemon/seed/invite/revoke", post(seed_invite_revoke))
        .route(
            "/api/daemon/seed/invites/{project_id}",
            get(seed_invite_list),
        )
        // Sprint 74 Phase F: best-effort multi-seed availability count.
        .route("/api/daemon/seed-count/{project_id}", get(seed_count))
        // Sprint 75 Phase D: node identity exposure — the subscribed node
        // directories grouped by publishing node (read-only projection).
        .route("/api/daemon/nodes", get(list_nodes))
        // Sprint 77 Phase J: read-only status of a private compute-group shard
        // session. Control-plane only — an AGGREGATE status (member count),
        // NEVER the group's member identities. Same loopback bearer+Host+Origin
        // tier as its siblings (authed_routes).
        .route("/api/daemon/shard-session/{session_id}", get(shard_session))
        .route("/api/daemon/publish", post(publish_project))
        .route("/api/daemon/publish-blob", post(publish_blob))
        .route("/api/daemon/directory/publish", post(publish_directory))
        .route("/api/daemon/default-curators", get(default_curators))
        .route("/api/daemon/panic/wipe", post(panic_wipe))
        .route(
            "/api/v1/contributor/verify/{project_id}/{node_id_hex}",
            get(crate::contributor_api::verify_contributor),
        )
        .route(
            "/api/v1/contributor/project/{project_id}",
            get(crate::contributor_api::list_contributors),
        )
        .route(
            "/api/v1/contributor/envelope/{project_id}/{node_id_hex}",
            get(crate::contributor_api::envelope),
        )
        // Sprint 23 Phase E : diagnostic neighborhood snapshot.
        // Returns the node's own ID and known peer IDs from the
        // iroh endpoint's remote info table. Diagnostic-only, no
        // wire format impact.
        .route(
            "/api/daemon/diagnostic/neighborhood",
            get(diagnostic_neighborhood),
        )
        // Sprint 30 Phase C : FROST DKG + ceremony admin endpoints.
        // Trust tier T0 — behind the same loopback bearer + Host +
        // Origin gate as every other authenticated route.
        .route(
            "/api/canary/frost/trusted-dealer",
            post(frost_trusted_dealer),
        )
        .route("/api/canary/frost/round1", post(frost_round1))
        .route("/api/canary/frost/round2", post(frost_round2))
        .route("/api/canary/frost/aggregate", post(frost_aggregate))
        .route("/api/v1/tasks/submit", post(coordinator_submit_task))
        .route("/api/v1/results/submit", post(coordinator_submit_result))
        .route("/api/v1/kudos/{project_id}", get(coordinator_get_kudos))
        .route(
            "/api/v1/kudos/{project_id}/verify",
            get(coordinator_verify_chain),
        )
        .route("/api/canary/observed", post(canary_observed))
        .route("/api/canary/network-health", get(canary_network_health))
        .route("/api/canary/freshness/{pubkey}", get(canary_freshness))
        .route(
            "/api/canary/inject-rate",
            post(crate::canary_api::set_inject_rate),
        )
        .route(
            "/api/canary/observed-divergence",
            get(crate::canary_api::observed_divergence),
        )
        .route("/api/v1/apps", get(crate::apps::list_apps))
        .route("/api/v1/apps/{project_id}", get(crate::apps::get_app))
        .route("/app/{name}/state", get(crate::storage_api::storage_list))
        .route(
            "/app/{name}/state/{key}",
            get(crate::storage_api::storage_get)
                .post(crate::storage_api::storage_set)
                .delete(crate::storage_api::storage_delete),
        )
        .route(
            "/api/daemon/storage/ticket/{app}",
            get(crate::storage_api::storage_ticket),
        )
        .route(
            "/api/daemon/storage/join",
            post(crate::storage_api::storage_join),
        )
        .route(
            "/api/daemon/storage/{app}/version",
            get(crate::storage_api::storage_version),
        )
        .route(
            "/api/daemon/feed/ticket",
            get(crate::feed_sync::feed_ticket),
        )
        .route("/api/daemon/feed/join", post(crate::feed_sync::feed_join))
        .route(
            "/api/daemon/feed/status",
            get(crate::feed_sync::feed_status),
        )
        .route(
            "/api/daemon/feed/insert",
            post(crate::feed_sync::feed_insert),
        )
        .route("/api/daemon/feed/cursor", get(get_feed_cursor))
        .route("/api/daemon/feed/entries", get(get_feed_entries))
        .route("/api/daemon/search", get(search_handler))
        .route("/api/daemon/proof-card/{project_id}", get(get_proof_card))
        .route("/api/v1/preview/load", post(preview_load))
        // Raw-zip deploy routes carry a body up to MAX_DEPLOY_BYTES; override
        // axum's 2 MB default body limit per-route so the handler's own
        // PAYLOAD_TOO_LARGE check is the real ceiling (a non-trivial forked
        // workspace easily exceeds 2 MB). Scoped to these routes so other
        // endpoints keep the safe small default.
        .route(
            "/api/v1/deploy",
            post(crate::deploy::deploy_private).layer(axum::extract::DefaultBodyLimit::max(
                crate::deploy::MAX_DEPLOY_BYTES,
            )),
        )
        .route(
            "/api/v1/deploy-from-repo",
            post(crate::deploy::deploy_from_repo),
        )
        // Sprint 74 Phase C : redeploy a locally forked/edited workspace under
        // this node's identity (atelier-fork loop). Fresh local-signed
        // provenance, is_open_source forced false (self-attestation only).
        .route(
            "/api/v1/deploy-workspace",
            post(crate::deploy::deploy_workspace).layer(axum::extract::DefaultBodyLimit::max(
                crate::deploy::MAX_DEPLOY_BYTES,
            )),
        )
        .route(
            "/api/v1/project/{project_id}/provenance",
            get(get_provenance),
        )
        .route("/api/v1/consent", get(crate::consent::get_consent))
        .route("/api/v1/consent/set", post(crate::consent::set_consent))
        .route(
            "/api/v1/consent/whitelist/add",
            post(crate::consent::whitelist_add),
        )
        .route(
            "/api/v1/consent/whitelist/remove",
            post(crate::consent::whitelist_remove),
        )
        .route("/api/v1/files/upload", post(crate::files::upload_file))
        .route(
            "/api/v1/files/{sha256}/manifest",
            get(crate::files::get_manifest),
        )
        .route("/api/v1/files/{sha256}", get(crate::files::stream_file))
        // Sprint 44 Phase B : health + shell + kudos + diagnostic
        .route(
            "/api/v1/coordinator/health",
            get(crate::health_api::coordinator_health),
        )
        .route("/api/v1/shell/discover", get(crate::shell_api::discover))
        .route("/api/v1/kudos/entries", get(crate::kudos_api::list_entries))
        .route(
            "/api/v1/kudos/{project_id}/leaderboard",
            get(crate::kudos_api::leaderboard),
        )
        // Sprint 76 Phase E (D4): per-node contribution dashboard. Distinct
        // top-level resource (not under /kudos/{project_id}) to avoid any
        // route-shadowing with the per-project leaderboard. Authed_routes =
        // loopback bearer + Host + Origin gate.
        .route(
            "/api/v1/contributor/{node_id}",
            get(crate::kudos_api::contributor_dashboard),
        )
        .route(
            "/api/v1/diagnostic/fairness",
            get(crate::diagnostic_api::fairness_metrics),
        )
        // Sprint 44 Phase C : tasks + worker_state
        .route("/api/v1/tasks", get(crate::tasks_api::list_tasks))
        .route("/api/v1/tasks/{task_id}", get(crate::tasks_api::get_task))
        .route(
            "/api/v1/tasks/{task_id}/result",
            get(crate::tasks_api::get_task_result),
        )
        .route(
            "/api/v1/worker/state",
            get(crate::worker_state_api::get_worker_state),
        )
        // Sprint 45 Phase A : invite + quarantine
        .route(
            "/api/v1/invite/create",
            post(crate::invite_api::create_invite),
        )
        .route("/api/v1/invite", get(crate::invite_api::list_invites))
        .route(
            "/api/v1/invite/{invite_id}",
            delete(crate::invite_api::revoke_invite),
        )
        .route(
            "/api/v1/quarantine",
            get(crate::quarantine_api::list_quarantine),
        )
        .route(
            "/api/v1/quarantine/{row_id}/flush",
            post(crate::quarantine_api::flush_quarantine),
        )
        .route(
            "/api/v1/quarantine/{row_id}/drop",
            post(crate::quarantine_api::drop_quarantine),
        )
        .layer(middleware::from_fn_with_state(auth, auth_required));

    let app = Router::new()
        .merge(public_routes)
        .merge(token_route)
        .merge(authed_routes)
        .with_state(state)
        .layer(cors_layer(cors_origins));

    if let Some(root) = web_root {
        let serve = ServeDir::new(root).fallback(ServeFile::new(root.join("index.html")));
        app.fallback_service(serve)
    } else {
        app
    }
}

/// Public endpoint returning the bearer token so the React shell
/// can bootstrap auth from the same origin. Protected by
/// Host + Origin loopback checks (no bearer required — that's
/// what we're handing out).
async fn auth_token_public(State(auth): State<AuthState>, req: Request) -> impl IntoResponse {
    let host_ok = req
        .headers()
        .get(axum::http::header::HOST)
        .and_then(|v| v.to_str().ok())
        .map(nexus_shell_daemon_core::auth::is_loopback_host)
        .unwrap_or(false);
    if !host_ok {
        return (StatusCode::FORBIDDEN, "host not allowed").into_response();
    }
    if let Some(origin) = req.headers().get(axum::http::header::ORIGIN) {
        let ok = origin
            .to_str()
            .ok()
            .map(nexus_shell_daemon_core::auth::is_loopback_origin)
            .unwrap_or(false);
        if !ok {
            return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
        }
    }
    match auth.current_token() {
        Some(token) => Json(serde_json::json!({ "token": token })).into_response(),
        None => (StatusCode::SERVICE_UNAVAILABLE, "token unavailable").into_response(),
    }
}

/// Middleware that injects security headers on every blob-serve
/// response, including error responses.
async fn blob_serve_csp_middleware(request: Request, next: Next) -> impl IntoResponse {
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

/// Build a CORS layer that always accepts loopback origins and
/// optionally accepts extra origins passed via `--cors-origin`.
fn cors_layer(extra_origins: &[String]) -> CorsLayer {
    if extra_origins.is_empty() {
        return CorsLayer::new().allow_origin(AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts: &_| is_loopback_origin(origin),
        ));
    }
    let allowed: Vec<HeaderValue> = extra_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o.as_str()).ok())
        .collect();
    CorsLayer::new().allow_origin(AllowOrigin::predicate(
        move |origin: &HeaderValue, _request_parts: &_| {
            is_loopback_origin(origin) || allowed.iter().any(|a| a == origin)
        },
    ))
}

/// Return `true` iff `origin` looks like a valid HTTP(S) origin
/// (scheme + host + optional port, no path). Used to reject
/// obviously malformed `--cors-origin` values at boot.
pub fn is_valid_origin(s: &str) -> bool {
    let rest = match s.strip_prefix("http://") {
        Some(r) => r,
        None => match s.strip_prefix("https://") {
            Some(r) => r,
            None => return false,
        },
    };
    if rest.is_empty() || rest.contains('/') {
        return false;
    }
    let (host, port_opt) = match rest.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (rest, None),
    };
    if host.is_empty() {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

/// Return `true` iff `origin` is an HTTP loopback URL with an
/// optional port and no path.
pub fn is_loopback_origin(origin: &HeaderValue) -> bool {
    let Ok(s) = origin.to_str() else {
        return false;
    };
    let rest = match s.strip_prefix("http://") {
        Some(r) => r,
        None => return false,
    };

    let host_port = rest.split('/').next().unwrap_or(rest);
    let (host, port_opt) = match host_port.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (host_port, None),
    };

    if host != "127.0.0.1" && host != "localhost" {
        return false;
    }
    if let Some(p) = port_opt {
        if p.parse::<u16>().is_err() {
            return false;
        }
    }
    true
}

// =================================================================
// Request / response DTOs
// =================================================================

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

/// Body of `GET /browse`.
///
/// Sorted flat list of every project entry across every cached
/// curator list, each row carrying a reachability bucket the
/// React shell renders as a coloured dot.
/// Test-only deserialization target for the `/browse` JSON. Production serves
/// the response via [`BrowseEntryView`] (BrowseEntry + the derived `is_own` +
/// `from_subscribed`), not this struct; the tests deserialize into it and
/// `BrowseEntry` simply ignores the extra derived keys (no
/// `deny_unknown_fields` on the entry).
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowseListResponse {
    pub entries: Vec<BrowseEntry>,
}

/// Body of `POST /publish`. Sprint 11 Phase A, extended Sprint 12.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    /// Project display name.
    pub project_name: String,
    /// Category tag (e.g. `"gov"`, `"investigation"`).
    pub category: String,
    /// Short description.
    pub description: String,
    /// List of app names available on this project.
    #[serde(default)]
    pub apps: Vec<String>,
    /// Hex hash of a zip blob already stored via `POST /publish-blob`.
    /// If present, the daemon mints a BlobTicket and includes it in
    /// the gossip announcement (Sprint 12 Phase A).
    #[serde(default)]
    pub archive_hash: Option<String>,
    /// URL of the public source code repository (Sprint 13 Phase B).
    #[serde(default)]
    pub repo_url: Option<String>,
    /// BLAKE3 hex hash of provenance.json (Sprint 14 Phase B).
    #[serde(default)]
    pub provenance_hash: Option<String>,
    /// Whether this project was deployed from a public repo with
    /// signed provenance (Sprint 16 Phase D). The coordinator sets
    /// this on every `deploy-from-repo` publish; private zip uploads
    /// and the legacy auto-publish path leave it at `false`. Workers
    /// running at consent level `OpenSource` only accept tasks from
    /// projects where this flag is true.
    #[serde(default)]
    pub is_open_source: bool,
}

/// Body of `POST /publish` (success). Sprint 11 Phase A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResponse {
    pub published: bool,
}

/// Body of `GET /default-curators`. Sprint 11 Phase B.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultCuratorsResponse {
    /// Configured default curator Ed25519 public keys (hex).
    pub default_curators: Vec<String>,
}

/// Body of `POST /publish-blob` (success). Sprint 12 Phase A.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishBlobResponse {
    /// Hex-encoded BLAKE3 hash of the stored blob.
    pub hash: String,
}

/// Body of `GET /diagnostic/neighborhood`. Sprint 23 Phase E.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NeighborhoodResponse {
    pub node_id: String,
    pub peers: Vec<String>,
}

/// Body returned when a curator runtime error must be surfaced
/// as a 4xx/5xx response.
#[derive(Debug, Clone, Serialize)]
struct ErrorResponse {
    error: String,
}

fn runtime_error_to_response(err: CuratorRuntimeError) -> (StatusCode, Json<ErrorResponse>) {
    // Sprint 8 audit C-2 split: `NotSubscribed` and
    // `EnvelopeMismatch` inherit the legacy 422 mapping, so
    // subscribe/unsubscribe callers (which never surface them in
    // practice) continue to behave identically. The split
    // matters to the gossip handler, not to the HTTP surface.
    let status = match &err {
        CuratorRuntimeError::BadPubkeyHex(_) => StatusCode::BAD_REQUEST,
        CuratorRuntimeError::AnnouncementParse(_)
        | CuratorRuntimeError::AnnouncementVersion { .. }
        | CuratorRuntimeError::NotSubscribed { .. }
        | CuratorRuntimeError::EnvelopeMismatch { .. }
        | CuratorRuntimeError::EntryParse(_)
        | CuratorRuntimeError::EntryVerify(_)
        | CuratorRuntimeError::RevisionRollback { .. }
        | CuratorRuntimeError::BlobFetch(_) => StatusCode::UNPROCESSABLE_ENTITY,
        CuratorRuntimeError::Persistence { .. } => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (
        status,
        Json(ErrorResponse {
            error: err.to_string(),
        }),
    )
}

// =================================================================
// Handlers
// =================================================================

/// `GET /health` — liveness probe.
async fn health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /health");
    Json(serde_json::json!({
        "status": "ok",
        "schema_version": nexus_shell_daemon_core::state::SCHEMA_VERSION,
        "daemon_version": state.daemon_version,
    }))
}

/// `GET /info` — full [`DaemonStateSnapshot`] for the shell's
/// Browse / Curators page header.
async fn info(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /info");
    (StatusCode::OK, Json(state.snapshot()))
}

/// `GET /project-info` — expose the local project doc id (Sprint 76
/// Phase H). An iframe app submits a compute task via the bridge, but
/// the node's on-demand local worker is whitelisted to exactly this
/// `project_doc.id()` (`local_worker.rs` provisioning): a task that
/// carries any other `project_id` is never claimed and its result
/// never materializes. The browser cannot derive the doc id (it is
/// not the daemon `node_id`), so the host bridge reads it here and
/// injects it as the submission's `project_id`. Read-only, same
/// loopback auth tier as the rest of `/api/daemon/*`; the id is not a
/// secret — it is already shared to the local worker via a write
/// ticket. `null` when no project doc is mounted yet.
async fn project_info(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /project-info");
    let project_doc_id = state.project_doc.as_ref().map(|doc| doc.id().to_string());
    (
        StatusCode::OK,
        Json(serde_json::json!({ "project_doc_id": project_doc_id })),
    )
}

/// `GET /curators` — list every cached curator list + the
/// current attention set.
async fn list_curators(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
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
async fn subscribe_curator(
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

/// `DELETE /curators/{pubkey}` — remove a curator from the
/// attention set and evict any cached list they had published.
async fn unsubscribe_curator(
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

/// `GET /browse` — Phase D reachability-annotated view of every
/// project across every cached curator list.
///
/// The aggregator flattens the Phase C curator runtime's list
/// snapshot, probes each referenced project endpoint (honouring
/// the TTL cache), and returns a sorted vector the React shell
/// renders as a Browse page. If the curator runtime is empty
/// (no subscribed curators, or no announcements received yet)
/// this returns `{"entries": []}` at 200 rather than an error —
/// the shell renders an empty-state card in that case.
/// A browse entry plus the daemon-derived `is_own` flag (KEEP-ONLINE-READ-PATH,
/// carry S74 Phase G). `is_own` is true iff the entry's hosting `node_id` equals
/// THIS node's id — the precise "did this node publish it" signal. It fixes the
/// shell's old `isOwn = (node_id === project_id)` heuristic, which is always
/// false for per-app deploys whose `project_id = blake3(name) != node_id`, so
/// the owner "Garder en ligne" toggle never rendered. A voluntarily-seeded
/// distant app keeps the AUTHOR's node_id, so it is correctly `is_own = false`
/// (the shell shows the volunteer CTA, never the owner toggle). `node_id` itself
/// stays `#[serde(skip)]`; only this derived boolean crosses to the shell.
/// UX-ARRIVAL (post-S75): `from_subscribed` is the second derived flag — the
/// shell uses it to split the arrival grid (MY sources) from the separate
/// "Découvert sur le réseau" section without un-skipping `node_id`.
///
/// The flag is CATALOG-BACKED, never attention-set-membership of the claimed
/// `node_id` alone (review SEC-UXARR-1/WIRE-UXA-1, skeptics-confirmed P1): a
/// `ProjectAnnouncement` carries NO signature, so its `node_id` is a freely
/// claimed string — deriving trust placement from "claimed node_id is
/// subscribed" would let one PoW-paying announcer name a public anchor's
/// pubkey and land an attacker app inside "Tes sources" (and the hero). So a
/// `direct` entry is `from_subscribed` only when the `(project_id,
/// archive_hash)` pair appears in the claimed node's Ed25519-VERIFIED signed
/// directory catalog (the PULL substrate): a spoofer cannot put rows into an
/// anchor's signed catalog, while a subscribed node's real apps are listed
/// there by construction (publish → directory revision > 0 → boot
/// re-announce). A subscribed node that never published a directory has its
/// pushed `direct` entries land in the discovery section instead — honest,
/// and consistent with the `/nodes` "waiting for first announcement" row.
///
/// Only DECISIVE for `direct` entries: the shell already classes `curator` /
/// `nodedirectory` rows by `source` (both subscription-gated at ingest — a
/// `curator` row's `node_id` is `None`, so the flag reads `false` there
/// without meaning "unsolicited"; a `nodedirectory` row matches its own
/// catalog by construction). Serialize-only, like `is_own` (§P58.2): zero
/// churn on the ~26 `BrowseEntry` construction sites.
#[derive(Serialize)]
struct BrowseEntryView {
    #[serde(flatten)]
    entry: BrowseEntry,
    is_own: bool,
    from_subscribed: bool,
}

/// `node_id_hex → {(project_id, archive_hash)}` of every SUBSCRIBED anchor's
/// Ed25519-verified catalog, all lowercase, empty hashes skipped (a
/// placeholder row proves nothing about a fetchable app). Built from
/// `directory_snapshot()`, which is itself `is_subscribed`-gated.
fn subscribed_catalog_index(
    dirs: &[nexus_core_rs::NodeDirectoryEntry],
) -> std::collections::HashMap<String, std::collections::HashSet<(String, String)>> {
    let mut index: std::collections::HashMap<String, std::collections::HashSet<(String, String)>> =
        std::collections::HashMap::new();
    for dir in dirs {
        let claims = index.entry(hex::encode(dir.directory.node_id)).or_default();
        for app in &dir.directory.catalog {
            if app.archive_hash.is_empty() {
                continue;
            }
            claims.insert((
                app.project_id.to_ascii_lowercase(),
                app.archive_hash.to_ascii_lowercase(),
            ));
        }
    }
    index
}

/// Pure projection from the aggregator rows to the `/browse` payload —
/// extracted so the derived `is_own` / `from_subscribed` flags are pinned by
/// unit tests (own / catalog-backed / spoofed / unknown) without a network
/// boot.
fn browse_views(
    entries: Vec<BrowseEntry>,
    me: &str,
    catalog_index: &std::collections::HashMap<String, std::collections::HashSet<(String, String)>>,
) -> Vec<BrowseEntryView> {
    entries
        .into_iter()
        .map(|entry| {
            let is_own = entry.node_id.as_deref() == Some(me);
            // Catalog-backed check: the claimed node must have a VERIFIED
            // signed catalog listing exactly this (project_id, archive_hash).
            // Everything is normalized lowercase (§P59.3) so hex case can
            // neither dodge nor fake the classification. An entry with no
            // archive_hash has no content address to match — never classed
            // as "from my sources" on a bare claim.
            let from_subscribed = is_own
                || match (entry.node_id.as_deref(), entry.archive_hash.as_deref()) {
                    (Some(node), Some(hash)) => catalog_index
                        .get(&node.to_ascii_lowercase())
                        .map(|claims| {
                            claims.contains(&(
                                entry.project_id.to_ascii_lowercase(),
                                hash.to_ascii_lowercase(),
                            ))
                        })
                        .unwrap_or(false),
                    _ => false,
                };
            BrowseEntryView {
                is_own,
                from_subscribed,
                entry,
            }
        })
        .collect()
}

async fn list_browse(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /browse");
    let entries = state
        .browse_aggregator
        .aggregate(&state.curator_runtime, &state.node)
        .await;
    let catalog_index = subscribed_catalog_index(&state.curator_runtime.directory_snapshot());
    let views = browse_views(entries, state.node_id.as_str(), &catalog_index);
    (
        StatusCode::OK,
        Json(serde_json::json!({ "entries": views })),
    )
}

/// `POST /api/daemon/browse/pull` — broadcast a browse_request
/// via gossip so peers replay their outbox. Returns immediately.
async fn browse_pull(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"requested": false})),
        );
    }
    let _ = state
        .gossip_cmd_tx
        .send(crate::runtime::GossipCmd::RequestBrowse)
        .await;
    (StatusCode::OK, Json(serde_json::json!({"requested": true})))
}

/// Sprint 20 Phase C : wrap an outbound gossip payload in a PoW
/// envelope. Solves (or reuses) a [`PowSolveCache`] entry for the
/// curator topic under the current live
/// [`RelayPowPolicy`], then concatenates `[u32 BE proof_len][proof
/// bytes][payload]` via [`PowEnvelope::encode`].
///
/// Returns an error if the policy clamps the topic's difficulty to
/// zero (misconfigured policy — loud failure rather than silent
/// bypass) or if the Hashcash solve times out (default 30 s, cf.
/// `SOLVE_TIMEOUT` in `pow_gossip`).
pub(crate) fn wrap_payload_with_pow(
    state: &DaemonHttpState,
    payload: &[u8],
) -> Result<Vec<u8>, nexus_core_rs::PowGossipError> {
    // Graceful degradation on a poisoned lock : recover the inner
    // policy rather than propagating a panic through the publish
    // handler. Mirrors the gossip receive loop
    // (`runtime.rs::spawn_gossip_subscribe_task`) so every reader
    // of `DaemonHttpState.pow_policy` survives a poisoning event.
    let policy = match state.pow_policy.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let proof = state.pow_solve_cache.ensure_proof(
        state.curator_gossip_topic,
        state.pow_keypair.as_ref(),
        &policy,
    )?;
    PowEnvelope::encode(&proof, payload)
}

/// `POST /publish` — broadcast a project announcement via gossip
/// and add it to the local browse aggregator. Sprint 11 Phase A.
///
/// Called by the coordinator's `POST /project/publish` endpoint
/// (proxied through `/daemon/publish`) when the project has
/// `visibility=public`. The daemon constructs a
/// [`ProjectAnnouncement`] from the request body + its own
/// `node_id`, broadcasts it on the curator gossip topic, and
/// adds the resulting [`BrowseEntry`] to the aggregator so it
/// appears in the local `/browse` immediately.
async fn publish_project(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<PublishRequest>,
) -> Response {
    debug!(project = %req.project_name, "POST /publish");

    // Sprint 20 Phase B : in duress mode, short-circuit BEFORE
    // touching the gossip sender so the fake keypair never
    // signs a ProjectAnnouncement. The response says
    // `published: false` — the handler is authoritative, not
    // the peer observer, so a local UI getting a false-flag
    // response is fine; the wire saw nothing.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (StatusCode::OK, Json(PublishResponse { published: false })).into_response();
    }

    // Sprint 16 audit finding D-1: the kickoff §D4 declares
    // `is_open_source` as "derived by coordinator, never
    // user-settable". The daemon is the gossip writer, so it
    // must refuse to flag a project open-source unless the
    // provenance chain (Sprint 14 deploy-from-repo) is present.
    // Without this check, any local process holding the bearer
    // token could submit `{"is_open_source": true, ...}` and
    // see workers at consent level L2 accept its tasks.
    if req.is_open_source && (req.provenance_hash.is_none() || req.repo_url.is_none()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "is_open_source=true requires both provenance_hash and repo_url \
                        (deploy-from-repo chain). The coordinator's \
                        `POST /project/deploy-from-repo` is the only supported path."
                    .to_string(),
            }),
        )
            .into_response();
    }

    // Per-app identity: blake3(project_name) hex — the same id the feed and
    // deploy already use. NOT node_id: one node hosts many apps, and keying the
    // browse card on node_id collapses them all to a single card (and gives the
    // same app a different id depending on the viewing node).
    let project_id = hex::encode(nexus_core_rs::crypto::blake3_hash(
        req.project_name.as_bytes(),
    ));

    // Remediation #8: route /publish through the single canonical
    // announce → broadcast → persist-to-outbox → index → cache helper in
    // `deploy.rs`, so the publish and deploy-from-repo paths can never diverge
    // (the deploy path used to skip the outbox persist).
    crate::deploy::publish_announcement(
        &state,
        crate::deploy::AnnouncementParams {
            project_id: &project_id,
            project_name: &req.project_name,
            category: &req.category,
            description: &req.description,
            apps: &req.apps,
            archive_hash: req.archive_hash.as_deref(),
            repo_url: req.repo_url.as_deref(),
            provenance_hash: req.provenance_hash.as_deref(),
            is_open_source: req.is_open_source,
        },
    )
    .await;

    (StatusCode::OK, Json(PublishResponse { published: true })).into_response()
}

/// Response for `POST /api/daemon/directory/publish`.
#[derive(Debug, Serialize)]
struct PublishDirectoryResponse {
    /// Hex of the publishing node's Ed25519 pubkey (== the signer).
    node_id: String,
    /// The monotone revision stamped on this directory.
    revision: u64,
    /// Number of apps advertised in the catalog.
    catalog_len: usize,
    /// Hex BLAKE3 hash of the stored signed directory blob.
    archive_hash: String,
}

/// `POST /api/daemon/directory/publish` — Sprint 75 Phase B. Build,
/// sign, blob-store, and gossip-announce THIS node's signed
/// [`nexus_core_rs::NodeDirectoryEntry`]: the catalog of apps it hosts,
/// advertised so fresh peers can PULL them (the discovery pivot — list
/// of nodes → a node's catalogue → download). Loopback-authenticated
/// like every `/api/daemon` route.
///
/// Anti-recentralization guards (kickoff §4): the node advertises only
/// its OWN apps (the browse aggregator's direct entries tagged with our
/// node id — never a peer's), signs with the LOCAL node keypair so
/// provenance stays the author's (verrou 4), and embeds no peer node id
/// anywhere (lock-3). The directory is a read-side projection of what we
/// host, never a write-side "publish to X" selector (verrou 1).
async fn publish_directory(State(state): State<Arc<DaemonHttpState>>) -> Response {
    debug!("POST /api/daemon/directory/publish");
    match build_sign_announce_directory(&state).await {
        Ok(DirectoryPublishOutcome::DuressNoop) => (
            StatusCode::OK,
            Json(serde_json::json!({ "published": false })),
        )
            .into_response(),
        Ok(DirectoryPublishOutcome::Published {
            node_id_hex,
            revision,
            catalog_len,
            archive_hash,
        }) => (
            StatusCode::OK,
            Json(PublishDirectoryResponse {
                node_id: node_id_hex,
                revision,
                catalog_len,
                archive_hash,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse { error: e }),
        )
            .into_response(),
    }
}

/// What [`build_sign_announce_directory`] produced.
pub(crate) enum DirectoryPublishOutcome {
    /// Duress mode: nothing was signed (never sign under the fake keypair).
    DuressNoop,
    /// A signed directory was built, blob-stored and (best-effort)
    /// gossip-announced.
    Published {
        node_id_hex: String,
        revision: u64,
        catalog_len: usize,
        archive_hash: String,
    },
}

/// Core of the directory authoring path, shared by the HTTP route and the
/// headless boot re-announce (Sprint 75 Phase E): build THIS node's signed
/// [`nexus_core_rs::NodeDirectoryEntry`] from the apps it actually holds,
/// blob-store it, and gossip-announce it with a fresh ticket + PoW.
///
/// Every anti-recentralization guard lives HERE so each caller (browser
/// route, scripted loopback call, headless boot driver) inherits them
/// identically: duress no-op BEFORE any signing, own-apps-only +
/// local-blob-held ownership gate (verrou 4), LOCAL node keypair
/// provenance, no peer node id anywhere (lock-3).
pub(crate) async fn build_sign_announce_directory(
    state: &Arc<DaemonHttpState>,
) -> Result<DirectoryPublishOutcome, String> {
    // Duress short-circuit BEFORE signing — never sign a directory under
    // the fake keypair (mirrors publish_project).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return Ok(DirectoryPublishOutcome::DuressNoop);
    }

    // Source the OWN catalog: the apps this node hosts (direct entries tagged
    // with our node id). A node never advertises a peer's apps. `node.node_id()`
    // is the lowercase-hex encoding of the SAME Ed25519 key as
    // `pow_keypair.public_bytes()` below: on a real install both derive from one
    // secret (the daemon keypair IS the iroh secret), so the catalog membership
    // and the signed directory identity are the same key (verrou 4).
    let my_node_id = state.node.node_id();
    let own = state.browse_aggregator.own_entries(&my_node_id);

    // Build the directory signed with the node keypair. directory.node_id == the
    // signing pubkey == the dialable identity a puller dials.
    let node_pubkey = state.pow_keypair.public_bytes();
    let revision = next_directory_revision(state);
    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let mut directory = nexus_core_rs::NodeDirectory::new(node_pubkey, revision);
    for e in &own {
        // Cap the catalog at NODE_DIRECTORY_MAX_ENTRIES so a pathological own-app
        // count cannot drive sign() into its over-cap error and 500 the route
        // (defense-in-depth; the gossip self-node_id guard already keeps a peer
        // from inflating own_entries).
        if directory.catalog.len() >= nexus_core_rs::NODE_DIRECTORY_MAX_ENTRIES {
            break;
        }
        // Only advertise PULLABLE apps with a well-formed content address: skip
        // an entry whose archive_hash is empty or not a valid BLAKE3 hash (exactly
        // 64 lowercase hex). The hash is NOT truncated — truncating a content
        // address yields a different, unfetchable hash; we skip the whole entry.
        let Some(archive_hash) = e
            .archive_hash
            .clone()
            .filter(|h| !h.is_empty() && nexus_core_rs::is_valid_archive_hash(h))
        else {
            continue;
        };
        // Content-addressing ownership guard: only advertise an app whose archive
        // blob this node ACTUALLY HOLDS locally. A gossiped ProjectAnnouncement
        // can forge `BrowseEntry.node_id == our node_id` (the gossip ingest does
        // not cross-check `ann.node_id` against the PoW publisher), so the
        // node_id filter alone is spoofable — a peer could otherwise trick us
        // into signing its app into OUR directory (verrou 4 violation). Requiring
        // local blob presence means a spoofed entry (whose blob we do not hold)
        // can never be signed in: content-addressing is the ownership truth, and
        // we only ever claim to host what we can actually serve.
        let Some(hash_arr) = hex::decode(&archive_hash)
            .ok()
            .and_then(|b| <[u8; 32]>::try_from(b.as_slice()).ok())
        else {
            continue;
        };
        if !matches!(blobs.has(hash_arr).await, Ok(true)) {
            continue;
        }
        // The DISPLAY fields are truncated to their NODE_DIRECTORY_*_MAX on a
        // UTF-8 boundary: the deploy/publish path imposes no length cap, so a
        // single over-long local description must NOT make sign() reject the
        // WHOLE catalog (a self-inflicted availability hole) — the app still
        // appears, just clamped.
        directory.catalog.push(nexus_core_rs::CatalogApp {
            project_id: truncate_on_char_boundary(
                &e.project_id,
                nexus_core_rs::NODE_DIRECTORY_PROJECT_ID_MAX,
            ),
            archive_hash,
            project_name: truncate_on_char_boundary(
                &e.project_name,
                nexus_core_rs::NODE_DIRECTORY_PROJECT_NAME_MAX,
            ),
            category: truncate_on_char_boundary(
                &e.category,
                nexus_core_rs::NODE_DIRECTORY_CATEGORY_MAX,
            ),
            description: truncate_on_char_boundary(
                &e.description,
                nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX,
            ),
        });
    }
    let catalog_len = directory.catalog.len();

    let entry = match nexus_core_rs::NodeDirectoryEntry::sign(directory, state.pow_keypair.as_ref())
    {
        Ok(entry) => entry,
        Err(e) => {
            return Err(format!("failed to sign node directory: {e}"));
        }
    };

    // Blob-store the signed entry JSON so peers can fetch it by ticket.
    let entry_bytes = match serde_json::to_vec(&entry) {
        Ok(b) => b,
        Err(e) => {
            return Err(format!("failed to serialize node directory: {e}"));
        }
    };
    let hash_hex = match blobs.add_bytes(entry_bytes).await {
        Ok(hash) => hex::encode(hash),
        Err(e) => {
            return Err(format!("failed to store node directory blob: {e}"));
        }
    };

    // Gossip-announce: PoW-wrap a NodeDirectoryAnnouncement and broadcast it.
    // Best-effort and LIVE-ONLY (a no-op while isolated): unlike the project
    // announce path this does NOT persist to the outbox — it does not need to.
    // The receive-side ingest arm that consumes a directory announcement is
    // `handle_directory_announcement` → `process_directory_announcement_bytes`
    // (Sprint 75 Phase C), and remote-catalog DURABILITY is handled
    // CONSUMER-side: a subscriber persists a re-fetch locator (`anchors.json`)
    // and re-pulls + re-validates at boot (`CuratorRuntime::repull_directories`).
    // The PRODUCER side re-emits at boot via `reannounce_directory_at_boot`
    // (Sprint 75 Phase E, the headless boot driver): state-driven on the
    // persisted revision counter, it re-builds + re-signs + re-announces this
    // same announcement so a subscribed peer ONLINE AT THIS ANCHOR'S BOOT
    // does not wait for the next manual publish. A subscriber that joins
    // LATER still needs a live overlap (boot-only re-emit, no outbox replay
    // for directory announcements — accepted residual of the Phase C
    // deferral closure).
    if let Ok(ticket) = mint_blob_ticket(state, &hash_hex).await {
        let announcement = nexus_shell_daemon_core::iroh_runtime::NodeDirectoryAnnouncement::new(
            node_pubkey,
            ticket,
        );
        if let Ok(payload) = announcement.to_bytes() {
            if let Ok(envelope) = wrap_payload_with_pow(state, &payload) {
                let sender_guard = state.gossip_sender.read().await;
                if let Some(sender) = sender_guard.as_ref() {
                    if let Err(e) = sender.broadcast(envelope).await {
                        debug!(error = %e, "node directory announce broadcast failed (non-fatal)");
                    }
                }
            }
        }
    }

    debug!(
        revision,
        catalog = catalog_len,
        "published signed node directory"
    );
    Ok(DirectoryPublishOutcome::Published {
        node_id_hex: hex::encode(node_pubkey),
        revision,
        catalog_len,
        archive_hash: hash_hex,
    })
}

/// Sprint 75 Phase E — the PRODUCER side of directory durability (the
/// Phase C deferral): `publish_directory`'s gossip announce is LIVE-only
/// and never persisted to the outbox, so after a reboot a catalogue
/// publisher goes silent — without this, a subscribed peer online at the
/// anchor's boot would wait for the next manual publish to (re)discover
/// the catalogue. The re-emit is boot-only: a subscriber that joins
/// later still needs a live overlap (accepted residual). The
/// consumer-side re-pull (`repull_directories`) covers SUBSCRIBERS, not
/// the producer's own re-emission.
///
/// State-driven gate: only a node that ALREADY published a directory
/// (persisted revision > 0) re-builds, re-signs (revision bump, monotone)
/// and re-announces at boot. A node that never published stays silent —
/// this is not a default-on behaviour, and the re-announce is a gossip
/// EMIT of our own signed catalogue, never a fetch (verrou 5). The
/// rebuilt catalogue reflects the apps actually held at boot, through the
/// same ownership gate as the route.
pub(crate) async fn reannounce_directory_at_boot(state: &Arc<DaemonHttpState>) -> bool {
    if read_directory_revision(state) == 0 {
        return false;
    }
    match build_sign_announce_directory(state).await {
        Ok(DirectoryPublishOutcome::Published {
            revision,
            catalog_len,
            ..
        }) => {
            info!(
                revision,
                catalog = catalog_len,
                "producer directory re-announced at boot"
            );
            true
        }
        Ok(DirectoryPublishOutcome::DuressNoop) => false,
        Err(e) => {
            warn!(error = %e, "producer directory boot re-announce failed");
            false
        }
    }
}

/// On-disk shape of `<sbfb-home>/directory_revision.json`: the monotone
/// counter stamped on this node's published directory. Persisted so a
/// re-publish after a restart bumps past the last value rather than
/// resetting to 1 (which a subscribed peer would reject as a rollback).
#[derive(Debug, Serialize, Deserialize)]
struct DirectoryRevisionFile {
    schema_version: u32,
    revision: u64,
}

/// Truncate `s` to at most `max_bytes` bytes without splitting a UTF-8
/// character (the cut falls back to the nearest lower char boundary). Used to
/// clamp catalog fields to their `NODE_DIRECTORY_*_MAX` before signing, since
/// the deploy/publish producers impose no length cap of their own, and to cap
/// the search `q` param to [`MAX_SEARCH_QUERY_BYTES`] (CARRY-5).
fn truncate_on_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

/// Read the persisted directory revision WITHOUT incrementing it. `0`
/// means this node never published a directory (no persisted counter, or
/// no resolvable home) — the state-driven gate
/// [`reannounce_directory_at_boot`] keys on: a non-producer must stay
/// silent at boot.
pub(crate) fn read_directory_revision(state: &DaemonHttpState) -> u64 {
    let Some(home) = state
        .sbfb_home
        .clone()
        .or_else(nexus_shell_daemon_core::auth::sbfb_home)
    else {
        return 0;
    };
    let path = home.join("directory_revision.json");
    std::fs::read(&path)
        .ok()
        .and_then(|b| serde_json::from_slice::<DirectoryRevisionFile>(&b).ok())
        .map(|f| f.revision)
        .unwrap_or(0)
}

/// Read the persisted directory revision, return `previous + 1`, and persist
/// the new value atomically. The home directory is `state.sbfb_home`,
/// resolved ONCE at daemon boot (explicit test override or
/// [`auth::sbfb_home`] `$SBFB_HOME` / `~/.sbfb`). WITHOUT a resolvable
/// home the counter would reset to 1 on every boot and a subscribed peer
/// would reject each re-publish as a revision rollback — the anti-rollback
/// control the `revision` field exists for would be inert (the shipped
/// systemd unit pins `SBFB_HOME` for exactly this reason). Best-effort on
/// the write side: an IO error skips the persist and still returns the
/// computed revision.
///
/// The read-modify-write is serialized by a process-wide lock so two
/// concurrent calls (the daemon runs on a multi-thread runtime) get
/// strictly-distinct, strictly-increasing revisions rather than both
/// reading the same value and signing two directories at the same revision
/// (which a peer would then reject the second of as a rollback). The
/// publish route and the boot re-announce are the only writers, both
/// in-process through [`build_sign_announce_directory`], so one
/// process-wide lock suffices.
fn next_directory_revision(state: &DaemonHttpState) -> u64 {
    static REVISION_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = REVISION_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let Some(home) = state
        .sbfb_home
        .clone()
        .or_else(nexus_shell_daemon_core::auth::sbfb_home)
    else {
        return 1;
    };
    let path = home.join("directory_revision.json");
    let next = read_directory_revision(state).saturating_add(1);
    if let Ok(body) = serde_json::to_vec_pretty(&DirectoryRevisionFile {
        schema_version: 1,
        revision: next,
    }) {
        let tmp = path.with_extension("json.tmp");
        if std::fs::write(&tmp, &body).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
    next
}

/// `POST /api/daemon/keep-online` — Sprint 74 Phase D — toggle a self-deployed
/// app's LOCAL pin. ON re-tags the archive blob (skip-GC) and lets the boot
/// re-broadcast diffuse it; OFF removes the per-intent tag (GC-eligible — no GC
/// runs today, so "stored but no longer diffused") and gates the re-broadcast
/// on EVERY outbox replay path (NeighborUp, browse_request, periodic republish).
/// Loopback-authenticated like every `/api/daemon` route.
#[derive(Debug, serde::Deserialize)]
struct KeepOnlineRequest {
    project_id: String,
    enabled: bool,
}

async fn set_keep_online(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<KeepOnlineRequest>,
) -> impl IntoResponse {
    debug!(project = %req.project_id, enabled = req.enabled, "POST /api/daemon/keep-online");

    // Sprint 76 Phase B (B1, duress siblings): short-circuit BEFORE any local
    // mutation. A decoy node must perform ZERO keep_online persistence and ZERO
    // blob (un)tag — the duress launcher shares the operator's REAL
    // coordinator.db + blob store, so an un-gated toggle would pin/persist the
    // operator's real app set under the fake keypair, correlating the decoy
    // with the real node. Mirrors `run_boot_seed_driver` + `seed_voluntary`:
    // reply a plausible benign success so an observer cannot tell duress from a
    // normal toggle (the local-mutation half of the P1 wire-emit fix 23a08c9).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "enabled": req.enabled})),
        )
            .into_response();
    }

    // The archive blob to (un)pin comes from the app's own Browse card.
    let archive_hash = state
        .browse_aggregator
        .get_direct_entry(&req.project_id)
        .and_then(|e| e.archive_hash.clone());

    {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Err(e) = db.set_keep_online(&req.project_id, req.enabled, archive_hash.as_deref()) {
            warn!(error = %e, "keep_online DB write failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "keep_online write failed"})),
            )
                .into_response();
        }
    }

    // Tag/untag the archive blob (best-effort — the DB row is the source of
    // truth; a tag hiccup must not fail the toggle).
    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let tag = crate::deploy::keep_online_tag(&req.project_id);
    if req.enabled {
        if let Some(arr) = archive_hash
            .as_deref()
            .and_then(crate::deploy::decode_hash_hex)
        {
            if let Err(e) = blobs.set_tag(&tag, arr).await {
                debug!(error = %e, "keep-online tag set failed (non-fatal)");
            }
        }
    } else if let Err(e) = blobs.delete_tag(&tag).await {
        debug!(error = %e, "keep-online tag delete failed (non-fatal)");
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({"ok": true, "enabled": req.enabled})),
    )
        .into_response()
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
const DIRECTORY_PULL_TIMEOUT_SECS: u64 = 120;

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
fn find_directory_app_by_project(
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
fn directory_pull_providers(
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
        if let Ok(id) = iroh::EndpointId::from_str(hex_id) {
            if !providers.contains(&id) {
                providers.push(id);
            }
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

/// Sprint 75 Phase E (D3): the headless boot seed driver. For every
/// project id the operator EXPLICITLY listed under `[seed]
/// keep_online_projects`, acquire the app's archive — an app this node may
/// have NEVER deployed locally — pin it under the keep-online tag
/// (skip-GC), persist the `keep_online` row, and announce the seed to the
/// feed. This is how an always-on anchor seeds its operator's chosen apps
/// without a UI session. An EMPTY list does zero work and zero network
/// calls (verrou 5: the boot fetch is config-driven explicit, never a
/// shipped default — verrou 3 keeps the compiled default empty).
///
/// Resolution order per project id (most-authoritative content source
/// first): the local direct browse entry (an app this node hosts,
/// restored from its own outbox), then the persisted `keep_online` row's
/// archive hash (M18, the hash source-of-truth across reboots), then the
/// SUBSCRIBED node directories (the "configured app I never had" case).
/// Acquisition picks the FIRST APPLICABLE source ONLY — bytes already
/// held locally (re-pin, no network), else the direct entry's ticket,
/// else the Phase D multi-provider chain (`directory_pull_providers` →
/// `fetch_and_pin_multi`, a bare-hash download — NEVER a ticket re-mint:
/// `mint_ticket_for_hash` is the producer helper and bails on a blob we
/// do not hold). There is NO cross-tier failover: a dead ticket tier is
/// one warn + skip (same shape as PULL-3, deferred to the S76 audit).
///
/// Sequential on purpose (one bounded network budget per app — the Phase
/// C re-pull pattern): a long list cannot fan out unbounded dials at
/// boot, and a fully-dead provider set costs at most one timeout per app.
/// Best-effort, ONE-SHOT: a failed app is logged and skipped, the rest
/// proceed; nothing re-drives until the next daemon restart. Known
/// first-boot dead window: on a FRESH anchor (no persisted `anchors.json`
/// yet) the boot re-pull has nothing to restore, so a configured app that
/// only exists in a not-yet-ingested directory is skipped this boot —
/// the operator remedy is `POST /api/daemon/seed {project_id}` once the
/// directory ingests live, or a daemon restart (re-drive-on-ingest is a
/// tracked S76 carry). Returns the number of apps pinned (newly acquired
/// or re-pinned).
pub(crate) async fn run_boot_seed_driver(
    state: &Arc<DaemonHttpState>,
    configured: &[String],
) -> u64 {
    // Duress short-circuit (mirrors every signing/publishing surface,
    // and the sibling `reannounce_directory_at_boot` via its DuressNoop):
    // a decoy node must perform ZERO seed acquisition, ZERO keep_online
    // mutation and ZERO `SeedAnnounced` emission. The launcher's duress
    // path swaps only the identity — config.toml, coordinator.db and the
    // blob store are the operator's REAL ones — so an un-gated driver
    // would re-pin and announce the real configured app set under the
    // fake keypair, correlating the decoy with the real node.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return 0;
    }
    let mut pinned = 0u64;
    let mut seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for pid in configured {
        if !seen.insert(pid.as_str()) {
            continue;
        }

        // --- Resolve the archive hash (+ the anchor when directory-resolved).
        let direct = state.browse_aggregator.get_direct_entry(pid);
        // Lexical block: the DB guard must provably never cross an await
        // (clippy::await_holding_lock reasons on scopes, not drop()).
        let keep_online_row = {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.get_keep_online(pid).ok().flatten()
        };
        // Trust boundary: the subscribed anchor IS the gate — a directory
        // hit pins whatever hash the FIRST advertising anchor (snapshot
        // sorted by node_id) signed for this project id, with BLAKE3 as
        // the only integrity check (no author-provenance verification at
        // auto-seed time). Multiple subscribed anchors advertising the
        // same project id resolve lexicographic-first; tracked with the
        // Sybil-sampling residual in the S76 audit.
        let dir_hit =
            find_directory_app_by_project(&state.curator_runtime.directory_snapshot(), pid, None);
        let Some(hash_hex) = direct
            .as_ref()
            .and_then(|e| e.archive_hash.clone())
            .or_else(|| keep_online_row.as_ref().and_then(|(_, h)| h.clone()))
            .or_else(|| dir_hit.as_ref().map(|(h, _)| h.clone()))
        else {
            warn!(
                project = %pid,
                "boot seed driver: configured app not resolvable yet (no direct entry, no keep_online hash, not in any subscribed directory) — skipped"
            );
            continue;
        };
        let Some(want_hash) = crate::deploy::decode_hash_hex(&hash_hex) else {
            warn!(project = %pid, hash = %hash_hex, "boot seed driver: malformed archive hash — skipped");
            continue;
        };

        // --- Acquire (or re-pin) the bytes, one bounded budget per app.
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let tag = crate::deploy::keep_online_tag(pid);
        let already_held = matches!(blobs.has(want_hash).await, Ok(true));
        let acquired = if already_held {
            // Re-pin (plan §E.3 #2): the blob survived in the store; make
            // sure the keep-online skip-GC tag does too — idempotent.
            match blobs.set_tag(&tag, want_hash).await {
                Ok(()) => true,
                Err(e) => {
                    warn!(project = %pid, error = %e, "boot seed driver: re-pin set_tag failed");
                    false
                }
            }
        } else if let Some(ticket) = direct.as_ref().and_then(|e| e.archive_ticket.clone()) {
            match tokio::time::timeout(
                std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                blobs.fetch_and_pin(
                    state.node.endpoint(),
                    state.node.memory_lookup(),
                    &ticket,
                    &tag,
                ),
            )
            .await
            {
                Ok(Ok(h)) if h == want_hash => true,
                Ok(Ok(_)) => {
                    // The ticket's content disagrees with the resolved hash:
                    // drop the misplaced pin (mirrors the seed handler).
                    let _ = blobs.delete_tag(&tag).await;
                    warn!(project = %pid, "boot seed driver: ticket content does not match the resolved archive hash — skipped");
                    false
                }
                Ok(Err(e)) => {
                    warn!(project = %pid, error = %e, "boot seed driver: ticket fetch failed");
                    false
                }
                Err(_) => {
                    warn!(project = %pid, "boot seed driver: ticket fetch timed out");
                    false
                }
            }
        } else if let Some((_, anchor_hex)) = dir_hit.as_ref() {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let providers = directory_pull_providers(
                &state.seed_registry,
                &state.node_id,
                anchor_hex,
                pid,
                &hash_hex,
                now,
            );
            if providers.is_empty() {
                warn!(project = %pid, "boot seed driver: no dialable provider for this app — skipped");
                false
            } else {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                    blobs.fetch_and_pin_multi(state.node.endpoint(), want_hash, providers, &tag),
                )
                .await
                {
                    Ok(Ok(h)) if h == want_hash => true,
                    // Defensively unreachable: fetch_and_pin_multi returns
                    // the requested hash by construction (content-addressed
                    // download). Kept as the verrou-4 belt-and-braces guard.
                    Ok(Ok(_)) => {
                        let _ = blobs.delete_tag(&tag).await;
                        warn!(project = %pid, "boot seed driver: fetched content does not match the requested hash — skipped");
                        false
                    }
                    Ok(Err(e)) => {
                        warn!(project = %pid, error = %e, "boot seed driver: multi-provider pull failed");
                        false
                    }
                    Err(_) => {
                        warn!(project = %pid, "boot seed driver: multi-provider pull timed out across all providers");
                        false
                    }
                }
            }
        } else {
            warn!(
                project = %pid,
                "boot seed driver: hash known but no acquisition source (no local bytes, no ticket, no directory anchor) — skipped"
            );
            false
        };
        if !acquired {
            continue;
        }

        // --- Persist + announce.
        let was_already_announced = seed_already_announced(&keep_online_row, &hash_hex);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            if let Err(e) = db.set_keep_online(pid, true, Some(&hash_hex)) {
                warn!(project = %pid, error = %e, "boot seed driver: keep_online persist failed");
            }
        }
        // `reannounce_seeds_at_boot` already re-emitted `SeedAnnounced` for
        // every row that was ALREADY enabled with this hash when the daemon
        // booted — only emit for an app this driver newly acquired/enabled,
        // so a configured app never double-announces in one boot.
        if !was_already_announced {
            if let Some(ref fs) = state.feed_sync_state {
                if let Err(e) = crate::feed_sync::emit_seed_announced(
                    fs,
                    &state.coordinator_db,
                    &state.pow_keypair,
                    pid,
                    &hash_hex,
                )
                .await
                {
                    warn!(project = %pid, error = %e, "boot seed driver: seed announce failed (non-fatal)");
                }
            }
        }
        info!(project = %pid, held_locally = already_held, "boot seed driver: app pinned + kept online");
        pinned += 1;
    }
    pinned
}

/// Pure predicate behind the driver's anti-double-emission guard: was this
/// app ALREADY enabled with this EXACT hash when the daemon booted? If so,
/// `reannounce_seeds_at_boot` (awaited inline before the driver spawns)
/// already re-emitted its `SeedAnnounced` this boot, and the driver must
/// not emit a second one. A row that is disabled, hash-less, or enabled
/// for a DIFFERENT hash was not covered by the boot re-announce for the
/// hash being pinned now — emit.
pub(crate) fn seed_already_announced(row: &Option<(bool, Option<String>)>, hash_hex: &str) -> bool {
    matches!(row, Some((true, Some(h))) if h == hash_hex)
}

/// `GET /api/daemon/nodes` response envelope (Sprint 75 Phase D).
///
/// ENVELOPE, not a bare array (S73-E lesson — the search route pins
/// `{results,total,took_ms}` for the same reason): the Phase-F frontend Zod
/// schema validates `{ nodes: [...] }` and additive fields stay possible.
/// One element per SUBSCRIBED publishing node — the directory store is keyed
/// by node pubkey, so the grouping is structural, never recomputed.
#[derive(Debug, Serialize)]
struct NodesResponse {
    nodes: Vec<NodeSummary>,
    /// UX-ARRIVAL (post-S75): NON-subscribed publishers heard on gossip —
    /// cheap-envelope metadata only (the catalog blob is never fetched for an
    /// unsolicited announce, THREAT_MODEL §15.1), surfaced so the arrival
    /// screen can offer a subscribe CTA. ALWAYS serialized (even empty): the
    /// frontend envelope schema is `.strict()`, so this key must never be
    /// conditional.
    observed: Vec<ObservedNodeView>,
}

/// One observed (non-subscribed) publisher in [`NodesResponse`]. Two fields
/// by design — `revision`/`app_count` live in the signed blob, which is never
/// fetched for a non-subscribed node (preflight UX-ARRIVAL, S4 trace 1): this
/// identity is PoW-backed metadata, not an Ed25519-verified catalog claim.
#[derive(Debug, Serialize)]
struct ObservedNodeView {
    /// Lowercase hex Ed25519 pubkey the announcement named.
    node_id: String,
    /// Unix seconds (LOCAL receive clock) of the last accepted announce.
    last_seen: u64,
}

/// One catalog-publishing node in [`NodesResponse`].
#[derive(Debug, Serialize)]
struct NodeSummary {
    /// Lowercase hex Ed25519 pubkey — the node's dialable identity AND the
    /// signing identity of its directory (they are the same key).
    node_id: String,
    /// The directory's monotonic revision (anti-rollback floor).
    revision: u64,
    /// Convenience count of catalog rows.
    app_count: usize,
    /// The advertised apps, verbatim from the verified signed directory.
    /// The anchor is a DISCOVERY source, never an authority: provenance is
    /// derived from the author-signed provenance.json at pull time (verrou 4).
    catalog: Vec<nexus_core_rs::CatalogApp>,
}

/// Pure projection from the verified directory snapshot to the `/nodes`
/// response — extracted so the envelope shape is pinned by a unit test
/// without a network boot.
fn nodes_response(
    dirs: Vec<nexus_core_rs::NodeDirectoryEntry>,
    observed: Vec<([u8; 32], u64)>,
) -> NodesResponse {
    NodesResponse {
        nodes: dirs
            .into_iter()
            .map(|d| NodeSummary {
                node_id: hex::encode(d.directory.node_id),
                revision: d.directory.revision,
                app_count: d.directory.catalog.len(),
                catalog: d.directory.catalog,
            })
            .collect(),
        observed: observed
            .into_iter()
            .map(|(pubkey, last_seen)| ObservedNodeView {
                // `hex::encode` is lowercase by contract (§P59.3 read side).
                node_id: hex::encode(pubkey),
                last_seen,
            })
            .collect(),
    }
}

/// `GET /api/daemon/nodes` — Sprint 75 Phase D — node identity exposure.
///
/// Read-only projection of every SUBSCRIBED node directory (already
/// signature-verified + revision-gated at ingest), grouped by publishing
/// node. This is the additive route chosen over un-skipping
/// `BrowseEntry.node_id`, which would have changed the `/browse` bytes —
/// the preflight S2/S4 trace keeps that surface byte-identical. The full
/// node-Browse front (`/nodes` page) consumes this in Phase F.
async fn list_nodes(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /api/daemon/nodes");
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    (
        StatusCode::OK,
        Json(nodes_response(
            state.curator_runtime.directory_snapshot(),
            state.curator_runtime.observed_snapshot(now),
        )),
    )
        .into_response()
}

// =====================================================================
// Sprint 77 Phase J — read-only shard-session status (control plane)
// =====================================================================

// Sprint 77 Phase L: `ShardSessionView` + `ShardSessionStatusResponse` moved to
// `nexus-core-rs` (`schemas/shard.rs`) so their `schema_for!` can live next to
// the other shard wire schemas — the daemon depends on core, so a core schema
// cannot reference a daemon-private type. The projection + route below consume
// the re-exported types unchanged; the privacy whitelist (THREAT_MODEL §16
// SI-3/SI-4) is the type shape itself — only `session_id` + `member_count` are
// exposed, never a `worker_pubkey`/`initiator`.
use nexus_core_rs::{ShardSessionStatusResponse, ShardSessionView};

/// Read-only, privacy-whitelisted projection of a shard session manifest.
///
/// Exposes ONLY the aggregate `member_count`, never any `worker_pubkey` /
/// `initiator` (the private-group composition, SI-3/SI-4).
fn project_shard_session(manifest: &nexus_core_rs::ShardedSessionManifest) -> ShardSessionView {
    ShardSessionView {
        session_id: manifest.session_id.clone(),
        member_count: manifest.plan.assignments.len(),
    }
}

/// Stub lookup for a live shard session by id.
///
/// The `sbfb/shard/1` protocol primitive exists and the front contract is
/// pinned against an EMPTY store (Sprint 77 Phase J), but there is no
/// live HTTP-readable shard-session store yet, so this always misses and
/// returns `None`. It is the explicit seam where such a store would plug
/// in (a tracked carry-over, not yet built). Any future ingest MUST gate
/// on a `DOMAIN_SHARD_PLAN_V1` signature + `is_member` check BEFORE insert
/// (preflight S3), so the route can never serve an unauthenticated manifest.
fn live_shard_session(_session_id: &str) -> Option<nexus_core_rs::ShardedSessionManifest> {
    None
}

/// Pure projection for `GET /api/daemon/shard-session/{id}` — pinned by a unit
/// test without a network boot. With no live store every id misses and the
/// deterministic empty envelope `{found:false, session:null}` is returned
/// (200, not 404 — `seed_count` precedent: a read-only route answers 200
/// with honest defaults so the parse succeeds).
fn shard_session_response(session_id: &str) -> ShardSessionStatusResponse {
    match live_shard_session(session_id) {
        Some(manifest) => ShardSessionStatusResponse {
            found: true,
            session: Some(project_shard_session(&manifest)),
        },
        None => ShardSessionStatusResponse {
            found: false,
            session: None,
        },
    }
}

/// `GET /api/daemon/shard-session/{id}` — Sprint 77 Phase J — read-only status
/// of a private compute-group shard session.
///
/// Control-plane only: an AGGREGATE status (member count), NEVER the group's
/// member identities (SI-3/SI-4). The richer status (pipeline status, attained
/// verification level) would be added with a live data plane (a tracked S78
/// carry). Loopback-authenticated (lives in `authed_routes`). There is no live
/// session registry yet, so the route deterministically returns
/// `{found:false, session:null}` for every id; the front renders the "no active
/// session" empty state from that.
async fn shard_session(Path(session_id): Path<String>) -> impl IntoResponse {
    debug!("GET /api/daemon/shard-session");
    (StatusCode::OK, Json(shard_session_response(&session_id))).into_response()
}

/// `POST /api/daemon/seed` — Sprint 74 Phase E — VOLUNTARY community seed.
/// This node helps keep a DISTANT public app online: it fetches the app's
/// archive blob, pins it under the keep-online tag (skip-GC), and records a
/// local `keep_online` row so the boot re-announce (Phase F) re-diffuses it.
/// No `SeedRequest`, no invite, no author approval — the content is already
/// public and content-addressed (BLAKE3), so a supporter can only ever hold
/// the author's exact bytes and never re-signs any provenance (the author
/// stays the author). Loopback-authenticated.
///
/// Two acquisition paths (Sprint 75 Phase D closed GAP R5b):
///  - a DIRECT (gossip) entry carries an archive ticket → single-provider
///    `fetch_and_pin` via the ticket, the original Phase-E path;
///  - a DIRECTORY-ONLY app (discovered through a subscribed node directory)
///    has NO ticket, only `(anchor node_id, archive_hash)` → multi-provider
///    `fetch_and_pin_multi` from the anchor + the best-effort seeders.
#[derive(Debug, serde::Deserialize)]
struct SeedVoluntaryRequest {
    project_id: String,
    /// Sprint 75 Phase F (review-D deferral): optional version discriminator.
    /// When present, the seed targets the EXACT archive version the user was
    /// shown — a direct entry carrying a DIFFERENT version no longer shadows
    /// the requested one, and the directory first-match is narrowed to this
    /// hash (multi-anchor collision). `#[serde(default)]` = runtime tolerance:
    /// a body omitting it keeps the pre-F version-agnostic behaviour.
    #[serde(default)]
    archive_hash: Option<String>,
}

/// How `seed_voluntary` acquires the archive bytes for the requested app.
enum SeedFetchPlan {
    /// Direct entry: dial the single provider embedded in the BlobTicket.
    Ticket(String),
    /// Directory-only app: ordered multi-provider fetch by bare hash (Q5).
    Multi(Vec<iroh::EndpointId>),
}

async fn seed_voluntary(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedVoluntaryRequest>,
) -> impl IntoResponse {
    debug!(project = %req.project_id, "POST /api/daemon/seed (voluntary)");

    // Sprint 76 Phase B (B1, duress siblings): short-circuit BEFORE any fetch,
    // pin, keep_online persist, or SeedAnnounced emit. A decoy node must perform
    // ZERO voluntary-seed work — the duress launcher shares the operator's REAL
    // blob store + coordinator.db, so an un-gated seed would pin the operator's
    // app set AND emit a SeedAnnounced under the fake keypair (the local-mutation
    // sibling of the P1 wire-emit fix 23a08c9; this single early-return covers
    // BOTH the local pin and the emit). Reply a plausible benign success.
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({"ok": true, "seeding": req.project_id})),
        )
            .into_response();
    }

    // The app must be visible in Browse so we know its archive hash. A user
    // can only seed what they can see. A direct (gossip) entry wins WHEN it
    // can serve the request — it carries a ready ticket; otherwise fall back
    // to the subscribed node directories (directory-only apps have no ticket
    // by design: a stored ticket would freeze a stale address, the Phase A
    // bug). Sprint 75 Phase F (review-D deferral): a direct entry is skipped
    // when it has NO archive (a ticket-less card must not shadow a pullable
    // directory listing) or when the caller pinned a SPECIFIC version the
    // direct entry does not carry.
    // Reads normalize like writes (hex-case lesson, Phase D SeedRegistry): a
    // mixed-case hash from a raw client must match the lowercase hashes the
    // daemon mints everywhere, never miss on case alone.
    let requested_hash = req.archive_hash.as_deref().map(str::to_ascii_lowercase);
    let requested_hash = requested_hash.as_deref();
    let direct_entry = state.browse_aggregator.get_direct_entry(&req.project_id);
    let had_direct_entry = direct_entry.is_some();
    // The direct card's DISPLAYED hash even when it carries no ticket: the
    // agnostic fallback below must not silently pin a DIFFERENT version than
    // the one the direct card shows the user (review F P3 — pre-F this shape
    // was a 400, never a divergent pin).
    let direct_hash_no_ticket = direct_entry.as_ref().and_then(|e| {
        if e.archive_ticket.is_none() {
            e.archive_hash.clone()
        } else {
            None
        }
    });
    let direct_plan =
        direct_entry.and_then(|entry| match (entry.archive_ticket, entry.archive_hash) {
            (Some(ticket), Some(hash_hex)) => match requested_hash {
                Some(want) if want != hash_hex => None,
                _ => Some((hash_hex, SeedFetchPlan::Ticket(ticket))),
            },
            _ => None,
        });

    // Sprint 76 Phase B (B3, PULL-3): resolve the directory tier UP FRONT, even
    // when a direct ticket exists, so a dead ticket can fall through to it
    // instead of returning a terminal BAD_GATEWAY (audit S75 Track E: the
    // iroh-blobs downloader's intra-vector failover never covers a single dead
    // ticket, which is not a provider SEQUENCE). The fallback targets the SAME
    // content the direct tier would have served (cross-tier = same bytes,
    // different source); for a ticket-less app it targets the requested/displayed
    // version, exactly as before.
    let directory_constraint = direct_plan
        .as_ref()
        .map(|(h, _)| h.as_str())
        .or(requested_hash)
        .or(direct_hash_no_ticket.as_deref());
    let directory_hit = find_directory_app_by_project(
        &state.curator_runtime.directory_snapshot(),
        &req.project_id,
        directory_constraint,
    );
    let mut directory_hit_without_provider = false;
    let directory_plan: Option<(String, SeedFetchPlan)> =
        if let Some((hash_hex, anchor_hex)) = directory_hit {
            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            let providers = directory_pull_providers(
                &state.seed_registry,
                &state.node_id,
                &anchor_hex,
                &req.project_id,
                &hash_hex,
                now,
            );
            if providers.is_empty() {
                directory_hit_without_provider = true;
                None
            } else {
                Some((hash_hex, SeedFetchPlan::Multi(providers)))
            }
        } else {
            None
        };

    let chain = build_seed_fetch_chain(direct_plan, directory_plan);
    if chain.is_empty() {
        // No tier resolved — preserve the precise pre-B3 error disambiguation.
        if directory_hit_without_provider {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": "no dialable provider for this app"})),
            )
                .into_response();
        } else if had_direct_entry && requested_hash.is_none() {
            // A direct card with nothing to pull (no archive) and no directory
            // fallback is a 400, not an unknown app (pre-F behaviour preserved).
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "app has no archive to seed"})),
            )
                .into_response();
        } else if requested_hash.is_some() {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "no source for the requested app version"})),
            )
                .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "unknown app (not in browse)"})),
            )
                .into_response();
        }
    }

    let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
    let tag = crate::deploy::keep_online_tag(&req.project_id);
    // Try each tier in order; the first that returns the wanted bytes wins. A
    // dead tier-1 ticket falls through to the tier-2 directory multi-provider.
    let mut last_error: (StatusCode, &'static str) =
        (StatusCode::BAD_GATEWAY, "could not fetch the app archive");
    for (hash_hex, plan) in chain {
        let Some(want_hash) = crate::deploy::decode_hash_hex(&hash_hex) else {
            last_error = (StatusCode::BAD_REQUEST, "app has a malformed archive hash");
            continue;
        };
        let fetched = match plan {
            SeedFetchPlan::Ticket(ticket) => {
                blobs
                    .fetch_and_pin(
                        state.node.endpoint(),
                        state.node.memory_lookup(),
                        &ticket,
                        &tag,
                    )
                    .await
            }
            SeedFetchPlan::Multi(providers) => {
                match tokio::time::timeout(
                    std::time::Duration::from_secs(DIRECTORY_PULL_TIMEOUT_SECS),
                    blobs.fetch_and_pin_multi(state.node.endpoint(), want_hash, providers, &tag),
                )
                .await
                {
                    Ok(r) => r,
                    Err(_) => Err(nexus_core_rs::NexusError::Blobs(
                        "directory pull timed out across all providers".into(),
                    )),
                }
            }
        };
        match fetched {
            Ok(h) if h == want_hash => {
                {
                    let db = state
                        .coordinator_db
                        .lock()
                        .unwrap_or_else(|p| p.into_inner());
                    if let Err(e) = db.set_keep_online(&req.project_id, true, Some(&hash_hex)) {
                        warn!(error = %e, "voluntary seed: keep_online persist failed");
                    }
                }
                // Sprint 74 Phase F: announce to the feed that this node now seeds
                // the distant app, so the author + other peers see "Toi + N pairs"
                // rise. The lock is taken+dropped inside the helper (never across
                // the await). Best-effort: a feed hiccup must not undo the pin.
                if let Some(ref fs) = state.feed_sync_state {
                    if let Err(e) = crate::feed_sync::emit_seed_announced(
                        fs,
                        &state.coordinator_db,
                        &state.pow_keypair,
                        &req.project_id,
                        &hash_hex,
                    )
                    .await
                    {
                        warn!(error = %e, "voluntary seed: SeedAnnounced emit failed (non-fatal)");
                    }
                }
                return (
                    StatusCode::OK,
                    Json(serde_json::json!({"ok": true, "seeding": req.project_id})),
                )
                    .into_response();
            }
            Ok(_) => {
                // Content hash disagreed with the declared hash — unpin and try
                // the next tier (a mismatched tier never wins).
                let _ = blobs.delete_tag(&tag).await;
                last_error = (StatusCode::BAD_GATEWAY, "fetched content hash mismatch");
            }
            Err(e) => {
                debug!(error = %e, "voluntary seed: tier fetch failed (trying next tier if any)");
                last_error = (StatusCode::BAD_GATEWAY, "could not fetch the app archive");
            }
        }
    }
    // Every tier failed (dead ticket AND no live directory provider).
    let (code, msg) = last_error;
    (code, Json(serde_json::json!({"error": msg}))).into_response()
}

/// Build the ordered cross-tier fetch chain for a voluntary seed (Sprint 76
/// Phase B, B3 PULL-3). Tier 1 is the direct entry's embedded ticket (a ready
/// provider address); tier 2 is the subscribed node directories' multi-provider
/// fetch by bare hash. A dead tier-1 ticket falls THROUGH to tier 2 instead of a
/// terminal BAD_GATEWAY — the cross-tier failover audit S75 Track E (PULL-3)
/// flagged as missing (a single ticket is not a provider SEQUENCE, so the
/// iroh-blobs downloader's intra-vector retry never covers it). Order is
/// load-bearing: the ticket is the cheapest single dial, the directory is the
/// resilient fallback. Pure + total so the chain shape is unit-testable without
/// a network.
fn build_seed_fetch_chain(
    direct_plan: Option<(String, SeedFetchPlan)>,
    directory_plan: Option<(String, SeedFetchPlan)>,
) -> Vec<(String, SeedFetchPlan)> {
    let mut chain = Vec::with_capacity(2);
    if let Some(p) = direct_plan {
        chain.push(p);
    }
    if let Some(p) = directory_plan {
        chain.push(p);
    }
    chain
}

/// Query string for [`seed_count`] (Sprint 75 Phase C, WIRE-2).
#[derive(Debug, serde::Deserialize)]
struct SeedCountQuery {
    /// Optional EXACT archive version to count. When present, `peer_count` is
    /// the seeders of that specific BLAKE3 hash (the honest "peers that can serve
    /// the bytes I am about to pull" answer) and `self_seeding` is true only if
    /// this node's own pin IS that version. When absent, the count is STRICTLY
    /// version-agnostic — the distinct seeders across all versions, the exact
    /// pre-WIRE-2 semantics (no silent substitution of this node's own pinned
    /// hash). Backward compatible: an old caller that omits it keeps the
    /// previous behaviour.
    #[serde(default)]
    archive_hash: Option<String>,
}

/// `GET /api/daemon/seed-count/{project_id}` — Sprint 74 Phase F — the
/// best-effort multi-seed availability count for an app.
///
/// Returns `{ peer_count, self_seeding, self_pin_enabled }`:
///  - `peer_count`: distinct REMOTE seeders seen within the TTL (from the
///    in-memory `SeedRegistry`, fed by ingested `SeedAnnounced` feed ops).
///  - `self_seeding`: whether THIS node actively keeps the app online (an
///    `enabled = 1` keep_online row). The front renders the pair as "Toi + N
///    pairs (vus recemment)" — `self_seeding` is the "Toi", `peer_count` the N.
///  - `self_pin_enabled` (Sprint 75 Phase F, WEB-1): the operator's PERSISTED
///    keep-online intent, three-valued — `null` = never toggled (no
///    keep_online row; the app still rebroadcasts by default, only an
///    explicit OFF row gates the outbox replay), `true`/`false` = the
///    explicit toggle state. Distinct from `self_seeding`, which is
///    version-scoped serving truth: the shell's "Garder en ligne" toggle
///    must reflect INTENT, and a fresh never-toggled own app must not render
///    OFF (it is still diffused via the outbox replay).
///
/// Best-effort by design (scope cut #11): content-addressing (BLAKE3) is the
/// truth of reachability, this count is only a freshness hint. A dedicated
/// route (vs a `seed_count` field on every BrowseEntry) keeps the count fetched
/// live with its TTL semantics and avoids churning every BrowseEntry site.
///
/// Sprint 75 Phase C (WIRE-2): an optional `?archive_hash=` scopes `peer_count`
/// to the seeders of that exact version (see [`SeedCountQuery`]).
async fn seed_count(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<SeedCountQuery>,
) -> impl IntoResponse {
    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let keep_online_row = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.get_keep_online(&project_id).ok().flatten()
    };
    // WEB-1: the raw persisted intent, BEFORE the row-absent default collapses
    // it — `None` (never toggled) must stay distinguishable from `Some(false)`
    // (explicit OFF) for the shell toggle.
    let self_pin_enabled: Option<bool> = keep_online_row.as_ref().map(|(enabled, _)| *enabled);
    let (keep_online_enabled, own_hash) = keep_online_row.unwrap_or((false, None));
    // Reads normalize like writes (hex-case lesson): without this, a
    // mixed-case query would still COUNT the version's peers (the registry
    // normalizes internally) while denying the "Toi" (the own_hash compare
    // below is byte-exact) — an inconsistent answer from one handler.
    let requested = params.archive_hash.as_deref().map(str::to_ascii_lowercase);
    let requested = requested.as_deref();
    // WIRE-2: `peer_count` is scoped to the EXACT version the caller asks about
    // (`?archive_hash=`), else a version-agnostic distinct count across all
    // versions when omitted. The omitted case is STRICTLY the pre-WIRE-2
    // non-regression semantics (`None`) — we do NOT silently substitute our own
    // pinned hash, which would surprise a caller that asked for an aggregate count
    // (Codex GAP). The shell passes the displayed entry's archive_hash on every
    // surface that knows it, so the version-specific path is the practical one.
    let peer_count = state
        .seed_registry
        .count_recent(&project_id, requested, now);
    // `self_seeding` ("Toi") must be HONEST about the queried version: when the
    // caller asks about a SPECIFIC archive_hash, this node only counts as a
    // self-seeder if its pinned hash IS that exact version. Without this check a
    // node pinning version Y would falsely claim "Toi" for a query about version
    // X (Codex GAP). With no version requested, it reflects the enabled state.
    let self_seeding = keep_online_enabled
        && match requested {
            Some(req) => own_hash.as_deref() == Some(req),
            None => true,
        };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "peer_count": peer_count,
            "self_seeding": self_seeding,
            "self_pin_enabled": self_pin_enabled,
        })),
    )
        .into_response()
}

/// `POST /api/daemon/seed/invite` — Sprint 74 Phase E — mint a revocable seed
/// invite token (Tailscale model). The token authorizes a trusted peer to ask
/// THIS node, over the `sbfb/seed/0` protocol, to seed the given app. The invite
/// is bound to the app's CURRENT archive hash (derived from this node's own
/// browse view — "you can only authorize what you can see"), so an invited peer
/// cannot redeem it to make this node pin foreign content (review P2). Returns
/// the opaque token; the row stays local (only the token id ever travels).
#[derive(Debug, serde::Deserialize)]
struct SeedInviteMintRequest {
    project_id: String,
    /// Lifetime in seconds; defaults to 30 days (Tailscale default).
    expires_in_secs: Option<u64>,
    /// Optional cap on redemptions; `None` = reusable until expiry/revoke.
    max_uses: Option<i64>,
}

async fn seed_invite_mint(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedInviteMintRequest>,
) -> impl IntoResponse {
    // Bind the invite to the exact content this node currently sees for the app
    // (the operator can only authorize what is in their own browse view), not to
    // an attacker-chosen hash (review P2).
    let archive_hash = state
        .browse_aggregator
        .get_direct_entry(&req.project_id)
        .and_then(|e| e.archive_hash.clone());
    let Some(archive_hash) = archive_hash else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "app not visible (or has no archive) to authorize"})),
        )
            .into_response();
    };
    let token = hex::encode(nexus_core_rs::random_nonce());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let ttl = req.expires_in_secs.unwrap_or(30 * 24 * 3600);
    let expires_at = now.saturating_add(ttl) as i64;
    {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if let Err(e) = db.mint_seed_invite(
            &token,
            &req.project_id,
            &archive_hash,
            expires_at,
            req.max_uses,
        ) {
            warn!(error = %e, "seed invite mint failed");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "seed invite mint failed"})),
            )
                .into_response();
        }
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "token": token,
            "expires_at": expires_at,
            "archive_hash": archive_hash,
        })),
    )
        .into_response()
}

/// `POST /api/daemon/seed/invite/revoke` — Sprint 74 Phase E — revoke a seed
/// invite token in real time (the next `SeedRequest` carrying it is refused).
#[derive(Debug, serde::Deserialize)]
struct SeedInviteRevokeRequest {
    token: String,
}

async fn seed_invite_revoke(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedInviteRevokeRequest>,
) -> impl IntoResponse {
    let revoked = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.revoke_seed_invite(&req.token).unwrap_or(false)
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({"revoked": revoked})),
    )
        .into_response()
}

/// `GET /api/daemon/seed/invites/{project_id}` — Sprint 74 Phase E — list the
/// seed invites minted for an app, for the local management UI.
async fn seed_invite_list(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let rows = {
        let db = state
            .coordinator_db
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        db.list_seed_invites(&project_id).unwrap_or_default()
    };
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "token": r.token,
                "project_id": r.project_id,
                "archive_hash": r.archive_hash,
                "expires_at": r.expires_at,
                "max_uses": r.max_uses,
                "uses_count": r.uses_count,
                "revoked_at": r.revoked_at,
                "created_at": r.created_at,
            })
        })
        .collect();
    (StatusCode::OK, Json(serde_json::json!({"invites": items}))).into_response()
}

/// Wall-clock budget for one outbound `sbfb/seed/0` request (dial +
/// request + the seeder's own fetch of our archive + signed response).
/// The seeder side fetches the app archive BEFORE replying, so this is
/// aligned on [`DIRECTORY_PULL_TIMEOUT_SECS`] — the budget the codebase
/// already grants the equivalent transfer. NOTE for callers: a
/// 504 from the route does NOT prove the seed failed — the seeder may
/// still complete its fetch + pin after our deadline (and a single-use
/// invite is consumed BEFORE the fetch), so verify via the per-app
/// seed-count rather than blind-retrying a fresh invite.
const SEED_REQUEST_TIMEOUT_SECS: u64 = DIRECTORY_PULL_TIMEOUT_SECS;

/// `POST /api/daemon/seed/request` — Sprint 75 Phase E — the REQUESTER leg
/// of the authenticated `sbfb/seed/0` protocol (S74 Phase E), and the
/// first production caller of [`crate::seed_protocol::request_seed`].
///
/// "Ask a DESIGNATED peer (typically my always-on VPS anchor) to fetch,
/// pin and keep online an app whose archive THIS node holds." Loopback-
/// authenticated and fully scriptable — the headless operational model:
/// after a deploy, a script (or the future peer-designation UI) posts
/// here to hand the app to the anchor, no browser required.
///
/// Roles (do not conflate, preflight delta #4): this is the AUTHOR-side
/// REQUESTER — the voluntary community-seed path (`POST /api/daemon/seed`)
/// is the SEEDER-side unilateral act and never uses `SeedRequest`. The
/// designated peer enforces its own gates (Ed25519 + dialer cross-check +
/// nonce + ts window + the M19 invite ledger bound to
/// `(project_id, archive_hash)`); an `invite_token` minted BY THE PEER is
/// ALWAYS required — the S74 handler rejects an empty token
/// unconditionally (`"no-invite"`), there is no same-key exemption in the
/// wire protocol.
///
/// Anti-recentralization: the peer is the operator's EXPLICIT choice per
/// request — no default peer exists anywhere (verrou 3), and the archive
/// ticket is minted fresh from `my_endpoint_addr()` at request time
/// (Phase A: never a stored snapshot). The seeder ends up with the
/// author's exact BLAKE3 bytes and re-signs no provenance (verrou 4).
#[derive(Debug, serde::Deserialize)]
struct SeedRequestPeerRequest {
    /// Hex Ed25519 endpoint id of the designated seeder peer.
    peer_node_id: String,
    project_id: String,
    /// Invite token minted by the PEER for `(project_id, archive_hash)`.
    /// ALWAYS required by the seeder's M19 handler (an empty token is
    /// rejected `"no-invite"`); the `#[serde(default)]` is runtime
    /// tolerance only — an omitted field deserializes to empty instead of
    /// a 422, then fails the peer's gate with a clear reason.
    #[serde(default)]
    invite_token: String,
}

async fn seed_request_peer(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<SeedRequestPeerRequest>,
) -> Response {
    debug!(project = %req.project_id, peer = %req.peer_node_id, "POST /api/daemon/seed/request");

    // Duress short-circuit BEFORE signing — never sign a SeedRequest under
    // the fake keypair (mirrors publish_project / publish_directory).
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "requested": false })),
        )
            .into_response();
    }

    use std::str::FromStr as _;
    let Ok(peer_id) = iroh::EndpointId::from_str(&req.peer_node_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "malformed peer_node_id (expected an iroh endpoint id)"})),
        )
            .into_response();
    };
    // Compare PARSED identities, not raw strings: `from_str` also accepts
    // the base32 rendering of an endpoint id, which a raw string compare
    // against our hex-lowercase node_id would let through.
    if peer_id.to_string() == state.node_id {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "cannot designate this node as its own seeder"})),
        )
            .into_response();
    }

    // The app must be a local direct entry with a known archive: the
    // requester PROPOSES a source, so it must actually hold the bytes.
    let Some(entry) = state.browse_aggregator.get_direct_entry(&req.project_id) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "unknown app (not in browse)"})),
        )
            .into_response();
    };
    let Some(hash_hex) = entry.archive_hash else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "app has no archive to seed"})),
        )
            .into_response();
    };
    // Fresh ticket from my_endpoint_addr() at request time. The producer
    // helper also enforces local blob presence — a node can never ask a
    // peer to seed bytes it does not itself hold.
    let ticket = match mint_blob_ticket(&state, &hash_hex).await {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({
                    "error": format!("archive blob not mintable locally: {e}")
                })),
            )
                .into_response();
        }
    };

    let now = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let request = nexus_core_rs::seed::SeedRequest {
        version: nexus_core_rs::seed::SEED_FORMAT_VERSION,
        project_id: req.project_id.clone(),
        archive_hash: hash_hex.clone(),
        archive_ticket: ticket,
        requester_node_id: state.pow_keypair.public_bytes(),
        nonce: nexus_core_rs::seed::random_nonce(),
        ts: now,
        invite_token: req.invite_token.clone(),
    };
    let sent_nonce = request.nonce.clone();
    // The daemon signs with its node keypair — the SAME Ed25519 secret the
    // iroh endpoint boots with (runtime.rs), so the seeder's
    // `author_pubkey == conn.remote_id()` dialer cross-check holds.
    let envelope =
        match nexus_core_rs::seed::SeedRequestEnvelope::sign(request, state.pow_keypair.as_ref()) {
            Ok(env) => env,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": format!("failed to sign seed request: {e}")})),
                )
                    .into_response();
            }
        };

    // A bare EndpointId is dialable: pkarr (presets::N0) resolves it in
    // production; tests pre-seed the node's MemoryLookup (which merges,
    // never overwrites, so the empty-addr add inside request_seed is
    // harmless).
    let peer_addr = iroh::EndpointAddr::from(peer_id);
    let resp = match tokio::time::timeout(
        std::time::Duration::from_secs(SEED_REQUEST_TIMEOUT_SECS),
        crate::seed_protocol::request_seed(
            state.node.endpoint(),
            state.node.memory_lookup(),
            peer_addr,
            &envelope,
        ),
    )
    .await
    {
        Ok(Ok(r)) => r,
        Ok(Err(e)) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(serde_json::json!({"error": format!("seed request failed: {e}")})),
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::GATEWAY_TIMEOUT,
                Json(serde_json::json!({"error": "seed request timed out"})),
            )
                .into_response();
        }
    };
    // Correlation defence-in-depth on top of request_seed's signature +
    // dialed-peer checks: the signed response must echo OUR nonce, so a
    // (signed) response to some other request cannot be confused in.
    if resp.response.nonce != sent_nonce {
        return (
            StatusCode::BAD_GATEWAY,
            Json(serde_json::json!({"error": "seed response does not echo the request nonce"})),
        )
            .into_response();
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "accepted": resp.response.decision == nexus_core_rs::seed::SeedDecision::Accepted,
            "reason": resp.response.reason,
            "seeder_node_id": hex::encode(resp.author_pubkey),
        })),
    )
        .into_response()
}

/// Index a browse entry into the FTS5 search corpus so the app is findable by
/// name/category/description from the search bar. Best-effort: a search-index
/// write must never fail a deploy/publish (the durable browse aggregator entry
/// already succeeded). Mirrors the best-effort feed-ingest indexing in
/// `feed_sync`. Shared by the deploy and self-publish paths; the gossip-announce
/// path indexes via this helper too once it carries a per-app project_id.
/// Whether a browse entry's `is_open_source=true` claim is backed by a provable
/// provenance chain (a signed provenance hash AND a source repo URL). A gossiped
/// [`publish::ProjectAnnouncement`] from an untrusted peer can SET
/// `is_open_source=true` with neither field; this single predicate is what every
/// ingress chokepoint uses to downgrade such a claim to `false` so a forged badge
/// never reaches the search index NOR the `/browse` payload. This is DECLARATIVE
/// trust (provenance present), NOT a cryptographic attestation that the archive
/// was actually built from that repo — front "verrou 4" reads `source=="direct"`
/// plus this flag, and the THREAT_MODEL §15.1 row documents that distinction.
pub(crate) fn trustworthy_open_source(
    is_open_source: bool,
    provenance_hash: Option<&str>,
    repo_url: Option<&str>,
) -> bool {
    is_open_source && provenance_hash.is_some() && repo_url.is_some()
}

pub(crate) fn index_browse_entry(
    db: &nexus_coordinator_rs::db::CoordinatorDb,
    entry: &BrowseEntry,
) {
    // Sprint 74 Phase B (B.6): re-apply the `is_open_source` invariant at this
    // shared chokepoint. `index_browse_entry` is the single browse-index path
    // for ALL three production callers — deploy-from-repo (`deploy.rs`), local
    // `/publish` (gated at http.rs:934), AND gossip ingest from an untrusted
    // peer (`runtime.rs`). The HTTP gate only guards the local-write path: a
    // byzantine peer can gossip `is_open_source=true` with a null
    // `provenance_hash`/`repo_url`, and without this the search index would
    // carry the lie — driving the fork consumer and worker L2 consent on a
    // forged source claim (THREAT_MODEL §5.6). Downgrade to `false` here so the
    // index reflects only a genuinely provable open-source chain.
    //
    // Sprint 76 Phase B (B2, CARRY-3): the SAME predicate is now also applied at
    // the `/browse`-aggregator ingress (`runtime::handle_project_announcement`)
    // so the served Browse card — not only the search index — reflects the
    // downgrade. Both chokepoints share `trustworthy_open_source`.
    let trustworthy_open_source = trustworthy_open_source(
        entry.is_open_source,
        entry.provenance_hash.as_deref(),
        entry.repo_url.as_deref(),
    );
    if entry.is_open_source && !trustworthy_open_source {
        tracing::warn!(
            project = %entry.project_id,
            "downgrading is_open_source at browse-index: missing provenance_hash/repo_url"
        );
    }
    let provenance = nexus_coordinator_rs::search::Provenance {
        repo_url: entry.repo_url.as_deref(),
        commit_sha: None,
        archive_hash: entry.archive_hash.as_deref(),
        provenance_hash: entry.provenance_hash.as_deref(),
        is_open_source: trustworthy_open_source,
    };
    if let Err(e) = nexus_coordinator_rs::search::index_entry(
        db,
        &entry.project_id,
        &entry.project_name,
        &entry.category,
        &entry.description,
        "",       // op_type: browse rows are not feed operations
        "browse", // source_type
        &provenance,
    ) {
        tracing::warn!(
            error = %e,
            project = %entry.project_id,
            "failed to index browse entry for search (non-fatal)"
        );
    }
}

/// `GET /default-curators` — return the daemon's configured
/// default curator pubkeys from `[curator]` config section.
/// Sprint 11 Phase B.
async fn default_curators(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /default-curators");
    (
        StatusCode::OK,
        Json(DefaultCuratorsResponse {
            default_curators: state.default_curators.clone(),
        }),
    )
}

/// `POST /publish-blob` — store raw bytes as an iroh blob and
/// return the hex hash. Sprint 12 Phase A.
///
/// Called by the coordinator to upload a zip archive before
/// publishing. The coordinator then passes the hash to
/// `POST /publish` as `archive_hash`.
async fn publish_blob(State(state): State<Arc<DaemonHttpState>>, body: Bytes) -> impl IntoResponse {
    debug!(size = body.len(), "POST /publish-blob");
    // Sprint 20 Phase B : in duress mode, reject task / blob
    // dispatch with a generic 503. Matches the observable
    // surface of any daemon in a maintenance window — no
    // duress-specific signal.
    if crate::noop_identity::task_dispatch_in_duress(state.identity_mode)
        == crate::noop_identity::DispatchOutcome::Reject503
    {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "service in maintenance mode".to_string(),
            }),
        )
            .into_response();
    }
    let blobs = BlobsClient::new(state.node.blobs_store());
    match blobs.add_bytes(body.to_vec()).await {
        Ok(hash) => {
            let hash_hex = hex::encode(hash);
            (StatusCode::OK, Json(PublishBlobResponse { hash: hash_hex })).into_response()
        }
        Err(e) => {
            warn!(error = %e, "failed to store blob");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("blob store failed: {e}"),
                }),
            )
                .into_response()
        }
    }
}

/// `POST /panic/wipe` — Sprint 20 Phase B. Irreversibly destroy
/// the daemon's on-disk state (identity blobs + OS keyring
/// entries + subscriptions.json + blob cache) then exit the
/// process. Triggered by the shell's 5-tap `Ctrl+Shift+Alt+W`
/// gesture. The handler replies 200 BEFORE scheduling the exit
/// so the shell receives confirmation; the actual
/// `process::exit` runs from a spawned tokio task that sleeps
/// 100 ms to let axum flush the response.
async fn panic_wipe(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
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
async fn blob_serve(
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

/// Decode the blob hash (hex) a `BlobTicket` string points to.
///
/// The archive hash never travels on a gossip `ProjectAnnouncement` — only the
/// ticket does. Deriving the hash at ingest lets a discovered app expose its
/// `archive_hash` so the shell knows it HAS an archive and builds the blob-serve
/// URL; blob-serve then resolves the ticket back from the aggregator to download
/// the zip on first open. Returns `None` for a malformed ticket.
pub(crate) fn archive_hash_from_ticket(ticket_str: &str) -> Option<String> {
    let ticket = ticket_str.parse::<iroh_blobs::ticket::BlobTicket>().ok()?;
    let (_addr, hash, _format) = ticket.into_parts();
    Some(hex::encode(hash.as_bytes()))
}

/// Mint a BlobTicket from a hex hash in the local blob store.
pub(crate) async fn mint_blob_ticket(
    state: &DaemonHttpState,
    hash_hex: &str,
) -> Result<String, anyhow::Error> {
    use iroh_blobs::Hash;

    let hash_bytes: [u8; 32] = hex::decode(hash_hex)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("hash must be 32 bytes"))?;

    // Share the mint-from-current-address logic with the replay re-mint path
    // (Sprint 75 Phase A): a ticket's EndpointAddr must always come from
    // my_endpoint_addr() at mint time, never a stored snapshot. The helper also
    // verifies the blob is still held locally.
    crate::runtime::mint_ticket_for_hash(&state.node, Hash::from_bytes(hash_bytes)).await
}

/// `GET /diagnostic/neighborhood` — Sprint 23 Phase E. Returns the
/// node's own ID and the peer pubkeys currently in the daemon's
/// observable neighborhood. iroh 0.98 does not expose a DHT routing
/// table enumeration (`remote_info_iter` landed post-0.98), so the
/// observable neighborhood is the set of subscribed curator pubkeys
/// — the peers this daemon actively tracks via gossip. Post-0.98
/// upgrade or pkarr canary integration (S24) will enrich this with
/// transport-layer peer discovery.
async fn diagnostic_neighborhood(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    debug!("GET /diagnostic/neighborhood");
    let peers = state.curator_runtime.subscribed_pubkeys_hex();
    (
        StatusCode::OK,
        Json(NeighborhoodResponse {
            node_id: state.node_id.clone(),
            peers,
        }),
    )
}

// =================================================================
// FROST DKG + ceremony endpoints (Sprint 30 Phase C)
// =================================================================

#[derive(Debug, Deserialize)]
struct FrostTrustedDealerRequest {
    k: u16,
    n: u16,
}

#[derive(Debug, Serialize)]
struct FrostTrustedDealerResponse {
    shares: Vec<nexus_shell_daemon_core::canary::DkgShareFile>,
    pubkey_package: nexus_shell_daemon_core::canary::DkgPubkeyFile,
}

async fn frost_trusted_dealer(Json(body): Json<FrostTrustedDealerRequest>) -> impl IntoResponse {
    match nexus_shell_daemon_core::canary::generate_dkg(body.k, body.n) {
        Ok((shares, pubkey_package)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostTrustedDealerResponse {
                shares,
                pubkey_package
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostRound1Request {
    participant: u16,
    key_package_hex: String,
}

#[derive(Debug, Serialize)]
struct FrostRound1Response {
    commitment: nexus_shell_daemon_core::canary::CeremonyCommitment,
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
}

async fn frost_round1(Json(body): Json<FrostRound1Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round1(
        body.participant,
        &frost_share.key_package,
    ) {
        Ok((commitment, nonces)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostRound1Response {
                commitment,
                nonces
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostRound2Request {
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    key_package_hex: String,
    participant: u16,
}

async fn frost_round2(Json(body): Json<FrostRound2Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round2(
        &body.nonces,
        &body.signing_package,
        &frost_share.key_package,
    ) {
        Ok(sig_share) => (StatusCode::OK, Json(serde_json::json!(sig_share))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
struct FrostAggregateRequest {
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    shares: Vec<nexus_shell_daemon_core::canary::CeremonySignatureShare>,
    pubkey_package_hex: String,
}

async fn frost_aggregate(Json(body): Json<FrostAggregateRequest>) -> impl IntoResponse {
    let pubkey_file = nexus_shell_daemon_core::canary::DkgPubkeyFile {
        verifying_key_hex: String::new(),
        pubkey_package_hex: body.pubkey_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let pubkey = match nexus_shell_daemon_core::canary::load_pubkey(&pubkey_file) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_aggregate(
        &body.signing_package,
        &body.shares,
        pubkey.package(),
    ) {
        Ok(sig) => (
            StatusCode::OK,
            Json(serde_json::json!({ "signature_hex": hex::encode(sig) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// =================================================================
// Sprint 35 Phase B — Coordinator Rust-native task submission
// =================================================================

async fn coordinator_submit_task(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(submission): axum::Json<nexus_coordinator_rs::types::TaskSubmission>,
) -> impl IntoResponse {
    let input_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
        system_prompt: &submission.system_prompt,
        user_prompt: &submission.prompt,
        model_output: "",
    };
    let input_check = nexus_coordinator_rs::guardrails::default_input_chain().run(&input_ctx);
    if !input_check.passed {
        let reason = input_check
            .tripwire
            .unwrap_or_else(|| "input_guardrail_rejected".into());
        tracing::warn!(
            project_id = %submission.project_id,
            %reason,
            "task rejected by input guardrail"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "input_rejected", "reason": reason})),
        )
            .into_response();
    }

    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    let keypair = (*state.pow_keypair).clone();
    match nexus_coordinator_rs::dispatcher::submit_task(&db, &keypair, submission) {
        Ok(entry) => {
            if let Some(ref tx) = state.task_dispatch_tx {
                if let Err(e) = tx.try_send(entry.clone()) {
                    tracing::warn!("dispatch channel full or closed: {e}");
                }
            }
            // Hotfix #5 (maillon A): nudge the on-demand local worker so
            // a node executes its own tasks without a manual
            // `nexus-worker` setup. Fire-and-forget — the cold start
            // (worker boot + doc sync) runs in the background; the
            // submit returns the task id immediately. Idempotent.
            if let Some(doc) = state.project_doc.clone() {
                let lw = std::sync::Arc::clone(&state.local_worker);
                // Sprint 76 Phase A (D1): pass the user's resolved
                // SBFB_HOME so the provisioned worker can adopt the
                // public sharing level the "offer my power" panel wrote.
                let user_home = state.sbfb_home.clone();
                tokio::spawn(async move { lw.ensure_spawned(doc, user_home).await });
            }
            match serde_json::to_value(&entry) {
                Ok(body) => (StatusCode::OK, Json(body)).into_response(),
                Err(e) => {
                    tracing::error!("task entry serialization failed: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "internal"})),
                    )
                        .into_response()
                }
            }
        }
        Err(nexus_coordinator_rs::error::CoordinatorError::Validation(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("task submit failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase B — Coordinator Rust-native result submission
// =================================================================

async fn coordinator_submit_result(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(entry): axum::Json<nexus_core_rs::task::ResultEntry>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::validator::validate_result_pre_guardrail(&db, &entry) {
        Ok((
            nexus_coordinator_rs::validator::ValidationOutcome::Accepted,
            Some(task_record),
            Some(pending),
        )) => {
            // Sprint 73 Phase A (D5): run the output guardrail BEFORE
            // persisting. The pre phase has written no `result_text` yet, so
            // a tripwire here leaves zero retrievable content (nothing for
            // `GET /api/v1/tasks/{id}/result` to serve) and credits no kudos.
            let guardrail_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
                system_prompt: "",
                user_prompt: "",
                model_output: &pending.result_text,
            };
            let gr = nexus_coordinator_rs::guardrails::default_output_chain().run(&guardrail_ctx);
            if !gr.passed {
                let reason = gr.tripwire.unwrap_or_else(|| "guardrail_rejected".into());
                tracing::warn!(
                    task_id = %entry.payload.task_id,
                    %reason,
                    "result rejected by output guardrail — not persisted, no kudos credited"
                );
                // CARRY-2 (S74 audit, Sprint 75 Phase G): a tripwire is
                // terminal — the validated submission is already consumed, so
                // leaving the task Pending/AwaitingQuorum would zombie it
                // forever. Same transition as the gossip `validator_loop`.
                if let Err(e) =
                    nexus_coordinator_rs::validator::reject_result_on_guardrail_trip(&db, &pending)
                {
                    tracing::error!(
                        task_id = %entry.payload.task_id,
                        "failed to mark guardrail-tripped task rejected: {e}"
                    );
                }
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"outcome": "rejected", "reason": "guardrail_rejected"}),
                    ),
                )
                    .into_response();
            }
            if let Err(e) =
                nexus_coordinator_rs::validator::validate_result_post_guardrail(&db, &pending)
            {
                tracing::error!(task_id = %entry.payload.task_id, "result persist failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
            let worker_id = hex::encode(entry.worker_pubkey);
            if let Err(e) = nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                &task_record.project_id,
                &worker_id,
                &entry.payload.task_id,
                entry.payload.tokens_generated,
                entry.payload.generation_time_ms,
            ) {
                tracing::warn!("kudos credit failed (non-fatal): {e}");
            }
            let _ = state
                .result_event_tx
                .send(crate::validator_loop::ResultEvent::NewResult(entry));
            (
                StatusCode::OK,
                Json(serde_json::json!({"outcome": "accepted"})),
            )
                .into_response()
        }
        Ok((nexus_coordinator_rs::validator::ValidationOutcome::AwaitingQuorum, _, _)) => (
            StatusCode::OK,
            Json(serde_json::json!({"outcome": "awaiting_quorum"})),
        )
            .into_response(),
        Ok((outcome, _, _)) => {
            let reason = match outcome {
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedBadSignature => {
                    "bad_signature"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotFound => {
                    "task_not_found"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotPending => {
                    "task_not_pending"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::QuorumRejected => {
                    "quorum_divergence"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::Accepted
                | nexus_coordinator_rs::validator::ValidationOutcome::AwaitingQuorum => {
                    unreachable!()
                }
            };
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"outcome": "rejected", "reason": reason})),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("result validation failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase C — Kudos read endpoint
// =================================================================

async fn coordinator_get_kudos(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match nexus_coordinator_rs::kudos_ledger::get_project_kudos(&db, &project_id, now_secs) {
        Ok(kudos) => match serde_json::to_value(&kudos) {
            Ok(body) => (StatusCode::OK, Json(body)).into_response(),
            Err(e) => {
                tracing::error!("kudos serialization failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response()
            }
        },
        Err(e) => {
            tracing::error!("kudos query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 38 Phase A — verify_chain endpoint
// =================================================================

async fn coordinator_verify_chain(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::kudos_ledger::verify_chain(&db, &project_id) {
        Ok(valid) => (StatusCode::OK, Json(serde_json::json!({"valid": valid}))).into_response(),
        Err(e) => {
            tracing::error!("verify_chain failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 39 Phase C — Canary registry HTTP endpoints
// =================================================================

async fn canary_observed(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let obs = match nexus_coordinator_rs::canary_registry::coerce_canary_payload(&payload) {
        Ok(o) => o,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
                .into_response();
        }
    };
    match state.canary_registry.lock() {
        Ok(mut reg) => {
            reg.observe_canary(obs);
            (StatusCode::OK, Json(serde_json::json!({"status": "ok"}))).into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

async fn canary_network_health(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let health = reg.network_health();
            (
                StatusCode::OK,
                Json(serde_json::to_value(&health).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

async fn canary_freshness(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(pubkey): axum::extract::Path<String>,
) -> impl IntoResponse {
    match state.canary_registry.lock() {
        Ok(reg) => {
            let freshness = reg.freshness(&pubkey);
            (
                StatusCode::OK,
                Json(serde_json::to_value(&freshness).unwrap_or_default()),
            )
                .into_response()
        }
        Err(_poisoned) => {
            tracing::error!("canary registry mutex poisoned");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 63 Phase B — Provenance endpoint
// =================================================================

async fn get_provenance(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match db.get_provenance_by_project(&project_id) {
        Ok(Some(record)) => {
            let record_json = nexus_coordinator_rs::provenance::provenance_to_json(&record);
            let provenance_hash = nexus_coordinator_rs::provenance::provenance_blake3_hex(&record);
            let (status, verified) = match hex::decode(&record.node_id) {
                Ok(bytes) if bytes.len() == 32 => {
                    let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
                    let v = nexus_coordinator_rs::provenance::verify_provenance(
                        &record_json,
                        &pub_bytes,
                    );
                    if v {
                        ("verified", true)
                    } else {
                        ("failed", false)
                    }
                }
                _ => ("failed", false),
            };
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "record": record,
                    "verified": verified,
                    "status": status,
                    "provenance_hash": provenance_hash,
                })),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "absent",
                "verified": false,
                "record": null,
                "provenance_hash": null,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("provenance DB query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 63 Phase C — Feed cursor endpoint
// =================================================================

async fn get_feed_cursor(State(state): State<Arc<DaemonHttpState>>) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match db.load_feed_cursor() {
        Ok(Some((last_seq, last_entry_hash))) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "last_seq": last_seq,
                "last_entry_hash": last_entry_hash,
            })),
        )
            .into_response(),
        Ok(None) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "last_seq": 0,
                "last_entry_hash": null,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("feed cursor query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct FeedEntriesQuery {
    #[serde(default)]
    after_seq: Option<u64>,
    #[serde(default = "default_feed_limit")]
    limit: u64,
    #[serde(default)]
    project_id: Option<String>,
    #[serde(default)]
    op_type: Option<String>,
}

fn default_feed_limit() -> u64 {
    50
}

async fn get_feed_entries(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Query(params): axum::extract::Query<FeedEntriesQuery>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let limit = params.limit.min(100);
    let after_seq = params.after_seq.unwrap_or(0);

    let rows = match db.get_feed_entries_after_seq(after_seq) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("feed entries query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let filtered: Vec<_> = rows
        .into_iter()
        .filter(|row| {
            if let Some(ref pid) = params.project_id {
                let payload: serde_json::Value =
                    serde_json::from_str(&row.payload).unwrap_or_default();
                if payload.get("project_id").and_then(|v| v.as_str()) != Some(pid.as_str()) {
                    return false;
                }
            }
            if let Some(ref ot) = params.op_type {
                if row.op_type != *ot {
                    return false;
                }
            }
            true
        })
        .take(limit as usize)
        .map(|row| {
            serde_json::json!({
                "seq": row.seq,
                "op_type": row.op_type,
                "payload": serde_json::from_str::<serde_json::Value>(&row.payload).unwrap_or_default(),
                "author": row.author,
                "entry_hash": row.entry_hash,
                "prev_hash": row.prev_hash,
                "created_at": row.created_at,
            })
        })
        .collect();

    let count = filtered.len();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "entries": filtered,
            "count": count,
        })),
    )
        .into_response()
}

// =================================================================
// Sprint 67 Phase B: FTS5 search endpoint
// =================================================================

#[derive(Debug, serde::Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_search_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_search_limit() -> usize {
    20
}

/// Server-side caps on the attacker-supplied search params (Sprint 75
/// Phase G, CARRY-5 / S74 audit). `limit` was already clamped to 100; an
/// unbounded `offset` walks the whole FTS5 match set inside SQLite
/// (`LIMIT ?2 OFFSET ?3`) — and `usize::MAX as i64` even flips negative,
/// which SQLite silently treats as "no offset". An unbounded `q` is
/// tokenised + quoted per word before the MATCH parse, so a megabyte
/// query is a cheap CPU/allocation lever on the loopback API.
const MAX_SEARCH_OFFSET: usize = 10_000;
const MAX_SEARCH_QUERY_BYTES: usize = 1024;

async fn search_handler(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let limit = params.limit.min(100);
    let offset = params.offset.min(MAX_SEARCH_OFFSET);
    // UTF-8-safe truncation: a naive byte slice would panic mid-char.
    let q = truncate_on_char_boundary(&params.q, MAX_SEARCH_QUERY_BYTES);
    let start = std::time::Instant::now();

    let (results, total) = match nexus_coordinator_rs::search::search(&db, &q, limit, offset) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("search query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let took_ms = start.elapsed().as_millis() as u64;
    let entries: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "project_id": r.project_id,
                "project_name": r.project_name,
                "category": r.category,
                "description": r.description,
                "op_type": r.op_type,
                "source_type": r.source_type,
                "score": r.score,
                // Provenance triplet (Sprint 73 Phase D): additive keys so a
                // search hit can drive a fork in S74. `null` for non-release
                // ops; never matchable (UNINDEXED). No wire-format bump —
                // search_index is local, FEED_FORMAT_VERSION stays 1.
                "repo_url": r.repo_url,
                "commit_sha": r.commit_sha,
                "archive_hash": r.archive_hash,
                "provenance_hash": r.provenance_hash,
                "is_open_source": r.is_open_source,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "results": entries,
            "total": total,
            "took_ms": took_ms,
        })),
    )
        .into_response()
}

// =================================================================
// Sprint 68 Phase B — Ephemeral preview load endpoint
// =================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewLoadResponse {
    pub hash: String,
}

async fn preview_load(State(state): State<Arc<DaemonHttpState>>, body: Bytes) -> impl IntoResponse {
    debug!(size = body.len(), "POST /api/v1/preview/load");
    match state.preview_store.load(body.to_vec()) {
        Ok(hash) => (StatusCode::OK, Json(PreviewLoadResponse { hash })).into_response(),
        Err(nexus_shell_daemon_core::preview::PreviewError::TooLarge { actual, limit }) => (
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": format!("preview size {actual} exceeds limit {limit}")
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

// =================================================================
// Sprint 68 Phase A — ProofCard evidence score endpoint
// =================================================================

async fn get_proof_card(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    // 1. Look up browse entry (direct entries from project announcements).
    let browse_entry = state.browse_aggregator.get_direct_entry(&project_id);

    // 2. Count distinct curators vouching for this project.
    let curator_snapshot = state.curator_runtime.list_snapshot();
    let mut curator_names: Vec<String> = Vec::new();
    let mut seen_pubkeys = std::collections::HashSet::new();
    for list_entry in &curator_snapshot {
        let curator_hex = hex::encode(list_entry.curator_pubkey);
        for project in &list_entry.list.entries {
            if project.project_id == project_id && seen_pubkeys.insert(curator_hex.clone()) {
                curator_names.push(list_entry.list.curator_name.clone());
            }
        }
    }

    // 3. Extract metadata from browse entry or curator lists.
    let (project_name, is_open_source, archive_hash, provenance_hash, entry_repo_url) =
        match &browse_entry {
            Some(e) => (
                e.project_name.clone(),
                e.is_open_source,
                e.archive_hash.clone(),
                e.provenance_hash.clone(),
                e.repo_url.clone(),
            ),
            None => {
                let name = curator_snapshot
                    .iter()
                    .flat_map(|le| le.list.entries.iter())
                    .find(|p| p.project_id == project_id)
                    .map(|p| p.project_name.clone());
                match name {
                    Some(n) => (n, false, None, None, None),
                    None => {
                        return (
                            StatusCode::NOT_FOUND,
                            Json(serde_json::json!({"error": "project not found"})),
                        )
                            .into_response();
                    }
                }
            }
        };

    // 4. Query provenance from the coordinator DB.
    let provenance_opt = {
        let db = match state.coordinator_db.lock() {
            Ok(guard) => guard,
            Err(_poisoned) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
        };
        match db.get_provenance_by_project(&project_id) {
            Ok(record) => record,
            Err(e) => {
                tracing::error!("proof card DB query failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
        }
    };

    // 5. Verify provenance signature if a record exists.
    let (provenance_verified, repo_url, commit_sha, deploy_timestamp) = match &provenance_opt {
        Some(record) => {
            let record_json = nexus_coordinator_rs::provenance::provenance_to_json(record);
            let verified = match hex::decode(&record.node_id) {
                Ok(bytes) if bytes.len() == 32 => {
                    let pub_bytes: [u8; 32] = bytes.try_into().unwrap();
                    nexus_coordinator_rs::provenance::verify_provenance(&record_json, &pub_bytes)
                }
                _ => false,
            };
            (
                verified,
                Some(record.repo_url.clone()),
                Some(record.commit_sha.clone()),
                Some(record.timestamp.clone()),
            )
        }
        None => (false, None, None, None),
    };

    let effective_repo_url = repo_url.or(entry_repo_url);

    // 6. Compute the ProofCard.
    let input = nexus_coordinator_rs::proof_card::ProofCardInput {
        project_id: project_id.clone(),
        project_name,
        provenance_verified,
        repo_url: effective_repo_url,
        commit_sha,
        is_open_source,
        archive_hash,
        provenance_hash,
        license_spdx: None,
        curator_count: seen_pubkeys.len(),
        curator_names,
        deploy_timestamp_rfc3339: deploy_timestamp,
    };

    let card = nexus_coordinator_rs::proof_card::compute_proof_card(input);
    (StatusCode::OK, Json(card)).into_response()
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{KeyPair, create_node};
    use nexus_shell_daemon_core::blob_serve::BlobServeCache;
    use nexus_shell_daemon_core::browse::BrowseAggregator;
    use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
    use tower::ServiceExt;

    /// Sprint 74 Phase B (B.6): the shared browse-index chokepoint downgrades a
    /// byzantine `is_open_source=true` carrying no provenance/repo to `false`,
    /// so a gossiped lie from an untrusted peer cannot poison the search index
    /// (the HTTP `/publish` gate only covers the local-write path; gossip
    /// ingest bypasses it).
    #[test]
    fn browse_index_rejects_open_source_without_provenance() {
        fn entry(
            project_id: &str,
            name: &str,
            repo: Option<&str>,
            prov: Option<&str>,
        ) -> BrowseEntry {
            BrowseEntry {
                project_id: project_id.into(),
                node_id: None,
                project_name: name.into(),
                category: "tools".into(),
                description: "fork-source candidate".into(),
                curator_pubkey: String::new(),
                curator_name: String::new(),
                source: nexus_shell_daemon_core::browse::BrowseSource::Direct,
                status: nexus_shell_daemon_core::browse::BrowseStatus::Unknown,
                last_probed_at: None,
                archive_ticket: None,
                archive_hash: Some("ab".repeat(32)),
                repo_url: repo.map(String::from),
                provenance_hash: prov.map(String::from),
                is_open_source: true,
            }
        }

        let db = nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("db");

        // Byzantine: claims open-source with NO provenance chain → downgraded.
        index_browse_entry(&db, &entry("byzantine-app", "ByzantineApp", None, None));
        let (hits, _) = nexus_coordinator_rs::search::search(&db, "ByzantineApp", 10, 0).unwrap();
        let hit = hits
            .iter()
            .find(|h| h.project_id == "byzantine-app")
            .expect("byzantine entry indexed");
        assert!(
            !hit.is_open_source,
            "byzantine open-source claim with no provenance must be downgraded to false"
        );

        // Honest: full provenance chain (repo_url + provenance_hash) → preserved.
        index_browse_entry(
            &db,
            &entry(
                "honest-app",
                "HonestApp",
                Some("https://codeberg.org/me/app.git"),
                Some(&"ef".repeat(32)),
            ),
        );
        let (hits, _) = nexus_coordinator_rs::search::search(&db, "HonestApp", 10, 0).unwrap();
        let hit = hits
            .iter()
            .find(|h| h.project_id == "honest-app")
            .expect("honest entry indexed");
        assert!(
            hit.is_open_source,
            "honest open-source with full provenance chain must be preserved"
        );
    }

    /// Sprint 16 Phase A: known-valid bearer token used by every
    /// test via [`build_test_router`]. 64-char lowercase hex,
    /// the shape
    /// [`nexus_shell_daemon_core::auth::load_or_generate_token`]
    /// would produce but fixed so assertions stay deterministic.
    const TEST_TOKEN: &str = "deadbeefcafebabefeedfaceabadc0de0123456789abcdef0123456789abcdef";

    /// Canonicalized `X-SBFB-Token` header name for the test-only
    /// layer below. Kept as a separate const because `HeaderMap`
    /// wants a `HeaderName`, not a `&str`.
    const AUTH_HEADER_NAME: axum::http::HeaderName =
        axum::http::HeaderName::from_static("x-sbfb-token");

    /// Build the production router plus an outer test-only layer
    /// that injects `X-SBFB-Token` and a loopback `Host` on every
    /// inbound request so the tests below can keep their one-liner
    /// `Request::builder().uri(..)` shape without re-attaching
    /// headers by hand 40+ times. Only the outermost layer is
    /// synthetic — every route still runs the real
    /// [`auth_required`] middleware, and the 401 / 403 paths are
    /// covered by `auth::tests` in the core crate.
    fn build_test_router(state: Arc<DaemonHttpState>) -> Router {
        build_test_router_with_cors(state, &[])
    }

    fn build_test_router_with_cors(state: Arc<DaemonHttpState>, cors: &[String]) -> Router {
        use axum::http::HeaderValue;
        use axum::http::header::{HOST, ORIGIN};
        build_router(state, AuthState::new(TEST_TOKEN.to_string()), cors, None).layer(
            middleware::from_fn(
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
            ),
        )
    }

    /// Build a [`DaemonHttpState`] backed by a live iroh node.
    /// Every HTTP test spins up a fresh node because the
    /// browse route reaches through the Arc<Node> to probe
    /// endpoints. The `_node_guard` return keeps the node
    /// alive for the scope of the test; letting it drop
    /// calls the synchronous Drop path which is fine for
    /// unit tests.
    async fn mk_state() -> Arc<DaemonHttpState> {
        mk_state_with_mode(nexus_core_rs::IdentityMode::Normal).await
    }

    async fn mk_state_with_sbfb_home(home: std::path::PathBuf) -> Arc<DaemonHttpState> {
        let mut state = (*mk_state().await).clone();
        state.sbfb_home = Some(home);
        Arc::new(state)
    }

    async fn mk_state_with_mode(mode: nexus_core_rs::IdentityMode) -> Arc<DaemonHttpState> {
        mk_state_with_mode_tx(mode, tokio::sync::mpsc::channel(8).0).await
    }

    // Variant that injects a caller-supplied gossip_cmd_tx so a test can hold
    // the receiver and assert what the announce path pushed to the outbox
    // (remediation #8). The default mk_state drops the rx, which closes the
    // channel — fine for tests that don't assert on it.
    async fn mk_state_with_mode_tx(
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
        })
    }

    fn own_browse_entry(project_id: &str, name: &str, owner: Option<String>) -> BrowseEntry {
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

    #[test]
    fn browse_views_derives_from_subscribed() {
        // UX-ARRIVAL: the derived flag that splits MY sources from the
        // unsolicited ambient is CATALOG-BACKED (review SEC-UXARR-1, P1
        // skeptics-confirmed): a ProjectAnnouncement's node_id is a free
        // unsigned claim, so naming a subscribed anchor must NOT buy the
        // "Tes sources" placement — only an app the claimed node lists in
        // its Ed25519-verified signed catalog qualifies.
        let me = "11".repeat(32);
        let kp_friend = KeyPair::generate();
        let friend = hex::encode(kp_friend.public_bytes());
        let stranger = "33".repeat(32);
        let listed_hash = "ab".repeat(32);

        // The subscribed friend's VERIFIED catalog: lists (friend-app,
        // listed_hash) plus a placeholder row (empty hash) that the index
        // must skip — a placeholder proves nothing fetchable.
        let mut dir = nexus_core_rs::NodeDirectory::new(kp_friend.public_bytes(), 1);
        dir.catalog = vec![
            catalog_app("friend-app", &listed_hash, "FriendApp"),
            catalog_app("placeholder-app", "", "Placeholder"),
        ];
        let entry = nexus_core_rs::NodeDirectoryEntry::sign(dir, &kp_friend).unwrap();
        let index = subscribed_catalog_index(&[entry]);
        assert!(
            !index[&friend].contains(&("placeholder-app".into(), String::new())),
            "an empty-hash placeholder row must not enter the index"
        );

        let with_hash = |pid: &str, name: &str, owner: Option<String>, hash: Option<String>| {
            let mut e = own_browse_entry(pid, name, owner);
            e.archive_hash = hash;
            e
        };
        let views = browse_views(
            vec![
                with_hash("own-app", "OwnApp", Some(me.clone()), None),
                with_hash(
                    "friend-app",
                    "FriendApp",
                    Some(friend.clone()),
                    Some(listed_hash.clone()),
                ),
                // THE spoof: claims the SUBSCRIBED friend's node_id, but the
                // (pid, hash) pair is NOT in the friend's signed catalog.
                with_hash(
                    "spoof-app",
                    "SpoofApp",
                    Some(friend.clone()),
                    Some("cc".repeat(32)),
                ),
                with_hash(
                    "stranger-app",
                    "StrangerApp",
                    Some(stranger),
                    Some(listed_hash.clone()),
                ),
                // Hex-case probe: node_id AND hash uppercased.
                with_hash(
                    "friend-app",
                    "MixedCase",
                    Some(friend.to_ascii_uppercase()),
                    Some(listed_hash.to_ascii_uppercase()),
                ),
                // A bare claim with no content address is never "my sources".
                with_hash("no-hash-app", "NoHash", Some(friend.clone()), None),
                with_hash("curator-app", "CuratorApp", None, None),
            ],
            &me,
            &index,
        );

        let by_name = |name: &str| {
            views
                .iter()
                .find(|v| v.entry.project_name == name)
                .expect("fixture row present")
        };
        let own = by_name("OwnApp");
        assert!(own.is_own, "hosting node_id == me");
        assert!(own.from_subscribed, "own implies from_subscribed");
        let friend_view = by_name("FriendApp");
        assert!(!friend_view.is_own);
        assert!(
            friend_view.from_subscribed,
            "a catalog-listed app of a subscribed node belongs to MY sources"
        );
        assert!(
            !by_name("SpoofApp").from_subscribed,
            "naming a subscribed node_id without a signed catalog row must NOT buy the placement (SEC-UXARR-1)"
        );
        assert!(
            !by_name("StrangerApp").from_subscribed,
            "an unknown announcer is the ambient (unsolicited) class"
        );
        // Hex-case normalization: case can neither fake nor dodge the split.
        assert!(by_name("MixedCase").from_subscribed);
        assert!(
            !by_name("NoHash").from_subscribed,
            "no archive_hash = no content address to verify against the catalog"
        );
        assert!(
            !by_name("CuratorApp").from_subscribed,
            "a None-node_id row reads false (non-decisive: classed by source)"
        );

        // The serialized row carries BOTH derived keys (the Zod entry schema
        // is .strict(): key and schema ship in the same commit).
        let json = serde_json::to_value(friend_view).unwrap();
        assert_eq!(json["from_subscribed"], true);
        assert_eq!(json["is_own"], false);
    }

    /// Sprint 75 Phase B: the authoring route builds a signed directory
    /// from the node's OWN apps, stores it as a verifiable blob, and the
    /// signature provenance is the node keypair (verrou 4). A remote
    /// node's app (different node_id) is excluded from our catalog.
    #[tokio::test]
    async fn publish_directory_route_signs_and_announces() {
        // sbfb_home is an isolated tempdir so the persisted revision counter does
        // not touch (or read a stale value from) the real ~/.sbfb via the
        // auth::sbfb_home fallback.
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let a = "a".repeat(64);
        let b = "b".repeat(64);
        let c = "c".repeat(64);

        // The two OWN apps reference blobs the node actually HOLDS (the ownership
        // truth that blocks gossip-spoofed entries from being signed in).
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let ha = hex::encode(blobs.add_bytes(b"zip-babel".to_vec()).await.unwrap());
        let hb = hex::encode(blobs.add_bytes(b"zip-atlas".to_vec()).await.unwrap());
        let mut ea = own_browse_entry(&a, "Babel", Some(my_id.clone()));
        ea.archive_hash = Some(ha);
        let mut eb = own_browse_entry(&b, "Atlas", Some(my_id.clone()));
        eb.archive_hash = Some(hb);
        state.browse_aggregator.add_direct_entry(ea);
        state.browse_aggregator.add_direct_entry(eb);
        // A remote app discovered via gossip — different hosting node id (excluded
        // by the node_id filter before the blob check).
        state.browse_aggregator.add_direct_entry(own_browse_entry(
            &c,
            "RemoteApp",
            Some("dead".repeat(16)),
        ));

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["revision"], 1);
        assert_eq!(v["catalog_len"], 2);
        assert_eq!(
            v["node_id"].as_str().unwrap(),
            hex::encode(state.pow_keypair.public_bytes())
        );

        // Fetch the stored blob back and prove it is a verifiable signed
        // directory carrying only our OWN apps, sorted by project_id.
        let archive_hash = v["archive_hash"].as_str().unwrap();
        let hash: [u8; 32] = hex::decode(archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        entry
            .verify_signature()
            .expect("published directory must verify");
        assert_eq!(entry.node_id, state.pow_keypair.public_bytes());
        assert_eq!(entry.directory.revision, 1);
        let ids: Vec<&str> = entry
            .directory
            .catalog
            .iter()
            .map(|app| app.project_id.as_str())
            .collect();
        assert_eq!(ids, vec![a.as_str(), b.as_str()]);
        assert!(
            entry
                .directory
                .catalog
                .iter()
                .all(|app| app.project_name != "RemoteApp"),
            "a remote node's app must never appear in our own directory"
        );
    }

    /// Sprint 75 Phase B: in duress mode the route never signs a
    /// directory under the fake keypair — it returns `published: false`
    /// before touching the keypair (mirrors `publish_project`).
    #[tokio::test]
    async fn publish_directory_noop_in_duress() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(resp.status(), axum::http::StatusCode::OK);
        let body = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["published"], false);
    }

    /// Sprint 75 Phase B: the directory revision is a monotone counter
    /// persisted under sbfb_home, so a re-publish after a restart bumps
    /// past the last value rather than resetting to 1 (which a subscribed
    /// peer would reject as a rollback).
    #[tokio::test]
    async fn publish_directory_revision_is_monotone_across_publishes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let r1 = publish_directory(axum::extract::State(state.clone())).await;
        let b1 = to_bytes(r1.into_body(), usize::MAX).await.unwrap();
        let v1: serde_json::Value = serde_json::from_slice(&b1).unwrap();
        assert_eq!(v1["revision"], 1);

        let r2 = publish_directory(axum::extract::State(state.clone())).await;
        let b2 = to_bytes(r2.into_body(), usize::MAX).await.unwrap();
        let v2: serde_json::Value = serde_json::from_slice(&b2).unwrap();
        assert_eq!(v2["revision"], 2);
    }

    /// Sprint 75 Phase B: the revision counter persists on disk, so a logical
    /// restart (a fresh `DaemonHttpState` over the same home) continues the
    /// sequence rather than resetting to 1 — the scenario the doc comment
    /// motivates. Distinct from the same-state test above (which proves the
    /// write→read→write round-trip within one process lifetime).
    #[tokio::test]
    async fn publish_directory_revision_survives_logical_restart() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let s1 = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let r1 = publish_directory(axum::extract::State(s1)).await;
        let v1: serde_json::Value =
            serde_json::from_slice(&to_bytes(r1.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(v1["revision"], 1);

        // Fresh state, SAME on-disk home — simulates a daemon restart.
        let s2 = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let r2 = publish_directory(axum::extract::State(s2)).await;
        let v2: serde_json::Value =
            serde_json::from_slice(&to_bytes(r2.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(
            v2["revision"], 2,
            "the counter must survive a logical restart"
        );
    }

    /// Sprint 75 Phase B (review P1): production `DaemonHttpState` carries
    /// `sbfb_home: None`, so `next_directory_revision` MUST fall back to
    /// `auth::sbfb_home()` (`$SBFB_HOME` / `~/.sbfb`) — without it the counter
    /// resets to 1 on every boot and peers reject re-publishes as rollbacks.
    /// This drives the route with `sbfb_home: None` and only `$SBFB_HOME` set,
    /// the way production resolves it. (nextest runs each test in its own
    /// process, so the env mutation is isolated; no other test reads
    /// `$SBFB_HOME` via the fallback.)
    #[tokio::test]
    async fn publish_directory_revision_falls_back_to_sbfb_home_env() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // SAFETY (edition 2024): same env-mutation pattern the runtime tests use.
        unsafe {
            std::env::set_var("SBFB_HOME", tmp.path());
        }
        let state = mk_state().await; // sbfb_home: None — the production shape.
        let r1 = publish_directory(axum::extract::State(state.clone())).await;
        let v1: serde_json::Value =
            serde_json::from_slice(&to_bytes(r1.into_body(), usize::MAX).await.unwrap()).unwrap();
        let r2 = publish_directory(axum::extract::State(state.clone())).await;
        let v2: serde_json::Value =
            serde_json::from_slice(&to_bytes(r2.into_body(), usize::MAX).await.unwrap()).unwrap();
        unsafe {
            std::env::remove_var("SBFB_HOME");
        }
        assert_eq!(v1["revision"], 1, "first publish via env-resolved home");
        assert_eq!(
            v2["revision"], 2,
            "fallback home persists the counter (regression guard for the or_else fix)"
        );
    }

    /// Sprint 75 Phase B (review P1): the deploy/publish path imposes no length
    /// cap, but the directory signer enforces NODE_DIRECTORY_*_MAX. A single
    /// over-cap local app must NOT 500 the whole route — the field is truncated
    /// and the app still appears.
    #[tokio::test]
    async fn publish_directory_truncates_oversized_fields() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let held = hex::encode(blobs.add_bytes(b"zip-babel".to_vec()).await.unwrap());
        let mut entry = own_browse_entry(&"a".repeat(64), "Babel", Some(my_id));
        entry.archive_hash = Some(held);
        entry.description = "x".repeat(nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX + 50);
        state.browse_aggregator.add_direct_entry(entry);

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        assert_eq!(
            resp.status(),
            axum::http::StatusCode::OK,
            "an over-cap field must be clamped, not 500 the route"
        );
        let v: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(v["catalog_len"], 1);
        let archive_hash = v["archive_hash"].as_str().unwrap();
        let hash: [u8; 32] = hex::decode(archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let signed: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        signed
            .verify_signature()
            .expect("truncated directory must still verify");
        assert!(
            signed.directory.catalog[0].description.len()
                <= nexus_core_rs::NODE_DIRECTORY_DESCRIPTION_MAX,
            "description must be clamped to the cap"
        );
    }

    /// Sprint 75 Phase B (Codex round 2 GAP): a gossiped ProjectAnnouncement can
    /// forge `BrowseEntry.node_id == our node_id`. Such a spoofed entry — whose
    /// archive blob we do NOT hold — must never be signed into our directory
    /// (verrou 4: we only ever claim to host what we can actually serve).
    /// Content-addressing (local blob presence) is the ownership truth.
    #[tokio::test]
    async fn publish_directory_excludes_spoofed_unheld_blob() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let my_id = state.node.node_id();
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());

        // A real, locally-held app → legitimately advertised.
        let held = hex::encode(blobs.add_bytes(b"real-zip".to_vec()).await.unwrap());
        let mut real = own_browse_entry(&"a".repeat(64), "Real", Some(my_id.clone()));
        real.archive_hash = Some(held);
        state.browse_aggregator.add_direct_entry(real);

        // A spoofed entry: our node_id (as a remote gossip could forge), valid
        // hash FORMAT, but a blob we do NOT hold.
        let mut spoof = own_browse_entry(&"b".repeat(64), "Spoofed", Some(my_id));
        spoof.archive_hash = Some("c".repeat(64));
        state.browse_aggregator.add_direct_entry(spoof);

        let resp = publish_directory(axum::extract::State(state.clone())).await;
        let v: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
        assert_eq!(
            v["catalog_len"], 1,
            "only the locally-held app is advertised"
        );
        let hash: [u8; 32] = hex::decode(v["archive_hash"].as_str().unwrap())
            .unwrap()
            .try_into()
            .unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(entry.directory.catalog.len(), 1);
        assert_eq!(entry.directory.catalog[0].project_name, "Real");
        assert!(
            entry
                .directory
                .catalog
                .iter()
                .all(|app| app.project_name != "Spoofed"),
            "a spoofed entry whose blob we do not hold must never be signed in"
        );
    }

    /// Sprint 75 Phase B (Codex GAP): two CONCURRENT publishes (the daemon runs
    /// on a multi-thread runtime) must get strictly-distinct, monotone revisions
    /// — not both read the same value and sign two directories at the same
    /// revision (the second of which a peer would reject as a rollback). Guards
    /// the process-wide revision lock.
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn publish_directory_concurrent_revisions_are_distinct() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let (ra, rb) = tokio::join!(
            publish_directory(axum::extract::State(state.clone())),
            publish_directory(axum::extract::State(state.clone())),
        );
        let va: serde_json::Value =
            serde_json::from_slice(&to_bytes(ra.into_body(), usize::MAX).await.unwrap()).unwrap();
        let vb: serde_json::Value =
            serde_json::from_slice(&to_bytes(rb.into_body(), usize::MAX).await.unwrap()).unwrap();
        let mut revs = [
            va["revision"].as_u64().unwrap(),
            vb["revision"].as_u64().unwrap(),
        ];
        revs.sort_unstable();
        assert_eq!(
            revs,
            [1, 2],
            "concurrent publishes must produce distinct monotone revisions"
        );
    }

    #[tokio::test]
    async fn publish_announcement_persists_to_outbox_for_replay() {
        // Remediation #8 real-frontier test (§P57): the canonical announce path
        // must persist its envelope to the outbox even when ISOLATED
        // (gossip_sender == None). That persist-while-isolated is what lets a
        // deploy-from-repo / publish app be replayed to peers on NeighborUp AND
        // restored into Browse at boot (#7). No mock: real PoW envelope, real
        // mpsc channel, real aggregator.
        use nexus_core_rs::crypto::blake3_hash;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let pid = hex::encode(blake3_hash(b"Outbox Test App"));

        crate::deploy::publish_announcement(
            &state,
            crate::deploy::AnnouncementParams {
                project_id: &pid,
                project_name: "Outbox Test App",
                category: "tools",
                description: "persisted for replay",
                apps: &[],
                archive_hash: None,
                repo_url: None,
                provenance_hash: None,
                is_open_source: false,
            },
        )
        .await;

        // (a) the announce path pushed an Outbox command despite no live sender.
        let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("outbox send must arrive within 2s")
            .expect("channel open");
        let crate::runtime::GossipCmd::Outbox(payload) = cmd else {
            panic!("expected GossipCmd::Outbox, got a different command");
        };
        // (b) Sprint 75 Phase A: the outbox carries the UNWRAPPED announcement
        // payload (so every replay re-mints the address + re-stamps a fresh PoW),
        // NOT a frozen PoW envelope. It parses directly as a ProjectAnnouncement.
        assert!(
            nexus_shell_daemon_core::publish::is_project_announcement(&payload),
            "outbox entry must be the unwrapped announcement payload, not a PoW envelope"
        );
        let ann =
            nexus_shell_daemon_core::publish::ProjectAnnouncement::from_gossip_bytes(&payload)
                .expect("payload is a project announcement");
        assert_eq!(ann.project_id, pid);
        assert_eq!(ann.project_name, "Outbox Test App");
        // (c) the card is in the aggregator immediately as well.
        assert_eq!(state.browse_aggregator.direct_entry_count(), 1);
        assert!(state.browse_aggregator.get_direct_entry(&pid).is_some());
    }

    #[tokio::test]
    async fn contributor_verify_rejects_non_hex_path_params() {
        let app = build_test_router(mk_state().await);
        let bad_project = "NOT-HEX";
        let node_hex = "a".repeat(64);
        let uri = format!("/api/v1/contributor/verify/{bad_project}/{node_hex}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // ---- Sprint 75 Phase D: directory-only pull + node identity ----

    /// Sign a directory under `kp` (the anchor identity, possibly never
    /// dialable), host its blob on `host`, and ingest it into `state`'s
    /// curator runtime through the REAL subscription-gated path (subscribe +
    /// announcement + blob fetch + signature/revision verify).
    async fn ingest_remote_directory(
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

    fn catalog_app(project_id: &str, archive_hash: &str, name: &str) -> nexus_core_rs::CatalogApp {
        nexus_core_rs::CatalogApp {
            project_id: project_id.into(),
            archive_hash: archive_hash.into(),
            project_name: name.into(),
            category: "tools".into(),
            description: "fixture".into(),
        }
    }

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

    #[test]
    fn nodes_response_pins_envelope_and_grouping() {
        // Plan D.3 #6 (renamed from `nodes_endpoint_groups_by_node_id` for
        // honesty: this pins the PROJECTION — the entire handler body — and
        // the route itself is traversed over HTTP in
        // `reachable_via_seeder_status` part (c)). The /api/daemon/nodes
        // ENVELOPE shape is pinned now, before the Phase-F frontend consumer
        // exists (S73-E lesson: envelope, not bare array; S72-D lesson: never
        // ship a consumer-less shape without a producer-side pin test). Two
        // apps of one node stay grouped under ONE node element.
        let kp_a = KeyPair::generate();
        let kp_b = KeyPair::generate();
        let mut dir_a = nexus_core_rs::NodeDirectory::new(kp_a.public_bytes(), 3);
        dir_a.catalog = vec![
            catalog_app(&"1".repeat(64), &"a1".repeat(32), "Babel"),
            catalog_app(&"2".repeat(64), &"a2".repeat(32), "Atlas"),
        ];
        let entry_a = nexus_core_rs::NodeDirectoryEntry::sign(dir_a, &kp_a).unwrap();
        let mut dir_b = nexus_core_rs::NodeDirectory::new(kp_b.public_bytes(), 7);
        dir_b.catalog = vec![catalog_app(&"3".repeat(64), &"b1".repeat(32), "Solo")];
        let entry_b = nexus_core_rs::NodeDirectoryEntry::sign(dir_b, &kp_b).unwrap();

        // UX-ARRIVAL: the envelope also carries the observed (non-subscribed)
        // publishers — two cheap-envelope fields, freshest-first order pinned
        // by `observed_snapshot`, lowercase hex out.
        let observed_pk = [0xabu8; 32];
        let json = serde_json::to_value(nodes_response(
            vec![entry_a, entry_b],
            vec![(observed_pk, 1_700_000_123)],
        ))
        .unwrap();

        let nodes = json["nodes"]
            .as_array()
            .expect("envelope: a top-level `nodes` array, never a bare array");
        assert_eq!(nodes.len(), 2, "one element per publishing node");
        let observed = json["observed"]
            .as_array()
            .expect("envelope: a top-level `observed` array (always present — the frontend envelope is .strict())");
        assert_eq!(observed.len(), 1);
        assert_eq!(observed[0]["node_id"], hex::encode(observed_pk));
        assert_eq!(observed[0]["last_seen"], 1_700_000_123u64);
        assert_eq!(
            observed[0].as_object().unwrap().len(),
            2,
            "observed rows are cheap-envelope metadata: node_id + last_seen, never revision/app_count (no fetch for a non-subscribed node)"
        );
        // The envelope key-count is pinned (review WIRE-UXA-2): the frontend
        // schema is .strict() on the envelope, so ANY new top-level key must
        // ship both sides in the same commit — this assertion is the seam.
        assert_eq!(
            json.as_object().unwrap().len(),
            2,
            "envelope = exactly {{nodes, observed}}"
        );
        // The empty shape still serializes the key (the .strict() contract).
        let empty = serde_json::to_value(nodes_response(vec![], vec![])).unwrap();
        assert!(empty["observed"].as_array().unwrap().is_empty());
        assert!(empty["nodes"].as_array().unwrap().is_empty());
        assert_eq!(empty.as_object().unwrap().len(), 2);
        assert_eq!(nodes[0]["node_id"], hex::encode(kp_a.public_bytes()));
        assert_eq!(nodes[0]["revision"], 3);
        assert_eq!(nodes[0]["app_count"], 2);
        let cat = nodes[0]["catalog"].as_array().unwrap();
        assert_eq!(cat.len(), 2, "both apps grouped under their node");
        assert_eq!(cat[0]["project_id"], "1".repeat(64));
        assert_eq!(cat[0]["archive_hash"], "a1".repeat(32));
        assert_eq!(cat[0]["project_name"], "Babel");
        assert_eq!(nodes[1]["node_id"], hex::encode(kp_b.public_bytes()));
        assert_eq!(nodes[1]["revision"], 7);
        assert_eq!(nodes[1]["app_count"], 1);
    }

    #[test]
    fn shard_session_response_pins_empty_envelope() {
        // Sprint 77 Phase J. No live shard-session store exists yet (the
        // `sbfb/shard/1` data plane is not wired to a control-plane registry —
        // a tracked S78 carry), so EVERY id misses and the route answers a deterministic
        // empty envelope. 200 + `{found:false, session:null}`, NEVER a 404: the
        // frontend Zod schema is `.strict()` on the envelope and a miss must be a
        // SUCCESSFUL parse (seed_count precedent), not a transport error. The
        // `session` key is ALWAYS serialized (null), so an additive field stays
        // possible and the "no active session" empty state is unambiguous.
        let json = serde_json::to_value(shard_session_response("any-session-id")).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["found"], false);
        // The `session` key must be PHYSICALLY PRESENT (serialized as null on a
        // miss), not absent: `json["session"].is_null()` alone also passes for a
        // MISSING key under serde_json indexing, so assert key presence first.
        assert!(
            obj.contains_key("session"),
            "the session key is always serialized (.strict() envelope contract)"
        );
        assert!(obj["session"].is_null(), "session is null on a miss");
        assert_eq!(obj.len(), 2, "envelope = exactly {{found, session}}");
    }

    #[test]
    fn shard_session_projection_hides_member_identities() {
        // The projection is the PRIVACY seam (THREAT_MODEL §16 SI-3/SI-4): it
        // exposes an AGGREGATE `member_count` but NEVER a worker_pubkey /
        // initiator (the private group's composition). Two distinct workers must
        // collapse to `member_count: 2` with ZERO identity bytes in the
        // serialized view. The view is exactly the two whitelisted fields.
        let initiator = [0x11u8; 32];
        let worker_a = [0xAAu8; 32];
        let worker_b = [0xBBu8; 32];
        let mk = |pk: [u8; 32], start: u32, end: u32| nexus_core_rs::ShardAssignment {
            worker_pubkey: pk,
            layer_start: start,
            layer_end: end,
            role: nexus_core_rs::ShardRole::LayerWorker,
            shard_hashes: vec![[0x22u8; 32]],
            kv_cache_policy: nexus_core_rs::KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [0x33u8; 32],
        };
        let plan = nexus_core_rs::ShardPlan::new(vec![mk(worker_a, 0, 16), mk(worker_b, 16, 32)]);
        let manifest = nexus_core_rs::ShardedSessionManifest::new(
            initiator,
            "session-xyz",
            "group-abc",
            1,
            plan,
            [0x44u8; 32],
            [0x55u8; 32],
            [0x66u8; 32],
        );

        let view = project_shard_session(&manifest);
        assert_eq!(
            view.member_count, 2,
            "two workers collapse to an aggregate count"
        );

        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["session_id"], "session-xyz");
        assert_eq!(json["member_count"], 2);
        // The whitelist seam: NO member identity ever appears in the projection.
        let serialized = serde_json::to_string(&view).unwrap();
        assert!(
            !serialized.contains(&hex::encode(worker_a)),
            "worker_a pubkey must not leak"
        );
        assert!(
            !serialized.contains(&hex::encode(worker_b)),
            "worker_b pubkey must not leak"
        );
        assert!(
            !serialized.contains(&hex::encode(initiator)),
            "initiator pubkey must not leak"
        );
        assert!(!serialized.contains("worker_pubkey"));
        assert!(!serialized.contains("initiator"));
        assert_eq!(
            json.as_object().unwrap().len(),
            2,
            "view = exactly {{session_id, member_count}} — runtime status/level are additive fields"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn reachable_via_seeder_status() {
        // Q7 (plan D.3 #5) — the HONEST backend signal pair for a
        // directory-only app whose anchor is dead but whose bytes a seeder
        // still holds: (a) the Browse row NEVER lies `Reachable` on the dead
        // anchor, (b) the version-exact seed-count reports the live seeder.
        // The visible "reachable-via-seeder" badge that renders this pair is
        // Phase F (keeping `/browse` byte-identical in a core+daemon phase).
        let state = mk_state().await;
        let host = create_node().await.expect("boot host node");

        // The anchor identity never boots a node → its probe can only fail.
        let kp_anchor = KeyPair::generate();
        let pid = "d".repeat(64);
        let archive_hash = "ee".repeat(32);
        ingest_remote_directory(
            &state,
            &host,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Ghost App")],
            1,
        )
        .await;

        // A live seeder announced it holds this exact archive version.
        let seeder = hex::encode(KeyPair::generate().public_bytes());
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder, now, now);

        // (a) The browse row for the directory app reports the ANCHOR truth.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let entries = json["entries"].as_array().expect("entries array");
        let row = entries
            .iter()
            .find(|e| e["project_id"] == pid)
            .expect("the directory app must be discoverable (verrou 2)");
        assert_eq!(row["source"], "nodedirectory");
        assert_eq!(
            row["status"], "unreachable",
            "a dead anchor must never be reported Reachable (Q7 honesty)"
        );

        // (b) The version-exact seed-count carries the live-seeder signal.
        let app = build_test_router(state.clone());
        let uri = format!("/api/daemon/seed-count/{pid}?archive_hash={archive_hash}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["peer_count"], 1,
            "the seeder holding the BLAKE3 must be visible in the backend signal"
        );
        assert_eq!(json["self_seeding"], false);
        // WEB-1 (Phase F): never-toggled app → the persisted intent is null,
        // NOT false — the shell toggle must not render OFF for it.
        assert_eq!(json["self_pin_enabled"], serde_json::Value::Null);

        // (c) Route-level coverage of GET /api/daemon/nodes (the envelope
        // shape itself is pinned by `nodes_response_pins_envelope_and_grouping`):
        // the registered path serves the subscribed anchor's catalog.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/nodes")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let nodes = json["nodes"].as_array().expect("envelope over HTTP");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["node_id"], hex::encode(kp_anchor.public_bytes()));
        assert_eq!(nodes[0]["catalog"][0]["project_id"], pid);

        host.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_voluntary_directory_only_app() {
        // Sprint 75 Phase D closed GAP R5b: a directory-only app (no direct
        // entry, no ticket) becomes voluntarily seedable. The anchor identity
        // here is DEAD (never booted), so the pull must fall back to the
        // SeedRegistry seeder that actually holds the bytes — the full
        // multi-provider chain, E2E through the HTTP route.
        let state = mk_state().await;
        let seeder_node = create_node().await.expect("boot seeder node");

        // The seeder holds the app archive (author bytes, content-addressed).
        let payload = b"the-author-exact-archive-bytes".to_vec();
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        let archive_hash_bytes = blobs_seeder.add_bytes(&payload).await.unwrap();
        let archive_hash = hex::encode(archive_hash_bytes);

        // A dead anchor advertises the app in its (validly signed) directory,
        // whose blob the seeder node hosts.
        let kp_anchor = KeyPair::generate();
        let pid = "e".repeat(64);
        ingest_remote_directory(
            &state,
            &seeder_node,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Fallback App")],
            1,
        )
        .await;
        // NOT a direct entry — this is the directory-only shape.
        assert!(state.browse_aggregator.get_direct_entry(&pid).is_none());

        // The live seeder announced this exact version; seed its address so
        // the fallback dial resolves without live pkarr propagation timing.
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder_node.node_id(), now, now);
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must expose an address");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        // Phase F: the request pins the EXACT displayed version (the shell
        // passes the entry's archive_hash on every surface that knows it) —
        // this E2E exercises the discriminated resolution path end-to-end,
        // not the agnostic one.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": archive_hash})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "voluntary seed of a directory-only app must succeed via the seeder fallback"
        );

        // The node now HOLDS the author bytes under the exact hash...
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(blobs_local.has(archive_hash_bytes).await.unwrap());
        let got = blobs_local.get_bytes(archive_hash_bytes).await.unwrap();
        assert_eq!(got, payload, "content-addressing: the author's exact bytes");
        // ...pinned skip-GC under the keep-online tag — this is the ONLY test
        // exercising fetch_and_pin_multi, so the pin half of its contract
        // must be asserted here (mirror of the ticket-path test).
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "fetch_and_pin_multi must leave the keep-online pin tag behind"
        );
        // ...and the keep-online row records the seed for the boot re-announce.
        // Lexical block so the MutexGuard provably never crosses the await
        // below (clippy::await_holding_lock reasons on scopes, not drop()).
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            let row = db.get_keep_online(&pid).expect("keep_online read");
            assert_eq!(row, Some((true, Some(archive_hash.clone()))));
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_voluntary_version_discriminator_local_rejects() {
        // Sprint 75 Phase F (review-D deferral closed): the optional
        // `archive_hash` on POST /api/daemon/seed pins the EXACT version.
        // Local-rejection paths only — no fetch is ever started.
        let state = mk_state().await;

        // A direct card carries version A (ready ticket).
        let version_a = "aa".repeat(32);
        let pid = "5".repeat(64);
        let mut entry = own_browse_entry(&pid, "Two Versions", None);
        entry.archive_ticket = Some("ticket-version-a".into());
        entry.archive_hash = Some(version_a.clone());
        state.browse_aggregator.add_direct_entry(entry);

        // Asking for version B (listed nowhere) must NOT silently fall back
        // to the direct card's version A — 404, version-specific message.
        let version_b = "bb".repeat(32);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": version_b})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "a direct entry of a DIFFERENT version must not shadow the requested one"
        );
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            json["error"], "no source for the requested app version",
            "the rejection names the version miss, not an unknown app"
        );

        // Pre-F behaviour preserved: an archive-less direct card with no
        // requested version (and no directory fallback) is still a 400.
        let pid_bare = "6".repeat(64);
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid_bare, "No Archive", None));
        // own_browse_entry sets a placeholder hash — strip it to model the
        // archive-less card shape.
        let mut bare = state.browse_aggregator.get_direct_entry(&pid_bare).unwrap();
        bare.archive_hash = None;
        bare.archive_ticket = None;
        state.browse_aggregator.add_direct_entry(bare);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid_bare}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "app has no archive to seed");

        // The MATCHING-version branch takes the Ticket arm (review F P2: the
        // main prod path was never selection-pinned). The ticket is malformed
        // so the fetch fails fast — 502 "could not fetch", which proves the
        // selection entered the Ticket arm instead of 404ing the version.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid, "archive_hash": version_a})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::BAD_GATEWAY,
            "a matching requested version must select the direct ticket arm"
        );
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "could not fetch the app archive");

        // Case normalization: the SAME request with an UPPERCASE hash must
        // reach the same arm (hex-case lesson), never 404 on case alone.
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "project_id": pid,
                            "archive_hash": version_a.to_ascii_uppercase()
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);

        // A direct card with a hash but NO ticket (restored-from-outbox shape)
        // and no directory fallback: still the pre-F 400 — and the agnostic
        // fallback is narrowed by the card's own hash, so it can never pin a
        // DIFFERENT version than the one displayed (review F P3).
        let pid_hash_only = "8".repeat(64);
        let mut hash_only = own_browse_entry(&pid_hash_only, "Hash No Ticket", None);
        hash_only.archive_ticket = None;
        hash_only.archive_hash = Some("dd".repeat(32));
        state.browse_aggregator.add_direct_entry(hash_only);
        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid_hash_only}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["error"], "app has no archive to seed");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn seed_count_exposes_self_pin_intent() {
        // WEB-1 (Sprint 75 Phase F): `self_pin_enabled` is the THREE-valued
        // persisted intent — null (never toggled, still diffused by default),
        // true (explicit ON), false (explicit OFF). `self_seeding` stays the
        // version-scoped serving truth and must NOT be conflated with it.
        let state = mk_state().await;
        let pid = "7".repeat(64);
        let hash = "cd".repeat(32);

        let get_count = |uri: String| {
            let state = state.clone();
            async move {
                let app = build_test_router(state);
                let resp = app
                    .oneshot(
                        Request::builder()
                            .method(Method::GET)
                            .uri(&uri)
                            .body(axum::body::Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();
                assert_eq!(resp.status(), StatusCode::OK);
                let body = to_bytes(resp.into_body(), 4096).await.unwrap();
                serde_json::from_slice::<serde_json::Value>(&body).unwrap()
            }
        };

        // Never toggled: intent is null, not false.
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], serde_json::Value::Null);
        assert_eq!(json["self_seeding"], false);

        // Explicit ON.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash)).unwrap();
        }
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], true);
        assert_eq!(json["self_seeding"], true);
        // Intent is NOT version-scoped: a query about a DIFFERENT version
        // keeps the intent (true) while the serving truth drops to false.
        let other = "ef".repeat(32);
        let json = get_count(format!("/api/daemon/seed-count/{pid}?archive_hash={other}")).await;
        assert_eq!(json["self_pin_enabled"], true);
        assert_eq!(json["self_seeding"], false);

        // Explicit OFF.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, false, Some(&hash)).unwrap();
        }
        let json = get_count(format!("/api/daemon/seed-count/{pid}")).await;
        assert_eq!(json["self_pin_enabled"], false);
        assert_eq!(json["self_seeding"], false);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn boot_seed_driver_pins_configured_projects() {
        // Plan §E.3 #1 — THE Phase E test: a headless anchor seeds an app
        // it NEVER deployed locally, purely from its operator-written
        // `[seed]` accept-list. The app resolves through a subscribed node
        // directory (whose anchor identity is dead) and the bytes come
        // from a live seeder — the same Phase D multi-provider consumer
        // chain as seed_voluntary, never a ticket re-mint.
        let state = mk_state().await;
        let seeder_node = create_node().await.expect("boot seeder node");

        let payload = b"vps-config-driven-seed-bytes".to_vec();
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        let archive_hash_bytes = blobs_seeder.add_bytes(&payload).await.unwrap();
        let archive_hash = hex::encode(archive_hash_bytes);

        let kp_anchor = KeyPair::generate();
        let pid = "f".repeat(64);
        ingest_remote_directory(
            &state,
            &seeder_node,
            &kp_anchor,
            vec![catalog_app(&pid, &archive_hash, "Configured App")],
            1,
        )
        .await;
        assert!(
            state.browse_aggregator.get_direct_entry(&pid).is_none(),
            "the configured app must NOT be a local/direct app (never deployed here)"
        );

        // A live seeder announced this exact version; pre-seed its address
        // so the dial resolves without live pkarr propagation timing.
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        state
            .seed_registry
            .record(&pid, &archive_hash, &seeder_node.node_id(), now, now);
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder must expose an address");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        let pinned = run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await;
        assert_eq!(pinned, 1, "the configured app must be acquired + pinned");

        // The anchor now HOLDS the author's exact bytes (content-addressed)...
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(blobs_local.has(archive_hash_bytes).await.unwrap());
        assert_eq!(
            blobs_local.get_bytes(archive_hash_bytes).await.unwrap(),
            payload
        );
        // ...pinned skip-GC under the keep-online tag...
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "the boot driver must leave the keep-online pin tag behind"
        );
        // ...with the keep_online row recorded for future boots.
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).expect("keep_online read"),
                Some((true, Some(archive_hash.clone())))
            );
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn boot_repins_keep_online_blobs() {
        // Plan §E.3 #2 — re-pin, not just re-announce: a kept-online app
        // whose blob survived in the store but whose skip-GC tag is gone
        // gets its pin re-asserted at boot, with ZERO network involved
        // (the keep_online row's hash is the M18 source-of-truth).
        let state = mk_state().await;
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"locally-held-keep-online-bytes".to_vec();
        let hash = blobs.add_bytes(&payload).await.unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "9".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_hex)).unwrap();
        }
        let tag = crate::deploy::keep_online_tag(&pid);
        assert!(
            !has_tag(&state, &tag).await,
            "precondition: no keep-online tag before the driver runs"
        );

        // Deliberate duplicate in the configured list: the `seen` dedup
        // guarantees ONE acquisition — the counter discriminates (without
        // the guard, the idempotent set_tag would yield 2).
        let pinned = run_boot_seed_driver(&state, &[pid.clone(), pid.clone()]).await;
        assert_eq!(pinned, 1);
        assert!(
            has_tag(&state, &tag).await,
            "the driver must re-assert the skip-GC pin on locally-held bytes"
        );
    }

    #[tokio::test]
    async fn boot_seed_driver_empty_config_is_noop() {
        // Verrou 5: an empty accept-list (the compiled default, verrou 3)
        // does zero work. An unresolvable configured id is skipped loudly,
        // never fabricated into a keep_online row.
        let state = mk_state().await;
        assert_eq!(run_boot_seed_driver(&state, &[]).await, 0);

        let unknown = "8".repeat(64);
        assert_eq!(
            run_boot_seed_driver(&state, std::slice::from_ref(&unknown)).await,
            0
        );
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&unknown).unwrap(),
                None,
                "an unresolvable configured app must leave no keep_online row"
            );
        }
    }

    #[tokio::test]
    async fn vps_authoring_signs_own_directory() {
        // Plan §E.3 #4 + the Phase C producer-reannounce carry: the
        // headless authoring path — no HTTP route, no browser — signs THIS
        // node's directory with the node keypair, and the boot re-announce
        // is state-driven: a node that never published stays silent; one
        // that did re-signs at a bumped (monotone) revision.
        let tmp = tempfile::tempdir().expect("tempdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        // Never published → the boot re-announce must be a strict no-op.
        assert!(
            !reannounce_directory_at_boot(&state).await,
            "a node that never published a directory must stay silent at boot"
        );
        assert_eq!(read_directory_revision(&state), 0);

        // Publish headlessly via the boot-builder core (the same core the
        // HTTP route wraps): one OWN app whose blob this node holds.
        let my_id = state.node.node_id();
        let pid = "7".repeat(64);
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let held = hex::encode(blobs.add_bytes(b"vps-own-app-zip".to_vec()).await.unwrap());
        let mut e = own_browse_entry(&pid, "VpsApp", Some(my_id));
        e.archive_hash = Some(held);
        state.browse_aggregator.add_direct_entry(e);

        let out = build_sign_announce_directory(&state)
            .await
            .expect("headless publish must succeed");
        let DirectoryPublishOutcome::Published {
            node_id_hex,
            revision,
            catalog_len,
            archive_hash,
        } = out
        else {
            panic!("expected a Published outcome");
        };
        assert_eq!(revision, 1);
        assert_eq!(catalog_len, 1);
        assert_eq!(node_id_hex, hex::encode(state.pow_keypair.public_bytes()));
        // The stored blob is a verifiable signed directory — provenance is
        // the node keypair, no browser anywhere in this path.
        let hash: [u8; 32] = hex::decode(&archive_hash).unwrap().try_into().unwrap();
        let bytes = blobs.get_bytes(hash).await.unwrap();
        let entry: nexus_core_rs::NodeDirectoryEntry = serde_json::from_slice(&bytes).unwrap();
        entry
            .verify_signature()
            .expect("headless-published directory must verify");
        assert_eq!(entry.node_id, state.pow_keypair.public_bytes());

        // Reboot shape: the producer re-announce now fires and bumps the
        // monotone revision (a subscriber's persisted floor accepts it).
        assert!(
            reannounce_directory_at_boot(&state).await,
            "a publisher must re-announce its directory at boot"
        );
        assert_eq!(
            read_directory_revision(&state),
            2,
            "the boot re-announce re-signs at a bumped revision"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn request_seed_prod_caller() {
        // Plan §E.3 #3 (preflight delta #4 honored — REQUESTER role, not
        // the seed driver): `request_seed`'s first production caller is
        // the loopback route `POST /api/daemon/seed/request` — the author
        // asks a DESIGNATED peer (its anchor) to seed an app the author
        // holds. The peer runs the real `sbfb/seed/0` handler with its M19
        // invite ledger; the route signs with the node identity (the same
        // Ed25519 secret as the QUIC dialer, exactly the prod boot shape).
        use nexus_core_rs::node::{SEED_ALPN, create_node_with_protocols};
        use nexus_core_rs::{NodeConfig, create_node_with_config};

        // Requester state whose pow_keypair IS the node identity.
        let secret = KeyPair::generate().secret_bytes();
        let kp = KeyPair::from_secret_bytes(&secret);
        let node = create_node_with_config(NodeConfig::default().with_secret_key(secret))
            .await
            .expect("requester node");
        let mut state = (*mk_state().await).clone();
        state.node_id = node.node_id();
        state.node = Arc::new(node);
        state.pow_keypair = Arc::new(kp);
        let state = Arc::new(state);

        // The app: a local direct entry whose blob THIS node holds (the
        // route mints a fresh ticket — producer side, blob presence gated).
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"author-app-handed-to-anchor".to_vec();
        let hash = blobs.add_bytes(&payload).await.unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "6".repeat(64);
        let mut entry = own_browse_entry(&pid, "HandedApp", Some(state.node_id.clone()));
        entry.archive_hash = Some(hash_hex.clone());
        state.browse_aggregator.add_direct_entry(entry);

        // The designated seeder peer: real SeedProtocol handler + invite
        // minted for exactly (project_id, archive_hash) — M19.
        let seeder_secret = KeyPair::generate().secret_bytes();
        let seeder_kp = Arc::new(KeyPair::from_secret_bytes(&seeder_secret));
        let seeder_db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().expect("seeder db"),
        ));
        let factory = crate::seed_protocol::seed_protocol_factory(
            std::sync::Arc::clone(&seeder_db),
            Arc::clone(&seeder_kp),
            Arc::new(crate::seed_protocol::NonceCache::default()),
        );
        let seeder_node = create_node_with_protocols(
            NodeConfig::default().with_secret_key(seeder_secret),
            vec![(SEED_ALPN.to_vec(), factory)],
        )
        .await
        .expect("seeder node");
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        seeder_db
            .lock()
            .unwrap()
            .mint_seed_invite(
                "tok-prod-caller",
                &pid,
                &hash_hex,
                (now + 1000) as i64,
                Some(1),
            )
            .unwrap();
        // Tests skip live pkarr: pre-seed the requester's lookup (it
        // merges, so request_seed's empty-addr add cannot clobber it).
        let seeder_addr = nexus_core_rs::DiscoveryClient::new(seeder_node.endpoint())
            .my_endpoint_addr()
            .await
            .expect("seeder addr");
        state.node.memory_lookup().add_endpoint_info(seeder_addr);

        let body = serde_json::json!({
            "peer_node_id": seeder_node.node_id(),
            "project_id": pid,
            "invite_token": "tok-prod-caller",
        });
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(json["accepted"], true, "reason: {}", json["reason"]);
        assert_eq!(json["seeder_node_id"], seeder_node.node_id());

        // The designated peer now holds + keeps the author's exact bytes
        // (it re-signed no provenance — the author stays the author).
        let blobs_seeder = nexus_core_rs::BlobsClient::new(seeder_node.blobs_store());
        assert!(blobs_seeder.has(hash).await.unwrap());
        assert_eq!(blobs_seeder.get_bytes(hash).await.unwrap(), payload);
        {
            let db = seeder_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                Some((true, Some(hash_hex)))
            );
        }

        seeder_node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn boot_seed_driver_noop_in_duress() {
        // Review P1 (security): a decoy node must perform ZERO seed work —
        // no fetch, no keep_online mutation, no SeedAnnounced — even with a
        // resolvable configured list (the duress launcher shares the real
        // data root, so the driver would otherwise replay the operator's
        // real app set under the fake keypair).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let hash = blobs
            .add_bytes(b"duress-held-bytes".to_vec())
            .await
            .unwrap();
        let hash_hex = hex::encode(hash);
        let pid = "4".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_hex)).unwrap();
        }

        assert_eq!(
            run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await,
            0,
            "a decoy node must perform zero seed work"
        );
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "no pin tag may appear under duress"
        );
    }

    #[test]
    fn seed_already_announced_predicate() {
        // The driver's anti-double-emission guard, as pure logic: only an
        // app ALREADY enabled with the EXACT hash being pinned was covered
        // by reannounce_seeds_at_boot — everything else must emit.
        let h = "ab".repeat(32);
        assert!(seed_already_announced(&Some((true, Some(h.clone()))), &h));
        assert!(!seed_already_announced(
            &Some((true, Some("cd".repeat(32)))),
            &h
        ));
        assert!(!seed_already_announced(&Some((false, Some(h.clone()))), &h));
        assert!(!seed_already_announced(&Some((true, None)), &h));
        assert!(!seed_already_announced(&None, &h));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn boot_driver_prefers_keep_online_hash_over_directory() {
        // Pins the resolution priority (direct > keep_online row M18 >
        // subscribed directories): an anchor advertising a DIFFERENT hash
        // for the same project id must not override the M18 row's
        // source-of-truth hash, trigger a network fetch, or rewrite the row.
        let state = mk_state().await;
        let host = create_node().await.expect("host node");

        let blobs = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        let payload = b"version-A-bytes".to_vec();
        let hash_a = blobs.add_bytes(&payload).await.unwrap();
        let hash_a_hex = hex::encode(hash_a);
        let pid = "5".repeat(64);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            db.set_keep_online(&pid, true, Some(&hash_a_hex)).unwrap();
        }
        // A subscribed anchor advertises ANOTHER version of the same app.
        let kp_anchor = KeyPair::generate();
        let hash_b_hex = "bb".repeat(32);
        ingest_remote_directory(
            &state,
            &host,
            &kp_anchor,
            vec![catalog_app(&pid, &hash_b_hex, "Other Version")],
            1,
        )
        .await;

        let pinned = run_boot_seed_driver(&state, std::slice::from_ref(&pid)).await;
        assert_eq!(pinned, 1);
        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                Some((true, Some(hash_a_hex.clone()))),
                "the M18 row must keep hash A — never rewritten to the directory's hash"
            );
        }
        assert!(has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await);
        let hash_b: [u8; 32] = hex::decode(&hash_b_hex).unwrap().try_into().unwrap();
        assert!(
            !blobs.has(hash_b).await.unwrap(),
            "the directory's other version must never be fetched"
        );

        host.shutdown().await.ok();
    }

    #[tokio::test]
    async fn seed_request_peer_noop_in_duress() {
        // Mirrors publish_directory_noop_in_duress: never sign a
        // SeedRequest under the fake keypair — short-circuit BEFORE parse,
        // mint, or dial.
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let body = serde_json::json!({
            "peer_node_id": "ab".repeat(32),
            "project_id": "1".repeat(64),
            "invite_token": "tok",
        });
        let resp = build_test_router(state)
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(
            json["requested"], false,
            "duress must short-circuit before signing"
        );
    }

    #[tokio::test]
    async fn set_keep_online_noop_in_duress() {
        // Sprint 76 Phase B (B1): a decoy node must perform ZERO keep_online
        // mutation — no DB row, no blob skip-GC tag — and reply a plausible
        // benign success. The duress launcher shares the operator's REAL
        // coordinator.db + blob store, so an un-gated toggle would persist the
        // real app set under the fake keypair (local-mutation sibling of the
        // P1 wire-emit fix 23a08c9).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let pid = "7".repeat(64);
        // The app is visible, so a NON-duress toggle WOULD write a row + tag.
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid, "Decoy App", None));

        let resp = set_keep_online(
            State(state.clone()),
            Json(KeepOnlineRequest {
                project_id: pid.clone(),
                enabled: true,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                None,
                "duress must not persist a keep_online row"
            );
        }
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "duress must not tag the archive blob"
        );
    }

    #[tokio::test]
    async fn seed_voluntary_noop_in_duress() {
        // Sprint 76 Phase B (B1): a decoy node must perform ZERO voluntary-seed
        // work — no fetch, no pin, no keep_online row, no SeedAnnounced — and
        // reply a plausible benign success. The single early-return covers BOTH
        // the local pin and the emit (the local-mutation sibling of 23a08c9).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let pid = "8".repeat(64);
        state
            .browse_aggregator
            .add_direct_entry(own_browse_entry(&pid, "Decoy App", None));

        let resp = seed_voluntary(
            State(state.clone()),
            Json(SeedVoluntaryRequest {
                project_id: pid.clone(),
                archive_hash: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(resp.status(), StatusCode::OK);

        {
            let db = state
                .coordinator_db
                .lock()
                .unwrap_or_else(|p| p.into_inner());
            assert_eq!(
                db.get_keep_online(&pid).unwrap(),
                None,
                "duress must not persist a keep_online row"
            );
        }
        assert!(
            !has_tag(&state, &crate::deploy::keep_online_tag(&pid)).await,
            "duress must not tag/pin under the fake keypair"
        );
    }

    #[test]
    fn pull_falls_back_across_tiers_when_ticket_dead() {
        // Sprint 76 Phase B (B3, PULL-3): when a direct entry carries a ticket
        // AND the subscribed directories resolve the app, the fetch chain has
        // BOTH tiers IN ORDER — ticket FIRST, directory multi-provider SECOND.
        // The handler loop tries them in order, so a dead tier-1 ticket falls
        // through to tier 2 instead of a terminal BAD_GATEWAY (pre-B3 a
        // ticket-bearing entry produced only [Ticket]).
        let chain = build_seed_fetch_chain(
            Some(("aa".repeat(32), SeedFetchPlan::Ticket("dead-ticket".into()))),
            Some(("aa".repeat(32), SeedFetchPlan::Multi(vec![]))),
        );
        assert_eq!(
            chain.len(),
            2,
            "both tiers must be present so a dead ticket can fail over to the directory"
        );
        assert!(
            matches!(chain[0].1, SeedFetchPlan::Ticket(_)),
            "the cheap ticket tier must be tried first"
        );
        assert!(
            matches!(chain[1].1, SeedFetchPlan::Multi(_)),
            "the resilient directory tier must be the fallback"
        );

        // Ticket-only (no directory hit) → single tier, unchanged.
        let only_ticket = build_seed_fetch_chain(
            Some(("bb".repeat(32), SeedFetchPlan::Ticket("t".into()))),
            None,
        );
        assert_eq!(only_ticket.len(), 1);
        assert!(matches!(only_ticket[0].1, SeedFetchPlan::Ticket(_)));

        // Directory-only (ticket-less app) → single directory tier.
        let only_dir =
            build_seed_fetch_chain(None, Some(("cc".repeat(32), SeedFetchPlan::Multi(vec![]))));
        assert_eq!(only_dir.len(), 1);
        assert!(matches!(only_dir[0].1, SeedFetchPlan::Multi(_)));

        // No tier → empty chain (the handler then returns the precise 400/404).
        assert!(build_seed_fetch_chain(None, None).is_empty());
    }

    #[tokio::test]
    async fn seed_request_peer_rejects_local_errors() {
        // The four pure-local rejections of the requester route — no
        // network, no peer: malformed id, self-designation, unknown app,
        // and the held-bytes gate (a node never proposes bytes it does not
        // hold — the producer-side mint enforces it).
        let state = mk_state().await;

        // (1) malformed peer id -> 400.
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": "zzz", "project_id": "1".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // (2) self-designation -> 400 (parsed-identity compare).
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": state.node_id, "project_id": "1".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // (3) unknown app -> 404.
        let other_peer = hex::encode(KeyPair::generate().public_bytes());
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": other_peer, "project_id": "2".repeat(64)})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // (4) app whose archive blob is NOT held locally -> 409.
        let pid = "3".repeat(64);
        let mut entry = own_browse_entry(&pid, "GhostBytes", Some(state.node_id.clone()));
        entry.archive_hash = Some("ee".repeat(32));
        state.browse_aggregator.add_direct_entry(entry);
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed/request")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"peer_node_id": other_peer, "project_id": pid})
                            .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::CONFLICT,
            "a node must never ask a peer to seed bytes it does not itself hold"
        );
    }

    #[tokio::test]
    async fn health_returns_200_with_fixed_shape() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["daemon_version"], "0.1.0-test");
    }

    #[tokio::test]
    async fn info_returns_full_snapshot() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let snap: DaemonStateSnapshot = serde_json::from_slice(&body).unwrap();
        assert_eq!(snap.schema_version, 1);
        assert_eq!(snap.node_id.len(), 64);
        assert_eq!(snap.api_host, "127.0.0.1");
        assert_eq!(snap.api_port, 12345);
        assert!(snap.subscribed_curators.is_empty());
        assert_eq!(snap.known_lists, 0);
        assert_eq!(snap.known_browse_entries, 0);
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/nope")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

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

    #[tokio::test]
    async fn info_reflects_live_curator_runtime_counts() {
        let state = mk_state().await;
        let kp = KeyPair::generate();
        state
            .curator_runtime
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();

        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let snap: DaemonStateSnapshot =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(snap.subscribed_curators.len(), 1);
    }

    #[tokio::test]
    async fn browse_returns_empty_list_when_no_curators_cached() {
        // Phase D smoke test: with an empty curator runtime the
        // aggregator has nothing to flatten, so /browse returns
        // `{"entries": []}` at 200. The full Reachable/Unreachable
        // behaviour is covered by the 2-node integration tests
        // in `browse::tests::aggregate_probes_seeded_peer_*`.
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
    }

    #[tokio::test]
    async fn publish_returns_200_and_adds_direct_entry() {
        // Sprint 11 Phase A: POST /publish adds a direct entry
        // to the browse aggregator and returns published=true.
        // Gossip broadcast is skipped (sender is None in tests).
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "gov-officiel".into(),
            category: "gov".into(),
            description: "Le projet gouvernance".into(),
            apps: vec!["gov".into()],
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let pub_resp: PublishResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(pub_resp.published);

        // The direct entry must now appear in /browse.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert_eq!(browse.entries[0].project_name, "gov-officiel");
        assert_eq!(
            serde_json::to_string(&browse.entries[0].source).unwrap(),
            "\"direct\""
        );
    }

    // ---------------------------------------------------------
    // Sprint 16 audit finding D-1 regression
    // ---------------------------------------------------------

    #[tokio::test]
    async fn publish_rejects_is_open_source_without_provenance_chain() {
        // Sprint 16 audit finding D-1: a malicious local process
        // holding the bearer token must not be able to flag a
        // zip deploy as open source without going through the
        // coord's deploy-from-repo clone+verify+sign path. The
        // daemon rejects `is_open_source=true` unless both
        // `provenance_hash` and `repo_url` are present.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // Case 1: flag=true, no provenance_hash, no repo_url → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-open-source".into(),
            category: "misc".into(),
            description: "pretend I'm OSS".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Case 2: flag=true, provenance_hash present, repo_url absent → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-2".into(),
            category: "misc".into(),
            description: "still pretending".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: None,
            provenance_hash: Some("cd".repeat(32)),
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        // Case 3: flag=true, repo_url present, provenance_hash absent → 400
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "fake-3".into(),
            category: "misc".into(),
            description: "one more try".into(),
            apps: vec![],
            archive_hash: Some("ab".repeat(32)),
            repo_url: Some("https://example.com/repo".into()),
            provenance_hash: None,
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn publish_accepts_is_open_source_with_full_provenance_chain() {
        // Mirror of the D-1 reject test: the happy path — both
        // provenance_hash and repo_url present — passes. This is
        // what the coord's `POST /project/deploy-from-repo` emits
        // after cloning and signing.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "legit-oss".into(),
            category: "gov".into(),
            description: "verified from repo".into(),
            apps: vec!["gov".into()],
            archive_hash: Some("ab".repeat(32)),
            repo_url: Some("https://github.com/example/sbfb-app".into()),
            provenance_hash: Some("cd".repeat(32)),
            is_open_source: true,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Browse entry must carry is_open_source=true.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert!(browse.entries[0].is_open_source);
    }

    // ---------------------------------------------------------
    // CORS helper unit tests (Phase A regression)
    // ---------------------------------------------------------

    #[test]
    fn loopback_origin_accepts_127_0_0_1_with_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:3000");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_accepts_localhost_without_port() {
        let h = HeaderValue::from_static("http://localhost");
        assert!(is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_remote_host() {
        let h = HeaderValue::from_static("http://example.com");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_https_scheme() {
        let h = HeaderValue::from_static("https://127.0.0.1");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_malformed_port() {
        let h = HeaderValue::from_static("http://127.0.0.1:not-a-port");
        assert!(!is_loopback_origin(&h));
    }

    #[test]
    fn loopback_origin_rejects_suffix_trick() {
        let h = HeaderValue::from_static("http://127.0.0.1.evil.com");
        assert!(!is_loopback_origin(&h));
    }

    // ---------------------------------------------------------
    // Sprint 33 Phase A: CORS layer with --cors-origin
    // ---------------------------------------------------------

    #[test]
    fn valid_origin_accepts_http_with_port() {
        assert!(is_valid_origin("http://192.168.1.10:8080"));
    }

    #[test]
    fn valid_origin_accepts_https_without_port() {
        assert!(is_valid_origin("https://example.com"));
    }

    #[test]
    fn valid_origin_rejects_no_scheme() {
        assert!(!is_valid_origin("192.168.1.10:8080"));
    }

    #[test]
    fn valid_origin_rejects_with_path() {
        assert!(!is_valid_origin("http://example.com/path"));
    }

    #[test]
    fn valid_origin_rejects_javascript_scheme() {
        assert!(!is_valid_origin("javascript:alert('xss')"));
    }

    #[test]
    fn valid_origin_rejects_data_scheme() {
        assert!(!is_valid_origin("data:text/html,<script>alert(1)</script>"));
    }

    #[test]
    fn valid_origin_rejects_file_scheme() {
        assert!(!is_valid_origin("file:///etc/passwd"));
    }

    fn build_cors_test_router(state: Arc<DaemonHttpState>, cors: &[String]) -> Router {
        build_router(state, AuthState::new(TEST_TOKEN.to_string()), cors, None)
    }

    #[tokio::test]
    async fn cors_loopback_default_allows_localhost() {
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
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(acao.is_some(), "loopback origin must be allowed by default");
    }

    #[tokio::test]
    async fn cors_loopback_default_rejects_external() {
        let app = build_cors_test_router(mk_state().await, &[]);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://192.168.1.10:8080")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_none(),
            "external origin must be rejected by default"
        );
    }

    #[tokio::test]
    async fn cors_custom_origin_allows_configured() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://192.168.1.10:8080")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(acao.is_some(), "configured origin must be allowed");
    }

    #[tokio::test]
    async fn cors_custom_origin_preserves_loopback() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
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
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_some(),
            "loopback must still be allowed with custom origins"
        );
    }

    #[tokio::test]
    async fn cors_rejects_unconfigured_external() {
        let origins = vec!["http://192.168.1.10:8080".to_string()];
        let app = build_cors_test_router(mk_state().await, &origins);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/health")
                    .header("origin", "http://evil.com:9999")
                    .header("access-control-request-method", "GET")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let acao = resp.headers().get("access-control-allow-origin");
        assert!(
            acao.is_none(),
            "unconfigured external origin must be rejected"
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

    // ---------------------------------------------------------
    // Sprint 76 Phase H: project-info endpoint (bridge compute)
    // ---------------------------------------------------------

    #[tokio::test]
    async fn project_info_field_present_and_null_without_doc() {
        // The bridge reads `project_doc_id` to inject the submission's
        // `project_id` (the local worker is whitelisted to it). The
        // field must always be present so the bridge can branch on
        // `null`; the `mk_state` harness mounts no project doc.
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/project-info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            v.get("project_doc_id").is_some(),
            "project_doc_id must always be present"
        );
        assert!(
            v["project_doc_id"].is_null(),
            "project_doc_id is null when no project doc is mounted"
        );
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

    // ---------------------------------------------------------
    // Sprint 12 Phase A: blob-serve + publish-blob
    // ---------------------------------------------------------

    /// Helper: create a minimal zip archive in memory.
    fn make_zip(files: &[(&str, &[u8])]) -> Vec<u8> {
        use std::io::Write;
        let buf = Vec::new();
        let cursor = std::io::Cursor::new(buf);
        let mut writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for (name, data) in files {
            writer.start_file(*name, options).unwrap();
            writer.write_all(data).unwrap();
        }
        writer.finish().unwrap().into_inner()
    }

    #[tokio::test]
    async fn publish_blob_stores_and_returns_hash() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let zip_bytes = make_zip(&[("index.html", b"<h1>Hello</h1>")]);

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(zip_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let res: PublishBlobResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(res.hash.len(), 64, "hash should be 32 bytes hex-encoded");
    }

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
    async fn archive_hash_from_ticket_decodes_the_hash() {
        let node = create_node().await.unwrap();
        let blobs = BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"some bytes".to_vec()).await.unwrap();
        let hash_hex = hex::encode(hash);
        let addr = nexus_core_rs::DiscoveryClient::new(node.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();
        assert_eq!(archive_hash_from_ticket(&ticket), Some(hash_hex));
        assert_eq!(archive_hash_from_ticket("not-a-valid-ticket"), None);
        node.shutdown().await.ok();
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
    async fn publish_with_archive_hash_populates_browse_entry() {
        // Sprint 12 Phase D: POST /publish with archive_hash
        // sets archive_hash on the browse entry visible in /browse.
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // Store a blob first.
        let zip_bytes = make_zip(&[("index.html", b"<h1>Hi</h1>")]);
        let blobs = BlobsClient::new(state.node.blobs_store());
        let hash = blobs.add_bytes(zip_bytes).await.unwrap();
        let hash_hex = hex::encode(hash);

        // Publish with archive_hash.
        let body = serde_json::to_vec(&PublishRequest {
            project_name: "web-app".into(),
            category: "misc".into(),
            description: "test archive".into(),
            apps: vec![],
            archive_hash: Some(hash_hex.clone()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // Verify /browse returns the entry with archive_hash.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(browse.entries.len(), 1);
        assert_eq!(
            browse.entries[0].archive_hash.as_deref(),
            Some(hash_hex.as_str())
        );
        assert!(browse.entries[0].archive_ticket.is_some());
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

    // =============================================================
    // Sprint 20 Phase B — duress runtime HTTP surface
    // =============================================================

    /// #B-rt-1 The `/publish` handler in Duress mode returns 200
    /// with `{published: false}` and does NOT add a direct entry
    /// to the browse aggregator. A peer observer sees no gossip
    /// broadcast (the gossip_sender is None here so we rely on
    /// the handler short-circuit before even reading the sender
    /// guard — the empty browse aggregator is the local witness).
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_publishes_fake_curator_empty() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));

        let body = serde_json::to_vec(&PublishRequest {
            project_name: "real-project".into(),
            category: "gov".into(),
            description: "should-not-reach-wire".into(),
            apps: vec!["gov".into()],
            archive_hash: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        })
        .unwrap();

        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let pub_resp: PublishResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            !pub_resp.published,
            "Duress mode must report published=false (no wire broadcast)"
        );

        // The browse aggregator must be empty — no direct entry
        // was added under the fake identity.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let browse: BrowseListResponse =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            browse.entries.is_empty(),
            "browse aggregator must stay empty in Duress mode"
        );
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

    /// #B-rt-3 The `/publish-blob` handler in Duress mode returns
    /// 503 with a generic "maintenance" payload — no signal that
    /// duress is active, just a plausible service-unavailable.
    #[tokio::test]
    async fn daemon_boot_in_duress_mode_rejects_task_dispatch() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish-blob")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(b"fake blob bytes".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    // ---------------------------------------------------------
    // Sprint 23 Phase E: diagnostic neighborhood endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn diagnostic_neighborhood_returns_own_node_id_and_empty_peers() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/diagnostic/neighborhood")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let neighborhood: NeighborhoodResponse = serde_json::from_slice(&body).unwrap();
        assert_eq!(neighborhood.node_id, expected_node_id);
        assert!(
            neighborhood.peers.is_empty(),
            "fresh node should have no known peers"
        );
    }

    // ---------------------------------------------------------
    // Sprint 53 Phase A: SPA route collision regression tests
    // ---------------------------------------------------------

    fn build_test_router_with_web_root(state: Arc<DaemonHttpState>) -> (Router, tempfile::TempDir) {
        use axum::http::header::{HOST, ORIGIN};
        use std::fs;
        let tmp = tempfile::tempdir().expect("tempdir for web_root");
        fs::write(
            tmp.path().join("index.html"),
            "<!doctype html><div id=root></div>",
        )
        .unwrap();
        let router = build_router(
            state,
            AuthState::new(TEST_TOKEN.to_string()),
            &[],
            Some(tmp.path()),
        )
        .layer(middleware::from_fn(
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
        ));
        (router, tmp)
    }

    #[tokio::test]
    async fn spa_fallback_serves_browse_as_html_document() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/html"),
            "GET /browse with web_root must serve SPA HTML, got content-type: {ct}"
        );
    }

    #[tokio::test]
    async fn spa_fallback_serves_curators_as_html_document() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/curators")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let ct = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            ct.contains("text/html"),
            "GET /curators with web_root must serve SPA HTML, got content-type: {ct}"
        );
    }

    #[tokio::test]
    async fn api_daemon_browse_still_returns_json_with_web_root() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let list: BrowseListResponse = serde_json::from_slice(&body).unwrap();
        assert!(list.entries.is_empty());
    }

    #[tokio::test]
    async fn api_daemon_info_still_returns_json_with_web_root() {
        let state = mk_state().await;
        let (app, _tmp) = build_test_router_with_web_root(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/info")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let snap: DaemonStateSnapshot = serde_json::from_slice(&body).unwrap();
        assert_eq!(snap.schema_version, 1);
    }

    // =============================================================
    // Sprint 31 Phase D — HTTP integration tests FROST endpoints
    // =============================================================

    #[tokio::test]
    async fn frost_http_trusted_dealer_returns_shares_and_pubkey() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let shares = json["shares"].as_array().expect("shares array");
        assert_eq!(shares.len(), 3, "K=2/N=3 must produce 3 shares");
        for (i, share) in shares.iter().enumerate() {
            assert_eq!(
                share["participant"].as_u64().unwrap(),
                (i + 1) as u64,
                "share participant must be 1-indexed"
            );
            assert!(
                !share["key_package_hex"].as_str().unwrap().is_empty(),
                "key_package_hex must be non-empty"
            );
        }
        let pubkey = &json["pubkey_package"];
        assert!(
            !pubkey["verifying_key_hex"].as_str().unwrap().is_empty(),
            "verifying_key_hex must be non-empty"
        );
        assert_eq!(
            pubkey["verifying_key_hex"].as_str().unwrap().len(),
            64,
            "verifying key must be 32 bytes (64 hex chars)"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_returns_commitment_and_nonces() {
        let app_dealer = build_test_router(mk_state().await);
        let dealer_resp = app_dealer
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dealer_body = to_bytes(dealer_resp.into_body(), 16384).await.unwrap();
        let dealer_json: serde_json::Value = serde_json::from_slice(&dealer_body).unwrap();
        let share = &dealer_json["shares"][0];
        let key_package_hex = share["key_package_hex"].as_str().unwrap();
        let participant = share["participant"].as_u64().unwrap() as u16;

        let round1_body = serde_json::json!({
            "participant": participant,
            "key_package_hex": key_package_hex
        });

        let app_round1 = build_test_router(mk_state().await);
        let resp = app_round1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(round1_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            !json["commitment"]["commitment_hex"]
                .as_str()
                .unwrap()
                .is_empty(),
            "commitment_hex must be non-empty"
        );
        assert!(
            !json["nonces"]["nonces_hex"].as_str().unwrap().is_empty(),
            "nonces_hex must be non-empty"
        );
        assert_eq!(
            json["commitment"]["participant"].as_u64().unwrap(),
            participant as u64
        );
    }

    #[tokio::test]
    async fn frost_http_round2_returns_signature_share() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r1_resp.status(), StatusCode::OK);
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let sp = nexus_shell_daemon_core::canary::build_signing_package(
            &commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect::<Vec<nexus_shell_daemon_core::canary::CeremonyCommitment>>(),
            b"round2 HTTP test message",
        )
        .expect("build signing package");

        let r2_body = serde_json::json!({
            "nonces": nonces_list[0],
            "signing_package": sp,
            "key_package_hex": dj["shares"][0]["key_package_hex"],
            "participant": dj["shares"][0]["participant"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round2")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r2_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(
            !json["signature_share_hex"].as_str().unwrap().is_empty(),
            "signature_share_hex must be non-empty"
        );
    }

    #[tokio::test]
    async fn frost_http_aggregate_returns_valid_signature() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let message = b"aggregate HTTP test message";
        let ceremony_commitments: Vec<nexus_shell_daemon_core::canary::CeremonyCommitment> =
            commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect();
        let sp =
            nexus_shell_daemon_core::canary::build_signing_package(&ceremony_commitments, message)
                .expect("build signing package");

        let mut sig_shares = Vec::new();
        for i in 0..2 {
            let r2_body = serde_json::json!({
                "nonces": nonces_list[i],
                "signing_package": sp,
                "key_package_hex": dj["shares"][i]["key_package_hex"],
                "participant": dj["shares"][i]["participant"]
            });
            let app = build_test_router(Arc::clone(&state));
            let resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round2")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r2_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let rj: serde_json::Value =
                serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
            sig_shares.push(rj);
        }

        let agg_body = serde_json::json!({
            "signing_package": sp,
            "shares": sig_shares,
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        let sig_hex = json["signature_hex"]
            .as_str()
            .expect("signature_hex present");
        assert_eq!(sig_hex.len(), 128, "64-byte Ed25519 sig = 128 hex chars");

        let sig_bytes = hex::decode(sig_hex).expect("valid hex");
        let vk_hex = dj["pubkey_package"]["verifying_key_hex"].as_str().unwrap();
        let vk_bytes = hex::decode(vk_hex).expect("valid vk hex");
        let vk: [u8; 32] = vk_bytes.try_into().expect("32 bytes");
        let sig: [u8; 64] = sig_bytes.try_into().expect("64 bytes");
        nexus_core_rs::crypto::verify(&vk, message, &sig)
            .expect("aggregated FROST sig must verify as Ed25519");
    }

    #[tokio::test]
    async fn frost_http_invalid_threshold_k_gt_n() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":5,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_malformed_json_body() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k": not valid json"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_client_error(),
            "malformed JSON should return 4xx"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_invalid_key_package() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"participant":1,"key_package_hex":"deadbeef"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_aggregate_invalid_pubkey() {
        let state = mk_state().await;

        let app = build_test_router(Arc::clone(&state));
        let dealer_resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let agg_body = serde_json::json!({
            "signing_package": { "signing_package_hex": "deadbeef" },
            "shares": [],
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    // ===============================================================
    // Sprint 36 Phase B — result submission integration tests
    // ===============================================================

    fn make_test_submission() -> nexus_coordinator_rs::types::TaskSubmission {
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

    fn make_result_entry(task_id: &str, worker_kp: &KeyPair) -> nexus_core_rs::task::ResultEntry {
        make_result_entry_with_text(task_id, worker_kp, "result text")
    }

    fn make_result_entry_with_text(
        task_id: &str,
        worker_kp: &KeyPair,
        text: &str,
    ) -> nexus_core_rs::task::ResultEntry {
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: text.into(),
            tokens_generated: 42,
            generation_time_ms: 1000,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 100,
            finished_at: 200,
            output_token_ids: vec![],
        };
        nexus_core_rs::task::ResultEntry::sign(payload, worker_kp).expect("sign result")
    }

    #[tokio::test]
    async fn result_submit_accepts_valid() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "accepted");

        let db = state.coordinator_db.lock().unwrap();
        let task = db
            .get_task(&task_entry.task.task_id)
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            nexus_coordinator_rs::types::TaskStatus::Completed
        );
    }

    #[tokio::test]
    async fn result_submit_rejects_bad_signature() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let mut result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);
        result_entry.signature[0] ^= 0xff;

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "bad_signature");
    }

    #[tokio::test]
    async fn result_submit_rejects_unknown_task() {
        let state = mk_state().await;
        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry("nonexistent-task-id", &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_found");
    }

    #[tokio::test]
    async fn result_submit_rejects_completed_task() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        {
            let db = state.coordinator_db.lock().unwrap();
            db.set_task_result(&task_entry.task.task_id, "w1", "r1", "prior text", 100)
                .expect("complete");
        }

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_pending");
    }

    // ===============================================================
    // Sprint 73 Phase A — guardrail-before-persist (D5)
    // ===============================================================

    #[tokio::test]
    async fn submit_result_rejected_by_guardrail_persists_nothing() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        // An invisible character trips the output guardrail deterministically.
        let result_entry = make_result_entry_with_text(
            &task_entry.task.task_id,
            &worker_kp,
            "leaked\u{200B}secret",
        );

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "rejected");
        assert_eq!(body["reason"], "guardrail_rejected");

        // Sprint 73 Phase A (D5): the guardrail runs BEFORE persistence, so a
        // rejected result leaves no completed task, no retrievable text, and
        // credits no kudos. Since CARRY-2 (Sprint 75 Phase G) the trip is also
        // TERMINAL on this HTTP path: the task flips to Rejected instead of
        // silently keeping its prior non-terminal state.
        let db = state.coordinator_db.lock().unwrap();
        let task = db
            .get_task(&task_entry.task.task_id)
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            nexus_coordinator_rs::types::TaskStatus::Rejected,
            "guardrail-rejected result must terminally reject the task (CARRY-2)"
        );
        assert!(
            db.get_task_result(&task_entry.task.task_id)
                .expect("get")
                .expect("found")
                .result_text
                .is_none(),
            "guardrail-rejected result must persist no retrievable text"
        );
        assert_eq!(
            db.get_project_kudos_total("test-project").expect("kudos"),
            0,
            "no kudos for guardrail-rejected output"
        );
    }

    #[tokio::test]
    async fn submit_result_accepted_persists_after_guardrail() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry =
            make_result_entry_with_text(&task_entry.task.task_id, &worker_kp, "clean answer");

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "accepted");

        // The cleared text is persisted retrievably only after the guardrail.
        let db = state.coordinator_db.lock().unwrap();
        assert_eq!(
            db.get_task_result(&task_entry.task.task_id)
                .expect("get")
                .expect("found")
                .result_text
                .as_deref(),
            Some("clean answer"),
        );
    }

    // ===============================================================
    // Sprint 36 Phase C — kudos integration tests
    // ===============================================================

    #[tokio::test]
    async fn e2e_task_result_kudos_credited() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let db = state.coordinator_db.lock().unwrap();
        let total = db
            .get_project_kudos_total("test-project")
            .expect("kudos total");
        assert!(total > 0, "kudos must be credited after accepted result");
    }

    #[tokio::test]
    async fn kudos_endpoint_returns_json() {
        let state = mk_state().await;

        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                "proj-abc",
                "worker-xyz",
                "task-1",
                100,
                1_000,
            )
            .expect("credit");
        }

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-abc")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["project_id"], "proj-abc");
        assert!(
            body["total"].as_u64().unwrap() > 0,
            "total must be positive after credit"
        );
        assert_eq!(body["contributors"][0]["worker_node_id"], "worker-xyz");
    }

    // =========================================================
    // Mutex poisoned tests (P2-REVIEW-A-1/B-1)
    // =========================================================

    #[tokio::test]
    async fn submit_task_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        // Poison the mutex by panicking while holding the guard.
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(
            state.coordinator_db.lock().is_err(),
            "mutex must be poisoned"
        );

        let app = build_test_router(state);
        let body = serde_json::json!({
            "project_id": "p1",
            "task_type": "inference",
            "prompt": "test",
            "system_prompt": "",
            "model": "llama3"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn submit_result_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let kp = KeyPair::generate();
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: "t-1".to_string(),
            result_text: "out".to_string(),
            tokens_generated: 1,
            generation_time_ms: 1,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 0,
            finished_at: 1,
            output_token_ids: vec![],
        };
        let entry = nexus_core_rs::task::ResultEntry::sign(payload, &kp).unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&entry).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_kudos_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn verify_chain_endpoint_returns_valid() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(
                &db, "proj-vc", "worker-a", "task-1", 10, 1_000,
            )
            .expect("credit");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-vc/verify")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["valid"], true);
    }

    #[tokio::test]
    async fn submit_task_pii_rejected() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let mut sub = make_test_submission();
        sub.prompt = "Contact me at test@example.com for details".into();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(serde_json::to_vec(&sub).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["error"], "input_rejected");
    }

    #[tokio::test]
    async fn canary_observed_post_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let payload = serde_json::json!({
            "version": 1,
            "pubkey_hex": "aa".repeat(32),
            "date": "2026-04-29",
            "headline": "All clear",
            "next_update": "2026-05-29",
            "signature_hex": "bb".repeat(64)
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/canary/observed")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&payload).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
    }

    #[tokio::test]
    async fn canary_network_health_get_ok() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/canary/network-health")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // =============================================================
    // Sprint 46 Phase A — integration tests 12 MANDATORY routes
    // =============================================================

    // --- consent.rs (4 routes) ---

    #[tokio::test]
    async fn consent_get_returns_default_config() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/consent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 1);
    }

    #[tokio::test]
    async fn consent_set_invalid_level_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"level": 0});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_set_level_5_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"level": 5});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_whitelist_add_invalid_node_id_400() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"project_id": "not-valid-hex"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn consent_whitelist_add_missing_project_id_422() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn consent_whitelist_remove_missing_project_id_422() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/remove")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // --- files.rs (3 routes) ---

    #[tokio::test]
    async fn files_manifest_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/not-a-valid-sha/manifest")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_manifest_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_stream_invalid_sha_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/files/bad-sha")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn files_stream_not_found_404() {
        let app = build_test_router(mk_state().await);
        let sha = "b".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn files_upload_too_large_413() {
        let app = build_test_router(mk_state().await);
        let big_body = vec![0u8; 50 * 1024 * 1024 + 1];
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "application/octet-stream")
                    .body(axum::body::Body::from(big_body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    // --- canary_api.rs (3 routes) ---

    #[tokio::test]
    async fn canary_freshness_returns_200() {
        let app = build_test_router(mk_state().await);
        let pubkey = "aa".repeat(32);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/canary/freshness/{pubkey}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_freshness_unknown_pubkey_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/freshness/unknown-key")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn canary_inject_rate_updates() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let body = serde_json::json!({"inject_rate": 50});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/inject-rate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let resp_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(resp_body["status"], "updated");
        assert!(resp_body["inject_rate"].as_u64().is_some());
    }

    #[tokio::test]
    async fn canary_observed_divergence_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/canary/observed-divergence")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["divergences"].as_array().unwrap().is_empty());
    }

    // --- contributor_api.rs (2 routes) ---

    #[tokio::test]
    async fn contributor_project_empty_list() {
        let app = build_test_router(mk_state().await);
        let project_id = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/contributor/project/{project_id}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["contributors"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn contributor_project_invalid_hex_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/project/not-a-hex")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn contributor_envelope_not_found_404() {
        let app = build_test_router(mk_state().await);
        let project_id = "a".repeat(64);
        let node_id = "b".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!(
                        "/api/v1/contributor/envelope/{project_id}/{node_id}"
                    ))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn contributor_envelope_invalid_hex_400() {
        let app = build_test_router(mk_state().await);
        let valid = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/contributor/envelope/bad-hex/{valid}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // =============================================================
    // Sprint 46 Phase B — integration tests 14 recent routes + debt
    // =============================================================

    // --- invite_api.rs (3 routes) ---

    #[tokio::test]
    async fn invite_create_success() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"scope": "observer"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/invite/create")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let id = body["id"].as_str().expect("id must be a string");
        assert!(id.starts_with("inv-"), "invite ID must start with inv-");
        let parts: Vec<&str> = id.split('-').collect();
        assert_eq!(parts.len(), 4, "format inv-{{node8}}-{{ts}}-{{seq}}");
        assert_eq!(body["scope"], "observer");
        assert!(
            body["wire"].as_str().unwrap().starts_with("nx1"),
            "wire must be nx1-encoded"
        );
    }

    #[tokio::test]
    async fn invite_worker_requires_project_doc() {
        let app = build_test_router(mk_state().await);
        let body = serde_json::json!({"scope": "worker"});
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/invite/create")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn invite_list_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/invite")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["invites"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn invite_revoke_not_found_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/v1/invite/nonexistent-id")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // --- quarantine_api.rs (3 routes) ---

    #[tokio::test]
    async fn quarantine_list_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/quarantine")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn quarantine_flush_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/flush")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn quarantine_drop_not_found() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/quarantine/99999/drop")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // --- tasks_api.rs (2 routes) ---

    #[tokio::test]
    async fn tasks_list_default() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["tasks"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn tasks_get_not_found_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks/nonexistent-task-id")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // Sprint 72 Phase D: `/{task_id}/result` is 404 while the task is
    // pending and returns the human-readable text once completed — the
    // primitive the Operator network arm polls then fetches.
    #[tokio::test]
    async fn task_result_route_404_then_text_on_completed() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();
        let db_handle = state.coordinator_db.clone();

        let task_id = {
            let db = db_handle.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit")
                .task
                .task_id
        };

        let app = build_test_router(state);

        // Pending → 404 (status carried in the error message).
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/tasks/{task_id}/result"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        // Complete it with a retrievable text.
        {
            let db = db_handle.lock().unwrap();
            db.set_task_result(&task_id, "w1", "sig-hex", "the network reply", 100)
                .expect("complete");
        }

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/tasks/{task_id}/result"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["status"], "completed");
        assert_eq!(body["result_text"], "the network reply");
    }

    // --- kudos_api.rs (2 routes) ---

    #[tokio::test]
    async fn kudos_entries_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["entries"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn kudos_leaderboard_empty() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-test/leaderboard")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
        assert!(body["leaderboard"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn contributor_dashboard_aggregates_node_credits() {
        // Sprint 76 Phase E (D4): the /contributor/{node_id} route returns
        // the node's cross-project standing — mirror of leaderboard but
        // per-node. Credit one node across two projects, then read it back.
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "node-x", "t1", 100, 1_000)
                .unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p2", "node-x", "t2", 50, 1_000)
                .unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "node-y", "t3", 10, 1_000)
                .unwrap();
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/node-x")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_node_id"], "node-x");
        assert_eq!(body["tasks_served"], 2, "node-x served 2 tasks");
        assert!(body["effective_kudos"].as_u64().unwrap() > 0);
        assert_eq!(
            body["per_project"].as_array().unwrap().len(),
            2,
            "node-x served 2 distinct projects"
        );
    }

    #[tokio::test]
    async fn contributor_dashboard_empty_for_unknown_node() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/contributor/nobody")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["tasks_served"], 0);
        assert!(body["per_project"].as_array().unwrap().is_empty());
    }

    // --- health_api.rs (1 route) ---

    #[tokio::test]
    async fn coordinator_health_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/coordinator/health")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["status"], "ok");
        assert_eq!(body["daemon_version"], "0.1.0-test");
    }

    // --- shell_api.rs (1 route) ---

    #[tokio::test]
    async fn shell_discover_returns_self() {
        let state = mk_state().await;
        let expected_node_id = state.node_id.clone();
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/shell/discover")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
        let coordinators = body["coordinators"].as_array().unwrap();
        assert_eq!(coordinators.len(), 1);
        assert_eq!(coordinators[0]["node_id"], expected_node_id);
    }

    // --- diagnostic_api.rs (1 route) ---

    #[tokio::test]
    async fn diagnostic_fairness_ok() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_count"], 0);
    }

    #[tokio::test]
    async fn diagnostic_fairness_ema_on_nonempty_ledger() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w1", "t1", 100, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w2", "t2", 100, 1_000).unwrap();
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["worker_count"], 2);
        let gini = body["gini"].as_f64().unwrap();
        assert!(
            gini < 0.01,
            "two equal-contribution workers must have near-zero Gini (got {gini})"
        );
    }

    // --- worker_state_api.rs (1 route) ---

    #[tokio::test]
    async fn worker_state_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/worker/state")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body.get("running").is_some());
    }

    // --- Pagination tests (debt items 2 + 4) ---

    #[tokio::test]
    async fn kudos_entries_with_limit_offset() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w1", "t1", 10, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w2", "t2", 20, 1_000).unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(&db, "p1", "w3", "t3", 30, 1_000).unwrap();
        }
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=0")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 2);
        assert_eq!(body["total_count"], 3);

        let app2 = build_test_router(Arc::clone(&state));
        let resp2 = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/entries?limit=2&offset=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp2.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body2["count"], 1);
        assert_eq!(body2["total_count"], 3);
    }

    #[tokio::test]
    async fn tasks_list_with_limit() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .unwrap();
        }
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/tasks?limit=1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
    }

    // --- Debt item 5: diagnostic error propagation ---

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_corrupted_db() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            db.execute_batch_raw("DROP TABLE IF EXISTS kudos")
                .expect("drop kudos table");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(body["error"].as_str().unwrap().contains("kudos_entries"));
    }

    #[tokio::test]
    async fn diagnostic_fairness_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(state.coordinator_db.lock().is_err());

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/diagnostic/fairness")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    // --- deploy.rs integration tests (2 routes) ---

    fn make_test_zip() -> Vec<u8> {
        use std::io::Write;
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zw = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zw.start_file("index.html", opts).unwrap();
            zw.write_all(b"<html><body>test</body></html>").unwrap();
            zw.finish().unwrap();
        }
        buf.into_inner()
    }

    #[tokio::test]
    async fn deploy_private_valid_zip_returns_200() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let zip = make_test_zip();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy")
                    .body(axum::body::Body::from(zip))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["deployed"], true);
        assert!(body["hash"].as_str().unwrap().len() >= 32);
    }

    #[tokio::test]
    async fn deploy_private_invalid_zip_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy")
                    .body(axum::body::Body::from(b"not a zip".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn deploy_from_repo_non_http_url_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy-from-repo")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "repo_url": "ssh://git@github.com/test/repo.git",
                            "project_name": "test"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn deploy_from_repo_invalid_sha_returns_400() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/deploy-from-repo")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({
                            "repo_url": "https://github.com/test/repo.git",
                            "project_name": "test",
                            "commit_sha": "not-a-sha"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // --- apps.rs integration tests (2 routes) ---

    #[tokio::test]
    async fn apps_list_empty_returns_200() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 0);
    }

    #[tokio::test]
    async fn apps_list_with_entries_returns_populated() {
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};
        let state = mk_state().await;
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: "a".repeat(64),
            node_id: None,
            project_name: "Test App".into(),
            category: "test".into(),
            description: "A test app".into(),
            curator_pubkey: "b".repeat(64),
            curator_name: "Test Curator".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("c".repeat(64)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["count"], 1);
        assert_eq!(body["apps"][0]["project_name"], "Test App");
    }

    #[tokio::test]
    async fn apps_get_by_id_returns_detail() {
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};
        let state = mk_state().await;
        let pid = "d".repeat(64);
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: pid.clone(),
            node_id: None,
            project_name: "Detail App".into(),
            category: "test".into(),
            description: "Detailed".into(),
            curator_pubkey: "e".repeat(64),
            curator_name: "Curator".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Unknown,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: None,
            repo_url: Some("https://example.com/repo".into()),
            provenance_hash: None,
            is_open_source: true,
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/apps/{pid}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["project_name"], "Detail App");
        assert_eq!(body["is_open_source"], true);
    }

    #[tokio::test]
    async fn apps_get_unknown_id_returns_404() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/apps/nonexistent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // --- auth/token integration test (1 route) ---

    #[tokio::test]
    async fn auth_token_returns_200_from_loopback() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/auth/token")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["token"], TEST_TOKEN);
    }

    // --- consent.rs happy path tests (4 routes) ---

    #[tokio::test]
    async fn consent_set_level_2_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"level": 2}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 2);
    }

    #[tokio::test]
    async fn consent_get_returns_persisted_level() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/set")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"level": 3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/consent")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["level"], 3);
    }

    #[tokio::test]
    async fn consent_whitelist_add_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let pid = "a".repeat(64);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/add")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            body["allowed_project_ids"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str() == Some(&pid))
        );
    }

    #[tokio::test]
    async fn consent_whitelist_remove_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let pid = "b".repeat(64);

        let app1 = build_test_router(Arc::clone(&state));
        app1.oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/consent/whitelist/add")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    serde_json::json!({"project_id": pid}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/consent/whitelist/remove")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"project_id": pid}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert!(
            body["allowed_project_ids"]
                .as_array()
                .unwrap()
                .iter()
                .all(|v| v.as_str() != Some(&pid))
        );
    }

    // --- files.rs happy path tests (3 routes) ---

    #[tokio::test]
    async fn files_upload_small_returns_201_with_sha() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .header("x-original-name", "test.txt")
                    .body(axum::body::Body::from(b"hello world".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap().len(), 64);
        assert_eq!(body["size"], 11);
        assert_eq!(body["original_name"], "test.txt");
    }

    #[tokio::test]
    async fn files_manifest_after_upload_returns_200() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .header("content-type", "text/plain")
                    .body(axum::body::Body::from(b"manifest test".to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}/manifest"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["sha256"].as_str().unwrap(), sha);
    }

    #[tokio::test]
    async fn files_stream_after_upload_returns_content() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let state = mk_state_with_sbfb_home(tmp.path().to_path_buf()).await;
        let content = b"stream test content";

        let app1 = build_test_router(Arc::clone(&state));
        let resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/files/upload")
                    .body(axum::body::Body::from(content.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let upload_body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        let sha = upload_body["sha256"].as_str().unwrap().to_owned();

        let app2 = build_test_router(Arc::clone(&state));
        let resp = app2
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/v1/files/{sha}"))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body_bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert_eq!(&body_bytes[..], content);
    }

    #[tokio::test]
    async fn storage_join_rejects_non_replicated_app() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/storage/join")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_string(&serde_json::json!({
                            "app": "unknown-app",
                            "ticket": "placeholder"
                        }))
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::BAD_REQUEST,
            "non-replicated app must be rejected with 400"
        );
        let body_bytes = to_bytes(resp.into_body(), 4096).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        let err = body["error"].as_str().unwrap();
        assert!(
            err.contains("not a replicated app"),
            "error must identify the replicated-app guard, got: {err}"
        );
    }

    #[tokio::test]
    async fn storage_set_rate_limited_returns_429() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        for i in 0..15 {
            let resp = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/app/test-app/state/key1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(
                            serde_json::to_string(&serde_json::json!({ "v": i })).unwrap(),
                        ))
                        .unwrap(),
                )
                .await
                .unwrap();
            if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                return;
            }
        }
        panic!("expected at least one 429 TOO_MANY_REQUESTS after 15 rapid writes");
    }

    #[tokio::test]
    async fn provenance_endpoint_absent_status() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/project/nonexistent/provenance")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "absent");
        assert_eq!(json["verified"], false);
        assert!(json["record"].is_null());
        assert!(json["provenance_hash"].is_null());
    }

    #[tokio::test]
    async fn provenance_endpoint_found_and_verified() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let kp = &state.pow_keypair;
        let record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            kp,
        );
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], true);
        assert_eq!(json["status"], "verified");
        assert_eq!(json["record"]["repo_url"], "https://github.com/user/repo");
        assert_eq!(json["record"]["artifact_hash"], "deadbeef");
        assert_eq!(json["record"]["schema_version"], 1);
        assert!(
            json["provenance_hash"].as_str().is_some(),
            "response must include provenance_hash"
        );
        assert_eq!(json["provenance_hash"].as_str().unwrap().len(), 64);
    }

    #[tokio::test]
    async fn provenance_cross_node_verified() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let other_kp = KeyPair::generate();
        let record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/other/repo",
            "abc123def456abc123def456abc123def456abc1",
            "cafebabe",
            &hex::encode(other_kp.public_bytes()),
            &other_kp,
        );
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], true);
        assert_eq!(json["status"], "verified");
        assert!(json["record"]["repo_url"].as_str().is_some());
    }

    #[tokio::test]
    async fn provenance_cross_node_tampered() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let signer_kp = KeyPair::generate();
        let mut record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/tampered/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(signer_kp.public_bytes()),
            &signer_kp,
        );
        let impostor_kp = KeyPair::generate();
        record.node_id = hex::encode(impostor_kp.public_bytes());
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["verified"], false);
        assert_eq!(json["status"], "failed");
    }

    #[tokio::test]
    async fn provenance_endpoint_returns_app_version() {
        let state = mk_state().await;
        let project_id = state.node_id.clone();
        let kp = &state.pow_keypair;
        let mut record = nexus_coordinator_rs::provenance::generate_provenance(
            "https://github.com/user/versioned",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            kp,
        );
        record.app_version = Some("3.2.1".to_string());
        {
            let db = state.coordinator_db.lock().unwrap();
            db.insert_provenance_record(&project_id, &record)
                .expect("insert");
        }

        let app = build_test_router(state);
        let uri = format!("/api/v1/project/{project_id}/provenance");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["record"]["app_version"], "3.2.1");
    }

    // -- Sprint 63 Phase C: feed cursor endpoint tests --

    #[tokio::test]
    async fn feed_cursor_empty_returns_zero() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/cursor")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["last_seq"], 0);
        assert!(json["last_entry_hash"].is_null());
    }

    #[tokio::test]
    async fn feed_cursor_returns_saved_position() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            db.save_feed_cursor(42, "abcdef1234567890").expect("save");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/cursor")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["last_seq"], 42);
        assert_eq!(json["last_entry_hash"], "abcdef1234567890");
    }

    // -- Sprint 67 Phase A: feed entries endpoint tests --

    fn insert_test_feed_entry(
        db: &nexus_coordinator_rs::db::CoordinatorDb,
        project_id: &str,
        op_type_str: &str,
    ) {
        let kp = nexus_core_rs::KeyPair::from_secret_bytes(&[42u8; 32]);
        let pk = hex::encode(kp.public_bytes());
        let op = serde_json::json!({
            "op_type": op_type_str,
            "project_id": project_id,
            "repo_url": "https://github.com/org/app",
            "commit_sha": "a".repeat(40),
            "artifact_hash": "b".repeat(64),
            "provenance_hash": "c".repeat(64),
            "is_open_source": true
        });
        nexus_coordinator_rs::public_feed::insert_feed_operation(db, op, &pk, |d| {
            kp.sign(d).to_vec()
        })
        .unwrap();
    }

    #[tokio::test]
    async fn test_feed_entries_endpoint_paginated() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            let pid = "a1".repeat(32);
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
            insert_test_feed_entry(&db, &pid, "ReleasePublished");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/feed/entries?after_seq=1&limit=2")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 2);
        let entries = json["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(entries[0]["seq"].as_u64().unwrap() > 1);
    }

    #[tokio::test]
    async fn test_feed_entries_endpoint_filters_by_project_id() {
        let state = mk_state().await;
        let pid_a = "a1".repeat(32);
        let pid_b = "b2".repeat(32);
        {
            let db = state.coordinator_db.lock().unwrap();
            insert_test_feed_entry(&db, &pid_a, "ReleasePublished");
            insert_test_feed_entry(&db, &pid_b, "ReleasePublished");
            insert_test_feed_entry(&db, &pid_a, "ReleasePublished");
        }
        let app = build_test_router(state);
        let uri = format!("/api/daemon/feed/entries?project_id={pid_a}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["count"], 2);
        let entries = json["entries"].as_array().unwrap();
        for e in entries {
            assert_eq!(e["payload"]["project_id"].as_str().unwrap(), pid_a);
        }
    }

    // -- Sprint 67 Phase B: search endpoint test --

    #[tokio::test]
    async fn test_search_endpoint_http() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-search",
                "Babel Translator",
                "translation",
                "A real-time translation tool",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance::default(),
            )
            .expect("index");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=translation")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total"], 1);
        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["project_name"], "Babel Translator");
        assert!(json["took_ms"].as_u64().is_some());
    }

    // -- Search hotfix (Sprint 73 audit): real publish->search boundary --

    /// E2E at the REAL boundary (no mockFetch, no test-only `index_entry`
    /// injection): a project published through the production `POST
    /// /api/daemon/publish` handler MUST become findable through the production
    /// `GET /api/daemon/search` handler. This crosses the deploy/publish ->
    /// FTS5-index seam that every prior test mocked or bypassed, which is why a
    /// fully broken search shipped with green tests. Asserts the three facets of
    /// the hotfix at once: (1) the app is indexed on publish, (2) PREFIX search
    /// ("Bab" -> "Babel") works, (3) re-publish dedups instead of duplicating.
    #[tokio::test]
    async fn publish_makes_app_searchable_by_name() {
        let state = mk_state().await;

        async fn do_publish(router: Router) -> StatusCode {
            let body = serde_json::json!({
                "project_name": "Babel Translator",
                "category": "translation",
                "description": "real-time peer to peer translation",
            });
            router
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/daemon/publish")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
        }

        async fn do_search(router: Router, q: &str) -> serde_json::Value {
            let resp = router
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri(format!("/api/daemon/search?q={q}"))
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let bytes = to_bytes(resp.into_body(), 16384).await.unwrap();
            serde_json::from_slice(&bytes).unwrap()
        }

        // Publish through the real handler (shares coordinator_db across routers).
        assert_eq!(
            do_publish(build_test_router(state.clone())).await,
            StatusCode::OK
        );

        // (1) exact-name search finds it.
        let json = do_search(build_test_router(state.clone()), "Babel").await;
        assert_eq!(
            json["total"], 1,
            "publish must make the app searchable by name"
        );
        assert_eq!(json["results"][0]["project_name"], "Babel Translator");

        // (2) prefix search ("Bab") finds "Babel".
        let json = do_search(build_test_router(state.clone()), "Bab").await;
        assert_eq!(json["total"], 1, "prefix search must find the app");

        // (3) re-publishing the same project dedups (deterministic browse rowid).
        assert_eq!(
            do_publish(build_test_router(state.clone())).await,
            StatusCode::OK
        );
        let json = do_search(build_test_router(state.clone()), "Babel").await;
        assert_eq!(
            json["total"], 1,
            "re-publish must not duplicate the index row"
        );
    }

    /// Publish an app through the real `POST /api/daemon/publish` handler.
    async fn publish_app(
        state: &Arc<DaemonHttpState>,
        name: &str,
        category: &str,
        desc: &str,
    ) -> StatusCode {
        let body = serde_json::json!({
            "project_name": name,
            "category": category,
            "description": desc,
        });
        build_test_router(Arc::clone(state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/publish")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    /// Search through the real `GET /api/daemon/search` handler; returns `total`.
    async fn search_total(state: &Arc<DaemonHttpState>, q: &str) -> u64 {
        let uri = format!("/api/daemon/search?q={}", q.replace(' ', "%20"));
        let resp = build_test_router(Arc::clone(state))
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        json["total"].as_u64().unwrap()
    }

    #[tokio::test]
    async fn published_app_searchable_by_category() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Quiet Name", "translation", "x").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "translation").await,
            1,
            "find by category"
        );
        assert_eq!(search_total(&state, "transl").await, 1, "category prefix");
    }

    #[tokio::test]
    async fn published_app_searchable_by_single_letter() {
        // The exact user symptom, end-to-end through the real handlers: a
        // published "sbfb-*" app must be found by typing the single letter "s".
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "sbfb-explorer", "tools", "protocol explorer").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "s").await,
            1,
            "single-letter 's' finds it"
        );
        assert_eq!(
            search_total(&state, "explor").await,
            1,
            "inner-token prefix finds it"
        );
    }

    #[tokio::test]
    async fn published_app_searchable_by_description_word() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Plain", "misc", "end to end encryption demo").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "encryption").await,
            1,
            "find by description word"
        );
        assert_eq!(
            search_total(&state, "encrypt").await,
            1,
            "description prefix"
        );
    }

    #[tokio::test]
    async fn published_app_searchable_by_multi_word_query() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Babel Translator", "translation", "fast").await,
            StatusCode::OK
        );
        // Multi-word query (space URL-encoded): all terms must match (AND).
        assert_eq!(
            search_total(&state, "babel translator").await,
            1,
            "multi-word AND matches"
        );
        assert_eq!(
            search_total(&state, "nomatch translator").await,
            0,
            "a missing term yields no match"
        );
    }

    /// GET /api/daemon/browse and return the entries array.
    async fn browse_entries(state: &Arc<DaemonHttpState>) -> Vec<serde_json::Value> {
        let resp = build_test_router(Arc::clone(state))
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/browse")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 65536).await.unwrap()).unwrap();
        json["entries"].as_array().cloned().unwrap_or_default()
    }

    #[tokio::test]
    async fn multiple_apps_get_distinct_browse_cards() {
        // One node hosting two apps must show TWO distinct Browse cards, keyed by
        // per-app project_id (blake3(name)). Before the fix both took the node_id
        // as project_id and the second collapsed onto the first (single card).
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "App One", "tools", "first").await,
            StatusCode::OK
        );
        assert_eq!(
            publish_app(&state, "App Two", "tools", "second").await,
            StatusCode::OK
        );

        let entries = browse_entries(&state).await;
        let ids: std::collections::HashSet<&str> = entries
            .iter()
            .filter_map(|e| e["project_id"].as_str())
            .collect();
        assert_eq!(
            ids.len(),
            2,
            "two apps -> two distinct cards, not collapsed"
        );
        // Each is individually searchable.
        assert_eq!(search_total(&state, "One").await, 1);
        assert_eq!(search_total(&state, "Two").await, 1);
    }

    #[tokio::test]
    async fn published_app_browse_id_is_blake3_not_node_id() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Identity Test", "tools", "x").await,
            StatusCode::OK
        );
        let expected = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Identity Test"));
        let entries = browse_entries(&state).await;
        let entry = entries
            .iter()
            .find(|e| e["project_name"] == "Identity Test")
            .expect("published app present in browse");
        assert_eq!(
            entry["project_id"].as_str().unwrap(),
            expected,
            "browse card id is blake3(project_name)"
        );
        assert_ne!(
            entry["project_id"].as_str().unwrap(),
            state.node_id,
            "browse card id is NOT the node_id"
        );
    }

    // -- Sprint 74 Phase C: atelier-fork redeploy under local identity --

    /// POST a forked workspace's zip to `/api/v1/deploy-workspace` and return the
    /// HTTP status. Mirrors `publish_app` but for the local-redeploy path.
    async fn deploy_workspace_app(
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

    #[tokio::test]
    async fn fork_redeploy_resigns_provenance_as_local_node() {
        // A locally forked/edited workspace, redeployed through
        // /deploy-workspace, gets a FRESH provenance signed by THIS node's
        // keypair (R5: a fork is a new LOCAL author act; the original author's
        // provenance is never inherited). No mock: real state, real provenance
        // signing, real DB round-trip.
        let state = mk_state().await;
        let zip = make_zip(&[("index.html", b"<h1>my fork</h1>")]);
        assert_eq!(
            deploy_workspace_app(&state, "Forked App", zip).await,
            StatusCode::OK
        );
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Forked App"));
        let record = {
            let db = state.coordinator_db.lock().unwrap();
            db.get_provenance_by_project(&pid)
                .unwrap()
                .expect("provenance recorded for the redeployed fork")
        };
        // Re-signed under THIS node's identity, not the original author's.
        assert_eq!(
            record.node_id, state.node_id,
            "provenance node_id is the local node"
        );
        // The signature genuinely verifies against the local signing keypair —
        // independent of any node_id/pow-key alignment.
        let json = nexus_coordinator_rs::provenance::provenance_to_json(&record);
        assert!(
            nexus_coordinator_rs::provenance::verify_provenance(
                &json,
                &state.pow_keypair.public_bytes()
            ),
            "provenance is signed by the local node's keypair"
        );
    }

    #[tokio::test]
    async fn fork_redeploy_loop_e2e_single_node() {
        // The full atelier loop on a single node: a forked workspace's bytes go
        // in via /deploy-workspace and come out as a discoverable Browse card
        // with a per-app id and an HONEST is_open_source=false (local
        // self-attestation, not a verifiable public build). Real frontier
        // (§P57): real HTTP handler, real blob store, real aggregator + search.
        let state = mk_state().await;
        let zip = make_zip(&[("index.html", b"<h1>loop</h1>")]);
        assert_eq!(
            deploy_workspace_app(&state, "Loop Fork", zip).await,
            StatusCode::OK
        );
        let entries = browse_entries(&state).await;
        let entry = entries
            .iter()
            .find(|e| e["project_name"] == "Loop Fork")
            .expect("redeployed fork present in browse");
        let expected = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Loop Fork"));
        assert_eq!(
            entry["project_id"].as_str().unwrap(),
            expected,
            "per-app project_id"
        );
        assert_eq!(
            entry["is_open_source"].as_bool(),
            Some(false),
            "a local fork redeploy is a self-attestation, never open-source"
        );
        // Findable by name through the real search handler.
        assert_eq!(search_total(&state, "Loop").await, 1);
    }

    #[tokio::test]
    async fn deploy_per_app_distinct_browse_cards() {
        // OFF-SPRINT-2 regression: two workspace redeploys on ONE node yield TWO
        // distinct Browse cards keyed by per-app blake3(name), never collapsed
        // onto the node_id.
        let state = mk_state().await;
        assert_eq!(
            deploy_workspace_app(&state, "Fork Alpha", make_zip(&[("index.html", b"a")])).await,
            StatusCode::OK
        );
        assert_eq!(
            deploy_workspace_app(&state, "Fork Beta", make_zip(&[("index.html", b"b")])).await,
            StatusCode::OK
        );
        let entries = browse_entries(&state).await;
        let ids: std::collections::HashSet<&str> = entries
            .iter()
            .filter_map(|e| e["project_id"].as_str())
            .collect();
        assert_eq!(ids.len(), 2, "two forks -> two distinct cards");
        assert!(
            !ids.contains(state.node_id.as_str()),
            "no card keyed by node_id"
        );
    }

    #[tokio::test]
    async fn publish_and_gossip_use_per_app_project_id() {
        // OFF-SPRINT-2b regression: the gossip ProjectAnnouncement carries the
        // per-app project_id (blake3(name)), distinct from the hosting node_id.
        // The node_id stays on the wire as the dialable identity, but the app
        // identity is per-app. Captures the real outbox envelope (no mock).
        use nexus_core_rs::crypto::blake3_hash;
        let (tx, mut rx) = tokio::sync::mpsc::channel::<crate::runtime::GossipCmd>(8);
        let state = mk_state_with_mode_tx(nexus_core_rs::IdentityMode::Normal, tx).await;
        let pid = hex::encode(blake3_hash(b"Per App Gossip"));
        crate::deploy::publish_announcement(
            &state,
            crate::deploy::AnnouncementParams {
                project_id: &pid,
                project_name: "Per App Gossip",
                category: "tools",
                description: "x",
                apps: &[],
                archive_hash: None,
                repo_url: None,
                provenance_hash: None,
                is_open_source: false,
            },
        )
        .await;
        let cmd = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("outbox arrives")
            .expect("channel open");
        let crate::runtime::GossipCmd::Outbox(payload) = cmd else {
            panic!("expected GossipCmd::Outbox");
        };
        // Sprint 75 Phase A: the outbox carries the UNWRAPPED announcement payload
        // (each replay re-mints + re-stamps), so it parses directly — no PoW unwrap.
        let ann =
            nexus_shell_daemon_core::publish::ProjectAnnouncement::from_gossip_bytes(&payload)
                .unwrap();
        assert_eq!(ann.project_id, pid, "per-app id on the wire");
        assert_ne!(ann.project_id, ann.node_id, "per-app id is not the node_id");
    }

    /// POST a raw query string + zip body to /api/v1/deploy-workspace; return the
    /// HTTP status (for the validation-branch + body-limit tests).
    async fn post_workspace(state: &Arc<DaemonHttpState>, query: &str, zip: Vec<u8>) -> StatusCode {
        build_test_router(Arc::clone(state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri(format!("/api/v1/deploy-workspace?{query}"))
                    .body(axum::body::Body::from(zip))
                    .unwrap(),
            )
            .await
            .unwrap()
            .status()
    }

    #[tokio::test]
    async fn deploy_workspace_with_lineage_stays_not_open_source() {
        // The frozen invariant: a fork redeploy is is_open_source=false EVEN when
        // it carries a valid https lineage repo_url + 40-hex commit_sha. A
        // regression wiring lineage -> is_open_source=true (the L2-consent / R5
        // escalation) must fail this test. Attribution (repo_url) is kept; the
        // open-source standing is denied.
        let state = mk_state().await;
        let zip = make_zip(&[("index.html", b"<h1>x</h1>")]);
        let query = format!(
            "project_name=Lineage%20Fork&category=tools&description=d&repo_url={}&commit_sha={}",
            "https%3A%2F%2Fcodeberg.org%2Forig%2Fapp.git",
            "a".repeat(40)
        );
        assert_eq!(post_workspace(&state, &query, zip).await, StatusCode::OK);

        let entries = browse_entries(&state).await;
        let entry = entries
            .iter()
            .find(|e| e["project_name"] == "Lineage Fork")
            .expect("redeployed fork present");
        assert_eq!(
            entry["is_open_source"].as_bool(),
            Some(false),
            "lineage repo_url must NOT grant open-source standing"
        );
        assert!(
            entry["repo_url"]
                .as_str()
                .unwrap_or_default()
                .starts_with("https://codeberg.org/orig/app"),
            "lineage repo_url IS recorded for attribution"
        );
    }

    #[tokio::test]
    async fn deploy_workspace_rejects_bad_inputs() {
        // The five new validation branches (mirrors deploy_from_repo's 400 tests).
        let state = mk_state().await;
        let zip = || make_zip(&[("index.html", b"<h1>x</h1>")]);
        assert_eq!(
            post_workspace(&state, "project_name=&category=tools", zip()).await,
            StatusCode::BAD_REQUEST,
            "empty project_name"
        );
        assert_eq!(
            post_workspace(
                &state,
                "project_name=A&repo_url=http%3A%2F%2Fx.example",
                zip()
            )
            .await,
            StatusCode::BAD_REQUEST,
            "non-https lineage repo_url"
        );
        assert_eq!(
            post_workspace(&state, "project_name=A&commit_sha=notasha", zip()).await,
            StatusCode::BAD_REQUEST,
            "invalid commit_sha"
        );
        assert_eq!(
            post_workspace(&state, "project_name=A", make_zip(&[("other.html", b"x")])).await,
            StatusCode::BAD_REQUEST,
            "zip without index.html"
        );
    }

    #[tokio::test]
    async fn deploy_workspace_accepts_body_over_2mb() {
        // Regression for the per-route DefaultBodyLimit override: axum's 2 MB
        // default would 413 a realistic forked workspace before the handler runs.
        // make_zip uses Stored (no compression) so the body is genuinely >2 MB.
        let state = mk_state().await;
        let filler = vec![b'A'; 3 * 1024 * 1024];
        let zip = make_zip(&[("index.html", b"<h1>big</h1>"), ("blob.bin", &filler)]);
        assert!(
            zip.len() as u64 > 2 * 1024 * 1024,
            "test body must exceed axum's 2MB default ({} bytes)",
            zip.len()
        );
        assert_eq!(
            post_workspace(&state, "project_name=Big%20Fork&category=tools", zip).await,
            StatusCode::OK,
            "a >2MB workspace must pass the body limit (handler ceiling is 100MB)"
        );
    }

    #[tokio::test]
    async fn finalize_deploy_open_source_arm_propagates_version_and_flag() {
        // Protects deploy_from_repo's is_open_source=true path (untestable over
        // HTTP without a network clone) after the finalize_deploy extraction:
        // is_open_source propagates to the Browse card and app_version + commit
        // propagate to the signed provenance record.
        let state = mk_state().await;
        let zip = make_zip(&[("index.html", b"<h1>os</h1>")]);
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"OS App"));
        let sha = "b".repeat(40);
        crate::deploy::finalize_deploy(
            &state,
            &pid,
            zip,
            crate::deploy::FinalizeDeployParams {
                project_name: "OS App",
                category: "tools",
                description: "d",
                apps: &[],
                repo_url: Some("https://codeberg.org/os/app.git"),
                commit_sha: &sha,
                is_open_source: true,
                app_version: Some("2.1.0".to_string()),
            },
        )
        .await
        .expect("finalize_deploy open-source arm");

        let entries = browse_entries(&state).await;
        let entry = entries
            .iter()
            .find(|e| e["project_name"] == "OS App")
            .expect("verified deploy present");
        assert_eq!(entry["is_open_source"].as_bool(), Some(true));

        let record = {
            let db = state.coordinator_db.lock().unwrap();
            db.get_provenance_by_project(&pid).unwrap().unwrap()
        };
        assert_eq!(record.app_version.as_deref(), Some("2.1.0"));
        assert_eq!(record.commit_sha, sha);
    }

    // -- Sprint 74 Phase D: keep-online local pin --

    /// True iff a tag with the exact `name` exists in the blob store.
    async fn has_tag(state: &Arc<DaemonHttpState>, name: &str) -> bool {
        use futures_lite::StreamExt;
        let store = state.node.blobs_store();
        let mut stream = store
            .tags()
            .list_prefix(name.as_bytes())
            .await
            .expect("list tags");
        stream.next().await.is_some()
    }

    #[tokio::test]
    async fn voluntary_seed_distant_public_app_no_approval() {
        // Sprint 74 Phase E (amendement PO §13): a node may VOLUNTARILY keep a
        // DISTANT public app online — fetch+pin its archive + record keep_online —
        // with NO SeedRequest, NO invite, NO author approval (the content is
        // public + content-addressed). This test also covers
        // `voluntary_seeder_serves_author_provenance_intact`: the seeder ends up
        // with the AUTHOR's exact bytes (it re-signs no provenance). Real
        // frontier: a 2nd iroh node hosts the blob, the route fetches it P2P.
        use nexus_shell_daemon_core::browse::{BrowseEntry, BrowseSource, BrowseStatus};

        // A distant node hosts the public app archive and mints a ticket.
        let remote = create_node().await.expect("remote node");
        let blobs_r = nexus_core_rs::BlobsClient::new(remote.blobs_store());
        let payload = b"distant-public-app-author-signed-bytes".to_vec();
        let hash = blobs_r.add_bytes(&payload).await.unwrap();
        let r_addr = nexus_core_rs::discovery::DiscoveryClient::new(remote.endpoint())
            .my_endpoint_addr()
            .await
            .expect("remote addr");
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            r_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        // The local node (the seeder) learned the app via gossip → a direct
        // browse entry carrying the ticket + hash.
        let state = mk_state().await;
        let pid = "distant-public-app";
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: pid.to_string(),
            node_id: Some(remote.node_id()),
            project_name: "Distant App".into(),
            category: "demo".into(),
            description: "a public app".into(),
            curator_pubkey: String::new(),
            curator_name: "Distant".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: Some(ticket),
            archive_hash: Some(hex::encode(hash)),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        // Voluntary seed via the real route — the body is ONLY the project_id:
        // no invite, no token, no approval anywhere in the request.
        let body = serde_json::json!({"project_id": pid});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/seed")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        // The seeder fetched + pinned the blob and recorded keep_online — with no
        // approval step.
        let blobs_local = nexus_core_rs::BlobsClient::new(state.node.blobs_store());
        assert!(
            blobs_local.has(hash).await.unwrap(),
            "the seeder fetched the distant blob"
        );
        assert!(
            has_tag(&state, &crate::deploy::keep_online_tag(pid)).await,
            "the seeder pinned the blob (skip-GC)"
        );
        assert_eq!(
            state
                .coordinator_db
                .lock()
                .unwrap()
                .get_keep_online(pid)
                .unwrap(),
            Some((true, Some(hex::encode(hash))))
        );
        // Provenance intact: the seeder serves the AUTHOR's exact bytes.
        assert_eq!(
            blobs_local.get_bytes(hash).await.unwrap(),
            payload,
            "the seeder serves the author's exact bytes (no re-provenance)"
        );

        remote.shutdown().await.ok();
    }

    #[tokio::test]
    async fn keep_online_off_removes_tag() {
        // OFF removes ONLY this app's per-intent keep-online tag
        // (keep-online/<project_id>); a sibling app's pin survives. Per-intent
        // keying is exactly what makes a shared archive blob safe to unpin per
        // app (preflight S3). Real frontier: real route + real blob-store tags.
        let state = mk_state().await;
        assert_eq!(
            deploy_workspace_app(&state, "Pin A", make_zip(&[("index.html", b"a")])).await,
            StatusCode::OK
        );
        assert_eq!(
            deploy_workspace_app(&state, "Pin B", make_zip(&[("index.html", b"b")])).await,
            StatusCode::OK
        );
        let pid_a = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Pin A"));
        let pid_b = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Pin B"));
        let tag_a = crate::deploy::keep_online_tag(&pid_a);
        let tag_b = crate::deploy::keep_online_tag(&pid_b);
        assert!(has_tag(&state, &tag_a).await, "A pinned at deploy");
        assert!(has_tag(&state, &tag_b).await, "B pinned at deploy");

        // Turn A OFF via the real route.
        let body = serde_json::json!({"project_id": pid_a, "enabled": false});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/keep-online")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        assert!(
            !has_tag(&state, &tag_a).await,
            "A's keep-online tag removed"
        );
        assert!(
            has_tag(&state, &tag_b).await,
            "B's pin (different intent) survives"
        );

        // Deploy wrote keep_online=true rows (review P2: pin the deploy-time DB
        // write + recorded archive_hash, not just the tag).
        {
            let db = state.coordinator_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid_a).unwrap().map(|(e, _)| e),
                Some(false)
            );
            assert!(
                matches!(db.get_keep_online(&pid_b).unwrap(), Some((true, Some(_)))),
                "B still ON with a recorded archive_hash"
            );
            assert_eq!(db.list_keep_online_disabled().unwrap(), vec![pid_a.clone()]);
        }

        // Turn A back ON via the route (the "remettre en ligne" cycle) — the ON
        // arm must re-resolve archive_hash and re-pin the blob (review P2:
        // ON->re-tag path was otherwise untested).
        let on_body = serde_json::json!({"project_id": pid_a, "enabled": true});
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/keep-online")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&on_body).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        assert!(has_tag(&state, &tag_a).await, "A re-pinned after toggle ON");
        {
            let db = state.coordinator_db.lock().unwrap();
            assert_eq!(
                db.get_keep_online(&pid_a).unwrap().map(|(e, _)| e),
                Some(true)
            );
            assert!(db.list_keep_online_disabled().unwrap().is_empty());
        }
    }

    // -- Sprint 73 Phase D: search JSON carries the provenance triplet --

    #[tokio::test]
    async fn search_handler_json_includes_triplet() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-fork",
                "Forkable App",
                "tools",
                "an app a search hit can fork",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance {
                    repo_url: Some("https://github.com/test/forkable"),
                    commit_sha: Some("abc1230000000000000000000000000000000000"),
                    archive_hash: Some(
                        "dd00000000000000000000000000000000000000000000000000000000000000",
                    ),
                    provenance_hash: Some(
                        "ee00000000000000000000000000000000000000000000000000000000000000",
                    ),
                    is_open_source: true,
                },
            )
            .expect("index");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=forkable")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total"], 1);
        let hit = &json["results"].as_array().unwrap()[0];
        // The four additive provenance keys (+ open-source flag) are present
        // and populated so the S74 atelier can fork from a search hit.
        assert_eq!(hit["repo_url"], "https://github.com/test/forkable");
        assert_eq!(
            hit["commit_sha"],
            "abc1230000000000000000000000000000000000"
        );
        assert_eq!(
            hit["archive_hash"],
            "dd00000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            hit["provenance_hash"],
            "ee00000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(hit["is_open_source"], true);
    }

    #[tokio::test]
    async fn search_clamps_offset_and_query() {
        // CARRY-5 (S74 audit, Sprint 75 Phase G): `offset` and `q` are
        // attacker-supplied query params; only `limit` was clamped before.
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-clamp",
                "Clampable App",
                "tools",
                "an app for the clamp test",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance::default(),
            )
            .expect("index");
        }

        // (a) offset = usize::MAX. Unclamped, `usize::MAX as i64` flips to -1
        // and SQLite treats a negative OFFSET as zero — the row would come
        // BACK. Clamped to MAX_SEARCH_OFFSET (way past the 1-row match set),
        // the page is defined and empty while `total` still counts the match.
        let resp = build_test_router(state.clone())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/daemon/search?q=clampable&offset={}", usize::MAX).as_str())
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(json["total"], 1, "total still counts the match");
        assert_eq!(
            json["results"].as_array().unwrap().len(),
            0,
            "a huge offset must be clamped, not wrap negative and return rows"
        );

        // (b) q far beyond MAX_SEARCH_QUERY_BYTES, with a multi-byte char
        // straddling the 1024-byte cut: a naive byte slice would panic
        // mid-char (500); the boundary-safe truncation must answer 200.
        // `%C3%A9` percent-encodes "é" (2 bytes once decoded), so the
        // decoded q is 1023 ASCII bytes then 2000 two-byte chars — the cut
        // at byte 1024 falls mid-"é".
        let big_q = format!(
            "{}{}",
            "x".repeat(MAX_SEARCH_QUERY_BYTES - 1),
            "%C3%A9".repeat(2_000)
        );
        let resp = build_test_router(state.clone())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/daemon/search?q={big_q}").as_str())
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "an oversized q must be truncated UTF-8-safely, never error"
        );

        // (c) sanity: a normal query still finds the row.
        let resp = build_test_router(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=clampable")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(json["results"].as_array().unwrap().len(), 1);
    }

    // ---------------------------------------------------------
    // Sprint 68 Phase A — ProofCard endpoint
    // ---------------------------------------------------------

    #[tokio::test]
    async fn test_proof_card_endpoint_http() {
        use nexus_shell_daemon_core::browse::{BrowseSource, BrowseStatus};
        let state = mk_state().await;
        let project_id = "f".repeat(64);

        // Seed a direct browse entry so the handler finds metadata.
        state.browse_aggregator.add_direct_entry(BrowseEntry {
            project_id: project_id.clone(),
            node_id: None,
            project_name: "test-app".into(),
            category: "tools".into(),
            description: "a test app".into(),
            curator_pubkey: String::new(),
            curator_name: "Self-published".into(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("deadbeef".into()),
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        });

        let app = build_test_router(state);
        let uri = format!("/api/daemon/proof-card/{project_id}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["project_id"], project_id);
        assert_eq!(json["project_name"], "test-app");
        assert_eq!(json["formula_version"], 1);
        assert_eq!(json["confidence"], 35);
    }

    #[tokio::test]
    async fn test_proof_card_endpoint_not_found() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let unknown_id = "0".repeat(64);
        let uri = format!("/api/daemon/proof-card/{unknown_id}");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    // =============================================================
    // Sprint 68 Phase B — preview load tests
    // =============================================================

    #[tokio::test]
    async fn test_preview_load_returns_hash() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let zip_bytes = make_test_zip();

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/preview/load")
                    .header("Content-Type", "application/octet-stream")
                    .body(axum::body::Body::from(zip_bytes))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let hash = json["hash"].as_str().unwrap();
        assert_eq!(hash.len(), 64, "BLAKE3 hash should be 64 hex chars");
    }

    #[tokio::test]
    async fn test_preview_blob_serve_accessible() {
        let state = mk_state().await;
        let zip_bytes = make_test_zip();
        let hash = state.preview_store.load(zip_bytes).unwrap();

        let app = build_test_router(state);
        let uri = format!("/blob-serve/{hash}/index.html");
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(&uri)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 4096).await.unwrap();
        assert!(
            std::str::from_utf8(&body)
                .unwrap()
                .contains("<body>test</body>")
        );
    }

    #[tokio::test]
    async fn test_preview_eviction_after_ttl() {
        use nexus_shell_daemon_core::preview::PreviewStore;
        let store = PreviewStore::new(std::time::Duration::from_millis(1));
        let data = b"ephemeral zip".to_vec();
        let hash = store.load(data).unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        store.evict_expired();
        assert!(!store.has(&hash));
    }

    #[tokio::test]
    async fn test_preview_max_size_rejected() {
        let state = mk_state().await;
        let app = build_test_router(state);
        let oversized = vec![0u8; nexus_shell_daemon_core::preview::MAX_PREVIEW_BYTES + 1];

        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/preview/load")
                    .header("Content-Type", "application/octet-stream")
                    .body(axum::body::Body::from(oversized))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}
