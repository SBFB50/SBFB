# Sprint 73 Phase C Preflight

Date: 2026-06-04
HEAD: `a4e1542`
Verdict: **EXECUTE**

## Evidence Rules
- Claim policy: every claim below cites a repo path:line, a command and its
  relevant output, a URL/date, or an explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `crates/nexus-coordinator-rs/src/search.rs` (full)
  - `crates/nexus-coordinator-rs/src/db.rs` (full)
  - `crates/nexus-shell-daemon/src/feed_sync.rs` (full)
  - `crates/nexus-shell-daemon/src/runtime.rs:760-799`
  - `crates/nexus-shell-daemon/src/http.rs:1955-2014` (search_handler)
  - `crates/nexus-coordinator-rs/src/public_feed.rs:1-160`
  - `crates/nexus-shell-daemon-core/src/feed_limiter.rs` (GCRA quota)
  - `docs/security/THREAT_MODEL.md:492-601` (§10 feed surface, §11 search surface)
  - `.planning/active/sprint73_plan.md` §Phase C (199-238), §3, §7, §8
  - `.planning/active/sprint73_kickoff.md:33-35, 237-246, 556-564, 607`
  - memory `feedback_approach.md`, `feedback_context7_systematic.md`
- Commands run (relevant output inline below): `git rev-parse`, `git log --all`,
  `grep` on `Cargo.lock`, registry inspection of `libsqlite3-sys-0.34.0`.

## Scope
- Plan source: `.planning/active/sprint73_plan.md` §Phase C lines 199-238.
- Target files (verified):
  - `crates/nexus-coordinator-rs/src/search.rs` — NEW `upsert_feed_entry()`,
    NEW `extract_index_fields()` helper, `rebuild_from_feed()` refactored to
    reuse the helper.
  - `crates/nexus-shell-daemon/src/feed_sync.rs` — call `upsert_feed_entry`
    inside the same `db` lock scope, right after `Ok(seq)` from
    `insert_feed_entry`.
  - `crates/nexus-coordinator-rs/src/db.rs` — explicit `busy_timeout` at open.
- Deps/APIs/specs touched: SQLite FTS5 (via rusqlite 0.36.0 / libsqlite3-sys
  0.34.0). No new dependency. No `Cargo.toml` change.
- Security/protocol surfaces: NONE on the wire. `search_index` is a local-only
  FTS5 virtual table (db.rs:212-222, M15), reconstructible from `public_feed`.
  THREAT_MODEL §11 (search surface) is doc-relevant (see S3).
- Tests expected (plan §C.3): `feed_ingest_indexes_entry_hot`,
  `reindex_hot_is_idempotent`, `extract_index_fields_shared_with_rebuild`,
  `hot_reindex_does_not_block_search_reader`, `rebuild_from_feed_still_repairs`.

## S1a OSS Prior Art
- Domain: maintaining a **standalone** (non-external-content) SQLite FTS5
  full-text index incrementally, in sync with an append-only source-of-truth
  table, under WAL, without triggers — using `INSERT OR REPLACE INTO ft(rowid,
  ...)` keyed on a monotone INTEGER.
- Sources (dated):
  - SQLite FTS5 official reference `https://sqlite.org/fts5.html` (current, ref
    SQLite 3.49+/3.53) — an FTS5 table has an implicit `INTEGER PRIMARY KEY`
    named `rowid`; for standalone (no `content=`) tables the canonical
    idempotent upsert is `INSERT OR REPLACE INTO ft(rowid, col...)`; the
    `'delete'`/`'rebuild'`/`'optimize'` special commands and the
    insert/update/delete **triggers** are the external-content pattern (i.e.
    `content='tbl'`), not the standalone pattern.
  - context7 `/websites/sqlite_docs` query "FTS5 standalone vs external content
    triggers" (2026-06-04) — confirms the trigger-based sync example is shown
    only under `CREATE VIRTUAL TABLE ... content='articles'
    content_rowid='id'`; the plain standalone FTS5 table is populated by direct
    `INSERT` (no trigger machinery).
  - SQLite User Forum threads (sqlite.org/forum) on FTS5 rowid / "REPLACE INTO"
    consistency (accessed 2026-06-04) — `INSERT OR REPLACE` on external-content
    content tables needs `recursive_triggers` to fire delete triggers, and
    badly written triggers omitting `rowid` corrupt the index. This is exactly
    why D1 chose **standalone + direct upsert by explicit rowid**, sidestepping
    the trigger hazard. Corroborates kickoff §C lines 34, 240-243.
  - sqlite.org/wal.html (current) — WAL: readers do not block writers and a
    writer does not block readers, across **separate connections**; a single
    connection serializes its own read/write transactions.
