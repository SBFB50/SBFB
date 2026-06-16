# P2P Protocol, Wire Formats, Crypto & iroh Integration

**Analysis Date:** 2026-05-18

---

## 1. iroh 0.98 Integration

### Node Bootstrap

Every SBFB node boots a full iroh protocol stack via `crates/nexus-core-rs/src/node.rs`:

- **Endpoint**: `iroh::Endpoint` with `presets::N0` (pkarr DHT discovery automatic)
- **Docs**: `iroh_docs::protocol::Docs` (replicated key/value logs for tasks/results)
- **Gossip**: `iroh_gossip::net::Gossip` (HyParView + PlumTree broadcast trees)
- **Blobs**: `iroh_blobs::BlobsProtocol` with `MemStore` (content-addressed storage)
- **Router**: `iroh::protocol::Router` multiplexes incoming QUIC connections by ALPN

ALPNs registered:
- `iroh_blobs::ALPN` (BLOBS_ALPN)
- `iroh_gossip::ALPN` (GOSSIP_ALPN)
- `iroh_docs::ALPN` (DOCS_ALPN)

**Node identity**: Ed25519 keypair. Random by default; persistent via `NodeConfig::with_secret_key()`. Node ID = 64 hex chars of the public key.

**Persistence**: `NodeConfig::with_data_dir()` persists iroh-docs replica (`docs.redb`) and default author. Blobs use in-memory `MemStore` (no persistence yet).

**Custom relays**: `SBFB_CUSTOM_RELAYS` env or `~/.sbfb/relays.json` overrides the N0 preset relay set. Loaded by `crates/nexus-core-rs/src/relay_config.rs`.

**Memory lookup**: `MemoryLookup` registered on every endpoint for out-of-band peer address seeding (from BlobTickets, DocTickets).

### iroh-docs Usage

Wrapped by `crates/nexus-core-rs/src/docs.rs`:

- **Authors**: `author_create()`, `author_default()`, `author_list()`
- **Documents**: `create_doc()`, `import_ticket()`, `import_and_subscribe()`, `open_doc()`, `drop_doc()`, `list_docs()`
- **Entry I/O**: `set()`, `get_exact()`, `get_many_by_prefix()`, `get_many_latest_per_key_prefix()`, `get_latest_by_key()`
- **Live sync**: `subscribe()` returns `Stream<Item = LiveEvent>`
- **Sharing**: `share_write()`, `share_read()` produce `DocTicket`

Key prefixes used in task docs: `"task:"`, `"claim:"`, `"result:"`.

iroh-docs is LWW (last-write-wins by timestamp). Two workers can write `ClaimEntry` for the same task concurrently; the coordinator breaks ties by earliest timestamp.

### iroh-gossip Usage

Wrapped by `crates/nexus-core-rs/src/gossip.rs`:

- `GossipClient::join_topic(topic_bytes, bootstrap)` -- blocking join (waits for NeighborUp)
- `GossipClient::subscribe_topic(topic_bytes, bootstrap)` -- non-blocking subscribe
- `GossipClient::join_topic_with_age_witness(...)` -- Couche 1 age-admission gate
- `GossipClient::join_topic_with_pow(...)` -- PoW-gated admission
- `TopicHandle::broadcast(message)` / `TopicHandle::next_event()`
- `TopicHandle::split()` -> `(TopicSender, TopicReceiver)`

**Topic ID derivation**:
- Per-curator: `BLAKE3("nexus-grid/curator/" || curator_pubkey)[..32]`
- Per-project heartbeat: `BLAKE3("nexus-grid/project/" || project_pubkey || "/announce")[..32]`
- Key rotation: `BLAKE3("nexus-grid/key-rotation/v1")[..32]` (topic string: `KEY_ROTATION_TOPIC`)
- Warrant canary: `BLAKE3("nexus-grid/warrant-canary/v1")[..32]`
- Duress ack: `BLAKE3("nexus-grid/canary-duress-ack/v1")[..32]`

### iroh-blobs Usage

Wrapped by `crates/nexus-core-rs/src/blobs.rs`:

- `BlobsClient::add_bytes(data)` -> BLAKE3 content hash `[u8; 32]`
- `BlobsClient::get_bytes(hash)` -> `Vec<u8>`
- `BlobsClient::has(hash)` -> bool
- `BlobsClient::fetch_ticket(endpoint, memory_lookup, ticket_str)` -> download from peer

**Curator list flow**: curator publishes signed JSON blob via `add_bytes`, announces the BlobTicket on gossip, subscribers `fetch_ticket` + `get_bytes` to parse.

---

## 2. Canonical Serialization

**File**: `crates/nexus-core-rs/src/canonical.rs`

