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
///
/// Pre-launch policy (cf. `CLAUDE.md` §Pre-launch protocol
/// policy): the project has no live network, so the current
/// canonical schema is the first one that will ever ship. The
/// constant stays at 1 until the `v1.0` tag freezes the format ;
/// every break after that bumps the version.
pub const PROJECT_ANNOUNCEMENT_VERSION: u32 = 1;

/// The JSON payload broadcast on the curator gossip topic to
/// announce a project directly (without a curator intermediary).
///
/// Discriminated from curator announcements by the `msg_type`
/// field which is always `"project"`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectAnnouncement {
    /// Wire format version. Must equal
    /// [`PROJECT_ANNOUNCEMENT_VERSION`].
    pub v: u32,
    /// Message type discriminator. Always `"project"`.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Hex node_id of the announcing daemon.
    pub node_id: String,
    /// Per-app project identity: `blake3(project_name)` hex (64 chars).
    ///
    /// Distinct from `node_id` (the hosting daemon): one node can host many
    /// apps, each with its own `project_id`. Keying browse entries on this
    /// instead of `node_id` is what lets a remote viewer see N distinct cards
    /// for an N-app node (and resolve detail/proof-card/provenance consistently
    /// cross-node). `#[serde(default)]` keeps decode tolerant of a legacy
    /// announcement that predates this field — an empty value makes the receiver
    /// fall back to `node_id`. Same id the feed and deploy already use.
    #[serde(default)]
    pub project_id: String,
    /// Project display name.
    pub project_name: String,
    /// Category tag.
    pub category: String,
    /// Short description.
    pub description: String,
    /// List of app names available on this project.
    #[serde(default)]
    pub apps: Vec<String>,
    /// BlobTicket of the zip archive. `None` when the project
    /// has not uploaded a blob yet (rare on public projects).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_ticket: Option<String>,
    /// URL of the public source code repository. Required for
    /// public projects, optional for private (`visibility=private`
    /// zip uploads).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    /// BLAKE3 hex hash of the provenance.json attestation. Present
    /// only when the project was deployed via `deploy-from-repo`
    /// with a signed provenance record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance_hash: Option<String>,
    /// True iff the coordinator deployed this project via
    /// `deploy-from-repo` (clone + SBFB.json + signed provenance).
    /// Derived by the coordinator at publish time, never
    /// user-settable (enforced daemon-side: `POST /publish`
    /// refuses `is_open_source=true` without the full provenance
    /// chain). Workers at consent level L2 (OpenSource) accept
    /// tasks only from projects that carry this flag.
    ///
    /// `#[serde(default)]` keeps decode robust against a minimal
    /// JSON body — runtime tolerance, not historical compat.
    #[serde(default)]
    pub is_open_source: bool,
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
    /// The `project_id` field is present but not a valid 64-char hex string.
    #[error("invalid project_id: expected empty or 64 hex chars, got {0:?}")]
    InvalidProjectId(String),
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
            project_id: String::new(),
            project_name,
            category,
            description,
            apps,
            archive_ticket: None,
            repo_url: None,
            provenance_hash: None,
            is_open_source: false,
        }
    }

    /// Attach the per-app `project_id` (`blake3(project_name)` hex).
    pub fn with_project_id(mut self, project_id: String) -> Self {
        self.project_id = project_id;
        self
    }

    /// Attach a BlobTicket to the announcement.
    pub fn with_archive_ticket(mut self, ticket: String) -> Self {
        self.archive_ticket = Some(ticket);
        self
    }

    /// Attach the public repo URL.
    pub fn with_repo_url(mut self, url: String) -> Self {
        self.repo_url = Some(url);
        self
    }

    /// Attach the provenance BLAKE3 hex hash.
    pub fn with_provenance_hash(mut self, hash: String) -> Self {
        self.provenance_hash = Some(hash);
        self
    }

    /// Mark the project as open source.
    ///
    /// The coordinator calls this for every `deploy-from-repo`
    /// publish — the clone + SBFB.json + signed provenance chain
    /// already establishes that the code on the network matches
    /// the public repo. Private zip uploads leave the flag at
    /// its `false` default.
    pub fn with_open_source(mut self, is_open_source: bool) -> Self {
        self.is_open_source = is_open_source;
        self
    }

    /// Serialize to JSON bytes for gossip broadcast.
    pub fn to_gossip_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Parse and validate a project announcement from gossip bytes.
    ///
    /// Only the current `v == PROJECT_ANNOUNCEMENT_VERSION` is
    /// accepted — pre-launch policy, see the constant's doc. Once
    /// the format is frozen at `v1.0`, this is where a tolerant
    /// accept range will live (e.g. `v == 1 || v == 2`).
    pub fn from_gossip_bytes(bytes: &[u8]) -> Result<Self, ProjectAnnouncementError> {
        let ann: Self = serde_json::from_slice(bytes)?;
        if ann.v != PROJECT_ANNOUNCEMENT_VERSION {
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
        // Per-app project_id: empty is tolerated (legacy announcement predating
        // the field), but a present value must be a 64-char hex string (blake3).
        if !ann.project_id.is_empty()
            && (ann.project_id.len() != 64
                || !ann.project_id.chars().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(ProjectAnnouncementError::InvalidProjectId(ann.project_id));
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

/// Check if raw gossip bytes are a browse pull request.
pub fn is_browse_request(bytes: &[u8]) -> bool {
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(bytes) else {
        return false;
    };
    value.get("type").and_then(|v| v.as_str()) == Some("browse_request")
}

/// Encode a browse pull request as gossip bytes.
pub fn browse_request_bytes() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"type": "browse_request"})).expect("static json")
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_id_round_trips_through_gossip_bytes() {
        let pid = "ab".repeat(32); // 64 hex
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "Name".into(),
            "cat".into(),
            "desc".into(),
            vec![],
        )
        .with_project_id(pid.clone());
        let bytes = ann.to_gossip_bytes().expect("encode");
        let decoded = ProjectAnnouncement::from_gossip_bytes(&bytes).expect("decode");
        assert_eq!(decoded.project_id, pid);
        assert_ne!(
            decoded.project_id, decoded.node_id,
            "per-app project_id is distinct from the hosting node_id"
        );
    }

    #[test]
    fn empty_project_id_is_tolerated_as_legacy() {
        // An announcement built without with_project_id() decodes fine (empty).
        let ann = ProjectAnnouncement::new(
            "b".repeat(64),
            "Name".into(),
            "cat".into(),
            "desc".into(),
            vec![],
        );
        let bytes = ann.to_gossip_bytes().expect("encode");
        assert_eq!(
            ProjectAnnouncement::from_gossip_bytes(&bytes)
                .expect("decode")
                .project_id,
            ""
        );
        // A JSON missing the field entirely also decodes (serde default).
        let json = serde_json::json!({
            "v": PROJECT_ANNOUNCEMENT_VERSION, "type": "project",
            "node_id": "c".repeat(64), "project_name": "X",
            "category": "c", "description": "d"
        });
        assert_eq!(
            ProjectAnnouncement::from_gossip_bytes(json.to_string().as_bytes())
                .expect("decode legacy")
                .project_id,
            ""
        );
    }

    #[test]
    fn malformed_project_id_is_rejected() {
        for bad in [
            "xyz".to_string(),
            "a".repeat(63),
            "a".repeat(65),
            "g".repeat(64),
        ] {
            let json = serde_json::json!({
                "v": PROJECT_ANNOUNCEMENT_VERSION, "type": "project",
                "node_id": "d".repeat(64), "project_id": bad,
                "project_name": "X", "category": "c", "description": "d"
            });
            assert!(
                matches!(
                    ProjectAnnouncement::from_gossip_bytes(json.to_string().as_bytes()),
                    Err(ProjectAnnouncementError::InvalidProjectId(_))
                ),
                "malformed project_id {bad:?} must be rejected"
            );
        }
    }

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
                expected: 1
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
    // Field round trips
    // ---------------------------------------------------------

    #[test]
    fn announcement_with_archive_ticket_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_archive_ticket("blobticket123abc".into());

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.archive_ticket.as_deref(), Some("blobticket123abc"));
    }

    #[test]
    fn announcement_without_archive_ticket_omits_field() {
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

    #[test]
    fn announcement_with_repo_url_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_repo_url("https://github.com/example/gov".into());

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(
            back.repo_url.as_deref(),
            Some("https://github.com/example/gov")
        );
    }

    #[test]
    fn announcement_without_repo_url_omits_field() {
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

    #[test]
    fn announcement_with_provenance_hash_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_repo_url("https://github.com/test/gov".into())
        .with_provenance_hash("bb".repeat(32));

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert_eq!(back.provenance_hash.as_deref(), Some(&*"bb".repeat(32)));
    }

    #[test]
    fn announcement_without_provenance_hash_omits_field() {
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

    // ---------------------------------------------------------
    // is_open_source
    // ---------------------------------------------------------

    #[test]
    fn announcement_with_open_source_true_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "gov".into(),
            "gov".into(),
            "desc".into(),
            vec!["gov".into()],
        )
        .with_repo_url("https://github.com/test/gov".into())
        .with_provenance_hash("bb".repeat(32))
        .with_open_source(true);

        assert!(ann.is_open_source);

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert!(back.is_open_source);
    }

    #[test]
    fn announcement_with_open_source_false_round_trips() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "private".into(),
            "misc".into(),
            "desc".into(),
            vec![],
        )
        .with_open_source(false);

        assert!(!ann.is_open_source);

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
        assert!(!back.is_open_source);
    }

    #[test]
    fn announcement_always_serializes_is_open_source_field() {
        // Unlike Option<_> fields, the bool is always serialized so
        // every peer sees the explicit value (true or false).
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "test".into(),
            "misc".into(),
            "test".into(),
            vec![],
        );
        let json_str = serde_json::to_string(&ann).unwrap();
        assert!(
            json_str.contains("\"is_open_source\":false"),
            "is_open_source=false must be present in JSON, got: {json_str}"
        );

        let ann_true = ann.clone().with_open_source(true);
        let json_str_true = serde_json::to_string(&ann_true).unwrap();
        assert!(
            json_str_true.contains("\"is_open_source\":true"),
            "is_open_source=true must be present in JSON, got: {json_str_true}"
        );
    }

    #[test]
    fn builders_compose_without_losing_state() {
        let ann = ProjectAnnouncement::new(
            "a".repeat(64),
            "compose".into(),
            "gov".into(),
            "stacked".into(),
            vec!["gov".into()],
        )
        .with_open_source(true)
        .with_archive_ticket("ticket_abc".into())
        .with_repo_url("https://github.com/test/compose".into())
        .with_provenance_hash("dd".repeat(32));

        assert!(ann.is_open_source);
        assert_eq!(ann.archive_ticket.as_deref(), Some("ticket_abc"));
        assert_eq!(
            ann.repo_url.as_deref(),
            Some("https://github.com/test/compose")
        );
        assert_eq!(ann.provenance_hash.as_deref(), Some(&*"dd".repeat(32)));

        let bytes = ann.to_gossip_bytes().unwrap();
        let back = ProjectAnnouncement::from_gossip_bytes(&bytes).unwrap();
        assert_eq!(back, ann);
    }

    // ---------------------------------------------------------
    // T28 — node_id validation
    // ---------------------------------------------------------

    #[test]
    fn rejects_empty_node_id() {
        let json = serde_json::json!({
            "v": 1,
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
            "v": 1,
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
        let err = ProjectAnnouncement::from_gossip_bytes(b"{\"v\": 1, \"type\":").unwrap_err();
        assert!(matches!(err, ProjectAnnouncementError::Parse(_)));
    }

    #[test]
    fn is_browse_request_accepts_valid() {
        let bytes = browse_request_bytes();
        assert!(is_browse_request(&bytes));
        assert!(!is_project_announcement(&bytes));
    }

    #[test]
    fn is_browse_request_rejects_project() {
        let kp = nexus_core_rs::KeyPair::generate();
        let node_id = hex::encode(kp.public_bytes());
        let ann = ProjectAnnouncement::new(node_id, "p".into(), "c".into(), "d".into(), vec![]);
        assert!(!is_browse_request(&ann.to_gossip_bytes().unwrap()));
    }

    #[test]
    fn is_browse_request_rejects_garbage() {
        assert!(!is_browse_request(b"not json"));
    }
}
