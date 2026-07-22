Mon verdict global est **PARTIEL**. H1 et H2 sont fermés. H3 conserve un contournement sémantique des FK composites, et H4 ne documente pas encore entièrement cette borne. Audit statique de l’arbre courant sur `e034a574`, sans lancer de suite ni mutation : **0 P0/P1, 2 P2**. Ce n’est donc pas encore CLEAN, mais aucun bloquant P1 ne subsiste sous la frontière OS mono-utilisateur explicitement acceptée.

### H1 — CONFIRMÉ

- `index_list('outbox')` récupère bien `unique` et `partial` ([store.rs:2138](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2138)).
- Un index non unique est refusé ([store.rs:2154](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2154)) et `partial != 0` produit `InvalidBackup` ([store.rs:2159](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2159)).
- `index_info` collecte les colonnes, les trie et exige exactement `{node_instance_id, run_id}` ([store.rs:2168](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2168)).
- Le contrôle rouge part bien d’un vrai backup, remplace l’index par un UNIQUE partiel, puis exige l’erreur `PARTIAL` ([run_store.rs:1381](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1381), [run_store.rs:1396](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1396)).

H1 est fermée.

### H2 — CONFIRMÉ

Les rangs PK provenant de `table_info` sont conservés ([store.rs:2057](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2057)). Le code construit ensuite l’ensemble réel des colonnes `pk > 0`, trie réel et attendu, puis exige leur égalité exacte ([store.rs:2077](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2077), [store.rs:2086](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2086), [store.rs:2094](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2094)).

Une PK élargie ajoute nécessairement une colonne à `actual_pk` et est donc refusée. H2 est fermée.

### H3 — PARTIEL

Le triplet annoncé est effectivement vérifié : `target == "runs"`, `from == "run_id"` et `to == "run_id"` ou implicite `NULL` ([store.rs:2110](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2110)).

Mais le code réduit toutes les lignes de `foreign_key_list` à un booléen `fk_ok`. Il ne lit ni l’identifiant ni le rang de la contrainte et ne prouve donc pas qu’il s’agit d’une FK mono-colonne. Simultanément, l’inventaire des colonnes exige seulement la présence des colonnes attendues ; les colonnes supplémentaires restent acceptées ([store.rs:2065](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2065)).

Un schéma peut donc conserver toutes les PK, colonnes et index attendus, puis ajouter :

```sql
-- runs
fk_pad TEXT,
UNIQUE (run_id, fk_pad)

-- inbox
fk_pad TEXT,
FOREIGN KEY (run_id, fk_pad) REFERENCES runs(run_id, fk_pad)
```

La ligne `run_id → runs.run_id` rend `fk_ok = true`. Or `record_inbox_message` n’insère que `run_id`, `message_id` et `processed_at_ms`, laissant `fk_pad` à `NULL` ([store.rs:1460](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1460)). Une FK composite comportant une composante enfant `NULL` ne contraint alors pas `run_id` : un message pour un run inconnu peut être accepté. Ce schéma SQLite valide n’est pas rejeté par `quick_check` ([store.rs:2184](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2184)).

La fermeture robuste consiste à grouper `foreign_key_list` par `id` et à exiger une contrainte composée d’une seule ligne, `seq == 0`, exactement `run_id → runs(run_id)`.

Je classe ce résiduel **P2**, et non P1, car il nécessite un backup forgé dans la frontière OS mono-utilisateur où l’attaquant peut déjà réécrire le store. Mais la claim « vérifiée sémantiquement » n’est pas encore exacte.

### H4 — PARTIEL

Le `THREAT_MODEL` énumère correctement les limites relatives aux types, `NOT NULL`, `CHECK`, défauts, collations et violations FK préexistantes ([THREAT_MODEL.md:1767](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1767)).

Deux écarts restent :

- `STORE_MIGRATION_OPS` omet les violations FK préexistantes dans sa liste des propriétés non comparées ([STORE_MIGRATION_OPS.md:83](/C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:83)).
- Les deux documents affirment encore une vérification des FK vers `runs(run_id)` sans mentionner que l’arité/groupement des contraintes et les colonnes supplémentaires ne sont pas comparés. Le contournement H3 réfute donc la portée sémantique actuelle.

### GAPs restants

- **P0/P1 bloquants : aucun.**
- **P2 — H3 / L7, impact conditionnel L4 :** une FK composite nullable peut passer la validation restore et supprimer silencieusement le refus des runs inconnus.
- **P2 — H4 / L9 :** les bornes documentées restent incomplètes et `STORE_MIGRATION_OPS` omet notamment les violations FK préexistantes.
- **P3 : aucun.**

Aucun élément nouveau ne rouvre L1–L3, L5–L6, L8 ou L10. La clôture complète demande soit le contrôle exact du groupe FK, soit l’acceptation et la documentation explicites de cette limite.

