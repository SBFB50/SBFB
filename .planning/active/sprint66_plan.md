# Sprint 66 — Plan (Durabilite)

**Ecrit** : 2026-05-19.
**Tip master** : `a2fec86`.
**Roadmap** : Sprint 2/11, v2.1 Arc 1 Fondations (2/2).

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1333 | `cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest | 268 | `(cd web && npm run test:unit)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| **Total** | **~1607** | | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | iroh-docs persistence via data_dir | `runtime.rs` (with_data_dir), `boot_*_namespace` (fallback robustesse) |
| D2 | iroh-blobs FsStore activation | `node.rs` (enum BlobStore), `blobs.rs` (BlobsClient &Store), `blob_serve.rs`, `deploy.rs`, `http.rs` |
| D3 | Feed republish au boot + feed_join handle fix | `runtime.rs` (republish bloc), `feed_sync.rs` (feed_join handle + shutdown) |
| D4 | Provenance 3 etats (MANDATORY P2-PROVENANCE-404-BRIDGE) | `http.rs` (get_provenance status), `useBridge.ts`, `BrowsedProject.tsx` |
| D5 | Verification cross-node (MANDATORY P2-VERIFY-LOCAL-KEY-ONLY) | `http.rs` (get_provenance node_id extraction) |

---

## §3 Graphe de dependances inter-phases

```
Phase A ──→ Phase B ──→ Phase C ──→ Phase D ──→ Phase E
(persist)   (dette)     (feed+      (orphan+    (E2E+
                        provenance) revocation) wrap-up)
```

- Phase B depend de A parce que la dette Phase B (THREAT_MODEL
  feed section, PATTERNS raw-op) est independante du code mais
  le pragma SQLite FULL doit etre teste apres le boot persistent
  Phase A.
- Phase C depend de A parce que le feed republish au boot
  presuppose que iroh-docs est persistent (data_dir active).
- Phase D depend de C parce que P2-ORPHAN-REPUBLISH-RECOVERY
  affine le republish delivre en Phase C (detection d'orphans
  specifique).
- Phase E depend de D parce que le test E2E restart valide
  l'ensemble des phases A-D.

---

## §4 Phase A — iroh data_dir + FsStore

### §4.1 Scope

Activer la persistence iroh dans le daemon shell : data_dir pour
iroh-docs (redb), FsStore pour iroh-blobs (redb). Le daemon boot
avec les memes namespaces et blobs apres restart.

### §4.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-core-rs/src/node.rs` | Enum `BlobStore { Mem(MemStore), Fs(FsStore) }`. `Node.blobs_store` passe de `MemStore` a `BlobStore`. `Node::blobs_store()` retourne `&Store` (via impl Deref sur BlobStore). `create_node_with_config` : si `data_dir.is_some()`, creer `FsStore::load(data_dir.join("blobs")).await?`. A shutdown, appeler `store.shutdown()` pour FsStore. |
| `crates/nexus-core-rs/src/blobs.rs` | `BlobsClient::new` prend `&'a Store` au lieu de `&'a MemStore`. |
| `crates/nexus-core-rs/src/lib.rs` | Re-export `BlobStore` enum si necessaire. |
| `crates/nexus-shell-daemon/src/runtime.rs` | `NodeConfig::default().with_secret_key(secret).with_data_dir(opts.paths.root.join("iroh"))` — ajout `with_data_dir`. |
| `crates/nexus-shell-daemon/src/runtime.rs` | `boot_feed_namespace` et `boot_storage_namespace` : fallback `list_docs()` si `create_doc` echoue avec "already exists" (migration zero→persistent). |
| `crates/nexus-shell-daemon-core/src/blob_serve.rs` | `BlobsClient::new(node.blobs_store())` — adaptation type (MemStore→Store). |
| `crates/nexus-shell-daemon/src/http.rs` | `BlobsClient::new(state.node.blobs_store())` — adaptation type. |
| Adaptation deploy.rs, autres | Meme pattern : `&MemStore` → `&Store`. |

### §4.3 Tests plan

1. `test_persistent_fsstore_survives_reboot` — boot un Node avec
   data_dir + FsStore, add_bytes "hello", shutdown, reboot avec
   meme data_dir, get_bytes retrouve "hello".
