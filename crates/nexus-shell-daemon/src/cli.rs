// SPDX-License-Identifier: AGPL-3.0-or-later
//! Command-line interface for the `nexus-shell-daemon` binary.
//!
//! Phase A locks the CLI shape so Phase C/D/E can plug their code
//! in without renegotiating the flag layout. Every subcommand
//! handler lives in `main.rs` alongside the tokio runtime; this
//! module defines the clap types and nothing else.
//!
//! The subcommand set deliberately mirrors the `nexus-worker`
//! CLI wherever the semantics are the same (`start`, `stop`,
//! `status`, `config`). The `register` / `join` / `projects` /
//! `browse` / `stats` subcommands do not apply — the daemon has
//! no identity file (a fresh keypair is minted on every boot),
//! no allowlist, and no interactive browsing surface. Browse
//! happens in the React shell, which calls into the daemon via
//! the coordinator proxy.

use clap::{Parser, Subcommand};

/// Top-level parser for `nexus-shell-daemon`.
///
/// `--version` is wired automatically to `CARGO_PKG_VERSION`; the
/// `about` text is deliberately short so `--help` stays readable.
#[derive(Debug, Parser)]
#[command(
    name = "nexus-shell-daemon",
    version,
    about = "SBFB shell daemon — long-lived P2P process for the React shell's Browse / Curators pages",
    long_about = None,
    propagate_version = true,
)]
pub struct Cli {
    /// Path to the shell daemon config file.
    ///
    /// Defaults to `~/.nexus-grid/shell-daemon/config.toml`
    /// (platform-resolved via the `directories` crate). Set this
    /// to point at a test fixture during e2e tests or to run the
    /// daemon with an override that is easier to inspect.
    #[arg(long, value_name = "PATH", global = true)]
    pub config: Option<std::path::PathBuf>,

    /// Increase log verbosity (repeatable: -v info, -vv debug,
    /// -vvv trace).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
///
/// Only the minimum Phase A needs is implemented in `main.rs`;
/// the others return a `"not yet implemented"` error with a
/// pointer to the wave that will wire them. Keep the shape
/// stable across Sprint 7 so external scripts and the Phase E
/// coordinator proxy do not have to change mid-sprint.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the shell daemon.
    ///
    /// Phase A: boots the iroh endpoint, writes a singleton
    /// `running.json`, binds an ephemeral loopback HTTP port,
    /// and blocks on ctrl+c. Phase C will add the curator
    /// subscribe pipeline; Phase D will add pkarr browse
    /// resolution; the HTTP surface grows additively.
    Start {
        /// Force headless mode. The daemon has **no** interactive
        /// frontend — there is no TUI, no prompt, no opt-in
        /// foreground UI. The `--headless` flag is accepted for
        /// parity with `nexus-worker start --headless` but is
        /// redundant: the daemon is always headless. Passing it
        /// is a no-op.
        #[arg(long)]
        headless: bool,
    },

    /// Stop a running shell daemon.
    ///
    /// Phase A stub. Phase E will read `running.json`, locate
    /// the live pid, and send a platform-appropriate shutdown
    /// signal (SIGTERM on Unix, a shutdown HTTP request on
    /// Windows where signal handling is more awkward).
    Stop,

    /// Print the live daemon's public state to stdout.
    ///
    /// Phase A stub. Phase E will read `running.json` and curl
    /// the daemon's `/info` endpoint via the coordinator proxy
    /// (or directly, since both point at the same loopback
    /// listener).
    Status,

    /// Read or update shell daemon config values.
    ///
    /// The key format is dotted (e.g. `logging.level`,
    /// `network.api_host`). Phase A stub; Phase E will wire
    /// this to `ShellDaemonConfig::save`.
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Publish / verify the project's monthly warrant canary.
    ///
    /// Sprint 18 Phase E2. The declaration text lives at
    /// `CANARY.txt` at the repo root; the Ed25519 signature is
    /// minted with a persistent maintainer key stored at
    /// `<sbfb_home>/canary-key.key`. `publish` also broadcasts
    /// the canary on the `nexus-grid/warrant-canary/v1` gossip
    /// topic so live daemons can flag a stale canary even
    /// without re-cloning the repo.
    #[command(subcommand)]
    Canary(CanaryCommand),
}

