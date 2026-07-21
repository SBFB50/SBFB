# Sprint 83 Phase G — Review round 2 (vérification ciblée des corrections r1)

**Sprint** : 83. **Phase** : G — Langage workflow, agents et rôles (`SBFBWorkflowGraph.v1`
+ `SBFBAgentDefinition.v1`, 4 goldens). **Date** : 2026-07-21. **HEAD** : `23fdc1dc`
(arbre sale, commit de phase à suivre). **Round précédent** :
`sprint83_phase_g_review_r1.md` (verdict CONCERN — 0 P0/P1, 5 P2, 9 P3).
**Méthode** : vérification ciblée des 9 corrections fix-now annoncées après r1, par
DEUX vérificateurs indépendants, chacune re-mesurée fichier:ligne, complétée par une
re-vérification propre du writer sur le point contesté (D5-01) et par un grep exhaustif
`cycle-free`/`cycle` sur tout le crate. Aucune correction n'est prise sur parole : chaque
statut ci-dessous est confirmé sur le fichier livré.

## Verdict: CONCERN

**8 corrections sur 9 sont RÉELLES et COMPLÈTES ; la correction 1 (D5-01) est INCOMPLÈTE.**
Aucune régression fonctionnelle (crate 64/64 reste vert, doc-only). Le CONCERN est porté
par un seul reste dur : la sur-affirmation « v1 top-level graph cycle-free » N'A PAS disparu
PARTOUT. Le module doc et le freeze §5/§8 sont bien corrigés, mais la garantie fausse subsiste
**verbatim** dans la doc du type `RepeatNode` à `graph.rs:409-411` :

> « The bound is a constant of the signed format (…), and the v1 top-level graph stays
> cycle-free because the loop body is a pinned child, not a back-edge. »

Cette phrase contredit désormais le module doc corrigé 12 lignes plus haut (`:22-23`, « a
hand-authored `edges` list can still spell an arbitrary cycle »), et c'est EXACTEMENT la
garantie fausse que D5-01 devait faire disparaître — dans un fichier source livré du commit G.
Le grep `cycle-free` sur tout le crate retourne ce seul hit résiduel (`graph.rs:410`). Le
projet interdit d'expédier une garantie fausse : ce site doit être reformulé avant que D5-01
puisse être déclaré fermé et avant le passage en Codex. Les 8 autres corrections sont
techniquement exactes et vérifiées en profondeur ; c'est pourquoi le verdict reste CONCERN
et NON FAIL (aucun P0/P1, aucun impact machine, reste unique et strictement doc-only).

## Table des 9 corrections

