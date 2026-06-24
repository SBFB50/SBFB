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
#                      PROVISIONAL marker and the cardinal caveat
#                      "admission ≠ confidentialité"; HOW_TO_WIRE names the
#                      S78 orchestrator carry; REFERENCE marks its thresholds
#                      "S78-pending tuning".
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

# ── (2) honesty-gate ─────────────────────────────────────────────────
require_marker() { # file marker label
  if ! grep -qF "$2" "$1"; then
    echo "MISSING HONESTY MARKER [$3] in $1: '$2'"
    fail=1
  fi
}
for d in "${FR_DOCS[@]}"; do
  require_marker "$d" "PROVISIONAL" "provisional-banner"
  require_marker "$d" "admission ≠ confidentialité" "cardinal-caveat"
done
require_marker "$DOCS_DIR/HOW_TO_WIRE.md" "S78" "orchestrator-carry"
require_marker "$DOCS_DIR/REFERENCE.md" "S78-pending tuning" "threshold-tuning"

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

if [ "$fail" -ne 0 ]; then
  echo "check-sharding-docs: FAILED"
  exit 1
fi

echo "check-sharding-docs: clean (links + anchors + honesty + french-body)"
exit 0
