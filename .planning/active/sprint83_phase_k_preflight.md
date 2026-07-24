# Preflight G8 — Sprint 83 Phase K — Effets, retries et idempotence

Date : 2026-07-23. Tip au scan : `a23510e8` (Phase J DONE). Arbre propre.
Méthode : Workflow ultracode `wf_0a6d98fa` — 9 agents Opus 4.8 1M (6 scans
factuels S0-socle / S1a-OSS / S1b-deps / S2-histoire / S3-threat / S4-wire +
3 lentilles adversariales réfutation / complétude / design-conflict),
1 377 719 tokens, 230 tool uses. Synthèse main-thread. Rapports bruts :
scratchpad session `preflight_k/`.

## Verdict : PLAN-ADAPT

Le scope K.1/K.2 est intégral (AM-4 ne l'ampute pas — vérifié
`sprint83_plan.md:2315`) et le socle E/I/J le porte déjà aux deux tiers.
Mais le plan a été écrit AVANT que E–J n'existent : six adaptations
evidence-groundées s'imposent, plus deux décisions-à-signaler qui ont
chacune un défaut in-charte (aucun arbitrage PO bloquant si les défauts
sont pris ; l'option lourde de l'une ou l'autre basculerait la phase en
DESIGN-CONFLICT). Vérification adversariale : 12/12 claims load-bearing
CONFIRMÉES en ré-ouvrant les fichiers, 0 réfutée sur le socle ; 1 hypothèse
de scan réfutée (réemploi du tag `DOMAIN_PACK_INSTALL_RECEIPT_V1` — voir
DS-1).

## 1. Socle vérifié (ce que K n'a PAS à construire)

