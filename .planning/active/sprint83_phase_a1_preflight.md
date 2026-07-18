# Sprint 83 — Phase A1 — Preflight G8

## Verdict: PLAN-ADAPT

Verdict global = plus sévère des verdicts corrigés par carry (DESIGN-CONFLICT >
PLAN-ADAPT > SCOPE-CUT-CONSISTENT > EXECUTE). 8 carries sur 9 sont **EXECUTE** ;
le carry **7 (flake nextest)** est **PLAN-ADAPT** : la cible du fix proposé est
factuellement fausse (voir §Plan carry 7) — il faut ré-identifier les tests flaky
réels depuis les logs CI + root-cause AVANT de toucher `.config/nextest.toml`.
Aucun DESIGN-CONFLICT. Aucune décision D1..D5/Day-0/pre-launch ré-ouverte.

HEAD vérifié : **6fcf51d** (Phase A `f4b257c` + Research Packs/Q12 `6fcf51d`).
Arbre de travail propre au démarrage.

## Contexte

Phase A1 = **lot carries CI/hygiène HORS-thème Workflow Agents** (condition 2 audit
S82, kickoff §6.2). Sprint impair ⇒ lot volontaire. **Interdiction de toucher au
chantier** : `crates/sbfb-workflow-core`, `sprint83_gate_matrix.md`,
`sprint83_t2_spec.md`, `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`.

Commit cible : `fix(ci): Sprint 83 Phase A1 — lot carries hors-theme. 0 wire bump.`
Le titre `fix(ci): … Sprint 83 Phase A1` déclenche Codex + préflight + 9-sections
(lightcheck Check 7/7bis) ⇒ artefacts `sprint83_phase_a1_{preflight,review,codex_review}.md`
DUS. Le fix G1 (carry 8) **n'auto-bloque PAS** ce commit : `PHASE_UPPER=A1 ≠ "A"`.

**Décisions PO gravées 2026-07-18** :
- **SEEDER-DIAL-TAIL = CORRIGER** (pas de WONTFIX) → sampling déterministe + test + doc.
- **macos-14 watcher = CORRIGER** (pas de pérennisation) → fix backend/override, tests stables.

**0 wire bump garanti** sur les 9 carries : aucun `*_VERSION` / `FEED_FORMAT_VERSION` /
`DOMAIN_*_V1` touché. SEEDER = réordonnancement best-effort (aucune enveloppe) ;
TEST-ISOLATION = struct Rust test-only ; goldens = caractérisation ; CI/hook/docs = infra.

## Plan de fix par carry

### Carry 1 — CI-1 supply-chain (licences + pip zombies + re-label + doc) — EXECUTE
- **État (fichier:ligne)** : `deny.toml:103-119` allowlist SPDX SANS `Unlicense` ni
  `MIT-0` ; `deny.toml:122` `exceptions = []`. Manifestes lus (refutation A1-guard) :
  `async_executors 0.7.0` = `license = "Unlicense"` (MONO, pas dual) + `borrow-or-share
  0.2.4` = `license = "MIT-0"` (MONO) ⇒ `cargo-deny check licenses` ROUGE certain (pas
  de licence de repli allowlistée). Job pip-audit `supply-chain.yml:76-124`, matrice
  `nexus-sdk/nexus-coordinator/nexus-app-gov` (`:86-89`) : `git ls-files packages` = 0
  (100 % non-tracké, purgé S50-51) ; seul `examples/hello-world-app/pyproject.toml`
  tracké (hors matrice) ⇒ `uv export --package X` (`:110`) échoue = 3 legs zombies ⇒
  job entier mort. `deny.toml:128-137` = P2-AUDIT-2-RESIDUEL (section `[bans]`,
  `multiple-versions="warn"`, distinct des licences). `CLAUDE.md:439` **annote DÉJÀ**
  la claim « supply-chain attendu vert au push » comme FAUSSE (audit S82 `74531d4`) ;
  le statut rouge subsiste à `:225` et `:436-439`.
