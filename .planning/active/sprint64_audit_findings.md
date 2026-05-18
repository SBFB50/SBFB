# Sprint 64 — Audit findings

**Auditeur** : session fraiche independante (2026-05-18).
**Sprint audite** : Sprint 64 — Hardening public cible (v2.0).
**Tip de reference** : `cf1100b` (docs(protocol): Sprint 64 Phase E).
**Audit plan** : `.planning/active/sprint65_audit_plan.md` (7 tracks).

---

## Verdict : PASS

| Severite | Count |
|---|---|
| P0 (regression securite / crash / data loss) | 0 |
| P1 (bug fonctionnel reproductible) | 0 |
| P2 (gap documentaire / hygiene) | 2 |
| P3 (nit / cosmetic) | 1 |

**0 P0, 0 P1** — aucun fix bloquant requis avant ouverture Sprint 65.
**2 P2 + 1 P3** — rigor signal G4 satisfait (>= 1 P2 documente).

---

## Tracks d'audit

### Track 1 — Tests adversariaux completude : PASS

15 tests adversariaux au total (5 pre-existants + 10 S64), tous
distincts et non-redondants.

| # | Test | Vecteur | Origine | Redondant |
|---|---|---|---|---|
| 1 | test_verify_chain_forged_signature | Forged signature (zero bytes) | Pre-S64 | non |
| 2 | test_verify_chain_tampered_hash | Tampered hash (entry_hash flip) | Pre-S64 | non |
| 3 | test_verify_chain_multi_author | Multi-author chain interleaving | Pre-S64 | non |
| 4 | test_verify_chain_out_of_order_insertion | Out-of-order arrival | Pre-S64 | non |
| 5 | test_verify_chain_empty | Empty chain baseline | Pre-S64 | non |
| 6 | test_adversarial_fork_bomb_spam_rejected | Fork-bomb rate-limit 5/min | S64 Phase C | non |
| 7 | test_adversarial_payload_oversized_rejected | Payload > 64 KB | S64 Phase C | non |
| 8 | test_adversarial_bad_repo_url_rejected | Bad repo URL (8 variantes) | S64 Phase C | non |
| 9 | test_adversarial_bad_artifact_hash_rejected | Bad artifact hash (5 variantes) | S64 Phase C | non |
| 10 | test_adversarial_seq_gap_detection | Seq gap broken linkage | S64 Phase C | non |
| 11 | test_adversarial_cross_author_forgery_rejected | Cross-author sig mismatch | S64 Phase C | non |
| 12 | test_adversarial_ed25519_forgery_feed_entry | Ed25519 random 64 bytes | S64 Phase D | non |
| 13 | test_adversarial_blake3_tamper_canonical | BLAKE3 1-bit timestamp flip | S64 Phase D | non |
| 14 | test_adversarial_pow_nonce_difficulty_check | PoW 16-bit random nonces | S64 Phase D | non |
| 15 | test_adversarial_future_timestamp_rejected | Future timestamp > 30 jours | S64 Phase D | non |

Tous les tests dans `public_feed.rs` lignes 691-1553.

### Track 2 — MANDATORY 3/3 resolution : PASS

| Item | Composant | Preuve code | Preuve test | Verdict |
|---|---|---|---|---|
| F1 VERSION-NOT-STORED — M13 | db.rs:196 | ALTER TABLE provenance_records ADD COLUMN app_version TEXT | db.rs:1282-1289 insert/fetch | CLOSED |
| F1 VERSION-NOT-STORED — insert | deploy.rs:166 | `prov.app_version = sbfb.version.clone()` | deploy.rs:717-735 SBFB.json parse | CLOSED |
| F1 VERSION-NOT-STORED — endpoint | http.rs:1730,1738 | provenance_to_json retourne app_version | http.rs:5517,5536 endpoint | CLOSED |
| F5 IROH-INFRA-TIMEOUT — timeout | feed_sync.rs:307-310 | `tokio::time::timeout(Duration::from_secs(30), ...)` | warn "timed out (30s)" | CLOSED |
| F5 IROH-INFRA-TIMEOUT — backoff | feed_sync.rs:302-303,320 | backoff 500ms-30s exponentiel | test_subscribe_stream_break_backoff_progression | CLOSED |
| F5 IROH-INFRA-TIMEOUT — JoinHandle | runtime.rs:897-900 | `self.feed_handle.take()` + `.await` | test_feed_subscribe_joinhandle_shutdown | CLOSED |