2. `test_data_dir_creates_iroh_subdir` — boot avec data_dir,
   verifie que `<data_dir>/iroh/` et `<data_dir>/blobs/` existent.
3. `test_memstore_still_works_without_data_dir` — boot un Node
   sans data_dir, add_bytes + get_bytes fonctionnent (regression
   guard).
4. `test_boot_storage_namespace_persistent_reopen` — boot daemon,
   creer storage namespace, shutdown, reboot, namespace rouvert
   (pas recree).
5. `test_boot_feed_namespace_persistent_reopen` — idem pour feed
   namespace.

### §4.4 Critere d'acceptation

```bash
cargo nextest run --workspace --locked    # tous verts
cargo nextest run -E 'test(persistent_fsstore)' -p nexus-core-rs  # PASS
cargo nextest run -E 'test(boot_.*namespace.*persistent)' -p nexus-shell-daemon  # PASS
```

### §4.5 Commit cible

`feat(persistence): Sprint 66 Phase A — iroh data_dir + FsStore`

Body : Contexte, Fichiers, Delta tests, Verification §7.4,
Scope cuts respectes (kickoff §7), G8 traceability,
Pre-launch protocol, Carry closure.

---

## §5 Phase B — Dette pair

### §5.1 Scope

Phase exclusivement dediee aux items differes (Regle 1 §6.2.1
sprint pair). 4 items : 2 du S65 audit, 1 carry 1/3, 1
amelioration SQLite.

### §5.2 Livrables

| Fichier | Changement |
|---|---|
| `docs/claude/README.md` | Section §4.1 : ajouter note que les deletions de source code doivent etre dans un commit `chore(cleanup)` ou dans le feat de la phase, pas dans un `chore(planning)`. CLOSE P2-S65-CHORE-MISCLASSIFIED. |
| `docs/rust/PATTERNS.md` | Ajouter pattern P{N} "Raw-op store+forward" : `FeedEntry.op: Value`, `try_parse_op()` pour typed access, `validate_feed_operation` accept-unknown pour forward compat, `op_type()` helper. Ref `public_feed.rs:67-117`. CLOSE P2-S65-RAWOP-PATTERN-UNDOC. |
| `docs/security/THREAT_MODEL.md` | Ajouter section "Feed surface" avec threats T-FEED-1 a T-FEED-4 (feed replay, rate-limit bypass, payload oversized, cross-author forgery). Ref `PUBLIC_FEED_SPEC.md §12 Security Considerations`. Carry P2-THREAT-MODEL-FEED-SURFACE 1/3→2/3. |
| `crates/nexus-coordinator-rs/src/db.rs` | Ajouter `conn.pragma_update(None, "synchronous", "FULL")?;` dans `CoordinatorDb::open()` apres le pragma WAL (l.218). |

### §5.3 Tests plan

1. `test_coordinator_db_synchronous_full` — ouvrir un
   CoordinatorDb, verifier via `PRAGMA synchronous` que la valeur
   est 2 (FULL).

### §5.4 Critere d'acceptation

```bash
cargo nextest run -E 'test(synchronous_full)' -p nexus-coordinator-rs  # PASS
grep -q "synchronous" crates/nexus-coordinator-rs/src/db.rs  # present
grep -q "T-FEED" docs/security/THREAT_MODEL.md  # present
grep -q "Raw-op" docs/rust/PATTERNS.md  # present
```

### §5.5 Commit cible

`feat(dette): Sprint 66 Phase B — dette pair + THREAT_MODEL feed + PATTERNS raw-op`

---

## §6 Phase C — Feed republish + provenance cross-node

### §6.1 Scope

Republish des entries feed SQLite vers iroh-docs au boot.
Fix du JoinHandle leak de feed_join. Resolution des deux
MANDATORY 3/3 (provenance 3 etats + verification cross-node).

