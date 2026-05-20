from __future__ import annotations

import argparse
import importlib.util
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AGENTCTL_PATH = ROOT / "scripts" / "agent" / "agentctl.py"


def load_agentctl():
    spec = importlib.util.spec_from_file_location("agentctl_under_test", AGENTCTL_PATH)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def test_prompt_kinds_include_universal():
    agentctl = load_agentctl()

    assert agentctl.PROMPT_KINDS["universal"] == "universal.md"


def test_phase_title_accepts_refactor_phase_commit():
    agentctl = load_agentctl()

    sprint, phase = agentctl.phase_from_title("refactor(sprint35): Sprint 35 Phase A - native coordinator")

    assert sprint == "35"
    assert phase == "a"


def test_phase_title_rejects_scope_title_mismatch():
    agentctl = load_agentctl()

    title = "feat(sprint66): Sprint 65 Phase A bad scope"
    sprint, phase = agentctl.phase_from_title(title)

    assert (sprint, phase) == ("65", "a")
    assert agentctl.phase_title_errors(title, sprint, phase) == [
        "commit scope sprint66 conflicts with title Sprint 65"
    ]


def test_phase_title_ignores_planning_commit():
    agentctl = load_agentctl()

    sprint, phase = agentctl.phase_from_title("chore(planning): sprint 35 audit findings PASS")

    assert sprint is None
    assert phase is None


def test_commit_title_strips_utf8_bom():
    agentctl = load_agentctl()

    assert agentctl.commit_title("\ufefffeat(sprint67): Sprint 67 Phase B search\n") == (
        "feat(sprint67): Sprint 67 Phase B search"
    )


def test_prompt_context_includes_claude_sources(monkeypatch):
    agentctl = load_agentctl()

    def fake_git(args: list[str]) -> str:
        if args == ["status", "--short"]:
            return " M scripts/agent/agentctl.py"
        return ""

    monkeypatch.setattr(agentctl, "git", fake_git)
    args = argparse.Namespace(kind="universal", sprint=35, phase="A", depth="standard")

    context = agentctl.prompt_context(args)

    assert "- CLAUDE.md" in context
    assert "- docs/claude/README.md" in context
    assert "- docs/claude/SPRINT_LOG.md" in context


def test_review_gate_rejects_pass_pending_and_pending_codex():
    agentctl = load_agentctl()

    errors = agentctl.review_gate_errors("## Verdict : PASS-PENDING\nReady for Codex verification\n")

    assert any("PASS-PENDING" in error for error in errors)
    assert any("Codex verification" in error for error in errors)


def test_codex_skip_text_does_not_exempt_missing_artifact(tmp_path, monkeypatch):
    agentctl = load_agentctl()
    monkeypatch.setattr(agentctl, "ROOT", tmp_path)

    errors = agentctl.codex_review_errors(
        "feat(sprint67): Sprint 67 Phase C factory",
        "## Codex verification : skipped",
        "67",
        "c",
        set(),
    )

    assert errors == ["missing Codex review artifact: .planning/active/sprint67_phase_c_codex_review.md"]


def test_staged_diff_check_errors_reports_git_check_failure(monkeypatch):
    agentctl = load_agentctl()

    def fake_run(args: list[str], *, cwd=None, check: bool = False):
        assert args == ["git", "diff", "--cached", "--check"]
        return subprocess.CompletedProcess(args, 2, "file.md:1: trailing whitespace.\n")

    monkeypatch.setattr(agentctl, "run", fake_run)

    assert agentctl.staged_diff_check_errors() == [
        "git diff --cached --check: file.md:1: trailing whitespace."
    ]


def test_phase_body_requires_all_nine_sections():
    agentctl = load_agentctl()

    errors = agentctl.commit_body_section_errors(
        "fix(sprint67): Sprint 67 Phase B search",
        "## Contexte\n\n## Codex verification\n",
        "67",
        "b",
    )

    assert errors
    assert "## Fichiers" in errors[0]
    assert "## Carry closure" in errors[0]


def test_nexus_skip_env_does_not_bypass_auditor_gate(tmp_path, monkeypatch):
    agentctl = load_agentctl()
    review = tmp_path / "sprint67_phase_b_review.md"
    review.write_text("## Verdict : PASS-PENDING\nReady for Codex verification\n", encoding="utf-8")

    monkeypatch.setenv("NEXUS_SKIP_PHASE_AUDITOR", "1")
    monkeypatch.setattr(
        agentctl,
        "parse_commit_message",
        lambda _: "feat(sprint67): Sprint 67 Phase B search\n\nbody",
    )
    monkeypatch.setattr(agentctl, "review_file", lambda sprint, phase: review)

    assert agentctl.cmd_auditor_gate(argparse.Namespace(message_file=None)) == 2


def test_verify_on_write_uses_all_targets_for_rust_bin_crates(monkeypatch):
    agentctl = load_agentctl()
    calls: list[list[str]] = []

    def fake_run(args: list[str], *, cwd=None, check: bool = False):
        calls.append(args)
        return subprocess.CompletedProcess(args, 0, "")

    def fake_which(name: str):
        return None if name == "semgrep" else name

    monkeypatch.setattr(agentctl, "run", fake_run)
    monkeypatch.setattr(agentctl.shutil, "which", fake_which)
    args = argparse.Namespace(file="crates/nexus-launcher/src/main.rs")

    assert agentctl.cmd_verify_on_write(args) == 0
    assert calls[0] == [
        "cargo",
        "clippy",
        "-p",
        "nexus-launcher",
        "--all-targets",
        "--locked",
        "--",
        "-D",
        "warnings",
    ]
