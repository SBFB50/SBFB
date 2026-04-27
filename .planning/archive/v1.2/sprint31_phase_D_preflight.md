# Sprint 31 Phase D — preflight G8

Date : 2026-04-27 | HEAD : `687f6db` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid, research before code — N/A pour batch docs+tests
- feedback_context7_systematic.md : context7 avant code lib/API — N/A (pas de nouvelle dep, pas de nouvelle API)

## Scans (all clean)
- S1a OSS prior art : Phase D = batch docs fixes + HTTP integration tests FROST endpoints existants. Pas de decision design a challenger — APPROACH-ALIGNED (testing existing code is standard practice)
- S1b deps : frost-ed25519 2.1 workspace pinned, 0 nouvelle dep Phase D, 0 delta — clean
- S2 historiques : 4 fichiers scannes (VALIDATED_BLUEPRINT, SPLIT_INFERENCE_DESIGN, HARDENING_ROADMAP, shell-daemon/tests/), 0 rejection/DEVIATION pertinente — clean
- S3 threat model : fast-path verified (pas de nouveau composant securite, pas de nouveau wire format). HARDENING_ROADMAP S31 entry existe, Phase D la met a jour — clean
- S4 wire format : fast-path verified (docs + tests only, pas canonical.rs/schemas). VERSION=1 preserve, iroh 0.97 pin inchange, Day 0 preservees — clean

## Telemetrie preflight
- Duree totale : ~2m
- S1a : 30s / 0 projets OSS consultes (batch docs+tests, pas d'approche a challenger) / finding : clean
- S1b : 20s / 1 lib scannee (frost-ed25519 2.1) / finding : clean
- S2 : 30s / 4 fichiers scannes / finding : clean
- S3 : fast-path / 20s
- S4 : fast-path / 20s

## Note WebAppFrame
Le plan §8 mentionne "Supprimer WebAppFrame.tsx + test (si pas fait Phase B)".
Phase B commit `0771dc8` a deja effectue le cleanup (titre: "output filter E2E wire + WebAppFrame cleanup").
Phase D ne re-supprime pas — item deja resolu.

## Action
Proceder code phase D.
