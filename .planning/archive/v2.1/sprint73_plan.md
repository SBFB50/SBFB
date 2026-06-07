# Sprint 73 — Plan d'execution (Recherche reseau cablee)

**Ecrit** : 2026-06-03 (apres kickoff, avant 1er commit feat/fix).
**Kickoff** : `sprint73_kickoff.md` (D1..D6 gelees, arbitrage Checkpoint §11
rendu 2026-06-03).
**Design review** : `sprint73_design_review.md` (G1, D1 ✅ D2 ⚠️ D3 ⚠️ D4 ✅
D5 ✅ D6 ✅).
**Roadmap** : v5 Factory Complete Vision, Arc 3.5, sprint **3/6**.

---

## §1 Etat verifie a l'entree

**Tip master** : `087e781` (`chore(planning): Sprint 72 audit findings —
PASS (S73 Phase 0)`). 30 ahead origin, **rien pousse** (pre-launch §1.4).

| Suite | Count entree | Source |
|---|---|---|
| Rust nextest (canonique CI Linux) | **1544** | audit S72 `--no-fail-fast` (1543 Windows + 1 flake OPERATOR-TIMEOUT) ; CI Linux 1544/1544 |
| Vitest (`web/`) | 279 | non touche depuis S72 |
| size-limit | 6/6 | — |
| `factory-operator` | tsc+eslint+build exit 0 (pas de test runner — P2-OPERATOR-NO-TEST-RUNNER, Phase B) | — |
| clippy workspace | 0 warning | audit S72 |
| fmt | exit 0 | audit S72 |

**Re-mesure obligatoire** : au demarrage reel (Phase A preflight) sur le SHA
post-kickoff (le commit chore decale le tip ; le compte Rust reste 1544,
le kickoff n'ajoute pas de test).

**Etat infra recherche (cartographie workflow `wq01d17lj`)** :
- FTS5 `search_index` M15 (`db.rs:211-222`), `search.rs` (278 l :
  sanitize_query, index_entry, search bm25+pagination, rebuild_from_feed,
  clear_all), route `GET /api/daemon/search` (`http.rs:360` +
  `search_handler` `:1957-2010`). **Existe et marche en local.**
- **Gaps** : reindex boot-only (`runtime.rs:778`, `feed_sync.rs:260` insert
  sans reindex) ; `SearchResult` 7 champs sans triplet ; shell Browse sans
  champ recherche (bridge SDK `search` non consomme cote shell).

---

## §2 Decisions Day 0 (gelees — rappel)

| D | Decision | Implication code |
|---|---|---|
| D1 | Reindex FTS5 a chaud = upsert incremental `INSERT OR REPLACE` par `feed seq` au point feed_sync, helper extraction partage, rebuild = reparation | `search.rs` (upsert NEW + helper), `feed_sync.rs:~261`, `db.rs` busy_timeout |
| D2 | SearchResult +4 champs triplet provenance UNINDEXED + migration M17 DROP/recreate | `search.rs:7-16,34-128`, `db.rs:211-222`, `http.rs:1989-2001` |
| D3 | Defer SearchManifest ; feed-local-replique + design note forme correcte | NEW `.planning/research/s73_searchmanifest_index_node_design.md` ; 0 code wire |
| D4 | Barre recherche = champ dedie Browse via `searchBrowse()` | `web/src/api/daemon.ts`, `web/src/pages/Browse.tsx`, i18n FR |
| D5 | Guardrail AVANT persist sur 2 chemins (split validate_result) | `validator.rs:25-89,155`, `http.rs:1500-1522`, `validator_loop.rs:62-80`, THREAT_MODEL §14 + LOOPBACK §3.1 |
| D6 | worker-pump fix `multi_thread` cross-platform (fallback exemption formelle) | `dispatch_loop.rs` + worker `runtime.rs` tests, `PATTERNS §P54` |

---

## §3 Research consulte

- **FTS5 hot reindex** : sqlite.org/fts5 + /wal (3.50.x), rusqlite #1226
  (2024), Cargo.lock (`libsqlite3-sys 0.34` = SQLite 3.50.x → `INSERT OR
  REPLACE` + WAL). Pattern retenu : upsert standalone par rowid=seq, meme tx
  WAL, helper extraction partage. (D1)
- **Discovery decentralisee** : F-Droid / IPFS DHT (ARES 2024 Sybil mono-
  machine ; provider records 24h) / Nostr NIP-50 (relays) / Radicle (scope
  interet) / SSB (proximite sociale). Convergence : feed-local pre-launch,
  noeud-index opt-in plus tard. (D3)
- **iroh-docs pump Windows** : tokio #2499/#7049 (deadlock Windows
  current_thread), iroh blog (acteur thread dedie), example in-repo
  `multi_thread`. Root-cause = flavor runtime test. (D6)
