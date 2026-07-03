// SPDX-License-Identifier: AGPL-3.0-or-later
//! ProofCard evidence-score computation.
//!
//! A ProofCard is a **local compute** artefact — the daemon
//! assembles data it already possesses (browse entry, provenance
//! record, curator lists) and produces a deterministic 0-100
//! evidence-completeness score. It is NOT a signed wire format
//! and does NOT go through `canonical_bytes`.

use serde::Serialize;

pub const FORMULA_VERSION: u32 = 1;

const FRESH_THRESHOLD_DAYS: u64 = 30;
const AGING_THRESHOLD_DAYS: u64 = 90;
const OLD_RELEASE_THRESHOLD_DAYS: u64 = 180;

/// Caller-provided data for the score computation.
#[derive(Debug, Clone)]
pub struct ProofCardInput {
    pub project_id: String,
    pub project_name: String,
    pub provenance_verified: bool,
    pub repo_url: Option<String>,
    pub commit_sha: Option<String>,
    pub is_open_source: bool,
    pub archive_hash: Option<String>,
    pub provenance_hash: Option<String>,
    pub license_spdx: Option<String>,
    pub curator_count: usize,
    pub curator_names: Vec<String>,
    pub deploy_timestamp_rfc3339: Option<String>,
}

// -----------------------------------------------------------------
// Output structs (serialised to JSON for the HTTP response)
// -----------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ProofCard {
    pub project_id: String,
    pub project_name: String,
    pub hash: ProofCardHash,
    pub license: ProofCardLicense,
    pub freshness: ProofCardFreshness,
    pub provenance: ProofCardProvenance,
    pub risk: ProofCardRisk,
    pub curation: ProofCardCuration,
    pub confidence: u8,
    pub formula_version: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardHash {
    pub archive_hash: Option<String>,
    pub provenance_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardLicense {
    pub spdx: Option<String>,
    pub source: LicenseSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenseSource {
    Manifest,
    Inferred,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardFreshness {
    pub last_verified_at: Option<String>,
    pub age_days: Option<u64>,
    pub state: FreshnessState,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FreshnessState {
    Fresh,
    Aging,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardProvenance {
    pub verified: bool,
    pub repo_url: Option<String>,
    pub commit_sha: Option<String>,
    pub slsa_level: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardRisk {
    pub level: RiskLevel,
    pub factors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProofCardCuration {
    pub curator_count: usize,
    pub curator_names: Vec<String>,
}

// -----------------------------------------------------------------
// Score computation
// -----------------------------------------------------------------

pub fn compute_proof_card(input: ProofCardInput) -> ProofCard {
    compute_proof_card_at(input, chrono::Utc::now())
}

fn compute_proof_card_at(input: ProofCardInput, now: chrono::DateTime<chrono::Utc>) -> ProofCard {
    // -- Freshness --------------------------------------------------
    let (last_verified_at, age_days, freshness_state) = match &input.deploy_timestamp_rfc3339 {
        Some(ts) => match chrono::DateTime::parse_from_rfc3339(ts) {
            Ok(dt) => {
                let age = (now - dt.with_timezone(&chrono::Utc)).num_days().max(0) as u64;
                let state = if age <= FRESH_THRESHOLD_DAYS {
                    FreshnessState::Fresh
                } else if age <= AGING_THRESHOLD_DAYS {
                    FreshnessState::Aging
                } else {
                    FreshnessState::Stale
                };
                (Some(ts.clone()), Some(age), state)
            }
            Err(_) => (None, None, FreshnessState::Unknown),
        },
        None => (None, None, FreshnessState::Unknown),
    };

    // -- Risk factors -----------------------------------------------
    let mut risk_factors: Vec<String> = Vec::new();

    if !input.provenance_verified {
        if input.provenance_hash.is_some() {
            risk_factors.push("unverified_deploy".into());
        } else if input.repo_url.is_some() {
            risk_factors.push("no_provenance".into());
        }
    }
    if freshness_state == FreshnessState::Stale {
        risk_factors.push("stale_source".into());
    }
    if let Some(age) = age_days
        && age > OLD_RELEASE_THRESHOLD_DAYS
    {
        risk_factors.push("old_release".into());
    }

    // -- Score (additive, deterministic) -----------------------------
    let mut score: i32 = 30; // base: the project exists

    // Evidence bonuses
    if input.provenance_verified {
        score += 20;
    }
    if input.is_open_source {
        score += 10;
    }
    match freshness_state {
        FreshnessState::Fresh => score += 10,
        FreshnessState::Aging => score += 5,
        _ => {}
    }
    if input.curator_count >= 1 {
        score += 10;
    }
    if input.curator_count >= 3 {
        score += 10;
    }
    if input.license_spdx.is_some() {
        score += 5;
    }
    if input.archive_hash.is_some() {
        score += 5;
    }

    // Risk deductions
    for f in &risk_factors {
        match f.as_str() {
            "no_provenance" => score -= 15,
            "stale_source" => score -= 10,
            "unverified_deploy" => score -= 10,
            "old_release" => score -= 5,
            _ => {}
        }
    }

    let confidence = score.clamp(0, 100) as u8;

    // -- Risk level -------------------------------------------------
    let risk_level = if risk_factors.is_empty() {
        RiskLevel::Low
    } else if risk_factors
        .iter()
        .any(|f| f == "no_provenance" || f == "unverified_deploy")
    {
        RiskLevel::High
    } else {
        RiskLevel::Medium
    };

    // -- SLSA level -------------------------------------------------
    let slsa_level = u8::from(input.provenance_verified);

    ProofCard {
        project_id: input.project_id,
        project_name: input.project_name,
        hash: ProofCardHash {
            archive_hash: input.archive_hash,
            provenance_hash: input.provenance_hash,
        },
        license: ProofCardLicense {
            spdx: input.license_spdx,
            source: LicenseSource::Unknown,
        },
        freshness: ProofCardFreshness {
            last_verified_at,
            age_days,
            state: freshness_state,
        },
        provenance: ProofCardProvenance {
            verified: input.provenance_verified,
            repo_url: input.repo_url,
            commit_sha: input.commit_sha,
            slsa_level,
        },
        risk: ProofCardRisk {
            level: risk_level,
            factors: risk_factors,
        },
        curation: ProofCardCuration {
            curator_count: input.curator_count,
            curator_names: input.curator_names,
        },
        confidence,
        formula_version: FORMULA_VERSION,
    }
}

// -----------------------------------------------------------------
// Tests
// -----------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    fn minimal_input() -> ProofCardInput {
        ProofCardInput {
            project_id: "a".repeat(64),
            project_name: "test-project".into(),
            provenance_verified: false,
            repo_url: None,
            commit_sha: None,
            is_open_source: false,
            archive_hash: None,
            provenance_hash: None,
            license_spdx: None,
            curator_count: 0,
            curator_names: vec![],
            deploy_timestamp_rfc3339: None,
        }
    }

    fn full_input(now: chrono::DateTime<Utc>) -> ProofCardInput {
        let ts =
            (now - chrono::Duration::days(5)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        ProofCardInput {
            project_id: "b".repeat(64),
            project_name: "full-project".into(),
            provenance_verified: true,
            repo_url: Some("https://github.com/org/app".into()),
            commit_sha: Some("abc123".into()),
            is_open_source: true,
            archive_hash: Some("deadbeef".into()),
            provenance_hash: Some("cafebabe".into()),
            license_spdx: Some("AGPL-3.0-or-later".into()),
            curator_count: 3,
            curator_names: vec!["Alice".into(), "Bob".into(), "Charlie".into()],
            deploy_timestamp_rfc3339: Some(ts),
        }
    }

    #[test]
    fn test_proof_card_full_evidence() {
        let now = Utc::now();
        let card = compute_proof_card_at(full_input(now), now);
        assert_eq!(card.confidence, 100);
        assert!(card.risk.factors.is_empty());
        assert_eq!(card.risk.level, RiskLevel::Low);
    }

    #[test]
    fn test_proof_card_minimal() {
        let card = compute_proof_card(minimal_input());
        assert_eq!(card.confidence, 30);
        assert!(card.risk.factors.is_empty());
    }

    #[test]
    fn test_proof_card_provenance_boost() {
        let mut input = minimal_input();
        input.provenance_verified = true;
        input.provenance_hash = Some("hash".into());
        let card = compute_proof_card(input);
        assert_eq!(card.confidence, 50);
        assert!(card.provenance.verified);
        assert_eq!(card.provenance.slsa_level, 1);
    }

    #[test]
    fn test_proof_card_risk_no_provenance() {
        let mut input = minimal_input();
        input.repo_url = Some("https://github.com/org/app".into());
        let card = compute_proof_card(input);
        assert_eq!(card.confidence, 15);
        assert!(card.risk.factors.contains(&"no_provenance".to_string()));
        assert_eq!(card.risk.level, RiskLevel::High);
    }

    #[test]
    fn test_proof_card_formula_version() {
        let card = compute_proof_card(minimal_input());
        assert_eq!(card.formula_version, FORMULA_VERSION);
        assert_eq!(card.formula_version, 1);
    }

    #[test]
    fn test_proof_card_clamp_bounds() {
        // Score cannot exceed 100 even with hypothetical over-counting
        let now = Utc::now();
        let card = compute_proof_card_at(full_input(now), now);
        assert!(card.confidence <= 100);

        // Score cannot go below 0 even with maximum risk deductions
        let mut input = minimal_input();
        input.repo_url = Some("https://github.com/org/app".into());
        let stale_ts = (Utc::now() - chrono::Duration::days(200))
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        input.deploy_timestamp_rfc3339 = Some(stale_ts);
        let card = compute_proof_card(input);
        assert_eq!(card.confidence, 0);
    }

    #[test]
    fn test_proof_card_freshness_states() {
        let now = Utc.with_ymd_and_hms(2026, 5, 21, 12, 0, 0).unwrap();

        // Fresh (5 days)
        let mut input = minimal_input();
        let ts =
            (now - chrono::Duration::days(5)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        input.deploy_timestamp_rfc3339 = Some(ts);
        let card = compute_proof_card_at(input, now);
        assert_eq!(card.freshness.state, FreshnessState::Fresh);
        assert_eq!(card.freshness.age_days, Some(5));

        // Aging (60 days)
        let mut input = minimal_input();
        let ts =
            (now - chrono::Duration::days(60)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        input.deploy_timestamp_rfc3339 = Some(ts);
        let card = compute_proof_card_at(input, now);
        assert_eq!(card.freshness.state, FreshnessState::Aging);

        // Stale (120 days)
        let mut input = minimal_input();
        let ts =
            (now - chrono::Duration::days(120)).to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
        input.deploy_timestamp_rfc3339 = Some(ts);
        let card = compute_proof_card_at(input, now);
        assert_eq!(card.freshness.state, FreshnessState::Stale);
        assert!(card.risk.factors.contains(&"stale_source".to_string()));
    }

    #[test]
    fn test_proof_card_unverified_deploy() {
        let mut input = minimal_input();
        input.provenance_hash = Some("hash".into());
        input.provenance_verified = false;
        let card = compute_proof_card(input);
        assert!(card.risk.factors.contains(&"unverified_deploy".to_string()));
        assert!(!card.risk.factors.contains(&"no_provenance".to_string()));
        assert_eq!(card.confidence, 20);
    }
}
