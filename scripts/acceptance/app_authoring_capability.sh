#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Sprint 79 Phase H — app-authoring CSP capability acceptance harness (T2 gate).
#
# The machine-readable wrapper around the RUNTIME self-check. It drives the
# hermetic Playwright spec `web/e2e/app-authoring.spec.ts` (T1) — which spawns
# a real nexus-shell-daemon, seeds a CLEAN and a DIRTY fixture app, replays
# each inside the production sandboxed iframe under the REAL BLOB_SERVE_CSP,
# and (a) byte-compares the SERVED CSP header to the single-source contract,
# (b) asserts the clean app emits zero CSP violation, (c) asserts the dirty
# app's runtime-assembled fetch IS caught — then emits a JSON verdict.
#
# === Machine-readable contract (T2 testability gate, README §4) ===
# Every exit writes a JSON artefact (default
# `scripts/acceptance/.app_authoring_last_result.json`, override ARTIFACT) AND
# exits with a status-specific code, so the result is never a prose-only
# verdict:
#   - PASS       exit 0  — served CSP == contract AND clean=0 violations AND
#                          dirty>=1 violation. The runtime net both (i) tests
#                          the real served policy and (ii) DETECTS what the
#                          static gate misses (the negative control is
#                          load-bearing: a clean-only pass is vacuous).
#   - BLOCK      exit 1  — the self-check ran but a check failed: CSP drift
#                          (gate would protect a fictional policy), OR the
#                          dirty fixture was NOT detected (false-green hole),
#                          OR the clean fixture emitted a violation. The
#                          product question we test.
#   - RIG-ABSENT exit 3  — the self-check could not run end-to-end (no Node /
#                          npx, no built shell `web/dist`, no daemon binary,
#                          or no Chromium for Playwright). NOT a product
#                          failure; the test simply could not run.
# Artefact shape:
#   {"status","stage","blob_serve_csp_equals_contract","clean_clean",
#    "dirty_detected","tests_passed","tests_total","diagnosis"}
#
# === Honest expected status TODAY: PASS (hermetic, no rig) ===
# Unlike the sharding T2, this capability needs NO multi-machine rig: a single
# host with a built shell + the daemon binary + Chromium reaches a real PASS.
# RIG-ABSENT here means strictly "no browser/build to run the check", never a
# masked product gap.
#
# Usage (from the repo root):
#   bash scripts/acceptance/app_authoring_capability.sh
# Prereqs (the same the hermetic E2E needs; built by verify.sh before this):
#   - web/dist           (npm --prefix web run build)
#   - the daemon binary  (cargo build -p nexus-shell-daemon --release)
#   - Chromium           (npx --prefix web playwright install chromium)

set -uo pipefail

# Repo root = two levels up from this script (scripts/acceptance/).
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WEB_DIR="$REPO_ROOT/web"

ARTIFACT="${ARTIFACT:-$SCRIPT_DIR/.app_authoring_last_result.json}"
PW_JSON="${PW_JSON:-$SCRIPT_DIR/.app_authoring_pw.json}"

# Runtime state referenced by the artefact writer (filled as we progress).
CSP_EQ="null"      # blob_serve_csp_equals_contract
CLEAN_CLEAN="null" # clean fixture emitted zero violation
DIRTY_DET="null"   # dirty fixture violation detected
TESTS_PASSED="null"
TESTS_TOTAL="null"

log() { printf '[app-authoring] %s\n' "$*"; }

# --- JSON artefact (machine-readable, written on every exit) --------------
# python3 (json.dumps) preferred; pure-bash fallback is lossy but always valid.
# Boolean fields pass through as the literals true/false/null (never quoted).
_json_safe() { printf '%s' "$1" | tr -d '\\"' | tr -d '\000-\037'; }

