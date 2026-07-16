// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shard-session control-plane HTTP handlers
//! (`/api/daemon/shard-session/*`) — extracted verbatim from `http.rs`
//! (Sprint 82 Phase N, first domain split of the monolithic HTTP surface).
//!
//! The routes stay registered in `crate::http::build_router` inside
//! `authed_routes` (loopback bearer + Host + Origin), which references
//! these handlers by full path; route paths, JSON shapes and status
//! codes are unchanged.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use tracing::debug;

use crate::http::DaemonHttpState;

// =====================================================================
// Sprint 77 Phase J — read-only shard-session status (control plane)
// =====================================================================

// Sprint 77 Phase L: `ShardSessionView` + `ShardSessionStatusResponse` moved to
// `nexus-core-rs` (`schemas/shard.rs`) so their `schema_for!` can live next to
// the other shard wire schemas — the daemon depends on core, so a core schema
// cannot reference a daemon-private type. The projection + route below consume
// the re-exported types unchanged; the privacy whitelist (THREAT_MODEL §16
// SI-3/SI-4) is the type shape itself — only aggregate fields are exposed,
// never a `worker_pubkey`/`initiator`.
// Sprint 81 Phase I: the S77 `live_shard_session` STUB is gone — the lookup
// now reads the in-memory `ShardSessionRegistry` populated by the mount
// orchestrator (`crate::shard_session`), whose insert is gated on the
// `DOMAIN_SHARD_PLAN_V1` signature + `is_member` checks the stub mandated.
// Sprint 82 Phase G: the two primitive-only request bodies
// (`ShardGroupMintRequest`, `ShardGenerateRequest`) moved to core for the
// same S77-L reason — the type lives where its `schema_for!` snapshot is
// generated; the handlers below consume them unchanged.
use nexus_core_rs::{
    ShardGenerateRequest, ShardGroupMintRequest, ShardSessionResultResponse,
    ShardSessionResultView, ShardSessionStatusResponse, ShardSessionView,
};

/// Pure projection for `GET /api/daemon/shard-session/{id}` — pinned by a unit
/// test without a network boot. A miss returns the deterministic empty
/// envelope `{found:false, session:null}` (200, not 404 — `seed_count`
/// precedent: a read-only route answers 200 with honest defaults so the
/// parse succeeds).
fn shard_session_response(
    registry: &crate::shard_session::ShardSessionRegistry,
    session_id: &str,
) -> ShardSessionStatusResponse {
    match registry.status_data(session_id) {
        Some(data) => ShardSessionStatusResponse {
            found: true,
            session: Some(ShardSessionView {
                session_id: data.session_id,
                member_count: data.member_count,
                rtt_frontier_ms: data.rtt_frontier_ms,
            }),
        },
        None => ShardSessionStatusResponse {
            found: false,
            session: None,
        },
    }
}

/// `GET /api/daemon/shard-session/{id}` — read-only status of a private
/// compute-group shard session (Sprint 77 Phase J route, Sprint 81 Phase I
/// live registry).
///
/// Control-plane only: an AGGREGATE status (member count + frontier RTT),
/// NEVER the group's member identities (SI-3/SI-4). Loopback-authenticated
/// (lives in `authed_routes`).
pub(crate) async fn shard_session(
    State(state): State<Arc<DaemonHttpState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    debug!("GET /api/daemon/shard-session");
    (
        StatusCode::OK,
        Json(shard_session_response(&state.shard_sessions, &session_id)),
    )
        .into_response()
}

