// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::secret_scanner;
use crate::template_lock::TemplateLock;
use sbfb_manifest::SbfbManifest;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, thiserror::Error)]
pub enum FactoryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("manifest: {0}")]
    Manifest(#[from] sbfb_manifest::ManifestError),

    #[error("template not found: {0}")]
    TemplateNotFound(String),

    #[error("validation: {0}")]
    Validation(String),

    #[error("path traversal: {0}")]
    PathTraversal(String),

    #[error("walkdir: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

struct TemplateFile {
    name: &'static str,
    content: &'static str,
    substitute: bool,
}

const STATIC_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        name: "index.html",
        content: include_str!("templates/static/index.html"),
        substitute: true,
    },
    TemplateFile {
        name: "sbfb-bridge.js",
        content: include_str!("templates/static/sbfb-bridge.js"),
        substitute: false,
    },
    TemplateFile {
        name: "README.md",
        content: include_str!("templates/static/README.md"),
        substitute: true,
    },
    TemplateFile {
        name: ".gitignore",
        content: include_str!("templates/static/gitignore"),
        substitute: false,
    },
];

fn substitute(content: &str, name: &str, version: &str) -> String {
    content
        .replace("{{name}}", name)
        .replace("{{version}}", version)
}

pub fn create(template: &str, name: &str, output_dir: &str) -> Result<(), FactoryError> {
    if template != "static" {
        return Err(FactoryError::TemplateNotFound(template.to_string()));
    }

    let out = Path::new(output_dir);
    fs::create_dir_all(out)?;

    let version = "0.1.0";

    let mut template_files: Vec<(String, String)> = Vec::new();
    for tf in STATIC_TEMPLATE {
        let content = if tf.substitute {
            substitute(tf.content, name, version)
        } else {
            tf.content.to_string()
        };
        fs::write(out.join(tf.name), &content)?;
        template_files.push((tf.name.to_string(), tf.content.to_string()));
    }

    let manifest = SbfbManifest {
        schema_version: Some(2),
        name: Some(name.to_string()),
        version: Some(version.to_string()),
        description: Some("SBFB app created with sbfb-factory".to_string()),
        category: Some("general".to_string()),
        node_id: None,
        repo_url: None,
        bridge: None,
    };
    manifest.validate()?;
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(out.join("SBFB.json"), &manifest_json)?;

    let lock = TemplateLock::generate("static", "1.0.0", &template_files, name, version);
    fs::write(out.join("factory.template.lock"), lock.to_json()?)?;

    eprintln!("Created SBFB app '{}' in {}", name, output_dir);
    Ok(())
}

pub fn validate(path: &str) -> Result<(), FactoryError> {
    if path.contains("..") {
        return Err(FactoryError::PathTraversal(
            "path contains '..' components".to_string(),
        ));
    }

    let dir = Path::new(path);
    if !dir.is_dir() {
        return Err(FactoryError::Validation(format!(
            "'{}' is not a directory",
            path
        )));
    }

    let mut issues = Vec::new();

    let manifest_path = dir.join("SBFB.json");
    if !manifest_path.exists() {
        issues.push("SBFB.json not found".to_string());
    } else {
        let content = fs::read_to_string(&manifest_path)?;
        match SbfbManifest::parse(&content) {
            Ok(m) => {
                if let Err(e) = m.validate() {
                    issues.push(format!("SBFB.json: {e}"));
                }
            }
            Err(e) => issues.push(format!("SBFB.json: {e}")),
        }
    }

    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = entry?;
        if entry.path_is_symlink() {
            let rel = entry.path().strip_prefix(dir).unwrap_or(entry.path());
            issues.push(format!("symlink: {}", rel.display()));
        }
    }

    let findings = secret_scanner::scan_directory(dir);
    for f in &findings {
        issues.push(format!(
            "secret in {}:{}: {}",
            f.file.display(),
            f.line,
            f.pattern_name
        ));
    }

    if issues.is_empty() {
        eprintln!("Validation passed.");
        Ok(())
    } else {
        for issue in &issues {
            eprintln!("  {issue}");
        }
        Err(FactoryError::Validation(format!(
            "{} issue(s) found",
            issues.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_generates_sbfb_json_v2() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "test-app", out.to_str().unwrap()).unwrap();

        let json = fs::read_to_string(out.join("SBFB.json")).unwrap();
        let m = SbfbManifest::parse(&json).unwrap();
        assert_eq!(m.effective_schema_version(), 2);
        assert_eq!(m.name.as_deref(), Some("test-app"));
        assert!(m.validate().is_ok());
    }

    #[test]
    fn test_create_generates_index_html() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "test-app", out.to_str().unwrap()).unwrap();

        assert!(out.join("index.html").exists());
        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("<title>"));
    }

    #[test]
    fn test_create_generates_template_lock() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "test-app", out.to_str().unwrap()).unwrap();

        let lock_path = out.join("factory.template.lock");
        assert!(lock_path.exists());
        let lock_json: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(lock_path).unwrap()).unwrap();
        assert_eq!(lock_json["template_id"], "static");
        assert!(lock_json["template_hash"].as_str().unwrap().len() == 64);
    }

    #[test]
    fn test_create_substitutes_name() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "my-cool-app", out.to_str().unwrap()).unwrap();

        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("my-cool-app"));
        assert!(!html.contains("{{name}}"));
    }

    #[test]
    fn test_validate_accepts_valid_manifest() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "valid-app", out.to_str().unwrap()).unwrap();

        assert!(validate(out.to_str().unwrap()).is_ok());
    }

    #[test]
    fn test_validate_rejects_invalid_manifest() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("SBFB.json"), r#"{"schema_version": 2}"#).unwrap();

        let result = validate(out.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_path_traversal_rejected() {
        let result = validate("some/path/../../../etc");
        assert!(matches!(result, Err(FactoryError::PathTraversal(_))));
    }

    #[test]
    fn test_symlink_rejected() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        fs::create_dir_all(&out).unwrap();

        let manifest = SbfbManifest {
            schema_version: Some(2),
            name: Some("test".to_string()),
            version: Some("1.0.0".to_string()),
            description: None,
            category: None,
            node_id: None,
            repo_url: None,
            bridge: None,
        };
        fs::write(
            out.join("SBFB.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let target = tmp.path().join("outside.txt");
        fs::write(&target, "secret").unwrap();
        let link = out.join("link.txt");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target, &link).unwrap();
            let result = validate(out.to_str().unwrap());
            assert!(result.is_err());
        }

        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(&target, &link).is_ok() {
                let result = validate(out.to_str().unwrap());
                assert!(result.is_err());
            }
        }
    }
}
