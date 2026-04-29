# Sprint 40 — Plan d'execution

**Tip master** : `f8fae0c`
**Sprint pair** : Phase A dette obligatoire (§6.2.1 Regle 1)

---

## §1 Etat verifie a l'entree

| Metrique | Valeur |
|---|---|
| Tip | `f8fae0c` |
| Rust nextest | 991 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky) |
| Coord pytest | 409 + 36f + 6s |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2f |
| size-limit | 7/7 |
| clippy warnings | 0 |
| cargo fmt | clean |

---

## §2 Decisions Day 0 (gelees) — rappel

- **D1** : canary_input.py → Rust, Tripwire guardrail, strsim Levenshtein
- **D2** : Tier 3 batch 4 modules → 4 fichiers Rust separees
- **D3** : 5 items dette pair Phase A
- **D4** : P3-grammar/watermark resolus via Tier 3 migration
- **D5** : 12 scope cuts (wire routes S41+, mutation guardrail post-v1.0)

---

## §3 Research consulte

- **strsim 0.11** : dep workspace existante. `normalized_levenshtein(a, b) -> f64` retourne 1.0 pour identique, 0.0 pour completement different. Equivalent de `rapidfuzz.distance.Levenshtein.normalized_similarity`.
- **sha2 0.10** : dep transitive workspace (via iroh stack). `Sha256::digest(data)` → [u8; 32]. Pour hash comparison redundancy.
- **hmac 0.12** : dep transitive workspace. `Hmac<Sha256>` pour PRF watermark detection.
- **ed25519-dalek 2.x** : dep transitive workspace (via iroh-base). `SigningKey::generate(&mut OsRng)` pour key gen honeypot.
- **nexus-core-rs sign_bytes/verify_bytes** : deja utilise pour canary signatures existantes.
- **rusqlite** : dep directe nexus-coordinator-rs (via db.rs). Query pour rerun result hash lookup.
- **Hot-reload pattern** : etabli dans `output_filter.rs` (mtime check + Arc swap) et `pow_policy_loader.rs`. Meme pattern pour canary_input policy.

---

## §4 Phase A — Dette pair MANDATORY

### §A.1 Scope

Resoudre 5 items dette P2/P3 a 2/3 avant escalade 3/3 MANDATORY :

1. **P2-REVIEW-A-1-S38 result_event_tx** : wire le `result_event_tx`
   dans le handler `coordinator_submit_result` de `http.rs`. Quand
   un result est valide avec succes, envoyer un `ResultEvent` via
   le sender. Cela complete le pipeline validator_loop ← HTTP.

