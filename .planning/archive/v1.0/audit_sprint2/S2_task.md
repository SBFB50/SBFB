# S2 Audit — `task.rs` (nexus-core-rs)

Audited: 2026-04-10 | File: `crates/nexus-core-rs/src/task.rs` (444 lines)

---

## Conforme

- Plan S8 requires: `Task`, `TaskEntry`, `ResultEntry`, `Claim`, canonical serialization for signatures. All four types present.
- `serde-big-array 0.5` usage is correct: `#[serde(with = "BigArray")]` on `[u8; 64]` fields (lines 142, 231). Confirmed correct API for 0.5.x.
- Clear separation: `canonical_bytes()` (line 309) is the single signing surface; envelope types (`TaskEntry`, `ResultEntry`) carry the payload + pubkey + signature as distinct fields.
- Signatures computed over `canonical_bytes(&self.task)` / `canonical_bytes(&self.payload)` — never over the envelope itself.
- `TASK_FORMAT_VERSION` constant (line 42) appears in every signable struct.

## Manquant

- `Claim` has no `sign()`/`verify_signature()` methods — only a `new()` constructor. Claim signing is only exercised ad-hoc in one test (line 425). A production `ClaimEntry` wrapper (signed claim) is absent.
- No test exercises deserialization from an external/Python-produced JSON to verify cross-language canonical compatibility (`json.dumps(sort_keys=True)` ↔ `serde_json`).

## Déviations

- `canonical_bytes` docstring (line 300) states "sorts struct fields in declaration order" — this is accurate for serde_json but is an implementation detail not guaranteed by spec. No domain prefix (e.g., `b"nexus-task-v1:"`) before the JSON bytes; malleability possible if same payload type is reused across domains.
- `serde_json::to_vec` serializes struct fields in **declaration order**, not alphabetically. Only `BTreeMap` keys are sorted. Cross-language verifiers must know declaration order — this is fragile if fields are ever reordered in source.

## Qualité

Good. Module-level doc is thorough. Field-level docs explain purpose, constraints, and iroh-specific caveats (e.g., LWW tie-breaking for `Claim`). `canonical_bytes` is generic and single-purpose. `metadata: BTreeMap<String, String>` (line 99) — deterministic, values constrained to `String` (no floats, no nesting).

## Tests

10/10 pass (`cargo test -p nexus-core-rs --lib task`). Coverage:
- Determinism of canonical bytes
- BTreeMap insertion-order independence
- Sign → verify (Task and Result)
- Tamper detection (payload and pubkey)
- Round-trip deserialize
- Version field presence in output

## Bugs (DO NOT FIX)

1. **MEDIUM — No signed Claim envelope**: `Claim` has no `sign()`/`verify_signature()`. The LWW race-condition described in the docstring (line 259) requires coordinator to authenticate claims, but nothing enforces it at the type level.
2. **LOW — Struct field ordering fragile for cross-language**: `serde_json::to_vec` emits fields in declaration order (not alphabetically). Python `json.dumps(sort_keys=True)` sorts keys alphabetically. If Python ever produces canonical bytes for a `Task` (coordinator path), field order mismatch will break signature verification. This is **not** a bug today (Rust signs and verifies), but becomes critical in Sprint 4 when the Python coordinator must sign `TaskEntry`.
3. **LOW — No domain separation**: Canonical bytes have no type prefix. A valid `canonical_bytes(&claim)` could theoretically be submitted as if it were `canonical_bytes(&task)` for a structurally similar payload.
