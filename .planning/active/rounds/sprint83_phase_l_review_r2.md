# Sprint 83 — Phase L review (round r2) — Permissions et atténuation

## Verdict: PASS-PENDING

Round r2 = re-review après application des correctifs r1. Le round r1 (Workflow
8 agents Opus 4.8 1M, `rounds/sprint83_phase_l_review_r1.md`) avait rendu
PASS-PENDING, **0 P0/P1**, avec **3 P2 substantiels + 12 P3/observations**.
Tous traités ci-dessous ; **aucun ne touchait l'algèbre livrée** sauf F3
(raffinement de type délibéré). PASS-PENDING (pas PASS) car la gate Codex
GPT-5.6 Sol effort max n'a pas encore tourné.

## Correctifs r1 appliqués (par finding)

### P2 (fermés)

- **F1 — invariant d'ordre HITL faux + placement non testé** → FERMÉ.
  Le docstring de `effective_capabilities` est réécrit : les couches 1-6
  (key-filters + field-meets) commutent, mais **la couche 7 (gate HITL) DOIT
  rester en dernier** car elle lit `cap.approval`, champ que les meets 2/3/4
  ÉLÈVENT par `max` — la faire tourner avant laisserait une capacité montée à
  `Human` passer sans décision (fail-open). Test discriminant NEUF
  `hitl_gate_must_run_after_approval_raise` (declared=`Auto` + grant=`Human`
  + `HitlValidity::none()` → drop ; avec décision → survit `Human`).
  **Mutation M7** (réordonner la couche 7 avant les grants) → le test ROUGIT
  (prouvé).

- **F2 — couche 2 (policy locale) sans test discriminant** → FERMÉ.
  Test NEUF `local_policy_denies_and_tightens` : `RefConstraint::Only(map)`
  où (a) une ref absente de l'allow-map est droppée, (b) une ref présente est
  meet-tightened à la borne EXACTE de la policy (`quota=AtMost(1)`,
  `approval=Human`). **Mutation M8** (`meet_with_allow → eff.clone()`, no-op)
  → le test ROUGIT (prouvé). C'était un vrai trou de couverture — la campagne
  de mutation initiale l'avait manqué (`feedback_mutation_before_trust`).

