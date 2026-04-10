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
use nexus_worker_core::config::{WorkerConfig, WorkerPaths};
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

async fn handle_start(paths: &WorkerPaths, tui: bool, headless: bool) -> Result<()> {
    let mode = if tui {
        "tui"
    } else if headless {
        "headless"
    } else {
        "auto"
    };

    // W9 will replace this stub with the real engine boot. For
    // now, make sure the worker is actually registered and that
    // the keypair loads — any failure here surfaces a bad config
    // before the engine-loop waves land.
    let cfg = WorkerConfig::load_required(&paths.config_file).context(
        "worker not registered; run `nexus-worker register` first or pass --config <PATH>",
    )?;
    let key_path = cfg.resolve_secret_key_path(paths);
    let keypair =
        KeyPair::load_or_generate(&key_path).context("failed to load worker keypair from disk")?;

    print_stub(
        "start",
        "W9 (engine loop) + W10 (TUI) + W11 (logging)",
        &[
            ("mode", mode),
            ("name", cfg.identity.name.as_str()),
            ("ollama", cfg.ollama.endpoint.as_str()),
        ],
    );

    // Sanity check that the core-rs link still works with the
    // persistent identity. Replaced in W9 by the full engine
    // boot sequence via create_node_with_config.
    let node = nexus_core_rs::create_node_with_config(
        nexus_core_rs::NodeConfig::default().with_secret_key(keypair.secret_bytes()),
    )
    .await?;
    println!("  iroh endpoint ready, node id: {}", node.node_id());
    node.shutdown().await?;
    Ok(())
}

async fn handle_join(_paths: &WorkerPaths, invite: String) -> Result<()> {
    print_stub("join", "W8 (invite tokens)", &[("invite", invite.as_str())]);
    Ok(())
}

async fn handle_projects(_paths: &WorkerPaths, cmd: ProjectsCommand) -> Result<()> {
    match cmd {
        ProjectsCommand::List => print_stub("projects list", "W7 (allowlist SQLite)", &[]),
        ProjectsCommand::Enable { project_id } => print_stub(
            "projects enable",
            "W7 (allowlist SQLite)",
            &[("project_id", project_id.as_str())],
        ),
        ProjectsCommand::Disable { project_id } => print_stub(
            "projects disable",
            "W7 (allowlist SQLite)",
            &[("project_id", project_id.as_str())],
        ),
        ProjectsCommand::Budget { project_id, joules } => {
            let joules_str = joules.to_string();
            print_stub(
                "projects budget",
                "W7 (allowlist SQLite)",
                &[
                    ("project_id", project_id.as_str()),
                    ("joules", joules_str.as_str()),
                ],
            );
        }
    }
    Ok(())
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
