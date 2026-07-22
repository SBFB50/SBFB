Mon verdict global est **GAP**. Les fermetures F3 et F4 sont complètes, et l’invariant absorbant de F1 est correctement implémenté. En revanche, F2 ne valide toujours pas les contraintes réelles du schéma restauré. F1 conserve aussi une incohérence documentaire après son changement de sémantique. Audit statique uniquement, sans relancer les suites ni les mutations : **0 P0, 1 P1, 2 P2**. Tout n’est donc pas fermé.

### F1 — PARTIEL

Le mécanisme de code est **confirmé** :

- `record_terminal` n’accepte que `status='running'`, exige `rows_affected == 1`, puis ajoute l’événement dans la même transaction ([store.rs:1026](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1026), [store.rs:1041](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1041), [store.rs:1055](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1055)).
- `record_run_completed` possède la garde symétrique et retourne `InvalidTransition` sinon ([store.rs:1077](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1077), [store.rs:1082](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1082), [store.rs:1092](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1092)).
- Une erreur après l’`UPDATE` provoque bien le rollback de toute la transaction ([store.rs:2210](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2210)).
- Ce sont les deux seuls `UPDATE runs SET status` de la production.
- Sur la branche memoïsée, l’engine ne journalise `RunTerminal` que si le run est encore `Running`, puis retourne toujours `StepHashMismatch` ([engine.rs:344](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:344), [engine.rs:349](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:349), [engine.rs:362](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:362)).
- Le test vérifie les deux transitions interdites et la survie du statut terminal après GC ([run_store.rs:1300](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1300), [run_store.rs:1326](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1326)). M9 est donc statiquement discriminante.
- Le golden utilise désormais un vrai run `Running` après crash B1 ([store_spike.rs:716](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:716)), et épingle les deux hashes dans `RunTerminal` ([run_event_terminal.golden.json:23](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/run_event_terminal.golden.json:23)).

La fermeture reste **partielle** côté contrat documentaire :

- Le Rustdoc de `EngineError::StepHashMismatch` affirme encore « The run is terminal » ([engine.rs:113](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:113)), alors que le replay divergent d’un run complété le laisse explicitement `Completed` ([store_spike.rs:389](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:389)).
- Le commentaire du test fait la même affirmation erronée ([store_spike.rs:361](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:361)).
- L’artefact Phase E conserve historiquement l’ancienne assertion terminale ([sprint83_e_durable_spike.json:62](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_e_durable_spike.json:62)). Il se déclare enregistré au commit E, donc il ne faut pas réécrire son histoire, mais sa supersession doit être explicite : son validateur ne lie que la forme du test, pas ses assertions ([store_spike.rs:847](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:847)).

Conséquence transverse : la clause initiale L8 « harnais E adapté par pur renommage sans changement sémantique » n’est plus vraie après Round 3. La surface API de production de L8 reste correcte, mais cette phrase doit être retirée ou marquée supersédée.

### F2 — GAP

Les ajouts annoncés existent : inventaire de colonnes par version ([store.rs:1972](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1972)), `table_info` typé ([store.rs:2057](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2057)) et `quick_check` ([store.rs:2092](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2092)).

Mais la validation reste insuffisante :

- `table_info` ne conserve que le nom de colonne, `row.get(1)`. Types, `NOT NULL`, PK et valeurs par défaut sont ignorés ([store.rs:2057](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2057)).
- Aucun `foreign_key_list` ou `foreign_key_check` n’est effectué.
- L’« index unique » est validé uniquement par `type='index' AND name='outbox_one_effect_per_node'` ([store.rs:2075](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2075)). Son unicité, sa table, ses colonnes et son éventuel prédicat ne sont jamais inspectés. Or l’invariant attendu est précisément `UNIQUE ON outbox(run_id, node_instance_id)` ([store.rs:128](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:128)).
- Le test forgé crée lui-même un index sur la mauvaise colonne `outbox(x)` ([run_store.rs:1358](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1358)) ; il est refusé par les colonnes étrangères avant que la sémantique de l’index soit éprouvée. M10 prouve donc la boucle de noms de colonnes, pas l’index attendu.

Contre-exemple statique : six tables portant toutes les colonnes attendues, mais une `inbox` sans PK/FK et un index homonyme non unique ou posé sur `effect_id`, passent les contrôles actuels. À `user_version=2`, aucune migration ne rétablit les contraintes ([store.rs:494](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:494)). `record_inbox_message` dépend pourtant entièrement de la PK et de la FK pour dédupliquer et refuser un run inconnu ([store.rs:1442](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1442)). La dégradation peut donc être silencieuse, pas « loud au premier read ».

Les claims « index unique » et « types menteurs échouent loud » sur-affirment le code dans [THREAT_MODEL.md:1767](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1767) et [STORE_MIGRATION_OPS.md:83](/C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:83).

### F3 — CONFIRME

La fenêtre `keep_last` est bien reclassée dans chaque transaction de purge :

- Le scan initial ne transporte que les `run_id`, avec le tri `closed_at_ms DESC, run_id DESC` ([store.rs:1659](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1659)).
- Statut et date sont relus sous `BEGIN IMMEDIATE` ([store.rs:1686](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1686)).
- Le nombre de lignes plus récentes est recalculé avec exactement le même ordre et les mêmes critères d’éligibilité ([store.rs:1714](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1714)).
- Pending et branche `Completed`/`Terminal` sont ensuite décidés depuis le même snapshot transactionnel ([store.rs:1739](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1739)).
- Le test re-date effectivement la victime entre scan et transaction, puis vérifie qu’elle est épargnée ([run_store.rs:1378](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1378)).

Aucun nouvel interleaving réfutant cette fermeture n’est visible.

### F4 — CONFIRME

Après validation de version et parsing JSON, `u64::try_from(seq)` classe toute séquence négative en `SnapshotRead::Poisoned` avant toute construction de `SnapshotRead::Valid` ([store.rs:1560](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1560), [store.rs:1568](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1568)). Le test altère réellement la row à `-1` et exige exactement `Poisoned` ([run_store.rs:1342](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1342)).

Impacts L1–L10 : L5 et L6 sont désormais confirmés sur ces fermetures ; L7 reste **GAP** via F2 ; L8 est **PARTIEL** uniquement sur l’ancienne claim « pur renommage » ; L9 reste **GAP** tant que les deux docs restore sur-affirment la validation. Aucun nouveau doute substantiel sur L1–L4 ou L10.

### GAPs P0/P1 — bloquants

- **P0 : aucun.**
- **P1 — F2/L7/L9 :** restore accepte encore un schéma portant les bons noms de tables/colonnes mais dépourvu des contraintes requises, ou un index homonyme non unique/mal ciblé. Cela peut casser silencieusement inbox, fencing et unicité outbox après remplacement du store sain.

### GAPs P2/P3 — documentables

- **P2 — F1/L8 :** Rustdoc `StepHashMismatch`, commentaire du test et articulation avec l’artefact historique E ne reflètent pas complètement la nouvelle sémantique absorbante.
- **P2 — F2/L9 :** `THREAT_MODEL` et `STORE_MIGRATION_OPS` annoncent un index unique validé et un échec aval systématiquement loud, garanties absentes du code.
- **P3 : aucun.**