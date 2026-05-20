#!/usr/bin/env bash
#
# TeammateIdle hook: keeps the process supervisor engaged during dirty work.
# It does not force an infinite "always awake" loop once the repo is clean.

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
import subprocess
import sys
from pathlib import Path

try:
    payload = json.loads(os.environ.get("HOOK_INPUT", "{}"))
except json.JSONDecodeError:
    sys.exit(0)

name = str(payload.get("teammate_name") or "").lower()
if name != "supervisor":
    sys.exit(0)

repo = Path(os.environ["REPO_ROOT"])
status = subprocess.run(
    ["git", "status", "--short"],
    cwd=repo,
    text=True,
    stdout=subprocess.PIPE,
    stderr=subprocess.DEVNULL,
    check=False,
).stdout.strip()

if status:
    print(
        "[process-teammate-idle] supervisor must keep monitoring: worktree is "
        "dirty. Check the task list, gate order, and current artifacts before "
        "going idle. Current git status: " + status.replace("\n", "; "),
        file=sys.stderr,
    )
    sys.exit(2)

sys.exit(0)
PY
