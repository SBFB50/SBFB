# Sprint 72 Phase C Preflight

Date: 2026-06-03
HEAD: `08b6cb2`
Verdict: **PLAN-ADAPT** — RECLASSE **DESIGN-CONFLICT** par ground-truth
implementer (cf. addendum §"Ground-truth correction" en bas + pivot proposal
`sprint72_phase_c_pivot_proposal.md`). Le scan S1b a clear schemars sur le
**changelog 0.3.0** (0.8.21→0.8.22) ; le patch **0.3.4** reellement resolu
depend de **schemars 1.2**, ce qui casse le bound `TaskResponse:
schemars::JsonSchema` du worker. Arbitrage PO requis (options A/B/C).

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure source-of-truth)
  - `.planning/active/sprint72_plan.md` (Phase C = §6)
  - `.planning/active/sprint72_kickoff.md` (D1/D2/D5 + R7 §9)
  - `crates/nexus-worker-core/src/llm/ollama.rs` (deterministic_options + req_build + FormatType site)
  - `crates/nexus-worker-core/src/llm/schema_bridge.rs` (JsonStructure::new)
  - `crates/nexus-core-rs/src/schemas/task_response.rs` (TaskResponse derives schemars JsonSchema)
  - `crates/sbfb-factory/src/llm_bridge.rs` (StreamChunk + spawn_claude_stream + assemble_prompt)
  - `crates/sbfb-factory/src/main.rs` (crate is a BINARY — no lib.rs; mod list)
  - `crates/sbfb-factory/src/operator_server.rs` (ChatSession, ChatSendRequest, handle_chat_stream :822-898)
  - `crates/nexus-worker-core/src/engine/runtime.rs` (build_generate_params + verifiable_task_uses_greedy_seed)
  - `Cargo.toml` (workspace pin `ollama-rs = "0.2"`, `schemars = "0.8.21"`)
  - `crates/sbfb-factory/Cargo.toml` + `crates/nexus-worker-core/Cargo.toml`
  - `Cargo.lock` (ollama-rs 0.2.6 -> schemars 0.8.22 transitive; schemars 0.9.0 + 1.2.1 also present)
- Commands run:
  - `git rev-parse --short HEAD` -> `08b6cb2`
  - `git log --oneline -8 -- crates/nexus-worker-core/src/llm/ollama.rs` -> last change `0daff81` (S71 Phase B greedy seed)
  - `Grep "name = \"ollama-rs\""` Cargo.lock -> `version = "0.2.6"`, deps include `schemars 0.8.22`
  - `Grep "name = \"schemars\""` Cargo.lock -> 0.8.22, 0.9.0, 1.2.1 all resolved (different consumers)
  - `Grep` _VERSION across crates -> all `*_VERSION = 1` (TASK_FORMAT_VERSION, TASK_RESPONSE_VERSION, etc.)
  - context7 `/pepperoni21/ollama-rs` (queried 2026-06-03)
  - docs.rs ollama-rs 0.3.4 (GenerationRequest, FormatType, Ollama::generate_stream, GenerationResponseStream, parameters module index)
  - WebSearch ollama-rs 0.3.0 changelog (2026-06-03)

## Scope
- Plan source: `.planning/active/sprint72_plan.md` §6 (Phase C), two ordered blocs.
- Target files:
  - Bloc 1 (migration): `Cargo.toml` (workspace pin `ollama-rs`), `crates/nexus-worker-core/Cargo.toml`,
    `crates/nexus-worker-core/src/llm/ollama.rs`, `crates/sbfb-factory/Cargo.toml`.
  - Bloc 2 (dispatch): NEW `crates/sbfb-factory/src/provider_router.rs`, `crates/sbfb-factory/src/main.rs`
    (NOT `lib.rs` — see Finding S1b-3), `crates/sbfb-factory/src/llm_bridge.rs` (Claude arm reuses
    `spawn_claude_stream`, unchanged).
- Deps/APIs/specs: `ollama-rs` 0.2.6 -> 0.3.4 (workspace pin + new direct dep on sbfb-factory with
  feature `stream`); transitive `schemars` (stays 0.8.x); `futures::Stream`, `async-stream`, `tokio`.
- Security/protocol surfaces: bras Ollama hits Ollama loopback (`127.0.0.1:11434` via `Ollama::default()`);
  SSE `StreamChunk` contract (local Operator<->front, NOT a P2P wire). No new inbound surface in Phase C
  (operator_server wiring is Phase D).
