//! # nexus-core-rs
//!
//! Core Rust library for the SBFB P2P compute network. Wraps the iroh
//! stack (net, docs, gossip, blobs, pkarr) behind a small, stable API
//! that is also exposed to Python via `crates/nexus-core-py` (PyO3).
//!
//! This crate is intentionally thin: it does not implement any
//! business logic (task dispatching, kudos, apps, ...). Those live in
//! the Python coordinator (`packages/nexus-coordinator`) and in the
//! `nexus-worker` binary. The only responsibility here is to give
//! callers a stable handle to a running iroh node and a minimal
//! surface for the primitives the coordinator needs.
//!
//! ## Sprint 1 scope
//!
//! `create_node()` only. Sprint 2 will add `Doc`, `Gossip`, `Blobs`,
//! `Discovery` and `Verifier` wrappers as separate submodules.
//!
//! ## Example (Rust)
//!
//! ```no_run
//! # async fn example() -> nexus_core_rs::Result<()> {
//! let node = nexus_core_rs::create_node().await?;
//! tracing::info!("node id: {}", node.node_id());
//! node.shutdown().await?;
//! # Ok(())
//! # }
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]
#![warn(missing_docs)]

pub mod crypto;
pub mod error;
pub mod node;
pub mod task;
pub mod verification;

pub use crypto::{blake3_hash, verify, Blake3Chain, KeyPair};
pub use error::{NexusError, Result};
pub use node::{create_node, Node};
pub use task::{canonical_bytes, Claim, ResultEntry, ResultPayload, Task, TaskEntry};
pub use verification::{spot_check_rate, CheckStatus, LayerResult, VerificationReport, Verifier};
