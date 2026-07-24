# Rapport de vérification croisée — Sprint 83 Phase K

## Verdict global : GAP

Phase K non validable en l’état. Aucun P0 relevé, mais cinq P1 contredisent des garanties centrales de durabilité, annulation, confinement des processus, intégrité des receipts et fencing target-side.

Base contrôlée : `HEAD a23510e828fab9d7e96370d0811c7d3e39ed9316`. Le worktree est resté sale mais stable pendant l’audit. Un fichier modifié absent de la liste fournie a également été vérifié : `crates/sbfb-workflow-engine/tests/run_store.rs:257-265` ne fait qu’actualiser la version supportée à M4.

## 1. Trait `Activity` et chemin `EffectTarget` gelé — CONFIRME

- `Activity` expose bien `start`, `poll`, `cancel`, `runtime_handle` et `reap_orphan`, avec contrat non bloquant : `crates/sbfb-workflow-engine/src/activity.rs:147-195`.
- Le chemin bloquant reste séparé du chemin Activity : `crates/sbfb-workflow-engine/src/orchestrator.rs:1169-1211`.
- Les quatre goldens orchestrateur sont toujours exercés : `crates/sbfb-workflow-engine/tests/orchestrator_goldens.rs:167-184`.
- Aucun diff des goldens journal/orchestrateur par rapport à HEAD ; les quatre goldens orchestrateur et les goldens store passent.

## 2. Retries, backoff, deadlines et budget — GAP

Les bornes nominales sont correctes :

- Backoff exponentiel borné 1–60 s : `orchestrator.rs:121-139`.
- Deadline absolue persistée par attempt depuis la politique du plan : `orchestrator.rs:1279-1289`, résolution exact scope → racine → défaut à `orchestrator.rs:1601-1612`.
- Budget `1 + max_retries_per_node` correctement dérivé : `reducer.rs:813-833`.
- Résolution de scope identique côté orchestrateur : `orchestrator.rs:1614-1623`.

**[P1 K-RETRY-01] Fenêtre de crash rendant un run définitivement inerte.**

`record_attempt_failed` est committé et ferme l’attempt dans une transaction (`store.rs:2502-2544`), puis le timer de retry est inséré dans une seconde transaction (`orchestrator.rs:1541-1547`, `store.rs:2231-2238`). Un crash entre les deux laisse :

1. un failure journalisé ;
2. aucune attempt ouverte ;
3. aucun timer `er:`.

Au resume, tout failure exige un timer déjà fired (`orchestrator.rs:1266-1277`) ; un timer absent vaut `false` (`store.rs:2687-2697`). Le run reste donc `Running/FailedRetryable` et retourne `idle` indéfiniment. Le mécanisme analogue du workflow-timeout répare explicitement son absence à chaque reprise (`orchestrator.rs:608-624`), mais pas le retry.

La claim « timers durables, pacing pur » n’est donc pas vraie sous crash.

## 3. `outcome_unknown` et réconciliation §12.5 — PARTIEL

Chemins normaux confirmés :

- Timeout non idempotent : cancel puis `unknowable_outcome` à `orchestrator.rs:1444-1479`.
- Orphan non idempotent : `reap_orphan` puis unknown à `orchestrator.rs:1416-1438`.
- Défaut J inchangé : `park_unknown_for_reconciliation: false` à `orchestrator.rs:147-174`.
- PARK opt-in ou terminal-first à `orchestrator.rs:1493-1518`.
- `Abandon` ferme bien en `outcome_unknown` lors d’un appel sans crash : `orchestrator.rs:423-438`.
- Un terminal est absorbant au store : `store.rs:1325-1352`; le test de non-conversion est réel à `tests/effects.rs:1113-1132`.

Conclusion précise : un `run_terminal{outcome_unknown}` n’est jamais transformable en succès. Un `node_outcome_unknown` seulement parqué est, lui, transformable par `ResultRecovered` + completion, ce qui est le comportement opt-in attendu.

