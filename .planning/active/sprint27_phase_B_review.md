# Phase Review — Sprint 27 Phase B

## Verdict : PASS

(Rigor signal : 2 findings P2 documentés / >=1 requis pour PASS rigoureux)

## Memory consultation (Step 1.5)

- feedback_approach.md : OSS prior art obligatoire (G10) — respecté
  (preflight S1a 3 projets consultés). Pick deepest — respecté
  (HMAC-SHA256 standard RustCrypto, pas custom hash). N/A band-aid.
- feedback_context7_systematic.md : context7 MCP non disponible session
  — compensé WebSearch (synthid-text, MarkLLM, hmac RustSec). N/A
  lib interne (module watermark nouveau).

## Staging check (Step 1bis)

- Phase fichiers : 10 (5 modified + 2 new code + 2 new tests + 1 Cargo.lock)
- Planning/docs split : chore(planning) fait `94d4077` (preflight)
- Untracked accidentels : 0

## Suites (Step 2)

- cargo fmt : clean
- cargo clippy : clean (0 warnings)
- Rust nextest : 807 passed → 807 (+4 Phase B : prf_determinism,
  prf_different_tokens_differ, compute_bias_applies_delta_to_green_only,
  should_inject_requires_both)
- Rust doctests : 0 (unchanged)
- Python ruff format + check : clean (150 files)
- Python SDK : 195 passed (+0)
- Python coordinator : 391 passed + 36 failed (known PyO3 wheel stale,
  identical baseline) + 4 new watermark tests in 391
- Python app-gov : 46 passed (+0)
- Frontend lint : 0 errors (7 warnings, unchanged)
- Frontend tsc : clean
- Frontend Vitest : 264 passed (+0)
- Frontend build : clean
- Frontend size-limit : 7/7 budgets
- scan-en-strings : clean
- Release build : nexus-shell-daemon --release clean

## Delta tests (Step 3)

| Suite | Avant | Après | Delta |
|---|---|---|---|
| Rust workspace (nextest) | 803 | 807 | +4 |
| Python coordinator | ~387 passing | ~391 passing | +4 |
| Python SDK | 195 | 195 | +0 |
| Python app-gov | 46 | 46 | +0 |
| Vitest | 264 | 264 | +0 |
| Playwright | — | — | +0 (non lancé, no frontend change) |
| **Total** | **~1768** | **~1776** | **+8** |

Plan §4 Phase B.3 ciblait +7 (4 Python + 3 Rust). Livré +8
(4 Python + 4 Rust). Over-delivery +1 Rust (prf_different_tokens_differ).

## Commit body validation (Step 4)

- Format titre : `feat(sprint27): Sprint 27 Phase B — ...` ✅
- Contexte : présent (SynthID-inspired, BIRA-resistant) ✅
- Fichiers touchés : listés avec rationale ✅
- Delta tests cumulé : cohérent avec Step 3 ✅
- Scope cuts honoured : listés ✅
- Co-Authored-By : présent ✅

## Research grounding (Step 4bis)

### 4bis-A — OSS prior art (G10)

- Preflight `sprint27_phase_B_preflight.md` S1a : 3 projets OSS
  consultés (google-deepmind/synthid-text, THU-BPM/MarkLLM,
  jwkirchenbauer/lm-watermarking). Verdict APPROACH-ALIGNED. ✅

### 4bis-B — Deps/API via context7

- **P2-B-1** : plan.md n'a pas de section §Research consulté formelle.
  Le preflight S1a/S1b couvre la recherche (WebSearch synthid-text,
  MarkLLM, hmac RustSec, COMPUTE_THREATS §4.5). Carry-over : ajouter
  §Research consulté aux futurs plans.

## Horizon long-terme + documentation amont (Step 4ter)

- Design doc : N/A (pas de nouveau module structurant > 1 sprint ;
  watermark est un composant Sprint 27 dont le design vit dans le
  plan §4.2 + preflight §S1a + COMPUTE_THREATS §4.5)
- D1 cite alternatives : ✅ (Kirchenbauer KGW rejeté BIRA, Tournament
  Sampling scope-cut S28+, bias additif simple retenu)
- Solution la plus poussée : ✅ (HMAC-SHA256 standard RustCrypto,
  pas custom hash ; z-test statistique standard vs heuristique ad-hoc)
- **P2-B-2** : kickoff contient LOC estimates prospectives (lignes 83,
  86, 88, 89 : ~1500/~500/~700/~300 LOC). Pre-existant (écrit au
  kickoff, pas Phase B). §6.7 interdit les estimations LOC
  prospectives. Carry-over S28 kickoff.

## Modified-file branch coverage (Step 2bis, G9)

- task.rs : `with_watermark_seed()` (3 LOC builder) — trivial,
  consistent avec `with_open_source()` / `with_estimates()` /
  `with_redundancy_factor()` non testés individuellement. CONCERN.
- config.rs : `WatermarkConfig::default()` (5 LOC) — exercé
  indirectement par `WorkerConfig::default()` tests existants +
  `should_inject(false, &[])`. PASS.
- verification.rs : field ajouté, 0 nouvelle branche. PASS.
- runtime.rs : field ajouté, 0 nouvelle branche. PASS.
- llm/mod.rs : `pub mod watermark;` ajouté, 0 nouvelle branche. PASS.

## Scope cuts verification (Step 5)

- Ollama backend watermark injection → S28+ : 0 modification
  ollama.rs ✅
- SynthID Tournament Sampling → S28+ : 0 mention "tournament" dans
  diff ✅
- Tor/Arti/Domain fronting/GPU lockup/etc. : 0 fichier touché ✅

## Findings (G4 rigor signal)

- **P2-B-1** : plan.md §Research consulté absent (compensé par
  preflight S1a). Carry-over documentation process.
- **P2-B-2** : kickoff LOC estimates prospectives (pre-existant).
  Carry-over S28 kickoff cleanup.
- **P3-B-1** : over-delivery +1 test Rust (prf_different_tokens_differ).
  Non-bloquant.

## Recommendation

- Ready to commit : **oui**
- Carry-overs S28 audit : P2-B-1 (§Research consulté), P2-B-2
  (LOC estimates kickoff)
- Corrections needed : aucune
