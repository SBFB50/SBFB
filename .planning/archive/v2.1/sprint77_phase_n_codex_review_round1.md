## docs/sharding/examples/sign_verify.rs ??? CLEAN
- Fixtures are inlined with matching fields/values: source fixtures at `crates/nexus-core-rs/src/shard_plan.rs:602`, `:615`, `:633`; example copies them at `docs/sharding/examples/sign_verify.rs:39`, `:52`, `:70`.
- Signing/verifying bodies preserve the assertions: source manifest test `crates/nexus-core-rs/src/shard_plan.rs:651-664`; example `docs/sharding/examples/sign_verify.rs:90-103`. Source run-proof test `crates/nexus-core-rs/src/shard_plan.rs:686-699`; example `docs/sharding/examples/sign_verify.rs:105-119`.
- Uses regular `//` comments, and documents why inner docs are forbidden for `include!`: `docs/sharding/examples/sign_verify.rs:1-30`.

## crates/nexus-core-rs/tests/shard_sign_verify.rs ??? CLEAN
- `include!` path resolves from `CARGO_MANIFEST_DIR` of `crates/nexus-core-rs` to repo-root `docs/sharding/examples/sign_verify.rs`: `crates/nexus-core-rs/tests/shard_sign_verify.rs:16-19`.
- The included example carries its own test functions: `docs/sharding/examples/sign_verify.rs:90` and `docs/sharding/examples/sign_verify.rs:105`.
- Imported public items are exported by core: `crates/nexus-core-rs/src/lib.rs:100`, `crates/nexus-core-rs/src/lib.rs:192-196`.

## docs/sharding/WIRING_SPEC.md ??? PARTIAL
- Required five sections exist: authority `docs/sharding/WIRING_SPEC.md:11`, actor model `:42`, per-step contract `:59`, HTTP contract `:129`, invariants `:154`.
- Contiguity is not conflated with coverage: spec says `is_pipeline_contiguous` does not require first layer 0 at `docs/sharding/WIRING_SPEC.md:77-83`; source agrees at `crates/nexus-core-rs/src/shard_plan.rs:208-212` and `crates/nexus-coordinator-rs/src/placement.rs:295-304`.
- Threat honesty is present: PROVISIONAL/S78 `docs/sharding/WIRING_SPEC.md:29-33`, admission != confidentialite `:35-40`, run-proof S78 caveat `:115-124`.
- GAP: spec promises every contract clause carries `path:Symbol` at `docs/sharding/WIRING_SPEC.md:8-9` and `:61-62`, but the caps clause uses bare `SESSION_ID_MAX` / `SHARD_GROUP_ID_MAX` / `SHARD_HASHES_MAX` with only a prose link at `:98-99`; the actual symbols exist at `crates/nexus-core-rs/src/shard_plan.rs:97`, `:103`, `:108`.
- GAP: OBSERVE is part of the sequence at `docs/sharding/WIRING_SPEC.md:56`, but the per-step contract is only “See §4” at `:126-127`, so it lacks the promised signed?/DOMAIN/caps/preconditions fields from `:61-62`.

## docs/sharding/llms.txt ??? CLEAN
- Truth Stack and Not evidenced rule present: `docs/sharding/llms.txt:12-15`.
- Agent entry points and source refs resolve: shard plan refs `docs/sharding/llms.txt:24` to `crates/nexus-core-rs/src/shard_plan.rs:356`; coverage ref `docs/sharding/llms.txt:25` to `crates/nexus-coordinator-rs/src/placement.rs:299`; HTTP projection refs `docs/sharding/llms.txt:33` to `crates/nexus-shell-daemon/src/http.rs:2124` and `:2100`.
- PROVISIONAL/S78 status present: `docs/sharding/llms.txt:8-10`.

## llms.txt ??? PARTIAL
- Scope banner says sharding-only and whole-repo index deferred: `llms.txt:8-12`.
- GAP: file still indexes non-sharding project orientation links under a dedicated section: `llms.txt:20-23`. That violates the “sharding subsystem only” PO scope.

## docs/sharding/examples/observe.curl.md ??? CLEAN
- Required route present: `docs/sharding/examples/observe.curl.md:10-12`.
- Loopback auth described and curl includes token + loopback Host, with absent Origin allowed: `docs/sharding/examples/observe.curl.md:18-22`, `:30-33`.
- Empty-store response matches daemon source: doc `docs/sharding/examples/observe.curl.md:36-42`; source `crates/nexus-shell-daemon/src/http.rs:2115-2133`.
- Projection does not expose `worker_pubkey` / `initiator`: doc `docs/sharding/examples/observe.curl.md:46-50`; source `crates/nexus-shell-daemon/src/http.rs:2096-2104`, test `crates/nexus-shell-daemon/src/http.rs:5272-5287`.

## docs/sharding/examples/bridge_gap.md ??? PARTIAL
- Correctly states the bridge has no shard method: `docs/sharding/examples/bridge_gap.md:25-27`; schema method enum has no shard entry at `web/src/bridge/protocol.ts:20-49`.
- Correctly marks future shard method as PROPOSED/GAP-not-shipped: `docs/sharding/examples/bridge_gap.md:1-5`, `:39-47`.
- GAP: “name and shape are frozen for S78” overclaims a future contract with no rank-1 source: `docs/sharding/examples/bridge_gap.md:3-4`, `:43-47`; current rank-1 schema is only the existing enum at `web/src/bridge/protocol.ts:20-49`.

## scripts/check-sharding-docs.sh ??? CLEAN
- Phase N files are asserted present: `scripts/check-sharding-docs.sh:121-133`.
- Rank-1 prefix list is exactly `crates|docs|web|scripts`: `scripts/check-sharding-docs.sh:156-159`, `:193`.
- Broken file refs fail: `scripts/check-sharding-docs.sh:172-175`; missing non-numeric symbols fail: `:178-183`; numeric line refs are range-checked: `:185-190`.
- Truth Stack / Not evidenced assertions exist: `scripts/check-sharding-docs.sh:199-203`.
- Honesty extension exists: WIRING_SPEC PROVISIONAL/S78/caveat at `scripts/check-sharding-docs.sh:209-211`, sharding llms PROVISIONAL/S78 at `:212-213`, root scope banner at `:214`.
- BusyBox-grep constraint holds in commands: no `grep -P`; grep usage is `-oE`, `-qF`, `-qE` at `scripts/check-sharding-docs.sh:65`, `:70`, `:89`, `:108`, `:153`, `:180`, `:193`.

## OVERALL ??? GAPS FOUND
1. P1: `WIRING_SPEC.md` has unanchored contract clauses and an incomplete OBSERVE per-step contract. Fix: rewrite bare caps as `crates/nexus-core-rs/src/shard_plan.rs:SESSION_ID_MAX`, `:SHARD_GROUP_ID_MAX`, `:SHARD_HASHES_MAX`; add OBSERVE signed?/DOMAIN/caps/preconditions with `crates/nexus-shell-daemon/src/http.rs:authed_routes`, `:shard_session_response`, `:project_shard_session`.
2. P2: root `llms.txt` leaks outside sharding scope. Fix: remove `llms.txt:20-23` or move it out of root `llms.txt`; keep only sharding links plus the deferred whole-repo banner.
3. P2: `bridge_gap.md` over-freezes a future method shape. Fix: remove “name and shape are frozen” and the concrete `shard_observe` payload, or label it explicitly non-contract/not frozen/not implemented until a rank-1 S78 source exists.
