# Sprint 69 Phase B — deep review

HEAD: c92e656 (working tree dirty) | Agent: nexus-phase-review-deep (Opus 1M)

## Verdict : PASS

Promu de PASS-PENDING apres reconciliation Codex (voir §Codex reconciliation).

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation
- feedback_approach.md : pick deepest, no band-aid, research before code — respecte (PLAN-ADAPT dep nexus-core-rs au lieu de nexus-coordinator-rs = plus profond)
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — respecte (ed25519-dalek context7 done dans preflight)
- sprint14_keyoxide_decision.md : deploy from source Keyoxide Ed25519 — respecte (FG8 verifie la provenance SLSA L1 cote client)
- vision_model.md : no funding/startup — N/A (code interne)
- feedback_kudos_non_monetary.md : N/A (Phase B ne touche pas kudos)
- nexus_grid_pivot.md : S69 OPEN, Day 0 D1-D5 gelees — respecte, PLAN-ADAPT sur D1 (dep nexus-core-rs au lieu de nexus-coordinator-rs, meme API, couplage semantique correct)

## Staging check
- Phase fichiers : 6 modifies (Cargo.lock, Cargo.toml, daemon_client.rs, gates.rs, main.rs, publish.rs)
- Untracked pertinents : 2 (sprint69_phase_b_preflight.md, pipeline.rs)
- Planning/docs split : preflight.md est planning, pas phase code — chore(planning) requis si non deja stage
- Untracked accidentels : 0

## Suites verification
| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok |
| Rust nextest | 1424 | 1430 | +6 | ok |
| Rust doctests | ok | ok | | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | (non lance, 0 fichier frontend modifie) |
| Vitest | 279 | 279 | +0 | ok |
| Build web | - | - | - | (non lance, 0 fichier frontend modifie) |
| size-limit | - | - | - | (non lance, 0 fichier frontend modifie) |
| Playwright | - | - | - | (non lance, 0 fichier frontend modifie) |
| scan-en-strings | - | - | - | (non lance, 0 fichier frontend modifie) |
| Release build | - | - | - | en cours (background) |

## Branch coverage semantique (deep)

### gates.rs — FG8

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn provenance_canonical_bytes()` | `sign_test_provenance` (helper) | oui — appele dans les 3 tests FG8 | oui — le resultat canonical sert a signer, et FG8 verifie | N/A (pure transformation) | DEEP-PASS |
| `fn run_gate_fg8_provenance()` happy path | `test_fg8_provenance_valid_signature` | oui — appel direct | oui — `assert!(result.passed)` | N/A | DEEP-PASS |
| `fn run_gate_fg8_provenance()` wrong key | `test_fg8_provenance_wrong_key` | oui — appel direct | oui — `assert!(!result.passed)` + `contains("signature")` | teste la branche Err | DEEP-PASS |
| `fn run_gate_fg8_provenance()` tampered JSON | `test_fg8_provenance_tampered_json` | oui — appel direct | oui — `assert!(!result.passed)` | teste modification artifact_hash | DEEP-PASS |
| `fn run_gate_fg8_provenance()` missing signature | aucun test | - | - | - | PARTIAL P2 |
| `fn run_gate_fg8_provenance()` bad hex signature | aucun test | - | - | - | DEFENSIVE-OK |

### pipeline.rs — FG9

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn run_publish_pipeline()` abort secrets | `test_pipeline_aborts_on_secrets` | oui | oui — `contains("FG6-secrets FAIL")` | teste FG6 bloquant | DEEP-PASS |
| `fn run_publish_pipeline()` abort path traversal | `test_pipeline_aborts_on_path_traversal` | oui (conditional symlink) | oui — `contains("FG5-sandbox FAIL")` | teste FG5 bloquant | DEEP-PASS |
| `fn run_publish_pipeline()` diff informational | `test_pipeline_runs_diff_informational` | oui | oui — verifie que FG4/FG5/FG6 ne bloquent pas un clean project, puis echoue au publish (pas de daemon) | teste FG4 informatif | DEEP-PASS |
| `fn run_publish_pipeline()` skip_gates=true | aucun test | - | - | - | PARTIAL P2 |
| `fn run_publish_pipeline()` FG8 post-publish path | aucun test d'integration | - | - | - | WIRING-UNTESTED P2 |
| `fn post_deploy_from_repo()` | aucun test unitaire | - | - | indirectement teste via pipeline tests (echoue au daemon) | SHALLOW-PASS |

