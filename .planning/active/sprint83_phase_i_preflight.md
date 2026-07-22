# Sprint 83 Phase I — Preflight G8

**Sprint** : 83 — chantier « Workflow Agents » (43 phases A..AJ1).
**Phase** : I — Run store durable SQLite, plan canonique
`.planning/active/sprint83_plan.md:732-786` (I.1 scope :734-744, I.2 livrables :746-755,
I.3 tests :757-766, I.4 acceptation :768-775, I.5 commit :777-784). Décision Day-0 **D3
GELÉE** `.planning/active/sprint83_kickoff.md:330-423`.
**Date** : 2026-07-22. **HEAD** : `e034a574` (round Codex r7 Phase H) sur `3e51d05b`
(compilateur `CompiledPlan.v1`), arbre propre.
**Méthode** : mode ultracode Workflow — **5 agents de scan** (S1a prior-art OSS profond,
S1b dépendances/CVE/graphe transitif, S2 décisions historiques traversées, S3 couverture
THREAT_MODEL, S4 invariants wire/frontières) + passes adversariales, puis cette synthèse.
**Honnêteté d'exécution** : dans le run Workflow initial, seule la passe adversariale
RÉFUTEUR a abouti ; les passes COMPLÉTUDE et FAITS-DATÉS sont mortes (cap StructuredOutput
×5) et le scan S1a n'a rendu qu'un placeholder. Les TROIS rôles ont été REJOUÉS en fan-out
Agent avant-plan le 2026-07-22 — résultats intégrés en **§9bis** et adaptations 16-22 (§10).
Le synthétiseur a **rejoué à HEAD** les mesures porteuses :
stub core `RunEvent {}` vide (`contracts.rs:74-77`) + `WorkflowPublicProjection {}` vide
(`contracts.rs:94-97`) → **5/7 identités remplies** (pack/graph/agent_definition/factory_profile/
compile via `pub use`), registre `len == 7` (`contracts.rs:142-150`) ; `RUN_EVENT_FORMAT_VERSION
= 1` **constante LOCALE** (`event.rs:33`) ; `load_events` désérialise `from_str` direct sans
étape upcaster ni garde plafond (`store.rs:654,660-664`) ; schéma `runs` **sans colonne lease**
(`store.rs:52-59`) ; clause concurrence « single-writer fencing OUT OF SCOPE » (`lib.rs:48-53`) ;
crate engine **NE dépend PAS** de `sbfb-workflow-core` (`sbfb-workflow-engine/Cargo.toml` grep
= 0) ; census DOMAIN gelé == **29** (`grep -roh DOMAIN_*_V1 crates/nexus-core-rs/src` = 29 uniques),
`DOMAIN_RUN_EVENT_V1` RÉSERVÉ 0-producteur (`canonical.rs:79,404`) ; `crates/nexus-shell-daemon/
src/workflow_api.rs` **ABSENT** (ls = No such file) ; THREAT_MODEL §18 « run store durable = I »
(`docs/security/THREAT_MODEL.md:1728`) + cadence §19 « Revue et evolution » (`:1740-1742`) ;
pins `rusqlite = "0.36" features=["bundled"]` (`Cargo.toml:137`) + `rusqlite_migration = "2.2"`
(`:138`) ; `arti-client = "0.41"` (`:467`, contrainte tor-dirmgr `:466`).
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux de scan : S1a=REFUTÉ (placeholder),
S1b=EXECUTE, S2=PLAN-ADAPT, S3=PLAN-ADAPT, S4=PLAN-ADAPT.

---

## 1. Rationale du verdict

**Aucune décision Day-0 (D3 GELÉE) ni aucun invariant du plan n'est en tension structurelle
avec Phase I.** Toutes les briques que I doit livrer — journal `RunEvent.v1` enveloppe versionnée
une fois + payload `Value`, upcasters à l'interprétation, snapshots jetables, inbox/outbox
transactionnels + idempotence, fencing single-writer (lease), rétention/backup/restore/GC,
rejouer un ancien run + refuser un downgrade dangereux, run store = une classe de plus dans
l'inventaire migrations, doctrine sécurité checkpoints — sont **énumérées verbatim dans D3**
(`kickoff:368-388`) et dans le scope I.1 (`plan:734-744`). Phase I **étend le socle du spike E**
(`6de49b12`), délibérément conçu extensible-sans-DROP (`store.rs:47-51`) ; elle ne le refond pas.

Le verdict n'est **pas DESIGN-CONFLICT** : aucune alternative rejetée au kickoff (replay-total,
wasmtime/Obelisk, cqrs-es, redb comme run store) n'est ré-ouverte, aucune dépendance neuve ni
bump n'est requis, aucun arbitrage PO n'est déclenché. `git log --grep=DEVIATION` sur les 2 crates
workflow est vide (le seul hit `rejected -i` est le nom de test `unknown_field_rejected_for_every_kind`,
Phase G `24ff47e2` — pas une alternative ré-ouverte ; vérifié §9bis) ; toutes les corrections sont des précisions de FORME (upcaster non câblé, garde
downgrade non enforced, lease/inbox/snapshots net-neufs, amendement THREAT_MODEL, inventaire D7 +
docs-contrat, décision stub RunEvent.v1), chacune appuyée sur une evidence code/OSS rejouée à HEAD.

Le verdict n'est **pas EXECUTE** parce que le texte du plan I omet des obligations matérielles :
(a) le point d'insertion de l'upcaster **et** la garde de refus de downgrade sont NET-NEUFS — le
spike lit `format_version` sans jamais le comparer au courant (`store.rs:660-664`) ; (b) I.5 ne
liste AUCUN amendement THREAT_MODEL alors que §18:1728 défère explicitement « run store durable
= I » et §19:1742 impose la cadence ; (c) les routes `workflow_api.rs` (fichier NEUF) sont une
VRAIE frontière docs-contract (runtime distinct shell/Hub), obligation D7 LOOPBACK + clôture
docs-contrat (canon S80) ; (d) la décision « remplir le stub core `RunEvent.v1` vs garder
l'enveloppe locale » n'est pas tranchée (le plan I.2 liste 0 livrable core) ; (e) le backup doit
passer par `VACUUM INTO` (SQL pur) et non l'API `Connection::backup` (feature-change workspace).

Le verdict n'est **pas SCOPE-CUT-CONSISTENT** : rien n'est retiré — le périmètre est conservé et
précisé.

**Correction de la prémisse de la tâche** : « identité 6/7 remplie » est **FAUX** — le code montre
**5/7 remplies** ; DEUX stubs restent vides (`RunEvent.v1` traité par I, `WorkflowPublicProjection.v1`
différé phase publication/RRV). Même si I remplit `RunEvent.v1`, le registre atteint 6/7, jamais 7/7.

**Risques de sévérité haute** : (1) un test `old_run_replays_after_upgrade` VERT alors qu'aucun
upcaster n'upcast rien = vacueux (learning `feedback_mutation_before_trust`) → contrôle rouge
discriminant obligatoire ; (2) un `snapshot_is_disposable_cache` qui ne prouve pas qu'un snapshot
empoisonné est détecté/ignoré = faux vert.

---

## 2. Base de départ mesurée (rejouée à HEAD `e034a574`)

