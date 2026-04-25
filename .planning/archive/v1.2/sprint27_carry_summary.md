# Sprint 27 — Carry summary

**Ecrit** : 2026-04-25 (ouverture Sprint 27).
**Source** : `sprint26_verification.md §4+§5` + `sprint26_audit_findings.md §Findings`.

---

## 1. Cap G7 — 0/2 slots utilises

Les 7 P2 de l'audit S26 sont absorbes en Phase A (items d'audit, pas
carries formels au sens G7). Aucun carry formel S27 consomme.

---

## 2. P2 items audit S26 (absorbes Phase A)

| ID | Description | Source | Status |
|---|---|---|---|
| P2-A-1 | validate_stage_guard_map non wiree dans Dispatcher.__init__ | audit_findings S26 | Phase A S27 |
| P2-C-1 | emit_capability_event catch silencieux (pass au lieu de logger.debug) | audit_findings S26 | Phase A S27 |
| P2-D-1 | TaskHandlerDescriptor sans champ description (4-tuple vs 5-tuple) | audit_findings S26 | Phase A S27 |
| P2-C-2 | JsonFileWriter sans rotation taille-based | audit_findings S26 | Phase A S27 |
| P2-C-3 | EtwWriter naming trompeur (cross-platform tracing, pas ETW direct) | audit_findings S26 | Phase A S27 |
| P2-B-1 | MCP lifespan __aenter__/__aexit__ explicites fragile | audit_findings S26 | Phase A S27 |
| P2-E-1 | LOC estimates dans plan.md (informatif, convention integree) | audit_findings S26 | N/A (process) |

---

## 3. Hors cap — items long-terme

| ID | Description | Status |
|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | ROADMAP_COMMITMENTS, latent |
| LT-2 | Radicle activation | ROADMAP_COMMITMENTS, trigger tag v1.0 |
| LT-3 | Contribution family Sybil matrix | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-4 | OS biometric gate | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-5 | Redundancy persistence | ROADMAP_COMMITMENTS, latent |
| LT-6 | iroh neighborhood | ROADMAP_COMMITMENTS, latent |
