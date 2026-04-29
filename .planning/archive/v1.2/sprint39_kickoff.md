# Sprint 39 — Kickoff (PiiRedactor Rust Tier 1 part 2 + CanaryRegistry Rust Tier 2 debut)

**Ecrit** : 2026-04-29 (session fraiche post-audit gate S38 `294e276`).
**Type** : **sprint impair** — pas de phase dette obligatoire
(§6.2.1 Regle 1 : S38 pair, S39 impair).
**Tip master d'entree** : `294e276` (chore(planning) audit findings
S38 PASS).
**Phase 0 audit Sprint 38** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint38_audit_findings.md` (verdict **PASS**,
0 P0/P1, 1 P2 [off-by-one substring output_filter.rs fixe `4df0928`],
1 P3 [lowercase divergence carry]).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-29) : HARDENING_ROADMAP last_validated
  `2026-04-29` (S38 CLOSED). 0 trigger actif (meme jour que S38
  close, aucun dep n'a change).

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - frost-ed25519 > 3.0 : stable 3.0 inchange — NOT FIRED
  - Tous les autres (wasmtime, Tor PoW, NIST PQC, etc.) : NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S39** :
  - `regex` (crates.io) : crate standard Rust pour expressions
    regulieres. Deja dans le Cargo.lock (dep transitive). A ajouter
    comme workspace dep directe pour nexus-coordinator-rs.
  - `time` : deja dep directe de nexus-coordinator-rs. Utilise pour
    date computations canary freshness (OffsetDateTime, Date).
  - `serde` / `serde_json` : deja dans le workspace. Utilise pour
    CanaryRegistry persistence JSON.

- **Roadmap reference** : `.planning/roadmap_v1_migration_rust.md`
  §S39 — "PiiRedactor Rust (Tier 1 part 2) + canary registry
  (Tier 2 debut)".

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem, LT-4
  biometric, LT-5 redundancy : tous requierent tag v1.0 ou
  condition externe → aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 38 CLOSED. 3 phases A-C livrees + Phase D wrap-up :
- Phase A : MANDATORY validator_loop tokio 3/3 ferme + dette pair
  P2 batch (rowid doc + verify_chain endpoint + launcher log_dir test)
- Phase B : OutputFilter Rust migration (invisible text scanner +
  prompt echo cascade + EED strsim)
- Phase C : Guardrails pipeline Rust (Guardrail trait + GuardrailChain
  + OutputSafetyGuardrail + wire submit_result)

Audit gate S38 : **PASS** (0 P0/P1, 1 P2 [off-by-one substring fixe
`4df0928`], 1 P3 [lowercase divergence carry]).

Roadmap migration Python→Rust : Tier 1 part 1 livre (OutputFilter +
Guardrails). Prochaine etape = Tier 1 part 2 (PiiRedactor) + Tier 2
debut (CanaryRegistry).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-29 (S38 CLOSED). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `294e276`)

| Suite | Count |
|---|---|
| Rust nextest | 968 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~1971** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **migre PiiRedactor de Python vers Rust** (regex-only,
sans dep ML/ONNX pre-v1.0) et **migre CanaryRegistry de Python vers
Rust** (observations + freshness + network health + persistence JSON),
avec **wire integration** dans le GuardrailChain (PiiInputGuardrail
input direction) et les routes HTTP daemon (canary endpoints Rust).
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 38

**DONE** — `294e276`. Verdict PASS (0 P0/P1, 1 P2 fixe + 1 P3).
Cf. `.planning/archive/v1.2/sprint38_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — PiiRedactor Rust : strategie regex-only

**Retenu** : porter `pii_redactor.py` (483 LOC) vers un module
`pii_redactor.rs` dans nexus-coordinator-rs. Strategie :

(a) **Regex-only** — pas de dep ML (ONNX/Presidio/ort).
    Patterns : email, telephone (international E.164 + formats US),
    SSN (US `XXX-XX-XXXX`), carte de credit (Luhn validation),
    adresses IP (v4 + v6).
(b) **RedactionPolicy** : struct deserialise depuis TOML.
    Hot-reload pattern identique a OutputFilter/pow_policy_loader.
    Champs : `enabled_patterns` (liste blanche), `replacement`
    (string de remplacement, default `[REDACTED]`).
(c) **PiiRedactor** struct : `redact(&self, text: &str) -> String`,
    `has_pii(&self, text: &str) -> bool`.
    Interne : compile les regex une fois (lazy_static ou OnceLock),
    scan + rewrite en un pass.
