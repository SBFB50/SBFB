**Findings First**

1. `PARTIAL` - `404 -> pending` is valid for “task exists but no result yet,” but it also masks “task id not found.” Backend returns `404` for both cases (`tasks_api.rs:187-198`), while `getTaskResult` maps every `404` to `{ready:false,status:"pending"}` (`web/src/api/coordinator.ts:724-725`). The compute tester then waits up to 120s (`examples/compute-tester/app.js:28-29`, `74-99`). Sane for submit-then-poll; not strictly “not masking genuine error.”

2. `PARTIAL` - Repointing iframe `task_submit` does not break any repo-visible real `submitTask` caller. Excluding SDK/tests, only `examples/compute-tester/app.js:59` calls it. However, the surviving TabView `task_submit` surface still calls `submitAppTask` (`ButtonBlock.tsx:64-72`) which still targets `/app/{name}/tasks/submit` (`coordinator.ts:617-621`), while the daemon exposes `/api/v1/tasks/submit` and app state routes, not app task submit (`http.rs:340`, `360-362`). Not a new regression, but still a broken legacy surface if TabView task buttons are in scope.

3. `PARTIAL` - The 5 named SDK copies are byte-identical, hash `BF9D3BB9419719BFB607CC5E02EE5C83078922AD8F135DC980073B21243A2A8A`. But there are 4 additional factory template `sbfb-bridge.js` files still lacking `getTaskResult` (`crates/sbfb-factory/src/templates/static/sbfb-bridge.js:28`, same pattern in react/pyodide/static-reader). Fine if “5 copies” means runtime/example bundles only; gap if factory-generated apps should inherit Phase H.

**Per Deliverable**

- `CONFIRMED` - Daemon `GET /api/daemon/project-info` is additive and loopback-authed. Route is under `authed_routes` (`http.rs:274-278`) and the auth middleware wraps the route set (`http.rs:497`). Handler returns `{project_doc_id}` or null (`http.rs:841-846`). Unit test covers the null branch (`http.rs:6915-6940`); non-null is only live-acceptance/planning-backed (`sprint76_verification.md:266-286`).

- `CONFIRMED` - `project_id` spoof resistance is correct. `useBridge` reads project-info first, then parses `{ ...req.payload, project_id: info.project_doc_id }`, so host value overwrites app payload (`web/src/bridge/useBridge.ts:242-252`). Test asserts fetch order and injected body (`useBridge.test.ts:507-519`).

- `CONFIRMED` - Submit now uses daemon-level primitives and nested `task.task_id`. `submitComputeTask` posts to `/api/v1/tasks/submit` and returns `entry.task.task_id` (`coordinator.ts:681-691`). Rust `TaskSubmission` required/default fields match the TS schema shape (`types.rs:72-111`, `coordinator.ts:661-667`).

- `CONFIRMED` - Rust/TS bridge allowlists are mirrored at 16 methods. Source arrays include `task_result` (`sbfb-manifest/src/lib.rs:67-90`, `web/src/bridge/protocol.ts:20-48`), tests pin the canonical set (`protocol.test.ts:149-178`, `lib.rs:198-229`). Minor doc nit: `lib.rs:191` still says “15 methods.”

- `PARTIAL` - Allowlist parity is declarative, not per-app runtime authorization. The manifest code says this explicitly (`sbfb-manifest/src/lib.rs:52-57`), and `useBridge` validates source and `BridgeRequestSchema` but does not check the loaded app’s `SBFB.json bridge.methods` before dispatch (`useBridge.ts:126-164`, `226+`). This is pre-existing, but adding `task_result` makes the global dispatch surface larger.

- `CONFIRMED` - No new P2P wire format found in the reviewed diff. The change is loopback HTTP plus bridge/API code. `TaskSubmission` is unchanged (`types.rs:72-111`); no version bump appears in the touched protocol path.

- `CONFIRMED` - No static secret leak found. `project_doc_id` is already used to enroll/share the local worker doc ticket (`local_worker.rs:305-322`) and whitelist exactly that project (`local_worker.rs:330-335`). The acceptance script fetches the loopback token at runtime and prints only token length (`scripts/acceptance/phase_h_compute_local.sh:15-21`).

- `CONFIRMED` - `task_result` SDK method is present in the named runtime/example bundle (`web/public/sbfb-bridge.js:173-182`) and compute tester declares both bridge methods (`examples/compute-tester/SBFB.json:7-9`). The example uses a button, not form submit (`index.html:57`), and polls via `getTaskResult` (`app.js:74-99`).

- `CONFIRMED` - Scope cuts are honest. Production search found no `task_result_ready`; only tests mention it. Planning says push SSE/iroh-docs subscribe is deferred to S77 and not started (`sprint76_phase_h_review.md:134-137`), matching code comments (`protocol.ts:44-48`, `app.js:10-14`).

**Overall Verdict**

`PARTIAL`, not reject. The core Phase H local compute bridge path is confirmed: project-info, host-injected project id, daemon submit, result polling, allowlist parity, and the compute tester all line up. The remaining issues are bounded but real: 404 conflates pending with missing task, factory template SDKs lag the five named SDK copies, per-app bridge method declarations are still not runtime-enforced, and TabView’s legacy app-scoped task submit remains dead rather than fixed. I did not rewrite code or rerun the full suites; I ran read-only diff/status/hash/grep checks plus `git diff --check`, which was clean.
