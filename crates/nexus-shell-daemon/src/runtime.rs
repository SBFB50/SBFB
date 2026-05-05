// SPDX-License-Identifier: AGPL-3.0-or-later
//! Boot, HTTP serve, gossip subscribe, graceful shutdown.
//!
//! [`DaemonRuntime::start`] is the single entry point the binary
//! uses to bring up the daemon. It performs the following ordered
//! sequence (Phase A + Phase C):
//!
//! 1. [`nexus_shell_daemon_core::registry::check_stale_or_bail`]
//!    — refuse to boot if a live daemon is already running.
//! 2. `nexus_core_rs::create_node()` — spin up the iroh endpoint +
//!    protocol router + blobs/docs/gossip stack. Wrapped in
//!    [`Arc`] so the gossip subscribe task can hold a second
//!    reference for its whole lifetime.
//! 3. Bind a TCP listener on `(api_host, 0)` so the OS picks an
//!    ephemeral port.
//! 4. Write `running.json` (Phase A singleton marker).
//! 5. Construct [`CuratorRuntime::with_persistence`] over
//!    `<root>/shell-daemon/subscriptions.json` so the attention
//!    set survives daemon restarts (R7 mitigation).
//! 6. Build the shared HTTP state (carries the `Arc<CuratorRuntime>`)
//!    and spawn the axum serve task on a oneshot shutdown channel.
//! 7. Spawn the gossip subscribe task on a second oneshot
//!    shutdown channel. The task joins the curator topic, pulls
//!    events, and hands every message to
//!    [`CuratorRuntime::process_announcement_bytes`].
//!
//! [`DaemonRuntime::shutdown`] drives the reverse order:
//! stop gossip task → stop HTTP serve → shutdown iroh node
//! (via `Arc::try_unwrap` once every task has dropped its clone)
//! → remove `running.json`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{anyhow, Context, Result};
use nexus_core_rs::{
    create_node_with_config, load_quorum_resolvers_from_env, relay_pow_policy_file_path,
    GossipClient, GossipEvent, KeyPair, Node, NodeConfig, PowSolveCache, PowVerifyCache,
    RelayPowPolicy,
};
use nexus_shell_daemon_core::auth;
use nexus_shell_daemon_core::browse::{
    BrowseAggregator, BrowseAggregatorHandle, BrowseEntry, BrowseSource, BrowseStatus,
};
use nexus_shell_daemon_core::config::{CuratorConfig, ShellDaemonPaths};
use nexus_shell_daemon_core::iroh_runtime::{
    curator_topic_id, CuratorRuntime, CuratorRuntimeError, CuratorRuntimeHandle,
};
use nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher;
use nexus_shell_daemon_core::publish;
use nexus_shell_daemon_core::registry::{
    self, new_running_state, remove_running, write_running, StaleOutcome,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use nexus_trace_core::batch_log::BatchLogProcessor;

use crate::http::{build_router, DaemonHttpState, GossipSenderHandle};

// Sprint 20 Phase A : the env var name used by the launcher to
// hand the unlocked 32-byte Ed25519 secret key to the daemon child
// is defined canonically in `nexus_core_rs::keystore::
// SBFB_IDENTITY_SECRET_HEX_ENV` so launcher + daemon share exactly
// one string constant. We import it here rather than redeclaring.
use nexus_core_rs::SBFB_IDENTITY_SECRET_HEX_ENV;

/// Read the identity env var if the launcher set one, parse it as
/// 32 hex bytes, wipe the env table entry, and return the bytes.
///
/// The env var is removed before this function returns so a
/// subsequent child spawn (for example a future worker process the
/// daemon may fork) does not inherit the secret. The remaining
/// copies — the `[u8; 32]` returned to the caller and the original
/// heap string pulled by `env::var` — are either owned by the
/// caller (and handed straight to `NodeConfig::with_secret_key`,
/// where `iroh::SecretKey` takes over lifetime management) or are
/// dropped and zeroed by `zeroize::Zeroize` inside this function.
fn read_optional_identity_env() -> Option<[u8; 32]> {
    let mut raw = match std::env::var(SBFB_IDENTITY_SECRET_HEX_ENV) {
        Ok(v) => v,
        Err(_) => return None,
    };
    // Remove the env var from the process environment regardless of
    // whether the parse succeeds. If the var is malformed we want
    // the daemon to fall back to legacy mode rather than a second
    // parse attempt later picking up stale state.
    std::env::remove_var(SBFB_IDENTITY_SECRET_HEX_ENV);

    let decoded = match hex::decode(raw.trim()) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "SBFB_IDENTITY_SECRET_HEX is not valid hex, falling back to ephemeral identity");
            use zeroize::Zeroize;
            raw.zeroize();
            return None;
        }
    };
    use zeroize::Zeroize;
    raw.zeroize();

    if decoded.len() != 32 {
        warn!(
            actual = decoded.len(),
            "SBFB_IDENTITY_SECRET_HEX decoded to wrong length (expected 32), falling back to ephemeral identity"
        );
        let mut decoded = decoded;
        decoded.zeroize();
        return None;
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    let mut decoded = decoded;
    decoded.zeroize();
    Some(out)
}

/// Load the 32-byte Ed25519 secret from `<root>/node_key`, or
/// generate a fresh one and persist it. This gives the daemon a
/// stable identity across restarts without requiring the launcher
/// keystore (`SBFB_IDENTITY_SECRET_HEX`).
fn load_or_generate_node_key(root: &Path) -> Result<[u8; 32]> {
    let path = root.join("node_key");
    if path.exists() {
        let data = std::fs::read(&path)
            .with_context(|| format!("failed to read node_key from {}", path.display()))?;
        if data.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&data);
            return Ok(out);
        }
        warn!(len = data.len(), "node_key has wrong length, regenerating");
    }
    let secret = KeyPair::generate().secret_bytes();
    std::fs::write(&path, secret)
        .with_context(|| format!("failed to write node_key to {}", path.display()))?;
    Ok(secret)
}

/// Options the binary hands to [`DaemonRuntime::start`].
///
/// Owned rather than borrowed so the runtime can hold on to the
/// paths for the whole daemon lifetime and the binary's `main`
/// does not have to keep `Arc`-ing them around.
pub struct DaemonStartOptions {
    pub paths: ShellDaemonPaths,
    pub api_host: String,
    /// Caller-supplied port; `0` means ephemeral. The Phase A
    /// config default is `0` so most real runs let the OS pick.
    pub api_port: u16,
    pub daemon_version: String,
    /// Sprint 11 Phase B: `[curator]` section from config.
    pub curator: CuratorConfig,
    /// Sprint 20 Phase B : which slot the launcher's `sbfb
    /// unlock` matched. `Normal` in the typical case, `Duress`
    /// when the user typed the duress PIN. Propagated into
    /// `DaemonHttpState` so the noop routing helpers can gate
    /// every publish / subscribe / dispatch handler.
    pub identity_mode: nexus_core_rs::IdentityMode,
    /// Sprint 33 Phase A: extra CORS origins from `--cors-origin`.
    pub cors_origins: Vec<String>,
    /// Path to built React shell directory. When set, the daemon
    /// serves static files on `/` without bearer auth.
    pub web_root: Option<PathBuf>,
}