- **F3 — linéarisation `DataPolicy` sur axes orthogonaux** → FERMÉ (option b,
  la plus profonde). `DataPolicy` passe d'un enum ordonné (`Unrestricted <
  LocalOnly < Ephemeral`, meet=`max`) à un **struct de flags INDÉPENDANTS**
  (`no_egress`, `no_persist`), meet = UNION (chaque flag OR), join = INTER
  (AND), refines = flag-wise `>=`. Le meet ne perd JAMAIS une contrainte : une
  donnée `no_persist` rencontrant une donnée `no_egress` garde les DEUX (une
  localité éphémère qui sortait du nœud est impossible). Pas de `Ord` total
  (l'ordre est partiel). `approval_meets_by_max_not_min` prouve l'union
  (`meet({no_persist}, {no_egress}) == {no_egress, no_persist}`). **Mutation
  M9** (drop de l'OR d'un flag) → le test ROUGIT (prouvé).

### P3 (fermés ou documentés)

- **F4** liste d'ordre du commentaire vs code → FERMÉ (docstring réécrit en
  invariants, pas en séquence numérotée trompeuse).
- **F5** property tests sans témoin de non-vacuité → FERMÉ **et révélateur** :
  les témoins ajoutés (`saw_nonempty_after_all_layers`,
  `saw_survivor_strictly_tightened`) ont montré que
  `intersection_only_attenuates` était **VACUOUS** — sur 2000 itérations, le
  générateur (grants fail-closed double-audience aléatoires) ne produisait
  JAMAIS de survivant, donc le test « cardinal » ne prouvait rien
  (`empty.refines(base)` trivial). Le générateur est réécrit : les grants
  couvrent les refs déclarées avec la MÊME classe et des bornes aléatoires,
  la closure garde les refs déclarées avec prob 3/4 → survivants fréquents,
  atténuation par champ RÉELLEMENT exercée. Le bypass closure est désormais
  aussi tué par `intersection_only_attenuates` (il ne l'était pas avant).
- **F6** trace plan de la reliure DS-1 → FERMÉ : §Phase V.1 du plan reçoit la
  ligne de câblage `effective_capabilities → run_effect` (recompute
  par-attempt `now_ms` watermarké, refus journalisés, ferme
  T-CAP-EXPIRY-TOCTOU-RETRY / T-CAP-REPLAY-REAUTHORIZE).
- **F7** §18 CLASSIFICATION-LAUNDER présent-accompli → FERMÉ : conditionné
  comme la row LLM-MUTATION (« fermera … une fois la computation reliée au
  dispatch, DS-1 ; aujourd'hui L ne juge rien per-effet »).
- **F8** anchors `compile.rs:27`/`:607` → `:27` FERMÉ (flip honnête « delivered
  in `sbfb-workflow-engine::capability`, Phase L ; per-effect binding = run-
  surface phase ») ; **`:607` LAISSÉ SCIEMMENT** — c'est un doc de CHAMP
  schemars (alimente `contracts.schema.json`) : l'éditer régénérerait le
  golden core (le preflight §6 exige `contracts.schema.json` byte-intact) pour
  une imprécision P3 dans un commentaire pré-existant (Phase H) dont le texte
  « Phases J/K/L » n'est pas faux (le binding s'étend bien sur ces phases).
- **F9** OVERVIEW:44 role-cell → FERMÉ (surface permissions ajoutée).
- **F10 / F13 / F15** notes de reliure (plancher d'approbation depuis
  `declared`, ordre de grants stable, closure = `keys()`) → FERMÉES : section
  NEUVE « Notes for the binding phase (DS-1) » dans le module doc, captées
  in-code là où l'auteur de la phase run-surface les lira.
- **F11** borne de domaine appliquée une fois → FERMÉ (commentaire d'invariant
  « aucune couche postérieure n'insère de clé »).
- **F12** `Rng::below(0)` panic latent → FERMÉ (`debug_assert!(n > 0)`).
- **F14** aveuglement sur-atténuation (observation availability) → documenté
  au body ; non-corrigé (exigerait un oracle « eff == intersection attendue »,
  hors scope L, fail-closed-safe).

## Points vérifiés OK (repris de r1, ré-confirmés)

- Algèbre : directions de meet par champ justes (dont data_policy union
  désormais) ; cohérence meet↔refines ; lois de semi-treillis sous drop
  symétrique ; **aucun chemin de widening** (constructeur unique, 0 insert/
  widen public, JOIN intra-audience toujours re-borné par le meet externe).
- **9 mutations discriminantes tuées** : approval max→min, closure bypass,
  Ceiling min→max, HITL gate drop, live_for audience, refines direction, +
  **M7 ordre HITL, M8 couche 2 no-op, M9 union data_policy** (les 3 neuves).
- PA-1..PA-8, DS-1 (module PUR, 0 call-site dispatch), charte pure (0 serde
  derive, 0 I/O, trait injecté), invariants pré-launch (0 bump wire, census
  DOMAIN 29, `contracts.rs` len==7, goldens byte-intacts, 0 dep neuve).

## Suites §7.4

- Gate G-CAPABILITY (`cargo nextest run -p sbfb-workflow-engine capability`)
  = **15/15 vert** (13 initiaux + `hitl_gate_must_run_after_approval_raise` +
  `local_policy_denies_and_tightens`).
- Workspace Windows (fmt/clippy/nextest/doctest/release) + Docker sbfb-ci
  relancés sur l'arbre CORRIGÉ — résultats à réconcilier avant commit.

## P2/P3 pour le body de commit

Tous les P2 (F1/F2/F3) FERMÉS. P3 : F8:`:607` laissé sciemment (golden intact),
F14 aveuglement sur-atténuation documenté. Le reste fermé.

## Décision

0 P0/P1, 3 P2 fermés, P3 fermés/documentés. Promotion en PASS conditionnée à
la boucle Codex (critère « CLEAN ou P2/P3 documentés ») + réconciliation §7.4
dual-platform.
