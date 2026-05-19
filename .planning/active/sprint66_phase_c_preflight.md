# Sprint 66 Phase C — preflight G8

Date : 2026-05-19 | HEAD : `543eb45` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : "pick deepest technical option" + "OSS prior art OBLIGATOIRE avant chaque phase (G10)" — applicable, S1a scan complet execute.
- feedback_context7_systematic.md : "context7 avant tout code touchant lib/API/spec" — tokio docs queries pour watch/JoinHandle pattern.
- sprint14_keyoxide_decision.md : "deploy from source, clone+Keyoxide+SLSA L1" — Phase C D5 (cross-node verification) est directement alignee : la cle du deployer est dans le record, la verification utilise cette cle. Coherent.
- nexus_grid_pivot.md : "Pre-launch protocol policy : *_FORMAT_VERSION restent a 1" — Phase C ne touche aucune VERSION. Coherent.
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 5 projets/patterns recherches (SSB, Automerge, npm attestation, Sigstore Rekor, Keyoxide), APPROACH-ALIGNED — clean
- S1b deps : 5 libs scannees (serde_json, hex, ed25519-dalek, tokio, chrono), 0 delta CVE — clean
- S2 historiques : 5 fichiers cibles, 14 commits bodies lus — clean
- S3 threat model : FULL, 7 vectors analyses — clean
- S4 wire format : FULL / VERSION=1, Day 0 preserved — clean

---

## S1a — OSS prior art deep analysis

### Projets analyses en profondeur

#### [SSB — Secure Scuttlebutt] (https://github.com/ssbc/ssb-server)
- Pattern : append-only log, local persistence = source of truth, replication via peers. At boot, rebuild index by replaying local log. Peers receive entries via gossip/replication only for entries present in the peer's log.
- Alignment Phase C : feed republish at boot from SQLite to iroh-docs mirrors SSB's "local log is the master, replication layer is secondary". One-shot synchronous replay before accepting connections = SSB pattern.
- Edge cases : SSB handles out-of-order entries via hash-chain (same as SBFB BLAKE3 chain). SSB does NOT rate-limit boot replay (consistent with plan — replay is local data, not remote).

#### [Automerge] (https://github.com/automerge/automerge)
- Pattern : CRDT log replay at boot from persistence to sync distributed state. Boot is synchronous — the node must be consistent before accepting remote changes.
- Alignment Phase C : synchronous boot replay before HTTP server start is the Automerge pattern. Not lazy/async.

#### [npm attestation / Sigstore] (https://github.com/npm/provenance + sigstore.dev)
- Pattern : provenance verification distinguishes 3 states: (1) no attestation/bundle present, (2) attestation present + valid, (3) attestation present + invalid. npm CLI `npm audit signatures` outputs "missing attestation" vs "invalid attestation" distinctly. Sigstore verify returns `VerificationResult { Verified, NotVerified(reason), NoBundle }`.
- Alignment Phase C : `status: "absent" | "verified" | "failed"` maps directly to the 3-state pattern. APPROACH-ALIGNED.
- Evidence : npm attestation README states "Provenance transparency in npm allows users to see where their package was built and how it arrived". The distinction between absent and failed is a first-class concern.

#### [Sigstore Rekor] (https://github.com/sigstore/rekor)
- Pattern : transparency log verification extracts the certificate/identity from the log entry itself, not from the verifier's local keystore. "Trust the record's identity, verify the crypto against it."
- Alignment Phase C D5 : extracting pubkey from `record.node_id` instead of using local `pow_keypair.public_bytes()` = same pattern. Cross-node verification is the SLSA L1 intended usage: the provenance contains the builder identity, the verifier extracts it.