(d) **PiiInputGuardrail** adapter : impl Guardrail trait (S38),
    direction `Input`. `check()` appelle `has_pii()` → si true →
    `Tripwire`, sinon `Pass`.
(e) Luhn validation pour credit card (port direct du Python
    `_luhn_valid()`).

**Rejete** :
- ONNX Runtime Rust (`ort` crate 2.0) : ~50MB runtime dep, opset
  19 coverage incomplete pour DeBERTa-v3 GLiNER, wasm32-browser
  non supporte. Pre-v1.0 les regex suffisent (email/phone/SSN/CC
  couvrent 90%+ des cas PII structuree).
- Presidio (Python-only, objectif = suppression coordinator Python).
- regex + ML hybride (complexite inutile pre-v1.0 — ajouter ML
  post-v1.0 quand ort est stable).
- Port des spans Presidio (dep Python, pas portable Rust).

**Implications code** :
- `crates/nexus-coordinator-rs/Cargo.toml` (+regex dep)
- `Cargo.toml` racine (+regex workspace dep)
- `crates/nexus-coordinator-rs/src/pii_redactor.rs` (NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (pub mod)
- Tests : 8-10 tests (chaque pattern + Luhn + policy + guardrail)

### D2 — CanaryRegistry Rust : port direct avec persistence JSON

**Retenu** : porter `canary_registry.py` (366 LOC) vers un module
`canary_registry.rs` dans nexus-coordinator-rs. Design :

(a) **Types serde** : CanaryObservation, DuressAckObservation,
    CanaryFreshness, NetworkHealth. Port direct des dataclasses
    Python vers des structs Rust avec `#[derive(Serialize, Deserialize)]`.
(b) **CanaryRegistry** struct :
    - `observe_canary(&mut self, obs: CanaryObservation)`
    - `observe_duress_ack(&mut self, obs: DuressAckObservation)`
    - `freshness(&self, pubkey_hex: &str) -> CanaryFreshness`
    - `network_health(&self) -> NetworkHealth`
    - `known_pubkeys(&self) -> Vec<String>`
(c) **Persistence** : JSON file (`canary_registry.json`).
    Pattern load_if_exists + persist identique au Python.
    `Mutex<CanaryRegistry>` pour thread safety.
(d) **Coerce functions** : `coerce_canary_payload(payload)` et
    `coerce_duress_ack_payload(payload)` pour le wire format daemon.
(e) **Freshness classification** : `_classify_canary_age(days)` →
    fresh/aging/stale/expired. Memes seuils que le Python (7/14/30j).

**Rejete** :
- SQLite pour persistence canary (JSON suffit, <100 observations
  attendues pre-v1.0, pas de queries complexes).
- Crate separe (366 LOC Python → ~200 LOC Rust, trop petit).
- Gossip sync distribue (Niveau 2 post-v1.0, necessiterait
  iroh-docs entries).

**Implications code** :
- `crates/nexus-coordinator-rs/src/canary_registry.rs` (NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (pub mod)
- Tests : 6-8 tests (observe, freshness, health, persist, coerce)

### D3 — Wire integration GuardrailChain + HTTP canary routes

**Retenu** :

(a) **PiiInputGuardrail dans GuardrailChain** : ajouter une fonction
    `default_input_chain()` dans guardrails.rs qui inclut
    PiiInputGuardrail. Wire dans le handler `coordinator_submit_task`
    AVANT dispatch (input direction).

(b) **CanaryRegistry HTTP routes** dans daemon http.rs :
    - `POST /api/canary/observed` : observe canary signing
    - `GET /api/canary/network-health` : aggregate health status
    - `GET /api/canary/freshness/:pubkey` : per-key freshness
    Ces routes remplacent le proxy Python correspondant.

(c) **State extension** : ajouter `canary_registry: Arc<Mutex<CanaryRegistry>>`
    a DaemonHttpState. Init au boot avec persist_path.

**Rejete** :
- Wire PII dans le validator_loop (le PII est un input guardrail,
  pas output — le validator_loop traite les results output).
- Routes canary separees du daemon (le daemon est le point d'entree
  unique, proxy pattern existant).

**Implications code** :
- `crates/nexus-coordinator-rs/src/guardrails.rs` (default_input_chain)
- `crates/nexus-shell-daemon/src/http.rs` (+3 routes + 3 handlers +
  canary_registry state + input guardrail wire)
- `crates/nexus-shell-daemon/src/runtime.rs` (init canary_registry)

### D4 — P2 batch carries

**Retenu** : resoudre les items P2 les plus proches du seuil
MANDATORY :
- P2-REVIEW-A-1-S37 launcher logging test 2/3 : investiguer ce qui
  reste partiel, completer la resolution. Si le test existant couvre
  deja l'invariant complet, documenter comme resolu.

**Rejete** :
- Resoudre les carries 1/3 (P2-REVIEW-A-1-S38, B-1, C-1) : trop
  tot, pattern normal de maturation.

### D5 — Scope cuts S39

1. **ML PII detection (ONNX/ort)** — post-v1.0 (dep instable, regex
   suffisant)
2. **CanaryRegistry gossip sync** — post-v1.0 (Niveau 2)
3. **canary_input.py migration** — S40 (Tier 2 fin, 782 LOC)
4. **redundancy/re-run/honeypot** — S40 (Tier 3)
5. **Migration complete coordinator** — S41+ (Tier 4 jalon)
6. **Suppression coordinator Python** — S45
7. **CI multi-OS release** — S46
8. **VPS deployment** — S47
9. **Tag v1.0** — S48
10. **Kudos debit/stake** — interdit (Day 0 #7)
11. **PII sliding window** — post-v1.0 (P3-REVIEW-B-2-S38 EED
    dilution, meme pattern appliquerait au PII)
12. **CanaryRegistry distributed gossip** — post-v1.0

---

**Acknowledged review findings (G1)** :

Scoring : D1 ⚠️, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (3 ⚠️ + 2 ✅ sur 5).

D1 ⚠️ (1 finding) :
- Regex phone patterns divergent du Python baseline (US/intl
  separes vs combine). **Accept** : les patterns seront portes
  directement du Python (`pii_redactor.py` L341+) en Phase A G8
  preflight. Le plan §A.2 est un schema, pas les regex finaux.
  Coverage pre-v1.0 acceptable (90%+).

D2 ⚠️ (1 finding) :
- Date parsing TZ ambigue Python↔Rust. **Accept** : les deux
  utilisent ISO 8601 / RFC 3339 UTC. Le format sera fixe a
  `YYYY-MM-DDTHH:MM:SSZ` (UTC explicite) dans les deux impls.
  Phase B documentera le format dans les structs.

D3 ⚠️ (1 finding) :
- Routes canary adjacence avec FROST `/api/canary/frost/*`.
  **Accept** : pas de collision nominale. Les deux sont des
  aspects du meme domaine (canary = observabilite vs frost =
  signing ceremony). Le namespace partage est intentionnel.

---

## §5 Plan Phase outline A..D

### Phase A — PiiRedactor Rust regex-only

**But** : migrer pii_redactor.py (regex subset) vers Rust.
- RedactionPolicy struct + from_toml
- PiiRedactor struct + compile regex patterns
- Luhn validation credit card
- PiiInputGuardrail adapter (Guardrail trait)
- Tests : 8-10 tests couvrant chaque pattern + Luhn + policy
- Commit : `feat(sprint39): Sprint 39 Phase A — PiiRedactor Rust
  regex-only pii_redactor.rs`

### Phase B — CanaryRegistry Rust

**But** : migrer canary_registry.py vers Rust.
- Types serde (Observation, Freshness, Health)
- CanaryRegistry struct (observe, freshness, health, persist)
- Coerce functions wire format
- Tests : 6-8 tests observe/freshness/health/persist/coerce
- Commit : `feat(sprint39): Sprint 39 Phase B — CanaryRegistry
  Rust canary_registry.rs`

### Phase C — Wire integration + P2 batch

**But** : wire PiiInputGuardrail + canary HTTP routes + P2.
- default_input_chain() dans guardrails.rs
- Wire dans submit_task handler (input direction)
- 3 routes HTTP canary + handlers + state
- P2-REVIEW-A-1-S37 launcher logging resolution
- Tests : 4-6 tests (input chain + canary routes)
- Commit : `feat(sprint39): Sprint 39 Phase C — wire PiiInput
  guardrail + CanaryRegistry HTTP + P2 batch`

### Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint40_audit_plan.md
- SPRINT_LOG.md row S39
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S39
- Migration `.planning/active/sprint39_audit_plan.md` →
  `.planning/archive/v1.2/`
- Commit : `chore(sprint39): Phase D — wrap-up + verification
  + audit plan S40 + migration`

---

## §6 Items carry/dette

### Resolus S39 (plan)

- [plan] PiiRedactor regex-only : Phase A
- [plan] CanaryRegistry port : Phase B
- [plan] Wire PiiInputGuardrail + canary HTTP : Phase C
- [plan] P2-REVIEW-A-1-S37 launcher logging test 2/3 : Phase C

### Carries confirmes S40

- [carry] P2-A-1 rand blocker upstream 6+/3 : blocker externe
  inchange. Exemption §6.2.1 blocker externe.
- [carry] P2-AUDIT-2-S35 pre-release transitives iroh : condition
  heritee pin 0.98.
- [carry] P3-grammar executor 3/3+ : S40 Tier 3 re-run/redundancy.
- [carry] P3-watermark executor 3/3+ : S40 Tier 3 meme justification.
- [carry] P2-REVIEW-A-1-S38 result_event_tx dead code 1/3 : wire
  gossip S40+.
- [carry] P2-REVIEW-B-1-S38 substring O(n*m) 1/3 : perf post-v1.0.
- [carry] P2-REVIEW-C-1-S38 chain Arc singleton 1/3 : perf post-v1.0.
- [carry] P3-AUDIT-A-2b-S38 lowercase divergence 1/3 : doc post-v1.0.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **ML PII detection (ONNX/ort)** — post-v1.0
2. **CanaryRegistry gossip sync** — post-v1.0 (Niveau 2)
3. **canary_input.py** — S40 (Tier 2 fin)
4. **redundancy/re-run/honeypot** — S40 (Tier 3)
5. **Migration routes API** — S42-44 (Tier 5)
6. **Suppression coordinator Python** — S45
7. **CI multi-OS release** — S46
8. **VPS deployment** — S47
9. **Tag v1.0** — S48
10. **Kudos debit/stake** — interdit (Day 0 #7)
11. **PII sliding window** — post-v1.0
12. **CanaryRegistry distributed** — post-v1.0

---

## §8 Tracabilite scope (S38 → S39)

| Item S38 carry / scope cut | Ou dans S39 |
|---|---|
| SC-1 PiiRedactor Rust | §5 Phase A (regex-only) |
| SC-2 CanaryRegistry Rust | §5 Phase B |
| P2-REVIEW-A-1-S37 launcher logging 2/3 | §5 Phase C |
| P2-REVIEW-A-1-S38 dead code 1/3 | §6 carry S40 |
| P2-REVIEW-B-1-S38 substring 1/3 | §6 carry S40 |
| P2-REVIEW-C-1-S38 chain singleton 1/3 | §6 carry S40 |
| P3-AUDIT-A-2b-S38 lowercase 1/3 | §6 carry S40 |
| SC-3 ML PII ONNX | §7.1 scope cut post-v1.0 |
| SC-4 canary_input.py | §7.3 scope cut S40 |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | Regex PII patterns incomplets vs Presidio NER (faux negatifs) | Low | Pre-v1.0 : 90%+ PII structuree couverte par regex (email/phone/SSN/CC/IP). ML post-v1.0 pour PII non-structuree (noms, adresses). |
| R2 | CanaryRegistry JSON persistence race condition multi-thread | Medium | Mutex<CanaryRegistry> + persist atomique (write tmp + rename). Pattern deja valide par Python (GIL = implicit lock). |
| R3 | PiiInputGuardrail false positives (regex trop aggressive) | Low | Policy TOML configurable per-pattern. Default conservateur (patterns standard seulement). |
| R4 | Wire input guardrail dans submit_task casse tests existants | Low | Les tests existants n'envoient pas de PII dans les task inputs. Guardrail pass-through par defaut. |

---

## §10 Audit gate pattern — rappel

Phase 0 (audit S38) deja jouee. Phase D devra produire
`sprint40_audit_plan.md` pour la session suivante.

---

## §11 Checkpoint de validation

1. **D1** : regex-only sans ML pour PiiRedactor, acceptable pre-v1.0 ?
   → recommandation : oui, 90%+ PII structuree. ML = post-v1.0.
2. **D2** : JSON persistence pour CanaryRegistry, vs SQLite ?
   → recommandation : JSON (< 100 observations, pas de queries
   complexes, pattern identique au Python).
3. **D3** : wire PiiInputGuardrail dans submit_task input, pas dans
   validator_loop output ?
   → recommandation : oui, PII = input guardrail (scan avant
   dispatch), pas output (le LLM ne genere pas de SSN).
4. **D4** : P2-REVIEW-A-1-S37 a 2/3, resoudre maintenant ou risquer
   3/3 MANDATORY S40 ?
   → recommandation : resoudre Phase C (trivial, investigation +
   doc/test).
5. **D5** : CanaryRegistry gossip distribue differe post-v1.0,
   acceptable ?
   → recommandation : oui, le registry local est suffisant pour un
   reseau a 1 noeud pre-v1.0.