- **Clé d'idempotence** : `IdempotencyKey(run_id, node_instance_id,
  effect_id)` `hash.rs:167-225` = byte-pour-byte le triplet K.2 ;
  `derive_effect_id` engine-derived stable `hash.rs:321-330` (obligation
  E/I : jamais injectable — préservé).
- **Outbox transactionnel** : `store.rs:115-135`, PK
  `(run_id,node_instance_id,effect_id)` + UNIQUE INDEX
  `outbox_one_effect_per_node` (`:134`) = au plus une intention durable
  par node, fail-closed. `record_intent_atomic` (`store.rs:838`, appelé
  `orchestrator.rs:818`) / `record_completion` (`store.rs:1013`, appelé
  `orchestrator.rs:833`) — carry dead_code I1 bien RÉSORBÉ en J.
- **`outcome_unknown` RUN-level déjà câblé** : `orchestrator.rs:799-813`
  → `record_terminal(TerminalReason::OutcomeUnknown)` ; états clos
  ABSORBANTS par le store (`store.rs:1231-1258`, garde
  `WHERE status=Running` → `InvalidTransition`).
- **Vocabulaire journal effets déjà dans RunEvent.v1** :
  `EffectIntended{effect_id}` `run_event.rs:131`,
  `EffectCompleted{effect_id, receipt_digest}` `:137` ; journal
  digest-only par TYPE (`run_event.rs:35`). `TerminalReason` = 3 variantes
  `:67-84`. `RunEventKind` = 10 variantes `:126-214`.
- **États node K déjà TYPÉS jamais dérivés** : `NodeState::{Dispatched,
  FailedRetryable, FailedTerminal, OutcomeUnknown}` `reducer.rs:96-120`,
  doc-comment « the effect layer of Phase K owns them ».
- **Infra timers réutilisable pour poser les deadlines** :
  `schedule_timer` (ABSOLUTE, INSERT OR IGNORE 1er-gagne) `store.rs:2124`,
  `due_timers :2147`, watermark anti-régression `:2048` ; précédent
  per-run `WORKFLOW_TIMEOUT_TIMER_ID` `orchestrator.rs:75,342-352`.
  NUANCE (adv-complétude) : `fire_timer_with_cancel` `store.rs:2168-2189`
  est spécifique cancel-RUN — le firing per-node (timeout effet, backoff)
  = logique NEUVE.
- **`record_late_completion` PRÊTE, 0 appelant prod** (`store.rs:1119-1184`,
  frontier JAMAIS avancée `:1181` ; seule référence = commentaire
  `orchestrator.rs:400`) — la porte §12.7 que K doit câbler.
- **Harnais out-of-process réutilisable** : self-reexec `current_exe` +
  env + `exit(87)` (`tests/run_store.rs:373-476` + 2 miroirs).
- **`attempt_id` = 0 occurrence dans tout le dépôt** (grep) — 100 % neuf.
- **Single-writer intact pour K** : toute écriture store passe
  `in_immediate_tx` epoch-fencé (`store.rs:3133-3152`) — timers/retries K
  restent dans le lease écrivain J. Seul non-fencé : `target.apply`
  (`orchestrator.rs:831`) = la fenêtre effet-externe, carry central.

## 2. Adaptations PLAN-ADAPT (le code suit CES corrections)

**PA-1 — Trait `Activity` NEUF start/poll/cancel, à côté du `EffectTarget`
gelé.** « Traits activity complétés » (K.2) ne peut pas muter
`EffectTarget::apply` (`echo.rs:94-98`) : `engine.rs:501` est le proof
GELÉ de Phase E (`orchestrator.rs:721-725` « must not disturb it »). Et le
modèle bloquant actuel rend INATTEIGNABLES trois livrables K (lentille
complétude, P1) : (a) un « résultat tardif après cancel » n'a aucune
fenêtre d'existence si `apply` bloque et que le cancel est vérifié en tête
de drive (`orchestrator.rs:393`) ; (b) tuer un arbre de processus PENDANT
un run exige que le dispatch rende la main ; (c) le fencing token
target-side doit être porté par la signature du trait. ⇒ trait `Activity`
non-bloquant (start → poll → cancel), consommé par le SEUL orchestrateur ;
`EffectTarget`/`FileEchoTarget` restent la cible echo du spike (le harnais
echo peut implémenter les deux).

**PA-2 — Fencing token propagé et vérifié CÔTÉ CIBLE (carry central,
closure datée K).** `lib.rs:68-72` + `THREAT_MODEL.md:1768`
T-RUN-DOUBLE-EMIT-STALE-WRITER : la fence CAS couvre les écritures store,
PAS la fenêtre intent-committé→apply d'un writer déposé. K propage l'epoch
de lease dans le contexte d'activité ; la cible vérifie le token dans SON
backend de dédup (enregistrement epoch-monotone atomique avec le side
effect) et refuse un apply déchu. Test dédié « writer déchu rejeté À LA
CIBLE ». La row THREAT passe de « bornée (dedup cible) » à « fermée
(token vérifié cible) » — formulation honnête : la fermeture vaut pour
une cible dont le check-and-apply est atomique dans son backend.

**PA-3 — `outcome_unknown` NODE + réconciliation : PARK-avant-terminal,
opt-in politique ; défaut = terminal absorbant J intact (lentille design,
axe 4).** Fait dur : un run terminal `outcome_unknown` est IN-réconciliable
par construction (absorbance store `store.rs:1231-1258`) — §12.5
« réconciliation ou décision humaine » ne peut PAS s'appliquer au terminal
J. Sans adaptation, « réconciliation » est un livrable fantôme (K.4 est
trivialement satisfait par l'existant — lentille complétude P1). ⇒
- Sans politique de réconciliation configurée (DÉFAUT) : comportement J
  inchangé (terminal absorbant immédiat).
- Avec politique explicite (§12.7 « pas propagé sans politique
  explicite ») : le NŒUD est parqué `NodeState::OutcomeUnknown`
  (dérivable, état PRÉ-terminal non-clos, façon WaitingHuman), run vivant ;
  une décision opérateur-local JOURNALISÉE d'origine FERMÉE (miroir
  `CancelOrigin`, jamais un pack ni une sortie modèle) résout : reçu
  récupéré → le run progresse via cette décision attestée (le nœud reste
  étiqueté outcome_unknown, JAMAIS ré-étiqueté succeeded) ; abandon →
  terminal.
- INTERDIT (à inscrire au plan + THREAT) : ré-ouvrir un terminal clos ou
  ré-étiqueter outcome_unknown→succeeded = violation absorbance-I + K.4.
- Décision scheduler explicite : `NodeState::OutcomeUnknown` parqué n'est
  PAS `is_settled()` (`reducer.rs:125-130` ne l'inclut pas — trouvaille
  adv-réfutation C9) mais n'est PAS re-dispatchable (garde explicite,
  sinon boucle).

**PA-4 — Journal des tentatives : RunEventKind SÉPARÉS, jamais un champ
sur EffectIntended/EffectCompleted.** Réconciliation de la tension
S0-F3 (journaliser pour le reducer pur) vs S4 (garder attempt hors
journal) : le reducer DOIT dériver `FailedRetryable`/`FailedTerminal`/
`OutcomeUnknown` node (§12.3) et il est pur sur (plan, journal) — donc les
échecs de tentative qui changent l'état dérivé SONT journalisés, via des
variantes NEUVES (ex. `EffectAttemptFailed{effect_id, attempt, retryable}`,
park, réconciliation), PAS via un champ ajouté aux variantes existantes.
Garanties tenues : « au plus une INTENTION durable par clé » intact (les
events de tentative ne mintent pas d'intent) ; goldens non-retry
byte-identiques (dont le golden E gelé) ; redéfinition v1 pré-launch,
`RUN_EVENT_FORMAT_VERSION` reste 1, chaîne upcast vide. La comptabilité
de scheduling (deadlines, compteur courant) vit dans le STORE : table
enfant `attempts` (l'UNIQUE index `outbox_one_effect_per_node` interdit
N lignes outbox par node — adv-réfutation C6), migration forward-only.
`default_node_timeout_seconds` + `max_retries_per_node` (champs plan
0-consommateur, carry J §3.5) trouvent ici leurs PREMIERS consommateurs,
avec tests nommés.

**PA-5 — Routes daemon mutantes restore/GC/lease-force : re-route
EXPLICITE, pas de defer silencieux.** Les 3 rows D7
(`LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103-105`) sont datées « J/K » mais le
plan K est engine-only (`T1 = N-A-no-frontend-change`, `plan:906`) et J a
shippé 0 route. K met à jour les 3 rows + le front-matter
`inventory_policy` : re-routées vers la phase qui shippe la surface
opérateur (V « Centre des Runs », kickoff l.648, qui porte
cancel/retry/replay/outcome_unknown). Même traitement pour l'authz de la
route de dépôt de signal (carry J SD-1 : principal = L1, route = phase à
route). La réconciliation K reste un geste OPÉRATEUR-LOCAL (précédent
restore/GC I `:1769`).

**PA-6 — Tests K.3 durcis pour être DISCRIMINANTS (mutation-avant-
confiance).** Enrichissements obligatoires du plan de tests :
- `double_command_single_transition` : asserte `side_output_count == 1`
  ET exactement un `EffectCompleted` au journal. Mutations tuées :
  court-circuit memoïsé neutralisé (`orchestrator.rs:737-763`) ; index
  UNIQUE contourné.
- `non_idempotent_without_receipt_becomes_unknown` : cible le code NEUF
  (déclencheur timeout/absence-de-reçu §12.5 `research:1091`), PAS un
  rejeu du chemin pending E/J déjà prouvé (risque de vacuité, lentille
  complétude P1). Mutation tuée : le chemin timeout re-tente au lieu de
  parquer/terminer.
- `fs_effect_bound_to_head_index_worktree` : préconditions = digests
  d'état git INJECTÉS (trait provider host-side ; l'engine ne lit ni
  git ni env — charte `lib.rs:93-99`) ; mismatch → refus + 0 mutation
  FS. Mutation tuée : comparaison de précondition désactivée.
- `descendant_process_cleanup_windows/_unix` : harnais exit87 étendu
  d'un TÉMOIN durable — le petit-enfant détient un lock exclusif/heartbeat
  fsync'd ; le parent (a) attend la preuve de VIE (anti-vacuité), (b) tue
  l'arbre, (c) poll borné sur la cessation (anti-flaky, jamais de check
  PID nu). Atteignable uniquement avec PA-1 (dispatch non-bloquant).
- NEUFS : writer déchu rejeté à la cible (PA-2) ; résultat tardif après
  cancel → `record_late_completion`, frontier intacte (PA-1/§12.7) ;
  réconciliation origine-fermée + jamais-succès (PA-3) ; consommation
  `max_retries_per_node`/`default_node_timeout_seconds` depuis le plan
  compilé (PA-4) ; backoff borné → épuisement → park/terminal, jamais
  retry infini.

## 3. Décisions-à-signaler (défauts in-charte pris ; l'option lourde = DESIGN-CONFLICT à arbitrer)

**DS-1 — « receipts signés (rôle §17.1) » = receipt d'activité STRUCTURÉ
digest-bound LOCAL en K ; l'attestation Ed25519 riche reste Phase O.**
Convergence des 6 scans + 3 lentilles :
- Le rôle §17.1(6) « receipts d'activités locales » n'a AUCUN tag DOMAIN
  réservé ; les 3 tags réservés couvrent les rôles 3/5/4
  (`canonical.rs:395,404,414`). Réutiliser `DOMAIN_PACK_INSTALL_RECEIPT_V1`
  (rôle 4, install/anti-rollback) serait du role-mixing interdit §17.1 —
  hypothèse d'un scan RÉFUTÉE par `canonical.rs:406-414`.
- Le receipt Ed25519 riche (provider/modèle/CLI/exit/digests) est
  EXPLICITEMENT Phase O (`research:2613-2619`, gate
  `G-CLAUDE-CODEX-RECEIPT`).
- Précédent doctrinal des context-strings LOCAUX hors census :
  `hash.rs:25-27` (« not a signing domain, never gossiped … deliberately
  does not follow the DOMAIN_*_V1 naming »).
⇒ DÉFAUT PRIS : K livre la struct `Receipt` canonicalisée JCS liant le
tuple complet `(run_id, node_instance_id, effect_id, attempt, request_hash,
receipt_digest, …)` sous context-string local — la SÉPARATION de rôle que
§17.1 demande — contenu matériel en outbox, digest au journal. 0 DOMAIN
neuf, census 29 INTACT, 0 dep signature, claim `lib.rs:141-146` intacte.
Résiduel documenté (row THREAT) : la signature Ed25519 = O.
OPTION LOURDE (si le PO exige Ed25519 minté en K) : `DOMAIN_ACTIVITY_
RECEIPT_V1` census 29→30 + dep directe + ré-écriture `lib.rs:141-146` =
DESIGN-CONFLICT à arbitrer avant tout Edit.

**DS-2 — Kill d'arbre descendant : FFI target-gated in-crate, 0 nouveau
nœud lock.** std ne suffit PAS (aucun killpg en std, aucune API Job Object
en std ; `taskkill /T`/enum-kill = raceable PID-reuse, ne satisfait pas le
critère binaire). Pattern SOTA déjà in-tree :
`local_worker.rs:396-434` (CreateJobObjectW + KILL_ON_JOB_CLOSE +
AssignProcessToJobObject) + `:215-226` (process_group + PR_SET_PDEATHSIG
Linux-only). ⇒ DÉFAUT PRIS : `windows-sys` (`cfg(windows)`, nœud 0.61.2
DÉJÀ en lock `Cargo.lock:10748`) + `libc` (`cfg(unix)`, nœud 0.2.186 déjà
en lock `:4462`) en deps directes target-gated de l'engine — 0 nouveau
nœud lock, précédent littéral in-crate (`Cargo.toml:26-27` rusqlite
« Already a workspace dependency … adds no new lock node ») et Phase F
core (ed25519-dalek, même rationale). Durcissements : assignation
Job Object sur process CREATE_SUSPENDED puis resume (ferme la course
assign-après-spawn de `local_worker.rs:427`) ; résidu macOS documenté
honnêtement (pdeathsig Linux-only — le chemin EXPLICITE kill(-pgid) au
cancel/resume couvre macOS, seul l'auto-reap sur mort ANORMALE du parent
reste résiduel, comme le documente local_worker). La ligne « 0 dep
runtime » du body citera le rationale — signal PO en une ligne, pas
d'arbitrage bloquant (définition opérante du projet = nœud lock).

## 4. Obligations héritées — traitement dans K (aucun defer silencieux)

| Carry (provenance) | Traitement K |
|---|---|
| Fenêtre effet-externe, fencing token cible (Codex I r1 ; `lib.rs:68-72` ; THREAT `:1768`) | PA-2 — fermé target-side + test writer déchu |
| `record_late_completion` 0-appelant (porte §12.7, J) | PA-1 — routage résultat tardif post-cancel + test frontier-intacte |
| JoinFired{sources} cohorte figée au fire (J §3.1 ; THREAT `:1810`) | Livré K : event `JoinFired` journalisé + reducer/scheduler la consomment ; corrige la row BACKPRESSURE sur-affirmée |
| Borne `decision_label` vs String compilateur (J §3.2, « K/L ») | Livré K : `maxLength` contrat + refus compile (`compile.rs`) alignés sur la borne store ; schema régénéré sous `-p` |
| Squat de nonce / re-ouverture wait (J §3.3 ; T-RUN-SIGNAL `:1805`) | Adossé à PA-3 : la primitive de réconciliation opérateur-local couvre le re-arm (nouveau nonce) ; si le découpage l'exclut, re-route EXPLICITE avec MAJ de row |
| Authz route de dépôt de signal (J SD-1) | PA-5 — re-route vers la phase à surface (V/L1), MAJ row |
| Champs plan 0-consommateur `default_node_timeout_seconds`/`max_retries_per_node` (J §3.5) | PA-4 — premiers consommateurs + tests nommés |
| O(N²) FileEchoTarget « si appelant réel » (E `sprint83_e_durable_spike.json:148`) | Le VRAI adaptateur K est `Activity` (PA-1), FileEchoTarget reste harnais spike : re-déclaration datée (ou index mémoire trivial si touché) |
| Validation causale du journal à la réduction (THREAT `:1808`) | Durcissement K au reducer : completion sans intent / décision sur node non-satisfait = refus |
| effect_id engine-derived non-injectable (E/I) | Préservé — aucune API de clé injectable |
| Routes mutantes D7 restore/GC/lease-force « J/K » | PA-5 — re-route V + MAJ rows `:103-105` + `inventory_policy` |

## 5. Menaces — rows à ajouter (THREAT v26, nouvelle §19.2 « Surface effets »)

T-RUN-RECEIPT-FORGE-REPLAY (S/R — tuple complet + run_id anti-rejeu ;
résiduel signature = O) ; T-RUN-DOUBLE-EMIT-STALE-WRITER (MAJ → fermée
target-side, PA-2) ; T-RUN-FS-EFFECT-ESCAPE (T — confinement worktree
canonicalize + starts_with, refus `..`/symlink ; précondition relue dans
la fenêtre de mutation) ; T-RUN-PROCESS-SPAWN-INJECTION (T/EoP — args
sans shell, env allowlisté, commande bornée au plan compilé, jamais une
sortie modèle) ; T-RUN-DESCENDANT-SURVIVAL (EoP/DoS — Job Object
KILL_ON_JOB_CLOSE / killpg ; handle join au shutdown §P59.7) ;
T-RUN-RECONCILE-LAUNDER (T/R — réconciliation = décision journalisée
origine fermée, jamais ré-label d'un clos ; late-completion non propagé
sans politique) ; T-RUN-RETRY-STORM (DoS — backoff borné + plafond →
outcome_unknown, jamais retry infini). Contraintes suivies (pas de rows
neuves) : T-RUN-TIMER-SKEW (deadlines ABSOLUES watermarkées, lecture
unique d'horloge) ; T-RUN-ID-PII (journal digest-only par TYPE — receipts
jamais en journal) ; duress : AUCUN gate engine (doctrine I1 — reads non
gatés, l'engine n'a pas d'identité) ; §4 DFD + §2 assets + §20 mis à jour
(la couche effets = nouveau composant).

## 6. Impacts wire / census / goldens / frontière

- **0 bump wire** : FEED/ANNOUNCEMENT intacts ; `RUN_EVENT_FORMAT_VERSION`
  reste 1 (redéfinition v1 pré-launch par variantes ADDITIVES, chaîne
  upcast vide).
- **Census DOMAIN 29 INTACT** (DS-1 défaut). NAME-LIST
  `check-frontier-contracts.sh:222-244` non éditée.
- **Goldens** : les 3 `run_event*.golden.json` ne régénèrent PAS tant
  qu'aucun scénario retry n'entre aux fixtures existants (PA-4 variantes
  séparées) ; nouveaux goldens pour les scénarios neufs. Schema core :
  `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core` (piège J : TRIÉ
  sous `-p`, jamais sous workspace).
- **Frontière docs-contrat (test-acteur)** : AUCUNE nouvelle —
  `outcome_unknown` est DÉJÀ projeté et documenté côté daemon
  (`workflow_api.rs:287-296`, `REFERENCE.md:51-57`) ; K le rend
  ATTEIGNABLE sans changer l'allowlist I1. Receipts/outbox restent
  INTERNES (interdiction explicite `workflow_api.rs:29-31`). Les rows D7
  re-routées (PA-5) sont une MAJ de doc existante, pas une frontière.
- **Charte** : core reste PUR (variantes RunEventKind = pure-données) ;
  engine reste synchrone/sans-tokio (Activity non-bloquant = start/poll
  piloté par le drive, pas d'async).

## 7. Signal méta

PLAN-ADAPT consécutifs (I1, J, K) : attendu et déclaré — le plan 43
phases a été écrit avant E–J ; chaque adapt de K découle d'un fait créé
par une phase POSTÉRIEURE au plan (absorbance I, cancel-sans-émission J,
proof E gelé), aucun ne touche une décision Day-0 ni un invariant de
sprint. Le signal reste surveillé.
