# Sharding — agent wiring spec (contract-dense, source-anchored)

> **Audience: an LLM / agent that must wire or review the shard pipeline without
> hallucinating.** This is the machine-actionable contract layer above the human
> docs ([`EXPLANATION.md`](./EXPLANATION.md), [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md),
> [`REFERENCE.md`](./REFERENCE.md)) and the wire spec
> ([`../protocol/SHARD_PROTOCOL_SPEC.md`](../protocol/SHARD_PROTOCOL_SPEC.md)).
> Every contract clause carries a **source_ref** of the form `path:Symbol` to a
> rank-1 repo file; verify the ref (grep the symbol) before you act on the claim.

## 1. Authority — Truth Stack

When this spec and any other source disagree, trust the higher rank:

```
repo files > .planning/active/ > commits > prompts > chat
```

- **Rank-1 = repo files** (`crates/`, `docs/`, `web/`, `scripts/`). The only
  authority. Every contract clause here cites one as `path:Symbol`.
- **Rank-2 = `.planning/active/`** is an *in-flight pointer* — it is archived
  when the sprint closes, so never treat an `.planning/active/...` path as a
  durable anchor. (For this reason the source-ref-check resolves only rank-1
  paths.)
- Lower ranks (commit bodies, prompts, chat) are context, not contract.
- **Rule: a fact absent from rank-1 is `Not evidenced`** — do not assert it. If
  you cannot point at a repo file, say so rather than inventing a symbol.

> **Status: LIVE-PROVEN (S81 I/J).** The signed wire primitives, the data
> plane, the in-vivo session orchestrator (mount / drive / result,
> `shard_session.rs`) and the driver-signed `RunProof` emission are live —
> benchmark PASS on the real 2-machine rig
> (`sprint81_t2_j_shard_inference.json`). Still PROVISIONAL: per-worker
> proof emission + dispute arbitration (routed S82) — mark only THOSE
> clauses accordingly.