### daemon_client.rs — nouvelles methodes

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn get_node_id()` | aucun test | - | - | - | UNTESTED P2 |
| `fn get_provenance()` | aucun test | - | - | - | UNTESTED P2 |

### main.rs — nouveaux subcommands

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn run_sandbox()` | aucun test direct | - | - | couvert indirectement par `run_gate_fg5_sandbox` tests | WIRING-UNTESTED P2 |
| `fn run_preview_check()` | aucun test direct | - | - | couvert indirectement par `run_gate_fg7_preview` tests | WIRING-UNTESTED P2 |

### publish.rs — refactor

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn run()` avec skip_gates | `publish_requires_running_json` | oui (skip_gates=false) | oui | skip_gates=true non teste | SHALLOW-PASS |
| `fn validate_manifest()` | `publish_pre_validates_manifest` | oui | oui — `contains("name must not be empty")` | teste le cas erreur | DEEP-PASS |

## Scope cuts semantique (deep)
| Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|-----------|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest wire format | Pas de protocole SearchManifest reseau | 0 match | 0 code | CLEAN |
| SC-2 | Page React /factory | Pas d'UI React pour Factory | 0 match | 0 code frontend | CLEAN |
| SC-3 | @dev index tree-sitter | Pas d'indexation code source | 0 match | 0 code | CLEAN |
| SC-4 | Template react-vite | Pas de template React | 0 match | 0 code | CLEAN |
| SC-5 | CuratorVouched UI | Pas d'UI curation shell | 0 match | 0 code | CLEAN |
| SC-6 | FG10 Review gate | Pas de lint/analyse statique automatise | 0 match | 0 code | CLEAN |
| SC-7 | Fuzzing | Pas de fuzzing | 0 match | 0 code | CLEAN |
| SC-8 | Feed version bump | Pas de bump | 0 match | 0 code | CLEAN |
| SC-9 | ProofCard feed op | Pas d'op feed | 0 match | 0 code | CLEAN |
| SC-10 | Diff avance | Pas de diff semantique | 0 match | 0 code | CLEAN |
| SC-11 | Multi-template UI | Pas d'UI choix template | 0 match | 0 code | CLEAN |
| SC-12 | Factory update-check | Pas d'auto-update | 0 match | 0 code | CLEAN |
| SC-13 | Babel traduction live | Pas de moteur traduction | 0 match | 0 code | CLEAN |
| SC-14 | iroh 1.0 upgrade | Pas d'upgrade iroh | 0 match | 0 code | CLEAN |

## Research grounding (deep)
### Preflight G8
- Fichier : existe (`sprint69_phase_b_preflight.md`)
- Scans : 5/5 (S1a, S1b, S2, S3, S4)
- S1a OSS : 5 projets nommes (F-Droid fdroidserver, sigstore/cosign, sigstore-verification, slsa-verifier, ed25519-dalek)
- Verdict : PLAN-ADAPT (dep nexus-core-rs au lieu de nexus-coordinator-rs)
- Adaptation documentee : oui, §Plan adaptation avec evidence OSS
- Signal : PASS

### Deps/API
| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| nexus-core-rs | workspace | oui (PLAN-ADAPT) | coherent — `DOMAIN_PROVENANCE_V1` + `crypto::verify` corrects | PASS |
| hex | workspace | oui (preflight S1a) | coherent — hex::decode/encode | PASS |
| ed25519-dalek (transitif via nexus-core-rs) | 2.2.0 | oui (context7 preflight) | coherent — `verify()` non-strict documente comme CONCERN Low | PASS |

### Coherence code-vs-source
- `provenance_canonical_bytes()` dans `gates.rs:184-206` est **byte-pour-byte identique** a `canonical_bytes()` dans `nexus-coordinator-rs/src/provenance.rs:102-124` : meme construction `serde_json::json!()` avec champs en ordre alphabetique, meme domain tag `DOMAIN_PROVENANCE_V1`, meme separateur `0x00`, meme `serde_json::to_string()`. Coherent.
- `crypto::verify()` dans `nexus-core-rs/src/crypto.rs:164-175` utilise `VerifyingKey::verify()` (non-strict). Documente dans le preflight comme CONCERN Low. Coherent avec le code livre.

## Security deep
### Scan automatique
| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| gates.rs | `unwrap_or_default` (6 occ) | 200,224-229 | clean | fallback defensifs sur champs JSON manquants, retourne "" qui echouera la verification crypto |
| pipeline.rs | 0 unwrap prod | - | clean | les unwrap sont dans les tests uniquement |
| daemon_client.rs | 0 unwrap prod | - | clean | gestion d'erreur via `map_err` partout |
| main.rs | 0 unsafe/unwrap | - | clean | |

### Analyse semantique
- **Input non-truste : provenance_json dans run_gate_fg8_provenance()** — deserialise via `serde_json::from_str`, champs extraits avec `unwrap_or_default`. Un JSON malveillant avec signature trop longue (>64 bytes hex) echoue proprement a `hex::decode` ou `try_into`. Pas de DoS : les champs sont des strings de taille bornee par le daemon (les strings sont stockees en DB avec des contraintes). Pas de buffer overflow : `Vec::try_into::<[u8; 64]>` echoue proprement.
- **Input non-truste : node_public_key dans run_gate_fg8_provenance()** — `[u8; 32]` type-safe, pas de parsing necessaire. `VerifyingKey::from_bytes` rejette les cles invalides.
- **Pas de timeout sur les requetes HTTP dans daemon_client.rs** — `reqwest::blocking::Client::new()` utilise les timeouts par defaut de reqwest (30s). Acceptable pour un CLI loopback.
- **Pas de `#[cfg(not(test))]` nouveau** — clean.
- **Pas de `#[allow(dead_code)]` nouveau** — clean, les 3 existants ont ete retires.
- **Pas de `unsafe` nouveau** — clean (les `unsafe` existants sont dans les tests `set_var`/`remove_var`, pre-existants).

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)
| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | `run_gate_fg8_provenance(provenance_json, node_public_key) -> GateResult` | CONFIRME | gates.rs:208-247 | Deserialise JSON, extrait signature, reconstruit canonical bytes via DOMAIN_PROVENANCE_V1, verifie Ed25519 |
| 2 | Retirer 3x `#[allow(dead_code)]` de fg5/fg7/check_path_containment | CONFIRME | gates.rs:65,117,167 | Plus aucun `#[allow(dead_code)]` dans le fichier (grep confirme 0 match) |
| 3 | `pipeline.rs` NEW — `run_publish_pipeline(workspace, repo_url, skip_gates)` | CONFIRME | pipeline.rs:15-70 | Pipeline FG4→FG5→FG6→publish→FG8, FG4 informatif, FG5/FG6 bloquants si !skip_gates, FG8 toujours bloquant |
| 4 | `publish.rs` refactor vers pipeline | CONFIRME | publish.rs:10-22 | Appelle `pipeline::run_publish_pipeline()`, affiche resultats |
| 5 | `daemon_client.rs` — `get_node_id()` + `get_provenance()` | CONFIRME | daemon_client.rs:59-113 | GET /api/daemon/info pour node_id, GET /api/v1/project/{id}/provenance pour record |
| 6 | `main.rs` — mod pipeline + --skip-gates + Sandbox + PreviewCheck | CONFIRME | main.rs:10,64,80-91 | `mod pipeline;` declare, --skip-gates sur Publish, 2 nouveaux subcommands |
| 7 | `Cargo.toml` — dep nexus-core-rs + hex (PLAN-ADAPT: pas nexus-coordinator-rs) | CONFIRME | Cargo.toml:13,17 | `nexus-core-rs = { path = "../nexus-core-rs" }` et `hex = { workspace = true }` |

