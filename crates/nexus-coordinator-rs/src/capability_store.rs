// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability toggles gate-off-by-default store
//! (Sprint 41 Phase B, port of capability_store.py S25).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const KNOWN_CAPABILITIES: &[&str] = &[
    "biometric_gate",
    "federation_canary",
    "mcp_server_expose",
    "rag_retrieval",
    "streaming_bridge",
    "tool_calling",
];

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityEntry {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub enabled_at: String,
    #[serde(default)]
    pub enabled_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilityFileData {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    integrity_hash: String,
    #[serde(default)]
    capability: BTreeMap<String, CapabilityEntry>,
}

fn default_version() -> u32 {
    1
}

pub struct CapabilityStore {
    path: PathBuf,
    capabilities: BTreeMap<String, CapabilityEntry>,
}

impl CapabilityStore {
    pub fn load(path: &Path) -> Self {
        if !path.exists() {
            let store = Self::all_off(path);
            let _ = store.write();
            return store;
        }
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => return Self::all_off(path),
        };
        let data: CapabilityFileData = match toml::from_str(&text) {
            Ok(d) => d,
            Err(_) => return Self::all_off(path),
        };
        let computed = compute_integrity_hash(&text);
        if !data.integrity_hash.is_empty() && data.integrity_hash != computed {
            return Self::all_off(path);
        }
        let mut caps = BTreeMap::new();
        for &name in KNOWN_CAPABILITIES {
            let entry = data.capability.get(name).cloned().unwrap_or_default();
            caps.insert(name.to_string(), entry);
        }
        Self {
            path: path.to_path_buf(),
            capabilities: caps,
        }
    }

    pub fn all_off(path: &Path) -> Self {
        let mut caps = BTreeMap::new();
        for &name in KNOWN_CAPABILITIES {
            caps.insert(name.to_string(), CapabilityEntry::default());
        }
        Self {
            path: path.to_path_buf(),
            capabilities: caps,
        }
    }

    pub fn is_enabled(&self, cap_name: &str) -> bool {
        self.capabilities.get(cap_name).is_some_and(|e| e.enabled)
    }

    pub fn enable(&mut self, cap_name: &str, actor: &str) -> Result<(), String> {
        if !KNOWN_CAPABILITIES.contains(&cap_name) {
            return Err(format!("unknown capability: {cap_name}"));
        }
        let now = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        self.capabilities.insert(
            cap_name.to_string(),
            CapabilityEntry {
                enabled: true,
                enabled_at: now,
                enabled_by: actor.to_string(),
            },
        );
        self.write()
    }

    pub fn disable(&mut self, cap_name: &str) -> Result<(), String> {
        if !KNOWN_CAPABILITIES.contains(&cap_name) {
            return Err(format!("unknown capability: {cap_name}"));
        }
        self.capabilities
            .insert(cap_name.to_string(), CapabilityEntry::default());
        self.write()
    }

    pub fn audit_trail(&self) -> Vec<(String, bool, String, String)> {
        self.capabilities
            .iter()
            .map(|(name, entry)| {
                (
                    name.clone(),
                    entry.enabled,
                    entry.enabled_at.clone(),
                    entry.enabled_by.clone(),
                )
            })
            .collect()
    }

    fn write(&self) -> Result<(), String> {
        let data = CapabilityFileData {
            version: 1,
            integrity_hash: String::new(),
            capability: self.capabilities.clone(),
        };
        let body = toml::to_string_pretty(&data).map_err(|e| format!("TOML serialize: {e}"))?;
        let hash = compute_integrity_hash(&body);
        let mut data_with_hash = data;
        data_with_hash.integrity_hash = hash;
        let final_text =
            toml::to_string_pretty(&data_with_hash).map_err(|e| format!("TOML serialize: {e}"))?;
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create dir: {e}"))?;
        }
        std::fs::write(&self.path, final_text).map_err(|e| format!("write file: {e}"))?;
        Ok(())
    }
}

fn compute_integrity_hash(file_text: &str) -> String {
    let filtered: String = file_text
        .lines()
        .filter(|line| !line.starts_with("integrity_hash"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "sha256-{}",
        hex::encode(Sha256::digest(filtered.as_bytes()))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_default_all_off() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("capabilities.toml");
        let store = CapabilityStore::load(&path);
        for &name in KNOWN_CAPABILITIES {
            assert!(!store.is_enabled(name));
        }
        assert!(path.exists());
    }

    #[test]
    fn enable_disable_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("capabilities.toml");
        let mut store = CapabilityStore::load(&path);
        assert!(!store.is_enabled("tool_calling"));
        store.enable("tool_calling", "admin").unwrap();
        assert!(store.is_enabled("tool_calling"));
        store.disable("tool_calling").unwrap();
        assert!(!store.is_enabled("tool_calling"));
    }

    #[test]
    fn integrity_hash_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("capabilities.toml");
        let mut store = CapabilityStore::load(&path);
        store.enable("federation_canary", "test").unwrap();
        let reloaded = CapabilityStore::load(&path);
        assert!(reloaded.is_enabled("federation_canary"));
    }

    #[test]
    fn tampered_file_falls_back_all_off() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("capabilities.toml");
        let mut store = CapabilityStore::load(&path);
        store.enable("tool_calling", "admin").unwrap();
        let mut text = std::fs::read_to_string(&path).unwrap();
        text = text.replace("enabled = true", "enabled = false");
        std::fs::write(&path, text).unwrap();
        let reloaded = CapabilityStore::load(&path);
        assert!(!reloaded.is_enabled("tool_calling"));
    }

    #[test]
    fn unknown_capability_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("capabilities.toml");
        let mut store = CapabilityStore::load(&path);
        assert!(store.enable("nonexistent", "admin").is_err());
    }
}
