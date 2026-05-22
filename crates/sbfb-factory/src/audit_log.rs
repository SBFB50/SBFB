// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub timestamp: String,
    pub command: String,
    pub args: Vec<String>,
    pub result: String,
}

pub fn audit_log_path() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".sbfb").join("factory-audit.log"))
        .unwrap_or_else(|| PathBuf::from("factory-audit.log"))
}

pub fn log_entry(entry: &AuditEntry) -> Result<(), Box<dyn std::error::Error>> {
    log_entry_to(&audit_log_path(), entry)
}

pub fn log_entry_to(
    path: &std::path::Path,
    entry: &AuditEntry,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(entry)?;
    writeln!(file, "{line}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;

    fn make_entry(cmd: &str) -> AuditEntry {
        AuditEntry {
            timestamp: "2026-05-22T12:00:00Z".to_string(),
            command: cmd.to_string(),
            args: vec!["--name".to_string(), "test".to_string()],
            result: "success".to_string(),
        }
    }

    #[test]
    fn audit_log_writes_jsonl() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("factory-audit.log");

        let entry = make_entry("create");
        log_entry_to(&path, &entry).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(content.trim()).unwrap();
        assert_eq!(parsed["command"], "create");
        assert_eq!(parsed["result"], "success");
    }

    #[test]
    fn audit_log_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("factory-audit.log");

        log_entry_to(&path, &make_entry("create")).unwrap();
        log_entry_to(&path, &make_entry("validate")).unwrap();

        let file = fs::File::open(&path).unwrap();
        let lines: Vec<String> = std::io::BufReader::new(file)
            .lines()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(lines.len(), 2);
        let first: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
        let second: serde_json::Value = serde_json::from_str(&lines[1]).unwrap();
        assert_eq!(first["command"], "create");
        assert_eq!(second["command"], "validate");
    }
}
