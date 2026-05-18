# Planning System, Sprint History, Roadmap & Decisions

**Analysis Date:** 2026-05-18

---

## 1. Project Timeline & Evolution

### 1.1 Origin and Pivot

The project started as **NEXUS cold-case** (political investigation tool). On **2026-04-10**, it pivoted to **SBFB (nexus-grid)** — a decentralized P2P compute network for apps. The old NEXUS cold-case code was deleted in Sprint 51 but remains in git history for reference.

The pre-pivot roadmaps still live in `.planning/`:
- `NEXUS_GOV_ROADMAP.md` — political transparency tool vision (obsolete)
- `DISTRIBUTED_GPU_ROADMAP.md` — GPU sharing architecture (pre-pivot concept, partially absorbed)
- `OPEN_SOURCE_ROADMAP.md` — open source strategy (pre-pivot)

These files are **not actively maintained** but remain as historical artifacts.

### 1.2 Version Milestones

| Version | Sprints | Theme | Status |
|---------|---------|-------|--------|
| **v1.0** | S0-S13 | Pivot SBFB, P2P iroh, universal render, bridge postMessage, launcher | CLOSED |
| **v1.1** | S14-S15 | Verified deploy (Keyoxide + SLSA L1), bridge bidirectionnel, CPU watchdog | CLOSED |
| **v1.2** | S16-S60 | Security hardening, Gate 2 prerequisites, Python-to-Rust migration, self-hosted build, installer, tag v1.0 | CLOSED — tag v1.0 |
| **v2.0** | S61-S64+ | Public Verifiable Protocol Feed (6-sprint roadmap) | OPEN |

Archive locations:
- `v1.0`: `.planning/archive/v1.0/` (49 files, S3-S13)
- `v1.1`: `.planning/archive/v1.1/` (10 files, S14-S15)
- `v1.2`: `.planning/archive/v1.2/` (500+ files, S16-S60)
- `v2.0`: `.planning/archive/v2.0/` (S61-S63 archived)
- Active: `.planning/active/` (S64 files + S65 audit plan)

---

## 2. Sprint-by-Sprint History

### 2.1 v1.0 Era (S0-S13) — Foundation

| Sprint | Key Deliverable |
|--------|----------------|
| **S0** | Initial stabilization, `stabilize/compute` branch merged |
| **S1** | Rust workspace setup, iroh 0.97 integration, PyO3 bindings |
| **S2** | Core architecture, retrospective audit established |
| **S3** | 12 commits (W1-W12), first structured verification |
| **S4** | First with kickoff + plan + verification docs |
| **S5** | Monolithic kickoff (950 lines) — last before kickoff/plan split |
| **S6** | **First with 4 planning docs** + audit gate pattern invented |
| **S7** | **First complete audit gate cycle** (permanent from here) |
| **S8-S9** | Conditional PASS patterns established |
| **S10** | **First ops sprint** (CI/CD + 3 VPS bootstrap, no app code) |
| **S11** | **First P2P end-to-end** (publish + discovery + render) |
| **S12** | **First universal render** (archive zip -> daemon blob-serve -> iframe sandbox) |
| **S13** | **First bridge iframe <-> reseau** (postMessage + open source enforcement + launcher) |

Test count at v1.0 close: ~908 tests.

### 2.2 v1.1 Era (S14-S15) — Verified Deploy

| Sprint | Key Deliverable |
|--------|----------------|
| **S14** | **First verified-from-source deploy.** Coordinator clones repo, verifies SBFB.json (Keyoxide Ed25519), zips content, signs `provenance.json` (SLSA L1). Multi-forge support. |
| **S15** | Bridge bidirectional (host -> iframe push). CPU watchdog heartbeat. CLI `sbfb init`. ~934 tests. |

### 2.3 v1.2 Era (S16-S60) — Security Hardening & Rust Migration

This is the longest era (45 sprints). Major sub-themes:

