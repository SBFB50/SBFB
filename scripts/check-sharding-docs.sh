#!/usr/bin/env bash
#
# Sprint 77 Phase M — doc-lint gate for docs/sharding/.
#
# Three deterministic checks (mirrors scripts/check-spdx.sh discipline,
# BusyBox-safe so it also runs on the Woodpecker bash:5 image — no GNU-only
# grep features: no -P, no --include, no \b):
#
#   (1) link-check   — every repo-relative markdown link target resolves from
#                      the linking doc's directory; cited section anchors
#                      (THREAT_MODEL §16, PATTERNS §P64/§P67, the read-only
#                      route, the front helper) are present in their files.
#   (2) honesty-gate — README + EXPLANATION + HOW_TO_WIRE each carry the
#                      residual-honesty marker (post-S81 the orchestrator +
#                      live benchmark shipped, so the marker is now the S82
#                      residual carry — per-worker proofs / dispute
#                      arbitration — NOT the retired PROVISIONAL banner) and
#                      the cardinal caveat "admission ≠ confidentialité"
#                      (UNCHANGED — the confidentiality boundary is
#                      permanent); REFERENCE marks its thresholds
#                      "S82-pending tuning". The cardinal caveat is the real
#                      backstop; the residual marker keeps a future
#                      "everything done, no caveats" reword failing CI.
#   (3) french-body  — no English UI strings leak into the 3 French docs
#                      (REFERENCE.md is English-body by design, exempted).
#
# Exit 0 = clean, exit 1 = at least one violation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DOCS_DIR="docs/sharding"
FR_DOCS=("$DOCS_DIR/README.md" "$DOCS_DIR/EXPLANATION.md" "$DOCS_DIR/HOW_TO_WIRE.md")
ALL_DOCS=("${FR_DOCS[@]}" "$DOCS_DIR/REFERENCE.md")

fail=0

# All four docs must exist before any other check is meaningful.
for d in "${ALL_DOCS[@]}"; do
  if [ ! -f "$d" ]; then
    echo "MISSING DOC: $d"
    fail=1
  fi
done
if [ "$fail" -ne 0 ]; then
  echo "check-sharding-docs: FAILED (missing docs)"
  exit 1
fi

# ── (1) link-check ───────────────────────────────────────────────────
# Every markdown inline link [text](target) whose target is a repo-relative
# path (not http(s), not a bare #anchor, not mailto) must resolve from the
# linking doc's directory. The optional #anchor is stripped for the file test.
for d in "${ALL_DOCS[@]}"; do
  doc_dir="$(dirname "$d")"
  while IFS= read -r link; do
    target="${link#](}"
    target="${target%)}"
    case "$target" in
      http://*|https://*|mailto:*|'#'*|'') continue ;;
    esac
    path="${target%%#*}"
    [ -z "$path" ] && continue
    if ! ( cd "$doc_dir" && [ -e "$path" ] ); then
      echo "BROKEN LINK in $d -> $target"
      fail=1
    fi
  done < <(grep -oE '\]\([^)]+\)' "$d" || true)
done

# Cited section anchors must be present in their target files.
anchor_present() { # file marker
  if [ ! -f "$1" ] || ! grep -qF "$2" "$1"; then
    echo "MISSING ANCHOR '$2' in $1"
    fail=1
  fi
}
anchor_present "docs/security/THREAT_MODEL.md" "## 16."
anchor_present "docs/rust/PATTERNS.md" "§P64"
anchor_present "docs/rust/PATTERNS.md" "§P65"
anchor_present "docs/rust/PATTERNS.md" "§P66"
anchor_present "docs/rust/PATTERNS.md" "§P67"
anchor_present "docs/rust/PATTERNS.md" "§P68"
anchor_present "docs/rust/PATTERNS.md" "§P69"
anchor_present "docs/rust/PATTERNS.md" "§P39"
anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "sbfb/shard/1"
anchor_present "crates/nexus-shell-daemon/src/http.rs" "shard-session"
anchor_present "web/src/api/daemon.ts" "getShardSession"
# S82 Phase G — the 3 control-plane request bodies are documented frontiers:
# the SPEC must cite each by name (§3 schema table for the two schematised
# ones, §6.1 Request-body tables for all three).
anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "ShardGroupMintRequest"
anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "MountSessionRequest"
anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "ShardGenerateRequest"
# S82 Phase T — the index layer (agent llms.txt + human REFERENCE and
# HOW_TO_WIRE) must ALSO cite the 3 request bodies. ROW/ENTRY-level
# anchors (not bare names) for llms + REFERENCE, so deleting the indexed
# Types-table row or the llms entry FAILS the gate even while the name
# survives elsewhere in prose (Codex T round-1 catch); HOW_TO_WIRE holds
# its only mention in the §START renvoi, so the bare name IS the entry.
# Without these anchors the "3 docs gates exit 0" machine criterion is
# non-discriminant: it was green before the index existed.
anchor_present "docs/sharding/llms.txt" "request bodies (S82 G):"
anchor_present "docs/sharding/llms.txt" "shard_group_mint_request.schema.json"
anchor_present "docs/sharding/llms.txt" "shard_generate_request.schema.json"
anchor_present "docs/sharding/REFERENCE.md" '| `ShardGroupMintRequest` |'
anchor_present "docs/sharding/REFERENCE.md" '| `MountSessionRequest` |'
anchor_present "docs/sharding/REFERENCE.md" '| `ShardGenerateRequest` |'
anchor_present "docs/sharding/HOW_TO_WIRE.md" "ShardGroupMintRequest"
anchor_present "docs/sharding/HOW_TO_WIRE.md" "MountSessionRequest"
anchor_present "docs/sharding/HOW_TO_WIRE.md" "ShardGenerateRequest"

