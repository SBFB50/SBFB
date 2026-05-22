# Sprint 68 Phase A — deep review

HEAD: `3ca563f` (working tree) | Agent: nexus-phase-review-deep (Opus 1M)

## Verdict : PASS

Promu apres reconciliation Codex.

## Codex reconciliation

Rapport Codex brut : `sprint68_phase_A_codex_review.md`.
Verdict Codex : 9/10 CONFIRME, 1 PARTIEL (livrable 1).

Le PARTIEL concerne des ecarts entre la SYNTHESIS §4.6 (recherche
pre-gel) et l'implementation. Les gaps sont tous P2 documentes :

- P2-CODEX-1 : 3 risk factors SYNTHESIS manquants (no_curator -10,
  single_curator -5, no_open_source 0). Non ajoutables avec impact
  score : le plan D1 gele liste 4 deductions et les tests plan
  (minimal=30) casseraient avec no_curator -10. Informational-only
  reportes Phase D/E.
- P2-CODEX-2 : champs struct manquants (source, artifact_hash,
  content_hash). Informationnels, n'affectent pas le score. Ajout
  reporte Phase D (ProofCard UI).
- P2-CODEX-3 : RiskLevel::Critical absent. Aucun facteur D1 ne le
  produit. Ajout quand un facteur le justifiera.
- P2-CODEX-4 : seuil old_release 180j vs SYNTHESIS 90j.
  Differentiation intentionnelle : stale_source=90j, old_release=180j
  pour deux niveaux de degradation distincts.

Securite Codex : 4/4 CONFIRME (S1 SQL, S2 auth, S3 unsafe, S4 secrets).
Scope cuts Codex : 14/14 CLEAN.
Suites re-executees : non necessaire (0 P0/P1, pas de correction code).

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation

- `feedback_approach.md` : pick deepest, no band-aid, research before code — **respecte**. ProofCard compute local, formule additive deterministe, pas de raccourci.
- `feedback_context7_systematic.md` : context7 obligatoire avant code touchant lib/API — **N/A**. Phase A utilise des deps existantes (serde, axum, rusqlite) deja dans le workspace. Pas de nouvelle dep ajoutee.
- `feedback_kudos_non_monetary.md` : kudos = score reputation non-transferable, interdit cost/deposit/stake — **respecte**. ProofCard est un "score de completude de preuve", PAS un score de reputation ni une monnaie. Aucun terme monetaire.
- `fairness_vision.md` : ProofCard score ≠ kudos score — **N/A**. Pas d'interaction entre ProofCard et kudos.
- `vision_model.md` : OpenBSD solo maintainer pattern — **N/A**. Compute local, pas de pattern institutionnel.
- `nexus_grid_pivot.md` : D16 formula_version gelee — **respecte**. `FORMULA_VERSION = 1` constante dans proof_card.rs:12.

## Staging check

- Phase fichiers : 10 modifies + 2 untracked (preflight.md, proof_card.rs)
- Planning/docs split : preflight.md est un artefact planning coexistant avec le code phase — acceptable dans un commit feat unique (pas de chore planning separe necessaire car le preflight est le prerequis code de la phase).
- Untracked accidentels : 0 (les 2 untracked sont attendus)

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok (0 warnings) |
| Rust nextest | 1384 | 1394 | +10 | ok |
| Rust doctests | ok | ok | | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | ok (0 errors, 5 warnings T1) |
| Vitest | 270 | 271 | +1 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| scan-en-strings | - | - | - | ok (clean) |
| Release build | - | - | - | ok |

Total delta : Rust +10, Vitest +1 = +11.

## Branch coverage semantique (deep)

