// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB coordinator business logic in Rust.
//!
//! Sprint 35 Phase A — foundation crate. This library contains the
//! coordinator's core services (task dispatch, result validation,
//! kudos ledger) that were previously in the Python coordinator
//! (`packages/nexus-coordinator/`). The daemon binary
//! (`nexus-shell-daemon`) calls into this crate from its axum
//! HTTP server.

pub mod db;
pub mod error;
pub mod types;
