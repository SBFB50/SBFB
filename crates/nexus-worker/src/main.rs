//! # nexus-worker
//!
//! SBFB single-binary GPU contributor for the P2P compute network.
//!
//! Runs on each volunteer's machine, connects to the projects
//! they have opted into via their allowlist, and serves LLM
//! inference tasks via a local Ollama backend.
//!
//! ## Sprint 3 architecture (2026-04-10)
//!
//! This binary is a thin CLI + TUI wrapper around the headless
//! engine living in [`nexus_worker_core`]. The split is
//! deliberate: the engine must run and be testable without any
//! terminal, so the binary stays focused on argument parsing,
//! subscriber wiring, and presentation.
//!
//! ## W1 status
//!
//! W1 locks the CLI shape only. Every subcommand currently
//! prints a "not yet implemented" placeholder that names the
//! wave responsible for it. The real handlers land incrementally
//! through W3..W12. Do not remove the placeholders — the CLI
//! tests in `cli.rs` assert the shape, and the e2e test in W12
//! will assert the wiring.

mod cli;

use anyhow::{Context, Result};
use clap::Parser;
use nexus_core_rs::KeyPair;
use nexus_worker_core::allowlist::{Allowlist, NewProject};
use nexus_worker_core::config::{WorkerConfig, WorkerPaths};
use nexus_worker_core::engine::{Engine, EngineBoot, WorkerState};
use nexus_worker_core::invite::{current_unix_secs, Invite};
use tracing_subscriber::{fmt, EnvFilter};

use cli::{Cli, Command, ConfigCommand, ProjectsCommand};

#[tokio::main]
async fn main() -> Result<()> {
    // W11 will replace this with a richer subscriber (file
    // appender, JSON mode, TUI widget sink). For W1 we just
    // honour RUST_LOG and fall back to info.
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    tracing::debug!(
        core_version = nexus_worker_core::VERSION,
        "nexus-worker parsed CLI"
    );

    // Every subcommand that touches disk state needs the resolved
    // WorkerPaths. Computing it once here keeps the `--config`
    // override consistent across subcommands.
    let paths = WorkerPaths::resolve(cli.config.clone())
        .context("could not resolve worker paths for this platform")?;

    match cli.command {
        Command::Register { name } => handle_register(&paths, name).await,
        Command::Start { tui, headless } => handle_start(&paths, tui, headless).await,
        Command::Join { invite } => handle_join(&paths, invite).await,
        Command::Projects(cmd) => handle_projects(&paths, cmd).await,
        Command::Browse => handle_browse(&paths).await,
        Command::Stats => handle_stats(&paths).await,
        Command::Config(cmd) => handle_config(&paths, cmd).await,
    }
}

// -----------------------------------------------------------------
// Placeholders — every handler is a no-op that prints the wave
// that will implement it. This keeps the binary buildable and
// lets the CLI tests pass while the real code is written wave by
// wave. Do not add any real logic here; put it in
// `nexus-worker-core` and call into it from these handlers.
// -----------------------------------------------------------------

async fn handle_register(paths: &WorkerPaths, name: Option<String>) -> Result<()> {
    // Refuse to clobber an existing registration — users that
    // want to rotate keys must explicitly remove worker.toml
    // first. This protects accidental loss of the Ed25519
    // identity (and therefore any kudos associated with it).
    if paths.config_file.exists() {
        anyhow::bail!(
            "worker already registered at {}\n\
             delete it first if you really want to create a new identity",
            paths.config_file.display()
        );
    }

    paths
        .ensure_dirs()
        .context("failed to create worker data directories")?;

    // Build a default config, then stamp the optional name into
    // the identity section.
    let mut cfg = WorkerConfig::default();
    if let Some(name) = name {
        cfg.identity.name = name;
    }

    // Generate (or load, if the key file exists independently
    // of the config) the Ed25519 identity.
    let key_path = cfg.resolve_secret_key_path(paths);
    let keypair = KeyPair::load_or_generate(&key_path).context(format!(
        "failed to load or generate worker keypair at {}",
        key_path.display()
    ))?;

    cfg.save(&paths.config_file)
        .context("failed to write worker.toml")?;

    let pub_hex = hex::encode(keypair.public_bytes());
    println!("nexus-worker v{}", env!("CARGO_PKG_VERSION"));
    println!("  registered as:    {}", cfg.identity.name);
    println!("  public key (hex): {pub_hex}");
    println!("  config:           {}", paths.config_file.display());
    println!("  secret key:       {}", key_path.display());
    println!("  data dir:         {}", paths.data_dir.display());
    println!();
    println!("Next steps:");
    println!("  nexus-worker join <invite>    enroll in a project");
    println!("  nexus-worker start            begin serving tasks");
    Ok(())
}

