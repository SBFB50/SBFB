// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bridge between the [`nexus_core_rs::TaskResponse`] schema and
//! backend-specific grammar types.
//!
//! The trait [`super::LlmBackend`] carries the schema as an
//! opaque `serde_json::Value` so the interface stays generic. At
//! Sprint 20 the only schema identity in use is `TaskResponse`,
//! so each backend resolves the value through a helper defined
//! here :
//!
//! - [`task_response_schema_value`] — the canonical schema
//!   `serde_json::Value` both backends compare against.
//! - [`ollama_json_structure`] — wraps
//!   `ollama_rs::generation::parameters::JsonStructure::new::<T>()`.
//!   `JsonStructure` has a private `schema: RootSchema` field, so
//!   building one from a raw `serde_json::Value` would require an
//!   upstream `from_value` constructor that does not exist. At
//!   Sprint 20 we know the type statically — this helper hard-codes
//!   the `TaskResponse` type so the Ollama backend never needs to
//!   dynamically reflect a `serde_json::Value` back into a
//!   `RootSchema`. A future sprint that adds per-task schemas will
//!   either contribute a `from_value` upstream or carry a richer
//!   type-based bridge here.
//!
//! When a future sprint introduces more than one schema identity,
//! the resolver should switch on a marker inside the
//! `serde_json::Value` (e.g. the `$id` JSON-Schema field) rather
//! than growing the trait signature.

use nexus_core_rs::{TaskResponse, task_response_schema};
use ollama_rs::generation::parameters::JsonStructure;

/// Canonical `serde_json::Value` for the Sprint 20 `TaskResponse`
/// schema. Equivalent to `nexus_core_rs::task_response_schema()`
/// — re-exported here as a single worker-side entry point so the
/// backend factory / CLI can reference it without reaching into
/// the core crate's nested module path.
pub fn task_response_schema_value() -> serde_json::Value {
    task_response_schema()
}

/// Build the Ollama-side [`JsonStructure`] for `TaskResponse`.
///
/// Sprint 20 ships only one schema identity so this helper is
/// parameter-less. When a future sprint adds per-task schemas
/// the caller will pass a marker and dispatch here.
pub fn ollama_json_structure() -> JsonStructure {
    JsonStructure::new::<TaskResponse>()
}

/// Check whether an arbitrary `schema` value looks like the
/// `TaskResponse` schema. At Sprint 20 this is a simple equality
/// against [`task_response_schema_value`]. A future sprint that
/// adds per-task schemas will switch on the JSON `$id` marker.
///
/// Returns `true` when the backend should run the
/// `TaskResponse`-specific enforcement path.
pub fn schema_is_task_response(schema: &serde_json::Value) -> bool {
    *schema == task_response_schema_value()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_value_matches_core_export() {
        let from_worker = task_response_schema_value();
        let from_core = nexus_core_rs::task_response_schema();
        assert_eq!(from_worker, from_core);
    }

    #[test]
    fn ollama_json_structure_builds_without_panic() {
        // We cannot inspect `JsonStructure` internals (private
        // field), but constructing one exercises the full
        // `schemars::schema_for!(TaskResponse)` + `RootSchema`
        // pipeline. If schemars or schema_for change shape in a
        // way that breaks the private constructor, this test
        // fires at compile time or with a panic.
        let _ = ollama_json_structure();
    }

    #[test]
    fn schema_is_task_response_accepts_canonical() {
        let schema = task_response_schema_value();
        assert!(schema_is_task_response(&schema));
    }

    #[test]
    fn schema_is_task_response_rejects_unrelated() {
        let other = serde_json::json!({"type": "integer"});
        assert!(!schema_is_task_response(&other));
    }
}
