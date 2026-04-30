// SPDX-License-Identifier: AGPL-3.0-or-later
//! App listing handlers (Sprint 42 Phase C, port of apps.py S8).
//!
//! Two endpoints:
//! - `GET /api/v1/apps` — list all known apps from the browse aggregator
//! - `GET /api/v1/apps/:id` — detail for a single app by project_id

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use nexus_shell_daemon_core::browse::BrowseEntry;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::http::DaemonHttpState;

#[derive(Debug, Serialize)]
pub struct AppSummary {
    pub project_id: String,
    pub project_name: String,
    pub category: String,
    pub description: String,
    pub is_open_source: bool,
    pub archive_hash: Option<String>,
    pub repo_url: Option<String>,
    pub has_provenance: bool,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct AppListResponse {
    pub apps: Vec<AppSummary>,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct AppDetailResponse {
    pub project_id: String,
    pub project_name: String,
    pub category: String,
    pub description: String,
    pub is_open_source: bool,
    pub archive_hash: Option<String>,
    pub archive_ticket: Option<String>,
    pub repo_url: Option<String>,
    pub provenance_hash: Option<String>,
    pub status: String,
    pub source: String,
    pub curator_pubkey: String,
    pub curator_name: String,
    pub last_probed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AppListQuery {
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub open_source: Option<bool>,
}

fn status_str(entry: &BrowseEntry) -> String {
    format!("{:?}", entry.status).to_lowercase()
}

fn source_str(entry: &BrowseEntry) -> String {
    format!("{:?}", entry.source).to_lowercase()
}

fn to_summary(entry: &BrowseEntry) -> AppSummary {
    AppSummary {
        project_id: entry.project_id.clone(),
        project_name: entry.project_name.clone(),
        category: entry.category.clone(),
        description: entry.description.clone(),
        is_open_source: entry.is_open_source,
        archive_hash: entry.archive_hash.clone(),
        repo_url: entry.repo_url.clone(),
        has_provenance: entry.provenance_hash.is_some(),
        status: status_str(entry),
        source: source_str(entry),
    }
}

fn to_detail(entry: &BrowseEntry) -> AppDetailResponse {
    AppDetailResponse {
        project_id: entry.project_id.clone(),
        project_name: entry.project_name.clone(),
        category: entry.category.clone(),
        description: entry.description.clone(),
        is_open_source: entry.is_open_source,
        archive_hash: entry.archive_hash.clone(),
        archive_ticket: entry.archive_ticket.clone(),
        repo_url: entry.repo_url.clone(),
        provenance_hash: entry.provenance_hash.clone(),
        status: status_str(entry),
        source: source_str(entry),
        curator_pubkey: entry.curator_pubkey.clone(),
        curator_name: entry.curator_name.clone(),
        last_probed_at: entry.last_probed_at.clone(),
    }
}

pub async fn list_apps(
    State(state): State<Arc<DaemonHttpState>>,
    Query(query): Query<AppListQuery>,
) -> impl IntoResponse {
    debug!("GET /api/v1/apps");
    let entries = state
        .browse_aggregator
        .aggregate(&state.curator_runtime, &state.node)
        .await;

    let mut apps: Vec<AppSummary> = entries
        .iter()
        .filter(|e| {
            if let Some(ref cat) = query.category {
                if !e.category.eq_ignore_ascii_case(cat) {
                    return false;
                }
            }
            if let Some(os) = query.open_source {
                if e.is_open_source != os {
                    return false;
                }
            }
            true
        })
        .map(to_summary)
        .collect();

    apps.dedup_by(|a, b| a.project_id == b.project_id);

    let count = apps.len();
    (StatusCode::OK, Json(AppListResponse { apps, count }))
}

pub async fn get_app(
    State(state): State<Arc<DaemonHttpState>>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    debug!(id = %project_id, "GET /api/v1/apps/:id");
    let entries = state
        .browse_aggregator
        .aggregate(&state.curator_runtime, &state.node)
        .await;

    match entries.iter().find(|e| e.project_id == project_id) {
        Some(entry) => (
            StatusCode::OK,
            Json(serde_json::to_value(to_detail(entry)).unwrap()),
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("app {} not found", project_id)})),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_shell_daemon_core::browse::{BrowseSource, BrowseStatus};

    fn make_entry(id: &str, name: &str, category: &str, open_source: bool) -> BrowseEntry {
        BrowseEntry {
            project_id: id.to_string(),
            project_name: name.to_string(),
            category: category.to_string(),
            description: format!("{name} description"),
            curator_pubkey: "aabbcc".to_string(),
            curator_name: "Test Curator".to_string(),
            source: BrowseSource::Direct,
            status: BrowseStatus::Reachable,
            last_probed_at: None,
            archive_ticket: None,
            archive_hash: Some("deadbeef".to_string()),
            repo_url: Some("https://github.com/test/repo".to_string()),
            provenance_hash: Some("cafebabe".to_string()),
            is_open_source: open_source,
        }
    }

    #[test]
    fn to_summary_maps_fields() {
        let entry = make_entry("proj-1", "My App", "gov", true);
        let summary = to_summary(&entry);
        assert_eq!(summary.project_id, "proj-1");
        assert_eq!(summary.project_name, "My App");
        assert_eq!(summary.category, "gov");
        assert!(summary.is_open_source);
        assert!(summary.has_provenance);
        assert_eq!(summary.status, "reachable");
        assert_eq!(summary.source, "direct");
        assert_eq!(summary.archive_hash.as_deref(), Some("deadbeef"));
        assert_eq!(
            summary.repo_url.as_deref(),
            Some("https://github.com/test/repo")
        );
    }

    #[test]
    fn to_summary_no_provenance() {
        let mut entry = make_entry("proj-2", "No Prov", "science", false);
        entry.provenance_hash = None;
        let summary = to_summary(&entry);
        assert!(!summary.has_provenance);
        assert!(!summary.is_open_source);
    }

    #[test]
    fn to_detail_maps_all_fields() {
        let entry = make_entry("proj-3", "Detail App", "tools", true);
        let detail = to_detail(&entry);
        assert_eq!(detail.project_id, "proj-3");
        assert_eq!(detail.project_name, "Detail App");
        assert_eq!(detail.category, "tools");
        assert!(detail.is_open_source);
        assert_eq!(detail.provenance_hash.as_deref(), Some("cafebabe"));
        assert_eq!(detail.curator_pubkey, "aabbcc");
        assert_eq!(detail.curator_name, "Test Curator");
        assert_eq!(detail.archive_ticket, None);
        assert_eq!(detail.last_probed_at, None);
    }

    #[test]
    fn status_str_formats_variants() {
        let mut entry = make_entry("p1", "A", "cat", false);
        assert_eq!(status_str(&entry), "reachable");
        entry.status = BrowseStatus::Unreachable;
        assert_eq!(status_str(&entry), "unreachable");
        entry.status = BrowseStatus::Unknown;
        assert_eq!(status_str(&entry), "unknown");
    }

    #[test]
    fn source_str_formats_variants() {
        let mut entry = make_entry("p1", "A", "cat", false);
        assert_eq!(source_str(&entry), "direct");
        entry.source = BrowseSource::Curator;
        assert_eq!(source_str(&entry), "curator");
    }

    #[test]
    fn to_detail_serializes_to_json() {
        let entry = make_entry("proj-4", "JSON App", "general", true);
        let detail = to_detail(&entry);
        let json = serde_json::to_value(&detail).unwrap();
        assert_eq!(json["project_id"], "proj-4");
        assert_eq!(json["project_name"], "JSON App");
        assert_eq!(json["is_open_source"], true);
        assert_eq!(json["status"], "reachable");
        assert_eq!(json["curator_name"], "Test Curator");
    }

    #[test]
    fn app_list_query_defaults() {
        let q: AppListQuery = serde_json::from_str("{}").unwrap();
        assert!(q.category.is_none());
        assert!(q.open_source.is_none());
    }

    #[test]
    fn app_list_query_with_filters() {
        let q: AppListQuery =
            serde_json::from_str(r#"{"category":"gov","open_source":true}"#).unwrap();
        assert_eq!(q.category.as_deref(), Some("gov"));
        assert_eq!(q.open_source, Some(true));
    }
}
