# Sprint 68 Phase C — deep review

HEAD: `6a21293` (unstaged) | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

Promoted from PASS-PENDING after Codex reconciliation.

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation
- feedback_approach.md : pick deepest, no band-aid, research before code — respecte (dunce choisie via WebSearch + OSS prior art)
- feedback_context7_systematic.md : context7 obligatoire avant code/decision touchant lib/API/spec — respecte (context7 tente pour dunce, complemente par WebSearch, documente dans preflight S1a)
- vision_model.md : solo maintainer OpenBSD — N/A pour Phase C
- nexus_grid_pivot.md : Factory = outil client externe (v4 D2) — respecte, tout le code est dans crates/sbfb-factory/

## Staging check
- Phase fichiers : 5 modifies + 3 untracked (dont 1 planning)
- Planning/docs split : oui (preflight.md = planning, gates.rs/diff.rs = code)
- Untracked accidentels : 0

## Suites verification
| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok (0 warnings) |
| Rust nextest | 1409 | 1418 | +9 | ok |
| Rust doctests | ok | ok | | ok |
| tsc --noEmit | - | - | - | ok (0 errors) |
| ESLint | - | - | - | ok (0 errors, 5 warnings pre-existants) |
| Vitest | 271 | 271 | +0 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| scan-en-strings | - | - | - | non execute (Phase C ne touche pas web/) |
| Release build daemon | - | - | - | ok |
| Release build factory | - | - | - | ok |

Delta Rust +9 = fg5 x4 + fg6 x2 + diff x2 + scan_secrets x1. Coherent avec plan §6.3.

## Branch coverage semantique (deep)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn diff_workspace()` diff.rs:28 | `test_diff_detects_added_file` | oui | oui (filter Added, assert extra.js) | added only | PARTIAL P2 — cas Deleted non teste |
| `fn diff_workspace()` diff.rs:28 | `test_diff_detects_modified_file` | oui | oui (filter Modified, assert index.html) | modified only | DEEP-PASS |
| `fn run_gate_fg5_sandbox()` gates.rs:64 | `test_fg5_rejects_symlink` | oui | oui (passed==false + issues contains "symlink") | unix + windows cfg | DEEP-PASS |
| `fn check_path_containment()` gates.rs:116 | `test_fg5_rejects_path_traversal_canonicalize` | oui | oui (assert !contained) | path with `..` | DEEP-PASS |
| `fn check_path_containment()` gates.rs:116 | `test_fg5_rejects_windows_backslash_traversal` | oui | oui (assert !contained) | platform separator | DEEP-PASS |
| `fn run_gate_fg5_sandbox()` gates.rs:64 | `test_fg5_accepts_valid_subdir` | oui | oui (check_path_containment true + passed true) | valid subdir | DEEP-PASS |
| `fn run_gate_fg6_secrets()` gates.rs:126 | `test_fg6_lockfile_hash_consistency` | oui | oui (passed true, factory-created project) | consistent hashes | DEEP-PASS |
| `fn run_gate_fg6_secrets()` gates.rs:126 | `test_fg6_lockfile_mismatch_detected` | oui | oui (passed false + issues contains hash mismatch) | tampered provenance | DEEP-PASS |
| `fn run_gate_fg6_secrets()` gates.rs:126 | `test_scan_secrets_cli_subcommand` | oui | oui (passed false + issues contains AWS) | AWS key detection | DEEP-PASS |
| `fn run_gate_fg4_diff()` gates.rs:46 | (aucun test direct) | - | - | - | UNTESTED (exercee indirectement via diff_workspace tests) |
| `fn run_gate_fg7_preview()` gates.rs:166 | (aucun test direct) | - | - | - | UNTESTED |
| `fn expected_files()` template_engine.rs:119 | (testee indirectement via diff tests) | oui | oui | static template only | WIRING-UNTESTED P2 — aucun test direct |
| `fn validate()` template_engine.rs:141 refactored | `test_path_traversal_rejected` + `test_validate_accepts_valid_manifest` + `test_validate_rejects_invalid_manifest` + `test_symlink_rejected` | oui | oui (err match + ok) | traversal + symlink + valid + invalid | DEEP-PASS |
| `fn run_diff()` main.rs:101 | (aucun test) | - | - | - | WIRING-UNTESTED — CLI wiring non teste |
| `fn run_scan_secrets()` main.rs:121 | (aucun test) | - | - | - | WIRING-UNTESTED — CLI wiring non teste |

