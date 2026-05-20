# Sprint 67 — Plan (Factory Foundation)

**Ecrit** : 2026-05-20.
**Tip master** : `3821508`.
**Roadmap** : Sprint 1/3, v2.1 Arc 2 Factory + RRV @dev + Canari.

---

## S1 Etat verifie a l'entree

| Suite | Count | Commande | Observed |
|---|---|---|---|
| Rust nextest | 1349 | `cargo nextest run --workspace --locked` | |
| Rust doctests | ok | `cargo test --workspace --locked --doc` | |
| cargo fmt | 0 diff | `cargo fmt --all --check` | |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` | |
| Vitest | 269 | `(cd web && npm run test:unit)` | |
| size-limit | 6/6 | `(cd web && npm run size)` | |
| release build | ok | `cargo build -p nexus-shell-daemon --release` | |
| **Total** | **~1624** | | |

---

## S2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | FTS5 search @protocole dans le daemon | db.rs M15, search.rs NEW, http.rs endpoint, runtime.rs boot |
| D2 | sbfb-manifest crate partage + SBFB.json v2 | sbfb-manifest/ NEW, deploy.rs refactor, examples/ migration |
| D3 | CuratorVouched/CuratorDisendorsed feed ops | public_feed.rs variants, feed_materializer.rs |
| D4 | Feed entries read paginee | http.rs handler GET, db.rs filtre |
| D5 | sbfb-factory CLI crate + create + validate | sbfb-factory/ NEW, templates/, main.rs clap |

---

## S3 Graphe de dependances inter-phases

```
Phase A (Primitives daemon neutres)
  |
  |---> Phase B (FTS5 search)
  |       Depend de A : sbfb-manifest valide les browse entries
  |       indexees. CuratorVouched ops dans le feed = nouvelles
  |       entries a indexer. Feed/entries endpoint = source de
  |       donnees pour le search index.
  |
  |---> Phase C (sbfb-factory crate)
  |       Depend de A : sbfb-manifest est une dep de sbfb-factory.
  |       SBFB.json v2 est le format que Factory genere.
  |
  Phase B ---> Phase D (Factory provenance + dette)
  Phase C ---> Phase D
                Depend de B : THREAT_MODEL update (3/3 MANDATORY)
                  inclut la surface search.
                Depend de C : factory.provenance.json utilise le
                  meme template que create.
                |
                Phase D ---> Phase E (wrap-up)
```

Phase A est le fondement : elle produit sbfb-manifest + les
primitives daemon (feed ops, feed/entries, node_id optionnel)
dont dependent B (search indexe ces donnees) et C (Factory
utilise sbfb-manifest). Phase D consolide (provenance Factory,
dette, THREAT_MODEL 3/3). Phase E documente et ferme.

---

## S4 Phase A — Primitives daemon neutres

### S4.1 Scope

Phase A livre les 4 primitives daemon manquantes identifiees dans
SYNTHESIS §2.5 et les fondations du crate partage sbfb-manifest :

1. **sbfb-manifest crate** : struct `SbfbManifest` parsant
   SBFB.json v1 et v2, validation `validate()`, allowlist bridge
   methods.
2. **node_id optionnel** : `SbfbJson` dans deploy.rs refactorisee
   via import sbfb-manifest. `node_id: Option<String>` +
   suppression verification.
3. **CuratorVouched/CuratorDisendorsed** : 2 variantes dans
   `PublicFeedOperation` avec payloads et validation.
4. **GET /api/daemon/feed/entries** : endpoint pagine sequence-based
   avec filtres project_id et op_type.
5. **Migration exemples** : sbfb-explorer et sbfb-ideas SBFB.json
   vers v2 (sans node_id, avec schema_version: 2).

### S4.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/sbfb-manifest/Cargo.toml` | NEW — deps serde, serde_json, thiserror |
| `crates/sbfb-manifest/src/lib.rs` | NEW — SbfbManifest struct, parse, validate, BridgeMethodAllowlist |
| `Cargo.toml` workspace | Ajouter sbfb-manifest comme membre workspace |
| `crates/nexus-shell-daemon/Cargo.toml` | Ajouter dep sbfb-manifest |
| `crates/nexus-shell-daemon/src/deploy.rs` | Refactor SbfbJson → import sbfb-manifest. node_id Option. Supprimer verification l.119-128 |
| `crates/nexus-coordinator-rs/src/public_feed.rs` | Ajouter CuratorVouched + CuratorDisendorsed variants + payloads + validation |
| `crates/nexus-coordinator-rs/src/feed_materializer.rs` | Handle nouveaux op types dans materialize (log sans effet projet) |
| `crates/nexus-shell-daemon/src/http.rs` | Handler GET /api/daemon/feed/entries + query params (after_seq, limit, project_id, op_type) |
| `examples/sbfb-explorer/SBFB.json` | Migration v2 : schema_version: 2, supprimer node_id, ajouter champs v2 |
| `examples/sbfb-ideas/SBFB.json` | Migration v2 : idem |

