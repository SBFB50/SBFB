# Sprint 83 Phase I1 — Preflight G8 (delta-scan sur socle Phase I)

Date : 2026-07-22. HEAD `e48d8280` (Phase I DONE). Socle : `sprint83_phase_i_preflight.md`
(§10 adaptations 1..23, §11.4 interdits, §11.5 invariants) — **vaut intégralement**, seul le
delta routes daemon est re-scanné ici (adaptation 22 : « le preflight I vaut socle pour I1,
delta-scan léger au démarrage de I1 »).

Méthode : Workflow ultracode `wf_8833b41e-541` — fan-out 5 scans factuels (état daemon http,
surface store engine, inventaire D7 + THREAT_MODEL, golden_http + discipline tests, clôture
docs-contrat) + 3 agents adversariaux (concurrence/sécurité, périmètre, complétude), 8/8
agents Opus 4.8 1M, 172 tool-uses. Synthèse main-thread.

## Verdict G8 : PLAN-ADAPT

La promesse I1 (adaptation 22 du socle + body I §Scope cuts) est STRUCTURELLE : module
`workflow_api.rs` + routes dans `authed_routes` + famille golden_http + compte 89→89+N +
inventaire D7 §3 + clôture docs-contrat. Elle est fermée par une coupe **reads-only**.
L'adaptation majeure (A1 ci-dessous) diffère la famille MUTANTE (restore/GC/lease-force) à
son consommateur réel (J/K) avec rows D7 pré-classées T1 datées — evidence concrète, aucune
Day-0 touchée, cohérent avec le précédent S2A-9 (traits store différés à J, adaptation 23 du
socle).

## 1. Base mesurée à HEAD

- `workflow_api.rs` ABSENT (0 collision) ; 21 modules `*_api.rs` ; `nexus-shell-daemon/Cargo.toml`
  ne dépend PAS de `sbfb-workflow-engine` (deps `:21-32`, pattern intra-workspace = PATH).
- `grep -c ".route(" crates/nexus-shell-daemon/src/http.rs` = **89 EXACT**. Aucune assertion
  Rust `==89` : le compte est verrouillé DOC (`LOOPBACK_ENDPOINTS_TRUST_TIERS.md:5,67` +
  prose §P75 `PATTERNS.md:4360`).
- `build_router` = assemblage unique (`http.rs:247-637`) ; `authed_routes` `:285-622` terminé
  par le layer `auth_required` `:622` (bearer constant-time + Host + Origin,
  `nexus-shell-daemon-core/src/auth.rs:395` ; peer-creds au niveau listener). Les modules
  `*_api.rs` n'exportent QUE des handlers ; les `.route()` sont câblés centralement
  (modèle shard-session `http.rs:345-375`, syntaxe path-param `{session_id}`).
- `DaemonHttpState` `http.rs:76-203` (Clone) ; état de test construit par `mk_state*`
  (`test_support.rs:114-201`, tempdir leaké `:149`) ; `CoordinatorDb` de test =
  `open_in_memory()` `:170` — **`RunStore` n'a AUCUN constructeur in-memory** (WAL fichier-only).
- Surface `RunStore` (engine, `store.rs`) : lectures `&self` non-fencées (`run_status:547`,
  `terminal_reason:560`, `spec_digest:598`, `event_count:1243`, `frontier_node:1230`,
  `lease_state:1417`, `inbox_count:1471`, `orphan_outbox_rows:1201`) ; mutations
  (`gc:1621`, `backup_into:1838`, `restore_from_backup:1938` = fn ASSOCIÉE,
  lease `&mut self` `:1266/:1344/:1400`) ; `record_intent_*`/`record_completion` =
  `pub(crate)` hors `spike-faults` → inatteignables du daemon (bien). Pas de `list_runs`
  (seul scan privé `gc_impl:1660`).
- THREAT_MODEL v23 §19 posé par I (`:1743-1781`, asset A11 `:69`, 7 rows T-RUN-*) ; les routes
  I1 sont ANNONCÉES (`:1773-1775` « exposition loopback … = Phase I1 ») sans row d'endpoint.
- Duress (`noop_identity.rs` module-doc) : sous duress les données locales sont RÉELLES (seul
  le keypair est decoy), la surface reste responsive ; SEULES les opérations
  sortantes/signantes sont no-op. Les GET locaux (browse, shard-session status) ne sont PAS
  gatés.
