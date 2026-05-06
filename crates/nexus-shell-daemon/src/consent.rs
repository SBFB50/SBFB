// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU consent endpoints — port of api/consent.py (Sprint 43 Phase B).

use std::path::PathBuf;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::http::DaemonHttpState;

fn consent_path(override_home: Option<&std::path::Path>) -> Option<PathBuf> {
    let home = override_home
        .map(|p| p.to_path_buf())
        .or_else(nexus_shell_daemon_core::auth::sbfb_home);
    home.map(|d| d.join("consent.json"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Caps {
    #[serde(default = "default_max_watts")]
    pub max_watts: Option<u32>,
    #[serde(default = "default_max_vram_mb")]
    pub max_vram_mb: Option<u32>,
    #[serde(default = "default_max_hours_day")]
    pub max_hours_day: Option<f64>,
}

fn default_max_watts() -> Option<u32> {
    Some(400)
}
fn default_max_vram_mb() -> Option<u32> {
    Some(16 * 1024)
}
fn default_max_hours_day() -> Option<f64> {
    Some(12.0)
}

impl Default for Caps {
    fn default() -> Self {
        Self {
            max_watts: default_max_watts(),
            max_vram_mb: default_max_vram_mb(),
            max_hours_day: default_max_hours_day(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentConfig {
    #[serde(default = "default_level")]
    pub level: u8,
    #[serde(default)]
    pub caps: Caps,
    #[serde(default)]
    pub allowed_project_ids: Vec<String>,
    #[serde(default)]
    pub own_node_id: String,
    #[serde(default)]
    pub level_threat_note: String,
    #[serde(default)]
    pub residual_threats_acknowledged: Vec<String>,
}

fn default_level() -> u8 {
    1
}

impl Default for ConsentConfig {
    fn default() -> Self {
        Self {
            level: 1,
            caps: Caps::default(),
            allowed_project_ids: Vec::new(),
            own_node_id: String::new(),
            level_threat_note: String::new(),
            residual_threats_acknowledged: Vec::new(),
        }
    }
}

fn threat_note_for_level(level: u8) -> &'static str {
    match level {
        1 => "Aucune exposition tierce. Seules vos propres apps s\u{2019}ex\u{e9}cutent.",
        2 => {
            "Apps open source v\u{e9}rifi\u{e9}es (SLSA L1). Exposition Sybil si contributeur malveillant."
        }
        3 => {
            "Apps s\u{e9}lectionn\u{e9}es manuellement. Vous \u{ea}tes responsable de la v\u{e9}rification."
        }
        4 => "Toute app publique du r\u{e9}seau. Risque maximum de consommation abusive.",
        _ => "",
    }
}

fn residual_threats_for_level(level: u8) -> Vec<String> {
    match level {
        1 => vec![],
        2 => vec!["R2-supply-chain".into(), "R5-kudos-linkability".into()],
        3 => vec![
            "R2-supply-chain".into(),
            "R3-rate-limit-absent".into(),
            "R5-kudos-linkability".into(),
        ],
        4 => vec![
            "R2-supply-chain".into(),
            "R3-rate-limit-absent".into(),
            "R4-consent-race".into(),
            "R5-kudos-linkability".into(),
        ],
        _ => vec![],
    }
}

fn enrich(mut cfg: ConsentConfig) -> ConsentConfig {
    cfg.level_threat_note = threat_note_for_level(cfg.level).to_string();
    cfg.residual_threats_acknowledged = residual_threats_for_level(cfg.level);
    cfg
}

fn load_consent(home: Option<&std::path::Path>) -> ConsentConfig {
    let path = match consent_path(home) {
        Some(p) => p,
        None => return ConsentConfig::default(),
    };
    let body = match std::fs::read_to_string(&path) {
        Ok(b) => b,
        Err(_) => return ConsentConfig::default(),
    };
    serde_json::from_str(&body).unwrap_or_default()
}

fn save_consent(cfg: &ConsentConfig, home: Option<&std::path::Path>) -> Result<(), String> {
    let path = consent_path(home).ok_or("cannot resolve SBFB_HOME")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let body = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &body).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

fn validate_node_id(id: &str) -> bool {
    id.len() == 64 && id.chars().all(|c| c.is_ascii_hexdigit())
}

pub async fn get_consent(State(state): State<Arc<DaemonHttpState>>) -> Json<ConsentConfig> {
    Json(enrich(load_consent(state.sbfb_home.as_deref())))
}

pub async fn set_consent(
    State(state): State<Arc<DaemonHttpState>>,
    Json(cfg): Json<ConsentConfig>,
) -> Result<Json<ConsentConfig>, (StatusCode, String)> {
    if !(1..=4).contains(&cfg.level) {
        return Err((StatusCode::BAD_REQUEST, "level must be 1-4".into()));
    }
    for id in &cfg.allowed_project_ids {
        if !validate_node_id(id) {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("invalid node_id format: {id}"),
            ));
        }
    }
    save_consent(&cfg, state.sbfb_home.as_deref())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    tracing::info!(
        level = cfg.level,
        whitelist_size = cfg.allowed_project_ids.len(),
        "consent.json updated"
    );
    Ok(Json(enrich(cfg)))
}

#[derive(Debug, Deserialize)]
pub struct WhitelistEntry {
    pub project_id: Option<String>,
    pub repo_url: Option<String>,
}

pub async fn whitelist_add(
    State(state): State<Arc<DaemonHttpState>>,
    Json(entry): Json<WhitelistEntry>,
) -> Result<Json<ConsentConfig>, (StatusCode, String)> {
    let pid = match entry.project_id {
        Some(ref id) => {
            if !validate_node_id(id) {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("invalid node_id format: {id}"),
                ));
            }
            id.clone()
        }
        None => {
            if entry.repo_url.is_some() {
                return Err((
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "repo_url -> node_id resolution not yet wired; paste the node_id hex instead"
                        .into(),
                ));
            }
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "project_id or repo_url required".into(),
            ));
        }
    };

    let mut cfg = load_consent(state.sbfb_home.as_deref());
    if !cfg.allowed_project_ids.contains(&pid) {
        cfg.allowed_project_ids.push(pid.clone());
        save_consent(&cfg, state.sbfb_home.as_deref())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        tracing::info!(project_id = %pid, "consent whitelist add");
    }
    Ok(Json(cfg))
}