/// A live `nexus-shell-daemon` process.
///
/// Owns the iroh [`Node`] (via an `Arc` so the gossip task can
/// share it), the running.json path (so it can remove it on
/// shutdown), the HTTP and gossip task handles, their oneshot
/// shutdown senders, and the shared [`CuratorRuntime`] handle
/// that HTTP routes read out of.
///
/// The `curator_runtime` field is cloned into both the HTTP
/// state and the gossip task, so from the `cargo build` main
/// binary's perspective it looks unused (the main code path
/// only hands the clone off once and forgets about it). The
/// test harness and Phase D browse path need access to it
/// through [`DaemonRuntime::curator_runtime`], so the field is
/// explicitly allowed as dead_code. Phase D will reach into it
/// to filter browse entries by the current attention set.
pub struct DaemonRuntime {
    node: Option<Arc<Node>>,
    #[allow(dead_code)]
    curator_runtime: CuratorRuntimeHandle,
    running_json: PathBuf,
    http_handle: JoinHandle<()>,
    http_shutdown: Option<oneshot::Sender<()>>,
    gossip_handle: JoinHandle<()>,
    gossip_shutdown: Option<oneshot::Sender<()>>,
    /// Sprint 16 Phase B (D2): UDS (Unix) or Named Pipe (Windows)
    /// accept loop running beside the TCP listener. `None` only
    /// when the bind failed at boot — the daemon then runs with
    /// TCP-only loopback (warn already logged).
    peer_handle: Option<JoinHandle<()>>,
    peer_shutdown: Option<oneshot::Sender<()>>,
    /// Sprint 18 audit fix D-1: file-watcher on `tokens.json`. Keeps
    /// the rotator handed to `AuthState::Rotated` synchronised with
    /// the launcher's 24 h rotation. `None` when no `tokens.json`
    /// exists at boot — the static `auth_token` path is used and
    /// no watcher is spawned. Stored on the runtime so the watcher
    /// thread + inotify handle live for the whole daemon process.
    #[allow(dead_code)]
    tokens_watcher: Option<nexus_shell_daemon_core::auth::TokenRotatorWatcher>,
    /// Sprint 20 Phase C : file-watcher on `relay_pow_policy.toml`.
    /// Kept alive for the daemon's whole lifetime so reloads reach
    /// both the publish handler (`PowSolveCache` next-solve) and the
    /// gossip receive loop (`PowVerifyCache` policy check). `None`
    /// when no `sbfb_home` resolves — the daemon then runs on a
    /// detached default-policy handle (fallback logged at boot).
    #[allow(dead_code)]
    pow_policy_watcher: Option<PowPolicyWatcher>,
    dispatch_handle: Option<JoinHandle<()>>,
    dispatch_shutdown: Option<oneshot::Sender<()>>,
    bound_addr: std::net::SocketAddr,
}

