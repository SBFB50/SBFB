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

## §P35 — Sprint 23 Phase B : ephemeral worker lifecycle (restart + VRAM wipe)

Workers restart after `max_tasks_before_restart` tasks (default 50)
via `EphemeralState` state machine in `crates/nexus-worker-core/src/
ephemeral.rs`. Between each task, `cudaMemset` zeroes visible VRAM
(mitigation: model weight extraction by task N+1).

**State machine** : `Idle -> Running -> RestartPending -> Idle`.
`completed_count` tracks tasks since last restart. When
`completed_count >= max_tasks`, state transitions to
`RestartPending` and the engine signals the supervisor to restart
the process.

**Config** : `worker.toml` field `max_tasks_before_restart: u32`
(default 50). `cuda_wipe_enabled: bool` (default true, feature-gated
`gpu-ephemeral` for headless CI builds without GPU).

**Pattern** : restart-based recycling rather than process-pool
(cold-start latency Ollama ~3-8s per spawn unacceptable) or
TEE-only (consumer RTX hardware lacks NVIDIA CCM). Similar to
systemd `RuntimeMaxSec` / Kubernetes `activeDeadlineSeconds` but
task-count-based rather than time-based.

**Test guarantee** : `ephemeral::tests::*` — lifecycle state
transitions, wipe mock, config parse, restart trigger at boundary.

---

## §P36 — Sprint 23 Phase D : redundancy voting (3-worker majority)

Coordinator dispatches tasks with `redundancy_factor > 1` (values
1/3/5, wire format `Task.redundancy_factor`) to N workers and
aggregates via majority vote on hash of canonical result bytes.

**Module** : `packages/nexus-coordinator/src/nexus_coordinator/
redundancy.py` — `RedundancyDispatcher` in-memory tracker. Registers
pending tasks, collects results, votes when quorum reached.
Mismatch → outlier worker IDs recorded → quarantine route.

**Hash comparison** : SHA-256 of raw result bytes (deviation from
D3 spec BLAKE3 — functionally equivalent for equality, not used
for crypto integrity since Ed25519 sigs cover that). See
`redundancy.py::hash_result_bytes` docstring.

**Wire** : `redundancy_factor` field in `Task` struct uses
`#[serde(default)]` for runtime robustness (Python omission
deserialises to 1). Excluded from canonical bytes (`#[serde(skip)]`
in `TaskCanonical`) since it is a dispatch-only policy, not task
identity. Fix landed `34c77ce`.

**Pattern** : BOINC/Folding@Home result validator majority (10+ years
production). Fixed factor discrete (1/3/5) rather than continuous
float or coordinator-free gossip consensus.

---

## §P37 — Sprint 27 Phase B : watermark output SynthID-inspired (PRF z-test)

Two-component watermark for compute-theft detection (C-ComputeTheft,
COMPUTE_THREATS §4). Complements canary-input S22 Phase E (watermark
INPUT prompt probe).

**Detector (coordinator-side, Python)** :
`packages/nexus-coordinator/src/nexus_coordinator/watermark_detector.py`
— `WatermarkDetector` performs binomial z-test on green token ratio.
PRF = `HMAC-SHA256(secret, context_window || token_id) mod 1.0`.
Tokens with PRF score > 0.5 = green. Z-score above threshold (default
2.0) = watermarked. Non-blocking: log warning if non-watermarked
(worker may be non-opt-in), no task rejection.

**Injector (worker-side, Rust)** :
- PRF module : `crates/nexus-worker-core/src/llm/watermark.rs` —
  `compute_bias`, `should_inject`, `prf_score` (pure crypto, no
  llama.cpp dep).
- Integration call site : `crates/nexus-worker-core/src/llm/llama_cpp.rs`
  — per-step `LlamaSampler::logit_bias` chain with `+delta_logit`
  (default 2.0) on green tokens before sampling. Context window of
  last N token IDs hashed via PRF. Opt-in via `[watermark]` section
  in `worker.toml` (`enabled`, `delta_logit`, `window_size`).
  Sample config : `configs/watermark.toml.sample`.

**Design choice** : SynthID-inspired additive bias (Nature 2024 Google
DeepMind) rather than Kirchenbauer KGW (ICML 2023). KGW rejected:
vulnerable to BIRA attack (arXiv:2509.23019, sept 2025) — deterministic
green-list partitioning exploitable via iterative rewriting. Our
multi-token context hash + rotatable secret makes partitioning
non-reproducible by attacker.

**Scope cuts** : Ollama backend injection → S28+ (API no logit hook).
Full SynthID Tournament Sampling → S28+ (CDF modification complex).

---

## §P38 — Sprint 27 Phase C : trust-web multi-forge cross-validate

Couche 3 Sybil-resistance (CONTRIBUTOR_ATTESTATION_RFC.md). Provides
offline verification that a contributor has signed commits on 2+ forges
(jurisdictional diversity = Sybil-expensive).

**ForgeParser** : `crates/nexus-core-rs/src/attestations/forge_parser.rs`
— executes `git log --show-signature --format=<custom>` on local clones.
Parses GPG (RFC 4880) and SSH (RFC 8709) signatures. Aggregates by
fingerprint: commit count, first/last seen, forge URL, sig type.
No `git2-rs` dep (overhead for a one-shot parser).

**TrustCache** : `crates/nexus-shell-daemon-core/src/trust_cache.rs`
— SQLite LRU cache (rusqlite 0.32, WAL mode). TTL 7 days. Schema:
`forge_contributions(repo_url, fingerprint, commit_count, first_seen,
last_seen, sig_type, cached_at)`. Pattern reuse quarantine_queue S21.

**TrustWebManager** : `crates/nexus-shell-daemon-core/src/trust_web.rs`
— aggregates cross-forge contributions. Trust score = `forge_count ×
commit_tenure × delegation_depth_decay` (decay -1 trust_level per hop
from anchor seed, minimum 1). Gossip topic
`nexus-grid/trust-web/v1` for DelegationCert publication/subscription.

**DelegationCert v1 extended** : `crates/nexus-core-rs/src/attestations/
delegation.rs` — added `trust_level: u8` (1-5), `valid_until:
Option<DateTime<Utc>>`, `scope: DelegationScope { org_name,
forge_urls }`. Canonical JCS domain `DOMAIN_DELEGATION_CERT_V1`
unchanged (pre-launch protocol redefinition).

**Bootstrap** : `configs/trust_web_seeds.toml` — placeholder FlowUP
Ed25519 key as sole seed anchor. Real ONG anchors (Amnesty, HRW, CPJ,
EFF) target S28 outreach.

---

## §P39 — Sprint 36 Phase A : coordinator DB singleton in DaemonHttpState

`nexus-coordinator-rs` owns a single SQLite database opened at daemon
boot via `CoordinatorDb::open(~/.sbfb/coordinator.db)` with WAL mode.
The connection lives in `DaemonHttpState` as
`Arc<Mutex<CoordinatorDb>>`. HTTP handlers lock briefly (~1 ms) per
SQL operation.

**Lesson (P2-A-2)** : Sprint 35 Phase B shipped the submit handler
with `open_in_memory()` per request — a proof-of-concept shortcut
that made the endpoint functionally stateless. The fix (singleton in
state) was deferred as P2-REVIEW-B-1. Per-request ephemeral DB
instances should never ship even as PoC; always wire the shared state
from the start.

The `submit_task` free function in `dispatcher.rs` takes `&CoordinatorDb`
by reference so both owned (`TaskDispatcher` for unit tests) and shared
(`Arc<Mutex<CoordinatorDb>>` for handlers) usage patterns work without
lifetime gymnastics.

---

## §P40 — Sprint 40 Phase A : case-sensitivity convention Rust vs Python wire identifiers

Python coordinator uses `.lower()` on wire identifiers (pubkey_hex,
project_id) before storage/comparison. Rust coordinator-rs preserves
the original case. Both are valid pre-launch (no cross-language wire
traffic yet). Post-v1.0, when Python coordinator is removed (S45),
the Rust convention becomes authoritative. Until then, any wire
comparison that crosses the Python/Rust boundary must normalize case
explicitly. The `hex::encode` function already outputs lowercase.

---

## §P41 — Sprint 42 Phase A : warrant canary WARN/ALARM threshold rationale

Canary registry uses two time-based thresholds for dead-man-switch
detection (`canary_registry.rs` / `canary_registry.py`):

- `WARN_THRESHOLD_DAYS = 30` — expected refresh cadence. A canary
  older than 30 days triggers a warning-level status. Aligned with
  RFC 9591 (FROST DKG) recommended signing cadence for periodic
  attestations (monthly rotation).
- `ALARM_THRESHOLD_DAYS = 45` — hard dead-man-switch boundary.
  Equal to `CANARY_VALIDITY_DAYS`. Past this age, the canary is
  considered expired and the dead-man-switch fires.

**Why 30/45 and not 7/14 or 90/180?** The 30-day warn window
gives operators one full calendar month to refresh. Shorter windows
(weekly) create noise for solo operators without oncall rotation.
Longer windows (quarterly) delay detection of compromise. The 45-day
alarm provides a 15-day grace period after warn — enough for a
human to act on the warning before the switch fires.

These values are constants in both Python and Rust coordinators.
Pre-v1.0, the Python values are authoritative. Post-v1.0 (S45,
Python coordinator removed), the Rust constants become sole
source of truth.

---

## §P42 — Sprint 44 Phase A : ChainResult mutations contract

`guardrails.rs` `ChainResult` carries a `mutations: Vec<(String,
String)>` field — pairs of `(reason, replacement)`. As of S44, no
guardrail implementation emits `Mutation` outcomes: the three active
guardrails (`OutputSafetyGuardrail`, `PiiGuardrail`,
`CanaryInputGuardrail`) return only `Pass` or `Reject`.

The mutations vector exists for post-v1.0 guardrails that transform
content instead of rejecting it (e.g. PII masking, profanity
substitution). When the first Mutation consumer is implemented:
- `reason` = human-readable string (e.g. `"pii_redacted"`)
- `replacement` = the transformed text to substitute into the
  response
- The chain runner in `run()` collects mutations from *all*
  guardrails, even after a `Reject` from a later guardrail — the
  caller decides whether to use mutations from a partially-rejected
  chain.

---

## §P43 — Sprint 44 Phase A : pow_keypair identity equivalence

`DaemonHttpState.pow_keypair: Arc<KeyPair>` is the daemon's
long-lived Ed25519 identity. Despite its historical name (PoW
challenge context Sprint 19), it serves three roles:

1. **iroh node identity** — derived from the same keypair that the
   iroh `Endpoint` uses for peer-to-peer connections
2. **provenance signer** — `deploy.rs` signs artifact hashes and
   provenance attestations with this keypair
3. **coordinator identity** — equivalent to the Python
   `coordinator.keypair` used for task signing and kudos ledger

This equivalence holds through v1.0. Post-S45 (Python coordinator
removed), `pow_keypair` becomes the sole source of truth for all
daemon identity operations.

---

## §P44 — Sprint 56 Phase D : forbid vs deny unsafe_code convention

Workspace convention for `#![forbid(unsafe_code)]` vs
`#![deny(unsafe_code)]` + `#![cfg_attr(test, allow(unsafe_code))]`:

- **`#![forbid(unsafe_code)]`** — used by crates with zero transitive
  FFI and no need for `unsafe` in tests (e.g. `nexus-trace-core`).
  `forbid` cannot be overridden by inner `#[allow]` attributes, so
  it provides the strongest guarantee.
- **`#![deny(unsafe_code)]`** + **`#![cfg_attr(test, allow(unsafe_code))]`**
  — used by crates that depend on transitive FFI (e.g.
  `nexus-worker-core` depends on `llama-cpp-2` behind the
  `llm_llama_cpp` feature, whose C bindings contain `unsafe`) or
  whose tests call APIs that became `unsafe` in edition 2024
  (e.g. `std::env::set_var`). `deny` provides the same safety bar
  as `forbid` in production code while allowing test-only
  flexibility.

Why not `forbid` + `cfg_attr(test, allow(…))`? Because `forbid`
is a hard ceiling — `#[allow]` cannot override it even under
`cfg_attr`. The compiler emits E0453 "allow(unsafe_code)
incompatible with previous forbid". This was the root cause of
build failures when transitioning to edition 2024 (set_var
unsafe).

Current crate usage (Sprint 56):
| Crate | Lint | Reason |
|---|---|---|
| `nexus-trace-core` | `forbid` | pure Rust, no FFI, no unsafe tests |
| `nexus-core-rs` | `deny` + `cfg_attr` | tests use set_var (edition 2024 unsafe) |
| `nexus-shell-daemon-core` | `deny` + `cfg_attr` | same |
| `nexus-worker-core` | `deny` + `cfg_attr` | llama-cpp-2 FFI + set_var |

---

## §P45 — Sprint 56 Phase D : rustfmt version drift between sessions

**Root cause**: Woodpecker CI pins `rust:1.94` (with SHA256
digest for supply chain), while the local dev environment runs
rustc 1.95 / rustfmt 1.9.0. Formatting output differs between
rustfmt versions for certain patterns (edition 2024 import
reordering, trailing comma placement, match arm formatting).

**Symptom**: `cargo fmt --check` passes locally (1.95) but fails
in CI (1.94), or vice versa. Developers reformat for CI, then the
next session with a different rustfmt version re-introduces drift.