- **Code lu (file:line)** : 8 cartographies, voir kickoff §Sources +
  `sprint73_design_review.md`.

**Dependances inter-phases** :
```
A (securite guardrail + doc)  ── independant, en tete (surface recuperation)
B (dette: worker-pump + test + NetworkProvider)  ── independant
C (reindex chaud upsert + helper)  ──┐
                                     ├─→ D (enrichissement: reutilise helper+upsert C)
                                     │        └─→ E (barre shell: consomme endpoint enrichi D)
F (wrap-up)  ── apres A-E
```
A et B sont independants de C-D-E (securite/dette vs feature recherche).
C → D (D etend l'upsert/helper de C). D → E (E consomme l'endpoint enrichi).

---

## Phase A — Securite invariant guardrail (D5) + lot doc menace

### A.1 Scope
Corriger **P2-RESULT-TEXT-GUARDRAIL-ORDER** (audit S72 headline, priorite
haute). Aujourd'hui `validator::validate_result` persiste `result_text` via
`set_task_result` (validator.rs:74-80 single, :155 quorum) **pendant** la
validation ; le chemin HTTP (`http.rs:1500-1522`) lance le guardrail
`default_output_chain` **apres** (ligne 1507) → sur rejet, deja persiste,
status=completed, pas de rollback ; le chemin `validator_loop` (`:62-80`)
n'a **aucun** guardrail. Split `validate_result` en `*_pre_guardrail`
(signature+status+quorum, **pas de persist**) et `*_post_guardrail`
(`set_task_result` apres passage). Reordonner HTTP (pre → guardrail →
post) ; injecter le guardrail dans `validator_loop` avant persist (skip +
log + pas de kudos sur rejet) ; quorum sur texte agree (`best_hash`).
Corriger les claims fausses doc. **Lot doc absorbe** : P2-TIER-MODEL
(Operator :3001 en tier formel LOOPBACK §2/§8), P2-HARDENING-ROADMAP-META-
STALE (re-cadrage §3 + last_validated).

