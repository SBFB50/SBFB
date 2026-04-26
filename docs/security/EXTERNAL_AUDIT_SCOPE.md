# External Audit Scope — SBFB pre-Gate 3

**Date** : 2026-04-26
**Statut** : scope document, pre-engagement
**Auteur** : FlowUP (solo maintainer)
**Sprint source** : S28 Phase D

---

## 1. Objectif

Audit de securite externe independant avant deblocage Gate 3
(Alexandria showcase app, cf.
[`RELEASE_GATES.md`](RELEASE_GATES.md) et
[`HARDENING_ROADMAP.md §7`](HARDENING_ROADMAP.md)).

SBFB est un reseau P2P decentralise de compute et d'hebergement
d'apps. La surface d'attaque principale est crypto + protocol +
wire format — pas UI web classique. L'audit cible les primitives
de securite qui protegent les utilisateurs a risque (journalistes,
ONG, activistes — cf. [`ADVERSARIES.md`](ADVERSARIES.md) tiers
T0-T5).

---

## 2. Scope in

### 2.1 Crypto primitives (7 primitives)

| Primitive | Usage | Crate/module | Fichiers cles |
|---|---|---|---|
| Ed25519 (RFC 8032) | Keypair identity, canary signing, DelegationCert, ProvenanceRecord, AgeWitness, ContributorAttestation | `ed25519-dalek 2.1` | `crates/nexus-core-rs/src/crypto.rs`, `canonical.rs` |
| AES-256-GCM | Keystore encryption at rest | `aes-gcm 0.10` | `crates/nexus-core-rs/src/keystore.rs` |
| Argon2id | KEK derivation from passphrase | `argon2 0.5` | `crates/nexus-core-rs/src/keystore.rs` |
| FROST Ed25519 (RFC 9591) | Warrant canary threshold signing K-of-N | `frost-ed25519 2.1` (ZF, ToB 2023 audit) | `crates/nexus-core-rs/src/canary.rs` |
| HMAC-SHA256 | Watermark PRF (SynthID-inspired green-list bias) | `hmac 0.12` + `sha2 0.10` | `crates/nexus-worker-core/src/llm/watermark.rs` |
| BLAKE3 | Content-addressed hashing (blob integrity, re-run sampling) | `blake3 1.5` | `crates/nexus-worker-core/src/engine/runtime.rs` |
| Hashcash PoW | Anti-Sybil gossip admission cost | custom impl | `crates/nexus-core-rs/src/pow.rs` |

**Focus audit** : correctness of domain separation (`DOMAIN_*_V1`
constants), key management lifecycle (generation → storage →
rotation → revocation), side-channel resistance of Ed25519
operations, Argon2id parameter hardness (64 MiB memory cost).

### 2.2 Wire formats (6 canonical structures)

Tous les wire formats utilisent JCS (RFC 8785) canonical JSON pour
les signatures Ed25519. Structures :

| Structure | Domain constant | Fichier |
|---|---|---|
| `Task` | `DOMAIN_TASK_V1` | `canonical.rs` |
| `CuratorList` | `DOMAIN_CURATOR_LIST_V1` | `canonical.rs` |
| `CanarySigned` | `DOMAIN_WARRANT_CANARY_V1` | `canonical.rs` |
| `AgeWitness` | `DOMAIN_AGE_WITNESS_V1` | `canonical.rs` |
| `ContributorAttestation` | `DOMAIN_CONTRIBUTOR_ATTESTATION_V1` | `canonical.rs` |
| `DelegationCert` | `DOMAIN_DELEGATION_CERT_V1` | `canonical.rs` |

**Focus audit** : domain separation correctness (no cross-domain
signature confusion), JCS canonicalization compliance RFC 8785,
`version = 1` enforced (pre-launch protocol policy — no multi-
version decoder), serde deserialization robustness (fuzzing).

### 2.3 Auth loopback

