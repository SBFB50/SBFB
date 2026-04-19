<!--
written: 2026-02-15  # Sprint 17 Phase D
last_validated: 2026-04-20  # G2 — Sprint 22 Phase A livrée (`0bc499f` rate-limit engine wire + hot-reload + sample smoke, delta +7 Rust 659→666) + amendement S23 hors-sprint (ajout design doc `docs/fairness/CONTRIBUTION_FAMILIES_V1.md` + `/diagnostic/fairness` observability endpoint, pré-requis LT-3 post-v1.0 — cf. `docs/release/ROADMAP_COMMITMENTS.md §LT-3` + research `.planning/research/S22_contribution_family_sybil_matrix.md` commit `dbc4ceb`)
triggers_revalidate:
  - "iroh release > 0.97 (PkarrPublisher API + relay TLS hooks)"
  - "wasmtime LTS bump (CVE refresh §S18)"
  - "arti-client release > 1.x stable (S25-S26 reroute possible)"
  - "Tor PoW spec hspow change (impacte D2 PoW choice S19)"
  - "NIST PQC FIPS 203/204 ecosystem default (impacte S26+)"
  - "NVIDIA H100 CCM driver release (impacte S30)"
  - "Sprint S+2 commence vs sprint cible (S19+2=S21 → re-scan §3 S21)"
  - "frost-ed25519 release > 2.1 (CanarySigner FROST primitive S20 Phase E.2)"
  - "RFC 9591 erratum publication (FROST threshold spec, drives S20 E.2 + S30 Niveau 1 enforcement)"
