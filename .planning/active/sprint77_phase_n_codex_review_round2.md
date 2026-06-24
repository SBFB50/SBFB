## docs/sharding/examples/sign_verify.rs ??? CLEAN
- Lift fidelity holds: fixtures in `docs/sharding/examples/sign_verify.rs:39-86` match `crates/nexus-core-rs/src/shard_plan.rs:602-649`.
- Test bodies are not weakened: `docs/sharding/examples/sign_verify.rs:90-103` matches `crates/nexus-core-rs/src/shard_plan.rs:651-664`; `docs/sharding/examples/sign_verify.rs:105-119` matches `crates/nexus-core-rs/src/shard_plan.rs:686-700`.
- Include-safe comments: regular `//`, with explicit rationale at `docs/sharding/examples/sign_verify.rs:11-17`.

## crates/nexus-core-rs/tests/shard_sign_verify.rs ??? CLEAN
- The wrapper `include!`s `../../docs/sharding/examples/sign_verify.rs` from `CARGO_MANIFEST_DIR`, so the path resolves from `crates/nexus-core-rs` to repo root: `crates/nexus-core-rs/tests/shard_sign_verify.rs:16-19`.
- The included file carries the runnable tests itself: `docs/sharding/examples/sign_verify.rs:90` and `docs/sharding/examples/sign_verify.rs:105`.
- External visibility is satisfied by public exports: `crates/nexus-core-rs/src/lib.rs:100` and `crates/nexus-core-rs/src/lib.rs:192-197`.

## docs/sharding/WIRING_SPEC.md ??? PARTIAL
- Required 5-section structure exists: authority `docs/sharding/WIRING_SPEC.md:11`, actor model `docs/sharding/WIRING_SPEC.md:42`, per-step contract `docs/sharding/WIRING_SPEC.md:59`, HTTP contract `docs/sharding/WIRING_SPEC.md:137`, invariants `docs/sharding/WIRING_SPEC.md:162`.
- Contiguity is not conflated with coverage: `docs/sharding/WIRING_SPEC.md:76-83`, matching `crates/nexus-core-rs/src/shard_plan.rs:201-212` and `crates/nexus-coordinator-rs/src/placement.rs:294-305`.
- Threat honesty is present: PROVISIONAL/S78 at `docs/sharding/WIRING_SPEC.md:29-33`, admission != confidentialite at `docs/sharding/WIRING_SPEC.md:35-40`, run-proof PROVISIONAL/S78 at `docs/sharding/WIRING_SPEC.md:117-126`.
- GAP: HTTP auth/route clauses claim loopback bearer+Host+Origin and `authed_routes` at `docs/sharding/WIRING_SPEC.md:132-133` and `docs/sharding/WIRING_SPEC.md:147-149`, but the cited source_refs there only cover response/projection at `docs/sharding/WIRING_SPEC.md:129-130` and `docs/sharding/WIRING_SPEC.md:143-146`. The missing rank-1 anchors exist: `crates/nexus-shell-daemon/src/http.rs:276`, `crates/nexus-shell-daemon/src/http.rs:309`, `crates/nexus-shell-daemon/src/http.rs:2147`, `crates/nexus-shell-daemon-core/src/auth.rs:395`.

## docs/sharding/llms.txt ??? CLEAN
- Truth Stack and Not-evidenced rule are present: `docs/sharding/llms.txt:12-15`.
- Sharding index points to contract, wire spec, primitives, control plane, examples, and bridge gap: `docs/sharding/llms.txt:17-41`.
- Contiguity/coverage separation is indexed directly: `docs/sharding/llms.txt:24-25`.

## llms.txt ??? CLEAN
- Scope banner is explicitly sharding-only and defers whole-repo indexing: `llms.txt:8-12`.
- Indexed entries are sharding-only: `llms.txt:14-18`.
- Non-sharding material is explicitly not agent-indexed yet: `llms.txt:20-22`.

## docs/sharding/examples/observe.curl.md ??? CLEAN
- Route and source are stated: `docs/sharding/examples/observe.curl.md:10-16`.
- Loopback auth headers are shown in the request: `docs/sharding/examples/observe.curl.md:26-33`.
- Empty-store response is exactly `{ "found": false, "session": null }`: `docs/sharding/examples/observe.curl.md:36-42`, matching `crates/nexus-shell-daemon/src/http.rs:2119-2134`.
- Projection avoids identities: `docs/sharding/examples/observe.curl.md:44-50`, backed by `crates/nexus-shell-daemon/src/http.rs:2096-2105` and `crates/nexus-shell-daemon/src/http.rs:5231-5287`.

## docs/sharding/examples/bridge_gap.md ??? CLEAN
- GAP-not-shipped / PROPOSED status is explicit: `docs/sharding/examples/bridge_gap.md:1-5`.
- Listed bridge methods match the closed enum, with no shard method: `docs/sharding/examples/bridge_gap.md:18-23` and `web/src/bridge/protocol.ts:20-49`.
- Unknown shard bridge method is rejected by schema boundary per doc claim: `docs/sharding/examples/bridge_gap.md:25-27`, backed by `web/src/bridge/protocol.ts:53-57`.
- Placeholder method is marked non-contractual and not shipped: `docs/sharding/examples/bridge_gap.md:39-48`.

## scripts/check-sharding-docs.sh ??? CLEAN
- Phase-N file existence coverage includes all 8 deliverable surfaces: `scripts/check-sharding-docs.sh:121-134`.
- Source-ref-check fails missing files and missing symbols: `scripts/check-sharding-docs.sh:156-193`.
- Rank-1 prefix list is exactly `crates|docs|web|scripts`: `scripts/check-sharding-docs.sh:156-159` and `scripts/check-sharding-docs.sh:193`.
- Truth Stack and Not-evidenced assertions are enforced: `scripts/check-sharding-docs.sh:196-203`.
- PROVISIONAL/S78/cardinal/root-scope honesty markers are enforced: `scripts/check-sharding-docs.sh:205-214`.
- BusyBox grep constraints are respected by the implementation: no `grep -P` / no `\b` policy at `scripts/check-sharding-docs.sh:5-7`, and source-ref grep uses `grep -oE` / `grep -qF` at `scripts/check-sharding-docs.sh:180` and `scripts/check-sharding-docs.sh:193`.

## OVERALL ??? GAPS FOUND
1. P2: `docs/sharding/WIRING_SPEC.md` leaves the HTTP route/auth tier clauses without their own `path:Symbol` source_ref. Fix: add explicit refs for `crates/nexus-shell-daemon/src/http.rs:authed_routes`, `crates/nexus-shell-daemon/src/http.rs:shard_session`, and `crates/nexus-shell-daemon-core/src/auth.rs:auth_required` near `docs/sharding/WIRING_SPEC.md:132-149`.
