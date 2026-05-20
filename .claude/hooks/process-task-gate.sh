#!/usr/bin/env bash
#
# TaskCreated / TaskCompleted hook for the Claude Code Agent Team task list.
# Task creation must allow a full future plan before artifacts exist. Blocking
# is reserved for completing gate/implementation tasks without the artifacts
# that prove the process actually happened.

set -eo pipefail

INPUT=$(cat)
REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"

if [ ! -f "$REPO_ROOT/Cargo.toml" ] || [ ! -d "$REPO_ROOT/.planning/active" ]; then
  exit 0
fi

PYTHON_BIN=""
if command -v python3 >/dev/null 2>&1; then
  PYTHON_BIN="python3"
elif command -v python >/dev/null 2>&1; then
  PYTHON_BIN="python"
fi

if [ -z "$PYTHON_BIN" ]; then
  exit 0
fi

HOOK_INPUT="$INPUT" REPO_ROOT="$REPO_ROOT" "$PYTHON_BIN" - <<'PY'
import json
import os
import re
import sys
from pathlib import Path

try:
    payload = json.loads(os.environ.get("HOOK_INPUT", "{}"))
except json.JSONDecodeError:
    sys.exit(0)

repo = Path(os.environ["REPO_ROOT"])
active = repo / ".planning" / "active"
event = str(payload.get("hook_event_name") or "")
subject = str(payload.get("task_subject") or "")
description = str(payload.get("task_description") or "")
text = f"{subject}\n{description}".lower()


def block(message):
    print(f"[process-task-gate] {message}", file=sys.stderr)
    sys.exit(2)


def files(pattern):
    return list(active.glob(pattern))


def has_review_pass():
    for path in files("sprint*_phase_*_review.md"):
        try:
            content = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        if re.search(r"^##\s*Verdict\s*:?\s*PASS\s*$", content, re.MULTILINE):
            return True
    return False


if event == "TaskCreated":
    # A sequential plan is allowed to contain future implementation/review tasks
    # before their artifacts exist. TaskCompleted below enforces the gates.
    sys.exit(0)

if event != "TaskCompleted":
    sys.exit(0)

if "g-preflight" in text or "preflight" in text:
    if "sprint 67" in text and "phase c" in text:
        if not (active / "sprint67_phase_c_preflight.md").exists():
            block("G-PREFLIGHT cannot complete: sprint67_phase_c_preflight.md is missing.")
    elif not files("sprint*_phase_*_preflight.md"):
        block("G-PREFLIGHT cannot complete: no sprint phase preflight artifact exists.")

if "g-review" in text or "review-deep" in text:
    if not files("sprint*_phase_*_review.md"):
        block("G-REVIEW cannot complete: no sprint phase review artifact exists.")

if (
    re.search(r"phase\s*c|sbfb-factory|factory", text)
    and re.search(r"code|coder|implement|create|crate", text)
    and not (active / "sprint67_phase_c_preflight.md").exists()
):
    block(
        "Sprint 67 Phase C/factory implementation task cannot complete before "
        "sprint67_phase_c_preflight.md exists."
    )

if "g-codex" in text or "codex" in text:
    if not files("sprint*_phase_*_codex_review.md"):
        block("G-CODEX cannot complete: no codex_review artifact exists.")
    if not has_review_pass():
        block("G-CODEX cannot complete: no final review verdict PASS found.")

if "g-commit" in text:
    for path in files("sprint*_phase_*_review.md"):
        try:
            content = path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        if "PASS-PENDING" in content:
            block(f"G-COMMIT cannot complete: {path.name} still contains PASS-PENDING.")

sys.exit(0)
PY
