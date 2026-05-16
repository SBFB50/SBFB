# Sprint 64 — Plan (hardening public cible)

**Ecrit** : 2026-05-16.
**Tip master** : `ebebe89`.
**Roadmap** : Sprint 4/6, v2.0 Public Verifiable Protocol Feed.

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande |
|---|---|---|
| Rust nextest | 1305 | `cargo nextest run --workspace --locked` |
| Rust doctests | ok (1 ignored) | `cargo test --workspace --locked --doc` |
| cargo fmt | 0 diff | `cargo fmt --all --check` |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Vitest | 265 | `(cd web && npm run test:unit)` |
| size-limit | 6/6 | `(cd web && npm run size)` |
| release build | ok | `cargo build -p nexus-shell-daemon --release` |
| **Total** | **~1576** | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | Tests adversariaux deterministes (pas fuzzing) | `public_feed.rs` tests + potentiel `adversarial_tests.rs` |
| D2 | 1 test E2E nouveau noeud (DaemonCluster) | `multi_daemon.rs` |
| D3 | M13 `app_version TEXT` nullable + SBFB.json field version | `db.rs`, `deploy.rs`, `http.rs`, `sbfb-bridge.js`, examples/ |
| D4 | Timeout 30s subscribe + retry backoff | `feed_sync.rs`, `multi_daemon.rs` |
| D5 | PUBLIC_FEED_SPEC.md §10/§11/§12 enrichissement | `docs/protocol/PUBLIC_FEED_SPEC.md` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (MANDATORY) — standalone, prerequis aucune autre
Phase B (dette)     — standalone, independant de A
Phase C (feed adv.) — depend de A (M13 schema present pour tests)
Phase D (crypto+E2E)— depend de C (patterns tests establis)
Phase E (doc+wrap)  — depend de C+D (resultats tests documente)
```

Phase A et B sont independantes — execution sequentielle mais pas
dependantes. C depend du schema final (M13 Phase A). D depend des
patterns tests etablis en C. E depend de C+D pour documenter les
scenarios couverts.

---

## §4 Phase A — MANDATORY 3/3 (F1 version + F5 timeout)

### §4.1 Scope

Resout les 2 items MANDATORY 3/3 :

**F1 P2-VERSION-NOT-STORED** :
- Ajouter field `version` dans SBFB.json schema (examples/)
- Migration M13 : colonne `app_version TEXT` nullable dans
  `provenance_records`
- `deploy.rs` : lire version depuis SBFB.json, passer a
  `insert_provenance_record()`
- `http.rs` : endpoint provenance retourne `app_version`
- `sbfb-bridge.js` : `provenance_get` retourne version

**F5 P2-IROH-INFRA-TIMEOUT** :
- `feed_sync.rs` : envelopper subscribe dans
  `tokio::time::timeout(Duration::from_secs(30))` avec retry
- Test SBFB_INTEGRATION : subscribe reconnecte apres timeout

### §4.2 Fichiers touches

| Fichier | Role |
|---|---|
| `examples/sbfb-explorer/SBFB.json` | Ajouter field `"version": "1.0.0"` |
| `examples/sbfb-ideas/SBFB.json` | Ajouter field `"version": "0.1.0"` |
| `crates/nexus-coordinator-rs/src/db.rs` | M13 migration ALTER TABLE + insert fn signature |
| `crates/nexus-shell-daemon/src/deploy.rs` | Lire version SBFB.json + passer a insert |
| `crates/nexus-shell-daemon/src/http.rs` | Endpoint provenance retourne app_version |
| `web/public/sbfb-bridge.js` | provenance_get inclut version |
| `crates/nexus-shell-daemon/src/feed_sync.rs` | Timeout wrapper subscribe + retry |
| `crates/nexus-coordinator-rs/tests/multi_daemon.rs` | Test stabilite subscribe timeout |

### §4.3 Tests plan

1. `test_insert_provenance_with_version` — insert avec version, select retourne version
2. `test_insert_provenance_without_version` — insert NULL version, backward safe
3. `test_provenance_endpoint_returns_version` — HTTP GET provenance inclut app_version
4. `test_subscribe_timeout_reconnects` — subscribe timeout simule → reconnexion auto (SBFB_INTEGRATION)

### §4.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(provenance)'
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(subscribe_timeout)' # si SBFB_INTEGRATION=1
cargo nextest run --workspace --locked  # 0 regression
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §4.5 Commit cible

```
feat(feed): Sprint 64 Phase A — MANDATORY version stored + subscribe timeout

