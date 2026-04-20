# Rust patterns for SBFB

_Scratchpad for Sprint 1 learning. Fill in as you go, no pressure._

This file is the personal notebook for the Rust learning work in
Sprint 1. The goal is not to rewrite the Rust Book — it is to
capture the handful of patterns you will hit over and over again
while writing `nexus-core-rs`, `nexus-core-py` and `nexus-worker`.
When something clicks or bites, note it here. When Sprint 2 starts
you will come back to these notes instead of re-learning.

## Sprint 1 compile-time lessons (already learned)

Before Sprint 2 begins, here is the concrete drift between the
SBFB plan (written from memory) and the actual crates as published
on crates.io at the time of first compile. These were all fixed
during Sprint 1 and the fixes are the best reference for what to
expect when the next crate bump happens.

### iroh 0.97 `Endpoint::builder(preset: impl Preset)`

The plan's example was:

```rust
let endpoint = Endpoint::builder()
    .discovery_n0()
    .bind()
    .await?;
```

The real iroh 0.97 API is:

```rust
use iroh::endpoint::presets;
use iroh::Endpoint;

let endpoint = Endpoint::builder(presets::N0)
    .bind()
    .await?;
```

- `builder()` takes a required `preset: impl Preset` argument
- The preset bundles discovery (pkarr DHT) AND relay config
- `presets::N0` lives at `iroh::endpoint::presets::N0`
- There is no separate `iroh-pkarr-node-discovery` crate at 0.97 —
  pkarr is folded into iroh core

### `Endpoint::id()` returns `EndpointId`, not `node_id()`

The plan assumed `endpoint.node_id() -> NodeId`. Real API:

```rust
// Method is id(), not node_id()
let id: iroh::EndpointId = endpoint.id();
let as_string = id.to_string(); // 64 hex chars, e.g. "66b1bc28..."
```

### `Endpoint::close()` is infallible (no `?`)

```rust
// Returns (), not Result<()>
endpoint.close().await;
```

### PyO3 0.28 `Bound<'py, T>` replaces `&PyAny` / `&PyModule`

This is a major migration from the plan's PyO3 0.22 assumption.
Every legacy reference type has been replaced by `Bound<'py, T>`:

| Old (pre-0.22)             | New (0.22+ and 0.28)              |
|----------------------------|-----------------------------------|
| `&PyAny`                   | `Bound<'py, PyAny>`               |
| `&PyModule`                | `&Bound<'py, PyModule>`           |
| `PyResult<&PyAny>`         | `PyResult<Bound<'py, PyAny>>`     |
| `fn(py: Python<'_>, m: &PyModule)` | `fn(m: &Bound<'_, PyModule>)` |

The `#[pymodule]` signature no longer takes a `Python<'_>`
parameter at all — it is passed implicitly.

```rust
#[pymodule]
fn nexus_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_node, m)?)?;
    m.add_class::<PyNode>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
```

### `pyo3-asyncio` was renamed to `pyo3-async-runtimes`

The plan pinned `pyo3-asyncio = "0.21"`. That crate name still
exists on crates.io (capped at 0.20) but the canonical async bridge
was absorbed into the PyO3 org as `pyo3-async-runtimes`. Current
version is 0.28 matching pyo3 0.28.

```toml
# In Cargo.toml
pyo3-async-runtimes = { version = "0.28", features = ["tokio-runtime"] }
```

```rust
// In code — note the module name is pyo3_async_runtimes with
// underscore, not hyphen
pyo3_async_runtimes::tokio::future_into_py(py, async move {
    // Return PyResult<T> where T: IntoPyObject
})
```

### `future_into_py` now returns `PyResult<Bound<'py, PyAny>>`

Old signature (pyo3-asyncio 0.21): `fn(...) -> PyResult<&PyAny>`
New signature (pyo3-async-runtimes 0.28):

```rust
pub fn future_into_py<F, T>(
    py: Python<'_>,
    fut: F,
) -> PyResult<Bound<'_, PyAny>>
where
    F: Future<Output = PyResult<T>> + Send + 'static,
    T: for<'py> IntoPyObject<'py> + Send + 'static,
```

The wrapped function itself should return `PyResult<Bound<'py, PyAny>>`,
not `PyResult<&PyAny>`.

### General migration lesson

When pinned versions in a plan don't match crates.io, don't
pattern-match — open docs.rs/<crate>/<version> for the exact
signatures. `cargo doc --open -p <crate>` does the same locally.
The 6 drifts above were all caught in a single `cargo check`
iteration once the docs were read, not by guessing.

## Sprint 1 J9-J10 — iroh-docs setup recipe

The full working pattern, validated by the
`crates/nexus-core-rs/examples/two_nodes_docs_sync.rs` deliverable
which runs two endpoints in the same process and observes a live
`LiveEvent::InsertRemote` propagation. Key takeaways:

### Docs is a "meta protocol"

`iroh-docs` alone is not enough. You also need `iroh-blobs` (for
the entry content store) and `iroh-gossip` (for peer neighborhood
broadcast). All three must be registered on the endpoint's Router.

```rust
use iroh::{endpoint::presets, protocol::Router, Endpoint};
use iroh_blobs::{store::mem::MemStore, BlobsProtocol, ALPN as BLOBS_ALPN};
use iroh_docs::{protocol::Docs, ALPN as DOCS_ALPN};
use iroh_gossip::{net::Gossip, ALPN as GOSSIP_ALPN};

let endpoint = Endpoint::bind(presets::N0).await?;
let blobs = MemStore::default();
let gossip = Gossip::builder().spawn(endpoint.clone());
let docs = Docs::memory()
    .spawn(endpoint.clone(), (*blobs).clone(), gossip.clone())
    .await?;

let _router = Router::builder(endpoint.clone())
    .accept(BLOBS_ALPN, BlobsProtocol::new(&blobs, None))
    .accept(GOSSIP_ALPN, gossip)
    .accept(DOCS_ALPN, docs.clone())
    .spawn();
```

### Author creation

Every write needs an `AuthorId`. The Docs client exposes
`author_create()` to mint a fresh one. Persistent nodes get a
default author automatically — `author_default()` returns it.

```rust
let author = docs.author_create().await?;
```

### Share / import flow

```rust
use iroh_docs::api::protocol::{AddrInfoOptions, ShareMode};

// On the creator:
let doc = docs.create().await?;
let ticket = doc.share(ShareMode::Write, AddrInfoOptions::RelayAndAddresses).await?;

// On the joiner (another machine or another process):
let doc = docs.import(ticket).await?;
```

### Live sync observation

`doc.subscribe()` returns a `Stream<Result<LiveEvent>>`. To observe
a remote write you pattern-match on `LiveEvent::InsertRemote`:

```rust
use futures_lite::StreamExt;
use iroh_docs::engine::LiveEvent;

let mut events = doc.subscribe().await?;
while let Some(ev) = events.next().await {
    match ev? {
        LiveEvent::InsertRemote { entry, .. } => {
            // remote peer wrote `entry` and we just synced it
            println!("key: {:?}", entry.key());
        }
        LiveEvent::SyncFinished(_) => { /* initial sync done */ }
        LiveEvent::NeighborUp(_pk) => { /* new peer in swarm */ }
        _ => {}
    }
}
```

### Subscribe BEFORE you write

A subtle gotcha: call `doc.subscribe()` on the receiving peer
**before** the writing peer issues `set_bytes`, otherwise you
risk missing the first `InsertRemote` for entries that arrive
while the subscription is still being wired up. The example
sleeps 500 ms between subscribe and write so the two nodes have
time to establish their QUIC connection before the first entry
lands.

### Local + remote = same stream

`LiveEvent::InsertLocal` fires for writes YOU made, while
`LiveEvent::InsertRemote` fires for writes that arrived via
sync. Both go through the same subscribe stream, so filter on
the variant if you only care about remote changes.

Sections below are intentionally empty except for prompts. Do not
delete sections you cannot answer yet — leave them with a `TODO`
and come back later.

---

## 1. Ownership and borrowing

What the Rust Book covers in chapters 4-5.

### What `move` means in practice

TODO: write down the first time you hit a `value moved here` error
and how you fixed it. The canonical example is storing something
into a `tokio::spawn(async move { ... })` block — why does the
closure need `move`? What would happen without it?

### `&T` vs `&mut T` vs `T` — picking the right one

TODO: the rule of thumb is "start with `&T`, bump to `&mut T` only
when the compiler complains, never take `T` unless you really want
to consume it". Write examples from your own code.

### `Clone` is not the enemy

