# Validated Long-Term Security Blueprint

**Document de vision long-terme** — pas un plan d'exécution tactique
(voir [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) pour la
sequence sprint-by-sprint). Ce document capture l'architecture
cible **maximaliste** composée de briques OSS matures, chaque
dimension validée contre l'état de l'ecosysteme 2026 via docs
officielles + advisories + benchmarks externes.

Methodologie validation :

- Cross-check crates.io + docs.rs + context7 MCP pour API state
  2026
- WebSearch sur release notes + security advisories + CVE
  2024-2026
- Benchmark per-dimension vs Signal / Tor / Briar / SecureDrop /
  Holochain / IPFS / Wasmtime-based platforms (Shopify, Fastly,
  Figma)
- Symbolic Software (7 avril 2026) findings sur `hax`/`libcrux`
  semantic gaps integres

Principes guidant le design :

1. **Defense-in-depth partout** — aucun composant ne suppose
   les autres sains
2. **Zero-trust inter-composants** — daemon, worker, blob-serve,
   coordinator = 4 processus isoles
3. **Composition over reinvention** — embed OSS audite, jamais
   fork modificateur
4. **Formal verification sur critical path** — wire format +
   primitives crypto + protocole identity
5. **Metadata minimization by design** — padding constant-rate,
   chaque octet reseau justifie
6. **Post-quantum by default** — ML-KEM-1024 + ML-DSA obligatoires
   baseline, pas hybride optionnel
7. **Reproducible builds deterministes** — tout binaire
   verifiable by anyone from source
8. **Multi-party governance** — aucune decision critique
   solo-maintainer

---

## Couche 0 — Host hardening OS-level

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `seccompiler` (rust-vmm/AWS) | 0.5.0 | Apache-2.0 | GO | Prod Firecracker, aucune CVE |
| `rust-landlock` | ~0.4+ (ABI 6) | MIT/Apache-2.0 | GO | Linux LSM, usage Ubuntu |
| `tss-esapi` | 7.6.0 | Apache-2.0 | CAUTION | `-sys` 0.6.0-alpha, attendre 1.0 |
| `windows-rs` | Regulier Microsoft | MIT/Apache-2.0 | GO | Job Object + AppContainer bindings |
| `security-framework` (macOS) | 2.x | MIT/Apache-2.0 | GO | Keychain + Secure Enclave |

**Process isolation** : 4 daemons distincts avec IPC via unix
domain sockets (Unix) + SO_PEERCRED ou Named Pipes DACL
(Windows), capability-restricted. Cap'n Proto ou bincode sur
socket seccomp-limited.

**Memory safety** : Rust partout, zero C unsafe custom. Seuls
deps C autorises = crates auditees (`ring`, `aws-lc-rs`).

**Retiree du blueprint initial** : `capsicum-rs` (FreeBSD only,
hors scope SBFB Win11+Linux).

---

## Couche 1 — Identity hardware-backed

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `aws-lc-rs` | Actif (AWS) | Apache-2.0/ISC | GO | **FIPS 140-3 ML-KEM valide**, premier OSS |
| `ed25519-dalek` | 2.1.x | MIT/Apache-2.0 | GO | RUSTSEC-2022-0093 fixed v2.0 |
| `frost-ed25519` (Zcash Foundation) | 2.0.0 | MIT/Apache-2.0 | GO | Threshold M-of-N prod Zcash |
| `zeroize` | 1.8.x | MIT/Apache-2.0 | GO | Memoire clees zeroisee, standard |
| `secrecy` | 0.10.x | MIT/Apache-2.0 | GO | Wrapper typed secrets sur zeroize |
| `ring` | 0.17.x | Apache-2.0/ISC | GO | Foundation crypto ecosystem Rust |
| `halo2` (Zcash/PSE) | 0.3.x | MIT | CAUTION | Prod Scroll/Taiko, soundness bug 2024 patched |
| `didkit-rs` / `ssi-rs` (Spruce) | 0.15.0 | Apache-2.0 | CAUTION | W3C VC 2.0 (W3C Rec mai 2025), HTTP non-prod |

**Hardware backing obligatoire** :

- macOS : Secure Enclave via `security-framework` +
  `Security.framework` API
- Windows : TPM 2.0 via `windows-rs` + DPAPI-NG PCR-bound
- Linux : `systemd-creds` + TPM2 via `tss-esapi` (quand -sys 1.0)
- Fallback degrade : `keyring-rs` pour systemes sans TPM
- Optional : YubiKey FIDO2 via `ctaphid-rs`

