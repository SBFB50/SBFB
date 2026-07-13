#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 76 (D2/D3 — B-3 palier 1 + redundancy quorum palier 2) —
# cross-machine compute acceptance harness, switched by REDUNDANCY.
# Proves the FIRST compute execution across two OS
# processes on two physical hosts: a VPS coordinator/anchor dispatches a
# task that a PC (RTX 5080, real Ollama) claims and executes on its GPU,
# and the signed `result:` replicates back over the WAN. This is a
# MANUAL acceptance — it needs the operator's real PC + VPS + WAN; it is
# never run in CI. Run it from the PC (the worker side), which must have
# SSH access to the VPS.
#
# === Machine-readable contract (process-evolution Commit 2, T2 gate) ===
# Every exit writes a JSON artefact (default
# `scripts/acceptance/.b3_last_result.json`, override B3_ARTIFACT) AND
# exits with a status-specific code, so the result is never a prose-only
# "DIFFERE-materiel":
#   - PASS       exit 0  — result_text became visible within budget.
#   - BLOCK      exit 1  — task submitted but no result within budget;
#                          the convergence question we actually test.
#                          Auto-diagnosed from the worker log into
#                          "task never reached worker replica" (delivery /
#                          gossip neighborhood) vs "reached but no result"
#                          (claim / inference / result replication).
#   - RIG-ABSENT exit 3  — the rig is not present/functioning (SSH down,
#                          Ollama or MODEL missing, worker binary absent,
#                          or the VPS hosts a DIFFERENT project than
#                          PROJECT_ID — the confounder that produced a
#                          false WAN BLOCK before). NOT a product failure;
#                          the test simply could not run.
# Artefact shape:
#   {"status","stage","delay_s","claim_s","task_id","diagnosis","last_response"}
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
# Two paliers, one harness, switched by the REDUNDANCY env:
#   - Palier 1 (REDUNDANCY=1, default): single worker, no cohort gate.
#     PC<->VPS, proves the first cross-machine compute execution.
#   - Palier 2 (REDUNDANCY=2): the redundancy>1 deterministic quorum
#     over a HOMOGENEOUS cohort (Phase D). Enroll a SECOND worker that is
#     byte-for-byte homogeneous with the first (same MODEL tag + same
#     quant/runtime family — e.g. PC RTX 5080 + Mac, both Ollama
#     llama3.1:8b) BEFORE running. The task is submitted with
#     redundancy_factor=REDUNDANCY AND verifiable=true (deterministic
#     greedy temp=0 + fixed seed — the quorum PREREQUISITE: without it the
#     workers sample and diverge, and the dispatcher skips the cohort gate,
#     so no quorum ever forms). result_text becoming visible means the
#     validator formed a quorum of REDUNDANCY byte-identical results (the
#     `d75ae77` per-worker bridge dedup is the prod prerequisite — before
#     it the 2nd worker's result was dropped). A non-homogeneous 2nd
#     worker is EXPECTED to diverge and never reach quorum (anti
#     false-green): that is the correct negative, not a failure of the
#     harness.
#
# Rig config as DATA (process-evolution Commit 2): instead of a long
# inline env line, drop a gitignored `scripts/acceptance/rig.local.env`
# (override path with RIG_ENV) exporting VPS_SSH / PROJECT_ID / MODEL /
# WORKER_BIN / VPS_DAEMON / REDUNDANCY. It is sourced first; explicit
# env vars still win.
#
# Usage (from the PC):
#   # one-time: scripts/acceptance/rig.local.env (gitignored)
#   #   export VPS_SSH=user@vps.example.net
#   #   export PROJECT_ID=<project-doc-id the VPS hosts>
#   #   export MODEL=llama3.1:8b
#   #   export WORKER_BIN=./target/release/nexus-worker
#   bash scripts/acceptance/b3_live_pc_vps.sh            # palier 1
#   REDUNDANCY=2 bash scripts/acceptance/b3_live_pc_vps.sh  # palier 2
#
# Required (env or rig.local.env):
#   VPS_SSH      ssh destination of the VPS coordinator/anchor.
#   PROJECT_ID   the project-doc id the VPS hosts (RECONCILED below
#                against GET /api/daemon/project-info — a mismatch is
#                RIG-ABSENT, eliminating the wrong-project confounder).
# Optional:
#   VPS_DAEMON   VPS daemon loopback URL as seen over SSH (default
#                http://127.0.0.1:7777).
#   MODEL        Ollama model tag the PC has pulled (default llama3.1:8b).
#   PROMPT       task prompt (default a short deterministic question).
#   WORKER_BIN   path to the local nexus-worker binary. If set, the
#                script enrolls + starts it AND captures its log for the
#                BLOCK auto-diagnostic; otherwise it prints the invite and
#                waits for you to enroll the worker yourself (no log →
#                degraded diagnostic). For palier 2, this enrolls ONE
#                worker (the PC); enroll the 2nd homogeneous worker
#                yourself on its host first.
#   REDUNDANCY   redundancy_factor of the submitted task (default 1).
#   BOOT_AFTER_SUBMIT  Sprint 82 Phase A boot-SEED re-jeu. 1 = submit the task
#                BEFORE booting the worker, so the cold-booting worker must
#                catch up a `task:` already pending in the doc (the
#                S81-G-ESC-1 escalation scenario); the measured submit->visible
#                delay then includes the worker's cold boot. Default 0 = boot
#                the cold worker first, then submit an incremental `task:`.
#   GATE_TIMEOUT_SECS  convergence budget in seconds (default 30).
#   POLL_SECS    result poll interval (default 2).
#   B3_ARTIFACT  JSON artefact path (default scripts/acceptance/.b3_last_result.json).
#   WORKER_LOG   worker log path (default scripts/acceptance/.b3_worker.log).
#   RIG_ENV      rig config path (default scripts/acceptance/rig.local.env).
#
# The full trace this prints is what gets consigned in
# `.planning/active/sprint{N}_verification.md` at Phase G; the JSON
# artefact is its machine-readable companion for the T2 testability gate.