### §6.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` | Apres `boot_feed_namespace` (l.612+), ajouter bloc republish : `replay_all()` depuis coordinator_db → `publish_feed_entry_to_docs()` pour chaque entry → log count. One-shot synchrone avant spawn HTTP server. |
| `crates/nexus-shell-daemon/src/feed_sync.rs` | `feed_join` : (1) stocker le JoinHandle dans un Vec partagee (Arc<Mutex<Vec<JoinHandle>>>), (2) ajouter un shutdown watch channel clone, (3) retourner 200 avec handle ID. |
| `crates/nexus-shell-daemon/src/runtime.rs` | Ajouter `feed_join_handles: Arc<Mutex<Vec<JoinHandle<()>>>>` au DaemonRuntime. Au shutdown, drain et join chaque handle. |
| `crates/nexus-shell-daemon/src/http.rs` | `get_provenance` : (1) sur Ok(None), retourner `{status: "absent", verified: false, record: null}` au lieu de 404. (2) sur Ok(Some), extraire pubkey depuis `record.node_id` (hex decode → [u8;32]) et passer a `verify_provenance`. Ajouter `status: "verified"` ou `status: "failed"` selon resultat. |
| `web/src/bridge/useBridge.ts` | `provenance_verify` : propager `status` depuis la reponse. Sur 404 (backward compat servers) : `status = "absent"`. |
| `web/src/pages/BrowsedProject.tsx` | Badge provenance : 4 etats visuels. `status === "absent"` → badge neutre "Provenance" (FileCheck). `status === "verified"` → "Signature verifiee" vert. `status === "failed"` → "Verification echouee" rouge. Loading inchange. |

### §6.3 Tests plan

1. `test_feed_republish_at_boot` — inserer 5 entries feed en
   SQLite, boot daemon, verifier que les entries sont presentes
   dans iroh-docs.
2. `test_feed_join_handle_tracked` — appeler feed_join, verifier
   que le handle est dans le Vec, signaler shutdown, verifier
   join.
3. `test_provenance_endpoint_absent_status` — query provenance
   pour un projet sans record, verifier `status: "absent"` et
   `verified: false` (pas 404).
4. `test_provenance_cross_node_verified` — generer provenance avec
   keypair A, query provenance sur un noeud avec keypair B,
   verifier `verified: true` (cross-node).
5. `test_provenance_cross_node_tampered` — generer provenance,
   tamper le record, verifier `verified: false` et
   `status: "failed"`.
6. Vitest : "badge shows 'Provenance' when status is absent" —
   mock provenance avec `{status: "absent"}`, verifier texte
   "Provenance".

### §6.4 Critere d'acceptation

```bash
cargo nextest run -E 'test(feed_republish)' -p nexus-shell-daemon  # PASS
cargo nextest run -E 'test(provenance_.*absent)' -p nexus-shell-daemon  # PASS
cargo nextest run -E 'test(provenance_cross)' -p nexus-shell-daemon  # PASS
(cd web && npm run test:unit)  # tous verts
```

### §6.5 Commit cible

`feat(feed+provenance): Sprint 66 Phase C — feed republish + provenance cross-node`

---

## §7 Phase D — Orphan recovery + RevocationCache persistence

### §7.1 Scope

P2-ORPHAN-REPUBLISH-RECOVERY et RevocationCache persistence
SQLite. Finalise la durabilite des caches critiques.

### §7.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` | Orphan detection : apres le republish Phase C, comparer les entries en SQLite (replay_all) vs entries en iroh-docs (get_many_by_prefix "feed/"). Les entries presentes en SQLite mais absentes en iroh-docs sont orphelines → republish. Tail-safe : skip les entries avec prev_hash invalide. |
| `crates/nexus-coordinator-rs/src/db.rs` | Migration M14 : table `key_rotations (id INTEGER PRIMARY KEY, old_pubkey TEXT NOT NULL, new_pubkey TEXT NOT NULL, timestamp INTEGER NOT NULL, transition_days INTEGER NOT NULL, signature TEXT NOT NULL, created_at TEXT DEFAULT CURRENT_TIMESTAMP)`. |
| `crates/nexus-coordinator-rs/src/db.rs` | `insert_key_rotation()` et `load_key_rotations()` methodes. |
| `crates/nexus-shell-daemon/src/runtime.rs` | Au boot apres coordinator_db open, appeler `load_key_rotations()` et populer le `RevocationCache` partage. |
| `crates/nexus-shell-daemon-core/src/key_rotation_handler.rs` | `handle_key_rotation` : apres insertion dans le cache, persister via `insert_key_rotation()` dans coordinator_db. |

