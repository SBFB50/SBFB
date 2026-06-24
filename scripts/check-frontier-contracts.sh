#!/usr/bin/env bash
#
# Sprint 79 Phase B — frontier-contracts gate (generic, repo-wide).
#
# Canonises the docs-contract cadence in the SBFB process itself
# (docs/claude/README.md, docs/agent/AGENT_SYSTEM.md, docs/rust/PATTERNS.md
# §P70, doctrine .planning/research/doctrine_contrat_pour_llm.md §2/§7).
# Three deterministic checks (mirrors scripts/check-sharding-docs.sh
# discipline — BusyBox-safe so it also runs on the Woodpecker bash:5
# image: no grep -P, no --include, no \b, no \s, no mapfile/readarray):
#
#   (1) anti-promise source-ref — an in-code provenance comment must
#       point only at the IMMUTABLE PAST. A future promise anchored to a
#       phase / sprint / worker-wave ("lands in Phase K", "Phase C will
#       populate", "Sprint 2 will add", "W9 will layer") rots into a lie:
#       the STALE-PHASE-K anti-pattern (real S77 incident, http.rs). The
#       pattern is ANCHORED (a phase/sprint/wave token adjacent to a
#       future verb) so it never fires on generic prose ("the values the
#       consumer will read", "node A adds a blob", "a future sprint adds
#       a field"). Scanned over tracked crates/ + web/src/ source only;
#       docs/ (which describes the anti-pattern verbatim), this script,
#       vendored crates, target/ and node_modules/ are out of scope by
#       construction.
#   (2) frontier-tag coverage (opt-in, INCREMENTAL) — every type opted
#       in with "// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION"
#       must resolve its domain + version consts AND carry a generated
#       schema (schema_for!(<name>)) OR an explicit
#       "// FRONTIER-NO-SCHEMA: <name> <reason>" exemption. UNannotated
#       wire types are NOT violations — the registry is opt-in and grows
#       incrementally (the remaining DOMAIN_*_V1 families are a tracked carry,
#       routed to the next sprint's audit-plan, created at sprint closure).
#   (3) BLOB_SERVE_CSP non-regression — the canonical sandbox CSP
#       constant must keep every 'none' exfiltration directive. The two
#       existing Rust tests only assert a "connect-src 'none'" substring,
#       so a drift dropping form-action/base-uri would stay green. This
#       check asserts EACH of the 6 'none' directives against the const
#       source-of-truth (blob_serve.rs); the served-HTTP-header / vitrine
#       per-directive CSP gate is the Sprint 79 Phase E/H scope.
#
# Exit 0 = clean, exit 1 = at least one violation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

fail=0

# ── (1) anti-promise source-ref ──────────────────────────────────
PROMISE_RE='lands? (in )?Phase [A-Z0-9]|arrive(ra|nt|ront)? en Phase [A-Z0-9]|Phase [A-Z0-9]+ (will|adds|ships)|Sprint [0-9]+ will|S[0-9]+ will|W[0-9]+(\.[0-9]+)? (will|adds|ships|introduce)|When Sprint [0-9]+|inert until Phase|will land (in|with)'

while IFS= read -r f; do
  [ -f "$f" ] || continue
  hits="$(grep -nE "$PROMISE_RE" "$f" || true)"
  if [ -n "$hits" ]; then
    echo "STALE-PHASE-K: future-anchored provenance promise in $f"
    printf '%s\n' "$hits"
    echo "  -> rewrite to point at the immutable past (drop 'Phase X will/adds/ships', 'Sprint N will', 'lands in Phase')."
    fail=1
  fi
done < <(find crates web/src -type f \
  \( -name '*.rs' -o -name '*.toml' -o -name '*.sample' -o -name '*.ts' -o -name '*.tsx' \) \
  ! -path '*/llama.cpp/*' ! -path '*/target/*' ! -path '*/node_modules/*')

# ── (2) frontier-tag coverage (opt-in, incremental) ──────────────
frontier_count=0
while IFS= read -r line; do
  # line content: "...// FRONTIER: <name> domain=<D> version=<V>"
  body="${line##*// FRONTIER: }"
  name="${body%% *}"
  domain=""
  version=""
  # Intentional word-split on the annotation tokens.
  # shellcheck disable=SC2086
  for tok in $body; do
    case "$tok" in
      domain=*) domain="${tok#domain=}" ;;
      version=*) version="${tok#version=}" ;;
    esac
  done
  frontier_count=$((frontier_count + 1))
  if [ -z "$name" ] || [ -z "$domain" ] || [ -z "$version" ]; then
    echo "FRONTIER tag malformed (need '<name> domain=<D> version=<V>'): $line"
    fail=1
    continue
  fi
  if ! grep -rqF "const $domain" crates; then
    echo "FRONTIER '$name': domain const '$domain' not declared under crates/"
    fail=1
  fi
  if ! grep -rqF "const $version" crates; then
    echo "FRONTIER '$name': version const '$version' not declared under crates/"
    fail=1
  fi
  if grep -rF "schema_for!($name)" crates | grep -qvF "FRONTIER"; then
    : # a REAL schema_for! call (not the annotation line) — drift-gated by its snapshot test
  elif grep -rqF "// FRONTIER-NO-SCHEMA: $name" crates; then
    : # documented exemption
  else
    echo "FRONTIER '$name': no schema_for!($name) and no '// FRONTIER-NO-SCHEMA: $name' exemption"
    fail=1
  fi
done < <(find crates -type f -name '*.rs' ! -path '*/llama.cpp/*' ! -path '*/target/*' \
  -exec grep -hE '// FRONTIER: ' {} + 2>/dev/null || true)

# Anti silent-removal: the Phase B dogfood entry MUST stay registered so the
# coverage branch is never a permanent no-op (deleting the sole tag would
# otherwise leave the loop with 0 iterations and the gate green).
if ! grep -rqF "// FRONTIER: ShardPlan " crates; then
  echo "FRONTIER registry: required dogfood tag '// FRONTIER: ShardPlan' is missing"
  fail=1
fi

# ── (3) BLOB_SERVE_CSP non-regression ────────────────────────────
CSP_FILE="crates/nexus-shell-daemon-core/src/blob_serve.rs"
if [ ! -f "$CSP_FILE" ]; then
  echo "MISSING: $CSP_FILE (BLOB_SERVE_CSP source of truth)"
  fail=1
else
  csp_line="$(grep -E 'pub const BLOB_SERVE_CSP' "$CSP_FILE" || true)"
  if [ -z "$csp_line" ]; then
    echo "MISSING: BLOB_SERVE_CSP constant in $CSP_FILE"
    fail=1
  else
    for directive in \
      "connect-src 'none'" \
      "worker-src 'none'" \
      "frame-src 'none'" \
      "object-src 'none'" \
      "base-uri 'none'" \
      "form-action 'none'"; do
      case "$csp_line" in
        *"$directive"*) ;;
        *)
          echo "BLOB_SERVE_CSP regression: missing directive \"$directive\" in $CSP_FILE"
          echo "  -> the sandbox CSP must keep every 'none' exfiltration directive."
          fail=1
          ;;
      esac
    done
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "check-frontier-contracts: FAILED"
  exit 1
fi

echo "check-frontier-contracts: clean (anti-promise + frontier-tag coverage [$frontier_count tagged] + BLOB_SERVE_CSP non-regression)"
exit 0
