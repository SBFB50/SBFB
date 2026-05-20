# Sprint 66 Phase E — preflight G8

Date : 2026-05-20 | HEAD : `5acb391` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest, no band-aid, research before
  code. Phase E = tests E2E + docs wrap-up. Les tests E2E restart
  et crash recovery sont la verification finale de la durabilite
  delivree A-D. Pas de shortcut (pas de tests stub ou placeholder).
  RESPECTE.
- feedback_context7_systematic.md : N/A — Phase E n'ajoute aucune
  nouvelle lib/API. Les deps utilisees (tokio, tempfile, serde_json,
  nexus-coordinator-rs, nexus-core-rs) sont toutes existantes et
  deja auditees Phases A-D.
- feedback_full_failfast.md : les 3 blocs + release build tournent
  avant le commit docs(sprint66). RESPECTE.
- feedback_memory_update.md : nexus_grid_pivot.md + MEMORY.md mis
  a jour dans Phase E (wrap-up standard). RESPECTE.

## Scans (all clean)

- S1a OSS prior art : 5 projets recherches (iroh-blobs, SQLite,
  BOINC, SSB, Kubernetes), APPROACH-ALIGNED -- clean
- S1b deps : 6 libs scannees (iroh 0.98, rusqlite 0.36, tokio,
  tempfile, serde_json, hex), 0 CVE critique -- clean
- S2 historiques : 8 fichiers, 12 commits bodies lus -- clean
- S3 threat model : FULL, 5 vectors analyses -- clean
- S4 wire format : FULL / VERSION=1, Day 0 preserved -- clean

---

## S1a — OSS prior art deep analysis

### Projets analyses en profondeur

#### [iroh-blobs] — n0-computer/iroh-blobs (https://github.com/n0-computer/iroh-blobs)
- Fichiers source lus : DESIGN.md (~400 LOC), src/store/fs.rs
  (tests section ~200 LOC)
- Pattern architectural extrait : FsStore persistence test =
  `FsStore::load(dir) → add data → store.shutdown() → FsStore::load(dir) → verify`.
  Bitfield-based consistency model, crash recovery via BLAKE3
  chunk revalidation. `shutdown()` requis pour flush redb.
- Edge cases geres : partial import recovery (2-stage test),
  bitfield deletion recovery, unclean shutdown (data loss
  acceptable pour recent writes, correctness guaranteed).
- Verdict : ALIGNED — le pattern de test restart dans le plan
  (boot → operate → shutdown → reboot → verify) est identique
  au pattern iroh-blobs FsStore officiel.

#### [SQLite WAL] — sqlite.org (https://sqlite.org/wal.html)
- Fichiers source lus : WAL documentation, recovery spec
  (~300 LOC doc)
- Pattern architectural extrait : crash recovery = WAL replay
  automatique au premier `open()`. `synchronous=FULL` garantit
  que les transactions commitees sont sur disque. Le premier
  connect post-crash acquiert un exclusive lock et replay le WAL.
- Edge cases geres : WAL checksum validation, salt verification,
  uncommitted frame drop, exclusive lock during recovery.
- Verdict : ALIGNED — le plan Phase E teste exactement ce scenario
  (insert → drop sans shutdown → reopen → verify data present).

#### [BOINC] — BOINC/boinc-server-test (https://github.com/BOINC/boinc-server-test)
- Fichiers source lus : README.md (~100 LOC), integration test
  framework overview
- Pattern architectural extrait : PHPUnit integration tests,
  service RPC validation, daemon auto-start at boot.
- Verdict : N/A (PHP-based, pattern trop different).

#### [SSB Scuttlebutt] — ssbc protocol guide
- Fichiers source lus : protocol guide overview (~200 LOC)
- Pattern architectural extrait : log-replay au boot = pattern
  standard feeds append-only. Source de verite = fichier log local,
  replication = transport P2P.
- Verdict : ALIGNED — le feed republish (Phase C) + E2E verification
  (Phase E) reproduisent le pattern SSB standard.

#### [Kubernetes] — kubernetes/kubernetes PR #127070
- Fichiers source lus : PR description + daemon_set E2E test
  (~100 LOC)
- Pattern architectural extrait : E2E tests spawn processes,
  verify state, restart, re-verify. Test gate via env var.