set -uo pipefail

# Resolve the script's own dir so the rig config + artefact + worker log
# land next to it regardless of the caller's cwd (the JSON artefact is the
# T2 gate's source of truth — it must never be silently lost to a wrong cwd).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# --- 0. Rig config as data ------------------------------------------------
RIG_ENV="${RIG_ENV:-$SCRIPT_DIR/rig.local.env}"
if [ -f "$RIG_ENV" ]; then
  # shellcheck disable=SC1090
  . "$RIG_ENV"
fi

VPS_SSH="${VPS_SSH:-}"
PROJECT_ID="${PROJECT_ID:-}"
VPS_DAEMON="${VPS_DAEMON:-http://127.0.0.1:7777}"
MODEL="${MODEL:-llama3.1:8b}"
PROMPT="${PROMPT:-In one word, what is the capital of France?}"
WORKER_BIN="${WORKER_BIN:-}"
REDUNDANCY="${REDUNDANCY:-1}"
# Sprint 82 Phase A (boot-SEED re-jeu): when 1, submit the task BEFORE the
# worker boots, so the cold-booting worker must catch up a `task:` that was
# already pending in the doc (the S81-G-ESC-1 escalation scenario). Default 0
# keeps the audit re-jeu: cold-boot the worker, then submit an incremental
# `task:` ~3s later (worker up + subscribed but its gossip neighbor not yet
# formed — the exact S81-K observation).
BOOT_AFTER_SUBMIT="${BOOT_AFTER_SUBMIT:-0}"
GATE_TIMEOUT_SECS="${GATE_TIMEOUT_SECS:-30}"
POLL_SECS="${POLL_SECS:-2}"
OLLAMA_URL="${OLLAMA_URL:-http://127.0.0.1:11434}"
B3_ARTIFACT="${B3_ARTIFACT:-$SCRIPT_DIR/.b3_last_result.json}"
WORKER_LOG="${WORKER_LOG:-$SCRIPT_DIR/.b3_worker.log}"

# Runtime state referenced by the artefact writer (filled as we progress).
TASK_ID=""
DELAY=""
CLAIM_S=""
LAST_RESPONSE=""

