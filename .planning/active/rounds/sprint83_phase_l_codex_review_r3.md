L’algèbre r3 est saine et les deux P1 précédents sont réellement fermés : je ne trouve aucun GAP P0/P1, mais quatre GAP P2, un PARTIEL P2 et trois écarts P3 empêchent encore une promotion honnête de `PASS-PENDING` vers `PASS`.

1. **P2 — GAP — audience liée au type, pas au sujet exact.**  
   [`capability.rs:512-536`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:512), [`sprint83_plan.md:1501-1514`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1501)

   `GrantAudience` distingue seulement `Release` de `Pack`; `CapabilityGrant` ne porte ni identité/digest de release ni identité/version de pack. Le confused deputy demandé — grant `Release` fourni à la couche `Pack` — est bien fermé. En revanche, le module ne peut pas distinguer un grant de release A utilisé pour release B, ou pack v1 pour pack v2. Les sept exigences V ne gravent pas cette liaison exacte. Elles devraient aussi imposer `EffectNeed::One` au dispatch et la concordance de la classe/opération réelle avec `BoundedCapability.class`. Aucun EoP live aujourd’hui, car DS‑1 signifie zéro call-site.

2. **P2 — GAP — `refines` reste insuffisamment protégé par les tests.**  
   [`capability.rs:387-397`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:387), [`capability.rs:1542-1588`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:1542), [`capability.rs:1779-1893`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:1779)

   Le code courant est correct, mais les mutations suivantes survivraient probablement aux 19 tests :

   - supprimer de `BoundedCapability::refines` le contrôle de `class`, `timeout_ms`, `data_policy` ou `approval`;
   - retourner le premier grant uniquement pour `bounds.len() >= 4`;
   - ignorer expiry/revocation uniquement dans l’audience `Pack`.

   Le nouveau test négatif ne pine directement que `Ceiling`, `DestinationBound` et un cas `EffectiveCapabilities` gouverné par `quota`. Or `dominating_grant` dépend de l’intégralité de l’ordre produit.

3. **P2 — GAP — la “garde d’exhaustivité `ALL` compile-time” est sur‑affirmée.**  
   [`capability.rs:186-207`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:186), [`capability.rs:869-901`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:869), [`THREAT_MODEL.md:1877`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1877)

   L’état courant contient bien exactement 17 variantes distinctes. Mais une modification ajoutant une 18e variante à l’enum **et** au `match`, tout en oubliant `ALL`, compile et laisse le test vert : la boucle n’itère que sur `ALL`. Une macro source unique générant enum + `ALL` fermerait réellement ce drift.

4. **P2 — PARTIEL — le type résultat reste forgeable.**  
   [`capability.rs:400-416`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:400), [`THREAT_MODEL.md:1876-1877`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1876), [`sprint83_plan.md:1513-1514`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_plan.md:1513)

   Il n’existe effectivement aucun `insert`/`extend`/`widen` public et `inner` est privé. Mais `pub from_declared` produit exactement le même type que `effective_capabilities`; “Constructed ONLY by effective_capabilities” est donc faux et `T-CAP-WIDEN-BYPASS = fermée` est trop fort. La review a vu ce point et route correctement un newtype non forgeable vers V : état actuel **borné**, pas fermé.

5. **P2 — GAP — trois anchors `compile.rs` parlent encore d’un runtime typé déjà actif.**  
   [`compile.rs:140-144`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:140), [`compile.rs:176-181`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:176), [`compile.rs:2419-2425`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:2419)

   « the runtime intersection judges/refines by type » est au présent alors que [`orchestrator.rs:14-19`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/orchestrator.rs:14) confirme qu’aucun dispatch ne la consulte. Il faut écrire « jugera une fois reliée ». Le THREAT model, lui, donne correctement cette qualification future.