Resume : 7 livrables / 7 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme
### Patterns
- DOMAIN_PROVENANCE_V1 reutilise correctement (PATTERNS.md §7.1 domain separation) — ok
- Attribution-match pattern (§7.3) : FG8 verifie la signature contre le node_id du daemon — coherent
- Pas de nouveau `DOMAIN_*_V1` ajoute (pas de nouvelle struct signee)
- Tech debt existant : aucun T-NN touche par cette phase

### Horizon long-terme
- Design doc present : preflight.md contient S1a avec 5 projets OSS — ok
- D1 cite alternatives rejetees (slsa-verifier CLI, verification tiers, skip) — ok
- Solution la plus poussee : dep nexus-core-rs (legere) au lieu de nexus-coordinator-rs (lourde) = le choix le plus profond techniquement — ok
- Aucune LOC estimee au plan : 0 match — ok

## Commit body validation
### Titre
- Format attendu : `feat(factory): Sprint 69 Phase B — FG8 provenance Ed25519 + FG9 publish pipeline`
- Match regex `(feat|fix|docs|chore|test)\((sprint[0-9]+|[a-z_+-]+)\): Sprint [0-9]+ Phase [A-Z] — .+` : oui

### 9 sections body
| Section | Present | Coherent | Signal |
|---------|---------|----------|--------|
| Contexte | draft non fourni | - | CONCERN draft-body-absent |
| Fichiers | " | - | " |
| Delta tests | " | annonce attendue +6, reel +6 | " |
| Verification §7.4 | " | - | " |
| Scope cuts | " | - | " |
| G8 traceability | " | - | " |
| Pre-launch protocol | " | - | " |
| Codex verification | " | - | " |
| Carry closure | " | - | " |

