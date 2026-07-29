Verdict externe Round 3 : **FAIL — 1 P1 bloquant, 3 P2.** Je ne peux pas autoriser le commit dans cet état.

Toutes les lignes citées proviennent de `git show :<chemin>` ; le diff non staged est vide.

## P1 — le cinquième membre existe

`delete` possède quatre sites physiques de restitution, mais le premier agrège deux refus sémantiques dans `ArtifactPolicy::authorise_delete` : `UntrustedReference` et `DeletionRefused` ([artifact_acl.rs:876](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:876), [artifact_acl.rs:1073](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1073)).

Le cinquième chemin est :

> artefact authentiquement minté `Published` + capability étrangère fournie ⇒ `DeletionRefused` doit rendre cette capability intacte.

Le test publié appelle actuellement `delete` avec `None`, donc ne peut rien vérifier sur la restitution ([artifact_acl.rs:1536](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1536), [artifact_acl.rs:1551](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1551)). Le contrôle fonctionnel du même site de retour ne couvre que `UntrustedReference` ([artifact_acl.rs:1782](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1782), [artifact_acl.rs:1810](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1810)).

Mutation mesurée, limitée à `error == DeletionRefused` :

```text
capability reçue détruite, ReadCapability([0; 32]) restituée
cargo nextest run -p sbfb-workflow-engine artifact_acl --locked --offline
Summary: 22 tests run: 22 passed, 160 skipped
```

Avec un bras rouge publiant un artefact tiers puis exigeant que la capability rendue le rouvre :

```text
1 test run: 0 passed, 1 failed
SealerRefused { reason: "capability does not open this artefact" }
```

Sur le code non muté, ce bras passe **1/1** : l’implémentation est correcte aujourd’hui, mais sa garantie n’est pas protégée.

Cela réfute « les quatre bras […] restitution vérifiée FONCTIONNELLEMENT » dans [sprint83_plan.md:1107](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1107). Il y a quatre sites, mais **cinq refus sémantiques avec capability** :

1. `UntrustedReference`
2. `DeletionRefused`
3. blob absent
4. contenu/adresse incohérents
5. `SealerRefused`

Correctif minimal : étendre `published_artefact_refuses_deletion` avec une vraie capability tiers, la récupérer et rouvrir l’artefact tiers ; conserver séparément le bras `None`.

## Matrice des autres sorties

Tous les autres chemins sont discriminants :

| Mutation | Résultat ciblé |
|---|---:|
| garde provenance supprimée | 0/1 |
| garde mode publié supprimée | 0/1 |
| absence de capability → `Ok(())` | 0/1 |
| restitution falsifiée sur `UntrustedReference` | 0/1 |
| restitution falsifiée sur blob absent | 0/1 |
| re-hash supprimé | 0/1 |
| restitution falsifiée sur `DigestMismatch` | 0/1 |
| restitution falsifiée sur `SealerRefused` | 0/1 |
| succès sans `untag` | 0/1 |
| restauration victime remplacée par `Vec::new()` | 0/1 |

Le correctif P1-A du round 2 est donc **PASS** : sa mutation échoue bien à [artifact_acl.rs:2200](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2200).

## P2 restants

- **P2 — borne `MalformedDigest` non gardée.** L’implémentation à [artifact_acl.rs:292](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:292) fonctionne : sonde 200 kB ASCII + Unicode ⇒ 72 caractères et ellipse, **1/1 PASS**. Mais revenir à `hex.to_string()` laisse **22/22 PASS** ; le test ne vérifie que la variante d’erreur ([artifact_acl.rs:2040](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2040)). En plus, le rustdoc affirme encore que le champ contient « le texte offert », alors qu’il contient désormais un préfixe borné ([artifact_acl.rs:660](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:660)). Ajouter un bras long ASCII/Unicode et corriger ce rustdoc.

- **P2 — `ARTIFACT-DELETE-UNWIND` mal borné.** Le fait est vrai : une couture qui panique traverse `delete`, ne produit aucun `DeleteRefusal`, et la sonde `catch_unwind` passe **1/1**. Mais l’exit « borne écrite » est déjà satisfait par le paragraphe qui vient de l’écrire ; le carry peut donc se fermer sans geste ([sprint83_plan.md:1136](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1136)). Nommer `Owner = Phase Y` et exiger soit un garde rendant la capability, soit une borne exécutable sur le profil `panic=abort` et l’exclusion contractuelle des panics.

- **P2 — `DOCTEST-ERRCODE-NOT-ENFORCED` reste mal borné.** L’exit autorise encore une simple décision selon laquelle le doctest suffit ([sprint83_plan.md:1141](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1141)), tandis que `vault.rs` affirme que l’annotation empêche un vert sur erreur étrangère ([vault.rs:195](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:195)). Mesure indépendante sur rustc 1.95.0 : bloc `compile_fail,E0277` produisant réellement `E0432` ⇒ **rustdoc 1 passed, exit 0**. L’exit doit être « jumeau qui compile **ET** correction de `vault.rs:195-197` ».

`ARTIFACT-UNTAG-NAME-MISMATCH` est, lui, correctement routé : owner Phase Y, trigger premier implémenteur de production, exit concret ([sprint83_plan.md:1126](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1126), [sprint83_plan.md:1948](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1948)).

## Verdict par livrable

| Livrable | Verdict R3 |
|---|---|
| `artifact_acl.rs` | **FAIL** — P1 cinquième refus non gardé ; P2 troncature non gardée/rustdoc faux. |
| `artifact_seal.rs` | **PASS** — inchangé, primitive et 11 tests non réfutés. |
| `artifact_sealer.rs` | **PASS** — inchangé, implémentation réelle et 4 tests non réfutés. |
| `secret.rs` canari | **PASS**. |
| Registrations/re-exports | **PASS**. |
| `ArtifactRef` / wire | **PASS borné** ; balayage circulaire = 0 hit, test de re-hash présent une fois en §M.3. |
| `THREAT_MODEL.md` | **PASS factuel sur le code actuel**, mais la preuve universelle de restitution reste incomplète. |
| `sprint83_plan.md` | **FAIL** — compte sémantique faux + deux carrys P2 mal bornés. |
| Gate matrix row 26 | **FAIL sémantique**, mécaniquement correcte. |
| Rapports / index | **PASS mécanique** — 20 chemins staged, diff non staged vide. |
| `WORKFLOW_AGENTS_OVERVIEW.md` | **PASS**. |

Je n’ai pas rejoué le workspace complet : les **2466 passed / 7 skipped** et le flake iroh déclaré ne sont pas réfutés. État final : sondes supprimées, aucun processus Cargo/Rustc, aucun fichier suivi modifié.