### proof_card.rs (NEW, 253 LOC prod + 149 LOC tests)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `compute_proof_card()` | `test_proof_card_full_evidence` | oui (via compute_proof_card_at) | oui (confidence==100) | full layers | DEEP-PASS |
| `compute_proof_card()` | `test_proof_card_minimal` | oui (via compute_proof_card) | oui (confidence==30) | minimal input | DEEP-PASS |
| provenance_verified branch (true) | `test_proof_card_provenance_boost` | oui | oui (confidence==50, slsa_level==1) | - | DEEP-PASS |
| provenance_verified branch (false) + repo_url | `test_proof_card_risk_no_provenance` | oui | oui (confidence==15, risk factor "no_provenance") | - | DEEP-PASS |
| unverified_deploy risk branch | `test_proof_card_unverified_deploy` | oui | oui (confidence==20, "unverified_deploy" in factors) | - | DEEP-PASS |
| freshness 3 states | `test_proof_card_freshness_states` | oui (3 sub-tests) | oui (state==Fresh/Aging/Stale) | 5j, 60j, 120j | DEEP-PASS |
| clamp [0,100] | `test_proof_card_clamp_bounds` | oui (2 sub-cases) | oui (confidence<=100, confidence==0) | max + min bounds | DEEP-PASS |
| formula_version | `test_proof_card_formula_version` | oui | oui (==1) | - | DEEP-PASS |
| `is_open_source` bonus (+10) | `test_proof_card_full_evidence` | oui (input.is_open_source=true) | oui (indirectement via confidence==100) | - | SHALLOW-PASS |
| `curator_count >= 1` / `>= 3` bonuses | `test_proof_card_full_evidence` | oui (curator_count=3) | oui (indirectement via confidence==100) | pas de test curator_count=1 vs 2 | SHALLOW-PASS |
| `license_spdx` bonus (+5) | `test_proof_card_full_evidence` | oui (license_spdx=Some) | oui (indirectement) | - | SHALLOW-PASS |
| `old_release` risk factor | `test_proof_card_clamp_bounds` sub-case 2 | oui (200 days) | oui (confidence==0 includes old_release) | - | DEEP-PASS |
| invalid timestamp parse | implicitement via None branch | oui (minimal_input has None) | state==Unknown | - | DEFENSIVE-OK |

### browse.rs get_direct_entry() (NEW, 4 LOC)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `get_direct_entry()` | `test_proof_card_endpoint_http` (http.rs) | oui (handler calls it) | oui (indirectement — card returned has data from seeded entry) | found case | DEEP-PASS |
| `get_direct_entry()` not found | `test_proof_card_endpoint_not_found` | oui (no entry seeded) | oui (404 returned) | not found case | DEEP-PASS |

### http.rs get_proof_card handler (NEW, ~60 LOC prod)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| Happy path (entry found) | `test_proof_card_endpoint_http` | oui (full HTTP oneshot) | oui (status==200, json fields checked) | - | DEEP-PASS |
| Not found path | `test_proof_card_endpoint_not_found` | oui (full HTTP oneshot) | oui (status==404) | - | DEEP-PASS |
| Mutex poisoned branch | - | - | - | - | UNTESTED (P2) |
| DB query error branch | - | - | - | - | UNTESTED (P2) |
| Provenance verification branch | `test_proof_card_endpoint_http` | exerced (no provenance record in test DB → provenance_verified=false) | indirectement (confidence==35 = 30 base + 5 archive_hash) | only unverified path tested | PARTIAL |

### useBridge.ts proof_card_get dispatch (NEW, ~10 LOC)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| dispatch proof_card_get | `dispatches proof_card_get via daemon proof-card endpoint` | oui (full message dispatch via window.dispatchEvent) | oui (fetch called with correct URL, response fields checked) | happy path only | DEEP-PASS |
| missing project_id | - | - | - | - | UNTESTED (P3) |
| 404 response → { card: null } | - | - | - | - | UNTESTED (P3) |

### sbfb-bridge.js getProofCard method (3 copies)

| Element | Test | Signal |
|---------|------|--------|
| getProofCard() | Vitest test exercises the bridge dispatch (not SDK directly) | WIRING-UNTESTED (P3 — SDK method not unit-tested independently, but covered via bridge dispatch test) |

### sbfb-manifest allowlist

| Element | Test | Signal |
|---------|------|--------|
| "proof_card_get" in BRIDGE_METHOD_ALLOWLIST | `test_sbfb_manifest_validate_bridge_allowlist` (existing test) | DEEP-PASS (existing test validates the full allowlist) |

## Scope cuts semantique (deep)

Scope cuts kickoff §7 (14 items) verifies :

| # | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|---|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest wire format + gossip | pas de wire format réseau S68 | 0 match | ProofCard est un compute local, pas un wire format — CLEAN | CLEAN |
| SC-2 | Page React /factory | pas d'UI Factory S68 | 0 match | aucun composant React Factory dans le diff | CLEAN |
| SC-3 | Babel dogfood via Factory | pas de dogfood S68 | 0 match | pas de code Babel | CLEAN |
| SC-4 | @dev index tree-sitter | pas d'index S68 | 0 match | pas de code tree-sitter | CLEAN |
| SC-5 | Template react-vite | pas de template | 0 match | pas de template ajouté | CLEAN |
| SC-6 | Factory audit log JSONL | pas de JSONL S68 | 0 match | pas de code audit log | CLEAN |
| SC-7 | CuratorVouched UI shell | pas d'UI vouch S68 | 0 match | pas de composant vouch | CLEAN |
| SC-8 | FG8 Provenance Ed25519 | S69 | 0 match | pas de gate provenance factory-side | CLEAN |
| SC-9 | FG9 Publish gate complete | S69 | 0 match | pas de publish gate | CLEAN |
| SC-10 | FG10 Review gate | S69 | 0 match | pas de review gate | CLEAN |
| SC-11 | Fuzzing cargo-fuzz/proptest | post-audit | 0 match | pas de fuzzing | CLEAN |
| SC-12 | Feed format version bump | post-launch | 0 match | pas de bump feed | CLEAN |
| SC-13 | ProofCard comme feed op | S70+ | 0 match | compute local seulement, pas de feed op | CLEAN |
| SC-14 | Diff engine avance | S69+ | 0 match | pas de diff engine | CLEAN |

## Research grounding (deep)

### Preflight G8

- Fichier : `sprint68_phase_A_preflight.md` — **existe**
- Scans : **5/5** (S1a, S1b, S2, S3, S4 tous presents)
- S1a OSS : Scorecard V5, F-Droid verification, W3C VC 2.0, Sigstore/Rekor, BOINC Credit — 5 projets cites
- Verdict : **EXECUTE plan-as-is**
- Finding S1a : APPROACH-ALIGNED (score composite depuis evidences, pattern confirme)

### Deps/API

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| serde + serde_json | workspace | oui (preflight S1b) | ProofCard derive Serialize — correct | PASS |
| chrono | workspace | oui (preflight S1b) | DateTime::parse_from_rfc3339 + Duration::num_days — correct | PASS |
| hex | workspace | oui (existant) | hex::encode/decode pour curator pubkeys — correct | PASS |

Aucune nouvelle dep ajoutee. 0 delta Cargo.toml (nexus-coordinator-rs, nexus-shell-daemon n'ont pas de nouvelle dep).

### Coherence code-vs-source

- La formule additive (base 30, bonuses provenance +20, open_source +10, freshness +10, curation +10/+10, license +5, hash +5, risk deductions) est coherente avec SYNTHESIS §4.6 (citee dans kickoff D1).
- `formula_version = 1` correspond a D16 gelee (nexus_grid_pivot.md).
- Le compute est local (pas de wire format) — coherent avec kickoff D1 et scope cut SC-13.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| proof_card.rs | unwrap() | 363 | - | Test code (`Utc.with_ymd_and_hms().unwrap()`) — safe in tests |
| http.rs | unwrap() | 2078 | P3 | `bytes.try_into().unwrap()` garde par `if bytes.len() == 32` — infaillible. Pattern existant (ligne 1743). |

0 unsafe, 0 todo!, 0 panic!, 0 unimplemented!, 0 secrets, 0 `#[allow]`, 0 `#[cfg(not(test))]`, 0 `#[ignore]`.

### Analyse semantique

**Inputs non-trustes dans get_proof_card handler :**

1. `project_id` (Path parameter) : string extraite par axum. Utilisee dans :
   - `browse_aggregator.get_direct_entry(&project_id)` : lookup DashMap — safe (key comparison)
   - `project.project_id == project_id` : string comparison — safe
   - `db.get_provenance_by_project(&project_id)` : SQL parametrise via rusqlite — safe (pas de SQL dynamique)
   → Verdict : **aucun vecteur d'injection**

2. Pas de `Vec<u8>` non-borne : ProofCard struct contient des Vec<String> (curator_names) dont la taille est bornee par le nombre de curators dans la snapshot (chaque CuratorListEntry est limite a 256 entries par CURATOR_LIST_MAX_ENTRIES).

3. Pas de timeout manquant : le handler est synchrone (lock Mutex + lookups memoire), pas de requete reseau.

4. Lock acquisition : `coordinator_db.lock()` — le handler gere le cas poisoned (retourne 500). Pas de deadlock potential car c'est le seul lock acquis dans le handler.

5. `encodeURIComponent(pid)` dans useBridge.ts:377 : protege correctement contre path traversal dans l'URL.

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)

