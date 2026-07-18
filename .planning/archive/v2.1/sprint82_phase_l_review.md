# Sprint 82 Phase L — Review (Workflow)

Date : 2026-07-15. Review ultracode = Workflow 8 dimensions
(1 fidélité-du-diff / 2 séquence-boot-observable / 3 sécurité-deep /
4 couplage-A↔L / 5 livrables-vs-plan / 6 tests-oracle-T1 /
7 patterns-docs-contrat / 8 ressources-handles-async), chacune = agent
review + agent de vérification adversariale de CHAQUE finding
(opus-4-8[1m]) ; 3 lentilles adversariales transverses ajoutées
(DIFF-FACTS, SECURITY, PROCESS/TESTABILITÉ). Phase L = **refacto PUR**
de `DaemonRuntime::start()` : éclater le monolithe boot (958 l) en
sous-fonctions boot nommées `<~150 l`, co-localiser les helpers
annonce/outbox, hook re-drive-on-ingest Phase A préservé.
**0 wire, 0 dep, 0 changement de comportement ni de séquence de boot
observable.** Covers `REFACTO-DAEMON-RUNTIME-START`. Preflight verdict
**PLAN-ADAPT** (`sprint82_phase_l_preflight.md`, 7 faits corrigés,
contraintes A-G + 11 paires ordonnées de sécurité + plan de découpe).

Diff review (working tree, pré-commit, PAS encore committé) = **un seul
fichier de code** : `crates/nexus-shell-daemon/src/runtime.rs`
(854 insertions / 696 suppressions, vérifié `git diff --stat`).
Artefact `.planning/active/sprint82_phase_l_preflight.md` (untracked,
à stager avec la phase). HORS PHASE, non reviewés, non-défauts :
`verification_blueprint.md` (tracké modifié, édition PO mi-session) +
`workflow_agents_app_conception_ultradeep_2026-07-15.md` +
`workflow_hub_product_conception_2026-07-15.md` (2 untracked recherche
PO). L'index est PROPRE (rien stagé → review bien pré-commit).

## Verdict: PASS

Aucun P0/P1. Après vérification adversariale, réconciliation
inter-dimensions et re-vérification disque par le synthétiseur :
**1 P2 (hygiène de scoping pré-commit, non-code) + 7 P3 distincts, tous
CONFIRMED**. Le refacto est **génuinement pur** : re-lecture multiset
triée (ws-strippée) HEAD↔working, chaque ligne de code
retirée/ajoutée s'explique par une extraction (Result-wrap, dédup `&Arc`,
`.to_vec()`/`.to_path_buf()`/`.to_string()` des paramètres empruntés,
re-source via `state.*`, reflow multiligne→inline) ou un déplacement
verbatim de free-function — **aucune condition ajoutée/inversée, aucun
opérateur changé, aucun argument permuté, aucun bloc dupliqué ou perdu**.
Les 11 paires ordonnées de sécurité sont toutes préservées (3 même
RENFORCÉES au type-level : clamp host intra-fn, DB en arg obligatoire,
`seed_driver_lock` créé une seule fois dans la fn qui câble les 2
consommateurs). La déviation `build_http_state`-non-extraite (arbitrage
demandé par la tâche) est **ÉVALUÉE LÉGITIME, non-P1**. Verdict initial
PASS-PENDING promu **PASS** après clôture des deux gates restants :
**Docker nextest 2103/2103 == baseline EXACT + doctests VERTS** (arrivé
pendant la review) et **Codex GPT-5.6 Sol CLEAN round 1 (7/7 CONFIRMÉ,
0 GAP, 0 PARTIEL)** — cf. `## Codex reconciliation` en fin de document.

---

## Dimension 1 — Fidélité du diff (refacto pur, CONFORME, 1 P3)

