Verdict externe Round 4 : **PASS — 0 P1 bloquant.** La Phase M est techniquement autorisée au commit. J’ai trouvé **1 P2 et 1 P3 mesurés, non bloquants**.

## Findings résiduels

### P2 — le sixième refus : `MissingCapability`

Le code est correct : sans capability, `delete` restitue `None` ([artifact_acl.rs:1080](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1080)). Le contrat exige bien « as supplied, untouched » et `None` lorsque rien n’a été fourni ([artifact_acl.rs:685](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:685)).

Mais le test `deleting_a_local_artefact_requires_its_key` ne vérifie jamais `refusal.capability.is_none()` ([artifact_acl.rs:1617](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1617)).

Mutation :

```text
MissingCapability:
capability: None
→ capability: Some(ReadCapability::new(vec![0; 32]))

cargo nextest run -p sbfb-workflow-engine artifact_acl --locked --offline
Summary: 22 tests run: 22 passed, 160 skipped
```

Le compte exact est donc :

- **5 sites** de `DeleteRefusal` au total ;
- **6 refus sémantiques**, dont `MissingCapability` ;
- parmi eux, **5 refus avec capability**, agrégés sur 4 sites.

La formulation du plan « quatre sites / cinq refus » ([sprint83_plan.md:1107](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1107)) est exacte seulement si elle précise « sites restituant une capability ». Correctif minimal : ajouter `assert!(refusal.capability.is_none())` et préciser le compte. Ce n’est pas un P1 : aucune clé réelle n’est actuellement perdue ou divulguée.

### P3 — la borne est gardée, pas l’identité du préfixe

L’implémentation renvoie correctement les 72 premiers caractères puis `…` ([artifact_acl.rs:292](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:292)), conformément au rustdoc « prefix of the text offered » ([artifact_acl.rs:660](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:660)).

Le test ne vérifie cependant que `≤ 73` et l’ellipse ([artifact_acl.rs:2105](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:2105)).

Mutation :

```text
let echo = "x".repeat(ECHO_LIMIT);

cargo nextest run -p sbfb-workflow-engine artifact_acl --locked --offline
Summary: 22 tests run: 22 passed, 160 skipped
```

Un texte arbitraire borné passe donc. Correctif facultatif : vérifier que la partie avant `…` est réellement un préfixe de l’entrée. Diagnostic seulement, code actuel correct : P3.

## Correctifs R3 falsifiés

| Mutation | Résultat |
|---|---|
| `DeletionRefused` restitue `[0;32]` à la place de la capability fournie | **0/1**, échec `capability does not open this artefact` |
| `DeletionRefused(None)` fabrique une capability | **0/1**, échec sur `refusal.capability.is_none()` |
| retour à `hex.to_string()` | **0/1**, écho ASCII de **200000 caractères** |
| retour intégral limité au seul Unicode | **0/1**, écho de **100000 caractères** |
| troncature sans ellipse | **0/1**, échec `a truncated echo must say so` |

Les deux bras de `published_artefact_refuses_deletion` sont donc discriminants ([artifact_acl.rs:1548](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:1548)), de même que les bras ASCII et multi-octets.

## Carrys re-bornés

- **`ARTIFACT-DELETE-UNWIND` : oui, fermable par geste.** L’exit exige soit un garde restituant la capability, soit un contrôle exécutable du profil release plus l’exclusion contractuelle des panics ([sprint83_plan.md:1138](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1138)). Mesure :  
  `cargo rustc -p sbfb-workflow-engine --lib --release --locked --offline -- --print cfg` → exit 0, `panic="abort"`.  
  Une simple répétition du paragraphe ne satisfait donc plus l’exit.

- **`DOCTEST-ERRCODE-NOT-ENFORCED` : oui, fermable par geste.** L’exit impose désormais **les deux** actions : jumeau qui compile et correction de l’affirmation fausse de [vault.rs:195](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/vault.rs:195), voir [sprint83_plan.md:1146](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1146). Il ne peut plus s’auto-clore par décision écrite.

## Verdict par livrable

| Livrable | Verdict R4 |
|---|---|
| `artifact_acl.rs` | **PASS avec P2/P3 non bloquants** ci-dessus. Baseline exacte : **22/22**. |
| `artifact_seal.rs` | **PASS — 11/11** ; AES-GCM, AAD, cadrage, erreurs et zéroisation cohérents ([artifact_seal.rs:169](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/artifact_seal.rs:169)). |
| `artifact_sealer.rs` | **PASS — 4/4** ; la capability atteint réellement AES-GCM ([artifact_sealer.rs:66](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/artifact_sealer.rs:66)). |
| `secret.rs` canari | **PASS**, sixième surface avec bras rouge ([secret.rs:630](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/secret.rs:630)). |
| Registrations/re-exports | **PASS** ([core lib:39](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/lib.rs:39), [daemon main:32](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:32), [engine lib:180](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/lib.rs:180)). |
| `ArtifactRef` / wire | **PASS borné** ; champs privés, origine hors wire, comparaison dédiée ([artifact_acl.rs:401](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/artifact_acl.rs:401)). |
| `THREAT_MODEL.md` | **PASS factuel**, DS-1 et bornes explicités ([THREAT_MODEL.md:2318](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2318)). |
| `sprint83_plan.md` | **PASS pour les correctifs R3**, avec le P2 de comptage précis ci-dessus. |
| Gate row 26 | **PASS** : trois commandes réelles ([sprint83_gate_matrix.md:60](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_gate_matrix.md:60)), **22 + 11 + 4 = 37**, donc 2429 + 37 = **2466**. |
| Rapports/index | **PASS mécanique** : 21 chemins stagés, diff non stagé vide. R4 n’a pas été écrit conformément à la contrainte de non-modification. |
| `WORKFLOW_AGENTS_OVERVIEW.md` | **PASS**, DS-1 et partage des responsabilités exacts ([WORKFLOW_AGENTS_OVERVIEW.md:48](/C:/Users/FlowUP/Documents/Code/nexus/docs/factory/WORKFLOW_AGENTS_OVERVIEW.md:48)). |

Concernant les flakes : les deux tests sont dans `dispatch_loop.rs`, fichier absent du diff, et utilisent effectivement la borne `Duration::from_secs(20)` ([dispatch_loop.rs:481](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/dispatch_loop.rs:481), [dispatch_loop.rs:649](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/dispatch_loop.rs:649)). Je n’ai pas rejoué leurs 3× isolés ; les éléments fournis sont cohérents avec une classe iroh-networked préexistante, pas avec une régression M.

État final : HEAD `a67e1eb3`, 21 entrées stagées, aucun diff non stagé, copies temporaires supprimées, aucun processus Cargo/Rustc. **Explicitement : je ne trouve plus de P1.**

