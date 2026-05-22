// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::diff;
use crate::secret_scanner;
use crate::template_engine::FactoryError;
use nexus_core_rs::canonical::DOMAIN_PROVENANCE_V1;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct GateResult {
    pub gate: &'static str,
    pub passed: bool,
    pub issues: Vec<String>,
}

impl GateResult {
    fn pass(gate: &'static str) -> Self {
        Self {
            gate,
            passed: true,
            issues: Vec::new(),
        }
    }

    fn fail(gate: &'static str, issues: Vec<String>) -> Self {
        Self {
            gate,
            passed: false,
            issues,
        }
    }
}

impl std::fmt::Display for GateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.passed { "PASS" } else { "FAIL" };
        write!(f, "[{}] {}", self.gate, status)?;
        for issue in &self.issues {
            write!(f, "\n  {issue}")?;
        }
        Ok(())
    }
}

pub fn run_gate_fg4_diff(workspace: &Path) -> Result<GateResult, FactoryError> {
    let entries = diff::diff_workspace(workspace)?;
    let mut lines = Vec::new();
    for entry in &entries {
        let tag = match entry.status {
            diff::DiffStatus::Added => "added",
            diff::DiffStatus::Modified => "modified",
            diff::DiffStatus::Deleted => "deleted",
        };
        lines.push(format!("{tag}: {}", entry.path));
    }
    Ok(GateResult {
        gate: "FG4-diff",
        passed: true,
        issues: lines,
    })
}

pub fn run_gate_fg5_sandbox(workspace: &Path) -> Result<GateResult, FactoryError> {
    let canonical = dunce::canonicalize(workspace).map_err(|e| {
        FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", workspace.display()))
    })?;

    if !canonical.is_dir() {
        return Ok(GateResult::fail(
            "FG5-sandbox",
            vec![format!("'{}' is not a directory", workspace.display())],
        ));
    }

    let mut issues = Vec::new();

    for entry in WalkDir::new(&canonical).follow_links(false) {
        let entry = entry?;

        if entry.path_is_symlink() {
            if let Ok(target) = fs::read_link(entry.path()) {
                let abs_target = if target.is_absolute() {
                    target
                } else {
                    entry.path().parent().unwrap_or(&canonical).join(target)
                };
                match dunce::canonicalize(&abs_target) {
                    Ok(resolved) if !resolved.starts_with(&canonical) => {
                        let rel = entry
                            .path()
                            .strip_prefix(&canonical)
                            .unwrap_or(entry.path());
                        issues.push(format!("symlink escapes workspace: {}", rel.display()));
                    }
                    Err(_) => {
                        let rel = entry
                            .path()
                            .strip_prefix(&canonical)
                            .unwrap_or(entry.path());
                        issues.push(format!("broken symlink: {}", rel.display()));
                    }
                    _ => {}
                }
            }
        }
    }

    if issues.is_empty() {
        Ok(GateResult::pass("FG5-sandbox"))
    } else {
        Ok(GateResult::fail("FG5-sandbox", issues))
    }
}

pub fn check_path_containment(base: &Path, candidate: &Path) -> Result<bool, FactoryError> {
    let base_canonical = dunce::canonicalize(base).map_err(|e| {
        FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", base.display()))
    })?;
    let candidate_canonical = dunce::canonicalize(candidate).map_err(|e| {
        FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", candidate.display()))
    })?;
    Ok(candidate_canonical.starts_with(&base_canonical))
}

pub fn run_gate_fg6_secrets(workspace: &Path) -> Result<GateResult, FactoryError> {
    let mut issues = Vec::new();

    let findings = secret_scanner::scan_directory(workspace);
    for f in &findings {
        let rel = f.file.strip_prefix(workspace).unwrap_or(&f.file);
        issues.push(format!(
            "secret {}:{}: {}",
            rel.display(),
            f.line,
            f.pattern_name
        ));
    }

    let lock_path = workspace.join("factory.template.lock");
    let prov_path = workspace.join("factory.provenance.json");

    if lock_path.exists() && prov_path.exists() {
        let lock: serde_json::Value = serde_json::from_str(&fs::read_to_string(&lock_path)?)?;
        let prov: serde_json::Value = serde_json::from_str(&fs::read_to_string(&prov_path)?)?;

        let lock_hash = lock["template_hash"].as_str().unwrap_or("");
        let prov_hash = prov["template_hash"].as_str().unwrap_or("");

        if !lock_hash.is_empty() && !prov_hash.is_empty() && lock_hash != prov_hash {
            issues.push(format!(
                "template_hash mismatch: lock={} provenance={}",
                &lock_hash[..8.min(lock_hash.len())],
                &prov_hash[..8.min(prov_hash.len())]
            ));
        }
    }

    if issues.is_empty() {
        Ok(GateResult::pass("FG6-secrets"))
    } else {
        Ok(GateResult::fail("FG6-secrets", issues))
    }
}