La correction du receipt est présente : un `ActivityReceipt` reconnu vérifie bien `run_id`, node, `effect_id` et `request_hash` à `orchestrator.rs:441-455`. Le receipt echo brut reste accepté et associé au `pending.request_hash` par la completion door à `orchestrator.rs:464-470`.

**[P2 K-CAUSAL-01] La garde anti-blanchiment peut être préautorisée avant le PARK.**

- Le reducer accepte tout `ReconciliationRecorded(ResultRecovered)` et pose seulement un booléen par node, sans exiger un park antérieur ni faire correspondre l’`effect_id` : `reducer.rs:468-473`.
- `record_reconciliation` est public et ne vérifie que l’état `Running`, pas le PARK : `store.rs:2633-2661`.
- La completion ne vérifie ensuite que la présence de ce booléen : `reducer.rs:376-395`.

La séquence `intent → reconciliation_recovered → park → completion` est donc acceptée. De plus, `reconcile_unknown(ResultRecovered)` écrit reconciliation puis completion dans deux transactions (`orchestrator.rs:457-470`) : un crash entre les deux laisse cette préautorisation durable. Les quatre gardes demandées sont présentes, mais la garantie causale complète ne l’est pas.

## 4. `ActivityReceipt` digest-bound — GAP

La structure et la génération sont correctes :

- Tuple riche, attempt, activité, scope, epoch, facts et output : `activity.rs:198-225`.
- JCS généré à `activity.rs:228-234`.
- Digest calculé et écrit atomiquement avec l’outbox : `store.rs:1142-1196`.
- Décision DS-1 respectée : hash local, sans Ed25519, à `hash.rs:355-361`.

**[P1 K-RECEIPT-01] Le digest n’est jamais vérifié lors de la lecture du matériel.**

`completed_effect` lit seulement `effect_id`, `request_hash` et `receipt` depuis l’outbox (`store.rs:812-844`), sans retrouver ni comparer `receipt_digest`. La résolution aval fait directement confiance à ce matériel (`orchestrator.rs:2055-2081`).

Une altération de `outbox.receipt` après le commit est donc consommée malgré le digest différent conservé dans le journal. Le digest est écrit mais n’est pas une liaison effectivement vérifiée. La row `T-RUN-RECEIPT-FORGE-REPLAY` sur-affirme la mitigation à `docs/security/THREAT_MODEL.md:1831`.

## 5. Effet FS — PARTIEL

La correction du symlink feuille est correcte dans sa borne annoncée :

- Parent canonicalisé et contraint sous la racine : `fs_effect.rs:145-160`.
- Symlink feuille existant refusé : `fs_effect.rs:165-175`.
- Fichier régulier existant autorisé ; `NotFound` autorisé pour la première écriture : `fs_effect.rs:168-177`.
- Test Unix pertinent : `tests/effects.rs:1458-1505`.
- La fenêtre TOCTOU externe et le hardlink feuille sont explicitement documentés : `fs_effect.rs:26-32`, `THREAT_MODEL.md:1832`.

**[P2 K-FS-01] La porte accepte tous les types de feuille non-symlink.**

Le branchement est `Ok(_) => {}` à `fs_effect.rs:170-177`, pas « fichier régulier seulement ». Sous Unix, FIFO, socket ou autre fichier spécial passe la garde. Le `std::fs::read` suivant (`fs_effect.rs:215-219`) peut alors bloquer sur une FIFO, en contradiction avec la couche Activity non bloquante, ou transformer l’effet fichier en interaction IPC. Cette borne n’est pas documentée.

**[P2 K-FS-02] Le collapse idempotent peut devenir inaccessible après le propre write de l’effet.**

La précondition d’état est vérifiée avant le test « contenu déjà identique » (`fs_effect.rs:201-220`). Après un crash post-write/pré-completion sur un fichier suivi, un provider réel reflétant le nouveau `worktree_digest` divergera de l’état attendu et refusera la reprise avant de constater que les bons octets sont déjà présents. Le side output existe alors, mais aucune completion n’est récupérée. Les tests utilisent un provider fixe (`fs_effect.rs:59-67`) et n’exercent pas cette reprise.

