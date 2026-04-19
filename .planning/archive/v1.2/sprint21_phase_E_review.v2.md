# Sprint 21 Phase E — nexus-phase-auditor review

HEAD pre-commit: f830579 (Phase D tip)
Phase E commit: 49f0d32
Draft commit body: feat(sprint21): Phase E - tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix + PATTERNS P34)
Auditeur: nexus-phase-auditor claude-sonnet-4-6 (session independante 2026-04-19)
Timebox: 40m

## Verdict : PASS

0 P0, 0 P1. 2 P2 pre-documentes et confirmes independamment par relecture fichiers.
3 P3 cosmetiques. Toutes les dimensions explorees avec evidence inline citee.

Note G4 pre-audit : un fichier sprint21_phase_E_review.md pre-ecrit par l executeur
etait present dans le commit Phase E 49f0d32 (classe CRAFT dans le body commit).
Ceci constitue une deviation du principe G4 d independance de l audit gate.
Le present fichier REMPLACE ce pre-ecrit et constitue le verdict officiel.
Classification P3 car le contenu factuellement concordait avec les observations independantes.
Ne pas reproduire S22.

---

## Dimensions

### Security

- [x] forbid(unsafe_code) confirme lib.rs:28 -- aucun bloc unsafe dans le diff
- [x] Aucun unwrap hors test : canary/mod.rs:258 utilise .map_err(), lib.rs:1125-1163 utilise PyResult
- [x] verify_canary cryptographiquement correcte : 3 verifications (version v==1 only, pubkey/sig decode fixed-size, canonical_bytes recomputes via DOMAIN_WARRANT_CANARY_V1) -- lu canary/mod.rs:226-237
- [x] Domaine de signature distinct (DOMAIN_WARRANT_CANARY_V1) -- pas de cross-type replay possible -- lu canary/mod.rs:46+234
- [x] Injection JSON impossible -- json.dumps(dict) propre, lu canary.py:113
- [x] Aucun secret dans le diff (0 occurrence AKIA/ghp_/pat_/sbfb_)
- [x] Aucune nouvelle route loopback sans PeerCredsVerified (POST /api/canary/observed pre-existait Sprint 20)

Signing path : lu canary/mod.rs:234 -- bytes signes = canonical_bytes(canary.signed, DOMAIN_WARRANT_CANARY_V1).
Transport path : lu canary/mod.rs:258 -- serde_jcs::to_vec(canary). Chemins distincts confirmes.
PyO3 binding : lu lib.rs:1159-1163 -- serde_json::from_str parse, rs_verify_canary deleguee. Rust SoT.

0 finding P0/P1 securite.

### Patterns

- [x] Pattern PyO3 binding respecte -- compare verify_task_entry lib.rs:872-878 vs verify_canary lib.rs:1159-1163 : meme structure str JSON / from_str / delegation Rust / py_err / PyResult
- [x] Pattern path-dep interne respecte -- nexus-shell-daemon-core path dep miroir nexus-worker-core ligne 35 meme fichier
- [x] Pattern JCS canonical respecte pour wire transport -- serde_jcs::to_vec a canary/mod.rs:258
- [x] Enregistrement pymodule complet -- build_canary et verify_canary a lib.rs:1220-1221
- [ ] P3 : build_canary lib.rs:1134 utilise serde_json::to_string au lieu de serde_jcs::to_vec (incoherence visuelle, 0 regression)

### Working tree audit (G5)

- [x] PHASE : 9 fichiers (canary/mod.rs + Cargo.toml x2 + lib.rs + canary.py + test_api_canary.py + sprint20_plan.md + PATTERNS.md + Cargo.lock)
- [x] CRAFT : 2 fichiers planning declares dans body commit
- [x] DEBT : 0 fichier scope cut non autorise
- [x] NOISE : 0
- [x] Section Working tree audit presente dans body commit

### G8 traceability

- [x] Artefact G8 present : archive/v1.2/sprint21_phase_E_preflight.md (lu lignes 1-24)
- [x] Verdict preflight : SCOPE-CUT-CONSISTENT
- [x] HEAD preflight = f830579 (Phase D tip -- pre-implementation confirme)
- [x] 3 findings non-bloquants absorbes inline dans preflight
- [x] Findings non-bloquants portes S22 dans PATTERNS.md §P34 T-NN+2
- [x] Aucun pivot_proposal (pas de DESIGN-CONFLICT)

### Scope-cuts

