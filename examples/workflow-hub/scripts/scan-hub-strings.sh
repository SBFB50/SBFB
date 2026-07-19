#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 83 Phase B — Workflow Hub i18n gate: no-literal discipline (scoped).
#
# The Hub is French-first (like the shell). Re-derived from
# web/scripts/scan-en-strings.sh — NOT imported. Scans the Hub's user-facing
# source (index.html + app.js) for English user-facing words that leak into the
# rendered UI. The list is deliberately NARROW so it never trips on the English
# axis identifiers the readout renders (theme, contrast, motion — technical
# tokens, not prose). When the React authoring surface lands, this becomes the
# Lingui no-literal macro gate on the Hub `src/`.
#
# Exit codes: 0 = clean; 1 = at least one English user-facing string.
set -uo pipefail
cd "$(dirname "$0")/.."

# NOT fail-open (Codex S83-B): the required user-facing sources MUST exist, and
# a grep error (exit >1) FAILS rather than silently passing via `|| true`.
for f in index.html app.js; do
  if [ ! -f "$f" ]; then
    echo "scan-hub-strings: FAILED (required Hub source missing: $f)"
    exit 1
  fi
done

EN_WORDS='\b(Welcome\b|Dashboard\b|Sign\s*in\b|Log\s*in\b|Sign\s*up\b|Please\b|Click\s*here\b|Coming\s*soon\b|Loading\.\.\.)'

FILES="index.html app.js"
if [ -n "$(ls -1 src/*.ts src/*.tsx 2>/dev/null || true)" ]; then
  FILES="$FILES $(ls -1 src/*.ts src/*.tsx 2>/dev/null)"
fi

MATCHES=$(grep -rnE "$EN_WORDS" $FILES 2>/dev/null)
rc=$?
if [ "$rc" -gt 1 ]; then
  echo "scan-hub-strings: FAILED (grep error rc=$rc)"
  exit 1
fi

# Drop comment lines (JSDoc `*`, line `//`, HTML `<!--`).
if [ -n "$MATCHES" ]; then
  MATCHES=$(printf '%s\n' "$MATCHES" | grep -vE ':\s*(\*|//|<!--)' || true)
fi

if [ -n "$MATCHES" ]; then
  echo "scan-hub-strings: English user-facing string(s) in the Hub (use the French catalog):"
  printf '%s\n' "$MATCHES"
  exit 1
fi

echo "scan-hub-strings: clean (no English user-facing strings in the Hub)"
