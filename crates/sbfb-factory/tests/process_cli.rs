// SPDX-License-Identifier: AGPL-3.0-or-later

use std::process::Command;

fn factory_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sbfb-factory"))
}

#[test]
fn prompt_handoff_assembles() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "handoff", "--depth", "deep"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Handoff"), "should contain handoff content");
    assert!(stdout.contains("Sprint context"), "should have section 1");
    assert!(stdout.contains("Next actions"), "should have section 9");
}

#[test]
fn prompt_preflight_assembles() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "preflight",
            "--depth",
            "deep",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("S1"), "should contain S1 scan");
    assert!(stdout.contains("S2"), "should contain S2 scan");
    assert!(stdout.contains("S3"), "should contain S3 scan");
    assert!(stdout.contains("S4"), "should contain S4 scan");
}

#[test]
fn prompt_phase_review_assembles() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "phase-review",
            "--depth",
            "deep",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Staging coherence"), "dimension 1");
    assert!(stdout.contains("Patterns drift"), "dimension 8");
    assert!(stdout.contains("Horizon"), "dimension 9");
    assert!(stdout.contains("Livrables check"), "dimension 10");
    assert!(stdout.contains("Carry routing"), "dimension 11");
}

#[test]
fn prompt_commit_body_assembles() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "commit-body",
            "--depth",
            "deep",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("## Contexte"), "9 sections template");
    assert!(stdout.contains("## Codex verification"), "codex section");
    assert!(stdout.contains("Anti-Patterns"), "anti-patterns section");
}

#[test]
fn prompt_audit_gate_assembles() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "audit-gate",
            "--depth",
            "deep",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Track A"), "track A suites");
    assert!(stdout.contains("Track B"), "track B security");
    assert!(stdout.contains("Track I"), "track I meta-process");
}

#[test]
fn prompt_phase_auditor_assembles() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "phase-auditor",
            "--depth",
            "deep",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("7 Dimensions"), "7 dimensions section");
    assert!(stdout.contains("opinion-first"), "opinion-first pattern");
}

#[test]
fn process_context_includes_agent_system() {
    let output = factory_bin()
        .args(["process", "context"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("AGENT_SYSTEM"),
        "context should reference AGENT_SYSTEM"
    );
    assert!(
        stdout.contains("prompt_kinds"),
        "context should list prompt kinds"
    );
    assert!(
        stdout.contains("process_docs"),
        "context should list process docs"
    );
}

#[test]
fn prompt_alias_review_resolves() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "review"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "alias 'review' should resolve to phase-review"
    );
    assert!(
        stdout.contains("Phase Review"),
        "should contain phase-review content"
    );
}

#[test]
fn prompt_alias_auditor_resolves() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "auditor"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "alias 'auditor' should resolve to phase-auditor"
    );
    assert!(
        stdout.contains("Auditeur"),
        "should contain phase-auditor content"
    );
}

#[test]
fn prompt_alias_audit_resolves() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "audit"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "alias 'audit' should resolve to audit-gate"
    );
    assert!(
        stdout.contains("Track A"),
        "should contain audit-gate content"
    );
}

#[test]
fn prompt_universal_assembles() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "universal"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("Universal LLM Agent"),
        "should contain universal content"
    );
}

#[test]
fn prompt_base_assembles() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "base"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("nexus-grid"), "should contain base content");
}

#[test]
fn prompt_unknown_kind_fails() {
    let output = factory_bin()
        .args(["process", "prompt", "--kind", "nonexistent"])
        .output()
        .expect("failed to run");
    assert!(!output.status.success(), "unknown kind should fail");
}

#[test]
fn prompt_provider_local_strips_cloud() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "preflight",
            "--provider",
            "local",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        !stdout.to_lowercase().contains("websearch"),
        "local provider should strip WebSearch references"
    );
    assert!(
        !stdout.to_lowercase().contains("context7"),
        "local provider should strip context7 references"
    );
}

#[test]
fn prompt_provider_claude_keeps_all() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "preflight",
            "--provider",
            "claude",
        ])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    assert!(
        stdout.contains("Preflight"),
        "should keep full content for claude"
    );
}