/// Subcommands for `nexus-shell-daemon canary ...`.
///
/// Sprint 18 Phase E2. `publish` is the one flow wired in this
/// phase; `verify` is a cheap helper that re-reads `CANARY.txt`
/// and re-validates the signature, useful for CI and for the
/// `scripts/verify-canary.sh` shell wrapper.
#[derive(Debug, Subcommand)]
pub enum CanaryCommand {
    /// Build + sign + publish a fresh canary. Creates
    /// `<sbfb_home>/canary-key.key` on first run. Writes
    /// `CANARY.txt` to the output path (default: `./CANARY.txt`)
    /// and, unless `--no-gossip` is set, broadcasts the canary
    /// on the warrant canary gossip topic.
    Publish {
        /// Headline text that proves the canary was minted
        /// on-or-after today's date. Typically a major news
        /// headline of the day.
        #[arg(long, value_name = "HEADLINE")]
        headline: String,

        /// Output path for the human-readable `CANARY.txt`
        /// mirror. Defaults to `CANARY.txt` in the current
        /// working directory.
        #[arg(long, value_name = "PATH", default_value = "CANARY.txt")]
        output: std::path::PathBuf,

        /// Skip the gossip broadcast step. Useful when the
        /// operator just wants to refresh `CANARY.txt` without
        /// booting an iroh endpoint (e.g. in the monthly GitHub
        /// Action that runs in a network-restricted CI runner).
        #[arg(long)]
        no_gossip: bool,
    },

    /// Re-parse a canary file and verify its signature locally.
    /// Prints the date + headline + next-update on success; exits
    /// non-zero with a descriptive message on any parse or
    /// signature failure.
    Verify {
        /// Path to the `CANARY.txt` (or JSON) file to verify.
        /// Defaults to `CANARY.txt` in the current working
        /// directory.
        #[arg(long, value_name = "PATH", default_value = "CANARY.txt")]
        input: std::path::PathBuf,
    },
}

/// Subcommands for `nexus-shell-daemon config ...`.
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Print a single config value to stdout.
    Get {
        /// Dotted config key (e.g. `logging.level`).
        #[arg(value_name = "KEY")]
        key: String,
    },
    /// Update a single config value in the shell daemon config file.
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
        // aliases, required-arg ordering issues.
        Cli::command().debug_assert();
    }

    #[test]
    fn parses_start_without_flags() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "start"]).unwrap();
        match cli.command {
            Command::Start { headless } => assert!(!headless),
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn parses_start_with_headless() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "start", "--headless"]).unwrap();
        match cli.command {
            Command::Start { headless } => assert!(headless),
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn parses_stop() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "stop"]).unwrap();
        assert!(matches!(cli.command, Command::Stop));
    }

    #[test]
    fn parses_status() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "status"]).unwrap();
        assert!(matches!(cli.command, Command::Status));
    }

    #[test]
    fn parses_config_get() {
        let cli =
            Cli::try_parse_from(["nexus-shell-daemon", "config", "get", "logging.level"]).unwrap();
        match cli.command {
            Command::Config(ConfigCommand::Get { key }) => assert_eq!(key, "logging.level"),
            other => panic!("expected Config::Get, got {other:?}"),
        }
    }

    #[test]
    fn parses_config_set() {
        let cli = Cli::try_parse_from([
            "nexus-shell-daemon",
            "config",
            "set",
            "network.api_host",
            "127.0.0.1",
        ])
        .unwrap();
        match cli.command {
            Command::Config(ConfigCommand::Set { key, value }) => {
                assert_eq!(key, "network.api_host");
                assert_eq!(value, "127.0.0.1");
            }
            other => panic!("expected Config::Set, got {other:?}"),
        }
    }

    #[test]
    fn global_config_flag_attaches_to_any_subcommand() {
        let cli = Cli::try_parse_from([
            "nexus-shell-daemon",
            "--config",
            "/tmp/fixture.toml",
            "status",
        ])
        .unwrap();
        assert_eq!(
            cli.config
                .as_deref()
                .map(|p| p.to_string_lossy().into_owned()),
            Some("/tmp/fixture.toml".to_string())
        );
        assert!(matches!(cli.command, Command::Status));
    }

    #[test]
    fn verbose_flag_counts_repetitions() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "-vv", "start"]).unwrap();
        assert_eq!(cli.verbose, 2);
        assert!(matches!(cli.command, Command::Start { .. }));
    }
}
