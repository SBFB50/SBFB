# Sprint 23 — Plan d'exécution détaillé

**Écrit** : 2026-04-20.
**Kickoff source** : `.planning/active/sprint23_kickoff.md`.
**Tip master** : `2438c59`.

---

## 1. État vérifié à l'entrée

```
Rust workspace:     710 tests (nextest + doc)
Python SDK:         185 tests
Python coord:       263 + 3 skipped
Python app-gov:     46 tests
Vitest unit:        264 tests
Playwright:         38 tests
Size-limit:         7/7 budgets
SPDX:               246+ licenses
Total:              ~1509 tests
Clippy warnings:    0
Ruff violations:    0
TSC errors:         0
```

---

## 2. Décisions Day 0 (gelées — rappel kickoff §4)

| D | Choix | Implication code |
|---|---|---|
| D1 | Ephemeral restart + VRAM wipe | `ephemeral.rs` + cudarc + worker.toml |
| D2 | PoW geometric ×2 per-(consumer, model) | `EscalatingPolicy` + compteur coord |
| D3 | Redundancy factor 1/3/5 majority | Task wire + `redundancy.py` + dispatcher |
| D4 | Canary peer rotation + alert 80%/3rot | `honeypot.py` + daemon neighborhood |
| D5 | B1 guardrails → S24 Phase A | Zero code guardrails S23 |

---

## 3. Research consult��

- **cudarc 0.12** (docs.rs) : safe CUDA bindings, `CudaDevice::
  synchronize()` + raw `cuMemsetD8` via `CudaFunction`. Feature-gated
  pour build sans GPU (CI).
- **BOINC result validation** (boinc.berkeley.edu) : validator compare
  canonical result hash, N replicas, outlier quarantine. Pattern
  direct pour D3.
- **iroh 0.97 `Endpoint::node_addr()`** : seul moyen d'observer le
  neighborhood (addr info des peers connectés). Canary peers = publish
  dummy NodeAddr, observer qui les resolve.
- **`rapidfuzz` 3.x** (Python) : déjà dep coord-side (canary_input
  S22E). Pas de nouvelle dep pour fairness endpoint (Gini = pure
  math, pas de lib externe).
- **G1 acknowledged** : Equi-X trigger S24, BOINC > SecureDrop pour
  D3 design note.

---

## 4. Phase A — P2 audit cleanup batch + process fix

### Fichiers modifiés

- `crates/nexus-worker-core/Cargo.toml` — retirer `dashmap` dep
  directe (P2-S22A-1)
- `docs/rust/PATTERNS.md` §P33 — update struct snippet
  `RwLock<RateLimiterState>` + paragraphe post-S22 wire-up
  (P2-S22A-3)
- `web/src/sdk/pii/wrapper.ts:308-313` — update commentaire L309
  "Model returns 0 findings = no PII detected (defense-in-depth
  fallback kept)" (P2-B-2)
- `packages/nexus-coordinator/src/nexus_coordinator/canary_input.py`
  — rename `_reload_policy_locked` → `_reload_policy_inner` + update
  caller L508 (P2-E-1)
- `docs/claude/README.md` §6.7 — amend "Pas d'estimation LOC" section
  avec clarification (P2-E-2)
- `.claude/.bypass_audit_trail.log` — ajouter header "Note: forward-
  only from Phase F S22 creation" (P2-Meta-hook-1)
- `crates/nexus-core-rs/src/lib.rs` — ajouter re-export
  `DOMAIN_PROVENANCE_V1` + `DOMAIN_WARRANT_CANARY_V1` (P3-C-1)

### Tests à écrire

Aucun nouveau test (cleanup batch). Tous les tests existants doivent
rester verts (non-regression).

### Critère d'acceptation

- `cargo nextest run --workspace --locked` vert
- `cargo clippy --workspace --all-targets --locked -- -D warnings` vert
- `uv run pytest packages/nexus-coordinator/tests/ -q` vert
- `cd web && npm run test:unit` vert
- `dashmap` absent de `grep -r "dashmap" crates/nexus-worker-core/`
  (ni Cargo.toml ni use)

### Commit cible

```
feat(sprint23): Phase A — P2 cleanup batch S22 audit findings + README §6.7 LOC convention amend
```

---

