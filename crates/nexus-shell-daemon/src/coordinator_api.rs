// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator Rust-native task/result/kudos loopback HTTP domain —
//! extracted verbatim from `http.rs` (Sprint 82 Phase Q, PO-10 extended
//! discipline: the domain's 13 router-driven tests co-migrated below via
//! `crate::test_support`).
//!
//! Task submit (Sprint 35 Phase B) runs the input guardrail chain
//! BEFORE dispatch plus the S76-A local-worker nudge; result submit keeps the
//! output-guardrail-BEFORE-persist invariant (Sprint 73 Phase A, D5)
//! with the CARRY-2 terminal tripwire and feeds accepted entries to the
//! `result_event_tx` bridge (Sprint 36 Phase B); kudos read (Sprint 36
//! Phase C) and verify_chain (Sprint 38 Phase A) are read-only ledger
//! views. T0 tier: the routes stay registered in
//! `crate::http::build_router` inside `authed_routes` (loopback bearer +
//! Host + Origin) and re-point here by full path; route paths, JSON
//! shapes and status codes are unchanged.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::http::DaemonHttpState;

// =================================================================
// Sprint 35 Phase B — Coordinator Rust-native task submission
// =================================================================

pub(crate) async fn coordinator_submit_task(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(submission): axum::Json<nexus_coordinator_rs::types::TaskSubmission>,
) -> impl IntoResponse {
    let input_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
        system_prompt: &submission.system_prompt,
        user_prompt: &submission.prompt,
        model_output: "",
    };
    let input_check = nexus_coordinator_rs::guardrails::default_input_chain().run(&input_ctx);
    if !input_check.passed {
        let reason = input_check
            .tripwire
            .unwrap_or_else(|| "input_guardrail_rejected".into());
        tracing::warn!(
            project_id = %submission.project_id,
            %reason,
            "task rejected by input guardrail"
        );
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "input_rejected", "reason": reason})),
        )
            .into_response();
    }

    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    let keypair = (*state.pow_keypair).clone();
    match nexus_coordinator_rs::dispatcher::submit_task(&db, &keypair, submission) {
        Ok(entry) => {
            if let Some(ref tx) = state.task_dispatch_tx
                && let Err(e) = tx.try_send(entry.clone())
            {
                tracing::warn!("dispatch channel full or closed: {e}");
            }
            // Hotfix #5 (maillon A): nudge the on-demand local worker so
            // a node executes its own tasks without a manual
            // `nexus-worker` setup. Fire-and-forget — the cold start
            // (worker boot + doc sync) runs in the background; the
            // submit returns the task id immediately. Idempotent.
            if let Some(doc) = state.project_doc.clone() {
                let lw = std::sync::Arc::clone(&state.local_worker);
                // Sprint 76 Phase A (D1): pass the user's resolved
                // SBFB_HOME so the provisioned worker can adopt the
                // public sharing level the "offer my power" panel wrote.
                let user_home = state.sbfb_home.clone();
                tokio::spawn(async move { lw.ensure_spawned(doc, user_home).await });
            }
            match serde_json::to_value(&entry) {
                Ok(body) => (StatusCode::OK, Json(body)).into_response(),
                Err(e) => {
                    tracing::error!("task entry serialization failed: {e}");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "internal"})),
                    )
                        .into_response()
                }
            }
        }
        Err(nexus_coordinator_rs::error::CoordinatorError::Validation(msg)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": msg})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("task submit failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase B — Coordinator Rust-native result submission
// =================================================================

pub(crate) async fn coordinator_submit_result(
    State(state): State<Arc<DaemonHttpState>>,
    axum::Json(entry): axum::Json<nexus_core_rs::task::ResultEntry>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::validator::validate_result_pre_guardrail(&db, &entry) {
        Ok((
            nexus_coordinator_rs::validator::ValidationOutcome::Accepted,
            Some(task_record),
            Some(pending),
        )) => {
            // Sprint 73 Phase A (D5): run the output guardrail BEFORE
            // persisting. The pre phase has written no `result_text` yet, so
            // a tripwire here leaves zero retrievable content (nothing for
            // `GET /api/v1/tasks/{id}/result` to serve) and credits no kudos.
            let guardrail_ctx = nexus_coordinator_rs::guardrails::GuardrailContext {
                system_prompt: "",
                user_prompt: "",
                model_output: &pending.result_text,
            };
            let gr = nexus_coordinator_rs::guardrails::default_output_chain().run(&guardrail_ctx);
            if !gr.passed {
                let reason = gr.tripwire.unwrap_or_else(|| "guardrail_rejected".into());
                tracing::warn!(
                    task_id = %entry.payload.task_id,
                    %reason,
                    "result rejected by output guardrail — not persisted, no kudos credited"
                );
                // CARRY-2 (S74 audit, Sprint 75 Phase G): a tripwire is
                // terminal — the validated submission is already consumed, so
                // leaving the task Pending/AwaitingQuorum would zombie it
                // forever. Same transition as the gossip `validator_loop`.
                if let Err(e) =
                    nexus_coordinator_rs::validator::reject_result_on_guardrail_trip(&db, &pending)
                {
                    tracing::error!(
                        task_id = %entry.payload.task_id,
                        "failed to mark guardrail-tripped task rejected: {e}"
                    );
                }
                return (
                    StatusCode::BAD_REQUEST,
                    Json(
                        serde_json::json!({"outcome": "rejected", "reason": "guardrail_rejected"}),
                    ),
                )
                    .into_response();
            }
            if let Err(e) =
                nexus_coordinator_rs::validator::validate_result_post_guardrail(&db, &pending)
            {
                tracing::error!(task_id = %entry.payload.task_id, "result persist failed: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response();
            }
            let worker_id = hex::encode(entry.worker_pubkey);
            if let Err(e) = nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                &task_record.project_id,
                &worker_id,
                &entry.payload.task_id,
                entry.payload.tokens_generated,
                entry.payload.generation_time_ms,
            ) {
                tracing::warn!("kudos credit failed (non-fatal): {e}");
            }
            let _ = state
                .result_event_tx
                .send(crate::validator_loop::ResultEvent::NewResult(entry));
            (
                StatusCode::OK,
                Json(serde_json::json!({"outcome": "accepted"})),
            )
                .into_response()
        }
        Ok((nexus_coordinator_rs::validator::ValidationOutcome::AwaitingQuorum, _, _)) => (
            StatusCode::OK,
            Json(serde_json::json!({"outcome": "awaiting_quorum"})),
        )
            .into_response(),
        Ok((outcome, _, _)) => {
            let reason = match outcome {
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedBadSignature => {
                    "bad_signature"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotFound => {
                    "task_not_found"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::RejectedTaskNotPending => {
                    "task_not_pending"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::QuorumRejected => {
                    "quorum_divergence"
                }
                nexus_coordinator_rs::validator::ValidationOutcome::Accepted
                | nexus_coordinator_rs::validator::ValidationOutcome::AwaitingQuorum => {
                    unreachable!()
                }
            };
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"outcome": "rejected", "reason": reason})),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("result validation failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 36 Phase C — Kudos read endpoint
// =================================================================

pub(crate) async fn coordinator_get_kudos(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    match nexus_coordinator_rs::kudos_ledger::get_project_kudos(&db, &project_id, now_secs) {
        Ok(kudos) => match serde_json::to_value(&kudos) {
            Ok(body) => (StatusCode::OK, Json(body)).into_response(),
            Err(e) => {
                tracing::error!("kudos serialization failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal"})),
                )
                    .into_response()
            }
        },
        Err(e) => {
            tracing::error!("kudos query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

// =================================================================
// Sprint 38 Phase A — verify_chain endpoint
// =================================================================

pub(crate) async fn coordinator_verify_chain(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Path(project_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            tracing::error!("coordinator DB mutex poisoned");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };
    match nexus_coordinator_rs::kudos_ledger::verify_chain(&db, &project_id) {
        Ok(valid) => (StatusCode::OK, Json(serde_json::json!({"valid": valid}))).into_response(),
        Err(e) => {
            tracing::error!("verify_chain failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use nexus_core_rs::KeyPair;
    use tower::ServiceExt;

    use crate::test_support::*;

    // ===============================================================
    // Sprint 36 Phase B — result submission integration tests
    // ===============================================================

    fn make_result_entry(task_id: &str, worker_kp: &KeyPair) -> nexus_core_rs::task::ResultEntry {
        make_result_entry_with_text(task_id, worker_kp, "result text")
    }

    fn make_result_entry_with_text(
        task_id: &str,
        worker_kp: &KeyPair,
        text: &str,
    ) -> nexus_core_rs::task::ResultEntry {
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: task_id.to_string(),
            result_text: text.into(),
            tokens_generated: 42,
            generation_time_ms: 1000,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 100,
            finished_at: 200,
            output_token_ids: vec![],
        };
        nexus_core_rs::task::ResultEntry::sign(payload, worker_kp).expect("sign result")
    }

    #[tokio::test]
    async fn result_submit_accepts_valid() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "accepted");

        let db = state.coordinator_db.lock().unwrap();
        let task = db
            .get_task(&task_entry.task.task_id)
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            nexus_coordinator_rs::types::TaskStatus::Completed
        );
    }

    #[tokio::test]
    async fn result_submit_rejects_bad_signature() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let mut result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);
        result_entry.signature[0] ^= 0xff;

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "bad_signature");
    }

    #[tokio::test]
    async fn result_submit_rejects_unknown_task() {
        let state = mk_state().await;
        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry("nonexistent-task-id", &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_found");
    }

    #[tokio::test]
    async fn result_submit_rejects_completed_task() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        {
            let db = state.coordinator_db.lock().unwrap();
            db.set_task_result(&task_entry.task.task_id, "w1", "r1", "prior text", 100)
                .expect("complete");
        }

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["reason"], "task_not_pending");
    }

    // ===============================================================
    // Sprint 73 Phase A — guardrail-before-persist (D5)
    // ===============================================================

    #[tokio::test]
    async fn submit_result_rejected_by_guardrail_persists_nothing() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        // An invisible character trips the output guardrail deterministically.
        let result_entry = make_result_entry_with_text(
            &task_entry.task.task_id,
            &worker_kp,
            "leaked\u{200B}secret",
        );

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "rejected");
        assert_eq!(body["reason"], "guardrail_rejected");

        // Sprint 73 Phase A (D5): the guardrail runs BEFORE persistence, so a
        // rejected result leaves no completed task, no retrievable text, and
        // credits no kudos. Since CARRY-2 (Sprint 75 Phase G) the trip is also
        // TERMINAL on this HTTP path: the task flips to Rejected instead of
        // silently keeping its prior non-terminal state.
        let db = state.coordinator_db.lock().unwrap();
        let task = db
            .get_task(&task_entry.task.task_id)
            .expect("get")
            .expect("found");
        assert_eq!(
            task.status,
            nexus_coordinator_rs::types::TaskStatus::Rejected,
            "guardrail-rejected result must terminally reject the task (CARRY-2)"
        );
        assert!(
            db.get_task_result(&task_entry.task.task_id)
                .expect("get")
                .expect("found")
                .result_text
                .is_none(),
            "guardrail-rejected result must persist no retrievable text"
        );
        assert_eq!(
            db.get_project_kudos_total("test-project").expect("kudos"),
            0,
            "no kudos for guardrail-rejected output"
        );
    }

    #[tokio::test]
    async fn submit_result_accepted_persists_after_guardrail() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry =
            make_result_entry_with_text(&task_entry.task.task_id, &worker_kp, "clean answer");

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["outcome"], "accepted");

        // The cleared text is persisted retrievably only after the guardrail.
        let db = state.coordinator_db.lock().unwrap();
        assert_eq!(
            db.get_task_result(&task_entry.task.task_id)
                .expect("get")
                .expect("found")
                .result_text
                .as_deref(),
            Some("clean answer"),
        );
    }

    // ===============================================================
    // Sprint 36 Phase C — kudos integration tests
    // ===============================================================

    #[tokio::test]
    async fn e2e_task_result_kudos_credited() {
        let state = mk_state().await;
        let coord_kp = (*state.pow_keypair).clone();

        let task_entry = {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::dispatcher::submit_task(&db, &coord_kp, make_test_submission())
                .expect("submit task")
        };

        let worker_kp = KeyPair::generate();
        let result_entry = make_result_entry(&task_entry.task.task_id, &worker_kp);

        let app = build_test_router(state.clone());
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        serde_json::to_vec(&result_entry).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let db = state.coordinator_db.lock().unwrap();
        let total = db
            .get_project_kudos_total("test-project")
            .expect("kudos total");
        assert!(total > 0, "kudos must be credited after accepted result");
    }

    #[tokio::test]
    async fn kudos_endpoint_returns_json() {
        let state = mk_state().await;

        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(
                &db,
                "proj-abc",
                "worker-xyz",
                "task-1",
                100,
                1_000,
            )
            .expect("credit");
        }

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-abc")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(body["project_id"], "proj-abc");
        assert!(
            body["total"].as_u64().unwrap() > 0,
            "total must be positive after credit"
        );
        assert_eq!(body["contributors"][0]["worker_node_id"], "worker-xyz");
    }

    // =========================================================
    // Mutex poisoned tests (P2-REVIEW-A-1/B-1)
    // =========================================================

    #[tokio::test]
    async fn submit_task_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        // Poison the mutex by panicking while holding the guard.
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();
        assert!(
            state.coordinator_db.lock().is_err(),
            "mutex must be poisoned"
        );

        let app = build_test_router(state);
        let body = serde_json::json!({
            "project_id": "p1",
            "task_type": "inference",
            "prompt": "test",
            "system_prompt": "",
            "model": "llama3"
        });
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn submit_result_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let kp = KeyPair::generate();
        let payload = nexus_core_rs::task::ResultPayload {
            version: nexus_core_rs::task::TASK_FORMAT_VERSION,
            task_id: "t-1".to_string(),
            result_text: "out".to_string(),
            tokens_generated: 1,
            generation_time_ms: 1,
            model_digest: [0u8; 32],
            logprobs_hash: [0u8; 32],
            started_at: 0,
            finished_at: 1,
            output_token_ids: vec![],
        };
        let entry = nexus_core_rs::task::ResultEntry::sign(payload, &kp).unwrap();
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/v1/results/submit")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(serde_json::to_vec(&entry).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn get_kudos_returns_500_on_poisoned_mutex() {
        let state = mk_state().await;
        let db_arc = Arc::clone(&state.coordinator_db);
        let _ = std::thread::spawn(move || {
            let _guard = db_arc.lock().unwrap();
            panic!("intentional poison");
        })
        .join();

        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-1")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn verify_chain_endpoint_returns_valid() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::kudos_ledger::credit(
                &db, "proj-vc", "worker-a", "task-1", 10, 1_000,
            )
            .expect("credit");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/v1/kudos/proj-vc/verify")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["valid"], true);
    }

    #[tokio::test]
    async fn submit_task_pii_rejected() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let mut sub = make_test_submission();
        sub.prompt = "Contact me at test@example.com for details".into();
        let resp = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/submit")
                    .header("content-type", "application/json")
                    .header("host", "127.0.0.1")
                    .header("authorization", format!("Bearer {TEST_TOKEN}"))
                    .body(axum::body::Body::from(serde_json::to_vec(&sub).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
        assert_eq!(body["error"], "input_rejected");
    }
}
