# Sprint 28 — Kickoff (Watermark end-to-end + dette + process isolation design + audit prep)

**Ecrit** : 2026-04-25 (session fraiche post-audit gate S27 `fbc63b3`).
**Type** : **sprint consolidation + design** (fermeture gaps watermark +
dette sprint pair + design docs process isolation + audit prep).
**Tip master d'entree** : `fbc63b3` (chore(planning): sprint 27 audit
gate — findings verdict PASS, 0 P0/P1, 5 P2).
**Phase 0 audit Sprint 27** : **DEJA JOUE** — findings dans
`.planning/active/sprint27_audit_findings.md` (verdict **PASS**,
0 P0/P1, 5 P2, 1 P3).

---

## Sources context7 + WebSearch consultees (pre-gel)

- **G2 trigger scan** (2026-04-25, meme jour que S27 Phase D
  `last_validated: 2026-04-25` — pas de delta trigger depuis) :
  Tous les 12 triggers HARDENING_ROADMAP inactifs ou sans impact
  S28 (confirme par scan S27 Phase D, delta < 12h). Detail :
  `iroh > 0.97` INACTIVE, `wasmtime LTS` INACTIVE (pin 43.0.1+
  couvre), `arti-client > 1.x` INACTIVE, `frost-ed25519 > 2.1`
  INACTIVE, `MCP spec` INACTIVE, `openai-agents-python > 0.7.0`
  ACTIVE mais sans impact S28 (guardrails autonome), `RFC 9591`
  INACTIVE, `microsoft/sudo > 24H2` INACTIVE, `NIST PQC` INACTIVE,
  `NVIDIA H100 CCM` INACTIVE, `Sprint S+2` = S30 entries (Nym
  phase 2 + FROST N1 + TEE) — non-bloquant S28.

- **G9 research codebase (3 agents Explore, 2026-04-25)** :
  - **Nym mixnet feasibility** : zero reference code dans le
    codebase. `VALIDATED_BLUEPRINT.md` rate `nym-sdk 1.27.0` comme
    CAUTION (beta, latences 200-800ms). Aucune abstraction transport
    SOCKS dans iroh 0.97. Scope-cut S30+ recommande.
  - **Process isolation readiness** : `nexus-shell-daemon` = daemon
    monolithique. Bearer auth + UDS/NP (S16) dans le meme processus.
    Aucune separation broker/executor existante. Readiness ~40%.
    PROCESS_ARCHITECTURE.md prereq non ecrit. Cold-start benchmark
    Ollama requis (<5s budget). Design-only S28.
  - **MIG partitioning** : zero reference MIG dans le codebase.
    `nvml-wrapper 0.12.1` expose profiling (S22 Phase D) mais MIG
    est A100/H100 only — **RTX 5080 ne supporte pas MIG**. Scope-cut
    post-v1.0 (quand workers H100 existent).
  - **Technical debt verification** : watermark.rs 119 LOC correct
    et teste (4 tests), zero call site dans llama_cpp.rs, runtime.rs
    `output_token_ids: vec![]` jamais peuple. watermark.toml.sample
    absent. trust_web_seeds.toml fingerprint dummy `000...`. P37
    chemin incorrect post-revert `6eee5ca`. Platform writers
    JournaldWriter + OsLogWriter = stubs avec JsonFileWriter fallback.

- **G9 platform writers** : `nexus-events-core/src/lib.rs` L166-189
  definit les stubs. Trait `EventWriter` en place. Implementations
  cibles : `libsystemd` crate (journald, crate stable >= 0.7,
  `sd_journal_send` FFI) + `oslog` crate (macOS Unified Logging,
  crate stable >= 0.2). Gated `#[cfg(target_os)]`. Windows couvert
  par TracingWriter existant (tracing-etw subscriber layer). Testing
  via trait mock sur Windows dev + CI Linux/macOS futur.

- **HARDENING_ROADMAP S28 original vs arbitrage** : le HARDENING_
  ROADMAP §3 S28 prescrit "Nym mixnet + MIG + audit prep + D2/D3/C4
  process isolation" (~4000 LOC, +45% norme). Apres G9 factual :
  - Nym : beta, deferred S30+
  - MIG : hardware mismatch RTX 5080, deferred post-v1.0
  - D2/D3/C4 process isolation : prereqs design doc non ecrits,
    cold-start benchmark manquant → design-only S28, code S29
  - S28 sprint pair → phase dette obligatoire (§6.2.1 Regle 1)
  - 4 P2 audit S27 a absorber