## 5. Phase B — Ephemeral workers (restart + VRAM wipe)

### Fichiers ajoutés/modifiés

- `crates/nexus-worker-core/src/ephemeral.rs` (NOUVEAU) :
  - `EphemeralConfig { max_tasks: u32, vram_wipe: bool }`
  - `EphemeralLifecycle` state : `Ready → Running → WipePending →
    RestartPending → Exiting`
  - `fn should_restart(&self, completed: u32) -> bool`
  - `async fn wipe_vram() -> Result<()>` (cudarc `cuMemsetD8` sur
    chaque device visible)
  - `fn request_exit(&self)` (signale au runtime)
- `crates/nexus-worker-core/src/engine/runtime.rs` — intégration
  post-`complete_task()` : check ephemeral lifecycle, wipe, restart
  signal
- `crates/nexus-worker-core/Cargo.toml` — add `cudarc = { version =
  "0.12", optional = true, features = ["driver"] }`
- Feature flag `gpu-ephemeral` (default off, enabled quand GPU
  présent)
- `configs/worker.toml.sample` — section `[ephemeral]`

### Tests à écrire

1. `test_lifecycle_transitions` — Ready→Running→WipePending→Exiting
2. `test_should_restart_at_max` — max_tasks=3, completes 3 → true
3. `test_should_not_restart_below_max` — completes 2 → false
4. `test_wipe_vram_no_gpu` — feature off → Ok(()) noop
5. `test_wipe_vram_mock_device` — mock cudarc → memset called
6. `test_config_parse_toml` — deserialize sample config
7. `test_config_default_values` — max_tasks=50, wipe=true
8. `test_restart_signal_sets_exit` — request_exit → WorkerState::
   Exiting

### Critère d'acceptation

- 8+ tests verts dans `ephemeral.rs`
- `cargo build -p nexus-worker-core --no-default-features --locked`
  (build sans GPU)
- `cargo build -p nexus-worker-core --features gpu-ephemeral --locked`
  (build avec GPU)
- Runtime.rs intégration compile + tests existants engine inchangés

### Commit cible

```
feat(sprint23): Phase B — ephemeral worker lifecycle restart + VRAM cudaMemset wipe inter-task
```

---

## 6. Phase C — Escalating PoW difficulty ramp

### Fichiers ajoutés/modifiés

- `crates/nexus-core-rs/src/pow.rs` — ajout :
  - `EscalatingPolicy { base_difficulty: u32, multiplier: f64,
    tranche_size: u32, max_difficulty: u32 }`
  - `fn difficulty_for(policy: &EscalatingPolicy, task_count: u64)
    -> u32`
  - `fn should_reset(last_reset: SystemTime) -> bool` (minuit UTC)
- `packages/nexus-coordinator/src/nexus_coordinator/pow_counter.py`
  (NOUVEAU) :
  - `PowCounter` class : SQLite table `pow_task_counts (consumer_id,
    model_id, count, last_reset_utc)`
  - `increment(consumer_id, model_id) -> u64`
  - `get_count(consumer_id, model_id) -> u64`
  - `reset_expired()` (daily UTC)
- `crates/nexus-core-rs/src/gossip.rs` — `join_topic_with_pow`
  accepte `DifficultyTarget` dynamic (pas const)
- `configs/pow_escalation.toml.sample` — config sample

### Tests à écrire

1. `test_difficulty_base` — count=0 → base_difficulty
2. `test_difficulty_ramp_first_tranche` — count=K → base×2
3. `test_difficulty_ramp_third_tranche` — count=3K → base×8
4. `test_difficulty_cap_max` — count=100K → max_difficulty
5. `test_reset_daily` — after midnight UTC → reset to 0
6. `test_per_consumer_isolation` — consumer A count ≠ consumer B
7. `test_per_model_isolation` — model X count ≠ model Y
8. `test_overflow_u32` — multiplier overflow → saturate max
9. `test_pow_counter_increment` (Python) — counter increments
10. `test_pow_counter_reset_expired` (Python) — old entries reset
11. `test_dynamic_difficulty_wire` — gossip accepts dynamic target

### Critère d'acceptation

- 8+ tests Rust verts (pow.rs)
- 3+ tests Python verts (pow_counter.py)
- `cargo clippy` vert
- Integration : gossip subscribe avec dynamic difficulty compile

