// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::diff;
use crate::secret_scanner;
use crate::template_engine::FactoryError;
use nexus_core_rs::canonical::DOMAIN_PROVENANCE_V1;
use nexus_core_rs::csp::CSS_URL_ALLOW;
use regex::Regex;
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

// =================================================================
// FG-CSP-authoring — static CSP conformance gate (Sprint 79 Phase E)
// =================================================================

/// One CSP directive the authoring gate enforces, plus the regex patterns
/// that flag a runtime asset breaching it. `directive` mirrors a CSP
/// directive name; the cross-crate coverage test asserts every directive
/// `nexus_core_rs::csp::BLOB_SERVE_CSP` sets to `'none'` has an entry here, so
/// a future edit that adds a `'none'` directive to the policy fails the build
/// until a detection rule is added (anti-drift).
struct CspRule {
    /// CSP directive this rule enforces (e.g. `"connect-src"`).
    directive: &'static str,
    /// `(pattern, human label)` pairs applied to asset text. HTML/CSS patterns
    /// carry an inline `(?i)` (attributes/`url()` are case-insensitive); bare
    /// JS-identifier patterns are intentionally case-sensitive (`fetch` is the
    /// API, `Fetch` is a user symbol) and use `\b` to avoid `prefetcher`.
    patterns: &'static [(&'static str, &'static str)],
}

