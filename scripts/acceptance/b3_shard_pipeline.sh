#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 77 Phase K — sharded inference acceptance harness (T2 gate).
#
# Extends `b3_live_pc_vps.sh` (single-machine compute) to the SHARDED
# pipeline: a ~20 GB arch-llama model is split across TWO heterogeneous
# machines — an RTX 5080 (CUDA) and a Mac M2 (Metal) — over the
# `sbfb/shard/1` data-plane, with each machine loading ONLY its layer
# block (pipeline-parallel, P-D). The model fits on NEITHER machine
# alone, which is the whole point: it proves the sharding value.
#
# === Machine-readable contract (T2 testability gate) ===
# Every exit writes a JSON artefact (default
# `scripts/acceptance/.b3_shard_last_result.json`, override
# B3_ARTIFACT) AND exits with a status-specific code, so the result is
# never a prose-only "DIFFERE-materiel" (README §4 testability gate):
#   - PASS       exit 0  — a sharded generation completed end-to-end:
#                          a signed RunProof was collected for each
#                          shard AND toks_per_s >= 1 (the §14.4 floor).
#   - BLOCK      exit 1  — the session mounted but the generation did
#                          not converge within budget, OR a network
#                          gate tripped (BLOCK{rtt>80ms} /
#                          BLOCK{relay-hot-path} = NO-GO, not a timeout
#                          to inflate). The product question we test.
#   - RIG-ABSENT exit 3  — the rig is not present/functioning (a host
#                          down, the ~20 GB model missing, the second
#                          Metal machine unconfigured, a project
#                          mismatch) OR no live shard-session is
#                          mountable end-to-end. NOT a product failure;
#                          the test simply could not run.
# Artefact shape (§14.2):
#   {"status","stage","model","n_shards","ttft_s","toks_per_s",
#    "rtt_frontier_ms","run_proof","diagnosis","last_response"}
#
# === Honest expected status TODAY: RIG-ABSENT ===
# The cross-machine sharding CORE is delivered and hermetically tested
# (placement, routing, N0-N3 verification, the `sbfb/shard/1`
# data-plane boundary-frame forwarder over a long-lived bi-stream with
# admission, the forked layer-block backend, the worker claim). What is
# NOT yet wired is the
# production SESSION ORCHESTRATOR that mounts a shard session, drives a
# token-by-token cross-shard generation, measures TTFT/tok-s, and
# collects an in-vivo signed RunProof:
#   - the daemon read-only route `GET /api/daemon/shard-session/{id}`
#     is a Phase J STUB (`live_shard_session` returns `None`, so the
#     envelope is always `{found:false, session:null}`);
#   - no production caller constructs or signs a `RunProof` in-vivo;
#     the only `RunProof::new` / `RunProofEntry::sign` call-sites live
#     under `#[cfg(test)]`;
#   - `open_shard_connection` documents a "caller" that drives a
#     long-lived bi-stream generation — that caller does not exist in
#     production yet.
# So `ttft_s` / `toks_per_s` / `run_proof` are populated by NO
# production path, and `pass()` (which REQUIRES a non-empty run_proof
# AND toks_per_s >= 1) is structurally unreachable today, with or
# without the 2-machine rig. This harness therefore reports RIG-ABSENT
# with a precise diagnosis. This is PLANNED by plan §14.4 (the shard
# feature stays PROVISIONAL + carry P1 into S78); the JSON `status`
# field is the verdict, never a prose excuse.
# The harness is written FORWARD: once the S78 session orchestrator
# lands, the `session-mount` stage returns a live session and the
# generation/measurement/churn stages below run unchanged, producing a
# real PASS or BLOCK.
#
# Rig config as DATA: drop a gitignored
# `scripts/acceptance/rig.local.env` (override path with RIG_ENV)
# exporting PC_DAEMON / MAC_SSH / PROJECT_ID / MODEL_20GB / WORKER_BIN
# / SHARD_SESSION_ID. It is sourced first; explicit env vars still win.
# A template lives next to this script as `rig.local.env.example`.
#
# Usage (from the RTX 5080 host, the pipeline head):
#   # one-time: scripts/acceptance/rig.local.env (gitignored)
#   #   export PC_DAEMON=http://127.0.0.1:7777
#   #   export MAC_SSH=user@mac-m2.local
#   #   export PROJECT_ID=<project-doc-id the PC daemon hosts>
#   #   export MODEL_20GB=mixtral:8x7b-instruct-v0.1-q3_K_M
#   #   export WORKER_BIN=./target/release/nexus-worker
#   #   export SHARD_SESSION_ID=<a live shard-session id, once one exists>
#   bash scripts/acceptance/b3_shard_pipeline.sh

