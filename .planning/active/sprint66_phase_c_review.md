# Sprint 66 Phase C — deep review

HEAD: `543eb45` (dirty) | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

(Rigor signal : 0 P1 / 2 P2 documentes — apres correction)

Initial review FAIL (2 P1 bloquants). Corrections appliquees :
- P1-FEED-REPUBLISH-UNTESTED → test_feed_republish_at_boot ajoute
- P1-FEED-JOIN-HANDLE-UNTESTED → test_feed_join_handles_tracked_and_shutdown ajoute
- P2-FEED-JOIN-HANDLES-UNBOUNDED → cap 10 + retain(is_finished) ajoute
Re-run suites : 1342 Rust / 269 Vitest tous verts.

## Memory consultation

- feedback_approach.md : "pick deepest, no band-aid" — respecte
  (cross-node verification = la solution complete, pas un patch).
- feedback_context7_systematic.md : "context7 avant code touchant
  lib/API" — tokio watch/JoinHandle documente dans preflight S1a.
  Respecte.
- sprint14_keyoxide_decision.md : "deploy from source, cle du
  deployer dans le record" — coherent avec D5 cross-node
  verification (node_id extraction). Respecte.
- nexus_grid_pivot.md : "*_FORMAT_VERSION restent a 1" — Phase C
  ne touche aucune VERSION. Respecte.
- fairness_vision.md : N/A (pas de code kudos dans ce diff).
- vision_model.md : N/A (pas de modele economique dans ce diff).

## Staging check

- Phase fichiers : 6 modifies (runtime.rs, feed_sync.rs, http.rs,
  useBridge.ts, BrowsedProject.tsx, BrowsedProject.test.tsx)
- Planning/docs split : 1 untracked (`sprint66_phase_c_preflight.md`)
  — planning artefact, doit etre commite dans un `chore(planning)`
  separe AVANT le commit phase. Melange planning+phase si commite
  ensemble.
- Untracked accidentels : 0

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok |
| Rust nextest | 1339 | 1340 | +1 | ok |
| Rust doctests | 6 | 6 | +0 | ok |
| tsc --noEmit | - | - | - | ok |
| ESLint | - | - | - | ok (0 errors, 5 pre-existing warnings) |
| Vitest | 268 | 269 | +1 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| scan-en-strings | - | - | - | ok |
| scan-trust-wording | - | - | - | ok |
| Release build | - | - | - | ok |

## Branch coverage semantique (deep)

