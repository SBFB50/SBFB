Mon verdict global est **GAP** : je ne transforme pas le `PASS-PENDING` en `PASS`. Les primitives SQLite sont globalement solides, mais j’identifie **0 P0, 3 P1 et plusieurs P2**. Les trois bloquants sont la fenêtre d’effet externe non couverte par le fencing, une course TOCTOU dans le GC pouvant effacer un diagnostic terminal, et une validation de backup insuffisante avant remplacement. Audit strictement statique du working tree sur `e034a574` ; aucune suite n’a été relancée.

**L1 — CONFIRME.**

`RunEvent.v1` est une enveloppe pure serde/schemars, avec version locale `1`, union taggée et `title = "RunEvent.v1"` ([run_event.rs:53–163](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:53)). Le registre reste à sept contrats ; seul `WorkflowPublicProjection` reste vide ([contracts.rs:96–140](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:96)), et le test impose `len() == 7` ([contracts.rs:148](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:148)). Le JSON régénéré contient le titre et les champs requis cohérents ([contracts.schema.json:1405](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1405), [contracts.schema.json:1572](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1572)). L’engine conserve ses chemins par ré-export ([event.rs:17](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/event.rs:17)).

**L2 — CONFIRME.**

La garde `user_version` précède bien `to_latest` ([store.rs:480–488](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:480)). La chaîne de production est vide ([run_event.rs:172–183](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:172)), mais `upcast_payload` impose d’abord `chain.len() == current - 1`, puis refuse version zéro, version future et hop troué ([run_event.rs:252–280](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:252)). `load_events` passe par cette porte unique avant le décodage typé ([store.rs:1027–1064](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1027)).

Les bras statiques couvrent chaîne vide, hop identité, chaque hop, `ChainGap` et `NewerThanSupported` ([run_event.rs:317](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:317), [run_store.rs:144](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:144), [run_store.rs:182](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:182)).

**L3 — PARTIEL.**

Le fencing des **transactions SQLite** est correctement câblé :

- acquisition validée et epoch bumpé à chaque prise ([store.rs:1157–1228](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1157)) ;
- renew et release par CAS, release gardant l’epoch ([store.rs:1235–1302](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1235)) ;
- CAS dans `in_immediate_tx`, sans condition d’expiry ([store.rs:1829–1847](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1829)) ;
- les appels directs unfenced sont limités à acquire/renew/release. Un handle sans lease reste explicitement unfenced ([store.rs:433–445](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:433)).

**GAP P1 : le lease clôt les écritures DB, pas le double effet externe.** Un writer A peut committer l’intent/outbox, rester suspendu avant `target.apply`, puis perdre son lease. B reprend le pending effect sous un nouvel epoch ; A et B peuvent ensuite atteindre `target.apply` ([engine.rs:430–492](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:430)). Le prochain CAS d’A n’arrive qu’à la completion, donc après l’effet ([engine.rs:499–526](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:499)). Le `FileEchoTarget` fait un check-then-append non atomique ([echo.rs:171–188](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/echo.rs:171)) : deux processus peuvent lire « absent », puis appendre la même clé. Il n’y aura pas deux rows outbox, mais la même outbox peut être délivrée deux fois. La déclaration « double effet fermé en mode leased » est donc trop forte.

**GAP P2 :** le bras « processus mort » ne prouve pas que le run finit. Il appelle `record_completion` ([run_store.rs:429](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:429)), qui ne fait qu’avancer `frontier_node` ([store.rs:907](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:907)). La fermeture exige `record_run_completed` ([store.rs:987](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:987)).

**L4 — CONFIRME.**

La table porte bien la PK `(run_id, message_id)`, `processed_at_ms` et la FK ([store.rs:154–160](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:154)). `record_inbox_message` valide `message_id`, emploie `INSERT OR IGNORE` et retourne `inserted == 1` ([store.rs:1339–1358](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1339)). Les bras couvrent duplication, second identifiant, run inconnu et identifiant invalide ([run_store.rs:786](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:786)). Une FK SQLite n’est effectivement pas absorbée par `OR IGNORE`.

**L5 — PARTIEL.**

Les deux sites utilisent bien `snapshot_digest_subject(version, upto_sequence, state)` ([store.rs:1383–1417](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1383), [store.rs:1438–1477](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1438)). Version obsolète → `Stale`, JSON/digest invalide → `Poisoned`, et la porte non vérifiée est sous `spike-faults`.

