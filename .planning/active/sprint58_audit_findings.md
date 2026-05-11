# Sprint 58 — Audit findings

**Date** : 2026-05-11
**Auditeur** : session fraiche, sans historique du sprint audite
**Guide** : `sprint59_audit_plan.md` (7 tracks A-G)
**Tip master audite** : `1b9c1d5` (HEAD)
**Verdict** : **PASS** (0 P0, 0 P1, 2 P2, 2 P3)

G4 rigor signal satisfait : 2 P2 + 2 P3 documentes (>=1 P2+ requis
pour PASS, cf. §6.1.1).

---

## Track A — Phase integrity

| Check | Resultat |
|---|---|
| 4/4 preflights G8 (A-D) | PRESENT dans `.planning/active/` |
| 5/5 reviews (A-E) | PRESENT dans `.planning/active/` |
| Delta tests cumule | coherent : +8 Rust (A +1, B +0, C +6, D +1), 1232→1240 |
| 2 fixes post-Phase D | `7fb817b` + `3ca0ba1` documentes verification.md rows 31-32 |
| 4 preflights EXECUTE | 4/4 EXECUTE confirme |

**Verification delta tests** : Phase A body "+1 (1232→1233)", Phase B
"+0 (1233→1233)", Phase C "+6 (1233→1239)", Phase D "+1 (1239→1240)".
Total +8 = 1240 final. Coherent avec verification.md §2.

PASS.

---

## Track B — MANDATORY resolution

### B.1 P2-JITTER-SCOPE 3/3 → CLOSED Phase A

`runtime.rs:1566` — `fn jitter_bounds_are_within_range()` :
200 iterations, assert `d.as_secs() >= 30 && d.as_secs() <= 60`.
Appelle `jittered_republish_duration()` qui fait `gen_range(30..=60)`.
Test present et pertinent. PASS.

### B.2 P2-INVITE-U16-WIRE 3/3 → CLOSED Phase A

`docs/rust/PATTERNS.md` line 2387 — §P47 documente :
- Historique rename INVITE_VERSION → INVITE_FORMAT_VERSION
- u8 → u16
- Pre-launch policy (version = 2, pas de compat multi-version)
- Post-v1.0 policy (bump + range)

PASS.

### B.3 P2-RETAIN-RECENT 2/3 → CLOSED Phase B

`runtime.rs:1068-1069` — `retain_interval = tokio::time::interval(60s)`,
premiere tick consommee. `runtime.rs:1210-1211` — branch select
`retain_interval.tick() => browse_limiter.retain_recent()`.
Timer periodique 60s dans la gossip loop. PASS.

### B.4 P2-BRIDGE-SYNC 1/3 → CLOSED Phase B

`scripts/sync-bridge-sdk.sh` (34 lignes) :
- Source `web/public/sbfb-bridge.js`
- Copie vers chaque `examples/*/sbfb-bridge.js`
- SHA256 verification post-copie (sha256sum)
- Exit 1 si drift

Script correct et fonctionnel. PASS.

---

## Track C — AppStorage P2P correctness

### C.1 boot_storage_namespace()

`runtime.rs:583-593` — au boot, pour chaque app repliquee, le daemon
cree ou reouvre le namespace iroh-docs. L'etat est stock dans
`DaemonHttpState.storage_namespaces` (Arc<RwLock<HashMap>>).
`spawn_storage_subscribe()` appele immediatement apres creation. PASS.

### C.2 Routing dual backend

`storage_api.rs:31-33` — `is_replicated()` verifie si l'app est dans
`REPLICATED_APPS` (constante contenant "sbfb-ideas"). Les handlers
storage_get/set/list/delete routent vers iroh-docs si repliquee, sinon
HashMap+SQLite local. PASS.

### C.3 Migration M8

`db.rs` — `CREATE TABLE IF NOT EXISTS storage_namespaces (app_name TEXT
PRIMARY KEY, namespace_id BLOB NOT NULL, doc_ticket TEXT)`. Helpers
`get_storage_namespace` / `set_storage_namespace` avec UPSERT. PASS.

### C.4 Tombstone filtering

`storage_api.rs:108-111` — `fn is_tombstone(value)` verifie
`value["deleted"] == true || value["retracted"] == true`. Utilise dans
storage_list pour filtrer les entries supprimees. PASS.