---

## 1. Constat d'entree

### 1.1 D'ou on part

- **Tip** : `fbc63b3` — S27 DONE + audit PASS (0 P0/P1, 5 P2).
- **Working tree** : propre.
- **v1.2** : continuation security hardening. Pas de nouvelle version.

### 1.2 Ancrage HARDENING_ROADMAP

HARDENING_ROADMAP §3 S28 prescrit : "Nym mixnet + MIG + external audit
prep + D2/D3/C4 process isolation" (~4000 LOC).

**Arbitrage S28** (post-G9 factual) :
1. Nym : SDK beta 200-800ms latence, pas de fondation code, deferred
   S30+. Le HARDENING_ROADMAP sera mis a jour pour refleter.
2. MIG : RTX 5080 ne supporte pas MIG (A100/H100 enterprise only).
   Deferred post-v1.0.
3. D2/D3/C4 process isolation : design doc seulement (PROCESS_
   ARCHITECTURE.md). HARDENING_ROADMAP mandate ce doc "prealable" au
   code. Code → S29 (co-landing D2+C4).
4. External audit scope doc : maintenu, prep S29 Cure53/ToB.
5. P2 batch S27 audit : 4 items en Phase A.
6. Sprint pair → phase dette (platform writers + ONNX CI fixture).

Scope S28 redimensionne : ~400 LOC code + ~800 LOC docs. Sprint de
**consolidation** qui ferme les gaps S27 + prepare le terrain S29
(process isolation + audit externe).

### 1.3 Compteurs tests entree (tip `fbc63b3`)

| Suite | Count | Notes |
|---|---|---|
| Rust nextest | 821 | all pass |
| Rust doctests | pass | |
| Python SDK | 195 | all pass |
| Python coord | 391 pass + 36 fail + 6 skip | 36 fail = stale PyO3 wheel |
| Python gov | 46 | all pass |
| Vitest | 264 | all pass |
| Playwright | ~43 (27p + 16f) | 16 fail = env PyO3 wheel |
| Size-limit | 7/7 | |
| **Total** | **~1802** | (+0 vs sortie S27 — tip audit commit, pas code) |

### 1.4 Pre-launch protocol policy

Pas de deploiement live. `*_FORMAT_VERSION` restent a 1. S28
n'introduit PAS de nouveau wire format P2P gossip. Le watermark
wiring est interne worker-coordinator. Les platform writers sont
locaux (audit trail). Les design docs sont non-code. Aucun
`*_VERSION` bump.

---

## 2. Goal

Consolider les livrables S27 en fermant le gap watermark end-to-end
(injection + detection wiring effectif), resoudre la dette technique
accumulee (platform writers, ONNX, docs), et preparer le terrain S29
(process isolation design doc, audit externe scope doc).

**Critere SMART : 20+ rows fail-fast verts au `verification.md`,
mesure binaire au Phase E wrap-up.**

---

## 3. Phase 0 — Audit gate Sprint 27

**Verdict** : PASS (0 P0/P1, 5 P2, 1 P3).
**Commit** : `fbc63b3` — `sprint27_audit_findings.md` dans
`.planning/active/`.
**P2 carry S28** : 4 items documentes dans §6.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Watermark end-to-end wiring : llama_cpp.rs sampling + runtime output_token_ids

**Retenu** : fermer le gap P2-B-1 identifie par l'audit S27 — wirer
`watermark.rs` (compute_bias, should_inject) dans le sampling loop de
`llama_cpp.rs`, et populer `output_token_ids` dans `runtime.rs` pour
que le z-test coordinator-side ait des donnees. Gate :
`watermark.enabled = true` dans WorkerConfig (desactive par defaut).

§Research consulte : G9 agent Explore 2026-04-25 confirme :
`watermark.rs` L1-119 expose 3 fonctions publiques testees (4 tests
Rust). Zero call site dans `llama_cpp.rs` (grep 0 match). `runtime.rs
:1062` fixe `output_token_ids: vec![]`. Le gap est ~30-50 LOC
d'integration code + population du champ wire existant.

