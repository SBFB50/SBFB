#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 33 Phase C — 2-daemon localhost smoke test.
#
# Spawns 2 nexus-shell-daemon instances on ephemeral ports with
# isolated NEXUS_GRID_ROOT dirs, waits for /health, verifies
# distinct node_ids, then shuts both down.
#
# Usage:
#   bash scripts/test-multi-node.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info()  { printf "${GREEN}[SMOKE]${NC} %s\n" "$*"; }
fail()  { printf "${RED}[FAIL]${NC}  %s\n" "$*" >&2; exit 1; }

cleanup() {
    info "Cleaning up..."
    for pid in "${DAEMON_PIDS[@]}"; do
        if kill -0 "$pid" 2>/dev/null; then
            kill -INT "$pid" 2>/dev/null || true
            wait "$pid" 2>/dev/null || true
        fi
    done
    rm -rf "${TMP_DIRS[@]}" 2>/dev/null || true
}
trap cleanup EXIT

DAEMON_PIDS=()
TMP_DIRS=()
PORTS=()
NODE_IDS=()

DAEMON_BIN="$REPO_ROOT/target/debug/nexus-shell-daemon"
if [[ ! -x "$DAEMON_BIN" ]]; then
    DAEMON_BIN="$REPO_ROOT/target/release/nexus-shell-daemon"
fi
if [[ ! -x "$DAEMON_BIN" ]]; then
    info "Building nexus-shell-daemon (debug)..."
    cargo build -p nexus-shell-daemon --manifest-path "$REPO_ROOT/Cargo.toml"
    DAEMON_BIN="$REPO_ROOT/target/debug/nexus-shell-daemon"
fi
info "Using daemon: $DAEMON_BIN"

spawn_daemon() {
    local idx=$1
    local root
    root=$(mktemp -d)
    local sbfb_home
    sbfb_home=$(mktemp -d)
    TMP_DIRS+=("$root" "$sbfb_home")

    NEXUS_GRID_ROOT="$root" SBFB_HOME="$sbfb_home" RUST_LOG=warn \
        "$DAEMON_BIN" start --headless &
    local pid=$!
    DAEMON_PIDS+=("$pid")
    info "Daemon $idx spawned (pid $pid, root $root)"

    local running_json="$root/shell-daemon/running.json"
    local tries=0
    while [[ ! -f "$running_json" ]] && (( tries < 120 )); do
        sleep 0.25
        (( tries++ ))
    done
    if [[ ! -f "$running_json" ]]; then
        fail "Daemon $idx: running.json not created after 30s"
    fi

    local port
    port=$(python3 -c "import json,sys; print(json.load(open('$running_json'))['api_port'])" 2>/dev/null || true)
    if [[ -z "$port" ]]; then
        port=$(grep -o '"api_port":[0-9]*' "$running_json" | grep -o '[0-9]*')
    fi
    PORTS+=("$port")

    local node_id
    node_id=$(python3 -c "import json,sys; print(json.load(open('$running_json'))['node_id'])" 2>/dev/null || true)
    if [[ -z "$node_id" ]]; then
        node_id=$(grep -o '"node_id":"[^"]*"' "$running_json" | cut -d'"' -f4)
    fi
    NODE_IDS+=("$node_id")

    info "Daemon $idx: port=$port, node_id=${node_id:0:16}..."
}

check_health() {
    local idx=$1
    local port=${PORTS[$idx]}
    local tries=0
    while (( tries < 40 )); do
        local status
        status=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:$port/health" 2>/dev/null || echo "000")
        if [[ "$status" == "200" ]]; then
            info "Daemon $idx: health OK"
            return 0
        fi
        sleep 0.25
        (( tries++ ))
    done
    fail "Daemon $idx: health check failed after 10s"
}

# ── Main ─────────────────────────────────────────────────────

info "=== SBFB 2-daemon smoke test ==="

spawn_daemon 0
spawn_daemon 1

check_health 0
check_health 1

if [[ "${NODE_IDS[0]}" == "${NODE_IDS[1]}" ]]; then
    fail "Both daemons have the same node_id: ${NODE_IDS[0]}"
fi
info "Node IDs are distinct ✓"

if [[ "${PORTS[0]}" == "${PORTS[1]}" ]]; then
    fail "Both daemons bound to the same port: ${PORTS[0]}"
fi
info "Ports are distinct ✓"

info "=== Smoke test PASSED ==="