- Finding: **APPROACH-ALIGNED.** The D1 design (standalone FTS5, explicit
  `rowid = seq`, `INSERT OR REPLACE` for idempotency, shared JSON extraction
  helper, `rebuild` kept as repair) matches the documented SQLite-canonical
  approach for non-external-content FTS5 maintenance. The rejected alternatives
  (external-content triggers, contentless-delete, full O(N) rebuild per ingest)
  are correctly rejected per the same OSS evidence. No mature license-compatible
  library supersedes this (rusqlite already bundles SQLite; no extra crate
  needed).
- Impact: none. Plan stands.

## S1b Dependencies, CVEs, Release Notes
- Scanned (PRECISE pinned versions, honoring P2-PREFLIGHT-TRANSITIVE-DEPTH —
  inspected the locked version, not latest OSS):
  - `rusqlite 0.36.0` (Cargo.lock checksum
    `3de23c3319433716cf134eed225fe9986bc24f63bed9be9f20c329029e672dc7`).
  - `libsqlite3-sys 0.34.0` (Cargo.lock checksum
    `91632f3b4fb6bd1d72aa3d78f41ffecfcf2b1a6648d8c241dbe7dbfaf4875e15`).
  - `rusqlite_migration 2.2.0` (manages `user_version`; M16 already lands).
- Commands/sources:
  - `grep -A3 'name = "libsqlite3-sys"' Cargo.lock` -> version 0.34.0.
  - `grep -m1 "define SQLITE_VERSION " ~/.cargo/registry/src/*/libsqlite3-sys-0.34.0/sqlite3/sqlite3.h`
    -> `#define SQLITE_VERSION "3.49.2"`. **Authoritative bundled engine =
    SQLite 3.49.2** (single registry dir for this lock entry).
  - `grep "pub fn busy_timeout" ~/.cargo/registry/src/*/rusqlite-0.36.0/src/busy.rs`
    -> `pub fn busy_timeout(&self, timeout: Duration) -> Result<()>` at
    `busy.rs:26` — the API the plan calls EXISTS in 0.36.0.
- CORRECTION (non-blocking, P2-PREFLIGHT-TRANSITIVE-DEPTH discipline): the plan
  §3 and kickoff §C say "libsqlite3-sys 0.34.0 bundled = SQLite 3.50.x". The
  pinned crate actually bundles **3.49.2** (off by one minor). This does not
  change feasibility: `INSERT OR REPLACE` (since SQLite 3.0), WAL (since 3.7.0),
  and FTS5 standalone `INSERT OR REPLACE INTO ft(rowid,...)` upsert (FTS5 GA
  since 3.9.0, 2015) are all present in 3.49.2. The commit body / plan note
  should cite 3.49.2 rather than 3.50.x for accuracy.
- CVE check: no known critical/high advisory on rusqlite 0.36 / libsqlite3-sys
  0.34 / SQLite 3.49.2 affecting FTS5 INSERT/REPLACE, WAL, or the index path as
  of 2026-06-04. The feature surface (FTS5 standalone upsert, WAL,
  busy_timeout) is decade-stable. No crypto/wire/network/sandbox/signing code
  is touched.
- Finding: **clean** (one accuracy correction on the bundled-version string,
  non-blocking).

