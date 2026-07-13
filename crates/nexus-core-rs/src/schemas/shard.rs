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
/// Scope, honestly bounded: the aggregate fields here (`session_id`,
/// `member_count`, `rtt_frontier_ms`) are everything a control-plane can
/// truthfully expose from a mounted session WITHOUT leaking the private
/// group's composition. A richer runtime status (pipeline lifecycle,
/// attained verification level) stays additive / 0-bump; Sprint 81 Phase I
/// landed the live registry + the readiness-barrier RTT, the deeper
/// per-shard telemetry is the live-benchmark concern (Phase J).
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
    /// Worst frontier RTT measured at the session's readiness barrier,
    /// milliseconds (`null` until sampled). Sprint 81 Phase I: the live
    /// registry populates it; the b3_shard network gate reads it. An
    /// aggregate transport measurement — still no identity leaves the
    /// whitelist. Additive on the inner view (the front Zod envelope is
    /// `.strict()`, the view is not), 0-bump.
    #[schemars(required)]
    pub rtt_frontier_ms: Option<u64>,
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

/// Read-only result of one driven shard-session generation, as returned by
/// `GET /api/daemon/shard-session/{id}/result` (Sprint 81 Phase I — the
/// route the `b3_shard_pipeline.sh` live harness polls).
///
/// Same privacy whitelist as [`ShardSessionView`]: measurements and the
/// driver's signed proof, NEVER a `worker_pubkey` / `initiator` identity.
/// `run_proof` is the lowercase hex of the driver's [`crate::shard_plan::
/// RunProofEntry`] Ed25519 signature (the full signed entry stays in the
/// node-local registry); every measurement field is `null` until a drive
/// completes, and `failure` carries the clean `BLOCK{diagnosis}`-style
/// diagnostic of a failed drive (anti-false-green: the harness never
/// PASSes on an empty proof).
// Loopback-API frontier (Sprint 81 Phase I, doctrine §7): consumed by a
// distinct runtime (`scripts/acceptance/b3_shard_pipeline.sh`). Not a signed
// wire type, so it carries no `// FRONTIER:` domain/version tag (that opt-in
// registry is for `DOMAIN_*_V1` families); its machine contract is the
// generated schema snapshot below (`shard_session_result_response.schema.json`,
// drift-gated) + the S81 docs-contract closure index.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ShardSessionResultView {
    /// The session id.
    pub session_id: String,
    /// Final generated text (`null` until a drive completes).
    #[schemars(required)]
    pub result_text: Option<String>,
    /// Time to first output frame from the last stage, whole seconds.
    #[schemars(required)]
    pub ttft_s: Option<u64>,
    /// Measured decode rate, integer tokens per second. Transport-only
    /// echo sessions floor this at 1 (a liveness signal, not a rate
    /// claim); a REAL decode (Sprint 81 Phase J, Option B) reports the
    /// UNFLOORED measured rate, so a sub-1 tok/s pipeline surfaces as 0
    /// and the harness' >=1 gate BLOCKs honestly.
    #[schemars(required)]
    pub toks_per_s: Option<u64>,
    /// Output tokens of the last drive (`1` for a transport-only echo
    /// pass; the generated token count for a REAL decode — the harness'
    /// anti-false-green tell, Sprint 81 Phase J).
    #[schemars(required)]
    pub tokens: Option<u64>,
    /// Lowercase hex of the driver's signed RunProof Ed25519 signature.
    #[schemars(required)]
    pub run_proof: Option<String>,
    /// Worst frontier RTT measured at the readiness barrier, milliseconds.
    #[schemars(required)]
    pub rtt_frontier_ms: Option<u64>,
    // --- Sprint 82 Phase B: standard benchmark metrics ------------------
    // Additive on this inner NON-SIGNED view (mirror of `rtt_frontier_ms`
    // S81-I: « Additive ... 0-bump »), so the `b3_shard` harness and the
    // committed `sprint82_t2_benchmarks.json` artefact read the
    // vLLM/MLPerf-vocabulary metrics WITHOUT a wire bump. These are the
    // HONEST fine metrics measured host-side in `drive_decode_loop`; the
    // signed `RunProof::RunMetrics` is untouched (adding to it would bump
    // `RUN_PROOF_FORMAT_VERSION`). `null` until a drive completes.
    /// Precise **TTFT** (time-to-first-token), milliseconds. The coarse
    /// whole-second [`Self::ttft_s`] loses the sub-second resolution a
    /// standard TTFT metric needs; this is the SAME measured value at
    /// millisecond precision. `null` until a drive completes.
    #[schemars(required)]
    pub ttft_ms: Option<u64>,
    /// **TPOT** (time-per-output-token), milliseconds: the mean inter-token
    /// gap AFTER the first token — the vLLM/MLPerf decode-latency metric.
    /// `0` when fewer than two tokens were generated (no inter-token gap
    /// exists); `null` until a drive completes.
    #[schemars(required)]
    pub tpot_ms: Option<u64>,
    /// **ITL p50** — median inter-token latency (ms), the REAL per-token
    /// distribution measured host-side (nearest-rank), not the coarse mean
    /// the signed `RunMetrics::p95_token_latency_ms` carries.
    #[schemars(required)]
    pub itl_p50_ms: Option<u64>,
    /// **ITL p95** — 95th-percentile inter-token latency (ms), nearest-rank:
    /// the honest tail-latency the p95 field-name on the signed proof
    /// promises but never was (that field is a mean).
    #[schemars(required)]
    pub itl_p95_ms: Option<u64>,
    /// Decode throughput in **milli-tokens per second** (2_300 = 2.3 tok/s):
    /// the sub-integer resolution the whole-integer [`Self::toks_per_s`]
    /// floor hides — the ~2 tok/s HUB baseline needs it to compare future
    /// optimisations (F2 KV-reuse, quant, topology).
    #[schemars(required)]
    pub decode_milli_tokens_per_sec: Option<u64>,
    /// Churn drops observed (SI-9 deadline re-routes + explicit
    /// `drop-shard` cuts).
    pub worker_drop_count: u32,
    /// Diagnostic of a failed drive (`null` while healthy) — the readable
    /// cause the harness surfaces instead of a silent hang. The text may
    /// carry 8-byte TRUNCATED worker-key prefixes (the repo's log
    /// convention, non-invertible to the 256-bit key); the property
    /// whitelist above is about full identities, which never appear.
    #[schemars(required)]
    pub failure: Option<String>,
}