- Verdict : ALIGNED — le pattern SBFB_INTEGRATION gate + spawn
  + verify + restart + re-verify est identique.

### Tableau comparatif

| Aspect | Plan Phase E | iroh-blobs | SQLite WAL | SSB |
|--------|-------------|-----------|-----------|-----|
| Restart test | boot→operate→shutdown→reboot→verify | FsStore::load→add→shutdown→load→get | N/A (recovery implicit) | log-replay at boot |
| Crash test | drop sans shutdown→reboot→verify | bitfield revalidation | WAL replay | N/A |
| Data dir reuse | same TempDir, mk_opts(tmp.path()) | same db_dir | same db path | same .ssb dir |
| Test gate | SBFB_INTEGRATION (optionnel) | standard #[tokio::test] | N/A | N/A |

### Finding S1a
- Classification : APPROACH-ALIGNED
- Evidence : iroh-blobs FsStore test pattern (DESIGN.md + fs.rs
  tests), SQLite WAL recovery spec, SSB log-replay pattern
- Impact sur le plan : aucun

---

## S2 — Decision chain reconstruction

### Fichiers scannes
- crates/nexus-test-harness/src/lib.rs : 5 commits lus
- crates/nexus-shell-daemon/tests/e2e.rs : 2 commits lus
- crates/nexus-shell-daemon/src/runtime.rs : 30 commits, 5 bodies
  pertinents lus en detail (Phase A/B/C/D + S33 harness creation)
- CLAUDE.md : 2 commits recents lus
- docs/claude/SPRINT_LOG.md : 1 commit lu

### Decisions historiques trouvees

#### Decision 1 : DaemonHandle process-level vs DaemonRuntime in-process tests
- Sprint 33, sha `3d3bd967` : creation DaemonCluster+DaemonHandle
  (process-level, spawn binaire)
  Body extrait : "DaemonCluster + DaemonHandle, spawn N daemons
  sur ports ephemeres avec NEXUS_GRID_ROOT + SBFB_HOME isoles"
- Sprint 66, sha `f3ea1c31` : Phase A persistence tests use
  DaemonRuntime directly (in-process)
  Body extrait : "test_boot_storage_namespace_persistent_reopen :
  daemon boot twice, no panic"
- Reverse-commit check : 3 commandes executees, resultat = pas
  de reversion. Les deux patterns coexistent : DaemonHandle pour
  multi-daemon E2E, DaemonRuntime pour single-daemon restart tests.
- Status : active (coexistence)
- Impact phase : aucun — Phase E peut utiliser DaemonRuntime
  directement (pattern etabli Phase A-D). Le plan dit
  "DaemonHandle boot" mais le pattern code reel utilise
  DaemonRuntime. Pas de conflit — c'est une precision
  d'implementation, pas un changement de design.

#### Decision 2 : SBFB_INTEGRATION gate
- Sprint 33, sha `3d3bd967` : introduction du gate SBFB_INTEGRATION
  pour tests multi-daemon process
- Sprint 57, sha `f1f26d5c` : gate utilise pour tests cross-daemon
  gossip
- Sprint 64, sha `f4c4fd7` : gate utilise pour adversarial crypto
  E2E
- Reverse-commit check : pas de reversion. Gate toujours actif.
- Status : active
- Impact phase : le plan Phase E mentionne "Gate SBFB_INTEGRATION"
  pour test_e2e_restart_full_cycle. A verifier si les tests
  peuvent tourner sans le gate (ils n'ont pas besoin de reseau P2P,
  juste de persistence locale).

### Memory constraints
- feedback_approach.md : "pick deepest" — les tests E2E doivent
  verifier la persistence reelle (blobs + feed + node_id), pas
  juste un boot sans panic. Le plan Phase E est conforme.
- feedback_full_failfast.md : fail-fast complet avant commit.
  Phase E = docs commit mais meme regle. Conforme.
- feedback_memory_update.md : mise a jour memory au wrap-up.
  Phase E inclut cette etape. Conforme.

---

## S3 — Threat model analysis

### Primitive analysee : E2E restart/crash recovery test

### Assets en jeu
- A1 feed entries (high) : persistence SQLite + iroh-docs
- A2 blobs archives (high) : persistence FsStore redb
- A3 node identity (high) : same node_id across restarts
- A4 curator subscriptions (medium) : persistence subscriptions.json
- A5 RevocationCache (medium) : persistence SQLite M14