**PQC signing hybride** : Ed25519 + ML-DSA-65 via `aws-lc-rs`.
**Correction post-validation** : `libcrux` ML-DSA/ML-KEM NON
retenu pour prod cause 5 semantic gaps demontres par Symbolic
Software (7 avril 2026) dans pipeline hax→F*. libcrux OK pour
primitives secondaires (codec, hash) pas crypto critique.

**Threshold signing** : FROST-Ed25519 2.0 pour actions critiques
(publish app, curator list update, revocation list) — M-of-N
signature quorum.

**Retire du blueprint initial** :

- FROST-ML-DSA — papiers TALUS/Quorus 2025-2026 academiques,
  zero implementation Rust publiee. Attendre ou contribuer
  upstream.
- `zkgroup` crate independent — archive, absorbe dans libsignal
  qui decourage usage externe.
- Claim "formally verified" sur libcrux — downgrade obligatoire
  post-Symbolic Software findings. OK pour petites functions,
  NOT OK pour claim prod-grade.

---

## Couche 2 — Transport

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `quinn` | 0.11.9 | MIT/Apache-2.0 | GO | QUIC Rust, prod iroh |
| `s2n-quic` (AWS) | 1.68.x | Apache-2.0 | GO | Kani-verified, prod AWS |
| `rustls` + `prefer-post-quantum` | 0.23.22+ | MIT/Apache-2.0 | GO | ML-KEM feature flag natif, Prossimo financed |
| `arti` (Tor Rust) | 2.2 (avril 2026) | MIT/Apache-2.0 | GO | Library embed, HTTP CONNECT default |
| `maybenot` (Mullvad) | 1.x | GPLv3 (compat AGPL-3.0) | GO | Traffic shaping state machines, prod Mullvad DAITA |
| `nym-sdk` | 1.27.0 | Apache-2.0 | CAUTION | Beta, latences 200-800ms, async uses only |
| `gotatun` (Mullvad) | Actif | Apache-2.0 | GO | WireGuard Rust + DAITA, audite Assured fev 2026 |

**Design** :

- **QUIC core** : `quinn` 0.11.9 ou `s2n-quic` 1.68.x (Kani
  formally verified par AWS)
- **PQC KEX obligatoire** : X25519 + ML-KEM-1024 hybride
  (ML-KEM-1024 plutot que 768 pour security margin max)
- **Onion routing** : `arti-client` 2.2 embed, default-on pour
  apps Gate 3+
- **Mixnet** : `nym-sdk` 1.27 SOCKS wrapper pour apps Gate 4,
  reserve usages asynchrones (latence exclue streaming)
- **Pluggable transports** : delegues a `arti` qui gere
  `lyrebird` subprocess Go (obfs4 + meek + Snowflake + webtunnel).
  Pas d'integration directe Rust.
- **Traffic shaping** : `maybenot` machines etat constant-rate
  padding
- **Dual-transport fallback** : WebSocket TCP/443 domain-fronted
  (Cloudflare, Fastly)

**Retire du blueprint initial** :

- `hickory-dns` — README dit "not recommended for production".
  Remplace par `reqwest` + Cloudflare DoH endpoint direct pour
  les rares cas DNS necessaires.
- `lyrebird` Rust embed direct — n'existe pas, seulement Go.
  Delegue a `arti`.

---

## Couche 3 — Overlay P2P

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `pkarr` (Nuhvi) | Actif | MIT | GO | Mainline BT DHT + pubkey DNS |
| `iroh` (n0-computer) | 0.97 (1.0 pas encore) | Apache-2.0/MIT | GO | SBFB deja pinne 0.97 |
| `libp2p-gossipsub` | **0.49.4+** | MIT/Apache-2.0 | CAUTION | CVE-2026-33040 + CVE-2026-34219 patched 0.49.4 |

**Directory authorities pattern** : 10 organisations
independantes jurisdictionnellement diverses (Amnesty, EFF, CCC,
Riseup, Calyx, FPF, Tor Project, NLnet Labs, FDN, La Quadrature
du Net) publient consensus horaire signe FROST-threshold. Adapte
du Tor directory authorities model.

**Guard nodes** : algorithme `tor-guardmgr` pattern applique a
iroh peer selection — 3 guards persistants weighted par
`kudos × uptime × AS-diversity`.

**Eclipse detection** : honeypot peer rotation + canary heartbeat
+ AS-diversity enforced.