F1 P2-VERSION-NOT-STORED (3/3 MANDATORY) CLOSED:
- M13 migration: ALTER TABLE provenance_records ADD app_version TEXT
- deploy.rs: read version from SBFB.json, pass to insert
- http.rs: provenance endpoint returns app_version field
- sbfb-bridge.js: provenance_get includes version
- examples/SBFB.json: add "version" field

F5 P2-IROH-INFRA-TIMEOUT (3/3 MANDATORY) CLOSED:
- feed_sync.rs: tokio::time::timeout(30s) on subscribe + retry
- multi_daemon.rs: test_subscribe_timeout_reconnects (SBFB_INTEGRATION)

Delta tests: Rust +4 (2 provenance version + 1 endpoint + 1 subscribe), Vitest +0.
Cumule sprint: Rust 1305 → 1309, Vitest 265, size 6/6.
Scope cuts respectes: 12/12 items non touches.
```

---

## §5 Phase B — Dette pair (5 items P2)

### §5.1 Scope

Sprint pair — phase dette obligatoire (§6.2.1 Regle 1). 5 items :

1. **P2-FEED-SUBSCRIBE-JOINHANDLE** (2/3) :
   `spawn_feed_subscribe()` retourne `JoinHandle<()>`.
   `DaemonRuntime` stocke `feed_handle: Option<JoinHandle<()>>`.
   `shutdown()` join le handle.

2. **P2-BACKFILL-6PLUS-TEST** (2/3) :
   Test integration : setup doc avec 8 feed entries → join →
   verify all ingested + rate limiter NOT applied.

3. **P2-FEED-PUBLISH-ORPHAN** (2/3) :
   Si `insert_feed_operation()` DB reussit mais iroh-docs insert
   echoue → rollback (DELETE row DB). Garantit atomicite.

4. **P2-SUBSCRIBE-STREAM-BREAK** (2/3) :
   Subscribe loop detect stream end (`None` de `stream.next()`) →
   log + backoff + re-subscribe. Pas de crash silencieux.

5. **P2-PROCESS-FORMAT** (herite) :
   Ajouter dans `docs/claude/README.md` §6.7 une clause exemption :
   "L'estimation LOC dans plan.md est un artefact de session ancienne.
   plan.md §X.2 NE DOIT PAS contenir d'estimation LOC. Les plans
   ecrits avant cette regle sont exemptes retroactivement."

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/feed_sync.rs` | JoinHandle return + stream break reconnexion |
| `crates/nexus-shell-daemon/src/runtime.rs` | feed_handle field + join shutdown |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | rollback orphan insert |
| `crates/nexus-coordinator-rs/tests/multi_daemon.rs` | test backfill 6+ entries |
| `docs/claude/README.md` | §6.7 exemption LOC |

### §5.3 Tests plan

1. `test_feed_subscribe_joinhandle_shutdown` — spawn subscribe → shutdown → handle joined (pas de leak)
2. `test_backfill_six_plus_entries` — 8 entries dans doc → join → 8 ingested (SBFB_INTEGRATION)
3. `test_feed_publish_orphan_rollback` — mock iroh-docs fail → DB row cleaned up
4. `test_subscribe_stream_break_reconnects` — stream returns None → re-subscribe after backoff

### §5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-shell-daemon --locked -E 'test(feed_subscribe)'
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(backfill|orphan)'
cargo nextest run --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §5.5 Commit cible

