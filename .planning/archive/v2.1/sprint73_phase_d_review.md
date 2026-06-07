# Sprint 73 Phase D — Deep Adversarial Review

## Verdict: PASS

Date: 2026-06-04
HEAD (uncommitted): `47c9ff7` + working tree (db.rs, search.rs, http.rs, PATTERNS.md, design note, preflight)
Reviewer: substitute for `nexus-phase-review-deep` (adversarial line-by-line)

Promoted PASS-PENDING → PASS after Codex reconciliation (see section below).
The diff is correct, fully covered, scope-honest, security-clean. No P0/P1.
Three P3 nits documented, none blocking; the commit body records the name
bridge + "M17 = local schema, no wire bump" as required.

---

## Codex reconciliation

Codex GPT-5.5 cross-model review run via `codex exec` (raw output, not
rewritten) → `.planning/active/sprint73_phase_d_codex_review.md`.

- **Verdict Codex : 8/8 livrables CONFIRME, 0 GAP, 0 PARTIEL.**
- Codex re-executed the suites independently:
  `cargo test -p nexus-coordinator-rs search::tests:: --locked` → 16 passed
  (incl. the 4 new Phase D tests); `cargo test -p nexus-shell-daemon
  search_handler_json_includes_triplet --locked` → 1 passed.
- Codex independently confirmed the load-bearing items: the
  `artifact_hash`→`archive_hash` name bridge (`search.rs:219`,
  `public_feed.rs:36`, test `:653`); M17 as the 17th `M::up` after M16
  (`rg "M::up"`); `FEED_FORMAT_VERSION` stays 1 (no `*_VERSION` diff);
  boot `rebuild_from_feed` reconstructibility (`runtime.rs:773-780`); the
  design note carries **no** wire code (only `public_feed.rs:78`
  forward-compat comment); PATTERNS §P56 present.
- **No GAP P0/P1/P2 → no fix loop required.** The review P3 nits stand as
  documented (non-blocking). Suites already green (main thread full
  fail-fast: 1566/1566, fmt/clippy/doctests/release all exit 0).

Sequence honored: review PASS-PENDING → Codex (raw) → reconcile → PASS → commit.

---

## Evidence read
- `git diff` of the 4 modified files (db.rs +32, search.rs +304, http.rs +73, PATTERNS.md +80) — full.
- `crates/nexus-coordinator-rs/src/public_feed.rs` (:26-40 `ReleasePublishedPayload`, :73-87 internally-tagged enum, :141-144 `op_type`, :265-320 `validate_known_operation`).
- `crates/nexus-coordinator-rs/src/feed_materializer.rs` (:43-58 existing `p.artifact_hash` consumer).
- `crates/nexus-shell-daemon-core/src/browse.rs` (:206-224 `BrowseEntry.archive_hash`/`repo_url`/`provenance_hash`/`is_open_source`).
- `crates/nexus-coordinator-rs/src/proof_card.rs` (:27,55 `ProofCardInput.archive_hash`).
- `crates/nexus-shell-daemon/src/feed_sync.rs` (:240-279 hot path — passes `&feed_entry.op`, stores `to_string(&op)`).
- `crates/nexus-shell-daemon/src/runtime.rs` (:773-782 unconditional boot `rebuild_from_feed`).
- `crates/nexus-coordinator-rs/src/db.rs` (`grep -c M::up` = 17; :17-21 schema_version row table; :1131,:1285 `schema_version()==1` asserts).
- Targeted test runs: 4 new + related (15 PASS) in coordinator-rs; `search_handler_json_includes_triplet` (1 PASS) in daemon.

---

## Axis 1 — Name bridge `artifact_hash` → `archive_hash` (the load-bearing finding)

**CORRECT.** `extract_index_fields` reads `opt_field("artifact_hash")` and
stores it into `IndexFields.archive_hash` (search.rs:208-215), with an explicit
8-line comment documenting the source→consumer rename. Verified end-to-end:

- **Source name is `artifact_hash`**: `ReleasePublishedPayload.artifact_hash`
  (public_feed.rs:36), validated hex-64 at insert (public_feed.rs:277). The
  existing materializer already reads `p.artifact_hash` (feed_materializer.rs:56),
  corroborating the source name.
