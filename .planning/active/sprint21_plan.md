# Sprint 21 — Plan détaillé (rate-limit + PII SDK defense-in-depth + output filter + quarantine queue)

**Écrit** : 2026-04-18 (session fraîche post-audit gate S20
`66a3a7c`).
**Tip master d'entrée** : `66a3a7c` (chore(sprint20): audit gate
S20 — findings verdict PASS).
**Source D1..D5 gelées** : `sprint21_kickoff.md §4`.
**Range audit-plan cible Phase F** : commits S21 ouverture à
Phase F wrap-up.

---

## 1. État vérifié à l'entrée

### 1.1 Tip master + tests

- HEAD : `66a3a7c chore(sprint20): audit gate S20 — findings
  (verdict PASS, no blocking fix)` (2026-04-18).
- Range S20 audité : `3a7f0a3..131f32b` (26 commits), verdict
  PASS, 0 P0/P1.
- Compteurs tests entrée S21 (réplication `sprint20_verification.
  md §2`, Rust re-joué par auditeur S20 : 642 pass 0 skip, 11.5 s) :

| Suite | Count observé |
|---|---|
| Rust workspace nextest | **642** (re-run auditeur 2026-04-18) |
| Python SDK | 185 |
| Python coordinator | 213 + 3 skipped |
| Python app-gov | 46 |
| Vitest unit | 241 |
| Playwright | 38 |
| size-limit | 7/7 |
| SPDX | 246+ |
| **Total** | **~1371** |

### 1.2 Clippy + size + linters

- `cargo clippy --workspace --all-targets -- -D warnings` : 0
  warning (vérifié S20 Phase F `sprint20_verification.md §3 row
  4`).
- `cargo fmt --all --check` : 0 diff (Phase E fmt residual
  `1ad2def` puis Phase F).
- Frontend `npm run lint` + `npx tsc --noEmit` + `npm run build` +
  `npm run size` : tous verts S20 fin.
- Python `ruff format --check` + `ruff check` : tous verts S20
  fin.

### 1.3 Audit gate S20 leveraged

- Meta-1 Radicle-v1.0 re-carry confirmé (`sprint21_carry_summary.
  md §1 C-1`).
- C-PLAN-1 plan docs fix re-carry confirmé (`sprint21_carry_
  summary.md §1 C-2`).
- 3 tech debt hors cap tracés (`sprint21_carry_summary.md §2`).

### 1.4 Pre-launch protocol policy

- `BLOB_VERSION = 0x01`, `TASK_RESPONSE_VERSION = 1`, `CANARY_
  VERSION = 1`, `ANNOUNCEMENT_VERSION = 1` tous inchangés S21.
- Aucun bump version prévu S21 (scope S21 touche uniquement
  runtime behaviors + nouveaux modules SDK + config files locaux,
  pas de wire format existant).

---

## 2. Décisions Day 0 gelées (rappel kickoff §4)

| # | Décision | Cœur technique |
|---|---|---|
| D1 | Rate-limit | `governor 0.10.2` GCRA + `tower-governor 0.8` axum + DashMap keyed + policy.toml hot-reload + budgets tier per-(consumer, worker, model) |
| D2 | PII SDK defense-in-depth | iframe = `onnxruntime-web 1.24.3` + `@huggingface/transformers` tokenizer + `knowledgator/gliner-pii-edge-v1.0` (backbone Phase B G8 S1) + regex fallback / coord = `presidio-analyzer 2.2.362` + `GLiNERRecognizer` même modèle ONNX |
| D3 | Output filter | `LLM Guard 0.3.16` `InvisibleText` + EED prompt echo (Levenshtein Sim > 0.85) + SM exact fallback |
| D4 | Quarantine queue | SQLite WAL local + schema `quarantine_messages` + TTL 15 min + REST `/quarantine/*` + CLI `sbfb quarantine list/flush/drop` + pattern S19 reuse |
| D5 | Cap G7 2/2 | Meta-1 Radicle + C-PLAN-1 plan docs fix ; tech debt T-NN/T-NN+1/T-NN+2 hors cap |

---

## 3. Research consulté (pré-gel, listé pour Phase 0 audit S22)

### Rate-limit Rust 2026 (D1)

- `governor 0.10.2` crates.io (2025-11-13, MIT, GCRA, DashMap keyed)
- `tower-governor 0.8` github.com/benwis/tower-governor (axum 0.8)
- `tokio-rate-limit 0.8` crates.io (2025-11, lock-free per-key)
- RUSTSEC advisories 2025 : aucun report sur governor/leaky-bucket/
  tokio-rate-limit (best-effort search, pas exhaustif)

### PII SDK (D2)

- `onnxruntime-web 1.24.3` npm (Microsoft 2026-03, WASM SIMD+threads)
- `presidio-analyzer 2.2.362` PyPI (Microsoft MIT 2026-03-15)
- `@huggingface/transformers` v4 npm (2026-02, tokenizer)
- `knowledgator/gliner-pii-edge-v1.0` HF (Apache-2.0, 2024-01-29,
  F1 0.755, backbone à confirmer Phase B)
- Rejet Rust-first : `tract 0.22.1` opset 9-18 vs GLiNER opset 19,
  `wasm32-unknown-unknown` non documenté ; `gline-rs v1.0.1`
  (2026-01) a choisi `ort` pas tract
