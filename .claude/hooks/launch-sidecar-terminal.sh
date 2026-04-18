#!/usr/bin/env bash
# .claude/hooks/launch-sidecar-terminal.sh
#
# SessionStart hook — spawn an external terminal running sidecar-input.js
# so the user can type questions asynchronously while Claude works. Each
# line typed in the sidecar is appended to .claude/.sidecar-queue.jsonl
# and gets injected as a block-decision at Claude's next Stop event (see
# sidecar-drain-on-stop.js).
#
# Mirrors the launch-narration-terminal.sh pattern : heartbeat file skip
# when a live sidecar is already running.

set -u

CWD="${CLAUDE_PROJECT_DIR:-$(pwd)}"
HEARTBEAT="$CWD/.claude/.sidecar-terminal.heartbeat"
VIEWER="$CWD/.claude/hooks/sidecar-input.js"

[ -f "$VIEWER" ] || exit 0

# Skip spawn when a sidecar is alive (heartbeat < 300s old). See
# launch-narration-terminal.sh for rationale on the 30s->300s bump.
# Sidecar writes heartbeat every 5s so 300s tolerates 1 min of
# stuckness before assuming dead.
if [ -f "$HEARTBEAT" ]; then
  now=$(date +%s)
  if mtime=$(stat -c %Y "$HEARTBEAT" 2>/dev/null) || mtime=$(stat -f %m "$HEARTBEAT" 2>/dev/null); then
    age=$((now - mtime))
    if [ "$age" -lt 300 ]; then
      exit 0
    fi
  fi
fi

win_path() {
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$1"
  else
    printf '%s' "$1"
  fi
}

VIEWER_WIN=$(win_path "$VIEWER")

if command -v wt.exe >/dev/null 2>&1; then
  (wt.exe -w 0 nt --title "Nexus Sidecar" node "$VIEWER_WIN" >/dev/null 2>&1 &) >/dev/null 2>&1
elif command -v cmd.exe >/dev/null 2>&1; then
  (cmd.exe //c start "Nexus Sidecar" cmd.exe //k node "$VIEWER_WIN" >/dev/null 2>&1 &) >/dev/null 2>&1
else
  printf 'sidecar input: run `node "%s"` in a separate terminal\n' "$VIEWER" >&2
fi

exit 0
