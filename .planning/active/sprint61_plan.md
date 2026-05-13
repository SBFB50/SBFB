# Sprint 61 — Plan d'execution detaille

**Ecrit** : 2026-05-13 (post-kickoff).
**Source kickoff** : `.planning/active/sprint61_kickoff.md`
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md` Sprint 1

---

## §1 Etat verifie a l'entree

| Metrique | Valeur |
|---|---|
| Tip master | `32c07e2` |
| Rust nextest | 1259 pass, 0 fail |
| Rust doctests | 6 (1 ignored) |
| Vitest | 258 pass |
| Playwright | 0 (global-setup fail pre-existant) |
| size-limit | 6/6 |
| cargo fmt | 0 diff |
| cargo clippy | 0 warnings |
| Total | ~1523 |

---

## §2 Decisions Day 0 (gelees) — rappel synthetique

- **D1** : enum `PublicFeedOperation` avec `ReleasePublished` + `SourceBecameStale`
- **D2** : table SQLite `public_feed` dans coordinator.db (migration M9)
- **D3** : BLAKE3 hash-chain + Ed25519 signature + `DOMAIN_FEED_V1` + JCS
- **D4** : spec `docs/protocol/PUBLIC_FEED_SPEC.md` + versioning post-v1.0
- **D5** : `FeedMaterializer` → `PublicRegistryView` (source supplementaire, pas remplacement BrowseAggregator)

---

## §3 Research consulte

- Pattern hash-chain : `crates/nexus-coordinator-rs/src/kudos_ledger.rs`
  (BLAKE3, prev_hash genesis, canonical_bytes JCS, verify_chain)
- Pattern canonical domain : `crates/nexus-core-rs/src/canonical.rs`
  (14 domaines, `canonical_bytes_jcs()`, domain separation Ed25519)
- Pattern migration SQLite : `crates/nexus-coordinator-rs/src/db.rs`
  (8 migrations M1-M8, `MIGRATIONS` array, `rusqlite_migration`)
- Pattern gossip outbox : M6 table `gossip_outbox` (id AUTOINCREMENT,
  envelope BLOB, added_at INTEGER)
- Pattern BrowseAggregator : `crates/nexus-shell-daemon-core/src/browse.rs`
  (DashMap cache, `BrowseEntry`, probing TTL 60s)
- Recherche p2panda : `.planning/research/p2panda_public_protocol_briques.md`
  (11 operations specifiees, is_open_source validation rule)
- Roadmap : `.planning/research/public_verifiable_feed_roadmap.md`
  Sprint 1 phases A-D

### Graphe de dependances inter-phases

```
Phase A (types + spec) ──→ Phase B (store + hash-chain)
                                    │
                                    ↓
                              Phase C (materialisation + cursor)
                                    │
                                    ↓
                              Phase D (tests + wrap-up)
```

Phase A est prerequis pour B (types). B est prerequis pour C
(store API). D depend de B+C (tests E2E replay + materialisation).

---

## §4 Phase A — Spec executable + types Rust

### §4.1 Scope

Creer la spec executable `docs/protocol/PUBLIC_FEED_SPEC.md` qui
formalise le format du feed public SBFB. Definir les types Rust
correspondants dans un nouveau module `public_feed.rs` du crate
`nexus-coordinator-rs`. Ajouter le domaine `DOMAIN_FEED_V1` dans
`canonical.rs` pour la signature des operations.

La spec couvre :
- 2 operations Sprint 1 : `ReleasePublished`, `SourceBecameStale`
- 4 operations futures mentionnees : `CuratorVouched`,
  `BuildQuorumReached`, `SourceRecovered`, `SearchManifestPublished`
  (implementees Sprint 2+)
- Format canonique JCS/RFC8785 + domain separation
- Hash-chain BLAKE3 (genesis, construction, verification)
- Regles de replay (ordering, idempotence)
- Cursor format (seq + entry_hash checkpoint)
- Test vectors JSON
- Politique versioning post-v1.0 (`FEED_FORMAT_VERSION = 1`)

### §4.2 Fichiers touches

| Fichier | Role |
|---|---|
| `docs/protocol/PUBLIC_FEED_SPEC.md` | **NEW** — spec executable complète |
| `crates/nexus-core-rs/src/canonical.rs` | +constante `DOMAIN_FEED_V1` |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | **NEW** — types `PublicFeedOperation`, `FeedEntry`, `ReleasePublishedPayload`, `SourceBecameStalePayload`, `FeedEntryCanonical` |
| `crates/nexus-coordinator-rs/src/lib.rs` | +`pub mod public_feed;` |

### §4.3 Tests plan

1. `test_feed_operation_serde_roundtrip` — serialiser/deserialiser
   `ReleasePublished` et `SourceBecameStale`, verifier roundtrip
2. `test_canonical_bytes_feed_deterministic` — memes donnees →
   memes canonical bytes (JCS determinisme)
3. `test_feed_format_version` — `FEED_FORMAT_VERSION == 1`

### §4.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run -p nexus-coordinator-rs --locked
cargo nextest run -p nexus-core-rs --locked
```

