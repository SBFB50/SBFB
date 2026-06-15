CONFIRMED CARRY-5 clamps `offset` before search call and UTF-8-safe truncates `q` before FTS5: crates/nexus-shell-daemon/src/http.rs:3678
CONFIRMED `truncate_on_char_boundary` backs down to a char boundary, avoiding mid-char panic: crates/nexus-shell-daemon/src/http.rs:1359
CONFIRMED `search_clamps_offset_and_query` proves huge offset empty page, oversized UTF-8 query 200, normal query still finds; targeted test passed: crates/nexus-shell-daemon/src/http.rs:10743
CONFIRMED CARRY-2 terminal helper sets `TaskStatus::Rejected`: crates/nexus-coordinator-rs/src/validator.rs:190
CONFIRMED pre-guardrail status gate refuses terminal `Rejected` resurrection: crates/nexus-coordinator-rs/src/validator.rs:98
CONFIRMED HTTP ingress calls `reject_result_on_guardrail_trip` on output trip before returning 400: crates/nexus-shell-daemon/src/http.rs:3181
CONFIRMED gossip ingress calls the same helper on output trip and skips persistence/kudos: crates/nexus-shell-daemon/src/validator_loop.rs:81
CONFIRMED terminality test sends a clean post-trip result and still ends Rejected with no text/kudos; targeted test passed: crates/nexus-shell-daemon/src/validator_loop.rs:332
CONFIRMED quorum-path guardrail trip is also covered and rejects clean resurrection; targeted test passed: crates/nexus-shell-daemon/src/validator_loop.rs:428
CONFIRMED PULL-1 strips existing `provenance.json` before artifact hash and fresh injection: crates/nexus-shell-daemon/src/deploy.rs:389
CONFIRMED `strip_zip_member` is exact-name, all-match stripping with byte-identical absent-member return and raw-copy of other entries: crates/nexus-shell-daemon/src/deploy.rs:957
CONFIRMED `deploy_strips_existing_provenance` proves single fresh survivor, `index.html` survival, and absent-member byte identity; targeted test passed: crates/nexus-shell-daemon/src/deploy.rs:1094
CONFIRMED FORK-1 caps archive entries at 4096 and returns before `create_dir_all(dest)`: crates/sbfb-factory/src/fork.rs:220
CONFIRMED `fork_entry_count_capped` builds 4097 entries and asserts `!dest.exists()`; targeted test passed: crates/sbfb-factory/src/fork.rs:631
CONFIRMED Dockerfile is pinned to `rust:1.94` and installs `libgtk-3-dev`: docker/ci/Dockerfile:4
CONFIRMED no Cargo/web dependency delta in Phase G tracked diff; changed status contains no Cargo.toml/Cargo.lock/web files: docker/ci/Dockerfile:8
CONFIRMED wire constants remain version 1; `NODE_DIRECTORY_FORMAT_VERSION=1` is additive and existing `INVITE_FORMAT_VERSION=2` is pre-existing: crates/nexus-core-rs/src/node_directory.rs:84
CONFIRMED THREAT_MODEL §15.1 states real residuals instead of claiming duress/fresh-flood closure: docs/security/THREAT_MODEL.md:880
CONFIRMED PATTERNS §P59 matches terminal guardrail, strip-before-inject, and entry-cap decisions: docs/rust/PATTERNS.md:3294
CONFIRMED PATTERNS P37 keeps `/browse` wire unchanged and composes seeder badge front-side: docs/shell/PATTERNS.md:2251
CONFIRMED verification.md counters are current for this tree: local `cargo nextest list --workspace --locked` counted 1755, matching the documented +1 quorum test adjustment: .planning/active/sprint75_verification.md:70
CONFIRMED SPRINT_LOG routes live residuals to S76 and reports 1755 Windows / 1759 Docker / 367 Vitest without closing deferred debt: docs/claude/SPRINT_LOG.md:19
OVERALL: PASS