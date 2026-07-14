#!/usr/bin/env bash
#
# Sprint 9 Phase A (D5) — full fail-fast verification suite.
# Python-era steps removed at Sprint 82 Phase E (project is Rust +
# Frontend pure since S50-S51; see docs/DEPRECATED.md).
#
# One-shot script that reproduces the ordered checks we run before
# every phase commit. Matches the order of `docs/claude/README.md`
# §4.3 and the `sprint{N}_plan.md` fail-fast checklist. Any red
# step aborts immediately (`set -e`) — the exit code + last line
# tell you exactly which check failed.
#
# Usage:
#   ./scripts/verify.sh           # full run (Rust + web + Playwright)
#   ./scripts/verify.sh --quick   # skip Playwright for fast feedback during a phase
#
# The script is meant to be runnable on a fresh checkout: Rust
# toolchain (rustup) + `cd web && npm install`, plus a one-time
# `npx playwright install chromium` for the full (non --quick) run.

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

cd "$REPO_ROOT/web"

step 4 "tsc --noEmit -p tsconfig.app.json"
npx tsc --noEmit -p tsconfig.app.json

step 5 "npm run lint"
npm run lint

step 6 "npm run test:unit"
npm run test:unit

step 7 "npm run test:coverage"
npm run test:coverage

step 8 "npm run build"
npm run build

step 9 "npm run size"
npm run size

if [[ "$QUICK" -eq 0 ]]; then
  # Real hermetic Playwright E2E (process-evolution Commit 2): spawns a
  # real daemon serving the dist built at step 8 via --web-root and
  # drives the actual shell in chromium. The @compute flagship is
  # env-gated and excluded here (needs Ollama + a deployed app).
  # Build the daemon explicitly and PIN SBFB_DAEMON_BIN so global-setup
  # never silently picks up a stale target/release binary. `playwright
  # install chromium` must have been run once on the machine.
  step 10 "build daemon + npm run test:e2e (hermetic Playwright)"
  ( cd "$REPO_ROOT" && cargo build -p nexus-shell-daemon )
  E2E_DAEMON_BIN="$REPO_ROOT/target/debug/nexus-shell-daemon"
  [ -f "$E2E_DAEMON_BIN.exe" ] && E2E_DAEMON_BIN="$E2E_DAEMON_BIN.exe"
  SBFB_DAEMON_BIN="$E2E_DAEMON_BIN" npm run test:e2e
else
  echo ""
  echo "==> [10] SKIPPED Playwright (--quick mode)"
fi

step 11 "bash scripts/scan-en-strings.sh"
bash scripts/scan-en-strings.sh

step 12 "npm audit --audit-level=high"
npm audit --audit-level=high

cd "$REPO_ROOT"

step 13 "bash scripts/check-spdx.sh"
bash scripts/check-spdx.sh

step 14 "bash scripts/check-sharding-docs.sh"
bash scripts/check-sharding-docs.sh

step 15 "bash scripts/check-frontier-contracts.sh"
bash scripts/check-frontier-contracts.sh

step 16 "bash scripts/check-factory-docs.sh"
bash scripts/check-factory-docs.sh

echo ""
echo "==> verify.sh passed all steps"