# ── (2) honesty-gate ─────────────────────────────────────────────────
require_marker() { # file marker label
  if ! grep -qF "$2" "$1"; then
    echo "MISSING HONESTY MARKER [$3] in $1: '$2'"
    fail=1
  fi
}
for d in "${FR_DOCS[@]}"; do
  require_marker "$d" "S82" "residual-carry"
  require_marker "$d" "admission ≠ confidentialité" "cardinal-caveat"
done
require_marker "$DOCS_DIR/HOW_TO_WIRE.md" "S81" "orchestrator-shipped"
require_marker "$DOCS_DIR/REFERENCE.md" "S82-pending tuning" "threshold-tuning"

# ── (3) french-body ──────────────────────────────────────────────────
# Narrow English-UI word list — a BusyBox-safe subset of
# web/scripts/scan-en-strings.sh (the \b / \s* anchors are dropped for grep
# portability), applied to the 3 French docs only. REFERENCE.md is
# English-body by design and exempt.
EN_WORDS='(Welcome|Dashboard|Sign in|Log in|Sign up|Please|Click here|Coming soon|Loading\.\.\.)'
for d in "${FR_DOCS[@]}"; do
  if grep -qE "$EN_WORDS" "$d"; then
    echo "ENGLISH UI STRING in French doc $d:"
    grep -nE "$EN_WORDS" "$d" || true
    fail=1
  fi
done

# ── (4) Phase N — agent-consumable layer (llms.txt + WIRING_SPEC + examples) ──
# These docs are ENGLISH (agent-facing, like REFERENCE.md), so the french-body
# check does NOT apply. They get: existence + repo-relative link resolution + a
# source-ref-check (every cited `path:Symbol` resolves) + a Truth-Stack header
# assertion + the PROVISIONAL/S78 honesty markers.

WIRING_SPEC="$DOCS_DIR/WIRING_SPEC.md"
SHARD_LLMS="$DOCS_DIR/llms.txt"
ROOT_LLMS="llms.txt"
AGENT_DOCS=("$WIRING_SPEC" "$SHARD_LLMS" "$ROOT_LLMS")
EXAMPLE_DOCS=("$DOCS_DIR/examples/observe.curl.md" "$DOCS_DIR/examples/bridge_gap.md")
EXAMPLE_SRC="$DOCS_DIR/examples/sign_verify.rs"
EXAMPLE_TEST="crates/nexus-core-rs/tests/shard_sign_verify.rs"

for d in "${AGENT_DOCS[@]}" "${EXAMPLE_DOCS[@]}" "$EXAMPLE_SRC" "$EXAMPLE_TEST"; do
  if [ ! -f "$d" ]; then
    echo "MISSING PHASE-N FILE: $d"
    fail=1
  fi
done

# Repo-relative markdown links must resolve from the linking doc's own directory
# (same rule as the human docs above).
for d in "${AGENT_DOCS[@]}" "${EXAMPLE_DOCS[@]}"; do
  [ -f "$d" ] || continue
  doc_dir="$(dirname "$d")"
  while IFS= read -r link; do
    target="${link#](}"
    target="${target%)}"
    case "$target" in
      http://*|https://*|mailto:*|'#'*|'') continue ;;
    esac
    path="${target%%#*}"
    [ -z "$path" ] && continue
    if ! ( cd "$doc_dir" && [ -e "$path" ] ); then
      echo "BROKEN LINK in $d -> $target"
      fail=1
    fi
  done < <(grep -oE '\]\([^)]+\)' "$d" || true)