> **Cardinal caveat — admission ≠ confidentialité.** Group admission is
> authenticated, but it is **not** confidentiality: activations travel in clear
> between members over `sbfb/shard/1` (SI-1 reconstruction High, SI-4 collusion
> High are residual ASSUME — no consumer GPU TEE in 2026). Never document or wire
> this path as if it protected prompt/activation secrecy. See
> [`../security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16.

## 2. Actor model + sequence

Three actors, one private compute group:

- **Initiator** — a node that owns the run. Builds the plan, signs the manifest,
  authorises the group. Never an app (the iframe bridge has no shard method).
- **Worker** — a node admitted to the `ComputeGroup` allowlist that executes one
  contiguous layer block; since S81 K it also ATTESTS its loaded stage at
  stage-link establishment. The signed `RunProof` of a run is emitted by the
  session DRIVER (S81 I/J); per-worker proofs are routed S82.
- **Observer** — any loopback caller reading aggregate session status; sees a
  `member_count`, never an identity.

Sequence (each step is contracted in §3):

```
START  →  PLAN  →  SIGN  →  CLAIM/ADMIT  →  ATTEST(S81 K)  →  RUN-PROOF(driver, S81 I/J)  →  OBSERVE
```

## 3. Per-step contract

Each step lists **source_ref** (`path:Symbol`, grep-resolvable) · **signed?** ·
**DOMAIN tag** · **caps / preconditions**.

### START — node-privileged, not an app capability
- source_ref `web/src/components/ShardSessionPanel.tsx:ShardSessionPanel`.
- Signed? n/a. DOMAIN n/a.
- Precondition: initiation is a **node** action.
  `web/src/bridge/protocol.ts:BridgeMethodSchema` is a **closed enum of app-facing
  methods, none of which is a shard method** (see
  [`examples/bridge_gap.md`](./examples/bridge_gap.md)). Do not pre-wire one.

### PLAN — the ordered pipeline
- source_ref `crates/nexus-core-rs/src/shard_plan.rs:ShardPlan` (assignments:
  `crates/nexus-core-rs/src/shard_plan.rs:ShardAssignment`).
- Signed? not on its own — signed as part of the manifest (next step).
- Preconditions (cite BOTH; they are **different checks**):
  - **Contiguity** — `crates/nexus-core-rs/src/shard_plan.rs:is_pipeline_contiguous`:
    adjacent assignments join with no gap/overlap. This does **NOT** require the
    first block to start at layer 0.
  - **Full-model coverage** — `crates/nexus-coordinator-rs/src/placement.rs:covers_full_model`:
    a *separate* scheduler check that the plan spans `[0, total_layers)`
    (contiguity **and** first.layer_start==0 **and** last.layer_end==total). Do
    not conflate coverage with contiguity.
  - Caps: `crates/nexus-core-rs/src/shard_plan.rs:SHARD_PLAN_MAX_ASSIGNMENTS` =
    256, enforced at both sign and verify.

### SIGN — the manifest the initiator AUTHORISES
- source_ref `crates/nexus-core-rs/src/shard_plan.rs:ShardedSessionManifest`,
  signed into `crates/nexus-core-rs/src/shard_plan.rs:ShardedSessionManifestEntry`.
- Signed? **YES**, Ed25519 over canonical JCS bytes
  (`crates/nexus-core-rs/src/canonical.rs:canonical_bytes`).
- DOMAIN tag `crates/nexus-core-rs/src/canonical.rs:DOMAIN_SHARD_PLAN_V1`.
- Verify: `crates/nexus-core-rs/src/shard_plan.rs:verify_signature` re-derives the
  bytes and rejects a payload whose embedded `initiator` ≠ the signing key
  (attribution check). Runnable proof:
  [`examples/sign_verify.rs`](./examples/sign_verify.rs), compiled+run by
  `crates/nexus-core-rs/tests/shard_sign_verify.rs`.
- Caps: `crates/nexus-core-rs/src/shard_plan.rs:SESSION_ID_MAX` /
  `crates/nexus-core-rs/src/shard_plan.rs:SHARD_GROUP_ID_MAX` (128),
  `crates/nexus-core-rs/src/shard_plan.rs:SHARD_HASHES_MAX` (64) — see
  [`REFERENCE.md`](./REFERENCE.md).

### CLAIM / ADMIT — crypto before any IO
- Worker claim: `crates/nexus-worker-core/src/engine/shard_claim.rs:authorize_claim`
  — verifies the signed plan **FIRST**, before touching the network or the GGUF.
- Group membership: `crates/nexus-core-rs/src/compute_group.rs:ComputeGroup`,
  `crates/nexus-core-rs/src/compute_group.rs:is_member`.
- Data-plane admission: the acceptor checks
  `crates/nexus-core-rs/src/shard.rs:is_member` **BEFORE** the iroh
  `Connection::accept_bi` call (`crates/nexus-core-rs/src/shard.rs:accept_bi`) /
  any frame read; a non-member is closed at handshake with
  `crates/nexus-core-rs/src/shard.rs:SHARD_REJECT_NOT_MEMBER`.
- DOMAIN tags in scope: `crates/nexus-core-rs/src/canonical.rs:DOMAIN_COMPUTE_GROUP_V1`,
  `crates/nexus-core-rs/src/canonical.rs:DOMAIN_VRF_DRAW_V1`,
  `crates/nexus-core-rs/src/canonical.rs:DOMAIN_ACTIVATION_COMMIT_V1`.
- Caps: data-plane frame ≤ `crates/nexus-core-rs/src/shard.rs:MAX_SHARD_FRAME_BYTES`
  = 256 MiB; context ≤ `crates/nexus-core-rs/src/shard.rs:MAX_SHARD_N_CTX` = 8192.

### RUN-PROOF — what a worker EXECUTED (driver emission LIVE since S81 I/J)
- source_ref `crates/nexus-core-rs/src/shard_plan.rs:RunProof`, signed into
  `crates/nexus-core-rs/src/shard_plan.rs:RunProofEntry`; integer-only metrics
  `crates/nexus-core-rs/src/shard_plan.rs:RunMetrics`.
- Signed? **YES**, DOMAIN tag
  `crates/nexus-core-rs/src/canonical.rs:DOMAIN_RUN_PROOF_V1`.
- **Scope (S81 I/J)**: the session DRIVER signs a `RunProofEntry` in production
  at the end of every drive (`crates/nexus-shell-daemon/src/shard_session.rs:generate_session`);
  `participants` names the workers that ACTUALLY executed (fallbacks included)
  and `activation_fingerprint` binds the LAST step's N0 TOPLOC commitment.
  The driver's proof covers the run it DROVE — it is a self-claim, not an
  independent verification. **Per-worker** RunProofs from remote shards need a
  control-plane return channel and are routed S82: mark any "each worker emits
  its own signed proof" claim PROVISIONAL until then.

### OBSERVE — read-only aggregate status
- source_ref `crates/nexus-shell-daemon/src/http.rs:shard_session_response` (body),
  `crates/nexus-shell-daemon/src/http.rs:shard_session_response` (privacy projection).
- Signed? n/a (read-only GET). DOMAIN n/a.
- Auth tier: loopback only — `x-sbfb-token` bearer + loopback Host + absent-or-loopback
  Origin, enforced by `crates/nexus-shell-daemon-core/src/auth.rs:auth_required`;
  route registered in `crates/nexus-shell-daemon/src/http.rs:authed_routes`.
- Preconditions: none to call; the projection NEVER exposes `worker_pubkey`/`initiator`
  (only `session_id`, `member_count` and the aggregate `rtt_frontier_ms`, S81 I).
  Full HTTP contract + response shapes in §4.

## 4. Control-plane HTTP contract

```
GET /api/daemon/shard-session/{id}
```

- source_ref `crates/nexus-shell-daemon/src/http.rs:shard_session_response`
  (builds the body); the status is served from the LIVE in-memory registry
  `crates/nexus-shell-daemon/src/shard_session.rs:ShardSessionRegistry`
  (S81 Phase I replaced the S77 empty-store stub `live_shard_session`, which
  no longer exists); `crates/nexus-shell-daemon/src/http.rs:shard_session_response`
  (the privacy projection).
- **Auth tier — loopback only**: `x-sbfb-token` bearer **+** loopback `Host` **+**
  absent-or-loopback `Origin`, enforced by the middleware
  `crates/nexus-shell-daemon-core/src/auth.rs:auth_required` (header
  `crates/nexus-shell-daemon-core/src/auth.rs:AUTH_HEADER`); the route is
  registered in `crates/nexus-shell-daemon/src/http.rs:authed_routes` (handler
  `crates/nexus-shell-daemon/src/http.rs:shard_session`). Not reachable from a
  sandboxed iframe.
- **Response (unknown id)** — deterministically:

  ```json
  { "found": false, "session": null }
  ```

  `200 OK` with honest defaults (never `404`) so the front parse succeeds.
- **Response (mounted id, S81 I)** — `session` exposes `member_count` and the
  aggregate `rtt_frontier_ms`, never a `worker_pubkey` / `initiator`. The orchestrator's write/drive routes
  (`group`, `mount`, `generate`, `result`, `drop-shard`) are specified in
  `docs/protocol/SHARD_PROTOCOL_SPEC.md` §6.
- Front wrapper: `web/src/api/daemon.ts:getShardSession` (Zod `.strict()`).
- Runnable example: [`examples/observe.curl.md`](./examples/observe.curl.md).

## 5. INVIOLABLE invariants

Violating any of these is a wire/security defect, not a style nit:

1. **No floats in signed payloads.** All metrics are integers
   (`crates/nexus-core-rs/src/shard_plan.rs:RunMetrics`) — a float would make
   canonical bytes non-deterministic across platforms and break signatures.
2. **Never expose `worker_pubkey` or `initiator`** on any read surface. The HTTP
   projection (`crates/nexus-shell-daemon/src/http.rs:shard_session_response`)
   emits only aggregates — `session_id`, `member_count`, `rtt_frontier_ms`
   (THREAT_MODEL §16 SI-3/SI-4).
3. **Additive-only, 0-bump (pre-v1.0).**
   `crates/nexus-core-rs/src/shard_plan.rs:SHARD_PLAN_FORMAT_VERSION` /
   `crates/nexus-core-rs/src/shard_plan.rs:RUN_PROOF_FORMAT_VERSION` stay at 1; a
   new raw-op does not bump them.
4. **Crypto before IO.** Verify a signature before allocating buffers, opening
   streams, or loading a model
   (`crates/nexus-worker-core/src/engine/shard_claim.rs:authorize_claim`,
   `is_member`-before-`accept_bi`).
5. **Each claim carries a source_ref.** This document, and any agent acting on
   it, cites a rank-1 file for every contract clause; an unanchored claim is
   `Not evidenced` (§1).

## See also

- Wire spec (types, schemas, caps): [`../protocol/SHARD_PROTOCOL_SPEC.md`](../protocol/SHARD_PROTOCOL_SPEC.md).
- Human docs: [`README.md`](./README.md), [`EXPLANATION.md`](./EXPLANATION.md),
  [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md), [`REFERENCE.md`](./REFERENCE.md).
- Index for agents: [`llms.txt`](./llms.txt).
- Generated schemas: `crates/nexus-core-rs/src/schemas/shard_plan.schema.json`,
  `crates/nexus-core-rs/src/schemas/run_proof.schema.json`,
  `crates/nexus-core-rs/src/schemas/shard_session_status_response.schema.json`.