async fn handle_start(paths: &WorkerPaths, tui: bool, _headless: bool) -> Result<()> {
    // W10 will add a real TUI frontend on top of the engine.
    // Until then, --tui degrades to headless with a warning so
    // users are not surprised when the switch does nothing.
    if tui {
        tracing::warn!("--tui is a no-op until Sprint 3 W10 lands; running headless");
    }

    // Load config + keypair. Both are required — a missing
    // worker.toml means the user hasn't run `register` yet.
    let cfg = WorkerConfig::load_required(&paths.config_file).context(
        "worker not registered; run `nexus-worker register` first or pass --config <PATH>",
    )?;
    let key_path = cfg.resolve_secret_key_path(paths);
    let keypair =
        KeyPair::load_or_generate(&key_path).context("failed to load worker keypair from disk")?;

    // Allowlist is opened here so a broken DB surfaces before
    // the engine starts touching the network.
    paths.ensure_dirs()?;
    let allowlist = Allowlist::open(paths.default_allowlist_db())
        .context("failed to open allowlist database")?;

    println!("nexus-worker v{}", env!("CARGO_PKG_VERSION"));
    println!("  worker:  {}", cfg.identity.name);
    println!("  pubkey:  {}", hex::encode(keypair.public_bytes()));
    println!("  ollama:  {}", cfg.ollama.endpoint);
    println!("  config:  {}", paths.config_file.display());
    println!();

    // Build and run the engine.
    let boot = EngineBoot {
        worker_config: cfg,
        keypair,
        allowlist,
    };
    let mut engine = Engine::new(boot).await.context("engine boot failed")?;
    println!("  node id: {}", engine.node_id());

    let gpus = engine.gpu_info();
    if gpus.is_empty() {
        println!("  gpu:     none visible (CPU-only mode)");
    } else {
        for g in gpus {
            let vram_gib = g.vram_total_bytes as f64 / 1024.0 / 1024.0 / 1024.0;
            println!(
                "  gpu:     [{}] {} ({:.1} GiB, backend={})",
                g.index, g.name, vram_gib, g.backend
            );
        }
    }

    // Wire graceful Ctrl+C → Engine shutdown.
    let shutdown_tx = engine
        .take_shutdown_sender()
        .expect("engine shutdown sender available at first take");
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("ctrl+c received, sending engine shutdown signal");
            let _ = shutdown_tx.send(());
        }
    });

    // Minimal state observer so the operator sees transitions
    // live in the terminal without needing the W10 TUI.
    let mut state_rx = engine.state_rx();
    tokio::spawn(async move {
        let mut last = state_rx.borrow().clone();
        println!("  state:   {last}");
        while state_rx.changed().await.is_ok() {
            let current = state_rx.borrow().clone();
            if current != last {
                println!("  state:   {current}");
                last = current.clone();
                if matches!(current, WorkerState::Shutdown) {
                    break;
                }
            }
        }
    });

    println!("  (press ctrl+c to shut down)");
    println!();

    engine
        .run_until_shutdown()
        .await
        .context("engine loop exited with an error")?;

    println!();
    println!("nexus-worker exited cleanly.");
    Ok(())
}

async fn handle_join(paths: &WorkerPaths, invite: String) -> Result<()> {
    // Must be registered first — the join flow does not create
    // a config on its own. Clear error points at `register`.
    let _cfg = WorkerConfig::load_required(&paths.config_file).context(
        "worker not registered; run `nexus-worker register` first or pass --config <PATH>",
    )?;

    // Decode + verify signature (decode calls verify_signature
    // internally and refuses unsupported versions).
    let invite = Invite::decode(invite.trim()).context("failed to decode invite token")?;

    // Reject expired tokens with a dedicated error so the CLI
    // shows "expired at X, now Y" instead of a generic decode
    // failure.
    invite
        .ensure_not_expired(current_unix_secs())
        .context("invite token is expired")?;

    // Enroll in the allowlist. AlreadyEnrolled is downgraded to
    // a friendly message because users frequently re-paste an
    // invite to make sure it worked.
    paths.ensure_dirs()?;
    let db = Allowlist::open(paths.default_allowlist_db())
        .context("failed to open allowlist database")?;

    let new = NewProject {
        id: invite.payload.project_id.clone(),
        name: invite.payload.project_name.clone(),
        enabled: true,
        budget_joules: 0,
    };
    match db.enroll(new) {
        Ok(()) => {
            println!("nexus-worker v{}", env!("CARGO_PKG_VERSION"));
            println!("  joined project:   {}", invite.payload.project_name);
            println!("  project id:       {}", invite.payload.project_id);
            println!(
                "  coordinator pub:  {}",
                hex::encode(invite.payload.coordinator_pubkey)
            );
            println!(
                "  scope:            {}",
                if invite.payload.scope.can_serve_tasks() {
                    "worker (can serve tasks)"
                } else {
                    "observer (read-only)"
                }
            );
            println!("  expires at (unix): {}", invite.payload.expires_at_unix);
            println!();
            println!("Next: `nexus-worker start` to begin serving tasks.");
        }
        Err(nexus_worker_core::allowlist::AllowlistError::AlreadyEnrolled(id)) => {
            println!("already enrolled in project {id} — nothing to do");
        }
        Err(e) => return Err(e).context("failed to enroll project in allowlist"),
    }
    Ok(())
}

