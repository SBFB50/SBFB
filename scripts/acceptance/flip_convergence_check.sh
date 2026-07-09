#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 81 Phase H — live-flip convergence check (transport axis).
#
# One parameterized check, run against ONE daemon at a time, that turns
# the Phase H runbook's "convergence verifiee apres CHAQUE noeud" from
# manual curl prose into a committed, machine-readable gate (preflight H
# go/no-go item; same closed T2 vocabulary as the b3 harness). Two uses,
# same script, switched only by WHICH app you target:
#
#   - LOCAL health (right after a node's first 1.0.1 boot): target an
#     app the node itself seeds. Proves boot + store migration + local
#     serving without needing any peer (the first flipped node is
#     PARTITIONED from the 0.98 rest by design, R4).
#   - CROSS-node convergence (from the 2nd flipped node onward, and
#     again after the VPS flip): target an app published/seeded by the
#     OTHER 1.0.1 node. Proves docs-sync + gossip + blob fetch across
#     the migrated pair (the E3 acceptance couple: browse reachable +
#     blob byte-integrity).
#
# The check chain, in order (each failure names its stage):
#   0. GET /health                      -> daemon up?        else RIG-ABSENT
#   1. GET /auth/token                  -> loopback token    else RIG-ABSENT
#   2. GET /api/daemon/info             -> node_id; if EXPECT_NODE_ID is
#      set and differs -> BLOCK (identity regression — the runbook's
#      hard STOP: restore tar + redeploy 0.98). This is the empirical
#      assert closing preflight H-S1a-02/03 (warn-only regeneration at
#      runtime.rs load_or_generate_node_key would otherwise pass silently).
#   3. poll GET /api/daemon/browse until the entry whose archive_hash
#      == ARCHIVE_HASH has status "reachable" (budget GATE_TIMEOUT_SECS)
#      -> else BLOCK (browse convergence not reached).
#   4. GET /blob-serve/{ARCHIVE_HASH}/index.html, sha256 compared to
#      BASELINE_SHA256 captured BEFORE the flip -> mismatch is BLOCK
#      (byte-integrity), match is PASS.
#
# === Machine-readable contract (T2 gate, closed vocabulary) ===
# Every exit writes a JSON artefact (default
# `scripts/acceptance/.flip_last_result.json`, override FLIP_ARTIFACT)
# AND exits with a status-specific code — never a prose-only verdict:
#   - PASS       exit 0 — node healthy, app reachable, bytes identical.
#   - BLOCK      exit 1 — daemon up but a convergence/identity/integrity
#                         gate failed; diagnosis names the stage.
#   - RIG-ABSENT exit 3 — the daemon itself is not reachable/authable
#                         (not a product failure; the check could not run).
# Artefact shape:
#   {"status","stage","node_id","archive_hash","sha256","baseline_sha256",
#    "delay_s","diagnosis"}
#
# Env:
#   BASE               daemon base URL       (default http://127.0.0.1:7654)
#   ARCHIVE_HASH       blake3 hex of the app archive to probe (REQUIRED)
#   BASELINE_SHA256    sha256 hex of that app's index.html captured
#                      BEFORE the flip (REQUIRED — the byte-integrity
#                      baseline; capture it with this same script's
#                      --capture-baseline mode, below)
#   EXPECT_NODE_ID     64-hex node_id captured BEFORE the flip (optional
#                      but STRONGLY recommended on every node; the VPS
#                      flip MUST set it)
#   REQUIRE_NODE_ID    set to 1 to make the identity assert FAIL-CLOSED:
#                      an empty/missing EXPECT_NODE_ID is then RIG-ABSENT
#                      instead of silently skipping the check. MANDATORY
#                      on the VPS flip (THREAT_MODEL §15.5 row 3 — the
#                      identity assert is the only automatic backstop of
#                      the warn-only node_key regeneration)
#   GATE_TIMEOUT_SECS  browse-reachable poll budget (default 60; the E3
#                      live palier converged in <=10s)
#   POLL_SECS          poll interval (default 2)
#   FLIP_ARTIFACT      artefact path (default scripts/acceptance/.flip_last_result.json)
#
# Baseline capture mode (run BEFORE the flip, against the pre-flip node):
#   flip_convergence_check.sh --capture-baseline
#     prints `node_id`, and for ARCHIVE_HASH (if set) the sha256 of its
#     index.html — paste these into the flip session notes / runbook.
#
# Usage examples (from the Phase H runbook):
#   # before flipping anything (per node):
#   BASE=http://127.0.0.1:7654 ARCHIVE_HASH=62a6ab... ./flip_convergence_check.sh --capture-baseline
#   # after each node's first 1.0.1 boot (local health):
#   EXPECT_NODE_ID=<captured> ARCHIVE_HASH=<own app> BASELINE_SHA256=<captured> ./flip_convergence_check.sh
#   # after the 2nd node is on 1.0.1 (cross-node, run on either side):
#   EXPECT_NODE_ID=<captured> ARCHIVE_HASH=<other node's app> BASELINE_SHA256=<captured> ./flip_convergence_check.sh

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

