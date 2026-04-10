//! # nexus-worker
//!
//! SBFB single-binary GPU contributor for the P2P compute network.
//! Runs on each volunteer's machine, connects to the projects they
//! have opted into via their allowlist, and serves LLM inference
//! tasks via a local Ollama backend.
//!
//! ## Sprint 1 scope
//!
//! Stub only — prints a greeting, boots an anonymous iroh node via
//! `nexus_core_rs::create_node()`, shows its node id and exits.
//!
//! The real CLI (`register`, `start`, `browse`, `join`, `projects`,
//! `stats`, `config`) lands in Sprint 3. The state machine, Ollama
//! client, GPU detection and TUI dashboard all come in Sprint 3 too.
//! This main function exists now purely so `cargo build --workspace`
//! has a valid binary target to compile.

use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "nexus-worker starting (Sprint 1 stub)"
    );

    let node = nexus_core_rs::create_node().await?;
    println!("nexus-worker v{}", env!("CARGO_PKG_VERSION"));
    println!("node id: {}", node.node_id());
    println!();
    println!("This is a Sprint 1 stub. The full worker CLI lands in Sprint 3:");
    println!("  nexus-worker register --name <name>");
    println!("  nexus-worker start");
    println!("  nexus-worker browse");
    println!("  nexus-worker join <nx://...>");
    println!("  nexus-worker projects list|enable|disable");

    node.shutdown().await?;
    Ok(())
}
