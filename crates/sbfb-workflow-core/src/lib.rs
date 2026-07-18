// SPDX-License-Identifier: AGPL-3.0-or-later
//! # sbfb-workflow-core — pure contract core for the Workflow Agents chantier
//!
//! This crate is the PURE half of the two-crate durable-execution core
//! (kickoff D3): schema identities, canonicalisation + digests, static
//! validation, and the immutable compiled-plan compilation belong here. The
//! host-neutral engine (reducer, scheduler, event store, effects, inbox /
//! outbox) is a separate `sbfb-workflow-engine` crate. `sbfb-workflow-core`
//! links no Tokio, no network stack and no database — it stays deterministic
//! and testable in isolation.
//!
//! ## Phase A status — reserved contract frontiers
//!
//! Sprint 83 Phase A reserves the seven versioned contract identities as
//! named [`schemars`] types with empty bodies. Each type is a frontier whose
//! wire shape is authored in its own dedicated contract phase (the pack
//! format, the workflow graph, the run journal, and so on). What Phase A
//! locks is the IDENTITY — the schemars `title` carries the `.v1` contract
//! name — and the fact that every identity emits a valid JSON Schema draft
//! 2020-12 object. The generated schemas are pinned by the
//! `schema_snapshot_stub` golden test against a committed snapshot so an
//! identity cannot drift silently once its fields are added.
//!
//! The seven reserved identities (kickoff D2/D3/D4):
//! - [`contracts::SbfbPromptAgentPack`] — `SBFBPromptAgentPack.v1`, root of
//!   the signed portable prompt+agent pack.
//! - [`contracts::SbfbWorkflowGraph`] — `SBFBWorkflowGraph.v1`, the
//!   data-flow workflow graph.
//! - [`contracts::SbfbAgentDefinition`] — `SBFBAgentDefinition.v1`, one
//!   bounded agent island.
//! - [`contracts::FactoryProcessProfile`] — `FactoryProcessProfile.v1`, the
//!   machine-readable Factory process contract.
//! - [`contracts::RunEvent`] — `RunEvent.v1`, one append-only run-journal
//!   entry.
//! - [`contracts::CompiledPlan`] — `CompiledPlan.v1`, the immutable compiled
//!   plan.
//! - [`contracts::WorkflowPublicProjection`] — `WorkflowPublicProjection.v1`,
//!   the allow-listed public projection consumed by RRV.
//!
//! ## Pre-launch protocol
//!
//! These are NEW contract identities, not a bump of any existing wire
//! format. No `*_FORMAT_VERSION` and no `DOMAIN_*_V1` constant is declared
//! here: the signing domains and format versions are added when each
//! contract is filled, which keeps the frozen `DOMAIN_*_V1` family census
//! untouched (cf. `CLAUDE.md §Pre-launch protocol policy`).

pub mod contracts;