#### [tokio task management patterns] (docs.rs/tokio + cybernetist.com)
- Pattern : `watch::channel` for shutdown signaling to spawned tasks. JoinHandle stored for graceful await at shutdown. Fire-and-forget spawn = resource leak anti-pattern (task cannot be awaited/cancelled).
- Alignment Phase C : plan proposes `Vec<JoinHandle>` + watch channel for feed_join, mirrors `spawn_feed_subscribe` (L293 feed_sync.rs) which already uses this pattern. APPROACH-ALIGNED.
- The codebase already implements this pattern correctly for `spawn_feed_subscribe` — Phase C extends it to `feed_join`.

### Tableau comparatif

| Aspect | Plan Phase C | SSB | npm/Sigstore | tokio pattern |
|--------|-------------|-----|--------------|---------------|
| Boot replay | synchrone, one-shot, avant HTTP | synchrone, log replay | N/A | N/A |
| Provenance states | 3 (absent/verified/failed) | N/A | 3 (missing/valid/invalid) | N/A |
| Cross-node verify | pubkey from record | inherent (key in log entry) | cert from transparency log | N/A |
| JoinHandle tracking | Vec + watch shutdown | N/A | N/A | standard, recommended |

### Finding S1a
- Classification : APPROACH-ALIGNED
- Evidence : 4/4 sub-deliverables align with mature OSS patterns
- Impact sur le plan : aucun

---

## S2 — Decision chain reconstruction

### Fichiers scannes
- `feed_sync.rs` : 5 commits lus (594855a, cedadd3, cd7c46a, 587016f, d391b1b)
- `http.rs` (provenance section) : 4 commits lus (e362092, 272523c, 9b8abfa, fa7cd52)
- `runtime.rs` : 3 commits lus (f3ea1c3, 118ada0, 2b57d37)
- `provenance.rs` : 2 commits lus (e362092, 9b8abfa)

### Decisions historiques trouvees

#### Decision 1 : P2-VERIFY-LOCAL-KEY-ONLY
- Sprint 63 Phase B, sha `e362092` : provenance endpoint uses `state.pow_keypair.public_bytes()` for verification. Comment in review: "P2-VERIFY-LOCAL-KEY-ONLY : verification cle locale seulement, cross-node carry S64".
- Sprint 64/65 : carried forward without change.
- Sprint 66 kickoff D5 : explicitly plans to fix this by extracting pubkey from `record.node_id`.
- Reverse-commit check : no reversion found, no contradicting decision.
- Status : active carry, Phase C resolves it.
- Impact phase : aucun (Phase C is the resolution).

#### Decision 2 : P2-PROVENANCE-404-BRIDGE
- Sprint 63 Phase C review, sha `5b6ec41` : "P2-PROVENANCE-404-BRIDGE cosmetic".
- Sprint 64/65 : carried forward as cosmetic P2.
- Sprint 66 kickoff D4 : escalated to 3/3 MANDATORY, plans 3-state provenance.
- Reverse-commit check : no reversion found.
- Status : active carry 3/3 MANDATORY, Phase C resolves it.
- Impact phase : aucun.

#### Decision 3 : feed_join fire-and-forget (P2-FEED-JOIN-HANDLE-LEAK)
- Sprint 62 Phase B, sha `594855a` : `tokio::spawn` in `feed_join` without storing JoinHandle.
  Body extrait : "P2 : publish_feed_entry_to_docs() pas auto-cable"
- Identified as P2 in subsequent reviews.
- Sprint 66 kickoff D3 : plans JoinHandle tracking + shutdown channel.
- Reverse-commit check : no reversion found.
- Status : active carry 2/3, Phase C escalates to 3/3 and resolves.
- Impact phase : aucun.

#### Decision 4 : Feed replay_all() is a synchronous SQLite read
- Sprint 61 Phase A : `replay_all()` function introduced as synchronous `Vec<FeedEntry>` return from SQLite.
- No decision to make it async or paginated.
- Phase C uses it for boot-time one-shot replay. <100 entries expected (R3 risk acknowledged).
- Status : active design, no contradiction.
- Impact phase : aucun.