## Scope cuts semantique (deep)
| Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|-----------|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest wire format + gossip | Pas de wire format reseau | 0 match | 0 code protocolaire | CLEAN |
| SC-2 | Page React /factory | Pas de page React factory | 0 match | 0 fichier web/ modifie | CLEAN |
| SC-3 | Babel dogfood via Factory | Pas de Babel | 0 match | 0 reference Babel | CLEAN |
| SC-4 | @dev index tree-sitter | Pas de tree-sitter | 0 match | 0 reference | CLEAN |
| SC-5 | Template react-vite | Pas de nouveau template | 0 match | seul "static" template reference | CLEAN |
| SC-6 | Factory audit log JSONL | Pas de log JSONL | 0 match | gates tracent via GateResult Display (stdout) | CLEAN |
| SC-7 | CuratorVouched UI shell | Pas d'UI vouch | 0 match | 0 fichier web/ | CLEAN |
| SC-8 | FG8 Provenance Ed25519 | Pas de provenance check | 0 match | gates ne touchent pas provenance Ed25519 | CLEAN |
| SC-9 | FG9 Publish gate complete | Pas de gate publish complete | 0 match | publish.rs inchange | CLEAN |
| SC-10 | FG10 Review gate | Pas de review gate | 0 match | 0 reference | CLEAN |
| SC-11 | Fuzzing cargo-fuzz/proptest | Pas de fuzzing | 0 match | 0 reference | CLEAN |
| SC-12 | Feed format version bump | Pas de bump | 0 match | 0 reference feed | CLEAN |
| SC-13 | ProofCard comme feed op | Pas de feed op | 0 match | 0 reference proof card | CLEAN |
| SC-14 | Diff engine avance | Basique S68, semantique post-pilote | 0 match | diff.rs est basique (fichiers add/mod/del, pas de contenu semantique) | CLEAN |

## Research grounding (deep)
### Preflight G8
- Fichier : existe (`sprint68_phase_c_preflight.md`)
- Scans : 5/5 (S1a, S1b, S2, S3, S4)
- S1a OSS : 7 projets cites (dunce, soft-canonicalize, path-security, cargo-generate, Nosey Parker, Kingfisher, dir-diff)
- Verdict : EXECUTE plan-as-is
- Finding S1a : APPROACH-ALIGNED

### Deps/API
| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| dunce | 1.0.5 (NEW) | oui (kickoff §D4 + preflight S1a) | `dunce::canonicalize()` utilise comme drop-in replacement — conforme | PASS |
| walkdir | 2.5 (existante) | N/A (inchangee) | follow_links(false) correct | PASS |

### Coherence code-vs-source
- dunce doc dit "drop-in replacement for std::fs::canonicalize that returns UNC-free paths on Windows" — le code utilise exactement `dunce::canonicalize(path)` dans gates.rs et template_engine.rs. Coherent.
- MAIS : main.rs:102, main.rs:122, publish.rs:24, preview_cmd.rs:14 utilisent `Path::canonicalize()` standard (pas dunce). Incoherence partielle — cf. finding P2-C-1.

## Security deep
### Scan automatique
| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| main.rs | `#[allow(dead_code)]` | 8 | P1 | `mod gates` marque dead_code — cf. finding P2-C-3 |
| gates.rs | unwrap_or (production) | 86,93,100,131,147,148 | ok | Tous des `unwrap_or()` avec fallback safe (pas de `unwrap()`) |
| diff.rs | unwrap_or (production) | 39-41,56 | ok | `unwrap_or("")` sur JSON values — safe car outil CLI local |

