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
/// Only the minimum the daemon needs is implemented in `main.rs`;
/// the stub subcommands print a "not yet implemented" status and
/// exit successfully (Ok). The enum shape is kept stable so
/// external scripts and the coordinator proxy do not have to change.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Start the shell daemon.
    ///
    /// Boots the iroh endpoint, writes a singleton `running.json`,
    /// binds an ephemeral loopback HTTP port, and blocks on
    /// ctrl+c. The curator subscribe pipeline and pkarr browse
    /// resolution are layered on the same boot; the HTTP surface
    /// grows additively.
    Start {
        /// Force headless mode. The daemon has **no** interactive
        /// frontend — there is no TUI, no prompt, no opt-in
        /// foreground UI. The `--headless` flag is accepted for
        /// parity with `nexus-worker start --headless` but is
        /// redundant: the daemon is always headless. Passing it
        /// is a no-op.
        #[arg(long)]
        headless: bool,

        /// Extra CORS origins to allow (repeatable). Each value
        /// must be a valid HTTP(S) origin (`http://host[:port]`).
        /// Loopback origins are always allowed regardless.
        /// Env fallback: `NEXUS_DAEMON_CORS_ORIGINS` (comma-separated).
        #[arg(long = "cors-origin", value_name = "ORIGIN")]
        cors_origins: Vec<String>,

        /// Path to the built React shell directory (e.g. `web/dist`).
        /// When set, the daemon serves these static files on `/`
        /// without bearer auth so the browser can load the shell.
        /// Env fallback: `SBFB_WEB_ROOT`.
        #[arg(long, value_name = "PATH")]
        web_root: Option<std::path::PathBuf>,
    },

    /// Stop a running shell daemon.
    ///
    /// Stub (not yet implemented). A real implementation would
    /// read `running.json`, locate the live pid, and send a
    /// platform-appropriate shutdown signal (SIGTERM on Unix, a
    /// shutdown HTTP request on Windows where signal handling is
    /// more awkward).
    Stop,

    /// Print the live daemon's public state to stdout.
    ///
    /// Stub (not yet implemented). A real implementation would
    /// read `running.json` and curl the daemon's `/info` endpoint
    /// via the coordinator proxy (or directly, since both point at
    /// the same loopback listener).
    Status,

    /// Read or update shell daemon config values.
    ///
    /// The key format is dotted (e.g. `logging.level`,
    /// `network.api_host`). Stub (not yet implemented); a real
    /// implementation would wire this to `ShellDaemonConfig::save`.
    #[command(subcommand)]
    Config(ConfigCommand),

    /// Initialize project directory + coordinator database.
    ///
    /// Sprint 49 Phase B. Creates the shell-daemon directory
    /// structure and coordinator.db (if not already present).
    /// Does NOT start the daemon — use `start` for that.
    /// Useful for pre-configuring invites/capabilities before boot.
    Init,

    /// Manage project invitations (offline — no daemon required).
    ///
    /// Sprint 49 Phase B. Subcommands operate directly on the
    /// coordinator.db without a running daemon (G1 D3 ack).
    #[command(subcommand)]
    Invite(InviteCommand),

    /// Manage quarantine queue (offline — no daemon required).
    ///
    /// Sprint 49 Phase B.
    #[command(subcommand)]
    Quarantine(QuarantineCommand),

    /// Manage capability toggles (offline — no daemon required).
    ///
    /// Sprint 49 Phase B.
    #[command(subcommand)]
    Capability(CapabilityCommand),

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

    /// Operate an in-vivo shard session (Sprint 81 Phase I, ex-S78).
    ///
    /// The operator tool around the shard-session orchestrator:
    /// `serve` runs a transport-only `sbfb/shard/1` worker on THIS
    /// machine; `group`/`mount`/`status`/`generate`/`result`/
    /// `drop-shard` talk to the LOCAL running daemon over its
    /// hardened loopback API (running.json discovery + /auth/token
    /// bootstrap — no manual port/token plumbing).
    #[command(subcommand)]
    ShardSession(ShardSessionCommand),
}