/// Envelope for `GET /api/daemon/shard-session/{id}/result` — same
/// `{found, result}` shape as [`ShardSessionStatusResponse`] (the
/// S73-E / S75-D envelope lesson: a miss is a successful parse, never a
/// 404 transport error).
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ShardSessionResultResponse {
    /// Whether a mounted session matched the requested id.
    pub found: bool,
    /// The result view when `found`, else `null` (key always present).
    #[schemars(required)]
    pub result: Option<ShardSessionResultView>,
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

/// JSON Schema of the privacy-whitelisted [`ShardSessionResultView`].
pub fn shard_session_result_view_schema() -> serde_json::Value {
    to_value(schema_for!(ShardSessionResultView))
}

/// JSON Schema of the `GET /api/daemon/shard-session/{id}/result` envelope
/// [`ShardSessionResultResponse`].
pub fn shard_session_result_response_schema() -> serde_json::Value {
    to_value(schema_for!(ShardSessionResultResponse))
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
        (
            "shard_session_result_view.schema.json",
            shard_session_result_view_schema(),
        ),
        (
            "shard_session_result_response.schema.json",
            shard_session_result_response_schema(),
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

        // Same envelope + always-serialized contract for the S81 Phase I
        // result route: every measurement key is present (null until a
        // drive completes), so the harness' flat scrape never mistakes an
        // omitted key for an unmeasured value.
        let result_envelope = required(&shard_session_result_response_schema());
        for key in ["found", "result"] {
            assert!(
                result_envelope.contains(&key.to_string()),
                "result envelope must require `{key}`"
            );
        }
        let result_view = required(&shard_session_result_view_schema());
        for key in [
            "session_id",
            "result_text",
            "ttft_s",
            "toks_per_s",
            "run_proof",
            "rtt_frontier_ms",
            // Sprint 82 Phase B benchmark metrics (additive, always serialized).
            "ttft_ms",
            "tpot_ms",
            "itl_p50_ms",
            "itl_p95_ms",
            "decode_milli_tokens_per_sec",
            "worker_drop_count",
            "failure",
        ] {
            assert!(
                result_view.contains(&key.to_string()),
                "result view must require `{key}` (always serialized, null until measured)"
            );
        }
    }

    /// SECURITY (SI-3/SI-4): the result view publishes measurements + the
    /// driver's proof hex ONLY — never an identity field (same whitelist
    /// proof as the status view: the property set is the contract).
    #[test]
    fn shard_session_result_view_schema_is_whitelisted() {
        let schema = shard_session_result_view_schema();
        let props = schema
            .pointer("/properties")
            .and_then(|v| v.as_object())
            .expect("ShardSessionResultView schema must publish `properties`");
        let mut keys: Vec<&str> = props.keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            vec![
                "decode_milli_tokens_per_sec",
                "failure",
                "itl_p50_ms",
                "itl_p95_ms",
                "result_text",
                "rtt_frontier_ms",
                "run_proof",
                "session_id",
                "tokens",
                "toks_per_s",
                "tpot_ms",
                "ttft_ms",
                "ttft_s",
                "worker_drop_count",
            ],
            "the result view must expose only measurements + proof hex \
             (Sprint 82 Phase B added the fine benchmark metrics, all aggregate)"
        );
        for forbidden in ["worker_pubkey", "initiator", "members", "participants"] {
            assert!(
                !keys.contains(&forbidden),
                "the result view must never publish a `{forbidden}` property (SI-3/SI-4)"
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
            vec!["member_count", "rtt_frontier_ms", "session_id"],
            "the observed view must expose only the aggregate whitelist \
             (rtt_frontier_ms added S81 Phase I: a transport measurement, not an identity)"
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

        // S81 Phase K extension: the application-level payloads carried
        // INSIDE the opaque frames (step payloads J, stage attestation K)
        // must be cited by the spec, with their version guards and kind
        // discriminants bound to the real consts — exactly the drift class
        // that occurred when Phase J shipped the types while the doc layer
        // stayed untouched. Presence-only assertions (never a spec parser).
        for type_name in [
            "ShardStepRequest",
            "ShardStepReply",
            "ShardStageAttestationRequest",
            "ShardStageAttestation",
        ] {
            assert!(
                spec.contains(type_name),
                "spec must cite the in-frame payload type `{type_name}`"
            );
        }
        for (guard_name, guard_value) in [
            ("SHARD_STEP_PAYLOAD_V", crate::SHARD_STEP_PAYLOAD_V),
            ("SHARD_ATTEST_PAYLOAD_V", crate::SHARD_ATTEST_PAYLOAD_V),
        ] {
            assert!(guard_value > 0, "guard {guard_name} must be non-zero");
            assert!(
                spec.contains(guard_name),
                "spec must cite the payload version guard `{guard_name}`"
            );
        }
        for kind in [
            crate::SHARD_ATTEST_REQUEST_KIND,
            crate::SHARD_ATTEST_REPLY_KIND,
        ] {
            assert!(
                spec.contains(kind),
                "spec must cite the attestation kind discriminant `{kind}`"
            );
        }
    }
}
