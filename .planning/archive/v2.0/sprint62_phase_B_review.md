# Phase Review — Sprint 62 Phase B

## Verdict : PASS

Rigor signal : 1 P2 + 1 P3 documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation (Step 1.5)
- feedback_approach.md : OSS prior art obligatoire — satisfait (SSB/OrbitDB/iroh-docs dans preflight S1a)
- feedback_context7_systematic.md : context7 avant code API tierce — satisfait (iroh-docs LiveEvent/subscribe queried)
- Tensions plan vs memory : aucune

## Staging check (Step 1bis)
- Phase fichiers : 4 modifies + 1 nouveau (feed_sync.rs)
- Planning/docs split : chore(planning) fait (cf1661d)
- Untracked accidentels : 0

## Suites
- cargo fmt : 0 diff
- cargo clippy : 0 warnings
- Rust nextest : 1286 -> 1289 (+3 Phase B)
- Rust doctests : pass
- Release build : OK
- npm lint : 0 error
- tsc : 0 error
- Vitest : 258/258
- npm build : OK
- size-limit : 6/6 pass

## Modified-file branch coverage (Step 2bis, G9)
- db.rs : `feed_entry_exists_by_hash()` (5 LOC) -> exercee par subscribe handler (integration path)
- db.rs : `get_last_feed_entry_hash_by_author()` (8 LOC) -> exercee par subscribe handler
- runtime.rs : `boot_feed_namespace()` (~40 LOC) -> miroir boot_storage_namespace(), teste integration runtime::tests
- http.rs : champ `feed_sync_state` -> set None dans tests, accesseur simple
- Pattern coherent avec storage_api.rs S58 (meme schema : pas de test unitaire boot/subscribe, E2E Phase C)

## Commit body validation
- Format titre : "feat(feed): Sprint 62 Phase B — feed sync P2P via iroh-docs + LiveEvent subscribe"
- Delta tests coherent : +3 (plan +3)
- Scope cuts honoured : 10/10 clean
- Co-Authored-By present : oui

## Research grounding (Step 4bis)
- 4bis-A S1a OSS prior art : PASS — 3 projets consultes (SSB, OrbitDB, Hypermerge), APPROACH-ALIGNED
- 4bis-B deps/API : PASS — plan §Research consulte non-vide, context7 iroh-docs query documentee, 0 nouvelle dep ajoutee

## Horizon long-terme + documentation amont (Step 4ter)
- Design doc present : PUBLIC_FEED_SPEC.md (S61) + roadmap research doc
- D1..D5 avec alternatives + rationale : 5/5 dans kickoff
- Solution la plus poussee : iroh-docs = meilleur transport P2P disponible (confirme S1a)
- Aucune LOC estimee au plan : 0 match grep

## Scope cuts verification
- CuratorVouched : 0 fichiers diff
- BuildQuorumReached : 0 fichiers diff
- Endpoint verify-release : 0 fichiers diff
- Bridge getProvenanceRecord/verifyRelease : 0 fichiers diff
- UI VerificationDetail : 0 fichiers diff
- Quarantine feed : 0 fichiers diff
- Age witness gate : 0 fichiers diff
- Go-live public : 0 fichiers diff
- Multi-forge >3 noeuds : 0 fichiers diff
- Feed format version bump : 0 fichiers diff

## Findings

- **P2** : `publish_feed_entry_to_docs()` n'est pas auto-cable dans `insert_feed_operation()`. La fonction pub existe mais requiert un appel explicite. Phase C E2E l'exercera, et le wiring production (coordinator → publish on insert) est scope Phase D ou integration sprint. Carry S63 si non resolu Phase D.
- **P3** : `feed_entry_exists_by_hash()` et `get_last_feed_entry_hash_by_author()` n'ont pas de tests unitaires dedies — coherent avec le pattern codebase (db.rs methods testees via API de plus haut niveau).

## Recommendation
- Ready to commit : oui
- Carry-overs S63 : P2 publish auto-wire (si non resolu Phase D)
- Corrections needed : aucune

---

## Post-commit : review croisee GPT 5.5 + fix cedadd3

### 3 P1 confirmes et corriges (cedadd3)

1. **P1 verify_chain() order-dependent** : refactored en chain-linkage
   walk per-author. DB verifiable quel que soit l'ordre d'insertion.
   +1 test test_verify_chain_out_of_order_insertion.
2. **P1 publish pas wire** : insert_and_publish_feed_operation() wrapper
   pub API cree. `#[allow(dead_code)]` — 0 caller production (wiring
   coordinator = Phase D). Phase C E2E doit l'appeler explicitement.
3. **P1 feed_join() collision key** : cle "sbfb-feed-joined-{ns_id}"
   au lieu de FEED_NAMESPACE_KEY.

### 4 P2 documentes (non-bloquants)

- import_ticket vs import_and_subscribe — meme pattern storage_api.rs
- Subscribe JoinHandle non trackee — dette codebase-wide (spawn_storage_subscribe idem)
- 0 test 401/403 feed routes — auth testee centralement dans auth::tests
- Tests Phase B = format + roundtrip — E2E = Phase C scope

### Nuances second pass GPT 5.5

- **Sur-claim corrigee** : insert_and_publish n'est PAS auto-wire
  production. C'est une API prete pour Phase C E2E, pas une preuve
  que "un daemon qui insere publie". Le wiring production
  (coordinator deploy event → insert_and_publish) = Phase D.
- **Materializer order** : materialize_verified() applique les entrees
  dans l'ordre SQLite local (seq AUTOINCREMENT). verify_chain() est
  maintenant order-independent, mais la vue materialisee pourrait
  diverger entre noeuds si l'ordre local differe. Phase C
  test_feed_offline_catchup DOIT comparer PublicRegistryView
  convergence entre A et B, pas juste le count.

### Delta tests final post-fix

Rust workspace : 1286 -> 1290 (+4 : 3 Phase B + 1 out-of-order fix)
Vitest : 258/258 (Node 25 : NODE_OPTIONS=--no-experimental-webstorage)
