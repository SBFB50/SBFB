# Sprint 47 Phase C — preflight G8

Date : 2026-05-01 | HEAD : `d86edd6` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — happy path tests (pas de skip)
- feedback_cd_web_trap.md : JAMAIS cd web dans Bash chaine — utiliser subshell
- feedback_context7_systematic.md : N/A — pas de nouvelle dep

## Scans (all clean)
- S1a OSS prior art : N/A — phase tests + cleanup frontend aliases
- S1b deps : 0 nouvelle dep — clean
- S2 historiques : consent.rs, files.rs, coordinator.ts scannes, 0 DEVIATION — clean
- S3 threat model : fast-path. Pas de nouveau composant securite — clean
- S4 wire format : fast-path. Phase C ne touche pas canonical/schemas — clean

## Note G1 D4
Finding D4 acknowledge : handlers consent/files utilisent sbfb_home()
(env var SBFB_HOME), PAS DaemonHttpState. Tests setteront SBFB_HOME
vers tmpdir.

## Action
Proceder code phase C.
