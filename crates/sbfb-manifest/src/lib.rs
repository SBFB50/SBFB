// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB.json manifest parser, validator, and bridge method allowlist.
//!
//! Supports both v1 (node_id required, no schema_version) and v2
//! (schema_version: 2, node_id optional/deprecated). Pre-launch
//! redefinition — v1 remains parsable via `#[serde(default)]`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SbfbManifest {
    #[serde(default)]
    pub schema_version: Option<u32>,

    #[serde(default)]
    pub node_id: Option<String>,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub category: Option<String>,

    #[serde(default)]
    pub repo_url: Option<String>,

    #[serde(default)]
    pub bridge: Option<BridgeConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeConfig {
    #[serde(default)]
    pub methods: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("validation error: {0}")]
    Validation(String),
}

/// The set of bridge methods an app may DECLARE in its `SbfbManifest.bridge`.
///
/// This is a DECLARATIVE manifest-validation allowlist, NOT the runtime dispatch
/// boundary: the host shell's postMessage router (`web/src/bridge`) is what
/// actually dispatches a method and enforces the sandbox. Validating a manifest
/// against this list is a courtesy ("you declared a method the host does not
/// know"), never a sandbox-escape control — an app cannot reach a method by
/// declaring it, nor lose sandbox protection by omitting it.
///
/// Sprint 76 Phase B (B10, BRIDGE-ALLOWLIST-DRIFT): this list MUST mirror the
/// host dispatch schema `BridgeMethodSchema` in `web/src/bridge/protocol.ts`
/// (16 methods; S76-H added `task_result`). Pre-B10 it carried only 10, so a manifest declaring a genuinely
/// host-dispatched method (`pii_redact`, `storage_version`, `provenance_get`,
/// `provenance_verify`, `feed_cursor_get`) was wrongly rejected. The parity test
/// `allowlist_mirrors_host_dispatch_schema` locks the two sides together.
const BRIDGE_METHOD_ALLOWLIST: &[&str] = &[
    "task_submit",
    "storage_get",
    "storage_set",
    // Sprint 21 Phase B — host-dispatched locally (no coordinator round-trip).
    "pii_redact",
    // Sprint 56 Phase C — bridge extensions for pre-v1.0 apps.
    "storage_list",
    "storage_delete",
    "identity_pubkey",
    "node_status",
    "browse_list",
    // Sprint 58 Phase D — storage version polling for live updates.
    "storage_version",
    // Sprint 63 Phase C — verification bridge methods.
    "provenance_get",
    "provenance_verify",
    "feed_cursor_get",
    // Sprint 67 Phase B — FTS5 full-text search.
    "search",
    // Sprint 68 Phase A — ProofCard evidence score.
    "proof_card_get",
    // Sprint 76 Phase H — poll a completed compute task's result text.
    "task_result",
];

impl SbfbManifest {
    pub fn parse(json: &str) -> Result<Self, ManifestError> {
        Ok(serde_json::from_str(json)?)
    }

    pub fn effective_schema_version(&self) -> u32 {
        self.schema_version.unwrap_or(1)
    }

