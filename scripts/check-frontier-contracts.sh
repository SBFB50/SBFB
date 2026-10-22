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
#       the DOMAIN_*_V1 family census is FROZEN by check (2b) below — 31
#       named families, 27 without a generated schema — and a NEW family must
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
  # Strip CR before word-splitting. `.gitattributes` forces LF repo-wide, so a
  # CI checkout is always LF — but a Windows working tree can hold CRLF files
  # (three `sbfb-workflow-core` sources did, S83 L1 round 9). Without this, the
  # LAST token carries a trailing `\r`, so `grep -F "const PACK_FORMAT_VERSION\r"`
  # misses and the leg reports "version const not declared under crates/" for a
  # const that IS declared — a red that names the wrong cause, which is worse
  # than no red at all. Measured under Linux against a CRLF `pack.rs`.
  body="$(printf '%s' "$body" | tr -d '\r')"
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
# §P70): 31 distinct `const DOMAIN_*_V<n>` families across crates/ (29 in
# canonical.rs + DOMAIN_KEYSTORE_V1 + DOMAIN_TRACE_EVENT_V1), of which 4
# carry a generated schema (COMPUTE_GROUP, SHARD_PLAN, RUN_PROOF,
# WORKFLOW_PACK) -> 27 unschematised: 22 accept-and-closed (S82 G) + 2 chat
# v1 (CHAT_INVITE / CHAT_ENVELOPE, 2026-10-21, D8 = no-schema: app-level
# payloads with ONE producer/consumer crate, nexus-messaging-core, pinned by
# its round-trip + tamper tests; not a network contract other runtimes read) + 3
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
# S83 Phase F took the census 25 -> 29 (WORKFLOW_PACK filled + 3 reserved);
# chat v1 Phase B took it 29 -> 31 (CHAT_INVITE + CHAT_ENVELOPE, no-schema).
DOMAIN_CENSUS_FROZEN="DOMAIN_ACTIVATION_COMMIT_V1
DOMAIN_AGE_WITNESS_V1
DOMAIN_CHAT_ENVELOPE_V1
DOMAIN_CHAT_INVITE_V1
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
  # SCOPE of this leg, restored after being dropped by mistake (review r5
  # L1-R5-D5-5 — the same diff added a scope paragraph to HITL-PARITY and
  # deleted this one, on the leg that needs it MORE: unlike HITL, this one
  # guards a WIRED surface, so no DS-1 bound caps it).
  #  - Names are bounded to `[a-z0-9_]` on BOTH sides. A method named
  #    `taskSubmitV2` or `task-submit-v2` is invisible to both extractions, so a
  #    REAL drift between the mirrors stays GREEN (measured). Only snake_case
  #    names are actually compared.
  #  - It is a parity of NAMES: payload schemas are not compared at all.
  #
  # ONE definition of the extraction, used by BOTH the real run and the
  # self-test below. Duplicating the pipeline into the self-test would mean the
  # self-test validates a COPY: loosen the real regex and the copy stays green.
  _bp_extract_ts() {
    awk 'index($0,"BridgeMethodSchema")>0 && index($0,"z.enum([")>0 {f=1;next} f && index($0,"]);")>0 {f=0} f {print}' \
      | grep -oE '"[a-z0-9_]+"' | tr -d '"' | sort -u
  }
  # TS: quoted lowercase tokens inside `BridgeMethodSchema = z.enum([ ... ]);`.
  # `|| true` on ALL SIX command substitutions that run an extraction pipeline:
  # the four real assignments (`ts_methods`, `rs_methods`, `hitl_rs_fields`,
  # `hitl_ts_fields`) AND the two SELF-TESTS (`_bp_got`, `_hp_got`). Without it,
  # an empty extraction makes the terminal `grep -oE` exit 1, `pipefail`
  # propagates and `set -e` aborts the script BEFORE the guard can name the leg
  # — exit 1 with zero output. Round 7 fixed the four assignments and left the
  # two self-tests aborting mute; round 8 measured that and closed the CLASS.
  # Any NEW extraction added below must carry it too.
  ts_methods="$(_bp_extract_ts < "$PROTOCOL_TS" || true)"
  # Rust: quoted tokens inside `BRIDGE_METHOD_ALLOWLIST: &[&str] = &[ ... ];`
  # (the `EXPECTED` test copy is skipped — its start line carries no allowlist name).
  rs_methods="$(awk 'index($0,"BRIDGE_METHOD_ALLOWLIST")>0 && index($0,"&[")>0 {f=1;next} f && index($0,"];")>0 {f=0} f {print}' "$MANIFEST_RS" | grep -oE '"[a-z0-9_]+"' | tr -d '"' | sort -u || true)"
  ts_count="$(printf '%s\n' "$ts_methods" | grep -c . || true)"
  rs_count="$(printf '%s\n' "$rs_methods" | grep -c . || true)"
  # Self-test of the EXTRACTOR against a reference fixture.
  #
  # The previous version compared the set to `tail -n +2` of ITSELF, which
  # differs by construction as soon as the set has two elements — the failure
  # branch was unreachable and the block proved nothing while being cited as
  # evidence (ex-review r3 L1-D5-5). What this replacement adds, precisely: an
  # extractor that returns the WRONG set while still returning SOMETHING — a
  # loosened anchor that starts matching doc lines, a regex that swallows a
  # neighbouring token. The `-z` guard below covers the empty case — and it is
  # REACHABLE only since review round 7 added `|| true` to the four extraction
  # assignments; before that, errexit aborted the script first and the guard
  # was dead code. Mutation-tested both ways. It still
  # cannot see a non-empty but wrong extraction, and neither could the old
  # block. Claiming more than that would repeat the defect being fixed.
  # The fixture is built to KILL mutations, not to look plausible (review r5
  # L1-R5-D5-1: the first version's decoy carried no lowercase quoted token, so
  # six mutations of the pipeline survived it — including the two its own
  # comment claimed to catch). Each element below kills one:
  #  - quoted tokens BEFORE and AFTER the range  -> a loosened start/end anchor
  #    starts capturing them. NOT `next`: dropping it was re-measured in review
  #    round 6 and SURVIVES, because `next` only matters when the OPENING line
  #    itself carries a quoted token, which neither this fixture nor
  #    `protocol.ts:20` does. The claim was here for two rounds without being
  #    replayed — the exact defect this block exists to prevent;
  #  - a second `BridgeMethodSchema` mention without `z.enum([`  -> a range
  #    restart;
  #  - `"Zulu-Cap"`, which the `[a-z0-9_]` class must NOT match -> a widened
  #    regex captures it;
  #  - `zzz_last` declared BEFORE `aaa_first` -> a dropped `sort` changes the
  #    output order;
  #  - `aaa_first` declared TWICE -> a `sort` that lost its `-u` emits it twice.
  _bp_fixture='const decoyBefore = "before_token";
