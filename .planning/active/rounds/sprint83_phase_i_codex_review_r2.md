Mon verdict global reste **GAP**. L’arbre corrige réellement plusieurs constats du round 1, notamment le cadrage de l’effet externe, le digest lié au run, les gardes GC négatives, l’API `spike-faults` et M1 byte-identique. Mais deux P1 demeurent : le statut terminal n’est pas absorbant et peut donc encore être purgé, et la validation restore ne vérifie toujours pas le schéma réel. Audit statique uniquement, sans relancer les suites : **0 P0, 2 P1, 2 P2**.

### R1 — CONFIRME

La claim est désormais honnête.

Le threat model limite explicitement la fermeture à la « couche STORE, mode leased », reconnaît le mode unfenced et laisse la fenêtre `intent commit → target.apply` à la Phase K ([THREAT_MODEL.md:1766](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1766)). `lib.rs` expose également la course `FileEchoTarget` read-then-append et précise que le fencing ne couvre pas l’effet externe ([lib.rs:48](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:48)).

Cela correspond au code réel : l’intent est committé avant `target.apply`, puis le prochain CAS store n’arrive qu’à la completion ([engine.rs:472](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:472), [engine.rs:485](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:485), [engine.rs:518](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/engine.rs:518)). Le `FileEchoTarget` reste bien non atomique sous concurrence ([echo.rs:171](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/echo.rs:171)).

### R2 — PARTIEL

La correction annoncée est présente :

- le scan ne remonte plus que les `run_id` ([store.rs:1599](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1599)) ;
- `status`, `closed_at_ms` et pending sont relus sous `BEGIN IMMEDIATE` ([store.rs:1626](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1626)) ;
- le test intercale bien `Completed → Terminal` depuis une connexion séparée et vérifie la survie du diagnostic ([run_store.rs:1096](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1096)).

Deux contre-exemples restent cependant.

**P1 — `Terminal` n’est pas absorbant.** Le contrat dit qu’un `RunTerminal` signifie que le run « stopped and will not advance » ([run_event.rs:104](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:104)). Pourtant `record_run_completed` n’exige aucun statut antérieur : il append `RunCompleted` puis remplace inconditionnellement le statut par `completed` ([store.rs:1040](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1040)). Ainsi, même sous un lease valide :

1. `record_terminal` écrit le diagnostic ;
2. `record_run_completed` transforme le run en `Completed` ;
3. GC emprunte la branche completed et supprime tous les events, dont `run_terminal`, puis la row `runs` ([store.rs:1664](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1664)).

Le nouveau re-read ne protège donc pas l’invariant absolu « diagnostic terminal jamais purgé ».

**P2 — le rang `keep_last` n’est pas relu.** `OFFSET keep_last` est calculé uniquement pendant le scan hors transaction ([store.rs:1599](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1599)). Dans la transaction, seule l’éligibilité individuelle `closed_at_ms <= cutoff` est revérifiée ([store.rs:1636](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1636)). Un candidat re-daté entre les deux via le public `record_run_completed` peut devenir le run le plus récent, donc protégé par `keep_last`, tout en étant purgé par la liste périmée.

### R3 — GAP

Les trois contrôles annoncés existent : `user_version >= 1`, liste de tables selon la version et `PRAGMA quick_check` ([store.rs:1863](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1863), [store.rs:1887](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1887), [store.rs:1918](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1918)).

Mais la racine du P1 n’est pas fermée : « inventaire complet » signifie seulement six requêtes sur `sqlite_master` vérifiant `type='table' AND name=?` ([store.rs:1899](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1899)). Aucun contrôle ne vérifie :

- les colonnes et leurs types/contraintes ;
- les PK/FK ;
- `closed_at_ms` ;
- l’index unique `outbox_one_effect_per_node`.

Un SQLite sain contenant six tables arbitraires portant les bons noms, avec `user_version=2`, passe ces contrôles et `quick_check`. Il est ensuite copié sur le store sain ([store.rs:1930](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1930)). À la réouverture, `user_version == 2` entraîne directement `to_latest` sans validation applicative du schéma ([store.rs:480](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:480)).

Le nouveau test ne forge que la variante « table `runs` seule » ([run_store.rs:1211](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:1211)). Il ne couvre pas le forgeage à six noms. Le remplacement destructif reste donc possible : **P1 non fermé**.