All signed payloads use RFC 8785 JSON Canonicalization Scheme (JCS) with type-specific domain separation:

```text
<domain bytes> <0x00> <serde_jcs::to_vec(value)>
```

The `0x00` separator prevents a crafted value from smuggling the domain into its own JSON payload.

### Domain Tags (15 total)

| Constant | Bytes | Used For |
|----------|-------|----------|
| `DOMAIN_TASK_V1` | `b"nexus-task-v1"` | `Task` |
| `DOMAIN_RESULT_V1` | `b"nexus-result-v1"` | `ResultPayload` |
| `DOMAIN_CLAIM_V1` | `b"nexus-claim-v1"` | `Claim` |
| `DOMAIN_INVITE_V1` | `b"nexus-invite-v1"` | Invite payloads |
| `DOMAIN_KUDOS_V1` | `b"nexus-kudos-v1"` | Kudos ledger entries |
| `DOMAIN_CURATOR_LIST_V1` | `b"nexus-curator-list-v1"` | `CuratorList` |
| `DOMAIN_PROVENANCE_V1` | `b"nexus-provenance-v1"` | Provenance attestations |
| `DOMAIN_WARRANT_CANARY_V1` | `b"nexus-warrant-canary-v1"` | Warrant canary |
| `DOMAIN_POW_V1` | `b"nexus-pow-v1"` | Hashcash PoW challenge |
| `DOMAIN_DURESS_ACK_V1` | `b"nexus-duress-ack-v1"` | Duress ack heartbeat |
| `DOMAIN_AGE_WITNESS_V1` | `b"nexus-age-witness-v1"` | Sybil-resistance Couche 1 |
| `DOMAIN_CONTRIBUTOR_ATTESTATION_V1` | `b"nexus-contributor-attestation-v1"` | Couche 2 governance |
| `DOMAIN_KEY_ROTATION_V1` | `b"nexus-key-rotation-v1"` | Key rotation |
| `DOMAIN_DELEGATION_CERT_V1` | `b"nexus-delegation-cert-v1"` | Couche 3 delegation |
| `DOMAIN_FEED_V1` | `b"nexus-feed-v1"` | Public feed entries |

Cross-type replay is impossible by construction: a signature valid under one domain is invalid under any other.

### JCS Rationale

Rust's `serde_json` emits struct fields in declaration order; Python's `json.dumps(sort_keys=True)` emits alphabetically. JCS standardizes key ordering, number formatting, string escaping, and whitespace. Both sides use the same canonical bytes.

---

## 3. Wire Format Types

### 3.1 Task (`crates/nexus-core-rs/src/task.rs`)

**Version**: `TASK_FORMAT_VERSION = 1` (stays at 1 until v1.0 tag)

```rust
pub struct Task {
    pub version: u16,
    pub task_id: String,
    pub task_type: String,        // "analysis", "summary", etc.
    pub prompt: String,
    pub system_prompt: String,
    pub model: String,            // "llama-3.1-8b"
    pub priority: u8,             // 1=highest, 10=lowest
    pub created_at: u64,          // unix seconds
    pub parent_task_id: String,   // "" if no parent
    pub metadata: BTreeMap<String, String>,
    pub is_open_source: bool,     // #[serde(default)]
    pub estimated_watts: u32,     // #[serde(default)]
    pub estimated_vram_mb: u64,   // #[serde(default)]
    pub estimated_hours: f64,     // #[serde(default)]
    pub redundancy_factor: u8,    // #[serde(default = "1")]
    pub watermark_seed: Vec<u8>,  // #[serde(default)]
}
```

**Signing**: `TaskEntry` wraps `Task` + `author_pubkey: [u8; 32]` + `signature: [u8; 64]`. Signed with `DOMAIN_TASK_V1`. The `redundancy_factor` field is excluded from canonical bytes (dispatch-only policy, not cryptographic identity).

### 3.2 Claim (`crates/nexus-core-rs/src/task.rs`)

```rust
pub struct Claim {
    pub version: u16,
    pub task_id: String,
    pub claimed_by: [u8; 32],  // worker pubkey
    pub claimed_at: u64,
}
```

**Signing**: `ClaimEntry` wraps `Claim` + `worker_pubkey: [u8; 32]` + `signature: [u8; 64]`. Signed with `DOMAIN_CLAIM_V1`. Verification checks `claim.claimed_by == worker_pubkey` (attribution consistency) before signature.

### 3.3 ResultPayload (`crates/nexus-core-rs/src/task.rs`)