/// Subcommands for `nexus-shell-daemon shard-session ...`.
///
/// Two-machine operator flow (the Phase J live benchmark rig):
/// 1. worker machine: `shard-session identity` → copy the pubkey hex;
/// 2. head machine:   `shard-session group --member <hex> --out group.json`;
/// 3. worker machine: `shard-session serve --group group.json` → copy the
///    printed endpoint address JSON;
/// 4. head machine:   `shard-session mount <config.json>` (workers = the
///    printed addresses), then `generate` / `result` / `drop-shard` — or
///    hand over to `scripts/acceptance/b3_shard_pipeline.sh`, which polls
///    the same routes.
#[derive(Debug, Subcommand)]
pub enum ShardSessionCommand {
    /// Print this machine's persistent shard-serve identity (pubkey hex),
    /// minting the key file on first use. Run BEFORE `group` on the head
    /// so the group can admit this worker.
    Identity {
        /// Key file override (default: `<shell-daemon-root>/shard-serve.key`).
        #[arg(long, value_name = "PATH")]
        key: Option<std::path::PathBuf>,
    },

    /// Serve `sbfb/shard/1` on this machine with the given signed group
    /// as the admission allowlist. Transport-only (echo forwarder): the
    /// real layer-block backend stays the worker's feature-gated build.
    /// Prints the endpoint address JSON the head's mount config needs,
    /// then blocks until ctrl+c.
    Serve {
        /// Path to the signed `ComputeGroupEntry` JSON (from `group --out`).
        #[arg(long, value_name = "PATH")]
        group: std::path::PathBuf,
        /// Key file override (default: `<shell-daemon-root>/shard-serve.key`).
        #[arg(long, value_name = "PATH")]
        key: Option<std::path::PathBuf>,
    },

    /// Mint the signed private compute group via the local daemon (the
    /// daemon's keypair signs; the head is added as a member
    /// automatically — it is the dialer the workers admit).
    Group {
        /// Stable group handle.
        #[arg(long, value_name = "ID")]
        group_id: String,
        /// Worker pubkey hex (repeatable), from `identity` on each worker.
        #[arg(long = "member", value_name = "PUBKEY_HEX")]
        members: Vec<String>,
        /// Write the signed group JSON here (also printed to stdout).
        #[arg(long, value_name = "PATH")]
        out: Option<std::path::PathBuf>,
    },

    /// Mount a session from a JSON config file (a `MountSessionRequest`:
    /// session_id + group + workers[addr,vram] + model) via the local
    /// daemon: placement → signed manifest → readiness barrier → live.
    Mount {
        /// Path to the mount config JSON.
        #[arg(value_name = "CONFIG_JSON")]
        config: std::path::PathBuf,
    },

    /// Read a session's live aggregate status.
    Status {
        /// The session id.
        session_id: String,
    },

    /// Drive one generation through the mounted pipeline.
    Generate {
        /// The session id.
        session_id: String,
        /// The prompt to drive.
        #[arg(long)]
        prompt: String,
    },

    /// Poll the measured result of the last driven generation.
    Result {
        /// The session id.
        session_id: String,
    },

    /// Explicitly cut the tail shard (churn probe, counted drop).
    DropShard {
        /// The session id.
        session_id: String,
    },
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

    /// FROST threshold signing operations (Sprint 30 Phase C).
    ///
    /// Air-gapped DKG + interactive signing ceremony for warrant
    /// canary Niveau 1 per `WARRANT_CANARY_HARDENING.md §4`.
    #[command(subcommand)]
    Frost(FrostCommand),
}