| # | Item | Statut | Évidence (fichier:ligne, mesurée) |
|---|---|---|---|
| 1 | **D5-01** — graph.rs loops-bounded bullet + freeze §5/§8 | **INCOMPLETE** | Module doc `graph.rs:17-26` reformulé OK (NO back-edge primitive vs « hand-authored `edges` list can still spell an arbitrary cycle: rejecting ANY cycle over arbitrary edges is a Phase H compiler refusal (§10.5) » + note DoS `u32::MAX` :24-26). freeze §5 `:84-96` réécrit OK. freeze §8 `:129-144` = 5 règles, #3 `:137-138` « Rejet de tout cycle sur edges quelconques » présent. **RÉSIDU NON SUPPRIMÉ** : `graph.rs:409-411` (doc `RepeatNode`) affirme encore verbatim « the v1 top-level graph stays cycle-free because the loop body is a pinned child, not a back-edge » — contredit `:22-23`. Grep `cycle-free` sur le crate = **1 seul hit** (`graph.rs:410`). Le fix a ciblé le module doc + freeze mais a raté ce 2e site dans le même fichier. |
| 2 | **D4-SEC-01** — freeze §6 portée `PRESENTATION_KEYS` + §8 #4 | **COMPLETE** | freeze §6 `:106-115` : « PORTÉE HONNÊTE… couvre les CHAMPS DE STRUCT et les 4 goldens committés — PAS un graphe arbitraire : les clés DATA-DRIVEN (`nodes` id, clés de `field_map`) ne sont pas couvertes… le digest faisant autorité est `content`, pas `graph_semantic` ». §8 règle #4 `:139-142` « Clés data-driven disjointes de `pack::PRESENTATION_KEYS` ». Aucune occurrence résiduelle « ne peuvent pas s'effondrer » (grep : seul hit = §6:111 = version honnête). Tests cités existent (`schema_property_names_disjoint_from_presentation_keys`, `goldens_pass_pack_semantic_view_unchanged`). |
| 3 | **D4-SEC-04** — freeze §8 #5 (`default` hors `branch_labels` + non-vide) | **COMPLETE** | freeze §8 règle #5 `:143-144` : « Labels de routage sains : `"default"` ∉ `branch_labels` (collision port réservé) et `branch_labels`/`decision_labels` non vides ». Numérotation §8 cohérente 1-5 ; intro `:129` « Cinq règles ». |
| 4 | **D4-SEC-03 + D5-02** — graph.rs budgets + zéro-secret | **COMPLETE** | D5-02 : `graph.rs:40-46` budgets reformulé (« attenuation is STRUCTURAL for tools/handoffs/context/memory… while the budget MAGNITUDE is not clamped at parse: the numeric child<=parent comparison is a Phase H compiler refusal ») ; type doc déjà précis (`:203-206` `AgentInvokeNode`). D4-SEC-03 : `graph.rs:47-55` zéro-secret honnête (« "Zero secret" holds by CONSTRUCTION only for the absence of any secret-bearing field; the free-text fields… are NOT credential-scanned here… a best-effort scan of pinned bytes is routed to the pack-assembly/Hub phases (H/T) »). Aucun « can only ATTENUATE » non-qualifié résiduel. |
| 5 | **G3-01** — ideas visibility bout-en-bout | **COMPLETE** | Node `normalise-refined` (transform) inséré : `input_schema_ref` `refine-report` → `output_schema_ref` `idea-note`, `field_map {summary:draft, visibility:visibility}`. Edges : `refine-rounds→normalise-refined→collect` (ancien `refine-rounds→collect` ABSENT). `collect.source_node_ids` mis à jour. Schémas re-lus : `refine-report`={draft,ready,visibility} et `idea-note`={summary,visibility} ⇒ le `field_map` ne mappe QUE des champs existants (`draft`,`visibility` sources présents) vers le SET requis d'`idea-note`. `champion-verdict`/`refine-input`/`refine-report` portent tous `visibility` ; prompt champion ajusté (« choose its visibility »). Branche feature conforme de bout en bout vers `idea-note` + `publish-gate` (discriminant `visibility`). Branches bug/vision intactes, aucun orphelin. Pins b3sum recalculés (12 refs en cascade) et indépendamment vérifiés — matchent. |
| 6 | **G3-02** — conception `design-angle` (item) | **COMPLETE** | `design-angle.schema.json` `:5-13` = {angle} required, `additionalProperties:false`. `conception-variant.workflow.json` référence `design-angle` en `input_schema_ref`, node `cv-in`, node `propose`. Grep `angle-list` sur le crate = 1 seul fichier restant (`conception.workflow.golden.json`, node `expand` qui PRODUIT le conteneur — légitime) ⇒ AUCUNE ref `angle-list` résiduelle dans `conception-variant`. Pins matchent. |
| 7 | **G3-03** — conception `judge-input` | **COMPLETE** | `judge-input.schema.json` `:5-20` = {variants,checks} required. `conception-judge.agent.json` `:10-13` `input_schema_ref` = `judge-input` (blake3 `d480b2ff`). Pins matchent. |
| 8 | **G3-04** — timeouts factory 5400 / ideas 2700 | **COMPLETE** | `factory.workflow.golden.json` `default_node_timeout_seconds=5400` ; max node `wall_clock` = `drive-implementation` 5400 (==, couvre). `ideas.workflow.golden.json` `default_node_timeout_seconds=2700` ; max node `wall_clock` = `deep-research` 2700 (==, couvre). Aucun autre node ne dépasse (factory `independent-review` 3600 / `scan-threat` 600 ; ideas `champion-review` 1800 / `synthesise` 600 / `repro-check` 900 / `triage` 300). Conception inchangé 2700 couvre son max (`judge` 2700). Pas de régression. |
| 9 | **D2-01** — agent_definition control helper | **COMPLETE** | `agent_definition.rs:289-292` `fn control_base_parses()` parse la fixture de base avec `.expect` ; appelé en tête des 4 tests négatifs (`missing_budget…:296`, `missing_budget_dimension…:310`, `missing_context_policy…:323`, `unknown_field_is_rejected:331`). Doc `:287-288` explicite l'intention (un `expect_err` ne peut plus passer pour la mauvaise raison). Les 4 compilent et passent (inclus dans 64/64). |

## Régressions

- **1 incohérence doc-only, corollaire direct du D5-01 partiel** : le module doc corrigé
  (`graph.rs:22-23`) dit qu'un `edges` list hand-authored peut porter un cycle arbitraire,
  tandis que la doc de type `RepeatNode` (`:409-411`) affirme toujours catégoriquement que
  « the v1 top-level graph stays cycle-free ». Avant le fix, les deux sites sur-affirmaient
  uniformément ; après le fix, un seul est corrigé ⇒ le fichier se contredit. C'est le même
  reste que la correction 1 INCOMPLETE, pas une régression distincte. **Pas de régression
  fonctionnelle ni de test** (crate 64/64 reste vert, doc-only).
