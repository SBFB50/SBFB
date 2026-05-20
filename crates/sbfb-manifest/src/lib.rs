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

const BRIDGE_METHOD_ALLOWLIST: &[&str] = &[
    "task_submit",
    "storage_get",
    "storage_set",
    "storage_list",
    "storage_delete",
    "identity_pubkey",
    "node_status",
    "browse_list",
    "search",
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
}
