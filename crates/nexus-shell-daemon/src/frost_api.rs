// SPDX-License-Identifier: AGPL-3.0-or-later
//! FROST DKG + ceremony loopback HTTP endpoints — extracted verbatim
//! from `http.rs` (Sprint 82 Phase P, PO-10 extended discipline: the
//! domain's 8 tests co-migrated below via `crate::test_support`).
//!
//! Stateless Json -> Json glue over the
//! `nexus_shell_daemon_core::canary` FROST primitives (Sprint 30
//! Phase C warrant-canary DKG + signing ceremony). T0-admin tier:
//! the routes stay registered in `crate::http::build_router` inside
//! `authed_routes` (loopback bearer + Host + Origin) and re-point
//! here by full path; route paths, JSON shapes and status codes are
//! unchanged. No secret is ever persisted by this layer.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct FrostTrustedDealerRequest {
    k: u16,
    n: u16,
}

#[derive(Debug, Serialize)]
struct FrostTrustedDealerResponse {
    shares: Vec<nexus_shell_daemon_core::canary::DkgShareFile>,
    pubkey_package: nexus_shell_daemon_core::canary::DkgPubkeyFile,
}

pub(crate) async fn frost_trusted_dealer(
    Json(body): Json<FrostTrustedDealerRequest>,
) -> impl IntoResponse {
    match nexus_shell_daemon_core::canary::generate_dkg(body.k, body.n) {
        Ok((shares, pubkey_package)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostTrustedDealerResponse {
                shares,
                pubkey_package
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct FrostRound1Request {
    participant: u16,
    key_package_hex: String,
}

#[derive(Debug, Serialize)]
struct FrostRound1Response {
    commitment: nexus_shell_daemon_core::canary::CeremonyCommitment,
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
}

pub(crate) async fn frost_round1(Json(body): Json<FrostRound1Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round1(
        body.participant,
        &frost_share.key_package,
    ) {
        Ok((commitment, nonces)) => (
            StatusCode::OK,
            Json(serde_json::json!(FrostRound1Response {
                commitment,
                nonces
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct FrostRound2Request {
    nonces: nexus_shell_daemon_core::canary::CeremonyNonces,
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    key_package_hex: String,
    participant: u16,
}

pub(crate) async fn frost_round2(Json(body): Json<FrostRound2Request>) -> impl IntoResponse {
    let share_file = nexus_shell_daemon_core::canary::DkgShareFile {
        participant: body.participant,
        key_package_hex: body.key_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let frost_share = match nexus_shell_daemon_core::canary::load_share(&share_file) {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_round2(
        &body.nonces,
        &body.signing_package,
        &frost_share.key_package,
    ) {
        Ok(sig_share) => (StatusCode::OK, Json(serde_json::json!(sig_share))).into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct FrostAggregateRequest {
    signing_package: nexus_shell_daemon_core::canary::CeremonySigningPackage,
    shares: Vec<nexus_shell_daemon_core::canary::CeremonySignatureShare>,
    pubkey_package_hex: String,
}

pub(crate) async fn frost_aggregate(Json(body): Json<FrostAggregateRequest>) -> impl IntoResponse {
    let pubkey_file = nexus_shell_daemon_core::canary::DkgPubkeyFile {
        verifying_key_hex: String::new(),
        pubkey_package_hex: body.pubkey_package_hex,
        min_signers: 0,
        max_signers: 0,
    };
    let pubkey = match nexus_shell_daemon_core::canary::load_pubkey(&pubkey_file) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response();
        }
    };
    match nexus_shell_daemon_core::canary::ceremony_aggregate(
        &body.signing_package,
        &body.shares,
        pubkey.package(),
    ) {
        Ok(sig) => (
            StatusCode::OK,
            Json(serde_json::json!({ "signature_hex": hex::encode(sig) })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

    #[tokio::test]
    async fn frost_http_trusted_dealer_returns_shares_and_pubkey() {
        let app = build_test_router(mk_state().await);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let shares = json["shares"].as_array().expect("shares array");
        assert_eq!(shares.len(), 3, "K=2/N=3 must produce 3 shares");
        for (i, share) in shares.iter().enumerate() {
            assert_eq!(
                share["participant"].as_u64().unwrap(),
                (i + 1) as u64,
                "share participant must be 1-indexed"
            );
            assert!(
                !share["key_package_hex"].as_str().unwrap().is_empty(),
                "key_package_hex must be non-empty"
            );
        }
        let pubkey = &json["pubkey_package"];
        assert!(
            !pubkey["verifying_key_hex"].as_str().unwrap().is_empty(),
            "verifying_key_hex must be non-empty"
        );
        assert_eq!(
            pubkey["verifying_key_hex"].as_str().unwrap().len(),
            64,
            "verifying key must be 32 bytes (64 hex chars)"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_returns_commitment_and_nonces() {
        let app_dealer = build_test_router(mk_state().await);
        let dealer_resp = app_dealer
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dealer_body = to_bytes(dealer_resp.into_body(), 16384).await.unwrap();
        let dealer_json: serde_json::Value = serde_json::from_slice(&dealer_body).unwrap();
        let share = &dealer_json["shares"][0];
        let key_package_hex = share["key_package_hex"].as_str().unwrap();
        let participant = share["participant"].as_u64().unwrap() as u16;

        let round1_body = serde_json::json!({
            "participant": participant,
            "key_package_hex": key_package_hex
        });

        let app_round1 = build_test_router(mk_state().await);
        let resp = app_round1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(round1_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            !json["commitment"]["commitment_hex"]
                .as_str()
                .unwrap()
                .is_empty(),
            "commitment_hex must be non-empty"
        );
        assert!(
            !json["nonces"]["nonces_hex"].as_str().unwrap().is_empty(),
            "nonces_hex must be non-empty"
        );
        assert_eq!(
            json["commitment"]["participant"].as_u64().unwrap(),
            participant as u64
        );
    }

    #[tokio::test]
    async fn frost_http_round2_returns_signature_share() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(r1_resp.status(), StatusCode::OK);
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let sp = nexus_shell_daemon_core::canary::build_signing_package(
            &commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect::<Vec<nexus_shell_daemon_core::canary::CeremonyCommitment>>(),
            b"round2 HTTP test message",
        )
        .expect("build signing package");

        let r2_body = serde_json::json!({
            "nonces": nonces_list[0],
            "signing_package": sp,
            "key_package_hex": dj["shares"][0]["key_package_hex"],
            "participant": dj["shares"][0]["participant"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round2")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r2_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(
            !json["signature_share_hex"].as_str().unwrap().is_empty(),
            "signature_share_hex must be non-empty"
        );
    }

    #[tokio::test]
    async fn frost_http_aggregate_returns_valid_signature() {
        let state = mk_state().await;

        let app1 = build_test_router(Arc::clone(&state));
        let dealer_resp = app1
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let mut commitments = Vec::new();
        let mut nonces_list = Vec::new();
        for i in 0..2 {
            let share = &dj["shares"][i];
            let r1_body = serde_json::json!({
                "participant": share["participant"],
                "key_package_hex": share["key_package_hex"]
            });
            let app = build_test_router(Arc::clone(&state));
            let r1_resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round1")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r1_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let r1j: serde_json::Value =
                serde_json::from_slice(&to_bytes(r1_resp.into_body(), 16384).await.unwrap())
                    .unwrap();
            commitments.push(r1j["commitment"].clone());
            nonces_list.push(r1j["nonces"].clone());
        }

        let message = b"aggregate HTTP test message";
        let ceremony_commitments: Vec<nexus_shell_daemon_core::canary::CeremonyCommitment> =
            commitments
                .iter()
                .map(|c| serde_json::from_value(c.clone()).unwrap())
                .collect();
        let sp =
            nexus_shell_daemon_core::canary::build_signing_package(&ceremony_commitments, message)
                .expect("build signing package");

        let mut sig_shares = Vec::new();
        for i in 0..2 {
            let r2_body = serde_json::json!({
                "nonces": nonces_list[i],
                "signing_package": sp,
                "key_package_hex": dj["shares"][i]["key_package_hex"],
                "participant": dj["shares"][i]["participant"]
            });
            let app = build_test_router(Arc::clone(&state));
            let resp = app
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/canary/frost/round2")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(r2_body.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let rj: serde_json::Value =
                serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
            sig_shares.push(rj);
        }

        let agg_body = serde_json::json!({
            "signing_package": sp,
            "shares": sig_shares,
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        let sig_hex = json["signature_hex"]
            .as_str()
            .expect("signature_hex present");
        assert_eq!(sig_hex.len(), 128, "64-byte Ed25519 sig = 128 hex chars");

        let sig_bytes = hex::decode(sig_hex).expect("valid hex");
        let vk_hex = dj["pubkey_package"]["verifying_key_hex"].as_str().unwrap();
        let vk_bytes = hex::decode(vk_hex).expect("valid vk hex");
        let vk: [u8; 32] = vk_bytes.try_into().expect("32 bytes");
        let sig: [u8; 64] = sig_bytes.try_into().expect("64 bytes");
        nexus_core_rs::crypto::verify(&vk, message, &sig)
            .expect("aggregated FROST sig must verify as Ed25519");
    }

    #[tokio::test]
    async fn frost_http_invalid_threshold_k_gt_n() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":5,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_malformed_json_body() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k": not valid json"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(
            resp.status().is_client_error(),
            "malformed JSON should return 4xx"
        );
    }

    #[tokio::test]
    async fn frost_http_round1_invalid_key_package() {
        let state = mk_state().await;
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/round1")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"participant":1,"key_package_hex":"deadbeef"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }

    #[tokio::test]
    async fn frost_http_aggregate_invalid_pubkey() {
        let state = mk_state().await;

        let app = build_test_router(Arc::clone(&state));
        let dealer_resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/trusted-dealer")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"k":2,"n":3}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        let dj: serde_json::Value =
            serde_json::from_slice(&to_bytes(dealer_resp.into_body(), 16384).await.unwrap())
                .unwrap();

        let agg_body = serde_json::json!({
            "signing_package": { "signing_package_hex": "deadbeef" },
            "shares": [],
            "pubkey_package_hex": dj["pubkey_package"]["pubkey_package_hex"]
        });
        let app = build_test_router(Arc::clone(&state));
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/canary/frost/aggregate")
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(agg_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert!(body["error"].as_str().is_some());
    }
}
