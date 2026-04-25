# Sprint 27 Phase A — preflight G8

Date : 2026-04-25 | HEAD : `5586d0b` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : no band-aids, pick deepest, research before code — Phase A P2 batch = cleanup d'abord features ensuite, aligne
- Aucune zone specifique (pas kudos, pas governance, pas deploy, pas lib externe)

## Scans (all clean)
- S1a OSS prior art : N/A — P2 batch (7 fixes code quality), pas de design decision a challenger. Aucune approche OSS alternative applicable.
- S1b deps : 0 nouvelle dep, 0 bump — clean
- S2 historiques : 7 fichiers scannes, 6 commits historiques trouves (S4/S7/S9/S18/S21) — aucun ne contredit les 7 fixes P2. Pas de DEVIATION/rejected sur le perimetre Phase A — clean
- S3 threat model : fast-path verified, Phase A ne cree aucun nouveau composant securite ni wire format. HARDENING_ROADMAP S27 items = Phases B-D — clean
- S4 wire format : fast-path verified, aucun canonical.rs/schemas/*_VERSION touche. EtwWriter rename = infra audit trail (pas wire format). VERSION=1 preserve, Day 0 preserved — clean

## Telemetrie preflight
- Duree totale : ~1m30s
- S1a : N/A (P2 batch, pas de design)
- S1b : <30s / 0 libs nouvelles / clean
- S2 : <30s / 7 fichiers, 6 commits / clean
- S3 : fast-path / <15s
- S4 : fast-path / <15s

## Action
Proceder code Phase A.
