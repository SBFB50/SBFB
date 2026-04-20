# Sprint 22 — Plan détaillé (Sybil-resistance composition 3 couches + rate-limit engine wire + GLiNER span-decoder + NVML baseline + watermark canari primitive + process fixes)

**Écrit** : 2026-04-19 (session fraîche post-audit gate S21
`96a953b`).
**Tip master d'entrée** : `96a953b` (chore(sprint21): audit gate
S21 — findings verdict PASS).
**Source D1..D5 gelées** : `sprint22_kickoff.md §4`.
**Range audit-plan cible Phase F** : commits S22 ouverture →
Phase F wrap-up.

---

## 1. État vérifié à l'entrée

### 1.1 Tip + tests (rappel kickoff §2)

- HEAD : `96a953b` (2026-04-19).
- Baseline tests S22 entrée : **659 Rust / 185 SDK / 249+3 skipped
  coord / 46 gov / 256 Vitest / 38 Playwright / 7/7 size / 246+
  SPDX ≈ 1436 tests total**.
- Delta S21 livré = **+65** vs baseline S20 (1371).

### 1.2 Clippy + size + lint

Tous verts S21 fin (cf. kickoff §2.2). Aucune régression attendue
pré-S22 Phase A.

### 1.3 Audit gate S21 leveraged

- Meta-1 Radicle-v1.0 reclassification **LT-2** régularisée au
  kickoff S22 (cf. kickoff §4 D5).
- 6 P2 + 4 P3 carry findings tracés résolus S22 phases A/B/C/F
  (cf. §12 carry summary).
- 3 tech debts hors cap : T-NN + T-NN+1 fermés S21 Phase E,
  T-NN+2 iframe Rust-wasm **ouvert PATTERNS §P34** hors cap formel.

### 1.4 Pre-launch protocol policy

- `BLOB_VERSION = 0x01`, `TASK_RESPONSE_VERSION = 1`,
  `CANARY_VERSION = 1`, `ANNOUNCEMENT_VERSION = 1` inchangés S22.
- **Nouveau wire format introduit S22** : `ContributorAttestation
  v1` + `AgeWitness v1` + `DelegationCert v1` (design-only) en
  pre-launch policy (pas de bump, format stable redéfini jusqu'à
  v1.0).

---

## 2. Décisions Day 0 gelées (rappel kickoff §4)

| # | Décision | Cœur technique |
|---|---|---|
| D1 | Sybil-resistance composition 3 couches | Couche 1 age node_id ≥7j + PoW S19 réutilise `gossip.rs:140-162` + `AgeWitness` peer-attestation Ed25519 / Couche 2 `ContributorAttestation` predicate in-toto extend `ProvenanceRecord` S14 + wire `curator.rs:252-274` / Couche 3 design-only RFC `SBFB.json::contributions[]` + delegation cert |
| D2 | Scope γ hybride 6 phases | A rate-limit wire / B span-logits decoder / C Sybil composé / D NVML foundation / E watermark canari / F wrap + process fixes |
| D3 | NVML baseline log-only | `nvml-wrapper 0.12.1` stats-only, foundation S24 (pas anomaly) |
| D4 | Watermark canari-input | Primitive consumer 1/N known-answer Ed25519 coord-signed (distinct watermark-output Kirchenbauer) |
| D5 | Cap G7 1/2 + LT-2 | Slot 1 T-NN+2 iframe Rust-wasm hors cap formel / Slot 2 LIBRE / LT-2 Meta-1 Radicle reclassification |

---

## 3. Research consulté (pré-gel, listé pour Phase 0 audit S23)

### Sybil-resistance D1
- `gossipsub-v1.1` spec libp2p (score P₁..P₇ canonical)
- IEEE S&P 2024 arXiv 2212.05197 (formal impossibility score-only)
- Tor Guard flag (dirauth-centralised, day-8 WFU)
- Nostr NIP-13 (brisé ≤20 bits empirical 2025)
- S19 Phase B `edfc51b` PoW Hashcash 2^18 live
- S14 `95807b1` `ProvenanceRecord` Ed25519 `DOMAIN_PROVENANCE_V1`
- in-toto attestation framework (predicates extensible)
- Radicle Heartwood 1.8.0 Ed25519 did:key (vuln replay 2026-03-30)
- Git 2.34+ SSH signing `allowed_signers` universel 2026

### NVML D3
- `nvml-wrapper 0.12.1` (2026-03-27) + `last_seen_timestamp` 0.11.0
- DCGM exporter (rejet lourd)
- MagTracer ACM MobiCom 2023 (rejet hardware)

### Watermark D4
- Kirchenbauer 2023 ICML (rejet vulnérable BIRA 2025)
- `LLM-Canary` OSS (pas distributed)
- Pattern nexus-grid canari-input distinct watermark-output

### Rate-limit wire-up Phase A
- `governor 0.10.2` déjà intégré S21 (primitive live non-câblée)
- `tower-governor 0.8` axum 0.8 pattern
- Arc swap hot-reload pattern

### GLiNER span-decoder Phase B
- `urchade/GLiNER` paper output format (sigmoid + greedy dedup)
- `GLiNER.js` npm référence algorithme (TS ~1500 LOC)
- `@xenova/transformers.js v4` + `onnxruntime-web 1.24.3` déjà
  chargés S21 Phase B

