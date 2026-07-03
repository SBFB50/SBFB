// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::Serialize;

#[derive(Serialize)]
pub struct SprintHistoryResult {
    pub sprint: u32,
    pub status: String,
    pub branch: String,
    pub head: String,
    pub entry_tip: Option<String>,
    pub exit_tip: Option<String>,
    pub roadmap: Option<String>,
    pub total_commits: usize,
    pub phase_commits: usize,
    pub chore_commits: usize,
    pub phases: Vec<PhaseHistory>,
    pub commits: Vec<CommitInfo>,
    pub tests: TestSummary,
    pub scope_cuts: Vec<ScopeCutItem>,
    pub carries_closed: Vec<CarryItem>,
    pub carries_open: Vec<CarryItem>,
    pub verification: Option<VerificationSummary>,
    pub preflight_bilan: PreflightBilan,
}

#[derive(Serialize)]
pub struct PhaseHistory {
    pub letter: String,
    pub title: String,
    pub commit_sha: Option<String>,
    pub commit_date: Option<String>,
    pub commit_type: Option<String>,
    pub preflight_verdict: Option<String>,
    pub review_verdict: Option<String>,
    pub codex_confirmed: Option<u32>,
    pub codex_partial: Option<u32>,
    pub codex_gap: Option<u32>,
    pub rust_delta: i32,
    pub vitest_delta: i32,
    pub files_changed: Vec<FileChange>,
    pub deliverables: Vec<String>,
    pub findings: Vec<Finding>,
}

#[derive(Serialize)]
pub struct CommitInfo {
    pub sha: String,
    pub short: String,
    pub title: String,
    pub author: String,
    pub date: String,
    pub commit_type: String,
    pub scope: String,
    pub is_phase: bool,
    pub phase: Option<String>,
    pub insertions: u32,
    pub deletions: u32,
    pub files: Vec<String>,
    pub body_sections: Vec<String>,
}

#[derive(Serialize)]
pub struct FileChange {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
    pub status: String,
}

#[derive(Serialize)]
pub struct TestSummary {
    pub rust_entry: u32,
    pub rust_exit: u32,
    pub rust_delta: i32,
    pub vitest_entry: u32,
    pub vitest_exit: u32,
    pub vitest_delta: i32,
    pub size_limit: String,
    pub per_phase: Vec<PhaseTestDelta>,
}

#[derive(Serialize)]
pub struct PhaseTestDelta {
    pub phase: String,
    pub rust_delta: i32,
    pub vitest_delta: i32,
    pub detail: String,
}

#[derive(Serialize)]
pub struct ScopeCutItem {
    pub number: u32,
    pub item: String,
    pub target: String,
    pub respected: bool,
}

#[derive(Serialize)]
pub struct CarryItem {
    pub code: String,
    pub description: String,
    pub disposition: String,
    pub phase_closed: Option<String>,
}

#[derive(Serialize)]
pub struct Finding {
    pub severity: String,
    pub code: String,
    pub description: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct VerificationSummary {
    pub total_checks: u32,
    pub passed: u32,
    pub failed: u32,
    pub checks: Vec<VerificationCheck>,
}

#[derive(Serialize)]
pub struct VerificationCheck {
    pub number: u32,
    pub name: String,
    pub command: String,
    pub result: String,
}

#[derive(Serialize)]
pub struct PreflightBilan {
    pub total: u32,
    pub execute: u32,
    pub plan_adapt: u32,
    pub design_conflict: u32,
    pub phases: Vec<PreflightPhase>,
}

#[derive(Serialize)]
pub struct PreflightPhase {
    pub phase: String,
    pub verdict: String,
    pub file: String,
}

static PHASE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    // Phase token is `[A-Z]+[0-9]?` (README §4 — unbounded): A..Z, then AA..,
    // with an optional sub-phase digit (F1/F2). The old `[A-G][0-9]?` was a
    // DOUBLE cap (letter <= G AND single letter): a `Sprint 77 Phase N` or
    // `Phase AA` commit parsed as non-phase and dropped from the history.
    Regex::new(
        r"^(feat|fix|docs|chore|test|refactor)\(([^)]+)\):\s*Sprint\s+(\d+)\s+Phase\s+([A-Z]+[0-9]?)\s*[—–-]\s*(.+)"
    ).unwrap()
});

static COMMIT_TYPE_RE: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
    Regex::new(r"^(feat|fix|docs|chore|test|refactor)\(([^)]+)\)").unwrap()
});

#[derive(Serialize)]
pub struct AllSprintsResult {
    pub sprints: Vec<SprintSummary>,
    pub total: usize,
}

#[derive(Serialize)]
pub struct SprintSummary {
    pub sprint: u32,
    pub version: String,
    pub status: String,
    pub phase_count: usize,
    pub phases_pass: usize,
    pub has_verification: bool,
    pub dir: String,
}

pub fn all_sprints_data(root: &Path) -> AllSprintsResult {
    let mut sprints = Vec::new();
    let mut seen = BTreeSet::new();

    let dirs = discover_sprint_dirs(root);
    for (dir, version) in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(n) = name
                    .strip_prefix("sprint")
                    .and_then(|s| s.split('_').next())
                    .and_then(|s| s.parse::<u32>().ok())
                    && name.contains("_kickoff.md")
                    && seen.insert(n)
                {
                    let summary = build_sprint_summary(dir, n, version);
                    sprints.push(summary);
                }
            }
        }
    }

    sprints.sort_by_key(|s| s.sprint);
    let total = sprints.len();
    AllSprintsResult { sprints, total }
}