set -uo pipefail

# Resolve the script's own dir so the rig config + artefact land next to
# it regardless of the caller's cwd (the JSON artefact is the T2 gate's
# source of truth — it must never be silently lost to a wrong cwd).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- 0. Rig config as data ------------------------------------------------
RIG_ENV="${RIG_ENV:-$SCRIPT_DIR/rig.local.env}"
if [ -f "$RIG_ENV" ]; then
  # shellcheck disable=SC1090
  . "$RIG_ENV"
fi

PC_DAEMON="${PC_DAEMON:-http://127.0.0.1:7777}"
MAC_SSH="${MAC_SSH:-}"
PROJECT_ID="${PROJECT_ID:-}"
MODEL_20GB="${MODEL_20GB:-}"
WORKER_BIN="${WORKER_BIN:-}"
SHARD_SESSION_ID="${SHARD_SESSION_ID:-}"
N_SHARDS="${N_SHARDS:-2}"
GATE_TIMEOUT_SECS="${GATE_TIMEOUT_SECS:-120}"
RTT_GATE_MS="${RTT_GATE_MS:-80}"
POLL_SECS="${POLL_SECS:-2}"
OLLAMA_URL="${OLLAMA_URL:-http://127.0.0.1:11434}"
PROMPT="${PROMPT:-In one word, what is the capital of France?}"
B3_ARTIFACT="${B3_ARTIFACT:-$SCRIPT_DIR/.b3_shard_last_result.json}"

# Runtime state referenced by the artefact writer (filled as we progress).
MODEL="$MODEL_20GB"
TTFT_S=""
TOKS_PER_S=""
RTT_FRONTIER_MS=""
RUN_PROOF=""
LAST_RESPONSE=""

log() { printf '[b3-shard] %s\n' "$*"; }

# --- JSON artefact (machine-readable, written on every exit) --------------
# Encoding via python3 (json.dumps — bullet-proof for backslashes, quotes,
# and newlines that `last_response` routinely carries). The no-python3
# fallback DELETES the unsafe chars so the artefact is always valid JSON,
# lossy. Numeric fields go through num(): an integer or JSON null, never a
# string (n_shards / ttft_s / toks_per_s / rtt_frontier_ms).
_json_safe() {
  printf '%s' "$1" | tr -d '\\"' | tr -d '\000-\037'
}