### S4.3 Tests plan

1. `test_sbfb_manifest_parse_v1` — verifie que SbfbManifest parse un SBFB.json v1 (node_id present, pas de schema_version) sans erreur
2. `test_sbfb_manifest_parse_v2` — verifie que SbfbManifest parse un SBFB.json v2 complet avec tous les champs
3. `test_sbfb_manifest_validate_v2_rejects_missing_name` — verifie que validate() retourne erreur si name manquant en v2
4. `test_sbfb_manifest_validate_bridge_allowlist` — verifie que les methodes bridge declarees sont dans l'allowlist
5. `test_deploy_from_repo_accepts_no_node_id` — verifie que deploy accepte un SBFB.json sans node_id (v2)
6. `test_deploy_from_repo_warns_with_node_id` — verifie que deploy emet un warning si node_id present (deprecated)
7. `test_curator_vouched_roundtrip` — verifie serde roundtrip CuratorVouched insert + read
8. `test_curator_disendorsed_roundtrip` — verifie serde roundtrip CuratorDisendorsed
9. `test_curator_vouched_validation_rejects_bad_pubkey` — verifie que validate rejette un pubkey non hex-64
10. `test_curator_vouched_unknown_op_forward_compat` — verifie que try_parse_op retourne None pour un op_type inconnu (pattern raw-op P51)
11. `test_feed_entries_endpoint_paginated` — verifie que GET /api/daemon/feed/entries retourne les entries avec pagination (after_seq + limit)
12. `test_feed_entries_endpoint_filters_by_project_id` — verifie le filtre project_id

