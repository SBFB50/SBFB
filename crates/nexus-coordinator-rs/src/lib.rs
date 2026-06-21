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
pub mod capability_store;
pub mod contributor_registry;
pub mod db;
pub mod dispatcher;
pub mod error;
pub mod fairness;
pub mod feed_materializer;
pub mod forge;
pub mod guardrails;
pub mod honeypot;
pub mod invite;
pub mod kudos_ledger;
pub mod output_filter;
pub mod pii_redactor;
pub mod placement;
pub mod pow_counter;
pub mod proof_card;
pub mod provenance;
pub mod public_feed;
pub mod quarantine_queue;
pub mod rerun;
pub mod routing;
pub mod search;
pub mod types;
pub mod upload_queue;
pub mod validator;
pub mod watermark_detector;
