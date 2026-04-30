# Sprint 44 Phase B — preflight G8

Date : 2026-04-30 | HEAD : `0ef7358` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest, no band-aid" — Phase B
  porte 4 routes API existantes, pattern S42-S43 etabli.
- feedback_kudos_non_monetary.md : kudos = reputation non-transferable.
  Phase B = list entries + leaderboard (lecture seule). Pas de
  cost/deposit/stake. Respecte.
- feedback_context7_systematic.md : 0 nouvelle dep. N/A.

## Scans (all clean)
- S1a OSS prior art : port de routes Python existantes vers
  axum Rust. Pattern etabli S35-S43 (15+ routes portees). Pas de
  decision architecturale nouvelle. APPROACH-ALIGNED.
- S1b deps : 0 nouvelle dep, 0 bump — clean.
- S2 historiques : 5 fichiers scannes (http.rs, main.rs,
  kudos_ledger.rs, fairness.rs, db.rs). Commits DEVIATION/rejected
  trouves sur http.rs (S36/S39/S40) et main.rs (S18/S7) — tous
  non-lies aux routes health/shell/kudos/diagnostic. Clean.
- S3 threat model : fast-path verified. Phase B n'introduit
  aucun nouveau composant securite ni wire format. Routes = lecture
  d'etat existant (health, shell discover, kudos entries,
  fairness metrics). Clean.
- S4 wire format : fast-path verified. canonical.rs non touche.
  VERSION=1 inchange. Day 0 preservees. Clean.

## G1 blind spots adresses (design review)
- D2(a) health.py payload : health handler = coordinator state
  snapshot. Pas de breaking change, nouveau endpoint.
- D2(e) diagnostic.py fairness.rs : fairness.rs expose
  compute_gini/compute_top_k_share/compute_churn_rate publics,
  prennent &[f64] ou &[KudosEntry]. Wire direct HTTP→fairness.

## Telemetrie preflight
- Duree totale : ~2m
- S1a : N/A (port pattern etabli)
- S1b : ~30s / 0 libs nouvelles / clean
- S2 : ~1m / 5 fichiers, 5 commits scannes / clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Proceder code Phase B.