**GAP P2 : le digest ne lie pas le snapshot au `run_id`.** Le lookup est contextualisé par `run_id` ([store.rs:1441–1445](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1441)), mais le sujet authentifié ne contient que `v`, `upto` et `state` ([store.rs:1875–1884](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1875)). Copier le tuple complet et son digest d’un run A vers la row d’un run B produit `SnapshotRead::Valid` pour B sans connaître ni recalculer le digest. Aucun champ retourné par `Snapshot` n’échappe littéralement au hash, mais l’identité contextuelle du cache, elle, y échappe.

Le journal restant l’autorité et la consommation production étant différée à J, je classe ce défaut P2, à corriger avant usage des snapshots.

**L6 — GAP.**

La politique est explicite et sans `Default` ([store.rs:399–408](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:399)). La requête sélectionne bien les runs clos, datés avant cutoff et après `keep_last` ([store.rs:1531–1550](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1531)). Le pending est vérifié dans la transaction ; les branches stables completed/terminal et le rollback sur fault sont correctement écrites ([store.rs:1556–1614](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1556)).

**GAP P1 : course TOCTOU sur le statut.** `status` est lu dans la liste de candidats hors transaction, puis réutilisé sans être relu dans `in_immediate_tx` ([store.rs:1540–1564](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1540)). Interleaving valide avec deux handles unfenced :

1. GC mémorise le run comme `Completed`.
2. `record_terminal` ajoute `run_terminal` et passe la row en `Terminal` ([store.rs:950–980](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:950)).
3. GC entre ensuite en transaction mais suit sa branche `Completed`, efface tous les events puis la row `runs` ([store.rs:1568–1582](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1568)).

Le diagnostic terminal censé survivre « toujours » est donc supprimable. Il faut revalider `status`, `closed_at_ms` et l’éligibilité dans la transaction de purge.

**GAP P2 :** `min_age_ms` n’est pas validé. Avec une valeur négative, `now_ms.saturating_sub(min_age_ms)` avance le cutoff dans le futur et autorise la purge de runs trop récents ([store.rs:1525–1536](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1525)).

**L7 — GAP.**

Le backup respecte les mécanismes annoncés : destination existante refusée, `synchronous` relu, `VACUUM INTO ?1`, sibling temporaire puis rename ([store.rs:1657–1688](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1657)). La feature Cargo `backup` reste absente. Le restore ouvre en read-only, mappe les lectures et refuse un `user_version` trop récent ; les siblings sont bien supprimés avant la copie ([store.rs:1717–1773](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1717)).

**GAP P1 : « backup validé » est insuffisant pour autoriser un remplacement destructif.** La validation vérifie uniquement `user_version` et l’existence d’une table nommée `runs` ([store.rs:1727–1751](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1727)). Une base SQLite valide portant le `user_version` courant et une table `runs`, mais dépourvue de `run_events`, `outbox`, `lease`, colonnes attendues, etc., passe puis remplace le bon store. `RunStore::open` ne valide ensuite aucun inventaire de schéma lorsque le `user_version` est déjà courant ([store.rs:480–490](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:480)). Il faut au minimum un `quick_check`/`integrity_check` et une validation des tables/colonnes/index critiques.

**GAP P2 :** `backup_into` supprime sans garde tout `dest.tmp` existant ([store.rs:1665–1677](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1665)). Aucun contrôle ne refuse que ce sibling alias la source SQLite ou un autre fichier important. Les alias source/destination/backup/siblings doivent être rejetés explicitement.

**L8 — PARTIEL.**

La séparation `spike-faults` est réelle. `SpikeEngine::new` et `execute` n’exposent pas les knobs/hooks ([engine.rs:228–273](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:228)) ; les variantes de falsifiabilité sont conditionnelles, et les méthodes production du store n’acceptent pas de `FaultHook`. Le diff de `store_spike.rs` est bien un ajustement mécanique de noms/appels, sans changement matériel d’oracle.

**GAP P2 sur B7 complet :** le preflight exige aussi que `effect_id` ne soit pas injectable hors `execute` ([sprint83_phase_i_preflight.md:597](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_i_preflight.md:597)). Pourtant les méthodes production publiques `record_intent_atomic` et `record_completion` prennent encore un `effect_id` fourni par l’appelant ([store.rs:667](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:667), [store.rs:818](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:818)). Le sous-objectif hooks/knobs est fermé ; la dette B7 au sens du preflight ne l’est pas entièrement.

