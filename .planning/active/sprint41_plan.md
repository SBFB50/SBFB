# Sprint 41 — Plan d'execution

**Tip d'entree** : `dacb7ce` (audit findings S40 PASS).
**Version cible** : v1.2 (security hardening + migration Rust).
**Theme** : Tier 4 infra batch → jalon "Python supprimable".

---

## §1 Etat verifie a l'entree

| Suite | Count | Warnings |
|---|---|---|
| Rust nextest | 1023 | 0 |
| Rust doctests | 0 pass (1 ignored) | 0 |
| Rust clippy | 0 warnings | 0 |
| cargo fmt | OK | — |
| SDK pytest | 195 | 1 flaky (Windows file-lock) |
| Coord pytest | 409 + 36f + 6s | PyO3 stale |
| Gov pytest | 46 | 0 |
| Vitest | 267 | 0 |
| Playwright | 42 + 2f | env pre-existing |
| size-limit | 7/7 | 0 |

---

## §2 Decisions Day 0 (gelees — rappel synthetique)

- **D1** : port direct 7 modules Tier 4 dans coordinator-rs
- **D2** : extension CoordinatorDb +5 tables (CREATE IF NOT EXISTS)
- **D3** : background loops differees Tier 5 (methodes sans threads)
- **D4** : PyO3 → direct nexus-core-rs (invite + contributor_registry)
- **D5** : scope cuts 12 items (wire-up → S42-44, Python → S45, v1.0 → S48)

---

## §3 Research consulte

- **rusqlite 0.36** : dep workspace existante, WAL mode confirme
  fonctionnel (30+ tables en production S35-S40). Pattern P39
  CoordinatorDb singleton.
- **chrono 0.4** : dep workspace existante, `Utc::now().date_naive()`
  pour le reset quotidien pow_counter.
- **toml 0.8** : dep coordinator-rs existante (S40 Phase B). Utilise
  pour hot-reload capability_store.
- **sha2 0.10** : dep coordinator-rs existante (S40 Phase C). Utilise
  pour integrity hash capability_store.
- **nexus-core-rs crypto** : `KeyPair::generate()`, `sign()`,
  `verify()` — API stable depuis S14, utilisee par canary_input,
  honeypot, trust-web. Direct access sans PyO3.
- **rand crate** : dep workspace existante. `rand::thread_rng()` pour
  upload_queue jitter. Nota : P2-A-1 rand blocker upstream (getrandom
  Windows) — utiliser `OsRng` ou fallback SystemTime si rand build fail.

---

## §4 Dependencies inter-phases

```
Phase A (fairness + pow_counter)
  → Phase B utilise pow_counter comme reference pattern DB
Phase B (contributor_registry + invite + capability_store)
  → Phase C utilise contributor_registry comme reference pattern DB
Phase C (quarantine_queue + upload_queue)
  → autonome (utilise patterns A+B)
```

---

## Phase A — fairness + pow_counter (194 LOC Python)

### §A.1 Scope

Migrer `fairness.py` (62 LOC, 3 fonctions pures) et `pow_counter.py`
(132 LOC, compteur quotidien SQLite) vers Rust. Etablir le pattern
d'extension de schema CoordinatorDb.

**fairness.rs** :
- `compute_gini(values: &[f64]) -> f64` : coefficient de Gini sur un
  vecteur de kudos/contributions.
- `compute_top_k_share(values: &[f64], k_pct: f64) -> f64` : part des
  top k% dans le total.
- `compute_churn_rate(prev: &HashSet<String>, curr: &HashSet<String>) -> f64` :
  taux de churn entre deux periodes.

**pow_counter.rs** :
- `PowCounter` struct avec `CoordinatorDb` reference.
- `increment(&self, consumer_id: &str, model_id: &str) -> Result<u32>` :
  incremente le compteur du jour UTC courant.
- `get_count(&self, consumer_id: &str, model_id: &str) -> Result<u32>` :
  lit le compteur du jour.
- `reset_expired(&self) -> Result<usize>` : supprime les entrees
  anterieures au jour courant.
- Schema : `pow_task_counts(consumer_id TEXT, model_id TEXT,
  day_utc TEXT, count INTEGER, PRIMARY KEY(consumer_id, model_id,
  day_utc))`.

