//! # nexus-worker-core
//!
//! Headless worker engine for the SBFB P2P compute network.
//!
//! This crate contains every moving part of a worker *except* the
//! user interface: the config loader, the project allowlist, the
//! GPU monitor trait, the Ollama client, the state machine, and
//! the engine loop that glues all of that to [`nexus_core_rs`].
//!
//! ## Why a separate crate
//!
//! Sprint 3 of the SBFB plan explicitly picks a **headless-first**
//! architecture: the engine must run and be fully testable without
//! any terminal, TUI, or interactive prompt. The `nexus-worker`
//! binary crate depends on this one and adds a `clap` CLI plus an
//! optional `ratatui` dashboard. Alternative frontends (a systemd
//! service, a daemon controlled over a Unix socket, a GUI) can
//! depend on this same crate and skip the `nexus-worker` binary
//! entirely.
//!
//! ## Module layout
//!
//! The modules below are added wave by wave through Sprint 3. The
//! layout is locked now so every wave has an obvious home. Empty
//! modules are documented but contain no code until their wave
//! lands.
//!
//! - `config` — (W3) `WorkerConfig` loaded from
//!   `~/.nexus-grid/worker.toml` via `directories` + `config`.
//! - `gpu` — (W4) `GpuMonitor` trait with `NvmlBackend` and
//!   `NoopBackend` implementations.
//! - `ollama` — (W5) async Ollama client with healthcheck and
//!   retry.
//! - `engine::state` — (W6) `WorkerState` state machine + legal
//!   transitions.
//! - `allowlist` — (W7) SQLite-backed project allowlist.
//! - `invite` — (W8) invite-token parser / encoder.
//! - `engine::loop` — (W9) the main engine loop that drives the
//!   state machine and talks to `nexus_core_rs`.

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod config;
pub mod gpu;

/// Version of the `nexus-worker-core` crate, taken from
/// `CARGO_PKG_VERSION` at compile time. The `nexus-worker` binary
/// re-exports this so `nexus-worker --version` matches the engine
/// version exactly.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Reverse-DNS qualifier used by the `directories` crate to pick
/// a platform-specific application directory (e.g.
/// `com.Example.App` on macOS). See
/// [`config::WorkerPaths::resolve`] for the resolved paths.
pub const PROJECT_QUALIFIER: &str = "dev";

/// Organization name used by the `directories` crate. Becomes
/// part of the config/data directory path on every platform.
pub const PROJECT_ORGANIZATION: &str = "FlowUP";

/// Application name used by the `directories` crate. This is the
/// last path component on Linux (`~/.config/nexus-grid/`) and the
/// leaf directory on Windows / macOS alongside the organization.
pub const PROJECT_APPLICATION: &str = "nexus-grid";
