# Sprint 72 Phase C — Review (align ollama-rs 0.3.4 + ExecutionTarget dispatch + Ollama)

Date: 2026-06-03
HEAD (pre-commit): `08b6cb2`
Reviewer: main thread (skill `nexus-phase-review` fallback — `nexus-phase-review-deep` agent not registered this session).
Preflight: `sprint72_phase_c_preflight.md` (PLAN-ADAPT → reclassed DESIGN-CONFLICT) +
`sprint72_phase_c_pivot_proposal.md` (PO Option A).

## Verdict: PASS

Review Claude OK ; Codex GPT-5.5 cross-check reconciled (0 GAP). See ## Codex reconciliation.

---

## 1. Scope delivered vs plan §6 (adapted by PO Option A)

| Plan/Decision item | Delivered | Evidence |
|---|---|---|
| Bloc 1: bump `ollama-rs` 0.2→0.3.4 (workspace pin + factory direct dep `stream`) | ✅ | `Cargo.toml:105`, `crates/sbfb-factory/Cargo.toml` (`{ workspace = true, features=["stream"] }`) |
| Bloc 1: rename `GenerationOptions`→`ModelOptions` + import | ✅ | `nexus-worker-core/src/llm/ollama.rs:29,~239` |
| Bloc 1 (PLAN-ADAPT S1b-1): `FormatType::StructuredJson(Box<JsonStructure>)` | ✅ | `ollama.rs:~163` boxes the payload |
| Bloc 1 (PO Option A): schemars 0.8→1.2 workspace bump | ✅ | `Cargo.toml:330` `schemars = "1.2"`; snapshot regenerated |
| Bloc 1 (ground-truth ripple): `nexus-executor` `GenerationOptions`→`ModelOptions` | ✅ | `nexus-executor/src/task_runner.rs:5,~17` (2nd consumer, missed by plan/preflight, caught by clippy) |
| Bloc 1 (ground-truth ripple): token counters `Option<u32>`→`Option<u64>` (`.map(u64::from)` removed) | ✅ | `ollama.rs:~230-231` (clippy `useless_conversion`) |
| Bloc 2: NEW `provider_router.rs` `ExecutionTarget` + `ProviderStream` + `from_provider` + `run` | ✅ | `crates/sbfb-factory/src/provider_router.rs` |
| Bloc 2: Claude arm = `spawn_claude_stream` verbatim (idle-timeout D6, gate unchanged) | ✅ | arm delegates; `claude_target_is_behaviorally_unchanged` test |
| Bloc 2: Ollama arm = `generate_stream` → Delta/Done + idle-timeout + diagnostic | ✅ | `ollama_stream`; maps `Vec<GenerationResponse>` per tick |
| Bloc 2: Network arm = todo (Phase D) | ✅ | `network_not_implemented` yields a clear "Phase D" Error |
| Bloc 2 (PLAN-ADAPT S1b-3): module host `main.rs` (no `lib.rs`) | ✅ | `main.rs:~19` `#[allow(dead_code)] mod provider_router;` |
| Scope boundary: operator_server NOT wired (Phase D) | ✅ | `operator_server.rs:898` still `spawn_claude_stream` direct — untouched |

Scope cuts honored (kickoff §7): network wiring + provider cabling = Phase D; front UX = Phase E;
no `*_VERSION` bump (pre-launch); no `ollama-client` extraction (caduc).

## 2. Day-0 + DESIGN-CONFLICT resolution

- D1 (enum-dispatch `Pin<Box<dyn Stream<StreamChunk>>>`): honored — `ExecutionTarget` + `ProviderStream`.
- D2 (ollama-rs 0.3.4 everywhere + bump worker): honored — **plus** the PO-arbitrated schemars 0.8→1.2
  bump (Option A), which the kickoff/plan/preflight had not foreseen. The DESIGN-CONFLICT (ollama-rs 0.3.4
  ⇒ schemars 1.2 ⇒ `TaskResponse: schemars_1::JsonSchema` bound) was surfaced at ground-truth, escalated,
  and resolved by PO Option A. Documented in the pivot proposal + preflight addendum.
- D5 (3 orthogonal axes, name `ExecutionTarget`): honored; no change to `process.rs` `providers_list`.

## 3. R7 — determinism non-regression (binary criterion)

The 4 quorum tests + `deterministic_options_wire_temperature_and_seed` stay GREEN post-bump:
`verifiable_task_uses_greedy_seed` (worker), `two_honest_workers_same_hash`,
`quorum_accepts_deterministic_redundancy`, `quorum_rejects_nondeterministic_divergence` (coordinator).
Observed in the targeted `nextest -p sbfb-factory -p nexus-worker-core -p nexus-coordinator-rs` run:
**578 passed, 0 skipped**. The seed/options API survived the bump (mechanical rename), so the greedy-seed
hash-exact quorum path is preserved.

## 4. Branch coverage (semantic) — new code

`provider_router.rs` tests (6): `execution_target_from_provider_parses_closed_set` (claude/ollama/local/
network/unknown/empty), `claude_target_is_behaviorally_unchanged` (delegation, no real spawn),
`ollama_diagnostic_flags_connection_refused` (pure mapping), `ollama_unreachable_yields_diagnostic`
(real dead-port connect → one Error), `ollama_stream_maps_to_chunks` (availability-gated Delta…Done),
`network_target_reports_not_implemented`. Every arm of `from_provider` and `run` is exercised; the Ollama
arm's error path (outer `Result`) and unreachable path are both covered.

## 5. Security / threat model

