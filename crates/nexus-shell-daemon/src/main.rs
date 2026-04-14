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
//! - `stop` / `status` / `config` — stubs that land in Phase E
//!   alongside the coordinator proxy.
//!
//! Phase C adds the curator gossip subscribe pipeline; Phase D
//! adds pkarr browse resolution; Phase E adds the coordinator
//! proxy + the shell pages. **None of those are in Phase A.**

mod cli;
mod http;
mod logging;
#[cfg(windows)]
mod named_pipe_server;
mod runtime;
#[cfg(unix)]
mod uds_server;

use anyhow::{Context, Result};
use clap::Parser;
use nexus_shell_daemon_core::config::{ShellDaemonConfig, ShellDaemonPaths};

use cli::{Cli, Command, ConfigCommand};
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
        Command::Start { headless } => handle_start(paths, headless).await,
        Command::Stop => handle_stop(&paths).await,
        Command::Status => handle_status(&paths).await,
        Command::Config(cmd) => handle_config(&paths, cmd).await,
    }
}

async fn handle_start(paths: ShellDaemonPaths, _headless: bool) -> Result<()> {
    // Load the config to pick up any user-tuned bind host / port.
    // A missing file is fine — Phase A defaults to 127.0.0.1:0.
    let cfg = ShellDaemonConfig::load(&paths.config_file)
        .context("failed to load shell-daemon config")?;

    println!("nexus-shell-daemon v{}", env!("CARGO_PKG_VERSION"));
    println!("  core version: {}", nexus_shell_daemon_core::VERSION);
    println!("  config:       {}", paths.config_file.display());
    println!("  running.json: {}", paths.running_json.display());
    println!();

    let opts = DaemonStartOptions {
        paths,
        api_host: cfg.network.api_host.clone(),
        api_port: cfg.network.api_port,
        daemon_version: env!("CARGO_PKG_VERSION").to_string(),
        curator: cfg.curator.clone(),
    };

    let runtime = DaemonRuntime::start(opts)
        .await
        .context("daemon start failed")?;

    let addr = runtime.bound_addr();
    println!("  listening on: http://{}", addr);
    println!("  (press ctrl+c to shut down)");
    println!();

    runtime
        .wait_shutdown()
        .await
        .context("ctrl+c handler failed")?;

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

/// Uniform placeholder output for unimplemented subcommands.
///
/// Kept as a free function so the stub format is trivially
/// grep-able across every handler: when a real implementation
/// lands, `grep print_stub` flags any handler still on the stub
/// path.
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
