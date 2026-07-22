Mon verdict global est **GAP**. G2 est correctement fermé, mais G1 ne constitue pas encore une validation sémantique complète du schéma restauré, et G3 sur-affirme donc toujours cette protection. Audit statique uniquement, sans suites ni mutations relancées : **0 P0, 1 P1, 1 P2**. Tout n’est pas fermé.

### G1 — PARTIEL

Les ajouts annoncés existent :

- `table_info` conserve le nom et le rang PK, puis vérifie que les colonnes attendues participent à la PK ([store.rs:2057](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2057), [store.rs:2077](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2077)).
- Les quatre tables enfants doivent exposer au moins une FK ([store.rs:2095](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2095)).
- L’index doit porter le bon nom, avoir `unique == 1`, puis exposer exactement les deux colonnes attendues après tri ([store.rs:2114](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2114), [store.rs:2140](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2140)).
- Le nouveau contrôle rouge part d’un vrai backup et remplace bien l’index par un homonyme non unique ; il serait discriminant si le contrôle `unique` était neutralisé ([run_store.rs:1378](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1378), [run_store.rs:1390](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1390)).

Mais trois contournements restent acceptés :

1. `index_list` ne lit que le nom et `unique`; le flag `partial` et le prédicat SQL ne sont jamais inspectés. `index_info` ne révèle que les colonnes ([store.rs:2117](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2117), [store.rs:2140](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2140)). Ainsi :

   ```sql
   CREATE UNIQUE INDEX outbox_one_effect_per_node
   ON outbox(run_id, node_instance_id)
   WHERE state = 'applied';
   ```

   passe les contrôles actuels, mais ne protège aucun doublon `pending`, alors que l’index existe précisément pour garantir une seule outbox par node ([store.rs:128](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:128)). `quick_check` ne réfute pas un index partiel SQLite valide.

2. La PK n’est pas comparée comme un ensemble exact : chaque colonne attendue doit seulement avoir `pk > 0` ([store.rs:2086](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2086)). Une PK élargie comme `(run_id, sequence, discriminator)` passe et permet plusieurs événements pour le même `(run_id, sequence)`.

3. Les FK ne sont pas vérifiées sémantiquement : le code compte les entrées, sans comparer colonne enfant, table cible ou colonne cible ([store.rs:2097](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2097)). Une FK d’inbox portant sur `message_id` plutôt que `run_id` passe, mais invalide la garantie « run inconnu = erreur » portée par `record_inbox_message` ([store.rs:1442](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1442)).

La fermeture P1 est donc seulement partielle. Le test M11 couvre le cas homonyme non unique, pas l’index partiel, la PK élargie ou la FK mal ciblée.

### G2 — CONFIRMÉ

La documentation correspond maintenant au comportement :

- Le Rustdoc distingue correctement `Running → Terminal` de `Completed` absorbant ([engine.rs:113](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:113)).
- La branche memoïsée ne journalise `RunTerminal` que si le statut relu est `Running`, puis retourne toujours `StepHashMismatch` ([engine.rs:346](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:346), [engine.rs:351](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:351), [engine.rs:364](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:364)).
- Le test explicite la supersession Phase I et affirme que le run complété reste `Completed`, sans raison terminale ([store_spike.rs:361](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:361), [store_spike.rs:392](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/store_spike.rs:392)).
- L’artefact historique conserve effectivement l’ancienne assertion terminale ([sprint83_e_durable_spike.json:62](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_e_durable_spike.json:62)); `git diff --quiet e034a574` confirme qu’il n’a pas été réécrit.
- La claim L8 « pur renommage » est explicitement supersédée dans la review courante ([sprint83_phase_i_review_r4.md:21](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i_review_r4.md:21)).

G2 est fermé.

### G3 — PARTIEL

Les documents décrivent correctement la validation de version, l’inventaire, les contrôles PK/FK présents et `quick_check`. Mais ils qualifient l’index de « vérifié sémantiquement » et affirment qu’un schéma étranger est refusé ([THREAT_MODEL.md:1767](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1767), [STORE_MIGRATION_OPS.md:83](/C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:83)). Les contournements G1 réfutent ces formulations.

Les bornes documentées sont aussi incomplètes : outre types/CHECK/defaults, le code ne compare pas `NOT NULL`, l’exactitude complète des PK, les tuples FK, les violations FK existantes, ni le caractère non partiel de l’index. `STORE_MIGRATION_OPS` omet même les défauts alors que la review r4 prétend qu’ils sont documentés dans les deux surfaces ([sprint83_phase_i_review_r4.md:16](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/rounds/sprint83_phase_i_review_r4.md:16)).

Conséquence transverse : **L7 et L9 restent non fermés**. Aucun élément de ce round ne justifie de rouvrir L1–L6, L8 ou L10 indépendamment du risque d’importer, via restore, un schéma qui affaiblit leurs invariants.

### GAPs P0/P1 — bloquants

- **P0 : aucun.**
- **P1 — G1/L7 :** le restore accepte encore un index UNIQUE partiel, une PK élargie ou une FK mal ciblée. Ces formes peuvent casser silencieusement l’unicité outbox, l’ordre du journal ou la dédup inbox après remplacement d’un store sain.

### GAPs P2/P3 — documentables

- **P2 — G3/L9 :** `THREAT_MODEL` et `STORE_MIGRATION_OPS` sur-affirment la validation sémantique et n’énumèrent pas les bornes restantes.
- **P3 : aucun.**

