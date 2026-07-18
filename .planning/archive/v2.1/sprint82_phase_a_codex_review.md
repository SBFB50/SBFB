Gate verdict: **PASS**. Both Round-3 P1 fixes are sound; Round-2 exit-race and duress fixes did not regress. No remaining P0/P1.

- **ANCRE rate limiting — PASS.** First pass remains immediate (`runtime.rs:2141-2148`). After every pass, the chain stays `active` throughout `sleep(pace)` (`runtime.rs:2152-2156`), then atomically consumes `dirty` or clears `active` under the coordinator mutex (`runtime.rs:2157-2166`). Therefore:
  - ingests before the decision set `dirty` and receive a trailing pass;
  - ingests after `active=false` start a new immediate chain;
  - consecutive passes are separated by at least `pace`;
  - no exit window can strand an ingest.
- **Serialization — PASS.** The boot driver acquires the shared lock at `runtime.rs:1193-1197`; every re-drive pass acquires the same lock at `runtime.rs:2144-2148`. The boot/re-drive double-`SeedAnnounced` race is closed.
- **Duress/subscription regression — PASS.** The identity gate precedes `configured.to_vec()` and any logging/DB/network operation (`runtime.rs:2117-2139`; only the non-observable `is_empty()` check precedes it). Production invokes the helper only after accepted directory ingest and `Some(boot_driver_state)` (`runtime.rs:1787-1810`). The previously verified callee gate is untouched by this diff.
- **BOOT_AFTER_SUBMIT attribution — PASS.** The mode requires `WORKER_BIN` before submission and truncates/captures the cold worker’s own log (`b3_live_pc_vps.sh:388-398`, `:339-356`). PASS now requires one log line containing both the exact task ID and `result written` (`:463-477`). A warm competitor producing the result while the cold worker merely sees/claims it cannot satisfy this gate.
- **WORKER warm transition — PASS.** `warm` is initialized outside the resubscription loop (`doc_sync.rs:270-282`), becomes permanently true after `NeighborUp`, and the immediate rebuilt-ticker tick is guarded by the now non-empty neighbor set (`doc_sync.rs:330-344`). The 60s transition is cooldown-gated (`doc_sync.rs:363-381`).

- **Tests — PARTIEL, P2.** The coalescing test awaits the chain, but the app was already pinned before this section (`http.rs:6261-6277`). Its final `has_tag` assertion (`:6297-6322`) would still pass if the active branch returned `None` without setting `dirty`, and it does not measure pass spacing. Add a test-only pass counter/barrier and assert one immediate pass plus exactly one pass no earlier than `pace`.
- **Docs — PARTIEL, P3.** `PATTERNS.md:2379-2390` still says to pace “only the trailing pass” and discusses ingests “during a pass”; the essential post-pass active grace window should be stated explicitly. The security claim itself remains supported by the implementation.

**Commit gate: PASS with one non-blocking P2 regression-test gap and one P3 wording cleanup.**