audited_findings:
  - "2026-04-16 S19 deep analysis : D2 Hashcash daté vs Equi-X 2023, S20 keyring crate, S21 grammar ≠ prompt injection defense, S25 Arti 2.0 Feb 2026 disponible, S26+ PQC trop tardif (HNDL liability)"
  - "2026-04-16 S20 open : double-layer encryption at rest (Argon2id 64MiB + AES-256-GCM + OS keyring wrap KEK) requis — DPAPI user-scope gap confirmé Sygnia 2024 + SpecterOps 2026 (same-user process malicious = full bypass). Duress PIN pattern = fake keypair noop responses (pas wipe immédiat GrapheneOS-style), panic wipe = 5-tap gesture séparé. Structured output via llguidance (Rust Microsoft 50µs/token, llama.cpp -DLLAMA_LLGUIDANCE=ON) retenu over XGrammar (pas llama.cpp), Outlines (Python IPC), GBNF native (slower). PoW wire scope S20 Phase C (reclassé carry S19 A-2). TLS wire iroh T20 tech debt long-terme (iroh 0.97 ClientBuilder hook cfg-test only). DHT canary enforcement strict reporté post-Gate-2."
  - "2026-04-18 S20 Phase E pivot Option C deep-evolution (G8 codification, cf. .planning/active/sprint20_phase_E_pivot_proposal.md + sprint20_phase_E_preflight.md) : warrant canary auto-publish scheduler (plan §8.1 item 1 original) supprimé sur scan S2 — décision threat-model S18 E2 04c9621 toujours valide (cle Ed25519 accessible auto = compromission GHA = compromission cle = dead-man-switch cassé sous gag order). Federation foundations livrées : CanarySigner trait abstraction + FrostCanarySigner K-of-N (RFC 9591 jan 2025, ZF crate 2.1, ToB 2023 audit), DuressAck channel (gossip topic distinct, daily granularité), AttestationProvider trait + NoopAttestation (decouple signing != attestation), Federated CanaryRegistry coord-side (POST /api/canary/observed + GET /api/canary/network-health). E.6 ajusté inline post-G8 S1 finding : iroh 0.91 a supprimé l'option TCP raw → relays = WSS TCP 443 unique mode automatic, transport_probe.rs dégradé en diagnostic-only (probe 3x UDP QUIC + log warn + metric degraded_mode). Wire format CanarySigned v1 + DOMAIN_WARRANT_CANARY_V1 préservés (FROST sig = Ed25519 RFC 8032 byte-identical). Niveau 1 enforcement (cross-juridiction recruitment + TEE H100) added to §3 S30 line."
  - "2026-04-19 S21 CLOSED : 5 phases A-E livrées sur le thème rate-limit + PII SDK defense-in-depth + output filter + quarantine queue + tech debt batch. Phase A `63afe4e` rate-limit governor 0.10.2 GCRA worker-engine R1 (axum 0.7→0.8 bump prereq workspace-wide `5e67ce0` post-G8 pivot Option C). Phase B `d5b0035` PII SDK iframe (onnxruntime-web 1.24.3 + GLiNER PII edge ONNX). Phase C `23abb11` PII coord (presidio-analyzer 2.2.362 + GLiNERRecognizer extra [gliner] même modèle ONNX SoT + InvisibleText scanner curated + EED Levenshtein 0.85). Phase D `f830579` quarantine queue SQLite WAL + Typer CLI (réalignement coord-Python G8 SCOPE-CUT-CONSISTENT `a82e8db`). Phase E `49f0d32` tech debt batch — T-NN canary_wire_bytes JCS canonical (RFC 8785) + T-NN+1 CanaryRegistry verify Ed25519 at ingest via nexus_core.verify_canary PyO3 binding (path-dep nexus-shell-daemon-core ajoutée à nexus-core-py) + plan docs S20 §6 wire-point fix C-PLAN-1 + PATTERNS.md §P34 closeout (T-NN résolu + T-NN+1 résolu + T-NN+2 ouvert S22+ blocked tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen). Premier sprint avec G8 systématique 5/5 phases : 1 DESIGN-CONFLICT (Phase A axum bump) + 4 SCOPE-CUT-CONSISTENT (B/C/D/E). Cap G7 carry-overs respecté 2/2 → S22 : Meta-1 Radicle-v1.0 re-carry + T-NN+2 PATTERNS hors cap formel. Compteurs finals : 659 Rust / 185 SDK / 249+3 coord / 46 gov / 256 Vitest / 38 Playwright / ~1436 tests (+65 vs baseline 1371). Carries S22 audit_plan : P2-E-DURESS-ACK verify_duress_ack hors-scope explicit + P2-E-WIRE-PRE-LAUNCH-FIX check maturin develop --release fresh dans bootstrap §7 + P3-E-2 align build_canary serde_json → JCS pour cohérence + Meta-track hook coverage gap Phase D sans review.md + Phase A R1 rate_limit_policy.toml.sample manquant + Phase B drift Playwright PII end-to-end."
  - "2026-04-18 S21 open : D2 PII SDK requalifié post-research G2. Libellé roadmap §3 S21 original S17 'spaCy NER wasm ~500 LOC' obsolète 2026 (spaCy pas de port wasm officiel maintenu). Stack retenue defense-in-depth : client iframe = onnxruntime-web 1.24.3 (Microsoft, npm mars 2026) + @huggingface/transformers v4 tokenizer + knowledgator/gliner-pii-edge-v1.0 (Apache-2.0, 2024-01-29, F1 0.755, backbone à confirmer Phase B G8 S1 scan pre-first-line-of-code) + regex fallback curated ; coord-side = presidio-analyzer 2.2.362 (Microsoft MIT, 2026-03-15) + GLiNERRecognizer extra [gliner] + même modèle ONNX source-of-truth unique. Full Rust-first iframe (tract + GLiNER + wasm-bindgen) rejeté factuellement : tract 0.22.1 teste opset 9-18 vs GLiNER export opset 19 (DisentangledSelfAttention DeBERTa-v3 non documenté), tract wasm32-unknown-unknown (browser) non documenté officiellement (seul wasm32-wasi wasmtime), zero precedent production, gline-rs v1.0.1 (Rust GLiNER mainstream 01/2026) a choisi ort pas tract. Rust-wasm iframe realignement Option G reporté S22+ via tech debt T-NN+2 (re-evaluate triggers: tract opset 19 coverage OR ort wasm32-browser stable OR gline-rs wasm-bindgen target). Decisions D1 governor 0.10.2 GCRA + D3 LLM Guard 0.3.16 InvisibleText + PLeak EED + D4 SQLite WAL pattern S19 reuse + CLI sbfb quarantine. Cf. sprint21_kickoff.md §D1-D5 + sprint21_design_review.md."
-->

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
| **B-Sybil** | Sybil identity flood | T2+ pre-S19, T5 post-S19+S22 | H | ❌ | L | kudos-Sybil-resistant |
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
  - **Wire `nexus_core_rs::dht_quorum::redundant_resolve` au
    browse aggregator + curator runtime** (S18 audit fix C-1
    carry-over) — la primitive 3-paralleles-quorum-2/3 est livree
    + testee Phase C `9d0ad7a`, manque uniquement le glue avec
    iroh-relay 0.97 per-pkarr-relay lookup — ~150 LOC + 5 tests
