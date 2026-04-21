# Kudos v2 Wire Format — multi-family ledger entries

Sprint 23 Phase F — wire format spec (pre-launch design-only, no code).

- **Status** : design-only. No code lands S23.
- **Depends on** : `CONTRIBUTION_FAMILIES_V1.md` (companion design doc).
- **Consumed by** : LT-1 implementation sprint (post-v1.0).

---

## 1. Current state (Kudos v1)

The current `KudosEntry` (Python coordinator, SQLite) stores:

```python
class KudosEntry:
    node_id: str          # Ed25519 hex
    project_id: str       # project pkarr hex
    task_count: int       # monotonic
    last_updated_ts: int  # unix seconds
```

Signed with `DOMAIN_KUDOS_V1` domain separation. Single dimension
(task_count). No decay, no families.

## 2. Kudos v2 wire shape (proposed)

```rust
pub struct KudosEntryV2 {
    pub node_id: [u8; 32],
    pub family: ContributionFamily,
    pub project_id: Option<[u8; 32]>,  // None for relay (network-wide)
    pub raw_score: u64,
    pub timestamp_ts: i64,             // when this contribution occurred
    pub entry_sig: [u8; 64],           // coordinator Ed25519 over JCS
                                       // with DOMAIN_KUDOS_V2 (reserved)
}

pub enum ContributionFamily {
    Compute = 0,
    Storage = 1,
    Relay = 2,
}
```

### Field semantics

| Field | Description |
|---|---|
| `node_id` | Contributing node's Ed25519 pubkey |
| `family` | Which contribution family this entry belongs to |
| `project_id` | For Compute/Storage: the project benefiting. For Relay: `None` (network service) |
| `raw_score` | Family-specific raw contribution units (task_count for Compute, bytes_served/1024 for Storage, messages_relayed for Relay) |
| `timestamp_ts` | UTC unix seconds of contribution event |
| `entry_sig` | Coordinator signature over canonical JCS (coordinator is the ledger authority pre-v1.0) |

### Domain separation

```rust
pub const DOMAIN_KUDOS_V2: &[u8] = b"nexus-kudos-v2";
```

Reserved. Not defined in code until LT-1 implementation. The existing
`DOMAIN_KUDOS_V1` remains for backward compatibility during the
transition window (if any nodes have cached v1 entries at that point).

### Pre-launch note

Since no deployment exists pre-v1.0, the migration path is simply:
drop v1 table, create v2 table. No online migration needed. The
`DOMAIN_KUDOS_V2` is a separate domain tag (not a version bump of v1)
because the signing surface is structurally different (new fields
change the canonical bytes shape).

## 3. Canonical bytes

Same pattern as all SBFB signed payloads:

```
DOMAIN_KUDOS_V2 <0x00> <serde_jcs::to_vec(unsigned_entry)>
```

The `unsigned_entry` is the full `KudosEntryV2` with `entry_sig`
replaced by `[0u8; 64]` (or empty string if JSON representation).

## 4. Storage schema (SQLite)

```sql
CREATE TABLE kudos_v2 (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id BLOB NOT NULL,           -- 32 bytes
    family INTEGER NOT NULL,         -- 0=Compute, 1=Storage, 2=Relay
    project_id BLOB,                 -- 32 bytes or NULL
    raw_score INTEGER NOT NULL,
    timestamp_ts INTEGER NOT NULL,
    entry_sig BLOB NOT NULL,         -- 64 bytes
    created_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_kudos_v2_node ON kudos_v2(node_id);
CREATE INDEX idx_kudos_v2_family ON kudos_v2(family, timestamp_ts);
CREATE INDEX idx_kudos_v2_project ON kudos_v2(project_id) WHERE project_id IS NOT NULL;
```

### Aggregation query (on-demand, not materialized)

```sql
SELECT
    node_id,
    family,
    SUM(
        CASE family
            WHEN 0 THEN raw_score * POWER(0.5, (strftime('%s','now') - timestamp_ts) / (90.0 * 86400))
            WHEN 1 THEN raw_score * MAX(0, 1.0 - (strftime('%s','now') - timestamp_ts) / (180.0 * 86400))
            WHEN 2 THEN raw_score * POWER(0.5, (strftime('%s','now') - timestamp_ts) / (60.0 * 86400))
        END
    ) AS decayed_score
FROM kudos_v2
WHERE timestamp_ts > strftime('%s','now') - (365 * 86400)  -- 1-year horizon
GROUP BY node_id, family;
```

The 1-year horizon prunes entries whose decayed contribution rounds
to effectively zero (< 0.1% of original for exponential decay).

## 5. Composite kudos computation

```python
def composite_kudos(node_id: bytes, weights: WeightVector) -> float:
    scores = query_decayed_scores(node_id)  # {family: decayed_score}
    return (
        weights.compute * scores.get(Family.COMPUTE, 0) +
        weights.storage * scores.get(Family.STORAGE, 0) +
        weights.relay * scores.get(Family.RELAY, 0)
    )
```

Where `WeightVector` is loaded from coordinator config (governance-
tunable). Default: `{compute: 0.50, storage: 0.30, relay: 0.20}`.

## 6. Migration strategy

Since the project has no live deployment, migration is trivial:

1. Create `kudos_v2` table alongside existing `kudos` table.
2. Backfill: for each existing `KudosEntry`, insert one
   `KudosEntryV2` row with `family=Compute`, `raw_score=task_count`,
   `timestamp_ts=last_updated_ts`, and re-sign with `DOMAIN_KUDOS_V2`.
3. Drop `kudos` table.
4. Rename diagnostic endpoint to serve v2 Gini per-family.

This happens atomically at the LT-1 implementation sprint. No
rolling upgrade needed (pre-launch policy).

## 7. Non-goals

- Token, currency, or transferable value semantics (cf.
  `feedback_kudos_non_monetary.md`).
- Real-time streaming of kudos updates to peers (ledger is
  coordinator-local, queried on-demand via loopback API).
- Per-entry revocation (entries are append-only; the coordinator
  can quarantine a node_id which zeroes its composite score).

## 8. References

- Companion: `docs/fairness/CONTRIBUTION_FAMILIES_V1.md`
- Current ledger: `packages/nexus-coordinator/src/nexus_coordinator/kudos/`
- Fairness endpoint: `packages/nexus-coordinator/src/nexus_coordinator/fairness.py`
- Domain constants: `crates/nexus-core-rs/src/canonical.rs`
- Memory: `fairness_vision.md`, `feedback_kudos_non_monetary.md`