fn discover_sprint_dirs(root: &Path) -> Vec<(PathBuf, String)> {
    let mut dirs = Vec::new();

    let active = root.join(".planning/active");
    if active.is_dir() {
        dirs.push((active, "active".to_string()));
    }

    let archive = root.join(".planning/archive");
    if let Ok(entries) = std::fs::read_dir(&archive) {
        let mut versions: Vec<_> = entries
            .flatten()
            .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
            .collect();
        versions.sort_by_key(|e| e.file_name());
        for entry in versions {
            let name = entry.file_name().to_string_lossy().to_string();
            dirs.push((entry.path(), name));
        }
    }

    dirs
}

fn find_sprint_dir(root: &Path, sprint: u32) -> Option<PathBuf> {
    let active = root.join(".planning/active");
    let kickoff = format!("sprint{sprint}_kickoff.md");
    if active.join(&kickoff).exists() {
        return Some(active);
    }
    let archive = root.join(".planning/archive");
    if let Ok(entries) = std::fs::read_dir(&archive) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(&kickoff).exists() {
                return Some(path);
            }
        }
    }
    None
}

fn build_sprint_summary(dir: &Path, sprint: u32, version: &str) -> SprintSummary {
    // Phases discovered from disk (unbounded + case-insensitive) rather than a
    // hardcoded ['A'..'G'] alphabet read via uppercase paths.
    let mut phase_count = 0usize;
    let mut phases_pass = 0usize;

    for label in crate::phase::discover_phase_labels(dir, sprint) {
        if let Some(review) = crate::phase::find_phase_artifact(dir, sprint, &label, "review") {
            if let Ok(content) = std::fs::read_to_string(&review) {
                phase_count += 1;
                if content.lines().any(|l| l.trim() == "## Verdict: PASS") {
                    phases_pass += 1;
                }
            }
        } else if crate::phase::find_phase_artifact(dir, sprint, &label, "preflight").is_some() {
            phase_count += 1;
        }
    }

    let has_verification = dir.join(format!("sprint{sprint}_verification.md")).exists();

    let status = if has_verification && phases_pass > 0 {
        "completed".to_string()
    } else if phase_count > 0 {
        "in_progress".to_string()
    } else {
        "kickoff_only".to_string()
    };

    SprintSummary {
        sprint,
        version: version.to_string(),
        status,
        phase_count,
        phases_pass,
        has_verification,
        dir: dir.to_string_lossy().to_string(),
    }
}

pub fn sprint_history_data(root: &Path) -> Option<SprintHistoryResult> {
    let active_dir = root.join(".planning/active");
    if !active_dir.is_dir() {
        return None;
    }
    let sprint = detect_history_sprint(&active_dir)?;
    sprint_history_for(root, sprint)
}

pub fn sprint_history_for(root: &Path, sprint: u32) -> Option<SprintHistoryResult> {
    let sprint_dir = find_sprint_dir(root, sprint)?;
    let entry_tip = find_entry_tip(sprint);
    let commits = collect_sprint_commits(sprint, entry_tip.as_deref());
    let phases = build_phase_histories(&sprint_dir, sprint, &commits);
    let tests = build_test_summary(&sprint_dir, sprint, &phases);
    let scope_cuts = parse_scope_cuts(&sprint_dir, sprint);
    let (carries_closed, carries_open) = parse_carries(&sprint_dir, sprint);
    let verification = parse_verification(&sprint_dir, sprint);
    let preflight_bilan = build_preflight_bilan(&sprint_dir, sprint);

    let phase_commits = commits.iter().filter(|c| c.is_phase).count();
    let chore_commits = commits.len() - phase_commits;
    let status = if phases
        .iter()
        .all(|p| p.review_verdict.as_deref() == Some("PASS"))
        && !phases.is_empty()
    {
        "completed".to_string()
    } else {
        "in_progress".to_string()
    };

    Some(SprintHistoryResult {
        sprint,
        status,
        branch: git_cmd(&["rev-parse", "--abbrev-ref", "HEAD"]),
        head: git_cmd(&["rev-parse", "--short", "HEAD"]),
        entry_tip: entry_tip.clone(),
        exit_tip: commits.first().map(|c| c.short.clone()),
        roadmap: extract_roadmap(&sprint_dir, sprint),
        total_commits: commits.len(),
        phase_commits,
        chore_commits,
        phases,
        commits,
        tests,
        scope_cuts,
        carries_closed,
        carries_open,
        verification,
        preflight_bilan,
    })
}

fn detect_history_sprint(active_dir: &Path) -> Option<u32> {
    let entries = std::fs::read_dir(active_dir).ok()?;
    let mut best = 0u32;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(n) = name
            .strip_prefix("sprint")
            .and_then(|s| s.split('_').next())
            .and_then(|s| s.parse::<u32>().ok())
            && (name.contains("_kickoff.md")
                || (name.contains("_plan.md") && !name.contains("_audit_plan.md")))
            && n > best
        {
            best = n;
        }
    }
    if best == 0 { None } else { Some(best) }
}

fn find_entry_tip(sprint: u32) -> Option<String> {
    let prev = sprint.checked_sub(1)?;
    let output = git_cmd(&[
        "log",
        "--all",
        "--oneline",
        "--grep",
        &format!("Sprint {} Phase", prev),
        "--format=%h",
    ]);
    output.lines().next().map(|s| s.trim().to_string())
}

