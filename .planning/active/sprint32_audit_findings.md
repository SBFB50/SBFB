# Sprint 32 — Audit findings (Phase 0 S33 gate)

**Date** : 2026-04-27
**Scope** : commits Phase A `90aff27` → Phase C `626221c`
(3 feat commits Sprint 32)
**Audit plan** : `sprint33_audit_plan.md`

## Verdict : PASS

0 P0 / 0 P1 / 3 P2 / 2 P3. Rigor signal G4 satisfait
(>=1 P2 documente). Aucun bloquant pour S33.

---

## Track A — iroh 0.98 migration (Phase A)

### A.1 Cargo.lock coherence — OK

Resolutions unifiees :
- iroh 0.98.1
- iroh-docs 0.98.0
- iroh-gossip 0.98.0
- iroh-blobs 0.100.0

Aucune resolution mixte 0.97/0.98 detectee.
308 tests nexus-core-rs verts.

### A.2 Breaking changes couverture — OK

Trois API cassantes verifiees :
- `SecretKey::generate` avec ancien arg Rng : absent ✅
- `as_vec` (renomme `to_vec`) : absent ✅
- `ConnectionType` (supprime) : absent ✅

### A.3 rand triple version — P2-A-1 (carry confirme, 1/3)

`cargo tree -d | grep rand` revele **triple** cohabitation,
pas dual comme documente :
- rand 0.8.6 (via frost-ed25519 2.x → frost-core → rand 0.8)
- rand 0.9.4 (deps intermediaires)
- rand 0.10.1 (iroh 0.98)

Impact runtime : nul (sous-arbres de deps disjoints).
Impact binaire : overhead estime ~45 KB par version rand
supplementaire. Acceptable pre-v1.0 mais a nettoyer quand
frost-ed25519 migrera vers rand >=0.9.

**Action S33** : mettre a jour la description du carry dans
le kickoff pour refléter "triple" au lieu de "dual".

### A.4 Commentaires stale "iroh 0.97" — P3-iroh-comments (carry confirme, 1/3)

7 commentaires dans 5 fichiers referencent encore "iroh 0.97" :
- `nexus-core-rs/src/attestations/age_witness.rs` (lignes 6, 21)
- `nexus-core-rs/src/gossip.rs` (ligne 723)
- `nexus-core-rs/src/discovery.rs` (lignes 4, 117)
- `nexus-core-rs/src/tls_pinning.rs` (ligne 32)
- `nexus-shell-daemon/src/http.rs` (ligne 1035)

Cosmetique, aucun impact runtime. Carry S33 P3.

---

## Track B — rusqlite + arti-client activation (Phase B)

### B.1 rusqlite_migration bump — OK

Versions Cargo.lock :
- rusqlite 0.36.0
- rusqlite_migration 2.2.0

183 tests nexus-worker-core verts (quarantine, trust cache,
allowlist, age witness tous couverts). Pas de regression API.

### B.2 tor feature gate — OK

`cargo build -p nexus-core-rs --features tor` compile
sans erreur ni deps inattendues. Module `tor_transport.rs`
coherent avec spec S31 Phase C (`TorClient::create_bootstrapped`
ligne 120). arti-client 0.41.0 dans Cargo.lock.

### B.3 tor-rtcompat absence — P2-B-1 (carry confirme, 1/3)

Aucune dep explicite `tor-rtcompat` dans les Cargo.toml du
workspace. `TorClient::create_bootstrapped` infere
`PreferredRuntime` — fonctionne car tokio est present dans
le sous-arbre de deps via iroh. Le build compile mais le
comportement depend du feature flag `preferred-runtime`
d'arti-client satisfait implicitement.

Risque : si une future mise a jour arti-client change la
resolution implicite du runtime, la compilation echouerait.
Carry S33 P2 — a expliciter si necessaire.

---

## Track C — P2 batch carries (Phase C)

### C.1 max_tokens wire — OK

