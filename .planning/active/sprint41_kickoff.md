# Sprint 41 — Kickoff (Tier 4 infra batch → jalon "Python supprimable")

**Ecrit** : 2026-04-29 (session fraiche post-audit gate S40 `dacb7ce`).
**Type** : **sprint impair** — pas de phase dette obligatoire.
**Tip master d'entree** : `dacb7ce` (chore(planning) audit findings
S40 PASS).
**Phase 0 audit Sprint 40** : **DEJA JOUE** — findings dans
`.planning/archive/v1.2/sprint40_audit_findings.md` (verdict **PASS**,
0 P0/P1, 0 P2 nouveau, 4 carries confirmes).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-29) : HARDENING_ROADMAP last_validated
  `2026-04-29` (S40 CLOSED meme jour). 0 trigger actif.

  Triggers verifies :
  - iroh > 0.98 : pas de release 0.99 — NOT FIRED
  - arti-client > 0.41 : stable 0.41.0 inchange — NOT FIRED
  - frost-ed25519 > 3.0 : stable 3.0 inchange — NOT FIRED
  - wasmtime LTS : inchange — NOT FIRED
  - Tor PoW spec : inchange — NOT FIRED
  - NIST PQC FIPS : inchange — NOT FIRED

  **0 trigger actif.** Pas de pre-research supplementaire requise.

- **Technologies utilisees S41** :
  - `rusqlite` : deja dep directe workspace (0.36). Utilise pour les
    5 modules SQLite WAL (quarantine, upload, pow_counter,
    contributor_registry, invite). Pattern P39 singleton CoordinatorDb.
  - `toml` : deja dep directe coordinator-rs (ajoutee S40 Phase B).
    Utilise pour capability_store.rs (hot-reload TOML).
  - `sha2` : deja dep directe coordinator-rs (ajoutee S40 Phase C).
    Utilise pour capability_store.rs integrity hash.
  - `nexus-core-rs` : deja dep directe coordinator-rs. Remplace les
    appels PyO3 de contributor_registry.py et invite.py.
  - `chrono` : dep workspace existante (0.4). Utilisee pour le reset
    UTC quotidien de pow_counter (remplacement Python `datetime`).
  - Pas de nouvelle dep externe.

- **Roadmap reference** : `.planning/roadmap_v1_migration_rust.md`
  §S41 — "Infra batch (Tier 4) → jalon Python supprimable".

- **ROADMAP_COMMITMENTS check** (G7 Regle 3) :
  LT-1 Gini trigger, LT-2 Radicle, LT-3 app ecosystem, LT-4
  biometric, LT-5 redundancy persistence : tous requierent tag v1.0
  ou condition externe → aucun declenche.

---

## §1 Constat d'entree

### §1.1 D'ou on part

Sprint 40 CLOSED + audit gate PASS. 3 phases A-C livrees :
- Phase A : dette pair 5 P2/P3 resolus (result_event_tx wire +
  substring dead code + OnceLock chain singleton + 3 HTTP tests +
  lowercase doc P40)
- Phase B : CanaryInput Rust (canary_input.rs 835 LOC)
- Phase C : Tier 3 batch (redundancy + watermark_detector + rerun +
  honeypot, 4 modules)

Roadmap migration Python→Rust : Tier 1-3 complet (output_filter +
guardrails + pii_redactor + canary_registry + canary_input +
redundancy + watermark_detector + rerun + honeypot). Prochaine
etape = Tier 4 infra batch (7 modules, 1730 LOC Python).

### §1.2 Ancrage HARDENING_ROADMAP

last_validated : 2026-04-29 (S40 CLOSED). 0 trigger ACTIF.
Prochain trigger possible : iroh 0.99 quand publie.

### §1.3 Compteurs tests entree (tip `dacb7ce`)

| Suite | Count |
|---|---|
| Rust nextest | 1023 |
| Rust doctests | 0 pass (1 ignored) |
| SDK pytest | 195 (1 flaky Windows file-lock) |
| Coord pytest | 409 + 36 fail (PyO3 stale) + 6 skip |
| Gov pytest | 46 |
| Vitest | 267 |
| Playwright | 42 + 2 fail (env pre-existing) |
| size-limit | 7/7 |
| **Total** | **~2026** |

### §1.4 Pre-launch protocol policy (rappel)

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1.
Pas de tolerant decoder multi-version. Cf. CLAUDE.md.

---

## §2 Goal en une phrase

