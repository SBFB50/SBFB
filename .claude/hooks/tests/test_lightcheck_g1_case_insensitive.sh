#!/usr/bin/env bash
#
# Regression test for S83 Phase A1 carry 8 — the G1 Design Review Board
# gate (Check 5) in phase-precommit-lightcheck.sh.
#
# Bug: Check 5 compared the LOWERCASED `$PHASE` to the literal "A"
# (`[ "$PHASE" = "A" ]`), which is never true — so the whole G1 gate was
# dead code and a Phase A commit without sprint{N}_design_review.md was
# never blocked. Fix: compare `$PHASE_UPPER` (line 256).
#
# This test proves, against an isolated throwaway repo:
#   (1) "Sprint 99 Phase A" + design_review absent + no exemption
#       => the Check 5 G1 BLOCK message IS emitted (the gate is live).
#   (2) same title but the kickoff carries "G1 skipped ..."
#       => the Check 5 G1 message is NOT emitted (exemption honoured).
#   (3) "Sprint 99 Phase B" => the gate does NOT fire (Phase A only).
#
# Exit 0 = all assertions pass; exit 1 = a regression.

set -uo pipefail

HOOK="$(cd "$(dirname "$0")/.." && pwd)/phase-precommit-lightcheck.sh"
G1_MSG="BLOCK: G1 Design Review Board manquant"
FAILURES=0

if [ ! -f "$HOOK" ]; then
  echo "FAIL: hook not found at $HOOK" >&2
  exit 1
fi

# The hook extracts the command via jq or python3; skip cleanly if neither
# is available (same guard the hook itself uses at runtime).
if ! command -v jq >/dev/null 2>&1 && ! command -v python3 >/dev/null 2>&1; then
  echo "SKIP: neither jq nor python3 available (hook cannot parse input)" >&2
  exit 0
fi

# --- Isolated throwaway repo with the minimal shape the hook requires ---
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/crates/nexus-core-rs" "$TMP/.planning/active"
printf '[workspace]\n' > "$TMP/Cargo.toml"
(
  cd "$TMP"
  git init -q
  git config user.email test@example.com
  git config user.name test
  git add -A
  git commit -qm init
)

# Run the hook with a synthesized `git commit -m "<title>"` PreToolUse event,
# from inside the throwaway repo (the hook uses `pwd` as REPO_ROOT).
run_hook() {
  local title="$1"
  local json
  json=$(printf '{"tool_input":{"command":"git commit -m \\"%s\\""}}' "$title")
  ( cd "$TMP" && printf '%s' "$json" | bash "$HOOK" 2>&1 )
}

assert_contains() {
  local haystack="$1" needle="$2" label="$3"
  if printf '%s' "$haystack" | grep -qF "$needle"; then
    echo "PASS: $label"
  else
    echo "FAIL: $label (expected to find: $needle)" >&2
    FAILURES=$((FAILURES + 1))
  fi
}

assert_absent() {
  local haystack="$1" needle="$2" label="$3"
  if printf '%s' "$haystack" | grep -qF "$needle"; then
    echo "FAIL: $label (unexpectedly found: $needle)" >&2
    FAILURES=$((FAILURES + 1))
  else
    echo "PASS: $label"
  fi
}

# (1) Phase A, no design_review, no exemption => gate fires.
OUT_BLOCK=$(run_hook "feat(sprint99): Sprint 99 Phase A test G1 gate")
assert_contains "$OUT_BLOCK" "$G1_MSG" \
  "Phase A without design_review is BLOCKED (case-insensitive gate is live)"

# (2) Phase A but kickoff exempts G1 => gate does not fire.
printf 'G1 skipped per user decision 2026-07-18\n' \
  > "$TMP/.planning/active/sprint99_kickoff.md"
OUT_EXEMPT=$(run_hook "feat(sprint99): Sprint 99 Phase A test G1 exempt")
assert_absent "$OUT_EXEMPT" "$G1_MSG" \
  "G1 exemption in kickoff suppresses the gate"
rm -f "$TMP/.planning/active/sprint99_kickoff.md"

# (3) Phase B => the Phase-A-only gate must not fire.
OUT_PHASE_B=$(run_hook "feat(sprint99): Sprint 99 Phase B test not phase A")
assert_absent "$OUT_PHASE_B" "$G1_MSG" \
  "Gate is scoped to Phase A only (Phase B does not trigger it)"

echo ""
if [ "$FAILURES" -eq 0 ]; then
  echo "OK: G1 case-insensitive gate — all 3 assertions passed."
  exit 0
else
  echo "FAILED: $FAILURES assertion(s) regressed." >&2
  exit 1
fi
