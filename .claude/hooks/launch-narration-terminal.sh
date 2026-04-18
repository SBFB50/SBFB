#!/usr/bin/env bash
# .claude/hooks/launch-narration-terminal.sh
#
# SessionStart hook : spawn an external terminal window that runs
# narration-viewer.js so the full Haiku feed is always visible next to
# the Claude Code TUI. Uses a heartbeat file (.claude/.narration-
# terminal.heartbeat, refreshed every second by the viewer) to skip
# re-spawning when a viewer is already running.

set -u

CWD="${CLAUDE_PROJECT_DIR:-$(pwd)}"
HEARTBEAT="$CWD/.claude/.narration-terminal.heartbeat"
VIEWER="$CWD/.claude/hooks/narration-viewer.js"

[ -f "$VIEWER" ] || exit 0

# Skip spawn when a viewer is alive (heartbeat written < 300s ago).
# Window bumped from 30s to 300s after observing that short windows
# let SessionStart (via /clear or resume) re-spawn duplicates when
# the viewer had a transient busy period. The viewer itself refreshes
# every 1s so 300s tolerates 5 min of stuckness before assuming dead.
if [ -f "$HEARTBEAT" ]; then
  now=$(date +%s)
  if mtime=$(stat -c %Y "$HEARTBEAT" 2>/dev/null) || mtime=$(stat -f %m "$HEARTBEAT" 2>/dev/null); then
    age=$((now - mtime))
    if [ "$age" -lt 300 ]; then
      exit 0
    fi
  fi
fi

# Convert POSIX path -> Windows path if running under Git Bash / MSYS.
win_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s' "$1"
  fi
}

VIEWER_WIN=$(win_path "$VIEWER")

# Prefer Windows Terminal when available (tabbed, keeps colors). Fall
# back to a classic cmd.exe window. On non-Windows hosts, just print an
# instruction so the user can open the viewer manually.
if command -v wt.exe >/dev/null 2>&1; then
  (wt.exe -w 0 nt --title "Nexus Narration" node "$VIEWER_WIN" >/dev/null 2>&1 &) >/dev/null 2>&1
elif command -v cmd.exe >/dev/null 2>&1; then
  (cmd.exe //c start "Nexus Narration" cmd.exe //k node "$VIEWER_WIN" >/dev/null 2>&1 &) >/dev/null 2>&1
else
  printf 'narration viewer: run `node "%s"` in a separate terminal\n' "$VIEWER" >&2
fi

exit 0
