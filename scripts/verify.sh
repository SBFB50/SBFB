#!/usr/bin/env bash
#
# Sprint 9 Phase A (D5) — full fail-fast verification suite.
#
# One-shot script that reproduces the ordered checks we run before
# every phase commit. Matches the order of `docs/claude/README.md`
# §4.3 and the `sprint{N}_plan.md` fail-fast checklist. Any red
# step aborts immediately (`set -e`) — the exit code + last line
# tell you exactly which check failed.
#
# Usage:
#   ./scripts/verify.sh           # full run (Rust + Python + web + Playwright)
#   ./scripts/verify.sh --quick   # skip Playwright for fast feedback during a phase
#
# The script is meant to be runnable on a fresh checkout after
# `./scripts/setup.sh` — it assumes `.venv/` exists and `nexus_core`
# is installed in editable mode.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

QUICK=0
if [[ "${1:-}" == "--quick" ]]; then
  QUICK=1
fi

step() {
  echo ""
  echo "==> [$1] $2"
}

step 1 "cargo fmt --all --check"
cargo fmt --all --check

step 2 "cargo clippy --workspace --all-targets --locked -- -D warnings"
cargo clippy --workspace --all-targets --locked -- -D warnings

step 3 "cargo test --workspace --locked"
cargo test --workspace --locked

step 4 "uv run ruff format --check packages/ examples/"
uv run ruff format --check packages/ examples/

step 5 "uv run ruff check packages/ examples/"
uv run ruff check packages/ examples/

step 6 "uv run pytest packages/nexus-sdk/tests/ -q"
uv run pytest packages/nexus-sdk/tests/ -q

step 7 "uv run pytest packages/nexus-coordinator/tests/ -q"
uv run pytest packages/nexus-coordinator/tests/ -q

step 8 "uv run pytest packages/nexus-app-gov/tests/ -q"
uv run pytest packages/nexus-app-gov/tests/ -q

cd "$REPO_ROOT/web"

step 9 "tsc --noEmit -p tsconfig.app.json"
npx tsc --noEmit -p tsconfig.app.json

step 10 "npm run lint"
npm run lint

step 11 "npm run test:unit"
npm run test:unit

step 12 "npm run test:coverage"
npm run test:coverage

step 13 "npm run build"
npm run build

step 14 "npm run size"
npm run size

if [[ "$QUICK" -eq 0 ]]; then
  step 15 "npx playwright test"
  npx playwright test
else
  echo ""
  echo "==> [15] SKIPPED Playwright (--quick mode)"
fi

step 16 "bash scripts/scan-en-strings.sh"
bash scripts/scan-en-strings.sh

step 17 "npm audit --audit-level=high"
npm audit --audit-level=high

cd "$REPO_ROOT"

step 18 "bash scripts/check-spdx.sh"
bash scripts/check-spdx.sh

echo ""
echo "==> verify.sh passed all steps"
