#!/usr/bin/env bash
#
# Sprint 79 Phase B — frontier-contracts gate (generic, repo-wide).
#
# Canonises the docs-contract cadence in the SBFB process itself
# (docs/claude/README.md, docs/agent/AGENT_SYSTEM.md, docs/rust/PATTERNS.md
# §P70, doctrine .planning/research/doctrine_contrat_pour_llm.md §2/§7).
# Deterministic checks (mirrors scripts/check-sharding-docs.sh
# discipline — BusyBox-safe so it also runs on the Woodpecker bash:5
# image: no grep -P, no --include, no \b, no \s, no mapfile/readarray):
#
#   (1) anti-promise source-ref — an in-code provenance comment must
#       point only at the IMMUTABLE PAST. A future promise anchored to a
#       phase / sprint / worker-wave ("lands in Phase K", "Phase C will
#       populate", "Sprint 2 will add", "W9 will layer", "until Sprint N
#       activates", "when SN lands", "the SN+ allow-list") rots into a
#       lie: the STALE-PHASE-K anti-pattern (real S77 incident, http.rs).
#       The pattern is ANCHORED (a phase/sprint/wave token adjacent to a
#       future verb or capability noun) so it never fires on generic
#       prose ("the values the consumer will read", "node A adds a blob",
#       "a future sprint adds a field") nor on past narration ("Sprint 20
#       ships only one schema identity"). Known residual: the
#       sandbox/allow-list/activates branches are not tense-anchored, so
#       future HISTORICAL prose like "the Sprint 22 sandbox was added"
#       would flag — acceptable, any "Sprint N sandbox" adjacency
#       deserves a look; reviewers arbitrate. Likewise a bare NON-sprint
#       S<digit> token would flag ("until S3 responds" [AWS], "S5
#       activates" [section label]) — no such usage exists in this repo,
#       where S<n> always reads as Sprint. grep -E is line-oriented:
#       only same-line token+verb forms are detectable; split-line forms
#       are caught by the verb-free token branch "until (the )?Sprint".
#       A PROMISE_RE self-test (non-vacuity + anchoring) runs before the
#       scan and fails the gate if the motif rots. Scanned over tracked
#       crates/ + web/src/ source only; docs/ (which describes the
#       anti-pattern verbatim), this script, vendored crates, target/
#       and node_modules/ are out of scope by construction.
#   (2) frontier-tag coverage (opt-in, INCREMENTAL) — every type opted
#       in with "// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION"
#       must resolve its domain + version consts AND carry a generated
#       schema (schema_for!(<name>)) OR an explicit
#       "// FRONTIER-NO-SCHEMA: <name> <reason>" exemption. UNannotated
#       wire types are NOT violations — the registry is opt-in and grows
#       incrementally. The backlog is accept-and-closed (S82 Phase G, D8):
#       the DOMAIN_*_V1 family census is FROZEN by check (2b) below — 29
#       named families, 25 without a generated schema — and a NEW family must
#       make its own conscious schema/no-schema decision instead of the
#       count drifting silently (no exhaustive tagging of the backlog).
#   (3) BLOB_SERVE_CSP non-regression — the canonical sandbox CSP
#       constant must keep every 'none' exfiltration directive. The two
#       existing Rust tests only assert a "connect-src 'none'" substring,
#       so a drift dropping form-action/base-uri would stay green. This
#       check asserts EACH of the 6 'none' directives against the const
#       source-of-truth. At S79 Phase E the const was factored out of
#       blob_serve.rs into crates/nexus-core-rs/src/csp.rs (single source,
#       re-exported by blob_serve.rs for the daemon); this gate follows the
#       declaration to its new home. The per-directive STATIC ASSET gate now
#       exists (sbfb-factory run_gate_csp_authoring, S79 Phase E); the runtime
#       served-header self-check is S79 Phase H.
#   (4) prompt-kind provenance edge — a knowledge-backed prompt-kind
#       fiche (prompts/agent/*.md that references docs/factory/knowledge/)
#       must keep its copied provenance resolvable: every inline blake3
#       16-hex digest must equal a value recorded in some
#       docs/factory/knowledge/*/MANIFEST.json (a copied digest that rots
#       silently when the pack is re-extracted is a lie — the "GUIDE non
#       gate" gap surfaced by the sprint79 doc-verification), and every
#       cited docs/factory/knowledge/... layer path must exist on disk.
#       The semantic "the cited line still supports the claim" check stays
#       the adversarial LLM review's job (companion); this gate guards the
#       path + hash drift only. Generic: any future prompt-kind is covered.
#
# Exit 0 = clean, exit 1 = at least one violation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

