#!/usr/bin/env python3
"""Vendor-neutral agent process helper for the nexus repository.

The script intentionally uses only the Python standard library. It is called by
Git hooks, human operators, and any model provider that can execute commands.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

PROMPT_KINDS = {
    "base": "base.md",
    "universal": "universal.md",
    "preflight": "preflight.md",
    "phase-review": "phase-review.md",
    "phase-auditor": "phase-auditor.md",
    "commit-body": "commit-body.md",
}

PHASE_TITLE_RE = re.compile(
    r"^(feat|fix|docs|chore|test|refactor)\(sprint(?P<sprint>\d+)\):\s*"
    r"Sprint\s+(?P=sprint)\s+Phase\s+(?P<phase>[A-Z]+[0-9]?)\b"
)
PHASE_TITLE_FALLBACK_RE = re.compile(
    r"^(feat|fix|docs|chore|test|refactor)\([^)]+\):\s*"
    r"Sprint\s+(?P<sprint>\d+)\s+Phase\s+(?P<phase>[A-Z]+[0-9]?)\b"
)
FINAL_PASS_RE = re.compile(r"^## Verdict\s*:\s*PASS\s*$", re.MULTILINE)
REQUIRED_PHASE_BODY_SECTIONS = (
    ("Contexte", r"^## Contexte\s*$"),
    ("Fichiers", r"^## Fichiers\s*$"),
    ("Delta tests", r"^## Delta tests\s*$"),
    ("Verification", r"^## V[eé]rification\b"),
    ("Scope cuts", r"^## Scope cuts\s*$"),
    ("G8 traceability", r"^## G8 traceability\s*$"),
    ("Pre-launch protocol", r"^## Pre-launch protocol\s*$"),
    ("Codex verification", r"^## Codex verification\s*$"),
    ("Carry closure", r"^## Carry closure(?:\s*/\s*Unblock)?\s*$"),
)


def repo_root() -> Path:
    cur = Path.cwd().resolve()
    for path in [cur, *cur.parents]:
        if (path / "Cargo.toml").exists() and (path / ".git").exists():
            return path
    return cur


ROOT = repo_root()


def rel(path: Path) -> str:
    try:
        return path.resolve().relative_to(ROOT).as_posix()
    except ValueError:
        return path.as_posix()


def run(args: list[str], *, cwd: Path | None = None, check: bool = False) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        args,
        cwd=str(cwd or ROOT),
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=check,
    )


def git(args: list[str]) -> str:
    proc = run(["git", *args])
    return proc.stdout.strip()


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="ignore")
    except FileNotFoundError:
        return ""


def staged_files() -> list[str]:
    out = git(["diff", "--cached", "--name-only"])
    return [line.strip() for line in out.splitlines() if line.strip()]


def staged_diff() -> str:
    return git(["diff", "--cached", "-U0"])


def staged_diff_check_errors() -> list[str]:
    proc = run(["git", "diff", "--cached", "--check"])
    if proc.returncode == 0:
        return []
    lines = [line.strip() for line in proc.stdout.splitlines() if line.strip()]
    if not lines:
        return ["git diff --cached --check failed"]
    return [f"git diff --cached --check: {line}" for line in lines]


def current_sprint() -> int | None:
    found: list[int] = []
    for base in [ROOT / ".planning" / "active", ROOT / ".planning" / "archive"]:
        if not base.exists():
            continue
        for path in base.rglob("sprint*_*.md"):
            m = re.search(r"sprint(\d+)_", path.name)
            if m:
                found.append(int(m.group(1)))
    return max(found) if found else None


def print_block(title: str, lines: list[str]) -> None:
    print(f"## {title}")
    for line in lines:
        print(line)
    print()


def cmd_context(_: argparse.Namespace) -> int:
    sprint = current_sprint()
    status = git(["status", "--short"])
    paths = [
        "AGENTS.md",
        "CLAUDE.md",
        "docs/agent/PROCESS.md",
        "docs/agent/TOOLING.md",
        "docs/claude/README.md",
        "docs/claude/SPRINT_LOG.md",
        "prompts/agent/",
        "scripts/agent/agentctl.py",
        ".githooks/",
    ]
    print_block("Repository", [str(ROOT)])
    print_block("Active Sprint", [str(sprint) if sprint else "unknown"])
    print_block("Process Paths", paths)
    print_block("Git Status", status.splitlines() if status else ["clean"])
    return 0


def prompt_context(args: argparse.Namespace) -> str:
    sprint = args.sprint if args.sprint is not None else current_sprint()
    phase = args.phase or "?"
    status = git(["status", "--short"])
    diff_stat = git(["diff", "--stat"])
    staged_stat = git(["diff", "--cached", "--stat"])
    depth = args.depth
    lines = [
        "# Runtime Context",
        "",
        f"- Repo: {ROOT}",
        f"- Sprint: {sprint if sprint is not None else 'unknown'}",
        f"- Phase: {phase}",
        f"- Prompt kind: {args.kind}",
        f"- Context depth: {depth}",
        "",
        "## Required repo references",
        "",
        "- AGENTS.md",
        "- CLAUDE.md",
        "- docs/agent/PROCESS.md",
        "- docs/agent/TOOLING.md",
        "- docs/claude/README.md",
        "- docs/claude/SPRINT_LOG.md",
        "- .planning/active/",
        "",
        "## Git status",
        "",
        "```text",
        status or "clean",
        "```",
        "",
        "## Worktree diff stat",
        "",
        "```text",
        diff_stat or "(none)",
        "```",
        "",
        "## Staged diff stat",
        "",
        "```text",
        staged_stat or "(none)",
        "```",
        "",
    ]
    if depth == "deep":
        branch = git(["branch", "--show-current"]) or "(detached)"
        head = git(["rev-parse", "--short", "HEAD"]) or "(unknown)"
        staged_names = git(["diff", "--cached", "--name-status"])
        unstaged_names = git(["diff", "--name-status"])
        recent = git(["log", "--oneline", "-5"])
        lines.extend(
            [
                "## Deep context",
                "",
                f"- Branch: {branch}",
                f"- HEAD: {head}",
                "",
                "### Staged files",
                "",
                "```text",
                staged_names or "(none)",
                "```",
                "",
                "### Unstaged files",
                "",
                "```text",
                unstaged_names or "(none)",
                "```",
                "",
                "### Recent commits",
                "",
                "```text",
                recent or "(none)",
                "```",
                "",
            ]
        )
    return "\n".join(lines)


def cmd_prompt(args: argparse.Namespace) -> int:
    prompt_file = PROMPT_KINDS[args.kind]
    path = ROOT / "prompts" / "agent" / prompt_file
    body = read_text(path)
    if not body:
        print(f"[agentctl] missing prompt: {rel(path)}", file=sys.stderr)
        return 2
    if args.sprint is not None:
        body = body.replace("{SPRINT}", str(args.sprint))
    if args.phase:
        body = body.replace("{PHASE}", args.phase)
    print(body.rstrip())
    print()
    print(prompt_context(args).rstrip())
    return 0


def codex_prompt_path(sprint: int | str, phase: str, recheck: int | None = None) -> Path:
    phase_token = phase.strip().upper()
    suffix = f"_RECHECK_{recheck:02d}" if recheck is not None else ""
    return ROOT / ".git" / f"CODEX_SPRINT{sprint}_PHASE_{phase_token}{suffix}.txt"


def cmd_codex_prompt_path(args: argparse.Namespace) -> int:
    if args.recheck is not None and args.recheck <= 0:
        print("[agentctl] --recheck must be a positive integer", file=sys.stderr)
        return 2

    path = codex_prompt_path(args.sprint, args.phase, args.recheck)
    print(str(path) if args.absolute else rel(path))
    return 0


def normalize_file(path: str) -> Path:
    p = Path(path)
    if not p.is_absolute():
        p = ROOT / p
    return p.resolve()


def should_skip_file(path: Path) -> bool:
    r = rel(path)
    skipped = (
        ".planning/",
        "docs/",
        "target/",
        "node_modules/",
        ".venv/",
        "dist/",
        "build/",
        ".git/",
        ".claude/",
    )
    return r.startswith(skipped)


def crate_for_file(path: Path) -> str | None:
    parts = rel(path).split("/")
    if len(parts) >= 3 and parts[0] == "crates":
        return parts[1]
    return None


def tail(text: str, n: int = 50) -> str:
    lines = text.splitlines()
    return "\n".join(lines[-n:])


def cmd_verify_on_write(args: argparse.Namespace) -> int:
    path = normalize_file(args.file)
    if not path.exists() or should_skip_file(path):
        return 0

    r = rel(path)
    suffix = path.suffix.lower()
    command: list[str] | None = None
    cwd = ROOT

    if suffix == ".rs":
        crate = crate_for_file(path)
        if not crate:
            return 0
        command = ["cargo", "clippy", "-p", crate, "--all-targets", "--locked", "--", "-D", "warnings"]
        print(f"[verify-on-write] rs: {' '.join(command)}", file=sys.stderr)
    elif suffix == ".py":
        if shutil.which("uv") is None:
            print("[verify-on-write] skip: uv not found", file=sys.stderr)
            return 0
        command = ["uv", "run", "ruff", "check", r]
        print(f"[verify-on-write] py: {' '.join(command)}", file=sys.stderr)
    elif suffix in {".ts", ".tsx", ".js", ".jsx"}:
        if not r.startswith("web/"):
            return 0
        if not (ROOT / "web" / "node_modules").exists():
            print("[verify-on-write] skip: web/node_modules missing", file=sys.stderr)
            return 0
        command = ["npx", "--no-install", "eslint", r.removeprefix("web/")]
        cwd = ROOT / "web"
        print(f"[verify-on-write] ts: {' '.join(command)}", file=sys.stderr)
    else:
        return 0

    proc = run(command, cwd=cwd)
    if proc.returncode != 0:
        print(tail(proc.stdout, 60), file=sys.stderr)
        print(f"[verify-on-write] BLOCK: command failed for {r}", file=sys.stderr)
        return 2

    semgrep = shutil.which("semgrep")
    semgrep_config = ROOT / ".semgrep" / "sbfb.yml"
    if semgrep and semgrep_config.exists():
        scan = run(
            [
                semgrep,
                "--config",
                str(semgrep_config),
                "--severity",
                "WARNING",
                "--severity",
                "ERROR",
                "--error",
                "--quiet",
                str(path),
            ]
        )
        if scan.returncode != 0:
            print(tail(scan.stdout, 60), file=sys.stderr)
            print(f"[verify-on-write] BLOCK: semgrep failed for {r}", file=sys.stderr)
            return 2
    return 0


def tracked(path: str) -> bool:
    return run(["git", "ls-files", "--error-unmatch", path]).returncode == 0


def pub_mod_errors(diff: str, staged: set[str]) -> list[str]:
    errors: list[str] = []
    current_file: str | None = None
    for line in diff.splitlines():
        m = re.match(r"^\+\+\+ b/(.*\.rs)$", line)
        if m:
            current_file = m.group(1)
            continue
        m = re.match(r"^\+pub\s+mod\s+([a-z_][a-z0-9_]*)\s*;", line)
        if not m or not current_file:
            continue
        # Vendored third-party crates (the Sprint 77 llama.cpp fork under vendor/) use the
        # modern `foo.rs` + `foo/` module layout this same-dir resolution does not follow;
        # they are not SBFB-authored, so skip the coherence check for them.
        if current_file.startswith("vendor/"):
            continue
        mod_name = m.group(1)
        current_dir = str(Path(current_file).parent).replace("\\", "/")
        candidates = [f"{current_dir}/{mod_name}.rs", f"{current_dir}/{mod_name}/mod.rs"]
        exists = [c for c in candidates if (ROOT / c).exists()]
        if not exists:
            errors.append(
                f"pub mod {mod_name} added in {current_file}, but neither {candidates[0]} nor {candidates[1]} exists"
            )
            continue
        if not any(c in staged or tracked(c) for c in candidates):
            errors.append(f"pub mod {mod_name} added, but module file is untracked or unstaged")
    return errors


def loc_plan_errors(files: list[str], diff: str) -> list[str]:
    errors: list[str] = []
    plan_files = [f for f in files if re.search(r"sprint\d+_plan\.md$", f) and "/archive/" not in f]
    if not plan_files:
        return errors
    patterns = re.compile(
        r"~\s*\d+\s*(LOC|lignes)|environ\s+\d+\s+LOC|budget\s+LOC|LOC\s+total",
        re.IGNORECASE,
    )
    current_file: str | None = None
    for line in diff.splitlines():
        m = re.match(r"^\+\+\+ b/(.*)$", line)
        if m:
            current_file = m.group(1)
            continue
        if current_file in plan_files and line.startswith("+") and not line.startswith("+++"):
            if patterns.search(line):
                errors.append(f"{current_file}: LOC estimate is forbidden in sprint plans: {line[1:].strip()}")
    return errors


def parse_commit_message(message_file: str | None) -> str:
    if not message_file:
        return ""
    path = normalize_file(message_file)
    if not path.exists():
        print(f"[agentctl] WARN: commit message file not found: {rel(path)}", file=sys.stderr)
        return ""
    return read_text(path)


def commit_title(message: str) -> str:
    for line in message.splitlines():
        stripped = line.strip().lstrip("\ufeff")
        if stripped and not stripped.startswith("#"):
            return stripped
    return ""


def file_reference_warnings(message: str) -> list[str]:
    warnings: list[str] = []
    if not message:
        return warnings
    # Strip URL-like tokens before scanning for repo paths. A regex-only path
    # scan otherwise turns `https://host/docs/foo.md` into a false missing-file
    # warning for `host/docs/foo.md`.
    message = re.sub(r"\b[a-zA-Z][a-zA-Z0-9+.-]*://\S+", " ", message)
    matches = re.finditer(
        r"\.?[A-Za-z0-9_./~-]+\.(?:md|rs|py|ts|tsx|toml|sh|json|yml|yaml)",
        message,
    )
    refs = sorted({match.group(0) for match in matches})
    for ref_path in refs:
        ref_path = ref_path.strip("`'\"()[]{}")
        ref_path = ref_path.rstrip(".,:;")
        ref_path = ref_path.replace("\\", "/")
        while ref_path.startswith("./"):
            ref_path = ref_path[2:]
        if "://" in ref_path or "example." in ref_path:
            continue
        if "/" not in ref_path:
            continue
        resolved = (ROOT / ref_path).resolve()
        try:
            resolved.relative_to(ROOT)
        except ValueError:
            warnings.append(f"commit message references file outside repo: {ref_path}")
            continue
        if not resolved.exists():
            warnings.append(f"commit message references missing file: {ref_path}")
    return warnings


def wire_warnings(files: list[str], diff: str, sprint: str | None, phase: str | None) -> list[str]:
    wire_files = [f for f in files if re.search(r"canonical\.rs|schemas/|schema|VERSION", f, re.IGNORECASE)]
    wire_diff = bool(
        re.search(r"^\+.*(_VERSION\s*[:=]|DOMAIN_|canonical_bytes|serde\(default\)|schema)", diff, re.MULTILINE)
    )
    if not wire_files and not wire_diff:
        return []
    warning = "wire-format surface staged"
    if wire_files:
        warning += ": " + ", ".join(wire_files)
    if wire_diff:
        warning += " (content marker detected)"
    warnings = [warning]
    if sprint and phase:
        preflight = ROOT / ".planning" / "active" / f"sprint{sprint}_phase_{phase}_preflight.md"
        content = read_text(preflight)
        if not content:
            warnings.append(f"missing preflight for Sprint {sprint} Phase {phase}; verify wire invariants manually")
        elif not re.search(r"S4.*full|full.*S4|FULL SCAN", content, re.IGNORECASE):
            warnings.append("wire-format change should use full S4 preflight scan")
    else:
        warnings.append("commit title does not expose sprint/phase; verify wire invariants manually")
    return warnings


def phase_from_title(title: str) -> tuple[str | None, str | None]:
    m = PHASE_TITLE_RE.match(title)
    if not m:
        m = PHASE_TITLE_FALLBACK_RE.match(title)
    if not m:
        return None, None
    return m.group("sprint"), m.group("phase").lower()


def phase_title_errors(title: str, sprint: str | None, phase: str | None) -> list[str]:
    if not sprint or not phase:
        return []
    errors: list[str] = []
    scope = re.match(r"^(feat|fix|docs|chore|test|refactor)\((?P<scope>[^)]*)\):", title)
    if scope:
        scope_sprint = re.search(r"sprint(?P<sprint>\d+)", scope.group("scope"), re.IGNORECASE)
        if scope_sprint and scope_sprint.group("sprint") != sprint:
            errors.append(
                f"commit scope sprint{scope_sprint.group('sprint')} conflicts with title Sprint {sprint}"
            )
    plan = ROOT / ".planning" / "active" / f"sprint{sprint}_plan.md"
    if plan.exists():
        plan_text = read_text(plan)
        if not re.search(rf"\bPhase\s+{re.escape(phase.upper())}\b", plan_text):
            errors.append(f"Sprint {sprint} Phase {phase.upper()} is not declared in {rel(plan)}")
    return errors


def commit_body_section_errors(title: str, message: str, sprint: str | None, phase: str | None) -> list[str]:
    if not sprint or not phase or not phase_commit_requires_codex(title):
        return []
    if not message.strip():
        return ["phase commit message is empty; expected 9 canonical body sections"]
    missing = [
        name
        for name, pattern in REQUIRED_PHASE_BODY_SECTIONS
        if not re.search(pattern, message, re.MULTILINE | re.IGNORECASE)
    ]
    if not missing:
        return []
    return ["missing canonical phase body sections: " + ", ".join(f"## {name}" for name in missing)]


def design_review_exists(sprint: str) -> bool:
    active = ROOT / ".planning" / "active" / f"sprint{sprint}_design_review.md"
    if active.exists():
        return True
    return any((ROOT / ".planning" / "archive").glob(f"v*/sprint{sprint}_design_review.md"))


def kickoff_exempts_g1(sprint: str) -> bool:
    candidates = [ROOT / ".planning" / "active" / f"sprint{sprint}_kickoff.md"]
    candidates.extend((ROOT / ".planning" / "archive").glob(f"v*/sprint{sprint}_kickoff.md"))
    for path in candidates:
        text = read_text(path)
        if re.search(r"G1\s+(skip|exempt)|design\s+review\s+(skip|exempt)|Phase 0 audit skipped", text, re.I):
            return True
    return False


def phase_commit_requires_codex(title: str) -> bool:
    """True for phase implementation commits that must carry Codex evidence."""
    if not re.match(r"^(feat|fix|docs|test|refactor)\(", title):
        return False
    return bool(re.search(r"Sprint\s+\d+\s+Phase\s+[A-Z]+[0-9]?\b", title))


def codex_review_errors(title: str, message: str, sprint: str | None, phase: str | None, staged: set[str]) -> list[str]:
    if not sprint or not phase or not phase_commit_requires_codex(title):
        return []

    review_rel = f".planning/active/sprint{sprint}_phase_{phase}_codex_review.md"
    review = ROOT / review_rel
    errors: list[str] = []
    if not review.exists():
        return [f"missing Codex review artifact: {review_rel}"]

    if not tracked(review_rel) and review_rel not in staged:
        errors.append(f"Codex review exists but is not staged: {review_rel}")
    if git(["diff", "--name-only", "--", review_rel]).strip():
        errors.append(f"Codex review has unstaged changes: {review_rel}")

    text = read_text(review)
    if not text.strip():
        errors.append(f"Codex review is empty: {review_rel}")
        return errors
    if re.search(r"(?mi)^\s*#\s*Codex Review|Auditeur.*Claude|agent independant", text):
        errors.append(f"Codex review looks rewritten by Claude; expected raw `codex exec -o`: {review_rel}")
    if not re.search(r"\b(CONFIRME|CONFIRM[EÉ]|GAP|PARTIEL|PARTIAL|CONFIRMED)\b", text, re.IGNORECASE):
        errors.append(f"Codex review has no per-deliverable verdict markers: {review_rel}")
    if not re.search(r"(?i)\b(Evidence|Fichier|File|ligne|line)\b|:[0-9]{1,5}\b", text):
        errors.append(f"Codex review has no file:line evidence markers: {review_rel}")

    has_partial = bool(re.search(r"Statut\s*:\s*PARTIEL|Partiels?\s*:\s*[1-9]", text, re.IGNORECASE))
    if has_partial and message:
        if re.search(r"0\s+PARTIEL", message, re.IGNORECASE):
            errors.append(f"commit body says 0 PARTIEL but Codex artifact contains PARTIEL: {review_rel}")
        elif not re.search(r"[1-9][0-9]*\s+PARTIEL|PARTIELS?", message, re.IGNORECASE):
            errors.append(f"commit body does not report Codex PARTIEL findings: {review_rel}")

    has_gap = bool(re.search(r"Statut\s*:\s*GAP|Gaps?\s*:\s*[1-9]", text, re.IGNORECASE))
    if has_gap and message:
        if re.search(r"0\s+GAP", message, re.IGNORECASE):
            errors.append(f"commit body says 0 GAP but Codex artifact contains GAP: {review_rel}")
        elif not re.search(r"[1-9][0-9]*\s+GAP|GAPS?", message, re.IGNORECASE):
            errors.append(f"commit body does not report Codex GAP findings: {review_rel}")
    return errors


def cmd_precommit_lightcheck(args: argparse.Namespace) -> int:
    files = staged_files()
    diff = staged_diff() if args.scope in {"all", "staged", "message"} else ""
    staged = set(files)
    message = parse_commit_message(args.message_file)
    title = commit_title(message)
    sprint, phase = phase_from_title(title)

    errors: list[str] = []
    warnings: list[str] = []
    if args.scope in {"all", "staged"}:
        errors.extend(staged_diff_check_errors())
        errors.extend(pub_mod_errors(diff, staged))
        errors.extend(loc_plan_errors(files, diff))
    if args.scope in {"all", "message"}:
        errors.extend(phase_title_errors(title, sprint, phase))
        errors.extend(commit_body_section_errors(title, message, sprint, phase))
        warnings.extend(file_reference_warnings(message))
        warnings.extend(wire_warnings(files, diff, sprint, phase))

    if (
        args.scope in {"all", "message"}
        and sprint
        and phase == "a"
        and not design_review_exists(sprint)
        and not kickoff_exempts_g1(sprint)
    ):
        errors.append(f"missing G1 design review for Sprint {sprint} Phase A")

    if args.scope in {"all", "message"}:
        errors.extend(codex_review_errors(title, message, sprint, phase, staged))

    for warning in warnings:
        print(f"[lightcheck] WARN: {warning}", file=sys.stderr)
    for error in errors:
        print(f"[lightcheck] BLOCK: {error}", file=sys.stderr)
    if errors:
        return 2
    return 0


def review_file(sprint: str, phase: str) -> Path | None:
    active = ROOT / ".planning" / "active" / f"sprint{sprint}_phase_{phase}_review.md"
    if active.exists():
        return active
    matches = sorted((ROOT / ".planning" / "archive").glob(f"v*/sprint{sprint}_phase_{phase}_review.md"))
    return matches[0] if matches else None


def review_gate_errors(text: str) -> list[str]:
    errors: list[str] = []
    if not FINAL_PASS_RE.search(text):
        errors.append("review verdict is not exactly `## Verdict : PASS` or `## Verdict: PASS`")
    verdict = next((line for line in text.splitlines() if line.startswith("## Verdict")), "")
    if "PASS-PENDING" in verdict:
        errors.append("PASS-PENDING is a transient pre-Codex state and cannot satisfy the commit gate")
    if re.search(r"Codex.*EN ATTENTE|EN ATTENTE.*Codex|Ready for Codex verification", text, re.IGNORECASE):
        errors.append("review still marks Codex verification as pending")
    return errors


def cmd_auditor_gate(args: argparse.Namespace) -> int:
    message = parse_commit_message(args.message_file)
    title = commit_title(message)
    if title.startswith("chore(planning):"):
        return 0
    sprint, phase = phase_from_title(title)
    if not sprint or not phase:
        return 0
    review = review_file(sprint, phase)
    if not review:
        print("[phase-auditor-gate] BLOCK: missing review file", file=sys.stderr)
        print(f"  expected: .planning/active/sprint{sprint}_phase_{phase}_review.md", file=sys.stderr)
        print(
            f"  run: python scripts/agent/agentctl.py prompt --kind phase-auditor --sprint {sprint} --phase {phase}",
            file=sys.stderr,
        )
        return 2
    text = read_text(review)
    review_errors = review_gate_errors(text)
    if review_errors:
        verdict = next((line for line in text.splitlines() if line.startswith("## Verdict")), "(unknown)")
        print("[phase-auditor-gate] BLOCK: review not PASS", file=sys.stderr)
        print(f"  file: {rel(review)}", file=sys.stderr)
        print(f"  verdict: {verdict}", file=sys.stderr)
        for error in review_errors:
            print(f"  reason: {error}", file=sys.stderr)
        return 2
    return 0


def cmd_install_hooks(_: argparse.Namespace) -> int:
    print("Enable portable hooks with:")
    print("  git config core.hooksPath .githooks")
    print()
    print("The command is intentionally not run automatically by agentctl.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Vendor-neutral agent process helper")
    sub = parser.add_subparsers(dest="command", required=True)

    context = sub.add_parser("context", help="Print repo process context")
    context.set_defaults(func=cmd_context)

    prompt = sub.add_parser("prompt", help="Print an assembled model prompt")
    prompt.add_argument("--kind", choices=sorted(PROMPT_KINDS), required=True)
    prompt.add_argument("--sprint", type=int)
    prompt.add_argument("--phase")
    prompt.add_argument(
        "--depth",
        choices=("standard", "deep"),
        default="standard",
        help="Include lightweight repo metadata; deep adds file name-status and recent commits",
    )
    prompt.set_defaults(func=cmd_prompt)

    codex_path = sub.add_parser("codex-prompt-path", help="Print stable .git Codex prompt path")
    codex_path.add_argument("--sprint", type=int, required=True)
    codex_path.add_argument("--phase", required=True)
    codex_path.add_argument("--recheck", type=int, help="Append _RECHECK_NN for targeted reruns")
    codex_path.add_argument("--absolute", action="store_true", help="Print an absolute filesystem path")
    codex_path.set_defaults(func=cmd_codex_prompt_path)

    vow = sub.add_parser("verify-on-write", help="Run scoped verification for one file")
    vow.add_argument("--file", required=True)
    vow.set_defaults(func=cmd_verify_on_write)

    light = sub.add_parser("precommit-lightcheck", help="Run staged diff light checks")
    light.add_argument("--message-file")
    light.add_argument(
        "--scope",
        choices=("all", "staged", "message"),
        default="all",
        help="Limit checks to staged diff mechanics, commit-message checks, or both",
    )
    light.set_defaults(func=cmd_precommit_lightcheck)

    gate = sub.add_parser("auditor-gate", help="Require PASS phase review for phase commits")
    gate.add_argument("--message-file", required=True)
    gate.set_defaults(func=cmd_auditor_gate)

    install = sub.add_parser("install-hooks", help="Print Git hook install command")
    install.set_defaults(func=cmd_install_hooks)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
