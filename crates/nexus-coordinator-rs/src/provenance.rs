// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance attestation for verified deploy (SLSA L1).
//! (Sprint 42 Phase B, port of provenance.py S14).
//!
//! Each attestation is signed by the coordinator's Ed25519 keypair
//! and proves that:
//! 1. The artifact was built from a specific repo + commit.
//! 2. The signing coordinator is identified by its `node_id`.
//! 3. The artifact content matches the recorded BLAKE3 hash.

use nexus_core_rs::canonical::DOMAIN_PROVENANCE_V1;
use nexus_core_rs::crypto::{KeyPair, blake3_hash};
use serde::{Deserialize, Serialize};

pub const PROVENANCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceRecord {
    pub schema_version: u32,
    pub repo_url: String,
    pub commit_sha: String,
    pub artifact_hash: String,
    pub node_id: String,
    pub timestamp: String,
    pub signature: String,
}

pub fn generate_provenance(
    repo_url: &str,
    commit_sha: &str,
    artifact_hash: &str,
    node_id_hex: &str,
    keypair: &KeyPair,
) -> ProvenanceRecord {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let canonical = canonical_bytes(
        PROVENANCE_SCHEMA_VERSION,
        repo_url,
        commit_sha,
        artifact_hash,
        node_id_hex,
        &timestamp,
    );
    let sig = keypair.sign(&canonical);
    ProvenanceRecord {
        schema_version: PROVENANCE_SCHEMA_VERSION,
        repo_url: repo_url.to_string(),
        commit_sha: commit_sha.to_string(),
        artifact_hash: artifact_hash.to_string(),
        node_id: node_id_hex.to_string(),
        timestamp,
        signature: hex::encode(sig),
    }
}

pub fn verify_provenance(record_json: &str, public_key: &[u8; 32]) -> bool {
    let Ok(data) = serde_json::from_str::<serde_json::Value>(record_json) else {
        return false;
    };
    let Some(sig_hex) = data["signature"].as_str() else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(sig_hex) else {
        return false;
    };
    let Ok(sig): Result<[u8; 64], _> = sig_bytes.try_into() else {
        return false;
    };

    let schema_version = data["schema_version"].as_u64().unwrap_or(0) as u32;
    let repo_url = data["repo_url"].as_str().unwrap_or_default();
    let commit_sha = data["commit_sha"].as_str().unwrap_or_default();
    let artifact_hash = data["artifact_hash"].as_str().unwrap_or_default();
    let node_id = data["node_id"].as_str().unwrap_or_default();
    let timestamp = data["timestamp"].as_str().unwrap_or_default();

    let canonical = canonical_bytes(
        schema_version,
        repo_url,
        commit_sha,
        artifact_hash,
        node_id,
        timestamp,
    );
    nexus_core_rs::crypto::verify(public_key, &canonical, &sig).is_ok()
}

pub fn provenance_to_json(record: &ProvenanceRecord) -> String {
    serde_json::to_string_pretty(record).unwrap_or_default()
}

pub fn provenance_blake3_hex(record: &ProvenanceRecord) -> String {
    let json = provenance_to_json(record);
    let hash = blake3_hash(json.as_bytes());
    hex::encode(hash)
}

fn canonical_bytes(
    schema_version: u32,
    repo_url: &str,
    commit_sha: &str,
    artifact_hash: &str,
    node_id: &str,
    timestamp: &str,
) -> Vec<u8> {
    let payload = serde_json::json!({
        "artifact_hash": artifact_hash,
        "commit_sha": commit_sha,
        "node_id": node_id,
        "repo_url": repo_url,
        "schema_version": schema_version,
        "timestamp": timestamp,
    });
    let json_bytes = serde_json::to_string(&payload).unwrap_or_default();
    let mut result = Vec::with_capacity(DOMAIN_PROVENANCE_V1.len() + 1 + json_bytes.len());
    result.extend_from_slice(DOMAIN_PROVENANCE_V1);
    result.push(0x00);
    result.extend_from_slice(json_bytes.as_bytes());
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_and_verify_provenance() {
        let kp = KeyPair::generate();
        let record = generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        assert_eq!(record.schema_version, PROVENANCE_SCHEMA_VERSION);
        assert_eq!(record.repo_url, "https://github.com/user/repo");

        let json = provenance_to_json(&record);
        assert!(verify_provenance(&json, &kp.public_bytes()));
    }

    #[test]
    fn verify_fails_wrong_key() {
        let kp = KeyPair::generate();
        let other = KeyPair::generate();
        let record = generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        let json = provenance_to_json(&record);
        assert!(!verify_provenance(&json, &other.public_bytes()));
    }

    #[test]
    fn verify_fails_tampered() {
        let kp = KeyPair::generate();
        let record = generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        let json = provenance_to_json(&record).replace("deadbeef", "tampered");
        assert!(!verify_provenance(&json, &kp.public_bytes()));
    }

    #[test]
    fn blake3_hex_deterministic() {
        let kp = KeyPair::generate();
        let record = generate_provenance(
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
            &hex::encode(kp.public_bytes()),
            &kp,
        );
        let h1 = provenance_blake3_hex(&record);
        let h2 = provenance_blake3_hex(&record);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }
}
