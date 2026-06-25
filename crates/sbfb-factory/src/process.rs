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
    // app-authoring (Sprint 79 Phase C, decision D2): surfaces the anime.js
    // CSP-safe authoring mastery to app-building agents. Resolves through the
    // generic `prompt_filename` arm to `prompts/agent/app-authoring.md` (no
    // alias, no special case). `prompt_kinds_resolve_to_existing_files` fails
    // the build if that file is absent — the drift-gated label for this
    // LLM-frontier primitive (docs/claude/README.md §6.12).
    "app-authoring",
];

const KIND_ALIASES: &[(&str, &str)] = &[
    ("review", "phase-review"),
    ("auditor", "phase-auditor"),
    ("audit", "audit-gate"),
];

/// Prompt-adaptation **providers** the Factory targets when it
/// generates a context pack — i.e. *which agent reads the prompt*
/// (`claude`, `codex`, `gpt`, `local`, `human`). This is a distinct
/// axis from the worker's runtime **execution backend** (the
/// `LlmBackend` in `nexus-worker-core`: Ollama / llama_cpp), which
/// decides *what engine runs an inference task*. The two are
/// orthogonal — a `claude`-targeted prompt and an Ollama-executed
/// compute task never meet on the same code path — so they are
/// intentionally NOT unified. (Sprint 71 Phase B / D8 ; rationale in
/// `docs/rust/PATTERNS.md`.)
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
            if (name.contains("_kickoff.md")
                || (name.contains("_plan.md") && !name.contains("_audit_plan.md")))
                && n > max_with_kickoff
            {
                max_with_kickoff = n;
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
    // A sprint is "done" once its verification.md exists — the same signal
    // `sprint_history::build_sprint_summary` already uses. Completion is NEVER
    // keyed on a literal phase letter: the old `Some('G') => "done"` meant a
    // >7-phase sprint (S77 reached phase N) could never report "done".
    if active_dir
        .join(format!("sprint{sprint}_verification.md"))
        .exists()
    {
        return "done".to_string();
    }
    // Otherwise the current phase is the successor of the furthest phase whose
    // review carries a final PASS verdict, discovered from disk (unbounded +
    // case-insensitive) rather than from a hardcoded ['A'..'G'] alphabet.
    let last_pass = crate::phase::discover_phase_artifacts(active_dir, sprint, "review")
        .into_iter()
        .rfind(|a| {
            std::fs::read_to_string(&a.path)
                .map(|c| has_final_pass_verdict(&c))
                .unwrap_or(false)
        })
        .map(|a| a.label);
    match last_pass {
        Some(label) => crate::phase::display_label(&crate::phase::next_phase_label(&label)),
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

    // One entry per phase actually present on disk (unbounded + case-insensitive
    // discovery), replacing the hardcoded ['A'..'G'] alphabet + uppercase path
    // construction that capped status at 7 phases and read nothing on Linux/CI.
    let phases: Vec<PhaseStatusEntry> = crate::phase::discover_phase_labels(&active_dir, s)
        .into_iter()
        .map(|label| {
            let review_path = crate::phase::find_phase_artifact(&active_dir, s, &label, "review");
            let verdict = review_path
                .as_ref()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|c| extract_verdict(&c));
            PhaseStatusEntry {
                letter: crate::phase::display_label(&label),
                has_preflight: crate::phase::find_phase_artifact(
                    &active_dir,
                    s,
                    &label,
                    "preflight",
                )
                .is_some(),
                has_review: review_path.is_some(),
                review_verdict: verdict,
                has_codex: crate::phase::find_phase_artifact(
                    &active_dir,
                    s,
                    &label,
                    "codex_review",
                )
                .is_some(),
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

// Phase token is `[A-Z]+[0-9]?` (README §4): A..Z, then AA.., with an optional
// sub-phase digit (F1/F2). A `[A-Z]` (single letter) would silently truncate a
// multi-letter phase and break this commit gate on it.
const PHASE_TITLE_RE: &str =
    r"^(feat|fix|docs|chore|test|refactor)\([^)]+\):\s*Sprint\s+(\d+)\s+Phase\s+([A-Z]+[0-9]?)";

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
    // `--end-of-options` forces git to treat `rev` as a revision, never an
    // option (defense in depth vs git option injection — S71 Phase D).
    let output = std::process::Command::new("git")
        .args([
            "log",
            "--format=%s\n---BODY---\n%b",
            "-1",
            "--end-of-options",
            rev,
        ])
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
        // The commit title carries the phase UPPERCASE ("Phase A"); on-disk
        // artifacts are lowercase for the active sprint and mixed-case in the
        // archive. Match case-insensitively against the real file name (never
        // rebuild the path from the title letter) so this gate is correct on
        // case-sensitive filesystems (Linux/CI/VPS), not only on Windows.
        let phase_label = caps[3].to_ascii_lowercase();

        if matches!(
            commit_type,
            "feat" | "fix" | "docs" | "chore" | "test" | "refactor"
        ) {
            let active_dir = root.join(".planning/active");
            let review_path =
                crate::phase::find_phase_artifact(&active_dir, sprint_num, &phase_label, "review");

            if let Some(review_path) = review_path {
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
                                crate::phase::find_phase_artifact(
                                    &e.path(),
                                    sprint_num,
                                    &phase_label,
                                    "review",
                                )
                                .is_some()
                            })
                        })
                        .unwrap_or(false);
                if !archive_exists {
                    issues.push("missing review file".into());
                }
            }

            let codex_path = crate::phase::find_phase_artifact(
                &active_dir,
                sprint_num,
                &phase_label,
                "codex_review",
            );
            if codex_path.is_none() {
                let archive_codex = root.join(".planning/archive").exists()
                    && std::fs::read_dir(root.join(".planning/archive"))
                        .ok()
                        .map(|entries| {
                            entries.flatten().any(|e| {
                                crate::phase::find_phase_artifact(
                                    &e.path(),
                                    sprint_num,
                                    &phase_label,
                                    "codex_review",
                                )
                                .is_some()
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

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 71 Phase D / G6: the off-sprint prompt-kind/provider plumbing
    // had no unit coverage. These pin the alias table, the provider set, and
    // the git-root resolution that the operator/context paths rely on.

    #[test]
    fn resolve_kind_aliases() {
        // Short aliases resolve to their canonical prompt kind.
        assert_eq!(resolve_kind("review"), Some("phase-review"));
        assert_eq!(resolve_kind("auditor"), Some("phase-auditor"));
        assert_eq!(resolve_kind("audit"), Some("audit-gate"));
        // A canonical kind passes through unchanged.
        assert_eq!(resolve_kind("preflight"), Some("preflight"));
        assert_eq!(resolve_kind("base"), Some("base"));
        // An unknown kind resolves to nothing (callers surface an error).
        assert_eq!(resolve_kind("definitely-not-a-kind"), None);
    }

    #[test]
    fn providers_list_is_canonical() {
        // D8 (resolved Phase B): the prompt-adaptation providers are a fixed
        // set, distinct from the runtime LlmBackend.
        assert_eq!(
            providers_list(),
            vec!["claude", "codex", "gpt", "local", "human"]
        );
    }

    #[test]
    fn repo_root_resolves() {
        let root = repo_root_pub();
        assert!(
            root.is_absolute(),
            "repo root must be absolute: {}",
            root.display()
        );
        assert!(
            root.join(".git").exists(),
            "repo root must contain a .git entry: {}",
            root.display()
        );
    }

    #[test]
    fn prompt_kinds_resolve_to_existing_files() {
        // P2-F-3 (closed 3/3, Sprint 72 Phase B): every canonical prompt kind
        // must resolve to a `prompts/agent/<file>.md` that exists on disk.
        // Guards against a kind being added to `PROMPT_KINDS` (or its filename
        // mapping in `prompt_filename` changed) without the backing prompt
        // file — a breakage `prompt_data` would otherwise only surface at
        // runtime as a "prompt file not found" error on the operator/context
        // path.
        let root = repo_root();
        for kind in PROMPT_KINDS {
            let path = root.join("prompts/agent").join(prompt_filename(kind));
            assert!(
                path.exists(),
                "PROMPT_KINDS entry '{kind}' has no prompt file at {}",
                path.display()
            );
        }
    }

    #[test]
    fn app_authoring_prompt_surfaces_csp_markers() {
        // Sprint 79 Phase C: the app-authoring fiche must surface the hard CSP
        // pitfalls verbatim so any authoring agent (claude / gpt / local)
        // inherits the sealed-iframe doctrine. These five markers are the T1a
        // assertion of the per-sprint testability gate (plan §4). Asserting both
        // providers proves *today* that all five markers survive the `local`
        // path's `strip_cloud_references` pass (the fiche carries no
        // websearch/context7/mcp token, so the strip is currently a no-op), and
        // stands as a forward guard: it bites the day a marker ever shares a line
        // with a stripped token. Both providers must print the identical doctrine.
        const MARKERS: &[&str] = &[
            "box-shadow STATIQUE",
            "motion-path cx=0",
            "morphTo mono-trace",
            "prefers-reduced-motion → état-final",
            "UMD classic-script jamais type=module",
        ];
        for provider in ["claude", "local"] {
            let out = prompt_data("app-authoring", "shallow", provider)
                .unwrap_or_else(|e| panic!("prompt_data app-authoring/{provider}: {e}"));
            for marker in MARKERS {
                assert!(
                    out.contains(marker),
                    "app-authoring prompt ({provider}) missing CSP marker {marker:?}"
                );
            }
        }
    }

    #[test]
    fn app_authoring_prompt_surfaces_daisyui_markers() {
        // Sprint 79 Phase F: the app-authoring fiche must also surface the
        // daisyUI build + per-class CSP doctrine so an authoring agent inherits
        // the corrected facts (no in-iframe Tailwind build / purge — not "fails
        // to compile"; lean template = NONE of the 35 built-in themes, never the
        // false "8"). These markers are the Phase F slice of the per-sprint
        // testability gate. Asserting both providers proves the markers survive
        // the `local` path's `strip_cloud_references` pass (the daisyUI section
        // carries no websearch/context7/mcp token, so the strip is a no-op) and
        // stands as a forward guard against a future marker sharing a line with a
        // stripped token.
        const MARKERS: &[&str] = &[
            "daisyUI 5.5.23",
            "source(none)",
            "sbfb-reflect",
            "aucun des 35 thèmes built-in",
            "app.css --minify",
            "classes-bank.json",
        ];
        for provider in ["claude", "local"] {
            let out = prompt_data("app-authoring", "shallow", provider)
                .unwrap_or_else(|e| panic!("prompt_data app-authoring/{provider}: {e}"));
            for marker in MARKERS {
                assert!(
                    out.contains(marker),
                    "app-authoring prompt ({provider}) missing daisyUI marker {marker:?}"
                );
            }
        }
    }

    #[test]
    fn detect_current_phase_is_unbounded_and_case_insensitive() {
        // Regression guard for the ['A'..'G'] cap + the uppercase-path bug.
        // Active-sprint artifacts are lowercase; phases run past G (S77 hit N).
        // On a case-sensitive filesystem the old code read nothing here and
        // froze at "A"; the cap also made >G phases invisible.
        let dir = tempfile::tempdir().expect("tempdir");
        let active = dir.path();
        for label in ["a", "b", "h", "i"] {
            std::fs::write(
                active.join(format!("sprint79_phase_{label}_review.md")),
                "## Verdict: PASS\n",
            )
            .unwrap();
        }
        // Furthest PASS is phase i -> current phase is j (uppercased display).
        assert_eq!(detect_current_phase(active, 79), "J");

        // verification.md present => "done", never keyed on a literal letter.
        std::fs::write(active.join("sprint79_verification.md"), "done").unwrap();
        assert_eq!(detect_current_phase(active, 79), "done");
    }

    #[test]
    fn phase_title_re_accepts_unbounded_multi_letter() {
        // Guard the most visible line of the unbounded-phase fix: the commit
        // title token is [A-Z]+[0-9]?, NOT the old capped [A-G][0-9]?. A typo
        // reverting the class would otherwise pass every other test (they all
        // use a mono-letter "Phase A").
        let re = regex::Regex::new(PHASE_TITLE_RE).unwrap();
        for (title, want) in [
            ("feat(x): Sprint 77 Phase N — t", "N"),   // letter beyond G
            ("fix(x): Sprint 79 Phase AA — t", "AA"),  // multi-letter
            ("docs(x): Sprint 80 Phase F1 — t", "F1"), // sub-phase digit
        ] {
            let caps = re
                .captures(title)
                .unwrap_or_else(|| panic!("PHASE_TITLE_RE no match: {title}"));
            assert_eq!(&caps[3], want, "phase token for {title}");
        }
    }

    #[test]
    fn agent_wrappers_reference_existing_prompts() {
        // P2-F-3 (closed 3/3, Sprint 72 Phase B): every `prompts/agent/<file>.md`
        // path named by a `.claude/agents/*.md` wrapper must exist on disk.
        // `prompt_kinds_resolve_to_existing_files` covers the canonical kind
        // set; this covers the *wrapper -> prompt* coupling directly, catching
        // a wrapper that points at a renamed/typo'd file outside the kind set
        // — the exact P2-F-3 breakage (1/3 S70 -> 2/3 S71 -> 3/3 S72, closed
        // here, never carried again). The stability contract is documented in
        // `docs/agent/AGENT_SYSTEM.md`.
        let root = repo_root();
        let agents_dir = root.join(".claude/agents");
        assert!(
            agents_dir.is_dir(),
            "agent wrappers directory missing: {}",
            agents_dir.display()
        );

        let mut checked = 0usize;
        for dir_entry in std::fs::read_dir(&agents_dir).expect("read .claude/agents") {
            let wrapper_path = dir_entry.expect("dir entry").path();
            if wrapper_path.extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            let content = std::fs::read_to_string(&wrapper_path).expect("read wrapper");
            for prompt_ref in prompt_refs_in(&content) {
                let target = root.join(&prompt_ref);
                assert!(
                    target.exists(),
                    "wrapper {} references missing prompt {}",
                    wrapper_path.display(),
                    target.display()
                );
                checked += 1;
            }
        }
        assert!(
            checked > 0,
            "expected at least one `prompts/agent/*.md` reference across the \
             agent wrappers (the coupling guard would be vacuous otherwise)"
        );
    }

    /// Extract every `prompts/agent/<name>.md` path mentioned in a wrapper's
    /// markdown body. References appear inside backticks (e.g.
    /// ``Lis `prompts/agent/preflight.md` en entier``), so we scan for the
    /// literal `prompts/agent/` marker and read up to the first terminator
    /// (backtick, whitespace, `)` or `,`).
    fn prompt_refs_in(content: &str) -> Vec<String> {
        const MARKER: &str = "prompts/agent/";
        let mut refs = Vec::new();
        let mut rest = content;
        while let Some(pos) = rest.find(MARKER) {
            let after = &rest[pos..];
            let end = after
                .find(|c: char| c == '`' || c.is_whitespace() || c == ')' || c == ',')
                .unwrap_or(after.len());
            let candidate = &after[..end];
            if candidate.ends_with(".md") {
                refs.push(candidate.to_string());
            }
            // The marker contains no terminator chars, so `end` is always past
            // it; `max(1)` is a belt-and-braces guard against a zero-width step.
            rest = &after[end.max(1)..];
        }
        refs
    }
}