## 6. Processus, environnement et kill-tree — GAP

Corrections confirmées :

- `ProcessActivity::new` reçoit bien une `BTreeMap` injectée : `process.rs:199-237`.
- Aucun `std::env`, `env!` ou `option_env!` dans le code de production du crate.
- `env_clear`, map hôte, puis `SBFB_EFFECT_INPUT` appliqué en dernier : `process.rs:257-272`.
- Unix : groupe de processus, ESRCH toléré et garde `pgid <= 1` : `process.rs:133-184`.
- Windows : enfant suspendu, kill explicite sur erreur d’assignation ou erreur remontée par le helper de resume : `process.rs:86-105`.
- La charte de `lib.rs:102-108` est donc vraie pour la lecture d’environnement ambiant.

**[P1 K-PROC-01] La garantie « whole descendant tree, crash included » est fausse sous Unix.**

- Le code ne possède qu’un process group et un `PR_SET_PDEATHSIG` posé sur l’enfant direct : `process.rs:69-84`.
- Un descendant appelant `setsid` sort du groupe et échappe à `kill(-pgid)`.
- `PDEATHSIG` ne constitue pas une liaison du groupe complet.
- Le handle PGID n’est persisté qu’après `activity.start`, donc après le spawn, dans une transaction séparée : `orchestrator.rs:1291-1295`. Un crash dans cette fenêtre laisse notamment macOS sans pdeathsig et sans handle re-jouable.

Le module promet pourtant cancel, timeout et crash pour l’arbre entier à `process.rs:2-23`, et THREAT classe le double-fork escape « fermé » à `THREAT_MODEL.md:1834`.

**[P2 K-WIN-01] La détection de l’échec réel de `ResumeThread` est absente.**

`ResumeThread(thread)` voit sa valeur de retour ignorée, puis `resumed = true` est posé inconditionnellement : `process.rs:413-425`. La branche extérieure tue bien l’enfant si le helper renvoie `Err`, mais le helper traite un retour d’échec de `ResumeThread` comme un succès. La correction « kill si resume échoue » est donc seulement partielle.

**[P2 K-PROC-SCOPE-01] L’environnement n’est pas lié à l’identité durable du process.**

`dedup_scope` contient programme et arguments, mais pas `env` : `process.rs:249-254`. Le request hash ne porte que le nom d’activité et l’input : `orchestrator.rs:1098-1100`. Une reprise avec la même name/program/args mais une map env différente passe les gardes de scope et peut ré-exécuter une sémantique différente.

## 7. Store M4 et ordre `close_open_attempt` — CONFIRME

- Migration M4 forward-only et table `attempts` : `store.rs:241-264`.
- PK et FK : `store.rs:251-262`.
- Validation restore de la table et de ses colonnes : `store.rs:3268-3309`.
- Démarrage refusé sans intent ou avec une attempt déjà ouverte : `store.rs:2380-2438`.
- Completion, outbox et fermeture d’attempt sont atomiques : `store.rs:1142-1196`.
- Chemin terminal non-PARK : `record_terminal` précède bien `close_open_attempt` à `orchestrator.rs:1498-1518`.
- La justification fail-safe est correctement documentée dans le store : `store.rs:2575-2584`.

## 8. Reducer, déterminisme et join — PARTIEL

Confirmé :

- `reduce` reste une fonction de `(plan, events)` : `reducer.rs:263-270`.
- Completion sans intent : `reducer.rs:376-395`.
- Attempt failure sans intent : `reducer.rs:438-455`.
- PARK sans intent : `reducer.rs:457-466`.
- Completion parquée sans reconciliation : `reducer.rs:387-393`.
- `max_retries_for` et `max_retries` ont la même résolution exact → racine → 0 : `reducer.rs:824-833`, `orchestrator.rs:1614-1623`.
- Le reducer stocke volontairement `join_fired` comme présence `()` : `reducer.rs:475-479`.
- La résolution de valeur lit la cohorte gelée : `orchestrator.rs:1932-1979`.