fail=0

# ── (1) anti-promise source-ref ──────────────────────────────────
# S82 Phase F broadened the motif with the "until/when Sprint N
# activates/lands" class (carry S79-P2-1 / S80-G-2): 4 new branches —
# "until (the )?(Sprint |S)N" (verb-free token, also catches the
# split-line form), "[Ww]hen (Sprint |S)N lands|activates|ships",
# "(Sprint |S)N+? sandbox|allow-list", "(Sprint |S)N+? activates".
# lands/ships stay [Ww]hen-anchored: a bare "(Sprint |S)N (lands|ships)"
# branch false-positives on past narration ("Sprint 20 ships only...").
PROMISE_RE='lands? (in )?Phase [A-Z0-9]|arrive(ra|nt|ront)? en Phase [A-Z0-9]|Phase [A-Z0-9]+ (will|adds|ships)|Sprint [0-9]+ will|S[0-9]+ will|W[0-9]+(\.[0-9]+)? (will|adds|ships|introduce)|When Sprint [0-9]+|inert until Phase|will land (in|with)|until (the )?(Sprint |S)[0-9]|[Ww]hen (Sprint |S)[0-9]+ (lands|activates|ships)|(Sprint |S)[0-9]+\+? (sandbox|allow-list)|(Sprint |S)[0-9]+\+? activates'

# Self-test: PROMISE_RE must stay non-vacuous AND anchored (mirrors the
# FRONTIER ShardPlan anti-silent-removal guard below). The scan loop
# wraps grep in `|| true`, so a malformed motif (grep exit 2) would
# otherwise silently green the whole anti-promise check. Assertions run
# inside `if` so a malformed regex fails loudly here (exit 2 != 0 ->
# "vacuous or malformed") instead of aborting the gate via set -e.
# Fixtures are single-line by design (grep -E is line-oriented). The
# four positive fixtures each match EXACTLY ONE S82 branch (until-token
# / when-verb / capability-noun / bare-activates), so silently deleting
# any single new branch fails the self-test (no overlap: the until
# fixture carries no verb, the activates fixture no until/when/noun).
_promise_neg='the values the consumer will read once a future sprint adds a field'
for _promise_pos in \
  'promise: tool_calls stay inert until Sprint 22' \
  'promise: the schema does not bump when S25 lands' \
  'promise: match the name against the S25+ allow-list' \
  'promise: S25 activates the pump'; do
  if ! printf '%s\n' "$_promise_pos" | grep -qE "$PROMISE_RE"; then
    echo "PROMISE_RE self-test: vacuous or malformed (positive fixture not detected: $_promise_pos)"
    fail=1
  fi
done
if printf '%s\n' "$_promise_neg" | grep -qE "$PROMISE_RE"; then
  echo "PROMISE_RE self-test: over-broad (anchored negative fixture matched)"
  fail=1
fi

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
# Same rule for the signed pack (S83 Phase F): the tag count is not frozen, so
# without this pin, dropping the pack's tag would shrink the registry silently.
if ! grep -rqF "// FRONTIER: SbfbPromptAgentPack " crates; then
  echo "FRONTIER registry: required tag '// FRONTIER: SbfbPromptAgentPack' is missing"
  fail=1
fi

