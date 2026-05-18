#!/usr/bin/env bash
# Sprint 65 Phase C — scan for trust-related wording violations.
#
# Exit 1 if any forbidden pattern is found in user-facing files.
# Whitelisted: test files, archive/, SPRINT_LOG.md, planning docs,
# and legitimate verb/sentence uses of the scanned terms.
set -euo pipefail

cd "$(dirname "$0")/.."

VIOLATIONS=0

filter_noise() {
  grep -vE '(__tests__|\.test\.|\.spec\.)' \
  | grep -vE '(SPRINT_LOG|\.planning/|archive/)' \
  | grep -vE '^\s*//' \
  | grep -vE '^\s*\*' \
  || true
}

# 1. "Verifie/Verifiee" as a bare badge/label in UI+examples.
#    Allowed: "Signature verifiee", "source verifiable",
#    verb-in-sentence ("verifie que", "verifie l'identite", etc.),
#    qualified adjective ("deploiement verifie").
#    Forbidden: standalone "Verifie" as a badge label.
BARE_VERIFIE=$(grep -rnEi '\bverifi(e|ee|es|er)\b' \
  web/src/ examples/ \
  --include='*.tsx' --include='*.ts' --include='*.js' --include='*.html' \
  2>/dev/null \
  | filter_noise \
  | grep -viE '(Signature verifi|source verifi|verifi.+ live|verifi.+ hash)' \
  | grep -viE '(verifi.+ localement|verifi.+ maintenant|verifi.+ avec|verifi.+ par)' \
  | grep -viE '(verifie que |verifie l|verifie son|verifie le |verifie la )' \
  | grep -viE '(verifie un|verifie des|verifie si|verifie,|verifie\.)' \
  | grep -viE '(deploiement verifie|feed verifie|hash.chain)' \
  | grep -viE '(scan-trust-wording|Reverifier|reverifi)' \
  || true)
if [ -n "$BARE_VERIFIE" ]; then
  echo "VIOLATION [bare-verifie]:"
  echo "$BARE_VERIFIE"
  echo ""
  VIOLATIONS=$((VIOLATIONS + 1))
fi

# 2. "open source" describing apps (not SBFB codebase AGPL).
#    Allowed: near AGPL/OSI/licence/license/codebase/github.com refs.
OPEN_SOURCE=$(grep -rnEi '\bopen.source\b' \
  web/src/ examples/ \
  --include='*.tsx' --include='*.ts' --include='*.js' --include='*.html' \
  2>/dev/null \
  | filter_noise \
  | grep -viE '(AGPL|OSI|licence|license|codebase|github\.com)' \
  || true)
if [ -n "$OPEN_SOURCE" ]; then
  echo "VIOLATION [open-source-misuse]:"
  echo "$OPEN_SOURCE"
  echo ""
  VIOLATIONS=$((VIOLATIONS + 1))
fi

# 3. "de confiance" in automated/system context in UI.
#    Allowed: negation ("pas.*de confiance" = warning against trust).
DE_CONFIANCE=$(grep -rnE '\bde confiance\b' \
  web/src/ \
  --include='*.tsx' --include='*.ts' \
  2>/dev/null \
  | filter_noise \
  | grep -viE '(pas .* de confiance|ne .* de confiance)' \
  || true)
if [ -n "$DE_CONFIANCE" ]; then
  echo "VIOLATION [de-confiance]:"
  echo "$DE_CONFIANCE"
  echo ""
  VIOLATIONS=$((VIOLATIONS + 1))
fi

# 4. "Le code sur le reseau" — over-promise about code equivalence.
CODE_RESEAU=$(grep -rnEi 'Le code sur le (reseau|réseau)' \
  web/src/ examples/ \
  --include='*.tsx' --include='*.ts' --include='*.js' --include='*.html' \
  2>/dev/null \
  | filter_noise \
  || true)
if [ -n "$CODE_RESEAU" ]; then
  echo "VIOLATION [code-reseau-overpromise]:"
  echo "$CODE_RESEAU"
  echo ""
  VIOLATIONS=$((VIOLATIONS + 1))
fi

if [ "$VIOLATIONS" -gt 0 ]; then
  echo "scan-trust-wording: $VIOLATIONS violation(s) found"
  exit 1
fi

echo "scan-trust-wording: clean"
exit 0