| Mechanism | Sprint | Fichier |
|---|---|---|
| Bearer token (`X-SBFB-Token`) | S16 | `crates/nexus-shell-daemon-core/src/auth.rs` |
| UDS peer creds (Linux/macOS) | S16 | `crates/nexus-shell-daemon-core/src/named_pipe_server.rs` |
| Named Pipe SDDL DACL (Windows) | S16 | `crates/nexus-shell-daemon-core/src/named_pipe_server.rs` |
| Host allowlist (`localhost` / `127.0.0.1` / `[::1]`) | S16 | `crates/nexus-shell-daemon/src/main.rs` |
| Origin allowlist | S16 | `crates/nexus-shell-daemon/src/main.rs` |

**Focus audit** : token entropy + storage, TOCTOU on peer cred
check, SDDL DACL correctness (Windows), Host/Origin bypass via
DNS rebinding (S16-A mitigated, verify completeness).

### 2.4 Transport

| Component | Version | Fichier |
|---|---|---|
| iroh gossip | 0.97 | `crates/nexus-core-rs/src/gossip.rs` |
| iroh blobs | 0.99 | `crates/nexus-core-rs/src/blobs.rs` |
| iroh DHT pkarr | 0.97 | `crates/nexus-shell-daemon-core/src/browse.rs` |
| TLS SPKI pinning (relay) | S19 | `crates/nexus-core-rs/src/transport.rs` |
| PoW Hashcash gossip admission | S19 | `crates/nexus-core-rs/src/pow.rs` |
| DNS fallback DoH+DoT | S24 | `crates/nexus-core-rs/src/dns_fallback.rs` |

**Focus audit** : iroh relay trust model (single-relay dependency
pre-federation), gossip message authentication (Ed25519 required),
PoW difficulty tuning (anti-DoS vs usability), DHT poisoning
resistance (quorum 2/3 lookup).

**Note** : iroh 0.97 lui-meme n'a pas d'audit public connu
(zone rouge R-iroh-audit P0). L'audit SBFB couvre le **usage** de
iroh, pas iroh interne. Un audit iroh dedié est un item futur
séparé.

### 2.5 Sandbox

| Component | Fichier |
|---|---|
| blob-serve iframe CSP (`sandbox="allow-scripts"`, no `allow-same-origin`) | `crates/nexus-shell-daemon-core/src/blob_serve.rs` |
| postMessage bridge 3 methods (`task_submit`, `storage_get`, `storage_set`) | `web/public/sbfb-bridge.js` |
| Correlation ID validation + source iframe check | `web/src/hooks/useBridgeMessages.ts` |

**Focus audit** : CSP bypass vectors (cf. A-S1 ATTACK_SCENARIOS),
postMessage origin validation completeness, bridge method
parameter injection.

### 2.6 Process isolation (si livre S29)

| Component | Design doc |
|---|---|
| Broker/executor split (IPC JSON-RPC 2.0, UDS/Named Pipe) | [`PROCESS_ARCHITECTURE.md`](PROCESS_ARCHITECTURE.md) |

**Conditionnel** : si le split broker/executor est implemente
avant l'engagement audit (S29 Phase D2), inclure dans le scope.
Sinon, differer a un audit follow-up.

### 2.7 Version verification at RFP time

Before sending the RFP to the audit vendor, verify that all
dependency versions listed in §2.1-§2.6 match the actual
`Cargo.lock` / `pyproject.toml` at the scope freeze commit.
Concrete checklist:

| Check | Command |
|---|---|
| Ed25519 dalek version | `cargo tree -p ed25519-dalek --depth 0` |
| AES-GCM version | `cargo tree -p aes-gcm --depth 0` |
| FROST version | `cargo tree -p frost-ed25519 --depth 0` |
| iroh pinned | `cargo tree -p iroh --depth 0` |
| iroh-blobs pinned | `cargo tree -p iroh-blobs --depth 0` |
| opentelemetry (if wired) | `cargo tree -p opentelemetry --depth 0` |
| Scope freeze commit | `git rev-parse --short HEAD` at freeze time |

The scope freeze commit must be recorded in §7 Timeline and
communicated to the vendor as the exact revision to audit. Any
code merged after the freeze commit is out of scope for the
initial engagement.

---

## 3. Scope out

