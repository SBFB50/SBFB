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
grep -q 'factory-operator.accessibility.v2' public/preboot.js || fail "preboot must read the v2 accessibility schema"
test -f src/components/AccessibilityPanel.tsx || fail "missing AccessibilityPanel"
test -f src/components/AdaptiveSurface.tsx || fail "missing AdaptiveSurface primitive"
grep -q 'NEED_GROUPS' src/components/AccessibilityPanel.tsx || fail "panel must expose stackable disability need groups"
grep -q 'resolveAccessibilityPreferences' src/preferences/accessibility.ts || fail "missing stackable accessibility resolver"
grep -q 'lowVision' src/preferences/accessibility.ts || fail "resolver must include low-vision need"
grep -q 'dyslexia' src/preferences/accessibility.ts || fail "resolver must include dyslexia need"
grep -q 'photosensitive' src/preferences/accessibility.ts || fail "resolver must include photosensitive need"

for token in \
  'data-theme="calm"' \
  'data-theme="paper"' \
  'data-theme="forced"' \
  'data-theme="light"' \
  'data-contrast="high"' \
  'data-color-vision="safe"' \
  'data-color-vision="monochrome"' \
  'data-pointer="large"' \
  'data-text-spacing="loose"' \
  'data-font="legible"' \
  'data-motion="reduced"' \
  'data-transparency="reduced"' \
  'data-density="focus"' \
  'data-reading="assist"' \
  'data-focus="strong"' \
  'data-scale="112"' \
  'data-scale="150"' \
  'data-assistive-tech="screen-reader"' \
  'data-captions="on"'
do
  grep -q "$token" public/accessibility.css || fail "missing CSS mode $token"
done

grep -q 'forced-colors: active' public/accessibility.css || fail "forced-colors mode must be covered"
grep -q 'adaptive-surface' public/accessibility.css || fail "surface-level accessibility CSS must be present"
grep -q 'data-shortcuts' src/state/useFocalKeys.ts || fail "single-letter shortcuts must be preference-gated"
grep -q 'operator-body' public/accessibility.css || fail "operator shell must reflow instead of forcing desktop rails"
grep -q 'role="status"' src/components/verify/VerifyScene.tsx || fail "VERIFY state band must be a live status"
grep -q 'lineKindLabel' src/components/verify/plein/DiffViewer.tsx || fail "diff color/glyph markers need text labels"
grep -q 'Stacking Rules' ../../docs/factory/OPERATOR_ACCESSIBILITY_DESIGN_SYSTEM.md || fail "design-system research doc must describe stacking rules"

for file in \
  src/components/steer/SteerScene.tsx \
  src/components/verify/VerifyScene.tsx \
  src/components/surfaces/SurfaceHost.tsx \
  src/components/surfaces/ProcedeSurface.tsx \
  src/components/surfaces/SessionsSurface.tsx \
  src/components/surfaces/KnowledgeSurface.tsx \
  src/components/surfaces/DocumentsSurface.tsx \
  src/components/verify/ContextPackInspector.tsx \
  src/components/verify/Terminal.tsx \
  src/components/verify/plein/DiffViewer.tsx
do
  grep -q 'AdaptiveSurface' "$file" || fail "$file must declare its page-level accessibility surface"
done

echo "check-accessibility-system: clean"
