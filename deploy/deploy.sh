#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# nexus-grid binary deployment script.
# Usage:
#   ./deploy/deploy.sh --host <ip> --key <ssh_key> --role daemon
#   ./deploy/deploy.sh --host <ip> --key <ssh_key> --role coordinator
#
# Prerequisites: VPS provisioned via provision.sh, binaries in dist/.

set -euo pipefail

HOST=""
KEY=""
ROLE="daemon"

while [[ $# -gt 0 ]]; do
  case $1 in
    --host) HOST="$2"; shift 2 ;;
    --key) KEY="$2"; shift 2 ;;
    --role) ROLE="$2"; shift 2 ;;
    *) echo "Unknown option: $1"; exit 1 ;;
  esac
done

if [[ -z "$HOST" || -z "$KEY" ]]; then
  echo "Usage: deploy.sh --host <ip> --key <ssh_key_path> --role daemon|coordinator"
  exit 1
fi

SSH="ssh -i $KEY -o StrictHostKeyChecking=accept-new nexus@$HOST"
SCP="scp -i $KEY -o StrictHostKeyChecking=accept-new"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DIST="$REPO_ROOT/dist"

echo "==> Deploying to $HOST (role: $ROLE)"

# ── Upload daemon binary ────────────────────────────────────
echo "  [1/4] Uploading nexus-shell-daemon..."
$SCP "$DIST/nexus-shell-daemon-linux-x86_64" \
  "nexus@$HOST:/opt/nexus-grid/bin/nexus-shell-daemon"
$SSH "chmod +x /opt/nexus-grid/bin/nexus-shell-daemon"

# ── Upload coordinator binary (if coordinator role) ─────────
if [[ "$ROLE" == "coordinator" ]]; then
  echo "  [2/4] Uploading nexus-worker + coordinator..."
  $SCP "$DIST/nexus-worker-linux-x86_64" \
    "nexus@$HOST:/opt/nexus-grid/bin/nexus-worker"
  $SSH "chmod +x /opt/nexus-grid/bin/nexus-worker"

  # Upload Python coordinator package
  echo "  Uploading coordinator wheel..."
  $SCP "$DIST"/nexus_coordinator-*.whl "$DIST"/nexus_sdk-*.whl \
    "nexus@$HOST:/tmp/"
  $SSH "pip install --user /tmp/nexus_coordinator-*.whl /tmp/nexus_sdk-*.whl"
else
  echo "  [2/4] Skipped (daemon-only role)"
fi

# ── Restart services ────────────────────────────────────────
echo "  [3/4] Restarting services..."
$SSH "sudo systemctl restart nexus-daemon"

if [[ "$ROLE" == "coordinator" ]]; then
  $SSH "sudo systemctl restart nexus-coordinator || true"
fi

# ── Smoke test ──────────────────────────────────────────────
echo "  [4/4] Smoke test..."
sleep 3
$SSH "systemctl is-active nexus-daemon"

if [[ "$ROLE" == "coordinator" ]]; then
  $SSH "systemctl is-active nexus-coordinator || echo 'coordinator not active yet (may need init)'"
fi

echo ""
echo "==> Deploy to $HOST complete."
