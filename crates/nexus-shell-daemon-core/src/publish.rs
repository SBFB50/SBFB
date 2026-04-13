// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 11 Phase A — gossip-based project announcement.
//!
//! When a coordinator with `visibility=public` starts, it tells its
//! local shell daemon to broadcast a [`ProjectAnnouncement`] on the
//! curator gossip topic. Other daemons receive the announcement and
//! add the project to their [`crate::browse::BrowseAggregator`] as
//! a "direct" entry — no curator intermediary needed.
//!
//! The announcement reuses the same gossip topic as curator lists
//! (`nexus-grid/curator/v1`) but carries `"type": "project"` so
//! receivers can dispatch on the discriminator before parsing the
//! full payload.

use serde::{Deserialize, Serialize};

/// Wire format version for project announcements.
/// v2 adds `archive_ticket` (Sprint 12 Phase A).
/// v3 adds `repo_url` (Sprint 13 Phase B).
/// v4 adds `provenance_hash` (Sprint 14 Phase B).
pub const PROJECT_ANNOUNCEMENT_VERSION: u32 = 4;

/// The JSON payload broadcast on the curator gossip topic to
/// announce a project directly (without a curator intermediary).
///
/// Discriminated from curator announcements by the `msg_type`
/// field which is always `"project"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectAnnouncement {
    /// Wire format version. Always 1 for Sprint 11.
    pub v: u32,
    /// Message type discriminator. Always `"project"`.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Hex node_id of the announcing daemon.
    pub node_id: String,
    /// Project display name.
    pub project_name: String,
    /// Category tag.
    pub category: String,
    /// Short description.
    pub description: String,
    /// List of app names available on this project.
    #[serde(default)]
    pub apps: Vec<String>,
    /// BlobTicket of the zip archive for this project (Sprint 12).
    /// `None` for v1 announcements from older daemons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_ticket: Option<String>,
    /// URL of the public source code repository (Sprint 13 Phase B).
    /// Required for public projects, optional for private.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    /// BLAKE3 hex hash of the provenance.json attestation (Sprint 14).
    /// Present when the app was deployed via `deploy-from-repo` with
    /// a signed provenance record. `None` for legacy deploys or
    /// private apps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_hash: Option<String>,
}

/// Error validating a project announcement.
#[derive(Debug, thiserror::Error)]
pub enum ProjectAnnouncementError {
    /// JSON deserialization failed.
    #[error("JSON parse failed: {0}")]
    Parse(#[from] serde_json::Error),
    /// Unknown wire format version.
    #[error("unknown version {got} (expected {expected})")]
    Version { got: u32, expected: u32 },
    /// The `type` field is not `"project"`.
    #[error("wrong message type: {0} (expected \"project\")")]
    WrongType(String),
    /// The `node_id` field is not a valid 64-char lowercase hex string.
    #[error("invalid node_id: expected 64 hex chars, got {0:?}")]
    InvalidNodeId(String),
}

impl ProjectAnnouncement {
    /// Construct a new announcement at the current version.
    pub fn new(
        node_id: String,
        project_name: String,
        category: String,
        description: String,
        apps: Vec<String>,
    ) -> Self {
        Self {
            v: PROJECT_ANNOUNCEMENT_VERSION,
            msg_type: "project".to_string(),
            node_id,
            project_name,
            category,
            description,
            apps,
            archive_ticket: None,
            repo_url: None,
            provenance_hash: None,
        }
    }

    /// Construct a v2 announcement with an archive ticket.
    pub fn with_archive_ticket(mut self, ticket: String) -> Self {
        self.archive_ticket = Some(ticket);
        self
    }

    /// Set the repo_url (v3, Sprint 13 Phase B).
    pub fn with_repo_url(mut self, url: String) -> Self {
        self.repo_url = Some(url);
        self
    }

    /// Set the provenance_hash (v4, Sprint 14 Phase B).
    pub fn with_provenance_hash(mut self, hash: String) -> Self {
        self.provenance_hash = Some(hash);
        self
    }

