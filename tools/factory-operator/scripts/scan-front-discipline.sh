#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 Phase B — gate (5): the front never RENDERS a verdict.
# Sprint 80 Phase I — axis 2: the front never RENDERS a score/gauge either.
#
# "Connaissance consommée, jamais autoritaire" (kickoff invariant): the
# Operator restitutes a verdict computed elsewhere, it never asserts one.
# The slot ÉTAT is a named enumerated state machine that never says
# "PASS". Axis 1 forbids the literal words PASS / Vérifié / Approuvé
# as visible UI text in src/. It deliberately allows:
#   - comparisons against a backend verdict: `=== 'PASS'` / `=== "PASS"`
#   - JSDoc / line comments (developer-facing)
#
# Axis 2 (garde-fou §6 — anti-score): no fabricated health/trust/quality
# scoring vocabulary may reach the UI — neither in src/ code NOR in the
# i18n catalogs (.po msgstr, defense in depth: a translated catalog could
# smuggle a gauge the source never had). The anti-PASS/verdict axis on .po
# is already owned by check-i18n-verdict-cross-locale.sh — only the SCORE
# axis is new here.
#
# Anti-vacuous proof: a built-in self-test runs FIRST on every invocation —
# it plants violations (one per axis, code + catalog) in a temp dir and
# fails hard if the scan does not catch them. A regex regression can never
# turn this gate silently green.
#
# Twin of web/scripts/scan-en-strings.sh.
set -euo pipefail
cd "$(dirname "$0")/.."

# Catch the accented French verdict words AND their unaccented spellings
# (a sloppy `Verifie`/`Approuve` must not slip the gate — Codex P1-A).
# Capitalised-first only, so lowercase prose (« code vérifié manuellement »)
# is not a false positive; the rendered verdict badge is always capitalised.
FORBIDDEN='\b(PASS|Vérifié|Verifie|Approuvé|Approuve)\b'

# Axis 2 — scoring/gauge vocabulary, case-insensitive. Word-bounded so
# `jauger` (verb, prose) is out of reach but `jauge` is caught; `% santé`
# and `santé ... %` variants are both matched. English kept for defense in
# depth (a stray untranslated seam).
FORBIDDEN_SCORE='trust[- ]?score|score de (confiance|santé|sante|qualité|qualite)|(health|quality) score|\bjauge\b|%[[:space:]]*(de[[:space:]]+)?(santé|sante|confiance|qualité|qualite)|(santé|sante|confiance|qualité|qualite)[[:space:]]*(:)?[[:space:]]*[0-9]+[[:space:]]*%'

# --- Axis 1: verdict words as UI text in $1/src (tsx/ts, tests excluded) ---
scan_verdicts() {
  local base="$1"
  local matches
  matches=$(grep -rnE --include='*.tsx' --include='*.ts' \
    --exclude='*.test.ts' --exclude='*.test.tsx' \
    --exclude-dir=node_modules \
    --exclude-dir=bundle \
    --exclude-dir=dist \
    "$FORBIDDEN" "$base" 2>/dev/null || true)

  # Drop comment lines (`*` JSDoc, `//` line). Then STRIP the legitimate
  # backend comparison token `=== 'PASS'` / `=== "PASS"` from each line and
  # RE-CHECK for a forbidden word — never drop the whole line. Dropping the
  # comparison line wholesale let a line that BOTH compares AND renders
  # (e.g. `verdict === 'PASS' ? <span>PASS</span> : null`) slip the gate
  # (Codex P1). After stripping the comparison, a rendered verdict word
  # still triggers; a pure `=== 'PASS'` comparison leaves nothing behind.
  if [ -n "$matches" ]; then
    matches=$(printf '%s\n' "$matches" \
      | grep -vE ':\s*\*' \
      | grep -vE ':\s*//' \
      | sed -E "s/===[[:space:]]*['\"]PASS['\"]//g" \
      | grep -E "$FORBIDDEN" || true)
  fi
  printf '%s' "$matches"
}

# --- Axis 2: scoring vocabulary in $1 code (tsx/ts) AND catalogs (.po) ---
scan_scores() {
  local base="$1"
  {
    grep -rniE --include='*.tsx' --include='*.ts' \
      --exclude='*.test.ts' --exclude='*.test.tsx' \
      --exclude-dir=node_modules \
      --exclude-dir=bundle \
      --exclude-dir=dist \
      "$FORBIDDEN_SCORE" "$base" 2>/dev/null || true
    # Catalogs: only translated text (msgstr) can reach the UI; msgid and
    # `#:` source references are not rendered.
    grep -rniE --include='*.po' "^msgstr.*($FORBIDDEN_SCORE)" "$base" 2>/dev/null || true
  } | grep -vE ':\s*\*' | grep -vE ':\s*//' || true
}

# --- Anti-vacuous self-test (runs on EVERY invocation, fails hard) ---
# The tsx and po probes live in SEPARATE dirs so each surface is proven
# INDEPENDENTLY (review P2-2: a union result would stay green if the .po
# grep silently broke while the .tsx grep still fired).
self_test() {
  local dir_tsx dir_po
  dir_tsx=$(mktemp -d)
  dir_po=$(mktemp -d)
  cat >"$dir_tsx/Vacuous.tsx" <<'EOF'
export function Vacuous() {
  return <span>Vérifié — trust-score 87 %</span>
}
EOF
  cat >"$dir_po/vacuous.po" <<'EOF'
msgid "health"
msgstr "score de santé : 87 %"
EOF
  local v s_tsx s_po
  v=$(scan_verdicts "$dir_tsx")
  s_tsx=$(scan_scores "$dir_tsx")
  s_po=$(scan_scores "$dir_po")
  rm -rf "$dir_tsx" "$dir_po"
  if [ -z "$v" ] || [ -z "$s_tsx" ] || [ -z "$s_po" ]; then
    echo "scan-front-discipline: SELF-TEST FAILED — a planted violation was not caught (verdict/tsx: ${v:+ok}${v:-MISS}, score/tsx: ${s_tsx:+ok}${s_tsx:-MISS}, score/po: ${s_po:+ok}${s_po:-MISS})"
    exit 1
  fi
}

self_test

V_MATCHES=$(scan_verdicts src)
if [ -n "$V_MATCHES" ]; then
  echo "scan-front-discipline: forbidden verdict word as UI text in src/ (PASS / Vérifié / Approuvé)"
  echo "$V_MATCHES"
  exit 1
fi

S_MATCHES=$(scan_scores src)
if [ -n "$S_MATCHES" ]; then
  echo "scan-front-discipline: forbidden score/gauge vocabulary in src/ (trust-score / score de santé / jauge / % santé)"
  echo "$S_MATCHES"
  exit 1
fi

echo "scan-front-discipline: clean (the front renders no verdict and no score/gauge; self-test armed)"