**Rejete** :
- **Ollama backend wiring** : l'API `ollama-rs` n'expose pas de hook
  logit pre-sampling. Pas de mecanisme pour injecter un bias dans la
  generation Ollama. Defer post-ollama-API-hook (si jamais expose).
- **Full SynthID Tournament Sampling** : la modification CDF complete
  (Tournament Sampling) est significativement plus complexe que le
  bias additif PRF. Le bias additif green-only est un compromis
  robuste (deja decide S27 D1, pas de nouveau fait qui l'invalide).
- **Detection-only sans wiring injection** : le z-test sans injection
  est non-concluant (canary-input S22 suffit pour detection-only).
  La valeur est dans le combo injection+detection.

**Implications** :
- Update `crates/nexus-worker-core/src/llm/llama_cpp.rs` : call
  `compute_bias` pre-sampling, applique delta aux logits green
- Update `crates/nexus-worker-core/src/engine/runtime.rs` : populate
  `output_token_ids` dans TaskResult avec les tokens generes
- Bonus : `configs/watermark.toml.sample` (P2-B-2)
- Bonus : `configs/trust_web_seeds.toml` fingerprint reel (P2-C-1)
- Bonus : `docs/rust/PATTERNS.md` P37 path fix (P2-D-1)

### D2 — Platform event writers : journald + oslog implementations reelles

**Retenu** : remplacer les stubs JournaldWriter et OsLogWriter dans
`nexus-events-core` par des implementations reelles. Architecture :

1. **Linux journald** : crate `libsystemd` >= 0.7 (wrapper FFI stable
   autour de `sd_journal_send`). Chaque `SecurityEvent` serialise en
   champs journald structures (MESSAGE, PRIORITY, SBFB_EVENT_TYPE,
   SBFB_DETAILS JSON). Gated `#[cfg(target_os = "linux")]`.

2. **macOS oslog** : crate `oslog` >= 0.2 (wrapper Unified Logging
   `os_log_with_type`). Subsystem `com.sbfb.security`, category
   `events`. Gated `#[cfg(target_os = "macos")]`.

3. **Windows** : deja couvert par TracingWriter (tracing-etw subscriber
   layer S26). Pas de changement.

4. **init_emitter routing** : update la logique singleton pour
   selectionner automatiquement le writer natif selon la plateforme au
   boot.

§Research consulte : G9 agent Explore 2026-04-25 confirme stubs
L166-189 dans `nexus-events-core/src/lib.rs`. Trait `EventWriter`
en place (L69-71). `JsonFileWriter` + `TracingWriter` = 2 impls
live. Les stubs log debug + fallback noop.

**Rejete** :
- **Direct FFI sans crate wrapper** : reimplementation fragile de
  l'interface journald/oslog. Les crates `libsystemd` (maintenu,
  50+ deps sur crates.io) et `oslog` (API stable Apple) sont des
  wrappers matures.
- **Unified tracing-subscriber only** : TracingWriter existe deja et
  couvre la cas cross-platform. Les platform writers natifs apportent
  l'integration avec les outils OS (journalctl, Console.app) que les
  ops attendant. Les deux coexistent.
- **Windows Event Log direct** : `tracing-etw` est l'approche moderne
  Microsoft (structured events ETW). Pas de raison de reimplementer
  via `windows-rs` WriteEventLog.

**Implications** :
- `crates/nexus-events-core/Cargo.toml` : ajout deps optionnelles
  `libsystemd` (target linux) + `oslog` (target macos)
- `crates/nexus-events-core/src/lib.rs` : remplacement stubs
- Tests : trait mock sur Windows dev, integration CI Linux/macOS futur
- **Constraint dev** : RTX 5080 = Windows 11. Les implementations
  journald/oslog compilent (cfg gate) mais ne s'executent pas sur la
  machine dev. Tests fonctionnels via mock EventWriter.

### D3 — Process isolation : PROCESS_ARCHITECTURE.md design-only

**Retenu** : ecrire le design doc prerequis pour le split
broker/executor prescrit par HARDENING_ROADMAP §3 S28. Pas de code ce
sprint — design seulement. Le doc definira :

1. **IPC boundary** : mecanisme inter-process broker↔executor (Unix
   domain socket sur Linux/macOS, Named Pipe sur Windows — reutilise
   la stack S16). Format : JSON-RPC 2.0 sur le canal IPC (simplicite,
   debuggabilite).
