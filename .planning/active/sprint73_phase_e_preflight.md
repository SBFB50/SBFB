# Sprint 73 Phase E Preflight

Date: 2026-06-04
HEAD: `0f86e5a`
Verdict: **SCOPE-CUT-CONSISTENT**

## Evidence Rules
- Claim policy: every claim below cites a path, command output, URL/date, or
  explicit assumption.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure source of truth)
  - `.planning/active/sprint73_plan.md` (Phase E §289-325)
  - `.planning/active/sprint73_kickoff.md` (D4 §363-400)
  - `crates/nexus-coordinator-rs/src/search.rs` (SearchResult producer struct)
  - `crates/nexus-shell-daemon/src/http.rs:1957-2040` (search_handler JSON
    producer) + `:6400-6510` (HTTP tests of the exact shape)
  - `web/src/api/daemon.ts` (callDaemon + listBrowse consumer pattern)
  - `web/src/pages/Browse.tsx` (consumer + existing repo_url anchor render)
  - `web/src/api/auth.ts` (authFetch bearer)
  - `web/src/api/__tests__/daemon.test.ts` (Vitest harness pattern)
  - `web/scripts/scan-en-strings.sh` (FR-only gate)
  - `web/src/bridge/useBridge.ts:359-369` (bridge SDK search method)
  - memory `feedback_approach.md`, `feedback_context7_systematic.md`
- Commands run:
  - `git rev-parse --short HEAD` -> `0f86e5a`
  - `git log --oneline -10` -> Phase A `6f5ff30`, B `a4e1542`, C `47c9ff7`,
    D `0f86e5a` all committed; E next
  - dep resolution: `@tanstack/react-query` 5.100.9, `zod` 3.25.76 (lockfile)
  - `git log --all -- web/src/pages/Browse.tsx` (S2, no search-bar rejection)

## Scope
- Plan source: `.planning/active/sprint73_plan.md` §Phase E (lines 289-325);
  kickoff D4 (lines 363-400).
- Target files:
  - `web/src/api/daemon.ts` (~297-311): NEW `searchBrowse(baseUrl, q, limit,
    offset)` + `SearchResponseSchema` Zod.
  - `web/src/pages/Browse.tsx` (~39-108): dedicated search field, React Query
    `['daemon-search', coordUrl, q]`, hit render with provenance.
  - `web/src/i18n/*`: plan names this path; **does not exist** (see S2 F2).
    The shell uses inline FR string literals, not an i18n layer.
- Deps/APIs/specs: NO new dependency. Reuses `@tanstack/react-query` 5.100.9,
  `zod` 3.25.76, `authFetch`, `DaemonResult<T>`, `callDaemon` (all present).
- Security/protocol surfaces: front consuming the loopback endpoint
  `GET /api/daemon/search` (T0 loopback trust). No wire-format change
  (search_index is local; `FEED_FORMAT_VERSION` stays 1).
- Tests expected (Vitest `web/`, 4): `searchBrowse_calls_daemon_search_endpoint`,
  `browse_search_renders_enriched_results`, `browse_search_empty_state_french`,
  `search_response_schema_parses_triplet`.

## S1a OSS Prior Art
- Domain: React Query v5 search/typeahead field bound to a REST endpoint.
- Sources (dates):
  - TanStack Query v5 migration guide (context7, GitHub main docs, 2025):
    `keepPreviousData` removed; idiom is `placeholderData: keepPreviousData`.
  - TanStack Query disabling-queries guide (context7, GitHub main docs):
    `enabled: !!filter` defers the fetch until input is non-empty.
  - TanStack discussion #6460 (keepPreviousData deprecated), supastarter
    keepPreviousData guide (2025-08-24).