**Retire du blueprint initial** :

- HyParView / Plumtree Rust — aucune impl mature publiee.
  Rester sur gossipsub patched 0.49.4+.

---

## Couche 4 — Sybil resistance multi-layer

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| PoW Hashcash gated | Implementation custom | N/A | GO | Difficulte auto-ajustee 2^20+ |
| W3C Verifiable Credentials 2.0 | W3C Recommendation 15 mai 2025 | W3C libre | GO | VC 2.0 + ZK presentation |
| BrightID | Active 2M+ users | MIT | CAUTION | Centralise ArangoDB nodes |
| Proof-of-Humanity (Kleros) | Active Ethereum | MIT-like | CAUTION | PoH v2 controverse |

**Proof-of-unique-human stack** (user choisit) :

- **Option A** : ONG-issued Verifiable Credentials via `ssi-rs` +
  ZK presentation (e.g. Amnesty issues credential, user proves
  possession sans identity disclosure)
- **Option B** : BrightID social graph (compromis : requires
  ArangoDB node tiers)
- **Option C** : Kleros Proof-of-Humanity video attestation
  (on-chain)
- **Option D (baseline)** : PoW Hashcash + kudos-weighted
  admission seul, sans biometric

**Kudos-weighted admission** : threshold ajuste par curator list.
Nodes >N kudos = full voice, others = read-only ou queue priorite
basse.

**Rate-limit per-identity** : sliding window + escalating PoW
per-model + exponential cooldown (1/2/4/... min) pour
depassement.

---

## Couche 5 — Storage at-rest

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `rage` (str4d) | Actif | MIT/Apache-2.0 | GO | Age Rust, prod Nixpkgs |
| `age-plugin-sntrup761x25519` | Deploye OpenSSH | MIT | GO | PQC at-rest mature |
| `automerge` (Rust backend) | 0.5.x | MIT/Apache-2.0 | GO | CRDT, rewrite Rust |
| `yrs` (Y.js Rust) | 0.21.x | MIT | GO | Prod Liveblocks |
| `blake3` | 1.8.3 | CC0 + Apache-2.0 | GO | Content-addressed hash, 80M+ downloads |

**Encryption by default** : `rage` avec recipient
`age-plugin-sntrup761x25519` (PQC at-rest deja deploye dans
OpenSSH) ou `age-plugin-mlkem768x25519` (emergent) pour blobs
prives.

**Duress + panic wipe** :

- **Duress keypair** : secondary identity unlock fake empty state
- **Panic wipe** : 5-tap gesture (Ctrl+Shift+Alt+W) wipe keypair
  + state sqlite + blob cache immediat sans confirmation
- **Plausible deniability** : nested encryption layers pattern
  VeraCrypt hidden-volume

**Replicated state** : CRDT via `automerge` ou `yrs` pour sync
offline-first.

**Content-addressed** : BLAKE3 deterministic, reproducible blob
hash.

**Retire du blueprint initial** :

- `cryptsetup-rs` / LUKS2 Rust bindings — pas de crate mature,
  LGPL-2.1 complexifie. `rage` at-rest suffit pour scope SBFB.

---

## Couche 6 — Compute TEE-attested

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `nvml-wrapper` | 0.11.0 | MIT/Apache-2.0 | GO | 2.9M downloads, prod |
| `sev` (AMD SEV-SNP) | 7.1.0 | Apache-2.0 | GO | VirTEE/Firecracker, attestation complete |
| `tdx-guest` (Intel TDX) | Actif mars 2026 | Apache-2.0 | GO | Intel support direct |
| NVIDIA H100 CC mode + NRAS | Public 2026 (cloud only) | Proprietaire NVIDIA | CAUTION | **RTX 5080 ne supporte PAS**, cloud deployment only |
| `llama.cpp` grammar/GBNF | Actif 2026 | MIT | GO | Structured output enforcement |

**TEE attestation obligatoire Gate 3+** :

- NVIDIA H100 Confidential Computing via `nvml-wrapper` + NVIDIA
  Attestation Service (NRAS)
- AMD SEV-SNP pour CPU workloads via `sev` 7.1
- Intel TDX alternative via `tdx-guest`
- Fallback Gate 1-2 : worker `no-attestation` marking, refuse
  Gate 3+ tasks automatiquement

**VRAM wipe verified** : `cudaMemset` post-task + driver-level
verification via NVML.

