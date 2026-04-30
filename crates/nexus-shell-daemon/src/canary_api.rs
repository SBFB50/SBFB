// SPDX-License-Identifier: AGPL-3.0-or-later
//! Canary input endpoints — port of remaining api/canary.py routes
//! (Sprint 43 Phase C). Completes the 3 routes already in http.rs
//! (network-health, observed, freshness) with inject-rate and
//! observed-divergence.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::http::DaemonHttpState;

#[derive(Debug, Deserialize)]
pub struct InjectRateBody {
    pub inject_rate: i64,
}

#[derive(Debug, Serialize)]
pub struct InjectRateResponse {
    pub status: String,
    pub inject_rate: usize,
}

pub async fn set_inject_rate(
    State(state): State<Arc<DaemonHttpState>>,
    Json(body): Json<InjectRateBody>,
) -> Result<Json<InjectRateResponse>, (StatusCode, String)> {
    let manager = state.canary_input.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "canary_input manager not initialised".into(),
    ))?;
    let new_rate = body.inject_rate.max(1) as usize;
    manager.set_inject_rate(new_rate);
    let current = manager.policy().inject_rate;
    Ok(Json(InjectRateResponse {
        status: "updated".into(),
        inject_rate: current,
    }))
}

#[derive(Debug, Deserialize)]
pub struct DivergenceQuery {
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

#[derive(Debug, Serialize)]
pub struct DivergenceResponse {
    pub divergences: Vec<serde_json::Value>,
    pub count: usize,
}

pub async fn observed_divergence(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<DivergenceQuery>,
) -> Result<Json<DivergenceResponse>, (StatusCode, String)> {
    let manager = state.canary_input.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "canary_input manager not initialised".into(),
    ))?;
    let capped = query.limit.min(100);
    let records = manager.recent_divergences(capped);
    let divergences: Vec<serde_json::Value> = records
        .iter()
        .map(|r| serde_json::to_value(r).unwrap_or_default())
        .collect();
    let count = divergences.len();
    Ok(Json(DivergenceResponse { divergences, count }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inject_rate_body_deserializes() {
        let json = r#"{"inject_rate": 50}"#;
        let body: InjectRateBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.inject_rate, 50);
    }

    #[test]
    fn divergence_query_defaults() {
        let q: DivergenceQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(q.limit, 50);
    }

    #[test]
    fn inject_rate_response_serializes() {
        let resp = InjectRateResponse {
            status: "updated".into(),
            inject_rate: 42,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("42"));
    }
}
