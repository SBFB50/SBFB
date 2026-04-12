#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 10 Phase D — build release binaries and Python wheels.
#
# Usage:
#   ./scripts/build-release.sh              # build for current platform
#   ./scripts/build-release.sh --all        # build all platforms (CI)
#
# Output: dist/ directory with binaries and wheels.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DIST="$REPO_ROOT/dist"
mkdir -p "$DIST"

ALL=0
if [[ "${1:-}" == "--all" ]]; then
  ALL=1
fi

echo "==> Building Rust release binaries..."
cargo build --release -p nexus-worker -p nexus-shell-daemon

# Detect platform and copy binaries
case "$(uname -s)" in
  Linux*)
    cp target/release/nexus-worker "$DIST/nexus-worker-linux-x86_64"
    cp target/release/nexus-shell-daemon "$DIST/nexus-shell-daemon-linux-x86_64"
    chmod +x "$DIST"/nexus-*-linux-*
    echo "  Linux binaries copied to dist/"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    cp target/release/nexus-worker.exe "$DIST/nexus-worker-windows-x86_64.exe"
    cp target/release/nexus-shell-daemon.exe "$DIST/nexus-shell-daemon-windows-x86_64.exe"
    echo "  Windows binaries copied to dist/"
    ;;
  Darwin*)
    cp target/release/nexus-worker "$DIST/nexus-worker-macos-x86_64"
    cp target/release/nexus-shell-daemon "$DIST/nexus-shell-daemon-macos-x86_64"
    chmod +x "$DIST"/nexus-*-macos-*
    echo "  macOS binaries copied to dist/"
    ;;
esac

echo ""
echo "==> Building Python wheels..."
uv build packages/nexus-sdk --wheel --out-dir "$DIST/"
uv build packages/nexus-coordinator --wheel --out-dir "$DIST/"

echo ""
echo "==> Release artifacts in dist/:"
ls -lh "$DIST/"

echo ""
echo "==> To validate wheels: pip install twine && twine check dist/*.whl"