Spec `docs/protocol/PUBLIC_FEED_SPEC.md` contient les 8 sections
listees dans D4. Au moins 1 test vector JSON dans la spec.

### §4.5 Commit cible

```
feat(feed): Sprint 61 Phase A — spec executable + types PublicFeedOperation

Spec protocol `docs/protocol/PUBLIC_FEED_SPEC.md` formalisee.
Types Rust : PublicFeedOperation enum (ReleasePublished +
SourceBecameStale), FeedEntry struct, canonical FeedEntryCanonical.
Domaine DOMAIN_FEED_V1 ajoute dans canonical.rs (15e domaine).
FEED_FORMAT_VERSION = 1 sous regime post-v1.0.

Delta tests : +3 Rust (1259 → 1262)
Scope cuts : 12/12 respectes
```

---

## §5 Phase B — Feed local append-only store

### §5.1 Scope

Implementer le `FeedStore` — stockage append-only SQLite avec
hash-chain BLAKE3. Migration M9 table `public_feed`. Methodes :
`insert_operation()` (persiste + maintient hash-chain),
`replay_all()` (relit toute la table), `verify_chain()` (verifie
chaque hash et signature depuis genesis), `get_latest_hash()`
(pour le chainage).

Le FeedStore est une brique independante qui ne connait pas le
BrowseAggregator. Phase C fera l'integration.

### §5.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` | Migration M9 : CREATE TABLE public_feed |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | +`FeedStore` struct : insert_operation, replay_all, verify_chain, get_latest_hash, load_cursor, save_cursor |

### §5.3 Tests plan

1. `test_insert_operation_persists` — inserer une operation,
   relire, verifier champs
2. `test_replay_all_ordered` — inserer 3 operations, replay_all
   retourne dans l'ordre seq
3. `test_hash_chain_valid` — inserer 3 operations, verify_chain
   retourne Ok
4. `test_hash_chain_genesis` — premiere entree a prev_hash zeros
5. `test_verify_chain_empty` — feed vide, verify_chain retourne Ok

### §5.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run -p nexus-coordinator-rs --locked
```

Toutes les methodes FeedStore fonctionnelles. Hash-chain BLAKE3
verifiable depuis genesis.

### §5.5 Commit cible

```
feat(feed): Sprint 61 Phase B — feed local SQLite append-only + hash-chain BLAKE3

Migration M9 table public_feed. FeedStore : insert_operation()
persiste avec hash-chain BLAKE3, replay_all() relit depuis genesis,
verify_chain() verifie chaque hash + signature Ed25519.

