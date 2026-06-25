#!/usr/bin/env bash
#
# Sprint 79 Phase I — doc-lint gate for docs/factory/ (app-authoring capability).
#
# A factory-scoped clone of scripts/check-sharding-docs.sh, same BusyBox-safe
# discipline (also runs on the Woodpecker bash:5 image — no GNU-only grep
# features: no -P, no --include, no \b). The script is strictly grep/compare —
# it NEVER eval/source a .md (a doc must not be able to inject a command).
#
#   (1) link-check    — every repo-relative markdown link target resolves from
#                       the linking doc's directory; cited anchors are present
#                       in their files (FACTORY_GATES, PATTERNS §P71, csp.rs,
#                       gates.rs).
#   (2) honesty-gate  — README + EXPLANATION + HOW_TO_WIRE each carry the
#                       cardinal caveat ("lint statique ≠ garantie runtime"),
#                       "0 verdict PASS", and a PROVISIONAL marker; REFERENCE
#                       carries PROVISIONAL + "Not evidenced".
#   (3) french-body   — no English UI strings leak into the 3 French docs
#                       (REFERENCE.md is English-body by design, exempted).
#   (4) agent layer   — WIRING_SPEC + llms.txt: existence + repo-relative link
#                       resolution + source-ref-check (every cited `path:Symbol`
#                       resolves) + required-anchor allowlist + Truth-Stack
#                       header + "Not evidenced" + PROVISIONAL/cardinal markers;
#                       the root llms.txt pins its factory scope; the runnable
#                       example + its include! test exist.
#   (5) prompt-kind   — line-semantic source-ref: every bare-name ref in
#                       prompts/agent/app-authoring.md (PRIMITIVES.md:N /
#                       README.md:N, slash-lists + ranges) resolves to an
#                       in-bounds line of the matching animejs pack file. The
#                       deterministic hash+path volet is the generic
#                       scripts/check-frontier-contracts.sh (volet 4); this is
#                       the complementary line-existence check.
#
# Exit 0 = clean, exit 1 = at least one violation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

DOCS_DIR="docs/factory"
FR_DOCS=("$DOCS_DIR/README.md" "$DOCS_DIR/EXPLANATION.md" "$DOCS_DIR/HOW_TO_WIRE.md")
ALL_DOCS=("${FR_DOCS[@]}" "$DOCS_DIR/REFERENCE.md")

fail=0

# All four human docs must exist before any other check is meaningful.
for d in "${ALL_DOCS[@]}"; do
  if [ ! -f "$d" ]; then
    echo "MISSING DOC: $d"
    fail=1
  fi
done
if [ "$fail" -ne 0 ]; then
  echo "check-factory-docs: FAILED (missing docs)"
  exit 1
fi

# ── (1) link-check ───────────────────────────────────────────────────
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
anchor_present "docs/factory/FACTORY_GATES.md" "FG-CSP-authoring"
anchor_present "docs/rust/PATTERNS.md" "§P71"
anchor_present "crates/nexus-core-rs/src/csp.rs" "BLOB_SERVE_CSP"
anchor_present "crates/sbfb-factory/src/gates.rs" "run_gate_csp_authoring"

# ── (2) honesty-gate ─────────────────────────────────────────────────
require_marker() { # file marker label
  if ! grep -qF "$2" "$1"; then
    echo "MISSING HONESTY MARKER [$3] in $1: '$2'"
    fail=1
  fi
}
for d in "${FR_DOCS[@]}"; do
  require_marker "$d" "lint statique ≠ garantie runtime" "cardinal-caveat-clause1"
  require_marker "$d" "jamais autoritaire" "cardinal-caveat-clause2"
  require_marker "$d" "0 verdict PASS" "no-pass-verdict"
  require_marker "$d" "PROVISIONAL" "provisional-banner"
done
require_marker "$DOCS_DIR/REFERENCE.md" "PROVISIONAL" "reference-provisional"
require_marker "$DOCS_DIR/REFERENCE.md" "Not evidenced" "reference-not-evidenced"

# ── (3) french-body ──────────────────────────────────────────────────
# Narrow English-UI word list — a BusyBox-safe subset of
# web/scripts/scan-en-strings.sh, applied to the 3 French docs only.
# REFERENCE.md is English-body by design and exempt.
EN_WORDS='(Welcome|Dashboard|Sign in|Log in|Sign up|Please|Click here|Coming soon|Loading\.\.\.)'
for d in "${FR_DOCS[@]}"; do
  if grep -qE "$EN_WORDS" "$d"; then
    echo "ENGLISH UI STRING in French doc $d:"
    grep -nE "$EN_WORDS" "$d" || true
    fail=1
  fi
done

# ── (4) agent-consumable layer (llms.txt + WIRING_SPEC + example) ─────
WIRING_SPEC="$DOCS_DIR/WIRING_SPEC.md"
FACTORY_LLMS="$DOCS_DIR/llms.txt"
ROOT_LLMS="llms.txt"
AGENT_DOCS=("$WIRING_SPEC" "$FACTORY_LLMS")
EXAMPLE_SRC="$DOCS_DIR/examples/csp_contract.rs"
EXAMPLE_TEST="crates/nexus-core-rs/tests/factory_csp_contract.rs"

