Le meet central est sain et le P1 « Frankenstein LUB » de r1 est réellement fermé, mais la phase n’est pas validable en l’état : il reste 1 GAP P1 nouveau sur l’ordre de trois grants, 3 GAP P2 de tests/docs/preuve, 1 PARTIEL P2 de reliure et 1 dette P3 confirmée; aucun P0.

## Findings adversariaux

1. **P1 — GAP — `crates/sbfb-workflow-engine/src/capability.rs:715-765` — l’agrégation reste dépendante de l’ordre dès trois grants.**

   Le correctif empêche bien toute borne synthétique : `combine_same_audience` retourne uniquement `a`, `b` ou `None`. Mais l’accumulateur n’est pas permutation-invariant.

   Contre-exemple, même audience/ref/classe, mêmes quota/timeout/approval :

   - A : destinations `{a}`;
   - B : destinations `{b}`;
   - C : destinations `{a,b}`.

   A et B sont incomparables, mais chacun raffine C.

   - ordre `[C,A,B]` : C est conservé, puis A et B raffinent C ⇒ **C survit**;
   - ordre `[A,B,C]` : A+B produit `None` à `:764`, puis le poison reste absorbant à `:723` ⇒ **drop**.

   Le même multiset autorise ou refuse selon l’ordre du slice, contrairement à « order-independently » à `:714` et à « any incomparable pair poisons » à `:123-127`. C’est un fail-open par rapport à la politique documentée de collision invalide, même si aucune borne absente des grants n’est fabriquée. DS-1 empêche une exploitation live aujourd’hui, mais ce code est destiné à devenir autoritaire.

   Correction saine : agréger toutes les bornes d’une ref, puis chercher un maximum global que toutes les autres raffinent; sinon drop/erreur typée. Si la politique voulue est réellement « toute paire incomparable empoisonne », il faut vérifier toutes les paires indépendamment de l’ordre. Ajouter un test sur toutes les permutations de trois grants.

2. **P2 — GAP — `capability.rs:923-1058`, `:1065-1086`, `:1731-1811` — la suite laisse encore survivre plusieurs mutations.**

   Le fichier contient désormais **18 tests**, pas 15. Le replay exact du seed de `intersection_only_attenuates` donne **220/2000** résultats non vides et **215/2000** survivants strictement resserrés : ce test n’est plus vacuous.

   Restent notamment vertes :

   - permutation de trois grants ci-dessus;
   - inversion du cas comparable : le test ne joue que `narrow, wide`, jamais `wide, narrow`;
   - `EffectiveCapabilities::refines` remplacé par `true` : aucun test négatif ne vérifie qu’un set avec une clé supplémentaire ou une borne plus permissive ne raffine pas l’autre;
   - mutations de sur-atténuation dans `Ceiling::refines`/`DestinationBound::refines`, car les cas multi-grants ne pinent pas chaque axe isolément;
   - `depth_enforced` reste un fold de meets génériques, pas le pipeline réel des sept couches.

3. **P2 — GAP — `docs/security/THREAT_MODEL.md:1876`, `:2248-2251`; `docs/workflow/llms.txt:72-74` — les docs ne sont pas encore exactes.**

   - `T-CAP-WIDEN-BYPASS` affirme l’agrégation « ordre-indépendante » et `llms.txt` généralise « incomparable → drop »; le contre-exemple à trois grants les réfute.
   - La row courante classe correctement `T-CAP-CLASS-UNKNOWN-FAILOPEN` comme **bornée** à `THREAT_MODEL.md:1877`, mais l’historique v27 dit encore **fermée** à `:2248-2251`.
   - A13 est, lui, corrigé et cohérent à `THREAT_MODEL.md:71`.
   - Les rows retry/replay sont honnêtement bornées DS-1 à `:1879-1880`.
   - v27 précède bien v26 à `:2242` et `:2275`.

4. **P2 — GAP — `.planning/active/sprint83_phase_l_review.md:38,93,103` — la preuve canonique ne couvre pas le code courant.**

   Le review canonique est byte-identique au vieux r2 et affirme encore :

   - `DataPolicy::join = INTER` à `:38`, alors que `join` a été supprimé;
   - 9 mutations à `:93`;
   - 15/15 tests à `:103`.

   En outre, `.planning/active/rounds/sprint83_phase_l_codex_review_r2.md:1-11` est apparu pendant l’audit et contient seulement un échec de sandbox affirmant que le dépôt n’a pas été lu. Il ne constitue aucune evidence. Le claim « 14 mutations tuées » peut être vrai, mais aucun artefact de review courant ne le démontre.