fn collect_sprint_commits(sprint: u32, entry_tip: Option<&str>) -> Vec<CommitInfo> {
    let range = match entry_tip {
        Some(tip) => format!("{tip}..HEAD"),
        None => "HEAD~50..HEAD".to_string(),
    };
    let raw = git_cmd(&["log", "--reverse", "--format=%H|%h|%aI|%an|%s", &range]);
    let sprint_str = format!("Sprint {sprint}");
    let sprint_str_lower = format!("sprint{sprint}");

    raw.lines()
        .filter(|line| !line.is_empty())
        .filter(|line| {
            line.contains(&sprint_str)
                || line.contains(&sprint_str_lower)
                || line.contains("chore(planning)")
                || line.contains("chore(factory)")
                || line.contains("chore(skill)")
        })
        .map(|line| {
            let parts: Vec<&str> = line.splitn(5, '|').collect();
            if parts.len() < 5 {
                return empty_commit(line);
            }
            let sha = parts[0].to_string();
            let short = parts[1].to_string();
            let date = parts[2].to_string();
            let author = parts[3].to_string();
            let title = parts[4].to_string();

            let (commit_type, scope) = COMMIT_TYPE_RE
                .captures(&title)
                .map(|c| (c[1].to_string(), c[2].to_string()))
                .unwrap_or(("unknown".into(), "unknown".into()));

            let (is_phase, phase) = PHASE_RE
                .captures(&title)
                .map(|c| (true, Some(c[4].to_string())))
                .unwrap_or((false, None));

            let (insertions, deletions, files) = git_diff_stats(&sha);
            let body_sections = extract_body_sections(&sha);

            CommitInfo {
                sha,
                short,
                title,
                author,
                date,
                commit_type,
                scope,
                is_phase,
                phase,
                insertions,
                deletions,
                files,
                body_sections,
            }
        })
        .collect()
}

fn build_phase_histories(
    active_dir: &Path,
    sprint: u32,
    commits: &[CommitInfo],
) -> Vec<PhaseHistory> {
    // Phases discovered from disk (preflight/review/codex) UNIONed with the
    // phases present in commit titles, in canonical order — unbounded +
    // case-insensitive, replacing the hardcoded ['A'..'G'] alphabet (which
    // dropped phases H+ and read nothing on case-sensitive filesystems).
    let mut labels = crate::phase::discover_phase_labels(active_dir, sprint);
    for c in commits {
        if let Some(p) = &c.phase {
            let lower = p.to_ascii_lowercase();
            if !labels.contains(&lower) {
                labels.push(lower);
            }
        }
    }
    labels.sort_by_key(|a| crate::phase::phase_order_key(a));

    labels
        .into_iter()
        .filter_map(|label| {
            let phase_commit = commits.iter().find(|c| {
                c.phase
                    .as_deref()
                    .map(|p| p.eq_ignore_ascii_case(&label))
                    .unwrap_or(false)
            });

            // Real on-disk paths (case-insensitive); fall back to the canonical
            // lowercase name (which simply won't exist) when an artifact is
            // absent, so the verdict/finding parsers return their empty default.
            let preflight_path =
                crate::phase::find_phase_artifact(active_dir, sprint, &label, "preflight")
                    .unwrap_or_else(|| {
                        active_dir.join(format!("sprint{sprint}_phase_{label}_preflight.md"))
                    });
            let review_path =
                crate::phase::find_phase_artifact(active_dir, sprint, &label, "review")
                    .unwrap_or_else(|| {
                        active_dir.join(format!("sprint{sprint}_phase_{label}_review.md"))
                    });
            let codex_path =
                crate::phase::find_phase_artifact(active_dir, sprint, &label, "codex_review")
                    .unwrap_or_else(|| {
                        active_dir.join(format!("sprint{sprint}_phase_{label}_codex_review.md"))
                    });

            // A phase needs either a commit or at least a preflight artifact.
            if phase_commit.is_none() && !preflight_path.exists() {
                return None;
            }

            let title = phase_commit
                .and_then(|c| PHASE_RE.captures(&c.title))
                .map(|cap| cap[5].to_string())
                .unwrap_or_default();

            let preflight_verdict = read_verdict(&preflight_path, "EXECUTE");
            let review_verdict = read_verdict(&review_path, "PASS");
            let (codex_confirmed, codex_partial, codex_gap) = parse_codex_counts(&codex_path);

            let (rust_delta, vitest_delta) = phase_commit
                .map(|c| extract_test_deltas_from_body(&c.sha))
                .unwrap_or((0, 0));

            let files_changed = phase_commit
                .map(|c| build_file_changes(&c.sha))
                .unwrap_or_default();

            let deliverables = phase_commit
                .map(|c| extract_deliverables(&c.sha))
                .unwrap_or_default();

            let findings = parse_review_findings(&review_path);

            Some(PhaseHistory {
                letter: crate::phase::display_label(&label),
                title,
                commit_sha: phase_commit.map(|c| c.short.clone()),
                commit_date: phase_commit.map(|c| c.date.clone()),
                commit_type: phase_commit.map(|c| c.commit_type.clone()),
                preflight_verdict,
                review_verdict,
                codex_confirmed,
                codex_partial,
                codex_gap,
                rust_delta,
                vitest_delta,
                files_changed,
                deliverables,
                findings,
            })
        })
        .collect()
}

