// SPDX-License-Identifier: AGPL-3.0-or-later
//! Trust-web manager for Couche 3 Sybil-resistance (Sprint 27 Phase C).
//!
//! Aggregates multi-forge cross-validation data and delegation chains
//! to produce trust scores. The gossip topic
//! `nexus-grid/trust-web/v1` carries signed [`DelegationCert`]s between
//! nodes; subscribers verify the Ed25519 signature + delegation chain
//! before accepting.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use nexus_core_rs::attestations::{DelegationCert, ForgeContribution};

/// Bootstrap seed anchor for the trust-web.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSeed {
    pub org_name: String,
    pub fingerprint: String,
    pub forge_urls: Vec<String>,
    #[serde(default)]
    pub description: String,
}

/// Trust score computed by the trust-web aggregator.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustScore {
    /// Number of distinct forges where the fingerprint has signed commits.
    pub forge_count: u32,
    /// Tenure in days (oldest signed commit to newest).
    pub commit_tenure_days: u32,
    /// Shortest delegation depth from a seed anchor (0 = is a seed).
    pub delegation_depth: u32,
    /// Effective trust level (decayed from the seed's trust_level).
    pub effective_trust_level: u8,
    /// Composite score: forge_count * tenure_factor * trust_level.
    pub composite: f64,
}

/// Result of cross-forge verification for a fingerprint.
#[derive(Debug, Clone, PartialEq)]
pub struct CrossForgeResult {
    /// Fingerprint checked.
    pub fingerprint: String,
    /// Contributions from each distinct forge.
    pub contributions: Vec<ForgeContribution>,
    /// Number of distinct forges (>= 2 = cross-validated).
    pub forge_count: u32,
    /// Whether cross-forge validation passed (>= 2 distinct forges).
    pub cross_validated: bool,
}

/// TOML config file structure for trust-web seeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustWebSeedsConfig {
    #[serde(default)]
    pub seeds: Vec<TrustSeed>,
}

/// Gossip topic for trust-web DelegationCert exchange.
pub const TRUST_WEB_GOSSIP_TOPIC: &str = "nexus-grid/trust-web/v1";

pub struct TrustWebManager {
    seeds: Vec<TrustSeed>,
    delegation_certs: Vec<DelegationCert>,
}

impl TrustWebManager {
    pub fn new(seeds: Vec<TrustSeed>) -> Self {
        Self {
            seeds,
            delegation_certs: Vec::new(),
        }
    }