## S2 Historical Decisions
- Commands:
  - `git log --all --oneline -- crates/nexus-coordinator-rs/src/search.rs`
    -> single commit `f46bc66 feat(search): Sprint 67 Phase B — FTS5 search
    @protocole + THREAT_MODEL feed 3/3`.
  - `git log --all --oneline -S "search_index" -- .../db.rs` -> same single
    commit `f46bc66` (M15 DDL introduced there).
  - Reverse-commit check:
    `git log --all --oneline f46bc66..HEAD -- search.rs feed_sync.rs` -> EMPTY.
    No commit has touched these files since their introduction; the
    standalone-FTS5 / no-trigger / boot-rebuild design has not been reverted.
- Decisions crossed:
  - **f46bc66 (S67 Phase B)** chose a standalone FTS5 `search_index` (db.rs:212-222,
    `UNINDEXED` rowid columns, `tokenize='unicode61'`) populated by direct
    `INSERT` (search.rs:43-48) and rebuilt at boot (runtime.rs:773-782). Phase C
    EXTENDS this (adds a hot incremental upsert path; keeps rebuild as repair).
    It does not contradict the original decision — it completes the missing
    freshness path the kickoff §1 explicitly identifies as a gap
    (`runtime.rs:778`, `feed_sync.rs:260` insert without reindex).
  - The "no triggers / no contentless-delete / no O(N) rebuild per ingest"
    rejections are the kickoff's OWN Day-0 rationale (kickoff §C lines 33-35,
    237-246; plan §3 lines 56-59), not a prior decision being overturned. They
    are sourced to sqlite.org and consistent with S1a.
  - Tantivy is GELE (frozen, gate post-S75; CLAUDE.md:306; plan §7 cut #9;
    kickoff line 666). Phase C does not touch the engine choice — it stays FTS5.
    No conflict.
- Finding: **clean.** No documented decision with a still-valid rationale is
  contradicted; the only relevant commit (f46bc66) is extended, not reversed
  (confirmed reversion check = no reversion needed, extension only).

