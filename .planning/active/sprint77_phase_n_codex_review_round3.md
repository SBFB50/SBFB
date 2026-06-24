## docs/sharding/examples/sign_verify.rs ??? CLEAN
- Fixtures are inlined and match the private test fixtures: `sample_assignment` at `docs/sharding/examples/sign_verify.rs:39` vs `crates/nexus-core-rs/src/shard_plan.rs:602`; `sample_manifest` at `docs/sharding/examples/sign_verify.rs:52` vs `crates/nexus-core-rs/src/shard_plan.rs:615`; `sample_proof` at `docs/sharding/examples/sign_verify.rs:70` vs `crates/nexus-core-rs/src/shard_plan.rs:633`.
- The two test bodies preserve the source assertions: manifest roundtrip `docs/sharding/examples/sign_verify.rs:90` vs `crates/nexus-core-rs/src/shard_plan.rs:651`; run-proof roundtrip `docs/sharding/examples/sign_verify.rs:105` vs `crates/nexus-core-rs/src/shard_plan.rs:686`.
- Comments are regular `//`; no `//!` found, and the file states why at `docs/sharding/examples/sign_verify.rs:11`.

## crates/nexus-core-rs/tests/shard_sign_verify.rs ??? CLEAN
- The wrapper includes the doc example from the crate manifest dir with `../../docs/sharding/examples/sign_verify.rs`, which resolves from `crates/nexus-core-rs` to repo root: `crates/nexus-core-rs/tests/shard_sign_verify.rs:16`.
- It is an integration test wrapper whose included file carries `#[test]` functions: `crates/nexus-core-rs/tests/shard_sign_verify.rs:5`, `docs/sharding/examples/sign_verify.rs:90`, `docs/sharding/examples/sign_verify.rs:105`.

## docs/sharding/WIRING_SPEC.md ??? PARTIAL
- Required five-part structure is present: authority `docs/sharding/WIRING_SPEC.md:11`, actor model `docs/sharding/WIRING_SPEC.md:42`, per-step contract `docs/sharding/WIRING_SPEC.md:59`, HTTP contract `docs/sharding/WIRING_SPEC.md:138`, invariants `docs/sharding/WIRING_SPEC.md:167`.
- Contiguity is correctly separated from coverage: doc says contiguity does not require first layer 0 at `docs/sharding/WIRING_SPEC.md:77`; source confirms that at `crates/nexus-core-rs/src/shard_plan.rs:206`; coverage is separate in `docs/sharding/WIRING_SPEC.md:80` and `crates/nexus-coordinator-rs/src/placement.rs:299`.
- Threat honesty is present: PROVISIONAL/S78 at `docs/sharding/WIRING_SPEC.md:29`, admission != confidentiality at `docs/sharding/WIRING_SPEC.md:35`, run-proof S78 caveat at `docs/sharding/WIRING_SPEC.md:117`.
- GAP: the contract promises every clause has a grep-resolvable source_ref at `docs/sharding/WIRING_SPEC.md:8`, but the load-bearing claim “is_member BEFORE accept_bi / any frame read” only cites `SHARD_REJECT_NOT_MEMBER` at `docs/sharding/WIRING_SPEC.md:108`; the actual ordering evidence is `is_member` before close at `crates/nexus-core-rs/src/shard.rs:304` and `accept_bi` later at `crates/nexus-core-rs/src/shard.rs:314`. Add `crates/nexus-core-rs/src/shard.rs:accept_bi` or a numeric call-site ref.

## docs/sharding/llms.txt ??? CLEAN
- Truth Stack and Not evidenced rule are present: `docs/sharding/llms.txt:12`.
- Agent index points to WIRING_SPEC, protocol spec, wire primitives, control plane, bridge whitelist, examples, and threat model: `docs/sharding/llms.txt:19`, `docs/sharding/llms.txt:24`, `docs/sharding/llms.txt:31`, `docs/sharding/llms.txt:37`, `docs/sharding/llms.txt:54`.
- Source refs spot-check resolve: `verify_signature` at `crates/nexus-core-rs/src/shard_plan.rs:356`, `covers_full_model` at `crates/nexus-coordinator-rs/src/placement.rs:299`, `BridgeMethodSchema` at `web/src/bridge/protocol.ts:20`.

## llms.txt ??? CLEAN
- Root scope banner is explicitly sharding-only and defers whole-repo indexing: `llms.txt:8`.
- Root index links only the sharding agent index, wiring spec, and shard wire spec: `llms.txt:14`.
- Non-sharding scope is explicitly excluded: `llms.txt:20`.

## docs/sharding/examples/observe.curl.md ??? CLEAN
- Route, loopback token header, Host header, and empty-store response are present: `docs/sharding/examples/observe.curl.md:11`, `docs/sharding/examples/observe.curl.md:31`, `docs/sharding/examples/observe.curl.md:32`, `docs/sharding/examples/observe.curl.md:41`.
- Source route and response match: route registered at `crates/nexus-shell-daemon/src/http.rs:309`; empty response built at `crates/nexus-shell-daemon/src/http.rs:2124`; test pins `{found:false, session:null}` at `crates/nexus-shell-daemon/src/http.rs:5208`.
- Projection exposes only `session_id` and `member_count`: doc example `docs/sharding/examples/observe.curl.md:50`; source projection `crates/nexus-shell-daemon/src/http.rs:2100`.

## docs/sharding/examples/bridge_gap.md ??? CLEAN
- GAP-not-shipped / PROPOSED wording is explicit: `docs/sharding/examples/bridge_gap.md:1`, `docs/sharding/examples/bridge_gap.md:3`, `docs/sharding/examples/bridge_gap.md:39`.
- Actual host bridge enum contains app-facing methods and no shard method: `web/src/bridge/protocol.ts:20`.
- SDK exposed methods contain no shard method; method list starts at `web/public/sbfb-bridge.js:169` and internal dispatcher is `web/public/sbfb-bridge.js:397`.

## scripts/check-sharding-docs.sh ??? PARTIAL
- It would fail on syntactically present broken refs: missing file fails at `scripts/check-sharding-docs.sh:172`; missing symbol fails at `scripts/check-sharding-docs.sh:180`; numeric line out of range fails at `scripts/check-sharding-docs.sh:186`.
- Rank-1 prefix list is exactly `crates|docs|web|scripts`: `scripts/check-sharding-docs.sh:193`.
- BusyBox-safe grep surface is maintained: no `grep -P` / `\b` policy at `scripts/check-sharding-docs.sh:6`; actual source-ref grep uses `grep -oE` at `scripts/check-sharding-docs.sh:193`.
- GAP: it only validates refs already present in backticks, from the token stream at `scripts/check-sharding-docs.sh:163`; it does not enforce that each contract clause has a source_ref. The missing `accept_bi` source_ref in `docs/sharding/WIRING_SPEC.md:108` would silently pass.

## OVERALL ??? GAPS FOUND
1. P1: WIRING_SPEC misses the call-site source_ref for the `is_member` before `accept_bi` ordering claim. Fix: add `crates/nexus-core-rs/src/shard.rs:accept_bi` or `crates/nexus-core-rs/src/shard.rs:314` beside `docs/sharding/WIRING_SPEC.md:108`.
2. P1: source-ref lint verifies existing refs but not missing refs per clause. Fix: add a required-anchor check for known load-bearing clauses, at minimum `accept_bi`, or restructure WIRING_SPEC per-step lines so every claim line has a `source_ref` token and lint enforces it.
3. Verification limit: I could not execute `scripts/check-sharding-docs.sh`; this Windows shell maps `bash` to WSL and no WSL distro is installed. Static script audit only.