| Fait | Mesure | Source |
|---|---|---|
| Stub core `RunEvent.v1` | `pub struct RunEvent {}`, `deny_unknown_fields`, `title` | `contracts.rs:74-77` |
| Stub core `WorkflowPublicProjection.v1` | vide (hors-scope I) | `contracts.rs:94-97` |
| Identités remplies | **5/7** (`pub use` pack/graph/agent_def/factory_profile/compile) | `contracts.rs:39,48,59,68,88` |
| Registre contrats | `len == 7`, refresh via `UPDATE_SNAPSHOTS` (snapshot STRICT) | `contracts.rs:142-150` |
| Doc-module PÉRIMÉ | dit « compiled-plan… still empty stubs » alors que `CompiledPlan` REMPLI :88 | `contracts.rs:8-9` vs `:88` |
| Envelope locale spike | `RUN_EVENT_FORMAT_VERSION: u16 = 1` (LOCAL, pas wire) | `event.rs:33` |
| Incohérence cosmétique | `//! RunEvent v0` alors que la constante = 1 | `event.rs:2` |
| Résiduel PII identifiants | `run_id`/`node_instance_id` caller-chosen, validés longueur/contrôle seuls | `event.rs:24-28` |
| Payload journal | enum TYPÉ `RunEventKind` `deny_unknown_fields`, sérialisé TEXT (pas `Value`) | `event.rs:53` ; `store.rs:67` |
| Schéma spike | tables `runs`/`run_events`/`outbox` extensibles SANS DROP | `store.rs:47-51` |
| `runs` sans lease | aucune colonne lease/owner/expiry | `store.rs:52-59` |
| `load_events` | `from_str` direct → `RunEventKind` (fail-closed `Corrupt`), 0 upcaster | `store.rs:654` |
| Garde version au read | `u16::try_from(format_version)` = range 0..65535 SEULEMENT, 0 compare au courant | `store.rs:660-664` |
| Idempotence outbox | index unique `(run_id, node_instance_id)` fail-closed | `store.rs:90-91` |
| Fencing single-writer | « OUT OF SCOPE for this spike », modèle single-writer sérialisé nommé | `lib.rs:48-53` |
| FaultHook sur `execute()` | param `faults: &dyn FaultHook` sur la signature ; garde `spike-faults` OFF défaut | `engine.rs:239-244` ; `Cargo.toml` |
| Frontière crate | test-actor : lecteurs = ce crate + processus ultérieur du même code ⇒ 0 frontier tag | `lib.rs:98-109` |
| Census DOMAIN | **29**, `DOMAIN_RUN_EVENT_V1` réservé 0-producteur | `canonical.rs:79,404` ; grep = 29 |
| Tension définitionnelle | contracts = journal LOCAL ; canonical:79 = « exported run event » | `contracts.rs:70-73` vs `canonical.rs:79` |
| `workflow_api.rs` | ABSENT (à créer) | `ls` = No such file |
| THREAT_MODEL | §18:1728 « run store durable = I » ; §19:1740-1742 cadence d'amendement | `THREAT_MODEL.md` |
| Pins run store | `rusqlite "0.36" bundled` :137 ; `rusqlite_migration "2.2"` :138 (0 bump) | `Cargo.toml:137-138` |
| Contrainte arti-client | `arti-client "0.41"` :467, `tor-dirmgr needs rusqlite >=0.36` :466 (tor OFF défaut) | `Cargo.toml:463-467` |
| Engine ↛ core | `sbfb-workflow-engine/Cargo.toml` n'a AUCUNE dep sur `sbfb-workflow-core` | grep = 0 |

---

## 3. Scan S1a — Prior art OSS profond (verdict de scan : REFUTÉ / placeholder)

Le scan S1a n'a retourné qu'une **donnée placeholder de test** (summary `"test"`, finding
`S1A-1` claim `"c"` evidence `"e"` implication `"i"`) — REFUTÉ en adversarial (aucune référence
fichier:ligne, rien de substantiel à confirmer). Il est **écarté du corpus**.

Conséquence : la dimension prior-art n'apporte **aucun delta neuf** pour Phase I. L'analyse OSS
de fond a déjà été menée et **gelée dans D3** (`kickoff:330-410`) : familles replay-de-commands
(Temporal) vs mémoïsation-de-steps + journal (DBOS/Restate/Inngest/Morling), rejet documenté de
`temporalio-sdk` 0.x, Obelisk/wasmtime, cqrs-es, redb ; chaîne CVE checkpointer LangGraph 2026
(CVE-2025-67644 SQLi + CVE-2026-28277 msgpack→RCE) motivant la doctrine D3.6. Rien n'est ré-ouvert.
Les faits dépendances/CVE actionnables pour I sont portés par S1b (§4).

**Rejeu RÉEL du scan (fan-out Agent, 2026-07-22, §9bis)** : verdict par cible ALIGNE/COMPLETE,
rien ne CONTREDIT le preflight, aucun DESIGN-CONFLICT ré-ouvert. 7 findings S1A-R1..R7 intégrés
(fencing CAS-first Kleppmann/Temporal/DBOS, limites VACUUM INTO sqlite.org, GC aligné
DBOS/Temporal/Inngest, upcasters par-hop Axon, inbox `processed_at`, correction fait
CVE-2025-6965 fixée dès rusqlite >=0.37.0). Delta SOTA fenêtre 2026-07-17→22 : rien de neuf.

---

## 4. Scan S1b — Dépendances / CVE / graphe transitif (verdict de scan : EXECUTE)

Phase I s'exécute avec **0 dépendance runtime/dev neuve** et **0 bump**. Tous **CONFIRMED**.

- **S1b-01 — 0 dep neuve, 0 bump. [INFO]** `rusqlite 0.36.0` (`Cargo.lock:7225-7227`),
  `rusqlite_migration 2.2.0` (`:7240-7242`), `libsqlite3-sys 0.34.0` (`:4523-4524`) : version unique
  chacun (`cargo tree -d` = rusqlite/libsqlite3-sys ABSENTS des paquets dupliqués, constat
  2026-07-22). La crate cible déclare déjà les deux (`sbfb-workflow-engine/Cargo.toml` deps
  workspace) et consomme déjà `rusqlite_migration` (`store.rs:42`). Coder schéma+migrations+
  upcasters+snapshots+inbox/outbox+lease ne touche NI `Cargo.toml` racine NI un nœud de lock.
- **S1b-02 — Backup via `VACUUM INTO`, PAS l'API `Connection::backup`. [P2]** La feature rusqlite
  `backup` n'est activée nulle part (seul `["bundled"]`, `Cargo.toml:137` ; grep = 0 `backup`).
  `VACUUM main INTO <path>` (SQL pur, supporté par le bundle SQLite 3.49.2 ≥ 3.27) tient la
  contrainte 0-feature-change/0-dep ; choisir l'API online `Connection::backup` ajouterait
  silencieusement `features=["backup"]` au rusqlite **workspace partagé par 5 crates** — à
  INTERDIRE ou documenter explicitement comme feature-change. Chemin de backup paramétré/validé.
- **S1b-03 — CVE-2025-6965 latente, hors-chantier. [P2]** SQLite bundlé 3.49.2 (`libsqlite3-sys-
  0.34.0/sqlite3/sqlite3.h` → `SQLITE_VERSION "3.49.2"`), fix = 3.50.2 (constat 2026-07-22) ;
  **NON surfacée par RustSec** (page libsqlite3-sys = seul RUSTSEC-2022-0090 résolu ≥ 0.25.1, on
  est à 0.34.0) → cargo-audit/deny verts. Closable seulement par le bump maintenance QPO-3
  hors-chantier. Surface d'exploit ~nulle pour le run store : SQL crate-authored paramétré
  (`params!` partout), état données-pures JSON, aucun SQL dérivé d'entrée non-fiable (doctrine D3.6).
- **S1b-04 — Dérive fichier:ligne pins. [P3]** kickoff:344 et plan:413 citent « Cargo.toml:135-136 »
  alors que les pins sont à **137-138** (134-136 = commentaire W7). Corriger au body I.5.