### Threat actors
- TA1 crash mid-write (accidental) : power loss, OOM kill, SIGKILL
- TA2 redb corruption (very low) : ACID copy-on-write B-tree

### Attack vectors identifies
1. V1 data loss on unclean shutdown : SQLite WAL + FULL pragma
   mitige (committed transactions safe). FsStore redb ACID mitige
   (recent writes may be lost, correctness preserved). Covered
   by T-FEED-INTEGRITY + FULL pragma Phase B.
2. V2 stale running.json blocks restart : Drop impl removes it.
   Singleton check detects stale (check_stale_or_bail). Covered.
3. V3 partial feed republish : tail-safe skip (Phase D orphan
   recovery). Covered.
4. V4 RevocationCache lost on crash : SQLite M14 persistence
   + restore au boot (Phase D). Covered.
5. V5 node_id change after crash : Ed25519 secret persisted in
   daemon.key, iroh endpoint boots with same key. Covered.

### Mitigations existantes
- T-FEED-INTEGRITY couvre V1 : hash-chain BLAKE3 + Ed25519
- SQLite synchronous=FULL couvre V1 : fsync per commit
- redb ACID couvre V1/V2 : copy-on-write B-tree
- Drop impl couvre V2 : running.json cleanup
- Phase D orphan recovery couvre V3
- Phase D RevocationCache couvre V4
- Phase A persistent identity couvre V5

### Gaps identifies
- Aucun gap. Phase E tests VALIDENT les mitigations existantes.

### Regression check
- Phase E ne modifie aucune mitigation. Elle ajoute des tests
  de verification. Aucune regression possible.

### Verdict S3 : clean

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase E ne touche aucune struct wire format. Aucun fichier
canonical.rs, public_feed.rs, curator.rs, task.rs, ou key_rotation.rs
n'est modifie. Les tests E2E n'alterent pas les wire formats — ils
les consomment en lecture seule pour verifier la persistence.

### Structs verifiees

Phase E ne cree ni ne modifie aucune struct serializable. Les tests
utilisent les fonctions existantes (insert_feed_operation, replay_all)
qui produisent des FeedEntry conformes.

### Day 0 check
- D1 iroh-docs data_dir : Phase E teste la persistence (valide D1)
- D2 iroh-blobs FsStore : Phase E teste la persistence (valide D2)
- D3 feed republish + handle fix : Phase E teste E2E restart (valide D3)
- D4 provenance 3 etats : Phase E n'ajoute pas de test provenance
  (couvert Phase C). N/A.
- D5 verification cross-node : Phase E n'ajoute pas de test
  cross-node (couvert Phase C). N/A.
- Aucune D1-D5 contredite.

### Decisions actees pivot.md
- Aucune decision actee contredite. Phase E = verification finale.

### Pre-launch policy
- *_VERSION = 1 : aucun bump par Phase E
- Pas de tolerant decoder multi-version : Phase E n'en ajoute pas
- Pas de tests "legacy decode" zombie : Phase E n'en ajoute pas

### Version constants check
```
FEED_FORMAT_VERSION = 1     (public_feed.rs:20) — untouched
CURATOR_LIST_FORMAT_VERSION = 1  (curator.rs:61) — untouched
KEY_ROTATION_FORMAT_VERSION = 1  (key_rotation.rs:32) — untouched
TASK_FORMAT_VERSION = 1     (task.rs:61) — untouched
POW_FORMAT_VERSION = 1      (pow.rs:85) — untouched
PIN_FILE_FORMAT_VERSION = 1 (tls_pinning.rs:102) — untouched
```

---

## Telemetrie preflight (agent deep)

- S1a : 5 projets OSS analyses / 5 fichiers source lus
  / ~1300 LOC reviewees / 0 context7 queries (pas de nouvelle lib)
  / 6 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 6 libs scannees / 4 CVE searches / finding : clean (0 CVE)
- S2 : 12 commits bodies lus / 0 archive files (S66 actif)
  / 5 memory files lus / finding : clean
- S3 : FULL / 5 vectors analyses / 0 gaps
- S4 : FULL / 0 structs modifiees / canonical.rs lu integralement :
  oui (296 lignes) + 6 version constants verifiees

## Action

Proceder code phase E.