fn build_test_summary(active_dir: &Path, sprint: u32, phases: &[PhaseHistory]) -> TestSummary {
    let verification = active_dir.join(format!("sprint{sprint}_verification.md"));
    let content = std::fs::read_to_string(&verification).unwrap_or_default();

    let (rust_entry, rust_exit) = extract_test_counts(&content, "Rust nextest");
    let (vitest_entry, vitest_exit) = extract_test_counts(&content, "Vitest");

    let per_phase: Vec<PhaseTestDelta> = phases
        .iter()
        .map(|p| PhaseTestDelta {
            phase: p.letter.clone(),
            rust_delta: p.rust_delta,
            vitest_delta: p.vitest_delta,
            detail: if p.rust_delta == 0 && p.vitest_delta == 0 {
                "docs-only".to_string()
            } else {
                format!("+{} Rust, +{} Vitest", p.rust_delta, p.vitest_delta)
            },
        })
        .collect();

    TestSummary {
        rust_entry,
        rust_exit,
        rust_delta: rust_exit as i32 - rust_entry as i32,
        vitest_entry,
        vitest_exit,
        vitest_delta: vitest_exit as i32 - vitest_entry as i32,
        size_limit: "6/6".to_string(),
        per_phase,
    }
}

fn parse_scope_cuts(active_dir: &Path, sprint: u32) -> Vec<ScopeCutItem> {
    let verification = active_dir.join(format!("sprint{sprint}_verification.md"));
    let content = std::fs::read_to_string(&verification).unwrap_or_default();
    let re = Regex::new(r"^\|\s*(\d+)\s*\|\s*(.+?)\s*\|\s*(.+?)\s*\|\s*(OUI|NON)\b").unwrap();

    let in_section = content.contains("## §3 Scope cuts");
    if !in_section {
        return Vec::new();
    }

    let section = extract_section(&content, "§3 Scope cuts");
    section
        .lines()
        .filter_map(|line| {
            re.captures(line).map(|cap| ScopeCutItem {
                number: cap[1].parse().unwrap_or(0),
                item: cap[2].trim().to_string(),
                target: cap[3].trim().to_string(),
                respected: &cap[4] == "OUI",
            })
        })
        .collect()
}

fn parse_carries(active_dir: &Path, sprint: u32) -> (Vec<CarryItem>, Vec<CarryItem>) {
    let verification = active_dir.join(format!("sprint{sprint}_verification.md"));
    let content = std::fs::read_to_string(&verification).unwrap_or_default();

    let closed_section = extract_section(&content, "Carries CLOSED");
    let open_section = extract_section(&content, "Carries S");

    let carry_re = Regex::new(r"^\|\s*(.+?)\s*\|\s*(.+?)\s*\|\s*(.+?)\s*\|").unwrap();

    let closed: Vec<CarryItem> = closed_section
        .lines()
        .filter_map(|line| {
            if line.contains("---") || line.contains("Carry") && line.contains("Phase") {
                return None;
            }
            carry_re.captures(line).map(|cap| CarryItem {
                code: cap[1].trim().to_string(),
                description: cap[3].trim().to_string(),
                disposition: "CLOSED".to_string(),
                phase_closed: Some(cap[2].trim().to_string()),
            })
        })
        .collect();

    let open: Vec<CarryItem> = open_section
        .lines()
        .filter_map(|line| {
            if line.contains("---") || line.contains("Carry") && line.contains("Compteur") {
                return None;
            }
            carry_re.captures(line).map(|cap| CarryItem {
                code: cap[1].trim().to_string(),
                description: cap[2].trim().to_string(),
                disposition: cap[3].trim().to_string(),
                phase_closed: None,
            })
        })
        .collect();

    (closed, open)
}

fn parse_verification(active_dir: &Path, sprint: u32) -> Option<VerificationSummary> {
    let path = active_dir.join(format!("sprint{sprint}_verification.md"));
    let content = std::fs::read_to_string(&path).ok()?;
    let section = extract_section(&content, "§1 Fail-fast");

    let check_re =
        Regex::new(r"^\|\s*(\d+)\s*\|\s*(.+?)\s*\|\s*`(.+?)`\s*\|\s*(.+?)\s*\|\s*(.+?)\s*\|")
            .unwrap();

    let checks: Vec<VerificationCheck> = section
        .lines()
        .filter_map(|line| {
            check_re.captures(line).map(|cap| VerificationCheck {
                number: cap[1].parse().unwrap_or(0),
                name: cap[2].trim().to_string(),
                command: cap[3].trim().to_string(),
                result: cap[5].trim().to_string(),
            })
        })
        .collect();

    let passed = checks.iter().filter(|c| c.result.contains("PASS")).count() as u32;
    let total = checks.len() as u32;

    Some(VerificationSummary {
        total_checks: total,
        passed,
        failed: total - passed,
        checks,
    })
}

fn build_preflight_bilan(active_dir: &Path, sprint: u32) -> PreflightBilan {
    let mut phases_out = Vec::new();
    let mut execute = 0u32;
    let mut plan_adapt = 0u32;
    let mut design_conflict = 0u32;

    // Preflight artifacts discovered from disk (unbounded + case-insensitive);
    // the emitted `file` field is the REAL on-disk name, not a rebuilt
    // uppercase one. Replaces the hardcoded ['A'..'G'] alphabet.
    for art in crate::phase::discover_phase_artifacts(active_dir, sprint, "preflight") {
        if let Ok(content) = std::fs::read_to_string(&art.path) {
            let verdict = extract_preflight_verdict(&content);
            match verdict.as_str() {
                "EXECUTE" => execute += 1,
                "PLAN-ADAPT" => plan_adapt += 1,
                "DESIGN-CONFLICT" => design_conflict += 1,
                _ => {}
            }
            phases_out.push(PreflightPhase {
                phase: crate::phase::display_label(&art.label),
                verdict,
                file: art
                    .path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
            });
        }
    }

    PreflightBilan {
        total: phases_out.len() as u32,
        execute,
        plan_adapt,
        design_conflict,
        phases: phases_out,
    }
}

