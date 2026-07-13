#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 82 Phase B — standard sharding benchmarks (T3 benchmark tier).
#
# Measures the sharding pipeline against RECOGNISED tools and vocabulary so
# future optimisations (F2 KV-reuse, quantisation, topology A/B) can be
# decided with versioned baselines instead of a lone "~2 tok/s vs a predicted
# window" claim (memory `po_benchmarks_standards_llm_sharding`):
#
#   1. llama-bench (llama.cpp OFFICIAL) — per-machine single-machine
#      baselines: pp512 (prompt-processing tok/s, compute-bound) + tg128
#      (text-generation tok/s, memory/latency-bound). The HONEST denominator
#      of the sharded HUB rate.
#   2. perplexity (llama.cpp OFFICIAL) on wikitext-2 — PPL(whole model) as the
#      parity reference. NOTE: PPL(sharded) is NOT wired in Phase B (no /result
#      producer emits it), so the parity is limited to ppl_whole; the tail-side
#      scalar design (never a cross-machine logprob route, guard S3) is future
#      work documented in docs/protocol/SHARD_BENCHMARKS.md.
#   3. The sharded pipeline's fine metrics — TTFT(ms) / TPOT / ITL p50/p95 /
#      milli-tokens/sec — read from a VALIDATED `b3_shard_pipeline.sh` artefact
#      (Sprint 82 Phase B host-side instrumentation, /result view). These are
#      REQUIRED for a PASS (a benchmark of the sharding must measure it).
#
# === PLAN-ADAPT (preflight G8, sprint82_phase_b_preflight.md) ===
# llama-bench + perplexity are NOT in the vendored fork
# (`vendor/llama-cpp-sys-2/build.rs` sets LLAMA_BUILD_TOOLS=OFF, tools/=mtmd
# only). They are BUILD-TOOLS built SEPARATELY from an UPSTREAM llama.cpp
# checkout PINNED to the same commit the shard backend bundles (provenance:
# THIRD-PARTY-NOTICES.md -> utilityai/llama-cpp-rs), with the SAME backend
# (CUDA on the 5080, Metal on the M2). That is 0 Cargo dep / 0 churn
# Cargo.lock (CI never builds them); pass their paths as data below.
#
# === Machine-readable contract (T3 tier, README §4) ===
# Writes a VERSIONED JSON artefact (default
# `.planning/active/sprint82_t2_benchmarks.json`, override BENCH_ARTIFACT)
# and exits with a status-specific code — never a prose-only claim:
#   - PASS       exit 0  — the SHARDING was really measured with comparable
#                          provenance: single-machine llama-bench pp/tg AND a
#                          VALIDATED current sharded metrics set (from a
#                          status=PASS, model-matched b3 artefact) AND the
#                          provenance pins (LLAMACPP_COMMIT set, model blake3
#                          available). A single-machine baseline ALONE is NOT a
#                          PASS. (ppl_whole is recorded when available but does
#                          not gate — the sharded PPL parity is unwired, note 2.)
#   - BLOCK      exit 1  — the rig is engaged (Phase A) but a required input
#                          is cold/missing: llama-bench binary or GGUF absent,
#                          no valid sharded metrics (session not mounted/driven,
#                          or a stale/mismatched b3 artefact), or provenance
#                          pins unset. `BLOCK{rig}`, the honest cold-rig verdict
#                          — NEVER `RIG-ABSENT` (rig engaged for Phase A boot-SEED).
# A write failure or an unassemblable artefact is FATAL (exit 2), never a
# hollow exit-0. Determinism: the artefact pins model NAME + blake3, quant,
# split, the llama.cpp commit, the wikitext-2 corpus hash (when perplexity
# runs), seed and thread count — so two runs are comparable and a future run
# detects a regression; a PASS is refused unless the load-bearing pins are set.
#
# HYGIENE (preflight S3 caution, confirmed on disk): the artefact carries the
# model NAME + blake3, NEVER a filesystem path (a committed `.b3_shard` sample
# leaked `C:/Users/<user>/spike_fork/...`). `redact_model` strips any path to
# its basename; `assert_no_fs_path` fails the run if a path still slips in.
#
# Usage (from the RTX 5080 head; rig config as DATA in rig.local.env):
#   export LLAMA_BENCH_BIN=/path/to/upstream/llama.cpp/build/bin/llama-bench
#   export PERPLEXITY_BIN=/path/to/upstream/llama.cpp/build/bin/perplexity
#   export WIKITEXT2_PATH=/path/to/wikitext-2-raw/wiki.test.raw
#   export MODEL_20GB=/path/to/codellama-34b.gguf   # (from b3 rig.local.env)
#   export LLAMACPP_COMMIT=<sha the shard backend bundles>
#   # optional 2nd machine (Metal): MAC_SSH + MAC_LLAMA_BENCH_BIN + MAC_MODEL
#   bash scripts/acceptance/benchmarks_standards.sh

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# --- 0. Rig config as data (shared with b3_shard_pipeline.sh) --------------
RIG_ENV="${RIG_ENV:-$SCRIPT_DIR/rig.local.env}"
if [ -f "$RIG_ENV" ]; then
  # shellcheck disable=SC1090
  . "$RIG_ENV"
