// SPDX-License-Identifier: AGPL-3.0-or-later
//! Task response wire format — the schema the LLM backend enforces
//! before the worker signs the result and dispatches back to the
//! coordinator.
//!
//! ## Minimal shape (Sprint 20)
//!
//! ```text
//! {
//!   "version": 1,
//!   "domain":  "TASK_RESPONSE_V1",
//!   "content": "<LLM output>",
//!   "reasoning": null,               // optional CoT trace
//!   "tool_calls": []                 // empty at S20 — S22+ sandbox activates
//! }
//! ```
//!
//! Every field is mandatory except `reasoning` + `tool_calls`
//! (defaults to `None` / empty vec). The LLM must emit every
//! required key — a missing `domain` or a wrong `version` is a
//! decode failure caught by the defensive validator before
//! signature.
//!
//! ## Why `domain` + `version` fields on the wire
//!
//! The Ed25519 + BLAKE3 signing layer already prepends a domain
//! tag to the canonical bytes before hashing (cf. `canonical.rs
//! DOMAIN_*`). Duplicating the tag _inside_ the JSON payload looks
//! redundant at first glance but it hardens against :
//!
//! 1. **Confused-deputy attacks** — a rogue coordinator that
//!    accidentally forwards a `TaskResponse` as a `Task` cannot
//!    verify because the domain tag string inside the payload
//!    doesn't match.
//! 2. **Grammar drift** — if a future sprint redefines the
//!    canonical domain tag without regenerating the schema
//!    snapshot, the test `test_schema_snapshot_matches_struct`
//!    fires.
//! 3. **Operator debugging** — parsing a hex-encoded blob from
//!    the wire is easier when the JSON itself says what it is.

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

/// The only valid `version` field value for this schema. Bumped
/// only at the `v1.0` tag (pre-launch protocol policy — cf.
/// `CLAUDE.md §Pre-launch protocol policy`).
pub const TASK_RESPONSE_VERSION: u8 = 1;

/// The `domain` tag that MUST appear verbatim inside every
/// `TaskResponse`. Binds the payload to its schema identity and
/// hardens the Ed25519 canonical bytes domain tag `DOMAIN_RESULT_V1`
/// (see `canonical.rs`) with a defense-in-depth string.
pub const TASK_RESPONSE_DOMAIN_TAG: &str = "TASK_RESPONSE_V1";

/// The structured response emitted by the worker's LLM backend
/// after [`LlmBackend::generate`] completes. Signed by the worker
/// before dispatch back to the coordinator.
///
/// Both backends force their sampler to emit a byte sequence
/// matching the JSON Schema generated from this type. A garbled
/// generation (invalid JSON, missing required field, version
/// mismatch) is caught by the defensive `serde_json::from_str`
/// validator on the text output — the worker refuses to sign a
/// response it could not parse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskResponse {
    /// Protocol version. Must equal [`TASK_RESPONSE_VERSION`] ;
    /// any other value is a decode failure.
    pub version: u8,
    /// Domain tag. Must equal [`TASK_RESPONSE_DOMAIN_TAG`] ; any
    /// other value is a decode failure.
    pub domain: String,
    /// The LLM output text the worker returns to the coordinator.
    pub content: String,
    /// Optional chain-of-thought / structured reasoning trace when
    /// the model was prompted to emit one. `None` (or absent)
    /// when reasoning was not requested.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Tool calls the worker asks the coordinator to dispatch on
    /// its behalf. Empty at Sprint 20 — every worker allowlisted
    /// at Sprint 20 is expected to emit `[]` until the Sprint 22
    /// tool-calling sandbox activates the gate.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

/// A single tool call the worker asks the coordinator to execute.
///
/// At Sprint 20 this structure is purely declarative — the
/// coordinator ignores `tool_calls` until Sprint 22 activates the
/// allow-list + wasmtime sandbox. The field is part of the wire
/// format from day one so the schema does not bump when S22 lands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolCall {
    /// Name of the tool to invoke (e.g. `"http_get"`,
    /// `"fs_read"`). Match against the S22+ allow-list.
    pub name: String,
    /// Free-form JSON arguments the tool will receive. Shape
    /// depends on the tool registered in the allow-list — the
    /// schema intentionally leaves this field as
    /// `serde_json::Value` so tool authors define their own
    /// inner validation.
    pub arguments: serde_json::Value,
}

impl TaskResponse {
    /// Build a `TaskResponse` with the required fields and empty
    /// `reasoning` / `tool_calls`. The version and domain are
    /// always set correctly ; callers cannot produce an invalid
    /// response through this constructor.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            version: TASK_RESPONSE_VERSION,
            domain: TASK_RESPONSE_DOMAIN_TAG.to_string(),
            content: content.into(),
            reasoning: None,
            tool_calls: Vec::new(),
        }
    }

    /// Validate that the version and domain match the schema
    /// identity of this build. Called by the defensive validator
    /// before the worker signs.
    pub fn validate_identity(&self) -> Result<(), &'static str> {
        if self.version != TASK_RESPONSE_VERSION {
            return Err("TaskResponse.version does not match TASK_RESPONSE_VERSION");
        }
        if self.domain != TASK_RESPONSE_DOMAIN_TAG {
            return Err("TaskResponse.domain does not match TASK_RESPONSE_DOMAIN_TAG");
        }
        Ok(())
    }
}

