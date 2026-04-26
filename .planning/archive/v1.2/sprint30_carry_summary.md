# Sprint 30 — Carry summary

**Date** : 2026-04-26
**Tip** : sera le commit Phase E

---

## P2 carry S29 — resolution

| ID | Description | Resolution S30 |
|---|---|---|
| P2-B-1-S28 | CI Linux/macOS writers (3/3 MANDATORY) | **DONE** rust-ci.yml matrice 3 OS couvre nexus-events-core (Phase B review P3 documente adaptation) |
| P2-C-1-S28 | blob-serve isolation gap (2/3) | **DONE** Phase B `a63562e` COOP/COEP headers (2/3→3/3) |
| P2-REVIEW-B-1 | consent.py mutation pattern | **DONE** Phase A `a731811` refactor pure function |
| P2-REVIEW-B-2 | §9.5 output filter not wired | **DOCUMENTED** Phase A `a731811` gap note THREAT_MODEL §9.5 → carry S31 wire (2/3) |
| P2-REVIEW-C-1 | task_runner.rs stub | **DOCUMENTED** Phase A `a731811` defense-in-depth comment → carry S31 impl (2/3) |
| P2-REVIEW-D-1 | executor trace log path relatif | **DONE** Phase A `a731811` commentaire intentionnalite asymetrie |

Score : 4/6 fermes, 2 documentes (carry S31).

---

## Nouveaux carry S31

| ID | Description | Reports | Source |
|---|---|---|---|
| P2-REVIEW-B-2 | §9.5 output filter not wired end-to-end | **2/3** | S29 B review → S30 A doc |
| P2-REVIEW-C-1 | task_runner.rs stub (impl reelle Ollama/llama.cpp IPC dispatch) | **2/3** | S29 C review → S30 A doc |
| P2-REVIEW-B-1-S30 | Playwright COEP iframe regression test dedie | 1/3 | S30 Phase B review |
| P2-REVIEW-D-1-S30 | VALIDATED_BLUEPRINT Couche 6 stale (Kirchenbauer→SynthID, spaCy→GLiNER) | 1/3 | S30 Phase D review |
| P3-REVIEW-D-1-S30 | SPLIT_INFERENCE_DESIGN confidence_score field | 1/3 | S30 Phase D review |
| P2-REVIEW-C-1-S30 | HTTP integration tests FROST endpoints (T0 admin) | 1/3 | S30 Phase C review |

Note : P2-REVIEW-B-2 et P2-REVIEW-C-1 atteignent 2/3 reports. Si non resolus
S31, ils passent 3/3 = **MANDATORY** S32 per §6.2.1 Regle 2.

---

## Items long-terme (ROADMAP_COMMITMENTS)

| ID | Description | Condition | Status |
|---|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | Radicle activation | tag v1.0 | Latent |
| LT-3 | Contribution family Sybil matrix | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | OS biometric gate | v1.0 + S30 FROST N1 + partnership | Latent (N1 code wiring S30 livre, mais partnership manquant) |
| LT-5 | Redundancy persistence SQLite | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh neighborhood enrichment | iroh > 0.97 OR v1.0 | **Trigger met** (iroh 0.98.0 2026-04-17) — bloque par Day 0 #3 pin, reste latent |

---

## ROADMAP_COMMITMENTS check (G7 Regle 3)

- LT-6 trigger partiellement rempli (iroh 0.98.0 publie) mais Day 0 #3
  (iroh 0.97 pinne, upgrade = sprint dedie) empeche l'activation. LT-6
  reste dans ROADMAP_COMMITMENTS avec note "condition met, awaits pin
  lift in dedicated upgrade sprint".
- Toutes les autres conditions de declenchement restent latentes.
- Aucun item LT ne redevient carry actif.