- Rejet `GLiNER.js` : inactif mars 2025
- Rejet `redact-core` : 2026-02 solo maintainer
- Rejet `Pyodide + Presidio iframe` : 500 MB overhead
- Rejet `Gretel gliner-bi-small` : 349 MB iframe trop lourd
  (candidat coord-side)

### Output filter (D3)

- `LLM Guard 0.3.16` ProtectAI MIT (PyPI, `InvisibleText` scanner)
- PLeak CCS'24 arXiv 2405.06823 (EED/SM/EM métriques)
- ProxyPrompt arXiv 2505.11459 (mai 2025, defense proactive hors
  scope S21)
- OWASP LLM Top 10 2025 #7 System Prompt Leakage
- Rejet NeMo Guardrails 0.11+ Python 3.8 dropped
- Rejet guardrails-ai LLM secondaire scoring

### Quarantine queue (D4)

- libp2p gossipsub v1.1 spec (graylist threshold automatique)
- SQLite WAL docs.sqlite.org
- Pattern S19 Phase D `f238d31` upload_queue.py
- Pas d'équivalent public P2P hold 15 min + manual flush

### Frontmatter HARDENING_ROADMAP update

Entrée `audited_findings` 2026-04-18 ajoutée dans ce commit
d'ouverture S21 (requalification D2 SDK).

---

## 4. Phase A — Rate-limit sliding-window multi-tier

### 4.0 Pré-requis chore hors-sprint (G8 Option C arbitré)

G8 preflight 2026-04-18 (`sprint21_phase_A_pivot_proposal.md`
verdict DESIGN-CONFLICT → user arbitre Option C) : **avant la 1re
ligne de code Phase A**, commit chore hors-sprint 
`chore: bump axum 0.7 → 0.8 workspace-wide` pour résoudre le clash
tower-governor 0.8 (requiert axum 0.8) vs workspace axum 0.7.

Scope concret audit pre-chore :
- `Cargo.toml` ligne 139 : `axum = "0.7"` → `"0.8"` (workspace dep
  inherited par `nexus-shell-daemon` + `nexus-launcher`).
- `crates/nexus-shell-daemon/src/http.rs:180` : `/:hash/*path` →
  `/{hash}/{*path}` (axum 0.8 path syntax).
- `crates/nexus-shell-daemon/src/http.rs:200` : `/curators/:pubkey`
  → `/curators/{pubkey}`.
- `crates/nexus-shell-daemon/src/http.rs:168,784` + `crates/nexus-
  shell-daemon-core/src/browse.rs:183` : doc comments sync syntax.
- Middleware `middleware::from_fn` + `from_fn_with_state` +
  `Next` signatures : ajustements compile-driven (axum 0.8
  `Request` / `Next` API mineurement different).
- Tests S16 loopback suites à re-valider (`auth.rs` test modules
  + `uds_server.rs` + `named_pipe_server.rs`) : bearer + Host +
  Origin + UDS/NP peer creds **invariants préservés**, rework =
  syntax only.

Aucun de : `PathParamsRejection` / `hyper::Body` / `axum::async_
trait` / `Option<Extractor>` / `into_make_service_with_connect_
info` n'est utilisé dans le workspace → breakage grid axum 0.8
bien plus réduit que proposal initial §3 Option C estimait.

Dépendances workspace déjà compat axum 0.8 : `tower = "0.5"` (ligne
140) + `tower-http = "0.6"` (ligne 141) + `hyper = "1"` (ligne 149).
Pas de bump transversal nécessaire.

### 4.1 Fichiers ajoutés / modifiés

- **`crates/nexus-worker-core/src/rate_limit.rs`** (nouveau) :
  ```
  pub struct RateLimiter {
      inner: governor::DefaultKeyedRateLimiter<RateKey>,
      evictor: tokio::task::JoinHandle<()>,
  }
  pub struct RateKey { consumer: ConsumerId, worker: WorkerId,
                      model: ModelId }
  impl RateLimiter {
      pub fn new(policy: RateLimitPolicy) -> Self { ... }
      pub fn check(&self, key: &RateKey) -> Result<(),
                   GovernorError> { ... }
      async fn evict_loop(interval_secs: u64, ...) { ... }
  }
  ```
- **`crates/nexus-worker-core/src/rate_limit_policy_loader.rs`**
  (nouveau) : pattern hot-reload `~/.sbfb/rate_limit_policy.toml`
  + `notify` file-watcher (50 ms debounce) + `malformed-reload-
  guard` + `file-deletion-guard` (cohérent S20 Phase C
  `pow_policy_loader.rs` + S18 D-1 `TokenRotator`). **Post-G8 R1
  scope-cut** : le loader vit worker-core (pas shell-daemon) car
  le consumer final de la policy est l'engine worker (consent
  pattern pré-existant `worker-core/src/consent/mod.rs`).