- **Consumer name is `archive_hash`**: `BrowseEntry.archive_hash` (browse.rs:206),
  `ProofCardInput.archive_hash` (proof_card.rs:27,55), and the new
  `SearchResult.archive_hash` — all agree. The column choice mirrors the S74
  fork consumer, as designed.
- **Test asserts the EXACT mapping, not just non-null**: `search_result_carries_provenance_triplet`
  (search.rs:625-648) serialises a real `ReleasePublishedPayload` (which has NO
  `archive_hash` key, only `artifact_hash`) and asserts
  `str_col("archive_hash") == Some("a".repeat(64))`. If the extractor read the
  wrong key it would yield `None` and the test would fail. This is the strongest
  possible assertion of the bridge. The test comment explicitly calls this out.

**Wire-shape fidelity confirmed**: the real wire op is the **internally-tagged**
enum (`#[serde(tag = "op_type")]`, public_feed.rs:81), which serialises to a FLAT
object `{op_type, project_id, repo_url, commit_sha, artifact_hash, provenance_hash,
is_open_source}`. `extract_index_fields` reads each field at top level via
`op.get(key)` — correct for the flat shape. The hot path passes `&feed_entry.op`
(the raw wire `Value`) and the rebuild path parses `entry.payload` (=
`to_string(&op)`), so both production paths feed the same flat shape the test
exercises. (See P3-1 for the one fidelity nit.)

## Axis 2 — Semantic branch coverage

All required branches covered:
- **(a) triplet via a real ReleasePublished through the feed path** — `search_result_carries_provenance_triplet` uses `upsert_feed_entry` with a serialised `ReleasePublishedPayload`. ✓
- **(b) M17 repopulates without loss via rebuild** — `migration_m17_recreates_index_unindexed` inserts a durable feed row, `clear_all` + `rebuild_from_feed`, asserts the triplet survives. ✓ (limitation: P3-2)
- **(c) non-release op (CuratorVouched) → triplet None, no crash** — `search_result_null_triplet_for_non_release_op` asserts all four `Option`s are `None` and `is_open_source == false` while the row is still matchable via `reason`. ✓
- **(d) UNINDEXED columns not matchable** — `enriched_fields_unindexed_not_matchable` MATCHes a hash AND a repo_url, both return 0 hits, while the indexed name returns 1. Stronger than required (covers repo_url too). ✓
- **(e) JSON HTTP carries the 5 populated keys** — `search_handler_json_includes_triplet` asserts all 5 keys equal their expected values through the real router. ✓
- **(f) browse path carries the triplet** — `test_search_index_browse_entry` now asserts `repo_url` and `is_open_source` via the full `search()` path. ✓

**`is_open_source` true AND false both proven**: `true` via the browse test +
the carries-triplet test + the HTTP test; `false` via the null-triplet test +
the M17 test (`is_open_source: false`). Both round-trip directions exercised.
**`provenance_hash` absent (None)**: the M17 test uses `provenance_hash: None`,
proving the optional source field degrades to a stored NULL/None correctly.

No untested branch found.

## Axis 3 — `is_open_source` storage/read robustness

**ROBUST.** Write binds a Rust `bool` (rusqlite → integer 0/1). Read uses
`row.get::<_, Option<bool>>(10)?.unwrap_or(false)` (search.rs:141). The
`Option<bool>` round-trips the integer correctly (proven: HTTP test asserts
`true` through `search()`; M17/null tests assert `false`). The defensive
`Option` + `unwrap_or(false)` guards a NULL column, but **NULL is structurally
unreachable post-M17**: both writers (`index_entry`, `upsert_feed_entry`) always
bind a non-null bool, and M17 DROP/recreate guarantees no pre-M17 NULL rows
survive. The defence is harmless and consistent with the pre-launch "runtime
tolerance" rationale documented inline. No gap.

## Axis 4 — Migration M17 correctness

**CORRECT.**
- **Numbering**: `grep -c M::up` = 17 total (the first is the `schema_version`
  table-creation migration at db.rs:17). M17 is appended **after** M16
  (`ALTER TABLE tasks ADD COLUMN result_text`, db.rs:228), so it is the latest
  `M::up`. `rusqlite_migration` advances `user_version` monotonically. ✓
- **`schema_version` row table untouched**: the hard-coded `VALUES (1)`
  (db.rs:21) is independent of the migration count; both `schema_version()==1`
  asserts (db.rs:1131, :1285) still pass (confirmed green in full run). ✓