```
feat(feed+docs): Sprint 64 Phase B — dette pair 5 items P2

P2-FEED-SUBSCRIBE-JOINHANDLE (2/3) CLOSED:
- feed_sync.rs: spawn_feed_subscribe returns JoinHandle
- runtime.rs: feed_handle stored + joined at shutdown

P2-BACKFILL-6PLUS-TEST (2/3) CLOSED:
- multi_daemon.rs: test_backfill_six_plus_entries (8 entries, SBFB_INTEGRATION)

P2-FEED-PUBLISH-ORPHAN (2/3) CLOSED:
- public_feed.rs: rollback DB row on iroh-docs insert failure

P2-SUBSCRIBE-STREAM-BREAK (2/3) CLOSED:
- feed_sync.rs: stream None detection + backoff + re-subscribe

P2-PROCESS-FORMAT (herite) CLOSED:
- docs/claude/README.md §6.7: LOC estimation exemption clause

Delta tests: Rust +4 (1 joinhandle + 1 backfill + 1 orphan + 1 stream break), Vitest +0.
Cumule sprint: Rust 1309 → 1313, Vitest 265, size 6/6.
Scope cuts respectes: 12/12 items non touches.
```

---

## §6 Phase C — Tests adversariaux feed public

### §6.1 Scope

Tests deterministes couvrant les scenarios adversariaux feed :

1. Fork-bomb spam : injecter 1000 operations avec meme author →
   rate limiter rejette apres quota (5/min)
2. Payload oversized : operation avec payload > 64 KB → rejet
   validation
3. Mauvais repo URL : operation ReleasePublished avec repo_url
   invalide (non-HTTPS, caracteres speciaux) → rejet
4. Mauvais artifact hash : hash non-hex ou longueur incorrecte →
   rejet validation
5. Seq gap injection : entry avec seq_num qui saute (ex: 1,2,5) →
   hash-chain broken detection
6. Signature cross-author : entry signee par author A mais
   claiming author B → rejet verify_chain

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/public_feed.rs` | 6 tests adversariaux dans le module tests |
| `crates/nexus-coordinator-rs/src/feed_limiter.rs` | Confirmer que rate limiter rejette correctement |

### §6.3 Tests plan

1. `test_adversarial_fork_bomb_spam_rejected` — 1000 ops same author, verify <= 5 accepted
2. `test_adversarial_payload_oversized_rejected` — payload > 64KB, insert returns error
3. `test_adversarial_bad_repo_url_rejected` — non-HTTPS URL, validation fails
4. `test_adversarial_bad_artifact_hash_rejected` — hash malformed, validation fails
5. `test_adversarial_seq_gap_detection` — seq gap → verify_chain returns false
6. `test_adversarial_cross_author_forgery_rejected` — wrong signer detected

### §6.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(adversarial)'
cargo nextest run --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §6.5 Commit cible

```
feat(feed): Sprint 64 Phase C — adversarial tests feed public

6 tests adversariaux deterministes couvrant :
- Fork-bomb spam (1000 ops, rate-limited to 5/min)
- Payload oversized (> 64 KB rejected)
- Bad repo URL (non-HTTPS rejected)
- Bad artifact hash (malformed rejected)
- Seq gap injection (hash-chain break detected)
- Cross-author signature forgery (wrong signer rejected)

Delta tests: Rust +6, Vitest +0.
Cumule sprint: Rust 1313 → 1319, Vitest 265, size 6/6.
Scope cuts respectes: 12/12 items non touches.
```

---

## §7 Phase D — Tests adversariaux crypto + nouveau noeud E2E

### §7.1 Scope

**Tests crypto** : verifier les primitives cryptographiques
contre les vecteurs d'attaque pertinents au feed :
1. Ed25519 forgery sur FeedEntry (random bytes signature)
2. BLAKE3 tamper canonical bytes (1 bit flip → hash mismatch)
3. PoW nonce brute-force check (difficulty 16 bits → random nonce
   fails with overwhelming probability)
4. Age witness future timestamp reject (timestamp > now + 30j)

**Nouveau noeud E2E** :
- Daemon neuf (pas de state) → join reseau (2eme daemon publie
  3 operations feed) → sync feed entier → rebuild Browse via
  materializer → verify_chain sur le feed sync → valider
  curseur coherent.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/public_feed.rs` | 4 tests crypto adversariaux |
