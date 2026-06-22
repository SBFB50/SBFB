## 1. Daemon Route + DTO
Verdict: CONFIRMED

Evidence:
- Public routes are only `/health` and `/blob-serve`: `crates/nexus-shell-daemon/src/http.rs:253-256`.
- `GET /api/daemon/shard-session/{session_id}` is registered inside `authed_routes`, not `public_routes`: `crates/nexus-shell-daemon/src/http.rs:274-309`.
- `authed_routes` receives `auth_required`: `crates/nexus-shell-daemon/src/http.rs:502`; that middleware enforces bearer token, Host, and Origin: `crates/nexus-shell-daemon-core/src/auth.rs:416-450`.
- DTOs derive `Serialize` and expose only `{session_id, member_count}` plus `{found, session}`: `crates/nexus-shell-daemon/src/http.rs:2103-2124`.
- The raw manifest contains private identities (`initiator`, `worker_pubkey`): `crates/nexus-core-rs/src/shard_plan.rs:142-147`, `crates/nexus-core-rs/src/shard_plan.rs:233-243`.
- The projection copies only `manifest.session_id` and `manifest.plan.assignments.len()`: `crates/nexus-shell-daemon/src/http.rs:2131-2135`.
- `live_shard_session` is a documented stub returning `None`: `crates/nexus-shell-daemon/src/http.rs:2138-2147`.
- Current `DaemonHttpState` has no shard-session registry field; its field list ends with `seed_registry`: `crates/nexus-shell-daemon/src/http.rs:75-196`.
- The response path is `shard_session -> shard_session_response -> project_shard_session`, and the handler always returns `StatusCode::OK`: `crates/nexus-shell-daemon/src/http.rs:2155-2180`.
- The 200-empty contract matches the `seed_count` precedent, which also returns `StatusCode::OK` with JSON defaults: `crates/nexus-shell-daemon/src/http.rs:2551-2558`.
- The handler logs a static string and does not log `session_id`: `crates/nexus-shell-daemon/src/http.rs:2178-2180`.

Clean for this deliverable: pubkey leak is not physically reachable through the current route.

## 2. Daemon Tests
Verdict: CONFIRMED

Evidence:
- `shard_session_response_pins_empty_envelope` serializes the pure response, asserts `found:false`, physical `session` key presence, `session:null`, and exactly two envelope keys: `crates/nexus-shell-daemon/src/http.rs:5238-5259`.
- `shard_session_projection_hides_member_identities` builds distinct `initiator`, `worker_a`, and `worker_b`: `crates/nexus-shell-daemon/src/http.rs:5263-5271`.
- The test inserts worker pubkeys into `ShardAssignment.worker_pubkey`: `crates/nexus-shell-daemon/src/http.rs:5272-5282`.
- It asserts `member_count == 2`, exact `session_id`, no worker/initiator hex in serialized output, no `worker_pubkey`/`initiator` keys, and exactly two view fields: `crates/nexus-shell-daemon/src/http.rs:5294-5323`.

Clean for this deliverable: the whitelist assertion is strong enough because object length `== 2` plus exact field assertions block alternate identity fields, not only the obvious key names.

## 3. Front API Client
Verdict: CONFIRMED

Evidence:
- `ShardSessionViewSchema` has only `session_id` and nonnegative integer `member_count`, and is not followed by `.strict()`: `web/src/api/daemon.ts:535-548`.
- `ShardSessionStatusResponseSchema` is strict at the envelope and requires `session: ShardSessionViewSchema.nullable()`: `web/src/api/daemon.ts:552-565`.
- This mirrors Rust’s always-serialized `Option<ShardSessionView>` field: `crates/nexus-shell-daemon/src/http.rs:2118-2124`.
- `getShardSession` calls `callDaemon` and URL-encodes the id path segment: `web/src/api/daemon.ts:575-583`.
- `callDaemon` uses `authFetch`, parses JSON, and throws `ApiProtocolError` on schema drift: `web/src/api/daemon.ts:231-246`, `web/src/api/daemon.ts:290-298`.
- Row tolerance for future `pipeline_status` / `verification_level` is explicitly documented and implemented by omitting `.strict()` on the row: `web/src/api/daemon.ts:540-548`.

Clean for this deliverable.

## 4. Front API Tests
Verdict: CONFIRMED

Evidence:
- Empty-state parse and exact URL assertion: `web/src/api/__tests__/daemon.test.ts:984-1003`.
- URL encoding assertion for `a b/c -> a%20b%2Fc`: `web/src/api/__tests__/daemon.test.ts:1005-1015`.
- Found-session parse for aggregate `member_count`: `web/src/api/__tests__/daemon.test.ts:1017-1027`.
- Strict-envelope rejection for an unknown top-level key: `web/src/api/__tests__/daemon.test.ts:1029-1035`.
- Additive row tolerance for `pipeline_status` / `verification_level`, with stripped unknown key asserted: `web/src/api/__tests__/daemon.test.ts:1037-1054`.

Clean for this deliverable.

## 5. Front Panel + Route + Nav
Verdict: CONFIRMED