### Memory constraints
- feedback_approach.md : "pick deepest" — Phase C resolves 2 MANDATORY carries and 1 resource leak. Deepest option.
- sprint14_keyoxide_decision.md : "deploy from source" — cross-node verification (D5) is consistent with the S14 provenance design where `node_id` in the record IS the deployer identity.
- feedback_context7_systematic.md : context7 query done for tokio watch/JoinHandle pattern.
- nexus_grid_pivot.md : pre-launch protocol policy — no VERSION bump, no wire format change. Consistent.

---

## S3 — Threat model analysis

### Primitive analysee : feed republish + provenance cross-node verification + feed_join handle fix

### Assets en jeu
- A1 Feed integrity (high) : feed entries in SQLite republished to iroh-docs must be authentic (signed, hash-chained).
- A2 Provenance integrity (high) : Ed25519 signature verification must use the correct public key (deployer's, not verifier's).
- A3 Daemon availability (medium) : feed_join handle leak can accumulate orphan tasks without bound.

### Threat actors
- TA1 Noeud byzantin P2P (AD3 in THREAT_MODEL) : could send malformed provenance records with invalid node_id hex.
- TA2 Extension navigateur (AD1) : could call provenance endpoint to observe verification status of projects.

### Attack vectors identifies

1. V1 Injection via crafted node_id hex in ProvenanceRecord : if `hex::decode(&record.node_id)` produces 32 bytes that happen to be a valid Ed25519 pubkey different from the deployer, verification could pass against wrong key.
   - Mitigation : the node_id in the record was set by the deployer at deploy time (signed into the canonical bytes). Tampering the node_id invalidates the signature. No regression.
   - Coverage : covered by existing `verify_provenance()` which checks signature against canonical bytes including node_id.

2. V2 Replay feed entries at boot : replaying SQLite entries to iroh-docs could re-publish stale entries.
   - Mitigation : entries are hash-chained (prev_hash + seq). Dedup by entry_hash UNIQUE index in SQLite + iroh-docs key = `feed/{author}/{seq}`. No duplicate possible. No regression.

3. V3 DoS via feed_join handle accumulation (pre-fix) : current code has no bound on spawned feed_join tasks.
   - Mitigation (Phase C fix) : JoinHandle tracked + shutdown channel allows cleanup. The plan mentions rate-limiting max 10 joins (R4 risk).

4. V4 Information leakage via status field : the new `status: "absent" | "verified" | "failed"` field reveals whether provenance exists for a project.
   - Assessment : provenance is public data (included in the zip archive, published via gossip). No new information leakage. The previous 404 response already revealed absence.

5. V5 node_id hex decode panic : malformed hex in `record.node_id` could panic the daemon if unwrap is used.
   - Mitigation : plan explicitly uses pattern matching with fallback `verified: false` on decode failure. No unwrap. R5 risk acknowledged.

6. V6 Temporal attack on boot replay timing : a concurrent feed_join during boot replay could cause race between SQLite read and iroh-docs write.
   - Mitigation : boot replay is synchronous BEFORE HTTP server starts (plan §6.2: "One-shot synchrone avant spawn HTTP server"). No race possible — HTTP handlers that accept feed_join are not yet serving.

7. V7 Supply chain : no new deps introduced. All existing deps in perimetre (hex, serde_json, tokio) have no open CVE.

### Mitigations existantes
- T-FEED-INTEGRITY (THREAT_MODEL §10) couvre V1, V2 : hash-chain + Ed25519 per-entry signature.
- T-FEED-SPAM (THREAT_MODEL §10) couvre V3 : rate-limiter per-author.
- Provenance signing (DOMAIN_PROVENANCE_V1) couvre V1 : domain-separated Ed25519 signature.

### Gaps identifies
- GAP0 : none identified. All 7 vectors are covered by existing or Phase C mitigations.