- Tests expected:
  - R7 binary criterion (non-regression): `verifiable_task_uses_greedy_seed`,
    `two_honest_workers_same_hash`, `quorum_accepts_deterministic_redundancy`,
    `quorum_rejects_nondeterministic_divergence`, `deterministic_options_wire_temperature_and_seed`
    stay GREEN post-bump.
  - NEW: `execution_target_from_provider_parses_closed_set`, `claude_target_is_behaviorally_unchanged`,
    `ollama_stream_maps_to_chunks` (stub/skip), `ollama_unreachable_yields_diagnostic`.

## S1a OSS Prior Art
- Domain: closed-set provider dispatch returning a unified `Pin<Box<dyn Stream<StreamChunk>>>`;
  Ollama Rust streaming via `ollama-rs`.
- Sources (2026-06-03):
  - context7 `/0xplaygrounds/rig` (rig-core 0.35.0) — `CompletionModel` provider-agnostic + streaming
    enum; pattern "stream of unified enum chunks cross-provider" == SBFB `StreamChunk` (`llm_bridge.rs:44`).
  - crates.io `enum_dispatch` + somethingsblog "When Enums Beat dyn Trait" (2025-04-20) — closed set ->
    enum+match static dispatch, no vtable, inlinable. Confirms D1 (enum over `async-trait` double-box).
  - context7 `/pepperoni21/ollama-rs` — `ModelOptions::default().temperature(f32).seed(i32).num_predict(i32)`,
    `GenerationRequest::new(model,prompt).options(opts).system(s)`, `generate_stream(req)` under feature `stream`.
- Finding: **APPROACH-ALIGNED**. The enum-dispatch + boxed-stream design matches mature OSS practice
  (rig/enum_dispatch). The Claude arm wraps the unchanged `spawn_claude_stream` (already
  `impl Stream<Item=StreamChunk>`, `llm_bridge.rs:95-125`); boxing it into `ProviderStream` is trivial.
- Impact: none on the architecture. The arm *bodies* need the corrected ollama-rs 0.3.4 API surface
  precised by S1b below.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `ollama-rs` 0.2.6 -> 0.3.4; transitive `schemars`; `generate_stream` stream type.
- Commands/sources: docs.rs ollama-rs 0.3.4 pages (GenerationRequest, FormatType, Ollama, GenerationResponseStream,
  parameters index), context7, WebSearch changelog 0.3.0, Cargo.lock greps.

- **CVE/advisory**: clean. RustSec has no advisory on the `ollama-rs` crate (kickoff §, re-confirmed —
  the 2026 Ollama CVEs target the *server*, not the Rust crate). The bump is advisory-clean.

- **The seed/options API SURVIVES (R7 cleared at the API level)**: docs.rs 0.3.4 confirms
  `GenerationRequest::options(self, options: ModelOptions)`, `.system(impl Into<Cow>)`, `.new(model, prompt)`
  unchanged. `ModelOptions::default().temperature(f32).seed(i32).num_predict(i32)` confirmed by context7.
  The deterministic greedy-seed path (`ollama.rs:239-254`) migrates by a mechanical rename. **This is NOT a
  DESIGN-CONFLICT** — R7's worst case (seed API gone) did not materialize.

- **Finding S1b-1 (BLOCKING for the original plan text) — second breaking site omitted by the plan.**
  docs.rs 0.3.4: `FormatType::StructuredJson(Box<JsonStructure>)`. The worker at `ollama.rs:159` currently
  calls `FormatType::StructuredJson(ollama_json_structure())` where `ollama_json_structure()` returns a bare
  `JsonStructure` (`schema_bridge.rs:47-49`). Under 0.3.4 this no longer type-checks — it must become
  `FormatType::StructuredJson(Box::new(ollama_json_structure()))`. **Plan §6.2 lists ONLY the
  `GenerationOptions`->`ModelOptions` rename at `ollama.rs:239-254`**; it does not list `ollama.rs:159`.
  This is a real, compile-breaking migration site the original plan missed.