emit_artifact() {
  # $1=status $2=stage $3=diagnosis
  local status="$1" stage="$2" diag="$3"
  local n_shards="${N_SHARDS:-}" ttft="${TTFT_S:-}" toks="${TOKS_PER_S:-}"
  local rtt="${RTT_FRONTIER_MS:-}" _emitted=0
  [ -z "$n_shards" ] && n_shards="null"
  [ -z "$ttft" ] && ttft="null"
  [ -z "$toks" ] && toks="null"
  [ -z "$rtt" ] && rtt="null"
  mkdir -p "$(dirname "$B3_ARTIFACT")" 2>/dev/null || true
  if command -v python3 >/dev/null 2>&1; then
    B3_STATUS="$status" B3_STAGE="$stage" B3_MODEL="${MODEL:-}" \
    B3_NSHARDS="$n_shards" B3_TTFT="$ttft" B3_TOKS="$toks" B3_RTT="$rtt" \
    B3_PROOF="${RUN_PROOF:-}" B3_DIAG="$diag" B3_RESP="${LAST_RESPONSE:-}" \
    python3 -c '
import json, os
def num(v):
    try:
        return int(v)
    except Exception:
        return None
print(json.dumps({
    "status": os.environ["B3_STATUS"],
    "stage": os.environ["B3_STAGE"],
    "model": os.environ["B3_MODEL"],
    "n_shards": num(os.environ["B3_NSHARDS"]),
    "ttft_s": num(os.environ["B3_TTFT"]),
    "toks_per_s": num(os.environ["B3_TOKS"]),
    "rtt_frontier_ms": num(os.environ["B3_RTT"]),
    "run_proof": os.environ["B3_PROOF"],
    "diagnosis": os.environ["B3_DIAG"],
    "last_response": os.environ["B3_RESP"],
}))' >"$B3_ARTIFACT" 2>/dev/null && [ -s "$B3_ARTIFACT" ] && _emitted=1
  fi
  if [ "$_emitted" -ne 1 ]; then
    # python3 absent OR failed (e.g. a non-executable Windows Store shim that
    # `command -v` finds but that errors when run) -> pure-bash encoder. Lossy
    # (drops backslash/quote/control) but always valid JSON, never a lost artefact.
    local model_e proof_e diag_e resp_e
    model_e="$(_json_safe "${MODEL:-}")"
    proof_e="$(_json_safe "${RUN_PROOF:-}")"
    diag_e="$(_json_safe "$diag")"
    resp_e="$(_json_safe "${LAST_RESPONSE:-}")"
    # Coerce numeric fields to integer-or-`null` (mirror python `num()`): a raw
    # non-numeric value reaching here — e.g. an invalid `N_SHARDS` caught at the
    # early validation rig_absent, before base-10 normalization — would otherwise
    # emit a bare `"n_shards":bad` and break JSON validity in the no-python path.
    case "$n_shards" in ''|*[!0-9]*) n_shards=null ;; esac
    case "$ttft" in ''|*[!0-9]*) ttft=null ;; esac
    case "$toks" in ''|*[!0-9]*) toks=null ;; esac
    case "$rtt" in ''|*[!0-9]*) rtt=null ;; esac
    printf '{"status":"%s","stage":"%s","model":"%s","n_shards":%s,"ttft_s":%s,"toks_per_s":%s,"rtt_frontier_ms":%s,"run_proof":"%s","diagnosis":"%s","last_response":"%s"}\n' \
      "$status" "$stage" "$model_e" "$n_shards" "$ttft" "$toks" "$rtt" \
      "$proof_e" "$diag_e" "$resp_e" \
      >"$B3_ARTIFACT"
  fi
  # Echo the artefact to the trace too.
  cat "$B3_ARTIFACT" 2>/dev/null || true
}

rig_absent() {
  # $1=reason. The rig is not present/functioning — exit 3, no product claim.
  emit_artifact "RIG-ABSENT" "preflight" "$1"
  printf '[b3-shard][RIG-ABSENT] %s\n' "$1" >&2
  exit 3
}

block() {
  # $1=stage $2=diagnosis. Session mounted but no convergence / gate trip — exit 1.
  emit_artifact "BLOCK" "$1" "$2"
  printf '[b3-shard][BLOCK] %s\n' "$2" >&2
  exit 1
}

pass() {
  # $1=stage $2=note. Reached ONLY with a non-empty run_proof AND toks_per_s>=1.
  emit_artifact "PASS" "$1" "$2"
  printf '[b3-shard][PASS] %s\n' "$2"
  exit 0
}

# --- Validate N_SHARDS ----------------------------------------------------
case "$N_SHARDS" in
  ''|*[!0-9]*) rig_absent "N_SHARDS must be a positive integer, got '$N_SHARDS'" ;;