5. **P2 — PARTIEL — `capability.rs:398-412`; `THREAT_MODEL.md:1877`; `sprint83_plan.md:1501-1507` — la frontière DS-1 reste ouverte et insuffisamment contractée dans V.**

   La row THREAT reconnaît correctement l’absence de résolveur autoritaire `String → BoundedCapability` et le fait que `from_declared` peut forger directement le même type que le résultat. Il n’existe bien aucun call-site hors tests.

   Mais la Phase V ne grave actuellement que « câbler `effective_capabilities` ». Elle devrait également exiger explicitement :

   - inconnu/mal mappé ⇒ refus;
   - provenance autoritaire de `EffectNeed`;
   - validation de la destination, quota, timeout et data policy du véritable effet contre la borne retournée;
   - unicité ou erreur typée pour plusieurs grants d’une même ref/audience;
   - distinction API entre candidat déclaré et résultat calculé.

   Ce n’est pas un EoP live en L, mais la requalification « bornée » n’est pas une fermeture.

6. **P3 — CONFIRME/PARTIEL — `crates/sbfb-workflow-core/src/compile.rs:606-610` — le doc schemars reste factuellement stale.**

   « Nothing outside this map can ever be granted » et « enforcement binding is Phases J/K/L » suggèrent une reliure accomplie. `orchestrator.rs:17-19,61-64` établit au contraire zéro consultation per-effet. Préserver `contracts.schema.json` explique la dette, mais ne rend pas le texte vrai; elle doit rester explicitement possédée par V.

## Vérification des points demandés

### Algèbre et HITL

- **CONFIRME** — `BoundedCapability::meet`, `capability.rs:367-393` : destinations ∩, quota/timeout `min`, approval `max`, `DataPolicy` OR.
- **CONFIRME** — `DataPolicy`, `:288-327` : deux flags indépendants; meet commutatif, associatif et idempotent; le meet raffine chaque opérande. Aucun `join` résiduel.
- **CONFIRME** — `EffectiveCapabilities::meet`, `:442-465` : intersection de clés et drop de mismatch symétrique.
- **CONFIRME** — aucun `insert`/`extend`/`widen` public; `inner` est privé. Les seuls constructeurs supplémentaires sont `empty`/`Default`, tous deux vides.
- **CONFIRME** — le LUB Frankenstein a disparu; l’agrégateur retourne une borne réellement grantée puis la reborne à `:738`.
- **GAP** — la réduction de plus de deux grants n’est pas associative/permutation-invariante.
- **CONFIRME** — doc HITL à `:623-636` et exécution terminale à `:678-681`.
- **CONFIRME** — `hitl_gate_must_run_after_approval_raise`, `:1615-1723`, isole local, release et pack. Un gate déplacé avant l’une de ces trois couches fait réellement rougir son cas.

Le drop d’incomparables est sûr contre l’EoP, mais il est volontairement sur-atténuant : ajouter un grant peut retirer une autorité valide. C’est acceptable seulement si les doublons sont déclarés invalides et remontés comme erreur; un drop silencieux crée sinon une surface DoS/configuration.

### Qualité des 18 tests courants

| Test | Verdict |
|---|---|
| `seventeen_classes_typed` `:875` | **CONFIRME, discriminant** — 17 distinctes + match exhaustif sans wildcard |
| `intersection_only_attenuates` `:923` | **PARTIEL, non-vacuous** — 220 survivants, 215 resserrés; ne vérifie que le résultat final |
| `meet_laws_*` `:1065` | **CONFIRME, discriminant** |
| `absent_layers_are_identity` `:1097` | **CONFIRME, discriminant** |
| `expiry_enforced` `:1138` | **CONFIRME** — avant/borne/après/None + `u64::MAX` |
| `revocation_per_call` `:1178` | **CONFIRME** — branche revoked réévaluée à chaque computation |
| `audience_enforced` `:1223` | **CONFIRME** — deux directions et audiences absentes |
| `depth_enforced` `:1279` | **PARTIEL** — non-vacuous, mais fold générique plutôt que pipeline sept couches |
| `undeclared_capability_refused` `:1304` | **CONFIRME, discriminant** — smuggled déclaré et double-granté |
| `human_approval_requires_valid_decision` `:1361` | **CONFIRME** — décision per-ref, Auto/Notify/Human |
| `ordering_invariants_pinned` `:1429` | **CONFIRME** |
| `approval_meets_by_max_not_min` `:1467` | **CONFIRME** — cinq directions, dont flags orthogonaux |
| `class_mismatch_drops_fail_closed` `:1520` | **CONFIRME** |
| `os_toggle_class_filter` `:1545` | **CONFIRME** |
| `hitl_gate_must_run_after_approval_raise` `:1615` | **CONFIRME, discriminant** |
| `multi_grant_same_audience_combines_soundly` `:1731` | **GAP** — deux grants seulement; triple/permutations manquants |
| `effect_need_narrows_to_single_surviving_ref` `:1818` | **CONFIRME, discriminant** |
| `local_policy_denies_and_tightens` `:1882` | **CONFIRME, discriminant** |

