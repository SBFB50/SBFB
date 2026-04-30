# Sprint 44 Phase A — preflight G8

Date : 2026-04-30 | HEAD : `ae9190e` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : "pick deepest, no band-aid" — Phase A
  resout 7 items MANDATORY reportes 3+ sprints, pas de band-aid.
- feedback_context7_systematic.md : N/A (pas de nouvelle dep/lib).

## Scans (all clean)
- S1a OSS prior art : N/A — dette batch (doc + gitignore +
  pagination + test + refactor). Pas de decision architecturale
  a challenger. APPROACH-ALIGNED.
- S1b deps : 0 nouvelle dep, 0 bump. Existing deps unchanged — clean.
- S2 historiques : 6 fichiers scannes (PATTERNS.md, .gitignore,
  apps.rs, canary_input.rs, browse.rs, http.rs). Commits
  DEVIATION/rejected trouves sur PATTERNS.md (S20/S21/S40) et
  browse.rs (S18/S20) — tous non-lies aux changes Phase A
  (doc ajout §P42/P43, pagination, as_str, RNG test, prefix
  route). Pas de conflit. Clean.
- S3 threat model : fast-path verified. Phase A n'introduit
  aucun nouveau composant securite ni wire format. HARDENING_ROADMAP
  S44 = routes restantes (Phase B/C), pas Phase A dette. Clean.
- S4 wire format : fast-path verified. canonical.rs non touche.
  VERSION=1 inchange. Day 0 preservees. Clean.

## Telemetrie preflight
- Duree totale : ~3m
- S1a : N/A (dette batch, pas de decision architecturale)
- S1b : ~30s / 0 libs nouvelles / clean
- S2 : ~1m / 6 fichiers, 10 commits scannes / clean
- S3 : fast-path / ~30s
- S4 : fast-path / ~30s

## Action
Proceder code Phase A.
