#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 Phase B — gate (1): "0 @radix-ui survives at runtime".
#
# Base UI (@base-ui/react) is the SOLE runtime primitive dependency
# (Day-0 D3). shadcn may be used build-time only (npx) but anything it
# emits must be re-authored onto Base UI before commit — no @radix-ui
# package may reach the runtime dependency tree. The greenfield front
# imports no Radix today; this gate is ANTI-DRIFT (catches a shadcn
# `add` that pulls Radix back via a third-party registry).
#
# Three deterministic layers:
#   (a) no `@radix-ui/*` key in package.json "dependencies"
#   (b) no `@radix-ui/*` import in src/
#   (c) no `@radix-ui` in the production (omit-dev) dependency tree
set -euo pipefail
cd "$(dirname "$0")/.."
fail=0

# (a) package.json runtime deps
if ! node -e 'const d=require("./package.json").dependencies||{};const r=Object.keys(d).filter(k=>k.startsWith("@radix-ui/"));if(r.length){console.error("  runtime @radix-ui dep(s):",r.join(", "));process.exit(1)}'; then
  echo "check-no-radix-runtime: @radix-ui in package.json dependencies"
  fail=1
fi

# (b) src references — match any quoted `@radix-ui/…` module specifier,
# regardless of the import form: `from '…'`, side-effect `import '…'`,
# dynamic `import('…')`, or `require('…')` (Codex P1: a from/import(-only
# grep let a bare side-effect `import '@radix-ui/x'` survive). In src/ a
# quoted `@radix-ui/` string is always an import specifier.
if grep -rnE "['\"]@radix-ui/" src 2>/dev/null; then
  echo "check-no-radix-runtime: @radix-ui reference in src/ — use @base-ui/react"
  fail=1
fi

# (c) production dependency tree (skips devDeps, where a build-time
# shadcn may legitimately pull Radix). Capture the tree to a variable
# FIRST: under `set -o pipefail`, `npm ls` exits non-zero on an
# out-of-sync tree, and `npm ls | grep -q` would inherit that non-zero
# and silently miss a real @radix-ui match (review P2-2).
if [ -d node_modules ]; then
  radix_tree=$(npm ls --omit=dev --all 2>/dev/null || true)
  if printf '%s\n' "$radix_tree" | grep -q "@radix-ui"; then
    echo "check-no-radix-runtime: @radix-ui present in the production dependency tree:"
    printf '%s\n' "$radix_tree" | grep "@radix-ui" || true
    fail=1
  fi
else
  echo "check-no-radix-runtime: node_modules absent — skipping production-tree layer (c)"
fi

if [ "$fail" -ne 0 ]; then
  echo "check-no-radix-runtime: FAILED"
  exit 1
fi
echo "check-no-radix-runtime: clean (Base UI is the sole runtime primitive dep)"
