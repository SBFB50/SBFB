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
//! Sprint 83 Phase A reserved the seven versioned contract identities as
//! named [`schemars`] types. Each is a frontier whose wire shape is authored
//! in its own dedicated contract phase (the pack format, the workflow graph,
//! the run journal, and so on); at Phase A every body was an empty stub, and
//! the first to be filled was `FactoryProcessProfile.v1` (Phase D, see
//! [`factory_profile`] below) — 6/7 are filled as of Phase I, with
//! `WorkflowPublicProjection.v1` the one remaining stub (publication/RRV
//! phase). What Phase A locked is the IDENTITY — the
//! schemars `title` carries the `.v1` contract name — and the fact that every
//! identity emits a valid JSON Schema draft 2020-12 object. The generated
//! schemas are pinned by the `schema_snapshot_stub` golden test against a
//! committed snapshot so an identity cannot drift silently as its fields land.
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
//! format. A signing domain and a format version are declared when a SIGNED
//! wire contract is filled, whereas a LOCAL contract declares neither —
//! `FactoryProcessProfile.v1` (Phase D) and `ResolvedAccessibilityProfile.v1`
//! (Phase B) are filled and carry no domain and no format version.
//!
//! `SBFBPromptAgentPack.v1` (Phase F) is the first SIGNED identity to be
//! filled, so it is the first to declare both: [`pack::PACK_FORMAT_VERSION`]
//! here, and `DOMAIN_WORKFLOW_PACK_V1` in `nexus_core_rs::canonical` — this
//! crate cannot import that constant (its charter forbids Tokio, network and
//! database, and `nexus-core-rs` pulls iroh), so [`pack::PACK_SIGNING_DOMAIN`]
//! mirrors the bytes under a disk-reading test that fails on divergence. Phase F
//! took the frozen `DOMAIN_*_V1` census from 25 to 29 (the pack family filled,
//! plus three RESERVED tags for the remaining D2 roles), refreshed in the same
//! commit across `scripts/check-frontier-contracts.sh` (a frozen NAME LIST
//! since F) and `docs/rust/PATTERNS.md` §P70 (cf. `CLAUDE.md §Pre-launch
//! protocol policy`).
//!
//! ## Sprint 83 Phase D — the Factory/process contract
//!
//! [`factory_profile`] fills the first of the seven reserved identities,
//! `FactoryProcessProfile.v1`: the machine-readable extract of this repo's own
//! sprint process (roles, modes, gates, hooks), each marked `documented` or
//! `enforced`, bound to the BLAKE3 digests of its prose sources and fail-closed
//! on drift (`PROFILE_STALE`). The struct is defined there and re-exported from
//! [`contracts`], so it stays entry 4/7 of
//! [`contracts::all_contract_schemas`] (`len == 7`) while its filled schema
//! refreshes `contracts.schema.json`. It carries its OWN instance golden
//! (`factory_profile.golden.json`) and, like the seven identities, declares no
//! `DOMAIN_*_V1` — it is a LOCAL entry golden, never signed or gossiped.
//!
//! ## Sprint 83 Phase B — the host→app accessibility projection
//!
//! [`accessibility_profile`] adds `ResolvedAccessibilityProfile.v1`, the
//! needs-free profile the host projects to a sandboxed SBFB app. It is a
//! standalone frontier with its OWN golden snapshot and is deliberately kept
//! OUT of [`contracts::all_contract_schemas`] (it is not one of the seven
//! signed pack/graph/run identities): keeping the two registries separate
//! preserves the `len == 7` invariant of that registry. Like the seven
//! identities it declares no `DOMAIN_*_V1` — it is a LOCAL host→app
//! projection, never signed or gossiped.
//!
//! ## Sprint 83 Phase F — the signed portable pack
//!
//! [`pack`] fills `SBFBPromptAgentPack.v1`: the root of the signed, portable
//! prompt+agent pack, its `PACK.lock` digest pinning, its six separate semantic
//! digests, in-toto-style provenance, and the bounded verification path
//! (size bound, then structural parse, then Ed25519, then the rich typed parse).
//! Like `FactoryProcessProfile.v1` it is defined in its own module and
//! re-exported from [`contracts`], so it stays entry 1/7 of
//! [`contracts::all_contract_schemas`] (`len == 7`) while its filled schema
//! refreshes `contracts.schema.json`.

//! ## Sprint 83 Phase G — the workflow language
//!
//! [`graph`] and [`agent_definition`] fill `SBFBWorkflowGraph.v1` and
//! `SBFBAgentDefinition.v1`: the data-flow plan (frozen 18-kind node roster,
//! internally-tagged, fail-closed on unknown kinds and fields) and the bounded
//! agent island (required multi-dimensional budgets, digest-pinned referents,
//! required context filters). Both are re-exported from [`contracts`], staying
//! entries 2/7 and 3/7 of the `len == 7` registry. Neither is signed on its
//! own — they are pinned BY DIGEST inside the pack — so neither declares a
//! `DOMAIN_*_V1` nor a format version. Four golden workflows (factory, ideas,
//! conception, babel) are committed next to this crate's sources with their
//! pinned artefact stores, and the node-profile freeze decision lives in
//! `.planning/active/sprint83_node_profile_freeze.md`.
//!
//! ## Sprint 83 Phase I — the run journal envelope
//!
//! [`run_event`] fills `RunEvent.v1`: the versioned-once envelope of the
//! append-only run journal (D3) and the PURE upcasting door
//! ([`run_event::upcast_payload`]) that interprets older payloads at read
//! time — with its fail-closed downgrade guard (a row written by a newer
//! engine is refused, never mis-read). The types lived in
//! `sbfb-workflow-engine::event` during the Phase E spike; the spike
//! returned GO, so the envelope moved here following the cadence of the
//! other filled identities and the engine now depends on this crate and
//! re-exports them. Like the graph and compile contracts it is a LOCAL,
//! never-signed artefact: no `DOMAIN_*_V1` (`DOMAIN_RUN_EVENT_V1` stays a
//! RESERVED tag with zero producers — it names a FUTURE exported form, not
//! this journal), and the identity stays entry 5/7 of
//! [`contracts::all_contract_schemas`] (`len == 7`).
//!
//! ## Sprint 83 Phase H — the authoritative compiler
//!
//! [`compile`] fills `CompiledPlan.v1`: the single semantic authority
//! (research §12.1 — the iframe may lint, never decide) that turns a resolved
//! pack into an immutable, deterministic plan. It enumerates the twenty
//! static refusal classes (thirteen from research §10.5, six consigned for
//! Phase H by the node-profile freeze §8, plus the nominal-by-digest port
//! compatibility of freeze §7), closes the capability set, expands
//! occurrence identities (`node_instance_id` as a pure function of the
//! structural path, RFC 6901 encoding), inherits budgets dimension by
//! dimension and pins the whole result under the Phase F digest door
//! (`digest_for`, JCS). Like the graph and agent contracts it is a LOCAL,
//! never-signed artefact in this phase: no `DOMAIN_*_V1`, no format version
//! (`DOMAIN_COMPILED_PLAN_V1` stays a RESERVED tag with zero producers), and
//! the identity stays entry 6/7 of [`contracts::all_contract_schemas`]
//! (`len == 7`).

pub mod accessibility_profile;
pub mod agent_definition;
pub mod compile;
pub mod contracts;
pub mod factory_profile;
#[cfg(test)]
pub(crate) mod golden_store;
pub mod graph;
pub mod pack;
pub mod run_event;
