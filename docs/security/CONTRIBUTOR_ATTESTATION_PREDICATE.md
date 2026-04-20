# ContributorAttestation — predicate spec

Sprint 22 Phase C — Sybil-resistance Couche 2 binaire.
**Livrable pre-code obligatoire** (plan §6.2.2 P0-G1-2 ack).

- **Status** : stable (pre-launch redefinition policy per
  [`CLAUDE.md §Pre-launch protocol policy`](../../CLAUDE.md))
- **Predicate URI** : `https://nexus-grid.org/contributor-attestation/v1`
- **Version** : `v1` (no bump before first `v1.0` tag)
- **Signing domain** : `DOMAIN_CONTRIBUTOR_ATTESTATION_V1 = b"nexus-contributor-attestation-v1"`
- **Depends on** : [`in-toto/attestation` v1.0 spec](https://github.com/in-toto/attestation/blob/main/spec/v1/README.md),
  [ProvenanceRecord SBFB S14](../../packages/nexus-coordinator/src/nexus_coordinator/provenance.py)

---

## 1. Motivation

The ContributorAttestation proves that a given `node_id` has
successfully completed a **verified deploy** (S14 Keyoxide + SLSA
L1 flow) for a given project. It is a binary "voice-per-contributor"
primitive : either a node_id is a verified contributor for a
project (0/1), or it is not.

This attestation is the Sybil-resistance Couche 2 primitive
arbitrated 2026-04-19 (kickoff §4 D1). It complements :

- **Couche 1** (`DOMAIN_AGE_WITNESS_V1`) — gossip admission via
  node age ≥ 7 days peer-witnessed + PoW S19.
- **Couche 3** (`DelegationCert`, design-only S22, implem
  S23-S27) — multi-forge cross-validation and git-log-based
  contribution weight.

The primitive is **voluntary** : the coordinator signs the
attestation at verified-deploy time (inside `api/deploy.py` after
`generate_provenance()`). No scheduler, no automatic periodic
signing — the threat-model decision `04c9621` (Sprint 18 Phase E2)
forbidding automatic canary signing is preserved by construction
(§6 verify).

### Scope non-goals

- **Not** a proof of fair contribution weight (Matthew effect
  "one layer deeper" remains : high-kudos workers publish more
  projects, get more attestations). See §8 limitations.
- **Not** a replacement for git-log provenance — that lives in
  Couche 3 S23+.
- **Not** a sybil-resistance layer for the public gossip mesh (that
  is Couche 1). ContributorAttestation gates **opt-in
  governance-strong** project curator lists, nothing else.

## 2. Predicate type URI

```
predicateType: https://nexus-grid.org/contributor-attestation/v1
```

The URI is **stable** pre-launch and will not be bumped until the
first `v1.0` tag. Breaking changes before `v1.0` redefine the v1
semantics (cf. CLAUDE.md Pre-launch protocol policy). After
`v1.0`, this URI becomes immutable — any semantic change bumps
the path segment to `/v2`.

Conformance target : [`in-toto/attestation` v1.0 predicate layer]
(https://github.com/in-toto/attestation/blob/main/spec/v1/predicate.md).
The enclosing envelope uses the in-toto v1.0 Statement structure
(§5 below).

## 3. JSON Schema (draft-07)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://nexus-grid.org/contributor-attestation/v1/predicate.schema.json",
  "title": "ContributorPredicate",
  "description": "Coordinator-signed attestation that `contributor_node_id` successfully completed a verified-deploy (SLSA L1 + Keyoxide) for the enclosing subject project at `commit_sha`.",
  "type": "object",
  "additionalProperties": false,
  "required": [
    "contributor_node_id",
    "first_deploy_ts",
    "commit_sha",
    "repo_url",
    "attestation_coord_sig"
  ],
  "properties": {
    "contributor_node_id": {
      "description": "Ed25519 public key of the contributor, hex-encoded (64 chars = 32 bytes).",
      "type": "string",
      "pattern": "^[0-9a-f]{64}$"
    },
    "first_deploy_ts": {
      "description": "UNIX timestamp (seconds since epoch, UTC) of the first successful verified-deploy for this (project, contributor) pair. Later deploys for the same pair do NOT replace this field — it is the anchor.",
      "type": "integer",
      "minimum": 0
    },
    "commit_sha": {
      "description": "Git SHA-1 of the commit the first verified-deploy was built from. Hex-encoded lowercase (40 chars). Pattern: `^[0-9a-f]{40}$`. SHA-256 migration path deferred until git ecosystem consensus (cf. §8).",
      "type": "string",
      "pattern": "^[0-9a-f]{40}$"
    },
    "repo_url": {
      "description": "Canonical source-of-truth URL of the repository (same as ProvenanceRecord.repo_url). Used for multi-forge cross-validation in Couche 3 S23+.",
      "type": "string",
      "format": "uri"
    },
    "attestation_coord_sig": {
      "description": "Ed25519 signature of the canonical-bytes-encoded predicate (without this field), base64-std-encoded (88 chars = 64 bytes + padding). Signed under domain tag DOMAIN_CONTRIBUTOR_ATTESTATION_V1.",
      "type": "string",
      "pattern": "^[A-Za-z0-9+/]{86}==$"
    }
  }
}
```

## 4. Fields

| Field | Type | Semantics |
|---|---|---|
| `contributor_node_id` | hex 64 | Ed25519 public key of the contributor. Bytes-identical to the node_id used in gossip + task dispatch + ProvenanceRecord. |
| `first_deploy_ts` | int64 | Anchor timestamp of the very first verified-deploy by this contributor for this project. Re-deploys do not update this value — a new deploy by the same contributor for the same project re-uses the existing attestation (the coordinator looks up the registry before signing). |
| `commit_sha` | hex 40 | The git commit SHA (SHA-1) the attestation references. Fixed at first-deploy time. |
| `repo_url` | URI | Source-of-truth repo URL. Used for Couche 3 S23+ git log cross-validation and multi-forge trust-web. Not signed over per attestation (it is part of the signed predicate, so it is tamper-proof). |
| `attestation_coord_sig` | base64 88 | Ed25519 signature over the canonical JCS-encoded predicate with `attestation_coord_sig` set to the empty string (pattern S14 `provenance.py:_signable_payload`). Domain-separated under `DOMAIN_CONTRIBUTOR_ATTESTATION_V1`. |

### Signing coordinator identity

The signing coordinator is **not** re-stated in the predicate.
Verification recovers it from the enclosing in-toto Statement
envelope (§5), specifically from the `subject[].name`-linked
deploy flow. The coordinator public key is transported via the
same channel as ProvenanceRecord (gossip `ProjectAnnouncement v5`
`coord_pubkey` field + curator list Ed25519 keyset).

This matches the S14 ProvenanceRecord philosophy : the signer is
identified by the transport, not by an additional field in the
payload (prevents field-swapping attacks).

## 5. Envelope — in-toto v1.0 Statement

A ContributorAttestation is wrapped in the standard in-toto v1.0
Statement envelope :

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "nexus-grid://project/<project_id_hex>",
      "digest": {
        "blake3": "<artifact_blake3_hex>"
      }
    }
  ],
  "predicateType": "https://nexus-grid.org/contributor-attestation/v1",
  "predicate": {
    "contributor_node_id": "...",
    "first_deploy_ts": 1713556800,
    "commit_sha": "...",
    "repo_url": "...",
    "attestation_coord_sig": "..."
  }
}
```

### Subject rules

- The `subject[]` array contains **exactly one** subject entry for
  v1. Multi-subject envelopes (batching attestations across
  projects) are reserved for S23+ Couche 3 expansion.
- `subject[].name` is a `nexus-grid://` URI identifying the
  project (the canonical project_id hex).
- `subject[].digest` uses BLAKE3 (algorithm identifier `blake3`)
  matching the zip artifact hash carried by ProvenanceRecord. This
  **binds** the attestation to a specific verified-deployed
  artifact — an attacker cannot re-use the attestation for a
  different artifact even if it belongs to the same project.
- Algorithm `blake3` is non-standard in the in-toto v1.0
  DigestSet reference list but the spec allows
  application-specific algorithms (cf. v1.0 README §DigestSet).

## 6. Verification procedure (offline)

Verifiers (curator peers, third-party auditors, Amnesty
trust-web nodes S27) verify a ContributorAttestation **offline**,
without reaching the coordinator :

1. **Parse envelope** : load JSON, check `_type ==
   "https://in-toto.io/Statement/v1"`.
2. **Match predicateType** : check `predicateType ==
   "https://nexus-grid.org/contributor-attestation/v1"`.
3. **Validate schema** : predicate fields match §3 draft-07
   schema. Reject on any missing required field or pattern
   mismatch.
4. **Reconstruct signable bytes** : clone the predicate object,
   replace `attestation_coord_sig` with the empty string, encode
   via JCS (RFC 8785) `serde_jcs::to_vec`.
5. **Verify signature** : call `nexus_core.verify_bytes(canonical,
   sig_bytes, coord_pubkey)` where :
   - `canonical = DOMAIN_CONTRIBUTOR_ATTESTATION_V1 || 0x00 ||
     jcs_bytes` (pattern S14 `provenance.py`, same domain
     separation with NUL terminator).
   - `sig_bytes = base64_decode(attestation_coord_sig)`.
   - `coord_pubkey` = recovered from the transport-layer
     ProjectAnnouncement v5 or curator list entry signing pubkey.
6. **Cross-check ProvenanceRecord** (defence-in-depth) : the
   enclosing subject must match the hash of a ProvenanceRecord
   whose `(repo_url, commit_sha)` equals the predicate's
   `(repo_url, commit_sha)`. If not, reject (attestation
   mismatch).

Verification is **fail-closed** : any exception → `false`.

Rust offline API (nexus-core-rs) :

```rust
pub fn verify_contributor_attestation(
    envelope_json: &[u8],
    coord_pubkey: &[u8; 32],
) -> Result<bool, AttestationError>;
```

Python offline API (PyO3 binding) :

```python
nexus_core.verify_contributor_attestation(envelope_json: bytes,
                                          coord_pubkey: bytes) -> bool
```

## 7. Examples

### 7.1 Minimal example (happy path)

```json
{
  "_type": "https://in-toto.io/Statement/v1",
  "subject": [
    {
      "name": "nexus-grid://project/2bf1ae3c8aa04d7a8b2e0b2e3b84f6d7c4f1a8b1e3d4c5a6b7c8d9e0f1a2b3c4d",
      "digest": {
        "blake3": "5fabc5..."
      }
    }
  ],
  "predicateType": "https://nexus-grid.org/contributor-attestation/v1",
  "predicate": {
    "contributor_node_id": "a3b1c2d3e4f5061728394a5b6c7d8e9f0a1b2c3d4e5f60718293a4b5c6d7e8f9",
    "first_deploy_ts": 1713556800,
    "commit_sha": "1a2b3c4d5e6f7890abcdef1234567890abcdef12",
    "repo_url": "https://codeberg.org/alice/transLingua",
    "attestation_coord_sig": "MEUCIQDlVH..."
  }
}
```

### 7.2 Reject example (commit_sha mismatch)

Verifier receives an envelope whose `predicate.commit_sha ==
"abcd..."` but the enclosing subject BLAKE3 hash, when looked up
in the local `ProvenanceRecord` cache, maps to a different
`commit_sha == "efab..."`. Step 6 fails →
`verify_contributor_attestation` returns `Ok(false)`.

This catches the "re-use attestation for different
artifact/commit" attack.

## 8. Limitations (P2-G1-3 ack)

This attestation proves **contribution** (at least one verified-
deploy completed). It does **not** prove **fair distribution of
contribution weight**.

The Matthew effect documented in
[`docs/FAIRNESS_VISION.md §7`](../FAIRNESS_VISION.md) reappears
one layer deeper : a high-kudos worker earns more tasks,
publishes more projects, and earns more ContributorAttestations
than a low-kudos worker. Each attestation is still binary, but
the **aggregate count** of attestations a node_id holds
correlates with historical compute share.

The SBFB fairness vision addresses this via the long-term
commitment :

- [`docs/release/ROADMAP_COMMITMENTS.md §LT-1`](../release/ROADMAP_COMMITMENTS.md) —
  Kudos-v2 reform (log-utility + DRF + EMA trust) scheduled
  post-`v1.0`.
- Empirical triggers for activating LT-1 :
  Gini > 0.70 on kudos distribution, OR top-5% own > 50% of
  active tasks, OR churn × hardware correlation exceeds pre-set
  threshold.

Until LT-1 lands, code consuming ContributorAttestation
(`curator::verify_with_contributor_registry`,
`ContributorAttestation::build`) carries an inline comment :

```rust
// NOTE: Interim Sybil-resistance S22. Contributor selection
// still biased toward high-kudos workers (Matthew effect one
// layer deeper). Post-v1.0 LT-1 Kudos-v2 reform will introduce
// log-utility + DRF + EMA trust to break this cycle. See:
// - docs/FAIRNESS_VISION.md §7 "Design-conflict S22"
// - docs/release/ROADMAP_COMMITMENTS.md §LT-1
```

### Other acknowledged limitations

- **git-SHA-1 dependency** : we lock commits to SHA-1, following
  current git default. When git-ecosystem consensus adopts
  SHA-256 object IDs, a migration path bumps the predicate URI
  to `/v2` with `commit_object_id_algo + commit_object_id`
  fields.
- **Coordinator key compromise** : if the signing coordinator's
  key leaks, all past ContributorAttestations it signed become
  spoofable. Revocation list (design-only `A-S12` mitigation in
  [`HARDENING_ROADMAP §7`](HARDENING_ROADMAP.md)) covers this for
  curator lists. Parallel mechanism for ContributorAttestation
  reserved S23+ (Couche 3 multi-forge cross-validation naturally
  mitigates by independent git-log witnesses).
- **No revocation path** in v1. A contributor expelled from a
  project (legitimately or not) cannot have their attestation
  revoked. Reserved S23+.