- **Approche** : (1) `deny.toml:122` — **exceptions par-crate** (plus serré qu'élargir
  l'allowlist globale) : `exceptions = [ { name = "async_executors", version = "=0.7.0",
  allow = ["Unlicense"] }, { name = "borrow-or-share", version = "=0.2.4", allow =
  ["MIT-0"] } ]` + commentaire `# reason:` (arbre depuis S20/S32, pas de remplaçant
  amont). (2) `supply-chain.yml` — **supprimer le job pip-audit ENTIER** (`:76-124`,
  pas seulement la matrice) + retirer sa mention de l'en-tête ; conserver `cargo-deny`
  (`:48-74`) et `npm-audit` (`:129-154`). (3) `deny.toml:128-137` — ré-étiqueter le
  commentaire « carry S82 » → « carry S83, external-blocked (upstream iroh RC ed25519-dalek) ».
  (4) `CLAUDE.md` (memory-update A1, **APRÈS** run CI vert) — passer `:225`/`:436-439`
  de « supply-chain rouge » → « supply-chain VERT (Unlicense/MIT-0 exceptés par-crate,
  pip-audit zombie supprimé) ».
- **Test / preuve** : `cargo deny check licenses` VERT local (0 erreur) ; job cargo-deny
  CI vert au push ; job pip-audit disparu.
- **Exit** : cargo-deny VERT (ou exceptions documentées ligne-à-ligne) ; pip-audit retiré ;
  P2-AUDIT-2-RESIDUEL ré-étiqueté external-blocked/S83 ; CLAUDE.md rectifiée.