- **~~`crates/nexus-shell-daemon/src/http.rs`~~** : **scope-cut
  R1 2026-04-19**. Plan initial mentionnait middleware
  `tower-governor` sur `/task/submit` mais cet endpoint vit côté
  Python FastAPI (`packages/nexus-coordinator/src/nexus_
  coordinator/api/tasks.py::POST /tasks/submit`, depuis Sprint 4
  Phase A), **pas** côté Rust axum. `tower-governor` ne peut pas
  middleware FastAPI. R1 arbitré user 2026-04-19 : **worker-
  engine gate pure Rust**, pas de middleware HTTP. HTTP middleware
  sera ré-évalué S22+ au niveau coord Python (`slowapi` ou
  équivalent) dans un sprint dédié sécurité API.
- **`Cargo.toml` workspace** (modifié) : deps
  `governor = "0.10.2"` + `nonzero_ext = "0.3"` (requis par
  `governor::Quota::per_second(nonzero!(..))`). `tower-governor =
  "0.8"` **non-ajouté R1** — scope-cut middleware HTTP. `notify
  = "6"` (déjà workspace) + `toml = "0.8"` (déjà).
- **`crates/nexus-worker-core/Cargo.toml`** (modifié) : deps
  `governor = { workspace = true }` + `nonzero_ext = { workspace =
  true }`.
- **`crates/nexus-worker-core/configs/rate_limit_policy.toml.
  sample`** (nouveau) : template default budgets tier +
  overrides (pattern S20 `relay_pow_policy.toml.sample`).
- **Tests** : `crates/nexus-worker-core/src/rate_limit.rs#tests`
  (unit, saturation/per-tuple/eviction/override). Pas de tests
  HTTP intégration R1 (scope-cut).

### 4.2 Tests à écrire (R1 scope-cut post-G8 drift detection)

Unit tests `rate_limit.rs` :
1. `rate_limit::saturation_rejects_over_budget` : saturer un tuple
   consumer/worker/model à 100 req/min, vérifier `governor` retourne
   `NotUntil` après N tokens bucket.
2. `rate_limit::per_tuple_independence` : 3 tuples distincts,
   vérifier saturation d'un n'affecte pas les 2 autres.
3. `rate_limit::eviction_after_quiet_period` : après silence,
   `retain_recent()` supprime les clés silencieuses de DashMap.
4. `rate_limit::override_consumer_whitelist` : override policy
   par-consumer pubkey remplace default budget.

Unit tests `rate_limit_policy_loader.rs` :
5. `spawn_missing_file_uses_default_policy` : pas de fichier →
   default budgets.
6. `spawn_existing_file_loads_override` : fichier valide →
   override appliqué.
7. `spawn_malformed_toml_fails_loud_at_boot` : TOML corrompu au
   boot → erreur explicite.
8. `policy_hot_reload_live` : modifier toml runtime, vérifier
   nouvelle limit appliquée sans restart worker.
9. `malformed_reload_keeps_previous_policy` : TOML corrompu
   runtime → policy précédente conservée, warn log, pas panic.
10. `removal_keeps_previous_policy` : fichier supprimé runtime →
    policy précédente conservée (pattern S20 pow_policy_loader).

**+10 tests Rust attendus** (delta original +15 → +10 via scope-
cut R1 : tests HTTP row 7-8 original drop, ajout de 3 tests
policy_loader symétriques pow_policy_loader pour couverture
complète). Test row 10 original « PATTERNS §P33 » reste couvert
par l'update PATTERNS.md hors-tests.

Tests HTTP différés S22+ scope-cut :
- `http::task_submit_429_on_rate_limit` (R1 drop — endpoint
  Python FastAPI, middleware Python dédié au sprint S22+).
- `http::rate_limit_middleware_order_before_pow_gate` (R1 drop
  — pas de middleware HTTP Rust dans R1).

### 4.3 Critère d'acceptation Phase A (R1)

- `cargo nextest run -p nexus-worker-core --locked` vert (incluant
  nouveaux tests rate-limit + policy_loader).
- `cargo nextest run --workspace --locked` vert ≥ 652 tests
  (baseline 642 + 10 Phase A).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
  0 warning.
- `cargo fmt --all --check` 0 diff.
- `cargo test --workspace --locked --doc` vert.
- **Test manuel HTTP 429 retiré** (R1 scope-cut, pas de middleware
  HTTP Phase A).

### 4.4 Commit cible Phase A

```
feat(sprint21): Phase A — rate-limit sliding-window multi-tier per-(consumer, worker, model) via governor GCRA worker-engine gate R1

Body riche avec :
- Delta tests +10 Rust (R1 scope-cut : 15 → 10, drop HTTP 429
  row 7-8, ajout 3 tests policy_loader symétriques pow_policy_
  loader S20 C)
- Scope cuts respectés (kudos-weighted admission, redundancy
  voting, HTTP middleware /task/submit S22+ dédié, etc.)
- Working tree audit G5 (PHASE / CRAFT / DEBT / NOISE)
- G8 pre-flight Phase A (commit `60adceb`) DESIGN-CONFLICT Option
  C arbitré + axum bump chore hors-sprint (commit `5e67ce0`)
- Drift Phase A §4.1 /task/submit (Python FastAPI, pas Rust axum)
  identifié mid-phase 2026-04-19 → R1 worker-engine gate arbitré
  user 2026-04-19 (scope-cut HTTP middleware S22+)
- Référence : `sprint21_phase_A_pivot_proposal.md` (Option C +
  R1 scope-cut inline §3 Day-0 Drift post-pivot)
```