Le sprint **migre les 7 modules Tier 4** (quarantine_queue +
upload_queue + fairness + pow_counter + contributor_registry +
invite + capability_store) de Python vers Rust dans
nexus-coordinator-rs, atteignant le **jalon "Python supprimable"** :
toute la logique metier du coordinator est en Rust natif.
**Critere SMART : 28+ rows fail-fast verts au verification.md,
mesure binaire au Phase D wrap-up.**

---

## §3 Phase 0 — Audit gate Sprint 40

**DONE** — `dacb7ce`. Verdict PASS (0 P0/P1, 0 P2 nouveau, 4
carries confirmes). Cf. `.planning/archive/v1.2/
sprint40_audit_findings.md`.

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Tier 4 batch : port direct 7 modules dans coordinator-rs

**Retenu** : porter les 7 modules Python Tier 4 vers des modules
Rust dans `crates/nexus-coordinator-rs/src/`. Strategie port direct
identique aux Tiers 1-3 (S38-S40) :

(a) **fairness.rs** (62 LOC Python → ~50 LOC Rust) : 3 fonctions
    pures sans etat (`compute_gini`, `compute_top_k_share`,
    `compute_churn_rate`). Pas de dep, pas de DB.
(b) **pow_counter.rs** (132 LOC) : compteur per-(consumer, model)
    avec reset quotidien UTC. Schema SQLite `pow_task_counts` table
    dans CoordinatorDb. Utilise `chrono` pour le calcul du jour UTC
    (remplacement `datetime` Python).
(c) **contributor_registry.rs** (281 LOC) : registre d'attestations
    contributeur (Couche 2 Sybil gate). Schema SQLite
    `contributor_attestations` table. Remplace les appels PyO3
    (`build_contributor_attestation`) par appels directs
    `nexus-core-rs`.
(d) **invite.rs** (216 LOC) : ledger d'invitations (mint/revoke/
    list/decode). Schema SQLite `invites` table. Remplace les appels
    PyO3 (`mint_invite`, `decode_invite`) par appels directs
    `nexus-core-rs`.
(e) **capability_store.rs** (274 LOC) : capability toggles
    gate-off-by-default. Hot-reload depuis TOML file
    (`~/.sbfb/capabilities.toml`). Integrity hash SHA-256. Pattern
    hot-reload identique a CanaryInputPolicy S40 (mtime debounce).
(f) **quarantine_queue.rs** (369 LOC) : queue SQLite WAL pour
    messages gossip borderline. TTL-based sweep. Schema SQLite
    `quarantine_messages` table.
(g) **upload_queue.rs** (396 LOC) : queue d'uploads differres avec
    jitter cryptographique anti-correlation. Schema SQLite
    `delayed_uploads` table. Random delay via `rand` crate (deja
    dep workspace).

**Rejete** :
- Module fourre-tout `infra.rs` pour les 7 : trop heterogene
  (math pure vs SQLite vs TOML). 7 modules separes = testabilite.
- Crates separes par module : 62-396 LOC chacun, trop petit pour
  des crates independants. Pattern etabli S38-S40.
- Port async Rust (tokio channels) pour les queues : le coordinator-rs
  est synchrone (rusqlite). Ajouter tokio overhead = overengineering.
  Les background loops (sweep/flush) sont differees a la wire-up
  Tier 5.

**Implications code** :
- `crates/nexus-coordinator-rs/src/{fairness,pow_counter,contributor_registry,invite,capability_store,quarantine_queue,upload_queue}.rs` (7 NEW)
- `crates/nexus-coordinator-rs/src/lib.rs` (+7 pub mod)
- `crates/nexus-coordinator-rs/src/db.rs` (+5 tables schema init)
- `crates/nexus-coordinator-rs/Cargo.toml` (+chrono si pas deja dep)

### D2 — Schema extension : 5 tables dans CoordinatorDb existant

**Retenu** : etendre le `CoordinatorDb` singleton existant (P39)
avec 5 nouvelles tables creees par `CREATE TABLE IF NOT EXISTS`
dans `db::init_schema()`. Tables :

- `quarantine_messages(id, payload_json, received_at, ttl_secs, source_pubkey_hex)`
- `delayed_uploads(id, blob_hash, scheduled_at, delay_secs, status, created_at)`
- `pow_task_counts(consumer_id, model_id, day_utc, count)`
- `contributor_attestations(project_id, fingerprint, forge_url, commit_count, first_seen, last_seen, sig_type)`
- `invites(invite_id, project_id, minted_by_hex, minted_at, revoked_at, token_hex)`

**Rejete** :
- DB separee par module (`quarantine.db`, `uploads.db` etc.) :
  fragmentation des connexions, pas de transactions cross-module,
  complexite backup. Le singleton WAL gere bien 10+ tables.
