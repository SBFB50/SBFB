// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use crate::daemon_client::DaemonConnection;
use crate::gates::{self, GateResult};

#[derive(Debug)]
pub struct PipelineResult {
    pub gate_results: Vec<GateResult>,
    pub hash: String,
    pub provenance_hash: String,
}

pub fn run_publish_pipeline(
    workspace: &Path,
    repo_url: &str,
    skip_gates: bool,
) -> Result<PipelineResult, Box<dyn std::error::Error>> {
    let mut gate_results = Vec::new();

    // --- Pre-publish gates ---
    let fg4 = gates::run_gate_fg4_diff(workspace)?;
    eprintln!("{fg4}");
    gate_results.push(fg4);

    if !skip_gates {
        let fg5 = gates::run_gate_fg5_sandbox(workspace)?;
        eprintln!("{fg5}");
        if !fg5.passed {
            let issues = fg5.issues.clone();
            gate_results.push(fg5);
            return Err(format!("FG5-sandbox FAIL: {}", issues.join("; ")).into());
        }
        gate_results.push(fg5);

        let fg6 = gates::run_gate_fg6_secrets(workspace)?;
        eprintln!("{fg6}");
        if !fg6.passed {
            let issues = fg6.issues.clone();
            gate_results.push(fg6);
            return Err(format!("FG6-secrets FAIL: {}", issues.join("; ")).into());
        }
        gate_results.push(fg6);
    }

    // --- Publish ---
    let (hash, provenance_hash) = post_deploy_from_repo(workspace, repo_url)?;

    // --- Post-publish gate: FG8 provenance verification ---
    let conn = DaemonConnection::discover()?;
    let node_public_key = conn.get_node_id()?;
    let node_id_hex = hex::encode(node_public_key);
    let provenance_json = conn.get_provenance(&node_id_hex)?;

    let fg8 = gates::run_gate_fg8_provenance(&provenance_json, &node_public_key)?;
    eprintln!("{fg8}");
    if !fg8.passed {
        let issues = fg8.issues.clone();
        gate_results.push(fg8);
        return Err(format!("FG8-provenance FAIL: {}", issues.join("; ")).into());
    }
    gate_results.push(fg8);

    Ok(PipelineResult {
        gate_results,
        hash,
        provenance_hash,
    })
}

fn post_deploy_from_repo(
    workspace: &Path,
    repo_url: &str,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let manifest_path = workspace.join("SBFB.json");
    if !manifest_path.exists() {
        return Err("SBFB.json not found — run `sbfb-factory validate` first".into());
    }
    let content = std::fs::read_to_string(&manifest_path)?;
    let manifest: sbfb_manifest::SbfbManifest = serde_json::from_str(&content)?;
    let name = manifest.name.as_deref().unwrap_or("");
    if name.is_empty() {
        return Err("SBFB.json: name must not be empty".into());
    }

    let conn = DaemonConnection::discover()?;
    let req = serde_json::json!({
        "repo_url": repo_url,
        "project_name": name,
        "category": manifest.category.as_deref().unwrap_or("general"),
        "description": manifest.description.as_deref().unwrap_or_default(),
        "apps": []
    });

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
    let hash = json["hash"].as_str().unwrap_or("unknown").to_string();
    let prov_hash = json["provenance_hash"]
        .as_str()
        .unwrap_or("none")
        .to_string();
    Ok((hash, prov_hash))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_engine;
    use std::fs;
    use tempfile::TempDir;

    fn create_project_with_secret(tmp: &TempDir) -> std::path::PathBuf {
        let out = tmp.path().join("app");
        template_engine::create("static", "test-app", out.to_str().unwrap()).unwrap();
        fs::write(out.join("secret.env"), "AWS_SECRET=AKIAIOSFODNN7EXAMPLE").unwrap();
        out
    }

    fn create_project_with_symlink(tmp: &TempDir) -> std::path::PathBuf {
        let out = tmp.path().join("app");
        template_engine::create("static", "test-app", out.to_str().unwrap()).unwrap();
        let outside = tmp.path().join("outside.txt");
        fs::write(&outside, "secret").unwrap();
        let link = out.join("escape.txt");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &link).unwrap();
        #[cfg(windows)]
        let _ = std::os::windows::fs::symlink_file(&outside, &link);
        out
    }

    fn create_clean_project(tmp: &TempDir) -> std::path::PathBuf {
        let out = tmp.path().join("app");
        template_engine::create("static", "test-app", out.to_str().unwrap()).unwrap();
        out
    }

    #[test]
    fn test_pipeline_aborts_on_secrets() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_project_with_secret(&tmp);
        let err = run_publish_pipeline(&workspace, "https://github.com/t/r", false).unwrap_err();
        assert!(
            err.to_string().contains("FG6-secrets FAIL"),
            "pipeline should abort on secrets: {err}"
        );
    }

    #[test]
    fn test_pipeline_aborts_on_path_traversal() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_project_with_symlink(&tmp);
        let link_exists =
            workspace.join("escape.txt").exists() || workspace.join("escape.txt").is_symlink();
        if !link_exists {
            return;
        }
        let err = run_publish_pipeline(&workspace, "https://github.com/t/r", false).unwrap_err();
        assert!(
            err.to_string().contains("FG5-sandbox FAIL"),
            "pipeline should abort on path traversal: {err}"
        );
    }

    #[test]
    fn test_pipeline_runs_diff_informational() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_clean_project(&tmp);
        // Pipeline will pass pre-publish gates (FG4 informational, FG5/FG6 pass)
        // but fail at publish (no daemon running). The key assertion is that
        // FG4 diff does NOT abort the pipeline.
        let err = run_publish_pipeline(&workspace, "https://github.com/t/r", false).unwrap_err();
        assert!(
            !err.to_string().contains("FG4")
                && !err.to_string().contains("FG5")
                && !err.to_string().contains("FG6"),
            "clean workspace should pass all pre-publish gates: {err}"
        );
    }
}
