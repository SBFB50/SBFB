# Audit S4 — node.rs

**File**: `crates/nexus-core-rs/src/node.rs` (287 LOC)
**Date**: 2026-04-10

---

## Conforme

- `Endpoint::builder(presets::N0)` + `.secret_key(sk)` + `.bind().await` — exact iroh 0.97 API (endpoint.rs:478, 203). Confirmed.
- `Router::builder(endpoint).accept(alpn, handler).spawn()` — exact iroh 0.97 API (protocol.rs:344, 403, 420). Confirmed.
- All three ALPNs registered: `BLOBS_ALPN`, `GOSSIP_ALPN`, `DOCS_ALPN` (lines 200–204).
- Stable identity: same secret key → same `endpoint.id()` (deterministic Ed25519 pubkey). Test `persistent_secret_key_reboots_with_same_id` confirms it.
- Accessors present: `docs()`, `gossip()`, `blobs_store()`, `endpoint()`, `node_id()` (lines 96–127).

## Manquant

- No `endpoint_id()` alias — only `node_id()`. The audit spec asked for both. Minor omission.
- No `blobs()` shorthand (returns `MemStore` via `blobs_store()`, not a `Blobs` handle). Plan mentions a future `blobs.rs` wrapper; acceptable for S4.
- No pkarr discovery auto-start (plan day 1-2 mentions it; deferred to `discovery.rs`).

## Déviations

- Struct named `Node` (not `FullNode` as the audit brief suggested). This is fine — matches the public API in `lib.rs`.
- `Node::shutdown` calls `drop(self.router)` (line 140) instead of `router.shutdown().await`. See **Bugs** below.

## Qualité

Doc-comments thorough. `Debug` impl delegates to `node_id()`. `NodeConfig` is `Clone + Default`. Code is clean.

## Tests

4 node tests, all pass: `create_node_returns_a_usable_handle`, `two_nodes_have_distinct_identities`, `persistent_secret_key_reboots_with_same_id`, `node_exposes_protocol_stack_handles`. Full suite: 7/7 ok in 4.71s.

## Bugs (DO NOT FIX)

**[CRITICAL] Shutdown ordering — non-graceful router drop (line 140)**

`Node::shutdown` calls `drop(self.router)` instead of `router.shutdown().await`. Per iroh 0.97 `protocol.rs:63-66`, `Router` carries an `AbortOnDropHandle` — dropping it aborts the run-loop task immediately, skipping the graceful sequence: `protocols.shutdown()` → `handler_cancel_token.cancel()` → `endpoint.close()`. The subsequent explicit `self.endpoint.close().await` (line 141) then races with the aborted task's own `endpoint.close()`. The correct call is `self.router.shutdown().await` (which internally calls `endpoint.close()`), making the outer `endpoint.close()` redundant and potentially double-closing. In practice tests pass because the race window is small in loopback tests, but under load this will leak in-flight streams.
