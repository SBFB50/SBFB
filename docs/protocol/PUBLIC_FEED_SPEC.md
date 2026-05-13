# SBFB Public Feed Protocol Specification

**Version:** `FEED_FORMAT_VERSION = 1`
**Status:** Sprint 61 — initial specification
**Versioning regime:** post-v1.0 (each breaking change bumps
`FEED_FORMAT_VERSION`, decoders accept a range, new optional
fields carry `#[serde(default)]` for forward compatibility)

---

## 1. Overview

The SBFB Public Feed is an append-only signed event log that
records verifiable protocol events (releases published, source
staleness, curator endorsements, build quorum results). Any node
can replay the feed from genesis to reconstruct a consistent view
of the network's public state.

### Design goals

- **Verifiable:** every entry is signed by its author (Ed25519)
  and chained (BLAKE3 hash-chain). Tampering is detectable.
- **Replayable:** the feed can be materialized from entry 0 to
  reconstruct the full public registry state.
- **Extensible:** new operation types can be added without
  breaking existing consumers (tagged union, serde).
- **Local-first:** Sprint 1 operates on a local SQLite store.
  P2P sync is Sprint 2.

---

## 2. Operation types

### 2.1 Sprint 1 operations

#### `ReleasePublished`

Records that a project has published a new release from source.

```json
{
  "op_type": "ReleasePublished",
  "project_id": "<NodeId hex>",
  "repo_url": "https://github.com/org/app",
  "commit_sha": "<40 hex chars>",
  "artifact_hash": "<BLAKE3 hex of archive zip>",
  "provenance_hash": "<BLAKE3 hex of provenance.json, optional>",
  "is_open_source": true
}
```

**`is_open_source` validation rule:** this field is server-derived
at publish time, not user-settable. It is `true` only when the
complete verification chain is present and valid:
`repo_url + commit_sha + artifact_hash + provenance_hash`.
A `ReleasePublished` with `is_open_source: true` but missing any
of the four fields is invalid.

#### `SourceBecameStale`

Records that a project's source is no longer reachable or has
diverged from the published artifact.

```json
{
  "op_type": "SourceBecameStale",
  "project_id": "<NodeId hex>",
  "reason": "repo_unreachable"
}
```

Valid reasons: `"repo_unreachable"`, `"commit_diverged"`,
`"manual"`.

### 2.2 Future operations (defined, not yet implemented)

- `CuratorVouched` — a curator endorses a project (Sprint 2+)
- `BuildQuorumReached` — build quorum achieved SHA256 consensus
  (Sprint 2+)
- `SourceRecovered` — a stale source becomes reachable again
  (Sprint 2+)
- `SearchManifestPublished` — a search manifest is published for
  the project (Sprint 6 optional)

---

## 3. Canonical serialization

All feed entries use RFC 8785 JSON Canonicalization Scheme (JCS)
with domain separation, matching the 14 existing domains in
`canonical.rs`.

### Domain

```rust
pub const DOMAIN_FEED_V1: &[u8] = b"nexus-feed-v1";
```

### Canonical bytes

```text
DOMAIN_FEED_V1 || 0x00 || serde_jcs::to_vec(FeedEntryCanonical)
```

The `FeedEntryCanonical` struct contains the fields that are
hashed and signed:

```rust
pub struct FeedEntryCanonical {
    pub version: u16,           // FEED_FORMAT_VERSION
    pub op: PublicFeedOperation, // tagged union
    pub author_pubkey: String,   // Ed25519 public key hex
    pub timestamp: u64,          // unix seconds
    pub prev_hash: String,       // hex hash of previous entry
}
```

---

## 4. Hash-chain construction

The feed is a BLAKE3 hash-chain following the `kudos_ledger`
pattern.

### Genesis

```
prev_hash = "genesis"  (string literal sentinel)
```

### Entry N

```
canonical = FeedEntryCanonical { version, op, author, timestamp, prev_hash }
canonical_bytes = DOMAIN_FEED_V1 || 0x00 || JCS(canonical)
entry_hash = hex(BLAKE3(canonical_bytes))
```

### Verification

`verify_chain()` replays all entries from the first:

1. For entry 0: verify `prev_hash == "genesis"`
2. For entry N: verify `prev_hash == entry[N-1].entry_hash`
3. For all entries: recompute `canonical_bytes`, verify
   `entry_hash == hex(BLAKE3(canonical_bytes))`
4. For all entries: verify `Ed25519::verify(author_pubkey,
   canonical_bytes, signature)`

If any step fails, the chain is corrupt.

---

## 5. Replay rules

### Ordering

Entries are ordered by `seq` (auto-increment). Within the same
seq, only one entry exists (enforced by SQLite PRIMARY KEY).

### Idempotence

The feed is append-only. An operation with the same content but
different `seq` is a distinct entry (not deduplicated). This is
by design — the feed records _events_, not _state_.

### State transitions

Operations on the same `project_id` form an implicit lifecycle:

```
(no entry) → ReleasePublished → SourceBecameStale → ReleasePublished → ...
```

The feed does NOT enforce this ordering — a `SourceBecameStale`
for a project with no prior `ReleasePublished` is accepted. The
`FeedMaterializer` handles the semantics when building the view.

### Local Draft exclusion

A project in Local Draft state (not yet deployed from source)
MUST NOT appear in the feed with a `ReleasePublished` event.
Only projects that have completed the verified deploy pipeline
(clone + Keyoxide + zip + provenance) produce feed events.

---

## 6. Cursor format

A cursor is a checkpoint for incremental materialization:

```
(last_seq: u64, last_entry_hash: String)
```

- `last_seq` is the highest `seq` processed
- `last_entry_hash` is the `entry_hash` of that entry (integrity
  check)

### Resumption

When resuming from a cursor:

1. Load cursor `(seq, hash)` from persistent storage
2. Read the entry at `seq` from the feed
3. If `entry.entry_hash == hash`: resume from `seq + 1`
4. If mismatch or entry missing: full replay from `seq = 0`

The safety fallback (step 4) handles corruption or feed
truncation.

---

## 7. Test vectors

### ReleasePublished canonical bytes

Input:

```json
{
  "version": 1,
  "op": {
    "op_type": "ReleasePublished",
    "project_id": "abc123def456",
    "repo_url": "https://github.com/org/app",
    "commit_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "artifact_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "provenance_hash": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "is_open_source": true
  },
  "author_pubkey": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  "timestamp": 1700000000,
  "prev_hash": "genesis"
}
```

Expected canonical bytes (JCS produces deterministic key ordering):
domain prefix `nexus-feed-v1` + null byte + JCS-serialized JSON.

The entry hash is `BLAKE3(canonical_bytes)` encoded as hex.

---

## 8. Versioning policy

- `FEED_FORMAT_VERSION` starts at `1`
- Each breaking change to the canonical format bumps the version
- Decoders SHOULD accept `version <= FEED_FORMAT_VERSION` (range)
- New optional fields carry `#[serde(default)]` for forward
  compatibility within the same version
- Adding a new `PublicFeedOperation` variant is NOT a breaking
  change (existing consumers skip unknown variants)
- Changing the hash algorithm or domain tag IS a breaking change

This is the first wire format designed under the post-v1.0
versioning regime.