2. **Executor lifecycle** : pool d'executors long-lived avec timeout
   inactivite (pattern systemd socket activation). Pas de spawn
   per-task (cold-start Ollama model load > 5s budget).
3. **State ownership** : broker = keypair + gossip subs + bearer auth
   + routing. Executor = model runtime + sampling + GPU access. Pas
   de state partage en memoire.
4. **Cold-start budget** : < 5s pour premier token (benchmark Ollama
   7B sur RTX 5080 a mesurer pre-commit S29 Phase D2).
5. **Fault isolation** : crash executor != crash broker. Broker
   re-spawn executor avec backoff exponentiel.

§Research consulte : G9 agent Explore 2026-04-25 confirme architecture
monolithique actuelle (main.rs → runtime.rs → iroh_runtime.rs). Bearer
auth + UDS/NP dans le meme process. Pas de separation existante.
RUNTIME_ISOLATION.md §3 decrit les phases VM (WSL2/Virtualization.
framework/systemd-nspawn) — le broker/executor est la brique
pre-VM.

**Rejete** :
- **Implementation code S28** : le design doc PROCESS_ARCHITECTURE.md
  est prescrit comme prereq par HARDENING_ROADMAP. Cold-start benchmark
  requis. Co-landing avec C4 (task-scoped sandbox) requis. Trop de
  prereqs non remplis pour coder. S29.
- **Monolithe maintenu** : contradicts HARDENING_ROADMAP §3 S28 D2
  obligatoire. Le design doc est le compromis minimal ce sprint.
- **gRPC/protobuf IPC** : overhead de serialization + code generation
  pour un canal local. JSON-RPC 2.0 est plus simple, debuggable via
  logs, et suffisant pour le volume (<100 req/s).

**Implications** :
- Nouveau `docs/security/PROCESS_ARCHITECTURE.md` (~400 LOC doc)
- Diagrams : flow broker→executor, state ownership, failure modes
- Prerequis S29 Phase D2/C4 implementation

### D4 — External audit : scope document + RFP draft

**Retenu** : ecrire le document de scope pour l'audit externe S29
(Cure53 ou Trail of Bits). Le doc definira :

1. **Critical paths** : crypto primitives (Ed25519, AES-256-GCM,
   Argon2id, FROST), wire formats (canonical JCS, Task/CuratorList/
   CanarySigned), loopback auth (bearer + UDS/NP), gossip transport
   (iroh 0.97).
2. **Non-critical paths** : UI React, docs planning, CI/CD, test
   infrastructure.
3. **Scope matrix** : Cure53 (web/infra focus, $20-50k, 2-4 weeks)
   vs Trail of Bits (crypto/protocol focus, $50-100k, 4-8 weeks).
4. **Budget framework** : fourchette $50-100k, timing S29
   (avril-mai 2026).
5. **Pre-conditions** : PROCESS_ARCHITECTURE.md livre (D3),
   per-mode residual risk doc THREAT_MODEL §9 (S29 B4).

§Research consulte : G9 agent Explore 2026-04-25 confirme zero
reference Cure53/ToB dans le code. HARDENING_ROADMAP §3 S29 prescrit
"Audit execution (~50-100k$ budget, 4-8 semaines)". Ce doc est le
prereq.

**Rejete** :
- **Audit engagement S28** : budget non securise, timeline trop courte
  (S28 = 1 sprint avant S29 execution). Le scope doc est le
  deliverable minimal.
- **Auto-audit interne** : complemente mais ne remplace pas un audit
  independant. Le sprint audit gate (Phase 0) est notre auto-audit.
  L'audit externe apporte des yeux frais sur des dimensions non
  couvertes (crypto side-channels, timing attacks, fuzzing).
- **Skip audit prep** : S29 ne peut pas commencer l'audit sans scope
  doc. Blocker sequentiel.

**Implications** :
- Nouveau `docs/security/EXTERNAL_AUDIT_SCOPE.md` (~200 LOC doc)
- Update HARDENING_ROADMAP §3 S29 pour referencer le scope doc

### D5 — Scope disposition : Nym + MIG + outstanding items

**Retenu** : scope-cut Nym et MIG de S28 avec justification factuelle
G9.

