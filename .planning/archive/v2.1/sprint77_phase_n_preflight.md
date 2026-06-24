# Sprint 77 — Phase N preflight (G8)

> Produit par un Workflow ultracode (5 scans factuels + 2 vérificateurs
> adversariaux + synthèse, 8 agents Opus 4.8 1M, 578k tokens). Run
> `wf_0deed109-aef`.

## Verdict: EXECUTE

All five scans plus both adversarial verifiers resolve every load-bearing
anchor in live code, no frozen Day-0 decision is contradicted, and the phase is
docs + 1 lifted example + shell-script extension (0 functional Rust, 0 wire
bump, 0 dep). The lone reported conflict (`is_pipeline_contiguous couvre
[0..L)`) is an imprecise plan-prose shorthand the source-ref-check corrects by
construction — a wording-fidelity fix, not a deviation.

## S1 SOTA / OSS prior-art delta (llms.txt convention, agent-doc patterns)
- Canonical llms.txt (llmstxt.org / Jeremy Howard, FastHTML ref): H1 project
  name (only required), blockquote summary, free markdown blocks, zero-or-more
  H2 file-list sections of `[name](url): note`, special `Optional` H2 for
  skippable URLs, served at root `/llms.txt`. Phase N's planned structure
  matches exactly.
- Scoping an llms.txt to one product-area is an established convention (Supabase
  per-framework, Vercel entry-points-only, Anthropic slim index + `llms-full.txt`).
  PO #3 (root indexes sharding only) is convention-compatible, not a deviation.
- `cargo` example-as-test (`[[example]] test = true`, `cargo test --examples`)
  is the idiomatic anti-bit-rot mechanism. Confirmed by cargo reference.
- Two non-blocking SOTA notes: (a) a root-level `/llms.txt` is conventionally
  the whole-site map, so the sharding-only root file SHOULD carry an explicit
  one-line banner stating it currently indexes the sharding subsystem and a
  whole-repo index is deferred; (b) consider an `Optional` H2 for THREAT_MODEL
  §16 / PATTERNS deep anchors so token-constrained agents ingest wire spec +
  examples first.

## S1b Deps / dependency surface (must be 0 new dep)
- CONFIRMED 0 new dep. The two lifted test fns import only `use super::*;`;
  transitive deps are `KeyPair` (ed25519-dalek + rand) and in-crate shard_plan
  types — none touch serde_json (only the un-lifted sibling does).
- All needed crates are pre-existing workspace + direct deps of nexus-core-rs:
  `serde_json=1.0`, `ed25519-dalek=2.1`, `rand=0.8`. Crypto surface backed
  solely by ed25519-dalek + rand + blake3.
- `check-sharding-docs.sh` uses only BusyBox-safe tooling (`grep -oE/-qF/-nE/-qE`,
  `dirname`, `cd`, `[ -e ]`; no `-P`, no `\b`, no `jq`). Runs on the `bash:5`
  image across all 3 CI gates — the source-ref-check extension must stay within
  this toolset.
- WIRE-NOTE (not a dep): plan puts `sign_verify.rs` at `docs/sharding/examples/`
  which is NOT a cargo-autodiscovered dir; wiring its compilation requires
  either a Cargo.toml `[[example]]` path entry (metadata, not a dep) or a
  `tests/` re-import. The deps-claim must not pretend this Cargo.toml/tests edit
  is absent.

## S2 Historical decisions traversed
- **PO #3 (root llms.txt sharding-only)** — plan §20 default #3 + §20.3 livrable;
  rationale "repo-wide llms.txt = livrable transverse hors scope, sa propre
  phase/sprint". Frozen, documented.
- **PO #4 (bridge exposes NO shard method, future signature PROPOSED/GAP-not-shipped)**
  — plan §20 #4 + §20.3. VERIFIED LIVE: `sbfb-bridge.js` exposes exactly
  `task_submit`/`storage_get`/`storage_set`, zero shard; host
  `BridgeMethodSchema` is a closed 3-enum (protocol.ts:20-23). `bridge_gap.md`
  must mark any shard method PROPOSED/GAP-not-shipped.
- **source-ref-check** — plan §20.3 acceptance (2): every cited
  `file:symbol`/`file:line` must grep-resolve (fail if anchoring into the void).
  NET-NEW gate added to `check-sharding-docs.sh`.