6. **P3 — PARTIEL — dette schemars connue mais factuellement fausse.**  
   [`compile.rs:606-610`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/compile.rs:606), [`sprint83_phase_l_review.md:71-74`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_phase_l_review.md:71)

   « Nothing outside this map can ever be granted » et « binding is Phases J/K/L » ne sont pas vrais aujourd’hui. Préserver le golden explique le choix, mais ne rend pas le texte exact. La dette est toutefois explicitement possédée par V.

7. **P3 — GAP — documentation interne stale après la suppression du fold.**  
   [`capability.rs:701-709`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:701), [`capability.rs:1773-1777`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:1773)

   Le rustdoc référence toujours `combine_same_audience`, supprimé. Le commentaire de test généralise aussi « incomparable bounds poison » alors que deux bornes incomparables survivent si un troisième grant les domine globalement. Aucun effet runtime, mais la review r3 l’a manqué.

8. **P3 — PARTIEL — “HITL MUST stay LAST” est plus fort que l’invariant réel.**  
   [`capability.rs:627-640`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:627), [`capability.rs:1663-1770`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:1663)

   Le test tue bien toute position dangereuse avant local/release/pack, y compris entre release et pack. Il ne rougirait pas si HITL passait juste avant `EffectNeed`, car ce dernier ne fait que filtrer des clés : ce déplacement est sémantiquement sûr. L’invariant précis est « après tout meet pouvant relever `approval` », pas littéralement « dernière opération ».

### Vérifications demandées

- **Algèbre — CONFIRME.** [`BoundedCapability::meet`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:371) applique bien ∩ destinations, min quota/timeout, max approval et OR des flags. `DataPolicy` est un produit booléen cohérent, et son meet raffine les deux opérandes ([`capability.rs:292-331`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:292)).
- **Meet de sets — CONFIRME.** Intersection des clés et mismatch de classe symétriquement droppé ([`capability.rs:446-458`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:446)).
- **P1 multi-grants r2 — CONFIRME FERMÉ.** [`dominating_grant`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:753) recherche un maximum sur l’ensemble entier. Pour toute cardinalité, le prédicat est indépendant de l’ordre; si deux maxima existent, l’antisymétrie du produit impose qu’ils soient égaux. Le drop sans maximum est sur‑atténuant mais fail-closed.
- **Non-vacuité — CONFIRME.** Le replay indépendant exact du seed donne **220/2000** résultats finaux non vides et **215/2000** avec resserrement strict, conformément aux témoins [`capability.rs:917-1051`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:917).
- **Closure — CONFIRME.** `smuggled` est déclaré et granté dans les deux audiences; seule la closure le retire ([`capability.rs:1298-1340`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/capability.rs:1298)).
- **Expiry/revoke — CONFIRME.** `now_ms >= exp` et revocation recalculée, miroir exact de [`grant_v0.rs:53-65`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-manifest/src/grant_v0.rs:53); aucune dépendance engine→manifest ([`Cargo.toml:11-30`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/Cargo.toml:11)).
- **Entrées hostiles — CONFIRME dans le périmètre pur.** Aucun `unwrap`, indexation ou calcul arithmétique production; `u64::MAX` est sûr. `below(0)` ne concerne que le générateur `#[cfg(test)]`. Les doublons intra-map sont absorbés par `BTreeMap`; les doublons inter-grants suivent domination-ou-drop.

### Les 19 tests

