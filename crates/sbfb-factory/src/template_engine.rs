// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::provenance::Provenance;
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

const STATIC_READER_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        name: "index.html",
        content: include_str!("templates/static-reader/index.html"),
        substitute: true,
    },
    TemplateFile {
        name: "sbfb-bridge.js",
        content: include_str!("templates/static-reader/sbfb-bridge.js"),
        substitute: false,
    },
    TemplateFile {
        name: "README.md",
        content: include_str!("templates/static-reader/README.md"),
        substitute: true,
    },
    TemplateFile {
        name: ".gitignore",
        content: include_str!("templates/static-reader/gitignore"),
        substitute: false,
    },
];

// Sprint 74 Phase C (PO Q7): a no-build React template. React/ReactDOM/htm are
// vendored same-origin (no CDN, no fetch) so the app runs under the SBFB sandbox
// CSP (default-src 'self'; connect-src 'none').
const REACT_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        name: "index.html",
        content: include_str!("templates/react/index.html"),
        substitute: true,
    },
    TemplateFile {
        name: "react.production.min.js",
        content: include_str!("templates/react/react.production.min.js"),
        substitute: false,
    },
    TemplateFile {
        name: "react-dom.production.min.js",
        content: include_str!("templates/react/react-dom.production.min.js"),
        substitute: false,
    },
    TemplateFile {
        name: "htm.umd.js",
        content: include_str!("templates/react/htm.umd.js"),
        substitute: false,
    },
    TemplateFile {
        name: "sbfb-bridge.js",
        content: include_str!("templates/react/sbfb-bridge.js"),
        substitute: false,
    },
    TemplateFile {
        name: "README.md",
        content: include_str!("templates/react/README.md"),
        substitute: true,
    },
    TemplateFile {
        name: ".gitignore",
        content: include_str!("templates/react/gitignore"),
        substitute: false,
    },
];

// Sprint 74 Phase C (PO Q7): an EXPERIMENTAL Pyodide scaffold. It does NOT run
// under the current sandbox CSP (connect-src 'none' blocks the Pyodide runtime
// fetch) — it is a starting point for a future extended-hosting mode. The
// README + an in-page banner say so honestly (no faux-functional app).
const PYODIDE_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        name: "index.html",
        content: include_str!("templates/pyodide/index.html"),
        substitute: true,
    },
    TemplateFile {
        name: "sbfb-bridge.js",
        content: include_str!("templates/pyodide/sbfb-bridge.js"),
        substitute: false,
    },
    TemplateFile {
        name: "README.md",
        content: include_str!("templates/pyodide/README.md"),
        substitute: true,
    },
    TemplateFile {
        name: ".gitignore",
        content: include_str!("templates/pyodide/gitignore"),
        substitute: false,
    },
];

// Sprint 79 Phase G: a daisyUI + anime.js starter. Unlike the React no-build
// template, daisyUI needs an ahead-of-time build (Tailwind v4 + daisyUI compiled
// into `app.css`); but the build OUTPUT is a static, same-origin stylesheet, so
// the published archive still has ZERO runtime dependency. anime.js v4.5.0 is
// vendored same-origin as a classic UMD `<script src>` (never `type=module`:
// COEP require-corp + opaque origin reject CORS-mode module fetches). The whole
// template loads under the sandbox CSP (`default-src 'self'; connect-src 'none'`)
// and passes the FG-CSP-authoring gate clean. First template with subdirectory
// entries (`src/`, `vendor/`, `scripts/`) — `create` materializes parents.
const DAISYUI_TEMPLATE: &[TemplateFile] = &[
    TemplateFile {
        name: "index.html",
        content: include_str!("templates/daisyui/index.html"),
        substitute: true,
    },
    TemplateFile {
        name: "app.js",
        content: include_str!("templates/daisyui/app.js"),
        substitute: false,
    },
    TemplateFile {
        name: "app.css",
        content: include_str!("templates/daisyui/app.css"),
        substitute: false,
    },
    TemplateFile {
        name: "src/input.css",
        content: include_str!("templates/daisyui/src/input.css"),
        substitute: false,
    },
    TemplateFile {
        name: "vendor/anime.umd.js",
        content: include_str!("templates/daisyui/vendor/anime.umd.js"),
        substitute: false,
    },
    TemplateFile {
        name: "scripts/vendor-anime.mjs",
        content: include_str!("templates/daisyui/scripts/vendor-anime.mjs"),
        substitute: false,
    },
    TemplateFile {
        name: "package.json",
        content: include_str!("templates/daisyui/package.json"),
        substitute: false,
    },
    TemplateFile {
        name: "README.md",
        content: include_str!("templates/daisyui/README.md"),
        substitute: true,
    },
    TemplateFile {
        name: ".gitignore",
        content: include_str!("templates/daisyui/gitignore"),
        substitute: false,
    },
];