**Solution applied (Sprint 56)**:
1. `docker/ci/Dockerfile` pinned to `rust:1.94` to match
   Woodpecker CI — the local Docker pipeline now catches drift
   before push.
2. Developer workflow: always run the Docker CI pipeline before
   push (per `feedback_wsl_before_push.md` memory). This ensures
   `cargo fmt --check` runs under the same rustfmt as CI.

**Recommended future fix**: add `rust-toolchain.toml` at repo
root pinning `channel = "1.94.0"`. This makes `rustup` use the
CI-matching toolchain for all local `cargo` commands, eliminating
drift without Docker. Update all three files (rust-toolchain.toml,
Woodpecker CI image, docker/ci/Dockerfile) atomically when
upgrading Rust.

---

## §P46 — Sprint 57 Phase A : cross-platform cfg strategy

The workspace targets Linux (CI + VPS) and Windows (dev machine).
macOS is tested in GHA but has no VPS deployment.

### Platform-gated production code

21 `cfg(unix)` / 12 `cfg(windows)` occurrences across 11 files.
All production code is correctly dual-gated:

| Feature | Unix | Windows | Files |
|---|---|---|---|
| File permissions (0o600) | `set_mode()` / `set_owner_only_perms()` | Inherited ACL (no-op) | auth.rs, crypto.rs, keystore.rs, runtime.rs |
| IPC socket | `uds_server` (Unix domain socket) | `named_pipe_server` (Win32 named pipe) | main.rs |
| Signal handling | `tokio::signal::unix::SignalKind::interrupt()` | `tokio::signal::ctrl_c()` | main.rs |
| Process kill | `libc::kill(pid, SIGINT)` | `child.kill()` | main.rs (launcher), lib.rs (test-harness) |
| IPC connect | `tokio::net::UnixStream` | `tokio::net::windows::named_pipe` | executor/main.rs |
| Build test | `Command::new("sleep")` | `Command::new("timeout")` | build_executor.rs |

### Platform-gated tests

Two test functions are gated `#[cfg(unix)] #[test]`:
- `auth.rs::unix_permissions_0600` (shell-daemon-core)
- `auth.rs::unix_permissions_0600` (launcher)

These test unix file permission enforcement (mode 0o600). No
Windows equivalent exists because Windows uses inherited ACLs
from the parent directory. The security property (only the owning
user can read the token file) is maintained on Windows via default
user-profile ACLs.

### CI coverage

GHA `rust-ci.yml` runs `cargo nextest run --workspace` on three
OS: ubuntu-latest, windows-latest, macos-14. All 1227+ tests pass
on all three platforms (unix-gated tests are skipped on Windows
via `cfg`, not via `#[ignore]`). Woodpecker CI (ci.sbfb.world)
runs Linux only.

---

## §P47 — Sprint 58 Phase A : INVITE_FORMAT_VERSION wire format policy

Sprint 55 Phase D renamed `INVITE_VERSION` (u8) to
`INVITE_FORMAT_VERSION` (u16) in `nexus-worker-core/src/invite.rs`
for naming consistency with `TASK_FORMAT_VERSION`.

Current value: `INVITE_FORMAT_VERSION: u16 = 2`. Version 1 was the
original Sprint 3 format (never distributed to external nodes, hard
bumped to 2 in Sprint 4 when the signed invite envelope was added).

**Pre-launch policy** (applies until tag v1.0):
- Version = 2, hardcoded. No multi-version decoder.
- Decoder rejects `version != INVITE_FORMAT_VERSION` immediately.
- Changes to the invite wire format redefine v2, they do not bump
  to v3. There is no external state to be backwards-compatible with.

**Post-v1.0 policy** (activates at tag v1.0):
- Each breaking change to the invite format bumps the version.
- Decoder accepts a range `[MIN_SUPPORTED..=CURRENT]`.
- New optional fields use `#[serde(default)]` with inline rationale
  for runtime tolerance (not historical compat).
- u16 range [0, 65535] provides decades of runway.

Cross-ref: `CLAUDE.md §Pre-launch protocol policy`,
`nexus-worker-core/src/invite.rs:73`.

---

## §P48 — Sprint 59 Phase C : StorageWriteLimiter GCRA per-author per-app

Storage write endpoints (`storage_set`, `storage_delete`) are
rate-limited to prevent a single author from flooding an app's
iroh-docs namespace.

### Implementation

`StorageWriteLimiter` in `nexus-shell-daemon-core/src/storage_limiter.rs`
wraps `governor::DefaultKeyedRateLimiter<String>`. The composite key
is `"{author}:{app_name}"`, giving independent quotas per author per
app.

```rust
pub const STORAGE_WRITES_PER_MINUTE: u32 = 10;

pub fn check_write(&self, author: &str, app: &str) -> bool {
    let key = format!("{author}:{app}");
    self.limiter.check_key(&key).is_ok()
}
```

The limiter lives in `DaemonHttpState` as `Arc<StorageWriteLimiter>`.
HTTP handlers call `check_write` before writing; rejection returns
HTTP 429.

### Design rationale

- GCRA (Generic Cell Rate Algorithm) via governor provides smooth
  rate limiting without hard window boundaries — a burst of 10
  writes is allowed, then the bucket refills continuously.
- Keyed by `{author}:{app}` rather than just author: one author
  contributing to many apps does not exhaust a shared quota.
- `retain_recent()` is called periodically to garbage-collect
  stale keys from the internal DashMap.

Cross-ref: `nexus-shell-daemon/src/storage_api.rs` (HTTP handler
integration), `nexus-shell-daemon/src/runtime.rs` (state wiring).

---

## §P49 — Sprint 59 Phase A : Kudos-v2 log-utility + EMA fairness

LT-1 reform: replace linear `credit(tokens)` with a logarithmic
utility function + exponential moving average decay.

### Log-utility credit

```rust
const KUDOS_LOG_SCALE: f64 = 1000.0;

pub fn log_utility(tokens: u64) -> u64 {
    (KUDOS_LOG_SCALE * (1.0 + tokens as f64).log2()).max(1.0) as u64
}
```

`log2` is chosen for informatics intuition: doubling the token
output adds exactly +1000 kudos. This compresses the reward gap
between large GPU tasks and small tasks (Matthew effect mitigation).

### EMA effective score

```rust
pub const KUDOS_EMA_ALPHA: f64 = 0.97;

pub fn effective_score(entries: &[KudosEntry], now_secs: u64) -> u64 {
    entries.iter().map(|e| {
        let age_days = now_secs.saturating_sub(e.created_at) / 86400;
        (e.amount as f64 * KUDOS_EMA_ALPHA.powi(age_days as i32)) as u64
    }).sum()
}
```

Each entry decays by `alpha^age_days`. Half-life ~23 days at
alpha=0.97. Inactive workers naturally lose score; consistent
contributors maintain theirs. Alpha=0.95 (S21 research) decayed
too fast for pre-launch contribution frequency.

### Invariants