### §7.3 Tests plan

1. `test_orphan_republish_recovery` — inserer entries en SQLite
   SANS les ecrire dans iroh-docs, boot daemon, verifier qu'elles
   apparaissent dans iroh-docs apres boot.
2. `test_key_rotation_persistence_survives_reboot` — appliquer une
   rotation de cle, shutdown, reboot, verifier que la
   RevocationCache contient la rotation.
3. `test_migration_m14_creates_key_rotations_table` — ouvrir un
   DB sans M14, verifier que la migration cree la table.

### §7.4 Critere d'acceptation

```bash
cargo nextest run -E 'test(orphan_republish)' -p nexus-shell-daemon  # PASS
cargo nextest run -E 'test(key_rotation_persistence)' -p nexus-shell-daemon  # PASS
cargo nextest run -E 'test(migration_m14)' -p nexus-coordinator-rs  # PASS
```

### §7.5 Commit cible

`feat(persistence): Sprint 66 Phase D — orphan recovery + RevocationCache SQLite`

---

## §8 Phase E — Test E2E restart + wrap-up

### §8.1 Scope

Tests E2E restart complet, verification.md, audit_plan S67,
compteurs CLAUDE.md, SPRINT_LOG.md.

### §8.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` ou `crates/nexus-test-harness` | Test E2E restart : DaemonHandle boot → deploy → feed insert → shutdown → reboot → verify persistence. |
| `.planning/active/sprint66_verification.md` | Self-report fail-fast. |
| `.planning/active/sprint67_audit_plan.md` | Plan audit S66. |
| `CLAUDE.md` | Compteurs, etat actuel, carries. |
| `docs/claude/SPRINT_LOG.md` | Row S66. |

### §8.3 Tests plan

1. `test_e2e_restart_full_cycle` — daemon boot → publish app →
   insert feed entry → subscribe curator → stop propre → restart
   → app accessible (blob persiste), feed entries presentes
   (SQLite + iroh-docs), curator subscription active, meme
   node_id. Gate SBFB_INTEGRATION.
2. `test_e2e_crash_recovery` — daemon boot → insert feed → drop
   sans shutdown (simule crash) → restart → feed intact.

### §8.4 Critere d'acceptation

```bash
cargo nextest run -E 'test(e2e_restart)' -p nexus-shell-daemon  # PASS
# Tous les fail-fast verts (cf. §10)
```

### §8.5 Commit cible

`docs(sprint66): Sprint 66 Phase E — E2E restart test + wrap-up`

---

## §9 Delta tests estime

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +5 | +0 | FsStore persistent, data_dir, namespace reopen |
| B | +1 | +0 | SQLite synchronous FULL |
| C | +5 | +1 | feed republish, feed_join handle, provenance absent/cross-node |
| D | +3 | +0 | orphan recovery, key_rotation M14, RevocationCache persistence |
| E | +2 | +0 | E2E restart, crash recovery |
| **Total** | **+16** | **+1** | |
| **Sortie estimee** | **1349** | **269** | **~1624** |

---