TODO: when is cloning a `String` or an `Arc<T>` perfectly fine?
(Hint: `Arc::clone()` is cheap, bumping a refcount is faster than
most people's mental model of it.)

---

## 2. Error handling

What the Rust Book covers in chapter 9.

### `Result<T, E>` and the `?` operator

TODO: examples from your own code. The `?` operator is the most
important syntax in real Rust codebases — every function that can
fail returns a `Result` and every caller propagates with `?`.

### `thiserror` vs `anyhow` — when to use which

- **`thiserror`**: when you are defining a library error type that
  callers will want to match on. Each variant becomes a distinct
  case they can handle. Example: `NexusError` in
  `crates/nexus-core-rs/src/error.rs`.
- **`anyhow::Error`**: when you are writing application code and you
  just want "something went wrong, here is a chain of causes, print
  it and exit 1". Example: `main()` in `nexus-worker`.

TODO: write down a specific case where you were tempted to use
`anyhow` inside a library and what convinced you to switch to
`thiserror`.

---

## 3. Async and await

What the Rust Book does NOT cover — this is tokio territory.

### `async fn` vs `fn -> impl Future`

TODO: explain to future-you why they look similar and when one
matters over the other.

### Why `tokio::spawn` needs `Send + 'static`

TODO: the first time you see the error
`` `std::rc::Rc<...>` cannot be sent between threads safely ``,
come back here and write down the fix.

### `select!`, `join!`, and when to use each

TODO.

### Cancellation safety

TODO: what does it mean for a future to be "cancellation safe" and
why does iroh care? (Hint: iroh's stream APIs are generally
cancellation safe but some composite operations are not.)

---

## 4. The tokio runtime

### Single-thread vs multi-thread

TODO: `#[tokio::main(flavor = "multi_thread")]` is the default for
server workloads. When would you ever want `current_thread`?

### `tokio::task::spawn_blocking` for sync CPU work

TODO: when you hit a synchronous CPU-heavy call (think
`blake3::hash(very_large_blob)`), wrapping it in `spawn_blocking`
lets the async executor keep servicing other tasks. Example.

---

## 5. iroh Endpoint lifecycle

This is the heart of `nexus-core-rs`.

### Boot sequence

```rust
let endpoint = iroh::Endpoint::builder()
    .discovery_n0()        // enables pkarr DHT + mDNS
    .bind()
    .await?;
```

TODO: what does each builder method actually configure? What is the
minimum you need for a local-network-only node? What do you need
extra for a node that must traverse NAT?

### Node identity — random vs persistent

TODO: `Endpoint::builder()` without a secret key mints a fresh
Ed25519 pair every boot. For the SBFB coordinators we need
persistent identity so peer ids stay stable across restarts. Find
the builder method and document it here.

### Graceful shutdown

`Endpoint::close().await` drains in-flight QUIC streams. Skipping
it leaves connections in an "aborted" state on the remote peer.
Always prefer the explicit close over `Drop`.

---

## 6. iroh-docs replica pattern

The core primitive for the SBFB task/result log.

### `Doc::create`, `Doc::join_by_ticket`

TODO.

### Author keys vs node keys — they are NOT the same thing

TODO: the node key identifies the peer, the author key identifies
who wrote an entry inside a doc. A single peer can author entries
under multiple author keys. Coordinator needs to understand this
before delegated multi-writer in v1.1.

### Subscribing to live changes

TODO: `Doc::subscribe()` returns a stream of events. When does it
yield vs wait? Cancellation safety?

### Last-write-wins caveat

The plan's "Limites honnêtes" section says iroh-docs is LWW on
timestamp, not causal order. Document here exactly where that bites
and the workaround we will use (`parent_task_id` metadata).

---

## 7. PyO3 gotchas

What you will hit writing `nexus-core-py`.

### `&PyAny` vs owned Python objects

TODO.

### Error conversion: `NexusError` -> `PyErr`

Already demonstrated in `crates/nexus-core-py/src/lib.rs` with the
`.map_err(|e| PyRuntimeError::new_err(format!("...: {e}")))`
pattern. Write down when you would want a custom Python exception
subclass instead (Sprint 2+).

### `pyo3-asyncio` and the tokio bridge

`pyo3_asyncio::tokio::future_into_py` wraps a Rust `Future` into a
Python awaitable. Under the hood it spawns the future on the
shared tokio runtime. This is the ONLY integration point between
Python's asyncio event loop and tokio. Do not try to call
`tokio::spawn` from a Python thread without going through this.

### Releasing the GIL around blocking operations

TODO: `py.allow_threads(|| { ... })` lets other Python threads run
while you are doing blocking Rust work. When do you need it?

---

## 8. Cargo ergonomics that matter

### `cargo build` vs `cargo check`

`cargo check` skips codegen and linking, runs in a fraction of the
time. Use it as your fast feedback loop during learning. Reserve
`cargo build` for when you actually need the binary.

### `cargo clippy` is not optional

Run `cargo clippy --workspace --all-targets` after every meaningful
change. It catches subtle bugs and teaches Rust idioms faster than
any tutorial.

### `cargo fmt` before every commit

No negotiation. Let the formatter handle style so you never think
about it again.

---

## 9. Debugging tips

TODO: fill in what works for you. Starting points:

- `RUST_LOG=debug` + the `tracing_subscriber::fmt()` pattern used in
  `nexus-worker/src/main.rs`
- `dbg!(&value)` is your friend for quick inspection
- `cargo expand` shows what macros generate when you are confused
- rust-analyzer hover / go-to-def in VS Code

---

## 10. Sprint 1 debrief — honest assessment

Due at the end of Sprint 1, before committing to Sprint 2.

Questions to answer:

1. Can you read Rust source code (e.g. iroh's own crate) without
   bouncing off?
2. Can you fix a compile error on your own most of the time, or
   does every issue still require LLM/Google?
3. Do you feel blocked on Sprint 2 (full `nexus-core-rs` + PyO3
   surface) or excited for it?
4. Is there a specific concept (lifetimes? async pinning? trait
   bounds?) that still feels opaque and would benefit from a
   targeted exercise?

If any of these answers is "no / blocked / opaque", that is the
trigger to invoke the no-return clause in the SBFB plan: pivot to
**Option D** (Python coordinator + small daemon sidecar talking
JSON-RPC, ~500 LOC Rust only). Do not push through if the fit is
wrong — the plan explicitly budgets 10 lost days for that case and
no more.

---

## Sprint 2 audit (2026-04-10)

Independent verification of 9 Sprint 2 modules by 9 parallel agents,
after commit `626d7eb`. Each agent cross-checked the code against
(a) the plan `magical-marinating-phoenix.md`, (b) Context7 docs,
(c) the local `~/.cargo/registry/...` source for iroh 0.97 crates,
and (d) ran `cargo test --lib <module>`. Reports stored in
`.planning/audit_sprint2/S{1..9}_*.md`.

### Scorecard

| # | Module          | Tests | Plan conformance           | Severity |
|---|-----------------|-------|----------------------------|----------|
| 1 | crypto.rs       | 15/15 | 100% (bonus `load_or_generate`) | clean |
| 2 | task.rs         | 10/10 | 100%                       | minor tech debt |
| 3 | verification.rs | 8/8   | 100% numeric fidelity      | 1 low contract drift |
| 4 | node.rs         | 4/4   | ALPN stack + stable id OK  | **1 critical** |
| 5 | docs.rs         | 3/3 (2-node sync PASS) | scope gap | **1 blocker for S3** |
| 6 | gossip.rs       | 2/2   | full API wrap              | 1 future bindings risk |
| 7 | blobs.rs        | 4/4   | scope gap                  | **1 blocker for S3** |
| 8 | discovery.rs    | 3/3   | scope gap (read-only)      | 1 scope, 1 quality |
| 9 | pyo3 bindings   | smoke PASS + `cargo check` clean | 100% | 2 minor |

Aggregate: **49 Rust unit tests + 1 Python smoke test green**. No
compilation errors. No `.unwrap()` in production code anywhere in
the stack.

### Critical finding — Node shutdown race (S4)

`crates/nexus-core-rs/src/node.rs:140` — `Node::shutdown` calls
`drop(self.router)` and then `self.endpoint.close().await`. Per
iroh 0.97 `protocol.rs:63-66`, `Router` carries an
`AbortOnDropHandle`: dropping it aborts the run-loop task immediately
and skips the graceful sequence (`protocols.shutdown()` →
`handler_cancel_token.cancel()` → `endpoint.close()`). The outer
`endpoint.close().await` then races with the router's own
abort-path close, potentially double-closing the endpoint.

**Correct pattern**:

```rust
pub async fn shutdown(self) -> Result<()> {
    self.router.shutdown().await?; // drives graceful teardown,
                                    // already closes endpoint
    Ok(())
}
```

The existing tests pass only because the loopback race window is
tiny. Under real load this leaks in-flight streams. **Must be
fixed before Sprint 3** because the worker binary will call
`shutdown` on SIGINT regularly.

### Scope gaps blocking Sprint 3

Two modules built a strict subset of what the plan required. Both
gaps block the Sprint 3 worker:

**S5 docs.rs — missing `query_prefix`**
: The plan Day 3 line is `"query par prefix, subscribe stream,
  export/import tickets"`. `subscribe` and share/import tickets are
  wrapped, but no prefix-scan method exists on `DocHandle`. The
  Sprint 3 worker binary will need to scan task-doc entries by
  prefix (`task:*`, `claim:*`, `result:*`) — without this, the
  worker cannot iterate pending tasks. Also missing:
  `author_list()`, `list()` for listing all docs on a node.

**S7 blobs.rs — missing `fetch via ticket`, `unpin`, `list_pinned`**
: The plan Day 5 line is
  `"fetch via ticket, pin, unpin, list_pinned"`. The wrapper has
  `add_bytes` / `get_bytes` / `has` only. The curator-list flow in
  the architecture is "gossip announces a blob ticket → worker
  fetches via ticket" — that whole code path has no wrapper yet.

Both can be added in <1 day each, before Sprint 3 begins or as
Sprint 3 Day 0.

### Tech debt logged (not blocking Sprint 3)

1. **S2 cross-language canonical bytes** —
   `crates/nexus-core-rs/src/task.rs:309`, `canonical_bytes()` uses
   `serde_json::to_vec` which emits **struct fields in declaration
   order**. Python's `json.dumps(sort_keys=True)` sorts
   **alphabetically**. `BTreeMap` keys inside `metadata` are fine,
   but the top-level `Task` / `Result` field order is fragile. This
   is NOT a bug today (Rust signs and verifies in a closed loop),
   but **will silently break** the moment a Python coordinator in
   Sprint 4 needs to produce canonical bytes for a `Task` and sign
   it. Fix options: (a) use `#[serde(rename_all = ...)]` + custom
   serializer that sorts keys, (b) define a canonical JSON writer
   with explicit key order. Decide before Sprint 4 Day 1.

2. **S2 missing Claim signed envelope** —
   `task.rs:259` — `Claim` has `new()` but no `sign()` / `verify`
   methods. The LWW race-condition described in the docstring
   requires the coordinator to authenticate claims, but the type
   system doesn't enforce it. Add `ClaimEntry { claim, pubkey, sig
   }` in Sprint 3 when the worker actually writes claims.

3. **S2 no domain-separation prefix** — canonical bytes carry no
   type tag. A valid `canonical_bytes(&claim)` is structurally
   similar to `canonical_bytes(&task)`. Prefix with e.g.
   `b"nexus-task-v1:"` / `b"nexus-result-v1:"` / `b"nexus-claim-v1:"`.

4. **S3 Layer 3 `passed` contract drift** —
   `verification.rs:229` returns `passed_overall = false` when
   logprobs check fails, but the Python reference
   (`nexus/compute/verification.py:251`) returns `passed=True`
   (only `trust_delta = -5` signals). Rust behavior is semantically
   sounder but not a faithful port. If any Python caller keys
   dispatch on `passed=True → keep dispatching`, it will incorrectly
   halt. Decision: either align Rust to Python, or align Python to
   Rust when the coordinator is ported.

5. **S6 `GossipClient<'a>` lifetime** —
   `gossip.rs:104-107` — `GossipClient` holds `&'a Gossip`. S9
   bindings got away with this by never storing a `GossipClient`
   across the FFI boundary, but any future binding that needs a
   long-lived Python `Gossip` handle will hit the borrow checker.
   Fix: change the field to owned `Gossip` (cheap clone via its
   internal `Arc`).

6. **S8 pkarr discovery not implemented** —
   `discovery.rs` only wraps local-address read. The plan S8 line
   also requires `publish pkarr record`, `resolve(node_id) →
   endpoints`, `periodic refresh`. Module doc explicitly defers to
   Sprint 4 — acceptable, but the plan's own acceptance criteria
   for S8 are unmet. Flag as Sprint 4 Day 1.

7. **S8 `my_addr()` no internal timeout** —
   `discovery.rs:80-133` polls up to 20 `updated().await` cycles
   without a wall-clock timeout. A Python caller without
   `asyncio.wait_for` can hang if the n0 relay is unreachable. Add
   `tokio::time::timeout` internally with a configurable duration.

8. **S9 `generate_secret` swallows errors** —
   `nexus-core-py/src/lib.rs:258-259` — `d.set_item(...).ok()` eats
   OOM errors. Change to `?` and return `PyResult`.

9. **S9 `Blobs::get_bytes` assumes 32-byte hash** —
   `lib.rs:477` — `array32()` cast on the returned hash vec. True
   for BLAKE3 today but not enforced in bindings.

10. **S1 TOCTOU on key file perms** —
    `crypto.rs:133-134` — `fs::write` then `set_owner_only_perms`.
    Window is microseconds on `tempdir`, but noted for
    high-sensitivity environments.

### Conclusion / go-no-go for Sprint 3

- 1 critical bug (S4 shutdown) → **fix in a dedicated commit** before
  Sprint 3 begins.
- 2 plan scope gaps (S5 query_prefix, S7 fetch-by-ticket) → **add in
  a second dedicated commit** before Sprint 3 begins (or as Sprint 3
  Day 0 — they're a prerequisite for the worker binary).
- 10 tech-debt items → tracked here, fix opportunistically in
  Sprint 3/4 when touched, with S2 canonical bytes and S6 lifetime
  as the two "must address before Sprint 4 Day 1" items.

Nothing in the audit invalidates the Sprint 2 architecture choices.
iroh 0.97 is correctly wired; the Rust port of the Python verifier
preserves all numeric deltas; PyO3 0.28 bindings smoke-test green
end-to-end. Sprint 3 is clear to start once the 3 blockers above
land as two clean commits.

---

## Sprint 7 canonical patterns (2026-04-11)

Sprint 7 shipped the `nexus-shell-daemon` P2P discovery layer
(curator lists + pkarr browse) in 5 atomic phases
(`2c896a8`..`6f32893`). The Rust-side patterns that differ from
or extend Sprint 2 are documented here for the Sprint 8 audit
gate to consume without reading every commit.

### Sprint 7.1 — `DOMAIN_CURATOR_LIST_V1` joins the canonical tags

`crates/nexus-core-rs/src/canonical.rs`:

```rust
pub const DOMAIN_CURATOR_LIST_V1: &[u8] = b"nexus-curator-list-v1";
```

The canonical bytes produced by `canonical_bytes(&list,
DOMAIN_CURATOR_LIST_V1)` are the signing surface for every
`CuratorListEntry`. The tag is independent from (and cannot
collide with) `DOMAIN_TASK_V1`, `DOMAIN_RESULT_V1`,
`DOMAIN_CLAIM_V1`, `DOMAIN_INVITE_V1`, `DOMAIN_KUDOS_V1` — a
valid curator list signature can never be replayed as a task /
result / claim / invite / kudos signature, and vice versa. The
regression test
`curator::tests::domain_separation_between_curator_and_task`
locks this cross-type guarantee.

Every new signed struct family MUST get its own
`DOMAIN_<NAME>_V1` tag. Never reuse an existing tag for a new
struct — the point of the prefix is to make cross-type replay
structurally impossible.

### Sprint 7.2 — `CuratorListEntry::verify_signature` check order

Layered checks in `crates/nexus-core-rs/src/curator.rs`, executed
in this exact order to make cheap-and-deterministic rejections
fire before expensive ones:

```
1. version == CURATOR_LIST_FORMAT_VERSION   (reject future payloads)
2. entries.len() <= CURATOR_LIST_MAX_ENTRIES (Sprint 7 plan R5 DoS cap)
3. list.curator_pubkey == envelope curator_pubkey (attribution match)
4. Ed25519 signature over canonical_bytes(&list, DOMAIN_CURATOR_LIST_V1)
```

`CURATOR_LIST_MAX_ENTRIES = 256` — conservative cap above which
no realistic early-access curator list lives, and well below any
RAM / gossip amplification threshold. The cap is enforced BOTH
at sign time (`CuratorListEntry::sign` refuses oversized inputs
so a curator cannot accidentally mint an unreadable list) AND at
verify time (a hand-crafted attacker that bypasses sign still
gets rejected).

Revision rollback protection (Sprint 7 plan R6) is NOT in this
module — it lives one layer up in
`nexus_shell_daemon_core::iroh_runtime::CuratorRuntime::process_announcement_bytes`,
which enforces `new.revision > stored.revision` on insert.
Separating the "is this blob cryptographically valid?" check
(curator.rs) from the "is this blob newer than what we have?"
check (iroh_runtime.rs) keeps the crypto layer pure and the
runtime layer testable with fake entries.

### Sprint 7.3 — Attribution-match pattern extended

The attribution-match pattern introduced in Sprint 2 audit fix
`ed2ea76` for `ClaimEntry` (`claim.claimed_by == worker_pubkey`)
is replicated **twice** in Sprint 7:

1. **Crypto level** —
   `CuratorListEntry::verify_signature` checks
   `list.curator_pubkey == envelope.curator_pubkey` before
   reaching the signature check. Catches a forwarder that
   attributes a signed list to a different pubkey than the one
   inside.

2. **Gossip level** —
   `CuratorRuntime::process_announcement_bytes` additionally
   checks that the curator pubkey declared in the gossip
   **announcement envelope** matches the one inside the fetched
   `CuratorListEntry`. Catches an attacker that staples a
   correctly-signed list from curator A onto an announcement
   tagged with attacker B's pubkey — same legitimate signature,
   wrong attribution on the wire.

Both checks share the same rationale: a split-brain condition
between the envelope and the payload is always a bug, never a
feature. Treat them as hard errors and never try to "reconcile"
them.

### Sprint 7.4 — Gossip topic id derivation

Curator list announcements live on a **single global topic** per
Sprint 7 D3:

```rust
// crates/nexus-shell-daemon-core/src/iroh_runtime.rs
pub const CURATOR_TOPIC_SEED: &[u8] = b"nexus-grid/curator/v1";

pub fn curator_topic_id() -> [u8; 32] {
    *blake3::hash(CURATOR_TOPIC_SEED).as_bytes()
}
```

One seed string → one BLAKE3 → 32-byte topic id that every SBFB
daemon joins on boot. Namespaced-per-curator topics
(`"nexus-grid/curator/v1/<pubkey>"`) are deliberately deferred
to Sprint 8+ : a fresh daemon with no curators known yet still
needs to receive announcements from every active curator in the
network, and that requires a global discovery channel.

Test `curator_topic_id_is_deterministic_and_32_bytes` asserts the
const bytes to catch anyone silently renaming the seed.

### Sprint 7.5 — Pkarr reachability probe via `Endpoint::connect`

Sprint 7 plan R1 acknowledged that iroh 0.97 ships no explicit
`Endpoint::lookup(id)` wrapper. Phase D falls back on
`Endpoint::connect(id, iroh_blobs::ALPN)` under a short wall-clock
timeout:

```rust
// crates/nexus-core-rs/src/discovery.rs
pub async fn probe_reachable(
    &self,
    endpoint_id_hex: &str,
    timeout_duration: Duration,
) -> Result<bool> {
    let endpoint_id = EndpointId::from_str(endpoint_id_hex)
        .map_err(|e| NexusError::Discovery(...))?;
    let connect_fut = self.endpoint.connect(endpoint_id, BLOBS_ALPN);
    match timeout(timeout_duration, connect_fut).await {
        Ok(Ok(_conn)) => Ok(true),
        Ok(Err(_dial_err)) => Ok(false),
        Err(_elapsed) => Ok(false),
    }
}
```

Why `BLOBS_ALPN`: every SBFB node accepts the iroh-blobs protocol
at boot (see `crates/nexus-core-rs/src/node.rs::create_node_with_config`),
so a connect probe to this ALPN reaches every live SBFB peer
regardless of role. The returned connection is immediately dropped
— the probe is a pure liveness check, it never opens a bi stream.

Error disambiguation: malformed hex is `Err(NexusError::Discovery)`
(caller bug), dial error / timeout is `Ok(false)` (network
condition). The browse aggregator collapses both buckets into
`BrowseStatus::Unreachable` but the Result surface preserves the
distinction for future tooling (CLI probe, metrics dashboard).

### Sprint 7.6 — singleton registry + hyphen/underscore normalization

`crates/nexus-shell-daemon-core/src/registry.rs`:

- `RunningState` schema v1 (different shape from
  `nexus_coordinator::registry::RunningState` — no `project_name`,
  no `visibility`, + `daemon_version`)
- atomic write via temp sibling + rename (same pattern Sprint 5
  `state_writer.rs` uses for the worker state snapshot)
- pid liveness check through `sysinfo::System::new_all()` +
  `process_name_matches` (R3 mitigation for Windows pid recycling)

`process_name_matches` normalizes both the observed and the
expected process name by lowercasing + `replace('-', '_')` before
a substring check. This bridges the hyphen-vs-underscore gap
between the production binary (`nexus-shell-daemon[.exe]`, clap
`[[bin]] name` convention) and the cargo test binary
(`nexus_shell_daemon[_core]-<hash>[.exe]`, Rust crate name
convention). Without this normalization the singleton enforcement
would silently become a no-op inside `cargo test` because the
test binary's name does not match the production substring.
Regression test:
`registry::tests::process_name_matches_handles_hyphen_underscore_drift`.

### Sprint 7 tech debt (2026-04-11) — Sprint 8 Phase A status update

Status of the 4 pre-confessed items + the 4 new audit-detected
items after Sprint 8 Phase A (commit `d321021`).

#### Pre-confessed items — Sprint 9 Phase E closures

1. **Probe TTL vs real-world pkarr latency** —
   **CLOSED Sprint 9 Phase E (E-1)**. `DEFAULT_PROBE_TIMEOUT`
   is now overridable via `NEXUS_PROBE_TIMEOUT_MS` env var
   (default 2000 ms unchanged). `probe_timeout_from_env()` in
   `browse.rs` reads and parses the env at
   `BrowseAggregator::new()`. Test
   `browse::tests::probe_timeout_env_override_parses_valid_ms`.

2. **Gossip loop backpressure** —
   **CLOSED Sprint 9 Phase E (C-4)**. `CuratorRuntime` now holds
   a `tokio::sync::Semaphore(MAX_INFLIGHT_ANNOUNCEMENTS = 32)`.
   Callers use `process_announcement_bytes_throttled()` which
   acquires a permit before delegating. Test
   `iroh_runtime::tests::gossip_semaphore_limits_inflight_announcements`.

3. **`subscriptions.json` persistence order** —
   **CLOSED Sprint 9 Phase E (D-3)**. `CuratorRuntime::subscribe`
   now does insert-then-persist with rollback: if
   `persist_subscriptions()` fails, `attention.remove(&pubkey)`
   is called so RAM and disk never diverge. Test
   `iroh_runtime::tests::subscribe_persist_first_rollback_on_disk_failure`.

4. **`nexus_core` wheel editable install drift** —
   **CLOSED Sprint 9 Phase A (H-3)**. The Sprint 7 Phase E test
   run showed that the editable install of `nexus-core-py` can
   get wiped by a `uv sync` somewhere in the workflow — Sprint 8
   did NOT add `scripts/setup.sh` nor pin the wheel via
   `pyproject.toml`, so the Sprint 9 Phase 0 audit session had
   to manually rebuild the wheel via
   `unset CONDA_PREFIX && VIRTUAL_ENV=$PWD/.venv maturin develop --release`
   to get `pytest packages/nexus-sdk/tests/test_curator.py` past
   9 `AttributeError: module 'nexus_core' has no attribute 'sign_curator_list'`
   failures — structurally blocked on a fresh checkout, not
   merely inconvenient.

   Sprint 9 Phase A ships `scripts/setup.sh` + `scripts/verify.sh`
   + `.githooks/post-merge` (opt-in via
   `git config core.hooksPath .githooks`) documented in the
   README and `docs/claude/README.md` §4.3. `setup.sh` hashes
   `Cargo.lock` + `crates/nexus-core-rs/src` +
   `crates/nexus-core-py/src` into `.venv/.nexus-core-hash` and
   skips the maturin rebuild when the hash is unchanged, so
   running it twice is a no-op. The post-merge hook fires a
   reminder only when a pull actually touched the Rust core
   sources.
   Audit reference: `.planning/sprint7_audit_findings.md` §H-3
   + `.planning/sprint8_audit_findings.md` §H-FX-2.

#### New items detected by Sprint 7 Phase 0 audit gate — CLOSED Sprint 8 Phase A

These four items were uncovered by the Sprint 7 audit gate
(session fraîche jouant `sprint7_audit_plan.md`, verdict PASS,
findings in `.planning/sprint7_audit_findings.md`). They were
treated as **Phase A hygiene** by Sprint 8, before the gov tab
migration started, so Sprint 9 inherits a cleaner Rust crypto +
runtime layer.

5. **A-4 — `CuratorProjectRef` strings sans length cap** —
   CLOSED Sprint 8 Phase A (commit `d321021`).
   `crates/nexus-core-rs/src/curator.rs::verify_signature` now
   enforces, in step 2 (right after the entries-count cap), that
   every `CuratorProjectRef` has `project_id ≤ 128`,
   `project_name ≤ 128`, `category ≤ 64`, `description ≤ 280`
   characters. The Zod mirror in `web/src/api/daemon.ts` adds the
   same `.max(...)` chain. Test:
   `curator::tests::verify_rejects_oversized_fields`. The total
   list size is now bounded `≤ ~150 KB` instead of unbounded.
   Audit reference: `.planning/sprint7_audit_findings.md` §A-4.

6. **C-2 — `AnnouncementAttributionMismatch` conflate two cases** —
   CLOSED Sprint 8 Phase A (commit `d321021`). The variant
   `CuratorRuntimeError::AnnouncementAttributionMismatch` is split
   into two distinct variants in
   `crates/nexus-shell-daemon-core/src/iroh_runtime.rs:208,224` :
   `NotSubscribed { curator: String }` (benign, expected flood from
   curators we don't follow, logged at `debug!` level) and
   `EnvelopeMismatch { announcement: String, entry: String }` (real
   spoofing attempt, logged at `warn!` with the attacker pubkey).
   Operators can now distinguish a normal traffic burst from a
   genuine attack via log filtering. Test:
   `iroh_runtime::tests::not_subscribed_and_envelope_mismatch_are_distinct`.
   Audit reference: `.planning/sprint7_audit_findings.md` §C-2.

7. **D-1 — `process_name_matches` substring trop large** —
   CLOSED Sprint 8 Phase A (commit `d321021`).
   `crates/nexus-shell-daemon-core/src/registry.rs::process_name_matches`
   no longer accepts arbitrary substrings: it requires either
   exact equality, equality stripped of `.exe`, or
   `<expected>-<hash>` / `<expected>_<hash>` (the cargo test binary
   pattern). A renamed user binary like
   `nexus_shell_daemon_launcher.exe` no longer falsely matches
   the production daemon. Test:
   `registry::tests::process_name_rejects_prefix_extension`.
   Audit reference: `.planning/sprint7_audit_findings.md` §D-1.

8. **G-3 — Daemon HTTP DTOs sans `#[serde(deny_unknown_fields)]`** —
   CLOSED Sprint 8 Phase A (commit `d321021`).
   `crates/nexus-shell-daemon/src/http.rs` now stamps
   `#[serde(deny_unknown_fields)]` on `SubscribeCuratorRequest`
   (line 162), `SubscriptionsResponse` (line 173),
   `CuratorsListResponse` (line 180), and `BrowseListResponse`
   (line 201). A POST body that carries an unknown field is now
   rejected by axum at deserialization time with HTTP 422 instead
   of being silently ignored — defense in depth against future
   schema drift between the shell and the daemon. Test:
   `http::tests::subscribe_rejects_extra_fields`.
   Audit reference: `.planning/sprint7_audit_findings.md` §G-3.

#### A-3 cross-language curator fixture — CLOSED Sprint 8 Phase A

Not strictly Rust-side, but cross-cuts the curator crypto layer.
`packages/nexus-sdk/tests/snapshots/curator_canonical.json` is a
deterministic Ed25519-signed `CuratorListEntry` fixture committed
in commit `d321021`. It is read by both
`packages/nexus-sdk/tests/test_curator.py::test_canonical_fixture_roundtrip`
(Python side, exercises PyO3 + Rust verify) AND
`web/src/api/__tests__/daemon.test.ts` (Vitest side, exercises Zod
parse). Any drift between Rust serde, Python signing, and Zod
schema will fire one of the two tests. This closes the same
"Python signs but Zod never validates the result" hole that
Sprint 6 patched for TabView with `tabview_canonical.json`.
Audit reference: `.planning/sprint7_audit_findings.md` §A-3.

Nothing in these notes invalidates the Sprint 7 architecture
choices. The Sprint 8 Phase F self-report fails fast on 32/32
checks at tip `9339bb6` and every Sprint 7 P2 hygiene closure is
locked by a regression test verified by an explicit fail-fast row
(rows 5-8).

### T19 — `unsubscribe` rollback test missing

Sprint 9 audit gate finding I3-F2. The D-3 persist-first
pattern was implemented for `subscribe` (with a rollback test
`subscribe_persist_first_rollback_on_disk_failure`), but no
matching test exists for `unsubscribe`. If
`persist_subscriptions()` fails after removing from RAM in the
`unsubscribe` path, the I3-F1 fix (landed in `48b332a`) does
the rollback, but the test coverage gap means the rollback
logic is untested.

Fix: add `unsubscribe_persist_failure_rollback` test in
`iroh_runtime.rs` that injects a disk write failure during
unsubscribe and asserts the subscription is still present in
RAM afterwards.

Audit reference: `.planning/sprint9_audit_findings.md` §I3-F2.

### T20 — iroh 0.97 `relay::client::ClientBuilder` has no public hook for a custom cert verifier

Sprint 19 Phase C delivered the `nexus_core_rs::tls_pinning`
primitive (SPKI hash extract + `PinValidator` + hot-reload file
watcher + 17 tests) but **did not** wire a `rustls::client::
danger::ServerCertVerifier` into the iroh relay fallback path,
because iroh 0.97 exposes
`relay::client::ClientBuilder::insecure_skip_cert_verify` only
under `#[cfg(any(test, feature = "test-utils"))]`. There is no
stable public API to inject a custom `ServerCertVerifier`
(context7 `/websites/rs_iroh` verified 2026-04-16).

Consequence : the SBFB transport layer currently validates relay
certs **WebPKI-only** at runtime. The `PinValidator` primitive
is in place and testable in isolation, but a T2/T3/T4/T5
adversary (state-MITM, compromised CA, hostile relay, BGP hijack)
who obtains a WebPKI-valid cert for a relay URL is not yet
blocked by the pin check — because the pin check is not called
during the TLS handshake.

Fix path : two-track approach decided in Phase C design doc §5.1 :

1. **Upstream iroh PR** proposing `ClientBuilder::custom_cert_
   verifier(Arc<dyn ServerCertVerifier>)` as a stable API. Track
   the PR from SBFB's side — once merged in iroh 0.98+, this
   item closes with a one-line `.custom_cert_verifier(Arc::new(
   PinningServerVerifier::new(pin_validator, webpki_fallback)))`
   at the relay builder site (likely in
   `crates/nexus-shell-daemon-core/src/iroh_runtime.rs`).
2. **Forked connect path** as a fallback if upstream merge
   stalls > 2 sprints. Copy ~150 LOC from `magicsock::transports::
   relay::actor::create_relay_builder` and inject a rustls
   `DangerousClientConfigBuilder::with_custom_certificate_verifier`.
   Tech-debt burden : re-sync at every iroh upgrade. Mark with an
   issue tracking link once created.

Tests landed in Phase C do exercise the `PinValidator` against
synthetic cert bytes, so a regression in the primitive itself
surfaces immediately. The gap is strictly in the runtime wire
path, not in the crypto.

Audit reference : `.planning/research/S19_phase_C_tls_cert_
pinning_design.md` §5.1 Option A vs B ; sprint kickoff
`.planning/active/sprint19_kickoff.md` §4 D3 ; commit
`feat(sprint19): Phase C`.

### T21 — TLS bootstrap pins for n0 relays not yet embedded in daemon binary

Sprint 19 Phase C delivered `PinValidator` + `~/.sbfb/relay-pins.
json` loader + `RELAY_PIN_BOOTSTRAP.md` with `openssl s_client`
extraction procedure, but **did not** embed the three known n0
relay SPKI hashes (`relay.iroh.network`, `relay-1.iroh.network`,
`relay-2.iroh.network`) into the daemon binary as a default
pinset. `RELAY_PIN_BOOTSTRAP.md §3.1` explicitly documents this
as a pre-launch choice awaiting maintainer co-sig in Sprint 20+.

Consequence : a fresh install boots with pinset empty, falling
back to WebPKI-only by §Sprint 19.3 point 2 (fail-open + warn).
The TLS pinning protection is effectively opt-in per user.

Fix path : add `crates/nexus-core-rs/src/tls_pinning_bootstrap.
rs` (new) that const-embeds the three SPKI hashes, extract them
at release tag time via the documented openssl pipeline (capture
in a dedicated CI job that produces `relay-pins.bootstrap.json`
as a release artefact), load them at daemon startup if `~/.sbfb/
relay-pins.json` is absent. Co-sig two maintainers before landing
(the bootstrap pins are a trust root — attacker who flips them in
a commit without review MITMs every fresh install until rotation).

Cross-ref : audit finding S19 P2-C2 (`.planning/active/sprint19_
audit_findings.md §Track C`).

### T22 — PoW Hashcash bench wall-clock number not archived

`crates/nexus-core-rs/benches/pow.rs` defines three Criterion
benches (2^12 ~5 ms, 2^18 default target ~100 ms, 2^20 ~400 ms
stress) to verify the difficulty-2^18 choice stays within the
budgeted window. The bench code runs, but **no wall-clock number
is archived in git** (Criterion output is not captured). A
regression caused by a future Rust toolchain flag change, a SHA-
256 implementation swap, or a CPU microarchitecture pessimization
would not surface via CI.

Fix path : add a CI job that runs `cargo bench --bench pow --
benches` and `grep "time:" | awk` to assert `time: 2^18 < 300ms`
on the runner hardware. Alternative : capture the locally-observed
numbers into this PATTERNS section as a dated reference and rely
on manual re-run before release tags. Non-blocking Sprint 20 Phase
A.

Cross-ref : audit finding S19 P2-B1 (`.planning/active/sprint19_
audit_findings.md §Track B`).

### T23 — Docker base image `FROM rust:1.94-slim-bookworm` not pinned to `@sha256:<digest>`

`docker/pkarr-relay/Dockerfile:14` pins `FROM rust:1.94-slim-
bookworm` by version tag but **not** by content digest. The
upstream tag is mutable — Docker Hub can push a new image under
the same tag (CVE patch roll, base OS update), breaking the
strict reproducibility of the daemon build. A rebuild at N days
later produces a different SLSA attestation, a different Trivy
scan output, and potentially different runtime behaviour.

Fix path : capture the current `@sha256:<digest>` via `docker
buildx imagetools inspect rust:1.94-slim-bookworm` at release tag
time, commit the digest to `docker/pkarr-relay/Dockerfile`, renew
quarterly or on security advisory. Optionally extend to every
`FROM` in the repo (currently only `docker/pkarr-relay/Dockerfile`
declares a base). Non-blocking Sprint 20 Phase A.

Cross-ref : audit finding S19 P2-E1 (`.planning/active/sprint19_
audit_findings.md §Track E`).

### T25 — `aws-lc-rs` FIPS 140-3 migration deferred (keystore AEAD)

Sprint 20 Phase A ships the `LocalFileKeyStore` AEAD path on
`aes-gcm = "0.10"` (RustCrypto) instead of the `aws-lc-rs` crate
called out by the kickoff §D1. The swap was forced by the
Windows dev-machine build : `aws-lc-sys` requires NASM to compile
AES-NI intrinsics and a standard Win11 install lacks it. The
algorithm is byte-identical (AES-256-GCM per RFC 5116), so this
is not a security regression — but it postpones the
VALIDATED_BLUEPRINT S17 "FIPS 140-3 track" promise until a
sprint that activates a `fips` build feature.

Fix path : add a `fips` feature to `nexus-core-rs`, switch
`build_aead_key` / `seal` / `open` call sites to `aws_lc_rs::aead
::LessSafeKey` under `#[cfg(feature = "fips")]`, gate the
workspace `aws-lc-sys` NASM requirement behind the same feature,
and document the Win11 NASM install requirement for maintainers
who enable the feature. Non-blocking Sprint 20 Phase A.

Cross-ref : audit finding S20 Phase A review P2-6
(`.planning/active/sprint20_phase_A_review.md`), kickoff
§D1 adjusted (terminology note).

### T26 — Argon2id PIN calibration gap vs target

`crates/nexus-core-rs/benches/keystore.rs::derive_kek_64_mib`
measures **82 ms/attempt** on RTX 5080 + DDR5-6400 with the
production parameters (m=64 MiB, t=3, p=1). The Sprint 20 kickoff
§D2 target was ~3 s/attempt ; we beat the target by 36× because
the Signal SVR calibration that informed §D2 was tuned on ARM
Android devices, not a 2026 x86_64 desktop. Implication : a
6-digit PIN brute force on this CPU costs ~83 s single-thread ;
a dedicated PIN-cracker farm brings it lower. The double layer
(kek1 from PIN + kek2 from OS keyring) still forces the attacker
to have live-user access to the keyring, so the scheme holds —
but the §D2 promise is not strictly honoured.

Fix path : bump m_cost to 128 MiB or t_cost to 6 when we have
deployment telemetry on the slowest supported platform
(Raspberry Pi 4 low-end, per §D2 acknowledge), re-run the bench,
update `§T-keystore-bench-reference`. Older blobs continue to
unlock because the Argon2 params live in the blob header — only
new inits pick up the tighter calibration. Non-blocking Sprint
20 Phase A.

Cross-ref : audit finding S20 Phase A review P2-3.

### T27 — Plaintext `--pin` CLI argument visible in `ps` / shell history

The `sbfb init --pin <p>` / `sbfb unlock --pin <p>` launcher
subcommands take the PIN as a plaintext argv position. Linux
`ps auxe` + bash `HISTFILE` both capture it ; Windows Task
Manager "Command line" column exposes it until the launcher
exits. Acceptable for Phase A dev / smoke-test flows — the PIN
is also passed verbatim by a human during the session — but not
for day-to-day operator use.

Fix path : Phase B introduces an interactive `rpassword`-style
prompt (no echo, reads /dev/tty on Unix, ReadConsole on
Windows) that replaces the `--pin` flag. The flag can stay for
batch / CI smoke-tests behind a `--pin-fd <fd>` alternative that
reads from a file descriptor instead of argv. Non-blocking
Sprint 20 Phase A.

Cross-ref : audit finding S20 Phase A review P2-5.

---

## Sprint 18 canonical patterns (2026-04-15)

### Sprint 18.1 — Persistent maintainer identity key vs ephemeral network identity key

Sprint 18 Phase E2 added a warrant canary signed with an Ed25519
key loaded from `<sbfb_home>/canary-key.key` via
`KeyPair::load_or_generate`. That key is **intentionally distinct**
from the shell daemon's `create_node()` iroh identity, which mints
a fresh keypair on every boot.

The split matters because the two keys answer two different
questions:

| Key | Persists across reboots? | Publishes pubkey externally? | Rotation cost |
|---|---|---|---|
| Daemon `create_node()` identity | No (ephemeral per boot) | Yes (node_id on gossip / docs / blobs) | Free — next reboot mints a new one |
| Maintainer `canary-key.key` | Yes | Yes (pubkey embedded in CANARY.txt + gossip) | High — verifiers who pinned the old pubkey must be notified out-of-band |

**Rule** : any signing surface that produces artefacts intended
to be verified **months or years after publication** must use a
dedicated persistent key file under `<sbfb_home>/`, not the
daemon's `create_node()` identity. This includes:

- Warrant canaries (Sprint 18.E2)
- SLSA provenance attestations on releases (Sprint 14, coord-side)
- Future release-signing keys (Sprint 19+ release attestations)

Any signing surface that produces artefacts **consumed in-band by
a currently-running daemon** may reuse the iroh identity — it only
needs to be stable for the duration of the daemon process. This
includes:

- Task/Result/Claim/Invite signatures (Sprint 2, all daemon-owned)
- Curator list entries (Sprint 7, re-signed on every rebroadcast)
- Gossip-level authenticated messages

**Concrete check**: when adding a new `DOMAIN_<NAME>_V1` tag to
`canonical.rs`, ask "could a verifier 6 months from now refuse to
accept a valid signature because the key rotated silently?" If yes,
use a persistent key file. If no, reuse the iroh identity.

Callers must **never** read `<sbfb_home>/auth_token` as a signing
key — that file is the loopback HTTP bearer (Sprint 16 Phase A),
not an Ed25519 keypair. Confusing the two was a risk the Sprint 18
Phase E2 auditor explicitly flagged (P2 carry-over).

Audit reference: `.planning/active/sprint18_phase_E2_review.md` §Q3.

---

## Sprint 19 canonical patterns (2026-04-16)

### Sprint 19.1 — Primitive / wire / enforcement separation

Sprint 19 Phase A landed the DHT quorum **runtime wire** (browse
aggregator canary) on top of the S18 DHT quorum **primitive** that
was testable in isolation. Sprint 19 Phase B repeats the pattern
for PoW :

| Layer | Where | What it owns |
|---|---|---|
| Primitive | `nexus-core-rs::pow` | `HashcashChallenge`, `HashcashProof`, `solve`, `verify_at` — pure crypto, no I/O, no async |
| Wire envelope | `nexus-core-rs::pow_gossip` | `PowEnvelope::encode/decode`, `PowSolveCache`, `PowVerifyCache` — stateless byte codec + session caches |
| Policy | `nexus-core-rs::relay_pow_policy` | `RelayPowPolicy` + TOML loader from `~/.sbfb/relay_pow_policy.toml` |
| Enforcement | `nexus-shell-daemon-core` + coord-side | Calls `broadcast_with_pow` / `verify_envelope` on its gossip paths (Sprint 20+ wiring) |

**Why the separation matters** : Phase B ships the primitive +
envelope + policy + caches fully tested (32 unit tests added to
`nexus-core-rs` — 14 `pow::`, 6 `relay_pow_policy::`, 12
`pow_gossip::`). Flipping enforcement on for a specific gossip
topic (curator list, task dispatch, etc.) is a one-line change
that future sprints can roll per-topic without touching the core
crypto. A regression in enforcement flipping does not regress the
primitive's security.

**Concrete check** : when adding a new defence-in-depth
capability, structure it as `primitive` + `wire` + `enforcement`
from day 1 even if you are shipping all three in the same sprint.
The audit-gate pattern rewards explicitly-separated layers
(primitive tests pass means the crypto is sound even if the wire
is rewritten).

Audit reference : `.planning/active/sprint19_phase_A_review.md`
(DHT quorum design canary), `.planning/active/sprint19_kickoff.md`
§4 D1 (Phase B scope).

### Sprint 19.2 — Forward-compat in PoW proof format

The `HashcashChallenge` wire struct includes `publisher_pubkey:
[u8; 32]` explicitly so that when the Sprint 26+ post-quantum
cutover lands (ML-DSA-65 hybrid), the migration is a format
version bump (`v1 → v2`) with a tolerant decoder, not a full
redesign.

The `publisher_pubkey` field also makes every proof
non-replayable cross-identity : a botnet that sniffs one solution
from the wire cannot reuse it under a second pubkey without
re-solving. This is the invariant the
`different_publishers_yield_different_solutions` test pins.

**Concrete check** : when designing a new wire format that will
carry a pubkey, embed it in the canonical-bytes domain separately
from any signature field. Deriving it from the signature (like
ed25519 recovery-id would do) or relying on the gossip envelope's
sender field couples the security to the transport and breaks
under HyParView peer rotation.

### Sprint 19.3 — TLS cert pinning : SPKI hash + hot-reload + defense-in-depth with WebPKI

Sprint 19 Phase C landed `nexus_core_rs::tls_pinning` with four
design commitments worth preserving in future sprints :

1. **SPKI hash, not full-cert pin**. Pin the SHA-256 of the
   DER-encoded `SubjectPublicKeyInfo` (RFC 7469 §2.4), not the
   cert bytes themselves. Lets operators rotate through Let's
   Encrypt (90 → 45 day renewals) without rewriting every
   client's pinset, as long as `--reuse-key` is set.
2. **Fail-open when empty, fail-closed when populated**. An
   absent or empty `~/.sbfb/relay-pins.json` triggers a loud
   warn and falls back to WebPKI-only — pre-launch convivialité.
   Once the user populates the file (opt-in), every relay URL
   present in the file must match its pin ; a URL that is
   **not** in the file (but other URLs are) fails closed with
   `PinError::NoPin`. Opt-in-then-strict posture.

   **Plan-vs-code deviation (documented here per audit finding
   S19 P2-C1)** : `sprint19_plan.md §6.4` originally called for
   fail-closed on empty pinset (« tous relays refusent »). The
   delivered code chose fail-open + warn because no bootstrap pins
   are embedded in the daemon binary this sprint (cf. T21 below),
   so fail-closed would leave a zero-config install unusable. The
   opt-in-then-strict posture lands the same invariant that plan
   wanted (pinning is strict **when configured**) without the UX
   trap. When bootstrap pins land via T21, revisit : the posture
   could flip to fail-closed-by-default, since zero-config would
   then already ship the pinset.
3. **Hot-reload via `notify` watcher**. Same pattern as
   `ConsentWatcher` (S16 Phase C) and `TokenRotator` (S18 Phase
   D file-watcher) : watch the **parent directory** (not the
   file itself) to support write+rename atomic saves, debounce
   50 ms, swap the in-memory `Arc<RwLock<RelayPinsFile>>` in a
   single operation. A reload that fails to parse **keeps the
   previous pinset** rather than fail-opening the pinset empty.
4. **Defense-in-depth, not replacement**. When the Phase C
   runtime wire eventually lands (T20 above), it must call
   `WebPkiServerVerifier::verify_server_cert` first, then the
   pin check — never skip WebPKI. A compromised CA that issues
   a cert for a non-pinned URL is still caught by WebPKI
   validity ; the pin is an **additional** check on top, not a
   substitute.

**Anti-pattern : repeat HPKP (RFC 7469)**. HPKP died because it
was a web-scale generic header with a 60-day TTL and no safe
revocation mechanism. SBFB's pinning operates in a completely
different context : **local file, user-editable, hot-reload in
50 ms, opt-in**. The primitive cryptographic construct (SPKI
hash) is valid ; the operational model is what made HPKP
dangerous.

**Concrete check** : before adding any new pinning-style
defense, enumerate :

- Where is the state stored? (file, env, db, remote?)
- How does an authorized user recover from a misconfig? (edit
  file in 30 s? file a support ticket? wait 60 days?)
- What happens during a planned key rotation? (overlap window?
  out-of-band announcement? per-release bootstrap?)

If any of those answers is "no mechanism", the pinning is not
safe to ship. Phase C's answer to all three is `~/.sbfb/
relay-pins.json` + hot-reload + backup pin RFC 7469 §4.3 +
`docs/release/RELAY_PIN_BOOTSTRAP.md §4` procedure.

Audit reference : `.planning/research/S19_phase_C_tls_cert_
pinning_design.md` §3 alternatives + §3.6 HPKP lessons + §5.4
fail-open vs fail-closed matrix.

---

## Sprint 20 canonical patterns (2026-04-16)

### Sprint 20.1 Encryption at rest — double layer (PIN + OS keyring)

Sprint 20 Phase A (`crates/nexus-core-rs/src/keystore.rs`) wraps
the daemon's Ed25519 identity keypair at rest with two independent
secrets so an attacker who recovers only one still faces a full
brute-force wall against the other.

```
PIN ── Argon2id(salt, m=64 MiB, t=3, p=1) ──► kek1  (32 bytes)
                                              │
OS keyring (platform-native credential store) ─► kek2  (32 bytes)
                                              │
                        BLAKE3(DOMAIN_KEYSTORE_V1 || kek1 || kek2)
                                              │
                                              ▼
              final_kek ── AES-256-GCM ── wrap(Ed25519 privée)
                                              │
                                              ▼
                 blob ~/.sbfb/shell-daemon/keyring/identity.enc
```

Canonical threat-model coverage :

| Adversary reads | Needs to decrypt | Cost |
|---|---|---|
| Blob file only | PIN + keyring | Argon2id wall + live user |
| OS keyring only | PIN + blob file | Argon2id wall + disk access |
| Blob file + keyring | PIN | Argon2id wall (~80 ms/attempt on modern CPUs) |
| Blob + keyring + PIN | — | decrypt |

Closes **T3 DPAPI user-scope gap** (Sygnia 2024 "DPAPI downfall"
+ SpecterOps 2024-2026): a same-user malicious process that dumps
DPAPI master keys from LSASS via Mimikatz `/unprotect` still
cannot unlock the blob — the PIN never enters `lsass.exe`.

Deviations from the Sprint 20 kickoff §D1 baseline, noted in the
Phase A commit body :

- **AEAD backend** : `aes-gcm` (RustCrypto) replaces `aws-lc-rs`
  from the D1 draft because `aws-lc-sys` requires NASM on Windows
  to build and that dependency blocks dev-machine setup. The
  algorithm (AES-256-GCM) is byte-identical ; the migration to
  `aws-lc-rs` behind a `fips` build feature is a one-file swap
  tracked for the sprint that actually enables FIPS 140-3. Pre-
  launch protocol policy (CLAUDE.md §Pre-launch) makes FIPS
  optional until the `v1.0` tag.
- **Blob filename** : the kickoff §D1 sketch said
  `<node_id>.enc` ; the implementation uses the fixed
  `identity.enc`. The node_id is only available AFTER the blob is
  decrypted, so prefixing the filename with it would require a
  sidecar `identity.pub` file. Phase A keeps the layout minimal ;
  Phase B will add `identity_duress.enc` as a second fixed slot.

### Sprint 20.2 Key-handling discipline

The daemon never holds `kek1` or `kek2` in plaintext beyond the
AEAD seal/open window. Every sensitive 32-byte buffer lives
inside `secrecy::SecretBox<[u8; 32]>` (heap-allocated, zeroed on
drop via `zeroize`). `SecretKeyBytes` is the only
`SecretBox`-compatible newtype around `[u8; SECRET_KEY_BYTES]` ;
it derives `Zeroize` so the secrecy box can zero the heap copy
without the caller thinking about it.

The launcher hands the decrypted secret to the daemon child via
the `SBFB_IDENTITY_SECRET_HEX` env var. Both sides remove the var
from their process environment immediately after reading it
(`std::env::remove_var` on the daemon side, inline `hex_str.
zeroize()` on the launcher side). This closes the most obvious
same-user leak path at the cost of leaving the hex bytes briefly
visible in `/proc/self/environ` during the spawn window.
Tightening to a Unix domain socket / Windows Named Pipe handoff
is tracked as T24 tech debt for a future sprint.

### Sprint 20.3 PoW gossip wire runtime

Sprint 19 Phase B landed the Hashcash primitive + envelope +
publisher/subscriber caches (`crates/nexus-core-rs/src/pow.rs` +
`pow_gossip.rs` + `relay_pow_policy.rs`). Sprint 20 Phase C wires
them into the daemon's live gossip path so every outbound payload
is PoW-gated and every inbound payload is dropped if it fails the
topic's difficulty bar.

**Two sides wired at one entry point each :**

- **Publish** — `crates/nexus-shell-daemon/src/http.rs ::
  publish_project` now calls `wrap_payload_with_pow(&state,
  &payload)` (same file). The helper reads the shared policy
  (`Arc<RwLock<RelayPowPolicy>>`), asks the
  `PowSolveCache` for a live proof keyed by the curator topic id,
  and `PowEnvelope::encode`s `[u32 BE proof_len][proof bytes]
  [payload]` before `TopicSender::broadcast`.
- **Receive** — `crates/nexus-shell-daemon/src/runtime.rs ::
  spawn_gossip_subscribe_task` unwraps every
  `GossipEvent::Message { content, .. }` through
  `PowVerifyCache::verify_envelope` **before** the
  `publish::is_project_announcement` / `handle_announcement`
  dispatch. A failed verify warns + drops — neither the browse
  aggregator nor the curator runtime ever sees non-PoW noise.

**Hot-reload :** the shared policy lives behind
`nexus_shell_daemon_core::pow_policy_loader::PowPolicyWatcher`, a
pattern-copy of `TokenRotatorWatcher` (Sprint 18 D-1 audit fix,
cf. `auth.rs:699-830`) : watch the parent dir (not the file), 50 ms
debounce, fail-safe on malformed reload (keep last known-good
policy). A missing `~/.sbfb/relay_pow_policy.toml` boots the gate
on the S19 `DEFAULT_POW_POLICY` (2^18 leading zero bits, no
overrides). The watcher is kept alive on `DaemonRuntime.
pow_policy_watcher` for the whole process.

**What the two call-sites grep guarantees :** the Phase F checklist
audits `grep -r "gossip\.subscribe\|\.join_topic" crates/
nexus-shell-daemon*/src/` — the only call-site inside the
run-time is `runtime.rs::spawn_gossip_subscribe_task::join_topic`,
and its receive loop is wrapped by `verify_envelope`. The `main.rs`
canary subcommand broadcasts on a one-shot topic and is
intentionally out of the S20 C scope (scheduler + gossip wire
livered by S20 Phase E).

**Extension points :**

- **S21 rate-limit per-(consumer, worker, model)** depends on PoW
  being active (this wire). After Phase C merges, the rate-limit
  lookup key can include the Hashcash publisher pubkey without
  extra auth.
- **S22 kudos-weighted admission** : the receive verify predicate
  becomes `pow_ok && kudos_score >= threshold`. The verify lives
  in `PowVerifyCache::verify_envelope` — a single place to edit,
  same cache key `(pubkey, topic)`.

### §T-keystore-bench-reference

Reference numbers for `cargo bench -p nexus-core-rs --bench
keystore` on the calibration hardware (RTX 5080 + DDR5-6400 +
Windows 11, 2026-04-16) :

| Bench | Median | Target |
|---|---|---|
| `keystore_prod/derive_kek_64_mib` | **82 ms** | < 5 s |
| `keystore_aead/unlock_fast_params` | (fast params — informational only) | — |
| `keystore_unlock_total/unlock_prod_params_no_keyring` | (prod params) | < 6 s |

The production Argon2id derive (64 MiB / t=3 / p=1) lands at
~82 ms on this CPU, well under the 5 s critère. It is **much**
faster than the Sprint 20 kickoff §D2 target of ~3 s/attempt
because RustCrypto's pure-Rust Argon2 implementation on a 2026
desktop + fast DDR5 beats the Signal Secure Value Recovery
reference calibration (which was tuned on ARM Android devices).
Implication for the threat model : at ~82 ms/attempt a 6-digit
PIN brute force costs ~83 seconds on a single thread, and a PIN
cracker farm brings that down further. The calibration is still
acceptable for Phase A (keyring layer + audit log raise the
effective cost), but bumping `m_cost` to 128 MiB or `t_cost` to
6 is a candidate for Sprint F consolidation or a follow-up
sprint once we have deployment telemetry. Record any such bump
here AND in the blob `flags` byte so older blobs keep unlocking.

Run the numbers yourself on your dev machine :

```bash
cargo bench -p nexus-core-rs --bench keystore -- derive_kek_64_mib
```

---

## §P30 — Sprint 20 Phase D : structured output dual-backend

> ⚠️ **STRUCTURED OUTPUT GRAMMAR IS NOT A DEFENSE AGAINST PROMPT
> INJECTION.** The grammar enforces *format* (JSON Schema), not
> *content*. A successful prompt injection against the user query
> can still produce schema-valid responses with malicious payload
> (e.g. `TaskResponse { content: "<exfiltrated secret>",
> tool_calls: [...] }` — valid shape, bad content). Defense against
> prompt injection belongs to Sprint 22 (tool-calling sandbox +
> wasmtime jail) and Sprint 21 (client-side redaction SDK).
> Grammar enforcement + signature chain integrity is *one layer*
> of defense, not a replacement for prompt-injection hardening.
> Cf. `docs/security/HARDENING_ROADMAP.md audited_findings
> 2026-04-18 "grammar ≠ prompt injection defense"`.

### Architecture

`nexus-worker-core::llm` ships two implementations of the
[`LlmBackend`] trait :

| Backend | Feature flag | Grammar engine | Typical perf |
|---|---|---|---|
| `OllamaBackend` | always on | Ollama's internal llama.cpp GBNF | ~200 µs/token |
| `LlamaCppBackend` | `llm_llama_cpp` | `llguidance` crate, Rust-side bridge | ~50 µs/token |

Worker config `worker.toml` selects the backend :

```toml
[llm]
backend = "llama_cpp"  # or "ollama"

[llm.ollama]
endpoint = "http://localhost:11434"
timeout_secs = 300

[llm.llama_cpp]
model_path = "~/.nexus-grid/models/qwen2.5-7b-instruct-q4_k_m.gguf"
n_ctx = 4096
n_gpu_layers = -1   # -1 = all, 0 = CPU-only
n_threads = 0       # 0 = let llama.cpp pick
```

### Schema source-of-truth

`nexus_core_rs::TaskResponse` is derived with
`#[derive(JsonSchema)]` — the JSON Schema is **generated from the
Rust struct**, not hand-written. Both backends consume the same
`serde_json::Value`. A snapshot lives at
`crates/nexus-core-rs/src/schemas/task_response.schema.json` as
a canary against silent drift ; regenerate with :

```bash
UPDATE_SNAPSHOTS=1 cargo test -p nexus-core-rs \
    schemas::task_response::tests::schema_snapshot_matches_struct
```

### Defense-in-depth

Both backends run a **defensive validator** on the decoded text
before returning :

```rust
let parsed: TaskResponse = serde_json::from_str(&text)?;
parsed.validate_identity()?;
```

**Sprint 20 enforcement coverage** :

| Backend | Sample-time enforcement | Post-decode validator |
|---|---|---|
| `OllamaBackend` | ✅ via `format` param (Ollama v0.5+ GBNF inside its llama.cpp) | ✅ |
| `LlamaCppBackend` | partial — matcher state advances via `compute_mask` + `ff_tokens`, but the logit-bias frame push is S21+ (P3-D3 carry). The picked token is checked post-hoc via `consume_token` → `SchemaViolation` on reject. | ✅ |

Until the `llama_cpp` logit-bias wire lands, the post-decode
validator is the load-bearing layer for `LlamaCppBackend` schema
correctness. Post-hoc `consume_token` raises `SchemaViolation`
when the sampler picked a token outside the grammar — the worker
refuses to sign. Between the two, the signature chain never sees
malformed JSON.

This catches :
- A broken Ollama daemon that ignores its own grammar (has
  happened on pre-0.5 versions that silently dropped the
  `format` param).
- A `LlamaCppBackend` misconfiguration where the matcher
  deterministic-fastforwards but somehow outputs off-schema
  bytes (grammar bugs in llguidance compiled against an old
  tokenizer).
- A wrong `version` or `domain` tag that would otherwise slip
  past the JSON Schema check (the schema validates structure,
  not semantic identity).

Refusing to sign an unparsable response is the signature-layer
analogue of the "fail closed" principle applied to every
certificate pin in S19 Phase C.

### Why Rust-side llguidance rather than `-DLLAMA_LLGUIDANCE=ON`

The llama.cpp build flag links `libllguidance` into the C-side
sampler. We bridge the crate in Rust instead (`llguidance::
Matcher` shares state with the `llama_cpp_2::sampling::LlamaSampler`
chain ; logit-bias push for full pre-sample enforcement is carried
to Sprint 21+ as P3-D3) so :

- Operators do **not** need a custom llama.cpp build — cargo
  pulls the crate transitively.
- The matcher state is owned by Rust code we can unit-test
  (`build_matcher_accepts_task_response_schema` in
  `llm::llama_cpp::tests`).
- Future tool-call interception (S22 sandbox) can hook on
  `matcher.consume_token` to observe grammar transitions
  without touching C code.

### Build chain caveats (operator)

- `llama-cpp-sys-2` uses `bindgen` → requires **LLVM / libclang**
  on the build host.
  - Windows : install LLVM + add `LIBCLANG_PATH` env var.
  - Linux : `sudo apt-get install libclang-dev`.
  - macOS : `brew install llvm` or use Xcode's bundled clang.
- CUDA cascade feature `llm_llama_cpp_cuda` requires CUDA
  toolkit 12.6+.
- A fresh `cargo build` without the feature **never touches
  cmake / bindgen** — this is why the default stays
  feature-off.

See `docs/shell/PATTERNS.md §P30` for the full operator runbook.

---

## §P31 — Sprint 20 Phase E : warrant canary federation foundations

The S20 Phase E pivot (G8 codification 2026-04-18, cf.
`.planning/active/sprint20_phase_E_pivot_proposal.md` + commit
`bd16e64` plan update) replaces the original §8.1 item 1
"auto-publish scheduler" — rejected on the G8 S2 finding that
S18 E2 commit `04c9621` already documents : a key Ed25519
accessible to any scheduler / cron / GHA workflow ≡ key
compromise under gag order ≡ dead-man-switch broken by design.

The pivot ships **federation primitives** (Niveau 1 scaffolding)
without any new way to produce signatures automatically. Every
canary signature still requires a human typing `sbfb canary
publish` (Niveau 0) or `K` humans cooperating in a synchronous
FROST round-1/round-2/aggregate session (Niveau 1+).

### Core abstraction : `CanarySigner`

```rust
pub trait CanarySigner: Send + Sync {
    fn pubkey(&self) -> [u8; PUBLIC_KEY_LENGTH];
    fn sign(&self, message: &[u8]) -> [u8; SIGNATURE_BYTES];
}
```

Two implementations ship in S20 :

- **`Ed25519CanarySigner`** (E.1) — wraps `nexus_core_rs::KeyPair`,
  produces a standalone Ed25519 RFC 8032 signature. The default
  for solo maintainers and for any Niveau 0 deploy. Byte-identical
  to the S18 E2 baseline behaviour (the trait migration is a pure
  refactor — same wire output, swappable callsite).
- **`FrostCanarySigner`** (E.2) — wraps a `K-of-N` FROST trusted-dealer
  setup via the `frost-ed25519` v2.x crate (RFC 9591 jan 2025, ZF
  reference impl, Trail of Bits 2023 audit). The aggregated
  signature is **byte-identical** to a standalone Ed25519 signature
  by RFC 8032 spec (FROST is a Schnorr-flavoured threshold scheme
  whose output verifies against the standard Ed25519 verifier
  unchanged). This is the wire-format invariant the entire pivot
  hinges on : `CanarySigned v1` does not bump, the `verify_canary`
  path does not change, only the *production* of the signature
  becomes K-party.

### Why a trait

Decoupling at the trait surface lets the build+sign+verify
pipeline (`build_canary` / `verify_canary` / `format_canary_txt` /
`parse_canary_txt`) stay algorithm-agnostic. Future Niveau 1+
backends (PQC ML-DSA, hardware-keystore-backed, TEE-attested
sign-inside-enclave) plug in by implementing the trait without
touching any wire code.

The trait is deliberately minimal :
- No `Result` return on `sign` — implementations construct from
  validated inputs (e.g. `FrostCanarySigner::trusted_dealer` is the
  fallible constructor ; the resulting signer cannot fail to sign
  with self-dealt shares). Keeps callers simple.
- No async — canary signing is a once-per-month operation, the
  perf budget is "done in seconds, not microseconds", and
  threshold ceremonies are synchronous interactive flows
  orchestrated outside the daemon process.

### Default K=2/N=2 minimum, not K=1/N=1

A common mistake is to assume `K=1/N=1` "FROST" exists as a way
to express the baseline single-key case via the trait. RFC 9591
§6.1 explicitly requires `K >= 2` (a "K=1 threshold" is
degenerate ; `frost-ed25519` v2.x rejects it at construct time).
The minimum legitimate FROST configuration is therefore **K=2/N=2**
(both shares cooperate, no redundancy). For the K=1-equivalent,
use `Ed25519CanarySigner` directly — the trait abstraction makes
that swap a single line at the callsite.

### Duress ack channel (E.4)

`crates/nexus-shell-daemon-core/src/canary/duress_ack.rs` adds a
**second** signed heartbeat stream on a **distinct gossip topic**
(`nexus-grid/canary-duress-ack/v1`) and **distinct domain tag**
(`DOMAIN_DURESS_ACK_V1 = b"nexus-duress-ack-v1"`). The same
maintainer pubkey signs both streams ; the registry tracks them
on independent freshness ladders.

Why a separate topic + domain : a duress ack signature MUST NOT
be replayable as a canary signature, and a relay / verifier
choosing to subscribe to one stream MUST NOT be forced to ingest
the other (the ack stream is daily-cadence, the canary stream is
monthly — different bandwidth profiles).

### `AttestationProvider` orthogonal axis (E.5)

`canary/attestation.rs` introduces a second trait — `Attestation
Provider` with a `NoopAttestation` baseline — that is
**deliberately disjoint** from `CanarySigner`. The two axes
(signing trust + process trust) compose freely : a maintainer
can use Ed25519+TDX, FROST+Noop, FROST+SNP, etc. The composition
matrix is documented in `docs/security/WARRANT_CANARY_HARDENING.md
§5`.

### Federated registry (E.3, Python-side)

`packages/nexus-coordinator/src/nexus_coordinator/canary_registry.py`
aggregates observed canaries + duress acks per maintainer pubkey
and persists to `<root>/canary-registry.json`. The
`GET /api/canary/network-health` endpoint surfaces a fleet
freshness snapshot the React shell can render directly. The
registry is **observational only** — it never re-signs, never
makes a trust decision. The wire JSON it persists is the same
shape the daemon-side `Canary` struct serializes to.

### Tests guarantee

The crucial wire-format invariant is verified end-to-end :
`canary::frost::tests::frost_sig_verifiable_by_standard_ed25519_verifier`
asserts that a `FrostCanarySigner::sign` output verifies against
`nexus_core_rs::crypto::verify` (a plain
`ed25519_dalek::VerifyingKey::verify` under the hood) — i.e. the
verify path does not need to know whether the signature came from
a single key or a K-party threshold. Combined with the
`frost_dkg_k2_n3_produces_valid_ed25519_sig` end-to-end test that
runs `build_canary(date, headline, &frost_signer)` followed by
`verify_canary(&canary)`, we have a hard guarantee the
`CanarySigned v1` wire format survives the trait migration
byte-for-byte.

See `docs/security/WARRANT_CANARY_HARDENING.md` for the full
4-layer strategy (L0 single-key → L1 federation primitives → L1
enforcement S25-30 → L2 cross-project federation post-v1.0) and
the FROST DKG ceremony procedure for cross-juridiction
recruitment.

---

## §P32 — Sprint 20 Phase E.6 (post-G8) : transport probe = observability-only

The S20 plan §8.1 E.6 originally called for a manual switch to
`RelayMode::Custom` with a `relay_wss_only = true` flag if the
boot-time UDP QUIC probe failed 3 times. The G8 phase pre-flight
S1 scan ([`sprint20_phase_E_preflight.md`]) discovered :

- iroh **0.91** removed the raw-TCP option from the relay client
  (cf. blog post `iroh-0-91-0-the-last-relay-break`). The relay
  speaks **WebSockets only** since 0.91, and 0.97 inherits.
- WebSockets ⇒ TCP 443 by default. There is no `relay_wss_only`
  flag because there is no other mode to opt out of.
- `RelayMode::Custom(RelayMap)` exists, but only to point the
  endpoint at a different relay set ; the transport (WSS) is
  fixed.
- The fall-back from a failed UDP QUIC hole-punch to a relay-WSS
  path is **automatic inside iroh** and requires no client-side
  configuration to activate.

The pattern lesson : **when the runtime already does the right
thing, the integration code is a metric, not a control loop.**
`crates/nexus-shell-daemon-core/src/transport_probe.rs` is
deliberately observability-only :

```rust
pub async fn probe_with_retries(
    prober: &dyn TransportProber,
    cfg: ProbeConfig,
) -> TransportProbeOutcome {
    for attempt in 1..=cfg.max_attempts {
        if prober.probe_once(cfg.attempt_timeout).await {
            return TransportProbeOutcome::Direct;
        }
    }
    warn!(
        target: "nexus_shell_daemon_core::transport_probe",
        transport_degraded_mode = true,
        ...
    );
    TransportProbeOutcome::Degraded
}
```

It never touches `iroh::Endpoint`, never calls `endpoint
.set_relay_mode()`, never constructs a `RelayMap`. It just
attempts a dial up to `max_attempts` times and emits a structured
`tracing::warn!` with a `transport_degraded_mode = true` field
on failure — the field is what ops dashboards / log shippers
filter on to surface "this daemon is on the relay-WSS fallback
path" without needing to count attempts themselves.

Future Sprint S+1 follow-up (if/when needed) would wire the
`IrohQuicProber` impl that runs `endpoint.connect(addr,
ALPN).await.is_ok()` against a configured peer ; for S20 only the
trait + retry loop + scripted-mock test surface is shipped, which
is enough to validate the metric format end-to-end.

---

## §P33 — Sprint 21 Phase A + Sprint 22 Phase A : rate-limit GCRA multi-tier (R1 worker-engine gate + hot-reload wire-up)

Phase A S21 delivers a rate-limit primitive per-(consumer, worker,
model) in `crates/nexus-worker-core/src/rate_limit.rs`, consumed
by the worker engine pre-task-execution. Sprint 22 Phase A wires
the primitive into `runtime.rs` pre-`ClaimEntry` gate and adds
`swap_policy` for hot-reload via Arc swap.

The primitive wraps `governor::DefaultKeyedRateLimiter<RateKey>`
(GCRA algorithm) behind a `RwLock<RateLimiterState>` for atomic
hot-reload rotations (readers never block other readers, writer
only blocks for Arc swap duration).

```rust
pub struct RateKey {
    pub consumer: ConsumerId,
    pub worker: WorkerId,
    pub model: ModelId,
}

struct RateLimiterState {
    default: Arc<DefaultKeyedRateLimiter<RateKey>>,
    overrides: Arc<HashMap<ConsumerId, Arc<DefaultKeyedRateLimiter<RateKey>>>>,
}

pub struct RateLimiter {
    state: RwLock<RateLimiterState>,
    policy: Arc<RwLock<RateLimitPolicy>>,
}

impl RateLimiter {
    pub fn from_policy(policy: Arc<RwLock<RateLimitPolicy>>) -> Result<Self, RateLimitError>;
    pub fn swap_policy(&self, new: RateLimitPolicy) -> Result<(), RateLimitError>;
    pub fn check(&self, key: &RateKey) -> Result<(), RateLimitError>;
}
```

Policy is loaded from `~/.sbfb/rate_limit_policy.toml` via
`RateLimitPolicyWatcher` (`rate_limit_policy_loader.rs`) with the
same pattern as `PowPolicyWatcher` (S20 §P29) + `TokenRotator`
(S18 D-1) + `ConsentWatcher` (S16) : parent-dir `notify` watch,
50 ms debounce, malformed-reload guard, file-deletion guard,
Arc<RwLock<RateLimitPolicy>> snapshot shared across engine tasks.

### R1 scope-cut rationale

The plan §4.1 (kickoff §D1 literal) originally called for a
`tower-governor` axum middleware on `/task/submit` in the Rust
shell-daemon. A mid-phase drift detection showed `/task/submit`
lives in Python FastAPI (`packages/nexus-coordinator/src/nexus_
coordinator/api/tasks.py::POST /tasks/submit` since Sprint 4
Phase A) — `tower-governor` cannot middleware FastAPI.

User arbitrated R1 worker-engine gate 2026-04-19. Rationale :
- `HARDENING_ROADMAP §3 S21` threats `C-ModelExtract` + `C-DosFlood`
  are defended at the worker level (protect inference + GPU), not
  HTTP coord (where the real threat is a botnet across workers,
  not per-worker rate).
- D1 core `governor 0.10.2 GCRA + DashMap keyed + policy hot-
  reload` preserved verbatim.
- HTTP middleware Python coord-side deferred S22+ (slowapi or
  equivalent dedicated API security sprint).

The pattern lesson : **when a design assumes a language or runtime
substrate that does not exist at the target path, G8 pre-flight
S1+S2+S3+S4 catches it** — but the mid-phase grep (pre first
`Edit`/`Write`) is the last-line-of-defense checklist. Always
grep for the target routes/endpoints before starting to wire
middleware.

### Policy TOML schema

```toml
[default]
per_min = 60             # u32 required, rejects zero at boot
burst_multiplier = 2.0   # f64 optional, floors to per_min if < 1.0

[[overrides.consumer]]
pubkey_hex = "0xabc..."  # 64-char Ed25519 hex
per_min = 500
burst_multiplier = 3.0
```

Tests cover : saturation rejects over budget, per-tuple
independence on all 3 axes (consumer / worker / model),
`retain_recent` idempotent housekeeping, override whitelist lifts
budget, invalid quota rejected at boot, TOML serde round-trip,
hot-reload live swap, malformed reload keeps previous, deletion
keeps previous. 16 Rust tests total.

### Engine integration (outline, consumed S21 Phase B+)

```rust
// Worker engine, pre-task-execution admission check
let key = RateKey::new(
    task.consumer_pubkey_hex.clone(),
    worker_id.clone(),
    task.model_id.clone(),
);
match rate_limiter.check(&key) {
    Ok(()) => admit_and_execute(task).await,
    Err(RateLimitError::Saturated { .. }) => {
        // Defer : task goes back to the coordinator with a
        // "retry later" signal, surfaced as a task status
        // update via the existing result channel.
        defer_with_retry(task).await
    }
    Err(RateLimitError::InvalidQuota(_)) => unreachable!(
        "loader rejected invalid quotas at boot"
    ),
}
```

The full engine wire-up (task accept loop integration + retry
channel) will land with Phase B or as a carry during Phase F
verification when the worker engine loop is exercised end-to-end.
Phase A ships the primitive + loader + tests only ; no engine
call-site yet.

---

## §P34 — Sprint 21 Phase E : warrant canary tech debt closeout

Resolves the two P2 carries that landed alongside the Sprint 20
Phase E federation foundations (`6a3f199`) and surfaces a new
S22+ debt entry that the same effort uncovered.

### T-NN — `canary_wire_bytes` JCS canonical (resolved Sprint 21 Phase E)

**Before (S20)** : `crates/nexus-shell-daemon-core/src/canary/
mod.rs::canary_wire_bytes` returned `serde_json::to_vec(canary)`,
i.e. the default Rust `serde_json` ordering (insertion order
from the struct). Two observers — Rust + Python — could produce
different bytes for the same logical canary, breaking any
hash-of-wire-bytes deduplication or observation tracking.

**After (S21)** : migrated to `serde_jcs::to_vec(canary)` (RFC
8785 JCS canonical). Aligns the canary on the project-wide JCS
pattern adopted Sprint 4 Day 0 (`1c1fcfb`) for Task / Result /
Claim / Curator. The signing path was already JCS via
`nexus_core_rs::canonical_bytes` so this migration does NOT
break any existing signature — `verify_canary` continues to
hash `canonical_bytes(&canary.signed, DOMAIN_WARRANT_CANARY_V1)`,
not the wire bytes.

**Pre-launch coverage** : no canary has been published in
production yet, so even an observer cache built from the old
`serde_json::to_vec` bytes would be wiped at the first prod
publish. Zero migration risk.

**Test guarantee** : `canary::tests::wire_bytes_is_jcs_canonical_
cross_language` asserts `canary_wire_bytes` matches `serde_jcs::
to_vec` directly AND that a JCS round-trip via `serde_json::Value`
(which loses ordering) re-canonicalises byte-identical — the
property a Python observer relies on (`jcs.canonicalize(json.
loads(bytes)) == bytes`).

### T-NN+1 — `CanaryRegistry` verify Ed25519 at ingest (resolved Sprint 21 Phase E)

**Before (S20)** : `nexus_coordinator.api.canary` `POST /api/
canary/observed` accepted any shape-valid payload and recorded
it observational-only. A peer pushing a forged `signature_hex`
could pollute the local registry and skew the network-health
diagnostic.

**After (S21)** : added `verify_canary` PyO3 binding to
`nexus-core-py` (path-dep `nexus-shell-daemon-core` in `Cargo.
toml`). The handler calls `nexus_core.verify_canary(json.dumps
(payload))` BEFORE `coerce_canary_payload` + `observe_canary`.
A signature / version / hex parse failure returns HTTP 401 and
the registry stays untouched.

**Defense-in-depth** : the loopback bearer auth (Sprint 16) is
orthogonal to the verify-at-ingest gate. Even a legitimate
loopback caller cannot poison the registry with garbage. Both
checks must pass.

**Test guarantee** : `test_api_canary.py::test_observed_endpoint_
accepts_valid_canary` (happy path with `nexus_core.build_canary`)
+ `test_observed_endpoint_rejects_malformed_signature` (forged
sig → 401, registry empty) + `test_observed_endpoint_rejects_
missing_fields` (broken shape trips the JSON parse before the
crypto verify).

**Test surface added** : `nexus_core.build_canary(date, headline,
secret) -> str` PyO3 binding so Python tests can produce real
signed canaries without re-implementing the canonical-bytes
recipe in Python (which would let the two paths drift). Rust
remains the single source of truth for the signing recipe.

**Carries closed** : `verify_duress_ack` binding stays out of
scope — Sprint 21 Phase E plan §8.1 E-2 explicitly mandated
canary verify only. Hardening the duress ack channel end-to-end
is a S22+ follow-up if the threat model elevates the cost of an
observational-only ack stream.

### T-NN+2 — Iframe PII SDK Rust-wasm realignement Option G (open S22+)

**Status** : open, blocked by upstream toolchain gaps detected
during Sprint 21 Phase B G8 preflight (cf. memory
`nexus_grid_pivot.md` § « Tech debt T-NN+2 ») :

- `tract 0.22.1` tests opset 9-18 ; GLiNER export is opset 19
  (`DisentangledSelfAttention` for DeBERTa-v3 not yet documented)
- `tract` `wasm32-unknown-unknown` (browser) target not
  officially documented (only `wasm32-wasi` via wasmtime is
  upstream-supported)
- Zero production precedent for `tract` in browser-WASM ML
  inference
- `gline-rs v1.0.1` (Rust GLiNER mainstream as of 2026-01-29)
  picked `ort` (ONNX Runtime) rather than `tract`

**Trigger to revisit** :

- `tract` adds opset 19 coverage including the DeBERTa-v3
  attention layout, OR
- `ort` ships a stable `wasm32-unknown-unknown` browser backend,
  OR
- `gline-rs` publishes a `wasm-bindgen` browser target

**Workaround in production** : Sprint 21 Phase B implementation
chose `onnxruntime-web 1.24.3` (Microsoft, npm Mar 2026) +
`@huggingface/transformers` v4 tokenizer + the `knowledgator/
gliner-pii-edge-v1.0` Apache-2.0 ONNX model. Same model is the
single source of truth (coord-side `presidio-analyzer 2.2.362`
+ `GLiNERRecognizer` extra `[gliner]`). The JS path runs the
defense-in-depth client-side redaction inside the iframe. The
Rust-wasm Option G realignment would replace only the JS
runtime, not the model.

---

## References

- [The Rust Book](https://doc.rust-lang.org/book/) — chapters 1-13
  are the Sprint 1 reading list
- [Asynchronous Programming in Rust](https://rust-lang.github.io/async-book/)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
- [PyO3 user guide](https://pyo3.rs/)
- [iroh examples](https://github.com/n0-computer/iroh-examples) —
  `cargo run --example` each one, read the source
- [sendme](https://github.com/n0-computer/sendme) — small, readable
  real-world iroh application
