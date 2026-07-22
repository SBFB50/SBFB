// SPDX-License-Identifier: AGPL-3.0-or-later
//! Test-only support: the committed golden workflows and their pinned
//! artefact stores (`src/workflow_golden_artifacts/<name>/`), shared by the
//! [`crate::graph`] language tests (Phase G) and the [`crate::compile`]
//! compiler tests (Phase H). Compiled only under `#[cfg(test)]` — nothing
//! here exists in the production crate.

use std::path::{Path, PathBuf};

use crate::agent_definition::SbfbAgentDefinition;
use crate::graph::SbfbWorkflowGraph;

/// The four committed golden workflows, factory FIRST (H.2: the factory
/// golden compiles first and is held to the committed process profile).
pub(crate) const GOLDEN_NAMES: [&str; 4] = ["factory", "ideas", "conception", "babel"];

pub(crate) fn src_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

pub(crate) fn golden_path(name: &str) -> PathBuf {
    src_dir().join(format!("{name}.workflow.golden.json"))
}

/// The per-golden pinned artefact store
/// (`src/workflow_golden_artifacts/<name>/`).
pub(crate) struct ArtifactStore {
    pub(crate) dir: PathBuf,
}

impl ArtifactStore {
    pub(crate) fn open(name: &str) -> Self {
        Self {
            dir: src_dir().join("workflow_golden_artifacts").join(name),
        }
    }

    pub(crate) fn file(&self, category: &str, id: &str, ext: &str) -> PathBuf {
        self.dir.join(category).join(format!("{id}.{ext}"))
    }

    /// The `PACK.lock` pin of one artefact file: BLAKE3 hex of the
    /// checkout-newline-canonicalised bytes. A real pack hashes raw
    /// archive bytes (archives never ride git's newline translation);
    /// committed fixtures do, so they reuse the
    /// [`crate::factory_profile`] source-digest canonicalisation.
    pub(crate) fn pin(&self, category: &str, id: &str, ext: &str) -> String {
        let path = self.file(category, id, ext);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("golden artefact {} must exist ({e})", path.display()));
        blake3::hash(&crate::factory_profile::canonicalise(&bytes))
            .to_hex()
            .to_string()
    }

    /// Ids of the artefacts of one category, by filename suffix, sorted.
    pub(crate) fn ids(&self, category: &str, suffix: &str) -> Vec<String> {
        let dir = self.dir.join(category);
        if !dir.exists() {
            return Vec::new();
        }
        let mut out: Vec<String> = std::fs::read_dir(&dir)
            .expect("store category readable")
            .map(|entry| entry.expect("dir entry").file_name())
            .filter_map(|n| {
                n.to_str()
                    .and_then(|n| n.strip_suffix(suffix))
                    .map(str::to_string)
            })
            .collect();
        out.sort();
        out
    }
}

pub(crate) fn parse_json(path: &Path) -> serde_json::Value {
    let bytes =
        std::fs::read(path).unwrap_or_else(|e| panic!("{} must exist ({e})", path.display()));
    serde_json::from_slice(&bytes)
        .unwrap_or_else(|e| panic!("{} must be valid JSON ({e})", path.display()))
}

pub(crate) fn parse_graph(path: &Path) -> (serde_json::Value, SbfbWorkflowGraph) {
    let value = parse_json(path);
    let graph = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("{} must parse strictly ({e})", path.display()));
    (value, graph)
}

pub(crate) fn parse_agent(path: &Path) -> (serde_json::Value, SbfbAgentDefinition) {
    let value = parse_json(path);
    let agent = serde_json::from_value(value.clone())
        .unwrap_or_else(|e| panic!("{} must parse strictly ({e})", path.display()));
    (value, agent)
}

pub(crate) fn write_pretty<T: serde::Serialize>(path: &Path, value: &T) {
    let text = serde_json::to_string_pretty(&serde_json::to_value(value).expect("serialises"))
        .expect("pretty-prints");
    std::fs::write(path, format!("{text}\n")).expect("golden written");
}