### S4.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-manifest --locked  # 4+ tests
cargo nextest run -p nexus-coordinator-rs -E 'test(curator_vouched)' --locked  # 3+ tests
cargo nextest run -p nexus-shell-daemon -E 'test(feed_entries)' --locked  # 2+ tests
cargo nextest run -p nexus-shell-daemon -E 'test(deploy_from_repo)' --locked  # includes new tests
grep -q '"schema_version": 2' examples/sbfb-explorer/SBFB.json
grep -q '"schema_version": 2' examples/sbfb-ideas/SBFB.json
```

### S4.5 Commit cible

`feat(daemon): Sprint 67 Phase A — sbfb-manifest + feed primitives + SBFB.json v2`

Body : Contexte, Fichiers, Delta tests, Verification §7.4,
Scope cuts respectes, G8 traceability, Pre-launch protocol,
Carry closure.

---

## S5 Phase B — FTS5 search @protocole

### S5.1 Scope

Phase B livre le search FTS5 local dans le daemon et complete le
3/3 MANDATORY P2-THREAT-MODEL-FEED-SURFACE.

1. **Migration M15** : CREATE VIRTUAL TABLE search_index USING
   fts5(project_name, category, description, op_type, payload,
   tokenize='unicode61'). Indexation au boot (BrowseEntries +
   FeedEntries existants).
2. **search.rs module** : dans nexus-coordinator-rs.
   `index_browse_entry()`, `index_feed_entry()`,
   `search(query, limit, offset)` retourne Vec<SearchResult>.
3. **GET /api/daemon/search** : endpoint HTTP dans http.rs.
   Query params : `q` (requis), `limit` (default 20, max 100),
   `offset` (default 0). Reponse : results + total + took_ms.
4. **Bridge method `search`** : schema Zod, dispatch useBridge.ts,
   SDK sbfb-bridge.js.
5. **THREAT_MODEL.md §10 enrichi** : T-SEARCH-INJECTION
   (sanitizer strip HTML, reject NUL bytes), T-CURATOR-VOUCH
   (endorsement spam via feed), T-SEARCH-DOS (rate limit query).
   CLOSE P2-THREAT-MODEL-FEED-SURFACE 3/3 MANDATORY.

### S5.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/nexus-coordinator-rs/src/db.rs` | M15 migration FTS5 virtual table |
| `crates/nexus-coordinator-rs/src/search.rs` | NEW — index + query functions |
| `crates/nexus-coordinator-rs/src/lib.rs` | pub mod search |
| `crates/nexus-shell-daemon/src/http.rs` | Handler GET /api/daemon/search |
| `crates/nexus-shell-daemon/src/runtime.rs` | Boot indexation call |
| `web/src/lib/protocol.ts` | Schema Zod search method |
| `web/src/hooks/useBridge.ts` | Dispatch case search |
| `web/public/sbfb-bridge.js` | SDK function search() |
| `examples/sbfb-explorer/sbfb-bridge.js` | Sync copy |
| `examples/sbfb-ideas/sbfb-bridge.js` | Sync copy |
| `docs/security/THREAT_MODEL.md` | §10 T-SEARCH-INJECTION + T-CURATOR-VOUCH + T-SEARCH-DOS |

### S5.3 Tests plan

1. `test_search_index_browse_entry` — verifie qu'un BrowseEntry indexe est retrouve par query MATCH
2. `test_search_index_feed_entry` — verifie qu'un FeedEntry ReleasePublished est retrouve
3. `test_search_query_returns_score` — verifie que les resultats ont un score bm25 non-zero
4. `test_search_query_pagination` — verifie que limit et offset fonctionnent
5. `test_search_query_empty_returns_empty` — verifie que une query sans match retourne []
6. `test_search_sanitizer_rejects_nul_bytes` — verifie que les NUL bytes sont stripped avant indexation
7. `test_search_endpoint_http` — verifie que GET /api/daemon/search?q=test retourne 200 + JSON
8. `test_search_bridge_method` — Vitest : verifie que le schema Zod search est valide
9. `test_threat_model_search_section_present` — grep THREAT_MODEL.md pour T-SEARCH

### S5.4 Critere d'acceptation

```bash
cargo nextest run -p nexus-coordinator-rs -E 'test(search)' --locked  # 6+ tests
cargo nextest run -p nexus-shell-daemon -E 'test(search)' --locked  # 1+ test
(cd web && npm run test:unit)  # includes bridge search test
grep -q "T-SEARCH" docs/security/THREAT_MODEL.md
grep -q "T-CURATOR-VOUCH" docs/security/THREAT_MODEL.md
```

### S5.5 Commit cible

`feat(search): Sprint 67 Phase B — FTS5 search @protocole + THREAT_MODEL feed 3/3`

Body : 9 sections obligatoires.

---

## S6 Phase C — sbfb-factory crate + template engine

### S6.1 Scope

Phase C livre le crate sbfb-factory avec les commandes `create`
et `validate`, le moteur de templates par copie+substitution, et
le template `static` embarque.

1. **sbfb-factory crate** : structure `crates/sbfb-factory/` avec
   Cargo.toml (deps: sbfb-manifest, clap derive, blake3, serde,
   serde_json, walkdir, zip, ed25519-dalek, thiserror).
2. **CLI clap** : `sbfb-factory create --template static --name
   <name> [--output <dir>]` et `sbfb-factory validate <path>`.
3. **Template engine** : copie de fichiers depuis template
   embarque (include_str! ou include_bytes!), substitution
   `{{name}}` et `{{version}}`. Generation SBFB.json v2 via
   sbfb-manifest.