- Finding: **APPROACH-ALIGNED** with one idiom refinement. The plan's React
  Query usage (debounced/term-in-key) matches mature OSS practice. The v5
  idiom the implementation should use:
  - debounced `q` in the query key `['daemon-search', coordUrl, q]`;
  - `enabled: q.trim().length > 0` so an empty field never fires a request
    (also avoids hitting `search()` which returns `([], 0)` on empty input —
    `search.rs:36-50` sanitize returns `None` -> empty);
  - `placeholderData: keepPreviousData` (import from `@tanstack/react-query`)
    so the result list does not flicker between keystrokes.
- Impact: none blocking. Use the v5 `placeholderData`/`enabled` idiom rather
  than v4 `keepPreviousData`. Non-blocking refinement; documented here.

## S1b Dependencies, CVEs, Release Notes
- Scanned: `@tanstack/react-query` (pinned `^5.96.2`, lock-resolved 5.100.9),
  `zod` (pinned `^3.25.76`, lock-resolved 3.25.76), `react` 19.2.4.
- Commands/sources:
  - lockfile: `node_modules/@tanstack/react-query` -> 5.100.9;
    `node_modules/zod` -> 3.25.76.
  - WebSearch zod advisories (2026-06): no CVE for zod 3.x; Snyk shows no
    direct vuln for the pinned line.
  - WebSearch TanStack supply chain (2026-05-11): 84 malicious versions across
    42 `@tanstack/*` packages published 19:20-19:26 UTC and yanked; TanStack
    blog + Snyk + Wiz confirm `@tanstack/react-query` core family CLEAN. The
    lock-resolved 5.100.9 is NOT in the compromised point-release set.
- Finding: **clean** (non-blocking). No new dependency is added by Phase E.
  Informational: if `web/` `npm install` was run against any `@tanstack/*`
  package on 2026-05-11 in CI, rotate exposed credentials (project is
  pre-launch, nothing pushed -> low exposure). No action required for the
  code of this phase.

## S2 Historical Decisions
- Commands:
  - `git log --all --oneline -- web/src/pages/Browse.tsx` -> 8 commits, none
    rejecting a shell search field; last touch S65 badges, S53 browse pull.
  - `grep -rln "barre recherche|search bar|champ recherche" .planning` -> only
    S73 artifacts (kickoff/plan/reviews) and S6 audit (unrelated curators).
- Decisions crossed:
  - **D4 (kickoff §363-400)** explicitly rejects the alternatives (global
    header bar; Command Palette full-text; bridge-SDK-only) and selects the
    dedicated Browse field via `searchBrowse()`. No reversion needed — this is
    the active, frozen Day 0 decision for this phase.
  - **Bridge SDK `search`** (`useBridge.ts:359-369`) is the iframe-app-facing
    path; no shell component consumes it (kickoff §371). Phase E adds a
    parallel shell-facing helper — not a conflict, consistent with D4's
    "reutiliser le bridge SDK seul -> Rejete".