**Security Hardening (S16-S22)**:
- **S16**: Loopback auth (bearer + Host + Origin + peer creds)
- **S17**: **Pure research sprint** (0 code, ~4823 LOC docs). Threat model taxonomy T0-T5, 12 attack scenarios, GPU compute threats, hardening roadmap S18-S30, VALIDATED_BLUEPRINT 13 layers. Established "research before code" pattern.
- **S18**: Supply chain CI (cargo-deny + pip-audit + npm audit), reproducible builds, multi-relay federation, NVIDIA driver CVE check, warrant canary Ed25519
- **S19**: Transport hardening — DHT quorum wire, PoW Hashcash, TLS cert pinning, delayed upload queue, pkarr relay self-hosted
- **S20**: Gate 2 prerequisites — encryption at rest (Argon2id + AES-256-GCM + OS keyring), duress PIN/panic wipe, PoW runtime wire, structured output dual-backend, FROST canary federation. **First G8 DESIGN-CONFLICT detected and resolved.**
- **S21**: Rate-limit + PII defense-in-depth. First sprint with G8 systematic 5/5 phases.
- **S22**: Sybil-resistance 3 layers (AgeWitness, ContributorAttestation, trust-web RFC)

**Advanced Security & Architecture (S23-S32)**:
- **S23-S25**: Ephemeral workers, key rotation, delegation certs, MCP server, OS audit channels
- **S26-S27**: SynthID watermark, multi-forge trust web, FROST DKG
- **S28**: Warrant canary Niveau 1, blob-serve COOP/COEP, EXTERNAL_AUDIT_SCOPE
- **S29**: Task runner real execution + output filter E2E + Tor transport Phase 1
- **S30**: iroh 0.98 upgrade, rusqlite 0.36, arti-client activation
- **S31-S32**: Gossip resilience, bridge extensions, iroh neighborhood enrichment (LT-6 RESOLVED)

**Python-to-Rust Migration (S33-S51)**:
- **S33-S34**: Multi-node research, route namespace migration
- **S35-S36**: Migration Rust native Phase 1-2 (validator loop, output filter, guardrails)
- **S37-S41**: Tier 1-4 module migration (PiiRedactor, CanaryRegistry, CanaryInput, fairness, pow_counter, contributor registry, invite ledger, capability store, quarantine, upload queue)
- **S42-S46**: Tier 5 routes API (deploy, apps, tasks, health, shell, kudos, diagnostic, consent, files, canary, contributor, invite, quarantine)
- **S47-S49**: Carry resolution batches, coordinator lifecycle -> daemon Rust
- **S50-S51**: **Complete Python suppression** — 4 packages deleted (~30,853 LOC + ~505 tests), monolith nexus/ deleted (188 files, -72,335 LOC)

**Pre-v1.0 Readiness (S52-S60)**:
- **S52**: CI Woodpecker + self-hosted build design
- **S53**: P2P smoke test multi-platform (Win/Mac/VPS Helsinki). P2P validated cross-machine LAN+WAN.
- **S54**: Rust edition 2024 upgrade, E2E wire tasks_doc_ticket
- **S55**: CI self-hosted build, LT-7 Tier 1+2 (build executor, quorum SHA256 validation)
- **S56**: Gossip resilience (persistent outbox, browse rate-limit), bridge extensions (9 methods total)
- **S57**: Protocol Explorer MVP + Ideas Hub MVP (first 2 SBFB apps)
- **S58**: AppStorage P2P replication iroh-docs (live sync between nodes)
- **S59**: Launcher readiness, verified deploy E2E, **LT-1 Kudos-v2 CLOSED** (9 sprint carry), storage validation
- **S60**: **Installer NSIS + tray icon + LT-7 Tier 3 validation + tag v1.0.** Windows NSIS, Linux .deb, macOS .dmg. LT-7 P2P infra validated (gossip 3 machines WAN). **End user ready.**

Test count at v1.0 tag: 1259 Rust / 258 Vitest / 6/6 size-limit.

### 2.4 v2.0 Era (S61-S64) — Public Verifiable Protocol Feed

| Sprint | Theme | Key Deliverable | Tests Delta |
|--------|-------|----------------|-------------|
| **S61** | Spec executable + feed local | PUBLIC_FEED_SPEC.md, PublicFeedOperation enum, FeedStore SQLite M9, hash-chain BLAKE3, FeedMaterializer, verify_chain Ed25519 | +23 Rust |
| **S62** | Sync P2P durable + anti-spam | iroh-docs transport, offline catch-up, multi-daemon E2E, PoW + rate-limit + quarantine hot path feed | +17 Rust +7 Vitest |
| **S63** | Verification tiers + UX | Provenance endpoint HTTP, bridge verification methods, VerificationDetail modal, Protocol Explorer verification section | +6 Rust +7 Vitest |
| **S64** | **Hardening public cible** | MANDATORY F1 version stored + F5 timeout/retry, dette pair 5 items, 6 adversarial feed tests, 4 adversarial crypto tests, E2E new node, PUBLIC_FEED_SPEC.md finalized | +21 Rust |

