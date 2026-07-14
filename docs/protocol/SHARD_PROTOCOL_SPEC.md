# SBFB Sharded-Inference Protocol Specification

**Status:** Sprint 77 delivered the pipeline hermetically (A-K); Sprint
81 Phases I/J delivered the **in-vivo session orchestrator** (mount /
drive / result over the daemon loopback, §6) and the **live
cross-machine benchmark** (CodeLlama-34B split RTX 5080-CUDA + M2-Metal,
16 greedy tokens, harness PASS — `sprint81_t2_j_shard_inference.json`),
closing the S77 `RIG-ABSENT` carry. Sprint 81 Phase K added the
**stage attestation** (loaded-stage ↔ signed-manifest binding, §5.2).
The feature is live-proven on the operator rig; graded verification
beyond the driver's signed RunProof (per-worker proofs, dispute
arbitration) is routed S82.
**Versioning regime:** pre-v1.0 **raw-op additive**. Every signed
payload pins its `*_FORMAT_VERSION` at `1`; the five `DOMAIN_*_V1`
families and the `sbfb/shard/1` ALPN are brand-new, so introducing them
bumps **nothing** (`FEED_FORMAT_VERSION` unchanged) and adds **no new
dependency**. A decoder refuses a `version` it does not understand.
Post-`v1.0` the policy flips (each break bumps the version, decoders
accept a range).

---

## 1. Overview

A model too large for any single worker's VRAM is split across the
members of a **private compute group** and run **pipeline-parallel**:
each worker owns a contiguous half-open block of transformer layers
`[layer_start, layer_end)`. The **frozen S77 topology is a HUB (star)**,
not direct server-to-server: the session **driver** (the head, dialer
side) walks the pipeline order, sending each stage's input frame and
reading its output over that stage's long-lived QUIC stream, then
forwarding it as the next stage's input — every frontier crosses through
the driver (`shard_session.rs:drive_decode_loop`, S81 Phase J). A
Petals-style direct-s2s topology is a later optimisation, not what
ships. Pipeline-parallel (not tensor-parallel) is deliberate: it is
latency-bound, not bandwidth-bound, so it survives WAN links where an
all-reduce would not.

### Cardinal caveat — admission ≠ confidentiality

The `ComputeGroup` allowlist is **admission control**: it decides *who
may join* a pipeline. It is **not** a confidentiality guarantee over the
activations, and the control-plane status route deliberately exposes
**only an aggregate `member_count`**, never the member identities
(THREAT_MODEL §16 SI-3/SI-4). The posture is honest-but-curious.

### Attestation scope — auto-attestation, not proof of correctness

