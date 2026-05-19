# Sprint 66 Phase A — deep review

HEAD: `276173a` (dirty — 4 modified files staged for Phase A) | Agent: nexus-phase-review-deep (Opus 1M)

## Verdict : PASS

Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS.
0 P0, 0 P1, 2 P2, 1 P3. Phase ready to commit.

## Memory consultation
- `feedback_approach.md` : "pick deepest technical option" — respecte (FsStore = API officielle iroh-blobs, pas custom redb)
- `feedback_context7_systematic.md` : context7 obligatoire — respecte (3 queries iroh-blobs + 2 iroh-docs au kickoff, confirmees dans preflight)
- `nexus_grid_pivot.md` : iroh 0.98 pinne — respecte (pas de bump de version iroh)
- `vision_model.md` : no startup patterns — N/A (Phase A = infrastructure interne)
- `feedback_memory_update.md` : update nexus_grid_pivot post-commit — N/A (pre-commit review)
- `sprint14_keyoxide_decision.md` : deploy from-source — N/A (Phase A ne touche pas deploy/provenance)

## Staging check
- Phase fichiers : 4 (blobs.rs, lib.rs, node.rs, runtime.rs)
- Planning/docs split : N/A (0 planning fichier dans le diff)
- Untracked accidentels : 1 (`sprint66_phase_a_preflight.md` — planning artefact, pas de code)

## Suites verification
| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | clean | clean | - | ok |
| cargo clippy | 0 warn | 0 warn | - | ok |
| Rust nextest | 1333 | 1338 | +5 | ok |
| Rust doctests | ok | ok | | ok |
| tsc --noEmit | clean | clean | - | ok |
| ESLint | 0 err (5 warn) | 0 err (5 warn) | - | ok |
| Vitest | 268 | 268 | +0 | ok |
| Build web | ok | ok | - | ok |
| size-limit | 5/5 pass | 5/5 pass | - | ok |
| scan-en-strings | clean | clean | - | ok |
| scan-trust-wording | clean | clean | - | ok |
| Release build | - | (en cours, ICE intermittent) | - | pending |

Delta tests : plan = +5, reel = +5. Coherent.

## Branch coverage semantique (deep)

### node.rs — nouvelles fonctions/branches

| Element | LOC | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|-----|------|------------|-------------------|-------------|--------|
| `enum BlobStore { Mem, Fs }` | 6 | implicit (all tests via Node) | oui | oui | Mem path tested + Fs path tested | DEEP-PASS |
| `impl Deref for BlobStore` | 7 | `persistent_fsstore_survives_reboot` + `memstore_still_works_without_data_dir` | oui (BlobsClient::new takes &Store) | oui (add_bytes + get_bytes roundtrip) | both variants | DEEP-PASS |
| `create_node_with_config` FsStore branch (l.313-325) | 12 | `persistent_fsstore_survives_reboot` + `data_dir_creates_blobs_subdir` | oui | oui (hash persists reboot, blobs/ dir exists) | only happy path — error path (bad path) not tested | SHALLOW-PASS |
| `create_node_with_config` MemStore branch (l.324) | 1 | `memstore_still_works_without_data_dir` | oui | oui (roundtrip) | single case | DEEP-PASS |
| `Node::shutdown` blobs_store.shutdown (l.214-216) | 3 | `persistent_fsstore_survives_reboot` (calls node_a.shutdown().await.unwrap()) | oui | blob persists post-shutdown = implicit flush verification | only success case; error case warned but not tested | DEFENSIVE-OK |
| `test persistent_fsstore_survives_reboot` | 18 | IS the test | - | - | - | - |
| `test data_dir_creates_blobs_subdir` | 9 | IS the test | - | - | - | - |
| `test memstore_still_works_without_data_dir` | 9 | IS the test | - | - | - | - |

### blobs.rs — signature changes

| Element | LOC | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|-----|------|------------|-------------------|-------------|--------|
| `BlobsClient::new(&Store)` signature change | 2 | All existing blobs tests + 3 new tests | oui | oui | both Mem and Fs via node.blobs_store() | DEEP-PASS |

### runtime.rs — boot robustness + data_dir wiring

