Le meet central est correct, mais la phase n’est pas validable en l’état : j’ai trouvé **1 GAP P1 réel dans l’union intra-audience**, **5 P2** de sûreté/preuve et **3 P3** documentaires. Aucun P0 ; DS-1 est bien respectée, mais la conclusion Claude « 0 P0/P1 » est infirmée.

1. **P1 — GAP — `crates/sbfb-workflow-engine/src/capability.rs:705-756` — le join de plusieurs grants fabrique des capacités “Frankenstein”.**  
   `join_grant_bounds` calcule le join indépendamment par champ : union des destinations, maximum des plafonds, minimum d’approbation et intersection des flags de données. Ce LUB n’est pas l’union exacte de deux grants incomparables.

   Contre-exemple, même audience/ref/classe :

   - grant A : destination `{a}`, quota `100`, approval `Human`;
   - grant B : destination `{b}`, quota `1`, approval `Auto`;
   - résultat actuel : destinations `{a,b}`, quota `100`, approval `Auto`.

   Il autorise donc `a` sans HITL et 100 appels vers `b`, alors qu’aucun grant ne l’autorise. Un `declared` et un grant pack à `top()` ne re-bornent rien ; la couche HITL laisse ensuite passer `Auto` à `capability.rs:686-689`. Le meet externe reborne chaque coordonnée, mais ne restaure jamais les corrélations perdues entre champs. La note `capability.rs:115-121` sur les planchers déclarés ne corrige pas destinations×quota×approval. Aucun test n’entre dans `and_modify` : le générateur crée exactement un grant par audience à `capability.rs:978-992`.  
   Correction nécessaire : conserver les alternatives, ou n’agréger que des bornes comparables et refuser/dropper les bornes incomparables. `T-CAP-WIDEN-BYPASS` ne peut pas rester « fermée » à `docs/security/THREAT_MODEL.md:1874`.

2. **P2 — PARTIEL — `capability.rs:723-730`, `:745-748` — les classes divergentes restent left-biased.**  
   Sur mismatch, `join_grant_bounds` renvoie `None`, mais `and_modify` conserve l’entrée déjà vue. Le même ensemble de grants autorise ou refuse selon l’ordre. La review l’avait vu, mais la recommandation « ordre stable ou tri » à `capability.rs:122-126` ne rend pas la collision fail-closed : elle rend seulement le résultat reproductible. Il faut tombstoner/dropper la ref dès le premier conflit de classe.  
   En revanche, le drop de `EffectiveCapabilities::meet` est bien symétrique à `capability.rs:455-462`.

3. **P2 — GAP — `capability.rs:633-644`, `:1556-1603` — le test HITL ne prouve pas que le gate reste après les trois couches élévatrices.**  
   Le code et le docstring sont corrects. Mais le test met **les deux** grants release et pack à `Human` (`:1583-1584`). Une mutation plaçant le gate HITL **entre** release et pack reste verte : release a déjà élevé à `Human`, donc le cas sans décision droppe quand même. Le test prouve seulement « pas avant le premier grant élévateur », pas « après local/release/pack ». Il faut trois cas isolés : élévation uniquement par local policy, uniquement release, uniquement pack.

4. **P2 — GAP — `capability.rs:1284-1341` — la preuve de la couche 6 `EffectNeed` est vacuous.**  
   `smuggled` est correctement dans `declared` et les deux grants, mais la closure l’a déjà supprimée à `:1299-1300`. L’assertion `EffectNeed::One("smuggled") → ∅` à `:1328-1341` reste donc verte si on supprime entièrement la branche `EffectNeed::One` à `:681-684`. Il faut deux refs survivant aux couches 1–5, puis `One("a")`, avec assertion exacte que seul `a` reste.

5. **P2 — PARTIEL — `graph.rs:265-268`, `pack.rs:418-420`, `capability.rs:419`, `THREAT_MODEL.md:1875` — la classe inconnue est fermée dans l’enum, pas à la frontière réelle.**  
   Les entrées pack/plan restent des `String`; aucun résolveur autoritaire `capability_ref → BoundedCapability` ne garantit actuellement « inconnu ⇒ drop ». De plus, `EffectiveCapabilities::from_declared` est un constructeur public arbitraire du même type que le résultat autorisé : une future API de dispatch acceptant ce type ne pourra pas distinguer un résultat calculé d’un set forgé. DS-1 signifie qu’il n’y a pas encore d’exploitation live, mais `T-CAP-CLASS-UNKNOWN-FAILOPEN` devrait être « bornée jusqu’à la reliure », pas « fermée ».  
   Le test `seventeen_classes_typed` confirme les 17 variantes actuelles, mais une mutation ajoutant une 18e variante à l’enum sans l’ajouter à `ALL` reste verte.

