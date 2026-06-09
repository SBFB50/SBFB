# Sprint 74 Phase C — Review (atelier fork → redeploy under local identity)

**Method**: adversarial 5-dimension Workflow (correctness / security / scope-vs-preflight /
tests / architecture), 12 agents ~1.04M tok, each finding adversarially re-verified;
plus an independent Codex `gpt` pass (`sprint74_phase_c_codex_review.md`). Preflight
verdict: **SCOPE-CUT-CONSISTENT** (`sprint74_phase_c_preflight.md`).

## Scope delivered
- `deploy.rs`: extracted `finalize_deploy` (shared deploy tail — fresh **local-signed**
  provenance + blob store + canonical `publish_announcement` + `ReleasePublished`),
  `deploy_from_repo` routes through it (behaviour-preserving); NEW `deploy_workspace`
  (`POST /api/v1/deploy-workspace`) for the atelier-fork redeploy, `is_open_source`
  **forced false**.
- `http.rs`: route registered behind `auth_required` + per-route `DefaultBodyLimit`;
  8 new tests.
- `sbfb-factory`: `atelier.rs` (`fork` + `redeploy` CLI + `zip_workspace`), `fork`
  primitive wired (dropped `#[allow(dead_code)]`), `archive_hash` made load-bearing
  (blob-path integrity gate); `react` (vendored UMD, no-build, runs under CSP) +
  `pyodide` (honest experimental scaffold) templates.
- `web/BrowsedProject.tsx`: "Forker dans l'atelier" intention CTA (no fake action) +
  Source-anchor scheme guard.

## Findings & reconciliation

### Confirmed → FIXED in-phase
- **P1 (correctness) — `ForkTriplet.archive_hash` dead code under `clippy -D warnings`.**
  Reproduced (the mandatory clippy gate failed; `cargo build`/nextest only warn — the
  same gate-masking class as prior sprints). Fixed by making the field **load-bearing**:
  `fork_from_search_hit` now verifies `blake3(blob_bytes) == archive_hash` before writing
  the workspace (real integrity gate for the local `--archive` path, which bypasses the
  daemon's content-addressed fetch) + new `ForkError::ArchiveHashMismatch` + CLI
  `--archive-hash` + test `fork_from_search_hit_verifies_archive_hash`. `cargo clippy -p
  sbfb-factory --all-targets -- -D warnings` now green.
- **P2 (correctness) — axum 2 MB default body limit caps the deploy family before the
  100 MB handler ceiling.** A non-trivial fork (>2 MB) would 413 prematurely. Fixed with
  a per-route `DefaultBodyLimit::max(MAX_DEPLOY_BYTES)` on `/deploy` + `/deploy-workspace`
  + regression test `deploy_workspace_accepts_body_over_2mb` (real >2 MB Stored zip).
- **P2 (security) — `blob_serve::load` pre-allocated `Vec::with_capacity(entry.size())`
  from the untrusted zip header before the cumulative cap.** Fixed: cap the capacity hint
  at the remaining budget + bounded `take(remaining+1)` read (mirrors `fork::extract_zip`).
  Pre-existing surface; `deploy_workspace` adds an authenticated writer onto it, so closed
  here.
- **P2 (tests) — `is_open_source=false` never tested WITH a valid https lineage.** Added
  `deploy_workspace_with_lineage_stays_not_open_source` (the L2-consent / R5 regression
  guard: lineage `repo_url`+`commit_sha` present ⇒ card still `is_open_source=false`,
  attribution kept).
- **P2 (tests) — `deploy_workspace` 400 branches uncovered.** Added
  `deploy_workspace_rejects_bad_inputs` (empty name, non-https repo_url, bad sha, zip
  without index.html).
- **P3→fixed (architecture) — fork CTA narrower than the backend.** Broadened
  `canFork` to `isHttpsUrl(repo_url) || archive_hash` (matches forge OR archive). Tests
  added (archive-only shows; non-https-no-archive hidden).
- **GAP (Codex) — `ReleasePublished` op built with empty commit_sha then silently
  dropped** for a no-lineage workspace deploy. Fixed: the feed op is now gated on
  `is_open_source` (intentional, semantic — `ReleasePublished` asserts a verifiable
  release; a fork is still discoverable via gossip + browse-index).
- **GAP (Codex) — Source anchor unguarded `repo_url` (javascript:/data: XSS).** Pre-existing
  carry B.5; fixed in this file now (`isHttpsUrl` guard) — Phase G covers the remaining two
  anchors (Browse, VerificationDetail) + the multi-vector test.
- **GAP (Codex) — vendored `htm.umd.js` had no license header** while the README claims
  headers are preserved. Fixed: prepended an Apache-2.0 attribution header.
- **P3 (tests) — `finalize_deploy` is_open_source=true arm lost coverage** in the
  refactor. Added `finalize_deploy_open_source_arm_propagates_version_and_flag` (direct
  call; asserts card `is_open_source=true` + provenance `app_version`/`commit_sha`).
- **P3 — fork audit-log args / lineage clarity.** `--commit-sha`/`--archive`/`--archive-hash`
  now recorded in the audit log; `redeploy` documents why it omits a lineage commit
  (an edited fork ≠ a specific upstream commit).

### Confirmed → REFUTED (documented)
- **GAP (Codex) — proof_card gives a self-attested fork the same provenance boost.**
  Refuted: ALL provenance in SBFB is SLSA-L1 self-attestation (deploy-from-repo included),
  so the `+20 provenance_verified` is by-design; the `+10 is_open_source` bonus is the
  differentiator a fork (false) correctly misses. No Phase-C defect.

### Deferred (pre-existing, documented as carries)
- **P3 (security) — unescaped `{{name}}` substitution into generated HTML (self-XSS,
  sandbox-contained).** Pre-existing across ALL templates (static/static-reader predate
  Phase C); self-inflicted; the frozen CSP contains any payload. Carry to a template-wide
  `name` allowlist/escape pass (note in `sprint75_audit_plan.md`).
- **P3 — react/pyodide `bridge_methods: &[]`** despite shipping the bridge SDK — consistent
  with the existing `static` template (informational manifest hint, not an enforcement
  gate). Carry.
- **P2-family — `publish_blob`/`preview_load`/`files-upload` share the same 2 MB default
  limit.** Out of Phase C's deploy scope; the two `deploy` routes are fixed. Carry note.

## Verification (re-run after reconciliation)
`cargo fmt --all --check` 0 · `cargo clippy --workspace --all-targets --locked -- -D warnings`
0 · `cargo nextest run --workspace --locked` green · `web` lint+tsc+unit+build+size+scan green.
Dual-platform (Windows + Docker Linux) recorded in the commit body.

## Verdict: PASS
