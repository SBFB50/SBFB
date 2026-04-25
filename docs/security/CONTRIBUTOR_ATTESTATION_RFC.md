# ContributorAttestation RFC — Couche 3 multi-forge + git-log witness

Sprint 22 Phase C — Sybil-resistance **Couche 3 (design-only)**.
Implementation distribuée S23-S27 (cf. §7 triggers).

- **Status** : design-only. No code lands S22 under this RFC.
- **Depends on** : [`CONTRIBUTOR_ATTESTATION_PREDICATE.md`](
  CONTRIBUTOR_ATTESTATION_PREDICATE.md) (Couche 2 binary
  primitive, live S22).
- **Target sprints** : S23 `SBFB.json::contributions[]` schema +
  DelegationCert primitive / S24-S25 `git log --show-signature`
  offline parser / S26 multi-forge cross-validate (GH +
  Codeberg + Forgejo) / S27 trust-web Amnesty integration.

---

## 1. Motivation and scope

Couche 2 ContributorAttestation v1 is **binary** : either the
coordinator signed an attestation for `(project_id, node_id)` or
it did not. This is sufficient to gate governance-strong project
admission (Gate 2 prerequisite), but does not :

1. Express **degree of contribution** (LoC, commit count,
   review activity).
2. Cross-validate the contributor's identity across forges —
   a single-forge attestation is spoofable by any operator who
   controls that forge (e.g. a malicious GitHub admin can grant
   arbitrary commit access and the coordinator, trusting the
   forge mirror, will sign).
3. Compose with a trust-web of external witnesses (Amnesty
   International, EFF, CPJ, HRW) who independently vouch for
   a contributor's public identity.

Couche 3 addresses all three by :

- Extending `SBFB.json` with a declarative `contributions[]`
  array referencing other forges + DelegationCert bridging a
  SBFB node_id to an SSH signing key fingerprint used on those
  forges.
- Parsing `git log --show-signature` **offline** against a
  pre-seeded keyring, extracting per-commit signer fingerprints,
  weighting contributions by commit count × verification-status.
- Independently re-fetching the repo from ≥ 2 forge mirrors and
  comparing the commit graph to detect selective-commit attacks
  (forge operator injects an unsigned commit only on their
  mirror).
- Accepting attestations from a curated trust-web (see §6).

### Scope non-goals

- **Not** a replacement for Couche 1 gossip admission. The mesh
  always admits peers based on age + PoW, regardless of
  contribution weight.
- **Not** a quantitative "contribution fitness score". LoC and
  commit count are gameable (mechanical refactors, AI-generated
  bulk commits). Couche 3 outputs a boolean `is_verified_
  contributor_multi_forge` flag per `(project_id, node_id)`,
  not a float.
- **Not** a micropayment / kudos-weight update channel. Fairness
  reform ships post-v1.0 in LT-1 (cf.
  [`ROADMAP_COMMITMENTS §LT-1`](../release/ROADMAP_COMMITMENTS.md)).
  Couche 3 provides **inputs** to LT-1 (independent commit-graph
  evidence) but does not compute weights.

## 2. `SBFB.json` extension — `contributions[]`

The publisher manifests a `contributions[]` array declaring where
this project's git history lives across forges :

```json
{
  "node_id": "a3b1c2d3...",
  "keyoxide_proof": "...",
  "contributions": [
    {
      "forge": "codeberg",
      "url": "https://codeberg.org/alice/transLingua",
      "primary": true
    },
    {
      "forge": "github",
      "url": "https://github.com/alice-mirror/transLingua",
      "primary": false
    },
    {
      "forge": "radicle",
      "urn": "rad:z3Zbh24JG9s8iX1...",
      "primary": false
    }
  ]
}
```

### Rules

- **Exactly one** entry has `primary: true`. This is the
  coordinator's canonical source-of-truth during verified-deploy
  (S14 flow unchanged).
- Forge identifiers are enumerated :
  `{"github", "gitlab", "codeberg", "gitea", "forgejo",
    "radicle", "sourcehut"}`. Additional forges require RFC
  amendment.
- Non-primary entries are **mirrors** — they must carry the
  identical commit graph for the claimed primary-branch range.
  Divergence detected at verification time triggers a
  `multi-forge-divergence` warning (see §5).
