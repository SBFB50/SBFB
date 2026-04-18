# Sprint 21 — Carry-over summary (cap G7 = 2/2)

**Écrit** : 2026-04-18 (session fraîche post-audit gate S20 `66a3a7c`).
**Règle** : `docs/claude/README.md §6.2.1` — max 2 carry-overs par
sprint, re-confirmés ligne par ligne dans kickoff §6.

---

## 1. Carry-overs confirmés S20 → S21 (2/2)

### C-1 — Meta-1 Radicle-v1.0 activation tracking (re-carry S18 → S19 → S20 → S21)

- **ID** : Meta-1
- **Source originale** : Sprint 18 Phase E3 `95807b1` (pivot depuis
  Radicle vers Codeberg private mirror pre-launch) + décision carry
  Radicle-v1.0 pour le jour du tag `v1.0` go-public.
- **Rationale originale** : Radicle P2P public-only incompatible avec
  repo GitHub privé pre-launch ; différé v1.0 go-live. Runbook
  self-contained.
- **Sévérité** : P3 (pas un Gate-blocker fonctionnel, mais tracking
  release engineering nécessaire).
- **Owner** : FlowUP.
- **Deadline** : jour du tag `v1.0` go-public (pattern annual-ish
  tant que v1.0 pas tag — re-carry S22+ si v1.0 pas tag d'ici S22
  wrap-up).
- **Runbook self-contained** : `docs/release/MIRROR_FALLBACK.md
  §3.1-3.8` (8 sous-sections, 5 secrets GHA requis, action
  `gsaslis/mirror-to-radicle@514707f3` v0.2.0 pin SHA).
- **Statut S21** : confirmé re-carry, pas de touch wire. Grep
  `radicle` dans diff `3a7f0a3..131f32b` a retourné 0 changement
  S20. Audit gate S20 a confirmé re-carry explicite
  (`sprint20_audit_findings.md §5`).
- **Action S21** : aucune. Re-carry confirmé ligne `[x]` dans
  `sprint21_kickoff.md §6`.

### C-2 — C-PLAN-1 plan docs fix wire-point divergence

- **ID** : C-PLAN-1
- **Source originale** : Sprint 20 Phase C review, re-confirmée par
  audit gate S20 findings §3 Track C-PLAN-1.
- **Problème** : `sprint20_plan.md §6.2` et §6.4 (archivé
  `.planning/archive/v1.2/sprint20_plan.md`) citent comme wire-point
  PoW runtime le module `crates/nexus-shell-daemon-core/src/
  iroh_runtime.rs::GossipClient::subscribe()`. Vrai call-site =
  `crates/nexus-shell-daemon/src/runtime.rs::spawn_gossip_subscribe_
  task()`. `browse.rs::subscribe()` dans `-core` gère l'attention
  set, pas le transport gossip.
- **Rationale** : le code S20 Phase C est **correct** (PoW wire
  actif runtime sur tous les subscribes), mais la référence textuelle
  dans le plan archivé induit le lecteur en erreur. Les audits
  futurs ou contributors qui chercheraient le wire-point par le plan
  tomberaient sur le mauvais fichier.
- **Sévérité** : P2 (traçabilité docs, pas fonctionnel code).
- **Owner** : S21 executor (lead planning).
- **Deadline** : S21 Phase 0 (chore followup avant Phase A) OU
  intégré au Phase E tech debt batch S21.
- **Fix path** : 2 options équivalentes au choix de l'executeur :
  - **Option X** : `chore(sprint21): fix plan §6.2 wire-point
    divergence post-audit S20` — edit
    `.planning/archive/v1.2/sprint20_plan.md §6.2 + §6.4` avec
    note pointeur vers audit findings §3 Track C.
  - **Option Y** : note en tête du plan archivé (avant §1)
    indiquant la correction.
- **Action S21** : confirmé `[x]` dans kickoff §6. Résolution
  envisagée en Phase 0 S21 chore ou Phase E batch tech debt.

---

## 2. Hors cap G7 (tech debt PATTERNS.md — pas scope carry, invisible au cap)

Les 3 items suivants **ne comptent pas dans le cap carry** (règle
`README.md §6.2.1` — tech debt PATTERNS.md est séparé du scope
sprint) mais doivent être tracés pour que le planner S21 ne les
perde pas.

### Tech debt T-NN (S20 audit P2-E-1) — canary wire envelope → JCS

- **Location** : `crates/nexus-shell-daemon-core/src/canary/mod.rs`
  helper `canary_wire_bytes(canary: &Canary) -> Result<Vec<u8>,
  CanaryError>`.
- **Issue** : enveloppe canary broadcast utilise `serde_json::to_vec`.
  Signature couvre `canonical_bytes` JCS (RFC 8785) donc impact
  sécurité nul, mais ambiguïté cross-language pour subscribers Python
  qui re-serialize.
- **Fix path** : migrer vers `serde_jcs::to_vec`. Test de
  non-régression : snapshot cross-language Python ↔ Rust.
- **Owner** : S21 executor (intégration possible Phase E batch tech
  debt).
- **Rationale originale** : décision S18 E2 `04c9621` avait gardé
  `serde_json` pour simplicité, finding audit gate S20 promu P2 par
  rigor signal G4.