impl DaemonRuntime {
    /// Execute the full boot sequence and return a live
    /// [`DaemonRuntime`].
    pub async fn start(opts: DaemonStartOptions) -> Result<Self> {
        opts.paths
            .ensure_dirs()
            .context("failed to create shell-daemon directories")?;

        // 1. Singleton check.
        match registry::check_stale_or_bail(&opts.paths.running_json) {
            StaleOutcome::Live { pid, state } => {
                return Err(anyhow!(
                    "daemon already running (pid {}, node_id {}, port {}); stop it first or delete {} if the process is known-dead",
                    pid,
                    &state.node_id,
                    state.api_port,
                    opts.paths.running_json.display()
                ));
            }
            StaleOutcome::Stale { pid, .. } => {
                warn!(
                    pid = pid,
                    path = %opts.paths.running_json.display(),
                    "found stale running.json — previous daemon exited without cleanup, overwriting"
                );
            }
            StaleOutcome::Corrupt { reason } => {
                warn!(
                    reason = %reason,
                    path = %opts.paths.running_json.display(),
                    "running.json is unreadable, overwriting"
                );
            }
            StaleOutcome::NoFile => {
                info!("no existing running.json — fresh daemon boot");
            }
        }

        // 2. Boot the iroh endpoint + protocol router. Arc so
        //    the gossip task can hold a clone without fighting
        //    with the shutdown path for ownership.
        //
        //    Sprint 20 Phase A: if the launcher ran `sbfb unlock`
        //    before spawning us, it put the 32-byte Ed25519 secret
        //    key as a 64-char hex string into
        //    `SBFB_IDENTITY_SECRET_HEX`. We pick it up here and
        //    hand it to `NodeConfig::with_secret_key` so the iroh
        //    endpoint boots with a persistent identity instead of
        //    minting a fresh ephemeral keypair each run. Absent or
        //    malformed env → legacy `create_node()` path so dev
        //    flows that predate the encrypted keystore still work.
        // Sprint 20 Phase C : mint the PoW keypair from the same
        // secret the iroh endpoint consumes so the Hashcash
        // `publisher_pubkey` field matches the node identity the
        // peers already know via gossip. A fallback ephemeral keypair
        // (generated when the launcher did not hand a secret) pairs
        // naturally with the `create_node()` ephemeral identity path.
        let (node, pow_keypair) = match read_optional_identity_env() {
            Some(secret_bytes) => {
                info!("shell daemon using persistent identity from launcher keystore");
                let pow_kp = KeyPair::from_secret_bytes(&secret_bytes);
                let cfg = NodeConfig::default().with_secret_key(secret_bytes);
                let n = create_node_with_config(cfg)
                    .await
                    .context("failed to boot iroh node with persistent identity")?;
                (n, pow_kp)
            }
            None => {
                let secret_bytes = load_or_generate_node_key(&opts.paths.root)?;
                info!(
                    path = %opts.paths.root.join("node_key").display(),
                    "shell daemon using file-based persistent identity"
                );
                let pow_kp = KeyPair::from_secret_bytes(&secret_bytes);
                let cfg = NodeConfig::default().with_secret_key(secret_bytes);
                let n = create_node_with_config(cfg)
                    .await
                    .context("failed to boot iroh node with file-based identity")?;
                (n, pow_kp)
            }
        };
        let node_id = node.node_id();
        info!(node_id = %node_id, "shell daemon iroh node ready");
        let node = Arc::new(node);
        let pow_keypair = Arc::new(pow_keypair);

        // 3. Bind the TCP listener. An empty host in the config
        //    was clamped to 127.0.0.1 at load time (see
        //    `ShellDaemonConfig::clamped`); defend-in-depth here
        //    too so a future bypass of `load` cannot slip
        //    through.
        let host = if opts.api_host.is_empty() {
            "127.0.0.1".to_string()
        } else {
            opts.api_host.clone()
        };
        let bind_target = format!("{}:{}", host, opts.api_port);
        let listener = TcpListener::bind(&bind_target)
            .await
            .with_context(|| format!("failed to bind HTTP listener to {bind_target}"))?;
        let bound_addr = listener
            .local_addr()
            .context("local_addr on freshly bound TcpListener")?;
        info!(addr = %bound_addr, "shell daemon HTTP listener bound");

        // 4. Write running.json with the real port.
        let running_json_path = opts.paths.running_json.clone();
        let running_state = new_running_state(
            node_id.clone(),
            host.clone(),
            bound_addr.port(),
            opts.daemon_version.clone(),
        );
        write_running(&running_state, &running_json_path)
            .with_context(|| format!("failed to write {}", running_json_path.display()))?;

        // 5. Construct the curator runtime and pre-load any
        //    previously persisted attention set.
        let curator_runtime: CuratorRuntimeHandle = Arc::new(CuratorRuntime::with_persistence(
            opts.paths.subscriptions_json.clone(),
        ));

        // 5a. Sprint 11 Phase B: auto-subscribe to default curators.
        // Idempotent — curators already in the persisted attention
        // set are silently skipped.
        let mut auto_subscribed = 0u32;
        for hex_key in &opts.curator.default_curators {
            if curator_runtime.subscribed_pubkeys_hex().contains(hex_key) {
                debug!(curator = %hex_key, "default curator already subscribed, skipping");
                continue;
            }
            match curator_runtime.subscribe(hex_key) {
                Ok(_) => {
                    auto_subscribed += 1;
                    debug!(curator = %hex_key, "auto-subscribed to default curator");
                }
                Err(e) => {
                    warn!(curator = %hex_key, error = %e, "failed to auto-subscribe to default curator");
                }
            }
        }
        if auto_subscribed > 0 {
            info!(
                count = auto_subscribed,
                "auto-subscribed to default curator(s)"
            );
        }

        // 5b. Construct the Phase D browse aggregator. It is
        //     cheap (just a DashMap + two duration knobs) so
        //     it is always instantiated at boot; GET /browse
        //     reaches through this handle + the Arc<Node> to
        //     probe each project's reachability on demand.
        //
        //     Sprint 19 Phase A : if `SBFB_PKARR_RELAYS` is set,
        //     wire the pkarr quorum canary so probe_and_cache
        //     short-circuits to Unreachable on an incoherent
        //     multi-relay lookup (Eclipse-by-DHT defence active
        //     in production). When the env var is absent the
        //     aggregator boots without the canary — behaviour is
        //     byte-for-byte the pre-Sprint-19 path.
        let quorum_resolvers = load_quorum_resolvers_from_env()
            .context("failed to build pkarr quorum resolvers from SBFB_PKARR_RELAYS env")?;
        let browse_aggregator: BrowseAggregatorHandle = {
            let mut agg = BrowseAggregator::new();
            if let Some(resolvers) = quorum_resolvers {
                let count = resolvers.len();
                agg = agg.with_quorum_resolvers(resolvers);
                if count < 2 {
                    warn!(
                        count,
                        "pkarr quorum canary armed with a single relay — inter-relay cross-checking requires 2+ distinct URLs (SBFB_PKARR_RELAYS)"
                    );
                } else {
                    info!(
                        count,
                        "pkarr quorum canary armed — Eclipse-by-DHT defence active"
                    );
                }
            } else {
                info!(
                    "pkarr quorum canary disabled (SBFB_PKARR_RELAYS not set) — browse probes use the default iroh N0 discovery path"
                );
            }
            Arc::new(agg)
        };

        // 5c. Sprint 11 Phase A: shared gossip sender slot. Set to
        //     `Some(sender)` once the gossip task joins the topic.
        let gossip_sender: GossipSenderHandle = Arc::new(tokio::sync::RwLock::new(None));

        // 5d. Sprint 20 Phase B : provision the panic wipe service
        //     + read the identity mode the launcher handed us.
        //     The keystore points at the same `<root>/keyring/`
        //     directory the launcher's `sbfb unlock` wrote to,
        //     so `wipe_all` reaches the exact two blob files.
        //     The state_db path matches `subscriptions_json`
        //     (the Sprint 11 persistence file, named .json but
        //     treated here as the "session state" store — any
        //     future sqlite file would be added alongside).
        let keyring_dir = opts.paths.root.join("keyring");
        let blob_cache_dir = opts.paths.root.join("blob-cache");
        let keystore_for_panic = Arc::new(nexus_core_rs::LocalFileKeyStore::new(&keyring_dir));
        let panic_wipe = Arc::new(crate::panic::PanicWipeService::new(
            keystore_for_panic,
            opts.paths.subscriptions_json.clone(),
            blob_cache_dir,
            Arc::new(crate::panic::RealExit) as Arc<dyn crate::panic::ExitStrategy>,
        ));
        let identity_mode = opts.identity_mode;

        // 5e. Sprint 20 Phase C : spawn the PoW policy file-watcher
        //     and provision the publisher / subscriber caches.
        //
        //     The watcher resolves `~/.sbfb/relay_pow_policy.toml`
        //     (overridable via `SBFB_POW_POLICY_PATH`), loads the
        //     initial policy synchronously, then starts a background
        //     thread that reloads the TOML on every write. A missing
        //     file boots on the S19 default (2^18 leading zero bits
        //     for every topic, no overrides) ; a malformed file at
        //     boot fails loud, a malformed edit at runtime keeps the
        //     last known-good policy in memory.
        //
        //     When no `$SBFB_HOME` / `$HOME` resolves (rare — headless
        //     test harness that strips the environment) the watcher
        //     is skipped and we run on a detached default-policy
        //     handle so the gate still enforces 2^18 everywhere.
        let curator_topic = curator_topic_id();
        let pow_solve_cache = Arc::new(PowSolveCache::new());
        let pow_verify_cache = Arc::new(PowVerifyCache::new());
        let (pow_policy, _pow_policy_watcher) = match relay_pow_policy_file_path() {
            Some(path) => match PowPolicyWatcher::spawn(path.clone()) {
                Ok(w) => {
                    info!(
                        path = %path.display(),
                        "PoW policy watcher armed — hot-reload enabled"
                    );
                    (w.shared(), Some(w))
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        path = %path.display(),
                        "PoW policy watcher spawn failed, falling back to default policy"
                    );
                    (
                        nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
                        None,
                    )
                }
            },
            None => {
                warn!(
                    "SBFB_HOME / HOME / USERPROFILE not set — PoW policy locked to default (2^18)"
                );
                (
                    nexus_shell_daemon_core::pow_policy_loader::shared_default_policy(),
                    None,
                )
            }
        };