log() { printf '[b3] %s\n' "$*"; }

# --- JSON artefact (machine-readable, written on every exit) --------------
# Encoding is done with python3 (json.dumps — bullet-proof for backslashes,
# quotes, and newlines, which `last_response` routinely carries). Pure-bash
# ${//} escaping is NOT used: it silently corrupts backslashes/newlines in
# this (Git Bash) build, producing invalid JSON exactly in the error cases
# the artefact must survive. A no-python3 fallback DELETES the unsafe chars
# (backslash, quote, control) so the artefact is always valid JSON, lossy.
_json_safe() {
  printf '%s' "$1" | tr -d '\\"' | tr -d '\000-\037'
}

emit_artifact() {
  # $1=status $2=stage $3=diagnosis
  local status="$1" stage="$2" diag="$3"
  local delay="${DELAY:-}" claim="${CLAIM_S:-}"
  [ -z "$delay" ] && delay="null"
  [ -z "$claim" ] && claim="null"
  mkdir -p "$(dirname "$B3_ARTIFACT")" 2>/dev/null || true
  if command -v python3 >/dev/null 2>&1; then
    B3_STATUS="$status" B3_STAGE="$stage" B3_DELAY="$delay" B3_CLAIM="$claim" \
    B3_TID="${TASK_ID:-}" B3_DIAG="$diag" B3_RESP="${LAST_RESPONSE:-}" \
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
    "delay_s": num(os.environ["B3_DELAY"]),
    "claim_s": num(os.environ["B3_CLAIM"]),
    "task_id": os.environ["B3_TID"],
    "diagnosis": os.environ["B3_DIAG"],
    "last_response": os.environ["B3_RESP"],
}))' >"$B3_ARTIFACT"
  else
    local tid_e diag_e resp_e
    tid_e="$(_json_safe "${TASK_ID:-}")"
    diag_e="$(_json_safe "$diag")"
    resp_e="$(_json_safe "${LAST_RESPONSE:-}")"
    printf '{"status":"%s","stage":"%s","delay_s":%s,"claim_s":%s,"task_id":"%s","diagnosis":"%s","last_response":"%s"}\n' \
      "$status" "$stage" "$delay" "$claim" "$tid_e" "$diag_e" "$resp_e" \
      >"$B3_ARTIFACT"
  fi
  # Echo the artefact to the trace too.
  cat "$B3_ARTIFACT" 2>/dev/null || true
}

rig_absent() {
  # $1=reason. The rig is not present/functioning — exit 3, no product claim.
  emit_artifact "RIG-ABSENT" "preflight" "$1"
  printf '[b3][RIG-ABSENT] %s\n' "$1" >&2
  exit 3
}

block() {
  # $1=stage $2=diagnosis. Submitted but no convergence — exit 1.
  emit_artifact "BLOCK" "$1" "$2"
  printf '[b3][BLOCK] %s\n' "$2" >&2
  exit 1
}

pass() {
  # $1=stage $2=note
  emit_artifact "PASS" "$1" "$2"
  printf '[b3][PASS] %s\n' "$2"
  exit 0
}

# --- Validate REDUNDANCY (same semantics as before) -----------------------
# Palier 2 (>=2) demands a DETERMINISTIC task so the homogeneous workers
# converge byte-for-byte: `verifiable` flips inference to greedy temp=0 +
# fixed seed (task.rs / runtime.rs); without it the workers sample and
# DIVERGE, so the hash-exact quorum never forms. So REDUNDANCY>=2 submits
# `"verifiable":true`.
#
# SCOPE NOTE: this harness proves the deterministic QUORUM cross-machine via
# verifiable + OPERATOR-ensured homogeneity (you enroll 2 byte-homogeneous
# workers). It deliberately does NOT submit `required_runtime`, so it does
# not exercise the dispatcher's AUTO claim-gate (covered by the in-process
# unit tests `cohort_gate_admits_homogeneous_worker`/`cohort_gate_blocks_non_homogeneous_worker`, Phase C); submitting a
# tuple here would only add tuple-mismatch fragility to a manual run.
case "$REDUNDANCY" in
  ''|*[!0-9]*) rig_absent "REDUNDANCY must be a positive integer, got '$REDUNDANCY'" ;;
