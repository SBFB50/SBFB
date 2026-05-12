#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build release binaries and frontend for distribution.
#
# Usage:
#   ./scripts/build-release.sh
#
# Output: dist/ directory with binaries + web/.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"

echo "==> Building frontend..."
npm --prefix web run build

echo ""
echo "==> Building Rust release binaries..."
cargo build --release -p nexus-worker -p nexus-shell-daemon -p nexus-launcher

# Detect platform and copy binaries
case "$(uname -s)" in
  Linux*)
    cp target/release/nexus-worker "$DIST/nexus-worker-linux-x86_64"
    cp target/release/nexus-shell-daemon "$DIST/nexus-shell-daemon-linux-x86_64"
    cp target/release/nexus-launcher "$DIST/nexus-launcher-linux-x86_64"
    chmod +x "$DIST"/nexus-*-linux-*
    echo "  Linux binaries copied to dist/"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    cp target/release/nexus-worker.exe "$DIST/nexus-worker-windows-x86_64.exe"
    cp target/release/nexus-shell-daemon.exe "$DIST/nexus-shell-daemon-windows-x86_64.exe"
    cp target/release/nexus-launcher.exe "$DIST/nexus-launcher-windows-x86_64.exe"
    echo "  Windows binaries copied to dist/"
    ;;
  Darwin*)
    cp target/release/nexus-worker "$DIST/nexus-worker-macos-x86_64"
    cp target/release/nexus-shell-daemon "$DIST/nexus-shell-daemon-macos-x86_64"
    cp target/release/nexus-launcher "$DIST/nexus-launcher-macos-x86_64"
    chmod +x "$DIST"/nexus-*-macos-*
    echo "  macOS binaries copied to dist/"
    ;;
esac

echo ""
echo "==> Copying frontend build..."
rm -rf "$DIST/web"
cp -r web/dist "$DIST/web"

echo ""
echo "==> Release artifacts in dist/:"
ls -lhR "$DIST/"
