# Sprint 32 — Carry summary

**Sprint** : 32 (dette pair : iroh 0.98 upgrade + carries batch)
**Tip sortie** : Phase D commit (wrap-up)
**Date** : 2026-04-27

## Carries resolus S32

| ID | Description | Resolution |
|---|---|---|
| P2-REVIEW-C-1 (S31) | rusqlite 0.32→0.36 + arti-client dep activation | Phase B `a55a0ab` — **CLOSED** |
| P2-AUDIT-1 (S31) | Executor silent param drops max_tokens | Phase C `626221c` — **CLOSED** |
| P2-AUDIT-2 (S31) | HARDENING compteurs stale | Phase C `626221c` — **CLOSED** |
| P2-REVIEW-B-1-S30 | Playwright COEP iframe test (2/3→3/3) | Phase C `626221c` mock-only — **CLOSED** (real daemon E2E = P2-REVIEW-C-2 nouveau) |
| P3-AUDIT-1 (S31) | tor feature gate compile trap | Phase B `a55a0ab` — **CLOSED** |
| P3-AUDIT-2 (S31) | FROST HTTP error path tests | Phase C `626221c` 4 tests — **CLOSED** |
| P3-AUDIT-3 (S31) | Tor boot log misleading | Phase C `626221c` — **CLOSED** |
| LT-6 | iroh 0.98 upgrade (trigger met) | Phase A `90aff27` — **RESOLVED** |

**Score : 8/8 resolus (7 carries + 1 LT).**

## Carries ouverts → S33

| ID | Description | Reports | Prio | Sprint cible | Note |
|---|---|---|---|---|---|
| P2-REVIEW-A-1 | LOC plan meta-process | **3/3** | P2 | **MANDATORY S33** (§6.2.1 Regle 2) | Discipline plan-writing, pas d'action code |
| P2-A-1 | rand dual version (0.8+0.10) evaluation bump | 1/3 | P2 | S33 | iroh 0.98 tire rand 0.10 transitive, workspace utilise 0.8 direct |
| P2-B-1 | tor-rtcompat dep omise | 1/3 | P2 | S33+ | Ajout quand Phase 2 Tor stocke le handle client explicitement |
| P2-REVIEW-C-2 | Playwright COEP real daemon E2E test | 1/3 | P2 | S33 | Necessite daemon fixture infra ; mock-only livre S32 |
| P3-grammar | Executor grammar field wire | 1/3 | P3 | S33+ | Ollama ne supporte pas GBNF natif |
| P3-watermark | Executor watermark_config wire | 1/3 | P3 | S33+ | Defense-in-depth, SynthID inject worker-side |
| P3-iroh-comments | 7 fichiers .rs commentaires stale "iroh 0.97" | 1/3 | P3 | S33 | Batch nettoyage cosmétique |

**Note §6.2.1** : P2-REVIEW-A-1 atteint 3/3 reports → **MANDATORY S33**
(s'il n'est pas resolu S33, reclassification long-term impossible car
< 500 LOC action). Les 6 autres items sont a 1/3.

## Items long-terme (ROADMAP_COMMITMENTS)

| ID | Status post-S32 | Note |
|---|---|---|
| LT-1 | Latent | Conditions inchangees |
| LT-2 | Latent | Conditions inchangees |
| LT-3 | Latent | Conditions inchangees |
| LT-4 | Latent | Conditions inchangees |
| LT-5 | Latent | Conditions inchangees |
| LT-6 | **RESOLVED** S32 Phase A | iroh 0.98 deploye `90aff27` |
