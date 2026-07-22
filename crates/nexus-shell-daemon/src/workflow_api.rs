// SPDX-License-Identifier: AGPL-3.0-or-later
//! Workflow run store read handlers (`/api/daemon/workflow/*`) —
//! Sprint 83 Phase I1, the loopback surface over the Phase I durable
//! run store (`sbfb-workflow-engine`).
//!
//! The routes stay registered in `crate::http::build_router` inside
//! `authed_routes` (loopback bearer + Host + Origin; peer creds at the
//! listener layer) — never in `public_routes`, and never as a merged
//! sub-router (the D7 inventory pins the route count via
//! `grep -c ".route(" http.rs`).
//!
//! READS ONLY at this phase. The mutating family (restore / GC /
//! lease-force) is tier-classified T1 in the D7 inventory
//! (`docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §3) with its
//! implementation dated to the phases that grow its real consumer
//! (J reducer / K effects): T1 CONFIRM_PROMPT is design-only today, so
//! a mutating route would ship at effective T0; `restore_from_backup`
//! requires a CLOSED store (incompatible with the boot-held handle);
//! GC needs the Phase J clock source; lease-force has no legitimate
//! loopback caller while the runtime acquires its lease in-process.
//!
//! The daemon holds ONE `RunStore` handle (opened at boot, mirror of
//! `coordinator.db`) and NEVER acquires the writer lease here — these
//! reads are unfenced and safe under WAL; lease acquisition belongs to
//! the Phase J runtime write path.
//!
//! Privacy shape (mirror of the shard-session projection): the
//! responses are daemon-local DTOs carrying AGGREGATE run progress
//! only — never a raw `RunEvent` list (`WorkflowPublicProjection.v1`
//! is a later phase), never a snapshot state, never an effect
//! receipt / request hash.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use serde::Serialize;
use tracing::debug;

use sbfb_workflow_engine::{RunStatus, RunStore, StoreError};

use crate::http::DaemonHttpState;

/// File name of the workflow run store under the daemon data root
/// (`opts.paths.root`, sibling of `coordinator.db`) — NOT under the
/// `~/.sbfb` security root. Single definition shared by the boot path
/// (`runtime.rs`) and the test-state builder (`test_support.rs`).
pub(crate) const RUN_STORE_FILE: &str = "workflow_runs.sqlite3";

/// Server-side `run_id` bound (T-RUN-ID-PII, THREAT_MODEL §19): the
/// store accepts free TEXT, so the ROUTE is the validation choke
/// point. 128 is far above any engine-minted id and far below the
/// store's own 512-byte identity cap.
const MAX_RUN_ID_LEN: usize = 128;

/// Route-level `run_id` guard: 1..=128 chars drawn from
/// `[A-Za-z0-9._-]`. Rejecting everything else keeps the id safe to
/// echo into logs and JSON. A `run_id` is NEVER used to form a
/// filesystem path — it only ever travels as a `params!` SQL binding —
/// so this guard is an identity-hygiene bound, not a path-safety one.
fn valid_run_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_RUN_ID_LEN
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// Aggregate per-run progress view. Scalars only — the allowlist IS
/// the type shape (shard-session SI-3/SI-4 precedent).
#[derive(Debug, Serialize)]
pub(crate) struct WorkflowRunView {
    run_id: String,
    /// `"running" | "completed" | "terminal"` — manual mapping of the
    /// engine enum (its own `as_str` is private).
    status: &'static str,
    /// Set only when `status == "terminal"`.
    terminal_reason: Option<String>,
    event_count: u64,
    /// Last node instance whose effect COMPLETED (an internal node id,
    /// not a payload).
    frontier_node: Option<String>,
    spec_digest: Option<String>,
}