1. **Nym mixnet** : deferred S30+ — SDK beta (`nym-sdk 1.27.0`,
   VALIDATED_BLUEPRINT rate CAUTION), latences 200-800ms
   incompatibles real-time, aucune fondation code (zero reference
   codebase), pas de transport SOCKS abstraction dans iroh 0.97.
   **HARDENING_ROADMAP sera mis a jour** : S28 line "Nym mixnet
   integration phase 1" → deferred S30+ avec note "G9 2026-04-25
   SDK beta, deferred post-Gate 3".

2. **MIG partitioning** : deferred post-v1.0 — MIG est une
   feature A100/H100 enterprise. RTX 5080 (dev machine) et les
   GPU consumer des futurs workers initiaux ne supportent pas MIG.
   Actionnable uniquement quand workers H100 existent (post-v1.0
   partnership ONG ou cloud compute). **HARDENING_ROADMAP mis a
   jour** idem.

3. **Outstanding scope cuts reportes** : Tor (arti pre-1.0), domain
   fronting (legal review), C4 task-scoped sandbox (design-only dans
   PROCESS_ARCHITECTURE.md, code S29), streaming bridge C5 (S29+),
   GPU lockup (dep A4 process roles → S29+), C1 SQLiteSession (S29+).

§Research consulte : G9 agent Explore Nym 2026-04-25 + G9 agent
Explore MIG 2026-04-25.

**Rejete** :
- **Nym S28** : beta SDK, latence, pas de critical path pre-v1.0.
- **MIG S28** : hardware mismatch. Pas actionnable.
- **Absorber Tor S28** : arti-client toujours pre-1.0 (trigger
  INACTIVE).

**Implications** :
- Update HARDENING_ROADMAP §3 S28 et S30 pour refleter les deferrals
- Pas de nouveau dep Nym/MIG ce sprint

### Acknowledged review findings (G1)

Scoring : D1 ✅, D2 ⚠️, D3 ⚠️, D4 ✅, D5 ⚠️.
Rigor signal G4 satisfait (3 ⚠️ sur 5).

**D2 ⚠️** : `tracing-journald` (Tokio ecosystem) non comparee.
Decision : adjust — `tracing-journald` est un subscriber layer qui
s'integre via le framework tracing (routing automatique). Notre
architecture utilise le trait `EventWriter` avec des appels explicites
`write_event()`, pas du routing subscriber implicite. `libsystemd` (FFI
directe `sd_journal_send`) s'aligne mieux sur l'architecture trait-based
existante. Les deux approches coexisteraient : TracingWriter → tracing
subscriber (cross-platform), JournaldWriter → libsystemd (natif Linux
explicite). Comparaison ajoutee au §Rejete D2.

**D3 ⚠️** : JSON-RPC vs gRPC sans analyse quantifiee. Decision :
acknowledge — la comparaison quantitative (latency UDS, overhead
serialization) est un deliverable du design doc PROCESS_ARCHITECTURE.md
(Phase C), pas du kickoff D-choice. Le kickoff pose le choix default
(JSON-RPC 2.0), le design doc le validera ou l'infirmera avec des
mesures. Si l'analyse Phase C montre que gRPC < 50% overhead, le choix
sera revisite.

**D5 ⚠️** : sources non-sourcees. Decision : adjust —
- VALIDATED_BLUEPRINT.md est un doc codebase (`docs/security/
  VALIDATED_BLUEPRINT.md`, consulte par G9 agent, verifiable par
  `grep -A 5 "nym-sdk" docs/security/VALIDATED_BLUEPRINT.md`).
- NVIDIA MIG = datacenter GPUs only (A100/H100/H200). Ref : NVIDIA MIG
  User Guide "MIG is supported on A100, A30, H100, and H200 GPUs".
  RTX 5080 = consumer GPU Ada Lovelace, pas de MIG.
- Alternative Rust-native mixnet : aucun client Rust mixnet production
  n'existe a la date 2026-04-25. dandelion++ est un protocole
  (Monero, pas une lib Rust standalone). `nym-sdk` Rust existe mais
  est le SDK officiel Nym (celui rate CAUTION). Pas d'alternative
  concurrente Rust-native viable.
- HARDENING_ROADMAP update acte Phase D.

---

## 5. Phase outline A..E

### Phase A — P2 batch S27 audit (4 items)