    pub fn is_v2(&self) -> bool {
        self.effective_schema_version() >= 2
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.is_v2() && self.name.as_ref().is_none_or(|n| n.is_empty()) {
            return Err(ManifestError::Validation(
                "v2 manifest requires a non-empty 'name' field".into(),
            ));
        }

        if let Some(ref bridge) = self.bridge {
            for method in &bridge.methods {
                if !BRIDGE_METHOD_ALLOWLIST.contains(&method.as_str()) {
                    return Err(ManifestError::Validation(format!(
                        "bridge method '{}' is not in the allowlist",
                        method
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn bridge_method_allowlist() -> &'static [&'static str] {
        BRIDGE_METHOD_ALLOWLIST
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sbfb_manifest_parse_v1() {
        let json = r#"{"node_id": "abc123", "name": "test-app", "version": "1.0.0"}"#;
        let m = SbfbManifest::parse(json).unwrap();
        assert_eq!(m.effective_schema_version(), 1);
        assert!(!m.is_v2());
        assert_eq!(m.node_id.as_deref(), Some("abc123"));
        assert_eq!(m.name.as_deref(), Some("test-app"));
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_sbfb_manifest_parse_v2() {
        let json = r#"{
            "schema_version": 2,
            "name": "my-app",
            "version": "2.0.0",
            "description": "A cool app",
            "category": "tools",
            "repo_url": "https://github.com/org/app",
            "bridge": { "methods": ["task_submit", "storage_get"] }
        }"#;
        let m = SbfbManifest::parse(json).unwrap();
        assert_eq!(m.effective_schema_version(), 2);
        assert!(m.is_v2());
        assert!(m.node_id.is_none());
        assert_eq!(m.name.as_deref(), Some("my-app"));
        assert_eq!(m.description.as_deref(), Some("A cool app"));
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_sbfb_manifest_validate_v2_rejects_missing_name() {
        let json = r#"{"schema_version": 2}"#;
        let m = SbfbManifest::parse(json).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn test_sbfb_manifest_validate_bridge_allowlist() {
        let json = r#"{
            "schema_version": 2,
            "name": "test",
            "bridge": { "methods": ["task_submit", "evil_method"] }
        }"#;
        let m = SbfbManifest::parse(json).unwrap();
        let err = m.validate().unwrap_err();
        assert!(err.to_string().contains("evil_method"));
        assert!(err.to_string().contains("allowlist"));
    }

    /// Sprint 76 Phase B (B10, BRIDGE-ALLOWLIST-DRIFT): the declarative manifest
    /// allowlist MUST mirror the host dispatch schema `BridgeMethodSchema` in
    /// `web/src/bridge/protocol.ts`. This `EXPECTED` list is the canonical mirror
    /// of that TS enum (16 methods); if either side adds/removes a method, this
    /// test fails until both are updated. The allowlist is DECLARATIVE manifest
    /// validation, not the sandbox/dispatch boundary — a method present here is a
    /// method an app may legitimately DECLARE, never a sandbox escape.
    #[test]
    fn allowlist_mirrors_host_dispatch_schema() {
        // Mirror of web/src/bridge/protocol.ts BridgeMethodSchema (keep in sync).
        const EXPECTED: &[&str] = &[
            "task_submit",
            "storage_get",
            "storage_set",
            "pii_redact",
            "storage_list",
            "storage_delete",
            "identity_pubkey",
            "node_status",
            "browse_list",
            "storage_version",
            "provenance_get",
            "provenance_verify",
            "feed_cursor_get",
            "search",
            "proof_card_get",
            "task_result",
        ];
        let actual: std::collections::BTreeSet<&str> =
            BRIDGE_METHOD_ALLOWLIST.iter().copied().collect();
        let expected: std::collections::BTreeSet<&str> = EXPECTED.iter().copied().collect();
        assert_eq!(
            actual, expected,
            "the Rust manifest allowlist must mirror the TS BridgeMethodSchema (protocol.ts) — \
             update BOTH sides when a host-dispatched method is added or removed"
        );
        // No duplicates crept into the production constant.
        assert_eq!(
            BRIDGE_METHOD_ALLOWLIST.len(),
            EXPECTED.len(),
            "BRIDGE_METHOD_ALLOWLIST must have no duplicate entries"
        );

        // The 5 methods added in B10 are now declarable (were wrongly rejected).
        for m in [
            "pii_redact",
            "storage_version",
            "provenance_get",
            "provenance_verify",
            "feed_cursor_get",
        ] {
            let json =
                format!(r#"{{"schema_version":2,"name":"t","bridge":{{"methods":["{m}"]}}}}"#);
            assert!(
                SbfbManifest::parse(&json).unwrap().validate().is_ok(),
                "host-dispatched method {m} must be declarable in a manifest"
            );
        }
    }
}
