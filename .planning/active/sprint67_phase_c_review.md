# Sprint 67 Phase C — deep review

HEAD: `688b442` (dirty — sbfb-factory untracked + Cargo.toml/Cargo.lock modified)
Agent: nexus-phase-review-deep (Opus 1M)

## Verdict : PASS

Rigor signal : 3 findings P2+/P3 documentes (>= 1 requis pour PASS).
P2-C-1 (PEM test manquant) RESOLU post-review par ajout test.
1 P2 actif (P2-C-2 path traversal), 2 P3.

## Memory consultation

- feedback_approach.md : pick deepest, no band-aid — **respecte** (include_str! + String::replace est le bon choix per SYNTHESIS §3.3, pas Tera/Handlebars overkill)
- feedback_context7_systematic.md : context7 obligatoire avant code touchant lib/API — **respecte** (preflight documente queries clap 4.5, blake3 1.8.3, walkdir symlink API)
- vision_model.md : NO funding / NO fondation / NO startup — **N/A** (Phase C ne touche pas le modele economique)
- nexus_grid_pivot.md : Factory hors daemon (D2 v4) — **respecte** (sbfb-factory ne dep pas de nexus-shell-daemon-core ni nexus-coordinator-rs, confirme par grep)
- fairness_vision.md : N/A (Phase C ne touche pas kudos/reputation)
- feedback_kudos_non_monetary.md : N/A

## Staging check

- Phase fichiers : 2 tracked modifies (Cargo.toml, Cargo.lock) + 1 dir untracked (crates/sbfb-factory/) + 1 fichier untracked (.planning/active/sprint67_phase_c_preflight.md)
- Planning/docs split : preflight.md est un artefact planning mele aux fichiers phase — le chore planning (preflight) devrait etre committe separement ou inclus dans le commit phase (acceptable si le hook le permet, puisque le preflight est un artefact de la phase elle-meme)
- Untracked accidentels : 0

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | 0 diff | - | ok |
| cargo clippy | - | 0 warnings | - | ok |
| Rust nextest | 1368 | 1379 | +11 | ok |
| Rust doctests | ok | ok | - | ok |
| tsc --noEmit | - | 0 errors | - | ok |
| ESLint | - | 0 errors | - | ok |
| Vitest | 270 | 270 | +0 | ok |
| Build web | - | ok | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| Playwright | - | - | - | N/A (non lance — Phase C ne touche pas le frontend) |
| scan-en-strings | - | - | - | N/A |
| Release build daemon | - | ok | - | ok |

Notes :
- Rust nextest 1379/1379 PASS (0 skipped) — revalide apres ajout test PEM
- Vitest 270/270 PASS — inchange, Phase C ne touche pas le frontend
- Delta Rust +11 Phase C : 8 tests template_engine + 3 tests secret_scanner (AWS, PEM, GitHub). Plan estimait +10 — +1 ajout post-review (P2-C-1 fix).

## Branch coverage semantique (deep)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `fn create()` | `test_create_generates_sbfb_json_v2` | oui — appelle `create("static", "test-app", ...)` | oui — verifie schema_version == 2, name == "test-app", validate().is_ok() | template="static" only | DEEP-PASS |
| `fn create()` | `test_create_generates_index_html` | oui | oui — verifie existence + `<title>` present | - | DEEP-PASS |
| `fn create()` | `test_create_generates_template_lock` | oui | oui — verifie template_id == "static", hash len == 64 | - | DEEP-PASS |
| `fn create()` substitution | `test_create_substitutes_name` | oui | oui — verifie `contains("my-cool-app")` ET `!contains("{{name}}")` | name avec tirets | DEEP-PASS |
| `fn validate()` happy path | `test_validate_accepts_valid_manifest` | oui — genere puis valide | oui — `is_ok()` | - | SHALLOW-PASS (assert global) |
| `fn validate()` rejects | `test_validate_rejects_invalid_manifest` | oui — SBFB.json v2 sans name | oui — `is_err()` | schema_version sans name | DEEP-PASS |
| `fn validate()` path traversal | `test_path_traversal_rejected` | oui — `validate("some/path/../../../etc")` | oui — `matches!(_, Err(FactoryError::PathTraversal(_)))` | - | DEEP-PASS |
| `fn validate()` symlink | `test_symlink_rejected` | oui (conditionnel OS) | oui — `is_err()` | cfg(unix) et cfg(windows) separes | DEEP-PASS |
| `scan_directory()` AWS | `test_secret_scanner_detects_aws_key` | oui — ecrit AKIA pattern dans fichier | oui — verifie count == 1 ET pattern_name == "AWS access key" | - | DEEP-PASS |
| `scan_directory()` PEM | `test_secret_scanner_detects_pem_private_key` | oui — ecrit PEM header dans fichier | oui — verifie count == 1 ET pattern_name == "PEM private key" | RSA prefix | DEEP-PASS |
| `scan_directory()` GitHub | `test_secret_scanner_detects_github_token` | oui — ecrit ghp_ pattern | oui — verifie count == 1 ET pattern_name == "GitHub token" | - | DEEP-PASS |
| `TemplateLock::generate()` | via `test_create_generates_template_lock` | oui (indirect) | hash len == 64, template_id correct | - | DEEP-PASS |
| `substitute()` fn | via `test_create_substitutes_name` | oui (indirect) | `{{name}}` remplace, `{{version}}` non verifie directement | - | SHALLOW-PASS |
| `if template != "static"` branch | - | - | - | - | UNTESTED P3 |
| `FactoryError` variants | couvertes par tests individuels | - | - | - | DEFENSIVE-OK |

