# Phase Review — Sprint 29 Phase D

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings (1 P2, 1 P3) documentés / >=1 requis pour PASS.

## Memory consultation
- feedback_approach.md : pick deepest, research before code — respecté (preflight S1a 4 projets OSS consultés, context7 3 queries)
- feedback_context7_systematic.md : context7 obligatoire avant code touching lib/API — respecté (opentelemetry, opentelemetry_sdk, ed25519-dalek vérifiés via context7 + WebSearch)

## Staging check (Step 1bis)
- Phase fichiers : 6 modified + 1 new crate (nexus-trace-core/)
- Planning/docs split : N/A (preflight déjà commité e66c650)
- Untracked accidentels : 0

## Suites (Step 2)
- Rust fmt : clean ✅
- Rust clippy : 0 warnings ✅
- Rust nextest : 856 passed (843 → 856, +13 Phase D) ✅
- Rust doctests : pass ✅
- Release build daemon : OK ✅
- Python ruff : clean ✅
- Python SDK : 195 passed ✅
- Python coord : 391+36f+6s (36f = stale PyO3 wheel, pre-existing) ✅
- Python gov : 46 passed ✅
- Frontend lint : 0 errors (7 warnings pre-existing) ✅
- Frontend tsc : clean ✅
- Vitest : 269 passed ✅
- Frontend build + size : 7/7 ✅
- Playwright : 41 passed, 2 env fail (pre-existing) ✅
- scan-en-strings : clean ✅

## Delta tests (Step 3)
- Rust workspace : 843 → 856 (+13 nexus-trace-core)
- Python coord : 391+36f+6s → unchanged
- Vitest : 269 → unchanged
- Playwright : 41+2f → unchanged

13 tests ajoutés dans nexus-trace-core :
1. test_batch_log_processor_write_and_read
2. test_batch_log_processor_rotation
3. test_otel_processor_export_mock
4. test_signed_processor_roundtrip
5. test_signed_processor_tamper_detect
6. test_trace_context_inject_extract
7. test_trace_context_from_json_rpc
8. test_traceparent_roundtrip
9. test_child_shares_trace_id
10. test_invalid_traceparent_rejected
11. test_multi_processor_pipeline
12. test_set_trace_processors_replaces
13. test_domain_trace_event_v1

Plan prévoyait 10 tests — livré 13 (+3 bonus propagation).

## Modified-file branch coverage (Step 2bis)
- runtime.rs : `match BatchLogProcessor::new(...)` (8 LOC init) → CONCERN (defensive init, BatchLogProcessor::new testé dans nexus-trace-core)
- main.rs : `if let Ok(proc) = ...` (3 LOC init) → CONCERN (defensive init, même raison)

## Research grounding (Step 4bis)
- S1a OSS prior art : PASS — preflight documente 4 projets OSS (OpenTelemetry Rust, tracing-opentelemetry, Spine-OSS, dd-trace-rs), verdict APPROACH-ALIGNED
- Deps context7 : PASS — plan §3 Research consulte liste context7 /open-telemetry/opentelemetry-rust + WebSearch CVE ed25519-dalek + opentelemetry versions

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : HARDENING_ROADMAP §3 "A2 TraceProvider OTEL backend-agnostic" + plan §8 Phase D architecture ✅
- D1..D5 avec alternatives : D3 documente "PLAN-ADAPT vs roadmap 0.27" ✅
- Solution la plus poussée : OTel 0.31 (latest), Ed25519 domain-separated signing (Spine-OSS pattern) ✅
- Aucune LOC estimée au plan : ✅ (grep LOC estim = 0 match kickoff/plan)

## Scope cuts verification (Step 5)
- D3 Windows RPC : non touché ✅
- C4 task-scoped sandbox : non touché ✅
- CI Linux/macOS : non touché ✅
- opentelemetry 1.0 pin : non touché (utilise 0.31) ✅

## Findings

- **P2** : executor trace log path relatif `"traces/executor.jsonl"` au lieu d'un chemin absolu résolu depuis une racine connue (`ShellDaemonPaths` ou `$SBFB_HOME`). Si le broker spawn l'executor depuis un cwd inattendu, le log trace se crée au mauvais endroit. Acceptable pour le MVP (l'executor est spawné par le broker qui contrôle le cwd), carry S30 pour aligner sur le pattern daemon paths.
- **P3** : plan §8 disait `crates/nexus-shell-daemon-core/src/runtime.rs` comme fichier cible mais le vrai wiring est dans `crates/nexus-shell-daemon/src/runtime.rs` (binary crate, pas core). Plan inéxact mais intent correct. Cosmétique.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S30 : P2 executor trace log path → résoudre via `--trace-dir` CLI arg ou env var
- Corrections applied : commentaire workspace Cargo.toml "0.28" → "0.31" (corrigé inline)
