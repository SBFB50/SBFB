# Audit S7 — blobs.rs (iroh-blobs 0.99.0)

**Auditor**: Agent S7 | **Date**: 2026-04-10 | **File**: `crates/nexus-core-rs/src/blobs.rs`

---

## Conforme

- `MemStore::new()` signature matches (`store/mem.rs:118`). Used via `MemStore::default()` in `node.rs:193`.
- `MemStore` implements `Deref<Target = api::Store>` (`store/mem.rs:92–96`), so `.inner.blobs()` resolves correctly via deref coercion to `Store::blobs()` (`api.rs:233`).
- `add_bytes(bytes)` returns `AddProgress` which implements `IntoFuture → RequestResult<TagInfo>` (`api/blobs.rs:624`). Awaiting it is correct.
- `TagInfo.hash` is a **pub field** (`api/proto.rs:352`), not a method. `tag_info.hash.as_bytes()` at `blobs.rs:69` is correct.
- `Hash::as_bytes() -> &[u8; 32]` confirmed (`hash.rs:37`). Deref `*tag_info.hash.as_bytes()` yields `[u8; 32]` — correct.
- `Hash::from_bytes([u8; 32]) -> Hash` confirmed const fn (`hash.rs:42`).
- `Blobs::get_bytes(hash) -> ExportBaoResult<Bytes>` confirmed (`api/blobs.rs:368`). `.to_vec()` on `Bytes` is valid.
- `Blobs::has(hash) -> irpc::Result<bool>` confirmed (`api/blobs.rs:500`).
- Store is `MemStore` (volatile, in-process). Matches Sprint 2 plan spec.

## Manquant

- **fetch via ticket** — plan (`Day 5`) requires `fetch via ticket, pin, unpin, list_pinned`. None of these are implemented. Only `add_bytes`, `get_bytes`, `has` exist. Missing: ticket-based fetch, unpin, list_pinned.

## Déviations

- Plan session label is **S5** (Day 5 of Sprint 2), not S7. The commit `626d7eb` bundles docs/gossip/blobs/discovery together; blobs may have shipped as part of S5–S8 batch. No structural deviation in what was built — just incomplete scope.
- `add_bytes` auto-pins via `with_tag()` (creates a named tag internally, `api/blobs.rs:664–672`). The wrapper has no explicit `unpin` path. For MemStore (volatile), this is acceptable for Sprint 2, but means blobs cannot be released while the store is live.

## Qualité

- Module is clean, well-documented. Doc-comment in `blobs.rs:62` correctly notes `TagInfo.hash` is a field. No dead code. Error mapping via `NexusError::Blobs` is consistent with the rest of the crate.
- No `get_bytes` on unknown hash negative test — handled implicitly via `has_returns_false_for_unknown_hash` but error path of `get_bytes(unknown)` is untested.

## Tests

All 4 tests pass in 0.09s:
- `add_then_get_roundtrip` — round-trip verified
- `has_returns_true_after_add` — presence check verified
- `has_returns_false_for_unknown_hash` — absence check verified (all-zeros hash)
- `same_content_yields_same_hash` — content-addressing / dedup verified

## Bugs (DO NOT FIX)

1. **`blobs.rs:59` — `add_bytes` input type mismatch risk**: signature accepts `impl AsRef<[u8]>` but calls `.to_vec()` then passes `Vec<u8>` to `Blobs::add_bytes(impl Into<bytes::Bytes>)`. This compiles because `Vec<u8>: Into<Bytes>`, but the intermediate `.to_vec()` allocation is unnecessary when input is already `&[u8]` or `Bytes`. Not a bug, but wasteful.

2. **Missing negative test for `get_bytes(unknown_hash)`**: `blobs.rs:77–86` — `get_bytes` on a hash not in the store will return an error from `irpc`, but this error path is never exercised in tests. If the error type changes between iroh-blobs versions, it would be silently undetected.

3. **Scope gap (`blobs.rs` entire file)**: `fetch via ticket`, `unpin`, and `list_pinned` are called out in the Day 5 plan spec but are entirely absent. This is not a compilation error but means curator list fetch-by-ticket (the primary gossip→blobs flow described in the architecture) has no wrapper yet.