        // 5f. Sprint 29 Phase D: initialize the trace processor
        //     pipeline with a BatchLogProcessor writing to
        //     `<root>/traces/daemon.jsonl`. The processor is
        //     registered globally via `set_trace_processors` so
        //     any crate in the daemon process can call `emit()`.
        let trace_log_path = opts.paths.root.join("traces").join("daemon.jsonl");
        match BatchLogProcessor::new(&trace_log_path, 10 * 1024 * 1024) {
            Ok(proc) => {
                nexus_trace_core::set_trace_processors(vec![Box::new(proc)]);
                info!(path = %trace_log_path.display(), "trace processor pipeline initialized");
            }
            Err(e) => {
                warn!(error = %e, "failed to initialize trace processor — tracing disabled");
            }
        }

        // 6a. Open the coordinator SQLite database (persistent).
        let coordinator_db_path = opts.paths.root.join("coordinator.db");
        let coordinator_db = nexus_coordinator_rs::db::CoordinatorDb::open(&coordinator_db_path)
            .map_err(|e| anyhow::anyhow!("coordinator DB open failed: {e}"))?;
        let coordinator_db = std::sync::Arc::new(std::sync::Mutex::new(coordinator_db));

        // 6b. Sprint 38 Phase A: create the result event broadcast
        //     channel for the validator loop. The sender is stored in
        //     DaemonHttpState so future gossip wiring can forward
        //     result events to the loop.
        let (result_event_tx, result_event_rx) = crate::validator_loop::create_result_channel();

        // 6c. Sprint 49 Phase A: create or reopen the project iroh-docs
        //     document. The daemon acts as coordinator for the local
        //     user's project — single-project mode (D1).
        let docs_client = nexus_core_rs::docs::DocsClient::new(node.docs());
        let doc_author = docs_client
            .author_default()
            .await
            .context("failed to get default docs author")?;
        let project_doc = {
            let existing = docs_client
                .list_docs()
                .await
                .context("failed to list project docs")?;
            if let Some(&first_id) = existing.first() {
                docs_client
                    .open_doc(first_id)
                    .await
                    .context("failed to open project doc")?
                    .ok_or_else(|| anyhow!("project doc listed but failed to open"))?
            } else {
                docs_client
                    .create_doc()
                    .await
                    .context("failed to create project doc")?
            }
        };
        info!(
            doc_id = %project_doc.id(),
            author = %doc_author,
            "project doc ready for coordinator dispatch"
        );
        let project_doc = Arc::new(project_doc);

        // 6c-2. Sprint 49 Phase A: create the dispatch MPSC channel and
        //       spawn the dispatch loop. The loop is the sole writer to
        //       the project doc (G1 D2 ack — sequential writes, no
        //       contention with HTTP handlers).
        let (task_dispatch_tx, task_dispatch_rx) = crate::dispatch_loop::create_dispatch_channel();
        let (dispatch_shutdown_tx, dispatch_shutdown_rx) = oneshot::channel::<()>();
        let dispatch_handle = {
            let doc_clone = Arc::clone(&project_doc);
            tokio::spawn(crate::dispatch_loop::run(
                task_dispatch_rx,
                doc_clone,
                doc_author,
                dispatch_shutdown_rx,
            ))
        };

