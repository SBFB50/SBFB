#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# Sync sbfb-bridge.js from web/public/ to all example apps.
# Verifies SHA256 post-copy. Exit 1 on drift.
set -euo pipefail

SOURCE="web/public/sbfb-bridge.js"

if [ ! -f "$SOURCE" ]; then
  echo "ERROR: source $SOURCE not found" >&2
  exit 1
fi

SOURCE_HASH=$(sha256sum "$SOURCE" | awk '{print $1}')
FAIL=0

for dest in examples/*/sbfb-bridge.js; do
  dir=$(dirname "$dest")
  cp "$SOURCE" "$dest"
  DEST_HASH=$(sha256sum "$dest" | awk '{print $1}')
  if [ "$SOURCE_HASH" != "$DEST_HASH" ]; then
    echo "FAIL: $dest SHA256 mismatch" >&2
    FAIL=1
  else
    echo "OK: $dest ($DEST_HASH)"
  fi
done

if [ "$FAIL" -ne 0 ]; then
  echo "ERROR: SHA256 drift detected" >&2
  exit 1
fi

echo "sync-bridge-sdk: all copies match ($SOURCE_HASH)"