Current test count: **1326 Rust / 265 Vitest / 6/6 size-limit (~1597 total)**.

---

## 3. Planning System Architecture

### 3.1 PARA Pattern

The `.planning/` directory follows the **PARA pattern** (Projects / Areas / Resources / Archives):

```
.planning/
├── README.md              # Layout explanation
├── active/                # ONE sprint at a time (current S64)
├── archive/               # Closed sprints grouped by version
│   ├── v1.0/              # S0-13
│   ├── v1.1/              # S14-15
│   ├── v1.2/              # S16-60
│   └── v2.0/              # S61-63
├── codebase/              # Cross-sprint codebase cartography
├── research/              # Cross-sprint research notes (50+ docs)
├── DISTRIBUTED_GPU_ROADMAP.md  # Evergreen thematic roadmap
├── NEXUS_GOV_ROADMAP.md        # Evergreen thematic roadmap
└── OPEN_SOURCE_ROADMAP.md      # Evergreen thematic roadmap
```

**Source of truth**: `.planning/README.md`

### 3.2 Sprint Lifecycle

Each sprint follows a strict 6-step cycle documented in `docs/claude/README.md`:

1. **Phase 0 — Audit gate** (blocking): Fresh Claude session audits previous sprint via `sprint{N-1}_audit_plan.md`. Produces `sprint{N-1}_audit_findings.md`. P0/P1 must be fixed before Phase A.

2. **Kickoff** (`sprint{N}_kickoff.md`): 12 canonical sections including Day 0 frozen decisions (D1..D5), phase outline, carry items, scope cuts, risk register, checkpoint questions.

3. **Plan** (`sprint{N}_plan.md`): 9 canonical sections with per-phase detail (scope, files touched, test plan, acceptance criteria, commit template), fail-fast checklist (24-32 rows), git plan.

4. **Phases A..E/F** (code): One atomic commit per phase. Each phase has:
   - G8 pre-flight (5 factual scans) -> `sprint{N}_phase_{X}_preflight.md`
   - Code implementation
   - Phase review -> `sprint{N}_phase_{X}_review.md`
   - Commit `feat(scope): Sprint N Phase X — titre`

5. **Verification** (`sprint{N}_verification.md`): Self-report fail-fast checklist with "Observed" column filled. 9 canonical sections.

6. **Audit Plan** (`sprint{N}_audit_plan.md`): Plan for the next sprint's Phase 0 audit. 7 canonical sections with tracks A..I.

### 3.3 Documents Per Sprint

| Document | When Written | By Whom |
|----------|-------------|---------|
| `sprint{N}_kickoff.md` | Entry | Sprint N session |
| `sprint{N}_plan.md` | Entry | Sprint N session |
| `sprint{N}_design_review.md` | Pre-Phase A (G1) | Independent agent |
| `sprint{N}_phase_{X}_preflight.md` | Pre-code each phase (G8) | Sprint N session |
| `sprint{N}_phase_{X}_review.md` | Post-code each phase | Phase auditor |
| `sprint{N}_verification.md` | Exit | Sprint N session |
| `sprint{N}_audit_plan.md` | Exit | Sprint N session |
| `sprint{N-1}_audit_findings.md` | Phase 0 | Fresh session (Sprint N) |
| `sprint{N+1}_carry_summary.md` | Phase F wrap-up (G7) | Sprint N session |

### 3.4 Gate System (G1-G9)

Nine procedural gates protect sprint quality. Documented in `docs/claude/README.md` sections 6.1.1 through 6.10.

| Gate | Tag | When | What |
|------|-----|------|------|
| **G1** | `[DETER]` | Kickoff, after draft D1..D5 | Design Review Board scoring (independent agent) |
| **G2** | `[DETECT]` | Session-start | Re-validation of `triggers_revalidate` on long-life docs |
| **G3** | `[STRUCT]` | Kickoff §2 | Goal SMART -> verification.md fail-fast checklist |
| **G4** | `[DETECT]` | Phase review + audit gate | Rigor signal: 0 P0/P1 + >=1 P2+ for PASS |
| **G5** | — | Suppressed S24 | Working tree audit (replaced by hook lightcheck) |
| **G6** | `[STRUCT]` | Post-commit + Phase F | Memory update + carry-over |
| **G7** | `[STRUCT]` | Phase F carry generation | Escalation at 3 reports + debt phase even sprints |
| **G8** | `[DETECT]` | Pre-implementation phase | 5 factual scans (S1a OSS prior art + S1b deps + S2 history + S3 threat + S4 wire) |
| **G9** | `[DETER]` | Before D-decision draft | Factual research gate |