| `crates/nexus-coordinator-rs/tests/multi_daemon.rs` | 1 test E2E nouveau noeud |
| `crates/nexus-test-harness/src/lib.rs` | Helpers potentiels si necessaires |

### §7.3 Tests plan

1. `test_adversarial_ed25519_forgery_feed_entry` — random signature → verify fails
2. `test_adversarial_blake3_tamper_canonical` — 1 bit flip → hash mismatch
3. `test_adversarial_pow_nonce_difficulty_check` — random nonce vs 16-bit difficulty
4. `test_adversarial_age_witness_future_timestamp` — timestamp +31j → reject
5. `test_new_node_full_sync_and_verify` — daemon neuf → join → sync → rebuild → verify (SBFB_INTEGRATION)

### §7.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(adversarial)'
cargo nextest run -p nexus-coordinator-rs --locked -E 'test(new_node)' # si SBFB_INTEGRATION=1
cargo nextest run --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

### §7.5 Commit cible

```
feat(feed): Sprint 64 Phase D — adversarial crypto + new node E2E

4 tests crypto adversariaux :
- Ed25519 forgery on FeedEntry (random signature rejected)
- BLAKE3 canonical bytes tamper (1-bit flip detected)
- PoW nonce difficulty check (random nonce fails 16-bit)
- Age witness future timestamp (> 30d rejected)

1 test E2E nouveau noeud (SBFB_INTEGRATION) :
- Fresh daemon → join → sync full feed → rebuild Browse → verify chain → cursor valid

Delta tests: Rust +5 (4 crypto + 1 E2E), Vitest +0.
Cumule sprint: Rust 1319 → 1324, Vitest 265, size 6/6.
Scope cuts respectes: 12/12 items non touches.
```

---

## §8 Phase E — Documentation protocole + wrap-up

### §8.1 Scope

- PUBLIC_FEED_SPEC.md : ajouter §10 "Adversarial scenarios &
  mitigations" (table des 10+ vecteurs testes avec reference aux
  tests), §11 "New node bootstrap procedure" (algorithme :
  join → sync → verify → materialize → cursor), §12 "Security
  considerations" (resume threat model feed : PoW, rate-limit,
  quarantine, hash-chain integrity)
- verification.md : fail-fast checklist remplie
- sprint65_audit_plan.md : audit plan S65 (go-live public)
- CLAUDE.md : maj compteurs + etat
- SPRINT_LOG.md : entree S64

### §8.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/protocol/PUBLIC_FEED_SPEC.md` | §10, §11, §12 ajoutees |
| `.planning/active/sprint64_verification.md` | fail-fast rempli |
| `.planning/active/sprint65_audit_plan.md` | audit plan S65 |
| `CLAUDE.md` | Compteurs + etat courant |
| `docs/claude/SPRINT_LOG.md` | Entree S64 |

### §8.3 Tests plan

Pas de tests code — phase documentation.

### §8.4 Critere d'acceptation

```bash
# Toutes les suites vertes (verification finale)
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && npm run test:unit && npm run build && npm run size)
```

### §8.5 Commit cible