// --- Helpers ---

fn git_cmd(args: &[&str]) -> String {
    std::process::Command::new("git")
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Like [`git_cmd`] but pinned to a working directory. Sprint 80 Phase F:
/// the working-tree diff must target `OperatorState.root` (and a temp
/// fixture in tests) rather than the process cwd, so it takes the root
/// explicitly via `current_dir` (no `-C <path>` arg — nothing user-supplied
/// reaches the command line).
fn git_cmd_in(root: &Path, args: &[&str]) -> String {
    std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn git_diff_stats(sha: &str) -> (u32, u32, Vec<String>) {
    let numstat = git_cmd(&["diff", "--numstat", &format!("{sha}^..{sha}")]);
    let mut ins = 0u32;
    let mut del = 0u32;
    let mut files = Vec::new();
    for line in numstat.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            ins += parts[0].parse::<u32>().unwrap_or(0);
            del += parts[1].parse::<u32>().unwrap_or(0);
            files.push(parts[2].to_string());
        }
    }
    (ins, del, files)
}

fn extract_body_sections(sha: &str) -> Vec<String> {
    let body = git_cmd(&["log", "-1", "--format=%b", sha]);
    let re = Regex::new(r"(?m)^## (.+)$").unwrap();
    re.captures_iter(&body).map(|c| c[1].to_string()).collect()
}

fn extract_deliverables(sha: &str) -> Vec<String> {
    let body = git_cmd(&["log", "-1", "--format=%b", sha]);
    let section = extract_section(&body, "## Fichiers");
    let file_re = Regex::new(r"`([^`]+\.\w+)`").unwrap();
    file_re
        .captures_iter(&section)
        .map(|c| c[1].to_string())
        .filter(|f| !f.starts_with("cargo") && !f.starts_with("cd "))
        .collect()
}

fn extract_test_deltas_from_body(sha: &str) -> (i32, i32) {
    let body = git_cmd(&["log", "-1", "--format=%b", sha]);
    let section = extract_section(&body, "## Delta tests");

    let plus_re = Regex::new(r"\+(\d+)").unwrap();
    let rust_delta: i32 = section
        .lines()
        .filter(|l| l.contains("Rust") || l.contains("workspace"))
        .filter_map(|l| plus_re.captures(l).and_then(|c| c[1].parse::<i32>().ok()))
        .next_back()
        .unwrap_or(0);

    let vitest_delta: i32 = section
        .lines()
        .filter(|l| l.contains("Vitest"))
        .filter_map(|l| plus_re.captures(l).and_then(|c| c[1].parse::<i32>().ok()))
        .next_back()
        .unwrap_or(0);

    (rust_delta, vitest_delta)
}

fn read_verdict(path: &Path, _default_keyword: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    extract_preflight_verdict_generic(&content)
}

fn extract_preflight_verdict(content: &str) -> String {
    extract_preflight_verdict_generic(content).unwrap_or_else(|| "UNKNOWN".to_string())
}

fn extract_preflight_verdict_generic(content: &str) -> Option<String> {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("## Verdict") {
            let rest = rest.trim().strip_prefix(':').unwrap_or(rest).trim();
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
        if t.contains("**EXECUTE") {
            return Some("EXECUTE".to_string());
        }
        if t.contains("**PLAN-ADAPT") {
            return Some("PLAN-ADAPT".to_string());
        }
        if t.contains("**DESIGN-CONFLICT") {
            return Some("DESIGN-CONFLICT".to_string());
        }
        if t.contains("**SCOPE-CUT-CONSISTENT") {
            return Some("SCOPE-CUT-CONSISTENT".to_string());
        }
    }
    None
}

fn parse_codex_counts(path: &Path) -> (Option<u32>, Option<u32>, Option<u32>) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (None, None, None),
    };

    let confirmed = Regex::new(r"(\d+)\s+CONFIRME")
        .ok()
        .and_then(|re| re.captures(&content))
        .and_then(|c| c[1].parse().ok());
    let partial = Regex::new(r"(\d+)\s+PARTIEL")
        .ok()
        .and_then(|re| re.captures(&content))
        .and_then(|c| c[1].parse().ok());
    let gap = Regex::new(r"(\d+)\s+GAP")
        .ok()
        .and_then(|re| re.captures(&content))
        .and_then(|c| c[1].parse().ok());

    (confirmed, partial, gap)
}

fn parse_review_findings(path: &Path) -> Vec<Finding> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let finding_re =
        Regex::new(r"(?m)^-\s+\*\*(P[0-3])[-_]?([A-Z0-9-]*)\*\*\s*:?\s*(.+)$").unwrap();

    finding_re
        .captures_iter(&content)
        .map(|cap| Finding {
            severity: cap[1].to_string(),
            code: format!("{}-{}", &cap[1], &cap[2]),
            description: cap[3].trim().to_string(),
            status: if cap[3].contains("CORRIGE") || cap[3].contains("CLOSED") {
                "resolved".to_string()
            } else {
                "open".to_string()
            },
        })
        .collect()
}

