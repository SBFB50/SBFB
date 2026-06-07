# Sprint 73 Phase C Review (adversarial, pre-Codex)

Date: 2026-06-04
Reviewer role: `nexus-phase-review-deep` (fallback instance)
HEAD baseline: `a4e1542`
Scope reviewed: working-tree diff (4 files, +262/-26).

Method: line-by-line adversarial read of `git --no-pager diff HEAD`,
cross-checked against producers/consumers (`db.rs`, `feed_sync.rs`,
`runtime.rs`, `http.rs`), the FTS5 schema (M15), and the Phase C
preflight (`sprint73_phase_c_preflight.md`, verdict EXECUTE).
Independent targeted re-run of the 5 Phase C tests = 5/5 PASS (output
inline at point 8).

---

## Point 1 — Idempotence reelle

**Verdict: PASS.**

Evidence — `crates/nexus-coordinator-rs/src/search.rs:153-172`:
```rust
pub fn upsert_feed_entry(db, seq, op, op_type) -> ... {
    let fields = extract_index_fields(op);
    db.conn().execute(
        "INSERT OR REPLACE INTO search_index
            (rowid, project_id, project_name, category, description, op_type, source_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'feed')",
        params![seq as i64, ...],
    )?;
```
`search_index` is a **standalone** FTS5 table (db.rs:212-222, M15, no
`content=`). For a standalone FTS5 table `INSERT OR REPLACE INTO ft(rowid,
...)` is the SQLite-canonical idempotent upsert: a second insert at the same
`rowid` deletes-then-reinserts the row in place, so re-arrival of the same
`seq` rewrites byte-identical content with no duplicate. This is the documented
pattern (preflight S1a, sqlite.org/fts5.html), not the trigger/external-content
hazard. Idempotence is real.

The test proves it — `search.rs:391-403` `reindex_hot_is_idempotent`:
upserts seq `42` twice, then asserts `total == 1` AND `results.len() == 1`.
Both the FTS5 `COUNT(*)` (via `search()` at search.rs:62-65) and the returned
row vector are checked, so a phantom duplicate row would fail. Honest test.

Note: the rowid==seq identity is load-bearing and is verified — `public_feed.seq`
is `INTEGER PRIMARY KEY AUTOINCREMENT` (db.rs:157), and `insert_feed_entry`
returns `last_insert_rowid()` of that same connection (db.rs:889). On a freshly
inserted row, `last_insert_rowid()` == the assigned `seq`. The mapping
`insert -> seq -> upsert(rowid=seq)` is sound and monotone.

## Point 2 — Collision rowid browse/feed (THE key question)

**Verdict: CONCERN (documented latent issue, NOT a prod FAIL).**

Evidence — the shared rowid space is real:
- feed rows: explicit `rowid = seq`, range `[1, max seq]` (search.rs:161, 168).
- browse rows: `index_entry` does a plain `INSERT INTO search_index (... no rowid)`
  -> SQLite auto-assigns rowid, also starting at 1 (search.rs:43-47).
If both paths ran in the same prod table, a feed upsert at `rowid=N` could clobber
a browse row that landed at auto-rowid `N`. That is a genuine latent bug **iff
browse indexing is ever wired in production without partitioning the rowid space.**

Why it is NOT a prod bug today (grep evidence): the ONLY caller of `index_entry`
anywhere under `crates/` is `crates/nexus-shell-daemon/src/http.rs:6410`, inside
`#[tokio::test] async fn test_search_endpoint_http()` — test-only. No production
code path inserts a 'browse' row. Confirmed:
```
git grep -n "index_entry" crates/
  search.rs: definition + 4 in-module #[cfg(test)] callers (213/267/277/298/321)
  http.rs:6410  -> inside #[tokio::test]
```
So in production `search_index` holds only 'feed' rows, all keyed by `seq`; there
is no browse row to collide with. No P0/P1.