Absorbe les 4 P2 de l'audit gate S27. Code + docs :
- P2-B-1 : wire `compute_bias` dans `llama_cpp.rs` sampling loop +
  populate `output_token_ids` dans `runtime.rs`
- P2-B-2 : creer `configs/watermark.toml.sample`
- P2-C-1 : remplacer fingerprint dummy `000...` dans
  `configs/trust_web_seeds.toml` par vrai fingerprint Ed25519 FlowUP
- P2-D-1 : corriger P37 dans `docs/rust/PATTERNS.md` (watermark.rs
  pas llama_cpp.rs)

Commit cible : `feat(sprint28): Sprint 28 Phase A — watermark
end-to-end wiring + P2 batch S27 audit`

### Phase B — Phase dette (sprint pair obligatoire §6.2.1 Regle 1)

Items differes absorbes :
- SC-9 : Platform writers journald + oslog (remplacement stubs
  JournaldWriter/OsLogWriter dans nexus-events-core)
- SC-10 : ONNX CI fixture (mini model PII GLiNER pour Vitest)

Commit cible : `feat(sprint28): Sprint 28 Phase B — platform
writers journald/oslog + ONNX CI fixture`

### Phase C — Process isolation design doc

Livrable : `docs/security/PROCESS_ARCHITECTURE.md` — design doc
complet broker/executor split avec IPC boundary, executor lifecycle,
state ownership, cold-start budget, fault isolation.

Commit cible : `docs(sprint28): Sprint 28 Phase C — process isolation
PROCESS_ARCHITECTURE.md design doc`

### Phase D — External audit scope doc + HARDENING_ROADMAP update

Livrables :
- `docs/security/EXTERNAL_AUDIT_SCOPE.md` — scope matrix Cure53/ToB
- HARDENING_ROADMAP §3 S28 + S30 update (Nym/MIG deferrals)
- Gate 3 checklist update si applicable

Commit cible : `docs(sprint28): Sprint 28 Phase D — external audit
scope + HARDENING_ROADMAP update`

### Phase E — Wrap-up

Livrables :
- `sprint28_verification.md`
- `sprint29_audit_plan.md`
- `sprint28_carry_summary.md`
- Migration active → archive/v1.2/
- Updates CLAUDE.md, SPRINT_LOG, memory

Commit cible : `chore(sprint28): Phase E — wrap-up + verification +
audit plan S29 + migration`

---

## 6. Items carry/dette

### Carry S28 (absorbes)

| ID | Description | Source | Reports | Status S28 |
|---|---|---|---|---|
| P2-B-1 | Watermark injection non cablee sampling | S27 audit | 1/3 | Phase A |
| P2-B-2 | watermark.toml.sample absent | S27 audit | 1/3 | Phase A |
| P2-C-1 | Fingerprint seeds.toml dummy | S27 audit | 1/3 | Phase A |
| P2-D-1 | P37 chemin injector incorrect | S27 audit | 1/3 | Phase A |
| SC-9 | Platform writers journald/oslog | S27 scope-cut | 2/3 | Phase B |
| SC-10 | ONNX CI fixture | S22 carry | 5+/3 | Phase B |

### Hors cap — items long-terme

| ID | Description | Condition | Status |
|---|---|---|---|
| T-NN+2 | iframe Rust-wasm (PATTERNS §P34) | tract opset 19 / ort wasm32-browser / gline-rs wasm-bindgen | Triggers inactifs |
| LT-1 | Kudos-v2 fairness reform | v1.0 + design doc + Gini > 0.70 | Latent |
| LT-2 | Radicle activation | tag v1.0 | Latent |
| LT-3 | Contribution family Sybil matrix | v1.0 + 3+ contrib non-compute | Latent |
| LT-4 | OS biometric gate | v1.0 + S30 FROST N1 + partnership | Latent |
| LT-5 | Redundancy persistence SQLite | multi-worker deploy OR v1.0 | Latent |
| LT-6 | iroh neighborhood enrichment | iroh > 0.97 OR v1.0 | Latent |

### Check ROADMAP_COMMITMENTS (G7 Regle 3)

Resultat : toutes les conditions de declenchement sont latentes
(tag v1.0 non pose, iroh toujours 0.97, Gini non mesurable pre-prod,
pas de multi-worker deploy, pas de partnership ONG formelle). Aucun
item LT ne redevient carry actif.

---

## 7. Scope cuts — ce que Sprint 28 NE fait PAS

