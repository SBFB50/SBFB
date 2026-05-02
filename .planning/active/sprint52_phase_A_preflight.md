# Sprint 52 Phase A — preflight G8

Date : 2026-05-02 | HEAD : `e2ec4bb` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, no band-aid — dispatch fix est un vrai fix (oneshot signal), pas un commentaire. Aligne.
- Aucune zone-specifique applicable (dette pair, pas de kudos/crypto/deploy)
- Tensions plan vs memory : aucune

## Scans (all clean)
- S1a OSS prior art : phase micro-fix (oneshot shutdown signal) + DELETE docs + 1 ligne stale. Pattern standard tokio, APPROACH-ALIGNED par defaut. 0 projets OSS a consulter — clean
- S1b deps : 0 nouvelle dep, 0 lib bump. tokio::sync::oneshot deja dans le workspace — clean
- S2 historiques : 3 fichiers scannes (runtime.rs, dispatch_loop.rs, CLAUDE.md). git log grep : 1 hit S44 wrap-up (non pertinent). Archive scan : 1 hit S49 preflight runtime.rs (coordinator-in-daemon, non pertinent au shutdown signal). Memory feedback scan : 0 hit dispatch/shutdown — clean
- S3 threat model : fast-path verified. Phase ne cree aucun composant securite ni wire format. Pas de ligne S52 dans HARDENING_ROADMAP — clean
- S4 wire format : fast-path verified. Phase ne touche pas canonical.rs/schemas/*_VERSION. DELETE docs + micro-fix code interne — clean

## Telemetrie preflight
- Duree totale : ~1m
- S1a : N/A (micro-fix trivial) / 0 projets OSS / clean
- S1b : <15s / 0 libs / clean
- S2 : <30s / 3 fichiers + archive / clean
- S3 : fast-path / <10s
- S4 : fast-path / <10s

## Action
Proceder code phase A.