- **consommée-jamais-autoritaire** — WIRING_SPEC item (1) = Truth-Stack authority
  header (`repo files > .planning/active/ > commits > prompts > chat`) with rule
  "fait absent du rang-1 = Not evidenced". Docs emit pointers + abstention, never
  a PASS/GREEN verdict. Phase N must introduce no docs-sourced verdict language.
- **additive-only / 0-bump / 0-dep** — frozen (CLAUDE.md pre-launch protocol).
  Phase L commit `744f84a` confirms FORMAT_VERSION=1. Structurally guaranteed for
  a docs phase.
- **honesty PROVISIONAL/S78 grep-enforced** — plan §20.4 "grep-enforced dans M
  ET N". Existing honesty-gate covers M docs only; Phase N must extend it to
  WIRING_SPEC/llms.txt.
- Truth-Stack hygiene watch-item: `.planning/active/` is a moving pointer
  archived at sprint close — once S77 archives, rank-2 anchors dangle and the
  source-ref-check (correctly) would break CI unless it validates ONLY rank-1
  (repo files: `crates/`, `docs/`, `web/`, `scripts/`) anchors, and the header
  flags the active/ ref as in-flight-only.

## S3 Threat model coverage (no worker_pubkey/initiator leak, invariants accurate)
- **Privacy whitelist is the type shape, test-pinned**: `project_shard_session`
  (http.rs:2096-2105) projects `ShardSessionView{session_id, member_count}` only;
  `shard_session_projection_hides_member_identities` (http.rs:5232) confirms the
  literal strings `worker_pubkey`/`initiator` never serialize. Matches
  THREAT_MODEL §16 SI-3/SI-4. WIRING_SPEC invariant is backed by code.
- **Empty envelope deterministic & pinned**: `{found:false, session:null}` 200 OK
  via `live_shard_session`→None (:2115), pinned by
  `shard_session_response_pins_empty_envelope` (:5208). `observe.curl.md` accurate.
- **Loopback tier real**: route inside `authed_routes` (:309), bearer + loopback
  Host + Origin.
- **Two MUST-carry threat caveats**: (1) WIRING_SPEC must NOT claim the loopback
  route / private group make the computation confidential — activations circulate
  in clear on `sbfb/shard/1` (SI-1 reconstruction High + SI-4 collusion High =
  residual ASSUME, no consumer GPU TEE 2026); the cardinal "admission ≠
  confidentialité" caveat (THREAT_MODEL §16) must hold in WIRING_SPEC. (2) The
  signed RunProof on-wire emission + the orchestrator that pilots generation are
  a carry S78 — the run-proof step must be marked PROVISIONAL/S78; the lifted
  example proves the PRIMITIVE roundtrips, not in-vivo signed-proof emission.

## S4 Wire format / anchor resolution
0-BUMP confirmed: `SHARD_PLAN_FORMAT_VERSION=1` (shard_plan.rs:77),
`RUN_PROOF_FORMAT_VERSION=1` (:81) unchanged; all `DOMAIN_*` additive.

Three anchor traps the source-ref-check / implementer MUST handle: (a) `accept_bi`
is an iroh `Connection` call, not an SBFB `fn` — cite the call site shard.rs:314
+ preceding `is_member` gate :304, never `fn accept_bi`; (b) generated schemas
live at `crates/nexus-core-rs/src/schemas/`, so the llms.txt "schemas/" link must
point at the crate path, not `docs/protocol/`; (c) the lifted example's private
`sample_*` helpers must be inlined (external file cannot see `#[cfg(test)]` items).

## Adversarial verification result
- **0 anchors refuted.** Both verifiers independently grepped every symbol/path;
  all load-bearing primitives resolve.
- **0 frozen decisions refuted.** Pre-launch 0-bump, additive raw-op,
  consommée-jamais-autoritaire, bridge 3-method whitelist all consistent with
  Phase N livrables.
- The `is_pipeline_contiguous couvre [0..L)` item is explicitly REFUTED as a
  DESIGN-CONFLICT: live code already separates contiguity (shard_plan.rs:206-226,
  doc states it does NOT require first==0) from coverage (`covers_full_model`
  placement.rs:295-305 = contiguity + first.layer_start==0 + last.layer_end==total).
  The source-ref-check forces WIRING_SPEC to anchor to the real symbols,
  correcting the prose by construction.