### §A.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/fairness.rs` | NEW — 3 fonctions pures |
| `crates/nexus-coordinator-rs/src/pow_counter.rs` | NEW — compteur quotidien SQLite |
| `crates/nexus-coordinator-rs/src/db.rs` | +1 table pow_task_counts |
| `crates/nexus-coordinator-rs/src/lib.rs` | +2 pub mod |
| `crates/nexus-coordinator-rs/Cargo.toml` | +chrono workspace dep |

### §A.3 Tests plan

1. `gini_uniform_distribution` — Gini = 0 pour valeurs egales
2. `gini_complete_inequality` — Gini = 1 pour une seule valeur non-zero
3. `gini_realistic_distribution` — Gini ~0.3-0.5 pour distribution
   realiste
4. `top_k_share_all_equal` — top 10% = 10% du total
5. `churn_rate_no_change` — churn = 0 si meme set
6. `churn_rate_total_change` — churn = 1 si sets disjoint
7. `pow_counter_increment_and_get` — increment + get roundtrip
8. `pow_counter_reset_expired` — reset supprime les anciens jours

### §A.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

### §A.5 Commit cible

```
feat(sprint41): Sprint 41 Phase A — fairness + pow_counter Rust

Migre fairness.py (62 LOC) et pow_counter.py (132 LOC) vers Rust.
fairness.rs : 3 fonctions pures (gini + top_k_share + churn_rate).
pow_counter.rs : compteur per-(consumer, model) quotidien UTC avec
schema SQLite pow_task_counts.

Delta tests : +8 (1023→1031)
Scope cuts respectes : 12/12 (§7)
```

---

## Phase B — contributor_registry + invite + capability_store (771 LOC Python)

### §B.1 Scope

Migrer les 3 modules identity/access. Le plus gros volume du sprint
(771 LOC Python). Points cles :

**contributor_registry.rs** (281 LOC Python) :
- `ContributorRegistry` struct avec `&CoordinatorDb`.
- `record_attestation(project_id, fingerprint, forge_url, commit_count, sig_type)` — upsert idempotent.
- `is_verified_contributor(fingerprint) -> bool` — check 2+ forges.
- `list_for_project(project_id) -> Vec<Attestation>`.
- Schema : `contributor_attestations(...)` table.
- Remplace `nexus_core` PyO3 → `nexus_core_rs::sign_bytes` direct.

**invite.rs** (216 LOC Python) :
- `InviteLedger` struct avec `&CoordinatorDb`.
- `mint(project_id, minted_by_keypair) -> InviteToken` — genere un
  token signe Ed25519.
- `decode(token_hex) -> InvitePayload` — decode et verifie signature.
- `revoke(invite_id)` — marque comme revoque.
- `list(project_id) -> Vec<Invite>`.
- Schema : `invites(...)` table.
- Remplace `nexus_core` PyO3 → `nexus_core_rs::crypto` direct.

**capability_store.rs** (274 LOC Python) :
- `CapabilityStore` struct avec path TOML.
- `load() -> CapabilitySet` — deserialise TOML + verifie SHA-256.
- `is_enabled(capability: &str) -> bool` — check gate.
- `enable/disable(capability)` — toggle + re-hash + write.
- Hot-reload pattern (mtime debounce, identique canary_input S40).
- Pas de DB, fichier TOML uniquement.

### §B.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/contributor_registry.rs` | NEW — registre attestations |
| `crates/nexus-coordinator-rs/src/invite.rs` | NEW — ledger invitations |
| `crates/nexus-coordinator-rs/src/capability_store.rs` | NEW — capability toggles TOML |
| `crates/nexus-coordinator-rs/src/db.rs` | +2 tables (attestations + invites) |
| `crates/nexus-coordinator-rs/src/lib.rs` | +3 pub mod |

### §B.3 Tests plan

1. `attestation_record_upsert` — insert + update idempotent
2. `attestation_multi_forge_verified` — 2+ forges → verified
3. `attestation_single_forge_not_verified` — 1 forge → not verified
4. `attestation_list_by_project` — filtrage par project_id
5. `invite_mint_decode_roundtrip` — mint → token hex → decode → verify
6. `invite_tampered_fails` — tampered token → decode error
7. `invite_revoke` — revocation + list confirms revoked
8. `invite_list_by_project` — filtrage par project_id
9. `capability_load_default` — fichier absent → tout desactive
10. `capability_toggle_enable_disable` — enable + disable roundtrip
11. `capability_sha256_integrity` — hash valide apres toggle
12. `capability_hot_reload_mtime` — reload detecte changement

