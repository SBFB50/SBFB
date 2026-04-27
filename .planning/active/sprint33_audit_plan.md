# Sprint 33 — Audit plan (Phase 0 S32 gate)

**Ecrit** : 2026-04-27 (Phase D wrap-up Sprint 32)
**Scope audit** : commits Phase A `90aff27` → Phase C `626221c`
(3 feat commits Sprint 32)

## Objectif

Audit independant du Sprint 32 avant de commencer S33. Identifier
les P0/P1 qui bloquent et les P2/P3 qui informent le kickoff S33.

## Track A — iroh 0.98 migration (Phase A)

Verifier :
1. **Cargo.lock coherence** : les 4 crates iroh sont resolus aux
   versions cibles (iroh 0.98.x, iroh-docs 0.98.x, iroh-gossip
   0.98.x, iroh-blobs 0.100.x). Pas de resolution mixte 0.97/0.98.
2. **Breaking changes couverture** : les 8 breaking changes
   documentees dans le kickoff §Sources sont addressees ou N/A.
   Grep `SecretKey::generate` (devrait ne plus exister avec arg Rng),
   `as_vec` (devrait etre `to_vec`), `ConnectionType` (devrait etre
   absent).
3. **rand dual version** : `cargo tree -d | grep rand` — evaluer si
   la cohabitation 0.8+0.10 est acceptable ou si un bump workspace
   est necessaire (P2-A-1 carry).
4. **Commentaires stale "iroh 0.97"** : grep dans les fichiers .rs
   (P3 carry Phase A review). Verifier que le runtime n'est pas
   affecte.

## Track B — rusqlite + arti-client activation (Phase B)

Verifier :
1. **rusqlite_migration bump** : le bump 1.3→2.2.0 non planifie
   (P2-B-2) est API-compatible. Verifier que les tests SQLite
   existants (quarantine, trust cache, allowlist, age witness)
   passent bien avec rusqlite 0.36 + rusqlite_migration 2.2.
2. **tor feature gate** : `cargo build -p nexus-core-rs --features tor`
   compile et ne tire pas de deps inattendues. Verifier que le
   module `tor_transport.rs` reste coherent avec la spec S31 Phase C.
3. **tor-rtcompat absence** : P2-B-1 — confirmer que `TorClient::
   create_bootstrapped` infere bien `PreferredRuntime` sans dep
   explicite sur `tor-rtcompat`. Si non, le carry P2-B-1 devient P1.

## Track C — P2 batch carries (Phase C)

Verifier :
1. **max_tokens wire** : `task_runner.rs` passe bien `max_tokens`
   a `GenerationOptions::num_predict`. Test
   `execute_task_ollama_mock_respects_max_tokens` verifie la valeur
   propagee. Confirmer que les autres champs (grammar, watermark)
   sont correctement non-wires (P3 carry documente).
2. **FROST tests** : les 4 nouveaux tests error path couvrent les
   cas k>n, malformed JSON, wrong participant, invalid nonces.
   Verifier que les tests existants (4 happy path) ne sont pas
   regresses.
3. **Tor boot log** : `coordinator.py` differencie bien
   `enabled=false` (disabled) vs `enabled=true` mais echec connexion
   (unavailable). Verifier le format log et que les tests coord ne
   sont pas casses.
4. **HARDENING_ROADMAP compteurs** : last_validated S32, compteurs
   883 Rust / ~1883 total, arti-client version 0.41 (pas 2.0).
5. **Playwright COEP** : le test mock `blob-serve-coep.spec.ts`
   verifie les headers COEP/COOP/CORP/CSP. Confirmer qu'il n'y a
   pas de false-positive (headers dans le mock != headers du daemon
   reel, mais les constantes sont partagees via les tests Rust).

## Verdicts attendus

- **0 P0/P1** = verdict PASS (probable — sprint dette migration, pas
  feature structurante)
- **>=1 P2** = rigor signal G4 satisfait
- Si P0/P1 trouve : emit fix(sprint32): ... avant S33 kickoff

## Cap G7 carry-overs rappel

MANDATORY S33 : **P2-REVIEW-A-1** (LOC plan meta-process, 3/3).
Les autres carries (P2-A-1 rand, P2-B-1 tor-rtcompat, P2-REVIEW-C-2
daemon COEP E2E, P3 grammar/watermark/comments) sont a 1/3.