Couverture globale : 11 tests Phase C, 10 DEEP-PASS, 1 SHALLOW-PASS, 0 UNTESTED P2, 1 UNTESTED P3 (error path bad template name).

## Scope cuts semantique (deep)

| # | Scope cut | Intention | Grep mecanique | Diff semantique | Signal |
|---|-----------|-----------|----------------|-----------------|--------|
| 1 | Preview ephemere POST /api/v1/preview/load | Pas de preview servie par daemon | 0 match (le "preview" dans README template est usage normal du mot) | 0 code route/endpoint | CLEAN |
| 2 | Diff engine avance sbfb-factory | Pas de commande diff | 0 match | 0 sous-commande diff dans clap | CLEAN |
| 3 | Page React /factory | Pas d'UI Factory dans le shell | 0 match web/ | 0 composant React ajoute | CLEAN |
| 4 | Proof Cards computation | Pas de calcul score | 0 match | 0 code score/card | CLEAN |
| 5 | SearchManifest wire format | Pas de wire format | 0 match | 0 struct serde wire | CLEAN |
| 6 | Babel app generation | Pas de Babel | 0 match | 0 code traduction | CLEAN |
| 7 | @dev index tree-sitter | Pas d'index AST | 0 match | 0 dep tree-sitter | CLEAN |
| 8 | Bridge method proof_card_get | Pas de bridge proof | 0 match | 0 code bridge | CLEAN |
| 9 | Template react-vite | Pas de template React | 0 match "react-vite" | seul template = "static", match arm `if template != "static"` = rejet | CLEAN |
| 10 | Factory audit log JSONL | Pas d'audit log | 0 match | 0 fichier log cree | CLEAN |
| 11 | CuratorVouched UI shell | Pas d'UI curator vouch | 0 match web/ | 0 composant React | CLEAN |
| 12 | Publish path sbfb-factory → daemon | Pas de publish | 0 match HTTP daemon dans sbfb-factory | 0 dep daemon, 0 reqwest/hyper | CLEAN |
| 13 | Feed format version bump | Pas de bump | 0 match FEED_FORMAT | 0 code feed | CLEAN |
| 14 | Fuzzing cargo-fuzz/proptest | Pas de fuzzing | 0 match | 0 dev-dep proptest | CLEAN |

Verdict scope cuts : 14/14 CLEAN.

## Research grounding (deep)

### Preflight G8

- Fichier : `.planning/active/sprint67_phase_c_preflight.md` — **existe**
- Scans : **5/5** (S1a, S1b, S2, S3, S4)
- S1a OSS : 5 projets (cargo-generate, Copier, Backstage Scaffolder, Gitleaks, Kingfisher) — **adequate**
- Verdict : **EXECUTE plan-as-is**
- PLAN-ADAPT : N/A
- DESIGN-CONFLICT : N/A