pub fn run_gate_fg7_preview(workspace: &Path) -> Result<GateResult, FactoryError> {
    if !workspace.join("index.html").exists() {
        return Ok(GateResult::fail(
            "FG7-preview",
            vec!["index.html not found".into()],
        ));
    }

    match crate::daemon_client::DaemonConnection::discover() {
        Ok(_) => Ok(GateResult::pass("FG7-preview")),
        Err(e) => Ok(GateResult::fail(
            "FG7-preview",
            vec![format!("daemon: {e}")],
        )),
    }
}

fn provenance_canonical_bytes(
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

pub fn run_gate_fg8_provenance(
    provenance_json: &str,
    node_public_key: &[u8; 32],
) -> Result<GateResult, FactoryError> {
    let data: serde_json::Value = serde_json::from_str(provenance_json)
        .map_err(|e| FactoryError::Validation(format!("provenance JSON parse: {e}")))?;

    let sig_hex = data["signature"]
        .as_str()
        .ok_or_else(|| FactoryError::Validation("provenance: missing signature".into()))?;
    let sig_bytes = hex::decode(sig_hex)
        .map_err(|e| FactoryError::Validation(format!("provenance: bad signature hex: {e}")))?;
    let sig: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| FactoryError::Validation("provenance: signature must be 64 bytes".into()))?;

    let schema_version = data["schema_version"].as_u64().unwrap_or(0) as u32;
    let repo_url = data["repo_url"].as_str().unwrap_or_default();
    let commit_sha = data["commit_sha"].as_str().unwrap_or_default();
    let artifact_hash = data["artifact_hash"].as_str().unwrap_or_default();
    let node_id = data["node_id"].as_str().unwrap_or_default();
    let timestamp = data["timestamp"].as_str().unwrap_or_default();

    let canonical = provenance_canonical_bytes(
        schema_version,
        repo_url,
        commit_sha,
        artifact_hash,
        node_id,
        timestamp,
    );

    match nexus_core_rs::crypto::verify(node_public_key, &canonical, &sig) {
        Ok(()) => Ok(GateResult::pass("FG8-provenance")),
        Err(_) => Ok(GateResult::fail(
            "FG8-provenance",
            vec!["Ed25519 signature verification failed".into()],
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template_engine;
    use tempfile::TempDir;

    fn create_factory_project(tmp: &TempDir) -> std::path::PathBuf {
        let out = tmp.path().join("app");
        template_engine::create("static", "test-app", out.to_str().unwrap()).unwrap();
        out
    }

    #[test]
    fn test_fg5_rejects_path_traversal_canonicalize() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let traversal = workspace.join("..").join("outside");
        let contained = check_path_containment(&workspace, &traversal).unwrap();
        assert!(!contained, "path traversal via .. should escape workspace");
    }

    #[test]
    fn test_fg5_rejects_windows_backslash_traversal() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        let outside = tmp.path().join("outside");
        fs::create_dir_all(&workspace).unwrap();
        fs::create_dir_all(&outside).unwrap();

        let traversal_str = format!(
            "{}{}..{}outside",
            workspace.display(),
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        let traversal = std::path::PathBuf::from(&traversal_str);
        let contained = check_path_containment(&workspace, &traversal).unwrap();
        assert!(
            !contained,
            "path traversal via platform separator should escape workspace"
        );
    }

    #[test]
    fn test_fg5_rejects_symlink() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);
        let outside_file = tmp.path().join("secret.txt");
        fs::write(&outside_file, "secret data").unwrap();
        let link = workspace.join("link.txt");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside_file, &link).unwrap();
            let result = run_gate_fg5_sandbox(&workspace).unwrap();
            assert!(!result.passed, "symlink escaping workspace should fail FG5");
            assert!(
                result.issues.iter().any(|i| i.contains("symlink")),
                "issue should mention symlink"
            );
        }

        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(&outside_file, &link).is_ok() {
                let result = run_gate_fg5_sandbox(&workspace).unwrap();
                assert!(!result.passed, "symlink escaping workspace should fail FG5");
                assert!(
                    result.issues.iter().any(|i| i.contains("symlink")),
                    "issue should mention symlink"
                );
            }
        }
    }

    #[test]
    fn test_fg5_accepts_valid_subdir() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("workspace");
        let subdir = workspace.join("src").join("components");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("App.tsx"), "export default App").unwrap();

        let contained = check_path_containment(&workspace, &subdir.join("App.tsx")).unwrap();
        assert!(contained, "valid subdir path should be contained");

        let result = run_gate_fg5_sandbox(&workspace).unwrap();
        assert!(result.passed, "valid workspace should pass FG5");
    }

    #[test]
    fn test_fg6_lockfile_hash_consistency() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);

        let result = run_gate_fg6_secrets(&workspace).unwrap();
        assert!(
            result.passed,
            "factory-created project should have consistent hashes: {:?}",
            result.issues
        );
    }

    #[test]
    fn test_fg6_lockfile_mismatch_detected() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);

        let prov_path = workspace.join("factory.provenance.json");
        let mut prov: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&prov_path).unwrap()).unwrap();
        prov["template_hash"] = serde_json::Value::String("a".repeat(64));
        fs::write(&prov_path, serde_json::to_string_pretty(&prov).unwrap()).unwrap();

        let result = run_gate_fg6_secrets(&workspace).unwrap();
        assert!(!result.passed, "tampered provenance should fail FG6");
        assert!(
            result
                .issues
                .iter()
                .any(|i| i.contains("template_hash mismatch")),
            "issue should mention hash mismatch"
        );
    }

    #[test]
    fn test_fg6_detects_aws_secret() {
        let tmp = TempDir::new().unwrap();
        let workspace = tmp.path().join("app");
        fs::create_dir_all(&workspace).unwrap();
        fs::write(workspace.join("config.env"), "AWS_KEY=AKIAIOSFODNN7EXAMPLE").unwrap();

        let result = run_gate_fg6_secrets(&workspace).unwrap();
        assert!(!result.passed, "workspace with AWS key should fail FG6");
        assert!(
            result.issues.iter().any(|i| i.contains("AWS")),
            "issue should mention AWS key"
        );
    }

    fn sign_test_provenance(
        kp: &nexus_core_rs::crypto::KeyPair,
        repo_url: &str,
        commit_sha: &str,
        artifact_hash: &str,
    ) -> String {
        let node_id_hex = hex::encode(kp.public_bytes());
        let timestamp = "2026-05-22T12:00:00+00:00";
        let canonical = provenance_canonical_bytes(
            1,
            repo_url,
            commit_sha,
            artifact_hash,
            &node_id_hex,
            timestamp,
        );
        let sig = kp.sign(&canonical);
        serde_json::json!({
            "schema_version": 1,
            "repo_url": repo_url,
            "commit_sha": commit_sha,
            "artifact_hash": artifact_hash,
            "node_id": node_id_hex,
            "timestamp": timestamp,
            "signature": hex::encode(sig),
        })
        .to_string()
    }

    #[test]
    fn test_fg8_provenance_valid_signature() {
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let json = sign_test_provenance(
            &kp,
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
        );
        let result = run_gate_fg8_provenance(&json, &kp.public_bytes()).unwrap();
        assert!(result.passed, "valid provenance should pass FG8");
    }

    #[test]
    fn test_fg8_provenance_wrong_key() {
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let other = nexus_core_rs::crypto::KeyPair::generate();
        let json = sign_test_provenance(
            &kp,
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
        );
        let result = run_gate_fg8_provenance(&json, &other.public_bytes()).unwrap();
        assert!(!result.passed, "wrong key should fail FG8");
        assert!(result.issues.iter().any(|i| i.contains("signature")));
    }

    #[test]
    fn test_fg8_provenance_tampered_json() {
        let kp = nexus_core_rs::crypto::KeyPair::generate();
        let json = sign_test_provenance(
            &kp,
            "https://github.com/user/repo",
            "abc123def456abc123def456abc123def456abc1",
            "deadbeef",
        );
        let tampered = json.replace("deadbeef", "tampered");
        let result = run_gate_fg8_provenance(&tampered, &kp.public_bytes()).unwrap();
        assert!(!result.passed, "tampered provenance should fail FG8");
    }
}
