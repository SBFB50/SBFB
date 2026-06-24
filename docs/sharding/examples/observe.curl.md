# Example — OBSERVE a shard session (read-only control plane)

Status: **PROVISIONAL.** There is no live session store yet (the in-vivo session
orchestrator + registry is a Sprint **S78** carry), so this route deterministically
returns the empty envelope for every id. The contract below is exact and stable;
the *populated* shape is what S78 will fill in.

## Route

```
GET /api/daemon/shard-session/{id}
```

Source: [`../../../crates/nexus-shell-daemon/src/http.rs`](../../../crates/nexus-shell-daemon/src/http.rs)
(`shard_session_response` builds the body; `live_shard_session` is the empty-store
seam; `project_shard_session` is the privacy projection).

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

## Response — empty store (today, every id)

`200 OK` with honest defaults so the front parse succeeds (never `404`):

```json
{ "found": false, "session": null }
```

## Response — populated (S78, when a session exists)

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
