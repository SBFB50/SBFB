// SPDX-License-Identifier: AGPL-3.0-or-later
//! Versioned contract identities for the Workflow Agents chantier.
//!
//! Each type below is a NAMED FRONTIER reserved in Sprint 83 Phase A. Bodies are
//! authored in a dedicated contract phase per identity (pack format, workflow
//! graph, run journal, and so on) — most are still empty stubs; the first to be
//! filled is `FactoryProcessProfile.v1` (Phase D, defined in
//! [`crate::factory_profile`] and re-exported here). What Phase A locks is the
//! IDENTITY — the schemars `title` carries the `.v1` contract name — and that
//! every identity emits a valid JSON Schema draft 2020-12 object. The generated
//! schemas are pinned by the `schema_snapshot_stub` golden test against
//! `contracts.schema.json` next to this module so an identity cannot drift
//! silently as fields are added.
//!
//! Pre-launch protocol: these are NEW contract identities, not a bump of any
//! existing wire format. No `*_FORMAT_VERSION` const and no `DOMAIN_*_V1`
//! const is declared here — a signing domain and format version are added when
//! a SIGNED wire contract is filled (the pack/graph/run identities), whereas a
//! LOCAL contract declares neither: `FactoryProcessProfile.v1` (Phase D) is
//! filled and intentionally carries no domain and no format version. Either way
//! the frozen `DOMAIN_*_V1` family census stays untouched (cf.
//! `CLAUDE.md §Pre-launch protocol policy`).

use schemars::{JsonSchema, schema_for};
use serde::{Deserialize, Serialize};

/// `FactoryProcessProfile.v1` — the machine-readable Factory process contract
/// (D1/D4). Sprint 83 Phase A reserved this identity as an empty stub; Phase D
/// authored its fields in the dedicated [`crate::factory_profile`] module (roles,
/// modes, gates, hooks bound to the BLAKE3 digests of their sources, fail-closed
/// on drift via `PROFILE_STALE`). It is re-exported here so it stays entry 4/7
/// of [`all_contract_schemas`]; filling it refreshed `contracts.schema.json`
/// while keeping the registry `len == 7`.
pub use crate::factory_profile::FactoryProcessProfile;

/// `SBFBPromptAgentPack.v1` — root of the signed, portable prompt+agent pack
/// (D2). Carries the workflow graph, agent definitions, prompts, pinned
/// schemas, requested (never granted) capabilities, evals and provenance;
/// `layout/` stays outside the semantic digest. Frontier stub — fields
/// authored in the pack-format phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBPromptAgentPack.v1")]
pub struct SbfbPromptAgentPack {}

/// `SBFBWorkflowGraph.v1` — the data-flow workflow graph (D4): typed ports,
/// bounded `repeat`, subworkflows, `human.review`, `wait.signal`. The signed
/// graph IS the plan; no LLM extends it. Frontier stub — node profile
/// authored in the workflow-language phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBWorkflowGraph.v1")]
pub struct SbfbWorkflowGraph {}

/// `SBFBAgentDefinition.v1` — one bounded agent island inside a pack (D4):
/// prompt, policies, budgets, declared dependencies. `handoff` (explicit
/// transfer) is distinct from `agent.as_tool`. Frontier stub — fields
/// authored in the workflow-language phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "SBFBAgentDefinition.v1")]
pub struct SbfbAgentDefinition {}

/// `RunEvent.v1` — one append-only run-journal entry (D3), primary key
/// `(run_id, sequence)`, with an idempotency key on effects. The journal is
/// the source of truth for durable replay. Frontier stub — envelope authored
/// in the run-store phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "RunEvent.v1")]
pub struct RunEvent {}

/// `CompiledPlan.v1` — the immutable plan produced by the authoritative Rust
/// compiler (D3/D4): normalised graph, closed capabilities, semantic
/// digests. Frontier stub — fields authored in the compiler phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "CompiledPlan.v1")]
pub struct CompiledPlan {}

/// `WorkflowPublicProjection.v1` — the allow-listed public projection of a
/// workflow consumed by RRV (D1/D4): only whitelisted fields leave the node;
/// RRV neither launches nor mutates a run. Frontier stub — fields authored in
/// the publication/RRV phase.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "WorkflowPublicProjection.v1")]
pub struct WorkflowPublicProjection {}

