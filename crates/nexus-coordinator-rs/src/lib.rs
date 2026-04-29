// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB coordinator business logic in Rust.
//!
//! Sprint 35 Phase A — foundation crate. This library contains the
//! coordinator's core services (task dispatch, result validation,
//! kudos ledger) that were previously in the Python coordinator
//! (`packages/nexus-coordinator/`). The daemon binary
//! (`nexus-shell-daemon`) calls into this crate from its axum
//! HTTP server.

pub mod canary_input;
pub mod canary_registry;
pub mod db;
pub mod dispatcher;
pub mod error;
pub mod fairness;
pub mod guardrails;
pub mod honeypot;
pub mod kudos_ledger;
pub mod output_filter;
pub mod pii_redactor;
pub mod pow_counter;
pub mod redundancy;
pub mod rerun;
pub mod types;
pub mod validator;
pub mod watermark_detector;