```rust
pub struct ResultPayload {
    pub version: u16,
    pub task_id: String,
    pub result_text: String,
    pub tokens_generated: u64,
    pub generation_time_ms: u64,
    pub model_digest: [u8; 32],    // BLAKE3 of model NAME (S76 doc-note; weights-file digest = S77/llm_llama_cpp)
    pub logprobs_hash: [u8; 32],   // BLAKE3 of calibration logprobs
    pub started_at: u64,
    pub finished_at: u64,
    pub output_token_ids: Vec<u32>, // #[serde(default)]
}
```

**Signing**: `ResultEntry` wraps `ResultPayload` + `worker_pubkey: [u8; 32]` + `signature: [u8; 64]`. Signed with `DOMAIN_RESULT_V1`.

### 3.4 TaskResponse (`crates/nexus-core-rs/src/schemas/task_response.rs`)

The structured response the LLM backend emits:

```rust
pub struct TaskResponse {
    pub version: u8,            // TASK_RESPONSE_VERSION = 1
    pub domain: String,         // "TASK_RESPONSE_V1"
    pub content: String,
    pub reasoning: Option<String>,
    pub tool_calls: Vec<ToolCall>,
}
```

`#[serde(deny_unknown_fields)]` rejects extra keys. JSON Schema generated via `schemars` for constrained LLM generation.

### 3.5 CuratorList (`crates/nexus-core-rs/src/curator.rs`)

**Version**: `CURATOR_LIST_FORMAT_VERSION = 1`

```rust
pub struct CuratorList {
    pub version: u16,
    pub curator_pubkey: [u8; 32],
    pub curator_name: String,
    pub created_at: u64,
    pub revision: u64,          // monotonic, rollback-protected
    pub entries: Vec<CuratorProjectRef>,
}

pub struct CuratorProjectRef {
    pub project_id: String,     // pkarr node id hex, 64 chars
    pub project_name: String,
    pub category: String,
    pub description: String,
}
```

**Limits**:
- `CURATOR_LIST_MAX_ENTRIES = 256`
- `CURATOR_PROJECT_ID_MAX = 128` bytes
- `CURATOR_PROJECT_NAME_MAX = 128` bytes
- `CURATOR_CATEGORY_MAX = 64` bytes
- `CURATOR_DESCRIPTION_MAX = 280` bytes

**Signing**: `CuratorListEntry` wraps `CuratorList` + `curator_pubkey: [u8; 32]` + `signature: [u8; 64]`. Signed with `DOMAIN_CURATOR_LIST_V1`. Verification checks: version == 1, entries <= 256, field byte caps, attribution consistency, Ed25519 signature.

**Revocation integration**: `verify_with_revocation(cache, now_ts)` checks key rotation status. `verify_with_contributor_registry(registry)` checks Couche 2 governance-strong flag.

### 3.6 FeedEntry (`crates/nexus-coordinator-rs/src/public_feed.rs`)

**Version**: `FEED_FORMAT_VERSION = 1`

```rust
pub struct FeedEntry {
    pub version: u16,
    pub seq: u64,               // auto-increment
    pub op: PublicFeedOperation, // tagged union
    pub author_pubkey: String,   // Ed25519 hex
    pub timestamp: u64,
    pub entry_hash: String,      // BLAKE3 hex
    pub prev_hash: String,       // BLAKE3 hex or "genesis"
    pub signature: String,       // Ed25519 hex
    pub pow_nonce: Option<u64>,  // #[serde(default)], transport anti-spam
}

pub struct FeedEntryCanonical {
    pub version: u16,
    pub op: PublicFeedOperation,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub prev_hash: String,
}

pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
}
```

**Hash-chain**: `entry_hash = hex(BLAKE3(DOMAIN_FEED_V1 || 0x00 || JCS(FeedEntryCanonical)))`. Genesis: `prev_hash = "genesis"`.

**Multi-author**: each author's entries form an independent chain. `verify_chain()` rebuilds per-author chains via prev_hash linkage.

**Validation**: `validate_feed_operation()` enforces project_id hex-64, repo_url HTTPS, commit_sha hex-40, artifact_hash hex-64, valid stale reasons, `MAX_OPERATION_JSON_SIZE = 65_536` bytes. `is_open_source: true` requires `provenance_hash`.

**Rate limiting**: `FEED_RATE_LIMIT_PER_MINUTE = 5` per author. Enforced by `insert_feed_operation_rate_limited()`.

**Timestamp validation**: `FEED_MAX_FUTURE_SECS = 30 days`. Entries more than 30 days in the future are rejected.

**Feed PoW**: `FEED_POW_DIFFICULTY = 16` bits. `BLAKE3(entry_hash_ascii || nonce_le_bytes)` must have 16+ leading zero bits.

### 3.7 KeyRotationAnnouncement (`crates/nexus-core-rs/src/key_rotation.rs`)

**Version**: `KEY_ROTATION_FORMAT_VERSION = 1`