### R4 — PARTIEL

Les correctifs de code annoncés sont confirmés :

- digest snapshot lié au `run_id` aux deux portes ([store.rs:1436](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1436), [store.rs:1519](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1519), [store.rs:2054](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:2054)) ;
- âge et horloge GC négatifs refusés ([store.rs:1584](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1584)) ;
- alias DB/WAL/SHM et alias via `.tmp` refusés ([store.rs:1753](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1753)) ;
- bras out-of-process désormais clos et statut `Completed` asserté ([run_store.rs:415](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/tests/run_store.rs:415)) ;
- `record_intent_atomic` et `record_completion` sont `pub(crate)` hors `spike-faults` ([store.rs:667](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:667), [store.rs:848](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:848)) ;
- la feature reste off par défaut et activée par la self-dev-dependency du harnais ([Cargo.toml:32](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/Cargo.toml:32)) ;
- comparaison statique : le premier literal M1 courant ([store.rs:94](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:94)) est identique à `e034a574:store.rs:51-92`.

R4 reste toutefois partiel parce que les reformulations documentaires héritent des gaps R2/R3 : « éligibilité relue » est trop large dans [THREAT_MODEL.md:1768](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1768), et « restore valide le backup » reste trop fort dans [THREAT_MODEL.md:1767](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1767) et [STORE_MIGRATION_OPS.md:83](/C:/Users/FlowUP/Documents/Code/nexus/docs/release/STORE_MIGRATION_OPS.md:83).

### Re-vérification L1–L10

| Livrable | Verdict | Résultat |
|---|---|---|
| L1 | **CONFIRME** | `RunEvent.v1`, registre 7/6 remplis, schema et re-export cohérents ([run_event.rs:142](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:142), [contracts.rs:96](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:96), [contracts.schema.json:1572](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.schema.json:1572), [event.rs:17](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/event.rs:17)). |
| L2 | **CONFIRME** | Garde schema avant migration, chaîne vide v1 et `ChainGap` fail-closed, porte unique à la lecture ([store.rs:480](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:480), [run_event.rs:172](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:172), [run_event.rs:252](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:252), [store.rs:1108](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1108)). |
| L3 | **CONFIRME**, périmètre store | CAS/epoch/release/out-of-process sont corrects ; l’effet externe est désormais explicitement hors fermeture I. |
| L4 | **CONFIRME** | PK, FK, `INSERT OR IGNORE`, booléen et validation `message_id` ([store.rs:154](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:154), [store.rs:1392](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1392)). |
| L5 | **PARTIEL** | Le run-bound digest est fermé. Nouveau P2 : un `upto_sequence` négatif retourne une erreur store au lieu de `SnapshotRead::Poisoned`, donc un cache corrompu n’est pas toujours ignorable ([store.rs:1512](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:1512)). |
| L6 | **GAP** | P1 terminal non absorbant ; P2 rang `keep_last` périmable. |
| L7 | **GAP** | Validation de noms de tables, pas validation du schéma attendu. |
| L8 | **CONFIRME** | Entrées production sans hooks/knobs ; portes dégradées sous feature ; diff du harnais mécanique. |
| L9 | **GAP** | Structure v23 correcte, mais les lignes GC/restore/snapshot sur-affirment encore les garanties affectées. |
| L10 | **CONFIRME** | Version locale 1, domaine réservé, seule dépendance intra-workspace, M1 byte-identique, SPDX présents. Census statique : 36 tests, tous contenant `store`, 3 `upgrade_replay`, 4 `single_writer`, 0 `effect`. |

### GAPs P0/P1 — bloquants

- **P0 : aucun.**
- **P1 — R2/L6/L9 :** `record_run_completed` peut faire avancer un run terminal ; GC peut ensuite supprimer intégralement son diagnostic.
- **P1 — R3/L7/L9 :** un faux store avec les six bons noms de tables mais un schéma incompatible passe la validation puis remplace le store sain.

### GAPs P2/P3 — documentables

- **P2 — R2/L6/L9 :** le rang `keep_last` n’est pas revalidé dans la transaction.
- **P2 — L5/L9 :** un snapshot à séquence négative hard-fail au lieu d’être classé `Poisoned` et ignoré.
- **P3 : aucun nouveau substantiel.**

Donc non : tout n’est pas encore fermé.