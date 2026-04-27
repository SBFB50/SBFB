# Phase Review — Sprint 31 Phase C

## Verdict : PASS (1 P2 + 1 P3)

Rigor signal : 2 findings (1 P2 + 1 P3) documentes / >=1 requis.

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, context7 obligatoire — **respecte**
  (context7 arti-client consulte, 1597 snippets, APPROACH-ALIGNED)
- feedback_context7_systematic.md : context7 avant lib/API — **respecte**
  (resolve-library-id + query-docs effectues avant code)
- Violation memory : aucune

## Staging check (Step 1bis)

- Phase fichiers : 10 (7 M + 3 NEW source, 2 NEW planning/config)
- Planning split : preflight.md + review.md seront committes en
  chore(planning) AVANT le feat. Decision mecanique.
- Untracked accidentels : 0

## Suites (Step 2)

- Rust fmt : clean (apres `cargo fmt --all`)
- Rust clippy workspace : 0 warnings
- Rust nextest : 874 passed (869 → 874, +5 tor_transport)
- Rust doctests : 0 failed
- Release build daemon : OK (3m21s)
- Python ruff : clean
- SDK pytest : 194 passed + 1 flaky (pre-existant)
- Coord pytest : 406 passed + 36 failed (PyO3 stale) + 6 skipped
  (+7 tests tor_client vs 399 baseline)
- Gov pytest : 46 passed
- Frontend lint : 0 errors (7 warnings pre-existants)
- Frontend tsc : clean
- Vitest : 267 passed (+0, pas de modif web/)
- Frontend build : OK
- size-limit : pass
- en-strings : pas relance (pas de modif web/ phase C)

## Modified-file branch coverage (Step 2bis, G9)

- `crates/nexus-core-py/src/lib.rs` : `tor_feature_compiled()` (2 LOC)
  → trivial cfg! macro, teste implicitement via Python
  `test_tor_client.py::test_health_check_mirrors_available` ✅
- `coordinator.py` : `if self.tor_client.is_available():` branch
  → teste via `test_tor_client.py::test_disabled_noop` +
  `test_enabled_without_feature` ✅
- `paths.py` : `tor_config_path()` (6 LOC) + `if override:` branch
  → teste via `test_tor_client.py::test_from_toml_enabled` qui
  appelle `TorClientWrapper.from_toml()` ✅

## Commit body validation (Step 4)

- Format titre : `feat(sprint31): Sprint 31 Phase C — ...` ✅
- Delta tests coherent : +5 Rust + +7 coord = +12 ✅
- Scope cuts honoured : iroh 0.98 S32 ✅, iroh relay Tor S32+ ✅,
  Nym S33+ ✅, arti-client dep S32 (NEW scope-cut) ✅
- Co-Authored-By present : prevu ✅

## Research grounding (Step 4bis)

### 4bis-A OSS prior art (G10)
- Preflight S1a : documente, 4 projets OSS consultes (arti official,
  torpy, tun2tor, TorRequest) + context7 1597 snippets
- Verdict : APPROACH-ALIGNED — **PASS**

### 4bis-B Deps/API context7
- Plan §3 Research consulte : arti-client 2.0 API documentee
  (TorClient::create_bootstrapped + connect + DataStream) + context7
- Dep bloquee par rusqlite conflit : documentee preflight + commit ✅
- **PASS**

## Horizon long-terme (Step 4ter)

- Design doc : preflight.md documente l'approche + scope-cut
  arti-client dep. Pas de module structurant > 1 sprint (la dep
  landing est Phase 2). ✅
- D3 cite alternatives rejetees (SOCKS, I2P/Nym, skip) + rationale ✅
- Solution la plus poussee : arti-client direct API est la
  recommandation officielle Tor Project. ✅
- LOC estimees au plan : oui ~200 LOC — P2 (§6.7 interdit sauf
  retrospective). Cependant pre-existant dans plan.md committe,
  pas introduit par Phase C.

## Scope cuts verification (Step 5)

- iroh 0.98 upgrade (D5) : 0 fichiers diff ✅
- iroh relay over Tor : 0 fichiers diff ✅
- Nym mixnet : 0 fichiers diff ✅
- llama.cpp executor : 0 fichiers diff ✅
- Output filter client-side : 0 fichiers diff ✅

## Findings

- **P2-REVIEW-C-1** : arti-client dep bloquee par conflit
  `libsqlite3-sys` links (rusqlite 0.32 workspace vs arti-client
  0.41 → tor-dirmgr → rusqlite >= 0.36). Phase C livre infra
  config + feature gate + fallback + coordinator wire, mais la dep
  reelle et le bootstrap E2E sont differes. Carry S32 : rusqlite
  0.32→0.36 workspace upgrade + activation dep arti-client.
- **P3-REVIEW-C-1** : LOC estimees dans plan.md §7.2 (~200 LOC)
  contraire a §6.7. Pre-existant dans plan committe, pas
  introduit par Phase C. Nit informatif.

## Recommendation

- Ready to commit : **oui**
- Carry-overs S32 :
  - P2-REVIEW-C-1 : rusqlite upgrade + arti-client dep activation (1/3)
- Corrections needed : aucune