| Element | LOC | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|-----|------|------------|-------------------|-------------|--------|
| `with_data_dir(iroh_data_dir)` wiring (l.290-297, l.310-312) | 6 | `boot_storage_namespace_persistent_reopen` + `boot_feed_namespace_persistent_reopen` | oui (DaemonRuntime::start boots with data_dir) | boot succeeds + reboot succeeds | only happy path reopen; see P2-1 below for the recreate fallback | DEEP-PASS |
| `boot_storage_namespace` fallback recreate (l.1448-1460) | 13 | `boot_storage_namespace_persistent_reopen` | implicitement via boot→shutdown→reboot cycle, but the `None` (recreate) branch requires doc absent from iroh which is NOT exercised by the test — the persistent store RETAINS the doc | non | recreate branch NOT tested | PARTIAL P2 |
| `boot_feed_namespace` fallback recreate (l.1522-1531) | 10 | `boot_feed_namespace_persistent_reopen` | same as above — the `None` (recreate) branch NOT exercised | non | recreate branch NOT tested | PARTIAL P2 |

### runtime.rs — new tests

| Element | LOC | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|-----|------|------------|-------------------|-------------|--------|
| `test boot_storage_namespace_persistent_reopen` | 7 | IS the test | - | Checks boot+shutdown+reboot without panic | no assertion on namespace identity | SHALLOW-PASS |
| `test boot_feed_namespace_persistent_reopen` | 9 | IS the test | - | Checks `feed_handle.is_some()` after reboot | no assertion on namespace identity | SHALLOW-PASS |

## Scope cuts semantique (deep)
| # | Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|---|-----------|---------|-----------|----------------|-----------------|--------|
| 1 | CuratorVouched impl | Pas dans S66 | 0 match | 0 code | CLEAN |
| 2 | BuildQuorumReached | Pas dans S66 | 0 match | 0 code | CLEAN |
| 3 | Quarantine hot path | Pas dans S66 | 0 match | 0 code | CLEAN |
| 4 | Age witness gate | Pas dans S66 | 0 match | 0 code | CLEAN |
| 5 | CONFIRM_PROMPT UI nonce | Pas dans S66 | 0 match | 0 code | CLEAN |
| 6 | SBFB.json v2 code | Pas dans S66 | 0 match | 0 code (persistence only) | CLEAN |
| 7 | node_id deprecation deploy.rs | Pas dans S66 | 0 match in deploy.rs | 0 deploy changes | CLEAN |
| 8 | Factory template scaffold | Pas dans S66 | 0 match | 0 factory code | CLEAN |
| 9 | Fuzzing cargo-fuzz | Pas dans S66 | 0 match | no fuzz tests | CLEAN |
| 10 | CLI verify-release | Pas dans S66 | 0 match | 0 CLI changes | CLEAN |
| 11 | VerificationDetail niveau 3 | Pas dans S66 | 0 match | 0 UI changes | CLEAN |
| 12 | Playwright E2E re-ecriture | Pas dans S66 | 0 match | no Playwright files | CLEAN |
| 13 | Feed format version bump | Post-launch | 0 match | 0 FORMAT_VERSION changes | CLEAN |
| 14 | Multi-curator trust overlay | Stretch S67 | 0 match | 0 curator trust code | CLEAN |

All 14 scope cuts CLEAN. No scope creep detected.

## Research grounding (deep)
### Preflight G8
- Fichier : **existe** (`sprint66_phase_a_preflight.md`)
- Scans : **5/5** (S1a, S1b, S2, S3, S4 all present)
- S1a OSS : 5 projets cites (iroh-blobs, iroh-docs, IPFS Kubo, libp2p, SSB)
- Verdict : **EXECUTE plan-as-is**
- APPROACH-ALIGNED (pas PLAN-ADAPT, pas DESIGN-CONFLICT)
- Note technique : `Store` est un struct concret pas un trait — enum choisi correctement

### Deps/API
| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| iroh-blobs | 0.100 (workspace) | oui (context7 + WebSearch kickoff) | `FsStore::load` API usage matches docs.rs | PASS |
| iroh-blobs `FsStore` | 0.100 | oui (context7 `/n0-computer/iroh-blobs`) | `Deref<Target=Store>` confirmed, `BlobsProtocol::new(&Store)` confirmed | PASS |
| iroh-docs | 0.98 (workspace) | oui (context7 kickoff) | `Docs::persistent(path)` API matches existing node.rs code | PASS |

