<!--
written: 2026-02-15  # Sprint 17 Phase D
last_validated: 2026-04-26  # G2 — Sprint 28 Phase D : 12 triggers re-scanned (all INACTIVE, no S28 impact — openai-agents-python autonomous guardrails, arti pre-1.0, iroh 0.97 stable). S28 scope: watermark end-to-end wiring (Phase A) + platform writers journald/oslog + ONNX CI fixture dette (Phase B) + PROCESS_ARCHITECTURE.md design doc (Phase C) + EXTERNAL_AUDIT_SCOPE.md + HARDENING_ROADMAP update (Phase D). Nym deferred S30+ (SDK beta 200-800ms). MIG deferred post-v1.0 (A100/H100 only). Compteurs ~828 Rust / ~195 SDK / ~391+36f+6s coord / ~46 gov / ~268 Vitest / ~1813 total.
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
  - "openai-agents-python release > 0.7.0 (Agent/Runner/Guardrail API breaking, impacte B1 guardrails refactor S23 + A1 hooks S24 + C2 SDK decorator S25)"
  - "MCP spec revision Anthropic 2026+ (impacte B2 mcp_server_expose S25 + D5 capability toggle)"
  - "microsoft/sudo elevation mode release beyond Windows 11 24H2 inbox (impacte D1 trust tiers + D3 Windows RPC S28 + D4 biometric LT-4)"