### Commit cible

```
feat(sprint23): Phase C — escalating PoW geometric ramp per-(consumer, model) with daily reset
```

---

## 7. Phase D — Redundancy voting 3-worker majority

### Fichiers ajoutés/modifiés

- `crates/nexus-core-rs/src/task.rs` — champ `redundancy_factor: u8`
  dans `Task` struct (pre-launch protocol = redefine v1)
- `packages/nexus-coordinator/src/nexus_coordinator/redundancy.py`
  (NOUVEAU) :
  - `RedundancyDispatcher` class :
    - `dispatch_redundant(task, factor) -> List[WorkerAssignment]`
    - `collect_results(task_id) -> List[SignedResult]`
    - `vote(results) -> VoteOutcome` (Majority/Mismatch)
    - `quarantine_outliers(task_id, outlier_ids)`
  - `VoteOutcome` enum : `Majority(canonical_hash)` |
    `Mismatch(hashes)`
  - BLAKE3 hash of canonical result bytes for comparison
- `packages/nexus-coordinator/src/nexus_coordinator/dispatcher.py` —
  integration : if `task.redundancy_factor > 1` → route to
  `RedundancyDispatcher`
- `packages/nexus-coordinator/src/nexus_coordinator/api/tasks.py` —
  expose `redundancy_factor` dans task creation endpoint

### Tests à écrire

1. `test_dispatch_factor_1_passthrough` — factor=1 → normal dispatch
2. `test_dispatch_factor_3_three_workers` — 3 distinct workers assigned
3. `test_collect_all_match` — 3 identical hashes → Majority
4. `test_collect_2_of_3_match` — 2 match 1 differ → Majority + quarantine 1
5. `test_collect_all_differ` — 3 different → Mismatch + quarantine all
6. `test_quarantine_notifies_curator` — outlier → quarantine queue entry
7. `test_factor_5_majority` — 3/5 match → Majority
8. `test_hash_canonical_deterministic` — same result bytes → same hash
9. `test_task_wire_redundancy_factor` (Rust) — serialize/deserialize
10. `test_task_wire_default_factor_1` (Rust) — omitted field → 1
11. `test_dispatcher_routes_redundant` — factor>1 → RedundancyDispatcher
12. `test_api_accepts_redundancy_factor` — POST /tasks with factor=3

### Critère d'acceptation

- 2+ tests Rust (wire format)
- 10+ tests Python (redundancy module + dispatcher + API)
- Integration dispatcher route effective
- Pre-launch protocol : `TASK_FORMAT_VERSION` reste 1 (pas de bump)

### Commit cible

```
feat(sprint23): Phase D — redundancy voting 3-worker majority Task.redundancy_factor + quarantine outliers
```

---

## 8. Phase E — Honeypot Eclipse detection + fairness observability

### Fichiers ajoutés/modifiés

- `packages/nexus-coordinator/src/nexus_coordinator/honeypot.py`
  (NOUVEAU) :
  - `CanaryPeerFactory` : generate K dummy NodeId Ed25519 (rotation
    keypair every 6h)
  - `CanaryRotationScheduler` : async loop 6h, publish new canary
    peers via pkarr
  - `EclipseDetector` : track neighborhood reports per real worker,
    alert if worker appears in >80% canary neighborhoods across 3
    rotations
  - `EclipseAlert` dataclass → curator notification
- `crates/nexus-shell-daemon-core/src/api/diagnostic.rs` (NOUVEAU) :
  - `GET /diagnostic/neighborhood` — returns current peer neighborhood
    snapshot (node_ids of known peers from routing)
- `packages/nexus-coordinator/src/nexus_coordinator/api/diagnostic.py`
  (NOUVEAU ou extend existing) :
  - `GET /diagnostic/fairness` — returns Gini coefficient +
    top-5% compute share + churn-rate-vs-hardware from kudos ledger
- `packages/nexus-coordinator/src/nexus_coordinator/fairness.py`
  (NOUVEAU) :
  - `compute_gini(contributions: List[float]) -> float`
  - `compute_top_k_share(contributions, k=5) -> float`
  - `compute_churn_rate(ledger_history) -> float`

### Tests à écrire

