# Sprint 73 Phase D Preflight

Date: 2026-06-04
HEAD: `47c9ff7`
Verdict: **EXECUTE**

## Evidence Rules
- Claim policy: every claim below cites a path:line, a command output, a URL/date, or an explicit assumption.
- Local sources read:
  - `.planning/active/sprint73_plan.md` (§Phase D :241-285), `.planning/active/sprint73_kickoff.md` (§4 D2/D3 :261-361)
  - `crates/nexus-coordinator-rs/src/search.rs` (full, 486 lines)
  - `crates/nexus-coordinator-rs/src/public_feed.rs` (:26-87 payloads, :251-320 validation)
  - `crates/nexus-coordinator-rs/src/db.rs` (:16-229 MIGRATIONS array, :258-296 open/schema_version, :1099-1101/1253 schema_version test)
  - `crates/nexus-coordinator-rs/src/proof_card.rs` (:24-28, :55-56, :92 — archive_hash naming)
  - `crates/nexus-shell-daemon-core/src/browse.rs` (:169-225 BrowseEntry)
  - `crates/nexus-shell-daemon/src/http.rs` (:1957-2029 search_handler JSON)
  - `crates/nexus-shell-daemon/src/feed_sync.rs` (:240-289 hot ingest path)
  - `crates/nexus-shell-daemon/src/runtime.rs` (:778 boot rebuild call site)
  - `docs/protocol/PUBLIC_FEED_SPEC.md` (§2.1 :36-73, §2.2 :74-85, §9 :295-318)
  - `docs/security/THREAT_MODEL.md` (§11 :543-605)
  - `.planning/research/s70_s72_rrv_research.md` (:1-13 fossil flag, :980-995 "bump v2" claim)
  - `Cargo.lock` (libsqlite3-sys :4564-4566, rusqlite :7219-7222, rusqlite_migration :7234-7237)
- Commands run: `git log --all --oneline --grep`, `grep`/`rg` over crates+docs, context7 `/websites/sqlite_docs` FTS5 query.
- External: context7 `/websites/sqlite_docs` (SQLite FTS5 virtual-table creation, UNINDEXED, content-table sync) — 2026-06-04.

## Scope
- Plan source: `.planning/active/sprint73_plan.md` §Phase D (D.1-D.5, lines 241-285).
- Target files:
  - `crates/nexus-coordinator-rs/src/db.rs` (:211-222 → migration **M17** = 17th `M::up`, DROP+CREATE `search_index` + 4 UNINDEXED columns)
  - `crates/nexus-coordinator-rs/src/search.rs` (:7-16 SearchResult +4 fields; :101-134 `IndexFields`/`extract_index_fields` extend; :153-174 `upsert_feed_entry` columns; :34-49 `index_entry`; :51-93 `search()` SELECT offsets)
  - `crates/nexus-shell-daemon/src/http.rs` (:2007-2020 `search_handler` JSON additive +4 keys)
  - `.planning/research/s73_searchmanifest_index_node_design.md` (NEW — design note, zero wire code)
  - `docs/rust/PATTERNS.md` (FTS5 hot reindex + UNINDEXED triplet enrichment)
- Deps/APIs/specs: **none new** (kickoff §"Versions deps confirmees" :62-71; lockfile confirmed below).
- Security/protocol surfaces: `search_index` (local FTS5, not iroh-synced); `ReleasePublishedPayload` (read-only consumer, no wire change); THREAT_MODEL §11 (touch only, already updated for Phase C).
- Tests expected (plan D.3): `search_result_carries_provenance_triplet`, `migration_m17_recreates_index_unindexed`, `search_result_null_triplet_for_non_release_op`, `enriched_fields_unindexed_not_matchable`, `search_handler_json_includes_triplet`.