### A.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/validator.rs` | Split `validate_result` → `validate_result_pre_guardrail` (no persist) + `validate_result_post_guardrail` (persist). Quorum path : persist apres guardrail sur texte agree. |
| `crates/nexus-shell-daemon/src/http.rs` (~1485-1540) | `coordinator_submit_result` : pre → `default_output_chain().run()` → post ; sur rejet, 0 ligne persistee (meme reponse 400 JSON). |
| `crates/nexus-shell-daemon/src/validator_loop.rs` (62-80) | `process_result` : injecter guardrail apres pre, avant persist ; rejet → log + skip + pas de credit kudos. |
| `docs/security/THREAT_MODEL.md` (§14, 786-790) | Reecrire la claim : guardrail AVANT persist sur HTTP + validator_loop. |
| `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (§3.1 :56 + §2/§8) | Corriger justification GET /result + **P2-TIER-MODEL** : Operator :3001 en tier formel (vocab §2 + rows matrice §8). |
| `docs/security/HARDENING_ROADMAP.md` (§3, front-matter) | **P2-HARDENING-ROADMAP-META-STALE** : note de tete « backlog S18-30 clos » + pointeur threat docs vivants + last_validated 2026-06-03. |

### A.3 Tests plan
1. `submit_result_rejected_by_guardrail_persists_nothing` (HTTP) — guardrail
   trip → 400 + `get_task_result` = None + status non `completed`.
2. `submit_result_accepted_persists_after_guardrail` (HTTP) — pass → 200 +
   `result_text` lisible.
3. `validator_loop_rejected_result_not_persisted` — gossip-sourced rejet →
   skip persist + pas de kudos credite.
4. `validator_loop_accepted_result_persisted` — pass → persist + kudos.
5. `quorum_guardrail_runs_on_agreed_text` (redundancy>1) — guardrail sur
   `best_hash` agree, persist apres.
6. Doc presence : `grep` Operator tier formel LOOPBACK §2/§8 ; THREAT_MODEL
   §14 reecrit ; HARDENING_ROADMAP last_validated 2026-06.

### A.4 Critere d'acceptation
```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
grep -n "AVANT" docs/security/THREAT_MODEL.md   # claim corrigee
```
Guardrail prouve AVANT persist sur les 2 chemins (tests rejet → 0 persist) ;
claims doc corrigees ; tier Operator formel.

### A.5 Commit cible
`fix(sprint73): Sprint 73 Phase A — guardrail before result_text persist (2 paths) + Operator tier + hardening-roadmap recadre`
Body : 9 sections (Contexte / Fichiers / Delta tests +5 / Verification §7.4 /
Scope cuts respectes / G8 traceability / Pre-launch protocol / Codex
verification / Carry closure P2-RESULT-TEXT-GUARDRAIL-ORDER + P2-TIER-MODEL +
P2-HARDENING-ROADMAP-META-STALE). G1 design_review present (gate Phase A).

---

## Phase B — Dette (reservee, non-convertible) : worker-pump 3/3 + dette test + NetworkProvider/Operator

### B.1 Scope
**P2-A-1(S71) worker-pump 3/3 MANDATORY** (D6) : fix `multi_thread` sur les 2
tests E2E + defense timeout + `PATTERNS §P54`. **Lot dette test** :
P2-TEST-ZOMBIE, P2-OPERATOR-TIMEOUT, P2-OPERATOR-NO-TEST-RUNNER.
**Durcissement NetworkProvider/Operator (tout traiter S73)** :
P2-POLL-DIAGNOSTIC-LOSS, P2-SYNC-FS-ASYNC, P2-OLLAMA-MODEL-PICKER.

### B.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/dispatch_loop.rs` (test :146) | `#[tokio::test(flavor="multi_thread", worker_threads=2)]` ; garder `timeout(10s)`. |
| `crates/nexus-worker-core/src/engine/runtime.rs` (test miroir) | idem multi_thread sur le test pump. |
| `docs/rust/PATTERNS.md` (§P54) | Note : test pilotant pump iroh-docs + engine spawn → multi_thread obligatoire ; statut P2-A-1 CLOSED (ou exemption formelle si fix insuffisant). |
| `crates/sbfb-factory/src/process_cli.rs` (test :472-487) | **P2-TEST-ZOMBIE** : de-hardcoder le SHA S70 `6fb95df` via repo git fixture (temp repo construit dans le test). |
| `crates/sbfb-factory/tests/operator_server.rs` | **P2-OPERATOR-TIMEOUT** : serialiser le test-group (ex. `serial_test`) OU timeout client configurable ; passe Windows isole. |
| `tools/factory-operator/` (`vitest.config`, `package.json`, `*.test.ts`) | **P2-OPERATOR-NO-TEST-RUNNER** : infra Vitest (jsdom + mock EventSource) + 1-3 tests logique SSE/gate/mapping. |
| `crates/sbfb-factory/src/provider_router.rs` (:383-407) | **P2-POLL-DIAGNOSTIC-LOSS** : memoriser `last_err`, la surfacer au timeout. |
| `crates/sbfb-factory/src/provider_router.rs` (:273,321) + `daemon_client.rs` (:30,42) | **P2-SYNC-FS-ASYNC** : `std::fs::read_to_string` → `tokio::fs` ou `spawn_blocking`. |
| `crates/sbfb-factory/src/operator_server.rs` (:312,924-927) + `tools/factory-operator/src/lib/executionChat.ts` + `pages/ExecutionChat.tsx` | **P2-OLLAMA-MODEL-PICKER** : selecteur modele par intention non-Claude (front) + backend per-provider model (pas de defaut `claude-opus-4-8` aux providers Ollama/Network). |

