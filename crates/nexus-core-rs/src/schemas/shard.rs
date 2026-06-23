// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 77 Phase L — machine-readable JSON Schemas for the sharded-inference
//! wire primitives.
//!
//! The signed shard wire types ([`crate::shard_plan`], [`crate::compute_group`])
//! and the read-only control-plane DTO are the source of truth; this module
//! generates their JSON Schemas (draft 2020-12) via `schemars::schema_for!` so
//! a machine / LLM / agent can ingest the contract without guessing. Each
//! schema has a `*.schema.json` snapshot next to this module, gated by
//! [`tests::shard_schema_snapshot_matches_struct`]: if a struct evolves and the
//! snapshot is not regenerated, the test fails loudly with a diff. The snapshot
//! is _not_ the source of truth — it is a canary against silent struct drift,
//! the same mechanism as [`super::task_response`].
//!
//! ## Why generated, not hand-written
//!
//! `#[derive(JsonSchema)]` is **additive** on the wire types: it is inert to
//! `serde::Serialize`, so the canonical JCS bytes the Ed25519 signature covers
//! do not change, no `*_FORMAT_VERSION` bumps, and there is no fragile
//! hand-maintained type-mirror (PO decision §20.1). The human spec
//! `docs/protocol/SHARD_PROTOCOL_SPEC.md` and the agent layer (Phase N) point
//! at these schemas rather than re-describing the shapes.
//!
//! ## Signed envelopes are intentionally NOT schematised
//!
//! The `*Entry` envelopes ([`crate::shard_plan::ShardedSessionManifestEntry`],
//! [`crate::shard_plan::RunProofEntry`], [`crate::compute_group::ComputeGroupEntry`])
//! carry a `[u8; 64]` Ed25519 `signature` via `serde_big_array`; deriving a
//! schema there would emit a verbose 64-item array and the signature/redundant
//! identity are NOT part of the canonical bytes anyway. We schematise the
//! signed **payloads**, mirroring [`super::task_response`] (the type, not the
//! envelope).
//!
//! ## Refreshing the snapshots
//!
//! After an intentional schema change, run once with the refresh env var:
//!
//! ```text
//! UPDATE_SNAPSHOTS=1 cargo test -p nexus-core-rs schemas::shard::tests
//! ```
//!
//! Each test writes the pretty-printed JSON in place and succeeds. Commit the
//! updated `*.schema.json` alongside the struct change.

use schemars::{JsonSchema, schema_for};
use serde::Serialize;

use crate::compute_group::ComputeGroup;
use crate::shard_plan::{RunMetrics, RunProof, ShardAssignment, ShardPlan, ShardedSessionManifest};

/// Aggregate, privacy-whitelisted view of one shard session, as returned by the
/// daemon control-plane route `GET /api/daemon/shard-session/{id}`.
///
/// Exposes ONLY the session identity and an aggregate `member_count` — NEVER
/// the `worker_pubkey` / `initiator` identities of the private
/// [`ComputeGroup`] (THREAT_MODEL §16 SI-3/SI-4). The `ComputeGroup` is
/// ADMISSION control, not a confidentiality guarantee, and a loopback caller
/// still has no business enumerating who composes someone's pipeline.
///
/// Scope, honestly bounded: the two fields here are everything a control-plane
/// WITHOUT a running data plane can truthfully derive from a stored manifest.
/// A richer runtime status (pipeline lifecycle, attained verification level)
/// requires live telemetry from a running pipeline; it is added — additive,
/// 0-bump — once the live data-plane store lands (deferred, Sprint 78).
/// Shipping it now would be an un-populatable contract, not an honest one.
///
/// Defined in `nexus-core-rs` (not the daemon) so its `schema_for!` can live
/// next to the other shard schemas: the daemon depends on core, so a core
/// schema cannot reference a daemon-private type — the type lives where the
/// schema is generated, and the daemon consumes it (Phase L PLAN-ADAPT,
/// adaptation A).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ShardSessionView {
    /// The session id (only present when a session is actually found).
    pub session_id: String,
    /// Number of workers in the pipeline = `plan.assignments.len()`. An
    /// aggregate count, never the member identities.
    pub member_count: usize,
}

