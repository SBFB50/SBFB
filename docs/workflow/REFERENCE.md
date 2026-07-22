# Workflow runtime — human reference

*Reference document (Diátaxis). Human-readable twin of the machine sources
indexed by [`llms.txt`](./llms.txt). Seeded at **S83 Phase I1** with the run
store loopback API family; it grows with each phase and the chantier-wide
closure is the S83 wrap-up (Phase AJ1). The body is intentionally in English:
reference material consumed by external contributors and agents.*

## Single source of truth

The Rust code is the source of truth. This page is a convenience mirror — if
it ever disagrees with the handlers
([`crates/nexus-shell-daemon/src/workflow_api.rs`](../../crates/nexus-shell-daemon/src/workflow_api.rs))
or the store
([`crates/sbfb-workflow-engine/src/store.rs`](../../crates/sbfb-workflow-engine/src/store.rs)),
the code wins. The behavioural contract is drift-gated by
`golden_http_workflow_domain` (`test_support.rs`).

## Run store loopback API (daemon, S83 Phase I1)

Read-only surface over the durable workflow run store. Both routes live in
`authed_routes` (loopback bearer `X-SBFB-Token` + Host + Origin; peer creds
at the UDS/Named-Pipe listener layer). D7 tier **T0** (read-only family) —
rows in
[`LOOPBACK_ENDPOINTS_TRUST_TIERS.md`](../security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md) §3.
Not duress-gated: local reads stay live under a decoy boot (the duress no-op
posture covers outbound/signing operations only), and this family exposes no
mutation.

### GET /api/daemon/workflow/run/{run_id} — shipped S83 I1

Aggregate progress of one run. `run_id` is server-validated:
`[A-Za-z0-9._-]{1,128}`, anything else → `400 {"error":"invalid run_id"}`.
A miss is `200 {found:false, run:null}` — never a 404 (read-only routes
answer 200 with honest defaults, seed-count precedent).

```json
{
  "found": true,
  "run": {
    "run_id": "run-a",
    "status": "running",
    "terminal_reason": null,
    "event_count": 1,
    "frontier_node": null,
    "spec_digest": "digest-a"
  }
}
```

Field domains: `status` is one of `"running"`, `"completed"`,
`"terminal"`; `terminal_reason` is `null` unless the run is terminal,
then `"step_hash_mismatch"` or `"outcome_unknown"`; `frontier_node` is
the node-instance id string or `null`; `spec_digest` is the digest
string recorded at run creation or `null`.

Invariants: the projection is a scalar ALLOWLIST — never a raw `RunEvent`
list (`WorkflowPublicProjection.v1` is a later phase), never snapshot state,
never an effect receipt / request hash. `event_count` counts journal entries;
`frontier_node` is the last node instance whose effect COMPLETED.
Anchors: `workflow_api.rs:workflow_run_response`, `workflow_api.rs:valid_run_id`.

### GET /api/daemon/workflow/lease — shipped S83 I1

Raw single-writer lease row of the store. `found:false` covers both "no
lease row yet" and "released". `expires_at_ms` is the takeover HEURISTIC
recorded by the store — the route renders no liveness verdict (judging
"expired" needs a clock and belongs to the caller).

```json
{
  "found": true,
  "lease": { "owner": "runtime-a", "epoch": 3, "expires_at_ms": 173000 }
}
```

Anchors: `workflow_api.rs:workflow_lease_response`, `store.rs:lease_state`.

### Deferred mutating family (NOT exposed — dated J/K)

`POST …/run/{run_id}/restore`, `POST …/workflow/gc`,
`POST …/workflow/lease/force` are tier-classified **T1** in the D7 inventory
but deliberately NOT registered: T1 CONFIRM_PROMPT is design-only today (a
mutating route would ship at effective T0), `restore_from_backup` requires a
CLOSED store (incompatible with the boot-held handle), GC needs the Phase J
clock source, and lease-force has no legitimate caller while the runtime
acquires its lease in-process. See THREAT_MODEL §19 (v24) for the full
rationale.

## Storage placement

The store file is `<daemon data root>/workflow_runs.sqlite3` (sibling of
`coordinator.db`, opened once at boot — single handle, no lease acquired at
I1). It is NEVER under the `~/.sbfb` security root.
