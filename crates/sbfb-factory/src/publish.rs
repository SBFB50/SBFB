// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 68 Phase B — `sbfb-factory publish` subcommand.
//!
//! Pre-validates the project manifest, then delegates to the
//! daemon's `POST /api/v1/deploy-from-repo` endpoint which
//! handles clone, zip, provenance, and gossip broadcast.

use std::path::Path;

use crate::daemon_client::DaemonConnection;

#[derive(Debug, serde::Serialize)]
struct DeployFromRepoRequest {
    repo_url: String,
    project_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    commit_sha: Option<String>,
    category: String,
    description: String,
    apps: Vec<String>,
}

pub fn run(path: &str, repo_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = dunce::canonicalize(path)?;

    let manifest = load_and_validate_manifest(&project_dir)?;
    let conn = DaemonConnection::discover()?;

    let req = DeployFromRepoRequest {
        repo_url: repo_url.to_string(),
        project_name: manifest.name.clone().unwrap_or_default(),
        commit_sha: None,
        category: manifest
            .category
            .clone()
            .unwrap_or_else(|| "general".into()),
        description: manifest.description.clone().unwrap_or_default(),
        apps: Vec::new(),
    };

    let url = format!("{}/api/v1/deploy-from-repo", conn.base_url);
    let resp = conn
        .client()
        .post(&url)
        .header("X-SBFB-Token", &conn.token)
        .header("Host", "127.0.0.1")
        .json(&req)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("deploy failed ({status}): {body}").into());
    }

    let json: serde_json::Value = resp.json()?;
    let hash = json["hash"].as_str().unwrap_or("unknown");
    let prov = json["provenance_hash"].as_str().unwrap_or("none");
    eprintln!("published: hash={hash}, provenance={prov}");
    Ok(())
}

fn load_and_validate_manifest(
    dir: &Path,
) -> Result<sbfb_manifest::SbfbManifest, Box<dyn std::error::Error>> {
    let manifest_path = dir.join("SBFB.json");
    if !manifest_path.exists() {
        return Err("SBFB.json not found — run `sbfb-factory validate` first".into());
    }
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: sbfb_manifest::SbfbManifest = serde_json::from_str(&content)?;
    let name = manifest.name.as_deref().unwrap_or("");
    if name.is_empty() {
        return Err("SBFB.json: name must not be empty".into());
    }
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_requires_running_json() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = serde_json::json!({
            "sbfb_version": 2,
            "name": "test-app",
            "description": "A test"
        });
        std::fs::write(
            tmp.path().join("SBFB.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        unsafe { std::env::set_var("NEXUS_GRID_ROOT", tmp.path().join("fake-grid")) };
        let err = run(tmp.path().to_str().unwrap(), "https://github.com/test/app").unwrap_err();
        assert!(
            err.to_string().contains("daemon not running")
                || err.to_string().contains("running.json")
        );
        unsafe { std::env::remove_var("NEXUS_GRID_ROOT") };
    }

    #[test]
    fn publish_pre_validates_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let bad_manifest = serde_json::json!({
            "sbfb_version": 2,
            "name": ""
        });
        std::fs::write(
            tmp.path().join("SBFB.json"),
            serde_json::to_string_pretty(&bad_manifest).unwrap(),
        )
        .unwrap();

        let err = run(tmp.path().to_str().unwrap(), "https://github.com/test/app").unwrap_err();
        assert!(err.to_string().contains("name must not be empty"));
    }
}
