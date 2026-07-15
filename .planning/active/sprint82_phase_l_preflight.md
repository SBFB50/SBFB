# Sprint 82 Phase L — Preflight G8 (décomposition DaemonRuntime::start())

## Contexte + méthode (Workflow multi-agents, 2026-07-15)

Phase L est un **refacto PUR** : éclater `DaemonRuntime::start()` (le monolithe boot de
`crates/nexus-shell-daemon/src/runtime.rs`) en sous-fonctions boot nommées `<~150 l`, et
co-localiser les helpers annonce/outbox. **0 wire bump, 0 dep, 0 changement de comportement
ni de séquence de boot observable.** Cas B (pre-code). Couvre `REFACTO-DAEMON-RUNTIME-START`.
Baseline post-K : **Win 2099 / Docker 2103 / Vitest 412**.

Six scans factuels (S0 état-code, S1a prior-art/pattern, S1b deps/CVE, S2 décisions
historiques, S3 threat-model/paires ordonnées, S4 wire) ont été produits puis **vérifiés
adversarialement** (3 lentilles : FACTS/DRIFT, SECURITY, TESTABILITÉ). Le WRITER a **re-vérifié
à la source disque** au HEAD `713f0fa` chaque coordonnée porteuse : bornes de `start()`, taille
fichier, champ-count `GossipTaskConfig`, localisation du hook Phase A, nombre d'appelants,
localisation du test re-drive, les 8 paires ordonnées de sécurité, les noms de symboles
couplés cross-module, et les 2 dérives de pointeur du plan.

**Bilan** : le plan §Phase L est **réalisable sans toucher aucune décision Day-0 ni aucun
invariant de sécurité** (signature `start()` figée, 0 wire, iroh intact, tous les gates
préservables in-place). Mais **7 faits corrigés changent l'approche** (bornes réelles, pattern
de découpe IMPOSÉ par la nature constructeur, surface de couplage A↔L réduite à UN objet,
livrable « regrouper helpers » déjà largement fait, oracle T1 à resserrer, corrections de
comptage de tests). → **PLAN-ADAPT**.

**Dérives plan à consigner (vérifiées disque)** :
1. Bornes `start()` : plan §L l.285 dit `276-1224 (~950 l)` ; le réel est **276-1233 (958 l)**
   (`Ok(Self{...})` = 1208-1232, `}` de la fn = 1233, `bound_addr()` = 1238). Dérive +9 l
   (ajouts S81 A2/A4/C + S82 A DANS `start()`).
2. Taille fichier : plan l.421 dit `runtime.rs 5096 l` ; le réel est **5297 l** (`wc -l`). La
   ligne du tableau est stale de +201 l. **Corollaire : TOUTE coordonnée `runtime.rs:NNN` du
   plan est à re-dériver par NOM de symbole, jamais à suivre littéralement.**
3. Pointeur T20 (plan l.218, item Phase C — hors L) : cite `runtime.rs:2580` pour le
   backup-suffix ; la vraie logique (`docs_migration_backup_path`) est **:2754** (déjà signalé
   périmé par `sprint82_phase_h_preflight.md:79`). Confirme le point 2. Non-bloquant pour L.

---

## S0 — État réel du code (cible confirmée)

- **Fichier** : `crates/nexus-shell-daemon/src/runtime.rs` (**5297 l**). L'autre `**/runtime.rs`
  (`nexus-worker-core/src/engine/runtime.rs`) n'est PAS concerné.
- **`start()`** = `pub async fn start(opts: DaemonStartOptions) -> Result<Self>` **276 → 1233**
  (958 l). C'est un **CONSTRUCTEUR ASSOCIÉ** : il construit et retourne `Ok(Self { … })`
  (1208-1232), il n'y a **pas de `self` à emprunter**. `evidence: runtime.rs:276, :1208-1233, :1238`.