/// Envelope for `GET /api/daemon/shard-session/{id}`.
///
/// ENVELOPE, not a bare optional (S73-E / S75-D lesson): the frontend Zod
/// schema is `.strict()` on `{found, session}` and `session` is ALWAYS
/// serialized (`null` when absent), so an additive field stays possible and an
/// empty result is a successful parse, not a 404 transport error.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ShardSessionStatusResponse {
    /// Whether a live session matched the requested id.
    pub found: bool,
    /// The aggregate view when `found`, else `null` (the key is always
    /// present — the `.strict()` envelope contract). `#[schemars(required)]`
    /// makes the generated schema reflect that contract: `session` is always
    /// present (nullable), NOT an omittable optional — otherwise the schema
    /// would be looser than the wire (Codex Phase L PARTIAL).
    #[schemars(required)]
    pub session: Option<ShardSessionView>,
}

/// JSON Schema (draft 2020-12) of the signed [`ComputeGroup`] allowlist
/// payload (domain `nexus-compute-group-v1`).
pub fn compute_group_schema() -> serde_json::Value {
    to_value(schema_for!(ComputeGroup))
}

/// JSON Schema of one unsigned [`ShardAssignment`] (a worker's contiguous
/// layer block; only meaningful inside a signed manifest).
pub fn shard_assignment_schema() -> serde_json::Value {
    to_value(schema_for!(ShardAssignment))
}

/// JSON Schema of a [`ShardPlan`] (the ordered list of assignments).
pub fn shard_plan_schema() -> serde_json::Value {
    to_value(schema_for!(ShardPlan))
}

/// JSON Schema of the signed [`ShardedSessionManifest`] payload
/// (domain `nexus-shard-plan-v1`).
pub fn sharded_session_manifest_schema() -> serde_json::Value {
    to_value(schema_for!(ShardedSessionManifest))
}

/// JSON Schema of the all-integer [`RunMetrics`].
pub fn run_metrics_schema() -> serde_json::Value {
    to_value(schema_for!(RunMetrics))
}

/// JSON Schema of the signed [`RunProof`] payload (domain
/// `nexus-run-proof-v1`).
pub fn run_proof_schema() -> serde_json::Value {
    to_value(schema_for!(RunProof))
}

/// JSON Schema of the privacy-whitelisted [`ShardSessionView`].
pub fn shard_session_view_schema() -> serde_json::Value {
    to_value(schema_for!(ShardSessionView))
}

/// JSON Schema of the `GET /api/daemon/shard-session/{id}` envelope
/// [`ShardSessionStatusResponse`].
pub fn shard_session_status_response_schema() -> serde_json::Value {
    to_value(schema_for!(ShardSessionStatusResponse))
}

/// `schema_for!` returns a `schemars::Schema`; serialize it to a
/// `serde_json::Value` (mirrors [`super::task_response::task_response_schema`]).
fn to_value(schema: schemars::Schema) -> serde_json::Value {
    serde_json::to_value(schema).expect("schemars Schema serializes cleanly to serde_json::Value")
}