4. **Template `static`** : index.html minimal + sbfb-bridge.js
   copy + SBFB.json v2 + README.md + .gitignore.
5. **factory.template.lock** : JSON avec template_id,
   template_version, template_hash (BLAKE3 du contenu template),
   generated_at, variables.
6. **Secret scanner basique** : regex patterns (AWS keys, GitHub
   tokens, private keys) dans `validate` subcommand.
7. **Test path traversal** : validate rejette les paths avec
   `..` ou symlinks.

### S6.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/sbfb-factory/Cargo.toml` | NEW — deps sbfb-manifest, clap, blake3, serde, walkdir, zip, ed25519-dalek, thiserror |
| `crates/sbfb-factory/src/main.rs` | NEW — clap CLI, create + validate subcommands |
| `crates/sbfb-factory/src/template_engine.rs` | NEW — copie + substitution + SBFB.json generation |
| `crates/sbfb-factory/src/template_lock.rs` | NEW — factory.template.lock generation |
| `crates/sbfb-factory/src/secret_scanner.rs` | NEW — regex patterns scan |
| `crates/sbfb-factory/src/templates/static/` | NEW — template files embarques |
| `Cargo.toml` workspace | Ajouter sbfb-factory comme membre workspace |

### S6.3 Tests plan

1. `test_create_generates_sbfb_json_v2` — verifie que `create` genere un SBFB.json v2 valide (parsable par sbfb-manifest)
2. `test_create_generates_index_html` — verifie que `create` genere un index.html
3. `test_create_generates_template_lock` — verifie que factory.template.lock est genere avec hash BLAKE3
4. `test_create_substitutes_name` — verifie que {{name}} est remplace dans les fichiers generes
5. `test_validate_accepts_valid_manifest` — verifie que `validate` accepte un SBFB.json v2 valide
6. `test_validate_rejects_invalid_manifest` — verifie que `validate` rejette un manifest sans name
7. `test_secret_scanner_detects_aws_key` — verifie que le scanner detecte un pattern AKIA
8. `test_secret_scanner_detects_github_token` — verifie que le scanner detecte ghp_/gho_
9. `test_path_traversal_rejected` — verifie que validate rejette un chemin avec `../`
10. `test_symlink_rejected` — verifie que validate rejette un symlink dans le workspace

