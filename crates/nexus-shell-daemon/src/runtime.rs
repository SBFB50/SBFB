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

use anyhow::{Context, Result, anyhow};
use nexus_core_rs::{
    GossipClient, GossipEvent, KeyPair, Node, NodeConfig, PowSolveCache, PowVerifyCache,
    RelayPowPolicy, create_node_with_config, load_quorum_resolvers_from_env,
    relay_pow_policy_file_path,
};
use nexus_shell_daemon_core::auth;
use nexus_shell_daemon_core::browse::{
    BrowseAggregator, BrowseAggregatorHandle, BrowseEntry, BrowseSource, BrowseStatus,
};
use nexus_shell_daemon_core::browse_limiter::BrowseRequestLimiter;
use nexus_shell_daemon_core::config::{CuratorConfig, ShellDaemonPaths};
use nexus_shell_daemon_core::iroh_runtime::{
    CuratorRuntime, CuratorRuntimeError, CuratorRuntimeHandle, curator_topic_id,
};
use nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher;
use nexus_shell_daemon_core::publish;
use nexus_shell_daemon_core::registry::{
    self, StaleOutcome, new_running_state, remove_running, write_running,
};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use nexus_trace_core::batch_log::BatchLogProcessor;

use crate::http::{DaemonHttpState, GossipSenderHandle, build_router};

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
    // SAFETY: called during early daemon init, before async runtime.
    unsafe { std::env::remove_var(SBFB_IDENTITY_SECRET_HEX_ENV) };

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
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("failed to set node_key permissions on {}", path.display()))?;
    }
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
    /// 2026-06-05 platform remediation (hotfix #5): result-sync bridge
    /// task + its shutdown watch. Forwards worker-written `result:`
    /// doc entries into the validator loop. Kept on the runtime so the
    /// task lives for the daemon's whole lifetime and joins cleanly.
    result_sync_handle: Option<JoinHandle<()>>,
    result_sync_shutdown: Option<tokio::sync::watch::Sender<bool>>,
    /// Hotfix #5 (maillon A): the on-demand local worker supervisor.
    /// Killed first at shutdown so the worker process never outlives
    /// the daemon (the Job Object / PDEATHSIG covers an abrupt kill).
    local_worker: Option<Arc<crate::local_worker::LocalWorkerSupervisor>>,
    feed_handle: Option<JoinHandle<()>>,
    feed_shutdown: Option<tokio::sync::watch::Sender<bool>>,
    feed_join_handles: Option<Arc<std::sync::Mutex<Vec<JoinHandle<()>>>>>,
    feed_join_shutdown: Option<Arc<tokio::sync::watch::Sender<bool>>>,
    bound_addr: std::net::SocketAddr,
    #[allow(dead_code)]
    revocation_cache: Arc<std::sync::RwLock<nexus_core_rs::RevocationCache>>,
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
        let iroh_data_dir = opts.paths.root.join("iroh");
        let (node, pow_keypair) = match read_optional_identity_env() {
            Some(secret_bytes) => {
                info!("shell daemon using persistent identity from launcher keystore");
                let pow_kp = KeyPair::from_secret_bytes(&secret_bytes);
                let cfg = NodeConfig::default()
                    .with_secret_key(secret_bytes)
                    .with_data_dir(iroh_data_dir.clone());
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
                let cfg = NodeConfig::default()
                    .with_secret_key(secret_bytes)
                    .with_data_dir(iroh_data_dir.clone());
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

        // 6a-2. Sprint 66 Phase D: restore the RevocationCache from
        //       persisted key rotations in SQLite.
        let revocation_cache =
            nexus_shell_daemon_core::key_rotation_handler::shared_revocation_cache();
        {
            let db = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            match db.load_key_rotations() {
                Ok(rows) if !rows.is_empty() => {
                    let tuples: Vec<(String, String, u64, u16, String)> = rows
                        .iter()
                        .map(|r| {
                            (
                                r.old_pubkey.clone(),
                                r.new_pubkey.clone(),
                                r.timestamp,
                                r.transition_days,
                                r.reason.clone(),
                            )
                        })
                        .collect();
                    let applied = nexus_shell_daemon_core::key_rotation_handler::populate_cache(
                        &revocation_cache,
                        &tuples,
                    );
                    info!(
                        total = rows.len(),
                        applied, "RevocationCache restored from SQLite"
                    );
                }
                Ok(_) => {
                    debug!("no persisted key rotations to restore");
                }
                Err(e) => {
                    warn!(error = %e, "failed to load key rotations from DB");
                }
            }
        }

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

        // 6c-2b. 2026-06-05 platform remediation (hotfix #5): result-sync
        //        bridge. The dispatch loop writes `task:` entries; a
        //        worker writes `result:` entries back onto the same doc,
        //        which iroh-docs replicates here. This loop is the
        //        missing producer that forwards replicated `result:`
        //        entries into the validator loop (guardrail → persist →
        //        kudos). Without it `GET /api/v1/tasks/{id}/result`
        //        404'd forever and the Network execute arm timed out.
        let (result_sync_shutdown_tx, result_sync_shutdown_rx) = tokio::sync::watch::channel(false);
        let result_sync_handle = crate::result_sync::spawn_result_subscribe(
            Arc::clone(&project_doc),
            Arc::clone(&node),
            result_event_tx.clone(),
            result_sync_shutdown_rx,
        );

        // 6c-3. Sprint 58 Phase C: create or reopen iroh-docs storage
        //       namespaces for replicated apps. Uses the
        //       storage_namespaces M8 table to persist namespace IDs
        //       across restarts.
        let storage_namespaces = crate::storage_api::new_storage_namespaces();
        {
            let replicated_apps: &[&str] = &["sbfb-ideas"];
            for app_name in replicated_apps {
                match boot_storage_namespace(&docs_client, &coordinator_db, app_name, doc_author)
                    .await
                {
                    Ok(ns_state) => {
                        info!(
                            app = %app_name,
                            doc_id = %ns_state.doc.id(),
                            "storage namespace ready"
                        );
                        let ns_arc = Arc::new(ns_state);
                        crate::storage_api::spawn_storage_subscribe(
                            app_name.to_string(),
                            Arc::clone(&ns_arc),
                        );
                        storage_namespaces
                            .write()
                            .await
                            .insert(app_name.to_string(), ns_arc);
                    }
                    Err(e) => {
                        warn!(app = %app_name, error = %e, "failed to boot storage namespace");
                    }
                }
            }
        }

        // 6c-4. Sprint 62 Phase B: create or reopen iroh-docs feed
        //       namespace for public feed P2P sync. Reuses the M8
        //       storage_namespaces table with key "sbfb-feed".
        let feed_rate_limiter =
            Arc::new(nexus_shell_daemon_core::feed_limiter::FeedRateLimiter::new());
        let (feed_shutdown_tx, feed_shutdown_rx) = tokio::sync::watch::channel(false);
        let (feed_sync_state, feed_handle) =
            match boot_feed_namespace(&docs_client, &coordinator_db, doc_author).await {
                Ok(fs) => {
                    info!(
                        doc_id = %fs.doc.id(),
                        "feed sync namespace ready"
                    );
                    let fs_arc = Arc::new(fs);
                    let handle = crate::feed_sync::spawn_feed_subscribe(
                        Arc::clone(&fs_arc),
                        Arc::clone(&coordinator_db),
                        Arc::clone(&node),
                        Arc::clone(&feed_rate_limiter),
                        feed_shutdown_rx,
                    );
                    (Some(fs_arc), Some(handle))
                }
                Err(e) => {
                    warn!(error = %e, "failed to boot feed sync namespace");
                    (None, None)
                }
            };

        // 6c-5. Sprint 66 Phase C: republish SQLite feed entries to
        //       iroh-docs at boot (one-shot, synchronous before HTTP).
        if let Some(ref fs) = feed_sync_state {
            let entries_result = {
                let db = coordinator_db
                    .lock()
                    .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
                nexus_coordinator_rs::public_feed::replay_all(&db)
            };
            match entries_result {
                Ok(entries) => {
                    let mut published = 0u64;
                    for entry in &entries {
                        if let Err(e) =
                            crate::feed_sync::publish_feed_entry_to_docs(fs, entry).await
                        {
                            warn!(seq = entry.seq, error = %e, "feed republish failed");
                        } else {
                            published += 1;
                        }
                    }
                    info!(
                        total = entries.len(),
                        published, "feed entries republished to iroh-docs at boot"
                    );
                }
                Err(e) => warn!(error = %e, "feed replay_all failed, skipping republish"),
            }
        }

        // 6c-5b. Sprint 66 Phase D: orphan recovery — detect entries
        //        in SQLite but missing from iroh-docs and republish.
        if let Some(ref fs) = feed_sync_state {
            match fs.doc.get_many_by_prefix("feed/").await {
                Ok(doc_entries) => {
                    let present_keys: std::collections::HashSet<String> = doc_entries
                        .iter()
                        .filter_map(|e| String::from_utf8(e.key().to_vec()).ok())
                        .collect();

                    let entries_result = {
                        let db = coordinator_db
                            .lock()
                            .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
                        nexus_coordinator_rs::public_feed::replay_all(&db)
                    };
                    if let Ok(entries) = entries_result {
                        let entry_hash_set: std::collections::HashSet<&str> =
                            entries.iter().map(|e| e.entry_hash.as_str()).collect();
                        let mut orphan_count = 0u64;
                        let mut recovered = 0u64;
                        for entry in &entries {
                            let key =
                                crate::feed_sync::format_feed_key(&entry.author_pubkey, entry.seq);
                            if present_keys.contains(&key) {
                                continue;
                            }
                            orphan_count += 1;
                            let is_genesis = entry.prev_hash.chars().all(|c| c == '0');
                            if !is_genesis && !entry_hash_set.contains(entry.prev_hash.as_str()) {
                                warn!(
                                    seq = entry.seq,
                                    prev_hash = %entry.prev_hash,
                                    "orphan recovery: skipping broken chain tail"
                                );
                                continue;
                            }
                            if let Err(e) =
                                crate::feed_sync::publish_feed_entry_to_docs(fs, entry).await
                            {
                                warn!(seq = entry.seq, error = %e, "orphan recovery: republish failed");
                            } else {
                                recovered += 1;
                            }
                        }
                        if orphan_count > 0 {
                            info!(
                                orphans = orphan_count,
                                recovered, "feed orphan recovery completed"
                            );
                        }
                    }
                }
                Err(e) => {
                    warn!(error = %e, "orphan recovery: iroh-docs query failed, skipping");
                }
            }
        }

        // 6c-7. Sprint 67 Phase B: rebuild FTS5 search index from feed.
        {
            let db = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            match nexus_coordinator_rs::search::rebuild_from_feed(&db) {
                Ok(n) => info!(indexed = n, "search index rebuilt from feed at boot"),
                // H.1 (Sprint 74 Phase D, carry audit S73): boot recovery must be
                // LOUD, not a warn swallowed in the log. A failed FTS5 rebuild
                // leaves search stale/empty until the next deploy reindex; surface
                // it as an error so an operator notices (the index is fully
                // reconstructible from public_feed, so this is recoverable).
                Err(e) => tracing::error!(
                    error = %e,
                    "BOOT RECOVERY FAILED: search index rebuild_from_feed errored — \
                     search is stale/empty until the next successful deploy reindex"
                ),
            }
        }

        // 6c-6. Sprint 66 Phase C: feed_join shutdown channel +
        //       shared handle Vec for clean join at shutdown.
        let (feed_join_shutdown_tx, _) = tokio::sync::watch::channel(false);
        let feed_join_shutdown = Arc::new(feed_join_shutdown_tx);
        let feed_join_handles: Arc<std::sync::Mutex<Vec<JoinHandle<()>>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));

        // 6d. Build gossip command channel + shared HTTP state +
        //     spawn the serve task.
        let (gossip_cmd_tx, gossip_cmd_rx) = tokio::sync::mpsc::channel::<GossipCmd>(64);
        // Hotfix #5 (maillon A): the on-demand local worker supervisor.
        // Held both on the runtime (for shutdown) and in the HTTP state
        // (so the task submit handler can spawn the worker lazily).
        let local_worker = Arc::new(crate::local_worker::LocalWorkerSupervisor::new());
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
            local_worker: Arc::clone(&local_worker),
            app_storage: {
                let guard = coordinator_db.lock().unwrap();
                crate::storage_api::load_app_storage_from_db(&guard)
            },
            storage_namespaces: Arc::clone(&storage_namespaces),
            storage_write_limiter: Arc::new(
                nexus_shell_daemon_core::storage_limiter::StorageWriteLimiter::new(),
            ),
            feed_sync_state,
            feed_rate_limiter,
            feed_join_handles: Arc::clone(&feed_join_handles),
            feed_join_shutdown: Arc::clone(&feed_join_shutdown),
            preview_store: nexus_shell_daemon_core::preview::PreviewStore::new(
                nexus_shell_daemon_core::preview::DEFAULT_TTL,
            ),
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

        {
            let limiter = Arc::clone(&http_state.storage_write_limiter);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    limiter.retain_recent();
                }
            });
        }

        {
            let limiter = Arc::clone(&http_state.feed_rate_limiter);
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
                loop {
                    interval.tick().await;
                    limiter.retain_recent();
                }
            });
        }

        {
            let store = http_state.preview_store.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
                loop {
                    interval.tick().await;
                    store.evict_expired();
                }
            });
        }

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
        let initial_outbox = {
            let guard = coordinator_db
                .lock()
                .map_err(|e| anyhow::anyhow!("coordinator DB lock failed: {e}"))?;
            guard
                .load_outbox()
                .map_err(|e| anyhow::anyhow!("outbox load failed: {e}"))?
        };
        let gossip_handle = spawn_gossip_subscribe_task(GossipTaskConfig {
            node: Arc::clone(&node),
            curator_runtime: Arc::clone(&curator_runtime),
            browse_aggregator: Arc::clone(&browse_aggregator),
            gossip_sender_slot: Arc::clone(&gossip_sender),
            pow_verify_cache: Arc::clone(&pow_verify_cache),
            pow_policy: Arc::clone(&pow_policy),
            shutdown_rx: gossip_shutdown_rx,
            bootstrap_peers,
            cmd_rx: gossip_cmd_rx,
            pow_solve_cache: Arc::clone(&pow_solve_cache),
            pow_keypair: Arc::clone(&pow_keypair),
            curator_topic,
            coordinator_db: Arc::clone(&coordinator_db),
            initial_outbox,
        });

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
            result_sync_handle: Some(result_sync_handle),
            result_sync_shutdown: Some(result_sync_shutdown_tx),
            local_worker: Some(local_worker),
            feed_handle,
            feed_shutdown: Some(feed_shutdown_tx),
            feed_join_handles: Some(feed_join_handles),
            feed_join_shutdown: Some(feed_join_shutdown),
            bound_addr,
            revocation_cache,
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

    #[allow(dead_code)]
    pub fn revocation_cache(&self) -> &Arc<std::sync::RwLock<nexus_core_rs::RevocationCache>> {
        &self.revocation_cache
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
        // Hotfix #5: stop the on-demand worker first — it is a child
        // process that syncs the project doc, so it must go before the
        // doc subscriptions and the iroh node close.
        if let Some(lw) = self.local_worker.take() {
            lw.shutdown().await;
        }

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

        if let Some(tx) = self.feed_shutdown.take() {
            let _ = tx.send(true);
        }
        if let Some(mut handle) = self.feed_handle.take() {
            if let Err(e) = (&mut handle).await {
                warn!(error = %e, "feed subscribe task join failed");
            }
        }

        if let Some(sender) = self.feed_join_shutdown.take() {
            let _ = sender.send(true);
        }
        if let Some(handles_arc) = self.feed_join_handles.take() {
            let handles: Vec<_> = {
                let mut guard = handles_arc.lock().unwrap_or_else(|e| e.into_inner());
                std::mem::take(&mut *guard)
            };
            for mut h in handles {
                if let Err(e) = (&mut h).await {
                    warn!(error = %e, "feed join task join failed");
                }
            }
        }

        // Hotfix #5: stop the result-sync bridge before the dispatch
        // loop and the node — it subscribes to the project doc, so it
        // must release that subscription ahead of the iroh node close.
        if let Some(tx) = self.result_sync_shutdown.take() {
            let _ = tx.send(true);
        }
        if let Some(mut handle) = self.result_sync_handle.take() {
            if let Err(e) = (&mut handle).await {
                warn!(error = %e, "result sync task join failed");
            }
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

struct GossipTaskConfig {
    node: Arc<Node>,
    curator_runtime: CuratorRuntimeHandle,
    browse_aggregator: BrowseAggregatorHandle,
    gossip_sender_slot: GossipSenderHandle,
    pow_verify_cache: Arc<PowVerifyCache>,
    pow_policy: Arc<std::sync::RwLock<RelayPowPolicy>>,
    shutdown_rx: oneshot::Receiver<()>,
    bootstrap_peers: Vec<String>,
    cmd_rx: tokio::sync::mpsc::Receiver<GossipCmd>,
    pow_solve_cache: Arc<PowSolveCache>,
    pow_keypair: Arc<KeyPair>,
    curator_topic: [u8; 32],
    coordinator_db: std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    initial_outbox: Vec<Vec<u8>>,
}

/// Spawn the background task that subscribes to the curator
/// gossip topic (non-blocking), stores the sender immediately,
/// and replays the outbox on every NeighborUp event.
fn spawn_gossip_subscribe_task(cfg: GossipTaskConfig) -> JoinHandle<()> {
    let GossipTaskConfig {
        node,
        curator_runtime,
        browse_aggregator,
        gossip_sender_slot,
        pow_verify_cache,
        pow_policy,
        mut shutdown_rx,
        bootstrap_peers,
        mut cmd_rx,
        pow_solve_cache,
        pow_keypair,
        curator_topic,
        coordinator_db,
        initial_outbox,
    } = cfg;
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

        let mut outbox: Vec<Vec<u8>> = initial_outbox;
        if !outbox.is_empty() {
            info!(
                entries = outbox.len(),
                "gossip: loaded persisted outbox from DB"
            );
        }
        // Remediation #7 (Browse boot-restore): the in-memory Browse
        // aggregator starts empty on every boot, so a node's OWN published
        // apps vanish from its Browse after a restart even though their
        // announcements persist in the gossip outbox (only the outbox + feed
        // survive a restart, never the aggregator). Re-ingest each persisted
        // project announcement through the same handler the live gossip path
        // uses: it repopulates the aggregator AND, via index_browse_entry,
        // re-indexes the search corpus with the real project_name (the feed's
        // ReleasePublished op carries none, which is why search-by-name was
        // empty too). Restored self entries (our own node_id) render Reachable
        // through the aggregate() self-branch. We decode with
        // PowEnvelope::decode (structural, no PoW re-verification) rather than
        // verify_envelope: these are our OWN trusted envelopes, and a
        // difficulty-policy bump since they were minted must NOT drop them.
        // `restored` counts announcements re-ingested (one per project
        // announcement envelope), not distinct cards — several envelopes for
        // the same project_id collapse to one card via add_direct_entry dedup.
        let restored =
            restore_browse_from_outbox(&browse_aggregator, &coordinator_db, &node, &outbox);
        if restored > 0 {
            info!(
                restored,
                "gossip: restored project announcements from persisted outbox"
            );
        }
        let mut neighbor_count: u32 = 0;
        let browse_limiter = BrowseRequestLimiter::new();
        let republish_delay = tokio::time::sleep(jittered_republish_duration());
        tokio::pin!(republish_delay);
        let mut retain_interval = tokio::time::interval(std::time::Duration::from_secs(60));
        retain_interval.tick().await; // consume the immediate first tick

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
                                if !browse_limiter.check_peer(&delivered_from) {
                                    debug!(
                                        delivered_from = %delivered_from,
                                        "browse_request rate-limited — dropping"
                                    );
                                    continue;
                                }
                                debug!(delivered_from = %delivered_from, "browse_request received — replaying outbox");
                                // Phase D OFF gate (same as NeighborUp/republish).
                                let disabled = load_disabled_keep_online(&coordinator_db);
                                for envelope in &outbox {
                                    if !keep_online_allows_rebroadcast(envelope, &disabled) {
                                        continue;
                                    }
                                    if let Err(e) = sender.broadcast(envelope.clone()).await {
                                        debug!(error = %e, "browse_request outbox replay failed");
                                    }
                                }
                            } else if publish::is_project_announcement(&payload) {
                                handle_project_announcement(
                                    &browse_aggregator,
                                    &coordinator_db,
                                    &node,
                                    &payload,
                                );
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
                            // Sprint 74 Phase D: do not re-broadcast apps the node
                            // has turned OFF (keep_online disabled). Fast path: an
                            // empty disabled set replays all without decoding.
                            let disabled = load_disabled_keep_online(&coordinator_db);
                            for envelope in &outbox {
                                if !keep_online_allows_rebroadcast(envelope, &disabled) {
                                    continue;
                                }
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
                            // Best-effort DB persistence: gossip broadcast is the
                            // primary transport, DB insert is boot-recovery only.
                            // A failed insert still allows in-memory replay + broadcast.
                            if let Ok(guard) = coordinator_db.lock() {
                                if let Err(e) = guard.insert_outbox(&envelope) {
                                    warn!(error = %e, "outbox DB insert failed");
                                }
                            }
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
                                &pow_solve_cache,
                                &pow_policy,
                                &pow_keypair,
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
                _ = &mut republish_delay => {
                    if neighbor_count > 0 && !outbox.is_empty() {
                        // Phase D OFF gate (same as NeighborUp/browse_request) — an
                        // app turned OFF must stop diffusing on EVERY replay path.
                        let disabled = load_disabled_keep_online(&coordinator_db);
                        for envelope in &outbox {
                            if !keep_online_allows_rebroadcast(envelope, &disabled) {
                                continue;
                            }
                            if let Err(e) = sender.broadcast(envelope.clone()).await {
                                debug!(error = %e, "periodic republish broadcast failed");
                            }
                        }
                        debug!(entries = outbox.len(), neighbors = neighbor_count, "periodic republish completed");
                    }
                    republish_delay.as_mut().reset(tokio::time::Instant::now() + jittered_republish_duration());
                }
                _ = retain_interval.tick() => {
                    browse_limiter.retain_recent();
                }
                _ = &mut shutdown_rx => {
                    info!("gossip task shut down on signal");
                    break;
                }
            }
        }
    })
}