### 3.5 Audit Gate Verdicts

- **PASS**: 0 P0, 0 P1 -> Sprint N+1 Phase A starts directly
- **CONDITIONAL PASS**: 1-3 P1 fixable -> Phase A blocked until `fix(sprint{N}):` commits land
- **FAIL**: >= 1 P0 or >= 3 P1 -> Partial redesign required

### 3.6 G8 Pre-flight Verdicts

- **EXECUTE**: All scans clean, proceed with plan as-is
- **PLAN-ADAPT**: S1a finding (OSS prior art shows better approach), adapt inline, no user stop
- **SCOPE-CUT-CONSISTENT**: Non-blocking finding, proceed + carry S+1
- **DESIGN-CONFLICT**: Blocking structural conflict, STOP, emit pivot_proposal, user arbitrage

---

## 4. Carry-Over & Debt Management

### 4.1 Rules (G7, §6.2.1)

Three rules govern technical debt (amendment 2026-04-24):

1. **Rule 1 — Debt phase every other sprint**: Even sprints (S28, S30, S32...) reserve one phase exclusively for deferred items. Non-negotiable.

2. **Rule 2 — Automatic escalation at 3 reports**: An item deferred 3 consecutive sprints becomes MANDATORY. Must enter the plan as a phase, not as carry. No reclassification to "long-term" for items < 500 LOC.

3. **Rule 3 — Check ROADMAP_COMMITMENTS at kickoff**: Evaluate trigger conditions for each LT-* item. If condition met, item becomes active carry.

### 4.2 Current Carry Items (S64 -> S65)

**MANDATORY (3/3)**:
- **P2-FEED-INSERT-NO-AUTH-TIER**: `feed_insert` handler must verify auth tier before insert

**External Exemptions (permanent)**:
- **P2-A-1**: rand blocker (waiting rand 0.9 upstream)
- **P2-AUDIT-2**: iroh pre-release transitives (waiting iroh 1.0 upgrade)

**Monitoring**:
- **P2-G-1**: exe lock intermittent (needs 3x reproduction)

**Counter 2/3** (next report = MANDATORY):
- P2-PROVENANCE-404-BRIDGE, P2-COMMIT-TITLE-FORMAT, P2-REVIEW-ORDER, P2-PYTHON-BLOCK-EXEMPTION, P2-EXPLORER-ESCAPE-SINGLE-QUOTE, P2-PLAYWRIGHT-SPECS-STALE, P2-VERIFY-LOCAL-KEY-ONLY, P2-COVERAGE-DEPLOY-E2E

**Counter 1/3**:
- P2-FEED-JOIN-HANDLE-LEAK, P2-VERIFY-ENTRY-VERSION-GUARD, P2-ORPHAN-REPUBLISH-RECOVERY

### 4.3 Long-Term Commitments (ROADMAP_COMMITMENTS.md)

File: `docs/release/ROADMAP_COMMITMENTS.md`

| ID | Title | Status | Trigger |
|----|-------|--------|---------|
| **LT-1** | Kudos-v2 fairness reform | **CLOSED S59** | Pre-v1.0 reclassification |
| **LT-2** | Radicle mirror flip | **trigger PENDING** | Tag v1.0 pushed to origin |
| **LT-3** | Contribution family Sybil matrix | latent | Post-v1.0 |
| **LT-4** | OS biometric gate cross-platform | latent | Post-v1.0 |
| **LT-5** | Redundancy persistence SQLite | latent | Post-v1.0 |
| **LT-6** | iroh neighborhood enrichment | **RESOLVED S32** | — |
| **LT-7** | Self-hosted build | **gate satisfied** | Tier 1+2 S55, Tier 3 S60 |

---

## 5. Roadmap: Public Verifiable Protocol Feed (v2.0)

### 5.1 Overview

**Decision**: PO 2026-05-13. 6 sprints (5+1 reserve) for public verifiable protocol credibility.
**Source**: `.planning/research/public_verifiable_feed_roadmap.md`

### 5.2 Sprint Plan