### B.3 Tests plan
1. `dispatched_task_is_claimed_and_executed_by_worker_engine` (multi_thread)
   — passe Windows natif + Docker Linux (preuve carry CLOSED).
2. Test miroir worker pump (multi_thread) — passe.
3. `audit_commit_valid_phase_commit` (fixture git) — passe sur master pur /
   clone shallow (plus de SHA hardcode).
4. `operator_server` test-group serialise/timeout — passe Windows isole +
   full workspace.
5. `factory-operator` Vitest : `execution_chat_maps_stream_chunk`,
   `gate_renders_on_sensitive`, `eventsource_no_reconnect_storm` (>=1).
6. `network_provider_surfaces_last_error_on_timeout` (mock 401/500 boucle).
7. `resolve_daemon_reads_async` (pas de `std::fs` en contexte async — revue +
   test si applicable).
8. `non_claude_intent_uses_selected_model` (model-picker) — Ollama/Network
   n'herite plus `claude-opus-4-8`.

### B.4 Critere d'acceptation
```
# Windows natif (feedback_wsl_before_push)
cargo nextest run -p nexus-shell-daemon -p nexus-worker-core --locked -E 'test(worker_engine)'
# Docker Linux
docker run --rm -v "${PWD}:/workspace" -w /workspace rust:1.94 sh -lc "cargo test --workspace --locked"
(cd tools/factory-operator && npx vitest run)
```
worker-pump CLOSED (vert Windows+Linux) OU exemption formelle ecrite ;
3 dettes test resolues ; NetworkProvider/Operator durci (3 items).

### B.5 Commit cible
`fix(sprint73): Sprint 73 Phase B — close P2-A-1 worker-pump 3/3 (multi_thread) + test debt + NetworkProvider/Operator hardening`
Body : 9 sections, delta tests cumule, carry closure (worker-pump 3/3 +
6 P2). Phase non-convertible en feature.

---

## Phase C — Fraicheur : reindex FTS5 a chaud incremental (D1)

### C.1 Scope
Rendre les projets feed-distants cherchables a l'instant de l'ingest.
`search::upsert_feed_entry()` NEW (`INSERT OR REPLACE INTO search_index(rowid
= seq, …)`, idempotent) + helper `extract_index_fields(op: &Value)` partage
avec `rebuild_from_feed`. Appel apres `db.insert_feed_entry(&row)` Ok dans
`feed_sync.rs` (meme lock scope, meme tx WAL). `busy_timeout` explicite a
l'open DB. `rebuild_from_feed` reste chemin de reparation.

### C.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/search.rs` (34-128) | `upsert_feed_entry(db, seq, …)` NEW ; `extract_index_fields()` helper ; `rebuild_from_feed` refactore pour reutiliser le helper. |
| `crates/nexus-shell-daemon/src/feed_sync.rs` (~261, apres insert Ok) | Appeler `search::upsert_feed_entry(db, seq, …)` dans le meme scope `db` Mutex. |
| `crates/nexus-coordinator-rs/src/db.rs` (open) | `conn.busy_timeout(Duration::from_secs(5))` explicite. |

### C.3 Tests plan
1. `feed_ingest_indexes_entry_hot` — `upsert_feed_entry` apres insert →
   `search()` trouve l'entree **sans reboot**.
2. `reindex_hot_is_idempotent` — re-upsert meme `seq` → 1 seule ligne
   (no-op rewrite, pas de doublon).
3. `extract_index_fields_shared_with_rebuild` — helper produit les memes
   champs que `rebuild_from_feed` (anti-derive).