// BridgeMethodSchema named here with NO enum opener — the trap for a start
// anchor that stops requiring `z.enum([`.
const trapToken = "trap_token";
const BridgeMethodSchema = z.enum([
  "zzz_last",
  "Zulu-Cap",
  "aaa_first",
  "aaa_first",
]);
const decoyAfter = "after_token";'
  _bp_got="$(printf '%s\n' "$_bp_fixture" | _bp_extract_ts | tr '\n' ' ' || true)"
  if [ "$_bp_got" != "aaa_first zzz_last " ]; then
    echo "BRIDGE-PARITY self-test: the extractor did not read the reference fixture (got: '$_bp_got')"
    fail=1
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

# ── (6) HITL binding tuple parity (Sprint 83 Phase L1, review r2 P2-17) ────
# The HITL approval tuple is the PREIMAGE of a host signature, and the host chrome
# renders it field by field so the human approves exactly what will be signed
# (WYSIWYS). The TS mirror is therefore a security surface: if Rust gains a field
# and the mirror does not, the operator signs a field that was NEVER SHOWN — the
# precise negation of the invariant, and the front-side tests cannot catch it
# (`Record<keyof HitlBinding, …>` and the literal field list both compare the TS
# mirror to its own TS definition). Only a cross-language check does.
# Both sides are normalised to snake_case (camel -> snake), BusyBox-safe.
#
# SCOPE, stated so this leg is not cited for more than it checks (ex-review r3
# L1-R3-10): it compares NAME SETS. It is blind to (a) a TYPE drift — turn
# `seq: u64` into a `String` and the signed preimage changes while parity stays
# green (that one is caught by `binding_digest_matches_its_golden_vector`), (b)
# a `pub(crate)` field, which still enters the JCS preimage but does not match
# the `pub ` pattern, (c) optionality on the TS side (`field?:`), and (d) the
# SHAPE OF THE NAMES themselves — the same bound the BRIDGE leg states: names
# are matched as `[a-z0-9_]` on the Rust side and `[a-zA-Z][a-zA-Z0-9]*` on the
# TS side (no underscore). So a `pub r#type: String` enters the signed JCS
# preimage while being invisible to BOTH extractions, and an `extra_snake:
# string` renders to the operator without existing in the signed tuple: a REAL
# drift that stays GREEN. Measured on both regexes in review round 6; missing
# from this leg until round 7, where the BRIDGE twin already had it.
# The class it DOES close — a field of the signed tuple that the chrome never
# renders — is mutation-tested: a 16th Rust-only field turns this leg red, and
# so does destroying the `pub struct HitlBinding` anchor (named diagnostic).
HITL_RS="crates/sbfb-workflow-engine/src/hitl.rs"
HITL_TS="web/src/pages/approvalsQueue.ts"
if [ ! -f "$HITL_RS" ] || [ ! -f "$HITL_TS" ]; then
  echo "HITL-PARITY: missing $HITL_RS or $HITL_TS"
  fail=1
