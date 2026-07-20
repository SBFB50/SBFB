# Sprint 83 Phase E — Preflight G8

**Phase** : E — Spike durable BLOQUANT (GO/NO-GO), crate NEUF `crates/sbfb-workflow-engine`.
**Méthode** : 5 scans factuels (S1a prior-art durable / S1b chaîne d'approvisionnement /
S2 décisions gelées traversées / S3 couverture THREAT_MODEL / S4 invariants wire), chacun
suivi d'une réfutation adversariale, puis critique de complétude, puis cette synthèse.
Vérifications propres au synthétiseur rejouées à HEAD `e8180b8` (census, pragmas
coordinator, exit code nextest zéro-match, jeton T1 README).
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux : S1a=PLAN-ADAPT (2 BLOCKER
dont 1 réfuté au fond, 1 déclassé), S1b=EXECUTE (0 dep neuve confirmé), S2=EXECUTE sur le
gel D3 mais PLAN-ADAPT sur les tripwires, S3=PLAN-ADAPT (report THREAT_MODEL à formuler),
S4=EXECUTE (0 bump wire faisable sous 3 conditions binaires) ; critique de
complétude=PLAN-ADAPT (le harnais de test lui-même n'avait été scanné par personne).

---

## 1. Rationale du verdict

Aucune décision Day-0 ni aucun gel D3 n'est en tension avec le plan E. Vérifié sur disque :

- **Famille moteur GELÉE** = mémoisation-de-steps + journal append-only, renoncement
  EXPLICITE au replay-total déterministe (kickoff §4.3 D3 « Retenu » point 1). Temporal,
  WASM/Obelisk, cqrs-es/sqlite-es et redb comme run store sont REJETÉS (kickoff §4.3
  « Alternatives REJETEES »). Le plan E n'en re-ouvre aucune.
- **0 dépendance runtime neuve** : `rusqlite = { version = "0.36", features = ["bundled"] }`
  (Cargo.toml:136), `rusqlite_migration = "2.2"` (:137), `tempfile = "3.13"` (:337),
  `serial_test = "3.4"` (:345), `blake3` (:70) sont déjà `[workspace.dependencies]` et
  résolus au lock (rusqlite 0.36.0, un seul nœud ; libsqlite3-sys 0.34.0, un seul nœud).
  Le crate engine n'ajoute qu'un nœud : lui-même.
- **0 bump wire faisable** : census `DOMAIN_*_V<n>` **= 25** re-mesuré ce jour par la
  commande exacte du gate à HEAD `e8180b8` (`scripts/check-frontier-contracts.sh:190`
  `DOMAIN_CENSUS_FROZEN=25`) ; registre `CONTRACT_NAMES: [&str; 7]`
  (`crates/sbfb-workflow-core/src/contracts.rs:133`) et golden `contracts.schema.json`
  restent intacts si le spike ne touche pas `contracts.rs`.
- **`crates/sbfb-workflow-engine` est un greenfield réel** : absent de `crates/` et des
  `members` (Cargo.toml:3-18, `sbfb-workflow-core` en :17). Sa charte est écrite :
  « hôte-neutre … AUCUN couplage UI » (kickoff §4.3 D3 point 2), et
  `crates/sbfb-workflow-core/src/lib.rs:6-10` promet noir sur blanc que reducer/scheduler/
  event store vivent dans l'engine et que core « links no Tokio, no network stack and no
  database ».
- **`RUN_EVENT_FORMAT_VERSION` n'existe pas dans le code** (grep `crates/ scripts/ docs/`
  → 0 hit). L'amorcer à 1 est un AJOUT conforme au pre-launch protocol, pas un bump ;
  aucun gate ne recense la famille `*_FORMAT_VERSION`.
- **T1 `N-A-no-frontend-change` est un jeton canonique** du gate de testabilité
  (`docs/claude/README.md:629`) — la Phase E est fermable de ce côté. Angle clos.

**Pourquoi PLAN-ADAPT et pas EXECUTE** : le texte du plan E.3/E.4 est, tel qu'écrit,
**non falsifiable** et **mécaniquement piégé par le harnais de test du repo**. Trois faits
mesurés le prouvent :

1. `.config/nextest.toml:64` `[profile.ci] retries = 1` s'applique à **chaque** test (le
   commentaire du fichier le dit explicitement) et CI l'utilise
   (`.github/workflows/rust-ci.yml:156`). Un `kill_restart_replay_no_double_effect` qui
   produirait un double effet une fois sur deux serait **relancé et vert** : le GO serait
   fondé sur une propriété non tenue.
2. `cargo nextest run -p <crate> <filtre-sans-match>` sort **EXIT=4**, vérifié ce jour
   (`cargo nextest run -p sbfb-workflow-core zzz_no_such_test_xyz --locked` →
   « error: no tests to run », EXIT=4, cargo-nextest 0.9.132). Or la commande
   d'enforcement de `G-STORE` (gate_matrix ligne 12) est
   `cargo nextest run -p sbfb-workflow-engine store` et **aucun** des 4 noms de tests de
   E.3 ne contient `store` → le gate que E prétend OUVRIR sortirait ROUGE. Symétriquement
   `kill_restart_replay_no_double_effect` contient `effect` et ferait matcher
   prématurément la commande de `G-EFFECT` (gate_matrix ligne 5, propriétaire **K**) —
   faux positif de gate créé par E et consommé en K.