/// The seven reserved contract identities paired with their live JSON Schema
/// (schemars, draft 2020-12). The key is the canonical `.v1` contract name;
/// the value is the generated schema serialized to a [`serde_json::Value`]
/// (mirrors the `nexus-core-rs` `schemas` snapshot convention, PATTERNS
/// §P75).
pub fn all_contract_schemas() -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "SBFBPromptAgentPack.v1".to_string(),
        serde_json::to_value(schema_for!(SbfbPromptAgentPack)).expect("schema serializes"),
    );
    map.insert(
        "SBFBWorkflowGraph.v1".to_string(),
        serde_json::to_value(schema_for!(SbfbWorkflowGraph)).expect("schema serializes"),
    );
    map.insert(
        "SBFBAgentDefinition.v1".to_string(),
        serde_json::to_value(schema_for!(SbfbAgentDefinition)).expect("schema serializes"),
    );
    map.insert(
        "FactoryProcessProfile.v1".to_string(),
        serde_json::to_value(schema_for!(FactoryProcessProfile)).expect("schema serializes"),
    );
    map.insert(
        "RunEvent.v1".to_string(),
        serde_json::to_value(schema_for!(RunEvent)).expect("schema serializes"),
    );
    map.insert(
        "CompiledPlan.v1".to_string(),
        serde_json::to_value(schema_for!(CompiledPlan)).expect("schema serializes"),
    );
    map.insert(
        "WorkflowPublicProjection.v1".to_string(),
        serde_json::to_value(schema_for!(WorkflowPublicProjection)).expect("schema serializes"),
    );
    serde_json::Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The canonical `.v1` contract names, in registry order.
    const CONTRACT_NAMES: [&str; 7] = [
        "SBFBPromptAgentPack.v1",
        "SBFBWorkflowGraph.v1",
        "SBFBAgentDefinition.v1",
        "FactoryProcessProfile.v1",
        "RunEvent.v1",
        "CompiledPlan.v1",
        "WorkflowPublicProjection.v1",
    ];

    #[test]
    fn every_contract_emits_nonempty_draft_2020_12_object() {
        let all = all_contract_schemas();
        let obj = all.as_object().expect("registry is a JSON object");
        assert_eq!(obj.len(), CONTRACT_NAMES.len(), "seven reserved identities");
        for name in CONTRACT_NAMES {
            let schema = obj
                .get(name)
                .unwrap_or_else(|| panic!("registry must contain {name}"));
            let s = schema
                .as_object()
                .unwrap_or_else(|| panic!("{name} schema must be a JSON object"));
            let draft = s
                .get("$schema")
                .and_then(|v| v.as_str())
                .unwrap_or_else(|| panic!("{name} must carry the draft $schema URL"));
            assert!(
                draft.contains("2020-12"),
                "{name} must be JSON Schema draft 2020-12, got {draft}"
            );
            assert_eq!(
                s.get("title").and_then(|v| v.as_str()),
                Some(name),
                "{name} schema title must equal its contract identity"
            );
        }
    }

    /// Golden snapshot against `contracts.schema.json` next to this module —
    /// the canary against silent identity drift once fields land. Refresh
    /// after an intentional change with
    /// `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core schema_snapshot_stub`.
    ///
    /// Bootstrap semantics (canonical §P75, mirrors `nexus-core-rs`
    /// `schema_snapshot_matches_struct`): a missing/empty snapshot self-writes
    /// then returns green so the golden can be generated on first run; once the
    /// snapshot is committed, every run (CI included) takes the compare path.
    /// The committed `contracts.schema.json` is what pins the seven identities.
    #[test]
    fn schema_snapshot_stub() {
        let live = all_contract_schemas();
        let live_pretty = serde_json::to_string_pretty(&live).unwrap();

        let snapshot_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("contracts.schema.json");

        let needs_write = std::env::var_os("UPDATE_SNAPSHOTS").is_some()
            || !snapshot_path.exists()
            || std::fs::read_to_string(&snapshot_path)
                .map(|s| s.trim().is_empty())
                .unwrap_or(true);

        if needs_write {
            std::fs::write(&snapshot_path, format!("{live_pretty}\n"))
                .expect("write contracts.schema.json snapshot");
            return;
        }

        let snapshot_str =
            std::fs::read_to_string(&snapshot_path).expect("read contracts.schema.json snapshot");
        let snapshot: serde_json::Value =
            serde_json::from_str(&snapshot_str).expect("contracts.schema.json must be valid JSON");

        assert_eq!(
            live, snapshot,
            "contract schema drift detected — run with UPDATE_SNAPSHOTS=1 to refresh"
        );
    }
}