**L9 — GAP.**

La structure documentaire est correcte : A10/A11 sont versés dans la table d’assets ([THREAT_MODEL.md:68](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:68)), §19 contient sept lignes `T-RUN-*` ([THREAT_MODEL.md:1763](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1763)), et l’ancien §19 est devenu §20 sans rupture. L’entrée « workflow run store » mentionne bien forward-only, garde double et `VACUUM INTO` ([STORE_MIGRATION_OPS.md:83](/C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:83)).

Mais plusieurs mitigations sont sur-affirmées :

- `T-RUN-DOUBLE-EMIT-STALE-WRITER` dit « double effet fermé » alors que le CAS n’encadre pas `target.apply` ([THREAT_MODEL.md:1766](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1766)).
- `T-RUN-GC-EVIDENCE-LOSS` affirme que le diagnostic terminal survit « à chaque passe », contredit par la course L6 ([THREAT_MODEL.md:1768](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1768)).
- `T-RUN-SNAPSHOT-POISON` déclare la menace fermée sans mentionner la substitution inter-run permise par l’absence de `run_id` dans le digest ([THREAT_MODEL.md:1769](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1769)).
- `T-RUN-RESTORE-ROLLBACK` qualifie le backup de validé alors que seule une présence minimale de `runs` est vérifiée ([THREAT_MODEL.md:1767](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1767)).
- L’inventaire affirme que les `M::up` existants ne sont jamais édités, ce qui est faux byte-à-byte dans ce diff.

**L10 — PARTIEL.**

Confirmé statiquement :

- `RUN_EVENT_FORMAT_VERSION = 1` reste local ([run_event.rs:53](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:53)).
- `DOMAIN_RUN_EVENT_V1` reste réservé et sans producteur ([canonical.rs:397](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/canonical.rs:397)).
- Le census reste à 29 domaines.
- `Cargo.lock` n’ajoute que `sbfb-workflow-core` aux dépendances de l’engine, sans nouveau package externe ([Cargo.lock:7528](/C:/Users/FlowUP/Documents/Code/nexus/Cargo.lock:7528)).
- Lecture statique : 33 tests du crate, 33 noms contenant `store`, 3 `upgrade_replay`, 4 `single_writer`, 0 `effect`.
- Les nouveaux fichiers Rust portent leur SPDX dès la première ligne.

**GAP P2 : le premier `M::up` n’est pas intact byte-à-byte.** Le SQL Phase E a été réindenté lors de son passage dans le tableau à deux migrations ([store.rs:94–136](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:94)). Comparé à `e034a574:store.rs:51–92`, le literal passe de 1720 à 1876 octets ; les lignes normalisées sont sémantiquement identiques, mais le claim « jamais éditer l’existant » est factuellement faux. La migration est forward-only au sens SQL, pas au sens byte-preservation exigé par le preflight.

**GAPs P0/P1 — bloquants**

- **P0 : aucun.**
- **P1 — L3/L9 :** le lease fence le journal/outbox, mais pas l’effet externe entre intent et completion. Tant que le threat model affirme « double effet fermé », la Phase I ne peut pas passer.
- **P1 — L6/L9 :** statut GC capturé hors transaction ; un diagnostic terminal peut être intégralement supprimé.
- **P1 — L7/L9 :** un faux backup SQLite minimal peut passer la validation puis écraser un store sain.

**GAPs P2/P3 — documentables ou à fermer avant consommation**

- **P2 :** `run_id` absent du digest snapshot.
- **P2 :** `min_age_ms` négatif non rejeté.
- **P2 :** absence de gardes d’alias pour backup/restore et suppression aveugle de `.tmp`.
- **P2 :** le test out-of-process ne ferme pas réellement le run.
- **P2 :** `effect_id` reste injectable via deux APIs production, contrairement au preflight B7.
- **P2 :** migration M1 réindentée, donc non byte-identique.
- **P2 :** corrections correspondantes requises dans §19 et `STORE_MIGRATION_OPS`.
- **P3 accepté :** TOCTOU/symlinks/copie restore non atomique et fermeture cible réellement imposée seulement par le partage Windows ; ces résiduels sont déjà documentés.

