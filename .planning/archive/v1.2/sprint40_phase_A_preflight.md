# Sprint 40 Phase A — preflight G8

Date : 2026-04-29 | HEAD : `60ff6f5` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest technical option, no band-aid fixes, research before code. Phase A is dette pair — fixes are root-cause corrections not band-aids. ALIGNED.
- feedback_context7_systematic.md : N/A — 0 new deps, 0 new lib/API/spec.

## Scans (all clean)
- S1a OSS prior art : phase dette (corrections code existant), pas de nouveau design. Guardrail chain singleton = standard pattern (OnceLock/lazy_static). Substring early exit = standard optimization. APPROACH-ALIGNED — clean.
- S1b deps : 0 nouvelle dep, 0 bump — clean.
- S2 historiques : 5 fichiers cibles scannes (`git log --grep` DEVIATION/rejected/scope-cut/threat-model), 0 decision historique contredite par les 5 items dette. `feedback_approach.md` grep singleton/chain/substring/event_tx/lowercase : 0 match. Archive planning grep : 0 match. — clean.
- S3 threat model : fast-path verified. Pas de nouveau composant securite. HARDENING_ROADMAP §3 S40 : pas de ligne specifique (S40 = migration Tier 2 fin + Tier 3, pas hardening). — clean.
- S4 wire format : fast-path verified. canonical.rs non touche. `*_VERSION` fields : tous a 1 (TASK_FORMAT_VERSION, ANNOUNCEMENT_VERSION, AGE_WITNESS_VERSION, etc.). Day 0 preservees. Pre-launch protocol intact. — clean.

## Telemetrie preflight
- Duree totale : ~2m
- S1a : 30s / 0 projets OSS consultes (phase dette, pas de design) / finding : clean
- S1b : 10s / 0 libs scannees (0 new dep) / finding : clean
- S2 : 40s / 5 fichiers + archive grep / finding : clean
- S3 : fast-path / 20s
- S4 : fast-path / 20s

## Action
Proceder code phase A.
