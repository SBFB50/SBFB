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