---

## 5. Phase B — Client-side PII redaction SDK iframe

### 5.1 Pré-requis G8 S1 scan OBLIGATOIRE (avant 1re ligne de code)

Invoquer skill `nexus-phase-preflight` — scans S1 + S2 + S3 + S4.

**Scan S1 spécifique obligatoire** : fetch fresh via WebFetch +
context7 MCP `query-docs` pour figer :

- `knowledgator/gliner-pii-edge-v1.0` model card complete
  (backbone architecture, params count, tokenizer type)
- `config.json` du repo HF (architectures field, hidden_size,
  num_hidden_layers)
- ONNX opset version of the quantized int8 export
- Size exact du fichier ONNX quantized
- `onnxruntime-web 1.24.3` opset support (expected opset 19 OK but
  verify for DisentangledSelfAttention if DeBERTa-v3)

Output : `.planning/active/sprint21_phase_B_preflight.md`. Verdict :
- **EXECUTE plan-as-is** : edge OK + onnxruntime-web support opset
  confirmé.
- **SCOPE-CUT-CONSISTENT** : edge trop faible F1 0.755 ou taille
  ONNX > 80 MB → switch vers `onnx-community/gliner_multi_pii-v1`
  quantized (349 MB) OU `gliner_small-v2.1` onnx-community.
- **DESIGN-CONFLICT** : opset non supporté par onnxruntime-web →
  arbitrage user Option A (modèle alternative) / Option B (tract
  side-step) / Option C (defer S22).

### 5.2 Design doc pre-Phase-B

`.planning/research/S21_phase_B_iframe_pii_sdk_design.md`
(pattern S20 Phase B/D design docs `.planning/research/`).
Couvre :
- Backbone modèle retenu post-scan S1
- Architecture wrapper JS (tokenize → inference → classify →
  redact)
- Regex fallback curated (which PII types)
- Integration `sbfb-bridge.js` méthode `pii_redact`
- CSP iframe requirements (`wasm-unsafe-eval` directive, SAB cross-
  origin isolation si threads requis)
- Bundle size target + loading strategy (on-demand vs eager)

### 5.3 Fichiers ajoutés / modifiés

- **`web/src/sdk/pii/index.ts`** (nouveau) : exports publics
  `detect` + `redact` + `configure`.
- **`web/src/sdk/pii/wrapper.ts`** (nouveau) : tokenize → ONNX
  inference → span classifier → redact.
- **`web/src/sdk/pii/fallback.ts`** (nouveau) : regex curated
  (email, phone E.164, credit card Luhn, SSN US, IBAN).
- **`web/src/sdk/pii/policy.ts`** (nouveau) : charge config
  `pii_redaction_policy` via postMessage bridge.
- **`web/src/lib/sbfb-bridge.js`** (modifié) : ajout méthode
  whitelist `pii_redact(text, policy)` + correlation ID pattern
  S13.
- **`web/src/sdk/pii/__tests__/`** (nouveau) : Vitest unit tests
  + Playwright iframe réels tests.
- **`web/package.json`** (modifié) : deps
  `"onnxruntime-web": "1.24.3"`, `"@huggingface/transformers":
  "4.x"`.
- **`web/public/models/`** (nouveau, gitignore) : modèle ONNX
  téléchargé on-demand runtime (pas committé — trop lourd). Note
  `.gitignore` pattern ajouté.

### 5.4 Tests à écrire

1. `wrapper.ts` detect email/phone/CC/SSN simple cases.
2. `wrapper.ts` redact replacement policy (confidence threshold
   0.5 default).
3. `fallback.ts` regex engagé si model load fails.
4. `policy.ts` hot-reload live policy runtime.
5. Playwright : iframe charge modèle successfully (ou fallback
   trigger si CSP blocker).
6. Playwright : postMessage `pii_redact(text)` retourne redacted
   text.
7. Playwright : scenario multi-PII in single text (email + phone
   + name).
8. Playwright : no-redaction policy (disabled) pass-through.
9. Vitest : fallback regex false-positive rate vs curated
   positives.
10. Playwright : model load error → fallback trigger + warn log
    visible.

**+10 Vitest + 5 Playwright = +15 tests frontend**.

### 5.5 Critère d'acceptation Phase B

- `cd web && npm run lint && npx tsc --noEmit && npm run test:unit
  && npm run test:coverage && npm run build && npm run size &&
  npx playwright test && bash scripts/scan-en-strings.sh` tous
  verts.
- Bundle size iframe inchangé CI (ou acceptable augmentation si
  modèle ONNX lazy-loaded).
- `sprint21_phase_B_preflight.md` présent verdict EXECUTE ou
  SCOPE-CUT-CONSISTENT.

### 5.6 Commit cible Phase B

```
feat(sprint21): Phase B — client-side PII redaction SDK iframe (onnxruntime-web + GLiNER PII edge)

Body riche avec :
- Delta tests (+10 Vitest +5 Playwright)
- Scope cuts respectés
- Working tree audit G5
- G8 preflight verdict S1 scan backbone model confirmé
- Drift HARDENING_ROADMAP §audited_findings closed
```

---