```rust
pub struct KeyRotationAnnouncement {
    pub version: u16,
    pub old_public_key: [u8; 32],
    pub new_public_key: [u8; 32],
    pub timestamp: u64,
    pub reason: String,          // max 280 bytes
    pub transition_days: u16,    // max 90 days
}
```

**Signing**: `SignedKeyRotation` wraps announcement + `signature: [u8; 64]`. Signed by the **old** keypair with `DOMAIN_KEY_ROTATION_V1`. Gossip topic: `"nexus-grid/key-rotation/v1"`.

**Revocation**: `RevocationCache` (in-memory HashMap) tracks `old_pubkey -> RevocationEntry`. `is_revoked(pk, now)` = true when transition window expired. `is_in_transition(pk, now)` = true during window. Stale rotation rejected (timestamp must be strictly greater).

### 3.8 ProvenanceRecord (`crates/nexus-coordinator-rs/src/provenance.rs`)

**Version**: `PROVENANCE_SCHEMA_VERSION = 1`

```rust
pub struct ProvenanceRecord {
    pub schema_version: u32,
    pub repo_url: String,
    pub commit_sha: String,
    pub artifact_hash: String,
    pub node_id: String,
    pub timestamp: String,      // RFC 3339
    pub signature: String,      // Ed25519 hex
    pub app_version: Option<String>, // not part of signed bytes
}
```

Canonical bytes: `DOMAIN_PROVENANCE_V1 || 0x00 || JSON(artifact_hash, commit_sha, node_id, repo_url, schema_version, timestamp)` (alphabetical key order via `serde_json::json!`).

### 3.9 Warrant Canary (`crates/nexus-shell-daemon-core/src/canary/mod.rs`)

**Version**: `CANARY_VERSION = 1`

```rust
pub struct CanarySigned {
    pub version: u16,       // "v" on wire
    pub date: String,       // YYYY-MM-DD
    pub headline: String,   // max 512 bytes
    pub next_update: String, // date + 45 days
    pub pubkey_hex: String,
}

pub struct Canary {
    pub signed: CanarySigned, // flattened via #[serde(flatten)]
    pub signature_hex: String,
}
```

Signed with `DOMAIN_WARRANT_CANARY_V1`. Wire bytes: JCS canonical (`serde_jcs::to_vec`). Also mirrored as `CANARY.txt` in repo root.

**Frequency**: monthly, `CANARY_VALIDITY_DAYS = 45`. Missed publication = dead-man-switch alarm.

**Signing**: supports both single-key (`Ed25519CanarySigner`) and threshold K-of-N (`FrostCanarySigner` via FROST RFC 9591).

### 3.10 DuressAck (`crates/nexus-shell-daemon-core/src/canary/duress_ack.rs`)

Short heartbeat signed daily by the maintainer, broadcast on `"nexus-grid/canary-duress-ack/v1"` gossip topic. Signed with `DOMAIN_DURESS_ACK_V1`.

### 3.11 KudosEntry (`crates/nexus-coordinator-rs/src/types.rs`)

```rust
pub struct KudosEntry {
    pub entry_id: String,
    pub worker_node_id: String,
    pub task_id: String,
    pub project_id: String,
    pub amount: u64,           // log_utility(tokens_generated)
    pub created_at: u64,
    pub prev_hash: String,     // BLAKE3 hash-chain
    pub entry_hash: String,
}
```

Hash-chain: `BLAKE3(DOMAIN_KUDOS_V1 || 0x00 || JCS(hashable fields))`. Per-project chain starting from `"genesis"`.

**Log utility**: `amount = 1000 * log2(1 + tokens)`. EMA decay: `KUDOS_EMA_ALPHA = 0.97`.

---

## 4. Crypto Operations

### Ed25519 Signatures

**File**: `crates/nexus-core-rs/src/crypto.rs`

- Library: `ed25519-dalek`
- Key format: 32-byte secret, 32-byte public
- Signature format: 64 bytes (standard Ed25519)
- `KeyPair::generate()` uses `OsRng`
- `KeyPair::load_or_generate(path)` persists raw 32-byte secret to file (0600 Unix perms)
- `verify(public_key, message, signature)` -> `Result<()>`

All signing goes through `canonical_bytes(value, domain)` first.

### BLAKE3 Hashing

- `blake3_hash(data)` -> `[u8; 32]`
- `Blake3Chain` -- append-only hash chain: `H_{i+1} = BLAKE3(H_i || entry_i)`, genesis = all zeros
- Used by: kudos ledger, public feed, content-addressed blobs

### SHA256 Hashcash PoW

**File**: `crates/nexus-core-rs/src/pow.rs`

