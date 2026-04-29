# Sprint 41 — Audit findings (Phase 0 gate S42)

**Auditeur** : meme session (suboptimal G4, user override).
**Tip d'entree** : `19fb09b` (S41 Phase D wrap-up).
**Verdict** : **PASS** (0 P0, 0 P1, 0 P2 nouveau, 3 P2/P3 carries
confirmes).

## Evidence d'exploration par dimension

### Track A — Securite / identity modules (3/3 PASS)

**A-1 — contributor_registry record() idempotent** :
Read `contributor_registry.rs:41-65`. Verifie : si `get()` retourne
`Some(existing)`, retourne directement sans insert. Sinon, INSERT OR
IGNORE + UNIQUE(project_id, contributor_node_id) dans db.rs:71.
Test `record_idempotent` confirme que le second record() retourne
le meme first_deploy_ts. **PASS.**

**A-2 — invite wire field** :
Read `invite.rs:15,54,71`. Le champ `wire` est stocke en DB et
retourne dans InviteRecord. Le ledger ne decode/execute jamais le
wire — c'est un blob opaque. Pas de `decode()` dans InviteLedger
(le Python avait un `decode()` static, omis dans le port car
`nexus_worker_core::invite::Invite::decode()` est le point d'entree
natif). **PASS.**

**A-3 — capability_store SHA-256 integrity** :
Read `capability_store.rs:60-67,133-150,153-163,203-221`. write()
utilise compute_integrity_hash() → SHA-256 du body sans la ligne
integrity_hash. load() re-calcule et compare. Test
`tampered_file_falls_back_all_off` modifie "enabled = true" →
"enabled = false", le hash change, load() retourne all_off. **PASS.**

### Track B — Architecture / queues (3/3 PASS)

**B-1 — quarantine TTL flush_expired** :
Read `quarantine_queue.rs:94-101`. cutoff = now_epoch() - ttl_secs.
DELETE WHERE received_at < cutoff AND flush_status = 'pending'.
Tests : `flush_expired_removes_old` (entry at epoch 1000 → flushed),
`fresh_entry_not_expired` (entry vient d'etre ajoutee → survit).
**PASS.**

**B-2 — upload jitter distribution** :
Read `upload_queue.rs:105-110,120-131`. pseudo_random_f64() utilise
DefaultHasher + thread_id + nanos → [0,1). Exponential : -mean *
ln(u), clampe a max_jitter. Test `jitter_in_range` verifie 20
draws dans [0, 300]. Pattern P2-REVIEW-B-1-S40 (rand_range) —
meme finding, carry S42. **PASS.**

**B-3 — DB migration #4** :
Read `db.rs:91-113`. Migration #4 cree quarantine_messages (8
colonnes + 2 index) et delayed_uploads (5 colonnes + 1 index).
4 migrations sequentielles (#1 tasks+kudos, #2 pow_task_counts,
#3 contributor_attestations+invites, #4 quarantine+uploads). Pas
de collision ni skip. **PASS.**

### Track C — Tests / coverage (8/8 PASS)

**C-1** : delta 1023→1059 (+36) = 8+4+4+4+5+5+6. Chaque test
verifie une branche reelle (roundtrip, edge case, status machine).

**C-2..C-8** : chaque module teste ses fonctionnalites cles
(idempotence, revoke, tamper, TTL, jitter, status). Aucun test
trivial. **PASS.**

### Track D — Process / meta (3/3 PASS)

**D-1** : 3/3 preflights presents, tous verdict EXECUTE.
**D-2** : 12/12 scope cuts respectes (grep confirme 0 route HTTP,
0 tokio::spawn, 0 dispatcher wire).
**D-3** : 7/7 modules pub mod dans lib.rs. **PASS.**

### Track E — Dependencies (2/2 PASS)

**E-1** : chrono workspace dep, version 0.4.
**E-2** : 4 migrations sequentielles, pas de collision. **PASS.**

### Track F — Doc coherence (4/4 PASS)

**F-1** : HARDENING_ROADMAP 1059 Rust / ~2062 total. Coherent.
**F-2** : CLAUDE.md S41 CLOSED. Coherent.
**F-3** : 3/3 phase reviews + 1 Phase D review.
**F-4** : 3/3 phase preflights. **PASS.**

## Carries S42 confirmes

Nouveaux carries S41 correctement documentes :
- P2-REVIEW-A-1-S41 conn() pub encapsulation 1/3
- P2-REVIEW-C-1-S41 pseudo_random jitter 1/3
- P3-REVIEW-B-1-S41 MintRequest ergonomie 1/3

Pre-existants correctement bumpes a 2/3 :
- P2-REVIEW-A-1-S39, P2-REVIEW-B-1-S39, P2-REVIEW-B-1-S40,
  P2-REVIEW-C-1-S40, P3-REVIEW-A-2-S39, P3-REVIEW-B-2-S39,
  P3-AUDIT-A-1-S39, P3-REVIEW-B-1-S40, P3-REVIEW-C-1-S40.

## Recommendation

Commit autorise. 0 P0, 0 P1, aucun fix requis avant ouverture S42.