/// Envelope for `GET /api/daemon/workflow/run/{run_id}`. A miss is the
/// deterministic `{found:false, run:null}` (200, not 404 — seed_count /
/// shard-session precedent: read-only routes answer 200 with honest
/// defaults so the caller's parse succeeds).
#[derive(Debug, Serialize)]
pub(crate) struct WorkflowRunStatusResponse {
    found: bool,
    run: Option<WorkflowRunView>,
}

/// Raw lease-row view. `expires_at_ms` is the takeover HEURISTIC the
/// store records — never a liveness verdict; judging "expired" needs a
/// clock and belongs to the caller (restitution-pure route).
#[derive(Debug, Serialize)]
pub(crate) struct WorkflowLeaseView {
    owner: String,
    epoch: i64,
    expires_at_ms: i64,
}

/// Envelope for `GET /api/daemon/workflow/lease`. `found:false` covers
/// both "no lease row yet" and "released" (the engine filters the
/// empty-owner released row itself).
#[derive(Debug, Serialize)]
pub(crate) struct WorkflowLeaseResponse {
    found: bool,
    lease: Option<WorkflowLeaseView>,
}

fn run_status_label(status: RunStatus) -> &'static str {
    match status {
        RunStatus::Running => "running",
        RunStatus::Completed => "completed",
        RunStatus::Terminal => "terminal",
    }
}

/// Pure projection for the per-run route — pinned by unit tests
/// without a network boot. Reads stop at the existence check on a
/// miss; on a hit every field comes from a public read-only getter.
fn workflow_run_response(
    store: &RunStore,
    run_id: &str,
) -> Result<WorkflowRunStatusResponse, StoreError> {
    let Some(status) = store.run_status(run_id)? else {
        return Ok(WorkflowRunStatusResponse {
            found: false,
            run: None,
        });
    };
    Ok(WorkflowRunStatusResponse {
        found: true,
        run: Some(WorkflowRunView {
            run_id: run_id.to_string(),
            status: run_status_label(status),
            terminal_reason: store.terminal_reason(run_id)?,
            event_count: store.event_count(run_id)? as u64,
            frontier_node: store.frontier_node(run_id)?,
            spec_digest: store.spec_digest(run_id)?,
        }),
    })
}

/// Pure projection for the lease route.
fn workflow_lease_response(store: &RunStore) -> Result<WorkflowLeaseResponse, StoreError> {
    Ok(match store.lease_state()? {
        Some(lease) => WorkflowLeaseResponse {
            found: true,
            lease: Some(WorkflowLeaseView {
                owner: lease.owner,
                epoch: lease.epoch,
                expires_at_ms: lease.expires_at_ms,
            }),
        },
        None => WorkflowLeaseResponse {
            found: false,
            lease: None,
        },
    })
}

fn store_error_response(err: StoreError) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

/// `GET /api/daemon/workflow/run/{run_id}` — aggregate read-only
/// status of one workflow run (Sprint 83 Phase I1).
///
/// Not duress-gated: local reads stay live under a decoy boot (the
/// duress posture no-ops OUTBOUND/signing operations only; gating a
/// read would itself be a distinguishing signal), and this phase
/// exposes no mutation. D7 tier T0 (read-only family).
pub(crate) async fn workflow_run_status(
    State(state): State<Arc<DaemonHttpState>>,
    Path(run_id): Path<String>,
) -> Response {
    debug!("GET /api/daemon/workflow/run");
    if !valid_run_id(&run_id) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "invalid run_id" })),
        )
            .into_response();
    }
    let store = state.run_store.lock().expect("run store mutex poisoned");
    match workflow_run_response(&store, &run_id) {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => store_error_response(e),
    }
}

