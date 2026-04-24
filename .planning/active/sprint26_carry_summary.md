# Sprint 26 — Carry summary

**Ecrit** : 2026-04-22 (ouverture Sprint 26).
**Source** : `sprint25_verification.md §4+§5` + `sprint25_audit_findings.md §Findings`.

---

## 1. Cap G7 — 0/2 slots utilises

Pas de carry formel S26. Les 5 P2 de l'audit S25 sont absorbes en
Phase A (items d'audit, pas carries formels). Les 2 re-carry
historiques (P2-D-1, P2-E-1-iroh) sont reclassifies long-term
commitments (4e sprint de carry, regle §6.2.1).

---

## 2. P2 items audit S25 (absorbes Phase A)

| ID | Description | Source | Status |
|---|---|---|---|
| P2-ADMIN-1 | Windows MIL null ptr guard admin_check.py:62-64 | audit_findings S25 C-6 | Phase A S26 |
| P2-CAPS-1 | Permissions restrictives ~/.sbfb/ | audit_findings S25 C-2 | Phase A S26 |
| P2-REVOKE-1 | RevocationCache overwrite log + reject stale | audit_findings S25 A-5 | Phase A S26 |
| P2-HASH-1 | tomli_w determinism round-trip test | audit_findings S25 C-3 | Phase A S26 |
| P2-STAGE-1 | StageGuardrailMap key validation | Phase C review P2-C-1, audit_findings S25 B-1 | Phase A S26 |

---

## 3. Reclassifications long-term (G7 §6.2.1 auto-trigger)

| ID | Description | Carry history | Reclassification |
|---|---|---|---|
| P2-D-1 | Redundancy persistence in-memory → SQLite + wire-up | S23 cree → S24 carry → S25 carry → S26 = 4 sprints | LT-5 ROADMAP_COMMITMENTS.md |
| P2-E-1-iroh | iroh neighborhood enrichment | S23 cree → S24 carry → S25 carry → S26 = 4 sprints | LT-6 ROADMAP_COMMITMENTS.md |

**Justification** : regle §6.2.1 "apres 3 sprints consecutifs en carry
sans livraison, reclassification automatique long-term commitment." Les 2
items sont pre-v1.0 non-bloquants (in-memory suffisant, neighborhood
optionnel). Conditions de declenchement definies dans ROADMAP_COMMITMENTS.

---

## 4. Hors cap — items long-terme

| ID | Description | Status |
|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | ROADMAP_COMMITMENTS, latent |
| LT-2 | Radicle activation | ROADMAP_COMMITMENTS, trigger tag v1.0 |
| LT-3 | Contribution family Sybil matrix | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-4 | OS biometric gate | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-5 | Redundancy persistence | ROADMAP_COMMITMENTS, NEW S26 |
| LT-6 | iroh neighborhood | ROADMAP_COMMITMENTS, NEW S26 |