fi

LLAMA_BENCH_BIN="${LLAMA_BENCH_BIN:-}"
PERPLEXITY_BIN="${PERPLEXITY_BIN:-}"
WIKITEXT2_PATH="${WIKITEXT2_PATH:-}"
MODEL_20GB="${MODEL_20GB:-}"
LLAMACPP_COMMIT="${LLAMACPP_COMMIT:-unknown}"
QUANT="${QUANT:-unknown}"
N_SHARDS="${N_SHARDS:-2}"
# Determinism knobs (pinned defaults; override in rig.local.env if needed).
BENCH_THREADS="${BENCH_THREADS:-8}"
BENCH_PP="${BENCH_PP:-512}"
BENCH_TG="${BENCH_TG:-128}"
BENCH_REPS="${BENCH_REPS:-3}"
PPL_SEED="${PPL_SEED:-1234}"
# Optional 2nd machine (Metal).
MAC_SSH="${MAC_SSH:-}"
MAC_LLAMA_BENCH_BIN="${MAC_LLAMA_BENCH_BIN:-}"
MAC_MODEL="${MAC_MODEL:-}"
# Shard-path fine metrics: the b3_shard artefact this run reads (already
# produced by a prior/paired b3_shard_pipeline.sh run on the same rig).
B3_ARTIFACT="${B3_ARTIFACT:-$SCRIPT_DIR/.b3_shard_last_result.json}"
# Reject a b3 artefact older than this (minutes) as stale — the sharded run
# that produced the metrics must be from the current operator session.
B3_MAX_AGE_MIN="${B3_MAX_AGE_MIN:-120}"
BENCH_ARTIFACT="${BENCH_ARTIFACT:-$REPO_ROOT/.planning/active/sprint82_t2_benchmarks.json}"
BENCH_SCHEMA_VERSION="1"

log() { printf '[bench] %s\n' "$*"; }

have_py3() { command -v python3 >/dev/null 2>&1; }

# --- model hygiene: NAME + blake3, never a filesystem path -----------------
redact_model() {
  # $1 = a model path or name -> basename only (strip any directory prefix,
  # Windows or POSIX). "C:/Users/x/spike_fork/codellama-34b.gguf" -> "codellama-34b.gguf".
  local m="$1"
  m="${m##*/}"
  m="${m##*\\}"
  printf '%s' "$m"
}

blake3_of() {
  # blake3 of a file for determinism/provenance. ALWAYS returns either a
  # VALIDATED lowercase 64-hex digest or the literal "unavailable" — never an
  # empty or malformed string (Codex P1-B: an empty `b3sum` output must not
  # slip past a `!= "unavailable"` provenance check as a null pin).
  local f="$1" h
  [ -f "$f" ] || { printf 'unavailable'; return; }
  command -v b3sum >/dev/null 2>&1 || { printf 'unavailable'; return; }
  h="$(b3sum "$f" 2>/dev/null | awk '{print $1}')"
  case "$h" in
    *[!0-9a-f]* | '') printf 'unavailable' ;;
    *) if [ "${#h}" -eq 64 ]; then printf '%s' "$h"; else printf 'unavailable'; fi ;;
  esac
}

# Return 0 iff $1 is a valid lowercase blake3 (64 hex) digest.
is_blake3() { case "$1" in *[!0-9a-f]* | '') return 1 ;; *) [ "${#1}" -eq 64 ] ;; esac; }

# --- verdict emitters ------------------------------------------------------
_ARTIFACT_JSON=""

