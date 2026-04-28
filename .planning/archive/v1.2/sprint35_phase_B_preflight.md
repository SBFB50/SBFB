# Sprint 35 Phase B — preflight G8

Date : 2026-04-28 | HEAD : `09471c6` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest — D4 appel direct nexus-core-rs sans PyO3, conforme
- feedback_context7_systematic.md : 0 nouvelle lib externe — N/A

## Scans (all clean)
- S1a OSS prior art : dispatcher pattern (coordinator signs + dispatches tasks) = BOINC/Golem standard. Rust native call chain (canonical_bytes + sign_task + iroh doc insert) = APPROACH-ALIGNED pour un systeme Rust-native — clean
- S1b deps : 0 nouvelle dep (nexus-core-rs + nexus-coordinator-rs existants workspace) — clean
- S2 historiques : S21 Phase A pivot (`sprint21_phase_A_pivot_proposal.md`) notait factuellemnt que `/task/submit` n'existe pas en Rust daemon. Ce n'est pas un rejet — c'est une observation. Le pivot portait sur le rate-limiting (worker-core native au lieu de HTTP middleware), pas sur l'existence de l'endpoint. Aucune decision historique ne rejette la creation d'un endpoint task submission Rust dans le daemon — clean
- S3 threat model : fast-path verified. Phase B utilise le middleware auth bearer existant du daemon. Pas de nouveau composant securite — clean
- S4 wire format : fast-path verified. Task/TaskEntry types inchanges dans nexus-core-rs. `*_VERSION = 1` preserve. Day 0 preservees — clean

## Telemetrie preflight
- Duree totale : ~1m30s
- S1a : instant / BOINC+Golem reference / APPROACH-ALIGNED
- S1b : instant / 0 nouvelle dep / clean
- S2 : 30s / 2 scans (git log + archive grep) / clean
- S3 : fast-path / instant
- S4 : fast-path / instant

## Action
Proceder code phase B.