### S6.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory --locked  # 10+ tests
cargo build -p sbfb-factory  # compile sans erreur
# Verification fonctionnelle manuelle :
# sbfb-factory create --template static --name test-app --output /tmp/test
# ls /tmp/test/SBFB.json /tmp/test/index.html /tmp/test/factory.template.lock
```

### S6.5 Commit cible

`feat(factory): Sprint 67 Phase C — sbfb-factory CLI + template engine + create + validate`

Body : 9 sections obligatoires.

---

## S7 Phase D — Factory provenance + dette residuelle

### S7.1 Scope

Phase D complete les artefacts Factory (provenance), absorbe les
P2 residuels, et documente les nouveaux patterns.

1. **factory.provenance.json** : generation dans sbfb-factory
   `create`. Contient : schema_version, template_hash,
   variables_hash, output_hash (BLAKE3 du workspace genere),
   generated_at. La signature Ed25519 est optionnelle S67 (le
   daemon n'a pas encore fourni sa keypair a Factory — c'est
   S68+ publish path). Le hash est suffisant pour la tracabilite
   locale.
2. **Test determinisme** : meme template + meme variables =
   meme output_hash (factory.provenance.json). Preuve
   d'idempotence.
3. **P2-66-2 BlobStore pattern** : ajouter P52 "Backend-agnostic
   enum with Deref" dans PATTERNS.md.
4. **P2-66-1 documentation** : note dans PATTERNS.md sur la
   limitation connue des tests feed republish (pas d'assertion
   iroh-docs).

### S7.2 Livrables

| Fichier | Changement |
|---|---|
| `crates/sbfb-factory/src/provenance.rs` | NEW — factory.provenance.json generation |
| `crates/sbfb-factory/src/main.rs` | Wiring provenance dans create subcommand |
| `docs/rust/PATTERNS.md` | P52 BlobStore enum + note feed republish limitation |

### S7.3 Tests plan

1. `test_provenance_hash_deterministic` — verifie que meme inputs = meme output_hash
2. `test_provenance_template_hash_matches_lock` — verifie que template_hash dans provenance == template_hash dans lock
3. `test_provenance_json_parsable` — verifie que factory.provenance.json est un JSON valide

### S7.4 Critere d'acceptation

```bash
cargo nextest run -p sbfb-factory -E 'test(provenance)' --locked  # 3+ tests
grep -q "P52" docs/rust/PATTERNS.md
```

### S7.5 Commit cible

`feat(factory): Sprint 67 Phase D — factory provenance + P52 BlobStore pattern + dette`

Body : 9 sections obligatoires.

---

## S8 Phase E — Wrap-up + verification

### S8.1 Scope

Phase E produit les artefacts de cloture.

1. **verification.md** : fail-fast checklist executee.
2. **sprint68_audit_plan.md** : plan audit S67, 9 tracks.
3. **CLAUDE.md** : mise a jour etat, compteurs, carries S68.
4. **SPRINT_LOG.md** : row S67.

### S8.2 Livrables

| Fichier | Changement |
|---|---|
| `.planning/active/sprint67_verification.md` | NEW — fail-fast |
| `.planning/active/sprint68_audit_plan.md` | NEW — 9 tracks |
| `CLAUDE.md` | Etat S67 DONE, compteurs, carries |
| `docs/claude/SPRINT_LOG.md` | Row S67 |

### S8.3 Tests plan

Pas de nouveaux tests (documentation seulement).

### S8.4 Critere d'acceptation

```bash
test -f .planning/active/sprint67_verification.md
test -f .planning/active/sprint68_audit_plan.md
```

### S8.5 Commit cible

`docs(sprint67): Sprint 67 Phase E — verification + wrap-up`

Body : 9 sections obligatoires.

---

## S9 Delta tests estime

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +12 | +0 | sbfb-manifest 4 + deploy 2 + curator_vouched 4 + feed_entries 2 |
| B | +7 | +1 | search index 6 + search endpoint 1 + bridge search Vitest 1 |
| C | +10 | +0 | create 4 + validate 2 + secret scanner 2 + path traversal 2 |
| D | +3 | +0 | provenance deterministic 2 + provenance lock match 1 |
| E | +0 | +0 | documentation seulement |
| **Total** | **+32** | **+1** | |
| **Sortie estimee** | **1381** | **270** | **~1657** |

---

## S10 Fail-fast checklist

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1381 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build daemon | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | release build factory | `cargo build -p sbfb-factory --release` | ok |
| 7 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 8 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 9 | Vitest | `(cd web && npm run test:unit)` | >= 270 |
| 10 | npm build | `(cd web && npm run build)` | ok |
| 11 | size-limit | `(cd web && npm run size)` | 6/6 |
| 12 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 13 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 14 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical |
| 15 | sbfb-manifest tests | `cargo nextest run -p sbfb-manifest --locked` | 4+ pass |
| 16 | sbfb-factory tests | `cargo nextest run -p sbfb-factory --locked` | 13+ pass |
| 17 | search tests | `cargo nextest run -p nexus-coordinator-rs -E 'test(search)' --locked` | 6+ pass |
| 18 | curator_vouched tests | `cargo nextest run -p nexus-coordinator-rs -E 'test(curator_vouched)' --locked` | 3+ pass |
| 19 | feed_entries endpoint | `cargo nextest run -p nexus-shell-daemon -E 'test(feed_entries)' --locked` | 2+ pass |
| 20 | deploy no node_id | `cargo nextest run -p nexus-shell-daemon -E 'test(deploy_from_repo)' --locked` | includes new |
| 21 | search endpoint HTTP | `cargo nextest run -p nexus-shell-daemon -E 'test(search)' --locked` | 1+ pass |
| 22 | bridge search Vitest | `(cd web && npm run test:unit)` includes search | pass |
| 23 | SBFB.json v2 examples | `grep -q '"schema_version": 2' examples/sbfb-explorer/SBFB.json` | present |
| 24 | THREAT_MODEL search | `grep -q "T-SEARCH" docs/security/THREAT_MODEL.md` | present |
| 25 | THREAT_MODEL curator | `grep -q "T-CURATOR-VOUCH" docs/security/THREAT_MODEL.md` | present |
| 26 | PATTERNS P52 | `grep -q "P52" docs/rust/PATTERNS.md` | present |
| 27 | factory no daemon dep | `! grep -q "nexus-shell-daemon" crates/sbfb-factory/Cargo.toml` | absent |
| 28 | verification.md | `test -f .planning/active/sprint67_verification.md` | exists |
| 29 | audit_plan S68 | `test -f .planning/active/sprint68_audit_plan.md` | exists |

---

## S11 Scope cuts

Reprise exhaustive depuis kickoff §7 :

| # | Item | Sprint cible | Rationale |
|---|---|---|---|
| 1 | Preview ephemere (POST /api/v1/preview/load) | S68 | Depend de sbfb-factory publish path |
| 2 | Diff engine avance sbfb-factory | S68+ | Pas requis pour create+validate MVP |
| 3 | Page React /factory | S68+ | CLI-first, UI optionnelle |
| 4 | Proof Cards computation | S68 | Depend de FTS5 + sbfb-manifest sortie S67 |
| 5 | SearchManifest wire format | S70+ | Couche protocole, pas service |
| 6 | Babel app generation | S69 | Depend sbfb-factory + Proof Cards + pilote |
| 7 | @dev index tree-sitter | S68-S69 | @protocole d'abord (D6 v4) |
| 8 | Bridge method proof_card_get | S68+ | Proof Cards pas en S67 |
| 9 | Template react-vite | S69+ | 2 templates max S67 |
| 10 | Factory audit log JSONL | S68+ | Pas requis pour create+validate MVP |
| 11 | CuratorVouched UI shell | S70+ | Gouvernance Full UI |
| 12 | Publish path sbfb-factory → daemon | S68+ | S67 = local only |
| 13 | Feed format version bump | post-launch | Pre-launch protocol |
| 14 | Fuzzing cargo-fuzz/proptest | post-audit | Hors scope feature sprint |

---

## S12 Risks

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | FTS5 indexation lente sur gros corpus | Low | Medium | Volume < 500 entries pre-launch. Indexation incrementale |
| R2 | sbfb-factory crate deps conflict workspace | Medium | Medium | Deps minimales. Pas de dep daemon. Test compilation workspace |
| R3 | SBFB.json v2 migration casse exemples | Low | High | #[serde(default)] compat. Tests v1 dans v2 struct |
| R4 | CuratorVouched semantique trop legere S68 | Medium | Low | Payload minimal extensible |
| R5 | Template engine trop simple | Medium | Low | Scope cut conditionals S68+ |
| R6 | iroh 0.98 bugs persistence | Low | High | E2E restart test vert S66. Gate 1 |
| R7 | 3 crates nouveaux alourdissent CI | Low | Low | Crates petits, CI compile workspace entier |

---

## S13 Checkpoint de cloture

Conditions pour dire "sprint ferme" :
- 29/29 fail-fast verts
- 4 commits feat (Phase A, B, C, D) + 1 commit docs (Phase E)
- verification.md + audit_plan S68 ecrits
- PATTERNS.md mis a jour (P52 BlobStore)
- THREAT_MODEL.md enrichi (T-SEARCH + T-CURATOR-VOUCH)
- CLAUDE.md + SPRINT_LOG.md a jour
- sbfb-factory compile et `sbfb-factory create` fonctionne
- sbfb-manifest partage entre daemon et factory
- Factory ne depend PAS de nexus-shell-daemon-core
- Memory nexus_grid_pivot.md tip + compteurs mis a jour