#[test]
fn process_context_json_parsable() {
    let output = factory_bin()
        .args(["process", "context"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("context output should be valid JSON");
    assert!(parsed.is_object());
    assert!(parsed.get("head").is_some(), "should have head field");
    assert!(parsed.get("branch").is_some(), "should have branch field");
    assert!(parsed.get("repo").is_some(), "should have repo field");
}

// --- status-sprint tests ---

#[test]
fn status_sprint_detects_active_kickoff() {
    let output = factory_bin()
        .args(["process", "status-sprint"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Sprint"), "should mention sprint number");
    assert!(stdout.contains("kickoff"), "should mention kickoff status");
}

#[test]
fn status_sprint_json_output() {
    let output = factory_bin()
        .args(["process", "status-sprint", "--json"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&stdout).expect("should be valid JSON");
    assert!(parsed.get("sprint").is_some(), "should have sprint field");
    assert!(
        parsed.get("current_phase").is_some(),
        "should have current_phase"
    );
    assert!(parsed.get("phases").is_some(), "should have phases array");
    assert!(
        parsed["has_kickoff"].as_bool() == Some(true),
        "should detect kickoff"
    );
}

#[test]
fn status_sprint_no_active_sprint() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(dir.path().join(".planning/active")).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    let output = factory_bin()
        .args(["process", "status-sprint"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    assert!(
        !output.status.success(),
        "should fail when no active sprint"
    );
}

// --- lint-planning tests ---

#[test]
fn lint_planning_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let active = dir.path().join(".planning/active");
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(active.join("sprint70_kickoff.md"), "# Sprint 70").unwrap();
    std::fs::write(active.join("sprint70_plan.md"), "# Plan").unwrap();
    std::fs::write(
        active.join("sprint70_phase_A_review.md"),
        "## Verdict: PASS\nClean review.",
    )
    .unwrap();
    let output = factory_bin()
        .args(["process", "lint-planning", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "lint should pass on clean fixtures: {}",
        stdout
    );
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["ok"], true, "should report ok");
}

#[test]
fn lint_planning_detects_orphan_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let active = dir.path().join(".planning/active");
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(active.join("sprint70_kickoff.md"), "# Sprint 70").unwrap();
    std::fs::write(active.join("sprint70_plan.md"), "# Plan").unwrap();
    std::fs::write(active.join("sprint65_old_file.md"), "# Old").unwrap();
    let output = factory_bin()
        .args(["process", "lint-planning", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let warnings = parsed["warnings"].as_array().unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| { w["code"].as_str() == Some("ORPHAN_FILE") }),
        "should detect orphan file"
    );
}

#[test]
fn lint_planning_detects_pass_pending() {
    let dir = tempfile::tempdir().expect("tempdir");
    let active = dir.path().join(".planning/active");
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(active.join("sprint70_kickoff.md"), "# Sprint 70").unwrap();
    std::fs::write(active.join("sprint70_plan.md"), "# Plan").unwrap();
    std::fs::write(
        active.join("sprint70_phase_A_review.md"),
        "## Verdict: PASS-PENDING\nReview en cours.",
    )
    .unwrap();
    let output = factory_bin()
        .args(["process", "lint-planning", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["ok"], false, "should report not ok");
    let errors = parsed["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e["code"].as_str() == Some("STALE_PASS_PENDING")),
        "should detect PASS-PENDING"
    );
}

// --- audit-commit tests ---

#[test]
fn audit_commit_valid_phase_commit() {
    // Use Phase F SHA — HEAD may be a chore commit
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "6fb95df", "--json"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "6fb95df should be a valid phase commit: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["ok"], true, "should pass");
    assert_eq!(parsed["is_phase_commit"], true);
}

#[test]
fn audit_commit_non_phase_commit() {
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "c4494a6", "--json"])
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["is_phase_commit"], false,
        "chore commit should not be phase commit"
    );
    assert_eq!(parsed["ok"], true, "non-phase commit should pass");
}

#[test]
fn audit_commit_missing_body_sections() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::write(repo.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "file.txt"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args([
            "commit",
            "-m",
            "feat(test): Sprint 1 Phase A — test\n\nNo body sections at all.",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "HEAD", "--json"])
        .current_dir(repo)
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "should fail on missing sections");
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["ok"], false);
    let issues = parsed["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i.as_str()
                .map(|s| s.contains("missing body sections"))
                .unwrap_or(false)
        }),
        "should report missing body sections"
    );
}