- **Aucune autre régression**. Les corrections 5-9 (goldens/tests) et 2-4 (docs) ne cassent
  aucune suite : `four_goldens_validate`, `golden_pins_resolve`, `golden_negative_controls_reject`,
  `golden_richness_floor` (18 kinds toujours exercés malgré l'ajout du node `transform`
  `normalise-refined`), `node_profile_frozen`, `goldens_pass_pack_semantic_view_unchanged`,
  `factory_golden_matches_process_profile` restent verts. Pins blake3 : les refs en cascade
  (schémas feuilles + fichiers agent/subworkflow + `agent_refs`/`subworkflow_refs` des goldens)
  recalculées et vérifiées `b3sum(canonicalise)` — toutes matchent ; aucun blake3 vide dans les
  4 goldens ni les stores.

## P3 restants ACCEPTÉS-documentés de r1 (inchangés, non re-débattus)

- **G3-05 (INFO)** — enveloppe factory `max_total_tokens=4M` < somme des maxima déclarés
  (~5,9M) ; plafonds ≠ réservations donc non-invalide. Optionnel : relever le plafond ou
  documenter le bornage volontaire. Non bloquant.
- **G6-02 (P3)** — deux renommages PLAN-ADAPT sans perte de couverture
  (`node_profile_18_frozen`→`node_profile_frozen` SET-check ;
  `…graph_schema_matches_snapshot`→`schema_snapshot_stub` étendu). À NOMMER dans la section
  traçabilité du body G.
- **G6-03 (P3)** — clôture docs-contrat GUIDE/llms.txt des 2 frontières neuves
  (`SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1`) différée au wrap-up S83 (AM-3) ; split
  correct §6.12. À NOMMER en Carry-closure du body pour que le Track K docs-contrat de l'audit
  gate la retrouve.
- **D4-SEC-02 (P3)** — bornes `NonZero` sans plafond haut (`u32::MAX` admissible) ; risque
  DoS-dans-le-budget, l'invariant d'autorité tient (borne = constante signée). Note H-point
  posée au module doc `graph.rs:24-26`. Doc-only, non bloquant.
- **D4-SEC-03 (P3)** — note honnête « zéro secret par construction » + scan best-effort routé
  Phase H/T : **posée** (couverte par la correction 4). Fermée côté doc.

## État de la vérification (mesuré ce jour, arbre sale)

- **crate `sbfb-workflow-core`** : `cargo nextest -p` **64/64 PASS, 0 skip** (inclut les
  suites goldens, freeze, drift-gate, contrôles négatifs, helper D2-01).
- **fmt 0 ; clippy `--all-targets` -D warnings 0** (mesuré post-fix).
- **Pins blake3 recalculés en cascade** après les corrections 5-8 (`UPDATE_SNAPSHOTS` sous
  workspace), indépendamment re-vérifiés `b3sum(canonicalise)` — 12 refs, toutes matchent.
- **Docker `sbfb-ci` r1** vert (fmt 1.94 + nextest 2213 dont 1 flake re-run solo PASS).
- **Re-scellement workspace Win + Docker EN COURS** (fan-out parallèle) = **GATE PRE-COMMIT** :
  doit être observé vert avant le commit de phase.
- **Livrables planning untracked (G6-01, action commit-time)** : à `git add` explicitement au
  commit (`sprint83_node_profile_freeze.md` livrable G.2 + critère G.4 ;
  `sprint83_phase_g_preflight.md` autorité de phase) ; vérifier `git diff --cached --stat`.

## Prochaine étape

1. **Fermer D5-01** : reformuler `graph.rs:409-411` (doc `RepeatNode`) pour supprimer
   l'affirmation catégorique « the v1 top-level graph stays cycle-free » — p. ex. « le node
   `repeat` délègue à un enfant PINNÉ content-addressé et n'introduit donc AUCUNE back-edge ;
   le rejet de tout cycle sur `edges` quelconques reste un refus compilateur Phase H (§10.5) ».
   Après ce fix, re-grep `cycle-free` = 0 hit ⇒ D5-01 fermé.
2. **Gate pré-commit** : observer le re-scellement workspace Win + Docker `sbfb-ci` vert.
3. **Stager** les 2 `.md` planning (G6-01) et **nommer au body** les PLAN-ADAPT (G6-02) +
   la dette docs-contrat AM-3 (G6-03).
4. **Codex round 1** : le verdict PASS-PENDING/PASS suit les rounds Codex, PAS ce round de
   review. Ce round reste CONCERN tant que D5-01 n'est pas fermé.
