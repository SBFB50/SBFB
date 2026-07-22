// SPDX-License-Identifier: AGPL-3.0-or-later
//! Re-exports of the `RunEvent.v1` envelope, authored in
//! `sbfb_workflow_core::run_event` since Sprint 83 Phase I.
//!
//! The envelope lived HERE during the Phase E spike, deliberately not
//! filling the reserved core identity "while the GO/NO-GO question is still
//! open". The spike returned GO, that rationale expired, and the types moved
//! to the pure core following the cadence of the other filled identities —
//! this module keeps the paths the engine's callers already use
//! (`sbfb_workflow_engine::event::RunEvent`, …) stable.
//!
//! The store applies [`sbfb_workflow_core::run_event::upcast_payload`] at
//! interpretation time in `load_events`, with the fail-closed downgrade
//! guard (`RunStore` refuses a row written by a newer engine). See the core
//! module for the full envelope and upcasting documentation.

pub use sbfb_workflow_core::run_event::{
    RUN_EVENT_FORMAT_VERSION, RUN_EVENT_UPCASTERS, RunEvent, RunEventKind, RunEventUpcaster,
    TerminalReason, UpcastError, upcast_payload,
};
