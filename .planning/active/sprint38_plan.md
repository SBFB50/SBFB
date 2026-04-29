# Sprint 38 — Plan

**Kickoff ref** : `sprint38_kickoff.md` (D1..D5 gelees).
**Phases** : A (dette pair + MANDATORY), B (OutputFilter),
C (Guardrails + wire), D (wrap-up).

---

## Phase A — dette pair : MANDATORY validator_loop + P2 batch

### §A.1 validator_loop tokio (D1)

1. **Expose LiveEvents depuis CuratorRuntimeHandle** :
   Modifier `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` :
   - Ajouter un champ `result_events_tx: tokio::sync::broadcast::Sender<ResultEvent>`
     dans `CuratorRuntime` (ou struct dediee).
   - Definir `ResultEvent` enum minimal :
     ```rust
     pub enum ResultEvent {
         NewResult { task_id: String, blob_hash: Vec<u8> },
     }
     ```
   - Ajouter `pub fn subscribe_result_events(&self) -> broadcast::Receiver<ResultEvent>`
     a CuratorRuntimeHandle.
   - Dans la gossip subscribe loop existante, quand un message
     gossip de type result est recu, envoyer sur le broadcast channel.

2. **Spawn validator_loop dans le daemon runtime** :
   Modifier `crates/nexus-shell-daemon/src/runtime.rs` :
   - Ajouter une fonction `spawn_validator_loop(db: Arc<Mutex<CoordinatorDb>>, rx: broadcast::Receiver<ResultEvent>)`.
   - Le loop : `while let Ok(event) = rx.recv().await { ... }`.
   - Pour chaque `NewResult` : deserialise le ResultEntry, appelle
     `validate_result()`, si Accepted appelle `credit()`.
   - Idempotence : `set_task_result()` retourne false si deja
     completed → pas de double credit.

3. **Wire dans DaemonHttpState** :
   Modifier `crates/nexus-shell-daemon/src/http.rs` :
   - Le validator_loop est spawne au boot, pas expose via HTTP.
   - Le handler `coordinator_submit_result` reste inchange
     (fallback synchrone).

4. **Tests** :
   - `validator_loop_processes_result` : mock broadcast → validate
   - `validator_loop_idempotent_double_submit` : 2 events meme task → 1 credit
   - `validator_loop_rejects_bad_signature` : event avec sig invalide → no credit

### §A.2 P2 batch dette

1. **Rowid documentation** (P2-REVIEW-B-1-S37, 2/3) :
   - Ajouter commentaire inline dans `db.rs` L202 et L217 :
     ```rust
     // rowid tiebreaker ensures deterministic ordering when
     // multiple entries share the same created_at second.
     ```
   - Ajouter section dans `docs/shell/PATTERNS.md` sous un
     nouveau pattern (P40 ou prochain numero) documentant
     l'invariant rowid pour les queries kudos.

2. **Launcher logging test** (P2-REVIEW-A-1-S37, 1/3) :
   - Ajouter un test dans `crates/nexus-launcher/src/main.rs` :
     ```rust
     #[test]
     fn launcher_log_dir_matches_daemon_log_dir() {
         // Both must resolve to <root>/logs/
         let launcher = launcher_log_dir();
         let daemon = nexus_shell_daemon_core::paths::log_dir()
             .expect("log_dir");
         assert_eq!(launcher, daemon);
     }
     ```

3. **verify_chain HTTP endpoint** (SC-12-S37) :
   - Ajouter route `GET /api/v1/kudos/{project_id}/verify` dans
     `http.rs` router.
   - Handler : lock coordinator_db → `verify_chain(&db, &project_id)`
     → `Json(json!({"valid": result}))`.
   - Poison guard identique aux 3 autres handlers coordinator.
   - 1 test : `verify_chain_endpoint_returns_true`.

### §A.3 Delta tests attendu

- +3 validator_loop tests
- +1 launcher log dir test
- +1 verify_chain endpoint test
- +1 rowid doc (pas de test, doc seulement)
Total : +5 tests

### §A.4 Verification Phase A