- Absence of `contributions[]` means "this project is
  single-forge, Couche 3 is inapplicable, fall back to Couche 2
  binary attestation only". Backward-compatible for S14+
  manifests.

## 3. DelegationCert — bridging node_id ↔ SSH signer key

Problem : commits on GitHub/Codeberg/etc. are signed with SSH
keys (or GPG), not Ed25519 SBFB node_id keys. The parser needs a
tamper-proof bridge from a node_id to the SSH signer
fingerprint the contributor uses on each forge.

### Format (v1 — pre-launch stable, redefined S27 Phase C)

```rust
pub struct DelegationScope {
    pub org_name: String,       // e.g. "FlowUP"
    pub forge_urls: Vec<String>, // e.g. ["https://github.com/SBFB50/SBFB"]
}

pub struct DelegationCert {
    pub node_id: NodeId,            // Ed25519 SBFB pubkey, 32 bytes
    pub delegated_pubkey_algo: String, // "ssh-ed25519" | "ssh-rsa" |
                                        // "openpgp-ed25519" (future)
    pub delegated_pubkey_fingerprint: String, // SHA-256 of SSH pubkey
                                              // (hex lowercase), pattern
                                              // matches `ssh-keygen -lf`
    pub issued_at_ts: i64,           // UTC unix seconds
    pub expires_at_ts: Option<i64>,  // optional TTL
    pub trust_level: u8,             // 1-5, default 3. Decays -1/hop
                                     // in trust-web delegation chains
    pub scope: Option<DelegationScope>, // optional org + forge URLs
    pub node_sig: Ed25519Signature,  // over canonical JCS of above,
                                     // under domain
                                     // DOMAIN_DELEGATION_CERT_V1
}

pub const DOMAIN_DELEGATION_CERT_V1: &[u8]
    = b"nexus-delegation-cert-v1";
```

#### DelegationCert v1 field specification

| Field | Type | Required | Default | Description |
|---|---|---|---|---|
| `node_id` | `[u8; 32]` | yes | — | Ed25519 SBFB pubkey of the issuer |
| `delegated_pubkey_algo` | `String` | yes | — | `"ssh-ed25519"` / `"ssh-rsa"` / `"openpgp-ed25519"` |
| `delegated_pubkey_fingerprint` | `String` | yes | — | SHA-256 hex lowercase, 64 chars |
| `issued_at_ts` | `i64` | yes | — | UTC unix seconds |
| `expires_at_ts` | `Option<i64>` | no | `None` | Optional expiry. `None` = no expiry |
| `trust_level` | `u8` | no | `3` | Trust level delegated (1=minimal, 5=full) |
| `scope` | `Option<DelegationScope>` | no | `None` | Optional scope restriction (org + forges) |
| `node_sig` | `[u8; 64]` | yes | — | Ed25519 signature over JCS canonical of all fields except `node_sig` |

#### C2PA mapping (informational)

The DelegationCert maps to C2PA Assertion structures as follows :

| DelegationCert field | C2PA Assertion concept |
|---|---|
| `node_id` | `claim_generator` — the identity making the claim |
| `delegated_pubkey_*` + `scope` | `signer_payload` — the assertion content |
| `trust_level` | `trust_list` confidence level — how much trust the issuer delegates |
| `node_sig` | C2PA Claim Signature — integrity proof over the claim |
| `DOMAIN_DELEGATION_CERT_V1` | Claim label namespace — prevents cross-type replay |

This mapping is informational for interoperability analysis. SBFB does
not implement C2PA natively ; the canonical serialization uses RFC 8785
JCS with domain-separated Ed25519 signatures.

### Semantics

- The **contributor's node** signs the DelegationCert with their
  SBFB node_id private key. No coordinator involvement.
- The DelegationCert is published alongside `SBFB.json` in the
  deploy archive (e.g. `.sbfb/delegations/<fingerprint>.json`).
- Expiry is optional. Best-practice : annual re-issue to rotate
  SSH keys without invalidating history.
- Multiple DelegationCerts per node_id are permitted (one per
  SSH key). Revocation = publish a new DelegationCert with
  `expires_at_ts: <past>` for the revoked key.
