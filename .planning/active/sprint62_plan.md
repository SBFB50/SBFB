# Sprint 62 — Plan d'execution detaille

**Ecrit** : 2026-05-14 (post-kickoff).
**Source kickoff** : `.planning/active/sprint62_kickoff.md`
**Roadmap source** : `.planning/research/public_verifiable_feed_roadmap.md` Sprint 2

---

## §1 Etat verifie a l'entree

| Metrique | Valeur |
|---|---|
| Tip master | `d05c41a` |
| Rust nextest | 1282 registered, 0 fail stable |
| Rust doctests | 0 pass, 1 ignored |
| Vitest | 258 pass |
| Playwright | 0 (global-setup fail pre-existant) |
| size-limit | 6/6 |
| cargo fmt | 0 diff |
| cargo clippy | 0 warnings |
| Total | ~1546 |

---

## §2 Decisions Day 0 (gelees) — rappel synthetique

- **D1** : iroh-docs namespace par noeud, cles `feed/{author_hex}/{seq}`, DocTicket share/join, LiveEvent::InsertRemote
- **D2** : hash-chains per-auteur + merge local (seq autoincrement), verification independante par auteur
- **D3** : pipeline anti-spam 4 gates (rate-limit GCRA + PoW Hashcash + validation stricte + Ed25519)
- **D4** : Phase A dette pair — F2/F3/F4/F6 (S61 audit P2) + P2-NSIS-UNINSTALL
- **D5** : gate de scission Phase C review — 4 criteres binaires (offline catch-up, replay idempotent, 2+ noeuds, anti-spam)

---

## §3 Research consulte

- Pattern AppStorage P2P : `nexus-shell-daemon/src/storage_api.rs`
  (`StorageNamespaceState`, `spawn_storage_subscribe()`, ticket/join,
  `LiveEvent::InsertRemote` → version increment)
- Pattern DocsClient : `nexus-core-rs/src/docs.rs`
  (`import_ticket()`, `share_write()`, `subscribe()`, `set()`,
  `get_many_latest_per_key_prefix()`)
- Pattern multi-daemon E2E : `nexus-test-harness/tests/multi_daemon.rs`
  (`DaemonCluster`, `test_cross_daemon_storage_sync()`, gate
  `SBFB_INTEGRATION=1`, polling 30s timeout)
- Pattern anti-spam : `pow.rs` (HashcashChallenge 18 bits),
  `storage_limiter.rs` (GCRA governor), `quarantine_queue.rs` (SQLite TTL)
- Pattern feed : `public_feed.rs` (FeedStore, insert/replay/verify_chain),
  `feed_materializer.rs` (PublicRegistryView, cursor)
- Pattern boot namespace : `runtime.rs:570-600`
  (`boot_storage_namespace()` → open/create → spawn subscribe)

### Graphe de dependances inter-phases

```
Phase A (dette pair)
    │
    │ durcit feed store pour multi-auteur
    ↓
Phase B (feed sync foundation)
    │
    │ iroh-docs namespace + LiveEvent + insert remote
    ↓
Phase C (catch-up + E2E) ← GATE DE SCISSION
    │
    │ si 4/4 criteres PASS
    ↓
Phase D (anti-spam + wrap-up)
```

---

## §4 Phase A — Dette pair obligatoire

### §4.1 Objectif

Durcir le feed store (`public_feed.rs`, `feed_materializer.rs`,
`PUBLIC_FEED_SPEC.md`) pour qu'il soit pret a recevoir des entrees
de sources multiples (sync P2P Phase B). Resoudre les 4 P2 audit
S61 bloquants + 1 carry 2/3.

### §4.2 Taches

| # | Tache | Fichier(s) | Estimation |
|---|---|---|---|
| A1 | F4 : wrap `get_last_feed_entry_hash()` + `insert_feed_entry()` dans `tx.execute_batch("BEGIN IMMEDIATE")` / `tx.execute_batch("COMMIT")` | `public_feed.rs` |
| A2 | F3 : durcir `validate_feed_operation()` — project_id hex 64, repo_url starts with `https://`, commit_sha hex 40, artifact_hash hex 64, reason non-vide | `public_feed.rs` |
| A3 | F2 : dans `materialize_incremental()`, apres cursor match et lecture des nouvelles entrees, verifier signature Ed25519 + entry_hash per-entree | `feed_materializer.rs` |
| A4 | F6 : ajouter §5.1 "Trust model" dans la spec — local DB = trust implicit, remote sync = verify everything (signature + hash + validation) | `PUBLIC_FEED_SPEC.md` |
| A5 | Preparer multi-auteur : ajouter tracking per-auteur dans verify logic — `verify_chain()` accepte des entrees de N auteurs, verifie chaque chaine independamment | `public_feed.rs` |
| A6 | P2-NSIS-UNINSTALL : lister launcher.exe + nexus-shell-daemon.exe + nexus-worker.exe dans la section Delete du script uninstall | `packaging/windows/installer.nsi` |

### §4.3 Tests attendus