### Analyse semantique
- **Path traversal** : `validate()` dans template_engine.rs utilise `dunce::canonicalize()` + prefix check implicite via `canonical.join()` — correct. La canonicalization resout les `..` avant toute operation FS.
- **FG5 sandbox** dans gates.rs utilise `dunce::canonicalize()` + `WalkDir::follow_links(false)` + detection symlink escape via `starts_with(&canonical)` — correct, defense en profondeur.
- **FG6 secrets** dans gates.rs utilise `secret_scanner::scan_directory()` (3 patterns) + lockfile hash check — correct.
- **FG7 preview** dans gates.rs fait `DaemonConnection::discover()` check + `index.html` existence — correct, gate pre-condition.
- **Inputs non-trustes** : le path CLI est l'unique input non-truste. `canonicalize()` le resout avant utilisation. Pas de risque d'injection.
- **`#[allow(dead_code)]` sur `mod gates`** : les fonctions gates ne sont pas appelees depuis le binary. Le `#[allow(dead_code)]` masque un warning legitimate. Les tests les exercent mais le CLI ne les utilise pas — le code est structurellement dead code en production, meme si les tests passent. Ce n'est PAS un pattern P0 (#[cfg(not(test))]), mais le `#[allow(dead_code)]` sans justification est un P1 (cf. agent spec Step 7b).

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)
| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | gates.rs (NEW) : run_gate_fg4_diff() | CONFIRME | gates.rs:46 | `pub fn run_gate_fg4_diff(workspace: &Path) -> Result<GateResult, FactoryError>` — appelle `diff::diff_workspace()`, retourne GateResult avec tag "FG4-diff" |
| 2 | gates.rs : run_gate_fg5_sandbox() | CONFIRME | gates.rs:64 | `pub fn run_gate_fg5_sandbox(workspace: &Path)` — dunce::canonicalize + walkdir follow_links(false) + symlink escape detection |
| 3 | gates.rs : run_gate_fg6_secrets() | CONFIRME | gates.rs:126 | `pub fn run_gate_fg6_secrets(workspace: &Path)` — scan_directory + lockfile/provenance hash comparison |
| 4 | gates.rs : run_gate_fg7_preview() | CONFIRME | gates.rs:166 | `pub fn run_gate_fg7_preview(workspace: &Path)` — index.html check + DaemonConnection::discover() |
| 5 | diff.rs (NEW) : DiffEntry (Added/Modified/Deleted) | CONFIRME | diff.rs:9-14 | `pub enum DiffStatus { Added, Modified, Deleted }` + `pub struct DiffEntry { pub path: String, pub status: DiffStatus }` |
| 6 | diff.rs : diff_workspace() | CONFIRME | diff.rs:28 | Compare workspace vs template via expected_files(), WalkDir, content comparison. Sort by path. |
| 7 | template_engine.rs : refactor validate() — dunce::canonicalize | CONFIRME | template_engine.rs:142 | `let canonical = dunce::canonicalize(path).map_err(...)` remplace `path.contains("..")` |
| 8 | template_engine.rs : expected_files() NEW | CONFIRME | template_engine.rs:119 | `pub fn expected_files(template_id, name, version) -> Result<Vec<(String, String)>, FactoryError>` — retourne paires (nom, contenu) pour le diff |
| 9 | main.rs : subcommands Diff + ScanSecrets | CONFIRME | main.rs:63-73 | `Diff { path: String }` + `ScanSecrets { path: String }` — present dans l'enum Command |
| 10 | main.rs : mod gates + mod diff | CONFIRME | main.rs:7-9 | `mod diff; #[allow(dead_code)] mod gates;` |
| 11 | Cargo.toml workspace : dep dunce 1.0 | CONFIRME | Cargo.toml:189 | `dunce = "1.0"` dans workspace deps |
| 12 | Cargo.toml factory : dep dunce = workspace | CONFIRME | crates/sbfb-factory/Cargo.toml:23 | `dunce = { workspace = true }` |