### Regression check
- No existing mitigation T0-T5 is weakened by Phase C changes.
- The cross-node verification (D5) IMPROVES security : previously, verification of non-local provenances always returned `verified: false` (false negative). After Phase C, it correctly verifies using the deployer's key.
- No new uncovered vector created.

### Verdict S3 : clean

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase C does not modify canonical.rs. All 13 DOMAIN_*_V1 constants remain unchanged. The `canonical_bytes<T>()` function is not touched.

### Structs verifiees

#### ProvenanceRecord (provenance.rs:17-29)
- schema_version = PROVENANCE_SCHEMA_VERSION = 1 : OK, unchanged
- serde derives : Serialize, Deserialize, Debug, Clone, PartialEq, Eq : OK
- serde(default) on app_version : present, with `skip_serializing_if = "Option::is_none"`. Rationale : runtime tolerance for records without app_version. Legitimate.
- DOMAIN_PROVENANCE_V1 signature : used in `canonical_bytes()` within `provenance.rs:102-124`. Preserved.
- JCS serialization : provenance uses manual JSON construction in `canonical_bytes()` (not `serde_json::to_string` directly). OK.
- New fields : NONE. Phase C does not modify ProvenanceRecord. The `status` field is in the HTTP JSON response, not in the struct.
- Option<T> usage : `app_version: Option<String>` — correct.

#### FeedEntry (public_feed.rs)
- version = FEED_FORMAT_VERSION = 1 : OK, unchanged
- Phase C does not modify FeedEntry struct.
- `replay_all()` reconstructs FeedEntry from DB rows with `version: FEED_FORMAT_VERSION`. Correct.

#### HTTP response JSON (not a wire format struct)
- The `status` field added to GET /provenance response is a JSON-only extension.
- `verified: bool` preserved for backward compat.
- This is NOT a P2P wire format change — it's a local HTTP API response.

### Day 0 check
- D1 (data_dir) : Phase A delivered. Phase C builds on it (republish presupposes persistence). No contradiction.
- D2 (FsStore) : Phase A delivered. No contradiction.
- D3 (feed republish + feed_join handle) : Phase C delivers this. Consistent.
- D4 (provenance 3 states MANDATORY) : Phase C delivers this. Consistent.
- D5 (cross-node verification MANDATORY) : Phase C delivers this. Consistent.
- Decisions actees pivot.md : aucune contredite.

### Pre-launch policy
- `*_VERSION` = 1 : all 13+ VERSION constants checked, none modified.
- No tolerant decoder multi-version : not introduced.
- No tests "legacy decode" zombie : not introduced.
- `#[serde(default)]` : not added in Phase C scope.
- Pre-launch protocol preserved.

### Version constants audit
- FEED_FORMAT_VERSION = 1 (public_feed.rs:20) : unchanged
- PROVENANCE_SCHEMA_VERSION = 1 (provenance.rs:15) : unchanged
- All other *_VERSION constants : unchanged (grep verified, 40+ hits all = 1 or existing values)

---

## Telemetrie preflight (agent deep)

- Duree totale : ~12min
- S1a : 5 projets OSS analyses (SSB, Automerge, npm provenance, Sigstore Rekor, tokio patterns) / 0 fichiers source lus via raw GitHub (used WebSearch + context7 for pattern extraction) / ~500 LOC reviewees via WebSearch results / 1 context7 queries (tokio watch/JoinHandle) / 6 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 5 libs scannees (serde_json, hex, ed25519-dalek, tokio, chrono) / 4 CVE searches / finding : clean (0 CVE, 0 breaking changes)
- S2 : 14 commits bodies lus / 0 archive files (current sprint) / 5 memory files / finding : clean (all carries consistent, no contradictions)
- S3 : FULL / 7 vectors analyses / 0 gaps / finding : clean
- S4 : FULL / 2 structs verifiees (ProvenanceRecord, FeedEntry) / canonical.rs lu integralement : oui / finding : clean

## Action

Proceder code phase C.