    /// Serialize to JSON bytes for gossip broadcast.
    pub fn to_gossip_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Parse and validate a project announcement from gossip bytes.
    ///
    /// Accepts v1 through v4 for backward compatibility with older daemons.
    pub fn from_gossip_bytes(bytes: &[u8]) -> Result<Self, ProjectAnnouncementError> {
        let ann: Self = serde_json::from_slice(bytes)?;
        // Accept v1, v2, v3, and v4 for backward compatibility.
        if ann.v == 0 || ann.v > PROJECT_ANNOUNCEMENT_VERSION {
            return Err(ProjectAnnouncementError::Version {
                got: ann.v,
                expected: PROJECT_ANNOUNCEMENT_VERSION,
            });
        }
        if ann.msg_type != "project" {
            return Err(ProjectAnnouncementError::WrongType(ann.msg_type));
        }
        // T28: validate node_id is a 64-char lowercase hex string.
        if ann.node_id.len() != 64 || !ann.node_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ProjectAnnouncementError::InvalidNodeId(ann.node_id));
        }
        Ok(ann)
    }
}

/// Check if raw gossip bytes look like a project announcement
/// (as opposed to a curator announcement). Does a cheap partial
/// parse checking for `"type": "project"`.
pub fn is_project_announcement(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    value.get("type").and_then(|v| v.as_str()) == Some("project")
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn announcement_round_trips_through_json() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov-officiel".into(),
            "gov".into(),
            "Gouvernement app".into(),
            vec!["gov".into(), "coldcase".into()],
        );
        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
    }

    #[test]
    fn announcement_rejects_wrong_version() {
        let mut ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        ann.v = 99;
        let bytes = serde_json::to_vec(&ann).unwrap();
        let err = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap_err();
        assert!(matches!(
            err,
            ProjectAnnouncementError::Version {
                got: 99,
                expected: 4
            }
        ));
    }

    #[test]
    fn announcement_rejects_wrong_type() {
        let mut ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        ann.msg_type = "curator".into();
        let bytes = serde_json::to_vec(&ann).unwrap();
        let err = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ProjectAnnouncementError::WrongType(_)));
    }

    #[test]
    fn is_project_announcement_detects_project_type() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        let bytes = ann.to_gossip_bytes().unwrap();
        assert!(is_project_announcement(&bytes));
    }

    #[test]
    fn is_project_announcement_rejects_curator_message() {
        let msg = serde_json::json!({"v": 1, "curator": "abc", "ticket": "xyz"});
        let bytes = serde_json::to_vec(&msg).unwrap();
        assert!(!is_project_announcement(&bytes));
    }

    #[test]
    fn is_project_announcement_rejects_garbage() {
        assert!(!is_project_announcement(b"not json"));
    }

    // ---------------------------------------------------------
    // Sprint 12 Phase A — v2 with archive_ticket
    // ---------------------------------------------------------

    #[test]
    fn v2_announcement_with_archive_ticket_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_archive_ticket("blobticket123abc".into());

        assert_eq!(ann.v, PROJECT_ANNOUNCEMENT_VERSION);
        assert_eq!(ann.archive_ticket.as_deref(), Some("blobticket123abc"));

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.archive_ticket.as_deref(), Some("blobticket123abc"));
    }

    #[test]
    fn v1_announcement_parses_without_archive_ticket() {
        // Simulate a v1 announcement from an older daemon.
        let json = serde_json::json!({
            "v": 1,
            "type": "project",
            "node_id": "a".repeat(64),
            "project_name": "test",
            "category": "misc",
            "description": "old daemon",
            "apps": []
        });
        let bytes = serde_json::to_vec(&json).unwrap();
        let ann = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(ann.v, 1);
        assert!(ann.archive_ticket.is_none());
    }

    #[test]
    fn v2_announcement_without_archive_ticket_omits_field() {
        // v2 with no archive_ticket should not serialize the field.
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        let json_str = serde_json::to_string(&ann).unwrap();
        assert!(
            !json_str.contains("archive_ticket"),
            "None archive_ticket should be omitted from JSON"
        );
    }

    // ---------------------------------------------------------
    // T28 — node_id validation
    // ---------------------------------------------------------

    #[test]
    fn rejects_empty_node_id() {
        let json = serde_json::json!({
            "v": 2,
            "type": "project",
            "node_id": "",
            "project_name": "test",
            "category": "misc",
            "description": "bad",
            "apps": []
        });
        let bytes = serde_json::to_vec(&json).unwrap();
        let err = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap_err();
        assert!(matches!(err, ProjectAnnouncementError::InvalidNodeId(_)));
    }

    #[test]
    fn rejects_short_node_id() {
        let json = serde_json::json!({
            "v": 2,
            "type": "project",
            "node_id": "abcd",
            "project_name": "test",
            "category": "misc",
            "description": "short",
            "apps": []
        });
        let bytes = serde_json::to_vec(&json).unwrap();
        assert!(ProjectAnnouncement::from_gossip_bytes(&bytes).is_err());
    }

    // ---------------------------------------------------------
    // T29 — truncated gossip message
    // ---------------------------------------------------------

    #[test]
    fn is_project_announcement_rejects_truncated_json() {
        assert!(!is_project_announcement(b"{\"type\": \"project\""));
    }

    #[test]
    fn from_gossip_bytes_rejects_truncated_json() {
        let err = ProjectAnnouncement::from_gossip_bytes(b"{\"v\": 2, \"type\":").unwrap_err();
        assert!(matches!(err, ProjectAnnouncementError::Parse(_)));
    }

    // ---------------------------------------------------------
    // Sprint 13 Phase B — v3 with repo_url
    // ---------------------------------------------------------

    #[test]
    fn v3_announcement_with_repo_url_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_repo_url("https://github.com/example/gov".into());

        assert_eq!(ann.v, PROJECT_ANNOUNCEMENT_VERSION);
        assert_eq!(
            ann.repo_url.as_deref(),
            Some("https://github.com/example/gov")
        );

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(
            back.repo_url.as_deref(),
            Some("https://github.com/example/gov")
        );
    }

    #[test]
    fn v2_announcement_parses_without_repo_url() {
        // Backward compat: v2 announcements from older daemons
        // must parse correctly, with repo_url defaulting to None.
        let json = serde_json::json!({
            "v": 2,
            "type": "project",
            "node_id": "a".repeat(64),
            "project_name": "test",
            "category": "misc",
            "description": "old daemon",
            "apps": [],
            "archive_ticket": "blobticket_abc"
        });
        let bytes = serde_json::to_vec(&json).unwrap();
        let ann = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(ann.v, 2);
        assert!(ann.repo_url.is_none());
        assert!(ann.archive_ticket.is_some());
    }

    #[test]
    fn v3_announcement_without_repo_url_omits_field() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        let json_str = serde_json::to_string(&ann).unwrap();
        assert!(
            !json_str.contains("repo_url"),
            "None repo_url should be omitted from JSON"
        );
    }

    // ---------------------------------------------------------
    // Sprint 14 Phase B — v4 with provenance_hash
    // ---------------------------------------------------------

    #[test]
    fn v4_announcement_with_provenance_hash_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_repo_url("https://github.com/test/gov".into())
        .with_provenance_hash("bb".repeat(32));

        assert_eq!(ann.v, PROJECT_ANNOUNCEMENT_VERSION);
        assert_eq!(ann.provenance_hash.as_deref(), Some(&*"bb".repeat(32)));

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.provenance_hash.as_deref(), Some(&*"bb".repeat(32)));
    }

    #[test]
    fn v3_announcement_parses_without_provenance_hash() {
        // Backward compat: v3 announcements from older daemons
        // must parse correctly, with provenance_hash defaulting to None.
        let json = serde_json::json!({
            "v": 3,
            "type": "project",
            "node_id": "a".repeat(64),
            "project_name": "test",
            "category": "misc",
            "description": "old daemon",
            "apps": [],
            "repo_url": "https://github.com/test/old"
        });
        let bytes = serde_json::to_vec(&json).unwrap();
        let ann = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(ann.v, 3);
        assert!(ann.provenance_hash.is_none());
        assert!(ann.repo_url.is_some());
    }

    #[test]
    fn v4_announcement_without_provenance_hash_omits_field() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        let json_str = serde_json::to_string(&ann).unwrap();
        assert!(
            !json_str.contains("provenance_hash"),
            "None provenance_hash should be omitted from JSON"
        );
    }
}
