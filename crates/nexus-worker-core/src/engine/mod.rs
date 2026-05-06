// SPDX-License-Identifier: AGPL-3.0-or-later
//! Worker engine: state machine + (coming in W9) the loop that
//! drives it.
//!
//! W6 ships `state.rs` only: the [`WorkerState`] enum, the
//! [`WorkerEvent`] enum, and a small [`StateMachine`] wrapper
//! that enforces the legal transition graph. W9 will layer the
//! async engine loop on top, consuming events from the iroh
//! Node / Ollama client / GPU monitor and emitting state
//! changes through a broadcast channel that the W10 TUI and
//! W11 logger subscribe to.
//!
//! Splitting W6 out from W9 keeps the state machine
//! exhaustively testable in isolation — no tokio, no network,
//! no disk — and means the behaviour contract is locked before
//! any async plumbing is written.

pub mod runtime;
pub mod state;
pub mod state_writer;

pub use runtime::{Engine, EngineBoot};
pub use state::{StateMachine, TransitionError, WorkerEvent, WorkerState};
pub use state_writer::{
    SCHEMA_VERSION, SnapshotInputs, StateWriterError, WorkerStateSnapshot, flush as flush_state,
    serialize_to,
};