```
docs(protocol): Sprint 64 Phase E — spec finalisee + wrap-up

PUBLIC_FEED_SPEC.md enrichi :
- §10 Adversarial scenarios & mitigations (10+ vecteurs documentes)
- §11 New node bootstrap procedure (algorithme join→sync→verify)
- §12 Security considerations (threat model feed resume)

Sprint 64 livrables planning :
- verification.md fail-fast checklist remplie
- sprint65_audit_plan.md (theme : go-live public)
- CLAUDE.md + SPRINT_LOG.md a jour

Delta tests: Rust +0, Vitest +0.
Cumule sprint: Rust 1324, Vitest 265, size 6/6 — total ~1595.
Scope cuts respectes: 12/12 items non touches.
```

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1324 | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors | |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors | |
| 8 | Vitest | `(cd web && npm run test:unit)` | 265 | |
| 9 | npm build | `(cd web && npm run build)` | ok | |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 | |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean | |
| 12 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical | |
| 13 | M13 migration | SELECT app_version FROM provenance_records | column exists | |
| 14 | Adversarial feed | `cargo nextest run -E 'test(adversarial)' -p nexus-coordinator-rs` | 10+ PASS | |
| 15 | New node E2E | test_new_node_full_sync_and_verify | PASS (SBFB_INTEGRATION) | |
| 16 | Phase A-E preflights G8 | 5 fichiers sprint64_phase_{A..E}_preflight.md | 5x EXECUTE | |
| 17 | Phase A-E reviews | 5 fichiers sprint64_phase_{A..E}_review.md | 5x PASS | |
| 18 | PUBLIC_FEED_SPEC §10-12 | sections presentes et coherentes | ok | |

---

## §10 Git plan

| # | Commit | Phase |
|---|---|---|
| 1 | `feat(feed): Sprint 64 Phase A — MANDATORY version stored + subscribe timeout` | A |
| 2 | `feat(feed+docs): Sprint 64 Phase B — dette pair 5 items P2` | B |
| 3 | `feat(feed): Sprint 64 Phase C — adversarial tests feed public` | C |
| 4 | `feat(feed): Sprint 64 Phase D — adversarial crypto + new node E2E` | D |
| 5 | `docs(protocol): Sprint 64 Phase E — spec finalisee + wrap-up` | E |

---

## §11 Scope cuts (copie kickoff §7)

| # | Item | Sprint cible |
|---|---|---|
| 1 | CuratorVouched operation | S65 |
| 2 | BuildQuorumReached operation | S65 |
| 3 | Quarantine feed hot path | S65 |
| 4 | Age witness gate feed | S65 |
| 5 | Multi-forge feed sync | S65+ |
| 6 | Feed format version bump | post-launch |
| 7 | CLI verify-release | S65 |
| 8 | VerificationDetail niveau 3 | S65+ |
| 9 | Fuzzing cargo-fuzz/proptest | S65+ post-audit |
| 10 | Docker compose test distribue | S65+ |
| 11 | Interop externe parsers tiers | post-plan |
| 12 | SearchManifestPublished feed | S66 reserve |

---

## §12 Risks (R1..R7)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Tests adversariaux revelent bug verify_chain | Medium | High | Fix inline P1, pas scope cut |
| R2 | M13 migration casse provenance existantes | Low | High | Additive nullable, backward safe |
| R3 | Subscribe timeout faux positifs | Medium | Medium | 30s heartbeat, pas bulk |
| R4 | Nouveau noeud E2E instable timing | Medium | Medium | Gate SBFB_INTEGRATION + retry |
| R5 | Phase B dette > 1 phase | Low | Low | 5 items autonomes, S62 Phase A precedent OK |
| R6 | SPEC drift vs code | Low | Medium | Genere depuis noms de tests |
| R7 | FEED-INSERT-NO-AUTH-TIER 3/3 S65 | Certain | Low | Documente explicitement |

---

## §13 Checkpoint de cloture

1. 18/18 fail-fast rows vertes
2. 5 commits atomiques (A-E) en sequence
3. Sprint delta : Rust +19 (1305 → 1324), Vitest +0 (265)
4. 2 MANDATORY CLOSED (F1 + F5)
5. 5 items dette CLOSED (Phase B)
6. PUBLIC_FEED_SPEC.md 12 sections
7. sprint65_audit_plan.md ecrit
8. CLAUDE.md + SPRINT_LOG.md a jour
9. 12/12 scope cuts non touches