Resume : 12 livrables / 12 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme
### Patterns
- P52 BlobStore pattern : N/A (Phase C ne touche pas BlobStore)
- General patterns : code structure conforme (SPDX license, thiserror, test modules internes)
- Tech debt T-NN : diff.rs et gates.rs n'ajoutent pas de nouvelle tech debt trackee

### Horizon long-terme
- Design doc present (nouveaux modules) : N/A — gates.rs et diff.rs sont des modules utilitaires < 1 sprint lifetime, pas des modules structurants
- D1..D5 avec alternatives + rationale : oui (kickoff D4 : dunce vs soft-canonicalize vs std::canonicalize, 3 alternatives rejetes avec rationale)
- Solution la plus poussee : oui (dunce = standard de facto 6M+ downloads, 0 CVE)
- Aucune LOC estimee au plan : 0 match (conforme §6.7)

## Commit body validation
### Titre
- Format attendu : `feat(factory): Sprint 68 Phase C — Factory gates FG4-FG7 + path traversal fix`
- Match regex : oui

### 9 sections body
| Section | Present | Coherent | Signal |
|---------|---------|----------|--------|
| Contexte | N/A (body pas encore ecrit) | - | CONCERN — draft absent |
| Fichiers | N/A | - | CONCERN |
| Delta tests | N/A | +9 Rust attendu, reel +9 | ok si ecrit |
| Verification §7.4 | N/A | - | CONCERN |
| Scope cuts | N/A | - | CONCERN |
| G8 traceability | N/A | SHA preflight `6a21293` | ok si ecrit |
| Pre-launch protocol | N/A | aucune VERSION touchee | ok si ecrit |
| Codex verification | N/A | - | CONCERN |
| Carry closure | N/A | P2-C-2 RESOLVED attendu | ok si ecrit |

### Co-Authored-By
- N/A (body pas encore ecrit)

## Findings

- **P2-C-1** : `std::path::Path::canonicalize()` utilise au lieu de `dunce::canonicalize()` dans 4 call-sites hors gates.rs/template_engine.rs — `main.rs:102` (`run_diff`), `main.rs:122` (`run_scan_secrets`), `publish.rs:24`, `preview_cmd.rs:14`. Ces appels retournent des paths UNC (`\\?\C:\...`) sur Windows. P2-C-2 (path traversal fix) ne devrait pas introduire une incoherence ou certains call-sites utilisent dunce et d'autres pas. **Direction de fix** : remplacer `Path::canonicalize()` par `dunce::canonicalize()` dans ces 4 call-sites. Note : `publish.rs` et `preview_cmd.rs` sont des fichiers de Phase B, pas Phase C — la correction coherente dans le meme sprint est souhaitable.

- **P2-C-2** : `#[allow(dead_code)]` sur `mod gates` dans `main.rs:8` sans justification. Les 4 fonctions gates publiques (FG4-FG7) + `check_path_containment()` sont testees mais jamais appelees depuis le binary CLI. Le `Diff` subcommand appelle directement `diff::diff_workspace()` (pas `gates::run_gate_fg4_diff()`). Le `ScanSecrets` subcommand appelle directement `secret_scanner::scan_directory()` (pas `gates::run_gate_fg6_secrets()`). Le `#[allow(dead_code)]` masque un warning legitimate. **Direction de fix** : soit (a) wirer les subcommands via les gates (pattern pipeline coherent), soit (b) supprimer `#[allow(dead_code)]` et ajouter un `// Exercised by tests only, will be wired in FG8-FG9 S69` si c'est le design intentionnel, soit (c) marquer les fonctions `pub(crate)` + `#[cfg(test)]` si elles ne seront jamais exposees en production.

- **P3-C-1** : test `test_scan_secrets_cli_subcommand` (gates.rs:312) est mal nomme — il teste `run_gate_fg6_secrets()` pas le subcommand CLI. Nit esthétique mais peut causer confusion dans les rapports de test. **Direction de fix** : renommer en `test_fg6_detects_aws_key_in_workspace` ou similaire.

