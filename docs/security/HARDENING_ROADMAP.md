# Hardening Roadmap — Sprint 18-30

**Sprint 17 Phase D livrable autoritaire.** Consolide les threats
identifies dans :

- [`ADVERSARIES.md`](ADVERSARIES.md) (Phase A) — tiers T0-T5
- [`ATTACK_SCENARIOS.md`](ATTACK_SCENARIOS.md) (Phase A) — 12 scenarios
- [`P2P_THREATS.md`](P2P_THREATS.md) (Phase B) — 7 vecteurs reseau
- [`COMPUTE_THREATS.md`](COMPUTE_THREATS.md) (Phase C) — 7 classes compute
- [`THREAT_MODEL.md`](THREAT_MODEL.md) (S16) — STRIDE/LINDDUN baseline

en une sequence Sprint 18-30 chiffree, priorisee par
`(impact × likelihood) / effort`, avec dependency graph et
mapping des release gates 1-4 (voir [`RELEASE_GATES.md`](RELEASE_GATES.md)
Phase E).

Scope : specification + sequencing. Zero code. Chaque ligne
`Sprint N` ici est un engagement sprint kickoff — tout decalage
doit etre justifie par un audit de mi-parcours ou un event
externe (CVE, partnership, incident).

---

## 1. Threat × Mitigation matrix

Colonnes :

- **ID** : prefixe A- (attack scenario), B- (P2P), C- (compute)
- **Threat** : nom court
- **Tier max** : adversaire tier maximum realistement atteignable
- **App-risk** : severite mise en danger (L/M/H/C = critical)
- **Coverage** : ❌ absent, ⚠️ partiel, ✅ couvert post-S16
- **Effort** : S (<300 LOC, <1 sprint), M (300-1000, 1 sprint),
  L (1000-2500, 2 sprints), XL (>2500, 3+ sprints)
- **Dep** : mitigations bloquantes

| ID | Threat | Tier max | App-risk | Coverage | Effort | Dep |
|---|---|---|---|---|---|---|
| **A-S1** | CSP bypass iframe | T1 | M | ⚠️ (S12 CSP base) | S | — |
| **A-S2** | DNS rebinding daemon | T1 | H | ✅ (S16-A Origin+Host) | — | — |
| **A-S3** | Supply chain repo | T2 | H | ⚠️ (Keyoxide S14) | M | reproducible-builds |
| **A-S4** | Crypto-mining via GPU share | T2 | M | ⚠️ (caps S16-C) | M | C-ComputeTheft |
| **A-S5** | Prompt exfil via fake AI app | T2 | H | ❌ | M | C-PromptLeak |
| **A-S6** | Discredit via fake vulns | T3 | H | ❌ | M | responsible-disclosure |
| **A-S7** | Maintainer infiltration | T3 | C | ⚠️ (AGPL+git) | M | CODEOWNERS+release-trans. |
| **A-S8** | Dragnet metadata corr. | T4 | C | ❌ | XL | B-TrafAnalysis |
| **A-S9** | Checkpoint seize forensics | T5 | C | ❌ | XL | encryption-at-rest+duress |
| **A-S10** | Turned contributor | T5 | C | ❌ | L | key-rotation+revocation |
| **A-S11** | ISP national block | T5 | H | ❌ | XL | B-ISPBlock |
| **A-S12** | Fake curator via keypair theft | T5 | H | ❌ | M | revocation-list |
| **B-Sybil** | Sybil identity flood | T5 | H | ❌ | L | kudos-Sybil-resistant |
| **B-Eclipse** | Eclipse peer isolation | T5 | C | ❌ | L | multi-relai-federation |
| **B-GossipPoison** | Gossip poisoning + DoS | T5 | M | ⚠️ (sig OK) | M | B-Sybil (PoW pre-req) |
| **B-DHT** | DHT/pkarr attacks | T5 | M | ⚠️ (sig OK) | M | multi-relai |
| **B-BGP** | BGP hijack / relay block | T5 | C | ⚠️ (E2E content) | XL | multi-relai+pluggable-tp |
| **B-TrafAnalysis** | Traffic analysis metadata | T5 | C | ❌ | XL | Tor/Nym+padding |
| **B-ISPBlock** | Country-level block | T5 | H | ❌ | XL | pluggable-transports |
| **C-PromptLeak** | Prompt leakage worker | T5 | H | ❌ | XL | ephemeral-workers+TEE |
| **C-ResultSpoof** | Result spoofing | T5 | H | ❌ | L | redundancy-voting |
| **C-ComputeTheft** | Compute theft / mining | T3 | M | ⚠️ (caps S16-C) | M | NVML-profile |
| **C-ModelExtract** | Model extraction | T3 | M | ❌ | M | rate-limit-per-consumer |
| **C-PromptInject** | Prompt injection exfil | T3 | H | ❌ | L | structured-output+filter |
| **C-SideChannel** | Side-channel GPU | T5 | H | ❌ | L | VRAM-wipe+driver-updates |
| **C-DosFlood** | DoS task flood | T5 | H | ⚠️ (caps hw) | M | rate-limit+Sybil |

