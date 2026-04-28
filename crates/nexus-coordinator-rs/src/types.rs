// SPDX-License-Identifier: AGPL-3.0-or-later
//! Coordinator-specific types for task lifecycle tracking.
//!
//! The wire-format types (`Task`, `TaskEntry`, `ResultEntry`) live
//! in `nexus_core_rs::task` — this module only adds the coordinator's
//! internal bookkeeping structs that track submission and validation
//! state in the local SQLite database.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Dispatched,
    Completed,
    Rejected,
    TimedOut,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Dispatched => "dispatched",
            Self::Completed => "completed",
            Self::Rejected => "rejected",
            Self::TimedOut => "timed_out",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "pending" => Self::Pending,
            "dispatched" => Self::Dispatched,
            "completed" => Self::Completed,
            "rejected" => Self::Rejected,
            "timed_out" => Self::TimedOut,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRecord {
    pub task_id: String,
    pub status: TaskStatus,
    pub project_id: String,
    pub model: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub task_hash: String,
    pub worker_node_id: Option<String>,
    pub result_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmission {
    pub project_id: String,
    pub task_type: String,
    pub prompt: String,
    #[serde(default)]
    pub system_prompt: String,
    pub model: String,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub parent_task_id: String,
    #[serde(default)]
    pub metadata: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub is_open_source: bool,
    #[serde(default)]
    pub estimated_watts: u32,
    #[serde(default)]
    pub estimated_vram_mb: u64,
    #[serde(default)]
    pub estimated_hours: f64,
    #[serde(default = "default_redundancy")]
    pub redundancy_factor: u8,
}

fn default_priority() -> u8 {
    5
}

fn default_redundancy() -> u8 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KudosEntry {
    pub entry_id: String,
    pub worker_node_id: String,
    pub task_id: String,
    pub project_id: String,
    pub amount: u64,
    pub created_at: u64,
    pub prev_hash: String,
    pub entry_hash: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_status_roundtrip() {
        for status in [
            TaskStatus::Pending,
            TaskStatus::Dispatched,
            TaskStatus::Completed,
            TaskStatus::Rejected,
            TaskStatus::TimedOut,
        ] {
            assert_eq!(TaskStatus::from_str_lossy(status.as_str()), status);
        }
    }

    #[test]
    fn task_submission_deserializes_with_defaults() {
        let json =
            r#"{"project_id":"p1","task_type":"analysis","prompt":"hello","model":"llama3"}"#;
        let sub: TaskSubmission = serde_json::from_str(json).expect("deserialize");
        assert_eq!(sub.priority, 5);
        assert_eq!(sub.redundancy_factor, 1);
        assert!(sub.system_prompt.is_empty());
        assert!(!sub.is_open_source);
    }

    #[test]
    fn task_status_unknown_falls_back_to_pending() {
        assert_eq!(TaskStatus::from_str_lossy("garbage"), TaskStatus::Pending);
    }
}