- Framework de migration (diesel, refinery, sqlx) : overengineering
  pre-v1.0. `CREATE TABLE IF NOT EXISTS` est idempotent et suffisant
  tant qu'il n'y a pas de deployement live avec schema existant
  (pre-launch policy).
- Tables avec colonnes JSONB/JSON : les payloads sont courts et
  bien structures, les colonnes typees sont plus requetables.

**Implications code** :
- `crates/nexus-coordinator-rs/src/db.rs` (+5 CREATE TABLE)
- Chaque module `*.rs` prend `&CoordinatorDb` en parametre
  (pattern P39)

### D3 — Background loops differees (wire-up Tier 5)

**Retenu** : les background loops Python (quarantine TTL sweep,
upload flush timer) ne sont PAS demarrees en S41. Le sprint porte
la logique metier (structs + methods + tests) sans activer les
loops dans le daemon. Les queues sont instanciees dans
DaemonHttpState comme champs mais les threads de sweep ne sont
pas lances.

Rationale : les loops necessitent du wire-up dans le lifecycle
daemon (tokio::spawn, shutdown signal, graceful drain). Ce
wire-up est naturellement fait quand les routes HTTP Tier 5
sont portees (S42-44) et que le code est effectivement appele.
Demarrer des loops vides = dead code fonctionnel.

Ce choix est coherent avec S40 ou canary_input, redundancy, rerun,
honeypot, watermark_detector ont ete portes comme modules standalone
sans wire-up.

**Rejete** :
- Demarrer les loops au boot daemon avec des no-op bodies :
  dead code, consomme des threads, complexifie le shutdown.
- Reporter le port des queues a S42 (quand les loops sont
  pertinentes) : casse le jalon "Python supprimable" S41.

**Implications code** :
- `crates/nexus-shell-daemon/src/runtime.rs` (+7 champs dans
  DaemonHttpState init)
- Pas de tokio::spawn supplementaire en S41

### D4 — PyO3 → direct nexus-core-rs (invite + contributor_registry)

**Retenu** : remplacer les appels PyO3 (`nexus_core` package
Python) par des appels Rust directs a `nexus-core-rs`. Les
fonctions crypto concernees sont :

- `build_contributor_attestation` → `nexus_core_rs::sign_bytes` +
  struct canonical
- `mint_invite` → `nexus_core_rs::crypto::KeyPair::sign` +
  token construction
- `decode_invite` → `nexus_core_rs::crypto::verify` + token parsing

Le code Python importait le package PyO3 `nexus_core` qui est un
wrapper autour des memes fonctions Rust. L'intermediaire PyO3 n'a
plus de raison d'etre quand le code appelant EST Rust.

**Rejete** :
- Garder un intermediaire d'abstraction `crypto_helpers.rs` : les
  appels sont directs et simples (sign/verify), pas besoin d'une
  couche supplementaire.
- Wrapper les appels PyO3 dans le Rust via pyo3::Python::
  with_gil : absurde (Rust appelant Python appelant Rust).

**Implications code** :
- `crates/nexus-coordinator-rs/src/contributor_registry.rs` (import
  nexus_core_rs::crypto)
- `crates/nexus-coordinator-rs/src/invite.rs` (import
  nexus_core_rs::crypto)
- Pas de modification de nexus-core-rs lui-meme

### D5 — Scope cuts S41

1. **Wire HTTP handlers quarantine/upload/invite/etc.** — S42-44
   (Tier 5, quand les routes API sont portees)
2. **Background loops sweep/flush** — S42-44 (wire-up lifecycle
   daemon)
3. **Wire rerun/redundancy/canary_input dans dispatcher** — S42
   (inline dans handlers, pas de hook framework)
4. **canary_input HTTP routes** — S43 (api/canary.py Tier 5)
5. **@require_capability axum middleware** — S42 (quand les routes
   qui l'utilisent sont portees)
6. **Migration routes API** — S42-44 (Tier 5)
7. **Suppression coordinator Python** — S45
8. **CI multi-OS release** — S46
9. **VPS deployment** — S47
10. **Tag v1.0** — S48
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **CanaryInput mutation guardrail** — post-v1.0

---

**Acknowledged review findings (G1)** :

Scoring : D1 ✅, D2 ✅, D3 ⚠️, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 5).

D3 ⚠️ (1 finding) :
- Wire-up lifecycle loops (tokio::spawn, shutdown, drain) non
  explicitement documente comme dependance S42-44 Tier 5.
  **Accept** : precedent S40 confirme l'absence de risque dead-code.
  Le preflight S42 Phase A devra explicitement gater l'activation
  des loops sur le Tier 5 completion. Note ajoutee dans §6 carries
  S42 : "S42 preflight gate loop activation on Tier 5".