3. « contrôle rouge prouvé » + « exit 0 » est logiquement incompatible tant que le contrôle
   rouge n'est pas une **assertion intra-suite**. Le repo a déjà le patron exact :
   `crates/sbfb-workflow-core/src/factory_profile.rs:805-849`
   (`profile_stale_on_source_digest_mismatch`, « The Ok-on-unchanged arm is the red
   control: it proves the mismatch arm is not vacuously failing »).

## 2. Adaptations obligatoires (approche corrigée que le code suit)

1. **Enregistrement du crate** — ajouter `"crates/sbfb-workflow-engine",` à Cargo.toml
   (après :17) et **committer le `Cargo.lock` régénéré dans le MÊME commit** (toutes les
   commandes du projet sont `--locked`). `Cargo.toml` du crate : gabarit
   `crates/sbfb-workflow-core/Cargo.toml` (`version/edition/rust-version/license/authors/
   repository` en `.workspace = true`), deps en `{ workspace = true }` UNIQUEMENT
   (`rusqlite`, `rusqlite_migration`, `serde`, `serde_json`, `blake3`, `thiserror`,
   `tracing`), dev-deps `tempfile` / `serial_test` en `{ workspace = true }`.
   **NE PAS** déclarer `thiserror = "2"` en local (le workspace pointe 1.0). **NE PAS**
   bumper rusqlite 0.36→0.40 (QPO-3 ; et le bump est doublement bloqué : tor-dirmgr 0.41.0
   exige `>=0.36.0,<0.39.0`, rusqlite_migration 2.2.0 exige `0.36.0` caret ⇒ la phrase du
   kickoff « fermable par bump 0.40.1 SOUS RESERVE arti-client » est inexacte).
2. **SPDX sur CHAQUE `.rs` neuf** : `SPDX-License-Identifier: AGPL-3.0-or-later` dans les
   5 premières lignes (`scripts/check-spdx.sh:19,25,31` — scan `find crates -name '*.rs'`).
   Précédent : `crates/sbfb-workflow-core/src/contracts.rs:1`.
3. **Override nextest anti-flake, committé dans le même commit** — ajouter à
   `.config/nextest.toml` :
   `[[profile.ci.overrides]] filter = 'package(sbfb-workflow-engine)'` → `retries = 0`
   (forme du précédent `binary(multi_daemon)` `.config/nextest.toml:85-88`). Un crash-test
   n'est **jamais** retenté ; un flake = un **NO-GO**. Sans cet override le verdict GO est
   structurellement non fiable.
4. **Nommage contraint par la gate matrix (BLOQUANT)** — les tests du spike vivent dans un
   module dont le chemin contient `store` (ex. `engine::store::spike::tests::*`) pour que
   `cargo nextest run -p sbfb-workflow-engine store` (row 12, `G-STORE`) matche ≥ 1 test au
   lieu de sortir EXIT=4. Et **aucun** test de E ne doit contenir la sous-chaîne `effect`
   hors de ce module : renommer `kill_restart_replay_no_double_effect` →
   `kill_restart_replay_no_duplicate_side_output` (ou équivalent sans `effect`), sinon la
   commande de `G-EFFECT` (row 5, phase K) matche prématurément. Le nom du test cité par
   le plan E.3 est donc **adapté**, avec traçabilité dans le body.
5. **Durabilité : `synchronous = FULL` + test de contrat de pragma** — le run store du
   spike ouvre avec `journal_mode=WAL` + `synchronous=FULL` + `foreign_keys=ON` +
   `busy_timeout(5s)`, clone du précédent `crates/nexus-coordinator-rs/src/db.rs:361-370`,
   et un test clone de `db.rs:2178-2188` assert `synchronous == 2`. Le repo a DEUX
   politiques divergentes (worker-core `allowlist.rs` = NORMAL) : le spike choisit
   explicitement FULL, la classe « journal durable » l'exige. La doc SQLite est claire :
   les transactions sont durables face aux crashs d'APPLICATION quel que soit
   `synchronous` ; seule la perte d'alimentation discrimine ⇒ **un harness de crash ne
   prouve JAMAIS fsync**, la durabilité disque se prouve pauvrement par le contrat de
   pragma. À écrire tel quel dans l'artefact (`"durability_claim": "logic-only"`).
6. **Store on-disk obligatoire pour le harness** — le harness s'exécute sur un fichier
   sous `tempfile::TempDir`, JAMAIS sur `:memory:` (un store in-memory n'a ni WAL ni fsync
   ni fichiers `-wal`/`-shm` à recouvrer : le test serait vide de sens). Corollaire
   TEST-ISOLATION (carry fermé S83 A1) : **zéro littéral de chemin**, zéro lecture d'env
   (`NEXUS_GRID_ROOT`/`SBFB_HOME`) dans le crate — le store n'ouvre QUE un chemin injecté
   par l'appelant (pattern des 4 ouvertures SQLite de prod du repo :
   `nexus-coordinator-rs/src/db.rs:362`, `nexus-shell-daemon-core/src/trust_cache.rs:44`,
   `nexus-worker-core/src/allowlist.rs:259`, `nexus-worker-core/src/gpu/profile.rs:331`).
   Un crate « hôte-neutre » ne résout pas de home dir.
