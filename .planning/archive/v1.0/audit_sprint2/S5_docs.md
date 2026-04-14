# Audit S5 — `docs.rs` (iroh-docs wrapper)

**File**: `crates/nexus-core-rs/src/docs.rs` (372 lignes)

---

## Conforme

All API signatures verified against iroh-docs 0.97.0 local source (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/iroh-docs-0.97.0/src/api.rs`):

- `DocsClient::author_create()` → `Result<AuthorId>` ✓ (matches `Docs::author_create`)
- `DocsClient::create_doc()` → `Result<DocHandle>` via `Docs::create()` → `Result<Doc>` ✓
- `DocsClient::import_ticket(ticket)` → `Result<DocHandle>` via `Docs::import(DocTicket)` ✓
- `DocHandle::set(author, key, value)` → `Result<[u8;32]>` via `Doc::set_bytes(AuthorId, Bytes, Bytes) -> Result<Hash>`, `.as_bytes()` dereference correct ✓
- `DocHandle::get_exact(author, key)` → `Result<Option<Entry>>` via `Doc::get_exact(AuthorId, &[u8], bool)`, hardwired `include_empty=false` ✓
- `DocHandle::subscribe()` → `Result<impl Stream<Item=Result<LiveEvent>>+Send+Unpin>` matches `Doc::subscribe()` ✓
- `DocHandle::share_write/read()` → `Result<DocTicket>` via `Doc::share(ShareMode, AddrInfoOptions)` ✓
- `ShareMode::Read`/`Write`, `AddrInfoOptions::RelayAndAddresses` — variants confirmed in `api/protocol.rs` ✓
- `import_and_subscribe` — uses `Docs::import_and_subscribe(DocTicket) -> Result<(Doc, impl Stream<...>)>` ✓

---

## Manquant

Plan (jour 3) requiert explicitement **"query par prefix"**. Absent du wrapper :
- No `get_many(query)` / prefix-scan method exposed on `DocHandle`
- No `author_list()` (available in `Docs` but not wrapped)
- No `list()` (list all docs on node)

These are in the iroh-docs API but not wrapped. Plan line: `query par prefix, subscribe stream, export/import tickets`.

---

## Déviations

- `DocsClient` uses a lifetime `'a` tied to `&'a Docs` — correct but means it cannot outlive the `Node`. Not a bug, but callers must not store `DocsClient` past the node.
- `DocHandle::set()` silently drops the `Vec<u8>` key and value parameters (converts via `Into<Vec<u8>>`), then passes to iroh as `Bytes`. Harmless.
- `author_default()` is wrapped but not mentioned in plan — minor bonus, not a deviation.

---

## Qualité

- All fallible iroh calls are mapped to `NexusError::Docs(String)` — no `.unwrap()` in production code (lines 1–281).
- All `unwrap()`/`expect()` calls are strictly inside `#[cfg(test)]` (lines 283–372) — acceptable.
- Error messages are descriptive (`"author_create failed: {e}"`).
- Re-exports of `AuthorId`, `DocTicket`, `NamespaceId` as `Docs*` aliases isolate downstream from iroh-docs dep — good hygiene.
- `inner()` escape hatch on `DocHandle` correctly documented for advanced use.

---

## Tests

**3/3 passed** — `cargo test -p nexus-core-rs --lib docs` (4.83s):

- `author_create_returns_distinct_ids` ✓
- `create_doc_and_set_get_roundtrip` ✓
- **`two_nodes_sync_via_share_import` ✓ — PASS** (2-node CRDT sync end-to-end in 4.70s, `LiveEvent::InsertRemote` observed on node B)

---

## Bugs (DO NOT FIX)

1. **Missing `query_prefix` method** (`docs.rs` — no line, method absent). Plan required it; without it workers cannot scan task-doc entries by prefix. Must be added before Sprint 3 worker integration.
2. **`get_exact` hides tombstones unconditionally** (`docs.rs:228`, `include_empty=false` hardwired). Deletion markers are invisible. Acceptable for now but documented tombstone GC strategy (plan p.441) will need a variant that exposes them.
