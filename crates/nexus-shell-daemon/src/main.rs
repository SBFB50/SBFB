// SPDX-License-Identifier: AGPL-3.0-or-later
//! # nexus-shell-daemon
//!
//! SBFB shell daemon: the long-lived P2P process that backs the
//! React shell's Browse / Curators pages. One instance per user,
//! reached by the shell exclusively via the coordinator
//! `/daemon/*` proxy (Sprint 7 D1 — HTTP loopback, not Unix
//! socket, not cross-origin direct call).
//!
//! ## Sprint 7 Phase A architecture
//!
//! This binary is a thin CLI + runtime wrapper around the
//! headless engine living in [`nexus_shell_daemon_core`]. The
//! split mirrors the Sprint 3 worker-core / worker pair: the
//! engine must run and be fully testable without any axum
//! server or clap parser.
//!
//! ## Phase A scope
//!
//! - `start` — singleton check → iroh node boot → HTTP serve on
//!   an ephemeral loopback port → `running.json` write → block
//!   on ctrl+c → graceful shutdown.
//! - `stop` / `status` / `config` — stubs (they print a "not yet
//!   implemented" pointer; the wiring never shipped).
//!
//! Later phases layered on the curator gossip subscribe pipeline
//! (Phase C), pkarr browse resolution (Phase D), and the
//! coordinator proxy + shell pages (Phase E). **None of those
//! were in the Phase A foundation.**

mod apps;
mod canary_api;
mod cli;
mod consent;
mod contributor_api;
mod deploy;
mod diagnostic_api;
mod dispatch_loop;
mod feed_sync;
mod files;
mod health_api;
mod http;
mod invite_api;
mod kudos_api;
mod local_worker;
mod logging;
#[cfg(windows)]
mod named_pipe_server;
mod noop_identity;
mod panic;
mod quarantine_api;
mod result_sync;
mod runtime;
mod seed_protocol;
mod seed_registry;
mod shard_session;
mod shard_session_http_api;
mod shell_api;
mod storage_api;
mod tasks_api;
#[cfg(test)]
mod test_support;
#[cfg(unix)]
mod uds_server;
mod validator_loop;
mod worker_state_api;

use anyhow::{Context, Result};
use clap::Parser;
use nexus_shell_daemon_core::config::{ShellDaemonConfig, ShellDaemonPaths};

use cli::{
    CanaryCommand, CapabilityCommand, Cli, Command, ConfigCommand, FrostCommand, InviteCommand,
    QuarantineCommand, ShardSessionCommand,
};
use runtime::{DaemonRuntime, DaemonStartOptions};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let paths = ShellDaemonPaths::resolve(cli.config.clone())
        .context("could not resolve shell-daemon paths for this platform")?;

    // Initialize structured logging. Try to honour the config's
    // log level if it exists; fall back to "info" before the
    // config is written (e.g. the very first `start` on a fresh
    // machine) so early errors still surface.
    let level = match ShellDaemonConfig::load(&paths.config_file) {
        Ok(cfg) => cfg.logging.level,
        Err(_) => "info".to_string(),
    };
    paths
        .ensure_dirs()
        .context("failed to create shell-daemon directories")?;
    let _log_guard = logging::init_logging(&paths.log_dir, &level, cli.verbose)
        .context("failed to initialize tracing subscriber")?;

    tracing::debug!(
        core_version = nexus_shell_daemon_core::VERSION,
        "nexus-shell-daemon parsed CLI"
    );

    match cli.command {
        Command::Start {
            headless,
            cors_origins,
            web_root,
        } => handle_start(paths, headless, cors_origins, web_root).await,
        Command::Stop => handle_stop(&paths).await,
        Command::Status => handle_status(&paths).await,
        Command::Init => handle_init(&paths).await,
        Command::Invite(cmd) => handle_invite(&paths, cmd).await,
        Command::Quarantine(cmd) => handle_quarantine(&paths, cmd).await,
        Command::Capability(cmd) => handle_capability(&paths, cmd).await,
        Command::Config(cmd) => handle_config(&paths, cmd).await,
        Command::Canary(cmd) => handle_canary(cmd).await,
        Command::ShardSession(cmd) => handle_shard_session(&paths, cmd).await,
    }
}

// =====================================================================
// Sprint 81 Phase I — shard-session operator tool
// =====================================================================

