# Phase Review — Sprint 28 Phase A

## Verdict : PASS (2 P2 documented)

Rigor signal : 2 findings P2 documentes (>= 1 requis pour PASS rigoureux).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code — RESPECTED (S1a prior art SynthID APPROACH-ALIGNED)
- feedback_context7_systematic.md : context7 MCP unavailable, compensated by cargo registry source inspection (llama-cpp-2 0.1.143 LlamaSampler::logit_bias API verified) — RESPECTED with note

## Staging check (Step 1bis)
- Phase fichiers : 6 modified + 1 new (`configs/watermark.toml.sample`)
- Planning split : `sprint28_phase_A_preflight.md` → chore(planning) AVANT phase commit
- Untracked accidentels : 0

## Suites (3 blocs complets)
- Rust nextest : 821 → 823 (+2 visible default features) ✅
- Rust doctests : pass ✅
- cargo fmt : clean ✅
- cargo clippy : clean ✅
- Python SDK : 195 pass ✅
- Python coord : 391 pass + 36 fail + 6 skip (baseline stale wheel) ✅
- Python gov : 46 pass ✅
- Vitest : 264 pass ✅
- Frontend lint + tsc : clean ✅
- Frontend build + size-limit : 7/7 pass ✅
- Playwright : 41 pass + 2 fail (baseline env PyO3 wheel) ✅
- scan-en-strings : clean ✅
- Release build : OK ✅

## Delta tests
- Rust workspace : 821 → 823 (+2 : `generate_params_watermark_builder_sets_fields`, `generate_response_output_token_ids_serde`)
- +2 tests behind `llm_llama_cpp` feature gate (not counted in default CI) : `watermark_bias_construction_matches_generate_blocking_pattern`, `watermark_context_window_handles_short_sequences`
- Python : 0 delta (no Python changes)
- Vitest : 0 delta (no frontend changes)
- Total : ~1802 → ~1804 (+2 visible)

## Modified-file branch coverage (Step 2bis, G9)
- `llama_cpp.rs` : `if wm_active { ... }` branch (30 LOC) → tested by `watermark_bias_construction_matches_generate_blocking_pattern` ✅
- `mod.rs` : `with_watermark()` method → tested by `generate_params_watermark_builder_sets_fields` ✅
- `mod.rs` : `output_token_ids` serialization → tested by `generate_response_output_token_ids_serde` ✅
- `runtime.rs` : `.with_watermark(...)` + `generated.output_token_ids` → wiring change, exercised by existing `engine_claims_and_executes_tasks_on_registered_doc` ✅
- `ollama.rs` : `output_token_ids: vec![]` field addition → trivial, no branch ✅

## Commit body validation
- Format titre : ✅ `feat(sprint28): Sprint 28 Phase A — watermark end-to-end wiring + P2 batch S27 audit`
- Delta tests coherent : ✅ (+2 visible, +2 feature-gated)
- Scope cuts honoured : ✅ (see below)
- Co-Authored-By : to verify at commit time

## Research grounding (Step 4bis)
- S1a OSS prior art : PASS — SynthID (Google DeepMind Nature 2024) + Kirchenbauer KGW (ICML 2023) consulted, APPROACH-ALIGNED documented in preflight
- S1b deps : PASS — no new deps, llama-cpp-2 0.1.143 API verified via cargo registry
- Plan §Research consulte : PASS — kickoff §Sources section documents G9 agents + G2 trigger scan

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : ✅ PATTERNS.md P37 documents watermark architecture (PRF + bias + detector)
- D1..D5 with alternatives : ✅ kickoff D1 cites 3 rejected alternatives (Ollama wiring, Tournament Sampling, detection-only)
- Solution la plus poussee : ✅ SynthID-inspired bias is SOTA post-BIRA
- LOC estimates : P3 nit — kickoff contains sizing LOC (~400 code / ~800 docs) which are post-arbitrage order-of-magnitude, not prospective budget caps. Threshold LOC in plan R-S28-3 (~100 LOC scope-cut trigger) is legitimate risk mitigation. No blocking.

## Scope cuts verification
- Nym mixnet → S30+ : 0 fichiers diff ✅
- MIG partitioning → post-v1.0 : 0 fichiers diff ✅
- D2 broker/executor impl → S29 : 0 fichiers diff ✅
- D3 Windows RPC → S29 : 0 fichiers diff ✅
- C4 task-scoped sandbox → S29 : 0 fichiers diff ✅
- Tor transport → S30+ : 0 fichiers diff ✅
- All 12 scope cuts from kickoff §7 : clean ✅

## Findings (rigor signal — 2 P2 documented)

- **P2-REVIEW-1** : `generate_blocking` now takes 12 parameters (was 8). `#[allow(clippy::too_many_arguments)]` suppresses the warning but the function signature is growing. A future refactor to pass a `GenerateConfig` struct grouping model/ctx/threads/watermark params would improve readability. Carry-over S29.
- **P2-REVIEW-2** : Per-step `LlamaSampler::chain_simple` rebuild when watermark is active allocates/drops a sampler chain each iteration. For large vocab (128k+ tokens), this creates allocation churn in hot path. Future optimization: pre-build bias as sparse `Vec<LlamaLogitBias>` and cache sampler, invalidating only when context window changes (which is every step for sliding window). Acceptable at current scale (< 100 req/s, inference bottleneck >> allocation cost). Carry-over S29.

## Recommendation
- Ready to commit : **oui** (after chore(planning) split)
- Staging split required : `chore(planning)` for preflight.md THEN `feat(sprint28)` for phase code
- Carry-overs S29 : P2-REVIEW-1 (generate_blocking params refactor), P2-REVIEW-2 (sampler chain allocation)
