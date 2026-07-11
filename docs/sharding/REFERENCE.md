# Sharding — human reference

*Reference document (Diátaxis). This is the **human-readable twin** of the
machine source of truth. For the narrative, see
[`EXPLANATION.md`](./EXPLANATION.md); for the role-by-role guide, see
[`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md).*

> **Status: LIVE-PROVEN (S81 I/J)** — the in-vivo session orchestrator and the
> live 2-machine benchmark shipped, see
> [`README.md`](./README.md). The body is intentionally in English: this page
> is reference material consumed by external contributors and agents.

## Single source of truth

The Rust structs are the source of truth. The machine spec
[`docs/protocol/SHARD_PROTOCOL_SPEC.md`](../protocol/SHARD_PROTOCOL_SPEC.md) and
its generated `*.schema.json` schemas in
[`crates/nexus-core-rs/src/schemas/`](../../crates/nexus-core-rs/src/schemas/)
are derived from them and guarded by a drift test. **This page is a convenience
mirror** — if it ever disagrees with the spec or the structs, the spec and the
structs win. Wire primitives live in
[`crates/nexus-core-rs/src/shard_plan.rs`](../../crates/nexus-core-rs/src/shard_plan.rs)
and
[`crates/nexus-core-rs/src/compute_group.rs`](../../crates/nexus-core-rs/src/compute_group.rs).

## Versioning regime

Pre-v1.0 **raw-op additive**. Every signed payload pins its
`*_FORMAT_VERSION` at `1`; the five `DOMAIN_*_V1` families and the
`sbfb/shard/1` ALPN are brand-new, so introducing them bumps **nothing** and
adds **no new dependency**. A decoder refuses a `version` it does not
understand. Post-`v1.0` the policy flips (each break bumps the version,
decoders accept a range).

## Domain-separation tags (signed payloads)

| Family | Domain tag (on-wire) | Signer |
|---|---|---|
| Compute-group allowlist | `nexus-compute-group-v1` | initiator |
| Sharded-session manifest | `nexus-shard-plan-v1` | initiator |
| Run proof | `nexus-run-proof-v1` | session driver (S81 I/J); per-worker = S82 |
| VRF spot-check draw (N1) | `nexus-vrf-draw-v1` | verifier |
| Activation commit-reveal (N3) | `nexus-activation-commit-v1` | worker |

## Types

`signed?` means the type carries an Ed25519 signature in its own envelope.
Unsigned types are only meaningful nested inside a signed parent. All metrics
are integers (no floats in signed payloads — see [`EXPLANATION.md`](./EXPLANATION.md)).

| Type | Key fields (Rust) | signed? | DOMAIN tag |
|---|---|---|---|
| `ComputeGroup` | `version: u16`, `group_id: String`, `initiator: [u8;32]`, `revision: u64`, `members: Vec<[u8;32]>` | yes | `nexus-compute-group-v1` |
| `ShardAssignment` | `worker_pubkey: [u8;32]`, `layer_start/layer_end: u32` (half-open), `role: ShardRole` (`layer_worker`), `shard_hashes: Vec<[u8;32]>`, `kv_cache_policy: KvCachePolicy` (`local_ephemeral`), `fallback_node: Option<[u8;32]>`, `launch_profile_hash: [u8;32]` | no (in a manifest) | — |
| `ShardPlan` | `assignments: Vec<ShardAssignment>` (Vec order **is** pipeline order) | no (in a manifest) | — |
| `ShardedSessionManifest` | `version: u16`, `initiator: [u8;32]`, `session_id: String`, `group_id: String`, `revision: u64`, `plan: ShardPlan`, `model_digest/tokenizer_hash/chat_template_hash: [u8;32]` | yes | `nexus-shard-plan-v1` |
| `RunMetrics` | `ttft_ms: u64`, `decode_milli_tokens_per_sec: u64` (tok/s × 1000), `p95_token_latency_ms: u64`, `network_rx_bytes/tx_bytes: u64`, `worker_drop_count: u32` | no (in a run proof) | — |
| `RunProof` | `version: u16`, `worker_pubkey: [u8;32]`, `session_id: String`, `model_digest: [u8;32]`, `prompt_profile_hash: [u8;32]`, `activation_fingerprint: [u8;32]` (N0; 32 zeros = not provided), `metrics: RunMetrics`, `participants: Vec<[u8;32]>` | yes | `nexus-run-proof-v1` |
| `ShardSessionView` | `session_id: String`, `member_count: usize` | no (observed DTO) | — |
| `ShardSessionStatusResponse` | `found: bool`, `session: Option<ShardSessionView>` | no (observed DTO) | — |

The control-plane DTO `ShardSessionView` exposes **exactly** `session_id` +
`member_count`, never a `worker_pubkey`/`initiator` (privacy whitelist,
THREAT_MODEL §16 SI-3/SI-4). `ShardSessionStatusResponse.session` is always
serialized (`null` when absent) so the front Zod `.strict()` envelope parses an
empty result as success, not a transport error.

## Data plane — ALPN `sbfb/shard/1`

One long-lived bidirectional QUIC stream per **driver↔stage** pair (the frozen
S77 HUB topology — the driver dials each stage, not adjacent workers), framing
length-prefixed big-endian. Admission (`is_member`) is checked **before**
`accept_bi` / any frame read; a correctness verdict (N0–N3) is **never** carried
on the data plane — it is derived from signed `RunProof`s afterwards.
Constraint: **llama-arch models only, same GGUF across the group**.

## Caps & limits (named constants)

Every collection/string cap is enforced at **both** sign and verify, so a node
can never produce a payload its own peers would reject.

| Constant | Value | Bounds |
|---|---|---|
| `MAX_SHARD_FRAME_BYTES` | 256 MiB | one data-plane frame |
| `MAX_SHARD_N_CTX` | 8192 | per-shard context tokens (policy const, never on-wire) |
| `SHARD_PLAN_MAX_ASSIGNMENTS` | 256 | assignments per plan |
| `RUN_PROOF_MAX_PARTICIPANTS` | 256 | participants per run proof |
| `SESSION_ID_MAX` | 128 | `session_id` bytes |
| `SHARD_GROUP_ID_MAX` | 128 | manifest `group_id` bytes |
| `SHARD_HASHES_MAX` | 64 | `shard_hashes` per assignment |
| `COMPUTE_GROUP_MAX_MEMBERS` | 256 | members per group |
| `COMPUTE_GROUP_ID_MAX` | 128 | `ComputeGroup.group_id` bytes |

`SHARD_GROUP_ID_MAX` (manifest) and `COMPUTE_GROUP_ID_MAX` (`ComputeGroup`) are
two **distinct** constants that happen to share the value 128.

## Verification thresholds (S82-pending tuning)

These are the **current calibration** of the graded-verification ladder. They
are documented for the implementer, but the values are **S82-pending tuning**:
they were chosen from the source papers, not yet re-calibrated on the real
two-machine rig. The live benchmark itself EXISTS since S81 Phase J
(`sprint81_t2_j_shard_inference.json`, PASS — the re-calibration baseline);
the tuning pass on it is routed S82.

| Stage | Constant | Value | Source |
|---|---|---|---|
| N0 TOPLOC | `TOPLOC_TOP_K` | 128 | `toploc.rs` |
| N0 TOPLOC | `TOPLOC_THRESH_EXP_MISMATCH` | 38 | `toploc.rs` |
| N0 TOPLOC | `TOPLOC_THRESH_MANT_MEAN` | 10 | `toploc.rs` |
| N0 TOPLOC | `TOPLOC_THRESH_MANT_MEDIAN` | 8 | `toploc.rs` |
| N1 spot-check | `SPOT_CHECK_RATE_TRUSTED_BP` | 100 bp (1 %) | `verification.rs` |
| N1 spot-check | `SPOT_CHECK_RATE_STANDARD_BP` | 500 bp (5 %) | `verification.rs` |
| N1 spot-check | `SPOT_CHECK_RATE_SUSPECT_BP` | 2000 bp (20 %) | `verification.rs` |
| N1 spot-check | `TRUST_TIER_TRUSTED` / `TRUST_TIER_STANDARD` | 80 / 50 | `verification.rs` |
| N3 SENTINEL | `SENTINEL_ALPHA_BP` | 9000 bp | `sentinel.rs` |
| N3 SENTINEL | `SENTINEL_DEVIATION_THRESH_BP` | 5000 bp | `sentinel.rs` |
| N3 SENTINEL | `SENTINEL_BP_DENOMINATOR` | 10000 | `sentinel.rs` |

Basis points (`bp`) use an integer denominator of 10000 (no floats). The spot-
check rate is selected by the prover's trust tier: `>= TRUST_TIER_TRUSTED` (80)
→ TRUSTED rate, else `>= TRUST_TIER_STANDARD` (50) → STANDARD rate, else SUSPECT
rate. There is no `TRUST_TIER_SUSPECT` constant — SUSPECT is the `else` branch
below `TRUST_TIER_STANDARD`. SENTINEL flags a frontier whose forward activation
EMA deviates beyond the threshold.

## See also

- Threat model & graded verification (N0–N3, SI-1..SI-11, incentive):
  [`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16.
- Rust patterns: [`docs/rust/PATTERNS.md`](../rust/PATTERNS.md) §P64–§P69
  (shard wire / verification) + §P39 (the read-only route's DB-singleton host).
- Machine spec: [`docs/protocol/SHARD_PROTOCOL_SPEC.md`](../protocol/SHARD_PROTOCOL_SPEC.md).
