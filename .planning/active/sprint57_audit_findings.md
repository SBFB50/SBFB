# Sprint 57 — Audit findings

**Date** : 2026-05-10
**Auditeur** : Claude (session fraiche, Cas A)
**Tip audite** : `f6b9570` (HEAD = memory tip, working tree clean)
**Verdict** : **PASS** (0 P0, 0 P1, 1 P2, 2 P3)

---

## Track A — Phase integrity

| Check | Resultat |
|---|---|
| 4 preflights G8 (A-D) | 4/4 EXECUTE |
| 4 reviews (A-D) | 4/4 PASS |
| Delta tests cumule (verification.md §2) | +5 Rust (1227→1232), +0 Vitest — coherent git log |
| Fix `a3943ed` sandbox forms | Documente verification.md row 27 |

**Verdict Track A** : PASS

---

## Track B — MANDATORY resolution

| Item | Evidence | Status |
|---|---|---|
| P2-S54-windows-test-cfg-unix | §P46 dans PATTERNS.md (l.2346+), commit `f1f26d5` | CLOSED |
| P2-S54-test-E2E-multi-noeuds | `test_cross_daemon_gossip_exchange` dans multi_daemon.rs:132, commit `f1f26d5` | CLOSED |
| P2-STORAGE-SQLITE | Table `app_storage` dans db.rs (M7), commit `636c87c` | CLOSED |

**Verdict Track B** : PASS — 3/3 MANDATORY resolus.

---

## Track C — Security hotfixes post-Phase C

3 commits fix(security) verifies :
1. `4780c5a` — harden blob-serve CSP + block direct-tab navigation
2. `8712890` — drop CSP sandbox (cassait le chargement sub-resources)
3. `0ee8cf4` — restore CSP sandbox + add CORP header

**CSP final** (blob_serve.rs:277) :
```
default-src 'self' 'unsafe-inline' 'unsafe-eval' data: blob:;
connect-src 'none'; worker-src 'none'; frame-src 'none';
object-src 'none'; base-uri 'none'; form-action 'none';
frame-ancestors *; sandbox allow-scripts
```
+ COOP `same-origin` + COEP `require-corp` + CORP `cross-origin`