### Deps/API

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| sbfb-manifest | workspace (Phase A) | oui (D2 kickoff) | struct SbfbManifest correctement utilisee | PASS |
| clap | 4.5 derive | oui (WebSearch kickoff) | derive API conforme (Parser, Subcommand) | PASS |
| blake3 | 1.5+ workspace | oui (WebSearch kickoff) | Hasher::new() + update + finalize + to_hex — API correcte | PASS |
| walkdir | 2.x NEW | oui (preflight S1b) | WalkDir::new().follow_links(false), path_is_symlink() — API correcte | PASS |
| regex | workspace existant | implicite (deja utilise) | Regex::new() + is_match — API standard | PASS |
| serde + serde_json | workspace | existant | to_string_pretty, Serialize derive — standard | PASS |
| time | workspace existant | implicite | OffsetDateTime::now_utc().format(Rfc3339) — API correcte | PASS |
| thiserror | workspace | existant | #[error], #[from] — standard | PASS |
| tempfile | dev-dep workspace | existant | TempDir::new() — standard | PASS |

Notes :
- Plan listait `zip` et `ed25519-dalek` comme deps Phase C — non inclus dans le code. C'est correct : ces deps sont pour Phase D (provenance). Le plan §S6.2 etait trop large dans la liste.
- `regex` est une dep ajoutee sans trace explicite dans le preflight. C'est un crate workspace existant — CONCERN mineur.

### Coherence code-vs-source

- `blake3::Hasher::new()` → API verifiee, conforme a blake3 1.8.x
- `walkdir::WalkDir::new(dir).follow_links(false)` → API verifiee, conforme a walkdir 2.x
- `clap::Parser` derive + `#[command(subcommand)]` → API verifiee, conforme a clap 4.5
- `time::OffsetDateTime::now_utc()` + `Rfc3339` → API verifiee, conforme a time 0.3

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| template_engine.rs | `unwrap()` | 182-295 | N/A | Tests #[cfg(test)] uniquement — acceptable |
| template_lock.rs | `unwrap_or_else` | 25 | clean | Fallback "unknown" pour format Rfc3339 — defensive |

0 `unsafe`, 0 `#[allow(dead_code)]`, 0 `#[cfg(not(test))]`, 0 secret pattern hors tests, 0 `todo!`, 0 `panic!`, 0 `unimplemented!`.

### Analyse semantique

1. **Path traversal validation** (template_engine.rs:112) : `path.contains("..")` est un check string-level. C'est suffisant pour un outil CLI local (l'input vient du CLI, pas d'un reseau). Un attaquant controlant les args CLI a deja un acces shell. Le check `WalkDir::follow_links(false)` + `path_is_symlink()` dans le scan de validation (l.143-148) apporte une deuxieme couche sur les fichiers internes. **Adequat pour le MVP.**

2. **Secret scanner** (secret_scanner.rs:34-71) : silencieuse en cas d'erreur I/O (`Err(_) => continue` l.43, l.53). Pour un outil local qui scanne des fichiers du developpeur, c'est acceptable — un fichier binaire ou sans permission sera simplement skippe. Pas de DoS possible (outil local, pas daemon). **Adequat.**

3. **Regex compilation** (secret_scanner.rs:36) : les patterns sont compiles a chaque appel `scan_directory()`. Pour le MVP (quelques fichiers), c'est negligeable. Performance concern pour gros workspaces (1000+ fichiers) mais scope cut Factory audit log S68+ adressera ce point. **P3 nit.**

4. **Template substitution injection** : `{{name}}` est injecte dans index.html sans sanitization HTML. Le name vient du CLI arg local. L'app finale tourne dans iframe sandbox="allow-scripts" sans allow-same-origin. Blast radius contenu. **Adequat pour le MVP.**

5. **TemplateLock hash determinisme** : le hash trie les fichiers par nom puis concatene name+content. L'ordre est deterministe. Pas de timestamp dans le hash input (le `generated_at` est dans le JSON output, pas dans le hash). **Correct.**

