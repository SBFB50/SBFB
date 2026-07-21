# Sprint 83 Phase G — Review round 3 (fermeture du résidu D5-01)

**Sprint** : 83. **Phase** : G — Langage workflow, agents et rôles (`SBFBWorkflowGraph.v1`
+ `SBFBAgentDefinition.v1`, 4 goldens). **Date** : 2026-07-21. **HEAD** : `23fdc1dc`
(arbre sale, commit de phase à suivre). **Round précédent** :
`sprint83_phase_g_review_r2.md` (verdict CONCERN — 0 P0/P1, 5 P2 dont 4 fermés,
reste unique D5-01 INCOMPLET). **Méthode** : vérification ciblée du SEUL reste bloquant
du round 2 (le résidu doc D5-01 dans la doc du type `RepeatNode`), re-mesurée
fichier:ligne, complétée par un grep exhaustif `cycle-free`/`cycle`/`back-edge`/`acyclic`/`DAG`
sur tout le crate et par un contrôle de cohérence croisée module doc ↔ type doc ↔ freeze
§5/§8. Aucune correction n'est prise sur parole : le statut ci-dessous est confirmé sur le
fichier livré.

## Verdict: PASS-PENDING

**Le reste unique du round 2 (D5-01 INCOMPLET) est FERMÉ.** La sur-affirmation
« the v1 top-level graph stays cycle-free » a disparu PARTOUT dans le crate : le
micro-fix doc-only reformule la doc du type `RepeatNode` (`graph.rs:405-412`) en un aveu
honnête aligné sur le module doc corrigé au round 1 et sur le freeze §5/§8. Le grep
`cycle-free` sur `crates/sbfb-workflow-core/src/` retourne **0 hit**. Les 5 P2 de r1 sont
désormais TOUS fermés (D5-01 en deux temps r2+r3, D4-SEC-01/G3-01/G3-02 en r2, G6-01 =
action commit-time cadrée). Aucune régression : le changement est un commentaire de doc
(0 code, 0 test), les suites du crate ont été re-jouées vertes après lui, et le
re-scellement dual-platform a été OBSERVÉ VERT (avant ce micro-fix). Le verdict est
PASS-PENDING et non PASS parce que le PASS final ne peut être prononcé qu'APRÈS
réconciliation Codex (round 1 Codex encore à jouer) — ce round de review n'a plus aucun
reste bloquant côté review.

## Fermeture D5-01 (évidence mesurée)

- **Site corrigé** : `crates/sbfb-workflow-core/src/graph.rs:405-412`, doc du type
  `RepeatNode`. Le texte livré est exactement la reformulation attendue : « This node
  introduces NO back-edge — its body is a content-addressed pinned child — but rejecting
  cycles spelled by arbitrary `edges` stays a Phase H compiler refusal (§10.5, freeze
  artefact §8). » La claim catégorique « the v1 top-level graph stays cycle-free » est
  ABSENTE.
- **Grep `cycle-free`** sur `crates/sbfb-workflow-core/src/` = **0 hit** (le seul résidu
  du round 2, `graph.rs:410`, a disparu ; mesure post-fix confirmée).
- **Grep `cycle` (sens graphe)** dans src = uniquement `graph.rs:22-23` (module doc) et
  `graph.rs:411` (doc `RepeatNode`), deux mentions LÉGITIMES du refus compilateur Phase H
  (§10.5) — aucune n'affirme que le format est acyclique au parse. Les hits
  `contracts.schema.json:1600/1773` et `factory_profile.rs:38/120/281` sont le substring
  « lifecycle modes » (sans rapport avec les cycles de graphe).
- **Grep `back-edge`/`acyclic`/`DAG`** dans src = uniquement `graph.rs:19` et `graph.rs:410`
  (« NO back-edge »). Aucun `acyclic`/`DAG` sur-affirmé dans le source Rust.