## 6. Phase C — Coord-side PII redaction + output filter

### 6.1 Design doc pre-Phase-C

`.planning/research/S21_phase_C_output_filter_design.md`. Couvre :
- EED (Extended Edit Distance) methodology + seuil 0.85 empirical
  tuning corpus
- Presidio + GLiNERRecognizer integration path
- Hook pre-dispatch worker ordering (rate-limit → PoW → PII redact
  → dispatch)
- Policy `~/.sbfb/output_filter_policy.toml` schema
- LLM Guard `InvisibleText` whitelist `Cf` (RLO/LRO i18n)

### 6.2 Fichiers ajoutés / modifiés

- **`packages/nexus-coordinator/src/nexus_coordinator/
  pii_redactor.py`** (nouveau) : `class PiiRedactor(presidio +
  GLiNER)`.
- **`packages/nexus-coordinator/src/nexus_coordinator/
  output_filter.py`** (nouveau) : `class OutputFilter(LLM Guard
  InvisibleText + EED echo detector)`.
- **`packages/nexus-coordinator/src/nexus_coordinator/
  task_response_validator.py`** (modifié) : ajout hook
  `output_filter.filter()` avant `validate_task_response`.
- **`packages/nexus-coordinator/pyproject.toml`** (modifié) :
  deps `presidio-analyzer = {version = "2.2.362", extras =
  ["gliner"]}`, `presidio-anonymizer = "2.2.362"`,
  `llm-guard = "0.3.16"`, `rapidfuzz = "3.x"` (EED Levenshtein
  optimisé).
- **`~/.sbfb/pii_redaction_policy.toml.sample`** (nouveau).
- **`~/.sbfb/output_filter_policy.toml.sample`** (nouveau).
- **Tests** : `packages/nexus-coordinator/tests/test_pii_
  redactor.py` + `test_output_filter.py`.

### 6.3 Tests à écrire

1. `test_pii_redactor.py::test_redact_email_phone_name` : entités
   basic détectées + anonymized.
2. `test_pii_redactor.py::test_redact_gate2_apps_strict_mode` :
   policy override gate2_apps = confidence_threshold 0.3, tout
   redact.
3. `test_pii_redactor.py::test_policy_hot_reload` : modifier
   policy.toml runtime, reload appliqué.
4. `test_output_filter.py::test_invisible_chars_stripped` :
   zero-width U+200B, PUA U+E000, Tag chars U+E0020 tous strippés.
5. `test_output_filter.py::test_rlo_lro_whitelisted_for_i18n` :
   U+202E RLO conservé (Arabe/Hébreu légitime).
6. `test_output_filter.py::test_prompt_echo_exact_match_blocks` :
   system_prompt substring match exact → task response rejetée.
7. `test_output_filter.py::test_prompt_echo_eed_similarity_above_
   0_85_blocks` : reconstruction partielle similaire 0.9 EED →
   bloquée.
8. `test_output_filter.py::test_prompt_echo_eed_similarity_below_
   0_85_passes` : reconstruction distincte 0.3 EED → passe.
9. `test_output_filter.py::test_pleak_attack_reconstruction_
   scenarios` : simuler 5 PLeak-style attacks, tous détectés.
10. `test_output_filter.py::test_benign_output_passes_through` :
    response normale utilisateur, pas de false positive.

**+10 Python coord tests**.

### 6.4 Critère d'acceptation Phase C

- `uv run pytest packages/nexus-coordinator/tests/ -q` vert
  incluant nouveaux tests.
- `uv run ruff format --check packages/` + `uv run ruff check
  packages/` verts.
- Design doc présent + G8 preflight Phase C verdict CLEAN.

### 6.5 Commit cible Phase C

```
feat(sprint21): Phase C — coord-side PII redaction + output filter (Presidio GLiNER + LLM Guard + EED echo)
```

---

## 7. Phase D — Quarantine queue SQLite WAL + CLI

### 7.1 Design doc pre-Phase-D

`.planning/research/S21_phase_D_quarantine_design.md`. Couvre :
- Schema SQLite evolution
- TTL clock semantics (received_at vs inserted_at, NTP sync)
- Security manual flush (bearer X-SBFB-Token + Host + Origin
  pattern S16)
- Interaction gossip layer (PoW gate at flush time vs pre-hold)
- Expected cardinality + benchmarks (~1000 msg/min/15min TTL
  estimate)

### 7.2 Fichiers ajoutés / modifiés

- **`packages/nexus-coordinator/src/nexus_coordinator/
  quarantine_queue.py`** (nouveau) : `class QuarantineQueue`
  SQLite WAL + tokio task TTL + REST handlers.
- **`crates/nexus-shell-daemon/src/api/quarantine.rs`** (nouveau) :
  REST routes `/quarantine/list|flush|drop` avec auth bearer
  pattern S16 + proxy vers coord Python si coord responsable.
- **`crates/nexus-launcher/src/cli.rs`** (modifié) : sous-commande
  `sbfb quarantine list|flush|drop`.
- **`~/.sbfb/quarantine.db`** (créé runtime, gitignore) : SQLite
  WAL.
- **Tests** : `packages/nexus-coordinator/tests/test_quarantine_
  queue.py` + `crates/nexus-shell-daemon/tests/quarantine_
  integration.rs`.