| Sprint | Theme | Status | Key Deliverables |
|--------|-------|--------|-----------------|
| **S61** (Sprint 1) | Spec executable + feed local rejouable | **DONE** | PUBLIC_FEED_SPEC.md, PublicFeedOperation types, FeedStore SQLite, hash-chain BLAKE3, FeedMaterializer, cursor |
| **S62** (Sprint 2) | Sync P2P durable + anti-spam minimal | **DONE** | iroh-docs transport, offline catch-up, multi-daemon E2E, PoW + rate-limit + quarantine on feed hot path |
| **S63** (Sprint 3) | Verification tiers + UX | **DONE** | HTTP provenance endpoint, bridge verification, VerificationDetail modal, Protocol Explorer demo |
| **S64** (Sprint 4) | Hardening public cible | **DONE** | Adversarial tests (10 feed + 4 crypto), new node E2E, PUBLIC_FEED_SPEC finalized |
| **S65** (Sprint 5) | Go-live public | **NEXT** | Release pipeline, evidence pack, external pilot (2-3 testers), mirror fallback, tag push |
| **S66** (Sprint 6) | Reserve — hardening post-pilote | **PLANNED** | Pilot fixes, anti-spam reinforcement, optional RRV, audit prep |

### 5.3 Gate de Scission

Sprint 2 had a scission gate: if offline catch-up + replay idempotent + 2/3 nodes + anti-spam hot path were NOT all proven at Phase C review, the sprint would split and the plan would go to 7 sprints. **Gate was passed** — no scission needed.

### 5.4 What the Roadmap Does NOT Cover

- External interop (alternative clients, third-party parsers)
- Formal third-party audit (only prep + RFP)
- p2panda private spaces (post-v1.0)
- SearchManifest/RRV complete (optional Sprint 6)
- Tor transport (arti, post-plan)
- Runtime isolation VM (post-plan)
- AppStorage P2P multi-app (dehardcode sbfb-ideas)

---

## 6. Day 0 Frozen Decisions

These architectural decisions are **not re-debatable**. Documented in `CLAUDE.md` and enforced by G1/G8 gates.

| Decision | Rationale | Source |
|----------|-----------|--------|
| P2P integral pivot, Option G hybrid Rust+Python (now pure Rust post-S51) | Decentralization is the product, not a feature | `nexus_grid_pivot.md` |
| iroh 0.98 pinned | Upgrade = dedicated sprint (S32 done 0.97->0.98). iroh 1.0.0-rc.0 exists but deferred. | S32, HARDENING_ROADMAP triggers |
| Zero central moderation | Curator lists Ed25519 + gossip + blobs | Day 0 |
| Kudos = non-monetary reputation score | No cost/deposit/stake/burn/refund. LT-1 v2 CLOSED S59. | `feedback_kudos_non_monetary.md` |
| Singleton strict shell daemon | One daemon per machine | Day 0 |
| AGPL-3.0 license maintained | Copyleft for P2P network integrity | Day 0 |
| Archive zip = universal format | daemon blob-serve = render, shell = iframe host agnostic | S12 |
| postMessage bridge = only iframe <-> network channel | 3 methods whitelist (expanded to 9 S56) | S13 |
| Deploy verified from source | Keyoxide Ed25519 + SLSA L1 provenance, multi-forge, zero OAuth | S14 |
| Launcher Rust minimal | Not Tauri. Browser = client. | S13 |
| Pre-launch protocol policy | `*_VERSION` stays at 1 until first v1.0 tag. No tolerant decoder multi-version. | CLAUDE.md §Pre-launch |
| No funding / No foundation / No startup | OpenBSD solo maintainer pattern | `vision_model.md` |
| Curated app updates only | No auto-update. Each version validated by curator quorum. | `curated_updates.md` |

---

## 7. Red Zones (Zones Rouges)

Active security concerns tracked in `docs/security/VALIDATED_BLUEPRINT.md`:

| Zone | Risk | Status |
|------|------|--------|
| **R-iroh-audit** | P0 — iroh crate has no formal security audit. Using pinned 0.98. | OPEN — monitoring |
| **R-wasmtime-cve** | P0 — wasmtime had 12 CVEs April 2026. Runtime isolation roadmap in `RUNTIME_ISOLATION.md`. | OPEN — pin 43.0.1+ |
| **R-libcrux-hax** | P2 — Symbolic Software semantic gaps (7 April 2026) in hax verification toolchain for libcrux | OPEN — monitoring |
| **R-pyodide-escape** | P2 — Pyodide iframe escape risk for Python-based apps | OPEN — sandbox `allow-scripts` without `allow-same-origin` |

