# Sprint 69 Phase C — deep review

HEAD: 1edaaa6 (chore planning) + working tree | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

(Rigor signal : 2 findings P2+P3 documentes / >=1 requis pour PASS)

## Codex reconciliation
- Codex GPT 5.5 (session 019e5037-913b-7662-a407-d159949bf5de) : 5/5 CONFIRME, 0 GAP, 0 PARTIEL
- Rapport brut non modifie : `.planning/active/sprint69_phase_c_codex_review.md`
- Suites non relancees (0 correction)
- Verdict promu de PASS-PENDING a PASS

## Memory consultation
- `feedback_approach.md` : pick deepest, research before code — Phase C est un template statique, le refacto multi-template est l'option la plus poussee entre "pas de template" (rejete D2) et "react-vite" (surdimensionne). Respecte.
- `vision_model.md` : solo maintainer OpenBSD — un template CLI est coherent. N/A.
- `feedback_context7_systematic.md` : Phase C ne touche aucune lib tierce nouvelle (include_str! compile-time). N/A.
- `feedback_kudos_non_monetary.md` : N/A (pas de kudos).
- `fairness_vision.md` : N/A (pas de scoring).

## Staging check
- Phase fichiers : 1 modified (template_engine.rs) + 4 new (templates/static-reader/*)
- Planning/docs split : chore fait oui (1edaaa6 pour research docs)
- Untracked accidentels : 2 (.planning/research/rrv_*.md — pre-existants, pas Phase C)

## Suites verification
| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok (0 warnings) |
| Rust nextest | 1430 | 1433 | +3 | ok (1433/1433) |
| Rust doctests | ok | ok | +6 | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | ok (0 errors, 5 warnings pre-existants T1) |
| Vitest | 279 | 279 | +0 | ok (279/279) |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| Playwright | N/A | N/A | - | N/A (pas de changement frontend) |
| scan-en-strings | N/A | N/A | - | N/A (pas de changement frontend) |
| Release build | - | - | - | ok |

## Branch coverage semantique (deep)
| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `STATIC_READER_TEMPLATE` const | `test_create_static_reader_template` | oui (via `create("static-reader", ...)`) | oui (7 fichiers asserts) | N/A | DEEP-PASS |
| `TemplateConfig` struct + `TEMPLATES` array | `test_create_static_reader_template` + `test_validate_static_reader_passes` | oui | oui (category="content", bridge 3 methodes) | "static" et "static-reader" testes | DEEP-PASS |
| `find_template()` happy path | `test_create_static_reader_template` | oui | oui (reussite creation) | 2 templates testes | DEEP-PASS |
| `find_template()` error path | tests existants (`test_path_traversal_rejected`) | indirect (via validate→expected_files) | oui (FactoryError returned) | N/A | DEEP-PASS |
| `bridge` config: empty vs populated | `test_create_generates_sbfb_json_v2` (static, bridge=None) + `test_validate_static_reader_passes` (static-reader, bridge=Some) | oui les 2 branches | oui (bridge.is_some + methods.contains) | 2 cotes testes | DEEP-PASS |
| `substitute()` sur index.html | `test_static_reader_template_substitution` | oui | oui (contains "babel-reader", !contains "{{name}}") | name verifie | DEEP-PASS |
| `substitute()` sur README.md | `test_static_reader_template_substitution` | oui | oui (contains "babel-reader", !contains "{{name}}", !contains "{{version}}") | multi-fichier substitution | DEEP-PASS |
| `expected_files()` refactored | tests existants pour "static" template | oui | oui | meme logique, meme resultat | DEEP-PASS |
| Navigation JS prev/next | N/A (runtime JS, pas de test Playwright) | - | - | - | DEFENSIVE-OK |
| `saveCursor()`/`loadCursor()` bridge | N/A (runtime JS) | - | - | - | DEFENSIVE-OK |

## Scope cuts semantique (deep)
| Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|-----------|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest wire format + gossip | Pas de protocole reseau | 0 match | 0 code reseau | CLEAN |
| SC-2 | Page React /factory | Pas de page UI Factory | 0 match | 0 fichier web/src/ modifie | CLEAN |
| SC-3 | @dev index tree-sitter | Pas de tree-sitter | 0 match | 0 code analyse source | CLEAN |
| SC-4 | Template react-vite | Pas de template react | 0 match | template = static HTML, pas react-vite | CLEAN |
| SC-5 | CuratorVouched UI shell | Pas de curation UI | 0 match | 0 code frontend | CLEAN |
| SC-6 | FG10 Review gate | Pas de lint automatise | 0 match | 0 code analyse statique | CLEAN |
| SC-7 | Fuzzing cargo-fuzz/proptest | Pas de fuzz | 0 match | 0 code fuzz | CLEAN |
| SC-8 | Feed format version bump | Pas de bump version | 0 match | 0 constante VERSION touchee | CLEAN |
| SC-9 | ProofCard comme feed op | Pas de feed op | 0 match | 0 code feed | CLEAN |
| SC-10 | Diff engine avance | Pas de diff semantique | 0 match | 0 code diff | CLEAN |
| SC-11 | Multi-template switching UI | Pas de UI template switch | 0 match | CLI seulement | CLEAN |
| SC-12 | Factory update-check | Pas d'auto-update | 0 match | 0 code telemetrie | CLEAN |
| SC-13 | Babel traduction live | Pas de moteur LLM | 0 match | placeholder statique | CLEAN |
| SC-14 | iroh 1.0 upgrade | Pas d'upgrade iroh | 0 match | 0 Cargo.toml modifie | CLEAN |

## Research grounding (deep)
### Preflight G8
- Fichier : existe (`sprint69_phase_c_preflight.md`)
- Scans : 5/5 (S1a + S1b + S2 + S3 + S4)
- S1a OSS : 5 projets cites (project-scaffold, cargo-scaffold, Vite create, mdBook, static-book-webpage-template)
- Verdict : EXECUTE plan-as-is
- APPROACH-ALIGNED documente avec evidence

### Deps/API
Phase C ne modifie pas Cargo.toml. 0 dep ajoutee, 0 dep bumpee.

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| (aucune nouvelle dep) | - | - | - | PASS |

### Coherence code-vs-source
- `include_str!` embedding : coherent avec project-scaffold / Vite create (S1a)
- Navigation prev/next sections JS : coherent avec mdBook / static-book-template (S1a)
- Multi-template routing `find_template()` : coherent avec Vite `--template` (S1a)
- BridgeConfig methods `["storage_get", "storage_set", "identity_pubkey"]` : les 3 sont dans BRIDGE_METHOD_ALLOWLIST (sbfb-manifest/src/lib.rs:52-63). Coherent.

## Security deep
### Scan automatique
| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| index.html | innerHTML | 95 | P3 | Contenu statique meme fichier, pas d'input dynamique. Mitigue iframe sandbox + CSP. |

Aucun `unwrap`, `unsafe`, `todo!`, `panic!`, `#[allow(dead_code)]`, secrets detectes dans le diff.

### Analyse semantique
- **Inputs non-trustes** : le template genere par `create()` est un outil CLI local. Seul input = `--name` substitue dans le HTML. Pas de deserialization reseau, pas de wire format, pas de loopback HTTP.
- **Path traversal `--template`** : `find_template()` fait un match exact sur `TEMPLATES` array. Aucune resolution filesystem basee sur le template name.
- **Bridge SDK** : embarque via `include_str!` au compile-time. Pas modifiable a runtime.
- **innerHTML index.html:95** : contenu `sections[i].content` defini statiquement. Mitigue par iframe sandbox.

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)
| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | `templates/static-reader/index.html` | CONFIRME | crates/sbfb-factory/src/templates/static-reader/index.html:1-154 | HTML reader, navigation prev/next, dark theme, keyboard arrows, bridge cursor save, `{{name}}` placeholder |
| 2 | `templates/static-reader/sbfb-bridge.js` | CONFIRME | crates/sbfb-factory/src/templates/static-reader/sbfb-bridge.js:1-58 | Starter bridge 5 methodes (submitTask, getStorage, setStorage, getIdentityPubkey, getNodeStatus) |
| 3 | `templates/static-reader/README.md` | CONFIRME | crates/sbfb-factory/src/templates/static-reader/README.md:1-34 | Instructions utilisateur, placeholders, commandes validate/publish |
| 4 | `templates/static-reader/.gitignore` | CONFIRME | crates/sbfb-factory/src/templates/static-reader/gitignore:1-5 | node_modules, dist, .DS_Store, Thumbs.db |
| 5 | `template_engine.rs` multi-template | CONFIRME | crates/sbfb-factory/src/template_engine.rs:64-126 | STATIC_READER_TEMPLATE, TemplateConfig, TEMPLATES, find_template() |
| 6 | SBFB.json programmatique | CONFIRME | crates/sbfb-factory/src/template_engine.rs:159-171 | schema_version 2, category "content", bridge 3 methodes, validate() ok |
| 7 | 3 tests plan §6.3 | CONFIRME | crates/sbfb-factory/src/template_engine.rs:415-467 | 3 tests couvrant create, validate, substitution |

Resume : 7 livrables / 7 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme
### Patterns
- P24 postMessage bridge : starter bridge respecte le protocole. Aligned.
- Aucun pattern PATTERNS.md viole.
- Tech debt : aucun T-NN touche.

### Horizon long-terme
- Design doc : N/A (enrichissement module existant, pas nouveau module structurant)
- D1..D5 avec alternatives : D2 cite 3 alternatives rejetees. OK
- Solution la plus poussee : refacto multi-template extensible. OK
- LOC estimees au plan : 0. OK

## Commit body validation
### Titre
- Format : `feat(factory): Sprint 69 Phase C — Babel Reader template + dogfood E2E` — match regex ok

### 9 sections body
Draft body non fourni. CONCERN draft-body-absent. Template `.claude/templates/commit_body_phase.txt`.

### Co-Authored-By
- A verifier au commit.

## Findings

- **P2-C-1** : Bridge SDK starter vs complet — `crates/sbfb-factory/src/templates/static-reader/sbfb-bridge.js:1-58` — Le plan §6.2 dit "copie SDK bridge depuis web/public/sbfb-bridge.js". Le code livre est un starter 58 lignes avec 5 methodes. Le SDK complet (web/public/sbfb-bridge.js) fait 422 lignes avec ~20 methodes (listStorage, deleteStorage, piiRedact, getBrowseList, search, getProofCard, onEvent, getStorageVersion, onStorageUpdate, getPublicFeedCursor, getProvenanceRecord, verifyRelease — toutes absentes). Le fail-fast #14 `sync-bridge-sdk` ne couvre pas ce starter (fichier different). Le starter suffit fonctionnellement pour le Babel Reader (5 methodes utilisees) mais cree une surface de desync. **Direction fix** : documenter explicitement "starter bridge" dans le README du template, ou copier le SDK complet comme Protocol Explorer et Ideas Hub.

- **P3-C-1** : innerHTML dans template HTML — `crates/sbfb-factory/src/templates/static-reader/index.html:95` — `div.innerHTML = "<h2>" + sections[i].title + "</h2>..."`. Contenu statique meme fichier, pas de XSS reelle. Mitigue par iframe sandbox + CSP. Nit.

## Codex reconciliation
- Status : N/A pre-Codex
- Rapport Codex : a produire
- GAPs P0/P1 : N/A
- P2/P3 documentes dans body : N/A

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep innerHTML/unsafe/unwrap/secrets sur 5 fichiers, scan sbfb-manifest allowlist | index.html, sbfb-bridge.js, template_engine.rs, sbfb-manifest/src/lib.rs | 1 P3 |
| Patterns | PATTERNS.md (1646+ lignes), shell/PATTERNS.md (1598+ lignes), P24 bridge protocol | docs/rust/PATTERNS.md, docs/shell/PATTERNS.md | 0 |
| Scope-cuts | 14 items kickoff §7, grep + lecture semantique diff complet | sprint69_kickoff.md §7, git diff HEAD | 0 |
| Branch coverage | 10 elements, 3 tests lus integralement (468 lignes fichier), 4 criteres | template_engine.rs complet | 0 |
| Research grounding | preflight 290 lignes lu, 5 scans verifies, allowlist cross-ref | preflight.md, sbfb-manifest/src/lib.rs | 0 |
| Livrables | 7/7 via Read avec line numbers | 5 fichiers template + template_engine.rs | 0 gaps |
| Horizon long-terme | D2 alternatives, LOC check, design doc check | kickoff.md §D2, plan.md §6 | 0 |
| Bridge SDK sync | diff web/public/sbfb-bridge.js (422 lignes) vs starter (58 lignes) | les 2 fichiers lus integralement | 1 P2 |

## Recommendation
- Ready to commit : NON (PASS-PENDING, Codex requis)
- Carry-overs S70 : P2-C-1 bridge SDK starter vs complet
- Corrections needed : 0 P0, 0 P1

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + compteurs 1433 Rust / 279 Vitest)
- [ ] Update MEMORY.md (si description pivot changee)
- [ ] Stage review.md + preflight.md dans commit phase ou chore suivant