A valid `ShardedSessionManifestEntry` signature proves only that *the
initiator authored this plan*. The `RunProofEntry` of a run is signed by
the session **DRIVER** (the head that drove the generation, S81 I/J),
not by each remote worker: it attests the run the driver measured
(`participants` names who actually executed, `activation_fingerprint`
binds the last step's N0 commitment) — a self-claim, non-repudiable for
the driver, never an independent verification. Per-worker signed proofs
need a control-plane return channel and are routed S82. Neither
signature attests that the computation is **correct**. Graded
verification (N0 TOPLOC → N1 VRF
spot-check → N2 tolerant redundancy → N3 commit-reveal/SENTINEL) raises
confidence but, until an independent verifier recomputes a fingerprint,
a `RunProof` is a **self-claim**. See THREAT_MODEL §16.

---

## 2. Domain-separation tags

Every signed payload is hashed with a domain-tag prefix
(`canonical.rs`), so a signature minted for one family can never verify
as another. The five families of this subsystem:

| Family | Domain tag (on-wire value) | Signer |
|---|---|---|
| Compute group allowlist | `nexus-compute-group-v1` | initiator |
| Sharded-session manifest | `nexus-shard-plan-v1` | initiator |
| Run proof | `nexus-run-proof-v1` | session driver (S81 I/J); per-worker = S82 |
| VRF spot-check draw (N1) | `nexus-vrf-draw-v1` | verifier |
| Activation commit-reveal (N3) | `nexus-activation-commit-v1` | worker |

---

## 3. Generated JSON Schemas (machine source of truth)

The Rust structs are the source of truth. `schemars::schema_for!`
generates a JSON Schema (draft 2020-12) per documented type, snapshotted
in `crates/nexus-core-rs/src/schemas/*.schema.json` and guarded by the
drift test `schemas::shard::tests::shard_schema_snapshot_matches_struct`
(it fails loudly if a struct evolves without a snapshot refresh). The
`#[derive(JsonSchema)]` is **additive** — inert to `serde::Serialize`,
so the canonical bytes the Ed25519 signature covers are unchanged.

| Type | Snapshot | Signed payload? |
|---|---|---|
| `ComputeGroup` | `compute_group.schema.json` | yes (`nexus-compute-group-v1`) |
| `ShardAssignment` | `shard_assignment.schema.json` | no (inside a manifest) |
| `ShardPlan` | `shard_plan.schema.json` | no (inside a manifest) |
| `ShardedSessionManifest` | `sharded_session_manifest.schema.json` | yes (`nexus-shard-plan-v1`) |
| `RunMetrics` | `run_metrics.schema.json` | no (inside a run proof) |
| `RunProof` | `run_proof.schema.json` | yes (`nexus-run-proof-v1`) |
| `ShardSessionView` | `shard_session_view.schema.json` | no (observed DTO) |
| `ShardSessionStatusResponse` | `shard_session_status_response.schema.json` | no (observed DTO) |
| `ShardSessionResultView` | `shard_session_result_view.schema.json` | no (observed DTO, S81 I) |
| `ShardSessionResultResponse` | `shard_session_result_response.schema.json` | no (observed DTO, S81 I) |
| `ShardGroupMintRequest` | `shard_group_mint_request.schema.json` | no (request body, S82 G) |
| `ShardGenerateRequest` | `shard_generate_request.schema.json` | no (request body, S82 G) |

The signed **envelopes** (`ComputeGroupEntry`,
`ShardedSessionManifestEntry`, `RunProofEntry`) are intentionally NOT
schematised: they wrap the payload with a redundant signer identity and
a `[u8; 64]` Ed25519 `signature` (`serde_big_array`), neither of which
is part of the canonical bytes.

`MountSessionRequest` (the third control-plane request body, §6.1) is
also NOT schematised, for the same envelope reason plus one more: it
embeds the signed `ComputeGroupEntry` verbatim, and its
`ShardWorkerSpec.addr` field is `iroh::EndpointAddr` — an upstream type
with no `JsonSchema` impl whose JSON shape iroh owns (a hand-written
proxy schema would drift silently at an iroh bump). Its machine
contract is the Request-body table in §6.1.

### No floats in signed payloads

`RunMetrics` is all-integer on purpose. An `f64` does not round-trip
bit-identically across platforms, so a signer (Rust) and a verifier
(e.g. a Python client) would derive divergent canonical bytes and the
signature would not verify. Rates use integer-friendly units
(`decode_milli_tokens_per_sec` = tokens/sec × 1000, bytes, ms).

---

## 4. Types

Source: `crates/nexus-core-rs/src/compute_group.rs`,
`crates/nexus-core-rs/src/shard_plan.rs`.

### 4.1 `ComputeGroup` — private admission allowlist

Domain `nexus-compute-group-v1`. Fields: `version: u16`,
`group_id: String`, `initiator: [u8; 32]` (owner pubkey),
`revision: u64`, `members: Vec<[u8; 32]>` (authorised worker pubkeys).
A peer whose `remote_id` is not in `members` is rejected at the
`sbfb/shard/1` handshake. `members.len()` is bounded by
`COMPUTE_GROUP_MAX_MEMBERS` and `group_id` by `COMPUTE_GROUP_ID_MAX`,
enforced at BOTH sign and verify.

### 4.2 `ShardAssignment` — one worker's layer block

Unsigned (meaningful only inside a signed manifest). Fields:
`worker_pubkey: [u8; 32]`, `layer_start: u32`, `layer_end: u32`
(half-open `[start, end)`), `role: ShardRole` (closed enum:
`layer_worker`), `shard_hashes: Vec<[u8; 32]>` (BLAKE3 weight pins,
bounded by `SHARD_HASHES_MAX`), `kv_cache_policy: KvCachePolicy` (closed
enum: `local_ephemeral`), `fallback_node: Option<[u8; 32]>`
(`#[serde(default)]`), `launch_profile_hash: [u8; 32]`.

### 4.3 `ShardPlan` — the ordered pipeline

Fields: `assignments: Vec<ShardAssignment>` (Vec order **is** pipeline
order; bounded by `SHARD_PLAN_MAX_ASSIGNMENTS`).
`ShardPlan::is_pipeline_contiguous` checks the blocks are gap-free and
non-overlapping.

### 4.4 `ShardedSessionManifest` — the run the initiator AUTHORISES

Domain `nexus-shard-plan-v1`. Fields: `version: u16`,
`initiator: [u8; 32]`, `session_id: String` (bounded by
`SESSION_ID_MAX`), `group_id: String` (bounded by `SHARD_GROUP_ID_MAX`),
`revision: u64`, `plan: ShardPlan`, `model_digest: [u8; 32]`,
`tokenizer_hash: [u8; 32]`, `chat_template_hash: [u8; 32]`. Signed via
`ShardedSessionManifestEntry`.

### 4.5 `RunMetrics` — all-integer execution metrics

Fields (all integer): `ttft_ms: u64`,
`decode_milli_tokens_per_sec: u64`, `p95_token_latency_ms: u64`,
`network_rx_bytes: u64`, `network_tx_bytes: u64`,
`worker_drop_count: u32`.

### 4.6 `RunProof` — what a worker EXECUTED

Domain `nexus-run-proof-v1`. Fields: `version: u16`,
`worker_pubkey: [u8; 32]`, `session_id: String` (bounded by
`SESSION_ID_MAX`), `model_digest: [u8; 32]`,
`prompt_profile_hash: [u8; 32]`, `activation_fingerprint: [u8; 32]`
(N0 TOPLOC commitment; 32 zeros = not provided; `#[serde(default)]`),
`metrics: RunMetrics`, `participants: Vec<[u8; 32]>` (bounded by
`RUN_PROOF_MAX_PARTICIPANTS`). Signed via `RunProofEntry`.

### 4.7 Observed DTO — `ShardSessionView` / `ShardSessionStatusResponse`

The control-plane projection (`crates/nexus-core-rs/src/schemas/shard.rs`).
`ShardSessionView` exposes `session_id: String`, `member_count: usize`
and `rtt_frontier_ms: Option<u64>` (an aggregate frontier-RTT transport
measurement, added S81 Phase I) — never a `worker_pubkey`/`initiator`
identity (SI-3/SI-4). `ShardSessionStatusResponse` = `{ found: bool, session:
Option<ShardSessionView> }`; `session` is always serialized (`null` when
absent) so the front Zod `.strict()` envelope parses an empty result as
success, not a transport error.

---

## 5. Data plane — ALPN `sbfb/shard/1`

Source: `crates/nexus-core-rs/src/shard.rs`.

Activations flow over a **custom ALPN `sbfb/shard/1`** registered on the
iroh-QUIC endpoint. In the shipped HUB topology (§1) each stream is
between the **driver and one stage worker** (the driver `open_bi`s to
each stage), not between adjacent workers; one long-lived bidirectional
QUIC stream per stage, no application-level ping. Framing is
**length-prefixed, big-endian**.

- **Admission before bytes:** the acceptor checks `is_member` (the dialing
  peer's Ed25519 id is on the `ComputeGroup` allowlist — non-spoofable,
  authenticated by QUIC) **BEFORE** `accept_bi` / any frame read. A
  non-member is rejected with `SHARD_REJECT_NOT_MEMBER`.
- **DoS caps both ways:** every frame is capped at `MAX_SHARD_FRAME_BYTES`
  (256 MiB) — enforced when *writing* and again from the *declared*
  length before allocating the read buffer. The per-shard context window
  is bounded by `MAX_SHARD_N_CTX` (8192 tokens).
- **Verdict out-of-band:** a correctness verdict (N0-N3) is never carried
  on the data plane; it is derived from signed `RunProof`s afterwards.

Constraint: llama-arch models only, same GGUF across the group
(homogeneous cohort).

### 5.1 Application-level step payloads (Sprint 81 Phase J)

A REAL inference session (manifest `model_digest != 0`) carries two
JSON payload shapes INSIDE the opaque frames, in addition to the raw
`[n_tokens, n_embd]` fp32-LE boundary tensor exchanged by middle
stages. They are **not wire structs**: no `*_FORMAT_VERSION`
governance, never signed — the signed artefacts stay the manifest and
the RunProof. Their `v` field (`SHARD_STEP_PAYLOAD_V = 1`) is an
application-level guard so a role mismatch fails LOUD; both codecs are
`deny_unknown_fields`, so a request never half-parses as a reply (and
vice versa).

- **`ShardStepRequest`** (driver → FIRST stage, per decode step):
  `{ v: u16, prompt: String, generated: Vec<i32> }`. The first stage
  owns the tokenizer: it re-derives its input ids each step (stateless
  per-step recompute, no cross-step KV reuse — what makes the SI-9
  fallback replay correct by construction).
- **`ShardStepReply`** (LAST stage → driver, per decode step):
  `{ v: u16, token_id: i32, piece: String, is_eos: bool,
  toploc_hex: String }` — greedy-sampled token + the N0 TOPLOC
  commitment hex of the post-norm hidden state (empty when the backend
  cannot provide one; the LAST step's commitment binds
  `RunProof.activation_fingerprint`).

Source: `crates/nexus-core-rs/src/shard.rs` (`ShardStepRequest`,
`ShardStepReply`, `SHARD_STEP_PAYLOAD_V`).

### 5.2 Stage attestation (Sprint 81 Phase K)

Before ANY data frame flows on a stage link of a real session, the
driver requests the stage's self-declared loaded stage and fail-closes
on mismatch with the signed manifest + `ShardAssignment`
(THREAT_MODEL §16 « Attestation loaded-stage »). Same opaque-frame JSON
posture as §5.1 (`SHARD_ATTEST_PAYLOAD_V = 1`, explicit `kind`
discriminants, `deny_unknown_fields`):

- **`ShardStageAttestationRequest`** (driver → stage):
  `{ v: u16, kind: "attest-stage-request" }`.
- **`ShardStageAttestation`** (stage → driver):
  `{ v: u16, kind: "stage-attestation", model_digest_hex: String
  (64-hex blake3 of the loaded GGUF, streaming-hashed; all-zeros = no
  real backend), layer_start: u32, layer_end: u32, is_first: bool,
  is_last: bool }`.

`ShardProtocol::accept` answers the request BEFORE the forwarder (a
real backend never sees the probe as activations); the echo/transport
path (`model_digest == 0`) never emits nor requires an attestation
(byte-identical to S77 Phase B). The attestation is a **self-claim by
an admitted member** — it closes the MISCONFIGURATION class, not a
deliberately byzantine stage (SI-4 residual).

---

## 6. Control plane — `/api/daemon/shard-session/*` (Sprint 81 Phase I)

Loopback-authenticated (bearer + Host + Origin, `authed_routes`; tiers
in `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` §3). The in-vivo
orchestrator surface, consumed verbatim by the b3_shard harness:

| Route | Effect |
|---|---|
| `POST /api/daemon/shard-session/group` | mint + sign the private `ComputeGroupEntry` (admission allowlist) |
| `POST /api/daemon/shard-session/mount` | placement → signed `ShardedSessionManifestEntry` → readiness barrier → gated registry insert |
| `POST .../{id}/generate` | drive one generation (echo pass or real decode loop per `model_digest`) |
| `GET .../{id}/result` | measured outcome (`ShardSessionResultView`, 14 fields): `session_id`, `result_text` (bounded), `ttft_s`, `toks_per_s`, `tokens`, `run_proof` (driver signature hex), `rtt_frontier_ms`, `worker_drop_count`, `failure` + the S82 B benchmark metrics `ttft_ms`, `tpot_ms`, `itl_p50_ms`, `itl_p95_ms`, `decode_milli_tokens_per_sec` |
| `POST .../{id}/drop-shard` | explicit counted churn of the tail shard (SI-9 acceptance lever) |
| `GET .../{session_id}` | read-only status (`ShardSessionStatusResponse`, §4.7) |

The registry is in-memory and node-local; insertion is gated on the
`DOMAIN_SHARD_PLAN_V1` signature + `is_member` checks, so the status
route can never serve an unauthenticated manifest. The projection stays
privacy-whitelisted (aggregate `member_count`, never a
`worker_pubkey`/`initiator`). There is **no `sbfb-bridge.js` shard
method**: an app cannot start/join a session from inside a sandboxed
iframe; entry is the shell `/compute` panel + the operator CLI
(`shard-session serve|plan|identity`, a local operator tool).

### 6.1 Request bodies (Sprint 82 Phase G)

The three POST bodies are loopback-API frontiers (doctrine §7). The two
primitive-only bodies are schematised (drift-gated snapshots, §3); the
mount body is table-documented here (see §3 for why).

**`POST /api/daemon/shard-session/group`** — `ShardGroupMintRequest`
(`crates/nexus-core-rs/src/schemas/shard.rs`, snapshot
`shard_group_mint_request.schema.json`):

| Field | Type | Required | Notes |
|---|---|---|---|
| `group_id` | string | yes | stable group handle |
| `members` | string[] | yes | worker Ed25519 pubkeys, lowercase hex (the head/dialer is added automatically) |
| `revision` | u64 | no (default 1) | monotonic group revision |

**`POST /api/daemon/shard-session/mount`** — `MountSessionRequest`
(`crates/nexus-shell-daemon/src/shard_session.rs`, NOT schematised — §3):

| Field | Type | Required | Notes |
|---|---|---|---|
| `session_id` | string | yes | bounded by `SESSION_ID_MAX` at the signed layer |
| `group` | `ComputeGroupEntry` | yes | the signed envelope minted by `group`, shared VERBATIM with every worker |
| `workers` | `ShardWorkerSpec[]` | yes | candidates: `{addr: iroh::EndpointAddr, vram_free_bytes: u64, shard_hashes?: [u8;32][], launch_profile_hash?: [u8;32]}` |
| `model` | `ShardModelSpec` | yes | `{total_layers: u32, quantized_vram_bytes: u64, model_digest?: [u8;32], tokenizer_hash?: [u8;32], chat_template_hash?: [u8;32]}` |
| `readiness_deadline_ms` | u64 | no | readiness-probe deadline override |
| `hop_deadline_ms` | u64 | no | per-hop dispatch deadline override (SI-9) |

**`POST /api/daemon/shard-session/{id}/generate`** —
`ShardGenerateRequest` (`crates/nexus-core-rs/src/schemas/shard.rs`,
snapshot `shard_generate_request.schema.json`):

| Field | Type | Required | Notes |
|---|---|---|---|
| `session_id` | string | no | redundant echo of the path id — see below |
| `prompt` | string | yes | the prompt to drive |
| `max_tokens` | u32 | no | REAL decode budget, clamped to `MAX_NEW_TOKENS_CAP`; ignored by a transport-only echo session |

**The PATH is authoritative** (a runtime contract no JSON Schema can
express): the generate route addresses the session by its path `{id}`;
a body `session_id` that disagrees with the path is rejected with `400`
— the daemon never silently drives another session. The optional
`session_id` body field is a redundant echo, not an alternative
addressing channel.

---

## 7. Caps & limits (named constants)

| Constant | Value | Bounds |
|---|---|---|
| `MAX_SHARD_FRAME_BYTES` | 256 MiB | one data-plane frame |
| `MAX_SHARD_N_CTX` | 8192 | per-shard context tokens |
| `SHARD_PLAN_MAX_ASSIGNMENTS` | 256 | assignments per plan |
| `RUN_PROOF_MAX_PARTICIPANTS` | 256 | participants per run proof |
| `SESSION_ID_MAX` | 128 | `session_id` bytes |
| `SHARD_GROUP_ID_MAX` | 128 | manifest `group_id` bytes |
| `SHARD_HASHES_MAX` | 64 | `shard_hashes` per assignment |
| `COMPUTE_GROUP_MAX_MEMBERS` | 256 | members per group |
| `COMPUTE_GROUP_ID_MAX` | 128 | group `group_id` bytes |

Every collection / string cap is enforced at BOTH sign and verify, so a
node can never produce a payload its own peers would reject.

---

## 8. References

- Threat model & graded verification (N0-N3, SI-1..SI-11, incentive):
  [`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16.
- Rust patterns: [`docs/rust/PATTERNS.md`](../rust/PATTERNS.md)
  §P64-69 (shard wire / verification) + §P39.
- Wire primitives: `crates/nexus-core-rs/src/shard_plan.rs`,
  `crates/nexus-core-rs/src/compute_group.rs`,
  `crates/nexus-core-rs/src/canonical.rs` (domain tags).
- Data plane: `crates/nexus-core-rs/src/shard.rs`.
- Generated schemas: `crates/nexus-core-rs/src/schemas/shard.rs` +
  `crates/nexus-core-rs/src/schemas/*.schema.json`.