- **S1b-05 — Dérive + nuance arti-client. [P3]** kickoff:346/plan:413 citent « Cargo.toml:461-465 »
  alors que `arti-client = "0.41"` est à **467** et le commentaire de contrainte à **463-467**
  (`:466` « tor-dirmgr (transitive) needs rusqlite >=0.36 »). arti-client est **optionnel derrière
  la feature `tor` (OFF par défaut)** (`nexus-core-rs/Cargo.toml:131,134`) → la contrainte rusqlite
  ne lie que sous `--features tor` ; sans impact sur le code Phase I (n'active pas tor).
- **S1b-06 — RUSTSEC propre. [INFO]** Aucun advisory ouvert affectant rusqlite 0.36.0 /
  rusqlite_migration 2.2.0 ; le gate supply-chain (cargo-deny advisories) reste vert sur l'axe run
  store. Aucune remédiation dep requise pour fermer G-STORE côté sécurité des dépendances.

---

## 5. Scan S2 — Décisions historiques traversées (verdict de scan : PLAN-ADAPT)

Phase I ÉTEND le spike E ; elle ne le REFOND pas. Tous **CONFIRMED** sauf S2A-3 **NUANCE**.

- **S2A-1 — Le code E est une FONDATION à étendre, pas un jetable. [INFO]** `store.rs:47-51` : les
  tables `runs`/`run_events`/`outbox` sont « chosen to be extensible by the full run store without
  a DROP » ; le commit E (`6de49b12`) route retention/GC/fencing/snapshots/upcasters/backup vers I
  comme AJOUTS au MÊME store. `engine.rs` (SpikeEngine linéaire) reste jusqu'à J. → I édite
  `store.rs`/`event.rs`, ne réécrit ni crate ni schéma, ne réinvente pas le journal/outbox/
  idempotence déjà prouvés GO.
- **S2A-2 — Upcaster + garde downgrade NET-NEUFS. [P2]** `store.rs:654` fait `from_str` direct en
  `RunEventKind` ; `:660-664` fait `u16::try_from(format_version)` (range-check seul), jamais
  comparé à `RUN_EVENT_FORMAT_VERSION=1` (`event.rs:33`). I doit : (1) insérer l'étape upcaster
  à l'interprétation (lire payload comme `Value`, transformer, puis parser) ; (2) refuser
  fail-closed `format_version > courant` (downgrade dangereux) ; (3) committer un golden d'une forme
  de payload ANTÉRIEURE synthétique + contrôle rouge (retirer l'upcaster casse le test).
- **S2A-3 — Dette E « à fermer avant le premier consommateur ». [P2, NUANCE]** Claim cœur VRAI :
  `engine.rs:239-244` expose `execute(spec, target, faults: &dyn FaultHook)` sur la signature de
  production, et `effect_id` est injectable via l'API bas-niveau (store append/insert). **Nuance
  adversariale** : la feature `spike-faults` (OFF par défaut) gate déjà les constructeurs dégradés
  et `NoFaults` (`lib.rs:90`) ne se déclenche jamais → défaut **inerte**, hygiène de surface, pas
  un trou actif. I introduit précisément le 1er consommateur réel (routes daemon) → sortir FaultHook
  de la surface publique de production (garde `spike-faults`) et rendre `effect_id` non-injectable
  hors `execute()`.
- **S2A-4 — `workflow_api.rs` = VRAIE frontière docs-contract. [P2]** Fichier ABSENT (constaté).
  Contrairement à E hermétique (test-acteur : 0 lecteur hors du Rust du crate, `lib.rs:98-109`), les
  routes run store sont lues par le shell/Hub (runtime distinct) → obligation d'inventaire D7
  LOOPBACK (`inventory_policy: representative-locked`, `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:5`) +
  clôture docs-contrat (canon S80 : une API loopback lue par un runtime distinct EST une frontière).
  Le plan I.2:754-755 engage l'inventaire mais ni I.2 ni I.5 ne mentionnent la clôture docs-contrat.
- **S2A-5 — Décision stub core `RunEvent.v1`. [P2]** `contracts.rs:74-77` = `pub struct RunEvent {}`
  (doc :72-73 « envelope authored in the run-store phase » = I). Le spike a délibérément gardé
  l'enveloppe locale « while the GO/NO-GO question is still open » (`event.rs:12-17`, `lib.rs:98-109`)
  — **rationale EXPIRÉ** (E = GO). Décision consciente à trancher : remplir le schéma `RunEvent.v1`
  dans core (comme H a rempli `CompiledPlan.v1`, len==7 préservé, census 29) OU garder l'enveloppe
  locale au crate engine. Incohérence cosmétique à réconcilier : `event.rs:2` dit « v0 » alors que
  la constante = 1.
- **S2A-6 — Lease single-writer NET-NEUF. [P3]** Le modèle E est single-writer sérialisé, fencing
  NOMMÉ hors-périmètre. `store.rs:52-59` : `runs` sans colonne lease ; `BEGIN IMMEDIATE` ne garde
  qu'UNE connexion. Le lease doit empêcher DEUX PROCESSUS possédant le même fichier store d'émettre
  la même outbox (owner token + expiration monotone, via colonne/table nouvelle en migration forward).
- **S2A-7 — 0-bump strict + migrations forward-only. [P3]** `Cargo.toml:137-138` (0 bump) ; `params!`
  partout (`store.rs:41-42`, 0 concaténation SQL). Toute table neuve (inbox/snapshots/lease) = `M::up`
  appendu à `MIGRATIONS` (jamais éditer l'existant). Doctrine paramétrée + état données-pures JSON
  conservée. NB : le finding cite parfois « 134-138 » (englobant) ; pins réels 137-138.
- **S2A-8 — Run store = une classe de plus dans l'inventaire migrations. [P3]** plan I.1:741-742
  (« pattern S82 T, par classe de store ») ; le golden est déjà semé
  (`sbfb-workflow-engine/src/run_event.golden.json`, livré E). Culture §P75 (`PATTERNS.md:4329-4360`)
  + `store_migration.json` PASS par classe (`sprint82_audit_findings.md:530`). → I produit un
  artefact/entrée d'inventaire pour la classe « workflow run store » + goldens d'événements committés
  rejoués (`golden_events_replay_stable`, oracle byte-identique cross-run).
- **S2A-9 — Découpe reducer/scheduler = J, pas I. [INFO]** `engine.rs:16-19` « There is NO in-memory
  frontier cache: the store is re-read at every step ». Les routes de I sont store-level (inspection/
  CRUD : run status, événements, frontière, outbox pending) ; le pilotage d'exécution
  (branches/parallel/map/join/repeat/cancel) est J/K. Le snapshot ajouté en I est un cache JETABLE
  reconstructible-du-log, jamais autorité. → Ne pas sur-investir SpikeEngine en I.
- **S2A-10 — Précédents internes à RÉUTILISER, pas recoder. [P3]** `kickoff:348-350` : dedup
  `(worker_pubkey, task_id)` S76 D → inbox dedup at-least-once ; replay outbox S75 A ; binding
  fail-closed S81 K → gardes downgrade/scope ; feed raw-op enveloppe+`Value` → extensibilité payload
  sans bump. L'inbox est NET-NEUVE (E n'a que `runs`/`run_events`/`outbox`, `store.rs:52-83`).

---

## 6. Scan S3 — Couverture THREAT_MODEL (verdict de scan : PLAN-ADAPT)

Le run store est un ASSET DURABLE NEUF non modélisé. Tous **CONFIRMED**.

- **S3-01 — Aucun amendement THREAT_MODEL prévu au plan. [P1]** `THREAT_MODEL.md:1728` défère
  textuellement « run store durable = I » (§18 bornes honnêtes) ; §19:1740-1742 impose la cadence
  (« Chaque sprint qui livre une mitigation… ajouter §5.x STRIDE + §6 LINDDUN + ligne §2 Assets +
  §4 DFD »). Précédents A9 (§17, Phase B) et A10 (§18, Phase H). Or plan I.2:746-755 et I.5:777-784
  ne listent AUCUN amendement. → I doit AJOUTER un asset run store (~A11 : état d'exécution durable
  = journal+outbox+lease+snapshots) + menaces T-RUN-* (store forgé/anti-rollback, downgrade
  mal-parsé, double-émission par lease-spoof, perte de preuve GC, snapshot poisoning, SQLi/RCE
  fermées par paramétré+données-pures) + amender §19 + refléter dans I.5. Sinon la surface ship
  un-modeled et l'audit gate le lèvera (docs-contrat = livrable de fermabilité).
- **S3-02 — Refus de downgrade non enforced. [P1]** `store.rs:654` construit `RunEventKind` sans
  jamais rejeter une `format_version` supérieure à `RUN_EVENT_FORMAT_VERSION=1` (`:660-664` =
  range-check seul). Scope plan:740 « refuser un downgrade dangereux » = NET-NEUF. Un binaire ancien
  ouvrant un store écrit par un format plus récent doit fail-closed DÉLIBÉRÉMENT (ni mal-interpréter
  ni erreur opaque). G-UPGRADE-REPLAY doit prouver LES DEUX directions : upcast ancien→courant ET
  refus courant<neuf (= aussi la mitigation anti-rollback du restore).
- **S3-03 — Tier D7 délibéré par famille. [P2]** `workflow_api.rs` ABSENT → les routes neuves
  déclenchent le trigger D7 « Nouveau endpoint loopback risky ajouté daemon ». restore/GC/lease-force
  = perte-d'évidence/destructif ⇒ candidat **T1** ; status/timeline = **T0**. Plan I.2 engage
  l'inventaire mais le tier doit être DÉLIBÉRÉ : la route restore (écrase le journal = anti-rollback
  + perte d'audit) est la plus risquée et n'est pas T0.
- **S3-04 — Doctrine Ed25519-avant-parse mappée au bon endroit. [P2]** `spec_digest` stocké TEXT en
  clair, non signé (`store.rs:56`) ; l'autorité Ed25519 est vérifiée EN AMONT par le compilateur
  (Phase H), jamais par événement. Application concrète I : la route daemon de création de run ne
  doit pas accepter un spec non-vérifié (re-vérifie la signature avant création) ; les upcasters ne
  parsent que du JSON données-pures (transforms `Value→Value`, JAMAIS de code) = la partie load-bearing
  de la réponse à la chaîne CVE LangGraph. Trust boundary = single-writer local (frontière OS
  mono-utilisateur), input non-fiable = le SPEC/pack (vérifié en amont, binding Phase K).
- **S3-05 — Résiduel PII d'identifiant. [P2]** `event.rs:24-28` documente : `run_id`/`node_instance_id`
  sont des `String` caller-chosen validées longueur/contrôle seulement ; digest-only vaut pour les
  CHAMPS de payload par type, PAS pour les identifiants. Dès qu'une route accepte `run_id` depuis
  l'iframe/Hub, l'« appelant » devient du contenu d'app non-fiable → générer/valider `run_id` côté
  serveur OU documenter le résiduel PII (le journal survit au process). À inscrire dans l'amendement.
- **S3-06 — Disposabilité des snapshots = obligation de PREUVE. [P2]** plan:738-739 « snapshots
  jetables, jamais source de vérité » + test prévu :764. Un snapshot corrompu/supprimé doit donner
  le MÊME état réduit reconstruit du journal ; le runtime ne doit JAMAIS traiter un snapshot comme
  autorité. → Enforced par construction (réduction re-dérivable + invalidation par `snapshot_version`)
  AVEC contrôle rouge discriminant (snapshot empoisonné détecté/ignoré) — culture fault-injection E
  (mémo `feedback_mutation_before_trust`).
- **S3-07 — GC/rétention vs preuve d'audit. [P2]** journal/outbox croissent sans borne
  (`store.rs:33-37`, GC routé I) ; le diagnostic terminal (`event.rs:74-85` RunTerminal/
  outcome_unknown) est nécessaire au post-mortem. Politique EXPLICITE + testée : ne jamais purger
  (a) runs en vol, (b) outbox pending, (c) le diagnostic terminal ; ne purger que completed/terminal
  au-delà d'un horizon explicite. L'amendement acte « GC = perte potentielle de preuve d'audit,
  bornée par politique ».
- **S3-08 — Surface du lease single-writer. [P2]** `lib.rs:48-53` : `FileEchoTarget` dédup
  read-then-append sans lock, « two writers… can both read absent and both append. Single-writer
  fencing is out of scope for this spike ». `BEGIN IMMEDIATE` sérialise les transactions sur UN
  fichier mais n'empêche pas deux PROCESS d'ouvrir le même fichier. Un token de fencing (monotone +
  owner + expiry, vérifié DANS la transaction d'avancement) est requis ; G-SINGLE-WRITER doit prouver
  qu'un writer fenced-out ne peut PAS émettre d'outbox. Modéliser lease-spoof + crash-during-lease
  en T-RUN-*.
- **S3-09 — Tension raw-op vs `deny_unknown_fields`. [INFO]** `event.rs:53` pose
  `deny_unknown_fields` sur `RunEventKind` (+ enveloppe) et stocke un enum TYPÉ en payload TEXT
  (`store.rs:67`), pas un `serde_json::Value` ; le plan I.1:737-738 exige « payload `Value` extensible
  (politique feed raw-op) ». Un enum `deny_unknown_fields` n'est PAS forward-tolérant. Le store étant
  single-writer local (ses propres écritures passées), la tolérance store-and-forward compte moins que
  pour le gossip — mais I doit réconcilier enveloppe-typée + payload-`Value` et graver le contrat
  upcaster (`Value→Value`, jamais de code). À trancher explicitement ; pas un défaut de sécurité en soi.

---

## 7. Scan S4 — Invariants wire format / frontières (verdict de scan : PLAN-ADAPT)

Le run store est LOCAL : 0 octet cross-nœud, 0 signature d'entrée. Tous **CONFIRMED** sauf S4-03
**NUANCE**.

- **S4-01 — 0 bump wire, 0 DOMAIN neuf, census 29. [INFO]** `RUN_EVENT_FORMAT_VERSION=1` est une
  version de store LOCAL (`event.rs:32-33`, `lib.rs:95-96`), jamais un wire/DOMAIN ; le journal n'est
  pas signé (0 sign/verify/Ed25519 dans le crate engine, `lib.rs:104-106`). `DOMAIN_RUN_EVENT_V1`
  reste réservé 0-producteur (`canonical.rs:404`). Census figé à **29** (grep = 29 uniques). → Le
  body I doit affirmer : `RUN_EVENT_FORMAT_VERSION=1` local, 0 bump, 0 DOMAIN, census 29 inchangé.
- **S4-02 — Décision remplir ou non le stub core `RunEvent.v1`. [P1]** `contracts.rs:73` assigne
  l'authoring à « the run-store phase » (=I) et la cadence des 6 autres identités le confirme (D→F→G→H
  chacune remplie dans sa phase), mais le plan I.2 liste 0 livrable core, et le spike argumente
  explicitement de NE PAS remplir (rationale « GO/NO-GO ouvert » désormais expiré, E=GO). Vérifié :
  `sbfb-workflow-engine/Cargo.toml` n'a AUCUNE dep sur `sbfb-workflow-core`. → Si remplissage : struct
  schemars pure dans core + refresh `contracts.schema.json` (golden §P75) en gardant `len==7` (corps
  5/7→6/7), déplacer `RunEventKind` (serde pur) vers core OU garder l'enveloppe locale mirror, ajouter
  la dép intra-workspace (autorisée, non-externe), amender `event.rs:12-17`/`lib.rs:98-109` (rationale
  expiré). Sinon : justifier la rupture de cadence.
- **S4-03 — Deux définitions de `RunEvent.v1` à réconcilier. [P2, NUANCE]** `contracts.rs:70-73`
  décrit `RunEvent.v1` = « append-only run-journal entry (D3)… source of truth for durable replay »
  (LOCAL, non signé) ; `canonical.rs:79` décrit `DOMAIN_RUN_EVENT_V1` = « exported run event »
  (réservé 0-producteur). **Nuance** : tension latente à faible impact I (le journal LOCAL non signé
  porte un tag de signature DOMAIN réservé pour un futur EXPORT distinct). Day-0 D3 (`kickoff:369`)
  nomme le JOURNAL « RunEvent.v1 » → lecture locale privilégiée. Load-bearing seulement dès qu'un
  export signé est construit (phase ultérieure) ; sans impact code I (active aucun domaine).
- **S4-04 — Prémisse « 6/7 remplie » FAUSSE = 5/7. [P2]** `contracts.rs` : remplis = pack:48,
  graph:59, agent_definition:68, factory_profile:39, compile:88 (5 `pub use`) ; vides = `RunEvent {}`
  :77 et `WorkflowPublicProjection {}` :97 (2). `CONTRACT_NAMES len==7` :142-150. Même si I remplit
  `RunEvent.v1`, le registre atteint 6/7 — le commit I ne doit pas revendiquer « registre complété ».
- **S4-05 — `workflow_api.rs` = inventaire D7, PAS tag wire FRONTIER. [P2]** Fichier ABSENT. Ses
  routes relèvent de l'inventaire D7 loopback (`inventory_policy: representative-locked`), pas du tag
  `// FRONTIER:` de `check-frontier-contracts.sh` (réservé aux types signés domain+version). Toute
  donnée de run exposée au front doit être une PROJECTION allow-listée (`WorkflowPublicProjection` =
  phase ultérieure), pas un `RunEvent` brut. → Routes I = store-level (status/CRUD internes),
  enregistrées dans `build_router` (compteur de routes), arbitrées dans l'inventaire D7.
- **S4-06 — Point d'insertion de l'upcaster. [P2]** `store.rs:636-675` `load_events`, `:654`
  `from_str` direct (deny_unknown_fields `event.rs:53` ⇒ fail-closed `Corrupt` sur forme ancienne),
  `:660-664` `format_version` relu range-checké seulement. L'upcaster chaîné de I doit s'intercaler
  entre lecture de la ligne (`Value`) et parse `RunEventKind` courant, à l'INTERPRÉTATION (jamais
  migration destructive du journal). Le fail-closed actuel est le bon défaut mais bloque le replay
  sans l'upcaster câblé.
- **S4-07 — Pattern feed raw-op déjà appliqué + payload non-JCS légitime. [INFO]** `store.rs:60-92`
  (colonnes `format_version`+`payload` TEXT), `:778-793` (append : payload = `to_string(kind)`,
  format_version écrit), `:656-664` (relu depuis la ligne). I ajoute inbox/snapshots/lease via de
  NOUVEAUX `M::up` appendus (jamais éditer l'existant), inscrit la classe run-store à l'inventaire
  migrations, et conserve le payload journal en `serde_json` (non-JCS) — légitime car le journal
  n'est ni signé ni hashé (les digests JCS portent sur step-request/receipt `hash.rs`, pas sur la
  ligne de journal).

---

## 8. Additions adversariales intégrées

- **ADD-1 — Doc-comment de module PÉRIMÉ. [P3]** `contracts.rs:8-9` liste « the run-journal,
  compiled-plan and public-projection identities are still empty stubs » — or compiled-plan
  (`CompiledPlan`) a été REMPLI en Phase H (`contracts.rs:88` `pub use crate::compile::CompiledPlan`,
  commit `3e51d05b`). Non capté par S2/S3/S4. → Quand I éditera `contracts.rs` (si elle remplit
  `RunEvent.v1`), corriger ce doc-comment pour ne plus lister compiled-plan comme stub vide (état
  réel = 5/7 remplis, restent `RunEvent.v1` [I] + `WorkflowPublicProjection.v1` [publication/RRV]).
- **ADD-2 — Vocabulaire gates G-STORE à aligner. [INFO]** I.1:744 nomme `G-STORE` (à fermer) tandis
  que I.5:783 dit « ferme G-STORE partiel » ; la consigne de tâche dit « G-STORE (ferme) ». →
  Trancher au preflight : **G-STORE se ferme en I** (le store complet — schéma+migrations+upcasters+
  snapshots+inbox/outbox+backup/GC — est livré), **G-UPGRADE-REPLAY + G-SINGLE-WRITER s'ouvrent-et-
  prouvent en I**. Aligner le body de commit ; sans impact code.

---

## 9. Passe adversariale — ce qui a été réfuté / nuancé, et pourquoi

Sur 30 findings examinés (tous P0/P1 + toutes les scans PLAN-ADAPT S2/S3/S4 + S1b vérifiés),
**1 RÉFUTÉ**, **2 NUANCÉS**, le reste **CONFIRMÉ** après ré-ouverture du code réel à HEAD.

- **S1A-1 — RÉFUTÉ.** Donnée placeholder de test (scan S1a summary=`"test"`, claim=`"c"`,
  evidence=`"e"`). `e` n'est pas une référence fichier:ligne vérifiable, aucun contenu substantiel.
  Écarté du corpus (§3). Impact : la dimension prior-art n'apporte aucun delta neuf — l'analyse OSS
  est déjà gelée dans D3.
- **S2A-3 — NUANCÉ.** La dette E (FaultHook sur `execute()` + `effect_id` injectable) est réelle
  MAIS la feature `spike-faults` (OFF défaut) gate déjà les constructeurs dégradés et `NoFaults` ne
  se déclenche jamais → défaut **inerte** (hygiène de surface à fermer avant le consommateur daemon,
  pas un trou actif). La claim « à fermer avant le premier consommateur » reste valide.
- **S4-03 — NUANCÉ.** La tension `RunEvent.v1` local (contracts) vs « exported run event »
  (canonical:79) est réelle mais **à faible impact I** (le crate active aucun domaine, le journal
  n'est ni signé ni exporté en I). Load-bearing seulement dès qu'un export signé est construit.

Les **4 findings P1** survivent tous : S3-01 (aucun amendement THREAT_MODEL), S3-02 (refus de
downgrade non enforced), S4-02 (décision stub `RunEvent.v1` non tranchée). La prémisse de la tâche
« identité 6/7 remplie » est **corrigée en 5/7** (S4-04). Aucun blocage dépendance (S1b : 0 dep /
0 bump faisable).

---

## 9bis. Rattrapage adversarial (fan-out Agent avant-plan, 2026-07-22)

Les passes COMPLÉTUDE et FAITS-DATÉS du Workflow étant mortes (cap StructuredOutput ×5) et le
scan S1a n'ayant rendu qu'un placeholder, les trois rôles ont été rejoués en agents Explore
parallèles. Résultats : **rien ne contredit le verdict PLAN-ADAPT** ; 20 findings neufs le
renforcent (4 P1, 8 P2, 4 P3, 4 INFO). Intégrés en adaptations 16-22 (§10).

### S1a rejoué — prior-art OSS réel (7 findings)

- **S1A-R1 [P1] Fencing = CAS sur token monotone ; l'expiry n'est JAMAIS la condition de commit.**
  Kleppmann fencing-tokens (« you cannot make assumptions about timing ») ; Temporal `rangeID` =
  « monotonically increasing generation number used for fencing », avancement par conditional
  update ; DBOS `executor_id` + `recovery_attempts`, timeout 60 s = déclencheur de takeover, pas
  garde de commit (constats 2026-07-22). → lease avec colonne `epoch` monotone bumpée à chaque
  acquisition ; l'avancement fait `UPDATE … WHERE owner = ? AND epoch = ?` et **exige
  `rows_affected == 1`** sinon abort fail-closed ; l'expiry n'est lu QUE par un aspirant-owner
  pour décider de voler le lease (bump epoch). Spécificité SQLite : `BEGIN IMMEDIATE` sérialise
  déjà les write-txns — le fencing protège contre l'owner PÉRIMÉ après takeover, pas contre deux
  commits simultanés.
- **S1A-R2 [P2] Lease-check + append outbox = UN SEUL `BEGIN IMMEDIATE`.** Un SELECT lease en
  read-txn puis upgrade write déclenche `SQLITE_BUSY_SNAPSHOT` SANS invoquer le busy_handler
  (sqlite.org/forum, constat 2026-07-22) → jamais lire le lease dans une txn séparée ; connexion
  persistante + busy_timeout (déjà le shape du spike, à préserver).
- **S1A-R3 [P2] `VACUUM INTO` : 3 limites à graver** (sqlite.org/lang_vacuum.html, 2026-07-22) :
  (1) échoue si transaction/statement ouverts sur la connexion ; (2) cible fsync'd SEULEMENT si
  `synchronous` NORMAL/FULL — écrire vers chemin temporaire + rename atomique, cible partielle =
  invalide ; (3) snapshot ponctuel, PAS du PITR (Litestream/LiteFS font du page-shipping, autre
  problème). Le choix VACUUM INTO reste ALIGNÉ pour un backup ponctuel mono-fichier.
- **S1A-R4 [P2] Couverture upcaster PAR-HOP.** Pattern Axon UpcasterChain confirmé (transform
  lazy à l'interprétation, chaque upcaster franchit UN cran x→x+1, chaîne ordonnée par version) ;
  le golden unique « une forme antérieure » ne prouve pas une chaîne >1 version → graver la FORME
  du test : un golden + bras rouge par cran (trivial aujourd'hui à 1 cran, la forme évite la
  dette au 2e bump).
- **S1A-R5 [P3] Inbox : `processed_at` + correction citation.** Standard idempotent-consumer =
  message_id UNIQUE + `processed_at`, insert dans la MÊME txn. Le précédent interne S76 D réel =
  `UNIQUE (task_id, worker_id)` + `INSERT OR IGNORE` (`nexus-coordinator-rs/src/db.rs:120-128,
  543-551`, test `validator.rs:864-892`) — colonne `worker_id`, PAS `worker_pubkey` (à rectifier
  dans S2A-10).
- **S1A-R6 [P3] Corrections de fait version + hygiène i64.** Le fix CVE-2025-6965 (SQLite 3.50.2)
  est disponible dès rusqlite **0.37.0** (release notes, constat 2026-07-22) — la note QPO-3
  « bump 0.40 » (§12) est corrigée en « >=0.37.0 ». rusqlite 0.38.0 désactive par défaut
  `ToSql/FromSql` pour `u64`/`usize` → typer les colonnes entières NEUVES (epoch, timestamps,
  seuils) en **`i64`** natif SQLite, 0 coût, forward-compatible QPO-3. Aucun bump maintenant.
- **S1A-R7 [INFO] Delta SOTA fenêtre 2026-07-17→22 : RIEN.** Aucune release rusqlite ni advisory
  SQLite/rusqlite dans la fenêtre ; RUSTSEC-2026-0001 = rkyv (non applicable, store en
  serde_json TEXT) ; Obelisk « SQLite is all you need » (2026-06-26) renforce la thèse.

### Complétude rejouée — ce que le preflight initial MANQUAIT (13 findings)

- **ADD-C1 [P1] Inbox livrée mais NON TESTÉE.** Ni I.3 (plan) ni §11.3 ne listaient de test
  inbox — une garantie neuve (dedup at-least-once) partait sans preuve ni bras rouge.
- **ADD-C2 [P1] Mécanique daemon omise : golden_http_* + compte de routes.** `test_support.rs:
  351-945` (19 `golden_http_*` + `golden_families_stable`), carry A1 de CE sprint = 0-golden
  familles FERMÉ ; un module API neuf à 0 golden les RÉOUVRE. Front-matter LOOPBACK cadenassé
  « 89 routes réelles » (`grep -c ".route(" http.rs` = 89 vérifié) : ajouter N routes doit
  mettre à jour 89→89+N + l'oracle §P75 « routes 89==89 ». [routé **Phase I1**, cf. adaptation 22]
- **ADD-C3 [P1] Bras rouges manquants sur 4 des 6 garanties neuves.** Seuls upcast et snapshot
  avaient un contrôle discriminant ; lease fencing, GC borné, refus downgrade, inbox n'en
  avaient AUCUN — trois faux-verts potentiels dans la phase dont le sujet EST le faux-vert
  (doctrine `feedback_mutation_before_trust` + `fault.rs:27-31`).
- **ADD-C4 [P2] FaultPoints NEUFS requis.** `fault.rs:22-50` n'a que les 6 frontières E ; les
  transactions composites neuves (lease-acquire, inbox-insert, snapshot-write, GC-delete) ont
  besoin de leurs points de faute intra-transaction, sinon leur atomicité n'a aucune fenêtre de
  crash falsifiable.
- **ADD-C5 [P2] `restore` sans test ni mécanisme.** I.2:752 livre backup/**restore**/GC ; le
  restore n'apparaissait que comme menace/tier. Mécanisme à définir (store FERMÉ, swap fichier,
  ré-ouverture pragmas relus + refus downgrade, gestion `-wal`/`-shm`) + test round-trip
  backup→restore→replay identique.
- **ADD-C6 [P2] Harnais golden mal référencé.** `golden_events_replay_stable` doit réutiliser
  `assert_golden` STRICT de `tests/store_spike.rs:622-688` (UPDATE_SNAPSHOTS + fail-on-missing +
  round-trip typé, goldens réels `src/run_event*.golden.json`) — PAS la mécanique golden_http
  §P75 du daemon (autre mécanisme, qui vaut pour I1).
- **ADD-C7 [P2] Bras out-of-process pour le lease.** Le crash-during-lease sous mort RÉELLE du
  détenteur (owner mort → expiry → takeover epoch+1) est la seule preuve honnête du fencing ;
  un halt in-process ne prouve pas l'expiration d'un lease dont le détenteur est vraiment mort
  (`store_spike.rs:947-1082` ne couvre que B1/B2/B3).
- **ADD-C8 [P2] Refactor `spike-faults` sous-spécifié.** `execute()` EXIGE `faults: &dyn
  FaultHook` en signature de PRODUCTION (`engine.rs:239-244`) et `NoFaults` est pub (`lib.rs:
  125`). Il faut une entrée de production SANS FaultHook (sinon l'adaptation 12 n'est pas
  tenable) sans casser le harnais E — décision d'API à trancher avant le 1er Edit (B7).
- **ADD-C9 [P2] Re-coupe I1 jamais évaluée.** plan:743 désigne I « candidate à re-coupe
  suffixée (I1) » ; le preflight empilait 15 adaptations + 7 fichiers sans jamais évaluer un
  split. Tranché en B6 (adaptation 22).
- **ADD-C10 [P3] Livrable « traits `store` » non analysé.** `store.rs:202` = struct concrète ;
  D3:366 liste les traits comme seam hôte-neutre. Tranché en B8 : différé à J (le consommateur
  réel est le reducer J ; une abstraction sans consommateur = sur-investissement S2A-9),
  scope cut daté au body I.
- **ADD-C11 [P3] Refus downgrade sans test dédié.** Replié dans `old_run_replays_after_upgrade`
  sans bras rouge propre → test nommé + bras rouge (recoupe C3).
- **ADD-C12 [INFO] Middleware d'auth nommé.** Routes daemon neuves = bloc `authed_routes`
  (`http.rs:285-622`, layer `auth_required` bearer + Host + Origin) ; `public_routes` (:263)
  contourne l'auth — un enregistrement au mauvais endroit est un auth-bypass silencieux. [I1]
- **ADD-C13 [INFO] integration-nightly N-A à déclarer.** Engine hermétique + routes store-level :
  aucun chemin relay/multi-daemon — le déclarer au body pour fermer l'arbre CI honnêtement.

### Faits-datés rejoué — verdict sur l'exactitude du preflight

Tableau §2 rejoué ligne par ligne + mesures + git + compteurs : **TOUT EXACT sauf 1** —
la claim « `git log --grep=DEVIATION/rejected` vide » (corrigée §1 : le hit `rejected` est un
nom de test Phase G, faux positif). Précisions : chemin exact `docs/rust/PATTERNS.md` (§P75) ;
census 29 = 28 `pub const` + `DOMAIN_TRACE_EVENT_V1` déclaré à son point d'usage
(`canonical.rs:364`) — pas de gonflage. SQLite bundlé 3.49.2 re-vérifié dans le registry cargo
local ; compteurs 2251/2255+1skip conformes au body H ; `params!` partout confirmé (les 8
`format!` de store.rs sont des messages d'erreur, jamais du SQL).

---

## 10. Verdict G8

**PLAN-ADAPT.**

Justification (§4.5.7) — **aucune décision Day-0 (D3 GELÉE) ni aucun invariant du plan n'est en
tension structurelle** : mémoïsation-de-steps + journal append-only, PK `(run_id, sequence)`, clé
d'idempotence `(run_id, node_instance_id, effect_id)`, outbox dans la MÊME transaction que
l'avancement, `effect_id` stable / `attempt_id` change, effet non idempotent sans reçu →
`outcome_unknown` jamais retry auto, enveloppe versionnée UNE fois + payload `Value` + upcasters à
l'interprétation, snapshots jetables, fencing single-writer, run store = une classe de plus dans
l'inventaire migrations, doctrine sécurité checkpoints — restent exactement ceux du kickoff
`:330-423` et du scope I.1 `:734-744`. Le prior art OSS (déjà gelé D3) valide ces choix. Le verdict
n'est **pas DESIGN-CONFLICT** : aucune alternative rejetée (replay-total, wasmtime/Obelisk, cqrs-es,
redb) n'est ré-ouverte, aucune dep neuve ni bump, aucun arbitrage PO requis ; toutes les corrections
sont des précisions appuyées sur une evidence code/OSS rejouée à HEAD. Le verdict n'est **pas
EXECUTE** parce que le texte du plan I laisse l'upcaster et la garde de downgrade non câblés,
n'inscrit aucun amendement THREAT_MODEL, ne mentionne pas la clôture docs-contrat des routes neuves,
et laisse la décision stub `RunEvent.v1` + le choix backup implicites. Il n'est **pas
SCOPE-CUT-CONSISTENT** : rien n'est retiré — le périmètre est conservé et précisé.

### Adaptations que le code DOIT suivre (autorité : ce préflight)

1. **Upcaster + garde downgrade** — insérer l'étape upcaster à l'interprétation dans `load_events`
   (`store.rs:654` : lire payload comme `Value`, transformer `Value→Value`, puis parser
   `RunEventKind`) ; refuser fail-closed `format_version > RUN_EVENT_FORMAT_VERSION`. Test
   `old_run_replays_after_upgrade` avec golden d'une forme ANTÉRIEURE synthétique + **contrôle rouge
   discriminant** (retirer l'upcaster casse le test) + un test de refus de downgrade dangereux.
   [S3-02, S4-02→S4-06, S2A-2]
2. **Décision stub core `RunEvent.v1`** — remplir l'identité schemars dans `sbfb-workflow-core` (comme
   H a rempli `CompiledPlan.v1`) : refresh `contracts.schema.json` (`UPDATE_SNAPSHOTS=1`, même commit),
   `len==7` conservé (corps 5/7→6/7), census 29 inchangé ; ajouter la dép intra-workspace
   `sbfb-workflow-core` à engine ; amender `event.rs:12-17`/`lib.rs:98-109` (rationale GO/NO-GO expiré)
   et réconcilier `event.rs:2` « v0 ». **Recommandé : remplir** ; sinon, justifier explicitement la
   rupture de cadence. Le body I ne revendique JAMAIS « registre complété » (6/7, pas 7/7). [S4-02, S2A-5]
3. **Amendement THREAT_MODEL** — asset run store (~A11 : état d'exécution durable = journal+outbox+
   lease+snapshots) + menaces T-RUN-* (store forgé/anti-rollback, downgrade mal-parsé, double-émission
   par lease-spoof, perte de preuve GC, snapshot poisoning, SQLi/RCE fermées par paramétré+
   données-pures) + trust boundary (frontière OS mono-utilisateur, input non-fiable = SPEC/pack vérifié
   Ed25519 en amont) + résiduel PII identifiants ; amender §19 + refléter dans I.5. [S3-01, S3-04, S3-05,
   S3-07, S3-08]
4. **Lease single-writer** — colonne/table lease (owner token + expiry monotone) via `M::up` forward,
   vérifiée DANS la transaction d'avancement ; `single_writer_lease_fencing` prouve qu'un writer
   fenced-out n'émet PAS d'outbox. Modéliser lease-spoof + crash-during-lease. [S2A-6, S3-08]
5. **inbox net-neuf** — table inbox dedup (at-least-once + dedup, réutiliser le précédent
   `(worker_pubkey, task_id)` S76 D, pas recoder) via `M::up`. [S2A-10]
6. **Snapshots jetables** — enforced par construction (réduction re-dérivable du log + invalidation
   par `snapshot_version`), JAMAIS autorité ; `snapshot_is_disposable_cache` avec **contrôle rouge
   discriminant** (snapshot empoisonné détecté/ignoré, état re-dérivé identique). [S3-06, S2A-9]
7. **Backup via `VACUUM INTO`** — SQL pur (chemin paramétré/validé), JAMAIS l'API `Connection::backup`
   (ajouterait `features=["backup"]` au rusqlite workspace partagé par 5 crates = feature-change à
   interdire). [S1b-02]
8. **Rétention/GC explicite + testée** — ne jamais purger (a) runs en vol, (b) outbox pending,
   (c) diagnostic terminal (RunTerminal/outcome_unknown) ; ne purger que completed/terminal au-delà
   d'un horizon explicite. [S3-07]
9. **`workflow_api.rs` (neuf)** — routes store-level (status/CRUD internes, outbox pending),
   enregistrées dans `build_router`, versées à l'inventaire D7 §3 LOOPBACK avec **tier délibéré par
   famille** (restore/GC/lease-force = candidat T1 perte-d'évidence/destructif ; status/timeline = T0)
   + **clôture docs-contrat** (llms.txt/REFERENCE, canon S80). `run_id` généré/validé côté serveur
   (résiduel PII). Toute donnée front-facing = projection allow-listée, PAS `RunEvent` brut. [S2A-4,
   S3-03, S3-05, S4-05]
10. **Réconcilier enveloppe-typée + payload `Value`** — graver le contrat upcaster (`Value→Value`,
    jamais de code) ; trancher explicitement enveloppe `deny_unknown_fields` + payload extensible.
    [S3-09, S4-07]
11. **Migrations forward-only + inventaire par classe** — toute table neuve (inbox/snapshots/lease) =
    `M::up` appendu à `MIGRATIONS` (jamais éditer l'existant) ; inscrire la classe « workflow run
    store » dans l'inventaire migrations (pattern S82 T) + fixtures golden events committées rejouées
    (`golden_events_replay_stable`, oracle byte-identique §P75). 0 bump, 0 dep. [S2A-7, S2A-8, S4-07]
12. **Fermer la dette E** — sortir FaultHook de la surface publique de production (garde `spike-faults`)
    et rendre `effect_id` non-injectable hors `execute()`, I étant le 1er consommateur réel. [S2A-3]
13. **Doctrine sécurité D3.6** — `params!` partout (déjà tenu), état données-pures JSON, Ed25519
    vérifiée en amont (la route de création de run re-vérifie la signature avant toute création) ;
    ne pas ré-inventer digest/JCS (payload journal `serde_json` non-JCS légitime). [S3-04, S4-01, S4-07]
14. **Ne pas sur-investir SpikeEngine** (`engine.rs`) — superseded par le reducer/scheduler J ; les
    routes I sont store-level, le pilotage d'exécution est J/K. [S2A-9]
15. **Corrections de traçabilité** — citer `Cargo.toml:137-138` (pas 135-136) et `arti-client`
    `Cargo.toml:463-467` (pas 461-465, contrainte active seulement sous `--features tor`) au body I.5 ;
    corriger la prémisse « 6/7 » → 5/7 ; corriger le doc-comment périmé `contracts.rs:8-9` (compiled-plan
    n'est plus un stub) ; aligner le vocabulaire G-STORE (fermé en I). [S1b-04, S1b-05, S4-04, ADD-1, ADD-2]

### Adaptations complémentaires (rattrapage §9bis, même autorité)

16. **Fencing CAS-first** — colonne `epoch` (`i64`) monotone bumpée à chaque acquisition ;
    l'avancement (append outbox/journal) fait `UPDATE … WHERE owner = ? AND epoch = ?` DANS le
    même `BEGIN IMMEDIATE` et exige `rows_affected == 1` sinon abort fail-closed ; l'expiry
    (heuristique de takeover SEULEMENT) n'est JAMAIS la condition de commit ; jamais de
    lease-check en read-txn séparée (piège `SQLITE_BUSY_SNAPSHOT`). Bras rouges : owner périmé
    volé qui échoue à émettre même si son horloge le croit titulaire, ET owner réellement MORT
    (bras out-of-process, takeover epoch+1). [S1A-R1, S1A-R2, ADD-C7]
17. **Backup/restore complets** — `VACUUM INTO` sur connexion sans txn/statement ouverts, écrire
    vers chemin temporaire + rename atomique, cible partielle = invalide, refuser
    `synchronous=OFF` ; snapshot ponctuel ≠ PITR (documenté). Restore = mécanisme DÉFINI (store
    fermé, swap fichier, ré-ouverture avec pragmas relus + refus downgrade, gestion `-wal`/`-shm`)
    + test round-trip backup→restore→replay identique. [S1A-R3, ADD-C5]
18. **Bras rouge pour CHAQUE garantie neuve** — inbox dedup (test + mutation qui retire
    `INSERT OR IGNORE`/l'unicité → double-traitement observé), lease fencing (cf. 16), GC (garde
    retirée → purge d'un run en vol/outbox pending/diagnostic terminal OBSERVÉE), refus downgrade
    (test dédié + bras rouge : garde retirée → format futur mal-accepté). FaultPoints NEUFS pour
    les transactions composites neuves (lease-acquire, inbox-insert, snapshot-write, GC-delete).
    [ADD-C1, ADD-C3, ADD-C4, ADD-C11]
19. **Harnais golden : réutiliser `assert_golden` STRICT** de `tests/store_spike.rs` (goldens
    `src/run_event*.golden.json`, UPDATE_SNAPSHOTS + fail-on-missing) pour
    `golden_events_replay_stable` et le golden par-hop de l'upcaster — PAS la mécanique
    golden_http du daemon (qui vaut pour I1). Test upcaster PAR-HOP (forme gravée dès 1 cran).
    [ADD-C6, S1A-R4]
20. **API de production sans FaultHook (B7)** — `execute()` public de prod SANS param `faults`
    (le chemin garanti est le seul compilé hors feature) ; la variante à hooks passe derrière
    `spike-faults` (harnais E compile inchangé via la feature) ; `NoFaults` sort de la surface
    par défaut. `effect_id` non-injectable hors `execute()`. [ADD-C8, S2A-3]
21. **Traçabilité complémentaire** — précédent inbox = `UNIQUE (task_id, worker_id)` +
    `INSERT OR IGNORE` (`worker_id`, pas `worker_pubkey`) ; note QPO-3 : fix CVE-2025-6965 dès
    rusqlite **>=0.37.0** (pas 0.40) ; colonnes entières neuves en `i64` (jamais `u64`/`usize`,
    forward-compat rusqlite 0.38) ; inbox porte `processed_at` ; integration-nightly N-A déclaré
    au body ; chemin exact `docs/rust/PATTERNS.md`. [S1A-R5, S1A-R6, ADD-C13]
22. **Re-coupe DÉCIDÉE : I → I + I1 (B6)** — autorisée par plan:743, motivée par le volume (15+7
    adaptations, 7 fichiers) et la frontière naturelle engine/daemon. **Phase I** = run store
    engine COMPLET (migrations inbox/snapshots/lease, upcasters + refus downgrade, GC/rétention,
    backup/restore, fencing, refactor spike-faults) + remplissage stub core `RunEvent.v1` (B1) +
    THREAT_MODEL A11/T-RUN-* + inventaire migrations par classe — ferme la TOTALITÉ de la
    condition binaire I.4 (G-STORE fermé, G-UPGRADE-REPLAY + G-SINGLE-WRITER prouvés).
    **Phase I1** = routes daemon `workflow_api.rs` + `build_router`/`authed_routes` + golden_http
    famille + compte 89→89+N + inventaire D7 §3 + clôture docs-contrat. Le preflight I vaut socle
    pour I1 (delta-scan léger au démarrage de I1). [ADD-C9, ADD-C2, ADD-C12]
23. **Traits `store` différés à J (B8)** — le consommateur réel du seam est le reducer J ; une
    abstraction sans consommateur contredit S2A-9. Scope cut DATÉ au body I, routé J. [ADD-C10]

---

## 11. Ce que la phase doit faire (actionnable)

### 11.1 À trancher AVANT le premier `Edit`

- **B1 — Stub core `RunEvent.v1`** (adaptation #2) : **TRANCHÉ = REMPLIR** (cadence des 6 autres
  identités, rationale GO/NO-GO expiré ; l'enveloppe serde pure migre vers core, engine dépend de
  core — dép intra-workspace, 0 nœud de lock externe). Le body ne revendique jamais 7/7 (→ 6/7).
- **B2 — Acquittement THREAT_MODEL** (adaptation #3) : I sème l'asset A11 + amende §19 (jamais un
  silence).
- **B3 — Tier D7 des routes** (adaptation #9) : status/timeline = T0, restore/GC/lease-force = T1
  délibéré par famille, AVANT de coder les routes. [routé I1, adaptation 22]
- **B4 — Backup** (adaptations #7 + 17) : `VACUUM INTO` (SQL pur, temp+rename, refus
  synchronous=OFF), JAMAIS `Connection::backup`.
- **B5 — G-STORE fermé entier** (ADD-2) : aligner le body ; G-UPGRADE-REPLAY + G-SINGLE-WRITER
  s'ouvrent-et-prouvent.
- **B6 — Re-coupe I → I + I1** (adaptation 22) : **TRANCHÉ = OUI** ; I = engine + core +
  THREAT_MODEL + inventaire migrations ; I1 = routes daemon + D7 + golden_http + docs-contrat.
- **B7 — API prod sans FaultHook** (adaptation 20) : **TRANCHÉ = OUI** ; chemin garanti seul
  compilé hors feature, harnais E inchangé derrière `spike-faults`.
- **B8 — Traits `store`** (adaptation 23) : **TRANCHÉ = différés à J**, scope cut daté au body.

### 11.2 Modules / fichiers (ordre d'implémentation)

1. **`crates/sbfb-workflow-engine/src/store.rs`** — nouvelles tables `inbox`/`snapshots`/lease via
   `M::up` appendus à `MIGRATIONS` (forward-only, `store.rs:51`) ; étape upcaster + garde downgrade
   dans `load_events` (`:654,660-664`) ; backup `VACUUM INTO` ; GC/rétention paramétrés ; acquisition/
   renouvellement/expiration du lease vérifiés dans la transaction d'avancement. `params!` partout.
2. **`crates/sbfb-workflow-engine/src/event.rs`** — upcasters `RunEvent` v→v (`Value→Value`) ;
   réconcilier enveloppe-typée + payload extensible ; corriger `//! v0` → 1 ; amender le rationale
   « GO/NO-GO ouvert » expiré.
3. **`crates/sbfb-workflow-core/src/contracts.rs`** (si B1 = remplir) — remplir `RunEvent {}` schemars
   pur, `deny_unknown_fields` + `title` conservés, `len==7` intact, corriger doc-module :8-9 ; refresh
   `contracts.schema.json` (`UPDATE_SNAPSHOTS=1`, même commit) ; ajouter dép intra-workspace à engine.
4. **`docs/security/THREAT_MODEL.md`** — asset A11 + T-RUN-* + §19.
5. **Artefact/entrée d'inventaire migrations** classe « workflow run store » + goldens events committés.

Routés **Phase I1** (re-coupe B6, adaptation 22) : `crates/nexus-shell-daemon/src/workflow_api.rs`
(routes store-level, `build_router`/`authed_routes`, `run_id` serveur-validé, golden_http famille,
compte 89→89+N) + `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §3 + clôture docs-contrat
(llms.txt/REFERENCE).

### 11.3 Tests (au-delà des 5 nommés au plan)

- `store::tests::journal_fault_injection` (G-STORE, existant E à étendre) — crash journal/outbox →
  0 perte / 0 double émission, contrôle rouge conservé ; **FaultPoints NEUFS** sur les transactions
  composites (lease-acquire, inbox-insert, snapshot-write, GC-delete) [adaptation 18].
- `store::tests::old_run_replays_after_upgrade` (G-UPGRADE-REPLAY) + **bras rouge PAR-HOP**
  (retirer l'upcaster du cran n casse le replay du cran n) via `assert_golden` STRICT
  [adaptations 1, 19].
- Refus de downgrade : **test dédié** + bras rouge (garde retirée → format futur mal-accepté)
  [adaptation 18].
- `store::tests::single_writer_lease_fencing` (G-SINGLE-WRITER) — writer fenced-out n'émet pas
  d'outbox (CAS epoch `rows_affected == 1`) + **bras rouge owner périmé** + **bras
  out-of-process owner réellement mort → takeover epoch+1** [adaptations 16, 18].
- `store::tests::snapshot_is_disposable_cache` — invalidé/empoisonné → reconstruit du log identique,
  **bras rouge discriminant**.
- Inbox : test dedup at-least-once (`INSERT OR IGNORE` + unicité) + **bras rouge**
  (dédup retirée → double-traitement observé) [adaptations 5, 18].
- Backup/restore : round-trip `VACUUM INTO` → restore → replay identique ; cible partielle =
  invalide [adaptation 17].
- Rétention/GC : test « ne purge jamais runs en vol / outbox pending / diagnostic terminal » +
  **bras rouge** (garde retirée → purge interdite observée) [adaptation 18].
- `store::tests::golden_events_replay_stable` — oracle byte-identique cross-run via
  `assert_golden` STRICT (goldens committés `src/*.golden.json`) [adaptation 19].
- Dual-platform : verts Win natif ET Docker `sbfb-ci` 1.94 (`SBFB_TEST_HTTP_TIMEOUT_SECS=120`).
- T1 = `N-A-no-frontend-change` ; integration-nightly = N-A déclaré [adaptation 21].

### 11.4 Interdits

- Pas de bump `rusqlite`/`rusqlite_migration` ni de dep runtime/dev neuve (`Cargo.toml` intact) ;
  pas de `features=["backup"]` rusqlite (VACUUM INTO exclusif) ; pas de `DOMAIN_*`/wire bump (census 29) ;
  pas de `// FRONTIER:` machine sur les routes run store (non signées, inventaire D7).
- Pas de concaténation SQL (params! exclusif) ; pas de msgpack/pickle dans le payload (enum serde
  typé / `Value` données-pures) ; upcasters = transforms `Value→Value`, jamais de code.
- Pas d'édition destructive d'un `M::up` existant (append-only) ; pas de snapshot traité comme autorité.
- Pas de sur-investissement `engine.rs` (reducer/scheduler = J) ; pas de projection front-facing
  `RunEvent` brut (WorkflowPublicProjection = phase ultérieure) ; pas de touche à `web/`
  (T1=N-A-no-frontend-change).

### 11.5 Invariants à vérifier (commande par invariant)

| Invariant | Commande | Attendu |
|---|---|---|
| census intact | `bash scripts/check-frontier-contracts.sh` | exit 0, `[29 frozen]` |
| registre 7 (6/7 si B1) | `cargo nextest run -p sbfb-workflow-core --locked` | exit 0, `len == 7` |
| store + replay + fencing | `cargo nextest run -p sbfb-workflow-engine --locked` | exit 0 |
| upcaster non vacueux | bras rouge `old_run_replays_after_upgrade` | retirer upcaster → ROUGE |
| snapshot jetable | bras rouge `snapshot_is_disposable_cache` | snapshot empoisonné → détecté/ignoré |
| 0 dep / 0 bump | `git diff Cargo.toml Cargo.lock` | vide |
| schema à jour (si B1) | `UPDATE_SNAPSHOTS=1` puis run sans var | pas de drift |
| SPDX | `bash scripts/check-spdx.sh` | exit 0 |
| dual-platform | `fmt --check` ; `clippy --workspace --all-targets -D warnings` ; `nextest --workspace` ; `test --doc` ; `build -p nexus-shell-daemon --release` | verts Win **ET** Docker `sbfb-ci` (1.94) |

---

## 12. Scope NON couvert par I (frontières J/K/L/L1/P/T — honnêtes)

- **Phase J** — reducer/scheduler déterministe, machine d'états, executors de nodes, cancel honnête,
  branches/parallel/map/join/repeat. `node_instance_id`/`effect_id` définis par le plan (H),
  CONSOMMÉS/incrémentés par le moteur (J/K). I ne sur-investit pas SpikeEngine.
- **Phase K** — `effect_id`/`attempt_id`/reçus/idempotence d'effets runtime, revalidation tool-call.
- **Phase L / L1** — intersection 7 couches / coffre `secret_ref` + binding keystore / HITL signé.
- **Phase P** — SSRF/DNS/redirect.
- **Phase T** — miroir TS / canvas / projection `WorkflowPublicProjection` front-facing / run effectif
  du palier WA-T5.3 (T2 `NOT-RUN` jusqu'à AI2, `plan:774-775`).
- **Slot maintenance QPO-3** — bump rusqlite >=0.37.0 (fix CVE-2025-6965 via SQLite 3.50.2 ;
  corrigé S1A-R6, l'ancienne note « 0.40 » visait trop haut ; attention breaking 0.38 u64/usize)
  sous réserve arti-client, HORS chantier.
