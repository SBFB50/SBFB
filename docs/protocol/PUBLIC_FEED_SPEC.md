# SBFB Public Feed Protocol Specification

**Version:** `FEED_FORMAT_VERSION = 1`
**Status:** Sprint 64 — complete (§1-9 initial S61, §10-12 hardening S64)
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

**`is_open_source` validation rule:** `true` requires the complete
verification chain: `repo_url + commit_sha + artifact_hash +
provenance_hash`. Enforced by `validate_feed_operation()` at
insert time. A `ReleasePublished` with `is_open_source: true` but
missing `provenance_hash` is rejected.

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

## 5. Trust model

### 5.1 Local vs. remote trust

The feed operates under two trust regimes depending on the
source of entries:

**Local DB (self-authored entries):**
Trust is implicit. Entries written by `insert_feed_operation()`
are signed and hashed at write time — the local process is the
sole writer and the chain is maintained atomically (BEGIN
IMMEDIATE transaction). The materializer MAY skip verification
for local entries (performance optimization), though
`materialize_verified()` always verifies.

**Remote sync (entries received from peers):**
Trust nothing — verify everything. For each received entry:

1. **Ed25519 signature** over canonical bytes (reject if invalid)
2. **entry_hash** recomputation from canonical (reject if mismatch)
3. **Per-author prev_hash** chain linkage (each author's entries
   form an independent chain, verified separately)
4. **Field format validation** (project_id hex-64, repo_url
   HTTPS, commit_sha hex-40, artifact_hash hex-64)
5. **Deduplication** by entry_hash (skip if already present)

This is the SSB model: per-feed (per-author) append-only
verification with Ed25519. The global feed interleaves entries
from multiple authors; verification is per-author, not global.

### 5.2 Multi-author chain verification

`verify_chain()` accepts entries from N authors. For each entry,
it tracks the last known `entry_hash` per author. An author's
first entry must have `prev_hash = "genesis"`. Subsequent
entries must chain to that author's previous entry_hash.

This enables replay verification of a merged feed containing
entries from multiple nodes without requiring a single global
chain ordering.

---

## 6. Replay rules

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

## 7. Cursor format

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

## 8. Test vectors

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

Expected canonical bytes: domain prefix `nexus-feed-v1` + null
byte `0x00` + JCS-serialized JSON (deterministic key ordering per
RFC 8785).

Expected entry hash (BLAKE3 of canonical bytes, hex-encoded):

```
f81ced7da512d9615a63e67e99b70fa89a1116b7101c0d3f313d83caf569ae2a
```

This value is verified by
`test_compute_feed_entry_hash_deterministic` in `public_feed.rs`.

---

## 9. Versioning policy

- `FEED_FORMAT_VERSION` starts at `1`
- Each breaking change to the canonical format bumps the version
- Decoders SHOULD accept `version <= FEED_FORMAT_VERSION` (range)
- New optional fields carry `#[serde(default)]` for forward
  compatibility within the same version
- Changing the hash algorithm or domain tag IS a breaking change

This is the first wire format designed under the post-v1.0
versioning regime.

### 9.1 Forward compatibility (raw-op)

Adding a new operation type is **NOT a breaking change**.
`FEED_FORMAT_VERSION` does not bump for new op types.

Since Sprint 65, `FeedEntry.op` is stored as a raw
`serde_json::Value` instead of a typed enum. This enables
forward compatibility:

- Nodes **MUST** store and propagate unknown `op_type` values
  without interpretation. Unknown ops are carried in the
  hash-chain and signed feed just like known ops.
- Nodes **MUST** verify hash-chain integrity (`entry_hash`) and
  Ed25519 signature for entries with unknown `op_type`. The
  cryptographic verification is independent of the payload
  semantics.
- Nodes **MUST NOT** interpret or act on unknown `op_type` values.
  Unknown ops are stored for replay but do not affect the
  materialized `PublicRegistryView`.
- The `op_type` field inside the JSON object serves as discriminant
  (same role as the former `#[serde(tag = "op_type")]` annotation).
- Known ops (`ReleasePublished`, `SourceBecameStale`) are validated
  semantically at insert time. Unknown ops pass a size check only
  (`MAX_OPERATION_JSON_SIZE`).

---

## 10. Adversarial scenarios & mitigations

The following attack vectors are covered by deterministic tests
(Sprint 62–64). Each row names the test function and the
defense layer that rejects the attack.

### 10.1 Feed-level attacks