- **DROP/recreate is sound**: FTS5 virtual tables reject `ALTER … ADD COLUMN`,
  so DROP+CREATE is the canonical path. The dropped data is **integrally
  reconstructible** from the durable signed `public_feed`. ✓
- **Boot rebuild repopulates after M17 on a real on-disk DB**: `open()` applies
  migrations, then `runtime.rs:778` calls `rebuild_from_feed` **unconditionally**
  at every boot (verified runtime.rs:773-782). So a real version-16→17 upgrade
  on disk re-fills the index with the triplet — the "no data loss" claim holds
  at the system level. ✓
- **Column list matches**: the M17 CREATE lists 11 columns; `index_entry`,
  `upsert_feed_entry`, and `search()` SELECT all reference the same 11 in the
  same order. UNINDEXED is applied to exactly the 5 non-text columns
  (`project_id`, `op_type`, `source_type`, + the 5 new = `repo_url`,
  `commit_sha`, `archive_hash`, `provenance_hash`, `is_open_source`). The 3
  matchable columns (`project_name`, `category`, `description`) stay indexed,
  identical to M15. ✓ (Note: M17 also re-fixes `tokenize='unicode61'`, matching M15.)

## Axis 5 — Scope cuts respected (kickoff §7)

**ALL HONORED.**
- No SearchManifest wire code: D3 is a 250-line **design note**
  (`.planning/research/s73_searchmanifest_index_node_design.md`), zero
  `SearchManifestPublished` op, no dead type. Pure doc — not scaffolding. ✓
- No fork/search/open commands (S74): the diff only **enriches** `SearchResult`
  + JSON; `index_entry` has **zero production callers** (verified — only
  test-only in search.rs/http.rs), so no fork path is wired. ✓
- FTS5 stays the engine; Tantivy not reopened. ✓
- No wire bump: `FEED_FORMAT_VERSION = 1` unchanged; M17 is local SQLite schema.
  The diff comments (db.rs M17, http.rs, PATTERNS §P56, design note §7) all
  state this explicitly. ✓

## Axis 6 — Security (THREAT_MODEL §11)

**NO REGRESSION.**
- The triplet is sourced from a feed op **pre-validated at insert**
  (public_feed.rs:271-287: `repo_url` https, `commit_sha` hex-40,
  `artifact_hash` hex-64, `provenance_hash` hex-64-or-None,
  `is_open_source=true⇒provenance_hash`). A malformed value cannot enter the feed.
- Returned **verbatim** as UNINDEXED (never interpreted, never matchable) via
  `serde_json::json!` in `search_handler` (proper escaping) → no JSON injection.
- M17 is a **one-shot** schema op inside `open()`, off the hot ingest path → no
  DoS amplification. The hot path stays the O(1) incremental upsert (Phase C),
  bounded by the existing GCRA + entry_hash dedup.
- No new T-* surface; the triplet mirrors what `BrowseEntry` already returns.
  THREAT_MODEL §11 was touched in Phase C and needs no further edit for D
  (preflight S3 confirms). ✓

## Axis 7 — Rowid partition tripwire (carried to S74)

