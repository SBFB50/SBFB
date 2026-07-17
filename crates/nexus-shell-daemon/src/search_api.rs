// SPDX-License-Identifier: AGPL-3.0-or-later
//! FTS5 search loopback HTTP domain — extracted verbatim from `http.rs`
//! (Sprint 82 Phase S3, PO-10 extended discipline: the domain's 8
//! router-driven tests co-migrated below via `crate::test_support`).
//!
//! `GET /api/daemon/search` runs the Sprint 67 Phase B FTS5 query
//! against the LOCAL `search_index` (no wire format —
//! `FEED_FORMAT_VERSION` stays 1), returning rows that carry the
//! Sprint 73 Phase D additive provenance triplet (UNINDEXED, never
//! matchable) so a search hit can drive a fork. Attacker-supplied
//! `offset`/`q` params are clamped server-side (`MAX_SEARCH_OFFSET`,
//! `MAX_SEARCH_QUERY_BYTES` — CARRY-5, Sprint 75 Phase G); the
//! UTF-8-safe truncation helper stays in `crate::http`
//! (`truncate_on_char_boundary`, dual-domain with the
//! `NODE_DIRECTORY_*_MAX` catalog caps). T0 tier: the route stays
//! registered in `crate::http::build_router` inside `authed_routes`
//! (loopback bearer + Host + Origin) and re-points here by full path;
//! route path, JSON shape and status codes are unchanged.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::http::{DaemonHttpState, truncate_on_char_boundary};

// =================================================================
// Sprint 67 Phase B: FTS5 search endpoint
// =================================================================

#[derive(Debug, serde::Deserialize)]
pub(crate) struct SearchQuery {
    q: String,
    #[serde(default = "default_search_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_search_limit() -> usize {
    20
}

/// Server-side caps on the attacker-supplied search params (Sprint 75
/// Phase G, CARRY-5 / S74 audit). `limit` was already clamped to 100; an
/// unbounded `offset` walks the whole FTS5 match set inside SQLite
/// (`LIMIT ?2 OFFSET ?3`) — and `usize::MAX as i64` even flips negative,
/// which SQLite silently treats as "no offset". An unbounded `q` is
/// tokenised + quoted per word before the MATCH parse, so a megabyte
/// query is a cheap CPU/allocation lever on the loopback API.
const MAX_SEARCH_OFFSET: usize = 10_000;
const MAX_SEARCH_QUERY_BYTES: usize = 1024;