---

## 8. Research Archive

50+ research documents in `.planning/research/`. Key categories:

### 8.1 Security Research (consumed by S17-S32)
- `S19_phase_B_pow_hashcash_design.md` — PoW design decisions
- `S19_phase_C_tls_cert_pinning_design.md` — TLS pinning approach
- `S20_phase_B_duress_panic_design.md` — Duress PIN threat model
- `S20_phase_D_structured_output_design.md` — LLM output enforcement
- `S21_phase_B_iframe_pii_sdk_design.md` — PII redaction architecture
- `S21_phase_C_output_filter_design.md` — Output safety pipeline
- `S21_phase_D_quarantine_design.md` — Quarantine queue design

### 8.2 Architecture Research
- `ARCHITECTURE.md` — Codebase architecture snapshot
- `REACTIVE_ARCHITECTURE.md` — Reactive patterns analysis
- `S49_coordinator_rust_migration.md` — Python-to-Rust migration strategy
- `sprint33_multinode_research.md` — Multi-node P2P research

### 8.3 Protocol & Feed
- `public_verifiable_feed_roadmap.md` — **Active roadmap** for v2.0 (6 sprints)
- `p2panda_public_protocol_briques.md` — p2panda protocol building blocks
- `community_code_validation_p2p.md` — P2P code validation patterns
- `vote_triggered_task_dispatch.md` — Vote-based task dispatch design

### 8.4 Product Vision
- `babel_translation_protocol.md` — Babel universal library post-v1.0 app concept
- `sbfb_cross_domain_use_cases.md` — Cross-domain use cases
- `sbfb_rrv_code_factory_vision_pitch.md` — RRV code factory vision
- `pre_v1_apps_protocol_explorer_ideas_hub.md` — Pre-v1.0 app strategy

### 8.5 Frontend & UX
- `frontend_vision_v2.md` — Frontend vision
- `frontend_inspiration_catalog.md` — Design inspiration
- `AWWWARDS_DESIGN_RESEARCH.md` — Design research
- `frontend_ux_protocol_analysis.md` — UX protocol analysis

### 8.6 Day 0 Open Questions
- `DAY0_OPEN_QUESTIONS.md` — Aggregated Day-0 parameter candidates for S20-S25 (29 open questions with defaults and trade-offs)

---

## 9. Security Documentation Landscape

Extensive security documentation in `docs/security/`:

| Document | Purpose |
|----------|---------|
| `THREAT_MODEL.md` | STRIDE/LINDDUN baseline threat model |
| `HARDENING_ROADMAP.md` | Sprint 18-30 sequenced hardening plan with threat x mitigation matrix |
| `VALIDATED_BLUEPRINT.md` | 13-layer design validated against 2026 OSS state-of-art (50+ components) |
| `ADVERSARIES.md` | Adversary tiers T0-T5 taxonomy |
| `ATTACK_SCENARIOS.md` | 12 concrete attack scenarios |
| `P2P_THREATS.md` | 7 P2P network attack vectors |
| `COMPUTE_THREATS.md` | 7 GPU compute threat classes |
| `RELEASE_GATES.md` | 4 release gates (DnD forge, early adopter, public, production) |
| `EXTERNAL_AUDIT_SCOPE.md` | Scope for external security audit (7 crypto + 6 wire + auth + transport + sandbox) |
| `RUNTIME_ISOLATION.md` | VM runtime isolation roadmap |
| `DURESS.md` | Duress PIN/panic wipe threat model |
| `WARRANT_CANARY_HARDENING.md` | 4-layer canary hardening (L0-L2 + FROST DKG) |
| `CAPABILITY_TOGGLES.md` | Gate-off-by-default capability design |
| `GUARDRAILS_ARCHITECTURE.md` | Guardrail pipeline architecture |
| `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` | 3 trust tiers (AUTO/CONFIRM/BIOMETRIC) |
| `PROCESS_ARCHITECTURE.md` | Broker/executor split design |
| `DOMAIN_FRONTING_DESIGN.md` | Domain fronting censorship resistance |
| `SPLIT_INFERENCE_DESIGN.md` | Split inference for privacy |
| `CONTRIBUTOR_ATTESTATION_RFC.md` | Multi-forge contributor attestation |
| `CONTRIBUTOR_ATTESTATION_PREDICATE.md` | In-toto predicate specification |

---

