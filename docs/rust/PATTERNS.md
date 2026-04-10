# Rust patterns for SBFB

_Scratchpad for Sprint 1 learning. Fill in as you go, no pressure._

This file is the personal notebook for the Rust learning work in
Sprint 1. The goal is not to rewrite the Rust Book — it is to
capture the handful of patterns you will hit over and over again
while writing `nexus-core-rs`, `nexus-core-py` and `nexus-worker`.
When something clicks or bites, note it here. When Sprint 2 starts
you will come back to these notes instead of re-learning.

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
