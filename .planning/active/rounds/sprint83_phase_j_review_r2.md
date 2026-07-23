# Sprint 83 Phase J — Review round r2 (post-corrections r1)

Date : 2026-07-23. Re-vérification main-thread ciblée des corrections r1
sur l'arbre corrigé (chaque fix re-mesuré, pas re-narré) + re-run des
suites.

## Verdict: PASS-PENDING

« Review OK, Codex pas encore fait » — jamais un verdict committable.

## Corrections r1 re-mesurées

- Variante `ClockRegression` orchestrateur supprimée, remplacée par la
  porte STORE `StoreError::ClockRegression` watermark-durable — câblée en
  tête de CHAQUE passe (`advance_clock_watermark`), testée par le type
  exact dans `orchestrator_clock_regression_loud` (now=500 vs watermark
  1000 → refus de la passe entière ; retour à 1500 → run complete).
- `structural_id` UTF-8-safe : `non_ascii_ids_survive_projection` vert
  (`/étape-clé~22/pièce` → `/étape-clé/pièce`).
- 7 tests neufs r1 verts : human_review_routes_decided_port,
  classifier_routes_proposal_and_defaults,
  non_idempotent_pending_goes_outcome_unknown,
  reduce_refuses_adverse_journal, join_quorum_fires_before_slow_branch,
  + asserts renforcés (parallel side-output, forged zero-rows).
- M3 étendue (`clock_watermark`) AVANT tout commit — la migration reste
  une seule M::up appendée ; restore-validation co-migrée (table + PK).
- Docs : THREAT rows TIMER-SKEW (watermark, gate Running) et SIGNAL-FORGE
  (bornes nonce/payload/squat + noms de tests exacts) ; REFERENCE.md ;
  STORE_MIGRATION_OPS ; docs 11-états ; doc amplification ; doc
  closed_at=None.

## Suites sur l'arbre corrigé

- `cargo nextest run -p sbfb-workflow-engine -p sbfb-workflow-core
  --locked` : 183/183 + 3 skip (engine 72, core 111).
- fmt --all appliqué + clippy crates -D warnings vert.
- Workspace Win + Docker `--no-fail-fast` + fmt 1.94 relancés sur l'arbre
  corrigé : verts (2320/2320 + 3 skip Win ; 2324/2324 + 3 skip Docker ;
  FMT94_OK) — compteurs consignés au commit body.

## Reste avant commit

Vérification croisée Codex GPT-5.6 Sol (rounds numérotés §4.5.8) puis
réconciliation et promotion du canonique à `## Verdict: PASS`.