Traceability for S74 is present and explicit — `search.rs:148-152` (doc on
`upsert_feed_entry`):
> "Browse-sourced rows (`index_entry`, auto rowid) are currently test-only;
> wiring browse indexing in production (S74) must partition the rowid space so a
> feed upsert cannot clobber a browse row."
This is the correct disposition: the hazard is named, attributed to the future
sprint that would introduce it, and the mitigation (partition the rowid space)
is stated. Judgement: **CONCERN to record, not a fix Phase C owes.** Phase C does
not introduce a production browse writer, so it cannot regress anything. The
collision can only become live if S74 wires `index_entry` in prod and ignores
this comment — which the comment exists to prevent.

Recommendation (non-blocking): when S74 wires browse indexing, either (a) offset
browse rowids into a disjoint high range, or (b) give browse rows their own table,
or (c) make `index_entry` use a deterministic non-colliding rowid. The current
comment is sufficient as a tripwire.

## Point 3 — Scope de lock feed_sync (no re-lock / no race / no deadlock / no await)

**Verdict: PASS.**

Evidence — `crates/nexus-shell-daemon/src/feed_sync.rs:232-298`:
- `db` is the `MutexGuard<CoordinatorDb>` acquired at line 232 (`coordinator_db.lock()`).
- `insert_feed_entry(&row)` at 260 and `upsert_feed_entry(&db, seq, ...)` at 268-273
  both execute under that SAME guard. There is no intervening `.lock()`, no `drop(db)`,
  no second acquisition between insert and upsert. The guard drops naturally at the
  end of the `match` (line 298).
- Coercion: `db: MutexGuard<CoordinatorDb>`; `&db` is `&MutexGuard<..>` which derefs
  to `&CoordinatorDb`, matching `upsert_feed_entry(db: &CoordinatorDb, ...)`
  (search.rs:153). Compiles (fail-fast clippy clean) — coercion confirmed.
- No `.await` between lock acquisition (232) and guard drop (298): the `Ok(seq)` arm
  contains only `insert_feed_entry`, `upsert_feed_entry`, and the synchronous
  `warn!`/`info!` macros. No async hold of a `std::sync::Mutex` across `.await`
  (which would be a Send/blocking hazard). Verified by reading 260-298.
- Single connection / single Mutex (db.rs:248-249 `conn: Connection`; production
  wrap `Arc<Mutex<CoordinatorDb>>` at runtime.rs:524): the upsert reuses the one
  connection, so no second-writer SQLITE_BUSY against itself, no deadlock.
- Correct gate ordering preserved: the guard at 232 is RE-acquired *after* the
  rate-limit check (the earlier guard was dropped at line 221 specifically so the
  GCRA token is consumed only for genuinely new entries — feed_sync.rs:219-230).
  So the hot upsert runs strictly AFTER dedup (204-217) AND rate-limit (223-230).

## Point 4 — Anti-derive helper (single source of truth)

**Verdict: PASS.**

Evidence — both paths call the SAME `extract_index_fields`:
- hot path: `upsert_feed_entry` -> `let fields = extract_index_fields(op);`
  (search.rs:154).
- repair path: `rebuild_from_feed` no longer extracts inline; it now calls
  `upsert_feed_entry(db, entry.seq, &op, &entry.op_type)` (search.rs:191), which
  itself calls `extract_index_fields`. So rebuild reaches the extractor THROUGH
  the hot-path function — there is structurally only one extractor and one writer.
  The old 30-line inline extraction block in rebuild (previous search.rs:102-122)
  is deleted in the diff. No divergent copy remains.

Test honesty — `search.rs:405-435` `extract_index_fields_shared_with_rebuild`:
runs Path A (`rebuild_from_feed`) then Path B (`clear_all` + `upsert_feed_entry`)
on the SAME op, and asserts equality of all six projected columns:
`project_id`, `project_name`, `category`, `description`, `op_type`, `source_type`
(search.rs:430-435). That is the full `SearchResult` surface minus the bm25 score
(which is query-derived, not stored). Comparing the read-back results of the two
write paths is a faithful anti-drift assertion. PASS.

## Point 5 — Fidelite comportement (historical extraction + rebuild rowid change)

**Verdict: PASS (neutral/improved, no regression).**