/// Detection rules. The six `'none'` directives of `BLOB_SERVE_CSP` are each
/// represented (coverage asserted by `test_csp_gate_covers_every_none_directive`);
/// the extra `default-src` entry forbids remote resource loads (absolute
/// `https?:` and protocol-relative `//host`) that the restrictive
/// `default-src 'self' ... data: blob:` allowlist blocks at runtime.
const CSP_RULES: &[CspRule] = &[
    CspRule {
        directive: "connect-src",
        patterns: &[
            (r"\bfetch\s*\(", "fetch()"),
            (r"\bXMLHttpRequest\b", "XMLHttpRequest"),
            (r"\bWebSocket\b", "WebSocket"),
            (r"\bEventSource\b", "EventSource"),
            (r"navigator\.sendBeacon\b", "navigator.sendBeacon"),
        ],
    },
    CspRule {
        directive: "worker-src",
        patterns: &[
            (r"new\s+Worker\b", "Web Worker"),
            (r"new\s+SharedWorker\b", "SharedWorker"),
            (r"\bimportScripts\s*\(", "importScripts"),
            (r"navigator\.serviceWorker", "Service Worker"),
        ],
    },
    CspRule {
        // Tag-presence patterns require a boundary char `[\s/>]` after the tag
        // name so a custom element (`<iframe-foo>`, `<object-x>`) is not a
        // false positive (custom element names always contain a hyphen).
        directive: "frame-src",
        patterns: &[
            (r"(?i)<iframe[\s/>]", "<iframe> (nested frame)"),
            (
                r#"(?i)createElement\(\s*["']iframe["']"#,
                "createElement('iframe')",
            ),
        ],
    },
    CspRule {
        directive: "object-src",
        patterns: &[
            (r"(?i)<object[\s/>]", "<object>"),
            (r"(?i)<embed[\s/>]", "<embed>"),
        ],
    },
    CspRule {
        // Attribute patterns require a `[\s/]` boundary after the tag name so
        // `<base-x>`/`<form-x>` custom elements do NOT match while the HTML
        // slash-separator quirk (`<base/href=…>`, `<script/src=…>`) still does.
        directive: "base-uri",
        patterns: &[
            (
                r"(?i)<base[\s/][^>]*href\s*=",
                "<base href> (base-uri hijack)",
            ),
            (
                r#"(?i)createElement\(\s*["']base["']"#,
                "createElement('base')",
            ),
        ],
    },
    CspRule {
        // `form-action 'none'` blocks form-based exfiltration `connect-src`
        // does not cover. The gate flags forms whose action targets a remote
        // origin (the exfil shape) + dynamic `setAttribute('action', …)`.
        // Relative/empty actions cannot exfiltrate cross-origin and are also
        // stopped by the runtime CSP + the iframe `sandbox` (no `allow-forms`),
        // so they are left to the Phase H runtime self-check (low false-positive).
        directive: "form-action",
        patterns: &[
            (
                r#"(?i)<form[\s/][^>]*action\s*=\s*["']?(?:https?:|//)"#,
                "<form action> to remote URL",
            ),
            (
                r#"(?i)\.setAttribute\(\s*["']action["']"#,
                "setAttribute('action', …)",
            ),
        ],
    },
    CspRule {
        directive: "default-src",
        patterns: &[
            (
                r#"(?i)<link[\s/][^>]*href\s*=\s*["']?(?:https?:|//)"#,
                "remote <link href>",
            ),
            (
                r#"(?i)<script[\s/][^>]*src\s*=\s*["']?(?:https?:|//)"#,
                "remote <script src>",
            ),
            // Only REMOTE @import is a violation — a relative `@import
            // url('./local.css')` / `@import "base.css"` resolves same-origin
            // ('self') and is allowed at runtime, so it must not false-positive.
            (
                r#"(?i)@import\s+url\(\s*["']?(?:https?:|//)"#,
                "remote CSS @import url()",
            ),
            (r#"(?i)@import\s+["'](?:https?:|//)"#, "remote CSS @import"),
            (r#"(?i)url\(\s*["']?(?:https?:|//)"#, "remote url() asset"),
        ],
    },
];

/// COEP `require-corp` + opaque origin: ES module scripts are fetched in CORS
/// mode and fail. Vendored bundles must load as classic `<script src>`.
/// `[\s/]` after the tag name catches the `<script/...>` slash-separator quirk.
const MODULE_SCRIPT_PATTERN: &str = r#"(?i)<script[\s/][^>]*type\s*=\s*["']module["']"#;

/// Any absolute `http(s)` URL — the catch-all for remote URLs in scanned
/// source. Mirrors `check-csp.mjs`'s `https?://[^\s"')]*`, additionally
/// excluding `>` so an unquoted attribute URL stops at the tag close.
/// Allowlisted non-fetched identifiers (`CSS_URL_ALLOW`) are exempt — see
/// [`url_allowlisted`].
const ABSOLUTE_URL_PATTERN: &str = r#"https?://[^\s"')>]*"#;

/// True if an absolute URL is an allowlisted non-fetched identifier. Matches
/// on an origin/path boundary (exact, or the allowlisted prefix followed by
/// `/`) so a look-alike host such as `https://tailwindcss.com.evil.com/x` is
/// NOT allowed even though it shares the prefix.
fn url_allowlisted(url: &str) -> bool {
    CSS_URL_ALLOW.iter().any(|a| {
        url == *a
            || url
                .strip_prefix(*a)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

#[derive(PartialEq, Eq)]
enum CspTier {
    /// Authored source (`*.html`/`*.js`/`*.css` that is neither under `vendor/`
    /// nor a recognized third-party bundle): zero network primitive, no
    /// `<script type=module>`, and every absolute URL must be allowlisted.
    Scanned,
    /// Vendored third-party bundles — either under a `vendor/` directory or
    /// named `*.umd.js` / `*.min.js` (the SBFB react template ships
    /// `react-dom.production.min.js` / `htm.umd.js` at the project root; the
    /// daisyui template uses `vendor/anime.umd.js`). Only live network
    /// primitives are forbidden — minified bodies legitimately carry XML
    /// namespace strings and license-banner URLs that are not in
    /// `CSS_URL_ALLOW`. The network-primitive check still applies, so a
    /// vendored bundle that actually calls `fetch`/opens a `WebSocket` is
    /// still rejected.
    Vendored,
    /// Not a runtime web asset (images, fonts, `SBFB.json`, `factory.*.json`,
    /// README, gitignore).
    Skip,
}

fn classify_csp_tier(rel: &Path) -> CspTier {
    let is_web = rel
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .is_some_and(|e| matches!(e.as_str(), "html" | "htm" | "js" | "mjs" | "css"));
    if !is_web {
        return CspTier::Skip;
    }
    let file_name = rel
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase())
        .unwrap_or_default();
    let is_vendored = rel
        .components()
        .any(|c| c.as_os_str().eq_ignore_ascii_case("vendor"))
        || file_name.ends_with(".umd.js")
        || file_name.ends_with(".min.js");
    if is_vendored {
        CspTier::Vendored
    } else {
        CspTier::Scanned
    }
}

/// FG-CSP-authoring — static, deterministic conformance gate (Sprint 79
/// Phase E). Scans an app workspace's runtime assets and FAILS if any would
/// breach the blob-serve sandbox CSP. The policy is imported from
/// `nexus_core_rs::csp::BLOB_SERVE_CSP` as the single source of truth — never
/// re-hardcoded.
///
/// This is a **surface / authoring-discipline** gate, not a proof of
/// non-exfiltration: a regex scanner cannot see network access assembled at
/// runtime (`fetch` via `atob`, `el.action`/`base.href` built dynamically,
/// CSS `url()` injected by JS). Those are caught by (a) the runtime CSP the
/// daemon injects on every blob-serve response, and (b) the runtime self-check
/// that replays the app under the real CSP (Sprint 79 Phase H). The gate
/// guarantees the *delivered static assets* are clean and gives the author an
/// immediate publish-time diagnostic.
///
/// Three tiers (mirrors the JS lint `check-csp.mjs`): scanned source
/// (`*.html`/`*.js`/`*.css` that is neither under `vendor/` nor a recognized
/// bundle) — zero network primitive + every absolute URL allowlisted
/// (`CSS_URL_ALLOW`) + no `<script type=module>`; vendored (`vendor/*` or
/// `*.umd.js`/`*.min.js`) — zero network primitive only; everything else
/// skipped.
pub fn run_gate_csp_authoring(workspace: &Path) -> Result<GateResult, FactoryError> {
    let canonical = dunce::canonicalize(workspace).map_err(|e| {
        FactoryError::PathTraversal(format!("cannot resolve '{}': {e}", workspace.display()))
    })?;
    if !canonical.is_dir() {
        return Ok(GateResult::fail(
            "FG-CSP-authoring",
            vec![format!("'{}' is not a directory", workspace.display())],
        ));
    }

    // Compile detection rules once (mirrors `secret_scanner`). A bad pattern is
    // a programmer bug — surface it as a gate issue rather than panicking.
    let mut compiled: Vec<(&'static str, Regex, &'static str)> = Vec::new();
    for rule in CSP_RULES {
        for (pat, label) in rule.patterns {
            match Regex::new(pat) {
                Ok(re) => compiled.push((rule.directive, re, label)),
                Err(e) => {
                    return Ok(GateResult::fail(
                        "FG-CSP-authoring",
                        vec![format!("internal: bad CSP rule /{pat}/: {e}")],
                    ));
                }
            }
        }
    }
    let module_script = Regex::new(MODULE_SCRIPT_PATTERN)
        .map_err(|e| FactoryError::Validation(format!("internal: module-script regex: {e}")))?;
    let abs_url = Regex::new(ABSOLUTE_URL_PATTERN)
        .map_err(|e| FactoryError::Validation(format!("internal: absolute-url regex: {e}")))?;

    let mut issues = Vec::new();
    for entry in WalkDir::new(&canonical).follow_links(false) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(&canonical)
            .unwrap_or(entry.path());
        let tier = classify_csp_tier(rel);
        if tier == CspTier::Skip {
            continue;
        }
        let text = match fs::read_to_string(entry.path()) {
            Ok(t) => t,
            // Binary / non-UTF8 asset (image, font): nothing textual to scan.
            Err(_) => continue,
        };

        // Network-reaching primitives are forbidden in every scanned tier.
        for (directive, re, label) in &compiled {
            if re.is_match(&text) {
                issues.push(format!("{}: {label} ({directive})", rel.display()));
            }
        }

        // Scanned source additionally forbids module scripts (COEP) and any
        // non-allowlisted absolute URL.
        if tier == CspTier::Scanned {
            if module_script.is_match(&text) {
                issues.push(format!(
                    "{}: <script type=module> — ES modules fail under COEP require-corp; vendor as classic <script src>",
                    rel.display()
                ));
            }
            for m in abs_url.find_iter(&text) {
                let url = m.as_str();
                if !url_allowlisted(url) {
                    issues.push(format!(
                        "{}: non-allowlisted absolute URL: {url}",
                        rel.display()
                    ));
                }
            }
        }
    }

    if issues.is_empty() {
        Ok(GateResult::pass("FG-CSP-authoring"))
    } else {
        Ok(GateResult::fail("FG-CSP-authoring", issues))
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

    // --- FG-CSP-authoring (Sprint 79 Phase E) ---

    fn csp_workspace(tmp: &TempDir, files: &[(&str, &str)]) -> std::path::PathBuf {
        let root = tmp.path().join("app");
        for (rel, content) in files {
            let p = root.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, content).unwrap();
        }
        root
    }

    #[test]
    fn test_csp_gate_clean_workspace_passes() {
        let tmp = TempDir::new().unwrap();
        let ws = csp_workspace(
            &tmp,
            &[
                (
                    "index.html",
                    r#"<!doctype html><html><body>
                       <svg xmlns="http://www.w3.org/2000/svg"></svg>
                       <script src="vendor/anime.umd.js"></script>
                       <script src="app.js"></script></body></html>"#,
                ),
                ("app.js", "document.title = 'ok';"),
                (
                    "app.css",
                    "/*! tailwindcss v4 | MIT License | https://tailwindcss.com */\n.btn{color:#fff}",
                ),
                (
                    "vendor/anime.umd.js",
                    "/* anime.js */ var NS = 'http://www.w3.org/2000/svg';",
                ),
            ],
        );
        let r = run_gate_csp_authoring(&ws).unwrap();
        assert!(
            r.passed,
            "clean daisyui/anime workspace should pass FG-CSP-authoring: {:?}",
            r.issues
        );
    }

    #[test]
    fn test_csp_gate_rejects_violations() {
        // (case name, file, content, expected issue substring)
        let cases: &[(&str, &str, &str, &str)] = &[
            (
                "fetch",
                "app.js",
                "const x = fetch('https://api.example/y');",
                "fetch",
            ),
            (
                "remote script",
                "index.html",
                r#"<script src="https://cdn.example/a.js"></script>"#,
                "script",
            ),
            (
                "protocol-relative script",
                "index.html",
                r#"<script src="//cdn.example/a.js"></script>"#,
                "script",
            ),
            (
                "websocket",
                "app.js",
                "const s = new WebSocket('wss://x.example');",
                "WebSocket",
            ),
            (
                "web worker",
                "app.js",
                "const w = new Worker('w.js');",
                "Worker",
            ),
            (
                "form remote action",
                "index.html",
                r#"<form action="https://attacker.example/c"></form>"#,
                "form",
            ),
            (
                "base href hijack",
                "index.html",
                r#"<base href="https://evil.example/">"#,
                "base",
            ),
            (
                "object",
                "index.html",
                "<object data='x'></object>",
                "object",
            ),
            ("embed", "index.html", "<embed src='x'>", "embed"),
            (
                "nested iframe",
                "index.html",
                "<iframe src='n.html'></iframe>",
                "iframe",
            ),
            (
                "css @import",
                "app.css",
                "@import url('https://fonts.example/f.css');",
                "import",
            ),
            (
                "css remote url",
                "app.css",
                ".x{background:url(https://evil.example/p.png)}",
                "url",
            ),
            (
                "type=module script",
                "index.html",
                r#"<script type="module" src="app.js"></script>"#,
                "module",
            ),
            (
                "vendored live network",
                "vendor/lib.js",
                "fetch('/data');",
                "fetch",
            ),
            (
                "non-allowlisted absolute url",
                "app.js",
                "const u = 'https://evil.example/';",
                "absolute URL",
            ),
            // One positive trigger per individual pattern label (not just per
            // directive) so a broken `(?i)`/`\b`/group on any single regex is
            // caught — the directive-coverage test only guards rule presence.
            (
                "xhr",
                "app.js",
                "const x = new XMLHttpRequest();",
                "XMLHttpRequest",
            ),
            (
                "event source",
                "app.js",
                "const s = new EventSource('/stream');",
                "EventSource",
            ),
            (
                "send beacon",
                "app.js",
                "navigator.sendBeacon('/log', d);",
                "sendBeacon",
            ),
            (
                "shared worker",
                "app.js",
                "const w = new SharedWorker('w.js');",
                "SharedWorker",
            ),
            (
                "import scripts",
                "app.js",
                "importScripts('x.js');",
                "importScripts",
            ),
            (
                "service worker",
                "app.js",
                "navigator.serviceWorker.register('sw.js');",
                "Service Worker",
            ),
            (
                "createElement iframe",
                "app.js",
                "document.createElement('iframe');",
                "createElement('iframe')",
            ),
            (
                "createElement base",
                "app.js",
                "document.createElement('base');",
                "createElement('base')",
            ),
            (
                "setAttribute action",
                "app.js",
                "el.setAttribute('action', remote);",
                "setAttribute('action'",
            ),
            (
                "remote link href",
                "index.html",
                r#"<link href="https://fonts.example/f.css" rel="stylesheet">"#,
                "link",
            ),
            // HTML slash-separator quirk: `<script/src=…>` is a valid remote
            // script load (no whitespace after the tag name) — must be caught.
            (
                "slash-separator script src",
                "index.html",
                "<script/src=//evil.example/a.js></script>",
                "script",
            ),
            (
                "slash-separator base href",
                "index.html",
                "<base/href=//evil.example/>",
                "base",
            ),
            // Guard the `[^>]*` group: the network attribute is NOT immediately
            // after the tag name — other attributes precede it.
            (
                "attrs before form action",
                "index.html",
                r#"<form class="x" method="post" action="https://evil.example/c"></form>"#,
                "form",
            ),
            (
                "attrs before link href",
                "index.html",
                r#"<link rel="stylesheet" href="https://evil.example/x.css">"#,
                "link",
            ),
        ];
        for (name, rel, content, needle) in cases {
            let tmp = TempDir::new().unwrap();
            let ws = csp_workspace(&tmp, &[(rel, content)]);
            let r = run_gate_csp_authoring(&ws).unwrap();
            assert!(!r.passed, "case '{name}' should FAIL FG-CSP-authoring");
            let needle_lc = needle.to_lowercase();
            assert!(
                r.issues
                    .iter()
                    .any(|i| i.to_lowercase().contains(&needle_lc)),
                "case '{name}': issues {:?} should mention '{needle}'",
                r.issues
            );
        }
    }

    #[test]
    fn test_csp_gate_local_css_import_passes() {
        // A LOCAL (relative same-origin) @import is CSP-allowed (default-src
        // 'self') and must NOT false-positive — only REMOTE @imports are blocked.
        let tmp = TempDir::new().unwrap();
        let ws = csp_workspace(
            &tmp,
            &[
                (
                    "index.html",
                    "<!doctype html><link rel=\"stylesheet\" href=\"app.css\">",
                ),
                (
                    "app.css",
                    "@import url('./theme.css');\n@import \"base.css\";\n.x{color:#fff}",
                ),
                ("theme.css", ".t{color:#000}"),
                ("base.css", ".b{margin:0}"),
            ],
        );
        let r = run_gate_csp_authoring(&ws).unwrap();
        assert!(
            r.passed,
            "local relative @import must pass (default-src 'self'): {:?}",
            r.issues
        );
    }

    #[test]
    fn test_csp_gate_case_sensitivity() {
        // JS-identifier patterns are case-sensitive (`fetch` is the API): a
        // user symbol `Fetch` and the word `prefetcher` must NOT false-positive.
        let tmp = TempDir::new().unwrap();
        let ws = csp_workspace(
            &tmp,
            &[(
                "app.js",
                "const Fetch = makeClient(); const p = prefetcher(); Fetch.run();",
            )],
        );
        let r = run_gate_csp_authoring(&ws).unwrap();
        assert!(
            r.passed,
            "case-sensitive JS identifiers must not false-positive: {:?}",
            r.issues
        );

        // HTML attributes are case-insensitive (`(?i)`): an uppercase
        // `<FORM ACTION=remote>` must still be caught.
        let tmp2 = TempDir::new().unwrap();
        let ws2 = csp_workspace(
            &tmp2,
            &[(
                "index.html",
                r#"<FORM ACTION="https://attacker.example/c"></FORM>"#,
            )],
        );
        let r2 = run_gate_csp_authoring(&ws2).unwrap();
        assert!(
            !r2.passed,
            "uppercase <FORM ACTION=remote> must be caught via (?i)"
        );
    }

    #[test]
    fn test_csp_gate_allowlist_matches_on_origin_boundary() {
        // A look-alike host that merely shares the allowlisted prefix must NOT
        // be allowlisted (the `starts_with` bypass): `tailwindcss.com.evil.com`.
        let tmp = TempDir::new().unwrap();
        let ws = csp_workspace(
            &tmp,
            &[(
                "app.css",
                "/* x */ .y{content:'https://tailwindcss.com.evil.com/exfil'}",
            )],
        );
        let r = run_gate_csp_authoring(&ws).unwrap();
        assert!(
            !r.passed,
            "look-alike host sharing the allowlist prefix must be rejected"
        );

        // The genuine allowlisted identifier (exact) and a sub-path still pass.
        assert!(url_allowlisted("https://tailwindcss.com"));
        assert!(url_allowlisted("https://tailwindcss.com/license"));
        assert!(url_allowlisted("http://www.w3.org/2000/svg"));
        assert!(!url_allowlisted("https://tailwindcss.com.evil.com/x"));
        assert!(!url_allowlisted("https://tailwindcss.community/x"));
    }

    #[test]
    fn test_csp_gate_vendored_tier_keeps_namespace_and_license_strings() {
        // A vendored bundle legitimately carries namespace identifiers and a
        // license-banner URL that is NOT in CSS_URL_ALLOW. The vendored tier
        // only forbids live network primitives, so this must PASS (S3 F6:
        // never break vendored bundles).
        let tmp = TempDir::new().unwrap();
        let ws = csp_workspace(
            &tmp,
            &[
                // under a vendor/ directory (daisyui template convention)
                (
                    "vendor/htm.umd.js",
                    "/* htm | MIT | https://github.com/developit/htm */\nvar NS='http://www.w3.org/2000/svg';",
                ),
                // recognized bundle by filename at the project root (react template
                // ships react-dom.production.min.js / htm.umd.js there)
                (
                    "react-dom.production.min.js",
                    "/* react-dom */ var e='see https://reactjs.org/docs/error-decoder.html?invariant='+i;",
                ),
            ],
        );
        let r = run_gate_csp_authoring(&ws).unwrap();
        assert!(
            r.passed,
            "vendored bundles (vendor/ dir or *.umd.js/*.min.js) keep license + namespace strings: {:?}",
            r.issues
        );
    }

    #[test]
    fn test_csp_gate_covers_every_none_directive() {
        // Cross-crate anti-drift contract: every directive BLOB_SERVE_CSP sets
        // to `'none'` must have a detection rule. If the policy gains a new
        // `'none'` directive, this fails until a rule is added.
        use nexus_core_rs::csp::{BLOB_SERVE_CSP, none_directives};
        for d in none_directives(BLOB_SERVE_CSP) {
            assert!(
                CSP_RULES.iter().any(|r| r.directive == d),
                "BLOB_SERVE_CSP sets `{d} 'none'` but FG-CSP-authoring has no detection rule for it"
            );
        }
    }
}
