# Sprint 46 — Design Review Board (G1)

**Reviewer** : agent Explore independant (session fraiche).
**Date** : 2026-04-30.
**Documents source** : sprint46_kickoff.md D1..D4 draft.

---

## Scoring

| Decision | Source | Alternative | Risk | Score |
|---|---|---|---|---|
| D1 Router oneshot harness | ✅ axum 0.8.8 (Jan 2026) + tower pattern | ✅ unit-only rejete avec rationale | ✅ .with_state() verifie | **✅** |
| D2 MANDATORY 12 + ext 14 | ✅ audit factuel 54 routes | ✅ 33/12-only analyses | ✅ scope realiste | **✅** |
| D3 Frontend direct-daemon | ✅ hotfix 1f1a017 live | ⚠️ dual-mode fallback note | ⚠️ migration orphans non traces | **⚠️** |
| D4 Debt batch 5 items S44 | ✅ carries documentes S44 audit | ✅ cherry-pick rejete | ⚠️ scope boundary 2/3 flou | **⚠️** |

**Crypto/spec checklist** : N/A (pas de choix crypto/spec).
**Rust-first checklist** : PASS (axum 0.8 pinne, pas de conflit).
**Sprint realism** : 4 phases realiste si D3 route inventory +
D4 scope clarifie pre-phase.

---

## Shadows

### Shadow-1 (D3 ⚠️ Medium) — Migration tracker absent

Le plan §C.2 liste 16 fichiers touches mais aucun inventaire
croisant les 20 appels coordinator.ts avec les routes daemon
`/api/v1/*`. Risque : composants comme ProjectDetail.tsx appelant
`listApps(coordUrl)` repointes vers daemon sans validation que
la route existe.

**Mitigation recommandee** : produire une table route-migration
(coordinator path → daemon /api/v1/* path) comme tache pre-Phase C
dans le preflight G8. Le scan S1a/S1b du preflight captera les
divergences de paths.

### Shadow-2 (D4 ⚠️ Low) — Scope boundary "2/3" ambigu

Le "2/3" dans les carries est un **compteur de reports** (nombre
de sprints consecutifs ou l'item est reporte), pas un pourcentage
de completion. Le plan doit clarifier les criteres d'acceptation
par item. De plus P3-AUDIT-B-1-S44 (diagnostic silent fallback)
etait PASS dans l'audit S44 — le re-work est justifie car le
carry designe le comportement silencieux (unwrap_or_default) qui
persiste meme si l'audit n'a pas bloque.

**Mitigation recommandee** : ajouter dans le plan Phase B §B.4
un critere d'acceptation par item dette (pas juste un compteur
global).

---

## Recommendation

✅ D1, D2 : EXECUTE immediatement.
⚠️ D3 : EXECUTE avec tache pre-phase (route inventory G8).
⚠️ D4 : EXECUTE avec clarification scope (criteres par item).

**Aucun showstopper.** Tous les choix sont techniquement corrects ;
les risques sont tracking + clarte scope, pas architecture.