fn substitute(content: &str, name: &str, version: &str) -> String {
    content
        .replace("{{name}}", name)
        .replace("{{version}}", version)
}

struct TemplateConfig {
    id: &'static str,
    version: &'static str,
    files: &'static [TemplateFile],
    description: &'static str,
    category: &'static str,
    bridge_methods: &'static [&'static str],
}

const TEMPLATES: &[TemplateConfig] = &[
    TemplateConfig {
        id: "static",
        version: "1.0.0",
        files: STATIC_TEMPLATE,
        description: "SBFB app created with sbfb-factory",
        category: "general",
        bridge_methods: &[],
    },
    TemplateConfig {
        id: "static-reader",
        version: "1.0.0",
        files: STATIC_READER_TEMPLATE,
        description: "SBFB reader app created with sbfb-factory",
        category: "content",
        bridge_methods: &["storage_get", "storage_set", "identity_pubkey"],
    },
    TemplateConfig {
        id: "react",
        version: "1.0.0",
        files: REACT_TEMPLATE,
        description: "SBFB React app (no-build, vendored UMD) created with sbfb-factory",
        category: "general",
        bridge_methods: &[],
    },
    TemplateConfig {
        id: "pyodide",
        version: "1.0.0",
        files: PYODIDE_TEMPLATE,
        description: "SBFB Python/Pyodide app (experimental scaffold) created with sbfb-factory",
        category: "general",
        bridge_methods: &[],
    },
    TemplateConfig {
        id: "daisyui",
        version: "1.0.0",
        files: DAISYUI_TEMPLATE,
        description: "SBFB daisyUI + anime.js app (vendored, CSP-safe) created with sbfb-factory",
        category: "general",
        bridge_methods: &[],
    },
];

fn find_template(id: &str) -> Result<&'static TemplateConfig, FactoryError> {
    TEMPLATES
        .iter()
        .find(|t| t.id == id)
        .ok_or_else(|| FactoryError::TemplateNotFound(id.to_string()))
}

pub fn create(template: &str, name: &str, output_dir: &str) -> Result<(), FactoryError> {
    let config = find_template(template)?;

    let out = Path::new(output_dir);
    fs::create_dir_all(out)?;

    let version = "0.1.0";

    let mut template_files: Vec<(String, String)> = Vec::new();
    for tf in config.files {
        let content = if tf.substitute {
            substitute(tf.content, name, version)
        } else {
            tf.content.to_string()
        };
        // `tf.name` may carry a forward-slash subpath (`src/input.css`,
        // `vendor/anime.umd.js`) — the daisyui template (Sprint 79 Phase G) is
        // the first non-flat one. `fs::write` does not create missing parents,
        // so materialize them first (the flat templates have no parent here).
        let dest = out.join(tf.name);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, &content)?;
        template_files.push((tf.name.to_string(), tf.content.to_string()));
    }

    let bridge = if config.bridge_methods.is_empty() {
        None
    } else {
        Some(sbfb_manifest::BridgeConfig {
            methods: config
                .bridge_methods
                .iter()
                .map(|s| s.to_string())
                .collect(),
        })
    };

    let manifest = SbfbManifest {
        schema_version: Some(2),
        name: Some(name.to_string()),
        version: Some(version.to_string()),
        description: Some(config.description.to_string()),
        category: Some(config.category.to_string()),
        node_id: None,
        repo_url: None,
        bridge,
    };
    manifest.validate()?;
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(out.join("SBFB.json"), &manifest_json)?;

    let lock = TemplateLock::generate(config.id, config.version, &template_files, name, version);
    fs::write(out.join("factory.template.lock"), lock.to_json()?)?;

    let variables = serde_json::json!({
        "name": name,
        "version": version,
    });
    let prov = Provenance::generate(out, &lock.template_hash, &variables)?;
    fs::write(out.join("factory.provenance.json"), prov.to_json()?)?;

    eprintln!("Created SBFB app '{}' in {}", name, output_dir);
    Ok(())
}