---

## 4. Phase A — Rate-limit engine wire-up (P2-S21-1 + P2-S21-2 + P2-S21-6)

### 4.1 Pré-requis G8 preflight

Invoquer skill `nexus-phase-preflight` avant 1re ligne code. Scans
S1-S4.

**Scan S1 obligatoire** : fresh `governor 0.10.2` changelog +
`DefaultKeyedRateLimiter` Arc swap pattern via context7 (mentionner
pre-flight si breaking change 0.10.x entre 2025-11 release et
2026-04).

**Verdict attendu** : EXECUTE (wire mécanique, pattern trivial).

### 4.2 Fichiers modifiés

- **`crates/nexus-worker-core/src/engine/runtime.rs`** (modifié) :
  wire `rate_limiter.check(RateKey)` AVANT `ClaimEntry` broadcast
  (~ligne 150 selon audit agent 8). Reject flow : si `NotUntil(ts)`
  retour, defer task sans broadcast claim.
- **`crates/nexus-worker-core/src/rate_limit.rs`** (modifié) :
  ajout méthode `pub fn swap_policy(&self, new: RateLimitPolicy)`
  Arc swap `DefaultKeyedRateLimiter` (hot-reload P2-S21-2).
- **`crates/nexus-worker-core/src/rate_limit_policy_loader.rs`**
  (modifié) : appel `rate_limiter.swap_policy(...)` dans callback
  hot-reload (pattern S20 `pow_policy_loader.rs::on_reload`).
- **`crates/nexus-worker-core/configs/rate_limit_policy.toml.sample`**
  (nouveau — fix P3-S21-4) : template default budgets tier +
  overrides documenté.
- **Tests** : `crates/nexus-worker-core/src/engine/runtime.rs#tests`
  intégration engine (~5-7 tests).

### 4.3 Tests à écrire

Tests intégration engine :

1. `engine::rate_limit_gate_rejects_saturated_tuple` : engine reçoit
   task pour tuple saturé, `should_claim()` retourne false, pas de
   `ClaimEntry` broadcast.
2. `engine::rate_limit_gate_admits_fresh_tuple` : engine reçoit
   task pour tuple fresh, claim broadcast normal.
3. `engine::rate_limit_gate_reloads_live_policy` : mutation
   `~/.sbfb/rate_limit_policy.toml` runtime, vérifier nouvelles
   quotas appliquées sans restart engine.
4. `engine::rate_limit_gate_defer_preserves_task` : reject ne
   detruit pas la task (pending SQLite coord-side intact).
5. `engine::rate_limit_policy_sample_loader_smoke` : load
   `configs/rate_limit_policy.toml.sample`, vérifier parse valide.

Tests policy loader Arc swap :

6. `rate_limit_policy_loader::swap_preserves_unsaturated_tuples` :
   policies change, tuples non-saturés avant restent non-saturés
   après swap (cohérence état transitoire).
7. `rate_limit_policy_loader::swap_clears_saturated_state` : policies
   change + nouveau budget plus laxe, tuples saturés avant retournent
   à admissibles (reconstruction `DefaultKeyedRateLimiter`).

**+7 tests Rust attendus** (delta Phase A = +7).

### 4.4 Critère d'acceptation Phase A

- `cargo nextest run -p nexus-worker-core --locked` vert (+7 tests).
- `cargo nextest run --workspace --locked` vert ≥ 666 tests.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
  0 warning.
- `cargo fmt --all --check` 0 diff.
- `cargo test --workspace --locked --doc` vert.
- Fichier `configs/rate_limit_policy.toml.sample` créé (fix
  P3-S21-4).

### 4.5 Commit cible Phase A

```
feat(sprint22): Phase A — rate-limit engine wire-up + hot-reload + policy sample

Body riche :
- Delta tests +7 Rust (integration engine gate + policy Arc swap)
- Résout P2-S21-1 + P2-S21-2 + P3-S21-4 (sample)
- HARDENING §3 S21 wording fix (P2-S21-6) inclus dans chore planning ouverture S22
- Working tree audit G5 (PHASE / CRAFT / DEBT / NOISE)
- G8 pre-flight Phase A preflight.md verdict EXECUTE
```

---

## 5. Phase B — GLiNER span-logits decoder iframe (P2-S21-3)

### 5.1 Pré-requis G8 S1 scan

Fresh fetch via WebFetch :
- `urchade/GLiNER` paper output format (sigmoid + greedy dedup
  algorithm pseudocode).
- `GLiNER.js` npm dernière release + licence compatibilité (mars
  2025 ok).
- `onnxruntime-web 1.24.3` inference output tensor shape
  documentation pour GLiNER model `gliner-pii-edge-v1.0`.

**Verdict attendu** : EXECUTE (fallback existant préserve, pas de
design conflict).

### 5.2 Fichiers modifiés

- **`web/src/sdk/pii/wrapper.ts`** (modifié lignes 82-108) :
  remplacer `return []` scaffold par :
  ```typescript
  // Decoder span-logits → Finding[]
  const [start_logits, end_logits, span_logits] = outputs;
  const spans = decodeSpans(start_logits, end_logits, span_logits,
                            tokens, this.policy.threshold);
  const deduplicated = greedyDedup(spans);
  return deduplicated.map(toFinding);
  ```