Le `sandbox allow-scripts` est present dans le CSP (defense-in-depth
avec l'attribut iframe `sandbox="allow-scripts"`). `form-action 'none'`
bloque les submits meme si un `<form>` se glissait dans l'app.
`connect-src 'none'` empeche toute exfiltration reseau.

**Verdict Track C** : PASS

---

## Track D — Apps SBFB quality

### Protocol Explorer (`examples/sbfb-explorer/`)
- 4 fichiers : index.html, style.css, app.js, sbfb-bridge.js
- F2 liens source : present (app.js:8 "wire source links")
- F3 live status : `node_status`, `browse_list`, `identity_pubkey` (app.js:28)
- Taille : ~36 KB (cible < 500 KB)

### Ideas Hub (`examples/sbfb-ideas/`)
- 4 fichiers : index.html, style.css, app.js, sbfb-bridge.js
- Bridge CRUD : `listStorage`, `setStorage`, `deleteStorage` via SBFBBridge
- Identite : `identity_pubkey` pour vote per-identity
- Taille : ~26 KB (cible < 300 KB)

### sbfb-bridge.js SHA256
```
web/public/           ef55ce96...9ffa9f
examples/sbfb-explorer/ ef55ce96...9ffa9f
examples/sbfb-ideas/    ef55ce96...9ffa9f
```
3/3 identiques.

**Verdict Track D** : PASS

---

## Track E — Carries entrants S58

| Item | Compteur | Priorite | Note |
|---|---|---|---|
| P2-JITTER-SCOPE | 3/3 | **MANDATORY** | Phase A S58 |
| P2-INVITE-U16-WIRE | 3/3 | **MANDATORY** | Phase A S58 |
| P2-RETAIN-RECENT | 2/3 | carry | |
| P2-A-1 rand blocker | exemption | externe | upstream |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 | |
| AppStorage replication P2P | NEW pre-v1.0 | decision 2026-05-10 | 4 docs research |
| sbfb-bridge.js sync script | NEW P2 | carry S57→S58 | divergence risk |

**Verdict Track E** : PASS — carries correctement documentes.

---

## Track F — Research docs quality

4 documents dans `.planning/research/` :

| Document | Confiance | Projets OSS cites | Grounding |
|---|---|---|---|
| p2p_storage_replication_iroh_docs.md | MEDIUM-HIGH | iroh-docs 0.98 | Code source SBFB + API docs iroh |
| gpu_pooling_distributed_inference.md | MEDIUM-HIGH | Petals, Exo, Prima.cpp, Parallax, GPUStack | 5 projets nommes, performance numbers |
| vote_triggered_task_dispatch.md | MEDIUM | OpenZeppelin Governor | DAO governance pattern |
| community_code_validation_p2p.md | MEDIUM-HIGH | Radicle | P2P code review primitives |

Cross-references :
- GPU pooling ↔ coordinator dispatch : task decomposition aligne avec `dispatch.rs`
- P2P storage ↔ iroh-docs : coherent avec pin iroh 0.98 en workspace
- Vote dispatch ↔ gossip : utilise pub/sub existant
- Code validation ↔ curator lists : etend le modele Ed25519 trust

**Verdict Track F** : PASS

---

## Track G — Roadmap coherence

| Check | Resultat |
|---|---|
| CLAUDE.md S57 CLOSED + carries S58 | Present (l.116-132) |
| HARDENING_ROADMAP last_validated | `2026-05-10` / S57 |
| SPRINT_LOG row S57 | Present |
| Memory tip | `f6b9570` = HEAD |
| Scope cut §9 AppStorage | Kickoff §7 dit "post-v1.0", verification §3 + CLAUDE.md disent "PRE-V1.0 (decision 2026-05-10)" — **P3**, kickoff fige, decision correctement capturee dans docs vivants |

**Verdict Track G** : PASS

---

## Track H — S58 pair obligations

- S58 est pair → phase dette obligatoire (§6.2.1 Regle 1) : note
- 2 items 3/3 MANDATORY (JITTER-SCOPE + INVITE-U16-WIRE) → Phase A
- AppStorage replication P2P → phase(s) dediee(s) avec research grounding

**Verdict Track H** : PASS — obligations correctement identifiees dans audit plan.

---

## Findings

### P2 — sbfb-bridge.js manual copy divergence (carry S58)

**Constat** : Les 3 copies de sbfb-bridge.js (web/public/, sbfb-explorer/,
sbfb-ideas/) sont identiques (SHA256 ef55ce96) mais la synchronisation
est manuelle. Toute evolution du SDK en S58 risque de desynchroniser
les copies.

**Recommendation** : Script de sync ou build step S58 (deja documente
comme carry dans Sprint 57 Phase D review).

**Gravite** : P2 — pas de divergence actuelle, mais risque structurel.

### P3 — Kickoff §7 scope cut 9 desaligne avec decision utilisateur

**Constat** : Le kickoff fige scope cut §7 item 9 a "post-v1.0" pour
AppStorage replication, mais la decision utilisateur du 2026-05-10
le replanifie "pre-v1.0". Les docs vivants (CLAUDE.md, verification.md,
memory) capturent correctement la decision.

**Gravite** : P3 — kickoff fige par design, docs vivants corrects.

### P3 — Docker Linux flaky test pre-existant

**Constat** : `sigint_triggers_graceful_shutdown_and_removes_running_json`
echoue en Docker (timing signal). Pre-existant, non lie a S57.
Documente verification.md §5.

**Gravite** : P3 — pre-existant, non-regression S57.

---

## Verdict global

| Gravite | Count | Bloquant ? |
|---|---|---|
| P0 | 0 | — |
| P1 | 0 | — |
| P2 | 1 | Non |
| P3 | 2 | Non |

**PASS** — Sprint 57 correctement livre. 0 P0/P1. Le P2 (bridge
sync script) est deja documente comme carry S58. Les P3 sont
informatifs. G4 rigor signal satisfait (1 P2 + 2 P3 documentes).

Sprint 58 peut etre ouvert.