Field extraction fidelity — `extract_index_fields` (search.rs:108-130):
- `project_id` <- `op.project_id` (string or ""). Matches old rebuild.
- `project_name` <- `op.project_name` (string or "" — no feed op carries it today,
  so "" in practice). Matches old rebuild.
- `category` <- hard `String::new()`. The OLD rebuild also passed literal `""` for
  category (old call `index_entry(db, .., "", description, ..)`). Matches, and the
  inline comment (search.rs:121-122) documents the rationale. PASS.
- `description` <- `op.reason || op.comment` (search.rs:124-129). Byte-identical to
  old rebuild precedence (`reason` then fall back `comment`). PASS.

Rebuild rowid change (old auto rowid -> new rowid=seq) — NOT a regression:
- Old rebuild used `index_entry` (auto rowid 1,2,3…). New rebuild uses
  `upsert_feed_entry(rowid=seq)`. `rebuild_from_feed` first
  `DELETE FROM search_index WHERE source_type='feed'` (search.rs:185), then
  repopulates. In production the table holds only feed rows (point 2), so the post-
  rebuild rowids now equal the feed `seq` values — i.e. rebuild output is now
  rowid-IDENTICAL to the hot path. This is strictly an IMPROVEMENT (rebuild and hot
  path are now byte-for-byte identical, which is exactly what makes the anti-drift
  test possible). Search semantics (bm25 ranking, MATCH) are rowid-independent, so
  user-visible behavior is unchanged. PASS.
- One subtle property: because rebuild now keys by `seq`, two rebuilds are
  themselves idempotent at the rowid level (DELETE + INSERT OR REPLACE by seq),
  removing the old behavior where successive rebuilds reshuffled auto rowids. Net
  positive.

## Point 6 — Scope discipline (no Phase D/E bleed)

**Verdict: PASS.**

Searched the full diff for out-of-scope markers:
- No `M17`, no `ALTER TABLE`, no migration added (only db.rs PRAGMA `busy_timeout`,
  which is a runtime connection setting, not schema). The sole new migration in
  the tree is still M16 (db.rs:228), pre-existing.
- No provenance triplet fields (`repo_url` / `commit_sha` / `archive_hash` /
  `provenance_hash`) anywhere in the diff.
- No new `SearchResult` field (struct untouched; the 6-column projection is
  unchanged).
- No `SearchManifest` wire/broadcast code.
- No shell search bar / frontend (`web/`) change.
- No Tantivy.
The only doc edit is THREAT_MODEL §11 (point 9). Scope cuts #1 (SearchManifest
defer), #9 (Tantivy frozen), M17/triplet (Phase D), shell bar (Phase E) all
respected.

## Point 7 — Securite / DoS

**Verdict: PASS.**

- O(1) not O(N): `upsert_feed_entry` issues exactly ONE `INSERT OR REPLACE` of one
  row (search.rs:160-170). It does NOT call `rebuild_from_feed` (which is O(N) over
  the whole feed). The per-ingest cost is a single short FTS5 write. This is the
  explicit anti-amplification design (preflight S3 T-SEARCH-DOS).
- Runs after the gates: dedup (feed_sync.rs:204-217) + GCRA rate-limit
  (5 ops/min/author, feed_sync.rs:223-230) both precede the upsert; the upsert only
  fires on `Ok(seq)` of a genuinely new insert (feed_sync.rs:260). So the hot
  reindex inherits the existing T-FEED-SPAM ceiling; it cannot be driven faster than
  the limiter allows. No new amplification surface.
- Best-effort failure handling: on upsert error the entry is already durably stored;
  the code `warn!`s and continues (feed_sync.rs:268-279). A failed reindex degrades
  gracefully to "searchable after next boot rebuild" — it never fails the ingest nor
  loses the durable feed entry. Correct trade-off (durability > index freshness).

## Point 8 — Tests assertions utiles

**Verdict: PASS.**