/// JSON Schema (draft 2020-12 via schemars 1.x default settings)
/// derived from the [`TaskResponse`] struct. Returned as a
/// `serde_json::Value` so both LLM backends consume the same
/// object (Ollama via `JsonStructure`, llama.cpp via
/// `llguidance::TopLevelGrammar::from_json_schema`).
///
/// Sprint 72 Phase C (D2): schemars bumped 0.8 → 1.2 (ollama-rs
/// 0.3.4 requires it). `schema_for!` now returns a `schemars::Schema`
/// (was `RootSchema`); `serde_json::to_value` still serializes it
/// cleanly. The emitted draft moves 07 → 2020-12 (`$defs`), hence
/// the regenerated snapshot below.
pub fn task_response_schema() -> serde_json::Value {
    let schema = schema_for!(TaskResponse);
    serde_json::to_value(schema).expect("schemars Schema serializes cleanly to serde_json::Value")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_parses_as_valid_json_object() {
        let schema = task_response_schema();
        assert!(
            schema.is_object(),
            "root schema must be a JSON object, got {schema:?}"
        );
        let obj = schema.as_object().unwrap();
        assert!(
            obj.contains_key("$schema"),
            "schemars adds the draft $schema URL"
        );
    }

    #[test]
    fn schema_includes_required_fields() {
        let schema = task_response_schema();
        let required = schema
            .pointer("/required")
            .and_then(|v| v.as_array())
            .expect("TaskResponse schema must publish a `required` array");
        // version + domain + content are mandatory ; reasoning is
        // serde(default, skip_serializing_if) and tool_calls is
        // serde(default), so both are optional in the schema.
        let names: Vec<&str> = required.iter().filter_map(|v| v.as_str()).collect();
        assert!(names.contains(&"version"), "version must be required");
        assert!(names.contains(&"domain"), "domain must be required");
        assert!(names.contains(&"content"), "content must be required");
    }

    #[test]
    fn new_constructor_produces_valid_identity() {
        let r = TaskResponse::new("hello");
        assert_eq!(r.version, TASK_RESPONSE_VERSION);
        assert_eq!(r.domain, TASK_RESPONSE_DOMAIN_TAG);
        assert_eq!(r.content, "hello");
        assert!(r.reasoning.is_none());
        assert!(r.tool_calls.is_empty());
        assert!(r.validate_identity().is_ok());
    }

    #[test]
    fn validate_identity_rejects_wrong_version() {
        let mut r = TaskResponse::new("x");
        r.version = 2;
        assert!(r.validate_identity().is_err());
    }

    #[test]
    fn validate_identity_rejects_wrong_domain() {
        let mut r = TaskResponse::new("x");
        r.domain = "SOMETHING_ELSE_V1".to_string();
        assert!(r.validate_identity().is_err());
    }

    #[test]
    fn serde_roundtrip_preserves_all_fields() {
        let mut r = TaskResponse::new("the answer");
        r.reasoning = Some("I thought about it carefully.".to_string());
        r.tool_calls.push(ToolCall {
            name: "fs_read".to_string(),
            arguments: serde_json::json!({"path": "/etc/hostname"}),
        });

        let wire = serde_json::to_string(&r).unwrap();
        let back: TaskResponse = serde_json::from_str(&wire).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn deny_unknown_fields_rejects_extra_keys() {
        // `deny_unknown_fields` on TaskResponse prevents a rogue
        // field from surviving deserialization — important so a
        // malicious LLM cannot stash extra data inside the payload
        // while still producing a schema-valid top-level shape.
        let poisoned = serde_json::json!({
            "version": TASK_RESPONSE_VERSION,
            "domain": TASK_RESPONSE_DOMAIN_TAG,
            "content": "hi",
            "tool_calls": [],
            "exfil": "should not appear",
        });
        let res: Result<TaskResponse, _> = serde_json::from_value(poisoned);
        assert!(res.is_err(), "unknown fields must be rejected");
    }

    #[test]
    fn default_tool_calls_deserialize_as_empty_vec() {
        // `reasoning` skipped (serde default) + `tool_calls`
        // skipped (serde default) must both accept absence.
        let minimal = serde_json::json!({
            "version": TASK_RESPONSE_VERSION,
            "domain": TASK_RESPONSE_DOMAIN_TAG,
            "content": "hi",
        });
        let r: TaskResponse = serde_json::from_value(minimal).unwrap();
        assert!(r.reasoning.is_none());
        assert!(r.tool_calls.is_empty());
    }

    /// Snapshot test against `task_response.schema.json` next to
    /// this module. The snapshot is the canary against silent
    /// struct drift : if a field changes shape without regenerating
    /// the snapshot, this test fails with a diff.
    ///
    /// ## Refreshing the snapshot
    ///
    /// After an intentional schema change, run once with the
    /// refresh env var :
    ///
    /// ```text
    /// UPDATE_SNAPSHOTS=1 cargo test -p nexus-core-rs \
    ///     schemas::task_response::tests::schema_snapshot_matches_struct
    /// ```
    ///
    /// The test writes the pretty-printed JSON in place and
    /// succeeds. Commit the updated JSON alongside the struct
    /// change.
    #[test]
    fn schema_snapshot_matches_struct() {
        let live = task_response_schema();
        let live_pretty = serde_json::to_string_pretty(&live).unwrap();

        let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("schemas")
            .join("task_response.schema.json");

        let needs_write = std::env::var_os("UPDATE_SNAPSHOTS").is_some()
            || !snapshot_path.exists()
            || std::fs::read_to_string(&snapshot_path)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);

        if needs_write {
            std::fs::write(&snapshot_path, format!("{live_pretty}\n"))
                .expect("write task_response.schema.json snapshot");
            return;
        }

        let snapshot_str = std::fs::read_to_string(&snapshot_path)
            .expect("read task_response.schema.json snapshot");
        let snapshot: serde_json::Value = serde_json::from_str(&snapshot_str)
            .expect("task_response.schema.json must be valid JSON");

        assert_eq!(
            live, snapshot,
            "TaskResponse schema drift detected — run with UPDATE_SNAPSHOTS=1 to refresh"
        );
    }
}