- `trust_level` decays by 1 per delegation hop in the trust-web
  chain (minimum 1). A seed anchor has effective level 5.
- `scope` restricts the delegation to named forges. `None` = unbounded.

### Attack surface

- **Stolen SBFB node key** → attacker can mint DelegationCerts for
  arbitrary SSH fingerprints and claim historical contributions.
  Mitigated by : (1) revocation via new cert, (2) trust-web §6
  cross-references, (3) age-witness Couche 1 requiring ≥7d
  peer-attestation for the attacker's new node_id.
- **Stolen SSH key** → attacker signs malicious commits that
  appear verified. Mitigated by : forge-side key-rotation SOP,
  and by detecting commit-graph divergence at verification time
  (§5).

## 4. `git log --show-signature` offline parser

The verifier parses `git log --show-signature` output against a
local keyring pre-populated from DelegationCerts :

```
commit 1a2b3c4d...
gpg: Signature made Fri Apr 19 12:34:56 2026 UTC
gpg: using EDDSA key SHA256:abcdef1234...
gpg: Good signature from "Alice <alice@example.org>"
Author: Alice <alice@example.org>
Date: ...

    commit message
```

### Parser contract

- Input : `git log --show-signature --format="%H|%G?|%GF"` on a
  local clone of the repo. `%G?` ∈ `{"G","B","U","X","Y","R","E","N"}`.
  `%GF` = signer fingerprint.
- Output per commit : `(commit_sha, sig_status,
  signer_fingerprint, matched_node_id)`.
- `matched_node_id` is resolved by looking up
  `signer_fingerprint` in the pre-loaded DelegationCert keyring.
  `None` if no matching cert.

### Rules

- Only commits with `sig_status == "G"` and non-`None`
  `matched_node_id` count towards contribution weight.
- Offline parsing : the verifier never calls an external GPG
  server. The keyring is populated solely from DelegationCerts
  in the deploy archive.
- The parser runs inside a sandbox (Python `subprocess` under
  the coordinator's resource-limited git clone, same flow as
  S14 verified-deploy §5 validate paths).
- **No** fallback to commit author email. Email is not a trust
  anchor. Only cryptographic signatures count.

## 5. Multi-forge cross-validation

For each mirror declared in `contributions[]` (non-primary), the
verifier :

1. Clones the mirror (depth = primary mirror commit count + 10 for
   comparison headroom).
2. Compares the commit-graph `HEAD` on the primary branch
   against the primary mirror. Requires exact commit SHA
   identity.
