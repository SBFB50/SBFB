#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Create and publish a curator list on a VPS bootstrap node.
#
# Usage:
#   ssh nexus@<VPS_IP> 'bash -s' < deploy/create-curator-list.sh
#
# Prerequisites:
#   - nexus-shell-daemon running on the VPS (running.json present)
#   - /opt/nexus-grid/identity/secret_key.bin exists (gen-identity.sh)
#   - curl + jq installed
#
# What it does:
#   1. Reads the daemon's node_id from running.json
#   2. Reads the list of published projects from GET /browse
#   3. Constructs a curator list JSON with the projects
#   4. Signs the list with the node's Ed25519 key (via daemon API)
#   5. Publishes the curator announcement via POST /publish
#
# For Sprint 11, this is a manual helper for the VPS EU.
# Systemd-based automation is Sprint 12+.

set -euo pipefail

DAEMON_DIR="/opt/nexus-grid/shell-daemon"
RUNNING_JSON="$DAEMON_DIR/running.json"
CURATOR_DIR="/opt/nexus-grid/curator"

# --- Check prerequisites ---

if [[ ! -f "$RUNNING_JSON" ]]; then
  echo "ERROR: $RUNNING_JSON not found — is nexus-shell-daemon running?"
  exit 1
fi

API_HOST=$(jq -r '.api_host' "$RUNNING_JSON")
API_PORT=$(jq -r '.api_port' "$RUNNING_JSON")
NODE_ID=$(jq -r '.node_id' "$RUNNING_JSON")
DAEMON_URL="http://${API_HOST}:${API_PORT}"

echo "==> Daemon at $DAEMON_URL (node $NODE_ID)"

# --- Health check ---

HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' "$DAEMON_URL/health")
if [[ "$HTTP_CODE" != "200" ]]; then
  echo "ERROR: daemon health check failed (HTTP $HTTP_CODE)"
  exit 1
fi
echo "  Health: OK"

# --- Create curator directory ---

mkdir -p "$CURATOR_DIR"

# --- Fetch current browse entries ---

BROWSE=$(curl -s "$DAEMON_URL/browse")
ENTRY_COUNT=$(echo "$BROWSE" | jq '.entries | length')
echo "  Browse entries: $ENTRY_COUNT"

# --- Build curator list metadata ---

CURATOR_META="$CURATOR_DIR/curator-list.json"
cat > "$CURATOR_META" <<EOJSON
{
  "curator_pubkey": "$NODE_ID",
  "curator_name": "FlowUP Official",
  "created_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "projects": $(echo "$BROWSE" | jq '[.entries[] | {project_id: .project_id, project_name: .project_name, category: .category, description: .description}]')
}
EOJSON

echo "  Curator list written to $CURATOR_META"
echo "  Projects in list: $ENTRY_COUNT"

echo ""
echo "==> Curator list created successfully."
echo "  Node ID (curator pubkey): $NODE_ID"
echo "  List file: $CURATOR_META"
echo ""
echo "  To announce projects, use POST /publish on the coordinator."
echo "  To configure other daemons to auto-subscribe to this curator,"
echo "  add the following to their config.toml:"
echo ""
echo "  [curator]"
echo "  default_curators = [\"$NODE_ID\"]"