#[test]
fn audit_commit_missing_review() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::create_dir_all(repo.join(".planning/active")).unwrap();
    std::fs::write(repo.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo)
        .output()
        .unwrap();
    let body = [
        "feat(test): Sprint 1 Phase A — test",
        "",
        "## Contexte",
        "ctx",
        "## Fichiers",
        "f",
        "## Delta tests",
        "d",
        "## Verification",
        "v",
        "## Scope cuts",
        "s",
        "## G8 traceability",
        "g",
        "## Pre-launch protocol",
        "p",
        "## Codex verification",
        "c",
        "## Carry closure",
        "cc",
    ]
    .join("\n");
    Command::new("git")
        .args(["commit", "-m", &body])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "HEAD", "--json"])
        .current_dir(repo)
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(parsed["ok"], false);
    let issues = parsed["issues"].as_array().unwrap();
    assert!(
        issues.iter().any(|i| {
            i.as_str()
                .map(|s| s.contains("missing review"))
                .unwrap_or(false)
        }),
        "should report missing review file"
    );
}

// --- Phase F: chore(sprintN) Phase gate + verdict exact ---

#[test]
fn audit_commit_chore_sprint_phase_requires_review() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::create_dir_all(repo.join(".planning/active")).unwrap();
    std::fs::write(repo.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo)
        .output()
        .unwrap();
    let body = [
        "chore(sprint70): Sprint 70 Phase B — dette pair",
        "",
        "## Contexte",
        "ctx",
        "## Fichiers",
        "f",
        "## Delta tests",
        "d",
        "## Verification",
        "v",
        "## Scope cuts",
        "s",
        "## G8 traceability",
        "g",
        "## Pre-launch protocol",
        "p",
        "## Codex verification",
        "c",
        "## Carry closure",
        "cc",
    ]
    .join("\n");
    Command::new("git")
        .args(["commit", "-m", &body])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "HEAD", "--json"])
        .current_dir(repo)
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["is_phase_commit"], true,
        "chore(sprint70) Sprint 70 Phase B should be a phase commit"
    );
    assert_eq!(
        parsed["ok"], false,
        "should fail because review file is missing"
    );
}

#[test]
fn audit_commit_chore_planning_not_phase() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = dir.path();
    Command::new("git")
        .args(["init"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::write(repo.join("file.txt"), "content").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "chore(planning): reconcile Phase A review"])
        .current_dir(repo)
        .output()
        .unwrap();
    let output = factory_bin()
        .args(["process", "audit-commit", "--rev", "HEAD", "--json"])
        .current_dir(repo)
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["is_phase_commit"], false,
        "chore(planning) without Sprint N Phase X is not a phase commit"
    );
    assert_eq!(parsed["ok"], true, "non-phase commit should pass");
}

#[test]
fn verdict_exact_rejects_spaced_colon() {
    let dir = tempfile::tempdir().expect("tempdir");
    let active = dir.path().join(".planning/active");
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(active.join("sprint70_kickoff.md"), "# Sprint 70").unwrap();
    std::fs::write(active.join("sprint70_plan.md"), "# Plan").unwrap();
    std::fs::write(
        active.join("sprint70_phase_A_review.md"),
        "## Verdict : PASS\nSpaced colon should be rejected.",
    )
    .unwrap();
    let output = factory_bin()
        .args(["process", "lint-planning", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["ok"], false,
        "spaced colon verdict should be detected as invalid: {}",
        stdout
    );
    let errors = parsed["errors"].as_array().unwrap();
    assert!(
        errors
            .iter()
            .any(|e| e["code"].as_str() == Some("INVALID_VERDICT_FORMAT")),
        "should flag INVALID_VERDICT_FORMAT for spaced colon: {:?}",
        errors
    );
}

#[test]
fn status_sprint_spaced_verdict_not_phase_complete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let active = dir.path().join(".planning/active");
    std::fs::create_dir_all(&active).unwrap();
    std::fs::create_dir_all(dir.path().join(".git")).unwrap();
    std::fs::write(active.join("sprint70_kickoff.md"), "# Sprint 70").unwrap();
    std::fs::write(active.join("sprint70_plan.md"), "# Plan").unwrap();
    std::fs::write(
        active.join("sprint70_phase_A_review.md"),
        "## Verdict : PASS\nSpaced colon — not valid.",
    )
    .unwrap();
    let output = factory_bin()
        .args(["process", "status-sprint", "--json"])
        .current_dir(dir.path())
        .output()
        .expect("failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(
        parsed["current_phase"].as_str(),
        Some("A"),
        "phase A with spaced colon verdict should NOT be marked complete: {}",
        stdout
    );
}

#[test]
fn prompt_provider_human_accepted() {
    let output = factory_bin()
        .args([
            "process",
            "prompt",
            "--kind",
            "preflight",
            "--provider",
            "human",
        ])
        .output()
        .expect("failed to run");
    assert!(
        output.status.success(),
        "human provider should be accepted: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