pub(crate) async fn search_handler(
    State(state): State<Arc<DaemonHttpState>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> impl IntoResponse {
    let db = match state.coordinator_db.lock() {
        Ok(guard) => guard,
        Err(_poisoned) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let limit = params.limit.min(100);
    let offset = params.offset.min(MAX_SEARCH_OFFSET);
    // UTF-8-safe truncation: a naive byte slice would panic mid-char.
    let q = truncate_on_char_boundary(&params.q, MAX_SEARCH_QUERY_BYTES);
    let start = std::time::Instant::now();

    let (results, total) = match nexus_coordinator_rs::search::search(&db, &q, limit, offset) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("search query failed: {e}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "internal"})),
            )
                .into_response();
        }
    };

    let took_ms = start.elapsed().as_millis() as u64;
    let entries: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "project_id": r.project_id,
                "project_name": r.project_name,
                "category": r.category,
                "description": r.description,
                "op_type": r.op_type,
                "source_type": r.source_type,
                "score": r.score,
                // Provenance triplet (Sprint 73 Phase D): additive keys so a
                // search hit can drive a fork in S74. `null` for non-release
                // ops; never matchable (UNINDEXED). No wire-format bump —
                // search_index is local, FEED_FORMAT_VERSION stays 1.
                "repo_url": r.repo_url,
                "commit_sha": r.commit_sha,
                "archive_hash": r.archive_hash,
                "provenance_hash": r.provenance_hash,
                "is_open_source": r.is_open_source,
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "results": entries,
            "total": total,
            "took_ms": took_ms,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::to_bytes;
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use crate::test_support::*;

    // -- Sprint 67 Phase B: search endpoint test --

    #[tokio::test]
    async fn test_search_endpoint_http() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-search",
                "Babel Translator",
                "translation",
                "A real-time translation tool",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance::default(),
            )
            .expect("index");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=translation")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total"], 1);
        let results = json["results"].as_array().unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["project_name"], "Babel Translator");
        assert!(json["took_ms"].as_u64().is_some());
    }

    // -- Search hotfix (Sprint 73 audit): real publish->search boundary --

    /// E2E at the REAL boundary (no mockFetch, no test-only `index_entry`
    /// injection): a project published through the production `POST
    /// /api/daemon/publish` handler MUST become findable through the production
    /// `GET /api/daemon/search` handler. This crosses the deploy/publish ->
    /// FTS5-index seam that every prior test mocked or bypassed, which is why a
    /// fully broken search shipped with green tests. Asserts the three facets of
    /// the hotfix at once: (1) the app is indexed on publish, (2) PREFIX search
    /// ("Bab" -> "Babel") works, (3) re-publish dedups instead of duplicating.
    #[tokio::test]
    async fn publish_makes_app_searchable_by_name() {
        let state = mk_state().await;

        async fn do_publish(router: Router) -> StatusCode {
            let body = serde_json::json!({
                "project_name": "Babel Translator",
                "category": "translation",
                "description": "real-time peer to peer translation",
            });
            router
                .oneshot(
                    Request::builder()
                        .method(Method::POST)
                        .uri("/api/daemon/publish")
                        .header("content-type", "application/json")
                        .body(axum::body::Body::from(serde_json::to_vec(&body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap()
                .status()
        }

        async fn do_search(router: Router, q: &str) -> serde_json::Value {
            let resp = router
                .oneshot(
                    Request::builder()
                        .method(Method::GET)
                        .uri(format!("/api/daemon/search?q={q}"))
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(resp.status(), StatusCode::OK);
            let bytes = to_bytes(resp.into_body(), 16384).await.unwrap();
            serde_json::from_slice(&bytes).unwrap()
        }

        // Publish through the real handler (shares coordinator_db across routers).
        assert_eq!(
            do_publish(build_test_router(state.clone())).await,
            StatusCode::OK
        );

        // (1) exact-name search finds it.
        let json = do_search(build_test_router(state.clone()), "Babel").await;
        assert_eq!(
            json["total"], 1,
            "publish must make the app searchable by name"
        );
        assert_eq!(json["results"][0]["project_name"], "Babel Translator");

        // (2) prefix search ("Bab") finds "Babel".
        let json = do_search(build_test_router(state.clone()), "Bab").await;
        assert_eq!(json["total"], 1, "prefix search must find the app");

        // (3) re-publishing the same project dedups (deterministic browse rowid).
        assert_eq!(
            do_publish(build_test_router(state.clone())).await,
            StatusCode::OK
        );
        let json = do_search(build_test_router(state.clone()), "Babel").await;
        assert_eq!(
            json["total"], 1,
            "re-publish must not duplicate the index row"
        );
    }

    #[tokio::test]
    async fn published_app_searchable_by_category() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Quiet Name", "translation", "x").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "translation").await,
            1,
            "find by category"
        );
        assert_eq!(search_total(&state, "transl").await, 1, "category prefix");
    }

    #[tokio::test]
    async fn published_app_searchable_by_single_letter() {
        // The exact user symptom, end-to-end through the real handlers: a
        // published "sbfb-*" app must be found by typing the single letter "s".
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "sbfb-explorer", "tools", "protocol explorer").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "s").await,
            1,
            "single-letter 's' finds it"
        );
        assert_eq!(
            search_total(&state, "explor").await,
            1,
            "inner-token prefix finds it"
        );
    }

    #[tokio::test]
    async fn published_app_searchable_by_description_word() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Plain", "misc", "end to end encryption demo").await,
            StatusCode::OK
        );
        assert_eq!(
            search_total(&state, "encryption").await,
            1,
            "find by description word"
        );
        assert_eq!(
            search_total(&state, "encrypt").await,
            1,
            "description prefix"
        );
    }

    #[tokio::test]
    async fn published_app_searchable_by_multi_word_query() {
        let state = mk_state().await;
        assert_eq!(
            publish_app(&state, "Babel Translator", "translation", "fast").await,
            StatusCode::OK
        );
        // Multi-word query (space URL-encoded): all terms must match (AND).
        assert_eq!(
            search_total(&state, "babel translator").await,
            1,
            "multi-word AND matches"
        );
        assert_eq!(
            search_total(&state, "nomatch translator").await,
            0,
            "a missing term yields no match"
        );
    }

    // -- Sprint 73 Phase D: search JSON carries the provenance triplet --

    #[tokio::test]
    async fn search_handler_json_includes_triplet() {
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-fork",
                "Forkable App",
                "tools",
                "an app a search hit can fork",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance {
                    repo_url: Some("https://github.com/test/forkable"),
                    commit_sha: Some("abc1230000000000000000000000000000000000"),
                    archive_hash: Some(
                        "dd00000000000000000000000000000000000000000000000000000000000000",
                    ),
                    provenance_hash: Some(
                        "ee00000000000000000000000000000000000000000000000000000000000000",
                    ),
                    is_open_source: true,
                },
            )
            .expect("index");
        }
        let app = build_test_router(state);
        let resp = app
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=forkable")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = to_bytes(resp.into_body(), 16384).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total"], 1);
        let hit = &json["results"].as_array().unwrap()[0];
        // The four additive provenance keys (+ open-source flag) are present
        // and populated so the S74 atelier can fork from a search hit.
        assert_eq!(hit["repo_url"], "https://github.com/test/forkable");
        assert_eq!(
            hit["commit_sha"],
            "abc1230000000000000000000000000000000000"
        );
        assert_eq!(
            hit["archive_hash"],
            "dd00000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            hit["provenance_hash"],
            "ee00000000000000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(hit["is_open_source"], true);
    }

    #[tokio::test]
    async fn search_clamps_offset_and_query() {
        // CARRY-5 (S74 audit, Sprint 75 Phase G): `offset` and `q` are
        // attacker-supplied query params; only `limit` was clamped before.
        let state = mk_state().await;
        {
            let db = state.coordinator_db.lock().unwrap();
            nexus_coordinator_rs::search::index_entry(
                &db,
                "proj-clamp",
                "Clampable App",
                "tools",
                "an app for the clamp test",
                "",
                "browse",
                &nexus_coordinator_rs::search::Provenance::default(),
            )
            .expect("index");
        }

        // (a) offset = usize::MAX. Unclamped, `usize::MAX as i64` flips to -1
        // and SQLite treats a negative OFFSET as zero — the row would come
        // BACK. Clamped to MAX_SEARCH_OFFSET (way past the 1-row match set),
        // the page is defined and empty while `total` still counts the match.
        let resp = build_test_router(state.clone())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/daemon/search?q=clampable&offset={}", usize::MAX).as_str())
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(json["total"], 1, "total still counts the match");
        assert_eq!(
            json["results"].as_array().unwrap().len(),
            0,
            "a huge offset must be clamped, not wrap negative and return rows"
        );

        // (b) q far beyond MAX_SEARCH_QUERY_BYTES, with a multi-byte char
        // straddling the 1024-byte cut: a naive byte slice would panic
        // mid-char (500); the boundary-safe truncation must answer 200.
        // `%C3%A9` percent-encodes "é" (2 bytes once decoded), so the
        // decoded q is 1023 ASCII bytes then 2000 two-byte chars — the cut
        // at byte 1024 falls mid-"é".
        let big_q = format!(
            "{}{}",
            "x".repeat(MAX_SEARCH_QUERY_BYTES - 1),
            "%C3%A9".repeat(2_000)
        );
        let resp = build_test_router(state.clone())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(format!("/api/daemon/search?q={big_q}").as_str())
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "an oversized q must be truncated UTF-8-safely, never error"
        );

        // (c) sanity: a normal query still finds the row.
        let resp = build_test_router(state)
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/daemon/search?q=clampable")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 16384).await.unwrap()).unwrap();
        assert_eq!(json["results"].as_array().unwrap().len(), 1);
    }
}