/// `POST /api/daemon/shard-session/group` — Sprint 81 Phase I — mint the
/// signed private compute group for a session (operator flow step 1).
///
/// The daemon signs with its long-lived keypair; the returned
/// `ComputeGroupEntry` JSON is shared VERBATIM with every
/// `shard-session serve` worker so admission and the mount gate check the
/// same signed bytes. Duress-gated: a decoy boot never signs under the
/// fake keypair (mirror of `publish_project` / `seed_request_peer`).
pub(crate) async fn shard_session_group(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<ShardGroupMintRequest>,
) -> Response {
    debug!(group = %req.group_id, "POST /api/daemon/shard-session/group");
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (StatusCode::OK, Json(serde_json::json!({ "minted": false }))).into_response();
    }
    let mut members = Vec::with_capacity(req.members.len());
    for m in &req.members {
        match crate::shard_session::parse_pubkey_hex(m) {
            Ok(pk) => members.push(pk),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({ "error": format!("member '{m}': {e}") })),
                )
                    .into_response();
            }
        }
    }
    match crate::shard_session::mint_compute_group(
        &state.pow_keypair,
        &req.group_id,
        req.revision.unwrap_or(1),
        &members,
    ) {
        Ok(entry) => (
            StatusCode::OK,
            Json(serde_json::json!({ "minted": true, "group": entry })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

/// `POST /api/daemon/shard-session/mount` — Sprint 81 Phase I — mount a
/// shard session: Parallax placement → initiator-signed manifest →
/// readiness barrier (transport-level ACK per shard, NO dispatch frame
/// before every shard answered) → gated registry insert.
///
/// A readiness failure returns 409 with the `BLOCK`-style diagnostic and
/// inserts NOTHING. Duress-gated BEFORE signing the manifest.
pub(crate) async fn shard_session_mount(
    State(state): State<Arc<DaemonHttpState>>,
    Json(req): Json<crate::shard_session::MountSessionRequest>,
) -> Response {
    debug!(session = %req.session_id, "POST /api/daemon/shard-session/mount");
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "mounted": false })),
        )
            .into_response();
    }
    match crate::shard_session::mount_session(
        state.node.endpoint(),
        state.node.memory_lookup(),
        &state.pow_keypair,
        &state.shard_sessions,
        req,
    )
    .await
    {
        Ok(report) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "mounted": true,
                "session_id": report.session_id,
                "member_count": report.member_count,
                "rtt_frontier_ms": report.rtt_frontier_ms,
            })),
        )
            .into_response(),
        Err(diagnostic) => (
            StatusCode::CONFLICT,
            Json(serde_json::json!({ "mounted": false, "error": diagnostic })),
        )
            .into_response(),
    }
}

/// `POST /api/daemon/shard-session/{id}/generate` — Sprint 81 Phase I —
/// drive one generation through the mounted pipeline (HUB walk with the
/// SI-9 per-hop deadline + fallback re-route). Async: returns 202
/// immediately and the drive updates the registry; the harness polls
/// `GET .../result` (its existing contract). Duress-gated BEFORE the
/// drive signs its RunProof.
pub(crate) async fn shard_session_generate(
    State(state): State<Arc<DaemonHttpState>>,
    Path(session_id): Path<String>,
    Json(req): Json<ShardGenerateRequest>,
) -> Response {
    debug!(session = %session_id, "POST /api/daemon/shard-session/generate");
    if crate::noop_identity::gossip_publish_in_duress(state.identity_mode)
        == crate::noop_identity::PublishOutcome::Noop
    {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "accepted": false })),
        )
            .into_response();
    }
    if let Some(body_id) = &req.session_id
        && body_id != &session_id
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "body session_id disagrees with the path session id"
            })),
        )
            .into_response();
    }
    match state.shard_sessions.status_of(&session_id) {
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "accepted": false, "error": "session not mounted" })),
            )
                .into_response();
        }
        // Best-effort 409 precheck (review Cible 2 P2): a concurrent drive
        // sees an honest "already generating" instead of a 202 whose spawned
        // task silently no-ops. The atomic guard in generate_session stays
        // the real backstop for the residual check-then-spawn TOCTOU.
        Some(crate::shard_session::ShardSessionStatus::Generating) => {
            return (
                StatusCode::CONFLICT,
                Json(serde_json::json!({ "accepted": false, "error": "already generating" })),
            )
                .into_response();
        }
        Some(_) => {}
    }
    let node = Arc::clone(&state.node);
    let keypair = Arc::clone(&state.pow_keypair);
    let registry = Arc::clone(&state.shard_sessions);
    let prompt = req.prompt;
    let max_tokens = req
        .max_tokens
        .unwrap_or(crate::shard_session::DEFAULT_MAX_NEW_TOKENS);
    tokio::spawn(async move {
        // Failure is recorded in the registry (`failure` diagnostic) and
        // surfaced by the result route — never a silent drop.
        let _ = crate::shard_session::generate_session(
            node.endpoint(),
            node.memory_lookup(),
            &keypair,
            &registry,
            &session_id,
            &prompt,
            max_tokens,
        )
        .await;
    });
    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({ "accepted": true })),
    )
        .into_response()
}