**Stats** : 27 threats. 15 ❌ absent, 9 ⚠️ partiel, 1 ✅ (S16
Phase A DNS rebinding fix). Effort total : 2×S, 10×M, 9×L, 6×XL.

---

## 2. Prioritization framework

**Score = (impact × likelihood) / effort**, chaque dimension 1-5.

- **Impact 1-5** : 1=frustration, 2=degraded UX, 3=data leak PII,
  4=rep/legal compromise, 5=life-safety (cf tier-mapping
  [`ADVERSARIES.md §3`](ADVERSARIES.md#3-mapping-tier--app-risk-gate))
- **Likelihood 1-5** : 1=rare given tier, 2=possible given tier,
  3=likely given tier, 4=common post-exploit, 5=certain given
  tier active
- **Effort 1-5** : 1=quick-win (<300 LOC, <1j), 2=S, 3=M, 4=L,
  5=XL

**Interpretation score** :

- **>3** : quick-wins + critical — **Sprint 18-20 obligatoires**
- **2-3** : medium priority — **Sprint 20-25**
- **1-2** : long-term — **Sprint 25-30** ou research-track
- **<1** : deferred v2 (generalement XL effort + tier marginal)

**Top-scoring** (calculs rapides, likelihood contextualise Gate 2
median app) :

| ID | I | L | E | Score | Sprint cible |
|---|---|---|---|---|---|
| A-S3 Supply chain | 4 | 4 | 3 | 5.3 | S18 |
| B-Eclipse | 5 | 3 | 4 | 3.75 | S18-20 |
| B-BGP relay | 5 | 3 | 5 | 3.0 | S18 (multi-relai) |
| C-PromptInject | 4 | 4 | 4 | 4.0 | S20-22 |
| B-Sybil | 4 | 4 | 4 | 4.0 | S19-22 |
| C-DosFlood | 4 | 4 | 3 | 5.3 | S21 |
| A-S9 Checkpoint | 5 | 2 | 5 | 2.0 | S20 (XL) |
| C-PromptLeak TEE | 5 | 2 | 5 | 2.0 | S30+ (Gate 4) |

**Arbitrages issus Phase C §10** :

- **§1 prompt leak + §2 spoofing** partagent le TEE H100 big-rock
  → **grouper S30+** (amortissement cout hardware, partenariats
  ONG pour provisionnement).
- **§3 theft + §6 side-channel** touchent worker-core → **sequencer**
  NVML profile (S22) avant VRAM wipe (S22-23) avant no-sharing
  policy (S26).
- **§4 extraction + §7 DoS** meme primitive rate-limit
  per-consumer → **mutualiser S21-22** (une seule implementation
  sliding-window, deux uses).
- **§5 injection avant tool-calling** → structured output (S20)
  **bloque** tool-calling design S22, pas l'inverse.
- **Transverse Sybil first** : §7 DoS et §4 extraction dependent
  de **kudos Sybil-resistant**. Sans cela rate-limit contourne
  par botnet identities. Sprint 19 PoW = prerequis S21 rate-limit.

---

## 3. Sprint roadmap Sprint 18-30

### Sprint 18 — Quick wins + supply chain baseline

- **Goal** : fermer gaps S=Small effort + etablir chaine
  reproductible. Consolide Gate 1.
- **Items** :
  - `cargo-audit` / `pip-audit` / `npm audit` en CI (bloque PR
    sur CVE critical) — ~150 LOC
  - Reproducible builds Rust (`--locked`, SOURCE_DATE_EPOCH) +
    SHA256 artifact attestation — ~200 LOC
  - Radicle mirror + warrant canary page minimal — ~300 LOC
  - Driver update check au launcher startup (NVIDIA CVE DB
    scrape) — ~250 LOC
  - Multi-relai federation **phase 1** : bootstrap list hardcoded
    n0 + 2 fallbacks, retry round-robin — ~400 LOC
  - DHT redundant lookup (3 relais pkarr paralleles, quorum 2/3)
    — ~200 LOC
- **LOC total** : ~1500
- **Tests delta** : +35
- **Dependencies** : aucune (quick wins)
- **Gate unlock** : Gate 1 (DnD Forge) debloque fin S18

### Sprint 19 — PoW gossip + TLS pinning + DHT

- **Goal** : imposer cost-of-identity minimal + durcir transport.
- **Items** :
  - PoW Hashcash per-gossip-subscribe (difficulty 2^18 initial,
    adjustable per-relai) — ~400 LOC
  - TLS cert pinning relays (iroh upstream contrib) — ~200 LOC
  - Delayed upload queue (randomized 0-5min batching) — ~300 LOC
  - pkarr relay self-hosted (docker image + ops doc) — ~400 LOC
- **LOC total** : ~1300
- **Tests delta** : +40
- **Dependencies** : S18 multi-relai pour TLS pinning
- **Gate unlock** : —

### Sprint 20 — Encryption at rest big-rock

- **Goal** : eliminer checkpoint-seize risk (A-S9). Gate 2
  prerequis critique.
- **Items** :
  - Keypair encryption at rest via Keychain (macOS) / DPAPI
    (Windows) / libsecret (Linux) — ~800 LOC
  - Duress PIN unlock (fake keypair → noop responses) — ~500 LOC
  - Panic wipe 5-tap gesture (shell shortcut Ctrl+Shift+Alt+W,
    wipe keypair + state sqlite + blob cache) — ~400 LOC
  - Structured output llama.cpp grammar (JSON schema
    enforcement) — ~300 LOC
  - Warrant canary auto-publish (gossip heartbeat monthly) —
    ~200 LOC
  - Dual-transport detection + WebSocket fallback TCP 443 —
    ~300 LOC
- **LOC total** : ~2500
- **Tests delta** : +65
- **Dependencies** : S18 multi-relai (warrant canary)
- **Gate unlock** : —

### Sprint 21 — Rate-limit + client-side redaction

- **Goal** : mutualiser rate-limit per-consumer pour §4 extraction
  + §7 DoS flood. SDK redaction pour apps Gate 2+.
- **Items** :
  - Rate limit sliding-window per-(consumer, worker, model)
    worker-core — ~400 LOC
  - Client-side redaction SDK module (regex PII + optional spaCy
    NER wasm) — ~500 LOC
  - Output filter lib SDK (system prompt echo detection, beacon
    chars) — ~300 LOC
  - Quarantine queue gossip (unverified-high-rate messages hold
    15min, manual flush) — ~200 LOC
- **LOC total** : ~1400
- **Tests delta** : +50
- **Dependencies** : S19 PoW (sinon rate-limit contourne)
- **Gate unlock** : —

### Sprint 22 — Sybil resistance + compute detection + voting

- **Goal** : consolider Gate 2 via Sybil-resistance kudos-weighted
  + detection runtime compute theft + redundancy voting Gate 3
  base.
- **Items** :
  - Kudos-weighted gossip admission (nodes >N kudos full voice,
    others read-only) — ~600 LOC
  - NVML util + duree profile worker-core, log-only baseline —
    ~400 LOC
  - Sandbox tool-calling allow-list strict + dry-run — ~500 LOC
  - Redundancy voting Task.redundancy_factor (3 workers majority)
    — ~400 LOC
  - Spot-check watermark canari (consumer glisse 1/N prompt
    verif) — ~300 LOC
- **LOC total** : ~2200
- **Tests delta** : +75
- **Dependencies** : S19 PoW, S21 rate-limit
- **Gate unlock** : Gate 2 (TransLingua, FamilyScan) debloque

### Sprint 23 — Ephemeral workers + escalating PoW + honeypot

- **Goal** : durcir contre worker-infiltre (honey-worker) + anti-
  extraction modele.
- **Items** :
  - Ephemeral workers pattern (restart after N tasks +
    `cudaMemset` VRAM wipe) — ~500 LOC
  - Escalating PoW per-(consumer, model) — difficulty ramp
    geometrique — ~300 LOC
  - Honeypot Eclipse detection (canary peer rotation, alert si
    toujours meme neighborhood) — ~400 LOC
  - Exponential cooldown per-identity overflow (1/2/4/... min) —
    ~200 LOC
  - Traffic padding design doc + iroh upstream PR draft — ~100
    LOC (mostly docs)
- **LOC total** : ~1500
- **Tests delta** : +60
- **Dependencies** : S22 Sybil kudos
- **Gate unlock** : —

### Sprint 24 — Re-run sampling + DNS fallback + key rotation

- **Goal** : detection runtime compute theft + durcissement
  revocation.
- **Items** :
  - Consumer random re-run 1-5% sampling + auto-report curator
    divergence — ~400 LOC
  - DNS-based fallback DHT (DoH + DoT) — ~300 LOC
  - Domain fronting design doc + CDN partners legal review —
    ~200 LOC
  - Ed25519 key rotation ceremony + revocation list gossip —
    ~500 LOC
- **LOC total** : ~1400
- **Tests delta** : +50
- **Dependencies** : S22 redundancy voting (pour seuil detection)
- **Gate unlock** : —

### Sprint 25 — Tor transport phase 1 + per-app quota + RAG

- **Goal** : commencer Tor integration Gate 3 prep.
- **Items** :
  - Tor SOCKS proxy wiring (iroh connection option Tor) — ~800 LOC
  - Per-app rate budget global coordinator-side — ~300 LOC
  - RAG sanitization pipeline (detox injection sources externes)
    — ~600 LOC
  - Pluggable transports obfs4 integration (fork-patch iroh) —
    ~400 LOC
- **LOC total** : ~2100
- **Tests delta** : +55
- **Dependencies** : S18 multi-relai, S24 domain fronting legal
- **Gate unlock** : —

### Sprint 26 — Tor complete + curator reliable + GPU lockup

- **Goal** : finaliser Tor transport + liste curateurs
  reliable-workers + policy no-GPU-sharing.
- **Items** :
  - Tor transport prod-ready (auto-bootstrap bridge list,
    fallback) — ~800 LOC
  - Reliable-workers curator list (extension namespace S10) —
    ~400 LOC
  - GPU exclusive lockup (process namespace + cgroups Linux,
    job object Windows) — ~500 LOC
  - No-sharing policy (worker-core detecte autre process
    significatif sur GPU, refuse task ou warn) — ~300 LOC
- **LOC total** : ~2000
- **Tests delta** : +60
- **Dependencies** : S25 Tor phase 1, S10 curator infra
- **Gate unlock** : —

### Sprint 27 — Watermark model + Sybil mature + Gate 3 push

- **Goal** : PolitiScan-ready suite complete.
- **Items** :
  - Watermark injection opt-in (technique Kirchenbauer 2023
    green-list tokens biased) — ~500 LOC
  - Sybil kudos-weighted mature : trust-web bootstrapped par
    Amnesty-class ONG pour Gate 4 — ~400 LOC
  - PolitiScan-specific hardening items (audit S16-S26 gaps) —
    ~300 LOC
- **LOC total** : ~1200
- **Tests delta** : +45
- **Dependencies** : S22 Sybil base, S26 Tor complete
- **Gate unlock** : Gate 3 (PolitiScan, NEXUS cold-case) debloque

### Sprint 28 — Nym mixnet + MIG + external audit prep

- **Goal** : Gate 4 prep — metadata protection maximum + isolation
  hardware.
- **Items** :
  - Nym mixnet integration phase 1 (SOCKS wrapper, test
    feasibility) — ~1500 LOC
  - MIG partitioning A100/H100 opt-in config — ~500 LOC
  - External audit scope doc + RFP Cure53/ToB — ~200 LOC (docs)
  - Amnesty/HRW/CPJ partnership outreach (non-code) — 0 LOC
- **LOC total** : ~2200
- **Tests delta** : +40
- **Dependencies** : S27 Sybil mature
- **Gate unlock** : —

### Sprint 29 — External audit + remediation buffer

- **Goal** : audit externe paid Cure53 ou Trail of Bits +
  remediation.
- **Items** :
  - Audit execution (~50-100k$ budget, 4-8 semaines)
  - Remediation findings (buffer ~1500 LOC estime)
  - Public disclosure responsible-disclosure policy +
    security.txt — ~200 LOC
- **LOC total** : ~1700 (majoritairement fix audit findings)
- **Tests delta** : +50
- **Dependencies** : S28 scope doc
- **Gate unlock** : —

### Sprint 30 — TEE H100 eval + split inference research

- **Goal** : Gate 4 eligibility partielle. TEE attestation
  big-rock pour Gate 4 complet.
- **Items** :
  - TEE H100 attestation integration (hardware partenaire ONG) —
    ~1200 LOC
  - Split inference research prototype (hors v1, document
    findings) — ~300 LOC (docs)
- **LOC total** : ~1500
- **Tests delta** : +25
- **Dependencies** : S28 MIG, S29 audit
- **Gate unlock** : Gate 4 eligibility partielle (prerequisites
  complets ; release reel requiert S31+ partnership + beta ferme
  18 mois)

**Total S18-30** : ~22100 LOC, ~650 tests delta, 13 sprints.

---

## 4. Quick-wins list

Items score >3 + effort S-M, landable Sprint 18-19 sans blocker.

| Item | Sprint | LOC | Effort |
|---|---|---|---|
| cargo-audit en CI | S18 | ~100 | 1 jour |
| pip-audit en CI | S18 | ~80 | 0.5 jour |
| npm audit en CI | S18 | ~60 | 0.5 jour |
| Driver update warn launcher | S18 | ~250 | 2 jours |
| Multi-relai phase 1 bootstrap | S18 | ~400 | 3 jours |
| DHT redundant lookup 3/quorum 2 | S18 | ~200 | 2 jours |
| Radicle mirror + warrant canary | S18 | ~300 | 2 jours |
| PoW Hashcash gossip subscribe | S19 | ~400 | 3 jours |
| TLS cert pinning relays | S19 | ~200 | 1.5 jours |
| Delayed upload queue 0-5min | S19 | ~300 | 2 jours |
| Token rotation automatique (S16 carry) | S18 | ~150 | 1 jour |
| Rate limit per-identity sliding (§7) | S21 | ~400 | 3 jours |

**12 quick-wins**, ~2840 LOC cumule, ~21 jours-dev si sequencees.
Repartis S18 (~7 items), S19 (~3 items), S21 (~1 item — Sybil
prerequis).

---

## 5. Big-rocks

Items score variable mais effort XL, necessitent sprint dedie
ou multi-sprint.

| Item | Sprint cible | LOC estimee | Motif |
|---|---|---|---|
| Encryption at rest keypair + duress PIN + panic wipe | S20 | ~2000 | Gate 2 prerequis (A-S9) |
| Tor transport integration (phases S25+S26) | S25-26 | ~2000 | Gate 3 prerequis (B-BGP, B-ISPBlock partiel) |
| Nym mixnet integration (research + phase 1) | S28+ | ~3000 | Gate 4 prerequis (B-TrafAnalysis max) |
| TEE H100 attestation | S30+ | ~1200 | Gate 4 prerequis (C-PromptLeak + C-ResultSpoof) |
| Relay federation protocol complet | S18-19 | ~1500 | Gate 1→2 bridge (B-Eclipse, A-S11) |
| External audit Cure53/ToB + remediation | S29 | ~1500 budget fix | Gate 3 obligatoire |
| Pluggable transports complet (obfs4+meek+Snowflake) | S25-26 | ~1500 | Gate 3 B-ISPBlock |
| Kudos-weighted Sybil resistance mature | S22+S27 | ~1000 | Transverse (B-Sybil, C-DosFlood, C-ModelExtract) |

**8 big-rocks**, ~13700 LOC cumule. Chaque big-rock est une
**decision go/no-go par sprint kickoff** — l'equipe doit avoir
le budget + le partenariat + la clarte scope AVANT d'ouvrir le
sprint.

---

## 6. Dependency graph

```
S18 multi-relai federation  ──────┬──> S19 TLS pinning relays
     ( A-S11, B-BGP, B-Eclipse )  ├──> S20 warrant canary
                                   ├──> S24 domain fronting
                                   └──> S25 Tor phase 1 (bridges)

S18 reproducible builds ──────────> S18 Radicle mirror
     ( A-S3, A-S7 )                    ( A-S7 maintainer infil. )

S19 PoW Hashcash gossip ──────────┬──> S21 rate-limit per-consumer
     ( B-Sybil, B-GossipPoison )  │        ( C-DosFlood, C-ModelExtract )
                                   └──> S22 kudos-weighted admission

S20 encryption at rest ───────────> S22 duress unlock testing
     ( A-S9, A-S10 )                    ( A-S9 full coverage )

S20 structured output grammar ────> S22 sandbox tool-calling
     ( C-PromptInject )                 ( C-PromptInject escalation block )

S22 Sybil kudos base ─────────────┬──> S23 escalating PoW
                                   ├──> S23 ephemeral workers
                                   ├──> S26 reliable-workers curator
                                   └──> S27 Sybil mature (trust-web ONG)

S22 NVML baseline profile ────────> S24 random re-run sampling
     ( C-ComputeTheft )                 ( C-ComputeTheft detection )

S25 Tor phase 1 ──────────────────> S26 Tor prod-ready
                                        └──> S28 Nym mixnet phase 1

S27 Sybil mature + S28 Nym + S28 MIG ─> S29 external audit
                                             └──> S30 TEE H100 (Gate 4 prep)

S29 external audit remediation ───> S30 Gate 4 eligibility
```

**Invariants critiques** :

- **Sybil resistance → rate-limit** : S19 PoW + S22 kudos-weighted
  precedes S21 rate-limit mature. Rate-limit sans Sybil = botnet
  trivialement contournable (confirme P2P_THREATS §1 + COMPUTE
  §7).
- **Multi-relai → tout transport durci** : S18 federation est
  racine de S19-S28 chain transport. Pas de federation = single
  point of failure n0 persist.
- **Encryption at rest → Keychain/DPAPI natif** : pas de lib
  Rust cross-platform complete — decision kickoff S20 : adapter
  `keyring-rs` OU wrapping platform-specific.
- **Tor → obfs4 bridges infrastructure** : S25 depend de la dispo
  de bridges operationnels. Partenariat EFF/Amnesty S28+ peut
  debloquer (operation bridges sponsorisees).
- **External audit → budget confirme** : S29 blocked si budget
  50-100k$ pas secure par S28 kickoff. Sinon push S31-32.

---

## 7. Gates debloquage sequencing

Table mapping Gate (1-4, cf [`RELEASE_GATES.md`](RELEASE_GATES.md)
Phase E) vs Sprint debloquant.

| Gate | Tier mitige | Sprint debloquant | Prerequis |
|---|---|---|---|
| **Gate 1** (DnD Forge, hello-world) | T0-T1 | S18 | Quick-wins S18 + audit S16 leve (deja fait) |
| **Gate 2** (TransLingua, FamilyScan) | T0-T2 | S22 | +encryption at rest (S20) +rate-limit (S21) +Sybil base (S22) +supply chain (S18) |
| **Gate 3** (PolitiScan, NEXUS cold-case) | T0-T3 | S27 | +Tor transport (S26) +redundancy voting (S22) +client-side redaction (S21) +RAG sanitization (S25) +reliable-worker curator (S26) +Sybil mature (S27) +audit externe publie (S29, noter que ordering S27<S29 = Gate 3 effectif est **S29**, pas S27) |
| **Gate 4** (LibanLive, war-crime doc) | T0-T5 | **S35+** | +Nym mixnet (S28-30+) +TEE H100 (S30+) +MIG (S28) +audit externe comprehensive (S29) +partenariat Amnesty/HRW/CPJ sign-off +18 mois beta ferme + ethics review board + formation OpSec contributeurs |

**Correction Gate 3** : S27 livre la suite technique mais Gate 3
operationnel requiert l'audit externe (S29). Donc **Gate 3
effectif = fin S29**, avec remediation incluse.

**Gate 4 n'est pas "fin S30"** : S30 livre TEE attestation qui
est un prerequis, mais les items non-code (partnership, beta
ferme 18 mois, ethics review board) decalent Gate 4 effectif a
**~S35-38** (cf [`ADVERSARIES.md §3.1`](ADVERSARIES.md#31-pourquoi-t5-non-atteignable-avant-gate-4-complet)).

**Ship-blocker ethique** : aucune app classee pour population
cible T5 (LibanLive-class) ne peut sortir en beta ouverte avant
Gate 4 effectif complet. Cette clause est structurelle — le code
sera techniquement capable de ship, le release **n'est pas
autorise** par policy. Voir [`RELEASE_GATES.md`](RELEASE_GATES.md)
Phase E pour enforcement mechanism.

**Escalation de gate** (app qui monte) : peut decaler le release
freeze selon gate cible. Ex : DnD Forge → hub social avec DMs
= Gate 2 = freeze jusqu'a S22. Ces transitions sont tracees par
le coordinator via `ProjectAnnouncement.gate_tier` (v? TBD
Sprint 18+).

---

**Fin Phase D**. Prochaine phase : [`RELEASE_GATES.md`](RELEASE_GATES.md)
(Phase E) — consolide gates 1-4 avec enforcement policy +
partnership strategy (Amnesty, HRW, CPJ, EFF, Cure53, ToB) +
responsible disclosure + warrant canary playbook.
