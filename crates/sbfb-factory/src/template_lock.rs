// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TemplateLock {
    pub template_id: String,
    pub template_version: String,
    pub template_hash: String,
    pub generated_at: String,
    pub variables: serde_json::Value,
}

impl TemplateLock {
    pub fn generate(
        template_id: &str,
        template_version: &str,
        files: &[(String, String)],
        name: &str,
        version: &str,
    ) -> Self {
        let template_hash = compute_hash(files);
        let generated_at = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            template_id: template_id.to_string(),
            template_version: template_version.to_string(),
            template_hash,
            generated_at,
            variables: serde_json::json!({
                "name": name,
                "version": version,
            }),
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

fn compute_hash(files: &[(String, String)]) -> String {
    let mut hasher = blake3::Hasher::new();
    let mut sorted: Vec<_> = files.iter().collect();
    sorted.sort_by_key(|(name, _)| name.as_str());
    for (name, content) in sorted {
        hasher.update(name.as_bytes());
        hasher.update(content.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}