- `HashcashChallenge` binds topic + publisher_pubkey + issued_at + difficulty
- `solve(challenge, timeout)` -> brute-force nonce search
- `verify(proof)` / `verify_at(proof, now)` / `verify_stateless(proof)`
- Hash: `SHA256(canonical_bytes(challenge) || nonce_le_bytes)`
- `leading_zero_bits(hash)` must meet difficulty

**Constants**:
- `DEFAULT_DIFFICULTY_BITS = 18` (~100ms on modern CPU)
- `MAX_DIFFICULTY_BITS = 30`
- `MAX_PROOF_AGE_SECS = 1800` (30 min)
- `POW_FORMAT_VERSION = 1`

**Escalating difficulty**: `EscalatingPolicy` ramps difficulty geometrically per (consumer, model) tranche. Resets daily at midnight UTC.

### Feed PoW

**File**: `crates/nexus-coordinator-rs/src/public_feed.rs`

- `FEED_POW_DIFFICULTY = 16` bits (~10-50ms)
- Hash: `BLAKE3(entry_hash_ascii || nonce_le_bytes)`
- Transport-level anti-spam, not part of canonical signed bytes

---

## 5. PoW-Gated Gossip Wire Envelope

**File**: `crates/nexus-core-rs/src/pow_gossip.rs`

### Wire Format

```text
+-------------------------------+------------------+-----------+
| proof_len (u32 BE, 4 bytes)  | proof JSON bytes | payload   |
+-------------------------------+------------------+-----------+
```

- `proof_len`: u32 big-endian
- `proof bytes`: `serde_json::to_vec(&HashcashProof)` (NOT JCS; JCS only for the inner pre-image)
- `payload`: arbitrary bytes (the actual gossip message)

### Publisher Cache (`PowSolveCache`)

- Keyed by 32-byte topic id
- Session window: `SESSION_WINDOW = 15 min`
- Solve timeout: `SOLVE_TIMEOUT = 30 sec`
- `ensure_proof(topic, keypair, policy)` -- solve once, reuse within window

### Subscriber Cache (`PowVerifyCache`)

- Keyed by `(publisher_pubkey, topic)`
- `DashMap` for thread-safe concurrent access
- `verify_envelope(bytes, policy, now)` -- full verify on first message, cache hit for session

### Security Properties

- **Publisher-bound**: proof includes `publisher_pubkey`, non-replayable across identities
- **Topic-bound**: proof includes `topic`, non-replayable across topics
- **Time-bound**: `issued_at` checked against `MAX_PROOF_AGE_SECS`
- **Payload integrity NOT covered**: PoW is cost-of-identity, not payload integrity. Payload integrity comes from signed types (curator lists, tasks, etc.)

---

## 6. Sybil-Resistance Composition (3 Couches)

### Couche 0 -- Hashcash PoW (Sprint 19)

Every gossip publisher solves a SHA256 Hashcash puzzle before broadcasting. Default 2^18 difficulty. Publisher-bound + topic-bound + time-bound.

### Couche 1 -- Age Admission Gate (Sprint 22)

**File**: `crates/nexus-core-rs/src/gossip.rs`

`AgeWitness` peer-attests that a `node_id` was first seen at `first_seen_ts`.

Decision tree in `evaluate_age_admission()`:
1. Bootstrap allowlist node -> admitted (pre-v1.0 bootstrap ceremony)
2. Valid witness provided, node age >= `MIN_AGE_DAYS` (7), witness age >= `MIN_WITNESS_AGE_DAYS` (30) -> admitted
3. No witness and not bootstrap -> PoW-only fallback (removed at v1.0)
4. Present-but-invalid witness -> hard reject (security signal)

### Couche 2 -- Governance-Strong Contributor Gate (Sprint 22)

`ContributorAttestation` (in-toto v1.0 predicate) signed by coordinator at verified-deploy time. Asserts a `node_id` completed at least one verified-deploy for a project.

`CuratorListEntry::verify_with_contributor_registry(registry)` checks each enrolled project entry.

Opt-in per-project: projects not enrolled bypass the gate.

### Couche 3 -- Multi-Forge Git-Log Cross-Validation (Sprint 23+)

`DelegationCert` binds SBFB Ed25519 `node_id` to SSH/PGP signing key fingerprint. Design-only Sprint 23; runtime wiring Sprint 24-27.

---

## 7. Curator System

### List Publication Flow

1. Curator builds `CuratorList` with project entries
2. Signs via `CuratorListEntry::sign(list, keypair)`
3. Stores as iroh blob via `BlobsClient::add_bytes()`
4. Announces `BlobTicket` on per-curator gossip topic
5. Subscribers `fetch_ticket()` + `get_bytes()` + parse + `verify_signature()`

### Verification Stack