- **Finding S1b-2 (BLOCKING for the original plan text) — `generate_stream` return type mis-stated in
  BOTH kickoff and plan.** docs.rs 0.3.4: `pub async fn generate_stream(&self, request) -> Result<GenerationResponseStream>`
  where `GenerationResponseStream = Pin<Box<dyn Stream<Item = Result<GenerationResponseStreamChunk>> + Send>>`.
  - Kickoff §3 says "stream de `Vec<GenerationResponse>`" — WRONG.
  - Plan §6.1 says "chaque `GenerationResponse.response` -> `StreamChunk::Delta`" (implies `Stream<GenerationResponse>`) — WRONG.
  - Reality: the outer call returns `Result<...>` (handle the connect error -> `StreamChunk::Error` diagnostic),
    and each stream item is `Result<GenerationResponseStreamChunk>` (handle per-chunk `Err` -> `StreamChunk::Error`).
    The `.response` text field and `final_data`/`done` marker are read off `GenerationResponseStreamChunk`.
  The Ollama-arm mapping code must be written against this `Result<Chunk>` shape, not a bare `GenerationResponse`.

- **Finding S1b-3 (NON-BLOCKING, design correction) — `sbfb-factory` has no `lib.rs`.**
  `crates/sbfb-factory/src/` contains `main.rs` (binary) and modules declared there (`main.rs:5-21`).
  Plan §6.2, D1, kickoff §4 all say "`mod provider_router;` dans `lib.rs`". There is no `lib.rs`
  (`Glob crates/sbfb-factory/src/**/*.rs` returns no `lib.rs`). The module must be declared in `main.rs`
  alongside the existing `mod ...` list. Mechanical, but the plan's file target is wrong.

- **schemars version risk INVESTIGATED and CLEARED (non-finding).** The worker's `JsonStructure::new::<TaskResponse>()`
  (`schema_bridge.rs:48`) requires `TaskResponse: ollama_rs::generation::parameters::JsonSchema`, and
  `TaskResponse` derives `JsonSchema` via `schemars::{JsonSchema, schema_for}` (`task_response.rs:42,66`).
  docs.rs 0.3.4 `parameters` module RE-EXPORTS `JsonSchema` + `schema_for` (they are schemars items, not a
  custom trait). The 0.3.0 changelog (WebSearch 2026-06-03) bumped schemars 0.8.21 -> **0.8.22** (NOT to 1.x,
  NOT removed). Cargo.lock already resolves `schemars 0.8.22` for ollama-rs 0.2.6 and the workspace pin is
  `^0.8.21`. So `TaskResponse: schemars 0.8::JsonSchema` still satisfies the bound post-bump — no trait
  version mismatch, no `schema_for` semantic break. (The `schemars 0.9.0`/`1.2.1` rows in the lock belong to
  unrelated consumers and do not affect this path.)

- **Finding S1b-4 (NON-BLOCKING, watch) — `Box::new` ripples to `schema_snapshot_matches_struct`?** No.
  The schemars 0.8.21 -> 0.8.22 patch did not change `schema_for!` output shape between those two patch
  versions for this struct (both already co-resolved in the current lock). The snapshot test
  (`task_response.rs:272`) should stay green; if a patch-level `$schema` string drift appears, it is a
  one-line `UPDATE_SNAPSHOTS=1` refresh, documented and non-blocking. Flagged so the implementer re-runs
  `cargo nextest run -p nexus-core-rs --locked` and treats any snapshot drift as expected migration output,
  not a regression.

- Aggregate S1b: the API is migrable (no DESIGN-CONFLICT), but the plan's implementation text under-specified
  the migration: 2 breaking sites instead of 1, a mis-stated stream type, and a wrong module-host file. These
  are corrections to the *implementation approach*, evidence-backed by docs.rs — the canonical trigger for
  **PLAN-ADAPT** (proceed with the corrected approach, document the delta, plan file stays a snapshot).

## S2 Historical Decisions
- Commands: `git log --oneline -8 -- crates/nexus-worker-core/src/llm/ollama.rs`,
  `-- crates/sbfb-factory/src/llm_bridge.rs`, `-- crates/sbfb-factory/src/operator_server.rs`.
