# SBFB Sharded-Inference Protocol Specification

**Status:** Sprint 77 — sharding pipeline delivered (A-K), **feature
PROVISIONAL** (live cross-machine benchmark `RIG-ABSENT`; the in-vivo
session orchestrator + benchmark are a carry to **Sprint 78**). This
document specifies the wire contract A-K shipped; it does not claim a
running production session.
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
`[layer_start, layer_end)`, streams the activations of its block to the
next worker over a long-lived QUIC stream, and the last worker emits the
output. Pipeline-parallel (not tensor-parallel) is deliberate: it is
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
initiator authored this plan*; a valid `RunProofEntry` signature proves
only *which worker* produced it (non-repudiation). Neither attests that
the computation is **correct**. Graded verification (N0 TOPLOC → N1 VRF
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
| Run proof | `nexus-run-proof-v1` | worker |
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

The signed **envelopes** (`ComputeGroupEntry`,
`ShardedSessionManifestEntry`, `RunProofEntry`) are intentionally NOT
schematised: they wrap the payload with a redundant signer identity and
a `[u8; 64]` Ed25519 `signature` (`serde_big_array`), neither of which
is part of the canonical bytes.

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
`ShardSessionView` exposes EXACTLY `session_id: String` +
`member_count: usize` — never a `worker_pubkey`/`initiator`
(SI-3/SI-4). `ShardSessionStatusResponse` = `{ found: bool, session:
Option<ShardSessionView> }`; `session` is always serialized (`null` when
absent) so the front Zod `.strict()` envelope parses an empty result as
success, not a transport error.

---

## 5. Data plane — ALPN `sbfb/shard/1`

Source: `crates/nexus-core-rs/src/shard.rs`.

Activations flow over a **custom ALPN `sbfb/shard/1`** registered on the
iroh-QUIC endpoint: one long-lived bidirectional QUIC stream per
adjacent worker pair (`open_bi`), no application-level ping. Framing is
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

---

## 6. Control plane — `GET /api/daemon/shard-session/{id}`

Loopback-authenticated (bearer + Host + Origin, lives in the daemon's
`authed_routes`). Returns `ShardSessionStatusResponse` (§4.7). With no
live data-plane store yet (the session registry is a Sprint 78 carry),
the route deterministically answers `200 {found:false, session:null}`
for every id — a read-only route answers 200 with honest defaults so the
parse succeeds (the `seed_count` precedent), never 404. There is **no
`sbfb-bridge.js` shard method**: an app cannot start/join a session from
inside a sandboxed iframe; entry is the shell `/compute` panel only.

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
