// SPDX-License-Identifier: AGPL-3.0-or-later
//! Worker engine: state machine + the async loop that drives it.
//!
//! `state.rs` holds the [`WorkerState`] enum, the [`WorkerEvent`]
//! enum, and a small [`StateMachine`] wrapper that enforces the
//! legal transition graph. The async engine loop sits on top,
//! consuming events from the iroh Node / Ollama client / GPU
//! monitor and emitting state changes through a broadcast channel
//! that the TUI and logger subscribe to.
//!
//! Splitting W6 out from W9 keeps the state machine
//! exhaustively testable in isolation — no tokio, no network,
//! no disk — and means the behaviour contract is locked before
//! any async plumbing is written.

pub mod runtime;
// Sprint 77 Phase F2: worker-side shard claim gate (crypto-before-I/O
// authorisation + fail-closed VRAM capacity check). Pure logic is CI-tested;
// the GGUF header read is feature-gated on `llm_llama_cpp`.
pub mod shard_claim;
pub mod state;
pub mod state_writer;

pub use runtime::{Engine, EngineBoot};
pub use state::{StateMachine, TransitionError, WorkerEvent, WorkerState};
pub use state_writer::{
    SCHEMA_VERSION, SnapshotInputs, StateWriterError, WorkerStateSnapshot, flush as flush_state,
    serialize_to,
};