1. `version == CURATOR_LIST_FORMAT_VERSION` (reject unknown versions)
2. `entries.len() <= CURATOR_LIST_MAX_ENTRIES` (DoS cap)
3. Per-field byte caps (project_id 128, name 128, category 64, description 280)
4. Attribution consistency: `list.curator_pubkey == envelope.curator_pubkey`
5. Ed25519 signature valid over `canonical_bytes(list, DOMAIN_CURATOR_LIST_V1)`
6. Optional: revocation check (`verify_with_revocation()`)
7. Optional: contributor registry check (`verify_with_contributor_registry()`)

### Rollback Protection

`revision` is monotonic. Shell-daemon runtime keeps `DashMap<curator_pubkey, latest_entry>` and refuses any revision <= stored.

---

## 8. Public Verifiable Feed System

### Specification

Full spec: `docs/protocol/PUBLIC_FEED_SPEC.md`

### Architecture

- **Storage**: SQLite via `CoordinatorDb` (`crates/nexus-coordinator-rs/src/db.rs`)
- **Hash-chain**: BLAKE3, per-author independent chains, genesis sentinel `"genesis"`
- **Signing**: Ed25519 over `DOMAIN_FEED_V1` canonical bytes
- **Anti-spam**: rate limiting (5/min/author) + PoW (16-bit BLAKE3)
- **Timestamp guard**: reject entries >30 days in the future

### Operations

| Operation | Fields |
|-----------|--------|
| `ReleasePublished` | project_id (hex-64), repo_url (HTTPS), commit_sha (hex-40), artifact_hash (hex-64), provenance_hash (optional hex-64), is_open_source |
| `SourceBecameStale` | project_id (hex-64), reason (repo_unreachable/commit_diverged/manual) |

Future: `CuratorVouched`, `BuildQuorumReached`, `SourceRecovered`, `SearchManifestPublished`

### Insert Paths

- **Local trust** (`insert_feed_operation`): validates semantics, no rate limiting. For self-authored entries.
- **Remote trust** (`insert_feed_operation_rate_limited`): validates semantics + enforces `FEED_RATE_LIMIT_PER_MINUTE`. For peer-received entries.

### Verification

- `verify_entry(entry)`: recompute entry_hash, verify Ed25519 signature. Does NOT check chain linkage.
- `verify_chain(entries)`: full multi-author chain verification. Rebuilds per-author chains via prev_hash -> entry_hash linkage. Detects broken linkage, forks, and forgeries.
- `validate_feed_entry_timestamp(entry, now)`: reject entries >30 days in future.

### Replay

`replay_all(db)` returns all entries ordered by seq. Feed can be replayed from genesis to reconstruct `PublicRegistryView`.

---

## 9. 3-Layer Proof-of-Computation Verification

**File**: `crates/nexus-core-rs/src/verification.rs`

### Layer 1 -- Ed25519 Signature (WHO produced it)

`ResultEntry::verify_signature()`. Failure: trust -50, auto-ban.

### Layer 2 -- Model Digest Whitelist (WHICH model ran)

BLAKE3 of the model NAME compared against `digest_whitelist` HashMap (S76 Phase C doc-note: the worker hashes the model name, not the weights file; a real weights-file digest is gated on `llm_llama_cpp`, S77, and `Verifier` has no live caller — the live path is the quorum over `result_text`). Failure: trust -50, auto-ban.

### Layer 3 -- Logprob Fingerprint (DID the model actually run)

BLAKE3 hash of calibration logprob fingerprint compared against `logprob_profiles`. Failure: trust -5, no ban (suspect).

### Spot-Check Rate

- trust >= 80: 1% (trusted)
- trust >= 50: 5% (standard)
- otherwise: 20% (suspect)

---

## 10. Blob-Serve (Web App Archives)

**File**: `crates/nexus-shell-daemon-core/src/blob_serve.rs`

### Archive Format

- **Input**: zip file stored as iroh blob
- **Served via**: `GET /blob-serve/{hash}/{path}`
- **Cache**: `BlobServeCache` with LRU eviction (`DEFAULT_MAX_CACHE_ENTRIES = 32`)
- **Decompression limit**: `DEFAULT_MAX_DECOMPRESSED_BYTES = 100 MB`

### Security

- **Path traversal**: `validate_zip_path()` rejects `..`, absolute paths, backslash
- **Zip bomb**: total decompressed size capped
- **CSP**: `connect-src 'none'; worker-src 'none'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; sandbox allow-scripts`
- **COOP**: `same-origin`
- **COEP**: `require-corp`

### Content-Type Detection

Extension-based (html, js, css, json, svg, png, jpg, gif, webp, ico, woff, woff2, ttf, otf, wasm, xml, txt, map) + magic bytes fallback (PNG, JPEG, GIF, WEBP, WASM) + HTML content sniffing.