2. **P2-REVIEW-B-1-S38 substring** : dans `output_filter.rs`
   `check_prompt_echo_substring`, ajouter early exit sur premier
   match (actuellement continue l'iteration). Le
   `DEFAULT_SUBSTRING_MIN_LEN=40` existant est deja suffisant,
   pas de changement au seuil.

3. **P2-REVIEW-C-1-S38 chain Arc singleton** : dans `guardrails.rs`,
   transformer `default_output_chain()` et `default_input_chain()` en
   constantes `OnceLock<GuardrailChain>`. Ajouter le chain dans
   `DaemonHttpState` ou utiliser `OnceLock` static. Les guardrails
   internes (PiiRedactor) utilisent deja OnceLock pour les regex.

4. **P2-REVIEW-C-1-S39 HTTP integration tests** : ajouter 3 tests
   async dans `http.rs` module tests :
   - `test_submit_task_pii_rejected` : prompt avec email → 400
   - `test_canary_observed_post` : POST /api/canary/observed → 200
   - `test_canary_network_health_get` : GET /api/canary/network-health → 200

5. **P3-AUDIT-A-2b-S38 lowercase divergence** : ajouter une note
   dans `docs/rust/PATTERNS.md` documentant la convention : Rust
   case-sensitive pour les identifiants wire, Python lowercase.

### §A.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/http.rs` | wire result_event_tx dans submit_result handler + 3 tests integration |
| `crates/nexus-coordinator-rs/src/output_filter.rs` | min_substring_len + early exit |
| `crates/nexus-coordinator-rs/src/guardrails.rs` | OnceLock chain singleton |
| `docs/rust/PATTERNS.md` | lowercase convention doc |

### §A.3 Tests plan

1. `test_submit_task_pii_rejected` — prompt "Contact me at test@example.com" → 400 input_rejected
2. `test_canary_observed_post` — POST canary observation JSON → 200
3. `test_canary_network_health_get` — GET network health → 200 avec status
4. Tests existants output_filter substring : verifier non-regression avec min_length

### §A.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon --locked
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §A.5 Commit cible

```
feat(sprint40): Sprint 40 Phase A — dette pair P2 batch 2/3 items + HTTP integration tests

Fichiers : http.rs (result_event_tx wire + 3 tests) +
output_filter.rs (min_substring_len 8 + early exit) +
guardrails.rs (OnceLock chain singleton) +
PATTERNS.md (lowercase convention)

Items resolus :
- P2-REVIEW-A-1-S38 result_event_tx dead code RESOLU (wire submit_result)
- P2-REVIEW-B-1-S38 substring O(n*m) RESOLU (min len + early exit)
- P2-REVIEW-C-1-S38 chain Arc singleton RESOLU (OnceLock static)
- P2-REVIEW-C-1-S39 HTTP integration tests RESOLU (3 tests PII + canary)
- P3-AUDIT-A-2b-S38 lowercase divergence RESOLU (doc PATTERNS)

Delta tests : 991 → 991+3 = 994 (+3)
Scope cuts respectes : 12/12
```

---

## §5 Phase B — canary_input.py migration (Tier 2 fin)

### §B.1 Scope

Migrer `canary_input.py` (783 LOC) vers `canary_input.rs` dans
nexus-coordinator-rs. Modules fonctionnels :

1. **Types serde** :
   - `CanaryPrompt` : prompt_id + prompt + expected_answer + tolerance
   - `CanaryInputSet` : version + prompts Vec + signature + pubkey
   - `DivergenceRecord` : prompt_id + expected + actual + similarity + timestamp
   - `CanaryInputPolicy` : enabled + inject_rate + tolerance + rotation_frequency + set_path

2. **CanaryInputSet signature** :
   - `signable_json(&self) -> String` : JSON sort_keys canonique (identique Python)
   - `build_canary_input_set(prompts, signing_key) -> CanaryInputSet`
   - `verify_canary_input_set(set) -> bool` via nexus-core-rs verify_bytes
   - `save/load_canary_input_set` : JSON file I/O

3. **CanaryInputInjector** :
   - `should_inject(&self) -> bool` : sampling 1/inject_rate round-robin
   - `next_prompt(&mut self) -> Option<&CanaryPrompt>` : round-robin
   - Thread-safe via AtomicUsize counter (pas Mutex)

4. **CanaryInputObserver** :
   - `observe(&mut self, prompt_id, expected, actual)` : compare via
     strsim::normalized_levenshtein, enregistre divergence si < tolerance
   - `divergences(&self) -> &VecDeque<DivergenceRecord>` : ring buffer borne

5. **CanaryInputPolicy hot-reload** :
   - Pattern mtime debounce identique output_filter.rs
   - `CanaryInputManager` : owns Injector + Observer + Policy
   - `maybe_reload(&mut self)` : check mtime, reload TOML + set file

6. **CanaryInputGuardrail** :
   - `impl Guardrail for CanaryInputGuardrail`
   - Direction Input, `check()` → Tripwire si is_canary flag

7. **DEFAULT_SEED_PROMPTS** : 5 prompts factory-default (identiques Python)

### §B.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/canary_input.rs` | NEW — module complet |
| `crates/nexus-coordinator-rs/src/lib.rs` | +pub mod canary_input |
| `crates/nexus-coordinator-rs/Cargo.toml` | +sha2 dep directe si necessaire |

### §B.3 Tests plan

1. `test_canary_prompt_serde` — roundtrip serialize/deserialize
2. `test_canary_input_set_sign_verify` — sign + verify Ed25519
3. `test_canary_input_set_tampered` — tampered set → verify fails
4. `test_canary_input_set_save_load` — file I/O roundtrip
5. `test_injector_rate` — 1/5 rate = ~20% injection
6. `test_injector_round_robin` — cycle through prompts
7. `test_observer_divergence_below_tolerance` — similarity < threshold → record
8. `test_observer_no_divergence_above_tolerance` — similarity >= threshold → no record
9. `test_observer_ring_buffer_bounded` — buffer doesn't grow past max
10. `test_policy_from_toml` — parse TOML config
11. `test_default_seed_prompts` — 5 prompts present
12. `test_guardrail_tripwire` — canary input → Tripwire

### §B.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §B.5 Commit cible

```
feat(sprint40): Sprint 40 Phase B — CanaryInput Rust canary_input.rs

Fichiers : canary_input.rs (NEW) + lib.rs (pub mod) +
Cargo.toml (deps)

Composants : CanaryInputSet Ed25519 signed + CanaryInputInjector
round-robin + CanaryInputObserver Levenshtein strsim +
CanaryInputPolicy TOML hot-reload + CanaryInputGuardrail Tripwire +
DEFAULT_SEED_PROMPTS 5 factory prompts

Delta tests : 994 → 994+12 = 1006 (+12)
Scope cuts respectes : 12/12
```

---

## §6 Phase C — Tier 3 batch migration

### §C.1 Scope

Migrer les 4 modules Tier 3 vers Rust dans nexus-coordinator-rs.
Resolut P3-grammar (3/3+) et P3-watermark (3/3+).

**Module 1 — redundancy.rs**

Port de `redundancy.py` (159 LOC). Vote majorite :
- `VoteVerdict` enum : Majority / Mismatch
- `VoteOutcome` struct : verdict + canonical_hash + outlier_worker_ids
- `RedundancyDispatcher` : register_task(task_id, n_workers) +
  collect_result(task_id, worker_id, result_hash) +
  vote(task_id) -> Option<VoteOutcome>
- Hash : SHA-256 via sha2 crate (parite Python hashlib.sha256)
- In-memory HashMap tracker

**Module 2 — watermark_detector.rs**

Port de `watermark_detector.py` (120 LOC). Z-test SynthID :
- `WatermarkResult` struct : is_watermarked + z_score + green_ratio + token_count
- `WatermarkDetector` struct : detect(token_ids, task_key) -> WatermarkResult
- `_prf_score(token_id, key)` : HMAC-SHA256 PRF
  (mirror exact `crates/nexus-worker-core/src/llm/watermark.rs`)
- Z-test binomial : z = (green_count - n/2) / sqrt(n/4)
- Threshold : z > 4.0 → watermarked (meme seuil que Python)

**Module 3 — rerun.rs**

Port de `rerun.py` (193 LOC). Spot-check :
- `RerunConfig` struct : sample_rate + max_pending
- `RerunSampler` : should_rerun(task_id) sampling aleatoire +
  register_rerun(rerun_id, original_id) + is_rerun(task_id)
- `DivergenceScorer` : score_result(original_hash, rerun_hash) -> f64
  (0.0 match, 1.0 mismatch)
- Anti-loop : rerun de rerun interdit
- DB query result hash via rusqlite (pattern db.rs existant)

**Module 4 — honeypot.rs**

Port de `honeypot.py` (222 LOC). Eclipse detection :
- `CanaryPeer` struct : public_key_hex + created_at
- `EclipseAlert` struct : worker_id + co_location_pct + consecutive_rotations
- `CanaryPeerFactory` : generate(n) → Vec<CanaryPeer> via ed25519-dalek
- `EclipseDetector` : evaluate_rotation(worker_id, seen_canaries) +
  check_alerts() → Vec<EclipseAlert>
  Seuils : co_location > 0.8 + 3 rotations consecutives
- `CanaryRotationScheduler` : cadence 6h, owns Factory + Detector

### §C.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/redundancy.rs` | NEW |
| `crates/nexus-coordinator-rs/src/watermark_detector.rs` | NEW |
| `crates/nexus-coordinator-rs/src/rerun.rs` | NEW |
| `crates/nexus-coordinator-rs/src/honeypot.rs` | NEW |
| `crates/nexus-coordinator-rs/src/lib.rs` | +4 pub mod |
| `crates/nexus-coordinator-rs/Cargo.toml` | +sha2 +hmac deps directes |

### §C.3 Tests plan

**redundancy.rs :**
1. `test_vote_majority_3_workers` — 2/3 agree → Majority
2. `test_vote_mismatch` — 3/3 different → Mismatch
3. `test_vote_pending` — not all results in → None

**watermark_detector.rs :**
4. `test_detect_watermarked` — biased tokens → is_watermarked=true
5. `test_detect_not_watermarked` — random tokens → is_watermarked=false
6. `test_prf_score_deterministic` — same input → same score
7. `test_z_threshold` — boundary z=4.0

**rerun.rs :**
8. `test_sampler_rate` — 10% rate → ~10% selected
9. `test_anti_loop_rerun_of_rerun` — rerun task → not selected
10. `test_divergence_scorer_match` — same hash → 0.0
11. `test_divergence_scorer_mismatch` — different hash → 1.0

**honeypot.rs :**
12. `test_factory_generate` — n canary peers with unique keys
13. `test_eclipse_alert_threshold` — 3 consecutive > 0.8 → alert
14. `test_eclipse_no_alert_below_threshold` — < 0.8 → no alert
15. `test_rotation_resets_factory` — new rotation → new peers

### §C.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §C.5 Commit cible

```
feat(sprint40): Sprint 40 Phase C — Tier 3 batch redundancy + watermark + rerun + honeypot Rust

Fichiers : redundancy.rs (NEW) + watermark_detector.rs
(NEW) + rerun.rs (NEW) + honeypot.rs (NEW)
+ lib.rs (+4 pub mod) + Cargo.toml (+sha2 +hmac)

Resolut :
- P3-grammar executor 3/3+ RESOLU (rerun.rs pipeline Rust)
- P3-watermark executor 3/3+ RESOLU (watermark_detector.rs Rust)

Delta tests : 1006 → 1006+15 = 1021 (+15)
Scope cuts respectes : 12/12
```

---

## §7 Phase D — Wrap-up

Verification.md 28+ rows fail-fast, sprint41_audit_plan.md,
SPRINT_LOG.md row S40, CLAUDE.md etat actuel, HARDENING_ROADMAP
compteurs + last_validated.

Commit : `chore(sprint40): Phase D — wrap-up + verification +
audit plan S41 + migration`
