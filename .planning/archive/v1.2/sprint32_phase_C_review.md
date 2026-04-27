# Phase Review — Sprint 32 Phase C

## Verdict : **PASS** (2 P2 + 1 P3)

Rigor signal G4 : 2 findings P2+ documentes (>=1 requis pour PASS).

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — respecte (max_tokens wired, not skipped) ✅
- feedback_context7_systematic.md : context7 obligatoire avant code — context7 indisponible session, WebSearch Ollama API substitute ✅

## Staging check (Step 1bis)
- Phase fichiers : 4 modified (`task_runner.rs`, `http.rs`, `HARDENING_ROADMAP.md`, `coordinator.py`) + 1 new (`blob-serve-coep.spec.ts`)
- Planning split : `sprint32_phase_C_preflight.md` → chore(planning) commit separe ✅
- Untracked accidentels : 0 ✅

## Suites (Step 2)
- Rust fmt : clean ✅
- Rust clippy : 0 warnings ✅
- Rust nextest : 878 → 883 (+5 : 1 max_tokens + 4 FROST error) ✅
- Rust doctests : clean ✅
- Release build daemon : success ✅
- Python ruff : clean ✅
- Python SDK : 195 pass ✅
- Python coord : 406 pass + 36 failed (PyO3 stale) + 6 skip ✅ (baseline)
- Python gov : 46 pass ✅
- Frontend lint : 0 errors ✅
- Frontend tsc : clean ✅
- Vitest : 267 pass ✅
- Frontend build : success ✅
- size-limit : 7/7 ✅
- Playwright : 42 pass + 2 fail (env) = 44 total (+1 COEP test) ✅
- en-strings : clean ✅

## Delta tests (Step 3)
- Rust : 878 → 883 (+5)
- Playwright : 43 → 44 (+1)
- Python/Vitest : inchanges
- **Total : ~1877 → ~1883 (+6)**

## Modified-file branch coverage (Step 2bis, G9)
- `task_runner.rs` : `GenerationOptions::default().num_predict()` wire (2 LOC) → tested by `execute_task_ollama_mock_respects_max_tokens` ✅
- `coordinator.py` : `elif self.tor_client.config.enabled:` log branch (1 LOC) → CONCERN (log-only defensive branch, tor_client module unit-tested but coordinator integration log not directly tested). Acceptable — cf. P3 below.
- `http.rs` : 4 new test functions only → self-verifying ✅
- `HARDENING_ROADMAP.md` : doc-only change → N/A
- `blob-serve-coep.spec.ts` : new test file → self-verifying ✅

## Research grounding (Step 4bis)
- 4bis-A OSS prior art : preflight S1a documented — Ollama API confirmed via WebSearch, ollama_rs 0.2.6 source read. APPROACH-ALIGNED. ✅
- 4bis-B Deps/API : plan §3 Research consulte references context7 iroh + crates.io + WebSearch. No new deps added Phase C. ✅

## Horizon long-terme (Step 4ter)
- Pas de nouveau module structurant Phase C (batch fixes) ✅
- D1..D5 avec alternatives + rationale : ✅ (kickoff §4)
- Solution la plus poussee : ✅ (direct wire `num_predict`, not workaround)
- Aucune LOC estimee au plan : ✅

## Scope cuts verification (Step 5)
12 scope cuts kickoff §7 : 0 fichiers diff touchent un scope cut ✅

## Findings

### P2-REVIEW-C-1 : HARDENING_ROADMAP arti-client version factually wrong

`docs/security/HARDENING_ROADMAP.md:3` — frontmatter `last_validated` referenced "arti-client 2.0" in two places. Correct version is 0.41.0 (crates.io crate, not Tor project version number). Propagation error from S31 kickoff pre-correction D2❌. **Fixed inline** this phase (same file already modified for counters).

### P2-REVIEW-C-2 : Playwright COEP test does not exercise real daemon blob-serve

`web/tests/blob-serve-coep.spec.ts` — test uses `page.route()` mock with COEP/COOP/CORP/CSP headers, not the real Rust blob-serve daemon. The defense-in-depth triple-layer (sandbox + CSP + COEP) is verified at browser level, but the Rust daemon header emission is only covered by the Rust unit constant `BLOB_SERVE_COEP`. A full E2E test requiring real daemon + blob-serve remains future work. **Carry S33** — note exemption: this is the best achievable without daemon fixture infrastructure.

### P3-REVIEW-C-1 : coordinator.py Tor log branch untested at integration level

`coordinator.py:379-380` — new `elif self.tor_client.config.enabled:` branch has no coordinator-level integration test. The tor_client module is unit-tested (6 tests). Log-only change, 1 LOC, defensive branch. Acceptable.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S33 : P2-REVIEW-C-2 (real daemon COEP E2E test — requires daemon fixture infra)
- Corrections needed : 0