3. On divergence :
   - If the mirror is **ahead** of primary, classify as
     `mirror-ahead` (benign, mirror is simply unsync'd).
   - If the mirror is **behind**, classify as `mirror-stale`
     (benign).
   - If the mirror has **distinct commits** (shared ancestor +
     divergent head), classify as `multi-forge-divergence` —
     this is the selective-commit attack signal. The project's
     `is_verified_contributor_multi_forge` flag becomes `false`
     until divergence is resolved.

### Rate and bandwidth budget

- Cross-validation runs **once per verified-deploy**, not on
  every curator verify. Results are cached in the coordinator's
  `contributor_registry` SQLite table with TTL 90 days.
- Bandwidth budget : same 500 MB cap as primary clone (S14 §5).
- Timeouts : 30 s per mirror. A timed-out mirror is classified
  `mirror-unreachable` (non-blocking, but flagged in the
  registry).

### Radicle / Tangled fork evaluation (deferred)

Radicle Heartwood 1.8.0 uses `did:key` SSH-format signed patches
rather than forge-mirrored git. Tangled (nostr-git fork, 2026)
uses secp256k1 which is incompatible with SBFB's Ed25519 node_id
keypair. Integration with these P2P forges is out-of-scope RFC
S22, reserved S26+ (requires a `did:key` ↔ Ed25519 bridge that
does not currently exist in the SBFB crypto stack).

## 6. Trust-web Amnesty integration (S27+)

External human-rights organisations act as independent witnesses.
The design :

- A fixed pre-launch keyring ships with the SBFB daemon
  (`crates/nexus-shell-daemon-core/src/trust_web_keys.rs`), listing
  Ed25519 public keys for participating organisations (Amnesty,
  EFF, CPJ, HRW — subject to formal partnership per
  [`PARTNERSHIPS.md`](PARTNERSHIPS.md) S30 line).
- Each org publishes a counter-signature on a `(project_id,
  contributor_node_id)` tuple stating "we confirm this contributor
  is a real person we've communicated with and verify against
  known aliases". The signature lives in a new domain
  `DOMAIN_EXTERNAL_WITNESS_V1` (reserved, not added S22).
- Projects may surface `external_witness_count` in their UI. A
  project with ≥ 1 external witness from the pre-shipped keyring
  gets a "third-party attested" badge (LibanLive / PolitiScan
  ship-blocker per `RELEASE_GATES.md` Gate 4).

Privacy : organisations counter-signing implies they have
relationship data with the contributor. Opt-in only, published
with explicit contributor consent. Revocation :  an org can
publish a superseding signature annulling a past witness.

## 7. Triggers for re-activation S23-S27

| Trigger | Target sprint | Scope |
|---|---|---|
| SBFB.json schema v5 `contributions[]` + DelegationCert primitive live Rust/Python | S23 | Schema evolution + RFC reference impl without verification wiring |
| `git log --show-signature` parser + offline keyring lookup | S24 | Single-forge parser behind feature flag, no commit-graph diff yet |
| Multi-forge cross-validate (GH + Codeberg + Forgejo first) | S26 | Full flow wired into `api/deploy.py` with divergence flagging + `multi-forge-divergence` state in registry |
| Radicle / Tangled P2P forge integration (if did:key bridge materialises) | S26+ | Deferred, requires upstream crypto decisions |
| External witness keyring + `DOMAIN_EXTERNAL_WITNESS_V1` | S27 | Amnesty / EFF / CPJ / HRW partnerships formalised per `PARTNERSHIPS.md` S30 |
| Kudos-v2 LT-1 consuming Couche 3 outputs as fairness inputs | post-v1.0 | Not a Couche 3 trigger — LT-1 is a separate reform on kudos distribution, couche 3 merely provides independent evidence |

### Meta-track S27 pivot

The `HARDENING_ROADMAP §3 S27` line originally read
"Sybil kudos-weighted mature". After the D1 arbitrage 2026-04-19,
that item is re-scoped to "Couche 3 mature (multi-forge
cross-validate + trust-web Amnesty)". Same FAIRNESS flag implicit
— fairness lands post-v1.0 via LT-1, not via kudos weighting at
admission time.

## 8. Open questions

- **SSH key rotation UX** : a contributor rotating their SSH key
  must re-issue DelegationCerts for each active project. UI
  support TBD S24.
- **Revocation propagation** : revocation by republishing a new
  DelegationCert with past `expires_at_ts` depends on every
  verifier re-fetching the `.sbfb/delegations/` directory. A
  gossip channel for revocations is reserved S26 design.
- **SHA-256 git object IDs** : when git-ecosystem consensus on
  SHA-256 lands, `commit_sha` in `CONTRIBUTOR_ATTESTATION_PREDICATE.md`
  Couche 2 bumps to `/v2` URI with
  `commit_object_id + commit_object_id_algo`. Couche 3 parser
  must support both during transition.

## 9. References

- In-toto v1.0 attestation spec :
  https://github.com/in-toto/attestation/blob/main/spec/v1/README.md
- SSH signed commits (OpenSSH 8.0+) :
  https://man.openbsd.org/ssh-keygen.1#Y~5
- Radicle Heartwood 1.8.0 "Drosera" :
  https://radicle.xyz/2026/03/30/radicle-1.8.0
- SLSA Provenance v1 :
  https://slsa.dev/spec/v1.1/provenance
- SBFB sibling docs :
  - [`CONTRIBUTOR_ATTESTATION_PREDICATE.md`](
    CONTRIBUTOR_ATTESTATION_PREDICATE.md) — Couche 2 live S22
  - [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) §3 S22-S27
  - [`ADVERSARIES.md`](ADVERSARIES.md) tiers T0-T5
  - [`PARTNERSHIPS.md`](PARTNERSHIPS.md) Amnesty/EFF/CPJ/HRW
    S30 partnership triggers