**PRESERVED, NOT AGGRAVATED.** The tripwire doc-comment lives on
`upsert_feed_entry` (search.rs:241-244: "feed rows own `[1, max feed seq]`;
browse-sourced rows … must partition the rowid space"). M17 DROP/recreate
rebuilds feed rows keyed `rowid=seq` via `rebuild_from_feed`, identical
discipline — it does not change the rowid space. PATTERNS §P56 restates the
S74 carry. (Preflight referenced the pre-Phase-C location :148-152; the comment
moved with the function but is intact.) ✓

## Axis 8 — Deliverables + patterns

- **Design note** (`s73_searchmanifest_index_node_design.md`, 250 lines):
  substantial and grounded — §3 tabulates all 7 OSS models (F-Droid, IPFS DHT,
  Nostr NIP-50, Radicle, SSB, pkarr/iroh, ARES 2024), §4 gives a concrete
  Ed25519-signed opt-in index-node design (default OFF, queries never sent to
  network), §5 a trigger criterion, §7 the (resolved) DESIGN-CONFLICT note. Not
  speculative scaffolding. ✓
- **PATTERNS §P56**: documents D1 (hot reindex) + D2 (UNINDEXED triplet + M17 +
  name bridge + DTO tolerance + rowid tripwire). Numbering sequential after §P55,
  cross-refs correct. ✓
- **Commit body (NOT yet written)** — must record (i) the `artifact_hash`→
  `archive_hash` name bridge, (ii) "M17 = local schema, FEED_FORMAT_VERSION=1,
  no wire bump", (iii) test delta +5 (4 coordinator + 1 daemon). This is the
  PASS-PENDING gate. See P3-3.

## Axis 9 — Code quality

- **`Provenance<'a>` (borrowed) vs `IndexFields` (owned)** — justified, not
  redundant. `Provenance` is the **call-site** arg for `index_entry` (callers
  hold `&str` slices, e.g. tests pass `Some("https://…")` / `Some(&archive)`),
  so borrowing avoids forcing every caller to allocate. `IndexFields` is the
  **extractor output** owning Strings pulled from a transient `serde_json::Value`
  that is dropped after extraction — it must own. Two lifetimes, two legitimate
  ownership models. Acceptable.
- **`#[allow(clippy::too_many_arguments)]` on `index_entry` (8 args)** —
  borderline but defensible: 6 were pre-existing, the 8th is the `&Provenance`
  struct that already *collapses* the 5 provenance fields into one arg (the
  right refactor was applied). Bundling the remaining 6 primitives into another
  struct would be churn for a test-only function. Acceptable; the `#[allow]` is
  honest.
- **`str_col` closure with `format!("SELECT {name} …")`** — `name` is a
  hard-coded string literal at every call site (`"repo_url"`, `"archive_hash"`,
  etc.), so there is no injection vector (test-only, no user input). Readable.
  Acceptable.

---

## Findings

### P0 — none
### P1 — none

### P2 — none blocking
(The Axis-9 `#[allow(too_many_arguments)]` and the borrowed/owned dual
representation were considered for P2 and dismissed as justified.)

### P3 (nits — non-blocking, may be addressed or noted in commit body)

- **P3-1 — Test op shape omits the `op_type` tag.** `search_result_carries_provenance_triplet`
  serialises the bare `ReleasePublishedPayload` (`to_value(&payload)`), which
  lacks the `"op_type": "ReleasePublished"` tag the real internally-tagged enum
  emits. `extract_index_fields` ignores `op_type` (it reads payload fields
  directly), so the bridge assertion is valid — but the test is marginally less
  faithful than `to_value(PublicFeedOperation::ReleasePublished(payload))` would
  be. Harmless (the daemon `op_type` arg is passed separately as `"ReleasePublished"`).
  Optional: serialise the full enum for exact wire fidelity.

- **P3-2 — M17 test simulates, does not seed a pre-M17 on-disk row.** In-memory
  `open_in_memory` applies all 17 migrations atomically, so there is never a
  real "row inserted at user_version=16, survives DROP at M17" case. The test
  proves (new columns exist) + (rebuild repopulates the triplet) + (UNINDEXED),
  and the system-level "no data loss" holds because boot `rebuild_from_feed`
  always re-fills from the durable feed (runtime.rs:778). The unit test's
  "no data loss" comment is therefore a *system* guarantee, not a *migration-DROP*
  guarantee. Acceptable given reconstructibility; note in the body if precision
  is wanted. Not worth a file-DB fixture.

- **P3-3 — Commit body not yet written (the PASS-PENDING gate).** Must document
  the name bridge, the local-schema/no-wire-bump rationale, and the +5 test
  delta. Mechanical; verification block already green per main thread
  (1566/1566, +5 vs Phase C 1561 Win; fmt/clippy/doc/release all exit 0).

---

## Conclusion

The diff is **correct, fully covered, scope-honest, and security-clean**. The
load-bearing `artifact_hash`→`archive_hash` name bridge is implemented and
asserted with the strongest possible test (a real payload that lacks the
consumer key). Migration M17 is correctly numbered, sound (reconstructible),
and the boot rebuild guarantees on-disk repopulation. UNINDEXED semantics are
proven by negative MATCH assertions. D3 is a substantial design note with zero
wire code — scope cut honored. No P0/P1/P2.

**PASS-PENDING → PASS** once the commit body records the name bridge + the
"M17 = local schema, FEED_FORMAT_VERSION=1, no wire bump" rationale + the +5
test delta. Proceed to Codex gate, then commit.