- `cargo nextest run -p nexus-shell-daemon --locked`
- `cargo nextest run -p nexus-shell-daemon-core --locked`
- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo nextest run -p nexus-launcher --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- `cargo build -p nexus-shell-daemon --release`
- Full fail-fast 3 blocs

---

## Phase B — OutputFilter Rust migration

### §B.1 Dependencies

1. Ajouter `strsim = "0.11"` au `[workspace.dependencies]` dans
   `Cargo.toml` racine.
2. Ajouter `strsim = { workspace = true }` a
   `crates/nexus-coordinator-rs/Cargo.toml`.

### §B.2 Module output_filter.rs

1. Creer `crates/nexus-coordinator-rs/src/output_filter.rs`.

2. **Types** :
   ```rust
   pub enum FilterReason {
       Ok,
       InvisibleText,
       PromptEchoExact,
       PromptEchoSubstring,
       PromptEchoEed,
   }

   pub struct FilterVerdict {
       pub is_valid: bool,
       pub reason: FilterReason,
       pub risk_score: f64,
       pub sanitized_output: String,
   }
   ```

3. **Invisible text scanner** :
   - `fn strip_invisible(input: &str) -> String` : itere sur chars,
     filtre les categories invisibles (zero-width, PUA, tags),
     preserve les bidi format chars.
   - `fn has_invisible_text(input: &str) -> bool` :
     `input.len() != strip_invisible(input).len()`.

4. **Prompt echo detection** :
   - `fn check_prompt_echo_exact(prompt: &str, output: &str) -> bool`
   - `fn check_prompt_echo_substring(prompt: &str, output: &str, min_len: usize) -> bool` :
     sliding window 40+ chars du system_prompt dans l'output.
   - `fn check_prompt_echo_eed(prompt: &str, output: &str, threshold: f64) -> bool` :
     `strsim::normalized_levenshtein(prompt, output) >= threshold`.

5. **OutputFilter struct** :
   ```rust
   pub struct OutputFilter {
       eed_threshold: f64,     // default 0.85
       substring_min_len: usize, // default 40
   }

   impl OutputFilter {
       pub fn filter(&self, system_prompt: &str, user_prompt: &str, output: &str) -> FilterVerdict;
   }
   ```

6. **Policy** : `OutputFilterPolicy` struct deserialise depuis TOML.
   Default values hardcoded. Pattern `load_policy(path) -> Policy`.

### §B.3 Wire dans lib.rs

1. Ajouter `pub mod output_filter;` dans
   `crates/nexus-coordinator-rs/src/lib.rs`.

### §B.4 Tests

- `strip_invisible_removes_zero_width` : U+200B, U+FEFF
- `strip_invisible_preserves_bidi` : U+202A, U+2066
- `strip_invisible_removes_pua` : U+E000
- `strip_invisible_removes_tags` : U+E0020
- `prompt_echo_exact_detected` : system_prompt == output
- `prompt_echo_substring_detected` : 50-char slice present
- `prompt_echo_eed_detected` : normalized_levenshtein >= 0.85
- `prompt_echo_eed_below_threshold` : similarity 0.5 → pass
- `filter_clean_output_passes` : normal text → Ok
- `filter_invisible_plus_echo_cascade` : invisible + no echo → InvisibleText

### §B.5 Delta tests attendu

- +10 tests output_filter.rs
- Existants inchanges

### §B.6 Verification Phase B

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- Full fail-fast 3 blocs

---

## Phase C — Guardrails pipeline Rust + wire

### §C.1 Module guardrails.rs

1. Creer `crates/nexus-coordinator-rs/src/guardrails.rs`.

2. **Types** :
   ```rust
   pub enum GuardrailDirection {
       Input,
       Output,
   }

   pub enum GuardrailOutcome {
       Pass,
       Flag { reason: String },
       Tripwire { reason: String },
   }

   pub struct GuardrailContext<'a> {
       pub system_prompt: &'a str,
       pub user_prompt: &'a str,
       pub model_output: &'a str,
   }

   pub trait Guardrail: Send + Sync {
       fn name(&self) -> &str;
       fn direction(&self) -> GuardrailDirection;
       fn check(&self, ctx: &GuardrailContext<'_>) -> GuardrailOutcome;
   }
   ```