esac
REDUNDANCY=$((10#$REDUNDANCY))  # canonicalize (strip leading zeros; base-10)
[ "$REDUNDANCY" -ge 1 ] || rig_absent "REDUNDANCY must be >= 1, got '$REDUNDANCY'"
if [ "$REDUNDANCY" -ge 2 ]; then VERIFIABLE=true; else VERIFIABLE=false; fi

vps() { ssh "$VPS_SSH" "$@"; }

# ==========================================================================
# PREFLIGHT — any failure here is RIG-ABSENT (exit 3), never a product BLOCK.
# ==========================================================================
log "=== preflight (rig presence + project reconciliation) ==="

[ -n "$VPS_SSH" ]    || rig_absent "VPS_SSH unset (set it in env or $RIG_ENV)"
[ -n "$PROJECT_ID" ] || rig_absent "PROJECT_ID unset (set it in env or $RIG_ENV)"
command -v ssh >/dev/null 2>&1 || rig_absent "ssh not found on PATH"

# SSH reachability.
if ! ssh -o ConnectTimeout=10 -o BatchMode=yes "$VPS_SSH" true >/dev/null 2>&1; then
  rig_absent "SSH to '$VPS_SSH' failed (host down, auth, or network)"
fi

# Local Ollama + MODEL pulled (this host is the worker side).
TAGS="$(curl -fsS "$OLLAMA_URL/api/tags" 2>/dev/null || true)"
[ -n "$TAGS" ] || rig_absent "Ollama not reachable at $OLLAMA_URL (start it on the worker host)"
# Match the model tag (with or without an explicit :tag suffix).
MODEL_BASE="${MODEL%%:*}"
if ! printf '%s' "$TAGS" | grep -q "$MODEL_BASE"; then
  rig_absent "model '$MODEL' not pulled on this worker (ollama pull $MODEL)"
fi

# Worker binary present (when the script is expected to enroll it).
if [ -n "$WORKER_BIN" ] && [ ! -x "$WORKER_BIN" ]; then
  rig_absent "WORKER_BIN '$WORKER_BIN' is not an executable nexus-worker binary"
fi

# VPS loopback token (public /auth/token).
TOKEN="$(vps "curl -fsS '$VPS_DAEMON/auth/token'" 2>/dev/null | sed -n 's/.*\"token\":\"\([^\"]*\)\".*/\1/p')"
[ -n "$TOKEN" ] || rig_absent "could not read loopback token from $VPS_DAEMON/auth/token on the VPS"
AUTH="-H 'x-sbfb-token: $TOKEN'"

# Project reconciliation: the VPS must actually host PROJECT_ID. A
# mismatch is the confounder that produced a false WAN BLOCK before
# (harness submitted under sbfb-explorer != project_doc.id()).
PI="$(vps "curl -fsS $AUTH '$VPS_DAEMON/api/daemon/project-info'" 2>/dev/null || true)"
LAST_RESPONSE="$PI"
VPS_PID="$(printf '%s' "$PI" | sed -n 's/.*"project_doc_id"[[:space:]]*:[[:space:]]*"\{0,1\}\([^",}]*\)"\{0,1\}.*/\1/p')"
if [ -z "$VPS_PID" ] || [ "$VPS_PID" = "null" ]; then
  rig_absent "VPS has no project doc mounted (GET /api/daemon/project-info -> $PI)"
fi
if [ "$VPS_PID" != "$PROJECT_ID" ]; then
  rig_absent "project mismatch: VPS hosts '$VPS_PID' but PROJECT_ID='$PROJECT_ID' (fix PROJECT_ID to the VPS project_doc id)"
fi
LAST_RESPONSE=""
log "preflight OK — VPS reachable, Ollama+$MODEL present, project reconciled ($VPS_PID)"

# ==========================================================================
# RUN — set up worker, submit, poll. Setup failures are RIG-ABSENT; only a
# post-submit timeout is a product BLOCK.
# ==========================================================================
if [ "$REDUNDANCY" -ge 2 ]; then
  PALIER="palier 2 — redundancy=$REDUNDANCY deterministic quorum (homogeneous cohort)"
else
  PALIER="palier 1 — redundancy=1 single worker"
fi
log "=== Sprint 76 — B-3 cross-machine compute acceptance ($PALIER) ==="
log "VPS coordinator : $VPS_SSH ($VPS_DAEMON)"
log "PC worker model : $MODEL"
log "project         : $PROJECT_ID"
log "redundancy      : $REDUNDANCY"
log "convergence gate: < ${GATE_TIMEOUT_SECS}s (else BLOCK, do not extend)"
log "artefact        : $B3_ARTIFACT"
if [ "$REDUNDANCY" -ge 2 ]; then
  log "palier 2 NOTE   : task submitted verifiable=true so honest workers CAN"
  log "                  converge. Enroll a 2nd worker HOMOGENEOUS with the PC"
  log "                  (same MODEL '$MODEL' + same quant/runtime) on another"
  log "                  host BEFORE this run, else the cohort diverges and the"
  log "                  poll BLOCKs on no quorum — diagnose, do not inflate."
fi

# --- Mint a worker-scope invite on the VPS --------------------------------
log "step 1: minting a worker-scope invite on the VPS"
INVITE_JSON="$(vps "curl -fsS -X POST $AUTH -H 'Content-Type: application/json' \
  -d '{\"scope\":\"worker\"}' '$VPS_DAEMON/api/v1/invite/create'" 2>/dev/null || true)"
INVITE="$(printf '%s' "$INVITE_JSON" | sed -n 's/.*\"wire\":\"\([^\"]*\)\".*/\1/p' | head -n1)"
[ -n "$INVITE" ] || rig_absent "invite/create returned no token: ${INVITE_JSON:-<none>}"
log "worker invite minted: ${INVITE:0:16}…"

# --- Enroll + start the local nexus-worker (capture log) ------------------
# Extracted into a function (Sprint 82 Phase A) so BOOT_AFTER_SUBMIT can run it
# AFTER the submit. Variables are the enclosing script's globals (bash
# functions share scope) — WORKER_PID / HAVE_WORKER_LOG / the EXIT trap all
# take effect globally.
WORKER_PID=""
HAVE_WORKER_LOG=0
enroll_and_start_worker() {
  if [ -n "$WORKER_BIN" ]; then
    log "step (worker): enrolling local worker via 'nexus-worker join'"
    if ! "$WORKER_BIN" join "$INVITE"; then
      rig_absent "nexus-worker join failed (invite rejected or worker misconfigured)"
    fi
    log "step (worker): starting local nexus-worker (real Ollama, GPU); log -> $WORKER_LOG"
    : >"$WORKER_LOG" 2>/dev/null || true
    # RUST_LOG raises the engine to debug so the worker logs the task_id at
    # SCAN/CLAIM (not only at completion) — the BLOCK auto-diagnostic and the
    # claim timer both grep this log for the task_id, so without it an
    # in-flight inference would be mis-diagnosed as "never reached worker".
    RUST_LOG="${RUST_LOG:-info,nexus_worker_core::engine=debug}" \
      "$WORKER_BIN" start --headless >"$WORKER_LOG" 2>&1 &
    WORKER_PID=$!
    HAVE_WORKER_LOG=1
    trap '[ -n "$WORKER_PID" ] && kill "$WORKER_PID" 2>/dev/null || true' EXIT
    sleep 3
    # A worker that died on start (bad config, port taken) must be RIG-ABSENT,
    # not a downstream BLOCK misattributed to WAN convergence.
    if ! kill -0 "$WORKER_PID" 2>/dev/null; then
      rig_absent "local nexus-worker died on start (see $WORKER_LOG)"
    fi
  else
    log "step (worker): WORKER_BIN unset — enroll the PC worker yourself, then press Enter:"
    log "        nexus-worker join $INVITE && nexus-worker start --headless"
    log "        (no worker log captured → BLOCK auto-diagnostic is degraded)"
    read -r _
  fi
}

# --- Submit a task to the VPS coordinator ---------------------------------
submit_task() {
  log "step (submit): submitting a task to the VPS (redundancy=$REDUNDANCY, verifiable=$VERIFIABLE)"
  SUBMIT_JSON="$(vps "curl -fsS -X POST $AUTH -H 'Content-Type: application/json' -d '$(
    printf '{\"project_id\":\"%s\",\"task_type\":\"analysis\",\"prompt\":\"%s\",\"model\":\"%s\",\"redundancy_factor\":%s,\"verifiable\":%s}' \
      "$PROJECT_ID" "$PROMPT" "$MODEL" "$REDUNDANCY" "$VERIFIABLE"
  )' '$VPS_DAEMON/api/v1/tasks/submit'" 2>/dev/null || true)"
  TASK_ID="$(printf '%s' "$SUBMIT_JSON" | sed -n 's/.*\"task_id\":\"\([^\"]*\)\".*/\1/p')"
  [ -n "$TASK_ID" ] || rig_absent "tasks/submit returned no task_id: ${SUBMIT_JSON:-<none>}"
  SUBMIT_AT="$(date +%s)"
  log "task submitted: $TASK_ID at epoch ${SUBMIT_AT}s"
}

# Order the two steps. Sprint 82 Phase A boot-SEED re-jeu (BOOT_AFTER_SUBMIT=1):
# submit FIRST so the `task:` is already pending in the doc, THEN cold-boot the
# worker — it must reconcile the pending entry once its gossip neighbor forms
# (the cold-boot catch-up the WORKER deliverable targets). Default: boot the
# cold worker first, then submit an incremental `task:` (the S81-K observation
# — worker up + subscribed but neighbor not yet formed).
if [ "$BOOT_AFTER_SUBMIT" = "1" ]; then
  # Codex P1-3: attributing PASS to the cold target worker needs its captured
  # log — fail fast before submitting if we cannot capture one.
  [ -n "$WORKER_BIN" ] || rig_absent "BOOT_AFTER_SUBMIT needs WORKER_BIN so the cold target worker's log can attribute the result (a competing worker would otherwise false-green the run)"
  log "BOOT_AFTER_SUBMIT=1: submit first, then cold-boot the worker (pending-task catch-up)"
  submit_task
  enroll_and_start_worker
else
  enroll_and_start_worker
  submit_task
fi

# --- Poll the VPS for the WAN-replicated result + measure timers ----------
log "step 4: polling $VPS_DAEMON/api/v1/tasks/$TASK_ID/result for the WAN result"
DEADLINE=$(( SUBMIT_AT + GATE_TIMEOUT_SECS ))
RESULT_TEXT=""
CLAIM_AT=""
while :; do
  NOW="$(date +%s)"

  # Sub-stage timer: first time the worker replica is seen to know the
  # task (claim signal from its own log), record claim latency.
  if [ -z "$CLAIM_AT" ] && [ "$HAVE_WORKER_LOG" -eq 1 ] \
     && grep -q "$TASK_ID" "$WORKER_LOG" 2>/dev/null; then
    CLAIM_AT="$NOW"
    CLAIM_S=$(( CLAIM_AT - SUBMIT_AT ))
    log "claim: worker replica saw task:$TASK_ID after ${CLAIM_S}s"
  fi

  RESP="$(vps "curl -fsS $AUTH '$VPS_DAEMON/api/v1/tasks/$TASK_ID/result'" 2>/dev/null || true)"
  LAST_RESPONSE="$RESP"
  RESULT_TEXT="$(printf '%s' "$RESP" | sed -n 's/.*\"result_text\":\"\([^\"]*\)\".*/\1/p')"
  if [ -n "$RESULT_TEXT" ]; then
    DELAY=$(( NOW - SUBMIT_AT ))
    break
  fi
  if [ "$NOW" -ge "$DEADLINE" ]; then
    DELAY=$(( NOW - SUBMIT_AT ))
    # Auto-diagnose the BLOCK from the worker log.
    if [ "$HAVE_WORKER_LOG" -eq 1 ] && [ -f "$WORKER_LOG" ]; then
      if grep -q "$TASK_ID" "$WORKER_LOG" 2>/dev/null; then
        block "result-replication" \
          "reached but no result: the worker replica saw task:$TASK_ID (grep hit in $WORKER_LOG) but produced no visible result_text within ${GATE_TIMEOUT_SECS}s — claim/inference/result-replication failure (check Ollama, model, GPU, signature verify). Last VPS response: ${RESP:-<none>}"
      else
        block "claim" \
          "task never reached worker replica: task:$TASK_ID is ABSENT from the worker log ($WORKER_LOG) — the incremental task: entry did not propagate VPS->PC (WAN delivery / gossip neighborhood not formed; cf. S75 SeedAnnounced peer_count:0, S77 Phase A convergence prerequisite). Last VPS response: ${RESP:-<none>}"
      fi
    else
      block "result-replication" \
        "result not visible at the VPS within ${GATE_TIMEOUT_SECS}s; no worker log captured (manual worker) — inspect the worker replica for task:$TASK_ID. Last VPS response: ${RESP:-<none>}"
    fi
  fi
  sleep "$POLL_SECS"
done

# --- Verdict --------------------------------------------------------------
NOTE="task_id=$TASK_ID result_text=$RESULT_TEXT submit->visible=${DELAY}s (budget ${GATE_TIMEOUT_SECS}s)"
log "task_id        : $TASK_ID"
log "result_text    : $RESULT_TEXT"
log "submit->visible: ${DELAY}s end-to-end (budget ${GATE_TIMEOUT_SECS}s; upper bound on result: WAN convergence)"
[ -n "$CLAIM_S" ] && log "claim latency  : ${CLAIM_S}s (inference+replication ~ $(( DELAY - CLAIM_S ))s)"
if [ "$REDUNDANCY" -ge 2 ]; then
  log "quorum         : redundancy=$REDUNDANCY — result_text became visible only"
  log "                 after the validator agreed $REDUNDANCY byte-identical"
  log "                 results from the homogeneous cohort (a diverging or"
  log "                 non-homogeneous worker would never have reached quorum)."
fi
# Sprint 82 Phase A (Codex P1-3): in BOOT_AFTER_SUBMIT mode the whole point is
# that the COLD target worker caught up the already-pending task. A visible
# result is NOT proof of that — a warm Mac or another enrolled worker could have
# produced it first. Require the target worker's OWN log to show it claimed
# task:$TASK_ID before attributing PASS; without a captured log (no WORKER_BIN)
# the result cannot be attributed, so the run is RIG-ABSENT.
if [ "$BOOT_AFTER_SUBMIT" = "1" ]; then
  if [ "$HAVE_WORKER_LOG" -ne 1 ]; then
    rig_absent "BOOT_AFTER_SUBMIT needs WORKER_BIN so the cold target worker's log can attribute the result (a competing worker would otherwise false-green the run)"
  fi
  # Attribution marker (Codex round 2): the worker logs "task completed and
  # result written" with the task_id ONLY after it signed + wrote the result
  # (nexus-worker-core engine/runtime.rs). A "saw task" SCAN/CLAIM log is NOT
  # enough — the cold target can see the task, log its id, then LOSE the claim
  # to a warm competing worker that actually produces the result.
  if ! grep "$TASK_ID" "$WORKER_LOG" 2>/dev/null | grep -qi "result written"; then
    block "attribution" \
      "result visible at the VPS but the cold target worker never logged 'task completed and result written' for task:$TASK_ID ($WORKER_LOG) — it may have SEEN the task but lost the claim to a competing worker (e.g. a warm Mac), so this does NOT prove the cold-boot catch-up. Run ONLY the cold target worker, or attribute the producing key. Last VPS response: ${RESP:-<none>}"
  fi
  log "attribution    : the cold target worker WROTE the result for task:$TASK_ID (claim seen at ${CLAIM_S:-?}s) — result is attributable to it"
fi
log "The PC executed a task submitted to the VPS; the signed result was"
log "rendered back over the WAN. Paste this trace into sprint{N}_verification.md."
pass "result-replication" "$NOTE"