emit_artifact() {
  # $1=status $2=stage $3=diagnosis
  local status="$1" stage="$2" diag="$3" _emitted=0
  mkdir -p "$(dirname "$ARTIFACT")" 2>/dev/null || true
  if command -v python3 >/dev/null 2>&1; then
    A_STATUS="$status" A_STAGE="$stage" A_CSP="$CSP_EQ" A_CLEAN="$CLEAN_CLEAN" \
    A_DIRTY="$DIRTY_DET" A_TP="$TESTS_PASSED" A_TT="$TESTS_TOTAL" A_DIAG="$diag" \
    python3 -c '
import json, os
def b(v):
    return {"true": True, "false": False}.get(v, None)
def num(v):
    try:
        return int(v)
    except Exception:
        return None
print(json.dumps({
    "status": os.environ["A_STATUS"],
    "stage": os.environ["A_STAGE"],
    "blob_serve_csp_equals_contract": b(os.environ["A_CSP"]),
    "clean_clean": b(os.environ["A_CLEAN"]),
    "dirty_detected": b(os.environ["A_DIRTY"]),
    "tests_passed": num(os.environ["A_TP"]),
    "tests_total": num(os.environ["A_TT"]),
    "diagnosis": os.environ["A_DIAG"],
}))' >"$ARTIFACT" 2>/dev/null && [ -s "$ARTIFACT" ] && _emitted=1
  fi
  if [ "$_emitted" -ne 1 ]; then
    local diag_e csp="$CSP_EQ" clean="$CLEAN_CLEAN" dirty="$DIRTY_DET"
    local tp="$TESTS_PASSED" tt="$TESTS_TOTAL"
    diag_e="$(_json_safe "$diag")"
    case "$csp" in true|false|null) ;; *) csp=null ;; esac
    case "$clean" in true|false|null) ;; *) clean=null ;; esac
    case "$dirty" in true|false|null) ;; *) dirty=null ;; esac
    case "$tp" in ''|*[!0-9]*) tp=null ;; esac
    case "$tt" in ''|*[!0-9]*) tt=null ;; esac
    printf '{"status":"%s","stage":"%s","blob_serve_csp_equals_contract":%s,"clean_clean":%s,"dirty_detected":%s,"tests_passed":%s,"tests_total":%s,"diagnosis":"%s"}\n' \
      "$status" "$stage" "$csp" "$clean" "$dirty" "$tp" "$tt" "$diag_e" >"$ARTIFACT"
  fi
  cat "$ARTIFACT" 2>/dev/null || true
}

rig_absent() {
  emit_artifact "RIG-ABSENT" "preflight" "$1"
  printf '[app-authoring][RIG-ABSENT] %s\n' "$1" >&2
  exit 3
}
block() {
  # $1=stage $2=diagnosis
  emit_artifact "BLOCK" "$1" "$2"
  printf '[app-authoring][BLOCK] %s\n' "$2" >&2
  exit 1
}
pass() {
  emit_artifact "PASS" "verdict" "$1"
  printf '[app-authoring][PASS] %s\n' "$1"
  exit 0
}

# ==========================================================================
# PREFLIGHT — any failure here is RIG-ABSENT (exit 3), never a product BLOCK.
# ==========================================================================
log "=== preflight (node + built shell + daemon binary) ==="
command -v node >/dev/null 2>&1 || rig_absent "node not found on PATH"
command -v npx  >/dev/null 2>&1 || rig_absent "npx not found on PATH"
[ -d "$WEB_DIR/dist" ] || rig_absent "web/dist missing — run \`npm --prefix web run build\` first (verify.sh builds the shell before the E2E step)"

# Daemon binary the hermetic global-setup spawns (release preferred, else debug).
DAEMON_BIN="${SBFB_DAEMON_BIN:-}"
if [ -z "$DAEMON_BIN" ]; then
  for cand in "$REPO_ROOT/target/release/nexus-shell-daemon" "$REPO_ROOT/target/release/nexus-shell-daemon.exe" \
              "$REPO_ROOT/target/debug/nexus-shell-daemon" "$REPO_ROOT/target/debug/nexus-shell-daemon.exe"; do
    [ -x "$cand" ] && { DAEMON_BIN="$cand"; break; }
  done
fi
[ -n "$DAEMON_BIN" ] && [ -x "$DAEMON_BIN" ] || rig_absent "nexus-shell-daemon binary not found (cargo build -p nexus-shell-daemon --release), and SBFB_DAEMON_BIN unset"
log "daemon binary  : $DAEMON_BIN"