**Ephemeral workers** : process restart after N tasks, cgroup +
namespace isolation (Linux), Job Object (Windows), MIG
partitioning H100/A100 opt-in.

**Redundancy voting** : k-of-n (3+ workers) majority via
`Task.redundancy_factor`, spot-check watermark canari aleatoire.

**Watermark injection opt-in** : Kirchenbauer 2023 green-list
tokens biased.

**Prompt redaction client-side** : regex PII + spaCy NER wasm
before submission.

**Output filtering** : `llama.cpp` grammar JSON schema
enforcement + beacon chars scan + prompt injection detection.

**Rate-limit multi-axis** : per-(consumer, worker, model)
sliding window + escalating PoW per-model.

**No-GPU-sharing policy** : worker-core detecte autre process
significatif sur GPU, refuse task.

**Note deployment** : TEE GPU = cloud deployment uniquement
(H100/A100 SXM/PCIe data-center). Machine dev locale (RTX 5080)
ne supporte pas CC mode. Gate de deploiement, pas feature dev
machine.

---

## Couche 7 — App runtime WASM capability-based

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `wasmtime` | **43.0.1+** ou LTS **36.0.7+** | Apache-2.0 | CAUTION | **12 CVE avril 2026** dont 2 Critical, pinning strict |
| WASI Preview 2 | Stable finalise 2024 | N/A | GO | Capability-based I/O |
| `cap-std` | 4.0.0 | Apache-2.0 | GO | Prod wasmtime foundation |
| `capnp` (Cap'n Proto) | 0.20.x | MIT | GO | IPC cross-process |

**Wasmtime 43.0.1+ obligatoire** — version minimum post-batch
CVE avril 2026 :

- **CVE-2026-34971** (CVSS 9.0) : sandbox escape aarch64
  Cranelift, arbitrary read/write
- **CVE-2026-34945** (CVSS 9.0) : host data leakage via tables
  64-bit + Winch
- 10 autres CVE patches 43.0.1 / 42.0.2 / 36.0.7 / 24.0.7

Cranelift x86_64 backend en prod. **Desactiver Winch**
(experimental).

**Isolation cross-process** : apps Pyodide/WASM tournent dans
sub-process Wasmtime isole, pas iframe seul. Evite sandbox
escape class CVE-2025-68668 (n8n) / Grist CellBreak.

**Capabilities systeme** : seccomp-BPF + Landlock (Linux),
AppContainer + Job Object (Windows), sandbox-exec (macOS).

**Runtime attestation** : app code BLAKE3 hash verified + signed
provenance matching before spawn.

**No dynamic code** : WASM statically compiled, pas
d'interpreteur embed dans-process.

**CSP + iframe** : `sandbox="allow-scripts"` uniquement, CSP
`connect-src 'none'` + `script-src 'self'` hashes only.

**PostMessage bridge** : whitelist method + typed schema +
correlation IDs + source origin verified + rate-limited.

**Per-app storage** : namespaced, encrypted avec per-app key
derived HKDF depuis node keypair.

---

## Couche 8 — Deploy reproducible verified

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| `cargo-vet` (Mozilla) | 0.9.x | MIT/Apache-2.0 | GO | Prod Mozilla, Google |
| `osv-scanner` v2 (Google) | V2 mars 2025 | Apache-2.0 | GO | CVE scanning continu |
| `cosign` v3 | v3.x (Go) | Apache-2.0 | GO | CNCF, stable. Utilise CLI subprocess |
| Nix flakes | Stable 2026 | MIT/LGPL-2.1 | GO | 100K+ packages, hermetic |
| Radicle | v1.6.0 jan 2026 | MIT/Apache-2.0 | GO | P2P Git forge, FOSDEM 2026 |
| `sigstore-rs` | <1.0 experimental | Apache-2.0 | CAUTION | API instable, attendre 1.0 |
| `in-toto-rs` | <1.0 unstable | Apache-2.0 | CAUTION | Breaking sur patch releases |

**Source-only publish** : coordinator clone repo, build zip —
jamais upload zip user (pattern Sprint 14 SBFB, unique OSS).

**Reproducible builds hermetic** : Nix flakes
(`nix-build --option sandbox true`) ou Bazel, SOURCE_DATE_EPOCH +
lockfile pinning.

**SLSA Level 3+** : builds in hardened environment + signed
provenance. Utilise `cosign` CLI v3 Go en subprocess (stable
CNCF) plutot que `sigstore-rs` Rust (experimental jusqu'a 1.0).

**Multi-forge redundancy** : publish to GitHub + GitLab +
Codeberg + Radicle simultane.

**Multi-builder attestation** : N independent reproducers
re-build from source, compare BLAKE3, publish attestation signee.
Seuil quorum required pour publish finale.

**Keyoxide proof-of-ownership multi-channel** : Git commit GPG +
Fediverse verification + DNS TXT + Keyoxide standard.

**Supply chain hardening** :

- `cargo-audit` + `pip-audit` + `npm audit` en CI (bloque PR sur
  CVE critical)
- `cargo-vet` reutilise audits Mozilla/Bytecode Alliance (couvre
  solo-maintainer gap partiellement)
- `osv-scanner` v2 continu sur Python + Rust + npm

---

## Couche 9 — Trust layer

| Brique | Version 2026 | License | Verdict |
|---|---|---|---|
| Custom curator lists | `frost-ed25519` 2.0 signing | Apache-2.0 | GO |
| CRDT kudos ledger | `automerge` / `yrs` | MIT | GO |
| Certificate Transparency patterns | Via HTTP API Rust standard | N/A | CAUTION |

**Curator lists FROST-signed** : M-of-N signature quorum, gossip
distributed, daily rotation.

**Kudos ledger** : per-project content-addressed CRDT,
non-transferable, audit log. Zero cost / deposit / stake / burn /
refund / achat (non-monnaie, figee Day 0).

**Multiple curator orgs** : user choisit curators de confiance
— Amnesty curator = high-trust, random unknown = low-trust.

**Revocation list** : gossip transparency log inspired
Certificate Transparency, signed, auditable.

**Warrant canary per-org** : signed monthly heartbeat +
event-triggered, aggregation dashboard.

---

## Couche 10 — Operational security

| Item | Validation 2026 |
|---|---|
| OpenSSF Alpha-Omega | $12.5M annonce mars 2026 (Anthropic, AWS, GitHub, Google, MS, OpenAI) |
| OTF Red Team Lab | $43.5M OTF total FY2025, audit Cure53/Include Security gratuit |
| NLnet NGI0 Commons | Budget 21.6M EUR jusqu'en 2027, calls tous les 2 mois |
| Sovereign Tech Agency (DE) | 17M EUR budget 2025 |
| ISRG Prossimo | Craig Newmark 100K$ 2026, AWS $1M |
| HackerOne Community Edition | Gratuit OSS, triage-as-a-service disponible |

**Fondation legale** : association loi 1901 (France) ou Stichting
(Pays-Bas) ou 501(c)(3) (US) multi-jurisdiction. Board minimum
3 orgs independantes jurisdictionnellement diverses — aucun
single government peut legalement compromettre.

**Release signing M-of-N** : minimum 3 maintainers signatures
required pour release. FROST-Ed25519 2.0.

**CI/CD reproducible transparent** : Nix + cachix.org public
cache, attestations par N independent builders.

**Continuous audit** : OTF Red Team Lab annual + community
bounty HackerOne Community Edition + OpenSSF Alpha-Omega
subscription.

**Responsible disclosure** : security.txt + PGP key + 90 days
embargo + CVE assignment workflow GitHub Security Advisories.

**Transparency reports quarterly** : legal requests received,
user count, network health.

**Vetting contributeurs** (leçon XZ Utils CVE-2024-3094) :

- GPG signing obligatoire tout commit merged
- 30-day delay minimum avant merge rights nouveau contributeur
- Code review tier-confiance pour PRs touchant crypto ou reseau
- 4-eyes minimum sur modifications canonical bytes / wire format

---

## Couche 11 — Formal verification

| Brique | Version 2026 | License | Verdict | Notes |
|---|---|---|---|---|
| ProVerif (INRIA) | 2.05 | Apache-2.0 | GO | Protocol-level, Signal Triple Ratchet model |
| Tamarin Prover | 1.8.x | GPLv3 (compat) | GO | WireGuard, PQ3, TLS 1.3 verified |
| `Kani` (AWS) | 0.66.0 | Apache-2.0 | GO | Model checking Rust, Firecracker/s2n-quic |
| `Creusot` | 0.9.0 (POPL 2026) | LGPL-2.1 (outil) | GO | Rust verification pre/post conditions |
| `proptest` | 1.x | MIT/Apache-2.0 | GO | Property-based testing |
| `cargo-fuzz` + `AFL++` | Mainstream | MIT | GO | Fuzzing stack |
| `hax` / F* | v0.1.0 | Apache-2.0 | CAUTION | **5 semantic gaps** prouves Symbolic Software 7 avril 2026 |

**Protocol-level verification** : ProVerif ou Tamarin pour
identity lifecycle + trust aggregation + wire format handshake.

**Code-level verification** : `Kani` model checking (utilise par
AWS sur s2n-quic) pour assertions critiques. `Creusot` pour
pre/post conditions Rust (POPL 2026).

**Wire format codec** : spec formelle + property-based testing
via `proptest` + fuzzing `cargo-fuzz` + AFL++.

**`hax` limite** : post-Symbolic Software 7 avril 2026, `hax` OK
pour petites fonctions deterministes (codec, hash), NOT OK pour
claim "fully verified" primitives crypto. Pipeline ne couvre
pas while-loops fuel=0, debug vs release overflow, assumption
poisoning.

---

## Couche 12 — Research track (post-v1.0)

| Brique | Version 2026 | License | Verdict |
|---|---|---|---|
| `tfhe-rs` (Zama) | 1.4.0 | BSD-3-Clause | GO (H100 accelere) |
| `arkworks-rs` Groth16/Plonk | 0.4.x | MIT/Apache-2.0 | CAUTION (prototype academique) |
| `halo2` Zcash/PSE | 0.3.x | MIT | GO (Zcash/Scroll/Taiko prod) |
| MPC frameworks | Fragmente (garble, swanky) | Mixed | REPLACE (no unified crate) |

**Research items hors scope production** :

- **Anonymous kudos via ZK** : `arkworks-rs` Groth16 circuits ou
  `halo2` (Zcash-style anonymous credentials)
- **FHE workloads** : `tfhe-rs` 1.4.0 pour inference chiffree
  niche (100-1000x slowdown vs clear)
- **Secure multi-party computation** : threshold decryption via
  MPC frameworks fragmentes
- **Split inference cross-nodes** : MPC-based pour inference
  ultra-privee
- **Deadman switch** : heartbeat-based auto-disclosure pattern
  journalists, pas de crate standard, implementation custom

---

## Position vs OSS state-of-the-art 2026

Par dimension, positionnement post-blueprint implemente :

| Dimension | Tier SBFB | Benchmark OSS atteint |
|---|---|---|
| Memory safety language-wide | **>** Signal/Tor/SecureDrop | Rust entier vs Java+C / C / Python |
| Crypto primitives | **=** AWS/WhatsApp/Chrome | `aws-lc-rs` FIPS 140-3 ML-KEM |
| PQC hybride transport + signing | **=** Signal PQXDH 2024, WhatsApp 2024 | rustls prefer-post-quantum + ML-KEM-1024 + ML-DSA |
| App sandbox cross-process | **=** Shopify/Fastly/Figma | Wasmtime 43.0.1+ + WASI Preview 2 |
| Supply chain provenance | **=** Mozilla/Kubernetes | cargo-vet + Sigstore + Nix + multi-builder attestation |
| At-rest encryption + duress | **=** Briar + VeraCrypt combined | rage + sntrup761 + TPM + panic wipe |
| Formal verif critical path | **=** Signal Triple Ratchet SPQR | ProVerif + Kani + Creusot |
| Fuzzing infrastructure | **=** Mozilla/Google | proptest + cargo-fuzz + AFL++ |
| TEE attestation | **=** Confidential Computing Consortium | SEV-SNP + TDX + H100 CC + NRAS |
| Traffic shaping | **=** Mullvad DAITA | maybenot state machines |
| Transport anonyme | **=** Briar (via Arti embed) | arti 2.2 + contribute bridges |
| Sybil resistance | **=** Tor consensus pattern | FROST DAs + PoW + W3C VC 2.0 |
| Eclipse resistance | **=** Bitcoin/Tor guard | Guards weighted + AS-diversity + honeypot |

**Dimensions leader unique OSS** (aucun concurrent) :

1. **Compute-sharing defense-in-depth** — 7 classes menace
   adressees avec controles specifiques (prompt leak + result
   spoof + compute theft + model extract + prompt inject +
   side-channel + DoS flood)
2. **Verified P2P app deploy** — multi-forge + multi-builder
   attestation + SLSA L3+ reproducible from source, le tout
   P2P sans host central
3. **Runtime GPU consent per-task** — 4 niveaux + caps
   W/VRAM/heures worker-side re-lu live, reset minuit-local

---

## Invariants structurels non-resolus par le blueprint

Meme blueprint implemente parfaitement en production, **2
invariants temporels** restent irreductibles :

1. **Years-of-public-cryptanalysis** — ML-KEM standardise NIST
   aout 2024. Chaque primitive crypto necessite 5-10 ans
   d'analyse adversariale publique pour maturite. Signal Triple
   Ratchet existe depuis 2013. Invariant temporel independant
   de la qualite du code SBFB.
2. **Audit externe publie + findings remedie + retest published**
   — code theoriquement excellent != code validated. Sans
   Cure53/ToB report published, claim reste unverified. Solvable
   via OTF Red Team Lab + OpenSSF Alpha-Omega + candidature
   NLnet, mais necessite le process + temps.

**3 invariants non-techniques** :

3. **Adoption reseau** — Tor 6000+ relays volontaires, Signal
   100M+ users = 10-20 ans adoption organique. Un design n'a
   pas d'utilisateurs.
4. **Solo maintainer social engineering risk** — XZ-pattern
   (Jia Tan, 2 ans de trust-building) cible maintainers
   individuels, pas le code. Vetting 30-days + 4-eyes + M-of-N
   ralentit, n'elimine pas.
5. **Upstream dependances black swan** — 0-day Chromium (CVE
   2025-2783 Mojo ITW, CVE-2025-4609 ipcz $250k), 0-day Linux
   kernel, TPM vendor compromise, NIST backdoor (precedent
   Dual-EC-DRBG 2013). Hors controle architecte solo.

---

## Briques ajoutees post-validation

Identifiees manquantes dans blueprint initial, ajoutees apres
validation externe 2026 :

1. **`aws-lc-rs`** (AWS Libcrypto Rust) — alternative ML-KEM
   FIPS 140-3, remplace `libcrux-ml-kem` pour prod critique
2. **`gotatun`** (Mullvad) — WireGuard Rust + DAITA/Maybenot
   integre, audite Assured fev 2026, crash rate 0.40% → 0.01%
3. **`ring`** — foundation crypto Rust, present iroh/rustls
4. **`zeroize`** — memoire clees zeroisee, standard de facto
5. **`secrecy`** — wrapper typed secrets, standard
6. **`sntrup761x25519`** age plugin — PQC at-rest mature (deja
   OpenSSH)
7. **`Creusot` 0.9.0** — Rust verification production-grade,
   complement `hax`
8. **`Kani` 0.66.0** (AWS) — model checking Rust,
   Firecracker/s2n-quic prod

---

## Briques retirees du blueprint initial

Invalidees par validation externe 2026 :

1. **FROST-ML-DSA** — papiers TALUS/Quorus 2025-2026 academiques,
   zero impl Rust publiee. Research-future.
2. **`libcrux-ml-kem` pour prod critique** — 5 semantic gaps
   pipeline hax→F* demontres Symbolic Software 7 avril 2026.
   Remplace par `aws-lc-rs`.
3. **`zkgroup` crate independent** — archive, absorbe libsignal
   qui decourage usage externe.
4. **HyParView / Plumtree Rust** — aucune impl mature publiee.
5. **`hickory-dns` production** — README dit "not recommended".
   Remplace reqwest + Cloudflare DoH direct.
6. **`lyrebird` Rust embed direct** — pas de port Rust, Go only.
   Delegue a `arti`.
7. **`capsicum-rs`** — FreeBSD only, hors scope SBFB.
8. **`cryptsetup-rs` / LUKS2 Rust bindings** — pas mature,
   LGPL-2.1 complexifie. `rage` at-rest suffit.
9. **Claim "libcrux fully verified"** — downgrader post-findings
   Symbolic Software.

---

## Verdict blueprint corrige

Avec les 8 ajouts + 9 retraits + 3 zones rouges corrigees
(wasmtime 43.0.1+, libp2p-gossipsub 0.49.4+, ML-KEM
aws-lc-rs/pas-libcrux) :

**Sur niveau de securite code** : egale ou depasse la plupart
OSS comparables sur toutes les dimensions ou SBFB joue.
**Leader unique** sur 3 dimensions specifiques (compute
defense, verified P2P deploy, runtime GPU consent).

**Sur qualite du code** : **superieur structurellement** via
Rust memory-safe entier (vs Signal Java+C, Tor C, SecureDrop
Python). Formal verification critical path matche Signal
Triple Ratchet-tier. Fuzzing infrastructure matche
Mozilla/Google.

**Sur robustesse technique** : defense-in-depth multi-layer +
zero-trust inter-composants + redundancy voting + ephemeral
workers + rate-limit multi-axis + traffic shaping = etat-de-
l'art theorique OSS 2026.

**Claim defendable** :

> SBFB blueprint corrige implemente en production = premier
> systeme OSS combinant formal-verified critical path (ProVerif
> + Kani + Creusot), Tor-tier overlay anonymity (Arti embed +
> contribute bridges + FROST directory authorities), Wasmtime
> capability-based app isolation 43.0.1+, TEE-attested GPU
> compute-sharing avec defense-in-depth 7-classes, verified
> deploy multi-builder reproducible SLSA L3+ transparency,
> memory-safe Rust entier, gouvernance multi-juridiction
> fondation avec continuous audit. Pas concurrent Signal/Tor
> sur leurs surfaces messaging/transport, complementary layer
> pour apps P2P compute — niche unique.

**Nuance honnete** : robustesse **theorique** = state-of-the-art
OSS 2026. Robustesse **eprouvee** = necessite audit externe
publie remedie + temps cryptanalyse + exposition adversaires
reels. Les 5 invariants structurels (2 techniques temporels, 3
non-techniques) ne se raccourcissent pas par le code.

---

## References

Docs officielles + advisories valides 2026 :

- [aws-lc-rs (AWS)](https://github.com/aws/aws-lc-rs)
- [ed25519-dalek 2.1](https://docs.rs/ed25519-dalek/2.1.0/)
- [frost-ed25519 2.0 (Zcash Foundation)](https://zfnd.org/frost-reference-implementation-v1-0-0-stable-release/)
- [quinn 0.11.9](https://github.com/quinn-rs/quinn/releases)
- [s2n-quic 1.68 (AWS, Kani-verified)](https://github.com/aws/s2n-quic)
- [rustls prefer-post-quantum](https://docs.rs/rustls-post-quantum/)
- [arti 2.2 (Tor Project)](https://blog.torproject.org/)
- [wasmtime 43.0.1 advisories](https://bytecodealliance.org/articles/wasmtime-security-advisories)
- [libp2p-gossipsub 0.49.4 CVE-2026-33040](https://github.com/libp2p/rust-libp2p/security/advisories/GHSA-gc42-3jg7-rxr2)
- [Symbolic Software — hax semantic gaps (7 avril 2026)](https://symbolic.software/blog/2026-04-07-cryspen-hax/)
- [Kani 0.66 (AWS)](https://github.com/model-checking/kani)
- [Creusot 0.9.0 (POPL 2026)](https://creusot-rs.github.io/)
- [maybenot (Mullvad DAITA)](https://github.com/maybenot-io/maybenot)
- [rage + sntrup761 plugin (str4d)](https://github.com/str4d/rage)
- [Nix flakes stable 2026](https://nixos.org/manual/nix/stable/)
- [cargo-vet (Mozilla)](https://mozilla.github.io/cargo-vet/)
- [osv-scanner v2 (Google)](https://security.googleblog.com/2025/03/announcing-osv-scanner-v2-vulnerability.html)
- [Sigstore cosign v3](https://blog.sigstore.dev/cosign-3-0-available/)
- [NIST FIPS 203 ML-KEM](https://csrc.nist.gov/pubs/fips/203/final)
- [NIST FIPS 204 ML-DSA](https://csrc.nist.gov/pubs/fips/204/final)
- [OpenSSF Alpha-Omega $12.5M mars 2026](https://alpha-omega.dev/blog/)
- [OTF Red Team Lab](https://www.opentech.fund/labs/red-team-lab/)
- [NLnet NGI0 Commons Fund](https://nlnet.nl/commonsfund/)
- [W3C VC 2.0 Recommendation mai 2025](https://www.w3.org/press-releases/2025/verifiable-credentials-2-0/)
- [Tamarin Prover](https://tamarin-prover.github.io/)
- [ProVerif](https://bblanche.gitlabpages.inria.fr/proverif/)

---

**Document pensee long-terme** — pas d'engagement sprint-par-sprint.
Pour sequencing tactique voir [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md).
Pour threat model de base voir [`THREAT_MODEL.md`](THREAT_MODEL.md),
[`ADVERSARIES.md`](ADVERSARIES.md), [`P2P_THREATS.md`](P2P_THREATS.md),
[`COMPUTE_THREATS.md`](COMPUTE_THREATS.md).