`nexus-executor/src/task_runner.rs:17` : `params.max_tokens`
passe a `GenerationOptions::num_predict`. Test
`execute_task_ollama_mock_respects_max_tokens` (ligne 97)
verifie la propagation (`assert_eq!(json["options"]["num_predict"], 256)`).
11 tests nexus-executor verts.

`grammar` et `watermark_config` correctement non-wires
(lignes 51-52, `None`). P3 carry documente.

### C.2 FROST tests — OK

Tests unitaires frost.rs : 6 tests (2 happy path + 4 error path)
- Happy : `frost_dkg_k2_n3_produces_valid_ed25519_sig`,
  `frost_minimum_threshold_k2_n2_round_trips_as_ed25519`
- Error : `frost_trusted_dealer_rejects_k1_per_rfc_9591`,
  `frost_aggregate_refuses_partial_below_k_threshold`,
  `frost_tampered_share_rejected`,
  `frost_sig_verifiable_by_standard_ed25519_verifier` (interop)

Tests HTTP error path http.rs : 4 tests
- `frost_http_invalid_threshold_k_gt_n` (k>n → 400)
- `frost_http_malformed_json_body` (JSON invalide → 4xx)
- `frost_http_round1_invalid_key_package` (key hex tronque → 400)
- `frost_http_aggregate_invalid_pubkey` (pubkey invalide → 400)

218 tests daemon-core + 81 tests daemon : tous verts.

### C.3 Tor boot log — OK

`coordinator.py` (lignes 377-380) differencie 3 etats :
1. `is_available()` → "Tor transport available for outbound HTTP"
2. `config.enabled` mais pas available → "Tor transport enabled
   but not available, using direct connections"
3. `!config.enabled` → (pas de log au niveau coordinator,
   `tor_client.py:65` logue "Tor transport disabled by configuration")

### C.4 HARDENING_ROADMAP compteurs — OK

`last_validated: 2026-04-27` (Sprint 32). Compteurs :
~883 Rust / ~1883 total. arti-client 0.41 (pas 2.0).
Conforme.

### C.5 Playwright COEP — P2-REVIEW-C-2 (carry confirme, 1/3)

Test `blob-serve-coep.spec.ts` utilise `page.route()` mock —
headers COEP/COOP/CORP/CSP definis dans le mock, pas servis
par le daemon reel. Le test verifie correctement le comportement
navigateur (fetch bloque depuis iframe sandbox avec headers
d'isolation). Les constantes headers cote daemon sont testees
separement dans les tests unitaires Rust.

Mock ≠ E2E : ne prouve pas que le daemon sert les headers corrects
en conditions reelles. Carry S33 P2.

---

## Finding supplementaire

### P3-coordinator-comment-arti-version

`coordinator.py:370` contient le commentaire
`# Sprint 31 Phase C — Tor transport (arti-client 2.0).`
alors que la version reelle est arti-client 0.41.0. Inconsistance
cosmetique, aucun impact runtime.

---

## Synthese carry-overs S33

| ID | Priorite | Reports | Description |
|----|----------|---------|-------------|
| P2-REVIEW-A-1 | P2 | **3/3 MANDATORY** | LOC plan meta-process |
| P2-A-1 | P2 | 1/3 | rand triple version (0.8+0.9+0.10) |
| P2-B-1 | P2 | 1/3 | tor-rtcompat implicit dep |
| P2-REVIEW-C-2 | P2 | 1/3 | daemon COEP E2E |
| P3-grammar | P3 | 1/3 | grammar executor wire |
| P3-watermark | P3 | 1/3 | watermark executor wire |
| P3-iroh-comments | P3 | 1/3 | 7 commentaires stale "iroh 0.97" |
| P3-coordinator-comment | P3 | NEW | commentaire arti-client 2.0 → 0.41 |

## Compteur tests valide

883 Rust (nextest) + doctests OK. Conforme a la memory.