- SSE Operator surface unchanged this phase (gate `SENSITIVE_ACTIONS` stays in `handle_chat_stream`
  BEFORE dispatch — operator_server untouched; cabling is Phase D). No new inbound surface.
- Ollama arm targets loopback `127.0.0.1:11434` (`Ollama::default()`); `SBFB_OLLAMA_ENDPOINT` is a NEW
  override defaulting to loopback. Within the hardened loopback boundary.
- Unreachable Ollama yields an actionable diagnostic, never a silent empty stream.

## 6. Research grounding

ollama-rs 0.3.4 API verified against the **vendored crate source** (ground truth), not just docs:
`generate_stream -> Result<GenerationResponseStream>` with `GenerationResponseStreamChunk = Vec<GenerationResponse>`
(`completion/mod.rs:12-17`); `GenerationResponse{ response, done, total_duration: Option<u64> }`;
`ModelOptions` at `ollama_rs::models`; `FormatType::StructuredJson(Box<JsonStructure>)`; `JsonStructure::new<T: JsonSchema>`
where `JsonSchema` = schemars 1.2 (`parameters/mod.rs`); `Ollama::try_new(IntoUrl)` for the endpoint override.

## 7. Patterns / tech debt

- `#[allow(dead_code)]` on `mod provider_router;` is a deliberate phase-sequencing artifact (Phase D wires
  `operator_server`); commented inline. To be removed in Phase D.
- llama.cpp `llguidance::from_json_schema(Value)` now receives a draft 2020-12 schema (schemars 1.2). The
  feature `llm_llama_cpp` is off in default/CI builds (cmake-free); the draft-change correctness for GPU
  workers is a watch-item (PO accepted under Option A). To verify when a llama.cpp build is exercised.
- `task_response.schema.json` snapshot regenerated (draft-07→2020-12, `definitions`→`$defs`, `uint8`
  gains `maximum:255`, `title` added). Semantically equivalent, structurally faithful.

## 8. Fail-fast results (Windows, rust 1.95.0 — matches CI rustc)

| Check | Result |
|---|---|
| `cargo fmt --all --check` | PASS (clean) |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS (0 warnings) |
| `cargo test --workspace --locked --doc` | PASS |
| `cargo build -p nexus-shell-daemon --release` | PASS (5m26s) |
| Touched crates — `cargo nextest run -p sbfb-factory -p nexus-worker-core -p nexus-coordinator-rs` | PASS — 578/578 (incl. R7 + 6 provider) |
| Touched crates — `nexus-core-rs schemas::*` + `nexus-executor` (filtered) | PASS — 20/20 (incl. `schema_snapshot_matches_struct` w/ schemars 1.2; `task_runner` ModelOptions) |
| `cargo nextest run --workspace --locked` (Windows native) | 714 passed, **1 FAIL + 15 TIMEOUT — all in the iroh P2P / daemon-singleton E2E layer** (`nexus-core-rs` blobs/docs/gossip/discovery/node @90s + `nexus-shell-daemon` http/dispatch/`second_start_refuses…`), the documented Windows-native iroh-docs hang (P2-A-1 S71, `feedback_wsl_before_push`). Orthogonal to Phase C (schema/routing touch nothing in the iroh path). |

**Verification stance**: every crate Phase C touches passes on Windows (598 tests across the
targeted runs). The full-workspace failures are exclusively the iroh networking + daemon-singleton
E2E tests that hang on Windows native by documented limitation — the canonical full count is CI
Linux (kickoff §1.3). A transient `LNK1201`/`LNK1285` (corrupted `pow_wire.pdb` from two concurrent
cargo builds) was resolved by purging the stale `pow_wire` artifacts.

Front (web/ Vitest, factory-operator tsc/eslint) = N/A this phase (web untouched; Operator UX is Phase E).
Python = N/A (Rust+Frontend project). Docker-Linux full-suite repro = pre-push activity (sprint pushes
nothing; local rustc 1.95 already matches CI rustc); it is the canonical home of the iroh E2E tests.

## Codex reconciliation

Codex GPT-5.5 raw report : `.planning/active/sprint72_phase_c_codex_review.md` (output brut,
non reecrit). Codex executed the suites itself (R7 quorum tests, schema, executor, provider_router
— all green).

- **8 livrables audites : 7 CONFIRMES, 0 GAP, 1 PARTIEL.** No P0/P1.
- **PARTIEL (Livrable 8)** : `ollama_stream_maps_to_chunks` skips silently without a live Ollama, so
  the Delta/Done mapping was not proven deterministically. **Resolved by addition** : new test
  `ollama_stream_maps_to_chunks_via_mock` (`provider_router.rs`) stands up a mock HTTP server that
  streams an NDJSON body and asserts `Delta("Hello") Delta(" world") Done{result:"Hello world",
  duration_ms:2}` — deterministic, no live Ollama (mirrors the worker's `execute_task_ollama_mock_*`).
  The availability-gated test stays as an opportunistic real-Ollama integration check. Suite re-run :
  `nextest -p sbfb-factory` 137/137 green ; fmt + clippy green.
- **P3 note (Livrable 1)** : stale comment in `crates/nexus-core-rs/Cargo.toml` still cited
  `ollama-rs 0.2.6` / schemars `0.8.21` — corrected to reflect the 0.3.4 / schemars 1.2 bump.
- Codex confirmed the control checks: operator_server NOT wired to `ExecutionTarget` (Phase D),
  `SENSITIVE_ACTIONS` gate intact, no `*_VERSION` changed.

Net : 0 GAP, PARTIEL closed by a deterministic test, P3 doc fixed. Verdict PASS.