| Test | Verdict |
|---|---|
| `seventeen_classes_typed` :869 | **PARTIEL** — courant discriminant, exhaustivité future non garantie |
| `intersection_only_attenuates` :917 | **PARTIEL mais non-vacuous** — résultat final seulement, 220/215 témoins |
| `meet_laws_*` :1059 | **DISCRIMINANT** |
| `absent_layers_are_identity` :1091 | **DISCRIMINANT** |
| `expiry_enforced` :1132 | **DISCRIMINANT release**, miroir pack manquant |
| `revocation_per_call` :1172 | **DISCRIMINANT release**, miroir pack manquant |
| `audience_enforced` :1217 | **DISCRIMINANT**, deux directions |
| `depth_enforced` :1273 | **PARTIEL** — fold générique, pas pipeline sept couches |
| `undeclared_capability_refused` :1298 | **DISCRIMINANT** |
| `human_approval_requires_valid_decision` :1355 | **DISCRIMINANT**, décision par-ref |
| `ordering_invariants_pinned` :1423 | **DISCRIMINANT** |
| `approval_meets_by_max_not_min` :1461 | **DISCRIMINANT**, cinq axes |
| `class_mismatch_drops_fail_closed` :1514 | **DISCRIMINANT pour meet** |
| `refines_negative_cases_pinned` :1542 | **PARTIEL**, axes Bounded incomplets |
| `os_toggle_class_filter` :1593 | **DISCRIMINANT** |
| `hitl_gate_must_run_after_approval_raise` :1663 | **DISCRIMINANT pour toutes les positions dangereuses** |
| `multi_grant_same_audience_combines_soundly` :1779 | **DISCRIMINANT à trois grants**, couverture ≥4/invariants par axe manquante |
| `effect_need_narrows_to_single_surviving_ref` :1901 | **DISCRIMINANT** |
| `local_policy_denies_and_tightens` :1965 | **DISCRIMINANT** |

### Docs, planning et pré-launch

- Les rows retry/replay sont honnêtement **bornées DS‑1**, pas fermées ([`THREAT_MODEL.md:1879-1880`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1879)).
- A13 est cohérent ([`THREAT_MODEL.md:71`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:71)); v27 précède bien v26 ([`:2242`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2242), [`:2277`](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:2277)).
- Gate matrix : row 8 reste L, 9/10/11 deviennent L1 ([`gate_matrix:42-45`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_gate_matrix.md:42)), row 44 devient L1/Q et HUB‑T2 inclut L1 ([`:90-92`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_gate_matrix.md:90), [`:126`](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint83_gate_matrix.md:126)). Census mécanique : 47 rows numérotées 1–47.
- Quatre versions toujours à 1 : [`pack.rs:84`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/pack.rs:84), [`run_event.rs:65`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/run_event.rs:65), [`store.rs:321`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-engine/src/store.rs:321), [`public_feed.rs:20`](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/public_feed.rs:20).
- Census `DOMAIN_*` = **29**; registre = **7** ([`contracts.rs:110-162`](/C:/Users/FlowUP/Documents/Code/nexus/crates/sbfb-workflow-core/src/contracts.rs:110)).
- `contracts.schema.json` est byte-identique à HEAD, hash Git `3d16bf0c…`; aucun diff Cargo/Cargo.lock; zéro derive serde dans `capability.rs`.
- `cargo fmt --all --check` et `git diff --check` passent. Nextest et les mutations n’ont pas été relancés dans ce sandbox lecture seule.
- HEAD est resté `9498e3fa`; état final inchangé : 8 fichiers modifiés et 8 non suivis.
- La review canonique est byte-identique au round r3 (`SHA-256 060A56B0…`) et reflète bien `dominating_grant`/19 tests, mais seulement **PARTIELLEMENT** les gaps ci-dessus. Son statut `PASS-PENDING` reste correct.

### Verdict par livrable

| Livrable | Verdict |
|---|---|
| `BoundedCapability` / `DataPolicy` / meet | **CONFIRME** |
| `EffectiveCapabilities::meet` | **CONFIRME** |
| `dominating_grant` r3 | **CONFIRME — P1 r1/r2 fermés** |
| Ordre HITL de sécurité | **CONFIRME**, formulation littérale PARTIELLE |
| 19 tests | **PARTIEL P2** |
| Fail-closed exact demandé | **CONFIRME**, hors frontières DS‑1 |
| Contrat de reliure V | **GAP P2** |
| THREAT/llms/orchestrator | **Globalement honnêtes, PARTIEL** |
| `compile.rs` | **PARTIEL/GAP documentaire** |
| Gate matrix et invariants pré-launch | **CONFIRME** |
| Review canonique r3 | **PARTIEL — conserver PASS-PENDING** |