- **`web/src/sdk/pii/decoder.ts`** (nouveau) : module pur
  `decodeSpans()` + `greedyDedup()` + `toFinding()` (~250 LOC TS).
- **`web/src/sdk/pii/__tests__/decoder.test.ts`** (nouveau) : tests
  Vitest unit (+4-6 tests).
- **`web/src/sdk/pii/__tests__/wrapper.test.ts`** (modifié) :
  assertions spans réels (pas `[]`) sur fixture texte connu.

### 5.3 Tests à écrire

Vitest unit decoder :

1. `decoder::decodeSpans_single_entity` : input fixture tensor
   avec 1 span email → 1 Finding retourné avec {entity: "EMAIL",
   start, end, confidence}.
2. `decoder::decodeSpans_multiple_overlapping` : 3 spans
   overlapping, greedy dedup conserve plus haute confiance.
3. `decoder::decodeSpans_threshold_filter` : spans sous threshold
   rejetés.
4. `decoder::decodeSpans_empty_input` : texte vide → `[]`.
5. `decoder::greedyDedup_non_overlap_preserved` : spans non-
   chevauchants tous conservés.
6. `wrapper::detect_real_fixture` (modifié) : texte `"Contact:
   alice@example.com and +33 6 12 34 56 78"` retourne ≥ 2 Findings
   (email + phone).

Optionnel Playwright (si fixture mini-model bundlable ≤ 10 MB) :
- Drift audit S21 Track B `P2-E-PII-PLAYWRIGHT-DRIFT` : noter
  investigation fixture model dédiée dans `sprint22_verification.
  md §4`. Pas de Playwright end-to-end S22 si bundle trop lourd
  CI.

**+6 Vitest tests attendus** (delta Phase B = +6).

### 5.4 Critère d'acceptation Phase B

- `cd web && npm run test:unit` vert (+6 Vitest, baseline
  256 → 262).
- `npm run lint` + `npx tsc --noEmit -p tsconfig.app.json` 0 erreur.
- `npm run build` + `npm run size` 7/7 pass (bundle iframe PII
  ≤ size-limit).
- `bash scripts/scan-en-strings.sh` 0 unexpected EN.
- `OnnxModelHandle.detect()` retourne spans non-vides sur fixture
  test.

### 5.5 Commit cible Phase B

```
feat(sprint22): Phase B — GLiNER span-logits decoder iframe SDK

Body riche :
- Delta tests +6 Vitest (decoder unit + wrapper fixture integration)
- Résout P2-S21-3 (scaffold → decoder fonctionnel)
- Audit Track B drift Playwright end-to-end : investigation mini-model fixture documentée (pas livré S22, carry S23)
- Working tree audit G5
- G8 pre-flight Phase B verdict EXECUTE
```

---

## 6. Phase C — Sybil-resistance composition 3 couches (Couches 1 + 2 live, Couche 3 design-only)

### 6.1 Pré-requis G8 S1-S4 OBLIGATOIRES

**Scan S1** — fresh fetch :
- `in-toto/attestation` predicate format stable 2026 (champs
  `subject`, `predicate_type`, `predicate`).
- `gossipsub-v1.1` P₅ application-specific intégration Rust
  `libp2p-gossipsub 0.49.x` (référence, pas bind direct iroh).
- Radicle Heartwood 1.8.0 `did:key` `z6Mk...` format (pattern
  similaire node_id SBFB).

**Scan S2** — historical decisions traversed :
- Threat-model S18 E2 `04c9621` rejet auto-publish scheduler —
  applicable ici ? NON (ContributorAttestation est signée par
  coordinator volontaire au deploy, pas scheduler). ✓
- S16 `is_open_source` flag v5 ProjectAnnouncement (dériver from
  deploy-from-repo). Réutilisation S22 Phase C : vérifier cohérence.

**Scan S3** — threat model coverage :
- B-Sybil §1 matrix : tier max T2+ pre-S19 / T5 post-S19+S22
  mention. S22 Phase C **fait ce bump** de T2 à T3 mitigation
  partielle (Couche 3 S27 pour T3 complet).
- C-ModelExtract §1 : rate-limit per-consumer (livré S21 Phase A,
  câblé S22 Phase A). Non-scope Phase C.

**Scan S4** — wire format invariants :
- `DOMAIN_AGE_WITNESS_V1`, `DOMAIN_CONTRIBUTOR_ATTESTATION_V1`,
  `DOMAIN_DELEGATION_CERT_V1` — nouveaux domain tags.
- Ajout dans `crates/nexus-core-rs/src/canonical.rs`
  (lignes 50-138 existantes pour autres domains).
- Pre-launch policy : format stable redéfini jusqu'à v1.0. Pas de
  bump `_VERSION` (tous inchangés).

**Verdict attendu** : EXECUTE si scans clean, ou SCOPE-CUT-CONSISTENT
si ajustements minimes arbitrables inline. DESIGN-CONFLICT
improbable (design validé 3 couches arbitré user 2026-04-19
post-synthèse G1).

### 6.2 Fichiers ajoutés / modifiés

#### Couche 1 — age node_id ≥7j + PoW S19 + AgeWitness + Bootstrap allowlist (P0-G1-1)

