# Sprint 29 — Carry summary

**Date** : 2026-04-26
**Tip** : sera le commit Phase E

---

## P2 carry S28 — resolution

| ID | Description | Resolution S29 |
|---|---|---|
| P2-REVIEW-1 | generate_blocking 12 params commentaire | **DONE** Phase A `b1c4148` |
| P2-REVIEW-2 | Sampler chain load assumption commentaire | **DONE** Phase A `b1c4148` |
| P2-B-1 | CI Linux/macOS writers | **SCOPE-CUT** S30 (2/3 → 3/3 = **MANDATORY** S30) |
| P2-B-2 | init_platform_emitter test direct | **DONE** Phase A `b1c4148` |
| P2-C-1 | blob-serve isolation gap | **DOCUMENTED** Phase C (broker garde blob-serve, 2/3) |
| P2-C-2 | Cold-start benchmark RTX 5080 | **DONE** Phase A `b1c4148` |
| P2-D-1 | Note realisme S29-S30 HARDENING_ROADMAP | **DONE** Phase A `b1c4148` |
| P2-D-2 | Version note at RFP time | **DONE** Phase A `b1c4148` |

Score : 5/8 fermes, 1 scope-cut (mandatory S30), 1 documente (carry S30), 1 scope-cut carry.

---

## Nouveaux carry S30

| ID | Description | Reports | Source |
|---|---|---|---|
| P2-B-1-S28 | CI Linux/macOS writers | **3/3 MANDATORY** | S28 Phase B → S29 scope-cut |
| P2-C-1-S28 | blob-serve isolation gap | 2/3 | S28 Phase C → S29 documented |
| P2-REVIEW-B-1 | consent.py mutation pattern | 1/3 | S29 Phase B review |
| P2-REVIEW-B-2 | §9.5 output filter not wired | 1/3 | S29 Phase B review |
| P2-REVIEW-C-1 | task_runner.rs stub | 1/3 | S29 Phase C review |
| P2-REVIEW-D-1 | executor trace log path relatif | 1/3 | S29 Phase D review |

---

## Items long-terme (inchanges)

| ID | Description | Condition | Status |
|---|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | Radicle activation | tag v1.0 | Latent |
| LT-3 | Contribution family Sybil matrix | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | OS biometric gate | v1.0 + S30 FROST N1 + partnership | Latent |
| LT-5 | Redundancy persistence SQLite | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh neighborhood enrichment | iroh > 0.97 OR v1.0 | Latent |

---

## ROADMAP_COMMITMENTS check (G7 Regle 3)

Toutes les conditions de declenchement restent latentes. Aucun
item LT ne redevient carry actif.