Le verdict reste PARTIEL à cause de **K-CAUSAL-01** : la reconciliation elle-même n’a pas de causalité park/effect.

## 9. `route-error-port` fail-closed — CONFIRME

- Seuls les nodes `FailedTerminal` et effets leaf sont examinés : `orchestrator.rs:739-746`. Aucun succeeded ou non-leaf ne déclenche le close.
- Avec policy route et arête `error`, le node route : `orchestrator.rs:747-750`.
- Sans arête, le run ferme `NodeFailed` : `orchestrator.rs:750-764`.
- Les deux bras sont testés : `tests/effects.rs:565-623` et `tests/effects.rs:625-665`.

**[P3 K-DOC-01]** Le commentaire contractuel de `TerminalReason::NodeFailed` est périmé : il affirme que sous `route-error-port` ce reason n’est jamais écrit (`run_event.rs:87-94`). La même affirmation fausse est régénérée dans `contracts.schema.json:1792-1795`.

## 10. Cancel §12.7 — GAP

Confirmé :

- Dès `CancelRequested`, la branche ferme sans redispatch d’effet : `orchestrator.rs:670-696`.
- Un `poll` déjà Ready est journalisé via `record_late_completion` : `orchestrator.rs:1723-1748`.
- La late door n’avance pas la frontier : `store.rs:1232-1277`.
- Le terminal cancelled reste absorbant.

**[P1 K-CANCEL-01] Le drain ne tue pas une attempt orpheline.**

Sur reprise, `ProcessActivity::poll` renvoie `UnknownAttempt` si sa map mémoire est vide (`process.rs:277-280`). Le drain traite toute réponse non-Ready par `activity.cancel` puis ferme la row (`orchestrator.rs:1726-1758`). Or `ProcessActivity::cancel` est un no-op si la map mémoire ne contient plus l’enfant (`process.rs:308-312`). Le `runtime_handle` persisté n’est jamais donné à `reap_orphan` dans ce chemin, contrairement au resume normal (`orchestrator.rs:1416-1424`).

Après fermeture de la row et terminal cancel, le groupe survivant n’est plus reaperable par l’engine.

La claim « un résultat fini juste avant le stop est toujours conservé » est également trop forte : si le process passe de Pending à terminé entre le poll et `cancel`, aucun second poll n’existe et son résultat est jeté.

## 11. Idempotence, double-drive et fencing — PARTIEL

- La completion store est exactement-once : le second update d’une outbox déjà applied échoue et la transaction contenant son event est rollbackée (`store.rs:1159-1175`).
- Le double-drive séquentiel vérifie une completion et un side output : `tests/effects.rs:244-295`.
- Les targets FS/process non fencing-aware sont honnêtement nommées comme telles dans `THREAT_MODEL.md:1769`.
- Le chemin Target/J reste couvert par les goldens inchangés.

**[P1 K-FENCE-01] `FencedFileActivity` n’a pas de check-and-append atomique.**

Il lit tout le log et décide fence/dedup (`activity.rs:466-489`), puis ouvre et append dans une opération distincte (`activity.rs:491-504`). Deux writers peuvent lire le même état, tous deux passer la garde, puis tous deux append ; un writer epoch inférieur peut aussi append après un writer supérieur si leurs lectures précèdent les writes.

Les tests de stale writer et double-drive sont séquentiels (`tests/effects.rs:787-832`, `tests/effects.rs:255-295`) et ne prouvent pas cette fenêtre. Le module reconnaît la borne à `activity.rs:25-28`, mais la classification THREAT « fermée target-side » reste trop forte.

## 12. Wire, schema, census et dépendances — CONFIRME