emit_and_exit() {
  # $1 = status (PASS|BLOCK) ; $2 = exit code ; $3 = diagnosis
  local status="$1" code="$2" diag="$3"
  mkdir -p "$(dirname "$BENCH_ARTIFACT")" 2>/dev/null || true
  # `_ARTIFACT_JSON` is assembled by build_artifact() before a PASS; for a
  # BLOCK it may be empty, so fall back to a minimal, still-valid artefact.
  if [ -z "$_ARTIFACT_JSON" ]; then
    if have_py3; then
      _ARTIFACT_JSON="$(B_STATUS="$status" B_DIAG="$diag" B_VER="$BENCH_SCHEMA_VERSION" \
        B_MODEL="$(redact_model "$MODEL_20GB")" B_COMMIT="$LLAMACPP_COMMIT" \
        B_QUANT="$QUANT" B_NSHARDS="$N_SHARDS" \
        B_PP="$BENCH_PP" B_TG="$BENCH_TG" B_THREADS="$BENCH_THREADS" B_REPS="$BENCH_REPS" python3 -c '
import json, os
print(json.dumps({
  "schema_version": int(os.environ["B_VER"]),
  "status": os.environ["B_STATUS"],
  "diagnosis": os.environ["B_DIAG"],
  "model": os.environ["B_MODEL"],
  "model_blake3": None,
  "quant": os.environ["B_QUANT"],
  "n_shards": int(os.environ["B_NSHARDS"]),
  "llamacpp_commit": os.environ["B_COMMIT"],
  "bench_params": {
    "pp": int(os.environ["B_PP"]),
    "tg": int(os.environ["B_TG"]),
    "threads": int(os.environ["B_THREADS"]),
    "repetitions": int(os.environ["B_REPS"]),
  },
  "single_machine": [],
  "sharded": None,
  "perplexity_parity": None,
}, indent=2))')"
    else
      _ARTIFACT_JSON="{\"schema_version\":$BENCH_SCHEMA_VERSION,\"status\":\"$status\",\"diagnosis\":\"$(printf '%s' "$diag" | tr -d '\\"' )\",\"model\":\"$(redact_model "$MODEL_20GB")\",\"single_machine\":[],\"sharded\":null,\"perplexity_parity\":null}"
    fi
  fi
  # Hygiene backstop: NEVER commit an artefact carrying a filesystem path.
  assert_no_fs_path "$_ARTIFACT_JSON"
  # A failed write (non-writable BENCH_ARTIFACT / full disk) must NEVER be a
  # false-green: the T3 gate requires a PARSEABLE artefact to exist, so a write
  # failure is FATAL (exit 2), overriding the requested exit code (Codex P1-3).
  if ! printf '%s\n' "$_ARTIFACT_JSON" >"$BENCH_ARTIFACT" || [ ! -s "$BENCH_ARTIFACT" ]; then
    printf '[bench][FATAL] could not write a non-empty artefact — refusing a hollow verdict\n' >&2
    exit 2
  fi
  cat "$BENCH_ARTIFACT"
  printf '[bench][%s] %s\n' "$status" "$diag" >&2
  exit "$code"
}