- Decisions crossed:
  - `0daff81` (S71 Phase B) — added `deterministic_options` greedy-seed for hash-exact quorum. **Most recent,
    NOT reverted.** Reverse-commit check: `git log 0daff81..HEAD -- ollama.rs` shows no later change to this
    file; the determinism contract is live. The bump MUST preserve it — this is exactly R7, and the API
    survives (S1b). Confirmed: documenting + re-verifying the 4 quorum tests is the correct guard. Non-blocking.
  - `c85397b` (S20 Phase D) — introduced the `ollama-rs 0.2` pin + `FormatType::StructuredJson(JsonStructure)`
    + `schemars 0.8.21` "to match ollama-rs 0.2.6 transitive constraint" (`Cargo.toml` comment). This pin is an
    S20 *implementation* pin, NOT a frozen Day 0 decision (the frozen decisions list in `nexus_grid_pivot.md` /
    CLAUDE.md does not pin ollama-rs version). Bumping it is in-bounds; the PO explicitly arbitrated D2
    (kickoff §4 D2, 2026-05-31). Non-blocking.
  - Frontier "Factory = crate externe hors daemon" (CLAUDE.md frozen decisions). The plan adds `ollama-rs`
    (a THIRD-PARTY crate) as a direct dep of `sbfb-factory`, NOT `nexus-worker-core`. The frontier forbids
    `sbfb-factory` pulling the worker core (iroh/GPU/engine); a third-party HTTP client is not a violation.
    Confirmed compliant. Non-blocking.
  - `a0337c6` (S71 Phase C) — gate SSE + idle-timeout `spawn_agent_stream` (D6). The Claude arm reuses
    `spawn_claude_stream` verbatim, preserving the gate (`operator_server.rs:865-879`, applied BEFORE dispatch)
    and the idle-timeout. Non-blocking.
- Finding: clean. No documented decision with a still-valid rationale is contradicted by Phase C. All crossed
  decisions are either preserved (greedy seed, gate, frontier) or were explicitly re-opened by PO arbitration
  (ollama-rs pin). No DESIGN-CONFLICT.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - Ollama-arm endpoint: the bras Ollama uses `Ollama::default()` -> `127.0.0.1:11434` loopback (matches the
    worker pattern `OllamaBackend::from_config` default port 11434, `ollama.rs:87`). `Grep SBFB_OLLAMA_ENDPOINT`
    returns NO existing hit — it is a NEW env override introduced by this phase; default stays loopback. Within
    the hardened loopback boundary; no new inbound surface. Non-regression.
  - Unreachable-Ollama diagnostic: the worker already classifies connection-refused (`ollama.rs:360-368`
    `looks_like_connection_refused`) and `spawn_agent_stream` already yields a clear not-found/timeout
    `StreamChunk::Error` (`llm_bridge.rs:163-213`). The Ollama arm must yield a comparable diagnostic
    (`StreamChunk::Error` with an install/`ollama serve` hint) on the `Result` error from `generate_stream`.
    Pattern exists; reuse it. Non-blocking.
  - SSE Operator surface (`:3001`): catalogued in Phase A (`105c054`, P2-H-1 closed). Phase C does NOT extend
    the inbound SSE surface (no `operator_server.rs` wiring this phase — that is Phase D). The gate
    SENSITIVE_ACTIONS (`operator_server.rs:865-879`) is unchanged and remains BEFORE any dispatch.
  - The token-daemon `X-SBFB-Token` auth (R3) concerns the Network arm (Phase D), not Phase C. Out of scope here.
- HARDENING_ROADMAP status: no S72 Phase C pre-requirement open. The only carry trigger
  ("before extending the Operator surface") was satisfied by Phase A.
- Finding: clean. No regression on a covered threat; the Ollama loopback arm reuses existing diagnostic and
  loopback patterns; the SSE gate is preserved and untouched in this phase.

## S4 Protocol And Wire Invariants
- Wire/security files checked: `Grep _VERSION` across `crates/` — `TASK_FORMAT_VERSION = 1` (`task.rs:61`),
  `TASK_RESPONSE_VERSION = 1` (`task_response.rs:48`), `CURATOR_LIST_FORMAT_VERSION`, `POW_FORMAT_VERSION`,
  `CANARY_VERSION`, `SCHEMA_VERSION` — all `= 1`.
- VERSION/domain/canonical status: NO `*_VERSION` is touched by Phase C. The bump of the `ollama-rs` pin in
  `Cargo.toml`/`Cargo.lock` is a dependency pin, not a wire format. `StreamChunk` (`llm_bridge.rs:44`) is the
  local SSE contract (Operator<->front, loopback), explicitly NOT a P2P wire (kickoff §1.4). The Network arm
  (Phase D) will CONSUME `TaskSubmission`/`TaskStatus` unchanged — but that is Phase D, not C.
- `serde(default)`: no new `serde(default)` introduced in Phase C. (`ChatSendRequest.provider`'s
  `#[serde(default)]` at `operator_server.rs:731` is pre-existing runtime tolerance, S71.)
