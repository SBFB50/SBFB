#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Local smoke-test for the supply-chain CI gate.
#
# Runs the same audits as `.github/workflows/supply-chain.yml`
# but on the developer machine. Used:
#
#   * before opening a PR (catches CVE/license issues without
#     waiting for the GitHub Action)
#   * by `gsd:execute` style verification loops to confirm Phase A
#     stays green after later phases land code
#
# Exit codes:
#   0  — all audits passed
#   1  — one or more audits failed (output above tells which)
#
# Usage:
#   bash scripts/ci-smoke/supply-chain-green.sh

set -euo pipefail

# Resolve repo root regardless of where the script is invoked from.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$REPO_ROOT"

red()   { printf '\033[31m%s\033[0m\n' "$*"; }
green() { printf '\033[32m%s\033[0m\n' "$*"; }
bold()  { printf '\033[1m%s\033[0m\n' "$*"; }

bold "[1/2] cargo-deny check"
if ! command -v cargo-deny >/dev/null 2>&1; then
    red  "cargo-deny is not installed."
    echo "Install with: cargo install cargo-deny --locked"
    exit 1
fi
cargo deny check
green "cargo-deny: ok"
echo

bold "[2/2] audit-ci (npm web)"
npm --prefix web run audit:ci
green "audit-ci: ok"
echo

bold "supply-chain smoke-test: ALL GREEN"