### 7.3 Tests à écrire

1. `test_quarantine_queue.py::test_add_then_list_returns_entry` :
   add + list round-trip.
2. `test_quarantine_queue.py::test_ttl_15min_auto_drop` : mock
   clock +900s → entry auto-removed.
3. `test_quarantine_queue.py::test_manual_flush_accept_sends_to_
   gossip` : flush flush_status='flushed' + gossip broadcast.
4. `test_quarantine_queue.py::test_manual_drop_sets_status` :
   drop flush_status='dropped', no gossip broadcast.
5. `test_quarantine_queue.py::test_cardinality_10k_entries_no_
   panic` : bulk insert 10k then TTL cleanup.
6. `quarantine_integration.rs::test_bearer_auth_required` :
   `/quarantine/*` sans bearer → 401.
7. `quarantine_integration.rs::test_host_origin_check` : wrong
   origin → 403 (pattern S16).
8. CLI test : `sbfb quarantine list --json` retourne JSON
   validé schema.

**+5 Python coord tests + 3 Rust tests = +8 tests**.

### 7.4 Critère d'acceptation Phase D

- Tests verts.
- CLI smoke test : `sbfb quarantine list` fonctionne loopback.
- Design doc présent.

### 7.5 Commit cible Phase D

```
feat(sprint21): Phase D — quarantine queue SQLite WAL + manual flush CLI
```

---

## 8. Phase E — Tech debt batch S20 carry (optionnel)

### 8.1 Contenu (3 items optionnels + PATTERNS.md update)

**E-1 — Canary JCS envelope migration** (tech debt T-NN S20 audit
P2-E-1) :
- Migrer `crates/nexus-shell-daemon-core/src/canary/mod.rs`
  `canary_wire_bytes` de `serde_json::to_vec` vers
  `serde_jcs::to_vec`.
- Test non-régression : snapshot cross-language Python ↔ Rust
  sur enveloppe canary.

**E-2 — CanaryRegistry verify Ed25519 at ingest** (tech debt
T-NN+1 S20 audit P2-E-2) :
- Modifier `packages/nexus-coordinator/src/nexus_coordinator/
  canary_registry.py` `POST /api/canary/observed` handler pour
  verify Ed25519 signature at ingest via `nexus-core-py`
  `verify_canary` binding.
- Décision maturité pre-launch : hardening avant v1.0 go-live
  (T2+) vs acceptable observational-only beta T0-T1. Discussion
  design doc inline body commit.
- Test non-régression : `CanaryObservation` avec signature
  malformée → 401.

**E-3 — C-PLAN-1 plan docs fix wire-point divergence** :
- Edit `.planning/archive/v1.2/sprint20_plan.md §6.2 + §6.4` :
  note en tête du plan indiquant correction wire-point
  `runtime.rs::spawn_gossip_subscribe_task` au lieu de
  `iroh_runtime.rs::GossipClient::subscribe`.

**E-4 — Update PATTERNS.md tech debt entries** :
- Ajouter T-NN canary JCS (résolu si Phase E livre).
- Ajouter T-NN+1 registry verify Ed25519 (résolu si Phase E livre).
- Ajouter T-NN+2 iframe Rust-wasm realignement Option G (S22+
  blocked). Cette entrée reste ouverte tant que blockers non
  levés.

### 8.2 Tests à écrire

1. `canary::test_wire_bytes_is_jcs_canonical_cross_language` :
   Python sign + Rust verify, byte-identique.
2. `test_canary_registry.py::test_observed_endpoint_rejects_
   malformed_signature` : Ed25519 verify at ingest.
3. `test_canary_registry.py::test_observed_endpoint_accepts_
   valid_canary` : happy path.

**+3 tests (1 Rust + 2 Python coord)**.

### 8.3 Critère d'acceptation Phase E

- Tests verts.
- PATTERNS.md tech debt section mise à jour.
- Plan S20 archivé corrigé.

### 8.4 Commit cible Phase E

```
feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix)
```

**Note** : Phase E peut être split en 3 chore commits distincts
(`chore(sprint21): tech-debt canary JCS`, `chore(sprint21): tech-
debt registry verify`, `chore(sprint21): fix S20 plan §6.2 wire-
point`) au choix de l'executeur. Si split, Phase E devient
Phase E.1/E.2/E.3.

---

## 9. Phase F — Consolidation + verification + audit plan S22

### 9.1 Livrables Phase F

- **`sprint21_verification.md`** : self-report fail-fast (source
  §10 ci-dessous).
- **`sprint21_audit_plan.md`** : plan audit gate pour S22 Phase 0
  (7+ tracks A-F + meta-track Radicle + dimension G8 retrospective
  si pivot Phase B ou C).
- **Update** `CLAUDE.md §État actuel` + `docs/claude/SPRINT_LOG.md`
  row S21 + memory `nexus_grid_pivot.md` fusion §5 Findings
  carry-over + `MEMORY.md` hook.
- **Update HARDENING_ROADMAP.md** : `last_validated: 2026-XX-XX`
  + §3 S21 résumé livré (cohérent avec `audited_findings`
  frontmatter).
- **Migration PARA** : tous `sprint21_*.md` + `sprint20_audit_
  findings.md` → `archive/v1.2/`.

