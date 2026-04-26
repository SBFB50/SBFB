# Sprint 28 — Carry summary

**Ecrit** : 2026-04-26 (cloture Sprint 28).
**Source** : `sprint28_verification.md §7` + 4 phase reviews (A/B/C/D).

---

## 1. P2 items phase reviews (carry S29)

| ID | Description | Reports | Source | Severite |
|---|---|---|---|---|
| P2-REVIEW-1 | generate_blocking 12 params → refactor GenerateConfig struct | 1/3 | Phase A review | P2 |
| P2-REVIEW-2 | Sampler chain per-step rebuild allocation churn hot path | 1/3 | Phase A review | P2 |
| P2-B-1 | JournaldWriter/OsLogWriter impls non testees fonctionnellement (CI Linux/macOS) | 1/3 | Phase B review | P2 |
| P2-B-2 | init_platform_emitter() sans test direct (trivial, 7 LOC) | 1/3 | Phase B review | P2 |
| P2-C-1 | blob-serve dans broker gap (isolation executor dedie S30+) | 1/3 | Phase C review | P2 |
| P2-C-2 | Cold-start benchmark RTX 5080 prereq S29 (non mesure) | 1/3 | Phase C review | P2 |
| P2-D-1 | HARDENING_ROADMAP §3 S29-S30 sans "Note realisme" | 1/3 | Phase D review | P2 |
| P2-D-2 | EXTERNAL_AUDIT_SCOPE versions crates sans note "at S28" | 1/3 | Phase D review | P2 |

---

## 2. Hors cap — items long-terme

| ID | Description | Status |
|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | ROADMAP_COMMITMENTS, latent |
| LT-2 | Radicle activation | ROADMAP_COMMITMENTS, trigger tag v1.0 |
| LT-3 | Contribution family Sybil matrix | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-4 | OS biometric gate | ROADMAP_COMMITMENTS, post-v1.0 |
| LT-5 | Redundancy persistence | ROADMAP_COMMITMENTS, latent |
| LT-6 | iroh neighborhood | ROADMAP_COMMITMENTS, latent |

---

## 3. Check ROADMAP_COMMITMENTS (G7 Regle 3)

Resultat : toutes les conditions de declenchement sont latentes
(tag v1.0 non pose, iroh toujours 0.97, Gini non mesurable pre-prod,
pas de multi-worker deploy, pas de partnership ONG formelle). Aucun
item LT ne redevient carry actif.

---

## 4. Items resolus ce sprint

| ID | Description | Phase |
|---|---|---|
| P2-B-1 (S27) | Watermark injection non cablee sampling | A |
| P2-B-2 (S27) | watermark.toml.sample absent | A |
| P2-C-1 (S27) | Fingerprint seeds.toml dummy | A |
| P2-D-1 (S27) | P37 chemin injector incorrect | A |
| SC-9 | Platform writers journald/oslog (2/3 reports) | B |
| SC-10 | ONNX CI fixture (5+/3 reports, escalade) | B |