## S1a OSS Prior Art
- Domain: FTS5 UNINDEXED column semantics + schema evolution of an FTS5 virtual table (standalone, non external-content).
- Sources:
  - context7 `/websites/sqlite_docs` (2026-06-04): FTS5 virtual tables are configured at `CREATE VIRTUAL TABLE` time with their column list and per-column options (`UNINDEXED`, `content=`, `tokenize=`). The documented synchronization/evolution path is recreate + repopulate (content-table + triggers pattern), not in-place `ALTER`.
  - In-repo precedent already proves both claims: `db.rs:211-222` (M15) declares `project_id UNINDEXED`, `op_type UNINDEXED`, `source_type UNINDEXED`; `search.rs:254-261` (`test_search_index_feed_entry`) asserts that an `op_type`/`project_id` token does NOT MATCH (UNINDEXED = excluded from the full-text index) yet the same row is returned via SELECT in `search()` (`search.rs:67-87` SELECTs `project_id`/`op_type`/`source_type` directly). This is exactly the (a) "returnable via SELECT but excluded from MATCH" guarantee the plan relies on.
  - rusqlite #1226 (2024, kickoff :38): `Mutex<Connection>` is the correct single-conn pattern under current load.
- Finding: **APPROACH-ALIGNED**.
  - (a) A UNINDEXED column is retrievable via SELECT but excluded from MATCH — confirmed by docs + the in-repo M15/test precedent. The 4 new triplet columns (`repo_url`, `commit_sha`, `archive_hash`, `provenance_hash`) being UNINDEXED is the right call: a 40/64-hex hash is not a natural-language token, so a MATCH against it is meaningless and inflates the index (D2 "Rejete: colonnes INDEXED").
  - (b) DROP+CREATE+rebuild is the only sound chemin: FTS5 virtual tables cannot be `ALTER TABLE ... ADD COLUMN`-ed (SQLite rejects ALTER on virtual tables; no documented FTS5 ADD COLUMN). Recreate + `rebuild_from_feed` is the canonical evolution path and is safe here because the index is integrally reconstructible from `public_feed` (`search.rs:176-195` `rebuild_from_feed`; spec §6 replay).
- Impact: none — plan matches mature practice and the project's own M15 precedent.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `rusqlite`, `libsqlite3-sys` (bundled SQLite), `rusqlite_migration`. No new dependency is introduced by Phase D (kickoff :62-71).
- Commands/sources:
  - `Cargo.lock:4564-4566` → `libsqlite3-sys 0.34.0` (checksum `91632f3b...`). libsqlite3-sys 0.34.0 bundles **SQLite 3.50.x** (the `bundled` feature of the 0.34 line ships the 3.50 series; consistent with kickoff §3 :56-64 "SQLite 3.50.x"). FTS5 UNINDEXED + `INSERT OR REPLACE` + WAL are all stable since well before 3.50 — no behavioral risk at this version.
  - `Cargo.lock:7219-7222` → `rusqlite 0.36.0`. `Cargo.lock:7234-7237` → `rusqlite_migration 2.2.0`.
- CVE/advisory: no critical/high CVE on the SQLite/rusqlite line touching FTS5 virtual-table recreate, WAL, or `INSERT OR REPLACE`. M17 touches only local schema (no crypto/wire/network/sandbox/signing surface), so the S1b "blocking" criteria do not apply.
- Finding: **clean** (P2-PREFLIGHT-TRANSITIVE-DEPTH satisfied — exact pinned versions inspected in the lockfile, not just the manifest).

## S2 Historical Decisions
- Commands:
  - `git log --all --oneline --grep='SearchManifest|search_index|UNINDEXED|FTS5|M16|M17|provenance triplet' -i` → only S67 `f46bc66` (FTS5 introduction) and S73 `47c9ff7` (Phase C) touch the search index; no prior reverted M17/triplet decision exists.
  - `git log --all --oneline -- crates/nexus-coordinator-rs/src/search.rs` → `47c9ff7` (Phase C) + `f46bc66` (S67 Phase B). Clean, linear history.
