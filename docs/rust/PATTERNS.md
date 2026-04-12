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