1. `test_canary_peer_generates_valid_ed25519` — keypair valid
2. `test_rotation_produces_new_peers` — rotation → different NodeId
3. `test_eclipse_below_threshold_no_alert` — 60% co-location → no alert
4. `test_eclipse_above_threshold_alert` — 85% × 3 rotations → alert
5. `test_eclipse_resets_on_rotation_miss` — miss 1 rotation → reset counter
6. `test_gini_equal_distribution` — all same → 0.0
7. `test_gini_maximum_inequality` — one has all → ~1.0
8. `test_gini_realistic` — sample data → expected range
9. `test_top_5_share` — top 5% of 100 contributors
10. `test_fairness_endpoint_returns_json` — GET /diagnostic/fairness → 200
11. `test_neighborhood_endpoint` (Rust) — GET /diagnostic/neighborhood → 200

### Critère d'acceptation

- 1+ test Rust (diagnostic endpoint)
- 10+ tests Python (honeypot + fairness)
- Endpoints accessibles via loopback bearer auth
- No new wire format (canary peers use existing gossip publish)

### Commit cible

```
feat(sprint23): Phase E — honeypot Eclipse canary peer detection + fairness observability diagnostic endpoint
```

---

## 9. Phase F — Design docs + wrap-up

### Fichiers ajoutés/modifiés

- `docs/fairness/CONTRIBUTION_FAMILIES_V1.md` (NOUVEAU) : design doc
  Option F 3 couches asymétriques (compute / storage / relay), weight
  vectors, decay functions, Gini trigger LT-1
- `docs/fairness/KUDOS_V2_WIRE.md` (NOUVEAU) : wire format spec
  (pre-launch design-only, pas de code)
- `crates/nexus-core-rs/src/attestations/delegation.rs` (extend) :
  `DelegationCert` struct format finalization (fields, serde, domain
  separation)
- `.planning/active/sprint23_verification.md` — fail-fast checklist
- `.planning/active/sprint23_audit_plan.md` — audit plan S24
- Memory + CLAUDE.md + SPRINT_LOG + HARDENING updates
- Migration `.planning/active/` → `.planning/archive/v1.2/`

### Tests à écrire

- 2+ tests Rust `DelegationCert` (serialize roundtrip + domain
  separation distinct)

### Critère d'acceptation

- 28+ rows fail-fast verts
- All suites green (Rust + Python + Web)
- Design docs present et self-contained
- `DelegationCert` struct + tests compile

### Commit cible

```
chore(sprint23): Phase F — contribution families design docs + Couche 3 cert format + wrap-up + verification + audit plan S24
```

---

## 10. Fail-fast checklist

| # | Check | Commande | Critère | Observed |
|---|---|---|---|---|
| 1 | Rust compile workspace | `cargo build --workspace --locked` | exit 0 | |
| 2 | Rust clippy clean | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | Rust fmt check | `cargo fmt --all --check` | exit 0 | |
| 4 | Rust nextest pass | `cargo nextest run --workspace --locked` | all pass | |
| 5 | Rust doctests pass | `cargo test --workspace --locked --doc` | all pass | |
| 6 | Python ruff format | `uv run ruff format --check packages/` | exit 0 | |
| 7 | Python ruff lint | `uv run ruff check packages/` | exit 0 | |
| 8 | Python SDK tests | `uv run pytest packages/nexus-sdk/tests/ -q` | all pass | |
| 9 | Python coord tests | `uv run pytest packages/nexus-coordinator/tests/ -q` | all pass | |
| 10 | Python gov tests | `uv run pytest packages/nexus-app-gov/tests/ -q` | all pass | |
| 11 | Web TSC check | `cd web && npx tsc --noEmit -p tsconfig.app.json` | exit 0 | |
| 12 | Web lint | `cd web && npm run lint` | exit 0 | |
| 13 | Web unit tests | `cd web && npm run test:unit` | all pass | |
| 14 | Web build | `cd web && npm run build` | exit 0 | |
| 15 | Web size-limit | `cd web && npm run size` | 7/7 pass | |
| 16 | Playwright e2e | `cd web && npx playwright test` | all pass | |
| 17 | Shell daemon build | `cargo build -p nexus-shell-daemon --release` | exit 0 | |
| 18 | Worker build no-gpu | `cargo build -p nexus-worker-core --no-default-features --locked` | exit 0 | |
| 19 | Worker build gpu | `cargo build -p nexus-worker-core --features gpu-ephemeral --locked` | exit 0 | |
| 20 | dashmap absent worker-core | `grep -r "dashmap" crates/nexus-worker-core/` | 0 matches | |
| 21 | ephemeral tests | `cargo nextest run -p nexus-worker-core -E "test(ephemeral)"` | 8+ pass | |
| 22 | pow escalation tests | `cargo nextest run -p nexus-core-rs -E "test(escalat)"` | 8+ pass | |
| 23 | redundancy tests | `uv run pytest packages/nexus-coordinator/tests/ -k redundancy -q` | 10+ pass | |
| 24 | honeypot tests | `uv run pytest packages/nexus-coordinator/tests/ -k honeypot -q` | 5+ pass | |
| 25 | fairness tests | `uv run pytest packages/nexus-coordinator/tests/ -k fairness -q` | 4+ pass | |
| 26 | Task wire roundtrip | `cargo nextest run -p nexus-core-rs -E "test(redundancy_factor)"` | pass | |
| 27 | diagnostic endpoint Rust | `cargo nextest run -p nexus-shell-daemon-core -E "test(neighborhood)"` | pass | |
| 28 | DelegationCert tests | `cargo nextest run -p nexus-core-rs -E "test(delegation)"` | 2+ pass | |
| 29 | SPDX scan | `cd web && bash scripts/scan-en-strings.sh` | exit 0 | |
| 30 | Pre-launch versions stable | `grep -rE "_VERSION\s*[:=]\s*[0-9]+" crates/nexus-core-rs/src/` | TASK_FORMAT_VERSION=1 unchanged | |