- Findings (non-blocking):
  - **F1 (plan-shape drift, non-blocking).** Plan §293/§301 says
    `SearchResponseSchema` "mirrors SearchResult enriched (7 base + 5
    provenance)". The actual endpoint returns a **wrapping envelope**
    `{ results: [...], total: u64, took_ms: u64 }` (http.rs:2031-2039), NOT a
    bare `SearchResult[]`, and each hit has **12 keys** (project_id,
    project_name, category, description, op_type, source_type, score +
    repo_url, commit_sha, archive_hash, provenance_hash, is_open_source). The
    Zod schema must model the envelope + the per-hit object. See S4.
  - **F2 (i18n path does not exist, non-blocking).** Plan §303 + D4 §399 name
    `web/src/i18n/*`. There is no i18n directory or framework in `web/src`
    (verified: `find web/src -iname "*i18n*"` empty). The shell uses inline
    French literals (e.g. `Browse.tsx:24,63,305`). `scan-en-strings.sh` only
    blocks a narrow EN blocklist (Welcome/Dashboard/Loading.../etc.), not
    arbitrary EN. Implementation should write inline FR strings matching the
    existing pattern, NOT scaffold an i18n layer (that would be the kind of
    scaffolding CLAUDE.md forbids). No code conflict; the "i18n FR" criterion
    is satisfied by inline FR.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - T0 loopback trust: the endpoint is loopback-only, `authFetch` injects the
    `x-sbfb-token` bearer (`auth.ts:130-139`), `callDaemon` routes through it
    (`daemon.ts:212`). `searchBrowse` MUST go through `callDaemon`/`authFetch`
    (not raw `fetch`) — the `listBrowse` pattern already does. No regression
    if mirrored faithfully.
  - Query injection: server-side `sanitize_query` (`search.rs:36-50`) quotes
    every token and strips NUL before FTS5 MATCH; the front passes `q` as a URL
    query param (must be `encodeURIComponent`-ed in the path build, as the
    bridge does at `useBridge.ts:366`). No client-side injection surface beyond
    correct URL encoding.
  - **XSS via provenance render (non-blocking hardening note).** The hit render
    will display `repo_url` (and possibly `commit_sha`/`archive_hash`). Text
    interpolation `{value}` is auto-escaped by React. BUT rendering `repo_url`
    as `<a href={hit.repo_url}>` does NOT block a `javascript:`/`data:` scheme
    — React does not sanitize `href`. This exactly mirrors the existing
    `Browse.tsx:264`, `BrowsedProject.tsx:367`, `VerificationDetail.tsx:185`
    anchors (all raw `href={...repo_url}` with `rel="noopener noreferrer"`,
    no scheme guard). The data is feed-sourced (a malicious feed op could carry
    `repo_url: "javascript:..."`). Since this is identical to the established
    pattern, it is **not a regression** and not blocking. Recommended (cheap,
    optional): render the link only when `repo_url` starts with `https://`
    (a one-line guard), or render `repo_url` as plain text. Flag for the S74
    fork work where the triplet drives an action.
- HARDENING_ROADMAP status: no Phase E pre-requirement. Scope cut #11
  (per-client rate-limit on search) was flagged for re-eval here: the search
  bar increases interactive traffic to `GET /api/daemon/search`, but the
  endpoint is loopback-only (single local user), the residual T-SEARCH-DOS is
  "acceptable pre-launch" (THREAT_MODEL §11), and the front already debounces.
  Re-eval outcome: **rate-limiter not required for S73**; carry to S74 stands.
- Finding: clean (T0 preserved; XSS note is non-regression hardening).

## S4 Protocol And Wire Invariants
- Wire/security files checked (producer->consumer trace, each key):
  - Producer: `http.rs::search_handler` (1975-2040). Response envelope:
    ```json
    { "results": [ <hit> ], "total": <u64>, "took_ms": <u64> }
    ```
    Per-hit `<hit>` object (http.rs:2010-2027), keys in snake_case:
    `project_id`(string), `project_name`(string), `category`(string),
    `description`(string), `op_type`(string), `source_type`(string),
    `score`(number, f64), `repo_url`(string|null), `commit_sha`(string|null),
    `archive_hash`(string|null), `provenance_hash`(string|null),
    `is_open_source`(bool).
  - Backing struct: `SearchResult` (`search.rs:7-34`). `repo_url/commit_sha/
    archive_hash/provenance_hash` are `Option<String>` -> serialise to JSON
    `string` or `null`. `is_open_source` is a plain `bool` (never null).
    `score` is `f64`. The four provenance keys are `null` for non-release ops
    (verified test `search_result_null_triplet_for_non_release_op`,
    search.rs:717-735) and for any pre-M17 row.
  - HTTP shape test that pins the contract: `search_handler_json_includes_
    triplet` (http.rs:6455-6510) asserts `json["results"][0]` carries the four
    provenance keys + `total` + `took_ms`. The S67 baseline test (http.rs:
    6442-6449) pins `total`, `results[]`, `took_ms`.
  - Consumer (to be written): `SearchResponseSchema` in `daemon.ts`. The
    `callDaemon` helper validates with `.strict()` schemas and **THROWS
    `ApiProtocolError`** on any extra/missing key (daemon.ts:249-252).
