# Sprint 39 — Plan

**Kickoff ref** : `sprint39_kickoff.md` (D1..D5 gelees).
**Phases** : A (PiiRedactor), B (CanaryRegistry), C (wire + P2),
D (wrap-up).

---

## Phase A — PiiRedactor Rust regex-only

### §A.1 Dependencies

1. Ajouter `regex = "1"` au `[workspace.dependencies]` dans
   `Cargo.toml` racine.
2. Ajouter `regex = { workspace = true }` a
   `crates/nexus-coordinator-rs/Cargo.toml`.

### §A.2 Module pii_redactor.rs

1. Creer `crates/nexus-coordinator-rs/src/pii_redactor.rs`.

2. **Types** :
   ```rust
   #[derive(Debug, Clone, Deserialize)]
   pub struct RedactionPolicy {
       pub enabled_patterns: Vec<String>,
       pub replacement: String,
   }

   pub struct PiiRedactor {
       patterns: Vec<(String, Regex)>,
       replacement: String,
   }
   ```

3. **Regex patterns** (compiles une fois via `OnceLock` ou
   construction directe) :
   - `email` : `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}`
   - `phone_us` : `\b\d{3}[-.]?\d{3}[-.]?\d{4}\b`
   - `phone_intl` : `\+\d{1,3}[\s-]?\d{4,14}`
   - `ssn` : `\b\d{3}-\d{2}-\d{4}\b`
   - `credit_card` : `\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b`
     (+ Luhn validation post-match)
   - `ipv4` : `\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b`
   - `ipv6` : pattern abrege pour les formes courantes

4. **Luhn validation** :
   ```rust
   fn luhn_valid(number: &str) -> bool {
       // Port direct du Python _luhn_valid()
   }
   ```

5. **PiiRedactor API** :
   ```rust
   impl PiiRedactor {
       pub fn new(policy: &RedactionPolicy) -> Self;
       pub fn redact(&self, text: &str) -> String;
       pub fn has_pii(&self, text: &str) -> bool;
   }
   ```

6. **Policy hot-reload** :
   Pattern identique a OutputFilter. `RedactionPolicy::from_toml()`.
   Default policy : tous les patterns actifs, replacement `[REDACTED]`.

### §A.3 PiiInputGuardrail adapter

1. Dans `pii_redactor.rs` :
   ```rust
   pub struct PiiInputGuardrail {
       redactor: PiiRedactor,
   }

   impl Guardrail for PiiInputGuardrail {
       fn name(&self) -> &str { "pii_input" }
       fn direction(&self) -> GuardrailDirection { GuardrailDirection::Input }
       fn check(&self, ctx: &GuardrailContext<'_>) -> GuardrailOutcome {
           // Scan user_prompt for PII
           if self.redactor.has_pii(ctx.user_prompt) {
               GuardrailOutcome::Tripwire { reason: "PII detected in input".into() }
           } else {
               GuardrailOutcome::Pass
           }
       }
   }
   ```

### §A.4 Wire dans lib.rs

1. Ajouter `pub mod pii_redactor;` dans lib.rs.

### §A.5 Tests

- `luhn_valid_visa` : 4111111111111111 → true
- `luhn_invalid` : 1234567890123456 → false
- `redact_email` : "contact me at user@test.com" → `[REDACTED]`
- `redact_phone` : "call 555-123-4567" → `[REDACTED]`
- `redact_ssn` : "SSN 123-45-6789" → `[REDACTED]`
- `redact_credit_card` : "card 4111 1111 1111 1111" → `[REDACTED]`
  (Luhn valid)
- `no_redact_non_luhn` : "number 1234 5678 9012 3456" → unchanged
  (Luhn invalid)
- `redact_ipv4` : "server at 192.168.1.1" → `[REDACTED]`
- `has_pii_true` : text with email → true
- `has_pii_false` : clean text → false
- `pii_input_guardrail_passes_clean` : clean → Pass
- `pii_input_guardrail_trips_on_pii` : email → Tripwire

### §A.6 Delta tests attendu

- +12 tests pii_redactor.rs
- Existants inchanges

### §A.7 Verification Phase A

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- Full fail-fast 3 blocs

---

## Phase B — CanaryRegistry Rust

### §B.1 Module canary_registry.rs

1. Creer `crates/nexus-coordinator-rs/src/canary_registry.rs`.

