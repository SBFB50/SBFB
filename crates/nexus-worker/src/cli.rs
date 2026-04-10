//! Command-line interface for the `nexus-worker` binary.
//!
//! This module defines the entire public CLI surface using
//! `clap`'s derive API. It contains **no logic** — every
//! subcommand handler lives next to the wave that implements it
//! (register/join in W8, start in W9, projects in W7, etc.). W1
//! just locks the shape so later waves can plug their code in
//! without renegotiating the flag layout.
//!
//! The Sprint 3 plan in `magical-marinating-phoenix.md` lists the
//! full command set; keep this file in sync whenever the plan
//! changes.

use clap::{Parser, Subcommand};

/// Top-level parser for `nexus-worker`.
///
/// `--version` is wired automatically to `CARGO_PKG_VERSION`; the
/// `about` text is deliberately short so `--help` stays readable.
#[derive(Debug, Parser)]
#[command(
    name = "nexus-worker",
    version,
    about = "SBFB P2P compute worker — contribute GPU time to projects you trust",
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    /// Path to the worker config file.
    ///
    /// Defaults to `~/.nexus-grid/worker.toml` (platform-resolved
    /// via the `directories` crate in W3). Set this to point at a
    /// test fixture during e2e tests or to run multiple worker
    /// identities from the same machine.
    #[arg(long, value_name = "PATH", global = true)]
    pub config: Option<std::path::PathBuf>,

    /// Increase log verbosity (repeatable: -v info, -vv debug,
    /// -vvv trace).
    ///
    /// W11 wires this into the `tracing-subscriber` `EnvFilter`.
    /// Ignored by W1.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Run the engine with a deterministic no-network Ollama
    /// stub. Sprint 4 Phase D added this for the hermetic
    /// end-to-end test suite so the worker can execute tasks
    /// on a CI host without a real Ollama install.
    ///
    /// When set, the engine's `OllamaClient` is replaced with
    /// [`nexus_worker_core::ollama::StubOllama`], which
    /// reports the daemon as "ready" and returns a canned
    /// `STUB[model]: <prompt>` response for every generate
    /// call.
    #[arg(long, global = true, default_value_t = false)]
    pub stub_ollama: bool,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
///
/// Order matches the order they land through Sprint 3, not
/// alphabetical. Any new command added in a future wave should
/// keep the shape stable so external scripts don't break.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate a persistent worker identity and seed the config
    /// file.
    ///
    /// Creates `~/.nexus-grid/worker.toml`, generates a fresh
    /// Ed25519 keypair via
    /// `nexus_core_rs::KeyPair::load_or_generate`, and records
    /// the public key for future `join` / `start` commands.
    ///
    /// Wave: W3 (config) + W4 (identity persistence).
    Register {
        /// Human-readable name stored in the config for your
        /// own reference. Never shared over the network.
        #[arg(long, value_name = "NAME")]
        name: Option<String>,
    },

    /// Start the worker engine and begin serving tasks from
    /// enrolled projects.
    ///
    /// Runs the state machine (W6), polls project docs for new
    /// tasks (W9), dispatches them to the local Ollama backend
    /// (W5), and publishes signed results back.
    ///
    /// Use `--headless` for an unattended daemon mode (logs to
    /// stdout / file), or `--tui` to launch the interactive
    /// ratatui dashboard. Defaults to `--tui` when stdout is a
    /// terminal, `--headless` otherwise.
    ///
    /// Wave: W9 (engine loop) + W10 (TUI layer) + W11 (logging).
    Start {
        /// Force the ratatui dashboard even in non-interactive
        /// shells.
        #[arg(long, conflicts_with = "headless")]
        tui: bool,

        /// Force headless mode (no TUI). Recommended for
        /// systemd / docker deployments.
        #[arg(long, conflicts_with = "tui")]
        headless: bool,
    },

    /// Accept a project invite token and enroll in the project.
    ///
    /// The token encodes the coordinator's endpoint address, the
    /// project namespace id, the granted scope and an expiry.
    /// On success the project is added to the local allowlist
    /// with `enabled = true`.
    ///
    /// Wave: W8 (invite tokens).
    Join {
        /// Invite token string (format: `nx1...` base32).
        #[arg(value_name = "INVITE")]
        invite: String,
    },

    /// Manage the list of projects this worker is willing to
    /// serve.
    ///
    /// Wave: W7 (allowlist SQLite).
    #[command(subcommand)]
    Projects(ProjectsCommand),

    /// Browse public projects discoverable through the DHT.
    ///
    /// Queries the curator lists configured for this worker,
    /// resolves the corresponding project manifests, and prints
    /// a filtered listing. Projects require an explicit
    /// subsequent `join` to become active.
    ///
    /// Wave: post-W9 (depends on curator list wrapping).
    Browse,

    /// Display runtime statistics for this worker: GPU state,
    /// engine state, per-project task counts, total kudos,
    /// current uptime.
    ///
    /// Wave: W4 (GPU) + W6 (engine state) + W7 (allowlist).
    Stats,

    /// Read or update worker config values.
    ///
    /// The key format is dotted (e.g. `ollama.endpoint`,
    /// `gpu.max_vram_fraction`).
    ///
    /// Wave: W3 (config).
    #[command(subcommand)]
    Config(ConfigCommand),
}