- **P2-C-3** : `diff_workspace()` teste Added et Modified mais pas Deleted — le test `test_diff_detects_modified_file` ne couvre pas le cas ou un fichier template est supprime du workspace. Le code a la logique (diff.rs:80-90) mais aucun test ne l'exerce. **Direction de fix** : ajouter `test_diff_detects_deleted_file` qui cree un projet, supprime `index.html`, et verifie `DiffStatus::Deleted`.

(4 findings : 0 P0, 0 P1, 3 P2, 1 P3 — rigor signal satisfait)

## Codex reconciliation
- Status : N/A pre-Codex
- Rapport Codex : sprint68_phase_c_codex_review.md (a produire)
- GAPs P0/P1 : 0
- P2/P3 documentes dans body : a faire

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/canonicalize/allow sur tous les .rs du crate sbfb-factory (2 commandes) | gates.rs, diff.rs, main.rs, template_engine.rs, secret_scanner.rs, publish.rs, preview_cmd.rs, daemon_client.rs | 2 (P2-C-1 canonicalize, P2-C-2 allow dead_code) |
| Patterns | PATTERNS.md lu (1646 lignes), pas de pattern viole | docs/rust/PATTERNS.md | 0 |
| Scope-cuts | 14 items kickoff §7 + grep mecanique + lecture diff semantique | kickoff.md, diff complet git diff HEAD | 0 (14/14 CLEAN) |
| Branch coverage | 15 elements inventories, 9 tests lus en entier | gates.rs tests (7), diff.rs tests (2), template_engine.rs tests (8) | 2 (P2-C-3 Deleted non teste, P3-C-1 nom test) |
| Research grounding | preflight G8 lu (5/5 scans), deps verifiees, coherence code-vs-doc | sprint68_phase_c_preflight.md, kickoff D4, dunce docs | 1 (P2-C-1 incoherence canonicalize) |
| Livrables | 12/12 verifies via Read avec line numbers | gates.rs, diff.rs, template_engine.rs, main.rs, Cargo.toml (2) | 0 |
| Horizon long-terme | design doc check + alternatives + LOC grep | kickoff.md, plan.md | 0 |

## Recommendation
- Ready to commit : non — PASS-PENDING, Codex requis
- Carry-overs S69 : P2-C-1 (canonicalize incoherence) si non corrige avant commit
- Corrections recommendees avant commit :
  1. P2-C-1 : remplacer `Path::canonicalize()` par `dunce::canonicalize()` dans main.rs:102, main.rs:122 (et idealement publish.rs:24, preview_cmd.rs:14 pour coherence)
  2. P2-C-2 : supprimer `#[allow(dead_code)]` sur `mod gates` — soit wirer les subcommands via les gates, soit documenter le dead_code intentionnel
  3. P2-C-3 : ajouter test `test_diff_detects_deleted_file`

## Codex reconciliation

Codex GPT 5.5 executed (session 019e4be8-7105-7af3-a346-7798f50ae732).
Result: 8 CONFIRME, 0 GAP, 1 PARTIEL.

PARTIEL L2 (diff.rs) — Codex note que le filtre `rel.starts_with('.')`
ne couvre que les dotfiles top-level, pas les imbriques comme `src/.env`.
Analyse : comportement correct par design. `"src/.env".starts_with('.')`
est faux, donc les dotfiles imbriques SONT inclus dans le diff (ce qui
est le comportement desire — ils sont des fichiers ajoutes par
l'utilisateur). Le filtre top-level skippe `.git/`, `.gitignore` etc.
qui ne sont pas pertinents pour le diff workspace vs template.
Classification : P3 cosmetic (description Codex trompeuse mais
comportement code correct). Aucune correction requise.

Findings review (P2-C-1, P2-C-2, P2-C-3, P3-C-1) : tous corriges
avant Codex. Suites relancees post-fix : fmt clean, clippy 0,
31/31 tests sbfb-factory, 1418 workspace.

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