fn extract_test_counts(content: &str, suite: &str) -> (u32, u32) {
    let re = Regex::new(&format!(
        r"(?m)^\|\s*{}\s*\|\s*(\d+)\s*\|\s*(\d+)\s*\|",
        regex::escape(suite)
    ))
    .unwrap();

    re.captures(content)
        .map(|c| (c[1].parse().unwrap_or(0), c[2].parse().unwrap_or(0)))
        .unwrap_or((0, 0))
}

fn extract_section(content: &str, header: &str) -> String {
    let mut in_section = false;
    let mut result = String::new();
    for line in content.lines() {
        if line.contains(header) {
            in_section = true;
            continue;
        }
        if in_section {
            if line.starts_with("## ") || line.starts_with("---") {
                break;
            }
            result.push_str(line);
            result.push('\n');
        }
    }
    result
}

fn extract_roadmap(active_dir: &Path, sprint: u32) -> Option<String> {
    let kickoff = active_dir.join(format!("sprint{sprint}_kickoff.md"));
    let content = std::fs::read_to_string(&kickoff).ok()?;
    for line in content.lines() {
        if line.contains("Roadmap") && line.contains(":") {
            return Some(line.trim().to_string());
        }
    }
    None
}

fn build_file_changes(sha: &str) -> Vec<FileChange> {
    let numstat = git_cmd(&["diff", "--numstat", &format!("{sha}^..{sha}")]);
    numstat
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                Some(FileChange {
                    path: parts[2].to_string(),
                    insertions: parts[0].parse().unwrap_or(0),
                    deletions: parts[1].parse().unwrap_or(0),
                    status: if parts[0] == "0" && parts[1] == "0" {
                        "renamed".to_string()
                    } else if parts[1] == "0" {
                        "added".to_string()
                    } else {
                        "modified".to_string()
                    },
                })
            } else {
                None
            }
        })
        .collect()
}

fn empty_commit(line: &str) -> CommitInfo {
    CommitInfo {
        sha: line.to_string(),
        short: String::new(),
        title: line.to_string(),
        author: String::new(),
        date: String::new(),
        commit_type: "unknown".into(),
        scope: "unknown".into(),
        is_phase: false,
        phase: None,
        insertions: 0,
        deletions: 0,
        files: Vec::new(),
        body_sections: Vec::new(),
    }
}

// --- Commit diff endpoint ---

#[derive(Serialize)]
pub struct CommitDiffResult {
    pub sha: String,
    pub title: String,
    pub files: Vec<FileDiff>,
}

#[derive(Serialize)]
pub struct FileDiff {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
    pub hunks: Vec<DiffHunk>,
}

#[derive(Serialize)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
}