    pub fn load_seeds_from_toml(path: &Path) -> Result<Vec<TrustSeed>, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("read seeds file: {e}"))?;
        let config: TrustWebSeedsConfig =
            toml::from_str(&content).map_err(|e| format!("parse seeds TOML: {e}"))?;
        Ok(config.seeds)
    }

    pub fn add_delegation_cert(&mut self, cert: DelegationCert) {
        if cert.verify_signature().is_ok() {
            self.delegation_certs.push(cert);
        }
    }

    /// Compute trust score for a fingerprint given all known forge contributions.
    pub fn compute_trust_score(
        &self,
        fingerprint: &str,
        contributions: &[ForgeContribution],
    ) -> TrustScore {
        let matching: Vec<&ForgeContribution> = contributions
            .iter()
            .filter(|c| c.fingerprint == fingerprint)
            .collect();
        let forge_count = {
            let mut forges = std::collections::HashSet::new();
            for c in &matching {
                forges.insert(&c.forge_url);
            }
            forges.len() as u32
        };
        let commit_tenure_days = if matching.is_empty() {
            0
        } else {
            let earliest = matching.iter().map(|c| c.first_seen).min().unwrap_or(0);
            let latest = matching.iter().map(|c| c.last_seen).max().unwrap_or(0);
            (latest.saturating_sub(earliest) / 86400) as u32
        };
        let (delegation_depth, effective_trust_level) = self.find_delegation_chain(fingerprint);

        let tenure_factor = if commit_tenure_days > 0 {
            (commit_tenure_days as f64).ln().max(1.0)
        } else {
            1.0
        };
        let composite = forge_count as f64 * tenure_factor * effective_trust_level as f64;

        TrustScore {
            forge_count,
            commit_tenure_days,
            delegation_depth,
            effective_trust_level,
            composite,
        }
    }

    /// Verify cross-forge presence for a fingerprint.
    pub fn verify_cross_forge(
        &self,
        fingerprint: &str,
        all_contributions: &[ForgeContribution],
    ) -> CrossForgeResult {
        let matching: Vec<ForgeContribution> = all_contributions
            .iter()
            .filter(|c| c.fingerprint == fingerprint)
            .cloned()
            .collect();
        let forge_count = count_distinct_forges(&matching);

        CrossForgeResult {
            fingerprint: fingerprint.to_string(),
            contributions: matching,
            forge_count,
            cross_validated: forge_count >= 2,
        }
    }

    /// Find the shortest delegation chain from a seed to the target fingerprint.
    /// Returns (depth, effective_trust_level). Decay: -1 trust_level per hop, min 1.
    fn find_delegation_chain(&self, fingerprint: &str) -> (u32, u8) {
        // Check if the fingerprint is a seed anchor
        if self.seeds.iter().any(|s| s.fingerprint == fingerprint) {
            return (0, 5);
        }

        // BFS through delegation certs to find shortest path from a seed
        let mut best_depth = u32::MAX;
        let mut best_level: u8 = 1;

        // Build adjacency: delegator (node_id hex) -> [(delegatee fingerprint, trust_level)]
        let mut delegations: HashMap<String, Vec<(String, u8)>> = HashMap::new();
        for cert in &self.delegation_certs {
            let delegator_hex = hex::encode(cert.node_id);
            delegations
                .entry(delegator_hex)
                .or_default()
                .push((cert.delegated_pubkey_fingerprint.clone(), cert.trust_level));
        }

        // For each seed, BFS
        for seed in &self.seeds {
            let mut queue: Vec<(String, u32, u8)> = vec![(seed.fingerprint.clone(), 0, 5)];
            let mut visited = std::collections::HashSet::new();

            while let Some((current_fp, depth, level)) = queue.pop() {
                if current_fp == fingerprint && depth < best_depth {
                    best_depth = depth;
                    best_level = level;
                }
                if visited.contains(&current_fp) {
                    continue;
                }
                visited.insert(current_fp.clone());

                if let Some(delegatees) = delegations.get(&current_fp) {
                    for (delegatee_fp, cert_level) in delegatees {
                        let decayed = (*cert_level).min(level).saturating_sub(1).max(1);
                        queue.push((delegatee_fp.clone(), depth + 1, decayed));
                    }
                }
            }
        }

        if best_depth == u32::MAX {
            (u32::MAX, 1)
        } else {
            (best_depth, best_level)
        }
    }

    pub fn seeds(&self) -> &[TrustSeed] {
        &self.seeds
    }

    pub fn delegation_certs(&self) -> &[DelegationCert] {
        &self.delegation_certs
    }
}