Analyse multiset triée (ws-strippée) HEAD `713f0fa` ↔ working tree :
100 % des lignes de code retirées/ajoutées expliquées par un renommage
documenté, un fmt-reflow (indentation réduite), une nouvelle
signature/type-retour, un wrap `Ok(...)`, ou un call-site. **7 helpers
relocalisés byte-à-byte** (extract+diff direct) :
`handle_project_announcement` (103 l), `handle_directory_announcement`
(42 l), `maybe_redrive_seed_on_ingest` (60 l), `restore_browse_from_outbox`
(21 l), `open_project_doc_for_dispatch` (34 l), `announcement_claims_own_node_id`
(5 l), `wrap_payload_with_pow_static` (14 l) — 0 différence. Chaque
sous-fn extraite + chaque helper relocalisé défini EXACTEMENT une fois
(0 copie stale / dead-code) ; comptage net top-level +7 (les 7 sous-fns
revendiquées ; `build_http_state`=0 def, cohérent avec la déviation).
`spawn_gossip_subscribe_task`, la boucle `select!` gossip et le hook
re-drive : AUCUN hunk dans le diff (non touchés, en contexte).

- **L-1 (P3, CONFIRMED — arbitrage demandé)** — Déviation
  `build_http_state` non-extraite, **ÉVALUÉE LÉGITIME (pas P1)**.
  Re-vérifié disque : le littéral `let http_state = Arc::new(DaemonHttpState {`
  siège inline à `runtime.rs:742` ; **0 ligne +/- du diff ne touche
  `app_storage`/`canary_registry`** → byte-identique à HEAD (le littéral
  n'apparaît dans aucun hunk changé). Pour un refacto PUR c'est le choix
  le PLUS SÛR : (a) le littéral contient des champs à expression-bloc
  avec drop-scope sensible — `canary_registry`, `app_storage` (bloc
  `{ let guard = coordinator_db.lock().unwrap(); ... }` std MutexGuard
  block-scope sans await), `sbfb_home` — dont l'ordre d'évaluation et le
  drop-scope seraient à préserver manuellement à l'extraction ; (b) un
  appel à ~28 arguments positionnels serait moins lisible et plus fragile
  que le littéral nommé ; (c) le préflight l'autorise EXPLICITEMENT
  (`sprint82_phase_l_preflight.md:339` « l'implémenteur peut regrouper
  autrement tant que A-G tiennent ») et signale lui-même le risque
  d'extraction de ce littéral (§S1a:98-99 = LE piège drop-scope/ordre-eval).
  Toutes les contraintes A-G tiennent sans l'extraction. Conséquence
  résiduelle (hors fidélité) : `start()` reste à ~602 l ; les 7 sous-fns
  extraites sont TOUTES `<150 l` (max ~91 l `boot_feed_recovery`). Aucune
  action.

## Dimension 2 — Séquence de boot observable (CONFORME, 1 P3)

Les **11 paires ordonnées S3** du préflight toutes préservées, re-lues
disque. #4 substitution duress `feed_sync_for_republish=None` calculée
AVANT et passée à `boot_feed_recovery` (signature `Option<&Arc<FeedSyncState>>`
déjà substituée, jamais re-dérivée) ; #5 clamp `127.0.0.1` AVANT
`TcpListener::bind` DANS `bind_api_listener` ; #8 singleton
`check_stale_or_bail` AVANT node/bind/`write_running` ; #9 `coordinator_db`
+ `seed_nonce_cache` AVANT `boot_node_identity` (SEED_ALPN factory
capturé dans les 2 bras d'identité) ; #10 `boot_replay_done` await AVANT
`run_boot_seed_driver` ; #11 même `seed_driver_lock`, re-annonce directory
FIRST. Les gates gossip #1 (PoW `verify_envelope` Err⇒continue), #2
(drop self-node_id-spoof AVANT `handle_project_announcement`), #3
(re-drive gaté `accepted && Some`) vivent dans `spawn_gossip_subscribe_task`
byte-identique → verbatim préservés. Séquence des logs
`info!`/`warn!`/`debug!` de boot IDENTIQUE (aucun log ajouté, supprimé,
déplacé ou reformulé). Ordre des `tokio::spawn` observables inchangé.

- **L-2 (P3, CONFIRMED)** — Reorder bénin : `boot_driver_state =
  Arc::clone(&http_state)` déplacé AVANT les janitors (était APRÈS en
  HEAD). Re-vérifié : le clone (`runtime.rs:819`) précède désormais
  `spawn_api_server:823` qui absorbe les 3 janitors ; en HEAD le clone
  était après les janitors. Le reorder est **forcé par le borrow-checker**
  (`spawn_api_server` prend `http_state: Arc<DaemonHttpState>` par valeur
  et `build_router` le consomme → le clone doit précéder le move), pas
  un choix libre. Effet observable NUL : `Arc::clone` = incrément de
  refcount synchrone, sans I/O ni log ni dépendance de donnée ; les
  janitors sont des `tokio::spawn` détachés lisant `http_state.*`, ne
  consomment pas le clone ; l'ordre interne des spawns
  (janitors→peer→serve) est inchangé DANS `spawn_api_server`. Consigné
  par transparence de la comparaison côte-à-côte ; aucun correctif.

## Dimension 3 — Sécurité deep (CONFORME, 1 P3)

Diff sûr. Substitution duress feed préservée (`boot_feed_recovery` prend
l'Option DÉJÀ substituée, `if let Some(fs)` sur les 2 blocs, aucun accès
à `feed_sync_state` → re-dérivation structurellement impossible en
interne, DURESS-BOOT-LEAK P1 tenu) ; `sync_set_entry_in_duress` dans les
3 boot fns byte-identiques ; `gossip_publish_in_duress` dans
`maybe_redrive_seed_on_ingest` byte-identique ; `identity_mode` capturé
une fois et threadé en arg OBLIGATOIRE aux puits duress S81
(`open_project_doc_for_dispatch`/`boot_storage_namespace`/`boot_feed_namespace`)
+ `http_state.identity_mode`, jamais default `Normal` hors `#[cfg(test)]` ;
clamp `127.0.0.1` AVANT bind ; `wire_auth` précédence env>rotated>static
+ garde `.filter(|t| !t.is_empty())` intacte ; bypass peer-creds scoppé
UDS/NP seul (`spawn_peer_listener` enveloppe le CLONE, `axum::serve` sert
le routeur non-enveloppé) ; `refuse_recreate_on_interrupted_migration`
+ `docs_migration_backup_path` byte-identiques (tripwire
`upstream_migration_backup_suffix_matches_shared_const` désarmé
impossible) ; aucun `std::sync::MutexGuard` retenu à travers un `.await` ;
`_pow_policy_watcher` + `tokens_watcher` rendus et threadés sur `Self`
(hot-reload PoW + rotation token non tués).

- **L-3 (P3, CONFIRMED — angle mort latent)** — `spawn_gossip_and_boot_seed_driver`
  source ses 9 dépendances gossip **transitivement via `DaemonHttpState`**
  (`state.node`/`curator_runtime`/`browse_aggregator`/`gossip_sender`/
  `pow_policy`/`pow_solve_cache`/`pow_keypair`/`curator_gossip_topic`/
  `coordinator_db`) au lieu de paramètres explicites (l'ancien code
  passait les locaux directement). Re-vérifié disque `runtime.rs:1470-1482` :
  chaque champ = `Arc::clone(&state.X)` du MÊME objet que le littéral
  `http_state` byte-identique (742-809) ; `pow_verify_cache` (absent de
  `DaemonHttpState`) correctement threadé en arg séparé (1448/1474).
  **Comportement byte-identique aujourd'hui.** Observation informative
  uniquement : la correction dépend désormais de la population correcte du
  littéral `http_state` ; un futur edit plaçant un `Arc` divergent dans
  `state.node` (ou l'un des 9) mis-câblerait silencieusement la tâche
  gossip (0 erreur compile, aucun test hermétique ne couvre cette égalité
  — angle mort déjà noté au préflight pour l'ordre A↔L). Atténuant : la
  tâche gossip dépendait DÉJÀ de `http_state` via `boot_driver_state`, la
  surface ajoutée est étroite. Aucune action cette phase.

## Dimension 4 — Couplage A↔L (garantie anti-double-emit `SeedAnnounced` de `19b92e6`) (CONFORME, 0 finding)

**Le couplage cardinal est FIDÈLE et même RENFORCÉ.** Re-vérifié disque :
`seed_driver_lock` créé EXACTEMENT une fois en prod
(`runtime.rs:1468 Arc::new(tokio::sync::Mutex::new(()))`, DANS
`spawn_gossip_and_boot_seed_driver`) ; le second hit
`tokio::sync::Mutex::new(())` est à `:3545` (module `mod tests`, un test
construit son propre `GossipTaskConfig`). Les DEUX consommateurs prod
reçoivent `Arc::clone` du MÊME lock : `GossipTaskConfig.seed_driver_lock:
Arc::clone(&seed_driver_lock)` (`:1487`) ET la closure boot driver
`let seed_driver_lock = Arc::clone(&seed_driver_lock)` (`:1494`, utilisé
`:1517 seed_driver_lock.lock().await`). C'est **l'option anti-split-brain
recommandée du préflight §B** (la sous-fn POSSÈDE la création et rend les
2 handles → split-brain / double-emit impossible). `boot_replay_done`
câblage correct : tx→config (`:1484`), rx→driver-timeout (`:1498`).
`BOOT_DRIVER_REPLAY_WAIT_SECS=90` inchangé (`:1497`). Re-annonce directory
`reannounce_directory_at_boot` (`:1511`) AVANT `{ let _guard =
seed_driver_lock.lock().await; run_boot_seed_driver(...) }` (`:1516-1518`)
— ordre #11 préservé. `redrive_coord` construit inline au site config
(`:1488`), consommateur unique. Le boot driver = SEUL émetteur
`SeedAnnounced` en prod. Hook re-drive gossip byte-identique. Aucun
finding.

## Dimension 5 — Livrables vs plan + scope (CONFORME, 1 P3)

Livrable 1 §L « éclater `start()` en sous-fns boot `<150 l` » : **MET** —
7 sous-fns boot extraites, TOUTES `<150 l` (`boot_node_identity:1071`,
`bind_api_listener:1128`, `restore_revocation_cache:1153`,
`boot_feed_recovery:1203` [≈91 l = la plus longue], `wire_auth:1306`,
`spawn_api_server:1350`, `spawn_gossip_and_boot_seed_driver:1446`).
Livrable 2 « regrouper helpers annonce/outbox » : **MET** — 4 sections
co-localisées contiguës avec headers commentés (Announcement ingest /
Re-drive-on-ingest / Outbox-replay / Boot namespaces-migration guards) ;
2 doc-comments historiquement mal attachés RÉATTACHÉS correctement (le
doc « project announcement » rendu à `handle_project_announcement`, doc
bref correct écrit pour `wrap_payload_with_pow_static` ; doc
« Remediation #7 restore » rendu à `restore_browse_from_outbox`) =
déplacements de lignes `///` uniquement, 0 ligne de code touchée. Scope
cut respecté : `spawn_gossip_subscribe_task` (410 l, la boucle gossip)
byte-identique, non éclatée (conforme préflight §A.2). Hook Phase A
préservé. 0 fichier hors `runtime.rs`, 0 dep, 0 wire (`grep
DOMAIN_|_VERSION runtime.rs = 0`).

- **L-4 (P3, CONFIRMED — record honnête)** — `start()` résiduel = **~602 l**
  (`276→877`, réduction 958→602 = ~356 l / ~37 %). **RECTIFICATION
  INTER-DIMENSIONS** : le scan Dim 5 avait chiffré le résiduel à « 761 l
  (276→1036) », le scan Fidélité à « ~601 l (276→877) ». **Re-vérifié
  disque par le synthétiseur : `start()` se ferme au `}` ligne 877 (le
  `Ok(Self{..})` clôt à 876) ; les lignes 879-1035 sont les accesseurs
  `bound_addr()`/`curator_runtime()`/`revocation_cache()`/`shutdown()` ; la
  ligne 1036 ferme le bloc `impl DaemonRuntime`, PAS `start()`.** Le chiffre
  correct est donc **602 l**, le « 761 l » conflatait `start()` avec
  `shutdown()` + accesseurs. La lentille adversariale PROCESS a
  erronément confirmé 761 (elle a commis exactement la conflation
  qu'elle reprochait au scan Fidélité) ; les lentilles DIFF-FACTS et
  SECURITY ont correctement établi 877/602. La CONCLUSION QUALITATIVE
  survit avec le chiffre corrigé : l'orchestrateur résiduel reste
  substantiel, mais c'est une **conséquence directe du plan de découpe
  PLAN-ADAPT ratifié** (préflight §354-360 énumère explicitement les ~18
  blocs d'orchestration + les 2 littéraux nommés `http_state`/`Ok(Self{})`
  comme restant inline) — pas une non-conformité. Note d'information.

## Dimension 6 — Tests + oracle T1 (CONFORME, 1 P3 process)

Module `#[cfg(test)]` de `runtime.rs` byte-identique HEAD↔working (aucun
`+`/`-` du diff ne contient `cfg(test)`/`#[test]`/`#[tokio::test]`/`mod
tests` ; dernier hunk borné avant le module test). 0 `#[cfg(test)]`
ajouté aux sous-fns extraites → count non gonflé. Tests boot §G présents
aux lignes actuelles (`redrive_on_ingest_pins_configured_app_without_restart`
`http.rs:6165` ; `browse_boot_restore_repopulates_aggregator_from_outbox_e2e` ;
`test_feed_republish_at_boot` ; `reannounce_seeds_noop_in_duress`
`feed_sync.rs:954` ; 10× `boot_*_namespace_*`). Compile-net cross-module
intact (call-sites `dispatch_loop.rs`/`http.rs` référencent
`crate::runtime::X` par noms préservés → build vert). Oracle T1
`count == baseline` EXACT satisfait : Win nextest **2099/2099 == baseline**.

- **L-5 (P3, CONFIRMED — process/record)** — Confirmation Docker (attendue
  **2103 == baseline**) **encore EN COURS** au lancement de cette review.
  Structurellement le count Docker est invariant : le diff touche
  UNIQUEMENT `runtime.rs`, dont le module `#[cfg(test)]` est byte-identique,
  et build/clippy/fmt Docker sont déjà verts (2103 = 2099 Win + 4 tests
  `#[cfg(unix)]`). Ce n'est PAS un défaut de code, mais **le commit DOIT
  rester bloqué tant que Docker n'affiche pas 2103 exact** (oracle
  préflight §G = `==`, pas `>=`) ; un flake env Docker-on-Windows (classe
  documentée, 3 flakes solo-PASS au run K) impose le re-run-solo canon
  avant conclusion, exactement comme côté Win/Vitest. Gate assumé par le
  main thread.

## Dimension 7 — Patterns + docs-contrat (CONFORME, 1 P3 doc-nit)

Tous les symboles `pub(crate)` préservés et résolvant à `crate::runtime::X`
(`boot_storage_namespace`/`boot_feed_namespace`/`open_project_doc_for_dispatch`/
`maybe_redrive_seed_on_ingest`/`RedriveCoord`/`REDRIVE_MIN_INTERVAL`/
`docs_migration_backup_path`/`refuse_recreate_on_interrupted_migration`/
`spawn_gossip_subscribe_task`/`load_or_generate_node_key`) → 0 rupture
cross-module, 0 édit doc ligne-ancrée. Docs sécurité (THREAT_MODEL,
PATTERNS §P74, etc.) référencent les symboles par NOM seul → 0 doc édité
(refacto name-preserving). Aucun langage de promesse-future dans les
lignes ajoutées (`grep '^+' will be/S83/next sprint/TODO/FIXME = NONE`).
`frontier_closure = N/A` justifié (0 route, 0 DTO, 0 constante wire ; les
7 sous-fns sont privées ; `ApiServerHandles` type privé ; le seul
`pub(crate)` du diff = `open_project_doc_for_dispatch` RELOCALISÉ, pas une
nouvelle frontière). Invariant pure-refactor `docs/rust/PATTERNS.md:1722`
respecté.

- **L-6 (P3, CONFIRMED — doc-nit)** — Le header de section Phase L
  (`runtime.rs:1046-1048`) affirme « `identity_mode` is always threaded as
  a mandatory argument — never defaulted — so a duress-gated callee cannot
  silently fall back to `Normal` (DURESS-BOOT-LEAK class) ». Re-vérifié
  disque : **AUCUNE des 7 sous-fns Phase L introduites sous ce header ne
  prend `identity_mode`** (signature `boot_node_identity:1071` = `root,
  iroh_data_dir, coordinator_db, seed_nonce_cache` — pas d'`identity_mode` ;
  idem les 6 autres). La seule sous-fn Phase L duress-gatée
  (`boot_feed_recovery`) atteint sa sûreté via le pattern
  `Option<&Arc<FeedSyncState>>` pré-substitué (documenté dans son PROPRE
  doc `:1199-1202`), PAS via `identity_mode`. La doctrine est VRAIE et
  load-bearing pour les boot-namespace fns S81 co-localisées
  (`open_project_doc_for_dispatch`/`boot_storage_namespace`/`boot_feed_namespace`,
  qui elles PRENNENT `identity_mode` en arg obligatoire), mais le header
  nomme un mécanisme (threading `identity_mode`) qu'aucune sous-fn Phase L
  n'utilise. Non-bloquant : 0 changement de comportement, refacto pur
  intact ; précision doc seulement. Fix optionnel : reformuler le header
  pour attribuer l'invariant `identity_mode` aux boot-namespace fns et
  noter que `boot_feed_recovery` utilise la variante Option pré-substituée.

## Dimension 8 — Ressources / handles / async (CONFORME, 1 P3)

Le littéral `Ok(Self{..})` (`runtime.rs:852-876`) thread les 4 handles API
depuis le tuple `spawn_api_server` (alias `ApiServerHandles`), le triplet
`(gossip_handle, gossip_shutdown_tx, boot_driver_handle)` depuis
`spawn_gossip_and_boot_seed_driver`, `tokens_watcher` depuis `wire_auth`,
`pow_policy_watcher: _pow_policy_watcher` (binding underscore NON dropé),
`revocation_cache`, `bound_addr` (SocketAddr Copy). `spawn_peer_listener`
préserve le fallback None-on-bind-failure (UDS/NP échoue → TCP-only,
`peer_handle`/`peer_shutdown` restent `Option`). Aucun `MutexGuard` std
retenu à travers un `.await` : `boot_feed_recovery` — les 2 guards DB en
blocs `{}` clos AVANT chaque `.await` (re-vérifié disque `:1209-1214` /
`:1244-1249`) ; le guard `app_storage` du littéral reste block-scope
inline dans `start()`. Aucun nouveau `unwrap`/`expect`/`panic` dans les
fns extraites (`?`/`.context()`/`.map_err(...)?`). Reorder janitors bénin
(cf. L-2). `tokio::spawn` valide (fns sync appelées depuis `start()`
async).

- **L-7 (P3, CONFIRMED — hygiène de review, non-code)** — Le tableau
  `checked` de cette dimension cite des **numéros de ligne périmés** de
  ~+500 l pour des entités correctes : `seed_driver_lock « :942 »` (réel
  **:1468**), `GossipTaskConfig config « :961 »` (réel **:1487**), closure
  boot driver « :968 » (réel **:1494**), `spawn_api_server « sig
  :824-830 »` (réel def **:1350**). La ligne :942 tombe DANS `shutdown()`,
  pas dans une fn de boot. La SUBSTANCE de la dimension est correcte
  (re-vérifiée indépendamment par le synthétiseur aux VRAIES lignes :
  lock-unique + 2 clones + handles threadés), mais l'évidence chiffrée
  viole la discipline README « finding avec evidence fichier:ligne
  vérifiée » : un relecteur suivant ces refs atterrit sur du code sans
  rapport. Les Dimensions 1-7 et les findings publiés utilisent, eux, les
  bons numéros. Note de qualité de review ; 0 impact code.

---

## Arbitrage explicite — déviation `build_http_state` non-extraite

La tâche demandait de trancher : **LÉGITIME, non-P1.** Le plan de découpe
préflight listait `build_http_state` parmi les 8 cibles MAIS accordait
explicitement la latitude « regrouper autrement tant que A-G tiennent »
(`sprint82_phase_l_preflight.md:339`) et signalait lui-même ce littéral
~30-champs comme LE risque d'extraction (drop-scope + ordre d'évaluation,
§S1a:98-99). J'ai re-vérifié disque que **le littéral est byte-identique
à HEAD** (0 ligne +/- ne touche `app_storage`/`canary_registry` ; le
`DaemonHttpState {` n'apparaît dans aucun hunk changé) — donc l'ordre des
champs ET le drop-scope du bloc `app_storage` (`{ let guard =
coordinator_db.lock().unwrap(); ... }`, std MutexGuard sans await) sont
TRIVIALEMENT préservés. Les 7 contraintes A-G tiennent toutes sans
l'extraction (signature `start()` figée, 0 rapport au couplage
`seed_driver_lock`, littéral local threadé à `Ok(Self)`, jamais un
symbole cross-module ni une ancre doc, count invariant). Le choix est
strictement **plus sûr** que l'extraction : forcer `build_http_state`
aurait imposé ~28 args positionnels (risque de swap silencieux) et le
piège S1a. Rationale de l'implémenteur (littéral nommé > appel
positionnel) cohérent avec l'objectif lisibilité de la phase.

## Table des findings (déduplication inter-dimensions, verdicts adversariaux)

| ID | Sév | Titre | Fichier:ligne | Dim | Verdict |
|---|---|---|---|---|---|
| L-0 | P2 | Scoping commit atomique : stager par chemins explicites (`runtime.rs` + préflight), exclure blueprint + 2 recherches untracked | working tree | pré-commit | CONFIRMED |
| L-1 | P3 | Déviation `build_http_state` non-extraite — ÉVALUÉE LÉGITIME (pas P1) | runtime.rs:742 | 1+3+4+5+8 | CONFIRMED |
| L-2 | P3 | Reorder bénin `boot_driver_state=Arc::clone` avant janitors (forcé borrow-checker, effet nul) | runtime.rs:819 | 2+8 | CONFIRMED |
| L-3 | P3 | `spawn_gossip_and_boot_seed_driver` source 9 deps via `state.*` — invariant same-Arc élargi (0 impact actuel) | runtime.rs:1470-1482 | 3 | CONFIRMED |
| L-4 | P3 | `start()` résiduel ~602 l (contradiction interne 601/761 réconciliée → 602, réel `276→877`) | runtime.rs:877 | 5 | CONFIRMED (761 REFUTED) |
| L-5 | P3 | Gate Docker nextest (2103==baseline) EN COURS — commit gaté main thread | — | 6 | CONFIRMED |
| L-6 | P3 | Section-doc `identity_mode` over-généralise (nomme un mécanisme qu'aucune sous-fn Phase L n'utilise) | runtime.rs:1046-1048 | 7 | CONFIRMED |
| L-7 | P3 | Évidence chiffrée périmée dans le tableau `checked` de Dim 8 (~+500 l off) — substance correcte | runtime.rs (Dim 8) | 8 | CONFIRMED (hygiène review) |

Total confirmés : **0 P0 / 0 P1 / 1 P2 / 7 P3**. 1 chiffre réfuté (le
« 761 l » du scan Dim 5, corrigé à 602 l par re-vérification disque). Le
finding `build_http_state` apparaissait dans 5 dimensions + 3 lentilles
adversariales, toutes CONFIRMED P3-légitime → fondu en L-1.

### Réconciliations / réfutations à la vérification (traçabilité)

- **Taille résiduelle `start()` (601 vs 761) — RÉCONCILIÉE à 602 l.** Deux
  scans du même package se contredisaient ; la lentille PROCESS confirmait
  à tort 761 (conflation avec `shutdown()`), les lentilles DIFF-FACTS et
  SECURITY établissaient 877/602. **Re-vérification disque par le
  synthétiseur** : `start()` ferme au `}` ligne 877 ; ligne 1036 ferme le
  bloc `impl DaemonRuntime`. Chiffre retenu = **602 l** (`276→877`),
  réduction ~37 %. La conclusion (résidu conforme au plan PLAN-ADAPT)
  survit ; seule l'évidence chiffrée du scan Dim 5 était fausse.
- **Nuance honnête sur `boot_feed_recovery` (soulevée par la lentille
  SECURITY).** Les listes `checked` Dim 3/5 présentent le pattern
  `Option<&Arc<FeedSyncState>>` comme « strictement plus sûr qu'inline ».
  C'est vrai que la fn ne peut PAS re-dériver EN INTERNE (aucun accès à
  `feed_sync_state`, re-vérifié disque). Mais le type n'ENFORCE pas la
  substitution : un FUTUR second appelant passant l'état brut sous duress
  ré-ouvrirait DURESS-BOOT-LEAK silencieusement. **Risque actuel = 0**
  (appelant unique correct `:678-690`, doc-warning explicite `:1199-1202`,
  et la substitution `None` était DÉJÀ calculée caller-side même inline) →
  P3 latent replié dans L-3/l'esprit de L-6 ; à nommer honnêtement plutôt
  que « strictement plus sûr » : la sûreté duress est **doc-enforced au
  call-site unique correct**, pas type-enforced comme le pattern
  `identity_mode`-en-arg-obligatoire utilisé ailleurs. Aucune action cette
  phase.
- **Nettoyage doc positif (mineur, non-défaut).** Le bras `None` de
  `boot_node_identity` a supprimé un commentaire HEAD périmé prétendant un
  fallback `create_node()` legacy alors que les 2 bras utilisent
  `create_node_with_protocols` → amélioration d'honnêteté-doc, 0 changement
  comportemental.

## Vérification §7.4 (suites, résultats main thread audités)

- **Rust Windows COMPLET VERT** : `fmt --check` OK ; `clippy --workspace
  --all-targets --locked -D warnings` OK ; **nextest workspace 2099/2099
  == baseline EXACT** (1 flake `convergence_remote_write_visible_to_local_subscriber`
  au run 2 fail-fast, re-joué SOLO PASS + run 3 complet no-fail-fast
  2099/2099 PASS ; run 1 post-refacto déjà 2099/2099) ; doctests OK ;
  release build OK. Le flake vit dans `dispatch_loop.rs:478` — ABSENT du
  diff (`git diff --name-only` = `runtime.rs` seul), **causalité refacto
  EXCLUE**, classe iroh-networked Docker/charge documentée.
- **Web COMPLET VERT** : lint OK (5 warnings pré-existants, 0 erreur) ;
  tsc OK ; Vitest **412/412 au re-run solo** (4 fails au run 1 sous charge
  de 2 builds cargo parallèles — classe `vitest_env_variance` connue, 0
  fichier web touché par la phase → invariant structurel à 412) ; build
  OK ; size 129.02/130 kB OK ; scan-en-strings clean.
- **Docker sbfb-ci** : `fmt` VERT + `clippy` VERT (run séparé) ; **nextest
  workspace `--no-fail-fast` 2103/2103 PASS (9 slow, 0 skip) + doctests
  VERTS** — le gate exigé au lancement de la review est CLOS : **2103 ==
  baseline EXACT** (oracle préflight §G `==`). Le préflight §S1b
  singularise le fmt DUAL-PLATFORM comme LE risque silencieux d'un
  refacto move-heavy (précédent S76-G) ; le `fmt` Docker VERT couvre ce
  risque.

Compteurs FINAUX : **Win 2099/2099 ; Docker 2103/2103 ; Vitest 412/412**.
Delta cumulé **±0** — cohérent avec un refacto pur (0 test ajouté/retiré).

## Codex reconciliation

- Rapport : `sprint82_phase_l_codex_review.md` (GPT-5.6 Sol reasoning
  max, round 1, output brut non réécrit).
- Verdict : **7/7 livrables CONFIRMÉ, 0 GAP, 0 PARTIEL — CLEAN round 1**.
  Vérification indépendante forte : mapping bloc-par-bloc HEAD→working
  tree des 7 extractions (renommages mécaniques seuls), 21 helpers
  déplacés « textuellement identiques », multiset des littéraux
  logs/erreurs strictement identique, 0 token de condition ajouté/retiré,
  0 unwrap/expect/panic nouveau, `git diff --check` propre, module
  `#[cfg(test)]` SHA-256 LF-normalisé IDENTIQUE à HEAD
  (`db620dd5…`), lock unique :1468 → clones :1487/:1494, 11 paires
  ordonnées re-déroulées une à une avec lignes.
- Réconciliation : 0 GAP à corriger — boucle arrêtée round 1 (critère
  « CLEAN ou P2/P3 documentés »). Le P2 L-0 (scoping du staging) est
  appliqué par le main thread au commit (chemins explicites, jamais
  `git add -A`). Correction à la volée L-6 appliquée : commentaire de
  la section sub-functions reformulé (le duress gating passe par les
  valeurs DÉJÀ substituées dans `start()` + les boot fns pré-existantes
  gardent `identity_mode` en argument obligatoire — plus de
  sur-généralisation « always threaded »). Ce header a été promu
  `## Verdict: PASS` après la gate Codex, conformément au canon §4.5.