6. **P2 — GAP — `capability.rs:55`, `:351`, `docs/workflow/llms.txt:71`, `THREAT_MODEL.md:1874` — les docs ont conservé l’ancien `DataPolicy=max`.**  
   Après F3, `DataPolicy` n’a plus d’ordre total : meet = OR des flags et join = AND (`capability.rs:311-335`). Dire « max data-policy » est faux et peut réintroduire précisément le bug corrigé.  
   Asset A13 est également incohérent à `THREAT_MODEL.md:71` : « une couche absente/vide/expirée ⇒ refus » est faux pour les attenuators absents, qui sont identité (`capability.rs:61-77`), et pour HITL vide, qui ne refuse que `Human` (`:79-86`). Seules les audiences de grants absentes/vides/expirées refusent toute ref.

7. **P3 — CONFIRME, non fermé — `compile.rs:606-610` — le champ schemars reste stale vis-à-vis de DS-1.**  
   « per-effect enforcement binding is Phases J/K/L » implique que la reliure est accomplie, alors que `orchestrator.rs:19`, `:62` et `sprint83_plan.md:1501-1507` disent correctement qu’elle appartient à V/run-surface. Préserver `contracts.schema.json` byte-identique explique l’absence de modification ; cela ne rend pas la phrase vraie. C’est défendable uniquement comme dette explicitement possédée par V, pas comme finding fermé.

8. **P3 — GAP — `.planning/active/sprint83_gate_matrix.md:126` — la réconciliation L→L1 est incomplète.**  
   Les rows principales sont correctes : row 8 reste L (`:42`), rows 9/10/11 sont L1 (`:43-45`) et row 44 L1/Q (`:78`). Mais HUB-T2 conserve les contributeurs `(C, L)` tout en dépendant de `G-HITL`; il doit inclure L1. Les 47 gates ont bien un propriétaire, mais la traçabilité acceptance→phase est partiellement stale.

9. **P3 — GAP — `THREAT_MODEL.md:1848-1849`, `:1861`, `:2264` — faits mécaniques périmés.**  
   « tous les `*_FORMAT_VERSION` à 1 » est globalement faux : `crates/nexus-worker-core/src/invite.rs:73` vaut 2. Cela ne constitue pas un bump Phase L. Le document revendique encore 6 mutations alors que la review r2 en revendique 9 à `.planning/active/sprint83_phase_l_review.md:93-95`. L’ordre historique demandé est en revanche correct : v27 précède v26 (`THREAT_MODEL.md:2240`, `:2269`).

### Qualité des 15 tests

Le replay indépendant du seed de `intersection_only_attenuates` donne **220/2000 résultats non vides** et **215/2000 survivants strictement resserrés** : les deux témoins sont réels, pas accidentels. Le seed des lois produit 81 meets non vides ; celui de depth produit 26 étapes non vides et 711 drops stricts.

| Test | Verdict |
|---|---|
| `seventeen_classes_typed` `:884-891` | **PARTIEL** — 17 actuelles distinctes et typées ; ajout d’une variante hors `ALL` non détecté |
| `intersection_only_attenuates` `:906-1041` | **PARTIEL, non-vacuous** — final seulement, pas chaque couche ni chaque grant |
| `meet_laws_*` `:1048-1070` | **CONFIRME, discriminant** |
| `absent_layers_are_identity` `:1080-1113` | **CONFIRME, discriminant** |
| `expiry_enforced` `:1121-1157` | **CONFIRME** — avant/borne/après/None+`u64::MAX` |
| `revocation_per_call` `:1161-1196` | **CONFIRME** |
| `audience_enforced` `:1206-1248` | **PARTIEL** — release→pack couvert ; pack→release unidirectionnel non couvert |
| `depth_enforced` `:1259-1273` | **PARTIEL** — meet-fold générique, pas le pipeline réel des sept couches |
| `undeclared_capability_refused` `:1284-1342` | **PARTIEL** — closure très discriminante ; clause EffectNeed vacuous |
| `human_approval_requires_valid_decision` `:1353-1403` | **PARTIEL** — aucune décision liée à une autre ref |
| `approval_meets_by_max_not_min` `:1410-1459` | **PARTIEL** — champs bien orientés ; ordre `Human > Notify` non épinglé |
| `class_mismatch_drops_fail_closed` `:1463-1484` | **CONFIRME** pour `EffectiveCapabilities::meet` |
| `os_toggle_class_filter` `:1488-1548` | **CONFIRME** |
| `hitl_gate_must_run_after_approval_raise` `:1556-1603` | **PARTIEL** — mutation entre release et pack survivante |
| `local_policy_denies_and_tightens` `:1610-1683` | **CONFIRME** — deny + borne exacte quota/approval |