## Livrable verification (Claude pre-Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | sbfb-factory crate structure | CONFIRME | crates/sbfb-factory/Cargo.toml:1-23 | Package sbfb-factory avec deps workspace, pas de dep daemon |
| 2 | CLI clap create + validate | CONFIRME | crates/sbfb-factory/src/main.rs:10-54 | `#[derive(Parser)]`, 2 subcommands Create + Validate, args --template --name --output |
| 3 | Template engine copie + substitution | CONFIRME | crates/sbfb-factory/src/template_engine.rs:40-108 | STATIC_TEMPLATE array avec include_str!, substitute() fn, create() genere fichiers + SBFB.json v2 + lock |
| 4 | Template static (4 fichiers) | CONFIRME | crates/sbfb-factory/src/templates/static/ | index.html (24 LOC, {{name}}), sbfb-bridge.js (57 LOC, starter SDK), README.md (21 LOC), gitignore (5 LOC) |
| 5 | factory.template.lock BLAKE3 | CONFIRME | crates/sbfb-factory/src/template_lock.rs:1-53 | TemplateLock struct avec template_hash (BLAKE3 sorted files), generated_at (Rfc3339), variables JSON |
| 6 | Secret scanner (AWS, GitHub, PEM) | CONFIRME | crates/sbfb-factory/src/secret_scanner.rs:19-32 | 3 patterns : AKIA[0-9A-Z]{16}, (ghp\|gho\|ghs\|ghr)_[A-Za-z0-9_]{36,}, PEM PRIVATE KEY header |
| 7 | Path traversal + symlink rejection | CONFIRME | crates/sbfb-factory/src/template_engine.rs:112-148 | `path.contains("..")` check + WalkDir follow_links(false) + path_is_symlink() |
| 8 | validate() SBFB.json parsing | CONFIRME | crates/sbfb-factory/src/template_engine.rs:128-140 | SbfbManifest::parse() + validate() avec error collection |
| 9 | 11 tests unitaires | CONFIRME | template_engine.rs:175-296 + secret_scanner.rs:73-119 | 8 tests template_engine + 3 tests secret_scanner = 11 total |
| 10 | Pas de dep daemon | CONFIRME | grep 0 match "nexus-shell-daemon\|nexus-coordinator" dans crates/sbfb-factory/ | Isolation confirmee |

Resume : 10 livrables / 10 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns

- P51 raw-op extensible : N/A (Phase C ne touche pas le feed)
- P52 BlobStore enum : N/A (Phase C ne touche pas BlobStore — P52 est Phase D)
- Nouveau pattern potentiel : template engine include_str! + substitute est un pattern simple et adapte. Pas de drift par rapport aux patterns existants.
- Le crate sbfb-factory suit le meme modele que les autres crates workspace (version.workspace, edition.workspace, etc.) — conforme.

### Horizon long-terme

- Design doc present : oui — SYNTHESIS_factory_rrv_protocol.md §3 + kickoff D5 + preflight S1a (5 projets OSS compares) — **adequat**
- D1..D5 avec alternatives + rationale : oui — D5 cite Copier, Tera/Handlebars, Factory-dans-daemon comme rejetes avec rationale — **conforme**
- Solution la plus poussee : include_str! + String::replace est le bon choix pour le MVP (< 200 LOC). Tera/Handlebars serait overengineering. Copier (Python) serait une dep lourde. cargo-generate (Liquid) est un outil different (generation from remote). Le choix est techniquement justifie.
- Aucune LOC estimee au plan : `grep -En 'LOC estim|~\s*[0-9]+\s*LOC|estim.*LOC' .planning/active/sprint67_plan.md` — 0 match. **Conforme.**

## Commit body validation

### Titre

Le commit n'est pas encore fait. Le titre cible (plan §S6.5) est :
`feat(factory): Sprint 67 Phase C — sbfb-factory CLI + template engine + create + validate`
Format regex : match `feat\((factory)\): Sprint 67 Phase C — .+` — **conforme**.

### 9 sections body

Draft body non fourni — **CONCERN** draft-body-absent. L'executeur devra generer le body avec les 9 sections obligatoires avant commit. Template de reference : `.claude/templates/commit_body_phase.txt`.

Rappel compteurs pour le body :
- Rust nextest : 1368 → 1379 (+11 Phase C, dont +1 post-review fix PEM)
- Vitest : 270 → 270 (+0)
- size-limit : 6/6

### Co-Authored-By

A inclure : `Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>`

## Findings

- **P2-C-1** : ~~`secret_scanner` PEM pattern non teste~~ — **RESOLU** post-review. Test `test_secret_scanner_detects_pem_private_key` ajoute a `secret_scanner.rs:93-104`. Verifie : 11/11 tests PASS (cargo nextest run -p sbfb-factory --locked). Le pattern PEM est maintenant couvert avec input `-----BEGIN RSA PRIVATE KEY-----`, assertion count == 1 et pattern_name == "PEM private key".