else
  # ONE definition, shared with the self-test below (see BRIDGE-PARITY).
  _hp_extract_rs() {
    awk 'index($0,"pub struct HitlBinding")>0 {f=1;next} f && index($0,"}")==1 {f=0} f' \
      | grep -oE '^[[:space:]]*pub [a-z0-9_]+:' | grep -oE ' [a-z0-9_]+:' | tr -d ' :' | sort -u
  }
  # Rust: `pub <name>:` inside `pub struct HitlBinding { ... }`.
  # `|| true`: see the note on the BRIDGE leg — without it the `-z` guard
  # below is unreachable under `set -euo pipefail`.
  hitl_rs_fields="$(_hp_extract_rs < "$HITL_RS" || true)"
  # TS: `name:` / `name?:` inside `export interface HitlBinding { ... }`, then
  # camelCase -> snake_case. JSDoc lines start with `*` and cannot match.
  hitl_ts_fields="$(awk 'index($0,"export interface HitlBinding")>0 {f=1;next} f && index($0,"}")==1 {f=0} f' "$HITL_TS" \
    | grep -oE '^[[:space:]]+[a-zA-Z][a-zA-Z0-9]*\??:' | grep -oE '[a-zA-Z][a-zA-Z0-9]*' \
    | sed -E 's/([A-Z])/_\1/g' | tr 'A-Z' 'a-z' | sort -u || true)"
  hitl_rs_count="$(printf '%s\n' "$hitl_rs_fields" | grep -c . || true)"
  hitl_ts_count="$(printf '%s\n' "$hitl_ts_fields" | grep -c . || true)"
  # Self-test of the EXTRACTOR against a reference fixture, brought up to the
  # same discipline as `_bp_fixture` (ex-review r6 P1-2: this twin had one trap
  # against five there — the "one twin only" failure this dossier keeps
  # reproducing).
  #
  # What the traps kill, MEASURED on this fixture (not inferred):
  #  - `pub avant:` placed BETWEEN the bare `HitlBinding` mention and the real
  #    `pub struct` opener -> kills a start anchor that stops requiring
  #    `pub struct` (it captures `avant`);
  #  - `pub apres:` AFTER the closing brace -> kills a dropped end anchor;
  #  - the doc line shaped like a field -> kills a dropped `^` anchor.
  #
  # What they do NOT kill, stated so nobody credits this block for it:
  #  - widening the FIRST regex past `[a-z0-9_]` is caught by the SECOND grep,
  #    which re-filters to lowercase — not a hole, just not this block's doing;
  #  - dropping `next` is inert here, because the `pub struct HitlBinding {`
  #    line itself does not match the field pattern.
  _hp_fixture='// HitlBinding named here with no `pub struct` opener.
    pub avant: u8,