/// Subcommands for `nexus-worker projects ...`.
#[derive(Debug, Subcommand)]
pub enum ProjectsCommand {
    /// List every enrolled project with its status.
    List,

    /// Mark a project as active (the engine will claim its
    /// tasks).
    Enable {
        /// Project id as reported by `projects list`.
        #[arg(value_name = "PROJECT_ID")]
        project_id: String,
    },

    /// Mark a project as paused (no new claims, in-flight work
    /// finishes).
    Disable {
        /// Project id as reported by `projects list`.
        #[arg(value_name = "PROJECT_ID")]
        project_id: String,
    },

    /// Set the per-day energy budget for a project, in joules.
    ///
    /// Once the budget is exhausted the worker stops claiming
    /// new tasks for that project until the daily reset.
    Budget {
        /// Project id.
        #[arg(value_name = "PROJECT_ID")]
        project_id: String,
        /// Budget, in joules.
        #[arg(value_name = "JOULES")]
        joules: u64,
    },
}

/// Subcommands for `nexus-worker config ...`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print a single config value to stdout.
    Get {
        /// Dotted config key (e.g. `ollama.endpoint`).
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Update a single config value in the worker config file.
    Set {
        /// Dotted config key.
        #[arg(value_name = "KEY")]
        key: String,
        /// New value (parsed into the existing field type).
        #[arg(value_name = "VALUE")]
        value: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_definition_is_valid() {
        // Runs clap's compile-time style assertions against the
        // derive macros. Catches name collisions, conflicting
        // aliases, required-arg ordering issues, etc.
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_register_without_name() {
        let cli = Cli::try_parse_from(["nexus-worker", "register"]).unwrap();
        match cli.command {
            Command::Register { name } => assert!(name.is_none()),
            other => panic!("expected Register, got {other:?}"),
        }
    }

    #[test]
    fn parses_register_with_name() {
        let cli =
            Cli::try_parse_from(["nexus-worker", "register", "--name", "rtx5080-home"]).unwrap();
        match cli.command {
            Command::Register { name } => {
                assert_eq!(name.as_deref(), Some("rtx5080-home"));
            }
            other => panic!("expected Register, got {other:?}"),
        }
    }

    #[test]
    fn start_flags_are_mutually_exclusive() {
        let err = Cli::try_parse_from(["nexus-worker", "start", "--tui", "--headless"]);
        assert!(err.is_err(), "--tui and --headless must conflict");
    }

    #[test]
    fn start_accepts_either_flag() {
        let cli = Cli::try_parse_from(["nexus-worker", "start", "--headless"]).unwrap();
        match cli.command {
            Command::Start { tui, headless } => {
                assert!(!tui);
                assert!(headless);
            }
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn parses_join_with_invite_token() {
        let cli = Cli::try_parse_from(["nexus-worker", "join", "nx1abcdef"]).unwrap();
        match cli.command {
            Command::Join { invite } => assert_eq!(invite, "nx1abcdef"),
            other => panic!("expected Join, got {other:?}"),
        }
    }

    #[test]
    fn parses_projects_enable() {
        let cli = Cli::try_parse_from(["nexus-worker", "projects", "enable", "proj-123"]).unwrap();
        match cli.command {
            Command::Projects(ProjectsCommand::Enable { project_id }) => {
                assert_eq!(project_id, "proj-123");
            }
            other => panic!("expected Projects::Enable, got {other:?}"),
        }
    }

    #[test]
    fn parses_projects_budget_with_numeric_value() {
        let cli =
            Cli::try_parse_from(["nexus-worker", "projects", "budget", "proj-123", "1800000"])
                .unwrap();
        match cli.command {
            Command::Projects(ProjectsCommand::Budget { project_id, joules }) => {
                assert_eq!(project_id, "proj-123");
                assert_eq!(joules, 1_800_000);
            }
            other => panic!("expected Projects::Budget, got {other:?}"),
        }
    }

    #[test]
    fn config_get_and_set_parse() {
        let get =
            Cli::try_parse_from(["nexus-worker", "config", "get", "ollama.endpoint"]).unwrap();
        assert!(matches!(
            get.command,
            Command::Config(ConfigCommand::Get { .. })
        ));

        let set = Cli::try_parse_from([
            "nexus-worker",
            "config",
            "set",
            "ollama.endpoint",
            "http://localhost:11434",
        ])
        .unwrap();
        match set.command {
            Command::Config(ConfigCommand::Set { key, value }) => {
                assert_eq!(key, "ollama.endpoint");
                assert_eq!(value, "http://localhost:11434");
            }
            other => panic!("expected Config::Set, got {other:?}"),
        }
    }

    #[test]
    fn global_config_flag_attaches_to_any_subcommand() {
        let cli = Cli::try_parse_from(["nexus-worker", "--config", "/tmp/fixture.toml", "stats"])
            .unwrap();
        assert_eq!(
            cli.config
                .as_deref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/tmp/fixture.toml".to_string())
        );
        assert!(matches!(cli.command, Command::Stats));
    }

    #[test]
    fn verbose_flag_counts_repetitions() {
        let cli = Cli::try_parse_from(["nexus-worker", "-vvv", "browse"]).unwrap();
        assert_eq!(cli.verbose, 3);
        assert!(matches!(cli.command, Command::Browse));
    }
}
