// SPDX-License-Identifier: AGPL-3.0-or-later
//! `sbfb-factory publish` subcommand.
//!
//! Delegates to the publish pipeline (FG4→FG5→FG6→deploy→FG8).

use std::path::Path;

use crate::pipeline;

pub fn run(path: &str, repo_url: &str, skip_gates: bool) -> Result<(), Box<dyn std::error::Error>> {
    let project_dir = dunce::canonicalize(path)?;
    validate_manifest(&project_dir)?;

    let result = pipeline::run_publish_pipeline(&project_dir, repo_url, skip_gates)?;
    let passed = result.gate_results.iter().filter(|g| g.passed).count();
    let total = result.gate_results.len();
    eprintln!(
        "published: hash={}, provenance={}, gates={}/{}",
        result.hash, result.provenance_hash, passed, total
    );
    Ok(())
}

fn validate_manifest(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
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
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publish_requires_running_json() {
        let tmp = tempfile::tempdir().unwrap();
        let out = tmp.path().join("app");
        crate::template_engine::create("static", "test-app", out.to_str().unwrap()).unwrap();

        unsafe { std::env::set_var("NEXUS_GRID_ROOT", tmp.path().join("fake-grid")) };
        let err = run(out.to_str().unwrap(), "https://github.com/test/app", false).unwrap_err();
        assert!(
            err.to_string().contains("daemon not running")
                || err.to_string().contains("running.json"),
            "expected daemon error, got: {err}"
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

        let err = run(
            tmp.path().to_str().unwrap(),
            "https://github.com/test/app",
            false,
        )
        .unwrap_err();
        assert!(err.to_string().contains("name must not be empty"));
    }
}