# ── (2c) pack JCS parity mirror existence (S83 Phase F) ───────────
# The Rust side of the cross-language parity claim stays green if the
# TypeScript mirror is deleted — the crate tests never open web/. Pin the
# mirror's existence and its two fixture reads here, the bridge-parity
# arrangement: the claim "byte-identical cross-language" must not be able to
# outlive the file that measures its other half.
pack_mirror="web/src/workflow/__tests__/pack_digest.test.ts"
if [ ! -f "$pack_mirror" ]; then
  echo "PACK-PARITY: TypeScript mirror $pack_mirror is missing"
  fail=1
elif ! grep -qF "pack_jcs_vectors.json" "$pack_mirror" \
  || ! grep -qF "pack_jcs_digests.golden.json" "$pack_mirror"; then
  echo "PACK-PARITY: $pack_mirror no longer reads the shared vectors + golden"
  fail=1
fi

# ── (2b) DOMAIN_*_V1 frozen family census (S82 Phase G, D8; S83 F) ─
# The census is a frozen NAME LIST since S83 Phase F (it was a bare count
# before, which a net-zero add+remove could fool — docs/rust/PATTERNS.md
# §P70): 29 distinct `const DOMAIN_*_V<n>` families across crates/ (27 in
# canonical.rs + DOMAIN_KEYSTORE_V1 + DOMAIN_TRACE_EVENT_V1), of which 4
# carry a generated schema (COMPUTE_GROUP, SHARD_PLAN, RUN_PROOF,
# WORKFLOW_PACK) -> 25 unschematised: 22 accept-and-closed (S82 G) + 3
# RESERVED tags with no payload yet (COMPILED_PLAN / RUN_EVENT /
# PACK_INSTALL_RECEIPT, S83 F, D8 rationale = no-schema until their
# contract identities are filled — the DOMAIN_KUDOS_V1 arrangement).
# Nuance since S83 Phase H: the CompiledPlan.v1 CONTRACT identity is now
# FILLED (compile.rs, schema in contracts.schema.json), but the plan stays
# a LOCAL derived artefact — DOMAIN_COMPILED_PLAN_V1 remains a RESERVED
# SIGNING tag with zero producers, so the NAME LIST is unchanged.
# A NEW family must make a conscious D8 decision (schema_for! + snapshot,
# or a motivated no-schema rationale) and add its NAME here — silent
# census growth, removal or rename is exactly what this tripwire catches.
# Honest bound: a commented-out `const DOMAIN_...` line still matches the
# grep; catching that class stays with the per-sprint review of canonical.rs.
# S83 Phase F took the census 25 -> 29 (WORKFLOW_PACK filled + 3 reserved).
DOMAIN_CENSUS_FROZEN="DOMAIN_ACTIVATION_COMMIT_V1
DOMAIN_AGE_WITNESS_V1
DOMAIN_CLAIM_V1
DOMAIN_COMPILED_PLAN_V1
DOMAIN_COMPUTE_GROUP_V1
DOMAIN_CONTRIBUTOR_ATTESTATION_V1
DOMAIN_CURATOR_LIST_V1
DOMAIN_DELEGATION_CERT_V1
DOMAIN_DURESS_ACK_V1
DOMAIN_FEED_V1
DOMAIN_INVITE_V1
DOMAIN_KEYSTORE_V1
DOMAIN_KEY_ROTATION_V1
DOMAIN_KUDOS_V1
DOMAIN_NODE_DIRECTORY_V1
DOMAIN_PACK_INSTALL_RECEIPT_V1
DOMAIN_POW_V1
DOMAIN_PROVENANCE_V1
DOMAIN_RESULT_V1
DOMAIN_RUN_EVENT_V1
DOMAIN_RUN_PROOF_V1
DOMAIN_SEED_REQUEST_V1
DOMAIN_SEED_RESPONSE_V1
DOMAIN_SHARD_PLAN_V1
DOMAIN_TASK_V1
DOMAIN_TRACE_EVENT_V1
DOMAIN_VRF_DRAW_V1
DOMAIN_WARRANT_CANARY_V1
DOMAIN_WORKFLOW_PACK_V1"
domain_census_live="$({ find crates -type f -name '*.rs' ! -path '*/llama.cpp/*' ! -path '*/target/*' \
  -exec grep -hoE 'const DOMAIN_[A-Z0-9_]+_V[0-9]+' {} + 2>/dev/null || true; } | sed 's/^const //' | LC_ALL=C sort -u)"