        // 6d. Build gossip command channel + shared HTTP state +
        //     spawn the serve task.
        let (gossip_cmd_tx, gossip_cmd_rx) = tokio::sync::mpsc::channel::<GossipCmd>(64);
        let http_state = Arc::new(DaemonHttpState {
            node_id,
            daemon_version: opts.daemon_version.clone(),
            boot_time: SystemTime::now(),
            api_host: host,
            api_port: bound_addr.port(),
            curator_runtime: Arc::clone(&curator_runtime),
            browse_aggregator: Arc::clone(&browse_aggregator),
            node: Arc::clone(&node),
            gossip_sender: Arc::clone(&gossip_sender),
            gossip_cmd_tx: gossip_cmd_tx.clone(),
            default_curators: opts.curator.default_curators.clone(),
            blob_serve_cache: Arc::new(nexus_shell_daemon_core::blob_serve::BlobServeCache::new(
                nexus_shell_daemon_core::blob_serve::DEFAULT_MAX_CACHE_ENTRIES,
            )),
            identity_mode,
            panic_wipe,
            pow_solve_cache: Arc::clone(&pow_solve_cache),
            pow_policy: Arc::clone(&pow_policy),
            pow_keypair: Arc::clone(&pow_keypair),
            curator_gossip_topic: curator_topic,
            // Sprint 22 Phase C : outbound HTTP client for the
            // contributor-verify proxy. Built once at boot so
            coordinator_db: Arc::clone(&coordinator_db),
            result_event_tx,
            canary_registry: {
                let dir = nexus_shell_daemon_core::paths::shell_daemon_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from(".sbfb"));
                Arc::new(std::sync::Mutex::new(
                    nexus_coordinator_rs::canary_registry::CanaryRegistry::new(
                        dir.join("canary_registry.json"),
                    ),
                ))
            },
            canary_input: None,
            sbfb_home: None,
            project_doc: Some(Arc::clone(&project_doc)),
            task_dispatch_tx: Some(task_dispatch_tx),
        });
        // Sprint 16 Phase A (D1): load the loopback bearer token.
        // The launcher generates it at first boot; if we are being
        // started directly (cargo run, tests, packaging without a
        // launcher), generate + persist one ourselves so the
        // shell binary can hit a clean daemon stand-alone.
        //
        // `SBFB_AUTH_TOKEN` env wins over the file so integration
        // tests can inject a known token without touching disk.
        //
        // Sprint 18 audit fix D-1: when the launcher has written a
        // `tokens.json` (rotation pair persisted by
        // `nexus_launcher::token_rotation::spawn_rotation_loop`),
        // boot the daemon in `AuthState::Rotated` mode and spawn a
        // file-watcher so subsequent rotations are picked up
        // without a daemon restart. The env var keeps absolute
        // precedence so test harnesses that inject a known token
        // are unaffected, and a missing file falls through to the
        // legacy single-token path (`AuthState::Static`).
        let env_token = std::env::var(auth::AUTH_TOKEN_ENV)
            .ok()
            .filter(|t| !t.is_empty());
        let (auth_state, tokens_watcher) = if let Some(t) = env_token {
            (auth::AuthState::new(t), None)
        } else if let Some(rotator_with_path) = load_initial_rotator()? {
            let (path, initial) = rotator_with_path;
            let watcher = auth::TokenRotatorWatcher::spawn(path, initial)
                .context("failed to spawn tokens.json watcher")?;
            let auth_state = auth::AuthState::rotated(watcher.shared());
            (auth_state, Some(watcher))
        } else {
            let static_token = resolve_token_from_disk()?;
            (auth::AuthState::new(static_token), None)
        };

        let router = build_router(
            http_state,
            auth_state,
            &opts.cors_origins,
            opts.web_root.as_deref(),
        );

        // 6a. Sprint 16 Phase B (D2): spawn the UDS / Named Pipe
        //     accept loop on a clone of the same router. The
        //     accept loop wraps the cloned router with the
        //     `PeerCredsVerified` extension layer so the auth
        //     middleware bypasses bearer + Host + Origin for
        //     kernel-authenticated peers. The TCP listener spawned
        //     just below keeps the strict triple-check for browser
        //     traffic.
        let (peer_handle, peer_shutdown) = spawn_peer_listener(router.clone());

        let (http_shutdown_tx, http_shutdown_rx) = oneshot::channel::<()>();
        let http_handle = tokio::spawn(async move {
            let serve = axum::serve(listener, router).with_graceful_shutdown(async move {
                let _ = http_shutdown_rx.await;
            });
            if let Err(e) = serve.await {
                warn!(error = %e, "axum serve exited with an error");
            }
        });

        // 6d. Sprint 38 Phase A: spawn the validator loop. It
        //     drains ResultEvents from the broadcast channel and
        //     validates+credits each result. Runs until the sender
        //     half is dropped (shutdown).
        let validator_db = Arc::clone(&coordinator_db);
        tokio::spawn(crate::validator_loop::run(validator_db, result_event_rx));

        // 7. Spawn the gossip subscribe task. It joins the
        //    curator topic, streams events, and forwards each
        //    message body to the curator runtime. The oneshot
        //    lets shutdown signal a clean exit.
        //
        //    Sprint 20 Phase C : the task also receives the
        //    `PowVerifyCache` + shared policy so every inbound
        //    gossip message is unwrapped from its PoW envelope
        //    and dropped if the proof fails to satisfy the
        //    policy's topic difficulty.
        let (gossip_shutdown_tx, gossip_shutdown_rx) = oneshot::channel::<()>();
        let bootstrap_peers = curator_runtime.subscribed_pubkeys_hex();
        let gossip_handle = spawn_gossip_subscribe_task(
            Arc::clone(&node),
            Arc::clone(&curator_runtime),
            Arc::clone(&browse_aggregator),
            Arc::clone(&gossip_sender),
            Arc::clone(&pow_verify_cache),
            Arc::clone(&pow_policy),
            gossip_shutdown_rx,
            bootstrap_peers,
            gossip_cmd_rx,
            Arc::clone(&pow_solve_cache),
            Arc::clone(&pow_keypair),
            curator_topic,
        );

        Ok(Self {
            node: Some(node),
            curator_runtime,
            running_json: running_json_path,
            http_handle,
            http_shutdown: Some(http_shutdown_tx),
            gossip_handle,
            gossip_shutdown: Some(gossip_shutdown_tx),
            peer_handle,
            peer_shutdown,
            tokens_watcher,
            pow_policy_watcher: _pow_policy_watcher,
            dispatch_handle: Some(dispatch_handle),
            dispatch_shutdown: Some(dispatch_shutdown_tx),
            bound_addr,
        })
    }

    /// Return the real bound socket address. Exposed so the
    /// binary's handler can print the listening port on boot
    /// for operators.
    pub fn bound_addr(&self) -> std::net::SocketAddr {
        self.bound_addr
    }

    /// Return the shared curator runtime handle. Used by tests
    /// that want to assert on the runtime state without an HTTP
    /// roundtrip, and reserved for the Phase D browse path that
    /// filters DHT resolutions by the live attention set.
    #[allow(dead_code)]
    pub fn curator_runtime(&self) -> &CuratorRuntimeHandle {
        &self.curator_runtime
    }

    /// Block on ctrl+c, returning when the user (or the test
    /// harness) signals shutdown.
    pub async fn wait_shutdown(&self) -> Result<()> {
        tokio::signal::ctrl_c()
            .await
            .context("failed to install ctrl+c handler")?;
        info!("ctrl+c received, initiating shell daemon shutdown");
        Ok(())
    }

    /// Gracefully tear down the runtime.
    ///
    /// Order:
    /// 1. Signal the gossip task to stop + join it. This drops
    ///    the gossip task's `Arc<Node>` clone.
    /// 2. Signal axum to stop + join the HTTP serve task.
    /// 3. Reclaim ownership of the iroh `Node` via
    ///    `Arc::try_unwrap` (succeeds iff every task has dropped
    ///    its clone) and call the async `shutdown()`. If
    ///    `try_unwrap` fails — meaning a task leaked a clone —
    ///    we log and let the Arc fall off the stack so Drop runs.
    /// 4. Remove `running.json`.
    ///
    /// Any individual step may fail; the function logs the
    /// failure and still runs the remaining steps so the user
    /// never sees a dangling `running.json` + dangling iroh
    /// endpoint combo from a half-executed shutdown.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(tx) = self.gossip_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.gossip_handle).await {
            warn!(error = %e, "gossip subscribe task join failed");
        }

        // Sprint 16 Phase B: signal + join the UDS / Named Pipe
        // accept loop before the TCP serve so an in-flight
        // peer-creds connection finishes draining first.
        if let Some(tx) = self.peer_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(mut handle) = self.peer_handle.take() {
            if let Err(e) = (&mut handle).await {
                warn!(error = %e, "peer (UDS / NP) accept task join failed");
            }
        }

        if let Some(tx) = self.http_shutdown.take() {
            let _ = tx.send(());
        }
        if let Err(e) = (&mut self.http_handle).await {
            warn!(error = %e, "HTTP serve task join failed");
        }

        if let Some(tx) = self.dispatch_shutdown.take() {
            let _ = tx.send(());
        }
        if let Some(mut handle) = self.dispatch_handle.take() {
            if let Err(e) = (&mut handle).await {
                warn!(error = %e, "dispatch loop task join failed");
            }
        }

        if let Some(node_arc) = self.node.take() {
            match Arc::try_unwrap(node_arc) {
                Ok(node) => {
                    if let Err(e) = node.shutdown().await {
                        warn!(error = %e, "iroh node shutdown returned an error");
                    }
                }
                Err(still_shared) => {
                    // A task leaked a clone — extremely
                    // unexpected but non-fatal. Log the strong
                    // count so a bug report has enough
                    // information, then drop the Arc and let the
                    // non-graceful Drop path run.
                    warn!(
                        strong_count = Arc::strong_count(&still_shared),
                        "iroh Node still shared at shutdown — cannot drive graceful close"
                    );
                }
            }
        }

        remove_running(&self.running_json);
        info!("shell daemon shutdown complete");
        Ok(())
    }
}

