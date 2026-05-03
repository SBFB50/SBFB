# Sprint 52 Phase B — preflight G8

Date : 2026-05-02 | HEAD : `22695ed` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — validation workflow existant, pas de nouveau design
- feedback_context7_systematic.md : N/A (pas de nouvelle lib)
- Tensions plan vs memory : aucune

## Scans (all clean)
- S1a OSS prior art : phase validation CI workflow, pas de design a challenger — APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep. G1 D3 flag cosign v2.4.1 vs v3.0.6 — Phase B verifie inline — clean
- S2 historiques : release.yml cree S18 Phase B, pas de decision rejetee sur le workflow — clean
- S3 threat model : fast-path verified, phase ne cree aucun composant securite — clean
- S4 wire format : fast-path verified, phase ne touche pas canonical.rs/schemas — clean

## Telemetrie preflight
- Duree totale : <1m
- S1a : N/A (validation, pas design) / clean
- S1b : <15s / 0 libs / clean (cosign check delegue Phase B inline)
- S2 : <15s / clean
- S3 : fast-path / <10s
- S4 : fast-path / <10s

## Pivot utilisateur (mid-phase)

Plan original : validation GHA workflow_dispatch.
Pivot : design doc self-hosted build (LT-7 pre-v1.0).
Raison : "le reseau qui ne peut pas se compiler lui-meme n'est pas
un reseau de compute — c'est un wrapper autour de GHA".
Le fix matrice GHA est deja pushe (bootstrap stage 0 suffisant).

## Action
Proceder Phase B avec scope pivote : design doc + LT-7 + fix GHA.