- **F3 (load-bearing S4 contract, non-blocking IF the schema mirrors the
  envelope exactly).** The Zod schema MUST model:
  - the envelope object `{ results, total, took_ms }` (NOT a bare array);
  - `total` and `took_ms` as `z.number().int()` (both serialised as JSON
    numbers from `u64`);
  - each hit with all 12 keys: the seven base keys as required strings/number,
    and the four provenance keys as `z.string().nullable()` (the Rust side
    ALWAYS serialises them, as `null` when absent — so `.nullable()` not
    `.optional()` is the faithful mirror; using `.optional()` is also tolerant
    but `.nullable()` matches the wire exactly). `is_open_source` as
    `z.boolean()` (always present, never null).
  - If `.strict()` is used (consistent with the file's other schemas), every
    one of the 12 keys must be declared or the parser throws. Mirror the
    `BrowseEntrySchema` precedent (daemon.ts:147-173) where the daemon "always
    serializes" optional-looking fields and the schema keeps `.optional()` as
    runtime tolerance — but here, because the producer always emits all 12
    keys, declare them all (provenance four as `.nullable()`).
- VERSION/domain/canonical status: no `*_VERSION` touched. `search_index` is a
  LOCAL daemon FTS5 table (M17, Phase D), not a network wire format; the
  endpoint JSON is a daemon-local DTO over loopback. `FEED_FORMAT_VERSION` and
  `PROJECT_ANNOUNCEMENT_VERSION` remain 1 (kickoff §145-150). No canonical
  bytes, no signing domain touched.
- Day 0 status: **preserved.** D4 (dedicated Browse field via `searchBrowse`),
  D3 (no SearchManifest wire), pre-launch raw-op policy all honoured. Frontend
  `web/`-only phase (Rust-first exemption per D4 §399).
- Finding: clean (the wire is stable; F3 is an implementation-correctness note
  on schema shape, not a wire-format conflict).

## Risks And Scope Cuts
- Blocking risks: none.
- Non-blocking risks / carry-over:
  - F1/F3 schema-shape: the schema must mirror the `{results,total,took_ms}`
    envelope and the 12-key hit (four provenance `.nullable()`). Getting this
    wrong throws `ApiProtocolError` at runtime -> caught by the planned
    `search_response_schema_parses_triplet` test if it asserts on the real
    producer shape (it must use a fixture matching http.rs:2010-2039).
  - F2 i18n: write inline FR strings; do not scaffold an i18n layer.
  - XSS provenance anchor: optional `https://` href guard (non-regression).
  - R5 (kickoff): EN strings -> write FR from the start; `scan-en-strings.sh`
    in the Phase E acceptance block.
- Scope cuts still honored (kickoff §7): #11 rate-limit -> re-evaluated here,
  NOT required S73 (loopback-only, debounced), carry to S74 stands; #14
  pagination buttons -> S74+; #2/#3/#4 search/open/fork + atelier -> S74. The
  provenance triplet is rendered for display only this sprint (fork is S74).

## Action
- **SCOPE-CUT-CONSISTENT**: proceed with Phase E. Implement the dedicated
  Browse search field via `searchBrowse()` + `SearchResponseSchema`, honoring:
  (1) Zod schema mirrors the `{results,total,took_ms}` envelope + 12-key hit
  (four provenance `.nullable()`, `is_open_source` bool) so `.strict()` does
  not throw; (2) route through `authFetch`/`callDaemon` (bearer + DaemonResult)
  exactly like `listBrowse`; (3) inline FR strings (no i18n scaffold);
  (4) React Query v5 idiom (`enabled: q non-empty`, `placeholderData:
  keepPreviousData`, debounced `q` in key); (5) optional `https://` href guard
  on the provenance link. Track the F1/F2 plan-vs-reality drifts in the commit
  body (no plan-file edit; snapshot preserved).