Delta tests : +5 Rust (1262 → 1267)
Scope cuts : 12/12 respectes
```

---

## §6 Phase C — Materialisation + cursor

### §6.1 Scope

`FeedMaterializer` lit le feed store et produit une
`PublicRegistryView` : ensemble de projets avec leur statut derive
des operations du feed. Le materializer maintient un cursor
(dernier seq traite) pour reprise apres interruption.

`PublicRegistryView` est une struct independante :
- `projects: HashMap<String, ProjectFeedStatus>` ou `ProjectFeedStatus`
  contient `published: bool`, `source_stale: bool`, dernier
  `release_hash`, `repo_url`, `timestamp` de derniere operation
- Materialisee en un seul pass lineaire sur le feed

Cursor :
- Persiste dans SQLite (table `feed_cursor` ou colonne dans
  `public_feed` metadata)
- Sauvegarde : `(last_seq, last_entry_hash)` — tuple checkpoint
- Reprise : materialiser depuis `last_seq + 1` si `last_entry_hash`
  correspond, sinon depuis 0 (safety)

### §6.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/feed_materializer.rs` | **NEW** — `FeedMaterializer`, `PublicRegistryView`, `ProjectFeedStatus` |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | +methodes cursor : `load_cursor()`, `save_cursor()` |
| `crates/nexus-coordinator-rs/src/db.rs` | +schema cursor si table separee |
| `crates/nexus-coordinator-rs/src/lib.rs` | +`pub mod feed_materializer;` |

### §6.3 Tests plan

1. `test_materialize_release_published` — 1 operation
   ReleasePublished → vue contient le projet comme publie
2. `test_materialize_source_stale` — ReleasePublished puis
   SourceBecameStale → projet marque source_stale
3. `test_cursor_persist_resume` — materialiser 3 ops, sauver cursor,
   ajouter 2 ops, materialiser depuis cursor → meme vue finale

### §6.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run -p nexus-coordinator-rs --locked
```

PublicRegistryView reconstructible depuis zero. Cursor reprise
fonctionnel (materialisation partielle + reprise = meme resultat
que materialisation complete).

### §6.5 Commit cible

```
feat(feed): Sprint 61 Phase C — materialisation PublicRegistryView + cursor persistant

FeedMaterializer lit le feed et produit PublicRegistryView
(HashMap projet → statut). Cursor persistant SQLite (last_seq,
last_entry_hash). Reprise apres interruption verifiee.

Delta tests : +3 Rust (1267 → 1270)
Scope cuts : 12/12 respectes
```

---

## §7 Phase D — Tests + wrap-up

### §7.1 Scope

Tests adversariaux basiques du feed local (hash-chain tamper,
transitions d'etat invalides, cursor restart). Verification.md +
audit_plan S62.

### §7.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/public_feed.rs` | +tests adversariaux hash-chain |
| `crates/nexus-coordinator-rs/src/feed_materializer.rs` | +tests transitions + cursor restart |
| `.planning/active/sprint61_verification.md` | **NEW** — self-report fail-fast |
| `.planning/active/sprint62_audit_plan.md` | **NEW** — plan audit S61 |
| `docs/claude/SPRINT_LOG.md` | +row S61 + header v2.0 |
| `docs/security/HARDENING_ROADMAP.md` | last_validated S61 |
| `CLAUDE.md` | compteurs mis a jour |

### §7.3 Tests plan

1. `test_chain_tamper_detect` — modifier un entry_hash, verify_chain
   detecte la corruption
2. `test_source_stale_without_release` — SourceBecameStale sans
   ReleasePublished precedent pour le meme projet → operation
   acceptee (le feed est append-only, pas un FSM strict)
3. `test_cursor_restart_consistency` — materialiser tout, sauver,
   re-materialiser depuis 0 → meme PublicRegistryView
4. `test_signature_verify_reject_forged` — signature invalide →
   verify_chain detecte

### §7.4 Critere d'acceptation

Full fail-fast :

```bash
# Rust
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release

# Frontend
cd web && npm install && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && \
  npm run build && npm run size
```

### §7.5 Commit cible

```
feat(feed): Sprint 61 Phase D — tests hash-chain tamper + transitions + cursor restart

Tests adversariaux : tamper detection, signature forgee rejetee,
cursor restart consistency, transitions d'etat. Verification.md
+ audit_plan S62. SPRINT_LOG.md + HARDENING_ROADMAP.md mis a jour.

Delta tests : +4 Rust (1270 → 1274)
Scope cuts : 12/12 respectes
```

---