### 9.2 Critère d'acceptation Phase F

- `.planning/active/` vide post-commit.
- Tous docs migrés dans `archive/v1.2/`.
- Memory tip sync HEAD Phase F.

### 9.3 Commit cible Phase F

```
chore(sprint21): Phase F — wrap-up + verification + audit plan S22 + migrate planning
```

---

## 10. Fail-fast checklist (30 rows cible)

Structure `| # | Check | Commande | Critère | Observed |` remplie
en Phase F verification.md.

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | Tip Phase F SHA | `git rev-parse --short HEAD` | 7-char SHA résolu |
| 2 | Rust workspace nextest | `cargo nextest run --workspace --locked` | `> 685` passed, 0 skip |
| 3 | Rust doctests | `cargo test --workspace --locked --doc` | 0 failed |
| 4 | Rust clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning |
| 5 | Rust fmt | `cargo fmt --all --check` | 0 diff |
| 6 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | 185 passed |
| 7 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | `> 228` passed |
| 8 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 passed |
| 9 | Python ruff format | `uv run ruff format --check packages/` | 0 diff |
| 10 | Python ruff check | `uv run ruff check packages/` | 0 warning |
| 11 | Frontend lint | `cd web && npm run lint` | 0 error |
| 12 | Frontend tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error |
| 13 | Frontend Vitest | `npm run test:unit` | `> 251` passed |
| 14 | Frontend build | `npm run build` | success |
| 15 | Frontend size | `npm run size` | 7/7 pass |
| 16 | Playwright | `npx playwright test` | `> 43` passed |
| 17 | Frontend strings FR | `bash scripts/scan-en-strings.sh` | 0 unexpected EN |
| 18 | SPDX license hook | pre-commit SPDX | `>= 248`/all checked |
| 19 | Phase A rate-limit module | `ls crates/nexus-worker-core/src/rate_limit.rs` | exists |
| 20 | Phase A governor dep | `grep "governor" Cargo.toml` | `= "0.10.2"` |
| 21 | Phase A rate_limit_policy.toml sample | `ls crates/nexus-shell-daemon/configs/rate_limit_policy.toml.sample` | exists |
| 22 | Phase B iframe SDK dir | `ls web/src/sdk/pii/` | exists with index.ts + wrapper.ts + fallback.ts |
| 23 | Phase B onnxruntime-web dep | `grep "onnxruntime-web" web/package.json` | `= "1.24.3"` |
| 24 | Phase B preflight.md | `ls .planning/archive/v1.2/sprint21_phase_B_preflight.md` | exists with verdict |
| 25 | Phase C pii_redactor.py | `ls packages/nexus-coordinator/src/nexus_coordinator/pii_redactor.py` | exists |
| 26 | Phase C output_filter.py | `ls packages/nexus-coordinator/src/nexus_coordinator/output_filter.py` | exists |
| 27 | Phase C presidio dep | `grep "presidio-analyzer" packages/nexus-coordinator/pyproject.toml` | `= "2.2.362"` |
| 28 | Phase D quarantine_queue.py | `ls packages/nexus-coordinator/src/nexus_coordinator/quarantine_queue.py` | exists |
| 29 | Phase D CLI cmd | `sbfb quarantine list --help` | `sbfb` exits 0 |
| 30 | Phase E tech debt T-NN PATTERNS | `grep "T-NN" docs/rust/PATTERNS.md` | entries present |
| 31 | Meta-1 Radicle-v1.0 re-carry S22 | `grep "Meta-1" .planning/active/sprint21_audit_plan.md` | explicit re-carry |
| 32 | Memory tip sync | `grep "Tip \`" memory/nexus_grid_pivot.md \| head -1` | match HEAD Phase F |

**32 rows fail-fast** — pattern S20 32 rows cohérent.

---

## 11. Git plan (commits atomiques attendus)

```
1. chore(planning): open Sprint 21 — rate-limit + PII SDK defense-in-depth + output filter + quarantine queue
   (ce commit) — 4 docs planning active/ + migration findings S20 + HARDENING_ROADMAP update + delete NOISE

2. (optionnel) chore(sprint21): fix S20 plan §6.2 wire-point divergence post-audit
   OU intégré Phase E

3. feat(sprint21): Phase A — rate-limit sliding-window multi-tier per-(consumer, worker, model) via governor GCRA
   (+15 Rust tests)

4. feat(sprint21): Phase B — client-side PII redaction SDK iframe (onnxruntime-web + GLiNER PII edge)
   (+15 frontend tests, G8 preflight S1 verdict)

5. feat(sprint21): Phase C — coord-side PII redaction + output filter (Presidio GLiNER + LLM Guard + EED echo)
   (+10 Python coord tests)

6. feat(sprint21): Phase D — quarantine queue SQLite WAL + manual flush CLI
   (+8 Python+Rust tests)

7. feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix)
   (+3 tests, Phase E peut être split en 3 chores distincts si préférence)

8. chore(sprint21): Phase F — wrap-up + verification + audit plan S22 + migrate planning
   (docs only, +0 tests)
```