---

## 11. Deploy Verification (SLSA L1)

**File**: `crates/nexus-coordinator-rs/src/provenance.rs`

### Flow

1. Coordinator clones repo from source
2. Builds artifact (zip archive)
3. Computes `BLAKE3(artifact)` -> `artifact_hash`
4. Signs provenance attestation with coordinator's Ed25519 keypair
5. Produces `provenance.json` (SLSA L1)

### Verification

`verify_provenance(json, public_key)` parses JSON, extracts signature, recomputes canonical bytes with `DOMAIN_PROVENANCE_V1`, verifies Ed25519.

### Provenance Hash

`provenance_blake3_hex(record)` = `BLAKE3(pretty_json(record))`. Used as `provenance_hash` in feed `ReleasePublished` entries.

---

## 12. Warrant Canary & FROST DKG

### Single-Key Canary

`Ed25519CanarySigner` -- maintainer's persistent keypair at `<sbfb_home>/canary-key.key`. Monthly publication, 45-day validity.

### FROST Threshold Canary

**Files**: `crates/nexus-shell-daemon-core/src/canary/frost.rs`, `dkg.rs`, `ceremony.rs`

- `frost_keygen_trusted_dealer(min_signers, max_signers)` -- DKG via FROST RFC 9591
- `FrostCanarySigner` -- K-of-N threshold signing (e.g., K=2/N=3 cross-jurisdiction)
- `ceremony_round1()`, `ceremony_round2()`, `ceremony_aggregate()` -- interactive ceremony
- Produces standard 64-byte Ed25519 signature, indistinguishable on the wire

---

## 13. Key Management

### KeyPair (`crates/nexus-core-rs/src/crypto.rs`)

- `KeyPair::generate()` -- OsRng
- `KeyPair::from_secret_bytes([u8; 32])`
- `KeyPair::load_or_generate(path)` -- 32-byte raw binary file, 0600 perms on Unix
- `secret_bytes()` / `public_bytes()` / `sign(message)`

### Keystore (`crates/nexus-core-rs/src/keystore.rs`)

Encrypted keystore with:
- Argon2id KDF (`ARGON2_MEM_COST_KIB`, `ARGON2_TIME_COST`, `ARGON2_PARALLELISM`)
- AES-256-GCM encryption
- Support for normal + duress identities (`KEYRING_ACCOUNT_NORMAL`, `KEYRING_ACCOUNT_DURESS`)
- OS keyring integration (`KEYRING_SERVICE = "sbfb"`)
- Blob format: `BLOB_MAGIC` + `BLOB_VERSION` + salt + nonce + ciphertext + tag
- Env override: `SBFB_IDENTITY_SECRET_HEX_ENV`

### Key Rotation (`crates/nexus-core-rs/src/key_rotation.rs`)

- `KeyRotationAnnouncement` signed by old key, names new key
- Transition window: `DEFAULT_TRANSITION_DAYS = 7`, max 90
- `RevocationCache` tracks in-memory (SQLite persistence deferred S26)
- Gossip topic: `"nexus-grid/key-rotation/v1"`

---

## 14. Discovery & Connectivity

### pkarr DHT (Automatic)

iroh 0.98 `presets::N0` automatically publishes/resolves via pkarr DHT. No explicit SBFB code needed.

### DiscoveryClient (`crates/nexus-core-rs/src/discovery.rs`)

- `my_node_id()` -- 64-char hex of Ed25519 public key
- `my_addr()` -> `NodeAddrInfo` (node_id, relay_url, direct_addresses)
- `my_endpoint_addr()` -> raw `EndpointAddr`
- `probe_reachable(endpoint_id_hex, timeout)` -- connect probe via blobs ALPN

### DNS Fallback (`crates/nexus-core-rs/src/dns_fallback.rs`)

DoH (DNS-over-HTTPS) + DoT (DNS-over-TLS) fallback to Cloudflare/Google when pkarr is unavailable.

### TLS Pinning (`crates/nexus-core-rs/src/tls_pinning.rs`)

SPKI SHA256 pin validation for relay connections.

### Tor Transport (`crates/nexus-core-rs/src/tor_transport.rs`)

Optional Tor transport via arti-client.

### Relay Configuration (`crates/nexus-core-rs/src/relay_config.rs`)

Custom relay list via `SBFB_CUSTOM_RELAYS` env or `~/.sbfb/relays.json`.

### pkarr Quorum Resolver (`crates/nexus-core-rs/src/pkarr_resolver.rs`)

Multi-relay quorum resolution for redundant pkarr lookups.

### DHT Quorum (`crates/nexus-core-rs/src/dht_quorum.rs`)

