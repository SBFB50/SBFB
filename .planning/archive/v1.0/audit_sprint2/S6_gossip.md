# Audit S6 — gossip.rs

**File**: `crates/nexus-core-rs/src/gossip.rs` (296 lines)
**iroh-gossip pinned**: 0.97.0 (Cargo.lock `4db5b64f`)

---

## Conforme

- `GossipClient::join_topic([u8;32], Vec<String>) -> Result<TopicHandle>` wraps `subscribe_and_join(TopicId, Vec<EndpointId>)` correctly. Actual API signature confirmed from local registry: `subscribe_and_join(&self, topic_id: TopicId, bootstrap: Vec<EndpointId>) -> Result<GossipTopic, ApiError>`. `EndpointId` is a type alias for `PublicKey` (iroh-base-0.97.0/src/key.rs:64), so the `PublicKey::from_str` parse at line 138 is correct.
- `TopicHandle::split()` calls `GossipTopic::split() -> (GossipSender, GossipReceiver)` — matches actual API exactly (api.rs line 207).
- `GossipSender::broadcast(Bytes)` — wrapper correctly converts `Vec<u8>` → `Bytes::from(message)` before calling the underlying method.
- `GossipReceiver` is a `Stream<Item=Result<Event, ApiError>>`; `try_next()` is supplied by `futures_lite::StreamExt` (imported at line 47), which implements `try_next` for any `Stream<Item=Result<T,E>>`. Compiles and works correctly.
- `GossipEvent` enum maps all four `iroh_gossip::api::Event` variants: `Received` → `Message`, `NeighborUp`, `NeighborDown`, `Lagged` (lines 88–99). Complete.
- `Lagged` event is explicitly handled and documented as a signal to tighten the processing loop (line 81).
- Drop-to-unsubscribe: `GossipTopic` and its split halves implement `Drop` implicitly — iroh-gossip leaves the topic once both `GossipSender` + `GossipReceiver` are dropped. `TopicHandle` owns the `GossipTopic`, so drop is clean. No explicit unsubscribe API needed or expected.

---

## Manquant

- No `broadcast_neighbors` method exposed (iroh-gossip provides `GossipSender::broadcast_neighbors` for neighbor-only flood). Not in plan scope; noted for completeness.

---

## Déviations

- `GossipClient` holds `&'a Gossip` (lifetime-bound reference, line 105). Every call site must ensure the `Node` outlives the `GossipClient`. The Python PyO3 binding layer will need to manage this carefully — binding a lifetime-bearing struct to Python is non-trivial. The PyO3 S9 binding currently wraps `Node` in `Arc`; `GossipClient` should probably be constructed internally and not exposed as a long-lived Python object.

---

## Qualité

- API is ergonomic from Python: single `join_topic` call returns a handle; `broadcast` and `next_event` are straightforward async methods.
- `Lagged` handling is documented but not actionable at this layer (no buffer-resize or reconnect path). Caller must re-subscribe manually after lagged — acceptable for Sprint 2.
- Error messages include context strings (e.g. `"bad bootstrap node id {s:?}: {e}"`). Good.

---

## Tests

```
cargo test -p nexus-core-rs --lib gossip
running 2 tests
test gossip::tests::broadcast_rejects_invalid_bootstrap ... ok
test gossip::tests::join_topic_returns_a_handle ... ok
test result: ok. 2 passed; 0 failed
```
2-node message exchange test deliberately deferred to Sprint 4 — not a bug.

---

## Bugs (DO NOT FIX)

1. **gossip.rs:104–107** — `GossipClient<'a>` is `Copy` + lifetime-bound. This lifetime will create friction in PyO3 bindings (Sprint 9 currently wraps `Node` in `Arc` but does not expose `GossipClient` directly). If a `GossipClient` is ever stored in a Python object, the borrow checker will reject it. The field should be changed to owned `Gossip` (cloneable via `Arc` internally) rather than a borrowed `&'a Gossip`.