**Total Rust tests attendus** : +15 (A) + 3 (D) + 1 (E canary) =
**+19 Rust** (642 → 661).
**Total Python tests attendus** : +10 (C) + 5 (D) + 2 (E) = **+17
Python coord** (213 → 230).
**Total Vitest/Playwright** : +10 Vitest + 5 Playwright = **+15
frontend** (241+38 → 251+43).
**Delta tests total attendu** : **+51 tests** (proche projection
HARDENING_ROADMAP +50).

---

## 12. Scope cuts (PAS dans ce sprint, recopie kickoff §8)

- **Kudos-weighted gossip admission** : S22
- **Sandbox tool-calling allow-list strict** : S22
- **Redundancy voting `Task.redundancy_factor`** : S22
- **Ephemeral workers + VRAM wipe** : S23
- **Honeypot Eclipse detection** : S23
- **Re-run sampling + DNS fallback DHT** : S24
- **Arti Tor bridge integration** : S25
- **Domain fronting Snowflake-WebRTC** : S25
- **PQC migration ML-DSA + ML-KEM** : S26+
- **Hardware keystore TPM/SE/StrongBox** : S22+
- **HPKE envelope peer-restore** : S22+
- **Rust-wasm iframe PII SDK realignement Option G** : S22+
  (tech debt T-NN+2)
- **LLM-based semantic output detection** : S23+
- **ProxyPrompt proactive defense** : S23+
- **spaCy NER wasm** : DEPRECATED (drift S17 requalifié)
- **`actions/checkout@v4` pin SHA sweep** : sprint ops futur

---

## 13. Risks (R1..RN)

### R1 — Phase B G8 S1 scan verdict DESIGN-CONFLICT

**Risk** : `knowledgator/gliner-pii-edge-v1.0` backbone ou opset
incompatible avec `onnxruntime-web 1.24.3`.

**Mitigation** : modèle fallback `gliner_small-v2.1` onnx-community
(testé ONNX Runtime Web v4 Transformers.js path). Scope-cut-
consistent verdict acceptable. Dernier recours defer Phase B S22.

### R2 — Phase C EED seuil 0.85 faux positifs/négatifs

**Risk** : seuil 0.85 trop strict (bloque responses légitimes) ou
trop lâche (laisse passer PLeak attacks).

**Mitigation** : corpus tests dédiés Phase C design doc, tuning
empirique, seuil configurable `~/.sbfb/output_filter_policy.toml`
override opérateur, logging `log::warn!` tout EED > 0.7 pour
observabilité sans blocage.

### R3 — Phase D quarantine schema evolution

**Risk** : SQLite schema change post-S21 (ex: ajouter colonne
`gossip_signature_verified`) casse migration.

**Mitigation** : schema v1 initial explicite + `PRAGMA user_
version` + migration script pattern S19 `upload_queue.py`.
Documenté design doc Phase D.

### R4 — `onnxruntime-web` + `@huggingface/transformers` bundle
size explosion

**Risk** : iframe cold-start trop lent (>5s) ou `npm run size`
échoue 7/7 budget.

**Mitigation** : lazy-load modèle on-demand first use (pas eager
boot), minimal build ORT-format si nécessaire, code-splitting
Vitest bundle analyzer Phase B.

### R5 — Presidio + GLiNERRecognizer Python environment weight

**Risk** : coord-side install explose size (spaCy 500 MB + torch
2 GB + GLiNER 350 MB).

**Mitigation** : GLiNERRecognizer via `onnxruntime` Python
backend (pas transformers torch) = skip torch. Documenté Phase C
design doc.

### R6 — Drift HARDENING_ROADMAP §3 S21 non propagé dans audit S22

**Risk** : audit S22 session fraîche ne détecte pas la
requalification D2 SDK et rebat spaCy wasm.

**Mitigation** : `audited_findings 2026-04-18` frontmatter
explicite dans ce commit + kickoff §D2 acknowledge explicit
G1 §5.

### R7 — Rust-first iframe realignement S22+ bloqué indéfiniment

**Risk** : tract n'ajoute jamais opset 19 / ort ne publie jamais
wasm32-browser stable → T-NN+2 tech debt orphan.

**Mitigation** : tech debt entry documente re-evaluate triggers
explicites (voir `sprint21_carry_summary.md §2 T-NN+2`). Si
toujours bloqué S25, discuter DEPRECATED vs re-scope Phase spike
dédié.

---

## 14. Checkpoint de clôture Phase F

9 conditions pour dire « sprint fermé » :

1. 7 commits S21 landed (1 planning + 5 feat + 1 wrap-up) OU
   variant split si Phase E subdivisée.
2. 32/32 fail-fast checklist verts (§10 ci-dessus).
3. `sprint21_verification.md` + `sprint21_audit_plan.md` écrits
   dans `active/`.
4. CLAUDE.md §État actuel + SPRINT_LOG.md row S21 + memory fusion
   mis à jour.
5. Planning `sprint21_*.md` migrés `active/` → `archive/v1.2/`.
6. `sprint20_audit_findings.md` migré (déjà fait ce commit ouverture).
7. Meta-1 Radicle-v1.0 re-carry S22 explicite dans audit_plan.md
   §meta-track.
8. `.planning/active/` vide post-commit.
9. Memory frontmatter tip sync HEAD Phase F.
