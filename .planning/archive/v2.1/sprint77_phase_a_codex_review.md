No blocking correctness bug found. Overall verdict: **CONFIRMED**, with the accepted limitation that this is not a live WAN/NAT proof; the repo discloses that the `b3` cross-machine harness remains the real WAN proof.

**Deliverable Verdicts**

1. **Keepalive primitive: CONFIRMED**
   - New module/export exists: `crates/nexus-core-rs/src/doc_sync.rs`, exported in `crates/nexus-core-rs/src/lib.rs:44,95`.
   - Uses `EndpointAddr` peers and reissues `doc.inner().start_sync(peers)` only through the keepalive path: `doc_sync.rs:68-71`, `doc_sync.rs:134-145`.
   - `subscribe()` is observability-only: it watches `NeighborUp/NeighborDown` at `doc_sync.rs:189-245`; worker claim remains poll-based via `get_many_by_prefix(b"task:")` at `crates/nexus-worker-core/src/engine/runtime.rs:911-923`.
   - Immediate empty-neighbor rejoin and periodic backstop are implemented: `doc_sync.rs:233-256`; cooldown is enforced at `doc_sync.rs:140-145`.
   - Reconnect backoff resets to `500ms` after a healthy subscription: `doc_sync.rs:177-179`, `doc_sync.rs:203-209`.
   - Shutdown is bounded and non-busy: watch shutdown exits in the main select and reconnect sleep: `doc_sync.rs:223-226`, `doc_sync.rs:263-266`. Empty peers exit immediately: `doc_sync.rs:171-175`.
   - The 64-buffer claim matches installed `iroh-docs-0.98.0`: `~/.cargo/.../iroh-docs-0.98.0/src/api.rs:459-471`; non-neighbor events are drained at `doc_sync.rs:243-245`.

2. **Engine wiring: CONFIRMED**
   - New `task_doc_peers` field exists: `runtime.rs:162-170`.
   - Capture-before-consume is correct: parsed `DocsTicket` is cloned for `ticket.nodes` before `import_ticket(ticket)`: `runtime.rs:360-368`.
   - Keepalives spawn only for imported docs with non-empty peers: `runtime.rs:684-699`.
   - `register_task_doc` only inserts `task_docs`, no peers, so injected tests get no keepalive: `runtime.rs:564-571`.
   - Teardown order is correct: keepalives are stopped and awaited before `node.shutdown()`: `runtime.rs:760-770`.

3. **Tests: CONFIRMED**
   - Red/green structural test is meaningful: B imports, baseline `k1` converges, B calls `leave()`, A writes `k2`, control asserts no convergence, then keepalive restores convergence: `doc_sync.rs:325-372`.
   - No-peers immediate-exit test exists: `doc_sync.rs:382-396`.
   - Real dispatch-loop live incremental test writes via `dispatch_loop::run` after `NeighborUp`: `dispatch_loop.rs:348-358`, `dispatch_loop.rs:408-426`.
   - Bulk catch-up non-regression exists: `dispatch_loop.rs:452-467`.
   - Remote result write symmetry exists: `dispatch_loop.rs:489-508`.
   - Engine peer-capture test asserts imported ticket peers are captured: `runtime.rs:1697-1740`.
   - I ran the targeted tests: `cargo test -p nexus-core-rs doc_sync::tests::`, `cargo test -p nexus-shell-daemon convergence_`, and `cargo test -p nexus-worker-core engine_captures_coordinator_peers_from_imported_ticket`; all passed.

4. **0 wire bump: CONFIRMED**
   - No `Cargo.toml`, `Cargo.lock`, `task.rs`, or `canonical.rs` tracked diff; changed tracked files are only `lib.rs`, `dispatch_loop.rs`, `runtime.rs`, `PATTERNS.md`, `THREAT_MODEL.md`.
   - `TASK_FORMAT_VERSION` remains `1`: `crates/nexus-core-rs/src/task.rs:61`.
   - canonical domains remain unchanged: `crates/nexus-core-rs/src/canonical.rs:73-80`.
   - Production `task:` key remains the existing dispatch key: `dispatch_loop.rs:35-41`.
   - iroh remains pinned to 0.98: `Cargo.toml:38-40`; lock has `iroh-docs 0.98.0` at `Cargo.lock:4031-4032`.

5. **Docs: CONFIRMED**
   - `PATTERNS.md §P63` documents the root cause and fix without claiming subscribe is the sync lever: `docs/rust/PATTERNS.md:3496-3528`.
   - Accepted limitation is honestly disclosed: in-process tests are green guards, live WAN proof is `b3`/T2: `docs/rust/PATTERNS.md:3533-3538`.
   - Threat model states no new admission boundary and same coordinator DocTicket basis: `docs/security/THREAT_MODEL.md:959-976`.

**Band-Aid Check**

No parallel HTTP push channel, no iroh fork, no iroh 1.0 upgrade, and no N0 relay inserted into the delivery hot path. The implementation uses existing iroh-docs 0.98 `start_sync`, `subscribe`, and `leave` primitives. Full workspace gates were not run; this review ran only the targeted phase tests.