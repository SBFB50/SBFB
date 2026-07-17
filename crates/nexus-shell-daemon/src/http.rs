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
    response::{IntoResponse, Json},
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
use tracing::{debug, warn};

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
/// the search `q` param to [`MAX_SEARCH_QUERY_BYTES`] (CARRY-5).
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
/// observable neighborhood. iroh exposes no DHT routing-table
/// enumeration (re-checked against 1.0.1 at the S81 Phase C bump:
/// only per-peer `Endpoint::remote_info(EndpointId)` exists — the
/// `remote_info_iter` once expected "post-0.98" never landed), so
/// the observable neighborhood is the set of subscribed curator
/// pubkeys — the peers this daemon actively tracks via gossip.
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
            if let Some(ref ot) = params.op_type
                && row.op_type != *ot {
                    return false;
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
    use crate::test_support::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::{KeyPair, create_node};
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
    // Sprint 12 Phase A: blob-serve (the publish-blob tests live in
    // publish_api.rs since S82 Phase S)
    // ---------------------------------------------------------

    /// S82 Phase D — exercise the feed/insert internal-auth guard (S65
    /// ace05b0, P2-FEED-INSERT-NO-AUTH-TIER) hermetically so a future
    /// refactor cannot silently regress it to a pre-S65 no-auth endpoint.
    /// The `multi_daemon` feed integration tests are relay-gated (they
    /// self-skip on a default run), so before this test the tree had NO
    /// default-CI coverage of the 403 path.
    #[tokio::test]
    async fn feed_insert_rejects_without_internal_header() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        // Without `x-sbfb-feed-internal` the guard rejects with 403 before
        // touching any feed state.
        let resp = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/feed/insert")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"op":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::FORBIDDEN,
            "feed/insert must reject a request lacking x-sbfb-feed-internal"
        );

        // With the header the guard passes; mk_state has no feed_sync_state
        // so the handler proceeds to 503 (NOT 403) — proving the header is
        // the gate, not an unrelated rejection. This positive control is
        // coupled to mk_state's `feed_sync_state: None`: if mk_state ever
        // gains a feed_sync_state, this assertion fails VISIBLY — re-anchor
        // it (e.g. assert any non-403 success/error), don't delete it.
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/feed/insert")
                    .header("content-type", "application/json")
                    .header("x-sbfb-feed-internal", "1")
                    .body(axum::body::Body::from(r#"{"op":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::SERVICE_UNAVAILABLE,
            "internal header passes the auth gate (503 = feed sync uninitialised in test state)"
        );
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
        // S81 Phase A4: hermetic — consent routes resolve ~/.sbfb when
        // sbfb_home is None, so a rig-level consent.json would leak in
        // (the exact pollution the A3 baseline hit). Pin a tempdir.
        let tmp = tempfile::tempdir().expect("tmpdir");
        let app = build_test_router(mk_state_with_sbfb_home(tmp.path().to_path_buf()).await);
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
