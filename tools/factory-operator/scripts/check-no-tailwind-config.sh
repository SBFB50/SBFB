#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 80 Phase B — gate (2): no Tailwind v3 `tailwind.config.*`.
#
# Day-0 D6: Tailwind v4 is CSS-first — the theme lives in
# src/index.css (@theme / @custom-variant), 0 JS config. A
# `tailwind.config.{js,ts,cjs,mjs}` or an `@config` directive signals a
# v3 regression.
set -euo pipefail
cd "$(dirname "$0")/.."
fail=0

for f in tailwind.config.js tailwind.config.ts tailwind.config.cjs tailwind.config.mjs; do
  if [ -e "$f" ]; then
    echo "check-no-tailwind-config: forbidden $f (Tailwind v4 is CSS-first)"
    fail=1
  fi
done

if grep -rnE "@config\b" src 2>/dev/null; then
  echo "check-no-tailwind-config: forbidden @config directive in src/"
  fail=1
fi

if [ "$fail" -ne 0 ]; then
  echo "check-no-tailwind-config: FAILED"
  exit 1
fi
echo "check-no-tailwind-config: clean (Tailwind v4 CSS-first)"