/// `GET /api/daemon/workflow/lease` — raw single-writer lease row of
/// the run store (Sprint 83 Phase I1). Read-only observation; forcing
/// a lease is the deferred T1 mutating family (dated J/K in the D7
/// inventory).
pub(crate) async fn workflow_lease(State(state): State<Arc<DaemonHttpState>>) -> Response {
    debug!("GET /api/daemon/workflow/lease");
    let store = state.run_store.lock().expect("run store mutex poisoned");
    match workflow_lease_response(&store) {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(e) => store_error_response(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{build_test_router, mk_state, mk_state_with_mode};
    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use tower::ServiceExt;

    /// Tempdir-backed store for the pure-projection tests (no
    /// in-memory constructor exists — WAL is file-only). The guard
    /// must stay alive for the scope of the test.
    fn test_store() -> (tempfile::TempDir, RunStore) {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = RunStore::open(&dir.path().join(RUN_STORE_FILE)).expect("open run store");
        (dir, store)
    }

    #[test]
    fn valid_run_id_accepts_bounded_slug() {
        assert!(valid_run_id("run-up"));
        assert!(valid_run_id("Run_1.retry"));
        assert!(valid_run_id(&"a".repeat(MAX_RUN_ID_LEN)));
    }

    #[test]
    fn valid_run_id_rejects_out_of_charset() {
        assert!(!valid_run_id(""));
        assert!(!valid_run_id(&"a".repeat(MAX_RUN_ID_LEN + 1)));
        assert!(!valid_run_id("run/id"));
        assert!(!valid_run_id("run id"));
        assert!(!valid_run_id("run\u{7}id"));
        assert!(!valid_run_id("run\u{e9}id"));
    }

    #[test]
    fn run_response_miss_is_deterministic_empty_envelope() {
        let (_dir, store) = test_store();
        let resp = workflow_run_response(&store, "absent").expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "found": false, "run": null }),
            "miss must be the exact {{found:false, run:null}} envelope"
        );
    }

    #[test]
    fn run_response_projects_running_then_terminal() {
        let (_dir, store) = test_store();
        store.ensure_run("run-a", "digest-a").expect("ensure");
        let resp = workflow_run_response(&store, "run-a").expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "found": true,
                "run": {
                    "run_id": "run-a",
                    "status": "running",
                    "terminal_reason": null,
                    "event_count": 1,
                    "frontier_node": null,
                    "spec_digest": "digest-a"
                }
            }),
            "running projection must expose exactly the aggregate allowlist"
        );

        store
            .record_terminal(
                "run-a",
                None,
                sbfb_workflow_engine::TerminalReason::OutcomeUnknown,
                None,
                None,
                None,
            )
            .expect("terminal");
        let resp = workflow_run_response(&store, "run-a").expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["run"]["status"], "terminal");
        assert_eq!(json["run"]["terminal_reason"], "outcome_unknown");
        assert_eq!(json["run"]["event_count"], 2);
    }

    /// Third status value — reachable daemon-side through the PUBLIC
    /// `record_run_completed` (review r1: the "completed" arm was the
    /// only unexercised label; mutation M-5 pins it).
    #[test]
    fn run_response_projects_completed() {
        let (_dir, store) = test_store();
        store.ensure_run("run-c", "digest-c").expect("ensure");
        store
            .record_run_completed("run-c", None)
            .expect("complete run");
        let resp = workflow_run_response(&store, "run-c").expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["found"], true);
        assert_eq!(json["run"]["status"], "completed");
        assert_eq!(
            json["run"]["terminal_reason"],
            serde_json::Value::Null,
            "a completed run has no terminal reason"
        );
    }

    /// The projection never carries event payloads, snapshot state or
    /// effect receipts — the allowlist is the serialized key SET
    /// itself (drift here = a review flag, mirror of SI-3/SI-4).
    /// Compared as sorted sets: serde_json's Map ordering differs
    /// between `-p` (BTreeMap) and workspace (preserve_order) builds.
    #[test]
    fn run_response_key_set_is_the_allowlist() {
        let (_dir, store) = test_store();
        store.ensure_run("run-b", "digest-b").expect("ensure");
        let resp = workflow_run_response(&store, "run-b").expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        let mut top: Vec<&str> = json
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        top.sort_unstable();
        assert_eq!(top, ["found", "run"]);
        let mut run: Vec<&str> = json["run"]
            .as_object()
            .unwrap()
            .keys()
            .map(|k| k.as_str())
            .collect();
        run.sort_unstable();
        assert_eq!(
            run,
            [
                "event_count",
                "frontier_node",
                "run_id",
                "spec_digest",
                "status",
                "terminal_reason"
            ]
        );
    }

    #[test]
    fn lease_response_empty_then_populated() {
        let (_dir, mut store) = test_store();
        let resp = workflow_lease_response(&store).expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json, serde_json::json!({ "found": false, "lease": null }));

        let epoch = store
            .acquire_lease("owner-a", 1_000, 30_000)
            .expect("lease");
        let resp = workflow_lease_response(&store).expect("projection");
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "found": true,
                "lease": { "owner": "owner-a", "epoch": epoch, "expires_at_ms": 31_000 }
            }),
            "lease view must be the raw row: owner + fencing epoch + expiry heuristic"
        );
    }

    async fn get_json(app: &axum::Router, uri: &str) -> (u16, serde_json::Value) {
        let resp = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let bytes = to_bytes(resp.into_body(), 1 << 20).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn workflow_routes_http_contract() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));

        let (status, json) = get_json(&app, "/api/daemon/workflow/run/absent-run").await;
        assert_eq!(status, 200);
        assert_eq!(json, serde_json::json!({ "found": false, "run": null }));

        let (status, json) = get_json(&app, "/api/daemon/workflow/run/bad!id").await;
        assert_eq!(status, 400);
        assert_eq!(json, serde_json::json!({ "error": "invalid run_id" }));

        let (status, json) = get_json(&app, "/api/daemon/workflow/lease").await;
        assert_eq!(status, 200);
        assert_eq!(json, serde_json::json!({ "found": false, "lease": null }));

        // Seed a run through the SAME handle the router state holds,
        // then observe it over HTTP — proves the route reads the
        // boot-held store, not a per-request open.
        state
            .run_store
            .lock()
            .unwrap()
            .ensure_run("run-http", "digest-http")
            .expect("seed run");
        let (status, json) = get_json(&app, "/api/daemon/workflow/run/run-http").await;
        assert_eq!(status, 200);
        assert_eq!(json["found"], true);
        assert_eq!(json["run"]["status"], "running");
        assert_eq!(json["run"]["event_count"], 1);
    }

    /// Duress boot: both reads answer IDENTICALLY to a normal boot.
    /// Local reads are not duress-gated — the noop posture covers
    /// outbound/signing operations only, and a gated read would itself
    /// distinguish a decoy boot. Discriminance (review r1): the test
    /// also SEEDS a run under the duress boot and asserts it is served
    /// — a mutation that gates reads to `{found:false}` under duress
    /// goes red here, not just the empty-store envelopes.
    #[tokio::test]
    async fn workflow_routes_identical_under_duress() {
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;
        let app = build_test_router(Arc::clone(&state));

        let (status, json) = get_json(&app, "/api/daemon/workflow/run/absent-run").await;
        assert_eq!(status, 200);
        assert_eq!(json, serde_json::json!({ "found": false, "run": null }));

        let (status, json) = get_json(&app, "/api/daemon/workflow/lease").await;
        assert_eq!(status, 200);
        assert_eq!(json, serde_json::json!({ "found": false, "lease": null }));

        state
            .run_store
            .lock()
            .unwrap()
            .ensure_run("run-duress", "digest-d")
            .expect("seed run under duress");
        let (status, json) = get_json(&app, "/api/daemon/workflow/run/run-duress").await;
        assert_eq!(status, 200);
        assert_eq!(
            json["found"], true,
            "local reads serve real data under duress"
        );
        assert_eq!(json["run"]["status"], "running");
    }
}
