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
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Json},
    routing::{delete, get, post},
};
use nexus_core_rs::{KeyPair, Node, PowEnvelope, PowSolveCache, RelayPowPolicy, TopicSender};
use nexus_shell_daemon_core::auth::{AuthState, auth_required};
use nexus_shell_daemon_core::blob_serve::BlobServeCache;
use nexus_shell_daemon_core::browse::{BrowseAggregatorHandle, BrowseEntry};
use nexus_shell_daemon_core::iroh_runtime::{CuratorRuntimeError, CuratorRuntimeHandle};
use nexus_shell_daemon_core::state::{DaemonStateSnapshot, StateInputs};
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tracing::debug;

#[cfg(test)]
use serde::Deserialize;

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
    /// Sprint 81 Phase I: in-memory registry of mounted shard sessions
    /// (the live store the S77 Phase J `live_shard_session` stub was the
    /// seam for). Inserts are gated on the `DOMAIN_SHARD_PLAN_V1`
    /// signature + `is_member` checks BEFORE insert; the projections stay
    /// privacy-whitelisted (SI-3/SI-4).
    pub shard_sessions: Arc<crate::shard_session::ShardSessionRegistry>,
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
        .route("/{hash}/{*path}", get(crate::blob_serve_http::blob_serve))
        .layer(middleware::from_fn(
            crate::blob_serve_http::blob_serve_csp_middleware,
        ));

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
        .route(
            "/api/daemon/curators",
            get(crate::curators_api::list_curators),
        )
        .route(
            "/api/daemon/curators/subscribe",
            post(crate::curators_api::subscribe_curator),
        )
        .route(
            "/api/daemon/curators/{pubkey}",
            delete(crate::curators_api::unsubscribe_curator),
        )
        .route("/api/daemon/browse", get(crate::browse_api::list_browse))
        .route(
            "/api/daemon/browse/pull",
            post(crate::browse_api::browse_pull),
        )
        // Sprint 74 Phase D: toggle a self-deployed app's local keep-online pin.
        .route(
            "/api/daemon/keep-online",
            post(crate::seed_api::set_keep_online),
        )
        // Sprint 74 Phase E: cross-node seed. `/seed` = voluntary community
        // seed of a distant public app; `/seed/invite*` = revocable invite
        // ledger for the authenticated `sbfb/seed/0` protocol.
        .route("/api/daemon/seed", post(crate::seed_api::seed_voluntary))
        // Sprint 75 Phase E: REQUESTER leg of the authenticated
        // `sbfb/seed/0` protocol — ask a designated peer (my anchor) to
        // seed an app this node holds. Scriptable, headless-compatible.
        .route(
            "/api/daemon/seed/request",
            post(crate::seed_api::seed_request_peer),
        )
        .route(
            "/api/daemon/seed/invite",
            post(crate::seed_api::seed_invite_mint),
        )
        .route(
            "/api/daemon/seed/invite/revoke",
            post(crate::seed_api::seed_invite_revoke),
        )
        .route(
            "/api/daemon/seed/invites/{project_id}",
            get(crate::seed_api::seed_invite_list),
        )
        // Sprint 74 Phase F: best-effort multi-seed availability count.
        .route(
            "/api/daemon/seed-count/{project_id}",
            get(crate::seed_api::seed_count),
        )
        // Sprint 75 Phase D: node identity exposure — the subscribed node
        // directories grouped by publishing node (read-only projection).
        .route("/api/daemon/nodes", get(crate::browse_api::list_nodes))
        // Sprint 77 Phase J: read-only status of a private compute-group shard
        // session. Control-plane only — an AGGREGATE status (member count),
        // NEVER the group's member identities. Same loopback bearer+Host+Origin
        // tier as its siblings (authed_routes).
        .route(
            "/api/daemon/shard-session/{session_id}",
            get(crate::shard_session_http_api::shard_session),
        )
        // Sprint 81 Phase I: the in-vivo shard-session orchestrator surface
        // (the ex-S78 session driver). Operator-facing loopback tool routes:
        // mint the signed private group, mount a session (placement +
        // manifest + readiness barrier + gated registry insert), drive a
        // generation, poll its measured result, and cut the tail shard
        // (explicit counted churn). The live b3_shard harness (Phase J)
        // consumes generate/result/drop-shard verbatim.
        .route(
            "/api/daemon/shard-session/group",
            post(crate::shard_session_http_api::shard_session_group),
        )
        .route(
            "/api/daemon/shard-session/mount",
            post(crate::shard_session_http_api::shard_session_mount),
        )
        .route(
            "/api/daemon/shard-session/{session_id}/generate",
            post(crate::shard_session_http_api::shard_session_generate),
        )
        .route(
            "/api/daemon/shard-session/{session_id}/result",
            get(crate::shard_session_http_api::shard_session_result),
        )
        .route(
            "/api/daemon/shard-session/{session_id}/drop-shard",
            post(crate::shard_session_http_api::shard_session_drop_shard),
        )
        .route(
            "/api/daemon/publish",
            post(crate::publish_api::publish_project),
        )
        .route(
            "/api/daemon/publish-blob",
            post(crate::publish_api::publish_blob),
        )
        .route(
            "/api/daemon/directory/publish",
            post(crate::publish_api::publish_directory),
        )
        .route(
            "/api/daemon/default-curators",
            get(crate::curators_api::default_curators),
        )
        .route(
            "/api/daemon/panic/wipe",
            post(crate::blob_serve_http::panic_wipe),
        )
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
            get(crate::diagnostic_api::diagnostic_neighborhood),
        )
        // Sprint 30 Phase C : FROST DKG + ceremony admin endpoints.
        // Trust tier T0 — behind the same loopback bearer + Host +
        // Origin gate as every other authenticated route.
        .route(
            "/api/canary/frost/trusted-dealer",
            post(crate::frost_api::frost_trusted_dealer),
        )
        .route(
            "/api/canary/frost/round1",
            post(crate::frost_api::frost_round1),
        )
        .route(
            "/api/canary/frost/round2",
            post(crate::frost_api::frost_round2),
        )
        .route(
            "/api/canary/frost/aggregate",
            post(crate::frost_api::frost_aggregate),
        )
        .route(
            "/api/v1/tasks/submit",
            post(crate::coordinator_api::coordinator_submit_task),
        )
        .route(
            "/api/v1/results/submit",
            post(crate::coordinator_api::coordinator_submit_result),
        )
        .route(
            "/api/v1/kudos/{project_id}",
            get(crate::coordinator_api::coordinator_get_kudos),
        )
        .route(
            "/api/v1/kudos/{project_id}/verify",
            get(crate::coordinator_api::coordinator_verify_chain),
        )
        .route(
            "/api/canary/observed",
            post(crate::canary_api::canary_observed),
        )
        .route(
            "/api/canary/network-health",
            get(crate::canary_api::canary_network_health),
        )
        .route(
            "/api/canary/freshness/{pubkey}",
            get(crate::canary_api::canary_freshness),
        )
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
        .route(
            "/api/daemon/feed/cursor",
            get(crate::feed_api::get_feed_cursor),
        )
        .route(
            "/api/daemon/feed/entries",
            get(crate::feed_api::get_feed_entries),
        )
        .route("/api/daemon/search", get(crate::search_api::search_handler))
        .route(
            "/api/daemon/proof-card/{project_id}",
            get(crate::preview_api::get_proof_card),
        )
        .route(
            "/api/v1/preview/load",
            post(crate::preview_api::preview_load),
        )
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
            get(crate::feed_api::get_provenance),
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
    if let Some(p) = port_opt
        && p.parse::<u16>().is_err()
    {
        return false;
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
    if let Some(p) = port_opt
        && p.parse::<u16>().is_err()
    {
        return false;
    }
    true
}

