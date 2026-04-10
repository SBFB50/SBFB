#!/usr/bin/env bash
# Sprint 5 Phase D — scan for English user-facing strings in the shell.
#
# The shell is French-only per CLAUDE.md. This script greps
# web/src/ for strings that are visible to the user and happen
# to be in English. It deliberately skips:
#   - JSDoc / line comments (they're developer-facing)
#   - HTTP method names (DELETE, POST, etc. — protocol)
#   - Route paths (/browse, /curators — URL identifiers)
#   - Component / import names (Browse, Curators — JS identifiers)
#   - Type literals ("submit" as a button type attribute)
#   - The `ui/` shadcn primitives (vendor-copied)
#
# Exit codes:
#   0  — zero matches; the UI is clean
#   1  — at least one match; CI should fail

set -euo pipefail

cd "$(dirname "$0")/.."

# Words that should never appear as visible user text. The list
# is deliberately narrow — a longer list produces false positives
# on identifiers. Add only words that have no role in code or
# protocol.
EN_WORDS='\b(Welcome\b|Dashboard\b|Sign\s*in\b|Log\s*in\b|Sign\s*up\b|Please\b|Click\s*here\b|Coming\s*soon\b|Loading\.\.\.)'

MATCHES=$(grep -rnE --include='*.tsx' --include='*.ts' \
  --exclude-dir=node_modules \
  --exclude-dir=dist \
  --exclude-dir=test-results \
  --exclude-dir=tests \
  --exclude-dir=scripts \
  --exclude-dir=ui \
  "$EN_WORDS" src 2>/dev/null || true)

# Filter out comment lines. JSDoc blocks start with `*`, line
# comments start with `//`. Both patterns are after optional
# leading whitespace.
if [ -n "$MATCHES" ]; then
  MATCHES=$(echo "$MATCHES" | grep -vE ':\s*\*' | grep -vE ':\s*//' || true)
fi

if [ -n "$MATCHES" ]; then
  echo "scan-en-strings: found English UI strings in src/ — please translate"
  echo "$MATCHES"
  exit 1
fi

echo "scan-en-strings: src/ is French-only, clean"
exit 0