- `RUN_EVENT_FORMAT_VERSION == 1` : `run_event.rs:60-65`.
- Chaîne upcast vide : `run_event.rs:371-382`.
- Quatre kinds présents et discriminants corrects : `run_event.rs:257-329`.
- `NodeFailed`, `ReconcileOrigin::HostOperator`, `Abandon` et `ResultRecovered` présents : `run_event.rs:67-140`.
- Schema régénéré avec les quatre kinds et `node_failed` : `contracts.schema.json:1681-1795`.
- `MAX_DECISION_LABEL_BYTES == MAX_SIGNAL_PAYLOAD_BYTES` : `compile.rs:117-125`, `store.rs:312-314`, test miroir `tests/effects.rs:1455`.
- Dépendances `libc`/`windows` target-gated : `Cargo.toml:32-58`.
- Le diff `Cargo.lock` ajoute seulement les arêtes de dépendance du crate ; les nœuds `libc 0.2.186` et `windows 0.57.0` existaient déjà : `Cargo.lock:4461-4465`, `Cargo.lock:10539-10547`.
- `scripts/check-frontier-contracts.sh` passe avec census DOMAIN figé à 29 ; liste figée à `scripts/check-frontier-contracts.sh:202-255`.

Les erreurs descriptives de schema sont classées dans le livrable documentation, pas comme dérive de forme wire.

## 13. Honnêteté des docs sécurité/workflow — GAP

Claims correctement bornées :

- FS TOCTOU et hardlink explicitement admis : `THREAT_MODEL.md:1832`.
- FS/process explicitement non fencing-aware : `THREAT_MODEL.md:1769`.
- Reroutage des endpoints opérateur vers V cohérent : `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:103-105`.
- Charte environnement vraie : `lib.rs:102-108`.

Claims faux ou trop forts :

- **[P1]** Receipt « digest-bound » présenté comme mitigation contre forge/replay malgré l’absence de vérification : `THREAT_MODEL.md:1831`.
- **[P1]** Attempts cancel toutes stoppées et arbre tué : `THREAT_MODEL.md:1809`.
- **[P1]** Descendant survival classé fermé, notamment double-fork/crash Unix : `THREAT_MODEL.md:1834`.
- **[P1]** Retry durable classé fermé malgré K-RETRY-01 : `THREAT_MODEL.md:1836`.
- **[P1]** Fencing target-side déclaré fermé malgré le check/apply non atomique : `THREAT_MODEL.md:1769`.
- **[P3]** « reducer ET value resolution lisent la cohorte » est faux : le reducer stocke `()` ; claim à `run_event.rs:299-303`, `contracts.schema.json:1750-1768` et `THREAT_MODEL.md:1813`.
- **[P3]** `REFERENCE.md:57-58` qualifie Phase K de « first real producer » de `outcome_unknown`, alors que le spike E en produit déjà à `engine.rs:386-438` et le teste à `tests/store_spike.rs:547-593`.
- **[P3]** La row restore mentionne encore une « implémentation datée J/K » à `THREAT_MODEL.md:1770`, contrairement au reroutage V explicitement corrigé dans l’inventaire D7.

## Validations exécutées

Passes :

- `cargo fmt --all --check`
- `cargo clippy -p sbfb-workflow-core -p sbfb-workflow-engine --all-targets --locked -- -D warnings`
- `cargo test -p sbfb-workflow-core --locked` — 111 tests + doctest compile-fail
- `cargo test -p sbfb-workflow-engine --locked` — suites unit/store/orchestrateur/effects et 4/4 goldens vertes
- `scripts/check-frontier-contracts.sh` — census DOMAIN 29 propre
- Comparaison des goldens avec HEAD — aucun changement

Bornes de validation : exécution sur Windows. `fs_effect_leaf_symlink_refused` et `descendant_process_cleanup_unix` n’ont donc pas été exécutés ; leurs chemins ont été inspectés statiquement. Aucun `cargo nextest --workspace`, doctest workspace complet ou frontend gate n’a été lancé, les P1 statiques suffisant déjà à invalider la phase.

**VERDICT FINAL SPRINT 83 PHASE K : GAP.** Un `## Verdict: PASS` serait matériellement faux tant que K-RETRY-01, K-RECEIPT-01, K-PROC-01, K-CANCEL-01 et K-FENCE-01 ne sont pas fermés ou que les claims correspondants ne sont pas explicitement rétrogradés.

