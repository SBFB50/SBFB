#!/usr/bin/env bash
# .claude/hooks/cleanup-orphan-hooks.sh
#
# SessionStart hook wrapper — calls the PowerShell cleanup on Windows
# so orphan node.exe hook processes from previous sessions are killed
# before we spawn new hook calls.
#
# Rationale : narrate-action.js (pre 2026-04-18 fix) + nexus-statusline
# .js could leak zombie node.exe processes because they never called
# process.exit(0) explicitly — the event loop kept them alive waiting
# on HTTP keep-alive sockets or buffered stdout. Between Sprint 20
# Phase D and Phase E a single session accumulated 11 zombies (~600MB
# RAM). Fix A in same commit adds process.exit(0) to prevent future
# leaks ; this script cleans up survivors from pre-fix sessions and
# acts as insurance against future regressions.
#
# Non-Windows hosts : no-op (viewers there are spawned via terminal
# applications that manage lifecycle themselves).

set -u

CWD="${CLAUDE_PROJECT_DIR:-$(pwd)}"
PS1_SCRIPT="$CWD/.claude/hooks/cleanup-orphan-hooks.ps1"

[ -f "$PS1_SCRIPT" ] || exit 0

# Convert POSIX path -> Windows path if running under Git Bash / MSYS.
win_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s' "$1"
  fi
}

if command -v powershell.exe >/dev/null 2>&1; then
  PS1_WIN=$(win_path "$PS1_SCRIPT")
  # Run non-blocking so SessionStart is not delayed by the scan.
  # -NoProfile keeps invocation fast, -ExecutionPolicy Bypass avoids
  # user profile restriction on unsigned local scripts.
  (powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$PS1_WIN" >/dev/null 2>&1 &) >/dev/null 2>&1
fi

exit 0