- **Appelants figeant la signature** : **1 prod** (`main.rs:643 DaemonRuntime::start(opts)`) +
  **27 sites de test** dans le module `#[cfg(test)]` (>3072). `evidence: grep DaemonRuntime::start(
  = 27 hits >3072 + main.rs:643`. (**Correction : 27, pas 29** comme l'ont écrit S0/risks.)
- **Refacto NON greenfield** : 3 sous-fonctions boot sont **déjà extraites (S81)** et seulement
  APPELÉES par `start()` :
  - `open_project_doc_for_dispatch` (def **2280**, appel **648**),
  - `boot_storage_namespace` (def **2817**, appel **696**),
  - `boot_feed_namespace` (def **2956**, appel **744**).
  Chacune a ses tests dédiés (`boot_*_namespace_*`, 10 fns 4566-4980).
- **Helpers annonce/outbox « à regrouper » = DÉJÀ des free-functions module-level** (hors
  `start()`, pas du code inline) : `handle_announcement:2005`, `handle_directory_announcement:2183`,
  `handle_project_announcement:2532`, `normalize_outbox_payload:2383`,
  `outbox_entry_is_serveable:2405` (le « serveable »), `prune_stale_outbox:2434`,
  `remint_and_wrap_for_replay:2481`, `announcement_claims_own_node_id:2526`,
  `restore_browse_from_outbox:2716`, `mint_ticket_for_hash*:2325/2343`, `current_replay_addr:2361`.
  → **« regrouper » = co-localiser/réordonner physiquement, VÉRIFIER le groupement — RIEN à
  extraire de `start()`.**
- **Le hook re-drive-on-ingest Phase A vit HORS `start()`** : call-site dans le `select!` de
  `spawn_gossip_subscribe_task` (**1790-1810**), fn `maybe_redrive_seed_on_ingest` **2112**,
  `const REDRIVE_MIN_INTERVAL` **2045**, `struct RedriveCoord` **2054**. `evidence: runtime.rs:1790-1810,
  2045, 2054, 2112`. Décomposer `start()` SEUL **ne touche pas** le hook.
- **`GossipTaskConfig` = 19 champs** (def 1529-1565, destructure 1571-1591). `evidence: comptage
  disque`. (**Correction : 19, pas 23** comme l'a écrit S1a.)

---

## S1a — Prior art / pattern de découpe IMPOSÉ

**Pattern (a) obligatoire — sous-fonctions qui PRENNENT/RENDENT des structs de contexte + clones
`Arc`, terminant sur UN littéral `Self`.** Pattern (b) « méthodes `&mut self` séquentielles »
est **IMPOSSIBLE** : `start()` retourne `Result<Self>`, il n'y a pas de `self` construit à
emprunter. Le repo **applique déjà** ce pattern (précédent à étendre) :
- `struct GossipTaskConfig { 19 champs }` → `spawn_gossip_subscribe_task(cfg)` qui destructure
  `let GossipTaskConfig { .. } = cfg;` (runtime.rs:1529/1570). C'est l'idiome BootContext.
- `boot_storage_namespace` / `boot_feed_namespace` : sous-fns boot qui prennent leurs entrées
  et renvoient `(feed_sync_state, handle)`.
- iroh 1.0.1 lui-même : `Builder::bind` agrège ~15 champs dans `socket::Options {..}` puis
  délègue à `EndpointInner::bind(opts)`.

**Recommandation** : pour chaque phase boot résiduelle, agréger les `Arc`-clones/channels dans
une petite struct de contexte nommée OU passer les paramètres directement, et déléguer à un
`boot_*` / `spawn_*` `<150 l`, en miroir de `boot_storage_namespace`.

**Pièges de mécanique (tous COMPILE-backstopped — échouent au build, pas silencieusement)** :
- **Send bounds** : aucune sous-fn `async` spawnée ne doit RETENIR un `std::sync::MutexGuard`
  (`coordinator_db.lock().unwrap()`) à travers un `.await`, ni le RENVOYER. Le seul guard std
  actuel est block-scope sans await (littéral `DaemonHttpState.app_storage`, ~978-981) → à
  préserver. `seed_driver_lock.lock().await` (1196) est un `tokio::sync::Mutex` (Send-safe).
- **Channels + JoinHandles capturés par `tokio::spawn(async move{})`** (dispatch 664, http 1088,
  validator 1102, gossip 1130, boot-driver 1171) : une sous-fn ne peut pas à la fois SPAWNER et
  renvoyer le `tx`/handle → threader les moitiés `tx` et les `JoinHandle` en RETOUR (tuple ou
  struct `BootWiring`), comme `GossipTaskConfig` aspire ses `rx` en ENTRÉE.
- **Littéral `DaemonHttpState` (~30 champs, champs à expression-bloc inline)** : `build_http_state`
  doit préserver le drop-scope des blocs inline et l'ordre d'évaluation des champs.

**Aucune règle « taille de fonction » citable dans PATTERNS.md** (§ Boot sequence = stub TODO
iroh, sans rapport). La doctrine à invoquer = l'invariant « pure refactor — same wire output,
swappable callsite » (`docs/rust/PATTERNS.md:1722`). Précédents de ton/portée : S54-B gossip
refactor `ed5bbdc`, S29-C broker/executor `6a23ebf`, S70-F agent wrappers.

---

## S1b — Deps / CVE (0 impact)

- **0 dep nouvelle, 0 bump** : `Cargo.toml`/`Cargo.lock` PROPRES (absents de `git status` ;
  seuls 3 `.planning/research` PO dirty, **NON touchés**). Le crate hôte `nexus-shell-daemon`
  déclare déjà toutes les deps de `runtime.rs`. Un découpage de fonctions n'ajoute aucun crate.
- **0 advisory pendant sur le crate hôte** : `deny.toml` post-K ne porte plus que les 2
  `quick-xml` (RUSTSEC-2026-0194/0195, transitif iroh → netwatch → plist, hors chemin L) +
  `[bans] multiple-versions=warn` (carry P2-AUDIT-2-RESIDUEL, orthogonal) + exemption rand.
- **2 pièges toolchain (procédure, pas code)** :
  1. **Incremental Windows / LNK1140** (précédent Phase K) : déplacer ~950 l peut laisser des
     artefacts incrémentaux stale. Si le release-build (`cargo build -p nexus-shell-daemon
     --release`) sort une erreur linker après le move → `cargo clean` puis re-run, **ne pas
     diagnostiquer comme bug code**.
  2. **fmt mass-reformat / précédent S76-G `http.rs:8531`** : décomposer reformate de larges
     spans → `cargo fmt --all --check` à gater sur **Windows ET Docker rust:1.94** (dual-platform),
     un mass-reformat pouvant révéler une violation fmt latente byte-identique sous une seule
     toolchain. Vérifier que le diff fmt ne contient QUE les moves intentionnels.

---

## S2 — Décisions historiques (contraintes de boot à préserver)

Aucune décision historique ne bloque le refacto ; toutes = invariants que le déplacement DOIT
préserver (evidence dans S3 pour les paires ordonnées). Les load-bearing :

1. **Ordre open-coordinator-DB AVANT boot-node** (S74-E `1486fc9`) : le handler protocole seed
   (`SEED_ALPN`) est enregistré sur le Router, qui **n'accepte aucun protocole post-spawn**.
   `coordinator_db` ouvert :336, `seed_nonce_cache` :342, capturés par `seed_protocol_factory`
   dans les DEUX bras d'identité (352-356 / 375-379) passés à `create_node_with_protocols`
   (357 / 380). `evidence: runtime.rs:330-388`. Ne pas réordonner.
2. **`identity_mode` capturé UNE fois (:516) et threadé** dans chaque sous-appel duress-gated :
   `open_project_doc_for_dispatch(648)`, `boot_storage_namespace(696)`, `boot_feed_namespace(744)`,
   `reannounce_seeds_at_boot(892)`, `http_state.identity_mode`, et via `boot_driver_state` vers
   le re-drive + boot driver. **Verrouiller par signature (arg obligatoire) ; jamais de default
   `Normal`** (même type, compiler-silent → défait DURESS-BOOT-LEAK sur ce chemin).
3. **Noms `pub(crate)` figés par des tripwires cross-module** : `boot_storage_namespace`,
   `boot_feed_namespace`, `open_project_doc_for_dispatch` sont appelés PAR NOM depuis
   `dispatch_loop.rs` (**694, 796, 809, 868, 881, 1077**) ; `maybe_redrive_seed_on_ingest` +
   `RedriveCoord` depuis `http.rs` (**6203, 6213, 6237, 6263, 6273, 6287, 6291**). Renommer /
   re-signaturer / enfouir dans un sous-module **sans re-export au chemin `crate::runtime::X`**
   casse le compile (fail-loud, mais 6+ call-sites à réparer).
4. **Discriminateur fail-fast namespace** (S81-A2 `23f3be8`) : `boot_storage_namespace` /
   `boot_feed_namespace` discriminent `Err(NotFound)` → recreate BRUYANT vs tout autre `Err` →
   propagation `?` aux call-sites (648/696/744) → `main.rs` `?` → exit≠0 → crash-loop systemd
   VOULU. Le self-heal legit (recreate sur NotFound, `6ca9702`) doit SURVIVRE au fail-fast.
5. **Garde migration redb interrompue** (S81-F / S82-H) : `refuse_recreate_on_interrupted_migration`
   (:2773) sur le bras `Replica not found` UNIQUEMENT, AVANT `create_doc` ;
   `docs_migration_backup_path` (:2754) dérive de `nexus_core_rs::MIGRATION_BACKUP_SUFFIX` via
   `OsString::push` (chemin **byte-identique, sans séparateur**), figé par le tripwire
   `upstream_migration_backup_suffix_matches_shared_const`. Ne pas rendre le chemin
   non-byte-identique (désarme silencieusement le guard).
6. **Sync-set entry duress-gated** (S81-C `f70fa5f`) : `doc.start_sync(vec![])` reste DANS le
   `match sync_set_entry_in_duress` (Enter vs Skip) dans les 3 boot fns. Test-encodé
   (`dispatch_loop.rs:985/1017/1034`). Un `start_sync` inconditionnel re-dialerait les vrais
   pairs sous clé leurre.
7. **`RedriveCoord` (single-flight + dirty) NON simplifiable** : Codex P1-1/P1-2 REJETÉS —
   ne pas remplacer par un cooldown leading-edge ni `seed_driver_lock.lock()` par `try_lock`-skip
   (perte d'ingest). §P74.

---

## S3 — Threat model : les paires ORDONNÉES à préserver

Toutes les coordonnées ci-dessous ont été **re-lues disque**. Le refacto est faisable sans
déplacer aucun gate, mais l'ordre relatif de ces paires est load-bearing. **Les gates du
`select!` gossip (1..3) vivent dans `spawn_gossip_subscribe_task`, PAS dans `start()`** — donc
décomposer `start()` seul ne les touche pas ; ils ne concernent L que si le refacto éclate AUSSI
la boucle gossip (2ᵉ monolithe ~730 l, hors périmètre littéral « `start()` »).

| # | Sév | Paire ordonnée | Ancre disque |
|---|---|---|---|
| 1 | HIGH | PoW `verify_envelope` (Err⇒`continue`) **AVANT** tout dispatch de payload | `1708-1721` ⇒ `1722+` (boucle gossip) |
| 2 | HIGH | drop self-node_id-spoof **AVANT** `handle_project_announcement` | `1770` (`announcement_claims_own_node_id`) ⇒ `1776` |
| 3 | HIGH | **A↔L** : `maybe_redrive_seed_on_ingest` gaté sur `accepted &&` du retour de `handle_directory_announcement` | `1790-1795` (accepted) ⇒ `1801-1810` |
| 4 | HIGH | substitution duress `feed_sync_for_republish = None` **calculée avant**, et LUE par, les 2 blocs (feed republish + orphan) | `782-789` ⇒ `792` + `824` |
| 5 | HIGH | clamp host `127.0.0.1` **AVANT** `TcpListener::bind` | `398-402` ⇒ `404` |
| 6 | HIGH | `start_sync(vec![])` maintenu DANS `match sync_set_entry_in_duress` (3 sites) | boot fns 2922-2936 etc. |
| 7 | HIGH | `refuse_recreate_on_interrupted_migration` **AVANT** `create_doc` (bras `Replica not found` seul) | `2773` ⇒ create_doc (boot fns) |
| 8 | MED | singleton `check_stale_or_bail` **AVANT** node + bind + `write_running` | `282` ⇒ `357/380` ⇒ `404` ⇒ `420` |
| 9 | MED | coordinator DB + nonce cache **AVANT** `create_node_with_protocols` (handler SEED_ALPN) | `335-342` ⇒ `357/380` |
| 10 | MED | `prune`/`restore`/`repull` **AVANT** `boot_replay_done.send` (débloque le boot driver) | `1630`/`1656`/`1671` ⇒ `1675-1677` |
| 11 | MED | boot driver + re-drive sous le **même** `seed_driver_lock` ; re-annonce directory FIRST | `1129` (lock) / `1190-1198` / re-drive `2148-2149` |

**`reannounce_seeds_at_boot` (887-895)** : correction retenue (S2/S4 vs SECURITY) — le gate
duress est **INTERNE** (`feed_sync.rs:172 gossip_publish_in_duress==Noop ⇒ return`) et
**test-encodé** (`feed_sync.rs:954 reannounce_seeds_noop_in_duress`). Ce n'est PAS la
substitution externe `None` des blocs 6c-5/6c-5b : il consomme `feed_sync_state` brut + reçoit
`identity_mode`. Un découpage ne peut PAS déplacer ce gate off-path ; seul risque résiduel =
passer `IdentityMode::Normal` par erreur (compiler-silent).

**Surfaces boot manquées par la cartographie des scans (re-vérifiées disque)** :
- **`PowPolicyWatcher::spawn` (:538)** retenu comme `_pow_policy_watcher` (binding underscore),
  threadé sur `Self` (:1219). Ressemble à fire-and-forget → **facile à droper** ; le droper
  détruit le watcher hot-reload PoW (silencieux, aucun test n'asserte sa vivacité). Toute sous-fn
  extrayant la région caches/identité (~534-568) doit RENDRE ce handle. `tokens_watcher` (:1218)
  idem.
- **Chokepoint auth** (1017-1031) : `.filter(|t| !t.is_empty())` (:1019) — un `SBFB_AUTH_TOKEN`
  vide ne doit JAMAIS poser un bearer vide (bypass) ; précédence env > rotated(tokens.json) >
  static (1020-1027). `build_router(auth_state)` (:1070) est un backstop compile partiel (pas de
  serve sans auth_state) mais ne rattrape PAS un auth_state vide/inversé.
- **Bypass peer-creds** : `spawn_peer_listener(router.clone())` (:1085) enveloppe le routeur
  UDS/Named-Pipe d'une couche qui **bypasse bearer+Host+Origin** pour les pairs kernel-authentifiés ;
  le listener TCP (:1088) garde le triple-check strict. La couche bypass doit rester scoppée au
  chemin peer/UDS UNIQUEMENT.
- **`RevocationCache` restore** (589-627) AVANT les loops qui la consultent ; retenue sur `Self`
  (:1231). Arm d'erreur = `warn!` (soft-fail préexistant, PAS une régression L) → garder le
  restore AVANT les loops.

**Doc-anchors (résultat actionnable)** : **AUCUN doc de sécurité standing ne référence
`runtime.rs` par LIGNE** — toutes les ancres sont par NOM de fonction (`maybe_redrive_seed_on_ingest`
THREAT v18 + PATTERNS §P74 ; `refuse_recreate_on_interrupted_migration` THREAT §15.4 ;
`spawn_gossip_subscribe_task` PATTERNS rust ; `load_or_generate_node_key` ; les 3 boot fns). Un
refacto **qui PRÉSERVE les noms de symboles ⇒ ZÉRO édit doc ligne-ancrée**. Les RENOMMER
imposerait de rafraîchir THREAT_MODEL §15.3.1 + PATTERNS §P74/rust **dans la MÊME phase**
(dimension review 6bis, clôture docs-contrat). `grep runtime.rs:[0-9] docs/` = seul hit =
`iroh_runtime.rs:208/224` (autre fichier).

---

## S4 — Wire format (0-wire PROUVÉ)

- **`runtime.rs` ne DÉFINIT aucune constante de version wire ni format canonique** — il consomme
  seulement. Census gelé Phase G, **entièrement hors du fichier** : `DOMAIN_*_V1`
  (`canonical.rs`), `*_FORMAT_VERSION` par module (`nexus-core-rs`), `FEED_FORMAT_VERSION`
  (`nexus-coordinator-rs/public_feed.rs:20`). Toutes = 1. `grep DOMAIN_|_VERSION runtime.rs` = 0.
- `start()` ne construit **aucun octet signé inline** : 2 appels-helpers (`reannounce_seeds_at_boot`
  :888 → octets `feed_sync.rs` ; `reannounce_directory_at_boot` :1190 → octets `http.rs:1485`) +
  le hook re-drive (délégué à `http::run_boot_seed_driver`). Le refacto DÉPLACE ces appels sans
  toucher la production d'octets.
- Seul site `to_gossip_bytes` du fichier = `remint_and_wrap_for_replay:2516`, qui consomme la
  sérialisation définie dans `nexus-shell-daemon-core/publish.rs:171` (consommée, pas définie).
- Garde-fous octets/JCS (`publish.rs`, `node_directory.rs`, `seed.rs`, `feed_sync` e2e) = dans
  les crates wire-type, **hors `start()`**, inatteignables par le refacto. **T1 = count nextest
  invariant + fmt + clippy suffisent à prouver 0-wire.** Politique pre-launch (doc-only) intacte.

---

## Vérification adversariale — table des claims

| # | Scan | Claim | Verdict | Correction retenue (source disque fait foi) |
|---|---|---|---|---|
| 1 | S0/S1a/S1b/S2/S3/S4 | `start()` = `276-1224 (~950 l)` | **CORRIGÉ** | Réel **276-1233 (958 l)** ; `bound_addr` :1238. Estimation ~950 tient à 8 l. |
| 2 | S1a | `GossipTaskConfig` = **23** champs | **REFUTED** | **19 champs** (comptés disque 1529-1565). Le point « précédent context-struct » tient. |
| 3 | S0/risks | `start()` = 1 prod + **29** tests | **CORRIGÉ** | **27** sites tests (>3072) + 1 prod (`main.rs:643`). Signature figée = conclusion tient. |
| 4 | S1b/S2 | test `redrive_on_ingest_pins_configured_app_without_restart` dans `runtime.rs` | **REFUTED** | Il est en **`http.rs:6165`**. Un refacto de `runtime.rs` ne le déplace pas ; il le CASSE AU COMPILE si `maybe_redrive_seed_on_ingest`/`RedriveCoord` sont renommés. Couverture **hors-fichier** à conserver dans le count. |
| 5 | S0 | « 4 champs Phase A de `GossipTaskConfig` = points de couplage dans `start()` à threader » | **CORRIGÉ** | **Seul `seed_driver_lock` (:1129)** est un couplage partagé genuine (créé dans `start()`, injecté à `GossipTaskConfig` :1148 ET closure boot driver :1173). `redrive_coord` est construit **inline** au site config (:1149, consommateur unique) ; `boot_driver_state`/`keep_online` = clones one-way. Surface de couplage sur-comptée. |
| 6 | scans | « count nextest = SEUL filet T1 » | **CORRIGÉ** | Double filet : (1) COMPILE-time cross-module (fort, fail-loud : `dispatch_loop.rs` + `http.rs` références par nom) ; (2) count invariant (faible, silencieux, attrape les suppressions). Le « count-seul » s'applique aux gates INLINE de `start()`, pas aux gates dans fns extraites (test-encodés). |
| 7 | plan | Livrable « regrouper helpers annonce/outbox » | **CORRIGÉ** | Déjà LARGEMENT fait — free-functions module-level 2005-2532. « Regrouper » = co-localiser/vérifier, pas extraire. Vrai travail = décomposer `start()`. |
| 8 | plan | `runtime.rs = 5096 l` (l.421) | **REFUTED** | **5297 l**. Toute coordonnée `runtime.rs:NNN` du plan à re-dériver par nom. |
| 9 | plan | T20 backup-suffix `runtime.rs:2580` (l.218) | **CORRIGÉ** | Réel **:2754** (`docs_migration_backup_path`). Item Phase C, hors L ; confirme la staleness systémique. |
| 10 | S3 | node/identité = « 3-way match » | **CORRIGÉ** | **2 bras** (`Some`/`None` de `read_optional_identity_env`, :345-388) ; chaque bras enregistre le factory SEED_ALPN. |
| 11 | S0/S1a/S3/S4 | bornes hors-`start()` (hook, helpers, boot fns), noms de symboles, paires ordonnées, 0-wire, drift docs=nom | **CONFIRMED** | — |

**UNVERIFIABLE / à lever au compile (aucun ne bloque le verdict)** :
- L'ordonnancement A↔L (`boot_replay_done` fire :1675 AVANT `run_boot_seed_driver` :1197) n'est
  couvert par AUCUN test hermétique : le test `http.rs:6165` exerce `maybe_redrive_seed_on_ingest`
  en ISOLATION avec lock/coord fabriqués main, il court-circuite `handle_directory_announcement`
  et ne couvre PAS le partage du `seed_driver_lock` créé :1129 ni l'ordre gossip-spawn→boot-await.
  → **relecture diff côte-à-côte OBLIGATOIRE** si L scinde wiring-gossip et spawn-boot-driver.
- Contrat anti-DoS `accepted==false ⇒ pas de re-drive` non testé au call-site réel de la boucle
  gossip → à vérifier par relecture, pas par test.

---

## Contraintes ORDONNÉES pour coder (checklist implémenteur)

**A. Signature & périmètre (Day-0, ne pas franchir)**
1. `pub async fn start(opts: DaemonStartOptions) -> Result<Self>` **INCHANGÉE** ; sous-fonctions
   **privées/`pub(crate)`** internes. 0 wire, 0 dep, 0 changement de comportement observable.
2. Périmètre littéral = **`start()` 276-1233**. Le hook re-drive (1790-1810), le boot-restore/repull
   (1656/1671) et les gates 1..3 du `select!` sont dans `spawn_gossip_subscribe_task` (>1273) —
   **hors périmètre**. Ne les toucher QUE si le refacto choisit d'éclater aussi la boucle gossip
   (alors le bloc hook 1801-1810 survit **verbatim**, avec ses 5 règles §P74).

**B. Le couplage A↔L (l'unique objet partagé)**
3. **`seed_driver_lock` (créé :1129) est LE seul couplage** : si le refacto scinde « wiring
   gossip » et « spawn boot driver » en sous-fns distinctes, créer le lock **UNE fois dans le
   parent** et passer `Arc::clone` aux DEUX consommateurs (config :1148 + closure :1173).
   Alternative plus sûre : **une seule** sous-fn `spawn_gossip_and_boot_seed_driver(...) ->
   (gossip_handle, boot_driver_handle)` qui possède la création du lock et rend les 2 handles →
   split-brain impossible. **Deux instances = double-emit `SeedAnnounced`** (régression
   silencieuse invisible aux tests hermétiques ; c'est la garantie de `19b92e6`).
4. `redrive_coord` reste construit inline au site config (:1149) ; `boot_driver_state`/
   `keep_online_projects` = clones one-way (pas des couplages).

**C. Ordres à préserver (cf. table S3)** — dans `start()` inline : #4 duress-feed (`None`
calculé :782-789 AVANT lecture :792/:824, JAMAIS re-dériver de `feed_sync_state` brut dans une
sous-fn), #5 host-clamp AVANT bind, #8 singleton AVANT node/bind/running.json, #9 DB AVANT node,
#10 replay AVANT `boot_replay_done.send`, #11 même `seed_driver_lock`. Threader `identity_mode`
(:516) à TOUS les sous-appels duress (arg obligatoire, jamais default `Normal`).

**D. Retours à threader hors des sous-fns** (sinon `Ok(Self{})` 1208-1232 ne compile pas —
backstop) : `http_handle`, `gossip_handle`, `boot_driver_handle`, `peer_handle`, `dispatch_handle`,
`result_sync_handle`, `feed_handle`, `feed_join_handles`, les moitiés `*_shutdown` tx,
`bound_addr`, `revocation_cache`, **`_pow_policy_watcher` (:538/:1219)** et **`tokens_watcher`
(:1218)** (faciles à droper — binding underscore). Ne pas retenir de `std::sync::MutexGuard` à
travers un `.await` dans une sous-fn spawnée (Send).

**E. Noms de symboles (ancres compile + doc)** — PRÉSERVER (ou mettre à jour docs+call-sites
dans la MÊME phase) : `open_project_doc_for_dispatch`, `boot_storage_namespace`,
`boot_feed_namespace` (couplés `dispatch_loop.rs`), `maybe_redrive_seed_on_ingest`, `RedriveCoord`
(couplés `http.rs`), `handle_project_announcement`, `handle_directory_announcement`,
`spawn_gossip_subscribe_task`, `refuse_recreate_on_interrupted_migration`, `docs_migration_backup_path`,
`load_or_generate_node_key`. Si un symbole est enfoui dans un sous-module → **re-export
`pub(crate) use` au chemin `crate::runtime::X`**, sinon 6+ call-sites cross-module cassés.

**F. Docs à mettre à jour in-phase** — **AUCUNE si les noms sont préservés** (ancres par nom, 0
ligne-ancrée sécurité). Si renommage : THREAT_MODEL §15.3.1 + `docs/shell/PATTERNS.md` §P74 +
`docs/rust/PATTERNS.md` (mêmes noms). Clôture docs-contrat = **N/A** (frontier_closure N/A, S4).

**G. Oracle T1 (RESSERRER le plan)** — le plan §L dit `count >= baseline` ; pour un refacto PUR
l'oracle correct est **`count == baseline` EXACT** (Win **2099** / Docker **2103** / Vitest 412).
Le `>=` tolère une hausse qui masquerait une perte de couverture nette. Ne PAS ajouter de
`#[cfg(test)]` aux sous-fns extraites (gonfle le count sans intention). Gates : `cargo fmt --all
--check` (dual-platform) + `cargo clippy --workspace --all-targets --locked -D warnings` +
`cargo nextest run --workspace --locked` == baseline + **relecture diff côte-à-côte des
call-sites gates** (le count ne prouve PAS l'ordre inter-étapes). Vérifier la PRÉSENCE nominale
de : `redrive_on_ingest_pins_configured_app_without_restart` (http.rs:6165),
`browse_boot_restore_repopulates_aggregator_from_outbox_e2e` (4057),
`test_feed_republish_at_boot` (4989), `reannounce_seeds_noop_in_duress` (feed_sync.rs:954),
sibling CONTROL/GREEN (dispatch_loop.rs), `boot_*_namespace_*` (×10). **T2 = N/A** (chemin boot
couvert par tests existants + T2 acceptance boot-SEED Phase A `sprint82_t2_bootseed.json` PASS,
non rejoué en L). **Dual-platform load-bearing** : `sigint_triggers_graceful_shutdown…`
(`e2e.rs:282`, `#[cfg(unix)]`, +4 Docker) est le SEUL observateur boot→sigint→graceful ; invisible
sur Windows. Les 2 vrais nets E2E boot (`e2e.rs:167` running.json+port+/health, `:282` sigint) sont
**timing-fragiles** sous Docker-on-Windows → re-run solo avant de conclure un BLOCK.

---

## Plan de découpe concret (sous-fonctions cibles + blocs sources)

Décomposition **recommandée** (l'implémenteur peut regrouper autrement tant que A-G tiennent).
Les 3 boot fns `open_project_doc_for_dispatch` / `boot_storage_namespace` / `boot_feed_namespace`
sont **déjà extraites** — ne pas re-toucher. Cibles résiduelles de `start()` :

| Sous-fn cible (privée) | Blocs sources | Rend | Contraintes |
|---|---|---|---|
| `boot_node_identity(opts, &coordinator_db, &seed_nonce_cache) -> Result<(Arc<Node>, Arc<KeyPair>)>` | 344-391 (match 2-bras + `create_node_with_protocols`) | `(node, pow_keypair)` | DB déjà ouverte AVANT (§9) ; factory SEED_ALPN dans CHAQUE bras ; `?` fail-fast |
| `bind_api_listener(opts) -> Result<(TcpListener, SocketAddr, String)>` | 393-410 | `(listener, bound_addr, host)` | clamp `127.0.0.1` AVANT bind (#5) |
| `restore_revocation_cache(&coordinator_db) -> Arc<RwLock<RevocationCache>>` | 589-627 | `revocation_cache` | AVANT loops (#RevocationCache) ; soft-fail `warn!` préservé |
| `boot_feed_recovery(feed_sync_for_republish: Option<&FeedSyncState>, &coordinator_db)` | 792-879 (6c-5 + 6c-5b) | `()` | prend l'Option **déjà substituée** (#4) — JAMAIS re-dériver de `feed_sync_state` ; **duress non testé au call-site → assertion review si extrait** |
| `build_http_state(...) -> DaemonHttpState` | 926-998 (littéral) | `Arc<DaemonHttpState>` | drop-scope du bloc `app_storage` (std guard sans await) ; ordre champs |
| `wire_auth(opts) -> (AuthState, Option<TokenRotatorWatcher>)` | 1017-1031 | `(auth_state, tokens_watcher)` | `.filter(!is_empty)` (:1019) + précédence env>rotated>static |
| `spawn_api_server(router, listener, ...) -> (http_handle, peer_handle, peer_shutdown, http_shutdown_tx)` | janitors 1035/1046/1057 + `build_router` 1070 + `spawn_peer_listener` 1085 + `axum::serve` 1088 | handles + tx | bypass peer-creds scoppé peer/UDS SEUL (pas TCP) |
| `spawn_gossip_and_boot_seed_driver(...) -> (gossip_handle, boot_driver_handle)` | `seed_driver_lock` 1129 + `GossipTaskConfig` 1130-1150 + boot driver 1171-1207 | `(gossip_handle, boot_driver_handle)` | **lock créé UNE fois, injecté aux deux (§3)** ; `boot_replay_done` oneshot AVANT driver-await (#10) ; re-annonce directory FIRST (#11) |

Blocs restant inline dans `start()` (petits/orchestration) : singleton 281-325, write running.json
412-420, curator runtime + auto-subscribe 425-465, browse aggregator + resolvers 468-495, gossip
sender slot 496, panic-wipe + `identity_mode` 507-516, PoW policy watcher + caches 534-568 (**rendre
`_pow_policy_watcher`, §D**), trace processor 574, result event channel 633, dispatch spawn 660-664,
result-sync spawn 680-681, storage/feed namespace boot 692-759 (appels aux fns existantes), duress
gate 782-789, reannounce seeds 887-895, FTS5 rebuild 898-917, feed_join channel 919-921, littéral
`Ok(Self{})` 1208-1232.

**Regroupement helpers annonce/outbox (livrable 2, « déjà fait »)** : co-localiser physiquement
`handle_announcement:2005` / `handle_directory_announcement:2183` / `handle_project_announcement:2532`
+ `normalize_outbox_payload:2383` / `outbox_entry_is_serveable:2405` / `prune_stale_outbox:2434` /
`remint_and_wrap_for_replay:2481` / `announcement_claims_own_node_id:2526` /
`restore_browse_from_outbox:2716` en un bloc contigu, **sans changer une ligne de logique**.
Préserver que `remint`/keep_online-gate/`prune` s'appliquent aux **3 sites de replay**
(NeighborUp/browse_request/republish périodique), pas à un seul.

---

## Approche d'implémentation (étapes ordonnées)

1. **Extraire** les sous-fns du tableau ci-dessus une par une, `cargo build -p nexus-shell-daemon`
   après chacune (le compilateur backstoppe les erreurs mécaniques : Send, tx/rx, Self-fields).
2. **Le couplage A↔L en dernier** (spawn_gossip_and_boot_seed_driver) : garder le `seed_driver_lock`
   créé une fois ; relire côte-à-côte l'ordre `boot_replay_done` / re-annonce FIRST / lock.
3. **Co-localiser** les helpers annonce/outbox (0 logique changée).
4. **Vérifier noms de symboles** préservés (grep `crate::runtime::` dans `dispatch_loop.rs` +
   `http.rs` doit rester vert) ; si renommage assumé → re-export `pub(crate) use` + docs in-phase.
5. **T1 dual-platform** : `cargo fmt --all --check` (Win + Docker rust:1.94) ; `cargo clippy
   --workspace --all-targets --locked -- -D warnings` ; `cargo nextest run --workspace --locked`
   **== 2099 Win / 2103 Docker** (exact) ; `cargo build -p nexus-shell-daemon --release`
   (`cargo clean` si LNK1140). Vérifier la présence nominale des tests boot listés (§G).
6. **Relecture diff côte-à-côte** des 11 paires ordonnées (le count ne prouve pas l'ordre
   inter-étapes) — filet obligatoire, pas optionnel.
7. **Commit body** : delta tests cumulé (attendu **±0** ; toute variation = signal), scope cuts,
   invariants préservés (signature figée, 0 wire, hook A↔L, gates duress, noms de symboles).

**Critère T1 machine** : `cargo nextest run --workspace --locked` == 2099 Win / 2103 Docker
**ET** fmt/clippy verts dual-platform **ET** 0 diff de comportement à la relecture côte-à-côte.
T2 = N/A. frontier_closure = **N/A** (prouvé S4).

---

## Verdict: PLAN-ADAPT

Le plan §Phase L (décomposition pure de `DaemonRuntime::start()` + regroupement des helpers
annonce/outbox, hook Phase A préservé) est **réalisable sans violer aucune décision Day-0 ni
aucun invariant de sécurité** : signature `start()` figée, 0 wire (S4 prouvé), 0 dep, iroh intact,
tous les gates duress/PoW/anti-rollback/singleton préservables **in-place**. Mais **7 faits
corrigés changent l'approche** (PLAN-ADAPT, evidence disque pour chacun) :

1. **Bornes réelles `276-1233` (958 l)**, pas `276-1224` ; **fichier 5297 l**, pas 5096 → toute
   coordonnée `runtime.rs:NNN` du plan à re-dériver par NOM de symbole.
2. **Pattern de découpe IMPOSÉ** : `start()` est un constructeur `-> Result<Self>` → sous-fns
   context-struct/`Arc`-clone terminant sur UN littéral `Self` ; méthodes `&mut self` IMPOSSIBLES.
3. **Surface de couplage A↔L réduite à UN objet** : seul `seed_driver_lock` (:1129) est partagé
   (créé parent, injecté à `GossipTaskConfig` :1148 ET boot driver :1173) ; `redrive_coord` est
   inline (:1149), `boot_driver_state`/`keep_online` = clones one-way.
4. **Livrable « regrouper helpers » déjà largement fait** (free-functions 2005-2532) → co-localiser/
   vérifier, PAS extraire de `start()` ; le vrai travail = décomposer `start()`.
5. **Oracle T1 resserré à `count == baseline` EXACT** (2099/2103), pas `>=` ; + relecture diff
   côte-à-côte des 11 paires ordonnées (le count ne prouve pas l'ordre) ; dual-platform
   load-bearing (`sigint…` `#[cfg(unix)]`).
6. **Corrections de comptage** : 27 sites tests (pas 29), `GossipTaskConfig` = 19 champs (pas 23),
   test re-drive en `http.rs:6165` (couverture hors-fichier à conserver dans le count).
7. **Doc-anchors par NOM, pas par ligne** : refacto name-preserving ⇒ 0 édit doc ; renommage ⇒
   THREAT §15.3.1 + PATTERNS §P74/rust dans la même phase + re-export `pub(crate) use`.

Angle mort explicite (à couvrir par relecture, pas par test) : l'ordonnancement A↔L
(`boot_replay_done` :1675 → `run_boot_seed_driver` :1197 sous `seed_driver_lock`) et le contrat
anti-DoS `accepted==false ⇒ pas de re-drive` ne sont couverts par AUCUN test hermétique.