- Day 0 status: **preserved**. D1 (enum-dispatch `Pin<Box<dyn Stream<StreamChunk>>>`), D2 (ollama-rs 0.3.4
  everywhere), D5 (3 orthogonal axes, `ExecutionTarget` name) all honored. The PLAN-ADAPT delta touches only
  *implementation detail* (Box wrap, stream Result shape, host file), not any Day 0 decision.
- Finding: clean. No pre-launch VERSION bump; canonical bytes and signing domains untouched; pre-launch
  protocol policy respected.

## Plan Adaptation
PLAN-ADAPT. The architecture is sound; the implementation text must be corrected on four ollama-rs 0.3.4 API
points the original plan under-specified. The plan file stays a snapshot; the Phase C code follows the
corrected approach below; the commit body must cite this file.

- **Original plan (§6.2)**: "Rename `GenerationOptions`->`ModelOptions` + import `ollama_rs::models::ModelOptions`.
  `deterministic_options` inchange fonctionnellement" — lists ONE migration site (`ollama.rs:239-254`) and the
  bras Ollama maps "`generate_stream(GenerationRequest)` -> chaque `GenerationResponse.response` -> `StreamChunk::Delta`".

- **Evidence requiring adaptation** (docs.rs ollama-rs 0.3.4, 2026-06-03):
  1. `FormatType::StructuredJson(Box<JsonStructure>)` — second breaking site at `ollama.rs:159`.
  2. `generate_stream -> Result<Pin<Box<dyn Stream<Item = Result<GenerationResponseStreamChunk>> + Send>>>`
     — outer Result + per-item Result; NOT `Stream<GenerationResponse>` nor `Stream<Vec<GenerationResponse>>`.
  3. `sbfb-factory` is a binary with `main.rs`, no `lib.rs` (`main.rs:5-21`).
  4. schemars stays 0.8.22 (0.3.0 changelog) — re-run `nexus-core-rs` tests; treat any `$schema` snapshot
     drift as an expected `UPDATE_SNAPSHOTS=1` refresh, not a regression.

- **Corrected approach (concrete)**:
  - Bloc 1 (worker migration), two edits in `ollama.rs` (plan listed one):
    - `:28` import: `use ollama_rs::generation::options::GenerationOptions;`
      -> `use ollama_rs::models::ModelOptions;` and rename the two `GenerationOptions` uses in
      `deterministic_options` (`:243`, return type `:239`) to `ModelOptions`. Builders `.temperature()`/`.seed(i32)`
      unchanged. The `deterministic_options_wire_temperature_and_seed` test (`:398`) asserts via the serialize
      impl — keep it; `ModelOptions` serializes the same `{temperature, seed}` JSON.
    - `:159`: `FormatType::StructuredJson(ollama_json_structure())`
      -> `FormatType::StructuredJson(Box::new(ollama_json_structure()))`. `ollama_json_structure()`
      (`schema_bridge.rs:47`) keeps returning a bare `JsonStructure`; only the call site boxes it.
    - `Cargo.toml` workspace pin `ollama-rs = "0.2"` -> `"0.3.4"` (line 105). Worker inherits via
      `{ workspace = true }` (`nexus-worker-core/Cargo.toml:83`). `crates/sbfb-factory/Cargo.toml` adds
      `ollama-rs = { version = "0.3.4", features = ["stream"] }` as a direct dep.
  - Bloc 2 (provider_router), corrected facts:
    - `mod provider_router;` goes in **`crates/sbfb-factory/src/main.rs`** (alongside the existing mod list),
      NOT `lib.rs`.
    - Ollama arm: `let stream = ollama.generate_stream(req).await;` returns `Result<...>` — on `Err`, yield one
      `StreamChunk::Error` diagnostic ("Ollama unreachable - run `ollama serve`") and return. On `Ok(stream)`,
      `while let Some(item) = stream.next().await`: `item` is `Result<GenerationResponseStreamChunk>`; on
      `Err` yield `StreamChunk::Error`; on `Ok(chunk)` emit `StreamChunk::Delta { text: chunk.response }`, and
      when `chunk.done`/`final_data` is set emit the terminal `StreamChunk::Done`. Bound the whole arm with the
      same idle-timeout pattern as `spawn_agent_stream` (`llm_bridge.rs:138`, D6).
    - Claude arm: boxes the unchanged `spawn_claude_stream(prompt, model, cwd)` into `ProviderStream`
      (`Box::pin(...)`). Behavior byte-identical to S71 (test `claude_target_is_behaviorally_unchanged`).
    - Network arm: `todo!`/empty in Phase C (Phase D).

