// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use serde::Serialize;

const PROMPT_KINDS: &[&str] = &[
    "base",
    "universal",
    "handoff",
    "preflight",
    "phase-review",
    "commit-body",
    "audit-gate",
    "phase-auditor",
];

const KIND_ALIASES: &[(&str, &str)] = &[
    ("review", "phase-review"),
    ("auditor", "phase-auditor"),
    ("audit", "audit-gate"),
];

const PROVIDERS: &[&str] = &["claude", "codex", "gpt", "local", "human"];

pub fn repo_root_pub() -> PathBuf {
    repo_root()
}

pub fn list_active_artifacts_pub(root: &Path) -> Vec<String> {
    list_active_artifacts(&root.join(".planning/active"))
}

pub fn prompt_filename_pub(kind: &str) -> String {
    let canonical = resolve_kind(kind).unwrap_or(kind);
    prompt_filename(canonical)
}

fn repo_root() -> PathBuf {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => {
            PathBuf::from(String::from_utf8_lossy(&o.stdout).trim().to_string())
        }
        _ => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

fn resolve_kind(kind: &str) -> Option<&'static str> {
    if let Some(&k) = PROMPT_KINDS.iter().find(|&&k| k == kind) {
        return Some(k);
    }
    KIND_ALIASES
        .iter()
        .find(|(alias, _)| *alias == kind)
        .map(|(_, canonical)| *canonical)
}

fn prompt_filename(kind: &str) -> String {
    match kind {
        "audit-gate" => "audit-gate-checks.md".to_string(),
        other => format!("{other}.md"),
    }
}

pub fn run_prompt(
    kind: &str,
    depth: &str,
    provider: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let output = prompt_data(kind, depth, provider)?;
    print!("{output}");
    Ok(())
}

fn strip_cloud_references(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            !lower.contains("websearch")
                && !lower.contains("context7")
                && !lower.contains("mcp__context7")
                && !lower.contains("mcp__claude")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn run_context() -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let ctx = context_data(&root);
    println!("{}", serde_json::to_string_pretty(&ctx)?);
    Ok(())
}

struct SprintInfo {
    number: u32,
    current_phase: String,
}

fn detect_sprint(active_dir: &Path) -> Option<SprintInfo> {
    let entries = std::fs::read_dir(active_dir).ok()?;
    let mut max_with_kickoff = 0u32;
    let mut max_any = 0u32;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(n) = name
            .strip_prefix("sprint")
            .and_then(|s| s.split('_').next())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if n > max_any {
                max_any = n;
            }
            if name.contains("_kickoff.md")
                || (name.contains("_plan.md") && !name.contains("_audit_plan.md"))
            {
                if n > max_with_kickoff {
                    max_with_kickoff = n;
                }
            }
        }
    }
    let best = if max_with_kickoff > 0 {
        max_with_kickoff
    } else {
        max_any
    };
    if best == 0 {
        return None;
    }

    let phase = detect_current_phase(active_dir, best);
    Some(SprintInfo {
        number: best,
        current_phase: phase,
    })
}

fn detect_current_phase(active_dir: &Path, sprint: u32) -> String {
    let phases = ['A', 'B', 'C', 'D', 'E', 'F', 'G'];
    let mut last_completed = None;
    for &p in &phases {
        let review = active_dir.join(format!("sprint{sprint}_phase_{p}_review.md"));
        if review.exists() {
            if let Ok(content) = std::fs::read_to_string(&review) {
                if has_final_pass_verdict(&content) {
                    last_completed = Some(p);
                }
            }
        }
    }
    match last_completed {
        Some('G') => "done".to_string(),
        Some(p) => {
            let next = (p as u8 + 1) as char;
            next.to_string()
        }
        None => "A".to_string(),
    }
}

fn list_active_artifacts(active_dir: &Path) -> Vec<String> {
    let mut artifacts = Vec::new();
    if let Ok(entries) = std::fs::read_dir(active_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".md") {
                    artifacts.push(name.to_string());
                }
            }
        }
    }
    artifacts.sort();
    artifacts
}

