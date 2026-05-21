// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::template_engine::{self, FactoryError};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, PartialEq, Eq)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug)]
pub struct DiffEntry {
    pub path: String,
    pub status: DiffStatus,
}

const METADATA_FILES: &[&str] = &[
    "factory.template.lock",
    "factory.provenance.json",
    "SBFB.json",
];

pub fn diff_workspace(workspace: &Path) -> Result<Vec<DiffEntry>, FactoryError> {
    let lock_path = workspace.join("factory.template.lock");
    if !lock_path.exists() {
        return Err(FactoryError::Validation(
            "factory.template.lock not found — was this project created with sbfb-factory?".into(),
        ));
    }

    let lock_content = fs::read_to_string(&lock_path)?;
    let lock: serde_json::Value = serde_json::from_str(&lock_content)?;

    let template_id = lock["template_id"].as_str().unwrap_or("static");
    let name = lock["variables"]["name"].as_str().unwrap_or("");
    let version = lock["variables"]["version"].as_str().unwrap_or("0.1.0");

    let expected = template_engine::expected_files(template_id, name, version)?;
    let expected_names: BTreeSet<&str> = expected.iter().map(|(n, _)| n.as_str()).collect();

    let mut entries = Vec::new();

    for entry in WalkDir::new(workspace).follow_links(false).min_depth(1) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(workspace)
            .unwrap_or(entry.path())
            .to_string_lossy()
            .replace('\\', "/");

        if rel.starts_with('.') || METADATA_FILES.contains(&rel.as_str()) {
            continue;
        }

        if let Some((_, expected_content)) = expected.iter().find(|(n, _)| n == &rel) {
            let actual = fs::read_to_string(entry.path())?;
            if actual != *expected_content {
                entries.push(DiffEntry {
                    path: rel,
                    status: DiffStatus::Modified,
                });
            }
        } else {
            entries.push(DiffEntry {
                path: rel,
                status: DiffStatus::Added,
            });
        }
    }

    for name in &expected_names {
        if name.starts_with('.') {
            continue;
        }
        if !workspace.join(name).exists() {
            entries.push(DiffEntry {
                path: (*name).to_string(),
                status: DiffStatus::Deleted,
            });
        }
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
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
    fn test_diff_detects_added_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);

        fs::write(workspace.join("extra.js"), "console.log('added')").unwrap();

        let diff = diff_workspace(&workspace).unwrap();
        let added: Vec<_> = diff
            .iter()
            .filter(|e| e.status == DiffStatus::Added)
            .collect();
        assert!(
            added.iter().any(|e| e.path == "extra.js"),
            "expected extra.js as added, got: {added:?}"
        );
    }

    #[test]
    fn test_diff_detects_modified_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);

        fs::write(workspace.join("index.html"), "<html>modified</html>").unwrap();

        let diff = diff_workspace(&workspace).unwrap();
        let modified: Vec<_> = diff
            .iter()
            .filter(|e| e.status == DiffStatus::Modified)
            .collect();
        assert!(
            modified.iter().any(|e| e.path == "index.html"),
            "expected index.html as modified, got: {modified:?}"
        );
    }

    #[test]
    fn test_diff_detects_deleted_file() {
        let tmp = TempDir::new().unwrap();
        let workspace = create_factory_project(&tmp);

        fs::remove_file(workspace.join("README.md")).unwrap();

        let diff = diff_workspace(&workspace).unwrap();
        let deleted: Vec<_> = diff
            .iter()
            .filter(|e| e.status == DiffStatus::Deleted)
            .collect();
        assert!(
            deleted.iter().any(|e| e.path == "README.md"),
            "expected README.md as deleted, got: {deleted:?}"
        );
    }
}