- Docs : `docs/factory/llms.txt:45-58`/`REFERENCE.md:89-192` indexent l'API OPERATOR (:3001),
  pas le daemon ; précédent EXACT pour une famille de routes daemon = layer par-chantier
  `docs/sharding/{llms.txt,REFERENCE.md}` (`docs/sharding/llms.txt:38` = module handlers +
  « routes stay registered in http.rs build_router/authed_routes »). Root `llms.txt:8-13`
  n'indexe que sharding+factory.
- `check-frontier-contracts.sh` : frontier-tag OPT-IN (type non-annoté ≠ violation) ; census
  DOMAIN 29 gelé ; l'étiquette de frontière des routes loopback se fait EN PROSE (inventaire
  D7 + REFERENCE/llms.txt + golden_http), `// FRONTIER:` réservé aux wire signés (interdit
  11.4 du socle confirmé).

## 2. Passe adversariale — réfutations retenues

- **(concurrence)** Le CAS epoch est CONDITIONNEL (`store.rs:2288` `if let Some(..) =
  &self.active_lease`) : un handle sans `acquire_lease` écrit UNFENCED sans jamais lever
  `LeaseLost`. « Single-writer » = discipline, pas verrou. GC est sûr par RELECTURE in-txn
  (`store.rs:1686-1720`), pas par le CAS. → un seul handle daemon, zéro write en I1.
- **(concurrence)** `restore_from_backup` = fn STATIQUE (`store.rs:1938`) : le Mutex ne la
  médie pas ; garde « store CLOSED » = sharing-violation **Windows-only** (`store.rs:1923-1927`),
  sur POSIX `fs::copy` réécrirait SOUS la connexion vivante → route restore online = arme
  chargée, incompatible avec un handle boot-tenu façon `coordinator_db`.
- **(sécurité)** T1 CONFIRM_PROMPT est **design-only** (`LOOPBACK…:4,:24`) : toute route
  « tier cible T1 » ship à T0 EFFECTIF (bearer+Host+Origin, + bypass peer-creds UDS/NP
  `auth.rs:412`). Exposer restore/GC/lease-force maintenant = actions destructives/vol de
  writer (`acquire_lease` takeover `store.rs:1317-1332`) joignables à T0.
- **(scope)** GC inopérant avant J : `RetentionPolicy` sans défaut (`store.rs:415`) + rétention
  temporelle exige `closed_at_ms` qu'aucune source ne fournit avant le trait clock (J).
  lease-force sans appelant légitime : le futur runtime acquiert son lease EN INTERNE (J).
- **(scope)** run_id : `validate_identity_component` (`hash.rs:93-113`) refuse seulement
  vide/>512 o/control chars — ce n'est PAS un garde path-safety (accepte `/`, `..`). SQL sans
  risque (`params!` partout). La route doit valider un charset borné et ne JAMAIS former un
  path depuis run_id.
- **(scope)** « timeline » : une DTO par-événement (kinds+seq+timestamps) = projection
  RunEvent prématurée — c'est exactement `WorkflowPublicProjection` (différée, interdit 11.4).
  Le tier « timeline T0 » de B3 est satisfait par l'AGRÉGAT scalaire de progression.
- **(complétude)** Plomberie greenfield à 4 sites ENSEMBLE (dep Cargo + champ
  `DaemonHttpState` + boot `runtime.rs` + `test_support.rs` mk_state) ; sans le 4e site la
  famille golden_http ne boote pas. Commit-target I1 non défini au plan → défini ici (A9).
  `probes` de `golden_families_stable` = optionnel (précédent shard-session hors probes).

## 3. Adaptations que le code DOIT suivre (autorité : ce préflight, sur socle I)

A1. **Famille mutante DIFFÉRÉE J/K** — restore/GC/lease-force NE sont PAS exposées en I1.
    B3 (tiers « délibéré par famille ») est honoré par des **rows D7 pré-classées T1 avec
    implémentation DATÉE J/K** (précédent : row `/canary/cosign` FUTUR absente du routeur,
    admise par `inventory_policy:5`). Scope cut DATÉ au body (jamais un silence). Evidence :
    T1 design-only, restore fn statique Windows-only-guard, GC clock=J, lease-force = vol de
    writer sans appelant, S2A-9 (même règle que traits store → J, socle adaptation 23).