- **LOC total** : ~1450
- **Tests delta** : +45
- **Dependencies** : S18 multi-relai pour TLS pinning
- **Gate unlock** : Eclipse-by-DHT defense pleinement active une
  fois le wire DHT quorum termine.

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

### Sprint 21 — Rate-limit + PII redaction defense-in-depth + output filter + quarantine queue

- **Goal** : mutualiser rate-limit per-consumer pour §4 extraction
  + §7 DoS flood. SDK redaction defense-in-depth (client iframe +
  coord) pour apps Gate 2+. Requalification post-research G2
  2026-04-18 vs libellé original S17 (spaCy wasm obsolète 2026).
- **Items** :
  - Rate limit sliding-window per-(consumer, worker, model)
    via `governor 0.10.2` GCRA + `tower-governor 0.8` axum
  - PII redaction SDK client iframe : `onnxruntime-web 1.24.3`
    (Microsoft, mars 2026) + `@huggingface/transformers` v4
    tokenizer + modèle HF `knowledgator/gliner-pii-edge-v1.0`
    (Apache-2.0, F1 0.755, backbone confirmé Phase B G8 S1) +
    regex fallback curated
  - PII redaction coord-side : `presidio-analyzer 2.2.362`
    (Microsoft MIT) + `GLiNERRecognizer` extra `[gliner]` +
    **même modèle ONNX source of truth unique**
  - Output filter : `LLM Guard 0.3.16` `InvisibleText` scanner
    (zero-width + PUA + Tag chars) + EED prompt echo detection
    (Levenshtein similarity > 0.85, seuil empirique Phase C
    configurable)
  - Quarantine queue SQLite WAL local + TTL 15 min + CLI
    `sbfb quarantine list/flush/drop` (pattern S19 `f238d31`
    `upload_queue.py` reuse)
- **Tests delta** : +50 (projection) — détail par phase
  `sprint21_plan.md §11`
- **Dependencies** : S19 PoW wire runtime (satisfait S20 Phase C
  `16b94ba`)
- **Gate unlock** : — (prerequisite Gate 2 consolidé apps
  confidentielles defense-in-depth PII + DoS)
- **Rust-wasm iframe realignement Option G** : reporté S22+ via
  tech debt `T-NN+2` (blocked tract opset 18 max vs GLiNER opset
  19 + wasm32-browser zero-precedent + gline-rs a choisi ort
  pas tract). Re-evaluate triggers documented
  `sprint21_carry_summary.md §2 T-NN+2`.

### Sprint 22 — Sybil resistance composition 3 couches + compute detection baseline + watermark primitive

- **Goal** : consolider Gate 2 via Sybil-resistance **composée
  3 couches** (flag FAIRNESS résolu arbitrage user 2026-04-19) +
  NVML baseline foundation S24 + watermark canari primitive.
