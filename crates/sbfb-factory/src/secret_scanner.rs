// SPDX-License-Identifier: AGPL-3.0-or-later

use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct SecretFinding {
    pub file: PathBuf,
    pub line: usize,
    pub pattern_name: String,
}

struct Pattern {
    name: &'static str,
    regex: &'static str,
}

const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "AWS access key",
        regex: r"AKIA[0-9A-Z]{16}",
    },
    Pattern {
        name: "GitHub token",
        regex: r"(ghp|gho|ghs|ghr)_[A-Za-z0-9_]{36,}",
    },
    Pattern {
        name: "PEM private key",
        regex: r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
    },
];

pub fn scan_directory(dir: &Path) -> Vec<SecretFinding> {
    let compiled: Vec<(&str, Regex)> = PATTERNS
        .iter()
        .filter_map(|p| Regex::new(p.regex).ok().map(|r| (p.name, r)))
        .collect();

    let mut findings = Vec::new();

    for entry in WalkDir::new(dir).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let content = match fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            for (name, re) in &compiled {
                if re.is_match(line) {
                    findings.push(SecretFinding {
                        file: entry.path().to_path_buf(),
                        line: line_num + 1,
                        pattern_name: (*name).to_string(),
                    });
                }
            }
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_secret_scanner_detects_aws_key() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("config.txt"),
            "aws_key = AKIAIOSFODNN7EXAMPLE",
        )
        .unwrap();

        let findings = scan_directory(tmp.path());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_name, "AWS access key");
    }

    #[test]
    fn test_secret_scanner_detects_pem_private_key() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("key.pem"),
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAK...\n-----END RSA PRIVATE KEY-----",
        )
        .unwrap();

        let findings = scan_directory(tmp.path());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_name, "PEM private key");
    }

    #[test]
    fn test_secret_scanner_detects_github_token() {
        let tmp = TempDir::new().unwrap();
        fs::write(
            tmp.path().join("env"),
            "GITHUB_TOKEN=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmn",
        )
        .unwrap();

        let findings = scan_directory(tmp.path());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern_name, "GitHub token");
    }
}
