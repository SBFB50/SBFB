Verdict global: **PASS-PARTIAL**. I found **0 P0/P1 GAP**. I found **1 P3 PARTIAL**: the scope/wire wording “all versions = 1 / 4 domains” is imprecise repo-wide.

1. **HARNESS — CONFIRMED**

`bash -n` passed via Git Bash. The no-config run produced `RIG-ABSENT`, exit `3`, and exactly 10 JSON keys.

Evidence: JSON fields are fixed in `scripts/acceptance/b3_shard_pipeline.sh:151-162` and fallback emitter mirrors the same 10 keys at `:181-184`; numeric coercion is int-or-null at `:146-150` and `:177-180`. `pass()` is only reached after `run_proof` non-empty and `toks_per_s >= 1` guards at `:369-388`; hollow PASS on the current stub is not reachable. No-config structural diagnosis is at `:229-239`.

Data-plane claim matches code: `accept` serves a long-lived bi-stream and loops over frames at `crates/nexus-core-rs/src/shard.rs:310-324`. `open_shard_connection` documents a future caller at `:183-200`; callsites found are test-module only (`#[cfg(test)]` at `:351-352`, test call at `:492`).

2. **THREAT_MODEL.md — CONFIRMED**

v14 is honest: Phase K wrap-up, T2 `RIG-ABSENT`, no prod orchestrator/RunProof caller, route stub, feature `PROVISIONAL + carry P1 S78` at `docs/security/THREAT_MODEL.md:1420-1438`.

Relabel grep is clean: remaining hits are historical v10/v12/v13 (`:1376`, `:1398`, `:1415`) plus v14 correction note (`:1432-1435`). Active text points S78 for in-vivo emission/sketch/arbitration/SI-9/SI-11/SI-5 at `:1032`, `:1074-1077`, `:1101-1104`, `:1260-1271`, `:1289-1295`.

`RunProof::new` is correctly described: it is public, not test-only, at `crates/nexus-core-rs/src/shard_plan.rs:457-461`; actual calls are under test modules, e.g. `shard_plan.rs:597-633`, `validator.rs:978-995`, `rerun.rs:152-175`.

Route phrase matches code: `ShardSessionView` exposes `session_id` + `member_count` at `crates/nexus-shell-daemon/src/http.rs:2104-2109`; `live_shard_session -> None` at `:2146-2148`; response returns `{found:false, session:null}` at `:2156-2164`.

3. **PATTERNS — CONFIRMED**

§P67 no longer says “ONE frame / NO decode loop”; it says looped boundary-frame forwarding, no autoregressive decode/orchestrator at `docs/rust/PATTERNS.md:3768-3775`. Code supports it: `ShardForwarder` at `shard.rs:223`, `EchoForwarder` at `:239`, loop at `:315`, worker `ShardBackendForwarder` at `crates/nexus-worker-core/src/llm/shard.rs:523-536`.

§P68 resolves: `placement_refuses_when_model_fits_single_worker` exists at `crates/nexus-coordinator-rs/src/placement.rs:665`; `covers_full_model` lives in `placement.rs:299-307`. §P69 resolves to `PerfMap`/DP/churn code at `routing.rs:125-141`, `:379-465`, `:488-557`.

Stale forward refs in P64/P65/P66 are S78-targeted, e.g. `docs/rust/PATTERNS.md:3566-3569`, `:3656-3659`, `:3740-3742`. Shell P39 is correct: whitelist `member_count`, stub `None`, stale source comments explicitly routed to S78, and French intentions at `docs/shell/PATTERNS.md:2303-2325`.

4. **verification.md — CONFIRMED**

Observed matrix matches the requested numbers: Win nextest `1949/1949 0 skipped`, Docker `1947/1953` with six named iroh-networked Docker-on-Windows failures, Vitest `411`, coverage `87.27/79.01/86.02/88.59`, E2E `41+1skip`, fmt/clippy/doctest zero at `.planning/active/sprint77_verification.md:19-39`.

§6 uses the corrected hermetic F-row names at `:66` and explicitly rejects `shard_backend_primitive` at `:74-78`. I mechanically resolved all listed functions; examples: `shard_window_validates_contiguous_range` at `llm/shard.rs:564`, `top_k_extracts_largest_by_magnitude_deterministically` at `:619`, `hidden_token_count_validates_shape` at `:645`, `toploc_commitment_is_deterministic_and_swap_sensitive` at `:658`. No code function matched `shard_backend_primitive`.

T2 artifact text matches the real harness output and correctly states structural unreachable PASS today at `.planning/active/sprint77_verification.md:89-124`.

5. **sprint78_audit_plan.md — CONFIRMED**

The four 3/3 carries are present at `.planning/active/sprint78_audit_plan.md:104-115`. `SHARD-PROVISIONAL` P1 is explicit at `:115` and detailed at `:168-176`. `TEST-ISOLATION-SBFB-HOME` is present at `:119` and detailed at `:129-138`. `STALE-PHASE-K-COMMENTS` is present at `:139-147`. Track Testabilité standing is present at `:154-166`.

6. **SCOPE/WIRE/DAY-0 — PARTIAL (P3 wording precision)**

Confirmed: `git diff HEAD --name-only` shows only docs/planning/CLAUDE/.gitignore paths, no `.rs/.ts/.tsx/.toml/.lock`. Day-0 invariants are explicitly preserved at `.planning/active/sprint78_audit_plan.md:39-42`. No Rust code `pub const DOMAIN_*_V2+` exists.

P3 PARTIAL: the phrase “all `*_FORMAT_VERSION`/`*_ANNOUNCEMENT_VERSION`/`SCHEMA_VERSION` = 1” is false repo-wide because `crates/nexus-worker-core/src/invite.rs:73` has pre-existing `INVITE_FORMAT_VERSION: u16 = 2`. Also “4 `DOMAIN_*_V1` additive” is imprecise for S77: the repo has five S77 additive domains: `DOMAIN_COMPUTE_GROUP_V1`, `DOMAIN_SHARD_PLAN_V1`, `DOMAIN_RUN_PROOF_V1`, `DOMAIN_VRF_DRAW_V1`, `DOMAIN_ACTIVATION_COMMIT_V1` at `crates/nexus-core-rs/src/canonical.rs:258`, `:276`, `:290`, `:310`, `:332`. `sprint77_verification.md:33` already states the correct caveat/count.

**GAP list:** none P0/P1.

**PARTIAL list:** P3 `SCOPE-WIRE-PRECISION` only, as above.