/// Persistent identity for the `shard-session serve` worker on THIS
/// machine (distinct from the daemon's `node_key`: the serve worker is
/// its own admitted member, so a machine can serve shards while its
/// daemon heads a different session). Same 32-raw-bytes format as
/// `load_or_generate_node_key`.
fn load_or_generate_serve_key(path: &std::path::Path) -> Result<[u8; 32]> {
    use nexus_core_rs::crypto::KeyPair;
    if path.exists() {
        let data = std::fs::read(path)
            .with_context(|| format!("failed to read shard-serve key from {}", path.display()))?;
        if data.len() == 32 {
            let mut out = [0u8; 32];
            out.copy_from_slice(&data);
            return Ok(out);
        }
        anyhow::bail!(
            "shard-serve key at {} has {} bytes (expected 32) — refusing to overwrite; \
             delete it to mint a fresh identity",
            path.display(),
            data.len()
        );
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let secret = KeyPair::generate().secret_bytes();
    std::fs::write(path, secret)
        .with_context(|| format!("failed to write shard-serve key to {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).with_context(
            || {
                format!(
                    "failed to set shard-serve key permissions on {}",
                    path.display()
                )
            },
        )?;
    }
    Ok(secret)
}

/// Discover the live local daemon (running.json) and bootstrap its
/// loopback bearer via the public `/auth/token` route — the same
/// host+origin-gated bootstrap the React shell uses, so the operator
/// never hand-plumbs a port or token.
async fn daemon_client(paths: &ShellDaemonPaths) -> Result<(reqwest::Client, String, String)> {
    use nexus_shell_daemon_core::registry::StaleOutcome;
    let state = match nexus_shell_daemon_core::registry::check_stale_or_bail(&paths.running_json) {
        StaleOutcome::Live { state, .. } => state,
        StaleOutcome::NoFile => anyhow::bail!(
            "no running daemon found ({} is absent) — `nexus-shell-daemon start` first",
            paths.running_json.display()
        ),
        StaleOutcome::Stale { pid, .. } => anyhow::bail!(
            "running.json points at dead pid {pid} — the daemon is not running; \
             `nexus-shell-daemon start` first"
        ),
        StaleOutcome::Corrupt { reason } => {
            anyhow::bail!("running.json is unreadable ({reason}) — restart the daemon")
        }
    };
    let base = format!("http://{}:{}", state.api_host, state.api_port);
    let client = reqwest::Client::new();
    let token: serde_json::Value = client
        .get(format!("{base}/auth/token"))
        .send()
        .await
        .with_context(|| format!("daemon unreachable at {base}"))?
        .error_for_status()
        .context("token bootstrap refused")?
        .json()
        .await
        .context("token bootstrap returned non-JSON")?;
    let token = token["token"]
        .as_str()
        .context("token bootstrap response carries no token")?
        .to_string();
    Ok((client, base, token))
}

/// POST/GET a shard-session route on the live daemon and pretty-print the
/// JSON reply. Exits non-zero on a non-2xx status so scripts can gate.
async fn shard_api_call(
    paths: &ShellDaemonPaths,
    method: reqwest::Method,
    route: &str,
    body: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let (client, base, token) = daemon_client(paths).await?;
    let mut req = client
        .request(method, format!("{base}{route}"))
        .header("x-sbfb-token", token);
    if let Some(body) = body {
        req = req.json(&body);
    }
    let resp = req.send().await.context("daemon request failed")?;
    let status = resp.status();
    let json: serde_json::Value = resp.json().await.context("daemon reply is not JSON")?;
    println!("{}", serde_json::to_string_pretty(&json)?);
    if !status.is_success() {
        anyhow::bail!("daemon answered {status}");
    }
    Ok(json)
}

async fn handle_shard_session(paths: &ShellDaemonPaths, cmd: ShardSessionCommand) -> Result<()> {
    use nexus_core_rs::crypto::KeyPair;
    let default_key = paths.root.join("shard-serve.key");
    match cmd {
        ShardSessionCommand::Identity { key } => {
            let key_path = key.unwrap_or(default_key);
            let secret = load_or_generate_serve_key(&key_path)?;
            let kp = KeyPair::from_secret_bytes(&secret);
            println!("shard-serve identity (pubkey hex):");
            println!("{}", hex::encode(kp.public_bytes()));
            println!("key file: {}", key_path.display());
            Ok(())
        }
        ShardSessionCommand::Serve {
            group,
            key,
            model,
            layer_start,
            layer_end,
            n_gpu_layers,
            n_ctx,
        } => {
            use nexus_core_rs::node::{NodeConfig, create_node_with_protocols};
            use nexus_core_rs::{DiscoveryClient, EchoForwarder, SHARD_ALPN};
            let key_path = key.unwrap_or(default_key);
            let secret = load_or_generate_serve_key(&key_path)?;
            let kp = KeyPair::from_secret_bytes(&secret);
            let raw = std::fs::read_to_string(&group)
                .with_context(|| format!("failed to read group file {}", group.display()))?;
            let entry: nexus_core_rs::ComputeGroupEntry = serde_json::from_str(&raw)
                .context("group file is not a signed ComputeGroupEntry JSON")?;
            entry
                .verify_signature()
                .map_err(|e| anyhow::anyhow!("group signature rejected: {e}"))?;
            if !entry.is_member(&kp.public_bytes()) {
                anyhow::bail!(
                    "this machine's serve identity {} is NOT a member of group '{}' — \
                     re-mint the group with `shard-session group --member <this pubkey>`",
                    hex::encode(kp.public_bytes()),
                    entry.group.group_id
                );
            }
            // Sprint 81 Phase J (Option B): with --model, load the layer
            // window through the forked backend and serve the REAL
            // role-aware stage; otherwise stay the transport-only echo.
            #[cfg(not(feature = "llm_llama_cpp"))]
            let _ = (layer_start, layer_end, n_gpu_layers, n_ctx);
            let (forwarder, role_banner): (
                std::sync::Arc<dyn nexus_core_rs::ShardForwarder>,
                String,
            ) = match &model {
                None => (
                    std::sync::Arc::new(EchoForwarder),
                    "transport-only echo forwarder".to_string(),
                ),
                #[cfg(feature = "llm_llama_cpp")]
                Some(gguf) => {
                    let start = layer_start.context("--layer-start is required with --model")?;
                    let end = layer_end.context("--layer-end is required with --model")?;
                    // Cheap window precondition (review J J8-1): an
                    // out-of-range window trips a native GGML_ASSERT that
                    // ABORTS the process inside `load_from_file`, BEFORE
                    // the recoverable ShardWindow check — `load`'s doc
                    // mandates callers pre-validate. Catch the trivially
                    // checkable operator typo here with a clean error; the
                    // model-bound check (`end > n_layer`) stays the fork's
                    // native assert (no metadata-only probe exposed by the
                    // binding).
                    if end != 0 && start >= end {
                        anyhow::bail!(
                            "invalid layer window [{start},{end}): start must be < end \
                             (use --layer-end 0 for 'to the model's last layer')"
                        );
                    }
                    println!(
                        "loading GGUF {} window [{start},{end}) (n_gpu_layers={n_gpu_layers}, \
                         n_ctx={n_ctx}) — partial load, this can take a minute…",
                        gguf.display()
                    );
                    let is_first = start == 0;
                    // `--layer-end 0` = "run to the model's last layer" =>
                    // the tail. A LITERAL end is declared non-last; if it
                    // actually equals the model's layer count the backend
                    // rejects the flag mismatch LOUD (window/flags check) —
                    // re-serve the true tail with `--layer-end 0`.
                    let is_last = end == 0;
                    let backend = nexus_worker_core::llm::shard::ShardBackend::load(
                        gguf,
                        start,
                        end,
                        is_first,
                        is_last,
                        n_gpu_layers,
                        n_ctx.min(nexus_core_rs::MAX_SHARD_N_CTX),
                    )
                    .map_err(|e| anyhow::anyhow!("shard backend load failed: {e}"))?;
                    // Stage attestation digest (Sprint 81 Phase K, closes
                    // the Phase J carry `THREAT_MODEL §16`): STREAMING
                    // blake3 of the loaded GGUF — never `std::fs::read`,
                    // which would OOM an 8 GB tail machine on a ~16 GB
                    // file (Codex GPT-5.6 Sol P1). The driver compares
                    // this self-declared digest + the loaded window/roles
                    // against the SIGNED manifest at every stage-link
                    // establishment and fail-closes on mismatch.
                    println!("hashing GGUF (streaming blake3) for the stage attestation…");
                    let model_digest = nexus_core_rs::crypto::blake3_hash_file(gguf)
                        .map_err(|e| anyhow::anyhow!("model digest failed: {e}"))?;
                    let w = backend.window();
                    // Banner prints the loaded window + role + digest so the
                    // operator can cross-check against `shard-session plan`
                    // and the mount (Codex GPT-5.6 Sol P1,
                    // operator-verifiability half); the drive now enforces
                    // the same binding in-band via the attestation.
                    let banner = format!(
                        "REAL layer-block stage [{},{}) of {} (is_first={}, is_last={}, \
                         n_embd={}, model_digest={}) — attested to the driver at every \
                         stage-link establishment",
                        w.start(),
                        w.end(),
                        gguf.display(),
                        w.is_first(),
                        w.is_last(),
                        backend.n_embd(),
                        hex::encode(model_digest),
                    );
                    (
                        std::sync::Arc::new(
                            nexus_worker_core::llm::shard::ShardStageForwarder::new(
                                std::sync::Arc::new(backend),
                                model_digest,
                            ),
                        ),
                        banner,
                    )
                }
                #[cfg(not(feature = "llm_llama_cpp"))]
                Some(_) => {
                    anyhow::bail!(
                        "--model requires a build with the forked backend: rebuild with \
                         `--features llm_llama_cpp_cuda` (RTX 5080) or \
                         `--features llm_llama_cpp_metal` (Mac)"
                    );
                }
            };
            let factory = nexus_core_rs::shard_protocol_factory(entry, forwarder)
                .map_err(|e| anyhow::anyhow!("shard protocol wiring failed: {e}"))?;
            let node = create_node_with_protocols(
                NodeConfig::default().with_secret_key(secret),
                vec![(SHARD_ALPN.to_vec(), factory)],
            )
            .await
            .map_err(|e| anyhow::anyhow!("failed to boot the shard-serve node: {e}"))?;
            let addr = DiscoveryClient::new(node.endpoint())
                .my_endpoint_addr()
                .await
                .map_err(|e| anyhow::anyhow!("no dialable address yet: {e}"))?;
            println!("serving sbfb/shard/1 ({role_banner})");
            println!("identity: {}", hex::encode(kp.public_bytes()));
            println!("paste this address into the mount config `workers[].addr`:");
            println!("{}", serde_json::to_string(&addr)?);
            println!("ctrl+c to stop");
            tokio::signal::ctrl_c().await.context("ctrl+c handler")?;
            node.shutdown()
                .await
                .map_err(|e| anyhow::anyhow!("shard-serve node shutdown failed: {e}"))?;
            Ok(())
        }
        ShardSessionCommand::Plan {
            session_id,
            total_layers,
            model_bytes,
            workers,
        } => {
            use nexus_coordinator_rs::placement::{
                ModelSpec, PlacementOutcome, RttMatrix, WorkerPlacementProfile, plan_placement,
            };
            let mut candidates = Vec::with_capacity(workers.len());
            for w in &workers {
                let (pk_hex, vram) = w.split_once(':').with_context(|| {
                    format!("worker '{w}' is not <pubkey_hex>:<vram_free_bytes>")
                })?;
                let pubkey = crate::shard_session::parse_pubkey_hex(pk_hex)
                    .map_err(|e| anyhow::anyhow!("worker '{w}': {e}"))?;
                let vram_free_bytes: u64 = vram
                    .parse()
                    .with_context(|| format!("worker '{w}': vram bytes is not a u64"))?;
                candidates.push(WorkerPlacementProfile {
                    worker_pubkey: pubkey,
                    vram_free_bytes,
                    shard_hashes: vec![],
                    launch_profile_hash: [0u8; 32],
                });
            }
            let spec = ModelSpec {
                total_layers,
                quantized_vram_bytes: model_bytes,
            };
            match plan_placement(&candidates, &RttMatrix::new(), &spec, &session_id)
                .map_err(|e| anyhow::anyhow!("placement failed: {e}"))?
            {
                PlacementOutcome::EndpointFederation => {
                    println!(
                        "NO SHARD: the model ({model_bytes} bytes) fits a single worker's \
                         declared free VRAM — use S76 endpoint federation instead"
                    );
                }
                PlacementOutcome::Sharded(plan) => {
                    println!(
                        "deterministic placement for session '{session_id}' \
                         ({total_layers} layers, {model_bytes} bytes):"
                    );
                    let last = plan
                        .assignments
                        .iter()
                        .map(|a| a.layer_end)
                        .max()
                        .unwrap_or(0);
                    for a in &plan.assignments {
                        let is_first = a.layer_start == 0;
                        let is_last = a.layer_end == last;
                        println!(
                            "  worker {}…  --layer-start {} --layer-end {}   \
                             (is_first={is_first}, is_last={is_last}, {} layers)",
                            &hex::encode(a.worker_pubkey)[..16],
                            a.layer_start,
                            if is_last { 0 } else { a.layer_end },
                            a.layer_end - a.layer_start,
                        );
                    }
                    println!(
                        "boot each `shard-session serve --model <gguf>` with ITS window above; \
                         the mount re-derives this exact plan from the same inputs"
                    );
                }
            }
            Ok(())
        }
        ShardSessionCommand::Group {
            group_id,
            members,
            out,
        } => {
            let json = shard_api_call(
                paths,
                reqwest::Method::POST,
                "/api/daemon/shard-session/group",
                Some(serde_json::json!({ "group_id": group_id, "members": members })),
            )
            .await?;
            if let Some(out) = out {
                let group = &json["group"];
                if group.is_null() {
                    anyhow::bail!("daemon minted no group (duress boot?) — nothing written");
                }
                std::fs::write(&out, serde_json::to_string_pretty(group)?)
                    .with_context(|| format!("failed to write {}", out.display()))?;
                println!("signed group written to {}", out.display());
            }
            Ok(())
        }
        ShardSessionCommand::Mount { mount_config } => {
            let raw = std::fs::read_to_string(&mount_config).with_context(|| {
                format!("failed to read mount config {}", mount_config.display())
            })?;
            let body: serde_json::Value =
                serde_json::from_str(&raw).context("mount config is not valid JSON")?;
            shard_api_call(
                paths,
                reqwest::Method::POST,
                "/api/daemon/shard-session/mount",
                Some(body),
            )
            .await?;
            Ok(())
        }
        ShardSessionCommand::Status { session_id } => {
            shard_api_call(
                paths,
                reqwest::Method::GET,
                &format!("/api/daemon/shard-session/{session_id}"),
                None,
            )
            .await?;
            Ok(())
        }
        ShardSessionCommand::Generate {
            session_id,
            prompt,
            max_tokens,
        } => {
            let mut body = serde_json::json!({ "session_id": session_id, "prompt": prompt });
            if let Some(n) = max_tokens {
                body["max_tokens"] = serde_json::json!(n);
            }
            shard_api_call(
                paths,
                reqwest::Method::POST,
                &format!("/api/daemon/shard-session/{session_id}/generate"),
                Some(body),
            )
            .await?;
            Ok(())
        }
        ShardSessionCommand::Result { session_id } => {
            shard_api_call(
                paths,
                reqwest::Method::GET,
                &format!("/api/daemon/shard-session/{session_id}/result"),
                None,
            )
            .await?;
            Ok(())
        }
        ShardSessionCommand::DropShard { session_id } => {
            shard_api_call(
                paths,
                reqwest::Method::POST,
                &format!("/api/daemon/shard-session/{session_id}/drop-shard"),
                None,
            )
            .await?;
            Ok(())
        }
    }
}

async fn handle_start(
    paths: ShellDaemonPaths,
    _headless: bool,
    cli_cors_origins: Vec<String>,
    cli_web_root: Option<std::path::PathBuf>,
) -> Result<()> {
    // Load the config to pick up any user-tuned bind host / port.
    // A missing file is fine — Phase A defaults to 127.0.0.1:0.
    let cfg = ShellDaemonConfig::load(&paths.config_file)
        .context("failed to load shell-daemon config")?;

    println!("nexus-shell-daemon v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_shell_daemon_core::VERSION);
    println!("  config:       {}", paths.config_file.display());
    println!("  running.json: {}", paths.running_json.display());
    println!();

    // Sprint 20 Phase B : the launcher sets `SBFB_IDENTITY_MODE=
    // duress` in the child's environment when `sbfb unlock`
    // matched the duress blob. Any other value (including unset)
    // falls through to `Normal`. The env var is read inside the
    // daemon process and never persisted on disk, matching the
    // Phase A pattern for `SBFB_IDENTITY_SECRET_HEX`.
    let cors_origins = if cli_cors_origins.is_empty() {
        std::env::var("NEXUS_DAEMON_CORS_ORIGINS")
            .ok()
            .map(|s| {
                s.split(',')
                    .map(|o| o.trim().to_string())
                    .filter(|o| !o.is_empty())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else {
        cli_cors_origins
    };
    for origin in &cors_origins {
        if !crate::http::is_valid_origin(origin) {
            anyhow::bail!(
                "invalid --cors-origin value: {origin:?} (expected http(s)://host[:port])"
            );
        }
    }

    let identity_mode = match std::env::var("SBFB_IDENTITY_MODE").ok().as_deref() {
        Some("duress") => nexus_core_rs::IdentityMode::Duress,
        _ => nexus_core_rs::IdentityMode::Normal,
    };
    // SAFETY: called during early daemon init, before async runtime.
    unsafe { std::env::remove_var("SBFB_IDENTITY_MODE") };

    let web_root = cli_web_root
        .or_else(|| {
            std::env::var("SBFB_WEB_ROOT")
                .ok()
                .map(std::path::PathBuf::from)
        })
        .filter(|p| p.join("index.html").exists());
    if let Some(ref wr) = web_root {
        println!("  web root:     {}", wr.display());
    }

    let opts = DaemonStartOptions {
        paths,
        api_host: cfg.network.api_host.clone(),
        api_port: cfg.network.api_port,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        curator: cfg.curator.clone(),
        seed: cfg.seed.clone(),
        sbfb_home: None,
        identity_mode,
        cors_origins,
        web_root,
    };

    // Register the SIGINT handler BEFORE start() writes running.json,
    // so external observers (test harness, systemd) can send SIGINT as
    // soon as running.json appears without hitting a race window.
    // tokio::signal::unix::signal() installs the handler at creation
    // time (not at first poll like ctrl_c()).
    #[cfg(unix)]
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .context("failed to register SIGINT handler")?;
    #[cfg(not(unix))]
    let ctrl_c = tokio::signal::ctrl_c();

    let runtime = DaemonRuntime::start(opts)
        .await
        .context("daemon start failed")?;

    let addr = runtime.bound_addr();
    println!("  listening on: http://{}", addr);
    println!("  (press ctrl+c to shut down)");
    println!();

    #[cfg(unix)]
    sigint.recv().await;
    #[cfg(not(unix))]
    ctrl_c.await.context("ctrl+c handler failed")?;

    println!();
    println!("nexus-shell-daemon shutting down...");
    runtime.shutdown().await.context("shutdown failed")?;
    println!("nexus-shell-daemon exited cleanly.");
    Ok(())
}

async fn handle_stop(paths: &ShellDaemonPaths) -> Result<()> {
    print_stub(
        "stop",
        "Phase E (coordinator proxy + process signaling)",
        &[("running_json", &paths.running_json.display().to_string())],
    );
    Ok(())
}

async fn handle_status(paths: &ShellDaemonPaths) -> Result<()> {
    print_stub(
        "status",
        "Phase E (coordinator proxy + /info passthrough)",
        &[("running_json", &paths.running_json.display().to_string())],
    );
    Ok(())
}

async fn handle_config(_paths: &ShellDaemonPaths, cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Get { key } => print_stub(
            "config get",
            "Phase E (config persist + dotted key lookup)",
            &[("key", key.as_str())],
        ),
        ConfigCommand::Set { key, value } => print_stub(
            "config set",
            "Phase E (config persist + dotted key lookup)",
            &[("key", key.as_str()), ("value", value.as_str())],
        ),
    }
    Ok(())
}

async fn handle_canary(cmd: CanaryCommand) -> Result<()> {
    use nexus_shell_daemon_core::canary::{
        CanaryBroadcaster, Ed25519CanarySigner, build_canary, format_canary_txt, parse_canary_txt,
        publish_canary, today_utc, warrant_canary_topic_id,
    };

    match cmd {
        CanaryCommand::Publish {
            headline,
            output,
            no_gossip,
        } => {
            // 1. Load (or create) the maintainer's persistent
            //    canary key. Separate from the daemon's ephemeral
            //    node identity on purpose — see the `canary_key_path`
            //    doc for the rationale.
            let key_path = nexus_shell_daemon_core::auth::canary_key_path().with_context(
                || "could not resolve SBFB home dir — set $SBFB_HOME or $HOME/$USERPROFILE",
            )?;
            let keypair =
                nexus_core_rs::KeyPair::load_or_generate(&key_path).with_context(|| {
                    format!(
                        "failed to load or create canary key at {}",
                        key_path.display()
                    )
                })?;
            // Wrap the keypair in the Sprint 20 Phase E.1
            // CanarySigner trait so the build_canary path stays
            // algorithm-agnostic (FrostCanarySigner is the opt-in
            // K-of-N alternative for cross-juridiction maintainer
            // federation).
            let signer = Ed25519CanarySigner::new(keypair);

            // 2. Build + sign the canary.
            let canary = build_canary(today_utc(), headline, &signer)
                .context("failed to build signed canary")?;

            // 3. Write the human-readable mirror.
            let txt = format_canary_txt(&canary);
            std::fs::write(&output, &txt).with_context(|| {
                format!("failed to write canary mirror to {}", output.display())
            })?;

            println!("SBFB canary written to {}", output.display());
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            println!("  key file:     {}", key_path.display());

            // 4. Broadcast on gossip unless opted out. Booting an
            //    iroh node is slow, so keep the noop fast path
            //    when CI just wants to refresh the repo-side file.
            if no_gossip {
                println!("  gossip:       skipped (--no-gossip)");
                return Ok(());
            }

            let node = nexus_core_rs::create_node()
                .await
                .context("failed to boot iroh endpoint for canary broadcast")?;
            let gossip = nexus_core_rs::GossipClient::new(node.gossip());
            let mut topic = gossip
                .join_topic(warrant_canary_topic_id(), Vec::new())
                .await
                .context("failed to join warrant canary gossip topic")?;

            struct TopicBroadcaster<'a> {
                inner: &'a mut nexus_core_rs::TopicHandle,
            }
            #[async_trait::async_trait]
            impl<'a> CanaryBroadcaster for TopicBroadcaster<'a> {
                async fn broadcast(&mut self, bytes: Vec<u8>) -> Result<(), String> {
                    self.inner.broadcast(bytes).await.map_err(|e| e.to_string())
                }
            }

            let mut broadcaster = TopicBroadcaster { inner: &mut topic };
            publish_canary(&canary, &mut broadcaster)
                .await
                .context("gossip broadcast of canary failed")?;
            node.shutdown().await.ok();

            println!("  gossip:       broadcast on warrant-canary/v1");
            Ok(())
        }

        CanaryCommand::Frost(frost_cmd) => handle_frost(frost_cmd).await,

        CanaryCommand::Verify { input } => {
            let text = std::fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))?;
            let canary =
                parse_canary_txt(&text).with_context(|| "canary file is not in SBFB format")?;
            nexus_shell_daemon_core::canary::verify_canary(&canary)
                .with_context(|| "signature does not validate")?;

            println!("canary OK");
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            Ok(())
        }
    }
}

async fn handle_frost(cmd: FrostCommand) -> Result<()> {
    use nexus_core_rs::canonical::{DOMAIN_WARRANT_CANARY_V1, canonical_bytes};
    use nexus_shell_daemon_core::canary::{
        CANARY_VALIDITY_DAYS, CANARY_VERSION, Canary, CanarySigned, build_signing_package,
        ceremony_aggregate, ceremony_round1, ceremony_round2,
        dkg::{generate_dkg, load_pubkey, load_share},
        format_canary_txt, today_utc, verify_canary,
    };
    use time::Duration;

    match cmd {
        FrostCommand::TrustedDealer { k, n, output_dir } => {
            std::fs::create_dir_all(&output_dir)
                .with_context(|| format!("failed to create output dir {}", output_dir.display()))?;

            let (share_files, pubkey_file) = generate_dkg(k, n)
                .with_context(|| format!("FROST trusted dealer DKG failed (K={k}, N={n})"))?;

            for sf in &share_files {
                let path = output_dir.join(format!("canary-share-{}.frost.json", sf.participant));
                let json = serde_json::to_string_pretty(sf).context("serialize share file")?;
                std::fs::write(&path, json).with_context(|| format!("write {}", path.display()))?;
                println!("  share {}: {}", sf.participant, path.display());
            }

            let pp_path = output_dir.join("canary-pubkey-package.frost.json");
            let pp_json =
                serde_json::to_string_pretty(&pubkey_file).context("serialize pubkey file")?;
            std::fs::write(&pp_path, pp_json)
                .with_context(|| format!("write {}", pp_path.display()))?;

            println!("  pubkey:   {}", pp_path.display());
            println!("  K={k}, N={n}");
            println!("  verifying key: {}", pubkey_file.verifying_key_hex);
            println!();
            println!("Distribute each share file to its participant via a");
            println!("separate secure channel. Destroy this machine's RNG");
            println!("seed after distribution (cf. WARRANT_CANARY_HARDENING §4.2).");
            Ok(())
        }

        FrostCommand::Round1 {
            share,
            commitment,
            nonces,
        } => {
            let share_json = std::fs::read_to_string(&share)
                .with_context(|| format!("read {}", share.display()))?;
            let share_file: nexus_shell_daemon_core::canary::DkgShareFile =
                serde_json::from_str(&share_json).context("parse share file")?;
            let frost_share = load_share(&share_file).context("load FROST share")?;

            let (c, n) = ceremony_round1(share_file.participant, &frost_share.key_package)
                .context("round 1 failed")?;

            let c_json = serde_json::to_string_pretty(&c).context("serialize commitment")?;
            std::fs::write(&commitment, &c_json)
                .with_context(|| format!("write {}", commitment.display()))?;

            let n_json = serde_json::to_string_pretty(&n).context("serialize nonces")?;
            std::fs::write(&nonces, &n_json)
                .with_context(|| format!("write {}", nonces.display()))?;

            println!("  participant: {}", share_file.participant);
            println!("  commitment:  {}", commitment.display());
            println!(
                "  nonces:      {} (SECRET — do not share)",
                nonces.display()
            );
            Ok(())
        }

        FrostCommand::BuildSigningPackage {
            commitments,
            pubkey_package: pp_path,
            headline,
            output,
        } => {
            let mut commitment_list = Vec::with_capacity(commitments.len());
            for path in &commitments {
                let json = std::fs::read_to_string(path)
                    .with_context(|| format!("read {}", path.display()))?;
                let c: nexus_shell_daemon_core::canary::CeremonyCommitment =
                    serde_json::from_str(&json).context("parse commitment")?;
                commitment_list.push(c);
            }

            let pp_json = std::fs::read_to_string(&pp_path)
                .with_context(|| format!("read {}", pp_path.display()))?;
            let pp_file: nexus_shell_daemon_core::canary::DkgPubkeyFile =
                serde_json::from_str(&pp_json).context("parse pubkey package")?;

            let date = today_utc();
            let next_update = date.saturating_add(Duration::days(CANARY_VALIDITY_DAYS));
            let signed = CanarySigned {
                version: CANARY_VERSION,
                date: format!(
                    "{:04}-{:02}-{:02}",
                    date.year(),
                    u8::from(date.month()),
                    date.day()
                ),
                headline: headline.clone(),
                next_update: format!(
                    "{:04}-{:02}-{:02}",
                    next_update.year(),
                    u8::from(next_update.month()),
                    next_update.day()
                ),
                pubkey_hex: pp_file.verifying_key_hex,
            };
            let canonical =
                canonical_bytes(&signed, DOMAIN_WARRANT_CANARY_V1).context("canonical bytes")?;

            let sp = build_signing_package(&commitment_list, &canonical)
                .context("build signing package")?;

            let sp_json = serde_json::to_string_pretty(&sp).context("serialize signing package")?;
            std::fs::write(&output, &sp_json)
                .with_context(|| format!("write {}", output.display()))?;

            println!("  signing package: {}", output.display());
            println!("  headline:        {}", headline);
            println!("  commitments:     {}", commitments.len());
            Ok(())
        }

        FrostCommand::Round2 {
            share,
            nonces,
            signing_package,
            output,
        } => {
            let share_json = std::fs::read_to_string(&share)
                .with_context(|| format!("read {}", share.display()))?;
            let share_file: nexus_shell_daemon_core::canary::DkgShareFile =
                serde_json::from_str(&share_json).context("parse share")?;
            let frost_share = load_share(&share_file).context("load share")?;

            let nonces_json = std::fs::read_to_string(&nonces)
                .with_context(|| format!("read {}", nonces.display()))?;
            let nonces_data: nexus_shell_daemon_core::canary::CeremonyNonces =
                serde_json::from_str(&nonces_json).context("parse nonces")?;

            let sp_json = std::fs::read_to_string(&signing_package)
                .with_context(|| format!("read {}", signing_package.display()))?;
            let sp: nexus_shell_daemon_core::canary::CeremonySigningPackage =
                serde_json::from_str(&sp_json).context("parse signing package")?;

            let ss = ceremony_round2(&nonces_data, &sp, &frost_share.key_package)
                .context("round 2 failed")?;

            let ss_json = serde_json::to_string_pretty(&ss).context("serialize sig share")?;
            std::fs::write(&output, &ss_json)
                .with_context(|| format!("write {}", output.display()))?;

            println!("  participant:  {}", share_file.participant);
            println!("  sig share:    {}", output.display());
            println!("  (destroy nonces file now)");
            Ok(())
        }

        FrostCommand::Aggregate {
            pubkey_package,
            signing_package,
            shares,
            headline,
            output,
        } => {
            let pp_json = std::fs::read_to_string(&pubkey_package)
                .with_context(|| format!("read {}", pubkey_package.display()))?;
            let pp_file: nexus_shell_daemon_core::canary::DkgPubkeyFile =
                serde_json::from_str(&pp_json).context("parse pubkey package")?;
            let pubkey = load_pubkey(&pp_file).context("load pubkey")?;

            let sp_json = std::fs::read_to_string(&signing_package)
                .with_context(|| format!("read {}", signing_package.display()))?;
            let sp: nexus_shell_daemon_core::canary::CeremonySigningPackage =
                serde_json::from_str(&sp_json).context("parse signing package")?;

            let mut sig_shares = Vec::with_capacity(shares.len());
            for path in &shares {
                let json = std::fs::read_to_string(path)
                    .with_context(|| format!("read {}", path.display()))?;
                let ss: nexus_shell_daemon_core::canary::CeremonySignatureShare =
                    serde_json::from_str(&json).context("parse sig share")?;
                sig_shares.push(ss);
            }

            let sig = ceremony_aggregate(&sp, &sig_shares, pubkey.package())
                .context("aggregate failed")?;

            let date = today_utc();
            let next_update = date.saturating_add(Duration::days(CANARY_VALIDITY_DAYS));
            let canary = Canary {
                signed: CanarySigned {
                    version: CANARY_VERSION,
                    date: format!(
                        "{:04}-{:02}-{:02}",
                        date.year(),
                        u8::from(date.month()),
                        date.day()
                    ),
                    headline,
                    next_update: format!(
                        "{:04}-{:02}-{:02}",
                        next_update.year(),
                        u8::from(next_update.month()),
                        next_update.day()
                    ),
                    pubkey_hex: pp_file.verifying_key_hex.clone(),
                },
                signature_hex: hex::encode(sig),
            };

            verify_canary(&canary).context("self-verification of aggregated canary")?;

            let txt = format_canary_txt(&canary);
            std::fs::write(&output, &txt).with_context(|| format!("write {}", output.display()))?;

            println!("FROST canary aggregated and verified.");
            println!("  date:         {}", canary.signed.date);
            println!("  headline:     {}", canary.signed.headline);
            println!("  next update:  {}", canary.signed.next_update);
            println!("  pubkey:       {}", canary.signed.pubkey_hex);
            println!("  output:       {}", output.display());
            Ok(())
        }
    }
}

async fn handle_init(paths: &ShellDaemonPaths) -> Result<()> {
    paths
        .ensure_dirs()
        .context("failed to create shell-daemon directories")?;
    let db_path = paths.root.join("coordinator.db");
    let _db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
        .map_err(|e| anyhow::anyhow!("coordinator DB open failed: {e}"))?;
    println!("Project initialized.");
    println!("  root:          {}", paths.root.display());
    println!("  coordinator.db: {}", db_path.display());
    println!("  config:        {}", paths.config_file.display());
    println!("\nRun `nexus-shell-daemon start` to boot the daemon.");
    Ok(())
}

async fn handle_invite(paths: &ShellDaemonPaths, cmd: InviteCommand) -> Result<()> {
    let db_path = paths.root.join("coordinator.db");
    let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
        .map_err(|e| anyhow::anyhow!("coordinator DB open failed: {e}"))?;
    let ledger = nexus_coordinator_rs::invite::InviteLedger::new(&db);
    match cmd {
        InviteCommand::Create => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            let id = format!("inv-{:08x}-{:016x}-0001", 0u32, now as u64);
            let expires = now + 7 * 86400;
            let req = nexus_coordinator_rs::invite::MintRequest::new(
                &id,
                "json",
                "project",
                "local",
                "local-project",
                expires,
            );
            let record = ledger.mint(&req).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("{}", record.id);
        }
        InviteCommand::List { limit } => {
            let invites = ledger.list(limit).map_err(|e| anyhow::anyhow!("{e}"))?;
            if invites.is_empty() {
                println!("No invitations.");
            } else {
                for inv in &invites {
                    let status = if inv.revoked_at.is_some() {
                        "revoked"
                    } else {
                        "active"
                    };
                    println!("{} [{}] created_at={}", inv.id, status, inv.created_at);
                }
                println!("\n{} invitation(s) total.", invites.len());
            }
        }
        InviteCommand::Revoke { id } => {
            let revoked = ledger.revoke(&id).map_err(|e| anyhow::anyhow!("{e}"))?;
            if revoked {
                println!("Revoked: {id}");
            } else {
                println!("Not found or already revoked: {id}");
            }
        }
    }
    Ok(())
}

