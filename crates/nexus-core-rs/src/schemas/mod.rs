// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 20 Phase D — JSON Schema source-of-truth for LLM task
//! response payloads.
//!
//! The worker's LLM backend forces its output to match a JSON
//! Schema so a garbled generation cannot poison the signature
//! chain : an invalid `TaskResponse` is caught at decode time
//! (defensive validator) before the worker signs. Both backends
//! read this schema at runtime :
//!
//! - `OllamaBackend` passes the schema to Ollama v0.5+ via
//!   `GenerationRequest::format(FormatType::StructuredJson(...))`
//!   so the daemon's internal llama.cpp enforces it at sample
//!   time (native GBNF).
//! - `LlamaCppBackend` feeds the same `serde_json::Value` to
//!   `llguidance::TopLevelGrammar::from_json_schema(...)` and
//!   masks disallowed tokens inside the custom sampler for a
//!   ~50 µs overhead per token (documented in the design doc
//!   `.planning/research/S20_phase_D_structured_output_design.md`).
//!
//! ## Why a Rust struct is the source of truth
//!
//! `schemars::schema_for!(TaskResponse)` generates the JSON Schema
//! from the typed Rust struct. If a future sprint adds a field to
//! `TaskResponse`, the schema bumps automatically and both backends
//! pick up the change without any JSON hand-edit — zero drift risk
//! between wire schema, worker dispatch, and coordinator parse.
//!
//! A `task_response.schema.json` file lives next to this module as
//! a **snapshot** used by `test_schema_snapshot_matches_struct`:
//! if the struct evolves and the snapshot is not regenerated, the
//! test fails loudly with a diff. The snapshot is _not_ the source
//! of truth — it's a canary against silent drift.
//!
//! ## `*_VERSION = 1` pre-launch protocol policy
//!
//! [`TaskResponse::version`] stays pinned at `1` until the `v1.0`
//! tag (cf. `CLAUDE.md §Pre-launch protocol policy`). A decoder
//! rejects any other value — there is no tolerant multi-version
//! decoder because the project has no live protocol speakers yet.

pub mod task_response;

pub use task_response::{
    TASK_RESPONSE_DOMAIN_TAG, TASK_RESPONSE_VERSION, TaskResponse, ToolCall, task_response_schema,
};