## §8 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1274, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 258 | |
| 9 | npm build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean | |
| 12 | Phase A preflight G8 | sprint61_phase_A_preflight.md | EXECUTE | |
| 13 | Phase A review | sprint61_phase_A_review.md | PASS | |
| 14 | Phase B preflight G8 | sprint61_phase_B_preflight.md | EXECUTE | |
| 15 | Phase B review | sprint61_phase_B_review.md | PASS | |
| 16 | Phase C preflight G8 | sprint61_phase_C_preflight.md | EXECUTE | |
| 17 | Phase C review | sprint61_phase_C_review.md | PASS | |
| 18 | Phase D preflight G8 | sprint61_phase_D_preflight.md | EXECUTE | |
| 19 | Spec PUBLIC_FEED_SPEC.md | 8 sections + 1+ test vector | complete | |
| 20 | FeedStore functional | insert + replay + verify_chain | tests pass | |
| 21 | FeedMaterializer | PublicRegistryView from feed | tests pass | |
| 22 | Cursor restart | materialise from 0 = materialise from cursor | tests pass | |
| 23 | Hash-chain tamper | tamper → detect | tests pass | |
| 24 | Scope cuts | 12/12 respectes | all checked | |
| 25 | Delta tests cumule | documented in commit bodies | documented | |
| 26 | Sync bridge SDK | `bash scripts/sync-bridge-sdk.sh` | exit 0 | |

---

## §9 Git plan

| # | Commit | Scope |
|---|---|---|
| 1 | `chore(planning): Sprint 61 kickoff + plan + design review + S60 migration` | planning |
| 2 | `chore(planning): Sprint 61 Phase A preflight G8` | planning |
| 3 | `chore(planning): Sprint 61 Phase A review` | planning |
| 4 | `feat(feed): Sprint 61 Phase A — spec executable + types PublicFeedOperation` | code + spec |
| 5 | `chore(planning): Sprint 61 Phase B preflight G8` | planning |
| 6 | `chore(planning): Sprint 61 Phase B review` | planning |
| 7 | `feat(feed): Sprint 61 Phase B — feed local SQLite append-only + hash-chain BLAKE3` | code |
| 8 | `chore(planning): Sprint 61 Phase C preflight G8` | planning |
| 9 | `chore(planning): Sprint 61 Phase C review` | planning |
| 10 | `feat(feed): Sprint 61 Phase C — materialisation PublicRegistryView + cursor persistant` | code |
| 11 | `chore(planning): Sprint 61 Phase D preflight G8` | planning |
| 12 | `feat(feed): Sprint 61 Phase D — tests hash-chain + transitions + cursor restart` | code + docs + planning |

---

## §10 Scope cuts

Copie de kickoff §7 :

| # | Item | Sprint cible |
|---|---|---|
| 1 | Sync P2P durable (iroh-docs feed) | Sprint 62 |
| 2 | Anti-spam feed (PoW + rate-limit + quarantine) | Sprint 62 |
| 3 | CuratorVouched operation | Sprint 62+ |
| 4 | BuildQuorumReached operation | Sprint 62+ |
| 5 | Endpoint HTTP verify-release | Sprint 63 |
| 6 | Bridge methods provenance/verify | Sprint 63 |
| 7 | UI proof-chain VerificationDetail | Sprint 63 |
| 8 | Tests adversariaux complets | Sprint 64 |
| 9 | Go-live public + tag push | Sprint 65 |
| 10 | AppImage Linux | post-roadmap |
| 11 | Interop externe | post-roadmap |
| 12 | Audit tiers formel | post-roadmap |

---

## §11 Risks

Copie de kickoff §9. R4 (FeedMaterializer/BrowseAggregator couplage)
est le risque principal — mitigation : D5 fige l'integration comme
supplementaire. Phase C n'importe pas browse.rs.

---

## §12 Checkpoint de cloture

1. Spec PUBLIC_FEED_SPEC.md complete (8 sections + test vectors)
2. Types Rust PublicFeedOperation compiles + tests serde roundtrip
3. FeedStore insert/replay/verify fonctionnel avec hash-chain BLAKE3
4. FeedMaterializer produit PublicRegistryView coherente
5. Cursor restart = meme resultat que materialisation complete
6. Hash-chain tamper detecte
7. >= 1274 Rust tests (delta +15 minimum)
8. 0 diff fmt, 0 warnings clippy
9. verification.md + audit_plan S62 ecrits
10. SPRINT_LOG.md + CLAUDE.md + HARDENING_ROADMAP.md mis a jour