# ==========================================================================
# RUN — drive the hermetic Playwright self-check, capture the JSON report.
# ==========================================================================
log "=== run (hermetic Playwright self-check: seed + replay under real CSP) ==="
# TRUNCATE in place (not `rm -f`, whose failure we used to swallow): a stale
# report from a PRIOR 3/3 run on this FIXED path must never be read as THIS run's
# result. If Playwright writes nothing this run (e.g. globalSetup throws before
# the JSON reporter emits), the file stays EMPTY → the `[ -s "$PW_JSON" ]` check
# below fails → rig_absent, never a hollow PASS on a périmé report. A truncate
# failure (locked/perms) is itself rig-absent, not silently ignored.
: > "$PW_JSON" 2>/dev/null || rig_absent "cannot truncate the report path $PW_JSON (locked/permissions) — refusing to risk reading a stale report"
PW_OUT="$SCRIPT_DIR/.app_authoring_pw.log"
# `unset SBFB_E2E_*` inside the subshell: the spec's hermetic seeding only runs
# when `SBFB_E2E_BASE_URL` is ABSENT (external-daemon mode skips it). A leaked
# env var would silently skip all 3 sub-tests — and Playwright exits 0 on skip,
# which the verdict gate below would otherwise read as a hollow PASS. Unsetting
# them here guarantees the hermetic self-check actually runs.
(
  cd "$WEB_DIR" \
    && unset SBFB_E2E_BASE_URL SBFB_E2E_COMPUTE SBFB_E2E_PROJECT_ID SBFB_E2E_MODEL \
    && SBFB_DAEMON_BIN="$DAEMON_BIN" PLAYWRIGHT_JSON_OUTPUT_NAME="$PW_JSON" \
       npx playwright test app-authoring.spec.ts --reporter=json
) >"$PW_OUT" 2>&1
PW_EXIT=$?
log "playwright exit: $PW_EXIT"

# A missing browser is RIG-ABSENT, not a product BLOCK. Narrow match so a real
# launch crash/timeout (a product BLOCK) is NOT mis-classified as rig-absent.
if grep -qiE "Executable doesn't exist|Please run.*playwright install" "$PW_OUT" 2>/dev/null; then
  rig_absent "Chromium not installed for Playwright (npx --prefix web playwright install chromium). Output: $(tail -c 400 "$PW_OUT" 2>/dev/null)"
fi

[ -s "$PW_JSON" ] || {
  # No JSON report produced at all — could not run end-to-end.
  rig_absent "Playwright produced no JSON report ($PW_JSON). The self-check did not run. Tail: $(tail -c 400 "$PW_OUT" 2>/dev/null)"
}

# ==========================================================================
# PARSE + VERDICT — map the three sub-tests to the capability fields.
# Anti-false-green: PASS requires CSP==contract AND clean clean AND dirty
# DETECTED. A missing/unparseable signal is a BLOCK to diagnose, never a
# hollow PASS.
# ==========================================================================
# python3 is REQUIRED to parse the report and verify the run honestly. There is
# NO coarse exit-code fallback: Playwright exits 0 on all-SKIPPED, so trusting
# the exit code alone is a false-green path. Without a parser we cannot tell a
# real 3/3 PASS from a skipped suite — that is RIG-ABSENT, never a hollow PASS.
command -v python3 >/dev/null 2>&1 || rig_absent "python3 not found — required to parse the Playwright JSON report and verify the self-check actually ran (Playwright exits 0 on skip, so the exit code alone cannot be trusted)."

PARSE="$(PW_JSON="$PW_JSON" python3 -c '
import json, os
data = json.load(open(os.environ["PW_JSON"], encoding="utf-8"))
# Flatten Playwright JSON report -> {title: status}.
res = {}
def walk(suite):
    for sp in suite.get("specs", []):
        st = "unknown"
        for t in sp.get("tests", []):
            for r in t.get("results", []):
                st = r.get("status", st)
        res[sp.get("title", "")] = st
    for s in suite.get("suites", []):
        walk(s)
for s in data.get("suites", []):
    walk(s)
def has(frag):
    for k, v in res.items():
        if frag in k:
            return v
    return "missing"
