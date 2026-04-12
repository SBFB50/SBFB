#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Build and deploy the nexus-grid shell web to a VPS nginx root.
#
# Usage:
#   ./deploy/deploy-web.sh --host <ip> --key <ssh_key>
#
# Prerequisites:
#   - VPS provisioned via provision.sh (nginx installed, /opt/nexus-grid/web/ exists)
#   - Node.js + npm available locally for the build

set -euo pipefail

HOST=""
KEY=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --host) HOST="$2"; shift 2 ;;
    --key) KEY="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [[ -z "$HOST" || -z "$KEY" ]]; then
  echo "Usage: deploy-web.sh --host <ip> --key <ssh_key_path>"
  exit 1
fi

SSH="ssh -i $KEY -o StrictHostKeyChecking=accept-new nexus@$HOST"
SCP="scp -i $KEY -o StrictHostKeyChecking=accept-new"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR="$SCRIPT_DIR/../web"

echo "==> Building web shell..."
cd "$WEB_DIR"
npm ci --silent
npm run build

echo "==> Uploading to $HOST:/opt/nexus-grid/web/..."
# Clear old files first, then upload fresh build.
$SSH "rm -rf /opt/nexus-grid/web/*"
$SCP -r dist/* "nexus@$HOST:/opt/nexus-grid/web/"

echo "==> Reloading nginx..."
$SSH "sudo systemctl reload nginx"

echo "==> Smoke test..."
HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' "http://$HOST/")
if [[ "$HTTP_CODE" == "200" ]]; then
  echo "  HTTP $HTTP_CODE — OK"
else
  echo "  WARNING: HTTP $HTTP_CODE (expected 200)"
fi

echo ""
echo "==> Web deploy to $HOST complete."