/// Resolve the bearer token from `~/.sbfb/auth_token`, generating
/// and persisting one if the file does not exist.
fn resolve_token_from_disk() -> Result<String> {
    let path = auth::auth_token_path()
        .ok_or_else(|| anyhow!("could not resolve ~/.sbfb/auth_token path for this platform"))?;
    auth::load_or_generate_token(&path).with_context(|| {
        format!(
            "failed to load or generate auth token at {}",
            path.display()
        )
    })
}

/// Sprint 18 audit fix D-1.
///
/// Try to load `<sbfb_home>/tokens.json` written by the launcher's
/// rotation loop. Returns:
///
/// - `Ok(Some((path, rotator)))` when the file is present and
///   well-formed — the daemon will boot in `AuthState::Rotated`
///   mode and spawn a watcher on `path`.
/// - `Ok(None)` when the file is absent (no launcher rotation in
///   place yet) or when the home dir does not resolve — the
///   daemon falls back to the legacy single-token `auth_token`
///   path.
/// - `Err(_)` when the file exists but is malformed — surfaced to
///   the caller because a corrupted rotation file is operator-
///   visible state, not a soft "use defaults" condition.
fn load_initial_rotator() -> Result<Option<(PathBuf, auth::TokenRotator)>> {
    let Some(path) = auth::tokens_file_path() else {
        return Ok(None);
    };
    match auth::TokenRotator::load(&path) {
        Ok(Some(rotator)) => Ok(Some((path, rotator))),
        Ok(None) => Ok(None),
        Err(e) => Err(anyhow!(
            "failed to read tokens.json at {}: {e}",
            path.display()
        )),
    }
}

impl Drop for DaemonRuntime {
    /// Best-effort `running.json` cleanup if `shutdown()` was
    /// never called (panic, early return, test drop). The iroh
    /// node and the HTTP task are left to their own Drop impls
    /// — they are not as visible to the user as the singleton
    /// marker file.
    fn drop(&mut self) {
        if Path::new(&self.running_json).exists() {
            remove_running(&self.running_json);
        }
    }
}

// =================================================================
// Peer (UDS / Named Pipe) accept loop
// =================================================================

/// Sprint 16 Phase B (D2): spawn the UDS (Unix) or Named Pipe
/// (Windows) accept loop on `router`. Returns `(None, None)` if
/// the bind fails — the daemon then falls back to TCP-only
/// loopback (with the bearer token still required), and the
/// failure is logged.
///
/// The router is consumed (cloned by the caller before the call
/// when the original is needed for TCP serving).
fn spawn_peer_listener(
    router: axum::Router,
) -> (Option<JoinHandle<()>>, Option<oneshot::Sender<()>>) {
    #[cfg(unix)]
    {
        let path = match crate::uds_server::resolve_socket_path() {
            Ok(p) => p,
            Err(e) => {
                warn!(error = %e, "could not resolve UDS path; UDS listener disabled");
                return (None, None);
            }
        };
        match crate::uds_server::spawn(router, path) {
            Ok((handle, tx)) => (Some(handle), Some(tx)),
            Err(e) => {
                warn!(error = %e, "UDS listener bind failed; daemon runs TCP-only with bearer auth");
                (None, None)
            }
        }
    }
    #[cfg(windows)]
    {
        let pipe_name = crate::named_pipe_server::resolve_pipe_name();
        match crate::named_pipe_server::spawn(router, pipe_name) {
            Ok((handle, tx)) => (Some(handle), Some(tx)),
            Err(e) => {
                warn!(error = %e, "Named Pipe listener bind failed; daemon runs TCP-only with bearer auth");
                (None, None)
            }
        }
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = router;
        warn!("no UDS/Named Pipe support on this platform; daemon runs TCP-only");
        (None, None)
    }
}

// =================================================================
// Gossip subscribe task
// =================================================================

/// Command sent to the gossip task from HTTP handlers or the
/// curator subscribe endpoint.
pub enum GossipCmd {
    /// A new announcement was published locally — add it to the
    /// outbox so it gets replayed on NeighborUp.
    Outbox(Vec<u8>),
    /// Broadcast a browse_request to all peers so they replay
    /// their outbox. Triggered by the "Rafraichir" button.
    RequestBrowse,
}

/// Channel sender for [`GossipCmd`]. Stored in [`DaemonHttpState`]
/// so HTTP handlers can push to the gossip outbox.
pub type GossipCmdTx = tokio::sync::mpsc::Sender<GossipCmd>;

