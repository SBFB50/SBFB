# Sprint 47 Phase A — preflight G8

Date : 2026-05-01 | HEAD : `dc891da` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — phase A resolves 3 carries a 2/3 avec evidence (tests + fix + audit), pas de pansement
- feedback_context7_systematic.md : N/A — pas de nouvelle lib/API dans cette phase

## Scans (all clean)
- S1a OSS prior art : N/A — phase de tests + fix mineur (invite ID prefix) + audit Python modules. Pas d'approche algorithmique a challenger.
- S1b deps : 0 nouvelle dep, 0 bump. diagnostic_api.rs, invite_api.rs, http.rs inchanges en deps — clean
- S2 historiques : 3 fichiers scannes (diagnostic_api.rs, invite_api.rs, Python modules), 0 commit DEVIATION/rejected/scope-cut sur ces fichiers — clean
- S3 threat model : fast-path verified. Pas de nouveau composant securite. Pas de ligne S47 dans HARDENING_ROADMAP — clean
- S4 wire format : fast-path verified. Phase A ne touche pas canonical.rs, schemas/, ni *_VERSION. Day 0 preservees — clean

## Telemetrie preflight
- Duree totale : ~3min
- S1a : N/A (phase tests/fix, pas d'approche algorithmique)
- S1b : 30s / 0 libs scannees (aucune nouvelle dep) / finding : clean
- S2 : 60s / 3 fichiers + archive grep / finding : clean
- S3 : fast-path / 30s
- S4 : fast-path / 30s

## Action
Proceder code phase A.