// =================================================================
// Request / response DTOs
// =================================================================

/// Body of `GET /browse`.
///
/// Sorted flat list of every project entry across every cached
/// curator list, each row carrying a reachability bucket the
/// React shell renders as a coloured dot.
/// Test-only deserialization target for the `/browse` JSON. Production serves
/// the response via `browse_api::BrowseEntryView` (BrowseEntry + the derived `is_own` +
/// `from_subscribed`), not this struct; the tests deserialize into it and
/// `BrowseEntry` simply ignores the extra derived keys (no
/// `deny_unknown_fields` on the entry).
#[cfg(test)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrowseListResponse {
    pub entries: Vec<BrowseEntry>,
}

/// Body returned when a curator runtime error must be surfaced
/// as a 4xx/5xx response.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ErrorResponse {
    /// `pub(crate)`: the publish handlers (Sprint 82 Phase S,
    /// `publish_api.rs`) CONSTRUCT the literal cross-module — the
    /// struct alone is not enough, the field must reach them too.
    pub(crate) error: String,
}

pub(crate) fn runtime_error_to_response(
    err: CuratorRuntimeError,
) -> (StatusCode, Json<ErrorResponse>) {
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

/// Truncate `s` to at most `max_bytes` bytes without splitting a UTF-8
/// character (the cut falls back to the nearest lower char boundary). Used to
/// clamp catalog fields to their `NODE_DIRECTORY_*_MAX` before signing, since
/// the deploy/publish producers impose no length cap of their own, and to cap
/// the search `q` param to `MAX_SEARCH_QUERY_BYTES` (CARRY-5).
pub(crate) fn truncate_on_char_boundary(s: &str, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
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
    // `/publish` (gated in `publish_api.rs`), AND gossip ingest from an untrusted
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

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{BlobsClient, KeyPair, create_node};
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

    // ---------------------------------------------------------
    // Sprint 12 Phase A: blob-serve ticket helper (the blob-serve
    // handler + CSP tests live in blob_serve_http.rs since S82 Phase S4;
    // the publish-blob tests in publish_api.rs since S82 Phase S)
    // ---------------------------------------------------------

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

    // ---------------------------------------------------------
    // Sprint 53 Phase A: SPA route collision regression tests
    // ---------------------------------------------------------

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
}