4. `hot_reindex_does_not_block_search_reader` — un upsert pendant un
   `search()` concurrent ne bloque pas (WAL, 1 stmt).
5. `rebuild_from_feed_still_repairs` — chemin de reparation toujours vert.

### C.4 Critere d'acceptation
```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked -E 'test(reindex) + test(feed_ingest) + test(rebuild)'
```
Ingest feed-distant → search hit immediat ; re-ingest idempotent ; reader
non bloque ; rebuild repare.

### C.5 Commit cible
`feat(search): Sprint 73 Phase C — hot incremental FTS5 reindex on feed ingest (freshness)`
Body : 9 sections, delta +5, scope cut #1 (pas de SearchManifest) respecte,
G8 traceability (D1 EXECUTE).

---

## Phase D — Enrichissement `SearchResult` triplet provenance (D2) + design note SearchManifest (D3)

### D.1 Scope
Migration M17 : recreer `search_index` FTS5 avec 4 colonnes UNINDEXED
(`repo_url`, `commit_sha`, `archive_hash`, `provenance_hash`) ; `SearchResult`
+4 champs `Option<String>` (+ `is_open_source: bool`, serde default) ;
`index_entry`/`upsert_feed_entry`/`extract_index_fields` extraient le triplet
du `ReleasePublishedPayload` ; `search()` SELECT (offsets 7-10) ;
`search_handler` JSON. **+ design note** `.planning/research/
s73_searchmanifest_index_node_design.md` (forme correcte noeud-index opt-in —
D3 mitigation). `PATTERNS` (FTS5 hot reindex + triplet enrichment) ecrit ici.