pub struct HitlBinding {
    /// a doc line shaped like a field: pub decoy_field: u8,
    pub zz_last: String,
    pub aa_first: u64,
    pub aa_first: u64,
}
    pub apres: u8,'
  _hp_got="$(printf '%s\n' "$_hp_fixture" | _hp_extract_rs | tr '\n' ' ' || true)"
  if [ "$_hp_got" != "aa_first zz_last " ]; then
    echo "HITL-PARITY self-test: the extractor did not read the reference fixture (got: '$_hp_got')"
    fail=1
  fi
  if [ -z "$hitl_rs_fields" ] || [ -z "$hitl_ts_fields" ]; then
    echo "HITL-PARITY: could not extract fields (RS=$hitl_rs_count TS=$hitl_ts_count) — check the struct/interface markers"
    fail=1
  elif [ "$hitl_rs_fields" != "$hitl_ts_fields" ]; then
    echo "HITL-PARITY drift: Rust HitlBinding ($hitl_rs_count) != TS HitlBinding mirror ($hitl_ts_count)"
    echo "  Rust fields: $(printf '%s' "$hitl_rs_fields" | tr '\n' ' ')"
    echo "  TS fields:   $(printf '%s' "$hitl_ts_fields" | tr '\n' ' ')"
    echo "  -> a field in the signed tuple that the host chrome does not render is a field the human signs BLIND."
    echo "  -> update BOTH crates/sbfb-workflow-engine/src/hitl.rs and web/src/pages/approvalsQueue.ts in lockstep."
    fail=1
  fi
fi

# ---------------------------------------------------------------------------
# §19.4 BOUND ORDERING — the numbered bounds of the threat model's §19.4 form
# ONE CommonMark ordered list. Only the FIRST marker counts: a renderer
# renumbers the rest sequentially, so an out-of-order source (…24, 26, 25…)
# silently shifts every ordinal on the rendered page — and the bounds are cited
# BY ORDINAL from many live places across docs, crates and planning, including
# `DURESS.md`, the runbook read under coercion. (No count is written here on
# purpose: three independent recounts in review round 8 disagreed with each
# other, which is exactly the class of claim this repo keeps getting wrong.)
#
# Source order must therefore be CONTIGUOUS, not merely increasing: a gap
# (…24, 26…) shifts every following ordinal just as an inversion does.
#
# Added in S83 L1 round 7 after that inversion shipped; REWRITTEN in round 8,
# which measured FOUR fail-OPEN mutations against the first version (a
# `#### 19.4.x` sub-heading silently ended the scan, bullet markers made it
# scan nothing, a gap passed, and `prev` was reset at a section seam).
# Mutation-tested, with the exact scope of each label (round 10 measured that
# two of them were written wider than what the mutation actually was):
#   RED  — inversion of two blocks; MIDDLE-of-list deletion (a gap); renamed
#          heading; sub-heading PLUS inversion; BULLETISATION of every marker;
#          duplicated §19.4 heading; unterminated fence.
#   GREEN, correctly — a sound list; a sub-heading alone (not a defect: the
#          fail-open was that it MASKED one); a code fence holding a numbered
#          line; a TAIL deletion.
# Assumed bound, stated rather than implied: deleting the LAST bound leaves the
# remaining sequence contiguous, so this leg cannot see it. Nothing here counts
# the bounds — a count no gate recomputes is the class of claim this repo keeps
# getting wrong (P1-1 of round 9 was exactly that).
THREAT_MD="docs/security/THREAT_MODEL.md"
if [ ! -f "$THREAT_MD" ]; then
  echo "§19.4 BOUND ORDERING: $THREAT_MD is missing — this leg cannot check anything"
  fail=1