#[derive(Serialize)]
pub struct DiffLine {
    pub kind: String,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

/// Working-tree line ceiling (per diff source) before truncation. Guards
/// the auto-rendered VERIFY diff-viewer (Phase H) against a multi-MB JSON
/// from a huge refactor / vendored bundle. Beyond it, the source is cut at
/// a line boundary and [`WorkingTreeDiff::truncated`] is set. Sprint 80
/// Phase F (preflight P2 hardening).
const MAX_DIFF_LINES: usize = 20_000;

/// The repo working-tree diff, computed in Rust (the single source of truth
/// — never a divergent JS diff; kickoff invariant #11). Sprint 80 Phase F.
///
/// `unstaged` = `git diff`, `staged` = `git diff --cached`. A partially
/// staged file legitimately appears in BOTH (git semantics), hence the
/// two-array envelope rather than a flat per-path list. `head` is the short
/// HEAD sha for freshness (`run@<rev>`). Untracked files are absent (they
/// are not in `git diff`, consistent with `git_dirty_files`). `old_lineno`/
/// `new_lineno` serialize as `null` when absent — the Phase H Zod contract
/// is `.nullable()`, and the envelope is `{head,unstaged,staged,truncated}`.
#[derive(Serialize)]
pub struct WorkingTreeDiff {
    pub head: String,
    pub unstaged: Vec<FileDiff>,
    pub staged: Vec<FileDiff>,
    pub truncated: bool,
}

pub fn working_tree_diff_data(root: &Path) -> WorkingTreeDiff {
    let head = git_cmd_in(root, &["rev-parse", "--short", "HEAD"]);
    let (unstaged_raw, t_unstaged) = bounded_working_tree_diff(root, false);
    let (staged_raw, t_staged) = bounded_working_tree_diff(root, true);
    WorkingTreeDiff {
        head,
        unstaged: parse_unified_diff(&unstaged_raw),
        staged: parse_unified_diff(&staged_raw),
        truncated: t_unstaged || t_staged,
    }
}

/// Run a deterministic working-tree `git diff` and cap it at
/// [`MAX_DIFF_LINES`] lines. `-c color.ui=false` + `--no-color` neutralise a
/// global `color.ui=always` (ANSI codes would break `parse_unified_diff`);
/// `--no-ext-diff` neutralises a user `diff.external`. No user input reaches
/// the command, so no `--end-of-options`/`is_safe_git_rev` is needed.
fn bounded_working_tree_diff(root: &Path, cached: bool) -> (String, bool) {
    let mut args: Vec<&str> = vec![
        "-c",
        "color.ui=false",
        "diff",
        "--no-color",
        "-U3",
        "--no-ext-diff",
    ];
    if cached {
        args.push("--cached");
    }
    let raw = git_cmd_in(root, &args);
    let mut lines: Vec<&str> = raw.lines().collect();
    let truncated = lines.len() > MAX_DIFF_LINES;
    if truncated {
        lines.truncate(MAX_DIFF_LINES);
    }
    (lines.join("\n"), truncated)
}

pub fn commit_diff_data(sha: &str) -> Option<CommitDiffResult> {
    // `--end-of-options` forces git to treat `sha` as a revision, never an
    // option, even if a guard upstream were bypassed (defense in depth vs
    // git option injection — S71 Phase D retro-Codex).
    let title = git_cmd(&["log", "-1", "--format=%s", "--end-of-options", sha]);
    if title.is_empty() {
        return None;
    }

    let raw_diff = git_cmd(&["diff", "-U3", "--end-of-options", &format!("{sha}^..{sha}")]);
    let files = parse_unified_diff(&raw_diff);

    Some(CommitDiffResult {
        sha: sha.to_string(),
        title,
        files,
    })
}

fn parse_unified_diff(raw: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let mut current_file: Option<FileDiff> = None;
    let mut current_hunk: Option<DiffHunk> = None;
    let mut old_line = 0u32;
    let mut new_line = 0u32;

    let hunk_re = Regex::new(r"^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@(.*)").unwrap();

    for line in raw.lines() {
        if line.starts_with("diff --git") {
            if let Some(hunk) = current_hunk.take()
                && let Some(ref mut f) = current_file
            {
                f.hunks.push(hunk);
            }
            if let Some(f) = current_file.take() {
                files.push(f);
            }
            current_file = Some(FileDiff {
                path: String::new(),
                insertions: 0,
                deletions: 0,
                hunks: Vec::new(),
            });
        } else if let Some(rest) = line.strip_prefix("+++ b/") {
            if let Some(ref mut f) = current_file {
                f.path = rest.to_string();
            }
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            if current_file.as_ref().map(|f| f.path.is_empty()) == Some(true)
                && let Some(ref mut f) = current_file
            {
                f.path = rest.to_string();
            }
        } else if let Some(cap) = hunk_re.captures(line) {
            if let Some(hunk) = current_hunk.take()
                && let Some(ref mut f) = current_file
            {
                f.hunks.push(hunk);
            }
            old_line = cap[1].parse().unwrap_or(1);
            new_line = cap[2].parse().unwrap_or(1);
            current_hunk = Some(DiffHunk {
                header: line.to_string(),
                lines: Vec::new(),
            });
        } else if let Some(ref mut hunk) = current_hunk {
            if let Some(rest) = line.strip_prefix('+').filter(|_| !line.starts_with("+++")) {
                hunk.lines.push(DiffLine {
                    kind: "add".to_string(),
                    content: rest.to_string(),
                    old_lineno: None,
                    new_lineno: Some(new_line),
                });
                if let Some(ref mut f) = current_file {
                    f.insertions += 1;
                }
                new_line += 1;
            } else if let Some(rest) = line.strip_prefix('-').filter(|_| !line.starts_with("---")) {
                hunk.lines.push(DiffLine {
                    kind: "del".to_string(),
                    content: rest.to_string(),
                    old_lineno: Some(old_line),
                    new_lineno: None,
                });
                if let Some(ref mut f) = current_file {
                    f.deletions += 1;
                }
                old_line += 1;
            } else if let Some(rest) = line.strip_prefix(' ') {
                hunk.lines.push(DiffLine {
                    kind: "ctx".to_string(),
                    content: rest.to_string(),
                    old_lineno: Some(old_line),
                    new_lineno: Some(new_line),
                });
                old_line += 1;
                new_line += 1;
            }
        }
    }

    if let Some(hunk) = current_hunk.take()
        && let Some(ref mut f) = current_file
    {
        f.hunks.push(hunk);
    }
    if let Some(f) = current_file.take() {
        files.push(f);
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sprint 71 Phase D / G6: `sprint_history.rs` (1048 lines) shipped
    // off-sprint with zero tests. The pure parsers are the high-value,
    // hermetic surface — exercised here with string fixtures (no git). The
    // HTTP endpoint wiring is covered in `tests/operator_server.rs` via the
    // authenticated `TestServer` harness.

    #[test]
    fn parse_unified_diff_classifies_line_kinds() {
        // A minimal one-file unified diff with context/add/del lines. The
        // parser must classify each line and keep the inline code verbatim.
        let raw = concat!(
            "diff --git a/foo.txt b/foo.txt\n",
            "index 1111111..2222222 100644\n",
            "--- a/foo.txt\n",
            "+++ b/foo.txt\n",
            "@@ -1,3 +1,4 @@\n",
            " context line\n",
            "-removed line\n",
            "+added line one\n",
            "+added line two\n",
            " trailing context\n",
        );

        let files = parse_unified_diff(raw);
        assert_eq!(files.len(), 1, "one file in the diff");
        let f = &files[0];
        assert_eq!(f.path, "foo.txt");
        assert_eq!(f.insertions, 2, "two + lines");
        assert_eq!(f.deletions, 1, "one - line");

        let hunk = &f.hunks[0];
        let kinds: Vec<&str> = hunk.lines.iter().map(|l| l.kind.as_str()).collect();
        assert_eq!(kinds, vec!["ctx", "del", "add", "add", "ctx"]);

        // Inline code content is preserved (the `+`/`-`/` ` marker is stripped).
        assert!(
            hunk.lines
                .iter()
                .any(|l| l.kind == "add" && l.content == "added line one"),
            "added inline code retained"
        );
        assert!(
            hunk.lines
                .iter()
                .any(|l| l.kind == "del" && l.content == "removed line"),
            "removed inline code retained"
        );
    }

    #[test]
    fn working_tree_diff_separates_unstaged_and_staged() {
        // Sprint 80 Phase F: hermetic test of the working-tree diff against a
        // throwaway git repo, so hunk classification is deterministic (the
        // HTTP test in tests/operator_server.rs only asserts the envelope
        // shape on the live, dirty repo).
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(root)
                .args(args)
                .output()
                .expect("git command");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.test"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(root.join("a.txt"), "line1\nline2\nline3\n").expect("write a");
        std::fs::write(root.join("b.txt"), "x\n").expect("write b");
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "init"]);
        // Unstaged edit to a.txt (one line changed -> add + del + ctx).
        std::fs::write(root.join("a.txt"), "line1\nline2 changed\nline3\n").expect("edit a");
        // Staged edit to b.txt.
        std::fs::write(root.join("b.txt"), "x\ny\n").expect("edit b");
        run(&["add", "b.txt"]);

        let diff = working_tree_diff_data(root);
        assert!(
            !diff.head.is_empty(),
            "short HEAD sha present after a commit"
        );
        assert!(!diff.truncated, "small diff is not truncated");

        let a = diff
            .unstaged
            .iter()
            .find(|f| f.path == "a.txt")
            .expect("a.txt is an unstaged change");
        assert!(a.insertions >= 1 && a.deletions >= 1, "a.txt has +/- lines");
        let a_kinds: Vec<&str> = a
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter().map(|l| l.kind.as_str()))
            .collect();
        assert!(a_kinds.contains(&"add"), "add line classified");
        assert!(a_kinds.contains(&"del"), "del line classified");
        assert!(a_kinds.contains(&"ctx"), "context line classified");