/// Subcommands for `nexus-shell-daemon canary frost ...`.
#[derive(Debug, Subcommand)]
pub enum FrostCommand {
    /// Generate K-of-N FROST key shares via trusted dealer.
    ///
    /// Writes `canary-share-{1..N}.frost.json` and
    /// `canary-pubkey-package.frost.json` to the output directory.
    /// Run on an air-gapped machine; distribute each share file
    /// to its participant via a separate secure channel.
    TrustedDealer {
        /// K threshold (minimum signers required).
        #[arg(long, value_name = "K", default_value = "2")]
        k: u16,
        /// N total shares to deal.
        #[arg(long, value_name = "N", default_value = "3")]
        n: u16,
        /// Output directory for share and pubkey files.
        #[arg(long, value_name = "DIR", default_value = ".")]
        output_dir: std::path::PathBuf,
    },

    /// Round 1: generate commitment + nonces from a key share.
    ///
    /// The commitment file is sent to the coordinator; the nonces
    /// file is SECRET and must be kept locally for round 2.
    Round1 {
        /// Path to the participant's `canary-share-{N}.frost.json`.
        #[arg(long, value_name = "PATH")]
        share: std::path::PathBuf,
        /// Output path for the commitment (public).
        #[arg(long, value_name = "PATH", default_value = "commitment.json")]
        commitment: std::path::PathBuf,
        /// Output path for the nonces (SECRET — local only).
        #[arg(long, value_name = "PATH", default_value = "nonces.json")]
        nonces: std::path::PathBuf,
    },

    /// Coordinator: build the signing package from K commitments.
    BuildSigningPackage {
        /// Comma-separated paths to commitment JSON files.
        #[arg(long, value_name = "PATHS", value_delimiter = ',')]
        commitments: Vec<std::path::PathBuf>,
        /// Path to the pubkey package from trusted-dealer.
        #[arg(long, value_name = "PATH")]
        pubkey_package: std::path::PathBuf,
        /// The canary headline (message to sign).
        #[arg(long, value_name = "TEXT")]
        headline: String,
        /// Output path for the signing package.
        #[arg(long, value_name = "PATH", default_value = "signing-package.json")]
        output: std::path::PathBuf,
    },

    /// Round 2: produce a signature share.
    ///
    /// Each participant runs this with their nonces, the signing
    /// package from the coordinator, and their key share.
    Round2 {
        /// Path to the participant's `canary-share-{N}.frost.json`.
        #[arg(long, value_name = "PATH")]
        share: std::path::PathBuf,
        /// Path to the nonces file from round 1 (SECRET).
        #[arg(long, value_name = "PATH")]
        nonces: std::path::PathBuf,
        /// Path to the coordinator's signing package.
        #[arg(long, value_name = "PATH")]
        signing_package: std::path::PathBuf,
        /// Output path for the signature share.
        #[arg(long, value_name = "PATH", default_value = "sig-share.json")]
        output: std::path::PathBuf,
    },

    /// Coordinator: aggregate K signature shares into a canary.
    Aggregate {
        /// Path to the pubkey package from trusted-dealer.
        #[arg(long, value_name = "PATH")]
        pubkey_package: std::path::PathBuf,
        /// Path to the signing package from build-signing-package.
        #[arg(long, value_name = "PATH")]
        signing_package: std::path::PathBuf,
        /// Comma-separated paths to signature share JSON files.
        #[arg(long, value_name = "PATHS", value_delimiter = ',')]
        shares: Vec<std::path::PathBuf>,
        /// The canary headline (must match build-signing-package).
        #[arg(long, value_name = "TEXT")]
        headline: String,
        /// Output path for CANARY.txt.
        #[arg(long, value_name = "PATH", default_value = "CANARY.txt")]
        output: std::path::PathBuf,
    },
}

/// Subcommands for `nexus-shell-daemon invite ...`.
#[derive(Debug, Subcommand)]
pub enum InviteCommand {
    /// Create a new project invitation.
    Create,
    /// List existing invitations.
    List {
        #[arg(long, default_value = "50")]
        limit: usize,
    },
    /// Revoke an invitation by ID.
    Revoke {
        /// Invite ID (format: inv-{node8}-{ts}-{seq}).
        id: String,
    },
}