- **P2-C-2** : `validate()` ne verifie pas que le path de traversal `..` est un composant de chemin vs partie d'un nom de fichier — `path.contains("..")` matcherait un nom de fichier contenant ".." (edge case rare). Le check est suffisant pour un CLI local pre-launch mais devrait utiliser `Path::components()` pour une validation canonique en S68+ quand validate sera appele par le publish path. **Fichier** : `crates/sbfb-factory/src/template_engine.rs:112`.

- **P3-C-1** : `create()` genere un SBFB.json v2 sans section `bridge` (bridge: None) alors que le template index.html appelle `bridge.getNodeStatus()`. L'incoherence est cosmetique pour le MVP (le daemon ne filtre pas encore sur la declaration bridge du manifest), mais deviendra visible quand les Proof Cards S68 utiliseront la section bridge pour la validation. **Fichier** : `crates/sbfb-factory/src/template_engine.rs:98`.

- **P3-C-2** : `walkdir` dep ajoutee au workspace sans commentaire d'introduction (contrairement aux autres deps comme zip l.186-187, notify l.190-193 qui ont des commentaires Sprint N). Nit de convention. **Fichier** : `Cargo.toml:188`.

## Codex reconciliation

- Status : N/A pre-Codex
- Rapport Codex : sprint67_phase_c_codex_review.md (a produire)
- GAPs P0/P1 : 0
- P2/P3 documentes dans body : N/A pre-body

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/allow/cfg(not(test))/todo/panic/secrets sur crates/sbfb-factory/src/ (4 greps) | main.rs, template_engine.rs, template_lock.rs, secret_scanner.rs (4 fichiers, ~470 LOC) | 0 P0/P1. Unwraps uniquement dans #[cfg(test)] |
| Patterns | PATTERNS.md lu (200 lignes), shell/PATTERNS.md lu (200 lignes) | 2 fichiers | 0 drift |
| Scope-cuts | 14 items kickoff §7 + grep mecanique + lecture semantique diff complet | template_engine.rs, secret_scanner.rs, template_lock.rs, main.rs, 4 templates, Cargo.toml | 0 leak — 14/14 CLEAN |
| Branch coverage | 11 tests, 15 elements codes audites | template_engine.rs tests (120 LOC), secret_scanner.rs tests (45 LOC) | 0 UNTESTED P2 (PEM resolu), 1 UNTESTED P3, 1 SHALLOW-PASS |
| Research grounding | preflight lu (398 lignes), 5 projets OSS documentes, 9 deps verifiees | sprint67_phase_c_preflight.md, Cargo.toml workspace, sbfb-factory/Cargo.toml | 0 dep non tracee (regex implicite — CONCERN mineur) |
| Livrables | 10/10 verifies via Read | 10 fichiers du crate + Cargo.toml workspace | 10/10 confirmes |
| Horizon long-terme | SYNTHESIS §3, kickoff D5, preflight S1a | sprint67_kickoff.md, sprint67_plan.md, sprint67_phase_c_preflight.md | 0 drift — design doc present, alternatives citees |

## Recommendation

- Ready to commit : **oui** — verdict PASS (Codex reconcilié)
- Corrections P0/P1 avant commit : **aucune** (0 P0, 0 P1)
- P2-C-1 (PEM test) : **RESOLU** post-review
- P2-C-2 (path components) : carry-over S68 acceptable (CLI local, pas de surface reseau)
- Carry-overs S68 : P2-C-2 (path components validation canonique pour publish path), P3-C-1 (bridge section dans SBFB.json genere)

## Codex reconciliation

Codex GPT 5.5 exécuté (`codex exec`, session `019e472e-3903-7011-95b1-8bd9a3d57b16`).
Résultat brut : `.planning/active/sprint67_phase_c_codex_review.md`.

- 10 livrables vérifiés : 9 CONFIRMÉS, 0 GAP, 1 PARTIEL
- PARTIEL L9 (`test_symlink_rejected` Windows) : limitation plateforme connue —
  Windows requiert SeCreateSymbolicLinkPrivilege pour créer des symlinks.
  Le bloc `#[cfg(windows)]` skip gracieusement si `symlink_file()` échoue.
  Sur Unix/CI, le test s'exécute complètement avec assertion. P3 — pas de
  correction nécessaire, documenté dans commit body.
- 0 GAP P0/P1 trouvé par Codex. Cohérent avec review Claude.
- Suites relancées après fix P2-C-1 : 1379/1379 Rust, 270 Vitest.

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
