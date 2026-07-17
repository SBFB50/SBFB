# Example — OBSERVE a shard session (read-only control plane)

Status: **LIVE (S81 Phase I).** The route serves the in-memory
`ShardSessionRegistry` (gated insert: manifest signature + `is_member`). For
an UNKNOWN id it deterministically returns the empty envelope below; for a
mounted session it returns the populated shape.

## Route

```
GET /api/daemon/shard-session/{id}
```

Source: [`../../../crates/nexus-shell-daemon/src/shard_session_http_api.rs`](../../../crates/nexus-shell-daemon/src/shard_session_http_api.rs)
(`shard_session_response` builds the body AND applies the privacy projection —
aggregate `member_count`, never a `worker_pubkey`/`initiator`; it reads the live
`ShardSessionRegistry`, S81 Phase I).

**Auth tier — loopback only.** The route lives in the daemon's `authed_routes`:
it requires the `x-sbfb-token` bearer **and** a loopback `Host` **and** an absent
or loopback `Origin`. The daemon binds `127.0.0.1:<ephemeral>`, so substitute the
actual bound port. An app inside a sandboxed iframe cannot reach it (no custom
header, cross-origin) — by design.

## Request

```bash
PORT=8787                       # the daemon's bound 127.0.0.1 port
TOKEN="$SBFB_AUTH_TOKEN"        # the node's loopback bearer

curl -sS \
  -H "x-sbfb-token: $TOKEN" \
  -H "Host: 127.0.0.1:$PORT" \
  "http://127.0.0.1:$PORT/api/daemon/shard-session/session-70b-1"
```

## Response — unknown id

`200 OK` with honest defaults so the front parse succeeds (never `404`):

```json
{ "found": false, "session": null }
```

## Response — populated (S81 I, when a session is mounted)

When a session is live, `session` exposes **only** the aggregate `member_count` —
**never** a `worker_pubkey` and **never** the `initiator` (privacy SI-3/SI-4):

```json
{ "found": true, "session": { "session_id": "session-70b-1", "member_count": 3 } }
```

The exact response type is `ShardSessionStatusResponse` / `ShardSessionView` —
see [`../REFERENCE.md`](../REFERENCE.md) §4.7 and the generated schema
[`../../../crates/nexus-core-rs/src/schemas/shard_session_status_response.schema.json`](../../../crates/nexus-core-rs/src/schemas/shard_session_status_response.schema.json).
The front wrapper is `getShardSession` in
[`../../../web/src/api/daemon.ts`](../../../web/src/api/daemon.ts) (Zod `.strict()`
envelope that treats the empty state as a success, not a transport error).