### D.2 Fichiers touches
| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` (211-222) | Migration **M17** : DROP + CREATE `search_index` avec 4 colonnes UNINDEXED. |
| `crates/nexus-coordinator-rs/src/search.rs` (7-16, 34-128) | `SearchResult` +4 champs (serde default) ; signatures `index_entry`/`upsert_feed_entry` + `extract_index_fields` extraient le triplet du payload ; `search()` SELECT + query_map offsets 7-10. |
| `crates/nexus-shell-daemon/src/http.rs` (1989-2001) | `search_handler` serialise les 4 champs (additif JSON). |
| `.planning/research/s73_searchmanifest_index_node_design.md` (NEW) | Design forme correcte SearchManifest (noeud-index opt-in signe Ed25519, anti-spam signature+kudos, default OFF, critere declenchement federation partielle). |
| `docs/rust/PATTERNS.md` | Pattern FTS5 hot reindex incremental (D1) + triplet enrichment UNINDEXED (D2). |

### D.3 Tests plan
1. `search_result_carries_provenance_triplet` — un hit `ReleasePublished`
   porte repo_url+commit_sha+archive_hash+provenance_hash.
2. `migration_m17_recreates_index_unindexed` — M17 cree les colonnes ;
   `rebuild_from_feed` repopule depuis le feed (pas de perte).
3. `search_result_null_triplet_for_non_release_op` — op sans triplet
   (CuratorVouched) → champs `None`, pas de crash.
4. `enriched_fields_unindexed_not_matchable` — un MATCH sur un hash ne
   retourne pas (UNINDEXED, retourne seulement).
5. `search_handler_json_includes_triplet` (HTTP) — la reponse JSON porte les
   4 champs.

### D.4 Critere d'acceptation
```
cargo nextest run -p nexus-coordinator-rs -p nexus-shell-daemon --locked -E 'test(provenance) + test(migration_m17) + test(search)'
test -f .planning/research/s73_searchmanifest_index_node_design.md
```
Hit search porte le triplet ; M17 verte (rebuild repopule) ; UNINDEXED ;
design note present. Depend de Phase C.

### D.5 Commit cible
`feat(search): Sprint 73 Phase D — enrich SearchResult with provenance triplet (M17 UNINDEXED) + SearchManifest design note`
Body : 9 sections, delta +5, G8 (D2 EXECUTE, D3 defer documente), scope cut
#1 respecte (design note ≠ implementation wire).

---

## Phase E — Barre de recherche shell (D4)

### E.1 Scope
`searchBrowse(baseUrl, q, limit, offset)` dans `api/daemon.ts` (miroir
`listBrowse` + `DaemonResult` + `authFetch` + `SearchResponseSchema` Zod) ;
champ recherche dedie dans `Browse.tsx` (au-dessus de la grille, React Query
`['daemon-search', coordUrl, q]`) ; rendu resultats avec provenance ; strings
**FR**. Command Palette inchangee (navigation).

### E.2 Fichiers touches
| Fichier | Role |
|---|---|
| `web/src/api/daemon.ts` (~297-311) | `searchBrowse()` + `SearchResponseSchema` Zod (miroir SearchResult enrichi). |
| `web/src/pages/Browse.tsx` (~39-108) | Champ recherche dedie + `useQuery` search + rendu hits (provenance pour fork S74). |
| `web/src/i18n/*` | Cles FR : placeholder recherche, etat vide, erreur (pas d'anglais — scan-en-strings). |

### E.3 Tests plan (Vitest `web/`)
1. `searchBrowse_calls_daemon_search_endpoint` — tape `GET /api/daemon/search?q=`.
2. `browse_search_renders_enriched_results` — hits affichent project_name +
   provenance.
3. `browse_search_empty_state_french` — etat vide en francais.
4. `search_response_schema_parses_triplet` — Zod parse les 4 champs.

### E.4 Critere d'acceptation
```
(cd web && npm run lint && npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run build && npm run size && \
  bash scripts/scan-en-strings.sh)
```
Barre tape `/api/daemon/search` ; hits enrichis ; strings FR ; tous verts.
Depend de Phase D.

### E.5 Commit cible
`feat(shell): Sprint 73 Phase E — wire Browse search bar to GET /api/daemon/search`
Body : 9 sections, delta Vitest +4, scope cut #14 (pas de pagination
boutons) respecte, strings FR.

---

## Phase F — Wrap-up

### F.1 Scope
`sprint73_verification.md` (fail-fast rempli) + `sprint74_audit_plan.md`.
**P2-PREFLIGHT-TRANSITIVE-DEPTH + P2-PREFLIGHT-WIRE-CONTRACT-DEPTH** : amender
les skills/agents preflight (`nexus-phase-preflight` SKILL + agent
`nexus-phase-preflight-deep`) — S1b inspecte le Cargo.toml/lock de la version
**precise** epinglee ; S4 trace chaque champ wire jusqu'au producteur/
consommateur (file:line) avant « inchange ». Lot P3 doc PATTERNS
(§P53/§P54/§P55) si peu couteux. Memory + SPRINT_LOG row S73 + CLAUDE.md.

### F.2 Fichiers touches
| Fichier | Role |
|---|---|
| `.planning/active/sprint73_verification.md` (NEW) | Self-report fail-fast. |
| `.planning/active/sprint74_audit_plan.md` (NEW) | Feuille de route audit S73 pour S74. |
| `.claude/skills/nexus-phase-preflight/SKILL.md` + `.claude/agents/nexus-phase-preflight-deep.md` | S1b version precise + S4 trace wire (P2-PREFLIGHT-*). |
| `docs/rust/PATTERNS.md` (§P53/§P54/§P55) | Lot P3 doc (rename ModelOptions, P2-A-2 ferme, LlmBackend trait, PROVIDERS &str) si peu couteux. |
| memory `nexus_grid_pivot.md` + `MEMORY.md` + `docs/claude/SPRINT_LOG.md` + `CLAUDE.md` | Etat S73. |

### F.3 Critere d'acceptation
100% fail-fast verts ; 2 docs planning ; skills preflight amendees ;
PATTERNS a jour ; memory a jour.

### F.4 Commit cible
`docs(sprint73): verification + audit plan for Sprint 74`

---

## §5 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | fmt | `cargo fmt --all --check` | exit 0 | |
| 2 | clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warning | |
| 3 | nextest workspace | `cargo nextest run --workspace --locked` | 0 fail (canonique Linux) | |
| 4 | doctests | `cargo test --workspace --locked --doc` | 0 fail | |
| 5 | build release | `cargo build -p nexus-shell-daemon --release` | OK | |
| 6 | A — guardrail HTTP rejet | `test(submit_result_rejected_by_guardrail_persists_nothing)` | vert (0 persist) | |
| 7 | A — guardrail accepte | `test(submit_result_accepted_persists_after_guardrail)` | vert | |
| 8 | A — validator_loop rejet | `test(validator_loop_rejected_result_not_persisted)` | vert | |
| 9 | A — quorum guardrail texte agree | `test(quorum_guardrail_runs_on_agreed_text)` | vert | |
| 10 | A — claim doc corrigee | `grep THREAT_MODEL §14 + LOOPBACK §3.1` | « AVANT persist » | |
| 11 | A — Operator tier formel | `grep operator LOOPBACK §2/§8` | rows AD2/AD4 | |
| 12 | B — worker-pump multi_thread Windows | `test(dispatched_task_…worker_engine)` Windows natif | vert (carry CLOSED) | |
| 13 | B — worker-pump Docker Linux | meme test Docker rust:1.94 | vert | |
| 14 | B — zombie de-hardcode | `test(audit_commit_valid_phase_commit)` master pur | vert | |
| 15 | B — operator_server serialise | `test(operator_…)` Windows isole | vert | |
| 16 | B — factory-operator Vitest | `(cd tools/factory-operator && npx vitest run)` | vert (infra NEW) | |
| 17 | B — NetworkProvider last_err | `test(network_provider_surfaces_last_error_on_timeout)` | vert | |
| 18 | B — model-picker non-Claude | `test(non_claude_intent_uses_selected_model)` | vert | |
| 19 | C — reindex chaud | `test(feed_ingest_indexes_entry_hot)` | hit sans reboot | |
| 20 | C — idempotent | `test(reindex_hot_is_idempotent)` | 1 ligne | |
| 21 | C — helper partage | `test(extract_index_fields_shared_with_rebuild)` | vert | |
| 22 | C — reader non bloque | `test(hot_reindex_does_not_block_search_reader)` | vert | |
| 23 | D — triplet provenance | `test(search_result_carries_provenance_triplet)` | 4 champs | |
| 24 | D — migration M17 | `test(migration_m17_recreates_index_unindexed)` | rebuild repopule | |
| 25 | D — UNINDEXED | `test(enriched_fields_unindexed_not_matchable)` | vert | |
| 26 | D — JSON endpoint | `test(search_handler_json_includes_triplet)` | 4 champs | |
| 27 | D — design note present | `test -f .../s73_searchmanifest_index_node_design.md` | present | |
| 28 | E — `web/` lint+tsc | `npm run lint && tsc --noEmit` | exit 0 | |
| 29 | E — Vitest `web/` | `npm run test:unit` | 0 fail (279 + nouveaux) | |
| 30 | E — build + size | `npm run build && npm run size` | 6/6 | |
| 31 | E — scan-en-strings | `bash scripts/scan-en-strings.sh` | 0 string EN | |
| 32 | E — barre tape /search | `test(searchBrowse_calls_daemon_search_endpoint)` | vert | |
| 33 | F — skills preflight amendees | `grep S1b version precise + S4 trace wire` | present | |
| 34 | F — 2 docs planning | `test -f verification.md + audit_plan.md` | present | |

---

## §6 Git plan

| Ordre | Phase | Type | Titre |
|---|---|---|---|
| 1 | kickoff | chore | `chore(planning): Sprint 73 kickoff + plan + design_review + migrate S72 archive` |
| 2 | A | fix | `fix(sprint73): Sprint 73 Phase A — guardrail before result_text persist (2 paths) + Operator tier + hardening-roadmap recadre` |
| 3 | B | fix | `fix(sprint73): Sprint 73 Phase B — close P2-A-1 worker-pump 3/3 (multi_thread) + test debt + NetworkProvider/Operator hardening` |
| 4 | C | feat | `feat(search): Sprint 73 Phase C — hot incremental FTS5 reindex on feed ingest (freshness)` |
| 5 | D | feat | `feat(search): Sprint 73 Phase D — enrich SearchResult with provenance triplet (M17 UNINDEXED) + SearchManifest design note` |
| 6 | E | feat | `feat(shell): Sprint 73 Phase E — wire Browse search bar to GET /api/daemon/search` |
| 7 | F | docs | `docs(sprint73): verification + audit plan for Sprint 74` |

Chaque phase code (A-E) : preflight G8 → review PASS-PENDING → Codex brut →
reconciliation → review PASS → body 9 sections.

---

## §7 Scope cuts (copie kickoff §7)

1 SearchManifest reseau-large → post-launch (D3, design note). 2
`search/open/fork` → S74. 3 projet cible distinct → S74. 4 reseau→atelier
fork → S74. 5 templates etendus → S74. 6 GPU cross-machine → S75. 7 quorum
cross-MACHINE → S75. 8 sharding → S76. 9 Tantivy → gate post-S75 (gele). 10
@dev tree-sitter → post-Gate 1. 11 rate-limit per-client search → S74+ (re-eval
Phase E). 12 webhook/SSE feed push → S74+. 13 token-par-token WAN → jamais
(PO-14). 14 pagination boutons → S74+.

---

## §8 Risks (R1..R7)

Cf. kickoff §9. R1 multi_thread insuffisant (→ exemption formelle). R2
migration M17 perte donnees (→ reconstructible feed + replica). R3 reindex
DoS (→ incremental O(1) + rate-limit existant). R4 ordre guardrail casse
tests integration (→ auditer tests lisant result apres 400). R5 strings EN
shell (→ FR des l'ecriture). R6 defer SearchManifest conteste audit (→
documente + design note). R7 scope creep « tout traiter » (→ phasage strict,
criteres binaires, A+C-E livrables si B deborde).

---

## §9 Checkpoint de cloture

- [ ] Fail-fast checklist §5 : 34/34 rows PASS (canonique CI Linux)
- [ ] Phases A-F landed (A securite+doc, B dette, C reindex, D enrichment, E shell, F wrap-up)
- [ ] **P2-A-1 worker-pump 3/3 CLOSED** (fix multi_thread vert Windows+Linux, OU exemption formelle ecrite) — plus jamais carry
- [ ] P2-RESULT-TEXT-GUARDRAIL-ORDER ferme (guardrail AVANT persist 2 chemins, claims doc corrigees)
- [ ] 12 P2 audit S72 traites (Phase A doc lot + Phase B dette/hardening + Phase F preflight)
- [ ] Reindex FTS5 a chaud (projet feed-distant cherchable sans reboot, idempotent)
- [ ] SearchResult enrichi triplet provenance (UNINDEXED, M17) — prerequis fork S74
- [ ] Barre recherche shell cablee `GET /api/daemon/search` (champ Browse, FR)
- [ ] D3 SearchManifest : defer documente + design note forme correcte present
- [ ] Pas de bump wire (pre-launch ; M17 = schema local ; FEED_FORMAT_VERSION=1)
- [ ] 6/6 phases code G8 (Phase 0 audit done) ; Codex 5/5 phases code (A-E)
- [ ] `sprint73_verification.md` + `sprint74_audit_plan.md` ecrits
- [ ] Skills preflight amendees (P2-PREFLIGHT-*) ; PATTERNS a jour
- [ ] Memory `nexus_grid_pivot.md` + `MEMORY.md` a jour + SPRINT_LOG row S73

**S73 CLOSED quand** : 34/34 fail-fast verts + worker-pump 3/3 CLOSED +
recherche reseau cablee (fraicheur + triplet + barre) + 7 commits + 3
fichiers planning. Arc 3.5 (Factory Complete Vision) **3/6** ; S74 (atelier
fork) debloque sous reserve de l'audit gate S73.