- Decisions crossed:
  1. **Migration numbering / M16 vs M17 collision** — RESOLVED, no collision. `db.rs:16-229` has exactly **16 `M::up` entries** (`grep -c 'M::up'` = 16), the last being M16 (`ALTER TABLE tasks ADD COLUMN result_text`, S72 Phase D, db.rs:228). M17 is the **17th** entry, appended after M16. `rusqlite_migration 2.2.0` tracks `user_version` = applied-migration count (db.rs:227 comment), so appending one new `M::up` advances `user_version` 16→17 monotonically. The `schema_version()` row table (db.rs:289-296) is a SEPARATE hard-coded value (`INSERT ... VALUES (1)`, db.rs:21) and is asserted == 1 in tests (db.rs:1101, :1253) — it is INDEPENDENT of the migration count, so **no test breaks** when M17 is added. Confirmed non-blocking.
  2. **Fossil "bump v2" claim (kickoff §4 D3 DESIGN-CONFLICT POTENTIEL)** — `.planning/research/s70_s72_rrv_research.md:984-986`: "SearchManifest dans le feed: faut-il un nouveau FEED_FORMAT_VERSION? Oui, ajouter un variant a l'enum `PublicFeedOperation` est un breaking change (cf. PUBLIC_FEED_SPEC.md §9). Bump a version 2." This claim is **stale and reasons about the wrong mechanism** (adding a typed enum variant), and it is contradicted by the live authoritative spec.
     - Reverse-commit check: the fossil is self-flagged at `:4-13` as "candidate/fossil mixed" with a 2026-05-22 amendment explicitly demoting its SearchManifest/S72 content to "candidats S71+ sauf import explicite par le futur kickoff". It was never imported as an active decision.
     - The LIVE spec settles it: `PUBLIC_FEED_SPEC.md §9.1 :307-318` states "Adding a new operation type is **NOT a breaking change**" (raw-op forward compat, pattern P51), and §2.2 :74-85 already lists `SearchManifestPublished` as a defined-but-not-implemented future op. `CLAUDE.md:355-357` (pre-launch raw-op policy) confirms a new op does NOT bump `FEED_FORMAT_VERSION`. `public_feed.rs:73-87` implements the raw-op path (`FeedEntry.op` is `serde_json::Value`, unknown ops stored+propagated).
     - Classification: **confirmed-superseded** (the live spec + CLAUDE.md policy + the fossil's own amendment all post-date and overrule it). **Non-blocking** for Phase D specifically because D3 DEFERS all wire code — Phase D writes a design note and zero `SearchManifestPublished` op. The conflict cannot bite a no-wire phase.
  3. **Tantivy freeze** — `CLAUDE.md:306` ("FTS5 pour RRV @protocole S67, Tantivy en gate post-S75 si >50K docs"); plan §7 scope cut #9. Phase D stays on FTS5; does not reopen. Non-blocking.
- Finding: **clean** (one stale fossil claim, confirmed-superseded by the live spec + policy + its own amendment; does not bite a defer-only phase).

## S3 Local Patterns And Threat Model
- Threats/contracts checked: THREAT_MODEL §11 Search surface (`docs/security/THREAT_MODEL.md:543-605`) — T-SEARCH-INJECTION, T-CURATOR-VOUCH, T-SEARCH-DOS.
- HARDENING_ROADMAP status: no Phase-D pre-requirement; §11 already lists the search-DOS residual as "acceptable pre-launch" (THREAT_MODEL:597). Plan §7 #11 carries per-client rate-limit re-eval to Phase E, not D.
- Analysis:
  - M17 DROP/recreate does NOT regress any covered threat. The recreate happens once at migration time inside `CoordinatorDb::open()` (db.rs:258-274) before the boot rebuild (runtime.rs:778); the index is reconstructible from the durable signed feed (no unique data lost; spec §6 replay). T-SEARCH-DOS amplification is unaffected — M17 is a one-shot schema op, not on the hot ingest path. The hot path stays the O(1) incremental `upsert_feed_entry` from Phase C (search.rs:153-174), bounded by the existing GCRA 5 ops/min + entry_hash dedup (THREAT_MODEL:592-596, already updated for Phase C).
  - **UNINDEXED triplet adds no new attack surface.** The 4 new columns are sourced from `ReleasePublishedPayload`, whose fields are already validated at feed insert time (`public_feed.rs:265-288`: `repo_url` must be `https://`, `commit_sha` hex-40, `artifact_hash` hex-64, `provenance_hash` hex-64-or-absent). A malicious `repo_url`/`commit_sha` cannot enter the feed without passing `validate_known_operation`. Even so, the triplet is returned **verbatim** in the search JSON (it is UNINDEXED — not matchable, not interpreted), exactly like BrowseEntry already returns these fields (browse.rs:206-224). No injection at JSON-return: `search_handler` (http.rs:2007-2020) serializes via `serde_json::json!` (proper escaping), and the front renders text, not HTML-eval. No new T-* needed; a one-line touch noting the triplet is carried UNINDEXED is sufficient.
- Finding: **clean** (no regression on T-SEARCH-*; triplet is pre-validated at feed insert and returned non-matchable; THREAT_MODEL §11 already reflects Phase C hot reindex).

## S4 Protocol And Wire Invariants
- Wire/security files checked (full producer→consumer trace per P2-PREFLIGHT-WIRE-CONTRACT-DEPTH):
  - `FEED_FORMAT_VERSION` = `1` (public_feed.rs:20) — **unchanged**. Phase D adds no feed op, no canonical change.
  - `PROJECT_ANNOUNCEMENT_VERSION`, `TASK_FORMAT_VERSION` — not touched by Phase D (search_index is local-only; not synced over iroh-docs/gossip — kickoff §1.4 :141-158, db.rs comment "local persistence only").
  - M17 = SQLite schema (local daemon), NOT a wire format. Pre-launch policy unaffected (CLAUDE.md:354-366; kickoff §1.4).
- **Field-name producer→consumer mapping (the load-bearing finding for the implementer):**

  | Concept | Feed source (producer) | Existing denormalized consumer | Plan's `SearchResult` field (D2) | JSON key (http.rs) |
  |---|---|---|---|---|
  | repo URL | `ReleasePublishedPayload.repo_url` (public_feed.rs:34; spec §2.1) | `BrowseEntry.repo_url` (browse.rs:210) | `repo_url` | `repo_url` |
  | commit | `ReleasePublishedPayload.commit_sha` (public_feed.rs:35) | — (BrowseEntry has none) | `commit_sha` | `commit_sha` |
  | archive/artifact hash | **`ReleasePublishedPayload.artifact_hash`** (public_feed.rs:36; spec §2.1 :45) | **`BrowseEntry.archive_hash`** (browse.rs:206) / `ProofCardInput.archive_hash` (proof_card.rs:27) | **`archive_hash`** (plan name) | `archive_hash` |
  | provenance | `ReleasePublishedPayload.provenance_hash: Option<String>` (public_feed.rs:37-38) | `BrowseEntry.provenance_hash` (browse.rs:214) | `provenance_hash` | `provenance_hash` |
  | open-source flag | `ReleasePublishedPayload.is_open_source: bool` (public_feed.rs:39) | `BrowseEntry.is_open_source: bool` (browse.rs:224) | `is_open_source: bool` | `is_open_source` |

  - **NAME BRIDGE REQUIRED (non-blocking, but must be coded correctly):** the feed payload field is **`artifact_hash`**, while the plan/SearchResult/BrowseEntry field is **`archive_hash`**. `extract_index_fields` (search.rs:114-134) must read `op.get("artifact_hash")` from the `ReleasePublishedPayload` JSON and store it into the `archive_hash` SearchResult column. Reading `op.get("archive_hash")` would silently yield `None` for every real release (the existing Phase-C test `search.rs:237-242` already feeds `"artifact_hash": "deadbeef"`, proving the source key). The plan's column name `archive_hash` is deliberately chosen to mirror BrowseEntry/ProofCard (the fork consumer S74 expects `archive_hash`), so the column name is correct — only the EXTRACTION key differs. Document this in the commit body and the PATTERNS note.
  - `SearchResult` +4 `Option<String>` + `is_open_source: bool` with serde default = **legitimate runtime tolerance**, NOT historical wire compat: an old index row (pre-M17) or a non-release op (CuratorVouched) yields `None`/`false` rather than a deserialization error. This is local-DTO tolerance (kickoff §1.4 :154-157), consistent with the pre-launch rationale-in-doc requirement. `provenance_hash` is already `Option` at the source (public_feed.rs:37-38), so `None` is a real value, not just legacy.
- VERSION/domain/canonical status: all `*_VERSION` constants unchanged; no `DOMAIN_*`, no `canonical_bytes`, no signing-domain touched. `search_index` is downstream of the signed feed, never an input to a signature.
- Day 0 status: **preserved**. D2 (UNINDEXED + M17 DROP/recreate) and D3 (defer SearchManifest, design note only) are exactly the frozen kickoff decisions; FTS5 stays the engine (Tantivy frozen); FEED_FORMAT_VERSION stays 1.
- Finding: **clean** (zero wire change; the only sharp edge is the `artifact_hash`→`archive_hash` extraction bridge, which is a naming discipline note, not a design conflict).

## Plan Adaptation
Not required (verdict is EXECUTE, no S1a APPROACH-NAIVE / LIB-EXISTS).

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks / carry-over:
  1. **Extraction key `artifact_hash` (source) → `archive_hash` (column)** — must be coded as a deliberate bridge in `extract_index_fields`; add a test asserting a `ReleasePublished` hit's `archive_hash` is populated from the payload `artifact_hash` (the planned `search_result_carries_provenance_triplet` should assert this exact mapping, not just non-null).
  2. **Phase-C rowid CONCERN carried, not aggravated** — `search.rs:148-152` documents that feed rows own rowid `[1, max seq]` and browse-sourced `index_entry` rows (auto rowid, currently test-only) must be partitioned at S74. M17 DROP/recreate REBUILDS feed rows from `public_feed` via `rebuild_from_feed` (keyed rowid=seq, search.rs:183-195), so the recreate preserves the existing rowid discipline and does not aggravate the S74 concern. Keep the tripwire doc comment; do not wire browse indexing in D.
  3. **`enriched_fields_unindexed_not_matchable` test** must MATCH on a hash value and assert zero hits — mirrors the existing `test_search_index_feed_entry` UNINDEXED assertion (search.rs:254-261); proven pattern.
  4. **Migration repopulation timing** — M17 runs inside `open()` (db.rs:270-271) and the boot rebuild also runs (runtime.rs:778); the plan's `migration_m17_recreates_index_unindexed` should assert that after migration + `rebuild_from_feed`, prior feed entries are searchable AND carry the triplet (no data loss; spec §6 replay).
- Scope cuts still honored (kickoff §7): #1 SearchManifest network broadcast → deferred (D3, design note only, zero wire — preserved); #9 Tantivy → frozen (FTS5 stays); #2/#3/#4 fork/search/open commands → S74 (D enriches SearchResult only, does not code the fork). FEED_FORMAT_VERSION stays 1 (pre-launch raw-op).

## Action
- **EXECUTE**: proceed with Phase D as planned (migration M17 DROP/recreate + 4 UNINDEXED columns + `is_open_source`; `SearchResult` +5 fields serde-default; `extract_index_fields` reads `artifact_hash`→`archive_hash` and the rest verbatim; `search()` SELECT offsets 7-11; `search_handler` JSON additive; design note + PATTERNS). Commit body must record the `artifact_hash`→`archive_hash` extraction bridge and the "M17 = local schema, no wire bump (FEED_FORMAT_VERSION=1)" rationale.