| Element | Raison |
|---|---|
| UI React (`web/`) | Pas de surface de securite directe (iframe host, pas d'auth, pas de donnees sensibles en frontend) |
| Docs / planning (`.planning/`, `docs/`) | Non-code |
| CI/CD (GitHub Actions) | Hors perimetre runtime (supply chain CI = track separate) |
| Infrastructure de test | Non-runtime |
| Python coordinator (`packages/nexus-coordinator/`) | Surface secondaire — HTTP loopback local-only, pas expose au reseau. Couvert partiellement via §2.3 auth. |
| Apps SDK (`packages/nexus-sdk/`, `packages/nexus-app-*/`) | Applications utilisateur, pas primitives securite |

---

## 4. Vendor matrix

| Critere | Cure53 | Trail of Bits |
|---|---|---|
| Specialite | Web + infrastructure + API | Crypto + protocol + formal verification |
| Engagement type | Penetration testing + code review | Code review + formal analysis + custom tooling |
| Track record P2P/crypto | Mullvad VPN, Briar, Obscura | FROST (ZF 2023), Zcash, Cosmos IBC, age |
| Cout typique engagement | $20-50k (2-4 semaines) | $50-100k (4-8 semaines) |
| Deliverables | Report + remediation verification | Report + formal properties + custom Semgrep rules |
| Fit SBFB surface | Auth loopback + transport + sandbox | **Crypto primitives + wire formats + protocol** |

**Note** : les couts indiques sont des fourchettes publiquement
connues pour des engagements de taille comparable. Ils ne
presupposent aucun budget securise.

---

## 5. Recommandation

**Trail of Bits** — le ratio surface SBFB (7 crypto primitives +
6 wire formats + watermark PRF scheme novel) est align avec
l'expertise crypto/protocol de Trail of Bits plutot que le profil
web/infra de Cure53.

Arguments :
- 7 primitives crypto dont FROST (ToB a audite le crate FROST ZF
  en 2023 — connaissance existante du code et du standard RFC 9591)
- JCS canonical serialization + domain separation = surface
  protocol, pas surface web
- Watermark PRF scheme (SynthID-inspired, non-standard) = besoin
  d'analyse formelle de la soundness du z-test binomial
- Wire format fuzzing (serde deserialization attack surface)

**Duree estimee** : 4-6 semaines (scope §2 = ~15 fichiers Rust
critiques + 6 wire formats + 7 primitives). Un engagement de
8 semaines couvrirait un fuzzing exhaustif des wire formats en
complement de la revue manuelle.

---

## 6. Pre-conditions S29

Avant engagement audit, les items suivants doivent etre livres :

| Item | Sprint | Statut |
|---|---|---|
| PROCESS_ARCHITECTURE.md design doc | S28 Phase C | DONE (`ccbb6ca`) |
| EXTERNAL_AUDIT_SCOPE.md (ce document) | S28 Phase D | DONE |
| THREAT_MODEL.md §9 per-mode residual risk | S29 Phase B4 | TODO |
| Wire formats stables (pre-launch protocol v1) | Ongoing | OK (VERSION=1, no bump) |
| Platform writers operationnels (audit trail) | S28 Phase B | DONE (`a43a1a1`) |
| Watermark end-to-end wiring | S28 Phase A | DONE (`c5f35f7`) |

---

## 7. Timeline

```
S28 Phase D (ce document)     → scope finalise
S29 Phase A                   → RFP envoi + engagement vendor
S29 Phase B                   → THREAT_MODEL §9 per-mode doc
S29 Phase C                   → audit execution (4-6 semaines)
S29 Phase D                   → findings reception + remediation
S29 Phase E                   → remediation verification + public report
```

**Ship-blocker Gate 3** : le rapport d'audit public avec
remediation verifiee est un prerequis au deblocage Gate 3
(cf. [`HARDENING_ROADMAP.md §7`](HARDENING_ROADMAP.md)).

---

## References

- [`THREAT_MODEL.md`](THREAT_MODEL.md) — STRIDE/LINDDUN baseline
- [`ADVERSARIES.md`](ADVERSARIES.md) — tiers T0-T5
- [`ATTACK_SCENARIOS.md`](ATTACK_SCENARIOS.md) — 12 scenarios
- [`PROCESS_ARCHITECTURE.md`](PROCESS_ARCHITECTURE.md) — broker/executor design
- [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) — sprint sequencing
- [`RELEASE_GATES.md`](RELEASE_GATES.md) — gate enforcement policy
