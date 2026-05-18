# Sprint 65 Phase C — preflight G8

Date : 2026-05-18 | HEAD : `28b3a43` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code — Phase C = UI enhancement standard, pas de deep alternative requise
- feedback_context7_systematic.md : context7 avant code touchant lib/API — N/A, pas de nouvelle lib/API (React state management + endpoint existant)

## Scans (all clean)
- S1a OSS prior art : badge dynamique verification = pattern standard (GitHub verified commits, npm package badges, Docker verified images). Endpoint `GET /api/v1/project/{id}/provenance` existe deja (VerificationDetail l'utilise on-demand). Appel direct `authFetch` depuis le shell, pas via bridge (BrowsedProject = shell host, pas iframe app). APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep, React Query + lucide-react + authFetch deja presents — clean
- S2 historiques : 2 fichiers scannes (BrowsedProject.tsx, scripts/scan-trust-wording.sh), 0 DEVIATION/rejected pertinent — clean
- S3 threat model : fast-path verified. Phase C ajoute un appel UI a un endpoint existant (pas nouveau vecteur d'attaque). HARDENING_ROADMAP N/A — clean
- S4 wire format : fast-path verified. 0 canonical.rs, 0 *_VERSION touche, 0 Day 0 rebattue — clean

## Note implementation

Le plan dit "appel provenance_verify via le bridge ou l'API daemon".
Clarification : BrowsedProject est le shell (pas une iframe app),
donc appel direct via `authFetch` au endpoint
`/api/v1/project/{id}/provenance` (comme VerificationDetail.tsx
`doFetch` l.42-69). Le bridge `provenance_verify` est reserve aux
apps iframe qui communiquent via postMessage.

## Telemetrie preflight
- Duree totale : ~3m
- S1a : ~1m / 3 patterns OSS consultes (GitHub verified, npm, Docker) / finding : clean (APPROACH-ALIGNED)
- S1b : ~30s / 0 libs scannees (pas de nouvelle dep) / finding : clean
- S2 : ~30s / 2 fichiers, 0 commits pertinents / finding : clean
- S3 : fast-path / ~20s
- S4 : fast-path / ~20s

## Action
Proceder code phase C.