A2. **N=2 routes READ (T0), inline dans `authed_routes`** (jamais public_routes, jamais un
    sous-Router mergé — casserait le grep count ET la couche auth) :
    - `GET /api/daemon/workflow/run/{run_id}` → agrégat scalaire par-run : `{found, run_id,
      status, terminal_reason, event_count, frontier_node, spec_digest}` — DTO daemon-local
      serde (RunStatus/LeaseState n'ont pas serde ; mapping manuel status →
      "running"/"completed"/"terminal"). Miss = 200 `{found:false, …:null}` JAMAIS 404
      (précédent seed_count/shard-session).
    - `GET /api/daemon/workflow/lease` → `{held, owner, epoch, expires_at_ms}` (`lease_state`,
      owner-vide filtré par l'engine).
    EXCLUS de toute réponse : `Vec<RunEvent>` (`load_events`), `read_snapshot` (modèle
    reducer=J), `completed_effect`/`pending_effect` (receipts matériels), inbox détail.
    `event_count`/`frontier_node`/`orphan_outbox_rows` : les deux premiers DANS l'agrégat,
    orphan_outbox exclu (diagnostic interne).
    *Note r1 (déviation ratifiée)* : livré en enveloppes IMBRIQUÉES `{found, run:{…}}` /
    `{found, lease:{…}}` (booléen uniforme `found`, précédent shard-session
    `{found, session}`) au lieu de l'esquisse plate/`held` ci-dessus — allowlist de
    CONTENU strictement identique, forme gatée par `golden_http_workflow_domain` +
    key-set test, docs alignées (REFERENCE/THREAT/D7).
A3. **Compte routes 89 → 91.** Cibles doc : `LOOPBACK…:5` (front-matter counts) + `:67`
    (chapeau §3). §P75 (`PATTERNS.md:4360`) décrit la série de splits S82 (passé immuable) —
    ne PAS réécrire ; ajouter la note datée S83 I1 dans LOOPBACK changelog.
A4. **Plomberie 4 sites** : (i) `Cargo.toml` daemon `sbfb-workflow-engine = { path = … }`
    SANS feature `spike-faults` (+1 ligne lock intra-workspace attendue, 0 dep externe) ;
    (ii) champ `DaemonHttpState.run_store: Arc<Mutex<RunStore>>` (vérifier `Debug` de
    RunStore — sinon impl manuelle/skip) ; (iii) boot EAGER `runtime.rs` miroir exact
    `coordinator.db` (`runtime.rs:315-323`) : `opts.paths.root.join("workflow_runs.sqlite3")`
    — PAS `~/.sbfb` (racine sécurité) ; (iv) `test_support.rs` : `mk_state*` ouvre
    `RunStore::open(<tempdir>/workflow_runs.sqlite3)` dans le tempdir leaké (pas
    d'in-memory possible).
A5. **Le daemon N'ACQUIERT PAS le lease en I1** : zéro write daemon-side ; les lectures sont
    non-fencées et sûres sous WAL (busy_timeout 5s) ; l'acquisition appartient au runtime J.
    UN SEUL handle, tout accès via le Mutex.
A6. **run_id serveur-validé (T-RUN-ID-PII)** : garde route stricte charset
    `[A-Za-z0-9._-]{1,128}` → 400 sinon ; ne forme JAMAIS un path ; SQL `params!` (déjà le
    cas engine). Pas de route create/list : I1 = read par run_id SEULEMENT, **0 méthode
    engine neuve, `sbfb-workflow-engine`/`-core` BYTE-INTACTS**.
A7. **Duress : PAS de gate** sur ces 2 GET (précédent : les reads locaux ne sont pas gatés,
    données locales réelles sous duress, gate = signal distinctif). Test dédié : sous
    `IdentityMode::Duress` les routes répondent À L'IDENTIQUE (aucun signal). Aucune
    mutation exposée ⇒ aucun risque destructif duress.
A8. **Clôture docs-contrat = layer par-chantier NEUF `docs/workflow/`** (`llms.txt` +
    `REFERENCE.md`) sur le patron sharding (module → handlers + envelopes + tier D7 +
    anchors), + ligne d'index dans le root `llms.txt` (3e subsystem). `docs/factory/` reste
    Operator-scoped ; `WORKFLOW_AGENTS_OVERVIEW.md` = croissance facultative (clôture AJ1),
    ne pas y déplacer l'autorité.
A9. **Commit-target I1** (absent du plan, défini ici) :
    `feat(daemon): Sprint 83 Phase I1 — run store loopback read routes (D7 inventory, golden_http)`
    Body 9 sections ; artefacts minuscules `sprint83_phase_i1_{preflight,review,codex_review}.md`
    + rounds numérotés §4.5.8. T1 = `N-A-no-frontend-change` ; integration-nightly = N-A.
A10. **LOOPBACK maintenance complète** : rows §3 (1 row T0 représentative famille « run store
    lecture seule » + 3 rows T1 FUTURES datées J/K avec duress-posture notée) + front-matter
    `last_validated` + ligne de précédent changelog (pattern S81 K) + décomptes
    `inventory_policy` mis à jour (rows et 89→91). Trigger « Nouveau endpoint loopback risky »
    = re-fired par CE commit.
A11. **THREAT_MODEL : concrétisation MINIMALE** de §19 « bornes honnêtes » (routes livrées
    reads-T0 ; mutations datées J/K) + row T-RUN-ID-PII (validation livrée) + changelog v24
    léger. Pas de nouvelle menace (aucune surface mutante exposée).
A12. **golden_http** : famille `golden_http_workflow_domain` (contrat EMPTY-NODE capturé en
    EXÉCUTANT, jamais dérivé du handler) + sondes ajoutées à `golden_families_stable`
    (recommandé, run1==run2 sur 2 nœuds frais) + 2 runs verts consécutifs sur HEAD (§P75).
    Champ volatil neuf éventuel → `GOLDEN_VOLATILE_FIELDS` (allowlist minimale).
A13. **Erreurs** : idiome dominant inline `(StatusCode, Json(json!({"error": …}))).into_response()` ;
    tests in-module `#[cfg(test)] mod tests` (projections pures sans boot + HTTP oneshot via
    `build_test_router`) ; 401/403 non re-testés par module (couvert `auth::tests` core).

## 4. Interdits (socle §11.4 + delta)

- 0 dep externe neuve, 0 bump wire, census DOMAIN 29 gelé, 0 `DOMAIN_*` produit.
- Pas de `// FRONTIER:` machine sur les routes (inventaire D7 + docs prose à la place).
- Pas d'édition de `sbfb-workflow-engine`/`sbfb-workflow-core` (byte-intacts, delta tests
  engine = 0) ; pas de route create/list/mutation ; pas de `Vec<RunEvent>`/snapshot/receipt
  dans une réponse ; pas de touche à `web/` (T1 N-A).
- Feature `spike-faults` JAMAIS activée par la dep daemon.

## 5. Invariants à vérifier (commande par invariant)

| Invariant | Commande | Attendu |
|---|---|---|
| census intact | `bash scripts/check-frontier-contracts.sh` | exit 0, 29 frozen |
| engine byte-intact | `git diff crates/sbfb-workflow-engine crates/sbfb-workflow-core` | vide |
| compte routes | `grep -c ".route(" crates/nexus-shell-daemon/src/http.rs` | 91 |
| routes vertes | `cargo nextest run -p nexus-shell-daemon --locked` | exit 0 |
| golden stable | famille + `golden_families_stable` 2 runs HEAD | verts ×2 |
| 0 dep externe | `git diff Cargo.lock` | +1 ligne intra-workspace seule |
| SPDX | `bash scripts/check-spdx.sh` | exit 0 |
| dual-platform | fmt/clippy/nextest workspace/doctests/release Win + Docker `sbfb-ci` 1.94 (`SBFB_TEST_HTTP_TIMEOUT_SECS=120`) | verts |

## 6. Scope NON couvert par I1 (frontières honnêtes)

- **Phase J** : reducer/scheduler, acquisition du lease par le runtime, trait clock, GC
  pilotable, traits store, route(s) de pilotage de run.
- **Phase K** : effets/idempotence runtime, fencing-token côté cible (fenêtre effet-externe).
- **Phases T/AJ1** : `WorkflowPublicProjection.v1` front-facing, timeline par-événement,
  verbes bridge Workflow, synthèse GUIDE chantier-wide + THREAT vNN de clôture.
- **Ops offline** : restore = geste opérateur-local (documenté §19), jamais une route tant
  qu'un lifecycle quiesce+drop n'existe pas.