### §B.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

### §B.5 Commit cible

```
feat(sprint41): Sprint 41 Phase B — contributor_registry + invite +
capability_store Rust

Migre contributor_registry.py (281 LOC), invite.py (216 LOC) et
capability_store.py (274 LOC) vers Rust. PyO3 remplace par appels
directs nexus-core-rs (sign/verify Ed25519). capability_store
hot-reload TOML + SHA-256 integrity.

Delta tests : +12 (1031→1043)
Scope cuts respectes : 12/12 (§7)
```

---

## Phase C — quarantine_queue + upload_queue (765 LOC Python)

### §C.1 Scope

Migrer les 2 modules queue, completant le Tier 4. Background loops
differees (D3 — methodes sans threads).

**quarantine_queue.rs** (369 LOC Python) :
- `QuarantineQueue` struct avec `&CoordinatorDb`.
- `enqueue(payload_json, source_pubkey_hex, ttl_secs)` — insert.
- `flush_expired() -> usize` — supprime les messages TTL expire.
- `pending_count() -> usize` — nombre de messages en attente.
- `drain(limit) -> Vec<QuarantineEntry>` — consomme N entrees.
- Schema : `quarantine_messages(id INTEGER PRIMARY KEY, payload_json
  TEXT, received_at REAL, ttl_secs REAL, source_pubkey_hex TEXT)`.

**upload_queue.rs** (396 LOC Python) :
- `UploadQueue` struct avec `&CoordinatorDb`.
- `schedule(blob_hash, delay_secs)` — insere avec status "pending".
- `ready_uploads() -> Vec<UploadEntry>` — entrees dont le delai est
  ecoule et status = "pending".
- `mark_done(id)` — status = "done".
- `mark_failed(id, reason)` — status = "failed".
- `compute_jitter(base_delay) -> f64` — random jitter
  anti-correlation (utilise rand ou SystemTime hash fallback).
- Schema : `delayed_uploads(id INTEGER PRIMARY KEY, blob_hash TEXT,
  scheduled_at REAL, delay_secs REAL, status TEXT, created_at REAL,
  error_reason TEXT)`.

### §C.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/quarantine_queue.rs` | NEW — queue quarantine SQLite |
| `crates/nexus-coordinator-rs/src/upload_queue.rs` | NEW — queue uploads differres |
| `crates/nexus-coordinator-rs/src/db.rs` | +2 tables (quarantine_messages + delayed_uploads) |
| `crates/nexus-coordinator-rs/src/lib.rs` | +2 pub mod |

### §C.3 Tests plan

1. `quarantine_enqueue_and_count` — enqueue + count = 1
2. `quarantine_flush_expired` — TTL expire → flush supprime
3. `quarantine_drain` — drain retourne et supprime les entrees
4. `quarantine_fresh_not_flushed` — TTL non expire → pas flush
5. `upload_schedule_and_ready` — schedule + delai ecoule → ready
6. `upload_not_ready_before_delay` — schedule + delai non ecoule → pas ready
7. `upload_mark_done` — status transition pending → done
8. `upload_mark_failed` — status transition pending → failed
9. `upload_jitter_range` — jitter dans bornes attendues
10. `upload_done_not_in_ready` — done entries excluses de ready

### §C.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo fmt --all --check
```

### §C.5 Commit cible

```
feat(sprint41): Sprint 41 Phase C — quarantine_queue + upload_queue
Rust

Migre quarantine_queue.py (369 LOC) et upload_queue.py (396 LOC) vers
Rust. Queues SQLite WAL avec TTL sweep + delay jitter. Background
loops differees Tier 5 (methodes standalone, pas de threads).
Tier 4 complet — jalon "Python supprimable" atteint.