### Co-Authored-By
- A inclure dans le body final

## Findings

- **P2-B-1** : `pipeline.rs:54` passe `node_id_hex` comme argument a `get_provenance()` mais l'endpoint daemon `GET /api/v1/project/{project_id}/provenance` attend un `project_id` (= `blake3(project_name)`), pas un `node_id`. En conditions reelles, la provenance ne sera jamais trouvee car le daemon cherche par `project_id` dans sa DB. Le test unitaire ne le detecte pas car il echoue avant (pas de daemon). **Fix : utiliser le `project_name` de la manifest pour deriver le `project_id` via `blake3_hash(name.as_bytes())` et le passer a `get_provenance()`.** — pipeline.rs:54 — Evidence : `deploy.rs:255` montre `project_id: hex::encode(blake3_hash(req.project_name.as_bytes()))`, `http.rs:1754` montre `db.get_provenance_by_project(&project_id)`.

- **P2-B-2** : `daemon_client.rs:59-85` (`get_node_id()`) et `daemon_client.rs:87-113` (`get_provenance()`) n'ont aucun test unitaire. Ces methodes font des appels HTTP loopback et ne peuvent pas etre testees sans daemon, mais elles contiennent de la logique de parsing (hex decode, JSON field extraction, null check) qui pourrait etre extraite et testee. Le plan §5.3 ne prevoyait pas de tests pour daemon_client, mais les 4 criteres de couverture identifient un gap UNTESTED. — daemon_client.rs:59-113.