## 10. Commit Discipline

### 10.1 Commit Format

```
feat(scope): Sprint N Phase X — titre court

## Contexte
[rationale, threat model, research grounding]

## Fichiers
| Fichier | Role |

## Delta tests
| Suite | Avant | Apres | Delta |

## Verification §7.4
[CI manifest complet]

## Scope cuts respectes (kickoff §8)
[ALL items exhaustive]

## G8 traceability
- Preflight: [SHA] verdict [EXECUTE/PLAN-ADAPT/SCOPE-CUT-CONSISTENT]
- Review: [SHA] verdict [PASS/CONCERN/FAIL]

## Pre-launch protocol
[VERSION unchanged, wire format preserved]

## Carry closure / Unblock

Co-Authored-By: Claude <model> <noreply@anthropic.com>
```

### 10.2 Commit Types

| Type | Usage |
|------|-------|
| `feat(scope): Sprint N Phase X` | Phase delivery |
| `fix(sprint{N}): description` | Post-audit or post-review fix |
| `docs(sprint{N}): kickoff + plan` | Sprint opening planning |
| `docs(sprint{N}): verification + audit plan` | Sprint closing |
| `chore(planning): sprint{N} Phase {X} — G8 preflight` | Pre-flight artifact |
| `fix: description` | Hotfix outside sprint cycle |

### 10.3 Windows Git Bash Rule

For commit bodies > 30 lines: use `git commit -F file.txt`, NOT heredoc. Windows Git Bash fails on French apostrophes and markdown backticks in heredocs. Standard path: `.git/COMMIT_EDITMSG_PHASE_X.txt`.

---

## 11. Memory System

External memory files persist across sessions in:
```
C:\Users\FlowUP\.claude\projects\C--Users-FlowUP-Documents-Code-nexus\memory\
```

### 11.1 Key Memory Files

| File | Purpose |
|------|---------|
| `MEMORY.md` | Index, one line per entry (< 150 chars) |
| `user_profile.md` | FlowUP, francophone dev, RTX 5080 |
| `nexus_grid_pivot.md` | **Project state** — roadmap, tips, test counts, carries |
| `sprint_audit_gate.md` | Audit gate pattern documentation |
| `feedback_approach.md` | No band-aids, product-first, verify scope cuts |
| `vision_model.md` | NO funding/foundation/startup — OpenBSD solo maintainer |
| `feedback_commit_heredoc.md` | Windows heredoc anti-pattern |
| `feedback_cd_web_trap.md` | Never `cd web &&` in chained bash |
| `feedback_background_checks.md` | Background checks for heavy verification |
| `feedback_dual_platform.md` | Dual-platform cargo (Windows + Docker) |

### 11.2 Memory Update Rules

- After each closed sprint: update `nexus_grid_pivot.md` with new tip + test counts
- After each feat commit: update memory tip description
- After user feedback: dedicated feedback file
- Never: code patterns, git log, who-changed-what (already in repo)

---

## 12. Workflow Tooling

Beyond vanilla Claude Code, the project uses 5 quality layers documented in `docs/claude/TOOLING.md`:

1. **Hooks**: `phase-auditor-gate.sh` (audit gate enforcement), `phase-precommit-lightcheck.sh` (staging coherence)
2. **Skills**: `nexus-phase-preflight/SKILL.md` (G8 5 scans), `nexus-phase-review/SKILL.md` (phase review)
3. **Agents**: `nexus-phase-auditor.md` (independent phase review)
4. **Semgrep**: Custom SBFB rules
5. **Trail of Bits**: Security-focused analysis patterns

---

## 13. Key Process Evolution Moments

| Sprint | Process Change | Impact |
|--------|---------------|--------|
| **S6** | Audit gate pattern invented | Independent verification becomes permanent |
| **S7** | First complete audit gate cycle | P0/P1 blocking becomes standard |
| **S16** | PARA pattern migration | `.planning/active/` + `archive/v{X}/` layout |
| **S17** | "Research before code" pattern established | Zero rework on research-first sprints |
| **S19** | G8 invented (DESIGN-CONFLICT detection) | Plan-vs-code drift prevention |
| **S20** | First G8 DESIGN-CONFLICT resolved | Proved G8 catches real issues (auto-publish canary conflict) |
| **S21** | G9 added (factual research gate on D-decisions) | D-drafts require factual research first |
| **S22** | Phase review routing to audit plan (§4.4) | P2/P3 findings no longer orphaned |
| **S24** | G5 suppressed (replaced by hook) | Mechanical enforcement replaces manual check |
| **S24** | G7 amendment (3 rules: debt phase, escalation, ROADMAP_COMMITMENTS) | Systematic debt management |
| **S26** | G1 enforcement via hook (Check 5) | Design review no longer skippable |
| **S54** | Post-plan phase exemption (G8) | Ad hoc phases can skip preflight with P2 documentation |