## S3 Local Patterns And Threat Model
- Threats/contracts checked (THREAT_MODEL §10 feed surface, §11 search surface):
  - **T-FEED-SPAM** (THREAT_MODEL:492-504): GCRA 5 ops/min per-author + 64 KB
    + PoW. Verified live in `feed_sync.rs:223-230` (`feed_limiter.check_author`)
    and `feed_limiter.rs` (`FEED_OPS_PER_MINUTE = 5`, `Quota::per_minute`).
    Phase C's upsert runs ONLY after the dedup check (feed_sync.rs:204-217) AND
    the rate-limit check (feed_sync.rs:223-230) have passed and a genuinely new
    entry was inserted (feed_sync.rs:260 `Ok(seq)`). So the hot reindex inherits
    the existing anti-spam gate; it cannot be invoked faster than 5/min/author.
  - **T-SEARCH-DOS** (THREAT_MODEL:581-595): the upsert is **O(1)** (single
    `INSERT OR REPLACE` of one row by rowid), NOT a per-ingest O(N) rebuild.
    This is the explicit DoS-amplification mitigation the kickoff cites (plan §8
    R3, kickoff §C line 245 "Rebuild complet a chaque ingest ... amplification
    DoS"). No amplification is introduced — actually an improvement over a naive
    rebuild approach.
  - **T-CURATOR-VOUCH** (THREAT_MODEL:565-579): mitigation row reads "GCRA
    5 ops/min + Ed25519 attribution + **boot reindex**" (line 578). Phase C
    upgrades "boot reindex" to "hot reindex"; this is strictly better
    (entries become attributable AND searchable immediately, still rate-limited
    and Ed25519-signed). The doc CLAIM at line 571-572 ("Le search index
    re-indexe au boot") and line 578 ("boot reindex") becomes stale after
    Phase C. The plan routes THREAT_MODEL edits to Phase A (§14/§3.1) and
    Phase D (PATTERNS), and does NOT list §11 for Phase C.
  - **T-SEARCH-INJECTION** (THREAT_MODEL:550-563): `sanitize_query` is the
    query-path defense (search.rs:18-32). Phase C touches the INDEX (write)
    path, not the query path; injection posture unchanged.
- HARDENING_ROADMAP status: no S73 pre-requirement is owed by Phase C (search
  freshness is a feature, not a hardening pre-req). HARDENING meta-staleness is
  a Phase A doc item (P2-HARDENING-ROADMAP-META-STALE), unrelated.
- Concurrency contract (precision note, non-blocking): the test
  `hot_reindex_does_not_block_search_reader` (plan §C.3 #4) — the codebase uses
  a SINGLE `Connection` behind a SINGLE `Arc<Mutex<CoordinatorDb>>` (db.rs:248-250).
  The search reader holds that Mutex for the whole `search()` call
  (http.rs:1979-2004); ingest holds it across insert+upsert (feed_sync.rs:232-281).
  Therefore reader/writer mutual exclusion is enforced at the **Rust Mutex**
  level, not by SQLite WAL (WAL's "readers don't block writers" applies to
  separate connections — sqlite.org/wal.html). The D1 rationale "meme tx WAL,
  invisible aux lecteurs concurrents" is technically imprecise: there is no true
  read/write concurrency on one connection; the non-blocking property the test
  can assert is that the short O(1) upsert does not measurably stall sequential
  search calls (Mutex held briefly). RECOMMENDATION (non-blocking): word the
  test as "search remains responsive / returns correct results while ingest
  upserts interleave" rather than asserting WAL-level concurrency; do NOT add a
  second connection (that is out of D1 scope and rejected as premature, plan §2
  D1 "pool connexions premature").
- Finding: **non-blocking notes only.**
  (1) THREAT_MODEL §11 lines 571-572/578 will be stale ("boot reindex") after
  Phase C — recommend the Phase C commit body flag it and Phase D PATTERNS or a
  one-line §11 touch capture "boot + hot reindex". Not a regression (strictly
  improved mitigation), so non-blocking for Phase C itself.
  (2) Concurrency wording precision for test #4 (above). Neither blocks.

## S4 Protocol And Wire Invariants
- Wire/security files checked:
  - `crates/nexus-coordinator-rs/src/public_feed.rs:1-160` — `FEED_FORMAT_VERSION
    = 1` (line 20), `FeedEntry` envelope (102-118), `FeedEntryCanonical`
    (126-133, JCS + `DOMAIN_FEED_V1`). Phase C touches NONE of these.
  - `crates/nexus-coordinator-rs/src/db.rs:212-222` — M15 `search_index` virtual
    table; documented "local persistence only", reconstructible from
    `public_feed`. Phase C adds NO migration (M17 is Phase D, plan §C.2 NOTE +
    plan §D.2). The only db.rs edit is a runtime PRAGMA (`busy_timeout`) — not a
    schema change, not a wire change.
- Field-level wire trace (honoring P2-PREFLIGHT-WIRE-CONTRACT-DEPTH — traced
  every field Phase C reads to its producer/consumer file:line before concluding
  "unchanged"):
  - `seq` (used as FTS5 rowid): PRODUCER `db.rs:867-882`
    `insert_feed_entry` returns `self.conn().last_insert_rowid() as u64`;
    backing column `public_feed.seq INTEGER PRIMARY KEY AUTOINCREMENT`
    (db.rs:157). Since the row is freshly inserted on the same connection,
    `last_insert_rowid()` == the assigned `seq` -> a monotone strictly
    increasing INTEGER, valid as an FTS5 `rowid`. CONSUMER (new) Phase C
    `upsert_feed_entry(db, seq, ...)`. CONFIRMED: `seq` is returned as a
    monotone INTEGER usable directly as rowid. (feed_sync.rs:260-261 destructures
    `Ok(seq)`.)
  - `op` / `payload`: the feed op JSON is serialized to `FeedEntryRow.payload`
    (String) at feed_sync.rs:241 (`serde_json::to_string(&feed_entry.op)`).
    `rebuild_from_feed` reads it back via `serde_json::from_str(&entry.payload)`
    (search.rs:102). So the shared `extract_index_fields(op: &Value)` helper
    must take the PARSED Value; the hot path will pass `&feed_entry.op`
    (already a `Value`, public_feed.rs:106) and the rebuild path will pass the
    re-parsed `payload`. SEMANTICALLY identical (round-trip of the same op
    JSON). No wire byte changes; only an in-process refactor.
  - Index field extraction (preserve current behavior): current
    `rebuild_from_feed` extracts `project_id` from `op.project_id`,
    `project_name` from `op.project_name`, `description` from
    `op.reason || op.comment` (search.rs:103-115). NOTE: no op payload in
    `public_feed.rs` (ReleasePublished:32-40, SourceBecameStale:43-47,
    CuratorVouched/Disendorsed:54-71) carries `project_name` or `category` ->
    today those columns are indexed EMPTY for feed entries. The helper MUST
    reproduce this exact extraction (project_id present; project_name/category
    empty; description from reason/comment). Phase C must NOT "fix" the empty
    fields — that is enrichment, which is Phase D (triplet) scope. Keeping the
    helper byte-equivalent is what test #3
    (`extract_index_fields_shared_with_rebuild`) asserts.
- VERSION/domain/canonical status: `FEED_FORMAT_VERSION = 1` UNCHANGED (no
  `op`/envelope structure change — pre-launch policy CLAUDE.md "bump only if the
  FeedEntry envelope structure changes"; it does not). No `DOMAIN_*` touched.
  No `canonical.rs` touched. No `*_ANNOUNCEMENT_VERSION` touched.
- `serde(default)` status: no new `serde(default)` introduced in Phase C
  (triplet `Option<String>` defaults are Phase D).
- Day 0 status: **preserved.** D1 implemented exactly as frozen (upsert by
  rowid=seq, shared helper, rebuild=repair, busy_timeout). No Day-0 contradicted.
- Finding: **clean.** Search index is local-only; zero wire impact confirmed by
  field-level trace.

## Plan Adaptation
Not applicable (verdict is EXECUTE, not PLAN-ADAPT).

## Risks And Scope Cuts
- Blocking risks: NONE.
- Non-blocking risks / carry-over:
  - Plan/kickoff bundled-SQLite string says 3.50.x; actual = 3.49.2. Cite 3.49.2
    in the Phase C commit body (S1b correction). Feasibility unaffected.
  - THREAT_MODEL §11 lines 571-572/578 ("boot reindex") become stale after the
    hot path lands; capture in commit body and/or a light §11 touch or Phase D
    PATTERNS note (the improved mitigation is not a regression).
  - Test #4 wording: assert search responsiveness/correctness under interleaved
    upsert, not WAL-level concurrency (single Mutex/connection reality). Do not
    add a second connection (premature, rejected by D1).
  - The shared helper must keep current empty `project_name`/`category` for feed
    ops (no payload provides them) — enrichment is Phase D, scope cut respected.
- Scope cuts still honored (kickoff §7 / plan §7):
  - #1 SearchManifest -> post-launch (D3): Phase C does no broadcast/wire work.
  - #9 Tantivy frozen: Phase C stays on FTS5.
  - M17 / triplet provenance enrichment deferred to Phase D (plan §C.2 NOTE,
    §D.1) — Phase C adds NO migration and NO new SearchResult field.
  - #11 per-client search rate-limit -> S74+ : not introduced (existing feed
    GCRA already gates the ingest/upsert path).

## Action
- **EXECUTE**: proceed with Phase C as planned (D1). Implement:
  `search::upsert_feed_entry(db, seq, ...)` (`INSERT OR REPLACE INTO
  search_index(rowid, project_id, project_name, category, description, op_type,
  source_type)` with rowid=seq, source_type='feed'); `extract_index_fields(op:
  &Value)` helper shared by `rebuild_from_feed`; call `upsert_feed_entry` in
  feed_sync.rs right after `Ok(seq)` inside the same `db` Mutex guard
  (feed_sync.rs:260-281, before the guard drops); add `conn.busy_timeout(
  Duration::from_secs(5))` in both `CoordinatorDb::open` and (for parity)
  `open_in_memory` paths in db.rs.
- Commit body MUST note (G8 traceability): D1 EXECUTE; bundled SQLite is 3.49.2
  (not 3.50.x); THREAT_MODEL §11 "boot reindex" becomes "boot + hot reindex";
  test #4 asserts responsiveness under interleave (single-connection Mutex, not
  WAL multi-conn concurrency); helper preserves current empty
  project_name/category for feed ops (enrichment is Phase D).