pub fn expected_files(
    template_id: &str,
    name: &str,
    version: &str,
) -> Result<Vec<(String, String)>, FactoryError> {
    let config = find_template(template_id)?;
    Ok(config
        .files
        .iter()
        .map(|tf| {
            let content = if tf.substitute {
                substitute(tf.content, name, version)
            } else {
                tf.content.to_string()
            };
            (tf.name.to_string(), content)
        })
        .collect())
}

pub fn validate(path: &str) -> Result<(), FactoryError> {
    let canonical = dunce::canonicalize(path)
        .map_err(|e| FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", path)))?;

    if !canonical.is_dir() {
        return Err(FactoryError::Validation(format!(
            "'{}' is not a directory",
            path
        )));
    }

    let mut issues = Vec::new();

    let manifest_path = canonical.join("SBFB.json");
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

    for entry in WalkDir::new(&canonical).follow_links(false) {
        let entry = entry?;
        if entry.path_is_symlink() {
            let rel = entry
                .path()
                .strip_prefix(&canonical)
                .unwrap_or(entry.path());
            issues.push(format!("symlink: {}", rel.display()));
        }
    }

    let findings = secret_scanner::scan_directory(&canonical);
    for f in &findings {
        let rel = f.file.strip_prefix(&canonical).unwrap_or(&f.file);
        issues.push(format!(
            "secret in {}:{}: {}",
            rel.display(),
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
    fn test_create_generates_provenance() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static", "prov-app", out.to_str().unwrap()).unwrap();

        let prov_path = out.join("factory.provenance.json");
        assert!(prov_path.exists());
        let prov: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(prov_path).unwrap()).unwrap();
        assert_eq!(prov["schema_version"], 1);
        assert!(prov["output_hash"].as_str().unwrap().len() == 64);
        assert!(prov["template_hash"].as_str().unwrap().len() == 64);
        assert!(prov["variables_hash"].as_str().unwrap().len() == 64);

        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("factory.template.lock")).unwrap())
                .unwrap();
        assert_eq!(prov["template_hash"], lock["template_hash"]);
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

    #[test]
    fn test_create_static_reader_template() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static-reader", "test-reader", out.to_str().unwrap()).unwrap();

        assert!(out.join("index.html").exists());
        assert!(out.join("sbfb-bridge.js").exists());
        assert!(out.join("SBFB.json").exists());
        assert!(out.join(".gitignore").exists());
        assert!(out.join("README.md").exists());
        assert!(out.join("factory.template.lock").exists());
        assert!(out.join("factory.provenance.json").exists());
    }

    #[test]
    fn test_validate_static_reader_passes() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static-reader", "valid-reader", out.to_str().unwrap()).unwrap();

        assert!(validate(out.to_str().unwrap()).is_ok());

        let json = fs::read_to_string(out.join("SBFB.json")).unwrap();
        let m = SbfbManifest::parse(&json).unwrap();
        assert_eq!(m.effective_schema_version(), 2);
        assert_eq!(m.category.as_deref(), Some("content"));
        assert!(m.bridge.is_some());
        let methods = &m.bridge.unwrap().methods;
        assert!(methods.contains(&"storage_get".to_string()));
        assert!(methods.contains(&"storage_set".to_string()));
        assert!(methods.contains(&"identity_pubkey".to_string()));
    }

    #[test]
    fn test_static_reader_template_substitution() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("static-reader", "babel-reader", out.to_str().unwrap()).unwrap();

        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("babel-reader"));
        assert!(!html.contains("{{name}}"));

        let readme = fs::read_to_string(out.join("README.md")).unwrap();
        assert!(readme.contains("babel-reader"));
        assert!(!readme.contains("{{name}}"));
        assert!(!readme.contains("{{version}}"));

        let lock: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(out.join("factory.template.lock")).unwrap())
                .unwrap();
        assert_eq!(lock["template_id"], "static-reader");
    }

    // -- Sprint 74 Phase C (PO Q7): react + pyodide templates --

    #[test]
    fn test_create_react_template() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("react", "test-react", out.to_str().unwrap()).unwrap();

        // The vendored runtime ships in the archive so the app runs under the
        // sandbox CSP (no CDN, no fetch).
        assert!(out.join("index.html").exists());
        assert!(out.join("react.production.min.js").exists());
        assert!(out.join("react-dom.production.min.js").exists());
        assert!(out.join("htm.umd.js").exists());
        assert!(out.join("sbfb-bridge.js").exists());
        assert!(out.join("SBFB.json").exists());
        assert!(out.join("README.md").exists());
        assert!(out.join(".gitignore").exists());
        assert!(out.join("factory.template.lock").exists());
        assert!(out.join("factory.provenance.json").exists());

        // React UMD license header preserved (real vendored runtime, not a stub).
        let react = fs::read_to_string(out.join("react.production.min.js")).unwrap();
        assert!(react.contains("@license React"));
    }

    #[test]
    fn test_react_template_substitution_and_no_cdn() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("react", "my-react-app", out.to_str().unwrap()).unwrap();

        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("my-react-app"));
        assert!(!html.contains("{{name}}"));
        // No CDN: React is loaded from a same-origin relative path, never an
        // absolute http(s) script src (which the sandbox CSP would block).
        assert!(html.contains("src=\"react.production.min.js\""));
        assert!(!html.to_lowercase().contains("unpkg"));
        assert!(
            !html.contains("src=\"http"),
            "no external/CDN script src — runtime is vendored same-origin"
        );
    }

    #[test]
    fn test_validate_react_passes() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("react", "valid-react", out.to_str().unwrap()).unwrap();
        assert!(validate(out.to_str().unwrap()).is_ok());

        let json = fs::read_to_string(out.join("SBFB.json")).unwrap();
        let m = SbfbManifest::parse(&json).unwrap();
        assert_eq!(m.effective_schema_version(), 2);
        assert_eq!(m.name.as_deref(), Some("valid-react"));
    }

    #[test]
    fn test_create_pyodide_template() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("pyodide", "test-py", out.to_str().unwrap()).unwrap();

        assert!(out.join("index.html").exists());
        assert!(out.join("sbfb-bridge.js").exists());
        assert!(out.join("SBFB.json").exists());
        assert!(out.join("README.md").exists());
        assert!(out.join(".gitignore").exists());
        assert!(out.join("factory.template.lock").exists());
    }

    #[test]
    fn test_pyodide_template_is_honest_experimental() {
        // PO Q7 + the frozen sandbox CSP: a Pyodide app cannot run under
        // connect-src 'none' (the runtime is fetched). The scaffold must say so
        // honestly — no faux-functional app (verrou "0 faux").
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("pyodide", "py-app", out.to_str().unwrap()).unwrap();

        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("py-app"));
        assert!(!html.contains("{{name}}"));
        // No external/CDN script src (the CSP would block it anyway).
        assert!(
            !html.contains("src=\"http"),
            "the pyodide scaffold must not load an external runtime"
        );

        let readme = fs::read_to_string(out.join("README.md")).unwrap();
        let readme_lower = readme.to_lowercase();
        assert!(
            readme_lower.contains("experimental") && readme_lower.contains("connect-src"),
            "the pyodide README must honestly document the sandbox limitation"
        );
    }

    #[test]
    fn test_validate_pyodide_passes() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("pyodide", "valid-py", out.to_str().unwrap()).unwrap();
        assert!(validate(out.to_str().unwrap()).is_ok());
    }

    #[test]
    fn test_create_daisyui_template() {
        // Sprint 79 Phase G: the first template with subdirectory entries. This
        // proves `create` materializes parent directories (`src/`, `vendor/`,
        // `scripts/`) — before the fix, `fs::write` panicked with NotFound on the
        // first subpath, on Windows and Linux alike.
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("daisyui", "test-daisy", out.to_str().unwrap()).unwrap();

        assert!(out.join("index.html").exists());
        assert!(out.join("app.js").exists());
        assert!(out.join("app.css").exists());
        assert!(out.join("src/input.css").exists(), "subdir file must exist");
        assert!(
            out.join("vendor/anime.umd.js").exists(),
            "vendored anime must exist"
        );
        assert!(out.join("scripts/vendor-anime.mjs").exists());
        assert!(out.join("package.json").exists());
        assert!(out.join("README.md").exists());
        assert!(out.join(".gitignore").exists());
        assert!(out.join("SBFB.json").exists());
        assert!(out.join("factory.template.lock").exists());
        assert!(out.join("factory.provenance.json").exists());

        // Substitution + no CDN: anime is loaded from a same-origin relative
        // path, never an absolute http(s) src, and never as an ES module.
        let html = fs::read_to_string(out.join("index.html")).unwrap();
        assert!(html.contains("test-daisy"));
        assert!(!html.contains("{{name}}"));
        assert!(html.contains("src=\"vendor/anime.umd.js\""));
        assert!(html.contains("data-theme=\"sbfb-reflect\""));
        assert!(
            !html.contains("src=\"http") && !html.to_lowercase().contains("type=\"module\""),
            "no CDN, no module script — runtime is vendored same-origin classic"
        );

        // The vendored anime bundle is the real v4.5.0 UMD (license header kept).
        let anime = fs::read_to_string(out.join("vendor/anime.umd.js")).unwrap();
        assert!(anime.contains("anime.js v4.5.0"));
    }

    #[test]
    fn test_daisyui_template_no_false_eight_themes() {
        // Sprint 79 Phase G (PLAN-ADAPT): the plan said "8 themes removed", which
        // Phase F proved false (daisyUI 5.5.23 ships 35 built-in themes; the lean
        // template activates none of them). Guard the corrected wording so the
        // stale "8 themes" claim never reappears in the shipped artifacts.
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("daisyui", "lean-themes", out.to_str().unwrap()).unwrap();

        let input_css = fs::read_to_string(out.join("src/input.css")).unwrap();
        let readme = fs::read_to_string(out.join("README.md")).unwrap();
        for doc in [&input_css, &readme] {
            assert!(
                !doc.contains("8 themes") && !doc.contains("8 thèmes"),
                "the stale '8 themes' claim must not appear"
            );
        }
        // The lean config loads daisyUI with zero built-in themes + only the
        // custom oklch theme.
        assert!(input_css.contains("themes: false"));
        assert!(input_css.contains("sbfb-reflect"));
        assert!(readme.contains("35 built-in"));
    }

    #[test]
    fn test_daisyui_package_json_pins_resolved_versions() {
        // Day-0 #10: pin the resolved build-time versions exactly (no carets), so
        // the compiled `app.css` is reproducible.
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("daisyui", "pinned", out.to_str().unwrap()).unwrap();

        let pkg = fs::read_to_string(out.join("package.json")).unwrap();
        assert!(
            !pkg.contains('^') && !pkg.contains('~'),
            "no caret/tilde ranges"
        );
        for pin in [
            "\"daisyui\": \"5.5.23\"",
            "\"tailwindcss\": \"4.3.1\"",
            "\"@tailwindcss/cli\": \"4.3.1\"",
            "\"@tailwindcss/node\": \"4.3.1\"",
            "\"@tailwindcss/oxide\": \"4.3.1\"",
            "\"animejs\": \"4.5.0\"",
        ] {
            assert!(pkg.contains(pin), "missing exact pin: {pin}");
        }
    }

    #[test]
    fn test_validate_daisyui_passes() {
        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("app");
        create("daisyui", "valid-daisy", out.to_str().unwrap()).unwrap();
        assert!(validate(out.to_str().unwrap()).is_ok());

        let json = fs::read_to_string(out.join("SBFB.json")).unwrap();
        let m = SbfbManifest::parse(&json).unwrap();
        assert_eq!(m.effective_schema_version(), 2);
        assert_eq!(m.name.as_deref(), Some("valid-daisy"));
    }
}
