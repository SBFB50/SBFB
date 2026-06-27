#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 Phase B — gate (5): the front never RENDERS a verdict.
#
# "Connaissance consommée, jamais autoritaire" (kickoff invariant): the
# Operator restitutes a verdict computed elsewhere, it never asserts one.
# The slot ÉTAT is a named enumerated state machine that never says
# "PASS". This scan forbids the literal words PASS / Vérifié / Approuvé
# as visible UI text in src/. It deliberately allows:
#   - comparisons against a backend verdict: `=== 'PASS'` / `=== "PASS"`
#   - JSDoc / line comments (developer-facing)
#
# Twin of web/scripts/scan-en-strings.sh. Extended (anti-score / gauge /
# trust-score) in Phase I.
set -euo pipefail
cd "$(dirname "$0")/.."

# Catch the accented French verdict words AND their unaccented spellings
# (a sloppy `Verifie`/`Approuve` must not slip the gate — Codex P1-A).
# Capitalised-first only, so lowercase prose (« code vérifié manuellement »)
# is not a false positive; the rendered verdict badge is always capitalised.
FORBIDDEN='\b(PASS|Vérifié|Verifie|Approuvé|Approuve)\b'

# Sprint 80 Phase D: this gate guards SHIPPED UI text. Unit tests legitimately
# reference a RESTITUTED verdict (`reviewTone('PASS')`, a fixture
# `review_verdict: 'PASS'`, a `not.toMatch(/PASS/)` guard) — they render no
# user-facing UI, so `*.test.{ts,tsx}` are excluded. The arbre de procédé that
# RESTITUTES a recorded verdict reads it from a variable (no literal in source),
# so production components stay covered.
MATCHES=$(grep -rnE --include='*.tsx' --include='*.ts' \
  --exclude='*.test.ts' --exclude='*.test.tsx' \
  --exclude-dir=node_modules \
  --exclude-dir=bundle \
  --exclude-dir=dist \
  "$FORBIDDEN" src 2>/dev/null || true)

# Drop comment lines (`*` JSDoc, `//` line). Then STRIP the legitimate
# backend comparison token `=== 'PASS'` / `=== "PASS"` from each line and
# RE-CHECK for a forbidden word — never drop the whole line. Dropping the
# comparison line wholesale let a line that BOTH compares AND renders
# (e.g. `verdict === 'PASS' ? <span>PASS</span> : null`) slip the gate
# (Codex P1). After stripping the comparison, a rendered verdict word
# still triggers; a pure `=== 'PASS'` comparison leaves nothing behind.
if [ -n "$MATCHES" ]; then
  MATCHES=$(printf '%s\n' "$MATCHES" \
    | grep -vE ':\s*\*' \
    | grep -vE ':\s*//' \
    | sed -E "s/===[[:space:]]*['\"]PASS['\"]//g" \
    | grep -E "$FORBIDDEN" || true)
fi

if [ -n "$MATCHES" ]; then
  echo "scan-front-discipline: forbidden verdict word as UI text in src/ (PASS / Vérifié / Approuvé)"
  echo "$MATCHES"
  exit 1
fi
echo "scan-front-discipline: clean (the front renders no verdict)"