BASE="${BASE:-http://127.0.0.1:7654}"
ARCHIVE_HASH="${ARCHIVE_HASH:-}"
BASELINE_SHA256="${BASELINE_SHA256:-}"
EXPECT_NODE_ID="${EXPECT_NODE_ID:-}"
REQUIRE_NODE_ID="${REQUIRE_NODE_ID:-}"
GATE_TIMEOUT_SECS="${GATE_TIMEOUT_SECS:-60}"
POLL_SECS="${POLL_SECS:-2}"
FLIP_ARTIFACT="${FLIP_ARTIFACT:-$SCRIPT_DIR/.flip_last_result.json}"

# Runtime state referenced by the artefact writer (filled as we progress).
NODE_ID=""
GOT_SHA256=""
DELAY=""

log() { printf '[flip] %s\n' "$*"; }

# --- JSON artefact (machine-readable, written on every exit) --------------
# Same encoding strategy as b3_live_pc_vps.sh: python3 json.dumps when
# available (bullet-proof for quotes/backslashes), else a lossy-but-valid
# fallback that deletes unsafe chars.
_json_safe() {
  printf '%s' "$1" | tr -d '\\"' | tr -d '\000-\037'
}

emit_artifact() {
  # $1=status $2=stage $3=diagnosis
  local status="$1" stage="$2" diag="$3"
  local delay="${DELAY:-}"
  [ -z "$delay" ] && delay="null"
  mkdir -p "$(dirname "$FLIP_ARTIFACT")" 2>/dev/null || true
  if command -v python3 >/dev/null 2>&1; then
    F_STATUS="$status" F_STAGE="$stage" F_DELAY="$delay" \
    F_NODE="$NODE_ID" F_HASH="$ARCHIVE_HASH" F_SHA="$GOT_SHA256" \
    F_BASE_SHA="$BASELINE_SHA256" F_DIAG="$diag" \
    python3 -c '
import json, os
def num(v):
    try:
        return int(v)
    except Exception:
        return None
print(json.dumps({
    "status": os.environ["F_STATUS"],
    "stage": os.environ["F_STAGE"],
    "node_id": os.environ["F_NODE"],
    "archive_hash": os.environ["F_HASH"],
    "sha256": os.environ["F_SHA"],
    "baseline_sha256": os.environ["F_BASE_SHA"],
    "delay_s": num(os.environ["F_DELAY"]),
    "diagnosis": os.environ["F_DIAG"],
}))' >"$FLIP_ARTIFACT"
  else
    printf '{"status":"%s","stage":"%s","node_id":"%s","archive_hash":"%s","sha256":"%s","baseline_sha256":"%s","delay_s":%s,"diagnosis":"%s"}\n' \
      "$status" "$stage" "$(_json_safe "$NODE_ID")" \
      "$(_json_safe "$ARCHIVE_HASH")" "$(_json_safe "$GOT_SHA256")" \
      "$(_json_safe "$BASELINE_SHA256")" "$delay" "$(_json_safe "$diag")" \
      >"$FLIP_ARTIFACT"
  fi
  cat "$FLIP_ARTIFACT" 2>/dev/null || true
}

rig_absent() {
  emit_artifact "RIG-ABSENT" "$1" "$2"
  printf '[flip][RIG-ABSENT] %s\n' "$2" >&2
  exit 3
}

block() {
  emit_artifact "BLOCK" "$1" "$2"
  printf '[flip][BLOCK] %s\n' "$2" >&2
  exit 1
}

pass() {
  emit_artifact "PASS" "$1" "$2"
  printf '[flip][PASS] %s\n' "$2"
  exit 0
}

# --- portable sha256 (Win Git Bash / Mac / Linux) --------------------------
sha256_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | cut -d' ' -f1
  else
    return 1
  fi
}

# Fetch a blob-serve body and echo its sha256 ONLY on HTTP 200 — a 404/5xx
# body must never masquerade as content bytes. On non-200 the HTTP code is
# echoed prefixed with "HTTP:" so callers can name it in the diagnosis.
fetch_blob_sha() {
  # $1=url
  local tmp code sha
  tmp="$(mktemp)"
  code="$(curl -s --max-time 30 -o "$tmp" -w '%{http_code}' "$1")"
  if [ "$code" != "200" ]; then
    rm -f "$tmp"
    printf 'HTTP:%s' "$code"
    return 1
  fi
  sha="$(sha256_stdin <"$tmp")" || { rm -f "$tmp"; return 2; }
  rm -f "$tmp"
  printf '%s' "$sha"
}

# --- 0. daemon up? ----------------------------------------------------------
HEALTH="$(curl -s --max-time 5 "$BASE/health" 2>/dev/null)" || true
if [ -z "$HEALTH" ]; then
  rig_absent "health" "daemon not reachable at $BASE (is it running?)"
fi