domain_census="$(printf '%s\n' "$domain_census_live" | grep -c . || true)"
if [ "$domain_census_live" != "$DOMAIN_CENSUS_FROZEN" ]; then
  echo "DOMAIN_*_V1 census drift: live registry does not match the frozen name list"
  diff <(printf '%s\n' "$DOMAIN_CENSUS_FROZEN") <(printf '%s\n' "$domain_census_live") | sed 's/^/  /' || true
  echo "  -> a new/renamed/removed DOMAIN family needs its own D8 decision (generated schema or"
  echo "     motivated no-schema), then refresh the frozen NAME LIST + the census prose in this"
  echo "     file (header (2), comment (2b)) + docs/rust/PATTERNS.md §P70."
  fail=1
fi

# ── (3) BLOB_SERVE_CSP non-regression ────────────────────────────
CSP_FILE="crates/nexus-core-rs/src/csp.rs"
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

# ── (4) prompt-kind provenance edge (knowledge-backed fiches) ─────
# Closes the "GUIDE non gate" gap (sprint79 doc-verification): the generic
# frontier gate excluded prompts/, so a fiche's copied pack digests and
# layer paths were ungated and could rot silently at the next pack rotation.
KNOW_DIR="docs/factory/knowledge"
if [ -d "$KNOW_DIR" ] && [ -d "prompts/agent" ]; then
  # Union of every blake3 16-hex digest recorded across all pack MANIFESTs.
  # `-h` is load-bearing: with >=2 packs `grep` over multiple files prefixes each
  # match with `filename:`, which would defeat the whole-line `grep -qxF` below and
  # flag EVERY cited hash as absent. (Latent until a 2nd pack exists; daisyui = S79 F.)
  manifest_hashes="$(find "$KNOW_DIR" -name MANIFEST.json -exec grep -hoE '[0-9a-f]{16}' {} + 2>/dev/null | sort -u || true)"
  for pf in prompts/agent/*.md; do
    [ -f "$pf" ] || continue
    # Only knowledge-backed fiches are in scope (they reference the pack dir).
    grep -qF "$KNOW_DIR" "$pf" || continue
    # 4a — every inline 16-hex digest must be a known pack digest.
    # CONVENTION: inside a knowledge-backed fiche EVERY lowercase 16-hex token
    # is treated as a pack digest — do not embed an unrelated 16-hex identifier
    # (e.g. a git SHA prefix) in such a fiche, or it will be flagged here.
    # Intentional word-split over the unique hash list.
    # shellcheck disable=SC2046
    for h in $(grep -oE '[0-9a-f]{16}' "$pf" | sort -u || true); do
      if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
        echo "PROMPT-PROVENANCE: $pf cites blake3 16-hex '$h' absent from every $KNOW_DIR/*/MANIFEST.json"
        echo "  -> a copied pack digest rots when the pack is re-extracted; re-sync the fiche to MANIFEST.json or drop the hash."
        fail=1
      fi
    done
    # 4b — every cited knowledge layer path must exist on disk.
    # Intentional word-split over the unique path list.
    # shellcheck disable=SC2046
    for kp in $(grep -oE "$KNOW_DIR/[A-Za-z0-9_./-]+\.(json|md|ts)" "$pf" | sort -u || true); do
      if [ ! -f "$kp" ]; then
        echo "PROMPT-PROVENANCE: $pf cites '$kp' which does not exist on disk"
        echo "  -> a moved/removed knowledge layer; re-anchor the path."
        fail=1
      fi
    done
  done
fi