| # | Vector | Test | Defense |
|---|--------|------|---------|
| 1 | **Fork-bomb spam** — same author floods 1000+ operations | `test_adversarial_fork_bomb_spam_rejected` | Per-author rate limiter (`FEED_RATE_LIMIT_PER_MINUTE = 5`). Ops beyond quota return `rate limit exceeded`. |
| 2 | **Payload oversized** — operation JSON > 64 KB | `test_adversarial_payload_oversized_rejected` | `validate_feed_operation()` rejects payloads exceeding `MAX_OPERATION_JSON_SIZE`. |
| 3 | **Bad repo URL** — `javascript:`, `file:///`, `ftp://`, empty | `test_adversarial_bad_repo_url_rejected` | URL validation requires `https://` scheme with non-empty path. |
| 4 | **Bad artifact hash** — non-hex, wrong length, null bytes | `test_adversarial_bad_artifact_hash_rejected` | Hex-64 regex validation on `artifact_hash` (and `provenance_hash`). |
| 5 | **Seq gap injection** — entry with fabricated `prev_hash` | `test_adversarial_seq_gap_detection` | `verify_chain()` tracks per-author last hash; broken linkage detected. |
| 6 | **Cross-author forgery** — attacker signs entry claiming another author | `test_adversarial_cross_author_forgery_rejected` | `verify_entry()` verifies Ed25519 signature against declared `author_pubkey`. Mismatch = rejected. |

### 10.2 Cryptographic attacks

| # | Vector | Test | Defense |
|---|--------|------|---------|
| 7 | **Ed25519 forgery** — random 64-byte signature | `test_adversarial_ed25519_forgery_feed_entry` | `verify_entry()` recomputes canonical bytes and verifies Ed25519 signature. Random bytes fail. |
| 8 | **BLAKE3 tamper** — 1-bit change in canonical field | `test_adversarial_blake3_tamper_canonical` | `verify_entry()` recomputes `BLAKE3(canonical_bytes)` and compares to stored `entry_hash`. Any field change causes mismatch. |
| 9 | **PoW nonce brute-force** — random nonces vs 16-bit difficulty | `test_adversarial_pow_nonce_difficulty_check` | `verify_feed_pow()` checks leading zero bits. Random nonces fail with overwhelming probability (< 0.2% pass rate). |
| 10 | **Future timestamp** — entry timestamp > now + 30 days | `test_adversarial_future_timestamp_rejected` | `validate_feed_entry_timestamp()` rejects entries more than 30 days in the future. Entries up to 1h ahead are tolerated (clock skew). |

### 10.3 Chain integrity (pre-S64 baseline)

| # | Vector | Test | Defense |
|---|--------|------|---------|
| 11 | **Forged signature on chain** | `test_verify_chain_forged_signature` | `verify_chain()` validates every entry signature during replay. |
| 12 | **Tampered hash in chain** | `test_verify_chain_tampered_hash` | `verify_chain()` recomputes and compares every `entry_hash`. |
| 13 | **Multi-author interleaving** | `test_verify_chain_multi_author` | Per-author chain tracking (SSB model): each author's `prev_hash` links to their own last entry. |
| 14 | **Out-of-order insertion** | `test_verify_chain_out_of_order_insertion` | `verify_chain()` detects seq ordering violations. |
| 15 | **Cursor hash mismatch** | `test_cursor_hash_mismatch_triggers_full_rebuild` | Materializer detects corrupt cursor and triggers full replay from `seq = 0`. |

### 10.4 Not covered (scope-cut)

- **Fuzzing** (cargo-fuzz / proptest): deferred to post-v1.0 audit
  preparation (Sprint 65+).
- **Sybil attack** (many distinct authors): requires curator
  vouching system (`CuratorVouched` operation, Sprint 65).
- **Eclipse attack** (feed partition): requires multi-peer sync
  protocol hardening (Sprint 65+).
- **Replay attack** (re-inject old valid entries): mitigated by
  deduplication on `entry_hash` at insert time, but no formal
  anti-replay beyond dedup.

---

## 11. New node bootstrap procedure

A fresh node with no prior state joins the network and
reconstructs the full public registry. This procedure is
validated by the E2E test `test_new_node_full_sync_and_verify`
(gated behind `SBFB_INTEGRATION=1`).

### 11.1 Algorithm