- **Items** :
  - **Sybil-resistance admission composée 3 couches** (remplace
    "Kudos-weighted gossip admission" flag FAIRNESS résolu) :
    - **Couche 1** : age node_id ≥7j + PoW S19 réutilise
      `crates/nexus-core-rs/src/gossip.rs:140-162` + bootstrap
      allowlist ≤20 nodes (self-witness pre-v1.0) — ~400 LOC
    - **Couche 2** : `ContributorAttestation` predicate in-toto
      extend `ProvenanceRecord` S14 + wire
      `crates/nexus-core-rs/src/curator.rs:252-274` + coord Python
      registry — ~500 LOC + ~100 Python
    - **Couche 3** : design-only S22 (RFC
      `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` +
      `CONTRIBUTOR_ATTESTATION_PREDICATE.md` spec), implémentation
      distribuée S23-S27 — ~250 LOC docs
    > **FAIRNESS FLAG résolu 2026-04-19** : arbitrage user post-
    > synthèse research G2 (6 agents deep-dive) + G1 Design Review
    > Board CONDITIONAL PASS. Alternative (c) voice-per-project
    > binaire adoptée comme Couche 2 (ContributorAttestation),
    > alternative (a) age+PoW adoptée comme Couche 1, alternative
    > (b) Passport multi-signal rejetée (centralisé Human Passport +
    > dormant BrightID + biais OECD GH). Design-conflict Matthew
    > effect **latent one-layer-deeper** acknowledgé via
    > [`ROADMAP_COMMITMENTS §LT-1`](../release/ROADMAP_COMMITMENTS.md)
    > + TODO code comments Phase C. Post-v1.0 refonte Kudos v2 =
    > LT-1 commitment. Cf. `sprint22_kickoff.md §4 D1` +
    > `sprint22_design_review.md §3 P2-G1-3`.
  - NVML util + duree profile worker-core, **log-only baseline
    stats-only** (foundation S24, pas anomaly detection) — ~300 LOC
  - ~~Sandbox tool-calling allow-list strict + dry-run — ~500 LOC~~
    **DEFERRÉ post-S25** (pas de surface tool-call live S22 :
    seul S20 structured output, pas de tool-registry ouvert.
    OWASP LLM06:2025 Excessive Agency ne se déclenche pas sans
    tool-call live. Re-évaluation trigger : S25 RAG ou S28+
    tool-registry LLM).
  - ~~Redundancy voting Task.redundancy_factor (3 workers majority)
    — ~400 LOC~~ **DEFERRÉ S23** (mitigue C-ResultSpoof tier T5
    = surdimensionné 3 gates au-dessus Gate 2 T0-T2. BOINC/F@H
    ont opéré 1-worker prod 20 ans. Gate 3 track §7 ligne 554.
    Co-deferrer dependency S24 ligne 311 `S22 redundancy voting`
    → `S23 redundancy voting`).
  - Spot-check watermark canari-input (consumer glisse 1/N prompt
    known-answer Ed25519-signed rotatable, distinct watermark-
    output Kirchenbauer vulnérable BIRA 2025) — ~300 Python
  - **Wire-up debts S21 absorbés en phases dédiées** (pas carry-
    overs G7 formels — pattern S20 Phase C `16b94ba` PoW runtime
    wire carry S19 A-2 absorbé) :
    - Phase A rate-limit engine wire (P2-S21-1 + P2-S21-2 hot-
      reload) — ~250 Rust
    - Phase B GLiNER span-logits decoder iframe (P2-S21-3) — ~350 TS
  - Process fixes Phase F : P2-S21-4 README §4.X review→audit_plan
    règle + P2-S21-5 GHA CI cross-check commit→review file —
    ~150 LOC
- **LOC total** : ~2500 (+14% vs nominal 2200 absorption wire-up
  debts + G1 findings P0 ceremony + predicate spec)
- **Tests delta** : **+43** (+28 Rust Phase A/C/D + 6 Vitest Phase B
  + 9 Python coord Phase C/E)
- **Dependencies** : S19 PoW (live `edfc51b`), S21 rate-limit
  (primitive live `63afe4e`, wire S22 Phase A), S14 ProvenanceRecord
  (live `95807b1`, extend S22 Phase C)
- **Gate unlock** : **Gate 2** (TransLingua, FamilyScan,
  EHPAD-Lien) débloqué à la clôture S22 Phase F.

### Sprint 23 — Ephemeral workers + escalating PoW + honeypot + redundancy voting (carry S22) + contribution families foundation

- **Goal** : durcir contre worker-infiltre (honey-worker) + anti-
  extraction modele + ajouter redundancy voting foundation Gate 3 +
  poser les fondations Option F pour LT-3 (contribution families
  Sybil matrix post-v1.0).