csp   = has("byte-equal to the single-source contract")
clean = has("zero violation")
dirty = has("caught by the CSP at runtime")
def b(v): return "true" if v in ("passed","expected") else ("null" if v=="missing" else "false")
passed  = sum(1 for v in res.values() if v in ("passed", "expected"))
skipped = sum(1 for v in res.values() if v == "skipped")
print("CSP=%s" % b(csp)); print("CLEAN=%s" % b(clean)); print("DIRTY=%s" % b(dirty))
print("TP=%d" % passed); print("SK=%d" % skipped); print("TT=%d" % len(res))
' 2>/dev/null)"
CSP_EQ="$(printf '%s\n' "$PARSE" | sed -n 's/^CSP=//p')"
CLEAN_CLEAN="$(printf '%s\n' "$PARSE" | sed -n 's/^CLEAN=//p')"
DIRTY_DET="$(printf '%s\n' "$PARSE" | sed -n 's/^DIRTY=//p')"
TESTS_PASSED="$(printf '%s\n' "$PARSE" | sed -n 's/^TP=//p')"
TESTS_TOTAL="$(printf '%s\n' "$PARSE" | sed -n 's/^TT=//p')"
SKIPPED="$(printf '%s\n' "$PARSE" | sed -n 's/^SK=//p')"
# Normalize the per-field signals (empty -> null) symmetrically.
[ -n "$CSP_EQ" ]      || CSP_EQ="null"
[ -n "$CLEAN_CLEAN" ] || CLEAN_CLEAN="null"
[ -n "$DIRTY_DET" ]   || DIRTY_DET="null"

# An empty/unparseable report (python3 ran but produced nothing — malformed JSON
# or a broken shim) is RIG-ABSENT, not a misleading per-field BLOCK.
case "${TESTS_TOTAL:-}" in
  ''|*[!0-9]*) rig_absent "could not parse the Playwright report (TESTS_TOTAL='${TESTS_TOTAL:-}') — the self-check verdict is unverifiable. Tail: $(tail -c 400 "$PW_OUT" 2>/dev/null)" ;;
esac
case "${TESTS_PASSED:-}" in ''|*[!0-9]*) TESTS_PASSED=0 ;; esac
case "${SKIPPED:-}" in ''|*[!0-9]*) SKIPPED=0 ;; esac

# Title-INDEPENDENT run gate (anti-false-green core): prove the 3 controls
# actually RAN and PASSED before looking at any per-field signal. Playwright
# exits 0 on all-skipped, so the exit code is never trusted on its own.
[ "$TESTS_TOTAL" -ge 3 ] || block "run" "expected >=3 self-check sub-tests, the report has $TESTS_TOTAL — the controls did not run (a leaked SBFB_E2E_* env, a stale --grep filter, or a renamed spec). Refusing a hollow PASS."
[ "$SKIPPED" -eq 0 ] || block "run" "$SKIPPED/$TESTS_TOTAL sub-tests were SKIPPED — Playwright exits 0 on skip, so this is NOT a pass. The hermetic self-check did not exercise its controls."
[ "$TESTS_PASSED" -eq "$TESTS_TOTAL" ] || block "run" "$((TESTS_TOTAL-TESTS_PASSED))/$TESTS_TOTAL sub-tests did not pass. Tail: $(tail -c 400 "$PW_OUT" 2>/dev/null)"

# The run gate proved >=3 ran + all passed + 0 skipped. A still-null per-field
# signal therefore means the spec TITLES drifted from the substrings this script
# matches — BLOCK loudly (so the substrings get fixed), never silently PASS on a
# stale breakdown.
for pair in "blob_serve_csp_equals_contract=$CSP_EQ" "clean_clean=$CLEAN_CLEAN" "dirty_detected=$DIRTY_DET"; do
  [ "${pair#*=}" = "null" ] && block "title-drift" "all $TESTS_TOTAL sub-tests ran and passed, but '${pair%%=*}' could not be mapped to a sub-test — the spec title drifted from this script's substring. Update the title substrings in app_authoring_capability.sh; refusing a PASS on a stale breakdown."
done

# Per-field verdict (defense-in-depth; after the gates above all three are "true"
# unless a control genuinely failed, which the run gate already caught).
[ "$CSP_EQ" = "true" ] || block "csp-contract" "served CSP header is NOT byte-equal to the single-source contract (csp-contract.json) — the static gate would protect a fictional policy. blob_serve_csp_equals_contract=$CSP_EQ"
[ "$DIRTY_DET" = "true" ] || block "detect" "the DIRTY fixture's runtime-assembled fetch was NOT detected as a CSP violation — false-green hole: the runtime net does not catch what the static gate misses. dirty_detected=$DIRTY_DET"
[ "$CLEAN_CLEAN" = "true" ] || block "clean" "the CLEAN fixture emitted a CSP violation — the self-check flags a conformant app. clean_clean=$CLEAN_CLEAN"

pass "served CSP == contract; clean app emitted 0 violations; dirty app's runtime fetch was caught ($TESTS_PASSED/$TESTS_TOTAL sub-tests passed, $SKIPPED skipped)"