```
1. SPAWN    — New daemon starts with empty SQLite DB.
2. TICKET   — Obtain a feed document ticket from an existing peer
               (GET /api/daemon/feed/ticket on the seed node).
3. JOIN     — POST /api/daemon/feed/join with the ticket.
               The daemon creates a local iroh-docs replica and
               starts subscribing to the document.
4. SYNC     — iroh-docs syncs all entries from the seed peer.
               The daemon's feed_subscribe loop ingests entries
               as they arrive, inserting them into the local
               feed_entries table via insert_remote_feed_entry().
5. VERIFY   — For each received entry:
               a) Recompute canonical_bytes from entry fields
               b) Verify BLAKE3(canonical_bytes) == entry_hash
               c) Verify Ed25519(author_pubkey, canonical_bytes, signature)
               d) Verify per-author prev_hash chain linkage
               e) Validate field formats (hex lengths, URL scheme)
               f) Deduplicate by entry_hash
6. REBUILD  — The FeedMaterializer replays verified entries to
               reconstruct the Browse registry view (project list
               with latest release info).
7. CURSOR   — Save cursor (last_seq, last_entry_hash) for
               incremental materialization on subsequent syncs.
```

### 11.2 Failure modes

| Failure | Recovery |
|---------|----------|
| Ticket invalid or expired | Retry with a different seed peer |
| Sync timeout (> 60s no new entries) | `feed_subscribe` timeout + backoff + re-subscribe |
| Entry fails verification (step 5) | Entry is rejected, does not enter local DB. Chain continues from last valid entry. |
| Cursor hash mismatch on resume | Full replay from `seq = 0` (§7 safety fallback) |
| Seed peer disappears mid-sync | iroh-docs handles peer disconnection; re-subscribe on next tick |

### 11.3 Invariants

- A fresh node MUST verify all entries before materializing
  (no trust-on-first-use for remote entries).
- The node's local feed is eventually consistent with peers —
  entry ordering is per-author, not globally sequenced.
- The cursor is only saved after successful materialization,
  preventing partial state on crash.

---

## 12. Security considerations

### 12.1 Threat model summary

The feed inherits the project-level threat model
(`docs/security/THREAT_MODEL.md`) with feed-specific additions:

- **T-FEED-INTEGRITY**: An attacker modifies a feed entry in
  transit or at rest. Mitigated by BLAKE3 hash-chain + Ed25519
  signatures on every entry. Tampering is detectable at
  verification time (§4, §10.2).

- **T-FEED-SPAM**: An attacker floods the feed with operations
  to exhaust storage or hide legitimate entries. Mitigated by
  per-author rate limiter (5 ops/min, §10.1 #1) and payload
  size limit (64 KB, §10.1 #2).

- **T-FEED-FORGERY**: An attacker publishes entries under another
  author's identity. Mitigated by Ed25519 signature verification
  against the declared `author_pubkey` (§10.1 #6, §10.2 #7).

- **T-FEED-CLOCK-SKEW**: An attacker sets entry timestamps far
  in the future to manipulate ordering or stale detection.
  Mitigated by the 30-day future timestamp gate (§10.2 #10).

### 12.2 Trust boundaries

```
┌─────────────────────────────────────────────┐
│ Local process (trusted)                     │
│  insert_feed_operation() → sign → hash →    │
│  atomic SQLite insert                       │
└──────────────────┬──────────────────────────┘
                   │ iroh-docs sync (untrusted transport)
┌──────────────────▼──────────────────────────┐
│ Remote entries (untrusted)                  │
│  insert_remote_feed_entry() → verify_entry  │
│  → validate fields → dedup → insert         │
└─────────────────────────────────────────────┘
```

Local entries are trusted because the process is the sole writer.
Remote entries are verified before insertion — no entry from a
peer bypasses `verify_entry()`.

### 12.3 Cryptographic primitives

| Primitive | Algorithm | Usage |
|-----------|-----------|-------|
| Signing | Ed25519 (RFC 8032) | Author signs canonical bytes of each entry |
| Hashing | BLAKE3 | Entry hash (integrity) + hash-chain linkage |
| Canonical | RFC 8785 JCS | Deterministic serialization for signing/hashing |
| Domain separation | `DOMAIN_FEED_V1` prefix + null byte | Prevents cross-domain signature reuse |
| Proof-of-work | Leading zero bits on `BLAKE3(entry_hash \|\| nonce)` | Optional anti-spam (16-bit difficulty) |

### 12.4 Residual risks

- **No Sybil resistance** until `CuratorVouched` operations are
  implemented (Sprint 65). Any Ed25519 keypair can author entries.
- **No feed-level quarantine** for suspicious authors (Sprint 65).
- **No auth-tier check** on `insert_feed_operation()` — the
  endpoint accepts inserts without verifying the caller's
  permission level (P2-FEED-INSERT-NO-AUTH-TIER, mandatory
  Sprint 65).
- **Single-peer bootstrap** — a new node trusts the seed peer
  to provide the complete feed. Multi-peer cross-validation is
  future work.
