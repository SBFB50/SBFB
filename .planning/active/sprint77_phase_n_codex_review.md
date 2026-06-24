## docs/sharding/examples/sign_verify.rs ??? CLEAN
- Fixture bodies are inlined from `shard_plan.rs`: `sample_assignment`/`sample_manifest`/`sample_proof` match `crates/nexus-core-rs/src/shard_plan.rs:602`, `:615`, `:633` and appear in `docs/sharding/examples/sign_verify.rs:39`, `:52`, `:70`.
- Both lifted test bodies preserve the assertions: source `crates/nexus-core-rs/src/shard_plan.rs:651` and `:686`; example `docs/sharding/examples/sign_verify.rs:90` and `:105`.
- Uses regular `//` comments, not inner docs, before the include boundary: `docs/sharding/examples/sign_verify.rs:1`.

## crates/nexus-core-rs/tests/shard_sign_verify.rs ??? CLEAN
- The wrapper includes the doc example via `env!("CARGO_MANIFEST_DIR")/../../docs/sharding/examples/sign_verify.rs`, which resolves from `crates/nexus-core-rs`: `crates/nexus-core-rs/tests/shard_sign_verify.rs:16`.
- The included file carries its own `#[test]` functions: `docs/sharding/examples/sign_verify.rs:90`, `docs/sharding/examples/sign_verify.rs:105`.

## docs/sharding/WIRING_SPEC.md ??? CLEAN
- Required five-section structure is present: authority `docs/sharding/WIRING_SPEC.md:11`, actor model `:42`, per-step contract `:59`, HTTP contract `:140`, invariants `:169`.
- Contiguity is not conflated with coverage: doc says first layer need not be 0 and cites `is_pipeline_contiguous`/`covers_full_model` at `docs/sharding/WIRING_SPEC.md:77`; source confirms split at `crates/nexus-core-rs/src/shard_plan.rs:206` and `crates/nexus-coordinator-rs/src/placement.rs:299`.
- Threat honesty is present: PROVISIONAL/S78 `docs/sharding/WIRING_SPEC.md:29`, admission != confidentiality `:35`, run-proof PROVISIONAL/S78 `:119`, no identity projection `:176`.

## docs/sharding/llms.txt ??? CLEAN
- Truth Stack and Not evidenced rule are explicit: `docs/sharding/llms.txt:12`.
- Sharding anchors resolve to rank-1 sources, including `verify_signature`, `covers_full_model`, `auth_required`, and `BridgeMethodSchema`: `docs/sharding/llms.txt:24`, `:25`, `:33`, `:35`.
- PROVISIONAL/S78 status is explicit: `docs/sharding/llms.txt:8`.

## llms.txt ??? CLEAN
- Root scope banner is sharding-only and defers whole-repo indexing: `llms.txt:8`.
- The indexed entries are sharding docs only: `llms.txt:16`, `llms.txt:17`, `llms.txt:18`.

## docs/sharding/examples/observe.curl.md ??? CLEAN
- Documents the exact route and loopback bearer/Host headers: `docs/sharding/examples/observe.curl.md:11`, `:30`.
- Empty-store response matches source behavior: doc `docs/sharding/examples/observe.curl.md:41`; source returns `found:false, session:None` at `crates/nexus-shell-daemon/src/http.rs:2130`.
- Populated projection omits `worker_pubkey` and `initiator`: doc `docs/sharding/examples/observe.curl.md:46`; source projection only emits `session_id` and `member_count` at `crates/nexus-shell-daemon/src/http.rs:2100`.

## docs/sharding/examples/bridge_gap.md ??? CLEAN
- Marks shard bridge API as GAP-not-shipped / PROPOSED / S78: `docs/sharding/examples/bridge_gap.md:1`, `:3`, `:39`.
- Closed bridge enum has no shard method in source: doc `docs/sharding/examples/bridge_gap.md:25`; source enum is `web/src/bridge/protocol.ts:20` through `web/src/bridge/protocol.ts:49`.

## scripts/check-sharding-docs.sh ??? CLEAN
- Phase-N file presence, link checks, source-ref checks, Truth Stack checks, and honesty markers are implemented: `scripts/check-sharding-docs.sh:129`, `:138`, `:156`, `:214`, `:223`.
- Source-ref rank prefixes are exactly `crates|docs|web|scripts`, with `.planning/` excluded by design: `scripts/check-sharding-docs.sh:156`, `scripts/check-sharding-docs.sh:193`.
- Broken file/symbol anchors set `fail=1`: missing file `scripts/check-sharding-docs.sh:172`, missing symbol `scripts/check-sharding-docs.sh:180`.
- BusyBox-sensitive grep is static-safe: script documents no `grep -P`/word-boundary use at `scripts/check-sharding-docs.sh:5`; implemented grep calls use `-oE`, `-qF`, `-qx`, `-qE` at `scripts/check-sharding-docs.sh:65`, `:70`, `:108`, `:193`.
- Runtime caveat: I could not execute this script here because `bash` resolves to unavailable WSL; the script requires bash at `scripts/check-sharding-docs.sh:1`.

## OVERALL ??? CLEAN
1. No P0/P1/P2 gaps found.
2. Fix: none required from this audit. Runtime gate still needs to be run in an environment with Bash and Cargo/nextest available. 
