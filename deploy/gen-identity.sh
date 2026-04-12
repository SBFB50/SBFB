#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Generate a persistent Ed25519 keypair for a nexus-grid bootstrap node.
# Run on the VPS after provisioning:
#   ./deploy/gen-identity.sh
#
# Stores the keypair in /opt/nexus-grid/identity/ and prints the
# public node ID for hardcoding in the bootstrap peer list.

set -euo pipefail

IDENTITY_DIR="/opt/nexus-grid/identity"
KEY_FILE="$IDENTITY_DIR/secret_key.bin"

if [[ -f "$KEY_FILE" ]]; then
  echo "Identity already exists at $KEY_FILE"
  echo "Public key:"
  # The shell daemon reads the key on startup and prints its node ID.
  # For now, just note the file exists.
  echo "  (start nexus-shell-daemon to see the node ID in logs)"
  exit 0
fi

echo "==> Generating Ed25519 keypair..."
mkdir -p "$IDENTITY_DIR"

# Generate 32 random bytes as the Ed25519 secret key seed.
# iroh's Endpoint::builder().secret_key(key) accepts a SecretKey
# constructed from 32 bytes.
dd if=/dev/urandom of="$KEY_FILE" bs=32 count=1 2>/dev/null
chmod 600 "$KEY_FILE"
chown nexus:nexus "$KEY_FILE"

echo "  Secret key: $KEY_FILE (600 permissions, nexus:nexus)"
echo "  Start nexus-shell-daemon to see the public node ID."
echo ""
echo "  IMPORTANT: back up $KEY_FILE — losing it means losing"
echo "  this node's identity in the DHT. Other peers would need"
echo "  to update their bootstrap list."
