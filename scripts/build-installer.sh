#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build the Windows NSIS installer via cargo-packager.
#
# Prerequisites:
#   cargo install cargo-packager --locked
#   NSIS >= 3.08 on PATH (or let cargo-packager download it)
#   Node.js + npm (for frontend build)
#
# Usage:
#   ./scripts/build-installer.sh
#
# Output: target/release/nexus-launcher_<version>_x64-setup.exe

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

command -v cargo >/dev/null 2>&1 || { echo "ERROR: cargo not found"; exit 1; }
cargo packager --version >/dev/null 2>&1 || {
  echo "ERROR: cargo-packager not installed."
  echo "  Run: cargo install cargo-packager --locked"
  exit 1
}

echo "==> Building frontend..."
npm --prefix web run build

echo ""
echo "==> Running cargo packager (builds binaries + NSIS installer)..."
cargo packager -c Packager.toml --verbose

echo ""
echo "==> Installer output:"
ls -lh target/release/*-setup.exe 2>/dev/null || echo "  (no setup .exe found — check cargo-packager output above)"

echo ""
echo "Done."
