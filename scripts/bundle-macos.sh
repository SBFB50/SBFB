#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Create a macOS .app bundle for nexus-launcher.
# Usage: ./scripts/bundle-macos.sh [path/to/nexus-launcher]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

BINARY="${1:-$REPO_ROOT/target/release/nexus-launcher}"
if [[ ! -x "$BINARY" ]]; then
    echo "Error: binary not found at $BINARY" >&2
    echo "Build first: cargo build -p nexus-launcher --release" >&2
    exit 1
fi

APP_DIR="$REPO_ROOT/target/NexusGrid.app"
CONTENTS="$APP_DIR/Contents"

rm -rf "$APP_DIR"
mkdir -p "$CONTENTS/MacOS" "$CONTENTS/Resources"

cp "$BINARY" "$CONTENTS/MacOS/nexus-launcher"
chmod +x "$CONTENTS/MacOS/nexus-launcher"
cp "$REPO_ROOT/configs/macos/Info.plist" "$CONTENTS/Info.plist"

# Generate .icns from PNG using the cross-platform Rust tool (S37 D2).
if [[ -f "$REPO_ROOT/assets/nexus-launcher.png" ]]; then
    cargo run -p png-to-icns --release -- \
        "$REPO_ROOT/assets/nexus-launcher.png" \
        "$REPO_ROOT/assets/nexus-launcher.icns"
fi

if [[ -f "$REPO_ROOT/assets/nexus-launcher.icns" ]]; then
    cp "$REPO_ROOT/assets/nexus-launcher.icns" "$CONTENTS/Resources/nexus-launcher.icns"
elif [[ -f "$REPO_ROOT/assets/nexus-launcher.png" ]]; then
    echo "Warning: .icns generation failed, using .png fallback"
    cp "$REPO_ROOT/assets/nexus-launcher.png" "$CONTENTS/Resources/nexus-launcher.png"
fi

echo "Created $APP_DIR"
echo "To install: cp -r $APP_DIR /Applications/"
