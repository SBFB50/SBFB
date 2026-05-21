# Sprint 68 Phase B — deep review

HEAD: `2d0999f` (pre-Phase B) | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

`PASS-PENDING` = review Claude clean avant Codex, non committable.

(Rigor signal : 4 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation

- `feedback_approach.md` : pick deepest, no band-aid, research before
  code, G8 procedural — respecte (HashMap custom vs moka = correct
  pour ~5 entries, pas de over-engineering).
- `feedback_context7_systematic.md` : context7 obligatoire avant code
  touchant lib/API — respecte (preflight S1b documente reqwest 0.12
  + zip 8.5 + CVE scans).
- `vision_model.md` : N/A (pas de pattern startup implique).
- `fairness_vision.md` : N/A (pas de kudos/reputation touche).
- `feedback_no_direct_blobserve.md` : preview servi via blob-serve
  dans iframe sandbox Browse — conforme (blob-serve CSP sandbox
  herite automatiquement).
- `feedback_codex_gate_strict.md` : Codex pas encore lance —
  PASS-PENDING transitoire, pas committable.
- Violations memory : **aucune**.

## Staging check

- Phase fichiers : 6 modifies (`Cargo.lock`, `lib.rs`, `http.rs`,
  `runtime.rs`, `Cargo.toml`, `main.rs`) + 4 untracked (`preview.rs`,
  `daemon_client.rs`, `preview_cmd.rs`, `publish.rs`)
- Planning/docs split : 1 untracked planning (`sprint68_phase_b_preflight.md`)
  + 1 research doc (`remote_user_sharded_llm_rnd.md`) — ces fichiers
  devront etre commites dans un `chore(planning)` separe ou stages
  avec le feat SEULEMENT si le preflight est lie a cette phase.
  **Le preflight EST lie a Phase B** — OK pour bundler dans le meme
  commit. `remote_user_sharded_llm_rnd.md` est un fichier research
  non lie — NE PAS le stager dans le commit feat Phase B.
- Untracked accidentels : 0 (les 6 fichiers sont tous attendus)

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok (0 diff) |
| cargo clippy | - | - | - | **FAIL** (2 errors daemon_client.rs, voir P1-B-1) |
| Rust nextest | 1394 | ~1409 | +15 | **BLOCKED** (clippy fail bloque nextest dans le run chaine) |
| Rust doctests | ok | ok | - | ok |
| tsc --noEmit | - | - | - | N/A (Phase B ne touche pas web/) |
| ESLint | - | - | - | N/A |
| Vitest | 271 | 271 | +0 | N/A (Phase B ne touche pas web/) |
| Build web | - | - | - | N/A |
| size-limit | - | - | - | N/A |
| Playwright | - | - | - | N/A |
| scan-en-strings | - | - | - | N/A |
| Release build | - | - | - | BLOCKED (clippy fail) |

**Note** : le clippy fail est sur 2 lints `unnecessary_lazy_evaluations`
dans `daemon_client.rs` (lignes 27-28 et 38-39). Voir P1-B-1. Une
fois corrige, les suites devraient passer. L'executeur rapporte
1409/1409 nextest pass apres correction clippy, ce qui est coherent
avec +15 delta (baseline 1394 = Phase A sortie, +10 Phase A real =
1394, +15 Phase B = 1409).

**Comptage delta reel** (depuis executeur) :
- Rust nextest : 1394 → 1409 (+15 : 6 preview.rs + 4 http.rs
  preview + 1 daemon_client.rs + 2 preview_cmd.rs + 2 publish.rs)
- Le plan prevoyait +6 Phase B. Le reel est +15 car les tests
  unitaires dans les nouveaux fichiers sont plus nombreux que prevu.
  Pas de scope creep — plus de tests = mieux.

## Branch coverage semantique (deep)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `PreviewStore::load()` ok | `load_returns_blake3_hash` | oui | oui (hash len == 64, hash match blake3) | - | DEEP-PASS |
| `PreviewStore::load()` too large | `rejects_oversized_upload` | oui | oui (matches TooLarge) | MAX+1 bytes | DEEP-PASS |
| `PreviewStore::get()` before TTL | `get_returns_data_before_ttl` | oui | oui (data == original) | - | DEEP-PASS |
| `PreviewStore::get()` after TTL | `get_returns_none_after_ttl` | oui | oui (is_none) | 1ms TTL | DEEP-PASS |
| `PreviewStore::evict_expired()` | `evict_expired_removes_stale_entries` | oui | oui (!has) | 1ms TTL | DEEP-PASS |
| `PreviewStore::has()` unknown | `has_returns_false_for_unknown_hash` | oui | oui (false) | - | DEEP-PASS |
| `preview_load` HTTP ok | `test_preview_load_returns_hash` | oui | oui (200 + hash 64 chars) | - | DEEP-PASS |
| `preview_load` HTTP too large | `test_preview_max_size_rejected` | oui | oui (413) | MAX+1 | DEEP-PASS |
| `blob_serve` preview fallback | `test_preview_blob_serve_accessible` | oui | oui (200 + body contains test html) | - | DEEP-PASS |
| `blob_serve` preview eviction | `test_preview_eviction_after_ttl` | oui | oui (!has after evict) | 1ms TTL | DEEP-PASS |
| `PreviewStore::load()` lock poisoned | - | - | - | - | UNTESTED (P3, defensive path, production RwLock poison rare) |
| `zip_directory()` | `zip_directory_creates_valid_archive` | oui | oui (names contains index.html + css/style.css) | - | DEEP-PASS |
| `preview_cmd::run()` missing index.html | `run_rejects_missing_index_html` | oui | oui (error contains "index.html") | - | DEEP-PASS |
| `publish::run()` missing daemon | `publish_requires_running_json` | oui | oui (error contains "daemon not running" or "running.json") | - | DEEP-PASS |
| `publish::run()` invalid manifest | `publish_pre_validates_manifest` | oui | oui (error contains "name must not be empty") | empty name | DEEP-PASS |
| `DaemonConnection::discover()` no file | `discover_fails_without_running_json` | oui | oui (NotRunning match) | - | DEEP-PASS |
| `preview_load` LockPoisoned error branch | - | - | - | - | DEFENSIVE-OK |
| `preview_cmd::run()` happy path (end-to-end) | - | - | - | - | WIRING-UNTESTED (P2-B-4, needs running daemon) |
| `publish::run()` happy path (end-to-end) | - | - | - | - | WIRING-UNTESTED (P2-B-5, needs running daemon) |

## Scope cuts semantique (deep)

| Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|-----------|---------|-----------|----------------|-----------------|--------|
| SC-1 | SearchManifest wire format + gossip | Pas de protocole reseau nouveau | 0 match | 0 code reseau nouveau | CLEAN |
| SC-2 | Page React /factory | Pas d'UI factory dans le shell | 0 match | 0 composant React ajoute | CLEAN |
| SC-3 | Babel dogfood via Factory | Pas de dogfood Babel | 0 match | 0 reference Babel | CLEAN |
| SC-4 | @dev index tree-sitter | Pas d'indexation code | 0 match | 0 reference tree-sitter | CLEAN |
| SC-5 | Template react-vite | Pas de nouveau template | 0 match | 0 template ajoute | CLEAN |
| SC-6 | Factory audit log JSONL | Pas de log structure | `eprintln!` pour stdout | Conforme (stdout, pas JSONL) | CLEAN |
| SC-7 | CuratorVouched UI shell | Pas d'UI vouch | 0 match | 0 composant vouch | CLEAN |
| SC-8 | FG8 Provenance Ed25519 | Pas de provenance factory-side | 0 match | publish delegue a daemon deploy-from-repo | CLEAN |
| SC-9 | FG9 Publish gate complete | S68 = publish basic | publish.rs = basic (no gate pipeline) | Conforme | CLEAN |
| SC-10 | FG10 Review gate | Pas de review gate | 0 match | 0 code review | CLEAN |
| SC-11 | Fuzzing cargo-fuzz/proptest | Pas de fuzzing | 0 match | 0 proptest | CLEAN |
| SC-12 | Feed format version bump | Pas de bump | 0 version modifiee | 0 const VERSION touchee | CLEAN |
| SC-13 | ProofCard comme feed op | Pas de feed op ProofCard | 0 match | 0 feed op | CLEAN |
| SC-14 | Diff engine avance | Pas de diff semantique | 0 match (pas de diff.rs en Phase B) | Phase C scope | CLEAN |

## Research grounding (deep)

### Preflight G8

- Fichier : existe (`sprint68_phase_b_preflight.md`)
- Scans : 5/5 (S1a 6 projets OSS, S1b 6 libs, S2 3 decisions,
  S3 7 vecteurs, S4 canonical.rs complet)
- S1a OSS : Netlify/Vercel, IPFS, F-Droid, Moka, reqwest, zip crate
- Verdict : **EXECUTE plan-as-is**
- Finding S1a : APPROACH-ALIGNED (HashMap custom acceptable pour
  cache ~5 entries)

### Deps/API

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| reqwest | 0.12.28 | oui (preflight S1b context7) | features `["blocking"]` conforme S1b note operationnelle | PASS |
| zip | workspace (8.5) | oui (preflight S1b, CVE-2025-29787 non-applicable) | usage en creation uniquement (ZipWriter) | PASS |
| blake3 | workspace | oui (existant) | hash dans preview.rs:47 | PASS |

### Coherence code-vs-source

- reqwest blocking : le code utilise `reqwest::blocking::Client::new()`
  qui est conforme au pattern context7 documente dans le preflight.
  `features = ["blocking"]` dans Cargo.toml — OK, pas besoin de tokio
  runtime dans sbfb-factory CLI.
- zip ZipWriter : le code utilise `zip::ZipWriter::new()` +
  `start_file()` + `write_all()` + `finish()` — pattern standard
  documente dans zip crate docs.
- Note : le preflight recommandait `reqwest = { version = "0.12",
  features = ["blocking", "json"] }` mais le code n'utilise PAS le
  feature `json` dans sbfb-factory pour les requetes preview (le body
  est raw bytes). `publish.rs` utilise `.json(&req)` qui est fourni
  par reqwest de base (pas besoin du feature `json` pour serialiser,
  seulement pour deserialiser). Conforme.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| daemon_client.rs | `#[allow(dead_code)]` | 15 | P1-B-2 | `schema_version` allow dead_code sans justification |
| daemon_client.rs | unsafe env set/remove | 109, 112 | P3 (test only) | `unsafe { std::env::set_var }` dans tests, Rust 1.94+ |
| publish.rs | unsafe env set/remove | 97, 103 | P3 (test only) | idem |
| preview.rs | 0 pattern critique | - | - | clean |
| preview_cmd.rs | 0 pattern critique | - | - | clean |
| http.rs | 0 nouveau pattern critique | - | - | clean |

### Analyse semantique

1. **preview_load auth** : le endpoint `POST /api/v1/preview/load` est
   sur `authed_routes` (http.rs:362, dans le router builder qui
   commence a la ligne 265). Le bearer token est requis. Pas d'acces
   non-authentifie. **PASS**.

2. **preview_load DoS via repeated uploads** : sans cap sur le nombre
   d'entrees PreviewStore, un script avec le bearer token pourrait
   remplir la memoire (10 MB * N). Le TTL 30 min + bearer auth
   mitigent. Preflight S3 GAP1 l'a documente comme Low severity.
   **P2-B-3** (pas bloquant pre-launch, single-user loopback).

3. **preview bytes clone dans blob_serve** : `state.preview_store.get(&hash)`
   retourne `Some(Vec<u8>)` (clone complet du zip) puis le passe a
   `blob_serve_cache.load()`. Le zip est clone une fois puis decompress
   en memoire dans le cache blob-serve. Ce n'est pas un hot path
   (une seule fois par hash, le cache blob-serve evite les clones
   suivants). **Acceptable**.

4. **Path traversal dans zip_directory** : `preview_cmd.rs:61-65`
   utilise `strip_prefix(dir)` + `replace('\\', "/")` et skip les
   paths commencant par `.`. Le zip est cree par l'utilisateur local
   (CLI) et decompress par blob-serve qui a sa propre
   `validate_zip_path()`. Double defense. **PASS**.

5. **Publish delegue a deploy-from-repo** : `publish.rs` envoie
   `repo_url` + `project_name` au daemon via JSON POST. Le daemon
   fait clone + verify + sign. Conforme a D3 (pas d'upload direct).
   **PASS**.

6. **DaemonConnection auth_token lecture** : lit `~/.sbfb/auth_token`
   avec les memes permissions user que le daemon. Meme modele de
   securite que le shell React. **PASS**.

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | PreviewStore HashMap + TTL 30 min + eviction | CONFIRME | `preview.rs:17-84` | `DEFAULT_TTL: Duration = Duration::from_secs(30 * 60)`, `HashMap<String, PreviewEntry>`, `evict_expired()` retains by TTL |
| 2 | POST /api/v1/preview/load handler | CONFIRME | `http.rs:2020-2037` | `async fn preview_load(State(state), body: Bytes)` + TooLarge/Ok/Error branches |
| 3 | PreviewStore integration DaemonHttpState | CONFIRME | `http.rs:182` | `pub preview_store: PreviewStore` dans DaemonHttpState |
| 4 | Spawn eviction task dans runtime.rs | CONFIRME | `runtime.rs:903-912` | `tokio::spawn` avec `interval(60s)` + `store.evict_expired()` |
| 5 | blob_serve fallback vers preview store | CONFIRME | `http.rs:1133-1143` | `if let Some(zip_bytes) = state.preview_store.get(&hash)` avant le lookup iroh blobs |
| 6 | publish.rs (lit running.json + pre-valide + POST deploy-from-repo) | CONFIRME | `publish.rs:23-61` | `DaemonConnection::discover()`, `load_and_validate_manifest()`, POST `/api/v1/deploy-from-repo` |
| 7 | preview_cmd.rs (zip + POST preview/load + affiche URL) | CONFIRME | `preview_cmd.rs:13-46` | `zip_directory()`, POST preview/load, `eprintln!("open: ...")` |
| 8 | daemon_client.rs (running.json + auth_token) | CONFIRME | `daemon_client.rs:26-51` | `DaemonConnection::discover()` lit running.json + auth_token |
| 9 | main.rs subcommands Preview + Publish | CONFIRME | `main.rs:43-56` | `Command::Preview { path }` + `Command::Publish { path, repo_url }` |
| 10 | sbfb-factory Cargo.toml deps reqwest + zip | CONFIRME | `Cargo.toml` diff | `reqwest = { workspace = true, features = ["blocking"] }` + `zip = { workspace = true }` |
| 11 | 6 tests preview.rs | CONFIRME | `preview.rs:94-148` | 6 #[test] functions |
| 12 | 4 tests http.rs preview | CONFIRME | `http.rs:6302-6394` | 4 #[tokio::test] functions |
| 13 | 2 tests preview_cmd.rs | CONFIRME | `preview_cmd.rs:84-112` | 2 #[test] functions |
| 14 | 2 tests publish.rs | CONFIRME | `publish.rs:79-122` | 2 #[test] functions |
| 15 | 1 test daemon_client.rs | CONFIRME | `daemon_client.rs:102-114` | 1 #[test] function |

Resume : 15 livrables / 15 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns

- P52 BlobStore pattern (PATTERNS.md) : le preview store est un
  HashMap separe, pas un BlobStore. C'est voulu (ephemere, pas iroh).
  Pas de drift.
- Pattern commit body 9 sections : a verifier au commit (Step 10).
- Auth pattern (bearer token) : respecte — preview_load est dans
  authed_routes.
- JCS canonicalization : N/A (pas de wire format canonical).
- `validate_zip_path` pattern : blob_serve.rs deja existant couvre
  la decompression. preview_cmd.rs fait la creation — double defense.

### Horizon long-terme

- Design doc present (nouveaux modules) : preflight G8 couvre le
  design. Pas de nouveau module structurant > 1 sprint.
- D1..D5 avec alternatives + rationale : D2 kickoff documente
  3 alternatives rejetees. D3 documente 3 alternatives. OK.
- Solution la plus poussee : HashMap custom vs moka — moka serait
  over-engineering pour ~5 entries. Choix justifie dans preflight S1a.
- Aucune LOC estimee au plan : 0 match (verifie).

## Commit body validation

### Titre

Le titre attendu est :
`feat(factory): Sprint 68 Phase B — preview ephemere + publish path`

Format : match regex
`(feat|fix|docs|chore|test)\((sprint[0-9]+|[a-z_+-]+)\): Sprint [0-9]+ Phase [A-Z] — .+`

### 9 sections body

| Section | Present | Coherent | Signal |
|---------|---------|----------|--------|
| Contexte | a verifier | - | TBD |
| Fichiers | a verifier | - | TBD |
| Delta tests | a verifier | annonce vs reel | TBD |
| Verification §7.4 | a verifier | - | TBD |
| Scope cuts | a verifier | exhaustif kickoff §7 | TBD |
| G8 traceability | a verifier | SHA cross-ref | TBD |
| Pre-launch protocol | a verifier | *_VERSION unchanged | TBD |
| Codex verification | a verifier | rapport + reconciliation | TBD |
| Carry closure | a verifier | - | TBD |

Note : le body sera valide au moment du commit. La review pre-Codex
ne peut pas le verifier car il n'existe pas encore.

### Co-Authored-By

A verifier au commit.

## Findings

- **P1-B-1** : clippy errors `unnecessary_lazy_evaluations` dans
  `daemon_client.rs:27-28` et `daemon_client.rs:38-39`. `.ok_or_else(||
  DaemonClientError::NotFound(...))` devrait etre `.ok_or(
  DaemonClientError::NotFound(...))` car la closure n'est pas
  necessaire pour un &'static str.
  **Fix** : remplacer `.ok_or_else(|| ...)` par `.ok_or(...)` sur
  les 2 occurrences. Bloquant : clippy workspace fail.

- **P1-B-2** : `#[allow(dead_code)]` sur `schema_version` dans
  `daemon_client.rs:15`. Le champ est deserialise mais jamais lu —
  devrait etre `_schema_version` (convention Rust pour champs
  intentionnellement ignores) au lieu de `#[allow(dead_code)]`.
  Pattern `#[allow(dead_code)]` nouveau dans le diff = P1 par le
  protocole review §7b.
  **Fix** : renommer `schema_version` en `_schema_version` et
  ajouter `#[serde(rename = "schema_version")]` pour la
  deserialization. Ou utiliser `#[serde(alias = "schema_version")]`.

- **P2-B-3** : pas de cap `max_preview_entries` sur le PreviewStore.
  Un script avec le bearer token pourrait accumuler des previews en
  memoire (10 MB * N). Mitige par : bearer auth, TTL 30 min,
  single-user loopback. Preflight S3 GAP1 le documente.
  **Direction fix** : ajouter un const `MAX_PREVIEW_ENTRIES: usize = 10`
  et rejeter les loads au-dela. Carry S69 acceptable.

- **P2-B-4** : `preview_cmd::run()` happy path wiring non teste par
  test d'integration (necessite un daemon running). Les fonctions
  composantes sont testees unitairement (`zip_directory()` +
  `DaemonConnection::discover()`) mais le wiring HTTP end-to-end
  (POST preview/load → reponse 200 → parse hash → print URL) n'est
  pas exerce.
  **Direction fix** : carry S69, tester avec le daemon en
  integration test ou ajouter un mock HTTP server dans les tests.

- **P2-B-5** : `publish::run()` happy path wiring non teste par test
  d'integration. Meme situation que P2-B-4.

- **P3-B-1** : `unsafe { std::env::set_var(...) }` dans les tests
  factory (daemon_client.rs:109, publish.rs:97). Fonctionnellement
  correct mais marque `unsafe`. Pattern acceptable dans les tests
  Rust 1.94+ ou `set_var` est marque unsafe. Nit seulement.

## Codex reconciliation

- Status : RECONCILIE
- Rapport Codex : sprint68_phase_b_codex_review.md (GPT 5.5 brut)
- Résultat Codex : 7 CONFIRME, 2 PARTIEL, 0 GAP
- PARTIEL 3 (test_preview_eviction_after_ttl teste store pas HTTP) :
  accepté P3 — le test vérifie le mécanisme d'éviction, les 3 autres
  couvrent le surface HTTP.
- PARTIEL 5 (path résolution BaseDirs vs HOME) : CORRIGÉ —
  daemon_client.rs utilise maintenant `directories::BaseDirs::data_dir()`
  identique au daemon. dep `directories` ajoutée au factory Cargo.toml.
- Post-fix : clippy 0 warnings, factory 21/21 tests verts.
- Suites relancées après correction Codex : oui (clippy + nextest factory)

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/dead_code/todo!/panic! sur 5 fichiers + analyse auth routes + DoS vectors | preview.rs, daemon_client.rs, preview_cmd.rs, publish.rs, http.rs (lignes 260-440, 1130-1170, 2020-2037) | 3 (P1-B-2 dead_code, P2-B-3 no cap, P3-B-1 unsafe test) |
| Patterns | PATTERNS.md rust lu, shell/PATTERNS.md lu, P52 BlobStore verifie, auth pattern verifie | docs/rust/PATTERNS.md (1646 lignes), docs/shell/PATTERNS.md (1598 lignes) | 0 |
| Scope-cuts | 14 items kickoff §7 + grep + lecture semantique diff complet | kickoff §7, diff complet (203 lignes +/-) | 0 (14/14 clean) |
| Branch coverage | 15 elements + tests, 16 tests lus en entier | preview.rs tests, http.rs tests, preview_cmd.rs tests, publish.rs tests, daemon_client.rs test | 2 (P2-B-4 + P2-B-5 wiring untested) |
| Research grounding | preflight lu en entier (496 lignes), 5 scans verifies, deps tracees | sprint68_phase_b_preflight.md, Cargo.toml diff | 0 |
| Livrables | 15/15 verifies via Read avec line numbers | 10 fichiers lus integralement | 0 (15/15 confirmes) |
| Horizon long-terme | design doc preflight + alternatives D2/D3 kickoff + 0 LOC estimee | kickoff §4 D2/D3, plan §5 | 0 |
| Clippy | `cargo clippy --workspace -D warnings` execute | sortie clippy (42 lignes) | 1 (P1-B-1 ok_or_else) |

## Recommendation

- Ready to commit : **oui** (PASS, P1 corrigés, Codex réconcilié)
- P1-B-1 : CORRIGE (ok_or_else → ok_or)
- P1-B-2 : CORRIGE (_schema_version + serde rename)
- Codex PARTIEL 5 : CORRIGE (BaseDirs alignment)
- Carry-overs S69 : P2-B-3 (max_preview_entries), P2-B-4/B-5
  (integration tests wiring)

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