/// The `(snapshot filename, live schema)` pairs covered by the drift test —
/// one generated JSON Schema per documented wire type.
#[cfg(test)]
fn schema_snapshots() -> Vec<(&'static str, serde_json::Value)> {
    vec![
        ("compute_group.schema.json", compute_group_schema()),
        ("shard_assignment.schema.json", shard_assignment_schema()),
        ("shard_plan.schema.json", shard_plan_schema()),
        (
            "sharded_session_manifest.schema.json",
            sharded_session_manifest_schema(),
        ),
        ("run_metrics.schema.json", run_metrics_schema()),
        ("run_proof.schema.json", run_proof_schema()),
        (
            "shard_session_view.schema.json",
            shard_session_view_schema(),
        ),
        (
            "shard_session_status_response.schema.json",
            shard_session_status_response_schema(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every generated schema is a JSON object carrying the draft `$schema`
    /// URL schemars stamps on the root.
    #[test]
    fn schema_parses_as_valid_json_object() {
        for (name, schema) in schema_snapshots() {
            assert!(schema.is_object(), "{name}: root schema must be an object");
            assert!(
                schema.as_object().unwrap().contains_key("$schema"),
                "{name}: schemars stamps the draft $schema URL"
            );
        }
    }

    /// Required-field spot checks on representative types: the signed payloads
    /// publish their mandatory keys, so a consumer can reject a malformed
    /// payload structurally.
    #[test]
    fn schemas_publish_required_fields() {
        let required = |schema: &serde_json::Value| -> Vec<String> {
            schema
                .pointer("/required")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default()
        };

        let manifest = sharded_session_manifest_schema();
        let m = required(&manifest);
        for key in [
            "version",
            "initiator",
            "session_id",
            "group_id",
            "revision",
            "plan",
        ] {
            assert!(
                m.contains(&key.to_string()),
                "manifest schema must require `{key}`"
            );
        }

        let proof = run_proof_schema();
        let p = required(&proof);
        for key in [
            "version",
            "worker_pubkey",
            "session_id",
            "metrics",
            "participants",
        ] {
            assert!(
                p.contains(&key.to_string()),
                "run proof schema must require `{key}`"
            );
        }

        let group = compute_group_schema();
        let g = required(&group);
        for key in ["version", "group_id", "initiator", "revision", "members"] {
            assert!(
                g.contains(&key.to_string()),
                "compute group schema must require `{key}`"
            );
        }

        // `#[serde(default)]` fields stay OPTIONAL: they must be ABSENT from
        // `required`, so the runtime tolerance survives into the schema (a
        // consumer can omit them). Asserting the exclusion guards against a
        // future struct edit that silently drops the `#[serde(default)]`.
        assert!(
            !p.contains(&"activation_fingerprint".to_string()),
            "RunProof.activation_fingerprint is #[serde(default)] → must be optional"
        );
        let assignment = required(&shard_assignment_schema());
        assert!(
            !assignment.contains(&"fallback_node".to_string()),
            "ShardAssignment.fallback_node is #[serde(default)] → must be optional"
        );

        // Envelope contract (S73-E / S75-D): `session` is ALWAYS serialized
        // (`null` on a miss), so the schema must REQUIRE it — not treat the
        // `Option` as omittable (Codex Phase L PARTIAL fix via
        // `#[schemars(required)]` on the field).
        let envelope = required(&shard_session_status_response_schema());
        for key in ["found", "session"] {
            assert!(
                envelope.contains(&key.to_string()),
                "status envelope must require `{key}`"
            );
        }
    }

    /// SECURITY (SI-3/SI-4): the observed-session schema publishes EXACTLY
    /// `session_id` + `member_count` and NEVER leaks an identity field. The
    /// schema is the documented contract; this locks the whitelist at the
    /// machine-readable layer, not just in the projection function.
    #[test]
    fn shard_session_view_schema_is_whitelisted() {
        let schema = shard_session_view_schema();
        let props = schema
            .pointer("/properties")
            .and_then(|v| v.as_object())
            .expect("ShardSessionView schema must publish `properties`");
        let mut keys: Vec<&str> = props.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec!["member_count", "session_id"],
            "the observed view must expose only the aggregate whitelist"
        );
        // The exact-keys assertion above IS the whitelist proof: an identity
        // field could only leak as a published property. We do NOT scan the
        // raw schema blob for `worker_pubkey`/`initiator` because the type's
        // doc-comment legitimately names them (to state they are excluded), so
        // they appear in `description` strings — the property set is the
        // contract, not the prose.
        for forbidden in ["worker_pubkey", "initiator", "members", "assignments"] {
            assert!(
                !keys.contains(&forbidden),
                "the observed-view schema must never publish a `{forbidden}` property (SI-3/SI-4)"
            );
        }
    }

    /// Drift canary: each committed `*.schema.json` must equal the live
    /// `schema_for!(T)`. Run with `UPDATE_SNAPSHOTS=1` to refresh after an
    /// intentional change (mirror of
    /// `schemas::task_response::tests::schema_snapshot_matches_struct`).
    #[test]
    fn shard_schema_snapshot_matches_struct() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("schemas");
        let refresh = std::env::var_os("UPDATE_SNAPSHOTS").is_some();

        for (name, live) in schema_snapshots() {
            let path = dir.join(name);
            let live_pretty = serde_json::to_string_pretty(&live).unwrap();

            let needs_write = refresh
                || !path.exists()
                || std::fs::read_to_string(&path)
                    .map(|s| s.trim().is_empty())
                    .unwrap_or(true);

            if needs_write {
                std::fs::write(&path, format!("{live_pretty}\n"))
                    .unwrap_or_else(|e| panic!("write {name} snapshot: {e}"));
                continue;
            }

            let snapshot: serde_json::Value =
                serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
                    .unwrap_or_else(|e| panic!("{name} must be valid JSON: {e}"));
            assert_eq!(
                live, snapshot,
                "{name} drift detected — run with UPDATE_SNAPSHOTS=1 to refresh"
            );
        }
    }

    /// doc<->code: every `DOMAIN_*_V1` tag and every named cap the spec cites
    /// must match the real Rust const, so the human spec cannot drift away from
    /// the wire it documents. We reference the consts directly (they exist by
    /// compilation) and assert the spec quotes their current value/name.
    #[test]
    fn spec_consts_exist() {
        let spec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("protocol")
            .join("SHARD_PROTOCOL_SPEC.md");
        let spec = std::fs::read_to_string(&spec_path).unwrap_or_else(|e| {
            panic!("read SHARD_PROTOCOL_SPEC.md ({}): {e}", spec_path.display())
        });

        // The 5 canonical domain tags are cited by their on-wire VALUE.
        for tag in [
            crate::canonical::DOMAIN_COMPUTE_GROUP_V1,
            crate::canonical::DOMAIN_SHARD_PLAN_V1,
            crate::canonical::DOMAIN_RUN_PROOF_V1,
            crate::canonical::DOMAIN_VRF_DRAW_V1,
            crate::canonical::DOMAIN_ACTIVATION_COMMIT_V1,
        ] {
            let value = std::str::from_utf8(tag).unwrap();
            assert!(
                spec.contains(value),
                "spec must cite domain tag value `{value}`"
            );
        }

        // ALPN + payload caps: each cited name is BOUND to its real Rust const.
        // The `let _: usize = cap_value` makes renaming or removing the const a
        // COMPILE error here (not a silent doc drift), and the `contains` keeps
        // the spec citing the canonical identifier by name.
        for (cap_name, cap_value) in [
            ("MAX_SHARD_FRAME_BYTES", crate::MAX_SHARD_FRAME_BYTES),
            ("MAX_SHARD_N_CTX", crate::MAX_SHARD_N_CTX as usize),
            (
                "SHARD_PLAN_MAX_ASSIGNMENTS",
                crate::SHARD_PLAN_MAX_ASSIGNMENTS,
            ),
            (
                "RUN_PROOF_MAX_PARTICIPANTS",
                crate::RUN_PROOF_MAX_PARTICIPANTS,
            ),
            ("SESSION_ID_MAX", crate::SESSION_ID_MAX),
            ("SHARD_HASHES_MAX", crate::SHARD_HASHES_MAX),
            (
                "COMPUTE_GROUP_MAX_MEMBERS",
                crate::COMPUTE_GROUP_MAX_MEMBERS,
            ),
        ] {
            // `cap_value` binds to the real const — renaming/removing it is a
            // compile error here; a cap is also meaningless if zero.
            assert!(cap_value > 0, "cap {cap_name} must be a positive bound");
            assert!(
                spec.contains(cap_name),
                "spec must cite the cap `{cap_name}`"
            );
        }

        // The ALPN identifier itself.
        assert!(
            spec.contains("sbfb/shard/1"),
            "spec must cite the ALPN `sbfb/shard/1`"
        );
    }
}