# ── (5) bridge method parity (Sprint 83 Phase C) ──────────────────
# As of S83 Phase C the manifest allowlist (Rust `BRIDGE_METHOD_ALLOWLIST`) and the
# host dispatch schema (TS `BridgeMethodSchema`) are the release-declaration layer of
# the per-call authority intersection: an app that declares a `bridge` config is
# FAIL-CLOSED on its declared set. Parity is therefore a SECURITY invariant, not a
# courtesy — a drift lets a release declare a method the host will not dispatch, or
# the host dispatch a method no release can name. Derived from the two canonical
# sources (no 4th synced copy). BusyBox-safe: index()-anchored awk, grep -oE (ERE).
PROTOCOL_TS="web/src/bridge/protocol.ts"
MANIFEST_RS="crates/sbfb-manifest/src/lib.rs"
if [ ! -f "$PROTOCOL_TS" ] || [ ! -f "$MANIFEST_RS" ]; then
  echo "BRIDGE-PARITY: missing $PROTOCOL_TS or $MANIFEST_RS"
  fail=1
else
  # TS: quoted lowercase tokens inside `BridgeMethodSchema = z.enum([ ... ]);`.
  ts_methods="$(awk 'index($0,"BridgeMethodSchema")>0 && index($0,"z.enum([")>0 {f=1;next} f && index($0,"]);")>0 {f=0} f {print}' "$PROTOCOL_TS" | grep -oE '"[a-z0-9_]+"' | tr -d '"' | sort -u)"
  # Rust: quoted tokens inside `BRIDGE_METHOD_ALLOWLIST: &[&str] = &[ ... ];`
  # (the `EXPECTED` test copy is skipped — its start line carries no allowlist name).
  rs_methods="$(awk 'index($0,"BRIDGE_METHOD_ALLOWLIST")>0 && index($0,"&[")>0 {f=1;next} f && index($0,"];")>0 {f=0} f {print}' "$MANIFEST_RS" | grep -oE '"[a-z0-9_]+"' | tr -d '"' | sort -u)"
  ts_count="$(printf '%s\n' "$ts_methods" | grep -c . || true)"
  rs_count="$(printf '%s\n' "$rs_methods" | grep -c . || true)"
  # Self-test (committed non-vacuity check): dropping one extracted method MUST change
  # the set, proving the `!=` comparison below is discriminant (not vacuously equal on
  # empty/garbled extraction). It proves the COMPARATOR is non-vacuous — not that the
  # extractor catches every possible name shape (the regex bounds names to [a-z0-9_]).
  if [ -n "$ts_methods" ] && [ "$ts_count" -ge 2 ]; then
    _bp_mut="$(printf '%s\n' "$ts_methods" | tail -n +2)"
    if [ "$_bp_mut" = "$ts_methods" ]; then
      echo "BRIDGE-PARITY self-test: a one-method drop did not change the set (vacuous comparison)"
      fail=1
    fi
  fi
  if [ -z "$ts_methods" ] || [ -z "$rs_methods" ]; then
    echo "BRIDGE-PARITY: could not extract methods (TS=$ts_count RS=$rs_count) — check the enum/allowlist markers"
    fail=1
  elif [ "$ts_methods" != "$rs_methods" ]; then
    echo "BRIDGE-PARITY drift: TS BridgeMethodSchema ($ts_count) != Rust BRIDGE_METHOD_ALLOWLIST ($rs_count)"
    echo "  TS methods:   $(printf '%s' "$ts_methods" | tr '\n' ' ')"
    echo "  Rust methods: $(printf '%s' "$rs_methods" | tr '\n' ' ')"
    echo "  -> update BOTH web/src/bridge/protocol.ts and crates/sbfb-manifest/src/lib.rs in lockstep."
    fail=1
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "check-frontier-contracts: FAILED"
  exit 1
fi

echo "check-frontier-contracts: clean (anti-promise + frontier-tag coverage [$frontier_count tagged] + DOMAIN census [$domain_census frozen] + BLOB_SERVE_CSP non-regression + prompt-kind provenance + bridge-parity [${ts_count:-?} methods] + pack-parity mirror)"
exit 0