fn jittered_republish_duration() -> std::time::Duration {
    use rand::Rng;
    let secs = rand::thread_rng().gen_range(30..=60);
    std::time::Duration::from_secs(secs)
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

fn handle_project_announcement(
    browse_aggregator: &BrowseAggregatorHandle,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    node: &Node,
    content: &[u8],
) {
    match publish::ProjectAnnouncement::from_gossip_bytes(content) {
        Ok(ann) => {
            // Per-app identity from the announcement (blake3(name)). Fall back to
            // node_id only for a legacy announcement that predates the field, so
            // a node hosting N apps shows N distinct cards instead of collapsing
            // them onto one node_id-keyed card.
            let project_id = if ann.project_id.is_empty() {
                ann.node_id.clone()
            } else {
                ann.project_id.clone()
            };
            // Derive the archive hash from the ticket: the hash itself never
            // travels on the announcement (only the ticket does), so without this
            // the shell sees archive_hash=None, marks the app as having no archive
            // (BrowsedProject hasArchive), and never opens it. blob-serve resolves
            // the ticket back from the aggregator to P2P-download the zip.
            let archive_hash = ann
                .archive_ticket
                .as_deref()
                .and_then(crate::http::archive_hash_from_ticket);
            // Remediation #6 (freshness): seed the announcing node's address —
            // carried inside the archive ticket — into our endpoint's address
            // book so the reachability probe (and a later blob fetch) can dial
            // node_id without waiting on a pkarr round-trip. This reconciles
            // gossip discovery with the iroh service layer; mirrors
            // blobs.rs::fetch_ticket.
            if let Some(ticket_str) = ann.archive_ticket.as_deref() {
                use std::str::FromStr;
                match iroh_blobs::ticket::BlobTicket::from_str(ticket_str) {
                    Ok(ticket) => {
                        let (addr, _hash, _format) = ticket.into_parts();
                        node.memory_lookup().add_endpoint_info(addr);
                    }
                    Err(e) => {
                        debug!(error = %e, "could not parse archive ticket for addr-seed");
                    }
                }
            }
            let entry = BrowseEntry {
                project_id,
                // Remediation #6: the hosting node's dialable identity. The
                // freshness probe dials this, NOT project_id (= blake3(name)).
                node_id: Some(ann.node_id.clone()),
                project_name: ann.project_name,
                category: ann.category,
                description: ann.description,
                curator_pubkey: String::new(),
                curator_name: "Self-published".into(),
                source: BrowseSource::Direct,
                status: BrowseStatus::Unknown,
                last_probed_at: None,
                archive_ticket: ann.archive_ticket,
                archive_hash,
                repo_url: ann.repo_url,
                provenance_hash: ann.provenance_hash,
                is_open_source: ann.is_open_source,
            };
            // Index the gossiped app for search (the gossip path deferred from the
            // search hotfix). Best-effort: a search-index hiccup must never drop
            // the discovered browse entry.
            if let Ok(db) = coordinator_db.lock() {
                crate::http::index_browse_entry(&db, &entry);
            }
            browse_aggregator.add_direct_entry(entry);
            // Path-agnostic: this handler ingests both live gossip arrivals
            // and boot-restored outbox entries (remediation #7).
            info!(
                node_id = %ann.node_id,
                "project announcement ingested"
            );
        }
        Err(e) => {
            warn!(error = %e, "failed to parse project announcement");
        }
    }
}

/// Remediation #7 (Browse boot-restore): repopulate the in-memory Browse
/// aggregator from the node's own persisted gossip outbox at startup.
///
/// The aggregator is in-memory and starts empty on every boot, so a node's
/// own published apps would otherwise disappear from its Browse after a
/// restart — only the outbox (and the feed) survive. Each outbox entry is a
/// PoW-wrapped envelope; we decode structurally with
/// [`nexus_core_rs::PowEnvelope::decode`] (no PoW re-verification: these are
/// our own trusted envelopes, and a difficulty-policy bump since they were
/// minted must not drop them) and re-ingest every project announcement through
/// [`handle_project_announcement`], which repopulates the aggregator and
/// re-indexes the search corpus with the real `project_name`. Returns the
/// number of project announcements restored. Idempotent: `add_direct_entry`
/// dedups by `project_id` and the search upsert is `INSERT OR REPLACE`.
///
/// Note: after a `daemon.key` identity rotation, restored entries carry the
/// pre-rotation `node_id`, so they probe as remote instead of taking the
/// self-branch — benign, since rotation also invalidates the old
/// announcements (they are re-published under the new identity).
/// Sprint 74 Phase D: should this outbox envelope still be re-broadcast to peers?
/// Apps the node has turned OFF (`keep_online` disabled) are skipped; everything
/// else — non-project envelopes (curator lists) and undecodable bytes — is
/// replayed, so a decode hiccup never silently drops diffusion. Fast path: an
/// empty disabled set replays all without decoding (the common case).
fn keep_online_allows_rebroadcast(
    envelope: &[u8],
    disabled: &std::collections::HashSet<String>,
) -> bool {
    if disabled.is_empty() {
        return true;
    }
    let Ok((_proof, payload)) = nexus_core_rs::PowEnvelope::decode(envelope) else {
        return true;
    };
    match publish::ProjectAnnouncement::from_gossip_bytes(payload) {
        Ok(ann) => {
            // Per-app id (blake3(name)); legacy-empty falls back to node_id, the
            // same key the toggle / disabled list uses.
            let pid = if ann.project_id.is_empty() {
                ann.node_id
            } else {
                ann.project_id
            };
            !disabled.contains(&pid)
        }
        Err(_) => true,
    }
}

/// Load the set of `project_id`s this node has turned OFF (keep_online disabled),
/// for the outbox re-broadcast gate. Best-effort (R6): a poisoned lock or DB
/// error collapses to an EMPTY set = replay-all (the safe default), never an
/// abort. Shared by ALL outbox->peer replay sites (NeighborUp, browse_request,
/// periodic republish) so the OFF gate cannot be bypassed by one path.
fn load_disabled_keep_online(
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
) -> std::collections::HashSet<String> {
    // R6: on ANY failure fall back to an empty set (= replay all, the safe
    // default) — but WARN, never swallow silently, so a real DB/lock fault is
    // observable instead of looking like "nothing is disabled".
    match coordinator_db.lock() {
        Ok(db) => match db.list_keep_online_disabled() {
            Ok(v) => v.into_iter().collect(),
            Err(e) => {
                warn!(error = %e, "keep_online disabled-set read failed; replaying all (R6 fallback)");
                std::collections::HashSet::new()
            }
        },
        Err(_) => {
            warn!("coordinator DB lock poisoned; replaying all keep_online (R6 fallback)");
            std::collections::HashSet::new()
        }
    }
}

fn restore_browse_from_outbox(
    browse_aggregator: &BrowseAggregatorHandle,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    node: &Node,
    outbox: &[Vec<u8>],
) -> usize {
    let mut restored = 0usize;
    for envelope in outbox {
        match nexus_core_rs::PowEnvelope::decode(envelope) {
            Ok((_proof, payload)) => {
                if publish::is_project_announcement(payload) {
                    handle_project_announcement(browse_aggregator, coordinator_db, node, payload);
                    restored += 1;
                }
            }
            Err(e) => {
                debug!(error = %e, "browse restore: skipping undecodable outbox envelope");
            }
        }
    }
    restored
}

/// Boot or reopen an iroh-docs storage namespace for a replicated
/// app. Checks the M8 `storage_namespaces` table for a persisted
/// NamespaceId. If found, reopens; otherwise creates a new namespace,
/// generates a Write ticket, and persists both.
async fn boot_storage_namespace(
    docs_client: &nexus_core_rs::docs::DocsClient,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    app_name: &str,
    author: nexus_core_rs::docs::DocsAuthorId,
) -> Result<crate::storage_api::StorageNamespaceState> {
    use std::sync::atomic::AtomicU64;

    let existing = {
        let db = coordinator_db
            .lock()
            .map_err(|e| anyhow!("coordinator DB lock failed: {e}"))?;
        db.get_storage_namespace(app_name)
            .map_err(|e| anyhow!("failed to read storage namespace for {app_name}: {e}"))?
    };

    let (doc, ticket_str) = match existing {
        Some(row) => {
            let bytes: [u8; 32] = row
                .namespace_id
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("invalid namespace_id length in DB"))?;
            let ns_id = nexus_core_rs::docs::DocsNamespaceId::from(bytes);
            match docs_client.open_doc(ns_id).await? {
                Some(doc) => {
                    let ticket_str = match row.doc_ticket {
                        Some(t) => t,
                        None => {
                            let ticket = doc.share_write().await?;
                            let t = ticket.to_string();
                            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                            db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&t))
                                .map_err(|e| {
                                    anyhow!("failed to persist storage namespace ticket: {e}")
                                })?;
                            t
                        }
                    };
                    (doc, ticket_str)
                }
                None => {
                    warn!(
                        app = %app_name,
                        "storage namespace in DB but missing from iroh — recreating"
                    );
                    let doc = docs_client.create_doc().await?;
                    let ticket = doc.share_write().await?;
                    let ticket_str = ticket.to_string();
                    let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                    db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&ticket_str))
                        .map_err(|e| {
                            anyhow!("failed to persist recreated storage namespace: {e}")
                        })?;
                    (doc, ticket_str)
                }
            }
        }
        None => {
            let doc = docs_client.create_doc().await?;
            let ticket = doc.share_write().await?;
            let ticket_str = ticket.to_string();
            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
            db.set_storage_namespace(app_name, doc.id().as_bytes(), Some(&ticket_str))
                .map_err(|e| anyhow!("failed to persist new storage namespace: {e}"))?;
            (doc, ticket_str)
        }
    };

    Ok(crate::storage_api::StorageNamespaceState {
        doc: Arc::new(doc),
        author,
        ticket: ticket_str,
        version: AtomicU64::new(0),
    })
}

