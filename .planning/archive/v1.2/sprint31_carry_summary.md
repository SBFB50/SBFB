# Sprint 31 — Carry summary

**Date** : 2026-04-27
**Tip** : sera le commit Phase E (post-migration)

---

## P2/P3 carry S30 — resolution

| ID | Description | Resolution S31 |
|---|---|---|
| P2-REVIEW-C-1 | task_runner stub (carry 2/3) | **DONE** Phase A `e85623a` LlmBackend Ollama wire reel — `task_runner.rs` rewrite + CLI `--ollama-endpoint` + 3 tests (stub, ollama mock, error path) |
| P2-REVIEW-B-2 | §9.5 output filter not wired (carry 2/3) | **DONE** Phase B `0771dc8` OutputSafetyGuardrail post-verify, results invalides marques `rejected` + 0 kudos credit, 5 tests E2E |
| P3-AUDIT-1 | WebAppFrame.tsx orphelin (carry 1/3) | **DONE** Phase B `0771dc8` delete `WebAppFrame.tsx` + `WebAppFrame.test.tsx` |
| P2-REVIEW-D-1-S30 | VALIDATED_BLUEPRINT Couche 6 stale (1/3) | **DONE** Phase D `ab09b5d` Kirchenbauer→SynthID + spaCy→GLiNER refresh |
| P3-REVIEW-D-1-S30 | SPLIT_INFERENCE confidence_score (1/3) | **DONE** Phase D `ab09b5d` §4.1 ajoute champ `confidence_score: f64 (0.0–1.0)` |
| P2-REVIEW-C-1-S30 | HTTP integration tests FROST endpoints (1/3) | **DONE** Phase D `ab09b5d` 4 tests `frost_http_*` (trusted-dealer, round1, round2, aggregate) |

Score : **6/6 carries S30 fermes** (vs S30 score 4/6).

---

## Nouveaux carry S32

| ID | Description | Reports | Source |
|---|---|---|---|
| P2-REVIEW-B-1-S30 | Playwright COEP iframe regression test dedie | **2/3** | S30 Phase B review → S31 differé (env Playwright instable, 2 PW env fail recurrentes) |
| P2-REVIEW-C-1 | rusqlite 0.32→0.36 workspace upgrade + arti-client dep activation | 1/3 | S31 Phase C review (libsqlite3-sys conflict : rusqlite 0.32 workspace vs arti-client 0.41 → tor-dirmgr → rusqlite >= 0.36). Phase C livre infra config + feature gate + fallback + coordinator wire, mais la dep reelle et le bootstrap E2E sont differes |
| P2-REVIEW-A-1 | LOC estimees prospectives dans plan §5.5 (Track meta-process) | 1/3 | S31 Phase A review — issue plan-level (estimations LOC contraires a §6.7), pas d'impact code, observation pour plans futurs |

Note : P2-REVIEW-B-1-S30 atteint 2/3 reports apres differement S31. Si non
resolu S32, il passera 3/3 = **MANDATORY** S33 per §6.2.1 Regle 2.

P2-REVIEW-C-1 (rusqlite + arti dep activation) est strategique pour le
deploy reel de Tor : Phase C S31 a livre l'infrastructure (config TOML +
feature gate + transport Rust + coordinator wire Python) sans la dep arti
elle-meme, qui necessite l'upgrade workspace de rusqlite. Resolution S32
depend de la disponibilite d'une fenetre upgrade dedie (couplee
potentiellement a iroh 0.98 si sprint pair S32 absorbe les deux).

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
| LT-6 | iroh neighborhood enrichment | iroh > 0.97 OR v1.0 | **Trigger met** (iroh 0.98.0 2026-04-17) — bloque par Day 0 #3 pin, **scheduled S32 upgrade dedie** (cf. D5 S31 kickoff) |

---

## ROADMAP_COMMITMENTS check (G7 Regle 3)

- LT-6 trigger remplit (iroh 0.98.0 publie 2026-04-17). Scope-cut S31
  D5 documente l'upgrade comme item dedie S32. La condition de
  declenchement reste met, mais l'execution est calendrisee, pas
  spontanee.
- LT-2 Radicle reste conditionne sur tag v1.0, hors-cycle S31.
- Toutes les autres conditions de declenchement restent latentes.
- Aucun item LT ne redevient carry actif (hors LT-6 scheduled S32).

---

## Compteurs reports apres S31 (entree S32)

| ID | Reports a l'entree S32 |
|---|---|
| P2-REVIEW-B-1-S30 (Playwright COEP) | 2/3 |
| P2-REVIEW-C-1 (rusqlite + arti dep) | 1/3 |
| P2-REVIEW-A-1 (LOC plan meta) | 1/3 |
| LT-6 (iroh 0.98) | scheduled S32 (condition met, pas un carry P2 standard) |

3 carries P2 actifs entrant S32, dont 1 a 2/3 (Playwright COEP) qui
deviendra MANDATORY S33 si non resolu S32.