| Test | Type | Crate |
|---|---|---|
| `test_validate_feed_operation_strict` | unit | nexus-coordinator-rs |
| `test_insert_feed_transaction_atomic` | unit | nexus-coordinator-rs |
| `test_incremental_verify_per_entry` | unit | nexus-coordinator-rs |
| `test_verify_chain_multi_author` | unit | nexus-coordinator-rs |

**Delta previsionnel** : +4 Rust (1282 → 1286).

### §4.4 Criteres d'acceptation Phase A

- `validate_feed_operation()` rejette project_id non-hex, repo_url
  non-HTTPS, commit_sha non-hex-40, artifact_hash non-hex-64
- `insert_feed_operation()` est atomique (BEGIN/COMMIT)
- `materialize_incremental()` verifie signature+hash sur nouvelles entrees
- `verify_chain()` fonctionne avec des entrees de 2+ auteurs
- PUBLIC_FEED_SPEC.md §5.1 documente trust model local vs remote
- NSIS uninstall liste les 3 binaires

---

## §5 Phase B — Feed sync foundation via iroh-docs

### §5.1 Objectif

Un noeud SBFB publie ses feed entries dans un namespace iroh-docs
partage, un second noeud rejoint via DocTicket, recoit les entrees
via `LiveEvent::InsertRemote`, les verifie et les insere dans son
feed local.

### §5.2 Taches

| # | Tache | Fichier(s) | Estimation |
|---|---|---|---|
| B1 | Creer `FeedSyncState` struct (pattern `StorageNamespaceState`) : `doc: Arc<DocHandle>`, `author: DocsAuthorId`, `ticket: String` | `feed_sync.rs` (nouveau) |
| B2 | `boot_feed_namespace()` : au boot daemon, ouvrir/creer le namespace feed, persister namespace_id et ticket dans coordinator.db (reutiliser M8 pattern ou nouvelle table M11) | `feed_sync.rs` + `db.rs` |
| B3 | `publish_feed_entry_to_docs()` : apres chaque `insert_feed_operation()`, ecrire l'entree dans iroh-docs avec cle `feed/{self_author_hex}/{seq_padded}` | `feed_sync.rs` |
| B4 | `spawn_feed_subscribe()` : tokio::spawn un handler sur `doc.subscribe()`. Sur `LiveEvent::InsertRemote` : deserialiser FeedEntry → verify Ed25519 → verify hash per-auteur → validate → insert local (skip si deja present par entry_hash). Log errors. | `feed_sync.rs` |
| B5 | Endpoints HTTP : `GET /api/daemon/feed/ticket` (retourne DocTicket), `POST /api/daemon/feed/join` (importe ticket, spawne subscribe) | `http.rs` |
| B6 | Integration runtime : `boot_feed_namespace()` dans la sequence de boot du daemon, `FeedSyncState` dans l'etat HTTP | `runtime.rs` + `http.rs` |

### §5.3 Tests attendus

| Test | Type | Crate |
|---|---|---|
| `test_publish_feed_entry_to_docs` | unit | nexus-shell-daemon-core |
| `test_feed_entry_roundtrip_iroh_docs` | unit | nexus-shell-daemon-core |
| `test_feed_subscribe_inserts_verified` | unit | nexus-shell-daemon-core |

**Delta previsionnel** : +3 Rust (1286 → 1289).

### §5.4 Criteres d'acceptation Phase B