Independent re-run (already compiled):
```
cargo nextest run -p nexus-coordinator-rs --locked \
  -E 'test(reindex)+test(feed_ingest)+test(rebuild)+test(extract_index)+test(hot_reindex)'
  PASS feed_ingest_indexes_entry_hot
  PASS reindex_hot_is_idempotent
  PASS extract_index_fields_shared_with_rebuild
  PASS hot_reindex_keeps_search_results_consistent
  PASS rebuild_from_feed_still_repairs
  Summary: 7 tests run (5 Phase C + 2 incidental rebuild), 7 passed, 253 skipped
```
Per-test assertion audit:
- `feed_ingest_indexes_entry_hot` (367-388): inserts a real feed row, hot-upserts,
  then `search("quantum")` asserts `total==1`, `len==1`, `source_type=="feed"`.
  Proves the END-TO-END "ingest -> searchable now" claim (the whole point of D1).
  Non-trivial.
- `reindex_hot_is_idempotent` (391-403): see point 1. Real (1-row) assertion.
- `extract_index_fields_shared_with_rebuild` (405-435): see point 4. Real 6-field
  equality across both write paths.
- `hot_reindex_keeps_search_results_consistent` (438-460): loops seq 1..=5, asserts
  `total==seq` after each upsert (monotone, no torn/dup rows under interleave). See
  point 8-honesty below. Real assertion.
- `rebuild_from_feed_still_repairs` (463-483): inserts, `clear_all` (simulate empty
  index), asserts `before==0`, runs `rebuild_from_feed`, asserts return `n==1` AND
  `after==1` AND `source_type=="feed"`. Proves the repair path survives the refactor.
None are empty/trivial.

Test #4 honesty — the test name was changed from the plan's
`hot_reindex_does_not_block_search_reader` to
`hot_reindex_keeps_search_results_consistent`, and the doc comment (search.rs:441-446)
states plainly that the DB is "a single Connection behind one Mutex, so an upsert and
a search serialize at the Rust lock (not via WAL reader isolation)" and that it asserts
"correctness under interleave … no torn or duplicated rows." This is HONEST: it does
NOT over-claim WAL non-blocking concurrency (which is impossible with one connection,
per preflight S3). The rename + comment correctly downgrade the claim to what the
single-connection architecture can actually guarantee. Good adversarial-proof wording.

## Point 9 — Doc accuracy (THREAT_MODEL §11)

**Verdict: PASS.**

Evidence — `docs/security/THREAT_MODEL.md` diff (T-CURATOR-VOUCH §11):
- Prose changed from "Le search index re-indexe au boot — les entries spam
  pre-rate-limit sont visibles…" to "Le search index est reindexe a chaud a
  l'ingest (Sprint 73 Phase C, apres les gates dedup + rate-limit) et reste
  reconstructible au boot — les entries spam admises restent visibles mais
  attribuables et bornees par le rate limiter."
- Mitigation row: `boot reindex` -> `hot/boot reindex`.
Both edits are factually accurate and match the code: hot reindex happens at ingest
AFTER dedup+rate-limit (point 7), and the boot rebuild remains as repair
(runtime.rs:778). It is not misleading and is strictly a stronger mitigation
statement (immediate + still bounded), not a weakening. Severity/Likelihood/Residual
rows are unchanged, which is correct — freshness does not change the spam attack's
severity. This is the §11 touch the preflight S3 recommended; the residual concern
about §10/§11 staleness raised in preflight is now closed for §11 by this edit.

---

## Branch coverage semantique

