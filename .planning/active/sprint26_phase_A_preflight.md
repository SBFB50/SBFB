# Sprint 26 Phase A — preflight G8

Date : 2026-04-24 | HEAD : `a97f7ca` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — Phase A = 5 audit fixes root-cause, pas pansements. Conforme.
- Routing table : aucune zone spécifique matchée (admin_check / capability_store / key_rotation / guardrails ne sont pas dans les zones kudos/deploy/crypto/lib externe).

## Scans (all clean)
- S1a OSS prior art : N/A — phase = P2 batch defensive fixes (NULL guard, permissions, stale rejection, determinism test, key validation). Patterns standard, pas de design à challenger. APPROACH-ALIGNED par nature.
- S1b deps : 0 nouvelle dep, 0 bump — clean
- S2 historiques : 7 fichiers scannés, 1 commit `f1e1f4d` (key_rotation.rs création S25 Phase B) — pas de rejet ni deviation sur la zone P2-REVOKE-1. Archive scan : 0 finding pertinent. Memory feedback : 0 contrainte violée — clean
- S3 threat model : fast-path verified. Phase ne crée pas de nouveau composant sécurité (fixes défensifs sur existants). HARDENING_ROADMAP §3 S26 aligné (P2 batch dans scope kickoff) — clean
- S4 wire format : fast-path verified. Aucun fichier canonical.rs / schemas/ / *_VERSION dans le périmètre. Day 0 préservées — clean

## Télémétrie preflight
- Durée totale : ~1m30s
- S1a : N/A (batch fixes, pas de design novel)
- S1b : <30s / 0 lib scannée (pas de nouvelle dep)
- S2 : ~30s / 7 fichiers, 3 commits scannés / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Action
Procéder code phase A.