- Un daemon qui insere une operation feed la publie dans iroh-docs
- `GET /api/daemon/feed/ticket` retourne un DocTicket valide
- `POST /api/daemon/feed/join` importe le ticket et spawn subscribe
- `LiveEvent::InsertRemote` → entree verifiee + inseree dans SQLite local
- Entrees dupliquees (meme entry_hash) sont ignorees (pas d'erreur)

---

## §6 Phase C — Catch-up offline + multi-daemon E2E (GATE)

### §6.1 Objectif

Prouver que la sync feed fonctionne end-to-end entre 2-3 noeuds,
y compris apres deconnexion offline. C'est la phase gate — si les
4 criteres D5 ne sont pas atteints, scission.

### §6.2 Taches

| # | Tache | Fichier(s) | Estimation |
|---|---|---|---|
| C1 | `test_cross_daemon_feed_sync()` : daemon A insere 3 operations feed, daemon B join via ticket, poll `/api/daemon/feed/status` ou query feed local jusqu'a observer les 3 entrees. Gate `SBFB_INTEGRATION=1` | `multi_daemon.rs` |
| C2 | `test_feed_offline_catchup()` : daemon B demarre apres que A ait publie N entrees. B join → rattrape tout l'historique via iroh-docs reconciliation. Verify PublicRegistryView convergence | `multi_daemon.rs` |
| C3 | `test_feed_replay_idempotent()` : daemon B rejoint 2 fois (ou resync) → pas de doublons dans le feed local | `multi_daemon.rs` |
| C4 | Endpoint feed status : `GET /api/daemon/feed/status` retourne `{ count, last_seq, authors: [{ pubkey, count }] }` pour les tests E2E | `http.rs` + `public_feed.rs` |
| C5 | Cursor sync : apres catchup, le cursor materializer pointe vers la derniere entree recue. `materialize_incremental()` reprend correctement | `feed_materializer.rs` |

### §6.3 Tests attendus

| Test | Type | Crate |
|---|---|---|
| `test_cross_daemon_feed_sync` | E2E | nexus-test-harness |
| `test_feed_offline_catchup` | E2E | nexus-test-harness |
| `test_feed_replay_idempotent` | E2E | nexus-test-harness |

**Delta previsionnel** : +3 Rust (1289 → 1292).

### §6.4 Criteres d'acceptation Phase C (= criteres gate D5)

1. **Offline catch-up** : B offline → A publie 3 ops → B rejoint →
   B.PublicRegistryView == A.PublicRegistryView ✓
2. **Replay idempotent** : re-sync ne cree pas de doublons ✓
3. **2+ noeuds E2E** : test multi-daemon PASS avec DaemonCluster ✓
4. **Anti-spam hot path** : _evalue Phase D_ (si Phase C review
   identifie un blocker sync, anti-spam est scope-cut et gate
   declenchee sur les 3 premiers criteres)

**Gate de scission** : 3/4 minimum pour continuer Phase D.
4/4 = sprint complet. 3/4 (anti-spam reportee) = sprint complet
avec carry anti-spam. <3/4 = scission.

---

## §7 Phase D — Anti-spam minimal + wrap-up

### §7.1 Objectif

Cabler les protections anti-spam sur le hot path feed sync (reception
d'entrees distantes) et produire les artefacts de fin de sprint.

### §7.2 Taches

| # | Tache | Fichier(s) | Estimation |
|---|---|---|---|
| D1 | `FeedRateLimiter` : GCRA keyed par `author_pubkey`, quota 5 ops/min. Pattern `storage_limiter.rs` (governor, DashMap) | `feed_limiter.rs` (nouveau) |
| D2 | Integrer `FeedRateLimiter` dans `spawn_feed_subscribe()` : avant insert, check rate-limit. Si depasse → log + drop entree | `feed_sync.rs` |
| D3 | PoW champ optionnel : ajouter `pow_nonce: Option<u64>` dans `FeedEntry` (`#[serde(default)]`). Verification dans subscribe handler si present | `public_feed.rs` + `feed_sync.rs` |
| D4 | Test rate-limit feed : inserer > 5 ops/min d'un meme auteur → entrees au-dela rejetees | `feed_sync.rs` tests |
| D5 | Test PoW feed : entree avec pow_nonce invalide → rejetee. Entree sans pow_nonce → acceptee (backwards compat) | `public_feed.rs` tests |
| D6 | verification.md : fail-fast checklist complete | `.planning/active/` | doc |
| D7 | audit_plan S63 : dimensions a auditer pour le prochain sprint | `.planning/active/` | doc |

### §7.3 Tests attendus

| Test | Type | Crate |
|---|---|---|
| `test_feed_rate_limiter_rejects_excess` | unit | nexus-shell-daemon-core |
| `test_feed_pow_verification` | unit | nexus-coordinator-rs |

**Delta previsionnel** : +2 Rust (1292 → 1294).

### §7.4 Criteres d'acceptation Phase D

- `FeedRateLimiter` rejette > 5 ops/min par auteur
- PoW champ optionnel (backwards compat, `#[serde(default)]`)
- verification.md redigee avec toutes les rows fail-fast
- audit_plan S63 pret

---

## §8 Fail-fast checklist (template verification.md)

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1294, 0 fail stable |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | npm lint | `npm run lint` (web/) | 0 error |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error |
| 8 | Vitest | `npm run test:unit` (web/) | >= 258 |
| 9 | npm build | `npm run build` (web/) | ok |
| 10 | size-limit | `npm run size` (web/) | 6/6 |
| 11 | scan-en-strings | `bash scripts/scan-en-strings.sh` | clean |
| 12 | sync-bridge-sdk | `bash scripts/sync-bridge-sdk.sh` | exit 0 |
| 13-16 | Phase A-D preflights G8 | sprint62_phase_{A..D}_preflight.md | EXECUTE |
| 17-20 | Phase A-D reviews | sprint62_phase_{A..D}_review.md | PASS |
| 21 | Gate scission Phase C | 4/4 criteres D5 (ou 3/4 avec carry) | PASS |
| 22 | Multi-daemon E2E | `test_cross_daemon_feed_sync` | PASS (SBFB_INTEGRATION=1) |
| 23 | Offline catch-up E2E | `test_feed_offline_catchup` | PASS |

**Delta total previsionnel** : +12 Rust (1282 → 1294), +0 Vitest.

---

## §9 Plan de contingence (gate de scission)

Si Phase C review identifie un blocker :

- **1 critere echoue (anti-spam)** : Phase D reduite a wrap-up
  sans anti-spam. Anti-spam → Sprint 63 Phase A. Sprint complet.
- **2+ criteres echouent** : Sprint 62 se termine. Phases restantes
  → Sprint 63. Le plan passe de 6 a 7 sprints. Phase D = wrap-up
  minimal (verification.md + audit_plan).
- **Scission propre** : le code livre en Phase A-C est stable et
  committé. Pas de code a moitie fini.
