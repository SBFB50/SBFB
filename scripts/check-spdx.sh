#!/usr/bin/env bash
#
# Sprint 10 Phase A (D6) — verify SPDX header presence on all source files.
#
# Checks that every .rs, .py, .ts, .tsx file in the project contains the
# SPDX-License-Identifier line within the first 5 lines. Exits non-zero
# if any file is missing the header.
#
# Usage:
#   ./scripts/check-spdx.sh           # check all files, exit 1 if missing
#   ./scripts/check-spdx.sh --count   # print count of compliant files

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SPDX_TAG="SPDX-License-Identifier: AGPL-3.0-or-later"

missing=()
compliant=0

while IFS= read -r file; do
  # Read first 5 lines and check for SPDX tag
  if head -n 5 "$file" | tr -d '\r' | grep -qF "$SPDX_TAG"; then
    compliant=$((compliant + 1))
  else
    missing+=("$file")
  fi
done < <(
  find "$REPO_ROOT/crates" -name '*.rs' -not -path '*/target/*'
  if [ -d "$REPO_ROOT/packages" ]; then
    find "$REPO_ROOT/packages" -name '*.py' -not -path '*/__pycache__/*' -not -path '*/.venv/*'
  fi
  find "$REPO_ROOT/web/src" \( -name '*.ts' -o -name '*.tsx' \) -not -path '*/node_modules/*'
)

if [[ "${1:-}" == "--count" ]]; then
  echo "$compliant"
  exit 0
fi

if [[ ${#missing[@]} -gt 0 ]]; then
  echo "SPDX header missing in ${#missing[@]} file(s):"
  for f in "${missing[@]}"; do
    echo "  $f"
  done
  exit 1
fi

echo "SPDX check passed: $compliant files compliant"