/// Pure projection for `GET /api/daemon/shard-session/{id}/result` —
/// pinned by a unit test without a network boot. Measurement fields stay
/// `null` until a drive completes (the harness polls on `result_text`);
/// `failure` carries the clean diagnostic of a failed drive.
fn shard_session_result_response(
    registry: &crate::shard_session::ShardSessionRegistry,
    session_id: &str,
) -> ShardSessionResultResponse {
    match registry.result_data(session_id) {
        Some(data) => ShardSessionResultResponse {
            found: true,
            result: Some(ShardSessionResultView {
                session_id: data.session_id,
                result_text: data.result_text,
                ttft_s: data.ttft_s,
                toks_per_s: data.toks_per_s,
                tokens: data.tokens,
                run_proof: data.run_proof,
                rtt_frontier_ms: data.rtt_frontier_ms,
                // Sprint 82 Phase B benchmark metrics (additive, 0-bump).
                ttft_ms: data.ttft_ms,
                tpot_ms: data.tpot_ms,
                itl_p50_ms: data.itl_p50_ms,
                itl_p95_ms: data.itl_p95_ms,
                decode_milli_tokens_per_sec: data.decode_milli_tokens_per_sec,
                worker_drop_count: data.worker_drop_count,
                failure: data.failure,
            }),
        },
        None => ShardSessionResultResponse {
            found: false,
            result: None,
        },
    }
}

/// `GET /api/daemon/shard-session/{id}/result` — Sprint 81 Phase I — the
/// measured outcome of the last driven generation (the poll target of the
/// b3_shard live harness). Same privacy whitelist as the status route.
pub(crate) async fn shard_session_result(
    State(state): State<Arc<DaemonHttpState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    debug!(session = %session_id, "GET /api/daemon/shard-session/result");
    (
        StatusCode::OK,
        Json(shard_session_result_response(
            &state.shard_sessions,
            &session_id,
        )),
    )
        .into_response()
}