fn git_short_head() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn git_branch() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn git_dirty_files() -> Vec<String> {
    std::process::Command::new("git")
        .args(["status", "--short", "--porcelain"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.starts_with("?? "))
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn git_staged_files() -> Vec<String> {
    std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn git_recent_commits(count: usize) -> Vec<String> {
    std::process::Command::new("git")
        .args(["log", "--oneline", &format!("-{count}")])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

// --- status-sprint ---

#[derive(Serialize, Clone)]
pub struct StatusSprintResult {
    pub sprint: u32,
    pub branch: String,
    pub head: String,
    pub current_phase: String,
    pub has_kickoff: bool,
    pub has_plan: bool,
    pub has_design_review: bool,
    pub has_audit_plan: bool,
    pub phases: Vec<PhaseStatusEntry>,
}

#[derive(Serialize, Clone)]
pub struct PhaseStatusEntry {
    pub letter: String,
    pub has_preflight: bool,
    pub has_review: bool,
    pub review_verdict: Option<String>,
    pub has_codex: bool,
}

pub fn status_sprint_data(root: &Path) -> Option<StatusSprintResult> {
    let active_dir = root.join(".planning/active");
    if !active_dir.exists() {
        return None;
    }
    let info = detect_sprint(&active_dir)?;
    let s = info.number;

    let phases: Vec<PhaseStatusEntry> = ['A', 'B', 'C', 'D', 'E', 'F', 'G']
        .iter()
        .map(|&p| {
            let prefix = format!("sprint{s}_phase_{p}");
            let has_review = active_dir.join(format!("{prefix}_review.md")).exists();
            let verdict = if has_review {
                std::fs::read_to_string(active_dir.join(format!("{prefix}_review.md")))
                    .ok()
                    .and_then(|c| extract_verdict(&c))
            } else {
                None
            };
            PhaseStatusEntry {
                letter: p.to_string(),
                has_preflight: active_dir.join(format!("{prefix}_preflight.md")).exists(),
                has_review,
                review_verdict: verdict,
                has_codex: active_dir
                    .join(format!("{prefix}_codex_review.md"))
                    .exists(),
            }
        })
        .collect();

    Some(StatusSprintResult {
        sprint: s,
        branch: git_branch(),
        head: git_short_head(),
        current_phase: info.current_phase,
        has_kickoff: active_dir.join(format!("sprint{s}_kickoff.md")).exists(),
        has_plan: active_dir.join(format!("sprint{s}_plan.md")).exists(),
        has_design_review: active_dir
            .join(format!("sprint{s}_design_review.md"))
            .exists(),
        has_audit_plan: active_dir.join(format!("sprint{s}_audit_plan.md")).exists(),
        phases,
    })
}

fn has_final_pass_verdict(content: &str) -> bool {
    content.lines().any(|l| {
        let t = l.trim();
        t == "## Verdict: PASS"
    })
}

fn extract_verdict(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("## Verdict") {
            let rest = rest.trim().strip_prefix(':').unwrap_or(rest).trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

pub fn run_status_sprint(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    match status_sprint_data(&root) {
        Some(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!("Sprint {} ({})", result.sprint, result.branch);
                println!("HEAD: {}", result.head);
                println!("Phase: {} (next)", result.current_phase);
                println!();
                let flag = |b: bool| if b { "ok" } else { "--" };
                println!(
                    "  kickoff: {}  plan: {}  design_review: {}  audit_plan: {}",
                    flag(result.has_kickoff),
                    flag(result.has_plan),
                    flag(result.has_design_review),
                    flag(result.has_audit_plan),
                );
                for p in &result.phases {
                    let verdict_str = p.review_verdict.as_deref().unwrap_or("--");
                    println!(
                        "  Phase {}: preflight={} review={} ({}) codex={}",
                        p.letter,
                        flag(p.has_preflight),
                        flag(p.has_review),
                        verdict_str,
                        flag(p.has_codex),
                    );
                }
            }
            Ok(())
        }
        None => Err("no active sprint found in .planning/active/".into()),
    }
}

// --- lint-planning ---

#[derive(Serialize, Clone)]
pub struct LintResult {
    pub ok: bool,
    pub errors: Vec<LintDiagnostic>,
    pub warnings: Vec<LintDiagnostic>,
}

#[derive(Serialize, Clone)]
pub struct LintDiagnostic {
    pub code: String,
    pub message: String,
    pub file: Option<String>,
}

pub fn lint_planning_data(root: &Path) -> LintResult {
    let active_dir = root.join(".planning/active");
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if !active_dir.exists() {
        return LintResult {
            ok: true,
            errors,
            warnings,
        };
    }

    let sprint_info = detect_sprint(&active_dir);
    let current_sprint = sprint_info.as_ref().map(|i| i.number).unwrap_or(0);

    if let Ok(entries) = std::fs::read_dir(&active_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.starts_with("sprint") || !name.ends_with(".md") {
                continue;
            }
            if let Some(file_sprint) = name
                .strip_prefix("sprint")
                .and_then(|s| s.split('_').next())
                .and_then(|s| s.parse::<u32>().ok())
            {
                if current_sprint > 0 && file_sprint + 1 < current_sprint {
                    warnings.push(LintDiagnostic {
                        code: "ORPHAN_FILE".into(),
                        message: format!(
                            "file from sprint {file_sprint} in active/ (current: {current_sprint})"
                        ),
                        file: Some(name.clone()),
                    });
                }
            }

            if name.contains("_review.md") && !name.contains("codex") && !name.contains("design") {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let has_pass_pending = content.contains("PASS-PENDING");
                    let has_final_pass = has_final_pass_verdict(&content);
                    if has_pass_pending && !has_final_pass {
                        errors.push(LintDiagnostic {
                            code: "STALE_PASS_PENDING".into(),
                            message: "review still at PASS-PENDING (not promoted to PASS)".into(),
                            file: Some(name.clone()),
                        });
                    }
                    let has_verdict_pass_loose = content.lines().any(|l| {
                        let t = l.trim();
                        t.starts_with("## Verdict")
                            && t.contains("PASS")
                            && !t.contains("PASS-PENDING")
                    });
                    if has_verdict_pass_loose && !has_final_pass {
                        errors.push(LintDiagnostic {
                            code: "INVALID_VERDICT_FORMAT".into(),
                            message:
                                "review has a PASS-like verdict but not exact '## Verdict: PASS'"
                                    .into(),
                            file: Some(name.clone()),
                        });
                    }
                }
            }
        }
    }

    if current_sprint > 0 {
        let has_kickoff = active_dir
            .join(format!("sprint{current_sprint}_kickoff.md"))
            .exists();
        let has_plan = active_dir
            .join(format!("sprint{current_sprint}_plan.md"))
            .exists();
        if has_plan && !has_kickoff {
            errors.push(LintDiagnostic {
                code: "PLAN_WITHOUT_KICKOFF".into(),
                message: format!("sprint{current_sprint}_plan.md exists without kickoff"),
                file: None,
            });
        }
    }

    let ok = errors.is_empty();
    LintResult {
        ok,
        errors,
        warnings,
    }
}

pub fn run_lint_planning(json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let result = lint_planning_data(&root);
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        if result.ok && result.warnings.is_empty() {
            println!("lint-planning: CLEAN");
        } else {
            for e in &result.errors {
                let f = e.file.as_deref().unwrap_or("");
                println!("ERROR [{}] {} {}", e.code, e.message, f);
            }
            for w in &result.warnings {
                let f = w.file.as_deref().unwrap_or("");
                println!("WARN  [{}] {} {}", w.code, w.message, f);
            }
        }
    }
    if !result.ok {
        return Err("lint-planning found errors".into());
    }
    Ok(())
}

// --- audit-commit ---

const PHASE_TITLE_RE: &str =
    r"^(feat|fix|docs|chore|test|refactor)\([^)]+\):\s*Sprint\s+(\d+)\s+Phase\s+([A-Z][0-9]?)";

const REQUIRED_BODY_SECTIONS: &[(&str, &str)] = &[
    ("Contexte", r"(?mi)^## Contexte\s*$"),
    ("Fichiers", r"(?mi)^## Fichiers\s*$"),
    ("Delta tests", r"(?mi)^## Delta tests\s*$"),
    ("Verification", r"(?mi)^## V[eé]rification\b"),
    ("Scope cuts", r"(?mi)^## Scope cuts\s*$"),
    ("G8 traceability", r"(?mi)^## G8 traceability\s*$"),
    ("Pre-launch protocol", r"(?mi)^## Pre-launch protocol\s*$"),
    ("Codex verification", r"(?mi)^## Codex verification\s*$"),
    ("Carry closure", r"(?mi)^## Carry closure"),
];

#[derive(Serialize, Clone)]
pub struct AuditCommitResult {
    pub rev: String,
    pub title: String,
    pub is_phase_commit: bool,
    pub ok: bool,
    pub issues: Vec<String>,
}

pub fn audit_commit_data(
    root: &Path,
    rev: &str,
) -> Result<AuditCommitResult, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .args(["log", "--format=%s\n---BODY---\n%b", "-1", rev])
        .output()?;
    if !output.status.success() {
        return Err(format!("git log failed for rev '{rev}'").into());
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    let (title, body) = match raw.split_once("\n---BODY---\n") {
        Some((t, b)) => (t.trim().to_string(), b.to_string()),
        None => (raw.trim().to_string(), String::new()),
    };

    let phase_re = regex::Regex::new(PHASE_TITLE_RE)?;
    let caps = phase_re.captures(&title);
    let is_phase_commit = caps.is_some();

    let mut issues = Vec::new();

    if let Some(caps) = caps {
        let commit_type = &caps[1];
        let sprint_num: u32 = caps[2].parse().unwrap_or(0);
        let phase_letter = &caps[3];

        if matches!(
            commit_type,
            "feat" | "fix" | "docs" | "chore" | "test" | "refactor"
        ) {
            let active_dir = root.join(".planning/active");
            let review_path =
                active_dir.join(format!("sprint{sprint_num}_phase_{phase_letter}_review.md"));

            if review_path.exists() {
                let review_content = std::fs::read_to_string(&review_path)?;
                if !has_final_pass_verdict(&review_content) {
                    issues.push(
                        "review file exists but missing '## Verdict: PASS' (found PASS-PENDING?)"
                            .to_string(),
                    );
                }
            } else {
                let archive_exists = root.join(".planning/archive").exists()
                    && std::fs::read_dir(root.join(".planning/archive"))
                        .ok()
                        .map(|entries| {
                            entries.flatten().any(|e| {
                                e.path()
                                    .join(format!(
                                        "sprint{sprint_num}_phase_{phase_letter}_review.md"
                                    ))
                                    .exists()
                            })
                        })
                        .unwrap_or(false);
                if !archive_exists {
                    issues.push("missing review file".into());
                }
            }

            let codex_path = active_dir.join(format!(
                "sprint{sprint_num}_phase_{phase_letter}_codex_review.md"
            ));
            if !codex_path.exists() {
                let archive_codex = root.join(".planning/archive").exists()
                    && std::fs::read_dir(root.join(".planning/archive"))
                        .ok()
                        .map(|entries| {
                            entries.flatten().any(|e| {
                                e.path()
                                    .join(format!(
                                        "sprint{sprint_num}_phase_{phase_letter}_codex_review.md"
                                    ))
                                    .exists()
                            })
                        })
                        .unwrap_or(false);
                if !archive_codex {
                    issues.push("missing codex_review file".into());
                }
            }

            let missing_sections: Vec<&str> = REQUIRED_BODY_SECTIONS
                .iter()
                .filter(|(_, pattern)| {
                    regex::Regex::new(pattern)
                        .ok()
                        .map(|re| !re.is_match(&body))
                        .unwrap_or(true)
                })
                .map(|(name, _)| *name)
                .collect();

            if !missing_sections.is_empty() {
                issues.push(format!(
                    "missing body sections: {}",
                    missing_sections
                        .iter()
                        .map(|s| format!("## {s}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
    }

    let ok = issues.is_empty();
    Ok(AuditCommitResult {
        rev: rev.to_string(),
        title,
        is_phase_commit,
        ok,
        issues,
    })
}

pub fn run_audit_commit(rev: &str, json: bool) -> Result<(), Box<dyn std::error::Error>> {
    let root = repo_root();
    let result = audit_commit_data(&root, rev)?;
    if json {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("audit-commit: {} ({})", result.rev, result.title);
        if result.is_phase_commit {
            println!("  type: phase commit");
        } else {
            println!("  type: non-phase commit (no review required)");
        }
        if result.ok {
            println!("  result: PASS");
        } else {
            println!("  result: FAIL");
            for issue in &result.issues {
                println!("  - {issue}");
            }
        }
    }
    if !result.ok {
        return Err("audit-commit found issues".into());
    }
    Ok(())
}

// --- context data (refactored for operator reuse) ---

pub fn context_data(root: &Path) -> serde_json::Value {
    let head = git_short_head();
    let branch = git_branch();
    let active_dir = root.join(".planning/active");

    let mut context = serde_json::Map::new();
    context.insert(
        "repo".into(),
        serde_json::Value::String(root.display().to_string()),
    );
    context.insert("branch".into(), serde_json::Value::String(branch));
    context.insert("head".into(), serde_json::Value::String(head));

    let agent_system = root.join("docs/agent/AGENT_SYSTEM.md");
    context.insert(
        "agent_system".into(),
        serde_json::Value::Bool(agent_system.exists()),
    );

    let process_docs: Vec<serde_json::Value> = [
        "docs/agent/PROCESS.md",
        "docs/agent/TOOLING.md",
        "docs/agent/AGENT_SYSTEM.md",
        "AGENTS.md",
        "CLAUDE.md",
    ]
    .iter()
    .filter(|p| root.join(p).exists())
    .map(|p| serde_json::Value::String(p.to_string()))
    .collect();
    context.insert(
        "process_docs".into(),
        serde_json::Value::Array(process_docs),
    );

    let prompt_kinds: Vec<serde_json::Value> = PROMPT_KINDS
        .iter()
        .map(|k| {
            let exists = root.join("prompts/agent").join(prompt_filename(k)).exists();
            serde_json::json!({"kind": k, "exists": exists})
        })
        .collect();
    context.insert(
        "prompt_kinds".into(),
        serde_json::Value::Array(prompt_kinds),
    );

    if active_dir.exists() {
        let sprint_info = detect_sprint(&active_dir);
        if let Some(ref info) = sprint_info {
            context.insert(
                "sprint".into(),
                serde_json::Value::Number(info.number.into()),
            );
            context.insert(
                "phase".into(),
                serde_json::Value::String(info.current_phase.clone()),
            );
        }

        let artifacts: Vec<serde_json::Value> = list_active_artifacts(&active_dir)
            .into_iter()
            .map(serde_json::Value::String)
            .collect();
        context.insert(
            "active_artifacts".into(),
            serde_json::Value::Array(artifacts),
        );
    }

    let dirty = git_dirty_files();
    context.insert(
        "dirty_files".into(),
        serde_json::Value::Array(dirty.into_iter().map(serde_json::Value::String).collect()),
    );

    let staged = git_staged_files();
    context.insert(
        "staged_files".into(),
        serde_json::Value::Array(staged.into_iter().map(serde_json::Value::String).collect()),
    );

    let recent = git_recent_commits(5);
    context.insert(
        "recent_commits".into(),
        serde_json::Value::Array(recent.into_iter().map(serde_json::Value::String).collect()),
    );

    serde_json::Value::Object(context)
}

// --- prompt data (refactored for operator reuse) ---

pub fn prompt_data(
    kind: &str,
    depth: &str,
    provider: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let canonical = resolve_kind(kind)
        .ok_or_else(|| format!("unknown prompt kind '{kind}'. valid: {PROMPT_KINDS:?}"))?;

    if !PROVIDERS.contains(&provider) {
        return Err(format!("unknown provider '{provider}'. valid: {PROVIDERS:?}").into());
    }

    let root = repo_root();
    let prompt_path = root.join("prompts/agent").join(prompt_filename(canonical));

    if !prompt_path.exists() {
        return Err(format!("prompt file not found: {}", prompt_path.display()).into());
    }

    let content = std::fs::read_to_string(&prompt_path)?;
    let mut output = String::new();

    if depth == "deep" {
        output.push_str(&format!(
            "# Provider: {provider} | Kind: {canonical} | Depth: deep\n\n"
        ));
        output.push_str(&format!(
            "Prompt source: {}\n\n---\n\n",
            prompt_path.display()
        ));
    }

    output.push_str(&content);

    if provider == "local" {
        output = strip_cloud_references(&output);
    }

    Ok(output)
}

pub fn providers_list() -> Vec<&'static str> {
    PROVIDERS.to_vec()
}