- **Items** :
  - Ephemeral workers pattern (restart after N tasks +
    `cudaMemset` VRAM wipe) — ~500 LOC
  - Escalating PoW per-(consumer, model) — difficulty ramp
    geometrique — ~300 LOC
  - Honeypot Eclipse detection (canary peer rotation, alert si
    toujours meme neighborhood) — ~400 LOC
  - **Redundancy voting `Task.redundancy_factor` (3 workers
    majority)** — ~400 LOC **(carry S22 co-deferré 2026-04-19,
    cf. `sprint22_carry_summary.md §4`)**. Pré-requis dependency
    S24 re-run sampling (ligne 311 update `S22 → S23`).
  - Couche 3 design doc finalisation (RFC émis S22 Phase C) +
    delegation cert format Rust struct — ~100 LOC (design-only)
  - **Contribution families design doc (Option F 3 couches
    asymétriques)** — `docs/fairness/CONTRIBUTION_FAMILIES_V1.md`
    (~300 LOC docs) + `docs/fairness/KUDOS_V2_WIRE.md` (~100 LOC
    docs, spec-only pas de code) — design-only, pré-requis LT-3
    post-v1.0 (research `.planning/research/S22_contribution_
    family_sybil_matrix.md` commit `dbc4ceb`). Item **net-new
    2026-04-20 hors-sprint** (research capture + stub reserved
    S31) ajouté via `chore(planning)` amendement.
  - **Fairness observability endpoint** `/diagnostic/fairness` —
    ~80 LOC Python + ~40 LOC tests (coord-side, **zéro wire
    impact, zéro schema change**). Calcule Gini + top-5% +
    churn-rate-vs-hardware du ledger compute existant. Rend
    triggers LT-1 et LT-3 (condition b) factuellement
    mesurables dès Gate 2 activation (fin S22 Phase F). Pattern
    reuse `packages/nexus-coordinator/src/nexus_coordinator/
    api/` existant.
  - ~~Exponential cooldown per-identity overflow (1/2/4/... min) —
    ~200 LOC~~ **DEFERRÉ** (redondant avec Couche 1 age gate S22 —
    node_id <7j déjà bloqué, pas besoin cooldown exponentiel)
  - ~~Traffic padding design doc + iroh upstream PR draft — ~100
    LOC~~ **REPORTÉ S28** (aligné Nym mixnet integration phase 1)
- **LOC total** : ~2220 (vs 1500 initial, +200 redundancy voting
  carry S22, +400 docs contribution families design, +120
  observability endpoint) — dans les normes S20 ~104 Rust + S21
  ~65 tests delta, pas de scope creep monstre
- **Tests delta** : +70 (+65 initial + 5 observability endpoint)
- **Dependencies** : S22 Sybil base (Couches 1+2), S22 Phase A
  (rate-limit wire-up `0bc499f` pour pattern kudos ledger query)
- **Gate unlock** : —
- **Scope reduction documentée** (arbitrage S22 ouverture 2026-
  04-19 + 2026-04-20) : Exponential cooldown redondant Couche 1
  age gate ; Traffic padding aligné Nym mixnet S28 plutôt que
  doc-only S23. **LT-3 implementation deferred post-v1.0** —
  S23 livre seulement design docs + observability foundation,
  pas de code métier multi-famille.

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
- **Dependencies** : **S23 redundancy voting** (pour seuil
  detection — dep update 2026-04-19 post co-defer S22→S23, cf.
  `sprint22_carry_summary.md §4`)
- **Gate unlock** : —

### Sprint 25 — Tor transport phase 1 + per-app quota + RAG

- **Goal** : commencer Tor integration Gate 3 prep.
- **Items** :
  - Tor SOCKS proxy wiring via Arti standalone subprocess (iroh
    relay HTTPS fallback over SOCKS5, **NOT** QUIC direct)
  - Per-app rate budget global coordinator-side
  - RAG sanitization pipeline (detox injection sources externes)
  - Pluggable transports : **wire bridge config + lyrebird
    subprocess** (Tor Project Go binary upstream)