---

## 14. Patterns & Tech Debt Tracking

### 14.1 Rust Patterns

File: `docs/rust/PATTERNS.md` (2564 lines)

Key patterns documented:
- iroh 0.97/0.98 API specifics (Endpoint, discovery, presets)
- PyO3 0.28 `Bound<'py, T>` migration
- Sprint 1 compile-time lessons (drift plan vs real API)
- Numerous section-specific patterns (P1-P50+) accumulated across sprints

### 14.2 Shell/Coordinator Patterns

File: `docs/shell/PATTERNS.md` (2154 lines)

Key patterns documented:
- P1: Typed coordinator client (Zod schemas, no raw fetch)
- P2: base-ui render prop (not Radix asChild)
- P3: Zustand 5 curried create syntax
- P4: React Query as only cache
- P5: CORS loopback-only
- P6: NEXUS_GRID_ROOT env override for tests
- P7+: File-based integration, and dozens more accumulated patterns
- Tech debt sections (T1-T7+) with tracked items

---

## 15. Test Count Evolution

| Milestone | Rust | Vitest/PW | Python | Total |
|-----------|------|-----------|--------|-------|
| S13 (v1.0 close) | ~200 | ~239/38 | ~385 | ~908 |
| S15 (v1.1 close) | ~220 | ~239/38 | ~395 | ~934 |
| S18 (Gate 1) | 474 | 239/38 | ~421 | ~1172 |
| S22 (Sybil) | 710 | 264/38 | ~449 | ~1509 |
| S32 (gossip) | ~864 | ~269 | ~394 | ~1854 |
| S50 (Python delete) | ~1199 | 248 | 0 | ~1447 |
| S55 (CI build) | 1216 | 250 | 0 | ~1466 |
| S60 (tag v1.0) | 1259 | 258 | 0 | ~1523 |
| S61 (feed spec) | 1282 | 258 | 0 | ~1546 |
| **S64 (current)** | **1326** | **265** | **0** | **~1597** |

Python tests dropped to 0 at S50-S51 when the complete Python codebase was deleted (~30,853 LOC + ~505 tests removed).

---

## 16. Upcoming: Sprint 65 (Go-Live Public)

### 16.1 Expected Theme

Sprint 5 of the 6-sprint v2.0 roadmap. Theme: **Go-live public**.

### 16.2 Expected Phases (from roadmap)

- **Phase A**: Release pipeline — CI workflow on tag, SLSA attestations, signed binaries, tag pushed to origin
- **Phase B**: Evidence pack — EXTERNAL_AUDIT_SCOPE.md scope freeze, THREAT_MODEL + BUILDING.md bundle, security@ PGP key, SECURITY.md finalized
- **Phase C**: External pilot — closed group (2-3 external testers), 3 curators, 2-3 published projects, monitoring
- **Phase D**: Mirrors + fallback — pkarr relay failover, mirror fallback, bootstrap allowlist cleanup

### 16.3 MANDATORY Items

- **P2-FEED-INSERT-NO-AUTH-TIER (3/3)**: feed_insert handler must verify auth tier before insert

### 16.4 Key Carry Items at 2/3

12 items at 2/3 — next report would make them MANDATORY. The S65 kickoff must address each explicitly.

### 16.5 LT-2 Radicle Trigger

Tag v1.0 is posed locally but NOT pushed to origin. If S65 pushes the tag, LT-2 (Radicle mirror flip) trigger activates. Runbook: `docs/release/MIRROR_FALLBACK.md §3`.

---

## 17. Language Policy

| Context | Language |
|---------|----------|
| User responses, planning docs, commit bodies, `docs/claude/` comments | **French** |
| Code, identifiers, commit titles, logs, error strings | **English** |
| `PATTERNS.md` files | Mostly **English** (consumed by agents + future external contributors) |
| `.planning/sprint*_*.md` | **French** |
| User-facing React strings | **French** (enforced by `web/scripts/scan-en-strings.sh`) |

---

*Planning history analysis: 2026-05-18*