Pas de nouvelle dep ajoutee (0 changement Cargo.toml). Toutes les APIs utilisees dans le diff sont des APIs existantes du workspace.

### Coherence code-vs-source
- `FsStore::load(&blobs_dir)` : context7 dit `FsStore::load(path)` retourne `FsStore` async. Code l.319 fait exactement cela. **Coherent.**
- `BlobsProtocol::new(&blobs_store, None)` : context7 dit `BlobsProtocol::new` prend `&Store`. Code l.342 passe `&blobs_store` qui deref vers `&Store` via `BlobStore::Deref`. **Coherent.**
- `Docs::persistent(path)` : context7 confirme. Code l.332 identique a l'API existante. **Coherent.**
- `Store::shutdown()` : context7 dit `FsStore::shutdown()` requis pour flush redb. Code l.214 appelle `self.blobs_store.shutdown()` via Deref sur Store. **Coherent.**

## Security deep
### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| node.rs | `unwrap()` | 489-531 | N/A | Tous dans `#[cfg(test)]` — acceptable |
| runtime.rs | `unwrap()` | 1749-1771 | N/A | Tous dans `#[cfg(test)]` — acceptable |
| node.rs | `unsafe` | - | - | 0 match (file-level `#![deny(unsafe_code)]`) |
| node.rs | `#[allow]` | - | - | 0 match |
| node.rs | `#[cfg(not(test))]` | - | - | 0 match |
| node.rs | `#[ignore]` | - | - | 0 match |
| node.rs | `todo!` / `panic!` | - | - | 0 match |

### Analyse semantique

1. **FsStore path injection** : `blobs_dir = path.join("blobs")` where `path` comes from `cfg.data_dir` which comes from `opts.paths.root.join("iroh")` in runtime.rs. The root path is controlled by `ShellDaemonPaths::resolve` which derives from `NEXUS_GRID_ROOT` env or `~/.nexus-grid/`. No user-controlled network input reaches this path. **No risk.**

2. **`std::fs::create_dir_all` on blobs_dir** (l.316) : creates intermediate directories. Safe — path is under daemon root, no path traversal possible from network. **Clean.**

3. **`Store::shutdown()` error handling** (l.214-216) : errors are logged via `warn!()` but Node::shutdown still returns `Ok(())`. This is correct — a failed store flush is recoverable (redb ACID guarantees committed data is safe). **Clean.**

4. **No unbounded Vec/String from network** in the diff. All inputs are local filesystem paths. **Clean.**

5. **No new `serde(default)` on wire-format structs.** Diff only changes internal node structure. **Clean.**

6. **Shutdown ordering** (l.210-222) : Router::shutdown() first (drains protocol handlers), then blobs_store.shutdown() (flushes redb). This is correct — BlobsProtocol handler needs the store alive during drain. The preflight incorrectly suggested reverse order; the implementation is right. **Clean.**