### Tech debt T-NN+1 (S20 audit P2-E-2) — CanaryRegistry verify Ed25519 at ingest

- **Location** : `packages/nexus-coordinator/src/nexus_coordinator/
  canary_registry.py` handler `POST /api/canary/observed`.
- **Issue** : registry observational-only, pas de verify Ed25519 at
  ingest. Attaquant local avec bearer token X-SBFB-Token pourrait
  injecter observations fake.
- **Mitigation actuelle** : bearer token loopback + CANARY.txt
  bootstrap pubkeys = trust root suffisant beta T0-T1.
- **Fix path** : verify Ed25519 at ingest via `nexus-core-py`
  `verify_canary` binding (primitive existe, juste à wirer). Test
  de non-régression : spoof `CanaryObservation` avec mauvaise
  signature → 401.
- **Owner** : S21 executor.
- **Décision maturité** : à trancher S21 Phase 0 — hardening avant
  v1.0 go-live (T2+) vs acceptable observational-only beta T0-T1.
- **Rationale originale** : `docs/security/WARRANT_CANARY_
  HARDENING.md §2 T-canary-registry-spoof` classifie explicitement
  le threat, mitigation trust-root documentée.

### Tech debt T-NN+2 (S21 D2) — iframe PII SDK Rust-wasm realignement Option G

- **Location** : future iframe SDK `web/src/sdk/pii/` (S21 Phase B
  livre JS + ONNX Runtime Web ; re-align Rust-wasm S22+).
- **Issue** : S21 Phase B iframe PII SDK livré en JS (custom wrapper
  ONNX Runtime Web + @huggingface/transformers tokenizer) au lieu
  d'un crate Rust→wasm qui serait l'alignement strict Option G
  stack.
- **Blocked currently by** (vérifié via research G2 2026-04-18) :
  - `tract` (Sonos Rust ONNX runtime) teste opset 9-18 ; GLiNER
    exporte typiquement opset 19 (DisentangledSelfAttention
    DeBERTa-v3 non documenté supporté tract).
  - `tract` `wasm32-unknown-unknown` (browser) non documenté
    officiellement ; seul `wasm32-wasi` (wasmtime runner) démontré
    dans examples Sonos.
  - Aucun precedent production OSS tract+GLiNER+wasm browser.
  - `gline-rs` v1.0.1 (2026-01, pure Rust GLiNER mainstream) a
    **choisi ort (wrapper ONNX Runtime Microsoft), PAS tract**.
    Signal fort que tract opset coverage insuffisant.
- **Re-evaluate S22+ if** :
  - tract publishes opset 19 test coverage, OR
  - ort (pyke) publishes `wasm32-unknown-unknown` backend
    publicly stable, OR
  - gline-rs ships wasm-bindgen target.
- **Goal** : port iframe SDK to Rust-wasm pour full Option G
  coverage (coord-side Presidio Python reste, iframe Rust-wasm au
  lieu de JS).
- **Owner** : S22+ executor quand blockers levés.

---

## 3. Scope reclassifié S20 → S21 (pas carry — scope direct S21)

Les 2 items suivants sont **scope natif S21** (HARDENING_ROADMAP
§3 S21), pas des carry-overs :

- **Rate-limit per-(consumer, worker, model)** : scope Phase A S21.
  Débloqué par PoW runtime wire S20 Phase C `16b94ba`
  (prerequisite satisfait).
- **Client-side redaction SDK** : scope Phase B S21. Requalifié du
  libellé roadmap S17 original « spaCy NER wasm ~500 LOC » vers la
  stack validée factuellement G2 (cf. `sprint21_design_review.md`
  §D2).

---

## 4. Décisions reclassifiées (pas carry — note mémo kickoff S21 §4 D5)

Les items suivants restent dans leurs fenêtres HARDENING_ROADMAP §3
original sans carry S21 :

- **Kudos-weighted gossip admission** : S22 (scope Sybil resistance).
- **Sandbox tool-calling allow-list strict** : S22.
- **Redundancy voting `Task.redundancy_factor`** : S22.
- **Ephemeral workers + VRAM wipe** : S23.
- **Honeypot Eclipse detection** : S23.
- **Re-run sampling + DNS fallback DHT** : S24.
- **Arti Tor bridge integration** : S25.
- **Domain fronting Snowflake-WebRTC** : S25.
- **PQC migration ML-DSA + ML-KEM** : S26+ (HNDL liability note
  HARDENING_ROADMAP audited_findings).
- **Hardware keystore TPM/SE/StrongBox** : S22+ (`trait KeyStore`
  abstraction livrée S20 Phase A prête).
- **HPKE envelope peer-restore** : S22+.
- **`actions/checkout@v4` pin SHA sweep** : sprint ops futur.

---

## 5. Cap G7 vérification

- **Carry confirmés** : 2 (C-1 Meta-1 Radicle + C-2 C-PLAN-1 plan
  docs fix).
- **Cap max** : 2 par sprint (`README.md §6.2.1`).
- **Statut** : **2/2 respecté**. ✓

Hors cap (tech debt PATTERNS.md) : 3 entrées documentées §2
ci-dessus (T-NN canary JCS + T-NN+1 registry verify + T-NN+2 Rust-
wasm realignement).