/// Spawn the background task that subscribes to the curator
/// gossip topic (non-blocking), stores the sender immediately,
/// and replays the outbox on every NeighborUp event.
#[allow(clippy::too_many_arguments)]
fn spawn_gossip_subscribe_task(
    node: Arc<Node>,
    curator_runtime: CuratorRuntimeHandle,
    browse_aggregator: BrowseAggregatorHandle,
    gossip_sender_slot: GossipSenderHandle,
    pow_verify_cache: Arc<PowVerifyCache>,
    pow_policy: Arc<std::sync::RwLock<RelayPowPolicy>>,
    mut shutdown_rx: oneshot::Receiver<()>,
    bootstrap_peers: Vec<String>,
    mut cmd_rx: tokio::sync::mpsc::Receiver<GossipCmd>,
    pow_solve_cache_cmd: Arc<PowSolveCache>,
    pow_keypair_cmd: Arc<KeyPair>,
    curator_topic: [u8; 32],
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let gossip = GossipClient::new(node.gossip());
        let topic_id = curator_topic_id();

        info!(
            count = bootstrap_peers.len(),
            "gossip: subscribing to topic (non-blocking)"
        );

        let topic = match gossip.subscribe_topic(topic_id, bootstrap_peers).await {
            Ok(t) => t,
            Err(e) => {
                warn!(error = %e, "gossip subscribe failed — task exits");
                return;
            }
        };

        let (sender, mut receiver) = topic.split();
        info!("gossip: sender available, topic subscribed (may be isolated until peers connect)");

        {
            let mut lock = gossip_sender_slot.write().await;
            *lock = Some(sender.clone());
        }

        let mut outbox: Vec<Vec<u8>> = Vec::new();
        let mut neighbor_count: u32 = 0;

        loop {
            tokio::select! {
                ev = receiver.next_event() => {
                    match ev {
                        Ok(Some(GossipEvent::Message { content, delivered_from })) => {
                            debug!(
                                delivered_from = %delivered_from,
                                bytes = content.len(),
                                "gossip message received"
                            );
                            let policy_snapshot = {
                                match pow_policy.read() {
                                    Ok(guard) => guard.clone(),
                                    Err(poisoned) => {
                                        warn!("PoW policy lock poisoned, using default");
                                        poisoned.into_inner().clone()
                                    }
                                }
                            };
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0);
                            let payload: Vec<u8> = match pow_verify_cache
                                .verify_envelope(&content, &policy_snapshot, now)
                            {
                                Ok((_proof, payload)) => payload.to_vec(),
                                Err(e) => {
                                    warn!(
                                        error = %e,
                                        delivered_from = %delivered_from,
                                        bytes = content.len(),
                                        "PoW envelope verify failed — dropping gossip message"
                                    );
                                    continue;
                                }
                            };
                            if publish::is_browse_request(&payload) {
                                debug!(delivered_from = %delivered_from, "browse_request received — replaying outbox");
                                for envelope in &outbox {
                                    if let Err(e) = sender.broadcast(envelope.clone()).await {
                                        debug!(error = %e, "browse_request outbox replay failed");
                                    }
                                }
                            } else if publish::is_project_announcement(&payload) {
                                handle_project_announcement(&browse_aggregator, &payload);
                            } else {
                                handle_announcement(&curator_runtime, &node, &payload).await;
                            }
                        }
                        Ok(Some(GossipEvent::NeighborUp { node_id })) => {
                            neighbor_count += 1;
                            info!(
                                neighbor = %node_id,
                                neighbors = neighbor_count,
                                outbox = outbox.len(),
                                "gossip: neighbor up — replaying outbox"
                            );
                            for envelope in &outbox {
                                if let Err(e) = sender.broadcast(envelope.clone()).await {
                                    debug!(error = %e, "outbox replay broadcast failed");
                                }
                            }
                        }
                        Ok(Some(GossipEvent::NeighborDown { node_id })) => {
                            neighbor_count = neighbor_count.saturating_sub(1);
                            debug!(neighbor = %node_id, neighbors = neighbor_count, "gossip: neighbor down");
                        }
                        Ok(Some(GossipEvent::Lagged)) => {
                            warn!("gossip receiver lagged — some messages dropped");
                        }
                        Ok(None) => {
                            info!("gossip stream ended cleanly");
                            break;
                        }
                        Err(e) => {
                            warn!(error = %e, "gossip next_event error");
                            break;
                        }
                    }
                }
                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(GossipCmd::Outbox(envelope)) => {
                            outbox.push(envelope.clone());
                            if neighbor_count > 0 {
                                if let Err(e) = sender.broadcast(envelope).await {
                                    debug!(error = %e, "outbox broadcast failed");
                                }
                            }
                        }
                        Some(GossipCmd::RequestBrowse) => {
                            let req = publish::browse_request_bytes();
                            if let Ok(envelope) = wrap_payload_with_pow_static(
                                &pow_solve_cache_cmd,
                                &pow_policy,
                                &pow_keypair_cmd,
                                &curator_topic,
                                &req,
                            ) {
                                if let Err(e) = sender.broadcast(envelope).await {
                                    debug!(error = %e, "browse_request broadcast failed");
                                } else {
                                    info!("browse_request broadcast sent to peers");
                                }
                            }
                        }
                        None => {
                            debug!("gossip cmd channel closed");
                            break;
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("gossip task shut down on signal");
                    break;
                }
            }
        }
    })
}

/// Hand a raw gossip message body to the curator runtime and
/// log the outcome.
///
/// Sprint 8 audit C-2 split: the legacy
/// `AnnouncementAttributionMismatch` variant conflated two very
/// different situations — a benign flood of non-subscribed
/// announcements (expected in a healthy gossip network) and a
/// genuine spoofing attempt where the envelope pubkey disagrees
/// with the fetched entry. The gossip handler now logs each
/// case at its own severity:
///
/// - `NotSubscribed` → `debug!`, silent drop.
/// - `EnvelopeMismatch` → `warn!` with both hexes so an
///   operator watching `warn` traffic can investigate the
///   culprit.
/// - `RevisionRollback` → `debug!`, expected in churny networks.
/// - Everything else (blob fetch, parse, signature, persistence)
///   → `warn!`.
async fn handle_announcement(curator_runtime: &CuratorRuntimeHandle, node: &Node, content: &[u8]) {
    // Sprint 9 audit I2-F2 fix: use the throttled variant so the
    // C-4 semaphore actually caps in-flight blob fetches.
    match curator_runtime
        .process_announcement_bytes_throttled(content, node)
        .await
    {
        Ok(entry) => {
            info!(
                curator = %hex::encode(entry.curator_pubkey),
                revision = entry.list.revision,
                "curator list accepted via gossip"
            );
        }
        Err(CuratorRuntimeError::NotSubscribed { curator }) => {
            debug!(curator = %curator, "dropped announcement from non-subscribed curator");
        }
        Err(CuratorRuntimeError::EnvelopeMismatch {
            announcement,
            entry,
        }) => {
            warn!(
                announcement = %announcement,
                entry = %entry,
                "gossip announcement attribution mismatch — a peer is stapling a signed list to a different pubkey"
            );
        }
        Err(CuratorRuntimeError::RevisionRollback { new, stored }) => {
            debug!(new, stored, "ignored revision rollback");
        }
        Err(e) => {
            warn!(error = %e, "failed to process curator announcement");
        }
    }
}

/// Handle a gossip message identified as a project announcement.
/// Parse, validate, and insert into the browse aggregator.
///
/// Sprint 11 Phase A.
fn wrap_payload_with_pow_static(
    solve_cache: &Arc<PowSolveCache>,
    pow_policy: &Arc<std::sync::RwLock<RelayPowPolicy>>,
    keypair: &Arc<KeyPair>,
    topic: &[u8; 32],
    payload: &[u8],
) -> std::result::Result<Vec<u8>, nexus_core_rs::PowGossipError> {
    let policy = match pow_policy.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    };
    let proof = solve_cache.ensure_proof(*topic, keypair.as_ref(), &policy)?;
    nexus_core_rs::PowEnvelope::encode(&proof, payload)
}

