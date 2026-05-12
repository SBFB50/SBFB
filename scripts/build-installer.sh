#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build platform-native installer via cargo-packager.
#   Windows: NSIS .exe  |  Linux: .deb + .AppImage  |  macOS: .dmg
#
# Prerequisites:
#   cargo install cargo-packager --locked
#   Node.js + npm (for frontend build)
#   Windows: NSIS >= 3.08 (auto-downloaded by cargo-packager)
#   macOS:   png-to-icns crate (in workspace) for .icns generation
#
# Usage:
#   ./scripts/build-installer.sh            # auto-detect platform
#   ./scripts/build-installer.sh nsis       # force format
#   ./scripts/build-installer.sh deb,appimage
#
# Output: target/release/ (installer artifacts)

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

# Generate .icns for macOS if missing
if [[ "$(uname -s)" == Darwin* ]] && [[ ! -f assets/nexus-launcher.icns ]]; then
  echo "==> Generating .icns from PNG..."
  cargo run -p png-to-icns --release -- \
    assets/nexus-launcher.png assets/nexus-launcher.icns
fi

echo "==> Building frontend..."
npm --prefix web run build

FORMAT_ARG=""
if [[ -n "${1:-}" ]]; then
  FORMAT_ARG="--formats $1"
fi

echo ""
echo "==> Running cargo packager${FORMAT_ARG:+ ($FORMAT_ARG)}..."
# shellcheck disable=SC2086
cargo packager -c Packager.toml --verbose $FORMAT_ARG

echo ""
echo "==> Installer output:"
ls -lh target/release/*-setup.exe 2>/dev/null || true
ls -lh target/release/*.deb 2>/dev/null || true
ls -lh target/release/*.AppImage 2>/dev/null || true
ls -lh target/release/*.dmg 2>/dev/null || true

echo ""
echo "Done."