`redundant_resolve()` queries N resolvers and requires majority agreement.

---

## 15. Invite System

**File**: `crates/nexus-coordinator-rs/src/invite.rs`

### InviteRecord

```rust
pub struct InviteRecord {
    pub id: String,
    pub wire: String,           // serialized invite token
    pub scope: String,          // "worker" or "observer"
    pub project_id: String,
    pub project_name: String,
    pub expires_at: i64,
    pub max_uses: Option<i64>,
    pub uses_count: i64,
    pub revoked_at: Option<i64>,
    pub note: Option<String>,
    pub created_at: i64,
    pub tasks_doc_ticket: Option<String>,
}
```

SQLite-backed. `InviteLedger` provides `mint()`, `revoke()`, `get()`, `list()`.

---

## 16. Quarantine Queue

**File**: `crates/nexus-coordinator-rs/src/quarantine_queue.rs`

SQLite-backed queue for borderline gossip messages. Tracks topic, sender pubkey, payload JSON, rate strikes, PoW status, flush status. TTL-based expiry.

---

## 17. Capability System

**File**: `crates/nexus-coordinator-rs/src/capability_store.rs`

Gate-off-by-default TOML store. Known capabilities: `biometric_gate`, `federation_canary`, `mcp_server_expose`, `rag_retrieval`, `streaming_bridge`, `tool_calling`. SHA256 integrity hash.

---

## 18. Version Policy (Pre-Launch)

All format versions stay at 1 until the `v1.0` tag:

| Type | Version Constant | Current Value |
|------|------------------|---------------|
| Task/Result | `TASK_FORMAT_VERSION` | 1 |
| CuratorList | `CURATOR_LIST_FORMAT_VERSION` | 1 |
| PoW | `POW_FORMAT_VERSION` | 1 |
| KeyRotation | `KEY_ROTATION_FORMAT_VERSION` | 1 |
| Feed | `FEED_FORMAT_VERSION` | 1 |
| Provenance | `PROVENANCE_SCHEMA_VERSION` | 1 |
| Canary | `CANARY_VERSION` | 1 |
| TaskResponse | `TASK_RESPONSE_VERSION` | 1 |

No tolerant multi-version decoder. `v == 1` only. No legacy decode tests. `#[serde(default)]` is for runtime robustness only (e.g., client sends minimal JSON).

Post-v1.0: each break bumps version, decoders accept a range, `#[serde(default)]` for ascending compat.

---

## 19. Gossip Topics Summary

| Topic Seed | Purpose |
|------------|---------|
| `"nexus-grid/curator/" + pubkey` | Per-curator list announcements |
| `"nexus-grid/project/" + pubkey + "/announce"` | Project heartbeats |
| `"nexus-grid/key-rotation/v1"` | Key rotation announcements |
| `"nexus-grid/warrant-canary/v1"` | Monthly warrant canary |
| `"nexus-grid/canary-duress-ack/v1"` | Daily duress ack heartbeat |

All topic IDs: `BLAKE3(seed)[..32]`.

---

## 20. Security Constants Summary

| Constant | Value | Location |
|----------|-------|----------|
| `DEFAULT_DIFFICULTY_BITS` | 18 | `pow.rs` |
| `MAX_DIFFICULTY_BITS` | 30 | `pow.rs` |
| `MAX_PROOF_AGE_SECS` | 1800 (30 min) | `pow.rs` |
| `SESSION_WINDOW` | 15 min | `pow_gossip.rs` |
| `SOLVE_TIMEOUT` | 30 sec | `pow_gossip.rs` |
| `MIN_AGE_DAYS` | 7 | `attestations/age_witness.rs` |
| `MIN_WITNESS_AGE_DAYS` | 30 | `attestations/age_witness.rs` |
| `CANARY_VALIDITY_DAYS` | 45 | `canary/mod.rs` |
| `MAX_HEADLINE_LEN` | 512 | `canary/mod.rs` |
| `CURATOR_LIST_MAX_ENTRIES` | 256 | `curator.rs` |
| `REASON_MAX_BYTES` | 280 | `key_rotation.rs` |
| `MAX_TRANSITION_DAYS` | 90 | `key_rotation.rs` |
| `FEED_POW_DIFFICULTY` | 16 | `public_feed.rs` |
| `FEED_RATE_LIMIT_PER_MINUTE` | 5 | `public_feed.rs` |
| `FEED_MAX_FUTURE_SECS` | 2592000 (30 days) | `public_feed.rs` |
| `MAX_OPERATION_JSON_SIZE` | 65536 | `public_feed.rs` |
| `DEFAULT_MAX_DECOMPRESSED_BYTES` | 104857600 (100 MB) | `blob_serve.rs` |

---

*Protocol analysis: 2026-05-18*
