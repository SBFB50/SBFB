#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 76 Phase C (D2 / B-3 palier 1) — cross-machine compute
# acceptance harness. Proves the FIRST compute execution across two OS
# processes on two physical hosts: a VPS coordinator/anchor dispatches a
# task that a PC (RTX 5080, real Ollama) claims and executes on its GPU,
# and the signed `result:` replicates back over the WAN. This is a
# MANUAL acceptance — it needs the operator's real PC + VPS + WAN; it is
# never run in CI. Run it from the PC (the worker side), which must have
# SSH access to the VPS.
#
# Falsifiable acceptance criterion (plan §C, D2 adjust): the delay from
# task SUBMIT (at the VPS) to the result_text becoming visible at the
# VPS is MEASURED and must be < GATE_TIMEOUT_SECS (default 30s =
# 150 * 200ms, the in-process gate budget). This is an END-TO-END
# delay (claim + GPU inference + `result:` WAN replication), i.e. an
# UPPER BOUND on the `result:` PC->VPS convergence the plan targets:
# for the tiny deterministic prompt below, claim + inference is a few
# seconds, so a delay near or beyond the budget implicates the WAN
# convergence (cf. the Sprint 75 observation `SeedAnnounced
# peer_count:0 ~10 min`). Exceeding the budget is a BLOCK to DIAGNOSE
# (root-cause — is it inference or replication?), NOT a timeout to
# inflate.
#
# Palier 1 uses redundancy=1 (single worker, no cohort gate). The
# `required_runtime` cohort gate and the redundancy>1 deterministic
# quorum are exercised by Phase D (palier 2), which reuses this harness.
#
# Usage (from the PC):
#   VPS_SSH=user@vps.example.net \
#   VPS_DAEMON=http://127.0.0.1:7777 \
#   PROJECT_ID=<project-id-known-to-the-vps> \
#   MODEL=llama3.1:8b \
#   WORKER_BIN=./target/release/nexus-worker \
#     bash scripts/acceptance/b3_live_pc_vps.sh
#
# Required env:
#   VPS_SSH      ssh destination of the VPS coordinator/anchor.
#   PROJECT_ID   a project the VPS already hosts (the task is submitted
#                under it; the PC enrolls into it via a worker invite).
# Optional env:
#   VPS_DAEMON   VPS daemon loopback URL as seen over SSH (default
#                http://127.0.0.1:7777).
#   MODEL        Ollama model tag the PC has pulled (default llama3.1:8b).
#   PROMPT       task prompt (default a short deterministic question).
#   WORKER_BIN   path to the local nexus-worker binary. If set, the
#                script enrolls + starts it; otherwise it prints the
#                invite and waits for you to enroll the worker yourself.
#   GATE_TIMEOUT_SECS  convergence budget in seconds (default 30).
#   POLL_SECS    result poll interval (default 2).
#
# The full trace this prints is what gets consigned in
# `.planning/active/sprint76_verification.md` at Phase G.

set -euo pipefail

VPS_SSH="${VPS_SSH:?set VPS_SSH=user@host (the VPS coordinator/anchor)}"
PROJECT_ID="${PROJECT_ID:?set PROJECT_ID to a project the VPS hosts}"
VPS_DAEMON="${VPS_DAEMON:-http://127.0.0.1:7777}"
MODEL="${MODEL:-llama3.1:8b}"
PROMPT="${PROMPT:-In one word, what is the capital of France?}"
WORKER_BIN="${WORKER_BIN:-}"
GATE_TIMEOUT_SECS="${GATE_TIMEOUT_SECS:-30}"
POLL_SECS="${POLL_SECS:-2}"

log() { printf '[b3] %s\n' "$*"; }
die() { printf '[b3][FATAL] %s\n' "$*" >&2; exit 1; }

# Run a curl against the VPS daemon loopback through SSH. The daemon's
# loopback routes are bearer + Host + Origin gated; we fetch the token
# over the same SSH session so it never leaves the VPS loopback.
vps() { ssh "$VPS_SSH" "$@"; }

command -v ssh >/dev/null || die "ssh not found on PATH"

log "=== Sprint 76 Phase C — B-3 cross-machine compute acceptance (palier 1) ==="
log "VPS coordinator : $VPS_SSH ($VPS_DAEMON)"
log "PC worker model : $MODEL"
log "project         : $PROJECT_ID"
log "convergence gate: < ${GATE_TIMEOUT_SECS}s (else BLOCK, do not extend)"

# --- 1. Bearer token (VPS loopback, public /auth/token) ------------------
log "step 1: fetching VPS daemon bearer token over SSH"
TOKEN="$(vps "curl -fsS '$VPS_DAEMON/auth/token'" | sed -n 's/.*\"token\":\"\([^\"]*\)\".*/\1/p')"
[ -n "$TOKEN" ] || die "could not read bearer token from $VPS_DAEMON/auth/token on the VPS"
AUTH="-H 'Authorization: Bearer $TOKEN'"