- **Tests delta** : +55
- **Dependencies** : S18 multi-relai, S24 domain fronting legal
- **Gate unlock** : —
- **Scope reduction documentee** (research deep avril 2026) :
  l'item original "obfs4 fork-patch iroh ~400 LOC" est
  **architecturalement infaisable** — Tor refuse UDP par design
  (cf. [Tor SOCKS extensions spec](https://spec.torproject.org/socks-extensions.html)),
  donc QUIC-over-Tor impossible. Le Tor mode = **HTTPS relay
  fallback only**, hole-punching iroh desactive. lyrebird
  subprocess via Arti gere natif les pluggable transports — pas
  besoin de fork iroh. Library-embed Arti differe S26+ (API
  pre-1.x instable, "expect breakage between now and 1.x" cf.
  [docs.rs/arti-client](https://docs.rs/arti-client/)).

### Sprint 26 — Tor complete + curator reliable + GPU lockup

- **Goal** : finaliser Tor transport + liste curateurs
  reliable-workers + policy no-GPU-sharing.
- **Items** :
  - Tor transport prod-ready (auto-bootstrap bridge list,
    Snowflake broker auto-fetch — **carry S25**)
  - Arti library-embed migration (subprocess → in-process,
    **carry S25**, conditionnel arti-client API stable >= 1.0)
  - Domain fronting implementation (CDN partner signe + ECH
    config + Snowflake-WebRTC fallback — **carry S24**, conditionnel
    legal review S24 abouti)
  - Reliable-workers curator list (extension namespace S10)
  - GPU exclusive lockup (process namespace + cgroups Linux,
    job object Windows)
  - No-sharing policy (worker-core detecte autre process
    significatif sur GPU, refuse task ou warn)
- **Tests delta** : +60
- **Dependencies** : S25 Tor phase 1, S24 domain fronting legal,
  S10 curator infra
- **Gate unlock** : —
- **Carry-overs S24/S25** (research deep avril 2026) : Domain
  fronting implem differe S24→S26 car majors (Google/AWS/Cloudflare/
  Fastly) ont ferme 2018-2024, seuls Snowflake-WebRTC + CDN
  minoritaires + ECH restent — demandent legal partnership prealable.
  Arti library-embed differe S25→S26 sur condition API stable.

### Sprint 27 — Watermark model + Couche 3 mature + Gate 3 push

- **Goal** : PolitiScan-ready suite complete.
- **Items** :
  - Watermark injection opt-in (technique Kirchenbauer 2023
    green-list tokens biased) — ~500 LOC
  - **Couche 3 mature (multi-forge cross-validate + trust-web
    Amnesty integration)** — ~700 LOC **(remplace "Sybil kudos-
    weighted mature" 2026-04-19 pivot, même flag FAIRNESS implicite
    que S22 item 1 original — cf. `sprint22_carry_summary.md §5`)**.
    Implem parser `git log --show-signature` offline + cache LRU
    SQLite + trust-web Amnesty-class ONG bootstrap seed.
  - PolitiScan-specific hardening items (audit S16-S26 gaps) —
    ~300 LOC
- **LOC total** : ~1500 (vs 1200 initial, +300 Couche 3 LOC)
- **Tests delta** : +50
- **Dependencies** : S22 Sybil base (Couches 1+2), S23 Couche 3
  design finalisé, S25-S26 Couche 3 implem partielle, S26 Tor
  complete
- **Gate unlock** : Gate 3 (PolitiScan, NEXUS cold-case) debloqué
  post-audit externe S29.

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
  - **Warrant canary Niveau 1 enforcement** (consumer of S20
    Phase E.2 + E.5 federation foundations) — recruit 3+
    cross-juridiction maintainers, distribute K=2/N=3 FROST
    shares per `WARRANT_CANARY_HARDENING.md §FROST DKG procedure`,
    wire `AttestationProvider` impl to TEE H100 quote backend ;
    flips warrant canary from Niveau 0 (single-key, this-machine
    trust root) to Niveau 1 (threshold-distributed,
    TEE-attested) — ~600 LOC + ops runbook
- **LOC total** : ~2100
- **Tests delta** : +35
- **Dependencies** : S28 MIG, S29 audit, S20 Phase E.2/E.5
  primitives (CanarySigner trait, FrostCanarySigner, Attestation
  Provider trait, NoopAttestation impl)
- **Gate unlock** : Gate 4 eligibility partielle (prerequisites
  complets ; release reel requiert S31+ partnership + beta ferme
  18 mois)

**Total S18-30** : ~22700 LOC, ~660 tests delta, 13 sprints.

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
| **Gate 3** (PolitiScan, NEXUS cold-case) | T0-T3 + partial T4 | **S29** (tech S27 + audit externe S29) | +Tor transport (S26) +redundancy voting (S22) +client-side redaction (S21) +RAG sanitization (S25) +reliable-worker curator (S26) +Sybil mature (S27) +audit externe Cure53/ToB publie (S29) |
| **Gate 4** (LibanLive, war-crime doc) | T0-T5 | **~S35-38** | +Nym mixnet (S28-30+) +TEE H100 (S30+) +MIG (S28) +audit externe comprehensive (S29) +partenariat Amnesty/HRW/CPJ sign-off +18 mois beta ferme + ethics review board + formation OpSec contributeurs |

**Gate 3 effectif = fin S29** : S27 livre la suite technique
(Sybil mature, watermark, PolitiScan-specific hardening) mais Gate
3 operationnel requiert l'audit externe Cure53/ToB publie avec
remediation incluse (Sprint 29).

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