2. **Types serde** :
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct CanaryObservation {
       pub pubkey_hex: String,
       pub signed_date: String, // ISO 8601 date
       pub observed_at: String, // ISO 8601 datetime
       pub signature_hex: String,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct DuressAckObservation {
       pub pubkey_hex: String,
       pub ack_date: String,
       pub observed_at: String,
   }

   #[derive(Debug, Clone, Default, Serialize)]
   pub struct CanaryFreshness {
       pub pubkey_hex: String,
       pub canary_status: String,
       pub canary_age_days: Option<i64>,
       pub duress_ack_status: String,
       pub duress_ack_age_days: Option<i64>,
   }

   #[derive(Debug, Clone, Default, Serialize)]
   pub struct NetworkHealth {
       pub total_keys: usize,
       pub fresh: usize,
       pub aging: usize,
       pub stale: usize,
       pub expired: usize,
       pub duress_detected: usize,
   }
   ```

3. **CanaryRegistry** :
   ```rust
   pub struct CanaryRegistry {
       canaries: HashMap<String, CanaryObservation>,
       duress_acks: HashMap<String, DuressAckObservation>,
       persist_path: PathBuf,
   }
   ```
   - `new(persist_path: PathBuf) -> Self` (+ load_if_exists)
   - `observe_canary(&mut self, obs: CanaryObservation)`
   - `observe_duress_ack(&mut self, obs: DuressAckObservation)`
   - `freshness(&self, pubkey_hex: &str) -> CanaryFreshness`
   - `network_health(&self) -> NetworkHealth`
   - `known_pubkeys(&self) -> Vec<String>`
   - `persist(&self) -> io::Result<()>` (write tmp + rename)

4. **Classification** :
   - `classify_canary_age(days: i64)` : <7 "fresh", <14 "aging",
     <30 "stale", ≥30 "expired"
   - `classify_duress_age(days: i64)` : <1 "recent", <7 "aging",
     ≥7 "stale"

5. **Coerce functions** :
   - `coerce_canary_payload(payload: &serde_json::Value) -> Result<CanaryObservation>`
   - `coerce_duress_ack_payload(payload: &serde_json::Value) -> Result<DuressAckObservation>`

### §B.2 Wire dans lib.rs

1. Ajouter `pub mod canary_registry;` dans lib.rs.

### §B.3 Tests

- `observe_canary_stores_latest` : 2 observations meme pubkey →
  derniere gagne
- `observe_duress_ack_stores_latest` : idem
- `freshness_fresh` : observation <7j → "fresh"
- `freshness_expired` : observation >30j → "expired"
- `freshness_unknown_key` : pubkey absente → "unknown"
- `network_health_mixed` : 3 keys differents ages → compteurs
  corrects
- `persist_and_reload` : write + new instance → memes donnees
- `coerce_canary_payload_valid` : JSON valide → Ok
- `coerce_canary_payload_missing_field` : JSON incomplet → Err

### §B.4 Delta tests attendu

- +9 tests canary_registry.rs
- Existants inchanges

### §B.5 Verification Phase B

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- Full fail-fast 3 blocs

---

## Phase C — Wire integration + P2 batch

### §C.1 Wire PiiInputGuardrail dans guardrails.rs

1. Ajouter `default_input_chain()` dans `guardrails.rs` :
   ```rust
   pub fn default_input_chain() -> GuardrailChain {
       GuardrailChain::new()
           .push(Box::new(PiiInputGuardrail::default()))
   }
   ```

2. Import `PiiInputGuardrail` depuis `pii_redactor`.

### §C.2 Wire dans submit_task handler

1. Modifier `coordinator_submit_task` dans `http.rs` :
   - AVANT dispatch : construire `GuardrailContext` avec
     user_prompt = task input.
   - Appeler `default_input_chain().run(&ctx)`.
   - Si `tripwire` → return 400 rejected (PII detected).
   - Si `flags` → log info, dispatch quand meme.
   - Si `passed` → dispatch normal.

### §C.3 CanaryRegistry HTTP routes

1. Ajouter `canary_registry: Arc<Mutex<CanaryRegistry>>` a
   `DaemonHttpState`.

2. Init dans `runtime.rs` : `CanaryRegistry::new(data_dir.join("canary_registry.json"))`.

3. Routes :
   - `POST /api/canary/observed` : deserialise body →
     `coerce_canary_payload()` → `observe_canary()` → persist.
   - `GET /api/canary/network-health` : `network_health()` → JSON.
   - `GET /api/canary/freshness/:pubkey` : `freshness(pubkey)` → JSON.

4. Poison guard identique aux handlers coordinator existants.

### §C.4 P2 batch

1. P2-REVIEW-A-1-S37 launcher logging test 2/3 :
   - Investiguer le test existant `launcher_log_dir_matches_daemon_log_dir`
     dans `crates/nexus-launcher/src/main.rs`.
   - Si le test couvre deja l'invariant complet (launcher + daemon
     utilisent le meme `~/.sbfb/logs/` path), documenter comme resolu.
   - Si partiel, completer le test ou ajouter un test complementaire.

### §C.5 Tests

- `input_chain_passes_clean` : clean input → passed=true
- `input_chain_trips_on_email` : email in input → tripwire
- `canary_observed_endpoint` : POST valid → 200
- `canary_network_health_endpoint` : GET → 200 + JSON
- `canary_freshness_endpoint` : GET /:pubkey → 200 + JSON

### §C.6 Delta tests attendu

- +5 tests (guardrails + canary HTTP)
- Existants inchanges

### §C.7 Verification Phase C

- `cargo nextest run -p nexus-coordinator-rs --locked`
- `cargo nextest run -p nexus-shell-daemon --locked`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo fmt --all --check`
- `cargo build -p nexus-shell-daemon --release`
- Full fail-fast 3 blocs

---

## Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint40_audit_plan.md
- SPRINT_LOG.md row S39
- CLAUDE.md etat actuel (compteurs, carries, etat)
- HARDENING_ROADMAP.md compteurs finaux + last_validated S39
- Migration `.planning/active/sprint39_audit_plan.md` →
  `.planning/archive/v1.2/`
- Commit : `chore(sprint39): Phase D — wrap-up + verification
  + audit plan S40 + migration`

---

## §5 Research consulte

- **regex 1.x** (crates.io) : crate standard Rust, MIT/Apache-2.0,
  0 dep unsafe, compile rapide. Pattern identique a re Python.
  La lib compile les regex en bytecode, pas en native code (sauf
  opt-in regex-automata).
- **time 0.3** : deja dep coordinator-rs. `OffsetDateTime::now_utc()`
  pour horodatage canary. `Date` pour age computation.
- **Luhn algorithm** : validation standard ISO/IEC 7812 pour cartes
  de credit. Implementation triviale, pas de dep externe.
- **PII regex patterns** : patterns standard tires de Presidio
  (Microsoft), Google DLP, et AWS Macie. Coverage : email (RFC 5322
  simplifie), phone US/intl, SSN, credit card, IP v4/v6.