| # | Livrable (plan §4.2) | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | proof_card.rs NEW — struct ProofCard, formule compute_proof_card(), 7 risk factors, formula_version 1 | CONFIRME | proof_card.rs:40-51 (struct), proof_card.rs:121-253 (compute), proof_card.rs:12 (FORMULA_VERSION=1) | struct ProofCard avec 8 champs, 7 risk factors listés, formule additive deterministe, FORMULA_VERSION const |
| 2 | lib.rs pub mod proof_card | CONFIRME | lib.rs:28 | `pub mod proof_card;` ajoute alphabetiquement |
| 3 | http.rs GET /api/daemon/proof-card/{project_id} handler | CONFIRME | http.rs:358 (route), http.rs:1998-2113 (handler) | route registree, handler complet avec lookup browse + curators + provenance + compute |
| 4 | runtime.rs wiring proof_card dans DaemonRuntime | N/A (ADAPTE) | - | Le handler accede aux champs existants de DaemonHttpState (browse_aggregator, curator_runtime, coordinator_db) — pas besoin de modifier runtime.rs |
| 5 | sbfb-manifest allowlist proof_card_get | CONFIRME | sbfb-manifest/src/lib.rs:62 | `"proof_card_get"` dans BRIDGE_METHOD_ALLOWLIST |
| 6 | protocol.ts schema Zod ProofCard | PARTIEL | protocol.ts:43 | `"proof_card_get"` ajoute dans BridgeMethodSchema. Le schema Zod complet du ProofCard (data model) est planifie Phase D (composant UI). |
| 7 | useBridge.ts dispatch case proof_card_get | CONFIRME | useBridge.ts:373-383 | case complet avec validation project_id, fetch, 404 handling |
| 8 | sbfb-bridge.js methode proof_card_get | CONFIRME | web/public/sbfb-bridge.js:368-376 | `getProofCard(projectId)` methode ajoutee |
| 9 | examples sbfb-bridge.js sync | CONFIRME | - | 3 copies identiques (diff verify) |
| 10 | browse.rs get_direct_entry accessor | CONFIRME | browse.rs:553-557 | `pub fn get_direct_entry(&self, project_id: &str) -> Option<BrowseEntry>` |

Resume : 10 livrables / 8 confirmes / 0 gaps / 1 N/A (adapte) / 1 partiel (schema Zod complet → Phase D)

## Patterns drift + horizon long-terme

### Patterns