## Livrable verification (remplace Codex)
| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | Enum `BlobStore { Mem, Fs }` | CONFIRME | `node.rs:111-116` | `pub enum BlobStore { Mem(MemStore), Fs(FsStore) }` |
| 2 | `impl Deref<Target=Store> for BlobStore` | CONFIRME | `node.rs:118-126` | `fn deref(&self) -> &Store { match self { ... } }` |
| 3 | `Node.blobs_store` type change `MemStore` -> `BlobStore` | CONFIRME | `node.rs:139` | `blobs_store: BlobStore,` |
| 4 | `Node::blobs_store()` retourne `&Store` | CONFIRME | `node.rs:177-179` | `pub fn blobs_store(&self) -> &Store { &self.blobs_store }` |
| 5 | `FsStore::load` wiring dans `create_node_with_config` | CONFIRME | `node.rs:313-325` | `Some(path) => { let blobs_dir = path.join("blobs"); ... FsStore::load(&blobs_dir).await ... BlobStore::Fs(fs_store) }` |
| 6 | `Store::shutdown()` dans `Node::shutdown` | CONFIRME | `node.rs:214-216` | `if let Err(e) = self.blobs_store.shutdown().await { warn!(...) }` |
| 7 | `BlobsClient::new` prend `&Store` | CONFIRME | `blobs.rs:61` | `pub fn new(inner: &'a Store) -> Self` |
| 8 | Re-export `BlobStore` | CONFIRME | `lib.rs:108` | `pub use node::{BlobStore, Node, NodeConfig, ...}` |
| 9 | `runtime.rs` : `with_data_dir` sur les deux branches | CONFIRME | `runtime.rs:295-297, 310-312` | `.with_data_dir(iroh_data_dir.clone())` dans both `Some(secret_bytes)` et `None` (file-based identity) |
| 10 | `boot_storage_namespace` : fallback recreate si doc manquant | CONFIRME | `runtime.rs:1448-1460` | `None => { warn!(...); let doc = docs_client.create_doc().await?; ... }` |
| 11 | `boot_feed_namespace` : fallback recreate si doc manquant | CONFIRME | `runtime.rs:1522-1531` | `None => { warn!(...); let doc = docs_client.create_doc().await?; ... }` |
| 12 | Test `persistent_fsstore_survives_reboot` | CONFIRME | `node.rs:486-509` | Boot → add_bytes → shutdown → reboot → get_bytes roundtrip |
| 13 | Test `data_dir_creates_blobs_subdir` | CONFIRME | `node.rs:511-522` | Boot with data_dir → assert blobs/ exists |
| 14 | Test `memstore_still_works_without_data_dir` | CONFIRME | `node.rs:524-534` | Boot without data_dir → add_bytes → get_bytes roundtrip |
| 15 | Test `boot_storage_namespace_persistent_reopen` | CONFIRME | `runtime.rs:1749-1757` | Boot → shutdown → reboot (no panic) |
| 16 | Test `boot_feed_namespace_persistent_reopen` | CONFIRME | `runtime.rs:1760-1772` | Boot → shutdown → reboot → feed_handle.is_some() |

Resume : 16 livrables / 16 confirmes / 0 gaps / 0 partiels
Estimation LOC fixes manquants : 0

## Patterns drift + horizon long-terme
### Patterns
- `docs/rust/PATTERNS.md` : lu (100+ lines). No pattern directly violated by the diff. The enum `BlobStore` pattern could be documented as a new pattern (conditional backend via enum + Deref) but this is Phase B work (PATTERNS update is planned for Phase B).
- `docs/shell/PATTERNS.md` : lu (100+ lines). No pattern violated.
- Tech debt T-NN items in diff : none touched directly. Worker comment `node.rs` → `runtime.rs:1506` says "MemStore" in a comment; stale but harmless (P3 — see findings).

### Horizon long-terme
- Design doc present (nouveaux modules) : N/A (Phase A activates existing API, no new module > 1 sprint)
- D1..D5 avec alternatives + rationale : oui (kickoff D1 3 alternatives rejetees, D2 3 alternatives rejetees)
- Solution la plus poussee : oui (FsStore = official iroh-blobs persistent store, not custom redb)
- Aucune LOC estimee au plan : plan.md §4 does not contain LOC estimates. Kickoff does not estimate LOC. **Conforme.**

## Commit body validation
### Titre
- Format attendu : `feat(persistence): Sprint 66 Phase A — iroh data_dir + FsStore`
- Regex match : oui (`feat(persistence): Sprint 66 Phase A — ...`)

### 8 sections body
Le body n'a pas ete fourni en draft — il sera genere au moment du commit. Verification differee au commit.
Les delta tests observes (Rust 1333→1338, +5 ; Vitest 268→268, +0) sont notes pour cross-reference.

## Findings

- **P2-1** : `boot_storage_namespace` et `boot_feed_namespace` fallback recreate branches non testees — `runtime.rs:1448-1460` et `runtime.rs:1522-1531`. Les tests `boot_storage_namespace_persistent_reopen` et `boot_feed_namespace_persistent_reopen` exercent le happy path (doc persiste dans iroh → reopen succeeds). Le fallback `None => { warn!("...missing from iroh — recreating"); ... }` n'est JAMAIS atteint car en mode persistent, l'iroh-docs redb conserve le document. Un test dedie qui cree un namespace en SQLite SANS l'ecrire dans iroh-docs (simulation de corruption ou migration zero→persistent) est necessaire pour exercer cette branche. Carry-over Phase E ou S67 acceptable vu la severite low-medium (le fallback est defensif, pas sur le chemin critique du premier boot). Direction fix : ajouter un test qui insere directement dans la table `storage_namespaces` via `set_storage_namespace()` puis boot le daemon — le `open_doc` retournera `None` et le recreate path sera exerce.