async fn handle_quarantine(paths: &ShellDaemonPaths, cmd: QuarantineCommand) -> Result<()> {
    let db_path = paths.root.join("coordinator.db");
    let db = nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
        .map_err(|e| anyhow::anyhow!("coordinator DB open failed: {e}"))?;
    let queue = nexus_coordinator_rs::quarantine_queue::QuarantineQueue::new(&db, 900);
    match cmd {
        QuarantineCommand::List => {
            let entries = queue.list_pending().map_err(|e| anyhow::anyhow!("{e}"))?;
            if entries.is_empty() {
                println!("Quarantine queue empty.");
            } else {
                for e in &entries {
                    println!(
                        "  [{}] sender={} received_at={}",
                        e.id, e.sender_pubkey_hex, e.received_at
                    );
                }
                println!("\n{} pending entry(ies).", entries.len());
            }
        }
        QuarantineCommand::Flush { row_id } => {
            let flushed = queue.flush(row_id).map_err(|e| anyhow::anyhow!("{e}"))?;
            if flushed {
                println!("Flushed entry {row_id}.");
            } else {
                println!("Entry {row_id} not found or already processed.");
            }
        }
        QuarantineCommand::Drop { row_id } => {
            let dropped = queue
                .drop_entry(row_id)
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            if dropped {
                println!("Dropped entry {row_id}.");
            } else {
                println!("Entry {row_id} not found or already processed.");
            }
        }
    }
    Ok(())
}