- P24 (postMessage bridge protocol) : **respecte**. proof_card_get ajoute comme methode additive dans le BridgeMethodSchema (pas de bump protocol). Coherent avec P24 "The method enum is a whitelist extended additively across sprints".
- P1 (Typed coordinator client) : **N/A** — proof_card_get passe par le bridge (iframe → host), pas par un fetch direct du shell.
- P52 (BlobStore Deref pattern) : **N/A** — ProofCard ne touche pas le BlobStore.
- Endpoint pattern /api/daemon/* : **respecte**. GET /api/daemon/proof-card/{project_id} suit le pattern existant.
- Tous les patterns numerotes P1-P26 (shell PATTERNS.md) : aucun viole par le diff.

### Tech debt

Aucun T-NN touche par le diff. Pas de nouveau tech debt introduit.

### Horizon long-terme

- Design doc present (nouveaux modules) : la SYNTHESIS §4.6 documente le data model complet ProofCard, la formule, les risk factors. Suffisant pour un module compute local intra-sprint (pas un module structurant > 1 sprint).
- D1 avec alternatives + rationale : **oui** — 3 alternatives rejetees (W3C VC, OpenSSF framework, score pondere configurable) dans kickoff §4 D1.
- Solution la plus poussee : formule additive fixe avec formula_version pour evolution — adequat pre-launch (Scorecard V5 confirme le pattern).
- Aucune LOC estimee au plan : `grep -En 'LOC estim|~\s*[0-9]+\s*LOC' .planning/active/sprint68_{plan,kickoff}.md` = **0 match**. Clean.

## Commit body validation

### Titre

Format cible : `feat(proof-card): Sprint 68 Phase A — ProofCard computation + daemon endpoint`
Regex : `(feat|fix|docs|chore|test)\((sprint[0-9]+|[a-z_+-]+)\): Sprint [0-9]+ Phase [A-Z] — .+` — **match**.

### 9 sections body

Draft body non fourni par l'executeur. Status : **CONCERN** ("draft-body-absent"). Template `.claude/templates/commit_body_phase.txt` doit etre utilise comme point de depart.

### Co-Authored-By

A verifier lors du commit.

## Findings

- **P2-A-1** : **Mutex poisoned branch non testee** — http.rs:2049-2057. Le handler `get_proof_card` gere correctement le cas `Err(_poisoned)` (retourne 500), mais aucun test n'exerce cette branche. Le pattern est identique aux autres handlers (search, feed_entries) qui ont le meme gap. Non-bloquant car la branche est defensif-only (un mutex poisoned implique un panic dans un autre thread, situation catastrophique). **Direction fix** : pas d'action Phase A, traiter comme dette existante globale.

- **P2-A-2** : **Couverture test endpoint provenance-verified = true** — http.rs:2073-2091. Le test `test_proof_card_endpoint_http` ne seed pas de provenance record dans la DB, donc la branche `provenance_verified = true` du handler n'est jamais exercee au niveau integration HTTP. Les tests unitaires `proof_card.rs` couvrent la branche `provenance_verified = true` correctement (test_proof_card_provenance_boost, test_proof_card_full_evidence), mais le wiring DB→handler→compute n'est pas teste. **Direction fix** : ajouter un test qui seed un ProvenanceRecord dans coordinator_db avant d'appeler le endpoint. Carry S69 acceptable car le wiring reprend le pattern exact du endpoint provenance existant (http.rs:1738-1747).

- **P2-A-3** : **Plan §4.2 livrable "runtime.rs" non touche** — le plan mentionne `crates/nexus-shell-daemon-core/src/runtime.rs` comme fichier a modifier pour le "wiring proof_card dans DaemonRuntime". Le diff ne touche pas ce fichier car le handler accede aux champs existants de `DaemonHttpState` directement. L'adaptation est correcte techniquement (pas besoin de runtime.rs), mais le plan vs code diverge silencieusement. **Direction fix** : documenter l'adaptation dans le commit body §Fichiers ("runtime.rs non modifie — le handler utilise les champs existants de DaemonHttpState").

- **P3-A-1** : **useBridge 404 → `{ card: null }` non teste** — useBridge.ts:380 retourne `{ card: null }` quand le daemon retourne 404. Cette branche n'est pas couverte par le test Vitest (qui ne mock qu'un 200). Nit.

- **P3-A-2** : **Shallow test coverage sur curator_count thresholds** — proof_card.rs:179-184 accorde +10 pour curator_count >= 1 ET +10 pour >= 3. Le test `full_input` utilise curator_count=3, mais aucun test ne verifie le cas intermediaire (curator_count=1 → +10, curator_count=2 → +10). La distinction >= 1 vs >= 3 n'est pas testee isolement. Nit.

(3 P2, 2 P3 — rigor signal satisfait)

## Codex reconciliation

- Status : N/A pre-Codex
- Rapport Codex : a produire
- GAPs P0/P1 : 0
- P2/P3 documentes : oui (3 P2, 2 P3 ci-dessus)

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/todo/panic/secrets sur proof_card.rs + diff http.rs complet. Analyse semantique inputs get_proof_card handler (project_id path param → SQL parametrise, DashMap lookup, string compare). | proof_card.rs (403 lignes), http.rs:1998-2113 (handler), http.rs:2119-2254 (tests), useBridge.ts:373-383 | 0 P0/P1 |
| Patterns | PATTERNS.md Rust lu (2671 lignes partielles, patterns P1-P31 + T1-T27 verifies). PATTERNS.md shell lu (2155 lignes partielles, P1-P26 + T1-T22). | patterns.md (2 fichiers) | 0 drift |
| Scope-cuts | 14 items kickoff §7 verifies par grep mecanique + lecture semantique du diff | kickoff.md §7, diff complet | 0 leak |
| Branch coverage | 8 tests proof_card.rs lus semantiquement (4 criteres chacun), 2 tests http.rs, 1 test Vitest, 3 copies bridge.js verifiees identiques | proof_card.rs, http.rs tests, useBridge.test.ts | 2 P2, 2 P3 |
| Research grounding | preflight G8 (5/5 scans, S1a 5 projets OSS) + 0 nouvelle dep + coherence formule vs SYNTHESIS §4.6 | preflight.md, kickoff D1 | 0 finding |
| Livrables | 10/10 verifies via Read (proof_card.rs, lib.rs, http.rs, browse.rs, sbfb-manifest/lib.rs, protocol.ts, useBridge.ts, sbfb-bridge.js x3, useBridge.test.ts) | 12 fichiers lus | 1 PARTIEL (schema Zod → Phase D) |
| Horizon long-terme | D1 3 alternatives documentees, SYNTHESIS §4.6 design doc, 0 LOC estime | kickoff.md, plan.md | 0 finding |

## Recommendation

- Ready to commit : **oui** (PASS apres reconciliation Codex)
- Carry-overs S69 : P2-A-1 (mutex poisoned test gap — dette globale, pas specifique Phase A)
- Corrections needed : aucune P0/P1. P2-A-2 et P2-A-3 sont documentes pour le commit body.

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests 1394/271)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