### Track 3 — Dette pair integrity : PASS

| Item | Fichier | Preuve | Verdict |
|---|---|---|---|
| P2-FEED-SUBSCRIBE-JOINHANDLE | feed_sync.rs:300,371 | `-> JoinHandle<()>` + joined at shutdown | CLOSED |
| P2-BACKFILL-6PLUS-TEST | feed_sync.rs:193-222 | dedup AVANT rate-limit, backfill exempt (apply_rate_limit=false) | CLOSED |
| P2-FEED-PUBLISH-ORPHAN | public_feed.rs:1108-1150 | test orphan rollback + refuses_if_chained | CLOSED |
| P2-SUBSCRIBE-STREAM-BREAK | feed_sync.rs:670-697 | test backoff progression + resets on success | CLOSED |
| P2-PROCESS-FORMAT | README.md:1220-1223 | exemption retroactive Sprint <= 63 | CLOSED |
| Fix: rate limiter split | public_feed.rs:1193-1206 | per-author test, local exempt | CLOSED |
| Fix: tail-safe rollback | db.rs:856-860 | SQL AND NOT EXISTS prevents chain break | CLOSED |

### Track 4 — Nouveau noeud E2E : PASS

Test `test_new_node_full_sync_and_verify` dans `multi_daemon.rs` :
10/10 points de spec conformes.

| Spec | Observation | Conforme |
|---|---|---|
| Daemon neuf sans donnees | DaemonCluster::spawn(2), d2 vierge | oui |
| Join ticket | GET /api/daemon/feed/ticket sur d1 | oui |
| POST /api/daemon/feed/join | d2 rejoint via ticket | oui |
| Sync >= 3 entries | 3 ops inserees d1, assert count >= 3 | oui |
| Poll feed_status | GET /api/daemon/feed/status (pas feed_cursor) | oui |
| Verify count | body["count"].as_u64() | oui |
| Verify last_seq | body["last_seq"].as_u64(), assert >= 3 | oui |
| Gate SBFB_INTEGRATION=1 | integration_enabled() check | oui |
| Timeout 60s | Duration::from_secs(60) | oui |
| Retry polling 500ms | sleep(Duration::from_millis(500)) | oui |

### Track 5 — PUBLIC_FEED_SPEC.md coherence : PASS

- **§10** : 15 vecteurs documentes, chacun reference un test reel
  existant dans public_feed.rs ou multi_daemon.rs. 0 invention.
  Constantes confirmees : FEED_RATE_LIMIT_PER_MINUTE=5 (l.196),
  FEED_POW_DIFFICULTY=16 (l.138), FEED_MAX_FUTURE_SECS=2592000
  (l.458).
- **§11** : algorithme bootstrap 7 etapes coherent avec
  test_new_node_full_sync_and_verify. Failure modes documentes.
- **§12** : threat model feed standalone. Residual risks couvrent
  les 3 gaps (Sybil, quarantine, auth-tier).

### Track 6 — Process compliance : PASS

| Check | Conforme | Detail |
|---|---|---|
| Preflights G8 | 5/5 EXECUTE | Phase A-E, tous dans archive/v2.0/ |
| Reviews | 5/5 PASS | Phase A-E |
| Commit discipline | 9/9 conformes | feat/fix/docs(scope): Sprint 64 Phase X |
| Sprint pair dette | Phase B | §6.2.1 Regle 1 satisfaite |
| Design review G1 | present | sprint64_design_review.md |

### Track 7 — Carries S65 : PASS

16 items carry-over documentes dans verification.md §5.
0 doublon entre items resolus S64 et carries S65.

| Categorie | Count |
|---|---|
| 3/3 MANDATORY | 1 (P2-FEED-INSERT-NO-AUTH-TIER) |
| 2/3 | 8 |
| 1/3 | 3 |
| Exemption externe | 2 |
| Monitoring | 1 |
| Pre-existant | 1 |
| **Total** | **16** |

---