- **Lifted-example feasibility CONFIRMED**: verbatim-by-content, NOT
  verbatim-by-copy — `sample_*` are private `#[cfg(test)]`; all underlying called
  primitives (`KeyPair::generate`, `public_bytes`, `…::sign`, `verify_signature`,
  `RunProof::new`, `RunProofEntry::sign`) are pub. Helpers must be inlined.

## Anchors Phase N MUST cite (authoritative list — this is the contract)
WIRING_SPEC + llms.txt MUST cite (and source-ref-check MUST resolve) exactly:
- Signing primitives: `shard_plan.rs:235` (ShardedSessionManifest), `:331` (sign),
  `:356` (verify_signature), `:462` (RunProof::new), `:505` (RunProofEntry::sign);
  `crypto.rs` (KeyPair::generate, public_bytes).
- Domain tags: `canonical.rs:258/276/290/310/332`
  (COMPUTE_GROUP / SHARD_PLAN / RUN_PROOF / VRF_DRAW / ACTIVATION_COMMIT V1).
- Preconditions (cite BOTH, state contiguity ≠ coverage): `shard_plan.rs:212`
  (is_pipeline_contiguous) AND `placement.rs:299` (covers_full_model).
- Admission gate / crypto-before-IO: `shard_claim.rs:271` (authorize_claim,
  verify FIRST); `compute_group.rs:150/239` (is_member); data-plane gate
  `shard.rs:304` → call site `shard.rs:314` (iroh accept_bi).
- Control plane: `http.rs:309` (route), `:2147` (handler), `:2124`
  (shard_session_response), `:2115` (live_shard_session→None), `:2096-2105`
  (ShardSessionView projection); `daemon.ts` (getShardSession).
- Privacy invariant: `THREAT_MODEL.md` §16 (member_count aggregate / never
  worker_pubkey/initiator; admission ≠ confidentialité).
- Format-version 0-bump proof: `shard_plan.rs:77/81`.
- Lifted example: `shard_plan.rs:651` + `:686`, with inlined helpers from
  `:602/:615/:633`.
- Schemas (crate path, NOT docs/): `crates/nexus-core-rs/src/schemas/*.schema.json`.
- Spec + human docs: `docs/protocol/SHARD_PROTOCOL_SPEC.md`;
  `docs/sharding/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE}.md`.

## Risks / watch-items for implementation (8)
1. **Example execution vs nextest (primary wiring risk).** nextest does NOT run
   example targets even with `[[example]] test=true`. Resolve the plan's OR-choice
   to ONE concrete CI-invoked mechanism: a `tests/` integration test that
   `include!`s `docs/sharding/examples/sign_verify.rs` (nextest WILL run it);
   drift = compile/test fail under the primary runner.
2. **Lifted helpers must be inlined**, not imported — `sample_*` are private
   `#[cfg(test)]`. "Lifté VERBATIM" = signing/verifying body verbatim + fixtures
   inlined.
3. **WIRING_SPEC precondition wording**: `is_pipeline_contiguous` = structural
   contiguity; full-model coverage is a SEPARATE `covers_full_model` (scheduler)
   check. Do not conflate (re-introduces the Phase M defect).
4. **Confidentiality caveat in WIRING_SPEC**: carry "admission ≠ confidentialité"
   + SI-1/SI-4 High-residual ASSUME; mark run-proof step PROVISIONAL/S78. No
   docs-sourced PASS/verdict language.
5. **Root llms.txt banner**: explicit one-line "indexes the sharding subsystem
   only; whole-repo index deferred".
6. **Truth-Stack `.planning/active/` is a moving pointer** archived at S77 close —
   source-ref-check validates ONLY rank-1 (repo-file) anchors; header flags
   active/ as in-flight-only.
7. **source-ref-check must stay BusyBox-safe** (no `grep -P`, no `\b`); catch the
   `accept_bi`/`fn`-vs-call-site and crate-vs-docs schema-path traps.
8. **Extend honesty-gate to WIRING_SPEC + llms.txt** (PROVISIONAL + S78 markers),
   not just the M docs.

## Decision
**EXECUTE.** Phase N is doc + 1 lifted example + doc-lint extension: 0 functional
Rust, 0 wire bump, 0 dep, all invariants and frozen PO/Day-0 decisions preserved.
Proceed, honoring the 8 watch-items — chiefly wiring the example to a runner the
CI actually invokes (1), inlining the private fixtures (2), and keeping
contiguity ≠ coverage in the WIRING_SPEC (3).