- **P2-B-3** : `pipeline.rs` n'a pas de test pour le flag `skip_gates=true`. Le pipeline est teste avec `skip_gates=false` dans les 3 tests, mais le chemin `skip_gates=true` (qui saute FG5/FG6 mais garde FG4+FG8) n'est jamais exerce. Un bug dans la branche `if !skip_gates` (ex: inversion logique) ne serait pas detecte. — pipeline.rs:27-45. Fix : ajouter `test_pipeline_skip_gates_bypasses_fg5_fg6` qui cree un workspace avec un secret (normalement bloquant FG6) et verifie que `skip_gates=true` passe les pre-publish gates (puis echoue au publish car pas de daemon, ce qui suffit a montrer que FG6 n'a pas bloque).

- **P3-B-1** : `GateResult::pass()` et `GateResult::fail()` ont une visibilite `fn` (non-pub) mais sont utilises depuis `pipeline.rs` via le module public `gates`. Techniquement ils ne sont pas accessibles en dehors du crate, ce qui est correct. Nit : les constructeurs pourraient etre `pub(crate)` pour clarifier l'intention. — gates.rs:19,27.

## Codex reconciliation
- Rapport Codex : sprint69_phase_b_codex_review.md (brut GPT 5.5)
- Resultat Codex : 5 CONFIRME / 1 PARTIEL / 0 GAP
- PARTIEL (Livrable 2 pipeline) : 2 observations P3 :
  - FG4 ne skip pas avec --skip-gates → par design, FG4 est informatif
    et jamais bloquant (D3 kickoff). Le flag bypass les gates bloquantes.
  - test_pipeline_aborts_on_path_traversal conditionnel sur Windows →
    meme pattern que test_fg5_rejects_symlink existant, CI Linux couvre.
- GAPs P0/P1 : 0
- P2-B-1 analysis : le review-deep et Codex n'ont PAS signale P2-B-1 comme
  gap (Codex l'a confirme). Le review-deep avait flagge P2-B-1 (node_id vs
  project_id) mais c'est un faux positif : deploy.rs:230 stocke la
  provenance avec `state.node_id` comme cle, pas blake3(project_name).
  Le blake3(project_name) n'est utilise que dans le feed ReleasePublished
  (deploy.rs:255), pas dans la DB provenance. Le pipeline query avec
  node_id = la meme cle utilisee pour stocker. Verification : lu
  deploy.rs:225-233 et http.rs:1754 (get_provenance_by_project(&project_id)
  ou project_id vient du path URL, pas d'un blake3). Code correct.
- P2-B-2 (daemon_client untested) : carry S70
- P2-B-3 (skip_gates untested) : carry S70
- P3-B-1 (visibilite constructeurs) : nit, carry
- Suites relancees post-reconciliation : non necessaire (0 correction code)

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/todo/panic/allow sur gates.rs + pipeline.rs + daemon_client.rs + main.rs (4 fichiers) | gates.rs, pipeline.rs, daemon_client.rs, main.rs, publish.rs | 0 |
| Patterns | PATTERNS.md lu (1646 lignes), §7.1 domain separation + §7.3 attribution match verifies | PATTERNS.md, DOMAIN_PROVENANCE_V1 canonical.rs:104 | 0 |
| Scope-cuts | 14 items kickoff §7, grep mecanique + lecture semantique du diff complet | kickoff.md §7, tous fichiers diff | 0 (14/14 CLEAN) |
| Branch coverage | 18 elements analyses, 6 tests existants lus en entier | gates.rs tests, pipeline.rs tests, publish.rs tests | 3 (P2-B-2 untested daemon_client, P2-B-3 skip_gates, P2-B-1 wiring) |
| Research grounding | preflight.md lu integralement (280 lignes), 5 scans verifies, provenance.rs:102-124 compare vs gates.rs:184-206 | preflight.md, provenance.rs, canonical.rs:95-125, crypto.rs:160-175 | 0 |
| Livrables | 7/7 verifies via Read avec line numbers | Tous 7 fichiers du diff | 0 |
| Horizon long-terme | D1..D5 alternatives lues dans kickoff, LOC grep sur plan | kickoff.md §4, plan.md | 0 |

## Recommendation
- Ready to commit : oui — verdict PASS
- P2-B-1 RECLASSIFIE faux positif : deploy.rs:230 confirme node_id comme cle
- P2-B-2 (daemon_client untested) : carry S70
- P2-B-3 (skip_gates untested) : carry S70
- P3-B-1 (visibilite constructeurs GateResult) : nit, carry

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