3. **GuardrailChain** :
   ```rust
   pub struct GuardrailChain {
       guardrails: Vec<Box<dyn Guardrail>>,
   }

   impl GuardrailChain {
       pub fn new() -> Self;
       pub fn add(mut self, g: Box<dyn Guardrail>) -> Self;
       pub fn run(&self, ctx: &GuardrailContext<'_>) -> ChainResult;
   }

   pub struct ChainResult {
       pub passed: bool,
       pub flags: Vec<String>,
       pub tripwire: Option<String>,
   }
   ```

4. **OutputSafetyGuardrail** :
   Adapter wrapping `OutputFilter` → `impl Guardrail`.
   `check()` appelle `output_filter.filter()` et mappe
   `is_valid=false` → `Tripwire`, `is_valid=true` → `Pass`.

### §C.2 Wire dans submit_result

1. Modifier `coordinator_submit_result` dans `http.rs` :
   - Apres `validate_result()` retourne `Accepted` et AVANT
     `kudos_ledger::credit()` :
   - Construire `GuardrailContext` avec system_prompt (vide
     pour l'instant, le ResultEntry n'a pas de prompt),
     user_prompt (vide), model_output = `entry.payload.result_text`.
   - Appeler `guardrail_chain.run(&ctx)`.
   - Si `tripwire` → rejecter, pas de credit, log warn.
   - Si `flags` → log info, credit quand meme.
   - Si `passed` → credit normal.

2. Ajouter `guardrail_chain: Arc<GuardrailChain>` a
   `DaemonHttpState` (ou construire inline — a evaluer).

3. Wire identique dans le `validator_loop` (Phase A) si deja
   merge. Sinon, ajouter dans le loop body.

### §C.3 Wire dans lib.rs

1. Ajouter `pub mod guardrails;` dans lib.rs.

### §C.4 Tests

- `chain_empty_passes` : chain vide → Pass
- `chain_pass_through` : 1 guardrail Pass → passed=true
- `chain_flag_accumulates` : 2 guardrails Flag → passed=true, 2 flags
- `chain_tripwire_short_circuits` : Tripwire → passed=false, stop
- `output_safety_guardrail_passes_clean` : clean output → Pass
- `output_safety_guardrail_trips_on_invisible` : invisible → Tripwire

### §C.5 Delta tests attendu

- +6 tests guardrails.rs
- Existants inchanges

### §C.6 Verification Phase C

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo nextest run -p nexus-shell-daemon --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- `cargo build -p nexus-shell-daemon --release`
- Full fail-fast 3 blocs

---

## Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint39_audit_plan.md
- SPRINT_LOG.md row S38
- CLAUDE.md etat actuel (compteurs, carries, etat)
- HARDENING_ROADMAP.md compteurs finaux + last_validated S38
- Migration `.planning/active/sprint37_audit_findings.md` +
  `.planning/active/sprint38_audit_plan.md` →
  `.planning/archive/v1.2/`
- Commit : `chore(sprint38): Phase D — wrap-up + verification
  + audit plan S39 + migration`

---

## §5 Research consulte

- **iroh-docs 0.98** : pinne workspace. CuratorRuntimeHandle
  (iroh_runtime.rs) expose DashMap snapshots. LiveEvents via
  context7 : Watcher pattern (stream asynchrone, cancel-safe).
  Le gossip subscribe loop existe deja dans CuratorRuntime —
  le broadcast channel s'y greffe.
- **strsim 0.11** (crates.io) : pure Rust, MIT, 0 dep. Fournit
  `normalized_levenshtein()` (0.0-1.0 similarity score).
  Alternative `edit-distance` plus simple mais pas de normalized.
  Alternative `rapidfuzz` Rust binding mais FFI overhead.
- **tokio::sync::broadcast** : bounded multi-consumer channel.
  Capacity 64 suffisant (debit resultats << 64/s pre-v1.0).
  `RecvError::Lagged(n)` = events manques, log warn + continue.
