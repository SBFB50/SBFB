Preflight verdict is **EXECUTE** at [.planning/active/sprint82_phase_s2_preflight.md:7](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_s2_preflight.md:7), consistent with the Sprint plan at [.planning/active/sprint82_plan.md:424](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_plan.md:424).

### 1. `browse_api.rs` production move — CONFIRMED

- File is exactly 635 LF-terminated lines with no CRLF: [browse_api.rs:1](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:1).
- Mechanical UTF-8 comparison against `git show be7e2be:.../http.rs`:
  - HEAD `853-1000` → current `37-184`: exact after only `list_browse` and `browse_pull` become `pub(crate)`.
  - HEAD `1161-1258` → current `186-283`: exact after only `list_nodes` becomes `pub(crate)`.
- Exactly three `pub(crate)` definitions exist, at [154](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:154), [170](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:170), and [269](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:269). Projection types remain private.
- Banner [1-23](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:1) only records present/past provenance and current invariants; no future promise.
- Cargo manifests and lockfile have zero diff.

### 2. Five co-migrated tests — CONFIRMED

Exact HEAD-to-current comparisons:

- `2180-2291` → [296-407](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:296)
- `2457-2528` → [409-480](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:409)
- `2530-2638` → [482-590](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:482)
- `2730-2752` → [592-614](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:592)
- `3445-3463` → [616-634](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:616)

Each definition occurs exactly once in the crate and none remains in `http.rs`. The test header at [285-294](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:285) includes `BrowseListResponse`, shared fixtures, `KeyPair`, and `create_node`. All six required fixtures remain `pub(crate)` in [test_support.rs:103](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:103), [114](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:114), [216](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:216), [705](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:705), [725](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:725), and [782](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:782).

### 3. `http.rs` constrained delta — CONFIRMED

- Line count: `6220 → 5635`; diff is `+7/-592`, net `-585`.
- An in-memory reconstruction of current `http.rs` from HEAD—removing only the six slices plus adjacent blanks, applying three route re-points and the one doc edit—returned `FULL_HTTP_RECONSTRUCTION_EXACT`.
- Routes remain in the authenticated router at [297-300](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:297) and [337](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:337).
- Parsed route-path multiset: HEAD `89`, current `89`, identical; each requested path occurs once.
- The only non-route textual edit is the de-bracketed reference at [758](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:758).
- `git diff --check` passes.

### 4. STAY symbols and tests — CONFIRMED

- `BrowseListResponse` remains `#[cfg(test)] pub` at [762-767](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:762). Its migrated browse tests consume it at [browse_api.rs:293](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:293), [612](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:612), and [632](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:632); `publish_api` retains its import and four consumers.
- Pull-resolution cluster remains intact at [http.rs:903-1013](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:903), with `blob_serve` callers at [1191-1217](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1191) and unchanged `seed_api` imports/callers.
- Hard-bound tests remain at [1957](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1957) and [2049](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2049).
- Other stayers remain: `wrap_payload_with_pow` [866](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:866), `truncate_on_char_boundary` [892](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:892), provenance/index chokepoints [1030-1038](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1030), and `mint_blob_ticket` [1295](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1295).
- The index regression test stays at [1874](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:1874); SPA `/browse` fallback stays at [2829](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2829).

### 5. `seed_api.rs` and `publish_api.rs` — CONFIRMED

`git diff --quiet be7e2be -- <file>` returned exit `0` for both:

- [seed_api.rs](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/seed_api.rs:23)
- [publish_api.rs](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/publish_api.rs:34)

The Phase-S zero-edit invariant is preserved.

### 6. `main.rs` module declaration — CONFIRMED

The sole delta is `mod browse_api;`, alphabetically between `apps` and `canary_api`, at [main.rs:31-33](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/main.rs:31).

### 7. `test_support.rs` documentation — CONFIRMED

Only the two comment lines changed. The refreshed description at [699-703](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/test_support.rs:699) accurately names migrated `seed_api`/`publish_api`/`browse_api` consumers and staying `http.rs` fork/pull-resolution consumers. No fixture implementation or visibility changed.

### 8. Documentation and anchors — CONFIRMED

- The sole `THREAT_MODEL.md` edit is the expected anchor at [line 1024](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/THREAT_MODEL.md:1024).
- Grep of all ten moved names across `docs/**/*.md` found only:
  - the corrected threat-model anchor;
  - name-only references in Rust/Shell patterns;
  - historical `SPRINT_LOG` content.
- No matching reference exists in `scripts/**/*.sh`, `.github`, or `.woodpecker`.
- No stale `crate::http::{moved_symbol}` qualifier remains anywhere.

### 9. Security and JSON invariants — CONFIRMED

- `browse_pull` begins immediately with the duress check at [171](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:171), returns `{"requested": false}` at [174-177](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:174), then sends gossip only at [179-182](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:179). It is the only duress occurrence in the module.
- Empty hashes are skipped at [101-103](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:101).
- CATALOG-BACKED membership logic is unchanged at [125-144](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:125).
- Verrou-4 wording remains at [227-230](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:227); `/browse` byte-identity rationale remains at [261-268](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:261).
- Shapes remain:
  - `{"entries": [...]}` at [164](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:164)
  - `{"requested": bool}` at [176/183](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:176)
  - `{nodes, observed}` at [194-202](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/browse_api.rs:194)

### 10. Gates and residual coverage — CONFIRMED, with sandbox qualification

- Fresh `cargo fmt --all --check`: exit `0`.
- Fresh `git diff --check`: exit `0`.
- The review records crate Nextest `466/466`, web `412/412`, and Windows workspace `2108/2108` at [review.md:108-115](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_s2_review.md:108).
- Full fresh Nextest was environment-blocked before compilation by denied access to `target\debug\.cargo-lock`.
- The newest test executable was built after all four changed Rust sources; its dep-info includes `browse_api.rs`. `--list` exposes all five migrated tests. Direct execution freshly passed:
  - `browse_views_derives_from_subscribed`
  - `nodes_response_pins_envelope_and_grouping`
- The three router tests reached execution but failed only because the read-only sandbox denied `tempdir` creation at `test_support.rs:137`, not because of assertions or routing.
- Confirmed residuals: nine golden functions exist but none targets browse/nodes; no direct `browse_pull` test. These are correctly consigned at [review.md:126-128](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_s2_review.md:126).

Notes:

- **P0/P1 gaps:** none.
- **P2 residual, pre-existing:** no browse/nodes golden and no direct `browse_pull` test.
- **P2 staging note:** unrelated research changes are also dirty in `.planning/research/`; do not use an indiscriminate `git add -A` for the S2 commit.
- **P3 reconciliation:** [review.md:42](/C:/Users/FlowUP/Documents/Code/nexus/.planning/active/sprint82_phase_s2_review.md:42) currently claims the status contains exactly the phase set, which is no longer literally true because of those unrelated research files.
- **Process note:** the review remains `PASS-PENDING`; it must be reconciled to exact `## Verdict: PASS` before commit, per [PROCESS.md:39-43](/C:/Users/FlowUP/Documents/Code/nexus/docs/agent/PROCESS.md:39).

**GLOBAL VERDICT: CLEAN / PASS WITH NOTES — Sprint 82 Phase S2 conforms to the EXECUTE contract; 0 P0, 0 P1.**