- **Risque** : Faible. **Anti-magie OBLIGATOIRE** avant d'écrire l'exception : `cargo deny
  check licenses` confirme le SPDX exact de chaque crate — si l'un était dual (`Unlicense
  OR MIT`), cargo-deny prendrait déjà MIT (allowlisté) et l'exception serait un no-op ⇒
  vraie cause ailleurs (manifestes lus = MONO, confirmé). **npm-audit non vérifié vert** :
  le workflow n'est VERT que si cargo-deny **ET** npm-audit passent — confirmer npm-audit
  vert sinon le workflow reste rouge après A1. Ne PAS toucher `multiple-versions="warn"`.

### Carry 2 — CI-2 Node 20→22 (@lingui/cli) — EXECUTE
- **État (fichier:ligne)** : `node-version: 20` à **3 sites** (le carry n'en citait que 2) :
  `ci.yml:70` (job test/web), `ci.yml:176` (job factory-operator), `supply-chain.yml:139`
  (job npm-audit). `@lingui/cli ^6.4.0` vit dans `tools/factory-operator` (PAS `web/` —
  grep web/package.json = 0 hit lingui).
- **Approche** : bumper `20`→`22` à `ci.yml:70` et `:176` (+ renommer les steps `Setup
  Node.js 20`→`22` à `:67`/`:173`) ; recommander aussi `supply-chain.yml:139` (+ nom
  `:136`) pour uniformité (npm-audit, risque nul). Envisager d'ajouter `engines.node:
  ">=22"` à `tools/factory-operator/package.json` (absent aujourd'hui ⇒ rien n'enforce
  Node 22 en local après le bump CI — un contributeur Node 20 casse silencieusement).
- **Test / preuve** : `node -v ≥ 22` local + leg factory-operator vert au push.
- **Exit** : leg factory-operator (et job web) VERTS sous Node 22 ; sinon échec résiduel
  documenté dette arc-front.
- **Risque** : Faible (Node 22 = LTS, setup-node@v4 OK). **Mécanisme non prouvé au HEAD**
  (refutation A1-CI) : le job factory-operator (`ci.yml:155-193`) n'invoque JAMAIS
  `@lingui/cli` directement (steps = build/npm ci/tsc/eslint/vitest/gates) ; si @lingui
  est atteint c'est transitivement (eslint-plugin-lingui, ou postinstall) — **lire le
  log CI réel** pour identifier le step rouge, ne pas assumer globSync. Le bump `:70`
  (web/, 0 lingui) est hygiène/uniformité, pas un fix. Preuve « vert » = run CI réel
  (non produisible au HEAD) — ne pas restater « vert » avant le push (miroir de la faute
  CI-1 que CLAUDE.md flag).

### Carry 3 — macos-14 watcher (CORRIGER, décision PO) — EXECUTE
- **État (fichier:ligne)** : ~10 tests fs-watcher positive-reload rouges **déterministes**
  (TRY-2-FAIL, retries ne rattrapent pas). Root-cause (inférence solide, confirmée par
  2 refutations) : filtre d'égalité de chemin non-canonicalisé
  `event.paths.iter().any(|p| p == &path_thread)` dans **6 familles de watcher** prod :
  `tls_pinning.rs:450`, `auth.rs:764`, `bootstrap_allowlist.rs:391`,
  `pow_policy_loader.rs:126`, `consent.rs:498`, `rate_limit_policy_loader.rs:204`. Sur
  macOS `tempfile` crée sous `/var/folders/...` (symlink) alors que FSEvents rapporte
  le chemin canonique `/private/var/...` ⇒ égalité échoue ⇒ event droppé ⇒ reload jamais
  déclenché ⇒ `wait_for` expire. Linux (inotify)/Windows (RDCW) rapportent le chemin
  surveillé tel quel ⇒ verts. Les tests négatifs (valeur inchangée : `pow_policy_loader.rs:329`,
  `rate_limit_policy_loader.rs:428`, `consent.rs:982`, `pow_wire.rs:258`) PASSENT (hors
  set rouge). **10 tests dans 9 fichiers** : `keystore_integration.rs:154`, `tls_pinning.rs:835`,
  `bootstrap_allowlist.rs:641`, `pow_policy_loader.rs:294` + `:376`, `pow_wire.rs:218`,
  `consent.rs:945`, `rate_limit_policy_loader.rs:386`, `loopback_token.rs:149`, et le 10e
  = `runtime.rs:2384` `rate_limit_gate_reloads_live_policy` (via helper). `.config/nextest.toml`
  retries=0 (default)/1 (ci) ; `rust-ci.yml:127` `- macos-14`.
- **Approche (2 voies PO-sanctionnées, choisir à l'implémentation)** :
  - **(A) TEST-ONLY, recommandé pour le lot hygiène** : canonicaliser la base tempdir
    dans chaque fixture positive-reload puis construire le chemin surveillé depuis la
    base canonicalisée. **CRITIQUE (refutation A1-macos)** : utiliser `dunce::canonicalize`,
    **PAS `std::fs::canonicalize`** — sur Windows std renvoie le préfixe verbatim `\\?\C:\…`
    qui casserait le filtre et rendrait ROUGES des tests actuellement verts ; le repo
    utilise déjà `dunce::canonicalize` partout (atelier.rs, main.rs…) précisément pour ça.
    ⇒ ajouter `dunce` en **[dev-dependencies]** (test-only, 0 dep runtime) à
    `nexus-worker-core`, `nexus-shell-daemon-core`, `nexus-core-rs`, `nexus-shell-daemon`.
    Le 10e test (`runtime.rs:2384`) se corrige DANS le helper `build_engine_with_rate_limit_policy`,
    pas dans une fixture. Envisager un helper partagé dans `nexus-test-harness` (dep test
    commune) pour éviter 10 copies dispersées.
  - **(B) PROD-hardening, aussi PO-sanctionnée** : canonicaliser (dunce) le **parent**
    dans chaque `spawn()` des 6 familles (`let parent = dunce::canonicalize(parent)
    .unwrap_or(parent);`) — ferme le bug latent macOS ET corrige les 10 tests d'un seul
    point. Blast-radius = 6 surfaces prod (cross-plateforme), à peser.
  - **REJETÉ** : override nextest serial/retries (les échecs sont DÉTERMINISTES, retries=1
    déjà rouge = pas de latence — masquer un bug viole la doctrine `.config/nextest.toml`) ;
    `PollWatcher` sous cfg(test) (testerait un watcher DIFFÉRENT de celui qui shippe).
- **Test / preuve** : 10 tests VERTS et STABLES sur macos-14 (10 runs, PASS au 1er essai) ;
  `rust-ci.yml` leg macos-14 vert.
- **Exit** : fix committé + 10 tests stables + **re-run dual-platform APRÈS fix** (Win natif
  + Docker Linux) prouvant 0 régression (cœur du risque `\\?\` Windows). retries inchangés.
- **Risque** : Root-cause = INFÉRENCE (rig macOS absent en local). **GATE OBLIGATOIRE** :
  logger `event.paths` vs `path_thread` une fois sur un Mac AVANT de figer (transforme
  l'inférence en fait). Si contre-hypothèse latence FSEvents, canonicaliser ne suffirait
  pas — mais retries déjà rouge exclut la latence pure.

### Carry 4 — SEEDER-DIAL-TAIL (CORRIGER, ≥3 reports, **PROD SENSIBLE**) — EXECUTE
- **État (fichier:ligne)** : `seed_registry.rs:331` `ids.sort()` = ordre LEXICOGRAPHIQUE
  dans `seeders_recent` (`:315-333`) ; la clé `(project_id, archive_hash)` est **déjà
  lowercased** à `:319-322`. Consommateur `blob_serve_http.rs:144-172`
  `directory_pull_providers` itère cet ordre jusqu'à `PULL_PROVIDER_CAP` (`:166`, prend
  la TÊTE) ⇒ un Sybil à préfixe-hex bas occupe déterministement les slots capés et évince
  un seeder honnête. §P68 `placement.rs:313` `fn sampling_key(session_id: &str, pubkey:
  &[u8;32]) -> [u8;32]` = blake3(session||pubkey) ferme le tail WORKER mais est **privé**
  et prend un **`&[u8;32]`** ; les seeder ids sont des `String` hex (refutation A1-guard —
  pas de reuse verbatim). `PATTERNS.md:4154` CARRY-1 = mis-credit documenté ; commentaire
  stale `blob_serve_http.rs:140-143`. Registre ÉPHÉMÈRE jamais sérialisé (`seed_registry.rs:2-13`).
- **Approche** : (1) `placement.rs:313` — promouvoir `pub fn sampling_key` + relaxer
  `pubkey: &[u8;32]` → `&[u8]` (coercion array→slice, appelants placement `&a.worker_pubkey`
  compilent inchangés — à confirmer au build). Alternative propre = hoister `sampling_key`
  dans `nexus-core-rs` comme primitive `sampling::` partagée par placement + seed_registry
  (single-source §P68) — noté comme option, hors-budget hygiène A1. (2) `seed_registry.rs:327-331`
  — remplacer `ids.sort()` par `let ctx = format!("{project_id_lc}{archive_hash_lc}");
  ids.sort_by_key(|id| sampling_key(&ctx, id.as_bytes()))` (les 2 sont hex 64 chars =
  concat non-ambiguë ; contexte (projet,archive) ⇒ un Sybil ne pré-mine pas une clé
  gagnante pour TOUS les projets). Retourner TOUS les ids (best-effort intact), seul
  l'ORDRE change. (3) mettre à jour les asserts d'ordre lexicographique multi-éléments
  (`seed_registry.rs:395-397` `expected.sort(); assert_eq!` → même clé de sampling OU
  assert-en-set) ; single-elem intacts. (4) docs : `PATTERNS.md` §P68/CARRY-1 → CLOSED,
  corriger commentaire stale `blob_serve_http.rs:140-143`, doc de `seeders_recent`, ligne
  THREAT_MODEL.
- **Test / preuve** : `dial_set_sampling_distribution` (mod tests `seed_registry.rs`,
  miroir `placement.rs:790-799`) : (a) NON-lexicographique pour ≥6 seeders ; (b)
  reproductible (mêmes inputs ⇒ même ordre) ; (c) context-scoped (2 (project,archive)
  distincts ⇒ ordres différents pour le MÊME set) ; (d) aucun id droppé.
- **Exit** : test de distribution vert + doc §P68/CARRY-1 à jour ; `cargo nextest -p
  nexus-shell-daemon` + `-p nexus-coordinator-rs` verts dual-platform.
- **Risque** : **ÉLEVÉ (code PROD, promu S75 D)**. Contrat best-effort DOIT tenir :
  (1) ne jamais dropper un seeder valide (réordonner seulement) ; (2) déterministe (dials
  reproductibles) ; (3) intégrité BLAKE3 inchangée (menteur = 1 dial raté, jamais de
  mauvais octets). Confirmer au build que la relaxation `&[u8;32]`→`&[u8]` laisse compiler
  les appelants placement. Cap consommateur (`:166`) inchangé.

### Carry 5 — TEST-ISOLATION state.json (classe ≥2 sites, **PROD SENSIBLE**) — EXECUTE
- **État (fichier:ligne)** : **mécanisme CORRIGÉ** (A1-isolation confirmé + A1-guard
  refutation d'A1-isolation initiale). Le leak N'EST PAS `data_dir:None` ni
  `SBFB_HOME`/`~/.sbfb`. La destination du flush = `state_flush_path_override`
  (`runtime.rs:190` champ + `:560` `set_state_flush_path`) ; quand `None` (défaut `:548`)
  → `worker_state_file()` (`paths.rs:80`) = **`~/.nexus-grid/worker/state.json`** via
  BaseDirs (racine DIFFÉRENTE de `~/.sbfb`). `flush_state_snapshot` (`runtime.rs:1332-1342`)
  appelé 3× dans `run_until_shutdown` (`:772`/`:800`/`:813`). `set_state_flush_path` a
  **0 caller** (grep). `sbfb_home_override` confine `~/.sbfb/` (consent/rate-limit) mais
  **PAS** `~/.nexus-grid/`. Sites de test qui bootent+run sans confiner le state (donc
  écrasent le state.json live) : `result_sync.rs:510` (épingle sbfb_home mais PAS le state
  path), `dispatch_loop.rs:218`, helper `runtime.rs:1979` (consommé par `:2067`), + sites
  worker-core `runtime.rs:1606`/`:1697`/`:1855` confirmés spawn+flush non-confinés (`:1769`
  à confirmer) ⇒ **~6-7 sites** (pas 2). `runtime.rs:4638` déjà confiné via env
  `NEXUS_GRID_ROOT` (`:4586`). Prod = `nexus-worker/main.rs:199` (reste None, légitime).
  Ces tests sont NON-`#[ignore]`, in-process, dans la suite standard ⇒ polluent aussi le
  runner CI.