Delta tests : +10 (1043→1053)
Scope cuts respectes : 12/12 (§7)
```

---

## §5 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | 1053+ pass | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | 0 pass (1 ignored) | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | ruff format | `uv run ruff format --check packages/` | OK | |
| 7 | ruff check | `uv run ruff check packages/` | OK | |
| 8 | SDK pytest | `uv run pytest packages/nexus-sdk/tests/ -q` | 195 | |
| 9 | Coord pytest | `uv run pytest packages/nexus-coordinator/tests/ -q` | 409+36f+6s | |
| 10 | Gov pytest | `uv run pytest packages/nexus-app-gov/tests/ -q` | 46 | |
| 11 | npm lint | `npm --prefix web run lint` | OK | |
| 12 | tsc | `npx --prefix web tsc --noEmit -p tsconfig.app.json` | OK | |
| 13 | Vitest | `npm --prefix web run test:unit` | 267 | |
| 14 | npm build | `npm --prefix web run build` | OK | |
| 15 | size-limit | `npm --prefix web run size` | 7/7 | |
| 16 | Phase A G8 | preflight EXECUTE | | |
| 17 | Phase A review | PASS | | |
| 18 | Phase B G8 | preflight EXECUTE | | |
| 19 | Phase B review | PASS | | |
| 20 | Phase C G8 | preflight EXECUTE | | |
| 21 | Phase C review | PASS | | |
| 22 | fairness.rs port | 3 functions pures | | |
| 23 | pow_counter.rs port | compteur quotidien SQLite | | |
| 24 | contributor_registry.rs port | registre attestations | | |
| 25 | invite.rs port | ledger invitations | | |
| 26 | capability_store.rs port | hot-reload TOML + SHA-256 | | |
| 27 | quarantine_queue.rs port | queue SQLite WAL | | |
| 28 | upload_queue.rs port | queue upload differee | | |
| 29 | Tier 4 complet 7/7 modules | jalon Python supprimable | | |
| 30 | Scope cuts respectes | 12/12 | | |
| 31 | Delta tests Phase A | +8 | | |
| 32 | Delta tests Phase B | +12 | | |
| 33 | Delta tests Phase C | +10 | | |
| 34 | Delta tests cumule | +30 (1023→1053) | | |

---

## §6 Git plan

1. `chore(planning): sprint 41 kickoff + plan + design review`
   (migration S40 → archive + kickoff + plan + design review)
2. `chore(planning): sprint 41 Phase A preflight G8`
3. `feat(sprint41): Sprint 41 Phase A — fairness + pow_counter Rust`
4. `chore(planning): sprint 41 Phase B preflight G8`
5. `feat(sprint41): Sprint 41 Phase B — contributor_registry + invite
   + capability_store Rust`
6. `chore(planning): sprint 41 Phase C preflight G8`
7. `feat(sprint41): Sprint 41 Phase C — quarantine_queue + upload_queue
   Rust`
8. `chore(sprint41): Phase D — wrap-up + verification + audit plan S42
   + counters`

---

## §7 Scope cuts (copie kickoff §D5)

1. Wire HTTP handlers → S42-44
2. Background loops sweep/flush → S42-44
3. Wire rerun/redundancy/canary dans dispatcher → S42
4. canary_input HTTP routes → S43
5. @require_capability middleware axum → S42
6. Migration routes API → S42-44
7. Suppression coordinator Python → S45
8. CI multi-OS release → S46
9. VPS deployment → S47
10. Tag v1.0 → S48
11. Kudos debit/stake → interdit (Day 0 #7)
12. CanaryInput mutation guardrail → post-v1.0

---

## §8 Risks

| # | Risque | Mitigation |
|---|---|---|
| R1 | 7 modules / 3 phases | Modules petits (62-396 LOC), pattern etabli |
| R2 | Schema bloat CoordinatorDb | WAL gere bien, CREATE IF NOT EXISTS |
| R3 | PyO3 → nexus-core-rs mismatch | Memes fonctions crypto, testes roundtrip |
| R4 | Hot-reload duplication canary_input | Acceptable (2 instances), factoriser a 3+ |
| R5 | rand blocker P2-A-1 upload_queue | Fallback SystemTime hash si necessaire |

---

## §9 Checkpoint de cloture

Le sprint est clos quand :
1. 7/7 modules Tier 4 portes et testes
2. CoordinatorDb etend avec 5 tables
3. 34/34 fail-fast rows vertes
4. Delta tests cumule +30 (1023→1053)
5. verification.md + sprint42_audit_plan.md produits
6. Jalon "Python supprimable" atteint
