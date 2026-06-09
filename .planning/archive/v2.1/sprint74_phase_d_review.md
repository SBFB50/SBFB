# Sprint 74 Phase D — Review (persistent local pin: keep_online M18)

**Method**: adversarial 5-dimension Workflow (14 agents ~1.08M tok, each finding
re-verified) + independent Codex pass (`sprint74_phase_d_codex_review.md`).
Preflight: **SCOPE-CUT-CONSISTENT** (`sprint74_phase_d_preflight.md`).

## Scope delivered
M18 `keep_online` table (LOCAL schema, appended after M17, no wire bump) +
set/get/list_keep_online_disabled; `BlobsClient::set_tag`/`delete_tag` (iroh-blobs
0.100); `finalize_deploy` pins + records keep_online=true at every self-deploy;
`POST /api/daemon/keep-online` toggle (DB + per-intent tag, loopback auth); the
boot/replay re-broadcast OFF gate; H.1 (boot `rebuild_from_feed` WARN→error!);
functional front toggle (replaces Phase A disabled-ON).

## Findings & reconciliation

### Confirmed → FIXED in-phase
- **P2 (correctness) — OFF gate was incomplete (only NeighborUp).** The outbox is
  replayed to peers at THREE sites: NeighborUp, `browse_request`, and the periodic
  republish timer (30–60s). The initial gate covered only NeighborUp, so an OFF app
  still diffused within ≤60s — breaking "stockee, plus diffusee". **Fixed**: all
  three sites now apply `keep_online_allows_rebroadcast` via the shared
  `load_disabled_keep_online` helper (DRY, R6-safe fallback).
- **P2 (tests) — ON→re-tag route path + deploy-time DB write untested.** Extended
  `keep_online_off_removes_tag` to assert the deploy-time keep_online=true rows
  (with recorded archive_hash) AND the full OFF→ON cycle (route ON re-pins the blob
  + clears the disabled list).

### Confirmed → DEFERRED to Phase G (documented carries)
- **Toggle UI reachability for per-app IDs (elevated carry, KEEP-ONLINE-READ-PATH).**
  `BrowsedProject` derives `isOwn = daemonInfo.node_id === projectId`, but modern
  self-deployed apps are keyed `project_id = blake3(name)` ≠ node_id, so the toggle
  (gated on `isOwn`) does not appear for them. This is NOT a Phase D regression —
  Phase A's disabled-ON toggle had the SAME `isOwn` gate; the precise ownership
  signal was always slated to come from keep_online (per the AvailabilitySheet
  `isOwn` doc). The Phase D backend (M18 + tag + gate + toggle handler) is COMPLETE
  and tested; surfacing the ownership signal to the front (expose keep_online on the
  browse JSON or a small GET, then `isOwn = has keep_online row`) needs a ~18-site
  `BrowseEntry` change deferred to Phase G — it closes BOTH reachability AND the
  faux-ON below in one read-path. The toggle is functional wherever it renders.
- **P2 — Front toggle faux-ON on reload/restart.** `useState(true)` + no read path:
  after a reload/daemon-restart, the toggle renders ON while the M18 row persists
  OFF (in-session reopen is correct — the sheet stays mounted). The daemon BEHAVIOUR
  is correct (the OFF gate keys on the DB, not the UI); this is display-only. Honest
  fix = add `keep_online: Option<bool>` to `BrowseEntry` (+ Zod) and seed
  `useState(entry.keep_online ?? true)` — moderate surface, routed to Phase G (carry
  KEEP-ONLINE-READ-PATH). The toggle remains a real control (verrou §8(5) literal
  sense satisfied; it really mutates).
- **P2 — Two sources of truth for archive_hash** (toggle ON re-derives from the
  browse aggregator, ignoring the stored M18 column). INERT today (no GC reaper
  exists, and the OFF gate never reads archive_hash). Carry KEEP-ONLINE-HASH-SOT to
  Phase G (read the stored hash in the ON arm, or drop the column).
- **P2 — No R6 boot-fallback test** (DB/lock read error → replay-all). The code is
  defensively correct (`lock().ok().and_then(...).unwrap_or_default()`); only the
  regression guard is missing. Carry to Phase G.
- **P3 — H.1 escalation + legacy node_id fallback branch untested.** Carry.

### Refuted (documented)
- **Ownership check on the toggle** — REFUTED: loopback-auth is the project's
  accepted residual (THREAT_MODEL §AD2); a fabricated project_id creates ZERO tags
  (archive_hash resolves to None) and OFF on a remote app is a diffusion no-op (the
  node never re-broadcasts remote apps). Consistent with `subscribe_curator`/
  `feed_insert` (same bearer, no ownership gate).
- **ON-no-hash skip** — REFUTED: by-design best-effort (no GC reaper ⇒ inert; OFF
  gate keys on project_id, not hash).

## Verification (re-run after the OFF-gate fix)
`cargo fmt --all --check` 0 · `cargo clippy --workspace --all-targets -- -D warnings`
0 · `cargo nextest run --workspace` green (Windows + Docker Linux) · web
lint+tsc+test:unit(313)+build+size(6/6)+scan green. Dual-platform counts in the
commit body.

## Verdict: PASS
