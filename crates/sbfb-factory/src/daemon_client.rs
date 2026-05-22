// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 68 Phase B — daemon connection helper.
//!
//! Reads `running.json` and `auth_token` from standard SBFB paths
//! to build authenticated HTTP requests against the local daemon.

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RunningState {
    api_host: String,
    api_port: u16,
    #[serde(rename = "schema_version")]
    _schema_version: u32,
}

#[derive(Debug)]
pub struct DaemonConnection {
    pub base_url: String,
    pub token: String,
}

impl DaemonConnection {
    pub fn discover() -> Result<Self, DaemonClientError> {
        let running_path = running_json_path().ok_or(DaemonClientError::NotFound(
            "cannot resolve running.json path",
        ))?;
        let content = std::fs::read_to_string(&running_path).map_err(|_| {
            DaemonClientError::NotRunning(format!(
                "daemon not running (no {})",
                running_path.display()
            ))
        })?;
        let state: RunningState = serde_json::from_str(&content)
            .map_err(|e| DaemonClientError::Parse(format!("running.json: {e}")))?;

        let token_path = auth_token_path().ok_or(DaemonClientError::NotFound(
            "cannot resolve auth_token path",
        ))?;
        let token = std::fs::read_to_string(&token_path)
            .map_err(|_| {
                DaemonClientError::NotFound("auth_token file missing — is the daemon initialized?")
            })?
            .trim()
            .to_string();

        Ok(Self {
            base_url: format!("http://{}:{}", state.api_host, state.api_port),
            token,
        })
    }

    pub fn client(&self) -> reqwest::blocking::Client {
        reqwest::blocking::Client::new()
    }

    pub fn get_node_id(&self) -> Result<[u8; 32], DaemonClientError> {
        let url = format!("{}/api/daemon/info", self.base_url);
        let resp = self
            .client()
            .get(&url)
            .header("X-SBFB-Token", &self.token)
            .header("Host", "127.0.0.1")
            .send()
            .map_err(|e| DaemonClientError::NotRunning(format!("GET /api/daemon/info: {e}")))?;
        if !resp.status().is_success() {
            return Err(DaemonClientError::NotRunning(format!(
                "GET /api/daemon/info: {}",
                resp.status()
            )));
        }
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| DaemonClientError::Parse(format!("info response: {e}")))?;
        let node_id_hex = json["node_id"]
            .as_str()
            .ok_or_else(|| DaemonClientError::Parse("info: missing node_id".into()))?;
        let bytes = hex::decode(node_id_hex)
            .map_err(|e| DaemonClientError::Parse(format!("node_id hex: {e}")))?;
        bytes
            .try_into()
            .map_err(|_| DaemonClientError::Parse("node_id must be 32 bytes".into()))
    }

    pub fn get_provenance(&self, project_id: &str) -> Result<String, DaemonClientError> {
        let url = format!("{}/api/v1/project/{}/provenance", self.base_url, project_id);
        let resp = self
            .client()
            .get(&url)
            .header("X-SBFB-Token", &self.token)
            .header("Host", "127.0.0.1")
            .send()
            .map_err(|e| DaemonClientError::NotRunning(format!("GET provenance: {e}")))?;
        if !resp.status().is_success() {
            return Err(DaemonClientError::NotRunning(format!(
                "GET provenance: {}",
                resp.status()
            )));
        }
        let json: serde_json::Value = resp
            .json()
            .map_err(|e| DaemonClientError::Parse(format!("provenance response: {e}")))?;
        let record = &json["record"];
        if record.is_null() {
            return Err(DaemonClientError::NotFound(
                "provenance record not found for project",
            ));
        }
        serde_json::to_string(record)
            .map_err(|e| DaemonClientError::Parse(format!("provenance serialize: {e}")))
    }
}

fn nexus_grid_root() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("NEXUS_GRID_ROOT") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    directories::BaseDirs::new().map(|b| b.data_dir().join("nexus-grid"))
}

fn running_json_path() -> Option<PathBuf> {
    nexus_grid_root().map(|r| r.join("shell-daemon").join("running.json"))
}

fn sbfb_home() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("SBFB_HOME") {
        if !p.is_empty() {
            return Some(PathBuf::from(p));
        }
    }
    std::env::var("HOME")
        .ok()
        .or_else(|| std::env::var("USERPROFILE").ok())
        .filter(|s| !s.is_empty())
        .map(|h| PathBuf::from(h).join(".sbfb"))
}

fn auth_token_path() -> Option<PathBuf> {
    sbfb_home().map(|d| d.join("auth_token"))
}

#[derive(Debug, thiserror::Error)]
pub enum DaemonClientError {
    #[error("{0}")]
    NotRunning(String),
    #[error("{0}")]
    NotFound(&'static str),
    #[error("{0}")]
    Parse(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_fails_without_running_json() {
        let tmp = tempfile::tempdir().unwrap();
        unsafe { std::env::set_var("NEXUS_GRID_ROOT", tmp.path()) };
        let err = DaemonConnection::discover().unwrap_err();
        assert!(matches!(err, DaemonClientError::NotRunning(_)));
        unsafe { std::env::remove_var("NEXUS_GRID_ROOT") };
    }
}
