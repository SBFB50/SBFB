// SPDX-License-Identifier: AGPL-3.0-or-later
//! SBFB.json manifest parser, validator, and bridge method allowlist.
//!
//! Supports both v1 (node_id required, no schema_version) and v2
//! (schema_version: 2, node_id optional/deprecated). Pre-launch
//! redefinition — v1 remains parsable via `#[serde(default)]`.

use serde::{Deserialize, Serialize};

pub mod grant_v0;

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
/// Two host-side roles:
///  1. Manifest validation (since S13): the universe of method names a release may
///     legitimately name in `bridge.methods`. An off-list name is rejected.
///  2. Release-declaration layer of the host authority intersection (Sprint 83
///     Phase C): when a manifest declares a `bridge` config, that declared set is
///     the FAIL-CLOSED release layer of `declared ∩ grant ∩ policy ∩ consent`,
///     evaluated per call by the shell host (`web/src/bridge`) and modelled in
///     [`grant_v0`]. Declaring a method is still NOT sufficient to reach it (the
///     grant/policy/consent layers must also admit it); OMITTING the `bridge` key
///     keeps a legacy app grandfathered (no release constraint). The allowlist never
///     grants a sandbox escape — the CSP iframe sandbox is orthogonal.
///
/// Sprint 76 Phase B (B10, BRIDGE-ALLOWLIST-DRIFT): this list MUST mirror the host
/// dispatch schema `BridgeMethodSchema` in `web/src/bridge/protocol.ts` (26 methods;
/// S76-H added `task_result`, chat v1 the 10 `chat_*` verbs). Pre-B10 it carried only 10, so a manifest declaring a
/// genuinely host-dispatched method (`pii_redact`, `storage_version`, `provenance_get`,
/// `provenance_verify`, `feed_cursor_get`) was wrongly rejected. As of S83 Phase C the
/// parity is a SECURITY invariant (a drift lets a release declare a method the host
/// will not dispatch, or the host dispatch a method no release can name): the unit
/// mirror `allowlist_mirrors_host_dispatch_schema` AND the mechanical gate
/// `scripts/check-frontier-contracts.sh` (bridge-parity block) lock the two sides.
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
    // Chat v1 (2026-10-21, ADR_CHAT_V1) — the SENSITIVE class (see
    // `SENSITIVE_BRIDGE_METHODS`): never grandfathered, consent-gated.
    "chat_identity",
    "chat_set_profile",
    "chat_list",
    "chat_create",
    "chat_invite",
    "chat_accept",
    "chat_send",
    "chat_messages",
    "chat_remove",
    "chat_mailbox",
];

/// Chat v1 sensitive class: methods a release may DECLARE but that are never part
/// of the grandfather grant and are granted only after explicit user consent
/// (host side: `web/src/bridge/authority.ts` `SENSITIVE_METHODS`). Reaching them
/// lets an app make the node join conversations and hand out write access.
pub const SENSITIVE_BRIDGE_METHODS: &[&str] = &[
    "chat_identity",
    "chat_set_profile",
    "chat_list",
    "chat_create",
    "chat_invite",
    "chat_accept",
    "chat_send",
    "chat_messages",
    "chat_remove",
    "chat_mailbox",
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
    /// of that TS enum (26 methods); if either side adds/removes a method, this
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
            "chat_identity",
            "chat_set_profile",
            "chat_list",
            "chat_create",
            "chat_invite",
            "chat_accept",
            "chat_send",
            "chat_messages",
            "chat_remove",
            "chat_mailbox",
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

    /// Sprint 83 Phase C: a manifest's declared `bridge.methods` compose with a host
    /// grant as the release-declaration layer of the authority intersection — the
    /// effective set keeps only what BOTH the release declares AND the grant confers.
    #[test]
    fn bridge_methods_intersect_grants() {
        use crate::grant_v0::{Grant, effective_methods};
        use std::collections::BTreeSet;

        let json = r#"{
            "schema_version": 2,
            "name": "hub",
            "bridge": { "methods": ["storage_get", "storage_set", "search"] }
        }"#;
        let m = SbfbManifest::parse(json).unwrap();
        m.validate().unwrap();
        let declared: Option<BTreeSet<String>> = m
            .bridge
            .as_ref()
            .map(|b| b.methods.iter().cloned().collect());
        assert!(
            declared.is_some(),
            "a declared bridge config is the fail-closed release layer"
        );

        // Operator grant covers a subset (storage_set, search) plus a method the
        // release never declared (task_submit): the intersection drops both the
        // undeclared method and the un-granted one.
        let grant = Grant {
            methods: ["storage_set", "search", "task_submit"]
                .iter()
                .map(|x| x.to_string())
                .collect(),
            expires_at_ms: None,
            revoked: false,
        };
        let eff = effective_methods(&declared, &grant, &None, 0);
        let expected: BTreeSet<String> = ["search", "storage_set"]
            .iter()
            .map(|x| x.to_string())
            .collect();
        assert_eq!(eff, expected);

        // No bridge key -> declared = None -> grandfathered legacy (no release layer).
        let legacy = SbfbManifest::parse(r#"{"schema_version":2,"name":"legacy"}"#).unwrap();
        assert!(legacy.bridge.is_none());
    }
}