| Element | LOC | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|-----|------|------------|-------------------|-------------|--------|
| `get_provenance` Ok(None) branch (http.rs:1764-1773) | 10 | `provenance_endpoint_absent_status` | oui (query nonexistent project) | oui (status=="absent", verified==false, record null, provenance_hash null) | happy path only (no error path needed — it's the "absent" case) | DEEP-PASS |
| `get_provenance` Ok(Some) + hex decode + verify (http.rs:1738-1762) | 25 | `provenance_endpoint_found_and_verified` + `provenance_cross_node_verified` + `provenance_cross_node_tampered` | oui (3 tests: local key verified, cross-node verified, cross-node tampered) | oui (status, verified, record fields) | true+false (verified=true for valid, verified=false for tampered) + cross-node (different keypair) | DEEP-PASS |
| `get_provenance` hex decode fallback `_ => ("failed", false)` (http.rs:1751) | 1 | - | non | - | - | UNTESTED mais DEFENSIVE-OK (fallback branch for malformed hex, <3 LOC, returns safe default) |
| `feed_join` shutdown via `tokio::select!` + `shutdown_rx.changed()` (feed_sync.rs:635-656) | 22 | - | non | - | - | **UNTESTED P1** |
| `feed_join_handles.push(handle)` + shutdown drain/join (feed_sync.rs:661-663, runtime.rs:951-964) | 18 | - | non | - | - | **UNTESTED P1** |
| feed republish at boot (runtime.rs:642-670) | 28 | - | non | - | - | **UNTESTED P1** |
| `provenance_get` bridge + 404 backward compat (useBridge.ts:324-335) | 12 | - (no Vitest for bridge dispatch) | - | - | - | WIRING-UNTESTED P2 (unit test exists for badge rendering, but bridge wiring untested — pre-existing gap, not new) |
| `provenance_verify` bridge + 404 backward compat (useBridge.ts:337-348) | 12 | - (no Vitest for bridge dispatch) | - | - | - | WIRING-UNTESTED P2 (pre-existing gap) |
| Badge 4 states rendering (BrowsedProject.tsx:301-341) | 40 | 4 tests: verified, failed, absent, loading | oui | oui (text content per state) | all 4 branches covered | DEEP-PASS |

## Scope cuts semantique (deep)

| Scope cut | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|-----------|---------|-----------|----------------|-----------------|--------|
| SC-1 | CuratorVouched/CuratorDisendorsed implementation | S67 Factory ops | 0 match in diff | 0 code implementation | CLEAN |
| SC-2 | BuildQuorumReached feed implementation | S67+ Factory ops | 0 match in diff | 0 code implementation | CLEAN |
| SC-3 | Quarantine feed hot path | S67+ anti-spam | pre-existing quarantine code untouched | 0 new quarantine in diff | CLEAN |
| SC-4 | Age witness gate feed admission | S67+ anti-spam | 0 match | 0 code | CLEAN |
| SC-5 | T1 CONFIRM_PROMPT complet (UI nonce) | S69 | 0 match | 0 code | CLEAN |
| SC-6 | SBFB.json v2 code implementation | S67 | 0 match | 0 code | CLEAN |
| SC-7 | node_id deprecation dans deploy.rs | S67 | diff uses node_id in provenance but does NOT deprecate it — consistent | 0 deprecation code | CLEAN |
| SC-8 | Factory template scaffold | S67+ | 0 match | 0 code | CLEAN |
| SC-9 | Fuzzing cargo-fuzz/proptest | post-audit | 0 match | 0 code | CLEAN |
| SC-10 | CLI verify-release | S67+ | 0 match | 0 code | CLEAN |
| SC-11 | VerificationDetail niveau 3 | S67+ | 0 match | 0 code | CLEAN |
| SC-12 | Playwright E2E tests re-ecriture | S69 | 0 Playwright files in diff | 0 code | CLEAN |
| SC-13 | Feed format version bump | post-launch | FEED_FORMAT_VERSION unchanged (=1) | 0 bump | CLEAN |
| SC-14 | Multi-curator trust overlay | S67 stretch | 0 match | 0 code | CLEAN |

## Research grounding (deep)

### Preflight G8
- Fichier : existe (`sprint66_phase_c_preflight.md`)
- Scans : 5/5 (S1a, S1b, S2, S3, S4)
- S1a OSS : 5 projets (SSB, Automerge, npm attestation, Sigstore
  Rekor, tokio task patterns) — substantif
- Verdict : EXECUTE plan-as-is
- PASS

### Deps/API

| Dep/API | Version | Trace §Research | Coherence code-vs-doc | Signal |
|---------|---------|-----------------|----------------------|--------|
| hex | 0.4 | kickoff D5 context7 refs | hex::decode + encode usage correct | PASS |
| tokio::sync::watch | 1.x | preflight S1a tokio patterns | watch::channel + subscribe + changed() pattern correct | PASS |
| serde_json | 1.x | pre-existing | json!() macro usage correct | PASS |

Aucune nouvelle dep Cargo.toml/package.json ajoutee dans ce diff.

### Coherence code-vs-source

- D5 cross-node verification (kickoff §D5) : source dit "decoder
  record.node_id → [u8;32] et passer a verify_provenance". Code
  http.rs:1738-1751 fait exactement cela. Coherent.
- D4 provenance 3 etats (kickoff §D4) : source dit "status:
  absent|verified|failed". Code http.rs:1738-1773 retourne les
  3 valeurs. Bridge useBridge.ts propage status. Badge
  BrowsedProject.tsx rend les 4 visuels (loading + 3 data states).
  Coherent.
- D3 feed republish (kickoff §D3) : source dit "apres
  boot_feed_namespace, replay_all() → publish_feed_entry_to_docs()
  pour chaque entry". Code runtime.rs:642-670 fait exactement cela.
  Coherent.
- D3 feed_join handle (kickoff §D3) : source dit "stocker le
  JoinHandle, shutdown channel, joined at shutdown". Code
  feed_sync.rs:617-663 + runtime.rs:951-964 implementent le
  pattern. Coherent.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| http.rs | `unwrap()` | 1740 | SAFE | `bytes.try_into().unwrap()` — guarde par `if bytes.len() == 32` sur L1739, ne peut pas paniquer |
| runtime.rs | `unwrap_or_else(\|e\| e.into_inner())` | 956 | SAFE | poison-recovery pattern standard pour std::sync::Mutex |

Aucun nouveau `unsafe`, `todo!`, `panic!`, `unimplemented!`,
`#[allow(dead_code)]`, `#[cfg(not(test))]`, `#[ignore]` dans le
diff.

### Checks specifiques par zone

| Zone touchee | Check | Resultat |
|---|---|---|
| Loopback HTTP (provenance endpoint) | PeerCredsVerified | N/A — GET /provenance est un endpoint public read-only, pas de mutation |
| canonical.rs / wire / schemas | JCS vs serde_json | canonical.rs NON modifie. provenance_to_json() pre-existant utilise serde_json::to_string_pretty — mais c'est pour display/debug, pas wire. canonical_bytes() dans provenance.rs:102 utilise la construction manuelle domain-separated. OK |
| Zip extract | Path traversal | N/A — pas de zip dans ce diff |
| `#[serde(default)]` | Rationale | N/A — aucun nouveau serde(default) |

### Analyse semantique securite

1. **Input non-truste : `record.node_id` (hex string from SQLite)**
   Chemin : DB → `get_provenance_by_project` → `record.node_id` →
   `hex::decode` → `try_into`. La DB contient des records inseres
   par le deploy flow (controlled). Pour les records recus via feed
   sync (future cross-node), le node_id est signe dans le canonical
   bytes. Tampered node_id = signature invalide = verified=false.
   Le fallback `_ => ("failed", false)` couvre les cas malformed hex.
   **Pas de DoS** : hex::decode est O(n) sur une string bornee
   (64 chars hex).

2. **Input non-truste : feed entries from SQLite (boot republish)**
   Chemin : DB → `replay_all()` → `publish_feed_entry_to_docs()`.
   Les entries en SQLite sont la source de verite (signees, hash-
   chainées). La republication est locale (pas de reseau). Pas de
   nouveau vecteur d'attaque.

3. **Concurrency : feed_join_handles Vec<JoinHandle>**
   `Arc<std::sync::Mutex<Vec<JoinHandle<()>>>>` : le lock est acquis
   brievement (push en O(1) ou drain en O(n)). Pas de deadlock risk
   (pas d'await sous le lock, le lock est std::sync pas tokio).
   Concern : Vec non-borne (cf. P2 ci-dessous).

4. **Concurrency : watch::channel shutdown**
   Pattern correct : `sender.send(true)` dans shutdown, chaque task
   `subscribe()` au boot du spawn. `tokio::select!` break sur
   `changed()`. Pas de race : le send est unidirectionnel.

## Livrable verification (remplace Codex)

| # | Livrable (plan §6.2) | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | Feed republish SQLite→iroh-docs au boot | CONFIRME | runtime.rs:642-670 | `replay_all(&db)` + `publish_feed_entry_to_docs(fs, entry).await` en boucle, log count |
| 2 | feed_join JoinHandle tracked + shutdown channel | CONFIRME | feed_sync.rs:617-663, http.rs:176-181, runtime.rs:951-964 | `state.feed_join_shutdown.subscribe()`, `tokio::select!`, `handles.push(handle)`, shutdown drain+join |
| 3 | get_provenance Ok(None) → 200 + status "absent" | CONFIRME | http.rs:1764-1773 | `StatusCode::OK, json!({"status": "absent", "verified": false, "record": null, "provenance_hash": null})` |
| 4 | get_provenance cross-node verification via node_id | CONFIRME | http.rs:1738-1751 | `hex::decode(&record.node_id)` → `verify_provenance(&record_json, &pub_bytes)` |
| 5 | useBridge.ts provenance_get/verify propagate status | CONFIRME | useBridge.ts:331,344 | `status: "absent"` sur 404, `data.status` sur 200 |
| 6 | BrowsedProject badge 4 etats | CONFIRME | BrowsedProject.tsx:301-341 | loading/verified/failed/absent branches |
| 7 | Test provenance_endpoint_absent_status | CONFIRME | http.rs:5544-5564 | assert status=="absent", verified==false, record null |
| 8 | Test provenance_cross_node_verified | CONFIRME | http.rs:5612-5647 | other_kp generates provenance, local state verifies, verified==true |
| 9 | Test provenance_cross_node_tampered | CONFIRME | http.rs:5649-5686 | signer signs, impostor node_id, verified==false, status=="failed" |
| 10 | Vitest badge absent | CONFIRME | BrowsedProject.test.tsx:403-422 | mock status "absent", assert "Provenance" text |
| 11 | **test_feed_republish_at_boot** (plan §6.3 #1) | **GAP** | - | Plan prevoit un test dedie. Non implemente. Code republish = ~28 LOC untested. Fix : test dans runtime.rs tests module qui boot un DaemonRuntime avec des entries feed pre-inserees en SQLite, puis verifie qu'elles apparaissent dans iroh-docs. ~30-40 LOC. |
| 12 | **test_feed_join_handle_tracked** (plan §6.3 #2) | **GAP** | - | Plan prevoit un test dedie. Non implemente. Code handle tracking + shutdown = ~20 LOC untested. Fix : test dans feed_sync::tests ou runtime::tests qui appelle feed_join, verifie handle dans Vec, signale shutdown, verifie join. ~25-35 LOC. |

Resume : 12 livrables (10 code + 2 tests plan) / 10 confirmes / 2 gaps / 0 partiels
Estimation LOC fixes manquants : ~60-75 LOC (2 tests)

## Patterns drift + horizon long-terme

### Patterns
- P51 Raw-op store+forward (PATTERNS.md) : Phase C ne touche pas
  les raw ops directement. N/A.
- Shutdown pattern (feed_subscribe uses watch + JoinHandle since
  S62) : Phase C reproduit le meme pattern pour feed_join.
  Coherent.
- Lock discipline (docs/rust/PATTERNS.md) : `coordinator_db.lock()`
  drop before await in runtime.rs:645-650 (scoped block). Correct.
  `feed_join_handles.lock()` in feed_sync.rs:661 is a quick push,
  no await. Correct.

### Tech debt
- T-NN items : aucun T-item touche par ce diff.
- Le Vec unbounded feed_join_handles est un nouveau candidat tech
  debt (cf. P2 ci-dessous).

### Horizon long-terme
- Design doc present (nouveaux modules) : N/A — pas de nouveau
  module structurant, extensions de modules existants.
- D1..D5 avec alternatives + rationale : oui (kickoff §4).
- Solution la plus poussee : oui — cross-node verification utilise
  la cle du record (pattern Sigstore/SLSA), pas un hack local.
- Aucune LOC estimee au plan : le plan §9 contient des estimations
  delta tests (+5 Rust, +1 Vitest par phase), mais ce sont des
  estimations de delta pas des budgets LOC. Acceptable.

## Commit body validation

### Titre
Draft absent — pas encore commite. Template attendu :
`feat(feed+provenance): Sprint 66 Phase C — feed republish + provenance cross-node`
Format regex match : oui (si utilise).

### 8 sections body
Draft body non fourni par l'executeur. CONCERN : draft-body-absent.
Rappel template : `.claude/templates/commit_body_phase.txt` contient
le squelette complet avec les 8 headers obligatoires.

### Co-Authored-By
A verifier au moment du commit.

## Findings

- **P1-FEED-REPUBLISH-UNTESTED** : Le code feed republish at boot
  (runtime.rs:642-670, ~28 LOC) n'a pas de test dedie. Le plan
  §6.3 test #1 prevoit `test_feed_republish_at_boot` explicitement.
  Ce code inclut la gestion du lock coordinator_db, l'iteration des
  entries, l'appel async `publish_feed_entry_to_docs`, et le log
  du count. Un bug dans cette sequence (ex: lock non-relache avant
  await, entry malformee qui crash la boucle) ne serait detecte par
  aucun test existant.
  Direction fix : ajouter un test d'integration dans
  `runtime.rs::tests` qui :
  (a) insere N entries feed en SQLite via `coordinator_db`,
  (b) boot un DaemonRuntime avec data_dir,
  (c) verifie que les entries apparaissent dans iroh-docs (via
  `feed_sync_state.doc.get_many_by_prefix("feed/")`).
  Estimation : ~30-40 LOC.

- **P1-FEED-JOIN-HANDLE-UNTESTED** : Le code feed_join handle
  tracking + shutdown channel (feed_sync.rs:617-663,
  runtime.rs:230-231+674-677+876-877+951-964, ~40 LOC total) n'a
  pas de test dedie. Le plan §6.3 test #2 prevoit
  `test_feed_join_handle_tracked` explicitement. Ce code est de la
  concurrence async (tokio::select!, watch channel, Arc<Mutex<Vec>>
  drain+join) — exactement le type de code qui requiert un test
  pour detecter les regressions (deadlock, race, shutdown hang).
  Direction fix : ajouter un test dans `feed_sync::tests` ou
  `runtime::tests` qui :
  (a) cree un DaemonHttpState avec les champs feed_join_*,
  (b) appelle feed_join (ou simule un push dans handles Vec),
  (c) signale shutdown via le watch sender,
  (d) verifie que le join complete sans timeout.
  Estimation : ~25-35 LOC.

- **P2-FEED-JOIN-HANDLES-UNBOUNDED** : `feed_join_handles`
  (http.rs:178, feed_sync.rs:661) est un `Vec<JoinHandle<()>>`
  sans limite. Le kickoff R4 identifie le risque et le plan §6.2
  mentionne "max 10 joins actifs" et "cleanup des handles termines
  periodiquement", mais aucune de ces mitigations n'est implementee.
  Un client malicieux qui appelle `POST /api/daemon/feed/join`
  rapidement peut accumuler des centaines de handles. Chaque handle
  contient un JoinHandle (~8 bytes) + la task elle-meme (backfill +
  live stream). Les taches terminees (stream ended/errored) restent
  dans le Vec.
  Direction fix : (a) filtrer les handles `.is_finished()` avant
  push, ou (b) limiter le Vec a 10 entries, ou (c) documenter
  comme carry S67 dans le commit body. Estimation : ~10 LOC pour
  option (a).
  **http.rs:178, feed_sync.rs:661**

- **P2-DELTA-TESTS-PLAN-VS-ACTUAL** : Le plan §6.3 prevoit 5 Rust
  tests + 1 Vitest = +6 delta. L'implementation livre +1 Rust
  (provenance_cross_node_tampered) + 1 Vitest (badge absent) = +2
  delta. Les 2 tests renommes (provenance_endpoint_absent_status,
  provenance_cross_node_verified) ne comptent pas comme delta car
  ils existaient avant sous d'autres noms. Les 2 tests manquants
  (P1 ci-dessus) sont les tests du plan §6.3 #1 et #2.
  Le commit body §Delta tests devra documenter +1/+1, pas +5/+1.

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep unwrap/unsafe/panic sur 3 fichiers diff + scan patterns critiques | http.rs (1728-1783), feed_sync.rs (608-666), runtime.rs (642-670, 935-974) | 0 (unwrap guarde, poison recovery OK) |
| Patterns | PATTERNS.md lu (100 premieres lignes), lock discipline verifie | runtime.rs (645-650), feed_sync.rs (661) | 0 |
| Scope-cuts | 14 items kickoff §7 + grep + lecture semantique diff entier | diff complet (215 insertions) | 0 |
| Branch coverage | 9 elements/branches, 9 tests lus en entier | http.rs tests (5544-5686), BrowsedProject.test.tsx (363-422) | 2 P1 (untested) |
| Research grounding | preflight lu + 3 deps verifiees + 4 coherences code-vs-source | preflight.md, http.rs, runtime.rs, feed_sync.rs, useBridge.ts, BrowsedProject.tsx | 0 |
| Livrables | 12/12 verifies via Read | 6 fichiers diff + tests | 2 gaps (tests manquants) |
| Horizon long-terme | design doc N/A + alternatives citees dans kickoff + LOC check | kickoff D1-D5, plan §6 | 0 |

## Recommendation

- Ready to commit : **NON** — 2 P1 bloquants
- Corrections needed :
  1. Implementer `test_feed_republish_at_boot` (~30-40 LOC)
  2. Implementer `test_feed_join_handle_tracked` (~25-35 LOC)
  3. Optionnel (P2) : ajouter un cap ou cleanup sur
     `feed_join_handles` Vec, ou documenter comme carry S67
  4. Commiter le preflight dans un chore(planning) separe avant le
     commit phase
- Carry-overs S67 : P2-FEED-JOIN-HANDLES-UNBOUNDED (si non fixe)

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