/// Subcommands for `nexus-shell-daemon quarantine ...`.
#[derive(Debug, Subcommand)]
pub enum QuarantineCommand {
    /// List pending quarantine entries.
    List,
    /// Flush (release) a quarantine entry by row ID.
    Flush {
        /// Row ID of the entry to flush.
        row_id: i64,
    },
    /// Drop (discard) a quarantine entry by row ID.
    Drop {
        /// Row ID of the entry to drop.
        row_id: i64,
    },
}

/// Subcommands for `nexus-shell-daemon capability ...`.
#[derive(Debug, Subcommand)]
pub enum CapabilityCommand {
    /// List all capabilities and their status.
    List,
    /// Enable a capability.
    Enable {
        /// Capability name.
        name: String,
    },
    /// Disable a capability.
    Disable {
        /// Capability name.
        name: String,
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
            Command::Start {
                headless,
                cors_origins,
                web_root,
            } => {
                assert!(!headless);
                assert!(cors_origins.is_empty());
                assert!(web_root.is_none());
            }
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn parses_start_with_headless() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "start", "--headless"]).unwrap();
        match cli.command {
            Command::Start {
                headless,
                cors_origins,
                ..
            } => {
                assert!(headless);
                assert!(cors_origins.is_empty());
            }
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn parses_start_with_cors_origins() {
        let cli = Cli::try_parse_from([
            "nexus-shell-daemon",
            "start",
            "--cors-origin",
            "http://192.168.1.10:8080",
            "--cors-origin",
            "https://example.com",
        ])
        .unwrap();
        match cli.command {
            Command::Start { cors_origins, .. } => {
                assert_eq!(cors_origins.len(), 2);
                assert_eq!(cors_origins[0], "http://192.168.1.10:8080");
                assert_eq!(cors_origins[1], "https://example.com");
            }
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

    #[test]
    fn parses_init() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "init"]).unwrap();
        assert!(matches!(cli.command, Command::Init));
    }

    #[test]
    fn parses_invite_create() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "invite", "create"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Invite(InviteCommand::Create)
        ));
    }

    #[test]
    fn parses_invite_list() {
        let cli =
            Cli::try_parse_from(["nexus-shell-daemon", "invite", "list", "--limit", "10"]).unwrap();
        match cli.command {
            Command::Invite(InviteCommand::List { limit }) => assert_eq!(limit, 10),
            other => panic!("expected Invite::List, got {other:?}"),
        }
    }

    #[test]
    fn parses_invite_revoke() {
        let cli =
            Cli::try_parse_from(["nexus-shell-daemon", "invite", "revoke", "inv-abc-123-0001"])
                .unwrap();
        match cli.command {
            Command::Invite(InviteCommand::Revoke { id }) => {
                assert_eq!(id, "inv-abc-123-0001")
            }
            other => panic!("expected Invite::Revoke, got {other:?}"),
        }
    }

    #[test]
    fn parses_quarantine_list() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "quarantine", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Quarantine(QuarantineCommand::List)
        ));
    }

    #[test]
    fn parses_quarantine_flush() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "quarantine", "flush", "42"]).unwrap();
        match cli.command {
            Command::Quarantine(QuarantineCommand::Flush { row_id }) => {
                assert_eq!(row_id, 42)
            }
            other => panic!("expected Quarantine::Flush, got {other:?}"),
        }
    }

    #[test]
    fn parses_capability_enable() {
        let cli = Cli::try_parse_from([
            "nexus-shell-daemon",
            "capability",
            "enable",
            "compute_request",
        ])
        .unwrap();
        match cli.command {
            Command::Capability(CapabilityCommand::Enable { name }) => {
                assert_eq!(name, "compute_request")
            }
            other => panic!("expected Capability::Enable, got {other:?}"),
        }
    }

    #[test]
    fn parses_capability_list() {
        let cli = Cli::try_parse_from(["nexus-shell-daemon", "capability", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Command::Capability(CapabilityCommand::List)
        ));
    }
}