---

## §5 Plan Phase outline A..D

### Phase A — Petits modules (fairness + pow_counter)

**But** : migrer les 2 plus petits modules (194 LOC Python total),
etablir le pattern schema extension.
- fairness.rs : 3 fonctions pures (gini + top_k + churn_rate)
- pow_counter.rs : compteur quotidien SQLite + `chrono` UTC
- db.rs : +1 table `pow_task_counts` (fairness n'a pas de DB)
- Tests : 6-8 tests (3 fairness math + 3-5 pow_counter CRUD/reset)
- Commit : `feat(sprint41): Sprint 41 Phase A — fairness + pow_counter
  Rust`

### Phase B — Modules identite (contributor_registry + invite + capability_store)

**But** : migrer les 3 modules identity/access (771 LOC Python total).
- contributor_registry.rs : registre attestations SQLite + nexus-core-rs
  direct
- invite.rs : ledger invitations SQLite + nexus-core-rs crypto
- capability_store.rs : hot-reload TOML + SHA-256 integrity + struct
  CapabilitySet
- db.rs : +3 tables (attestations + invites + capabilities si
  applicable)
- Tests : 10-14 tests (registry CRUD + invite mint/revoke + capability
  load/toggle)
- Commit : `feat(sprint41): Sprint 41 Phase B — contributor_registry +
  invite + capability_store Rust`

### Phase C — Modules queue (quarantine_queue + upload_queue)

**But** : migrer les 2 modules queue (765 LOC Python total),
completant le Tier 4.
- quarantine_queue.rs : queue SQLite WAL + TTL sweep (methodes, pas
  background loop)
- upload_queue.rs : queue SQLite WAL + delay jitter + status tracking
  (methodes, pas background loop)
- db.rs : +2 tables (quarantine_messages + delayed_uploads)
- Tests : 8-12 tests (enqueue/flush/TTL/status transitions)
- Commit : `feat(sprint41): Sprint 41 Phase C — quarantine_queue +
  upload_queue Rust`

### Phase D — Wrap-up

- verification.md fail-fast 28+ rows
- sprint42_audit_plan.md
- SPRINT_LOG.md row S41
- CLAUDE.md etat actuel
- HARDENING_ROADMAP.md compteurs + last_validated S41
- Migration `.planning/active/sprint41_*` → `.planning/archive/v1.2/`
  (sauf sprint42 files)
- Commit : `chore(sprint41): Phase D — wrap-up + verification
  + audit plan S42 + counters`

---

## §6 Items carry/dette

### Resolus S41 (plan)

- [plan] quarantine_queue.py migration : Phase C
- [plan] upload_queue.py migration : Phase C
- [plan] fairness.py migration : Phase A
- [plan] pow_counter.py migration : Phase A
- [plan] contributor_registry.py migration : Phase B
- [plan] invite.py migration : Phase B
- [plan] capability_store.py migration : Phase B

### Carries confirmes S42

- [carry] P2-A-1 rand blocker upstream 6+/3 : blocker externe
  inchange. Exemption §6.2.1 blocker externe.
- [carry] P2-AUDIT-2-S35 pre-release transitives iroh : condition
  heritee pin 0.98.
- [carry] P2-REVIEW-A-1-S39 Tripwire vs Mutation 2/3 : trait
  extension post-v1.0.
- [carry] P2-REVIEW-B-1-S39 warn threshold 2/3 : seuil cadence
  post-v1.0.
- [carry] P2-REVIEW-B-1-S40 rand_range non-random 2/3 : rand
  crate usage post-v1.0.
- [carry] P2-REVIEW-C-1-S40 SHA-256 vs BLAKE3 2/3 : alignment
  post-v1.0.
- [carry] P3-REVIEW-A-2-S39 LOC kickoff 2/3 : cosmetic.
- [carry] P3-REVIEW-B-2-S39 persist error silent 2/3 : robustness
  post-v1.0.
- [carry] P3-AUDIT-A-1-S39 URL single-quote 2/3 : cosmetic.
- [carry] P3-REVIEW-B-1-S40 Manager multiple Mutex 2/3 : cleanup
  post-v1.0.
- [carry] P3-REVIEW-C-1-S40 rerun deterministic hash 2/3 : same
  pattern Phase B.

**Note S42 pair** : S42 est un sprint pair → phase dette
obligatoire. Les items P2 a 2/3 atteindront 3/3 au S42 carry
et devront etre integres dans la phase dette S42. Anticiper :
les items `rand_range`, `SHA-256 vs BLAKE3`, `warn threshold`,
`Tripwire vs Mutation` devront etre resolus en S42.

### Long-terme inchanges

- T-NN+2 iframe Rust-wasm (PATTERNS §P34)
- LT-2 Radicle sortie cap G7 (trigger tag v1.0)
- LT-3/LT-4 hors-sprint (post-v1.0)
- LT-5 redundancy persistence (reclassifie S26)

---

## §7 Scope cuts

1. **Wire HTTP handlers** — S42-44 (Tier 5 routes)
2. **Background loops sweep/flush** — S42-44 (lifecycle daemon)
3. **Wire rerun/redundancy/canary dans dispatcher** — S42
4. **canary_input HTTP routes** — S43 (api/canary.py)
5. **@require_capability middleware axum** — S42
6. **Migration routes API** — S42-44
7. **Suppression coordinator Python** — S45
8. **CI multi-OS release** — S46
9. **VPS deployment** — S47
10. **Tag v1.0** — S48
11. **Kudos debit/stake** — interdit (Day 0 #7)
12. **CanaryInput mutation guardrail** — post-v1.0

---

## §8 Tracabilite scope (S40 → S41)

| Item S40 carry / scope cut | Ou dans S41 |
|---|---|
| SC-3 quarantine_queue Rust (Tier 4) | §5 Phase C |
| SC-4 upload_queue Rust (Tier 4) | §5 Phase C |
| Wire rerun/redundancy dispatcher S41 | §7 SC-3 → S42 |
| Wire canary_input routes S41/S43 | §7 SC-4 → S43 |
| Migration routes API S42-44 | §7 SC-6 inchange |
| Suppression Python S45 | §7 SC-7 inchange |
| CI/VPS/v1.0 S46-48 | §7 SC-8/9/10 inchange |
| Tier 4 modules (fairness, pow_counter, etc.) | §5 Phases A-C |

---

## §9 Risk register

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| R1 | 7 modules en 3 phases = charge elevee | Medium | 3 plus petits modules < 150 LOC chacun (fairness 62, pow_counter 132). Charge concentree sur Phase B (771 LOC) et Phase C (765 LOC). Pattern port direct etabli S38-S40. |
| R2 | CoordinatorDb 5 tables supplementaires = schema bloat | Low | CREATE TABLE IF NOT EXISTS idempotent, WAL mode gere bien 15+ tables, pas de migration framework necessaire pre-v1.0. |
| R3 | contributor_registry PyO3 → nexus-core-rs API mismatch | Low | Les fonctions crypto (sign/verify) sont identiques des deux cotes. Appels directs plus simples que PyO3 wrapping. Testes par roundtrip sign/verify/tamper. |
| R4 | capability_store hot-reload pattern duplication avec canary_input S40 | Low | Meme pattern (mtime debounce + reload). Factoriser en helper commun si un 3e module l'utilise. S41 ne factorise pas (2 instances ≠ premature abstraction). |
| R5 | upload_queue CSPRNG jitter via `rand` crate alors que P2-A-1 rand blocker existe | Low | P2-A-1 concerne `rand::thread_rng()` qui necessite getrandom. upload_queue peut utiliser `rand::rngs::OsRng` ou le pattern SystemTime+hash S40. Adapter selon resolution P2-A-1. |

---

## §10 Audit gate pattern — rappel

Phase 0 (audit S40) deja jouee. Phase D devra produire
`sprint42_audit_plan.md` pour la session suivante.

---

## §11 Checkpoint de validation

1. **D1** : 7 modules en 3 phases, faisable ?
   → recommandation : oui, les 3 plus petits < 150 LOC, les
   modules Python sont bien structures (classes claires, patterns
   etablis). Total 1730 LOC Python, comparable a S40 (~1700 LOC).
2. **D2** : 5 tables dans CoordinatorDb singleton, acceptable ?
   → recommandation : oui, SQLite WAL gere bien 15+ tables. Pattern
   etabli P39. Le singleton evite la fragmentation.
3. **D3** : background loops differees, pas trop de dead code ?
   → recommandation : oui, coherent avec S40 (canary_input/rerun/
   etc. portes sans wire-up). Les methods sont testees unitairement.
   Le wire-up vient naturellement avec les routes Tier 5.
4. **D4** : PyO3 → direct, pas de regression ?
   → recommandation : oui, le code Rust appelle directement les
   memes fonctions crypto que le wrapper PyO3 wrappait.
5. **D5** : wire rerun/redundancy defer S42, acceptable ?
   → recommandation : oui, le wire-up necessite les routes HTTP Tier 5.
   S42 pair aura aussi une phase dette pour les items 2/3 → 3/3.