/// Boot or reopen the iroh-docs namespace for the public feed
/// P2P sync. Mirrors `boot_storage_namespace` but produces a
/// `FeedSyncState` without the version counter (feed dedup is
/// hash-based, not version-based).
async fn boot_feed_namespace(
    docs_client: &nexus_core_rs::docs::DocsClient,
    coordinator_db: &std::sync::Arc<std::sync::Mutex<nexus_coordinator_rs::db::CoordinatorDb>>,
    author: nexus_core_rs::docs::DocsAuthorId,
) -> Result<crate::feed_sync::FeedSyncState> {
    let feed_key = crate::feed_sync::FEED_NAMESPACE_KEY;

    let existing = {
        let db = coordinator_db
            .lock()
            .map_err(|e| anyhow!("coordinator DB lock failed: {e}"))?;
        db.get_storage_namespace(feed_key)
            .map_err(|e| anyhow!("failed to read feed namespace: {e}"))?
    };

    let (doc, ticket_str) = match existing {
        Some(row) => {
            let bytes: [u8; 32] = row
                .namespace_id
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("invalid namespace_id length in DB"))?;
            let ns_id = nexus_core_rs::docs::DocsNamespaceId::from(bytes);
            match docs_client.open_doc(ns_id).await? {
                Some(doc) => {
                    let ticket_str = match row.doc_ticket {
                        Some(t) => t,
                        None => {
                            let ticket = doc.share_write().await?;
                            let t = ticket.to_string();
                            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                            db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&t))
                                .map_err(|e| {
                                    anyhow!("failed to persist feed namespace ticket: {e}")
                                })?;
                            t
                        }
                    };
                    (doc, ticket_str)
                }
                None => {
                    warn!("feed namespace in DB but missing from iroh — recreating");
                    let doc = docs_client.create_doc().await?;
                    let ticket = doc.share_write().await?;
                    let ticket_str = ticket.to_string();
                    let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
                    db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&ticket_str))
                        .map_err(|e| anyhow!("failed to persist recreated feed namespace: {e}"))?;
                    (doc, ticket_str)
                }
            }
        }
        None => {
            let doc = docs_client.create_doc().await?;
            let ticket = doc.share_write().await?;
            let ticket_str = ticket.to_string();
            let db = coordinator_db.lock().map_err(|e| anyhow!("{e}"))?;
            db.set_storage_namespace(feed_key, doc.id().as_bytes(), Some(&ticket_str))
                .map_err(|e| anyhow!("failed to persist new feed namespace: {e}"))?;
            (doc, ticket_str)
        }
    };

    Ok(crate::feed_sync::FeedSyncState {
        doc: Arc::new(doc),
        author,
        ticket: ticket_str,
    })
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn gossip_announcement_populates_archive_hash_from_ticket() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        // Node hosting the zip (provides the ticket address).
        let node = nexus_core_rs::create_node().await.unwrap();
        let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
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

        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Remote App"));
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "Remote App".into(),
            "tools".into(),
            "d".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        // archive_hash is derived from the ticket so the shell knows it HAS an
        // archive (the hash never travels on the announcement itself).
        let entry = agg.get_direct_entry(&pid).expect("entry present");
        assert_eq!(entry.archive_hash, Some(hash_hex));
        assert!(entry.archive_ticket.is_some());
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_announcement_uses_per_app_id_and_indexes() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "a".repeat(64);
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Cool App"));
        let ann = ProjectAnnouncement::new(
            node_id.clone(),
            "Cool App".into(),
            "tools".into(),
            "p2p tool".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        // Browse card keyed by per-app project_id, not node_id.
        assert_eq!(agg.direct_entry_count(), 1);
        assert!(
            agg.get_direct_entry(&pid).is_some(),
            "card keyed by blake3 id"
        );
        assert!(
            agg.get_direct_entry(&node_id).is_none(),
            "card is NOT keyed by node_id"
        );
        // And searchable through the gossip indexing path (deferred from #2).
        let (results, total) =
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Cool", 20, 0).unwrap();
        assert_eq!(total, 1);
        assert_eq!(results[0].project_id, pid);
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_two_apps_same_node_are_distinct_cards() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "b".repeat(64);
        for name in ["First App", "Second App"] {
            let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(name.as_bytes()));
            let ann = ProjectAnnouncement::new(
                node_id.clone(),
                name.into(),
                "tools".into(),
                "desc".into(),
                vec![],
            )
            .with_project_id(pid);
            super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        }
        // Same node, two apps -> two distinct cards (not collapsed on node_id).
        assert_eq!(agg.direct_entry_count(), 2);
        assert_eq!(
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "First", 20, 0)
                .unwrap()
                .1,
            1
        );
        assert_eq!(
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Second", 20, 0)
                .unwrap()
                .1,
            1
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn gossip_legacy_announcement_falls_back_to_node_id() {
        use nexus_shell_daemon_core::browse::BrowseAggregator;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;
        let node = nexus_core_rs::create_node().await.unwrap();
        let agg = std::sync::Arc::new(BrowseAggregator::new());
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        let node_id = "c".repeat(64);
        // No with_project_id() -> empty -> receiver falls back to node_id.
        let ann = ProjectAnnouncement::new(
            node_id.clone(),
            "Legacy App".into(),
            "tools".into(),
            "desc".into(),
            vec![],
        );
        super::handle_project_announcement(&agg, &db, &node, &ann.to_gossip_bytes().unwrap());
        assert!(
            agg.get_direct_entry(&node_id).is_some(),
            "legacy announcement falls back to node_id key"
        );
        node.shutdown().await.ok();
    }

    #[tokio::test]
    async fn freshness_probe_marks_gossiped_remote_app_reachable_e2e() {
        // Remediation #6 real-frontier gate (PATTERNS §P57): a genuine
        // two-node path with NO mock at the discovery<->service boundary.
        // node_a hosts an app and gossips a ProjectAnnouncement carrying a
        // real BlobTicket; node_b ingests it through the production handler
        // (which seeds node_a's addr from the ticket and stores node_a's
        // node_id on the direct entry), then aggregates /browse. The card
        // must flip Unknown -> Reachable, proving the freshness probe dials
        // the hosting node_id — not the per-app project_id (blake3(name)).
        use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseStatus, DEFAULT_PROBE_TTL};
        use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;

        let node_a = nexus_core_rs::create_node().await.unwrap(); // remote host
        let node_b = nexus_core_rs::create_node().await.unwrap(); // runs /browse

        // node_a mints a real blob + ticket (the ticket carries a_addr).
        let blobs_a = nexus_core_rs::BlobsClient::new(node_a.blobs_store());
        let hash = blobs_a.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
        let a_addr = nexus_core_rs::DiscoveryClient::new(node_a.endpoint())
            .my_endpoint_addr()
            .await
            .unwrap();
        let ticket = iroh_blobs::ticket::BlobTicket::new(
            a_addr,
            iroh_blobs::Hash::from_bytes(hash),
            iroh_blobs::BlobFormat::Raw,
        )
        .to_string();

        // Per-app project_id is blake3(name) — DISTINCT from node_a's id.
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Remote E2E App"));
        assert_ne!(pid, node_a.node_id(), "project_id must differ from node_id");
        let ann = ProjectAnnouncement::new(
            node_a.node_id(),
            "Remote E2E App".into(),
            "tools".into(),
            "discovered over gossip".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);

        let agg = std::sync::Arc::new(BrowseAggregator::with_durations(
            DEFAULT_PROBE_TTL,
            std::time::Duration::from_secs(5),
        ));
        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));

        // Production ingest on node_b: seeds a_addr from the ticket and
        // stores node_a's node_id on the direct entry.
        super::handle_project_announcement(&agg, &db, &node_b, &ann.to_gossip_bytes().unwrap());

        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node_b).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].status,
            BrowseStatus::Reachable,
            "a live gossiped remote app must probe Reachable end-to-end"
        );
        assert_eq!(out[0].project_id, pid, "per-app project_id preserved");
        assert!(out[0].last_probed_at.is_some());

        node_a.shutdown().await.ok();
        node_b.shutdown().await.ok();
    }

    #[tokio::test]
    async fn browse_boot_restore_repopulates_aggregator_from_outbox_e2e() {
        // Remediation #7 real-frontier test (PATTERNS §P57): a node's OWN
        // published app must reappear in its Browse after a daemon restart.
        // We persist a real PoW-wrapped ProjectAnnouncement through the
        // coordinator DB outbox, load it back exactly as boot does
        // (load_outbox), run the restore, and assert the aggregator now
        // carries the card with its real name — and, because the announcement
        // node_id is our own, Reachable. No mock at the persistence frontier:
        // real encode -> real DB round-trip -> real decode -> real ingest.
        use nexus_shell_daemon_core::browse::{BrowseAggregator, BrowseStatus};
        use nexus_shell_daemon_core::iroh_runtime::CuratorRuntime;
        use nexus_shell_daemon_core::publish::ProjectAnnouncement;

        let node = nexus_core_rs::create_node().await.unwrap();

        // Our own announcement, with a real ticket minted from our store.
        let blobs = nexus_core_rs::BlobsClient::new(node.blobs_store());
        let hash = blobs.add_bytes(b"zip-bytes".to_vec()).await.unwrap();
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
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"My Restored App"));
        let ann = ProjectAnnouncement::new(
            node.node_id(),
            "My Restored App".into(),
            "tools".into(),
            "persisted across reboot".into(),
            vec![],
        )
        .with_project_id(pid.clone())
        .with_archive_ticket(ticket);
        let payload = ann.to_gossip_bytes().unwrap();

        // Wrap in a real PoW envelope (difficulty 1 -> instant solve; the
        // restore decodes structurally without re-verifying, so the proof
        // difficulty is irrelevant to the path under test) and persist it the
        // way the daemon does.
        let kp = nexus_core_rs::KeyPair::generate();
        let policy = nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        };
        let solve = nexus_core_rs::PowSolveCache::new();
        let proof = solve.ensure_proof([7u8; 32], &kp, &policy).unwrap();
        let envelope = nexus_core_rs::PowEnvelope::encode(&proof, &payload).unwrap();

        let db = std::sync::Arc::new(std::sync::Mutex::new(
            nexus_coordinator_rs::db::CoordinatorDb::open_in_memory().unwrap(),
        ));
        db.lock().unwrap().insert_outbox(&envelope).unwrap();

        // Load the outbox exactly as boot does, then restore.
        let outbox = db.lock().unwrap().load_outbox().unwrap();
        assert_eq!(outbox.len(), 1, "outbox must persist the envelope");

        let agg = std::sync::Arc::new(BrowseAggregator::new());
        assert_eq!(
            agg.direct_entry_count(),
            0,
            "aggregator starts empty on a fresh boot"
        );
        let restored = super::restore_browse_from_outbox(&agg, &db, &node, &outbox);
        assert_eq!(
            restored, 1,
            "the persisted project announcement must be restored"
        );

        // The card is back, with its REAL name (not the empty feed op), and
        // Reachable because it is our own node.
        let curator = CuratorRuntime::new(None);
        let out = agg.aggregate(&curator, &node).await;
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].project_name, "My Restored App",
            "the real project_name is restored from the announcement"
        );
        assert_eq!(out[0].project_id, pid);
        assert_eq!(
            out[0].status,
            BrowseStatus::Reachable,
            "a node's own restored app must be Reachable (self-branch, no dial)"
        );

        // And the restore re-indexed the search corpus with the real name,
        // fixing the search-by-name gap for a node's own apps after a restart.
        let (results, total) =
            nexus_coordinator_rs::search::search(&db.lock().unwrap(), "Restored", 20, 0).unwrap();
        assert_eq!(total, 1, "restored app is findable by name");
        assert_eq!(results[0].project_id, pid);

        node.shutdown().await.ok();
    }

    #[test]
    fn keep_online_disabled_app_not_rebroadcast() {
        // Sprint 74 Phase D: the boot/NeighborUp re-broadcast gate. A real PoW
        // envelope for an app that the node turned OFF must be suppressed; every
        // other case replays (empty set, a different app disabled, undecodable
        // bytes). Per-app id keying is what makes a shared-blob OFF safe.
        use std::collections::HashSet;
        let pid = hex::encode(nexus_core_rs::crypto::blake3_hash(b"Disabled App"));
        let ann = publish::ProjectAnnouncement::new(
            "a".repeat(64),
            "Disabled App".into(),
            "tools".into(),
            "x".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        let payload = ann.to_gossip_bytes().unwrap();
        let kp = nexus_core_rs::KeyPair::generate();
        let policy = nexus_core_rs::RelayPowPolicy {
            default_difficulty: 1,
            topic_overrides: std::collections::BTreeMap::new(),
        };
        let proof = nexus_core_rs::PowSolveCache::new()
            .ensure_proof([7u8; 32], &kp, &policy)
            .unwrap();
        let envelope = nexus_core_rs::PowEnvelope::encode(&proof, &payload).unwrap();

        // Empty disabled set -> replay all (fast path, no decode).
        assert!(super::keep_online_allows_rebroadcast(
            &envelope,
            &HashSet::new()
        ));
        // This app disabled -> suppressed.
        let disabled: HashSet<String> = [pid.clone()].into_iter().collect();
        assert!(!super::keep_online_allows_rebroadcast(&envelope, &disabled));
        // A DIFFERENT app disabled -> this one still replays.
        let other: HashSet<String> = ["b".repeat(64)].into_iter().collect();
        assert!(super::keep_online_allows_rebroadcast(&envelope, &other));
        // Undecodable bytes -> never dropped on a decode hiccup.
        assert!(super::keep_online_allows_rebroadcast(
            b"not an envelope",
            &disabled
        ));
    }

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

    #[tokio::test]
    async fn test_feed_subscribe_joinhandle_shutdown() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let rt = DaemonRuntime::start(opts).await.expect("start");
        assert!(
            rt.feed_handle.is_some(),
            "feed subscribe must be spawned at boot"
        );
        rt.shutdown()
            .await
            .expect("shutdown must join feed handle without leak");
    }

    /// 2026-06-05 platform-remediation #6 — the E2E network-execute
    /// anti-recurrence GATE.
    ///
    /// The systemic bug the remediation fights is "discovery vs service
    /// never reconciled, and every test mocks the frontier". This gate
    /// drives the FULL real path with ZERO frontier mock: a real
    /// `DaemonRuntime` (real loopback HTTP + auth + iroh node +
    /// dispatch_loop + result_sync + validator_loop + coordinator DB),
    /// a real `nexus-worker` Engine on a SEPARATE iroh node joined by a
    /// real invite ticket, a task submitted over real HTTP, and the
    /// result polled back over real HTTP. The only mock is the
    /// deterministic `StubBackend` LLM. If any frontier link breaks
    /// (HTTP, auth, iroh cross-node sync, worker claim, the result
    /// bridge, the DB, or retrieval) this fails — which the ~1866 green
    /// unit tests did NOT, because they each mocked one side.
    // multi_thread + serial(sbfb_env): two iroh nodes + the worker pump
    // each need the docs actor on a dedicated thread (P2-A-1), and the
    // test mutates process env (token + the local-worker toggle).
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[serial_test::serial(sbfb_env)]
    async fn e2e_network_execute_gate_real_http_no_frontier_mock() {
        use nexus_worker_core::allowlist::{Allowlist, NewProject};
        use nexus_worker_core::config::{Engine as EngineCfg, WorkerConfig};
        use nexus_worker_core::consent::{ConsentConfig, ConsentLevel};
        use nexus_worker_core::engine::{Engine, EngineBoot};
        use nexus_worker_core::llm::StubBackend;
        use std::time::Duration;

        let tmp = tempdir().expect("tempdir");
        // Known bearer token via env (the daemon reads SBFB_AUTH_TOKEN
        // before any disk path), isolate the shell-daemon dir, and
        // disable the OS-process worker auto-spawn — this gate wires its
        // own in-process worker Engine so it stays hermetic (no reliance
        // on a built `nexus-worker` binary next to the test exe).
        let token = "a".repeat(64);
        unsafe {
            std::env::set_var(auth::AUTH_TOKEN_ENV, &token);
            std::env::set_var(
                nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV,
                tmp.path(),
            );
            std::env::set_var("SBFB_NO_LOCAL_WORKER", "1");
        }

        let rt = DaemonRuntime::start(mk_opts(tmp.path()))
            .await
            .expect("daemon boots");
        let base = format!("http://127.0.0.1:{}", rt.bound_addr().port());
        let client = reqwest::Client::new();
        let hdr = auth::AUTH_HEADER;

        // 1. Mint a worker invite over real HTTP — it carries the write
        //    ticket for the daemon's project doc.
        let inv: serde_json::Value = client
            .post(format!("{base}/api/v1/invite/create"))
            .header(hdr, &token)
            .json(&serde_json::json!({"scope": "worker"}))
            .send()
            .await
            .expect("invite request")
            .json()
            .await
            .expect("invite json");
        let wire = inv["wire"].as_str().expect("invite carries a wire");
        let invite = nexus_worker_core::invite::Invite::decode(wire).expect("decode invite");
        let project_id = invite.payload.project_id.clone();
        let ticket = invite
            .payload
            .tasks_doc_ticket
            .clone()
            .expect("worker invite carries the project doc ticket");

        // 2. A real worker Engine on its OWN iroh node, joined by the
        //    ticket (the production cross-node join path).
        let allowlist = Allowlist::open_in_memory().expect("allowlist");
        allowlist
            .enroll(NewProject {
                id: project_id.clone(),
                name: "gate".into(),
                enabled: true,
                budget_joules: 0,
                tasks_doc_ticket: Some(ticket),
            })
            .expect("enroll");
        let sbfb_tmp = tempdir().expect("sbfb tmp");
        let mut consent = ConsentConfig::default_for("gate-worker");
        consent.level = ConsentLevel::All;
        consent
            .save_atomic(&sbfb_tmp.path().join("consent.json"))
            .expect("consent");
        let boot = EngineBoot {
            worker_config: WorkerConfig {
                engine: EngineCfg {
                    task_poll_interval_ms: 100,
                    max_concurrent_tasks: 1,
                    state_flush_secs: 5,
                },
                ..WorkerConfig::default()
            },
            keypair: nexus_core_rs::KeyPair::generate(),
            allowlist,
            data_dir: None,
            llm_override: Some(Box::new(StubBackend::new())),
            sbfb_home_override: Some(sbfb_tmp.path().to_path_buf()),
            rate_limit_policy_path_override: None,
        };
        let mut engine = Engine::new(boot).await.expect("worker engine boots");
        let w_stop = engine.take_shutdown_sender().expect("shutdown sender");
        let worker = tokio::spawn(async move { engine.run_until_shutdown().await });

        // 3. Submit a task over real HTTP.
        let sub: serde_json::Value = client
            .post(format!("{base}/api/v1/tasks/submit"))
            .header(hdr, &token)
            .json(&serde_json::json!({
                "project_id": project_id,
                "task_type": "inference",
                "prompt": "ping",
                "model": "llama3",
                "is_open_source": true,
            }))
            .send()
            .await
            .expect("submit request")
            .json()
            .await
            .expect("submit json");
        let task_id = sub["task"]["task_id"]
            .as_str()
            .expect("submit returns a task id")
            .to_string();

        // 4. Poll real GET /result until the worker's output has flowed
        //    back across iroh, through the result bridge + validator,
        //    into the DB, and out the retrieval endpoint.
        let mut result_text = None;
        for _ in 0..150 {
            let resp = client
                .get(format!("{base}/api/v1/tasks/{task_id}/result"))
                .header(hdr, &token)
                .send()
                .await
                .expect("result request");
            if resp.status().is_success() {
                let body: serde_json::Value = resp.json().await.expect("result json");
                if let Some(t) = body["result_text"].as_str() {
                    result_text = Some(t.to_string());
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        assert!(
            result_text.is_some(),
            "result_text must return over the real HTTP path within 30s — \
             the full submit -> dispatch -> iroh sync -> worker -> result \
             bridge -> DB -> retrieval frontier, with no mock"
        );

        let _ = w_stop.send(());
        let _ = worker.await;
        rt.shutdown().await.expect("daemon shutdown");
        unsafe {
            std::env::remove_var(auth::AUTH_TOKEN_ENV);
            std::env::remove_var(nexus_shell_daemon_core::paths::NEXUS_GRID_ROOT_ENV);
            std::env::remove_var("SBFB_NO_LOCAL_WORKER");
        }
    }

    #[test]
    fn jitter_bounds_are_within_range() {
        for _ in 0..200 {
            let d = jittered_republish_duration();
            assert!(
                d.as_secs() >= 30 && d.as_secs() <= 60,
                "jitter {d:?} out of [30s, 60s]"
            );
        }
    }

    #[tokio::test]
    async fn boot_storage_namespace_persistent_reopen() {
        let tmp = tempdir().expect("tempdir");
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn boot_feed_namespace_persistent_reopen() {
        let tmp = tempdir().expect("tempdir");
        let opts1 = mk_opts(tmp.path());
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        assert!(rt1.feed_handle.is_some());
        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(rt2.feed_handle.is_some());
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_republish_at_boot() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "test-republish",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(entries.len(), 1, "SQLite must have 1 entry before reboot");
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(
            rt2.feed_handle.is_some(),
            "feed subscribe must be active after republish boot"
        );
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_feed_join_handles_tracked_and_shutdown() {
        let tmp = tempdir().expect("tempdir");
        let opts = mk_opts(tmp.path());
        let rt = DaemonRuntime::start(opts).await.unwrap();
        assert!(
            rt.feed_join_handles.is_some(),
            "feed_join_handles must be initialized at boot"
        );
        assert!(
            rt.feed_join_shutdown.is_some(),
            "feed_join_shutdown must be initialized at boot"
        );
        let handles = rt.feed_join_handles.as_ref().unwrap();
        assert_eq!(
            handles.lock().unwrap().len(),
            0,
            "no feed_join calls yet, Vec must be empty"
        );
        rt.shutdown()
            .await
            .expect("shutdown must join feed_join handles without leak");
    }

    #[tokio::test]
    async fn test_orphan_republish_recovery() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        rt1.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "test-orphan",
                "version": "2.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
            assert_eq!(
                db.count_feed_entries().unwrap(),
                1,
                "SQLite must have 1 entry before recovery boot"
            );
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must be active after orphan recovery"
        );
        rt2.shutdown().await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open DB");
            assert_eq!(
                db.count_feed_entries().unwrap(),
                1,
                "feed entry must survive recovery boot without data loss"
            );
        }

        let opts3 = mk_opts(tmp.path());
        let rt3 = DaemonRuntime::start(opts3).await.unwrap();
        assert!(
            rt3.feed_handle.is_some(),
            "feed sync must remain active after second boot"
        );
        rt3.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_key_rotation_persistence_survives_reboot() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();
        assert_eq!(
            rt1.revocation_cache().read().unwrap().len(),
            0,
            "no rotations before insert"
        );
        rt1.shutdown().await.unwrap();

        let kp = nexus_core_rs::KeyPair::generate();
        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let new_kp = nexus_core_rs::KeyPair::generate();
            db.insert_key_rotation(
                &hex::encode(kp.public_bytes()),
                &hex::encode(new_kp.public_bytes()),
                1_700_000_000,
                7,
                "test_sig",
                "test reason",
            )
            .expect("insert key rotation");
        }

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();
        assert_eq!(
            rt2.revocation_cache().read().unwrap().len(),
            1,
            "RevocationCache must contain the persisted rotation after reboot"
        );
        assert!(
            rt2.revocation_cache()
                .read()
                .unwrap()
                .is_in_transition(&kp.public_bytes(), 1_700_000_000),
            "old key must be in transition after restore"
        );
        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_e2e_restart_full_cycle() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();

        let node_id_1 = rt1.node.as_ref().unwrap().node_id();

        let kp = nexus_core_rs::KeyPair::generate();
        rt1.curator_runtime()
            .subscribe(&hex::encode(kp.public_bytes()))
            .unwrap();

        let blobs = nexus_core_rs::BlobsClient::new(rt1.node.as_ref().unwrap().blobs_store());
        let blob_hash = blobs.add_bytes(b"e2e-restart-payload").await.unwrap();
        let retrieved = blobs.get_bytes(blob_hash).await.unwrap();
        assert_eq!(retrieved, b"e2e-restart-payload");

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let feed_kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(feed_kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "e2e-restart-proj",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                feed_kp.sign(data).to_vec()
            })
            .expect("insert feed op");
        }

        rt1.shutdown().await.unwrap();

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();

        let node_id_2 = rt2.node.as_ref().unwrap().node_id();
        assert_eq!(
            node_id_1, node_id_2,
            "node_id must be identical across restart (persistent identity)"
        );

        assert!(
            rt2.curator_runtime().is_subscribed(&kp.public_bytes()),
            "curator subscription must survive restart"
        );

        let blobs2 = nexus_core_rs::BlobsClient::new(rt2.node.as_ref().unwrap().blobs_store());
        let data2 = blobs2.get_bytes(blob_hash).await.unwrap();
        assert_eq!(
            data2, b"e2e-restart-payload",
            "blob must survive restart (FsStore persistence)"
        );

        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must be active after restart"
        );

        {
            let db =
                nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open DB post-boot");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(
                entries.len(),
                1,
                "feed entry must survive restart in SQLite"
            );
        }

        assert_eq!(
            rt2.revocation_cache().read().unwrap().len(),
            0,
            "revocation cache must be initialized (empty, no rotations inserted)"
        );

        rt2.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_e2e_crash_recovery() {
        let tmp = tempdir().expect("tempdir");

        let opts1 = mk_opts(tmp.path());
        let db_path = opts1.paths.root.join("coordinator.db");
        let running_json = opts1.paths.running_json.clone();
        let rt1 = DaemonRuntime::start(opts1).await.unwrap();

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path).expect("open test DB");
            let kp = nexus_core_rs::KeyPair::generate();
            let pubkey = hex::encode(kp.public_bytes());
            let op = serde_json::json!({
                "type": "ReleasePublished",
                "project_id": "crash-recovery",
                "version": "1.0.0"
            });
            nexus_coordinator_rs::public_feed::insert_feed_operation(&db, op, &pubkey, |data| {
                kp.sign(data).to_vec()
            })
            .expect("insert feed op");
        }

        rt1.shutdown().await.unwrap();

        // Simulate crash aftermath: a stale running.json left behind
        // by a process that died before cleanup. The singleton check
        // must detect the stale marker and proceed.
        let stale = nexus_shell_daemon_core::registry::RunningState {
            schema_version: 1,
            node_id: "0".repeat(64),
            api_host: "127.0.0.1".to_string(),
            api_port: 1,
            pid: 0,
            started_at: "2000-01-01T00:00:00Z".to_string(),
            daemon_version: "0.0.0-crashed".to_string(),
        };
        raw_write_running(&stale, &running_json).unwrap();
        assert!(running_json.exists(), "stale running.json must be present");

        let opts2 = mk_opts(tmp.path());
        let rt2 = DaemonRuntime::start(opts2).await.unwrap();

        assert!(
            rt2.feed_handle.is_some(),
            "feed sync must recover after crash"
        );

        {
            let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
                .expect("open DB post-crash");
            let entries = nexus_coordinator_rs::public_feed::replay_all(&db).unwrap();
            assert_eq!(
                entries.len(),
                1,
                "feed entry must survive crash (SQLite WAL + FULL pragma)"
            );
        }

        rt2.shutdown().await.unwrap();
        assert!(
            !running_json.exists(),
            "running.json must be cleaned after recovery"
        );
    }
}
