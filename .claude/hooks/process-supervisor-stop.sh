#!/usr/bin/env bash
#
# .claude/hooks/process-supervisor-stop.sh
#
# Stop hook: catches obvious process misses when Claude tries to end a turn.
# This is not a replacement for the long-lived process-supervisor teammate.
# It is the automatic backstop that still fires even if the model forgets.

set -eo pipefail

INPUT=$(cat)
REPO_ROOT="${CLAUDE_PROJECT_DIR:-$(pwd)}"

if [ ! -f "$REPO_ROOT/Cargo.toml" ] || [ ! -d "$REPO_ROOT/.planning" ]; then
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
import subprocess
import sys
from pathlib import Path

try:
    payload = json.loads(os.environ.get("HOOK_INPUT", "{}"))
except json.JSONDecodeError:
    sys.exit(0)

repo = Path(os.environ["REPO_ROOT"])
message = str(payload.get("last_assistant_message") or "")
if not message.strip():
    sys.exit(0)


def git(args):
    return subprocess.run(
        ["git", *args],
        cwd=repo,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        check=False,
    ).stdout.strip()


status = git(["status", "--short"])
lower = message.lower()

finality = re.search(
    r"\b(done|completed|fixed|committed|clean|ready|final|finished)\b"
    r"|fait|corrig|termine|committ|propre|pret|livr",
    lower,
)

if status and finality:
    reason = (
        "[process-supervisor] Stop blocked: the last answer sounds like a "
        "completion report, but git status is not clean. Report the dirty "
        "state explicitly, or finish/stage/commit the intended process work "
        "before ending the turn. Current git status: "
        + status.replace("\n", "; ")
    )
    print(json.dumps({"decision": "block", "reason": reason}))
    sys.exit(0)

active = repo / ".planning" / "active"
phase_match = re.search(r"phase\s*([a-g])", lower)
sprint_match = re.search(r"sprint\s*(\d+)", lower)
if (
    phase_match
    and re.search(r"code|coder|implement|create|validate|crate", lower)
    and "preflight" not in lower
):
    sn = sprint_match.group(1) if sprint_match else ""
    pl = phase_match.group(1)
    if sn:
        expected = active / f"sprint{sn}_phase_{pl}_preflight.md"
    else:
        expected = None
    has_any_preflight = bool(list(active.glob("sprint*_phase_*_preflight.md")))
    if (expected and not expected.exists()) or (not expected and not has_any_preflight):
        label = f"sprint{sn}_phase_{pl}" if sn else f"phase_{pl}"
        reason = (
            f"[process-supervisor] Stop blocked: {label} implementation "
            f"language appeared, but no matching preflight.md exists in "
            f".planning/active/. Run/record G8 preflight first, or clarify "
            f"that no code will start yet."
        )
        print(json.dumps({"decision": "block", "reason": reason}))
        sys.exit(0)

sys.exit(0)
PY
