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

use anyhow::Result;
use clap::Parser;
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

    match cli.command {
        Command::Register { name } => handle_register(name).await,
        Command::Start { tui, headless } => handle_start(tui, headless).await,
        Command::Join { invite } => handle_join(invite).await,
        Command::Projects(cmd) => handle_projects(cmd).await,
        Command::Browse => handle_browse().await,
        Command::Stats => handle_stats().await,
        Command::Config(cmd) => handle_config(cmd).await,
    }
}

// -----------------------------------------------------------------
// Placeholders — every handler is a no-op that prints the wave
// that will implement it. This keeps the binary buildable and
// lets the CLI tests pass while the real code is written wave by
// wave. Do not add any real logic here; put it in
// `nexus-worker-core` and call into it from these handlers.
// -----------------------------------------------------------------

async fn handle_register(name: Option<String>) -> Result<()> {
    print_stub(
        "register",
        "W3 (config) + keypair persistence",
        &[("name", name.as_deref().unwrap_or("<unset>"))],
    );
    Ok(())
}

async fn handle_start(tui: bool, headless: bool) -> Result<()> {
    let mode = if tui {
        "tui"
    } else if headless {
        "headless"
    } else {
        "auto"
    };
    print_stub(
        "start",
        "W9 (engine loop) + W10 (TUI) + W11 (logging)",
        &[("mode", mode)],
    );

    // Sanity check that the core-rs link still works end-to-end.
    // Replaced in W9 by the full engine boot sequence.
    let node = nexus_core_rs::create_node().await?;
    println!("  iroh endpoint ready, node id: {}", node.node_id());
    node.shutdown().await?;
    Ok(())
}

async fn handle_join(invite: String) -> Result<()> {
    print_stub(
        "join",
        "W8 (invite tokens)",
        &[("invite", invite.as_str())],
    );
    Ok(())
}

async fn handle_projects(cmd: ProjectsCommand) -> Result<()> {
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
        ProjectsCommand::Budget {
            project_id,
            joules,
        } => {
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

async fn handle_browse() -> Result<()> {
    print_stub(
        "browse",
        "post-W9 (curator list discovery)",
        &[],
    );
    Ok(())
}

async fn handle_stats() -> Result<()> {
    print_stub(
        "stats",
        "W4 (GPU) + W6 (state) + W7 (allowlist)",
        &[],
    );
    Ok(())
}

async fn handle_config(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Get { key } => print_stub(
            "config get",
            "W3 (config)",
            &[("key", key.as_str())],
        ),
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