fn handle_project_announcement(browse_aggregator: &BrowseAggregatorHandle, content: &[u8]) {
    match publish::ProjectAnnouncement::from_gossip_bytes(content) {
        Ok(ann) => {
            let entry = BrowseEntry {
                project_id: ann.node_id.clone(),
                project_name: ann.project_name,
                category: ann.category,
                description: ann.description,
                curator_pubkey: String::new(),
                curator_name: "Self-published".into(),
                source: BrowseSource::Direct,
                status: BrowseStatus::Unknown,
                last_probed_at: None,
                archive_ticket: ann.archive_ticket,
                archive_hash: None, // Hash not available from gossip announcements; only from local publish
                repo_url: ann.repo_url,
                provenance_hash: ann.provenance_hash,
                is_open_source: ann.is_open_source,
            };
            browse_aggregator.add_direct_entry(entry);
            info!(
                node_id = %ann.node_id,
                "project announcement accepted via gossip"
            );
        }
        Err(e) => {
            warn!(error = %e, "failed to parse project announcement");
        }
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_shell_daemon_core::registry::write_running as raw_write_running;
    use tempfile::tempdir;

    fn mk_opts(root: &Path) -> DaemonStartOptions {
        let config_file = root.join("config.toml");
        let paths = ShellDaemonPaths::resolve(Some(config_file)).expect("resolve paths");
        DaemonStartOptions {
            paths,
            api_host: "127.0.0.1".to_string(),
            api_port: 0,
            daemon_version: "0.1.0-test".to_string(),
            curator: CuratorConfig::default(),
            identity_mode: nexus_core_rs::IdentityMode::Normal,
            cors_origins: vec![],
            web_root: None,
        }
    }

    #[tokio::test]
    async fn start_then_shutdown_roundtrip_cleans_up_running_json() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let running_json = opts.paths.running_json.clone();

        let rt = DaemonRuntime::start(opts).await.expect("start succeeds");
        assert!(
            running_json.exists(),
            "running.json must exist after a successful start"
        );
        assert_ne!(
            rt.bound_addr().port(),
            0,
            "bound_addr must resolve to a real ephemeral port"
        );
        // Curator runtime must be empty at boot.
        assert_eq!(rt.curator_runtime().known_list_count(), 0);

        rt.shutdown().await.expect("shutdown succeeds");
        assert!(
            !running_json.exists(),
            "running.json must be removed after a clean shutdown"
        );
    }

    #[tokio::test]
    async fn second_start_refuses_when_first_still_running() {
        // This unit test works **because** of the hyphen/
        // underscore normalization rule baked into
        // `is_process_alive`. Without it, the test binary
        // (`nexus_shell_daemon-<hash>.exe`) would not match the
        // production `EXPECTED_PROCESS_NAME = "nexus-shell-daemon"`
        // and the singleton enforcement would silently degrade
        // to a no-op inside cargo test. See the Sprint 7 Phase A
        // review note in `registry::EXPECTED_PROCESS_NAME`.
        let tmp = tempdir().expect("tempdir");
        let rt1 = DaemonRuntime::start(mk_opts(tmp.path()))
            .await
            .expect("first start");

        match DaemonRuntime::start(mk_opts(tmp.path())).await {
            Ok(_) => panic!("second start must fail while the first is alive"),
            Err(err) => {
                let message = format!("{err:#}");
                assert!(
                    message.contains("already running"),
                    "error should mention the singleton conflict, got: {message}"
                );
            }
        }

        rt1.shutdown().await.expect("shutdown first");
    }

    #[tokio::test]
    async fn start_overwrites_stale_running_json() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let running_json = opts.paths.running_json.clone();

        // Write a stale running.json with a pid that cannot
        // possibly be this test binary — pid 0 is either
        // invalid or the scheduler idle task on every platform.
        opts.paths.ensure_dirs().unwrap();
        let stale_state = nexus_shell_daemon_core::registry::RunningState {
            schema_version: 1,
            node_id: "0".repeat(64),
            api_host: "127.0.0.1".to_string(),
            api_port: 1,
            pid: 0,
            started_at: "2000-01-01T00:00:00Z".to_string(),
            daemon_version: "0.0.0-stale".to_string(),
        };
        raw_write_running(&stale_state, &running_json).unwrap();

        let rt = DaemonRuntime::start(opts).await.expect("overwrite stale");
        assert!(running_json.exists());
        rt.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn curator_runtime_persists_subscriptions_across_restart() {
        let tmp = tempdir().expect("tempdir");
        let subscriptions_path = {
            let opts = mk_opts(tmp.path());
            opts.paths.subscriptions_json.clone()
        };

        // First boot — subscribe then shutdown.
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        let kp = nexus_core_rs::KeyPair::generate();
        rt1.curator_runtime()
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();
        assert!(subscriptions_path.exists());
        rt1.shutdown().await.unwrap();

        // Second boot against the same fixture — the attention
        // set must be re-populated from the persistence file.
        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.curator_runtime().is_subscribed(&kp.public_bytes()));
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn auto_subscribe_default_curators_at_boot() {
        let tmp = tempdir().expect("tempdir");
        let kp = nexus_core_rs::KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        let mut opts = mk_opts(tmp.path());
        opts.curator.default_curators = vec![hex_key.clone()];

        let rt = DaemonRuntime::start(opts).await.unwrap();
        assert!(
            rt.curator_runtime().is_subscribed(&kp.public_bytes()),
            "default curator must be auto-subscribed at boot"
        );
        rt.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn auto_subscribe_is_idempotent() {
        let tmp = tempdir().expect("tempdir");
        let kp = nexus_core_rs::KeyPair::generate();
        let hex_key = hex::encode(kp.public_bytes());

        // First boot — manually subscribe.
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.curator_runtime().subscribe(&hex_key).unwrap();
        rt1.shutdown().await.unwrap();

        // Second boot — same key in default_curators. Must not
        // double-subscribe or error.
        let mut opts2 = mk_opts(tmp.path());
        opts2.curator.default_curators = vec![hex_key.clone()];
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.curator_runtime().is_subscribed(&kp.public_bytes()));
        // Only one entry in the attention set, not two.
        assert_eq!(rt2.curator_runtime().subscribed_pubkeys_hex().len(), 1);
        rt2.shutdown().await.unwrap();
    }
}