- `log_utility(0) >= 1` — every accepted result earns something.
- `effective_score(&[], _) == 0` — no entries = zero score.
- Kudos remain non-monetary, non-transferable (Day 0 decision #7).
- Hash chain integrity preserved: `compute_entry_hash` uses
  `canonical_bytes + DOMAIN_KUDOS_V1 + BLAKE3`.

Cross-ref: `nexus-coordinator-rs/src/kudos_ledger.rs`,
`memory/fairness_vision.md`.

---

## §P50 — Sprint 60 Phase A : tray icon main-thread message loop

The launcher uses `tray-icon` 0.24 + `muda` 0.19 for a Windows
notification area icon with context menu.

### Architecture constraint

Win32 requires the tray icon's hidden HWND and message pump to live
on the thread that created them. `tray-icon` 0.24 does NOT create
its own message pump — the caller must call `PeekMessageW` /
`TranslateMessage` / `DispatchMessageW` on the builder thread.

This means the main thread runs the tray event loop (blocking
polling with `thread::sleep(100ms)`), while the tokio runtime runs
on a background thread:

```rust
// main.rs — simplified
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all().build().unwrap();
    // spawn daemon, browser, identity init on rt...
    let tray_state = tray::create_tray()?;
    tray::run_event_loop(&tray_state, &url, &ctrl_c_rx);
    // on exit: rt drops, daemon shuts down
}
```

### Win32 message pump

`pump_win32_messages()` is a `#[cfg(windows)]` function using
direct FFI (`PeekMessageW` / `TranslateMessage` / `DispatchMessageW`)
rather than pulling in `windows-sys` as a dependency. The `Msg`
struct is a minimal `#[repr(C)]` layout matching Win32 `MSG`.

This is called every iteration of the event loop before checking
`MenuEvent::receiver()` and `TrayIconEvent::receiver()`.

### Fallback

On non-Windows platforms, `create_tray()` may fail (no GTK, no
AppKit). The launcher falls back to `ctrl_c().await` (pre-S60
behavior) with a warning log.

Cross-ref: `nexus-launcher/src/tray.rs`, `nexus-launcher/src/main.rs`.

---

## §P51 — Sprint 65 Phase A / Sprint 66 Phase B : Raw-op store+forward

The public feed uses a **raw-op** extensibility pattern: each
`FeedEntry.op` is stored as `serde_json::Value`, not as a typed enum.
This allows nodes to store and propagate unknown operation types
without interpretation — a node running v1 code will relay a
`CuratorVouched` operation added in v2 without dropping it.

### Core API (public_feed.rs)

```rust
// Struct: FeedEntry.op is Value (l.79)
pub struct FeedEntry {
    pub op: Value,  // NOT PublicFeedOperation
    // ...
}

// Typed access via try_parse_op (l.110-112)
pub fn try_parse_op(op: &Value) -> Option<PublicFeedOperation> {
    serde_json::from_value(op.clone()).ok()
}

// Discriminant extraction (l.115-117)
pub fn op_type(op: &Value) -> Option<&str> {
    op.get("op_type").and_then(|v| v.as_str())
}
```

### Validation (public_feed.rs l.224-236)

`validate_feed_operation` accepts unknown ops with size check only:

```rust
pub fn validate_feed_operation(op: &Value) -> Result<(), String> {
    // 1. Size gate (MAX_OPERATION_JSON_SIZE = 64 KB)
    // 2. If parseable as known op → validate_known_operation()
    // 3. Unknown ops pass (store + forward)
}
```

### Invariants

- Adding a new `PublicFeedOperation` variant does NOT bump
  `FEED_FORMAT_VERSION` — only envelope changes do.
- `#[serde(default)]` on `pow_nonce` is for runtime tolerance
  (local entries omit it), not historical compat.
- `FeedEntryCanonical` mirrors `FeedEntry` minus `entry_hash`,
  `signature`, `pow_nonce` — canonical bytes exclude transport
  fields.

Cross-ref: `PUBLIC_FEED_SPEC.md §2, §10`, `CLAUDE.md` pre-launch
protocol policy.

---

## §P52 — Sprint 66 Phase A / Sprint 67 Phase D : Backend-agnostic enum with Deref

`BlobStore` in `nexus-core-rs/src/node.rs` wraps `MemStore` and
`FsStore` behind a two-variant enum with a manual `Deref` to the
common trait object (`Store`). Callers receive `&Store` from
`Node::blobs_store()` regardless of backing implementation.

```rust
// node.rs l.111-126
pub enum BlobStore {
    Mem(MemStore),
    Fs(FsStore),
}

impl std::ops::Deref for BlobStore {
    type Target = Store;
    fn deref(&self) -> &Store {
        match self {
            BlobStore::Mem(s) => s,
            BlobStore::Fs(s) => s,
        }
    }
}
```

When to use: two concrete backends for the same API surface, decided
at startup, immutable for the process lifetime. The enum keeps
ownership (no `Box<dyn>`) while `Deref` erases the variant for
downstream code.

Limitation: if a third backend appears, the enum grows — acceptable
for <=3 variants, switch to `Box<dyn Store>` beyond that.

Cross-ref: S66 Phase A `BlobStore` + `FsStore`, S66 audit P2-66-2.

---

## Note — Feed republish test limitation (P2-66-1)

`test_feed_republish_at_boot` (runtime.rs l.1961) verifies that the
daemon boots without panic and that `feed_handle.is_some()` after
restart, but does NOT assert that feed entries are actually present
in iroh-docs after republish. The iroh-docs `Doc` API does not
expose a synchronous read-back that would make this assertion
deterministic in a unit test. A future integration test with a
second node (cross-node feed sync) is the proper verification path.

Cross-ref: S66 Phase C feed republish, S66 audit P2-66-1.

---

## T-NN+3 — canonical bytes duplication Factory/coordinator (open, S70 documented)

`crates/nexus-coordinator-rs/src/provenance.rs` and
`crates/sbfb-factory/src/gates.rs` each contain an independent
`canonical_bytes` implementation for building the provenance
signing payload (sorted-keys JSON via `serde_json` + domain
separator + version).
Both produce identical output today (verified S69 audit P2-C-1),
but changes to one without the other would silently break
signature verification.

**Root cause** : `sbfb-factory` was designed as an external crate
(D2 v4 roadmap) and cannot depend on `nexus-coordinator-rs`.
The shared logic belongs in `nexus-core-rs` which both crates
already depend on.

**Plan** : extract `canonical_bytes` into `nexus-core-rs` post-S70
(Phase C or later sprint). Both call-sites become thin wrappers.
Until extraction, any modification to the signing payload format
MUST update both files and add a cross-ref comment.

Cross-ref: S69 audit P2-C-1 (1/3→documented), S70 Phase B.

---

## Note — serde_json vs JCS pre-launch rationale (P2-C-2, S70 documented)

The provenance `canonical_bytes` functions use `serde_json`
(sorted-keys via `json!({})` macro / BTreeMap) rather than
`serde_jcs` (RFC 8785) for the signing payload. This is
intentional pre-launch:

- All provenance fields are ASCII strings, integers, and booleans.
  No floats, no Unicode normalization edge cases — the two
  serializers produce identical output for this payload shape.
- `serde_jcs` is a workspace dependency (used by `nexus-core-rs`
  for canary wire) but not by `nexus-coordinator-rs` or
  `sbfb-factory`. Adding it to these crates pre-launch brings
  zero practical gain for the current ASCII-only payload.
- The canary wire (`canary_wire_bytes`) already uses `serde_jcs`
  because its payload includes free-text fields where ordering
  matters across languages (§P34 T-NN).

**Post-launch policy** : if provenance gains float or free-text
fields, migrate to `serde_jcs` in the same commit. The
`canonical_bytes` extraction (T-NN+3) is the natural moment.

Cross-ref: S69 audit P2-C-2 (1/3→documented), S70 Phase B.

---

## Note — P2-G-1 exe lock release build CLOSED (S70, 8 sprints non-reproductible)

`cargo build -p nexus-shell-daemon --release` intermittently
failed on Windows with a file lock error on the output exe.
Timeline:

- S59 audit : first reported (P2 G-1).
- S60 Phase B : investigated with `handle.exe` (Sysinternals),
  5 consecutive builds clean → CLOSED.
- S60 Phase E : reproduced during verification → REOPENED.
- S61-S69 : monitoring every sprint, never reproduced.
- S70 Phase B : **CLOSED** definitively.

**Conditions to reopen** : if the exe lock reproduces 2+
consecutive times in the same sprint, reopen as P1 with
`handle.exe` capture of the locking process. A single transient
occurrence does not warrant reopening — the Windows file system
cache and antivirus are the likely culprits for the original
intermittent failure.

Cross-ref: S60 Phase B (`cfa3c3c`), S60 Phase E review,
S69 verification §5, S70 Phase B.

---

## §P53 — Sprint 71 Phase B : deterministic compute quorum (B-2) + provider/backend axes (D8)

### Deterministic inference as the prerequisite for hash-exact quorum

The redundant-task quorum (`validate_quorum`, `nexus-coordinator-rs/
src/validator.rs`) accepts a result when a **strict majority of
workers report an identical `result_text`** (stored in a column
named `sha256` for Sprint-55 build-task heritage — for inference it
is the raw text, no hash). That quorum is *useless* for inference
unless two honest workers produce the same text. Sprint 71 Phase B
(B-2 / D2 / PO-11) makes them converge by forcing **deterministic
decoding** at the source rather than loosening the comparison (a
fuzzy threshold would open a "close enough" attack surface and is
not reproducible — rejected in kickoff §5 D2).

Mechanism, end to end:

- **`Task.verifiable: bool`** (`nexus-core-rs/src/task.rs`) — a
  **signed** field (inside `task_canonical_bytes`, unlike
  `redundancy_factor` which is excluded as dispatch-only, Sprint 23
  `34c77ce`). The execution *mode* (greedy vs sampling) changes what
  the worker computes, so it is task identity: every worker in a
  quorum must agree on it under one coordinator signature, and a
  worker reads it only after `verify_signature()`. Same wire shape as
  `is_open_source` (`#[serde(default)]`, runtime tolerance,
  `TASK_FORMAT_VERSION` stays 1 pre-launch).
- **Worker submission** (`nexus-worker-core/src/engine/runtime.rs`,
  `build_generate_params`) — when `task.verifiable`, params are built
  with `GenerateParams::deterministic(seed)`: `temperature = 0` plus a
  seed **derived deterministically from `task_id`** (`deterministic_seed`,
  first 4 bytes of `blake3(task_id)`), so every honest worker on the
  same task pins the same seed. This determinism seed is NOT a secret
  and is distinct from the per-task `watermark_seed` PRF.
- **llama_cpp backend** is already deterministic and needs no change:
  its sampler chain ends in an unconditional `LlamaSampler::greedy()`
  (`llm/llama_cpp.rs:327`), a terminal argmax selector; temperature
  scales logits but never moves the argmax (llama.cpp #3005). The seed
  is inert on this path (greedy never draws).
- **Ollama backend was the real gap** (preflight S1a, PLAN-ADAPT):
  `OllamaBackend::generate` built its `GenerationRequest` with only
  `system` + `format` and **never attached `GenerationOptions`**, so
  Ollama applied the Modelfile defaults (temp ~0.8, random seed) and
  two honest workers diverged. Fix: `deterministic_options(params)`
  forwards `temperature` + `seed` via `GenerationOptions::default()
  .temperature(t).seed(s as i32)` (the API existed in the pinned
  ollama-rs 0.2.6, just unused; the type was renamed `ModelOptions`
  when S72 Phase C bumped ollama-rs to 0.3.4 — same seed/temperature
  contract). temperature=0 alone is insufficient
  there — a fixed seed is needed against residual non-determinism
  (ollama/ollama#5321).

**Limit (D2 ⚠️ / R1)** : determinism is guaranteed *same-machine /
same-backend / same-model-quant*. Cross-GPU float non-determinism can
break bit-exactness; the real cross-machine quorum proof is scope-cut
to **S75** (#11). The validator itself is unchanged by B-2 — it stays
mode-agnostic; outlier rejection (`quorum_rejects_nondeterministic_
divergence`) is preserved.

### `provider` (prompt-adaptation) vs `backend` (execution) — two orthogonal axes (D8)

Two unrelated notions both informally called "provider" exist and are
**intentionally not unified**:

- **Prompt-adaptation provider** (`sbfb-factory/src/process.rs`,
  `PROVIDERS = ["claude","codex","gpt","local","human"]`) — *which
  agent reads a generated context pack*. A Factory concern.
- **Execution backend** (`nexus-worker-core`, `LlmBackend`: Ollama /
  llama_cpp) — *what engine runs an inference task*. A worker concern.

They never meet on the same code path (a `claude`-targeted prompt and
an Ollama-executed compute task are different lifecycles), so merging
them would conflate two axes that vary independently. Documented here
per kickoff §5 D8.

### Dead-module cleanup (D8)

- **`RedundancyDispatcher` removed** (`nexus-coordinator-rs/src/
  redundancy.rs` deleted, `pub mod redundancy` dropped). It was an
  in-memory majority-vote port (Sprint 40, of `redundancy.py` S23)
  **superseded by the DB-backed `validate_quorum` at Sprint 55**
  (`0cb576d`); zero live callers at HEAD. No future consumer → pure
  removal.
- **`execute_build` kept but marked dormant** (`nexus-worker-core/src/
  build_executor.rs`). Tier 2 of LT-7 self-hosted build; the worker
  dispatch routes no `task_type == "build"` yet, so it has no live
  caller — but LT-7 worker-quorum build E2E is a *named* future
  consumer (S75), tracked in `docs/release/ROADMAP_COMMITMENTS.md`.
  Kept, not removed, to preserve working clone/build/hash logic.

### Off-sprint deps validated (G13)

CVE/advisory scan of the three deps the off-sprint Factory block
pulled in (versions from `Cargo.lock`): `portable-pty 0.9.0`,
`async-stream 0.3.6`, `futures 0.3.32`. None carries a critical/high
RustSec advisory, and none sits on a crypto/wire/network/signing path
(portable-pty is the local, loopback, gated Operator PTY). No bump
required. `ollama-rs 0.2.6` (touched by the B-2 wiring; bumped to
0.3.4 in S72 Phase C) is likewise advisory-clean.

Cross-ref: S71 Phase B `verifiable` field + greedy/seed quorum,
S71 preflight S1a (PLAN-ADAPT, Ollama gap), kickoff §5 D2/D8.

---

## §P54 — Sprint 71 Phase A : dispatch key alignment (B-1) + first cross-process compute E2E (B-3)

### B-1 — one key, one prefix

The coordinator dispatch loop wrote each task under the iroh-docs key
`format!("tasks/{id}")` (`nexus-shell-daemon/src/dispatch_loop.rs`)
while the worker engine has always scanned `get_many_by_prefix(b"task:")`
+ `strip_prefix("task:")` (`nexus-worker-core/src/engine/runtime.rs`).
The two never met: **no dispatched task was ever claimed by a real
worker** — every prior "it works" was an in-process test that hand-wrote
the `task:` key. Fix (D1): move the *writer* onto the worker's
long-standing `task:` prefix (one line), never the reader — the claim /
result / completed-id cache path is the rodé surface, far more
regression-prone than the single writer. No tolerant dual-prefix read
(that would be permanent dead code masking the bug, rejected kickoff
§5 D1). Pre-launch: this changes the applicative wire key but no
third-party node speaks it, so no migration / range decoder (intake
§2.3). The two `tasks/` hits remaining in `dispatch_loop.rs` are
*comments* documenting the old bug, not live writes.

### B-3 — first cross-process compute E2E

`dispatch_loop::tests::dispatched_task_is_claimed_and_executed_by_
worker_engine` is the first test that drives a task through the real
two-component path: daemon dispatch writes the doc → a worker `Engine`
(via the additive `Engine::docs()` accessor) claims it by prefix scan →
executes → result lands. It proves *routing + execution* and — since
S72 Phase B (`08b6cb2`, P2-A-2 closed) — payload integrity too: the
E2E now asserts `ResultEntry::verify_signature()`, not merely
`results.len()==1` (the pre-existing S4 mirror `runtime.rs` is the
sibling path).
The plan put this E2E in `nexus-test-harness`; it landed in
`nexus-shell-daemon` instead (it needs the dispatch-loop internals) —
a located, not scoped, change.

**Windows-native caveat (P2-A-1)** : the worker-engine iroh-docs pump
does **not** run green on native Windows — `dispatched_task_is_claimed
_...` and its pre-existing S4 mirror both *time out* on the dev box yet
pass in ~2 s under CI Linux / Docker. This is an environment artefact
(confirmed by bisection, shared by the untouched mirror), not a code
regression. Verify these worker-pump E2Es via Docker / CI before any
push (`feedback_wsl_before_push`), never on the Windows box alone. The
full-workspace nextest can also hit `os error 1455` (paging-file
exhaustion) at test-binary link time on cold Windows builds; the
canonical full-workspace count comes from CI Linux. (When the phase
binaries are already warm, a full Windows nextest does complete — S71
Phase E re-measured 1528/1528 locally.)

### P2-A-1 closure (Sprint 73 Phase B) — runtime flavor is the root cause

The Windows-native hang above is **not** an unfixable environment artefact:
the root cause is the **tokio runtime flavor of the tests**. Any test that
spawns the engine/dispatch pump and then waits on a **real-time** loop
(`tokio::time::sleep` + `get_many_by_prefix`) under `#[tokio::test]`
(current_thread) can deadlock on Windows during `cargo test`
shared-process teardown. The mechanism, confirmed by reading the code:

- `node.rs` spawns the iroh-docs store (`docs_builder.spawn(...)`) which
  runs as an **actor on its own dedicated thread**; `docs.rs`
  `get_many_by_prefix` round-trips a `Query` to that actor.
- The engine pump (`runtime.rs` `run_until_shutdown` → `scan_and_execute_
  tasks`) polls that actor in a loop; the test `tokio::spawn`s the pump
  and *also* polls the actor from the main future.
- On `current_thread`, the spawned pump only advances when the main future
  yields, and the cross-thread actor wakeups race the single worker — the
  Windows scheduler deadlocks what Linux tolerates (tokio #7049,
  current_thread `cargo test` hang; the related #2499 is the
  threaded-1-thread variant — both are Windows-only teardown hangs).

**Fix (D6), cross-platform, zero `#[cfg(windows)]`:** the real-time
pump tests use `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]`,
matching how the code runs **everywhere else** — the worker binary
(`nexus-worker/src/main.rs`), the daemon, and the only working 2-node
sync example (`examples/two_nodes_docs_sync.rs`,
`multi_thread, worker_threads = 4`). It is neutral on Linux (already
green) and removes the hang on Windows. The `tokio::time::timeout(10s)`
in each E2E is kept as **defence-in-depth** so a future regression fails
fast instead of hanging the whole nextest run.

**The rule:** any test driving the iroh-docs pump concurrently with an
engine/dispatch spawn via a **real-time** wait loop **MUST** be
`multi_thread`. Converted in S73: `dispatch_loop.rs`
(`dispatch_loop_writes_to_doc`, `dispatched_task_is_claimed_and_executed_
by_worker_engine`) and `runtime.rs` (`engine_applies_start_event_on_run`,
`engine_transitions_to_processing_when_project_is_enrolled`,
`engine_claims_and_executes_tasks_on_registered_doc`,
`rate_limit_gate_admits_fresh_tuple`, `engine_shuts_down_gracefully`).

**Multi_thread surfaces real races the single thread hid.** Converting
`dispatch_loop.rs` exposed a latent test bug: `run`'s `select!` races
`rx.recv()` against the shutdown signal, so a buffered task can be dropped
if shutdown wins. On `current_thread` the buffered message deterministically
won (a `yield_now` was enough); under `multi_thread` it is a true 50/50
race and the test flaked under full-workspace load (`left: 0`). The fix is
to **synchronise the test on the observable write** (poll the doc until the
task lands, then signal shutdown) rather than to weaken prod — dropping a
buffered dispatch on shutdown is acceptable (the HTTP client re-submits).
Lesson: when moving a test to `multi_thread`, audit every "this happens
before that" assumption that only held because one thread ran everything.

**Exception — virtual-time tests stay current_thread.**
`rate_limit_gate_rejects_saturated_tuple` and
`rate_limit_gate_defer_preserves_task` drive the pump with
`tokio::time::pause` + `advance`, which is **current_thread-only** and
fully deterministic. They never block on a real-time poll loop, so they
are immune to the hang and must **not** be converted (multi_thread is
incompatible with paused virtual time).

The upstream fix (iroh-docs 0.99.0, 2026-05-08, "Drain Actor::tasks
JoinSet") is **not adoptable** — 0.99 tracks iroh 1.0.0-rc.0 + redb@4,
forbidden by the frozen iroh 0.98 pin (R-iroh-audit P0). Revisit at the
iroh 0.98→1.0 upgrade (Gate 1). **Status: CLOSED by code fix** (verify
green on native Windows + Docker Linux per `feedback_wsl_before_push`
before declaring closed). Formal fallback if a teardown residue persisted
after multi_thread: `#[cfg_attr(windows, ignore = "P2-A-1: iroh-docs
0.98 actor pump + Windows scheduler; canonical run = CI Linux; revisit at
iroh 1.0")]` — not used, the multi_thread fix sufficed.

Cross-ref: S71 Phase A (`2f9238d`), preflight EXECUTE, review P2-A-1 /
P2-A-2 / P3-A-3, kickoff §5 D1 ; S73 Phase B kickoff §4 D6, preflight
EXECUTE (note 1).

---

## §P55 — Sprint 72 Phase C/D : three orthogonal LLM axes (D5)

SBFB now has three distinct enums that each answer a different "which
LLM?" question. Keeping them as **separate axes** (not one collapsed
`Provider`) is deliberate — each lives in a different crate, has a
different consumer, and a different lifetime. Collapsing them would
couple the operator chat router to the worker quorum runtime and to the
process prompt-portability layer, all of which evolve independently.

| Axis | Type | Crate / file | Question it answers |
|---|---|---|---|
| **Execution target** | `ExecutionTarget { Claude, Ollama, Network }` | `sbfb-factory/src/provider_router.rs` | *Where does this operator chat turn run* — Claude cloud (default pilot), Ollama local, or the SBFB network (submit→poll)? Parsed from the wire `provider` string; each arm yields the SAME `StreamChunk` contract so the SSE layer stays provider-agnostic. |
| **Prompt-adapt provider** | `PROVIDERS: &[&str]` + `&str` (process prompt portability) | `sbfb-factory/src/process.rs` | *Which agent consumes a portable prompt* — shapes `base/universal/handoff` prompt assembly per agent family (`prompt_data`, `providers_list`). Pure prompt text concern; no runtime dispatch. |
| **Worker backend** | `Box<dyn LlmBackend>` (trait object) | `nexus-worker-core/src/llm/` | *Which local inference runtime executes a quorum task* — `llama_cpp` vs `ollama`, behind the worker's deterministic-decoding contract (§P53). Never reaches the Factory. |

Why three, concretely:

- **`ExecutionTarget` is enum-dispatch, not `dyn Provider`** (§P52
  rationale): the target set is closed and known at compile time, each
  arm boxes its heterogeneous `impl Stream` into the shared
  `ProviderStream` only at the dispatch boundary (`run()`), and the
  network arm's submit→poll lifecycle (one terminal `Done`, PO-14) has
  nothing in common with Claude's subprocess NDJSON or Ollama's
  `generate_stream` — a trait would force a lowest-common-denominator
  shape and double-box.
- **The network arm is a daemon HTTP client, not an in-process runtime.**
  It submits a `TaskSubmission`-shaped body, polls `GET /tasks/{id}`,
  and reads the accepted text from `GET /tasks/{id}/result` (the
  Sprint 72 Phase D persistence primitive, `db.set_task_result` →
  `tasks.result_text`). It deliberately holds **no** `nexus-coordinator-rs`
  dependency (builds the body with `serde_json` inline) so the Factory
  crate stays free of the iroh-heavy coordinator graph (crate
  isolation).
- **The gate is upstream of all three axes.** `SENSITIVE_ACTIONS` runs
  in `handle_chat_stream` BEFORE `ExecutionTarget::from_provider(...).run()`,
  so no provider selection can bypass the spawn gate (T-OPERATOR-SPAWN,
  THREAT_MODEL §14).

Cross-ref: S72 Phase C (`3c9ea1b`, `ExecutionTarget` + Claude/Ollama
arms), S72 Phase D (network arm + result-text route + provider wiring),
§P52 (Deref backend enum), §P53 (deterministic quorum + provider/backend
axes D8).

---

## §P56 — Sprint 73 Phase C/D : FTS5 hot reindex + UNINDEXED provenance triplet (D1/D2)

The FTS5 `search_index` (M15) is a **standalone** virtual table (not
external-content): it indexes a JSON payload parsed in Rust, not a 1:1
SQL row. Two evolution patterns from Sprint 73:

**(D1) Hot incremental reindex, keyed by feed `seq` as the FTS5 rowid.**
Before S73 the index was only rebuilt at boot (`rebuild_from_feed`,
`runtime.rs`), so a gossiped project was invisible to search until the
next reboot. The fix is an `INSERT OR REPLACE INTO search_index(rowid =
seq, …)` called right after `insert_feed_entry` succeeds, **inside the
same `feed_sync` DB lock scope** (one short WAL write, no extra
round-trip). `INSERT OR REPLACE` is the canonical upsert for a standalone
FTS5 table — a re-arrived entry rewrites the same rowid (idempotent, a
second line of defence behind the `entry_hash` dedup), never a duplicate.

- **Do NOT use external-content triggers** for this. They apply only to
  `content='t'` tables; converting would mean materialising a mirror
  table + 3 triggers, and a trigger that omits `rowid` or does a bare
  DELETE corrupts the index (documented SQLite footgun). Over-engineering.
- **Do NOT rebuild O(N) on the hot path.** A full `rebuild_from_feed` per
  ingested entry holds the write lock proportionally to feed size →
  amplification DoS under feed-spam (THREAT_MODEL §11). The O(N) rebuild
  is kept ONLY as the explicit repair/migration path.
- **`busy_timeout` made explicit** (`Duration::from_secs(5)`) at DB open:
  a hot reindex may briefly contend with another writer on the single
  `Mutex<Connection>`; wait-and-retry rather than fail-fast `SQLITE_BUSY`.
- The shared extractor `extract_index_fields(op)` is used by BOTH the hot
  path (`upsert_feed_entry`) and the repair path (`rebuild_from_feed`) so
  rebuilt rows are byte-for-byte identical to hot-path rows (anti-drift).

**(D2) Enrich `SearchResult` with the provenance triplet — UNINDEXED
columns + DROP/recreate migration (M17).** A search hit must carry
`repo_url` + `commit_sha` + `archive_hash` + `provenance_hash` (+
`is_open_source`) so it can drive a fork in S74 without a second
round-trip.

- **FTS5 cannot `ALTER TABLE … ADD COLUMN`** (no ADD COLUMN on a virtual
  table). The canonical evolution path is **DROP + CREATE** with the new
  columns, then repopulate. Safe here because the index is **integrally
  reconstructible** from `public_feed` (the boot `rebuild_from_feed`
  refills it) — the drop loses no durable data. This is a **local schema**
  migration, NOT a wire format: `search_index` is never synced over
  iroh-docs, so `FEED_FORMAT_VERSION` stays 1 (pre-launch policy).
- **The provenance columns are UNINDEXED.** A 40/64-hex hash is not a
  natural-language token — a MATCH against it is meaningless and only
  inflates the index. UNINDEXED columns are **returned via SELECT but
  excluded from MATCH** (same as the existing `project_id`/`op_type`/
  `source_type` columns). Tests assert a MATCH on a hash value returns
  zero hits while the row is found by its indexed name.
- **Name bridge `artifact_hash` → `archive_hash`.** The feed payload field
  is `ReleasePublishedPayload.artifact_hash`, but the returned column / the
  S74 fork consumer (`ProofCardInput.archive_hash`) / `BrowseEntry` all
  name it `archive_hash`. `extract_index_fields` reads the **source** key
  (`artifact_hash`) and stores it under the **consumer** name
  (`archive_hash`). Reading `archive_hash` from the payload would silently
  yield `None` for every real release. The column name mirrors the
  downstream consumer; only the extraction key differs — assert the exact
  mapping in tests, not just non-null.
- **DTO tolerance, not wire compat.** `SearchResult`'s new fields are
  `Option<String>` (+ `bool`): a non-release op (CuratorVouched) or a
  pre-M17 row yields `None`/`false`, serialised to JSON `null`/`false`,
  never a deserialisation error. `is_open_source` is read back tolerantly
  (`row.get::<_, Option<bool>>().unwrap_or(false)`) — an FTS5 integer 0/1
  round-trips as a bool, and an absent column degrades to `false`.

**Rowid partition tripwire (carried to S74).** Feed rows own rowid
`[1, max seq]`; browse-sourced `index_entry` rows (auto rowid) are
currently test-only. M17 DROP/recreate rebuilds feed rows keyed rowid=seq,
so it does NOT aggravate the concern — but wiring browse indexing in
production (S74) must partition the rowid space so a feed upsert cannot
clobber a browse row. Keep the doc-comment tripwire in `search.rs`.

Cross-ref: S73 Phase C (`47c9ff7`, hot reindex D1), S73 Phase D (triplet
+ M17 D2 + SearchManifest defer D3 design note
`.planning/research/s73_searchmanifest_index_node_design.md`), M15 FTS5
introduction (S67 Phase B), THREAT_MODEL §11 (search surface).

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

## §P57 — 2026-06-05 remediation : the real-frontier E2E gate (no mock on both sides of a frontier)

The platform-remediation root cause (2026-06-05) was systemic, not a
single bug: every feature had a **discovery** half (gossip / iroh-docs)
and a **service** half (SQLite / HTTP) that were *never reconciled in
production*, and **every test mocked the frontier between them**
(mockFetch on the front, a test-only `index_entry` injection, a
mock-daemon in Rust). The suite was ~1866 green while the cross-node
core was broken, because each test stubbed one side of the boundary it
was meant to exercise.

**Rule: a frontier — a point where data crosses a process, a node, or a
transport — must have at least one test that exercises it for REAL on
both sides.** Mocking one side to unit-test the other is fine and
necessary; mocking *both* sides of the *same* frontier proves nothing
about the frontier and is how this class of bug hides.

The canonical guard is the gate
`runtime::tests::e2e_network_execute_gate_real_http_no_frontier_mock`
(`nexus-shell-daemon/src/runtime.rs`): it boots a real `DaemonRuntime`
(real loopback HTTP + auth + iroh node + dispatch_loop + result_sync +
validator_loop + coordinator DB) and a real `nexus-worker` Engine on a
**separate iroh node** joined by a real invite ticket, submits a task
over real HTTP and polls the result back over real HTTP. The only mock
is the deterministic `StubBackend` LLM (which is *not* a frontier — it
is the leaf compute). It runs under `nextest` (so it is part of the
blocking fail-fast) and `#[serial(sbfb_env)]` because it sets the bearer
token + the local-worker toggle via process env.

Companion live smokes (not committed as tests because they spawn real OS
binaries) proved the same path end to end with the actual release
binaries, including the on-demand worker auto-spawn (`local_worker.rs`)
and its Windows Job Object / Unix `PR_SET_PDEATHSIG` orphan kill — the
two things an in-process test cannot cover. Re-run them by hand from a
real daemon before a release tag.

**Instance — Browse-card freshness (2026-06-05, hotfix #6 follow-up).**
The same discovery↔service split hid a second bug: a remote app
discovered over gossip (`ProjectAnnouncement` → `BrowseAggregator`
direct entry) had its reachability **status** frozen at `Unknown`
forever, because `aggregate()` probed only curator-list entries and
appended direct entries verbatim. Worse, the two halves had drifted
apart: post-#4 a direct entry's `project_id` is `blake3(project_name)`,
**not** the hosting node's dialable `node_id`, so even a naive "probe
project_id" would have dialed a non-existent endpoint and reported
`Unreachable`. The fix carries the hosting `node_id` on the entry
(`#[serde(skip)]`, daemon-internal — the UI still reads `status`),
probes *that* through the curator TTL-cache + quorum/DNS canary,
short-circuits self-hosted cards (an endpoint cannot dial itself, and
a node receiving its own gossip echo must not flip its card offline),
and — to reconcile discovery with the iroh service layer — seeds the
announcing node's `EndpointAddr` (parsed from the archive ticket) into
`memory_lookup` at announce time, mirroring `blobs.rs::fetch_ticket`.
The real-frontier guard is
`runtime::tests::freshness_probe_marks_gossiped_remote_app_reachable_e2e`:
two real iroh nodes, the production `handle_project_announcement`
ingest (seed + node_id), and a genuine dial — no mock at the
gossip↔dial boundary. Asserting the per-app `project_id` stays distinct
from `node_id` is what proves the probe dials the node, not the app id.

**Instance — Browse boot-restore (2026-06-06, hotfix #7).** A third
manifestation of the same split, found the same way (the PO opened the
shell and his three deployed apps were gone). `BrowseAggregator`'s
`direct_entries` is in-memory and starts empty on every boot; the daemon
rebuilds the *search* index from the feed (`rebuild_from_feed`) but never
the *Browse* aggregator, and publish/deploy only ever write to the
in-memory map. So a node's own apps vanish from its own Browse after a
restart — `GET /api/daemon/browse` returns `{"entries":[]}` while the
provenance, feed, blobs and gossip outbox all still hold them on disk.
The fix reconciles the persisted half into the live half at boot:
`restore_browse_from_outbox` decodes the node's own persisted outbox
envelopes with `PowEnvelope::decode` (structural, no PoW re-verification —
our own trusted data must not be dropped by a later difficulty-policy
bump) and re-ingests each project announcement through
`handle_project_announcement`, which also re-indexes search with the real
`project_name` (the feed's `ReleasePublished` op carries none, so this
closes the search-by-name gap for own apps too). The real-frontier guard
is `runtime::tests::browse_boot_restore_repopulates_aggregator_from_outbox_e2e`:
real PoW encode → real DB `insert_outbox`/`load_outbox` round-trip → real
decode → real ingest, no mock at the persistence↔aggregator boundary. The
lesson generalises: **any in-memory index built from a persistent log
must be reconstructed at boot, or it silently diverges from the durable
state after the first restart.**

**Pre-tag gate:** the E2E gate test must be green, and a manual
full-binary smoke (daemon + auto-spawned worker, submit → poll result)
must pass on the release artifacts. A frontier added in a future sprint
(a new cross-node op, a new HTTP service path) carries the same
obligation: one real-on-both-sides test, or it does not ship.

## §P58 — Sprint 74 : seed cross-noeud + pin local (programme Disponibilite)

The availability programme (Arc 3.5, ex-LT-5 pulled forward) adds the
*hosting* half of the protocol: a node can keep a distant public app online
and the network can show a best-effort "Toi + N pairs" count. Three composed
primitives, no new crypto:

- **Pin local (Phase D, M18 `keep_online`).** A per-app overlay policy
  `(project_id, enabled, archive_hash, pinned_at)` that (a) tags the archive
  blob skip-GC and (b) gates the boot re-broadcast. It is an OVERLAY on the
  existing outbox replay (#7/#8), not a new broadcast path. BOTH states are
  explicit: a self-deploy (`finalize_deploy`) or voluntary seed (`seed_voluntary`)
  writes an `enabled = 1` row carrying the `archive_hash` (it drives the Phase F
  boot `SeedAnnounced` re-announce via `list_keep_online_enabled` + the skip-GC
  tag), and toggling "Garder en ligne" OFF writes an `enabled = 0` row. An absent
  row defaults to enabled for the rebroadcast gate (R6 fallback: the gate only
  suppresses rows it finds explicitly `enabled = 0`).

- **Seed authentifie (Phase E, ALPN `sbfb/seed/0`).** `SeedRequest`/`Response`
  signed Ed25519+JCS under a NEW domain `DOMAIN_SEED_REQUEST_V1` (separate ALPN
  message, so unlike a feed op it needs its own domain). Invite-gated (M19
  `seed_invite`, capability bound to the `(project_id, archive_hash)` PAIR — an
  invite can never be redeemed to pin foreign content). The 4th ALPN rides
  BEFORE node spawn (iroh 0.98 Router registers no post-spawn protocol →
  `ExtraProtocolFactory` closure). Anti-replay nonce TTL `2*window+1` (closes
  the inclusive-edge skew, Codex C3).

- **Annonce + compteur (Phase F, `SeedAnnounced`).** A TYPED feed-op variant
  (NOT a new domain — it rides the FeedEntry `DOMAIN_FEED_V1` chain). 0-bump
  `FEED_FORMAT_VERSION` (S67 precedent: a typed variant is 0-bump AND gives
  insert-time validation). `seeder_node_id == FeedEntry.author_pubkey` (the
  daemon's pow_keypair == node identity, same secret) — the seeder signs ONLY
  its seed claim, NEVER the app provenance (R5 / Radicle delegate!=seeder).
  The count lives in an in-memory `SeedRegistry` (TTL 48h lazy-purge +
  self-clocked global sweep), fed by ingest with three gates: op is
  SeedAnnounced, `seeder == author` (anti-impersonation), `seeder != my_node`
  (self counted once at query time, never via the feed echo). Best-effort by
  design — **content-addressing BLAKE3 is the truth of reachability; the count
  may over-state but a forged announcement can never serve bytes it lacks.**

Cardinal invariant across all three: **heberger != publier, seeder != auteur.**
Keeping an app online never changes its author; a seeder re-signs nothing of the
provenance chain.

### §P58.1 — typed feed-op validation: a KNOWN op_type that fails to parse is malformed

`validate_feed_operation` previously validated only ops that parsed via
`try_parse_op`; an op carrying a recognised `op_type` (e.g. `"SeedAnnounced"`)
with a missing/wrong-typed field fell through to the "unknown op, size-check
only" branch and was STORED as opaque junk. Fix (Phase F, Codex C1): a
`KNOWN_OP_TYPES` set — an op whose `op_type` is known but does not parse is
REJECTED as malformed; genuinely unknown `op_type`s still pass (raw-op
forward-compat, P51). Because serde does not support `deny_unknown_fields` with
an internally-tagged enum, an EXTRA key (e.g. a smuggled payload-level `sig`,
which F-3 forbids) is enforced separately by an exact-key-set check on the
SeedAnnounced op.

### §P58.2 — BrowseEntry `is_own` via a serialize-only flatten view (no field churn)

The shell needs "did THIS node publish it" to show the owner keep-online
toggle. `project_id = blake3(name) != node_id` for per-app deploys, so the old
`isOwn = node_id===project_id` heuristic was always false. The hosting `node_id`
is already on `BrowseEntry` (`#[serde(skip)]`, set at publish/deploy and at
gossip ingest, #6). `list_browse` wraps each aggregated entry in a
`BrowseEntryView { #[serde(flatten)] entry, is_own }` computing
`is_own = entry.node_id == state.node_id` at serialization time — ZERO changes
to any `BrowseEntry { .. }` construction site. A voluntarily-seeded distant
app keeps the AUTHOR's node_id, so it is correctly `is_own = false`.

## §P59 — Sprint 75 : PULL node-centric discovery (anchor directory + multi-provider fetch)

S75 replaced PUSH-ephemeral discovery (PoW window 1800s, replayed verbatim →
apps older than 30 min invisible to fresh peers) with PULL node-centric
discovery. Patterns extracted from phases A-G:

### §P59.1 — Persist the LOCATOR, never the content (anchors.json)

iroh downloads require the CONTENT-HASH: a bare node_id gives neither hash
nor a catalog RPC. The durable artifact is a locator
`anchors.json {pubkey, ticket, revision}` re-VALIDATED (signature + revision)
at every re-fetch — the F-Droid "fingerprint persists, index is re-fetched"
shape. Remote entries stay RAM-only. The persisted `revision` is the
anti-rollback floor; the dedup rule is split by state: **RAM present →
strict `>` dedup ; RAM empty (post-boot) → floor `>= persisted`** so a
same-revision re-announce can restore the catalog after a failed re-pull
without letting an older revision roll it back.

### §P59.2 — Enforce caps INSIDE the primitive, not at call sites

Lesson from the S73 guardrail-order carry (a convention-of-caller is not a
type-level invariant), applied three times in S75: `MAX_FETCH_PROVIDERS=16`
truncates inside `fetch_hash_multi` (D); the SEED-1 freshness clamp
`seen_at = min(seen_at, now)` lives inside `SeedRegistry::record` (D); the
`[seed]` config clamp (lowercase-then-64-hex) lives at config load (E). A
new call site cannot forget a cap it never had to apply.

### §P59.3 — Normalize hex case at write AND read (registry keys)

A pubkey/hash key has 2^64 case variants; an attacker can monopolize capped
slots (SEED-2: 1024 buckets / 64 seeders) by re-announcing the same identity
in different casings. Normalize to lowercase at BOTH the write path and every
read path — one slot per identity, no case-aliased displacement.

### §P59.4 — Guardrail tripwire is a TERMINAL transition (CARRY-2)

Both result-ingress points (HTTP `coordinator_submit_result`, gossip
`validator_loop`) run the output guardrail between
`validate_result_pre_guardrail` and `validate_result_post_guardrail`. A trip
must call `reject_result_on_guardrail_trip` (task → `Rejected`): the
validated submission is already consumed, so a logged-and-return leaves a
zombie Pending/AwaitingQuorum task no future event can rescue. The quorum
`task_results` rows are kept as audit trail — terminal status makes them
inert via the pre-guardrail status gate.

### §P59.5 — Zip member injection: strip-before-inject, count caps beside byte caps

`ZipWriter::new_append` + `start_file` always APPENDS — injecting
`provenance.json` into an archive that already carries one (fork redeploy of
a blob-reconstructed workspace) stacks two same-named members, and which one
an extractor sees is implementation-defined (PULL-1: `strip_zip_member`
first, byte-identical no-op when absent, then inject; the artifact hash then
covers app content only). Byte caps (`MAX_ARCHIVE_BYTES`,
`MAX_DECOMPRESSED_BYTES`) do NOT stop a flood of tiny entries — an
entry-count cap (`MAX_ARCHIVE_ENTRIES=4096`, FORK-1) must reject BEFORE any
disk write (inode/handle-exhaustion bomb class, GHSA-j47w-4g3g-c36v).

### §P59.6 — clippy `await_holding_lock` is lexical: use a block, not drop()

clippy's `await_holding_lock` ignores an explicit `drop(guard)` (lexical
analysis only). Wrap the `MutexGuard` in a `{ }` block so the guard's scope
visibly ends before the `.await`.

### §P59.7 — Boot driver hygiene (headless anchor, E)

A config-driven boot task must be: (a) duress-gated AT THE TOP (the duress
launcher swaps the identity, not the data root — an ungated driver replays
the REAL config under the decoy keypair); (b) retained as a handle and
abort()+joined at shutdown BEFORE node reclamation (a detached
`tokio::spawn` leaves live network work after shutdown); (c) resolution
priority FROZEN by test (direct > local pin row > subscribed directories) so
a divergent directory can never override the local pin.

## §P60 — Sprint 76 Phase D : redundancy>1 deterministic quorum over the bridge + TOPLOC étage-2 note

S76 Phase D proved the `redundancy_factor > 1` deterministic quorum (D3 étage
1 palier 2) and, doing so, surfaced and fixed a production gap that silently
blocked it. The proof is by composition (no single in-process test stacks all
of it — a literal multi-worker E2E needs three iroh nodes, too heavy for the
`cargo test` shared-process gate): two hermetic two-author tests exercise the
REAL bridge + validator loop + DB (red-before-green against the fix); the
pre-existing `worker_result_syncs_into_coordinator_db_across_two_nodes` proves
genuine cross-node iroh-docs replication (redundancy=1); the literal
cross-machine run (distinct OS processes on VPS + PC + Mac) is the Phase G
LIVE acceptance.

### §P60.1 — Mirror the validator's dedup identity in the result-sync bridge

The result-sync bridge (`nexus-shell-daemon/src/result_sync.rs`,
`forward_result_entry`) forwards each replicated `result:` doc entry into
the validator loop. It keeps a `seen` set to suppress duplicate forwards.
Pre-Phase-D it keyed `seen` on `task_id` **alone** — fine for
`redundancy_factor = 1`, fatal for a quorum: a `redundancy = 2` task
receives one `result:{task_id}` entry **per worker** under a DISTINCT
iroh-docs author (`runtime.rs` writes `result:{task_id}` keyed by the
worker's own author), so the second worker's vote was dropped ("result
already forwarded") before the validator ever counted it. The task sat in
`AwaitingQuorum` forever (the B.2 early-reject only fires at redundancy≥4),
so a cross-machine quorum NEVER completed. The synchronous HTTP-submit
ingress was never affected (no such dedup), which is why the gap hid behind
the co-located path.

Fix: key `seen` on `(worker_pubkey, task_id)` — the SAME identity the
validator uses (`validator_loop` derives `worker_id = hex(worker_pubkey)`
and `insert_task_result` dedups on `(worker_id, task_id)`). The two dedup
layers now agree on "one vote per worker." Same-worker refire
(`InsertRemote` re-emit, boot catch-up overlapping the live stream) is still
suppressed; distinct workers' votes all reach the validator. **Lesson: when
two layers dedup the same logical event, they must key on the SAME identity
— a narrower key in the bridge silently defeats the wider key in the
validator.** Zero wire/dep change (daemon-internal logic).

Security: the fix opens no new surface. Before, the bridge collapsed all
workers to the first vote (quorum never formed); after, it forwards exactly
one vote per distinct `worker_pubkey`. A single worker still cannot vote
twice (same pubkey deduped at BOTH layers — locked by
`validator_quorum_unchanged`). Quorum inflation needs N distinct keypairs =
the pre-existing Sybil concern (PoW / AgeWitness mitigations elsewhere),
unchanged. The exact-match strict-majority + outlier rejection in
`validate_quorum_pre_guardrail` stays the trust boundary, INCHANGÉ (diff
verrou).

### §P60.2 — Homogeneous-redundancy exact-match; cross-GPU divergence is expected, not a bug

The quorum accepts a result when a strict majority of workers report a
byte-identical `result_text` — the mature BOINC homogeneous-redundancy
pattern (`JobReplication`: "byte for byte" + "canonical result"). It is only
useful because `verifiable` forces deterministic decoding (greedy + a fixed
seed = the `u32` LITTLE-ENDIAN truncation of the first 4 bytes of
`blake3(task_id)`, `runtime.rs` `deterministic_seed`, §P53 — locked by
`verifiable_seed_is_cross_worker_stable`). The honest limit (D2 ⚠️, written as an acceptance criterion to
defeat false-green): exact-match holds for a HOMOGENEOUS cohort
(same model/quant/runtime, the Phase C cohort gate routes these together)
and is EXPECTED to diverge across heterogeneous GPUs (float reordering —
Thinking Machines 2025-09, Ingonyama). A cross-GPU divergence is rejected as
a quorum outlier (correct), NOT silently read as a bug. The cohort gate is
advisory routing; the exact-match quorum is the real defense.

### §P60.3 — TOPLOC étage-2 (`logprobs_hash`) is the cross-hardware slot, gated on a file-exposing backend

True cross-hardware verifiable inference needs a locality-sensitive
commitment over model intermediate activations (TOPLOC, Prime Intellect,
arXiv:2501.16007: "robust across diverse hardware configurations, GPU types,
and algebraic reorderings", 258 bytes / 32 tokens). The signed
`ResultPayload.logprobs_hash: [u8; 32]` (already v1, `task.rs`) is the
reserved slot — it is currently `[0u8; 32]` ("logprobs not provided"). A
real commitment requires access to top-k hidden states, which the Ollama
HTTP backend does not expose; it is gated on a file/activation-exposing
backend (`LlamaCppBackend`, feature `llm_llama_cpp`) = S77 (étage 2). S76
Phase D adds NO code here — design note only, zero wire bump (the slot is
already v1).

**Update (S77 Phase G)**: the real commitment primitive is now delivered
(`nexus-core-rs/src/toploc.rs`, see §P64) and the consumer Layer-3 of
`verification.rs` treats `logprobs_hash` as a TOPLOC commitment by equality. The
on-wire SIGNED emission of the slot still rides the session data-plane (S78);
this note's "currently `[0u8; 32]`" describes the S76 state, not S77 Phase G.

Cross-ref: S76 Phase C (`1cc28e7`, RuntimeTuple + cohort gate), §P53
(deterministic quorum B-2), §P54 (cross-process compute E2E B-3), §P59.4
(guardrail-before-persist terminal), kickoff §D3.

## §P61 — Sprint 76 Phase E : sanity-bound plausibility-check on a self-declared, out-of-quorum reward input

Origin: the contributor dashboard credits kudos from `tokens_generated`, a
field self-declared in the signed result payload but OUTSIDE the quorum (the
validator hash-compares only `result_text`). A solo worker could farm
reputation by declaring an absurd token count. The D4-Q hardening clamps the
credited count to a plausible maximum derived from another field of the same
payload, the wall-clock `generation_time_ms`:
`tokens.min(TOKENS_PER_MS_CEILING * max(1, generation_time_ms))`, applied
inside `credit()` BEFORE `log_utility` (`kudos_ledger.rs`). OSS analogue:
BOINC CreditNew's `wu.rsc_fpops_bound` — a plausibility ceiling on a
self-declared reward input (PFC).

Three properties make this a reusable idiom, distinct from the multi-worker
agreement patterns (§P60.2 exact-match, §P53 quorum):
1. **Asymmetric bound, not attestation.** Both inputs (`tokens_generated`,
   `generation_time_ms`) live in the SAME signed payload, so an adversary who
   controls the payload can satisfy the bound by forging both consistently.
   It catches the bug and the naive over-claim, NOT the Sybil/forger — document
   it as a plausibility-check, never as an anti-Sybil defense (THREAT_MODEL §15.3).
2. **Centralize the clamp at the single credit chokepoint**, not at each call
   site, so a new caller cannot bypass it. The two prod sites
   (`validator_loop.rs`, `http.rs`) just forward `entry.payload.*`.
3. **The bounding input must be REAL end-to-end.** A latent producer bug is the
   trap: the worker hardcoded `generation_time_ms: 0`, which (with the `max(1)`
   floor) collapsed every honest credit > the per-ms ceiling to a flat value —
   degrading the existing signal instead of bounding only the absurd. The fix
   is to measure the true duration at the producer (`Instant` around the
   inference call, `runtime.rs`) and to TEST the producer path
   (`StubBackend::with_delay_ms` → assert `generation_time_ms >= 1` on the
   signed result), not only the consumer arithmetic. A clamp is only as honest
   as the value it clamps against.

Cross-ref: §P60.2 (homogeneous exact-match — agreement, not self-report),
THREAT_MODEL §15.3, kickoff §D4 (BOINC/Gridcoin/EigenTrust survey),
`sprint76_phase_e_preflight.md` (D4-Q decision: HARDEN sanity-bound,
median-of-group re-scoped P2).

## §P62 — Sprint 76 wrap-up: whole-model cross-machine task routing proven, before sharding

S76 proved the prerequisite for sharding (S77): a node can route a WHOLE model
to a worker on ANOTHER physical machine and trust the result. The chain landed
across phases C–E and is the foundation a split-model pipeline will reuse:

1. **Homogeneous-cohort claim gate (Phase C).** A `RuntimeTuple{model, quant,
   runtime_family}` rides on `Task.required_runtime` (additive `#[serde(default)]`,
   0 wire bump). The worker is a PULLER: it does NOT claim a task whose tuple
   mismatches (the task stays live for a matching worker), instead of failing it.
   The dispatcher only sets `required_runtime` when `verifiable && redundancy > 1`.
   Wildcard-on-empty keeps legacy tasks claimable. This is advisory ROUTING, not
   a trust boundary (THREAT_MODEL §15.2).
2. **Per-worker bridge dedup (Phase D).** The result-sync bridge must dedup on
   `(worker_pubkey, task_id)` — a MIRROR of the validator — not on `task_id`
   alone. With `task_id` alone, two homogeneous workers writing the same
   `result:{task_id}` key under distinct iroh-docs authors collapsed to one, so
   the redundancy>1 quorum NEVER formed cross-machine (HTTP co-located path hid
   it). The fix (`forward_result_entry`, ~5 prod lines, 0 wire/dep) was proven
   red-before-green by revert. Lesson: any dedup in front of a quorum validator
   must use the validator's EXACT key, or it silently starves the quorum.
3. **Deterministic seed is little-endian, not the digest.** `verifiable_seed` =
   `u32::from_le_bytes(blake3(task_id)[..4])` — cross-worker-stable so every
   replica picks the same sampling path; the digest itself is NOT the seed.
4. **Validator untouched.** The cohort + bridge work added zero lines to the
   quorum core (`validator.rs`); a `validator_quorum_unchanged` test + a
   `git diff --stat` are the guard. Routing/transport changes must never need a
   trust-core edit.

Tech-debt forward (S77, do not re-derive): the cross-GPU HETEROGENEOUS quorum
(same GGUF diverges on different GPUs in stock llama.cpp) needs the **TOPLOC
stage-2 commitment** — the `logprobs_hash` slot is posted but unimplemented
(§P60.3). The redundancy>1 LIVE acceptance (palier 2) runs via
`scripts/acceptance/b3_live_pc_vps.sh REDUNDANCY=2` with a 2nd homogeneous
worker; it is operator-hardware-deferred, never claimed green from CI.

Cross-ref: §P60 (.1 dedup-mirror / .2 exact-match homogeneous / .3 TOPLOC=S77),
§P53 (quorum), §P54 (cross-process E2E), THREAT_MODEL §15.2,
`sprint76_phase_{c,d}_preflight.md`.

## §P63 — Sprint 77 Phase A: iroh-docs live delivery needs a gossip-neighbor keepalive

The S76 live cross-machine attempt blocked on convergence: a `task:` entry written
onto the project doc AFTER a remote worker imported never reached the worker
(`recv:0`, "gossip neighborhood non formé"), LAN+WAN (`sprint76_verification.md`
§5.1). Root cause, read against the installed `iroh-docs-0.98.0` source (NOT the
brief's guess that "the worker never subscribes"):

1. **The coordinator's incremental broadcast is gated by `is_syncing(namespace)`**
   (`live.rs:711-718`), and `is_syncing` is inserted ONLY by `start_sync`
   (`live.rs:409-414`). The coordinator opens the project doc via
   `create_doc`/`open_doc` (sync=false) and never calls `start_sync` — it relies
   entirely on the worker's INCOMING dial forming a gossip neighbor
   (`NeighborUp -> sync_with_peer`) to flip `is_syncing` true.
2. **`DocsApi::import(ticket)` calls `start_sync(ticket.nodes)` exactly once at
   boot** (`api.rs:220-225`), seeding the dial with the addresses frozen into the
   ticket at share time. So "the worker subscribes" is already true at engine level
   — adding an app-facing `doc.subscribe()` for sync is a FALSE LEVER
   (`ToLiveActor::Subscribe`, `live.rs:334-341`, only adds a stream consumer; it
   does NOT insert the namespace into the sync-set).
3. On real transport (NAT rebind, relay change, stale ticket addrs, a hot binary
   swap over persistent `docs.redb`) that one dial does not form/maintain a
   neighbor → the namespace swarm stays empty → only the initial bulk sync
   delivers, never the incremental writes.

Fix (`nexus-core-rs/src/doc_sync.rs`, wired into the worker engine
`run_until_shutdown`): a per-doc keepalive that **observes `NeighborUp`/
`NeighborDown` and re-issues `Doc::start_sync(peers)` whenever the neighbor is
absent** (immediate on `NeighborDown`, plus a periodic backstop for a missed
initial `NeighborUp`). Passing the coordinator's `EndpointAddr` (which carries the
endpoint id) lets `presets::N0` discovery re-resolve the coordinator's CURRENT
address via pkarr instead of the ticket's frozen addrs. The read path stays
poll-based; the subscription is observability-only and drained best-effort so it
can never backpressure the boot hot path (§P54). 0 wire bump — `start_sync` is an
existing 0.98 primitive, the `task:` key and canonical bytes are untouched.

Lessons (do not re-derive):
- **`import` already does `start_sync`; the convergence lever is keeping the gossip
  NEIGHBOR alive, not re-subscribing.** Read the dep source before naming a fix.
- **An in-process 2-node test converges trivially** (the golden
  `two_nodes_docs_sync.rs` proves 0.98 is not broken), so it is a GREEN guard, not
  the red→green proof. The dropped-neighbor recovery is proven red→green by forcing
  `doc.leave()` then asserting non-delivery (control) before the keepalive re-joins
  (`doc_sync::tests::keepalive_rejoins_doc_after_neighbor_loss`). The live WAN proof
  is the `b3` cross-machine harness (T2), never claimed from in-process tests.
- **Beware the coordinator-side faux-vert**: forcing `is_syncing` on the
  coordinator (`start_sync` at boot with no peer) can pass LAN while leaving WAN
  broken. The worker keepalive is the correct lever.

Cross-ref: §P54 (cross-process E2E / hot path), §P62 (whole-model routing),
`sprint77_phase_A_preflight.md`, THREAT_MODEL §15.

## §P64 — Sprint 77 Phase G: a hash commitment BINDS a fingerprint, it does NOT make it tolerant

N0 TOPLOC (`nexus-core-rs/src/toploc.rs`) fingerprints the top-k of the last
hidden state to detect a model/precision swap while tolerating honest GPU
non-determinism — the property `verification.rs` Layer-3 lacked (hash equality on
a logprob hash). Four load-bearing decisions, do not re-derive:

1. **The 32-byte wire slot can only hold a COMMITMENT, never the tolerant
   sketch.** `logprobs_hash` / `RunProof.activation_fingerprint` are `[u8; 32]`
   (= `BLAKE3_BYTES`); a TOPLOC encoding is 258 B/32 tok. The plan's "0 bump wire"
   forces a BLAKE3 commitment of the canonical integer encoding into the slot. A
   hash DESTROYS locality (one bit flip avalanches), so the slot binds the
   model-swap by EQUALITY and detects it by inequality — it can never be the
   tolerant comparator. The tolerant exponent/mantissa compare
   (`ToplocFingerprint::compare`) needs the full sketch on both sides; that is the
   "separate off-canonical payload" the old Layer-3 doc-note anticipated, and it
   rides the data-plane in N1/N2 (S78). Do NOT claim a 32-byte slot resolves
   cross-GPU tolerant verification.

2. **Sketch-direct over the GF(65497) polynomial (a verdict-sanctioned
   PLAN-ADAPT).** TOPLOC's native 258 B compression is a Newton interpolation over
   a finite field with `mod_inverse` + an injective-modulus search — and `y mod
   65497` aliases negative-activation bf16 patterns in `[65497, 65535]`. None of
   that buys anything while only the 32-byte commitment is on the wire (Phase G),
   so SBFB stores the direct integer sketch (indices `u32` + bf16 bits `u16`,
   index-sorted) for auditability. The GF compression is a deferrable on-wire
   optimisation for H/I if payload size ever matters.

3. **The hashed pre-image must be ALL-INTEGER or the commitment never matches
   cross-platform.** The float top-k values are quantised to bf16 bits
   (`(to_bits() >> 16) as u16`, a bit reinterpret — language semantics, identical
   on any endianness) BEFORE the encoding. No `f32` is ever serialised; a Rust
   signer and a Python verifier derive identical bytes. Same rule as `RunMetrics`'s
   all-integer fields and JCS no-float.

4. **Tolerant comparison stays integer even locally.** `mean < T` is evaluated as
   `sum < T * count`, `median < T` as `median_x2 < 2*T` (the even-length midpoint
   is `errs[n/2-1] + errs[n/2]`), and an empty exponent-match set is a `u64::MAX`
   sentinel reject. Thresholds (`TOPLOC_THRESH_EXP_MISMATCH=38`, mean 10, median 8)
   are named consts from arXiv 2501.16007v2 (bf16 set); rig re-calibration is
   S78. A `compare()` whose call-sites are all `#[cfg(test)]` is correct for
   Phase G: the primitive is exposed for the verifiers, NOT wired in-vivo.

**Auto-attestation (cardinal, mirror `task.rs` `model_digest`)**: a commitment a
worker computes for its own run is a self-claim, never proof, until an independent
verifier (N1/N2) recomputes it. Phase G delivers the primitive + the worker-side
helper that COMPUTES the commitment; writing it into a signed `RunProof` and
emitting it on-wire rides the session data-plane (S78). The live result path
stays the hash-exact `result_text` quorum, unchanged.

Cross-ref: §P60.3 (the slot was reserved here), §P62 (whole-model routing),
`sprint77_phase_g_preflight.md`, THREAT_MODEL §16 (N0).

## §P65 — Sprint 77 Phase H: a verifiable DRAW (not an ECVRF) + tolerant recompute + Token-DiFR for the N1 spot-check

N1 (`nexus-core-rs/src/verifiable_draw.rs` + `nexus-coordinator-rs/src/rerun.rs`)
picks which worker re-runs a ~1% prefill to audit a peer's N0 commitment. Five
load-bearing decisions, do not re-derive:

1. **The selector must be UNPREDICTABLE to the verified worker.** The Sprint 40
   `simple_hash(task_id) = BLAKE3(task_id)` selector is publicly computable, so a
   worker knew ex-ante whether it would be spot-checked and cheated only when
   unwatched. The fix signs a draw `seed` with the node key; the signature is the
   "proof", `BLAKE3(domain || proof)` the "output". A worker cannot precompute the
   output of a key it does not hold. Replace predictable selectors, never restore
   them.

2. **A deterministic Ed25519 draw is NOT an ECVRF (RFC 9381) — say so verbatim.**
   Ed25519 is malleable (a third party can derive another valid signature for the
   same message), so draw UNIQUENESS is not proven; Ed25519 is not a PRF, so
   UNPREDICTABILITY is not proven. This is a MITIGATION under one-honest-verifier
   for a 1-5% sample, not a guarantee. The 0-dep reuse of `crypto.rs` (precedent
   Phase D `blake3(session_id||pubkey)`) is assumed against a heavyweight ECVRF
   crate on a second curve. Over-claiming "VRF guarantees fair selection" is the
   exact over-promise the preflight flags (recurring S77: SI-3 doc-overstate
   Phase E, doc-honesty Phase G).

3. **The draw `seed` must be data the verified worker CANNOT choose.** Use
   `session_id || epoch || result_commitment` (all already signed). A seed the
   worker controls lets it grind the draw to steer a colluding verifier
   (THREAT_MODEL §16, surface "grinding"). Tested: `vrf_verify` rejects a tampered
   seed/key/proof.

4. **Recompute is TOLERANT (delegates to `ToplocFingerprint::compare`), never
   byte-equality, AND checks tokens (Token-DiFR).** A commitment-equality N1 would
   false-reject every honest cross-GPU re-run (§P64 item 1, §P60.2). And comparing
   ONLY the activation fingerprint lets a worker forge tokens then back-compute a
   matching fingerprint — so the verdict also requires output-token agreement
   under the SHARED VRF-derived seed (DiFR >98% match a fixed seed; we require
   `TOKEN_AGREEMENT_PCT=95`). temp+seed are derived deterministically from the
   draw output (`derive_spotcheck_temp_milli`/`derive_spotcheck_seed`) — milli-unit
   integers, the worker floats only at the GPU boundary (cf. §P64 item 3).

5. **Incentive is reputational and GATED; sanction is non-economic.** A drawn,
   proven verifier earns kudos via the EXISTING `kudos_ledger::credit` (there is no
   `curator` module; the kudos ledger IS the reputation). `spotcheck_creditable`
   requires (a) a re-verified VRF draw AND (b) a valid SIGNED `RunProof` from that
   verifier — never a self-declaration. Do NOT add a `reason=spotcheck` field to
   `HashableKudosEntry` (it would change the `DOMAIN_KUDOS_V1` pre-image = a silent
   wire bump). Sanction of a lazy/false verifier is non-credit / negative trust
   delta on the prover path — NEVER slash/bond/burn (PO-12 kudos invariant;
   VeriLLM's slashing-based game theory is forbidden here, so there is no
   anti-lazy-verifier defense — carry it honestly).

**Scope (mirror Phase G)**: Phase H delivers the PRIMITIVES (draw, tolerant
compare, Token-DiFR, credit gate, criticality mapping) + hermetic tests. The real
GPU prefill re-execution and the full-sketch transport (off the 32-byte
binding-only slot) are gated to S78. Criticality →
level (`criticality_maps_to_verification_level`) derives from `Task.verifiable`
(SIGNED — part of the canonical identity) and `Task.redundancy_factor` (NOT
signed — a dispatch policy excluded from the canonical bytes since Sprint 23
`34c77ce`), so the returned level is ADVISORY w.r.t. redundancy; the BINDING
minimum level is set by the consumer/group policy, never trusted from the
unsigned hint nor self-declared (auto-downgrade defense), and the N1 lottery
applies regardless of the criticality tag. 0 bump wire (1 additive
`DOMAIN_VRF_DRAW_V1`, slots already v1), 0 new dependency.

Cross-ref: §P64 (N0 commitment binding), §P61 (out-of-quorum reward inputs),
`sprint77_phase_h_preflight.md`, THREAT_MODEL §16 (N1 + Incentive).

## §P66 — Sprint 77 Phase I: N2 tolerant quorum is a CLIQUE, N3 is TWO primitives (commit-reveal ≠ SENTINEL), and "O(1)" is not "bisection"

Phase I wires N2 (`nexus-core-rs/src/redundancy.rs` + additive
`nexus-coordinator-rs/src/validator.rs`) and N3 (`activation_commit.rs` +
`sentinel.rs`). Load-bearing decisions, do not re-derive:

1. **N2 agreement is a CLIQUE of mutual tolerance, NOT a pivot star.** Tolerant
   agreement is not transitive: `A ≈ B` and `B ≈ C` does not give `A ≈ C` (a
   "straddling" fingerprint sits in the tolerance band of two mutually-divergent
   ones). Counting how many agree with one pivot over-counts — a lone straddler
   inflates a quorum that does not exist. `largest_agreeing_cluster` computes the
   maximum clique of the symmetric agreement graph (`fingerprints_agree` = both
   `compare` directions, since `ToplocFingerprint::compare` is directional). Bound
   the input (`N2_MAX_FINGERPRINTS`) — max clique is NP-hard, but a redundancy
   fan-out is single-digit. Reuse `TOPLOC_THRESH_*` (cf. §P64/§P65), never invent a
   second tolerance threshold.

2. **N2 is ADDITIVE; the exact `result_text` quorum is byte-for-byte UNCHANGED.**
   `validate_quorum_pre_guardrail` (the homogeneous exact-match path, §P60.2/§P53)
   is not edited — N2 is a separate function `validate_tolerant_quorum_shard` over
   fingerprints, never `result_text`. Wrap-up sentinel: `git diff` of the quorum
   body = 0 lines, plus a behavioural test that the exact quorum still
   accepts-on-majority / rejects-on-divergence (`validator_exact_quorum_unchanged`).

3. **N2's ACCEPT/REJECT rests on SIGNED inputs; selection is advisory.** *Which*
   tasks use N2 comes from `criticality_maps_to_verification_level`, ADVISORY
   because `redundancy_factor` is unsigned (S23 `34c77ce`, cf. §P65). The verdict
   only votes on submissions whose `RunProofEntry` signature verifies AND whose
   carried full sketch opens the signed N0 commitment
   (`sketch.commitment() == proof.activation_fingerprint`) — the off-slot sketch
   carrier (the comparable the 32-byte slot cannot hold, §P64 item 1) cannot be
   tampered, and a forged/unsigned proof never reaches the vote.

4. **N3 is TWO orthogonal primitives — do not fuse them.** The plan read "bisection
   opML + SENTINEL O(1 bloc)" as one mechanism; it is two. (a) `activation_commit`
   = opML-style commit-reveal: a worker signs `BLAKE3(sketch || nonce)` per
   frontier (`DOMAIN_ACTIVATION_COMMIT_V1`), and on dispute reveals the full sketch
   + nonce. (b) `sentinel` = a statistical forward-EMA monitor that localises
   *which* frontier to dispute. A true opML bisection is interactive and **O(log
   L)**; SENTINEL's direct per-frontier flag is **O(1)**. "Bisection O(1)" is an
   oxymoron — the O(1) comes precisely from the ABSENCE of a search. Name the two
   primitives separately and never claim opML fraud-proof soundness (SBFB has no
   bit-exact deterministic VM — the open question that motivates TOPLOC's tolerant
   compare in the first place; a cryptographic guarantee is N4 zkML, out of scope).

5. **The N3 reveal verdict is the TOLERANT compare, NEVER commitment equality.**
   Two steps: binding (`reveal.opens(committed)` — does `BLAKE3(sketch||nonce)`
   match the committed value?) THEN correctness (`verifier_recompute.compare(reveal
   .sketch).accepted`). Comparing the 32-byte commitments by equality would
   false-reject every honest cross-GPU reveal (BLAKE3 avalanche, §P64 item 1). The
   nonce binds the frontier `(session_id, frontier_index, worker_pubkey)` into the
   signed pre-image (anti-grinding/replay, same discipline as the N1 seed §P65
   item 3) and hides the fingerprint before reveal.

6. **SENTINEL is forward-only and integer; a flagged outlier does NOT update the
   baseline.** The paper (arXiv:2603.03592) is a TRAINING detector (forward +
   backward gradients); inference is forward-only, so only the forward-activation
   half is portable — do not replicate the gradient half. The EMA runs in integer
   basis points (`ema_step`, `SENTINEL_ALPHA_BP`), `bf16_bits`-style: no float on
   the wire or in the decision (cf. §P64 item 3). A flagged frontier does not fold
   into the EMA (outlier rejection, anti-spike-poisoning); the static threshold +
   slow-drift evasion (SI-11) is a DISCLOSED carry (adaptive IQR fence + absolute
   magnitude signal = S78), exactly like the N1 anti-lazy-verifier gap (§P65
   item 5). Sanction of a localised corrupt frontier is a correctness/reject
   verdict, NEVER a slash (PO-12).

0 bump wire (1 additive `DOMAIN_ACTIVATION_COMMIT_V1`, `*_FORMAT_VERSION` already
v1), 0 new dependency, no-float core. Scope (mirror Phase G/H): Phase I delivers
the PRIMITIVES + hermetic tests; the real cross-GPU in-vivo recompute, the
data-plane sketch transport, and the dispute arbitration loop are gated to S78
(Phase K is the wrap-up, 0 functional code).

Cross-ref: §P65 (N1 draw + tolerant recompute), §P64 (N0 commitment binding),
§P60.2 (homogeneous exact-match vs cross-GPU divergence), `sprint77_phase_i_preflight.md`,
THREAT_MODEL §16 (N2 + N3).

## §P67 — Sprint 77 (B/F2): the `sbfb/shard/1` data-plane forwards boundary frames over a bi-stream with admission, but has NO generation orchestrator; integrity is OUT-OF-BAND

The shard data-plane (`nexus-core-rs/src/shard.rs` + worker
`nexus-worker-core/src/llm/shard.rs`) is deliberately minimal. Load-bearing:

1. **A SEAM trait keeps the layer-block backend off the transport.**
   `ShardForwarder` is defined in `nexus-core-rs` (where the ALPN + QUIC live);
   `EchoForwarder` (preserves the Phase B echo) and the feature-gated
   `ShardBackendForwarder` (real llama.cpp layer-block) implement it. The
   transport calls the trait; the layer-block backend never sees an iroh
   `Connection` — it is pure compute over a decoded frame. `nexus-worker-core`
   DOES link `nexus-core-rs` (the worker owns its own iroh node), but adding the
   real backend (F2) added NO new iroh surface to the forwarder itself.
2. **Admission is crypto-BEFORE-IO.** `accept` verifies the peer `is_member` of
   the signed `ComputeGroup` before reading the frame; the worker claim
   (`shard_claim.rs::authorize_claim`) runs `verify_sig -> is_member -> in-plan`
   BEFORE any GGUF load, and `assess_capacity` pre-validates the layer window
   against MEASURED VRAM with a fail-CLOSED `is_degenerate_geometry` guard (a
   model whose head/embed geometry reads as 0 is REFUSED, never loaded
   fail-open). Cap the frame (256 MiB) before alloc.
3. **It serves a long-lived bi-stream; it does NOT orchestrate generation.**
   `accept` admits the peer, then loops (`while let Some(frame) = read_frame`)
   forwarding EACH inbound boundary frame through its layer block
   (`self.forwarder.forward(&frame)`) and writing the output downstream — that is
   transport, not an autoregressive token-generation driver. There is NO
   autoregressive decode loop, NO TTFT/tok-s metric, NO `RunProof` emission here.
   `open_shard_connection` documents a "caller" that drives the generation over
   that bi-stream — that production ORCHESTRATOR does not exist yet (S78 carry).
   Do not read the frame forwarder as an end-to-end sharded-generation pipeline.
4. **Integrity rides OUT-OF-BAND.** The transport proves nothing about the
   computation; the `RunProof` (N0-N3, §P64-66) carries the verdict. A lying
   shard is caught by the verification primitives, not by the frame protocol.

0 bump wire (ALPN string + raw frames; `DOMAIN_SHARD_PLAN_V1`/`DOMAIN_RUN_PROOF_V1`
additive §P64). Cross-ref: §P68 (placement), §P69 (routing/churn), THREAT_MODEL
§16 + §5.9, `sprint77_phase_f2_preflight.md`.

## §P68 — Sprint 77 Phase D: Parallax placement is INTEGER water-filling + deterministic k-medoids on MEASURED signals

`nexus-coordinator-rs/src/placement.rs` splits a model across workers. Load-bearing:

1. **Water-filling is INTEGER largest-remainder, proportional to MEASURED
   `vram_free_bytes`.** No float: layers are apportioned to each worker's measured
   free VRAM (`GpuStats`), remainder by largest fractional part with a pubkey
   tie-break. The sharding THRESHOLD is explicit — shard ONLY when the quantized
   model does not fit the single largest `vram_free` (`> max(...)`), else
   `EndpointFederation` (route the whole model). Refusing to shard a model that
   fits one machine is a tested invariant (`placement_refuses_when_model_fits_single_worker`).
2. **k-medoids is PAM BUILD+SWAP, fully DETERMINISTIC (0 rand).** Grouping
   low-RTT workers uses the classic PAM passes over a pairwise RTT matrix built
   from MEASURED `conn_rtt` (Phase B), NOT a stale `conn.stats()` snapshot and NOT
   geo-IP. Ties break on pubkey so two coordinators reach the SAME plan.
3. **Coverage is checked, not assumed.** `covers_full_model` requires
   `is_pipeline_contiguous` AND the union of windows = `[0..L)` exactly; a plan
   with a gap or overlap is rejected (`covers_full_model` in `placement.rs`; the
   contiguity primitive `is_pipeline_contiguous` is delegated to `shard_plan.rs`),
   never dispatched.
4. **Sybil-tail sampling is deterministic, non-lexicographic.** When more seeders
   than slots crowd the tail, selection is `blake3(session_id || pubkey)` ordered
   (anti lexicographic-crowding, reproducible) — closes SYBIL-SEEDER-TAIL (S75).

Internal types are non-wire (placement is a computation). 0 bump wire, no-float.
Cross-ref: §P67 (data-plane), §P69 (routing), `sprint77_phase_d_preflight.md`.

## §P69 — Sprint 77 Phase E: min-latency DAG routing + ACTIVE churn, with an UNSIGNED raw-op perf-map

`nexus-coordinator-rs/src/routing.rs`. Load-bearing:

1. **Routing is a deterministic DP sweep; `tau` is ADVISORY.** `route_min_latency`
   sweeps the pipeline DAG (`saturating_add`, pubkey tie-break) over MEASURED link
   cost `rho`; the per-stage compute `tau` is ADVISORY (it informs routing, never
   integrity). SI-3: churn re-orders by measured `rho` ALONE (`fallback_link_cost`
   EXCLUDES `stage_tau`).
2. **Churn is ACTIVE and bounded.** `replace_failed_server` is O(R) per stage
   (independent of pipeline length L); `assign_fallback_nodes` peoples
   `fallback_node` AT PLAN time and re-signs `revision+1`; the
   `ActivationReplayCache` is bounded (`ACTIVATION_REPLAY_CACHE_MAX`, oldest-first
   eviction). SI-4: fallbacks are allowlist-only members of the signed
   `ComputeGroup`, never an anonymous peer.
3. **The perf-map is an UNSIGNED raw-op of INTEGER micros.** `PerfMap` rides
   iroh-docs as a `serde_json::Value` raw-op (`PerfMapWire`), UNSIGNED (advisory
   telemetry, not a trust input), integer microseconds (no float on the wire). Cap
   BOTH `rho` and `tau` (DoS) BEFORE building the `BTreeMap`. As a raw-op it adds 0
   `DOMAIN_*` and 0 `FEED_FORMAT_VERSION` bump (pre-launch raw-op policy).

0 bump wire, 0 new iroh dep (crate-boundary split: the glue `doc.set` lives in the
daemon). Cross-ref: §P67, §P68, `sprint77_phase_e_preflight.md`, THREAT_MODEL §16.

## §P70 — Sprint 79 Phase B: the docs-contract cadence (generated étiquette drift-gated per phase, GUIDE at closure, provenance edges to the immutable past)

Origin: doctrine `.planning/research/doctrine_contrat_pour_llm.md` (§2 the 5
layers, §7 the gate-map). S79 is the first concrete instance. This is a PROCESS
STANDING RULE — every future sprint follows it, not just Factory — canonised here
+ in `docs/claude/README.md` §6.12 + `docs/agent/AGENT_SYSTEM.md`.

A *frontier* primitive is one read by an actor that is NOT the code: another node
(wire), an external client (API), a network app (app-contract / CSP), another LLM
(prompt-kind / knowledge). A purely internal helper is NOT a frontier — code +
tests suffice. The 5 layers of a frontier contract:

1. **CODE** — the behaviour (last-resort authority).
2. **ÉTIQUETTE** (generated schema, drift-gated) — the contract shape. Cadence:
   **PER PHASE**, in the commit of the primitive. FREE (the schema is generated)
   and un-rottable (drift → red build). Incarnated by `schema_for!` snapshots
   (8 sharding types + `TaskResponse`), `BRIDGE_METHOD_ALLOWLIST` Rust↔TS parity,
   Zod `.strict()`, the knowledge-pack `MANIFEST.json` per-file hash manifest
   (it records a blake3 per layer and excludes itself from the hash set; S79 A; F to come).
3. **COMMIT** — why/when/delta (attributable, 9-section body, signed).
4. **GUIDE + `llms.txt`** — the navigable index. Cadence: **ONE closure phase**
   (the full picture is only freezable at the end — mirror of S77 Phase N).
5. **Provenance edge in-code** (rank-1 comments) — code↔decision links. They
   point ONLY at the **immutable past** (a sprint/phase/§/decision that HAS
   happened), NEVER a future promise.

**Anti STALE-PHASE-K (cardinal lesson).** A provenance comment promising future
work anchored to a phase/sprint rots into a lie. Real incident: an S77 `http.rs`
comment promising the live shard store for a later phase survived that phase's
close (the store is an S78 carry). S79 Phase B scrubbed these comments across
`crates/` and the `web/src/` shard-session UI (doc-comments only, 0 behaviour
change) and gated recurrence: `scripts/check-frontier-contracts.sh` fails CI on a
phase/sprint/wave token adjacent to a future verb (the "Phase X will|adds|ships",
"lands [in] Phase X", "arrive en Phase X" [FR], "Sprint N will", "Wn[.n]
will|introduce", "inert until Phase", "will land in|with" forms), scoped to
`crates/` + `web/src/` (docs/ describe the anti-pattern verbatim → out of scope
by construction). The pattern is ANCHORED so it never fires on generic prose
("the values the consumer will read", "node A adds a blob", "a future sprint
adds a field"). Two classes are intentionally NOT gated (uncatchable without
false-positives on legit prose) and are caught by review, not the gate:
parenthetical forms ("(Phase K)") and non-adjacent forms where the future verb
is separated from the phase/sprint token ("the Sprint 4 coordinator will rely").

**The `// FRONTIER:` registry (opt-in, INCREMENTAL).** A type opts in with
`// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION`. The gate then
requires its domain + version consts to resolve AND a generated schema
(`schema_for!(<name>)`) OR an explicit `// FRONTIER-NO-SCHEMA: <name> <reason>`.
UNannotated types are NOT violations — the registry grows one primitive at a time
(S79 annotates `ShardPlan` as the first dogfood entry; 22 of the 25 `DOMAIN_*_V1`
families have no generated schema and stay unannotated, a tracked carry routed to
the S80 audit-plan (created at sprint closure)). This is
the doctrine §7 Q2 "explicit registry, opt-in" arbitration (PO-tranché).

**The cadence in one sentence**: generated étiquette PER PHASE in the primitive's
commit; GUIDE + `llms.txt` in ONE closure phase; provenance edges point only
backward at the immutable past. Neither "one doc phase per phase" (heavy) nor
"all docs at the end" (false mid-sprint). Truth-Stack for the GUIDE layer
(canonical form): `repo files > .planning/active/ > commits > prompts > chat`,
with "Not evidenced" for anything outside ranks 1-4.

Cross-ref: `docs/claude/README.md` §6.12 (process rule), `docs/agent/AGENT_SYSTEM.md`
(portable doctrine + Truth-Stack), `scripts/check-frontier-contracts.sh` (the
gate), `scripts/check-sharding-docs.sh` (the per-subsystem GUIDE lint it clones),
doctrine `.planning/research/doctrine_contrat_pour_llm.md`.

## Note — META-1 rule: a Codex GAP at commit time must be a DISCLOSED, TRACKED carry

Origin: S74 Phase D was committed while its Codex review still carried an
unresolved `GAP` verdict (`sprint74_phase_d_codex_review.md`). The commit body
DISCLOSED both gaps and routed them (one closed in Phase G, one re-routed
S75) — that is the acceptable shape. The RULE (S74 audit META-1, logged S75
Phase G): committing over a Codex `GAP` is allowed ONLY when (1) the commit
body `## Codex verification` section names each unresolved GAP explicitly,
(2) each GAP has a routed owner (a later phase of the same sprint, or the
next sprint's audit plan), and (3) the phase review.md documents the
reconciliation decision. A silent commit over a GAP — body says "0 GAP" or
omits it — is a P1 process violation (lightcheck Check 7 enforces the
artifact side; this rule covers the routing side).

## Sprint 77 audit gate — tech debt (2026-06-24, verdict PASS)

Logged by `sprint77_audit_findings.md` (Cas A audit gate, 11-track Workflow).
0 P0 / 0 P1 ; these are the rust-side P2 carries into the S78 ledger.

- **PAT-1 (P2) — §P69 overstates the perf-map daemon glue as shipped.** §P69
  (PATTERNS.md §P69 + `nexus-coordinator-rs/src/routing.rs:30-43`) narrates in the
  present tense that the perf-map republish + `is_member` ingest gate `doc.set`
  "lives in the daemon" / "the daemon owns the doc handle". That wiring exists
  NOWHERE — there is 0 daemon caller of `routing.rs`/`PerfMap` (the routing/churn
  primitives are correct and behave as claimed; only the daemon-side glue is
  unwired). Consistent with SHARD-PROVISIONAL (no orchestrator consumes it), so
  not a false-green — but §P69 lacks the explicit "not-yet-wired (S78 carry)"
  disclaimer that §P67 point 3 applies correctly. Fix: mark the perf-map
  republish + ingest gate as an S78 seam. Doc-only, 0 code.
- **CARRY-1 (P2) — SEEDER-DIAL-TAIL residual (SYBIL-SEEDER-TAIL mis-credited).**
  §P68 point 4 + `sprint78_audit_plan.md §3` declare "SYBIL-SEEDER-TAIL clos", but
  the blake3 anti-crowding sampling was applied to the WORKER shard-placement tail
  (`placement.rs:309-350`), NOT to the SEEDER dial-set the S75/S76 carry originally
  named. `seed_registry.rs:331 seeders_recent` still does plain `ids.sort()`
  (lexicographic), capped in `directory_pull_providers` (`http.rs:1709-1743`, whose
  own comment still says "carried to the S76 audit"). Availability-only (BLAKE3
  content-addressing keeps integrity intact — a crowding Sybil costs failed dials,
  never wrong bytes), so NOT P0/P1. Fix: track SEEDER-DIAL-TAIL explicitly in the
  S78 ledger, OR re-scope §P68 to "closes the WORKER-PLACEMENT Sybil tail only",
  OR apply the same `blake3(context||pubkey)` sampling to `seeders_recent`.
- **HARD-2 (P2) — TEST-ISOLATION-SBFB-HOME (pre-existing, not an S77 regression).**
  The e2e daemon-spawn tests (`nexus-shell-daemon/tests/e2e.rs`, last touched
  Sprint 10, 0 S77 delta) set `.env("NEXUS_GRID_ROOT", tmp)` but NEVER `SBFB_HOME`,
  so they share the global `$HOME/.sbfb` (`auth.rs:73 sbfb_home()` falls back to
  `$HOME/.sbfb`). Masked on real CI/dev by an existing `~/.sbfb`; surfaces as an
  `auth_token` race in a fresh parallel Docker-on-Windows nextest (the 6 failing
  iroh-networked tests in the canonical-Docker S77 audit run). Root-cause fix: add
  `.env("SBFB_HOME", tmp)` (or a per-test TempDir) alongside `NEXUS_GRID_ROOT` on
  every daemon-spawn e2e test. Carry S78.