for d in "${AGENT_DOCS[@]}" "$EXAMPLE_SRC" "$EXAMPLE_TEST"; do
  if [ ! -f "$d" ]; then
    echo "MISSING PHASE-I FILE: $d"
    fail=1
  fi
done

# Repo-relative markdown links in the factory agent docs must resolve from the
# linking doc's own directory (same rule as the human docs). The root llms.txt
# is link-checked by scripts/check-sharding-docs.sh; here we only assert its
# factory scope marker (below).
for d in "${AGENT_DOCS[@]}"; do
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
# is an in-flight pointer, NOT rank-1, so it is never resolved here.
bt='`'
for d in "${AGENT_DOCS[@]}"; do
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
# present; this asserts each load-bearing pillar HAS one. The required symbols
# are the factory primitives WIRING_SPEC must anchor; each must appear as the
# tail of a rank-1 source-ref token in WIRING_SPEC.md.
REQUIRED_ANCHORS="BLOB_SERVE_CSP none_directives CSS_URL_ALLOW run_gate_csp_authoring \
PROMPT_KINDS app-authoring authoring_knowledge handle_context_pack TemplateConfig"
wiring_symbols="$(grep -oE "${bt}(crates|docs|web|scripts)/[^${bt}]+${bt}" "$WIRING_SPEC" \
  | sed "s/.*://; s/${bt}//g")"
for req in $REQUIRED_ANCHORS; do
  if ! printf '%s\n' "$wiring_symbols" | grep -qx "$req"; then
    echo "MISSING REQUIRED source_ref '$req' in $WIRING_SPEC (load-bearing clause unanchored)"
    fail=1
  fi
done

# Truth-Stack authority header + the "Not evidenced" rank-1 rule must be present
# in the two factory agent contract files (consommée-jamais-autoritaire: the docs
# point, they never emit a PASS verdict).
TRUTH_STACK="repo files > .planning/active/ > commits > prompts > chat"
for d in "$WIRING_SPEC" "$FACTORY_LLMS"; do
  anchor_present "$d" "$TRUTH_STACK"
  anchor_present "$d" "Not evidenced"
done

# honesty-gate extension: the agent wiring spec + factory index carry the
# PROVISIONAL marker and the cardinal caveat (so a future flip to a
# "shipped/done" banner fails CI); the root index pins its factory scope (so a
# silent drop of the factory section fails CI).
require_marker "$WIRING_SPEC" "PROVISIONAL" "wiring-provisional"
require_marker "$WIRING_SPEC" "lint statique ≠ garantie runtime" "wiring-cardinal-caveat-clause1"
require_marker "$WIRING_SPEC" "jamais autoritaire" "wiring-cardinal-caveat-clause2"
require_marker "$FACTORY_LLMS" "PROVISIONAL" "llms-provisional"
require_marker "$FACTORY_LLMS" "lint statique ≠ garantie runtime" "llms-cardinal-caveat-clause1"
require_marker "$FACTORY_LLMS" "jamais autoritaire" "llms-cardinal-caveat-clause2"
require_marker "$ROOT_LLMS" "app-authoring (factory)" "root-factory-scope"

# ── (5) prompt-kind line-semantic source-ref (P2 carry, Phase C) ─────
# The app-authoring fiche cites bare-filename refs (PRIMITIVES.md:N / README.md:N,
# slash-lists like 112/627 and ranges like 1107-1179) RELATIVE to the animejs
# knowledge pack — NOT the path:Symbol rank-1 form. The deterministic hash+path
# volet is the generic scripts/check-frontier-contracts.sh (volet 4); here we add
# the complementary check that every cited line number is in-bounds of the
# matching pack file. (Whether the line still SUPPORTS the claim is an adversarial
# LLM review, not a deterministic gate.)
FICHE="prompts/agent/app-authoring.md"
# Bare-name refs (PRIMITIVES.md / README.md) in the fiche are animejs-pack at this
# revision (the daisyUI section cites FULL paths, never bare PRIMITIVES.md:N). If a
# future fiche adds bare daisyUI line-refs, extend this resolution to the right pack.
PACK_DIR="docs/factory/knowledge/animejs"
if [ -f "$FICHE" ]; then
  while IFS= read -r tok; do
    fname="${tok%%:*}"
    nums="${tok#*:}"
    packfile="$PACK_DIR/$fname"
    if [ ! -f "$packfile" ]; then
      echo "FICHE-REF into the void in $FICHE -> $tok (no pack file '$packfile')"
      fail=1
      continue
    fi
    pl="$(wc -l < "$packfile" | tr -d ' ')"
    nums_spaced="$(printf '%s' "$nums" | tr '/' ' ' | tr '-' ' ')"
    for n in $nums_spaced; do
      case "$n" in
        ''|*[!0-9]*) continue ;;
      esac
      if [ "$n" -lt 1 ] || [ "$n" -gt "$pl" ]; then
        echo "FICHE-REF line out of range in $FICHE -> $packfile:$n ($pl lines)"
        fail=1
      fi
    done
  done < <(grep -oE '(PRIMITIVES|README)\.md:[0-9][0-9/-]*' "$FICHE" || true)
fi

if [ "$fail" -ne 0 ]; then
  echo "check-factory-docs: FAILED"
  exit 1
fi

echo "check-factory-docs: clean (links + anchors + honesty + french-body + source-ref + fiche-lines)"
exit 0