Mutations supplémentaires qui resteraient vertes :

- supprimer entièrement le filtre `EffectNeed::One`;
- déplacer HITL entre release et pack;
- remplacer `decided_refs.contains(k)` par « le set de décisions est non vide »;
- placer `Notify` après `Human` dans l’enum;
- accepter un grant Pack dans la couche Release uniquement;
- ajouter une 18e `CapabilityClass` hors `ALL`;
- rendre `DataPolicy::refines` toujours vrai;
- casser tous les joins multi-grants : aucun test actuel ne présente deux grants de même audience/ref.

### Contrôles confirmés

- **Meet cœur : CONFIRME** — destinations ∩, quota/timeout min, DataPolicy OR, approval max (`capability.rs:375-388`); DataPolicy meet commutatif/associatif/idempotent et cohérent avec `refines` (`:311-335`).
- **Expiry/revocation : CONFIRME** — miroir exact de `grant_v0.rs:53-65`; l’engine ne dépend pas de `sbfb-manifest` (`Cargo.toml:11-30`).
- **Closure : CONFIRME** — `smuggled` déclaré et double-granté est retiré uniquement par la closure (`capability.rs:1284-1326`).
- **Panic : PARTIEL** — production sûre pour `u64::MAX`/bornes/sets. `below(0)` panique toujours (`:798-800`), mais c’est un helper privé sous `#[cfg(test)]` (`:777`), pas une entrée hostile runtime. Les doublons BTreeMap ne paniquent pas ; les doublons inter-grants exposent les défauts ci-dessus.
- **Docs DS-1 retry/replay : CONFIRME** — les rows restent honnêtement « bornées, reliure DS-1 » (`THREAT_MODEL.md:1877-1878`).
- **Pré-launch : CONFIRME** — quatre versions concernées toujours à 1 (`pack.rs:84`, `run_event.rs:65`, `store.rs:321`, `public_feed.rs:20`), census mécanique **29**, registre 7 (`contracts.rs:148-162`), `contracts.schema.json` sans diff, Cargo engine inchangé, aucun derive serde dans `capability.rs`.
- `cargo fmt --all --check` et `git diff --check` passent. Je n’ai pas relancé Nextest dans le sandbox read-only ; j’ai rejoué indépendamment les seeds purs.

### Verdict par livrable

| Livrable | Verdict |
|---|---|
| `capability.rs` algèbre de meet | **CONFIRME** |
| `capability.rs` agrégation multi-grants | **GAP P1 — bloque PASS** |
| 15 tests | **PARTIEL** |
| `lib.rs` module/re-exports | **CONFIRME** |
| `orchestrator.rs` bornes DS-1 | **CONFIRME** |
| `compile.rs` | **PARTIEL** — champ schemars stale |
| `THREAT_MODEL.md` / A13 / v27 | **GAP** |
| `llms.txt` | **GAP** — ancien `DataPolicy=max` |
| `WORKFLOW_AGENTS_OVERVIEW.md` | **CONFIRME** |
| `sprint83_gate_matrix.md` | **PARTIEL** |
| `sprint83_plan.md` reliure V | **CONFIRME**, à compléter avec la résolution des grants incomparables et la provenance de `EffectNeed` |
| Preflight + review r2/canonique | **GAP** — le verdict `0 P0/P1` doit être révisé |
| Invariants pré-launch | **CONFIRME** |

Le working tree final est resté sur `9498e3fa`, avec **8 fichiers modifiés et 5 non suivis** ; le cinquième non suivi est la review canonique, byte-identique à r2. La phase doit rester `PASS-PENDING`, corriger le P1, ajouter les tests discriminants, puis repasser une boucle indépendante complète.