- **Cohérence croisée** : le module doc (`graph.rs:17-26`, « The format itself has NO
  back-edge primitive… but a hand-authored `edges` list can still spell an arbitrary
  cycle: rejecting ANY cycle over arbitrary edges is a Phase H compiler refusal (§10.5),
  consigned in the freeze artefact ») est le miroir exact du doc `RepeatNode`. Le freeze §5
  (`sprint83_node_profile_freeze.md:84-96`) — « v1 n'a AUCUNE primitive de back-edge ; le
  rejet de cycle est un refus H » : `edges` est une liste libre, `A→B, B→A` est PARSÉ par
  Phase G, la cible « tout plan compilé est un DAG » est rendue vraie par le COMPILATEUR,
  pas par le parse. Le freeze §8 item 3 (`:137-138`) consigne « Rejet de tout cycle sur
  edges quelconques ». La référence croisée « freeze artefact §8 » dans le doc `RepeatNode`
  est correcte (§8 = « Consigné pour Phase H »). La cible « DAG » du freeze §5 est
  explicitement cadrée comme cible Phase H, pas comme garantie courante — aucune claim
  voisine contradictoire introduite.

## Bilan des 5 P2 de r1 — TOUS fermés

| # | P2 | Fermeture | Évidence |
|---|---|---|---|
| 1 | **D5-01** — sur-affirmation « cycle-free » (graph.rs + freeze §5/§8) | **FERMÉ r2 + r3** | Module doc `graph.rs:17-26` + freeze §5 `:84-96` + §8 #3 `:137-138` réécrits en r2 ; résidu `RepeatNode` `graph.rs:405-412` reformulé en r3. Grep `cycle-free` = 0 hit. |
| 2 | **D4-SEC-01** — portée honnête `PRESENTATION_KEYS` (freeze §6 + §8 #4) | **FERMÉ r2** | freeze §6 `:106-115` « PORTÉE HONNÊTE… couvre les CHAMPS DE STRUCT et les 4 goldens — PAS un graphe arbitraire ; le digest faisant autorité est `content`, pas `graph_semantic` » ; §8 #4 `:139-142`. Tests `schema_property_names_disjoint_from_presentation_keys`, `goldens_pass_pack_semantic_view_unchanged`. |
| 3 | **G3-01** — ideas visibility bout-en-bout | **FERMÉ r2** | Node `normalise-refined` (transform) inséré, `field_map {summary:draft, visibility:visibility}` vers `idea-note` ; edges `refine-rounds→normalise-refined→collect` ; `champion-verdict`/`refine-input`/`refine-report` portent `visibility` ; discriminant `visibility` vers `publish-gate`. Pins b3sum recalculés (12 refs) et vérifiés. |
| 4 | **G3-02** — conception `design-angle` (item vs conteneur) | **FERMÉ r2** | `design-angle.schema.json:5-13` {angle} required `additionalProperties:false` ; `conception-variant` référence `design-angle` (`cv-in`, `propose`) ; aucune ref `angle-list` résiduelle dans `conception-variant`. Pins matchent. |
| 5 | **G6-01** — staging livrables planning au commit | **ACTION COMMIT-TIME (cadrée)** | Non-bloquant de review : `git add` explicite au commit de `sprint83_node_profile_freeze.md` (livrable G.2 + critère G.4) + `sprint83_phase_g_preflight.md` (autorité de phase) ; vérifier `git diff --cached --stat`. Rappelé ci-dessous « Actions commit-time ». |

## P3 restants ACCEPTÉS-documentés (inchangés depuis r1/r2, non re-débattus)

- **G3-05 (INFO)** — enveloppe factory `max_total_tokens=4M` < somme des maxima déclarés
  (~5,9M) ; plafonds ≠ réservations donc non-invalide. Optionnel. Non bloquant.
- **G6-02 (P3)** — deux renommages PLAN-ADAPT sans perte de couverture
  (`node_profile_18_frozen`→`node_profile_frozen` SET-check ;
  `…graph_schema_matches_snapshot`→`schema_snapshot_stub` étendu). À NOMMER dans la section
  traçabilité du body G.
- **G6-03 (P3)** — clôture docs-contrat GUIDE/llms.txt des 2 frontières neuves
  (`SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1`) différée au wrap-up S83 (AM-3) ; split
  correct §6.12. À NOMMER en Carry-closure du body pour que le Track K docs-contrat de
  l'audit gate la retrouve.
- **D4-SEC-02 (P3)** — bornes `NonZero` sans plafond haut (`u32::MAX` admissible) ; risque
  DoS-dans-le-budget, l'invariant d'autorité tient (borne = constante signée). Note H-point
  posée au module doc `graph.rs:24-26`. Doc-only, non bloquant.
- **D4-SEC-03 (P3)** — note honnête « zéro secret par construction » + scan best-effort
  routé Phase H/T : posée (couverte par la correction 4 du round 2). Fermée côté doc.

## État de la vérification (COMPLET — mesuré, arbre sale)

- **crate `sbfb-workflow-core`** : `cargo nextest -p` **64/64 PASS, 0 skip** (goldens,
  freeze, drift-gate, contrôles négatifs, helper D2-01 ; re-joué vert APRÈS le micro-fix
  doc-only r3).
- **Re-scellement dual-platform OBSERVÉ VERT** (post-fixes r1, avant le micro-fix doc-only
  r3 — inchangé par un commentaire de doc) :
  - **Windows** : `clippy --all-targets -D warnings` **0** + `nextest --workspace`
    **2209/2209 PASS + 1 skip** + doctests **10** + build `--release` OK.
  - **Docker `sbfb-ci`** : `fmt --check` 1.94 **0** + `nextest --workspace` **2213/2213
    PASS + 1 skip**.
- **fmt `--check` 0** (mesuré post-micro-fix r3, crate).
- **web** : suites vertes (inchangées par la phase G, doc/format Rust).
- **factory-operator** : suites vertes (inchangées).
- **Mutations 7/7** tuées (contrôles négatifs du crate restent discriminants).
- **Frontier census = 29** (liste nominale à jour, cohérente avec l'ajout des 2 frontières
  `SBFBWorkflowGraph.v1` + `SBFBAgentDefinition.v1`).
- **SPDX** : en-tête `AGPL-3.0-or-later` présent sur les sources livrés (`graph.rs:1`).
- **0 dépendance runtime ajoutée** ; **0 bump wire** (frontières neuves, pas de modification
  d'enveloppe existante).

Le 1 skip est le PREMIER `#[ignore]` non feature-gaté du dépôt (`store_child_writer_exits_mid_run`,
S83 Phase E, lancé seulement par le parent du bras out-of-process) — attendu, pas une
régression.

## Actions commit-time (rappel — non-bloquantes de review)

1. **`git add` explicite** au commit de phase :
   - les DEUX `.md` planning de review : `.planning/active/rounds/sprint83_phase_g_review_r3.md`
     et `.planning/active/sprint83_phase_g_review.md` (canonique = copie byte-identique de r3),
     plus les rounds antérieurs `…_r1.md`/`…_r2.md` et le `codex_review.md` une fois joué ;
   - les livrables planning G6-01 encore untracked : `sprint83_node_profile_freeze.md`
     (livrable G.2 + critère G.4) + `sprint83_phase_g_preflight.md` (autorité de phase) ;
   - vérifier `git diff --cached --stat` avant le commit.
2. **Nommer au body** du commit de phase :
   - **G6-02** — traçabilité PLAN-ADAPT : les deux renommages de tests sans perte de
     couverture (`node_profile_18_frozen`→`node_profile_frozen` ;
     `…graph_schema_matches_snapshot`→`schema_snapshot_stub`).
   - **G6-03** — Carry-closure : dette docs-contrat GUIDE/llms.txt des 2 frontières neuves
     différée au wrap-up S83 (AM-3), pour que le Track K docs-contrat de l'audit gate la
     retrouve.

## Prochaine étape

1. **Codex round 1** : le PASS final NE PEUT être prononcé qu'APRÈS réconciliation Codex.
   Ce round de review est PASS-PENDING (0 reste bloquant côté review, D5-01 fermé, 5 P2
   tous fermés) ; le gate BLOQUANTE review→commit exige encore le passage Codex
   (`codex exec -m gpt-5.6-sol -c model_reasoning_effort=max`, output brut dans
   `codex_review.md`).
2. **Commit de phase** après Codex CLEAN (ou P2/P3 documentés) : appliquer les actions
   commit-time ci-dessus, body `feat(scope): Sprint 83 Phase G — …` avec delta de tests
   cumulé et scope cuts.