- **`crates/nexus-shell-daemon-core/src/bootstrap_allowlist.rs`**
  (nouveau ~100 LOC — **P0-G1-1 ack**) : seed list ≤ 20 nodes
  marqués `bootstrap_phase` permettant self-witness pendant la
  phase bootstrap pré-v1.0. Hot-reload pattern `pow_policy_loader.
  rs` S20. Expire automatique `expires_after: v1.0` (tag go-live
  trigger, aligné LT-2 Radicle).
- **`~/.sbfb/bootstrap_allowlist.toml`** (config) : format :
  ```toml
  [[bootstrap]]
  node_id_hex = "..."
  added_at = "2026-04-19"
  reason = "initial publisher seed"
  expires_at_tag = "v1.0"
  ```
- **`crates/nexus-core-rs/src/attestations/mod.rs`** (nouveau) :
  module parent attestations.
- **`crates/nexus-core-rs/src/attestations/age_witness.rs`**
  (nouveau ~200 LOC) :
  ```rust
  pub struct AgeWitness {
      pub node_id: NodeId,
      pub first_seen_ts: i64,
      pub witness_pubkey: NodeId,
      pub witness_sig: Ed25519Signature,
  }
  impl AgeWitness {
      pub fn sign(node_id: &NodeId, first_seen: i64,
                  witness: &SigningKey) -> Self { ... }
      pub fn verify(&self) -> bool { ... }
      pub fn age_days(&self, now: i64) -> i64 { ... }
  }
  pub const DOMAIN_AGE_WITNESS_V1: &[u8] = b"nexus-age-witness-v1";
  pub const MIN_AGE_DAYS: i64 = 7;
  ```
- **`crates/nexus-core-rs/src/canonical.rs`** (modifié) : ajout
  `DOMAIN_AGE_WITNESS_V1` domain tag.
- **`crates/nexus-core-rs/src/gossip.rs`** (modifié ~ligne 140) :
  `join_topic_with_age_witness()` extend avec paramètre
  `Option<&AgeWitness>` + `bootstrap_allowlist: &BootstrapAllowlist` ;
  logique admission :
  1. Si `node_id ∈ bootstrap_allowlist` → accept self-witness
     (P0-G1-1 bootstrap ceremony).
  2. Sinon si witness fourni et witness_pubkey est un peer existant
     connu ≥30j dans le gossip mesh → vérifier age ≥ MIN_AGE_DAYS
     + signature valide.
  3. Sinon fallback PoW-only (rétrocompat bootstrap très-early,
     flag log warn).
- **`crates/nexus-core-py/src/lib.rs`** (modifié) : PyO3 binding
  `py_verify_age_witness(bytes: &[u8]) -> bool`.

#### Couche 2 — ContributorAttestation extend ProvenanceRecord (P0-G1-2 + P2-G1-3)

**Livrable obligatoire AVANT 1re ligne code Phase C** (P0-G1-2 ack) :

- **`docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md`**
  (nouveau ~200 LOC docs) :
  - Section 1 : motivation + conformité in-toto v1.0 spec
  - Section 2 : `predicateType = "https://nexus-grid.org/
    contributor-attestation/v1"` (URI stable pre-launch)
  - Section 3 : JSON schema draft-07 (pattern S20 Phase D
    `task_response.schema.json`)
  - Section 4 : Fields `contributor_node_id` (hex 32 bytes),
    `first_deploy_ts` (int64 unix), `commit_sha` (string git
    SHA-1 hex), `repo_url` (string), `attestation_coord_sig`
    (base64 Ed25519 64 bytes)
  - Section 5 : Envelope structure in-toto v1.0 (`subject[]` +
    `predicateType` + `predicate{}`)
  - Section 6 : Verification procedure offline (réutilise
    `nexus_core::verify_bytes` pattern S14)
  - Section 7 : Exemples JSON minimal + multi-subject
  - Section 8 : Limitations (P2-G1-3 ack) — « Cette spec atteste
    de la **contribution** mais pas de l'**équité distribution**.
    L'équité est gouvernée par LT-1 post-v1.0 refonte kudos. »

