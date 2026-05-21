// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

const EXCLUDED_FILES: &[&str] = &["factory.template.lock", "factory.provenance.json"];

#[derive(Debug, Serialize)]
pub struct Provenance {
    pub schema_version: u32,
    pub template_hash: String,
    pub variables_hash: String,
    pub output_hash: String,
    pub generated_at: String,
}

impl Provenance {
    pub fn generate(
        output_dir: &Path,
        template_hash: &str,
        variables: &serde_json::Value,
    ) -> Result<Self, std::io::Error> {
        let variables_hash = {
            let json = serde_json::to_string(variables).unwrap_or_default();
            blake3::hash(json.as_bytes()).to_hex().to_string()
        };

        let output_hash = compute_output_hash(output_dir)?;

        let generated_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(Self {
            schema_version: 1,
            template_hash: template_hash.to_string(),
            variables_hash,
            output_hash,
            generated_at,
        })
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn compute_output_hash(dir: &Path) -> Result<String, std::io::Error> {
    let mut entries: Vec<(String, Vec<u8>)> = Vec::new();

    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = entry.map_err(std::io::Error::other)?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");

        if EXCLUDED_FILES.contains(&rel.as_str()) {
            continue;
        }

        let content = std::fs::read(entry.path())?;
        entries.push((rel, content));
    }

    entries.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut hasher = blake3::Hasher::new();
    for (name, content) in &entries {
        hasher.update(name.as_bytes());
        hasher.update(content);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_workspace(tmp: &TempDir) -> std::path::PathBuf {
        let out = tmp.path().join("app");
        std::fs::create_dir_all(&out).unwrap();
        std::fs::write(out.join("index.html"), "<h1>test</h1>").unwrap();
        std::fs::write(
            out.join("SBFB.json"),
            r#"{"schema_version":2,"name":"test"}"#,
        )
        .unwrap();
        out
    }

    #[test]
    fn test_provenance_hash_deterministic() {
        let tmp = TempDir::new().unwrap();
        let out = create_test_workspace(&tmp);

        let vars = serde_json::json!({"name": "test", "version": "0.1.0"});
        let p1 = Provenance::generate(&out, "abc123", &vars).unwrap();
        let p2 = Provenance::generate(&out, "abc123", &vars).unwrap();

        assert_eq!(p1.output_hash, p2.output_hash);
        assert_eq!(p1.variables_hash, p2.variables_hash);
        assert_eq!(p1.template_hash, p2.template_hash);
    }

    #[test]
    fn test_provenance_template_hash_matches_lock() {
        let tmp = TempDir::new().unwrap();
        let out = create_test_workspace(&tmp);
        let template_hash = "deadbeef".repeat(8);

        let vars = serde_json::json!({"name": "test", "version": "0.1.0"});
        let p = Provenance::generate(&out, &template_hash, &vars).unwrap();

        assert_eq!(p.template_hash, template_hash);
    }

    #[test]
    fn test_provenance_json_parsable() {
        let tmp = TempDir::new().unwrap();
        let out = create_test_workspace(&tmp);

        let vars = serde_json::json!({"name": "test", "version": "0.1.0"});
        let p = Provenance::generate(&out, "abc123", &vars).unwrap();
        let json = p.to_json().unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["schema_version"], 1);
        assert!(parsed["output_hash"].as_str().unwrap().len() == 64);
        assert!(parsed["variables_hash"].as_str().unwrap().len() == 64);
        assert!(!parsed["generated_at"].as_str().unwrap().is_empty());
    }

    #[test]
    fn test_provenance_excludes_lock_and_provenance_files() {
        let tmp = TempDir::new().unwrap();
        let out = create_test_workspace(&tmp);

        let vars = serde_json::json!({"name": "test", "version": "0.1.0"});
        let hash_before = Provenance::generate(&out, "abc", &vars)
            .unwrap()
            .output_hash;

        std::fs::write(out.join("factory.template.lock"), "lock content").unwrap();
        std::fs::write(out.join("factory.provenance.json"), "prov content").unwrap();

        let hash_after = Provenance::generate(&out, "abc", &vars)
            .unwrap()
            .output_hash;

        assert_eq!(hash_before, hash_after);
    }
}