- [x] 0 scope leak : governor / onnxruntime / GLiNER / presidio / quarantine / SQLite / WAL / LlmBackend / llguidance / Radicle -- 0 occurrence dans les 9 fichiers PHASE
- [x] Cap G7 2/2 respecte (T-NN + T-NN+1 clos, Meta-1 Radicle re-carry S22, T-NN+2 S22+ hors-cap PATTERNS §P34)

### Tests-delta

- [x] Rust +1 : wire_bytes_is_jcs_canonical_cross_language -- lu canary/mod.rs:491-517, 2 assertions (direct JCS + round-trip Value)
- [x] Python coord +20 : 5 fonctions test dans test_api_canary.py:55-163 (1 pre-existante + 4 nouvelles/splits E-2) + 16 wheel-stale reactives
- [x] SDK / Vitest / Playwright : 0 (non touche Phase E)
- [x] 0 skip sans reason= dans le diff

### Research-grounding

- [x] serde_jcs : workspace pre-existante Sprint 4 Day 0 1c1fcfb, trace preflight S1
- [x] nexus-shell-daemon-core : path dep interne, trace preflight S2-E2
- [x] time : workspace pre-existante Sprint 5
- [x] Cargo.lock : 0 nouvelle crate externe
- [x] Ed25519 RFC 8032 : trace Sprint 18 Phase E2 -- binding delègue a nexus_core_rs::crypto::verify pre-existant
- [x] 0 P0/P1 research-grounding

### Horizon long-terme + documentation amont

- [x] Design doc non requis (tech debt batch pur, pas de nouveau module structurant)
- [x] Alternatives rejetees documentees dans PATTERNS.md §P34 T-NN+2 (tract opset 19, ort wasm32, gline-rs wasm-bindgen)
- [x] Binding Rust SoT = choix techniquement superieur
- [x] 0 estimation LOC dans plan ou kickoff
- [x] CANARY_VERSION = 1 inchange (lu canary/mod.rs:70), decoder v == 1 only (lu canary/mod.rs:227) -- pre-launch protocol respecte

---

## Findings

- **P3** : build_canary PyO3 (lib.rs:1134) retourne serde_json::to_string(&canary) au lieu de serde_jcs::to_vec. Incoherence visuelle par rapport a E-1. 0 regression correctness. A aligner S22.

- **P3** : Pre-ecriture sprint21_phase_E_review.md par l executeur dans commit Phase E 49f0d32 (classe CRAFT body commit). Deviation principe G4 independance audit gate. Contenu correct apres verification independante. Ne pas reproduire S22.

- **P3** : PATTERNS.md §P34 entrees T-NN / T-NN+1 mentionnent Sprint 21 Phase E sans citer le SHA 49f0d32 explicitement. Tracabilite presente dans body commit. Chore S22 peut ajouter SHA inline.

- **P2 (pre-documente Sprint 20, confirme)** : P2-E-DURESS-ACK -- kind=duress_ack dans POST /api/canary/observed reste observational-only sans verify Ed25519 (canary.py:121-128). Injection faux duress_ack possible dans registry locale. Scope-cut delibere plan §8.1 E-2. Carry S22+.

- **P2 (pre-documente Sprint 20, confirme)** : P2-E-WIRE-PRE-LAUNCH-FIX -- 16 tests coord wheel-stale reactives dans delta +20. Cause racine : bootstrap §7 ne garantit pas maturin develop --release fresh. Carry S22 : documenter prerequis dans bootstrap §7.

---

## Recommendation

Commit 49f0d32 autorise. Verdict PASS confirme independamment.
0 P0. 0 P1. 2 P2 acceptes consciemment. 3 P3 cosmetiques.

Phase E livre : T-NN clos (canary/mod.rs:258) + T-NN+1 clos (canary.py:113-118 + lib.rs:1159-1163) + C-PLAN-1 resolu (sprint20_plan.md tete §6) + PATTERNS §P34 formalise. Invariants wire format preserves. Canary forgee rejetee HTTP 401 avant registry.

Carries S22 obligatoires :
1. P2-E-DURESS-ACK : entree sprint22_audit_findings.md
2. P2-E-WIRE-PRE-LAUNCH-FIX : entree sprint22_audit_findings.md + action bootstrap §7
3. P3 : aligner build_canary lib.rs:1134 vers serde_jcs::to_vec (chore cosmetique)
4. Interdire pre-ecriture review.md par executeur (regle G4 a rappeler S22 kickoff)