done

# source-ref-check: every backtick `path` or `path:Symbol` whose path is rank-1
# (crates/ docs/ web/ scripts/) must resolve — the file exists, and a non-numeric
# Symbol is grep-found in it (a numeric line stays within the file). `.planning/`
# is an in-flight pointer, NOT a rank-1 prefix, so it is never resolved here.
bt='`'
for d in "$WIRING_SPEC" "$SHARD_LLMS" "$ROOT_LLMS"; do
  [ -f "$d" ] || continue
  while IFS= read -r tok; do
    ref="${tok#$bt}"
    ref="${ref%$bt}"
    path="${ref%%:*}"
    if [ "$path" = "$ref" ]; then
      sym=""
    else
      sym="${ref#*:}"
    fi
    if [ ! -f "$path" ]; then
      echo "SOURCE-REF into the void in $d -> $ref (no file '$path')"
      fail=1
      continue
    fi
    [ -z "$sym" ] && continue
    case "$sym" in
      *[!0-9]*)
        if ! grep -qF "$sym" "$path"; then
          echo "SOURCE-REF symbol not found in $d -> $path:$sym"
          fail=1
        fi
        ;;
      *)
        lines="$(wc -l < "$path" | tr -d ' ')"
        if [ "$sym" -lt 1 ] || [ "$sym" -gt "$lines" ]; then
          echo "SOURCE-REF line out of range in $d -> $path:$sym ($lines lines)"
          fail=1
        fi
        ;;
    esac
  done < <(grep -oE "${bt}(crates|docs|web|scripts)/[^${bt}]+${bt}" "$d" || true)
done

# required-anchor check: the source-ref-check above validates refs that ARE
# present; this asserts that each load-bearing clause HAS one. Without it, a
# pillar clause that forgot its `path:Symbol` (e.g. the is_member-before-accept_bi
# ordering) would pass silently. The required symbols are the security/correctness
# pillars WIRING_SPEC must anchor; each must appear as the tail of a rank-1
# source-ref token in WIRING_SPEC.md.
REQUIRED_ANCHORS="is_pipeline_contiguous covers_full_model verify_signature \
DOMAIN_SHARD_PLAN_V1 DOMAIN_RUN_PROOF_V1 is_member authorize_claim accept_bi \
shard_session_response auth_required SHARD_PLAN_FORMAT_VERSION \
ShardGroupMintRequest MountSessionRequest ShardGenerateRequest"
wiring_symbols="$(grep -oE "${bt}(crates|docs|web|scripts)/[^${bt}]+${bt}" "$WIRING_SPEC" \
  | sed "s/.*://; s/${bt}//g")"
for req in $REQUIRED_ANCHORS; do
  if ! printf '%s\n' "$wiring_symbols" | grep -qx "$req"; then
    echo "MISSING REQUIRED source_ref '$req' in $WIRING_SPEC (load-bearing clause unanchored)"
    fail=1
  fi
done

# Truth-Stack authority header + the "Not evidenced" rank-1 rule must be present
# in the two agent contract files (consommée-jamais-autoritaire: the docs point,
# they never emit a PASS verdict).
TRUTH_STACK="repo files > .planning/active/ > commits > prompts > chat"
for d in "$WIRING_SPEC" "$SHARD_LLMS"; do
  anchor_present "$d" "$TRUTH_STACK"
  anchor_present "$d" "Not evidenced"
done

# honesty-gate extension (S81 Phase K requalification): the orchestrator +
# live benchmark shipped (S81 I/J), so the retired PROVISIONAL/S78 markers are
# replaced by the S82 RESIDUAL carry (per-worker proofs / dispute arbitration)
# — a real, still-open honesty clause, so a future flip to a "everything done,
# no caveats" reword still fails CI. The cardinal caveat "admission ≠
# confidentialité" is UNCHANGED (the confidentiality boundary is permanent).
# The root index pins its BOUNDED scope (not a whole-repo index; the factory
# section is gated by scripts/check-factory-docs.sh).
require_marker "$WIRING_SPEC" "S82" "residual-carry"
require_marker "$WIRING_SPEC" "admission ≠ confidentialité" "cardinal-caveat"
require_marker "$SHARD_LLMS" "S82" "llms-residual-carry"
require_marker "$ROOT_LLMS" "whole-repo agent index is" "root-scope-banner"

if [ "$fail" -ne 0 ]; then
  echo "check-sharding-docs: FAILED"
  exit 1
fi

echo "check-sharding-docs: clean (links + anchors + honesty + french-body + source-ref)"
exit 0
