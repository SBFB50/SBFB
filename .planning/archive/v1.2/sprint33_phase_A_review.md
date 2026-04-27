# Phase Review — Sprint 33 Phase A

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings documentes (2 P2 + 1 P3), >=1 requis pour PASS rigoureux.

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, context7 before code — context7 MCP non disponible, mitige par research kickoff same-day — respecte
- feedback_context7_systematic.md : tower-http + FastAPI CORS couverts par kickoff §Sources — respecte
- feedback_full_failfast.md : 3 blocs complets (Rust+Python+Frontend) — respecte

## Staging check (Step 1bis)
- Phase fichiers : 13 modifies (12 M + 1 Cargo.toml nit extra)
- New files : 2 (test_cors.py, sprint33_phase_A_preflight.md)
- Planning split : preflight.md + review.md → chore(planning) AVANT feat — mecanique
- Untracked accidentels : 0 dans scope Phase A

## Suites
- Rust nextest : 883 → 893 (+10) ✅
- Rust doctests : 0 fail, 1 ignored ✅
- Rust clippy : 0 warnings ✅
- Rust fmt : clean ✅
- Release build daemon : Finished ✅
- SDK : 195 pass ✅
- Coordinator : 406 → 409 (+3) + 36f PyO3 stale + 6s ✅
- Gov : 46 pass ✅
- Vitest : 267 pass ✅
- Playwright : 42 pass + 2f (env, baseline) ✅
- size-limit : 7/7 ✅
- en-strings : clean ✅
- Frontend lint : 0 errors ✅
- Frontend tsc : clean ✅
- Frontend build : success ✅

## Commit body validation
- Format titre : ✅ `feat(sprint33): Sprint 33 Phase A — CORS external access + LOC guard + P3 nits`
- Delta tests coherent : plan annonce +8, reel +13 (surplus = 4 is_valid_origin + 1 CLI test) ✅
- Scope cuts honoured : ✅
- Co-Authored-By present : a verifier au commit

## Modified-file branch coverage (Step 2bis, G9)
- http.rs : `cors_layer(extra_origins)` (14 LOC) → tested by 5 CORS tests ✅
- http.rs : `is_valid_origin(s)` (18 LOC) → tested by 4 unit tests ✅
- http.rs : `build_router(..., cors_origins)` signature change → tested by all existing 39+ tests ✅
- cli.rs : `cors_origins: Vec<String>` field → tested by `parses_start_with_cors_origins` ✅
- main.rs : env fallback + validation branch → exercised at runtime boot, validation tested via `is_valid_origin` ✅
- app.py : `cors_origins` param + middleware kwargs → tested by 3 Python CORS tests ✅
- start.py : CLI option + env merge → exercised at boot ✅
- lightcheck.sh : check 6 LOC guard → manual verification (hook is bash, not unit-testable) — CONCERN

## Scope cuts verification
- VPS deployment : 0 fichiers ✅
- Mobile browser testing : 0 fichiers ✅
- iroh relay over Tor : 0 fichiers ✅
- Docker daemon/worker : 0 fichiers ✅
- stop/status CLI : 0 fichiers ✅
- Build CI merge : 0 fichiers ✅
- Cross-node task (Ollama) : 0 fichiers ✅
- Output filter client-side : 0 fichiers ✅

## Research grounding (Step 4bis)
- S1a OSS prior art : present dans preflight, APPROACH-ALIGNED ✅
- Plan §Research consulte : present (tower-http 0.6, FastAPI, iroh 0.98) ✅
- 0 nouvelle dep ajoutee/bumpee → N/A ✅

## Horizon long-terme (Step 4ter)
- D1..D5 avec alternatives rejetees + rationale : ✅ (kickoff §4)
- Solution la plus poussee : tower-http CorsLayer = canonical axum ✅
- Aucune LOC estimee au plan : ✅

## Findings

### P2-1 : hook LOC guard (check 6) non testable automatiquement
Le check 6 dans lightcheck.sh est un script bash qui grep les plans
staged. Il n'est exercé par aucun test automatise — seul un test
d'integration manuelle (stager un plan avec `~500 LOC` et verifier
que le hook bloque) le couvre. Le hook est structurellement simple
(grep + exit 2) donc le risque de regression est faible, mais
l'absence de test automatise est un gap documenté.
**Carry-over** : ajouter un test d'integration hook dans un sprint
futur (ou le premier audit gate qui touche le hook).

### P2-2 : env fallback `NEXUS_DAEMON_CORS_ORIGINS` non teste unitairement
La branche env fallback dans `main.rs` (lignes ~97-106) parse la
variable d'environnement comma-separated. Elle est couverte par le
chemin boot (runtime start test) mais pas par un test unitaire
dedie qui injecte la var et verifie le parsing. Risque faible (le
parsing est trivial), carry-over S34 si le daemon ajoute plus
d'env vars.

### P3-1 : residuel `iroh 0.97` dans Cargo.toml comment
Trouve et corrige pendant la review (Cargo.toml:43 "iroh 0.97" →
"0.98"). Aussi http.rs:1081 "Post-0.97" → "Post-0.98". Les deux
sont maintenant clean.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S34 : P2-1 hook test integration, P2-2 env var unit test
- Corrections : P3-1 corrige inline