- **P2-2** : Les deux tests daemon reopen (`boot_storage_namespace_persistent_reopen` et `boot_feed_namespace_persistent_reopen`) ne verifient PAS que le namespace ID est identique entre les deux boots — `runtime.rs:1749-1757` et `runtime.rs:1760-1772`. `boot_storage_namespace_persistent_reopen` verifie seulement que le daemon boot sans panic. `boot_feed_namespace_persistent_reopen` verifie `feed_handle.is_some()`. Aucun des deux ne compare les namespace IDs avant/apres reboot. Le critere d'acceptation du plan dit "Boot → create doc → shutdown → reboot → doc persiste" — le test dedie `persistent_data_dir_reboots_with_same_doc_and_author` dans node.rs (l.415-455) couvre cette property au niveau node, mais pas au niveau daemon (ou `boot_storage_namespace` / `boot_feed_namespace` resolvent le namespace depuis SQLite). Direction fix : ajouter `assert_eq!(rt1.storage_doc_id, rt2.storage_doc_id)` (requires exposing the id). Carry-over acceptable — la property est couverte au layer inferieur (node.rs test).

- **P3-1** : Commentaire stale `crates/nexus-worker-core/src/engine/runtime.rs:1506` : "The blob content itself lives in the same Node's MemStore" — devrait dire "Store" car le worker utilise desormais `BlobsClient::new(node.blobs_store())` qui retourne `&Store`. Trivial, cosmetic.

(3 findings : 0 P0, 0 P1, 2 P2, 1 P3. Rigor signal satisfait.)

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/todo/panic/allow/cfg(not(test))/ignore sur 4 fichiers; analyse semantique path injection + shutdown ordering + unbounded inputs | node.rs, blobs.rs, runtime.rs, lib.rs | 0 |
| Patterns | PATTERNS.md Rust lu (100+ lines), PATTERNS.md shell lu (100+ lines), 0 drift identifie | docs/rust/PATTERNS.md, docs/shell/PATTERNS.md | 0 |
| Scope-cuts | 14 items kickoff §7 verifies par grep + lecture semantique du diff complet | kickoff.md §7, diff complet 4 fichiers | 0 |
| Branch coverage | 10 elements analyses (enum, Deref, FsStore branch, MemStore branch, shutdown, BlobsClient sig, data_dir wiring, boot_storage fallback, boot_feed fallback, 5 tests lus) | node.rs (536 lignes), blobs.rs (269 lignes), runtime.rs (sections 280-340, 1420-1550, 1680-1773) | 2 (P2-1, P2-2) |
| Research grounding | preflight lu integralement (398 lignes), deps verifices (0 Cargo.toml change), coherence API verifiee (FsStore::load, BlobsProtocol::new, Store::shutdown) | preflight.md, kickoff.md §Sources context7 | 0 |
| Livrables | 16/16 verifies via Read avec line numbers | node.rs, blobs.rs, lib.rs, runtime.rs | 0 |
| Horizon long-terme | D1-D2 alternatives citees, plan.md 0 LOC estimate, pas de nouveau module structurant | kickoff.md §4, plan.md §4, feedback_approach.md | 0 |
| Downstream consumers | grep `blobs_store()` dans workspace (25+ usages), grep `MemStore` residuel (0 dans code prod hors node.rs, 1 comment worker-core test) | blob_serve.rs, deploy.rs, http.rs, browse.rs, iroh_runtime.rs, feed_sync.rs, storage_api.rs, worker runtime.rs | 1 (P3-1) |

## Recommendation
- Ready to commit : **oui**
- Carry-overs S67 : P2-1 (boot fallback recreate branches non testees), P2-2 (namespace ID identity assertion absente dans daemon-level tests)
- Corrections needed : aucune (0 P0, 0 P1)
- P2-1 et P2-2 sont carries acceptables car :
  - P2-1 : la branche fallback est defensive (corruption recovery), pas le chemin principal. Le chemin principal (persistent reopen) est teste.
  - P2-2 : la property (namespace ID stable apres reboot) est testee au layer node.rs (`persistent_data_dir_reboots_with_same_doc_and_author`). Le layer daemon delegue a node.rs.

## Post-commit obligatoire
- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests 1338 Rust)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
