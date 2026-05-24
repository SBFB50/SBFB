// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

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

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::Value::Object(context))?
    );
    Ok(())
}

struct SprintInfo {
    number: u32,
    current_phase: String,
}

fn detect_sprint(active_dir: &Path) -> Option<SprintInfo> {
    let entries = std::fs::read_dir(active_dir).ok()?;
    let mut max_sprint = 0u32;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(n) = name
            .strip_prefix("sprint")
            .and_then(|s| s.split('_').next())
            .and_then(|s| s.parse::<u32>().ok())
        {
            if n > max_sprint {
                max_sprint = n;
            }
        }
    }
    if max_sprint == 0 {
        return None;
    }

    let phase = detect_current_phase(active_dir, max_sprint);
    Some(SprintInfo {
        number: max_sprint,
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
                if content.contains("## Verdict: PASS") {
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