- **Approche** : root-cause = confinement non-déclaré-à-la-construction (0 caller du
  mutateur, facile à oublier). (1) rendre le champ `state_flush_path_override: Option<PathBuf>`
  **explicite dans `EngineBoot`** (`runtime.rs:92` struct + `:250` destructure + `:548`
  assigner le champ au lieu du None hard-codé) — garder `set_state_flush_path` pour compat ;
  **initialiser aussi `EngineBoot::new` (`:115`, 0 caller mais compile requis)**. (2) câbler
  les ~6-7 sites de test avec `Some(tempdir/worker/state.json)` — **meilleur levier =
  câbler DANS les 2 helpers** (`build_engine_with_stub_ollama:1606`,
  `build_engine_with_rate_limit_policy:1979`) qui possèdent déjà un tempdir ⇒ auto-confine
  tous les consommateurs présents et futurs. (3) seul `main.rs:199` reste None. (4)
  **enforcement** : NE PAS mettre un garde `#[cfg(test)]` central dans `flush_state_snapshot`
  (ne franchit pas la frontière de crate vers nexus-shell-daemon) ; NE PAS keyer sur
  « NEXUS_GRID_ROOT absent » (le binaire worker en debug l'a légitimement absent) ⇒
  l'enforcement viable = confinement déclaré-au-boot + `test_engine_confined_root`.
- **Test / preuve** : `test_engine_confined_root` (mod tests worker-core) — booter un
  Engine avec `state_flush_path_override=Some(tmp/worker/state.json)`, driver un flush,
  asserter dest SOUS tmp et `!= worker_state_file()` BaseDirs. Grep prouvant 0 site de
  test **driven** sans override ni env NEXUS_GRID_ROOT.
- **Exit** : Engine de test refuse d'écrire hors racine confinée ; tous les sites drivés
  épinglent leur racine (override OU env) ; suites vertes Win+Docker. (mtime `~/.nexus-grid/
  worker/state.json` inchangé = smoke sanity, PAS le gate primaire.)
- **Risque** : **ÉLEVÉ**. (1) ajouter un champ à `EngineBoot` casse la compilation de
  TOUS les littéraux `EngineBoot { .. }` (~9 sites) — blast-radius Rust pur, grep exhaustif
  `EngineBoot {`. (2) **NE PAS** passer par `std::env::set_var(NEXUS_GRID_ROOT)` : sous
  `cargo test` shared-process (contrat P2-A-1) c'est process-global ⇒ race entre tests
  parallèles ; préférer l'override per-Engine (déterministe, parallel-safe). (3) le garde
  ne doit PAS vivre dans la dérivation de path PROD.

### Carry 6 — 0-golden familles (net §P75 étendu) — EXECUTE
- **État (fichier:ligne)** : net §P75 dans `test_support.rs` avec 9 `golden_http_*`
  (`public_tier:351`, `shard_session:394`, `seed:427`, `frost:469`, `coordinator:502`,
  `curators:544`, `publish:582`, `cors_preflight_raw:628`, `spa_fallback:669`) + infra
  (`GoldenCase:281`, `golden_check:294`, `golden_run:340`, `GOLDEN_VOLATILE_FIELDS:245`,
  `golden_redact:252`). **8 familles à 0-golden** dont les routes existent : browse
  (`http.rs:299`), browse/pull (`:301`), nodes (`:339`), feed/* (`:492-509`), search
  (`:512`), proof-card (`:514`), preview/load (`:519`), provenance (`feed_api::get_provenance`,
  `:547`). Aucun `golden_families_stable`.
- **Approche** : ajouter 8 `golden_http_*` dans **`test_support.rs`** (JAMAIS dans http.rs,
  §P75 pt5) sur le pattern `GoldenCase`/`golden_run`. `mk_state()` = nœud vide ⇒ la plupart
  pinnent le chemin vide/404/liste-vide. Étendre `GOLDEN_VOLATILE_FIELDS` UNIQUEMENT pour
  les champs volatils nouvellement observés (allowlist minimale, §P75 pt2). Ajouter
  `golden_families_stable` : rejoue chaque famille 2× contre `mk_state()` frais, assert
  byte-identique après `golden_redact` (oracle ±0 EXACT, §P75 pt3). Étendre `PATTERNS.md`
  §P75 (~`:4314`) — **§P est l'espace COMMUN rust+shell** (piège S82), vérifier l'unicité
  du numéro.
- **Test / preuve** : 8 nouveaux `golden_http_*` + `golden_families_stable` verts sur 2 runs.
- **Exit** : delta annoncé = +9 ; §P75 mentionne les 8 familles ; oracle ±0 EXACT cité.
- **Risque** : Faible-Moyen. Documenter EXPLICITEMENT chaque golden comme « contrat
  nœud-vide » (feed/provenance/proof-card ⇒ 404/empty, pas le chemin peuplé) pour ne pas
  induire un futur lecteur. Redaction trop large masque la dérive (allowlist stricte).
  Vérifier que proof-card/provenance acceptent un id factice sans panic.

### Carry 7 — flake dispatch_loop + running_json/sigint + boot_path_reenters — **PLAN-ADAPT**
- **État (fichier:ligne)** : `.config/nextest.toml` — test-group `two-node-convergence
  { max-threads = 2 }` (`:27`) + override (`:37-45`) filtre
  `test(/(convergence_|without_start_sync|reenters_sync_set|duress_skips_sync_set)/)`,
  slow-timeout 60s×3. **CIBLE PROPOSÉE FAUSSE (refutation A1-hygiene, vérifiée au HEAD)** :
  `dispatch_loop_writes_to_doc` (`dispatch_loop.rs:103`) boote **UN** nœud (`create_node`
  ×1 `:104`), `flavor=multi_thread, worker_threads=2` ; `dispatched_task_is_claimed`
  (`:182`, worker_threads=2) ne boote pas 2 nœuds iroh. Le groupe `two-node-convergence`
  est **spécifiquement** pour les tests bootant 2 nœuds iroh (rationale `:18-25`) ⇒ les y
  ajouter contredit le groupe. `boot_path_reenters_sync_set` (`:646`, 2 nœuds) est **DÉJÀ**
  couvert par le filtre `reenters_sync_set` (`:38`) — s'il flake sur Win malgré le groupe,
  le cap n'est PAS le remède. running_json/sigint : `runtime.rs:4388`/`:4443` (registry
  singleton + running.json partagé), e2e sigint `:282` — hors test-group.
- **Approche (PLAN-ADAPT)** : **NE PAS** élargir le filtre vers dispatch_loop_writes/
  dispatched_task (mauvaise cible, band-aid). Avant tout edit `.config/nextest.toml` :
  (1) **extraire les noms de tests flaky RÉELS** des logs CI Win/Docker (aucun log cité
  au HEAD — evidence gap) ; (2) **root-cause** chaque flake (synchronisation explicite
  plutôt que sérialiser un flake non prouvé) ; (3) `dispatch_loop_writes_to_doc`
  s'auto-synchronise déjà (boucle polling 10s, P2-A-1 fixé S73 B) ⇒ probablement pas flaky.
  Pour running_json : confirmer si contention harness vs vraie course registry AVANT de
  poser un test-group serial (un serial peut MASQUER une course). Documenter la classe
  ligne-à-ligne comme le bloc `:17-25`.
- **Test / preuve** : 2 runs verts consécutifs (Win+Docker) sur les tests **réellement**
  flaky une fois identifiés.
- **Exit** : cible ré-identifiée depuis logs CI + root-cause documentée ; override/fix
  ciblé committé ; delta tests = 0 (config) ou fix ciblé.
- **Risque** : Moyen. Sérialiser un flake non prouvé = band-aid (viole no-band-aid). Ne
  PAS gonfler les deadlines. **Ce carry ne doit pas être exécuté « tel que proposé »** —
  d'où PLAN-ADAPT : re-scoper depuis les faits CI avant code.

### Carry 8 — Hook lightcheck Check 5 mort (G1) — EXECUTE
- **État (fichier:ligne)** : `phase-precommit-lightcheck.sh:89` `PHASE=$(… tr [:upper:]
  [:lower:])` ⇒ minuscule ; `:90` `PHASE_UPPER` existe déjà ; `:256` `if [ "$PHASE" = "A" ]`
  compare minuscule vs `"A"` MAJUSCULE ⇒ **JAMAIS vrai** ⇒ tout le bloc G1 Design Review
  gate (`:256-287`, BLOCK + exemption kickoff `G1 skipped`) = CODE MORT. `PHASE_UPPER`
  déjà utilisé ailleurs (`:302`).
- **Approche** : `:256` → `[ "$PHASE_UPPER" = "A" ]`. **Ne changer QUE `:256`** — d'autres
  sites (`:297` chemin d'artefact) utilisent `$PHASE` lowercase et doivent le RESTER.
- **Test / preuve** : `test_lightcheck_g1_case_insensitive.sh` — titre `Sprint 99 Phase A`
  + design_review absent + kickoff sans exemption ⇒ exit ≠ 0 + message BLOCK ; variante
  avec `G1 skipped` ⇒ pass.
- **Exit** : Check 5 bloque un commit Phase A sans design_review ; test vert ; exemption
  (`:268`) honorée.
- **Risque** : Faible. Activer un BLOCK jusqu'ici mort = comportement VOULU (§6.1.1). **Le
  présent commit A1 n'est PAS auto-bloqué** : `PHASE_RAW=A1` → `PHASE_UPPER=A1 ≠ "A"`
  (vérifié) ⇒ Check 5 ne se déclenche pas sur A1.

### Carry 9 — nits M-7 + C-2 + spdx/shellcheck (P3) — EXECUTE (C-2 à localiser)
- **État (fichier:ligne)** : **M-7** = `test_support.rs:392` doc-comment `/// Shard-session
  domain (split target: 6 loopback handlers).` — cadrage « split target » périmé (split
  N→S4 TERMINÉ S82) ; `sprint83_audit_plan.md:162` dit encore « http.rs doc-comment »
  alors que le bloc golden a migré vers `test_support.rs:392`. **C-2** = `http.rs:2-33`
  module-doc narre encore la surface monolithique (~12 routes `:9-21`), ne mentionne NI le
  rôle hub résiduel (assemblage build_router) NI les modules domaine `*_api.rs` (**21 sur
  disque**, pas 12 — 12 = figure d'extraction S82). **spdx** Woodpecker #38 = glitch env
  runner (P3) ; **shellcheck** chronique 3 warnings (P3).
- **Approche** : M-7 — reformuler `test_support.rs:392` (retirer « split target » →
  « golden domain: shard-session, 6 loopback handlers ») + rectifier `audit_plan.md:162`
  pour pointer `test_support.rs:392`. C-2 — réécrire `http.rs:2-33` (rôle résiduel
  assemblage build_router + renvoi aux modules domaine `*_api.rs`, sans inventorier 89
  routes) ; **citer le compte exact (21 modules)**. spdx — documenter le glitch env
  (non-code) ou re-déclencher. shellcheck — corriger les 3 warnings OU
  `# shellcheck disable=SCxxxx` + raison ligne-à-ligne (jamais muet).
- **Test / preuve** : doc-only, delta tests = 0.
- **Exit** : `test_support.rs:392` + `http.rs:2-33` reflètent le post-split ;
  `audit_plan.md:162` corrigé ; P3 fixés ou justifiés ligne-à-ligne.
- **Risque** : Faible SAUF **C-2 « module-doc hub »** : **AUCUN module-doc « hub » hors
  chantier n'est localisé** (le seul crate Hub = `sbfb-workflow-core`, **OFF-LIMITS**
  sprint impair). ⇒ **LOCALISER C-2 avant tout edit** : s'il vise `http.rs` (rôle hub
  résiduel du router), OK ; s'il vise `sbfb-workflow-core`, le **ROUTER HORS A1**. Ne pas
  reclasser `audit_plan.md` comme wire-format par le hook.

## Risques prod

- **SEEDER (`seed_registry.rs`) — ÉLEVÉ** : seul carry qui modifie du code PROD dans le
  hot-path de dial. Le fix ne doit RÉORDONNER que (retourner tous les ids, jamais en
  dropper), rester DÉTERMINISTE par contexte (project||archive), préserver l'intégrité
  content-addressing BLAKE3. Registre éphémère jamais sérialisé ⇒ 0 wire. Vérifier au
  build la relaxation de signature `sampling_key`.
- **TEST-ISOLATION — test-only mais structurel** : le champ `EngineBoot` casse tous les
  littéraux (grep exhaustif requis) ; enforcement per-Engine (jamais env global sous
  shared-process). Aucune modif de la dérivation de path prod.
- **macos watcher — test-only par défaut (voie A)** : risque `\\?\` Windows si
  `std::fs::canonicalize` au lieu de `dunce` ⇒ re-run dual-platform obligatoire. Voie B
  (prod-hardening) PO-sanctionnée mais blast-radius 6 surfaces prod.
- **CI-1/CI-2** : preuve « vert » = run CI réel non produisible au HEAD ; ne pas restater
  « vert » avant le push (miroir de la faute flaggée CLAUDE.md:439). npm-audit à confirmer.

## Scope / décisions gelées / 0 wire bump

- **0 wire bump** confirmé sur les 9 carries : aucun `*_VERSION`/`FEED_FORMAT_VERSION`/
  `DOMAIN_*_V1` touché.
- **Chantier OFF-LIMITS** (sprint impair) : `crates/sbfb-workflow-core`,
  `sprint83_gate_matrix.md`, `sprint83_t2_spec.md`, `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`.
  Point de vigilance C-2 (carry 9).
- **Décisions gelées intactes** : pin iroh `=1.0.1` + `multiple-versions="warn"` ;
  contrat best-effort seeder (héberger≠publier, seeder≠auteur) préservé ; allowlist
  licences = POLICY supply-chain (pas Day-0).
- **Décisions PO 2026-07-18 respectées** : SEEDER=CORRIGER (sampling+test+doc, pas WONTFIX) ;
  macos=CORRIGER (fix backend/override, pas de pérennisation ignore).

## Ordre de commit + suites de vérification

Commit unique `fix(ci): Sprint 83 Phase A1 — lot carries hors-theme. 0 wire bump.`
(Codex + préflight + 9-sections DUS.)

**Ordre par risque croissant (implémentation)** :
1. **Trivial / doc** : G1 hook (8), M-7+C-2 (9), CI-2 node (2).
2. **CI policy** : CI-1 deny.toml + suppression pip-audit + re-label (1).
3. **Caractérisation** : 0-golden familles (6).
4. **PLAN-ADAPT — bloquant tant que non re-scopé** : flake nextest (7) — extraire noms CI
   + root-cause AVANT edit.
5. **Prod-sensible (root-cause + tests)** : SEEDER (4), TEST-ISOLATION (5).
6. **CI-only lent (rig absent)** : macos-14 watcher (3) — GATE log Mac + dual-platform.

**Suites de vérification (fail-fast complet avant commit, jamais filtrer par langage)** :
```
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked          # Win natif + Docker sbfb-ci (dual)
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
cargo deny check licenses                        # carry 1
bash .claude/hooks/tests/test_lightcheck_g1_case_insensitive.sh   # carry 8
# Frontend (carry 2 impact): (cd web && npm run lint && npx tsc --noEmit && npm run test:unit && npm run build && npm run size)
```
Preuve CI-1/CI-2/macos = **run CI réel au push** (non produisible au HEAD). macos = 10
runs stables leg macos-14. SEEDER/TEST-ISOLATION = tests unitaires dédiés + dual-platform.
