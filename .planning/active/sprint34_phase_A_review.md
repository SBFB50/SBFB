# Phase Review — Sprint 34 Phase A

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings P2+ documentés / >=1 requis pour PASS rigoureux.

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid → frost 3.0 upgrade au lieu de rester sur 2.x ✅
- feedback_context7_systematic.md : context7 avant code → frost eval via WebSearch agent, zip crate déjà workspace ✅

## Staging check (Step 1bis)
- Phase fichiers : 4 (Cargo.toml, Cargo.lock, nexus-test-harness/Cargo.toml, blob_serve_coep.rs)
- Planning docs : sprint34_phase_A_preflight.md → chore(planning) séparé
- Untracked hors-scope : 4 research docs `.planning/research/frontend_*` — pas touchés
- Decision : chore(planning) preflight AVANT feat commit

## Suites (Step 2)
- Rust nextest : 901 → 902 (+1 COEP E2E) ✅
- Rust doctests : 0 pass (1 ignored) ✅
- Rust fmt : clean ✅
- Rust clippy : clean ✅
- Release build : OK (3m33s) ✅
- Python SDK : 195 pass ✅
- Python coord : 409 + 36f (PyO3 stale) + 6s ✅
- Python gov : 46 pass ✅
- Frontend lint : 0 errors ✅
- Frontend tsc : clean ✅
- Vitest : 267 pass ✅
- Build : OK ✅
- size-limit : 7/7 ✅
- Playwright : 42 + 2f (pré-existants) ✅
- en-strings : clean ✅

## Modified-file branch coverage (Step 2bis)
N/A — seuls Cargo.toml/Lock modifiés (config). Le test COEP est un
nouveau fichier, pas un fichier existant modifié.

## Delta tests (Step 3)
- Rust : 901 → 902 (+1 blob_serve_coep_headers_on_real_zip)
- Python : inchangé
- Frontend : inchangé
- Total : ~1904 → ~1905 (+1)

## Scope cuts verification (Step 5)
- VPS deployment : ✅ non touché
- Code signing macOS : ✅ non touché
- MSI/NSIS installer : ✅ non touché
- .deb/.rpm packages : ✅ non touché
- Auto-update : ✅ non touché
- Tray icon : ✅ non touché
- frost-ed25519 3.0 upgrade : ✅ INCLUS (D4 critère rempli :
  0 LOC delta, signature byte-identical, 0 transitive conflict)
- CI pipeline : ✅ non touché
- stop/status CLI : ✅ non touché
- Cross-node Ollama réel : ✅ non touché
- Docker : ✅ non touché
- P3 grammar/watermark : ✅ non touché

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc : N/A (phase dette, pas nouveau module structurant)
- D1..D5 avec alternatives + rationale : ✅ (kickoff §4 complet)
- Solution la plus poussée : ✅ (frost 3.0 = dernier stable,
  COEP E2E = real daemon pas mock)
- LOC estimées au plan : ✅ 0 match

## Research grounding (Step 4bis)
- S1a OSS prior art : ✅ preflight documente APPROACH-ALIGNED
  (phase dette standard)
- context7/WebSearch : ✅ frost 3.0 évalué via agent WebSearch
  (changelog, API delta, signature format vérifié)
- zip crate : ✅ déjà workspace dep (v8.5), pas de recherche
  nécessaire

## Findings

### P2-A-1 : rand triple cohabitation non résolue — blocker upstream

rand 0.8 + 0.9 + 0.10 + getrandom 0.2 + 0.3 + 0.4 persistent
après `cargo update --aggressive` ET après frost 3.0 upgrade
(frost-core 3.0 utilise encore rand_core 0.6 → rand 0.8).
Sous-arbres disjoints, 0 impact runtime. Unification impossible
sans coordination upstream (frost-core + iroh stack convergent
sur des versions rand différentes).

**Action** : fermer comme bloquer externe documenté. Re-évaluer
si frost-core ou iroh migrent vers rand ≥ 0.10 unifié.

### P2-A-2 : cargo update aggressive a causé breakage pkcs8/ed25519

L'update aggressive initial a tiré `pkcs8 0.11.0` + `ed25519-3.0.0-rc.4`
(RC, pas stable) causant une erreur de compilation. Revert du
Cargo.lock + update ciblé a résolu. Le problème est que
`cargo update --aggressive` peut tirer des RC transitives non-stables.

**Action** : ne pas utiliser `--aggressive` en phase code. Préférer
des updates ciblés par crate. Carry S35 : documenter cette leçon
dans PATTERNS.md.

### P3-A-1 : frost-ed25519 DKG serialization format change silencieux

frost 3.0 ajoute un header version+ciphersuite aux structures
DKG sérialisées. Sous pre-launch protocol policy (0 DKG shares
déployées), c'est un non-issue. Mais si des fichiers `.frost.json`
de test existent, ils deviennent incompatibles.

**Action** : vérifier qu'aucun fichier `.frost.json` test fixture
n'existe dans le repo. Résultat : 0 match, non-issue confirmé.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S35 : P2-A-1 rand triple (bloquer upstream, re-évaluer),
  P2-A-2 aggressive update lesson (PATTERNS.md)
- Corrections needed : aucune