async fn handle_projects(paths: &WorkerPaths, cmd: ProjectsCommand) -> Result<()> {
    // Every projects subcommand needs the allowlist. Open it
    // once per invocation (no shared state — each CLI call is
    // its own process).
    paths.ensure_dirs()?;
    let db = Allowlist::open(paths.default_allowlist_db())
        .context("failed to open allowlist database")?;

    match cmd {
        ProjectsCommand::List => {
            let rows = db.list().context("failed to list projects")?;
            if rows.is_empty() {
                println!("no projects enrolled yet. use `nexus-worker join <invite>`.");
                return Ok(());
            }
            println!(
                "{:<24} {:<30} {:<10} {:<15} {:<10} {:<10}",
                "ID", "NAME", "ENABLED", "BUDGET(J/day)", "TASKS", "USED(J)"
            );
            for p in rows {
                let budget = if p.budget_joules == 0 {
                    "unlimited".to_string()
                } else {
                    p.budget_joules.to_string()
                };
                println!(
                    "{:<24} {:<30} {:<10} {:<15} {:<10} {:<10}",
                    truncate(&p.id, 24),
                    truncate(&p.name, 30),
                    p.enabled,
                    budget,
                    p.tasks_completed,
                    p.joules_used,
                );
            }
        }
        ProjectsCommand::Enable { project_id } => {
            db.enable(&project_id).context("failed to enable project")?;
            println!("enabled: {project_id}");
        }
        ProjectsCommand::Disable { project_id } => {
            db.disable(&project_id)
                .context("failed to disable project")?;
            println!("disabled: {project_id}");
        }
        ProjectsCommand::Budget { project_id, joules } => {
            db.set_budget(&project_id, joules)
                .context("failed to set project budget")?;
            if joules == 0 {
                println!("budget cleared (unlimited): {project_id}");
            } else {
                println!("budget set to {joules} J/day: {project_id}");
            }
        }
    }
    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let trunc: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{trunc}…")
    }
}

async fn handle_browse(_paths: &WorkerPaths) -> Result<()> {
    print_stub("browse", "post-W9 (curator list discovery)", &[]);
    Ok(())
}

async fn handle_stats(paths: &WorkerPaths) -> Result<()> {
    // W4/W6/W7 will fill in real numbers. For W3 we at least
    // report whether the worker is registered and where its
    // identity lives.
    match WorkerConfig::load_required(&paths.config_file) {
        Ok(cfg) => {
            let key_path = cfg.resolve_secret_key_path(paths);
            let key_status = if key_path.exists() {
                "present"
            } else {
                "missing"
            };
            print_stub(
                "stats",
                "W4 (GPU) + W6 (state) + W7 (allowlist)",
                &[
                    ("name", cfg.identity.name.as_str()),
                    ("config", &paths.config_file.display().to_string()),
                    ("secret_key", key_status),
                    ("ollama", cfg.ollama.endpoint.as_str()),
                ],
            );
        }
        Err(e) => {
            print_stub(
                "stats",
                "W4 (GPU) + W6 (state) + W7 (allowlist)",
                &[("status", "not registered"), ("reason", &e.to_string())],
            );
        }
    }
    Ok(())
}

async fn handle_config(_paths: &WorkerPaths, cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Get { key } => {
            print_stub("config get", "W3 (config)", &[("key", key.as_str())])
        }
        ConfigCommand::Set { key, value } => print_stub(
            "config set",
            "W3 (config)",
            &[("key", key.as_str()), ("value", value.as_str())],
        ),
    }
    Ok(())
}

/// Uniform placeholder output for unimplemented subcommands.
///
/// Kept as a free function so the stub format is trivially
/// grep-able across all handlers: when a real implementation
/// lands, the grep for `print_stub` will flag any handler still
/// on the stub path.
fn print_stub(name: &str, wave: &str, args: &[(&str, &str)]) {
    println!("nexus-worker v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_worker_core::VERSION);
    println!("  subcommand:   {name}");
    println!("  status:       not yet implemented, see Sprint 3 {wave}");
    if !args.is_empty() {
        println!("  args:");
        for (k, v) in args {
            println!("    {k} = {v}");
        }
    }
}
