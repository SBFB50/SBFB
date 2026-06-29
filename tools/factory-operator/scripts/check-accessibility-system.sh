#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
set -euo pipefail
cd "$(dirname "$0")/.."

fail() {
  echo "check-accessibility-system: $1"
  exit 1
}

test -f public/preboot.js || fail "missing public/preboot.js"
test -f public/accessibility.css || fail "missing public/accessibility.css"
grep -q 'src="/preboot.js"' index.html || fail "index.html must load the same-origin preboot before React"
grep -q 'href="/accessibility.css"' index.html || fail "index.html must load the same-origin accessibility CSS"
grep -q 'data-shortcuts' public/preboot.js || fail "preboot must apply shortcut preference"
test -f src/components/AccessibilityPanel.tsx || fail "missing AccessibilityPanel"

for token in \
  'data-theme="light"' \
  'data-contrast="high"' \
  'data-pointer="large"' \
  'data-text-spacing="loose"' \
  'data-font="legible"' \
  'data-motion="reduced"' \
  'data-scale="112"'
do
  grep -q "$token" public/accessibility.css || fail "missing CSS mode $token"
done

grep -q 'data-shortcuts' src/state/useFocalKeys.ts || fail "single-letter shortcuts must be preference-gated"
grep -q 'operator-body' public/accessibility.css || fail "operator shell must reflow instead of forcing desktop rails"
grep -q 'role="status"' src/components/verify/VerifyScene.tsx || fail "VERIFY state band must be a live status"
grep -q 'lineKindLabel' src/components/verify/plein/DiffViewer.tsx || fail "diff color/glyph markers need text labels"

echo "check-accessibility-system: clean"