**Pré-req Phase C S1 scan obligatoire** : fetch fresh in-toto v1.0
spec (https://github.com/in-toto/attestation) + vérifier pas de
breaking change post-2025-Q4.

- **`crates/nexus-core-rs/src/attestations/contributor.rs`**
  (nouveau ~300 LOC) :
  ```rust
  pub struct ContributorAttestation {
      pub predicate_type: String,  // "nexus-grid/contributor-attestation/v1"
      pub subject: Vec<InTotoSubject>,
      pub predicate: ContributorPredicate,
  }
  pub struct ContributorPredicate {
      pub contributor_node_id: NodeId,
      pub first_deploy_ts: i64,
      pub commit_sha: String,
      pub repo_url: String,
      pub attestation_coord_sig: Ed25519Signature,
  }
  impl ContributorAttestation {
      pub fn build(coord: &SigningKey, provenance: &ProvenanceRecord)
                   -> Self { ... }
      pub fn verify(&self, coord_pubkey: &NodeId) -> bool { ... }
  }
  pub const DOMAIN_CONTRIBUTOR_ATTESTATION_V1: &[u8]
      = b"nexus-contributor-attestation-v1";
  ```
- **`crates/nexus-core-rs/src/curator.rs`** (modifié lignes 252-274) :
  `CuratorListEntry::verify_signature()` extend prise param
  `contributor_registry: &ContributorRegistry` + check
  `is_verified_contributor(project_id, curator_pubkey)` si flag
  gouvernance-forte activé pour projet.

  **Code comment obligatoire P2-G1-3 LT-1 TODO** (inséré dans
  `verify_with_contributor_registry()` et
  `ContributorAttestation::build()`) :
  ```rust
  // NOTE: Interim Sybil-resistance S22. Contributor selection
  // still biased toward high-kudos workers (Matthew effect one
  // layer deeper). Post-v1.0 LT-1 Kudos-v2 reform will introduce
  // log-utility + DRF + EMA trust to break this cycle. See:
  // - docs/FAIRNESS_VISION.md §7 "Design-conflict S22"
  // - docs/release/ROADMAP_COMMITMENTS.md §LT-1
  ```
- **`crates/nexus-core-rs/src/canonical.rs`** (modifié) : ajout
  `DOMAIN_CONTRIBUTOR_ATTESTATION_V1`.
- **`crates/nexus-core-py/src/lib.rs`** (modifié) : PyO3 binding
  `py_build_contributor_attestation()` + `py_verify_contributor_
  attestation()`.

- **`packages/nexus-coordinator/src/nexus_coordinator/contributor_
  registry.py`** (nouveau ~150 LOC) :
  - SQLite `contributor_attestations` table (schema : `project_id`,
    `contributor_node_id`, `first_deploy_ts`, `commit_sha`,
    `repo_url`, `coord_sig`, `attestation_bytes`).
  - `ContributorRegistry.record(attestation)` lors du
    verified-deploy (hook dans `api/deploy.py` après
    `provenance.sign()`).
  - `ContributorRegistry.is_verified(project_id, node_id) -> bool`.
  - REST `GET /api/contributor/verify/{project_id}/{node_id_hex}`.
- **`packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py`**
  (modifié) : hook `ContributorAttestation` build + record après
  `generate_provenance()`.
- **`crates/nexus-shell-daemon/src/http.rs`** (modifié) : proxy
  loopback `/api/contributor/verify/...` (pattern S13 proxy coord
  endpoint).

#### Couche 3 — DESIGN-ONLY S22 (RFC interne)

- **`docs/security/CONTRIBUTOR_ATTESTATION_RFC.md`** (nouveau ~250
  LOC docs) :
  - Section 1 : motivation et scope multi-forge
  - Section 2 : format `SBFB.json::contributions[]` extension
  - Section 3 : `DelegationCert` Ed25519 format (node_id SBFB
    signe SSH key fingerprint)
  - Section 4 : parser `git log --show-signature` offline pattern
  - Section 5 : cross-validate multi-forge (Radicle + Codeberg +
    Forgejo + GH)
  - Section 6 : trust-web Amnesty integration S27
  - Section 7 : triggers re-activation phase implementation S23-S27

### 6.3 Tests à écrire

Tests Rust attestations :

1. `age_witness::sign_verify_roundtrip` : sign → verify OK.
2. `age_witness::verify_rejects_tampered` : mutation → verify fail.
3. `age_witness::age_days_precision` : computation delta + edge
   cases (witness future, witness ancient).
4. `age_witness::min_age_enforced` : `<7j` rejected, `≥7j` admitted.
5. `contributor_attestation::build_from_provenance` : build valide
   depuis ProvenanceRecord S14 ok.
6. `contributor_attestation::verify_coord_signature` : valid sig
   → OK, tampered → fail.
7. `contributor_attestation::predicate_format_in_toto_compat` :
   JSON serialize conforme in-toto spec.
8. `curator::verify_rejects_non_contributor_if_enforce` : curator
   pubkey non-registered + projet avec flag gouvernance-forte → verify
   reject.
9. `curator::verify_admits_registered_contributor` : pubkey registered
   → verify OK.
10. `gossip::join_topic_with_age_witness_admits` : witness valide
    ≥7j → join OK.
11. `gossip::join_topic_with_age_witness_rejects_under_min` :
    witness <7j → join reject.
12. `gossip::join_topic_fallback_pow_only_if_no_witness` : witness
    absent + PoW OK → join OK (bootstrap backward-compat).

Tests Python coord :

13. `contributor_registry::record_on_deploy` : verified-deploy flow
    trigger record.
14. `contributor_registry::is_verified_boolean` : query par project_id
    + node_id retourne bool correct.
15. `contributor_registry::sql_schema_migration` : schema creation
    idempotent.
16. `api_deploy::emit_contributor_attestation_post_provenance` :
    deploy flow enchaîne provenance + attestation correctement.

Tests supplémentaires bootstrap allowlist (P0-G1-1 ack) :

17. `bootstrap_allowlist::load_toml_schema` : parse valid TOML.
18. `bootstrap_allowlist::is_bootstrap_node` : node in list → true.
19. `bootstrap_allowlist::rejects_expired` : `expires_at_tag`
    posé sur master → reject.
20. `gossip::join_topic_admits_bootstrap_self_witness` : bootstrap
    node + self-witness → accept.

**+16 tests Rust + +4 tests Python coord attendus** (delta
Phase C = +20 vs +16 initial G1 ajust).

### 6.4 Critère d'acceptation Phase C

- `cargo nextest run --workspace --locked` vert ≥ 682 tests
  (baseline post-Phase A 666 + 16 Phase C = 682).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
  0 warning.
- `uv run pytest packages/nexus-coordinator/tests/ -q` vert (253+
  tests, baseline 249+3 skipped + 4).
- Documents **obligatoires** créés :
  - `docs/security/CONTRIBUTOR_ATTESTATION_PREDICATE.md` (P0-G1-2)
  - `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` (Couche 3 design-only)
  - Cross-referenced `HARDENING_ROADMAP §3 S27` pivot note.
- `BLOB_VERSION` / `TASK_RESPONSE_VERSION` / `CANARY_VERSION` /
  `ANNOUNCEMENT_VERSION` **inchangés** (pre-launch policy).

### 6.5 Commit cible Phase C

```
feat(sprint22): Phase C — Sybil-resistance composition 3 couches (age witness + contributor attestation + Couche 3 RFC)

Body riche :
- Delta tests +16 (+12 Rust + 4 Python coord)
- Couche 1 AgeWitness peer-attestation + gossip join_topic extend
- Couche 2 ContributorAttestation predicate in-toto compat + curator verify extend + coord registry
- Couche 3 DESIGN-ONLY : docs/security/CONTRIBUTOR_ATTESTATION_RFC.md créé (S23-S27 implem carry)
- Résout §3 S22 item 1 (scope-réaligné FAIRNESS-compatible via kickoff §4 D1)
- Working tree audit G5
- G8 pre-flight Phase C verdict EXECUTE (ou SCOPE-CUT-CONSISTENT si ajustement mineur)
```

---

## 7. Phase D — NVML baseline log-only foundation S24

### 7.1 Pré-requis G8 S1 scan

Fresh : `nvml-wrapper 0.12.1` docs.rs API `last_seen_timestamp` +
Windows build path vérifié (pattern S20 Phase A NASM deviation
non applicable ici).

**Verdict attendu** : SCOPE-CUT-CONSISTENT (stats-only scope, pas
anomaly detection — simplification).

### 7.2 Fichiers ajoutés / modifiés

- **`crates/nexus-worker-core/src/nvml_profile.rs`** (nouveau ~250
  LOC) :
  ```rust
  pub struct NvmlProfile {
      nvml: Nvml,
      db_path: PathBuf,
      sampling_interval: Duration,
  }
  impl NvmlProfile {
      pub fn new(db_path: PathBuf) -> Result<Self, NvmlError> { ... }
      pub async fn start_sampling(&self) -> JoinHandle<()> { ... }
      pub async fn stats_for_window(&self, window: Duration)
                                    -> NvmlWindowStats { ... }
  }
  pub struct NvmlWindowStats {
      pub gpu_util_avg: f32,
      pub gpu_util_p95: f32,
      pub vram_used_avg_mb: u64,
      pub compute_processes_count: u32,
      pub last_seen_timestamps: Vec<i64>,
  }
  ```
- **`~/.sbfb/nvml_profile.db`** : SQLite schema `nvml_samples`
  (timestamp, gpu_util, vram_used_mb, compute_processes_json).
- **`Cargo.toml` workspace** : `nvml-wrapper = "0.12.1"`.
- **`crates/nexus-worker-core/Cargo.toml`** : `nvml-wrapper =
  { workspace = true }`.

### 7.3 Tests à écrire

1. `nvml_profile::new_creates_schema` : init crée table SQLite.
2. `nvml_profile::sampling_persists_row` : sample runtime persiste
   une row.
3. `nvml_profile::stats_for_window_empty` : pas de data →
   empty stats (no panic).
4. `nvml_profile::stats_for_window_computes_avg_p95` : fixture
   rows → avg + p95 corrects.
5. `nvml_profile::handles_no_gpu_gracefully` : host sans GPU →
   `NvmlError::NotAvailable` pas panic.

**+5 tests Rust attendus** (delta Phase D = +5). CI mock NVML
`MockNvml` pattern pour tests headless sans GPU.

### 7.4 Critère d'acceptation Phase D

- `cargo nextest run -p nexus-worker-core --locked` vert (+5).
- `cargo nextest run --workspace --locked` vert ≥ 683 tests.
- Bench RTX 5080 sampling overhead ≤ 1% CPU (manuel, noter dans
  `sprint22_verification.md §4`).

### 7.5 Commit cible Phase D

```
feat(sprint22): Phase D — NVML util+duree profile log-only baseline foundation S24

Body riche :
- Delta tests +5 Rust (NvmlProfile + stats_for_window)
- Foundation pour S24 random re-run sampling (HARDENING §3 S24 dep)
- Scope-réduit stats-only (pas anomaly detection, pas ML)
- Working tree audit G5
- G8 pre-flight Phase D verdict SCOPE-CUT-CONSISTENT (stats-only)
```

---

## 8. Phase E — Watermark canari-input primitive

### 8.1 Pré-requis G8 S1 scan

Fresh : vérifier pas de regression BIRA attack arXiv 2509.23019
(confirmé septembre 2025) — pattern canari-input distinct donc
non impacté.

**Verdict attendu** : EXECUTE (primitive simple, pas ML).

### 8.2 Fichiers ajoutés / modifiés

- **`packages/nexus-coordinator/src/nexus_coordinator/canary_input.py`**
  (nouveau ~250 LOC) :
  - `class CanaryInputSet` : set de (prompt, expected_answer,
    tolerance) signé Ed25519 coord, rotatable.
  - `class CanaryInputInjector` : hook pre-dispatch 1/N tasks.
  - `class CanaryInputObserver` : hook post-result, compute
    Levenshtein similarity, alerte si < tolerance.
- **`packages/nexus-coordinator/src/nexus_coordinator/api/canary.py`**
  (modifié) : ajout endpoints `POST /canary/inject-rate` +
  `GET /canary/observed-divergence`.
- **`packages/nexus-coordinator/cli/commands.py`** ou équivalent
  Typer (modifié) : CLI `nexus-coordinator canary-rotate` +
  `canary-status`.
- **`~/.sbfb/canary_input_policy.toml`** : format config
  (inject_rate, tolerance, rotation_frequency).

### 8.3 Tests à écrire

1. `canary_input::inject_rate_1_per_100` : fixture 100 tasks, ≥ 1
   injection présente.
2. `canary_input::signature_rotation` : rotate CLI → nouveau set
   valide, ancien rejeté.
3. `canary_input::observer_alert_on_low_similarity` : mock result
   divergent → alert triggered.
4. `canary_input::observer_pass_on_high_similarity` : mock result
   proche → no alert.
5. `canary_input::api_endpoints_smoke` : `/canary/inject-rate`
   200 + `/canary/observed-divergence` 200.

**+5 tests Python coord attendus** (delta Phase E = +5).

### 8.4 Critère d'acceptation Phase E

- `uv run pytest packages/nexus-coordinator/tests/ -q` vert
  (258+ tests).
- CLI `nexus-coordinator canary-rotate --help` exits 0.

### 8.5 Commit cible Phase E

```
feat(sprint22): Phase E — watermark canari-input spot-check consumer 1/N primitive

Body riche :
- Delta tests +5 Python coord (inject + observer + rotation)
- Primitive distinct watermark-output Kirchenbauer (vulnérable BIRA 2025)
- Gap prior art académique documenté (opportunité OSS nexus-grid)
- Working tree audit G5
- G8 pre-flight Phase E verdict EXECUTE
```

---

## 9. Phase F — Wrap-up + verification + audit plan S23 + process fixes

### 9.1 Fichiers ajoutés / modifiés

- **`.planning/active/sprint22_verification.md`** (nouveau) : fail-
  fast checklist 30+ rows (suites §7.4 full + phase-specific asserts
  A/B/C/D/E).
- **`.planning/active/sprint22_audit_plan.md`** (nouveau) : 7+
  tracks (A-F + meta-tracks).
- **`.planning/active/sprint22_carry_summary.md`** (nouveau) :
  - Cap G7 : 1/2 slots utilisés.
  - Slot 1 : T-NN+2 iframe Rust-wasm hors cap PATTERNS §P34.
  - Slot 2 : LIBRE (ou audit findings post-Phase F).
- **`docs/claude/README.md §4.X`** (modifié — fix P2-S21-4) :
  règle Phase F "parse each `sprint{N}_phase_[A-F]_review.md`
  et intégrer P2/P3 dans audit_plan Track correspondant".
- **`.github/workflows/phase-review-cross-check.yml`** (nouveau —
  fix P2-S21-5) : GHA parse `git log --format='%s' master..HEAD
  | grep 'feat(sprint\d\+): Phase [A-F]'` et fail si review file
  absent.
- **`.claude/.bypass_audit_trail.log`** (nouveau — fix P2-S21-5
  follow-up) : trace chaque usage `NEXUS_SKIP_PHASE_AUDITOR=1`.
- **`CLAUDE.md` §État actuel** : row Sprint 22 CLOSED ajoutée.
- **`docs/claude/SPRINT_LOG.md v1.2`** : row S22 finale.
- **`docs/security/HARDENING_ROADMAP.md`** : `last_validated:
  2026-05-XX` bump (date Phase F).
- **Migration PARA** : `git mv .planning/active/sprint22_*.md`
  + `sprint21_audit_findings.md` → `.planning/archive/v1.2/`.
- **agents_sudo D1 trust tiers doc absorption** (ajouté S22
  hors-sprint 2026-04-20 post Phase B `e9530c2`, cf.
  `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`
  §1 Cluster D + §7) :
  - **`docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`** (déjà
    créé hors-sprint, vérifier présence Phase F — doc-only design,
    3 tiers AUTO / CONFIRM_PROMPT / BIOMETRIC_GATE, inventaire
    endpoints loopback + tier cible, extension `consent.json`
    schema `level_threat_note` + `residual_threats_acknowledged`).
  - **Vérifier** aucun code Rust/Python introduit S22 sur ce item
    (pur doc absorption). Implementation T1 `CONFIRM_PROMPT` =
    S25 co-landing D5. T2 `BIOMETRIC_GATE` = LT-4 post-v1.0.
  - **Note frontmatter HARDENING §3 S22** bump `last_validated`
    et entrée `audited_findings` "2026-04-20 S22 hors-sprint
    agents_sudo integration" déjà posée au moment de la Phase F.

### 9.2 Tests à écrire

Pas de tests code Phase F (wrap doc-only + process hooks).

### 9.3 Critère d'acceptation Phase F

- Fail-fast checklist `sprint22_verification.md` 30+ rows verte.
- Audit plan S23 émis (`sprint22_audit_plan.md`).
- Carry summary émis (cap G7 1/2).
- GHA workflow phase-review cross-check green sur dry-run master.
- Migration PARA terminée (active/ vide ou contenant uniquement
  `sprint22_audit_findings.md` si audit gate fixes post-Phase F).

### 9.4 Commit cible Phase F

```
chore(sprint22): Phase F — wrap-up + verification + audit plan S23 + process fixes (P2-S21-4 + P2-S21-5) + migrate planning

Body :
- Delta tests cumulés +39 Rust + +6 Vitest + +9 Python coord ≈ +54 (baseline 1436 → ~1490)
- Fail-fast checklist §Fail-fast verte (30+ rows)
- Process fixes P2-S21-4 README §4.X + P2-S21-5 GHA workflow
- Migration PARA active→archive/v1.2/ (6 phase_X_review + 6 preflight + kickoff + plan + design_review + verification + audit_plan + carry_summary)
- Working tree audit G5
```

---

## 10. Invariants pre-launch S22

| Wire format | Version S22 entrée | Version S22 sortie | Note |
|---|---|---|---|
| `BLOB_VERSION` | `0x01` | `0x01` | Inchangé |
| `TASK_RESPONSE_VERSION` | `1` | `1` | Inchangé |
| `CANARY_VERSION` | `1` | `1` | Inchangé |
| `ANNOUNCEMENT_VERSION` | `1` | `1` | Inchangé |
| `CURATOR_LIST_VERSION` | `1` | `1` | Inchangé (extend verify param, format stable) |
| `PROVENANCE_VERSION` | `1` | `1` | Inchangé (extend via predicate sidecar) |
| **`AGE_WITNESS_VERSION`** | — | **`1` (nouveau)** | Pre-launch stable |
| **`CONTRIBUTOR_ATTESTATION_VERSION`** | — | **`1` (nouveau)** | Pre-launch stable |
| **`DELEGATION_CERT_VERSION`** | — | **`1` (RFC design-only)** | Pas de code S22 |

Aucun tolerant decoder multi-version introduit. Pré-launch policy
respectée.

---

## 11. Tests projection §11

| Suite | Baseline S22 entrée | Delta projeté | Total Phase F |
|---|---|---|---|
| Rust workspace nextest | 659 | **+28** (Phase A +7, Phase C +16, Phase D +5) | **687** |
| Python SDK | 185 | 0 | 185 |
| Python coordinator | 249+3 skipped | **+9** (Phase C +4, Phase E +5) | **258+3 skipped** |
| Python app-gov | 46 | 0 | 46 |
| Vitest unit | 256 | **+6** (Phase B +6) | **262** |
| Playwright | 38 | 0 (drift documenté S23 carry) | 38 |
| size-limit | 7/7 | 0 | 7/7 |
| SPDX license hook | 246+ | 0 | 246+ |
| **Total** | **~1436** | **+43** | **~1479** |

---

## 12. Carry summary + cap G7

Cf. `sprint22_carry_summary.md` (émis Phase F) :
- **Cap G7** : 1/2 slot (T-NN+2 iframe Rust-wasm hors cap PATTERNS §P34).
- **LT-2 Meta-1 Radicle** reclassification régularisée kickoff S22.
- **Wire-up debts absorbés** (pas carry-overs G7) : P2-S21-1/2/3/6
  Phase A/B.
- **Process fixes** : P2-S21-4 + P2-S21-5 Phase F.
- **Items deferrés S23** (chore planning distinct, pas carry G7) :
  redundancy voting 3-worker majority (+ re-plomb dep §3 S24).
- **Items deferrés post-S25** : sandbox tool-calling allow-list.

---

## 13. Risk register

Cf. kickoff §6. R-S22-1 à R-S22-6 couverts par preflight G8 phase-
by-phase + tests intégration + doc scope explicite.

---

## 14. Cohérence §7.4 verification suites

Avant chaque commit Phase (A-F) :

```bash
# Rust (cible crate + workspace final)
cargo nextest run -p <crate-touche> --locked
cargo nextest run --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
cargo test --workspace --locked --doc

# Python coord
uv run ruff format --check packages/ && uv run ruff check packages/
uv run pytest packages/nexus-coordinator/tests/ -q
uv run pytest packages/nexus-sdk/tests/ -q
uv run pytest packages/nexus-app-gov/tests/ -q

# Frontend (Phase B cible)
cd web && npm run lint
npx tsc --noEmit -p tsconfig.app.json
npm run test:unit
npm run build
npm run size
bash scripts/scan-en-strings.sh

# Wheel PyO3 après Phase C changes (nexus-core-py bindings)
unset CONDA_PREFIX CONDA_DEFAULT_ENV && \
  VIRTUAL_ENV=$PWD/.venv maturin develop --release \
    --manifest-path crates/nexus-core-py/Cargo.toml
```

Hook pre-commit phase-auditor-gate.sh + phase-review-cross-check
GHA wire Phase F.

---

**Fin plan S22**. Chaque Phase est gated par preflight G8 + review
nexus-phase-review + audit post-commit. Verdict G8 par phase
attendu §6.2 Phase C + §4-8 autres phases.
