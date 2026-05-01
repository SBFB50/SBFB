# Sprint 50 — Design Review Board (G1)

**Reviewer** : agent Explore independant.
**Date** : 2026-05-01.
**Decisions** : D1..D4 du sprint50_kickoff.md.

---

## Scoring

| Decision | Verdict | Rationale |
|---|---|---|
| D1 — Bulk delete 4 packages | ✅ | Zero runtime Rust→Python dep. nexus-shell-daemon importe nexus-coordinator-rs (Rust), pas le coordinator Python. Pas de spawn Python dans le daemon. PyO3 bindings confines a nexus-core-py sans consommateur externe. |
| D2 — Delete app-gov sans conversion | ✅ | app-gov est WIP pre-S12. 0 reference active dans web/src/ (grep "app-gov"/"gov"/"governance" = 0 match hors AppTabPage dead code). Pas de users. |
| D3 — Delete useAppEvents + AppTabPage | ✅ | useAppEvents importe uniquement par AppTabPage (1 ref L42). AppTabPage reference uniquement dans App.tsx router (1 ref L70) en lazy route `/app/:appName/tabs/:tabName`. Route legacy SDK S8, pas utilisee par le modele archive iframe. Suppression atomique safe. |
| D4 — Delete MCP server sans port | ⚠️ | mcp_server.py (176 LOC) standalone, 0 import, 0 consommateur. MAIS : code avec capability gate + loopback auth + tool whitelist (task_submit, storage_get, storage_set) qui suggerent un intent reel (Sprint 26 Phase B). Le port Rust est defer post-v1.0 — documenter la spec (whitelist + gate) pour reference future. |

## Rigor signal

3 ✅ + 1 ⚠️ = G4 satisfait (>=1 concern documente).

## Cross-cutting

- 0 wire format change (sprint soustractif)
- Post-S50 baseline ~1509 tests (1195 Rust + 267 Vitest + 42 PW + 5 size)
- 0 regression risk : fonctionnalite 100% portee Rust