esac
N_SHARDS=$((10#$N_SHARDS))
[ "$N_SHARDS" -ge 2 ] || rig_absent "N_SHARDS must be >= 2 (sharding splits across machines), got '$N_SHARDS'"

# ==========================================================================
# PREFLIGHT — any failure here is RIG-ABSENT (exit 3), never a product BLOCK.
# ==========================================================================
log "=== preflight (orchestrator + 2-machine rig presence + project reconciliation) ==="

# STRUCTURAL precondition first (hardware-independent): is there even a
# shard session to drive? Today there is not — no production orchestrator
# creates one (the daemon route is a Phase J read-only stub), so a sharded
# generation cannot be driven end-to-end on ANY rig. This is THE reason T2
# is RIG-ABSENT, independent of the 5080/Mac hardware, and the honest
# primary diagnosis for the S78 carry.
if [ -z "$SHARD_SESSION_ID" ]; then
  rig_absent "no shard session to drive: SHARD_SESSION_ID is unset and no production \
orchestrator creates one. The sbfb/shard/1 data-plane serves a long-lived bi-stream, \
forwarding each boundary frame through one layer block with admission control, but no \
production caller drives a token-by-token cross-shard generation, measures TTFT/tok-s, \
or emits an in-vivo RunProof. \
The HTTP route GET /api/daemon/shard-session/{id} is a Phase J read-only STUB \
(live_shard_session -> None). The sharding CORE (placement, routing, N0-N3 verification, \
the data-plane forwarder + admission, the forked layer-block backend, the worker claim) is \
delivered and hermetically tested; only the live SESSION ORCHESTRATOR remains — an S78 \
carry. Set SHARD_SESSION_ID once it lands and a session can be mounted."
fi

[ -n "$PROJECT_ID" ]  || rig_absent "PROJECT_ID unset (set it in env or $RIG_ENV)"
[ -n "$MODEL_20GB" ]  || rig_absent "MODEL_20GB unset — the ~20 GB arch-llama model that fits on NEITHER machine alone (set it in env or $RIG_ENV)"
command -v curl >/dev/null 2>&1 || rig_absent "curl not found on PATH"

# Pipeline head (RTX 5080 host) daemon reachable + loopback token.
TOKEN="$(curl -fsS "$PC_DAEMON/auth/token" 2>/dev/null | sed -n 's/.*"token":"\([^"]*\)".*/\1/p')"
[ -n "$TOKEN" ] || rig_absent "could not read loopback token from $PC_DAEMON/auth/token (is the PC daemon running?)"
AUTH="-H 'x-sbfb-token: $TOKEN'"

# Project reconciliation: the PC daemon must actually host PROJECT_ID
# (the same wrong-project confounder b3_live guards against).
PI="$(eval curl -fsS "$AUTH" "'$PC_DAEMON/api/daemon/project-info'" 2>/dev/null || true)"
LAST_RESPONSE="$PI"
PC_PID="$(printf '%s' "$PI" | sed -n 's/.*"project_doc_id"[[:space:]]*:[[:space:]]*"\{0,1\}\([^",}]*\)"\{0,1\}.*/\1/p')"
if [ -z "$PC_PID" ] || [ "$PC_PID" = "null" ]; then
  rig_absent "PC daemon has no project doc mounted (GET /api/daemon/project-info -> $PI)"
fi
if [ "$PC_PID" != "$PROJECT_ID" ]; then
  rig_absent "project mismatch: PC daemon hosts '$PC_PID' but PROJECT_ID='$PROJECT_ID' (fix PROJECT_ID to the PC project_doc id)"
fi
LAST_RESPONSE=""

# Local Ollama / GGUF: the ~20 GB model must be present on the head.
TAGS="$(curl -fsS "$OLLAMA_URL/api/tags" 2>/dev/null || true)"
if [ -n "$TAGS" ]; then
  MODEL_BASE="${MODEL_20GB%%:*}"
  if ! printf '%s' "$TAGS" | grep -q "$MODEL_BASE"; then
    rig_absent "model '$MODEL_20GB' not present on the pipeline head (ollama pull $MODEL_20GB, or place its GGUF)"
  fi
else
  log "note: Ollama not reachable at $OLLAMA_URL — assuming a GGUF-only head; the session orchestrator validates the model at mount"
fi

# Second machine (Mac M2, Metal) must be configured + reachable: sharding
# is meaningless on a single host (the model would not fit).
[ -n "$MAC_SSH" ] || rig_absent "MAC_SSH unset — the 2nd (Metal) machine is required; a ~20 GB model does not fit on the 5080 alone (set it in env or $RIG_ENV)"
command -v ssh >/dev/null 2>&1 || rig_absent "ssh not found on PATH (needed to reach the Mac M2 shard)"
if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$MAC_SSH" true >/dev/null 2>&1; then
  rig_absent "SSH to the Mac M2 shard '$MAC_SSH' failed (host down, auth, or network)"
fi

# Worker binary present (when this run is expected to enroll/drive it).
if [ -n "$WORKER_BIN" ] && [ ! -x "$WORKER_BIN" ]; then
  rig_absent "WORKER_BIN '$WORKER_BIN' is not an executable nexus-worker binary"
fi

log "preflight OK — PC daemon reachable, project reconciled ($PC_PID), model '$MODEL_20GB' present, Mac M2 shard '$MAC_SSH' reachable"

# ==========================================================================
# SESSION-MOUNT — find a live shard session to drive.
# A failure here is RIG-ABSENT (the pipeline is not mountable end-to-end),
# never a product BLOCK: BLOCK is reserved for a mounted-but-diverging run.
# ==========================================================================
log "=== session-mount (locate a live shard session over $N_SHARDS shards) ==="

# SHARD_SESSION_ID is guaranteed set by the structural preflight gate above;
# here we confirm the daemon actually exposes it as a LIVE session.
SESS="$(eval curl -fsS "$AUTH" "'$PC_DAEMON/api/daemon/shard-session/$SHARD_SESSION_ID'" 2>/dev/null || true)"
LAST_RESPONSE="$SESS"
FOUND="$(printf '%s' "$SESS" | sed -n 's/.*"found"[[:space:]]*:[[:space:]]*\(true\|false\).*/\1/p')"
if [ "$FOUND" != "true" ]; then
  rig_absent "shard session '$SHARD_SESSION_ID' is not live: GET /api/daemon/shard-session/{id} \
returned found=$FOUND. Since S81 Phase I the daemon reads a live session registry populated by \
the operator orchestrator — mount the session first (nexus-shell-daemon shard-session group + \
serve on each worker + mount, see the shard-session subcommand help), then re-run this gate. \
Response: ${SESS:-<none>}"
fi
LAST_RESPONSE=""
log "session-mount OK — shard session '$SHARD_SESSION_ID' is live"

# ==========================================================================
# GENERATION + MEASUREMENT — drive a sharded generation and measure it.
# Reached only once a live session exists (S78+). Every metric below is
# MEASURED; a missing metric BLOCKs (anti-false-green), never PASSes empty.
# ==========================================================================
log "=== generation (drive a cross-shard token-by-token decode) ==="
SUBMIT_AT="$(date +%s)"

# Frontier RTT gate: the head measures the sbfb/shard/1 boundary RTT to the
# Mac M2 shard. A relay hot-path or RTT over the gate is a product NO-GO,
# not a timeout to inflate (plan §14.2 BLOCK{rtt>80ms} / BLOCK{relay-hot-path}).
RTT_FRONTIER_MS="$(eval curl -fsS "$AUTH" "'$PC_DAEMON/api/daemon/shard-session/$SHARD_SESSION_ID'" 2>/dev/null \
  | sed -n 's/.*"rtt_frontier_ms"[[:space:]]*:[[:space:]]*\([0-9]\{1,\}\).*/\1/p')"
if [ -n "$RTT_FRONTIER_MS" ] && [ "$RTT_FRONTIER_MS" -gt "$RTT_GATE_MS" ]; then
  block "network-gate" "frontier RTT ${RTT_FRONTIER_MS}ms exceeds the ${RTT_GATE_MS}ms gate (rtt>${RTT_GATE_MS}ms = NO-GO; a ~20 GB pipeline-parallel decode is not viable over this link — do not inflate the budget)"
fi

# Submit a deterministic prompt and poll the session for the generated
# response + the per-shard signed RunProofs.
GEN="$(eval curl -fsS -X POST "$AUTH" -H "'Content-Type: application/json'" \
  -d "'$(printf '{"session_id":"%s","prompt":"%s"}' "$SHARD_SESSION_ID" "$PROMPT")'" \
  "'$PC_DAEMON/api/daemon/shard-session/$SHARD_SESSION_ID/generate'" 2>/dev/null || true)"
LAST_RESPONSE="$GEN"

DEADLINE=$(( SUBMIT_AT + GATE_TIMEOUT_SECS ))
RESULT_TEXT=""
while :; do
  NOW="$(date +%s)"
  RESP="$(eval curl -fsS "$AUTH" "'$PC_DAEMON/api/daemon/shard-session/$SHARD_SESSION_ID/result'" 2>/dev/null || true)"
  [ -n "$RESP" ] && LAST_RESPONSE="$RESP"
  RESULT_TEXT="$(printf '%s' "$RESP" | sed -n 's/.*"result_text":"\([^"]*\)".*/\1/p')"
  if [ -z "$TTFT_S" ]; then
    TTFT_S="$(printf '%s' "$RESP" | sed -n 's/.*"ttft_s"[[:space:]]*:[[:space:]]*\([0-9]\{1,\}\).*/\1/p')"
  fi
  TOKS_PER_S="$(printf '%s' "$RESP" | sed -n 's/.*"toks_per_s"[[:space:]]*:[[:space:]]*\([0-9]\{1,\}\).*/\1/p')"
  RUN_PROOF="$(printf '%s' "$RESP" | sed -n 's/.*"run_proof"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
  if [ -n "$RESULT_TEXT" ]; then
    break
  fi
  if [ "$NOW" -ge "$DEADLINE" ]; then
    block "generation" "no result within ${GATE_TIMEOUT_SECS}s for session $SHARD_SESSION_ID — the session mounted but the cross-shard decode did not converge (check the Mac M2 shard's forward, the frontier link, and the per-shard RunProof signatures). Last response: ${RESP:-<none>}"
  fi
  sleep "$POLL_SECS"
done

# --- Worker-drop churn: drop the Mac M2 shard mid-flight, expect failover. -
# Only meaningful with a live session. The scheduler's active churn
# (replace_failed_server, Phase E) should re-assign the dropped stage.
log "=== churn (drop the Mac M2 shard, expect active failover) ==="
CHURN="$(eval curl -fsS -X POST "$AUTH" "'$PC_DAEMON/api/daemon/shard-session/$SHARD_SESSION_ID/drop-shard'" 2>/dev/null || true)"
[ -n "$CHURN" ] && LAST_RESPONSE="$CHURN"

# ==========================================================================
# VERDICT — anti-false-green gate: a PASS REQUIRES a non-empty run_proof
# AND a measured toks_per_s >= 1 (plan §14.4). An empty proof or an
# unmeasured rate is a BLOCK to diagnose, never a hollow PASS on a stub.
# ==========================================================================
if [ -z "$RUN_PROOF" ]; then
  block "verdict" "result_text present but run_proof EMPTY: no per-shard signed RunProof was collected — the cross-shard verification (N0-N3) did not bind the generation. Refusing a hollow PASS. Last response: ${LAST_RESPONSE:-<none>}"
fi
case "${TOKS_PER_S:-}" in
  ''|*[!0-9]*) block "verdict" "toks_per_s not measured (got '${TOKS_PER_S:-}') — cannot assert the >=1 tok/s floor. Last response: ${LAST_RESPONSE:-<none>}" ;;
esac
if [ "$TOKS_PER_S" -lt 1 ]; then
  block "verdict" "toks_per_s=${TOKS_PER_S} below the >=1 floor (plan §14.4) — the sharded pipeline is too slow to be viable over this rig/link. NO-GO, do not inflate."
fi

NOTE="model=$MODEL n_shards=$N_SHARDS ttft_s=${TTFT_S:-?} toks_per_s=$TOKS_PER_S rtt_frontier_ms=${RTT_FRONTIER_MS:-?} run_proof=${RUN_PROOF:0:16}… result_text=$RESULT_TEXT"
log "model          : $MODEL ($N_SHARDS shards)"
log "ttft           : ${TTFT_S:-?}s"
log "throughput     : ${TOKS_PER_S} tok/s (floor 1)"
log "frontier RTT   : ${RTT_FRONTIER_MS:-?}ms (gate ${RTT_GATE_MS}ms)"
log "run_proof      : ${RUN_PROOF:0:16}… (per-shard signed, N0-N3 bound)"
log "result_text    : $RESULT_TEXT"
log "A ~20 GB model was sharded across the 5080 + Mac M2 and generated a"
log "verified response over sbfb/shard/1. Paste this trace into sprint77_verification.md."
pass "generation" "$NOTE"