# --- 1. loopback token ------------------------------------------------------
TOKEN="$(curl -s --max-time 5 "$BASE/auth/token" | sed -E 's/.*"token":"([^"]+)".*/\1/')"
if [ -z "$TOKEN" ] || printf '%s' "$TOKEN" | grep -q '{'; then
  rig_absent "auth" "could not parse loopback token from $BASE/auth/token"
fi
H=(-H "x-sbfb-token: $TOKEN")

# --- 2. identity: node_id ---------------------------------------------------
INFO="$(curl -s --max-time 10 "${H[@]}" "$BASE/api/daemon/info")"
NODE_ID="$(printf '%s' "$INFO" | sed -E 's/.*"node_id":"([0-9a-f]{64})".*/\1/')"
if ! printf '%s' "$NODE_ID" | grep -qE '^[0-9a-f]{64}$'; then
  NODE_ID=""
  rig_absent "info" "no 64-hex node_id in /api/daemon/info response"
fi
log "node_id: $NODE_ID"

# --- capture-baseline mode: print reference values and exit ----------------
if [ "${1:-}" = "--capture-baseline" ]; then
  echo "EXPECT_NODE_ID=$NODE_ID"
  if [ -n "$ARCHIVE_HASH" ]; then
    BODY_SHA="$(fetch_blob_sha "$BASE/blob-serve/$ARCHIVE_HASH/index.html")" \
      || rig_absent "baseline" "index.html not fetchable pre-flip ($BODY_SHA) or no sha256 tool — fix before flipping"
    echo "BASELINE_SHA256=$BODY_SHA"
  else
    log "ARCHIVE_HASH not set — skipping baseline sha256 capture"
  fi
  # Baseline capture is informational; write a PASS artefact for the trace.
  GOT_SHA256="${BODY_SHA:-}"
  pass "baseline" "pre-flip baseline captured (node_id + sha256 above)"
fi

if [ -z "$ARCHIVE_HASH" ] || [ -z "$BASELINE_SHA256" ]; then
  rig_absent "params" "ARCHIVE_HASH and BASELINE_SHA256 are required (capture them pre-flip with --capture-baseline)"
fi

# Fail-closed identity mode: REQUIRE_NODE_ID=1 (mandatory on the VPS flip)
# refuses to run at all without a reference node_id — the assert below is
# the only automatic backstop of the warn-only node_key regeneration, so
# skipping it silently must not be possible on the node where it matters.
if [ "$REQUIRE_NODE_ID" = "1" ] && [ -z "$EXPECT_NODE_ID" ]; then
  rig_absent "params" "REQUIRE_NODE_ID=1 but EXPECT_NODE_ID is empty — capture it pre-flip with --capture-baseline"
fi

if [ -n "$EXPECT_NODE_ID" ] && [ "$NODE_ID" != "$EXPECT_NODE_ID" ]; then
  block "identity" "node_id DIVERGED after flip: expected $EXPECT_NODE_ID got $NODE_ID — STOP: restore tar + redeploy 0.98 (runbook rollback, 2 gestures)"
fi

# --- 3. browse: entry reachable within budget -------------------------------
START="$(date +%s)"
DEADLINE=$((START + GATE_TIMEOUT_SECS))
FOUND=""
MATCHES=""
while [ "$(date +%s)" -lt "$DEADLINE" ]; do
  BROWSE="$(curl -s --max-time 10 "${H[@]}" "$BASE/api/daemon/browse")"
  # Split the flat JSON into per-object lines and keep EVERY entry carrying
  # our archive_hash — the same hash can appear on several entries (own +
  # distant, §P59); the gate passes iff AT LEAST ONE of them is reachable,
  # never just the first one in aggregator order.
  MATCHES="$(printf '%s' "$BROWSE" | tr '}' '\n' | grep "$ARCHIVE_HASH")" || true
  if [ -n "$MATCHES" ] && printf '%s\n' "$MATCHES" | grep -q '"status":"reachable"'; then
    FOUND=1
    break
  fi
  sleep "$POLL_SECS"
done
DELAY=$(( $(date +%s) - START ))
if [ -z "$FOUND" ]; then
  if [ -n "$MATCHES" ]; then
    block "browse" "entry present but never status=reachable within ${GATE_TIMEOUT_SECS}s (probe/connectivity — check relay/discovery config on this node)"
  else
    block "browse" "archive_hash never appeared in browse within ${GATE_TIMEOUT_SECS}s (docs-sync/gossip convergence not reached)"
  fi
fi
log "browse reachable after ${DELAY}s"

# --- 4. blob byte-integrity --------------------------------------------------
GOT_SHA256="$(fetch_blob_sha "$BASE/blob-serve/$ARCHIVE_HASH/index.html")" || {
  case "$GOT_SHA256" in
    HTTP:*) block "blob" "index.html not served after flip ($GOT_SHA256) despite browse reachable" ;;
    *) rig_absent "blob" "no sha256 tool (sha256sum/shasum) on this host" ;;
  esac
}
if [ "$GOT_SHA256" != "$BASELINE_SHA256" ]; then
  block "blob" "index.html sha256 diverged: baseline $BASELINE_SHA256 got $GOT_SHA256"
fi

pass "converged" "browse reachable in ${DELAY}s + index.html byte-identical to pre-flip baseline"
