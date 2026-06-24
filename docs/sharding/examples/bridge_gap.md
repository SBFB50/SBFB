# Example — the bridge has NO shard method (GAP-not-shipped)

Status: **GAP-not-shipped.** Any shard method on the postMessage bridge is
**PROPOSED** only — no name or shape is decided; the decision is deferred to
Sprint **S78**. Do not pre-wire one.

## What exists today

The postMessage bridge that sandboxed apps use
([`../../../web/public/sbfb-bridge.js`](../../../web/public/sbfb-bridge.js)) has a
**closed whitelist** of app-facing methods, validated host-side by the
`BridgeMethodSchema` enum in
[`../../../web/src/bridge/protocol.ts`](../../../web/src/bridge/protocol.ts) — that
enum is the canonical, exhaustive list (this doc is a snapshot; trust the enum).
Today it covers task submission/results, P2P storage, PII redaction, identity /
node status, browse, provenance, feed cursor, full-text search, and proof cards:

```
task_submit, task_result, pii_redact,
storage_get, storage_set, storage_list, storage_delete, storage_version,
identity_pubkey, node_status, browse_list,
provenance_get, provenance_verify, feed_cursor_get, search, proof_card_get
```

**Not one of these touches sharding.** An app running in its iframe **cannot**
start or join a shard session through the bridge — there is no method to call, and
the host rejects any unknown method at the schema boundary.

## Why — and where the real entry point is

Initiating a shard session is a **node-privileged action**, not an app capability.
The entry point is the shell `/compute` panel
([`../../../web/src/components/ShardSessionPanel.tsx`](../../../web/src/components/ShardSessionPanel.tsx)),
which an app cannot reach from inside its sandbox. This keeps the private
compute-group composition off the untrusted iframe surface entirely.

## If you are an agent wiring this

An illustrative placeholder method — call it `shard_observe` — would be
**PROPOSED, not shipped** (nothing about it is decided):

```jsonc
// ILLUSTRATIVE PLACEHOLDER — NOT a proposed contract. The name `shard_observe`
// and this shape are invented for this example; nothing is decided or frozen.
// DO NOT implement against it. The bridge whitelist is the closed enum in
// protocol.ts — none of its methods is a shard method; adding one is an
// explicit S78 design decision.
{ "method": "shard_observe", "payload": { "session_id": "…" } }
```

Treat the bridge as **shard-free** until S78 ships an explicit decision. See
[`../HOW_TO_WIRE.md`](../HOW_TO_WIRE.md) (roles START / JOIN / OBSERVE) for the
node-side surfaces that *do* exist.