assert_no_fs_path() {
  # Fail if the artefact still carries an absolute FS path / a home dir /
  # a username-bearing prefix (the leak the preflight caught). `*Users*`
  # catches every slash direction and JSON-escaping of the Windows home
  # (C:/Users, C:\\Users) at once; `/home/` covers POSIX; `spike_fork` the
  # specific rig dir.
  case "$1" in
    *Users*|*/home/*|*spike_fork*)
      printf '[bench][FATAL] artefact carries a filesystem path — model must be NAME+blake3, not a path\n' >&2
      exit 2
      ;;
  esac
}

block_rig() { emit_and_exit "BLOCK" 1 "rig: $1"; }

# ==========================================================================
# PREFLIGHT — a cold rig is BLOCK{rig} (exit 1), never RIG-ABSENT: the rig is
# engaged for Phase A boot-SEED, so its absence for THIS phase is a cold
# input, not "the test could not run at all".
# ==========================================================================
log "=== preflight (standard-benchmark tool + corpus presence) ==="
have_py3 || log "note: python3 absent — using the lossy pure-bash artefact encoder"

# NOTE: block diagnoses reference VARIABLE NAMES, never raw path values — the
# artefact is committed, and a path value would leak the FS layout + username
# (preflight S3 caution). `assert_no_fs_path` is the backstop.
[ -n "$MODEL_20GB" ]      || block_rig "MODEL_20GB unset (the ~20 GB GGUF; set it in rig.local.env)"
[ -f "$MODEL_20GB" ]      || block_rig "MODEL_20GB does not point at a file on the head (fix the path in rig.local.env)"
[ -n "$LLAMA_BENCH_BIN" ] || block_rig "LLAMA_BENCH_BIN unset — build llama-bench from the pinned upstream llama.cpp checkout (see header) and point at it in rig.local.env"
[ -x "$LLAMA_BENCH_BIN" ] || block_rig "LLAMA_BENCH_BIN does not point at an executable (build llama-bench from the pinned upstream checkout)"

MODEL_NAME="$(redact_model "$MODEL_20GB")"
MODEL_B3="$(blake3_of "$MODEL_20GB")"
log "model      : $MODEL_NAME (blake3 ${MODEL_B3})"
log "llama.cpp  : commit $LLAMACPP_COMMIT (must match the shard backend bundle for a valid PPL parity)"

# ==========================================================================
# 1. SINGLE-MACHINE BASELINES — llama-bench pp512 / tg128.
# ==========================================================================
log "=== single-machine baseline: llama-bench pp${BENCH_PP} / tg${BENCH_TG} (head) ==="
run_llama_bench() {
  # $1 = bin ; $2 = model ; prints "pp_ts tg_ts" (tokens/s, floats) or empty.
  local bin="$1" model="$2" out
  out="$("$bin" -m "$model" -p "$BENCH_PP" -n "$BENCH_TG" -t "$BENCH_THREADS" \
        -r "$BENCH_REPS" -o json 2>/dev/null || true)"
  [ -n "$out" ] || { printf ''; return; }
  if have_py3; then
    # Tab-separated (empty field = missing metric). Old-style `%` formatting,
    # not an f-string: a quote/backslash inside an f-string EXPRESSION is a
    # SyntaxError before Python 3.12 (PEP 701), and this runs on whatever
    # python3 the rig ships.
    printf '%s' "$out" | python3 -c '
import json, sys
try:
    rows = json.load(sys.stdin)
except Exception:
    sys.exit(0)
pp = tg = None
for r in rows:
    # llama-bench json rows carry n_prompt/n_gen + avg_ts (tokens/s).
    if r.get("n_prompt", 0) and not r.get("n_gen", 0):
        pp = r.get("avg_ts")
    if r.get("n_gen", 0) and not r.get("n_prompt", 0):
        tg = r.get("avg_ts")
print("%s\t%s" % ("" if pp is None else pp, "" if tg is None else tg))'
  else
    printf ''
  fi
}

HEAD_BASELINE="$(run_llama_bench "$LLAMA_BENCH_BIN" "$MODEL_20GB")"
HEAD_PP="$(printf '%s' "$HEAD_BASELINE" | cut -f1)"
HEAD_TG="$(printf '%s' "$HEAD_BASELINE" | cut -f2)"
if [ -z "$HEAD_PP" ] || [ -z "$HEAD_TG" ]; then
  block_rig "llama-bench produced no pp/tg row on the head (needs -o json support; check the build backend and model fit)"
fi
log "head pp${BENCH_PP}=${HEAD_PP} tok/s  tg${BENCH_TG}=${HEAD_TG} tok/s"

# Optional 2nd machine (Metal) baseline over SSH.
MAC_PP=""; MAC_TG=""
if [ -n "$MAC_SSH" ] && [ -n "$MAC_LLAMA_BENCH_BIN" ] && [ -n "$MAC_MODEL" ]; then
  log "=== single-machine baseline: Metal machine $MAC_SSH ==="
  MAC_OUT="$(ssh -o ConnectTimeout=15 -o BatchMode=yes "$MAC_SSH" \
    "'$MAC_LLAMA_BENCH_BIN' -m '$MAC_MODEL' -p $BENCH_PP -n $BENCH_TG -t $BENCH_THREADS -r $BENCH_REPS -o json" 2>/dev/null || true)"
  if [ -n "$MAC_OUT" ] && have_py3; then
    MAC_PARSED="$(printf '%s' "$MAC_OUT" | python3 -c '
import json, sys
try: rows = json.load(sys.stdin)
except Exception: sys.exit(0)
pp=tg=None
for r in rows:
    if r.get("n_prompt",0) and not r.get("n_gen",0): pp=r.get("avg_ts")
    if r.get("n_gen",0) and not r.get("n_prompt",0): tg=r.get("avg_ts")
print("%s\t%s" % ("" if pp is None else pp, "" if tg is None else tg))')"
    MAC_PP="$(printf '%s' "$MAC_PARSED" | cut -f1)"
    MAC_TG="$(printf '%s' "$MAC_PARSED" | cut -f2)"
    log "mac  pp${BENCH_PP}=${MAC_PP:-?} tok/s  tg${BENCH_TG}=${MAC_TG:-?} tok/s"
  else
    log "note: Metal machine baseline unavailable (SSH/tool/model) — recording head only"
  fi
fi

# ==========================================================================
# 2. PERPLEXITY — PPL(whole) on wikitext-2 (the standard corpus), single
# machine. PPL(sharded) is NOT wired in Phase B (no /result producer), so the
# parity stays limited to ppl_whole; the tail-side scalar design (never a
# cross-machine logprob route, guard S3) is future work. ppl_whole is recorded
# when available but does NOT gate the PASS.
# ==========================================================================
PPL_WHOLE=""
WIKITEXT2_B3=""
if [ -n "$PERPLEXITY_BIN" ] && [ -x "$PERPLEXITY_BIN" ] && [ -n "$WIKITEXT2_PATH" ] && [ -f "$WIKITEXT2_PATH" ]; then
  log "=== perplexity parity: PPL(whole) on wikitext-2 (seed $PPL_SEED) ==="
  # Pin the corpus by content so two PPL runs are only comparable on the SAME
  # wikitext-2 file (a truncated / wrong-variant corpus is otherwise silent).
  WIKITEXT2_B3="$(blake3_of "$WIKITEXT2_PATH")"
  PPL_OUT="$("$PERPLEXITY_BIN" -m "$MODEL_20GB" -f "$WIKITEXT2_PATH" -t "$BENCH_THREADS" -s "$PPL_SEED" 2>&1 || true)"
  PPL_WHOLE="$(printf '%s' "$PPL_OUT" | sed -n 's/.*Final estimate: PPL = \([0-9.]*\).*/\1/p' | head -1)"
  log "PPL(whole) = ${PPL_WHOLE:-<not parsed>} (corpus blake3 ${WIKITEXT2_B3})"
else
  log "note: perplexity binary or wikitext-2 corpus absent — PPL(whole) skipped (single-machine baselines still recorded)"
fi

# ==========================================================================
# 3. SHARDED FINE METRICS — read the extended b3_shard artefact (Sprint 82
# Phase B host-side instrumentation on the /result view).
# ==========================================================================
SH_TTFT_MS=""; SH_TPOT_MS=""; SH_ITL_P50=""; SH_ITL_P95=""; SH_MTOKS=""; SH_TOKENS=""; PPL_SHARD=""
if [ -f "$B3_ARTIFACT" ] && have_py3; then
  log "=== sharded fine metrics: reading + validating the b3 artefact ==="
  # Freshness (Codex P1-A stale binding): reject a b3 artefact older than
  # B3_MAX_AGE_MIN. mtime is a proxy, not a cryptographic run-id (a deliberately
  # touched file defeats it — documented limitation), but an accidentally-reused
  # OLD artefact from a previous session is rejected.
  if [ -z "$(find "$B3_ARTIFACT" -mmin "-$B3_MAX_AGE_MIN" 2>/dev/null)" ]; then
    log "note: b3 artefact REJECTED — older than ${B3_MAX_AGE_MIN} min (stale; re-run b3_shard_pipeline.sh for the current session)"
  else
    # Validate EVERY required field belongs to THIS run (Codex P1-A): status=PASS,
    # model NAME + content blake3 match, n_shards match, and ALL FIVE fine
    # metrics + tokens are INTEGERS. A null / non-numeric / incomplete /
    # different-n_shards / different-content b3 artefact is REJECTED -> sharded
    # stays absent -> BLOCK{rig} at the PASS gate, never a hollow PASS.
    SH_JSON="$(B3="$B3_ARTIFACT" MN="$MODEL_NAME" MB3="$MODEL_B3" NS="$N_SHARDS" python3 -c '
import json, os
try:
    d = json.load(open(os.environ["B3"]))
except Exception:
    d = {}
def isint(v):
    return isinstance(v, int) and not isinstance(v, bool)
mb3 = os.environ["MB3"]
metrics = ("ttft_ms","tpot_ms","itl_p50_ms","itl_p95_ms","decode_milli_tokens_per_sec","tokens")
ok = (d.get("status") == "PASS"
      and os.path.basename(str(d.get("model",""))) == os.environ["MN"]
      and (d.get("model_blake3") == mb3)
      and str(d.get("n_shards")) == os.environ["NS"]
      and all(isint(d.get(k)) for k in metrics))
if not ok:
    print("")
else:
    fields = [str(d[k]) for k in metrics]
    ppl = d.get("ppl_sharded")
    fields.append("" if ppl is None else str(ppl))
    print("\t".join(fields))')"
    if [ -z "$SH_JSON" ]; then
      log "note: b3 artefact REJECTED (status != PASS, model NAME/blake3 mismatch, n_shards mismatch, or a fine metric is null/non-integer) — sharded stays absent"
    else
      SH_TTFT_MS="$(printf '%s' "$SH_JSON" | cut -f1)"
      SH_TPOT_MS="$(printf '%s' "$SH_JSON" | cut -f2)"
      SH_ITL_P50="$(printf '%s' "$SH_JSON" | cut -f3)"
      SH_ITL_P95="$(printf '%s' "$SH_JSON" | cut -f4)"
      SH_MTOKS="$(printf '%s' "$SH_JSON" | cut -f5)"
      SH_TOKENS="$(printf '%s' "$SH_JSON" | cut -f6)"
      PPL_SHARD="$(printf '%s' "$SH_JSON" | cut -f7)"
      log "sharded ttft_ms=$SH_TTFT_MS tpot_ms=$SH_TPOT_MS itl_p50=$SH_ITL_P50 itl_p95=$SH_ITL_P95 mtok/s=$SH_MTOKS tokens=$SH_TOKENS"
    fi
  fi
else
  log "note: no b3_shard artefact present — single-machine baselines only, sharded section null"
fi

# ==========================================================================
# BUILD THE VERSIONED ARTEFACT + PASS.
# ==========================================================================
build_artifact() {
  have_py3 || { _ARTIFACT_JSON=""; return; }
  _ARTIFACT_JSON="$(
    B_VER="$BENCH_SCHEMA_VERSION" B_MODEL="$MODEL_NAME" B_B3="$MODEL_B3" \
    B_QUANT="$QUANT" B_NSHARDS="$N_SHARDS" B_COMMIT="$LLAMACPP_COMMIT" \
    B_PP="$BENCH_PP" B_TG="$BENCH_TG" B_THREADS="$BENCH_THREADS" B_REPS="$BENCH_REPS" \
    B_HEAD_PP="$HEAD_PP" B_HEAD_TG="$HEAD_TG" B_MAC_PP="$MAC_PP" B_MAC_TG="$MAC_TG" B_MAC="$MAC_SSH" \
    B_PPL_WHOLE="$PPL_WHOLE" B_PPL_SHARD="$PPL_SHARD" B_SEED="$PPL_SEED" B_WIKI_B3="$WIKITEXT2_B3" \
    B_TTFT="$SH_TTFT_MS" B_TPOT="$SH_TPOT_MS" B_P50="$SH_ITL_P50" B_P95="$SH_ITL_P95" \
    B_MTOKS="$SH_MTOKS" B_TOKENS="$SH_TOKENS" \
    python3 -c '
import json, os
def f(k):
    v = os.environ.get(k, "")
    if v == "": return None
    try: return float(v)
    except Exception: return None
def i(k):
    v = os.environ.get(k, "")
    if v == "": return None
    try: return int(float(v))
    except Exception: return None
single = [{
    "machine": "head",
    "backend": "cuda",
    "pp_tok_s": f("B_HEAD_PP"),
    "tg_tok_s": f("B_HEAD_TG"),
}]
if os.environ.get("B_MAC"):
    # Non-identifying label: never the SSH user@host (that would commit an
    # operator identity/target into the artefact) — the backend says Metal.
    single.append({
        "machine": "mac-metal",
        "backend": "metal",
        "pp_tok_s": f("B_MAC_PP"),
        "tg_tok_s": f("B_MAC_TG"),
    })
ppl_whole = f("B_PPL_WHOLE"); ppl_shard = f("B_PPL_SHARD")
wiki_b3 = os.environ.get("B_WIKI_B3") or None
if wiki_b3 == "unavailable": wiki_b3 = None
parity = None
if ppl_whole is not None:
    parity = {
        "corpus": "wikitext-2-raw",
        "corpus_blake3": wiki_b3,
        "seed": i("B_SEED"),
        "ppl_whole": ppl_whole,
        "ppl_sharded": ppl_shard,
        "delta": (round(ppl_shard - ppl_whole, 4) if (ppl_shard is not None) else None),
        "note": "ppl_sharded/delta are NOT wired in Phase B: no /result producer emits a sharded PPL yet, so they stay null even on a hot rig. The intended design (PPL computed tail-side, emitted as a SCALAR via /result, never a cross-machine logprob route, guard S3) is documented in docs/protocol/SHARD_BENCHMARKS.md for future work.",
    }
sharded = None
if i("B_TOKENS") is not None:
    sharded = {
        "n_shards": i("B_NSHARDS"),
        "ttft_ms": i("B_TTFT"),
        "tpot_ms": i("B_TPOT"),
        "itl_p50_ms": i("B_P50"),
        "itl_p95_ms": i("B_P95"),
        "decode_milli_tokens_per_sec": i("B_MTOKS"),
        "tokens": i("B_TOKENS"),
    }
b3 = os.environ.get("B_B3") or None
if b3 == "unavailable": b3 = None
print(json.dumps({
    "schema_version": int(os.environ["B_VER"]),
    "status": "PASS",
    "diagnosis": "standard baselines measured",
    "model": os.environ["B_MODEL"],
    "model_blake3": b3,
    "quant": os.environ.get("B_QUANT"),
    "n_shards": int(os.environ["B_NSHARDS"]),
    "llamacpp_commit": os.environ.get("B_COMMIT"),
    "bench_params": {
        "pp": int(os.environ["B_PP"]),
        "tg": int(os.environ["B_TG"]),
        "threads": int(os.environ["B_THREADS"]),
        "repetitions": int(os.environ["B_REPS"]),
    },
    "single_machine": single,
    "sharded": sharded,
    "perplexity_parity": parity,
}, indent=2))')"
}

# ==========================================================================
# PASS GATE (Codex P1-1 / P1-2 / P1-4) — a PASS must mean the SHARDING was
# really measured with comparable provenance, NOT merely a single-machine
# baseline. Any missing requirement => BLOCK{rig}, never a hollow PASS.
# ==========================================================================
# Head single-machine baseline (already gated above; re-assert for clarity).
{ [ -n "$HEAD_PP" ] && [ -n "$HEAD_TG" ]; } || \
  block_rig "no single-machine baseline (llama-bench pp/tg on the head)"
# The sharded fine metrics are the CORE deliverable: a benchmark of the
# sharding that never measured the sharding is not a PASS. SH_TOKENS/SH_TTFT_MS
# are set ONLY from a VALIDATED (status=PASS, model-matched) b3 artefact.
{ [ -n "$SH_TOKENS" ] && [ -n "$SH_TTFT_MS" ]; } || \
  block_rig "no valid current sharded metrics — mount + drive a shard session (b3_shard_pipeline.sh) so this run reads a matching PASS b3 artefact; a stale/BLOCK/mismatched one is rejected"
# Provenance pins make runs comparable (the determinism claim): without the
# exact llama.cpp snapshot AND a valid model content hash, a PASS baseline is
# not reproducible -> BLOCK{rig}, never a false comparable-PASS. The pins are
# format-validated (Codex P1-B: an empty/malformed value must not pass as a pin).
case "$LLAMACPP_COMMIT" in
  unknown | '') block_rig "LLAMACPP_COMMIT unset — resolve the ggml-org sha (rig.local.env) so throughput/PPL are comparable across runs" ;;
  *[!0-9a-f]*) block_rig "LLAMACPP_COMMIT is not a lowercase-hex commit sha" ;;
esac
[ "${#LLAMACPP_COMMIT}" -ge 7 ] || block_rig "LLAMACPP_COMMIT too short to be a commit sha (>= 7 hex)"
is_blake3 "$MODEL_B3" || block_rig "model blake3 unavailable or invalid (install b3sum) — a valid 64-hex content pin is required for a reproducible baseline"

build_artifact
if [ -z "$_ARTIFACT_JSON" ]; then
  block_rig "could not assemble the versioned artefact (python3 required for the JSON schema encoder)"
fi
emit_and_exit "PASS" 0 "single-machine baselines + validated sharded metrics measured, provenance pinned"