/// `POST /api/daemon/shard-session/{id}/drop-shard` — Sprint 81 Phase I —
/// explicit operator churn cut (the b3_shard churn probe): counts the
/// drop, and closes the tail shard's connection when one is still held
/// (pre-drive). Post-drive the teardown already closed every connection,
/// so only the counter moves — the next drive re-dials regardless. A
/// mid-drive drop is handled by the SI-9 fallback path instead.
pub(crate) async fn shard_session_drop_shard(
    State(state): State<Arc<DaemonHttpState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    debug!(session = %session_id, "POST /api/daemon/shard-session/drop-shard");
    match state.shard_sessions.drop_tail_shard(&session_id) {
        Some(dropped) => (
            StatusCode::OK,
            Json(serde_json::json!({ "found": true, "dropped": dropped })),
        )
            .into_response(),
        None => (
            StatusCode::OK,
            Json(serde_json::json!({ "found": false, "dropped": false })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::{build_test_router, mk_state_with_mode};

    #[test]
    fn shard_session_response_pins_empty_envelope() {
        // Sprint 77 Phase J. No live shard-session store exists yet (the
        // `sbfb/shard/1` data plane is not wired to a control-plane registry —
        // a tracked S78 carry), so EVERY id misses and the route answers a deterministic
        // empty envelope. 200 + `{found:false, session:null}`, NEVER a 404: the
        // frontend Zod schema is `.strict()` on the envelope and a miss must be a
        // SUCCESSFUL parse (seed_count precedent), not a transport error. The
        // `session` key is ALWAYS serialized (null), so an additive field stays
        // possible and the "no active session" empty state is unambiguous.
        // Sprint 81 Phase I: the lookup now reads the live registry — an
        // EMPTY registry preserves the pinned S77 contract verbatim.
        let registry = crate::shard_session::ShardSessionRegistry::default();
        let json =
            serde_json::to_value(shard_session_response(&registry, "any-session-id")).unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["found"], false);
        // The `session` key must be PHYSICALLY PRESENT (serialized as null on a
        // miss), not absent: `json["session"].is_null()` alone also passes for a
        // MISSING key under serde_json indexing, so assert key presence first.
        assert!(
            obj.contains_key("session"),
            "the session key is always serialized (.strict() envelope contract)"
        );
        assert!(obj["session"].is_null(), "session is null on a miss");
        assert_eq!(obj.len(), 2, "envelope = exactly {{found, session}}");

        // Same discipline for the Phase I result route: an unmounted id is
        // a successful empty parse, never a transport error.
        let json = serde_json::to_value(shard_session_result_response(&registry, "any-session-id"))
            .unwrap();
        let obj = json.as_object().unwrap();
        assert_eq!(obj["found"], false);
        assert!(obj.contains_key("result"), "result key always serialized");
        assert!(obj["result"].is_null(), "result is null on a miss");
        assert_eq!(obj.len(), 2, "envelope = exactly {{found, result}}");
    }

    #[test]
    fn shard_session_projection_hides_member_identities() {
        // The projection is the PRIVACY seam (THREAT_MODEL §16 SI-3/SI-4): it
        // exposes an AGGREGATE `member_count` but NEVER a worker_pubkey /
        // initiator (the private group's composition). Sprint 81 Phase I: the
        // projection now reads a MOUNTED session from the live registry
        // (inserted through the signature + membership gate), and the view
        // gains the aggregate `rtt_frontier_ms` — still zero identity bytes.
        let head = nexus_core_rs::crypto::KeyPair::generate();
        let worker_a = nexus_core_rs::crypto::KeyPair::generate();
        let worker_b = nexus_core_rs::crypto::KeyPair::generate();
        let group = crate::shard_session::mint_compute_group(
            &head,
            "group-abc",
            1,
            &[worker_a.public_bytes(), worker_b.public_bytes()],
        )
        .expect("group mints");
        let mk = |pk: [u8; 32], start: u32, end: u32| nexus_core_rs::ShardAssignment {
            worker_pubkey: pk,
            layer_start: start,
            layer_end: end,
            role: nexus_core_rs::ShardRole::LayerWorker,
            shard_hashes: vec![[0x22u8; 32]],
            kv_cache_policy: nexus_core_rs::KvCachePolicy::LocalEphemeral,
            fallback_node: None,
            launch_profile_hash: [0x33u8; 32],
        };
        let plan = nexus_core_rs::ShardPlan::new(vec![
            mk(worker_a.public_bytes(), 0, 16),
            mk(worker_b.public_bytes(), 16, 32),
        ]);
        let manifest = nexus_core_rs::ShardedSessionManifest::new(
            head.public_bytes(),
            "session-xyz",
            "group-abc",
            1,
            plan,
            [0x44u8; 32],
            [0x55u8; 32],
            [0x66u8; 32],
        );
        let entry = nexus_core_rs::ShardedSessionManifestEntry::sign(manifest, &head)
            .expect("manifest signs");
        let mut addrs = std::collections::BTreeMap::new();
        for kp in [&worker_a, &worker_b] {
            let id = iroh::EndpointId::from_bytes(&kp.public_bytes()).expect("valid key");
            addrs.insert(kp.public_bytes(), iroh::EndpointAddr::new(id));
        }
        let registry = crate::shard_session::ShardSessionRegistry::default();
        registry
            .insert_gated(crate::shard_session::ShardSessionRecord {
                entry,
                group,
                addrs,
                conns: std::collections::BTreeMap::new(),
                status: crate::shard_session::ShardSessionStatus::Ready,
                outcome: None,
                rtt_frontier_ms: Some(42),
                worker_drop_count: 0,
                failure: None,
                hop_deadline: std::time::Duration::from_secs(10),
                readiness_deadline: std::time::Duration::from_secs(10),
            })
            .expect("gated insert");

        let json = serde_json::to_value(shard_session_response(&registry, "session-xyz")).unwrap();
        assert_eq!(json["found"], true, "a mounted session is live");
        let view = json["session"].as_object().expect("view present");
        assert_eq!(view["session_id"], "session-xyz");
        assert_eq!(
            view["member_count"], 2,
            "two workers collapse to an aggregate count"
        );
        assert_eq!(view["rtt_frontier_ms"], 42);
        // The whitelist seam: NO member identity ever appears in the projection.
        let serialized = json.to_string();
        assert!(
            !serialized.contains(&hex::encode(worker_a.public_bytes())),
            "worker_a pubkey must not leak"
        );
        assert!(
            !serialized.contains(&hex::encode(worker_b.public_bytes())),
            "worker_b pubkey must not leak"
        );
        assert!(
            !serialized.contains(&hex::encode(head.public_bytes())),
            "initiator pubkey must not leak"
        );
        assert!(!serialized.contains("worker_pubkey"));
        assert!(!serialized.contains("initiator"));
        assert_eq!(
            view.len(),
            3,
            "view = exactly {{session_id, member_count, rtt_frontier_ms}}"
        );

        // The result projection of a mounted-but-undriven session: found,
        // every measurement null, drop count 0, same identity whitelist.
        let json =
            serde_json::to_value(shard_session_result_response(&registry, "session-xyz")).unwrap();
        assert_eq!(json["found"], true);
        let result = json["result"].as_object().expect("result view present");
        assert!(result["result_text"].is_null(), "no drive yet");
        assert!(result["run_proof"].is_null(), "no proof yet");
        assert_eq!(result["worker_drop_count"], 0);
        assert!(result["failure"].is_null());
        let serialized = json.to_string();
        assert!(!serialized.contains(&hex::encode(worker_a.public_bytes())));
        assert!(!serialized.contains(&hex::encode(head.public_bytes())));
    }

    #[tokio::test]
    async fn shard_session_routes_noop_in_duress() {
        // Sprint 81 Phase I: never sign a compute group, a session
        // manifest, or a RunProof under the fake keypair — group, mount
        // and generate short-circuit to a plausible benign reply BEFORE
        // any signing or dialing (mirror of seed_request_peer_noop_in_duress).
        let state = mk_state_with_mode(nexus_core_rs::IdentityMode::Duress).await;

        // group → {minted:false}, no signature minted.
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/shard-session/group")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"group_id": "g", "members": []}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(json["minted"], false, "duress must not mint a group");

        // mount → {mounted:false}, before placement / manifest signing /
        // any readiness dial. The body must be a VALID MountSessionRequest
        // (the Json extractor runs before the handler), so mint a real
        // signed group with a throwaway keypair.
        let head = nexus_core_rs::crypto::KeyPair::generate();
        let worker = nexus_core_rs::crypto::KeyPair::generate();
        let group = crate::shard_session::mint_compute_group(
            &head,
            "duress-group",
            1,
            &[worker.public_bytes()],
        )
        .expect("group mints");
        let worker_id = iroh::EndpointId::from_bytes(&worker.public_bytes()).expect("valid key");
        let body = serde_json::json!({
            "session_id": "duress-session",
            "group": group,
            "workers": [{
                "addr": iroh::EndpointAddr::new(worker_id),
                "vram_free_bytes": 1000,
            }],
            "model": { "total_layers": 8, "quantized_vram_bytes": 1500 },
        });
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/shard-session/mount")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(json["mounted"], false, "duress must not mount a session");
        assert!(
            state.shard_sessions.status_data("duress-session").is_none(),
            "duress must not populate the registry"
        );

        // generate → {accepted:false}, before any RunProof signing.
        let resp = build_test_router(Arc::clone(&state))
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/daemon/shard-session/duress-session/generate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::json!({"prompt": "p"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(
            json["accepted"], false,
            "duress must not drive a generation"
        );
    }
}