fn count_distinct_forges(contributions: &[ForgeContribution]) -> u32 {
    let mut forges = std::collections::HashSet::new();
    for c in contributions {
        forges.insert(c.forge_url.clone());
    }
    forges.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_core_rs::attestations::DelegationScope;
    use nexus_core_rs::attestations::forge_parser::SigType;

    fn make_contributions() -> Vec<ForgeContribution> {
        vec![
            ForgeContribution {
                fingerprint: "abc123".to_string(),
                commit_count: 50,
                first_seen: 1_680_000_000, // ~2023-03
                last_seen: 1_710_000_000,  // ~2024-03
                forge_url: "https://github.com/user/repo1".to_string(),
                sig_type: SigType::Gpg,
            },
            ForgeContribution {
                fingerprint: "abc123".to_string(),
                commit_count: 20,
                first_seen: 1_690_000_000,
                last_seen: 1_710_000_000,
                forge_url: "https://codeberg.org/user/repo2".to_string(),
                sig_type: SigType::Gpg,
            },
            ForgeContribution {
                fingerprint: "xyz789".to_string(),
                commit_count: 5,
                first_seen: 1_705_000_000,
                last_seen: 1_710_000_000,
                forge_url: "https://github.com/user/repo1".to_string(),
                sig_type: SigType::Ssh,
            },
        ]
    }

    #[test]
    fn test_trust_web_cross_forge_score() {
        let seeds = vec![TrustSeed {
            org_name: "Test".to_string(),
            fingerprint: "seed_key".to_string(),
            forge_urls: vec![],
            description: String::new(),
        }];
        let mgr = TrustWebManager::new(seeds);
        let contribs = make_contributions();

        let score_abc = mgr.compute_trust_score("abc123", &contribs);
        assert_eq!(score_abc.forge_count, 2); // github + codeberg
        assert!(score_abc.commit_tenure_days > 0);

        let score_xyz = mgr.compute_trust_score("xyz789", &contribs);
        assert_eq!(score_xyz.forge_count, 1); // only github

        assert!(
            score_abc.composite > score_xyz.composite,
            "cross-forge contributor should score higher"
        );
    }

    #[test]
    fn test_trust_web_cross_forge_verification() {
        let mgr = TrustWebManager::new(vec![]);
        let contribs = make_contributions();

        let result_abc = mgr.verify_cross_forge("abc123", &contribs);
        assert!(result_abc.cross_validated);
        assert_eq!(result_abc.forge_count, 2);

        let result_xyz = mgr.verify_cross_forge("xyz789", &contribs);
        assert!(!result_xyz.cross_validated);
        assert_eq!(result_xyz.forge_count, 1);
    }

    #[test]
    fn test_trust_web_delegation_decay() {
        let seeds = vec![TrustSeed {
            org_name: "Root".to_string(),
            fingerprint: "seed_fp".to_string(),
            forge_urls: vec![],
            description: String::new(),
        }];
        let mgr = TrustWebManager::new(seeds);

        // Seed directly is depth 0, trust 5
        let (depth, level) = mgr.find_delegation_chain("seed_fp");
        assert_eq!(depth, 0);
        assert_eq!(level, 5);

        // Unknown fingerprint
        let (depth, level) = mgr.find_delegation_chain("unknown");
        assert_eq!(depth, u32::MAX);
        assert_eq!(level, 1);

        // Add a cert from seed_fp delegating to delegate_1
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let cert = nexus_core_rs::attestations::DelegationCert::sign(
            nexus_core_rs::attestations::DELEGATION_ALGO_SSH_ED25519,
            "a1b2c3d4e5f6071829304a5b6c7d8e9f0a1b2c3d4e5f6071829304a5b6c7d8e9",
            1_713_600_000,
            None,
            4,
            Some(DelegationScope {
                org_name: "Test".to_string(),
                forge_urls: vec![],
            }),
            &kp,
        )
        .unwrap();

        // The cert's node_id (delegator) is the kp's pubkey.
        // For the BFS to work, the delegator's hex-encoded node_id
        // must match a seed fingerprint. Let's set the seed to the kp's hex pubkey.
        let delegator_hex = hex::encode(kp.public_bytes());
        let mut mgr2 = TrustWebManager::new(vec![TrustSeed {
            org_name: "Root".to_string(),
            fingerprint: delegator_hex,
            forge_urls: vec![],
            description: String::new(),
        }]);
        mgr2.add_delegation_cert(cert);

        let target_fp = "a1b2c3d4e5f6071829304a5b6c7d8e9f0a1b2c3d4e5f6071829304a5b6c7d8e9";
        let (depth, level) = mgr2.find_delegation_chain(target_fp);
        assert_eq!(depth, 1);
        assert!((1..5).contains(&level), "trust should decay: got {level}");
    }

    #[test]
    fn test_seeds_toml_parse() {
        let toml_str = r#"
[[seeds]]
org_name = "FlowUP (bootstrap)"
fingerprint = "80b439cb0000000000000000000000000000000000000000000000000000abcd"
forge_urls = ["https://github.com/SBFB50/SBFB"]
description = "Bootstrap anchor"
"#;
        let config: TrustWebSeedsConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.seeds.len(), 1);
        assert_eq!(config.seeds[0].org_name, "FlowUP (bootstrap)");
    }
}