Covered by tests:
- Hot path success -> immediately searchable (test #1).
- Re-upsert same seq -> no duplicate (test #2, idempotence).
- Hot vs rebuild field parity (test #3, anti-drift).
- Interleaved upsert+search -> consistent monotone index (test #4).
- Repair path repopulates a cleared index (test #5).
- `description` from `reason` (#1/#2/#4/#5) and from the `project_name`-bearing op
  (#3); `category` always empty (#3 asserts equality incl. empty category).

NOT directly covered by unit tests (acceptable — covered by reasoning / fail-fast):
- **feed_sync hot-path wiring** (`feed_sync.rs:268-279`): the actual call site inside
  the Mutex guard is not exercised by an integration test; tests call
  `upsert_feed_entry` directly. The call site is a 3-line best-effort branch, type-
  checked by the compiler (clippy clean) and structurally trivial. Low risk; an
  end-to-end feed-ingest->search integration test would be the natural S74 follow-up,
  but is not required for Phase C correctness. (Recorded, non-blocking.)
- **upsert error -> warn-and-continue branch** (feed_sync.rs:274-279): not unit-
  tested (hard to force a write error on an in-memory DB without contrivance).
  Logic is a simple `if let Err` log; low risk.
- **busy_timeout(5s) effect under real contention**: not asserted (would need a
  multi-thread BUSY scenario). The PRAGMA is applied on both `open` and
  `open_in_memory` for parity (db.rs:264-265, 281-282); its correctness is the
  driver's. Non-blocking.
- **Browse/feed rowid collision**: deliberately NOT exercised because the prod
  browse writer does not exist (point 2). A collision test would require contriving
  the very prod path the comment forbids until S74.

## Scope cuts respectes

- #1 SearchManifest broadcast/wire -> defer (D3): no broadcast/wire code in diff. OK.
- #9 Tantivy frozen: stays FTS5. OK.
- M17 migration + provenance triplet (`repo_url`/`commit_sha`/`archive_hash`/
  `provenance_hash`) + `SearchResult` enrichment -> Phase D: NONE present. OK.
- Shell search bar / `web/` -> Phase E: no frontend change. OK.
- Per-client search rate-limit -> S74+: not introduced (existing feed GCRA gates the
  ingest/upsert path). OK.
- No wire-format bump: `FEED_FORMAT_VERSION` untouched; only a local FTS5 write path
  + a runtime PRAGMA. Consistent with pre-launch policy. OK.

## Findings summary

- No P0, no P1. Implementation matches frozen D1 exactly.
- The only material finding is the **browse/feed rowid collision**, which is a
  documented LATENT issue, not a production bug: the sole `index_entry` caller is a
  `#[tokio::test]` (`http.rs:6410`); no prod path writes a 'browse' row, so the
  shared rowid space cannot collide today. The `upsert_feed_entry` doc comment
  (search.rs:148-152) explicitly hands S74 the obligation to partition the rowid
  space before wiring browse indexing in prod. Disposition: CONCERN (tripwire is
  present and adequate).
- rebuild refactor is neutral-to-improved: rebuild now keys feed rows by `seq`,
  making rebuild output rowid-identical to the hot path (enables the anti-drift test).
- Test #4 was honestly renamed/reworded to assert interleave consistency, not WAL
  concurrency — correct for the single-connection/single-Mutex reality.
- THREAT_MODEL §11 edit is accurate and a strictly stronger mitigation statement.
- Non-blocking carry: no integration test exercises the feed_sync call site or the
  busy_timeout-under-contention path; both are low-risk and natural S74 follow-ups.

## Verdict: PASS

## Codex reconciliation

Codex GPT-5.5 cross-review (`sprint73_phase_c_codex_review.md`, output brut non
réécrit) : **7/7 livrables CONFIRME, 0 GAP, 0 PARTIEL**. Codex a en plus
ré-exécuté `cargo test -p nexus-coordinator-rs --lib search::tests::` → 12 tests
passés (dont les 5 de Phase C), et a confirmé indépendamment que les payloads
feed (`public_feed.rs:32-70`) ne portent pas `project_name`/`category` (donc
extraction vide conservée), que `rebuild_from_feed` réutilise bien
`upsert_feed_entry` (même extracteur + `rowid=seq`), que l'appel hot est dans le
même `MutexGuard` que l'insert, et qu'aucune migration M17 / champ triplet n'a
fui dans cette phase (scope Phase D préservé).

Aucun GAP P0/P1/P2 → pas de boucle de correction, pas de re-run de suites requis.
La review Claude (PASS-PENDING) est promue **PASS**. Le seul finding (collision
rowid browse/feed) reste un CONCERN latent documenté (tripwire dans le doc
comment `search.rs:148-152`, obligation S74), confirmé non-bloquant par les deux
revues (review-deep + Codex).
