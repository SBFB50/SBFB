// SPDX-License-Identifier: AGPL-3.0-or-later
//! Fairness diagnostic endpoint (Sprint 44 Phase B, port of diagnostic.py).
//!
//! Exposes Gini coefficient, top-5% share, and worker churn rate
//! computed from the kudos ledger.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use nexus_coordinator_rs::fairness;
use tracing::debug;

use crate::http::DaemonHttpState;

const DAY_SECS: u64 = 86400;

pub async fn fairness_metrics(
    State(state): State<Arc<DaemonHttpState>>,
) -> impl IntoResponse {
    debug!("GET /api/v1/diagnostic/fairness");
    let db = match state.coordinator_db.lock() {
        Ok(db) => db,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "db lock poisoned"})),
            )
                .into_response()
        }
    };

    let contributions = match db.worker_contributions() {
        Ok(c) => c,
        Err(_) => vec![],
    };

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let current_workers = db.active_workers_since(now.saturating_sub(DAY_SECS)).unwrap_or_default();
    let previous_workers = db
        .active_workers_since(now.saturating_sub(2 * DAY_SECS))
        .unwrap_or_default();

    drop(db);

    let worker_count = contributions.len();
    let gini = fairness::compute_gini(&contributions);
    let top_5_pct_share = fairness::compute_top_k_share(&contributions, 5);
    let churn_rate = fairness::compute_churn_rate(&previous_workers, &current_workers);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "gini": (gini * 10000.0).round() / 10000.0,
            "top_5_pct_share": (top_5_pct_share * 10000.0).round() / 10000.0,
            "churn_rate": (churn_rate * 10000.0).round() / 10000.0,
            "worker_count": worker_count,
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_secs_correct() {
        assert_eq!(DAY_SECS, 24 * 60 * 60);
    }

    #[test]
    fn rounding_precision() {
        let val = 0.12345_f64;
        let rounded = (val * 10000.0).round() / 10000.0;
        assert!((rounded - 0.1235).abs() < 1e-10);
    }
}