# --- 2. Mint a worker-scope invite on the VPS ----------------------------
log "step 2: minting a worker-scope invite on the VPS"
INVITE_JSON="$(vps "curl -fsS -X POST $AUTH -H 'Content-Type: application/json' \
  -d '{\"scope\":\"worker\"}' '$VPS_DAEMON/api/v1/invite/create'")"
# create_invite returns the encoded token under the "wire" key
# (invite_api.rs: {"id","wire","scope",...}).
INVITE="$(printf '%s' "$INVITE_JSON" | sed -n 's/.*\"wire\":\"\([^\"]*\)\".*/\1/p' | head -n1)"
[ -n "$INVITE" ] || die "invite/create returned no token: $INVITE_JSON"
log "worker invite minted: ${INVITE:0:16}…"

# --- 3. Enroll + start the local nexus-worker ----------------------------
WORKER_PID=""
if [ -n "$WORKER_BIN" ]; then
  [ -x "$WORKER_BIN" ] || die "WORKER_BIN '$WORKER_BIN' is not executable"
  log "step 3: enrolling local worker via 'nexus-worker join'"
  "$WORKER_BIN" join "$INVITE"
  log "step 3: starting local nexus-worker (real Ollama, GPU) in the background"
  "$WORKER_BIN" start --headless &
  WORKER_PID=$!
  trap '[ -n "$WORKER_PID" ] && kill "$WORKER_PID" 2>/dev/null || true' EXIT
  sleep 3
else
  log "step 3: WORKER_BIN unset — enroll the PC worker yourself, then press Enter:"
  log "        nexus-worker join $INVITE && nexus-worker start --headless"
  read -r _
fi

# --- 4. Submit a task to the VPS coordinator -----------------------------
log "step 4: submitting a task to the VPS (redundancy=1)"
SUBMIT_JSON="$(vps "curl -fsS -X POST $AUTH -H 'Content-Type: application/json' -d '$(
  printf '{\"project_id\":\"%s\",\"task_type\":\"analysis\",\"prompt\":\"%s\",\"model\":\"%s\",\"redundancy_factor\":1}' \
    "$PROJECT_ID" "$PROMPT" "$MODEL"
)' '$VPS_DAEMON/api/v1/tasks/submit'")"
TASK_ID="$(printf '%s' "$SUBMIT_JSON" | sed -n 's/.*\"task_id\":\"\([^\"]*\)\".*/\1/p')"
[ -n "$TASK_ID" ] || die "tasks/submit returned no task_id: $SUBMIT_JSON"
SUBMIT_AT="$(date +%s)"
log "task submitted: $TASK_ID at epoch ${SUBMIT_AT}s"

# --- 5. Poll the VPS for the WAN-replicated result + measure delay -------
# DELAY below is the end-to-end submit->result-visible time (an upper
# bound on the `result:` PC->VPS WAN convergence — see the header note).
log "step 5: polling $VPS_DAEMON/api/v1/tasks/$TASK_ID/result for the WAN result"
DEADLINE=$(( SUBMIT_AT + GATE_TIMEOUT_SECS ))
RESULT_TEXT=""
while :; do
  NOW="$(date +%s)"
  RESP="$(vps "curl -fsS $AUTH '$VPS_DAEMON/api/v1/tasks/$TASK_ID/result'" 2>/dev/null || true)"
  RESULT_TEXT="$(printf '%s' "$RESP" | sed -n 's/.*\"result_text\":\"\([^\"]*\)\".*/\1/p')"
  if [ -n "$RESULT_TEXT" ]; then
    DELAY=$(( NOW - SUBMIT_AT ))
    break
  fi
  if [ "$NOW" -ge "$DEADLINE" ]; then
    log "=== BLOCK ==="
    die "result not visible at the VPS within ${GATE_TIMEOUT_SECS}s. This is a BLOCK to \
DIAGNOSE the WAN convergence root-cause (cf. S75 SeedAnnounced peer_count:0), NOT a \
timeout to inflate. Last response: ${RESP:-<none>}"
  fi
  sleep "$POLL_SECS"
done

# --- 6. Verdict ----------------------------------------------------------
log "=== PASS ==="
log "task_id        : $TASK_ID"
log "result_text    : $RESULT_TEXT"
log "submit->visible: ${DELAY}s end-to-end (budget ${GATE_TIMEOUT_SECS}s; upper bound on result: WAN convergence)"
log "The PC executed a task submitted to the VPS; the signed result was"
log "rendered back over the WAN. Paste this trace into sprint76_verification.md."