audited_findings:
  - "2026-04-16 S19 deep analysis : D2 Hashcash daté vs Equi-X 2023, S20 keyring crate, S21 grammar ≠ prompt injection defense, S25 Arti 2.0 Feb 2026 disponible, S26+ PQC trop tardif (HNDL liability)"
  - "2026-04-16 S20 open : double-layer encryption at rest (Argon2id 64MiB + AES-256-GCM + OS keyring wrap KEK) requis — DPAPI user-scope gap confirmé Sygnia 2024 + SpecterOps 2026 (same-user process malicious = full bypass). Duress PIN pattern = fake keypair noop responses (pas wipe immédiat GrapheneOS-style), panic wipe = 5-tap gesture séparé. Structured output via llguidance (Rust Microsoft 50µs/token, llama.cpp -DLLAMA_LLGUIDANCE=ON) retenu over XGrammar (pas llama.cpp), Outlines (Python IPC), GBNF native (slower). PoW wire scope S20 Phase C (reclassé carry S19 A-2). TLS wire iroh T20 tech debt long-terme (iroh 0.97 ClientBuilder hook cfg-test only). DHT canary enforcement strict reporté post-Gate-2."
  - "2026-04-18 S20 Phase E pivot Option C deep-evolution (G8 codification, cf. .planning/active/sprint20_phase_E_pivot_proposal.md + sprint20_phase_E_preflight.md) : warrant canary auto-publish scheduler (plan §8.1 item 1 original) supprimé sur scan S2 — décision threat-model S18 E2 04c9621 toujours valide (cle Ed25519 accessible auto = compromission GHA = compromission cle = dead-man-switch cassé sous gag order). Federation foundations livrées : CanarySigner trait abstraction + FrostCanarySigner K-of-N (RFC 9591 jan 2025, ZF crate 2.1, ToB 2023 audit), DuressAck channel (gossip topic distinct, daily granularité), AttestationProvider trait + NoopAttestation (decouple signing != attestation), Federated CanaryRegistry coord-side (POST /api/canary/observed + GET /api/canary/network-health). E.6 ajusté inline post-G8 S1 finding : iroh 0.91 a supprimé l'option TCP raw → relays = WSS TCP 443 unique mode automatic, transport_probe.rs dégradé en diagnostic-only (probe 3x UDP QUIC + log warn + metric degraded_mode). Wire format CanarySigned v1 + DOMAIN_WARRANT_CANARY_V1 préservés (FROST sig = Ed25519 RFC 8032 byte-identical). Niveau 1 enforcement (cross-juridiction recruitment + TEE H100) added to §3 S30 line."
  - "2026-04-19 S21 CLOSED : 5 phases A-E livrées sur le thème rate-limit + PII SDK defense-in-depth + output filter + quarantine queue + tech debt batch. Phase A `63afe4e` rate-limit governor 0.10.2 GCRA worker-engine R1 (axum 0.7→0.8 bump prereq workspace-wide `5e67ce0` post-G8 pivot Option C). Phase B `d5b0035` PII SDK iframe (onnxruntime-web 1.24.3 + GLiNER PII edge ONNX). Phase C `23abb11` PII coord (presidio-analyzer 2.2.362 + GLiNERRecognizer extra [gliner] même modèle ONNX SoT + InvisibleText scanner curated + EED Levenshtein 0.85). Phase D `f830579` quarantine queue SQLite WAL + Typer CLI (réalignement coord-Python G8 SCOPE-CUT-CONSISTENT `a82e8db`). Phase E `49f0d32` tech debt batch — T-NN canary_wire_bytes JCS canonical (RFC 8785) + T-NN+1 CanaryRegistry verify Ed25519 at ingest via nexus_core.verify_canary PyO3 binding (path-dep nexus-shell-daemon-core ajoutée à nexus-core-py) + plan docs S20 §6 wire-point fix C-PLAN-1 + PATTERNS.md §P34 closeout (T-NN résolu + T-NN+1 résolu + T-NN+2 ouvert S22+ blocked tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen). Premier sprint avec G8 systématique 5/5 phases : 1 DESIGN-CONFLICT (Phase A axum bump) + 4 SCOPE-CUT-CONSISTENT (B/C/D/E). Cap G7 carry-overs respecté 2/2 → S22 : Meta-1 Radicle-v1.0 re-carry + T-NN+2 PATTERNS hors cap formel. Compteurs finals : 659 Rust / 185 SDK / 249+3 coord / 46 gov / 256 Vitest / 38 Playwright / ~1436 tests (+65 vs baseline 1371). Carries S22 audit_plan : P2-E-DURESS-ACK verify_duress_ack hors-scope explicit + P2-E-WIRE-PRE-LAUNCH-FIX check maturin develop --release fresh dans bootstrap §7 + P3-E-2 align build_canary serde_json → JCS pour cohérence + Meta-track hook coverage gap Phase D sans review.md + Phase A R1 rate_limit_policy.toml.sample manquant + Phase B drift Playwright PII end-to-end."
  - "2026-04-18 S21 open : D2 PII SDK requalifié post-research G2. Libellé roadmap §3 S21 original S17 'spaCy NER wasm ~500 LOC' obsolète 2026 (spaCy pas de port wasm officiel maintenu). Stack retenue defense-in-depth : client iframe = onnxruntime-web 1.24.3 (Microsoft, npm mars 2026) + @huggingface/transformers v4 tokenizer + knowledgator/gliner-pii-edge-v1.0 (Apache-2.0, 2024-01-29, F1 0.755, backbone à confirmer Phase B G8 S1 scan pre-first-line-of-code) + regex fallback curated ; coord-side = presidio-analyzer 2.2.362 (Microsoft MIT, 2026-03-15) + GLiNERRecognizer extra [gliner] + même modèle ONNX source-of-truth unique. Full Rust-first iframe (tract + GLiNER + wasm-bindgen) rejeté factuellement : tract 0.22.1 teste opset 9-18 vs GLiNER export opset 19 (DisentangledSelfAttention DeBERTa-v3 non documenté), tract wasm32-unknown-unknown (browser) non documenté officiellement (seul wasm32-wasi wasmtime), zero precedent production, gline-rs v1.0.1 (Rust GLiNER mainstream 01/2026) a choisi ort pas tract. Rust-wasm iframe realignement Option G reporté S22+ via tech debt T-NN+2 (re-evaluate triggers: tract opset 19 coverage OR ort wasm32-browser stable OR gline-rs wasm-bindgen target). Decisions D1 governor 0.10.2 GCRA + D3 LLM Guard 0.3.16 InvisibleText + PLeak EED + D4 SQLite WAL pattern S19 reuse + CLI sbfb quarantine. Cf. sprint21_kickoff.md §D1-D5 + sprint21_design_review.md."
  - "2026-04-20 S22 hors-sprint agents_sudo integration : analyse deep openai-agents-python + microsoft/sudo → 18 features produit identifiées (4 agents parallèles independants cluster A observability / B guardrails / C SDK+streaming / D process+OS integration). Mapping factuel S22 Phase F → S29 + LT-4 tracé dans `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`. S22 Phase F absorption : D1 three-mode trade-off doc (`docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` 3 tiers AUTO/CONFIRM_PROMPT/BIOMETRIC_GATE + consent.json threat_note field). S23 amendement : B1 guardrails refactor pipeline déclaratif (6 primitives S16-S22 → contrat Guardrail unifié `docs/security/GUARDRAILS_ARCHITECTURE.md`) + D5 design `docs/security/CAPABILITY_TOGGLES.md` capabilities gate-off-by-default via binaire `nexus-admin`. S24 amendement : A1 `TaskDispatchHooks` + C3 handoffs semantic dispatcher. S25 amendement : D5-implem + A3 OS audit channel ETW/journald/oslog + B2 MCP server expose + C2 `@task_handler` SDK + C5 streaming bridge (5 features = FAT, split recommandé). S26 amendement : C1 SQLiteSession abstraction crate + A4 process role tagging. S28 amendement : D2 broker/executor split + D3 Windows RPC + C4 task-scoped sandbox (cohérence runtime isolation). S29 amendement : A2 TraceProvider OTEL backend-agnostic + B4 per-mode residual risk doc THREAT_MODEL §9 pre-audit Cure53/ToB. LT-4 net-new : D4 OS biometric gate cross-platform post-v1.0 (trigger v1.0 + S30 FROST N1 + partnership OpSec signal). Cap G7 bilan : 0 slot carry-over formel consommé pre-v1.0 (tout via amendements §3 + items net-new + chore hors-sprint + Phase F absorption). Pre-launch protocol respectée (nouveaux DOMAIN_TRACE_EVENT_V1 + DOMAIN_OS_AUDIT_EVENT_V1 + capabilities.toml + bridge.schema.json extension P24 additif = design-only pre-launch stable, zéro bump *_VERSION). Arbitrages user différés kickoff S23 + S25 : B1 timing (dédié/distribué/défer), S25 split (D5+B2 priorité vs A3+C2+C5 carry), D5 enforcement (Semgrep strict vs CI manuel)."
  - "2026-04-25 S27 Phase D : SynthID-inspired PRF z-test watermark output remplace Kirchenbauer KGW (BIRA-resistant, arXiv:2509.23019 sept 2025). Couche 3 mature : ForgeParser git-log offline GPG/SSH + TrustCache SQLite LRU 7j + TrustWebManager cross-forge score + DelegationCert v1 étendu trust_level + trust-web seed FlowUP bootstrap (ONG S28). P2 batch S26 7 fixes (validate_stage_guard_map wire + emit_capability_event logger + TaskHandlerDescriptor description + JsonFileWriter rotation + TracingWriter rename + MCP lifespan comment + no-LOC convention). Gate 3 prerequisites checklist mise à jour : Alexandria showcase reframing. SELF_DISTRIBUTION.md design doc (binaries = blobs P2P). ~821 Rust / ~195 SDK / ~419+14 coord / ~46 gov / ~264 Vitest / ~41+2 PW / ~1797 total."
  - "2026-04-26 S28 Phase D : sprint consolidation. Phase A watermark end-to-end wiring (compute_bias llama_cpp.rs + output_token_ids runtime.rs) + P2 batch S27 4 items. Phase B platform writers réels JournaldWriter libsystemd + OsLogWriter oslog (cfg-gated) + ONNX CI fixture mini-model GLiNER (dette sprint pair). Phase C PROCESS_ARCHITECTURE.md design doc broker/executor split 11 sections (IPC JSON-RPC 2.0, pool mode, cold-start <5s, fault isolation). Phase D EXTERNAL_AUDIT_SCOPE.md (7 crypto + 6 wire + auth + transport + sandbox, vendor matrix Cure53/ToB) + HARDENING_ROADMAP update (Nym S30+, MIG post-v1.0). Nym deferred S30+ (SDK beta 200-800ms, VALIDATED_BLUEPRINT CAUTION). MIG deferred post-v1.0 (A100/H100 only, RTX 5080 no MIG). ~828 Rust / ~195 SDK / ~391+36f+6s coord / ~46 gov / ~268 Vitest / ~1813 total."
  - "2026-04-20 S22 CLOSED (Phase F wrap-up) : 5 phases A-E livrées sur le thème Sybil-resistance composition 3 couches + rate-limit engine wire + GLiNER span-decoder + NVML baseline + watermark canari primitive + process fixes. Phase A `0bc499f` rate-limit engine wire-up runtime.rs ClaimEntry gate + Arc swap hot-reload + policy sample (absorbe P2-S21-1/2/6 + P3-S21-4). Phase B `e9530c2` GLiNER span-logits decoder iframe SDK (absorbe P2-S21-3, `web/src/sdk/pii/decoder.ts` decodeSpans + greedyDedup + toFinding). Phase C `cf3918c` Sybil-resistance composition 3 couches : Couche 1 `AgeWitness` peer-attestation Ed25519 domain `DOMAIN_AGE_WITNESS_V1` + bootstrap allowlist `BootstrapAllowlistWatcher` hot-reload (P0-G1-1 ack) + `join_topic_with_age_witness` gossip admission ≥7j ; Couche 2 `ContributorAttestation` in-toto v1.0 predicate `nexus-grid/contributor-attestation/v1` + `ContributorRegistry` coord-side SQLite + `curator::verify_with_contributor_registry` + daemon proxy + Matthew-effect TODO inline LT-1 Kudos-v2 commitment (P0-G1-2 + P2-G1-3 acks) ; Couche 3 RFC design-only `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` multi-forge cross-validate + `DelegationCert` + trust-web Amnesty S27 integration. Phase D `56211f2` NVML util+duree profile log-only baseline foundation S24 (`nvml-wrapper 0.12.1` workspace-pinned + `crates/nexus-worker-core/src/gpu/profile.rs` + SQLite `nvml_samples` + `NvmlWindowStats` stats-only pas anomaly). Phase E `690fab3` watermark canari-input primitive consumer 1/N (`canary_input.py` ~520 LOC + `CanaryInputSet` Ed25519 signé coord rotatable + `CanaryInputInjector` hook pre-dispatch + `CanaryInputObserver` rapidfuzz Levenshtein similarity + Typer CLI `canary rotate/status` + hot-reload pattern output_filter S21). Phase F `<HEAD>` wrap-up + verification + audit plan S23 + process fixes P2-S21-4 `docs/claude/README.md §4.4` règle parse phase_[A-F]_review.md vers audit_plan Track + P2-S21-5 GHA `.github/workflows/phase-review-cross-check.yml` PR check + `.claude/.bypass_audit_trail.log` append-only + migration PARA active→archive/v1.2/. **Deuxième sprint avec G8 systématique 6/6 phases A-F (0 DESIGN-CONFLICT déclenché — G1 pre-gel post-S21 robuste)**. Cap G7 carry-overs respecté 1/2 → S23 : T-NN+2 iframe Rust-wasm Option G PATTERNS §P34 hors cap formel. LT-2 Meta-1 Radicle-v1.0 **reclassification sortie cap G7** régularisée kickoff §4 D5 (trigger unique tag v1.0 go-live, runbook `docs/release/MIRROR_FALLBACK.md §3`). LT-3 Contribution family Sybil matrix + LT-4 OS biometric gate ouverts hors-sprint (post-v1.0). Compteurs finals : 710 Rust / 185 SDK / 263+3 skipped coord / 46 gov / 264 Vitest / 38 Playwright / 7/7 size / 246+ SPDX (~1509 tests, +73 vs baseline 1436). Over-delivery +30 vs projection §11 documentée verification.md §4 (tests d'infrastructure bonus cross-couches Phase A + integration tests Phase C + bonus Phase E helpers). Wire formats nouveaux pre-launch stable : `AGE_WITNESS_VERSION = 1` + `CONTRIBUTOR_ATTESTATION_VERSION = 1` + `DELEGATION_CERT_VERSION = 1` (design-only Couche 3). Aucun tolerant decoder multi-version. Pas de nouvelle zone rouge. Carries S23 audit_plan : P2-S22A-1 dashmap dep unused post-refacto (worker-core Cargo.toml cleanup) + P2-S22A-3 PATTERNS.md §P33 structure obsolète (post-wire update) + P2-B-1 ONNX end-to-end non exercé CI (fixture model mini dédiée carry) + P2-B-2 wrapper.ts:308-311 fallbackDetect trigger 0 entities (sémantique explicit) + P2-E-1 `_reload_policy_locked` suffix trompeur (naming convention) + P2-E-2 pattern LOC estimations prospectives plans (3 occurrences S22, chore planning README §6.7 amend) + P3-E-1 `/api/canary/observed-divergence` expose expected_answer (carry S23 B1 alerting) + Meta-track Playwright PII end-to-end carry S23 Track B fixture model."
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
  - **agents_sudo D1 three-mode trust tiers doc** (absorbé Phase F,
    cluster D feature D1 — cf.
    `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`) :
    nouveau `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
    (~150 LOC docs design-only) + extension `consent.json` field
    `level_threat_note` + `residual_threats_acknowledged` (spec
    only, implem runtime S22 Phase F — pur doc). Zéro code Rust/
    Python S22 (Phase F absorption doc-only).
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
  - **agents_sudo B1 guardrails refactor pipeline déclaratif**
    (cluster B feature B1 — cf.
    `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`
    + design doc `docs/security/GUARDRAILS_ARCHITECTURE.md` écrit
    S22 hors-sprint 2026-04-20) : trait/ABC `Guardrail` +
    `GuardrailChain` Python + PyO3 Rust binding + retrofit 6
    primitives (`pii_redactor` S21C + `output_filter` S21D +
    `quarantine_queue` S21D + `rate_limit` S22A + `pii_iframe`
    S21B/S22B + `canary_input` S22E) vers contrat unifié avec
    exceptions typées `InputTripwire / OutputTripwire`. Bridge P24
    whitelist extend méthode `guardrails_check` iframe-side.
    **~800 LOC refactor + ~200 tests contract = ~1000 LOC**.
    **Arbitrage user kickoff S23** : 3 options viables (S23
    dédié avec split phase / S24-S25 distribué / défer S27
    post-Sybil mature).
  - **agents_sudo D5 capabilities toggle design doc hors-sprint**
    (cluster D feature D5 design — cf.
    `docs/security/CAPABILITY_TOGGLES.md` écrit S22 hors-sprint
    2026-04-20, implementation S25) : capabilities gate-off-by-
    default pattern microsoft/sudo + `capabilities.toml` schema +
    binaire `nexus-admin` Typer CLI spec + Semgrep PR-block rule
    spec. **Zéro code S23 sur cet item** (design-only pré-requis
    S25 tool-calling réactivation).
- **LOC total** : ~2220 initial + ~1000 B1 guardrails refactor =
  **~3220 LOC si B1 S23 dédié** (dépasse norme ~2500 LOC,
  arbitrage user kickoff S23 requis). OU **~2220 LOC si B1
  distribué S24-S25** (cohérent norme).
- **Tests delta** : +70 (+65 initial + 5 observability endpoint) +
  ~200 contract tests B1 = **+270 si B1 S23 dédié**.
- **Dependencies** : S22 Sybil base (Couches 1+2), S22 Phase A
  (rate-limit wire-up `0bc499f` pour pattern kudos ledger query).
  **B1 deps** : S22 Phase E watermark canary livré (dernière
  primitive à intégrer au refactor).
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
  - **agents_sudo A1 `TaskDispatchHooks` API** (cluster A feature
    A1 — cf.
    `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`) :
    trait Rust + PyO3 binding exposant 5 events typés
    (on_claim_broadcast / on_task_dispatched / on_result_received
    / on_validator_post_task / on_quarantine_enqueue) injectable
    `Dispatcher.__init__(hooks=[...])`. Consumer natif = re-run
    sampling ci-dessus (anomaly reader = hook on_result_received.
    divergence_score). Design-ready (cluster A research).
    **~400 LOC + ~30 tests**.
  - **agents_sudo C3 handoffs semantic dispatcher** (cluster C
    feature C3) : refactor `dispatcher.py` avec trait `Handoff`
    (on_handoff callback + input_filter re-redact PII policy-
    target + is_enabled skip low-reputation/Sybil-reject). Design
    doc long-life `docs/shell/DISPATCHER_HANDOFFS.md` préalable
    Phase A (pattern S22 Phase C preflight design doc). Dep
    rate-limit S22A ✓, Couches 1+2 S22C, redundancy voting S23.
    **~700 LOC + ~40 tests**.
- **LOC total** : ~1400 initial + 1100 (A1 + C3) = **~2500 LOC**
  (dans norme)
- **Tests delta** : +50 initial + 70 (A1 + C3) = **+120**
- **Dependencies** : **S23 redundancy voting** (pour seuil
  detection — dep update 2026-04-19 post co-defer S22→S23, cf.
  `sprint22_carry_summary.md §4`). **A1/C3 deps** : S22 Phase A
  rate-limit wire ✓, S22 Phase C Couches 1+2, S23 redundancy
  voting.
- **Gate unlock** : —
- **Post-delivery S24** (2026-04-21) : 5 phases A-E livrées. B1
  guardrails pipeline Guardrail ABC + GuardrailChain + 4 adapters
  retrofit coord-side. A1 TaskDispatchHooks 5 events + HookRunner
  fire-and-forget. Re-run sampling 1-5% + DivergenceScorer BLAKE3
  mismatch → quarantine. DNS fallback DoH+DoT via hickory-resolver
  0.24 + browse_aggregator fallback chain. Domain fronting design
  doc outline (implem S25+). Key rotation + C3 handoffs deferred
  S25. +58 tests (757 Rust / 315+3 coord / ~1621 total).

### Sprint 25 — fondations securitaires pre-tool-calling

> **Note realisme** (mise a jour 2026-04-24) : cette section sert
> de **backlog prioritise**, pas de plan sprint — le kickoff de
> chaque sprint arbitre le scope reel par objectif fonctionnel
> (pas par budget LOC). Le drift entre prescription et livraison
> est audite en Phase 0 via la track HARDENING drift (P2
> informatif, cf. `docs/claude/README.md §2.4`).

- **Goal** : poser les fondations securitaires pre-tool-calling (key
  rotation Ed25519, guardrails multi-stage, capability gates).
- **Prescrit originalement** (S17) : Tor transport phase 1 + per-app
  quota + RAG + pluggable transports + D5 capabilities + A3 OS audit
  + B2 MCP + C2 SDK + C5 streaming bridge (~3700 LOC, 9 features).
- **Livre effectivement** (2026-04-22) :
  - D5 capabilities gate-off-by-default (CapabilitiesStore, nexus-admin
    CLI, @require_capability, .semgrep/capability_gate.yml) — **seul
    item prescrit livre**
  - Key rotation ceremony Ed25519 self-signed + gossip revocation list
    (carry S24 D5)
  - C3 handoffs StageGuardrailMap multi-stage guardrail pipeline (carry
    S24 D5)
  - P2 batch DNS concurrent fallback + quarantine alerting (cleanup
    audit S24)
- **Tests delta** : +92 (790 Rust / 372+5 coord / ~1712 total)
- **LOC code** : ~2508 (dans norme)
- **Scope-cut S26+** : Tor phase 1, B2 MCP, A3 OS audit, C2 SDK, C5
  streaming, RAG, per-app rate, pluggable transports

- **Goal prescrit original** : commencer Tor integration Gate 3 prep.
- **Items** :
  - Tor SOCKS proxy wiring via Arti standalone subprocess (iroh
    relay HTTPS fallback over SOCKS5, **NOT** QUIC direct)
  - Per-app rate budget global coordinator-side
  - RAG sanitization pipeline (detox injection sources externes)
  - Pluggable transports : **wire bridge config + lyrebird
    subprocess** (Tor Project Go binary upstream)
  - **agents_sudo D5 capabilities implem** (cluster D feature D5,
    design `docs/security/CAPABILITY_TOGGLES.md` écrit S22
    hors-sprint) : binaire `nexus-admin` Typer CLI + `capabilities.
    toml` + check admin privilege cross-OS (EUID Unix / `IsUserAn
    Admin()` + Mandatory Integrity Level High Windows) + Semgrep
    custom rule `.semgrep/capability_gate.yml` PR-block. **Pré-
    requis** pour tool-calling réactivation dans RAG pipeline.
    **~400 LOC + ~30 tests**.
  - **agents_sudo A3 OS audit channel `nexus-events-core`**
    (cluster A feature A3) : crate Rust writers platform-native
    (tracing-etw Windows, sd_journal_send Linux, os_log macOS) +
    12 event types enum (consent_change, panic_fired,
    token_rotation, duress_unlock, quarantine_drop, sybil_
    admission_reject, pow_verify_fail, canary_published,
    canary_dead_mans_switch_tripped, transport_degraded,
    rate_limit_tier_breach, nvml_anomaly_detected,
    capability_changed). Dep A1 S24 (event enum typée).
    **~500 LOC + ~50 tests**.
  - **agents_sudo B2 MCP server exposition** (cluster B feature
    B2) : bridge S13 expose MCP server stdio/HTTP transport local-
    only, 3 tools whitelist (task_submit/storage_get/storage_set)
    avec JSON schema strict. Design doc long-life
    `docs/security/MCP_BRIDGE_ARCHITECTURE.md` + threat model
    adversary AD6 "MCP consumer malveillant". Gate
    `capability.mcp_server_expose` (D5). Dep B1 guardrails S23 +
    B3 Pydantic auto-derivation (S22F ou S23). **~600 LOC + ~40
    tests**.
  - **agents_sudo C2 `@task_handler` SDK auto-schema** (cluster C
    feature C2) : decorator `packages/nexus-sdk/src/nexus_sdk/
    decorators.py` introspecte signature Pydantic → auto-génère
    `task_request.schema.json` + `task_response.schema.json` +
    manifest auto-export `GET /app/<name>/manifest`. Dep B3
    Pydantic auto-derivation (S22F ou S23). **~300 LOC + ~30
    tests**.
  - **agents_sudo C5 streaming bridge events** (cluster C feature
    C5) : nouvelle méthode whitelist P24 `task_submit_streaming`
    + wire format formal `bridge.schema.json` discriminated union
    (token/tool_called/pii_masked/done/error). Playwright 3-
    browser matrix (Chromium+Firefox+WebKit). SDK client
    `web/public/sbfb-bridge.js` `onTaskStream(correlation_id,
    onEvent)`. Gate `capability.streaming_bridge` (D5). Dep S20
    Phase D structured output ✓, S21 Phase B PII iframe ✓ + S22
    Phase B decoder ✓. **~700 LOC + ~50 tests**.
- **Tests delta** : +55 initial + 200 (5 features) = **+255**
- **LOC total** : ~1200 initial + ~2500 (5 features) = **~3700
  LOC** — **FAT, dépasse norme ~50%**. Arbitrage user kickoff
  S25 requis : priorisation {D5 + B2} (tool-calling unlock) vs
  carry {A3 + C2 + C5} S26-S27.
- **Dependencies** : S18 multi-relai, S24 domain fronting legal,
  **B1 guardrails S23** (pour B2 MCP), **A1 hooks S24** (pour A3
  events typés), **B3 Pydantic auto** (pour B2 + C2, S22F ou S23).
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

> **Note realisme** (mise a jour 2026-04-22 kickoff S26) : cette
> section prescrit ~5300 LOC. S26 a arbitre : Tor bloque (arti
> pre-1.0), GPU/curator deferes S27. Scope retenu = B2 MCP server
> local-only + A3 OS audit SecurityEvent + C2 @task_handler SDK +
> P2 batch S25 (~1640 LOC). Les items prescrits non retenus restent
> dans le backlog ci-dessous pour S27+. P2-D-1 + P2-E-1-iroh
> reclassifies long-term (LT-5/LT-6 ROADMAP_COMMITMENTS.md).

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
  - **agents_sudo A4 process role tagging** (cluster A feature A4,
    amendement item GPU lockup ligne précédente — prérequis
    structurel cgroups/Job Object) : crate `nexus-role-core` avec
    `ProcessRole` enum (Launcher/Daemon/Worker/OllamaRuntime/
    IframeBlobServe) + `RoleTagger` trait platform-abstraction
    (Windows Job Object named per-role + Process Mitigation
    Policy, Linux cgroup v2 + Landlock LSM ruleset per-role,
    macOS launchd label + sandbox-exec profile). Role attribué
    au spawn via env var `NEXUS_PROCESS_ROLE` HMAC-signée (anti-
    spoof). **~400 LOC + ~30 tests**.
  - **agents_sudo C1 SQLiteSession abstraction** (cluster C
    feature C1, thème principal S26) : extraire crate Rust
    `nexus-session-store` + PyO3 binding exposant trait
    `Session<T>` uniforme avec `get_items / add_items / pop_item
    / clear_session`. Migration 5 stores coord existants
    (`quarantine_queue`, `kudos`, `canary_registry`,
    `contributor_registry` S22C, `upload_queue` S19) vers
    adapters `aiosqlite`. CLI unifiée `sbfb session <module>
    list/pop/clear` (remplace 5 CLI Typer séparées). **~800 LOC +
    ~80 tests**.
- **Tests delta** : +60 initial + 110 (A4 + C1) = **+170**
- **LOC total** : ~800 initial + 1200 (A4 + C1) = **~2000 LOC**
  (dans norme).
- **Dependencies** : S25 Tor phase 1, S24 domain fronting legal,
  S10 curator infra. **C1 deps** : S22 Phase C `contributor_
  registry` stabilisé (5e store cristallisé avant abstraction).
- **Gate unlock** : —
- **Carry-overs S24/S25** (research deep avril 2026) : Domain
  fronting implem differe S24→S26 car majors (Google/AWS/Cloudflare/
  Fastly) ont ferme 2018-2024, seuls Snowflake-WebRTC + CDN
  minoritaires + ECH restent — demandent legal partnership prealable.
  Arti library-embed differe S25→S26 sur condition API stable.

### Sprint 27 — Watermark SynthID output + Couche 3 mature multi-forge + Gate 3 showcase docs

> **Post-delivery S27** (2026-04-25) : 4 phases A-D livrées. Phase A
> P2 batch S26 audit 7 fixes. Phase B WatermarkDetector coord-side
> z-test PRF HMAC-SHA256 + WatermarkInjector llama.cpp logit bias
> opt-in (SynthID-inspired, Kirchenbauer KGW rejeté BIRA
> arXiv:2509.23019). Phase C Couche 3 mature : ForgeParser git-log
> --show-signature offline (GPG RFC 4880 + SSH RFC 8709) +
> TrustCache SQLite LRU 7j WAL + TrustWebManager cross-forge score
> (forge_count x tenure x delegation_depth decay -1/hop) +
> DelegationCert v1 étendu (trust_level 1-5 + DelegationScope) +
> trust-web seed FlowUP bootstrap (ONG S28 outreach) + gossip topic
> nexus-grid/trust-web/v1 + spec DelegationCert dans
> CONTRIBUTOR_ATTESTATION_RFC.md §3. Phase D Gate 3 showcase docs +
> SELF_DISTRIBUTION.md design doc. +19 tests (7 watermark + 9 Couche
> 3 + 3 P2 batch). ~821 Rust / ~1797 total.

- **Goal** : Gate 3 suite technique — watermark output SynthID-
  inspired + Couche 3 Sybil-resistance mature + Gate 3 showcase
  docs.
- **Items** :
  - Watermark output SynthID-inspired : WatermarkDetector coord-
    side (z-test binomial PRF HMAC-SHA256, BIRA-resistant) +
    WatermarkInjector worker-side llama.cpp logit bias +2.0 opt-in
    (`watermark.toml`). Kirchenbauer KGW rejeté (BIRA vulnérable
    arXiv:2509.23019 sept 2025).
  - **Couche 3 mature (multi-forge cross-validate + trust-web
    ONG bootstrap)** — ForgeParser Rust git-log --show-signature
    offline (GPG+SSH), TrustCache SQLite LRU 7j WAL, TrustWeb-
    Manager cross-forge scoring, DelegationCert v1 étendu
    (trust_level + scope + valid_until), trust-web seed config
    FlowUP bootstrap (ONG réelles S28 outreach).
  - P2 batch S26 audit 7 fixes (Phase A) : validate_stage_guard_map
    wire, emit_capability_event logger, TaskHandlerDescriptor
    description, JsonFileWriter rotation 10 MiB, TracingWriter
    rename, MCP lifespan comment, no-LOC convention.
  - Gate 3 showcase docs : HARDENING_ROADMAP update SynthID,
    COMPUTE_THREATS update, Gate 3 prerequisites checklist,
    PATTERNS.md P37-P38, SELF_DISTRIBUTION.md design doc.
- **Tests delta** : +19 (7 watermark + 9 Couche 3 + 3 P2 batch)
- **Dependencies** : S22 Sybil base (Couches 1+2), S23 Couche 3
  design finalisé
- **Gate unlock** : Gate 3 (Alexandria, showcase apps) débloqué
  post-audit externe S29.

### Sprint 28 — Watermark end-to-end + dette + process isolation design + audit prep

> **Post-delivery S28** (2026-04-26) : sprint consolidation 4 phases
> A-D. Scope redimensionné post-G9 factual kickoff : Nym deferred
> S30+ (SDK beta 200-800ms, VALIDATED_BLUEPRINT CAUTION), MIG deferred
> post-v1.0 (A100/H100 enterprise only, RTX 5080 dev = no MIG).
> D2/D3/C4 broker/executor = design-only (PROCESS_ARCHITECTURE.md),
> code S29. Sprint pair → phase dette obligatoire (§6.2.1 Règle 1).

- **Goal** : consolider livrables S27 (watermark end-to-end wiring),
  resoudre dette accumulee (platform writers, ONNX), preparer terrain
  S29 (process isolation design doc, audit externe scope doc).
- **Items livres** :
  - Phase A `c5f35f7` : watermark end-to-end wiring (`compute_bias`
    dans `llama_cpp.rs` sampling loop + `output_token_ids` populate
    dans `runtime.rs`) + P2 batch S27 audit 4 items (P2-B-1 injection
    wire, P2-B-2 `watermark.toml.sample`, P2-C-1 trust_web_seeds.toml
    fingerprint reel, P2-D-1 PATTERNS P37 path fix)
  - Phase B `a43a1a1` : platform writers reels (`JournaldWriter`
    `libsystemd` FFI + `OsLogWriter` `oslog` crate, cfg-gated) +
    ONNX CI fixture mini-model PII GLiNER Vitest (phase dette sprint
    pair)
  - Phase C `ccbb6ca` : `docs/security/PROCESS_ARCHITECTURE.md` design
    doc (~350 LOC, 11 sections) — broker/executor split, IPC JSON-RPC
    2.0 UDS/Named Pipe, pool mode N=1 default, cold-start budget <5s,
    fault isolation backoff exp. Prior art OSS BOINC/Golem/Ollama
    APPROACH-ALIGNED
  - Phase D : `docs/security/EXTERNAL_AUDIT_SCOPE.md` — scope audit
    externe (7 crypto primitives, 6 wire formats, auth loopback,
    transport iroh, sandbox iframe, vendor matrix Cure53/ToB) +
    HARDENING_ROADMAP §3 S28 update + Nym/MIG deferrals documentes
- **Nym mixnet** : **G9 2026-04-25 — SDK beta (`nym-sdk 1.27.0`,
  VALIDATED_BLUEPRINT rate CAUTION), latences 200-800ms, zero
  fondation code codebase, pas de transport SOCKS abstraction dans
  iroh 0.97. Deferred S30+ post-Gate 3.**
- **MIG partitioning** : **G9 2026-04-25 — MIG = feature A100/H100/
  H200 enterprise datacenter (NVIDIA MIG User Guide). RTX 5080 =
  consumer GPU, pas de MIG. Deferred post-v1.0 quand workers
  enterprise H100 disponibles.**
- **D2 broker/executor code** : deferred S29 (design doc prerequis
  PROCESS_ARCHITECTURE.md livre S28 Phase C, cold-start benchmark
  Ollama 7B requis pre-commit, co-landing C4 task-scoped sandbox)
- **D3 Windows RPC** : deferred S29 (co-landing D2 + `windows-rs`
  crate maturite)
- **C4 task-scoped sandbox** : deferred S29 (co-landing D2)
- **Tests delta** : +16 (7 watermark Phase A + 5 platform writers
  Phase B + 4 ONNX fixture Phase B)
- **Dependencies** : S27 Sybil mature, S27 watermark primitives
- **Gate unlock** : —

### Sprint 29 — External audit + remediation buffer

> **Note realisme S29-S30** (mise a jour 2026-04-26 kickoff S29) :
> l'audit externe est un engagement de 4-8 semaines avec un tiers
> (Trail of Bits ou Cure53). S29 prepare le package audit (process
> isolation code, TraceProvider, THREAT_MODEL §9, SECURITY.md,
> BUILDING.md, scope freeze) et livre les prerequis techniques.
> L'engagement vendor demarre fin S29 ou S30 selon timeline
> disponibilite. La remediation sera absorbee dans le sprint
> post-audit (S30 ou S31). Le budget ~1500 LOC remediation est une
> estimation — il sera affine quand les findings arriveront.

- **Goal** : audit externe paid Cure53 ou Trail of Bits +
  remediation.
- **Items** :
  - Audit execution (~50-100k$ budget, 4-8 semaines)
  - Remediation findings (buffer ~1500 LOC estime)
  - Public disclosure responsible-disclosure policy +
    security.txt — ~200 LOC
  - **agents_sudo A2 TraceProvider backend-agnostic** (cluster A
    feature A2, pré-audit Cure53/ToB obligation discipline tracing
    formelle) : `TraceProvider` trait unifié Rust (crate
    `nexus-trace-core`) + Python sibling via PyO3. Backends :
    `BatchLogProcessor` JSON structured → file default ;
    `OtelProcessor` OTLP/gRPC Grafana Tempo / Jaeger via
    `opentelemetry 0.27` ; `SignedCanaryProcessor` Ed25519-signed
    trace events (`DOMAIN_TRACE_EVENT_V1` nouveau domain design-
    only pre-launch stable) — réutilise `nexus-core-py::sign_bytes`
    pattern S14/S20 Phase E. W3C Trace Context propagation
    (`traceparent` header) cross-process Rust↔Python via HTTP
    loopback + gossip trace baggage. API `set_trace_processors()
    / add_trace_processor()` symétrique openai-agents-python.
    Consumer : A1 hooks S24 events typés + A3 nexus-events-core
    S25 comme TraceProcessor. **~600 LOC + ~40 tests**.
  - **agents_sudo B4 per-mode residual risk doc THREAT_MODEL §9**
    (cluster B feature B4, pré-audit Cure53/ToB input) : refactor
    `docs/security/THREAT_MODEL.md` ajout §9 "Residual risks
    per-configuration" sous-sections par feature configurable :
    9.1 consent GPU 4 niveaux S16C, 9.2 loopback 3 trust tiers
    S22F/S25/LT-4, 9.3 duress PIN S20B, 9.4 rate-limit tiers
    S22A, 9.5 pipeline guardrails disabled combos (B1 S23), 9.6
    capability toggles (D5 S25). Annotations in-product
    `consent.json` field `residual_threats_acknowledged` +
    `level_threat_note` (livrées S22 Phase F D1 design). UI
    launcher `web/src/components/GpuConsentDialog.tsx` affiche
    `level_threat_note` tooltip. **~200 LOC docs + ~100 LOC
    frontend + ~20 LOC backend**.
- **LOC total** : ~1700 initial + 920 (A2 + B4) = **~2620 LOC**
  (dans norme).
- **Tests delta** : +50 initial + 60 (A2 + B4) = **+110**
- **Dependencies** : S28 scope doc. **A2 deps** : A1 hooks S24,
  A3 events S25. **B4 deps** : tous les modes livrés S16-S28
  (consent GPU, loopback tiers, duress, rate-limit, guardrails
  B1, capabilities D5).
- **Gate unlock** : —

### Sprint 30 — Nym mixnet phase 1 + TEE H100 eval + split inference research

- **Goal** : Gate 4 eligibility partielle. Nym phase 1 (carry S28)
  + TEE attestation big-rock pour Gate 4 complet.
- **Items** :
  - **Nym mixnet integration phase 1** (carry S28 — deferred
    2026-04-25 post-G9 factual) : SOCKS5 wrapper iroh relay
    over Nym, test feasibility latence vs UX. Conditionnel
    `nym-sdk` sortie beta stable (trigger VALIDATED_BLUEPRINT
    rate CAUTION 2026-04-25). Si SDK toujours beta au kickoff
    S30, re-defer S32+. — ~1500 LOC
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
- **Dependencies** : S29 audit, S20 Phase E.2/E.5
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

S27 Sybil mature + S28 audit scope doc ──> S29 external audit
S30 Nym mixnet phase 1 (carry S28) ─────> S30+ Nym prod (Gate 4)
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
| **Gate 3** (Alexandria, showcase apps) | T0-T3 + partial T4 | **S29** (tech S27 + audit externe S29) | +watermark output SynthID (S27) +Couche 3 multi-forge trust-web (S27) +redundancy voting (S23) +client-side redaction (S21) +Sybil 3 couches mature (S22+S27) +audit externe Cure53/ToB publié (S29). Tor transport déféré post-Gate 3 (arti pre-1.0). Alexandria = première app showcase (stockage distribué + MCP tools, pas de GPU requis). |
| **Gate 4** (LibanLive, war-crime doc) | T0-T5 | **~S35-38** | +Nym mixnet (S28-30+) +TEE H100 (S30+) +MIG (S28) +audit externe comprehensive (S29) +partenariat Amnesty/HRW/CPJ sign-off +18 mois beta ferme + ethics review board + formation OpSec contributeurs |

**Gate 3 effectif = fin S29** : S27 livre la suite technique
(Sybil mature Couche 3, watermark SynthID output, Gate 3 showcase
docs) mais Gate 3 opérationnel requiert l'audit externe Cure53/ToB
publié avec remédiation incluse (Sprint 29). **Reframing S27** :
Gate 3 showcase apps = Alexandria (bibliothèque de connaissance
multilingue, stockage distribué + MCP tools, pas de GPU requis),
Surveillance forêt, D&D P2P. Remplace l'ancien "PolitiScan, NEXUS
cold-case" (cf. `docs/apps/LAUNCH_SHOWCASE.md`).

**Items Gate 3 livrés S22-S28** :
- Couche 1 AgeWitness ≥7j (S22 Phase C)
- Couche 2 ContributorAttestation in-toto v1.0 (S22 Phase C)
- Couche 3 ForgeParser + TrustCache + TrustWebManager (S27 Phase C)
- Watermark canary-input (S22 Phase E)
- Watermark output SynthID-inspired (S27 Phase B)
- Watermark end-to-end wiring llama_cpp.rs + runtime.rs (S28 Phase A)
- Rate-limit GCRA per-(consumer, worker, model) (S21 Phase A / S22 Phase A)
- Escalating PoW géométrique (S23 Phase C)
- Redundancy voting 3-worker majority (S23 Phase D)
- Ephemeral workers restart + VRAM wipe (S23 Phase B)
- Guardrails pipeline ABC + GuardrailChain (S24 Phase B)
- Capabilities gate-off-by-default (S25 Phase A)
- Key rotation Ed25519 + gossip revocation (S25 Phase B)
- OS audit SecurityEvent ETW/journald/oslog (S26 Phase B + S28 Phase B platform writers)
- MCP server local-only (S26 Phase A)
- Process isolation design doc PROCESS_ARCHITECTURE.md (S28 Phase C)
- External audit scope doc EXTERNAL_AUDIT_SCOPE.md (S28 Phase D)

**Items Gate 3 restants** :
- Audit externe Cure53/ToB (S29) ��� ship-blocker
- THREAT_MODEL §9 per-mode residual risk doc (S29 Phase B4)
- Tor transport phase 1 (S30+, conditionnel arti ≥ 1.0)

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