async fn handle_capability(paths: &ShellDaemonPaths, cmd: CapabilityCommand) -> Result<()> {
    let cap_path = paths.root.join("capabilities.toml");
    match cmd {
        CapabilityCommand::List => {
            let store = nexus_coordinator_rs::capability_store::CapabilityStore::load(&cap_path);
            let trail = store.audit_trail();
            if trail.is_empty() {
                println!("No capabilities configured.");
            } else {
                for (name, enabled, actor, ts) in &trail {
                    let status = if *enabled { "ON" } else { "OFF" };
                    println!("  {name}: {status} (by {actor} at {ts})");
                }
            }
        }
        CapabilityCommand::Enable { name } => {
            let mut store =
                nexus_coordinator_rs::capability_store::CapabilityStore::load(&cap_path);
            store
                .enable(&name, "cli")
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Enabled: {name}");
        }
        CapabilityCommand::Disable { name } => {
            let mut store =
                nexus_coordinator_rs::capability_store::CapabilityStore::load(&cap_path);
            store.disable(&name).map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Disabled: {name}");
        }
    }
    Ok(())
}

/// Uniform placeholder output for unimplemented subcommands.
fn print_stub(name: &str, phase: &str, args: &[(&str, &str)]) {
    println!("nexus-shell-daemon v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_shell_daemon_core::VERSION);
    println!("  subcommand:   {name}");
    println!("  status:       not yet implemented, see Sprint 7 {phase}");
    if !args.is_empty() {
        println!("  args:");
        for (k, v) in args {
            println!("    {k} = {v}");
        }
    }
}

#[cfg(test)]
mod handler_tests {
    use nexus_shell_daemon_core::config::ShellDaemonPaths;
    use tempfile::tempdir;

    fn test_paths(root: &std::path::Path) -> ShellDaemonPaths {
        ShellDaemonPaths::resolve(Some(root.join("config.toml"))).expect("resolve paths")
    }

    #[tokio::test]
    async fn init_creates_db() {
        let tmp = tempdir().expect("tempdir");
        let paths = test_paths(tmp.path());
        super::handle_init(&paths).await.expect("init");
        let db_path = paths.root.join("coordinator.db");
        assert!(db_path.exists(), "coordinator.db must exist after init");
        nexus_coordinator_rs::db::CoordinatorDb::open(&db_path)
            .expect("DB must be openable after init");
    }

    #[tokio::test]
    async fn invite_create_list_revoke_cycle() {
        let tmp = tempdir().expect("tempdir");
        let paths = test_paths(tmp.path());
        super::handle_init(&paths).await.expect("init");

        super::handle_invite(&paths, super::InviteCommand::Create)
            .await
            .expect("invite create");
        super::handle_invite(&paths, super::InviteCommand::List { limit: 50 })
            .await
            .expect("invite list");
        super::handle_invite(
            &paths,
            super::InviteCommand::Revoke {
                id: "nonexistent".into(),
            },
        )
        .await
        .expect("invite revoke nonexistent");
    }

    #[tokio::test]
    async fn quarantine_list_empty() {
        let tmp = tempdir().expect("tempdir");
        let paths = test_paths(tmp.path());
        super::handle_init(&paths).await.expect("init");
        super::handle_quarantine(&paths, super::QuarantineCommand::List)
            .await
            .expect("quarantine list");
    }

    #[tokio::test]
    async fn capability_enable_disable_cycle() {
        let tmp = tempdir().expect("tempdir");
        let paths = test_paths(tmp.path());
        super::handle_init(&paths).await.expect("init");
        super::handle_capability(
            &paths,
            super::CapabilityCommand::Enable {
                name: "mcp_server_expose".into(),
            },
        )
        .await
        .expect("capability enable");
        super::handle_capability(&paths, super::CapabilityCommand::List)
            .await
            .expect("capability list");
        super::handle_capability(
            &paths,
            super::CapabilityCommand::Disable {
                name: "mcp_server_expose".into(),
            },
        )
        .await
        .expect("capability disable");
    }
}