Evidence:
- `/compute` lazy route points to `ShardSessionPanel`: `web/src/App.tsx:82-87`.
- Nav exposes `/compute` as `Calcul`: `web/src/components/AppShell.tsx:59-64`.
- The panel imports `getShardSession`, not bridge APIs: `web/src/components/ShardSessionPanel.tsx:23-27`.
- The query calls `getShardSession(coordUrl, sessionId)` and is gated by `enabled: sessionId !== null && sessionId.length > 0`: `web/src/components/ShardSessionPanel.tsx:80-86`.
- `getShardSession` itself goes through `callDaemon`: `web/src/api/daemon.ts:575-583`.
- Mode state is exactly `idle | join | launch`: `web/src/components/ShardSessionPanel.tsx:71-78`.
- Launch resets `sessionId`; join exposes the form; submit trims and sets `sessionId`: `web/src/components/ShardSessionPanel.tsx:94-128`.
- Empty state is default when no lookup is active: `web/src/components/ShardSessionPanel.tsx:198-210`.
- All query outcomes are handled: loading, React Query error, unavailable daemon, HTTP error, found/not-found data: `web/src/components/ShardSessionPanel.tsx:223-280`.
- French user-visible intentions are exact: `web/src/components/ShardSessionPanel.tsx:101-115`.
- The private-group copy says invited machines participate and no central server; it does not claim encryption: `web/src/components/ShardSessionPanel.tsx:61-65`.
- Rendered status exposes only truncated `session_id` and `member_count`: `web/src/components/ShardSessionPanel.tsx:266-275`.

Clean for this deliverable.

## 6. Front Panel Tests
Verdict: CONFIRMED

Evidence:
- Test harness sets an active coordinator and renders `/compute`: `web/src/components/__tests__/ShardSessionPanel.test.tsx:44-62`.
- Test 1 asserts French CTAs, default empty state, and absence of `shard`, `ALPN`, `ComputeGroup` in rendered text: `web/src/components/__tests__/ShardSessionPanel.test.tsx:72-94`.
- Test 2 asserts unknown id renders “not found” and no status panel: `web/src/components/__tests__/ShardSessionPanel.test.tsx:96-116`.
- Test 3 injects `worker_pubkey` and `initiator` sentinels in the mocked row, then asserts only member count renders and the leak string is absent from the DOM: `web/src/components/__tests__/ShardSessionPanel.test.tsx:118-153`.
- Test 4 asserts the no-active-node branch: `web/src/components/__tests__/ShardSessionPanel.test.tsx:155-163`.

Clean for this deliverable.

## 7. E2E
Verdict: CONFIRMED

Evidence:
- Default E2E excludes only `@compute`: `web/package.json:14`.
- CI runs `npm run test:e2e`: `.github/workflows/ci.yml:87-94`.
- Hermetic shard-panel describe has no `@shard` tag and opens `/compute`: `web/e2e/compute-shard.spec.ts:26-44`.
- Hermetic join-form test asserts disabled submit before an id, blocking blind empty lookup: `web/e2e/compute-shard.spec.ts:46-52`.
- Cross-machine describe is tagged `@shard`: `web/e2e/compute-shard.spec.ts:58`.
- Cross-machine execution is gated by `test.skip` on `SBFB_E2E_SHARD` and a nonempty session id, not by grep-invert: `web/e2e/compute-shard.spec.ts:55-62`.
- FR CTA assertions are byte-exact: `web/e2e/compute-shard.spec.ts:33-39`.

Clean for this deliverable.

## Final Gaps / Invariants
P0/P1/P2/P3 gaps: none.

Key design decision verdict: CONFIRMED as genuine PLAN-ADAPT, not lazy scope cut. The plan/preflight did ask for pipeline status and verification level: `.planning/active/sprint77_plan.md:468-473`, `.planning/active/sprint77_phase_j_preflight.md:175-180`, `.planning/active/sprint77_phase_j_preflight.md:192-196`. The implementation has no live store, returns `None`, and documents those runtime fields as Phase K additive fields: `crates/nexus-shell-daemon/src/http.rs:2095-2102`, `crates/nexus-shell-daemon/src/http.rs:2138-2147`, `crates/nexus-shell-daemon/src/http.rs:2171-2176`. The frontend row is already additive-tolerant: `web/src/api/daemon.ts:540-548`.

0 wire bump: confirmed by diff metadata; the changed source introduces local HTTP DTOs only, not signed/canonical types: `crates/nexus-shell-daemon/src/http.rs:2103-2124`. The only `DOMAIN_*` occurrence in the new route area is a future-ingest comment: `crates/nexus-shell-daemon/src/http.rs:2143-2145`.

0 new dependency: confirmed by worktree diff metadata; changed code uses existing imports/deps (`serde` in Rust, Zod/authFetch in web, React Query/lucide in panel): `crates/nexus-shell-daemon/src/http.rs:56`, `web/src/api/daemon.ts:12-14`, `web/src/components/ShardSessionPanel.tsx:19-27`, `web/package.json:36`, `web/package.json:40`, `web/package.json:48`.

DaemonHttpState shard field: absent in current state shape; no shard registry is present in the listed fields: `crates/nexus-shell-daemon/src/http.rs:75-196`.

D5 private ComputeGroup invariant: respected by the projection whitelist; the raw manifest has `initiator` and worker identities, but the route serializes only `session_id` and `member_count`: `crates/nexus-core-rs/src/shard_plan.rs:142-147`, `crates/nexus-core-rs/src/shard_plan.rs:233-243`, `crates/nexus-shell-daemon/src/http.rs:2131-2135`.

Test delta: reconcilable. Plan estimated Phase J `+0 Rust / +3 Vitest`: `.planning/active/sprint77_plan.md:572`. Actual added test intent is Rust +2 (`shard_session_response_pins_empty_envelope`, `shard_session_projection_hides_member_identities`) and Vitest +9 (5 API + 4 panel): `crates/nexus-shell-daemon/src/http.rs:5238-5263`, `web/src/api/__tests__/daemon.test.ts:984-1037`, `web/src/components/__tests__/ShardSessionPanel.test.tsx:72-155`.