---

## 11. Git plan

```
1. feat(sprint23): Phase A — P2 cleanup batch S22 audit findings + README §6.7 LOC convention amend
2. feat(sprint23): Phase B — ephemeral worker lifecycle restart + VRAM cudaMemset wipe inter-task
3. feat(sprint23): Phase C — escalating PoW geometric ramp per-(consumer, model) with daily reset
4. feat(sprint23): Phase D — redundancy voting 3-worker majority Task.redundancy_factor + quarantine outliers
5. feat(sprint23): Phase E — honeypot Eclipse canary peer detection + fairness observability diagnostic endpoint
6. chore(sprint23): Phase F — contribution families design docs + Couche 3 cert format + wrap-up + verification + audit plan S24
```

---

## 12. Scope cuts (rappel kickoff §6)

1. B1 guardrails refactor → S24 Phase A
2. Couche 3 DelegationCert implem runtime → S25-S27
3. Contribution families implem code → post-v1.0 LT-3
4. Traffic padding → S28
5. Exponential cooldown per-identity → DÉFERÉ
6. Honeypot auto-quarantine → post-Gate 3
7. P2-B-1 ONNX CI fixture → S24 Track B
8. T-NN+2 iframe Rust-wasm → PATTERNS §P34

---

## 13. Risks

| ID | Risque | Mitigation |
|---|---|---|
| R1 | cudarc 0.12 ne compile pas sur Windows sans CUDA toolkit | Feature-gate `gpu-ephemeral` off par défaut, CI build teste `--no-default-features` |
| R2 | iroh 0.97 ne permet pas de "planter" un dummy peer dans le DHT | Canary peer = publish pkarr record, pas injection DHT directe. Fallback : honeypot via gossip topic dédié |
| R3 | Redundancy factor dans Task wire pourrait casser le canonical hash | `redundancy_factor` exclu du canonical bytes (champ dispatch-only, pas task identity). Serde `#[serde(default)]` pour robustesse |
| R4 | Gini coefficient edge case ledger vide | Guard explicit : empty ledger → Gini = 0.0, top_k = 0.0 |

---

## 14. Checkpoint de clôture

- [ ] 30/30 fail-fast verts
- [ ] 6 commits atomiques (Phase A-F)
- [ ] PATTERNS.md mis à jour (§P33 + nouveau §P35 ephemeral + §P36 redundancy)
- [ ] `sprint23_verification.md` + `sprint23_audit_plan.md` écrits
- [ ] Memory + CLAUDE.md + SPRINT_LOG + HARDENING `last_validated` mis à jour
- [ ] Pre-launch protocol respecté (0 version bump)