        assert!(
            diff.staged.iter().any(|f| f.path == "b.txt"),
            "b.txt is a staged change"
        );
        // A purely-unstaged file must not leak into staged and vice-versa.
        assert!(
            !diff.staged.iter().any(|f| f.path == "a.txt"),
            "a.txt (unstaged only) absent from staged"
        );
    }

    #[test]
    fn working_tree_diff_truncates_beyond_cap() {
        // Sprint 80 Phase F (review F1): a diff beyond MAX_DIFF_LINES is cut at
        // a line boundary and `truncated` is set. A freshly staged large file
        // emits one `+` per content line, easily exceeding the cap.
        let tmp = tempfile::tempdir().expect("tempdir");
        let root = tmp.path();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .current_dir(root)
                .args(args)
                .output()
                .expect("git command");
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.test"]);
        run(&["config", "user.name", "t"]);
        std::fs::write(root.join("seed.txt"), "seed\n").expect("write seed");
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "init"]);
        // Stage a file with > MAX_DIFF_LINES content lines.
        let big: String = (0..MAX_DIFF_LINES + 100)
            .map(|i| format!("line {i}\n"))
            .collect();
        std::fs::write(root.join("big.txt"), big).expect("write big");
        run(&["add", "big.txt"]);

        let diff = working_tree_diff_data(root);
        assert!(
            diff.truncated,
            "a diff beyond MAX_DIFF_LINES must set truncated"
        );
    }

    #[test]
    fn extract_section_stops_at_next_header() {
        let md = concat!(
            "## Alpha\n",
            "alpha line 1\n",
            "alpha line 2\n",
            "## Beta\n",
            "beta line\n",
        );
        let section = extract_section(md, "## Alpha");
        assert!(section.contains("alpha line 1"));
        assert!(section.contains("alpha line 2"));
        assert!(
            !section.contains("beta line"),
            "must stop at next ## header"
        );
    }

    #[test]
    fn extract_verdict_reads_plan_adapt() {
        let content = "intro\n## Verdict: PLAN-ADAPT\nmore text";
        assert_eq!(
            extract_preflight_verdict_generic(content).as_deref(),
            Some("PLAN-ADAPT")
        );
    }

    #[test]
    fn phase_re_accepts_unbounded_multi_letter() {
        // PHASE_RE token is [A-Z]+[0-9]? (README §4) — guard against a
        // regression to the old capped [A-G][0-9]? (which dropped S77's
        // Phase N and any multi-letter / >G phase from the history).
        for (title, want) in [
            ("feat(core): Sprint 77 Phase N — title", "N"),
            ("feat(web): Sprint 79 Phase AA — title", "AA"),
            ("fix(x): Sprint 80 Phase F1 — title", "F1"),
        ] {
            let caps = PHASE_RE
                .captures(title)
                .unwrap_or_else(|| panic!("PHASE_RE no match: {title}"));
            assert_eq!(&caps[4], want, "phase token for {title}");
        }
    }

    #[test]
    fn sprint_summary_is_unbounded_and_case_insensitive() {
        // Regression guard: phase counting must span past G and read both the
        // lowercase active convention and an uppercase archive-style file.
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::write(p.join("sprint80_phase_a_review.md"), "## Verdict: PASS").unwrap();
        std::fs::write(p.join("sprint80_phase_h_review.md"), "## Verdict: PASS").unwrap(); // beyond G
        std::fs::write(
            p.join("sprint80_phase_B_review.md"), // UPPERCASE archive-style
            "intro\n## Verdict: PASS-PENDING",
        )
        .unwrap();
        let summary = build_sprint_summary(p, 80, "vX");
        assert_eq!(
            summary.phase_count, 3,
            "a + B + h all counted (case-insensitive, beyond the old G cap)"
        );
        assert_eq!(
            summary.phases_pass, 2,
            "only the two final PASS reviews count (B is PASS-PENDING)"
        );
    }
}