- **File/test delta vs original plan**:
  - +1 migration site (`ollama.rs:159` Box wrap) the plan omitted.
  - module host `main.rs` not `lib.rs`.
  - Ollama-arm mapping written against `Result<GenerationResponseStreamChunk>`, not a bare `GenerationResponse`.
  - tests unchanged from plan §6.3 (4 quorum re-greens + 4 new). `ollama_unreachable_yields_diagnostic` now
    covers the outer-`Result` error path explicitly.

## Risks And Scope Cuts
- Blocking risks: none. No DESIGN-CONFLICT — every API change is migrable and no Day 0 is contradicted.
- Non-blocking risks (carry-over / watch):
  - R7 (quorum determinism): API survives; the 4 quorum tests + `deterministic_options_wire_temperature_and_seed`
    remain the binary criterion. Re-run `cargo nextest run -p nexus-worker-core --locked` after Bloc 1.
  - schemars 0.8.21->0.8.22 patch: re-run `nexus-core-rs` suite; any `$schema` snapshot drift is an expected
    `UPDATE_SNAPSHOTS=1` refresh (S1b-4), not a regression.
  - R2 (Ollama not testable without a live daemon): the `ollama_stream_maps_to_chunks` test must stub or
    feature-gate and skip cleanly when Ollama is absent (B-3 S71 availability-gate pattern).
  - R3 (token-daemon auth) belongs to the Network arm — Phase D, not C.
- Scope cuts still honored (kickoff §7): Network arm wiring + operator_server provider cabling = Phase D;
  front UX = Phase E; no `*_VERSION` bump (pre-launch, PO-14); no extraction of a shared `ollama-client`
  crate (scope cut #15, caduc — ollama-rs IS the shared lib).

## Action
- PLAN-ADAPT: proceed with the corrected approach above; the Phase C commit body must cite this file and
  document: "Plan listed one ollama-rs migration site + a `Stream<GenerationResponse>` mapping; preflight S1b
  identified two breaking sites (`ModelOptions` rename AND `FormatType::StructuredJson(Box<JsonStructure>)` at
  `ollama.rs:159`), a `generate_stream -> Result<Stream<Item=Result<GenerationResponseStreamChunk>>>` return
  shape, and `main.rs` (no `lib.rs`) as the module host (docs.rs ollama-rs 0.3.4, 2026-06-03); adapted
  accordingly. schemars stays 0.8.22, no trait mismatch." R7's 4 quorum tests remain the binary migration
  criterion.

## Ground-truth correction (implementer, post `cargo fetch` 2026-06-03) — INVALIDATES the schemars clearance

The S1b clearance "schemars stays 0.8.22" was wrong. It read the **0.3.0** changelog,
not the **0.3.4** resolved deps. Evidence:

- `~/.cargo/registry/src/.../ollama-rs-0.3.4/Cargo.toml:209` → `schemars = "1.2.0"`.
- `cargo update -p ollama-rs --precise 0.3.4` added transitive `schemars_derive v1.2.1`.
- `ollama-rs-0.3.4/src/generation/parameters/mod.rs:92` → `new<T: JsonSchema>()` where
  `JsonSchema` is schemars **1.2** (line 1-2 `use schemars::{... Schema}; pub use schemars::{schema_for, JsonSchema};`).

Therefore `nexus-worker-core` `JsonStructure::new::<TaskResponse>()` (`schema_bridge.rs:48`)
requires `TaskResponse: schemars_1.2::JsonSchema`, but `TaskResponse`
(`nexus-core-rs/.../task_response.rs:66`) derives schemars **0.8** → bound unsatisfied →
the worker no longer compiles. `JsonStructure::new_for_schema(Schema)` does NOT rescue
byte-clean because `new::<T>()` strips `$ref`s (`inline_subschemas`) and
`task_response_schema()` carries a `$ref` to the nested `ToolCall` — rebuilding from that
`serde_json::Value` would ship a `$ref` schema Ollama rejects.

This forces either a workspace schemars 0.8→1.2 bump on the core wire schema (against the
documented S20 "we avoid this churn" decision) or a contain-or-rescope choice. **Verdict
reclassified DESIGN-CONFLICT → PO arbitration (A/B/C) in `sprint72_phase_c_pivot_proposal.md`.**
The Bloc 1 *worker* migration is HELD pending that decision; the Bloc 2 *Factory* Ollama
target (free-text `generate_stream`, no schemars) is unaffected by the conflict.