1. **Nym mixnet integration** → S30+ (SDK beta 200-800ms, G9 2026-04-25)
2. **MIG partitioning** → post-v1.0 (A100/H100 only, RTX 5080 dev)
3. **D2 broker/executor implementation** → S29 (design-only S28, cold-start benchmark prereq)
4. **D3 Windows RPC** → S29 (co-landing D2 + windows-rs maturite)
5. **C4 task-scoped sandbox code** → S29 (co-landing D2)
6. **Tor transport** → S30+ (arti pre-1.0)
7. **Arti library-embed** → S30+ (conditionnel arti >= 1.0)
8. **Domain fronting implementation** → S30+ (legal review prereq)
9. **GPU lockup defense** → S29+ (dep A4 process roles)
10. **C1 SQLiteSession abstraction** → S29+ (pas prioritaire)
11. **Streaming bridge C5** → S29+
12. **Full Gate 3 showcase app** → post-Gate 3

---

## 8. Tracabilite scope

Table mappant les items "What's NOT" S27 au sprint de prise en charge :

| Item S27 scope cut | Sprint cible | Phase |
|---|---|---|
| Tor transport → S28+ | S30+ | deferred (arti pre-1.0) |
| Arti library-embed → S28+ | S30+ | deferred |
| Domain fronting impl → S28+ | S30+ | deferred (legal) |
| GPU lockup → S28+ | S29+ | deferred (dep A4 process roles) |
| A4 process roles → S28 | S29 | D2 broker/executor split (S28 design-only) |
| C1 SQLiteSession → S28+ | S29+ | deferred |
| Ollama watermark → S28+ | post-API-hook | deferred (API limitation) |
| SynthID Tournament → S28+ | post-v1.0 | deferred (CDF complexe) |
| Platform writers → S28 | **S28 Phase B** | absorbed dette |
| ONNX CI fixture → S28 | **S28 Phase B** | absorbed dette |
| Streaming bridge C5 → S28+ | S29+ | deferred |
| Full Gate 3 showcase → post-Gate 3 | post-S29 | deferred |

---

## 9. Risk register

| ID | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R-S28-1 | Watermark wiring casse structured output llguidance (R-S27-4 carry) | MED | HIGH | Test integration logit bias + llguidance. Si conflit : bias OFF quand grammar active (fallback configurable). |
| R-S28-2 | Platform writers non testables sur Windows dev | LOW | LOW | Trait mock testing. Code correct par construction (cfg gate + crate wrapper mature). CI Linux/macOS futur. |
| R-S28-3 | ONNX CI fixture trop complexe (model conversion, size) | MED | LOW | Si > 100 LOC ou model > 5 MB : scope-cut a fixture dummy + TODO S29. Pas de blocker. |
| R-S28-4 | PROCESS_ARCHITECTURE.md design incomplet (missing benchmark data) | LOW | MED | Design doc speculative OK (benchmark = prereq S29 Phase D2, pas S28). Le doc identifie les unknowns. |

---

## 10. Audit gate pattern — rappel

- Phase 0 audit S27 DONE (PASS, `fbc63b3`).
- Phase E wrap-up produira `sprint28_verification.md` +
  `sprint29_audit_plan.md`.
- Phase 0 Sprint 29 (prochain audit) = audit independant de S28.

---

## 11. Checkpoint de validation

Questions pour arbitrage utilisateur AVANT plan detaille :

1. **D1** : Le wiring watermark dans llama_cpp.rs (P2-B-1 ~30-50 LOC)
   est-il le bon scope Phase A, ou faut-il aussi ajouter un test
   integration watermark+llguidance (R-S28-1) ?

2. **D2** : Platform writers via `libsystemd` + `oslog` crates,
   testes par trait mock sur Windows — est-ce acceptable malgre
   l'absence de test natif sur la machine dev ?

3. **D3** : PROCESS_ARCHITECTURE.md design-only sans code — est-ce
   suffisant pour S28, ou faut-il inclure un PoC broker/executor
   minimal ?

4. **D4** : Audit scope Cure53/ToB — l'user a-t-il une preference
   de vendor ou budget a respecter ?

5. **D5** : Nym + MIG scope-cuts — la justification factuelle G9
   est-elle convaincante, ou l'user souhaite-t-il une phase
   research Nym ce sprint ?