else
  bound_break="$(awk '
    # Enter on the §19.4 heading; remember ITS depth so only a heading at the
    # SAME depth or shallower can close the section (a `#### 19.4.2` must not).
    # Fence rules come FIRST: inside a fenced block, NOTHING else is markup —
    # a `### 19.4` line in a code sample must not enter the section, and a
    # heading-shaped line must not close it. Ordering these after the heading
    # rules made both of those possible (round 9).
    #
    # ONE rule for open and close, remembering the OPENING character: CommonMark
    # allows both backtick and tilde fences, and a fence closes only on its OWN
    # character. Matching the two alternatives without remembering which opened
    # would let a backtick line close a tilde block (round 10 closed the CLASS,
    # not just the backtick instance).
    /^[[:space:]]*(```|~~~)/ {
      match($0, /[`~]/)
      c = substr($0, RSTART, 1)
      if (fence) { if (c == fence_char) { fence = 0 } }
      else { fence = 1; fence_char = c }
      next
    }
    fence { next }
    # `[^0-9.]` and not `[^0-9]`: a `#### 19.4.2` sub-heading must NOT re-enter
    # here, or it would reset the scan state and mask an inversion right after
    # it (measured in round 8 — that reset was one of the four fail-opens).
    # `!seen` on top: the section is entered ONCE. A second `### 19.4` heading
    # (a copy-paste, a merge artefact) would otherwise silently restart the
    # scan and forgive everything before it.
    !seen && /^#+[[:space:]]+19\.4([^0-9.]|$)/ {
      match($0, /^#+/); depth = RLENGTH
      inside = 1; seen = 1; first = 1; next
    }
    # A SECOND §19.4 heading closes the section as a same-level heading and
    # silently truncates the scan — everything after it goes unexamined, which
    # is how a duplicated heading hid an inversion in the round 9 mutation set.
    # (No apostrophes in this awk program: it is single-quoted in bash, and one
    # would terminate the string. That is exactly what happened here first.)
    seen && /^#+[[:space:]]+19\.4([^0-9.]|$)/ {
      print "  a SECOND §19.4 heading at line " NR " — a same-level heading closes the section, so anything after it is outside the scan"
      next
    }
    seen && !inside { next }
    inside && /^#+[[:space:]]/ {
      match($0, /^#+/)
      if (RLENGTH <= depth) { inside = 0 }
      next
    }
    inside && /^[0-9]+\.[[:space:]]/ {
      n = $1 + 0
      scanned++
      # `prev` is seeded ONCE, from the first marker actually seen, so a list
      # that legitimately starts at 1 and one that starts elsewhere both work.
      if (first) { first = 0 }
      else if (n != prev + 1) {
        print "  bound " prev " is followed by bound " n " at line " NR " (expected " prev + 1 ")"
      }
      prev = n
    }
    END {
      if (!seen) {
        print "  the §19.4 heading was not found — this leg silently checked NOTHING"
      } else if (scanned < 2) {
        print "  only " scanned + 0 " numbered bound(s) were scanned in §19.4 — the list markers stopped matching, so this leg silently checked NOTHING"
      }
      # An unterminated fence swallows the rest of the file, so a scan that ends
      # inside one has skipped an unknown amount of the section.
      if (fence) {
        print "  the scan ended inside an unterminated code fence — every line after it was skipped as fenced content"
      }
    }
  ' "$THREAT_MD" || true)"
  if [ -n "$bound_break" ]; then
    echo "§19.4 BOUND ORDERING: the numbered bounds are not a contiguous increasing sequence"
    printf '%s\n' "$bound_break"
    echo "  -> CommonMark renumbers them sequentially, so every 'bound N' citation in the repo shifts."
    echo "  -> move the block, do not renumber: the ordinals are load-bearing references."
    fail=1
  fi
fi

if [ "$fail" -ne 0 ]; then
  echo "check-frontier-contracts: FAILED"
  exit 1
fi

echo "check-frontier-contracts: clean (anti-promise + frontier-tag coverage [$frontier_count tagged] + DOMAIN census [$domain_census frozen] + BLOB_SERVE_CSP non-regression + prompt-kind provenance + bridge-parity [${ts_count:-?} methods] + pack-parity mirror + hitl-parity [${hitl_rs_count:-?} fields])"
exit 0