pub async fn whitelist_remove(
    State(state): State<Arc<DaemonHttpState>>,
    Json(entry): Json<WhitelistEntry>,
) -> Result<Json<ConsentConfig>, (StatusCode, String)> {
    let pid = entry.project_id.ok_or((
        StatusCode::UNPROCESSABLE_ENTITY,
        "project_id required".into(),
    ))?;

    let mut cfg = load_consent(state.sbfb_home.as_deref());
    if let Some(pos) = cfg.allowed_project_ids.iter().position(|x| x == &pid) {
        cfg.allowed_project_ids.remove(pos);
        save_consent(&cfg, state.sbfb_home.as_deref())
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
        tracing::info!(project_id = %pid, "consent whitelist remove");
    }
    Ok(Json(cfg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_consent_level_1() {
        let cfg = ConsentConfig::default();
        assert_eq!(cfg.level, 1);
        assert!(cfg.allowed_project_ids.is_empty());
    }

    #[test]
    fn enrich_adds_threat_fields() {
        let cfg = enrich(ConsentConfig {
            level: 3,
            ..Default::default()
        });
        assert!(!cfg.level_threat_note.is_empty());
        assert!(
            cfg.residual_threats_acknowledged
                .contains(&"R3-rate-limit-absent".to_string())
        );
    }

    #[test]
    fn validate_node_id_valid() {
        assert!(validate_node_id(&"a".repeat(64)));
    }

    #[test]
    fn validate_node_id_short() {
        assert!(!validate_node_id("abc"));
    }

    #[test]
    fn validate_node_id_non_hex() {
        assert!(!validate_node_id(&"g".repeat(64)));
    }

    #[test]
    fn roundtrip_serde() {
        let cfg = ConsentConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: ConsentConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.level, 1);
    }

    #[test]
    fn threat_note_all_levels() {
        for level in 1..=4 {
            assert!(!threat_note_for_level(level).is_empty());
        }
        assert!(threat_note_for_level(0).is_empty());
    }

    #[test]
    fn residual_threats_monotonic() {
        for level in 1..=3 {
            assert!(
                residual_threats_for_level(level).len()
                    <= residual_threats_for_level(level + 1).len()
            );
        }
    }
}