## Findings

### P2-THREAT-MODEL-FEED-SURFACE (P2, nouveau)

**Constat** : `docs/security/THREAT_MODEL.md` (derniere mise a jour
S16 Phase E) ne contient aucune section STRIDE pour le protocole
feed public (surface d'attaque introduite S61-S64). Le mot "feed"
n'apparait pas dans le fichier. La couverture securite feed est
dans `PUBLIC_FEED_SPEC.md §12` mais le threat model principal
devrait etre mis a jour pour referencer cette nouvelle surface.

**Impact** : doc gap, pas code gap. PUBLIC_FEED_SPEC §12 couvre
les menaces feed-specifiques. Mais un auditeur tiers lisant
THREAT_MODEL.md ne verrait pas le feed protocol.

**Recommandation** : ajouter une section §5.9 "Feed public
protocol" dans THREAT_MODEL.md (STRIDE feed) avec renvoi vers
PUBLIC_FEED_SPEC §12. Candidat Sprint 65 Phase docs ou S66.

**Owner** : planner S65.
**Compteur** : 1/3.

### P2-FEED-INSERT-NO-AUTH-TIER (P2, carry confirme 3/3 MANDATORY)

**Constat** : `feed_insert()` dans `feed_sync.rs:445` accepte les
insertions sans verifier le auth tier du caller. Le handler
extrait le keypair et construit l'entry directement sans check
d'autorisation. Confirme par grep independant.

**Impact** : tout client loopback authentifie (bearer valide) peut
inserer dans le feed, meme si son trust tier ne devrait pas le
permettre. Le bearer loopback limite deja la surface aux processus
locaux, mais la verification auth-tier est un defense-in-depth
necessaire avant go-live.

**Recommandation** : 3/3 MANDATORY S65 — deja documente dans
verification.md §5.

**Owner** : planner S65.
**Compteur** : 3/3 (obligatoire).

### P3-AUDIT-PLAN-COUNTER-DISCREPANCY (P3, nit)

**Constat** : le texte du bootstrap d'audit mentionnait "Items a
2/3 = 9" mais le decompte reel est 8 items a 2/3 (P2-BADGE-
WORDING-PREMATURE est classifie "pre-existant S14", pas 2/3).
Le total 16 items est correct, seule la ventilation par categorie
avait un ecart.

**Impact** : cosmetic. Aucun item manquant ou duplique.

---

## Compteurs tests verifies

| Suite | Memory | Reel | Match |
|---|---|---|---|
| Rust nextest | 1326 | **1326** (1326 passed, 0 skipped) | oui |
| Vitest | 265 | **265** (265 passed, 22 files) | oui |
| size-limit | 6/6 | 6/6 | oui |
| **Total** | **~1597** | **~1597** | oui |

---

## Scope cuts verification

12/12 scope cuts respectes — aucun code scope-cut n'a leaked :

- CuratorVouched/BuildQuorumReached : uniquement en doc comments (l.51-52)
- Quarantine feed hot path : absent
- Age witness gate : absent
- Fuzzing cargo-fuzz/proptest : absent
- VerificationDetail niveau 3 : absent
- Docker compose test distribue : absent
- CLI verify-release : absent
- Interop externe parsers tiers : absent
- SearchManifestPublished : uniquement en doc comments
- Multi-forge feed sync : absent
- Feed format version bump : FEED_FORMAT_VERSION inchange
- unsafe blocks : 0 dans public_feed.rs

---

## Conclusion

Sprint 64 livre son objectif "hardening public cible" avec rigueur.
Les 2 MANDATORY 3/3 (F1 version + F5 timeout) sont fermes avec
preuves code + tests. La dette pair (5 P2) est absorbee. Les 10
tests adversariaux couvrent 10 vecteurs d'attaque distincts et
non-redondants. PUBLIC_FEED_SPEC.md est complete et coherente.

Le seul gap code identifie (P2-FEED-INSERT-NO-AUTH-TIER) est un
carry 3/3 MANDATORY correctement documente pour S65. Le nouveau
finding (P2-THREAT-MODEL-FEED-SURFACE) est un gap documentaire
sans impact code.

**Verdict : PASS — ouverture Sprint 65 autorisee.**