7. **`crash_model` DÉCLARÉ** — l'artefact déclare `"crash_model": "in-process"` ou
   `"process-kill"`, et si `in-process`, énumère ce qui n'est **pas** prouvé (mort du
   writer, recouvrement `-wal`/`-shm`, sémantique `TerminateProcess` Windows). Note
   d'outillage : un `std::process::abort()` dans un test nextest EST un kill dur portable
   Win/Linux sans dérouler la pile (nextest isole chaque test dans son propre processus) —
   c'est la voie process-kill la moins chère, elle évite un re-exec `CARGO_BIN_EXE_*` qui
   forcerait une cible `[[bin]]` sur un crate lib-only et évite de tirer Tokio (le
   précédent `nexus-test-harness` est Tokio, incompatible avec la charte hôte-neutre).
8. **Hash de requête de step : `serde_jcs`, jamais `serde_json::to_vec`** — canonicaliser
   avant blake3. Raison **corrigée** (cf. §3) : la feature `serde_json/preserve_order` est
   activée par `ollama-rs 0.3.4` via `schemars`, donc présente dans le graphe
   `--workspace` mais ABSENTE du graphe `-p sbfb-workflow-engine` ⇒ un hash sur
   `serde_json::to_vec` serait stable en `-p` et pourrait diverger en `--workspace`. La
   struct de requête interdit structurellement les collections non ordonnées
   (`BTreeMap`/`BTreeSet` uniquement — JCS trie les clés d'objet, JAMAIS les tableaux) et
   proscrit les floats libres (`serde_jcs` renvoie `Err` sur tout float non fini, jamais
   un panic). Ne PAS activer `arbitrary_precision`. Aucune valeur d'environnement
   (horloge, random, chemin absolu, pid) n'entre dans la requête hashée.
9. **0 `const DOMAIN_*_V<n>`, 0 tag `// FRONTIER:`** — le journal du spike est local,
   non signé, non gossipé. Un tag FRONTIER exigerait mécaniquement un `const DOMAIN` +
   un `const VERSION` sous `crates/` (`check-frontier-contracts.sh:158-163`) ⇒ census 26 ⇒
   gate rouge. **Interdiction explicite** de l'échappatoire « réutiliser un `DOMAIN_*`
   existant » : le gate dédoublonne par `sort -u` et laisserait passer, mais réutiliser un
   domaine de signature pour une autre sémantique viole la doctrine de
   `crates/nexus-core-rs/src/canonical.rs`. Le domaine du pack et `DOMAIN_RUN_EVENT_V1`
   sont routés Phase F (plan:585-587). L'étiquette AM-3 est satisfaite en E comme en B/D :
   commentaire de provenance en PROSE + golden strict.
10. **`RunEvent` : `contracts.rs` INTACT** — le stub `RunEvent.v1`
    (`crates/sbfb-workflow-core/src/contracts.rs:64-71`) dit lui-même « envelope authored
    in the run-store phase » = **Phase I** (plan:728-742). Phase E amorce la CONSTANTE
    `pub const RUN_EVENT_FORMAT_VERSION: u16 = 1;` **dans le crate engine** et livre des
    **fixtures golden events locales** (`RunEvent v0` du plan E.2 = fixture pré-gel,
    cohérent avec E.1 « AUCUN format d'exécution gelé avant ce GO »). Ne PAS remplir le
    stub, ne PAS toucher `contracts.schema.json` (sinon `schema_snapshot_stub` rougit et
    `len == 7` devient un risque inutile). Aucun renommage de type imposé.
11. **Goldens STRICTS** — les fixtures golden events copient
    `factory_profile.rs:770-803` (golden manquant/vide = HARD FAIL, seul
    `UPDATE_SNAPSHOTS=1` écrit), **PAS** le `schema_snapshot_stub` fail-open de
    `contracts.rs:184-198` (self-write si absent → supprimer le golden l'auto-guérit).
12. **Périmètre de garantie honnête (remplace « 0 double effet aux 3 frontières »)** —
    > **CORRECTION (rounds 2/3 de la vérification croisée)** : la formulation proposée
    > ici — « au plus UNE tentative non confirmée par clé » — est **FAUSSE**. Après un
    > crash en B2 l'effet a déjà été tenté et chaque reprise rappelle la cible ; ce qui
    > est borné est l'**intention** et la stabilité de la **clé**, pas le nombre de
    > tentatives. La clause définitive est dans `sprint83_e_durable_spike.json
    > §guarantee`. Le texte d'origine est conservé ci-dessous pour la trace.
    la
    frontière B2 (effet exécuté / résultat non journalisé) est irréductible sans idempotence
    côté cible. Deux clauses : (i) l'engine garantit **au plus une tentative non confirmée**
    par clé `(run_id, node_instance_id, effect_id)` ; (ii) le 0-double-effet est prouvé
    **contre une cible qui déduplique sur cette clé** — l'echo-target IMPLÉMENTE cette
    dédup, et le bras rouge `dedup_effects=false` prouve que la garantie vient bien de là.
    Le `outcome_unknown` (kickoff §4.3 D3 point 4 : « effet non idempotent sans reçu →
    `outcome_unknown`, JAMAIS de retry auto ni de succès inventé ») est la sortie nommée de
    B2 ; en E il suffit de **nommer l'état terminal** et de garantir 0 retry auto —
    l'implémentation complète des effets est routée K.
13. **État terminal diagnosable sur divergence de hash** — `replay_diverges_fails_closed`
    renvoie un `Err(StepHashMismatch { expected, got, node_instance_id })` **typé**, jamais
    un panic/unwrap (précédent S81 K-2 : « rejet loud, jamais de panic »), et le run finit
    dans un état TERMINAL, jamais auto-rejoué (précédent CARRY-2 S75
    `reject_result_on_guardrail_trip`). Le redémarrage après crash ne doit JAMAIS re-créer
    silencieusement un état vide (classe de bug S81-F).
14. **THREAT_MODEL : report EXPLICITE, pas un silence** — v21 (tip `8252a45`), 18 sections,
    assets A1..A9 (`THREAT_MODEL.md:55-66`) ; aucun asset ne couvre un journal d'exécution
    durable local (A7 = « iroh-docs + hash-chain »). Le plan E ne liste aucun livrable
    THREAT_MODEL et route rétention/GC/backup → I (plan:743), durcissement → AH (:1935),
    `THREAT_MODEL vNN` → AJ1 (:2119). Précédent Phase D : 0 diff THREAT_MODEL. **Donc :
    pas de §19 en E, mais une ligne de SCOPE CUT datée dans le body** (« surface durable =
    §19 ouverte en Phase I : store complet + routes ; E est hermétique, 0 route, 0 réseau,
    0 lecteur hors du Rust du crate »). Le raisonnement de frontière §6.12 doit être écrit
    par le **test-acteur** (`README.md:2100-2103`), jamais par « 0 wire bump ».
15. **Posture données : exclusion PAR LE TYPE** — le journal survit au process : aucun
    secret matérialisé (seulement `secret_ref` opaque, kickoff:295-298), et le type de
    l'entrée de journal ne PEUT PAS porter de PII (champ absent, pas `#[serde(skip)]` —
    doctrine A9, `THREAT_MODEL.md:1683`). Coût nul maintenant, cher après le gel.
16. **Bornes / rétention** — le seul outbox persistant du repo est NON borné
    (`nexus-coordinator-rs/src/db.rs:701-715`, purge = `DELETE` global, 0 job de
    rétention). Ne pas répliquer la dette silencieusement : le body inscrit
    « bornes/rétention/GC = Phase I (G-STORE) » en scope cut.
17. **E ne FERME aucun gate** — il OUVRE `G-AUTHORITY` (row 3, fermé J) et `G-STORE`
    (row 12, fermé I). `WA-T1`/`WA-T5` restent `NOT-RUN` jusqu'à AI2 (evidence =
    `sprint83_t2_acceptance.json`, gouverné par `sprint83_t2_spec.md` dont l'enum
    `written_at_phase` = `{AI2, AJ, AJ1}` — E en est exclue). Le vocabulaire de l'artefact
    de E est donc **`GO`/`NO-GO`** (plan E.4), PAS `PASS`/`BLOCK{}`/`RIG-ABSENT`.

## 3. Faits réfutés par l'adversarial (à ne PAS sur-corriger)

- **« `serde_json/preserve_order` est activé workspace-wide via `schemars` »** → **RÉFUTÉ**.
  La source est `ollama-rs 0.3.4` (schemars n'est qu'un relais), et l'unification de
  features est par GRAPHE DE BUILD : `cargo tree -p sbfb-workflow-core -e features -i
  serde_json` → 0 occurrence de `preserve_order`. La panne réelle n'est pas un désordre
  d'insertion généralisé mais une **divergence de hash entre `-p` et `--workspace`**. La
  conclusion (utiliser `serde_jcs`) reste juste ; la justification change, et le test de
  stabilité doit être joué **sous les deux graphes**.
- **« serde_jcs trie-t-il vraiment un `Value` construit avec preserve_order ? — Not
  evidenced »** → **RÉFUTÉ par lecture de source** : `serde_jcs-0.2.0/src/lib.rs:141`
  bufferise les membres de chaque objet dans un `BTreeMap<Utf16Key, Vec<u8>>` émis trié
  (`:446`). Le tri a lieu à la sérialisation, indépendamment du type de map source. Le
  test reste utile comme garde-fou, pas comme preuve manquante.
- **« Comportement de serde_jcs sur NaN/Infinity : Not evidenced »** → **RÉFUTÉ** :
  `serde_jcs-0.2.0/src/lib.rs:82-85` + `:604-619` → `Err(io::Error "invalid float value")`,
  jamais un panic, jamais une valeur silencieuse.

  > **CORRECTION (phase E, mesurée) — cette réfutation est elle-même FAUSSE en
  > généralité.** Trois chemins mesurés sur `serde_jcs 0.2.0` : un float non fini
  > **scalaire nu** renvoie bien `Err` (la lecture de source ci-dessus est exacte pour
  > CE cas), mais **imbriqué dans une struct** — le seul cas que le crate hash — `NaN`,
  > `+Inf` et `-Inf` s'effondrent tous sur `null` **sans erreur**, avec ou sans passage
  > par `serde_json::to_value`. La collision de digest est donc réelle et la garde
  > `reject_nulls` est nécessaire. Trace conservée : une lecture de code ne remplace
  > pas une mesure.
- **« Le critère 0-double-effet est un BLOCKER de design »** → **DÉCLASSÉ en ADAPT** : la
  clé d'idempotence et `outcome_unknown` sont déjà GELÉS au kickoff §4.3 D3 points 3 et 4.
  C'est un défaut de **formulation d'E.4** (adaptation §2.12), pas de conception.
- **« Un `-p <crate> <filtre>` sans match sort VERT à vide »** → **RÉFUTÉ, mesuré** :
  EXIT=4 (« error: no tests to run »). Le risque est inversé : `G-STORE` sortirait ROUGE.
  Les deux scans s'étaient trompés, l'un par omission, l'autre par prédiction non testée.
- **« Le report THREAT_MODEL viole §18.3 »** → **NUANCÉ** : la cadence de §18 est le
  SPRINT, pas la phase (`THREAT_MODEL.md:1710-1712` point 5 : « En fin de sprint, le
  verification.md du sprint pointe vers les lignes modifiées ici »). L'obligation en E se
  réduit à une ligne de scope cut, pas à une §19.
- **« Le harnais out-of-process existe déjà (store_migration.rs) »** → **RÉFUTÉ** : le seul
  `process::Command` de `store_migration.rs` est un appel à `tar` (:390). Le seul vrai
  re-exec est `process_cli.rs:6` via `env!("CARGO_BIN_EXE_sbfb-factory")`, macro qui exige
  une cible `[[bin]]` — que l'engine lib-only n'aura pas.
- **« `extract_matches_golden` (staleness des 6 sources) va rougir parce que E ouvre
  G-AUTHORITY/G-STORE »** → **LARGEMENT RÉFUTÉ** : `docs/factory/FACTORY_GATES.md` ne
  contient aucune occurrence de `G-AUTHORITY`/`G-STORE`/`G-EFFECT` (c'est un document
  FG0..FG10). Le risque n'existe que si E édite `PROCESS.md`/`AGENT_SYSTEM.md`
  (`factory_profile.rs:150-159`) — faible, à ne pas dramatiser, mais à ne pas déclencher
  gratuitement.
- **« PROMISE_RE mord sur "Phase F ajoutera…" »** → **RÉFUTÉ** : la seule forme française
  de `PROMISE_RE` (`check-frontier-contracts.sh:89`) est
  `arrive(ra|nt|ront)? en Phase`. Le vrai risque est **anglais** (`Phase [A-Z0-9]+
  (will|adds|ships)`, `lands in Phase`, `inert until Phase`) — or les doc-comments Rust de
  ce repo sont en anglais. Et le scan couvre aussi `*.toml` (:126-128) : le `Cargo.toml`
  neuf est dans le champ.
- **« "AUCUN moteur OSS majeur ne hashe les entrées de step" »** → **NON PROUVÉ** (négation
  universelle appuyée sur 4-5 sources, non rejouée). À écrire dans le body comme « non
  constaté dans les moteurs examinés (kickoff §4.3 D3) », JAMAIS comme un fait universel.

## 4. Invariants à tenir (commande de vérif par invariant)

- census `DOMAIN_*_V<n> == 25` : `bash scripts/check-frontier-contracts.sh` exit 0
  (re-mesuré ce jour : 25).
- 0 `DOMAIN` neuf : `grep -rn 'const DOMAIN_' crates/sbfb-workflow-engine/` → vide.
- 0 tag FRONTIER neuf : `grep -rn '// FRONTIER:' crates/sbfb-workflow-engine/` → vide.
- registre contrats intact : `git diff --name-only | grep -E 'sbfb-workflow-core/src/contracts'`
  → vide ; `cargo nextest run -p sbfb-workflow-core --locked` exit 0.
- SPDX : `bash scripts/check-spdx.sh` exit 0.
- 0 dep runtime neuve : `git diff Cargo.lock` n'ajoute QUE le paquet
  `sbfb-workflow-engine` (aucune version d'un paquet existant ne bouge).
- lock committé : `cargo nextest run --workspace --locked` exit 0 (échoue si le lock n'est
  pas à jour).
- anti-flake : `grep -n 'sbfb-workflow-engine' .config/nextest.toml` → override
  `retries = 0` présent.
- gate `G-STORE` non-vacuous : `cargo nextest run -p sbfb-workflow-engine store --locked`
  → **≥ 1 test exécuté** (EXIT=4 = rouge, vérifié : zéro-match sort 4).
- gate `G-EFFECT` non pollué : `cargo nextest run -p sbfb-workflow-engine effect --locked`
  → **EXIT=4** attendu en Phase E (aucun test de E ne doit matcher).
- pragma durable : test `run_store_synchronous_full` assert `synchronous == 2` et
  `journal_mode == "wal"`.
- TEST-ISOLATION : `grep -rnE '"[A-Za-z]:\\\\|/home/|/Users/' crates/sbfb-workflow-engine/`
  → vide ; `grep -rn 'env::var' crates/sbfb-workflow-engine/src/` → vide.
- docs-contrat intact : `bash scripts/check-factory-docs.sh` exit 0 ;
  `git diff --name-only | grep -E '^docs/factory/|llms.txt'` → vide.
- T1 : `git diff --name-only | grep -E '^web/'` → vide (`N-A-no-frontend-change`,
  jeton canonique README:629).
- gate dual-platform : `cargo nextest run --workspace --locked` exit 0 Win natif ET Docker
  `sbfb-ci` ; `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked
  -- -D warnings`, `cargo build -p nexus-shell-daemon --release`.

## 5. Plan d'implémentation validé

### 5.1 Ordre des edits

1. `Cargo.toml` : `members += "crates/sbfb-workflow-engine"` ; `cargo check` pour
   régénérer `Cargo.lock` (à committer).
2. `crates/sbfb-workflow-engine/Cargo.toml` (gabarit workflow-core, deps `workspace = true`).
3. `crates/sbfb-workflow-engine/src/lib.rs` — SPDX ligne 1 ; module-doc **au présent/passé**
   (jamais « Phase F will/ships/freezes ») ; `pub const RUN_EVENT_FORMAT_VERSION: u16 = 1;`
   ; charte du crate (hôte-neutre, 0 env, 0 chemin résolu localement).
4. `src/event.rs` — enveloppe locale du spike (`RunEvent v0` fixture) : enveloppe versionnée
   UNE fois + payload `serde_json::Value` extensible ; `BTreeMap` partout ; 0 secret/PII par
   le TYPE.
5. `src/hash.rs` — `step_request_hash()` = blake3 sur `serde_jcs::to_vec` ; préfixe de
   domaine **local non nommé `DOMAIN_*_V<n>`**.
6. `src/store.rs` — ouverture SQLite sur chemin INJECTÉ, pragmas WAL/FULL/foreign_keys/
   busy_timeout ; `MIGRATIONS: &[M<'static>]` statique commenté par sprint/phase, nommage
   des tables ALIGNÉ sur Phase I (`runs`, `run_events`, `outbox`) pour que I étende sans
   DROP ; PK `(run_id, sequence)` ; requêtes **paramétrées** exclusivement (doctrine D3
   point 6, réponse à la chaîne CVE LangGraph ; SQLite embarqué = 3.49.2, CVE-2025-6965
   présente et non fermable ce sprint → à disclose dans l'artefact).
7. `src/txn.rs` (ou dans `store.rs`) — transaction unique
   `execute_batch("BEGIN IMMEDIATE")` + COMMIT/ROLLBACK autour d'un `_inner`, seul pattern
   de transaction explicite du repo (`nexus-coordinator-rs/src/public_feed.rs:439-453`,
   `:484-497`). Un écart (RAII `Connection::transaction()`) doit être justifié.
8. `src/effect.rs` — clé d'idempotence `(run_id, node_instance_id, effect_id)` **DÉRIVÉE
   côté moteur** (`effect_id` n'est jamais une donnée d'un pack ; en E il n'y a pas de pack,
   mais le type posé contraint K) ; état `outcome_unknown` nommé ; 0 retry auto.
9. `src/echo.rs` — echo-target déterministe, qui **implémente la dédup** sur la clé
   d'idempotence (sinon §2.12 (ii) n'est pas prouvable).
10. `src/reducer.rs` — reducer minimal, court-circuit des steps déjà journalisés indexé sur
    `node_instance_id` (JAMAIS un compteur ordinal — l'écueil nommé par le PoC Morling) ;
    snapshot/frontier en mémoire = cache JETABLE, invalidé (rechargé depuis le journal) dès
    qu'un step échoue ou qu'une divergence est détectée (précédent Temporal sticky-cache).
11. `src/store/spike/tests.rs` (module contenant `store`) — knobs test-only + les 4 tests
    à deux bras (§5.2) + le test de pragma + le test de validation d'artefact.
12. `.config/nextest.toml` — override `retries = 0` pour `package(sbfb-workflow-engine)`.
13. `.planning/active/sprint83_e_durable_spike.json` (§5.3).

### 5.2 Tests à écrire — chaque garantie a son contrôle rouge INTRA-SUITE

Modèle imposé : `factory_profile.rs:805-849` (« the Ok-on-unchanged arm is the red
control »). Interrupteurs `#[cfg(test)] struct EngineKnobs { dedup_effects: bool,
verify_step_hash: bool, atomic_outbox: bool }` — le contrôle rouge devient une
**assertion**, donc prouvé par `exit 0`.

| Test (module `…store::spike::tests`) | Bras vert | **Contrôle rouge (assertion)** |
|---|---|---|
| `journal_outbox_atomic` | crash entre append et outbox → 0 effet orphelin | `atomic_outbox=false` ⇒ **assert qu'un effet orphelin EST observé** |
| `kill_restart_replay_no_duplicate_side_output` | kill aux 3 frontières → 1 effet observé par clé | `dedup_effects=false` ⇒ **assert que le double effet EST observé à B2** |
| `replay_diverges_fails_closed` | `Err(StepHashMismatch{expected,got,node_instance_id})`, run TERMINAL | `verify_step_hash=false` ⇒ **assert que le run continue** (le fail-closed n'est pas vacuously rouge) |
| `echo_target_deterministic` | même input → même output, 0 effet externe | hash **stable sous deux ordres d'insertion de clés** (garde `serde_jcs`) + assert `Err` sur float non fini |
| `run_store_synchronous_full` | `synchronous == 2` ET `journal_mode == "wal"` | (contrat de pragma — clone `db.rs:2178-2188`) |
| `spike_artifact_matches_suite` | l'artefact désérialise ; `status=="GO"` **implique** 4 scénarios listés avec leurs deux bras, et les noms de scénarios == noms de tests réellement présents | assert d'incohérence sur un artefact muté |

Contrainte transverse : le test de stabilité du hash doit être vert **en `-p
sbfb-workflow-engine` ET en `--workspace`** (graphes de features divergents, cf. §3).
Découper les scénarios de crash en tests courts (`slow-timeout` = warn 30s / hard-kill 90s,
`.config/nextest.toml:33` et `:70`) plutôt qu'un test monolithique spawn/kill × 3 × N.

### 5.3 Contenu attendu de `sprint83_e_durable_spike.json`

```json
{
  "artifact": "sprint83_e_durable_spike.v1",
  "sprint": 83,
  "phase": "E",
  "commit": "<sha du commit de phase>",
  "status": "GO",
  "crash_model": "in-process | process-kill",
  "crash_model_not_proven": ["mort du writer", "recouvrement -wal/-shm", "TerminateProcess Windows"],
  "durability_claim": "logic-only",
  "durability_contract": { "journal_mode": "wal", "synchronous": "FULL", "proof": "run_store_synchronous_full" },
  "guarantee": {
    "engine": "au plus UNE tentative non confirmée par (run_id, node_instance_id, effect_id)",
    "zero_double_effect": "prouvé contre une cible qui déduplique sur cette clé (echo-target)",
    "b2_frontier": "irréductible sans idempotence cible ; sortie nommée = outcome_unknown, 0 retry auto"
  },
  "scenarios": [
    { "name": "journal_outbox_atomic", "frontier": "avant effet", "green_arm": "PASS", "red_control": "PASS" },
    { "name": "kill_restart_replay_no_duplicate_side_output", "frontier": "entre effet et journal (B2)", "green_arm": "PASS", "red_control": "PASS" },
    { "name": "replay_diverges_fails_closed", "frontier": "après journal", "green_arm": "PASS", "red_control": "PASS" },
    { "name": "echo_target_deterministic", "frontier": "n/a", "green_arm": "PASS", "red_control": "PASS" }
  ],
  "nextest": { "profile_override": "package(sbfb-workflow-engine) retries=0", "command": "cargo nextest run -p sbfb-workflow-engine --locked", "exit": 0 },
  "gates": { "G-AUTHORITY": "OPENED", "G-STORE": "OPENED" },
  "wa_paliers": { "WA-T1": "NOT-RUN", "WA-T5": "NOT-RUN", "evidence_target": "sprint83_t2_acceptance.json (AI2)" },
  "disclosed_residuals": [
    "SQLite embarqué 3.49.2 (libsqlite3-sys 0.34.0) — CVE-2025-6965 présente ; bump bloqué (tor-dirmgr >=0.36,<0.39 ET rusqlite_migration 2.2 caret 0.36) ; mitigation = requêtes paramétrées exclusivement ; slot maintenance QPO-3",
    "leg cargo-deny LICENSES rouge pré-existant (carry CI-1) — non aggravé, non fermé",
    "bornes/rétention/GC du journal = Phase I (G-STORE)",
    "THREAT_MODEL §19 = Phase I ; E est hermétique (0 route, 0 réseau)"
  ],
  "verdict_rationale": "<1-3 phrases factuelles>"
}
```

Règle d'honnêteté : `status` n'est `GO` que si **les deux bras** des 4 scénarios sont
`PASS`. Un flake = `NO-GO` (l'override `retries=0` garantit qu'un flake se voit).

## 6. Pièges anticipés

- **`retries = 1` en CI** (`.config/nextest.toml:64`, `rust-ci.yml:156`) — masque
  exactement le mode de panne cherché. Fermé par l'override §2.3. Divergence local/CI :
  `[profile.default] retries = 0` ⇒ vert local, flake masqué en CI sans l'override.
- **`slow-timeout` 30s warn / 90s hard-kill** — un harness spawn/kill × 3 frontières dans
  UN test peut être terminé par nextest sur runner saturé et enregistré comme échec (pas
  comme crash prouvé). Découper, ou poser un override motivé.
- **Zéro-match nextest = EXIT 4** — mesuré. Le nommage des tests est une contrainte de
  gate dure (§2.4).
- **`--locked` + `Cargo.lock`** — lock régénéré dans le MÊME commit, sinon CI rouge.
- **SPDX** sur chaque `.rs` neuf, dans les 5 premières lignes.
- **PROMISE_RE anglais** — le crate engine EST la phase qui débloque F : réflexe naturel
  d'écrire « the pack format is frozen in Phase F » dans un doc-comment = gate rouge.
  Rédiger « freezing the pack format is out of scope for this crate ». Le scan couvre aussi
  `*.toml`.
- **Lancer les gates DEPUIS LA RACINE** — `check-frontier-contracts.sh`/`check-spdx.sh`
  utilisent des chemins relatifs ; le piège `web/` (dépôt git imbriqué périmé) reste actif :
  toujours `git -C C:/Users/FlowUP/Documents/Code/nexus …`.
- **Windows** — pas de `Drop`, pas de déroulement de pile sur `TerminateProcess`/
  `abort()` ; verrouillage de fichier SQLite sous Windows (rusqlite bundled) : rouvrir le
  store dans le MÊME test après un kill peut se heurter à un handle non libéré ⇒ prévoir
  `busy_timeout` et une réouverture explicite. Le nettoyage des descendants de processus
  est routé K (plan:836) mais mord dès E si on spawne.
- **SQLite : durabilité ≠ crash-safety** — « transactions are durable across application
  crashes regardless of the synchronous setting » ; WAL + `synchronous=NORMAL` peut rouler
  en arrière après perte d'alimentation. Ne JAMAIS écrire « durabilité prouvée » dans le
  body ni l'artefact.
- **In-memory vs on-disk** — `open_in_memory` du précédent coordinator ne pose NI WAL NI
  `synchronous` (`db.rs:378-390`, parité limitée au `busy_timeout`) : un harness sur
  `:memory:` ne prouve rien. Fichier tempdir obligatoire.
- **`rusqlite` `bundled`** compile SQLite depuis les sources : premier build du crate neuf
  = une unité C de plus (impact temps CI non chiffré, à surveiller sur le bloc §7.4
  `cargo build --release`).
- **Fencing single-writer** — le kickoff l'exige (D3 point 4) mais il est routé **I**
  (row 14 `G-SINGLE-WRITER`). Un harness multi-process ouvre la fenêtre où l'ancien writer
  n'est pas encore mort : `busy_timeout(5s)` n'est PAS un fencing. Sérialiser le harness
  (`serial_test`) plutôt qu'implémenter un lease en E (sur-livraison).
- **Sur-livraison E/I** — snapshots, upcasters, fencing lease, backup/GC, routes daemon
  `workflow_api.rs` sont Phase I. Rester au noyau minimal suffisant pour les 3 frontières.

## 7. Open questions

- **OQ-1 (à trancher au 1er edit, pas au review)** : `crash_model` retenu — `in-process`
  (moins cher, prouve l'atomicité transactionnelle) vs `process-kill` via
  `std::process::abort()` sous nextest (prouve la reprise après mort du writer). Le plan
  E.3 nomme des tests in-process (`engine::spike::tests::*`) mais le mot « kill » du nom
  induit l'autre lecture. **Recommandation** : `process-kill` par `abort()` (portable
  Win/Linux, 0 dep, 0 cible `[[bin]]`), et déclarer le modèle dans l'artefact quel que
  soit le choix.
- **OQ-2** : nom exact du module hôte des tests (`engine::store::spike::tests` proposé) —
  doit contenir `store` (row 12) et exclure `effect` (row 5). Arbitrage de nommage, non
  de conception.
- **OQ-3** : numéro de section `PATTERNS.md` pour le pattern livré par E — `§P74` est
  libre (la numérotation saute P73→P75) mais aucun document ne dit s'il est réservé. À
  arbitrer au review, ne pas deviner.

  > **CORRECTION (review de phase, 2026-07-20)** — cette affirmation est **FAUSSE** et
  > l'erreur est conservée ici plutôt que réécrite, pour que la trace du raisonnement
  > reste lisible. `§P74` est **PRIS** : `docs/shell/PATTERNS.md:2219` (« Cold-boot
  > reconciliation », minté S82 Phase A `19b92e6`). Le préflight n'avait regardé que
  > `docs/rust/PATTERNS.md`, alors que **l'espace `§P` est COMMUN aux deux fichiers
  > depuis §P74** — c'est précisément le finding C-1 de l'audit S82
  > (`sprint82_audit_findings.md:188`, qui recense déjà 7 collisions historiques
  > P33..P39 comme un défaut à ne pas reproduire). Recensement à HEAD : rust = P70, P71,
  > P72, P73, P75 ; shell = P70, P74. **Prochain libre = `§P76`.** Le pattern livré par
  > la Phase E est rédigé au wrap-up du sprint (précédent explicite S82 M-6 → Phase T),
  > pas au motif — faux — que `PATTERNS.md` ne serait jamais touché par une phase
  > feature : `git log -S"## §P"` montre des sections mintées en phases feature en
  > continu (S77 A/G/H/I, S79 B/H, S80 I, S82 A/I).
