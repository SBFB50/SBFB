// SPDX-License-Identifier: AGPL-3.0-or-later
//! # nexus-shell-daemon-core
//!
//! Headless engine for the SBFB shell daemon — the long-lived
//! P2P process that backs the React shell's Browse / Curators
//! pages without ever becoming a second point of user-facing
//! HTTP.
//!
//! This crate contains every moving part of the shell-daemon
//! *except* the CLI + HTTP surface: the config loader, the
//! singleton registry (`running.json` with pid liveness check),
//! the shared filesystem layout, and the schema-versioned
//! `DaemonStateSnapshot` that the binary serializes back to the
//! shell via the coordinator proxy.
//!
//! ## Why a separate crate
//!
//! Sprint 7 Phase A follows the same **headless-first** split
//! the Sprint 3 `nexus-worker-core` / `nexus-worker` pair picked:
//! the engine must run and be fully testable without any axum
//! server or clap parser. The `nexus-shell-daemon` binary crate
//! depends on this one and adds the CLI dispatch, the HTTP
//! router, and the runtime wiring. Future alternative frontends
//! (a GUI, a systemd unit, a named-pipe controller) can depend
//! on this same crate and skip the `nexus-shell-daemon` binary
//! entirely.
//!
//! ## Module layout
//!
//! The module set is deliberately a strict subset of what the
//! worker core ships — the daemon is a simpler beast: no GPU
//! monitor, no Ollama client, no allowlist database, no invite
//! flow. Phase C will add a `curator_runtime` module for the
//! gossip subscribe pipeline, but that wave is explicitly out
//! of scope here.
//!
//! - `paths` — shared nexus-grid filesystem layout + the
//!   `NEXUS_GRID_ROOT` env override for hermetic tests.
//! - `config` — [`ShellDaemonConfig`] loaded from
//!   `~/.nexus-grid/shell-daemon/config.toml` (Phase A keeps the
//!   on-disk layer minimal; most fields have sensible defaults).
//! - `registry` — `running.json` singleton writer + reader +
//!   pid-based staleness check (D2 enforcement, cf. the Sprint 7
//!   kickoff §4).
//! - `state` — [`DaemonStateSnapshot`] schema v1. Frozen now so
//!   Phase C/D can extend the shape additively without bumping
//!   the schema version.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod auth;
pub mod blob_serve;
pub mod browse;
pub mod config;
pub mod iroh_runtime;
pub mod paths;
pub mod publish;
pub mod registry;
pub mod state;

/// Version of the `nexus-shell-daemon-core` crate, taken from
/// `CARGO_PKG_VERSION` at compile time. The `nexus-shell-daemon`
/// binary re-exports this via `/health` and `/info` so the shell
/// can detect a drift between its compiled-in schema and the
/// daemon's compiled-in schema.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reverse-DNS qualifier used by the `directories` crate to pick
/// a platform-specific application directory. Kept identical to
/// the `nexus-worker-core` constants so a single user has exactly
/// one `~/.nexus-grid/` tree on every platform.
pub const PROJECT_QUALIFIER: &str = "dev";

/// Organization name used by the `directories` crate. Must match
/// the worker-core value so the shared `nexus-grid` root resolves
/// to the same platform path regardless of which crate resolves
/// it first.
pub const PROJECT_ORGANIZATION: &str = "FlowUP";

/// Application name used by the `directories` crate. Matches the
/// worker-core value for the same reason as the qualifier and
/// organization constants above.
pub const PROJECT_APPLICATION: &str = "nexus-grid";