### Fail-closed / EoP

- **CONFIRME** — expiry miroir exact : `grant_v0.rs:53-65` et `capability.rs:535-555`, dont `now_ms >= exp`.
- **CONFIRME** — l’engine ne dépend pas de `sbfb-manifest`, `crates/sbfb-workflow-engine/Cargo.toml:11-30`.
- **CONFIRME** — confused deputy fermé à `capability.rs:546-555,667-671`.
- **CONFIRME** — ref hors closure refusée à `:648-655`.
- **PARTIEL** — classe inconnue fermée dans l’enum, mais pas encore à la frontière String→type.
- **CONFIRME** — aucun panic explicite dans le code production. `u64::MAX` ne produit aucun overflow.
- **PARTIEL non-runtime** — `Rng::below(0)` à `:789-791` paniquerait toujours (`debug_assert` puis modulo zéro), mais ce helper est privé sous `#[cfg(test)]` et tous ses appels passent une constante positive.
- Les doublons intra-`BTreeMap` sont écrasés avant ce module sans panic; les doublons inter-grants exposent le P1 d’ordre ci-dessus.

### Docs, matrice et invariants pré-launch

- **CONFIRME** — DS-1 explicite : `orchestrator.rs:17-19,61-64`, `llms.txt:15-18,76-78`, `sprint83_plan.md:1501-1507`.
- **CONFIRME** — gate matrix : row 8 reste L (`:42`), rows 9/10/11 passent L1 (`:43-45`), row 44 L1/Q (`:78`), HUB-T2 inclut L1 (`:126`). Census mécanique : 47 rows, 0 propriétaire primaire orphelin.
- **CONFIRME** — quatre versions du chantier inchangées à 1 : `pack.rs:84`, `run_event.rs:65`, `store.rs:321`, `public_feed.rs:20`; aucun diff de constante.
- **CONFIRME** — census DOMAIN = 29; `scripts/check-frontier-contracts.sh:222-254` et gate live `clean`.
- **CONFIRME** — registre `CONTRACT_NAMES: [&str; 7]`, `contracts.rs:148-162`.
- **CONFIRME** — `contracts.schema.json` et `Cargo.toml` engine sans diff.
- **CONFIRME** — aucun derive serde dans `capability.rs`.
- Contrôles exécutés : `cargo fmt --all --check`, `git diff --check` et `bash scripts/check-frontier-contracts.sh` verts. Nextest n’a pas été relancé dans le sandbox read-only; le seed pur a été rejoué indépendamment.
- `HEAD` est resté `9498e3fa`; le worktree final comporte 8 fichiers modifiés et 7 non suivis, dont le nouveau stub Codex r2 invalide.

## Verdict par livrable

| Livrable | Verdict |
|---|---|
| `capability.rs` meet/DataPolicy | **CONFIRME** |
| Agrégation multi-grants | **GAP P1 — bloque PASS** |
| Ordre HITL | **CONFIRME** |
| 18 tests | **PARTIEL** |
| `lib.rs` exports | **CONFIRME** |
| `orchestrator.rs` DS-1 | **CONFIRME** |
| `compile.rs` | **PARTIEL P3** |
| `THREAT_MODEL.md` / `llms.txt` | **GAP P2** |
| `WORKFLOW_AGENTS_OVERVIEW.md` | **CONFIRME** |
| `sprint83_gate_matrix.md` | **CONFIRME** |
| `sprint83_plan.md` | **PARTIEL P2** — contrat de reliure à compléter |
| Preflight/reviews/canonique | **GAP P2** — preuve post-correctifs absente/stale |
| Invariants pré-launch | **CONFIRME** |

La phase doit rester `PASS-PENDING`. Avant promotion : corriger l’agrégation pour être permutation-invariante, ajouter le test à trois grants et les cas négatifs de `refines`, réaligner THREAT/llms/v27, puis produire un nouveau review canonique sur le code courant et rejouer les gates ciblées.