## §10 Fail-fast checklist

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1349 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 8 | Vitest | `(cd web && npm run test:unit)` | >= 269 |
| 9 | npm build | `(cd web && npm run build)` | ok |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 12 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 13 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical |
| 14 | FsStore persistent | `cargo nextest run -E 'test(persistent_fsstore)' -p nexus-core-rs` | PASS |
| 15 | namespace reopen | `cargo nextest run -E 'test(boot_.*namespace.*persistent)' -p nexus-shell-daemon` | PASS |
| 16 | SQLite sync FULL | `cargo nextest run -E 'test(synchronous_full)' -p nexus-coordinator-rs` | PASS |
| 17 | feed republish | `cargo nextest run -E 'test(feed_republish)' -p nexus-shell-daemon` | PASS |
| 18 | provenance absent | `cargo nextest run -E 'test(provenance_.*absent)' -p nexus-shell-daemon` | PASS |
| 19 | provenance cross | `cargo nextest run -E 'test(provenance_cross)' -p nexus-shell-daemon` | PASS |
| 20 | feed_join handle | `cargo nextest run -E 'test(feed_join_handle)' -p nexus-shell-daemon` | PASS |
| 21 | orphan recovery | `cargo nextest run -E 'test(orphan_republish)' -p nexus-shell-daemon` | PASS |
| 22 | RevocationCache | `cargo nextest run -E 'test(key_rotation_persistence)' -p nexus-shell-daemon` | PASS |
| 23 | E2E restart | `cargo nextest run -E 'test(e2e_restart)' -p nexus-shell-daemon` | PASS |
| 24 | THREAT_MODEL feed | `grep -q "T-FEED" docs/security/THREAT_MODEL.md` | present |
| 25 | PATTERNS raw-op | `grep -q "Raw-op" docs/rust/PATTERNS.md` | present |
| 26 | Vitest badge absent | `(cd web && npm run test:unit)` includes badge absent test | PASS |
| 27 | verification.md | `test -f .planning/active/sprint66_verification.md` | exists |
| 28 | audit_plan S67 | `test -f .planning/active/sprint67_audit_plan.md` | exists |

---

## §11 Scope cuts

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | CuratorVouched/CuratorDisendorsed implementation | S67 | Factory Foundation, pas durabilite |
| 2 | BuildQuorumReached feed implementation | S67+ | idem |
| 3 | Quarantine feed hot path | S67+ | glue code anti-spam post-Factory |
| 4 | Age witness gate feed admission | S67+ | idem |
| 5 | T1 CONFIRM_PROMPT complet (UI nonce) | post-pilote S69 | requiert integration UI React + nonce |
| 6 | SBFB.json v2 code implementation | S67 Phase A | S66 = persistence, pas manifest |
| 7 | node_id deprecation dans deploy.rs | S67 Phase A | S66 = persistence, pas deploy refactor |
| 8 | Factory template scaffold | S67 Phase B+ | S66 = persistence, pas Factory |
| 9 | Fuzzing cargo-fuzz/proptest | post-audit | audit prep, pas sprint |
| 10 | CLI verify-release | S67+ | UX enrichissement post-durabilite |
| 11 | VerificationDetail niveau 3 | S67+ | UI enrichissement post-durabilite |
| 12 | Playwright E2E tests re-ecriture | S69 | suppression S65, re-ecriture post-Factory |
| 13 | Feed format version bump | post-launch | pre-launch policy |
| 14 | Multi-curator trust overlay | S67 Phase D (stretch) | roadmap v3 stretch S67 |

---

## §12 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | FsStore refactor casse la compilation de 3+ crates downstream | Medium | High | Enum BlobStore isole le changement. Compilation incremental Phase A. Test d'integration dans test-harness |
| R2 | iroh-docs data_dir corrompu apres crash mid-write | Low | High | redb ACID copy-on-write. SQLite FULL pragma defense-in-depth |
| R3 | Feed republish au boot lent sur gros feed | Low | Medium | Feed < 100 entries. Log temps. Paginer si >1s (stretch S67) |
| R4 | feed_join handles s'accumulent sans limite | Medium | Medium | Rate-limiter feed_join (max 10). Cleanup periodique handles termines |
| R5 | Provenance cross-node : node_id hex invalide crash | Low | High | Pattern matching exhaustif, fallback verified:false |
| R6 | Phase B dette trop chargee | Low | Low | Items exclusivement documentaires, scope reduit |
| R7 | Migration boot_feed_namespace incompatible data_dir | Medium | Medium | Fallback list_docs() si create_doc echoue |

---

## §13 Checkpoint de cloture

- 28/28 fail-fast verts
- 5 commits feat + 0-1 commits fix
- verification.md + audit_plan S67 ecrits
- PATTERNS.md mis a jour (raw-op pattern Phase B)
- THREAT_MODEL.md mis a jour (feed section Phase B)
- Memory nexus_grid_pivot.md a jour (tip + compteurs + carries)
- SPRINT_LOG.md row S66 ajoutee
- CLAUDE.md carries et compteurs mis a jour
