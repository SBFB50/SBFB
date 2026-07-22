# Sprint 83 Phase I — Review r3 (réconciliation Codex r2)

## Verdict: PASS-PENDING

Réconciliation du round Codex r2 (**GAP : 0 P0, 2 P1, 2 P2** —
`rounds/sprint83_phase_i_codex_review_r2.md`, brut jamais réécrit). Les 4
findings jugés RÉELS, fermés par code + tests + mutations. PASS-PENDING en
attente de Codex r3 (confirmation des fermetures).

## Fermetures

1. **P1 — états clos non absorbants** (`record_run_completed` écrasait un
   TERMINAL en completed → GC branche completed → diagnostic purgeable).
   FIX : transitions GARDÉES dans la transaction (`UPDATE … WHERE status =
   'running'`, `rows_affected == 1` sinon `StoreError::InvalidTransition`
   NEUVE) sur `record_run_completed` ET `record_terminal`. Conséquence de
   sémantique (PLAN-ADAPT tracé) : un replay divergent d'un run COMPLETED
   retourne `StepHashMismatch` SANS réécrire l'histoire du run (l'engine
   ne journalise le diagnostic terminal que sur un run RUNNING) — 2 tests
   E adaptés, golden terminal RÉGÉNÉRÉ sur un scénario produisant un vrai
   terminal (crash B1 puis resume divergent, les deux hashes pinnés).
   **Mutation M9** (garde retirée) → rouge.
2. **P1 — validation backup = noms de tables** : l'inventaire vérifie
   maintenant les COLONNES par table et par version (pragma typé
   `table_info`, zéro SQL construit) + l'index unique
   `outbox_one_effect_per_node` ; les TYPES ne sont pas comparés (borne
   documentée : types menteurs échouent loud en aval, fail-closed). Test
   forgeage 6-tables-bons-noms-colonnes-étrangères (index inclus pour que
   le bras colonnes soit discriminant) → refus. **Mutation M10**
   (inventaire colonnes neutralisé) → rouge — après durcissement du
   forgeage (1er essai : l'index manquant masquait la mutation, corrigé).
3. **P2 — `keep_last` non relu** : la fenêtre protégée est RE-CLASSÉE
   dans la transaction de purge (même tri que le scan) ; test
   d'interleaving (re-datation via connexion séparée → run épargné).
4. **P2 — séquence snapshot négative** : `Poisoned` (cache ignorable),
   plus une erreur dure ; testé.

Docs : §19 (GC re-lecture + keep_last + absorbant M9 ; restore inventaire
colonnes+index+quick_check ; bornes types honnêtes) + STORE_MIGRATION_OPS
alignés.

## Suites à la clôture de r3

Crates workflow Win 146 passed + 2 skipped ; **10/10 mutations tuées**
(M1..M8 + M9 transitions + M10 colonnes) ; fmt OK ; workspace Win + Docker
canonique relancés (compteurs finaux au body).

## Prochaine étape

Codex r3 (confirmation) puis promotion du canonique
`sprint83_phase_i_review.md` à `## Verdict: PASS` et commit.