### C.5 DocsClient helpers

`docs.rs:315` — `get_many_latest_per_key_prefix()` : Query prefix +
single_latest_per_key, dedup multi-auteur. Teste par
`get_many_latest_per_key_prefix_deduplicates` (docs.rs:614).

`docs.rs:337` — `get_latest_by_key()` : Query prefix + next().
Teste par `get_latest_by_key_returns_most_recent_across_authors`
(docs.rs:572). PASS.

### C.6 Ticket Write

`http.rs:316` — route GET `/api/daemon/storage/ticket/{app}`.
`storage_api.rs:454` — handler retourne le ticket genere au boot.
Sous `authed_routes` (bearer + Host + Origin). PASS.

### C.7 Ticket join

`http.rs:320` — route POST `/api/daemon/storage/join`.
`storage_api.rs:481` — handler parse ticket, import via DocsClient,
cree StorageNamespaceState, insere dans map, spawn subscribe.
Sous `authed_routes`. PASS (voir P2-AUDIT-S58-1 ci-dessous pour
l'absence de validation app name et anti-spam).

---

## Track D — Live events + sync E2E

### D.1 spawn_storage_subscribe()

`storage_api.rs:563-587` — tokio::spawn qui subscribe au doc,
matche `DocsLiveEvent::InsertRemote { .. }`, incremente
`ns_state.version` via `fetch_add(1, Ordering::Relaxed)`.
Appele au boot (runtime.rs:588) et au join (storage_api.rs:550). PASS.

### D.2 Bridge polling

Chaine complete verifiee :
- `protocol.ts:35` — "storage_version" dans BridgeMethodSchema
- `useBridge.ts:314-322` — dispatch → authFetch
  `/api/daemon/storage/${encodeURIComponent(app)}/version`
- `sbfb-bridge.js` — `getStorageVersion(app)` + `onStorageUpdate(app, cb)`
  avec setInterval(3s), compare version, invoke callback si change

PASS.

### D.3 Ideas Hub refresh

`examples/sbfb-ideas/app.js:337` —
`bridge.onStorageUpdate("sbfb-ideas", function () { loadAll().then(updateSyncIndicator); })`.
PASS.

### D.4 Test E2E

`crates/nexus-test-harness/tests/multi_daemon.rs:196` —
`test_cross_daemon_storage_sync` : 2 daemons, write A, ticket →
join B, sync visible. Gate `SBFB_INTEGRATION=1`. PASS.

### D.5-D.6 Fixes post-D

- `7fb817b` : percent-encode slash dans key test E2E (route
  `/app/{name}/state/{key}` matche un seul segment). PASS.
- `3ca0ba1` : fire callback au 1er poll si version > 0
  (UX: sync deja en avance au moment du connect). PASS.

---

## Track E — Dette pair Phase B quality

### E.1 retain_recent timer

Cf. Track B.3. Timer 60s dans tokio::select! gossip loop.
`browse_limiter.retain_recent()` appele periodiquement. PASS.

### E.2-E.3 sync-bridge-sdk.sh + SHA256

Cf. Track B.4. Script correct, SHA256 verifie post-copie. PASS.

### E.4 Aucun test Phase B

Justification coherente : retain_recent() est un appel a une methode
existante deja testee (`eviction_after_retain_recent_drops_stale_keys`
dans browse_limiter.rs). Le script est un outil build sans branche
testable. PASS.

---

## Track F — Carries residuels

| Item | Compteur S59 | Verification |
|---|---|---|
| P2-A-1 rand blocker upstream | 18+/3 | exemption externe — coherent CLAUDE.md/kickoff/verification |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 — coherent |
| LT-1 Kudos-v2 | pre-v1.0 | ROADMAP_COMMITMENTS — coherent CLAUDE.md |

Les 3 carries sont documentes de maniere coherente entre kickoff §6,
verification §4-§5, et CLAUDE.md. PASS.

---

## Track G — Roadmap coherence

| Check | Resultat |
|---|---|
| CLAUDE.md S58 CLOSED | ✓ carries S59 corrects |
| HARDENING_ROADMAP last_validated | ✓ 2026-05-10 (S58) |
| SPRINT_LOG row S58 | ✓ presente et detaillee |
| Memory tip vs HEAD | ⚠️ P2-AUDIT-1 ci-dessous |
| Roadmap S59→S60 | ✓ coherent CLAUDE.md/kickoff |
| 0 P0/P1 residuels | ✓ confirme |
| AppStorage Phase 2 scope | ✓ correctement S60+ |
| LT-1 Kudos-v2 | ✓ carry S59 |

---

## Findings

### P2-AUDIT-1 : Memory tip stale (5 commits)

`nexus_grid_pivot.md` tip = `1734cfb` (Phase E wrap-up).
HEAD = `1b9c1d5` (5 commits plus tard : 1 fix SHA verification +
4 chore docs/planning LT-7 + roadmap revision).

Le statut S58 CLOSED est correct dans la memory, mais le tip ne
track pas les commits post-Phase E. Doit etre mis a jour avant
le kickoff S59.

**Action** : update `nexus_grid_pivot.md` tip → `1b9c1d5` dans
cette session d'audit.

### P2-AUDIT-S58-1 : storage_join + anti-spam storage — scope incomplet

Deux lacunes liees au storage P2P, reconnues Phase D commit body
("Anti-spam couches 2-3 = dette explicite S59") mais non propagees
dans les surfaces de tracabilite repo (CLAUDE.md carry, audit_plan
Track F) :

**(a) storage_join pas de validation app name**
`storage_api.rs:481` — `storage_join` accepte n'importe quel
`body.app` sans verifier que l'app est dans `REPLICATED_APPS`.
Un client loopback pourrait creer une entree storage_namespaces
pour une app non prevue.

**(b) Anti-spam couches 2-3 non implementees**
Phase D scope cut (sprint58_phase_D_review.md:62) mentionne
"anti-spam per-author → S59". Cela couvre : rate-limit per-author
sur les ecritures iroh-docs + validation applicative (schema
ideas/{uuid}, format JSON, taille payload). L'audit initial
reduisait cela au seul storage_join validation, ce qui ne couvre
pas tout le sujet.

**Mitigation existante** : endpoint derriere loopback bearer auth,
reseau pre-v1.0 controle.
**Action** : tracker dans carries S59 :
- P2-STORAGE-JOIN-VALIDATE NEW 1/3 (validation app name)
- P2-STORAGE-ANTISPAM NEW 1/3 (rate-limit per-author + validation
  applicative). Propager dans CLAUDE.md carry S59 + audit_plan
  Track F.

### P3-AUDIT-3 : Ordering::Relaxed sur version AtomicU64

`storage_api.rs:576` — `fetch_add(1, Ordering::Relaxed)` et
`storage_api.rs:601` — `load(Ordering::Relaxed)`.

Sur x86_64 c'est equivalent a Acquire/Release (TSO). Avec le
polling 3s, un stale read retarderait le callback d'au plus 1
tick. Non actionnable pre-v1.0. Pourrait devenir pertinent sur
ARM (eventuel build mobile ou Raspberry Pi).

### P3-AUDIT-4 : onStorageUpdate polling swallows errors

`sbfb-bridge.js` — `.catch(function () { /* Swallow poll errors
silently. */ })`. Si le daemon devient injoignable, le polling
continue silencieusement sans feedback utilisateur. La derniere
sync indicator ne se met plus a jour mais l'utilisateur ne sait
pas pourquoi.

Consistent avec le pattern existant des SDK methods (getNodeStatus,
getBrowseList). Pre-v1.0 acceptable, UX improvement S59+.

---

## Verdict

**PASS** — 0 P0, 0 P1, 2 P2, 2 P3.

Sprint 58 a livre toutes les features planifiees (AppStorage P2P via
iroh-docs, 2 MANDATORY CLOSED, phase dette pair, live events + sync
E2E) avec une execution propre (4/4 G8 EXECUTE, 5/5 reviews PASS,
delta tests coherent +8, 12/12 scope cuts respectes, 2 fixes post-D
documentes).

Les 2 P2 sont : (1) memory tip stale — fixe dans cette session,
et (2) storage_join validation + anti-spam couches 2-3, reconnu
Phase D mais non propage dans les surfaces repo. Aucun ne bloque
le demarrage de S59, mais une correction de planning/carry dans
CLAUDE.md + audit_plan Track F doit etre faite au kickoff S59.
